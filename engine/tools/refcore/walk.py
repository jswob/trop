"""The day walk (E §4.2) and projections (E §4.3).

A walk keeps, per (place, unit), a log of entries in walk order, each with the balance state after it, so a
balance at any instant is a search.  An entry's key is (day, phase, ...): phase 0 the start of the day (readings
at `t start`), 1 observations, 2 plans, 3 decisions (by stratum), 4 readings at `t end`, 5 the close.  A state
is the known quantity (None before a place's first reading: HOLE-ANCHOR), the questions of the hole movements
since the last reading, the questions no reading answers (a hole day, HOLE-CHANGED), the windows a movement may
still fall in (HOLE-IMPRECISE), and the refusal that holds from a refused leg on.

Installed as Engine methods (`m_<name>` -> `<name>`).
"""
import bisect
import datetime

from .dates import ONE_DAY
from .moves import item_in_world
from .errors import TallyError, Refusal, LoadError, Unspecified, NotYet, load_now, RandomLine
from .model import known_first, key_text, SameDiffers
from .values import Num, Money, Hole, Thing, Mv, Iv, Rec, Text, NONE, CURRENCIES, fmt, fmt_hole, vkey
from .moves import Leg, Inst, ObsMv, AdjMv, PadMv, qty, unit_of_value, day_lo, day_hi, instant_text, merge

EMPTY = frozenset()


def install(E):
    for name, fn in list(globals().items()):
        if name.startswith('m_'):
            setattr(E, name[2:], fn)


def refuse(code, msg):
    return Refusal(code, msg)


def _coll():
    from .evaluate import Coll
    return Coll()


def K_end(d):
    return (d.toordinal(), 9, 0, '', 0)


def K_start(d):
    return (d.toordinal(), 0, 9, '', 0)


def K_before(d):
    return (d.toordinal(), -1, 0, '', 0)


def kday(key):
    return datetime.date.fromordinal(key[0])


def same_book_key(x, lg):
    """Where a movement's leg is booked in the day's order (Walk.prepare's keys): observations before plans, plans
    before decisions, a decision at its point (E §4.2 steps 1-3)."""
    d = lg.day if isinstance(lg.day, datetime.date) else day_lo(x.day)
    o = d.toordinal() if isinstance(d, datetime.date) else 0
    if getattr(x, 'status', None) == 'observed' or x.__class__.__name__ == 'ObsMv':
        return (o, 1, 0, '%s %s' % (x.id, lg.name), 0)
    if getattr(x, 'decision', False):
        pt = getattr(x, 'point', None)
        return pt if pt is not None and pt[0] == o else (o, 3, 0, x.id, 0)
    order = getattr(x, 'order', None)
    osort = '%s|%s' % (x.move.id, order[3] if order else '') if hasattr(x, 'move') else x.id
    return (o, 2, 0, osort, 0)


class St:
    __slots__ = ('val', 'hole', 'perm', 'imp', 'bad', 'rst')

    def __init__(self, val, hole=EMPTY, perm=EMPTY, imp=(), bad=None, rst=-1):
        self.val = val          # Num, or None before the place's first reading (HOLE-ANCHOR)
        self.hole = hole        # questions of hole movements since the last reset
        self.perm = perm        # questions no reading answers
        self.imp = imp          # ((window id, questions), ...) of movements that may still fall
        self.bad = bad          # the refusal that holds from a refused leg on
        self.rst = rst          # index of the last reset entry in its log


class LeftHole(Hole):
    """A leg end the joint model's own movement would book in a member's place with no link (L §6.1 SF-32): a hole that
    also lists the movement (left out:)."""
    __slots__ = ('left_out',)


class Ent:
    __slots__ = ('key', 'kind', 'val', 'qs', 'err', 'wid', 'mv', 'flow', 'reg', 'coll', 'st', 'win', 'note', 'sgn',
                 'leg', 'row')

    def __init__(self, key, kind, val=None, qs=EMPTY, err=None, wid=None, mv=None, flow=None, reg=(), coll=None,
                 win=None, note=None, sgn=None, leg=None, row=None):
        self.row = row          # a reading's row (its stated parts, L §5.19, tally 2.1)
        self.sgn = sgn          # a hole leg's direction at the place (-1 out, +1 in) and the leg: a partial ask line
        self.leg = leg          # reads the balance as its known part plus the leg's holes (mreads.m_partial_bal)
        self.key = key
        self.kind = kind
        self.val = val
        self.qs = qs
        self.err = err
        self.wid = wid
        self.mv = mv
        self.flow = flow
        self.reg = reg
        self.coll = coll
        self.st = None
        self.win = win
        self.note = note


class Log:
    __slots__ = ('ents', 'keys', 'cum', 'wins')

    def __init__(self):
        self.ents = []
        self.keys = []
        self.cum = []
        self.wins = []


def add_q(a, b, sign=1):
    if a.money:
        return Money(a.q + sign * b.q, a.u[0][0])
    return Num(a.q + sign * b.q, a.u)


def apply(st, e, idx):
    k = e.kind
    if k == 'reset':
        return St(e.val, frozenset(e.qs), st.perm, st.imp, st.bad, idx)
    if k == 'init':
        s = e.val
        return St(s.val, s.hole, s.perm, s.imp, s.bad, idx)
    if k == 'delta':
        v = st.val
        if v is not None:
            v = add_q(v, e.flow)
        imp = st.imp
        if e.wid is not None:
            imp = tuple(x for x in imp if x[0] != e.wid)
        return St(v, st.hole, st.perm, imp, st.bad, st.rst)
    if k == 'hole':
        imp = st.imp
        if e.wid is not None:
            imp = tuple(x for x in imp if x[0] != e.wid)
        return St(st.val, st.hole | e.qs, st.perm, imp, st.bad, st.rst)
    if k == 'perm':
        return St(st.val, st.hole, st.perm | e.qs, st.imp, st.bad, st.rst)
    if k == 'imp_on':
        return St(st.val, st.hole, st.perm, st.imp + ((e.wid, e.qs),), st.bad, st.rst)
    if k == 'bad':
        return St(st.val, st.hole, st.perm, st.imp, st.bad or e.err, st.rst)
    return st


