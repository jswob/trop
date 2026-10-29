"""Writer 3's checks (E §3.1 steps 3, 5-8): places, moves, movements and their fields, balances, flows,
projections, observations; running reads (LOAD-READS), the strata (LOAD-ORDER), LOAD-COVERED.

Installed as Checker methods (`m_<name>` -> `<name>`), as check2 is.
"""
from fractions import Fraction

from . import units as U
from . import ty as T
from .ty import Ty
import datetime
from .errors import LoadError, Unspecified, NotYet, TallyError
from .model import PlaceThing, key_text
from .node import Node, walk
from .node import walk as _nwalk
from .values import CURRENCIES, Thing, fmt

DYN = Ty('dyn')


def install(C):
    for name, fn in list(globals().items()):
        if name.startswith('m_'):
            setattr(C, name[2:], fn)


def _c():
    from . import check
    return check


def uerr(msg, loc):
    return LoadError('LOAD-UNIT', msg, loc)


def _txt(n):
    return _c()._txt(n)


def span_txt(sp):
    if sp is None:
        return ''
    if sp.k != 'span':
        return _txt(sp)
    a = _txt(sp.a) if sp.a is not None else ''
    if not sp.dots:
        return a
    b = _txt(sp.b) if sp.b is not None else ''
    return '%s..%s' % (a, b)


# -- unit types ------------------------------------------------------------------------------------

def m_unit_type(self, u, regime=None):
    """The static type of a quantity of unit u: money for a currency, num u for a unit, num of the unit kind
    (a stand-in unit named by the kind) for a leg of a unit kind, dyn for any."""
    m = self.m
    if u is None or u in ('any', 'one'):
        return DYN
    if u in CURRENCIES:
        return T.money(u)
    if u in m.units or u in m.unit_kind_keys:
        return T.num(((u, 1),))
    hit = m.lookup(u, regime)
    if hit is not None and hit[0] == 'kind':
        return T.num(((hit[1].id, 1),))
    return DYN


def m_leg_type(self, legdecl):
    m = self.m
    at = self.unit_type(legdecl.unit, m.kinds[legdecl.kind].regime)
    return T.record([('from', T.key('places')), ('to', T.key('places')), ('amount', at), ('day', T.DATE)])


def m_place_type_unit(self, t, node=None):
    """The unit type of a place given by an expression of type t (a key of a place kind)."""
    m = self.m
    r = getattr(node, '_r', None) if node is not None else None
    th = self.literal_place(node) if node is not None else None
    if th is not None:
        return self.unit_type(m.place_unit(m.things[th]))
    if r is not None and r[0] == 'place':
        # a keyed place section gives each instance the section's unit (L §5.10)
        pi = m.places[r[1]]
        units = set(m.place_unit(x) for x in pi.things.values()) if pi.things else set()
        if pi.unit is not None and pi.unit != 'any':
            units.add(pi.unit)
        units.discard(None)
        if len(units) == 1 and 'any' not in units:
            return self.unit_type(list(units)[0])
    if t.k == 'key' and t.ix in m.kinds and m.kinds[t.ix].spec == 'place':
        u = m.kind_unit(t.ix)
        return self.unit_type(u)
    return DYN


def m_literal_place(self, node):
    """The thing name of a place written literally (an id, a key of a place kind, a keyed place at literal
    keys), else None."""
    m = self.m
    if node is None:
        return None
    r = getattr(node, '_r', None)
    if r is None:
        return None
    if r[0] == 'key' and isinstance(r[1], Thing) and r[1].name in m.things:
        return r[1].name
    if r[0] == 'place':
        pi = m.places[r[1]]
        if not pi.binds:
            return pi.things[()].name
        convs = getattr(node, '_convs', None)
        if convs and all(c[0] == 'lit' for c in convs):
            from .model import _kv
            t = pi.things.get(tuple(_kv(c[1]) for c in convs))
            return t.name if t is not None else None
    return None


def m_place_target(self, node):
    """What a running read names, for LOAD-READS (E §3.1 step 7)."""
    th = self.literal_place(node)
    if th is not None:
        return ('thing', th)
    r = getattr(node, '_r', None)
    if r is not None and r[0] == 'place':
        return ('placesec', r[1])
    return ('unknown',)


def m_is_place_type(self, t):
    m = self.m
    return t.k == 'key' and t.ix in m.kinds and m.kinds[t.ix].spec == 'place'


def m_is_move_type(self, t):
    m = self.m
    return t.k == 'key' and t.ix in m.kinds and m.kinds[t.ix].spec == 'event'


# -- places and movement references ---------------------------------------------------------------------

def m_place_value(self, n, info, keys, sc):
    """A place read by its id, or a keyed place at keys (L §5.10)."""
    if len(keys) != len(info.binds):
        raise LoadError('LOAD-KEY', 'wrong number of keys for place %s: %d, not %d' % (info.id, len(keys),
                                                                                       len(info.binds)), n.loc)
    convs = []
    for kn, b in zip(keys, info.binds):
        if kn.k == 'star':
            raise LoadError('LOAD-PARSE', 'cannot parse: * stands only in a pin', kn.loc)
        convs.append(self.key_pos(kn, b.dom, sc))
    n._r = ('place', info.id)
    n._convs = convs
    self.edge(sc, ('place', info.id))
    return T.key(info.kind)


def m_move_value(self, n, info, keys, sc):
    """A movement instance named by its move and keys (`rent[m]`, L §2.5)."""
    if len(keys) != len(info.binds):
        raise LoadError('LOAD-KEY', 'wrong number of keys for %s: %d, not %d' % (info.id, len(keys), len(info.binds)),
                        n.loc)
    convs = []
    for kn, b in zip(keys, info.binds):
        # a stepped range binds keys as a def's does (L §5.11, §2.5)
        convs.append(self.key_pos(kn, self.move_dom(b), sc))
    n._r = ('moveref', info.id)
    n._convs = convs
    self.edge(sc, ('move', info.id))
    if info.decision:
        self.mark(sc, 'legs', ('move', info.id))
    return T.key(self.move_kind(info))


def m_move_dom(self, b):
    """The domain of a move's key bind for key positions: an event kind's or an observed input's movements are
    keys of kind events."""
    d = b.dom
    if d[0] == 'kind' and self.m.kinds[d[1]].spec == 'event':
        return ('kind', d[1])
    return d


def m_move_kind(self, info):
    if info.kind is None:
        self.resolve_move_kind(info)
    return info.kind


def m_resolve_move_kind(self, info):
    m = self.m
    s = info.decl
    if '.' in s.kind:
        q, x = s.kind.split('.', 1)
        hit = m.qualified(q, x)
    else:
        hit = m.lookup(s.kind, info.regime)
    if hit is None or hit[0] != 'kind':
        raise LoadError('LOAD-NAME', 'unknown id %s' % s.kind, s.loc)
    k = m.kinds[hit[1].id]
    if k.spec != 'event':
        raise LoadError('LOAD-KIND', 'move %s of %s, not an event kind' % (info.id, k.id), s.loc)
    if k.id == 'adjustments' or 'adjustments' in k.ancestors(m):
        # L §5.14.3: "Only the engine makes adjustments: a move or an observed row of kind adjustments, or of a subkind
        # of it, is refused (LOAD-KIND)"
        raise LoadError('LOAD-KIND', 'move %s of %s: only the engine makes adjustments' % (info.id, k.id), s.loc)
    info.kind = k.id


# -- marks: running reads (E §3.1 step 7) -----------------------------------------------------------------

def m_mark(self, sc, what, target, node=None):
    self.marks.setdefault(sc.item, set()).add((what, target))
    if node is not None and what in ('balnow',):
        self.balnow_items.add(sc.item)


# -- fields of movements (L §5.14, §2.2) -----------------------------------------------------------------

def m_movement_field(self, n, t, sc):
    """`s.cash`, `s.day`, `s.seq`, `a.adjusts`, `a.change` (L §5.14, §5.14.3)."""
    m = self.m
    name = n.name
    kid = t.ix
    legs = m.kind_legs(kid)
    for ld in legs:
        if ld.name == name:
            n._r = ('mvfield', 'leg', name)
            return self.leg_type(ld)
    for a, at in m.kind_attrs(kid):
        if a == name:
            n._r = ('mvfield', 'attr', name)
            if at is None:
                if kid == 'adjustments' or 'adjustments' in m.kinds[kid].ancestors(m):
                    if name == 'adjusts':
                        return T.key('events')
                    return DYN
                return DYN
            return at
    if name == 'day':
        n._r = ('mvfield', 'day', name)
        return T.DATE
    if name == 'fulfils':
        n._r = ('mvfield', 'fulfils', name)
        return T.key('events')
    if kid == 'events':
        # a movement of a kind not known statically (a.adjusts, an observed row): its legs and attributes are
        # read by name at run
        for k in m.kinds.values():
            if k.spec != 'event':
                continue
            for ld in m.kind_legs(k.id):
                if ld.name == name:
                    n._r = ('mvfield', 'leg', name)
                    return T.record([('from', T.key('places')), ('to', T.key('places')), ('amount', DYN),
                                     ('day', T.DATE)])
            for a, at in m.kind_attrs(k.id):
                if a == name:
                    n._r = ('mvfield', 'attr', name)
                    return at if at is not None else DYN
    # L §2.2: the built-in fields of a movement are `day fulfils`; it has no readable kind (a binding over a kind, or
    # `m in K`, tests it); a lifespan's from and to are no fields of a movement
    raise LoadError('LOAD-FIELD', '%s is not a field of %s' % (name, kid), n.loc)


