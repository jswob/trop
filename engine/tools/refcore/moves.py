"""Writer 3's plan pass (E §4.1, §5; L §5.14, §5.14.3, §5.19): move instances, their booking cuts and statuses,
observations and their links, balance readings, coverage, adjustments and their settlements.

Every movement of a timeline is one of the records below; the walk (walk.py) books them.  A timeline is keyed by
the context's cut, pins, axes, windows, wording, path, world and random clause (E §3.3); its instances are
memoised per key in the engine's run cache.

Installed as Engine methods (`m_<name>` -> `<name>`), as eval2 is.
"""
import datetime

from .dates import Period, ONE_DAY
from .errors import TallyError, Refusal, LoadError, Unspecified, NotYet, load_now, RandomLine
from .model import visible, row_known_sid, known_first, PlaceThing, _kv, key_text, in_world, _WSET
from .values import Num, Money, Hole, Thing, Mv, Iv, Rec, Text, NONE, NEVER, CURRENCIES, fmt, vkey


def install(E):
    orig = E.__init__

    def init(self, m):
        orig(self, m)
        self.run_cache = {}
        self.inst_active = set()
        self.walks = {}
        self.walk_warnings = []
    E.__init__ = init
    for name, fn in list(globals().items()):
        if name.startswith('m_'):
            setattr(E, name[2:], fn)


def refuse(code, msg):
    return Refusal(code, msg)


def merge(*vals):
    qs = set()
    for v in vals:
        if isinstance(v, Hole):
            qs |= v.qs
    return Hole(qs)


def _coll():
    from .evaluate import Coll
    return Coll()


# -- records ---------------------------------------------------------------------------------------------------

class Leg:
    __slots__ = ('name', 'frm', 'to', 'amount', 'unit', 'day', 'src')

    def __init__(self, name, frm, to, amount, unit, day, src=None):
        self.src = src        # (amount expression, env, context) of a hole amount: a partial ask line reads it again
        #                       for the holes it is linear in (L §5.18 Partial answers; mreads.m_partial_bal)
        self.name = name
        self.frm = frm        # Thing, or a Hole (a place nobody has told us, L §5.14)
        self.to = to
        self.amount = amount  # Num >= 0, a Hole, or a TallyError (a refused amount)
        self.unit = unit      # the unit's name, or None
        self.day = day        # date, Iv or Hole

    def rec(self):
        return Rec([('from', self.frm), ('to', self.to), ('amount', self.amount), ('day', self.day)])


class Movement:
    """What every movement shares (E §1): id, kind, day, legs, for, attributes, fulfils, status."""
    decision = False
    settles = None
    modelled = False

    def mv(self):
        if self._mv is None:
            self._mv = Mv(self.id, self)
        return self._mv


class Inst(Movement):
    """A move instance at one timeline (E §4.1)."""

    def __init__(self, mi, keys, env):
        self.move = mi
        self.keys = keys
        self.env = env
        self.id = '%s[%s]' % (mi.id, ', '.join(key_text(k) for k in keys)) if keys else mi.id
        self.kind = mi.kind
        self.day = None
        self.law = None
        self.cut = None
        self.forp = None
        self.exists = True
        self.legs = []
        self.attrs = {}
        self.fulfils = None
        self.err = None
        self.coll = None
        self.status = None
        self.obs = []
        self.adjs = []
        self.decision = mi.decision
        self.order = None
        self.ran = False
        self.modelled = False
        self.point = None
        self._mv = None
        self.day_coll = None

    def __repr__(self):
        return 'Inst(%s)' % self.id


def _obs_attr(v, o):
    """An observed row's attribute as a value: a hole names the row (L §3.1, §5.19); a ref to a movement (a key of an
    event kind, L §3.1, §5.7) is that movement, its record found at the reader's cut (mreads.mv_record). Writer 11."""
    if isinstance(v, tuple) and v:
        if v[0] == 'hole':
            return Hole([(v[1], o.sid)])
        if v[0] == 'mvref':
            return Mv(v[1])
    return v


class ObsMv(Movement):
    """An observation the cut sees (L §5.19)."""

    def __init__(self, o, legs):
        self.row = o
        self.id = o.sid
        self.kind = o.kind
        self.day = o.day
        self.legs = legs
        # a field that is a hole is read as that hole, naming the row; a ref as its movement (writer 11)
        self.attrs = {a: _obs_attr(v, o) for a, v in o.attrs.items()} if o.attrs else o.attrs
        self.fulfils = o.fulfils
        self.forp = None
        self.err = None
        self.status = 'observed'
        self.coll = None
        self.target = None
        self.exists = True
        self.order = (o.day, -1, o.inf.id, o.key)
        self._mv = None

    # its link's verdict (the instance it fulfils, or the refusal of its record) is judged when first read, the
    # target's links on their own (moves.obs_links_for; L §5.19)
    @property
    def target(self):
        self._judge_link()
        return self.__dict__.get('_target')

    @target.setter
    def target(self, v):
        self.__dict__['_target'] = v

    @property
    def err(self):
        self._judge_link()
        return self.__dict__.get('_err')

    @err.setter
    def err(self, v):
        self.__dict__['_err'] = v

    def _judge_link(self):
        j = self.__dict__.get('_judge')
        if j is None:
            return
        self.__dict__['_judge'] = None
        eng, tid, ctx = j
        try:
            eng.obs_links_for(tid, ctx)
        except TallyError as e:
            if isinstance(e, LoadError) or load_now(e):
                raise
            self.__dict__['_judge'] = j


class AdjMv(Movement):
    """An adjustment (E §5 step 4): moves nothing."""

    def __init__(self, inst, leg, delta, day, vids):
        self.inst = inst
        self.leg = leg
        self.delta = delta
        self.day = day
        self.vids = vids
        # L §5.14.3 (round 12b): "Its name is adjustments[m[keys].leg, V], the adjusted leg and the version"
        # a step of several versions makes one adjustment, which each of them names, printed under the first in
        # ASCII order (E §5 step 4)
        self.id = 'adjustments[%s.%s, %s]' % (inst.id, leg.name, sorted(vids)[0] if vids else '')
        self.made = False      # this step made the instance exist (E §5 step 1: its places are this version's)
        self.kind = 'adjustments'
        self.legs = []
        self.fulfils = None
        self.forp = None
        self.status = 'adjustment'
        self.settled = []
        self.waived = False
        self.err = None
        self.exists = True
        self.coll = None
        self.order = (day, -1, inst.move.id, inst.id, leg.name)
        self._mv = None
        self.attrs = {'adjusts': inst.mv(), 'change': delta}

    def remainder(self):
        """δ less the settlements; None once waived (E §5 step 5)."""
        if self.waived:
            return None
        r = self.delta
        for s, sign, amt in self.settled:
            if isinstance(amt, Hole) or isinstance(r, Hole):
                r = merge(r, amt)
                continue
            if isinstance(amt, TallyError):
                raise amt
            r = Num(r.q - sign * amt.q, r.u, money=r.money)
        return r


class PadMv(Movement):
    """A pad (E §6.1): takes up a statement's gap."""

    def __init__(self, vid, place, itext, amount, frm, to, day, unit):
        self.id = 'pads:%s:%s@%s' % (vid, place, itext)
        self.kind = 'pads'
        self.day = day
        self.legs = [Leg('cash', frm, to, amount, unit, day)]
        self.attrs = {}
        self.fulfils = None
        self.forp = None
        self.status = 'pad'
        self.err = None
        self.exists = True
        self.coll = None
        self.order = (day, 9, 'pads', self.id)
        self._mv = None


# -- small helpers -----------------------------------------------------------------------------------------------

def unit_of_value(v):
    if isinstance(v, Num) and v.u and len(v.u) == 1 and v.u[0][1] == 1:
        return v.u[0][0]
    return None


def qty(q, u):
    if u in CURRENCIES:
        return Money(q, u)
    return Num(q, ((u, 1),))


def day_lo(d):
    if isinstance(d, datetime.date):
        return d
    if isinstance(d, Iv):
        return d.lo
    return None


def day_hi(d):
    if isinstance(d, datetime.date):
        return d
    if isinstance(d, Iv):
        return d.hi
    return None


def m_wkey(self, ctx):
    """The key of the timeline a context reads (E §3.3): its cut, pins, axes, windows, wording, path, world and
    random clause.  `with uncertain` doesn't change the timeline: no decision sees it (L §5.14)."""
    return (ctx.cut, ctx.pins, ctx.axes, ctx.windows, ctx.wording, ctx.path, ctx.world, ctx.rnd)


