"""Resolution and static checks (E §3.1 steps 2, 3, 5, 6, 11).

Every expression node gets `_t` (its static type, L §3) and, where a name is
involved, `_r` (what it resolves to, L §2.5).  The unit rules of L §3.2
(LOAD-UNIT), the literal rule of L §7.1 (LOAD-LITERAL), keys by position
(LOAD-KEY), bindings (LOAD-ALIVE, LOAD-DOMAIN, LOAD-SHADOW) are checked here,
and the item graph is built for LOAD-CYCLE, LOAD-GUARD, LOAD-LAWDATE,
LOAD-FORKS, WARN-UNREAD and readers-of.
"""
import datetime
from fractions import Fraction

from . import units as U
from . import ty as T
from .ty import Ty
from .errors import LoadError, Unspecified, NotYet
from .loader import resolve_key_literal, resolve_text
from .model import InputInfo, DefInfo, BandInfo, key_text, row_timeless
from .dates import Period
from .node import Node, walk
from .parser import KEYWORDS
from .values import CURRENCIES, Key, Thing, Code

WRITER2 = {'exp', 'ln', 'sqrt', 'periodic', 'factor', 'dcf', 'accrue', 'cdf', 'pdf', 'pmf'}
WRITER3 = set()
DATE_FNS = {'start_of', 'end_of', 'year_of', 'quarter_of', 'month_of', 'day_of', 'days_between', 'add_days',
            'add_months', 'add_years', 'overlap_days', 'age', 'age_months', 'weekday', 'pos',
            'is_business', 'add_business', 'last_business', 'schedule', 'window', 'one_of'}


def uerr(msg, loc):
    return LoadError('LOAD-UNIT', msg, loc)


class Var:
    def __init__(self, name, ty, dom=None, node=None, keylist=None):
        self.name = name
        self.ty = ty
        self.dom = dom
        self.node = node
        self.keylist = keylist     # for k in [..]: the key texts; its index is fixed by its first key position
        self.fixed = None


class Scope:
    def __init__(self, checker, item, regime, where, keyvars=()):
        self.c = checker
        self.item = item           # ('def', DefInfo) ('band', BandInfo) ('ask', node) ...
        self.regime = regime
        self.where = where         # 'def' 'ask' 'band' 'at'
        self.vars = {}
        self.keyvars = list(keyvars)
        self.guards = set()        # key variables a guard tests here
        self.agg_depth = 0
        self.cited = False         # inside a rule with v (missing())
        self.in_when = False
        self.lawpin = False        # inside as if {law D}
        self.allow_lit = False     # a figure allowed here (key offset, pos comparison, step)
        self.readmode = None       # 'calendar' or 'explicit': a read not at the law date
        self.dist_role = False     # a dist-typed read here is the distribution, not its sample (L §5.16)
        self.in_query = False      # inside a query over random values (L §5.16)
        self.covered_spans = set() # spans a covered() guard holds here (L §5.19 LOAD-COVERED)
        self.narrow = {}           # var -> (kind K, [used]): a kind test narrows it here (L §5.7, tally 2.1, R-17)

    def child(self):
        s = Scope(self.c, self.item, self.regime, self.where, self.keyvars)
        s.vars = dict(self.vars)
        s.guards = set(self.guards)
        s.agg_depth = self.agg_depth
        s.cited = self.cited
        s.in_when = self.in_when
        s.lawpin = self.lawpin
        s.readmode = self.readmode
        s.in_query = self.in_query
        s.covered_spans = set(self.covered_spans)
        s.narrow = dict(self.narrow)
        return s


