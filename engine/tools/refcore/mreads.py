"""Writer 3's reads (L §5.10, §5.14, §5.14.1, §5.19; E §4.2-§4.3, §6): places and movements as values, their
fields, bindings over event kinds and observed inputs, bal, flows, value, covered, gap, observed, unit.

Installed as Engine methods (`m_<name>` -> `<name>`).
"""
import datetime
from fractions import Fraction

from . import units as U
from .dates import Period, ONE_DAY, next_day, END
from .errors import TallyError, Refusal, LoadError, Unspecified, NotYet, load_now, at_run
from .evaluate import member_owns, member_moves, move_owner, carry_hole, imprecise_hole, env_txt
from .model import Span, _kv, key_text, Undecided
from .values import (Num, Money, Int, Hole, Thing, Mv, Iv, Rec, Text, Key, NONE, CURRENCIES, fmt,
                     fmt_hole, vkey, day_text)
from .moves import (item_in_world, Inst, ObsMv, AdjMv, PadMv, qty, unit_of_value, day_lo, day_hi, merge, order_key,
                    same_groups, same_lines, once_obs, same_obs_differ, day_set)
from .walk import K_end, K_start, K_before, kday, add_q, St, dec_move


def install(E):
    for name, fn in list(globals().items()):
        if name.startswith('m_'):
            setattr(E, name[2:], fn)


def refuse(code, msg):
    return at_run(code, msg)


def reads_var(node, var):
    from .node import walk
    for x in walk(node):
        if x.k == 'name' and getattr(x, '_r', None) == ('var', var):
            return True
        if x.k == 'name' and x.id == var and getattr(x, '_r', None) is None:
            return True
    return False


# -- places and movements as values ----------------------------------------------------------------------------------

def m_e_place(self, n, env, ctx):
    """A place by its id, or a keyed place at keys (L §5.10): its thing."""
    m = self.m
    pi = m.places[n._r[1]]
    if not pi.binds:
        return Thing(pi.things[()].name)
    keynodes = n.keys if n.k == 'index' else []
    keys, hole = self.key_values(pi, keynodes, n._convs, env, ctx)
    if hole is not None:
        return hole
    t = pi.things.get(tuple(_kv(k) for k in keys))
    if t is None:
        raise refuse('RUN-KEY', '%s[%s] is not a place' % (pi.id, ', '.join(key_text(k) for k in keys)))
    return Thing(t.name)


def m_e_moveref(self, n, env, ctx):
    """A movement instance by its move and keys (L §2.5)."""
    m = self.m
    mi = m.moves[n._r[1]]
    keynodes = n.keys if n.k == 'index' else []
    keys = []
    for kn, conv, b in zip(keynodes, n._convs, mi.binds):
        if conv[0] == 'lit':
            k = conv[1]
        else:
            v = self.ev(kn, env, ctx)
            if isinstance(v, Hole):
                return v
            k = self.to_key(v, conv, b.dom, kn) if not isinstance(v, Mv) else v
        keys.append(k)
    inst = self.find_inst(mi, keys, ctx)
    iid = '%s[%s]' % (mi.id, ', '.join(key_text(k) for k in keys)) if keys else mi.id
    if inst is None:
        return Mv(iid, None)
    return inst.mv()


def m_e_obsrow(self, n, env, ctx):
    qid, rid = n._r[1], n._r[2]
    for o in self.observations(ctx):
        if o.row.inf.id == qid and o.row.key == rid:
            self.cur.rows[o.row.sid] = o.row
            return o.mv()
    return NONE


def m_mv_record(self, v, ctx):
    """The record behind a movement value at the context's timeline: a decision's run in the walk. A movement named by
    its id only (a row's ref to a movement, a batch's label) is found by its id at the context's cut."""
    src = v.src
    if src is None:
        src = self.mv_by_id(v.id, ctx)
        if src is None:
            return None
    if isinstance(src, Inst) and src.decision and not getattr(src, 'ran', False) and isinstance(src.day, Hole):
        # a decision whose day is a hole never runs: never waited for, its fields read where the plan has them,
        # else its hole (hole_day_dec_body; E §4.2 step 3; tally 2.3, the lead's P10, H1)
        return src
    if isinstance(src, Inst) and src.decision and not getattr(src, 'ran', False):
        w = ctx.walk if ctx.walk is not None else None
        run = None
        while w is not None and run is None:
            run = w.decs.get(src.id)
            w = w.base
        if run is None:
            if ctx.walk is not None:
                raise refuse('RUN-ORDER', 'decision %s is read before it has run' % src.id)
            run = self.main_walk(ctx).decs.get(src.id)
        if run is None:
            return src
        return run
    return src


def m_hole_day_changed(self, src, name, rec, ctx):
    """A leg of a hole-day instance read as a field: where a later version changes it (C-4's measure: an adjustment
    of that leg among moves.hole_adjustments), the field it changes is the instance's day's hole, its amount, or its
    places where it moves them; a field no later version changes stays a value (E §5 step 1; tally 2.3, the lead's
    P12, N1). Read while those adjustments are measured (a move that reads the field), the field is the booking's."""
    if ('hadjs', self.wkey(ctx)) + self.upto_tag() in self.inst_active:
        return rec
    amt = False
    ends = set()
    for a in self.hole_inst_adjustments(src, ctx):
        if a.leg.name == name:
            amt = amt or isinstance(a.delta, Hole) or a.delta.q != 0
            if getattr(a, 'moved', False):
                # only the end the version moves (E l.247 "a leg's field it changes"; G1's review, p13 k2_from)
                fr, to = getattr(a, 'froms', None), getattr(a, 'tos', None)
                if fr is None or fr[0] != fr[1]:
                    ends.add('from')
                if to is None or to[0] != to[1]:
                    ends.add('to')
    if not (amt or ends):
        return rec
    h = src.day
    return Rec([(k, h if (k == 'amount' and amt) or k in ends else v) for k, v in rec.fields])


def m_mv_by_id(self, mid, ctx):
    """The record of the movement with id mid at the context's cut: a move's instance, or an observation; None when
    the cut sees none (writer 11: a row's ref to a movement, L §3.1 `key K` of an event kind)."""
    name = mid.split('[', 1)[0]
    mi = self.m.moves.get(name)
    if mi is not None:
        insts = self.decision_instances(mi, ctx) if mi.decision else self.move_instances(mi, ctx)
        for inst in insts:
            if inst.id == mid:
                return inst
        return None
    for o in self.observations(ctx):
        if o.id == mid:
            return o
    return None


def m_leg_refused(self, iid, leg, ctx, day=None):
    """The refusal of a leg the walk refused (RUN-PLACE, RUN-LIFESPAN), or None: "every read of the refused leg
    itself" is refused (E §4.2 step 5, round 10). Inside a walk, what it has refused so far; at an ask, the main
    walk's (a walk being built, as in the plan pass, has refused nothing yet that a plan may read, E §4.1). Read
    inside a cell's evaluation, the walk is built only as far as it needs: a walk to the leg's day when the whole walk
    needs a cell still being evaluated (m_guarded_walk; E §3.2 "Evaluation is demand-driven")."""
    w = ctx.walk if ctx.walk is not None else self.ask_walk(ctx, day)
    while w is not None:
        err = w.bad_legs.get((iid, leg)) or getattr(w, 'hole_legs', {}).get((iid, leg)) or \
            getattr(w, 'hole_flows', {}).get((iid, leg))
        if err is not None:
            c = getattr(w, 'bad_reads', {}).get((iid, leg)) if isinstance(err, TallyError) else None
            if c is not None and self.cur is not None:
                # a read of the refused leg rests on what its refusal read (L §8.2 l.2248; tally 2.2.1 P35, B1)
                self.cur.merge(c)
            return err
        w = w.base
    return None


def m_ask_walk(self, ctx, day=None):
    """The walk an ask's read outside it sees, or None: the main walk of the ask's own timeline, never one a read at
    an instance's booking cut would start (that read is part of the plan pass, E §4.1)."""
    if not self.m.places and not self.m.moves:
        return None
    wk = ('walk',) + self.wkey(self.tctx(ctx))
    w = self.walks.get(wk)
    busy = bool(getattr(self, "inst_active", None)) or getattr(self, '_prefix_running', 0) or \
        any(getattr(x, "running", False) for x in self.walks.values())
    if not busy and ctx.cut == getattr(self, 'ask_K', None) and \
            (w is None or (w.done < w.end and getattr(w, 'failed', None) is None)):
        # only the ask's own timeline (a read at an instance's booking cut is part of the plan pass)
        if self.active and isinstance(day, datetime.date):
            w = self.guarded_walk(ctx, day)
        else:
            try:
                w = self.main_walk(ctx)
            except TallyError:
                return None
    return w


def m_outer_hit(self):
    """A walk attempt needed a cell still being evaluated outside it: start the journal of what it caches."""
    self._outer_hit = True
    for d in self.journaled():
        if getattr(d, 'journal', 1) is None:
            d.journal = []


def m_journaled(self):
    """What a walk attempt's take-back undoes (m_guarded_walk): the memo, the run cache, the walks and the invariant 5
    record, all journalled (tally 2.1.7 R1, G10: an amount booked while the cell it reads was still being evaluated
    outside the attempt is taken back with it, or a later ask meets it as a booking that changed)."""
    from .evaluate import JournalDict
    for name in ('memo', 'run_cache', 'walks'):
        d = getattr(self, name)
        if not isinstance(d, JournalDict):
            setattr(self, name, JournalDict(d))
    reg = self.m.__dict__.get('_inv5')
    if not isinstance(reg, JournalDict):
        reg = self.m.__dict__['_inv5'] = JournalDict(reg or {})
    return (self.memo, self.run_cache, self.walks, reg)


def m_guarded_walk(self, ctx, day):
    """The walk a leg read on `day` inside a cell's evaluation needs (E §4.2 step 5). The whole walk, unless building it
    needs a cell still being evaluated outside it (a plan booked after the leg reads the cell that reads the leg, as a
    send each quarter that pays out what the quarter's rents sum to): that is no cycle, since a leg's refusal rests on
    the walk up to its day only (E §4.2: a refused leg isn't booked, and the place's balance is refused from that
    point), and evaluation is demand-driven (E §3.2; L §5.11 RUN-CYCLE is a chain back to a cell still being
    evaluated). The attempt is then taken back, what it cached after the need dropped (MC-D-F1: else the answer depends
    on the order of the asks), and the walk is built up to the leg's day (m_prefix_walk)."""
    wk = ('walk',) + self.wkey(self.tctx(ctx))
    outer = frozenset(self.active)
    blocked = self.__dict__.setdefault('_walk_blocked', {}).get(wk)
    if blocked and blocked & outer:
        return self.prefix_walk(ctx, day)
    stores = self.journaled()
    prev = (getattr(self, '_walk_outer', frozenset()), getattr(self, '_outer_hit', False))
    self._walk_outer = prev[0] | outer
    self._outer_hit = False
    try:
        try:
            w = self.main_walk(ctx)
        except TallyError:
            w = None
        hit = self._outer_hit
    finally:
        self._walk_outer = prev[0]
        self._outer_hit = prev[1]
        journals = []
        for d in stores:
            journals.append(d.journal)
            d.journal = None
    if not hit:
        return w
    for d, j in zip(stores, journals):
        for k in j or ():
            d.pop(k, None)
    self.walks.pop(wk, None)
    self._walk_blocked[wk] = outer
    return self.prefix_walk(ctx, day)


def m_prefix_walk(self, ctx, day):
    """The walk of the ask's timeline from S up to `day`, booking only the plans dated on or before it (E §4.1, §4.2):
    what a leg read on that day needs. Kept per day; a walk to a later day serves an earlier one."""
    from .walk import Walk
    from .evaluate import Coll
    tctx = self.tctx(ctx)
    wk = ('walk',) + self.wkey(tctx)
    pws = self.__dict__.setdefault('prefix_walks', {})
    best = None
    for (k, upto), pw in pws.items():
        if k == wk and upto >= day and (best is None or upto < best[0]):
            best = (upto, pw)
    if best is not None:
        return best[1]
    m = self.m
    end = min(day, m.E)
    w = Walk(self, tctx, m.S, end)
    prev = getattr(self, '_body_upto', None)
    self._body_upto = day
    self._prefix_running = getattr(self, '_prefix_running', 0) + 1
    c = Coll()
    self.stack.append(c)
    w.running = True
    try:
        w.prepare()
        w.run_to(end)
    except TallyError:
        w = None
    finally:
        if w is not None:
            w.running = False
        self.stack.pop()
        self._body_upto = prev
        self._prefix_running -= 1
    pws[(wk, day)] = w
    return w


def _ntxt(n):
    from .evaluate import _ntxt as f
    return f(n)