def m_tctx(self, ctx):
    return ctx.but(law=None, partial=False, walk=None, wp=None, unc=False)


def m_kind_certainty(self, kid):
    m = self.m
    x = m.kinds[kid]
    while x.certainty is None and x.parent is not None:
        x = m.kinds[x.parent]
    return x.certainty


def m_key_sort(self, k, b):
    m = self.m
    d = getattr(b, 'mdom', None) or b.dom
    try:
        if d[0] == 'index':
            return (0, m.indexes[d[1]].pos(k) or 0, '')
        if d[0] == 'kind' and isinstance(k, Thing) and k.name in m.things:
            st = m.kind_order_key(m.things[k.name], d[1], self.ask_K)
            return (0, 0, st[0])
        if isinstance(k, Mv) and k.src is not None:
            o = getattr(k.src, 'order', None)
            dd = o[0] if o else None
            return (0, 0, '%s %s' % (dd.isoformat() if isinstance(dd, datetime.date) else '', k.id))
    except Exception:
        pass
    return (1, 0, key_text(k))


def m_inst_order(self, inst):
    d = day_lo(inst.day) or datetime.date.max
    ks = tuple(self.key_sort(k, b) for k, b in zip(inst.keys, inst.move.binds))
    return (d, inst.move.stratum, inst.move.id, ks)


# -- worlds, coverage, observations, readings (L §5.19, §6.4) ----------------------------------------------------------

def m_world_ok(self, version, ctx):
    """A scenario's version is seen in that scenario and those over it (L §6.4)."""
    return in_world(version)


def item_in_world(item):
    """An item declared in a scenario file exists only in that scenario and those over it (L §6.4)."""
    decl = getattr(item, 'decl', None)
    sc = getattr(decl, '_scen', None) if decl is not None else None
    return sc is None or sc in _WSET[0]


def m_coverage(self, ctx):
    """Place name -> [(span, version)] of the versions the cut sees that cover it (L §5.19)."""
    key = ('cov', self.wkey(ctx))
    hit = self.run_cache.get(key)
    if hit is not None:
        return hit
    out = {}
    und = []
    for vid in sorted(self.m.versions):
        v = self.m.versions[vid]
        if not getattr(v, 'covers', None):
            continue
        if not self.world_ok(v, ctx):
            continue
        vis = visible(v.known, ctx.cut, v.id)
        if vis is None:
            # L §4.5 (round 12): a version known in a window that holds the cut, with no windows clause: whether it
            # covers its places is undecided, so each covered place's balance is a hole from the span's first day
            # (HOLE-IMPRECISE)
            lo, hi = v.known
            q = ('date within window(%s, %s) not precise enough for the cut %s' % (lo, hi, ctx.cut), vid)
            for th, sp in v.covers:
                span = sp if sp is not None else v.span_at(ctx.cut)
                if span is not None:
                    und.append((th.name, span, q))
            continue
        if not vis:
            continue
        for th, sp in v.covers:
            span = sp if sp is not None else v.span_at(ctx.cut)
            if span is None:
                continue
            out.setdefault(th.name, []).append((span, v))
    self.run_cache[key] = out
    self.run_cache[('covund',) + key[1:]] = und
    return out


def m_coverage_undecided(self, ctx):
    """[(place, span, question)] of the covers of versions known in a window that holds the cut (L §4.5)."""
    self.coverage(ctx)
    return self.run_cache.get(('covund', self.wkey(ctx)), [])


def m_covering(self, place, day, ctx):
    return [v for span, v in self.coverage(ctx).get(place, ()) if span.contains(day)]


def m_covered_on(self, place, day, ctx):
    return bool(self.covering(place, day, ctx))


def m_observations(self, ctx):
    from .loader import row_layer
    """The observations the cut sees (L §5.19): each row id's latest restatement; `none` removes it."""
    key = ('obs', self.wkey(ctx))
    hit = self.run_cache.get(key)
    if hit is not None:
        return hit
    m = self.m
    out = []
    for qid in sorted(m.inputs):
        inf = m.inputs[qid]
        if not inf.observed:
            continue
        by = {}
        undecided = {}
        for o in getattr(inf, 'obs', []):
            if not self.world_ok(o.version, ctx):
                continue
            vis = visible(o.known, ctx.cut, o.version.id if getattr(o, 'own_known', None) is None else o.sid)
            if vis is None:
                # L §4.5 (round 12): known in a window that holds the cut, with no windows clause: "This holds for
                # everything the version or row brings: ... its observations" (HOLE-IMPRECISE): it is booked or not,
                # so each of its legs is a hole carrying that question
                lo, hi = o.known
                undecided[id(o)] = Hole([('date within window(%s, %s) not precise enough for the cut %s' % (
                    lo, hi, ctx.cut), o.sid)])
            elif not vis:
                continue
            cur = by.get(o.key)
            # the latest restatement; a scenario's stands above its base's whatever their known dates (L §4.6 step 2)
            if cur is None or (row_layer(m, o), known_first(o.known)) > (row_layer(m, cur), known_first(cur.known)):
                by[o.key] = o
        for k in sorted(by):
            o = by[k]
            if o.removed:
                continue
            hq = undecided.get(id(o))
            legs = [Leg(n, f, t, hq if hq is not None else a, u, d or o.day) for n, f, t, a, u, d in o.legs]
            ob = ObsMv(o, legs)
            ob.coll = _coll()
            ob.coll.rows[o.sid] = o
            if ob.fulfils is not None and ob.fulfils[0] == 'inst':
                # its link is judged when its target or its record's refusal is first read (obs_links_for)
                _, mid, kv = ob.fulfils
                ob.__dict__['_judge'] = (self, '%s[%s]' % (mid, ', '.join(key_text(k) for k in kv)) if kv else mid,
                                         ctx)
            out.append(ob)
    out.sort(key=lambda x: (x.day, x.id))
    self.run_cache[key] = out
    return out


def instant_text(r):
    ins = r.instant
    if ins[0] == 'after':
        from .check import _txt
        return 'after %s' % _txt(ins[1])
    return '%s %s' % (ins[0].isoformat(), ins[1])


def m_readings(self, ctx):
    """The balance readings the cut sees: (input id, place, unit, row), each restated reading replaced (L §5.19,
    §4.6)."""
    key = ('rd', self.wkey(ctx))
    hit = self.run_cache.get(key)
    if hit is not None:
        return hit
    m = self.m
    keep = {}
    for iid in ('opening', 'stated'):
        for r in m.inputs[iid].rows:
            if not self.world_ok(r.version, ctx):
                continue
            vis = visible(r.known, ctx.cut, row_known_sid(r))
            if vis is None:
                # L §4.5 (round 12): a reading known in a window that holds the cut, with no windows clause: its
                # value is HOLE-IMPRECISE (it is stated or not)
                import copy
                lo, hi = r.known
                r = copy.copy(r)
                r.val = ('hole', 'date within window(%s, %s) not precise enough for the cut %s' % (lo, hi, ctx.cut))
            elif not vis:
                continue
            place = r.pattern[0][1].name
            u = unit_of_value(r.val) if isinstance(r.val, Num) else m.place_unit(m.things[place])
            k = (iid, place, instant_text(r), u, row_owner(m, r))
            cur = keep.get(k)
            if cur is None or known_first(r.known) > known_first(cur.known):
                keep[k] = r
    # a place stated same in a joint model: its readings from the members are read together, a joint model's own
    # row settles them, and two that disagree are a hole naming both rows (L §6.1, HOLE-SAME)
    byk = {}
    for (iid, place, it, u, o), r in keep.items():
        byk.setdefault((iid, place, it, u), []).append((o, r))
    res = []
    for (iid, place, it, u), ors in sorted(byk.items(), key=lambda x: (x[0][1], x[0][2], x[0][0], str(x[0][3]))):
        if len(ors) == 1:
            res.append((iid, place, u, ors[0][1]))
            continue
        own = [r for o, r in ors if o == '']
        if own:
            res.append((iid, place, u, own[0]))
            continue
        ors.sort(key=lambda x: x[0])
        r0 = ors[0][1]
        if all(vkey(r.val) == vkey(r0.val) for o, r in ors):
            res.append((iid, place, u, r0))
            continue
        res.append((iid, place, u, same_hole_row(m, iid, place, it, [r for o, r in ors])))
    self.run_cache[key] = res
    return res


