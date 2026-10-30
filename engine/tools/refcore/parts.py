"""Writer 11: tally 2.1's parts (L §3.6, §5.7, §5.10, §5.13, §5.14, §5.14.1-3, §5.19, §8.1; E §4.2 Parts in the walk).

A place whose kind opts in (`owned by D`, `batched`) keeps, per (place, unit), its parts: a map from a label (a party,
a batch, or the pair) to a whole number of the kind's step, beside its balance. The checker half (`chk_*`, installed
on check.Checker) checks the parts clauses at load (LOAD-PARTS); the walk half lives in `partswalk.py`.
"""
from fractions import Fraction

from . import ty as T
from .ty import Ty
from .errors import LoadError, NotYet
from .node import walk as _nwalk

PART_TY = T.record([('owner', T.key('parties')), ('batch', T.key('events'))])
CLAUSE_ORDER = ('owned', 'batched', 'step', 'takes')
CLAUSE_TEXT = {'owned': 'owned by', 'batched': 'batched', 'step': 'step', 'takes': 'takes'}


def install(C):
    for name, fn in list(globals().items()):
        if name.startswith('chk_'):
            setattr(C, name[4:], fn)


def parts_kind(m, kid):
    """The kind whose parts clauses a kind keeps (itself or the nearest parent that states them), or None."""
    x = m.kinds.get(kid)
    while x is not None:
        if x.decl is not None and getattr(x.decl, 'pclauses', None):
            return x
        x = m.kinds[x.parent] if x.parent is not None else None
    return None


def thing_parts(m, th):
    """The parts kind of a place thing, or None (a place that keeps no parts books as on 2.0)."""
    if th is None or th.base is None:
        return None
    return parts_kind(m, th.base)


def labels_of(pk):
    """The sorts of label a parts kind keeps: a subset of {'owner', 'batch'}."""
    out = set()
    if pk.decl.owned is not None:
        out.add('owner')
    if pk.decl.batched:
        out.add('batch')
    return out


def _lp(msg, loc):
    return LoadError('LOAD-PARTS', msg, loc)


def chk_check_parts(self):
    """L §5.7 What a kind can be, the parts clauses (tally 2.1): "body lines of a place kind: owned by D [ties by e],
    batched, step u and takes k for w ..., in that order. A kind that states owned by or batched states step and takes,
    and step or takes stands only beside one of them. Its sign rule is min 0, and it is no subkind of assets ... D is a
    def of type num keyed by an event kind and by parties (either a subkind), in that order, at the law date m.day of
    its movement key m. A subkind of a kind that keeps parts keeps its clauses and states none of its own ... Anything
    else is refused (LOAD-PARTS)." L §8.3 LOAD-PARTS's messages."""
    m = self.m
    self.parts_reads = set()
    m.parts_kinds = {}
    for kid in sorted(m.kinds):
        k = m.kinds[kid]
        dk = k.decl
        if dk is None or not getattr(dk, 'pclauses', None):
            continue
        loc = dk.pclauses[0][1]
        if k.spec != 'place':
            raise _lp('kind %s keeps parts and is not a place kind' % kid, loc)
        x = m.kinds[k.parent] if k.parent is not None else None
        while x is not None:
            if x.decl is not None and getattr(x.decl, 'pclauses', None):
                raise _lp('%s states parts clauses under %s, which keeps parts' % (kid, x.id), loc)
            x = m.kinds[x.parent] if x.parent is not None else None
        names = [c for c, _ in dk.pclauses]
        for i, (c, cl) in enumerate(dk.pclauses):
            if c in names[:i]:
                raise _lp('kind %s: %s stated twice' % (kid, CLAUSE_TEXT[c]), cl)
            if i and CLAUSE_ORDER.index(c) < CLAUSE_ORDER.index(names[i - 1]):
                raise _lp('kind %s: %s after %s (the parts clauses go owned by, batched, step, takes)' % (
                    kid, CLAUSE_TEXT[c], CLAUSE_TEXT[names[i - 1]]), cl)
        if 'owned' in names or 'batched' in names:
            for c in ('step', 'takes'):
                if c not in names:
                    raise _lp('kind %s: %s missing' % (kid, CLAUSE_TEXT[c]), loc)
        else:
            raise _lp('kind %s: owned by or batched missing' % kid, loc)
        sign = m.kind_sign(kid)
        if sign != 'min0':
            raise _lp('kind %s keeps parts and is %s' % (kid, {'signed': 'signed', 'max0': 'max 0'}.get(sign, sign)),
                      loc)
        if kid in m.kinds['assets'].subtree(m):
            raise _lp('kind %s keeps parts and is an asset kind' % kid, loc)
        m.parts_kinds[kid] = k
        if dk.owned is not None:
            self.check_arrival_rule(k, dk.owned)
        self.check_takes(k, dk.takes)
    # LOAD-PARTS `owns[<p>, <x>]: <x> keeps parts` (L §3.6: "owns[p, x] at such a place is refused")
    inf = m.inputs.get('owns')
    if inf is not None and m.parts_kinds:
        for r in getattr(inf, 'rows', ()) or ():
            pat = getattr(r, 'pattern', None) or ()
            names = [getattr(kv[1], 'name', None) if kv and kv[0] == 'k' else '*' for kv in pat]
            for nm in names:
                th = m.things.get(nm) if isinstance(nm, str) else None
                if th is not None and thing_parts(m, th) is not None:
                    raise _lp('owns[%s]: %s keeps parts' % (', '.join(str(x) for x in names), nm),
                              getattr(r, 'loc', None))