# -- bindings over movements (L §5.6, §5.7, §5.19) --------------------------------------------------------------

def m_bind_movements(self, b, sc, kinfo=None, obs=None, key_bind=False):
    """A binding over an event kind or an observed input's rows: needs on or over (LOAD-ALIVE), but a fill's key bind
    (L §5.6, round 18 C1-38)."""
    m = self.m
    if b.on is None and b.over is None and not key_bind:
        raise LoadError('LOAD-ALIVE', 'binding over %s needs on or over' % (kinfo.id if kinfo else obs.id), b.loc)
    if kinfo is not None:
        b._dom = ('event', kinfo.id)
        sub = kinfo.subtree(m)
        for qid, mi in m.moves.items():
            if self.move_kind(mi) in sub:
                self.edge(sc, ('move', qid), 'bind')
                if mi.decision:
                    self.mark(sc, 'legs', ('move', qid))
        if 'pads' in sub:
            # a binding over pads is a running read (L §5.14.1 Running reads)
            self.mark(sc, 'legs', ('kind', 'pads'))
        for qid, inf in m.inputs.items():
            if inf.observed:
                self.edge(sc, ('input', qid), 'bind')
        ty = T.key(kinfo.id)
        # a binding over an event kind needs no covered guard: its plan instances stand on the days no statement
        # covers (L §5.19, round 7); LOAD-COVERED guards an aggregate of an observed input's rows
        certain = False
    else:
        b._dom = ('obs', obs.id)
        self.edge(sc, ('input', obs.id), 'bind')
        # the input is named here, not reached through a kind (L §6.4)
        self.edge(sc, ('input', obs.id), 'named')
        ty = T.key('events')
        certain = True
    if certain and sc.item[0] == 'def':
        # LOAD-COVERED (L §5.19): a def that aggregates observations over a span guards the aggregate with
        # covered(x, span), in its rule's when or an if
        st = span_txt(b.over) if b.over is not None else span_txt(b.on)
        if st not in sc.covered_spans:
            raise LoadError('LOAD-COVERED', '%s aggregates observations over a span with no covered() guard (%s in %s)'
                            % (sc.item[1], b.var, st), b.loc)
    return ty


def m_covered_spans_of(self, e):
    out = set()
    if e is None:
        return out
    for x in walk(e):
        if x.k == 'call' and x.name == 'covered' and len(x.args) == 2:
            out.add(span_txt(x.args[1]))
    return out


# -- moves (L §5.14) -----------------------------------------------------------------------------------

def m_check_move(self, mi):
    m = self.m
    s = mi.decl
    check = _c()
    self.resolve_move_kind(mi)
    kid = mi.kind
    sc = check.Scope(self, ('move', mi.id), mi.regime, 'def')
    sc.covered_spans = set()
    for bi in mi.binds:
        bn = bi.node
        v = self.bind(bn, sc)
        if bn._dom[0] == 'kind' and bn.on is None and bn.over is None:
            raise LoadError('LOAD-ALIVE', 'binding over %s needs on or over (a move\'s keys)' % bn._dom[1], s.loc)
        bi.dom = bn._dom if bn._dom[0] != 'event' else ('kind', bn._dom[1])
        bi.mdom = bn._dom
    sc.keyvars = [b.var for b in mi.binds]
    # on: the movement's day; a decision's day reads no running state (LOAD-READS, checked on the graph)
    son = sc.child()
    son.item = ('moveon', mi.id)
    self.edge(sc, ('moveon', mi.id))
    t = self.tc(s.on, son)
    if not (self.is_date(t) or t.k in ('hole',)):
        raise uerr('on of move %s is %s, not a date' % (mi.id, T.show(t)), s.loc)
    if s.at is not None:
        t = self.tc(s.at, son)
        if t.k == 'period' or (t.k == 'key' and self.key_value_kind(t) == 'period'):
            raise LoadError('LOAD-LAWPERIOD', 'at %s: a period is not a law date' % _txt(s.at), s.loc)
        if not self.is_date(t):
            raise uerr('at of move %s is %s, not a date' % (mi.id, T.show(t)), s.loc)
        from .node import walk as _nwalk
        mi.at_defs = sorted(set(x._r[2] for x in _nwalk(s.at) if getattr(x, '_r', None) and len(x._r) > 2 and
                                x._r[0] == 'read' and x._r[1] == 'def'))
    if s.forp is not None:
        self.tc(s.forp, sc)
    if s.when is not None:
        sw = sc.child()
        sw.in_when = True
        t = self.tc(s.when, sw)
        if not self.is_bool(t):
            raise uerr('when of move %s is %s, not bool' % (mi.id, T.show(t)), s.loc)
    # legs: every leg its kind names, and no other (LOAD-LEG)
    decls = m.kind_legs(kid)
    names = [d.name for d in decls]
    seen = set()
    for ln in s.legs:
        if ln.name not in names:
            raise LoadError('LOAD-LEG', 'leg %s is not a leg of %s' % (ln.name, kid), s.loc)
        if ln.name in seen:
            raise LoadError('LOAD-LEG', 'leg %s twice in %s' % (ln.name, mi.id), s.loc)
        seen.add(ln.name)
    for d in decls:
        if d.name not in seen:
            raise LoadError('LOAD-LEG', '%s lacks leg %s' % (mi.id, d.name), s.loc)
    mi.legs = []
    for ln in s.legs:
        d = decls[names.index(ln.name)]
        self.check_leg(mi, ln, d, sc)
        mi.legs.append((ln, d))
    # attributes: each the kind declares (LOAD-LEG)
    attrs = dict(m.kind_attrs(kid))
    seen = set()
    for a, e in s.attrs:
        if a not in attrs:
            raise LoadError('LOAD-LEG', 'attribute %s is missing or unknown (%s)' % (a, mi.id), s.loc)
        if a in seen:
            raise LoadError('LOAD-LEG', 'attribute %s twice in %s' % (a, mi.id), s.loc)
        seen.add(a)
        at = attrs[a]
        if at is None:
            raise AssertionError((a, kid))   # only adjustments has untyped attributes, and no move is of it
        self.tc(e, sc, expect=at)
        self.assign(at, e, s.loc, 'attribute %s of %s' % (a, mi.id))
    for a in attrs:
        if a not in seen:
            raise LoadError('LOAD-LEG', 'attribute %s is missing or unknown (%s)' % (a, mi.id), s.loc)
    mi.fulfils_adj = False
    if s.fulfils is not None:
        t = self.tc(s.fulfils, sc)
        if not self.is_move_type(t):
            raise uerr('fulfils of %s names %s, not a movement' % (mi.id, T.show(t)), s.loc)
        # E §5 step 2, L §5.14.3 (round 19, GD-17): "A movement whose fulfils names an adjustment (a settlement)
        # isn't measured"
        mi.fulfils_adj = t.k == 'key' and t.ix in m.kinds['adjustments'].subtree(m)
    if s.tie is not None:
        for e in s.tie:
            self.tc(e, sc)
    # reads: places and movements (L §5.14.1)
    mi.read_targets = []
    for r in s.reads:
        mi.read_targets.append(self.read_target(r, sc, mi))
    for w, other in s.order:
        hit = m.lookup(other, mi.regime)
        if hit is None or hit[0] != 'move':
            raise LoadError('LOAD-NAME', 'unknown move %s (%s %s)' % (other, w, other), s.loc)
    if s.reads:
        # a decision's plain bal(x, d) of its own day reads its own result, and of a later day its future: refused,
        # LOAD-CYCLE where load sees the day (L §5.14.1); here, a day written as the decision's own `on`, or that
        # plus a positive int or duration
        from .node import walk as _nwalk
        ontxt = _txt(s.on)
        exprs = [s.when] + [ln.amount for ln in s.legs] + [ln.frm for ln in s.legs] + [ln.to for ln in s.legs] + \
            [e for a, e in s.attrs]
        for e in exprs:
            if e is None:
                continue
            for x in _nwalk(e):
                if x.k != 'bal' or getattr(x, '_form', None) != 'day' or not x.args:
                    continue
                dn = x.args[-1]
                later = dn.k == 'bin' and dn.op == '+' and _txt(dn.l) == ontxt and dn.r.k == 'num' and \
                    (dn.r.v or 0) > 0
                # L §5.14.1 `bal(x, d start)`: "On the decision's day it reads none of that day's movements, its own
                # result included, so it is no cycle; for d after that day it is the decision's future, refused as
                # bal(x, d) is"
                if (_txt(dn) == ontxt and not x.start) or later:
                    raise LoadError('LOAD-CYCLE', '%s reads bal(%s, %s), %s: its own result or its future (write '
                                    'the projection)' % (mi.id, _txt(x.ref), _txt(dn), 'its own day' if not later
                                                         else 'a later day'), s.loc)
                self.cycle_cands.append((mi, x, dn))
    mi.sc = sc


