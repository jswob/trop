"""Writer 11: parts in the walk (tally 2.1; L §3.6, §5.10, §5.13, §5.14, §5.14.1-2, §5.19, §8.1; E §4.2 Parts in the
walk).

The walk books balances as on 2.0. At a place whose kind keeps parts, every entry it adds is also queued
(`Walk.padd`), and the queue is taken at each point of the day's order (`parts_flush`): the start readings, the day's
first batch (after its sign rule), each decision's point (after its sign rule), each closing reading, the close. A
point's entries are one group (`pgroup`): its arrivals are labelled (the arrival rule at an owned place; batches
started or kept at a batched place), its outflows take the parts by the leaving rule as a fill over (part, outflow)
arcs, and its readings state or leave the parts. The state after the group is kept per (place, unit) in `plog`, and
each outflow's take per (movement, leg) in `ptake`, for `bal(x, d) of q`, `value(...) of q` and `m.leg.amount of q`.

A read of parts first reads the place's balance as 2.0 does: a refused or unknown balance is the read's answer, so
the parts only matter where the total is known (L §8.1 "The place's total stays known throughout").

Installed as Walk methods (`w_<name>` -> `<name>`) and Engine methods (`eng_<name>` -> `<name>`).
"""
import datetime
import functools
import math
from fractions import Fraction

from .errors import TallyError, Refusal, NotYet, Unspecified, load_now
from .values import Num, Money, Hole, Thing, Mv, Rec, NONE, CURRENCIES, fmt, Key
from .parts import thing_parts, labels_of

ANY = '*'
EMPTY = frozenset()


def install(W, E):
    for name, fn in list(globals().items()):
        if name.startswith('w_'):
            setattr(W, name[2:], fn)
        elif name.startswith('eng_'):
            setattr(E, name[4:], fn)


def refuse(code, msg):
    return Refusal(code, msg)


class HolePart:
    """A part whose label is a hole (L §8.1 A hole part): its amount, the questions of what made it, its possible
    labels, a set of (owner, batch) patterns where ANY stands for any party or batch."""
    __slots__ = ('amt', 'qs', 'poss', 'excl', 'sure')

    def __init__(self, amt, qs, poss, excl=(), sure=False):
        self.amt = amt
        self.qs = frozenset(unwrap(qs))
        self.poss = frozenset(poss)
        self.excl = frozenset(excl)     # labels it can't be: a reading's stated ones (L §5.19 the unstated rest)
        self.sure = sure                # its amount is surely its possible labels' (a stated part's own gap)


class _Unk:
    """The amount of the hole part of a hole balance (L §5.10 Parts, 2.1-F10): unknown, at least 0, and with no bound,
    so a take may always reach it and it never covers an outflow; what is added to it or taken from it stays it."""
    __slots__ = ()

    def __repr__(self):
        return 'UNK'

    def __gt__(self, o):
        return True

    def __ge__(self, o):
        return True

    def __lt__(self, o):
        return False

    def __le__(self, o):
        return False

    def __eq__(self, o):
        return o is self

    def __ne__(self, o):
        return o is not self

    def __hash__(self):
        return 0

    def __add__(self, o):
        return self
    __radd__ = __add__
    __sub__ = __add__
    __rsub__ = __add__


UNK = _Unk()


def is_unk(x):
    return x is UNK


def hb_of(st):
    """The hole part of a hole balance among a state's hole parts, or None."""
    for h in st.holes:
        if h.amt is UNK:
            return h
    return None


_ROOTS = {}


def unwrap(qs):
    """The questions of what made a hole part (L §8.1): a HOLE-PART question it rests on (a share or a term that read
    parts) stands for that part's own questions."""
    out = set()
    for q in qs:
        r = _ROOTS.get(q[0])
        if r is not None:
            out |= r
        else:
            out.add(q)
    return out


class PState:
    """The parts of one (place, unit) at one point: known labels -> amounts (Fraction, the place's unit), hole parts,
    and the collector of what the labels rest on (the arrival rules' and takes' reads)."""
    __slots__ = ('known', 'holes', 'coll', 'bad')

    def __init__(self, known=None, holes=(), coll=None, bad=None):
        self.known = dict(known or {})
        self.holes = tuple(holes)
        self.coll = coll
        self.bad = bad          # a refusal the parts carry from a reading on (a gap's take refused, an unplaced case)


class Take:
    """What one outflow leg took at its from place: label -> amount, and hole parts (an undecided take)."""
    __slots__ = ('place', 'known', 'holes', 'bad')

    def __init__(self, place):
        self.place = place
        self.known = {}
        self.holes = []
        self.bad = None


class Supply:
    __slots__ = ('lab', 'amt', 'src', 'hole')

    def __init__(self, lab, amt, src=None, hole=None):
        self.lab = lab          # (owner, batch), or None for a hole part
        self.amt = amt
        self.src = src          # the movement id whose arrival brought it in this group (its own take skips it)
        self.hole = hole        # the HolePart, for a hole supply


def point(key):
    """A key's point in the day's order: the day's first batch (observations and plans) is one point; a start reading,
    a decision (with the legs deferred after it), a closing reading and the close are each their own."""
    if 1 <= key[1] <= 2:
        return (key[0], 2)
    return tuple(key[:4])


def gpoint(e):
    """An entry's point: a pad of a start reading is in the day's first batch (E §4.2: "Steps 0-2 are one batch at each
    such place: its pads at the start, observations and plans take from its parts together"); an anchor or an opening
    at the start books nothing and is its own point before the batch."""
    k = e.key
    if k[1] == 0 and e.mv is not None and e.mv.__class__.__name__ == 'PadMv':
        return (k[0], 2)
    return point(k)


def ready(pt, grp, upto):
    """A point's group is taken once the walk is past it: the first batch once all of it is booked."""
    if len(pt) == 2:
        return (pt[0], 2, 999) <= upto
    return grp[0].key <= upto


def _coll():
    from .evaluate import Coll
    return Coll()


def qty(q, u):
    from .moves import qty as _q
    return _q(q, u)


def step_of(w, pk, place, unit):
    """The kind's step u, a Fraction in the place's unit (L §3.6: "step u is the smallest part")."""
    c = w.__dict__.setdefault('_pstep', {})
    k = (pk.id, unit)
    if k in c:
        return c[k]
    eng = w.eng
    v = eng.ev(pk.decl.step, {}, w.tctx.but(law=w.m.on))
    if isinstance(v, Hole) or not isinstance(v, Num) or v.q <= 0:
        raise refuse('RUN-DOMAIN', 'the step of %s is %s, not a positive amount' % (pk.id, fmt(v) if not isinstance(
            v, Hole) else 'a hole'))
    if v.money and v.u[0][0] != unit:
        raise refuse('RUN-DOMAIN', 'the step of %s is in %s, and %s holds %s' % (pk.id, v.u[0][0], place, unit))
    c[k] = v.q
    return v.q


# -- the queue and its flushes ------------------------------------------------------------------------------------

def w_pk(self, place):
    c = self.__dict__.setdefault('_pk', {})
    if place not in c:
        c[place] = thing_parts(self.m, self.m.things.get(place)) if place != '*' else None
    return c[place]


def w_padd(self, place, unit, e):
    """Called by Walk.add: queue an entry at a place that keeps parts for its point's group."""
    if place == '*' or unit in ('*', None) or not getattr(self.m, 'parts_kinds', None):
        return
    if self.pk(place) is None:
        return
    self.__dict__.setdefault('ppend', {}).setdefault((place, unit), []).append(e)


def w_parts_flush(self, upto):
    """Take every queued group whose point ends by upto, place by place (a place another's group needs is taken on
    demand, pneed)."""
    pend = self.__dict__.get('ppend')
    if not pend:
        return
    for pu in sorted(pend, key=str):
        self.pflush(pu[0], pu[1], upto)


def w_pflush(self, place, unit, upto):
    pend = self.__dict__.get('ppend', {}).get((place, unit))
    while pend:
        pend.sort(key=lambda e: e.key)
        pt = min(gpoint(e) for e in pend)
        grp = [e for e in pend if gpoint(e) == pt]
        if not ready(pt, grp, upto):
            return
        rest = [e for e in pend if gpoint(e) != pt]
        self.ppend[(place, unit)] = rest
        busy = self.__dict__.setdefault('pbusy', {})
        stack = self.__dict__.setdefault('pstack', [])
        busy[(place, unit)] = pt
        stack.append((place, unit))
        try:
            self.pgroup(place, unit, grp, pt)
        except PartsCycle:
            # a cycle unwinds through this group: it is taken again once the cycle is settled
            self.ppend[(place, unit)] = grp + self.ppend.get((place, unit), [])
            raise
        finally:
            busy.pop((place, unit), None)
            stack.pop()
        pend = self.ppend.get((place, unit))


def w_pneed(self, place, unit, key):
    """Another group needs this place's parts up to key (a kept batch, a leg read): take its groups through key's
    point. A place whose group is being taken is a cycle among the batch's takes (L §5.14.2)."""
    busy = self.__dict__.get('pbusy', {})
    pt = point(key)
    if (place, unit) in busy and busy[(place, unit)] == pt:
        raise PartsCycle((place, unit), pt)
    self.pflush(place, unit, max(key[:5], (pt[0], 2, 999)) if len(pt) == 2 else key)


class PartsCycle(Exception):
    """A group needs a take of a group being taken at the same point: pu is that group's (place, unit), members the
    groups the need ran through, added as it unwinds; the group of pu settles it (pcycle)."""
    def __init__(self, pu, pt):
        self.pu = pu
        self.pt = pt
        self.members = []


class Cycle:
    """A cycle being settled at point pt (L §5.14.2): its groups, its question, and the mode: 'try' (the cycle's
    arrivals as parts of unknown labels, nothing kept) or 'hole' (a cycle of observed rows, kept)."""
    __slots__ = ('members', 'pt', 'q', 'mode')

    def __init__(self, members, pt, q, mode):
        self.members = members
        self.pt = pt
        self.q = q
        self.mode = mode


def w_plog_state(self, place, unit, key):
    """The parts of (place, unit) at an instant: the last group taken at or before key."""
    import bisect
    pl = self.__dict__.get('plog', {}).get((place, unit))
    if pl is not None:
        i = bisect.bisect_right(pl[0], key) - 1
        if i >= 0:
            return pl[1][i]
    return self.pinit(place, unit)


def w_pinit(self, place, unit):
    """The parts before a place's first group: a projection's are its base's at its start point (E §4.3), a place
    that starts empty in the run has none, and one alive before S "has no known parts before its first anchor reading,
    as it has no balance there ...: its parts are one hole part, of the hole balance, possibly anyone's" (L §5.10
    Parts, 2.1-F10); its question is the balance's HOLE-ANCHOR one (an opening at S drops it, preset)."""
    if self.base is not None and self.point is not None:
        return self.base.plog_state(place, unit, self.point)
    st0 = self.default(place, unit)
    if st0.val is None and not st0.hole and not st0.bad:
        return PState(holes=(HolePart(UNK, [self.anchor_question(place)], [(ANY, ANY)]),))
    return PState()


