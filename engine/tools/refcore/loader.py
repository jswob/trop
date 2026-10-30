"""Loading a reality (E §3.1): parse, resolve the namespace, build kinds,
things, indexes and versions, index the rows, check them.  Expressions are
resolved and typed by check.py, called at the end.
"""
import datetime
import os
from fractions import Fraction

from . import units as U
from . import ty as T
from .dates import (Period, day_period, month_period, year_period, quarter_period, add_days, add_months, ONE_DAY,
                    month_add)
from .errors import LoadError, Unspecified, NotYet
from .lexer import split_lines
from .model import (Reality, Span, ALWAYS, lit_period, lspan_to_span, visible, known_first, VersionInfo, KindInfo,
                    Entry, ThingInfo, IndexInfo, AxisInfo, BindInfo, Row, InputInfo, RuleVer, DefInfo, Table,
                    BandRow, BandInfo, PRELUDE_PLACE, BUILTIN_INPUTS, BUILTIN_INDEXES, PRELUDE_KINDS,
                    BUILTIN_IDS, key_text, _kv, PlaceInfo, PlaceThing, MoveInfo, ObsRow)
from .node import Node
from .parser import parse_file, prescan_units, BUILTINS, KEYWORDS, PERIOD_WORDS
from .values import (Num, Int, Money, Text, Key, Thing, Code, Rec, Lst, Iv, Cal, Hole, NONE, NEVER, CURRENCIES,
                     terminating)


def collect(paths):
    files = []
    for p in paths:
        if os.path.isdir(p):
            for root, dirs, fs in os.walk(p):
                dirs.sort()
                for f in sorted(fs):
                    if f.endswith('.tally'):
                        files.append(os.path.join(root, f))
        else:
            files.append(p)
    return files


def load_paths(paths):
    files = collect(paths)
    texts = []
    for f in files:
        with open(f, 'rb') as fh:
            raw = fh.read()
        try:
            src = raw.decode('utf-8')
        except UnicodeDecodeError:
            raise LoadError('LOAD-PARSE', 'cannot parse: file is not UTF-8', f)
        texts.append((f, src))
    return load_texts(texts)


def scenario_header(fname, src):
    """(id, over, loc) of a scenario file's header, or None for a model or part file (L §6.1, §6.4)."""
    lines = split_lines(src, fname)
    if not lines or not lines[0].toks or lines[0].toks[0].s != 'scenario':
        return None
    toks = lines[0].toks
    if len(toks) < 4 or toks[2].s != 'over':
        raise LoadError('LOAD-PARSE', 'cannot parse: scenario ID over ID', lines[0].loc)
    return (toks[1].s, toks[3].s, lines[0].loc)


def load_texts(texts, worlds=None, members=None):
    """texts: [(file name, source)] -> Reality.  worlds: every scenario id of the reality's files, for the asks
    that name one this reality doesn't load (L §6.4)."""
    all_lines = []
    for fname, src in texts:
        all_lines.append((fname, split_lines(src, fname)))
    return load_lines(all_lines, worlds, members)


def load_lines(all_lines, worlds=None, members=None, joint=None):
    """[(file name, lines)] -> Reality; joint: a joint model's JointInfo (joint.py), its members' lines merged in."""
    units = prescan_units([ls for _, ls in all_lines])
    trees = []
    for fname, lines in all_lines:
        if not lines:
            raise LoadError('LOAD-HEADER', 'file has no header', fname)
        trees.append((fname, parse_file(lines, units)))
    ld = Loader(trees, worlds, members)
    ld.joint = joint
    return ld.build()


def tag_scenario(node, sid):
    """Mark a section of a scenario file, and its items (a version's), with the scenario (L §6.4)."""
    node._scen = sid
    for it in getattr(node, 'items', None) or ():
        if hasattr(it, 'k'):
            it._scen = sid



def _txt_node(n):
    """A same line's side as written (its ids under their owners' prefixes)."""
    from .check import _txt
    try:
        return _txt(n)
    except Exception:
        return getattr(n, 'id', '?')

def _parts_overlap(a, b):
    """Two stated parts' labels that may name one part (L §5.19, 2.1-F6): one label twice, a party and a batch, or a party
    or a batch and a pair that holds it."""
    def pat(q):
        return (q[1], None) if q[0] == 'owner' else (None, q[1]) if q[0] == 'batch' else (q[1], q[2])
    pa, pb = pat(a), pat(b)
    return (pa[0] is None or pb[0] is None or pa[0] == pb[0]) and (pa[1] is None or pb[1] is None or pa[1] == pb[1])