def row_owner(m, r):
    """The member a row comes from in a joint model ('' for the joint model's own, or any row elsewhere)."""
    if getattr(m, 'joint', None) is None or r.version is None:
        return ''
    from .joint import owner_of
    return owner_of(r.version.id)


def same_hole_row(m, iid, place, it, rows):
    """A reading standing for two rows of one place, stated same, that disagree: its value is the hole of L §8.1's
    HOLE-SAME, `<p> and <q> are one, but <x> differs on <d>: <a> (<row>) and <b> (<row>)`."""
    import copy
    from .joint import shown
    J = m.joint
    names = [n for (o, n, out) in J.groups.get(place, ())]
    a, b = rows[0], rows[1]
    q = '%s and %s are one, but %s differs on %s: %s (%s) and %s (%s)' % (
        names[0] if names else shown(place), names[1] if len(names) > 1 else shown(place), iid, it,
        fmt(a.val) if not isinstance(a.val, tuple) else '?', shown(a.sid or ''),
        fmt(b.val) if not isinstance(b.val, tuple) else '?', shown(b.sid or ''))
    r = copy.copy(a)
    r.val = ('hole', q)
    return r


# -- move instances (E §4.1, §5 steps 0-1) -------------------------------------------------------------------------

def m_move_law(self, mi, env, ctx):
    """A move's law date `at e` (L §4.3) with the keys bound so far; None when there is none yet."""
    at = mi.decl.at
    if at is None:
        return None
    from .node import walk as nwalk
    for x in nwalk(at):
        r = getattr(x, '_r', None)
        if x.k == 'name' and r and r[0] == 'var' and r[1] not in env:
            return None
    v = self.ev(at, env, ctx.but(law=None))
    if isinstance(v, Iv) and isinstance(v.lo, datetime.date):
        v = self.window_choice(v, ctx)
    if isinstance(v, Hole):
        return v
    if v is NONE:
        raise refuse('RUN-NONE', 'the law date of %s is none here' % mi.id)
    if isinstance(v, Iv):
        # a window `at`: the instance is evaluated once per law-date class the window overlaps, as a def's (L §5.14
        # at e, §4.3); every day of the window gives its class's values
        return v
    return self.date_of(v, 'the law date of %s' % mi.id)


def law_days(law):
    """The law dates to evaluate an instance at: one date, or every day of a window `at` (L §5.14, §4.3)."""
    if isinstance(law, Iv):
        if law.days:
            return sorted(law.days)
        return [law.lo + datetime.timedelta(days=i) for i in range((law.hi - law.lo).days + 1)]
    return [law]


def imprecise_law(law, what):
    return Hole([('date within %s not precise enough for the law date of %s' % (law.window_text(), what), what)])


def m_move_tuples(self, mi, ctx):
    """The binding tuples of a move at the cut (E §4.1 step 1): (env, keys, status, coll), status True, a Hole (an
    unknown member) or ('domainhole', Hole); coll what deciding that tuple read."""
    out = []

    def go(i, env, keys, st, coll):
        if i == len(mi.binds):
            out.append((env, keys, st, coll))
            return
        b = mi.binds[i].node
        cd = _coll()
        cd.merge(coll)
        self.stack.append(cd)
        try:
            law = self.move_law(mi, env, ctx)
            if isinstance(law, Hole):
                out.append((env, keys, ('domainhole', law), cd))
                return
            lds = law_days(law)
            bctx = ctx.but(law=lds[0])
            res = self.bind_each(b, env, bctx)
            pc = getattr(b, '_colls', None)
            b._colls = None
            if len(lds) > 1 and not isinstance(res, Hole):
                k0 = [(vkey(v), vkey_ex(st) if not isinstance(st, TallyError) else 'err') for v, st in res]
                for dd in lds[1:]:
                    r2 = self.bind_each(b, env, ctx.but(law=dd))
                    b._colls = None
                    k2 = None if isinstance(r2, Hole) else [(vkey(v), vkey_ex(st) if not isinstance(st, TallyError)
                                                             else 'err') for v, st in r2]
                    if k2 != k0:
                        res = imprecise_law(law, mi.id)
                        break
        finally:
            self.stack.pop()
        if isinstance(res, Hole):
            out.append((env, keys, ('domainhole', res), cd))
            return
        for v, s in res:
            e2 = dict(env)
            e2[b.var] = v
            if isinstance(s, TallyError):
                out.append((e2, keys + [v] + [None] * (len(mi.binds) - i - 1), ('refused', s), cd))
                continue
            s2 = st
            if isinstance(s, Hole):
                s2 = s if st is True else merge(st, s)
            cw = _coll()
            cw.merge(cd)
            if pc:
                mc = pc.get(v.id if isinstance(v, Mv) else getattr(v, 'name', None))
                if mc is not None:
                    cw.merge(mc)
            if b.where is not None:
                self.stack.append(cw)
                try:
                    w = self.ev(b.where, e2, bctx)
                finally:
                    self.stack.pop()
                if w is False:
                    continue
                if w is NONE:
                    raise refuse('RUN-NONE', 'a where is none here')
                if isinstance(w, Hole):
                    s2 = w if s2 is True else merge(s2, w)
            go(i + 1, e2, keys + [v], s2, cw)
    go(0, {}, [], True, _coll())
    return out


def m_upto_tag(self):
    """The cache tag of a walk up to a day (mreads.m_prefix_walk): () for the whole timeline."""
    u = getattr(self, '_body_upto', None)
    return () if u is None else (('upto', u),)


def m_plan_list(self, mi, ctx):
    """A plan move's instances for the walk being built: all of them, or, for a walk up to a day, those dated on or
    before it (the others' bodies unevaluated; a move whose legs state their own days is taken whole)."""
    upto = getattr(self, '_body_upto', None)
    tctx = self.tctx(ctx)
    full = self.run_cache.get(('inst', mi.id, self.wkey(tctx)))
    if upto is None or full is not None or any(getattr(ln, 'on', None) is not None for ln, d in mi.legs):
        return self.move_instances(mi, ctx)
    key = ('inst-upto', mi.id, self.wkey(tctx), upto)
    hit = self.run_cache.get(key)
    if hit is not None:
        return hit
    if key in self.inst_active:
        raise refuse('RUN-CYCLE', '%s reads its own instances' % mi.id)
    self.inst_active.add(key)
    try:
        out = self.insts_of_tuples(mi, self.move_tuples(mi, tctx), tctx, upto=upto)
    finally:
        self.inst_active.discard(key)
    self.run_cache[key] = out
    return out


def m_move_instances(self, mi, ctx):
    """Every instance of a plan move in the timeline, dated at cut K and evaluated at its booking cut (E §4.1)."""
    tctx = self.tctx(ctx)
    key = ('inst', mi.id, self.wkey(tctx))
    hit = self.run_cache.get(key)
    if hit is not None:
        return hit
    if key in self.inst_active:
        raise refuse('RUN-CYCLE', '%s reads its own instances' % mi.id)
    self.inst_active.add(key)
    try:
        tuples = self.move_tuples(mi, tctx)
        out = self.insts_of_tuples(mi, tuples, tctx)
        if tuples and tctx.cut == self.m.on and not mi.decision:
            self.check_schedule(mi, out)
    finally:
        self.inst_active.discard(key)
    self.run_cache[key] = out
    return out


def m_insts_of_tuples(self, mi, tuples, tctx, upto=None):
    if True:
        out = []
        for env, keys, st, c in tuples:
            if isinstance(st, tuple) and st[0] == 'refused':
                if None in keys:
                    raise st[1]
                inst = Inst(mi, keys, env)
                inst.err = st[1]
                inst.coll = _coll()
            elif isinstance(st, tuple):
                inst = Inst(mi, keys, env)
                inst.day = st[1]
                inst.exists = st[1]
                inst.coll = _coll()
            else:
                inst = self.eval_inst(mi, env, keys, tctx, exists0=st, upto=upto)
                if inst is None:
                    continue
                if not mi.decision and inst.cut is not None and inst.err is None:
                    self.inv5(inst, tctx)
            inst.coll.merge(c)
            out.append(inst)
        for inst in out:
            inst.order = self.inst_order(inst)
        out.sort(key=lambda i: i.order)
    return out