def m_read_target(self, r, sc, mi):
    m = self.m
    if r.k == 'name' and r.id not in sc.vars:
        hit = m.lookup(r.id, mi.regime)
        if hit is not None and hit[0] == 'move':
            r._r = ('move', hit[1].id)
            return ('move', hit[1].id)
        if hit is not None and hit[0] == 'place' and hit[1].binds:
            # a keyed place named without keys stands for all its instances (`reads wcash`, L §5.14.1)
            r._r = ('placesec', hit[1].id)
            return ('placesec', hit[1].id)
        if hit is not None and hit[0] == 'kind' and m.kinds[hit[1].id].spec == 'event':
            # L §5.14.1 Running reads: a binding over a decision's movements or over pads: "the decision declares
            # the move or the kind in reads (reads pads)"
            r._r = ('kind', hit[1].id)
            return ('kind', hit[1].id)
    t = self.tc(r, sc, expect=T.key('places'))
    if not self.is_place_type(t):
        raise LoadError('LOAD-READS', '%s declares reads %s, which is not a place or a move' % (mi.id, _txt(r)), r.loc)
    return self.place_target(r)


def m_check_leg(self, mi, ln, d, sc):
    m = self.m
    pt = T.key('places')
    ends = []
    for side, e in (('from', ln.frm), ('to', ln.to)):
        t = self.tc(e, sc, expect=pt)
        if t.k == 'hole':
            ends.append(None)
            continue
        if not self.is_place_type(t):
            raise uerr('leg %s of %s: %s is %s, not a place' % (ln.name, mi.id, side, T.show(t)), ln.loc)
        if t.ix in m.kinds['assets'].subtree(m):
            raise LoadError('LOAD-KIND', 'a leg to or from an asset (%s of %s)' % (ln.name, mi.id), ln.loc)
        th = self.literal_place(e)
        if th is not None and m.things[th].base in m.kinds['assets'].subtree(m):
            raise LoadError('LOAD-KIND', 'a leg to or from an asset (%s of %s)' % (ln.name, mi.id), ln.loc)
        ends.append(th)
    lt = self.unit_type(d.unit, m.kinds[d.kind].regime)
    units = set()
    for th in ends:
        if th is None:
            continue
        pu = m.place_unit(m.things[th])
        if pu not in (None, 'any'):
            units.add(pu)
    if len(units) > 1:
        raise uerr('leg %s of %s runs between places of different units (%s)' % (ln.name, mi.id,
                                                                                 ', '.join(sorted(units))), ln.loc)
    if d.unit not in ('any',) and units:
        u = list(units)[0]
        if d.unit in CURRENCIES or d.unit in m.units:
            if u != d.unit:
                raise uerr('leg %s of %s is in %s; its places hold %s' % (ln.name, mi.id, d.unit, u), ln.loc)
    if lt.k == 'dyn' and len(units) == 1:
        lt = self.unit_type(list(units)[0])
    hitk = m.lookup(d.unit, m.kinds[d.kind].regime) if d.unit not in ('any', 'one') else None
    ukind = hitk[1].id if (hitk is not None and hitk[0] == 'kind' and d.unit not in m.units and
                           d.unit not in CURRENCIES) else None
    if ukind is not None:
        # a leg of a unit kind: the amount is in one of the kind's units (`4.2167 etf` for `leg units: funds`,
        # L §5.7, §5.14); which one is known from the value
        t = self.tc(ln.amount, sc)
        if t.k not in ('dyn', 'hole', 'flex') and not (t.k == 'num' and len(t.u or ()) == 1 and
                                                      m.unit_kind_keys.get(t.u[0][0]) in
                                                      m.kinds[ukind].subtree(m) and t.u[0][1] == 1):
            raise uerr('the amount of leg %s of %s is %s, not a unit of %s' % (ln.name, mi.id, T.show(t), ukind),
                       ln.loc)
    else:
        t = self.tc(ln.amount, sc, expect=lt if lt.k != 'dyn' else None)
    if lt.k != 'dyn' and ukind is None:
        self.assign(lt, ln.amount, ln.loc, 'the amount of leg %s of %s' % (ln.name, mi.id))
    elif not (self.numcls(t) in ('money', 'num', 'int', 'flex') or t.k in ('dyn', 'hole')):
        raise uerr('the amount of leg %s of %s is %s' % (ln.name, mi.id, T.show(t)), ln.loc)
    ln._lt = lt
    if ln.on is not None:
        t = self.tc(ln.on, sc)
        if not self.is_date(t):
            raise uerr('on of leg %s of %s is %s, not a date' % (ln.name, mi.id, T.show(t)), ln.loc)
    return ends


# -- balances, flows, projections (L §5.10, §5.14) --------------------------------------------------------------

def m_place_arg(self, r, sc, what):
    t = self.tc(r, sc, expect=T.key('places'))
    if not self.is_place_type(t):
        raise uerr('%s of %s, which is not a place' % (what, T.show(t)), r.loc)
    return t


def m_t_bal(self, n, sc, expect):
    m = self.m
    t = self.place_arg(n.ref, sc, 'bal')
    ut = self.place_type_unit(t, n.ref)
    target = self.place_target(n.ref)
    args = n.args
    n._unit = None
    if len(args) == 2:
        tu = self.tc(args[0], sc, expect=T.key('currencies'))
        if args[0].k == 'cur' or (getattr(args[0], '_r', None) and args[0]._r[0] == 'key'):
            uname = args[0]._r[1].name if args[0].k != 'cur' else args[0].v
            ut = self.unit_type(uname)
        else:
            ut = DYN
        n._unit = args[0]
        dnode = args[1]
    elif len(args) == 1:
        dnode = args[0]
    else:
        dnode = None
    if dnode is not None:
        dt = self.tc(dnode, sc)
        if not self.is_date(dt):
            raise uerr('the day of bal is %s, not a date' % T.show(dt), n.loc)
    elif n.start or n.modelled or n.proj:
        raise LoadError('LOAD-PARSE', 'cannot parse: bal(x) with no day takes no clause', n.loc)
    if n.modelled:
        n._form = 'modelled'
    elif n.proj and 'frm' in n.proj:
        n._form = 'proj'
        self.check_proj(n, sc, target)
    elif dnode is None:
        n._form = 'now'
        self.mark(sc, 'balnow', target, n)
        n._ctgt = ('balnow', target)
    else:
        n._form = 'day'
        self.mark(sc, 'bal', target)
        n._ctgt = ('bal', target)
    self.edge(sc, ('balances',), 'bal')
    return ut


def m_check_proj(self, n, sc, target):
    p = n.proj
    if 'frm' in p:
        ft = self.tc(p['frm'], sc)
        if not self.is_date(ft):
            raise uerr('from of a projection is %s, not a date' % T.show(ft), n.loc)
    if p.get('random') is not None and p['random'][0] == 'quantile':
        scq = sc.child()
        scq.allow_lit = sc.where == 'ask'
        qt = self.tc(p['random'][1], scq)
        if self.numcls(qt) not in ('num', 'flex', 'int'):
            raise uerr('the level of random at quantile is %s' % T.show(qt), n.loc)
    if 'frm' not in p:
        raise LoadError('LOAD-PARSE', 'cannot parse: a projection states from t (bal(x, d) from t)', n.loc)
    if p.get('windows') == 'each':
        # L §4.4: "The clause stands on a projection read ..., for that read": a line that reaches one splits by the
        # windows that read meets (ask.Asker)
        self.m.read_each = True
    self.mark(sc, 'proj', target)
    # L §8.3 item 10 (round 19, GD-4): a projection of a place a random movement books into is in the random cone;
    # one with a random clause reads the amounts after t as the question L §5.16 writes ("which reads no sample")
    if p.get('random') is None:
        self.proj_nc.setdefault(sc.item, set()).add(target)
        n._ctgt = ('proj', target)
    else:
        n._ctgt = ('projc', target)
    self.projs.append((n, sc.item))
    n._holder = sc.item


def m_t_low(self, n, sc, expect):
    t = self.place_arg(n.ref, sc, 'low')
    ut = self.place_type_unit(t, n.ref)
    target = self.place_target(n.ref)
    self.span(n.span, sc)
    if n.span.a is None or n.span.b is None:
        raise LoadError('LOAD-DOMAIN', 'low over a span with an open end', n.loc)
    if n.proj and 'frm' in n.proj:
        self.check_proj(n, sc, target)
        n._form = 'proj'
    else:
        n._form = 'day'
        self.mark(sc, 'bal', target)
        n._ctgt = ('bal', target)
    self.edge(sc, ('balances',), 'bal')
    return ut