def w_plog_put(self, place, unit, key, st):
    pl = self.__dict__.setdefault('plog', {}).setdefault((place, unit), ([], []))
    if pl[0] and pl[0][-1] >= key:
        # a later group at the same key (a pad and its reading) replaces it
        while pl[0] and pl[0][-1] >= key:
            pl[0].pop()
            pl[1].pop()
    pl[0].append(key)
    pl[1].append(st)


# -- one group -------------------------------------------------------------------------------------------------

def w_pgroup(self, place, unit, grp, pt):
    log = self.logs.get((place, unit))
    live = set(id(e) for e in log.ents) if log is not None else set()
    grp = [e for e in grp if id(e) in live]
    if not grp:
        return
    grp.sort(key=lambda e: e.key)
    gkey = grp[-1].key
    prev = self.plast(place, unit)
    for _ in range(64):
        try:
            st, refused = self.prun(place, unit, grp, prev, {})
            break
        except PartsCycle as c:
            if c.pu != (place, unit):
                if (place, unit) not in c.members:
                    c.members.append((place, unit))
                raise
            res = self.pcycle(place, unit, grp, prev, c)
            if res is not None:
                st, refused = res
                break
    else:
        raise NotYet('parts: a cycle among takes that its decided takes don\'t settle')
    for (mid, lname), err in refused.items():
        self.prefuse(place, unit, grp, mid, lname, err, gkey)
    self.plog_put(place, unit, gkey, st)


def w_plast(self, place, unit):
    pl = self.__dict__.get('plog', {}).get((place, unit))
    if pl is not None and pl[0]:
        return pl[1][-1]
    return self.pinit(place, unit)


def w_prun(self, place, unit, grp, prev, refused):
    """The group's state and its refused legs: a leg a rule refuses is left out and the group taken again."""
    refused = dict(refused)
    for _ in range(8):
        st, bad = self.pgroup_run(place, unit, grp, prev, refused)
        if not bad:
            return st, refused
        for k, err in bad.items():
            refused[k] = err
    raise NotYet('parts: a group whose refusals don\'t settle')


def w_prefuse(self, place, unit, grp, mid, lname, err, gkey):
    """A leg a parts rule refuses (RUN-LABEL, RUN-NO-RULE, RUN-PRORATA, RUN-TIE, RUN-LIFESPAN, a refusal in a term):
    "a refused leg isn't booked, and the place's balance is refused from that point" (E §4.2 step 5), at both its
    ends, as the sign rule refuses one (Walk.check_group)."""
    from .walk import Ent
    mv = lg = None
    for e in grp:
        if e.mv is not None and e.mv.id == mid and e.note == lname:
            mv = e.mv
            break
    if mv is None:
        for e in grp:
            if e.kind == 'reset' and _is_pad(e) and e.mv.id == mid:
                return self.prefuse_pad(e, lname, err, unit)
        return
    for x in mv.legs:
        if x.name == lname:
            lg = x
    if lg is None or (mv.id, lname) in self.bad_legs:
        return
    self.bad_legs[(mv.id, lname)] = err
    pt = point(gkey)
    for p in (lg.frm, lg.to):
        if isinstance(p, Thing):
            self.unbook(p.name, unit, mv, lg, lambda k: point(k) == pt)
    for p in (lg.frm, lg.to):
        if isinstance(p, Thing):
            self.add(p.name, unit, Ent(gkey, 'bad', err=err, mv=mv))


def w_prefuse_pad(self, e, lname, err, unit):
    """A pad a parts rule refuses (L §5.19: "A pad no rule reaches is refused (RUN-NO-RULE)"): its other end isn't
    booked, and both places' balances are refused from the reading on (E §4.2 step 5), after its reset."""
    from .walk import Ent
    mv = e.mv
    lg = mv.legs[0]
    if (mv.id, lg.name) in self.bad_legs:
        return
    self.bad_legs[(mv.id, lg.name)] = err
    k = e.key[:4] + (e.key[4] + 1,)
    for p in (lg.frm, lg.to):
        if isinstance(p, Thing):
            self.unbook(p.name, unit, mv, lg, lambda kk: kk[:4] == e.key[:4])
    for p in (lg.frm, lg.to):
        if isinstance(p, Thing):
            self.add(p.name, unit, Ent(k, 'bad', err=err, mv=mv))


def _leg_of(mv, name):
    for x in mv.legs:
        if x.name == name:
            return x
    return None


class _PadLeg:
    """A pad's leg at its reading's place, a delta of its point's take (L §5.19 A pad: "at x it is an arrival, labelled
    by the arrival rule ..., or an outflow, taken by the leaving rule, like any other")."""
    __slots__ = ('kind', 'key', 'mv', 'note', 'flow', 'reading')

    def __init__(self, e, lg):
        self.kind = 'delta'
        self.key = e.key
        self.mv = e.mv
        self.note = lg.name
        self.flow = e.flow
        self.reading = e


def _is_pad(e):
    return e.mv is not None and e.mv.__class__.__name__ == 'PadMv'


def w_pgroup_run(self, place, unit, grp, prev, refused):
    """The group's state from prev: resets in order, the deltas between them one take each (L §5.14: "at one point of
    the day's order ... the supplies are x's parts there and the demands are the outflows taking at x there"). A pad's
    reading is a delta of the take at its point (a start pad's, of the first batch's, E §4.2), its stated parts set
    after the take."""
    st = PState(prev.known, prev.holes, prev.coll)
    bad = {}
    batch = []
    pads = []
    for e in grp:
        if e.kind == 'delta' and e.mv is not None and isinstance(e.flow, Num):
            if (e.mv.id, e.note) in refused or (e.mv.id, e.note) in self.bad_legs:
                continue
            batch.append(e)
        elif e.kind == 'reset' and _is_pad(e):
            lg = e.mv.legs[0]
            if (e.mv.id, lg.name) not in refused and (e.mv.id, lg.name) not in self.bad_legs and \
                    isinstance(e.flow, Num) and e.flow.q != 0:
                batch.append(_PadLeg(e, lg))
            pads.append(e)
        elif e.kind == 'reset':
            if batch:
                st = self.ptake_batch(place, unit, st, batch, e.key, bad)
                batch = []
            st = self.preset(place, unit, st, e)
    others = [e for e in batch if not isinstance(e, _PadLeg)]
    if batch:
        st = self.ptake_batch(place, unit, st, batch, grp[-1].key, bad)
    for e in pads:
        if hb_of(st) is not None and isinstance(e.val, Num) and not isinstance(e.flow, Num):
            # the first pad of a place alive before S (2.1-F10): its amount is a hole, as the booked balance is, and
            # the reading states the total; L §5.19 with the hole balance's part among the booked parts (pstated_hb)
            if others and self.pstated_rows(e):
                st = self.ppad_parts(place, unit, st, e, others)
            else:
                st = self.pstated_safe(place, unit, st, e.val.q, e)
            continue
        st = self.ppad_parts(place, unit, st, e, others)
    return st, bad


# -- readings (L §5.19 Readings at a place that keeps parts) ------------------------------------------------------

def w_preset(self, place, unit, st, e):
    """An opening's or an anchor's reset (a pad's reading is a delta of its point's take, pgroup_run): the reading
    states the total, and its stated parts, if any (L §5.19 Readings at a place that keeps parts)."""
    if not isinstance(e.val, Num):
        return PState()
    base = PState(coll=st.coll) if e.note == 'opening' else st
    return self.pstated_safe(place, unit, base, e.val.q, e)


def w_pstated_safe(self, place, unit, base, total, e):
    """A reading's parts; a refusal there refuses the parts from the reading on (its total stands)."""
    try:
        st = self.pstated(place, unit, base, total, self.pstated_rows(e), e)
    except TallyError as err:
        if load_now(err):
            raise
        st = PState(coll=base.coll, bad=err)
    row = getattr(e, 'row', None)
    if st.bad is not None and row is not None and row.__dict__.get('parts'):
        self.__dict__.setdefault('pgaps', {})[row.sid] = st.bad
    return st


def w_pgap_lines(self, row, key):
    """A reading's stated parts' gaps for the ledger (E §6.3): [(label, booked, stated)], or its refusal."""
    self.parts_flush((key[0], 99))
    g = self.__dict__.get('pgaps', {}).get(row.sid)
    if isinstance(g, TallyError):
        raise g
    return g


def w_ppad_parts(self, place, unit, st, e, others):
    """After a pad's take: the reading's stated parts (writer 11, step 7)."""
    parts = self.pstated_rows(e)
    if not parts:
        return st
    if others:
        # L §5.19 (2.1-F2): "A reading at a day's start that pads and states parts, at a place the day's first batch
        # also moves, is refused (RUN-RECORD)"
        from .walk import kday
        bad = refuse('RUN-RECORD', 'reading %s of %s at %s start pads and states parts while the day\'s first batch '
                                   'moves %s' % (e.row.sid, place, kday(e.key).isoformat(), place))
        self.__dict__.setdefault('pgaps', {})[e.row.sid] = bad
        return PState(st.known, st.holes, st.coll, bad=bad)
    return self.pstated_safe(place, unit, st, e.val.q if isinstance(e.val, Num) else None, e)


def _st_total(st):
    return sum(st.known.values(), Fraction(0)) + sum((h.amt for h in st.holes), Fraction(0))


def w_pstated_rows(self, e):
    """A reading's stated parts the walk's cut sees: [(q node, value, row)] (L §5.19 Stated parts)."""
    from .model import visible, row_known_sid
    row = getattr(e, 'row', None)
    parts = row.__dict__.get('parts') if row is not None else None
    if not parts:
        return []
    out = []
    for qn, v, pr in parts:
        vis = visible(pr.known, self.K, row_known_sid(pr))
        if vis is None:
            raise NotYet('a stated part known in a window that holds the cut')
        if vis:
            out.append((qn, v, pr))
    return out


def _ntext(n):
    from .check import _txt
    try:
        return _txt(n)
    except Exception:  # a node the checker's printer doesn't know
        return n.k


def _pattern(q):
    if q[0] == 'owner':
        return (q[1], ANY)
    if q[0] == 'batch':
        return (ANY, q[1])
    return (q[1], q[2])


def _full_label(q, labs):
    """The one label q names at a place keeping labs, or None (a party at a place that is both, a batch there)."""
    if labs == {'owner'} and q[0] == 'owner':
        return (q[1], None)
    if labs == {'batch'} and q[0] == 'batch':
        return (None, q[1])
    if q[0] == 'pair':
        return (q[1], q[2])
    return None


def _state_of(known, holes, coll, bad=None):
    k = {}
    for lab, a in known.items():
        if a != 0:
            k[lab] = k.get(lab, Fraction(0)) + a
    return PState(k, tuple(h for h in holes if h.amt != 0), coll, bad)


