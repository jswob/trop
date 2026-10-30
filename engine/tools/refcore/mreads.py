"""Writer 3's reads (L §5.10, §5.14, §5.14.1, §5.19; E §4.2-§4.3, §6): places and movements as values, their
fields, bindings over event kinds and observed inputs, bal, low, flows, value, next, covered, gap, observed, unit.

Installed as Engine methods (`m_<name>` -> `<name>`).
"""
import datetime
from fractions import Fraction

from . import units as U
from .dates import Period, ONE_DAY
from .errors import TallyError, Refusal, LoadError, Unspecified, NotYet, load_now
from .evaluate import member_owns, member_moves, move_owner
from .model import Span, _kv, key_text, WindowUndecided
from .values import (Num, Money, Int, Hole, Thing, Mv, Iv, Rec, Text, Code, Key, NONE, CURRENCIES, fmt,
                     fmt_hole, vkey)
from .moves import item_in_world, Inst, ObsMv, AdjMv, PadMv, qty, unit_of_value, day_lo, day_hi, merge
from .walk import K_end, K_start, K_before, kday, add_q, St


def install(E):
    for name, fn in list(globals().items()):
        if name.startswith('m_'):
            setattr(E, name[2:], fn)


def refuse(code, msg):
    return Refusal(code, msg)


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
    w = ctx.walk
    if w is None:
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
    while w is not None:
        err = w.bad_legs.get((iid, leg))
        if err is not None:
            return err
        w = w.base
    return None


def m_outer_hit(self):
    """A walk attempt needed a cell still being evaluated outside it: start the journal of what it caches."""
    self._outer_hit = True
    for d in (self.memo, self.run_cache, self.walks):
        if getattr(d, 'journal', 1) is None:
            d.journal = []


def m_guarded_walk(self, ctx, day):
    """The walk a leg read on `day` inside a cell's evaluation needs (E §4.2 step 5). The whole walk, unless building it
    needs a cell still being evaluated outside it (a plan booked after the leg reads the cell that reads the leg, as a
    send each quarter that pays out what the quarter's rents sum to): that is no cycle, since a leg's refusal rests on
    the walk up to its day only (E §4.2: a refused leg isn't booked, and the place's balance is refused from that
    point), and evaluation is demand-driven (E §3.2; L §5.11 RUN-CYCLE is a chain back to a cell still being
    evaluated). The attempt is then taken back, what it cached after the need dropped (MC-D-F1: else the answer depends
    on the order of the asks), and the walk is built up to the leg's day (m_prefix_walk)."""
    from .evaluate import JournalDict
    wk = ('walk',) + self.wkey(self.tctx(ctx))
    outer = frozenset(self.active)
    blocked = self.__dict__.setdefault('_walk_blocked', {}).get(wk)
    if blocked and blocked & outer:
        return self.prefix_walk(ctx, day)
    for name in ('memo', 'run_cache', 'walks'):
        d = getattr(self, name)
        if not isinstance(d, JournalDict):
            setattr(self, name, JournalDict(d))
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
        for d in (self.memo, self.run_cache, self.walks):
            journals.append(d.journal)
            d.journal = None
    if not hit:
        return w
    for d, j in zip((self.memo, self.run_cache, self.walks), journals):
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