class Loader:
    def __init__(self, trees, worlds=None, members=None):
        self.trees = trees
        self.m = Reality()
        self.worlds = set(worlds or ())
        self.members = members or {}

    # -----------------------------------------------------------------
    def build(self):
        m = self.m
        models = [t for f, t in self.trees if t.header.kind == 'model']
        if len(models) != 1:
            raise LoadError('LOAD-HEADER', '%d files have a model header; exactly one does' % len(models))
        tallies = set(t.header.tally for f, t in self.trees)
        if len(tallies) != 1:
            raise LoadError('LOAD-VERSION', 'files declare different tally versions')
        if tallies != {2}:
            raise NotYet('tally %d (this interpreter runs tally 2)' % list(tallies)[0])
        h = models[0].header
        m.header = h
        m.S, m.E, m.on = h.start, h.end, h.on
        # a joint model's members: merged in under their prefixes (joint.py), and each loaded alone for its asks,
        # asked as a.q (L §6.1 Joint models)
        m.joint = getattr(self, 'joint', None)
        if h.sames and m.joint is None:
            raise NotYet('same lines of a model loaded without its joint model\'s pass (L §6.1)', h.loc)
        m.joint_members = {}
        for a, f, hsh in h.uses:
            if a not in self.members:
                raise NotYet('a joint model loaded without its members (%s)' % a, h.loc)
            m.joint_members[a] = self.members[a]
        sections = []
        # scenarios (L §6.4): this reality holds the base and one stack of scenarios; every section of a scenario
        # file is tagged with it, so its versions are seen only in that scenario and those over it
        m.scenario_over = {}
        m.scen_from = {}
        m.scen_header = {}
        m.wset = frozenset()
        for f, t in self.trees:
            hd = t.header
            if hd.kind == 'scenario':
                if hd.id in m.scenario_over:
                    raise LoadError('LOAD-SCENARIO', 'scenario %s declared twice' % hd.id, hd.loc)
                m.scenario_over[hd.id] = None if hd.over == h.id else hd.over
                m.scen_from[hd.id] = hd.frm
                m.scen_header[hd.id] = hd
                for sec in t.sections:
                    tag_scenario(sec, hd.id)
            sections.extend(t.sections)
        for sid, ov in m.scenario_over.items():
            if ov is not None and ov not in m.scenario_over:
                raise LoadError('LOAD-SCENARIO', 'scenario %s is over %s, neither the model nor a scenario' % (sid, ov),
                                m.scen_header[sid].loc)
        m.all_worlds = set(self.worlds) | set(m.scenario_over) | {h.id}
        self.sections = sections
        CURRENCIES.clear()
        self.register_builtins()
        self.register_names()
        self.build_units()
        self.build_kinds()
        self.build_indexes()
        self.build_axes()
        self.build_versions()
        self.build_entries()
        self.build_decls()
        self.prescan_open_keys()
        self.build_places()
        self.joint_place_sides()
        self.joint_place_sames()
        self.not_yet_sections()
        self.check_key_ids()
        self.build_items()
        self.check_rows()
        self.check_obs()
        self.joint_sames()
        self.check_scenarios()
        self.check_bands()
        # L §5.10 (round 14): a place's price input is named once (LOAD-PRICE)
        for th in m.things.values():
            m.price_of(th)
        from .check import Checker
        m.checker = Checker(m)
        m.checker.run()
        # the currencies of this reality: CURRENCIES is global, and another reality's load (a scenario's, a
        # member's) resets it; each answer restores its own (ask.Asker.answer)
        m.currencies = dict(CURRENCIES)
        return m

    def prescan_open_keys(self):
        """The keys of open indexes, from the rows of inputs keyed by them (L §4.1: a key exists from the earliest
        known of the versions whose rows bring it), gathered before places keyed by such an index are built."""
        m = self.m
        if not any(ix.form == 'open' for ix in m.indexes.values()):
            return
        for vid, v in m.versions.items():
            for it in v.node.items:
                if it.k != 'row' or it.given or it.keys is None:
                    continue
                try:
                    kind, inf = self.target(it.qual, it.target, it.loc, ('input',))
                except LoadError:
                    continue
                if inf.observed:
                    continue
                for kn, b in zip(it.keys, inf.binds):
                    if b.dom[0] != 'index' or m.indexes[b.dom[1]].form != 'open' or kn.k in ('star', 'krange'):
                        continue
                    try:
                        self.pattern_key(kn, b, it.loc, v)
                    except LoadError:
                        pass

    def not_yet_sections(self):
        """Sections left to later writers refuse the reality (REFCORE-NOT-YET): none left."""
        return

    def check_scenarios(self):
        """No scenario row governs a day before its from (L §6.4, LOAD-SCENARIO)."""
        m = self.m

        def early(sp, frm):
            return sp is None or sp.always or sp.start is None or sp.start < frm

        # L §6.4 (round 11b): "no scenario row of an item the base declares governs a day before D (LOAD-SCENARIO).
        # A row of an input the scenario declares may govern always (§4.3: it then needs no law date), since only the
        # scenario reads it."
        def base_item(info):
            decl = getattr(info, 'decl', None)
            return decl is None or getattr(decl, '_scen', None) is None

        def barred(sc, frm, what, loc):
            # L §8.3 `a row of <s> for <x>, an item of the base, governs a day before <from>`
            raise LoadError('LOAD-SCENARIO', 'a row of %s for %s, an item of the base, governs a day before %s' % (
                sc, what, frm), loc)

        for d in m.defs.values():
            if not base_item(d):
                continue
            for r in d.givens:
                sc = r.version.scenario if r.version is not None else None
                if sc is not None:
                    sp = r.own_span if r.own_span is not None else r.version.governs
                    if early(sp, m.scen_from[sc]):
                        barred(sc, m.scen_from[sc], d.id, r.loc)
            for lab, vers in d.rules.items():
                for rv in vers:
                    v = rv.version
                    if v is not None and v.scenario is not None:
                        sp = rv.span if rv.span is not None else v.governs
                        if early(sp, m.scen_from[v.scenario]):
                            barred(v.scenario, m.scen_from[v.scenario], '%s.%s' % (d.id, lab), rv.loc)
        for b in m.bands.values():
            if not base_item(b):
                continue
            for t in b.tables:
                v = t.version
                if v.scenario is not None and early(v.governs, m.scen_from[v.scenario]):
                    barred(v.scenario, m.scen_from[v.scenario], b.id, t.loc)
        for inf in m.inputs.values():
            if not base_item(inf):
                continue
            for r in inf.rows:
                sc = r.version.scenario if r.version is not None else None
                if sc is None:
                    continue
                frm = m.scen_from[sc]
                if r.instant is not None:
                    if r.instant[0] < frm:
                        barred(sc, frm, inf.id, r.loc)
                    continue
                sp = r.own_span if r.own_span is not None else r.version.governs
                if early(sp, frm):
                    barred(sc, frm, inf.id, r.loc)
            for o in getattr(inf, 'obs', []):
                sc = o.version.scenario if o.version is not None else None
                if sc is not None and early(o.version.governs, m.scen_from[sc]):
                    barred(sc, m.scen_from[sc], inf.id, o.loc)
        self.check_scenario_lifespans()

    def check_scenario_lifespans(self):
        """L §6.4 from D: "an entry the scenario states into a kind the base declares (the prelude's included), or a
        replace of a base thing's entry, may change who is alive only from D. The new lifespan replaces the old one
        whole and agrees with it on every day before D (none, for a new thing), so a scenario ends a thing's life on
        D - 1 day or later ... and starts one on D or later (LOAD-SCENARIO otherwise)" (PL-13); L §8.3 `<s> changes
        who is alive in <K> on <d>, before <from>`. The old lifespan is the latest-known entry of the scenario's base
        worlds (refcore compares the entries as written, under no cut)."""
        m = self.m

        def chain(sc):
            out = [None]
            x = m.scenario_over.get(sc)
            while x is not None and x not in out:
                out.append(x)
                x = m.scenario_over.get(x)
            return out

        def before(sp, frm):
            """The days of span sp before frm, as (first, first day after) or None when there are none."""
            if sp is None:
                return None
            lo = None if sp.always else sp.start
            hi = frm if (sp.always or sp.end is None or sp.end > frm) else sp.end
            if lo is not None and lo >= hi:
                return None
            return (lo, hi)

        for t in m.things.values():
            for e in t.entries:
                v = e.version
                sc = getattr(v, 'scenario', None) if v is not None else None
                if sc is None:
                    continue
                k = m.kinds.get(e.kind)
                if k is not None and getattr(getattr(k, 'decl', None), '_scen', None) is not None:
                    continue    # a kind the scenario declares: only the scenario reads it
                frm = m.scen_from[sc]
                bases = chain(sc)
                olds = [o for o in t.entries if o is not e and o.kind == e.kind and
                        (o.version is None or getattr(o.version, 'scenario', None) in bases)]
                old = None
                for o in olds:
                    if old is None or (o.replace and (old.version is None or o.version is not None and
                                                      known_first(o.version.known) >= known_first(old.version.known))):
                        old = o
                a = before(old.span, frm) if old is not None else None
                b = before(e.span, frm)
                if a == b:
                    continue
                # the first day before from on which the two disagree
                one = datetime.timedelta(days=1)
                if a is None or b is None:
                    x = a if a is not None else b
                    d = x[0] if x[0] is not None else x[1] - one
                elif a[0] == b[0]:
                    d = min(a[1], b[1])
                elif a[0] is None or b[0] is None:
                    d = (a[0] or b[0]) - one
                else:
                    d = min(a[0], b[0])
                raise LoadError('LOAD-SCENARIO', '%s changes who is alive in %s on %s, before %s' % (
                    sc, e.kind, d, frm), e.loc)

    def check_key_ids(self):
        """A key literal that is also a top-level id is refused (L §2.5, LOAD-KEY)."""
        m = self.m
        for ix in m.indexes.values():
            for text in ix.texts:
                if isinstance(text, str) and text in m.names:
                    raise LoadError('LOAD-KEY', 'a key is also an id: %s (a key of %s)' % (text, ix.id),
                                    ix.decl.loc if ix.decl is not None else None)
        for t in m.things.values():
            if t.name in m.names and t.name not in CURRENCIES and m.names[t.name][0] not in ('unit', 'place'):
                raise LoadError('LOAD-KEY', 'a key is also an id: %s (a thing)' % t.name)
        for ax in m.axes.values():
            for o in ax.options:
                if o in m.names:
                    raise LoadError('LOAD-KEY', 'a key is also an id: %s (an option of %s)' % (o, ax.id))

    def dup(self, name, loc):
        raise LoadError('LOAD-DUP', '%s is declared twice' % name, loc)

    # -- names -----------------------------------------------------------
    def register_builtins(self):
        m = self.m

        class _BN:
            where = None
            on = None
            over = None
        # the built-in inputs of L §5.19: balance readings keyed by place, and the importer's rules
        specs = {'opening': ('x', ('kind', 'places'), None, True),
                 'stated': ('x', ('kind', 'places'), None, True),
                 'account_place': ('a', ('index', 'bank_account'), T.key('places'), False)}
        for b in BUILTIN_INPUTS:
            inf = InputInfo(b, None, None)
            var, dom, ty, reading = specs[b]
            inf.binds = [BindInfo(var, dom, _BN())]
            inf.type = ty
            inf.reading = reading
            inf.unit_factor = Fraction(1)
            m.inputs[b] = inf
            m.names[b] = ('input', inf)
        for k in PRELUDE_KINDS:
            m.names[k] = ('kind', None)
        m.names['cut'] = ('cut', None)
        for i in BUILTIN_INDEXES:
            m.names[i] = ('index', None)

    def register_names(self):
        m = self.m
        for s in self.sections:
            k = s.k
            sid = s.id
            if k == 'unitdecl':
                kind = 'unit'
            else:
                kind = k
            if sid in m.names:
                if sid in BUILTIN_IDS:
                    raise LoadError('LOAD-DUP', '%s is declared twice (a built-in id)' % sid, s.loc)
                self.dup(sid, s.loc)
            if k == 'band' and sid in BUILTINS:
                raise LoadError('LOAD-DUP', "band %s has a built-in's name" % sid, s.loc)
            if k == 'version':
                m.names[sid] = ('version', None)
            elif k == 'ask':
                m.names[sid] = ('ask', s)
            elif k == 'source':
                m.names[sid] = ('source', s)
                m.sources[sid] = s
            else:
                m.names[sid] = (kind, None)
    # -- units -----------------------------------------------------------
    def build_units(self):
        m = self.m
        for s in self.sections:
            if s.k != 'unitdecl':
                continue
            if s.cur:
                mv = s.minor
                if mv.k != 'num' or mv.unit is not None or mv.v <= 0:
                    raise LoadError('LOAD-UNIT', 'the minor unit of %s must be a positive number' % s.id, s.loc)
                CURRENCIES[s.id] = Fraction(mv.v)
            else:
                if s.id in PERIOD_WORDS:
                    raise LoadError('LOAD-DUP', '%s is declared twice (a period word)' % s.id, s.loc)
                m.units.add(s.id)

    # -- kinds -------------------------------------------------------------
    def build_kinds(self):
        m = self.m
        K = m.kinds
        K['places'] = KindInfo('places', 'place')
        K['events'] = KindInfo('events', 'event')
        K['currencies'] = KindInfo('currencies', 'unit')
        # L §5.7 The prelude (tally 2.1, R-2): "and the plain kind parties, the root of the model's owners (§3.5)"
        K['parties'] = KindInfo('parties', 'plain')
        for kid, (unit, sign, outside) in PRELUDE_PLACE.items():
            k = KindInfo(kid, 'place', 'places')
            k.unit, k.sign, k.outside = unit, sign, outside
            K[kid] = k
        for kid, legs, attrs in (('transfers', [('cash', 'any')], []), ('adjustments', [], [
                ('adjusts', None), ('change', None)]), ('pads', [('cash', 'any')], [])):
            k = KindInfo(kid, 'event', 'events')
            k.certainty = 'certain'
            k.legs = legs
            k.attrs = attrs
            K[kid] = k
        decls = []
        for s in self.sections:
            if s.k == 'kind':
                decls.append((s, None))
        for s, rid in decls:
            qid = s.id if rid is None else '%s.%s' % (rid, s.id)
            spec = {'plain': 'plain', 'sub': None, 'place': 'place', 'unit': 'unit', 'event': 'event'}[s.spec]
            parent = None
            if s.spec == 'place':
                parent = 'places'
            elif s.spec == 'event':
                parent = 'events'
            k = KindInfo(qid, spec, parent, s, rid)
            k.certainty = s.certainty
            k.unit = s.unit
            k.sign = s.sign
            k.priced = s.priced
            k.outside = s.outside
            k.legs = list(s.legs)
            k.attrs = list(s.attrs)
            k._subof = s.parent
            K[qid] = k
            m.names.setdefault(qid, ('kind', k))
            if rid is None:
                m.names[s.id] = ('kind', k)
        for kid in PRELUDE_KINDS:
            m.names[kid] = ('kind', K[kid])
        # subkinds
        for s, rid in decls:
            qid = s.id if rid is None else '%s.%s' % (rid, s.id)
            k = K[qid]
            if s.spec == 'sub':
                pid = s.parent
                hit = m.lookup(pid, rid)
                if hit is None or hit[0] != 'kind':
                    raise LoadError('LOAD-NAME', 'unknown id %s' % pid, s.loc)
                pk = hit[1]
                k.parent = pk.id
        for kid, k in K.items():
            if k.parent is not None:
                if k.parent not in K:
                    raise LoadError('LOAD-NAME', 'unknown id %s' % k.parent, k.decl.loc if k.decl else None)
                K[k.parent].children.append(kid)
        # cycles and the root spec of subkinds
        for kid, k in K.items():
            seen = set()
            x = k
            while x.parent is not None:
                if x.id in seen:
                    raise LoadError('LOAD-CYCLE', 'kind %s is a subkind of itself' % kid, k.decl.loc if k.decl else None)
                seen.add(x.id)
                x = K[x.parent]
        for kid, k in K.items():
            if k.spec is None:
                x = k
                while x.spec is None:
                    x = K[x.parent]
                k.spec = x.spec
                if k.spec == 'event':
                    # a subkind of an event kind keeps its parent's legs and attributes
                    pass
        for kid, k in K.items():
            if k.spec == 'event' and k.certainty is None:
                x = k
                while x.certainty is None and x.parent is not None:
                    x = K[x.parent]
                k.certainty = x.certainty
        for kid in K:
            K[kid].children.sort()

    # -- indexes -------------------------------------------------------------
    def build_indexes(self):
        m = self.m
        S, E = m.S, m.E
        # built-in period indexes: the periods that meet [S, E] (L §4.1)
        days = IndexInfo('days', 'builtin')
        days.pk = 'day'
        d = S
        while d <= E:
            days.keys.append(d)
            d = d + ONE_DAY
        months = IndexInfo('months', 'builtin')
        months.pk = 'month'
        y, mo = S.year, S.month
        while (y, mo) <= (E.year, E.month):
            months.keys.append(month_period(y, mo))
            y, mo = month_add(y, mo, 1)
        years = IndexInfo('years', 'builtin')
        years.pk = 'year'
        for yy in range(S.year, E.year + 1):
            years.keys.append(year_period(yy))
        for ix in (days, months, years):
            ix.finish()
            m.indexes[ix.id] = ix
            m.names[ix.id] = ('index', ix)
        for b in ('bank_account',):
            ix = IndexInfo(b, 'open')
            ix.finish()
            m.indexes[b] = ix
            m.names[b] = ('index', ix)
        for s in self.sections:
            if s.k != 'index':
                continue
            ix = IndexInfo(s.id, s.form, s)
            if s.form == 'list':
                for kn in s.keys:
                    t = self.key_source_text(kn, s.loc)
                    kv = Key(s.id, t)
                    if t in ix.texts:
                        raise LoadError('LOAD-DUP', 'a key entered twice: %s' % t, s.loc)
                    ix.texts[t] = kv
                    ix.keys.append(kv)
            elif s.form == 'range':
                self.build_range(ix, s)
            ix.finish()
            m.indexes[s.id] = ix
            m.names[s.id] = ('index', ix)

    def key_source_text(self, kn, loc):
        if kn.k == 'name':
            return kn.id
        if kn.k == 'key':
            return kn.v
        if kn.k == 'num' and kn.isint and kn.unit is None:
            return kn.text
        if kn.k in ('date',):
            return kn.v.isoformat()
        if kn.k == 'month':
            return '%04d-%02d' % kn.v
        if kn.k == 'quarter':
            return '%04d-Q%d' % kn.v
        if kn.k == 'mmdd':
            return kn.text
        if kn.k == 'cur':
            return kn.v
        raise LoadError('LOAD-KEY', '%s is not a key' % (kn.get('text') or kn.k), loc)

    def build_range(self, ix, s):
        lo, hi, pk = s.lo, s.hi, s.pk
        loc = s.loc

        def is4(n):
            return n.k == 'num' and n.isint and n.unit is None and len(n.text.lstrip('-')) == 4
        if pk is None:
            if lo.k != 'num' or hi.k != 'num' or not lo.isint or not hi.isint or lo.unit or hi.unit:
                raise LoadError('LOAD-PARSE', 'cannot parse: a range without a period word is of integers', loc)
            if is4(lo) and is4(hi):
                raise LoadError('LOAD-AMBIG', '%s..%s reads as a year and as an integer here' % (lo.text, hi.text), loc)
            pk = 'int'
        if pk == 'int':
            if lo.k != 'num' or hi.k != 'num' or not lo.isint or not hi.isint:
                raise LoadError('LOAD-PARSE', 'cannot parse: an integer range', loc)
            a, b = int(lo.v), int(hi.v)
            if s.frm is not None:
                raise LoadError('LOAD-PARSE', 'cannot parse: from on an integer range', loc)
            step = s.by or 1
            if a > b:
                raise LoadError('LOAD-KEY', 'range %d..%d is out of order' % (a, b), loc)
            ix.isint = True
            ix.keys = list(range(a, b + 1, step))
            return
        ix.pk = pk
        n = s.by or 1
        if pk == 'year':
            if not (is4(lo) and is4(hi)):
                raise LoadError('LOAD-PARSE', 'cannot parse: a year range', loc)
            a, b = int(lo.v), int(hi.v)
            if a > b:
                raise LoadError('LOAD-KEY', 'range %d..%d is out of order' % (a, b), loc)
            mm, dd = 1, 1
            if s.frm is not None:
                if s.frm[0] != 'mmdd':
                    raise LoadError('LOAD-PARSE', 'cannot parse: a year range is anchored by a month-day', loc)
                mm, dd = s.frm[1]
                try:
                    datetime.date(2001, mm, dd)
                except ValueError:
                    # L §2.3 index: "MMDD is "04-06", a month-day every year has, so never 02-29, LOAD-PARSE"
                    raise LoadError('LOAD-PARSE', 'cannot parse: %02d-%02d is not a month-day every year has' % (mm, dd),
                                    loc)
            yy = a
            while yy <= b:
                st = datetime.date(yy, mm, dd)
                en = datetime.date(yy + n, mm, dd)
                ix.keys.append(Period('year', st, en, 'month', 12 * n))
                yy += n
        elif pk == 'month':
            if lo.k != 'month' or hi.k != 'month':
                raise LoadError('LOAD-PARSE', 'cannot parse: a month range', loc)
            dd = 1
            if s.frm is not None:
                if s.frm[0] != 'day' or not 1 <= s.frm[1] <= 28:
                    raise LoadError('LOAD-PARSE', 'cannot parse: a month range is anchored by a day 1-28', loc)
                dd = s.frm[1]
            y, mo = lo.v
            if lo.v > hi.v:
                raise LoadError('LOAD-KEY', 'range out of order', loc)
            while (y, mo) <= hi.v:
                st = datetime.date(y, mo, dd)
                ix.keys.append(Period('month', st, add_months(st, n), 'month', n))
                y, mo = month_add(y, mo, n)
        elif pk == 'day':
            if lo.k != 'date' or hi.k != 'date':
                raise LoadError('LOAD-PARSE', 'cannot parse: a day range', loc)
            if s.frm is not None:
                raise LoadError('LOAD-PARSE', 'cannot parse: from on a day range', loc)
            if lo.v > hi.v:
                raise LoadError('LOAD-KEY', 'range out of order', loc)
            d = lo.v
            while d <= hi.v:
                if n == 1:
                    ix.keys.append(d)
                else:
                    ix.keys.append(Period('day', d, add_days(d, n), 'day', n))
                d = add_days(d, n)

    def build_axes(self):
        m = self.m
        for s in self.sections:
            if s.k == 'axis':
                a = AxisInfo(s)
                if len(set(a.options)) != len(a.options):
                    raise LoadError('LOAD-DUP', 'a key entered twice in axis %s' % s.id, s.loc)
                m.axes[s.id] = a
                m.names[s.id] = ('axis', a)

    # -- versions ------------------------------------------------------------
    def build_versions(self):
        m = self.m
        for s in self.sections:
            if s.k != 'version':
                continue
            v = VersionInfo(s, m)
            v.scenario = getattr(s, '_scen', None)
            m.versions[s.id] = v
            m.names[s.id] = ('version', v)
            hit = m.names.get(s.cite)
            if hit is None or hit[0] != 'source':
                raise LoadError('LOAD-NAME', 'unknown id %s (version %s cites no source)' % (s.cite, s.id), s.loc)
            kn = v.known
            if isinstance(kn, tuple):
                if kn[0] > kn[1]:
                    raise LoadError('LOAD-DOMAIN', 'window(a, b) with a after b (L §8.4)', s.loc)
                if kn[0] > m.on:
                    raise LoadError('LOAD-KNOWN', 'version %s is known after the model\'s on date' % s.id, s.loc)
                if kn[1] > m.on:
                    # L §4.5 (round 12): "A version or row known after on, or whose known window ends after it, is
                    # refused (LOAD-KNOWN)"
                    raise LoadError('LOAD-KNOWN', 'version %s is known after the model\'s on date' % s.id, s.loc)
            elif kn > m.on:
                raise LoadError('LOAD-KNOWN', 'version %s is known after the model\'s on date' % s.id, s.loc)
        for vid, v in m.versions.items():
            for ref, ls in v.node.moves:
                if ref.k != 'name':
                    raise LoadError('LOAD-MOVES', 'moves %s: unknown, or not known before this version' % ref, v.loc)
                tgt = m.versions.get(ref.id)
                if tgt is None or not (known_first(tgt.known) < known_first(v.known)):
                    raise LoadError('LOAD-MOVES', 'moves %s: unknown, or not known before this version' % ref.id, v.loc)
                tgt.moved_by.append((v, lspan_to_span(ls, v.loc)))
        for vid, v in m.versions.items():
            seen = {}
            for mover, sp in v.moved_by:
                k = known_first(mover.known)
                if k in seen:
                    raise LoadError('LOAD-MOVES', 'two moves of %s known the same day' % vid, mover.loc)
                seen[k] = mover
        for src in m.sources:
            if not any(v.cite == src for v in m.versions.values()):
                m.warnings.append(('WARN-UNREAD', 'source %s is read by nothing' % src))

    # -- declarations: inputs, bands, defs -----------------------------------------
    def build_kind_types(self):
        """The declared types of event kinds' attributes; the prelude's adjustments have none (L §5.7)."""
        m = self.m
        for kid, k in m.kinds.items():
            k.attr_types = {}
            for name, t in k.attrs:
                if t is None:
                    k.attr_types[name] = None
                    continue
                k.attr_types[name], _ = self.decl_type(t, [], k.regime, k.decl.loc if k.decl else None)
            if k.spec == 'event':
                for name, u in k.legs:
                    if u == 'any':
                        continue
                    if u in CURRENCIES or u in m.units:
                        continue
                    hit = m.lookup(u, k.regime)
                    if hit is not None and hit[0] == 'kind' and m.kinds[hit[1].id].spec == 'unit':
                        continue
                    if len(u) == 3 and u.isupper():
                        raise LoadError('LOAD-UNIT', 'money in an undeclared currency %s' % u, k.decl.loc)
                    raise LoadError('LOAD-NAME', 'unknown id %s (the unit of leg %s of %s)' % (u, name, kid),
                                    k.decl.loc if k.decl else None)
            if k.spec == 'place' and k.decl is not None:
                u = k.unit
                if u not in (None, 'any', 'one') and u not in CURRENCIES and u not in m.units:
                    hit = m.lookup(u, k.regime)
                    if not (hit is not None and hit[0] == 'kind' and m.kinds[hit[1].id].spec == 'unit'):
                        if len(u) == 3 and u.isupper():
                            raise LoadError('LOAD-UNIT', 'money in an undeclared currency %s' % u, k.decl.loc)
                        raise LoadError('LOAD-NAME', 'unknown id %s (the unit of kind %s)' % (u, kid), k.decl.loc)

    def build_decls(self):
        m = self.m
        self.build_kind_types()
        decls = []
        for s in self.sections:
            if s.k in ('input', 'band', 'def', 'place', 'fill', 'move'):
                decls.append((s, None))
        self.decls = decls
        for s, rid in decls:
            qid = s.id if rid is None else '%s.%s' % (rid, s.id)
            if s.k == 'input':
                inf = InputInfo(qid, s, rid)
                inf.binds = self.decl_binds(s.binds, rid, s.loc, 'input')
                if s.observed:
                    inf.type = None
                else:
                    inf.type, inf.unit_factor = self.decl_type(s.type, inf.binds, rid, s.loc)
                    if inf.type.k == 'dist' and False:
                        pass
                inf.time_pos = self.time_pos(inf.binds)
                if s.prices is not None:
                    if inf.type.k != 'money':
                        raise LoadError('LOAD-UNIT', 'prices on an input of type %s; prices is for money' % T.show(inf.type), s.loc)
                    p = s.prices
                    if not (p.k == 'num' and p.isint and p.unit is None and len(p.text) == 4):
                        raise LoadError('LOAD-PARSE', 'cannot parse: prices takes a year', s.loc)
                    inf.prices = int(p.v)
                    if s.prices_by is not None:
                        if s.prices_by.k != 'name':
                            raise LoadError('LOAD-PARSE', 'cannot parse: prices by an index input', s.loc)
                        inf.prices_by = s.prices_by.id
                m.inputs[qid] = inf
                if rid is None:
                    m.names[qid] = ('input', inf)
            elif s.k == 'def':
                d = DefInfo(qid, s, rid)
                d.binds = self.decl_binds(s.binds, rid, s.loc, 'def')
                d.type, d.unit_factor = self.decl_type(s.type, d.binds, rid, s.loc)
                d.time_pos = self.time_pos(d.binds)
                for r in s.rules:
                    if r.label in d.rules:
                        raise LoadError('LOAD-LABEL', 'rule %s already exists in %s; write replace' % (r.label, qid),
                                        r.loc)
                    v = None
                    if r.v is not None:
                        v = m.versions.get(r.v)
                        if v is None:
                            # L §5.11 step 2 (round 19, GD-24): "a v that names no version of the reality is refused,
                            # LOAD-NAME"; L §8.3 `rule <d>.<r>: v <x> names no version`
                            raise LoadError('LOAD-NAME', 'rule %s.%s: v %s names no version' % (qid, r.label, r.v),
                                            r.loc)
                    span = lspan_to_span(r.governs, r.loc) if r.governs is not None else None
                    d.rules[r.label] = [RuleVer(r.label, r, None, v, span, False, r.loc)]
                    d.labels.append(r.label)
                m.defs[qid] = d
                if rid is None:
                    m.names[qid] = ('def', d)
            elif s.k == 'band':
                b = BandInfo(qid, s, rid)
                b.binds = self.decl_binds(s.binds, rid, s.loc, 'band')
                if len(b.binds) > 1:
                    raise LoadError('LOAD-PARSE', 'cannot parse: a keyed band has one key', s.loc)
                b.argtype, _ = self.decl_type(s.argtype, b.binds, rid, s.loc)
                if b.argtype.k == 'key' and b.argtype.ix in m.kinds:
                    # L §5.9 (round 12): "The argument is a number, a date, a period or a key of an index, never a
                    # thing (LOAD-UNIT): things order by entry, which the cut changes"
                    raise LoadError('LOAD-UNIT', 'band %s over things of %s' % (qid, b.argtype.ix), s.loc)
                b.type, b.unit_factor = self.decl_type(s.type, b.binds, rid, s.loc)
                m.bands[qid] = b
                if rid is None:
                    m.names[qid] = ('band', b)
            elif s.k == 'place':
                pi = PlaceInfo(qid, s, rid)
                pi.binds = self.decl_binds(s.binds, rid, s.loc, 'place')
                for b in pi.binds:
                    if b.node.where is not None or b.node.on is not None or b.node.over is not None:
                        raise LoadError('LOAD-DOMAIN', 'the key bind of place %s is defined for every key (L §5.7)'
                                        % qid, s.loc)
                m.places[qid] = pi
                if rid is None:
                    m.names[qid] = ('place', pi)
            elif s.k == 'fill':
                m.fills[qid] = s
                if rid is None:
                    m.names[qid] = ('fill', s)
            elif s.k == 'move':
                mi = MoveInfo(qid, s, rid)
                mi.binds = self.decl_binds(s.binds, rid, s.loc, 'move')
                m.moves[qid] = mi
                if rid is None:
                    m.names[qid] = ('move', mi)
        for s in self.sections:
            if s.k == 'ask':
                m.asks.append(s)
                m.ask_by_id[s.id] = s

    def time_pos(self, binds):
        tp = [i for i, b in enumerate(binds) if b.dom[0] == 'index' and self.m.indexes[b.dom[1]].pk is not None]
        if not tp:
            return None
        if len(tp) > 1:
            return 'multi'
        return tp[0]

    def resolve_domain(self, name, qual, rid, loc):
        m = self.m
        if qual is not None:
            hit = m.qualified(qual, name)
            if hit is None:
                raise LoadError('LOAD-NAME', 'unknown id %s.%s' % (qual, name), loc)
        else:
            hit = m.lookup(name, rid)
        if hit is None:
            raise LoadError('LOAD-NAME', 'unknown id %s' % name, loc)
        kind, info = hit
        if kind == 'index':
            return ('index', name if info is None else info.id)
        if kind == 'kind':
            return ('kind', info.id if info is not None else name)
        if kind == 'axis':
            return ('axis', name)
        raise LoadError('LOAD-KEY', '%s is not an index, a kind or an axis' % name, loc)

    def decl_binds(self, binds, rid, loc, what):
        out = []
        seen = set()
        for b in binds:
            if b.var in seen:
                raise LoadError('LOAD-DUP', 'bound variable %s is declared twice' % b.var, loc)
            seen.add(b.var)
            if b.var in self.m.names:
                raise LoadError('LOAD-SHADOW', 'bound variable %s has the name of a top-level id' % b.var, loc)
            if b.on is not None or b.over is not None:
                if what in ('def', 'input', 'band', 'place'):
                    # the key bind of a def, input or place "is defined for every key" (L §5.6, §5.7): on and
                    # over have no meaning there (L §8.4 What isn't defined is refused)
                    raise LoadError('LOAD-DOMAIN', 'an on or over clause on the key bind of a %s, which is defined '
                                    'for every key' % what, loc)
            d = b.dom
            if d.k == 'dname':
                dom = self.resolve_domain(d.id, d.qual, rid, loc)
                if (dom[0] == 'index' and b.var in self.m.indexes[dom[1]].texts) or (
                        dom[0] == 'kind' and self.m.thing_in(b.var, dom[1])) or (
                        dom[0] == 'axis' and b.var in self.m.axes[dom[1]].options):
                    raise LoadError('LOAD-SHADOW', 'bound variable %s has the spelling of a key of %s' % (b.var, dom[1]),
                                    loc)
            elif d.k == 'dlist':
                dom = ('list', d.keys, None)
            else:
                dom = ('range', d)
            bi = BindInfo(b.var, dom, b)
            out.append(bi)
        return out

    def decl_type(self, t, binds, rid, loc):
        """Parser type tuple -> (Ty, factor of a % in its unit)."""
        m = self.m
        k = t[0]
        if k == 'money':
            if t[1] not in CURRENCIES:
                raise LoadError('LOAD-UNIT', 'money in an undeclared currency %s' % t[1], loc)
            return T.money(t[1]), Fraction(1)
        if k in ('num', 'dist'):
            u, f = self.unit_of(t[1], binds, loc)
            if k == 'num' and t[1] == (('%', 1),):
                # `num %` differs from `num` only in how it prints (L §3.2)
                return T.pct(), f
            return (T.num(u) if k == 'num' else T.Ty('dist', u=u)), f
        if k == 'key':
            name = t[1]
            if '.' in name:
                q, x = name.split('.', 1)
                hit = m.qualified(q, x)
            else:
                hit = m.lookup(name, rid)
            if hit is None:
                raise LoadError('LOAD-NAME', 'unknown id %s' % name, loc)
            kind, info = hit
            if kind == 'kind':
                return T.key(info.id), Fraction(1)
            if kind in ('index', 'axis'):
                return T.key(name if info is None else info.id), Fraction(1)
            raise LoadError('LOAD-KEY', '%s is not an index or a kind' % name, loc)
        if k == 'record':
            fs = []
            names = set()
            for f, ft in t[1]:
                if f in names:
                    raise LoadError('LOAD-DUP', 'field %s twice' % f, loc)
                names.add(f)
                fty, _ = self.decl_type(ft, binds, rid, loc)
                fs.append((f, fty))
            return T.record(fs), Fraction(1)
        simple = {'int': T.INT, 'bool': T.BOOL, 'text': T.TEXT, 'date': T.DATE, 'dates': T.DATES,
                  'month': T.period('month'), 'quarter': T.period('quarter'), 'year': T.period('year'),
                  'monthday': T.MD, 'rate': T.RATE, 'calendar': T.CAL, 'rounding': T.code('rounding'),
                  'daycount': T.code('daycount'), 'roll': T.code('roll'), 'compounding': T.code('compounding')}
        return simple[k], Fraction(1)

    def unit_of(self, syn, binds, loc):
        if syn is None:
            return (), Fraction(1)
        vars_ = {b.var for b in binds}
        terms = []
        for b, e in syn:
            if b == '%':
                terms.append((b, e))
            elif b in vars_:
                terms.append(('$' + b, e))
            elif b in CURRENCIES or b in self.m.units or b in ('year', 'quarter', 'month', 'day'):
                terms.append((b, e))
            elif len(b) == 3 and b.isupper():
                raise LoadError('LOAD-UNIT', 'money in an undeclared currency %s' % b, loc)
            elif self.is_unit_key(b):
                terms.append((b, e))
            else:
                raise LoadError('LOAD-NAME', 'unknown id %s (a unit)' % b, loc)
        return U.from_syntax(terms)

    def is_unit_key(self, b):
        return b in self.m.unit_kind_keys

    # -- entries -------------------------------------------------------------
    def build_entries(self):
        m = self.m
        # currencies: keys of the prelude unit kind, alive always (L §5.4)
        for c in CURRENCIES:
            t = ThingInfo(c)
            t.base = 'currencies'
            t.entries.append(Entry('currencies', ALWAYS, None, False, None, None, None))
            m.things[c] = t
        # in known order, so a replace finds the earlier rows whatever the file order (L §6.1: a reality is a set)
        for v in sorted(m.versions.values(), key=lambda v: known_first(v.known)):
            for it in v.node.items:
                if it.k != 'enter':
                    continue
                self.entry(v, it)
        # base kinds
        for name, t in m.things.items():
            if t.base is not None:
                continue
            firsts = [e for e in t.entries if not e.replace]
            nonplain = sorted(set(e.kind for e in firsts if m.kinds[e.kind].spec != 'plain'))
            # L §5.7 (round 12): "A key enters at most one kind that isn't plain, which is its base; a key that enters
            # only plain kinds has as base the kind of its earliest entry, by known, then by its lifespan's start. Two
            # kinds that aren't plain, or two earliest plain entries tied on both, are refused (LOAD-DUP: a key
            # entered twice)."
            if len(nonplain) > 1:
                e2 = [e for e in firsts if e.kind == nonplain[1]][0]
                raise LoadError('LOAD-DUP', 'a key entered twice: %s (%s)' % (name, ', '.join(nonplain)), e2.loc)
            if nonplain:
                t.base = nonplain[0]
            else:
                def order(e):
                    kn = known_first(e.version.known) if e.version is not None else datetime.date.min
                    st = e.span.start if e.span is not None and e.span.start is not None else datetime.date.min
                    return (kn, st)
                ks = sorted(firsts, key=order)
                if len(ks) > 1 and order(ks[0]) == order(ks[1]) and ks[0].kind != ks[1].kind:
                    raise LoadError('LOAD-DUP', 'a key entered twice: %s (%s, %s, known and starting the same day)'
                                    % (name, ks[0].kind, ks[1].kind), ks[1].loc)
                t.base = ks[0].kind
        for name, t in m.things.items():
            for e in t.entries:
                if m.kinds[e.kind].spec == 'unit':
                    m.unit_kind_keys[name] = e.kind

    def entry(self, v, it):
        m = self.m
        q, x = it.qual, it.target
        if q is not None:
            hit = m.qualified(q, x)
        else:
            hit = m.lookup(x)
        if hit is None:
            raise LoadError('LOAD-NAME', 'unknown id %s' % x, it.loc)
        kind, info = hit
        if kind == 'index':
            ix = m.indexes[x]
            if ix.form != 'list':
                raise LoadError('LOAD-KIND', 'an addition to a %s index %s' % (
                    'range' if ix.form in ('range', 'builtin') else 'open', x), it.loc)
            if it.replace or it.lives is not None or it.unit is not None or it.priced is not None:
                raise LoadError('LOAD-PARSE', 'cannot parse: an index entry holds keys only', it.loc)
            for kn in it.keys:
                t = self.key_source_text(kn, it.loc)
                if t in ix.texts:
                    raise LoadError('LOAD-DUP', 'a key entered twice: %s' % t, it.loc)
                kv = Key(x, t)
                ix.texts[t] = kv
                ix.keys.append(kv)
                ix.added[_kv(kv)] = v
            ix.finish()
            return
        if kind != 'kind':
            raise LoadError('LOAD-NAME', '%s is not a kind or an index' % x, it.loc)
        k = info
        if k.children:
            raise LoadError('LOAD-LEAF', '%s enters %s, which has subkinds' % (
                ', '.join(self.key_source_text(n, it.loc) for n in it.keys), k.id), it.loc)
        if it.lives is None:
            raise LoadError('LOAD-LIFESPAN', '%s enters %s without lives' % (
                ', '.join(self.key_source_text(n, it.loc) for n in it.keys), k.id), it.loc)
        if k.spec == 'event':
            raise LoadError('LOAD-KIND', 'an entry into the event kind %s' % k.id, it.loc)
        if k.spec == 'place':
            ku = self.place_unit(k)
            if ku == 'one' and it.unit is None:
                raise LoadError('LOAD-PLACEKIND', '%s enters a kind of unit one without its unit' % (
                    self.key_source_text(it.keys[0], it.loc)), it.loc)
            if ku != 'one' and it.unit is not None:
                # L §5.10: "A unit is stated only for a kind of unit one ...; a place of any other kind has its kind's
                # unit, a stated one or any (as outside), so a unit stated on it is refused (LOAD-UNIT, P-3)"
                raise LoadError('LOAD-UNIT', '%s states %s; its kind %s is of unit %s, not one' % (
                    self.key_source_text(it.keys[0], it.loc), it.unit, k.id, ku), it.loc)
        elif it.unit is not None or it.priced is not None:
            raise LoadError('LOAD-PARSE', 'cannot parse: a unit on an entry into %s, not a place kind' % k.id, it.loc)
        span = lspan_to_span(it.lives, it.loc)
        if span.start is None and not span.always:
            # L §5.7 (round 12): "a lifespan has a start (lives ..b is refused, LOAD-LIFESPAN)"; L §8.3 "<k>'s lifespan
            # has no start"
            raise LoadError('LOAD-LIFESPAN', '%s\'s lifespan has no start' % ', '.join(
                self.key_source_text(n, it.loc) for n in it.keys), it.loc)
        for kn in it.keys:
            name = self.key_source_text(kn, it.loc)
            if name in m.names:
                raise LoadError('LOAD-KEY', 'a key is also an id: %s' % name, it.loc)
            t = m.things.get(name)
            if t is None:
                t = ThingInfo(name)
                m.things[name] = t
            olds = [e for e in t.entries if e.kind == k.id]
            if getattr(m, 'joint', None) is not None:
                # a thing two members (or a member and the joint model) state same: each side's entries are its own
                # facts, read together (L §6.1 What same does), so a side's replace finds its own earlier entry
                from .joint import owner_deep
                ow = owner_deep(v.id) if v.scenario is None else ''
                olds = [e for e in olds if e.version is None or
                        (owner_deep(e.version.id) if e.version.scenario is None else '') == ow]
            if it.replace:
                # in a scenario, a base entry is an earlier row whatever its known: "a base row known on the
                # scenario's own known day is replaced too" (L §6.4, §4.6 step 2; MC-X-F2)
                lv = version_layer(m, v)
                v._layer = lv
                if not any(known_first(e.version.known) < known_first(v.known) or version_layer(m, e.version) < lv
                           for e in olds if e.version):
                    raise LoadError('LOAD-REPLACE', 'replace %s + [%s]: no earlier row to replace' % (k.id, name), it.loc)
                for e in olds:
                    if e.version and e.replace and known_first(e.version.known) == known_first(v.known) and \
                            version_layer(m, e.version) == lv:
                        # neither is known before the other and each replaces the lifespan whole: a tie (L §4.6 steps
                        # 2 and 4, LOAD-OVERLAP; round 18 C1-11 took `two replace lines ... known the same day` out of
                        # LOAD-REPLACE)
                        raise LoadError('LOAD-OVERLAP', 'entries %s and %s of %s + [%s] overlap, neither replacing the '
                                                        'other' % (e.loc, it.loc, k.id, name), it.loc)
            else:
                if any(not e.replace for e in olds):
                    raise LoadError('LOAD-DUP', 'a key entered twice: %s in %s' % (name, k.id), it.loc)
            t.entries.append(Entry(k.id, span, v, it.replace, it.unit, it.priced, it.loc))

    def place_unit(self, k):
        m = self.m
        x = k
        while x.unit is None and x.parent is not None:
            x = m.kinds[x.parent]
        return x.unit

    def build_places(self):
        m = self.m
        for qid, pi in m.places.items():
            s = pi.decl
            hit = m.lookup(s.kind, pi.regime)
            if hit is None or hit[0] != 'kind':
                raise LoadError('LOAD-NAME', 'unknown id %s' % s.kind, s.loc)
            k = hit[1]
            if k.spec != 'place':
                raise LoadError('LOAD-KIND', 'place %s of %s, not a place kind' % (qid, k.id), s.loc)
            if k.children:
                raise LoadError('LOAD-LEAF', '%s enters %s, which has subkinds' % (qid, k.id), s.loc)
            ku = self.place_unit(k)
            if ku == 'one' and s.unit is None:
                raise LoadError('LOAD-PLACEKIND', '%s enters a kind of unit one without its unit' % qid, s.loc)
            if s.unit is not None:
                self.check_unit_word(s.unit, pi.regime, s.loc)
                if ku != 'one':
                    # L §5.10: "a unit stated on it is refused (LOAD-UNIT, P-3)", for a kind not of unit one
                    raise LoadError('LOAD-UNIT', 'place %s states %s; its kind %s is of unit %s, not one' % (
                        qid, s.unit, k.id, ku), s.loc)
            pi.kind = k.id
            pi.lives = lspan_to_span(s.lives, s.loc)
            if pi.lives.start is None and not pi.lives.always:
                # L §5.7 (round 12): "a lifespan has a start (lives ..b is refused, LOAD-LIFESPAN)"
                raise LoadError('LOAD-LIFESPAN', '%s\'s lifespan has no start' % qid, s.loc)
            if not pi.binds:
                name = qid
                t = ThingInfo(name)
                t.base = k.id
                t.entries.append(Entry(k.id, pi.lives, None, False, s.unit, s.priced, s.loc))
                if name in m.things:
                    self.dup(name, s.loc)
                m.things[name] = t
                pi.things[()] = t
                continue
            doms = []
            for b in pi.binds:
                d = b.dom
                if d[0] == 'kind':
                    sub = m.kinds[d[1]].subtree(m)
                    ks = []
                    for tn, t in sorted(m.things.items()):
                        if isinstance(t, PlaceThing):
                            continue
                        if any(e.kind in sub for e in t.entries):
                            ks.append(('thing', t))
                    doms.append(ks)
                elif d[0] == 'index':
                    ix = m.indexes[d[1]]
                    # an open index's keys come from the rows (prescan_open_keys); each keyed place exists from
                    # its key's known (L §4.1 open index, §5.10)
                    doms.append([('key', ix, k2) for k2 in ix.keys])
                else:
                    raise LoadError('LOAD-DOMAIN', 'place %s is keyed by %s; a place is keyed by kinds or indexes'
                                    % (qid, b.var), s.loc)
            tuples = [[]]
            for dl in doms:
                tuples = [t + [x] for t in tuples for x in dl]
            for tup in tuples:
                keys = []
                kthings = []
                kidx = []
                for x in tup:
                    if x[0] == 'thing':
                        keys.append(Thing(x[1].name))
                        kthings.append(x[1])
                    else:
                        keys.append(x[2])
                        if x[1].added.get(_kv(x[2])) is not None:
                            kidx.append((x[1], x[2]))
                name = '%s[%s]' % (qid, ', '.join(key_text(kk) for kk in keys))
                t = PlaceThing(name, pi, keys, kthings, kidx)
                m.things[name] = t
                pi.things[tuple(_kv(kk) for kk in keys)] = t

    def check_unit_word(self, u, rid, loc):
        m = self.m
        if u in CURRENCIES or u in m.units:
            return
        if u in m.unit_kind_keys:
            return
        if len(u) == 3 and u.isupper():
            raise LoadError('LOAD-UNIT', 'money in an undeclared currency %s' % u, loc)
        raise LoadError('LOAD-NAME', 'unknown id %s (a unit)' % u, loc)

    def place_ref(self, ref, loc, code='LOAD-NAME', allow_hole=False):
        """A place written in a row, a covers line or a gap role: a place's id, a keyed place konto[ana],
        or a thing of a place kind (L §5.10)."""
        m = self.m
        if ref.k == 'name':
            hit = m.lookup(ref.id)
            if hit is not None and hit[0] == 'place':
                pi = hit[1]
                if pi.binds:
                    raise LoadError('LOAD-KEY', 'wrong number of keys for place %s' % pi.id, loc)
                return Thing(pi.things[()].name)
            t = m.things.get(ref.id)
            if t is not None and m.is_place_thing(t):
                return Thing(t.name)
            if code == 'LOAD-COVERS':
                raise LoadError('LOAD-COVERS', '%s is not a place' % ref.id, loc)
            if hit is None and t is None:
                raise LoadError('LOAD-NAME', 'unknown id %s' % ref.id, loc)
            raise LoadError(code if code != 'LOAD-NAME' else 'LOAD-KIND', '%s is not a place' % ref.id, loc)
        if ref.k in ('index', 'placekey'):
            name = ref.obj.id if ref.k == 'index' and ref.obj.k == 'name' else (ref.id if ref.k == 'placekey' else None)
            keys = ref.keys
            hit = m.lookup(name) if name is not None else None
            if hit is None or hit[0] != 'place':
                if code == 'LOAD-COVERS':
                    raise LoadError('LOAD-COVERS', '%s is not a place' % name, loc)
                raise LoadError('LOAD-NAME', 'unknown id %s' % name, loc)
            return Thing(self.keyed_place(hit[1], keys, loc).name)
        if code == 'LOAD-COVERS':
            raise LoadError('LOAD-COVERS', '%s is not a place' % resolve_text(ref), loc)
        raise LoadError('LOAD-KIND', 'a place is written by its id: %s' % ref.k, loc)

    def keyed_place(self, pi, keynodes, loc):
        m = self.m
        if len(keynodes) != len(pi.binds):
            raise LoadError('LOAD-KEY', 'wrong number of keys for place %s' % pi.id, loc)
        kvs = []
        for kn, b in zip(keynodes, pi.binds):
            kvs.append(_kv(resolve_key_literal(m, kn, b.dom, loc)))
        t = pi.things.get(tuple(kvs))
        if t is None:
            raise LoadError('LOAD-KEY', 'no place %s at these keys' % pi.id, loc)
        return t

    # -- version items ---------------------------------------------------------
    def build_items(self):
        m = self.m
        for vid, v in m.versions.items():
            needs_governs = False
            tables = {}
            # covers x [span] (L §5.3, §5.19)
            v.covers = []
            for ref, ls in v.node.covers:
                th = self.place_ref(ref, v.loc, code='LOAD-COVERS')
                v.covers.append((th, lspan_to_span(ls, v.loc) if ls is not None else None))
            if v.covers and v.governs is None and any(sp is None for th, sp in v.covers):
                raise LoadError('LOAD-GOVERNS', 'version %s covers a place for its governs span and has no governs'
                                % vid, v.loc)
            # gap roles: pad x from y, anchor x (L §5.3)
            v.gap_roles = {}
            for kind_, xr, yr in v.node.gaps:
                x = self.place_ref(xr, v.loc)
                y = self.place_ref(yr, v.loc) if yr is not None else None
                if x.name in v.gap_roles:
                    raise LoadError('LOAD-GAPROLE', '%s states two gap roles for %s' % (vid, x.name), v.loc)
                v.gap_roles[x.name] = (kind_, y)
            for it in v.node.items:
                if it.k == 'enter':
                    continue
                if it.k == 'same':
                    m.sames.append((v, it))
                    continue
                if it.k == 'rulerep':
                    self.rule_item(v, it)
                    if it.rule.governs is None:
                        needs_governs = True
                    continue
                if it.k == 'bandrow':
                    self.band_item(v, it, tables)
                    needs_governs = True
                    continue
                if it.k == 'row':
                    own_time = self.row_item(v, it)
                    if not own_time:
                        needs_governs = True
            # LOAD-GAPROLE: stated readings of a place with neither pad nor anchor for it (L §5.3, §5.19)
            for r in m.inputs['stated'].rows:
                if r.version is v and r.pattern and r.pattern[0][0] == 'k':
                    nm = r.pattern[0][1].name
                    if nm not in v.gap_roles:
                        raise LoadError('LOAD-GAPROLE', '%s states balance readings of %s with neither pad nor anchor'
                                        % (vid, nm), r.loc)
            if needs_governs and v.governs is None:
                raise LoadError('LOAD-GOVERNS', 'version %s has rows with no time of their own and no governs' % vid,
                                v.loc)

    def target(self, q, x, loc, allowed):
        m = self.m
        if q is not None:
            hit = m.qualified(q, x)
            if hit is None:
                raise LoadError('LOAD-NAME', 'unknown id %s.%s' % (q, x), loc)
        else:
            hit = m.lookup(x)
            if hit is None:
                raise LoadError('LOAD-NAME', 'unknown id %s' % x, loc)
        kind, info = hit
        if kind not in allowed:
            raise LoadError('LOAD-NAME', '%s is not %s' % (x, ' or '.join(allowed)), loc)
        return kind, info

    def rule_item(self, v, it):
        kind, d = self.target(it.qual, it.d, it.loc, ('def',))
        r = it.rule
        vv = None
        if r.v is not None and r.v != v.id and r.v not in self.m.versions:
            # L §5.11 step 2 (round 19, C4-4): a v naming no version at all is LOAD-NAME
            raise LoadError('LOAD-NAME', 'rule %s.%s: v %s names no version' % (it.d, it.label, r.v), it.loc)
        if r.v is not None and r.v != v.id:
            # L §5.11 step 2 (round 19, C4-4): "a v on it may name only that version and adds nothing (LOAD-VERSION
            # where it names another version)"
            # L §8.3 `replace rule <d>.<r> in <V> names version <W>`
            raise LoadError('LOAD-VERSION', 'replace rule %s.%s in %s names version %s' % (it.d, it.label, v.id, r.v),
                            it.loc)
        span = lspan_to_span(r.governs, it.loc) if r.governs is not None else None
        rv = RuleVer(it.label, r, v, None, span, True, it.loc)
        if it.label not in d.rules:
            d.rules[it.label] = []
            d.labels.append(it.label)
        d.rules[it.label].append(rv)

    def band_item(self, v, it, tables):
        kind, b = self.target(it.qual, it.target, it.loc, ('band',))
        key = None
        if it.key is not None:
            if not b.binds:
                raise LoadError('LOAD-KEY', 'band %s has no key' % b.id, it.loc)
            key = self.pattern_key(it.key, b.binds[0], it.loc)
        elif b.binds:
            raise LoadError('LOAD-KEY', 'wrong number of keys: band %s is keyed' % b.id, it.loc)
        tk = (b.id, _kv(key) if key is not None else None)
        t = tables.get(tk)
        if t is None:
            t = Table(b, v, key, it.loc)
            tables[tk] = t
            b.tables.append(t)
        iv = it.interval
        lo = None if iv.lo is None else self.band_end(iv.lo, b, it.loc)
        hi = None if iv.hi is None else self.band_end(iv.hi, b, it.loc)
        t.rows.append(BandRow(lo, iv.lo_closed, hi, iv.hi_closed, it.value, it.locator, it.replace, it.loc, it))

    def band_end(self, node, b, loc):
        val = convert_value(self.m, node, b.argtype, loc, Fraction(1), 'band %s' % b.id)
        return order_key(val, self.m)

    def pattern_key(self, kn, bind, loc, version=None):
        """Resolve a key literal at a key position (L §2.5 Keys); a row's key of an open index brings the key
        (L §5.6: it exists from the earliest known of the versions whose rows bring it)."""
        m = self.m
        if bind.dom[0] == 'index' and m.indexes[bind.dom[1]].form == 'open' and version is not None:
            ix = m.indexes[bind.dom[1]]
            t = resolve_text(kn) if kn.k in ('name', 'key', 'num') else None
            if kn.k == 'date':
                t = kn.v.isoformat()
            if t is None:
                raise LoadError('LOAD-KEY', 'a key of the open index %s is a key literal' % ix.id, loc)
            kv = ix.texts.get(t)
            if kv is None:
                kv = Key(ix.id, t)
                ix.texts[t] = kv
                ix.keys.append(kv)
                ix.added[_kv(kv)] = version
                ix.finish()
            else:
                old = ix.added.get(_kv(kv))
                if old is not None and known_first(version.known) < known_first(old.known):
                    ix.added[_kv(kv)] = version
            return kv
        return resolve_key_literal(m, kn, bind.dom, loc)

    # -- observations (L §5.19) ----------------------------------------------------------
    def obs_item(self, v, it, inf):
        m = self.m
        if it.given:
            raise LoadError('LOAD-NAME', '%s is not a def' % inf.id, it.loc)
        if it.keys is None or len(it.keys) != 1 or it.keys[0].k not in ('name', 'key', 'num'):
            raise LoadError('LOAD-KEY', 'an observation row states its id: %s[<id>]' % inf.id, it.loc)
        if it.governs is not None or it.instant is not None:
            raise LoadError('LOAD-PARSE', 'cannot parse: an observation row states no governs or instant', it.loc)
        key = resolve_text(it.keys[0])
        own_known = None
        if it.known is not None:
            own_known = (it.known.a, it.known.b) if it.known.k == 'window' else it.known.v
            if known_first(own_known) < known_first(v.known):
                # L §8.3 LOAD-KNOWN "row <x>[k] is known before its version <v>"
                raise LoadError('LOAD-KNOWN', 'row %s[%s] is known before its version %s' % (inf.id, key, v.id), it.loc)
            if known_last(own_known) > m.on:
                raise LoadError('LOAD-KNOWN', 'row %s[%s] is known after the model\'s on date' % (inf.id, key), it.loc)
        if it.value.k not in ('record', 'none'):
            raise LoadError('LOAD-FIELD', 'an observation is a record with named fields: %s[%s]' % (inf.id, key),
                            it.loc)
        o = ObsRow(inf, key, it.value, v, own_known, it.locator, it.replace, it.loc)
        if it.assumed is not None:
            o.assumed = it.assumed
        inf.obs = getattr(inf, 'obs', [])
        inf.obs.append(o)
        if not o.removed:
            self.obs_fields(o)
        return True

    def obs_fields(self, o):
        m = self.m
        loc = o.loc
        fields = {}
        legs = {}
        for f, val in o.node.fields:
            if f == 'leg':
                if val.name in legs:
                    raise LoadError('LOAD-LEG', 'leg %s twice in %s' % (val.name, o.sid), loc)
                legs[val.name] = val
                continue
            if f in fields:
                raise LoadError('LOAD-FIELD', 'field %s twice in %s' % (f, o.sid), loc)
            fields[f] = val
        if 'on' not in fields:
            raise LoadError('LOAD-FIELD', 'on is a field of an observation; %s has none' % o.sid, loc)
        if 'kind' not in fields:
            raise LoadError('LOAD-FIELD', 'kind is a field of an observation; %s has none' % o.sid, loc)
        dn = fields.pop('on')
        if dn.k in ('window', 'oneof') or (dn.k == 'call' and dn.name in ('window', 'one_of')):
            # L §5.19: "A window belongs in an attribute ..., never in on, the observation's or a leg's (LOAD-LEG)"
            raise LoadError('LOAD-LEG', 'the on of %s is a window; a window belongs in an attribute' % o.sid, loc)
        if dn.k != 'date':
            raise LoadError('LOAD-UNIT', 'the on of %s is a day' % o.sid, loc)
        o.day = dn.v
        kn = fields.pop('kind')
        if kn.k == 'name':
            hit = m.lookup(kn.id)
        elif kn.k == 'field' and kn.obj.k == 'name':
            hit = m.qualified(kn.obj.id, kn.name)
        else:
            hit = None
        if hit is None or hit[0] != 'kind':
            raise LoadError('LOAD-NAME', 'unknown event kind in %s' % o.sid, loc)
        k = m.kinds[hit[1].id]
        if k.spec != 'event':
            raise LoadError('LOAD-KIND', '%s is not an event kind (%s)' % (k.id, o.sid), loc)
        if k.id == 'adjustments' or 'adjustments' in k.ancestors(m):
            # L §5.14.3: "a move or an observed row of kind adjustments, or of a subkind of it, is refused (LOAD-KIND)"
            raise LoadError('LOAD-KIND', '%s of %s: only the engine makes adjustments' % (o.sid, k.id), loc)
        if k.certainty == 'uncertain':
            raise LoadError('LOAD-LEG', 'observation of uncertain kind %s (%s)' % (k.id, o.sid), loc)
        # LOAD-LEAF is for things entered with `+` (L §5.7); a movement may be of a kind with subkinds: the
        # examples of L §5.19 (`kind: transfers` beside `kind groceries: transfers`) and §5.14 rely on it
        o.kind = k.id
        fn = fields.pop('fulfils', None)
        if fn is not None:
            o.fulfils = self.obs_fulfils(fn, o)
        decls = m.kind_legs(k.id)
        names = [d.name for d in decls]
        for ln in legs:
            if ln not in names:
                raise LoadError('LOAD-LEG', 'leg %s is not a leg of %s (%s)' % (ln, k.id, o.sid), loc)
        for d in decls:
            ln = legs.get(d.name)
            if ln is None:
                raise LoadError('LOAD-LEG', '%s lacks leg %s' % (o.sid, d.name), loc)
            frm = self.place_ref(ln.frm, loc)
            to = self.place_ref(ln.to, loc)
            amt, unit = self.obs_amount(ln.amount, d, frm, to, o)
            lday = None
            if ln.on is not None:
                dn = ln.on
                if dn.k in ('window', 'oneof') or (dn.k == 'call' and dn.name in ('window', 'one_of')):
                    # L §5.19: a window is "never in on, the observation's or a leg's (LOAD-LEG)"
                    raise LoadError('LOAD-LEG', 'the on of leg %s of %s is a window; a window belongs in an '
                                                'attribute' % (d.name, o.sid), loc)
                if ln.on.k != 'date':
                    raise LoadError('LOAD-UNIT', 'the on of leg %s of %s is a day' % (d.name, o.sid), loc)
                lday = ln.on.v
            o.legs.append((d.name, frm, to, amt, unit, lday))
        attrs = m.kind_attrs(k.id)
        anames = [a for a, t in attrs]
        for f in fields:
            if f not in anames:
                raise LoadError('LOAD-FIELD', '%s is not a field of %s (%s)' % (f, k.id, o.sid), loc)
        for a, t in attrs:
            if a not in fields:
                raise LoadError('LOAD-LEG', 'attribute %s is missing or unknown (%s)' % (a, o.sid), loc)
            if t is None:
                raise AssertionError((a, k.id))   # only adjustments has untyped attributes, refused above
            o.attrs[a] = convert_value(m, fields[a], t, loc, Fraction(1), o.sid)

    def obs_amount(self, node, d, frm, to, o):
        """An observed leg's amount, in its leg's unit and its places' (LOAD-LEG otherwise, L §5.19)."""
        m = self.m
        if node.k != 'num' or node.unit is None:
            raise LoadError('LOAD-LEG', 'the amount of leg %s of %s states its unit' % (d.name, o.sid), o.loc)
        uu, f = U.from_syntax(node.unit)
        if len(uu) != 1 or uu[0][1] != 1:
            raise LoadError('LOAD-LEG', 'leg %s of %s: %s is not a unit of a leg' % (d.name, o.sid, node.text), o.loc)
        u = uu[0][0]
        ok = False
        if d.unit == 'any':
            ok = u in CURRENCIES or u in m.units or u in m.unit_kind_keys
        elif d.unit in CURRENCIES or d.unit in m.units:
            ok = (u == d.unit)
        else:
            kd = m.lookup(d.unit, m.kinds[d.kind].regime)
            ok = u in m.unit_kind_keys and kd is not None and m.unit_kind_keys[u] in m.kinds[kd[1].id].subtree(m)
        for pl in (frm, to):
            pu = m.place_unit(m.things[pl.name])
            if pu not in ('any', u):
                ok = False
        if not ok:
            raise LoadError('LOAD-LEG', 'leg %s of %s is of the wrong unit (%s)' % (d.name, o.sid, node.text), o.loc)
        q = node.v * f
        if u in CURRENCIES:
            if (q / CURRENCIES[u]).denominator != 1:
                raise LoadError('LOAD-PRECISION', '%s is finer than the minor unit' % node.text, o.loc)
            return Money(q, u), u
        return Num(q, ((u, 1),)), u

    def obs_fulfils(self, fn, o):
        m = self.m
        if fn.k == 'index' and fn.obj.k == 'name':
            name, keys = fn.obj.id, fn.keys
        elif fn.k == 'name':
            name, keys = fn.id, []
        else:
            # L §5.19: "A fulfils naming something that is neither a planned instance nor an adjustment (a place, an
            # input) is refused (LOAD-KIND)"
            raise LoadError('LOAD-KIND', 'fulfils on a %s (%s), not a movement' % (
                'field' if fn.k == 'field' else fn.k, o.sid), o.loc)
        hit = m.lookup(name)
        if hit is None:
            raise LoadError('LOAD-NAME', 'unknown id %s' % name, o.loc)
        if hit[0] == 'kind' and name == 'adjustments' and fn.k == 'index':
            # L §5.14.3 (round 12b): an adjustment's name is adjustments[m[keys].leg, V], "so an observed row settles
            # it with fulfils: adjustments[rent[2026-03].cash, v2]"
            return self.adj_ref(fn, o)
        if hit[0] != 'move':
            raise LoadError('LOAD-KIND', 'fulfils on a %s %s (%s), not a movement' % (hit[0], name, o.sid), o.loc)
        mi = hit[1]
        if len(keys) != len(mi.binds):
            raise LoadError('LOAD-KEY', 'wrong number of keys for %s' % mi.id, o.loc)
        kvs = self.move_keys(mi, keys, o.loc)
        return ('inst', mi.id, tuple(kvs))

    def adj_ref(self, fn, o):
        """`adjustments[m[keys].leg, V]` (L §5.14.3) -> ('adj', move id, keys, leg, version id)."""
        m = self.m
        if len(fn.keys) != 2:
            raise LoadError('LOAD-KEY', 'an adjustment is named adjustments[<move>[keys].<leg>, <version>] (%s)'
                            % o.sid, o.loc)
        lr, vr = fn.keys
        if lr.k != 'field' or vr.k != 'name':
            raise LoadError('LOAD-KEY', 'an adjustment is named adjustments[<move>[keys].<leg>, <version>] (%s)'
                            % o.sid, o.loc)
        ir = lr.obj
        if ir.k == 'index' and ir.obj.k == 'name':
            name, keys = ir.obj.id, ir.keys
        elif ir.k == 'name':
            name, keys = ir.id, []
        else:
            raise LoadError('LOAD-KIND', 'fulfils on a %s (%s), not a movement' % (ir.k, o.sid), o.loc)
        hit = m.lookup(name)
        if hit is None:
            raise LoadError('LOAD-NAME', 'unknown id %s' % name, o.loc)
        if hit[0] != 'move':
            raise LoadError('LOAD-KIND', 'fulfils on a %s %s (%s), not a movement' % (hit[0], name, o.sid), o.loc)
        mi = hit[1]
        if len(keys) != len(mi.binds):
            raise LoadError('LOAD-KEY', 'wrong number of keys for %s' % mi.id, o.loc)
        legs = [d.name for d in m.kind_legs(mi.kind)] if getattr(mi, 'kind', None) else []
        if legs and lr.name not in legs:
            raise LoadError('LOAD-LEG', 'leg %s is not a leg of %s' % (lr.name, mi.kind), o.loc)
        vh = m.names.get(vr.id)
        if vh is None or vh[0] != 'version':
            raise LoadError('LOAD-NAME', 'unknown version %s' % vr.id, o.loc)
        kvs = self.move_keys(mi, keys, o.loc)
        return ('adj', mi.id, tuple(kvs), lr.name, vr.id)

    def move_keys(self, mi, keys, loc):
        """The written keys of a move instance (a fulfils, a same side); a key at a stepped range's position is one of
        its values (L §2.5, checked at run)."""
        m = self.m
        kvs = []
        earlier = {}
        for kn, b in zip(keys, mi.binds):
            if b.dom[0] == 'range':
                kv = range_literal(m, kn, loc)
                register_range_lit(m, b, kv, kn, loc, earlier)
            else:
                kv = resolve_key_literal(m, kn, b.dom, loc)
            kvs.append(kv)
            earlier[b.var] = kv
        return kvs

    # -- same lines of rows and movements (L §6.1, §5.19 SF-25) ---------------------------------------------------
    def joint_sames(self):
        """`same p = q` naming two rows or two movements, in a joint model's header or in a version: two rows are
        read together (HOLE-SAME where they disagree, a joint model's own row settles them), two movements count once
        (the walk).  Things, places and kinds were joined before parsing (joint.py)."""
        from .joint import owner_of, shown
        m = self.m
        m.same_moves = []
        m.same_rows = {}
        m.same_settle = {}
        pairs = [(l, r, m.header.loc, None) for l, r in m.header.sames] + [(it.l, it.r, it.loc, v) for v, it in m.sames]
        for l, r, loc, v in pairs:
            a = self.sref(l, loc)
            b = self.sref(r, loc)
            # L §6.1 (round 14): "The two sides are of one sort: two kinds declared alike ..., two things of one base
            # kind ..., two rows of one type, or two movements; any other pair is refused (LOAD-SAME)"; "A same in a
            # version that names a kind or a thing is refused (LOAD-SAME): only the header joins them"
            what = {'thing': 'a thing', 'row': 'a row', 'move': 'a movement'}
            if v is not None and (a[0] == 'thing' or b[0] == 'thing'):
                raise LoadError('LOAD-SAME', 'same %s = %s in a version names a kind or a thing' % (
                    shown(_txt_node(l)), shown(_txt_node(r))), loc)
            if a[0] == 'other' or b[0] == 'other' or a[0] != b[0]:
                wa = what.get(a[0]) or ('a %s' % a[1])
                wb = what.get(b[0]) or ('a %s' % b[1])
                raise LoadError('LOAD-SAME', 'same %s = %s: %s and %s' % (shown(_txt_node(l)), shown(_txt_node(r)),
                                                                         wa, wb), loc)
            if a[0] == 'thing':
                if (id(l), id(r)) in getattr(self, 'place_pairs', ()):
                    continue          # a keyed place at its keys, joined before the rows were read
                # things, places and kinds named by id are joined before parsing (joint.py); a thing written with keys
                # that isn't a keyed place is no thing (L §6.1: "any other pair is refused (LOAD-SAME)")
                raise LoadError('LOAD-SAME', 'same %s = %s: a thing written with keys that is not a keyed place' % (
                    shown(_txt_node(l)), shown(_txt_node(r))), loc)
            if a[0] == 'row':
                ia, ka, ta = a[1], a[2], a[3]
                ib, kb, tb = b[1], b[2], b[3]
                if repr(ia.type) != repr(ib.type):
                    raise LoadError('LOAD-SAME', 'same %s = %s: rows of types %s and %s' % (
                        shown(ta), shown(tb), T.show(ia.type), T.show(ib.type)), loc)
                kka = (ia.id, tuple(_kv(k) for k in ka))
                kkb = (ib.id, tuple(_kv(k) for k in kb))
                m.same_rows[kka] = (ib, kb, ta, tb)
                m.same_rows[kkb] = (ia, ka, tb, ta)
                continue
            m.same_moves.append((a[1], b[1], loc))
        # a row of the joint model on a member's item states the value of two rows stated same ("the joint model
        # settles it with a version row of its own"); on any other member's item it would change an imported one
        if getattr(m, 'joint', None) is not None:
            for inf in list(m.inputs.values()):
                if owner_of(inf.id) == '' or inf.observed:
                    continue
                own = []
                for r in inf.rows:
                    if r.version is not None and owner_of(r.version.id) == '':
                        if r.version.scenario is not None:
                            # L §6.1 Writes into members: "A scenario over a joint model (§6.4) may replace the
                            # members' rows too, named with their prefixes"
                            continue
                        key = (inf.id, tuple(_kv(x[1]) for x in r.pattern if x[0] == 'k'))
                        if key not in m.same_rows or len(key[1]) != len(inf.binds):
                            # L §6.1 (round 14) The joint model's own rows: "of a member's item only at a pair stated
                            # same, which they settle; a row ... of any other member's item would change an imported
                            # one, and is refused (LOAD-SAME)"
                            raise LoadError('LOAD-SAME', 'the joint model states %s%s, an item of member %s that no '
                                                         'same line joins' % (shown(inf.id), r.keys_text(),
                                                                              owner_of(inf.id)), r.loc)
                        m.same_settle.setdefault(key, []).append(r)
                        own.append(r)
                if own:
                    inf.rows = [r for r in inf.rows if r not in own]

    def joint_place_sames(self):
        """`same p = q` in a joint model's header where a side is a keyed place at its keys (L §6.1: "each side names
        ... a thing (a keyed place at its keys included)"): the two are one place, before the rows and moves that name
        them are read.  Places named by their ids alone were joined before parsing (joint.py)."""
        from .joint import owner_deep, shown
        m = self.m
        J = getattr(m, 'joint', None)
        self.place_pairs = set()
        if J is None:
            return
        loc = m.header.loc
        for l, r in m.header.sames:
            if not (_has_keys(l) or _has_keys(r)):
                continue
            a = self.place_side(l, loc)
            b = self.place_side(r, loc)
            if a is None and b is None:
                continue
            if a is None or b is None:
                # L §6.1: "two things of one base kind ...; any other pair is refused (LOAD-SAME)"
                raise LoadError('LOAD-SAME', 'same %s = %s: a place and %s' % (
                    shown(_txt_node(l)), shown(_txt_node(r)), 'another item' if (b or a) is None else 'a thing'), loc)
            self.place_pairs.add((id(l), id(r)))
            ta, tb = a[0], b[0]
            if ta is tb:
                continue
            oa, ob = m.kind_outside(ta.base), m.kind_outside(tb.base)
            if not oa and not ob:
                if ta.base != tb.base:
                    raise LoadError('LOAD-SAME', 'same %s = %s: places of kinds %s and %s' % (
                        shown(_txt_node(l)), shown(_txt_node(r)), shown(ta.base), shown(tb.base)), loc)
                # L §6.1: "Two places of which neither is outside must also be of one unit and name one price input,
                # if any (LOAD-SAME)"
                if m.place_unit(ta) != m.place_unit(tb):
                    raise LoadError('LOAD-SAME', 'same %s = %s: places of two units' % (
                        shown(_txt_node(l)), shown(_txt_node(r))), loc)
                if repr(m.price_of(ta)) != repr(m.price_of(tb)):
                    raise LoadError('LOAD-SAME', 'same %s = %s: places of two price inputs' % (
                        shown(_txt_node(l)), shown(_txt_node(r))), loc)
            # the representative: a place named by its id (an entry's thing) keeps its name; an outside place
            # contributes only its legs (L §6.1, C-14), so an inside one represents the pair
            sides = [(ta, a[1], oa, _txt_node(l)), (tb, b[1], ob, _txt_node(r))]
            sides.sort(key=lambda s: (bool(s[1]), s[2]))
            rep, oth = sides[0], sides[1]
            if not oth[1]:
                raise NotYet('two places stated same that are both named by their ids at keys (%s, %s)' % (
                    shown(ta.name), shown(tb.name)), loc)
            for pi, kt in oth[1]:
                pi.things[kt] = rep[0]
            if not oth[2] and not rep[2]:
                # L §6.1 What same does: "A thing's entries are such facts: an entry only one side states stands, and
                # on a day on which the two sides' entries into one kind disagree, whatever depends on the thing being
                # alive there is a HOLE-SAME hole": the other side's lifespan (a keyed place's lives on its keys'
                # lifespans, under each cut) is kept beside the representative's
                ob = owner_deep(oth[0].name.split('[', 1)[0])
                if ob != owner_deep(rep[0].name.split('[', 1)[0]):
                    same_other(rep[0], ob, oth[0])
                else:
                    sa, sb = m.lifespan(ta, m.on), m.lifespan(tb, m.on)
                    if (sa.text() if sa else None) != (sb.text() if sb else None):
                        raise NotYet('two places of one member stated same whose lifespans differ (%s and %s)' % (
                            shown(ta.name), shown(tb.name)), loc)
            if m.things.get(oth[0].name) is oth[0]:
                del m.things[oth[0].name]
            grp = J.groups.setdefault(rep[0].name, [])
            for t, holders, out, txt in (rep, oth):
                o = owner_deep(t.name.split('[', 1)[0])
                if not any(g[1] == shown(txt) for g in grp):
                    grp.append((o, shown(txt), out))
                if not out:
                    J.owner_place[(o, rep[0].name)] = True

    def joint_place_sides(self):
        """Places stated same by their ids whose lifespans may differ (joint.plan_joins, J.place_others): the other
        side's entries, as that side states them, kept beside the representative's (L §6.1 What same does; read by
        model.Model.same_sides, alive_test and lifespan, and in the walk)."""
        m = self.m
        J = getattr(m, 'joint', None)
        if J is None or not J.place_others:
            return
        from .joint import owner_deep
        for name, others in J.place_others.items():
            th = m.things.get(name)
            if th is None:
                raise NotYet('a place stated same whose representative %s has no lifespan' % name)
            for o, kind, ents in others:
                t2 = ThingInfo(name)
                t2.base = th.base
                for lives, vid, replace, loc in ents:
                    e = Entry(th.base, lspan_to_span(lives, loc), m.versions[vid] if vid is not None else None,
                              replace, None, None, loc)
                    t2.entries.append(e)
                same_other(th, o, t2)

    def place_side(self, n, loc):
        """A same side naming a place: (its thing, [(place info, key tuple)] that hold it), or None."""
        m = self.m
        if n.k == 'name':
            name, keys = n.id, []
        elif n.k == 'index' and n.obj.k == 'name':
            name, keys = n.obj.id, n.keys
        else:
            return None
        hit = m.lookup(name)
        if hit is None:
            t = m.things.get(name)
            if t is not None and not keys and m.is_place_thing(t):
                return (t, [])
            return None
        kind, pi = hit
        if kind != 'place':
            return None
        if not pi.binds:
            if keys:
                raise LoadError('LOAD-KEY', 'wrong number of keys for %s' % pi.id, loc)
            return (pi.things[()], [(pi, ())])
        if not keys:
            # a keyed place is a thing only at its keys (L §6.1 "a thing (a keyed place at its keys included)")
            from .joint import shown
            raise LoadError('LOAD-SAME', 'same names %s, a keyed place, without its keys' % shown(pi.id), loc)
        if len(keys) != len(pi.binds):
            raise LoadError('LOAD-KEY', 'wrong number of keys for %s' % pi.id, loc)
        kvs = [resolve_key_literal(m, kn, b.dom, loc) for kn, b in zip(keys, pi.binds)]
        kt = tuple(_kv(k) for k in kvs)
        t = pi.things.get(kt)
        if t is None:
            # L §6.1: "A side that names an instance that doesn't exist is refused, as a fulfils that names one is
            # (RUN-RECORD)"
            from .joint import shown
            from .errors import TallyError
            raise TallyError('RUN-RECORD', 'same names %s[%s], which doesn\'t exist' % (
                shown(pi.id), ', '.join(key_text(k) for k in kvs)), loc)
        return (t, [(pi, kt)])

    def sref(self, n, loc):
        """A side of a same line: ('move', id) | ('row', info, keys, text) | ('thing', name)."""
        m = self.m
        if n.k == 'name':
            name, keys = n.id, []
        elif n.k == 'index' and n.obj.k == 'name':
            name, keys = n.obj.id, n.keys
        else:
            # L §6.1 (round 14): any pair not of one sort is refused (LOAD-SAME)
            return ('other', 'field path', _txt_node(n))
        hit = m.lookup(name)
        if hit is None:
            if name in m.things:
                return ('thing', name)
            raise LoadError('LOAD-NAME', 'unknown id %s' % name, loc)
        kind, info = hit
        if kind == 'move':
            if len(keys) != len(info.binds):
                raise LoadError('LOAD-KEY', 'wrong number of keys for %s' % info.id, loc)
            kvs = self.move_keys(info, keys, loc)
            iid = '%s[%s]' % (info.id, ', '.join(key_text(k) for k in kvs)) if kvs else info.id
            return ('move', iid)
        if kind == 'input' and info.observed:
            if len(keys) != 1 or keys[0].k not in ('name', 'key', 'num'):
                raise LoadError('LOAD-KEY', 'an observation row is named by its id: %s[<id>]' % info.id, loc)
            return ('move', '%s[%s]' % (info.id, resolve_text(keys[0])))
        if kind == 'input':
            if len(keys) != len(info.binds):
                raise LoadError('LOAD-KEY', 'wrong number of keys for %s' % info.id, loc)
            kvs = [resolve_key_literal(m, kn, b.dom, loc) for kn, b in zip(keys, info.binds)]
            text = '%s%s' % (info.id, ('[%s]' % ', '.join(key_text(k) for k in kvs)) if kvs else '')
            return ('row', info, kvs, text)
        if kind in ('place', 'kind'):
            return ('thing', name)
        return ('other', kind, name)

    def row_item(self, v, it):
        m = self.m
        allowed = ('def',) if it.given else ('input',)
        kind, inf = self.target(it.qual, it.target, it.loc, allowed)
        of = getattr(it, 'of', None)
        if of is not None and not (kind == 'input' and inf.id in ('stated', 'opening') and getattr(inf, 'builtin', False)):
            # L §2.3 row (tally 2.1): "of: only on a row of opening or stated, §5.19, LOAD-PARTS otherwise"
            raise LoadError('LOAD-PARTS', 'of on a row of %s' % inf.id, it.loc)
        if kind == 'input' and inf.observed:
            return self.obs_item(v, it, inf)
        binds = inf.binds
        pattern = []
        nstated = 0
        if it.keys is not None:
            if len(it.keys) > len(binds):
                raise LoadError('LOAD-KEY', 'wrong number of keys for %s' % inf.id, it.loc)
            for kn, b in zip(it.keys, binds):
                if kn.k == 'star':
                    pattern.append(('*',))
                elif kn.k == 'krange':
                    if b.dom[0] == 'kind' or (b.dom[0] == 'index' and m.indexes[b.dom[1]].form == 'open'):
                        # L §4.6 step 1: a range matches keys in index order, "on any index but an open one, which
                        # has no order (§5.6)"; things have none either (§5.7); L §8.4: at load, LOAD-UNIT
                        raise LoadError('LOAD-UNIT', 'a key range over %s %s: it has no order' % (
                            'the kind' if b.dom[0] == 'kind' else 'the open index', b.dom[1]), it.loc)
                    a = self.pattern_key(kn.a, b, it.loc)
                    c = self.pattern_key(kn.b, b, it.loc)
                    if key_order(m, a, b.dom) > key_order(m, c, b.dom):
                        raise LoadError('LOAD-KEY', 'range %s..%s is out of order' % (key_text(a), key_text(c)), it.loc)
                    pattern.append(('r', a, c))
                    nstated += 1
                elif b.dom[0] == 'range':
                    # L §2.5: a written key at a stepped range's position is one of its values (LOAD-KEY)
                    kv = range_literal(m, kn, it.loc)
                    earlier = {bb.var: p[1] for bb, p in zip(binds, pattern) if p[0] == 'k'}
                    register_range_lit(m, b, kv, kn, it.loc, earlier)
                    pattern.append(('k', kv))
                    nstated += 1
                else:
                    pattern.append(('k', self.pattern_key(kn, b, it.loc, v)))
                    nstated += 1
        while len(pattern) < len(binds):
            pattern.append(('*',))
        own_span = None
        if it.governs is not None:
            own_span = lspan_to_span(it.governs, it.loc)
        own_known = None
        if it.known is not None:
            if it.known.k == 'window':
                own_known = (it.known.a, it.known.b)
            else:
                own_known = it.known.v
            ktxt = '%s%s' % (inf.id, pattern_text(pattern))
            if known_first(own_known) < known_first(v.known):
                # L §8.3 LOAD-KNOWN "row <x>[k] is known before its version <v>"
                raise LoadError('LOAD-KNOWN', 'row %s is known before its version %s' % (ktxt, v.id), it.loc)
            if known_last(own_known) > m.on:
                # L §4.5 (round 12): "A version or row known after on, or whose known window ends after it, is
                # refused (LOAD-KNOWN)"
                raise LoadError('LOAD-KNOWN', 'row %s is known after the model\'s on date' % ktxt, it.loc)
        instant = None
        reading = (kind == 'input' and inf.reading)
        balance = (kind == 'input' and inf.id in ('stated', 'opening'))
        if it.instant is not None:
            ins = it.instant
            if not (reading or balance):
                # L §4.6 Readings: "an instant on a row of an input that isn't a reading, is refused, LOAD-INSTANT"
                raise LoadError('LOAD-INSTANT', '%s is not a reading; its rows take no instant' % inf.id, it.loc)
            if ins.after is not None:
                instant = ('after', ins.after)
            else:
                if balance and ins.pos is None:
                    raise LoadError('LOAD-INSTANT', 'balance reading %s at %s needs start, end or after' % (
                        inf.id, ins.day), it.loc)
                instant = (ins.day, ins.pos)
            if own_span is not None:
                raise LoadError('LOAD-PARSE', 'cannot parse: a reading states an instant, not governs', it.loc)
        elif reading or balance:
            # L §4.6 Readings: "a row of a reading without an instant ... is refused, LOAD-INSTANT"
            raise LoadError('LOAD-INSTANT', 'row %s of a reading has no instant' % inf.id, it.loc)
        row = Row(inf, tuple(pattern), nstated, it.value, it.assumed, v, own_span, own_known, instant, it.locator,
                  it.given, it.replace, it.loc, it)
        if kind == 'input':
            if of is not None:
                # a reading's stated parts (L §5.19, tally 2.1): kept beside the readings, never a total
                row.of = of
                inf.__dict__.setdefault('part_rows', []).append(row)
            else:
                inf.rows.append(row)
        else:
            inf.givens.append(row)
        return own_span is not None or instant is not None

    # -- checks --------------------------------------------------------------------
    def check_rows(self):
        m = self.m
        for inf in list(m.inputs.values()) + list(m.defs.values()):
            rows = inf.rows if isinstance(inf, InputInfo) else inf.givens
            if not rows:
                continue
            tyv = inf.type
            for r in rows:
                if isinstance(inf, InputInfo) and inf.observed:
                    continue
                if isinstance(inf, InputInfo) and inf.builtin and inf.id in ('opening', 'stated'):
                    r.val = self.place_value(inf, r)
                    continue
                if r.value.k == 'rowexpr':
                    # a scenario row that is an expression over the base's facts (L §6.4): typed by the checker,
                    # evaluated at the row's keys and law date
                    if getattr(r.version, 'scenario', None) is None:
                        # L §2.3 row: "expr: only in a scenario's version, §6.4, LOAD-SCENARIO otherwise"; the message
                        # is L §8.3's (round 9)
                        raise LoadError('LOAD-SCENARIO', 'an expression row outside a scenario', r.loc)
                    r.val = ('sexpr', r.value.e)
                    continue
                if tyv is not None and tyv.k == 'dist':
                    # a dist row is an expression; a stated hole stays a hole row (L §5.16, §8.1)
                    r.val = ('hole', r.value.text) if r.value.k == 'hole' else ('node', r.value)
                    continue
                r.val = convert_value(m, r.value, tyv, r.loc, getattr(inf, 'unit_factor', Fraction(1)),
                                      inf.id, binds=inf.binds, pattern=r.pattern)
            self.check_overlap(inf, rows)
            if isinstance(inf, InputInfo) and inf.builtin and inf.id in ('opening', 'stated'):
                self.check_part_rows(inf)
            # stable ids (L §2.5)
            per = {}
            for r in rows:
                per.setdefault((r.version.id, r.pattern), []).append(r)
            for r in rows:
                sid = '%s:%s%s' % (r.version.id, inf.id, r.keys_text())
                if len(per[(r.version.id, r.pattern)]) > 1:
                    if r.own_span is not None:
                        sid += ' ' + r.own_span.text()
                    elif r.instant is not None:
                        sid += ' %s' % (r.instant[0],)
                r.sid = sid
        self.check_mv_refs()

    def check_mv_refs(self):
        """An observed row's attribute that names a movement (a key of an event kind, L §3.1, §5.7) names an observed
        row of that kind, or an instance of a move of it (LOAD-KEY otherwise, as for a key not of its kind)."""
        m = self.m
        obs = {}
        for inf in m.inputs.values():
            if getattr(inf, 'observed', False):
                for o in getattr(inf, 'obs', ()):
                    obs[o.sid] = o
        for o in obs.values():
            if not o.attrs:
                continue
            types = None
            for a, v in o.attrs.items():
                if not (isinstance(v, tuple) and v and v[0] == 'mvref'):
                    continue
                if types is None:
                    types = dict(m.kind_attrs(o.kind))
                kid = types[a].ix
                name = v[1].split('[', 1)[0]
                tk = m.moves[name].kind if name in m.moves else (obs[v[1]].kind if v[1] in obs else None)
                if tk is None or tk not in m.kinds[kid].subtree(m):
                    raise LoadError('LOAD-KEY', '%s is not a key of %s' % (v[1], kid), o.loc)

    def check_part_rows(self, inf):
        """A reading's stated parts (L §5.19, tally 2.1): "They stand beside the reading's total, in the same version
        and at the same instant (LOAD-PARTS otherwise), and sum to at most it (LOAD-PARTS where load sees both,
        RUN-RECORD otherwise)". Each total row gets its parts: [(q node, value, row)]."""
        from .moves import instant_text
        from .parts import thing_parts, labels_of
        m = self.m
        for r in inf.__dict__.get('part_rows', ()):
            r.val = self.place_value(inf, r)
            place = r.pattern[0][1].name
            th = m.things[place]
            labs = labels_of(thing_parts(m, th)) if thing_parts(m, th) is not None else set()
            q = self.of_row_q(r.of, r.loc)
            r.ofq = q
            if q is not None:
                need = {'owner': {'owner'}, 'batch': {'batch'}, 'pair': {'owner', 'batch'}}[q[0]]
                if not need <= labs:
                    qt = '(%s, %s)' % (q[1], q[2]) if q[0] == 'pair' else q[1]
                    raise LoadError('LOAD-PARTS', 'of %s at %s, which keeps no labels of its sort' % (qt, place),
                                    r.loc)
            it = instant_text(r)
            tot = [t for t in inf.rows if t.version is r.version and t.pattern[0][1].name == place and t.instant is not None
                   and instant_text(t) == it and t.val is not NONE]
            if not tot:
                raise LoadError('LOAD-PARTS', 'the parts of %s at %s have no total in %s' % (place, it, r.version.id),
                                r.loc)
            t = tot[-1]
            qt = None if q is None else ('(%s, %s)' % (q[1], q[2]) if q[0] == 'pair' else q[1])
            if isinstance(r.val, Num) and r.val.q < 0:
                # L §5.19 (2.1-F6): stated parts "are each at least 0 ..., LOAD-PARTS otherwise"
                raise LoadError('LOAD-PARTS', 'the stated part of %s at %s is below 0' % (
                    qt if qt is not None else _txt_node(r.of), place), r.loc)
            if q is not None:
                for q2, v2, r2 in t.__dict__.get('parts', ()):
                    o2 = getattr(r2, 'ofq', None)
                    if o2 is not None and _parts_overlap(o2, q):
                        # L §5.19 (2.1-F6): "no two of them overlap (a party and a batch at a place that keeps pairs,
                        # or a party or a batch and a pair that holds it), ... LOAD-PARTS otherwise"
                        q2t = '(%s, %s)' % (o2[1], o2[2]) if o2[0] == 'pair' else o2[1]
                        raise LoadError('LOAD-PARTS', 'the stated parts %s and %s of %s overlap' % (q2t, qt, place),
                                        r.loc)
            t.__dict__.setdefault('parts', []).append((r.of, r.val, r))
        for t in inf.rows:
            parts = t.__dict__.get('parts')
            if not parts or not isinstance(t.val, Num) or not all(isinstance(v, Num) for q, v, r in parts):
                continue
            s_ = sum((v.q for q, v, r in parts), Fraction(0))
            if s_ > t.val.q:
                from .values import fmt
                from .moves import qty
                raise LoadError('LOAD-PARTS', 'the stated parts of %s sum to %s, over its total %s' % (
                    t.pattern[0][1].name, fmt(qty(s_, t.val.u[0][0])), fmt(t.val)), parts[-1][2].loc)

    def of_row_q(self, n, loc):
        """q of a stated part where load sees it: ('owner', party) | ('batch', movement id) | ('pair', ...), or None
        (the walk resolves it)."""
        m = self.m

        def one(x):
            try:
                hit = self.sref(x, loc)
            except LoadError:
                return None
            if hit[0] == 'thing' and hit[1] in m.things:
                th = m.things[hit[1]]
                sub = m.kinds['parties'].subtree(m)
                if th.base in sub or any(e.kind in sub for e in getattr(th, 'entries', ())):
                    return ('owner', hit[1])
                return None
            if hit[0] == 'move':
                return ('batch', hit[1])
            return None
        if n.k == 'ofpair':
            a, b = one(n.a), one(n.b)
            if a is None or b is None:
                return None
            return ('pair', a[1], b[1])
        return one(n)

    def place_value(self, inf, r):
        """A balance reading's value: a quantity in its place's unit; a place of unit any states one row per unit
        (L §4.6 Readings)."""
        m = self.m
        if not r.pattern or r.pattern[0][0] != 'k':
            raise LoadError('LOAD-KEY', 'a row of %s states its place' % inf.id, r.loc)
        th = m.things[r.pattern[0][1].name]
        pu = m.place_unit(th)
        node = r.value
        if node.k == 'hole':
            return ('hole', node.text)
        if node.k == 'none':
            return NONE
        if node.k != 'num':
            raise LoadError('LOAD-UNIT', 'a value of the wrong type for %s[%s]' % (inf.id, th.name), r.loc)
        if pu == 'any' or pu is None:
            if node.unit is None:
                raise LoadError('LOAD-UNIT', '%s[%s] holds any unit; its row states the unit' % (inf.id, th.name),
                                r.loc)
            uu, f = U.from_syntax(node.unit)
            if len(uu) != 1 or uu[0][1] != 1:
                raise LoadError('LOAD-UNIT', '%s is not a unit of a place' % node.text, r.loc)
            pu = uu[0][0]
        if pu in CURRENCIES:
            ty = T.money(pu)
        else:
            ty = T.num(((pu, 1),))
        return convert_value(m, node, ty, r.loc, Fraction(1), inf.id)

    def check_obs(self):
        """Observation rows: one id is one movement, restated only with replace (L §5.19, §4.6); a covered place's
        observations lie inside the span its version covers (LOAD-COVERS)."""
        m = self.m
        for qid, inf in m.inputs.items():
            if not inf.observed:
                continue
            by = {}
            for o in getattr(inf, 'obs', []):
                by.setdefault(o.key, []).append(o)
            for key, os_ in by.items():
                # a scenario's restatement stands above its base's rows whatever their known dates (L §4.6 step 2,
                # §6.4): ordered by layer, then known
                os_.sort(key=lambda o: (row_layer(m, o), known_first(o.known)))
                for i, o in enumerate(os_):
                    if i == 0:
                        if o.replace:
                            raise LoadError('LOAD-REPLACE', 'replace %s: no earlier row to replace' % o.sid, o.loc)
                        continue
                    if known_first(o.known) == known_first(os_[i - 1].known) and \
                            row_layer(m, o) == row_layer(m, os_[i - 1]):
                        raise LoadError('LOAD-OVERLAP', 'rows %s and %s of %s overlap at [%s]' % (
                            os_[i - 1].loc, o.loc, qid, key), o.loc)
                    if not o.replace:
                        raise LoadError('LOAD-OVERLAP', 'rows %s and %s of %s overlap at [%s], neither replacing the '
                                        'other' % (os_[i - 1].loc, o.loc, qid, key), o.loc)
            for o in getattr(inf, 'obs', []):
                if o.removed:
                    continue
                v = o.version
                for th, sp in getattr(v, 'covers', []):
                    span = sp if sp is not None else v.span_at(m.on)
                    for ln in o.legs:
                        if th.name in (ln[1].name, ln[2].name):
                            d = ln[5] or o.day
                            if span is None or not span.contains(d):
                                raise LoadError('LOAD-COVERS', 'observation %s of %s is dated outside the span %s covers'
                                                % (o.sid, th.name, v.id), o.loc)

    def check_overlap(self, inf, rows):
        m = self.m
        K = m.on

        def sp(r):
            if r.instant is not None:
                return None
            s = r.span(K)
            return s

        def ov(a, b):
            if a.instant is not None or b.instant is not None:
                return a.instant == b.instant
            sa, sb = sp(a), sp(b)
            if sa is None or sb is None:
                return False
            return sa.overlaps(sb)
        groups = {}
        nones = []
        for r in rows:
            gk = pattern_norm(r.pattern)
            if isinstance(inf, InputInfo) and inf.builtin and isinstance(getattr(r, 'val', None), Num):
                gk = gk + (r.val.u,)
            elif isinstance(inf, InputInfo) and inf.builtin and getattr(r, 'val', None) is NONE:
                nones.append((gk, r))
                continue
            groups.setdefault(gk, []).append(r)
        for gk, r in nones:
            # a balance reading of none states no balance (L §5.19); it is of the unit of the readings it may replace
            units = [g for g in groups if g[:-1] == gk and len(g) == len(gk) + 1]
            groups.setdefault(units[0] if len(units) == 1 else gk, []).append(r)
        jo = getattr(m, 'joint', None) is not None
        if jo:
            from .joint import owner_of

        def owners_differ(a, b):
            if not jo or a.version is None or b.version is None:
                return False
            if a.version.scenario is not None or b.version.scenario is not None:
                # a scenario over a joint model replaces a member's row as any scenario row does (L §6.1, §6.4)
                return False
            return owner_of(a.version.id) != owner_of(b.version.id)
        for pat, rs in groups.items():
            for i in range(len(rs)):
                for j in range(i + 1, len(rs)):
                    a, b = rs[i], rs[j]
                    if not ov(a, b):
                        continue
                    if owners_differ(a, b):
                        # facts about one thing from two members (or the joint model) are read together (L §6.1)
                        continue
                    la, lb = row_layer(m, a), row_layer(m, b)
                    if la != lb:
                        # L §4.6 step 2, §6.4 (round 12b): "a row that overlaps a base row at the same keys says
                        # replace (LOAD-OVERLAP), and replaces it whatever the two rows' known dates"
                        hi = a if la > lb else b
                        if hi.replace:
                            continue
                        raise LoadError('LOAD-OVERLAP', 'rows %s and %s of %s overlap at %s, neither replacing the '
                                                        'other' % (a.loc, b.loc, inf.id, a.keys_text() or 'no keys'),
                                        hi.loc)
                    ka, kb = known_first(a.known), known_first(b.known)
                    if ka == kb and a.instant is None and b.instant is None and \
                            sp(a).always != sp(b).always and row_layer(m, a) == row_layer(m, b):
                        # two same-day rows that precedence orders, a span over always, load (L §6.3 item 2,
                        # round 13; L §4.6 step 3.3)
                        continue
                    if ka == kb:
                        # neither is known before the other, so neither replaces the other: on overlapping spans a tie
                        # (L §4.6 steps 2 and 4, LOAD-OVERLAP; round 18 C1-11: no LOAD-REPLACE for two same-day
                        # replace lines, which load on disjoint spans)
                        if a.version is b.version:
                            raise LoadError('LOAD-OVERLAP', 'rows %s and %s of %s overlap at %s in one version' % (
                                a.loc, b.loc, inf.id, a.keys_text() or 'no keys'), b.loc)
                        raise LoadError('LOAD-OVERLAP', 'rows %s and %s of %s overlap at %s, neither replacing the '
                                                        'other' % (a.loc, b.loc, inf.id, a.keys_text() or 'no keys'), b.loc)
                    later = a if ka > kb else b
                    if not later.replace:
                        raise LoadError('LOAD-OVERLAP', 'rows %s and %s of %s overlap at %s, neither replacing the '
                                                        'other' % (a.loc, b.loc, inf.id, a.keys_text() or 'no keys'),
                                        later.loc)
            for r in rs:
                # in a scenario, a base row at those keys is an earlier row whatever its known (L §4.6 step 2, §8.3
                # LOAD-REPLACE "in a scenario, no base row at those keys either")
                if r.replace and not any((known_first(o.known) < known_first(r.known) or
                                          row_layer(m, o) < row_layer(m, r)) and ov(o, r) and not owners_differ(o, r)
                                         for o in rs if o is not r and row_layer(m, o) <= row_layer(m, r)):
                    raise LoadError('LOAD-REPLACE', 'replace %s%s: no earlier row to replace' % (inf.id, r.keys_text()),
                                    r.loc)
        # ties between rows of different patterns (L §4.6 step 4)
        if isinstance(inf, InputInfo) and inf.builtin:
            return
        pats = list(groups.items())
        for i in range(len(pats)):
            for j in range(i + 1, len(pats)):
                pa, ra = pats[i]
                pb, rb = pats[j]
                # the narrower row wins (L §4.6 step 3.2, round 11b): `[2026]` beats `[2025..2029]`, but
                # `[2025..2027]` and `[2026..2029]` tie
                if narrower(m, inf, ra[0].pattern, rb[0].pattern) or narrower(m, inf, rb[0].pattern, ra[0].pattern):
                    continue
                if not patterns_meet(m, inf, ra[0].pattern, rb[0].pattern):
                    continue
                for a in ra:
                    for b in rb:
                        if owners_differ(a, b):
                            continue
                        if row_layer(m, a) != row_layer(m, b):
                            # inside a scenario the higher layer's row wins (L §4.6 step 3.1)
                            continue
                        if a.instant is not None or b.instant is not None:
                            if a.instant == b.instant:
                                raise LoadError('LOAD-OVERLAP', 'rows %s and %s of %s overlap and tie' % (
                                    a.loc, b.loc, inf.id), b.loc)
                            continue
                        sa, sb = sp(a), sp(b)
                        if sa.always != sb.always:
                            continue
                        if sa.overlaps(sb):
                            raise LoadError('LOAD-OVERLAP', 'rows %s and %s of %s overlap at %s and %s and tie' % (
                                a.loc, b.loc, inf.id, a.keys_text(), b.keys_text()), b.loc)

    def check_bands(self):
        m = self.m
        for bid, b in m.bands.items():
            for t in b.tables:
                self.check_table(b, t)
            # tables of one key: a later table overlapping an earlier one replaces rows only
            byk = {}
            for t in b.tables:
                byk.setdefault(_kv(t.key) if t.key is not None else None, []).append(t)
            for k, ts in byk.items():
                for t in ts:
                    earlier = [o for o in ts if o is not t and known_first(o.version.known) < known_first(t.version.known)
                               and _span_ov(o.version, t.version, m.on)]
                    same = [o for o in ts if o is not t and known_first(o.version.known) == known_first(t.version.known)
                            and _span_ov(o.version, t.version, m.on)]
                    if same:
                        raise LoadError('LOAD-OVERLAP', 'band %s: tables of %s and %s overlap, known the same day' % (
                            bid, t.version.id, same[0].version.id), t.loc)
                    if any(r.replace for r in t.rows):
                        # L §5.9 Rows replaced: every replace row's interval one of the earlier table's (its values
                        # replaced), or the replace rows form a whole table (which replaces the earlier wholly); a
                        # version mixing replace and plain rows, or whose replace rows do neither, is LOAD-BAND
                        if not all(r.replace for r in t.rows):
                            raise LoadError('LOAD-BAND', 'band %s in %s mixes replace rows and plain rows' % (
                                bid, t.version.id), t.loc)
                        if not earlier:
                            raise LoadError('LOAD-REPLACE', 'replace %s: no earlier row to replace' % bid, t.loc)
                        try:
                            self.check_table(b, t, whole=True)
                            t.whole_ok = True
                        except LoadError:
                            t.whole_ok = False
                        matches = all(any(any(r.same_interval(o) for o in e.rows) for e in earlier) for r in t.rows)
                        if not matches and not t.whole_ok:
                            raise LoadError('LOAD-BAND', 'band %s in %s: the replace rows neither match the earlier '
                                            'table\'s intervals nor form a whole table' % (bid, t.version.id), t.loc)
                        if not t.whole_ok:
                            # L §5.9 Rows replaced: "On a day of its span where no table is in force, only a whole
                            # table stands ...; a patch there is refused (LOAD-REPLACE) under every cut that sees the
                            # version" (writer 8, sixth pass: refcore refused it at a read of such a day, and refused
                            # a whole table there too)
                            sp = t.version.span_at(m.on)
                            gap = _uncovered(sp, [e.version.span_at(m.on) for e in earlier]) if sp is not None \
                                else None
                            if gap is not None:
                                raise LoadError('LOAD-REPLACE', 'replace rows of %s in %s patch no table on %s' % (
                                    bid, t.version.id, gap.text()), t.loc)
                    elif earlier:
                        raise LoadError('LOAD-OVERLAP', 'band %s: the table of %s overlaps the table of %s without '
                                                        'replace' % (bid, t.version.id, earlier[0].version.id), t.loc)

    def check_table(self, b, t, whole=False):
        if t.replace and not whole:
            return
        isint = b.argtype.k == 'int'
        rows = t.rows
        for r in rows:
            if r.lo is not None and r.hi is not None:
                if r.lo > r.hi or (r.lo == r.hi and not (r.lo_closed and r.hi_closed)):
                    raise LoadError('LOAD-BAND', 'band %s in %s: rows unordered, overlapping or with a gap at %s' % (
                        b.id, t.version.id, r.lo), r.loc)
        for p, n in zip(rows, rows[1:]):
            if isint and isinstance(p.hi, Fraction) and isinstance(n.lo, Fraction):
                ph = p.hi if p.hi_closed else p.hi - 1
                nl = n.lo if n.lo_closed else n.lo + 1
                if ph.denominator != 1 or nl.denominator != 1:
                    raise LoadError('LOAD-BAND', 'band %s in %s: an int band row with a fractional end' % (
                        b.id, t.version.id), n.loc)
        bad = band_defect(rows, isint)
        if bad is not None:
            x, r = bad
            raise LoadError('LOAD-BAND', 'band %s in %s: rows unordered, overlapping or with a gap at %s' % (
                b.id, t.version.id, 'inf' if x is None else ('-inf' if x == '-inf' else _fmt_pt(x))), r.loc)