class Walk:
    """One walk of a timeline: the main walk from S to E, or a projection's from day t (E §4.3)."""

    def __init__(self, eng, tctx, start, end, base=None, point=None, leftout=frozenset(), proj=False, unc=False):
        self.eng = eng
        self.m = eng.m
        self.tctx = tctx
        self.K = tctx.cut
        self.start = start
        self.end = end
        self.base = base
        self.point = point          # the base point a projection starts after
        self.leftout = leftout
        self.proj = proj
        self.unc = unc
        self.logs = {}
        self.joint_lo = {}          # place -> [(key, register entry)]: the joint model's own unlinked movements there
        self.done = start - ONE_DAY
        self.sched = {}
        self.after = {}
        self.after_keys = {}     # id(reading after m) -> the walk key it was applied at (E §6.3 balance lines)
        self.decs = {}              # decision instance id -> its run in this walk
        self.wp = None              # the walk point being evaluated
        self.gaps = {}
        self.pads = []
        self.cons = {}              # (unit) -> the day's booked flows, summed over every place (E §9 invariant 1)
        self.inv2 = {}              # (unit) -> [inside change, boundary + gaps], or None when unknown (invariant 2)
        self._inside = {}
        self.modelled = []
        self.warnings = []
        self.neg = {}
        self.touched = set()
        self.adj_regs = {}
        self.leftout_regs = {}
        self.leftout_by_place = {}
        self.decday = {}
        self.uncertain = []
        self.reads = None
        # the places the running decision reads as they stand at its point, which another decision of its day can
        # change (bal(x), flows through its day, a projection); a plain bal(x, d) reads a point before the day's
        # decisions (a later one is refused), so it orders nothing (E §4.2 step 3, L §5.14.2)
        self.live = None
        self.counted_leftout = {}
        self.leftout_insts = []     # (day, instance id) of the movements a projection leaves out (E §10)
        self.bad_legs = {}          # (movement id, leg) -> the refusal of a leg the walk refused (E §4.2 step 5)
        self.running = False

    # -- E §9 invariant 2: worth moves only at the boundary or at an anchor ---------------------------------------
    def inside(self, place):
        """Whether a place counts in the household's worth: every place that isn't outside (E §9 invariant 2)."""
        hit = self._inside.get(place)
        if hit is None:
            m = self.m
            th = m.things.get(place)
            hit = th is not None and th.base not in m.kinds['outside'].subtree(m)
            self._inside[place] = hit
        return hit

    def inv2_add(self, unit, expected):
        """Add to the day's expected change in worth in a unit (None: unknown that day)."""
        if unit in self.inv2 and self.inv2[unit] is None:
            return
        if expected is None:
            self.inv2[unit] = None
            return
        self.inv2[unit] = self.inv2.get(unit, 0) + expected

    def inv2_check(self, d):
        """E §9 invariant 2, per unit (so no price is needed): the day's change in the balances of the places that
        aren't outside equals the movements across the boundary, the pads from outside and the anchors' and
        openings' gaps (legs booked by their ends left out, as from invariant 1)."""
        exp = self.inv2
        self.inv2 = {}
        if self.proj:
            return
        actual = {}
        ke, kp = K_end(d), K_end(d - ONE_DAY)
        for place in self.touched:
            if not self.inside(place):
                continue
            for (p, u) in self.logs:
                if p != place or u == '*':
                    continue
                a = self.pstate(place, u, ke, raw=True)
                b = self.pstate(place, u, kp, raw=True)
                bad = lambda st: st.val is None or st.hole or st.perm or st.imp or st.bad
                if bad(a) or bad(b):
                    actual[u] = None
                    continue
                if actual.get(u, 0) is None:
                    continue
                actual[u] = actual.get(u, 0) + a.val.q - b.val.q
        for u in set(actual) | set(k for k in exp):
            x, y = actual.get(u, 0), exp.get(u, 0)
            if x is None or y is None:
                continue
            if x != y:
                raise refuse('ENGINE-INVARIANT', 'net worth: the places that aren\'t outside change by %s in %s on %s, '
                                                 'and the boundary, pads and gaps by %s (E §9 invariant 2)' % (
                                                     fmt(qty(x, u)), u, d, fmt(qty(y, u))))

    def signature(self):
        """The timeline's hash (E §9 invariant 4): every booked entry of every place."""
        parts = []
        for k in sorted(self.logs, key=repr):
            lg = self.logs[k]
            parts.append((repr(k), tuple((e.key, e.kind, repr(vkey(e.val)) if e.val is not None else None,
                                          repr(vkey(e.flow)) if isinstance(e.flow, Num) else None,
                                          getattr(e.mv, 'id', None)) for e in lg.ents)))
        return hash(tuple(parts))

    # -- logs ------------------------------------------------------------------------------------------------
    def log(self, place, unit):
        k = (place, unit)
        lg = self.logs.get(k)
        if lg is None:
            lg = Log()
            self.logs[k] = lg
        return lg

    def default(self, place, unit):
        if self.base is not None:
            b = self.base.pstate(place, unit, self.point, raw=True)
            return St(b.val, b.hole, b.perm, b.imp, b.bad, -1)
        if place == '*' or unit == '*':
            return St(Num(0), rst=-1) if place == '*' else St(None)
        m = self.m
        th = m.things.get(place)
        zero = qty(0, unit) if unit not in (None, '*') else Num(0)
        try:
            sp = m.lifespan(th, self.K) if th is not None else None
        except SameDiffers as e:
            # a place stated same whose sides' lifespans differ (L §6.1): its balance is zero at its lifespan's start,
            # or needs an anchor reading when it lives before S (L §5.10 Balances); where the sides disagree on which,
            # the balance depends on the place being alive, a HOLE-SAME hole until a reading
            zs = set(sp.start is not None and sp.start >= self.start for sp in self.side_spans(th))
            if zs == set([True]):
                return St(zero)
            if zs == set([False]):
                return St(None)
            return St(zero, hole=frozenset(e.hole().qs))
        if sp is not None and sp.start is not None and sp.start >= self.start:
            return St(zero)
        return St(None)

    def side_spans(self, th):
        """The lifespans each side of a place stated same states under the walk's cut (the ones it sees)."""
        m = self.m
        sub = set([th.base])
        return [sp for sp in (th.span_in(th.base, self.K, o) for o in (m.same_sides(th, sub) or [None]))
                if sp is not None]

    def alive_day(self, th, day):
        """Whether a place is alive on day; for a place stated same whose sides' lifespans disagree on that day, the
        HOLE-SAME hole (L §6.1 What same does: "whatever depends on the thing being alive there is a HOLE-SAME
        hole")."""
        m = self.m
        try:
            sp = m.lifespan(th, self.K)
        except SameDiffers:
            try:
                return m.alive_on(th, th.base, day, self.K)
            except SameDiffers as e:
                return e.hole()
        if sp is not None and sp.end_q is not None and sp.contains(day):
            # a lifespan whose end is a stated hole: alive on a day after its start is that hole (L §5.7, round 18)
            return Hole([(sp.end_q, th.name)])
        return sp is not None and sp.contains(day)

    def add(self, place, unit, e):
        lg = self.log(place, unit)
        prev = lg.ents[-1].st if lg.ents else self.default(place, unit)
        e.st = apply(prev, e, len(lg.ents))
        lg.ents.append(e)
        lg.keys.append(e.key)
        # tally 2.1: an entry at a place that keeps parts waits for its point's group (partswalk)
        self.padd(place, unit, e)
        for r in e.reg or ():
            if r and r[0] == 'left out joint':
                # L §6.1 (round 17b): "Every result that reads the place on or after the movement's day lists it
                # under left out:", a reading after it included (mreads.read_state, flows)
                self.joint_lo.setdefault(place, []).append((e.key, r))
        if place not in ('*',):
            self.touched.add(place)
        return e

    def state(self, place, unit, key):
        lg = self.logs.get((place, unit))
        if lg is None:
            return self.default(place, unit), None, -1
        i = bisect.bisect_right(lg.keys, key) - 1
        if i < 0:
            return self.default(place, unit), lg, -1
        return lg.ents[i].st, lg, i

    def pstate(self, place, unit, key, raw=False):
        """The state of a place's position in a unit at an instant, with what holds for all its units and for the
        whole walk."""
        st, lg, i = self.state(place, unit, key)
        if raw:
            a, _, _ = self.state(place, '*', key)
            g, _, _ = self.state('*', '*', key)
            if a.hole or a.perm or a.imp or a.bad or g.hole or g.perm or g.imp or g.bad:
                return St(st.val, st.hole | a.hole | g.hole, st.perm | a.perm | g.perm, st.imp + a.imp + g.imp,
                          st.bad or a.bad or g.bad, st.rst)
            return st
        return st

    def cum(self, lg, i):
        """The collector of every entry since the last reset through entry i."""
        if lg is None or i < 0:
            return None
        while len(lg.cum) <= i:
            j = len(lg.cum)
            e = lg.ents[j]
            c = _coll()
            if j > 0 and e.st.rst < j and lg.cum[j - 1] is not None and e.kind not in ('reset', 'init'):
                c.merge(lg.cum[j - 1])
            if e.coll is not None:
                c.merge(e.coll)
            for r in e.reg:
                c.reg.add(r)
            lg.cum.append(c)
        return lg.cum[i]

    # -- scheduling (E §4.1, §4.2) --------------------------------------------------------------------------------
    def at(self, key, kind, obj):
        self.sched.setdefault(key[0], []).append((key, kind, obj))

    def prepare(self):
        eng = self.eng
        m = self.m
        ctx = self.tctx
        S = self.start
        K = self.K
        if not self.proj:
            for iid, place, unit, r in eng.readings(ctx):
                ins = r.instant
                if ins[0] == 'after':
                    from .check import _txt
                    self.after.setdefault(_txt(ins[1]), []).append((iid, place, unit, r))
                    continue
                d, pos = ins
                if d < S or d > self.end:
                    if iid == 'opening' and d < S and d < K:
                        # E §4.2 step 0 (round 19, GD-21): an opening at t start anchors "with t before K or t = S"
                        self.at(K_before(S), 'read', (iid, place, unit, r))
                    continue
                ph = 0 if pos == 'start' else 4
                if iid == 'opening' and not (d == S and pos == 'start') and not d < K:
                    # "Like an observation, a reading applies only before the cut K" (L §5.19); E §4.2 steps 0 and 4
                    # (round 18, C2-2): an opening at t start, or t end, "with t before K". The walk "starts at S
                    # from the opening readings" (E §4.2; L §5.19 "gives its balance where the walk starts"); round 19
                    # (GD-21) states it: step 0 anchors an opening at t start "with t before K or t = S", L §5.19
                    # "except an opening reading at S start, which anchors whatever K"
                    continue
                self.at((d.toordinal(), ph, 1, place + ' ' + instant_text(r), 0), 'read', (iid, place, unit, r))
            for o in eng.observations(ctx):
                for lg in o.legs:
                    d = lg.day
                    if not isinstance(d, datetime.date) or d < S or d > self.end:
                        continue
                    self.at((d.toordinal(), 1, 0, o.id + ' ' + lg.name, 0), 'obs', (o, lg))
            # a cover whose version is known in a window that holds the cut (L §4.5): a hole from its first day
            for place, span, q in eng.coverage_undecided(ctx):
                d = span.start if span.start is not None and span.start > S else S
                if d <= self.end:
                    self.at((d.toordinal(), 0, 0, 'covers ' + place, 0), 'covhole', (place, q))
            links = eng.obs_links(ctx)
            # two observations of versions covering the two ends that fulfil one instance and disagree (E §6.1)
            for tid in sorted(links):
                os = links[tid]
                for a_i in range(len(os)):
                    for b_i in range(a_i + 1, len(os)):
                        oa, ob = os[a_i], os[b_i]
                        if oa.row.version is ob.row.version:
                            continue
                        la = {g.name: g.amount for g in oa.legs}
                        lb = {g.name: g.amount for g in ob.legs}
                        if any(n in lb and vkey(la[n]) != vkey(lb[n]) for n in la if not isinstance(la[n], Hole)):
                            txt = 'the two observations of %s disagree: %s (%s) and %s (%s)' % (
                                tid, oa.id, ', '.join(fmt(la[n]) for n in sorted(la)), ob.id,
                                ', '.join(fmt(lb[n]) for n in sorted(lb)))
                            if ('WARN-LEGS', txt) not in self.warnings:
                                self.warnings.append(('WARN-LEGS', txt))
        # a decision stating `before m` of a plan m runs ahead of m's legs on its days: "the decision then reads the
        # balance before m" (L §5.14.2 A written order outside a cycle, SF-15). Those legs are booked right after it.
        self.defer = {}
        for pid, decs in sorted(getattr(m, 'deferred_plans', {}).items()):
            for a in sorted(decs):
                mi = m.moves[a]
                if not item_in_world(mi):
                    continue
                for di in eng.decision_instances(mi, ctx):
                    if isinstance(di.day, datetime.date) and di.exists is not False:
                        o = di.day.toordinal()
                        cur = self.defer.setdefault(pid, {}).get(o)
                        k = (mi.stratum, a)
                        if cur is None or k > cur:
                            self.defer[pid][o] = k
        # plans
        for i in eng.plan_insts(ctx):
            if i.move.id in self.leftout:
                if i.status not in ('none',):
                    self.count_leftout(i)
                continue
            st = i.status
            if st == 'leftout':
                self.note_leftout(i)
                continue
            if st in ('none', 'modelled'):
                if st == 'modelled':
                    self.modelled.append(i)
                    if not i.obs and not self.same_observed(i):
                        # a planned instance in a covered span that nothing fulfils: open (L §5.19); one stated same
                        # as an observed row is that observed movement (L §6.1), as for unfulfilled (TX-F1)
                        for lg in i.legs:
                            if isinstance(lg.day, datetime.date) and lg.day >= S:
                                for p in (lg.frm, lg.to):
                                    if isinstance(p, Thing):
                                        self.at((lg.day.toordinal(), 2, 0, i.id, 0), 'mark',
                                                (p.name, lg.unit, ('open', i.id), i.coll))
                continue
            if st == 'uncertain':
                self.uncertain.append(i)
                continue
            d = day_lo(i.day)
            if d is not None and d < S and self.proj:
                continue
            if st == 'refused':
                self.sched_refused(i)
                continue
            if st == 'hole':
                self.sched_hole(i)
                continue
            for lg in i.legs:
                self.sched_leg(i, lg, 2)
        # decisions
        for qid in sorted(m.moves):
            mi = m.moves[qid]
            if not mi.decision or not item_in_world(mi):
                continue
            if qid in self.leftout:
                for i in eng.decision_instances(mi, ctx):
                    self.count_leftout(i)
                continue
            for i in eng.decision_instances(mi, ctx):
                d = i.day
                if i.exists is False:
                    continue
                if isinstance(d, datetime.date):
                    if eng.before_scen_from(i, d):
                        # a scenario's move books nothing before its from (L §6.4): left out
                        if not self.proj:
                            self.note_leftout(i)
                        continue
                    if d < S or d > self.end:
                        if not self.proj and (d < m.S or d > m.E):
                            self.note_leftout(i)
                        continue
                    k = (qid, d)
                    if k not in self.decday:
                        self.decday[k] = []
                        self.at((d.toordinal(), 3, mi.stratum, qid, 0), 'decday', (mi, d))
                    self.decday[k].append(i)
                elif isinstance(d, Iv) and isinstance(d.lo, datetime.date):
                    q = Hole([('date within %s not precise enough for the day of %s' % (d.window_text(), i.id),
                               i.id)])
                    self.dec_hole(i, K_before(max(d.lo, S)), q, ctx)
                elif isinstance(d, Hole):
                    self.dec_hole(i, K_before(S), d, ctx)
                elif isinstance(i.err, RandomLine) and ctx.path is None:
                    # a decision whose day reaches a sample, outside a path: refused at the places it books only
                    # (E §7 step 1; moves.eval_inst, W8-SEEN)
                    self.dec_bad(i, K_before(S), i.err, ctx)
                elif i.err is not None:
                    self.at(K_before(S), 'global_bad', i.err)
        # adjustments: HOLE-CHANGED and remainders (E §5 step 5)
        if not self.proj:
            for a in eng.adjustments(ctx):
                self.sched_adj(a)

    def dec_hole(self, i, key, h, ctx):
        """A decision instance whose day is a hole (or a window no clause fixes) is a hole movement: both ends of its
        legs are holes from the first day it may fall on (L §5.14, E §4.1 step 2, E §4.2 Holes in the walk); other
        places stand ("Results about earlier days, and about other places, stand"). Its legs' places are read at the
        cut K, as its day is; a leg whose place is itself a hole books nothing known, and where the places can't be
        read here every place is a hole."""
        from .evaluate import HoleSignal
        eng = self.eng
        places = set()
        try:
            for ln, _d in i.move.legs:
                for e in (ln.frm, ln.to):
                    p = eng.ev(e, i.env, ctx)
                    if isinstance(p, Thing):
                        places.add(p.name)
                    elif not isinstance(p, Hole):
                        raise ValueError
        except (TallyError, ValueError, HoleSignal):
            self.at(key, 'global_hole', h)
            return
        for p in sorted(places):
            self.at(key, 'placehole', (p, frozenset(h.qs), i))

    def dec_bad(self, i, key, err, ctx):
        """A decision instance refused for its day (a random day outside a path): every end of its legs is refused from
        the walk's start, at the places its legs name, read at the cut as dec_hole reads them; other places stand (E §7
        step 1: cells no random fact reaches are shared). Where the places can't be read here, every place is."""
        from .evaluate import HoleSignal
        eng = self.eng
        places = set()
        try:
            for ln, _d in i.move.legs:
                for e in (ln.frm, ln.to):
                    p = eng.ev(e, i.env, ctx)
                    if isinstance(p, Thing):
                        places.add(p.name)
                    else:
                        raise ValueError
        except (TallyError, ValueError, HoleSignal):
            self.at(key, 'global_bad', err)
            return
        for p in sorted(places):
            self.at(key, 'placebad', (p, err, i))

    def note_leftout(self, i):
        mid = i.move.id
        n, first = self.leftout_regs.get(mid, (0, None))
        d = day_lo(i.day)
        self.leftout_regs[mid] = (n + 1, d if first is None or (d is not None and d < first) else first)
        for lg in i.legs:
            for p in (lg.frm, lg.to):
                if isinstance(p, Thing):
                    self.leftout_by_place.setdefault(p.name, set()).add(mid)

    def count_leftout(self, i):
        """A movement a projection leaves out (L §5.14.1): listed when dated in its window."""
        d = day_lo(i.day)
        if d is None:
            return
        self.counted_leftout.setdefault(i.move.id, []).append(d)
        self.leftout_insts.append((d, i.id))

    def sched_leg(self, i, lg, phase):
        d = lg.day
        osort = '%s|%s' % (i.move.id, i.order[3] if i.order else '')
        if isinstance(d, datetime.date):
            if d < self.start:
                return
            dk = getattr(self, 'defer', {}).get(i.move.id, {}).get(d.toordinal()) if phase == 2 else None
            if dk is not None:
                # after the decision that states `before` it (L §5.14.2)
                self.at((d.toordinal(), 3, dk[0], dk[1], 1, osort), 'leg', (i, lg))
                return
            self.at((d.toordinal(), phase, 0, osort, 0), 'leg', (i, lg))
        elif isinstance(d, Iv) and isinstance(d.lo, datetime.date):
            wid = '%s.%s' % (i.id, lg.name)
            q = frozenset([('date within %s not precise enough for a balance' % d.window_text(), i.id)])
            if isinstance(lg.amount, Hole):
                # every question the balance rests on is printed (L §8.1): the amount's too
                q = q | frozenset(lg.amount.qs)
            self.at((max(d.lo, self.start).toordinal(), phase, 0, osort, 0), 'imp_on', (i, lg, wid, q))
            self.at((d.hi.toordinal(), phase, 0, osort, 1), 'leg', (i, lg))
        elif isinstance(d, Hole):
            self.at(K_before(self.start), 'perm', (i, lg, frozenset(d.qs)))

    def sched_hole(self, i):
        d = i.day
        if isinstance(d, Hole):
            qs = frozenset(d.qs)
            for lg in i.legs:
                self.at(K_before(self.start), 'perm', (i, lg, qs))
            if not i.legs:
                self.at(K_before(self.start), 'global_hole', d)
            return
        qs = frozenset(i.exists.qs) if isinstance(i.exists, Hole) else EMPTY
        for lg in i.legs:
            self.sched_leg_hole(i, lg, qs)

    def sched_leg_hole(self, i, lg, qs):
        d = lg.day if isinstance(lg.day, datetime.date) else day_lo(i.day)
        if d is None or d < self.start:
            d = self.start
        osort = '%s|%s' % (i.move.id, i.order[3] if i.order else '')
        self.at((d.toordinal(), 2, 0, osort, 0), 'legq', (i, lg, qs))

    def sched_refused(self, i):
        d = day_lo(i.day)
        if d is None or d < self.start:
            d = self.start
        known = [lg for lg in i.legs if isinstance(lg.frm, Thing) or isinstance(lg.to, Thing)]
        # a leg whose place is itself refused could book into any place, so no read of a balance or a flows may
        # leave it out in silence: it refuses every one (L §8.4; a refused place is not a hole place, L §5.14 Legs)
        whole = all(isinstance(lg.frm, Thing) and isinstance(lg.to, Thing) for lg in i.legs)
        if known and whole:
            for lg in known:
                self.at((d.toordinal(), 2, 0, i.move.id, 0), 'legbad', (i, lg, i.err))
        else:
            self.at(K_before(d), 'global_bad', i.err)

    def sched_adj(self, a):
        r = a.remainder()
        txt = None
        if r is not None and not (isinstance(r, Num) and r.q == 0):
            txt = 'adjustment: %s by %s on %s (version %s), remaining %s' % (
                a.inst.id, fmt(a.delta), a.day, ', '.join(a.vids), fmt(r) if not isinstance(r, Hole) else fmt_hole(r))
        # the adjusted leg's places: for a change of places, those it had before V and those V gives it (L §5.14.3)
        ends = []
        for p in tuple((a.leg.frm, a.leg.to)) + tuple(getattr(a, 'new_ends', ())):
            if isinstance(p, Thing) and p.name not in [x.name for x in ends]:
                ends.append(p)
        if not a.settled:
            q = frozenset([('%s on %s changed after it was paid (version %s); is back pay due, and when?' % (
                a.inst.id, a.inst.day, ', '.join(a.vids)), a.inst.id)])
            d = max(a.day, self.start)
            for p in ends:
                self.at(K_before(d), 'changed', (p.name, a.leg.unit, q, a))
        if txt is not None:
            for p in ends:
                self.adj_regs.setdefault(p.name, []).append((a.day, ('adjustment', txt[len('adjustment: '):])))

    # -- the walk ---------------------------------------------------------------------------------------------------
    def run_to(self, day):
        if getattr(self, 'failed', None) is not None:
            raise self.failed
        if day > self.end:
            day = self.end
        try:
            while self.done < day:
                d = self.done + ONE_DAY
                self.walk_day(d)
                self.done = d
        except TallyError as e:
            self.failed = e
            raise

    def walk_day(self, d):
        acts = self.sched.pop(d.toordinal(), [])
        acts.sort(key=lambda a: a[0])
        self.touched = set()
        self.batch_legs = {}
        batch_open = True
        i = 0
        closing = None
        o_ = d.toordinal()
        start_open = True
        while i < len(acts):
            key, kind, obj = acts[i]
            if start_open and key[1] >= 1:
                # tally 2.1: the start readings' parts before the day's first batch (partswalk)
                start_open = False
                self.parts_flush((o_, 0, 999))
            if batch_open and key[1] > 2:
                # the day's first batch ends: its sign rule, taken together (L §5.10 point 1)
                batch_open = False
                self.check_batch(d, (d.toordinal(), 2, 9, '\uffff', 9))
                self.parts_flush((o_, 2, 999))
            if closing is None and key[1] >= 4:
                # the readings at the day's end: their pads are tested together (L §5.10 point 3; E §4.2 step 5)
                closing = self.grp_legs = {}
            if kind == 'decday':
                # every decision of this move due today, in this stratum
                mi, dd = obj
                self.run_decisions(mi, dd, key)
            else:
                self.act(key, kind, obj)
            i += 1
        if start_open:
            self.parts_flush((o_, 0, 999))
        if batch_open:
            self.check_batch(d, (d.toordinal(), 2, 9, '\uffff', 9))
            self.parts_flush((o_, 2, 999))
        if closing is not None:
            self.grp_legs = None
            o = d.toordinal()
            self.check_group(closing, (o, 4, 9, '\uffff', 9), lambda k: k[0] == o and k[1] == 4,
                             lambda place, b, legs: '%s takes %s %s' % (
                                 ', '.join(sorted(set(mv.id for mv, lg in legs))), place,
                                 'below 0' if self.m.kind_sign(self.m.things[place].base) == 'min0' else 'above 0'))
        self.parts_flush((o_, 99))
        self.close_day(d)
        self.parts_close(d)

    def check_batch(self, d, key):
        """L §5.10 The sign rule, point 1: after the day's first batch, taken together (its readings at the start,
        its observations but those booked at a decision's point, and its plans). A balance that breaks it is refused
        from there (RUN-PLACE `<place> is <b> after the day's observations and plans of <d>`), with the batch's legs at
        the place (E §4.2 step 5: each leg of that batch at the place; a refused leg isn't booked, and every read of
        it is refused)."""
        bl = self.batch_legs
        self.batch_legs = {}
        o = d.toordinal()
        self.check_group(bl, key, lambda k: k[0] == o and k[1] <= 2,
                         lambda place, b, legs: '%s is %s after the day\'s observations and plans of %s' % (place, b, d))

    def check_group(self, bl, key, match, msg):
        """The sign rule at one of its points (L §5.10; E §4.2 step 5, round 18 C2-1): bl {(place, unit): [(movement,
        leg)]} the legs booked together, key the point. Each place is tested against the balance the group leaves,
        all of its legs booked; a place that breaks its kind's sign rule is refused from there with the group's legs
        at it (RUN-PLACE), which aren't booked (match picks their entries); every place a refused leg touches is
        refused from there, so taking the legs back leaves no other balance of the group to test. Every place is
        tested before any leg is taken back, so the order of places changes nothing (snap11 tested them one by one
        in name order, taking back as it went)."""
        m = self.m

        def broken(place, u):
            st = self.pstate(place, u, key, raw=True)
            if st.val is None or st.hole or st.perm or st.imp or st.bad:
                return None
            sign = m.kind_sign(m.things[place].base)
            nv = st.val.q
            if (sign == 'min0' and nv < 0) or (sign == 'max0' and nv > 0):
                return nv
            return None
        places = sorted(bl, key=str)
        bad = [(pu, nv) for pu in places for nv in [broken(*pu)] if nv is not None]
        for (place, u), nv in bad:
            err = refuse('RUN-PLACE', msg(place, fmt(qty(nv, u)), bl[(place, u)]))
            for mv, lg in bl[(place, u)]:
                if (mv.id, lg.name) in self.bad_legs:
                    continue
                self.bad_legs[(mv.id, lg.name)] = err
                for p in (lg.frm, lg.to):
                    if isinstance(p, Thing):
                        self.unbook(p.name, u, mv, lg, match)
                for p in (lg.frm, lg.to):
                    if isinstance(p, Thing):
                        self.add(p.name, u, Ent(key, 'bad', err=err, mv=mv))

    def unbook(self, place, u, mv, lg, match):
        """Take back a leg the sign rule refuses at a check of a group: "A refused leg isn't booked" (E §4.2 step 5);
        match picks the group's entries by key."""
        log = self.logs.get((place, u))
        if log is None:
            return
        drop = [j for j, e in enumerate(log.ents) if e.kind == 'delta' and e.mv is mv and e.note == lg.name and
                match(e.key)]
        if not drop:
            return
        for j in drop:
            e = log.ents[j]
            if self.inside(place) and e.flow is not None and isinstance(e.flow, Num):
                other = lg.to if lg.frm is not None and isinstance(lg.frm, Thing) and lg.frm.name == place else lg.frm
                if not (isinstance(other, Thing) and self.inside(other.name)):
                    self.inv2_add(u, -e.flow.q)
        i0 = drop[0]
        kept = log.ents[:i0] + [e for j, e in enumerate(log.ents) if j >= i0 and j not in drop]
        prev = kept[i0 - 1].st if i0 > 0 else self.default(place, u)
        for j in range(i0, len(kept)):
            kept[j].st = apply(prev, kept[j], j)
            prev = kept[j].st
        log.ents = kept
        log.keys = [e.key for e in kept]
        del log.cum[i0:]

    def act(self, key, kind, obj):
        eng = self.eng
        if kind == 'read':
            self.apply_reading(key, *obj)
        elif kind == 'obs':
            o, lg = obj
            self.book_obs_leg(o, lg, key)
            # a reading after an observation (a running balance printed row by row, L §4.6)
            self.after_mv(o, key, lg)
        elif kind == 'leg':
            i, lg = obj
            reg = ()
            if i.status == 'past' and not i.obs and not self.same_observed(i):
                reg = (('unfulfilled', i.id),)
            self.book_leg(i, lg, key, reg=reg, coll=i.coll)
            self.after_mv(i, key)
        elif kind == 'imp_on':
            i, lg, wid, q = obj
            for p in (lg.frm, lg.to):
                if isinstance(p, Thing):
                    self.add(p.name, lg.unit or '*', Ent(key, 'imp_on', wid=wid, qs=q, mv=i, coll=i.coll, win=lg.day))
        elif kind == 'legq':
            i, lg, qs = obj
            q = set(qs)
            if isinstance(lg.amount, Hole):
                q |= lg.amount.qs
            if isinstance(lg.amount, TallyError):
                self.leg_bad(i, lg, lg.amount, key)
                return
            for p in (lg.frm, lg.to):
                if isinstance(p, Thing):
                    self.add(p.name, lg.unit or '*', Ent(key, 'hole', qs=frozenset(q), mv=i, coll=i.coll,
                                                         flow=Hole(q)))
        elif kind == 'covhole':
            place, q = obj
            self.add(place, '*', Ent(key, 'hole', qs=frozenset([q]), mv=None, coll=None, flow=Hole([q])))
        elif kind == 'legbad':
            i, lg, err = obj
            self.leg_bad(i, lg, err, key)
        elif kind == 'perm':
            i, lg, qs = obj
            for p in (lg.frm, lg.to):
                if isinstance(p, Thing):
                    self.add(p.name, lg.unit or '*', Ent(key, 'perm', qs=qs, mv=i, coll=i.coll, flow=Hole(qs)))
        elif kind == 'changed':
            place, unit, q, a = obj
            self.add(place, unit or '*', Ent(key, 'perm', qs=q, mv=a, coll=a.coll, note='changed'))
        elif kind == 'mark':
            place, unit, r, c = obj
            self.add(place, unit or '*', Ent(key, 'mark', reg=(r,), coll=c))
        elif kind == 'placehole':
            place, qs, i = obj
            self.add(place, '*', Ent(key, 'perm', qs=qs, mv=i, coll=i.coll, flow=Hole(qs)))
        elif kind == 'global_hole':
            self.add('*', '*', Ent(key, 'perm', qs=frozenset(obj.qs)))
        elif kind == 'global_bad':
            self.add('*', '*', Ent(key, 'bad', err=obj))
        elif kind == 'placebad':
            place, err, i = obj
            self.add(place, '*', Ent(key, 'bad', err=err, mv=i))

    def leg_bad(self, mv, lg, err, key):
        for p in (lg.frm, lg.to):
            if isinstance(p, Thing):
                self.add(p.name, lg.unit or '*', Ent(key, 'bad', err=err, mv=mv))

    def after_mv(self, mv, key, lg=None):
        """Apply the readings `after m` of movement mv just after it (L §4.6): once, after the leg that touches the
        reading's place (an observation's legs are booked one by one), else after its last leg."""
        rs = self.after.get(mv.id)
        if not rs:
            return
        k2 = key[:4] + (key[4] + 5,)
        for iid, place, unit, r in rs:
            if id(r) in self.after_keys:
                continue
            if lg is not None:
                touch = any(isinstance(p, Thing) and p.name == place for p in (lg.frm, lg.to))
                if not touch and lg is not mv.legs[-1]:
                    continue
            if day_lo(mv.day) is not None and day_lo(mv.day) < self.K:
                self.after_keys[id(r)] = k2
                self.apply_reading(k2, iid, place, unit, r)

    # -- same lines and consolidation (L §6.1, §5.19 SF-25) -----------------------------------------------------
    def same_pairs(self):
        """{movement id: (group, the same line naming it)} from the same lines naming movements. L §6.1 (round 14):
        "same is an equivalence: lines that share a side make one group"; "A group of movements is one movement"."""
        sp = getattr(self, '_same_pairs', None)
        if sp is not None:
            return sp
        parent = {}

        def find(x):
            while parent.get(x, x) != x:
                x = parent[x]
            return x
        lines = {}
        order = []
        for a, b, loc in getattr(self.m, 'same_moves', None) or ():
            for x in (a, b):
                if x not in order:
                    order.append(x)
            ra, rb = find(a), find(b)
            if ra != rb:
                parent[rb] = ra
            lines.setdefault(a, (a, b))
            lines.setdefault(b, (a, b))
        groups = {}
        for x in order:
            groups.setdefault(find(x), []).append(x)
        sp = {}
        for g in groups.values():
            for x in g:
                sp[x] = (g, lines[x])
        self._same_pairs = sp
        return sp

    def same_observed(self, i):
        """Whether a plan instance is stated same as an observation (L §6.1: the group is one movement, and it was
        observed), so it is no unfulfilled instance."""
        pr = self.same_pairs().get(getattr(i, 'id', None))
        if pr is None:
            return False
        for x in pr[0]:
            o = self.movement(x)
            if o is not None and o is not i and isinstance(o, ObsMv):
                return True
        return False

    def movement(self, mid):
        """A plan instance or an observation of this timeline by id (for a same line's partner)."""
        idx = getattr(self, '_mv_index', None)
        if idx is None:
            idx = {}
            for i in self.eng.plan_insts(self.tctx):
                idx[i.id] = i
            for o in self.eng.observations(self.tctx):
                idx[o.id] = o
            for q in sorted(self.m.moves):
                mi = self.m.moves[q]
                if mi.decision:
                    for i in self.eng.decision_instances(mi, self.tctx):
                        idx.setdefault(i.id, i)
            self._mv_index = idx
        return idx.get(mid)

    def consol(self, mv, lg):
        """What each end of a leg books under the same lines (L §6.1): None (as usual), 'skip' (counted once, by the
        movement it is the same as), or a Hole (HOLE-SAME)."""
        from .joint import owner_of, shown
        m = self.m
        J = getattr(m, 'joint', None)
        pairs = self.same_pairs()
        if J is None and not pairs:
            return (None, None)
        if isinstance(mv, PadMv) or getattr(mv, 'kind', None) == 'adjustments' or not hasattr(mv, 'id'):
            return (None, None)
        sides = (lg.frm, lg.to)
        pr = pairs.get(mv.id)
        if pr is not None:
            group, line = pr
            others = []
            waiting = []
            for x in group:
                if x == mv.id:
                    continue
                o = self.decs.get(x) or self.movement(x)
                if o is None:
                    # L §6.1 (round 14): "A side that names an instance that doesn't exist is refused, as a fulfils
                    # that names one is (RUN-RECORD)"; L §8.4 "same <p> = <q>: <p> doesn't exist"
                    ln = pairs.get(x, (None, (x, mv.id)))[1]
                    raise refuse('RUN-RECORD', 'same %s = %s: %s doesn\'t exist' % (shown(ln[0]), shown(ln[1]),
                                                                                  shown(x)))
                if getattr(o, 'decision', False) and not getattr(o, 'ran', False) and \
                        not getattr(mv, 'decision', False):
                    waiting.append(o)
                    continue
                others.append(o)
            out = []
            for si, p in enumerate(sides):
                if not isinstance(p, Thing):
                    out.append(None)
                    continue
                w = self.same_wait_for(mv, lg, p, waiting)
                if w is not None:
                    out.append(w)
                    continue
                # the members that book p in this unit, with the leg and side that touch it. L §6.1: "A place one of
                # them books (an observed row books where its statement covers, §5.19) takes that one's legs"; a plan
                # instance modelled past is never booked (L §5.19 Coverage, "its observations stand in for it where
                # they are booked"), so it books no place here (TX-F1, writer 10: snap14 let it take acc, then booked it
                # nowhere), and an observation books an end another version covers from that version's row
                touch = [(mv, lg, si)]
                for o in others:
                    if getattr(o, 'status', None) == 'modelled' and not getattr(o, 'decision', False):
                        continue
                    for g in o.legs:
                        for sj, q in enumerate((g.frm, g.to)):
                            if isinstance(q, Thing) and q.name == p.name and (g.unit or '*') == (lg.unit or '*'):
                                if isinstance(o, ObsMv):
                                    gd = g.day if isinstance(g.day, datetime.date) else o.day
                                    cov = self.eng.covering(q.name, gd, self.tctx) if isinstance(gd, datetime.date) \
                                        else None
                                    if cov and o.row.version not in cov:
                                        continue
                                touch.append((o, g, sj))
                if len(set(id(x[0]) for x in touch)) < 2:
                    if J is not None and owner_of(mv.id) == '' and owner_of(p.name) != '':
                        # the joint model's own movement into a member's place that no same links to that member's
                        # own movement there (L §6.1 SF-32): not booked, a hole naming it, listed
                        q = '%s crosses to %s with no link' % (shown(mv.id), owner_of(p.name))
                        h = LeftHole([(q, shown(mv.id))])
                        h.left_out = (getattr(getattr(mv, 'move', None), 'id', None) or mv.id, lg.day)
                        out.append(h)
                        continue
                    out.append(None)      # "At a place only one of them touches, that one books as written"
                    continue
                agree = all(sj == si and vkey(g.amount) == vkey(lg.amount) and g.day == lg.day
                            for o, g, sj in touch[1:])
                if agree:
                    # L §6.1 (round 14): "it is booked once, at the point in the day's order of the latest of them"
                    latest = max(touch, key=lambda t: same_book_key(t[0], t[1]))
                    out.append(None if latest[0] is mv else 'skip')
                    continue
                # "where they give that place different amounts or days, it is a hole movement there (HOLE-SAME)
                # that may fall on either day": each member's end there is that hole
                # L §8.1 HOLE-SAME: "<p> and <q> are one, but <x> differs on <d>: <a> (<row>) and <b> (<row>)", the
                # members in the order the same lines write them
                pos = {x: n for n, x in enumerate(group)}
                touch.sort(key=lambda t: pos.get(t[0].id, 0))
                names = ' and '.join(shown(t[0].id) for t in touch)
                days = sorted(set(g.day for o, g, sj in touch if isinstance(g.day, datetime.date)))
                parts = ' and '.join('%s (%s)' % (fmt(g.amount) if not isinstance(g.amount, TallyError) else 'refused',
                                                  shown(o.id)) for o, g, sj in touch)
                q = '%s are one, but the leg at %s differs on %s: %s' % (
                    names, shown(p.name), ', '.join(str(d) for d in days), parts)
                out.append(Hole([(q, ', '.join(shown(t[0].id) for t in touch))]))
            return tuple(out)
        if J is None:
            return (None, None)
        x = owner_of(mv.id)
        out = []
        for p in sides:
            if not isinstance(p, Thing):
                out.append(None)
                continue
            rep = p.name
            if x == '' and owner_of(rep) != '':
                # L §6.1 The joint model's own movements (OQ-7, SF-32): "A movement of the joint model's own versions
                # into a member's place changes no member's answer, since the member is closed: it is booked there
                # only where same or fulfils links it to the member's own movement. Otherwise it is listed in the
                # joint model's register (left out:, §8.2), and a joint total over that place is a hole naming it";
                # L §8.1 HOLE-SAME `<movement> crosses to <member> with no link`. (A same line naming it is handled
                # above; a fulfils of a member's instance books as an observation does.)
                mem = owner_of(rep)
                q = '%s crosses to %s with no link' % (shown(mv.id), mem)
                h = LeftHole([(q, shown(mv.id))])
                h.left_out = (getattr(getattr(mv, 'move', None), 'id', None) or mv.id, lg.day)
                out.append(h)
                continue
            if rep not in J.groups:
                out.append(None)
                continue
            ys = sorted(set(o for (o, r) in J.owner_place if r == rep and o != x))
            if not ys:
                out.append(None)
                continue
            q = '%s crosses to %s with no link' % (shown(mv.id), ', '.join(y or m.header.id for y in ys))
            out.append(Hole([(q, shown(mv.id))]))
        return tuple(out)

    def same_wait_for(self, mv, lg, p, waiting):
        """An end at place p of a movement stated same as a decision instance not yet run (L §6.1: "a place two of them
        book ... takes it at the point of the last of them in the day's order (§5.14.2), since a decision's amount is
        known only at its point, and where they give it different amounts or days it is a hole movement there
        (HOLE-SAME) that may fall on either day"): ('wait', decision) to book it at the decision's point, a Hole where
        their days differ, None where the decision doesn't book p."""
        from .joint import shown
        for dec in waiting:
            dday = day_lo(dec.day)
            if dday is None:
                return None
            changes = self.inst_places(dec, dday)[0]
            if changes is not None and p.name not in changes:
                continue
            if isinstance(lg.day, datetime.date) and lg.day == dday:
                return ('wait', dec)
            q = '%s and %s are one, but the leg at %s differs on %s: %s (%s) and its day %s (%s)' % (
                shown(mv.id), shown(dec.id), shown(p.name), lg.day,
                fmt(lg.amount) if not isinstance(lg.amount, TallyError) else 'refused', shown(mv.id), dday,
                shown(dec.id))
            return Hole([(q, '%s, %s' % (shown(mv.id), shown(dec.id)))])
        return None

    def same_release(self, run, key):
        """At a decision's point, the ends that waited for it (same_wait_for): booked by the decision where it books
        the place (once, at the latest point, or the HOLE-SAME hole its booking found), else by their own movement
        now; refused where the instance doesn't exist (L §6.1 RUN-RECORD)."""
        from .joint import shown
        pend = getattr(self, 'same_waiting', {}).pop(run.id, [])
        for mv, lg, si, reg, coll in pend:
            p = (lg.frm, lg.to)[si]
            if run.exists is False:
                self.leg_bad(mv, lg, refuse('RUN-RECORD', 'same %s = %s: %s doesn\'t exist' % (
                    shown(mv.id), shown(run.id), shown(run.id))), key)
                continue
            hit = any(isinstance(q, Thing) and q.name == p.name and (g.unit or '*') == (lg.unit or '*')
                      for g in (run.legs or ()) for q in (g.frm, g.to)) and not getattr(run, 'modelled', False)
            if hit:
                continue
            ends = (si == 0, si == 1)
            self.book_leg(mv, lg, key[:4] + (key[4] + 1,), reg=reg, coll=coll, ends=ends)

    # -- booking (E §4.2 step 5) -----------------------------------------------------------------------------------
    def book_leg(self, mv, lg, key, reg=(), coll=None, ends=(True, True)):
        """Book a leg's two ends (or the ends asked for); RUN-LIFESPAN, RUN-PLACE refuse it from that point."""
        m = self.m
        amt = lg.amount
        if isinstance(amt, TallyError):
            self.leg_bad(mv, lg, amt, key)
            return
        cs = self.consol(mv, lg)
        if cs != (None, None):
            ends = list(ends)
            for si, (p, c) in enumerate(zip((lg.frm, lg.to), cs)):
                if c == 'skip':
                    ends[si] = False
                elif isinstance(c, tuple) and c[0] == 'wait':
                    if ends[si]:
                        self.__dict__.setdefault('same_waiting', {}).setdefault(c[1].id, []).append(
                            (mv, lg, si, reg, coll))
                    ends[si] = False
                elif isinstance(c, Hole) and ends[si] and isinstance(p, Thing):
                    # the end is a hole naming the link that is missing, or the two rows that disagree (HOLE-SAME)
                    self.inv2_add(lg.unit or '*', None)
                    r2 = reg
                    lo = getattr(c, 'left_out', None)
                    if lo is not None:
                        # listed under every result that reads the place (L §8.2 left out:, §6.1)
                        r2 = tuple(reg) + (('left out joint', lo[0], str(lo[1])),)
                    self.add(p.name, lg.unit or '*', Ent(key, 'hole', qs=frozenset(c.qs), mv=mv, coll=coll, flow=c,
                                                         reg=r2, note=lg.name))
                    ends[si] = False
            ends = tuple(ends)
            if not any(ends):
                return
        sides = [(lg.frm, -1, ends[0]), (lg.to, 1, ends[1])]
        wid = '%s.%s' % (mv.id, lg.name) if isinstance(lg.day, Iv) else None
        if isinstance(amt, Hole):
            self.inv2_add(lg.unit or '*', None)
            for p, s, on in sides:
                if on and isinstance(p, Thing):
                    self.add(p.name, lg.unit or '*', Ent(key, 'hole', qs=frozenset(amt.qs), mv=mv, coll=coll,
                                                         flow=amt, reg=reg, wid=wid, win=lg.day if wid else None,
                                                         note=lg.name, sgn=s, leg=lg))
            return
        if amt.q == 0:
            return
        day = kday(key)
        u = lg.unit
        err = None
        for si, (p, s, on) in enumerate(sides):
            if not on or not isinstance(p, Thing):
                continue
            th = m.things[p.name]
            al = self.alive_day(th, day)
            if isinstance(al, Hole):
                # a leg into or out of a place stated same on a day its sides' lifespans disagree on: RUN-LIFESPAN on
                # one side, booked on the other, so the place takes a hole movement there (L §6.1, HOLE-SAME)
                self.inv2_add(u or '*', None)
                self.add(p.name, u or '*', Ent(key, 'hole', qs=frozenset(al.qs), mv=mv, coll=coll, flow=al, reg=reg,
                                               note=lg.name))
                sides[si] = (p, s, False)
                continue
            if not al:
                err = refuse('RUN-LIFESPAN', '%s moves into or out of %s outside its lifespan' % (
                    '%s.%s' % (mv.id, lg.name), p.name))
                break
        if not any(on for p, s, on in sides):
            return
        batch = key[1] <= 2 and getattr(self, 'batch_legs', None) is not None
        # in the day's first batch the sign rule waits for the whole batch (L §5.10 point 1); a decision's legs, with
        # the observations booked at its point, wait for one another, as the closing readings' pads do (L §5.10 points
        # 2 and 3; E §4.2 step 5, round 18 C2-1)
        grp = self.batch_legs if batch else getattr(self, 'grp_legs', None)
        if err is None and grp is not None:
            for p, s, on in sides:
                if on and isinstance(p, Thing):
                    grp.setdefault((p.name, u), []).append((mv, lg))
        if err is None and grp is None:
            for p, s, on in sides:
                if not on or not isinstance(p, Thing):
                    continue
                st = self.pstate(p.name, u, key, raw=True)
                if st.val is None or st.hole or st.perm or st.imp or st.bad:
                    continue
                nv = st.val.q + s * amt.q
                sign = m.kind_sign(m.things[p.name].base)
                if sign == 'min0' and nv < 0:
                    hold = m.things[p.name].base in m.kinds['holdings'].subtree(m)
                    err = refuse('RUN-PLACE', '%s takes %s below 0%s' % (mv.id, p.name, ' units' if hold else ''))
                    break
                if sign == 'max0' and nv > 0:
                    err = refuse('RUN-PLACE', '%s takes %s above 0' % (mv.id, p.name))
                    break
        if err is not None:
            # "the place's balance is refused from that point ... and so is every read of the refused leg itself
            # (flows of its move over a span that holds it)" (E §4.2 step 5, round 10)
            self.bad_legs[(mv.id, lg.name)] = err
            for p, s, on in sides:
                if isinstance(p, Thing):
                    self.add(p.name, u, Ent(key, 'bad', err=err, mv=mv))
            return
        # conservation (E §9 invariant 1): a leg booked whole adds its two ends; a pad's other end is its reading's
        # reset; an observed leg booked end by end from two statements is the WARN-LEGS case (E §6.1), left out
        whole = all(on for p, s, on in sides) or isinstance(mv, PadMv)
        drop = self.labels_drop(lg)
        for i_, (p, s, on) in enumerate(sides):
            if on and isinstance(p, Thing):
                f = qty(s * amt.q, u)
                r_ = reg
                if drop and s == 1:
                    # tally 2.1 (L §3.6, §8.2, §8.5): WARN-LABELS, listed in the register of every result whose reads
                    # reach the receiving place from that leg on
                    r_ = tuple(reg) + (('labels', p.name, str(kday(key)), mv.id),)
                self.add(p.name, u, Ent(key, 'delta', mv=mv, flow=f, reg=r_, coll=coll, wid=wid,
                                        win=lg.day if wid else None, note=lg.name))
                if whole:
                    self.cons[u] = self.cons.get(u, 0) + s * amt.q
                if self.inside(p.name):
                    # invariant 2: a whole leg changes worth only across the boundary; a leg booked by its ends is
                    # left out, as from invariant 1
                    other = sides[1 - i_][0]
                    crosses = isinstance(other, Thing) and not self.inside(other.name)
                    if crosses or not whole:
                        self.inv2_add(u, s * amt.q)

    def book_obs_leg(self, o, lg, key):
        """Each end of an observed leg is booked from the observation of the version that covers its place that day;
        an end no version covers, from the observation itself; a leg with no covered end isn't booked (E §6.1)."""
        eng = self.eng
        d = lg.day
        if not d < self.K:
            return
        if o.target is not None and o.target.decision and day_lo(o.target.day) is not None and \
                d == day_lo(o.target.day):
            # one that fulfils a decision instance dated its day is booked at the instance's point; every other on
            # its own day, one dated before or after the instance it fulfils included (E §4.2 step 1, L §5.19)
            return
        self.book_obs_now(o, lg, key)

    def book_obs_now(self, o, lg, key):
        eng = self.eng
        d = lg.day
        if o.err is not None:
            self.leg_bad(o, lg, o.err, key)
            return
        # every observation is booked on its own day (L §5.19, E §4.2 step 1): each end of each leg from the
        # observation of the version that covers its place that day, an end no version covers from this one (E §6.1)
        ends = []
        for p in (lg.frm, lg.to):
            cov = eng.covering(p.name, d, self.tctx)
            ends.append(o.row.version in cov if cov else True)
        self.book_leg(o, lg, key, coll=o.coll, ends=tuple(ends))

    # -- readings (E §6.1) -----------------------------------------------------------------------------------------
    def apply_reading(self, key, iid, place, unit, r):
        m = self.m
        val = r.val
        c = _coll()
        c.rows[r.sid] = r
        if r.assumed is not None:
            c.reg.add(('assumed', r.assumed, ''))
        it = instant_text(r)
        if isinstance(val, tuple) and val[0] == 'hole':
            rv = Hole([(val[1], '%s[%s]' % (iid, place))])
        elif val is NONE:
            # L §5.19: "A reading whose value is none states no balance: it neither pads nor anchors, and with replace
            # it removes the reading it replaces"
            return
        else:
            rv = val
        if iid == 'opening':
            # invariant 2: an opening states the place's worth; its change counts as stated
            if not isinstance(rv, Hole) and self.inside(place):
                prev = self.pstate(place, unit, key, raw=True)
                if prev.val is None:
                    self.inv2_add(unit, rv.q)
                elif not (prev.hole or prev.perm or prev.imp or prev.bad):
                    self.inv2_add(unit, rv.q - prev.val.q)
                else:
                    self.inv2_add(unit, None)
            elif self.inside(place):
                self.inv2_add(unit, None)
            if isinstance(rv, Hole):
                self.add(place, unit, Ent(key, 'reset', val=qty(0, unit), qs=frozenset(rv.qs), coll=c,
                                          note='opening', row=r))
            else:
                self.add(place, unit, Ent(key, 'reset', val=rv, coll=c, note='opening', row=r))
            return
        day = kday(key)
        if not day < self.K:
            return
        role = r.version.gap_roles.get(place)
        if role is None:
            raise LoadError('LOAD-GAPROLE', '%s states balance readings of %s with neither pad nor anchor'
                            % (r.version.id, place))
        before = self.pstate(place, unit, key, raw=True)
        booked = self.st_value(before, place, unit)
        gap = self.sub(rv, booked)
        # the gap is the reading less the booked balance at its instant (L §5.19), so it rests on what that balance
        # rests on (L §8.2: the register is transitive), and so does a pad, whose amount is the gap
        bc = self.booked_coll(place, unit, key)
        bc.merge(c)
        self.gaps.setdefault((place, day), []).append((key, gap, r, role, unit, bc))
        if self.inside(place):
            # invariant 2: an anchor's gap, or a pad's from outside, moves worth (a pad from an inside place moves it
            # between two inside places)
            if role[0] == 'anchor' or not self.inside(role[1].name):
                self.inv2_add(unit, gap.q if isinstance(gap, Num) else None)
        if role[0] == 'anchor':
            txt = 'anchor: %s %s, stated %s, difference %s' % (place, it, fmt(rv) if not isinstance(rv, Hole)
                                                               else fmt_hole(rv), gtext(gap))
            self.add(place, unit, Ent(key, 'reset', val=rv if not isinstance(rv, Hole) else qty(0, unit),
                                      qs=frozenset(rv.qs) if isinstance(rv, Hole) else EMPTY, coll=c,
                                      reg=(('anchor', txt[len('anchor: '):]),), note='anchor', row=r))
            return
        y = role[1]
        if isinstance(gap, TallyError):
            amt = gap
        elif isinstance(gap, Hole):
            amt = gap
        else:
            amt = qty(abs(gap.q), unit)
        positive = isinstance(gap, Num) and gap.q > 0
        frm, to = (Thing(y.name), Thing(place)) if (positive or not isinstance(gap, Num)) else \
            (Thing(place), Thing(y.name))
        pad = PadMv(r.version.id, place, it, amt, frm, to, day, unit)
        pad.coll = bc
        pad.point = key
        self.pads.append(pad)
        txt = 'pad: %s %s, %s of %s' % (place, it, pad.id, gtext(gap))
        reg = (('pad', txt[len('pad: '):]),)
        lg = pad.legs[0]
        # the other side: an ordinary leg
        yi = 0 if frm.name == y.name else 1
        ends = (yi == 0, yi == 1)
        self.book_leg(pad, lg, key, reg=reg, coll=bc, ends=ends)
        # x's balance after the instant is the reading
        flow = gap if not isinstance(gap, TallyError) else None
        if isinstance(gap, Num) and (pad.id, lg.name) not in self.bad_legs:
            # the reading's end of the pad, beside its other end; a pad refused as it is booked (RUN-PLACE, "that
            # reading's pad", E §4.2 step 5) books neither (writer 8, sixth pass: was ENGINE-INVARIANT)
            self.cons[unit] = self.cons.get(unit, 0) + gap.q
        self.add(place, unit, Ent(key[:4] + (key[4] + 1,), 'reset',
                                  val=rv if not isinstance(rv, Hole) else qty(0, unit),
                                  qs=frozenset(rv.qs) if isinstance(rv, Hole) else EMPTY, coll=c, reg=reg,
                                  mv=pad, flow=flow, note='pad', row=r))

    def booked_coll(self, place, unit, key):
        """The register of a place's booked balance at an instant (as a balance read gathers it, mreads.read_state)."""
        eng = self.eng
        c = eng.push()
        try:
            eng.merge_cum(self, place, unit, key)
            d = kday(key)
            for (ad, e) in self.adj_regs.get(place, ()):
                if ad <= d:
                    eng.reg(e)
        finally:
            eng.stack.pop()
        return c

    def sub(self, a, b):
        if isinstance(b, TallyError):
            return b
        if isinstance(a, Hole) or isinstance(b, Hole):
            return merge(a, b)
        return add_q(a, b, -1)

    def st_value(self, st, place, unit):
        """A state's value: the quantity, a hole with its questions, or the refusal (as a value)."""
        if st.bad is not None:
            return st.bad
        qs = set(st.hole) | set(st.perm)
        for w, q in st.imp:
            qs |= q
        if st.val is None:
            qs.add(self.anchor_question(place))
        if qs:
            return Hole(qs)
        return st.val

    def anchor_question(self, place):
        """HOLE-ANCHOR (L §8.1): `<place> has no balance before <d>`, d the day of its first reading."""
        days = []
        for iid, p, u, r in self.eng.readings(self.tctx):
            if p == place and r.instant[0] != 'after':
                days.append(r.instant[0])
        if not days:
            base = self.base
            while base is not None and not days:
                days = [r.instant[0] for iid, p, u, r in self.eng.readings(base.tctx) if p == place and
                        r.instant[0] != 'after']
                base = base.base
        if not days:
            # L §8.1 HOLE-ANCHOR: "with no reading at all, <place> has no opening or stated reading"
            return ('%s has no opening or stated reading' % place, place)
        return ('%s has no balance before %s' % (place, min(days)), place)

    # -- decisions (E §4.2 step 3) -------------------------------------------------------------------------------
    def run_decisions(self, mi, d, key):
        eng = self.eng
        m = self.m
        insts = list(self.decday.get((mi.id, d), ()))
        if not insts:
            return
        # decisions of an unordered cycle due on one day: ordered by the places their legs give that day (E §4.2
        # step 3, L §5.14.2), RUN-ORDER only between instances that read what each other changes there
        for g in getattr(m, 'order_groups', []):
            if mi.id in g:
                others = [q for q in g if q != mi.id and self.decday.get((q, d))]
                if others:
                    self.run_group(sorted(g), d, key)
                    return
        if len(insts) > 1 and mi.decl.tie:
            insts = self.tie_order(mi, insts, d)
        group = []
        for n, i in enumerate(insts):
            k = key[:3] + ('%s|%06d' % (mi.id, n), 0)
            run = self.run_one(i, k)
            group.append(run)
        if len(group) > 1 and not mi.decl.tie and mi.self_reads:
            # instances of one move that read each other's places: their order is a tie (L §5.14.2)
            for a in range(len(group)):
                for b in range(a + 1, len(group)):
                    ra, rb = group[a], group[b]
                    if (rb.live_places & ra.changed) or (ra.live_places & rb.changed):
                        err = refuse('RUN-TIE', '%s has a tie at %s that changes the result, and no ties by' % (
                            mi.id, ', '.join(r.id for r in group)))
                        k = key[:3] + ('%s|999999' % mi.id, 9)
                        for r in group:
                            for p in r.changed:
                                self.add(p, m.place_unit(m.things[p]) or '*', Ent(k, 'bad', err=err))
                        return

    def refused_run(self, i, key, err):
        """A decision instance refused before it runs (RUN-ORDER): every read of it is refused too, flows of its move
        over a span that holds it included (E §4.2 step 5; L §8.4 a refusal spreads to what reads it)."""
        import copy
        run = copy.copy(i)
        run._mv = None
        run.legs = []
        run.attrs = {}
        run.ran = True
        run.point = key
        run.read_places = set()
        run.live_places = set()
        run.changed = set()
        run.err = err
        run.status = 'refused'
        self.decs[i.id] = run
        return run

    def inst_places(self, i, d):
        """(the places i's legs give on d, the places it declares it reads, the decisions whose legs it reads): None
        for a set that can't be read here (a hole, a refusal), which then meets every place."""
        from .evaluate import HoleSignal
        from .moves import law_days
        eng = self.eng
        m = self.m
        mi = i.move
        lds = law_days(i.law) if i.law is not None else [None]
        ctx = self.tctx.but(cut=min(d, self.K), law=lds[0])
        changes = set()
        try:
            for ln, _d in mi.legs:
                for e in (ln.frm, ln.to):
                    p = eng.ev(e, i.env, ctx)
                    if isinstance(p, Thing):
                        changes.add(p.name)
                    elif not isinstance(p, Hole):
                        changes = None
                        break
                if changes is None:
                    break
        except (TallyError, HoleSignal):
            changes = None
        reads = set()
        moves = set()
        for r in mi.decl.reads:
            rr = getattr(r, '_r', None)
            if rr is not None and rr[0] == 'move':
                moves.add(rr[1])
                continue
            if rr is not None and rr[0] == 'kind':
                moves |= set(q for q, x in m.moves.items() if x.decision and x.kind in m.kinds[rr[1]].subtree(m))
                continue
            if rr is not None and rr[0] == 'placesec':
                reads |= set(n for n, t in m.things.items() if getattr(getattr(t, 'place', None), 'id', None) ==
                             rr[1])
                continue
            try:
                p = eng.ev(r, i.env, ctx)
            except (TallyError, HoleSignal):
                p = None
            if isinstance(p, Thing):
                reads.add(p.name)
            else:
                reads = None
                break
        return changes, reads, moves

    def run_group(self, g, d, key):
        """The instances of an unordered cycle's decisions due on d (E §4.2 step 3): an instance whose `when` is false
        orders nothing; one that reads a place another changes that day runs after it, and one that reads another's
        legs; two that read what each other changes there are refused (`RUN-ORDER`), or, of one move, go by its
        `ties by` (`RUN-TIE` without one); the others are independent and run in the text order of their ids."""
        done = getattr(self, 'groups_done', None)
        if done is None:
            done = self.groups_done = set()
        tag = (tuple(g), d)
        if tag in done:
            return
        done.add(tag)
        eng = self.eng
        m = self.m
        insts = []
        for q in g:
            mq = m.moves[q]
            qi = list(self.decday.get((q, d), ()))
            if len(qi) > 1 and mq.decl.tie:
                qi = self.tie_order(mq, qi, d)
            insts.extend(qi)
        # when, read before any of them runs; a when whose reads meet what another changes isn't settled before the
        # other runs, and orders as true
        from .moves import law_days
        info = []
        for i in insts:
            ch, rd, mv = self.inst_places(i, d)
            wf, seen = False, set()
            mi = i.move
            if mi.decl.when is not None:
                lds = law_days(i.law) if i.law is not None else [None]
                ctx = self.tctx.but(cut=min(d, self.K), walk=self, wp=key, law=lds[0])
                old = self.wp, self.reads, self.live
                self.wp = key
                self.reads = set()
                self.live = set()
                try:
                    wf = eng.ev(mi.decl.when, i.env, ctx) is False
                    seen = set(self.live)
                except TallyError:
                    wf = False
                finally:
                    self.wp, self.reads, self.live = old
            info.append((i, ch, rd, mv, wf, seen))
        n = len(info)
        off = []
        for x in range(n):
            ok = info[x][4]
            for y in range(n):
                if y != x and ok and (info[y][1] is None or info[x][5] & info[y][1]):
                    ok = False
            off.append(ok)

        def reads_of(x, y):
            # x reads what y changes there, or y's legs
            ix, chx, rdx, mvx, _, _ = info[x]
            iy, chy, rdy, mvy, _, _ = info[y]
            if iy.move.id in mvx:
                return True
            if rdx is None or chy is None:
                return True
            return bool(rdx & chy)
        after = {x: set() for x in range(n)}
        for x in range(n):
            for y in range(n):
                if x != y and not off[x] and not off[y] and reads_of(x, y):
                    after[x].add(y)
        # a before or after line between two members orders their instances (L §5.14.2 Cycles: "the writer states
        # before m or after m on one of them"); the component only lifts the order the reads alone don't give
        wo = getattr(m, 'written_order', ())
        for x in range(n):
            for y in range(n):
                if x != y and not off[x] and not off[y] and (info[x][0].move.id, info[y][0].move.id) in wo:
                    after[x].add(y)
                    after[y].discard(x)
        bad = {}
        for x in range(n):
            for y in sorted(after[x]):
                if x not in after[y] or y < x:
                    continue
                ix, iy = info[x][0], info[y][0]
                if ix.move.id == iy.move.id:
                    if ix.move.decl.tie:
                        # one move's instances go by its ties by
                        if insts.index(ix) < insts.index(iy):
                            after[x].discard(y)
                        else:
                            after[y].discard(x)
                        continue
                    err = refuse('RUN-TIE', '%s has a tie at %s that changes the result, and no ties by' % (
                        ix.move.id, d))
                else:
                    err = refuse('RUN-ORDER', 'decisions %s read each other on %s' % (
                        ' and '.join(sorted([ix.move.id, iy.move.id])), d))
                bad.setdefault(x, err)
                bad.setdefault(y, err)

        def rank(x):
            return (info[x][0].move.id, insts.index(info[x][0]))
        order = []
        left = set(range(n)) - set(bad)
        while left:
            ready = [x for x in sorted(left, key=rank) if not (after[x] & left)]
            if not ready:
                err = refuse('RUN-ORDER', 'decisions %s read each other on %s' % (
                    ' and '.join(sorted(set(info[y][0].move.id for y in left))), d))
                for x in left:
                    bad[x] = err
                break
            order.append(ready[0])
            left.discard(ready[0])
        seq = 0
        for x in sorted(bad, key=rank):
            k = key[:3] + ('%s|%06d|%s' % (g[0], seq, info[x][0].move.id), 0)
            seq += 1
            ch = info[x][1]
            if ch:
                for p in sorted(ch):
                    self.add(p, m.place_unit(m.things[p]) or '*', Ent(k, 'bad', err=bad[x]))
            else:
                self.add('*', '*', Ent(k, 'bad', err=bad[x]))
            self.refused_run(info[x][0], k, bad[x])
        for x in order:
            k = key[:3] + ('%s|%06d|%s' % (g[0], seq, info[x][0].move.id), 0)
            seq += 1
            self.run_one(info[x][0], k)

    def tie_order(self, mi, insts, d):
        eng = self.eng
        scored = []
        from .moves import law_days
        for i in insts:
            lds = law_days(i.law)
            ctx = self.tctx.but(cut=min(d, self.K), law=lds[0])
            vals = [eng.ev(x, i.env, ctx) for x in mi.decl.tie]
            for dd in lds[1:]:
                v2 = [eng.ev(x, i.env, ctx.but(law=dd)) for x in mi.decl.tie]
                if [vkey(v) for v in v2] != [vkey(v) for v in vals]:
                    vals = [Hole([('date within %s not precise enough for the ties by of %s' % (
                        i.law.window_text(), mi.id), i.id)])]
                    break
            scored.append((vals, i))
        out = []
        for vals, i in scored:
            out.append((vals, i))

        def cmpkey(x):
            ks = []
            for v in x[0]:
                if isinstance(v, Hole):
                    raise refuse('RUN-TIE', '%s: its ties by is a hole' % mi.id)
                ks.append(vkey(v))
            return ks
        out.sort(key=cmpkey)
        for a, b in zip(out, out[1:]):
            if cmpkey(a) == cmpkey(b):
                raise refuse('RUN-TIE', '%s has a tie at %s that changes the result, and its ties by doesn\'t break '
                             'it' % (mi.id, d))
        return [i for v, i in out]

    def run_one(self, inst, key):
        """Run a decision instance at its point (E §4.2 step 3), then test the sign rule after its legs together (or
        the observations booked at its point), "that decision's legs at the place" refused (L §5.10 point 2; E §4.2
        step 5, round 18 C2-1); then the readings after it, each tested as it is booked."""
        prev = getattr(self, 'grp_legs', None)
        self.grp_legs = None
        try:
            run = self.run_one_body(inst, key)
            grp = self.grp_legs
        finally:
            self.grp_legs = prev
        if grp:
            self.check_group(grp, key[:4] + (2,), lambda k: k[:4] == key[:4],
                             lambda place, b, legs: '%s takes %s %s' % (
                                 run.id, place, 'below 0' + (' units' if self.m.things[place].base in
                                                             self.m.kinds['holdings'].subtree(self.m) else '')
                                 if self.m.kind_sign(self.m.things[place].base) == 'min0' else 'above 0'))
        # tally 2.1: the decision's takes and arrivals at its point (E §4.2 step 3, partswalk)
        self.parts_flush(tuple(key[:4]) + (999,))
        if getattr(run, '_after', False):
            self.after_mv(run, key)
        return run

    def run_one_body(self, inst, key):
        """Run a decision instance at its point (E §4.2 step 3): its when and legs with running reads here."""
        import copy
        eng = self.eng
        m = self.m
        mi = inst.move
        t = inst.day
        run = copy.copy(inst)
        run._mv = None
        run.legs = []
        run.attrs = {}
        run.ran = True
        run.point = key
        run.read_places = set()
        run.live_places = set()
        run.changed = set()
        run.coll = _coll()
        run.coll.merge(inst.coll)
        self.decs[inst.id] = run
        ctx = self.tctx.but(cut=min(t, self.K), walk=self, wp=key)
        old = self.wp, self.reads, getattr(self, 'less_obs', None), self.live
        self.wp = key
        self.reads = run.read_places
        self.live = run.live_places
        # its own observations booked before its day (step 1), which its computation doesn't see (E §4.2 step 3)
        own = [] if self.proj else [lg for o in eng.obs_links_for(inst.id, self.tctx) if o.err is None
                                    for lg in o.legs
                                    if isinstance(lg.day, datetime.date) and lg.day < t and lg.day < self.K and
                                    lg.day >= self.start]
        self.less_obs = own or None
        eng.stack.append(run.coll)
        try:
            try:
                b = eng.body_over_law(mi, inst.env, run, ctx, inst.law, inst.exists)
                run.exists, run.legs, run.attrs, run.forp, run.fulfils = b
            except TallyError as e:
                if load_now(e):
                    raise
                run.err = e
        finally:
            eng.stack.pop()
            self.wp, self.reads, self.less_obs, self.live = old
        # what this decision books at its point from here is one group for the sign rule (L §5.10 point 2)
        self.grp_legs = {}
        run.obs = eng.obs_links_for(inst.id, self.tctx)
        if run.err is not None:
            known = [lg for lg in run.legs if isinstance(lg.frm, Thing) or isinstance(lg.to, Thing)]
            if known:
                for lg in known:
                    self.leg_bad(run, lg, run.err, key)
            else:
                self.add('*', '*', Ent(key, 'bad', err=run.err))
            self.same_release(run, key)
            return run
        if run.exists is False:
            run.status = 'none'
            self.book_own_obs(run, key)
            self.same_release(run, key)
            return run
        for lg in run.legs:
            for p in (lg.frm, lg.to):
                if isinstance(p, Thing):
                    run.changed.add(p.name)
        # a decision on a covered day before K, or one an observation the cut sees fulfils, is modelled past (E §4.2
        # step 3)
        if not self.proj and ((t < self.K and eng.touches_covered(run, self.tctx)) or run.obs):
            run.status = 'modelled'
            run.modelled = True
            self.modelled.append(run)
            self.book_own_obs(run, key)
            self.same_release(run, key)
            return run
        if isinstance(run.exists, Hole):
            run.status = 'hole'
            for lg in run.legs:
                q = set(run.exists.qs)
                if isinstance(lg.amount, Hole):
                    q |= lg.amount.qs
                for p in (lg.frm, lg.to):
                    if isinstance(p, Thing):
                        self.add(p.name, lg.unit or '*', Ent(key, 'hole', qs=frozenset(q), mv=run, coll=run.coll,
                                                             flow=Hole(q)))
            self.same_release(run, key)
            return run
        if self.m.kinds[run.kind].certainty == 'uncertain' and not run.obs:
            run.status = 'uncertain'
            self.uncertain.append(run)
            self.same_release(run, key)
            return run
        run.status = 'past' if t < self.K else 'plan'
        # an instance stated same as an observation was observed: the group is one movement (L §6.1)
        reg = (('unfulfilled', run.id),) if (t < self.K and not run.obs and not self.same_observed(run)) else ()
        for lg in run.legs:
            if isinstance(lg.day, datetime.date) and lg.day != t:
                if lg.day > t:
                    self.at((lg.day.toordinal(), 2, 0, run.id, 0), 'leg', (run, lg))
                    continue
            self.book_leg(run, lg, key, reg=reg, coll=run.coll)
        self.book_own_obs(run, key)
        self.same_release(run, key)
        run._after = True   # the readings after it, once its legs are tested (run_one)
        return run

    def book_own_obs(self, run, key):
        if self.proj:
            return
        for o in run.obs:
            for lg in o.legs:
                if isinstance(lg.day, datetime.date) and lg.day == run.day and lg.day < self.K:
                    self.book_obs_now(o, lg, key[:4] + (key[4] + 1,))

    # -- the close (E §4.2 steps 4-5) ----------------------------------------------------------------------------
    def close_day(self, d):
        m = self.m
        key = (d.toordinal(), 5, 0, '', 0)
        for u in sorted(self.cons, key=str):
            if self.cons[u] != 0:
                raise refuse('ENGINE-INVARIANT', 'conservation: the legs booked on %s in %s sum to %s, not 0 (E §9 '
                                                 'invariant 1)' % (d, u, fmt(qty(self.cons[u], u))))
        self.cons = {}
        self.inv2_check(d)
        acc = m.kinds['accounts'].subtree(m) | m.kinds['cash'].subtree(m)
        for place in sorted(self.touched):
            th = m.things.get(place)
            if th is None:
                continue
            units = [u for (p, u) in self.logs if p == place and u != '*']
            if th.base in acc and not self.proj:
                for u in units:
                    st = self.pstate(place, u, key, raw=True)
                    if st.val is None or st.hole or st.perm or st.imp or st.bad:
                        continue
                    neg = st.val.q < 0
                    if neg and not self.neg.get((place, u)):
                        self.warnings.append(('WARN-NEGATIVE', '%s is below zero from %s' % (place, d)))
                    self.neg[(place, u)] = neg
        # a place whose lifespan ends today ends empty (RUN-LIFESPAN)
        for place, th in self.ending(d):
            # a place stated same that one side ends today and the other doesn't (L §6.1): the refusal depends on the
            # place being alive tomorrow, a HOLE-SAME hole
            nxt = self.alive_day(th, d + ONE_DAY) if getattr(th, 'same_others', None) else False
            for (p, u) in list(self.logs):
                if p != place or u == '*':
                    continue
                st = self.pstate(place, u, key, raw=True)
                if st.val is not None and st.val.q != 0 and not st.bad:
                    if isinstance(nxt, Hole):
                        self.add(place, u, Ent(key, 'hole', qs=frozenset(nxt.qs), flow=nxt))
                    else:
                        self.add(place, u, Ent(key, 'bad', err=refuse('RUN-LIFESPAN', '%s ends on %s holding %s' % (
                            place, d, fmt(st.val)))))

    def ending(self, d):
        m = self.m
        out = getattr(self, '_ending', None)
        if out is None:
            out = {}
            for name, th in m.things.items():
                if th.base is None or not m.is_place_thing(th):
                    continue
                try:
                    sps = [m.lifespan(th, self.K)]
                except SameDiffers:
                    # each side's end (L §6.1)
                    sps = self.side_spans(th)
                for end in sorted(set(sp.end for sp in sps if sp is not None and sp.end is not None)):
                    out.setdefault(end - ONE_DAY, []).append((name, th))
            self._ending = out
        return out.get(d, [])