def m_settle_candidates(self, mi, adjs, ctx):
    """The instances of a settlement move that may settle `adjs` (E §5 step 5). When its `fulfils` is the variable
    of its one binding over adjustments (`move wv[a in adjustments ...]` ... `fulfils a`), an instance fulfils the
    adjustment it is bound to, so only the instances bound to `adjs` are made: a plan that reads `flows` of another
    plan then reads that plan's settlements without the other moves' adjustments (E §4.1: "Plans read other plans
    through flows(m[keys], span) ... The instance graph is acyclic"; L §5.14.3: flows counts the booked amount plus
    the settlements dated in its span)."""
    tctx = self.tctx(ctx)
    full = self.run_cache.get(('inst', mi.id, self.wkey(tctx)))
    if full is not None:
        return full
    m = self.m
    f = mi.decl.fulfils
    ab = [b for b in mi.binds if getattr(b, 'mdom', None) and b.mdom[0] == 'event' and
          'adjustments' in m.kinds[b.mdom[1]].subtree(m)]
    if f is None or f.k != 'name' or len(ab) != 1 or f.id != ab[0].node.var or \
            self.run_cache.get(('adjs', self.wkey(ctx))) is not None:
        return self.move_instances(mi, ctx)
    key = ('instr', mi.id, self.wkey(tctx), tuple(sorted(a.id for a in adjs)))
    hit = self.run_cache.get(key)
    if hit is not None:
        return hit
    if key in self.inst_active:
        raise refuse('RUN-CYCLE', '%s reads its own instances' % mi.id)
    self.inst_active.add(key)
    prev = getattr(self, '_adj_restrict', None)
    self._adj_restrict = list(adjs)
    try:
        out = self.insts_of_tuples(mi, self.move_tuples(mi, tctx), tctx)
    finally:
        self._adj_restrict = prev
        self.inst_active.discard(key)
    self.run_cache[key] = out
    return out


def m_check_schedule(self, mi, insts):
    """LOAD-SCHEDULE (L §4.2): a move whose day is a schedule that gives no date in the run, at any of its keys, as
    the reality is known at its `on`.  A hole or refused day might give one: not refused then."""
    from .eval2 import strip
    on = strip(mi.decl.on)
    if not (on.k == 'call' and on.name == 'schedule'):
        return
    # L §4.2 (round 19, C4-5): "A schedule that gives no date in the run is refused at load where load sees it
    # (LOAD-SCHEDULE, §8.3); where it doesn't, each period whose day doesn't exist gives none". Load sees p and e
    # where they are literals or keys of a declared index folded through built-ins that read no fact (L §8.3 item 9);
    # the calendar is a row, so a rolled day is never seen: refused only where no period has its day at all
    ch = getattr(self.m, 'checker', None)
    if ch is None or len(on.args) < 2 or not (ch.seen_day(on.args[0], mi) and ch.seen_day(on.args[1], mi)):
        return
    S, E = self.m.S, self.m.E
    for i in insts:
        d = i.day
        if isinstance(d, datetime.date) or i.err is not None or isinstance(d, Hole):
            return
    from .check import _txt
    raise LoadError('LOAD-SCHEDULE', 'schedule %s (the day of %s) gives no date in the run' % (_txt(on), mi.id),
                    mi.decl.loc)


def m_inv5(self, inst, tctx):
    """E §9 invariant 5: a plan instance's A0 is the same under every K class that books it at the same cut.  Kept
    across the asks of a run, per timeline apart from its cut."""
    reg = self.m.__dict__.setdefault('_inv5', {})
    wk = self.wkey(tctx)
    k = (inst.id, inst.cut, tuple(repr(q.key()) for q in wk[1])) + tuple(repr(x) for x in wk[2:])
    v = body_key((inst.exists, inst.legs, None, None, None))
    old = reg.get(k)
    if old is None:
        reg[k] = (v, tctx.cut)
    elif old[0] != v:
        raise refuse('ENGINE-INVARIANT', 'a booked plan amount changed: %s booked at %s differs between the cuts %s '
                                         'and %s (E §9 invariant 5)' % (inst.id, inst.cut, old[1], tctx.cut))


def m_eval_inst(self, mi, env, keys, tctx, exists0=True, at_cut=None, upto=None):
    """One instance: its day at cut K, then its when, legs and attributes at min(t, K) (E §5 steps 0-1).  None when
    its day is never.  A decision's body is evaluated in the walk."""
    inst = Inst(mi, keys, env)
    inst.coll = _coll()
    self.stack.append(inst.coll)
    try:
        try:
            K = tctx.cut
            dc = _coll()
            self.stack.append(dc)
            law = None
            rline = None
            try:
                law = self.move_law(mi, env, tctx)
                inst.law = law
                if not isinstance(law, Hole):
                    lds = law_days(law)
                    day = self.ev(mi.decl.on, env, tctx.but(law=lds[0]))
                    for dd in lds[1:]:
                        d2 = self.ev(mi.decl.on, env, tctx.but(law=dd))
                        if vkey_ex(d2) != vkey_ex(day):
                            # the day differs between the window's law dates: the instance's day is a hole, its
                            # legs evaluated as for any hole day
                            day = imprecise_law(law, inst.id)
                            break
            except RandomLine as e:
                if mi.decision or tctx.path is not None:
                    raise
                rline = e
            finally:
                self.stack.pop()
                inst.coll.merge(dc)
                inst.day_coll = dc
            if rline is not None:
                # a plan whose day reaches a sample, outside a path (E §7 step 1: "Cells that no random fact reaches
                # are computed once and shared by every path"): its day is the path's, so here the instance is
                # refused by the random line, but its legs' places are read as for a hole day, so that only the
                # balances of the places it books meet the refusal (walk.sched_refused), and a line that reads
                # another place, which no random fact reaches, is answered (W8-SEEN)
                if law is not None and not isinstance(law, Hole):
                    try:
                        b = self.body_over_law(mi, env, inst, tctx.but(cut=K), law, exists0)
                        inst.exists, inst.legs, inst.attrs, inst.forp, inst.fulfils = b
                    except TallyError as e2:
                        if isinstance(e2, LoadError) or load_now(e2):
                            raise
                        inst.legs = []
                inst.err = rline
                return inst
            if isinstance(law, Hole):
                inst.day = law
                inst.exists = law
                return inst
            if day is NEVER or day is NONE:
                # "a move whose day is none has no instance at that key" (L §4.2 schedule)
                return None
            if isinstance(day, Period) and day.kind == 'day':
                day = day.start
            if isinstance(day, Iv):
                inst.wlo = day.lo
                day = self.window_choice(day, tctx)
            elif tctx.windows is not None and getattr(mi.decl, '_scen', None) is not None and \
                    isinstance(day, datetime.date):
                # a scenario's move dated by a window that the line's windows clause chose a day of: "an instance
                # whose day, a window's first day, or a leg's own day falls before D is left out" (L §6.4) reads the
                # window's first day, whichever day the clause takes
                c0 = _coll()
                self.stack.append(c0)
                try:
                    d0 = self.ev(mi.decl.on, env, tctx.but(law=law_days(law)[0], windows=None))
                except Exception:  # a refused or hole day: no window to read
                    d0 = None
                finally:
                    self.stack.pop()
                if isinstance(d0, Iv):
                    inst.wlo = d0.lo
            inst.day = day
            inst.exists = exists0
            if mi.decision:
                return inst
            first = day.lo if isinstance(day, Iv) else day
            if upto is not None and isinstance(first, datetime.date) and first > upto:
                # a walk up to a day books nothing after it (mreads.m_prefix_walk); a movement dated by a window
                # falls after it when the window's first day does (L §4.4; MC-X-F11: its body was evaluated, and an
                # amount that reads the cell being evaluated made the answer depend on the order of the terms)
                return None
            if at_cut is not None:
                cuts = [at_cut]
            elif isinstance(day, datetime.date):
                cuts = [min(day, K)]
            elif isinstance(day, Iv):
                cuts = sorted(set([min(day.lo, K), min(day.hi, K)]))
            else:
                cuts = [K]
            inst.cut = cuts[0]
            if not self.keys_seen(mi, keys, cuts[-1]):
                # a key entered after the booking cut: the instance didn't exist then, A0 = 0 (E §5 steps 1-3)
                inst.exists = False
                return inst
            bodies = [self.body_over_law(mi, env, inst, tctx.but(cut=c), law, exists0) for c in cuts]
            b0 = bodies[0]
            if len(bodies) > 1 and body_key(bodies[0]) != body_key(bodies[1]):
                # a version known inside a windowed instance's window (L §5.14.3)
                hq = Hole([('date within %s not precise enough for the booking cut of %s' % (day.window_text(),
                                                                                             inst.id), inst.id)])
                b1 = bodies[1]
                src = b0[1] if b0[1] else b1[1]
                legs = [Leg(lg.name, lg.frm, lg.to, hq, lg.unit, lg.day) for lg in src]
                ex = b0[0] if (b0[0] is b1[0] or vkey_ex(b0[0]) == vkey_ex(b1[0])) else hq
                b0 = (ex, legs, b0[2], b0[3], b0[4])
            inst.exists, inst.legs, inst.attrs, inst.forp, inst.fulfils = b0
        except TallyError as e:
            if isinstance(e, LoadError) or load_now(e):
                raise
            inst.err = e
    finally:
        self.stack.pop()
    return inst