def _fmt_pt(x):
    if isinstance(x, Fraction) and x.denominator == 1:
        return str(x.numerator)
    if isinstance(x, BandKey):
        o = x.orig
        return getattr(o, 'name', None) or str(o)
    return str(x)


def band_defect(rows, isint):
    """The first defect of a band table, as (x, the row it shows at), or None. L §8.3 LOAD-BAND (round 9): "x is the
    lower end of the first row, in the table's order, that doesn't start where the row before it ends: an overlap, a
    gap or a row out of order". Rows meet "closed against open" (L §5.9); `int` arguments are checked over the
    integers, so [0, 4] and [5, 10) meet. x is the row's lower end as written (`-inf` for a row from -inf)."""
    for p, n in zip(rows, rows[1:]):
        if p.hi is None or n.lo is None:
            return ('-inf' if n.lo is None else n.lo, n)
        if isint:
            ph = p.hi if p.hi_closed else p.hi - 1
            nl = n.lo if n.lo_closed else n.lo + 1
            meets = nl == ph + 1
        else:
            meets = n.lo == p.hi and p.hi_closed != n.lo_closed
        if not meets:
            return (n.lo, n)
    return None
    for (iv1, r1), (iv2, r2) in zip(ivs, ivs[1:]):
        if lo_key(iv2) < lo_key(iv1):
            return (iv2[0] if iv2[0] is not None else '-inf', r2)
    return None