def w_pstated(self, place, unit, base, total, parts, e):
    """L §5.19: "After the reading's pad or anchor has set the total, a stated part is q's balance, as a stated total
    is the place's. Where it differs from q's booked part, the difference is q's own gap: where positive, a part of q's
    of that amount, from the reading's document; where negative, a take of q's parts by the leaving rule, its terms on
    w holes as above. The unstated rest has a gap of its own, over the labels the reading doesn't state, by the anchor's
    rules above." With no stated parts: "An anchor whose gap is 0 leaves the parts as they are"; "a positive gap is a
    hole part of the gap's amount, possibly anyone's; a negative gap is a take of its size by an outflow nobody knows".
    q's booked part is its known parts; a hole part is in the unstated rest, which then can't be a stated label."""
    from .walk import kday
    if total is None:
        return PState()
    if base.bad is not None:
        return base
    coll = base.coll
    if hb_of(base) is not None:
        return self.pstated_hb(place, unit, base, total, parts, e)
    if not parts:
        gap = total - _st_total(base)
        if gap == 0:
            return base
        if gap > 0:
            return PState(base.known, base.holes + (HolePart(gap, [self.pgap_question(place, e, gap)],
                                                              [(ANY, ANY)]),), coll)
        return self.pgap_take(place, unit, base.known, base.holes, -gap, e, coll)
    m = self.m
    pk = self.pk(place)
    labs = labels_of(pk)
    ctx = self.tctx.but(law=kday(e.key))
    stated = []
    for qn, v, pr in parts:
        q = getattr(pr, 'ofq', None)
        if q is None:
            # load names a stated part's label: a party, a movement, or the pair (L §2.3 ofrow)
            raise refuse('RUN-DOMAIN', 'of %s: a party, a batch or the pair' % _ntext(qn))
        check_sort(m, pk, place, q)
        if not isinstance(v, Num):
            raise NotYet('a stated part that is a hole or none')
        # a stated part below 0 and two that overlap are refused at load (LOAD-PARTS, 2.1-F6: loader.check_part_rows)
        stated.append((q, v.q))
    tot_s = sum((v for q, v in stated), Fraction(0))
    if tot_s > total:
        return PState(coll=coll, bad=refuse('RUN-RECORD', 'the stated parts of %s at %s sum to %s, over its total %s' % (
            place, '%s %s' % (kday(e.key).isoformat(), 'start' if e.key[1] == 0 else 'end'), fmt(qty(tot_s, unit)),
            fmt(qty(total, unit)))))
    # E §6.3 (2.1-F3): the ledger's balance line shows each stated part's gap and the rest's, stated - booked
    gl = []
    known = {}
    holes = []
    pool = dict((lab, a) for lab, a in base.known.items() if not any(lab_hits(lab, q) for q, v in stated))
    for q, v in stated:
        kq = dict((lab, a) for lab, a in base.known.items() if lab_hits(lab, q))
        g = v - sum(kq.values(), Fraction(0))
        gl.append(('of %s' % _qtext(q), sum(kq.values(), Fraction(0)), v))
        if g >= 0:
            for lab, a in kq.items():
                known[lab] = known.get(lab, Fraction(0)) + a
            if g > 0:
                full = _full_label(q, labs)
                if full is not None:
                    known[full] = known.get(full, Fraction(0)) + g
                else:
                    holes.append(HolePart(g, [self.pgap_question(place, e, g, q)], [_pattern(q)], sure=True))
        else:
            r = self.pgap_take(place, unit, kq, (), -g, e, coll)
            if r.bad is not None:
                return r
            for lab, a in r.known.items():
                known[lab] = known.get(lab, Fraction(0)) + a
            holes.extend(r.holes)
    excl = [_pattern(q) for q, v in stated]
    rest_holes = [HolePart(h.amt, h.qs, h.poss, set(h.excl) | set(excl), h.sure) for h in base.holes]
    booked_rest = sum(pool.values(), Fraction(0)) + sum((h.amt for h in rest_holes), Fraction(0))
    rg = (total - tot_s) - booked_rest
    gl.append(('rest', booked_rest, total - tot_s))
    if getattr(e, 'row', None) is not None:
        self.__dict__.setdefault('pgaps', {})[e.row.sid] = gl
    if rg >= 0:
        for lab, a in pool.items():
            known[lab] = known.get(lab, Fraction(0)) + a
        holes.extend(rest_holes)
        if rg > 0:
            holes.append(HolePart(rg, [self.pgap_question(place, e, rg)], [(ANY, ANY)], excl))
    else:
        r = self.pgap_take(place, unit, pool, rest_holes, -rg, e, coll)
        if r.bad is not None:
            return r
        for lab, a in r.known.items():
            known[lab] = known.get(lab, Fraction(0)) + a
        holes.extend(r.holes)
    return _state_of(known, holes, coll)


def w_pstated_hb(self, place, unit, base, total, parts, e):
    """The first anchor or pad of a place alive before S (L §5.10 Parts, 2.1-F10, with L §5.19): the hole part of the
    hole balance is among the booked parts, so the booked total, and the gap of the unstated rest, is a hole. A stated
    part's booked part is its known parts (the hole part is in the rest), so its own gap is known and goes as §5.19
    says. The rest's gap could be negative by any amount, a take by an outflow nobody knows that "could reach under
    some reading of the holes" every booked part of the rest (L §5.14), so they merge: the rest is one hole part of
    its stated amount, the reading's unstated rest (`whose are <a> of <x> at <instant>?`, L §8.1), possibly anyone's
    but the stated labels; none where the stated parts cover the total."""
    from .walk import kday
    coll = base.coll
    hb = hb_of(base)
    stated = []
    if parts:
        m = self.m
        pk = self.pk(place)
        for qn, v, pr in parts:
            q = getattr(pr, 'ofq', None)
            if q is None:
                raise refuse('RUN-DOMAIN', 'of %s: a party, a batch or the pair' % _ntext(qn))
            check_sort(m, pk, place, q)
            if not isinstance(v, Num):
                raise NotYet('a stated part that is a hole or none')
            stated.append((q, v.q))
    tot_s = sum((v for q, v in stated), Fraction(0))
    if tot_s > total:
        return PState(coll=coll, bad=refuse('RUN-RECORD', 'the stated parts of %s at %s sum to %s, over its total %s' % (
            place, '%s %s' % (kday(e.key).isoformat(), 'start' if e.key[1] == 0 else 'end'), fmt(qty(tot_s, unit)),
            fmt(qty(total, unit)))))
    labs = labels_of(self.pk(place))
    known = {}
    holes = []
    gl = []
    for q, v in stated:
        kq = dict((lab, a) for lab, a in base.known.items() if lab_hits(lab, q))
        g = v - sum(kq.values(), Fraction(0))
        gl.append(('of %s' % _qtext(q), sum(kq.values(), Fraction(0)), v))
        if g >= 0:
            for lab, a in kq.items():
                known[lab] = known.get(lab, Fraction(0)) + a
            if g > 0:
                full = _full_label(q, labs)
                if full is not None:
                    known[full] = known.get(full, Fraction(0)) + g
                else:
                    holes.append(HolePart(g, [self.pgap_question(place, e, g, q)], [_pattern(q)], sure=True))
        else:
            r = self.pgap_take(place, unit, kq, (), -g, e, coll)
            if r.bad is not None:
                return r
            for lab, a in r.known.items():
                known[lab] = known.get(lab, Fraction(0)) + a
            holes.extend(r.holes)
    rest = total - tot_s
    gl.append(('rest', Hole(hb.qs), rest))
    if getattr(e, 'row', None) is not None:
        self.__dict__.setdefault('pgaps', {})[e.row.sid] = gl
    if rest > 0:
        holes.append(HolePart(rest, [self.pgap_question(place, e, rest)], [(ANY, ANY)],
                              [_pattern(q) for q, v in stated]))
    return _state_of(known, holes, coll)


class _GapMv:
    """The outflow nobody knows of a reading's negative gap (L §5.19): its order terms on w are holes."""
    def __init__(self, ident, q):
        self.id = ident
        self._q = q

    def mv(self):
        return Hole([self._q])


class _GapOut:
    __slots__ = ('mv', 'flow', 'key', 'note')

    def __init__(self, mv, flow, key):
        self.mv = mv
        self.flow = flow
        self.key = key
        self.note = 'gap'


class _GapLeg:
    name = 'gap'


def w_pgap_take(self, place, unit, known, holes, amount, e, coll):
    """A take of amount from the given parts by the place's leaving rule, its outflow nobody knows: "the order's terms
    on w are holes, its terms on k still rank, and the take is undecided as far as the terms on w reach" (L §5.19). A
    refusal of the take (RUN-TIE) refuses the parts from the reading on."""
    from .walk import kday
    pk = self.pk(place)
    step = step_of(self, pk, place, unit)
    it = '%s %s' % (kday(e.key).isoformat(), 'start' if e.key[1] == 0 else 'end')
    q = ('which outflow took %s from %s by %s?' % (fmt(qty(amount, unit)), place, it), place)
    gm = _GapMv('the gap of %s at %s' % (place, it), q)
    supplies = [Supply(lab, a) for lab, a in sorted(known.items(), key=lambda kv: _labkey(kv[0])) if a > 0]
    supplies += [Supply(None, h.amt, hole=h) for h in holes if h.amt > 0]
    bad = {}
    takes = self.__dict__.setdefault('ptake', {})
    c = _coll()
    if coll is not None:
        c.merge(coll)
    try:
        self.pfill(place, unit, pk, step, supplies, [(_GapOut(gm, qty(-amount, unit), e.key), _GapLeg())], e.key, c,
                   bad)
    except TallyError as err:
        if load_now(err):
            raise
        bad[(gm.id, 'gap')] = err
    finally:
        takes.pop((gm.id, 'gap'), None)
    if bad:
        return PState(coll=c, bad=list(bad.values())[0])
    kn = {}
    hs = []
    for s_ in supplies:
        if s_.amt <= 0:
            continue
        if s_.hole is not None:
            hs.append(HolePart(s_.amt, s_.hole.qs, s_.hole.poss, s_.hole.excl, s_.hole.sure))
        else:
            kn[s_.lab] = kn.get(s_.lab, Fraction(0)) + s_.amt
    return _state_of(kn, hs, c)


def w_pgap_question(self, place, e, amt, q=None):
    """The question of a reading's unknown money (L §8.1 A hole part, 2.1-F7: "for a gap or a reading's unstated rest
    whose are <a> of <x> at <instant>?; ... for a stated part whose batches are open which batches are <q>'s <a> of <x>
    at <instant>?"); at a place that is both, a stated batch's open owners are asked alike."""
    from .walk import kday
    a = fmt(qty(amt, e.val.u[0][0]) if e.val.money else Num(amt, e.val.u))
    it = '%s %s' % (kday(e.key), 'start' if e.key[1] == 0 else 'end')
    if q is not None:
        return ('which %s are %s\'s %s of %s at %s?' % ('owners' if q[0] == 'batch' else 'batches', _qtext(q), a,
                                                        place, it), place)
    return ('whose are %s of %s at %s?' % (a, place, it), place)