def m_t_flows(self, n, sc, expect):
    m = self.m
    r = n.ref
    legname = None
    mref = r
    if r.k == 'field':
        # flows(buy.eur, span): a move's leg (L §5.14.3)
        legname = r.name
        mref = r.obj
    base = mref.obj if mref.k == 'index' else mref
    hit = None
    if base.k == 'name' and base.id not in sc.vars:
        hit = m.lookup(base.id, sc.regime)
    if hit is not None and hit[0] == 'move':
        mi = hit[1]
        self.move_kind(mi)
        keys = mref.keys if mref.k == 'index' else []
        if len(keys) > len(mi.binds):
            raise LoadError('LOAD-KEY', 'wrong number of keys for %s' % mi.id, n.loc)
        n._convs = [self.key_pos(kn, self.move_dom(b), sc) for kn, b in zip(keys, mi.binds)]
        n._form = 'move'
        n._move = mi.id
        n._mref = mref
        legs = m.kind_legs(mi.kind)
        if legname is not None:
            legs = [d for d in legs if d.name == legname]
            if not legs:
                raise LoadError('LOAD-LEG', 'flows(%s.%s, span): %s names no leg %s' % (mi.id, legname, mi.kind,
                                                                                      legname), n.loc)
        elif len(legs) != 1:
            # a move whose kind has several legs names the leg, flows(buy.eur, 2026) (L §5.14.3)
            raise LoadError('LOAD-LEG', 'flows(%s, span): the kind %s has %d legs; name one, flows(%s.<leg>, ...)'
                            % (mi.id, mi.kind, len(legs), mi.id), n.loc)
        n._leg = legs[0].name
        ut = self.unit_type(legs[0].unit, m.kinds[legs[0].kind].regime)
        if ut.k == 'dyn':
            units = set()
            for ln, d in getattr(mi, 'legs', []) or []:
                lt = getattr(ln, '_lt', None)
                if lt is not None and lt.k != 'dyn' and ln.name == n._leg:
                    units.add(lt)
            if len(units) == 1:
                ut = list(units)[0]
        # the one fixed unit of the leg's places, where both are written by name: the unit of an empty span
        # (L §5.14.3: "Its unit is that of the places it reads ... the leg's places' for a move")
        n._leg_unit = None
        for ln, d in getattr(mi, 'legs', []) or []:
            if getattr(ln, 'name', None) != n._leg:
                continue
            us = set()
            for side in (ln.frm, ln.to):
                th = self.literal_place(side) if side is not None else None
                us.add(m.place_unit(m.things[th]) if th is not None else None)
            if len(us) == 1 and list(us)[0] not in (None, 'any', 'one'):
                n._leg_unit = list(us)[0]
        self.edge(sc, ('move', mi.id), 'flows')
        if mi.decision:
            self.mark(sc, 'flows', ('move', mi.id))
        n._ctgt = ('flows', ('move', mi.id))
    else:
        t = self.place_arg(r, sc, 'flows')
        ut = self.place_type_unit(t, r)
        n._form = 'place'
        if n.conv is None:
            # L §5.14.3 (round 12): "a place of unit any ... is refused (LOAD-UNIT where load sees it, RUN-DOMAIN
            # otherwise), unless the flows converts"
            th = self.literal_place(r)
            pu = m.place_unit(m.things[th]) if th is not None else (
                m.kind_unit(t.ix) if t.k == 'key' and t.ix in m.kinds else None)
            if pu == 'any':
                raise uerr('flows of %s, a place of unit any: it sums no one unit; convert it (in CUR at fx mode)'
                           % _txt(r), n.loc)
        self.mark(sc, 'flows', self.place_target(r))
        n._ctgt = ('flows', self.place_target(r))
        self.edge(sc, ('balances',), 'flows')
    self.span(n.span, sc)
    if n.forkey is not None:
        self.tc(n.forkey, sc)
    if n.conv is not None:
        cur, fx, mode = n.conv
        if cur not in CURRENCIES:
            raise uerr('money in an undeclared currency %s' % cur, n.loc)
        ft = self.tc(fx, sc)
        if ft.k != 'num' or not ft.u:
            raise uerr('flows in %s at %s: the rate is %s' % (cur, _txt(fx), T.show(ft)), n.loc)
        self.mode_arg(mode, sc, n)
        return T.money(cur)
    return ut


def m_t_value(self, n, sc, expect):
    m = self.m
    t = self.place_arg(n.ref, sc, 'value')
    dt = self.tc(n.d, sc)
    if not self.is_date(dt):
        raise uerr('the day of value is %s, not a date' % T.show(dt), n.loc)
    if n.cur not in CURRENCIES:
        raise uerr('money in an undeclared currency %s' % n.cur, n.loc)
    th = self.literal_place(n.ref)
    if th is not None:
        cands = [m.things[th]]
    elif t.k == 'key':
        sub = m.kinds[t.ix].subtree(m) if t.ix in m.kinds else set()
        cands = [x for x in m.things.values() if x.base in sub]
    else:
        cands = []
    self.price_edges(sc, cands)
    if n.by is None:
        pu = None
        priced = None
        if th is not None:
            pu = m.place_unit(m.things[th])
            try:
                priced = m.price_of(m.things[th])
            except Unspecified:
                priced = True
        elif t.k == 'key':
            pu = m.kind_unit(t.ix)
            priced = m.kind_priced(t.ix)
        if pu not in (None, 'any', 'one', n.cur) and not priced:
            raise LoadError('LOAD-PRICE', 'value(%s) in %s holds units in %s and states no by' % (_txt(n.ref), n.cur, pu),
                            n.loc)
        if priced is not None and priced is not True and getattr(priced, '_mode', None) == 'whole' and \
                priced._cur != n.cur:
            # the place's value is in priced._cur: a position in another unit than CUR needs `by p` (L §5.10)
            raise LoadError('LOAD-PRICE', 'value(%s) in %s holds units in %s and states no by' % (
                _txt(n.ref), n.cur, priced._cur), n.loc)
    else:
        sc2 = sc.child()
        sc2.readmode = 'explicit'
        by = n.by
        hit = m.lookup(by.id, sc.regime) if (by.k == 'name' and by.id not in sc.vars) else None
        if by.k == 'name' and hit is not None and hit[0] in ('input', 'def') and not n.bylast and \
                len(hit[1].binds) == 0:
            # an unkeyed price, num CUR/u: an expression of that type with its keys (none) (L §5.10)
            hit = None
        if hit is not None and hit[0] in ('input', 'def'):
            # an input or def of type num CUR/u keyed by unit, read at the position's unit (L §5.10)
            inf = hit[1]
            if len(inf.binds) != 1 or inf.type is None or inf.type.k != 'num':
                raise uerr('value by %s: a price is a num CUR/u keyed by unit, or an expression of that type with its '
                           'keys' % by.id, n.loc)
            if n.bylast and hit[0] != 'input':
                raise uerr('value by last(%s): last reads an input (L §4.6)' % by.id, n.loc)
            by._r = (hit[0], inf.id)
            n._price = inf.id
            n._price_kind = hit[0]
            self.edge(sc, (hit[0], inf.id), 'explicit')
        elif by.k == 'name' and hit is None and n.bylast:
            raise LoadError('LOAD-NAME', 'unknown price input %s' % by.id, n.loc)
        elif n.bylast:
            raise uerr('value by last(%s): last reads an input (L §4.6)' % _txt(by), n.loc)
        else:
            # an expression of type num CUR/u written with its keys (`by etf_price[month_of(d)]`, L §5.10, round 7b)
            pt = self.tc(by, sc)
            if pt.k != 'num' or not any(b == n.cur and e == 1 for b, e in (pt.u or ())) or len(pt.u) != 2:
                raise uerr('value by %s: the price is %s, not num %s/u' % (_txt(by), T.show(pt), n.cur), n.loc)
            n._price_expr = True
    self.edge(sc, ('balances',), 'bal')
    self.mark(sc, 'bal', self.place_target(n.ref))
    n._ctgt = ('bal', self.place_target(n.ref))
    return T.num(((n.cur, 1),))


def m_check_priced(self):
    """`priced p` / `priced last(p)` on a kind, a place section or an entry (L §2.3 price, L §5.10 Value,
    round 10): p names an input or a def; its type is num CUR (or money CUR), the value of the place's single position
    (tally 1's asset), or num CUR/u, a price per unit of its positions (a holding's `priced etf_price`)."""
    m = self.m
    seen = set()
    srcs = []
    for kid in sorted(m.kinds):
        k = m.kinds[kid]
        if k.priced is not None:
            srcs.append((k.priced, k.regime, 'kind %s' % kid))
    for name in sorted(m.things):
        for e in m.things[name].entries:
            if e.priced is not None:
                srcs.append((e.priced, None, 'place %s' % name))
    for pn, regime, what in srcs:
        if id(pn) in seen:
            continue
        seen.add(id(pn))
        self.resolve_price(pn, regime, what)
    # L §5.10 Value: "a thing of assets (a subkind included) names its price input on its kind, its place section or
    # its entry (LOAD-PRICE otherwise)"; L §8.3: "asset <x> names no price input"
    sub = m.kinds['assets'].subtree(m) if 'assets' in m.kinds else set()
    for name in sorted(m.things):
        th = m.things[name]
        if getattr(th, 'base', None) in sub and m.price_of(th) is None:
            loc = None
            for e in getattr(th, 'entries', ()) or ():
                loc = getattr(e, 'loc', None) or loc
            raise LoadError('LOAD-PRICE', 'asset %s names no price input' % name, loc)