def vkey_ex(x):
    return x if isinstance(x, bool) else vkey(x)


def body_key(b):
    ex, legs, attrs, forp, ful = b
    return (vkey_ex(ex),
            tuple((lg.name, vkey(lg.frm), vkey(lg.to), vkey(lg.amount) if not isinstance(lg.amount, TallyError)
                   else ('err', lg.amount.code, lg.amount.msg), vkey(lg.day) if lg.day is not None else None)
                  for lg in legs))


def m_body_over_law(self, mi, env, inst, c0, law, exists0=True):
    """An instance's body at its law date; for a window `at`, at each law date of the window: one value where all
    agree, otherwise HOLE-IMPRECISE (L §5.14 at e)."""
    lds = law_days(law)
    b0 = self.inst_body(mi, env, inst, c0.but(law=lds[0]), exists0)
    for dd in lds[1:]:
        b1 = self.inst_body(mi, env, inst, c0.but(law=dd), exists0)
        if body_key(b1) != body_key(b0):
            hq = imprecise_law(law, inst.id)
            src = b0[1] if b0[1] else b1[1]
            legs = [Leg(lg.name, lg.frm, lg.to, hq, lg.unit, lg.day) for lg in src]
            ex = b0[0] if vkey_ex(b0[0]) == vkey_ex(b1[0]) else hq
            return (ex, legs, b0[2], b0[3], b0[4])
    return b0


def m_inst_body(self, mi, env, inst, c1, exists0=True):
    """(exists, legs, attrs, for, fulfils) of an instance at the context's cut."""
    ex = exists0
    if mi.decl.when is not None:
        w = self.ev(mi.decl.when, env, c1)
        if w is NONE:
            raise refuse('RUN-NONE', 'the when of %s is none here' % inst.id)
        if w is False and isinstance(inst.day, Hole):
            # L §5.14 on d (TX2-F1, writer 10): "A day that is a hole and not a window has no bound, so that is every
            # span, earlier ones included, whatever the instance's `when` says: its booking cut min(t, K) is a hole
            # with it"; E §4.2 "every day, for a day that is a hole and not a window": the instance's existence is
            # the day's hole, and its legs are read as for any hole day
            ex = inst.day if ex is True else merge(ex, inst.day)
        elif w is False:
            return (False, [], {}, None, None)
        if isinstance(w, Hole):
            ex = w if ex is True else merge(ex, w)
    forp = self.ev(mi.decl.forp, env, c1) if mi.decl.forp is not None else None
    legs = [self.eval_leg(inst, ln, d, env, c1, inst.day) for ln, d in mi.legs]
    attrs = {}
    for a, e in mi.decl.attrs:
        attrs[a] = self.ev(e, env, c1)
    ful = self.ev(mi.decl.fulfils, env, c1) if mi.decl.fulfils is not None else None
    return (ex, legs, attrs, forp, ful)


def m_eval_leg(self, inst, ln, d, env, ctx, day):
    m = self.m
    frm = self.ev(ln.frm, env, ctx)
    to = self.ev(ln.to, env, ctx)
    for side, p in (('from', frm), ('to', to)):
        if p is NONE:
            raise refuse('RUN-NONE', 'the %s place of leg %s of %s is none here' % (side, ln.name, inst.id))
        if isinstance(p, Hole):
            continue
        if not isinstance(p, Thing) or p.name not in m.things or not m.is_place_thing(m.things[p.name]):
            raise refuse('RUN-DOMAIN', 'the %s of leg %s of %s is %s, not a place' % (side, ln.name, inst.id,
                                                                                    fmt(p)))
        if m.things[p.name].base in m.kinds['assets'].subtree(m):
            raise LoadError('LOAD-KIND', 'a leg to or from an asset (%s of %s)' % (ln.name, inst.id))
    lday = day
    if ln.on is not None:
        lday = self.ev(ln.on, env, ctx)
        if isinstance(lday, Iv):
            lday = self.window_choice(lday, ctx)
        if lday is NONE:
            raise refuse('RUN-NONE', 'the day of leg %s of %s is none here' % (ln.name, inst.id))
    unit = None
    try:
        amt = self.ev(ln.amount, env, ctx)
        if isinstance(amt, Hole):
            unit = self.leg_unit_static(d, frm, to)
        elif amt is NONE:
            raise refuse('RUN-NONE', 'the amount of leg %s of %s is none here' % (ln.name, inst.id))
        elif isinstance(amt, Num):
            amt, unit = self.leg_amount(inst, ln, d, amt, frm, to)
        elif isinstance(amt, Iv):
            raise self.imprecise(amt, 'the amount of leg %s of %s' % (ln.name, inst.id))
        else:
            raise refuse('RUN-DOMAIN', 'the amount of leg %s of %s is %s' % (ln.name, inst.id, fmt(amt)))
    except TallyError as e:
        if load_now(e):
            raise
        amt = e
    return Leg(ln.name, frm, to, amt, unit, lday, (ln.amount, env, ctx) if isinstance(amt, Hole) else None)


def m_leg_unit_static(self, d, frm, to):
    m = self.m
    if d.unit in CURRENCIES or d.unit in m.units:
        return d.unit
    for p in (frm, to):
        if isinstance(p, Thing):
            pu = m.place_unit(m.things[p.name])
            if pu not in (None, 'any'):
                return pu
    return None


def m_leg_amount(self, inst, ln, d, amt, frm, to):
    """A leg's amount in its unit: places of another unit are LOAD-UNIT (L §5.10), a negative amount RUN-NEG."""
    m = self.m
    units = set()
    for p in (frm, to):
        if isinstance(p, Thing):
            pu = m.place_unit(m.things[p.name])
            if pu not in (None, 'any'):
                units.add(pu)
    au = unit_of_value(amt)
    if d.unit in CURRENCIES or d.unit in m.units:
        u = d.unit
    else:
        u = au
        if u is None and amt.q == 0 and len(units) == 1:
            u = list(units)[0]
        if u is None:
            raise LoadError('LOAD-UNIT', 'the amount of leg %s of %s (%s) is in no unit of a place' % (
                ln.name, inst.id, fmt(amt)))
        if d.unit != 'any' and d.unit in m.kinds and m.unit_kind_keys.get(u) != d.unit and \
                not (d.unit == 'currencies' and u in CURRENCIES):
            raise LoadError('LOAD-UNIT', 'the amount of leg %s of %s is in %s, not a unit of %s' % (
                ln.name, inst.id, u, d.unit))
    if units and units != {u}:
        raise LoadError('LOAD-UNIT', 'leg %s of %s moves %s between places of %s' % (
            ln.name, inst.id, u, ', '.join(sorted(units))))
    if au is not None and au != u:
        raise LoadError('LOAD-UNIT', 'the amount of leg %s of %s is in %s, not %s' % (ln.name, inst.id, au, u))
    if amt.q < 0:
        raise refuse('RUN-NEG', 'negative amount %s for leg %s of %s' % (fmt(amt), ln.name, inst.id))
    if u in CURRENCIES and (amt.q / CURRENCIES[u]).denominator != 1:
        raise LoadError('LOAD-UNIT', 'the amount of leg %s of %s (%s) is not money of %s: round it' % (
            ln.name, inst.id, fmt(amt), u))
    return qty(amt.q, u), u


