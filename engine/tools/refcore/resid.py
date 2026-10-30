"""The residual ledger (E §6.3): `ask q: residuals x span [by f]`.

The model at the ask's cut K (each plan instance as booked or modelled at its booking cut, the settlements dated in
the span, each decision as the walk at K ran it) against the observations as known on the model's `on` date, for
place x, on the days some statement covers x.

Installed as Engine methods (`m_<name>` -> `<name>`).
"""
import datetime
from fractions import Fraction

from .errors import TallyError, Refusal, Unspecified
from .values import Num, Hole, Thing, Text, Key, Iv, fmt, fmt_hole
from .dates import month_period
from .moves import Inst, qty, day_lo, merge, key_text, instant_text
from .walk import K_start, K_end, kday


def install(E):
    for name, fn in list(globals().items()):
        if name.startswith('m_'):
            setattr(E, name[2:], fn)


def refuse(code, msg):
    return Refusal(code, msg)


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
    for o in self.observations(octx):
        f = o.fulfils
        if isinstance(f, tuple) and f[0] == 'inst' and self_touch(o.legs, place) and in_span(o.day):
            ful_ids.add('%s[%s]' % (f[1], ', '.join(key_text(k) for k in f[2])) if f[2] else f[1])
    for mi in self.plan_moves():
        for i in self.move_instances(mi, ctx):
            st = self.status(i, ctx)
            if st in ('none', 'leftout'):
                continue
            if st == 'refused':
                raise i.err
            d = i.day
            if isinstance(d, Hole) or (isinstance(d, Iv) and isinstance(d.lo, datetime.date) and
                                       (d.lo.year, d.lo.month) != (d.hi.year, d.hi.month)):
                # E §6.3 Grouping (round 12): "An instance that nothing fulfils whose day is a hole, or a window not
                # inside one month, prints an expected-but-missing line in a group of its own named ?, first among
                # the groups, with its day's questions; it counts in no month's totals" (writer 5's T-102)
                if isinstance(d, Iv) and ((span.end is not None and d.lo >= span.end) or
                                          (span.start is not None and d.hi < span.start)):
                    continue
                if not self_touch(i.legs, place):
                    continue
                undated.append(i)
                continue
            dd = day_lo(d)
            if not in_span(dd) or (not covered(dd) and i.id not in ful_ids):
                continue
            if not self_touch(i.legs, place):
                continue
            model.append(i)
    w = self.main_walk(ctx)
    for iid in sorted(w.decs):
        run = w.decs[iid]
        dd = day_lo(run.day)
        if dd is None or not in_span(dd) or not covered(dd):
            continue
        if run.err is not None:
            raise run.err
        if not self_touch(run.legs, place):
            continue
        model.append(run)
    model.sort(key=lambda i: (day_lo(i.day), i.order or ()))
    # the observations as known on `on`
    obs = [o for o in self.observations(octx) if self_touch(o.legs, place) and in_span(o.day)]
    # rows stated `same` are one movement (L §5.19, §6.1 SF-25): where their legs at x agree it is booked once, by the
    # latest of them, and listed once; where they disagree each shows (E §6.1)
    also = {}
    if getattr(w, 'same_pairs', None) is not None and w.same_pairs():
        keep = []
        pairs = w.same_pairs()
        ids_in = set(o.id for o in obs)
        for o in obs:
            skip = False
            for lg in o.legs:
                for si, p in enumerate((lg.frm, lg.to)):
                    if isinstance(p, Thing) and p.name == place:
                        try:
                            cs = w.consol(o, lg)
                        except TallyError:
                            cs = (None, None)
                        if cs[si] == 'skip':
                            skip = True
            if not skip:
                keep.append(o)
                pr = pairs.get(o.id)
                if pr is not None:
                    # "names both rows as its evidence" (E §6.3)
                    also[o.id] = sorted(x for x in pr[0] if x in ids_in)
        obs = keep
    by_target = {}
    by_adj = {}
    lone = []
    for o in obs:
        f = o.fulfils
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
            return vals.pop()
        return 'move %s' % mid

    for i in model:
        dd = day_lo(i.day)
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
        ob = None
        for o in os:
            ob = _add(ob, self_flow(o.legs, place, unit))
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
                ob = _add(ob, self_flow(o.legs, place, unit))
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
            # E §6.3 Grouping: "for an instance whose day is a hole or a window not inside one month, the day of the
            # last observation that fulfils it, observed(m[k]).day, as an observation answers a hole movement's day"
            seen.update(o.id for o in os)
            dd = max(o.day for o in os)
            mdl = self_flow(i.legs, place, unit)
            ob = None
            for o in os:
                ob = _add(ob, self_flow(o.legs, place, unit))
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
        dtxt = fmt_hole(d) if isinstance(d, Hole) else d.window_text()
        lines.append(RLine('expected but missing', None, (None, '?'), mdl, None, _sub(Fraction(0), mdl),
                           'expected but missing %s on %s: model %s' % (i.id, dtxt, self.rtext(mdl, unit)),
                           i.order))
    for o in obs:
        if o.id in seen:
            continue
        f = o.fulfils
        if f is not None and self.resid_held(f, ctx):
            # it fulfils an instance the model at K holds, outside this ledger (dated outside the span): its line is
            # that instance's
            continue
        # an observation that fulfils nothing, or whose instance or adjustment the model at K doesn't hold (E §6.3)
        ob = self_flow(o.legs, place, unit)
        if by is not None:
            g = obs_field(o, by) or 'unexplained'
        else:
            g = 'unexplained'
        ids = ', '.join(also.get(o.id) or [o.id])
        lines.append(RLine('unexplained', o.day, (month_period(o.day.year, o.day.month), g), None, ob, ob,
                           'unexplained %s on %s: observed %s' % (ids, o.day, self.rtext(ob, unit)), o.order))
    # balance lines: every stated reading of x in the span, at its instant (E §6.3)
    for iid, p, u, r in self.readings(octx):
        if iid != 'stated' or p != place:
            continue
        ins = r.instant
        if ins[0] == 'after':
            # E §6.3 Grouping (round 12): "a reading after m stands on m's day, right after m" (writer 5's T-104)
            key = w.after_keys.get(id(r))
            if key is None:
                continue    # m wasn't booked: the reading has no instant in the walk
            d = kday(key)
            if not in_span(d):
                continue
            pos = None
        else:
            d, pos = ins
            key = None
        if not in_span(d):
            continue
        val = r.val
        if isinstance(val, tuple) and val[0] == 'hole':
            stated = Hole([(val[1], 'stated[%s]' % place)])
        else:
            stated = val.q
        if key is None:
            key = (d.toordinal(), 0 if pos == 'start' else 4, 1, place + ' ' + ('%s %s' % (d, pos)), -1)
        else:
            key = key[:4] + (key[4] - 1,)
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
                txt += '; %s: booked %s, stated %s, gap %s' % (lab, self.rtext(b_, unit), self.rtext(s_, unit),
                                                               self.rtext(s_ - b_, unit))
        lines.append(RLine('balance', d, (month_period(d.year, d.month), 'balance'), booked, stated, gap, txt,
                           (d, 9)))
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
        ls.sort(key=lambda ln: (sk(ln.diff), ln.day or datetime.date.min, str(ln.group[1]), str(ln.order)))
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