def _uncovered(span, others):
    """The first stretch of span that no span of others covers, as a Span, or None."""
    lo_d, hi_d = datetime.date.min, datetime.date.max
    lo = lo_d if span.always or span.start is None else span.start
    hi = hi_d if span.always or span.end is None else span.end
    ivs = sorted((lo_d if o.always or o.start is None else o.start, hi_d if o.always or o.end is None else o.end)
                 for o in others if o is not None)
    cur = lo
    for a, b in ivs:
        if cur >= hi:
            return None
        if a > cur:
            end = min(a, hi)
            return Span(None if cur == lo_d else cur, None if end == hi_d else end)
        cur = max(cur, b)
    if cur < hi:
        return Span(None if cur == lo_d else cur, None if hi == hi_d else hi)
    return None


def _span_ov(v1, v2, K):
    a = v1.span_at(K)
    b = v2.span_at(K)
    if a is None or b is None:
        return False
    return a.overlaps(b)


def pattern_norm(p):
    return tuple(_pk(x) for x in p)


def _pk(x):
    if x[0] == '*':
        return ('*',)
    if x[0] == 'r':
        return ('r', _kv(x[1]), _kv(x[2]))
    return ('k', _kv(x[1]))


def _pos_rel(m, dom, a, b):
    """At one key position: 'eq' when a and b state the key alike (the same literal, the same range or `*`), 'in' when
    a lies strictly inside b (a literal inside a range that holds it, a range inside a wider one, anything inside `*`),
    else None (L §4.6 steps 2 and 3.2, round 11b)."""
    if _pk(a) == _pk(b):
        return 'eq'
    if b[0] == '*':
        return 'in'
    if a[0] == '*' or b[0] == 'k':
        return None
    lo_a, hi_a = (a[1], a[1]) if a[0] == 'k' else (a[1], a[2])
    try:
        la, ha = key_order(m, lo_a, dom), key_order(m, hi_a, dom)
        lb, hb = key_order(m, b[1], dom), key_order(m, b[2], dom)
    except (TypeError, ValueError, KeyError, AttributeError):
        return None
    if None in (la, ha, lb, hb):
        return None
    return 'in' if lb <= la and ha <= hb else None