def m_resolve_price(self, pn, regime, what):
    m = self.m
    r = pn.ref
    pn._expr = None
    if r.k == 'index' and r.obj.k == 'name':
        # L §2.3 price = ref | "last" "(" ref ")": a price input read at keys the reference states, literal keys or
        # the place section's own key variables (`place h[f in funds]: holdings units priced px[f]`), read at the
        # value's day (L §5.10)
        from .check import Scope, Var
        pi = None
        for q in sorted(m.places):
            if m.places[q].priced is pn:
                pi = m.places[q]
        sc = Scope(self, ('price', what), regime, 'def', [b.var for b in pi.binds] if pi is not None else [])
        if pi is not None:
            for b in pi.binds:
                sc.vars[b.var] = Var(b.var, self.dom_type(b.dom, b.node, sc), b.dom, b.node)
        t = self.tc(r, sc)
        rr = getattr(r, '_r', None)
        if not (rr and rr[0] == 'read' and rr[1] in ('input', 'def')):
            raise uerr('%s is priced by %s, not an input or a def' % (what, _txt(r)), pn.loc)
        if pn.last and rr[1] != 'input':
            raise uerr('%s is priced by last(%s): last reads an input (L §4.6)' % (what, _txt(r)), pn.loc)
        pn._expr = (pi, r)
        r = r.obj
    elif r.k != 'name':
        raise NotYet('a priced reference with fields (%s, %s)' % (_txt(r), what), pn.loc)
    hit = m.lookup(r.id, regime)
    if hit is None:
        raise LoadError('LOAD-NAME', 'unknown id %s' % r.id, pn.loc)
    kind, info = hit
    if kind not in ('input', 'def'):
        raise uerr('%s is priced by %s, a %s, not an input or a def' % (what, r.id, kind), pn.loc)
    if pn.last and kind != 'input':
        raise uerr('%s is priced by last(%s): last reads an input (L §4.6)' % (what, r.id), pn.loc)
    t = info.type
    u = None
    if t is not None and t.k == 'money':
        u = ((t.cur, 1),)
    elif t is not None and t.k == 'num':
        u = tuple(sorted(t.u or ()))
    curs = [b for b, e in (u or ()) if b in CURRENCIES and e == 1]
    if u is not None and len(u) == 1 and curs:
        pn._mode = 'whole'
        pn._cur = curs[0]
        pn._per = None
    elif u is not None and len(u) == 2 and len(curs) == 1 and any(e == -1 for b, e in u):
        pn._mode = 'unit'
        pn._cur = curs[0]
        pn._per = [b for b, e in u if e == -1][0]
    else:
        raise uerr('%s is priced by %s of type %s, not num CUR (a value) or num CUR/u (a price per unit)' % (
            what, r.id, T.show(t) if t is not None else '?'), pn.loc)
    if len(info.binds) > 1:
        raise LoadError('LOAD-KEY', 'wrong number of keys: %s is priced by %s, keyed by %d indexes' % (
            what, r.id, len(info.binds)), pn.loc)
    pn._r = (kind, info.id)


def m_price_edges(self, sc, things):
    """A value read reads the prices its places are priced by (L §5.10), at the value's day."""
    m = self.m
    for th in things:
        try:
            pn = m.price_of(th)
        except Unspecified:
            continue
        if pn is None or getattr(pn, '_r', None) is None:
            continue
        self.edge(sc, pn._r, 'explicit')


def m_c_next(self, n, sc, expect):
    m = self.m
    if len(n.args) != 2:
        raise LoadError('LOAD-PARSE', 'cannot parse: next(K, d) takes two arguments', n.loc)
    kn = n.args[0]
    if kn.k == 'name':
        hit = m.lookup(kn.id, sc.regime)
    elif kn.k == 'field' and kn.obj.k == 'name':
        hit = m.qualified(kn.obj.id, kn.name)
    else:
        hit = None
    if hit is None or hit[0] != 'kind' or m.kinds[hit[1].id].spec != 'event':
        raise LoadError('LOAD-NAME', 'next takes an event kind: %s' % _txt(kn), n.loc)
    kid = hit[1].id
    kn._r = ('kind', kid)
    kn._t = Ty('kindname')
    dt = self.tc(n.args[1], sc)
    if not self.is_date(dt):
        raise uerr('next(K, d): d is %s, not a date' % T.show(dt), n.loc)
    n._kind = kid
    sub = m.kinds[kid].subtree(m)
    for qid, mi in m.moves.items():
        if self.move_kind(mi) in sub:
            self.edge(sc, ('move', qid), 'next')
    return T.key(kid)


def m_c_covered(self, n, sc, expect):
    if len(n.args) != 2:
        raise LoadError('LOAD-PARSE', 'cannot parse: covered(x, span) takes two arguments', n.loc)
    self.place_arg(n.args[0], sc, 'covered')
    sp = n.args[1]
    if sp.k != 'span':
        raise LoadError('LOAD-PARSE', 'cannot parse: covered(x, span)', n.loc)
    self.span(sp, sc)
    return T.BOOL


def m_c_gap(self, n, sc, expect):
    if len(n.args) != 2:
        raise LoadError('LOAD-PARSE', 'cannot parse: gap(x, d) takes two arguments', n.loc)
    t = self.place_arg(n.args[0], sc, 'gap')
    dt = self.tc(n.args[1], sc)
    if not self.is_date(dt):
        raise uerr('gap(x, d): d is %s, not a date' % T.show(dt), n.loc)
    self.edge(sc, ('balances',), 'bal')
    return self.place_type_unit(t, n.args[0])


def m_c_observed(self, n, sc, expect):
    m = self.m
    if len(n.args) != 1:
        raise LoadError('LOAD-PARSE', 'cannot parse: observed(m[k]) takes one argument', n.loc)
    a = n.args[0]
    base = a.obj if a.k == 'index' else a
    hit = m.lookup(base.id, sc.regime) if base.k == 'name' and base.id not in sc.vars else None
    if hit is None or hit[0] != 'move':
        raise LoadError('LOAD-PARSE', 'cannot parse: observed takes a planned instance m[k]', n.loc)
    mi = hit[1]
    self.move_value(a, mi, a.keys if a.k == 'index' else [], sc)
    kid = mi.kind
    fs = [(ld.name, self.leg_type(ld)) for ld in m.kind_legs(kid)] + [('day', T.DATE)]
    for qid, inf in m.inputs.items():
        if inf.observed:
            self.edge(sc, ('input', qid), 'observed')
    return T.record(fs)


def m_c_unit(self, n, sc, expect):
    if len(n.args) != 1:
        raise LoadError('LOAD-PARSE', 'cannot parse: unit(x) takes one argument', n.loc)
    t = self.tc(n.args[0], sc, expect=T.key('places'))
    n._uq = False
    if not self.is_place_type(t):
        # unit(q) of a quantity, a leg's amount included: "the unit as a key of the unit kind that holds it
        # (currencies for a currency, §5.7)"; a quantity unit, a compound unit and a plain number are keys of no unit
        # kind, refused at load where load sees it (LOAD-UNIT), otherwise at the key (RUN-DOMAIN) (L §5.10 Kind and
        # unit; writer 8, sixth pass: refcore read unit(x) of a place only)
        return self.unit_of_quantity(n, t)
    # unit(x) is x's unit as a key of the unit kind that holds it (`currencies` for a currency); a place of unit any
    # has no one unit, and unit(x) of it is refused (L §5.10 Kind and unit, round 7c; §8.4: at load where the types
    # show it)
    m = self.m
    th = self.literal_place(n.args[0])
    units = None
    if th is not None:
        units = {m.place_unit(m.things[th])}
    elif t.k == 'key' and t.ix in m.kinds:
        ku = m.kind_unit(t.ix)
        if ku not in (None, 'one'):
            units = {ku}
        else:
            sub = m.kinds[t.ix].subtree(m)
            units = set(m.place_unit(x) for x in m.things.values() if x.base in sub and m.is_place_thing(x))
    if units and 'any' in units and len(units) == 1:
        raise uerr('unit(%s) of a place of unit any: it has no one unit' % _txt(n.args[0]), n.loc)
    if units and all(u not in CURRENCIES and u not in m.unit_kind_keys and u not in (None, 'any', 'one')
                     for u in units):
        # L §5.10 (round 12): "a quantity unit (unit hour …, §5.4) is a key of no unit kind, so unit(x) of either
        # place is refused (§8.4)", at load where the types show it
        raise uerr('unit(%s): its unit %s is a quantity unit, a key of no unit kind' % (
            _txt(n.args[0]), ', '.join(sorted(units))), n.loc)
    kinds = set()
    for u in (units or ()):
        if u in CURRENCIES:
            kinds.add('currencies')
        elif u in m.unit_kind_keys:
            kinds.add(m.unit_kind_keys[u])
        else:
            kinds.add(None)
    if len(kinds) == 1 and None not in kinds:
        return T.key(list(kinds)[0])
    return Ty('unitkey')