def m_find_inst(self, mi, keys, ctx):
    insts = self.decision_instances(mi, ctx) if mi.decision else self.move_instances(mi, ctx)
    kk = tuple(vkey(k) for k in keys)
    for inst in insts:
        if tuple(vkey(k) for k in inst.keys) == kk:
            return inst
    return None


def m_decision_instances(self, mi, ctx):
    """A decision's instances with their days at cut K (its day reads no running state, L §5.14.1); its when,
    legs and attributes are evaluated in the walk."""
    tctx = self.tctx(ctx)
    key = ('dinst', mi.id, self.wkey(tctx))
    hit = self.run_cache.get(key)
    if hit is not None:
        return hit
    tuples = self.move_tuples(mi, tctx)
    out = []
    for env, keys, st, c in tuples:
        if isinstance(st, tuple):
            inst = Inst(mi, keys, env)
            inst.day = st[1]
            inst.exists = st[1]
            inst.coll = _coll()
        else:
            inst = self.eval_inst(mi, env, keys, tctx, exists0=st)
            if inst is None:
                continue
        inst.coll.merge(c)
        out.append(inst)
    for inst in out:
        inst.order = self.inst_order(inst)
    out.sort(key=lambda i: i.order)
    self.run_cache[key] = out
    return out


# -- statuses (E §4.1 step 3) -----------------------------------------------------------------------------------

def m_obs_links(self, ctx):
    """Instance id -> the observations that fulfil it; RUN-RECORD for a link to nothing (L §5.19).

    "A link reads only what it names: the existence of the instance, by its keys, `where` and `when` (an instance
    exists only where all three hold, ENGINE.md §4.1), never its amounts" (L §5.19, round 17): the instance is found
    among the move's binding tuples with its when (link_tuple), and its legs' places and names are read, not its
    amounts (a plan whose amount reads a binding that checks the link is no cycle). Each target's links are judged on
    their own (obs_links_for), since a `when` read to judge one link may read the status of another instance, which
    rests on that instance's links (MC-X-F1: the whole table was handed out half built)."""
    key = ('links', self.wkey(ctx))
    hit = self.run_cache.get(key)
    if hit is not None:
        links, c = hit
        self.cur.merge(c)
        return links
    links = {}
    c = _coll()
    self.stack.append(c)
    try:
        for tid in self.link_targets(ctx):
            r = self.obs_links_for(tid, ctx)
            if r:
                links[tid] = r
    finally:
        self.stack.pop()
    self.cur.merge(c)
    self.run_cache[key] = (links, c)
    return links


def m_link_targets(self, ctx):
    """Instance id -> the observations whose fulfils names it, in the observations' order."""
    key = ('linkt', self.wkey(ctx))
    hit = self.run_cache.get(key)
    if hit is not None:
        return hit
    out = {}
    for o in self.observations(ctx):
        if o.fulfils is None or o.fulfils[0] != 'inst':
            continue
        _, mid, kv = o.fulfils
        tid = '%s[%s]' % (mid, ', '.join(key_text(k) for k in kv)) if kv else mid
        out.setdefault(tid, []).append(o)
    self.run_cache[key] = out
    return out


def m_obs_links_for(self, tid, ctx):
    """The observations that fulfil instance tid, its links judged (L §5.19). A link whose judging reads the
    instance's own observations again is a cycle (RUN-CYCLE), which link_tuple takes as existing."""
    wk = self.wkey(ctx)
    key = ('link1', wk, tid)
    hit = self.run_cache.get(key)
    if hit is not None:
        # what judging the links read (the target's when, its places) is carried to every reader (L §8.2: "each
        # `when` it tested")
        res, c = hit
        self.cur.merge(c)
        return res
    if key in self.inst_active:
        raise refuse('RUN-CYCLE', 'the link to %s reads its own observations' % tid)
    m = self.m
    res = []
    c = _coll()
    self.inst_active.add(key)
    self.stack.append(c)
    try:
        for o in self.link_targets(ctx).get(tid, []):
            o.__dict__['_judge'] = None
            _, mid, kv = o.fulfils
            mi = m.moves[mid]
            found = self.link_tuple(mi, list(kv), ctx)
            bad = found is None
            places = set()
            if not bad:
                places = self.link_places(mi, found, ctx)
                op = set()
                for lg in o.legs:
                    op.add(lg.frm.name)
                    op.add(lg.to.name)
                if places and not (places & op):
                    bad = True
            if bad:
                o.err = refuse('RUN-RECORD', 'observation %s fulfils %s, which doesn\'t exist or doesn\'t touch its '
                                             'places' % (o.id, tid))
                continue
            lnames = [ln.name for ln, _d in mi.legs]
            have = set(lg.name for lg in o.legs)
            lack = [n for n in lnames if n not in have]
            if lack and m.kind_legs(o.kind):
                # L §5.19 (round 12): a fulfils naming an instance "that has a leg the observation lacks, is refused
                # (RUN-RECORD)"; L §8.4 "observation <r> fulfils <instance> and lacks its leg <l>"
                o.err = refuse('RUN-RECORD', 'observation %s fulfils %s and lacks its leg %s' % (o.id, tid, lack[0]))
                # what reads the instance's realised movement rests on the refused record (L §8.4: a refusal
                # propagates like a hole)
                self.run_cache.setdefault(('badlinks', wk), {}).setdefault(tid, o.err)
                continue
            o.target = _LinkTarget(tid, self, mi, list(kv), ctx)
            res.append(o)
    finally:
        self.stack.pop()
        self.inst_active.discard(key)
    # each observation carries what judging its link read, so whatever counts it lists that too (L §8.2: the register
    # is transitive, "each `when` it tested")
    for o in self.link_targets(ctx).get(tid, []):
        if o.coll is not None:
            o.coll.merge(c)
    self.cur.merge(c)
    self.run_cache[key] = (res, c)
    return res


class _LinkTarget:
    """The instance a confirmed link names: its id at once; the instance itself (its for, its kind) only when a reader
    asks for more, so the link alone reads no amount."""

    def __init__(self, iid, eng, mi, keys, ctx):
        self.id = iid
        self._src = (eng, mi, keys, ctx)
        self._inst = None

    def __getattr__(self, name):
        if name.startswith('_'):
            raise AttributeError(name)
        if self._inst is None:
            eng, mi, keys, ctx = self._src
            self._inst = eng.find_inst(mi, keys, ctx)
        if self._inst is None:
            raise AttributeError(name)
        return getattr(self._inst, name)


def m_link_tuple(self, mi, keys, ctx):
    """The binding tuple of move mi at these keys, at the ask's cut (its keys, `where` and `when`, L §5.19), or None; an
    instance whose `when` is false at that cut doesn't exist (E §4.1 step 2: "An instance whose when is true exists,
    whatever its amounts"). A membership or a when that is a hole or refused is taken as existing: its link is judged
    where its instance is."""
    tctx = self.tctx(ctx)
    kk = tuple(vkey(k) for k in keys)
    for env, ks, st, c in self.move_tuples(mi, tctx):
        if tuple(vkey(k) for k in ks) != kk:
            continue
        if st is False:
            return None
        if mi.decl.when is not None and st is True:
            try:
                law = self.move_law(mi, env, tctx)
                if not isinstance(law, Hole):
                    w = self.ev(mi.decl.when, env, tctx.but(law=law_days(law)[0]))
                    if w is False:
                        return None
            except TallyError as e:
                if load_now(e):
                    raise
        return env
    return None


def m_link_places(self, mi, env, ctx):
    """The places the legs of mi's instance at env give, as known at the cut (no amount is read). Unread where a
    place can't be read here (a hole, a law date not known): the link is then judged by its keys alone."""
    tctx = self.tctx(ctx)
    places = set()
    try:
        law = self.move_law(mi, env, tctx)
        if isinstance(law, Hole):
            return set()
        lctx = tctx.but(law=law_days(law)[0]) if law is not None else tctx
        for ln, _d in mi.legs:
            for e in (ln.frm, ln.to):
                p = self.ev(e, env, lctx)
                if isinstance(p, Thing):
                    places.add(p.name)
                else:
                    return set()
    except TallyError as e:
        if load_now(e):
            raise
        return set()
    return places