def narrower(m, inf, pa, pb):
    """L §4.6 step 3.2 (round 11b): "the narrower row: at every key position its key is the other's or lies inside
    it ..., and at one position strictly inside"."""
    strict = False
    for i, (a, b) in enumerate(zip(pa, pb)):
        r = _pos_rel(m, inf.binds[i].dom, a, b)
        if r is None:
            return False
        if r == 'in':
            strict = True
    return strict


def row_layer(m, r):
    """A row's layer in a stack of scenarios: 0 for the base, the depth of its scenario otherwise (L §4.6 step 3.1)."""
    return version_layer(m, r.version)


def version_layer(m, v):
    """A version's layer in a stack of scenarios: 0 for the base, the depth of its scenario otherwise."""
    sc = getattr(v, 'scenario', None) if v is not None else None
    n = 0
    over = getattr(m, 'scenario_over', {}) or {}
    while sc is not None:
        n += 1
        sc = over.get(sc)
    return n


def patterns_meet(m, inf, pa, pb):
    """Could one read match both patterns?  A key matches a key equal to it, a range the keys from its
    first to its last in index order, `*` every key (L §4.6 step 1)."""
    for i, (a, b) in enumerate(zip(pa, pb)):
        if a[0] == '*' or b[0] == '*':
            continue
        dom = inf.binds[i].dom
        if a[0] == 'k' and b[0] == 'k':
            if _kv(a[1]) != _kv(b[1]):
                return False
            continue
        lo_a, hi_a = (a[1], a[1]) if a[0] == 'k' else (a[1], a[2])
        lo_b, hi_b = (b[1], b[1]) if b[0] == 'k' else (b[1], b[2])
        try:
            la, ha = key_order(m, lo_a, dom), key_order(m, hi_a, dom)
            lb, hb = key_order(m, lo_b, dom), key_order(m, hi_b, dom)
        except (TypeError, ValueError, KeyError):
            continue
        if la is None or ha is None or lb is None or hb is None:
            continue
        if ha < lb or hb < la:
            return False
    return True