def gtext(gap):
    if isinstance(gap, TallyError):
        return 'refused: %s %s' % (gap.code, gap.msg)
    if isinstance(gap, Hole):
        return fmt_hole(gap)
    return fmt(gap)


# -- engine side: timelines and walks ---------------------------------------------------------------------------------

def m_main_walk(self, ctx):
    """The main walk of a context's timeline, from S to E (E §3.3, §4.2)."""
    tctx = self.tctx(ctx)
    wk = ('walk',) + self.wkey(tctx)
    w = self.walks.get(wk)
    if w is not None:
        if getattr(w, 'failed', None) is not None:
            raise w.failed
        if not w.running and w.done < w.end:
            w.running = True
            try:
                w.run_to(w.end)
            except TallyError as e:
                w.failed = e
                raise
            finally:
                w.running = False
        return w
    m = self.m
    w = Walk(self, tctx, m.S, m.E)
    self.walks[wk] = w
    c = _coll()
    self.stack.append(c)
    w.running = True
    try:
        w.prepare()
        w.run_to(m.E)
    except TallyError as e:
        # a refusal the walk needs refuses every read of it (L §8.4), not a walk cut short
        w.failed = e
        raise
    finally:
        w.running = False
        self.stack.pop()
    for x in w.warnings:
        if x not in self.walk_warnings:
            self.walk_warnings.append(x)
    w.sig = w.signature()
    if tctx.world is not None:
        self.inv3(w, tctx)
    return w