def _strip_scope(self, sc):
    """Take back the edges a parts clause's terms recorded on their pseudo-item, keeping what they read (WARN-UNREAD:
    a def an arrival rule or a takes clause reads is read)."""
    es = self.edges.pop(sc.item, {})
    for t in es:
        self.parts_reads.add(t)
    self.marks.pop(sc.item, None)


def chk_check_arrival_rule(self, k, owned):
    """L §3.6 The arrival rule: "D is a def keyed by a movement and a party, def D[m in events, p in parties]: num at
    m.day (a subkind at either key allowed)"; "Its law date is the event's: it states at m.day (LOAD-PARTS
    otherwise)". The owned clause's ties by "reads D's two bound variables, the movement and the party, by the names
    D's binds give them"."""
    m = self.m
    did = owned.d
    msg = 'arrival rule %s is not a num keyed by a movement and a party at its movement\'s day' % did
    hit = m.lookup(did)
    if hit is None:
        raise LoadError('LOAD-NAME', 'unknown id %s' % did, owned.loc)
    if hit[0] != 'def':
        raise _lp(msg, owned.loc)
    d = m.defs[did]
    ok = len(d.binds) == 2
    if ok:
        b0, b1 = d.binds
        ok = (b0.dom[0] == 'kind' and b0.dom[1] in m.kinds and m.kinds[b0.dom[1]].spec == 'event' and
              b1.dom[0] == 'kind' and b1.dom[1] in m.kinds['parties'].subtree(m))
    if ok:
        ok = d.type is not None and d.type.k == 'num' and not d.type.u
    if ok:
        at = d.decl.at
        e = at.e if at is not None and at.k == 'law_expr' else None
        while e is not None and e.k == 'paren':
            e = e.e
        ok = (e is not None and e.k == 'field' and e.name == 'day' and e.obj.k == 'name' and
              e.obj.id == d.binds[0].var)
    if not ok:
        raise _lp(msg, owned.loc)
    self.parts_reads.add(('def', did))
    if owned.tie is not None:
        sc = self.def_scope(d)
        sc.item = ('parts', k.id)
        for e in owned.tie:
            self.tc(e, sc)
        _strip_scope(self, sc)


def chk_check_takes(self, k, tk):
    """L §3.6 The leaving rule: "takes k for w [v V] order e1, e2, ... [prorata [ties by e]] ... k is a part, read
    through its built-in fields k.owner ... and k.batch ...; w is the outflow, a movement whose kind and fields the terms
    read after a kind test"; "v V names the version whose law states the rule"."""
    m = self.m
    check = _c()
    if tk.part == tk.w:
        raise LoadError('LOAD-DUP', '%s is declared twice (takes %s for %s)' % (tk.w, tk.part, tk.w), tk.loc)
    if tk.v is not None and tk.v not in m.versions:
        raise LoadError('LOAD-NAME', 'takes of %s: v %s names no version' % (k.id, tk.v), tk.loc)
    sc = check.Scope(self, ('parts', k.id), None, 'def')
    for v in (tk.part, tk.w):
        if m.lookup(v) is not None:
            raise LoadError('LOAD-SHADOW', '%s shadows a top-level id' % v, tk.loc)
    sc.vars[tk.part] = check.Var(tk.part, PART_TY)
    sc.vars[tk.w] = check.Var(tk.w, T.key('events'))
    sc.keyvars = [tk.part, tk.w]
    for e in tk.order:
        # a term reads the outflow's fields after a kind test (L §5.7: an if's condition narrows)
        self.tc(e, sc)
    if tk.tie is not None:
        sct = check.Scope(self, ('parts', k.id), None, 'def')
        sct.vars[tk.part] = check.Var(tk.part, PART_TY)
        for e in tk.tie:
            self.tc(e, sct)
        _strip_scope(self, sct)
    _strip_scope(self, sc)
    if k.decl.step is not None:
        sst = check.Scope(self, ('parts', k.id), None, 'decl')
        sst.allow_lit = True
        self.tc(k.decl.step, sst)
        _strip_scope(self, sst)