# -- a take: the arrivals of one point and its outflows (L §3.6, §5.12, §5.13, §5.14) --------------------------------

def w_ptake_batch(self, place, unit, st, batch, key, bad):
    if st.bad is not None:
        # parts refused from a reading on: what an outflow takes is refused with them
        for e in batch:
            if e.flow.q < 0:
                tk = Take(place)
                tk.bad = st.bad
                self.__dict__.setdefault('ptake', {})[(e.mv.id, e.note)] = tk
        return st
    cyc = self.__dict__.get('_pcyc')
    if cyc is not None and cyc.mode == 'try' and len(key) == 5:
        key = key + ('try',)        # its reads are remembered apart from the kept ones
    self._pgkey = key
    m = self.m
    pk = self.pk(place)
    step = step_of(self, pk, place, unit)
    labs = labels_of(pk)
    arrivals = []
    relabels = []
    outflows = []
    for e in batch:
        lg = _leg_of(e.mv, e.note)
        if lg is None:
            continue
        if e.flow.q > 0:
            if isinstance(lg.frm, Thing) and lg.frm.name == place:
                relabels.append((e, lg))
            else:
                arrivals.append((e, lg))
        elif e.flow.q < 0:
            outflows.append((e, lg))
    coll = _coll()
    if st.coll is not None:
        coll.merge(st.coll)
    supplies = [Supply(lab, a) for lab, a in sorted(st.known.items(), key=lambda kv: _labkey(kv[0])) if a > 0]
    supplies += [Supply(None, h.amt, hole=h) for h in st.holes if h.amt > 0]
    # a group taken again recomputes its takes; a take a cycle decided stands (pcycle)
    takes = self.__dict__.setdefault('ptake', {})
    dec = self.__dict__.get('_pdecided', ())
    for e, lg in outflows:
        if (e.mv.id, lg.name) not in dec:
            takes.pop((e.mv.id, lg.name), None)
    # the point's arrivals are supplies of its takes (L §5.14 "The batch's arrivals are supplies")
    for e, lg in arrivals:
        try:
            parts = self.parrive(place, unit, pk, step, labs, e.mv, lg, e.flow.q, key, coll)
        except TallyError as err:
            if load_now(err):
                raise
            bad[(e.mv.id, lg.name)] = err
            continue
        for lab, a, hp in parts:
            supplies.append(Supply(lab, a, src=e.mv.id, hole=hp))
    # a relabel's arrival is an arrival of its point too, a supply of the point's other outflows; where it reads its
    # own take (its batches kept, a leg read in the rule), the place's take reads itself: a cycle among the point's
    # takes, settled as L §5.14.2 says (prelabel_cycle)
    late = []
    unknown = []
    for e, lg in relabels:
        if (e.mv.id, lg.name) in bad:
            continue
        parts = self.prelabel(place, unit, pk, step, labs, e, lg, key, coll, bad)
        if parts is None:
            unknown.append((e, lg))
        elif parts is not False:
            for lab, a, hp in parts:
                supplies.append(Supply(lab, a, src=e.mv.id, hole=hp))
    if unknown and not any(o.mv.id != e.mv.id for e, lg in unknown for o, olg in outflows):
        # its own take never includes its own arrival, and no other outflow takes at this point
        late, unknown = unknown, []
    if unknown:
        unknown = self.prelabel_cycle(place, unit, pk, step, labs, supplies, outflows, unknown, key, coll, bad)
    if unknown:
        cyc_e = [e for e, lg in unknown] + [o for o, olg in outflows if (o.mv.id, olg.name) not in bad]
        mvs = sorted(set(e.mv.id for e in cyc_e))
        if any(e.key[1] == 2 for e in cyc_e):
            err = refuse('RUN-CYCLE', 'the takes of %s at %s read each other in one batch' % (', '.join(mvs), place))
            for e in cyc_e:
                bad[(e.mv.id, e.note)] = err
            return st
        if any(e.key[1] != 1 for e in cyc_e):
            raise Unspecified('§5.14.2', 'a cycle among the takes of %s at %s, at a decision\'s point, that no take '
                                         'breaks' % (', '.join(mvs), place))
        q = (self.prelabel_q(mvs, key), place)
        for e, lg in unknown:
            supplies.append(Supply(None, e.flow.q, src=e.mv.id, hole=HolePart(e.flow.q, [q], [(ANY, ANY)])))
    if outflows:
        try:
            self.pfill(place, unit, pk, step, supplies, outflows, key, coll, bad)
        except TallyError as err:
            if load_now(err):
                raise
            for e, lg in outflows:
                bad.setdefault((e.mv.id, lg.name), err)
    for e, lg in late:
        if (e.mv.id, lg.name) in bad:
            continue
        try:
            parts = self.parrive(place, unit, pk, step, labs, e.mv, lg, e.flow.q, key, coll)
        except TallyError as err:
            if load_now(err):
                raise
            bad[(e.mv.id, lg.name)] = err
            continue
        for lab, a, hp in parts:
            supplies.append(Supply(lab, a, src=e.mv.id, hole=hp))
    known = {}
    holes = []
    for s in supplies:
        if s.amt == 0:
            continue
        if s.hole is not None:
            holes.append(HolePart(s.amt, s.hole.qs, s.hole.poss, s.hole.excl, s.hole.sure))
        else:
            known[s.lab] = known.get(s.lab, Fraction(0)) + s.amt
    return PState(dict((k, v) for k, v in known.items() if v != 0), holes, coll)


class _NeedsTake(Exception):
    """A relabel's arrival read its own take before the point's take was computed."""


def w_prelabel(self, place, unit, pk, step, labs, e, lg, key, coll, bad):
    """A relabel's arrival, if it doesn't read its own take still to be computed: its parts, None where it does, False
    where it is refused."""
    prev = self.__dict__.get('_preltry')
    self._preltry = (e.mv.id, lg.name)
    try:
        return self.parrive(place, unit, pk, step, labs, e.mv, lg, e.flow.q, key, coll)
    except _NeedsTake:
        return None
    except TallyError as err:
        if load_now(err):
            raise
        bad[(e.mv.id, lg.name)] = err
        return False
    finally:
        self._preltry = prev


def w_prelabel_q(self, mvs, key):
    from .walk import kday
    return 'which of %s went first on %s?' % (', '.join(mvs), kday(key).isoformat())


def w_prelabel_cycle(self, place, unit, pk, step, labs, supplies, outflows, unknown, key, coll, bad):
    """L §5.14.2: "each take is tried with the cycle's arrivals as parts of unknown labels, and a take whose outcome is
    the same under every reading of them ... is decided, which breaks the cycle". A relabel whose take is decided gets
    its arrival, a supply of the point's take; the relabels left are returned."""
    unknown = list(unknown)
    mvs = sorted(set([e.mv.id for e, lg in unknown] + [o.mv.id for o, olg in outflows]))
    q = (self.prelabel_q(mvs, key), place)
    outs = [(o, olg) for o, olg in outflows if (o.mv.id, olg.name) not in bad]
    while unknown:
        trial = [Supply(x.lab, x.amt, x.src, x.hole) for x in supplies]
        for e, lg in unknown:
            trial.append(Supply(None, e.flow.q, src=e.mv.id, hole=HolePart(e.flow.q, [q], [(ANY, ANY)])))
        saved = self.ptake
        self.ptake = dict(saved)
        tb = {}
        try:
            self.pfill(place, unit, pk, step, trial, outs, key, coll, tb)
            tried = self.ptake
        except TallyError as err:
            if load_now(err):
                raise
            tried = {}
        finally:
            self.ptake = saved
        progress = False
        for e, lg in list(unknown):
            tk = tried.get((e.mv.id, lg.name))
            if tk is None or tk.holes or (e.mv.id, lg.name) in tb:
                continue
            self.ptake[(e.mv.id, lg.name)] = tk
            parts = self.prelabel(place, unit, pk, step, labs, e, lg, key, coll, bad)
            unknown.remove((e, lg))
            progress = True
            if parts:
                for lab, a, hp in parts:
                    supplies.append(Supply(lab, a, src=e.mv.id, hole=hp))
        if not progress:
            break
    return unknown


def _labkey(lab):
    return tuple('' if x is None else str(x) for x in lab)


def w_pparties(self, kind):
    """The parties an arrival rule's party key ranges over: the things of its kind known at the walk's cut, in the
    text order of their stable ids (L §2.5: an order no rule reads)."""
    c = self.__dict__.setdefault('_pparties', {})
    if kind in c:
        return c[kind]
    m = self.m
    sub = m.kinds[kind].subtree(m)
    out = []
    for name in sorted(m.things):
        th = m.things[name]
        if th.base in sub or any(e.kind in sub for e in getattr(th, 'entries', ())):
            try:
                if m.thing_seen(th, self.K):
                    out.append(name)
            except TallyError:
                out.append(name)
    c[kind] = out
    return out


def w_parrive(self, place, unit, pk, step, labs, mv, lg, amt, key, coll):
    """An arrival's parts (L §3.6 The arrival rule; Batches start here): [(label, amount, hole part or None)]."""
    from .walk import kday
    m = self.m
    if (amt / step).denominator != 1:
        raise refuse('RUN-PRORATA', '%s arriving at %s isn\'t a whole number of its step %s' % (
            fmt(qty(amt, unit)), place, fmt(qty(step, unit))))
    # the batch portions: kept from the take at a batched from place, else this movement starts a batch
    portions = []
    if 'batch' in labs:
        frm = lg.frm
        fpk = self.pk(frm.name) if isinstance(frm, Thing) else None
        if fpk is not None and 'batch' in labels_of(fpk):
            tk = self.ptake_of(mv, lg, key)
            if tk.bad is not None:
                raise tk.bad
            for lab, a in sorted(tk.known.items(), key=lambda kv: _labkey(kv[0])):
                portions.append((lab[1], a, None))
            for h in tk.holes:
                portions.append((None, h.amt, h))
        else:
            self.batch_mv()[mv.id] = mv.mv()
            portions.append((mv.id, amt, None))
    else:
        portions.append((None, amt, None))
    if 'owner' not in labs:
        return [((None, b), a, None) if hp is None else (None, a, hp) for b, a, hp in portions]
    out = []
    shares, hparts, hqs = self.pshares(place, pk, mv, kday(key), coll)
    for b, a, hp in portions:
        if hp is not None:
            owners = set(p for p, s in shares if s != 0) | set(hparts)
            poss = [(o, pb) for o in sorted(owners) for (po, pb) in hp.poss] if owners else hp.poss
            out.append((None, a, HolePart(a, hp.qs | hqs, poss, hp.excl)))
            continue
        out.extend(self.psplit_owners(place, unit, pk, step, mv, b, a, shares, hparts, hqs))
    day = kday(key)
    for lab, a, hp in out:
        if lab is not None and lab[0] is not None and a != 0:
            th = m.things.get(lab[0])
            sp = m.lifespan(th, self.K) if th is not None else None
            if sp is not None and not sp.contains(day):
                raise refuse('RUN-LIFESPAN', '%s labels a part of %s to %s outside its lifespan' % (mv.id, place,
                                                                                                    lab[0]))
    return out