def m_inv3(self, w, tctx):
    """E §9 invariant 3: a scenario changes nothing before its from that it doesn't list.  Every balance at the end of
    the day before D equals the base run's, unless a decision ran before D (the ones whose projection read a scenario
    row, which the engine lists; this interpreter doesn't tell them apart and checks nothing then)."""
    m = self.m
    sc = tctx.world
    D = m.scen_from.get(sc)
    if D is None or D <= m.S:
        return
    if any(getattr(r, 'point', None) is not None and kday(r.point) < D for r in w.decs.values()):
        return
    base = m.scenario_over.get(sc)
    from .model import set_world
    old = set_world(self.world_chain(base))
    try:
        bw = self.main_walk(tctx.but(world=base))
    except TallyError:
        return
    finally:
        set_world(old)
    last = min(D - ONE_DAY, m.E)
    key = K_end(last)
    for (place, unit) in sorted(set(w.logs) | set(bw.logs), key=repr):
        if place == '*' or unit == '*':
            continue
        sa = w.pstate(place, unit, key, raw=True)
        sb = bw.pstate(place, unit, key, raw=True)
        a, b = sa.val, sb.val
        if place not in m.things or not (isinstance(a, Num) and isinstance(b, Num)) or a.q == b.q:
            continue
        if any((x.hole or x.perm or x.imp or x.bad) for x in (sa, sb)):
            continue
        # an instance of a scenario's move whose day, a window's first day, or a leg's own day falls before from
        # is left out (L §6.4), so nothing of the scenario books before from
        raise refuse('ENGINE-INVARIANT', 'scenario %s: %s holds %s at the end of %s, the base %s (E §9 invariant 3)' % (
            sc, place, fmt(a), last, fmt(b)))