class Checker:
    def __init__(self, m):
        self.m = m
        self.edges = {}            # reader item -> {read item: set of relations}
        self.shift_reads = []      # (reader, target, var, sign, guarded, loc)
        self.ask_reader_law = {}   # ask id -> needs law
        self.axes_of = {}          # item -> set of axes read directly
        self.year4_nodes = []      # four-digit literals whose reading (year or int) is decided later
        self.kl_uses = []          # key-list variables read as values, checked against the index they take
        self.member_readers = set()  # items that read a joint model's member (L §6.1): law-dated
        self.item_offsets = {}     # def -> {key variable: integer offsets of its key arithmetic} (L §7.1)
        self.at_reads = {}         # def id -> {item: relations} read by its `at e` (L §4.3)
        self.rand_edges = {}       # reader -> {item: roles}: dist, sample, value, query (L §5.16)
        self.queries = []          # (query or crps node, item, where) over random values
        self.draw_keys = []        # (key node, range end node, index id): a slot key a range up to a draw gives
        self.orient_items = set()  # items that hold crps or pdf (L §5.15)
        self.random_items = set()
        self.fill_infos = {}       # fill id -> its bindings' domains and taken's type (L §5.12)
        self.marks = {}            # item -> {(running read, target)} (E §3.1 step 7)
        self.proj_nc = {}          # item -> {target}: its projections with no random clause (L §8.3 item 10)
        self.cycle_cands = []      # (move, bal node, day node): a decision's plain bal at a day (L §5.14.1)
        self.period_cands = []     # (index, key node, holder item): a period read at a time index's key (L §4.6)
        self.balnow_items = set()  # items that hold bal(x) with no day
        self.projs = []            # (projection node, holder item)
        m.checker = self

    # -- graph -----------------------------------------------------------
    def edge(self, sc, target, rel='any'):
        r = sc.item
        self.edges.setdefault(r, {}).setdefault(target, set()).add(rel)

    def run(self):
        m = self.m
        # tally 2.1: the parts clauses (L §3.6, §5.7, LOAD-PARTS)
        self.check_parts()
        self.check_priced()
        # an input keyed by a stepped range (L §5.6): its range's ends and step are typed like a def's key binds; its
        # literals are a declaration's (L §7.1: date literals "in index declarations")
        for qid, inf in m.inputs.items():
            if any(b.dom[0] == 'range' for b in getattr(inf, 'binds', ())):
                sc = Scope(self, ('input', qid), getattr(inf, 'regime', None), 'decl', [b.var for b in inf.binds])
                for b in inf.binds:
                    sc.vars[b.var] = Var(b.var, self.dom_type(b.dom, b.node, sc), b.dom, b.node)
        for qid in sorted(m.moves):
            self.check_move(m.moves[qid])
        for qid in sorted(m.fills):
            self.check_fill(qid)
        for qid, d in m.defs.items():
            self.check_def(d)
        for qid, b in m.bands.items():
            self.check_band(b)
        top = self.top_scenario()
        if top is not None:
            # every ask is checked only in the worlds its line is evaluated in (L §6.4 The base is whole, §5.18): a
            # scenario's reality holds only the asks evaluated in it, those that name the scenario (cli evaluates
            # the others in the base's reality or another scenario's)
            m.asks = [a for a in m.asks if top in self.ask_worlds(a)]
        alone = []
        for a in m.asks:
            try:
                self.check_ask(a)
            except LoadError as e:
                # L §6.4, §8, §8.3 (round 19, GD-18): "an ask's LOAD-SCENARIO for a scenario whose file isn't loaded
                # refuses only that ask"; the base answers its other asks (E §11)
                if e.code == 'LOAD-SCENARIO' and 'neither the model nor a loaded scenario' in e.msg:
                    a.refused_alone = e
                    alone.append(a)
                    continue
                raise
        m.asks_alone = alone
        m.asks = [a for a in m.asks if a not in alone]
        self.check_prices()
        self.check_dist_rows()
        if getattr(m, 'scenario_over', None):
            self.check_scen_rows()
        # the literal rule on four-digit literals, by the reading each took (L §2.1, §7.1)
        for n in self.year4_nodes:
            if getattr(n, '_keylit', False):
                continue
            t = n._t
            if t is not None and t.k == 'period':
                self.lit_rule(n._litsc, n, 'date')
            elif t is not None and t.k in ('int', 'num'):
                self.lit_rule(n._litsc, n, 'num', n.v)
        self.kl_finish()
        self.inline_fill_edges()
        self.check_graph()
        self.check_moves_graph()
        self.parts_walk_bound()
        self.check_arrival_reads()
        self.check_after_readings()
        self.check_seen_days()
        self.check_period_keys()
        self.check_move_lawdates()
        self.finish_random()
        self.check_proj_seen()
        if getattr(m, 'scenario_over', None):
            self.check_srow_reads()
        self.check_scenario_reads()

    def item_scen(self, item):
        """The scenario whose file declares an item, or None (L §6.4)."""
        m = self.m
        if not isinstance(item, tuple) or len(item) < 2:
            return None
        kind, iid = item[0], item[1]
        if kind == 'srow':
            return item[3]
        if kind == 'moveon':
            # a move's day is the move's own (L §6.4: a scenario's move reads its scenario's items)
            kind = 'move'
        if kind == 'ask':
            for a in m.asks:
                if a.id == iid:
                    return getattr(a, '_scen', None)
            return None
        table = {'def': m.defs, 'input': m.inputs, 'move': m.moves, 'band': m.bands, 'place': m.places,
                 'kind': m.kinds, 'index': m.indexes, 'fill': getattr(m, 'fills', {})}.get(kind)
        if table is None:
            return None
        info = table.get(iid)
        if info is None:
            return None
        decl = getattr(info, 'decl', None)
        if decl is None and isinstance(info, dict):
            decl = info.get('decl')
        return getattr(decl, '_scen', None)

    def check_scenario_reads(self):
        """A read of an item declared in a scenario file is allowed only inside that scenario and the scenarios
        over it, and in diff (L §6.4, LOAD-SCENARIO)."""
        m = self.m
        if not getattr(m, 'scenario_over', None):
            return
        diffs = set(a.id for a in m.asks if a.form == 'diff')
        for r, ts in self.edges.items():
            if r[0] == 'ask' and r[1] in diffs:
                continue
            if not isinstance(r, tuple) or len(r) < 2:
                continue
            rs = self.item_scen(r)
            chain = set()
            ws = [rs]
            if r[0] == 'def' and r[1] in m.defs:
                # a rule a scenario's version brings (`replace rule x.l beats base = …`) is inside that scenario
                # (L §6.4): it reads the scenario's items
                for vers in m.defs[r[1]].rules.values():
                    for rv in vers:
                        if rv.version is not None:
                            ws.append(getattr(rv.version.node, '_scen', None))
            for w in ws:
                while w is not None:
                    chain.add(w)
                    w = m.scenario_over.get(w)
            for t in ts:
                if t[0] == 'srow':
                    # a scenario row's reads are its scenario's (L §6.4), not its target's
                    continue
                if ts[t] <= {'bind'}:
                    # L §6.4 (round 11b): "A read is by name: a base item evaluated in the scenario reaches the
                    # scenario's things through its bindings over their kind ..., which names none of them"
                    continue
                tsc = self.item_scen(t)
                if tsc is not None and tsc not in chain:
                    e = LoadError('LOAD-SCENARIO', '%s of scenario %s is read outside it (by %s)' % (t[1], tsc, r[1]))
                    # a reader in a base file: the base's refusal, the whole model's (L §8.3)
                    e.base_file = rs is None
                    raise e

    def top_scenario(self):
        """The scenario whose reality this is (the top of its stack), or None for the base's."""
        over = getattr(self.m, 'scenario_over', None) or {}
        if not over:
            return None
        below = set(v for v in over.values() if v is not None)
        tops = sorted(s for s in over if s not in below)
        return tops[0] if tops else None

    def ask_worlds(self, a):
        """The scenarios an ask's line is evaluated in: its file's, its `in`, a diff's worlds, `with scenario`, and
        `as if {scenario s}` in its line or a def it reaches (as cli.asif_worlds)."""
        from .node import walk as _walk
        out = set()
        if getattr(a, '_scen', None) is not None:
            out.add(a._scen)
        if a.world is not None:
            out.add(a.world)
        if a.form == 'diff':
            out |= set(w for w in a.diff[2:] if w is not None)
        for p in a.pins:
            if p.kind == 'scenario':
                out.add(p.id)

        def scan(node):
            if node is None:
                return
            for x in _walk(node):
                if getattr(x, 'k', None) == 'asif':
                    for p in x.pins or ():
                        if getattr(p, 'kind', None) == 'scenario':
                            out.add(p.id)
        scan(a)
        seen = set()
        stack = [('ask', a.id)]
        # before the ask is checked its own reads aren't in the graph: the defs its line names, then theirs
        for x in _walk(a):
            if getattr(x, 'k', None) in ('name', 'index', 'call'):
                xid = getattr(x, 'id', None) or getattr(x, 'name', None)
                if xid in self.m.defs:
                    stack.append(('def', xid))
        while stack:
            x = stack.pop()
            if x in seen:
                continue
            seen.add(x)
            if x[0] == 'def' and x[1] in self.m.defs:
                scan(getattr(self.m.defs[x[1]], 'decl', None))
            stack.extend(self.edges.get(x, {}).keys())
        return out

    def check_prices(self):
        """prices Y by idx: idx is an input or a def keyed by one time index with a key for the year Y (L §3.4)."""
        m = self.m
        for qid, inf in m.inputs.items():
            if getattr(inf, 'prices_by', None) is None:
                continue
            loc = inf.decl.loc if inf.decl is not None else None
            hit = m.lookup(inf.prices_by, inf.regime)
            if hit is None:
                raise LoadError('LOAD-NAME', 'unknown id %s' % inf.prices_by, loc)
            if hit[0] not in ('input', 'def') or len(hit[1].binds) != 1:
                raise LoadError('LOAD-KEY', 'prices by %s: not an input or a def keyed by one time index' %
                                inf.prices_by, loc)
            idx = hit[1]
            b = idx.binds[0]
            ix = m.indexes.get(b.dom[1]) if b.dom[0] == 'index' else None
            if ix is None or ix.pk is None:
                raise LoadError('LOAD-KEY', 'prices by %s: not keyed by a time index' % idx.id, loc)
            if not any(isinstance(k, Period) and k.kind == 'year' and k.label() == '%04d' % inf.prices
                       for k in ix.keys):
                raise LoadError('LOAD-KEY', 'prices %d by %s needs an index with a key for the year %d' % (
                    inf.prices, idx.id, inf.prices), loc)
            self.edges.setdefault(('input', qid), {}).setdefault((hit[0], idx.id), set()).add('any')

    # -- defs -------------------------------------------------------------
    def def_scope(self, d, where='def'):
        sc = Scope(self, ('def', d.id), d.regime, where, [b.var for b in d.binds])
        for b in d.binds:
            sc.vars[b.var] = Var(b.var, self.dom_type(b.dom, b.node, sc), b.dom, b.node)
        return sc

    def dom_type(self, dom, bnode, sc):
        k = dom[0]
        if k in ('index', 'kind', 'axis'):
            return T.key(dom[1])
        if k == 'list':
            return Ty('keylist')
        rng = dom[1]
        return self.range_type(rng, bnode, sc)

    def check_def(self, d):
        m = self.m
        # law date
        sc0 = self.def_scope(d, 'at')
        # a key bind's where is a condition over the key variables, read like the def's rules (L §2.3 bind, L §5.6)
        for b in d.binds:
            w = getattr(b.node, 'where', None) if b.node is not None else None
            if w is not None:
                t = self.tc(w, self.def_scope(d))
                if not self.is_bool(t):
                    raise uerr('where of a key bind of %s is %s, not bool' % (d.id, T.show(t)), d.loc)
        at = d.at
        if at is not None:
            before = {t: set(r) for t, r in self.edges.get(('def', d.id), {}).items()}
            if at.k == 'law_expr':
                t = self.tc(at.e, sc0)
                if t.k == 'period' or (t.k == 'key' and self.key_value_kind(t) == 'period'):
                    raise LoadError('LOAD-LAWPERIOD', 'at %s: a period is not a law date' % _txt(at.e), at.loc)
                if not self.is_date(t):
                    raise uerr('at of %s is %s, not a date' % (d.id, T.show(t)), at.loc)
            # the reads inside `at e` are made at the law date of the def's reader (L §4.3)
            after = self.edges.get(('def', d.id), {})
            self.at_reads[d.id] = {t: set(r) - before.get(t, set()) for t, r in after.items()
                                   if set(r) - before.get(t, set())}
        # rules
        labels = set(d.rules)
        beats = {}
        for lab, vers in d.rules.items():
            for rv in vers:
                for b in rv.node.beats:
                    if b not in labels:
                        raise LoadError('LOAD-LABEL', 'beats names no rule %s' % b, rv.loc)
                beats.setdefault(lab, set()).update(rv.node.beats)
        d.beats = beats
        # a beats cycle
        self._beats_cycle(d, beats)
        beaters = {lab: set() for lab in labels}
        for a, bs in beats.items():
            for b in bs:
                beaters[b].add(a)
        d.beaters = beaters
        when_vars = {}
        for lab, vers in d.rules.items():
            vs = set()
            for rv in vers:
                if rv.node.when is not None:
                    vs |= self.tested_vars(rv.node.when, set(b.var for b in d.binds))
            when_vars[lab] = vs
        # the integer offsets of the def's key arithmetic: the guard may hold the same integer (L §7.1)
        offs = {}
        vnames = set(b.var for b in d.binds)
        for lab, vers in d.rules.items():
            for rv in vers:
                for part in (rv.node.when, rv.node.value):
                    if part is None:
                        continue
                    for x in walk(part):
                        if x.k == 'bin' and x.op in '+-' and x.l.k == 'name' and x.l.id in vnames and \
                                x.r.k == 'num' and x.r.isint and not x.r.unit:
                            offs.setdefault(x.l.id, set()).add(int(x.r.v))
        self.item_offsets[('def', d.id)] = offs
        for lab, vers in d.rules.items():
            guard = set(when_vars[lab])
            for b in beaters[lab]:
                guard |= when_vars[b]
            for rv in vers:
                r = rv.node
                sc = self.def_scope(d)
                sc.cited = rv.v is not None
                sc.guards = set(guard)
                sc.covered_spans = self.covered_spans_of(r.when)
                # L §5.7 A kind test narrows (tally 2.1, R-17): "in a rule whose when is v in K, or a conjunction with
                # that test, throughout its value and its when"
                sc, nw = self.narrowed(sc, r.when)
                if r.when is not None:
                    sw = sc.child()
                    sw.in_when = True
                    t = self.tc(r.when, sw)
                    if not self.is_bool(t):
                        raise uerr('when of rule %s is %s, not bool' % (lab, T.show(t)), r.loc)
                val = r.value
                if val.k == 'hole':
                    val._t = T.HOLE
                elif val.k == 'solve':
                    if rv.v is None:
                        raise LoadError('LOAD-ASKONLY', 'solve outside an ask (allowed in a rule with v)', r.loc)
                    self.check_solve(val, sc, d.type, r.loc)
                elif d.type is not None and d.type.k == 'dist':
                    t = self.tc_dist(val, sc, expect=d.type)
                    self.assign(d.type, val, r.loc, 'rule %s of %s' % (lab, d.id))
                    self.dist_whole(val, r.loc, 'rule %s of %s' % (lab, d.id))
                else:
                    t = self.tc(val, sc, expect=d.type)
                    self.assign(d.type, val, r.loc, 'rule %s of %s' % (lab, d.id))
                self.narrow_mark(r.when, nw)
        for g in d.givens:
            pass

    # -- L §5.7 A kind test narrows (tally 2.1, R-17; occurrence typing) ----------------------------------------
    def narrow_tests(self, c):
        """The kind tests `v in K` (no on) that a condition is or holds as a conjunct; `or`, `not` and `on` narrow
        nothing (L §5.7: "or narrows to neither side, and x in K on d narrows nothing")."""
        if c is None:
            return []
        while c.k == 'paren':
            c = c.e
        if c.k == 'in' and c.on is None and c.l.k == 'name' and c.r.k == 'name':
            return [c]
        if c.k == 'and':
            return self.narrow_tests(c.l) + self.narrow_tests(c.r)
        return []

    def narrowed(self, sc, c):
        """A child scope in which each variable a kind test of c narrows to a subkind K of its bound kind reads K's
        legs and attributes; and the flags that say a narrowed field was read."""
        m = self.m
        out = None
        flags = []
        for t in self.narrow_tests(c):
            v = t.l.id
            var = sc.vars.get(v)
            if var is None or var.ty is None or var.ty.k != 'key' or var.ty.ix not in m.kinds:
                continue
            hit = m.lookup(t.r.id)
            if hit is None or hit[0] != 'kind' or hit[1] is None:
                continue
            K = hit[1].id
            P = sc.narrow[v][0] if v in sc.narrow else var.ty.ix
            if K == P or K not in m.kinds[P].subtree(m):
                continue
            if out is None:
                out = sc.child()
            f = [False]
            flags.append((t, f))
            out.narrow[v] = (K, f)
        return (out if out is not None else sc), flags

    def narrow_mark(self, c, flags):
        """L §5.7: "A conjunct that reads them is evaluated only where the test holds, and the conjunction is false
        where it doesn't": a conjunction whose narrowed fields are read evaluates its kind tests first (evaluate.e_and;
        a 2.0 text, which reads none, keeps its order)."""
        used = [t for t, f in flags if f[0]]
        if not used or c is None:
            return
        while c.k == 'paren':
            c = c.e
        if c.k == 'and':
            c._narrow = used

    def _beats_cycle(self, d, beats):
        state = {}

        def visit(x, path):
            st = state.get(x)
            if st == 1:
                raise LoadError('LOAD-CYCLE', 'a beats cycle in %s: %s' % (d.id, ' -> '.join(path + [x])))
            if st == 2:
                return
            state[x] = 1
            for y in sorted(beats.get(x, ())):
                visit(y, path + [x])
            state[x] = 2
        for x in sorted(beats):
            visit(x, [])

    def tested_vars(self, e, vars_):
        """The key variables a condition tests: a comparison that names k or pos(k) (L §5.11)."""
        out = set()
        for n in walk(e):
            if n.k in ('cmp', 'in'):
                for side in (n.l, n.r):
                    for x in walk(side):
                        if x.k == 'name' and x.id in vars_:
                            out.add(x.id)
        return out

    # -- bands ------------------------------------------------------------
    def check_band(self, b):
        for t in b.tables:
            for r in t.rows:
                sc = Scope(self, ('band', b.id), b.regime, 'band')
                sc.vars[b.decl.arg] = Var(b.decl.arg, b.argtype)
                if r.value.k == 'hole':
                    r.value._t = T.HOLE
                    continue
                self.tc(r.value, sc, expect=b.type)
                self.assign(b.type, r.value, r.loc, 'a row of band %s' % b.id)

    # -- asks -------------------------------------------------------------
    def check_ask(self, a):
        sc = Scope(self, ('ask', a.id), None, 'ask')
        worlds = getattr(self.m, 'all_worlds', {self.m.header.id})
        if a.world is not None and a.world not in worlds:
            # L §6.4: "an ask that names a scenario (its in, a diff, with scenario s, as if {scenario s}) loads only
            # with that scenario's file (LOAD-SCENARIO without it)"; L §8.3 (round 17) LOAD-SCENARIO `ask <q> names
            # <w>, neither the model nor a loaded scenario` (MC-B-F1)
            raise LoadError('LOAD-SCENARIO', 'ask %s names %s, neither the model nor a loaded scenario' % (
                a.id, a.world), a.loc)
        if getattr(a, '_scen', None) is not None and a.world is None and a.form != 'diff':
            raise LoadError('LOAD-WORLD', 'ask %s in a scenario file names no world' % a.id, a.loc)
        for b in a.binds:
            self.bind(b, sc, listing=True)
        has_law_pin = False
        for p in a.pins:
            self.pin(p, sc, ask=True)
            if p.kind == 'law':
                has_law_pin = True
        if a.law is not None:
            t = self.tc(a.law, sc)
            if not self.is_date(t):
                raise uerr('law of ask %s is %s, not a date' % (a.id, T.show(t)), a.loc)
        if a.known is not None:
            t = self.tc(a.known, sc)
            if not self.is_date(t):
                raise uerr('as known of ask %s is %s, not a date' % (a.id, T.show(t)), a.loc)
        if a.random is not None and a.random[0] == 'quantile':
            scq = sc.child()
            scq.allow_lit = True   # the level of a quantile (L §7.1 An ask)
            t = self.tc(a.random[1], scq)
        if a.windows == 'each':
            pass
        f = a.form
        if f == 'exprs':
            for e in a.exprs:
                self.tc(e, sc)
        elif f == 'check':
            l, r = a.check
            lt = self.tc(l, sc)
            rt = self.tc(r, sc, expect=lt)
            if lt.k in ('flex', 'keylit', 'year4'):
                self.tc(l, sc, expect=rt)
            self.compare('==', l, r, a.loc)
        elif f in ('explain', 'inputs-of'):
            self.tc(a.ref, sc)
            ok = ('name', 'index', 'field') + (('bal', 'low') if f == 'explain' else ())
            if a.ref.k not in ok:
                raise LoadError('LOAD-PARSE', 'cannot parse: %s takes a reference' % f, a.loc)
        elif f == 'readers-of':
            self.resolve_item_ref(a.ref, sc)
        elif f == 'solve':
            self.check_solve(a.solve, sc, None, a.loc)
        elif f == 'diff':
            e, bs, sw, bw = a.diff
            for w in (sw, bw):
                if w not in worlds:
                    raise LoadError('LOAD-SCENARIO', 'ask %s names %s, neither the model nor a loaded scenario' % (
                        a.id, w), a.loc)
            dsc = sc.child()
            for b in bs:
                self.bind(b, dsc, listing=True)
            self.tc(e, dsc)
        elif f == 'residuals':
            r, sp, by = a.resid
            t = self.tc(r, sc)
            if not self.is_place_type(t):
                raise uerr('residuals of %s, not a place' % T.show(t), a.loc)
            self.span(sp, sc)
        a._has_law = a.law is not None or has_law_pin

    def resolve_item_ref(self, ref, sc):
        if ref.k == 'name':
            hit = self.m.lookup(ref.id)
            if hit is None:
                raise LoadError('LOAD-NAME', 'unknown id %s' % ref.id, ref.loc)
            ref._r = ('item', hit[0], ref.id)
            return
        raise LoadError('LOAD-PARSE', 'cannot parse: readers-of takes an id', ref.loc)

    # -- pins ---------------------------------------------------------------
    def pin(self, p, sc, ask=False):
        m = self.m
        if sc.where == 'ask':
            sc = sc.child()
            sc.allow_lit = True    # an ask's pins hold literals (L §7.1 An ask); a def's pins don't
        if p.kind in ('law', 'known'):
            t = self.tc(p.value, sc)
            if not self.is_date(t):
                raise uerr('pin %s takes a date, not %s' % (p.kind, T.show(t)), p.loc)
            return
        if p.kind == 'wording':
            t = self.tc(p.value, sc)
            if not self.is_date(t):
                raise uerr('wording takes a date', p.loc)
            for i in p.ids:
                if '.' in i:
                    q, x = i.split('.', 1)
                    hit = m.qualified(q, x)
                else:
                    hit = m.lookup(i, sc.regime)
                if hit is None:
                    raise LoadError('LOAD-ASIF', 'pin on unknown id %s' % i, p.loc)
            return
        if p.kind == 'scenario':
            if p.id not in getattr(m, 'all_worlds', {m.header.id}):
                if sc.where == 'ask' and isinstance(sc.item, tuple) and sc.item[0] == 'ask':
                    # an ask's `with scenario s` or `as if {scenario s}` (L §6.4, §8.3 round 17)
                    raise LoadError('LOAD-SCENARIO', 'ask %s names %s, neither the model nor a loaded scenario' % (
                        sc.item[1], p.id), p.loc)
                # L §8.3 LOAD-ASIF `pin on unknown id <x>`
                raise LoadError('LOAD-ASIF', 'pin on unknown id %s' % p.id, p.loc)
            return
        tgt = p.target
        if p.sample:
            return self.sample_pin(p, sc)
        # resolve the target: an axis option, an input or a def (L §5.11 as if)
        name = tgt.id
        qual = tgt.qual
        fields = list(tgt.fields)
        hit = None
        if qual is not None:
            hit = m.qualified(qual, name)
        else:
            hit = m.lookup(name, sc.regime)
        if hit is None:
            raise LoadError('LOAD-ASIF', 'pin on unknown id %s' % name, p.loc)
        kind, info = hit
        if kind == 'axis':
            if tgt.keys is not None or fields:
                raise LoadError('LOAD-ASIF', 'an axis pin names its option', p.loc)
            v = p.value
            if v.k not in ('name', 'key') or resolve_text(v) not in info.options:
                raise LoadError('LOAD-KEY', '%s is not a key of %s' % (_txt(v), info.id), p.loc)
            p.axis = info.id
            p.option = resolve_text(v)
            p.pkind = 'axis'
            return
        if kind not in ('input', 'def'):
            raise LoadError('LOAD-ASIF', 'pin on %s, which is not an input, a def or an axis' % name, p.loc)
        p.pkind = kind
        p.info = info
        binds = info.binds
        keys = tgt.keys
        if keys is not None and len(keys) != len(binds):
            raise LoadError('LOAD-KEY', 'wrong number of keys for %s' % info.id, p.loc)
        if keys is None and binds:
            raise LoadError('LOAD-KEY', 'wrong number of keys for %s: write [*] to pin every key' % info.id, p.loc)
        p.keyconv = []
        for kn, b in zip(keys or [], binds):
            if kn.k == 'star':
                p.keyconv.append(None)
            else:
                p.keyconv.append(self.key_pos(kn, b.dom, sc))
        ty = info.type
        for f in fields:
            if ty is None or ty.k != 'record' or ty.field(f) is None:
                raise LoadError('LOAD-FIELD', '%s is not a field of %s' % (f, T.show(ty)), p.loc)
            ty = ty.field(f)
        p.fields = fields
        p.ty = ty
        # L §8.2 (round 11b): "Pins formed per binding tuple print once, as written, with their binding": the
        # aggregate bindings in scope whose variables the pin's keys or value name
        from .node import walk as _walk
        used = []
        for part in [p.value] + [kn for kn in (keys or []) if kn.k != 'star']:
            for nn in _walk(part):
                if nn.k == 'name' and nn.get('id') in sc.vars:
                    vb = sc.vars[nn.id].node
                    if getattr(vb, 'k', None) == 'bind' and getattr(vb, '_src', None) is not None and \
                            vb not in used:
                        used.append(vb)
        p._pertuple = used
        if self.pin_literal(p.value, ty, sc):
            self.edge(sc, (kind, info.id), 'pin')
            return
        if ty is not None and ty.k == 'dist':
            self.tc_dist(p.value, sc, expect=ty)
            self.dist_whole(p.value, p.loc, 'the pin of %s' % info.id)
        else:
            self.tc(p.value, sc, expect=ty)
        self.assign(ty, p.value, p.loc, 'the pin of %s' % info.id, pin=True)
        self.edge(sc, (kind, info.id), 'pin')

    def pin_literal(self, node, ty, sc):
        """A bare literal in a pin takes its target's declared type and unit (L §3.2 Inline figures: `with
        net_loss[b] = 2000` on a `money USD` def is 2 000.00 USD, never a year)."""
        if ty is None or ty.k not in ('money', 'num', 'int', 'rate'):
            return False
        chain = []
        inner = node
        sign = 1
        while inner.k in ('paren', 'neg'):
            chain.append(inner)
            if inner.k == 'neg':
                sign = -sign
            inner = inner.e
        if inner.k != 'num' or inner.unit:
            return False
        if inner.get('frac'):
            raise LoadError('LOAD-PARSE', 'cannot parse: a fraction is allowed only in rows', inner.loc)
        v = inner.v * sign
        self.lit_rule(sc, inner, 'num', v)
        if ty.k == 'rate':
            raise uerr('a bare figure as a rate: a rate states its period (L §3.3)', inner.loc)
        if ty.k == 'int' and Fraction(v).denominator != 1:
            raise uerr('%s is not an int' % _txt(node), inner.loc)
        if ty.k == 'money' and (Fraction(v) / CURRENCIES[ty.cur]).denominator != 1:
            raise LoadError('LOAD-PRECISION', '%s is finer than the minor unit' % _txt(node), inner.loc)
        inner._val = inner.v
        inner._u = ()
        inner._flexto = ty
        inner._t = T.flex(inner.v)
        for c in chain:
            c._t = T.flex(inner.v)
        return True

    # -- literals and the literal rule (L §7.1) ------------------------------------
    def lit_rule(self, sc, node, what, value=None):
        if sc.where in ('def', 'at'):
            if value is not None and value in (0, 1, -1):
                return
            if sc.allow_lit:
                return
            if what == 'date':
                raise LoadError('LOAD-LITERAL', 'a date in a rule; make it a version', node.loc)
            raise LoadError('LOAD-LITERAL', 'figure %s in a rule; make it a row' % _txt(node), node.loc)
        if sc.where == 'ask' and what != 'date' and not sc.allow_lit and (value is None or value not in (0, 1, -1)):
            # an ask's own literals: pins, a stepped range's ends and step, a quantile level, a round step,
            # dates and periods (L §7.1 An ask, §7.2)
            raise LoadError('LOAD-LITERAL', 'figure %s in an ask; make it a row' % _txt(node), node.loc)

    def tc(self, n, sc, expect=None):
        t = self._tc(n, sc, expect)
        n._t = t
        return t

    def _tc(self, n, sc, expect):
        k = n.k
        fn = getattr(self, 't_' + k, None)
        if fn is None:
            raise LoadError('LOAD-PARSE', 'cannot parse: %s here' % k, n.loc)
        return fn(n, sc, expect)

    def t_paren(self, n, sc, expect):
        return self.tc(n.e, sc, expect)

    def t_num(self, n, sc, expect):
        if n.unit:
            u, f = U.from_syntax(self.unit_terms(n.unit, n.loc))
            v = n.v * f
            # the inline figures are 0, 1 and -1 as written, with or without a unit: `1 %` (0.01), `100 %` (1)
            # (L §7.1)
            self.lit_rule(sc, n, 'num', v if v in (0, 1, -1) else n.v)
            n._val = v
            n._u = u
            if U.is_currency(u):
                cur = u[0][0]
                if cur not in CURRENCIES:
                    raise uerr('money in an undeclared currency %s' % cur, n.loc)
                if (v / CURRENCIES[cur]).denominator != 1:
                    raise LoadError('LOAD-PRECISION', '%s is finer than the minor unit' % n.text, n.loc)
                return T.money(cur)
            if n.unit == (('%', 1),):
                return T.pct()
            if expect is not None and expect.k == 'rate' and U.period_of(u) is not None:
                # a figure with a period unit where a rate is expected is that rate (L §3.3: `21 %/year`)
                return T.RATE
            return T.num(u)
        n._val = n.v
        n._u = ()
        if n.get('frac'):
            raise LoadError('LOAD-PARSE', 'cannot parse: a fraction is allowed only in rows', n.loc)
        if expect is not None and expect.k == 'key' and n.isint and expect.ix in self.m.indexes and \
                self.m.indexes[expect.ix].isint:
            # a key literal of an integer index, compared with a key (`k <= 10` on an integer index, L §7.1)
            n._r = ('key', resolve_key_literal(self.m, n, ('index', expect.ix), n.loc))
            return expect
        if sc.where == 'band' and not (n.isint and n.get('four') and expect is not None and expect.k == 'period'):
            # a bare literal in a band row takes its unit as a bare 0 does in a def (L §7.1, §5.9)
            return T.flex(n.v)
        if n.isint and n.v in (0, 1, -1):
            return T.flex(int(n.v))
        if n.isint and n.get('four'):
            # a year or an integer (L §2.1): the literal rule is applied to the reading taken
            if expect is not None:
                if expect.k == 'period' and expect.pk == 'year':
                    self.lit_rule(sc, n, 'date')
                    n._year = True
                    return T.period('year')
                if expect.k in ('int', 'num'):
                    self.lit_rule(sc, n, 'num', n.v)
                    return T.INT
            n._litsc = sc
            self.year4_nodes.append(n)
            return T.YEAR4
        self.lit_rule(sc, n, 'num', n.v)
        if n.isint:
            return T.INT
        return T.num(())

    def unit_terms(self, terms, loc):
        out = []
        for b, e in terms:
            if b == '%' or b in CURRENCIES or b in self.m.units or b in ('year', 'quarter', 'month', 'day') \
                    or b in self.m.unit_kind_keys:
                out.append((b, e))
            elif len(b) == 3 and b.isupper():
                raise uerr('money in an undeclared currency %s' % b, loc)
            else:
                raise LoadError('LOAD-NAME', 'unknown id %s (a unit)' % b, loc)
        return out

    def t_date(self, n, sc, expect):
        self.lit_rule(sc, n, 'date')
        return T.DATE

    def t_month(self, n, sc, expect):
        self.lit_rule(sc, n, 'date')
        return T.period('month')

    def t_quarter(self, n, sc, expect):
        # L §2.1 a quarter literal `2026-Q1`; a date literal for the literal rule (L §7.1 "months, quarters or years
        # used as dates or keys")
        self.lit_rule(sc, n, 'date')
        return T.period('quarter')

    def t_mmdd(self, n, sc, expect):
        from .values import monthday_ok
        if not monthday_ok(*n.v):
            # L §3.1 monthday (GD-2): "02-29 is no month-day, since not every year has it (refused, LOAD-PARSE)"
            raise LoadError('LOAD-PARSE', 'cannot parse: %s is not a month-day every year has' % n.text, n.loc)
        # L §7.1 (round 19): "Month-day literals (04-30, §3.1) are date literals here: they appear only in rows ...,
        # pins and asks, and a rule can't hold one (LOAD-LITERAL)"
        self.lit_rule(sc, n, 'date')
        return T.MD

    def t_key(self, n, sc, expect):
        return self.keylit(n, sc, expect)

    def t_cur(self, n, sc, expect):
        n._r = ('key', Thing(n.v))
        if n.v not in CURRENCIES:
            raise uerr('money in an undeclared currency %s' % n.v, n.loc)
        return T.key('currencies')

    def t_bool(self, n, sc, expect):
        return T.BOOL

    def t_str(self, n, sc, expect):
        if sc.where in ('def', 'at'):
            raise LoadError('LOAD-LITERAL', 'a text literal in a rule; make it a row', n.loc)
        return T.TEXT

    def t_none(self, n, sc, expect):
        return T.NONE

    def t_never(self, n, sc, expect):
        return T.NEVER

    def t_cut(self, n, sc, expect):
        return T.DATE

    def t_inf(self, n, sc, expect):
        raise LoadError('LOAD-PARSE', 'cannot parse: inf stands only in an interval', n.loc)

    def t_code(self, n, sc, expect):
        # "the integer of a code (nominal 12, §2.2): part of the code's name, not a figure" is allowed inline
        # (L §7.1)
        n._r = ('code', Code('compounding', 'nominal', n.v[1]))
        return T.code('compounding')

    def t_hole(self, n, sc, expect):
        return T.HOLE

    def t_record(self, n, sc, expect):
        fs = []
        for f, v in n.fields:
            if f == 'leg':
                # L §3.1 (round 12): "A leg is a field only of an observed row (§5.19), never of a record (LOAD-FIELD)"
                raise LoadError('LOAD-FIELD', 'leg %s: a leg is a field only of an observed row, not of a record'
                                % v.name, n.loc)
            ft = expect.field(f) if expect is not None and expect.k == 'record' else None
            if v.k == 'hole':
                v._t = T.HOLE
                fs.append((f, ft or T.HOLE))
                continue
            t = self.tc(v, sc, expect=ft)
            fs.append((f, t))
        return T.record(fs)

    def t_list(self, n, sc, expect):
        for it in n.items:
            self.tc(it, sc)
        if all(it.k == 'date' for it in n.items):
            return T.DATES if n.items else Ty('list', elem=None)
        return Ty('list', elem=None)

    # -- keys ----------------------------------------------------------------------
    def keylit(self, n, sc, expect):
        """A key literal: resolved by the type its position expects (L §2.5)."""
        text = resolve_text(n)
        if expect is not None:
            if expect.k == 'key':
                n._r = ('key', self.resolve_key(n, expect.ix, sc))
                return expect
            if expect.k == 'code':
                if text in T.CODE_WORDS[expect.code] and text != 'nominal':
                    n._r = ('code', Code(expect.code, text))
                    return expect
                raise uerr('%s is not a %s code' % (text, expect.code), n.loc)
        n._r = ('keylit', text)
        return Ty('keylit', n=text)

    def resolve_key(self, n, ix, sc):
        dom = self.dom_of(ix)
        if n.k == 'name' and n.id in sc.vars:
            raise LoadError('LOAD-KEY', '%s is a bound variable, not a key' % n.id, n.loc)
        if n.k in ('date', 'month', 'quarter'):
            self.lit_rule(sc, n, 'date')
        return resolve_key_literal(self.m, n, dom, n.loc)

    def dom_of(self, ix):
        m = self.m
        if ix in m.indexes:
            return ('index', ix)
        if ix in m.kinds:
            return ('kind', ix)
        if ix in m.axes:
            return ('axis', ix)
        raise LoadError('LOAD-NAME', 'unknown id %s' % ix, None)

    def key_value_kind(self, t):
        """What a key of type t is as a value: 'int', 'period', 'date', or None."""
        ix = t.ix
        m = self.m
        if ix in m.indexes:
            x = m.indexes[ix]
            if x.isint:
                return 'int'
            if x.pk == 'day' and (not x.keys or isinstance(x.keys[0], datetime.date)):
                return 'date'
            if x.pk is not None:
                return 'period'
        return None

    def unordered_key(self, t):
        """What a key type is when it has no order a rule can read (L §5.7 things, §5.6 an open index's keys), else
        None: an order on it is refused, at load where the types show it (L §8.4, LOAD-UNIT)."""
        if t is None or t.k != 'key':
            return None
        m = self.m
        if t.ix in m.kinds:
            return 'things of kind %s' % t.ix
        ix = m.indexes.get(t.ix)
        if ix is not None and ix.form == 'open':
            return 'keys of the open index %s' % t.ix
        return None

    def no_order(self, t, what, loc):
        if t is not None and t.k == 'mmdd':
            # L §3.1 monthday, §8.4 (GD-2, GD-23): month-days compare with == and != only; an order on them (`<`, min,
            # max) is an operation the type lacks, LOAD-UNIT
            raise uerr('%s orders month-days: they compare with == and != only' % what, loc)
        u = self.unordered_key(t)
        if u is not None:
            raise uerr('%s on %s has no meaning: they have no order a rule can read' % (what, u), loc)

    def key_pk(self, t):
        ix = self.m.indexes.get(t.ix)
        if ix is None:
            return None
        return ix.pk

    def key_pos(self, kn, dom, sc):
        """Type a key expression at a key position whose domain is dom; returns the
        conversion the evaluator applies (L §2.5, §4.2 Keys as values, §4.6 Reading at a date)."""
        m = self.m
        dk = dom[0]
        # an axis id in a key position reads the current option and fans out (L §5.5)
        if kn.k == 'name' and kn.id not in sc.vars and dk == 'axis' and kn.id == dom[1]:
            kn._t = T.key(dom[1])
            kn._r = ('axis', dom[1])
            self.axes_of.setdefault(sc.item, set()).add(dom[1])
            return ('axis', dom[1])
        if kn.k == 'name' and kn.id not in sc.vars and kn.id in m.axes and dk == 'axis':
            raise LoadError('LOAD-KEY', '%s is used as a key of %s' % (kn.id, dom[1]), kn.loc)
        # a date or period literal of another granularity at a time key reads the key whose period holds it
        # (L §4.6 Reading at a date)
        if dk == 'index' and m.indexes[dom[1]].pk is not None and kn.k in ('date', 'month', 'quarter'):
            gran = {'date': 'day', 'month': 'month', 'quarter': 'quarter'}[kn.k]
            if gran != m.indexes[dom[1]].pk:
                t = self.tc(kn, sc)
                return self.key_conv(t, dom, kn, sc)
        if dk == 'index' and m.indexes[dom[1]].pk not in (None, 'year') and kn.k == 'num' and kn.get('four'):
            t = self.tc(kn, sc, expect=T.period('year'))
            return self.key_conv(t, dom, kn, sc)
        if self.is_keylit_node(kn, sc):
            if dk in ('index', 'kind', 'axis'):
                if kn.k in ('date', 'month', 'quarter') or (kn.k == 'num' and kn.get('four') and dk == 'index'
                                                 and m.indexes[dom[1]].pk is not None):
                    self.lit_rule(sc, kn, 'date')
                v = resolve_key_literal(m, kn, dom, kn.loc)
                kn._r = ('key', v)
                kn._t = T.key(dom[1])
                return ('lit', v)
            if dk == 'list':
                v = resolve_key_literal(m, kn, dom, kn.loc)
                kn._r = ('key', v)
                kn._t = Ty('keylist')
                return ('lit', v)
            # L §2.5: "At a position keyed by a stepped range (§5.6), a key is a value of the range's type, written
            # or computed, and must be one of the range's values ... (LOAD-KEY for a literal, RUN-KEY at run)"
            kn._rangelit = True
        sc2 = sc.child()
        if (kn.k == 'bin' and kn.op in ('+', '-')) or getattr(kn, '_rangelit', False):
            sc2.allow_lit = True
        t = self.tc(kn, sc2, expect=(T.key(dom[1]) if dk in ('index', 'kind', 'axis') else None))
        if dk == 'index' and kn.k == 'name' and kn.id in sc.vars:
            v = sc.vars[kn.id]
            rng = getattr(v.node, 'dom', None) if v.node is not None else None
            if v.dom is not None and v.dom[0] == 'range' and rng is not None and getattr(rng, 'b', None) is not None:
                # a random number of items: slots read by a range that ends at the draw (L §5.16), checked once the
                # random cone is known
                self.draw_keys.append((kn, rng.b, dom[1]))
        return self.key_conv(t, dom, kn, sc)

    def is_keylit_node(self, kn, sc):
        if kn.k == 'name':
            if kn.id in sc.vars:
                return False
            hit = self.m.lookup(kn.id, sc.regime)
            return hit is None
        return kn.k in ('key', 'date', 'month', 'mmdd', 'cur') or (kn.k == 'num' and kn.unit is None and kn.isint)

    def key_conv(self, t, dom, kn, sc):
        m = self.m
        dk = dom[0]
        if t.k != 'keylist' and dk == 'index' and getattr(kn, '_klvar', None) is not None:
            # a key list of days or months (typed as dates or months where it is a value) at a key position: its keys
            # are key literals, resolved by that position (L §2.5 Bare names: "A bound variable over a key list ...
            # takes its index from the key positions it is used in"; Keys: "A key literal is resolved by its
            # position"), so over an anchored index 2022-06 is the key whose period starts in June 2022 (L §4.1;
            # MC-M-F1)
            t = Ty('keylist')
        if t.k == 'keylist':
            v = self.var_of(kn, sc)
            if v is not None:
                if v.fixed is None:
                    v.fixed = dom
                elif v.fixed != dom:
                    raise LoadError('LOAD-KEY', '%s is used as a key of %s and %s' % (v.name, v.fixed[1], dom[1]),
                                    kn.loc)
                for txt in v.keylist:
                    resolve_key_literal(m, txt, dom, kn.loc)
            return ('keylist', dom)
        if dk == 'index':
            ix = m.indexes[dom[1]]
            if t.k == 'key':
                if t.ix == ix.id:
                    return ('same',)
                if t.ix in m.indexes:
                    other = m.indexes[t.ix]
                    if ix.isint and other.isint:
                        return ('int',)
                    if ix.pk is not None and other.pk is not None:
                        return self.period_conv(ix, other.pk, kn, sc)
                raise LoadError('LOAD-KEY', '%s is used as a key of %s and %s' % (_txt(kn), t.ix, ix.id), kn.loc)
            if ix.isint and t.k in ('int', 'flex', 'year4'):
                return ('int',)
            if ix.isint and t.k == 'num' and not t.u:
                return ('int',)
            if ix.pk is not None:
                if t.k in ('date', 'never'):
                    return ('holding',)
                if t.k == 'period':
                    return self.period_conv(ix, t.pk, kn, sc)
            raise LoadError('LOAD-KEY', '%s (%s) is not a key of %s' % (_txt(kn), T.show(t), ix.id), kn.loc)
        if dk == 'kind':
            if t.k == 'key' and t.ix in m.kinds:
                a, b = m.kinds[t.ix], m.kinds[dom[1]]
                if t.ix in b.subtree(m) or dom[1] in a.subtree(m):
                    return ('thing', dom[1])
            if t.k == 'unitkey' and (m.kinds[dom[1]].spec == 'unit' or dom[1] == 'currencies'):
                # unit(x) where load can't see the key: read per value (L §5.10 Kind and unit, §3.2), the key
                # checked at run
                return ('thing', dom[1])
            raise LoadError('LOAD-KEY', '%s (%s) is not a key of %s' % (_txt(kn), T.show(t), dom[1]), kn.loc)
        if dk == 'axis':
            if t.k == 'key' and t.ix == dom[1]:
                return ('same',)
            raise LoadError('LOAD-KEY', '%s (%s) is not a key of %s' % (_txt(kn), T.show(t), dom[1]), kn.loc)
        if dk == 'list':
            return ('same',)
        # a stepped range
        return ('value',)

    def period_conv(self, ix, pk, kn, sc=None):
        order = {'day': 0, 'month': 1, 'quarter': 2, 'year': 3}
        if order.get(pk, 9) > order.get(ix.pk, 9):
            raise LoadError('LOAD-KEY', 'a %s holds no key of %s, whose keys are %ss' % (pk, ix.id, ix.pk), kn.loc)
        if sc is not None:
            # GD-27: a period load sees, read across two keys, is refused at load (check3.m_check_period_keys)
            self.period_cands.append((ix, kn, sc.item))
        return ('period',)

    def var_of(self, n, sc):
        if n.k == 'name' and n.id in sc.vars:
            return sc.vars[n.id]
        return None

    # -- names ---------------------------------------------------------------------
    def t_name(self, n, sc, expect):
        m = self.m
        bw = getattr(n, '_world', None)
        if bw is not None:
            # a base item named with the base's id as a prefix, in a scenario row (L §6.4)
            hit = m.lookup(n.id, sc.regime)
            if hit is None:
                raise LoadError('LOAD-NAME', 'unknown id %s.%s' % (bw[2], n.id), n.loc)
            tsc = self.item_scen((hit[0], hit[1].id if hasattr(hit[1], 'id') else n.id))
            if tsc is not None and tsc not in bw[3]:
                raise LoadError('LOAD-SCENARIO', '%s.%s: %s is declared in scenario %s, not in the base' % (
                    bw[2], n.id, n.id, tsc), n.loc)
            if hit[0] not in ('input', 'def', 'band'):
                # L §6.4: "It reads inputs, bands and defs, never a place or a movement (a balance, flows, a
                # move's instance), directly or through a def (LOAD-SCENARIO)"
                raise LoadError('LOAD-SCENARIO', '%s.%s: an expression row reads inputs, bands and defs, never a %s'
                                % (bw[2], n.id, hit[0]), n.loc)
        if n.id in sc.vars:
            v = sc.vars[n.id]
            n._r = ('var', n.id)
            if v.ty is not None and v.ty.k == 'keylist':
                n._klvar = v
                if v.keylist and all(kn.k == 'date' for kn in v.keylist):
                    # a key list of days: a key of a day index is a date (L §4.2 Keys as values; L §2.5 a key-list
                    # variable takes its index from its key positions, and a date is read at a day key, §4.6)
                    return T.DATE
                if v.keylist and all(kn.k == 'month' for kn in v.keylist):
                    return T.period('month')
                if v.keylist and all(kn.k == 'quarter' for kn in v.keylist):
                    return T.period('quarter')
            return v.ty
        hit = m.lookup(n.id, sc.regime)
        if hit is None:
            return self.keylit(n, sc, expect)
        kind, info = hit
        if kind in ('input', 'def'):
            return self.read(n, kind, info, [], sc)
        if kind == 'axis':
            n._r = ('axisval', n.id)
            self.axes_of.setdefault(sc.item, set()).add(n.id)
            self.edge(sc, ('axis', n.id))
            return T.key(n.id)
        if kind == 'cut':
            n._r = ('cut',)
            return T.DATE
        if kind in ('index', 'kind'):
            raise LoadError('LOAD-PARSE', 'cannot parse: %s %s stands only in a binding or a membership test' % (
                kind, n.id), n.loc)
        if kind == 'band':
            raise LoadError('LOAD-PARSE', 'cannot parse: band %s is called with ( )' % n.id, n.loc)
        if kind == 'place':
            return self.place_value(n, info, [], sc)
        if kind == 'move':
            return self.move_value(n, info, [], sc)
        if kind == 'fill':
            # L §5.12 Outputs: "absorb.taken[supply keys, demand keys]" is a fill's one output; a fill read otherwise
            # is no value
            raise LoadError('LOAD-PARSE', 'cannot parse: fill %s is read as %s.taken[...]' % (n.id, n.id), n.loc)
        if kind in ('version', 'source', 'unit', 'ask'):
            raise LoadError('LOAD-PARSE', 'cannot parse: %s %s is not a value' % (kind, n.id), n.loc)
        raise LoadError('LOAD-NAME', 'unknown id %s' % n.id, n.loc)

    def read(self, n, kind, info, keys, sc):
        """A read of an input or def at keys (or a fill's output, writer 2)."""
        binds = info.binds
        if len(keys) != len(binds) and not (kind == 'input' and info.observed):
            raise LoadError('LOAD-KEY', 'wrong number of keys for %s: %d, not %d' % (info.id, len(keys), len(binds)),
                            n.loc)
        if kind == 'input' and info.observed:
            # observation rows are rows keyed by their ids (L §5.19): o[r1]
            if len(keys) != 1:
                raise LoadError('LOAD-KEY', 'an observation is read by its id: %s[<id>]' % info.id, n.loc)
            kn = keys[0]
            if not self.is_keylit_node(kn, sc):
                raise LoadError('LOAD-KEY', 'an observation is read by its id: %s[<id>]' % info.id, n.loc)
            n._r = ('obsrow', info.id, resolve_text(kn))
            self.edge(sc, ('input', info.id), 'any')
            return T.key('events')
        convs = []
        env = {}
        for kn, b in zip(keys, binds):
            if kn.k == 'star':
                raise LoadError('LOAD-PARSE', 'cannot parse: * stands only in a pin', kn.loc)
            c = self.key_pos(kn, b.dom, sc)
            convs.append(c)
            if c[0] == 'lit':
                env[b.var] = key_text(c[1])
            elif kn.k == 'name' and kn.id in sc.vars:
                env[b.var] = '$' + kn.id
            else:
                env[b.var] = '$?'
        n._r = ('read', kind, info.id)
        n._convs = convs
        ty = info.type
        if kind == 'input' and info.prices is not None and info.prices_by is not None:
            ty = T.num(ty.u)
        rel = self.relation(sc, keys)
        if sc.readmode is not None:
            rel = sc.readmode
        elif sc.lawpin:
            rel = 'lawpin'
        self.edge(sc, (kind, info.id), rel)
        if rel.startswith('shift'):
            var, sign = rel.split(':')[1], rel.split(':')[2]
            self.shift_reads.append((sc.item, (kind, info.id), var, sign, var in sc.guards, n.loc))
        if kind == 'def':
            pass
        if ty is not None and ty.k in ('num', 'dist') and any(b.startswith('$') for b, e in ty.u):
            ty = Ty(ty.k, u=U.subst(ty.u, env))
        if ty is not None and ty.k == 'dist':
            return self.dist_read_type(n, kind, info, ty, sc)
        self.note_read(sc, kind, info)
        return ty

    def relation(self, sc, keys):
        """How a read's keys relate to the reader's (E §3.1 step 5)."""
        if sc.item[0] not in ('def', 'fill'):
            return 'any'
        if sc.agg_depth:
            # a read under an aggregate's binding is at any key
            names = [k.id for k in keys if k.k == 'name']
            if any(k.k == 'name' and k.id in sc.vars and k.id not in sc.keyvars for k in keys):
                return 'computed'
        kv = sc.keyvars
        shift = None
        for k in keys:
            if k.k == 'name' and k.id in kv:
                continue
            if k.k == 'bin' and k.op in ('+', '-') and k.l.k == 'name' and k.l.id in kv and k.r.k == 'num':
                if shift is not None:
                    return 'computed'
                shift = (k.l.id, '+' if k.op == '+' else '-')
                continue
            if k.k == 'paren':
                return 'computed'
            return 'computed'
        if shift is not None:
            return 'shift:%s:%s' % shift
        return 'same'

    def t_index(self, n, sc, expect):
        obj = n.obj
        m = self.m
        if obj.k == 'name' and obj.id not in sc.vars:
            hit = m.lookup(obj.id, sc.regime)
            if hit is None:
                raise LoadError('LOAD-NAME', 'unknown id %s' % obj.id, obj.loc)
            kind, info = hit
            if kind in ('input', 'def'):
                return self.read(n, kind, info, n.keys, sc)
            if kind == 'place':
                return self.place_value(n, info, n.keys, sc)
            if kind == 'move':
                return self.move_value(n, info, n.keys, sc)
            raise LoadError('LOAD-PARSE', 'cannot parse: %s %s is not read with keys' % (kind, obj.id), n.loc)
        if obj.k == 'field' and obj.obj.k == 'name' and obj.obj.id in getattr(m, 'joint_members', {}) and \
                obj.obj.id not in sc.vars:
            return self.member_read(n, obj.obj.id, obj.name, n.keys, sc)
        if obj.k == 'field' and self.fill_ref(obj, sc) is not None:
            return self.fill_read_type(n, sc, obj, n.keys)
        if obj.k == 'field':
            raise LoadError('LOAD-PARSE', 'cannot parse: %s is not read with keys' % _txt(obj), n.loc)
        raise LoadError('LOAD-PARSE', 'cannot parse: %s is not read with keys' % _txt(obj), n.loc)

    def member_read(self, n, alias, name, keys, sc):
        """A joint model reads a member's item as a.item (L §6.1 Joint models): typed by the member, its keys the
        member's keys."""
        mm = self.m.joint_members[alias]
        hit = mm.lookup(name)
        if hit is None:
            raise LoadError('LOAD-NAME', 'unknown id %s.%s' % (alias, name), n.loc)
        kind, info = hit
        if kind == 'ask':
            # nothing reads an ask (P-12): a member's ask is asked as a.q, never read (L §6.1), as an ask of the model
            # itself is no value
            raise LoadError('LOAD-PARSE', 'cannot parse: ask %s.%s is not a value' % (alias, name), n.loc)
        if kind not in ('input', 'def'):
            raise NotYet('a member\'s %s read by the joint model (%s.%s; consolidation of places and movements)'
                         % (kind, alias, name), n.loc)
        if len(keys) != len(info.binds):
            raise LoadError('LOAD-KEY', 'wrong number of keys for %s.%s: %d, not %d' % (alias, name, len(keys),
                                                                                       len(info.binds)), n.loc)
        if kind == 'input' and info.observed:
            raise NotYet('a member\'s observed input read by the joint model (%s.%s)' % (alias, name), n.loc)
        convs = []
        for kn, b in zip(keys, info.binds):
            if kn.k == 'star':
                raise LoadError('LOAD-PARSE', 'cannot parse: * stands only in a pin', kn.loc)
            if self.is_keylit_node(kn, sc):
                from .loader import resolve_key_literal
                v = resolve_key_literal(mm, kn, b.dom, kn.loc)
                kn._r = ('key', v)
                kn._t = T.key(b.dom[1]) if b.dom[0] in ('index', 'kind', 'axis') else None
                convs.append(('lit', v))
                continue
            t = self.tc(kn, sc)
            if t.k == 'key' and t.ix in self.m.indexes:
                # a key of a period or integer index is also that period or int (L §4.2 Keys as values)
                ix = self.m.indexes[t.ix]
                if ix.pk is not None:
                    t = T.period(ix.pk)
                elif ix.isint:
                    t = T.INT
            if t.k == 'key' or b.dom[0] not in ('index', 'range'):
                raise NotYet('a key of %s.%s given by a %s of the joint model (keys of two models are matched '
                             'only as periods or integers)' % (alias, name, T.show(t)), kn.loc)
            convs.append(mm.checker.key_conv(t, b.dom, kn, sc))
        n._r = ('member', alias, kind, info.id)
        n._convs = convs
        self.member_readers.add(sc.item)
        return info.type

    def t_field(self, n, sc, expect):
        m = self.m
        obj = n.obj
        if obj.k == 'name' and obj.id not in sc.vars and obj.id in getattr(m, 'joint_members', {}):
            return self.member_read(n, obj.id, n.name, [], sc)
        if self.fill_ref(n, sc) is not None:
            return self.fill_read_type(n, sc, n, [])
        t = self.tc(obj, sc)
        if obj.k == 'name' and obj.id not in sc.vars and t.k == 'keylit' and m.lookup(obj.id) is None:
            # GD-1 (round 19): a qualified name x.f whose prefix names no member, base or value is an unknown id
            # (L §2.5: a member's items are read as a.item; R.x went with regime; L §8.3 LOAD-NAME unknown id)
            raise LoadError('LOAD-NAME', 'unknown id %s.%s' % (obj.id, n.name), n.loc)
        if t.k == 'record':
            ft = t.field(n.name)
            if ft is None:
                raise LoadError('LOAD-FIELD', '%s is not a field of %s' % (n.name, T.show(t)), n.loc)
            n._r = ('field', n.name)
            return ft
        if t.k == 'key' and t.ix in m.kinds and m.kinds[t.ix].spec == 'event':
            return self.movement_field(n, t, sc)
        if t.k == 'key' and t.ix in m.kinds:
            # a thing has no fields but lifespan(k) and its facts' inputs (L §5.7)
            raise LoadError('LOAD-FIELD', '%s is not a field of a thing of %s' % (n.name, t.ix), n.loc)
        raise LoadError('LOAD-FIELD', '%s is not a field of %s' % (n.name, T.show(t)), n.loc)

    # -- operators -------------------------------------------------------------------
    def numcls(self, t):
        k = t.k
        if k in ('int', 'num', 'money', 'rate', 'flex', 'year4', 'dyn'):
            return k
        if k == 'key' and self.key_value_kind(t) == 'int':
            return 'int'
        return None

    def t_neg(self, n, sc, expect):
        if n.e.k == 'num' and not n.e.unit:
            # -1 is one of the inline figures
            if n.e.isint and n.e.v == 1:
                n.e._t = T.flex(-1)
                n.e._val = n.e.v
                n.e._u = ()
                return T.flex(-1)
        t = self.tc(n.e, sc, expect)
        c = self.numcls(t)
        if c is None:
            raise uerr('unary minus on %s' % T.show(t), n.loc)
        if c == 'flex':
            return T.flex(-t.n)
        if c == 'dyn':
            return t
        if t.k == 'key':
            return T.INT
        if c == 'year4':
            return T.INT
        return t

    def t_bin(self, n, sc, expect):
        op = n.op
        sub = sc
        # integer offsets in key arithmetic are allowed figures (L §7.1)
        lt = self.tc(n.l, sc, expect if op in '+-' else None)
        lvar = self.var_of(n.l, sc)
        if op in '+-' and (lt.k == 'key' or (lvar is not None and lvar.dom is not None and lvar.dom[0] in (
                'index', 'range', 'list'))) and n.r.k == 'num' and n.r.isint and not n.r.unit:
            # integer offsets in key arithmetic (L §7.1)
            sub = sc.child()
            sub.allow_lit = True
        offset = sub is not sc
        # an integer offset in key arithmetic is a plain int, never a key of the index (L §4.2, §7.1)
        rt = self.tc(n.r, sub, expect if op in '+-' and not offset else None)
        if lt.k == 'keylit':
            lt = self.tc(n.l, sc, rt)
        if rt.k == 'keylit':
            rt = self.tc(n.r, sub, lt)
        return self.arith(op, lt, rt, n)

    def arith(self, op, lt, rt, n):
        t = self._arith(op, lt, rt, n)
        if t is not None and t.k == 'num' and t.u and U.day_and_period(t.u):
            raise uerr('%s %s %s: day is a dimension of its own (1 day / 1 year is refused, L §3.2)' % (
                T.show(lt), op, T.show(rt)), n.loc)
        return t

    def _arith(self, op, lt, rt, n):
        loc = n.loc
        if lt.k == 'keylist' and getattr(n, 'l', None) is not None:
            lt = self.kl_value(n.l, 'int') or lt
        if rt.k == 'keylist' and getattr(n, 'r', None) is not None:
            rt = self.kl_value(n.r, 'int') or rt
        lk, rk = lt.k, rt.k
        if lk in ('hole',) or rk in ('hole',):
            raise LoadError('LOAD-PARSE', 'cannot parse: a hole stands only as a whole value', loc)
        if lk in ('none', 'never') or rk in ('none', 'never'):
            n._none = True
            other = rt if lk in ('none', 'never') else lt
            return other if other.k not in ('none', 'never') else T.INT
        if lk == 'bool' or rk == 'bool':
            raise uerr('bool in arithmetic', loc)
        if lk == 'dyn' or rk == 'dyn':
            # a quantity whose unit only the run knows (a leg of unit any, an adjustment's change): its units are
            # checked as it is computed
            other = rt if lk == 'dyn' else lt
            if self.numcls(other) is not None:
                n._dyn = True
                return Ty('dyn')
            raise uerr('%s and %s in arithmetic' % (T.show(lt), T.show(rt)), loc)
        if lk == 'year4' or rk == 'year4':
            return self.ambig(op, lt, rt, n)
        # dates, periods, keys
        if lk in ('date', 'period', 'key') or rk in ('date', 'period', 'key'):
            return self.arith_time(op, lt, rt, n)
        if lk in ('text', 'code', 'record', 'list', 'calendar', 'dates', 'keylit', 'mmdd', 'tuple', 'dist') or \
                rk in ('text', 'code', 'record', 'list', 'calendar', 'dates', 'keylit', 'mmdd', 'tuple', 'dist'):
            if lk == 'dist' or rk == 'dist':
                # a distribution value (a dist fact read where it is the distribution, L §5.16) is one of the closed
                # list's, "A distribution the list doesn't hold is written from one it does": no arithmetic makes one
                # (L §8.4 What isn't defined is refused; a type error at load)
                raise uerr('%s and %s in arithmetic: a distribution value is no number (L §5.16 The closed list)' % (
                    T.show(lt), T.show(rt)), loc)
            raise uerr('%s and %s in arithmetic' % (T.show(lt), T.show(rt)), loc)
        if op in '+-':
            self.pct_flex(getattr(n, 'l', None), getattr(n, 'r', None))
            if lk == 'flex' and rk == 'flex':
                return T.INT if _intflex(lt) and _intflex(rt) else T.num(())
            if lk == 'flex':
                return rt
            if rk == 'flex':
                return lt
            if T.is_pct(lt) and T.is_pct(rt):
                return T.pct()
            if lk == 'rate' or rk == 'rate':
                if lk == rk == 'rate':
                    return T.RATE
                raise uerr('%s %s %s: rate arithmetic outside L §3.3' % (T.show(lt), op, T.show(rt)), loc)
            if lk == 'int' and rk == 'int':
                return T.INT
            if lk == 'money' and rk == 'money':
                if lt.cur != rt.cur:
                    raise uerr('%s and %s have different units (money in two currencies)' % (T.show(lt), T.show(rt)), loc)
                return lt
            if lk == 'money' or rk == 'money':
                mo, ot = (lt, rt) if lk == 'money' else (rt, lt)
                if ot.k == 'num' and U.conv(ot.u, mo.u) is not None:
                    return T.num(mo.u)
                raise uerr('%s and %s have different units' % (T.show(lt), T.show(rt)), loc)
            lu = lt.u if lk == 'num' else ()
            ru = rt.u if rk == 'num' else ()
            if U.conv(ru, lu) is None:
                raise uerr('%s and %s have different units' % (T.show(lt), T.show(rt)), loc)
            return T.num(lu)
        # * and /: a bare figure is dimensionless (an int when it is one)
        if lk == 'flex':
            lt, lk = (T.INT, 'int') if _intflex(lt) else (T.num(()), 'num')
        if rk == 'flex':
            rt, rk = (T.INT, 'int') if _intflex(rt) else (T.num(()), 'num')
        if lk == 'rate' or rk == 'rate':
            if op == '*' and ((lk == 'rate' and rk in ('int',) or (rk == 'num' and not rt.u and lk == 'rate')) or
                              (rk == 'rate' and (lk == 'int' or (lk == 'num' and not lt.u)))):
                return T.RATE
            # a rate times a duration is a num: `21 %/year * 1 month` is 1.75 % (L §3.2 Period units)
            if op == '*':
                du = rt.u if lk == 'rate' else lt.u
                ok = (rk == 'num' if lk == 'rate' else lk == 'num')
                if ok and du and len(du) == 1 and du[0][1] == 1 and du[0][0] in ('day', 'month', 'quarter', 'year'):
                    n._rate_duration = True
                    return T.num(())
            raise uerr('%s %s %s: rate arithmetic outside L §3.3' % (T.show(lt), op, T.show(rt)), loc)
        lu = lt.u if lk in ('num', 'money') else ()
        ru = rt.u if rk in ('num', 'money') else ()
        if op == '*':
            self.fx_check(lu, ru, loc, '*')
            if lk == 'int' and rk == 'int':
                return T.INT
            if lk == 'money' and rk == 'int':
                return lt
            if rk == 'money' and lk == 'int':
                return rt
            u, f = U.mul(lu, ru)
            return T.num(u)
        self.fx_check(lu, ru, loc, '/')
        u, f = U.div(lu, ru)
        return T.num(u)

    def pct_flex(self, ln, rn):
        """A bare 0, 1 or -1 beside a `num %` operand takes its unit, % (L §7.1): 1 is 1 % (0.01), as a bare
        literal in a row takes its target's unit (L §3.2)."""
        for f, o in ((ln, rn), (rn, ln)):
            if f is None or o is None:
                continue
            ft, ot = getattr(f, '_t', None), getattr(o, '_t', None)
            if ft is not None and ft.k == 'flex' and T.is_pct(ot):
                self.flex_to(f, T.pct())

    def flex_to(self, node, target):
        """Give a bare figure (through brackets and a minus) the type it takes (L §7.1)."""
        while node.k in ('paren', 'neg'):
            node = node.e
        if node.k == 'num' and not node.unit:
            node._flexto = target

    def fx_check(self, xu, ru, loc, op):
        """An exchange rate A/B applies only from B to A (L §3.2, U-05)."""
        def fx(u):
            cs = U.currencies(u)
            if len(u) == 2 and len(cs) == 2 and sorted(e for b, e in cs) == [-1, 1]:
                a = [b for b, e in cs if e == 1][0]
                b = [b for b, e in cs if e == -1][0]
                return a, b
            return None
        if op == '*':
            for x, r in ((xu, ru), (ru, xu)):
                f = fx(r)
                if f and any(b == f[0] and e > 0 for b, e in x):
                    raise uerr('an exchange rate %s/%s applied to the wrong currency' % f, loc)
        else:
            f = fx(ru)
            if f and any(b == f[1] and e > 0 for b, e in xu):
                raise uerr('an exchange rate %s/%s applied to the wrong currency' % f, loc)

    def year4_as_number(self, *sides):
        """A four-digit literal where no period is expected is a number (L §2.1): the literal rule applies to it as a
        number (L §7.1) before any unit error."""
        for side in sides:
            if getattr(side, '_t', None) is not None and side._t.k == 'year4':
                lsc = getattr(side, '_litsc', None)
                if lsc is not None:
                    self.lit_rule(lsc, side, 'num', side.v)

    def ambig(self, op, lt, rt, n):
        readings = []
        for as_year in (True, False):
            l = (T.period('year') if as_year else T.INT) if lt.k == 'year4' else lt
            r = (T.period('year') if as_year else T.INT) if rt.k == 'year4' else rt
            try:
                readings.append((as_year, self.arith(op, l, r, n)))
            except LoadError:
                pass
        if len(readings) == 2:
            raise LoadError('LOAD-AMBIG', '%s reads as a year and as an integer here' % _txt(n), n.loc)
        if not readings:
            # L §2.1: a four-digit literal is a year only where a period is expected; elsewhere it is a number, and
            # the literal rule (L §7.1) reads it as `500` would be read (`x - 1000` is LOAD-LITERAL in a def)
            self.year4_as_number(n.l, n.r)
            raise uerr('%s and %s in arithmetic' % (T.show(lt), T.show(rt)), n.loc)
        for side in (n.l, n.r):
            if getattr(side, '_t', None) is not None and side._t.k == 'year4':
                side._year = readings[0][0]
                side._t = T.period('year') if readings[0][0] else T.INT
                lsc = getattr(side, '_litsc', None)
                if lsc is not None:
                    self.lit_rule(lsc, side, 'date' if readings[0][0] else 'num', side.v)
        return readings[0][1]

    def is_duration(self, t, pk=None):
        if t.k != 'num' or len(t.u) != 1 or t.u[0][1] != 1:
            return False
        b = t.u[0][0]
        if b not in U.PERIODS:
            return False
        return True

    def arith_time(self, op, lt, rt, n):
        loc = n.loc
        lk, rk = lt.k, rt.k
        intlike = ('int', 'flex')
        # key arithmetic (L §4.2)
        if lk == 'key':
            vk = self.key_value_kind(lt)
            # key arithmetic is a key moved by + or - an int (L §4.2); `*` and `/` on a key of an integer index are
            # int arithmetic (L §4.2 Keys as values: the key is also an int)
            if op in '+-' and (rk in intlike or (rk == 'key' and self.key_value_kind(rt) == 'int' and vk != 'int')):
                if lt.ix in self.m.kinds:
                    # "key arithmetic on things" has no meaning (L §8.4 What isn't defined is refused)
                    raise uerr('key arithmetic on things of kind %s' % lt.ix, loc)
                if lt.ix in self.m.axes:
                    raise uerr('key arithmetic on an axis', loc)
                # L §5.6: key arithmetic over an open index's keys is refused (§8.4)
                self.no_order(lt, 'key arithmetic', loc)
                n._keyarith = True
                return lt
            if op == '-' and rk == 'key' and rt.ix == lt.ix:
                self.no_order(lt, 'key arithmetic', loc)
                n._keydiff = True
                return T.INT
            if vk == 'int' and rk in ('int', 'flex', 'num', 'money', 'rate'):
                return self.arith(op, T.INT, rt, n)
            if vk == 'int' and rk == 'key' and self.key_value_kind(rt) == 'int':
                return self.arith(op, T.INT, T.INT, n)
            if vk == 'date':
                return self.arith_time(op, T.DATE, rt, n)
            if vk == 'period':
                return self.arith_time(op, T.period(self.key_pk(lt)), rt, n)
            raise uerr('%s %s %s' % (T.show(lt), op, T.show(rt)), loc)
        if rk == 'key' and op == '+' and lk in intlike and self.key_value_kind(rt) != 'int':
            raise uerr('%s + %s: write the key first' % (T.show(lt), T.show(rt)), loc)
        if rk == 'key':
            vk = self.key_value_kind(rt)
            if vk == 'int':
                return self.arith(op, lt, T.INT, n)
            if vk == 'date':
                return self.arith_time(op, lt, T.DATE, n)
            if vk == 'period':
                return self.arith_time(op, lt, T.period(self.key_pk(rt)), n)
            raise uerr('%s %s %s' % (T.show(lt), op, T.show(rt)), loc)
        if lk == 'date' and op not in '+-':
            # a date moves by + or - (L §4.2 Date arithmetic); a product or quotient of a date has no meaning (L §8.4)
            raise uerr('%s %s %s' % (T.show(lt), op, T.show(rt)), loc)
        if lk == 'period' and op not in '+-':
            raise uerr('%s %s %s' % (T.show(lt), op, T.show(rt)), loc)
        if lk == 'date':
            if rk in intlike:
                n._datearith = 'days'
                return T.DATE
            if self.is_duration(rt):
                n._datearith = 'duration'
                return T.DATE
            if op == '-' and rk == 'date':
                # L §4.2 defines date ± duration and date ± int; the difference of two dates is days_between
                # (L §8.4 What isn't defined is refused)
                raise uerr('date - date has no meaning; days_between(a, b) gives b - a in days', loc)
            raise uerr('%s %s %s' % (T.show(lt), op, T.show(rt)), loc)
        if lk == 'period':
            if rk in intlike:
                n._datearith = 'periods'
                return lt
            if op == '-' and rk == 'period' and rt.pk == lt.pk and lt.pk in ('month', 'quarter', 'year'):
                # L §4.2 Differences: "two values of one period type, keys or not, differ by an int of periods ...:
                # m - month_of(d) is the int number of months from month_of(d) to m" (MC-B-F3)
                n._datearith = 'pdiff'
                return T.INT
            if self.is_duration(rt):
                # a period moves by a duration that is a whole number of its periods (L §4.2 Date arithmetic):
                # days for a period of days, months, quarters or years for a period of months; whether the count
                # is whole shows at the key (RUN-DOMAIN, L §8.4)
                du = rt.u[0][0]
                pfam = 'day' if lt.pk == 'day' else 'month'
                dfam = 'day' if du == 'day' else 'month'
                if pfam == dfam:
                    n._datearith = 'pduration'
                    return lt
            # a period moves by an int of its periods or a whole number of them as a duration (L §4.2 Date
            # arithmetic); a period plus any other duration is refused (L §8.4)
            raise uerr('%s %s %s (a period moves by an int of its periods, or by a duration that is a whole number '
                       'of them)' % (T.show(lt), op, T.show(rt)), loc)
        raise uerr('%s %s %s' % (T.show(lt), op, T.show(rt)), loc)

    def t_not(self, n, sc, expect):
        t = self.tc(n.e, sc)
        if not self.is_bool(t):
            raise uerr('not of %s' % T.show(t), n.loc)
        return T.BOOL

    def t_and(self, n, sc, expect):
        lt = self.tc(n.l, sc)
        sc2 = sc.child()
        sc2.guards |= self.tested_vars(n.l, set(sc.keyvars))
        rt = self.tc(n.r, sc2)
        if not self.is_bool(lt) or not self.is_bool(rt):
            raise uerr('and of %s and %s' % (T.show(lt), T.show(rt)), n.loc)
        return T.BOOL

    t_or = t_and

    def t_if(self, n, sc, expect):
        # L §5.7 (tally 2.1, R-17): "in if c then e1 else e2 whose condition c is v in K or such a conjunction,
        # throughout c and e1"
        scn, nw = self.narrowed(sc, n.c)
        ct = self.tc(n.c, scn)
        if not self.is_bool(ct):
            raise uerr('if of %s' % T.show(ct), n.loc)
        sc2 = scn.child()
        sc2.guards |= self.tested_vars(n.c, set(sc.keyvars))
        sc2.covered_spans |= self.covered_spans_of(n.c)
        sc3 = sc.child()
        sc3.guards |= self.tested_vars(n.c, set(sc.keyvars))
        sc3.covered_spans |= self.covered_spans_of(n.c)
        at = self.tc(n.a, sc2, expect)
        bt = self.tc(n.b, sc3, expect)
        self.narrow_mark(n.c, nw)
        if expect is None:
            if at.k in ('keylit', 'flex', 'year4') and bt.k not in ('keylit', 'flex', 'year4', 'none', 'hole'):
                at = self.tc(n.a, sc2, bt)
            if bt.k in ('keylit', 'flex', 'year4') and at.k not in ('keylit', 'flex', 'year4', 'none', 'hole'):
                bt = self.tc(n.b, sc3, at)
        out = self.unify(at, bt, n.loc)
        # a bare figure as an if branch takes the type of the whole (L §2.5, §7.1)
        if out.k in ('num', 'money', 'int'):
            for br in (n.a, n.b):
                if br._t is not None and br._t.k == 'flex':
                    self.flex_to(br, out)
        return out

    def unify(self, a, b, loc):
        if a.k in ('none', 'hole'):
            return b
        if b.k in ('none', 'hole'):
            return a
        if a == b:
            return a
        ka, kb = self.numcls(a), self.numcls(b)
        if ka and kb:
            if 'dyn' in (ka, kb):
                return Ty('dyn')
            if ka == 'flex':
                return b
            if kb == 'flex':
                return a
            if {ka, kb} == {'int'}:
                return T.INT
            if ka == 'money' and kb == 'money':
                if a.cur != b.cur:
                    raise uerr('%s and %s have different units' % (T.show(a), T.show(b)), loc)
                return a
            if 'rate' in (ka, kb):
                if ka == kb:
                    return a
                raise uerr('%s and %s have different units' % (T.show(a), T.show(b)), loc)
            au = a.u if ka in ('num', 'money') else ()
            bu = b.u if kb in ('num', 'money') else ()
            if U.conv(au, bu) is None:
                raise uerr('%s and %s have different units' % (T.show(a), T.show(b)), loc)
            return T.num(au)
        if a.k == 'never' and b.k == 'date' or b.k == 'never' and a.k == 'date':
            return T.DATE
        if a.k == 'key' and b.k == 'key' and a.ix in self.m.kinds and b.ix in self.m.kinds:
            return a
        if self.is_date(a) and self.is_date(b):
            # L §3.1 (tally 2.1, R-14): "Where a date is expected (... min or max with a date, an if branch whose other
            # branch is a date ...), a key of a day index may stand, and means start_of(k)": a day index's keys are
            # dates here (key_value_kind); a month's, a year's or a day range's by n aren't, and stay LOAD-UNIT
            return T.DATE
        raise uerr('%s and %s have different types' % (T.show(a), T.show(b)), loc)

    def t_cmp(self, n, sc, expect):
        lt = self.tc(n.l, sc)
        sc2 = sc
        if n.r.k == 'num' and n.r.isint and not n.r.unit and n.l.k == 'call' and n.l.name == 'pos':
            # pos(k) > n, pos(k) == n: "the same integer in the guard that protects the read" (L §7.1, §5.11):
            # n must be an offset of the def's key arithmetic on k
            a0 = n.l.args[0] if n.l.args else None
            offs = self.item_offsets.get(sc.item)
            if sc.where in ('def', 'at') and offs is not None and n.r.v not in (0, 1, -1):
                if a0 is None or a0.k != 'name' or int(n.r.v) not in offs.get(a0.id, ()):
                    raise LoadError('LOAD-LITERAL', 'figure %s in a rule; make it a row (a guard holds only an '
                                    'offset of the key arithmetic it protects)' % _txt(n.r), n.r.loc)
            sc2 = sc.child()
            sc2.allow_lit = True
        rt = self.tc(n.r, sc2, expect=lt if lt.k not in ('keylit', 'flex', 'year4') else None)
        if lt.k in ('keylit', 'flex', 'year4') and rt.k not in ('keylit',):
            lt = self.tc(n.l, sc, expect=rt)
        self.compare(n.op, n.l, n.r, n.loc)
        return T.BOOL

    # -- a variable over a key list in a value position (L §2.5, §4.2 Keys as values, §2.1) ----------------------
    def kl_all_int(self, v):
        return bool(v.keylist) and all(kn.k == 'num' and kn.isint and not kn.unit for kn in v.keylist)

    def kl_value(self, node, use, other=None):
        """The value type of a key-list variable outside a key position: a key of the index its key positions fix;
        before one fixes it, an int when its keys are integers (a key of an integer index is also an int), or a year
        where a period is expected when they are four-digit (L §2.1).  Which index the positions fix is checked once
        every position is seen (kl_finish)."""
        v = getattr(node, '_klvar', None)
        if v is None:
            return None
        if other is not None and other.k == 'key' and other.ix in self.m.indexes:
            dom = ('index', other.ix)
            if v.fixed is None:
                v.fixed = dom
            elif v.fixed != dom:
                raise LoadError('LOAD-KEY', '%s is used as a key of %s and %s' % (v.name, v.fixed[1], dom[1]),
                                node.loc)
            for txt in v.keylist:
                resolve_key_literal(self.m, txt, dom, node.loc)
            node._t = T.key(other.ix)
            return node._t
        if not self.kl_all_int(v):
            return None
        if use == 'year':
            if not all(kn.get('four') for kn in v.keylist):
                return None
            node._year = True
            node._t = T.period('year')
        else:
            node._t = T.INT
        self.kl_uses.append((v, use, node))
        return node._t

    def kl_finish(self):
        for v, use, node in self.kl_uses:
            if v.fixed is None or v.fixed[0] != 'index':
                continue
            ix = self.m.indexes[v.fixed[1]]
            if use == 'int' and not ix.isint:
                raise uerr('%s is a key of %s, not an int' % (v.name, ix.id), node.loc)
            if use == 'year' and not (ix.isint or ix.pk == 'year'):
                raise uerr('%s is a key of %s, not a year' % (v.name, ix.id), node.loc)

    def compare(self, op, ln, rn, loc):
        lt, rt = ln._t, rn._t
        if lt.k == 'keylist':
            lt = self.kl_value(ln, 'int', rt) or lt
        if rt.k == 'keylist':
            rt = self.kl_value(rn, 'int', lt) or rt
        lk, rk = lt.k, rt.k
        eq = op in ('==', '!=')
        for side, t in ((ln, lt), (rn, rt)):
            if t.k == 'none':
                if not eq:
                    ln._cmpnone = True
                return
        if not eq:
            # L §5.7, §5.6, §8.4: things and an open index's keys compare only by == and !=
            self.no_order(lt, 'an order', loc)
            self.no_order(rt, 'an order', loc)
        if lk == 'never' or rk == 'never':
            other = rt if lk == 'never' else lt
            if other.k in ('date', 'never') or (other.k == 'key' and self.key_value_kind(other) == 'date'):
                return
            raise uerr('never is a date; compared with %s' % T.show(other), loc)
        # a period literal compared with a key of a period index is that index's key: "a bare key used elsewhere
        # takes the index ... of the other operand" (L §2.5; §7.1 a comparison with a key)
        for side, t, other in ((ln, lt, rt), (rn, rt, lt)):
            if other.k == 'key' and other.ix in self.m.indexes and \
                    self.m.indexes[other.ix].pk in ('year', 'quarter', 'month') \
                    and (t.k == 'year4' or (t.k == 'period' and side.k in ('month', 'quarter'))) and \
                    side.k in ('num', 'month', 'quarter'):
                ix = self.m.indexes[other.ix]
                if {'num': 'year'}.get(side.k, side.k) != ix.pk:
                    continue
                lsc = getattr(side, '_litsc', None)
                if lsc is not None:
                    self.lit_rule(lsc, side, 'date')
                side._r = ('key', resolve_key_literal(self.m, side, ('index', ix.id), side.loc))
                side._t = T.key(ix.id)
                side._year = False
                side._keylit = True
                return
        if lk == 'year4' or rk == 'year4':
            other = rt if lk == 'year4' else lt
            side = ln if lk == 'year4' else rn
            if other.k == 'period' and other.pk == 'year' or (other.k == 'key' and self.key_pk(other) == 'year'):
                side._year = True
                side._t = T.period('year')
                return
            if self.numcls(other) in ('int', 'num', 'flex'):
                side._year = False
                side._t = T.INT
                return
            self.year4_as_number(side)
            raise uerr('%s compared with %s' % (T.show(lt), T.show(rt)), loc)
        ka, kb = self.numcls(lt), self.numcls(rt)
        if ka and kb:
            self.pct_flex(ln, rn)
            if 'dyn' in (ka, kb):
                return
            if lk == 'key' and rk == 'key':
                if lt.ix != rt.ix:
                    raise LoadError('LOAD-KEY', 'keys of %s and %s compared' % (lt.ix, rt.ix), loc)
                return
            if 'flex' in (ka, kb):
                return
            if 'rate' in (ka, kb):
                if ka == kb:
                    return
                raise uerr('%s and %s compared' % (T.show(lt), T.show(rt)), loc)
            if ka == 'money' and kb == 'money' and lt.cur != rt.cur:
                raise uerr('%s and %s have different units' % (T.show(lt), T.show(rt)), loc)
            au = lt.u if ka in ('num', 'money') else ()
            bu = rt.u if kb in ('num', 'money') else ()
            if U.conv(au, bu) is None:
                raise uerr('%s and %s have different units' % (T.show(lt), T.show(rt)), loc)
            return
        if self.is_date(lt) and self.is_date(rt):
            return
        if lk == 'period' or rk == 'period' or (lk == 'key' and self.key_value_kind(lt) == 'period') or (
                rk == 'key' and self.key_value_kind(rt) == 'period'):
            pa = lt.pk if lk == 'period' else self.key_pk(lt) if lk == 'key' else None
            pb = rt.pk if rk == 'period' else self.key_pk(rt) if rk == 'key' else None
            if pa is not None and pa == pb:
                if lk == 'key' and rk == 'key' and lt.ix != rt.ix:
                    return
                return
            raise uerr('%s and %s compared' % (T.show(lt), T.show(rt)), loc)
        if (lk == 'unitkey' or rk == 'unitkey') and eq:
            # unit(x) of a place whose unit is known only at run is "x's unit as a key of the unit kind that holds it"
            # (L §5.10, round 7c): it compares with a key of a unit kind (`unit(x) == PLN`), or another unit(y)
            other = rt if lk == 'unitkey' else lt
            m = self.m
            if other.k == 'unitkey' or (other.k == 'key' and other.ix in m.kinds and m.kinds[other.ix].spec == 'unit'):
                return
        if lk == 'key' and rk == 'key':
            if lt.ix == rt.ix:
                return
            m = self.m
            if lt.ix in m.kinds and rt.ix in m.kinds:
                a, b = m.kinds[lt.ix], m.kinds[rt.ix]
                if lt.ix in b.subtree(m) or rt.ix in a.subtree(m):
                    return
            raise LoadError('LOAD-KEY', 'keys of %s and %s compared' % (lt.ix, rt.ix), loc)
        if lk == 'bool' and rk == 'bool':
            return
        if lk == 'text' and rk == 'text':
            if not eq:
                raise uerr('text is ordered by nothing; only == and != compare it', loc)
            return
        if lk == 'code' and rk == 'code' and lt.code == rt.code:
            if not eq:
                raise uerr('codes compare only with == and !=', loc)
            return
        if lk == 'mmdd' and rk == 'mmdd':
            if not eq:
                # L §3.1 monthday, §8.4 (GD-2, GD-23): "Month-days compare with == and != only"; an order on them is an
                # operation the type lacks, LOAD-UNIT
                raise uerr('%s orders month-days: they compare with == and != only' % op, loc)
            return
        if lk == 'keylit' or rk == 'keylit':
            raise LoadError('LOAD-KEY', 'ambiguous key %s: nothing names its index' % (
                lt.n if lk == 'keylit' else rt.n), loc)
        if lk == 'record' or rk == 'record':
            # L §3.1 (round 12): "Two records are never compared: ==, != and orders on them are refused (§8.4), and
            # their fields are compared instead; a test against none stays"
            raise uerr('a comparison of two records (compare their fields)', loc)
        raise uerr('%s and %s compared' % (T.show(lt), T.show(rt)), loc)

    def t_in(self, n, sc, expect):
        lt = self.tc(n.l, sc)
        r = n.r
        m = self.m
        if n.on is not None:
            ot = self.tc(n.on, sc)
            if not self.is_date(ot):
                raise uerr('on of a membership test is %s, not a date' % T.show(ot), n.loc)
        if r.k == 'name' and r.id not in sc.vars:
            hit = m.lookup(r.id, sc.regime)
            if hit is not None and hit[0] == 'kind':
                kinfo = hit[1]
                if lt.k == 'keylit':
                    # a bare key takes the kind of the other operand (L §2.5)
                    lt = self.tc(n.l, sc, expect=T.key(kinfo.id))
                if lt.k != 'key' or lt.ix not in m.kinds:
                    raise uerr('membership of %s in kind %s' % (T.show(lt), r.id), n.loc)
                r._r = ('kind', kinfo.id)
                r._t = Ty('kindname')
                return T.BOOL
            if hit is not None and hit[0] == 'index':
                # L §5.7 defines x in K for kinds and key lists only; an index gives the test no meaning
                # (L §8.4 What isn't defined is refused, at load where the types show it)
                raise uerr('membership in index %s: x in K is defined for a kind or a list (§5.7)' % r.id, n.loc)
        if r.k == 'list' and lt.k in ('key', 'code'):
            # a bare key in the list takes the index of the other operand (L §2.5 Keys)
            for it in r.items:
                self.tc(it, sc, expect=lt)
            r._t = Ty('list', elem=lt)
            return T.BOOL
        if r.k == 'list' and lt.k == 'keylit':
            raise LoadError('LOAD-KEY', 'ambiguous key %s: nothing names its index' % lt.n, n.loc)
        rt = self.tc(r, sc)
        if rt.k == 'dates' and self.is_date(lt):
            return T.BOOL
        if rt.k == 'list':
            return T.BOOL
        raise uerr('membership of %s in %s' % (T.show(lt), T.show(rt)), n.loc)

    def t_asif(self, n, sc, expect):
        sc2 = sc.child()
        for p in n.pins:
            self.pin(p, sc)
            if p.kind == 'law':
                sc2.lawpin = True
        return self.tc(n.e, sc2, expect)

    def is_bool(self, t):
        return t.k in ('bool', 'hole')

    def is_date(self, t):
        return t.k in ('date', 'never') or (t.k == 'key' and self.key_value_kind(t) == 'date')

    def is_period(self, t):
        return t.k == 'period' or (t.k == 'key' and self.key_value_kind(t) == 'period')

    def period_kind(self, t):
        if t.k == 'period':
            return t.pk
        if t.k == 'key':
            return self.key_pk(t)
        return None

    # -- assignment ---------------------------------------------------------------------
    def assign(self, target, node, loc, what, pin=False):
        t = node._t
        if t is None:
            return
        k, tk = t.k, target.k
        if k in ('none', 'hole'):
            return
        if k == 'dyn' and tk in ('money', 'num', 'int', 'dyn'):
            node._dynto = target
            return
        if k == 'keylit':
            if tk in ('key', 'code'):
                self.tc(node, Scope(self, ('x', None), None, 'ask'), expect=target)
                return
            raise LoadError('LOAD-KEY', 'ambiguous key %s for %s' % (t.n, what), loc)
        if tk == 'money':
            if k == 'money' and t.cur == target.cur:
                return
            if k == 'flex':
                if (Fraction(t.n) / CURRENCIES[target.cur]).denominator != 1:
                    raise LoadError('LOAD-PRECISION', '%s is finer than the minor unit' % t.n, loc)
                self.flex_to(node, target)
                node._flexto = target
                return
            if k == 'num' and U.conv(t.u, target.u) is not None:
                raise uerr('%s gives a num %s; a money value needs round or prorata (a num CUR stored as money)' % (
                    what, U.fmt(t.u)), loc)
            raise uerr('%s is %s, not %s' % (what, T.show(t), T.show(target)), loc)
        if tk == 'num':
            if k == 'flex':
                node._flexto = target
                return
            if k in ('num', 'money'):
                tu = target.u
                if any(b.startswith('$') for b, e in tu) or any(b.startswith('$') for b, e in t.u):
                    return
                if U.conv(t.u, tu) is not None:
                    return
            if k == 'int' and not target.u:
                return
            if k == 'key' and self.key_value_kind(t) == 'int' and not target.u:
                return
            raise uerr('%s is %s, not %s' % (what, T.show(t), T.show(target)), loc)
        if tk == 'int':
            if k in ('int', 'flex'):
                if k == 'flex':
                    if not _intflex(t):
                        raise uerr('%s is %s, not int' % (what, T.show(t)), loc)
                    node._flexto = target
                return
            if k == 'year4':
                node._year = False
                return
            if k == 'num' and not t.u:
                node._toint = True
                return
            if k == 'key' and self.key_value_kind(t) == 'int':
                return
            raise uerr('%s is %s, not int' % (what, T.show(t)), loc)
        if tk == 'rate':
            if k == 'rate':
                return
            if k == 'flex':
                # a rate carries its period in its value (L §3.1, §3.3); a bare figure states none, and the
                # type `rate` gives none: no meaning (L §8.4)
                raise uerr('%s: a bare figure as a rate states no period (L §3.3)' % what, loc)
            raise uerr('%s is %s, not rate' % (what, T.show(t)), loc)
        if tk == 'date':
            if self.is_date(t):
                return
            raise uerr('%s is %s, not date' % (what, T.show(t)), loc)
        if tk == 'period':
            if self.period_kind(t) == target.pk:
                return
            if k == 'year4' and target.pk == 'year':
                node._year = True
                return
            raise uerr('%s is %s, not %s' % (what, T.show(t), target.pk), loc)
        if tk == 'key':
            if k == 'key':
                if t.ix == target.ix:
                    return
                m = self.m
                if t.ix in m.kinds and target.ix in m.kinds and t.ix in m.kinds[target.ix].subtree(m):
                    return
                if t.ix in m.indexes and target.ix in m.indexes and m.indexes[t.ix].isint and m.indexes[target.ix].isint:
                    node._keyto = target.ix
                    return
            if k in ('int', 'flex') and target.ix in self.m.indexes and self.m.indexes[target.ix].isint:
                node._keyto = target.ix
                return
            # a key of a year, month or day index is a value of that period type (L §4.2 Keys as values)
            ix = self.m.indexes.get(target.ix)
            if ix is not None and ix.pk is not None and k in ('period', 'date', 'key') and \
                    (self.period_kind(t) == ix.pk or (k == 'date' and ix.pk == 'day')):
                node._keyto = target.ix
                return
            if k == 'tuple':
                raise LoadError('LOAD-KEY', '%s gives a tuple of keys, not one key of %s' % (what, target.ix), loc)
            raise LoadError('LOAD-KEY', '%s is %s, not a key of %s' % (what, T.show(t), target.ix), loc)
        if tk == 'bool' and k == 'bool':
            return
        if tk == 'text' and k == 'text':
            return
        if tk == 'code' and k == 'code' and t.code == target.code:
            return
        if tk == 'dates' and k in ('dates',):
            return
        if tk == 'dates' and k == 'list' and t.elem is None:
            return
        if tk == 'calendar' and k == 'calendar':
            return
        if tk == 'mmdd' and k == 'mmdd':
            # L §3.1 monthday (GD-2): a row, a pin, a def's value
            return
        if tk == 'record' and k == 'record':
            for f, ft in target.fields:
                g = t.field(f)
                if g is None:
                    raise LoadError('LOAD-FIELD', '%s lacks field %s' % (what, f), loc)
            return
        if tk == 'dist':
            if k == 'dist' and U.conv(t.u or (), target.u or ()) is not None:
                if getattr(t, 'ix', None) and target.u:
                    raise uerr('%s is a discrete distribution over keys of %s, not %s' % (what, t.ix, T.show(target)),
                               loc)
                return
            raise uerr('%s is %s, not %s' % (what, T.show(t), T.show(target)), loc)
        raise uerr('%s is %s, not %s' % (what, T.show(t), T.show(target)), loc)

    # -- bindings ------------------------------------------------------------------------
    def range_type(self, rng, bnode, sc):
        if sc.where == 'ask':
            sc = sc.child()
            sc.allow_lit = True    # a stepped range's ends and step (L §7.1 An ask)
        at = self.tc(rng.a, sc)
        bt = self.tc(rng.b, sc, expect=at if at.k not in ('flex', 'year4') else None)
        if at.k in ('flex', 'year4'):
            at = self.tc(rng.a, sc, expect=bt if bt.k not in ('flex',) else T.INT)
        st = None
        if rng.step is not None:
            sc2 = sc.child()
            # an ask's stepped range keeps its step's literal too (L §7.1 An ask: "a stepped range's ends and step")
            sc2.allow_lit = getattr(sc, 'allow_lit', False)
            st = self.tc(rng.step, sc2, expect=at)
        # L §5.6, §8.4: a range over an open index's keys (or over things) is refused
        self.no_order(at, 'a range', rng.loc)
        self.no_order(bt, 'a range', rng.loc)
        ints = all(t.k in ('int', 'flex') or (t.k == 'key' and self.key_value_kind(t) == 'int') for t in (at, bt))
        if ints and st is None:
            return T.INT
        if ints and st is not None:
            return T.INT
        if st is None:
            raise LoadError('LOAD-DOMAIN', 'range of quantities %s .. %s has no step' % (_txt(rng.a), _txt(rng.b)),
                            rng.loc)
        if self.is_date(at):
            if not self.is_date(bt) or not self.is_duration(st):
                raise uerr('a range of dates steps by a duration', rng.loc)
            return T.DATE
        u = self.unify(at, bt, rng.loc)
        self.unify(u if u.k != 'money' else T.num(u.u), st if st.k != 'money' else T.num(st.u), rng.loc)
        return u

    def bind(self, b, sc, listing=False, key_bind=False):
        """A binding in an aggregate or an ask's for (L §5.6); key_bind: a fill's key bind, which is defined for every
        key and so states no on or over (L §5.6, round 18 C1-38)."""
        m = self.m
        if b.var in sc.vars:
            raise LoadError('LOAD-SHADOW', 'bound variable %s has the name of a bound variable in scope' % b.var, b.loc)
        if m.lookup(b.var, sc.regime) is not None:
            raise LoadError('LOAD-SHADOW', 'bound variable %s has the name of a top-level id' % b.var, b.loc)
        d = b.dom
        if d.k == 'dname':
            if d.qual is not None:
                hit = m.qualified(d.qual, d.id)
            else:
                hit = m.lookup(d.id, sc.regime)
            if hit is None:
                raise LoadError('LOAD-NAME', 'unknown id %s' % d.id, b.loc)
            kind, info = hit
            if kind == 'index':
                ix = m.indexes[d.id]
                if b.var in ix.texts:
                    raise LoadError('LOAD-SHADOW', 'bound variable %s has the spelling of a key of %s' % (b.var, d.id),
                                    b.loc)
                b._dom = ('index', ix.id)
                ty = T.key(ix.id)
            elif kind == 'kind':
                kinfo = info
                if kinfo.spec == 'event':
                    return self.bind_finish(b, sc, self.bind_movements(b, sc, kinfo=kinfo, key_bind=key_bind))
                if m.thing_in(b.var, kinfo.id):
                    raise LoadError('LOAD-SHADOW', 'bound variable %s has the spelling of a key of %s' % (
                        b.var, kinfo.id), b.loc)
                if b.on is None and b.over is None and not key_bind:
                    raise LoadError('LOAD-ALIVE', 'binding over %s needs on or over' % kinfo.id, b.loc)
                if kinfo.spec == 'place':
                    sub = kinfo.subtree(m)
                    for pq, pi in m.places.items():
                        if pi.kind in sub:
                            self.edge(sc, ('place', pq), 'bind')
                b._dom = ('kind', kinfo.id)
                ty = T.key(kinfo.id)
            elif kind == 'input' and info.observed:
                return self.bind_finish(b, sc, self.bind_movements(b, sc, obs=info))
            elif kind == 'axis':
                # "A bind is over an index, a kind, a key list or a stepped range" (L §5.6) (L §8.4 What isn't defined is refused)
                raise LoadError('LOAD-DOMAIN', 'a binding over the axis %s: a bind is over an index, a kind, a key '
                                'list or a stepped range (L §5.6)' % d.id, b.loc)
            else:
                raise LoadError('LOAD-KEY', '%s is not an index, a kind, a key list or a range' % d.id, b.loc)
        elif d.k == 'dlist':
            b._dom = ('list', [resolve_text(x) for x in d.keys], d.keys)
            ty = Ty('keylist')
            for x in d.keys:
                if resolve_text(x) == b.var:
                    raise LoadError('LOAD-SHADOW', 'bound variable %s has the spelling of a key of its list' % b.var,
                                    b.loc)
        else:
            ty = self.range_type(d, b, sc)
            b._dom = ('range',)
        return self.bind_finish(b, sc, ty)

    def bind_finish(self, b, sc, ty):
        if getattr(self.m, 'joint', None) is not None and b._dom[0] in ('kind', 'event') and \
                isinstance(sc.item, tuple) and len(sc.item) > 1 and isinstance(sc.item[1], str):
            # L §6.1 (OQ-5): "A member's items mean what they mean alone: a binding inside a member ranges over that
            # member's things, the prelude's kinds included. The joint model's own bindings over a shared kind range
            # over every member's things."
            from .joint import owner_deep
            b._owner = owner_deep(sc.item[1]) or None
        v = Var(b.var, ty, b._dom, b, keylist=(b._dom[2] if b._dom[0] == 'list' else None))
        if b._dom[0] == 'kind':
            # a thing's own on may read it: `j in jobs on day_of(m, pay_dom[j])` (L §6.5)
            sc.vars[b.var] = v
        if b.on is not None:
            t = self.tc(b.on, sc)
            if not self.is_date(t):
                raise uerr('on of a binding is %s, not a date' % T.show(t), b.loc)
            if b._dom[0] not in ('kind', 'event', 'obs'):
                # on and over state when a thing of a kind is alive (L §5.6, §5.7); no other bind has one
                raise LoadError('LOAD-DOMAIN', 'on on a binding that is not over a kind', b.loc)
        if b.over is not None:
            self.span(b.over, sc)
            if b._dom[0] not in ('kind', 'event', 'obs'):
                raise LoadError('LOAD-DOMAIN', 'over on a binding that is not over a kind', b.loc)
        sc.vars[b.var] = v
        b._var = v
        if b.where is not None:
            t = self.tc(b.where, sc)
            if not self.is_bool(t):
                raise uerr('where of a binding is %s, not bool' % T.show(t), b.loc)
        return v

    def span(self, sp, sc):
        for e in (sp.a, sp.b):
            if e is not None:
                t = self.tc(e, sc)
                if t.k == 'keylist':
                    t = self.kl_value(e, 'year') or t
                if t.k == 'year4':
                    e._year = True
                    e._t = T.period('year')
                    t = e._t
                if not (self.is_date(t) or self.is_period(t)):
                    raise uerr('a span end is %s, not a date or a period' % T.show(t), sp.loc)

    # -- aggregates ----------------------------------------------------------------------
    def t_agg(self, n, sc, expect):
        sc2 = sc.child()
        sc2.agg_depth += 1
        for b in n.binds:
            self.bind(b, sc2)
        fn = n.fn
        if fn in ('first', 'last') and not n.tie:
            # L §5.11 Ties are stated (round 18 C1-14): over things, movements or an open index's keys "without one
            # they are refused, at load where load sees the binding (§8.4), otherwise at the key (RUN-TIE)"; L §8.4: an
            # order on things, at load where the types show it (this interpreter's code for it, as for min: LOAD-UNIT)
            for b in n.binds:
                dom = getattr(b, '_dom', None) or ()
                what = None
                if dom[:1] == ('kind',):
                    what = 'things'
                elif dom[:1] in (('event',), ('obs',)):
                    what = 'movements'
                elif dom[:1] == ('index',) and self.m.indexes[dom[1]].form == 'open':
                    what = 'open-index keys'
                if what is not None:
                    raise uerr('%s over %s has no ties by' % (fn, what), n.loc)
        if n.term is None:
            if fn == 'count':
                return T.INT
            if fn in ('first', 'last', 'min', 'max'):
                tys = [b._var.ty for b in n.binds]
                if fn in ('min', 'max') and len(tys) != 1:
                    # the least or greatest tuple of keys has no meaning given (L §8.4 What isn't defined is refused)
                    raise LoadError('LOAD-DOMAIN', '%s over a bare binding of several variables' % fn, n.loc)
                if fn in ('min', 'max'):
                    # L §5.11: never things or an open index's keys (§5.7, §5.6)
                    self.no_order(tys[0], fn, n.loc)
                if n.tie:
                    for e in n.tie:
                        self.tc(e, sc2)
                return tys[0] if len(tys) == 1 else Ty('tuple', items=tuple(tys))
            raise LoadError('LOAD-PARSE', 'cannot parse: %s takes a term' % fn, n.loc)
        if fn in ('first', 'last'):
            # L §5.11: first and last "return the key (one bind) or the tuple; they take no term (LOAD-PARSE)"
            raise LoadError('LOAD-PARSE', 'cannot parse: %s takes no term (a bare binding)' % fn, n.loc)
        tt = self.tc(n.term, sc2)
        if n.tie:
            for e in n.tie:
                self.tc(e, sc2)
        if fn == 'sum':
            c = self.numcls(tt)
            if c is None or c == 'rate' and False:
                raise uerr('sum of %s' % T.show(tt), n.loc)
            if c == 'dyn':
                return tt
            if c == 'flex':
                return T.INT
            if c == 'year4':
                return T.INT
            if tt.k == 'key':
                return T.INT
            return tt
        if fn == 'count':
            if not self.is_bool(tt):
                raise uerr('count of %s' % T.show(tt), n.loc)
            return T.INT
        if fn in ('min', 'max', 'argmin', 'argmax'):
            # L §5.11: they order numbers, money, dates and keys of a list or range index, never things or an open
            # index's keys (§5.7, §5.6)
            self.no_order(tt, fn, n.loc)
        if fn in ('min', 'max'):
            if tt.k == 'flex':
                return T.INT
            return tt
        if fn in ('any', 'all'):
            if not self.is_bool(tt):
                raise uerr('%s of %s' % (fn, T.show(tt)), n.loc)
            return T.BOOL
        if fn in ('argmin', 'argmax'):
            tys = [b._var.ty for b in n.binds]
            return tys[0] if len(tys) == 1 else Ty('tuple', items=tuple(tys))
        if fn in ('first', 'last'):
            raise LoadError('LOAD-PARSE', 'cannot parse: %s takes a bare binding' % fn, n.loc)
        raise LoadError('LOAD-PARSE', 'cannot parse: %s' % fn, n.loc)

    def t_query(self, n, sc, expect):
        if n.binds is None:
            return self.t_query_random(n, sc, expect)
        sc2 = sc.child()
        sc2.agg_depth += 1
        for b in n.binds:
            self.bind(b, sc2)
        tt = self.tc(n.arg, sc2)
        if n.fn == 'share':
            if not self.is_bool(tt):
                raise uerr('share of %s' % T.show(tt), n.loc)
            return T.NUM1
        if n.fn == 'mean':
            c = self.numcls(tt)
            if c is None:
                raise uerr('mean of %s' % T.show(tt), n.loc)
            if c in ('int', 'flex'):
                return T.NUM1
            if c == 'money':
                return T.num(tt.u)
            return tt
        scq = sc.child()
        if sc.where == 'ask':
            scq.allow_lit = True   # the level of a quantile (L §7.1 An ask)
        qt = self.tc(n.q, scq)
        if self.numcls(qt) not in ('num', 'flex', 'int'):
            raise uerr('the level of quantile is %s' % T.show(qt), n.loc)
        if sc.where in ('def', 'at') and not (n.q.k == 'num' and n.q.v in (0, 1)):
            pass
        return tt

    # -- calls -----------------------------------------------------------------------------
    def t_call(self, n, sc, expect):
        m = self.m
        name = n.name
        pre = getattr(n, 'prefix', None)
        if pre:
            return self.prefixed_call(n, pre, sc)
        if n.bkey is not None or (name not in DATE_FNS and m.lookup(name, sc.regime) is not None and
                                  m.lookup(name, sc.regime)[0] == 'band'):
            return self.band_call(n, sc)
        if name in WRITER3:
            raise NotYet('%s (writer 3)' % name, n.loc)
        from .parser import BUILTINS
        fn = getattr(self, 'c_' + name, None) if name in BUILTINS else None
        if fn is None:
            hit = m.lookup(name, sc.regime)
            if hit is None:
                raise LoadError('LOAD-NAME', 'unknown id %s' % name, n.loc)
            raise LoadError('LOAD-PARSE', 'cannot parse: %s is not called' % name, n.loc)
        n._r = ('builtin', name)
        return fn(n, sc, expect)

    def args(self, n, k):
        if len(n.args) != k:
            raise LoadError('LOAD-PARSE', 'cannot parse: %s takes %d arguments' % (n.name, k), n.loc)

    def want(self, t, pred, what, n):
        if not pred(t):
            raise uerr('%s: %s is %s' % (n.name, what, T.show(t)), n.loc)

    def tdate(self, e, sc, n, what='the date'):
        t = self.tc(e, sc)
        self.want(t, lambda t: self.is_date(t) or (t.k == 'key' and False), what, n)
        return t

    def tdate_or_period(self, e, sc, n):
        t = self.tc(e, sc)
        if t.k == 'year4':
            e._year = True
            e._t = t = T.period('year')
        self.want(t, lambda t: self.is_date(t) or self.is_period(t), 'the argument', n)
        return t

    def tint(self, e, sc, n, allow_duration=None):
        t = self.tc(e, sc, expect=T.INT)
        if allow_duration and t.k == 'num' and len(t.u) == 1 and t.u[0] == (allow_duration, 1):
            return t
        self.want(t, lambda t: self.numcls(t) in ('int', 'flex') or (t.k == 'num' and not t.u), 'the count', n)
        if t.k == 'num':
            e._toint = True
        return t

    def c_start_of(self, n, sc, expect):
        self.args(n, 1)
        self.tdate_or_period(n.args[0], sc, n)
        return T.DATE

    c_end_of = c_start_of

    def c_year_of(self, n, sc, expect):
        self.args(n, 1)
        self.tdate_or_period(n.args[0], sc, n)
        return T.period('year')

    def c_quarter_of(self, n, sc, expect):
        self.args(n, 1)
        self.tdate_or_period(n.args[0], sc, n)
        return T.period('quarter')

    def c_month_of(self, n, sc, expect):
        self.args(n, 1)
        self.tdate_or_period(n.args[0], sc, n)
        return T.period('month')

    def anchored_key(self, t, what, loc):
        """L §4.2 (round 18 C1-15): day_of and schedule of a key of an anchored index are refused, "at load where load
        sees the index (§8.4), otherwise at the key (RUN-DOMAIN)"; L §8.3 LOAD-DOMAIN "<operation> has no meaning for
        <values> (§8.4, seen at load)"."""
        if t is not None and t.k == 'key':
            ix = self.m.indexes.get(t.ix)
            if ix is not None and ix.decl is not None and getattr(ix.decl, 'frm', None) is not None:
                raise LoadError('LOAD-DOMAIN', '%s of a key of the anchored index %s has no meaning' % (what, ix.id),
                                loc)

    def c_day_of(self, n, sc, expect):
        self.args(n, 2)
        t = self.tdate_or_period(n.args[0], sc, n)
        self.anchored_key(t, 'day_of', n.loc)
        pk = self.period_kind(t)
        if pk == 'month':
            self.tint(n.args[1], sc, n)
            n._form = 'month'
        elif pk == 'year':
            t2 = self.tc(n.args[1], sc, expect=T.MD)
            if t2.k != 'mmdd':
                # L §4.2 day_of(y, md) (GD-2): md is a value of type monthday (§3.1)
                raise uerr('day_of(y, md): md is %s, not a monthday' % T.show(t2), n.loc)
            n._form = 'year'
        else:
            raise uerr('day_of takes a month or a year, not %s' % T.show(t), n.loc)
        return T.DATE

    def c_days_between(self, n, sc, expect):
        self.args(n, 2)
        self.tdate(n.args[0], sc, n)
        self.tdate(n.args[1], sc, n)
        return T.INT

    def c_add_days(self, n, sc, expect):
        self.args(n, 2)
        self.tdate(n.args[0], sc, n)
        self.tint(n.args[1], sc, n, allow_duration='day')
        return T.DATE

    def c_add_months(self, n, sc, expect):
        self.args(n, 2)
        self.tdate(n.args[0], sc, n)
        self.tint(n.args[1], sc, n, allow_duration='month')
        return T.DATE

    def c_add_years(self, n, sc, expect):
        self.args(n, 2)
        self.tdate(n.args[0], sc, n)
        self.tint(n.args[1], sc, n, allow_duration='year')
        return T.DATE

    def c_overlap_days(self, n, sc, expect):
        self.args(n, 3)
        self.tdate(n.args[0], sc, n)
        self.tdate(n.args[1], sc, n)
        t = self.tdate_or_period(n.args[2], sc, n)
        return T.INT

    def c_age(self, n, sc, expect):
        self.args(n, 2)
        self.tdate(n.args[0], sc, n)
        self.tdate(n.args[1], sc, n)
        return T.INT

    c_age_months = c_age

    def c_weekday(self, n, sc, expect):
        self.args(n, 1)
        self.tdate(n.args[0], sc, n)
        return T.INT

    def c_pos(self, n, sc, expect):
        self.args(n, 1)
        t = self.tc(n.args[0], sc)
        if t.k == 'keylit':
            # a key literal with no position to name its index: resolved when one index holds it, else
            # LOAD-KEY (L §2.5 "a key literal that could belong to two indexes ... is refused")
            text = t.n
            hits = [ix for ix in self.m.indexes.values() if text in ix.texts]
            hits += [ax for ax in self.m.axes.values() if text in ax.options]
            if len(hits) != 1:
                raise LoadError('LOAD-KEY', 'ambiguous key %s' % text, n.loc)
            t = self.tc(n.args[0], sc, expect=T.key(hits[0].id))
        if t.k != 'key':
            raise uerr('pos of %s, not a key' % T.show(t), n.loc)
        if t.ix in self.m.kinds:
            # "pos of a thing" has no meaning (L §8.4 What isn't defined is refused)
            raise uerr('pos of a thing of kind %s' % t.ix, n.loc)
        # L §5.6: pos over an open index's keys is refused (§8.4)
        self.no_order(t, 'pos', n.loc)
        return T.INT

    def c_window(self, n, sc, expect):
        self.args(n, 2)
        self.tdate(n.args[0], sc, n)
        self.tdate(n.args[1], sc, n)
        return T.DATE

    def c_one_of(self, n, sc, expect):
        self.args(n, 1)
        a = n.args[0]
        t = self.tc(a, sc)
        if t.k not in ('dates', 'list'):
            raise uerr('one_of takes a list of days', n.loc)
        return T.DATE

    def cal_arg(self, e, sc, n):
        """The calendar argument: a calendar input, read at each day asked about (L §4.2)."""
        m = self.m
        sc2 = sc.child()
        sc2.readmode = 'calendar'
        t = self.tc(e, sc2)
        if t.k != 'calendar':
            raise uerr('%s: the calendar is %s' % (n.name, T.show(t)), n.loc)
        if not (getattr(e, '_r', None) and e._r[0] == 'read' and e._r[1] == 'input'):
            # L §4.2 (round 12): "the calendar a business-day function takes is a read of a calendar input, at any
            # keys (cal[country[p]]); any other calendar expression (a def, an if) is refused (LOAD-UNIT)"
            raise uerr('%s: its calendar %s is not a read of a calendar input' % (n.name, _txt(e)), n.loc)

    def c_is_business(self, n, sc, expect):
        self.args(n, 2)
        self.tdate(n.args[0], sc, n)
        self.cal_arg(n.args[1], sc, n)
        return T.BOOL

    def c_add_business(self, n, sc, expect):
        self.args(n, 3)
        self.tdate(n.args[0], sc, n)
        self.tint(n.args[1], sc, n)
        self.cal_arg(n.args[2], sc, n)
        return T.DATE

    def c_last_business(self, n, sc, expect):
        self.args(n, 3)
        t = self.tdate_or_period(n.args[0], sc, n)
        if not self.is_period(t):
            raise uerr('%s takes a period' % n.name, n.loc)
        self.tint(n.args[1], sc, n)
        self.cal_arg(n.args[2], sc, n)
        return T.DATE

    def c_schedule(self, n, sc, expect):
        self.args(n, 4)
        t = self.tdate_or_period(n.args[0], sc, n)
        self.anchored_key(t, 'schedule', n.loc)
        pk = self.period_kind(t)
        if pk == 'month':
            self.tint(n.args[1], sc, n)
        else:
            # L §4.2 schedule (round 19): "the day of period p given by e (an int day of the month, p a month; a day in
            # every year is schedule(month_of(start_of(y)) + (pay_month - 1), pay_dom, roll, cal))": p is a month
            # (a value of the wrong type, LOAD-UNIT)
            raise uerr('schedule takes a month, not %s' % T.show(t), n.loc)
        t3 = self.tc(n.args[2], sc, expect=T.code('roll'))
        if not (t3.k == 'code' and t3.code == 'roll'):
            raise uerr('schedule: the roll is %s' % T.show(t3), n.loc)
        self.cal_arg(n.args[3], sc, n)
        return T.DATE

    def c_round(self, n, sc, expect):
        self.args(n, 3)
        x, u, mode = n.args
        xt = self.tc(x, sc)
        c = self.numcls(xt)
        if c is None or c == 'rate':
            # L §7.2 rounds a num U; a rate is no num (L §3.3) and enters arithmetic only through its built-ins:
            # no meaning is given, so it is refused at load (L §8.4 What isn't defined is refused)
            raise uerr('round of %s' % T.show(xt), n.loc)
        # the step: 1 with a unit, minor(CUR), or a row (an input) (L §7.1, §7.2)
        sc2 = sc.child()
        if sc.where == 'ask':
            sc2.allow_lit = True   # "An ask may write any positive step" (L §7.2)
        ut = self.tc(u, sc2, expect=xt)
        if u.k == 'num' and not u.unit and not u.get('frac'):
            # a bare step takes the unit of round's operand (L §7.1, §7.2, §7.1 An ask)
            ut = u._t = T.flex(u.v)
        uc = self.numcls(ut)
        if uc is None:
            raise uerr('round: the step is %s' % T.show(ut), n.loc)
        if sc.where in ('def', 'at'):
            ok = (u.k == 'num' and u.v == 1) or (u.k == 'call' and u.name == 'minor') or (
                getattr(u, '_r', None) and u._r[0] == 'read' and u._r[1] == 'input')
            if not ok:
                raise LoadError('LOAD-LITERAL', 'the step of a round is 1 with a unit, minor(CUR) or a row', n.loc)
        xu = xt.u if c in ('num', 'money') else ()
        su = ut.u if uc in ('num', 'money') else ()
        if c == 'dyn':
            # an operand whose unit only the run knows (a leg of unit any): load can't show the unit, so it is checked
            # at the key (L §3.2, §8.4: RUN-DOMAIN only where it really differs)
            n._dyn = True
            self.mode_arg(mode, sc, n)
            if uc == 'flex':
                u._stepflex = True
                return Ty('dyn')
            if uc == 'money':
                n._money = ut.cur
                return T.money(ut.cur)
            return T.num(su) if uc == 'num' else Ty('dyn')
        if uc != 'flex' and U.conv(su, xu) is None:
            raise uerr('round: the step %s and %s have different units' % (T.show(ut), T.show(xt)), n.loc)
        self.mode_arg(mode, sc, n)
        if uc == 'flex' and T.is_pct(xt):
            self.flex_to(u, T.pct())    # 1 in round(x, 1, m) for a num % x is 1 % (L §7.1)
        elif uc == 'flex':
            # a bare step takes the unit of round's operand (L §7.1, §7.2)
            u._stepflex = True
        if uc == 'money' and U.is_currency(xu):
            n._money = ut.cur
            return T.money(ut.cur)
        if uc == 'flex' and U.is_currency(xu):
            # money when U is a currency and the step is a whole number of minor units (L §7.2 Its type)
            cur = xu[0][0]
            if (Fraction(ut.n) / CURRENCIES[cur]).denominator == 1:
                n._money = cur
                return T.money(cur)
            return T.num(xu)
        if uc == 'flex' and ut.n == 1 and not xu and not T.is_pct(xt) and c in ('num', 'int', 'flex'):
            # "round(x, 1, mode) of a dimensionless num is an int" (L §7.2 Its type)
            n._int = True
            return T.INT
        if c in ('int', 'flex') and uc in ('int', 'flex'):
            return T.num(())
        if c == 'money':
            return T.num(xu)
        if c in ('int', 'flex'):
            return T.num(())
        return T.num(xu)

    def mode_arg(self, mode, sc, n):
        m = self.m
        if mode.k == 'name' and mode.id not in sc.vars and mode.id in m.axes:
            ax = m.axes[mode.id]
            for o in ax.options:
                if T.code_kind_of(o) != 'rounding':
                    raise uerr('axis %s is used as a mode, but %s is not a mode name' % (ax.id, o), n.loc)
            mode._r = ('axismode', ax.id)
            mode._t = T.code('rounding')
            self.axes_of.setdefault(sc.item, set()).add(ax.id)
            self.edge(sc, ('axis', ax.id))
            return
        t = self.tc(mode, sc, expect=T.code('rounding'))
        if not (t.k == 'code' and t.code == 'rounding'):
            raise uerr('%s: the mode is %s, not a rounding code' % (n.name, T.show(t)), n.loc)

    def c_minor(self, n, sc, expect):
        self.args(n, 1)
        a = n.args[0]
        if a.k != 'cur':
            raise LoadError('LOAD-PARSE', 'cannot parse: minor takes a currency code', n.loc)
        if a.v not in CURRENCIES:
            raise uerr('money in an undeclared currency %s' % a.v, n.loc)
        a._t = T.key('currencies')
        return T.money(a.v)

    def c_abs(self, n, sc, expect):
        self.args(n, 1)
        t = self.tc(n.args[0], sc)
        c = self.numcls(t)
        if c is None:
            raise uerr('abs of %s' % T.show(t), n.loc)
        if c == 'flex':
            return T.INT
        if t.k == 'key':
            return T.INT
        return t

    def c_min(self, n, sc, expect):
        ts = [self.tc(a, sc, expect) for a in n.args]
        out = ts[0]
        for i, t in enumerate(ts[1:], 1):
            if out.k in ('keylit',):
                out = self.tc(n.args[0], sc, t)
            if t.k == 'keylit':
                t = self.tc(n.args[i], sc, out)
            out = self.unify(out, t, n.loc)
        # L §5.11: min and max order numbers, money, dates and keys of a list or range index, never things or an open
        # index's keys (§5.7, §5.6)
        for t in ts:
            self.no_order(t, n.name, n.loc)
        if out.k == 'flex':
            return T.INT if all(t.k != 'flex' or _intflex(t) for t in ts) else T.num(())
        for a in n.args:
            if a._t is not None and a._t.k == 'flex' and out.k in ('num', 'money', 'int'):
                self.flex_to(a, out)
        return out

    c_max = c_min

    def c_lifespan(self, n, sc, expect):
        self.args(n, 1)
        t = self.tc(n.args[0], sc)
        if t.k == 'keylit' and t.n in self.m.things:
            # a bare key naming one thing (L §2.5)
            t = self.tc(n.args[0], sc, expect=T.key(self.m.things[t.n].base))
        if t.k != 'key' or t.ix not in self.m.kinds:
            raise uerr('lifespan of %s, not a thing' % T.show(t), n.loc)
        return T.record([('from', T.DATE), ('to', T.DATE)])

    def c_missing(self, n, sc, expect):
        self.args(n, 1)
        if not (sc.cited and sc.in_when):
            raise LoadError('LOAD-MISSING', 'missing() outside the when of a cited rule', n.loc)
        r = n.args[0]
        self.tc(r, sc)
        # L §5.11: "missing(r), r a read of an input (LOAD-MISSING otherwise)"; L §8.3: "missing(<r>): <r> is not a
        # read of an input"
        if not (getattr(r, '_r', None) and r._r[0] == 'read' and r._r[1] == 'input'):
            raise LoadError('LOAD-MISSING', 'missing(%s): %s is not a read of an input' % (_txt(r), _txt(r)),
                            n.loc)
        return T.BOOL

    def c_since(self, n, sc, expect):
        self.args(n, 2)
        r = n.args[0]
        sc2 = sc.child()
        sc2.readmode = 'explicit'
        self.tc(r, sc2)
        if not (getattr(r, '_r', None) and r._r[0] == 'read' and r._r[1] == 'input'):
            raise LoadError('LOAD-PARSE', 'cannot parse: since takes a read of a reading', n.loc)
        if not self.m.inputs[r._r[2]].reading:
            raise uerr('since reads a reading; %s is a state' % r._r[2], n.loc)
        self.tdate(n.args[1], sc, n)
        return T.INT

    def c_calendar(self, n, sc, expect):
        raise LoadError('LOAD-PARSE', 'cannot parse: calendar(...) is a row value (L §2.3 value), not an atom of an '
                        'expression', n.loc)

    def t_calval(self, n, sc, expect):
        # L §2.3: calval is a `value` (a row), not an `atom`: an expression can't hold one (writer 2, from writer 1's
        # open item "calendar(...) written inside an expression")
        raise LoadError('LOAD-PARSE', 'cannot parse: calendar(...) is a row value (L §2.3 value), not an atom of an '
                        'expression', n.loc)

    def band_ref(self, b, sc, n):
        """The band of band_lo, band_hi and marginal: b or b[k]."""
        m = self.m
        key = None
        if b.k == 'index' and b.obj.k == 'name':
            name = b.obj.id
            if len(b.keys) != 1:
                raise LoadError('LOAD-KEY', 'wrong number of keys', n.loc)
            key = b.keys[0]
        elif b.k == 'name':
            name = b.id
        else:
            raise LoadError('LOAD-PARSE', 'cannot parse: %s takes a band' % n.name, n.loc)
        hit = m.lookup(name, sc.regime)
        if hit is None or hit[0] != 'band':
            raise LoadError('LOAD-NAME', 'unknown band %s' % name, n.loc)
        info = hit[1]
        if bool(info.binds) != (key is not None):
            raise LoadError('LOAD-KEY', 'wrong number of keys for band %s' % info.id, n.loc)
        conv = None
        if key is not None:
            conv = self.key_pos(key, info.binds[0].dom, sc)
        b._r = ('band', info.id)
        b._t = Ty('bandref')
        n._band = (info.id, key, conv)
        self.edge(sc, ('band', info.id))
        return info

    def c_band_lo(self, n, sc, expect):
        self.args(n, 2)
        info = self.band_ref(n.args[0], sc, n)
        xt = self.tc(n.args[1], sc, expect=info.argtype)
        self.assign(info.argtype, n.args[1], n.loc, 'the argument of %s' % n.name)
        return info.argtype

    c_band_hi = c_band_lo

    def c_marginal(self, n, sc, expect):
        self.args(n, 2)
        info = self.band_ref(n.args[0], sc, n)
        for t in info.tables:
            if t.rows and t.rows[0].lo is None and not t.replace:
                raise LoadError('LOAD-BAND', 'marginal(%s, x) over a first row from -inf' % info.id, n.loc)
        xt = self.tc(n.args[1], sc, expect=info.argtype)
        self.assign(info.argtype, n.args[1], n.loc, 'the argument of marginal')
        rt = info.type
        xc = self.numcls(xt)
        rc = self.numcls(rt)
        if xc is None or rc is None:
            raise uerr('marginal of %s over rows of %s' % (T.show(xt), T.show(rt)), n.loc)
        xu = xt.u if xt.k in ('num', 'money') else ()
        ru = rt.u if rt.k in ('num', 'money') else ()
        u, f = U.mul(xu, ru)
        return T.num(u)

    def prefixed_call(self, n, pre, sc):
        """A band named with its prefixes (L §2.3 Calls, round 19; GD-1 removed `R.b(x)`):
        `a.b(x)` for a member's (§6.1), `household.b(x)` in a scenario's expression row (§6.4, rewritten bare by
        check_scen_rows)."""
        m = self.m
        full = '.'.join(pre + [n.name])
        if len(pre) == 1 and pre[0] in getattr(m, 'joint_members', {}):
            mm = m.joint_members[pre[0]]
            hit = mm.lookup(n.name)
            if hit is None:
                raise LoadError('LOAD-NAME', 'unknown id %s' % full, n.loc)
            if hit[0] != 'band':
                raise LoadError('LOAD-PARSE', 'cannot parse: %s is not a band, and there are no user functions' % full,
                                n.loc)
            info = hit[1]
            if info.binds:
                raise NotYet('a member\'s keyed band called by the joint model (%s)' % full, n.loc)
            if len(n.args) != 1:
                raise LoadError('LOAD-PARSE', 'cannot parse: a band takes one argument', n.loc)
            self.tc(n.args[0], sc, expect=info.argtype)
            self.assign(info.argtype, n.args[0], n.loc, 'the argument of band %s' % full)
            n._r = ('memberband', pre[0], info.id)
            return info.type
        hit = m.lookup(pre[0], sc.regime)
        raise LoadError('LOAD-NAME', 'unknown id %s' % full if hit is None else
                        'cannot parse: %s names no band (a prefix is a member or the base)' % full, n.loc)

    def band_call(self, n, sc, hit=None):
        m = self.m
        if hit is None:
            hit = m.lookup(n.name, sc.regime)
        if hit is None:
            raise LoadError('LOAD-NAME', 'unknown id %s' % n.name, n.loc)
        if hit[0] != 'band':
            raise LoadError('LOAD-PARSE', 'cannot parse: %s is not a band, and there are no user functions' % n.name,
                            n.loc)
        info = hit[1]
        if len(n.args) != 1:
            raise LoadError('LOAD-PARSE', 'cannot parse: a band takes one argument', n.loc)
        if bool(info.binds) != (n.bkey is not None):
            raise LoadError('LOAD-KEY', 'wrong number of keys for band %s' % info.id, n.loc)
        conv = None
        if n.bkey is not None:
            conv = self.key_pos(n.bkey, info.binds[0].dom, sc)
        self.tc(n.args[0], sc, expect=info.argtype)
        self.assign(info.argtype, n.args[0], n.loc, 'the argument of band %s' % info.id)
        n._r = ('band', info.id)
        n._bconv = conv
        self.edge(sc, ('band', info.id), 'same' if not info.binds else 'computed')
        return info.type

    def t_lastx(self, n, sc, expect):
        r = n.ref
        sc2 = sc.child()
        sc2.readmode = 'explicit'
        t = self.tc(r, sc2)
        if not (getattr(r, '_r', None) and r._r[0] == 'read' and r._r[1] == 'input'):
            raise LoadError('LOAD-PARSE', 'cannot parse: last(x[k], d) takes a read of an input', n.loc)
        self.tdate(n.d, sc, n)
        return t

    def t_star(self, n, sc, expect):
        raise LoadError('LOAD-PARSE', 'cannot parse: * stands only in a pin', n.loc)

    # -- the graph ------------------------------------------------------------------------
    def check_graph(self):
        m = self.m
        E = self.edges
        # same-key cycles (L §5.11 Recurrences)
        same = {}
        for r, ts in E.items():
            for t, rels in ts.items():
                if r[0] in ('def', 'band') and t[0] in ('def', 'band') and ('same' in rels or (
                        r[0] == 'band' and t[0] == 'def' and not m.defs[t[1]].binds)):
                    same.setdefault(r, set()).add(t)
        self._cycles(same)
        # reachability for recurrence reads
        reach_cache = {}

        def reaches(a, b):
            key = (a, b)
            if key in reach_cache:
                return reach_cache[key]
            seen = set()
            stack = [a]
            ok = False
            while stack:
                x = stack.pop()
                if x == b:
                    ok = True
                    break
                if x in seen:
                    continue
                seen.add(x)
                stack.extend(E.get(x, {}).keys())
            reach_cache[key] = ok
            return ok
        dirs = {}
        for reader, target, var, sign, guarded, loc in self.shift_reads:
            if target == reader or reaches(target, reader):
                if not guarded:
                    raise LoadError('LOAD-GUARD', '%s[%s %s n] is a recurrence read with no condition on %s or pos(%s) '
                                                  'guarding it' % (target[1], var, sign, var, var), loc)
                dirs.setdefault((reader, var), set()).add(sign)
        for (reader, var), ss in dirs.items():
            if len(ss) > 1:
                raise LoadError('LOAD-CYCLE', '%s reads itself both earlier and later on %s' % (reader[1], var))
        # law dates (L §4.3)
        self.check_lawdates()
        # forks (L §5.5)
        for a in m.asks:
            axes = self.reach_axes(('ask', a.id))
            pinned = set(p.axis for p in a.pins if getattr(p, 'pkind', None) == 'axis')
            n = 1
            for ax in axes - pinned:
                n *= len(m.axes[ax].options)
            if n > 64:
                raise LoadError('LOAD-FORKS', 'ask %s reaches %d axis combinations; the cap is 64' % (a.id, n), a.loc)
            a._axes = sorted(axes - pinned)
        # unread items (L §8.5)
        read = set()
        for r, ts in E.items():
            for t in ts:
                read.add(t)
        # tally 2.1: a def an arrival rule or a takes clause reads is read (parts.chk_check_parts)
        read |= getattr(self, 'parts_reads', set())
        for qid, inf in m.inputs.items():
            if inf.builtin:
                continue
            if ('input', qid) not in read and not inf.observed:
                m.warnings.append(('WARN-UNREAD', 'input %s is read by nothing' % qid))
        for qid, d in m.defs.items():
            if ('def', qid) not in read:
                m.warnings.append(('WARN-UNREAD', 'def %s is read by nothing' % qid))

    def _cycles(self, g):
        state = {}

        def visit(x, path):
            st = state.get(x)
            if st == 1:
                cyc = path[path.index(x):] + [x]
                raise LoadError('LOAD-CYCLE', '%s depends on itself at the same key: %s' % (
                    x[1], ' -> '.join(c[1] for c in cyc)))
            if st == 2:
                return
            state[x] = 1
            for y in sorted(g.get(x, ())):
                visit(y, path + [x])
            state[x] = 2
        for x in sorted(g):
            visit(x, [])

    def reach_axes(self, start):
        seen = set()
        stack = [start]
        out = set()
        while stack:
            x = stack.pop()
            if x in seen:
                continue
            seen.add(x)
            out |= self.axes_of.get(x, set())
            if x[0] == 'axis':
                out.add(x[1])
            for t in self.edges.get(x, {}):
                stack.append(t)
            if x == ('balances',):
                # a balance or a place's flows reads the timeline, which carries the axes the moves reach (E §3.3:
                # "the axis options are those of the axes the moves reach"; E §3.1 step 5, the item graph: a
                # place's flows reaches the moves that book into it and what their amounts read)
                for qid in self.m.moves:
                    stack.append(('move', qid))
                # tally 2.1: the walk reads every place's parts clauses, the arrival rule, the owned clause's ties by,
                # the takes order and prorata's ties by, so what they read is in the cone of an ask that reaches a
                # balance (L §5.5; the migrator's F-l)
                for t in sorted(getattr(self, 'parts_reads', ())):
                    stack.append(t)
                for kid in sorted(getattr(self.m, 'parts_kinds', None) or {}):
                    stack.append(('parts', kid))
            if x[0] == 'input':
                inf = self.m.inputs.get(x[1])
                if inf is not None:
                    for b in inf.binds:
                        if b.dom[0] == 'axis':
                            pass
        return out

    def check_lawdates(self):
        m = self.m
        E = self.edges
        lawdated = {}

        def ver_timeless(v):
            return v.governs is not None and v.governs.always and not v.moved_by

        def own_dated(d):
            # law-dated: "one with a row or version whose span isn't always" (L §4.3 rule 1): a given, a rule's
            # own governs, its v version's or its replacing version's span
            if any(not row_timeless(r) for r in d.givens):
                return True
            for lab, vers in d.rules.items():
                for rv in vers:
                    if rv.span is not None:
                        return True
                    if rv.v is not None and not ver_timeless(rv.v):
                        return True
                    if rv.version is not None and not ver_timeless(rv.version):
                        return True
            return False

        # a band is law-dated as an input is (L §4.3: "an input, band, def or rule with a row or version whose span
        # isn't always"): a table in a version whose governs isn't always (or that a moves line moves). Bands "are
        # read at the law date of their reader" (L §4.3, M-10), and so are the inputs, bands and defs their rows read
        # (L §5.9 Rows), so a band whose rows read a law-dated fact is law-dated too (P-6: the chain of readers goes
        # on through it). MC-X-F12: snap12 took every band as law-dated.
        banddated = {bid: any(not ver_timeless(t.version) for t in b.tables) or ('band', bid) in self.member_readers
                     for bid, b in m.bands.items()}
        self.banddated = banddated

        # a def whose `at e` reads law-dated facts reads them at its reader's law date (L §4.3), so its
        # readers need one, as an `at reader` def's do
        at_memo = {}

        def at_dated(qid, seen=()):
            if qid in at_memo:
                return at_memo[qid]
            if qid in seen:
                return False
            out = False
            for t, rels in self.at_reads.get(qid, {}).items():
                if not (rels - {'pin', 'calendar', 'explicit', 'lawpin'}):
                    continue
                if t[0] == 'band':
                    if banddated.get(t[1], True):
                        out = True
                elif t[0] == 'input':
                    inf = m.inputs.get(t[1])
                    if inf is not None and not inf.undated_read():
                        out = True
                elif t[0] == 'def':
                    td = m.defs[t[1]]
                    if td.at is not None and td.at.k == 'law_reader':
                        out = True
                    elif td.at is not None and at_dated(t[1], tuple(seen) + (qid,)):
                        out = True
                    elif td.at is None and own_dated(td):
                        out = True
            at_memo[qid] = out
            return out

        def needs_reader_law(qid):
            d = m.defs[qid]
            if d.at is None:
                return False
            return d.at.k == 'law_reader' or at_dated(qid)
        self.needs_reader_law = needs_reader_law
        changed = True
        timeless_reads = {}
        for qid, d in m.defs.items():
            # a member's item is read at its reader's law date (L §6.1, §4.3): a reader of one states its at
            lawdated[qid] = own_dated(d) or ('def', qid) in self.member_readers

        def reads_dated(item):
            # what a band's rows read, as a def's (below) reads them: a law-dated band, input or def
            for t, rels in E.get(item, {}).items():
                if not (rels - {'pin', 'calendar', 'explicit', 'lawpin'}):
                    continue
                if t[0] == 'band':
                    if banddated.get(t[1], True):
                        return True
                elif t[0] == 'input':
                    inf = m.inputs.get(t[1])
                    if inf is not None and (inf.observed or inf.undated_read()):
                        continue
                    return True
                elif t[0] == 'def':
                    if needs_reader_law(t[1]) or (m.defs[t[1]].at is None and lawdated[t[1]]):
                        return True
            return False

        # bands and defs are taken together to their least fixed point (each only turns law-dated); at_dated reads
        # banddated, so its memo is dropped whenever a band turns law-dated
        while changed:
            changed = False
            for bid in m.bands:
                if not banddated[bid] and reads_dated(('band', bid)):
                    banddated[bid] = True
                    changed = True
                    at_memo.clear()
            for qid, d in m.defs.items():
                if lawdated[qid]:
                    continue
                for t, rels in E.get(('def', qid), {}).items():
                    if not (rels - {'pin', 'calendar', 'explicit', 'lawpin'}):
                        continue
                    if t[0] == 'band':
                        if banddated.get(t[1], True):
                            lawdated[qid] = True
                    elif t[0] == 'input':
                        inf = m.inputs.get(t[1])
                        if inf is not None and inf.observed:
                            # observation rows are read at any law date (L §5.19 Reading history)
                            continue
                        if inf is not None and inf.lawless():
                            timeless_reads.setdefault(qid, set()).add(t[1])
                        elif inf is not None and inf.undated_read():
                            # L §4.3 rule 1 (round 14): "other than an input keyed by one period index, which is read
                            # at its key's period, §4.6 step 1"
                            pass
                        else:
                            lawdated[qid] = True
                    elif t[0] == 'def':
                        if needs_reader_law(t[1]) or (m.defs[t[1]].at is None and lawdated[t[1]]):
                            lawdated[qid] = True
                if lawdated[qid]:
                    changed = True
        for qid, d in m.defs.items():
            if lawdated[qid] and d.at is None:
                raise LoadError('LOAD-LAWDATE', '%s reads law-dated facts and states no at' % qid, d.decl.loc)
            # an input whose rows all govern always is read without a law date, by a def as by a move (L §4.3
            # rule 2)
        self.lawdated = lawdated

        for a in m.asks:
            if a._has_law:
                continue
            for t, rels in E.get(('ask', a.id), {}).items():
                if not (rels - {'pin', 'calendar', 'explicit', 'lawpin'}):
                    continue
                if t[0] == 'def' and needs_reader_law(t[1]):
                    d = m.defs[t[1]]
                    if d.at.k == 'law_reader':
                        raise LoadError('LOAD-LAWDATE', 'ask %s reaches an at reader def without law' % a.id, a.loc)
                    raise LoadError('LOAD-LAWDATE', 'ask %s reaches %s, whose at reads law-dated facts at its '
                                    'reader\'s law date, without law (L §4.3)' % (a.id, t[1]), a.loc)
                # L §4.3: "and so does an ask line that reads a law-dated input or band directly, not through a def
                # that states its own at: the line is its reader (LOAD-LAWDATE)"
                # (an input keyed by one period index is read at its own time key, L §4.6 step 1, so it needs none)
                if (t[0] == 'band' and banddated.get(t[1], True)) or (
                        t[0] == 'input' and t[1] in m.inputs and not m.inputs[t[1]].observed
                                      and not m.inputs[t[1]].lawless()
                                      and m.inputs[t[1]].time_pos in (None, 'multi')):
                    raise LoadError('LOAD-LAWDATE', 'ask %s reads law-dated %s directly, without law' % (a.id, t[1]),
                                    a.loc)

    def readers_of(self, item):
        """Every item that reads x, directly or not: direct readers first, then by depth, then by id."""
        inv = {}
        for r, ts in self.edges.items():
            for t in ts:
                inv.setdefault(t, set()).add(r)
        out = []
        seen = {item}
        frontier = [item]
        while frontier:
            nxt = set()
            for x in frontier:
                for r in inv.get(x, ()):
                    if r not in seen:
                        seen.add(r)
                        nxt.add(r)
            layer = sorted(nxt, key=lambda r: r[1])
            out.extend(layer)
            frontier = layer
        return out