def _c():
    from . import check
    return check


def chk_parts_walk_bound(self):
    """An arrival rule is run by the walk when it books an arrival (L §5.14.1 "The arrival rule is a reader too"): it
    and the defs it reaches that reach a running read are walk-bound (E §3.2), so they read the walk being built."""
    m = self.m
    if not getattr(m, 'parts_kinds', None):
        return
    E = self.edges
    roots = set(('def', k.decl.owned.d) for k in m.parts_kinds.values() if k.decl.owned is not None)
    seen = set()
    stack = list(roots)
    while stack:
        x = stack.pop()
        if x in seen:
            continue
        seen.add(x)
        for t in E.get(x, {}):
            if t[0] in ('def', 'fill'):
                stack.append(t)
    reached = getattr(self, 'reached_marks', None)
    for x in seen:
        if x[0] == 'def' and (reached is None or reached(x)):
            m.walk_bound.add(x[1])
    m.arrival_items = seen


def _day_offset(e, var):
    """0 for `var.day`, n for `var.day + n` (an int or a duration in days, as written), None otherwise: a day load
    sees relative to the arrival's (L §5.14.1, §8.3 item 9)."""
    while e.k == 'paren':
        e = e.e
    if e.k == 'field' and e.name == 'day' and e.obj.k == 'name' and e.obj.id == var:
        return 0
    if e.k == 'bin' and e.op in '+-':
        a = _day_offset(e.l, var)
        r = e.r
        if a is None or r.k != 'num' or not r.isint:
            return None
        n = int(r.v)
        return a + n if e.op == '+' else a - n
    return None


def chk_check_arrival_reads(self):
    """L §5.14.1 The arrival rule is a reader too: "The start of the arrival's day is booked before anything that day,
    so reading it is no cycle; a read of the end of the arrival's day or later is (LOAD-CYCLE where load sees the day,
    RUN-CYCLE otherwise)"; L §3.6: "a read at the end of the arrival's day or later is a cycle with the booking
    (LOAD-CYCLE)". Load sees the day in the rule itself, written from its movement key's day."""
    from .node import walk as nwalk
    m = self.m
    for k in sorted(getattr(m, 'parts_kinds', {}).values(), key=lambda k: k.id):
        if k.decl.owned is None:
            continue
        d = m.defs[k.decl.owned.d]
        var = d.binds[0].var
        for lab in d.labels:
            for rv in d.rules[lab]:
                for part in (rv.node.value, rv.node.when):
                    if part is None:
                        continue
                    for x in nwalk(part):
                        if x.k == 'bal' and x.args and getattr(x, '_form', None) == 'day':
                            off = _day_offset(x.args[-1], var)
                            if off is not None and (off > 0 or (off == 0 and not x.start)):
                                raise LoadError('LOAD-CYCLE', 'arrival rule %s reads bal(%s, %s%s): %s' % (
                                    d.id, _txt(x.ref), _txt(x.args[-1]), ' start' if x.start else '',
                                    'the end of its arrival\'s day' if off == 0 else 'a day after its arrival\'s'),
                                    rv.node.loc)
                        if x.k == 'value':
                            off = _day_offset(x.d, var)
                            if off is not None and off >= 0:
                                raise LoadError('LOAD-CYCLE', 'arrival rule %s reads value(%s, %s): %s' % (
                                    d.id, _txt(x.ref), _txt(x.d), 'the end of its arrival\'s day' if off == 0 else
                                    'a day after its arrival\'s'), rv.node.loc)


def _txt(n):
    from .check import _txt as t
    return t(n)