def w_batch_mv(self):
    return self.__dict__.setdefault('_batch_mv', {})


def w_pshares(self, place, pk, mv, day, coll):
    """The arrival rule's shares for movement mv: [(party, share)] known, the parties whose shares are holes, and their
    questions. A refusal of the rule (RUN-NO-RULE among them) refuses the arrival (L §3.6: "No rule is a refusal")."""
    m = self.m
    eng = self.eng
    did = pk.decl.owned.d
    d = m.defs[did]
    pkind = d.binds[1].dom[1]
    mkind = d.binds[0].dom[1]
    if mv.kind not in m.kinds[mkind].subtree(m):
        raise refuse('RUN-NO-RULE', 'no rule applies to %s[%s, *] on %s: %s is no movement of %s' % (
            did, mv.id, day, mv.id, mkind))
    gk = self.__dict__.get('_pgkey') or (day.toordinal(), 2, 8, '', 0)
    ctx = self.tctx.but(law=day, walk=self, wp=gk)
    known = []
    holes = []
    qs = set()
    mvv = mv.mv()
    prev_arr = self.__dict__.get('in_arrival')
    self.in_arrival = (did, mv.id)
    try:
        vals = []
        for p in self.pparties(pkind):
            c = _coll()
            eng.stack.append(c)
            try:
                vals.append((p, eng.read_def(d, [mvv, Thing(p)], ctx)))
            finally:
                eng.stack.pop()
            coll.merge(c)
    finally:
        self.in_arrival = prev_arr
    for p, v in vals:
        if isinstance(v, Hole):
            holes.append(p)
            qs |= set(v.qs)
            continue
        if v is NONE:
            continue
        q = v.q if isinstance(v, Num) else None
        if q is None:
            raise refuse('RUN-DOMAIN', 'the share %s[%s, %s] is %s, not a number' % (did, mv.id, p, fmt(v)))
        if q < 0:
            # L §3.6 (2.1-F4): "The shares of one arrival are each at least 0 and sum to exactly 1, or the arrival is
            # refused (RUN-LABEL ...)"
            raise refuse('RUN-LABEL', 'the share of %s in %s\'s arrival at %s is below 0' % (p, mv.id, place))
        if q != 0:
            known.append((p, q))
    s = sum((q for p, q in known), Fraction(0))
    if not holes and s != 1:
        raise refuse('RUN-LABEL', 'the shares of %s\'s arrival at %s sum to %s, not 1' % (mv.id, place, fmt(Num(s))))
    if holes and s > 1:
        raise refuse('RUN-LABEL', 'the shares of %s\'s arrival at %s sum to %s, not 1' % (mv.id, place, fmt(Num(s))))
    return known, holes, frozenset(qs)


def w_psplit_owners(self, place, unit, pk, step, mv, b, amt, shares, hparts, hqs):
    """Split one portion by the owners' shares, rest largest to the step (L §3.6 Its split; §5.13); the remainder of
    hole shares is one hole part (L §3.6: "Shares that are holes leave one hole part of the remainder, 1 less the known
    shares, whose possible owners are the parties whose shares are holes")."""
    keys = [p for p, s in shares]
    ws = [s for p, s in shares]
    hole_w = Fraction(1) - sum(ws, Fraction(0)) if hparts else Fraction(0)
    if hparts and hole_w > 0:
        keys.append(None)
        ws.append(hole_w)
    owned = pk.decl.owned
    d = self.m.defs[owned.d]

    def tie(tied):
        # only parties come here: "The hole part of hole shares ranks after every party" (L §3.6, 2.1-F5; psplit last)
        if owned.tie is None:
            raise refuse('RUN-TIE', 'the split of %s at %s ties at %s' % (mv.id, place, ', '.join(
                keys[i] for i in tied)))
        out = []
        for i in tied:
            env = {d.binds[0].var: mv.mv(), d.binds[1].var: Thing(keys[i])}
            out.append([self.eng.ev(x, env, self.tctx.but(law=mv.day if isinstance(mv.day, datetime.date) else None))
                        for x in owned.tie])
        return out
    amts = self.psplit(amt, ws, step, tie, lambda: 'the split of %s at %s' % (mv.id, place),
                       last=set(i for i, k in enumerate(keys) if k is None))
    out = []
    for k, a in zip(keys, amts):
        if a == 0:
            continue
        if k is None:
            out.append((None, a, HolePart(a, hqs, [(p, b if b is not None else None) for p in hparts])))
        else:
            out.append(((k, b), a, None))
    return out


def w_psplit(self, total, ws, step, tie, what, last=()):
    """L §5.13 rest largest at step u: each raw share rounded toward zero to u, the leftover units to the largest
    remainders, ties by `tie(tied indices)` (a list of term values per index, lower first), RUN-TIE otherwise; the
    indices in last rank after every other in a tie (the hole part of hole shares, L §3.6)."""
    tot_w = sum(ws, Fraction(0))
    if (total / step).denominator != 1:
        raise refuse('RUN-PRORATA', '%s isn\'t a whole number of %s' % (fmt(Num(total)), fmt(Num(step))))
    if tot_w == 0:
        raise refuse('RUN-WEIGHTS', '%s: the weights sum to 0' % what())
    raw = [total * w / tot_w for w in ws]
    down = [math.floor(r / step) for r in raw]
    rems = [r - d * step for r, d in zip(raw, down)]
    left = int(total / step) - sum(down)
    order = sorted(range(len(raw)), key=lambda i: -rems[i])
    give = set(order[:left])
    if 0 < left < len(raw):
        edge = rems[order[left - 1]]
        tied = [i for i in range(len(raw)) if rems[i] == edge]
        inside = [i for i in tied if i in give]
        if len(inside) != len(tied) and any(i in last for i in tied):
            first = [i for i in tied if i not in last]
            if len(inside) >= len(first):
                rest_ = [i for i in tied if i in last][:len(inside) - len(first)]
                give = (give - set(tied)) | set(first) | set(rest_)
                tied = inside = []
            else:
                give = (give - set(tied)) | set(first[:len(inside)])
                inside = [i for i in first if i in give]
                tied = first
        if len(inside) != len(tied):
            vals = tie(tied)
            for vs in vals:
                for v in vs:
                    if isinstance(v, Hole):
                        raise _HoleTie(v)
            scored = list(zip(vals, tied))
            eng = self.eng

            def cmp(a, c):
                for x, y in zip(a[0], c[0]):
                    r = eng.cmp3(x, y, None)
                    if r:
                        return r
                return 0
            scored.sort(key=functools.cmp_to_key(cmp))
            need = len(inside)
            if need < len(scored) and cmp(scored[need - 1], scored[need]) == 0:
                raise refuse('RUN-TIE', '%s ties at %s, and its ties by doesn\'t break it' % (what(), ', '.join(
                    str(i) for i in tied)))
            give = (give - set(tied)) | set(i for v, i in scored[:need])
    return [(down[i] + (1 if i in give else 0)) * step for i in range(len(raw))]


class _HoleTie(Exception):
    def __init__(self, h):
        self.h = h


def w_ptake_of(self, mv, lg, key):
    """The take of leg lg of mv at its from place, taken now if its group is still queued (a kept batch, a leg read)."""
    frm = lg.frm
    tk = self.__dict__.get('ptake', {}).get((mv.id, lg.name))
    if tk is None and self.__dict__.get('_preltry') == (mv.id, lg.name):
        raise _NeedsTake()
    cyc = self.__dict__.get('_pcyc')
    if tk is None and cyc is not None and isinstance(frm, Thing) and (frm.name, lg.unit) in cyc.members and \
            point(key) == cyc.pt:
        stack = self.__dict__.get('pstack', [])
        if stack and stack[-1] not in cyc.members:
            # a group outside the cycle reads one of its takes: it is in the cycle too
            raise PartsCycle((frm.name, lg.unit), cyc.pt)
        # the cycle's arrival as a part of unknown labels (L §5.14.2)
        tk = Take(frm.name)
        if isinstance(lg.amount, Num) and lg.amount.q != 0:
            tk.holes.append(HolePart(lg.amount.q, [(cyc.q, frm.name)], [(ANY, ANY)]))
        return tk
    if tk is None and isinstance(frm, Thing) and self.pk(frm.name) is not None:
        self.pneed(frm.name, lg.unit, key)
        tk = self.__dict__.get('ptake', {}).get((mv.id, lg.name))
        if tk is None:
            tk = self.ptake_same(mv, lg)
    if tk is None:
        if isinstance(lg.amount, Num) and lg.amount.q == 0:
            return Take(frm.name if isinstance(frm, Thing) else None)
        bad = self.bad_legs.get((mv.id, lg.name))
        if bad is not None:
            raise bad
        raise Unspecified('§5.14', 'the take of %s.%s, a leg the walk didn\'t book at %s' % (
            mv.id, lg.name, frm.name if isinstance(frm, Thing) else '?'))
    return tk


def w_ptake_same(self, mv, lg):
    """L §5.14: "Two observed rows stated `same` are one movement (§5.19), so its take at the `from` end and its arrival
    at the `to` end belong together": the take of mv's leg is the one its group booked at that place (a leg of the same
    name first, then any leg out of that place in that unit), or None."""
    pr = self.same_pairs().get(getattr(mv, 'id', None))
    if pr is None:
        return None
    frm = lg.frm
    takes = self.__dict__.get('ptake', {})
    for x in pr[0]:
        if x == mv.id:
            continue
        o = self.decs.get(x) or self.movement(x)
        if o is None:
            continue
        legs = sorted(o.legs, key=lambda g: g.name != lg.name)
        for g in legs:
            if isinstance(g.frm, Thing) and g.frm.name == frm.name and (g.unit or '*') == (lg.unit or '*'):
                tk = takes.get((x, g.name))
                if tk is not None:
                    return tk
                bad = self.bad_legs.get((x, g.name))
                if bad is not None:
                    raise bad
    return None


# -- the fill over (part, outflow) arcs (L §5.14 The take is a fill over arcs) ---------------------------------------

def w_ppart_rec(self, s):
    """A supply as the takes clause's k: a record of its built-in fields owner and batch (L §3.6); a field of a hole
    part is known where every possible label gives it one value, a hole otherwise (L §8.1)."""
    bm = self.batch_mv()

    def bval(b):
        if b is None:
            return NONE
        mv = bm.get(b)
        return mv if mv is not None else Mv(b)
    if s.hole is None:
        o, b = s.lab
        return Rec([('owner', Thing(o) if o is not None else NONE), ('batch', bval(b))])
    os_ = set(p[0] for p in s.hole.poss)
    bs = set(p[1] for p in s.hole.poss)
    h = Hole(s.hole.qs) if s.hole.qs else Hole([('the label of a hole part', '?')])
    ov = h if (len(os_) != 1 or ANY in os_) else (Thing(list(os_)[0]) if list(os_)[0] is not None else NONE)
    bv = h if (len(bs) != 1 or ANY in bs) else bval(list(bs)[0])
    return Rec([('owner', ov), ('batch', bv)])


