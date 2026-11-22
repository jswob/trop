"""The residual ledger (E §6.3): `ask q: residuals x span [by f]`.

The model at the ask's cut K (each plan instance as booked or modelled at its booking cut, the settlements dated in
the span, each decision as the walk at K ran it) against the observations as known on the model's `on` date, for
place x, on the days some statement covers x.

Installed as Engine methods (`m_<name>` -> `<name>`).
"""
import datetime
from fractions import Fraction

from .errors import TallyError, Refusal, Unspecified, at_run
from .values import Num, Hole, Thing, Text, Key, Iv, NONE, fmt, fmt_hole
from .dates import month_period
from .moves import Inst, qty, day_lo, merge, key_text, instant_text, order_key, same_groups, same_pairs
from .walk import K_start, K_end, kday


def install(E):
    for name, fn in list(globals().items()):
        if name.startswith('m_'):
            setattr(E, name[2:], fn)


def refuse(code, msg):
    return at_run(code, msg)


class RLine:
    """One line of the ledger: kind, its day, its group, model and observed amounts (Fraction, Hole or None), the
    difference, and its text."""

    def __init__(self, kind, day, group, model, observed, diff, text, order):
        self.kind = kind
        self.day = day
        self.group = group
        self.model = model
        self.observed = observed
        self.diff = diff
        self.text = text
        self.order = order


def _add(a, b):
    if a is None:
        return b
    if b is None:
        return a
    if isinstance(a, Hole) or isinstance(b, Hole):
        return merge(a, b) if isinstance(a, Hole) and isinstance(b, Hole) else (a if isinstance(a, Hole) else b)
    return a + b


def _sub(a, b):
    if isinstance(a, Hole) or isinstance(b, Hole):
        return merge(*[x for x in (a, b) if isinstance(x, Hole)])
    return (a or 0) - (b or 0)


def m_flow_into(self, legs, x, unit, only=False):
    """The movement's flow into x, signed (in +, out -), in x's unit: a Fraction or a Hole. With only (a ledger of
    one unit of a place of unit any), legs in another unit are another ledger's."""
    tot = Fraction(0)
    holes = []
    for lg in legs:
        if only and lg.unit is not None and lg.unit != unit:
            continue
        s = 0
        if isinstance(lg.to, Thing) and lg.to.name == x:
            s += 1
        if isinstance(lg.frm, Thing) and lg.frm.name == x:
            s -= 1
        if s == 0:
            continue
        a = lg.amount
        if isinstance(a, TallyError):
            raise a
        if isinstance(a, Hole):
            holes.append(a)
            continue
        if lg.unit is not None and unit is not None and lg.unit != unit:
            # L §5.14 Legs: "A and B are places of the leg's unit"; a leg between places of different units doesn't
            # load (L §5.10 LOAD-UNIT): an internal guard (writer 5's T-100, stated)
            raise refuse('LOAD-UNIT', 'a leg in %s touches %s, a place in %s' % (lg.unit, x, unit))
        tot += s * a.q
    if holes:
        return merge(*holes)
    return tot


def m_touches(self, legs, x, unit=None):
    return any(((isinstance(lg.to, Thing) and lg.to.name == x) or (isinstance(lg.frm, Thing) and lg.frm.name == x))
               and (unit is None or lg.unit is None or lg.unit == unit) for lg in legs)


def m_residual_units(self, place, span, ctx):
    """The units of a place of unit any's ledgers: those of the legs and readings touching it in the span, in the
    order of the units' ids (E §6.3: "A place of unit any gives one ledger per unit, in the order of the units'
    ids")."""
    m = self.m
    octx = ctx.but(cut=m.on)
    us = set()
    for mi in self.plan_moves():
        for i in self.move_instances(mi, ctx):
            for lg in getattr(i, 'legs', None) or ():
                if lg.unit is not None and self.touches([lg], place):
                    us.add(lg.unit)
    for o in self.observations(octx):
        for lg in o.legs:
            if lg.unit is not None and self.touches([lg], place):
                us.add(lg.unit)
    for iid, p, u, r in self.readings(octx):
        if p == place and u not in (None, '*'):
            us.add(u)
    return sorted(us)