def m_mv_field(self, n, env, ctx):
    """`s.cash`, `s.day`, an attribute, `a.adjusts`, `a.change` (L §5.14, §5.14.3)."""
    v = self.ev(n.obj, env, ctx)
    if isinstance(v, Hole):
        return v
    if v is NONE:
        # the read that gave none, as an observed row's id (L §8.4; tally 2.1.8 #169-l)
        raise refuse('RUN-NONE', '%s is none here: a field read of it' % env_txt(n.obj, env))
    if not isinstance(v, Mv):
        raise refuse('LOAD-FIELD', '%s is not a field of %s' % (n.name, fmt(v)))
    src = self.mv_record(v, ctx)
    if src is None:
        # a field read of an instance that doesn't exist is refused (L §5.14 existence, §8.4; tally 2.3.1 R6, C-2)
        raise refuse('RUN-NONE', '%s doesn\'t exist: a field read of it' % v.id)
    what = n._r[1]
    sh = None
    name = n.name
    identity = isinstance(src, AdjMv) and (what == 'day' or (what == 'attr' and name == 'adjusts'))
    if identity:
        pass    # fixed by the adjustment's identity: V.known, the instance and its leg (E §5 step 4)
    elif what == 'day' and getattr(src, 'day_coll', None) is not None:
        self.cur.merge(src.day_coll)
    elif src.coll is not None:
        self.cur.merge(src.coll)
    for r in getattr(src, 'reg', None) or ():
        # a binding that counts a pad lists it (L §8.2's `pad:` row; tally 2.2.1, #237)
        self.reg(r)
    if getattr(src, 'err', None) is not None and not identity:
        # a refused adjustment refuses its change and its remainder, never its identity, so a binding's `where` that
        # reads `a.adjusts` or `a.day` takes it or leaves it out (E §5 step 5; tally 2.1.2, issue #34, T's
        # STOP-REPORT on g11/adjust: the identity read refused, and with it every move bound over adjustments)
        raise src.err
    if what == 'leg' and isinstance(src, Inst) and ctx.walk is None and ctx.wp is None and \
            getattr(src, 'status', None) in ('past', 'plan'):
        # a leg the walk books is read with the axes the walk read to book it, so the read fans out over them (L §5.5
        # Fan-out; tally 2.1.8 R13, #169-k); a leg the walk refused is refused below (leg_refused)
        # building the walk to learn its axes reads other movements' days and amounts, none of them the value's
        # (E §10; tally 2.1.8 P50: b2_reads c3 listed pay's v:dom under an explain of xs)
        saved_tr = (self.trace_stack, self.tracing)
        self.trace_stack, self.tracing = [], False
        try:
            w = self.ask_walk(ctx, src.day if isinstance(src.day, datetime.date) else None)
        finally:
            self.trace_stack, self.tracing = saved_tr
        for lg in (src.legs if w is not None else ()):
            if lg.name != name:
                continue
            for p in (lg.frm, lg.to):
                if not isinstance(p, Thing) or p.name not in self.m.things:
                    continue
                u = lg.unit or self.m.place_unit(self.m.things[p.name]) or '*'
                log = w.logs.get((p.name, u))
                if log is None:
                    continue
                at = [j for j, e in enumerate(log.ents) if getattr(e.mv, 'id', None) == src.id]
                if at:
                    # what the walk read to book it there: the axes only, not the place's register
                    c = w.cum(log, at[-1])
                    if c is not None:
                        self.cur.axes |= c.axes
    if what in ('day', 'leg') and isinstance(src, (ObsMv, Inst)):
        # L §5.19 One movement on two statements: where rows stated same disagree on an amount or a day, "a read of
        # the movement away from its places (its leg's amount ... through a binding) is the disagreement's hole,
        # HOLE-SAME, naming both rows"
        if what == 'day':
            h = self.same_rows_hole(src, None, ctx, 'day')
            if h is not None:
                return h
        else:
            # per fact (tally 2.2.1 P36, Q3): the leg's amount and its day, each its own hole where the rows differ
            sh = (self.same_rows_hole(src, name, ctx, 'amount'), self.same_rows_hole(src, name, ctx, 'day'))
    if what == 'day':
        if isinstance(src, Inst) and (src.exists is False or getattr(src, 'status', None) == 'none'):
            # `.day` included (L §5.14 existence; tally 2.3.1 R6, gate review 8); an instance a later version made
            # exist exists as known now, its day its own, as its legs read (made_to_exist_leg; E §5 step 1; one
            # instance, one answer: g2/i3_later d2 beside x2 false)
            if any(self.made_to_exist_leg(src, a.leg.name, ctx) is not None for a in self.adjust_move(src.move, ctx)
                   if a.inst.id == src.id):
                return src.day
            raise refuse('RUN-NONE', '%s doesn\'t exist: a field read of it' % v.id)
        return src.day
    if what == 'fulfils':
        f = src.fulfils
        if f is None:
            return NONE
        if isinstance(f, tuple) and f[0] == 'adj':
            # adjustments[m[keys].leg, V] (L §5.14.3): the adjustment, at this cut
            _, mid, kv, leg, vid = f
            iid = '%s[%s]' % (mid, ', '.join(key_text(k) for k in kv)) if kv else mid
            for a in self.adjustments(ctx, settled=False):
                if a.inst.id == iid and a.leg.name == leg and vid in a.vids:
                    return a.mv()
            return NONE
        if isinstance(f, tuple):
            mi = self.m.moves[f[1]]
            if not mi.decision:
                # the link reads that instance's existence alone, never the move's list or its amounts (L §5.14
                # existence, §5.19; tally 2.3.1 R6: P22, P30)
                env = self.link_tuple(mi, list(f[2]), ctx, traced=True)
                if env is None:
                    return NONE
                if getattr(env, 'err', None) is not None and not isinstance(env, dict):
                    # a link to a refused instance is refused with it (rulings 16 Q1)
                    raise env.err
                return Mv('%s[%s]' % (f[1], ', '.join(key_text(k) for k in f[2])) if f[2] else f[1])
            env = self.link_tuple(mi, list(f[2]), ctx, traced=True)
            if env is None:
                return NONE
            if getattr(env, 'err', None) is not None and not isinstance(env, dict):
                raise env.err
            iid = '%s[%s]' % (f[1], ', '.join(key_text(k) for k in f[2])) if f[2] else f[1]
            if iid in self.run_cache.get(('deflinks', self.wkey(self.tctx(ctx))), ()):
                # a decision's link its running `when` leaves to its walk point is read there (C-9): before that point
                # another decision can't read it (RUN-ORDER, rulings 16 Q5); after it, the run decides
                w = ctx.walk
                inner = w is not None and ctx.wp is not None
                if not inner:
                    w = self.main_walk(ctx)
                run, ww = None, w
                while ww is not None and run is None:
                    run = ww.decs.get(iid)
                    ww = ww.base
                ob, ww = None, w
                while ww is not None and ob is None:
                    ob = getattr(ww, 'order_bad', {}).get(iid)
                    ww = ww.base
                if ob is None and run is not None and getattr(run, 'err', None) is not None:
                    ob = run.err
                if ob is not None:
                    # the instance the walk's order refused: its link is refused with it (rulings 15, 16 Q1)
                    raise ob
                if run is None and inner:
                    a, b = sorted([getattr(w, 'cur_dec', None) or '?', mi.id])
                    raise refuse('RUN-ORDER', 'decisions %s and %s read each other on %s' % (a, b, kday(ctx.wp)))
                if run is not None and getattr(run, 'exist_coll', None) is not None:
                    self.cur.merge(run.exist_coll)
                bad = self.run_cache.get(('badlinks', self.wkey(ctx)), {}).get(iid)
                if bad is not None:
                    raise bad
                if run is not None and (run.exists is False or getattr(run, 'status', None) == 'none'):
                    return NONE
            t = self.find_inst(mi, list(f[2]), ctx)
            return t.mv() if t is not None else NONE
        return f
    if what == 'leg':
        for lg in src.legs:
            if lg.name == name:
                if isinstance(lg.amount, TallyError):
                    raise lg.amount
                # whether the walk refused the leg is no read of its value: what building the walk reads (the plans'
                # amounts and days) isn't listed under an explain of it (E §10; tally 2.1.8 P50, group 4's fix
                # review: s6_reads_agree s4 listed d_p and amt)
                saved_tr = (self.trace_stack, self.tracing)
                self.trace_stack, self.tracing = [], False
                try:
                    err = self.leg_refused(src.id, name, ctx, lg.day if isinstance(lg.day, datetime.date) else
                                           src.day)
                finally:
                    self.trace_stack, self.tracing = saved_tr
                if isinstance(err, Hole):
                    return err
                if err is not None:
                    raise err
                rec = lg.rec()
                if isinstance(src, Inst) and isinstance(src.day, Hole):
                    rec = self.hole_day_changed(src, name, rec, ctx)
                if sh is not None and (sh[0] is not None or sh[1] is not None):
                    rec = Rec([(k, (sh[0] if k == 'amount' and sh[0] is not None else
                                    sh[1] if k == 'day' and sh[1] is not None else v)) for k, v in rec.fields])
                if isinstance(src, ObsMv):
                    # an observation's leg read through a binding prints as its row (E §10 l.472; P24 F-2, #169;
                    # tally 2.1.8 P42, group 4's review: whatever other asks run)
                    self.trace_obs_row(src.row, rec)
                return rec
        if src.exists is False or getattr(src, 'status', None) == 'none':
            made = self.made_to_exist_leg(src, name, ctx)
            if made is not None:
                return made
            raise refuse('RUN-NONE', '%s doesn\'t exist: a field read of it' % v.id)
        h = _body_hole(src)
        if h is not None:
            return h
        raise refuse('LOAD-FIELD', '%s is not a field of %s' % (name, v.id))
    if what == 'attr':
        if name in src.attrs:
            if isinstance(src.attrs[name], TallyError):
                # a refused adjustment's change (tally 2.1.2, issue #34)
                raise src.attrs[name]
            return src.attrs[name]
        h = _body_hole(src)
        if h is not None:
            return h
        raise refuse('RUN-DOMAIN', '%s is not a field of %s' % (name, v.id))
    raise NotYet('the field %s of a movement' % name)


def m_same_others(self, o, ctx):
    """The other movements of o's group stated same, observed rows and planned instances, in the group's order."""
    g = same_groups(self.m, ctx).get(getattr(o, 'id', None))
    if not g:
        return []
    idx = dict((x.id, x) for x in self.observations(ctx))
    if any(y not in idx for y in g):
        # a planned side: its instance (L §6.1, §8.1 HOLE-SAME `<row or instance>`; #219)
        for i in self.plan_insts(ctx):
            if i.id in g:
                idx.setdefault(i.id, i)
    return [idx[y] for y in g if y != o.id and y in idx]


def m_same_in_span(self, x, sp, ctx):
    """Whether a movement of a group stated same is in span sp, read from every side (L §6.1 l.1797, "by every read of
    either"; tally 2.2.1 P43 S2): the day is read for membership only. None where every day every side can take lies
    inside the span, or every one outside (x's own day decides; the other facts answer), else the day's hole by S1.
    Either way the read holds every side's day reads."""
    others = self.same_others(x, ctx)
    if not others:
        return None
    # every side's day reads, its own among them (L §8.2; P48: h10 f2d lists pinned: d2)
    for y in [x] + others:
        self.same_fact_reads(y, None, 'day')
    days = [x.day] + [y.day for y in others]
    if any(isinstance(d, Hole) for d in days):
        return self.same_rows_hole(x, None, ctx, 'day')
    sets = [day_set(d) for d in days]
    if any(z is None for z in sets):
        return None
    ins = set(sp.contains(d) for z in sets for d in z)
    if len(ins) == 1:
        return None
    h = self.same_rows_hole(x, None, ctx, 'day')
    if h is None:
        # one window on every side, across the span's end: that window's hole (L §4.4)
        w = next(d for d in days if isinstance(d, Iv))
        h = imprecise_hole(w, 'the day of %s' % x.id)
    return h


def m_same_fact_reads(self, y, leg, fact):
    """What another side gave for the one fact read (tally 2.2.1 P43 S3, the register per fact): its day's reads, its
    leg's amount's, or, for a read of the whole movement, all it read."""
    if self.cur is None:
        return
    c = None
    if fact == 'day':
        c = getattr(y, 'day_coll', None)
    elif fact == 'amount':
        lg = next((g for g in getattr(y, 'legs', ()) if g.name == leg), None)
        c = getattr(lg, 'coll', None) if lg is not None else None
    if c is None and (fact is None or not isinstance(y, Inst)):
        c = getattr(y, 'coll', None)
    if c is not None:
        self.cur.merge(c)


def m_same_rows_hole(self, o, leg, ctx, fact=None):
    """The HOLE-SAME of an observed row whose same group's other observed rows disagree with it on a leg's amount or
    day (on leg alone, where given; on fact alone, 'amount' or 'day', where given), else None (L §5.19, L §8.1's form;
    moves.same_obs_differ). The read is of both sides: it holds the other rows' reads for that fact (L §8.2; tally
    2.2.1 P35, B2; P43 S3)."""
    others = self.same_others(o, ctx)
    for x in others:
        self.same_fact_reads(x, leg, fact)
    return same_obs_differ(o, others, leg, m=self.m, fact=fact) if others else None


def _body_hole(src):
    """A hole movement whose body wasn't evaluated (its law date or its existence a hole, L §5.14): a read of its
    legs or attributes is that hole (L §5.14 "A hole day or when makes a hole movement")."""
    for x in (getattr(src, 'exists', None), getattr(src, 'law', None), getattr(src, 'day', None)):
        if isinstance(x, Hole):
            return Hole(x.qs)
    return None


def m_made_to_exist_leg(self, src, name, ctx):
    """The leg `name` of a plan instance that didn't exist at its booking cut and that a later version made exist
    (E §5 step 1: it "didn't exist then, and counts as A0 = 0"; step 3: δ = Aᵥ). It is the adjusted leg of its
    adjustments (E §5 step 5 names that leg's places and way), booked at A0 = 0 (step 6: a later read sees A0 plus
    the settlements). So `a.adjusts.cash.from` names a place for a settlement of such an adjustment (L §5.14.3).
    None when no adjustment at this cut made it exist."""
    if not isinstance(src, Inst) or src.decision or src.move.id not in self.m.moves:
        return None
    hits = [a for a in self.adjust_move(src.move, ctx) if a.inst.id == src.id and a.leg.name == name and
            not isinstance(a.exists, Hole)]
    if not hits:
        return None
    # E §5 step 1: "a read of its legs (a settlement's a.adjusts.cash.from) gives the places of the version that made
    # it exist, with amount 0": the last step that made it exist (a later version that moves the leg makes an
    # adjustment of its own, L §5.14.3, and leaves these places as they are)
    made = [a for a in hits if getattr(a, 'made', False)]
    if not made:
        return None
    a0 = made[-1]
    if a0.coll is not None:
        self.cur.merge(a0.coll)
    lg = a0.leg
    zero = qty(0, lg.unit) if lg.unit is not None else Num(0)
    return Rec([('from', lg.frm), ('to', lg.to), ('amount', zero), ('day', lg.day)])


def m_mv_kind_in(self, v, kind, ctx):
    src = v.src
    if src is None:
        # writer 12 (F-1): a movement value with no record (a row's attribute naming a movement, moves._obs_attr; a
        # batch label) is the movement of that id as an ask finds it, at the reader's cut (mv_by_id; L §3.1 `key K`, a
        # movement a thing of its event kind, §5.7); one the cut doesn't see is in no kind
        src = self.mv_by_id(v.id, ctx)
    if src is None:
        return False
    return src.kind in self.m.kinds[kind].subtree(self.m)


# -- bindings over kinds, event kinds and observed inputs (L §5.6, §5.7, §5.19) -------------------------------------

def m_bind_each(self, b, env, ctx):
    """The members of a binding with their status: [(value, True | Hole)], or a Hole (an unknown domain)."""
    m = self.m
    dom = b._dom
    if dom[0] in ('event', 'obs'):
        return self.movement_members(b, env, ctx, defer=True)
    if dom[0] == 'kind' and ((b.on is not None and reads_var(b.on, b.var)) or
                             (b.over is not None and reads_var(b.over, b.var))):
        kind = dom[1]
        things = [t for t in m.things.values() if m.thing_seen(t, ctx.cut) and m.thing_in(t.name, kind) and
                  member_owns(m, b, t)]
        things.sort(key=lambda t: self.safe_order_key(t, kind, ctx.cut))
        out = []
        colls = {}
        for t in things:
            e2 = dict(env)
            e2[b.var] = Thing(t.name)
            c = self.push()
            try:
                ok = self.alive_test(b, e2, ctx, t, kind)
            except Undecided as w:
                # L §5.11: a thing whose life the text leaves open: an unknown member
                ok = w.hole()
            except TallyError as e:
                # refused at this member only (L §8.4): its tuple is refused, the others stand
                if isinstance(e, LoadError) or load_now(e) or not isinstance(e, Refusal):
                    raise
                ok = e
            finally:
                self.stack.pop()
            if ok is False:
                self.cur.merge(c)
                continue
            colls[t.name] = c
            out.append((Thing(t.name), ok if isinstance(ok, (Hole, TallyError)) else True))
        b._colls = colls
        return out
    b._unknown = None
    vals, hole, past = self.bind_values(b, env, ctx)
    if hole is not None:
        return hole
    unknown = getattr(b, '_unknown', None) or {}
    return [(v, unknown.get(getattr(v, 'name', None), True)) for v in vals]


def m_alive_test(self, b, env, ctx, t, kind):
    m = self.m
    if b.on is not None:
        d = self.ev(b.on, env, ctx)
        if isinstance(d, Hole):
            return d
        if d is NONE:
            return False
        d = self.date_of(d, 'the day of a binding')
        return m.alive_on(t, kind, d, ctx.cut, getattr(b, '_owner', None))
    sp = self.span_value(b.over, env, ctx)
    if isinstance(sp, Hole):
        return sp
    return m.alive_over(t, kind, sp, ctx.cut, getattr(b, '_owner', None))