def m_status(self, i, ctx):
    """The status of a plan instance in the timeline (E §4.1 step 3, E §1)."""
    wk = self.wkey(ctx)
    if i.status is not None and getattr(i, '_skey', None) == wk:
        return i.status
    m = self.m
    S, E, K = m.S, m.E, ctx.cut
    i.obs = self.obs_links_for(i.id, ctx)
    d = i.day
    lo, hi = day_lo(d), day_hi(d)
    if i.exists is False:
        st = 'none'
    elif lo is not None and (hi < S or lo > E):
        st = 'leftout'
    elif self.scen_early(i, lo):
        # L §6.4: "A move the scenario declares books nothing before D: an instance whose day, a window's first day,
        # or a leg's own day falls before D is left out"
        st = 'leftout'
    elif i.err is not None:
        st = 'refused'
    elif i.obs and not i.decision:
        # on any day, an instance that an observation the cut sees fulfils is modelled past: its observations stand
        # in for it (E §4.1 step 3, L §5.19); one fulfilling a hole movement answers it
        st = 'modelled'
    elif isinstance(d, Hole) or isinstance(i.exists, Hole):
        st = 'hole'
    elif i.decision:
        st = 'decision'
    elif self.kind_certainty(i.kind) == 'uncertain' and not i.obs:
        st = 'uncertain'
    elif lo < K and self.touches_covered(i, ctx):
        st = 'modelled'
    elif lo < K:
        st = 'past'
    else:
        st = 'plan'
    i.status = st
    i._skey = wk
    return st


def m_scen_early(self, i, lo):
    """L §6.4: an instance of a move a scenario declares whose day, a window's first day, or a leg's own day falls
    before the scenario's from."""
    wlo = getattr(i, 'wlo', None)
    if wlo is not None and (lo is None or wlo < lo):
        lo = wlo
    if lo is not None and self.before_scen_from(i, lo):
        return True
    mi = getattr(i, 'move', None)
    sc = getattr(mi.decl, '_scen', None) if mi is not None and mi.decl is not None else None
    frm = getattr(self.m, 'scen_from', {}).get(sc) if sc is not None else None
    if frm is None:
        return False
    for lg in getattr(i, 'legs', None) or ():
        ld = day_lo(getattr(lg, 'day', None))
        if ld is not None and ld < frm:
            return True
    return False


def m_before_scen_from(self, i, day):
    """Whether i is an instance of a move a scenario declares, dated before that scenario's from (L §6.4)."""
    mi = getattr(i, 'move', None)
    sc = getattr(mi.decl, '_scen', None) if mi is not None and mi.decl is not None else None
    if sc is None:
        return False
    frm = getattr(self.m, 'scen_from', {}).get(sc)
    if frm is None or not day < frm:
        return False
    ch = getattr(self.m, 'checker', None)
    if ch is not None and not ch.edges.get(('moveon', mi.id)) and mi.decl.when is None:
        # its day reads nothing but its keys, and it has no when: load sees the day (LOAD-SCENARIO where load sees
        # the day, L §6.4)
        # L §8.3 (round 9): "move <m> of <s> books on <d>, before <from>"
        raise LoadError('LOAD-SCENARIO', 'move %s of %s books on %s, before %s' % (mi.id, sc, day, frm), mi.decl.loc)
    return True


def m_touches_covered(self, i, ctx):
    for lg in i.legs:
        ld = lg.day if isinstance(lg.day, datetime.date) else day_lo(i.day)
        if ld is None:
            continue
        for p in (lg.frm, lg.to):
            if isinstance(p, Thing) and self.covered_on(p.name, ld, ctx):
                return True
    return False


def m_plan_moves(self):
    return [self.m.moves[q] for q in sorted(self.m.moves) if not self.m.moves[q].decision and
            item_in_world(self.m.moves[q])]


def m_settlement_move(self, mi):
    """E §5 step 2 (round 19, GD-17): "A movement whose fulfils names an adjustment (a settlement) isn't measured";
    L §5.14.3 "Settlements are never measured". snap14 took a move that binds over adjustments as the settlement."""
    return bool(getattr(mi, 'fulfils_adj', False))


def m_plan_insts(self, ctx):
    """Every plan instance of the timeline with its status, in movement order (for a walk up to a day,
    mreads.m_prefix_walk, the ones dated on or before it)."""
    key = ('plans', self.wkey(ctx)) + self.upto_tag()
    hit = self.run_cache.get(key)
    if hit is not None:
        return hit
    out = []
    for mi in self.plan_moves():
        for i in self.plan_list(mi, ctx):
            self.status(i, ctx)
            out.append(i)
    out.sort(key=lambda i: i.order)
    self.run_cache[key] = out
    return out


# -- adjustments (E §5 steps 2-5) ----------------------------------------------------------------------------------

def m_known_dates(self):
    m = self.m
    kd = getattr(m, '_known_dates', None)
    if kd is not None:
        return kd
    ds = {}
    for v in m.versions.values():
        ds.setdefault(known_first(v.known), set()).add(v.id)
    for inf in m.inputs.values():
        for r in inf.rows:
            if r.own_known is not None:
                ds.setdefault(known_first(r.own_known), set()).add(r.version.id)
        for o in getattr(inf, 'obs', []):
            if o.own_known is not None:
                ds.setdefault(known_first(o.own_known), set()).add(o.version.id)
    m._known_dates = sorted(ds.items())
    return m._known_dates


def m_adjustments(self, ctx, settled=True):
    """Every adjustment of the timeline (E §5): per booked plan instance and changed leg, per later step.  With
    settled, its settlements and waivers linked (step 5); a binding over adjustments needs only the records, so a
    settlement move that binds over them doesn't read itself."""
    key = ('adjs', self.wkey(ctx)) + self.upto_tag()
    out = self.run_cache.get(key)
    if out is None:
        if key in self.inst_active:
            raise refuse('RUN-CYCLE', 'the adjustments read themselves (a measured move reads adjustments)')
        self.inst_active.add(key)
        try:
            out = self.adjust_all(ctx)
        finally:
            self.inst_active.discard(key)
        self.run_cache[key] = out
    skey = ('settled',) + key[1:]
    if settled and skey not in self.run_cache:
        self.run_cache[skey] = True
        self.settle(out, ctx)
        names = set()
        for a in out:
            for vid in a.vids:
                names.add((a.inst.id, a.leg.name, vid))
        for o in self.observations(ctx):
            f = o.fulfils
            if isinstance(f, tuple) and f[0] == 'adj':
                _, mid, kv, leg, vid = f
                iid = '%s[%s]' % (mid, ', '.join(key_text(k) for k in kv)) if kv else mid
                if (iid, leg, vid) not in names and o.err is None:
                    # L §5.19 A link is judged by existence: "a fulfils, an observation's or a settling move's, naming
                    # an adjustment that no version made by that cut, or whose version makes no change to that leg
                    # there" is refused (RUN-RECORD); L §8.4 "<movement> fulfils <adjustment>, which no version made
                    # by the cut"
                    o.err = refuse('RUN-RECORD', '%s fulfils adjustments[%s.%s, %s], which no version made by the cut'
                                   % (o.id, iid, leg, vid))
    return out


def m_adjust_all(self, ctx):
    out = []
    for mi in self.plan_moves():
        if self.settlement_move(mi):
            continue
        out.extend(self.adjust_move(mi, ctx))
    out.sort(key=lambda a: a.order)
    return out


def m_adjust_move(self, mi, ctx):
    """The adjustments of one plan move's instances (E §5 steps 1-4), kept per move, so a plan that reads another
    plan's flows measures that plan only (E §4.1: the instance graph is acyclic)."""
    key = ('adjm', mi.id, self.wkey(ctx)) + self.upto_tag()
    out = self.run_cache.get(key)
    if out is not None:
        return out
    if self.settlement_move(mi):
        self.run_cache[key] = []
        return []
    if key in self.inst_active:
        raise refuse('RUN-CYCLE', 'the adjustments of %s read themselves' % mi.id)
    self.inst_active.add(key)
    try:
        out = self.adjust_move_compute(mi, ctx)
    finally:
        self.inst_active.discard(key)
    self.run_cache[key] = out
    return out