def m_mv_field(self, n, env, ctx):
    """`s.cash`, `s.day`, an attribute, `a.adjusts`, `a.change` (L §5.14, §5.14.3)."""
    v = self.ev(n.obj, env, ctx)
    if isinstance(v, Hole):
        return v
    if v is NONE:
        raise refuse('RUN-NONE', 'a movement is none here: a field read of it')
    if not isinstance(v, Mv):
        raise refuse('LOAD-FIELD', '%s is not a field of %s' % (n.name, fmt(v)))
    src = self.mv_record(v, ctx)
    if src is None:
        raise refuse('RUN-RECORD', '%s doesn\'t exist' % v.id)
    what = n._r[1]
    name = n.name
    if isinstance(src, AdjMv) and (what == 'day' or (what == 'attr' and name == 'adjusts')):
        pass    # fixed by the adjustment's identity: V.known, the instance and its leg (E §5 step 4)
    elif what == 'day' and getattr(src, 'day_coll', None) is not None:
        self.cur.merge(src.day_coll)
    elif src.coll is not None:
        self.cur.merge(src.coll)
    if getattr(src, 'err', None) is not None:
        raise src.err
    if what == 'day':
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
            t = self.find_inst(mi, list(f[2]), ctx)
            return t.mv() if t is not None else NONE
        return f
    if what == 'leg':
        for lg in src.legs:
            if lg.name == name:
                if isinstance(lg.amount, TallyError):
                    raise lg.amount
                err = self.leg_refused(src.id, name, ctx, lg.day if isinstance(lg.day, datetime.date) else
                                       src.day)
                if err is not None:
                    raise err
                return lg.rec()
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
            return src.attrs[name]
        h = _body_hole(src)
        if h is not None:
            return h
        raise refuse('LOAD-FIELD', '%s is not a field of %s' % (name, v.id))
    raise NotYet('the field %s of a movement' % name)


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
    hits = [a for a in self.adjust_move(src.move, ctx) if a.inst.id == src.id and a.leg.name == name]
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
            except WindowUndecided as w:
                # L §4.5: an entry known in a window holding the cut: an unknown member (§5.11)
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
        out.sort(key=lambda x: (day_lo(x.day) or datetime.date.max, str(x.order)))
        return out
    key = ('mvs', dom, self.wkey(ctx), ctx.unc, id(ctx.walk) if ctx.walk is not None else None, ctx.wp, owner)
    hit = self.run_cache.get(key)
    if hit is not None:
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
    for mi in self.plan_moves():
        if mi.kind not in sub:
            continue
        for i in self.move_instances(mi, ctx):
            st = self.status(i, ctx)
            if st in ('none', 'modelled'):
                continue
            if st == 'uncertain' and not ctx.unc:
                continue
            out.append(i)
    for o in self.observations(ctx):
        if o.kind in sub:
            out.append(o)
    if 'adjustments' in sub:
        out.extend(self.adjustments(ctx, settled=False))
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
                if r is None or r.exists is False or getattr(r, 'status', None) in ('modelled', 'none'):
                    continue
                if getattr(r, 'status', None) == 'uncertain' and not ctx.unc:
                    continue
                out.append(r)
    if owner is not None:
        from .joint import owns
        out = [x for x in out if owns(owner, move_owner(x))]
    out = once_by_same(m, out, owner)
    out.sort(key=lambda x: (day_lo(x.day) or datetime.date.max, str(x.order)))
    self.run_cache[key] = out
    return out


def once_by_same(m, out, owner=None):
    """Two movements stated same are one: "a movement listed twice counts once" (L §6.1), so a binding over them
    binds the left one of the same line. Inside a member (owner), only the same lines of its own versions: a member
    is closed, so the joint model's lines change none of its answers (L §6.1)."""
    sm = getattr(m, 'same_moves', None)
    if sm and owner is not None:
        from .joint import owns
        ows = getattr(m, 'same_owners', None) or [''] * len(sm)
        sm = [x for x, o in zip(sm, ows) if owns(owner, o)]
    if not sm:
        return out
    ids = set(getattr(x, 'id', None) for x in out)
    # same is an equivalence (L §6.1, round 14): a group counts once, by its first member written
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
    for x in order:
        if x not in ids:
            continue
        r = find(x)
        if r in kept:
            drop.add(x)
        else:
            kept.add(r)
    return [x for x in out if getattr(x, 'id', None) not in drop]


def m_same_kind_hole(self, x, kind, ctx):
    """L §6.1: "One movement has one base kind (P-2), so rows that state different kinds disagree: a binding over
    either kind that meets the movement is a HOLE-SAME hole naming both rows" (OQ-2).  A movement of a same group
    whose members state kinds not all inside the binding's kind is an unknown member of it; where every kind the
    group states is inside the binding's kind, the movement is a member either way (round 17: "A binding over a kind
    that holds both kinds meets the one movement once, whatever its kind")."""
    from .joint import shown
    m = self.m
    sm = getattr(m, 'same_moves', None)
    if not sm or not hasattr(x, 'id'):
        return None
    groups = getattr(m, '_same_groups', None)
    if groups is None:
        parent = {}

        def find(a):
            while parent.get(a, a) != a:
                a = parent[a]
            return a
        order = []
        for a, b, loc in sm:
            for y in (a, b):
                if y not in order:
                    order.append(y)
            ra, rb = find(a), find(b)
            if ra != rb:
                parent[rb] = ra
        groups = {}
        for y in order:
            groups.setdefault(find(y), []).append(y)
        groups = dict((y, g) for g in groups.values() for y in g)
        m._same_groups = groups
    g = groups.get(x.id)
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
    a, b = mem[0], next(o for o in mem[1:] if getattr(o, 'kind', None) != mem[0].kind)
    d = x.day if isinstance(x.day, datetime.date) else (a.day if isinstance(a.day, datetime.date) else b.day)
    q = '%s and %s are one, but the kind differs on %s: %s (%s) and %s (%s)' % (
        shown(a.id), shown(b.id), d if isinstance(d, datetime.date) else 'every day', a.kind, shown(a.id), b.kind,
        shown(b.id))
    return Hole([(q, '%s, %s' % (shown(a.id), shown(b.id)))])


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
        sp = Span(d, d + ONE_DAY)
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
                (sp.end is None or sp.end > t + ONE_DAY):
            raise refuse('RUN-FUTURE', 'a decision on %s binds over %s through %s, after its day' % (
                t, b._dom[1], (sp.end - ONE_DAY) if sp.end is not None else 'the end'))
    out = []
    fulfilled = set()
    for x in self.movements_of(b._dom, ctx, owner):
        d = x.day
        st = True
        if d is None and x.err is not None:
            # refused before its day was known: whether it is a member is refused with it (L §8.4)
            raise x.err
        if isinstance(d, Hole):
            st = d
        elif isinstance(d, Iv) and isinstance(d.lo, datetime.date):
            inside = sp.contains(d.lo) and sp.contains(d.hi)
            outside = (sp.end is not None and d.lo >= sp.end) or (sp.start is not None and d.hi < sp.start)
            if outside:
                continue
            if not inside:
                st = Hole([('date within %s not precise enough for a binding over %s' % (d.window_text(),
                                                                                         b._dom[1]), x.id)])
        elif isinstance(d, datetime.date):
            if not sp.contains(d):
                continue
        else:
            continue
        if isinstance(x, Inst) and isinstance(x.exists, Hole):
            st = x.exists if st is True else merge(st, x.exists)
        if isinstance(x, Inst) and x.err is not None:
            raise x.err
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
            if isinstance(x, Inst) and getattr(x, 'status', None) == 'uncertain':
                self.reg(('uncertain', x.id))
            if x.coll is not None:
                self.cur.merge(x.coll)
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
        raise refuse('RUN-NONE', '%s is none here: a place' % fmt_node(node))
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