def m_movements_of(self, dom, ctx, owner=None):
    """Every movement a binding over an event kind (or an observed input) ranges over at the timeline, in movement
    order: plans (left-out ones too, not the modelled past), observations, adjustments, pads, decisions. Inside a
    member (owner), that member's movements only, and the same lines it states (L §6.1: "A member is closed. A
    member's items mean what they mean alone"; the migrator's F-k)."""
    m = self.m
    restrict = getattr(self, '_adj_restrict', None)
    if restrict is not None and dom[0] == 'event' and 'adjustments' in m.kinds[dom[1]].subtree(m):
        # a settlement move's instances bound to the adjustments being settled (moves.settle_candidates)
        out = [a for a in restrict if a.kind in m.kinds[dom[1]].subtree(m)]
        out.sort(key=lambda x: (day_lo(x.day) or END, order_key(x.order)))
        return out
    key = ('mvs', dom, self.wkey(ctx), ctx.unc, id(ctx.walk) if ctx.walk is not None else None, ctx.wp, owner)
    hit = self.run_cache.get(key)
    if hit is not None:
        for e in self.run_cache.get(('mvpins',) + key, ()):
            self.reg(e)
        return hit
    out = []
    if dom[0] == 'obs':
        for o in self.observations(ctx):
            if o.row.inf.id == dom[1]:
                out.append(o)
        # L §5.19 Bindings: "A binding over an observed input, `o in X over s`, ranges over X's rows dated in s, each
        # once, whether or not a version states two of them `same`, since an input's keys are its rows" (OQ-3)
        self.run_cache[key] = out
        return out
    sub = m.kinds[dom[1]].subtree(m)
    pins = set()
    stc = []
    self.run_cache[('mvstat',) + key] = stc
    for mi in self.plan_moves():
        if mi.kind not in sub:
            continue
        for i in self.move_instances(mi, ctx):
            # judging its status reads its link: those reads are a reader's only where the instance's day can fall in
            # its span (movement_members; L §8.2, tally 2.3.1 R2 item 6, G2's review finding 4)
            c_st = self.push()
            try:
                st = self.status(i, ctx)
            finally:
                self.stack.pop()
            if st == 'none':
                # read to be left out: its `when`'s rows are the binding's where its day is in the span (as above)
                c_st.rows.update(_rows_only(getattr(i, 'coll', None)).rows)
            stc.append((i, c_st))
            if st in ('none', 'modelled'):
                # the binding read its day to leave it out, so the pins that day took are the binding's reads too
                # (R7; item 9 of the follow-up: a count that finds none lists its pin)
                ic = getattr(i, 'coll', None)
                pins |= {e for e in (ic.reg if ic is not None else ()) if e[0] == 'pinned'}
                continue
            if st == 'uncertain' and not ctx.unc:
                continue
            if st == 'hole' and not ctx.unc and self.kind_certainty(i.kind) == 'uncertain':
                # a binding counts an unfulfilled movement of an uncertain kind only with `with uncertain`, whatever
                # its day (L §5.14 Certainty; tally 2.3.1 R1, N250.1)
                continue
            out.append(i)
    self.run_cache[('mvpins',) + key] = pins
    for e in sorted(pins, key=str):
        self.reg(e)
    for o in self.observations(ctx):
        if o.kind in sub:
            out.append(o)
    if 'adjustments' in sub:
        out.extend(self.adjustments(ctx, settled=False))
        out.extend(self.hole_adjustments(ctx))
    decs = [q for q in sorted(m.moves) if m.moves[q].decision and m.moves[q].kind in sub and
            item_in_world(m.moves[q])]
    if 'pads' in sub or decs:
        limit = None
        if ctx.walk is not None and ctx.wp is not None:
            # L §5.14.1 Running reads: "a binding over a decision's movements, or over pads: those booked up to this
            # point of the walk, as flows of a decision counts them" (writer 5's T-65)
            w = ctx.walk
            limit = ctx.wp
        else:
            w = self.main_walk(ctx)
        if 'pads' in sub:
            seen = set()
            ww = w
            while ww is not None:
                for p in ww.pads:
                    if p.id in seen:
                        continue
                    seen.add(p.id)
                    if limit is not None and (getattr(p, 'point', None) is None or p.point >= limit):
                        continue
                    out.append(p)
                ww = getattr(ww, 'base', None)
        for q in decs:
            for i in self.decision_instances(m.moves[q], ctx):
                r = None
                ww = w
                while ww is not None and r is None:
                    r = ww.decs.get(i.id)
                    ww = getattr(ww, 'base', None)
                if r is not None and limit is not None and (r.point is None or r.point >= limit):
                    continue
                if r is None and isinstance(i.day, Hole) and not ctx.unc and self.kind_certainty(i.kind) == 'uncertain':
                    # an unfulfilled decision of an uncertain kind, whatever its day (L §5.14 Certainty; N250.1)
                    continue
                if r is None and isinstance(i.day, Hole) and i.exists is not False and not getattr(i, 'err', None):
                    # a decision whose day is a hole is never walked, and is a member with its hole: "a movement whose
                    # day is a hole makes every flows and binding over a span it might fall in a hole" (L §5.14;
                    # tally 2.3 review of G1, B1); its legs are read as a plain hole day's are, so a `where` on its
                    # places is decided (a running read in its amount is the hole, its day being one)
                    self.hole_day_dec_body(m.moves[q], i, ctx)
                    out.append(i)
                    continue
                if r is not None and (r.exists is False or getattr(r, 'status', None) == 'none'):
                    # read to be left out: its reads' rows are the binding's where its day is in the span (the third
                    # mechanism; movement_members merges stc's by day)
                    rc = _rows_only(getattr(r, 'coll', None))
                    rc.rows.update(_rows_only(getattr(r, 'exist_coll', None)).rows)
                    stc.append((r, rc))
                if r is None or r.exists is False or getattr(r, 'status', None) in ('modelled', 'none'):
                    continue
                if getattr(r, 'status', None) == 'uncertain' and not ctx.unc:
                    continue
                out.append(r)
    if owner is not None:
        from .joint import owns
        out = [x for x in out if owns(owner, move_owner(x))]
    out = once_by_same(m, out, ctx, owner)
    out.sort(key=lambda x: (day_lo(x.day) or END, order_key(x.order)))
    self.run_cache[key] = out
    return out


def m_hole_day_dec_body(self, mi, i, ctx):
    """A hole-day decision's legs, read once outside the walk (B1): the walk never runs it, and a binding's `where`
    reads its places. Its places and amounts are its day's hole, by name and through a binding alike: a decision's
    legs are read in the walk at its day (E §4.2 step 3; tally 2.3.1 R1, N250.8, C-13.9)."""
    if getattr(i, '_hd_body', False):
        return
    i._hd_body = True
    # its reads are the member's, as a walked decision's are, not the binding's that met it
    from .evaluate import Coll
    self.stack.append(Coll())
    try:
        legs = self.refused_places(mi, i.env, i, self.tctx(ctx))
    finally:
        self.stack.pop()
    decl = {ln.name: d for ln, d in mi.legs}
    for lg in legs:
        lg.amount = i.day
        lg.unit = self.leg_unit_static(decl[lg.name], lg.frm, lg.to)
        lg.frm = lg.to = i.day
    i.legs = legs


def once_by_same(m, out, ctx, owner=None):
    """Two movements stated same are one: "a movement listed twice counts once" (L §6.1), so a binding over them
    binds the left one of the same line, of the lines seen at ctx (moves.same_lines, R12). Inside a member (owner),
    only the same lines of its own versions: a member is closed, so the joint model's lines change none of its
    answers (L §6.1)."""
    from .joint import owns
    sm = [(a, b, loc) for a, b, loc, o in same_lines(m, ctx) if owner is None or owns(owner, o)]
    if not sm:
        return out
    ids = set(getattr(x, 'id', None) for x in out)
    # same is an equivalence (L §6.1, round 14): a group counts once, by its first member in id order, as S6 names
    # its rows (L §6.1: section order changes nothing; tally 2.1.8 P42, group 4's review N2)
    parent = {}

    def find(x):
        while parent.get(x, x) != x:
            x = parent[x]
        return x
    order = []
    for a, b, loc in sm:
        for x in (a, b):
            if x not in order:
                order.append(x)
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[rb] = ra
    drop = set()
    kept = set()
    from .moves import id_order
    for x in sorted(order, key=lambda y: id_order(m, y)):
        if x not in ids:
            continue
        r = find(x)
        if r in kept:
            drop.add(x)
        else:
            kept.add(r)
    return [x for x in out if getattr(x, 'id', None) not in drop]


def m_same_sides_reg(self, x, ctx):
    """The other sides of instance x's `same` group, each unfulfilled one registered: one movement, its register every
    side its line names (rulings 17 B)."""
    g = same_groups(self.m, ctx).get(x.id)
    if not g:
        return
    K = ctx.cut
    by_id = dict((i.id, i) for i in self.plan_insts(ctx))
    for y in g:
        i = by_id.get(y)
        if y == x.id or i is None or getattr(i, 'obs', None):
            continue
        if (not i.decision and i.status == 'past') or (
                i.decision and day_lo(i.day) is not None and day_lo(i.day) < K):
            self.reg(('unfulfilled', y))


def m_same_kind_hole(self, x, kind, ctx):
    """L §6.1: "One movement has one base kind (P-2), so rows that state different kinds disagree: a binding over
    either kind that meets the movement is a HOLE-SAME hole naming both rows" (OQ-2).  A movement of a same group
    whose members state kinds not all inside the binding's kind is an unknown member of it; where every kind the
    group states is inside the binding's kind, the movement is a member either way (round 17: "A binding over a kind
    that holds both kinds meets the one movement once, whatever its kind")."""
    from .joint import shown
    m = self.m
    if not hasattr(x, 'id'):
        return None
    g = same_groups(m, ctx).get(x.id)
    if g is None:
        return None
    idx = {}
    for o in self.observations(ctx):
        idx[o.id] = o
    if any(y not in idx for y in g):
        for i in self.plan_insts(ctx):
            idx.setdefault(i.id, i)
    mem = [idx[y] for y in g if y in idx]
    kinds = []
    for o in mem:
        k = getattr(o, 'kind', None)
        if k is not None and k not in kinds:
            kinds.append(k)
    if len(kinds) < 2:
        return None
    sub = m.kinds[kind].subtree(m)
    if all(k in sub for k in kinds):
        return None
    from .moves import and_list
    mem = [o for o in mem if getattr(o, 'kind', None) is not None]
    a = mem[0]
    d = x.day if isinstance(x.day, datetime.date) else next(
        (o.day for o in mem if isinstance(o.day, datetime.date)), None)
    # L §8.1 HOLE-SAME, the rows in the order of their ids (same_groups'), each of a larger class named (tally 2.1.8 S6)
    q = '%s are one, but the kind differs on %s: %s' % (
        and_list(shown(o.id) for o in mem), d if isinstance(d, datetime.date) else 'every day',
        and_list('%s (%s)' % (o.kind, shown(o.id)) for o in mem))
    return Hole([(q, ', '.join(shown(o.id) for o in mem))])


def m_obs_booked(self, o, ctx):
    """Whether an observation is booked (L §5.19 Booking an observation, E §4.2 step 1): dated before the cut, and
    touching a place a version covers that day."""
    if not isinstance(o.day, datetime.date) or not o.day < ctx.cut or o.err is not None:
        return False
    for g in o.legs:
        gd = g.day if isinstance(g.day, datetime.date) else o.day
        for p in (g.frm, g.to):
            if isinstance(p, Thing) and self.covering(p.name, gd, ctx):
                return True
    return False


def m_movement_members(self, b, env, ctx, defer=False):
    """A binding `x in K on d` / `over s` over an event kind or an observed input: the movements dated that day or
    in the span; one whose day is a hole, or a window across the span's end, is an unknown member (L §5.14, §4.4)."""
    from .evaluate import EMPTY_BIND
    if b.on is not None:
        d = self.bind_day(b, b.on, env, ctx)
        if d is EMPTY_BIND:
            if defer:
                b._colls = {}
            return []
        if isinstance(d, Hole):
            return d
        d = self.date_of(d, 'the day of a binding')
        sp = Span(d, next_day(d))
    else:
        sp = self.bind_day(b, b.over, env, ctx, span=True)
        if sp is EMPTY_BIND:
            if defer:
                b._colls = {}
            return []
        if isinstance(sp, Hole):
            return sp
    res, colls = self.span_members(b._dom, sp, ctx, getattr(b, '_owner', None))
    if getattr(b, '_owner', None) is not None:
        res = [(v, s, x) for v, s, x in res if member_moves(self.m, b, x)]
    if defer:
        b._colls = colls
    else:
        for c in colls.values():
            self.cur.merge(c)
    return [(v, s) for v, s, x in res]