# -- the movement graph: running reads, LOAD-READS, the strata (E §3.1 steps 7, 8) -----------------------

def _tname(t):
    if t[0] == 'thing':
        return t[1]
    if t[0] in ('placesec', 'move', 'kind'):
        return t[1]
    return 'a place'


def _covers(declared, tgt, m):
    """Does a declared read cover a running read's target?"""
    if tgt[0] == 'unknown':
        return True
    for d in declared:
        if d == tgt:
            return True
        if d[0] == 'placesec' and tgt[0] == 'thing':
            t = m.things.get(tgt[1])
            if isinstance(t, PlaceThing) and t.place.id == d[1]:
                return True
        if d[0] == 'kind' and d[1] in m.kinds:
            sub = m.kinds[d[1]].subtree(m)
            if tgt[0] == 'kind' and tgt[1] in sub:
                return True
            if tgt[0] == 'move' and tgt[1] in m.moves and m.moves[tgt[1]].kind in sub:
                return True
        if d[0] == 'unknown':
            return True
    return False


def m_check_moves_graph(self):
    m = self.m
    E = self.edges
    memo = {}

    def reached(item):
        if item in memo:
            return memo[item]
        out = set()
        seen = set()
        stack = [item]
        while stack:
            x = stack.pop()
            if x in seen:
                continue
            seen.add(x)
            out |= self.marks.get(x, set())
            for t in E.get(x, {}):
                if t[0] in ('def', 'fill', 'moveon'):
                    stack.append(t)
        memo[item] = out
        return out
    self.reached_marks = reached
    # LOAD-READS (L §5.14.1, E §3.1 step 7)
    for qid in sorted(m.moves):
        mi = m.moves[qid]
        rs = reached(('move', qid))
        loc = mi.decl.loc
        if not mi.decision:
            if rs:
                w, tgt = sorted(rs, key=repr)[0]
                raise LoadError('LOAD-READS', '%s reaches a running read or projection of %s without reads %s' % (
                    qid, _tname(tgt), _tname(tgt)), loc)
            continue
        on_rs = reached(('moveon', qid))
        if on_rs:
            w, tgt = sorted(on_rs, key=repr)[0]
            raise LoadError('LOAD-READS', '%s: its day reads running state (%s); a decision\'s day reads no running '
                            'state' % (qid, _tname(tgt)), loc)
        for w, tgt in sorted(rs, key=repr):
            if not _covers(mi.read_targets, tgt, m):
                raise LoadError('LOAD-READS', '%s reaches a running read or projection of %s without reads %s' % (
                    qid, _tname(tgt), _tname(tgt)), loc)
        for d in mi.read_targets:
            if not any(_covers([d], tgt, m) and (tgt == d or tgt[0] != 'unknown') for w, tgt in rs) and \
                    not any(tgt[0] == 'unknown' for w, tgt in rs):
                raise LoadError('LOAD-READS', '%s declares reads %s and never reads it' % (qid, _tname(d)), loc)
    # bal(x) with no day outside a decision and the defs it reaches
    from_decisions = set()
    for qid, mi in m.moves.items():
        if not mi.decision:
            continue
        stack = [('move', qid)]
        while stack:
            x = stack.pop()
            if x in from_decisions:
                continue
            from_decisions.add(x)
            for t in E.get(x, {}):
                if t[0] in ('def', 'fill', 'moveon'):
                    stack.append(t)
    for item in sorted(self.balnow_items, key=repr):
        if item not in from_decisions:
            raise LoadError('LOAD-READS', '%s holds bal(x) with no day: a running read outside a decision and its '
                            'defs' % item[1], None)
    m.from_decisions = from_decisions
    # walk-bound defs: "A def that reaches a mark and that a decision reaches is walk-bound: its cells carry the walk
    # point (§3.2); in an ask, or a def no movement reaches, a plain bal(x, d) is the balance on day d at the ask's
    # cut" (E §3.1 step 7, round 18 C3-8; L §5.14.1). A plan that reaches a mark is LOAD-READS (above).
    m.walk_bound = set(d for d in m.defs if reached(('def', d)) and ('def', d) in from_decisions)
    m.walk_bound_fills = set(f for f in m.fills if reached(('fill', f)) and ('fill', f) in from_decisions)
    # the movements a projection leaves out: those that depend on its holder (E §3.1 step 7, L §5.14.1)
    inv = {}
    for r, ts in E.items():
        for t in ts:
            inv.setdefault(t, set()).add(r)
    lo_memo = {}
    for n, holder in self.projs:
        if holder not in lo_memo:
            if holder[0] == 'ask':
                lo_memo[holder] = frozenset()
            else:
                start = ('move', holder[1]) if holder[0] == 'moveon' else holder
                seen = set()
                stack = [start]
                moves = set()
                while stack:
                    x = stack.pop()
                    if x in seen:
                        continue
                    seen.add(x)
                    if x[0] == 'move':
                        moves.add(x[1])
                    for r in inv.get(x, ()):
                        stack.append(r)
                lo_memo[holder] = frozenset(moves)
        n._leftout = lo_memo[holder]
    self.strata()


def m_move_changes(self, mi):
    m = self.m
    out = set()
    for ln, d in mi.legs:
        for e in (ln.frm, ln.to):
            th = self.literal_place(e)
            if th is not None:
                out.add(('thing', th))
                continue
            r = getattr(e, '_r', None)
            if r is not None and r[0] == 'place':
                out.add(('placesec', r[1]))
                continue
            t = getattr(e, '_t', None)
            if t is not None and t.k == 'key':
                out.add(('kind', t.ix))
            else:
                out.add(('unknown',))
    return out


def _meets(read, changes, m):
    for c in changes:
        if c == read or c[0] == 'unknown' or read[0] == 'unknown':
            return True
        if c[0] == 'placesec' and read[0] == 'thing':
            t = m.things.get(read[1])
            if isinstance(t, PlaceThing) and t.place.id == c[1]:
                return True
        if read[0] == 'placesec' and c[0] == 'thing':
            t = m.things.get(c[1])
            if isinstance(t, PlaceThing) and t.place.id == read[1]:
                return True
        if c[0] == 'kind' and read[0] == 'thing':
            t = m.things.get(read[1])
            if t is not None and t.base in m.kinds[c[1]].subtree(m):
                return True
        if c[0] == 'kind' and read[0] == 'placesec':
            if m.places[read[1]].kind in m.kinds[c[1]].subtree(m):
                return True
    return False


def m_strata(self):
    """The decision graph and its layers (E §3.1 step 8, L §5.14.2)."""
    m = self.m
    dec = sorted(q for q, mi in m.moves.items() if mi.decision)
    changes = {q: self.move_changes(mi) for q, mi in m.moves.items()}

    def reaches(a, b):
        seen = set()
        stack = [a]
        while stack:
            x = stack.pop()
            if x == b:
                return True
            if x in seen:
                continue
            seen.add(x)
            stack.extend(self.edges.get(x, {}).keys())
        return False
    after = {q: set() for q in dec}
    for a in dec:
        mi = m.moves[a]
        for b in dec:
            reads_b = ('move', b) in mi.read_targets
            if a == b:
                continue
            if (any(_meets(r, changes[b], m) for r in mi.read_targets if r[0] != 'move') or reads_b) and \
                    not reaches(('move', b), ('move', a)):
                after[a].add(b)
    # before / after lines
    m.deferred_plans = {}
    for a in dec:
        mi = m.moves[a]
        for w, other in mi.decl.order:
            hit = m.lookup(other, mi.regime)
            b = hit[1].id
            if b not in after:
                if w == 'before':
                    # L §5.14.2 (round 19, GD-11, C4-1): "It orders a decision only against another decision: plans
                    # and observations come first, and before m with m a plan or an observation is refused
                    # (LOAD-ORDER)"; L §8.3 `<a> before <m>: <m> is a plan or an observation`
                    raise LoadError('LOAD-ORDER', '%s before %s: %s is a plan or an observation' % (a, b, b),
                                    mi.decl.loc)
                continue
            if w == 'before':
                if ('move', b) in mi.read_targets:
                    raise LoadError('LOAD-ORDER', '%s before %s contradicts a read of %s\'s legs' % (a, b, b),
                                    mi.decl.loc)
                after[a].discard(b)
                after[b].add(a)
            else:
                after[a].add(b)
                after[b].discard(a)
    # cycles left: LOAD-ORDER where the days are known at load, RUN-ORDER in the walk otherwise
    state = {}
    m.order_groups = []

    def visit(x, path):
        st = state.get(x)
        if st == 1:
            cyc = path[path.index(x):]
            known = all(not self.edges.get(('moveon', q)) for q in cyc)
            if known:
                raise LoadError('LOAD-ORDER', 'decisions %s read each other with no before or after' % (
                    ' and '.join(cyc)), m.moves[cyc[0]].decl.loc)
            m.order_groups.append(set(cyc))
            return
        if st == 2:
            return
        state[x] = 1
        for y in sorted(after[x]):
            visit(y, path + [x])
        state[x] = 2
    for x in dec:
        visit(x, [])
    # cycles that share a decision are one component
    merged = []
    for g in m.order_groups:
        g = set(g)
        for h in [h for h in merged if h & g]:
            g |= h
            merged.remove(h)
        merged.append(g)
    m.order_groups = merged
    for g in m.order_groups:
        for q in g:
            for p in g:
                after[q].discard(p)
        # a component no written order breaks is one stratum (E §3.1 step 8): its members act after what any of them
        # acts after, and the walk orders their instances by the places their legs give (E §4.2 step 3)
        ext = set()
        for q in g:
            ext |= after[q]
        ext -= set(g)
        for q in g:
            after[q] = set(ext)
    level = {}

    def lev(x, seen=()):
        if x in level:
            return level[x]
        if x in seen:
            return 1
        v = 1 + max([lev(y, seen + (x,)) for y in after[x]] or [0])
        level[x] = v
        return v
    for q, mi in m.moves.items():
        mi.stratum = lev(q) if mi.decision else 0
        mi.self_reads = mi.decision and any(_meets(r, changes[q], m) for r in mi.read_targets if r[0] != 'move')
    m.after = after