def m_proj_walk(self, ctx, t, leftout, clauses):
    """The walk of a projection from day t (E §4.3): from the main walk's state at the start of t, or at the reading
    decision's point on t; every movement as known on t, those that depend on the holder left out."""
    K0 = ctx.walk.K if ctx.walk is not None else ctx.cut
    c = min(t, K0)
    base = ctx.walk
    if base is not None and base.proj and ctx.wp is None:
        base = None
    point = None
    if base is not None and ctx.wp is not None and kday(ctx.wp) == t:
        point = ctx.wp
    elif base is not None and ctx.wp is not None:
        if t > kday(ctx.wp):
            raise refuse('RUN-FUTURE', 'a decision on %s reads a projection from %s, after its day' % (kday(ctx.wp), t))
        point = K_start(t)
    else:
        base = self.main_walk(ctx.but(cut=K0))
        point = K_start(t)
    if t < self.m.S:
        # L §5.14.1 (round 12): "a t before S is a hole, HOLE-PRERUN: nothing before S is walked"
        from .evaluate import HoleSignal
        raise HoleSignal(Hole([('before the run\'s start', 'from %s' % t)]))
    windows, rnd, unc = clauses
    if rnd and rnd[0] == 'proj':
        # a projection inside a projection's walk: the outer read's clause still holds (L §5.18 "every projection
        # the line reaches")
        rnd = (rnd[1], rnd[2]) if rnd[1] is not None else None
    # the projection's start rides with its clause: facts after t read at the clause (L §5.16); a decision's
    # projection carries the decision's day, since "An ask line's own projection is no decision" (L §5.16)
    dec = kday(ctx.wp) if (ctx.walk is not None and ctx.wp is not None) else None
    prnd = ('proj', rnd[0] if rnd else None, rnd[1] if rnd else None, t, dec)
    tctx = self.tctx(ctx).but(cut=c, windows=windows, rnd=prnd)
    wk = ('proj', id(base), point, c, leftout, windows, rnd, unc) + self.wkey(tctx)
    w = self.walks.get(wk)
    if w is not None and getattr(w, 'failed', None) is not None:
        raise w.failed
    if w is None:
        w = Walk(self, tctx, t, self.m.E, base=base, point=point, leftout=leftout, proj=True, unc=unc)
        self.walks[wk] = w
        cc = _coll()
        self.stack.append(cc)
        try:
            w.prepare()
            if point is not None and point[1] >= 1:
                # read at a point of the walk: what that day has booked so far is in the start; only what follows
                # it counts (L §5.14.1)
                acts = w.sched.get(t.toordinal(), [])
                w.sched[t.toordinal()] = [a for a in acts if a[0] > point and a[1] in ('decday',)]
        except TallyError as e:
            w.failed = e
            raise
        finally:
            self.stack.pop()
    return w


from . import partswalk as _partswalk  # noqa: E402