def m_span_members(self, dom, sp, ctx, owner=None):
    """The movements of an event kind or an observed input dated in span sp, in movement order, each with its status
    (True, or a Hole for an unknown member) and the movement itself: ([(Mv, status, movement)], {id: Coll}), the
    Coll holding what the member brings to a reader (L §5.14, §4.4, §5.19)."""
    class _B:
        pass
    b = _B()
    b._dom = dom
    if ctx.walk is not None and ctx.wp is not None and b._dom[0] == 'event':
        # L §5.14.1 Running reads: "A span past the reader's day, in a binding or in flows, is its future, refused as
        # a plain bal(x, d) after its day is (LOAD-CYCLE where load sees the day, RUN-FUTURE otherwise)"
        m = self.m
        sub = m.kinds[b._dom[1]].subtree(m)
        t = kday(ctx.wp)
        arr = getattr(ctx.walk, 'in_arrival', None)
        if arr is not None and ('pads' in sub or any(mi.decision and mi.kind in sub for mi in m.moves.values())) and \
                (sp.end is None or sp.end > t):
            # an arrival rule's running read of its arrival's day's end or later is RUN-CYCLE (L §5.14.1, 2.1-C7b-3)
            raise arrival_span_cycle(arr, '%s over %s' % (b._dom[1], span_text(sp)))
        if ('pads' in sub or any(mi.decision and mi.kind in sub for mi in m.moves.values())) and \
                (sp.end is None or sp.end > next_day(t)):
            raise refuse('RUN-FUTURE', 'a decision on %s binds over %s through %s, after its day' % (
                t, b._dom[1], (sp.end - ONE_DAY) if sp.end is not None else 'the end'))
    out = []
    fulfilled = set()
    xs = self.movements_of(b._dom, ctx, owner)
    for i, c_st in self.run_cache.get(('mvstat', 'mvs', b._dom, self.wkey(ctx), ctx.unc,
                                         id(ctx.walk) if ctx.walk is not None else None, ctx.wp, owner), ()):
        if not (isinstance(i.day, datetime.date) and not sp.contains(i.day)):
            self.cur.merge(c_st)
    for x in xs:
        e1 = self.s1_refusal(getattr(x, 'id', None), ctx)
        if e1 is not None and (sp.contains(x.day) if isinstance(x.day, datetime.date) else True):
            # a binding over movements rests on each it takes (L §5.19 S1, P42): a count or a sum is refused
            raise e1
        sh = self.same_in_span(x, sp, ctx) if b._dom[0] == 'event' else None
        d = x.day if sh is None else sh
        st = True
        if d is None and x.err is not None:
            # refused before its day was known: whether it is a member is refused with it (L §8.4), a refused member
            # (rulings 17 A: `any`/`all` decide past it; every other reader is refused by it)
            out.append((x.mv(), x.err, x))
            continue
        if isinstance(d, Hole):
            st = d
        elif isinstance(d, datetime.date):
            if not sp.contains(d):
                if getattr(x, 'day_coll', None) is not None:
                    # the reads that put its day outside the span (L §8.2; tally 2.2.1 P35, N1)
                    self.day_reads(x.day_coll)
                tsp = getattr(x, 'tspan', None)
                if tsp is not None and tsp[0] is not None and self.trace_stack:
                    # a decision run outside the binding's span is out of its read set: its traced reads go (L
                    # §8.2; tally 2.3.1 R2, P18, explain's second mechanism)
                    drop = set(tsp[1])
                    tsp[0].children = [c for c in tsp[0].children if id(c) not in drop]
                continue
        else:
            continue
        if isinstance(x, (Inst, AdjMv)) and isinstance(x.exists, Hole):
            # an adjustment of a hole-day instance: its existence is the day's hole (E §5 step 1; tally 2.3, C-4)
            st = x.exists if st is True else merge(st, x.exists)
        cq =getattr(x, 'class_member', False) if isinstance(x, Inst) else False
        if cq:
            # booked on some of its window's days, modelled or left out on the rest (moves.m_window_class_depends):
            # whether it is a member is the window's hole (E §4.1 step 2; item 10 of the follow-up)
            h = carry_hole(Hole([('date within %s not precise enough for a binding over %s' % (
                d.window_text(), b._dom[1]), x.id)]), d)
            st = h if st is True else merge(st, h)
        if isinstance(x, Inst) and x.err is not None:
            # a refused member (rulings 17 A)
            out.append((x.mv(), x.err, x))
            continue
        if isinstance(x, ObsMv) and x.target is not None:
            fulfilled.add(x.target.id)
        if b._dom[0] == 'event':
            kh = self.same_kind_hole(x, b._dom[1], ctx)
            if kh is not None:
                st = kh if st is True else merge(st, kh)
        out.append((x.mv(), st, x))
    ids = set(x.id for v, s, x in out if isinstance(x, Inst))
    if fulfilled & ids:
        # L §5.19: "a bind over an event kind includes the observations of that kind, counting a planned instance and
        # the observations that fulfil it once: by the observations where they are booked (Booking an observation),
        # otherwise by the instance"
        drop = set()
        for iid in fulfilled & ids:
            obs = [x for v, s, x in out if isinstance(x, ObsMv) and x.target is not None and x.target.id == iid]
            booked = [o for o in obs if self.obs_booked(o, ctx)]
            if booked:
                drop.add(iid)
                drop |= set(o.id for o in obs if o not in booked)
            else:
                drop |= set(o.id for o in obs)
        out = [(v, s, x) for v, s, x in out if x.id not in drop]
    res = []
    K = ctx.cut
    colls = {}
    for v, s, x in out:
        # what the member brings to a reader (its registers, what made it): merged by the binding's reader only when
        # the member passes the binding's where
        c = self.push()
        try:
            if isinstance(x, Inst) and not x.decision and x.status == 'past' and not x.obs:
                self.reg(('unfulfilled', x.id))
            elif isinstance(x, Inst) and x.decision and day_lo(x.day) is not None and day_lo(x.day) < K and \
                    not getattr(x, 'obs', None):
                self.reg(('unfulfilled', x.id))
            if isinstance(x, Inst):
                # a `same` group binds once, by its first side (once_by_same); its register lists every side its
                # line names, each unfulfilled one (rulings 17 B; p10 ny, p09 cnt)
                self.same_sides_reg(x, ctx)
            if isinstance(x, Inst) and getattr(x, 'status', None) == 'uncertain':
                self.reg(('uncertain', x.id))
            if x.coll is not None:
                self.cur.merge(x.coll)
            for r in getattr(x, 'reg', None) or ():
                # a binding that counts a pad movement lists it, whatever it reads of it (L §8.2's `pad:` row;
                # tally 2.2.1 P35, N3)
                self.reg(r)
        finally:
            self.stack.pop()
        colls[x.id] = c
        res.append((v, s, x))
    return res, colls


# -- balances (L §5.10, §5.14.1; E §4.2) -----------------------------------------------------------------------------

def m_place_arg(self, node, env, ctx):
    p = self.ev(node, env, ctx)
    if isinstance(p, Hole):
        return p
    if p is NONE:
        raise refuse('RUN-NONE', '%s is none here: a place' % env_txt(node, env))
    if not isinstance(p, Thing) or p.name not in self.m.things or not self.m.is_place_thing(self.m.things[p.name]):
        raise refuse('RUN-DOMAIN', '%s is not a place' % fmt(p))
    return p


def fmt_node(n):
    from .check import _txt
    return _txt(n)


def m_bal_unit(self, n, place, env, ctx):
    m = self.m
    pu = m.place_unit(m.things[place])
    if getattr(n, '_unit', None) is not None:
        u = self.ev(n._unit, env, ctx)
        if isinstance(u, Hole):
            return u
        if isinstance(u, Thing):
            uname = u.name
        elif isinstance(u, Key):
            uname = u.name
        else:
            uname = getattr(u, 'v', None) or fmt(u)
        if pu not in (None, 'any') and uname != pu:
            raise refuse('LOAD-UNIT', 'bal(%s, %s, d): %s holds %s' % (place, uname, place, pu))
        return uname
    if pu in (None, 'any'):
        raise refuse('LOAD-UNIT', 'bal(%s, d) of a place of unit any states its unit: bal(x, u, d)' % place)
    return pu


def m_run_ends(self, place, d):
    """The walk holds the days S to E: a day outside them is a hole naming the end it passes, for every read of a
    balance (L §8.1 HOLE-PRERUN, HOLE-POSTRUN; E §2; tally 2.1.7 S1, #93), or None."""
    if d < self.m.S:
        return Hole([('before the run\'s start', place)])
    if d > self.m.E:
        return Hole([('after the run\'s end', place)])
    return None


def m_read_state(self, w, place, unit, key):
    """The balance of a place at an instant of a walk, with its register (L §8.2). The run's ends are tested here,
    once, so every walked read (bal, value, bal of, a running read) meets them alike (the review's finding 2)."""
    d = kday(key)
    out = self.run_ends(place, d)
    if out is not None:
        return out
    st = w.pstate(place, unit, key, raw=True)
    self.merge_cum(w, place, unit, key)
    for (ad, e) in w.adj_regs.get(place, ()):
        if ad <= d:
            self.reg(e)
    ww = w
    while ww is not None:
        for mid in sorted(getattr(ww, 'leftout_by_place', {}).get(place, ())):
            n, first = ww.leftout_regs[mid]
            from .ask import day_text
            self.reg(('left out', '%s, %d instances, first %s' % (mid, n, day_text(first))))
        ww = ww.base
    self.joint_left_out(w, place, key)
    self.later_reads(w, place, lambda ld: ld is None or ld > d)
    v = w.st_value(st, place, unit)
    if isinstance(v, TallyError):
        raise v
    return v


LATER_KINDS = ('pinned', 'assumed')


def _rows_only(c):
    """A collector holding c's rows alone (no register line), for inputs-of's read set."""
    from .evaluate import Coll
    out = Coll()
    if c is not None:
        out.rows.update(c.rows)
    return out


def m_day_reads(self, dc):
    """What the reads that put a movement's day outside a read list there: the pins and the assumed rows N1 names
    (a law date's held: rows are the statement's too, but 60 stored records would move: W8's hand-off)."""
    for r in (dc.reg if dc is not None else ()):
        if r[0] in LATER_KINDS:
            self.reg(r)
    if dc is not None and self.cur is not None:
        # the rows those reads read are the read set's, which inputs-of lists (L §8.2 One read set: "the reads that
        # decided a movement doesn't count"; tally 2.3.1 R2, explain's third mechanism); the register as above
        self.cur.rows.update(dc.rows)


def m_later_reads(self, w, place, outside):
    """The reads that decided a movement at place doesn't count (its day outside the read, outside(day) true; left
    out, day None): in the read set like any other (L §8.2 One read set; tally 2.2.1 P35, N1)."""
    for ld, dc in getattr(w, 'later_by_place', {}).get(place, ()):
        if outside(ld):
            self.day_reads(dc)


def m_joint_left_out(self, w, place, key):
    """L §6.1 (round 17b): "Every result that reads the place on or after the movement's day lists it under left
    out: (§8.2)": the joint model's own unlinked movements into the place dated at or before the read, though a
    reading of the member's since then states the balance (the walk's reset drops them from the balance's
    collector)."""
    lim = key
    ww = w
    while ww is not None:
        for k, r in getattr(ww, 'joint_lo', {}).get(place, ()):
            if k <= lim:
                self.reg(r)
        if ww.base is not None and ww.point is not None:
            lim = min(lim, ww.point)
        ww = ww.base


def m_merge_cum(self, w, place, unit, key):
    st, lg, i = w.state(place, unit, key)
    c = w.cum(lg, i)
    if c is not None:
        self.cur.merge(c)
    reset_seen = lg is not None and i >= 0 and st.rst >= 0
    if w.base is not None and not reset_seen:
        self.merge_cum(w.base, place, unit, w.point)


def m_e_bal(self, n, env, ctx):
    form = n._form
    p = self.place_arg(n.ref, env, ctx)
    if isinstance(p, Hole):
        return p
    place = p.name
    unit = self.bal_unit(n, place, env, ctx)
    if isinstance(unit, Hole):
        return unit
    if getattr(n, 'of', None) is not None:
        # tally 2.1 (L §5.10): bal(x, d) of q, bal(x, d start) of q, and in a decision the running bal(x) of q
        return self.e_bal_of(n, place, unit, env, ctx)
    if form == 'now':
        w = ctx.walk
        if w is None or ctx.wp is None:
            raise refuse('LOAD-READS', 'bal(%s) with no day outside a decision' % place)
        note_read(w, place, True)
        v = self.read_state(w, place, unit, ctx.wp)
        v = less_own_obs(w, place, unit, ctx.wp, v)
        if self.tracing and self.trace_stack:
            # E §10 (round 14): a walk-bound def's cell explained with "its running reads as balances up to that
            # point"
            from .evaluate import TraceNode
            from .ask import fmt_value
            self.trace_stack[-1].children.append(('node', TraceNode('group', 'running read bal(%s) = %s, the balance '
                                                                    'up to this point of %s' % (
                                                                        place, fmt_value(v), kday(ctx.wp)))))
        return v
    d = self.ev(n.args[-1], env, ctx)
    if isinstance(d, Hole):
        return d
    if isinstance(d, Iv) and isinstance(d.lo, datetime.date) and d.lo != d.hi:
        raise self.imprecise(d, 'the day of bal(%s, d)' % place)
    d = self.date_of(d, 'the day of bal')
    if self.tracing and self.trace_stack and form != 'proj' and getattr(self, 'explainer', None) is not None:
        # E §10, §3.2 (tally 2.1.7 R7, #132b): a cause that is a balance prints in the balance's own form, its last
        # reading or opening and the movements since, not the walk's own reads
        from .evaluate import TraceNode
        saved, self.trace_stack, self.tracing = self.trace_stack, [], False
        tn = TraceNode('balread', 'bal(%s, %s%s)' % (place, d, ' start' if getattr(n, 'start', False) else ''))
        try:
            tn.value = self.bal_at(n, place, unit, d, env, ctx)
            asker, a = self.explainer
            tn.lines = asker.balance_cause(self, a, n, env, ctx)
        except Refusal as e:
            # a refused balance prints its refusal, with no balance form under it
            tn.refused = e
            raise
        finally:
            self.trace_stack, self.tracing = saved, True
            saved[-1].children.append(('node', tn))
        return tn.value
    return self.bal_at(n, place, unit, d, env, ctx)


def m_bal_at(self, n, place, unit, d, env, ctx):
    """bal(x, d) with the read's own clauses (L §5.14.1). A day outside the run is its hole (run_ends) before the
    read's own forms: a projection's and the modelled walk's checks come after it."""
    if isinstance(d, datetime.date):
        out = self.run_ends(place, d)
        if out is not None:
            return out
    form = n._form
    if form == 'proj':
        return self.proj_bal(n, place, unit, d, env, ctx)
    if form == 'modelled':
        # bal(x, d) in the model's own account, at the plain read's instant (L §5.19; tally 2.1.8 R16, B5)
        return self.bal_modelled(place, unit, d, ctx, start=getattr(n, 'start', False))
    key = K_start(d) if getattr(n, 'start', False) else K_end(d)
    v = self.bal_day(place, unit, d, key, ctx)
    # a plain read counts uncertain movements only when it states the clause; the line's clause reaches
    # projections and bindings (L §5.14, §5.18)
    unc = n.proj.get('uncertain') if n.proj else False
    if unc and not isinstance(v, TallyError):
        v = self.with_uncertain(place, unit, d, v, ctx)
    return v


def note_read(w, place, live):
    """Record a running decision's read of a place (explain's "read at its walk point"), and, where the read sees the
    place at the decision's point (live), for the order of the day's decisions (walk.live)."""
    if w.reads is not None:
        w.reads.add(place)
    if live and getattr(w, 'live', None) is not None:
        w.live.add(place)


def less_own_obs(w, place, unit, key, v):
    """A modelled-past decision is computed against the booked balances less its own observations: one dated before
    its day was booked on its own day (E §4.2 step 3, L §5.19). w.less_obs holds those observations' legs while it
    runs."""
    legs = getattr(w, 'less_obs', None)
    if not legs or not isinstance(v, Num):
        return v
    delta = Fraction(0)
    for lg, frm_booked, to_booked in legs:
        o = lg.day.toordinal()
        if not (key[0] > o or (key[0] == o and key[1] > 1)):
            continue
        if not isinstance(lg.amount, Num):
            continue
        if unit not in (None, '*') and lg.unit not in (None, '*') and lg.unit != unit:
            continue
        # only the ends it booked: of two rows stated same, each books the end its statement covers (L §5.19)
        if frm_booked and isinstance(lg.frm, Thing) and lg.frm.name == place:
            delta += lg.amount.q
        if to_booked and isinstance(lg.to, Thing) and lg.to.name == place:
            delta -= lg.amount.q
    if delta == 0:
        return v
    return add_q(v, Num(delta, v.u, money=v.money), 1)


def span_text(sp):
    a = sp.start.isoformat() if sp.start is not None else ''
    b = (sp.end - ONE_DAY).isoformat() if sp.end is not None else ''
    return '%s..%s' % (a, b)


def arrival_span_cycle(arr, what):
    """L §5.14.1 (2.1-C7b-3): an arrival rule's running read of its arrival's day's end or later, reached through a
    computed day, is RUN-CYCLE (`<x>[k] reads itself through <path>`, L §8.4), never RUN-FUTURE, a decision's."""
    return refuse('RUN-CYCLE', '%s[%s, *] reads itself through %s: the end of its arrival\'s day or later' % (
        arr[0], arr[1], what))


def m_bal_day(self, place, unit, d, key, ctx):
    """A plain balance on a day: at the ask's cut, or a running read in a decision (L §5.14.1)."""
    w = ctx.walk
    if w is not None and ctx.wp is not None:
        t = kday(ctx.wp)
        note_read(w, place, False)
        if getattr(w, 'in_arrival', None) is not None:
            # tally 2.1 (L §5.14.1): the arrival rule's reads of its day's end or later (partswalk)
            from .partswalk import arrival_cycle
            arrival_cycle(w, ctx, key, place, d)
        if key[0] > ctx.wp[0] or (key[0] == ctx.wp[0] and key[1] >= 3):
            if d > t:
                raise refuse('RUN-FUTURE', 'a decision on %s reads %s at %s after its day' % (t, place, d))
            raise refuse('RUN-CYCLE', '%s reads itself through bal(%s, %s): a decision\'s plain read of its own day'
                         % (dec_move(ctx.wp[3]), place, d))
        return less_own_obs(w, place, unit, key, self.read_state(w, place, unit, key))
    w = self.main_walk(ctx)
    v = self.read_state(w, place, unit, key)
    if ctx.partial and isinstance(v, Hole):
        v = self.partial_bal(w, place, unit, key, v)
    return v