def m_read_state(self, w, place, unit, key):
    """The balance of a place at an instant of a walk, with its register (L §8.2)."""
    st = w.pstate(place, unit, key, raw=True)
    self.merge_cum(w, place, unit, key)
    d = kday(key)
    for (ad, e) in w.adj_regs.get(place, ()):
        if ad <= d:
            self.reg(e)
    ww = w
    while ww is not None:
        for mid in sorted(getattr(ww, 'leftout_by_place', {}).get(place, ())):
            n, first = ww.leftout_regs[mid]
            self.reg(('left out', '%s, %d instances, first %s' % (mid, n, first)))
        ww = ww.base
    self.joint_left_out(w, place, key)
    v = w.st_value(st, place, unit)
    if isinstance(v, TallyError):
        raise v
    return v


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
    if isinstance(d, Iv) and isinstance(d.lo, datetime.date):
        d = self.window_choice(d, ctx)
        if isinstance(d, Iv):
            raise self.imprecise(d, 'the day of bal(%s, d)' % place)
    d = self.date_of(d, 'the day of bal')
    if form == 'proj':
        return self.proj_bal(n, place, unit, d, env, ctx)
    if form == 'modelled':
        return self.bal_modelled(place, unit, d, ctx)
    key = K_start(d) if n.start else K_end(d)
    if n.proj and 'windows' in n.proj:
        ctx = ctx.but(windows=n.proj['windows'])
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
    for lg in legs:
        o = lg.day.toordinal()
        if not (key[0] > o or (key[0] == o and key[1] > 1)):
            continue
        if not isinstance(lg.amount, Num):
            continue
        if unit not in (None, '*') and lg.unit not in (None, '*') and lg.unit != unit:
            continue
        if isinstance(lg.frm, Thing) and lg.frm.name == place:
            delta += lg.amount.q
        if isinstance(lg.to, Thing) and lg.to.name == place:
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
                         % (ctx.wp[3].split('|')[-1 if ctx.wp[3].count('|') > 1 else 0], place, d))
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
    from .evaluate import lin_of, lin_add, lin_scale
    st, lg, i = w.state(place, unit, key)
    if st.bad is not None or st.val is None or st.perm or st.imp or lg is None:
        return v
    if st.rst < 0 and (w.base is not None or w.default(place, unit).hole):
        return v
    acc = lin_of(st.val)
    seen = frozenset()
    for e in lg.ents[st.rst + 1:i + 1]:
        if e.kind != 'hole':
            continue
        lgx = e.leg
        if e.sgn is None or lgx is None or lgx.src is None:
            return v
        node, env, c = lgx.src
        h = self.ev(node, env, c.but(partial=True))
        lh = lin_of(h) if isinstance(h, Hole) else None
        if lh is None:
            return v
        acc = lin_add(acc, lin_scale(lh, Num(e.sgn, (), isint=True)), 1)
        seen |= e.qs
    if not acc[1] or set(seen) != set(st.hole):
        return v
    return Hole(v.qs, acc)


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
                        hs.append(g.amount)
                    elif isinstance(g.amount, TallyError):
                        raise g.amount
                    else:
                        add += s * g.amount.q
    if isinstance(v, Hole) or hs:
        return merge(v, *hs)
    return qty(v.q + add, unit) if v.money or v.u else Num(v.q + add, v.u)