def _intflex(t):
    """A bare figure that is an integer (0, 1, -1, or an integer in a band row)."""
    return Fraction(t.n).denominator == 1


def _txt(n):
    if n is None:
        return ''
    k = n.k
    if k == 'name':
        return n.id
    if k == 'num':
        return n.text + ((' ' + U.fmt(U.from_syntax([(b, e) for b, e in n.unit])[0])) if n.unit else '')
    if k == 'key':
        return n.v
    if k == 'date':
        return n.v.isoformat()
    if k == 'month':
        return '%04d-%02d' % n.v
    if k == 'quarter':
        return '%04d-Q%d' % n.v
    if k == 'index':
        return '%s[%s]' % (_txt(n.obj), ', '.join(_txt(x) for x in n.keys))
    if k == 'field':
        return '%s.%s' % (_txt(n.obj), n.name)
    if k == 'bin':
        return '%s %s %s' % (_txt(n.l), n.op, _txt(n.r))
    if k == 'call':
        return '%s(%s)' % (n.name, ', '.join(_txt(a) for a in n.args))
    if k == 'paren':
        return '(%s)' % _txt(n.e)
    if k == 'neg':
        return '-' + _txt(n.e)
    if k == 'cmp':
        return '%s %s %s' % (_txt(n.l), n.op, _txt(n.r))
    if k in ('and', 'or'):
        return '%s %s %s' % (_txt(n.l), k, _txt(n.r))
    if k == 'not':
        return 'not ' + _txt(n.e)
    if k == 'if':
        return 'if %s then %s else %s' % (_txt(n.c), _txt(n.a), _txt(n.b))
    if k == 'in':
        return '%s in %s%s' % (_txt(n.l), _txt(n.r), (' on ' + _txt(n.on)) if n.on is not None else '')
    if k == 'str':
        return '"%s"' % n.v
    if k == 'bool':
        return 'true' if n.v else 'false'
    if k in ('none', 'never', 'cut'):
        return k
    if k == 'cur':
        return n.v
    if k == 'asif':
        return '%s as if {...}' % _txt(n.e)
    if k == 'agg':
        return '%s(...)' % n.fn
    if k == 'lastx':
        return 'last(%s, %s)' % (_txt(n.ref), _txt(n.d))
    if k == 'span':
        a = _txt(n.a) if n.a is not None else ''
        if not n.dots:
            return a
        b = _txt(n.b) if n.b is not None else ''
        return '%s .. %s' % (a, b) if b else '%s ..' % a
    if k == 'bal':
        s = 'bal(%s%s%s)' % (_txt(n.ref), ''.join(', ' + _txt(a) for a in n.args), ' start' if n.start else '')
        if n.modelled:
            s += ' modelled'
        if n.proj and 'frm' in n.proj:
            s += ' from ' + _txt(n.proj['frm'])
        return s
    if k == 'low':
        s = 'low(%s, %s)' % (_txt(n.ref), _txt(n.span))
        if n.proj and 'frm' in n.proj:
            s += ' from ' + _txt(n.proj['frm'])
        return s
    if k == 'flows':
        s = 'flows(%s, %s%s)' % (_txt(n.ref), _txt(n.span), (', for ' + _txt(n.forkey)) if n.forkey is not None else '')
        if n.conv is not None:
            s += ' in %s at %s %s' % (n.conv[0], _txt(n.conv[1]), _txt(n.conv[2]))
        return s
    if k == 'value':
        return 'value(%s, %s) in %s' % (_txt(n.ref), _txt(n.d), n.cur)
    if k == 'query':
        s = _txt(n.arg)
        if n.binds is not None:
            s += ' for ...'
        if n.where is not None:
            s += ' where ' + _txt(n.where)
        if n.q is not None:
            s += ', ' + _txt(n.q)
        return '%s(%s)' % (n.fn, s)
    return k


from . import check2 as _check2  # noqa: E402
_check2.install(Checker)
from . import check3 as _check3  # noqa: E402
_check3.install(Checker)
from . import parts as _parts  # noqa: E402
_parts.install(Checker)