def m_residuals(self, place, span, by, ctx, unit=None):
    """The ledger's groups: [(group name, model total, observed total, difference, [RLine])], sorted (E §6.3).
    For a place of unit any, `unit` names the one ledger (residual_units)."""
    m = self.m
    only = unit is not None
    if unit is None:
        unit = m.place_unit(m.things[place])
    if unit == 'any':
        raise refuse('RUN-DOMAIN', 'residuals of %s, a place of unit any, read as one ledger' % place)
    octx = ctx.but(cut=m.on)
    if only:
        ft, tc = self.flow_into, self.touches
        self_flow = lambda legs, x, u: ft(legs, x, u, only=True)
        self_touch = lambda legs, x: tc(legs, x, unit)
    else:
        self_flow = self.flow_into
        self_touch = self.touches
    K = ctx.cut

    def covered(d):
        return self.covered_on(place, d, octx)

    def in_span(d):
        return isinstance(d, datetime.date) and span.contains(d)

    # the model: plan instances (booked or modelled at their booking cut, settlements included) and decisions as
    # the walk at K ran them
    model = []
    undated = []
    # the instances an observation (as known on `on`) fulfils: compared all the same on a day no statement covers,
    # their matched line on the instance's day (E §6.3)
    ful_ids = set()
    # the last day of the observations that fulfil each instance, in any span: a hole-day instance's day once an
    # observation fixes it (E §6.3 Grouping, L §5.19, in every ledger; tally 2.2.1 P37, N7; tally 2.3)
    ful_last = {}
    for o in self.observations(octx):
        f = o.fulfils
        if isinstance(f, tuple) and f[0] == 'inst' and self_touch(o.legs, place):
            tid = '%s[%s]' % (f[1], ', '.join(key_text(k) for k in f[2])) if f[2] else f[1]
            if isinstance(o.day, datetime.date):
                ful_last[tid] = max(ful_last.get(tid, o.day), o.day)
            if in_span(o.day):
                ful_ids.add(tid)
    def may_touch(i):
        # tally 2.1.7 R1, R2 (SR-1): a refused instance refuses the ledger exactly where a hole one would be a line of
        # it: a place its legs give, or any place where its legs' places couldn't be read (L §8.4)
        if i.legs:
            return self_touch(i.legs, place) or any(not isinstance(p, Thing) for lg in i.legs for p in (lg.frm, lg.to))
        return bool(i.move.legs)

    for mi in self.plan_moves():
        for i in self.move_instances(mi, ctx):
            st = self.status(i, ctx)
            if st in ('none', 'leftout'):
                continue
            touches = may_touch if st == 'refused' else (lambda j: self_touch(j.legs, place))
            d = i.day
            # a refused instance whose day is refused may fall on any day, as a hole day's (tally 2.1.8 R11, #191 Q1)
            # a hole day an observation fulfils takes its last fulfilling observation's day, in every ledger, once
            # (tally 2.2.1 P43 S6, P45 (f); tally 2.3); only one nothing fulfils is a group of its own
            if (isinstance(d, Hole) and i.id not in ful_last) or (st == 'refused' and day_lo(d) is None):
                # E §6.3 Grouping (round 12): "An instance that nothing fulfils whose day is a hole prints an
                # expected-but-missing line in a group of its own named ?, first among the groups, with its day's
                # questions; it counts in no month's totals" (writer 5's T-102)
                if not touches(i):
                    continue
                if st == 'refused':
                    raise i.err
                undated.append(i)
                continue
            dd = day_lo(d)
            if isinstance(d, Hole):
                # the month of the observation holds the line (P37, N7)
                dd = ful_last[i.id]
            if not in_span(dd) or (not covered(dd) and i.id not in ful_ids):
                continue
            if not touches(i):
                continue
            if st == 'refused':
                raise i.err
            model.append(i)
    w = self.main_walk(ctx)
    for iid in sorted(w.decs):
        run = w.decs[iid]
        dd = day_lo(run.day)
        if dd is None or not in_span(dd) or not covered(dd):
            continue
        if run.err is not None:
            if may_touch(run):
                raise run.err
            continue
        if not self_touch(run.legs, place):
            continue
        model.append(run)
    model.sort(key=lambda i: (ful_last[i.id] if isinstance(i.day, Hole) else day_lo(i.day), i.order or ()))
    # the observations as known on `on`
    obs = [o for o in self.observations(octx) if self_touch(o.legs, place) and in_span(o.day)]
    # rows stated `same` are one movement (L §5.19, §6.1 SF-25): where their legs at x agree it is booked once, by the
    # latest of them, and listed once; where they disagree each shows (E §6.1)
    also = {}
    hole_at = {}     # a row whose same group's rows both book x and disagree there: its observed at x, the HOLE-SAME
    # the pairs as known on `on`, the observations' own cut (E §6.3; tally 2.1.8 R12: their same lines with them)
    if same_pairs(m, octx):
        keep = []
        pairs = same_pairs(m, octx)
        ids_in = set(o.id for o in obs)
        kept_groups = set()
        for o in obs:
            skip = False
            for lg in o.legs:
                for si, p in enumerate((lg.frm, lg.to)):
                    if isinstance(p, Thing) and p.name == place:
                        gd = lg.day if isinstance(lg.day, datetime.date) else o.day
                        if o.id in pairs and isinstance(gd, datetime.date) and not w.obs_end_on(o, p, gd, octx):
                            # E §6.3: "a pair of rows stated same is one movement: the ledger lists it once, on one
                            # line, and names both rows"; at x it is what x's statement books (L §5.19: "each end is
                            # booked once, from the statement that covers it"), not the other row's end
                            skip = True
                            continue
                        try:
                            cs = w.consol(o, lg)
                        except TallyError:
                            cs = (None, None)
                        if cs[si] == 'skip':
                            skip = True
                        elif isinstance(cs[si], Hole) and o.id in pairs:
                            # rows that both book x and disagree are one hole movement there (L §6.1, HOLE-SAME), so
                            # the ledger lists it once, on one line (E §6.3), its observed that hole
                            g = tuple(pairs[o.id][0])
                            if g in kept_groups:
                                skip = True
                            else:
                                # a row with two legs at x: the hole carries both legs' questions (L §8.1;
                                # tally 2.2.1 P36, N6), not the last leg's
                                hole_at[o.id] = cs[si] if o.id not in hole_at else merge(hole_at[o.id], cs[si])
            if not skip and o.id in hole_at:
                kept_groups.add(tuple(pairs[o.id][0]))
            if not skip:
                keep.append(o)
                pr = pairs.get(o.id)
                if pr is not None:
                    # "names both rows as its evidence" (E §6.3)
                    also[o.id] = sorted(x for x in pr[0] if x in ids_in)
        obs = keep

    def obs_flow(o):
        h = hole_at.get(o.id)
        return h if h is not None else self_flow(o.legs, place, unit)
    by_target = {}
    by_adj = {}
    lone = []
    # a pair stated same is one movement, "its fulfils ... stated on either row" (L §5.19): a row's link is its
    # partner's where it states none
    groups = same_groups(m, octx)
    all_obs = None
    for o in obs:
        f = o.fulfils
        if f is None and o.id in groups:
            if all_obs is None:
                all_obs = dict((x.id, x) for x in self.observations(octx))
            f = next((all_obs[x].fulfils for x in groups[o.id] if x in all_obs and all_obs[x].fulfils is not None),
                     None)
        if f is None:
            lone.append(o)
            continue
        if isinstance(f, tuple) and f[0] == 'inst':
            _, mid, kv = f
            tid = '%s[%s]' % (mid, ', '.join(key_text(k) for k in kv)) if kv else mid
            by_target.setdefault(tid, []).append(o)
            continue
        if isinstance(f, tuple) and f[0] == 'adj':
            # E §6.3 matched (round 12): "an adjustment that observations settle, against its change δ on V.known"
            # (writer 5's T-103)
            _, mid, kv, leg, vid = f
            iid = '%s[%s]' % (mid, ', '.join(key_text(k) for k in kv)) if kv else mid
            by_adj.setdefault((iid, leg, vid), []).append(o)
            continue
        raise refuse('RUN-RECORD', 'observation %s fulfils %r' % (o.id, f))
    lines = []
    seen = set()

    def field_group(os, mid):
        if by is None:
            return 'move %s' % mid
        vals = set()
        for o in os:
            vals.add(obs_field(o, by))
        if len(vals) == 1 and None not in vals:
            # a value group prints `<f> <value>`, as a move's prints `move <id>`, so it is never taken for balance,
            # unexplained or a move (E §6.3; tally 2.1.8 S3, #178)
            return '%s %s' % (by, vals.pop())
        return 'move %s' % mid

    for i in model:
        dd = ful_last[i.id] if isinstance(i.day, Hole) else day_lo(i.day)
        mdl = self_flow(i.legs, place, unit)
        os = by_target.get(i.id, [])
        seen.update(o.id for o in os)
        if not os:
            if not isinstance(mdl, Hole) and mdl == 0:
                continue    # an instance of amount 0 with no observation is no movement
            diff = _sub(Fraction(0), mdl)
            grp = (month_period(dd.year, dd.month), field_group([], i.move.id))
            lines.append(RLine('expected but missing', dd, grp, mdl, None, diff,
                               'expected but missing %s on %s: model %s' % (i.id, dd, self.rtext(mdl, unit)),
                               i.order))
            continue
        if isinstance(i.day, Hole):
            # an observation that fulfils the movement fixes its day (L §5.19): a hole day's is the last one's (E §6.3
            # Grouping; tally 2.2.1, #221)
            dd = max(o.day for o in os)
        ob = None
        for o in os:
            ob = _add(ob, obs_flow(o))
        diff = _sub(ob, mdl)
        grp = (month_period(dd.year, dd.month), field_group(os, i.move.id))
        timing = ', '.join(sorted(set('%+d days' % (o.day - dd).days for o in os)))
        if self.kind_certainty(i.kind) == 'uncertain' and not isinstance(ob, Hole) and not isinstance(mdl, Hole) \
                and abs(ob) < abs(mdl):
            rem = mdl - ob
            lines.append(RLine('partly fulfilled', dd, grp, mdl, ob, diff,
                               'partly fulfilled %s on %s: model %s, observed %s, difference %s, remainder %s: when '
                               'does the rest arrive, or is it written off?' % (
                                   i.id, dd, self.rtext(mdl, unit), self.rtext(ob, unit), self.rtext(diff, unit),
                                   self.rtext(rem, unit)), i.order))
            continue
        lines.append(RLine('matched', dd, grp, mdl, ob, diff,
                           'matched %s on %s: model %s, observed %s, difference %s, timing %s' % (
                               i.id, dd, self.rtext(mdl, unit), self.rtext(ob, unit), self.rtext(diff, unit), timing),
                           i.order))
    if by_adj:
        adjs = {}
        for a in self.adjustments(ctx):
            for vid in a.vids:
                adjs[(a.inst.id, a.leg.name, vid)] = a
        for nm in sorted(by_adj):
            os = by_adj[nm]
            a = adjs.get(nm)
            if a is None:
                # E §6.3 unexplained: "an observation ... whose instance or adjustment the model at K doesn't hold (one
                # that only a version known after K makes: the forecast didn't expect it)": its line is below
                continue
            seen.update(o.id for o in os)
            sgn = (1 if isinstance(a.leg.to, Thing) and a.leg.to.name == place else 0) - \
                (1 if isinstance(a.leg.frm, Thing) and a.leg.frm.name == place else 0)
            dl = a.delta
            if isinstance(dl, TallyError):
                raise dl
            mdl = dl if isinstance(dl, Hole) else sgn * dl.q
            ob = None
            for o in os:
                ob = _add(ob, obs_flow(o))
            diff = _sub(ob, mdl)
            dd = a.day
            grp = (month_period(dd.year, dd.month), field_group(os, a.inst.move.id))
            timing = ', '.join(sorted(set('%+d days' % (o.day - dd).days for o in os)))
            lines.append(RLine('matched', dd, grp, mdl, ob, diff,
                               'matched %s on %s: model %s, observed %s, difference %s, timing %s' % (
                                   a.id, dd, self.rtext(mdl, unit), self.rtext(ob, unit), self.rtext(diff, unit),
                                   timing), a.order))
    for i in undated:
        os = by_target.get(i.id)
        if os:
            # E §6.3 Grouping: "for an instance whose day is a hole, the day of the
            # last observation that fulfils it, observed(m[k]).day, as an observation answers a hole movement's day"
            seen.update(o.id for o in os)
            dd = max(o.day for o in os)
            mdl = self_flow(i.legs, place, unit)
            ob = None
            for o in os:
                ob = _add(ob, obs_flow(o))
            diff = _sub(ob, mdl)
            grp = (month_period(dd.year, dd.month), field_group(os, i.move.id))
            timing = ', '.join(sorted(set('%+d days' % (o.day - dd).days for o in os)))
            lines.append(RLine('matched', dd, grp, mdl, ob, diff,
                               'matched %s on %s: model %s, observed %s, difference %s, timing %s' % (
                                   i.id, dd, self.rtext(mdl, unit), self.rtext(ob, unit), self.rtext(diff, unit),
                                   timing), i.order))
            continue
        mdl = self_flow(i.legs, place, unit)
        if not isinstance(mdl, Hole) and mdl == 0:
            continue
        d = i.day
        dtxt = fmt_hole(d)
        lines.append(RLine('expected but missing', None, (None, '?'), mdl, None, _sub(Fraction(0), mdl),
                           'expected but missing %s on %s: model %s' % (i.id, dtxt, self.rtext(mdl, unit)),
                           i.order))
        # the day it prints is an unknown day the ledger carries (tally 2.1.8 R13, L §4.4)
        lines[-1].carry_day = d
    for o in obs:
        if o.id in seen:
            continue
        f = o.fulfils
        if f is not None and self.resid_held(f, ctx):
            # it fulfils an instance the model at K holds, outside this ledger (dated outside the span): its line is
            # that instance's
            continue
        # an observation that fulfils nothing, or whose instance or adjustment the model at K doesn't hold (E §6.3)
        ob = obs_flow(o)
        if by is not None:
            v = obs_field(o, by)
            g = '%s %s' % (by, v) if v is not None else 'unexplained'
        else:
            g = 'unexplained'
        ids = ', '.join(also.get(o.id) or [o.id])
        lines.append(RLine('unexplained', o.day, (month_period(o.day.year, o.day.month), g), None, ob, ob,
                           'unexplained %s on %s: observed %s' % (ids, o.day, self.rtext(ob, unit)), o.order))
    # balance lines: every stated reading of x in the span, at its instant (E §6.3)
    for iid, p, u, r in self.readings(octx):
        if iid != 'stated' or p != place:
            continue
        d, pos = r.instant
        if not in_span(d):
            continue
        key = (d.toordinal(), 0 if pos == 'start' else 4, 1, place + ' ' + ('%s %s' % (d, pos)), -1)
        val = r.val
        if val is NONE:
            # a reading `replace … = none` removes: no reading, so no line (L §5.19; 236.2)
            continue
        if isinstance(val, tuple) and val[0] == 'hole':
            stated = Hole([(val[1], 'stated[%s]' % place)])
        else:
            stated = val.q
        st = w.pstate(place, u, key, raw=True)
        bv = w.st_value(st, place, u)
        if isinstance(bv, TallyError):
            raise bv
        booked = bv if isinstance(bv, Hole) else bv.q
        gap = _sub(stated, booked)
        role = r.version.gap_roles.get(place)
        mark = 'anchored' if role is not None and role[0] == 'anchor' else 'padded'
        it = '%s %s' % (d, pos) if pos is not None else instant_text(r)
        txt = 'balance %s %s (%s): booked %s, stated %s, gap %s' % (
            place, it, mark, self.rtext(booked, unit), self.rtext(stated, unit), self.rtext(gap, unit))
        if r.__dict__.get('parts'):
            # E §6.3 (tally 2.1, 2.1-F3): "a reading that states parts also shows each stated part's gap, of <q>, and
            # the unstated rest's, rest, each stated - booked, in the order of the reading's rows"; the layout (each
            # after the total's, with its three figures) is this interpreter's
            gl = w.pgap_lines(r, key)
            for lab, b_, s_ in gl or ():
                # a booked part that is a hole (the hole balance's, 2.1-F10) has a hole gap
                txt += '; %s: booked %s, stated %s, gap %s' % (lab, self.rtext(b_, unit), self.rtext(s_, unit),
                                                               self.rtext(b_ if isinstance(b_, Hole) else s_ - b_,
                                                                          unit))
        lines.append(RLine('balance', d, (month_period(d.year, d.month), 'balance'), booked, stated, gap, txt,
                           (d, 9)))
    # E §6.3 (tally 2.1.7 S1, #93): a period that E cuts holds its days through E; the days after E are one line, a
    # hole `after the run's end` (HOLE-POSTRUN, L §8.1), since the walk holds none of them
    last = span.end - datetime.timedelta(days=1) if span.end is not None else None
    if last is not None and last > m.E:
        lines = [ln for ln in lines if not (isinstance(ln.day, datetime.date) and ln.day > m.E)]
        a = max(m.E + datetime.timedelta(days=1), span.start)
        h = Hole([('after the run\'s end', place)])
        lines.append(RLine('after', a, (None, 'after the run\'s end'), h, h, h,
                           'days %s .. %s: %s' % (a, last, self.rtext(h, unit)), (a, 0)))
    # groups, totals, order (E §6.3)
    groups = {}
    for ln in lines:
        groups.setdefault(ln.group, []).append(ln)
    out = []
    for g, ls in groups.items():
        mt = None
        ot = None
        for ln in ls:
            if ln.model is not None:
                mt = _add(mt, ln.model)
            if ln.observed is not None:
                ot = _add(ot, ln.observed)
        mt = Fraction(0) if mt is None else mt
        ot = Fraction(0) if ot is None else ot
        dt = _sub(ot, mt)
        ls.sort(key=lambda ln: (sk(ln.diff), ln.day or datetime.date.min, str(ln.group[1]), order_key(ln.order)))
        out.append((g, mt, ot, dt, ls))
    out.sort(key=lambda x: (0 if x[0][0] is None else 1, sk(x[3]),
                            x[0][0].start if x[0][0] is not None else datetime.date.min, str(x[0][1])))
    return out, unit