def key_order(m, k, dom):
    if dom[0] == 'index':
        ix = m.indexes[dom[1]]
        return ix.pos(k)
    if dom[0] == 'axis':
        return m.axes[dom[1]].options.index(k.name)
    if dom[0] == 'kind':
        t = m.things[k.name]
        return m.kind_order_key(t, dom[1], m.on)
    if dom[0] == 'list':
        return [resolve_text(x) for x in dom[1]].index(key_text(k))
    return k


def resolve_text(n):
    if isinstance(n, str):
        return n
    if n.k == 'name':
        return n.id
    if n.k == 'key':
        return n.v
    return n.get('text') or str(n.get('v'))


def _has_keys(n):
    return n.k == 'index' or (n.k == 'field' and _has_keys(n.obj))


def same_other(th, owner, other):
    """Keep `other`, a side stated same as the place th whose lifespan may differ from th's, under its owner (L §6.1):
    th's own version-less entries (its place section's) are marked as its owner's (model.entry_owner)."""
    from .joint import owner_deep
    if not hasattr(th, 'same_others'):
        th.same_others = {}
        own = owner_deep(th.name.split('[', 1)[0])
        for e in th.entries:
            if e.version is None:
                e.own = own
    if owner in th.same_others or owner == owner_deep(th.name.split('[', 1)[0]):
        raise NotYet('a place stated same with another place of the same member (%s)' % th.name)
    th.same_others[owner] = other