def m_proj_clauses(self, proj, ctx):
    windows = proj.get('windows', ctx.windows)
    if windows == 'each':
        # windows each on the read (L §4.4, "for that read"): the line's runner's assignment applies inside it
        lw = ctx.windows if isinstance(ctx.windows, tuple) else getattr(ctx.cut, 'windows', None)
        if isinstance(lw, tuple) and lw:
            windows = ('each', lw[1])
    rnd = ctx.rnd
    if proj.get('random') is not None:
        rnd = self.rnd_clause(proj['random'], ctx)
    unc = proj.get('uncertain', ctx.unc)
    return windows, rnd, unc


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
    self.proj_regs(w, t, d)
    if clauses[2] and not isinstance(v, Hole):
        v = self.proj_uncertain(w, place, unit, d, v)
    return v


def m_proj_regs(self, w, t, d):
    for mid in sorted(w.counted_leftout):
        days = [x for x in w.counted_leftout[mid] if t <= x <= d]
        if days:
            self.reg(('left out', '%s, %d instances, first %s' % (mid, len(days), min(days))))


def m_proj_uncertain(self, w, place, unit, d, v):
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
                        hs.append(g.amount)
                    elif isinstance(g.amount, TallyError):
                        raise g.amount
                    else:
                        add += s * g.amount.q
    if hs or isinstance(v, Hole):
        return merge(v, *hs)
    return qty(v.q + add, unit)


def m_bal_modelled(self, place, unit, d, ctx):
    """`bal(x, d) modelled` (L §5.19, E §6.2): "from x's last reading before d" (round 18, C2-13: E said "at or
    before d"), "the opening, before the first", plus every booked movement of x since through d, the modelled-past
    instances in place of the observations on covered days; "Comparing it with the reading is the residual", so a
    reading on day d itself is not its start, and the gaps of readings on d are left out (they are not movements)."""
    w = self.main_walk(ctx)
    key = K_end(d)
    st, lg, i = w.state(place, unit, key)
    if lg is not None:
        # the last reading dated before day d; else an opening on day d (the opening, before the first reading)
        st0, _, i0 = w.state(place, unit, (d.toordinal(), -2, 0, '', 0))
        r0 = st0.rst if i0 >= 0 else -1
        if r0 < 0:
            r0 = next((j for j in range(i, -1, -1) if lg.ents[j].kind == 'reset' and
                       lg.ents[j].note == 'opening' and lg.ents[j].key[0] == d.toordinal()), -1)
        st = St(st.val, st.hole, st.perm, st.imp, st.bad, r0)
    if lg is None or st.rst < 0:
        start = None
        v = w.default(place, unit).val
        lo = -1
    else:
        e = lg.ents[st.rst]
        v = e.val
        lo = st.rst
        if e.qs:
            return Hole(e.qs)
    if v is None:
        return Hole([w.anchor_question(place)])
    q = v.q
    hs = []
    for j in range(lo + 1, (i if lg is not None else -1) + 1):
        e = lg.ents[j]
        if e.kind == 'bad':
            raise e.err
        if isinstance(e.mv, ObsMv) or e.kind == 'reset':
            # a reading on day d is what the modelled balance is compared with, not where it starts (C2-13)
            continue
        if e.kind == 'delta':
            q += e.flow.q
        elif e.kind in ('hole', 'perm'):
            hs.append(Hole(e.qs))
        if e.coll is not None:
            self.cur.merge(e.coll)
    since = lg.ents[lo].key if (lg is not None and lo >= 0) else None
    for x in w.modelled:
        for g in x.legs:
            gd = g.day if isinstance(g.day, datetime.date) else day_lo(x.day)
            if gd is None or gd > d or (since is not None and K_end(gd) <= since):
                continue
            for p, s in ((g.frm, -1), (g.to, 1)):
                if isinstance(p, Thing) and p.name == place:
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