def w_pterms(self, pk, s, w_mv, day, key, coll):
    """The takes clause's order terms for arc (part s, outflow w): a tuple of values, a Hole where a term is one."""
    tk = pk.decl.takes
    if not tk.order:
        return ()
    eng = self.eng
    env = {tk.part: self.ppart_rec(s), tk.w: w_mv if w_mv is not None else Hole([('the outflow nobody knows', '?')])}
    ctx = self.tctx.but(law=day, walk=self, wp=key)
    c = _coll()
    eng.stack.append(c)
    try:
        vals = tuple(eng.ev(e, env, ctx) for e in tk.order)
    finally:
        eng.stack.pop()
    coll.merge(c)
    if isinstance(w_mv, Mv) and any(isinstance(v, Hole) for v in vals):
        vals = self.pterms_readings(tk, env, ctx, w_mv, vals)
    return vals


def w_pterms_readings(self, tk, env, ctx, w_mv, vals):
    """L §5.14 A hole in a term: the take's reach is "under some reading of the holes". An attribute of w that is a
    hole stands for a value of its declared type, so where it is a key of a kind (or a bool), each thing of that kind
    is one reading of it; a term that every reading gives one value is that value, as an order term over a hole part
    is known where every possible label gives it one value (L §8.1). Other holes stay holes. The readings are
    evaluated apart: what they cache is dropped, and they leave no register, trace or walk record (writer 11)."""
    import itertools
    import collections
    eng = self.eng
    m = self.m
    src = eng.mv_record(w_mv, ctx)
    attrs = getattr(src, 'attrs', None) or {}
    holes = [a for a, v in sorted(attrs.items()) if isinstance(v, Hole)]
    if not holes or getattr(src, 'kind', None) not in m.kinds:
        return vals
    types = dict(m.kind_attrs(src.kind))
    axes = []
    for a in holes:
        t = types.get(a)
        if t is None:
            continue
        if t.k == 'bool':
            axes.append((a, [True, False]))
        elif t.k == 'key' and t.ix in m.kinds:
            sub = m.kinds[t.ix].subtree(m)
            things = [Thing(x.name) for x in sorted(m.things.values(), key=lambda x: x.name) if x.base in sub or
                      any(getattr(e, 'kind', None) in sub for e in getattr(x, 'entries', ()))]
            if things:
                axes.append((a, things))
    if not axes:
        return vals
    import copy
    from .values import vkey
    per = []
    old_memo, old_tr, old_wb = eng.memo, eng.tracing, getattr(eng, '_explaining_wb', False)
    eng.memo = collections.ChainMap({}, old_memo)
    eng.tracing = False
    eng._explaining_wb = True
    try:
        for combo in itertools.product(*[vs for a, vs in axes]):
            rec = copy.copy(src)
            rec.attrs = dict(attrs)
            for (a, vs), v in zip(axes, combo):
                rec.attrs[a] = v
            if hasattr(rec, 'ran'):
                rec.ran = True
            env2 = dict(env)
            env2[tk.w] = Mv(w_mv.id, rec)
            eng.stack.append(_coll())
            try:
                row = []
                for e in tk.order:
                    try:
                        row.append(eng.ev(e, env2, ctx))
                    except TallyError as err:
                        if load_now(err):
                            raise
                        row.append(None)
                per.append(row)
            finally:
                eng.stack.pop()
    finally:
        eng.memo, eng.tracing, eng._explaining_wb = old_memo, old_tr, old_wb
    out = []
    for i, v in enumerate(vals):
        if isinstance(v, Hole):
            got = [r[i] for r in per]
            if got and all(g is not None and not isinstance(g, Hole) for g in got) and \
                    len(set(vkey(g) for g in got)) == 1:
                v = got[0]
        out.append(v)
    return tuple(out)


def w_pfill(self, place, unit, pk, step, supplies, outflows, key, coll, bad):
    """The take: §5.12's steps 2-4 over (part, outflow) arcs, sorted by the order's terms; ties as L §5.14 says. "A
    hole in a term ... makes the arcs it ranks undecided. Only the parts the take could reach under some reading of the
    holes merge ...; a part ranked after enough certainly-ranked parts to cover the outflow stays known": the arcs are
    taken in order while every undecided arc still live is certainly ranked after them; from the first that may not
    be, what is left of the take is undecided (pundecided). Parts of one label are one supply for ties: their order
    changes no take."""
    from .walk import kday
    eng = self.eng
    tk = pk.decl.takes
    day = kday(key)
    outs = []
    for e, lg in outflows:
        if (e.mv.id, lg.name) in bad:
            continue
        outs.append([e, lg, -e.flow.q])
    for o in outs:
        if (o[2] / step).denominator != 1:
            bad[(o[0].mv.id, o[1].name)] = refuse('RUN-PRORATA', '%s taken from %s isn\'t a whole number of its step '
                                                                 '%s' % (fmt(qty(o[2], unit)), place,
                                                                         fmt(qty(step, unit))))
    outs = [o for o in outs if (o[0].mv.id, o[1].name) not in bad]
    if not outs:
        return
    # arcs: (supply index, outflow index, terms)
    certain = []
    unsure = []
    for si, s in enumerate(supplies):
        for oi, o in enumerate(outs):
            if s.src is not None and s.src == o[0].mv.id:
                continue        # "A movement's take never includes its own arrivals"
            try:
                t = self.pterms(pk, s, o[0].mv.mv(), day, key, coll)
            except TallyError as err:
                if load_now(err):
                    raise
                bad[(o[0].mv.id, o[1].name)] = err
                continue
            (unsure if any(isinstance(v, Hole) for v in t) or s.amt is UNK else certain).append((si, oi, t))
    if any((o[0].mv.id, o[1].name) in bad for o in outs):
        return
    takes = self.__dict__.setdefault('ptake', {})
    for o in outs:
        takes[(o[0].mv.id, o[1].name)] = Take(place)

    def cmp(a, b):
        for x, y in zip(a[2], b[2]):
            r = eng.cmp3(x, y, None)
            if r:
                return r
        return 0

    def after(u, t):
        """Whether undecided arc u is ranked after terms t under every reading of its holes."""
        for x, y in zip(u[2], t):
            if isinstance(x, Hole) or isinstance(y, Hole):
                return False
            r = eng.cmp3(x, y, None)
            if r:
                return r > 0
        return False
    certain.sort(key=functools.cmp_to_key(cmp))
    classes = []
    for a in certain:
        if classes and cmp(classes[-1][0], a) == 0:
            classes[-1].append(a)
        else:
            classes.append([a])
    S = [s.amt for s in supplies]
    D = [o[2] for o in outs]

    def take(si, oi, a):
        if a == 0:
            return
        S[si] -= a
        D[oi] -= a
        s = supplies[si]
        rec = takes[(outs[oi][0].mv.id, outs[oi][1].name)]
        if s.hole is not None:
            rec.holes.append(HolePart(a, s.hole.qs, s.hole.poss, s.hole.excl, s.hole.sure))
        else:
            rec.known[s.lab] = rec.known.get(s.lab, Fraction(0)) + a

    def live(a):
        return S[a[0]] > 0 and D[a[1]] > 0

    def group(si):
        s = supplies[si]
        return ('lab', s.lab) if s.hole is None else ('hole', si)
    def before(t1, t2):
        """Whether terms t1 rank before t2 under every reading of their holes: the order is lexicographic (§5.12), so
        a known first difference decides whatever the later terms are (writer 11, the migrator's item 8)."""
        for x, y in zip(t1, t2):
            if isinstance(x, Hole) or isinstance(y, Hole):
                return False
            r = eng.cmp3(x, y, None)
            if r:
                return r < 0
        return False
    undecided = False
    ci = 0
    while True:
        while ci < len(classes) and not any(live(a) for a in classes[ci]):
            ci += 1
        cl = [a for a in classes[ci] if live(a)] if ci < len(classes) else []
        lu = [u for u in unsure if live(u)]
        if cl and all(after(u, cl[0][2]) for u in lu):
            ci += 1
        else:
            # an undecided arc whose known first terms rank it before every other live arc is taken first under every
            # reading of its holes: a class of its own, with the arcs of its outflow from parts of its label (their
            # terms are its terms, and their order changes no take). The hole balance's part never is (its amount is
            # unknown).
            pick = None
            for u in lu:
                if supplies[u[0]].amt is UNK:
                    continue
                mates = [b for b in lu if b[1] == u[1] and group(b[0]) == group(u[0])]
                ids = set(id(b) for b in mates)
                if all(id(b) in ids or before(u[2], b[2]) for b in lu) and (not cl or before(u[2], cl[0][2])):
                    pick = mates
                    break
            if pick is None:
                if lu:
                    undecided = True
                break
            cl = pick
        by_part = {}
        by_out = {}
        for a in cl:
            by_part.setdefault(a[0], []).append(a[1])
            by_out.setdefault(a[1], []).append(a[0])
        # two outflows tied for one part that can't cover both (RC-3: "Two outflows never share a part"); parts of one
        # label are one part
        by_grp = {}
        for si, os_ in by_part.items():
            by_grp.setdefault(group(si), [set(), Fraction(0)])
            by_grp[group(si)][0].update(os_)
            by_grp[group(si)][1] += S[si]
        for g, (os_, have) in by_grp.items():
            if len(os_) > 1 and have < sum(D[oi] for oi in os_):
                names = sorted(set(outs[oi][0].mv.id for oi in os_))
                err = refuse('RUN-TIE', '%s tie for a part of %s that can\'t cover both' % (', '.join(names), place))
                for oi in os_:
                    bad[(outs[oi][0].mv.id, outs[oi][1].name)] = err
                return
        for oi, ps in sorted(by_out.items()):
            grps = []
            for si in ps:
                g = group(si)
                for x in grps:
                    if x[0] == g:
                        x[1].append(si)
                        break
                else:
                    grps.append((g, [si]))
            need = D[oi]
            have = sum(S[si] for si in ps)
            if len(grps) == 1 or need >= have:
                for si in ps:
                    take(si, oi, min(S[si], D[oi]))
                continue
            shared = any(len(by_grp[group(si)][0]) > 1 for si in ps)
            if not tk.prorata or shared:
                err = refuse('RUN-TIE', 'the take of %s at %s ties at %s that changes the result' % (
                    outs[oi][0].mv.id, place, ', '.join(self.plab_text(supplies[x[1][0]]) for x in grps)))
                bad[(outs[oi][0].mv.id, outs[oi][1].name)] = err
                return
            # prorata: "shares one outflow among its parts tied on every term, weighted by their amounts, by rest
            # largest at the kind's step, its remainder's ties going by its ties by over k"
            ws = [sum(S[si] for si in x[1]) for x in grps]

            def tie(tied, grps=grps, oi=oi):
                if tk.tie is None:
                    raise refuse('RUN-TIE', 'the split of %s at %s ties at %s' % (
                        outs[oi][0].mv.id, place, ', '.join(self.plab_text(supplies[grps[i][1][0]]) for i in tied)))
                out = []
                ctx = self.tctx.but(law=day)
                for i in tied:
                    env = {tk.part: self.ppart_rec(supplies[grps[i][1][0]])}
                    out.append([eng.ev(x, env, ctx) for x in tk.tie])
                return out
            try:
                amts = self.psplit(need, ws, step, tie, lambda oi=oi: 'the split of %s at %s' % (outs[oi][0].mv.id,
                                                                                              place))
            except _HoleTie:
                raise NotYet('parts: a prorata take whose ties by is a hole')
            except TallyError as err:
                if load_now(err):
                    raise
                bad[(outs[oi][0].mv.id, outs[oi][1].name)] = err
                return
            for x, a in zip(grps, amts):
                for si in x[1]:
                    t_ = min(S[si], a)
                    take(si, oi, t_)
                    a -= t_
    if not undecided and any(live(u) for u in unsure):
        undecided = True
    if undecided:
        for si, s in enumerate(supplies):
            s.amt = S[si]
        rem = [oi for oi in range(len(outs)) if D[oi] > 0]
        idx = dict((oi, j) for j, oi in enumerate(rem))
        outs2 = [[outs[oi][0], outs[oi][1], D[oi]] for oi in rem]
        arcs2 = [(a[0], idx[a[1]], a[2]) for a in certain + unsure if a[1] in idx and S[a[0]] > 0]
        self.pundecided(place, unit, supplies, outs2, arcs2, coll)
        return
    for oi, o in enumerate(outs):
        if D[oi] > 0:
            hold = self.m.things[place].base in self.m.kinds['holdings'].subtree(self.m)
            bad[(o[0].mv.id, o[1].name)] = refuse('RUN-PLACE', '%s takes %s below 0%s' % (
                o[0].mv.id, place, ' units' if hold else ''))
            return
    for si, s in enumerate(supplies):
        s.amt = S[si]