def resolve_key_literal(m, kn, dom, loc):
    """A key literal at a position whose domain is dom -> key value (LOAD-KEY if none)."""
    kind = dom[0]
    if kind == 'index':
        ix = m.indexes[dom[1]]
        if ix.isint:
            if kn.k == 'num' and kn.isint and kn.unit is None:
                v = int(kn.v)
                if ix.has(v):
                    return v
            raise LoadError('LOAD-KEY', '%s is not a key of %s' % (kn.get('text') or resolve_text(kn), ix.id), loc)
        if ix.pk is not None:
            label = None
            if kn.k == 'num' and kn.isint and kn.unit is None and len(kn.text) == 4:
                label = kn.text
            elif kn.k == 'month':
                label = '%04d-%02d' % kn.v
            elif kn.k == 'quarter':
                label = '%04d-Q%d' % kn.v
            elif kn.k == 'date':
                label = kn.v.isoformat()
            if label is not None:
                for k in ix.keys:
                    lab = k.isoformat() if isinstance(k, datetime.date) else k.label()
                    if lab == label and _pk_matches(k, kn):
                        return k
            raise LoadError('LOAD-KEY', '%s is not a key of %s' % (kn.get('text') or resolve_text(kn), ix.id), loc)
        t = resolve_text(kn) if kn.k in ('name', 'key', 'num', 'mmdd', 'cur') else None
        if kn.k == 'date':
            t = kn.v.isoformat()
        if kn.k == 'month':
            t = '%04d-%02d' % kn.v
        if kn.k == 'quarter':
            t = '%04d-Q%d' % kn.v
        if t is not None and t in ix.texts:
            return ix.texts[t]
        if ix.form == 'open':
            # its keys are those its inputs' rows bring (L §4.1), all gathered before any read is resolved
            raise LoadError('LOAD-KEY', '%s is not a key of %s: no row brings it' % (t or kn.k, ix.id), loc)
        raise LoadError('LOAD-KEY', '%s is not a key of %s' % (t or kn.k, ix.id), loc)
    if kind == 'kind':
        if kn.k == 'placekey' or (kn.k == 'index' and kn.obj.k == 'name'):
            name = kn.id if kn.k == 'placekey' else kn.obj.id
            hit = m.lookup(name)
            if hit is not None and hit[0] == 'place' and hit[1].binds:
                pi = hit[1]
                if len(kn.keys) != len(pi.binds):
                    raise LoadError('LOAD-KEY', 'wrong number of keys for place %s' % pi.id, loc)
                kvs = tuple(_kv(resolve_key_literal(m, x, b.dom, loc)) for x, b in zip(kn.keys, pi.binds))
                t = pi.things.get(kvs)
                if t is not None and m.thing_in(t.name, dom[1]):
                    return Thing(t.name)
                raise LoadError('LOAD-KEY', '%s[...] is not a key of %s' % (name, dom[1]), loc)
        t = resolve_text(kn) if kn.k in ('name', 'key', 'num', 'cur') else None
        if t is not None and kn.k == 'name':
            hit = m.lookup(t)
            if hit is not None and hit[0] == 'place' and not hit[1].binds:
                t = hit[1].things[()].name
        if t is not None and m.thing_in(t, dom[1]):
            return Thing(t)
        raise LoadError('LOAD-KEY', '%s is not a key of %s' % (t or kn.k, dom[1]), loc)
    if kind == 'axis':
        t = resolve_text(kn) if kn.k in ('name', 'key', 'num') else None
        if t is not None and t in m.axes[dom[1]].options:
            return Key(dom[1], t)
        raise LoadError('LOAD-KEY', '%s is not a key of %s' % (t or kn.k, dom[1]), loc)
    if kind == 'list':
        t = resolve_text(kn)
        if t in [resolve_text(x) for x in dom[1]]:
            if isinstance(kn, str):
                return int(t) if t.lstrip('-').isdigit() else Key('[list]', t)
            if kn.k == 'num' and kn.isint:
                return int(kn.v)
            return Key('[list]', t)
        raise LoadError('LOAD-KEY', '%s is not a key of the list' % t, loc)
    if kind == 'range':
        return range_literal(m, kn, loc)
    raise LoadError('LOAD-KEY', '%s is not a key' % (kn.get('text') or kn.k), loc)