def m_resid_held(self, f, ctx):
    """Whether the model at the ask's cut holds what a `fulfils` names: an instance that exists there, or an
    adjustment some version made by then (E §6.3 unexplained)."""
    if isinstance(f, tuple) and f[0] == 'inst':
        mi = self.m.moves.get(f[1])
        if mi is None:
            return False
        try:
            t = self.find_inst(mi, list(f[2]), ctx)
        except TallyError:
            return True
        return t is not None and t.exists is not False
    if isinstance(f, tuple) and f[0] == 'adj':
        _, mid, kv, leg, vid = f
        iid = '%s[%s]' % (mid, ', '.join(key_text(k) for k in kv)) if kv else mid
        for a in self.adjustments(ctx):
            if a.inst.id == iid and a.leg.name == leg and vid in a.vids:
                return True
        return False
    return True


def obs_field(o, f):
    """The text of field f of an observation (`kind`, or one of its kind's attributes), or None."""
    if f == 'kind':
        return o.kind
    v = o.attrs.get(f) if o.attrs else None
    if isinstance(v, tuple) and v and v[0] in ('mvref', 'hole'):
        # the loader's markers: a ref prints as rows print, a hole as its question (writer 11)
        return v[1] if v[0] == 'mvref' else '?"%s"' % v[1]
    return fmt(v) if v is not None else None


def sk(v):
    """Holes first, among themselves by their first question; then by absolute value, biggest first (E §6.3)."""
    if isinstance(v, Hole):
        qs = sorted(q[0] for q in v.qs)
        return (0, qs[0] if qs else '', 0)
    return (1, '', -abs(v))


def m_rtext(self, v, unit):
    if isinstance(v, Hole):
        return fmt_hole(v)
    if v is None:
        return '-'
    return fmt(qty(v, unit))