def m_e_low(self, n, env, ctx):
    """low(x, a .. b) [from t]: the lowest end-of-day balance (L §5.14.1)."""
    p = self.place_arg(n.ref, env, ctx)
    if isinstance(p, Hole):
        return p
    place = p.name
    unit = self.bal_unit(n, place, env, ctx)
    sp = self.span_value(n.span, env, ctx)
    if isinstance(sp, Hole):
        return sp
    a, b = sp.start, sp.end - ONE_DAY
    if n._form == 'proj':
        t = self.ev(n.proj['frm'], env, ctx)
        if isinstance(t, Hole):
            return t
        t = self.date_of(t, 'the day of a projection')
        if a < t:
            raise refuse('RUN-DOMAIN', 'low(%s, %s .. %s) from %s: a day before the projection\'s start' % (
                place, a, b, t))
        if t < self.m.S:
            # L §5.14.1 (round 12): HOLE-PRERUN
            return Hole([('before the run\'s start', place)])
        clauses = self.proj_clauses(n.proj, ctx)
        w = self.proj_walk(ctx, t, n._leftout, clauses)
        if ctx.walk is not None:
            note_read(ctx.walk, place, True)
        w.run_to(b)
        self.proj_regs(w, t, b)

        def get(dd):
            # L §5.14.1: low(x, span) from t is "min(bal(x, d) from t for d in days where ...)", "with the same
            # clauses": with uncertain counts the uncertain instances as bal(x, d) from t does (fault f_low)
            v = self.read_state(w, place, unit, K_end(dd))
            if clauses[2] and not isinstance(v, Hole):
                v = self.proj_uncertain(w, place, unit, dd, v)
            return v
    else:
        if n.proj and 'windows' in n.proj:
            ctx = ctx.but(windows=n.proj['windows'])
        get = lambda dd: self.bal_day(place, unit, dd, K_end(dd), ctx)
    vals = []
    d = a
    while d <= b:
        vals.append(get(d))
        d += ONE_DAY
    hs = [v for v in vals if isinstance(v, Hole)]
    if hs:
        return merge(*hs)
    best = vals[0]
    for v in vals[1:]:
        if v.q < best.q:
            best = v
    return best


# -- flows (L §5.14.3) --------------------------------------------------------------------------------------------

def m_e_flows(self, n, env, ctx):
    sp = self.span_value(n.span, env, ctx)
    if isinstance(sp, Hole):
        return sp
    if sp.start is None or sp.start < self.m.S:
        return Hole([('before the run\'s start', fmt_node(n.ref))])
    fk = None
    if n.forkey is not None:
        fk = self.ev(n.forkey, env, ctx)
        if isinstance(fk, Hole):
            return fk
    conv = None
    if n.conv is not None:
        cur, fx, mode = n.conv
        if getattr(mode, '_r', None) and mode._r[0] == 'axismode':
            mv = Code('rounding', self.axis_option(mode._r[1], ctx))
        else:
            mv = self.ev(mode, env, ctx)
        conv = (cur, fx, mv)
    if n._form == 'move':
        return self.flows_move(n, sp, fk, conv, env, ctx)
    p = self.place_arg(n.ref, env, ctx)
    if isinstance(p, Hole):
        return p
    return self.flows_place(p.name, sp, conv, env, ctx, fk=fk)


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


def m_flows_place(self, place, sp, conv, env, ctx, fk=None):
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
    units = sorted(set(u for (p, u) in w.logs if p == place and u != '*'))
    pu = m.place_unit(m.things[place])
    if pu not in (None, 'any'):
        units = [pu]
    totals = {}
    holes = []
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
            if e.kind == 'bad' and (e.key > lo or e.key == k0):
                raise e.err
            if e.kind in ('perm', 'hole') and e.qs:
                holes.append(Hole(e.qs))
                if e.coll is not None:
                    self.cur.merge(e.coll)
    for u in units + ['*']:
        lg = w.logs.get((place, u))
        if lg is None:
            continue
        wins = {}
        for e in lg.ents:
            if e.key <= lo:
                if e.kind == 'perm' and e.key <= limit:
                    holes.append(Hole(e.qs))
                if e.kind == 'imp_on':
                    wins[e.wid] = e.win
                continue
            if e.key > limit:
                if e.kind == 'delta' and e.wid is not None and e.wid in wins:
                    holes.append(Hole([('date within %s not precise enough for flows of %s' % (
                        e.win.window_text(), place), e.mv.id)]))
                continue
            if e.kind == 'bad':
                raise e.err
            if e.kind == 'imp_on':
                wins[e.wid] = e.win
                continue
            if other_for(e) or (fk is not None and e.kind == 'reset'):
                continue
            if e.kind in ('hole', 'perm'):
                holes.append(Hole(e.qs))
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
                    holes.append(Hole([('date within %s not precise enough for flows of %s' % (
                        win.window_text(), place), e.mv.id)]))
                    continue
            for r in e.reg:
                self.reg(r)
            if e.coll is not None:
                self.cur.merge(e.coll)
            if isinstance(f, Hole):
                holes.append(f)
                continue
            if conv is not None:
                f = self.convert(f, kday(e.key), conv, env, ctx)
                if isinstance(f, Hole):
                    holes.append(f)
                    continue
            uu = conv[0] if conv is not None else u
            totals[uu] = totals.get(uu, Fraction(0)) + f.q
    for (ad, e) in w.adj_regs.get(place, ()):
        if ad <= last:
            self.reg(e)
    # the joint model's own unlinked movements into the place on or before the span's last day (L §6.1, round 17b)
    self.joint_left_out(w, place, limit)
    if holes:
        return merge(*holes)
    if conv is not None:
        return Money(totals.get(conv[0], 0), conv[0])
    # L §5.14.3 (round 12): "Its unit is that of the places it reads ...; a place of unit any, movements in two
    # units, or an empty span whose unit load can't see is refused (LOAD-UNIT where load sees it, RUN-DOMAIN
    # otherwise), unless the flows converts"
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