def range_literal(m, kn, loc):
    """A key literal at a position keyed by a stepped range: "a value of the range's type, written or computed, and
    must be one of the range's values, a + i·s <= b ... (LOAD-KEY for a literal, RUN-KEY at run)" (L §2.5, round 12).
    The value as written; its membership is checked at run (the ends may be rows), `register_range_lit`."""
    if kn.k == 'date':
        return kn.v
    if kn.k == 'month':
        return month_period(*kn.v)
    if kn.k == 'quarter':
        return quarter_period(*kn.v)
    if kn.k == 'num':
        if kn.unit:
            terms = [(b, e) for b, e in kn.unit]
            for b, e in terms:
                if not (b == '%' or b in CURRENCIES or b in m.units or b in ('year', 'quarter', 'month', 'day')
                        or b in getattr(m, 'unit_kind_keys', ())):
                    raise LoadError('LOAD-UNIT', 'an unknown unit %s' % b, loc)
            u, f = U.from_syntax(terms)
            q = Fraction(kn.v) * f
            if U.is_currency(u):
                return Num(q, u, money=True)
            return Num(q, u)
        if kn.isint:
            return Int(int(kn.v))
        return Num(Fraction(kn.v), ())
    raise LoadError('LOAD-KEY', '%s is not a value of a stepped range' % (kn.get('text') or resolve_text(kn)), loc)


def known_last(k):
    """The last day a known date or window can be."""
    return k[1] if isinstance(k, tuple) else k


def pattern_text(pattern):
    if not pattern or all(p[0] == '*' for p in pattern):
        return ''
    parts = []
    for p in pattern:
        if p[0] == '*':
            parts.append('*')
        elif p[0] == 'r':
            parts.append('%s..%s' % (key_text(p[1]), key_text(p[2])))
        else:
            parts.append(key_text(p[1]))
    return '[%s]' % ', '.join(parts)


def register_range_lit(m, bind, value, kn, loc, earlier=None):
    """Keep a written key at a stepped-range position for its membership check at run (L §2.5: LOAD-KEY for a
    literal). earlier: the keys at the positions before it ({var: key}), which the range's ends may read."""
    kn._rangelit = True
    m.__dict__.setdefault('range_lits', []).append((bind, value, kn, loc, dict(earlier or {})))


def _pk_matches(k, kn):
    if kn.k == 'date':
        return isinstance(k, datetime.date) or (isinstance(k, Period) and k.kind == 'day')
    if kn.k == 'month':
        return isinstance(k, Period) and k.kind == 'month'
    if kn.k == 'quarter':
        return isinstance(k, Period) and k.kind == 'quarter'
    return isinstance(k, Period) and k.kind == 'year'


class BandKey:
    """A band end, or argument, that is a key or a period: ordered by its index (L §4.2 Keys compare in index order;
    a period by its start)."""
    __slots__ = ('rank', 'orig')

    def __init__(self, rank, orig):
        self.rank = rank
        self.orig = orig

    def __lt__(self, o):
        return self.rank < o.rank

    def __gt__(self, o):
        return self.rank > o.rank

    def __le__(self, o):
        return self.rank <= o.rank

    def __ge__(self, o):
        return self.rank >= o.rank

    def __eq__(self, o):
        return isinstance(o, BandKey) and self.rank == o.rank

    def __hash__(self):
        return hash(self.rank)


def order_key(v, m=None):
    """A comparable key of a band end (and of a band's argument)."""
    if isinstance(v, Num):
        return v.q
    if isinstance(v, datetime.date):
        return v
    if isinstance(v, Period):
        return BandKey((v.start, v.end), v)
    if isinstance(v, Key) and m is not None:
        if v.index in m.indexes:
            p = m.indexes[v.index].pos(v)
            if p is not None:
                return BandKey((p,), v)
        if v.index in m.axes and v.name in m.axes[v.index].options:
            return BandKey((m.axes[v.index].options.index(v.name),), v)
        raise LoadError('LOAD-KEY', '%s is not a key of %s' % (v.name, v.index))
    if isinstance(v, (int, Key, Thing)):
        # L §5.9 (round 12): "The argument is a number, a date, a period or a key of an index, never a thing
        # (LOAD-UNIT): things order by entry, which the cut changes"
        raise LoadError('LOAD-UNIT', 'a band over things (%s)' % getattr(v, 'name', v))
    # a band is "a function of one argument over intervals" (L §5.9): its argument is ordered; text, bool and
    # records are ordered by nothing (L §8.4 What isn't defined is refused)
    raise LoadError('LOAD-UNIT', 'a band over values that are not numbers or dates')


def convert_value(m, node, ty, loc, factor, target, binds=None, pattern=None, allow_frac=True):
    """A row's literal value as a runtime value of the target's declared type (L §5.8 Values).
    Holes and none are kept as markers; the read builds the hole with its keys."""
    k = node.k
    if k == 'hole':
        return ('hole', node.text)
    if k == 'none':
        return NONE
    if k == 'never':
        if ty is not None and ty.k not in ('date',):
            raise LoadError('LOAD-UNIT', 'never is a date; %s is %s' % (target, T.show(ty)), loc)
        return NEVER
    if ty is None:
        raise NotYet('the value of an observed row', loc)
    tk = ty.k
    if k == 'num':
        if node.get('frac') and terminating(node.v):
            raise LoadError('LOAD-PRECISION', '%s terminates; write it as a decimal' % node.text, loc)
        uu, f = U.from_syntax(node.unit) if node.unit else (None, Fraction(1))
        if tk == 'money':
            if uu is None:
                q = node.v * factor
            else:
                if uu != ty.u:
                    raise LoadError('LOAD-UNIT', '%s and %s have different units' % (node.text, T.show(ty)), loc)
                q = node.v * f
            minor = CURRENCIES[ty.cur]
            if (q / minor).denominator != 1:
                raise LoadError('LOAD-PRECISION', '%s is finer than the minor unit' % node.text, loc)
            return Money(q, ty.cur)
        if tk == 'num':
            tu = ty.u
            if binds is not None and pattern is not None and any(b.startswith('$') for b, e in tu):
                env = {}
                for bi, p in zip(binds, pattern):
                    if p[0] == 'k':
                        env[bi.var] = key_text(p[1])
                tu2 = U.subst(tu, env)
            else:
                tu2 = tu
            if uu is None:
                return Num(node.v * factor, tu)
            c = U.conv(uu, tu2)
            if c is None:
                if any(b.startswith('$') for b, e in tu2):
                    # L §3.2 (round 12): "A row's value has its type's unit at every key it states, so a row that
                    # leaves such a key unstated or * gives one unit for keys of several and is refused (LOAD-UNIT),
                    # unless its value is a bare literal, which takes each key's unit"
                    raise LoadError('LOAD-UNIT', '%s gives one unit for keys of several units (%s)' % (
                        node.text, T.show(ty)), loc)
                raise LoadError('LOAD-UNIT', '%s and %s have different units' % (node.text, T.show(ty)), loc)
            return Num(node.v * f * c, tu)
        if tk == 'int':
            if uu is not None or not node.isint or node.get('frac'):
                raise LoadError('LOAD-UNIT', '%s is not an int' % node.text, loc)
            return Int(int(node.v))
        if tk == 'rate':
            if uu is None or U.period_of(uu) is None:
                raise LoadError('LOAD-UNIT', 'a rate states its period: %s' % node.text, loc)
            return Num(node.v * f, uu, rate=True)
        if tk == 'period' and ty.pk == 'year' and node.isint and uu is None and len(node.text) == 4:
            return year_period(int(node.v))
        if tk == 'key':
            return key_value(m, node, ty, loc)
        if tk == 'dist':
            return ('node', node)
        raise LoadError('LOAD-UNIT', '%s and %s have different units (a value of the wrong type)' % (
            node.text, T.show(ty)), loc)
    if k == 'bool':
        if tk != 'bool':
            raise LoadError('LOAD-UNIT', 'a value of the wrong type: %s for %s' % (node.v, T.show(ty)), loc)
        return node.v
    if k == 'str':
        if tk != 'text':
            raise LoadError('LOAD-UNIT', 'a value of the wrong type: text for %s' % T.show(ty), loc)
        return Text(node.v)
    if k == 'date':
        if tk == 'date':
            return node.v
        if tk == 'key':
            return key_value(m, node, ty, loc)
        raise LoadError('LOAD-UNIT', 'a value of the wrong type: a date for %s' % T.show(ty), loc)
    if k == 'month':
        if tk == 'period' and ty.pk == 'month':
            return month_period(*node.v)
        if tk == 'key':
            return key_value(m, node, ty, loc)
        raise LoadError('LOAD-UNIT', 'a value of the wrong type: a month for %s' % T.show(ty), loc)
    if k == 'quarter':
        if tk == 'period' and ty.pk == 'quarter':
            return quarter_period(*node.v)
        if tk == 'key':
            return key_value(m, node, ty, loc)
        raise LoadError('LOAD-UNIT', 'a value of the wrong type: a quarter for %s' % T.show(ty), loc)
    if k == 'mmdd':
        # L §3.1 monthday, §4.2 (GD-2, round 19): a row may hold a month-day (`fam1_file_by = 04-30`)
        from .values import MonthDay, monthday_ok
        if tk == 'mmdd':
            if not monthday_ok(*node.v):
                raise LoadError('LOAD-PARSE', 'cannot parse: %s is not a month-day every year has' % node.text, loc)
            return MonthDay(*node.v)
        if tk == 'key':
            return key_value(m, node, ty, loc)
        raise LoadError('LOAD-UNIT', 'a value of the wrong type: a month-day for %s' % T.show(ty), loc)
    if k in ('window', 'oneof'):
        if tk != 'date':
            raise LoadError('LOAD-UNIT', 'a window is a date; %s is %s' % (target, T.show(ty)), loc)
        if k == 'window':
            if node.a > node.b:
                raise LoadError('LOAD-DOMAIN', 'window(a, b) with a after b (L §8.4)', loc)
            return Iv(node.a, node.b, None, 'window(%s, %s)' % (node.a, node.b))
        ds = frozenset(node.days)
        return Iv(min(ds), max(ds), ds, 'one_of([%s])' % ', '.join(d.isoformat() for d in sorted(ds)))
    if k == 'list':
        if tk != 'dates':
            raise LoadError('LOAD-UNIT', 'a list is of type dates; %s is %s' % (target, T.show(ty)), loc)
        out = []
        for it in node.items:
            if it.k != 'date':
                raise LoadError('LOAD-UNIT', 'dates holds days', loc)
            out.append(it.v)
        return Lst(out)
    if k == 'record':
        if tk != 'record':
            raise LoadError('LOAD-UNIT', 'a record for %s' % T.show(ty), loc)
        vals = {}
        for f, fv in node.fields:
            if f == 'leg':
                raise LoadError('LOAD-FIELD', 'a leg is not a field of %s' % T.show(ty), loc)
            ft = ty.field(f)
            if ft is None:
                raise LoadError('LOAD-FIELD', '%s is not a field of %s' % (f, T.show(ty)), loc)
            if f in vals:
                raise LoadError('LOAD-FIELD', 'field %s twice' % f, loc)
            vals[f] = convert_value(m, fv, ft, loc, Fraction(1), target)
        missing = [f for f, _ in ty.fields if f not in vals]
        if missing:
            # L §3.1 (round 12): "A literal states every field of its type (LOAD-FIELD), so a field not known yet is a
            # hole and one that doesn't apply none"; L §8.3 "<row> lacks field <f>"
            raise LoadError('LOAD-FIELD', '%s lacks field %s' % (target, missing[0]), loc)
        return Rec([(f, vals[f]) for f, _ in ty.fields])
    if k == 'calval':
        if tk != 'calendar':
            raise LoadError('LOAD-UNIT', 'a calendar for %s' % T.show(ty), loc)
        from .dates import WEEKDAYS
        for w in node.weekends:
            if w not in WEEKDAYS:
                # L §4.2: "the weekdays mon tue wed thu fri sat sun ... another word is refused, LOAD-PARSE"
                raise LoadError('LOAD-PARSE', '%s is not a weekday (mon tue wed thu fri sat sun)' % w, loc)
        added = []
        for a in node.added:
            if a.k == 'date':
                added.append(('day', a.v))
            elif a.k == 'hole':
                added.append(('hole', a.text))
            elif a.k == 'window':
                added.append(('window', a.a, a.b))
            else:
                added.append(('oneof', frozenset(a.days)))
        return Cal(tuple(node.weekends), tuple(node.yearly), node.yfrom, tuple(added))
    if tk == 'key' and ty.ix in m.kinds and ty.ix in m.kinds['events'].subtree(m) and k in ('index', 'name'):
        # a key of an event kind is a movement (L §3.1 `key K`, "a thing of kind K"; L §5.7; L §2.3 field: a ref): an
        # observed row `kb[k03]` or a move's instance, kept by its id and read as that movement (writer 11, the
        # migrator's lot_ref)
        mid = movement_ref(m, node, loc)
        if mid is not None:
            return ('mvref', mid)
    if k in ('name', 'key', 'cur', 'mmdd'):
        if tk == 'key':
            return key_value(m, node, ty, loc)
        if tk == 'code':
            word = resolve_text(node)
            if word not in T.CODE_WORDS[ty.code] or word == 'nominal':
                raise LoadError('LOAD-UNIT', '%s is not a %s code' % (word, ty.code), loc)
            return Code(ty.code, word)
        raise LoadError('LOAD-UNIT', 'a value of the wrong type: %s for %s' % (resolve_text(node), T.show(ty)), loc)
    if k == 'code':
        if tk == 'code' and ty.code == 'compounding':
            return Code('compounding', 'nominal', node.v[1])
        raise LoadError('LOAD-UNIT', 'nominal %d is a compounding code' % node.v[1], loc)
    if k == 'to' or k == 'dist':
        if tk == 'dist':
            return ('node', node)
        raise LoadError('LOAD-UNIT', 'a distribution for %s' % T.show(ty), loc)
    raise LoadError('LOAD-UNIT', 'a value of the wrong type for %s' % target, loc)


def movement_ref(m, node, loc):
    """The id of the movement a row's ref names (an observed row by its id, a move's instance by its keys), or None
    when it names no movement."""
    if node.k == 'name':
        name, keys = node.id, []
    elif node.k == 'index' and node.obj.k == 'name':
        name, keys = node.obj.id, node.keys
    else:
        return None
    hit = m.lookup(name)
    if hit is None:
        return None
    kind, info = hit
    if kind == 'input' and getattr(info, 'observed', False):
        if len(keys) != 1 or keys[0].k not in ('name', 'key', 'num'):
            raise LoadError('LOAD-KEY', 'an observation row is named by its id: %s[<id>]' % info.id, loc)
        return '%s[%s]' % (info.id, resolve_text(keys[0]))
    if kind == 'move':
        if len(keys) != len(info.binds):
            raise LoadError('LOAD-KEY', 'wrong number of keys for %s' % info.id, loc)
        kvs = [resolve_key_literal(m, kn, b.dom, loc) for kn, b in zip(keys, info.binds)]
        return '%s[%s]' % (info.id, ', '.join(key_text(k) for k in kvs)) if kvs else info.id
    return None


def key_value(m, node, ty, loc):
    ix = ty.ix
    if ix in m.indexes:
        dom = ('index', ix)
    elif ix in m.kinds:
        dom = ('kind', ix)
    elif ix in m.axes:
        dom = ('axis', ix)
    else:
        raise LoadError('LOAD-NAME', 'unknown id %s' % ix, loc)
    k = resolve_key_literal(m, node, dom, loc)
    if isinstance(k, int) and not isinstance(k, bool):
        # a key of an integer index is also an int (L §4.2 Keys as values)
        return Int(k)
    return k