def m_adjust_move_compute(self, mi, ctx):
    m = self.m
    K = ctx.cut
    tctx = self.tctx(ctx)
    out = []
    kds = self.known_dates()
    if True:
        for i in self.plan_list(mi, ctx):
            st = self.status(i, ctx)
            if st not in ('past', 'hole', 'none'):
                continue
            if not isinstance(i.day, datetime.date) or i.day >= K:
                continue
            t = i.day
            steps = [(c, vs) for c, vs in kds if t < c <= K]
            if not steps:
                continue
            prev = i
            prev_ex = i.exists is True
            hole0 = isinstance(i.exists, Hole) or any(isinstance(lg.amount, Hole) for lg in i.legs)
            for c, vids in steps:
                j = self.eval_inst(mi, i.env, i.keys, tctx, at_cut=c)
                if j is None or j.err is not None:
                    continue
                if not self.keys_seen(mi, i.keys, c):
                    j.exists = False
                if hole0:
                    if isinstance(j.exists, Hole) or any(isinstance(lg.amount, Hole) for lg in j.legs):
                        continue
                    # a version answers the hole movement: its amounts are the instance's (E §5 step 1)
                    i.exists, i.legs = j.exists, j.legs
                    i.coll.merge(j.coll)
                    i.status = None
                    self.status(i, ctx)
                    hole0 = False
                    prev, prev_ex = j, j.exists is True
                    continue
                if isinstance(j.exists, Hole):
                    continue
                ex = j.exists is True
                legs_prev = {lg.name: lg for lg in prev.legs} if prev_ex else {}
                legs_new = {lg.name: lg for lg in j.legs} if ex else {}
                # E §5 step 3: "for each leg let δ = Aᵥ − A_previous ... when V ends one, δ = −A_previous": the legs
                # are those of the two states compared, the previous step's (when it existed) and this one's. Writer 9,
                # MC-X-F13: snap13 took them from the booked instance, else this step's, so an instance that a step
                # made exist and a later step ended (no legs at κ₀, none now) lost its ending adjustment
                names = []
                for lg in list(i.legs) + list(legs_prev.values()) + list(legs_new.values()):
                    if lg.name not in names:
                        names.append(lg.name)
                pool = list(i.legs or j.legs) + list(legs_prev.values())
                for ln in names:
                    a = legs_prev.get(ln)
                    b = legs_new.get(ln)
                    qa = a.amount if a is not None else None
                    qb = b.amount if b is not None else None
                    if isinstance(qa, TallyError) or isinstance(qb, TallyError):
                        continue
                    if a is None and b is None:
                        # the instance exists neither before this step nor after it (V neither makes nor ends it):
                        # both count as 0 (E §5 step 1), so δ = 0, and no leg has places V changes: nothing to
                        # record (E §5 steps 3-4). Writer 9, MC-X-F13: snap13 read the unit of neither leg
                        # (AttributeError) when a step after one that ended the instance found it still ended
                        continue
                    u = (b or a).unit
                    if isinstance(qa, Hole) or isinstance(qb, Hole):
                        lg = [x for x in pool if x.name == ln][0]
                        adj = AdjMv(i, lg, merge(qa, qb), c, self.step_versions(j, vids))
                        adj.coll = j.coll
                        i.adjs.append(adj)
                        out.append(adj)
                        continue
                    if u is None:
                        continue
                    da = qa.q if qa is not None else 0
                    db = qb.q if qb is not None else 0
                    # L §5.14.3 (round 12b): a version that changes "the places of one of its legs (then even with
                    # δ = 0: the booked movement stays where it was booked, and the change is never silent)"; E §5
                    # step 4 "Record δ ≠ 0, or a leg whose places V changes (δ may be 0)"
                    moved = a is not None and b is not None and \
                        (vkey(a.frm), vkey(a.to)) != (vkey(b.frm), vkey(b.to))
                    if db != da or moved:
                        lg = [x for x in pool if x.name == ln][0]
                        adj = AdjMv(i, lg, qty(db - da, u), c, self.step_versions(j, vids))
                        adj.coll = j.coll
                        adj.made = not prev_ex and ex
                        adj.moved = moved
                        if moved:
                            # the leg's places before V and those V gives it (L §5.14.3 How a settlement leg counts)
                            adj.froms = (vkey(a.frm), vkey(b.frm))
                            adj.tos = (vkey(a.to), vkey(b.to))
                            adj.new_ends = (b.frm, b.to)
                        i.adjs.append(adj)
                        out.append(adj)
                prev, prev_ex = j, ex
    out.sort(key=lambda a: a.order)
    return out


def m_keys_seen(self, mi, keys, c):
    m = self.m
    for b, k in zip(mi.binds, keys):
        d = b.dom
        if d[0] == 'index':
            if not m.indexes[d[1]].visible(k, c):
                return False
        elif d[0] == 'kind' and isinstance(k, Thing) and k.name in m.things:
            if not m.thing_seen(m.things[k.name], c):
                return False
    return True


def m_step_versions(self, j, vids):
    read = set()
    for r in j.coll.rows.values():
        v = getattr(r, 'version', None)
        if v is not None:
            read.add(v.id)
    hit = sorted(v for v in vids if v in read)
    return hit or sorted(vids)


def m_settle(self, adjs, ctx):
    """Link settlements and waivers to their adjustments (E §5 step 5)."""
    # each adjustment is linked once, whether by a move's own flows (its adjustments only) or by the whole pass
    adjs = [a for a in adjs if not getattr(a, '_linked', False)]
    if not adjs:
        return
    for a in adjs:
        a._linked = True
    m = self.m
    byid = {a.id: a for a in adjs}
    for mi in self.plan_moves():
        if mi.decl.fulfils is None:
            continue
        for s in self.settle_candidates(mi, adjs, ctx):
            if s.exists is not True or s.err is not None:
                continue
            f = s.fulfils
            if not isinstance(f, Mv) or f.id not in byid:
                continue
            a = byid[f.id]
            s.settles = a
            if not m.kind_legs(s.kind):
                a.waived = True
                a.settled.append((s, 0, Num(0)))
                continue
            for lg in s.legs:
                a.settled.append((s, settle_sign(lg, a, s), lg.amount))
    # observed rows that settle an adjustment, named adjustments[m[keys].leg, V] (L §5.14.3, §5.19)
    named = {}
    for o in self.observations(ctx):
        f = o.fulfils
        if isinstance(f, tuple) and f[0] == 'adj':
            _, mid, kv, leg, vid = f
            iid = '%s[%s]' % (mid, ', '.join(key_text(k) for k in kv)) if kv else mid
            named.setdefault((iid, leg, vid), []).append(o)
    if named:
        for a in adjs:
            for vid in a.vids:
                for o in named.get((a.inst.id, a.leg.name, vid), ()):
                    # one adjustment of a step of several versions, named by each of them (E §5 step 4)
                    o.settles = a
                    if not o.legs:
                        a.waived = True
                        a.settled.append((o, 0, Num(0)))
                        continue
                    for lg in o.legs:
                        a.settled.append((o, settle_sign(lg, a, o), lg.amount))


def settle_sign(lg, a, s):
    """The sign of a settlement leg (E §5 step 5, round 12): +amount "when it runs the adjusted leg's way, out of its
    from or into its to", -amount "when it runs the other way, out of its to or into its from"; "a settlement leg that
    touches neither of the adjusted leg's places is refused (RUN-RECORD)"."""
    if lg.unit is not None and a.leg.unit is not None and lg.unit != a.leg.unit:
        raise LoadError('LOAD-UNIT', 'settlement %s of %s is in %s, not %s' % (s.id, a.id, lg.unit, a.leg.unit))
    if getattr(a, 'moved', False):
        # L §5.14.3 How a settlement leg counts: "For a change of places, the leg's places are both those it had
        # before V and those V gives it, so a settlement leg between two of its `to` places, or between two of its
        # `from` places, runs both ways and counts 0 ... A version that turns a leg round, a place its `from` on one
        # side and its `to` on the other, gives settlement legs no sign: they are refused (RUN-RECORD)"
        froms, tos = set(a.froms), set(a.tos)
        if froms & tos:
            raise refuse('RUN-RECORD', 'settlement %s of %s: the leg is turned round' % (s.id, a.id))
        f, t = vkey(lg.frm), vkey(lg.to)
        fwd = f in froms or t in tos
        back = f in tos or t in froms
        if fwd and back:
            return 0
        if fwd:
            return 1
        if back:
            return -1
        raise refuse('RUN-RECORD', 'settlement %s of %s touches neither place of its leg' % (s.id, a.id))
    f, t = vkey(lg.frm), vkey(lg.to)
    af, at = vkey(a.leg.frm), vkey(a.leg.to)
    if f == af or t == at:
        return 1
    if f == at or t == af:
        return -1
    raise refuse('RUN-RECORD', 'settlement %s of %s touches neither place of its leg' % (s.id, a.id))