def m_partial_bal(self, w, place, unit, key, v):
    """A balance read by a partial ask line: "a balance means ... its opening plus what is booked since" (L §5.19,
    §5.10), a sum, so the line prints its known part and the holes it is linear in (L §5.18 Partial answers): the booked
    known part plus each hole leg's amount, signed by its direction at the place, as the line reads it (MC-B-F2). A
    hole of any other making (an anchor, a window, a hole movement's day, a sign rule) leaves the plain hole."""
    from .evaluate import lin_of, lin_add
    st, lg, i = w.state(place, unit, key)
    if lg is None and w.base is not None and w.point is not None and st.hole and st.val is not None and \
            st.bad is None and not st.perm and not st.imp:
        # a projection that books nothing at the place: its start balance, the base's at the point (B9)
        return self.partial_bal(w.base, place, unit, w.point, v)
    if st.bad is not None or st.val is None or st.perm or st.imp or lg is None:
        return v
    acc = lin_of(st.val)
    seen = frozenset()
    if st.rst < 0 and w.default(place, unit).hole:
        # a projection's walk starts from its base's balance at t (E §4.3): the start balance plus the counted legs,
        # so its hole terms are the base's at the point, read the same way (L §5.18; tally 2.1.8 R15, B9 of group
        # 3's review); its known part is already in this walk's value
        d0 = w.default(place, unit)
        if w.base is None or w.point is None:
            return v
        bv = self.partial_bal(w.base, place, unit, w.point, Hole(d0.hole))
        if not isinstance(bv.lin, tuple) or bv.lin[0] is None:
            return v
        acc = lin_add(acc, (None, bv.lin[1]), 1)
        seen |= frozenset(d0.hole)
    for e in lg.ents[st.rst + 1:i + 1]:
        if e.kind != 'hole':
            continue
        lh = self.leg_lin(e.leg, e.sgn, e.qs)
        if lh is None:
            return v
        acc = lin_add(acc, lh, 1)
        seen |= e.qs
    # every hole the read carries is a leg's amount: a hole movement's day, which the walk carries beside them, is no
    # operand (L §5.18; tally 2.1.8 R15, #169-i)
    if not acc[1] or set(seen) != set(st.hole) or set(seen) != set(v.qs):
        return v
    return Hole(v.qs, acc)


def m_leg_lin(self, lgx, sgn, qs):
    """A hole leg's amount as its linear form, read again under partial and signed by its direction (L §5.18; tally
    2.1.8 R15), or None: a plain hole, or a form naming other holes than the leg's."""
    from .evaluate import lin_of, lin_scale
    src = getattr(lgx, 'src', None)
    if sgn is None or src is None:
        return None
    node, env, c = src
    h = self.ev(node, env, c.but(partial=True))
    lh = lin_of(h) if isinstance(h, Hole) else None
    if lh is None or set(a for a, _c in lh[1]) != set(qs):
        return None
    return lin_scale(lh, Num(sgn, (), isint=True))


def lin_fold(known, holes):
    """A walked sum under partial (L §5.18; tally 2.1.8 R15): its known part plus each hole's linear form; the plain
    hole unless every hole is one (h.lin a form) and the known part is a number."""
    from .evaluate import lin_add
    qs = set()
    for h in holes:
        qs |= h.qs
    if not isinstance(known, Num) or any(not isinstance(h.lin, tuple) for h in holes):
        return Hole(qs, False)
    acc = (known, ())
    for h in holes:
        acc = lin_add(acc, h.lin)
    return Hole(qs, acc)


def m_with_uncertain(self, place, unit, d, v, ctx):
    """bal(x, d) with uncertain: plus the unfulfilled movements of uncertain kinds dated through d (L §5.14)."""
    w = self.main_walk(ctx)
    st, lg, i = w.state(place, unit, K_end(d))
    since = lg.ents[st.rst].key if (lg is not None and st.rst >= 0) else None
    add = Fraction(0)
    hs = []
    for x in w.uncertain:
        for g in x.legs:
            gd = g.day if isinstance(g.day, datetime.date) else day_lo(x.day)
            if gd is None or gd > d or (since is not None and K_end(gd) <= since):
                continue
            for p, s in ((g.frm, -1), (g.to, 1)):
                if isinstance(p, Thing) and p.name == place:
                    self.reg(('uncertain', x.id))
                    if x.coll is not None:
                        self.cur.merge(x.coll)
                    if isinstance(g.amount, Hole):
                        hs.append(self.partial_leg(g, s, ctx))
                    elif isinstance(g.amount, TallyError):
                        raise g.amount
                    else:
                        add += s * g.amount.q
    if ctx.partial and (isinstance(v, Hole) or hs):
        # L §5.18 (tally 2.1.8 R15): the total is a sum, linear in the balance and the uncertain legs
        if isinstance(v, Hole):
            if not isinstance(v.lin, tuple) or v.lin[0] is None:
                return merge(v, *hs)
            k, parts = v.lin[0], [Hole(v.qs, (None, v.lin[1]))]
        else:
            k, parts = v, []
        known = qty(k.q + add, unit) if k.money or k.u else Num(k.q + add, k.u)
        return lin_fold(known, parts + hs)
    if isinstance(v, Hole) or hs:
        return merge(v, *hs)
    return qty(v.q + add, unit) if v.money or v.u else Num(v.q + add, v.u)


def m_partial_leg(self, g, sgn, ctx):
    """A hole leg's amount as an operand of a walked sum: under partial, its linear form signed (leg_lin)."""
    a = g.amount
    if not ctx.partial:
        return a
    lh = self.leg_lin(g, sgn, a.qs)
    return Hole(a.qs, lh if lh is not None else False)


def m_proj_clauses(self, proj, ctx):
    rnd = ctx.rnd
    if proj.get('random') is not None:
        rnd = self.rnd_clause(proj['random'], ctx)
    unc = proj.get('uncertain', ctx.unc)
    return rnd, unc


def m_rnd_clause(self, r, ctx):
    """A `random at mean|quantile(q)` clause as (mode, q)."""
    if r[0] == 'quantile':
        q = self.ev(r[1], {}, ctx)
        if not isinstance(q, Num):
            raise refuse('RUN-DOMAIN', 'the level of random at quantile is %s' % fmt(q))
        return ('quantile', q.q)
    return ('mean', None)


def m_proj_bal(self, n, place, unit, d, env, ctx):
    """`bal(x, d) from t` (L §5.14.1, E §4.3)."""
    t = self.ev(n.proj['frm'], env, ctx)
    if isinstance(t, Hole):
        return t
    t = self.date_of(t, 'the day of a projection')
    if d < t:
        raise refuse('RUN-DOMAIN', 'bal(%s, %s) from %s: the day is before the projection\'s start' % (place, d, t))
    if t < self.m.S:
        # L §5.14.1 (round 12): "a t before S is a hole, HOLE-PRERUN: nothing before S is walked"
        return Hole([('before the run\'s start', place)])
    clauses = self.proj_clauses(n.proj, ctx)
    w = self.proj_walk(ctx, t, n._leftout, clauses)
    if ctx.walk is not None:
        note_read(ctx.walk, place, True)
    w.run_to(d)
    v = self.read_state(w, place, unit, K_end(d))
    if ctx.partial and isinstance(v, Hole):
        # a projection's balance is a sum as the plain one is (L §5.18; tally 2.1.8 R15)
        v = self.partial_bal(w, place, unit, K_end(d), v)
    self.proj_regs(w, t, d)
    if clauses[1] and (not isinstance(v, Hole) or ctx.partial):
        # with uncertain partial: the uncertain amounts join the known part of a hole's linear form (L §5.18, §5.14;
        # tally 2.1.8 R15, B3 of group 3's review)
        v = self.proj_uncertain(w, place, unit, d, v, ctx)
    return v


def m_proj_regs(self, w, t, d):
    for mid in sorted(w.counted_leftout):
        days = [x for x in w.counted_leftout[mid] if t <= x <= d]
        if days:
            self.reg(('left out', '%s, %d instances, first %s' % (mid, len(days), min(days))))


def m_proj_uncertain(self, w, place, unit, d, v, ctx=None):
    add = Fraction(0)
    hs = []
    for x in w.uncertain:
        for g in x.legs:
            gd = g.day if isinstance(g.day, datetime.date) else day_lo(x.day)
            if gd is None or gd > d or gd < w.start:
                continue
            for p, s in ((g.frm, -1), (g.to, 1)):
                if isinstance(p, Thing) and p.name == place:
                    self.reg(('uncertain', x.id))
                    if x.coll is not None:
                        self.cur.merge(x.coll)
                    if isinstance(g.amount, Hole):
                        hs.append(self.partial_leg(g, s, ctx) if ctx is not None else g.amount)
                    elif isinstance(g.amount, TallyError):
                        raise g.amount
                    else:
                        add += s * g.amount.q
    if hs and ctx is not None and ctx.partial and not isinstance(v, Hole):
        return lin_fold(qty(v.q + add, unit), hs)
    if ctx is not None and ctx.partial and isinstance(v, Hole) and isinstance(v.lin, tuple) and v.lin[0] is not None:
        # the start balance plus the counted legs, linear in its hole terms and the uncertain legs (R15)
        k = v.lin[0]
        known = qty(k.q + add, unit) if k.money or k.u else Num(k.q + add, k.u)
        return lin_fold(known, [Hole(v.qs, (None, v.lin[1]))] + hs)
    if hs or isinstance(v, Hole):
        return merge(v, *hs)
    return qty(v.q + add, unit)


def m_modelled_account(self, w, place, unit, key, d):
    """`bal(x, d) modelled` is bal(x, d) in the model's own account (L §5.19; tally 2.1.8 R16): from x's last reading
    before day d (the opening, before the first; a reading on day d is what it is compared with), every booked movement
    of x since that is no observation, with every instance the observations stand in for (the modelled past) at the
    point its walk books it, up to the read's instant.
    Returns (lg, r0, ents, legs): r0 the start reading's index in lg (-1: the place's default), ents the log entries
    counted, legs the modelled instances' legs, (x, g, sign, key) or (x, None, err or hole, None) for an instance whose
    day is refused or a hole, which may fall anywhere."""
    from .walk import same_book_key
    st, lg, i = w.state(place, unit, key)
    r0 = -1
    if lg is not None:
        st0, _, i0 = w.state(place, unit, (d.toordinal(), -2, 0, '', 0))
        r0 = st0.rst if i0 >= 0 else -1
        if r0 < 0:
            r0 = next((j for j in range(i, -1, -1) if lg.ents[j].kind == 'reset' and
                       lg.ents[j].note == 'opening' and lg.ents[j].key[0] == d.toordinal()), -1)
    since = lg.ents[r0].key if (lg is not None and r0 >= 0) else None
    ents = []
    if lg is not None:
        for e in lg.ents[r0 + 1:i + 1]:
            if isinstance(e.mv, ObsMv) or e.kind == 'reset':
                # an observation is the reality the model is compared with; a reading on day d its comparison
                continue
            ents.append(e)
    legs = []
    for x in w.modelled:
        for g in x.legs:
            if not any(isinstance(p, Thing) and p.name == place for p in (g.frm, g.to)):
                continue
            if not isinstance(g.day, datetime.date) and day_lo(x.day) is None:
                bad = x.err if getattr(x, 'err', None) is not None else x.day if isinstance(x.day, Hole) else None
                if bad is not None:
                    legs.append((x, None, bad, None))
                continue
            bk = same_book_key(x, g)
            if bk > key or (since is not None and bk <= since):
                continue
            for p, s in ((g.frm, -1), (g.to, 1)):
                if isinstance(p, Thing) and p.name == place:
                    legs.append((x, g, s, bk))
    return lg, r0, ents, legs


def m_bal_modelled(self, place, unit, d, ctx, start=False):
    """`bal(x, d) modelled` (L §5.19; tally 2.1.8 R16): the account modelled_account gives, at the plain read's
    instant (`start` included), with its holes and refusals."""
    w = self.main_walk(ctx)
    key = K_start(d) if start else K_end(d)
    lg, r0, ents, legs = self.modelled_account(w, place, unit, key, d)
    if lg is None or r0 < 0:
        v = w.default(place, unit).val
    else:
        e = lg.ents[r0]
        v = e.val
        if e.qs:
            return Hole(e.qs)
    if v is None:
        return Hole([w.anchor_question(place)])
    q = v.q
    hs = []
    for e in ents:
        if e.kind in ('bad', 'pbad'):
            raise e.err
        if e.kind == 'delta':
            q += e.flow.q
        elif e.kind in ('hole', 'perm') and e.qs:
            hs.append(Hole(e.qs))
        if e.coll is not None:
            self.cur.merge(e.coll)
    for x, g, s, _bk in legs:
        if g is None:
            # an instance whose day is refused or a hole may fall on any day (L §8.4, §5.14 `on d`)
            if isinstance(s, TallyError):
                raise s
            hs.append(s)
            continue
        if x.coll is not None:
            self.cur.merge(x.coll)
        if isinstance(g.amount, Hole):
            hs.append(g.amount)
        elif isinstance(g.amount, TallyError):
            raise g.amount
        else:
            q += s * g.amount.q
    if hs:
        return merge(*hs)
    return qty(q, unit)


# -- flows (L §5.14.3) --------------------------------------------------------------------------------------------

def m_e_flows(self, n, env, ctx):
    sp = self.span_value(n.span, env, ctx)
    if isinstance(sp, Hole):
        return sp
    if sp.start is None or sp.start < self.m.S:
        # the read with its keys' values (L §8.1; tally 2.2.1, 235.6)
        from .evaluate import env_txt
        return Hole([('before the run\'s start', env_txt(n.ref, env))])
    fk = None
    if n.forkey is not None:
        fk = self.ev(n.forkey, env, ctx)
        if isinstance(fk, Hole):
            return fk
    if n._form == 'move':
        return self.flows_move(n, sp, fk, env, ctx)
    p = self.place_arg(n.ref, env, ctx)
    if isinstance(p, Hole):
        return p
    return self.flows_place(p.name, sp, env, ctx, fk=fk)


def mv_for(mv):
    """The period a movement is for (L §5.14.3, round 12): "A movement with no for (of a move without one, a pad, an
    observation that fulfils nothing) is for no period ...; an observation that fulfils an instance is for the
    instance's period"."""
    if mv is None:
        return None
    if getattr(mv, 'status', None) == 'observed':
        t = getattr(mv, 'target', None)
        return getattr(t, 'forp', None) if t is not None else None
    return getattr(mv, 'forp', None)