def m_convert(self, f, day, conv, env, ctx):
    """One movement converted (L §5.14.3): the rate at its day, its amount (>= 0) rounded to the minor unit with
    the mode, then its direction."""
    from .evaluate import round_mode
    cur, fx, mode = conv
    u = unit_of_value(f)
    if u == cur:
        return f
    rate = self.ev(fx, env, ctx.but(law=day))
    if isinstance(rate, Hole):
        return rate
    if isinstance(mode, Hole):
        return mode
    if not isinstance(rate, Num):
        raise refuse('LOAD-UNIT', 'flows in %s at %s: the rate is %s' % (cur, fmt_node(fx), fmt(rate)))
    want = tuple(sorted([(cur, 1), (u, -1)]))
    if tuple(sorted(rate.u)) != want:
        raise refuse('LOAD-UNIT', 'an exchange rate %s applied to %s' % (fmt(rate), u))
    q = abs(f.q) * rate.q
    minor = CURRENCIES[cur]
    k = round_mode(q / minor, mode.word)
    q = k * minor
    if f.q < 0:
        q = -q
    return Money(q, cur)


def m_flows_move(self, n, sp, fk, conv, env, ctx):
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
    else:
        insts = self.move_instances(mi, ctx)
    total = None
    holes = []
    u = None
    seen_units = set()
    for i in insts:
        if keys and tuple(vkey(k) for k in i.keys[:len(keys)]) != tuple(keys):
            continue
        if getattr(i, '_rawhole', False):
            if i.err is not None:
                raise i.err
            holes.append(i.day)
            continue
        st = i.status if mi.decision else self.status(i, ctx)
        if st in ('none', 'uncertain'):
            continue
        d = i.day
        if isinstance(d, Hole):
            holes.append(d)
            continue
        if isinstance(d, Iv) and isinstance(d.lo, datetime.date):
            if sp.contains(d.lo) and d.hi <= last:
                pass
            elif (sp.end is not None and d.lo > last) or d.hi < sp.start:
                continue
            else:
                holes.append(Hole([('date within %s not precise enough for flows of %s' % (d.window_text(), mi.id),
                                    i.id)]))
                continue
        elif not sp.contains(d):
            continue
        if fk is not None and (i.forp is None or vkey(i.forp) != vkey(fk)):
            continue
        if st == 'modelled':
            # on a day a statement covers, the observations that fulfil the instance stand in for it (L §5.14.3)
            obs = getattr(i, 'obs', None) or self.obs_links_for(i.id, ctx)
            if not obs:
                self.reg(('open', i.id))
            for o in obs:
                if o.coll is not None:
                    self.cur.merge(o.coll)
                for lg in o.legs:
                    if leg is not None and lg.name != leg:
                        continue
                    a = lg.amount
                    if isinstance(a, Hole):
                        holes.append(a)
                        continue
                    if conv is not None:
                        a = self.convert(a, lg.day if isinstance(lg.day, datetime.date) else o.day, conv, env, ctx)
                        if isinstance(a, Hole):
                            holes.append(a)
                            continue
                    total = a.q if total is None else total + a.q
                    u = unit_of_value(a)
                    seen_units.add(u)
            continue
        if st == 'refused' or i.err is not None:
            raise i.err
        if i.coll is not None:
            self.cur.merge(i.coll)
        if isinstance(i.exists, Hole):
            holes.append(i.exists)
            continue
        if st == 'past' and not getattr(i, 'obs', None):
            self.reg(('unfulfilled', i.id))
        for lg in i.legs:
            if leg is not None and lg.name != leg:
                continue
            a = lg.amount
            if isinstance(a, TallyError):
                raise a
            if isinstance(a, Hole):
                holes.append(a)
                continue
            err = self.leg_refused(i.id, lg.name, ctx, lg.day if isinstance(lg.day, datetime.date) else i.day) \
                if a.q != 0 else None
            if err is not None:
                raise err
            if conv is not None:
                a = self.convert(a, lg.day if isinstance(lg.day, datetime.date) else day_lo(d), conv, env, ctx)
                if isinstance(a, Hole):
                    holes.append(a)
                    continue
            total = a.q if total is None else total + a.q
            u = unit_of_value(a)
            seen_units.add(u)
    # settlements dated in the span, and HOLE-CHANGED (E §5 steps 5-6): only this move's adjustments are measured
    # unless it has some, whose settlements need the whole pass (E §4.1)
    own = self.adjust_move(mi, ctx) if not mi.decision else []
    if own:
        self.settle(own, ctx)
        for a in own:
            if a.inst.move.id != mi.id:
                continue
            if leg is not None and a.leg.name != leg:
                continue
            if keys and tuple(vkey(k) for k in a.inst.keys[:len(keys)]) != tuple(keys):
                continue
            if not a.settled and last >= a.day:
                holes.append(Hole([('%s on %s changed after it was paid (version %s); is back pay due, and when?' % (
                    a.inst.id, a.inst.day, ', '.join(a.vids)), a.inst.id)]))
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
    if holes:
        return merge(*holes)
    if conv is not None:
        return Money(total or 0, conv[0])
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
            return v
        amt = v.q
        if priced._cur != cur:
            if getattr(n, '_price', None) is None:
                raise refuse('LOAD-PRICE', 'value(%s) in %s holds units in %s and states no by' % (place, cur,
                                                                                                 priced._cur))
            pr = self.price_read(n._price, [Thing(priced._cur)], d, n, env, ctx, place, priced._cur)
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
    total = Fraction(0)
    holes = []
    ofq = None
    if getattr(n, 'of', None) is not None:
        # tally 2.1 (L §5.10): "value(x, d) in C [by p] of q is those parts at their prices"
        ofq = self.of_value(n.of, env, ctx)
        if isinstance(ofq, Hole):
            return ofq
    for u in units:
        q = self.bal_day(place, u, d, K_end(d), ctx) if ofq is None else self.bal_of_day(place, u, d, K_end(d), ctx,
                                                                                         ofq)
        if isinstance(q, Hole):
            holes.append(q)
            continue
        if u == cur:
            total += q.q
            continue
        if q.q == 0:
            continue
        if getattr(n, '_price_expr', False):
            # a price written as an expression with its keys, read at the value's day (L §5.10); `by last(ref)` with
            # keys is last(ref, d) (writer 12, A1: check3.m_t_value's _by_expr)
            pr = self.ev(getattr(n, '_by_expr', None) or n.by, env, ctx.but(law=d))
            if isinstance(pr, Hole):
                holes.append(pr)
                continue
            if pr is NONE:
                raise refuse('RUN-NONE', 'the price of %s in %s on %s is none here' % (u, cur, d))
            want = tuple(sorted([(cur, 1), (u, -1)]))
            if not isinstance(pr, Num) or tuple(sorted(pr.u)) != want:
                raise refuse('RUN-DOMAIN', 'value(%s) in %s: the price %s is not in %s/%s' % (place, cur, fmt(pr), cur,
                                                                                            u))
            total += q.q * pr.q
            continue
        if getattr(n, '_price', None):
            pr = self.price_read(n._price, [Thing(u)], d, n, env, ctx, place, u)
            if getattr(n, '_price_unkeyed', False) and isinstance(pr, Num):
                # an unkeyed price read through its carry is one unit's price: the position's unit must be it, as for a
                # price written as an expression (L §5.10; writer 11)
                want = tuple(sorted([(cur, 1), (u, -1)]))
                if tuple(sorted(pr.u)) != want:
                    raise refuse('RUN-DOMAIN', 'value(%s) in %s: the price %s is not in %s/%s' % (
                        place, cur, fmt(pr), cur, u))
        elif priced is not None:
            if priced._cur != cur or priced._per != u:
                raise refuse('LOAD-UNIT', 'value(%s) in %s: %s is priced in %s/%s, not %s/%s' % (
                    place, cur, place, priced._cur, priced._per, cur, u))
            pr = self.priced_read(priced, th, u, d, n, env, ctx)
        else:
            raise refuse('LOAD-PRICE', 'value(%s) in %s holds units in %s and states no by' % (place, cur, u))
        if isinstance(pr, Hole):
            holes.append(pr)
            continue
        total += q.q * pr.q
    if holes:
        return merge(*holes)
    return Num(total, ((cur, 1),))


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