def w_plab_text(self, s):
    if s.hole is not None:
        return 'a hole part'
    o, b = s.lab
    if o is not None and b is not None:
        return '(%s, %s)' % (o, b)
    return o if o is not None else b


def w_pundecided(self, place, unit, supplies, outs, arcs, coll):
    """L §5.14 A hole in a term: "Only the parts the take could reach under some reading of the holes merge, into one
    hole part with the total known; a part ranked after enough certainly-ranked parts to cover the outflow stays known".
    A part is out of reach when, for every outflow, the parts certainly ranked before it cover every outflow of the
    take together."""
    eng = self.eng
    total = sum(o[2] for o in outs)
    qs = set()
    for si, oi, t in arcs:
        for v in t:
            if isinstance(v, Hole):
                qs |= set(v.qs)

    def before(t1, t2):
        # lexicographic under every reading of the holes: a known first difference decides (§5.12)
        for x, y in zip(t1, t2):
            if isinstance(x, Hole) or isinstance(y, Hole):
                return False
            r = eng.cmp3(x, y, None)
            if r:
                return r < 0
        return False
    reach = set()
    for si, s in enumerate(supplies):
        if s.amt is UNK:
            # the hole balance's part (2.1-F10) is always within reach and never covers what is ranked after it
            if any(x == si for x, oi, t in arcs):
                reach.add(si)
            continue
        if s.amt <= 0:
            continue
        safe = True
        for x, oi, t in arcs:
            if x != si:
                continue
            ahead = sum(supplies[y].amt for y, o2, t2 in arcs
                        if o2 == oi and y != si and supplies[y].amt is not UNK and before(t2, t))
            if ahead < total:
                safe = False
                break
        if not safe:
            reach.add(si)
    amt = sum(supplies[si].amt for si in reach)
    if amt < total:
        hold = self.m.things[place].base in self.m.kinds['holdings'].subtree(self.m)
        raise refuse('RUN-PLACE', '%s takes %s below 0%s' % (', '.join(sorted(set(o[0].mv.id for o in outs))),
                                                            place, ' units' if hold else ''))
    poss, excl = merge_labels([supplies[si] for si in reach], qs)
    takes = self.ptake
    for o in outs:
        takes[(o[0].mv.id, o[1].name)].holes.append(HolePart(o[2], qs, poss, excl))
    merged = None
    for si in reach:
        supplies[si].amt = Fraction(0)
    if amt - total > 0:
        merged = Supply(None, amt - total, hole=HolePart(amt - total, qs, poss, excl))
        supplies.append(merged)


def merge_labels(sups, qs):
    """The possible labels of a merge of supplies (L §8.1: "their possible labels joined"), and what it still can't be:
    what every hole part merged can't be, but a label a known part merged has."""
    poss = set()
    excl = None
    known = []
    for s in sups:
        if s.hole is not None:
            poss |= set(s.hole.poss)
            qs |= set(s.hole.qs)
            excl = set(s.hole.excl) if excl is None else excl & set(s.hole.excl)
        else:
            poss.add(s.lab)
            known.append(s.lab)
    excl = excl or set()
    for lab in known:
        excl = set(x for x in excl if not ((x[0] in (ANY, lab[0])) and (x[1] in (ANY, lab[1]))))
    return poss, excl


def w_pcycle(self, place, unit, grp, prev, c):
    """A cycle among the takes of one batch (L §5.14.2 Takes in the first batch): "each take is tried with the cycle's
    arrivals as parts of unknown labels, and a take whose outcome is the same under every reading of them ... is
    decided, which breaks the cycle. Otherwise a cycle that holds a plan is refused (RUN-CYCLE, naming the movements)
    ... A cycle of observed rows only is an unknown fact ...: the takes it reaches are undecided, each place holds a
    hole part with its total known". Returns None when takes were decided (the group is taken again), else the
    group's (state, refused legs)."""
    from .walk import kday
    members = set(c.members) | {(place, unit)}
    pt = c.pt
    for _ in range(64):
        try:
            decided, links = self.ptry(place, unit, grp, prev, members, pt)
            break
        except PartsCycle as c2:
            if c2.pt != pt:
                raise
            grown = members | set(c2.members) | {c2.pu}
            if grown == members:
                raise NotYet('parts: a cycle among takes whose groups don\'t settle')
            members = grown
    else:
        raise NotYet('parts: a cycle among takes whose groups don\'t settle')
    if decided:
        self.__dict__.setdefault('ptake', {}).update(decided)
        self.__dict__.setdefault('_pdecided', set()).update(decided)
        return None
    mvs = sorted(set(e.mv.id for e in links))
    places = sorted(set(pu[0] for pu in members))
    if any(e.key[1] == 2 for e in links):
        err = refuse('RUN-CYCLE', 'the takes of %s at %s read each other in one batch' % (', '.join(mvs),
                                                                                        ', '.join(places)))
        refused = dict(((e.mv.id, e.note), err) for e in links if e in grp)
        return self.prun(place, unit, grp, prev, refused)
    if any(e.key[1] != 1 for e in links):
        raise Unspecified('§5.14.2', 'a cycle among the takes of %s at %s, at a decision\'s point, that no take '
                                     'breaks' % (', '.join(mvs), ', '.join(places)))
    q = 'which of %s went first on %s?' % (', '.join(mvs), kday(grp[-1].key).isoformat())
    prevc = self.__dict__.get('_pcyc')
    self._pcyc = Cycle(members, pt, q, 'hole')
    try:
        return self.prun(place, unit, grp, prev, {})
    finally:
        self._pcyc = prevc


def w_ptry(self, place, unit, grp, prev, members, pt):
    """Each group of the cycle taken with the cycle's arrivals as parts of unknown labels, nothing kept: the takes with
    no hole part are decided. Also the cycle's links, the legs between two of its places."""
    from .walk import kday
    places = set(pu[0] for pu in members)
    groups = {}
    links = []
    for mu in sorted(members, key=str):
        if mu == (place, unit):
            mg, mp = grp, prev
        else:
            log = self.logs.get(mu)
            live = set(id(e) for e in log.ents) if log is not None else set()
            mg = sorted((e for e in self.__dict__.get('ppend', {}).get(mu, []) if point(e.key) == pt and id(e) in
                         live), key=lambda e: e.key)
            mp = self.plast(mu[0], mu[1])
        groups[mu] = (mg, mp)
        for e in mg:
            if e.kind != 'delta' or e.mv is None:
                continue
            lg = _leg_of(e.mv, e.note)
            if lg is not None and isinstance(lg.frm, Thing) and isinstance(lg.to, Thing) and \
                    lg.frm.name in places and lg.to.name in places and lg.frm.name != lg.to.name:
                links.append(e)
    q = 'which of %s went first on %s?' % (', '.join(sorted(set(e.mv.id for e in links))), kday(pt).isoformat())
    prevc = self.__dict__.get('_pcyc')
    saved = self.__dict__.get('ptake', {})
    decided = {}
    stack = self.__dict__.setdefault('pstack', [])
    busy = self.__dict__.setdefault('pbusy', {})
    for mu in sorted(members, key=str):
        mg, mp = groups[mu]
        if not mg:
            continue
        self._pcyc = Cycle(members, pt, q, 'try')
        self.ptake = dict(saved)
        stack.append(mu)
        was = busy.get(mu)
        busy[mu] = pt
        try:
            self.pgroup_run(mu[0], mu[1], mg, mp, {})
            tried = self.ptake
        finally:
            self.ptake = saved
            self._pcyc = prevc
            stack.pop()
            if was is None:
                busy.pop(mu, None)
            else:
                busy[mu] = was
        for e in mg:
            if e.kind == 'delta' and e.mv is not None and isinstance(e.flow, Num) and e.flow.q < 0:
                tk = tried.get((e.mv.id, e.note))
                if tk is not None and tk is not saved.get((e.mv.id, e.note)) and not tk.holes:
                    decided[(e.mv.id, e.note)] = tk
    return decided, links


# -- reads (L §5.10 bal ... of q, value ... of q; §5.14 m.leg.amount of q) --------------------------------------------

def eng_of_value(self, node, env, ctx):
    """q of `of q`: ('owner', party) | ('batch', movement id) | ('pair', party, movement id), or a Hole."""
    if node.k == 'ofpair':
        a = self.ev(node.a, env, ctx)
        b = self.ev(node.b, env, ctx)
        for v in (a, b):
            if isinstance(v, Hole):
                return v
        if not isinstance(a, Thing) or not isinstance(b, Mv):
            raise refuse('RUN-DOMAIN', 'of (%s, %s): a pair is a party and a batch' % (fmt(a), fmt(b)))
        return ('pair', a.name, b.id)
    v = self.ev(node, env, ctx)
    if isinstance(v, Hole):
        return v
    if v is NONE:
        raise refuse('RUN-NONE', 'of %s: it is none here' % fmt(v))
    if isinstance(v, Thing):
        return ('owner', v.name)
    if isinstance(v, Mv):
        return ('batch', v.id)
    raise refuse('RUN-DOMAIN', 'of %s: a party, a batch or the pair' % fmt(v))