def m_flows_place(self, place, sp, env, ctx, fk=None):
    m = self.m

    def other_for(e):
        # `, for k` keeps only the movements for period k (L §5.14.3)
        if fk is None or e.mv is None:
            return False
        fp = mv_for(e.mv)
        return fp is None or vkey(fp) != vkey(fk)
    last = sp.end - ONE_DAY if sp.end is not None else m.E
    w = ctx.walk
    limit = K_end(last)
    if w is not None and ctx.wp is not None:
        t = kday(ctx.wp)
        note_read(w, place, last >= t)
        arr = getattr(w, 'in_arrival', None)
        if arr is not None and last >= t:
            # an arrival rule's running read of its arrival's day's end or later is RUN-CYCLE (L §5.14.1, 2.1-C7b-3)
            raise arrival_span_cycle(arr, 'flows(%s, %s)' % (place, span_text(sp)))
        if last > t:
            raise refuse('RUN-FUTURE', 'a decision on %s reads flows of %s through %s, after its day' % (
                t, place, last))
        if last == t:
            limit = ctx.wp
    else:
        w = self.main_walk(ctx)
    lo = K_before(sp.start)
    # E §4.3 step 1 (tally 2.1.2, issue #38): a projection's walk continues the walk it starts from, so the days up to
    # its start point are that walk's, as a balance reads them (Walk.default) and flows of a decision move do
    # (m_flows_move): each walk of the chain gives its entries up to the point the walk after it starts after
    chain = [(w, limit)]
    while chain[-1][0].base is not None:
        ww, cut = chain[-1]
        chain.append((ww.base, min(cut, ww.point)))
    units = sorted(set(u for ww, cut in chain for (p, u) in ww.logs if p == place and u != '*'))
    pu = m.place_unit(m.things[place])
    if pu not in (None, 'any'):
        units = [pu]
    totals = {}
    holes = []
    for w, limit in chain:
        self.flows_place_walk(w, limit, place, units, lo, sp, last, env, ctx, fk, other_for, totals, holes)
    if last > m.E:
        self.flows_post_e(chain[-1][0], place, sp, last, env, ctx, fk, totals, holes)
    w, limit = chain[0]
    for (ad, e) in w.adj_regs.get(place, ()):
        if ad <= last:
            self.reg(e)
    # the joint model's own unlinked movements into the place on or before the span's last day (L §6.1, round 17b)
    self.joint_left_out(w, place, limit)
    if holes and ctx.partial and pu != 'any' and len(totals) <= 1 and (totals or units or pu):
        # flows is a sum of its movements (L §5.18; tally 2.1.8 R15): linear in each hole leg's amount
        if totals:
            u, q = list(totals.items())[0]
        else:
            u, q = (units[0] if len(units) == 1 else pu), 0
        if u is not None:
            return lin_fold(qty(q, u), holes)
    if holes:
        return merge(*holes)
    # L §5.14.3 (round 12): "Its unit is that of the places it reads ...; a place of unit any, movements in two
    # units, or an empty span whose unit load can't see is refused (LOAD-UNIT where load sees it, RUN-DOMAIN
    # otherwise)"
    if pu == 'any':
        raise refuse('RUN-DOMAIN', 'flows of %s, a place of unit any: it sums no one unit' % place)
    if len(totals) > 1:
        raise refuse('RUN-DOMAIN', 'flows of %s: movements in %s' % (place, ' and '.join(sorted(totals))))
    if not totals:
        if len(units) == 1:
            return qty(0, units[0])
        if pu not in (None, 'any'):
            return qty(0, pu)
        raise refuse('RUN-DOMAIN', 'flows of %s over a span with no movement: its unit is unknown' % place)
    u, q = list(totals.items())[0]
    return qty(q, u)


def m_refuse_ent(self, e):
    """A refused entry a read meets: its refusal, with the entries its reads gave (L §8.2 l.2248: a refusal lists them
    like a value; tally 2.2.1 P35, B1)."""
    for r in e.reg:
        self.reg(r)
    if e.coll is not None:
        self.cur.merge(e.coll)
    raise e.err


def m_flows_post_e(self, w, place, sp, last, env, ctx, fk, totals, holes):
    """flows of a place over days after E (E §2 Movements outside [S, E]; tally 2.1.7 S1, #93): the walk books none
    of them (walk.note_post_e), and flows counts them, a plan's instance with its amount and a decision's instance as
    a hole movement, HOLE-POSTRUN, since its amount reads a walk that doesn't hold its day (L §8.1)."""
    from .evaluate import HoleSignal
    for i in w.post_e:
        if fk is not None and (mv_for(i) is None or vkey(mv_for(i)) != vkey(fk)):
            continue
        if i.decision:
            if not sp.contains(i.day):
                continue
            try:
                ps = set()
                for ln, _d in i.move.legs:
                    for e in (ln.frm, ln.to):
                        p = self.ev(e, i.env, ctx)
                        ps.add(p.name if isinstance(p, Thing) else None)
            except (TallyError, HoleSignal):
                ps = {None}
            if place in ps or None in ps:
                holes.append(Hole([('after the run\'s end', i.id)]))
            continue
        for lg in i.legs or ():
            lo, hi = day_lo(lg.day), day_hi(lg.day)
            if not isinstance(lo, datetime.date) or hi < sp.start or lo > last:
                continue
            sides = [s for q, s in ((lg.frm, -1), (lg.to, 1)) if isinstance(q, Thing) and q.name == place]
            if not sides:
                continue
            if i.err is not None:
                raise i.err
            if i.coll is not None:
                self.cur.merge(i.coll)
            if isinstance(i.exists, Hole):
                holes.append(i.exists)
                continue
            if lo != hi and not (sp.contains(lo) and hi <= last):
                holes.append(carry_hole(Hole([('date within %s not precise enough for flows of %s' % (
                    lg.day.window_text(), place), i.id)]), lg.day))
                continue
            a = lg.amount
            if isinstance(a, TallyError):
                raise a
            if isinstance(a, Hole):
                holes.append(a)
                continue
            u = unit_of_value(a)
            for s in sides:
                totals[u] = totals.get(u, Fraction(0)) + s * a.q


def _reask(h, what):
    """A window's hole the walk worded for a balance, asked as the read's own question (§14: the printed question of
    a hole is the read's), each question carrying the days the walk's did."""
    from .evaluate import CARRY
    tail = ' not precise enough for a balance'
    qs = []
    for q in h.qs:
        t, nm = q
        if isinstance(t, str) and t.startswith('date within ') and t.endswith(tail):
            q2 = (t[:-len(tail)] + ' not precise enough for ' + what, nm)
            if q in CARRY and q2 not in CARRY:
                CARRY[q2] = CARRY[q]
            qs.append(q2)
        else:
            qs.append(q)
    return Hole(qs, h.lin)


def _zero_flow(f):
    """A window day's movement of a known amount 0 adds 0 to flows on whichever day it falls, so its value doesn't
    depend on the day (L §4.4; tally 2.1.8 P50, group 4's fix review: m2_g3_fix2_r3 fa, m2_gate8_window_moves)."""
    return f is not None and not isinstance(f, Hole) and getattr(f, 'q', None) == 0


def _zero_at(mv, place):
    """Whether every leg of mv at place is of a known amount 0 (_zero_flow)."""
    legs = [lg for lg in getattr(mv, 'legs', None) or ()
            if any(getattr(x, 'name', None) == place for x in (lg.frm, lg.to))]
    return bool(legs) and all(_zero_flow(lg.amount) for lg in legs)


def m_flows_place_walk(self, w, limit, place, units, lo, sp, last, env, ctx, fk, other_for, totals, holes):
    """One walk's part of flows of a place: its entries in (lo, limit], added to totals and holes."""
    # a movement at the place its day keeps outside the span: the reads that decided it (L §8.2 "a day outside the
    # span"; tally 2.2.1 P35, N1)
    st0 = getattr(sp, 'start', None)
    self.later_reads(w, place, lambda ld: ld is None or ld > last or (st0 is not None and ld < st0))
    # what holds for the whole walk (a refused movement whose places aren't known, a decision refused, a movement
    # whose day is a hole): a balance reads it (Walk.pstate), and so does a place's flows, which could count it
    # (L §8.4, L §5.14 `on d`: "a movement whose day is a hole makes every flows ... over a span it might fall in a
    # hole"); without this, such a movement was left out of flows in silence
    glog = w.logs.get(('*', '*'))
    if glog is not None:
        k0 = K_before(w.start) if getattr(w, 'start', None) is not None else None
        for e in glog.ents:
            if e.key > limit:
                continue
            if e.kind in ('bad', 'pbad') and (e.key >= lo or e.key == k0):
                # tally 2.1.7 R2 (#107): a refusal keyed before a day's first event is the refusal "from its day"
                # (K_before): a span that starts on that day may hold the movement, so its flows is refused too
                self.refuse_ent(e)
            if e.kind in ('perm', 'hole') and e.qs:
                holes.append(Hole(e.qs))
                if e.coll is not None:
                    self.cur.merge(e.coll)
    for u in units + ['*']:
        lg = w.logs.get((place, u))
        if lg is None:
            continue
        wins = {}
        closed = set()
        # a perm before the span is a hole of the span unless its window ended before it (an unperm: the decision has
        # fallen, walk.dec_win_end; B5 of group 3's review)
        perms = []
        # a `same` group is one movement, its day a hole bounded by its sides' days (rulings 17 B; L l.1359): a span
        # holding the first and the last counts it known (the first's hole and the last's `unsame` both inside), one
        # holding the last alone is its hole (G1's review, N9: p09 f_a, f_b)
        in_sp = [e for e in lg.ents if lo < e.key <= limit]
        before = set(e.qs for e in lg.ents if e.key <= lo and e.kind == 'hole' and e.qs)
        opened = set(e.qs for e in in_sp if e.kind == 'hole' and e.qs) - before
        closing = set(e.qs for e in in_sp if e.kind == 'unsame' and e.qs in opened)
        # a group opened before the span and closed after it may fall in it: the hole (p09 f_6)
        uns = {}
        for e in lg.ents:
            if e.kind == 'unsame' and e.qs:
                uns.setdefault(e.qs, []).append(e.key)
        for q in sorted(before, key=str):
            if q in uns and all(k > limit for k in uns[q]):
                holes.append(_reask(Hole(q), 'flows of %s' % place))
        prev_unsame = None
        for e in lg.ents:
            if prev_unsame is not None and e.kind == 'delta' and e.key == prev_unsame.key:
                # the leg the late `unsame` books: its group may have fallen before the span, so the hole stands; the
                # side is still the read's, listed (rulings 17 B: every side its line names, p10 fa)
                prev_unsame = None
                for r in e.reg:
                    self.reg(r)
                if e.coll is not None:
                    self.cur.merge(e.coll)
                continue
            prev_unsame = None
            if lo < e.key <= limit and e.kind == 'hole' and e.qs in closing:
                # the group's first side, counted known at its last: its register is still the read's (N1)
                for r in e.reg:
                    self.reg(r)
                if e.coll is not None:
                    self.cur.merge(e.coll)
                continue
            if lo < e.key <= limit and e.kind == 'unsame' and e.qs and e.qs not in opened:
                holes.append(_reask(Hole(e.qs), 'flows of %s' % place))
                prev_unsame = e
                continue
            if e.kind == 'delta' and e.wid is not None:
                closed.add(e.wid)
            if e.key <= lo:
                if e.kind == 'perm' and e.key <= limit:
                    perms.append(e)
                if e.kind == 'unperm':
                    perms = [x for x in perms if x.mv is not e.mv]
                if e.kind == 'pbad' and e.key <= limit:
                    # a refused movement whose day is a hole or refused may fall in the span, as perm's (R11)
                    self.refuse_ent(e)
                if e.kind == 'imp_on':
                    wins[e.wid] = e.win
                continue
            if perms:
                holes.extend(Hole(x.qs) for x in perms)
                perms = []
            if e.key > limit:
                if e.kind == 'delta' and e.wid is not None and e.wid in wins and not _zero_flow(e.flow):
                    holes.append(carry_hole(Hole([('date within %s not precise enough for flows of %s' % (
                        e.win.window_text(), place), e.mv.id)]), e.win))
                continue
            if e.kind in ('bad', 'pbad'):
                self.refuse_ent(e)
            if e.kind == 'imp_on':
                wins[e.wid] = e.win
                continue
            if other_for(e) or (fk is not None and e.kind == 'reset'):
                continue
            if e.kind == 'hole' and not e.qs:
                # a window side counted by another, ended here (Walk.book_leg): nothing to count
                continue
            if e.kind == 'mark':
                # a covered movement nothing fulfils: flows lists what the balance lists (P35; tally 2.2.1 P43 S5)
                for r in e.reg:
                    self.reg(r)
                if e.coll is not None:
                    self.cur.merge(e.coll)
                continue
            if e.kind in ('hole', 'perm'):
                lh = self.leg_lin(e.leg, e.sgn, e.qs) if ctx.partial and e.kind == 'hole' else None
                holes.append(_reask(Hole(e.qs, lh if lh is not None else False), 'flows of %s' % place))
                # its register too, as a balance takes it (Walk.cum): the joint model's own movement into a member's
                # place, a hole there, is listed under every result that reads the place (L §6.1, round 17; §8.2
                # left out:)
                for r in e.reg:
                    self.reg(r)
                if e.coll is not None:
                    self.cur.merge(e.coll)
                continue
            f = e.flow
            if f is None or e.kind not in ('delta', 'reset'):
                continue
            if e.kind == 'delta' and e.wid is not None:
                win = e.win
                if not (sp.contains(win.lo) and win.hi <= last):
                    if not _zero_flow(f):
                        holes.append(carry_hole(Hole([('date within %s not precise enough for flows of %s' % (
                            win.window_text(), place), e.mv.id)]), win))
                    continue
            for r in e.reg:
                self.reg(r)
            if e.coll is not None:
                self.cur.merge(e.coll)
            if isinstance(f, Hole):
                holes.append(f)
                continue
            totals[u] = totals.get(u, Fraction(0)) + f.q
        holes.extend(Hole(x.qs) for x in perms)
        # a movement whose window opened by the span's end and closes after the walk's end books no delta in this walk:
        # whether it falls in the span is the window's hole, as for one that closes after the span (L §4.4; tally
        # 2.1.8 §4, #169-e: a window crossing the run's end)
        for wid, win in sorted(wins.items(), key=lambda x: str(x[0])):
            overlaps = win.lo <= last and (sp.start is None or win.hi >= sp.start) if isinstance(win, Iv) else False
            if wid not in closed and overlaps and not (sp.contains(win.lo) and win.hi <= last):
                mv = next((e.mv for e in lg.ents if e.kind == 'imp_on' and e.wid == wid), None)
                if _zero_at(mv, place):
                    continue
                holes.append(carry_hole(Hole([('date within %s not precise enough for flows of %s' % (
                    win.window_text(), place), mv.id if mv is not None else place)]), win))