# -- next, covered, gap, observed, unit (L §5.14.1, §5.19, §5.10) -------------------------------------------------------

def m_c_next(self, n, env, ctx):
    """next(K, d) is `first(m in K over d ..) ties by m.day` (L §5.14.1 table): the first movement of event kind K
    dated on or after d; two on that first day are a tie (RUN-TIE); none at the cut is HOLE-NOTYET (L §3.1), since
    the span `d ..` reaches past every cut."""
    kid = n._kind
    d = self.ev(n.args[1], env, ctx)
    if isinstance(d, Hole):
        return d
    d = self.date_of(d, 'the day of next')
    res, colls = self.span_members(('event', kid), Span(d, None), ctx)
    what = 'next(%s, %s)' % (kid, d)
    best = self.first_by_day(res, what)
    if isinstance(best, Hole):
        return best
    if best is None:
        return notyet_hole('m', kid)
    v, x = best
    c = colls.get(x.id)
    if c is not None:
        self.cur.merge(c)
    return v


def notyet_hole(var, dom):
    """L §8.1 HOLE-NOTYET: `<x>: no <event> yet, and its span reaches past the cut`, naming the binding."""
    return Hole([('%s: no %s yet, and its span reaches past the cut' % (var, dom), dom)])


def m_first_by_day(self, res, what):
    """The member first by day among [(Mv, status, movement)] (ties by m.day, L §5.11): (Mv, movement), None when
    there is none, a Hole when an unknown member or a window could come first, RUN-TIE when two share the first day."""
    if not res:
        return None
    def lo(x):
        return day_lo(x.day)

    def hi(x):
        return day_hi(x.day)
    known = [(v, s, x) for v, s, x in res if not isinstance(x.day, Hole)]
    holes = [s for v, s, x in res if isinstance(x.day, Hole)]
    if holes:
        # a movement whose day is a hole may come first (L §5.14)
        return merge(*holes)
    top = min(hi(x) for v, s, x in known)
    cands = [(v, s, x) for v, s, x in known if lo(x) <= top]
    hs = [s for v, s, x in cands if isinstance(s, Hole)]
    if hs:
        return merge(*hs)
    if len(cands) == 1:
        v, s, x = cands[0]
        return (v, x)
    wins = [x for v, s, x in cands if isinstance(x.day, Iv)]
    if wins:
        # their order is a comparison of windows, undecided (L §4.4)
        w = wins[0]
        return Hole([('date within %s not precise enough for %s' % (w.day.window_text(), what), w.id)])
    raise refuse('RUN-TIE', "%s has a tie at %s that changes the result, and its ties by doesn't break it" % (
        what, ', '.join(x.id for v, s, x in cands)))