def m_check_move_lawdates(self):
    """A move states at when anything it reads is law-dated (L §4.3, §5.14: LOAD-LAWDATE)."""
    m = self.m
    E = self.edges
    skip = {'pin', 'calendar', 'explicit', 'lawpin', 'bind', 'flows', 'next', 'observed', 'leg', 'bal'}
    for qid, mi in m.moves.items():
        # an at reader def reached from `at e` itself has no law date to take: the chain ends at the law date `at`
        # gives (L §5.14 at e, round 7c): LOAD-LAWDATE
        for did in getattr(mi, 'at_defs', ()):
            if self.needs_reader_law(did):
                raise LoadError('LOAD-LAWDATE', '%s: its at reads %s, which takes its law date from its reader' % (
                    qid, did), mi.decl.loc)
        if mi.decl.at is not None:
            # L §4.3: "The at ends every chain, so it reads nothing law-dated itself (LOAD-LAWDATE)", L §8.3 `move <m>'s
            # at reads law-dated <x>`: an input or band the at reads directly (a def it reads is the check above). A
            # band whose every table governs always isn't law-dated (MC-X-F12); writer 9: snap12 left both to the
            # interpreter guard at the read
            from .node import walk as _nwalk
            for x in _nwalk(mi.decl.at):
                r = getattr(x, '_r', None)
                dated = None
                if r and r[0] == 'band' and self.banddated.get(r[1], True):
                    dated = r[1]
                elif r and len(r) > 2 and r[0] == 'read' and r[1] == 'input':
                    inf = m.inputs.get(r[2])
                    if inf is not None and not inf.observed and not inf.builtin and not inf.undated_read():
                        dated = r[2]
                if dated is not None:
                    raise LoadError('LOAD-LAWDATE', "move %s's at reads law-dated %s" % (qid, dated), mi.decl.loc)
            continue
        for item in (('move', qid), ('moveon', qid)):
            for t, rels in E.get(item, {}).items():
                if not (rels - skip):
                    continue
                dated = False
                if t[0] == 'band':
                    # a band whose every table governs always, and whose rows read nothing law-dated, isn't
                    # law-dated (L §4.3; MC-X-F12)
                    dated = self.banddated.get(t[1], True)
                elif t[0] == 'input':
                    inf = m.inputs.get(t[1])
                    dated = inf is not None and not inf.observed and not inf.builtin and not inf.undated_read()
                elif t[0] == 'def':
                    d = m.defs[t[1]]
                    dated = self.needs_reader_law(t[1]) or (d.at is None and self.lawdated.get(t[1]))
                if dated:
                    raise LoadError('LOAD-LAWDATE', '%s reads law-dated facts and states no at' % qid, mi.decl.loc)


def m_unit_of_quantity(self, n, t):
    """The type of unit(q), q a quantity (L §5.10 Kind and unit)."""
    m = self.m
    n._uq = True
    if t.k in ('dyn', 'hole', 'flex'):
        return Ty('unitkey')
    c = self.numcls(t)
    if c not in ('money', 'num'):
        raise uerr('unit of %s, which is neither a place nor a quantity' % T.show(t), n.loc)
    us = list(t.u or ())
    if len(us) != 1 or us[0][1] != 1:
        raise uerr('unit of %s: a plain number or a compound unit is a key of no unit kind' % T.show(t), n.loc)
    u = us[0][0]
    if u in CURRENCIES:
        return T.key('currencies')
    if u in m.kinds and m.kinds[u].spec == 'unit':
        # a leg declared with a unit kind: the key is read per value (§3.2)
        return T.key(u)
    if u in m.unit_kind_keys:
        return T.key(m.unit_kind_keys[u])
    raise uerr('unit of %s: %s is a quantity unit, a key of no unit kind' % (T.show(t), u), n.loc)


# -- what load sees (L §8.3 item 9, round 19, GD-4) -------------------------------------------------------------

FOLD_FNS = ('end_of', 'start_of', 'day_of', 'add_days', 'add_months', 'add_years', 'month_of', 'year_of',
            'quarter_of')


def m_seen_day(self, e, mi):
    """L §8.3 item 9: "A move's day and a leg's place are seen where they are written as literals or keys of a declared
    index, and where they are computed from these alone, folded through built-ins that read no fact (`on end_of(m)` for
    a key m of a declared index is seen)". The keys of an open index come from rows, which load doesn't see (item 5)."""
    m = self.m
    binds = dict((b.var, b) for b in mi.binds)
    for x in _nwalk(e):
        k = x.k
        if k in ('date', 'month', 'quarter'):
            continue
        if k == 'num':
            continue
        if k == 'name':
            b = binds.get(x.id)
            r = getattr(x, '_r', None)
            if b is None or r is None or r[0] != 'var':
                return False
            dom = getattr(b, 'dom', None)
            if not (isinstance(dom, tuple) and dom[0] == 'index' and dom[1] in m.indexes and
                    m.indexes[dom[1]].form != 'open'):
                return False
            continue
        if k == 'call' and x.name in FOLD_FNS and not getattr(x, 'prefix', None) and getattr(x, 'bkey', None) is None:
            continue
        if k in ('bin', 'neg') and getattr(x, 'op', '-') in ('+', '-'):
            continue
        return False
    return True


def m_check_seen_days(self):
    """L §5.14.1: a decision's plain bal(x, d) of its own day or a later one is refused, "LOAD-CYCLE where load sees the
    day (§8.3)"; L §8.3 item 9 (round 19): a day computed from literals and keys of a declared index through built-ins
    that read no fact is seen. Each such pair is folded at every key of the move's bindings."""
    m = self.m
    if not self.cycle_cands:
        return
    import itertools
    from .evaluate import Engine, Ctx
    eng = None
    for mi, x, dn in self.cycle_cands:
        on = mi.decl.on
        if not self.seen_day(on, mi) or not self.seen_day(dn, mi):
            continue
        doms = []
        for b in mi.binds:
            dom = getattr(b, 'dom', None)
            doms.append(m.indexes[dom[1]].keys if isinstance(dom, tuple) and dom[0] == 'index' else None)
        if any(d is None for d in doms):
            continue
        if eng is None:
            eng = Engine(m)
        ctx = Ctx(None, m.on)
        for keys in itertools.product(*doms):
            env = dict((b.var, k) for b, k in zip(mi.binds, keys))
            try:
                day = eng.ev(on, env, ctx)
                d = eng.ev(dn, env, ctx)
            except TallyError:
                continue
            if not (isinstance(day, datetime.date) and isinstance(d, datetime.date)):
                continue
            if d > day or (d == day and not x.start):
                kt = ', '.join(fmt(k) for k in keys)
                raise LoadError('LOAD-CYCLE', '%s%s on %s reads bal(%s, %s%s) = %s, %s: its own result or its future '
                                '(write the projection)' % (mi.id, '[%s]' % kt if kt else '', day, _txt(x.ref),
                                                            _txt(dn), ' start' if x.start else '', d,
                                                            'its own day' if d == day else 'a later day'),
                                mi.decl.loc)