def m_flows_move(self, n, sp, fk, env, ctx):
    """flows(m[keys], span [, for k]): the move's movements in the span, settlements included (L §5.14.3)."""
    m = self.m
    mi = m.moves[n._move]
    leg = getattr(n, '_leg', None)
    keys = []
    r = getattr(n, '_mref', n.ref)
    keynodes = r.keys if r.k == 'index' else []
    for kn, cv, b in zip(keynodes, n._convs, mi.binds):
        if cv[0] == 'lit':
            keys.append(vkey(cv[1]) if not isinstance(cv[1], int) else vkey(cv[1]))
        else:
            v = self.ev(kn, env, ctx)
            if isinstance(v, Hole):
                return v
            k = self.to_key(v, cv, b.dom, kn) if not isinstance(v, Mv) else v
            keys.append(vkey(k))
    last = sp.end - ONE_DAY if sp.end is not None else m.E
    pre_holes = []
    if mi.decision:
        w = ctx.walk
        limit = None
        if w is not None and ctx.wp is not None:
            t = kday(ctx.wp)
            arr = getattr(w, 'in_arrival', None)
            if arr is not None and last >= t:
                # an arrival rule's running read of its arrival's day's end or later (L §5.14.1, 2.1-C7b-3)
                raise arrival_span_cycle(arr, 'flows(%s, %s)' % (mi.id, span_text(sp)))
            if last > t:
                raise refuse('RUN-FUTURE', 'a decision on %s reads flows of %s through %s, after its day' % (
                    t, mi.id, last))
            if last == t:
                limit = ctx.wp
        else:
            w = self.main_walk(ctx)
        insts = []
        for i in self.decision_instances(mi, ctx):
            run = None
            ww = w
            while ww is not None and run is None:
                run = ww.decs.get(i.id)
                ww = ww.base
            if run is None:
                if isinstance(i.day, datetime.date) and i.day > m.E and i.exists is not False:
                    # a decision instance after E: never walked, so its amount is a hole flows counts (E §2 Movements
                    # outside [S, E], HOLE-POSTRUN; tally 2.1.7 S1, #93), as a place's flows does (flows_post_e)
                    i._postrun = True
                    insts.append(i)
                    continue
                if isinstance(i.day, Hole) and i.exists is not False:
                    # L §5.14 on d (TX2-F2, writer 10): "A movement whose day is a hole makes every flows and binding
                    # over a span it might fall in a hole (G-MP-2). A day that is a hole and not a window has no
                    # bound, so that is every span": a decision instance with such a day has no walk point, and is
                    # counted as the hole it is
                    i._rawhole = True
                    insts.append(i)
                continue
            if limit is not None and run.point is not None and run.point >= limit:
                continue
            insts.append(run)
    elif keys and len(keys) == len(mi.binds):
        # one instance: its own cell, so a tuple that reads another of its move is no cycle (L §5.14 existence;
        # tally 2.3.1 R6, C-13.10)
        one = self.tuple_inst(mi, keys, ctx)
        insts = [one] if one is not None else []
    else:
        insts = self.move_instances(mi, ctx)
    total = None
    holes = list(pre_holes)
    u = None
    seen_units = set()
    counted = set()
    for i in insts:
        if keys and tuple(vkey(k) for k in i.keys[:len(keys)]) != tuple(keys):
            continue
        if getattr(i, '_rawhole', False):
            if i.err is not None:
                raise i.err
            holes.append(i.day)
            continue
        if getattr(i, '_postrun', False):
            if sp.contains(i.day) and (fk is None or (i.forp is not None and vkey(i.forp) == vkey(fk))):
                holes.append(Hole([('after the run\'s end', i.id)]))
            continue
        if mi.decision:
            st = i.status
        else:
            # judging its status reads its link; that read is this reader's only where the instance's day can fall in
            # the span: one out of it by its day rests on no link here (L §8.2, §5.19; tally 2.3.1 R2, item 6)
            c_st = self.push()
            try:
                st = self.status(i, ctx)
            finally:
                self.stack.pop()
            if not (isinstance(i.day, datetime.date) and not sp.contains(i.day)):
                self.cur.merge(c_st)
        if st in ('none', 'uncertain'):
            if st == 'none' and not (isinstance(i.day, datetime.date) and not sp.contains(i.day)):
                # an observation fulfils it, so the link is refused: what rests on it is refused (L §8.4; tally 2.3.1
                # R6, C-13.6)
                bad = self.run_cache.get(('badlinks', self.wkey(ctx)), {}).get(i.id)
                if bad is not None:
                    if getattr(i, 'exist_coll', None) is not None:
                        self.cur.merge(i.exist_coll)
                    raise bad
            if st == 'none' and isinstance(i.day, datetime.date) and sp.contains(i.day) and \
                    getattr(i, 'coll', None) is not None and self.cur is not None:
                # an instance in the span whose when is false is read to be left out: its reads are the line's, as a
                # binding's (L §8.2; P48, r1_when f_mv: pinned: on_)
                self.cur.merge(i.coll)
            continue
        d = i.day
        if d is None and i.err is not None:
            # a movement whose day is refused may fall in any span: its refusal (L §8.4, tally 2.1.6)
            raise i.err
        if not mi.decision and st != 'modelled':
            # a movement stated same is in the span by every side's day (L §6.1 l.1797; tally 2.2.1 P36, N4)
            sh = self.same_in_span(i, sp, ctx)
            if sh is not None:
                holes.append(sh)
                continue
        if isinstance(d, Hole):
            if i.err is not None:
                # a refused movement whose day is a hole reaches every span as a hole day does, refused (L §8.4 "a
                # refused day as a hole one"; §8: a refusal outranks a hole; tally 2.3.1 R6, w3)
                raise i.err
            holes.append(d)
            continue
        if not sp.contains(d):
            if getattr(i, 'day_coll', None) is not None:
                # the reads that put its day outside the span (L §8.2; tally 2.2.1 P35, N1)
                self.day_reads(i.day_coll)
            continue
        if st == 'hole' and getattr(i, 'class_q', None):
            # booked on some of its window's days, modelled or left out on the rest (E §4.1 step 2, L §4.4)
            holes.append(carry_hole(Hole([('date within %s not precise enough for flows of %s' % (
                d.window_text(), mi.id), i.id)]), d))
            continue
        if fk is not None and not isinstance(i.forp, Hole) and (i.forp is None or vkey(i.forp) != vkey(fk)):
            continue
        counted.add(id(i))
        e1 = self.s1_refusal(i.id, ctx)
        if e1 is not None:
            # an instance a class of rows stated same fulfils with another: what reads it is refused (S1, P42)
            raise e1
        if st == 'modelled':
            if mi.decision and getattr(i, 'exist_coll', None) is not None:
                # what decided the decision exists at its point rests under its link (G2's review finding 3)
                self.cur.merge(i.exist_coll)
            # on a day a statement covers, the observations that fulfil the instance stand in for it (L §5.14.3)
            obs = getattr(i, 'obs', None) or self.obs_links_for(i.id, ctx)
            bad = self.run_cache.get(('badlinks', self.wkey(ctx)), {}).get(i.id)
            if bad is not None:
                # its realised movement rests on a refused record (L §5.19, §8.4: a refusal propagates like a hole)
                raise bad
            if not obs:
                self.reg(('open', i.id))
                if i.coll is not None:
                    # what decided it, as the balance's open: entry carries it (P35; tally 2.2.1 P43 S5)
                    self.cur.merge(i.coll)
            # rows stated same are one movement, counted once by flows, their disagreement a HOLE-SAME (L §5.19)
            for o, others in once_obs(m, obs, ctx):
                for x in [o] + others:
                    if x.coll is not None:
                        self.cur.merge(x.coll)
                # the group's every row, its fulfils "stated on either row" (L §5.19)
                h = self.same_rows_hole(o, leg, ctx)
                if h is not None:
                    holes.append(h)
                    continue
                for lg in o.legs:
                    if leg is not None and lg.name != leg:
                        continue
                    a = lg.amount
                    if isinstance(a, Hole):
                        holes.append(a)
                        continue
                    total = a.q if total is None else total + a.q
                    u = unit_of_value(a)
                    seen_units.add(u)
            continue
        if i.coll is not None:
            # a refused result lists what it read like a value (L §8.2 One read set; tally 2.2.1, #208)
            self.cur.merge(i.coll)
        if st == 'refused' or i.err is not None:
            raise i.err
        if isinstance(i.exists, Hole):
            holes.append(i.exists)
            continue
        if st == 'past' and not getattr(i, 'obs', None):
            self.reg(('unfulfilled', i.id))
        if fk is not None and isinstance(i.forp, Hole):
            # a for the window doesn't decide (moves.body_over_law; L §4.4, #169-e): whether it counts is its hole
            holes.append(i.forp)
            continue
        for lg in i.legs:
            if leg is not None and lg.name != leg:
                continue
            a = lg.amount
            if isinstance(a, TallyError):
                raise a
            if isinstance(a, Hole):
                holes.append(self.partial_leg(lg, 1, ctx))
                continue
            ha = self.same_rows_hole(i, lg.name, ctx, 'amount') if not mi.decision else None
            if ha is not None:
                # its amount read from both sides: never one side's figure (L §6.1 l.1797; tally 2.2.1 P36, N4)
                holes.append(ha)
                continue
            err = self.leg_refused(i.id, lg.name, ctx, lg.day if isinstance(lg.day, datetime.date) else i.day) \
                if a.q != 0 else None
            if isinstance(err, Hole):
                # a window leg whose lifespan test some of its days pass (walk.window_lifespan): the flows read's hole
                w = lg.day if isinstance(lg.day, Iv) else d
                if not isinstance(w, Iv):
                    # a leg of a fixed day the window's test reaches: the hole it gives (walk.imp_hole), asking the
                    # read's own question (tally 2.1.8 §14, P54: `for flows of ob`, not the walk's `for a balance`)
                    holes.append(_reask(err, 'flows of %s' % mi.id))
                    continue
                holes.append(carry_hole(Hole([('date within %s not precise enough for flows of %s' % (
                    w.window_text(), mi.id), i.id)]), w))
                continue
            if err is not None:
                raise err
            total = a.q if total is None else total + a.q
            u = unit_of_value(a)
            seen_units.add(u)
    if mi.decision and self.trace_stack:
        # the runs of the walk this flows doesn't count are out of its read set: their traced reads go (L §8.2 One
        # read set: "nothing a list or a walk read for another reader"; tally 2.3.1 R2, P18)
        for i in insts:
            tsp = getattr(i, 'tspan', None)
            if tsp is not None and tsp[0] is not None and id(i) not in counted:
                drop = set(tsp[1])
                tsp[0].children = [c for c in tsp[0].children if id(c) not in drop]
    # settlements dated in the span, and HOLE-CHANGED (E §5 steps 5-6): only this move's adjustments are measured
    # unless it has some, whose settlements need the whole pass (E §4.1)
    own = self.adjust_move(mi, ctx, one=insts[0] if insts and keys and len(keys) == len(mi.binds) else None) \
        if not mi.decision else []
    if own:
        self.settle(own, ctx)
        for a in own:
            if a.inst.move.id != mi.id or isinstance(a.exists, Hole):
                # a hole-day instance's adjustment (C-4): flows of its move is its day's hole already
                continue
            if leg is not None and a.leg.name != leg:
                continue
            if keys and tuple(vkey(k) for k in a.inst.keys[:len(keys)]) != tuple(keys):
                continue
            if a.err is not None:
                # tally 2.1.2 (issue #34): a refused adjustment refuses flows over a span reaching V.known, where an
                # unsettled one is the HOLE-CHANGED hole
                if last >= a.day:
                    raise a.err
                continue
            if not a.settled and last >= a.day:
                holes.append(Hole([('%s on %s changed after it was paid (version %s); is back pay due, and when?' % (
                    a.inst.id, day_text(a.inst.day), ', '.join(a.vids)), a.inst.id)]))
                continue
            rem = a.remainder()
            if rem is not None and not (isinstance(rem, Num) and rem.q == 0) and last >= a.day:
                self.reg(('adjustment', '%s by %s on %s (version %s), remaining %s' % (
                    a.inst.id, fmt(a.delta), a.day, ', '.join(a.vids), fmt(rem) if not isinstance(rem, Hole)
                    else fmt_hole(rem))))
            for s, sign, amt in a.settled:
                sd = s.day
                if not isinstance(sd, datetime.date) or not sp.contains(sd) or sign == 0:
                    continue
                if isinstance(amt, Hole):
                    holes.append(amt)
                    continue
                total = sign * amt.q if total is None else total + sign * amt.q
                u = u or unit_of_value(amt)
    if holes and ctx.partial and len(seen_units) <= 1:
        # flows of a move is a sum of its movements (L §5.18; tally 2.1.8 R15)
        lt = getattr(n, '_t', None)
        known = qty(total or 0, u) if u is not None else (
            Money(total or 0, lt.cur) if lt is not None and lt.k == 'money' else
            Num(total or 0, lt.u) if lt is not None and lt.k == 'num' and lt.u else
            qty(total or 0, n._leg_unit) if getattr(n, '_leg_unit', None) is not None else None)
        if known is not None:
            return lin_fold(known, holes)
    if holes:
        return merge(*holes)
    # L §5.14.3 (round 12): the unit of flows of a move is its leg's places', known at load where they have one fixed
    # unit, otherwise taken from the movements it counts; movements in two units, or an empty span whose unit load
    # can't see, are refused (RUN-DOMAIN here)
    if len(seen_units) > 1:
        raise refuse('RUN-DOMAIN', 'flows of %s: movements in %s' % (mi.id, ' and '.join(sorted(
            str(x) for x in seen_units))))
    if u is None:
        lt = getattr(n, '_t', None)
        if lt is not None and lt.k == 'money':
            return Money(total or 0, lt.cur)
        if lt is not None and lt.k == 'num' and lt.u:
            return Num(total or 0, lt.u)
        pu = getattr(n, '_leg_unit', None)
        if pu is not None:
            return qty(total or 0, pu)
        raise refuse('RUN-DOMAIN', 'flows of %s over a span with no movement: its unit is unknown' % mi.id)
    return qty(total or 0, u)


# -- value (L §5.10) -----------------------------------------------------------------------------------------------

def m_e_value(self, n, env, ctx):
    from .evaluate import lin_of
    m = self.m
    p = self.place_arg(n.ref, env, ctx)
    if isinstance(p, Hole):
        return p
    place = p.name
    th = m.things[place]
    d = self.ev(n.d, env, ctx)
    if isinstance(d, Hole):
        return d
    d = self.date_of(d, 'the day of value')
    cur = n.cur
    priced = m.price_of(th)
    if th.base in m.kinds['assets'].subtree(m) and priced is None:
        # refused at load (check3.check_priced, L §5.10: LOAD-PRICE "asset <x> names no price input")
        raise refuse('LOAD-PRICE', 'asset %s names no price input' % place)
    if priced is not None and priced._mode == 'whole':
        # `priced p` with p a value in CUR: "priced p values a place whose single position is valued by the price
        # input p (tally 1's asset)" (L §5.7); `value` = p at d (L §5.10)
        v = self.priced_read(priced, th, None, d, n, env, ctx)
        if isinstance(v, Hole):
            if ctx.partial and priced._cur == cur and lin_of(v) is not None:
                # value passes its price on unchanged, so under partial its hole stays linear (L §5.18, tally 2.1.8
                # R15; T's STOP-REPORT 7, ASK-02)
                return Hole(v.qs, lin_of(v))
            return v
        amt = v.q
        if priced._cur != cur:
            pr = self.by_price(n, place, priced._cur, cur, d, env, ctx)
            if pr is None:
                raise refuse('LOAD-PRICE', 'value(%s) in %s holds units in %s and states no by' % (place, cur,
                                                                                                 priced._cur))
            if isinstance(pr, Hole):
                return pr
            amt = amt * pr.q
        return Num(amt, ((cur, 1),))
    pu = m.place_unit(th)
    if pu in (None, 'any'):
        w = self.main_walk(ctx) if ctx.walk is None else ctx.walk
        units = sorted(set(u for (pp, u) in w.logs if pp == place and u != '*'))
    else:
        units = [pu]
    if n.by is not None and priced is not None and priced._mode == 'unit' and priced._per != cur:
        # L §5.10 Value, §4.6 step 4 (tally 2.1.7 R4, F11a): a priced place's price has one home, so a `by` for a
        # position it already prices is a second claim on that price, tied (load refuses it for a place it sees)
        raise refuse('LOAD-PRICE', value_by_twice(n, place, priced._per))
    total = Fraction(0)
    holes = []
    ofq = None
    if getattr(n, 'of', None) is not None:
        # tally 2.1 (L §5.10): "value(x, d) in C [by p] of q is those parts at their prices"
        ofq = self.of_value(n.of, env, ctx)
        if isinstance(ofq, Hole):
            return ofq
    def price(u):
        pr = self.by_price(n, place, u, cur, d, env, ctx)
        if pr is None and priced is not None:
            per = self.priced_per(priced, th, u)
            if priced._cur != cur or per != u:
                raise refuse('LOAD-UNIT', 'value(%s) in %s: %s is priced in %s/%s, not %s/%s' % (
                    place, cur, place, priced._cur, per, cur, u))
            pr = self.priced_read(priced, th, u, d, n, env, ctx)
        elif pr is None:
            raise refuse('LOAD-PRICE', 'value(%s) in %s holds units in %s and states no by' % (place, cur, u))
        return pr
    from .evaluate import lin_of, lin_scale, lin_add
    # under partial, each hole position's term as a linear form (L §5.18; tally 2.1.8 R15): a hole number of units
    # at a known price, or a known number of units at a hole price; both holes, the plain hole
    lins = []
    for u in units:
        q = self.bal_day(place, u, d, K_end(d), ctx) if ofq is None else self.bal_of_day(place, u, d, K_end(d), ctx,
                                                                                         ofq)
        if isinstance(q, Hole):
            holes.append(q)
            if ctx.partial:
                pr = price(u) if u != cur else Num(1, (), isint=True)
                if isinstance(pr, Hole):
                    holes.append(pr)
                    lins.append(None)
                else:
                    lins.append(lin_scale(q.lin, pr) if isinstance(q.lin, tuple) else None)
            continue
        if u == cur:
            total += q.q
            continue
        if q.q == 0:
            continue
        pr = price(u)
        if isinstance(pr, Hole):
            holes.append(pr)
            if ctx.partial:
                lp = lin_of(pr)
                lins.append(lin_scale(lp, q) if lp is not None else None)
            continue
        total += q.q * pr.q
    if holes and ctx.partial and lins and all(lx is not None for lx in lins):
        acc = (Num(total, ((cur, 1),)), ())
        for lx in lins:
            acc = lin_add(acc, lx)
        return Hole(set().union(*[h.qs for h in holes]), acc)
    if holes:
        return merge(*holes)
    return Num(total, ((cur, 1),))