def m_c_covered(self, n, env, ctx):
    """covered(x, span): versions known by the cut cover x on every day of the span; never a hole (L §5.19)."""
    p = self.ev(n.args[0], env, ctx)
    if isinstance(p, Hole) or p is NONE:
        return False
    sp = self.span_value(n.args[1], env, ctx)
    if isinstance(sp, Hole):
        return False
    if sp.start is None or sp.end is None:
        return False
    spans = sorted((s for s, v in self.coverage(ctx).get(p.name, ())), key=lambda s: s.start or datetime.date.min)
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
    obs = self.obs_links_for(v.id, ctx)
    bad = self.run_cache.get(('badlinks', self.wkey(ctx)), {}).get(v.id)
    if bad is not None:
        raise bad
    if not obs:
        return NONE
    mi = m.moves[a._r[1]]
    legs = []
    for ld in m.kind_legs(mi.kind):
        tot = None
        frm = to = None
        day = None
        for o in obs:
            self.cur.rows[o.row.sid] = o.row
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
            raise refuse('RUN-NONE', '%s is none here: a quantity' % fmt_node(n.args[0]))
        us = list(getattr(v, 'u', None) or ())
        if not isinstance(v, Num) or len(us) != 1 or us[0][1] != 1 or \
                (us[0][0] not in CURRENCIES and us[0][0] not in self.m.unit_kind_keys):
            raise refuse('RUN-DOMAIN', 'unit of %s: a key of no unit kind' % fmt(v))
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
    """last(p[k], d) (L §4.6), for value(...) by last(p) (L §5.10)."""
    from .evaluate import HoleSignal, NOPIN
    pv = self.pin_value(info, keys, ctx)
    if pv is not NOPIN and not isinstance(pv, tuple):
        return pv
    org = self.origin(info, keys)
    try:
        cands = self.candidates(info.rows, keys, d, ctx, info, last_before=d)
    except HoleSignal as h:
        return h.hole
    if not cands:
        return NONE
    if info.reading:
        latest = max(c.instant[0] for c in cands)
        cands = [c for c in cands if c.instant[0] == latest]
    else:
        latest = max(c.span(ctx.cut).start for c in cands)
        cands = [c for c in cands if c.span(ctx.cut).start == latest]
    row = self.choose(cands, info, 'last')
    v = self.row_value(row, info, keys, ctx, None, None)
    return v