def _qtext(q):
    if q[0] == 'pair':
        return '(%s, %s)' % (q[1], q[2])
    return q[1]


def check_sort(m, pk, place, q):
    labs = labels_of(pk) if pk is not None else set()
    need = {'owner': {'owner'}, 'batch': {'batch'}, 'pair': {'owner', 'batch'}}[q[0]]
    if not need <= labs:
        raise refuse('RUN-DOMAIN', 'of %s at %s, which keeps no labels of its sort' % (_qtext(q), place))


def lab_hits(lab, q):
    o, b = lab
    if q[0] == 'owner':
        return o == q[1]
    if q[0] == 'batch':
        return b == q[1]
    return o == q[1] and b == q[2]


def poss_hits(poss, q):
    for o, b in poss:
        if q[0] == 'owner' and (o == q[1] or o == ANY):
            return True
        if q[0] == 'batch' and (b == q[1] or b == ANY):
            return True
        if q[0] == 'pair' and (o == q[1] or o == ANY) and (b == q[2] or b == ANY):
            return True
    return False


def poss_all(poss, q):
    """Whether every possible label of a hole part is one q names (L §8.1: a read of q is a hole only for a q the part
    may be, and isn't known to be)."""
    if not poss:
        return False
    for o, b in poss:
        if q[0] == 'owner' and o != q[1]:
            return False
        if q[0] == 'batch' and b != q[1]:
            return False
        if q[0] == 'pair' and (o != q[1] or b != q[2]):
            return False
    return True


def excl_hits(excl, q):
    """Whether every label q names is one a hole part can't be (its excluded patterns)."""
    for o, b in excl:
        if q[0] == 'owner' and o == q[1] and b == ANY:
            return True
        if q[0] == 'batch' and b == q[1] and o == ANY:
            return True
        if q[0] == 'pair' and ((o == q[1] and b in (q[2], ANY)) or (o == ANY and b == q[2])):
            return True
    return False


def parts_value(place, unit, known, holes, q, money):
    """The sum of the parts labelled q; a hole where a hole part may be q's (HOLE-PART)."""
    tot = Fraction(0)
    for lab, a in known.items():
        if lab_hits(lab, q):
            tot += a
    qs = set()
    for h in holes:
        if h.sure and poss_all(h.poss, q):
            tot += h.amt        # every label it may have is q's (a stated part's gap at a place that is both)
            continue
        if poss_hits(h.poss, q) and not excl_hits(h.excl, q):
            inner = '; '.join(sorted(set(x[0] for x in h.qs))) if h.qs else 'no question'
            msg = '%s: %s may be %s\'s: %s' % (place, fmt(qty(h.amt, unit)), _qtext(q), inner)
            _ROOTS[msg] = h.qs
            qs.add((msg, place))
    if qs:
        return Hole(qs)
    return qty(tot, unit)


def eng_bal_of(self, w, place, unit, key, q, flush=True):
    """`bal(x, d) of q` at an instant of walk w: the balance first (its refusal or hole is the answer), then the parts."""
    m = self.m
    pk = thing_parts(m, m.things.get(place))
    check_sort(m, pk, place, q)
    v = self.read_state(w, place, unit, key)
    if isinstance(v, Hole):
        return v
    if flush and getattr(w, 'running', False):
        w.parts_flush(key)
    st = w.plog_state(place, unit, key)
    if st.coll is not None:
        self.cur.merge(st.coll)
    if st.bad is not None:
        raise st.bad
    return parts_value(place, unit, st.known, st.holes, q, None)


def eng_e_legof(self, n, env, ctx):
    """`m.leg.amount of q` (L §5.14 The leg read): what leg `leg` of movement m took at its from place from the parts
    labelled q."""
    f = n.e
    if f.k != 'field' or f.name != 'amount' or f.obj.k != 'field':
        raise NotYet('of after a field other than a leg\'s amount')
    leg = f.obj.name
    mvv = self.ev(f.obj.obj, env, ctx)
    if isinstance(mvv, Hole):
        return mvv
    q = self.of_value(n.q, env, ctx)
    if isinstance(q, Hole):
        return q
    if not isinstance(mvv, Mv):
        raise refuse('RUN-DOMAIN', '%s is not a movement' % fmt(mvv))
    rec = self.ev(f.obj, env, ctx)        # the leg's record: its refusals, holes and register as m.leg reads them
    if isinstance(rec, Hole):
        return rec
    frm = rec.get('from')
    amt = rec.get('amount')
    if isinstance(frm, Hole):
        return frm
    if isinstance(amt, Hole):
        return amt
    if not isinstance(frm, Thing):
        raise refuse('RUN-DOMAIN', 'the leg %s.%s has no place' % (mvv.id, leg))
    m = self.m
    pk = thing_parts(m, m.things.get(frm.name))
    unit = amt.u[0][0] if isinstance(amt, Num) and amt.u else None
    check_sort(m, pk, frm.name, q)
    w = ctx.walk if ctx.walk is not None else self.main_walk(ctx)
    src = mvv.src if mvv.src is not None else self.mv_record(mvv, ctx)
    lg = _leg_of(src, leg) if src is not None else None
    if lg is None:
        raise refuse('RUN-RECORD', '%s has no leg %s' % (mvv.id, leg))
    if isinstance(amt, Num) and amt.q == 0:
        return qty(0, unit)
    key = ctx.wp if ctx.wp is not None else None
    if key is None:
        from .walk import K_end
        d = lg.day if isinstance(lg.day, datetime.date) else None
        key = K_end(d) if d is not None else (10 ** 9,)
    tk = w.ptake_of(src if not hasattr(src, 'ran') else src, lg, key)
    if tk.bad is not None:
        raise tk.bad
    if tk.known or tk.holes:
        return parts_value(frm.name, unit, tk.known, tk.holes, q, None)
    return qty(0, unit)


def eng_e_bal_of(self, n, place, unit, env, ctx):
    from .walk import K_start, K_end
    from .mreads import note_read
    form = n._form
    q = self.of_value(n.of, env, ctx)
    if isinstance(q, Hole):
        return q
    if form == 'now':
        w = ctx.walk
        if w is None or ctx.wp is None:
            raise refuse('LOAD-READS', 'bal(%s) of %s with no day outside a decision' % (place, _qtext(q)))
        note_read(w, place, True)
        if getattr(w, 'less_obs', None):
            raise NotYet('a modelled-past decision\'s running read of parts (bal(x) of q)')
        return self.bal_of(w, place, unit, ctx.wp, q)
    if form in ('proj', 'modelled') or (n.proj and n.proj.get('uncertain')):
        raise NotYet('of on a projection, a modelled balance or with uncertain (bal(x, d) of q from t)')
    d = self.ev(n.args[-1], env, ctx)
    if isinstance(d, Hole):
        return d
    d = self.date_of(d, 'the day of bal')
    key = K_start(d) if n.start else K_end(d)
    return self.bal_of_day(place, unit, d, key, ctx, q)


def eng_bal_of_day(self, place, unit, d, key, ctx, q):
    """A plain read of parts on a day: at the ask's cut, or a running read in a decision or an arrival rule (L §5.14.1),
    with the cycle and future checks of a plain balance read (mreads.m_bal_day)."""
    from .walk import kday
    from .mreads import note_read
    w = ctx.walk
    if w is not None and ctx.wp is not None:
        t = kday(ctx.wp)
        note_read(w, place, False)
        arrival_cycle(w, ctx, key, place, d)
        if key[0] > ctx.wp[0] or (key[0] == ctx.wp[0] and key[1] >= 3):
            if d > t:
                raise refuse('RUN-FUTURE', 'a decision on %s reads %s at %s after its day' % (t, place, d))
            raise refuse('RUN-CYCLE', '%s reads itself through bal(%s, %s) of %s: a plain read of its own day' % (
                ctx.wp[3].split('|')[-1 if ctx.wp[3].count('|') > 1 else 0] or 'the booking', place, d, _qtext(q)))
        if getattr(w, 'less_obs', None):
            raise NotYet('a modelled-past decision\'s read of parts (bal(x, d) of q)')
        return self.bal_of(w, place, unit, key, q)
    w = self.main_walk(ctx)
    return self.bal_of(w, place, unit, key, q)


def w_labels_drop(self, lg):
    """L §3.6 Labels that leave the books: "A leg from an owned place to a place of the model that keeps no parts, and
    isn't outside, drops its labels"."""
    if not getattr(self.m, 'parts_kinds', None):
        return False
    frm, to = lg.frm, lg.to
    if not isinstance(frm, Thing) or not isinstance(to, Thing) or frm.name == to.name:
        return False
    pk = self.pk(frm.name)
    if pk is None or pk.decl.owned is None or self.pk(to.name) is not None:
        return False
    th = self.m.things.get(to.name)
    return th is not None and th.base is not None and not self.m.kind_outside(th.base)


def w_parts_close(self, d):
    """L §5.7 Parts and lifespans: "a party's parts must be gone by the end of its lifespan. That is checked after the
    whole order of the lifespan's last day"; E §4.2 step 5: "a party whose lifespan ends on t and that holds a part at
    the close of t refuses that part's place from t on (RUN-LIFESPAN)"."""
    from .walk import Ent
    from .model import SameDiffers
    pl = self.__dict__.get('plog')
    if not pl:
        return
    ends = self.__dict__.get('_pends')
    m = self.m
    if ends is None:
        ends = {}
        for name, th in m.things.items():
            if th.base is None or th.base not in m.kinds['parties'].subtree(m):
                continue
            try:
                sp = m.lifespan(th, self.K)
            except (SameDiffers, TallyError):
                continue
            if sp is not None and sp.end is not None:
                ends.setdefault(sp.end - datetime.timedelta(days=1), set()).add(name)
        self._pends = ends
    gone = ends.get(d)
    if not gone:
        return
    key = (d.toordinal(), 5, 1, '', 0)
    for (place, unit) in sorted(pl, key=str):
        st = self.plog_state(place, unit, key)
        holders = sorted(set(lab[0] for lab, a in st.known.items() if a != 0 and lab[0] in gone))
        if holders:
            err = refuse('RUN-LIFESPAN', '%s ends on %s holding parts of %s' % (', '.join(holders), d, place))
            self.add(place, unit, Ent(key, 'bad', err=err))


def arrival_cycle(w, ctx, key, place, d):
    """L §5.14.1: an arrival rule's read of its arrival's day's end or later is a cycle with the booking (RUN-CYCLE
    where load doesn't see the day); the start of the day is booked before anything that day."""
    arr = getattr(w, 'in_arrival', None)
    if arr is None:
        return
    if key[0] > ctx.wp[0] or (key[0] == ctx.wp[0] and key[1] >= 1):
        raise refuse('RUN-CYCLE', '%s[%s, *] reads itself through bal(%s, %s): the end of its arrival\'s day or later'
                     % (arr[0], arr[1], place, d))