def value_by_twice(n, place, u):
    """LOAD-PRICE's form for a `by` that prices a position its place's price already gives (L §8.3)."""
    from .check import _txt
    by = ('last(%s)' % _txt(n.by)) if getattr(n, 'bylast', False) else _txt(n.by)
    return "value(%s) by %s: %s's price already gives its %s" % (place, by, place, u)


def m_by_price(self, n, place, u, cur, d, env, ctx):
    """The price of unit u in cur that value's `by` gives on day d (L §5.10): a named price read at the unit, or an
    expression with its keys (`by last(ref)` with keys is last(ref, d)), one path for every position, a place valued
    whole included (tally 2.1.7 R6, #126c); None when the read states no by. A unit that comes with the value is
    checked here, in load's words (L §8.4; tally 2.1.7 R5)."""
    if getattr(n, '_price_expr', False):
        pr = self.ev(getattr(n, '_by_expr', None) or n.by, env, ctx.but(law=d))
        if pr is NONE:
            raise refuse('RUN-NONE', 'the price of %s in %s on %s is none here' % (u, cur, d))
    elif getattr(n, '_price', None):
        pr = self.price_read(n._price, [Thing(u)], d, n, env, ctx, place, u)
    else:
        return None
    if isinstance(pr, Hole):
        return pr
    want = tuple(sorted([(cur, 1), (u, -1)]))
    if not isinstance(pr, Num) or tuple(sorted(pr.u)) != want:
        raise refuse('RUN-DOMAIN', 'value(%s) in %s: the price %s is not in %s/%s' % (place, cur, fmt(pr), cur, u))
    return pr


def m_priced_per(self, pn, th, u):
    """The unit a place's price is per at a position of unit u (L §3.2 `num CUR/u` with u the key; tally 2.1.7 B3):
    a price keyed by unit is per the key it is read at, the position's unit, or the place's own where it is keyed by
    the place."""
    per = pn._per
    if per is None or not per.startswith('$'):
        return per
    m = self.m
    kind, pid = pn._r
    info = m.inputs[pid] if kind == 'input' else m.defs[pid]
    dom = info.binds[0].dom if info.binds else None
    if dom is not None and dom[0] == 'kind' and m.thing_in(th.name, dom[1]):
        return m.place_unit(th)
    return u


def m_priced_read(self, pn, th, u, d, n, env, ctx):
    """The price a place is stated `priced` by, at the value's day d (L §5.10, round 10): `priced last(p)`
    reads p with its carry, last(p[k], d) (L §4.6); a plain `priced p` reads p as any reading is read (a
    hole on a day with no reading). p is keyed by nothing, by the place's kind, or by its positions' unit."""
    m = self.m
    kind, pid = pn._r
    info = m.inputs[pid] if kind == 'input' else m.defs[pid]
    ks = []
    ex = getattr(pn, '_expr', None)
    if ex is not None:
        # a reference with its keys (L §2.3 price), the place section's key variables bound to the place's keys
        pi, r = ex
        e2 = dict(env)
        if pi is not None and getattr(th, 'place', None) is pi:
            for b, k in zip(pi.binds, th.keys):
                e2[b.var] = k
        ks, hole = self.key_values(info, r.keys, r._convs, e2, ctx.but(law=d))
        if hole is not None:
            return hole
    elif info.binds:
        b = info.binds[0]
        dom = b.dom
        if dom[0] == 'kind' and m.thing_in(th.name, dom[1]):
            ks = [Thing(th.name)]
        elif dom[0] == 'kind' and u is not None and u in m.things and m.thing_in(u, dom[1]):
            ks = [Thing(u)]
        else:
            raise refuse('LOAD-KEY', '%s is priced by %s, keyed by %s, which holds neither %s nor its unit' % (
                th.name, pid, dom[1], th.name))
    c = ctx.but(law=d)
    if kind == 'def':
        v = self.read_def(info, ks, c)
    elif pn.last:
        v = self.last_of(info, ks, d, env, c)
    else:
        v = self.read_input(info, ks, c)
    if v is NONE:
        return Hole([('%s has no price in %s on %s' % (u or th.name, n.cur, d), '%s[%s]' % (
            pid, ', '.join(key_text(k) for k in ks)))])
    for f in getattr(pn, '_fields', ()):
        # `priced p.f`: the field of the record read (L §2.3 ref, §5.10; tally 2.1.7 S2, #101)
        if isinstance(v, Hole):
            return v
        v = v.get(f)
    return v


def m_price_read(self, pid, keys, d, n, env, ctx, place, u):
    m = self.m
    if getattr(n, '_price_kind', 'input') == 'def':
        # a price def keyed by unit (L §5.10), read at the value's day
        info = m.defs[pid]
        v = self.read_def(info, keys, ctx.but(law=d))
        if v is NONE:
            return Hole([('%s has no price in %s on %s' % (u or place, n.cur, d), '%s[%s]' % (
                pid, ', '.join(key_text(k) for k in keys)))])
        return v
    inf = m.inputs[pid]
    ks = keys if inf.binds else []
    if inf.binds:
        b = inf.binds[0]
        if b.dom[0] == 'kind' and keys and not m.thing_in(keys[0].name, b.dom[1]):
            ks = keys
    c = ctx.but(law=d)
    if getattr(n, 'bylast', False):
        v = self.last_of(inf, ks, d, env, c)
    else:
        v = self.read_input(inf, ks, c)
    if isinstance(v, Hole) or v is NONE:
        if v is NONE or isinstance(v, Hole):
            return Hole([('%s has no price in %s on %s' % (u or place, n.cur, d), '%s[%s]' % (
                pid, ', '.join(key_text(k) for k in ks)))]) if v is NONE else v
    return v


# -- covered, gap, observed, unit (L §5.19, §5.10) ----------------------------------------------------------------

def m_c_covered(self, n, env, ctx):
    """covered(x, span): versions known by the cut cover x on every day of the span (L §5.19); never a hole (L §8.1)."""
    p = self.ev(n.args[0], env, ctx)
    if isinstance(p, Hole) or p is NONE:
        return False
    from .evaluate import HoleSignal
    try:
        sp = self.span_value(n.args[1], env, ctx)
    except HoleSignal:
        # a span whose end is a window isn't known to be covered, as one whose end is a `?` isn't: false, never the
        # window's hole (L §8.1; tally 2.3, the lead's P7, rulings item 12)
        return False
    if isinstance(sp, Hole):
        return False
    if sp.start is None or sp.end is None:
        return False
    return covers_all([s for s, v in self.coverage(ctx).get(p.name, ())], sp)


def covers_all(spans, sp):
    """Whether the spans cover every day of span sp."""
    spans = sorted(spans, key=lambda s: s.start or datetime.date.min)
    d = sp.start
    while d < sp.end:
        nxt = None
        for s in spans:
            if s.contains(d):
                e = s.end
                if e is None:
                    return True
                if nxt is None or e > nxt:
                    nxt = e
        if nxt is None:
            return False
        d = nxt
    return True


def m_c_gap(self, n, env, ctx):
    """gap(x, d): the gap at the last stated reading of x on day d (L §5.19)."""
    m = self.m
    p = self.place_arg(n.args[0], env, ctx)
    if isinstance(p, Hole):
        return p
    d = self.ev(n.args[1], env, ctx)
    if isinstance(d, Hole):
        return d
    d = self.date_of(d, 'the day of gap')
    w = self.main_walk(ctx)
    gs = w.gaps.get((p.name, d))
    if not gs:
        ends = self.run_ends(p.name, d)
        if ends is not None and any(iid == 'stated' and pl == p.name and r.instant[0] == d
                                    for iid, pl, u, r in self.readings(ctx)):
            # a reading at an instant outside [S, E]: the booked balance it is less is the run's hole there, so the
            # gap is too (L §5.19 gap, L §8.1 HOLE-PRERUN, HOLE-POSTRUN; tally 2.1.8 #159)
            return ends
        return Hole([('stated[%s] has no reading at %s' % (p.name, d), 'stated[%s]' % p.name)])
    gs = sorted(gs, key=lambda x: x[0])
    key, gap, r, role, unit, bc = gs[-1]
    self.cur.merge(bc)
    self.cur.rows[r.sid] = r
    if isinstance(gap, TallyError):
        raise gap
    return gap


def m_c_observed(self, n, env, ctx):
    """observed(m[k]): the realised movement of a planned instance, or none (L §5.19)."""
    m = self.m
    a = n.args[0]
    v = self.e_moveref(a, env, ctx)
    if isinstance(v, Hole):
        return v
    mi = m.moves[a._r[1]]
    obs = self.obs_links_for(v.id, ctx)
    if mi.decision and v.id in self.run_cache.get(('deflinks', self.wkey(self.tctx(ctx))), ()):
        # a decision's link its running `when` leaves to its walk point is judged there (L §5.19, E §4.2 step 3;
        # tally 2.3.1 R6, C-9), what decided its existence with it (G2's review finding 3)
        w = self.main_walk(ctx)
        run = w.decs.get(v.id) if w is not None else None
        if run is not None and getattr(run, 'exist_coll', None) is not None:
            self.cur.merge(run.exist_coll)
    if mi.decision and mi.decl.when is not None and ctx.walk is None:
        # an instance whose existence is refused (its `when` in a RUN-ORDER cycle at its point, Walk.refused_run) is
        # refused with every read of it, `observed` too (R6, B5; x/dl_chk5 ob); a decision with no `when` exists
        # whatever its run, and `observed` reads its rows
        run = self.mv_record(v, ctx)
        if getattr(run, 'status', None) == 'refused' and getattr(run, 'err', None) is not None:
            raise run.err
    bad = self.run_cache.get(('badlinks', self.wkey(ctx)), {}).get(v.id)
    if bad is not None:
        raise bad
    if not obs:
        return NONE
    legs = []
    # rows stated same are one movement, counted once (L §5.19: "counted once by observed(m[k])"); where they
    # disagree, the leg's amount is the disagreement's hole, HOLE-SAME
    once = once_obs(m, obs, ctx)
    for o in obs:
        self.cur.rows[o.row.sid] = o.row
    for ld in m.kind_legs(mi.kind):
        tot = None
        frm = to = None
        day = None
        hole = dhole = None
        for o, others in once:
            # per fact (tally 2.2.1 P36; P48, the tests session's F-C): the amount's hole where the rows' amounts
            # differ, the day's where their days do; a fact they agree on is its value
            h = self.same_rows_hole(o, ld.name, ctx, 'amount')
            if hole is None:
                hole = h
            hd = self.same_rows_hole(o, ld.name, ctx, 'day')
            if dhole is None:
                dhole = hd
            for lg in o.legs:
                if lg.name != ld.name:
                    continue
                # L §5.19: "a leg's from or to is none where their legs name different places"
                if tot is None:
                    frm, to = lg.frm, lg.to
                else:
                    if frm is not NONE and vkey(frm) != vkey(lg.frm):
                        frm = NONE
                    if to is not NONE and vkey(to) != vkey(lg.to):
                        to = NONE
                tot = lg.amount if tot is None else Num(tot.q + lg.amount.q, tot.u, money=tot.money)
                day = lg.day if day is None or lg.day > day else day
        if tot is None:
            # unreachable: an observation that lacks a leg of the instance it fulfils is RUN-RECORD (L §5.19)
            raise refuse('RUN-RECORD', 'the observations fulfilling %s lack its leg %s' % (v.id, ld.name))
        if hole is not None:
            tot = hole
        if dhole is not None:
            day = dhole
        legs.append((ld.name, Rec([('from', frm), ('to', to), ('amount', tot), ('day', day)])))
    day = max(o.day for o in obs)
    return Rec(legs + [('day', day)])


def m_c_unit(self, n, env, ctx):
    if getattr(n, '_uq', False):
        # unit(q) of a quantity: the unit as a key of the unit kind that holds it (L §5.10 Kind and unit)
        v = self.ev(n.args[0], env, ctx)
        if isinstance(v, Hole):
            return v
        if v is NONE:
            raise refuse('RUN-NONE', '%s is none here: a quantity' % env_txt(n.args[0], env))
        us = list(getattr(v, 'u', None) or ())
        # in load's words (L §8.4 R-load, 2.1.7's R5; tally 2.2.1, 233.c)
        if not isinstance(v, Num) or len(us) != 1 or us[0][1] != 1:
            raise refuse('RUN-DOMAIN', 'unit of %s: a plain number or a compound unit is a key of no unit kind'
                         % fmt(v))
        if us[0][0] not in CURRENCIES and us[0][0] not in self.m.unit_kind_keys:
            raise refuse('RUN-DOMAIN', 'unit of %s: %s is a quantity unit, a key of no unit kind' % (fmt(v), us[0][0]))
        return Thing(us[0][0])
    p = self.place_arg(n.args[0], env, ctx)
    if isinstance(p, Hole):
        return p
    u = self.m.place_unit(self.m.things[p.name])
    if u in (None, 'any'):
        # a place of unit any has no one unit (L §5.10 Kind and unit, round 7c; §8.4)
        raise refuse('RUN-DOMAIN', 'unit(%s) of a place of unit any: it has no one unit' % p.name)
    if u not in CURRENCIES and u not in self.m.unit_kind_keys:
        # L §5.10 (round 12): "a quantity unit (unit hour …, §5.4) is a key of no unit kind, so unit(x) of either place
        # is refused (§8.4)"
        raise refuse('RUN-DOMAIN', 'unit(%s): its unit %s is a quantity unit, a key of no unit kind' % (p.name, u))
    return Thing(u)


def m_last_of(self, info, keys, d, env, ctx):
    """last(p[k], d) (L §4.6), for value(...) by last(p) and priced last(p) (L §5.10): last's own carry, its held
    mark with it (tally 2.1.7 R6, #123); NONE where no row began, the price's hole (HOLE-PRICE) at its reader."""
    return self.last_value(info, keys, d, ctx)