def m_check_after_readings(self):
    """L §4.6 Readings (round 19, GD-10, C4-2): "A reading `after m` with m an observation or a plan is refused at load,
    which sees m's class (LOAD-ORDER): the batch has no order (§5.14.2, P-7). The refusal is by class, so it holds for
    an observation booked at a decision's point too"; L §8.3 `<x> after <m>: <m> is a plan or an observation`."""
    m = self.m
    for iid in ('stated', 'opening'):
        inf = m.inputs.get(iid)
        if inf is None:
            continue
        for r in inf.rows:
            ins = getattr(r, 'instant', None)
            if not (isinstance(ins, tuple) and ins and ins[0] == 'after'):
                continue
            node = ins[1]
            base = node.obj if node.k == 'index' else node
            if base.k != 'name':
                continue
            mv = m.moves.get(base.id)
            obs = base.id in m.inputs and m.inputs[base.id].observed
            if (mv is not None and not mv.decision) or obs:
                raise LoadError('LOAD-ORDER', '%s%s after %s: %s is a plan or an observation' % (
                    iid, r.keys_text(), _txt(node), _txt(node)), r.loc)


def m_check_proj_seen(self):
    """L §5.16 Decisions and projections: a projection a decision reaches, with neither a random clause nor a query
    around the read, that reads a random fact after its t "is refused: at load where load sees the projection reach the
    random fact after the decision's day (LOAD-RANDOM, §8.3), otherwise at the read (RUN-FUTURE)"; E §3.1 step 10;
    L §8.3 item 9 (round 19, GD-4): a move's day and a leg's place computed from literals and keys of a declared index
    through built-ins that read no fact are seen. Writer 10 builds the case load sees whole: a decision's own
    projection `bal(x, d) from t` (its day, t and d seen), a random movement booking x with its day seen inside [t, d]
    and no condition on its amount, and a random fact its amount reads whose law date is seen after t (the run's own
    test, eval2). Anything else stays the run's RUN-FUTURE; L §8.3 `<projection> reaches random <x> after <t> with no
    random clause or query`."""
    m = self.m
    rand = getattr(self, 'random_items', None) or set()
    rmoves = [m.moves[x[1]] for x in sorted(rand) if x[0] == 'move' and x[1] in m.moves]
    if not rmoves or not self.projs:
        return
    import itertools
    from .evaluate import Engine, Ctx
    inq = set()
    for qn, _item, _where in self.queries:
        for x in _nwalk(qn):
            inq.add(id(x))

    def keysets(mi):
        doms = []
        for b in mi.binds:
            dom = getattr(b, 'dom', None)
            if not (isinstance(dom, tuple) and dom[0] == 'index' and dom[1] in m.indexes and
                    m.indexes[dom[1]].form != 'open'):
                return None
            doms.append(m.indexes[dom[1]].keys)
        return [dict((b.var, k) for b, k in zip(mi.binds, ks)) for ks in itertools.product(*doms)]

    def rreads(rm, target):
        """The random fact reads of rm's legs that book target, or None where load doesn't see them whole."""
        out = []
        for ln in rm.decl.legs:
            if target not in (self.place_target(ln.frm), self.place_target(ln.to)):
                continue
            for x in _nwalk(ln.amount):
                if x.k == 'if':
                    return None
                r = getattr(x, '_r', None)
                if x.k not in ('index', 'name') or not (isinstance(r, tuple) and len(r) == 3 and
                                                        r[0] in ('read', 'item') and r[1] in ('def', 'input')):
                    continue
                info = (m.defs if r[1] == 'def' else m.inputs).get(r[2])
                ty = getattr(info, 'type', None)
                if getattr(ty, 'k', None) != 'dist' or getattr(x, '_distrole', False):
                    continue
                keys = list(x.keys or []) if x.k == 'index' else []
                if not all(self.seen_day(kx, rm) for kx in keys):
                    return None
                out.append((x, r[1], r[2], keys))
        return out

    eng = None
    for n, holder in self.projs:
        ct = getattr(n, '_ctgt', None)
        if not ct or ct[0] != 'proj' or id(n) in inq or holder[0] not in ('move', 'moveon'):
            continue
        mi = m.moves.get(holder[1])
        if mi is None or not mi.decision or mi.decl.when is not None or n.k != 'bal' or not n.args:
            continue
        target = ct[1]
        if target[0] != 'thing':
            continue
        frm, dn = n.proj.get('frm'), n.args[-1]
        if frm is None or not all(self.seen_day(e, mi) for e in (mi.decl.on, frm, dn)):
            continue
        mks = keysets(mi)
        if mks is None:
            continue
        for rm in rmoves:
            if rm is mi or rm.decl.when is not None or rm.decl.at is None or not self.seen_day(rm.decl.on, rm) or \
                    not self.seen_day(rm.decl.at, rm):
                continue
            rr = rreads(rm, target)
            rks = keysets(rm)
            if not rr or rks is None:
                continue
            if eng is None:
                eng = Engine(m)
            ctx = Ctx(None, m.on)
            for env in mks:
                try:
                    day, t, d = (eng.ev(e, env, ctx) for e in (mi.decl.on, frm, dn))
                except TallyError:
                    continue
                if not all(isinstance(v, datetime.date) for v in (day, t, d)) or not (m.S <= day <= m.E):
                    continue
                for renv in rks:
                    try:
                        rday = eng.ev(rm.decl.on, renv, ctx)
                        rlaw = eng.ev(rm.decl.at, renv, ctx)
                    except TallyError:
                        continue
                    if not (isinstance(rday, datetime.date) and isinstance(rlaw, datetime.date)) or \
                            not (t < rday <= d and m.S <= rday <= m.E):
                        continue
                    for x, what, iid, keys in rr:
                        law = rlaw
                        if what == 'def':
                            info = m.defs[iid]
                            at = info.at
                            if at is None:
                                continue
                            if at.k == 'law_expr':
                                if not self.seen_day(at.e, info):
                                    continue
                                try:
                                    kv = [eng.ev(kx, renv, ctx) for kx in keys]
                                    law = eng.ev(at.e, dict((b.var, k) for b, k in zip(info.binds, kv)),
                                                 ctx.but(law=rlaw))
                                except TallyError:
                                    continue
                            elif at.k != 'law_reader':
                                continue
                        if not isinstance(law, datetime.date) or law <= t:
                            continue
                        kt = ', '.join(fmt(k) for k in (env[b.var] for b in mi.binds))
                        rkt = ', '.join(fmt(eng.ev(kx, renv, ctx)) for kx in keys)
                        raise LoadError('LOAD-RANDOM', '%s%s on %s: %s reaches random %s%s after %s with no random '
                                        'clause or query' % (mi.id, '[%s]' % kt if kt else '', day, _txt(n), iid,
                                                             '[%s]' % rkt if rkt else '', t), n.loc)


class _NoBinds:
    binds = ()


def m_check_period_keys(self):
    """L §4.6 Reading at a date (GD-27, 2026-10-28): "A period no longer than the keys that no key's period holds (it
    lies across two keys, as a calendar month across the months of an anchored index) is refused at load where load
    sees the period and the keys (LOAD-KEY, §8.3), and otherwise at that value (RUN-DOMAIN)"; L §8.3 `<period> lies
    across keys <a> and <b> of <index>`. Load sees a period written as a literal, or folded from literals and keys of
    the reader's own bindings over declared indexes through built-ins that read no fact (L §8.3 item 9), and the keys
    of a declared index (item 5). A period computed otherwise (from a row, or a variable of an aggregate) is left to
    the value (evaluate, RUN-DOMAIN)."""
    m = self.m
    if not self.period_cands:
        return
    import itertools
    from .evaluate import Engine, Ctx
    from .dates import Period
    eng = None
    for ix, kn, holder in self.period_cands:
        if ix.form == 'open' or not ix.keys or not all(isinstance(k, Period) for k in ix.keys):
            continue
        if not isinstance(holder, tuple) or len(holder) < 2:
            continue
        if holder[0] == 'ask':
            info = _NoBinds()
        elif holder[0] == 'def':
            info = m.defs.get(holder[1])
        elif holder[0] in ('move', 'moveon'):
            info = m.moves.get(holder[1])
        else:
            continue
        if info is None or not self.seen_day(kn, info):
            continue
        doms = []
        for b in info.binds:
            dom = getattr(b, 'dom', None)
            if not (isinstance(dom, tuple) and dom[0] == 'index' and dom[1] in m.indexes and
                    m.indexes[dom[1]].form != 'open'):
                doms = None
                break
            doms.append(m.indexes[dom[1]].keys)
        if doms is None:
            continue
        if eng is None:
            eng = Engine(m)
        ctx = Ctx(None, m.on)
        for ks in itertools.product(*doms):
            env = dict((b.var, k) for b, k in zip(info.binds, ks))
            try:
                v = eng.ev(kn, env, ctx)
            except TallyError:
                continue
            if not isinstance(v, Period) or any(k == v or (k.start <= v.start and v.end <= k.end) for k in ix.keys):
                continue
            over = [k for k in ix.keys if k.start < v.end and v.start < k.end]
            if len(over) < 2 or all((k.end - k.start) < (v.end - v.start) for k in ix.keys):
                continue
            raise LoadError('LOAD-KEY', '%s lies across keys %s and %s of %s' % (fmt(v), fmt(over[0]), fmt(over[1]),
                                                                               ix.id), kn.loc)
