"""Answering asks (L §5.18, §5.5 fan-out, §8.2 register; E §10).

The printed form.  The spec fixes how values, holes, refusals and register
entries print (L §5.18 Output, §8.1, §8.2) but not the layout of a line; this
interpreter prints, for each line:

    <ask id> (<var> = <key>, ...) [<axis> = <options>, ...]: <result>
      <register entry>
      ...

the binding part only when the ask has a `for`, the axis part only when the
ask reaches an axis.  The register entries follow the order of the table in
L §8.2, each kind sorted by code point, `law date <d>, cut <K>` last.
"""
import datetime
import re

from .errors import TallyError, Unspecified, NotYet, Refusal
from .evaluate import Engine, Ctx, Coll, TraceNode, HoleSignal, OutKey, ListKey, merge, lin_of
from .model import key_text, set_world, set_each
from .values import (CURRENCIES, fmt_typed, Num, Hole, Iv, Special, Thing, Key, Lst, Text, NONE, vkey, fmt, fmt_hole, fmt_q)
from .check import _txt
from .eval2 import QOut


REG_ORDER = ['assumed', 'given', 'missing', 'pinned', 'pinned sample', 'sample', 'world', 'reading', 'anchor', 'pad',
             'adjustment', 'unfulfilled', 'open', 'left out', 'uncertain', 'held']
MAX_PATHS = 100000   # E §7: N <= 100 000 (REQ-PATHS)


class LoadAtRun(Exception):
    """A load refusal (L §8.3) that this interpreter finds only while evaluating (a value's unit or
    period known only from the rows).  L §8.3: the model does not run, so it stops every ask and prints
    as a load refusal, never per value."""

    def __init__(self, err):
        Exception.__init__(self, err)
        self.err = err


_PIN_DAY = re.compile(r'^(pinned: (?:known|law) .+) = (\d{4}-\d{2}-\d{2})$')


def merge_pin_days(pins):
    """L §8.2 `pinned:`: "The pins one def forms under many as if (a known d at each of its keys, as cpi_yy[m] as if
    {known d}) print once, their days merged into maximal spans". An as if pin given by an expression prints
    `pinned: known <expr> = <day>`; those of one written pin print once, `= <a> .. <b>, <c>` (refcore's layout)."""
    groups = {}
    rest = set()
    for t in pins:
        mt = _PIN_DAY.match(t)
        if mt is None:
            rest.add(t)
            continue
        groups.setdefault(mt.group(1), set()).add(datetime.date.fromisoformat(mt.group(2)))
    for head, days in groups.items():
        if len(days) == 1:
            rest.add('%s = %s' % (head, next(iter(days))))
            continue
        spans = []
        for d in sorted(days):
            if spans and spans[-1][1] + datetime.timedelta(days=1) == d:
                spans[-1][1] = d
            else:
                spans.append([d, d])
        rest.add('%s = %s' % (head, ', '.join(('%s .. %s' % (a, b)) if a != b else str(a) for a, b in spans)))
    return rest


def is_load(err):
    return isinstance(err, TallyError) and err.code.startswith('LOAD-')


def member_names_world(mm, a):
    """Whether a member's base ask names one of the member's scenarios: its `in`, a diff's worlds, a scenario pin, an
    `as if {scenario s}` in its line or a def it reaches."""
    ws = getattr(mm, 'member_worlds', None)
    if not ws:
        return False
    need = set()
    if a.world is not None:
        need.add(a.world)
    if a.form == 'diff':
        need |= set(w for w in a.diff[2:] if w is not None)
    for p in a.pins:
        if p.kind == 'scenario':
            need.add(p.id)
    from .cli import asif_worlds
    need |= set(asif_worlds(mm, a))
    return bool(need & set(ws))


class Result:
    """One evaluation of an ask line under one axis combination."""
    __slots__ = ('vals', 'coll', 'trace')

    def __init__(self, vals, coll, trace=None):
        self.vals = vals
        self.coll = coll
        self.trace = trace

    def key(self):
        return tuple(rkey(v) for v in self.vals)


def rkey(v):
    if isinstance(v, TallyError):
        return ('refused', v.code, str(v))
    if isinstance(v, CheckOut):
        return ('check', v.key())
    if isinstance(v, RowSet):
        return ('rows', tuple(sorted(v.rows)))
    if isinstance(v, QOut):
        return v.key()
    if isinstance(v, Hole):
        return ('hole', v.qs, lin_key(v.lin))
    return vkey(v)


def lin_key(l):
    if not l:
        return None
    k, ts = l
    return (vkey(k) if k is not None else None, tuple((a, vkey(c)) for a, c in ts))


class CheckOut:
    def __init__(self, text, kind):
        self.text = text
        self.kind = kind

    def key(self):
        return (self.kind, self.text)


class RowSet:
    def __init__(self, rows, texts):
        self.rows = rows
        self.texts = texts


def fmt_value(v, partial=False, t=None):
    if isinstance(v, TallyError):
        return 'refused: %s %s' % (v.code, v.msg)
    if isinstance(v, Hole):
        if partial and v.lin:
            return fmt_partial(v)
        return fmt_hole(v)
    if isinstance(v, Iv):
        if isinstance(v.lo, datetime.date):
            return fmt(v)
        return fmt_hole(Hole([('date within %s not precise enough for a print' % v.window_text(),
                               v.origin[1] if v.origin else '')]))
    if isinstance(v, (OutKey,)):
        return 'refused: RUN-KEY %s is outside %s' % (v.text, v.index)
    if isinstance(v, ListKey):
        return v.text
    if isinstance(v, (CheckOut, QOut)):
        return v.text
    if isinstance(v, tuple) and v and v[0] == 'mmdd':
        return '%02d-%02d' % v[1]
    return fmt_typed(v, t)


def fmt_bind(b, v):
    """A binding's value in a line's head: a key of a key list prints as written (L §2.5), an integer key too."""
    dom = getattr(b, '_dom', None)
    if dom is not None and dom[0] == 'list' and isinstance(v, Num) and v.isint:
        return str(int(v.q))
    return fmt_value(v)


def fmt_partial(h):
    """412 300.00 PLN + 1 × ?"value of the firm share" (L §5.18 Partial answers)."""
    k, ts = h.lin
    parts = []
    if k is not None:
        parts.append(fmt(k))
    for (text, origin), c in ts:
        q = '?"%s"' % text.replace('\\', '\\\\').replace('"', '\\"')
        if origin:
            q += ' (%s)' % origin
        parts.append('%s × %s' % (fmt(c), q))
    return ' + '.join(parts)


def lin_day(d):
    from .moves import day_lo
    return day_lo(d)


class EachRes:
    """One axis combination's windows-each lines, as a result the axis grouping compares (L §5.5)."""

    def __init__(self, rows):
        self.trace = rows
        self.vals = None
        self.coll = Coll()
        for suffix, ln in rows:
            self.coll.merge(ln.coll)

    def key(self):
        return tuple((suffix, tuple(rkey(v) for v in ln.vals)) for suffix, ln in self.trace)


class ExplainLines:
    """An explanation given as text lines (a movement instance, E §10)."""

    def __init__(self, lines):
        self.lines = lines


class Line:
    def __init__(self, env_text, axes_text, vals, coll, reading, trace):
        self.env_text = env_text
        self.axes_text = axes_text
        self.vals = vals
        self.coll = coll
        self.reading = reading
        self.trace = trace


class Asker:
    def __init__(self, m):
        self.m = m
        self.walk_warnings = []

    def run(self, ids=None):
        m = self.m
        asks = {a.id: a for a in list(m.asks) + list(getattr(m, 'asks_alone', ()))}
        members = getattr(m, 'joint_members', {})
        if ids:
            order = []
            for i in ids:
                if '.' in i and i.split('.', 1)[0] in members:
                    # a member's ask, asked inside the joint model as a.q (L §6.1): answered by the member alone
                    order.append(i)
                    continue
                if i not in asks:
                    raise TallyError('LOAD-NAME', 'no ask %s' % i)
                order.append(asks[i])
        else:
            order = [asks[i] for i in sorted(asks)]
        out = []
        try:
            for a in order:
                if isinstance(a, str):
                    out.extend(self.member_ask(a))
                    continue
                out.extend(self.answer(a))
        except LoadAtRun as x:
            return ['refused: %s' % x.err.text()]
        for code, msg in m.warnings:
            out.append('warning: %s %s' % (code, msg))
        for code, msg in self.walk_warnings:
            out.append('warning: %s %s' % (code, msg))
        return out

    def member_alone(self, qid, alias, aid, mm):
        """The member alone, all its files loaded (its scenarios included), answers its ask aid (L §6.1)."""
        from .cli import run_texts
        out = []
        for l in run_texts(mm.member_all, [aid], mm.member_options):
            if l.startswith('refused: '):
                out.append('%s: %s' % (qid, l))
            elif l.startswith('scenario '):
                # its scenario's load refused (L §8: that scenario only), named under the member's prefix
                out.append('scenario %s.%s' % (alias, l[len('scenario '):]))
            elif not l.startswith((' ', 'warning')):
                out.append(alias + '.' + l)
            else:
                out.append(l)
        return out

    def member_ask(self, qid):
        alias, aid = qid.split('.', 1)
        mm = self.m.joint_members[alias]
        hit = [b for b in mm.asks if b.id == aid]
        sub = self.__dict__.setdefault('_member_askers', {})
        if not hit and '.' in aid and aid.split('.', 1)[0] in (getattr(mm, 'joint_members', None) or {}):
            # a member that is a joint model asks its members' asks as b.q, so they are asked here as a.b.q (L §6.1)
            if alias not in sub:
                sub[alias] = Asker(mm)
            return [alias + '.' + l if not l.startswith((' ', 'warning')) else l for l in sub[alias].member_ask(aid)]
        if not hit:
            if aid in getattr(mm, 'scenario_asks', ()):
                # an ask in one of the member's scenario files: the member is all its files (E §3.1 step 1), so
                # the member alone, its scenarios loaded, answers it in its world (L §6.1 "A member's asks are asked
                # inside the joint model as a.q"; L §6.4 Asking)
                return self.member_alone(qid, alias, aid, mm)
            raise TallyError('LOAD-NAME', 'no ask %s' % qid)
        if member_names_world(mm, hit[0]):
            # a base ask of the member that names one of its scenarios (in, diff, a scenario pin, as if): L §6.1 "A
            # member's asks are asked by name in a run of the joint model, as a.q, and answered by the member
            # alone", and L §5.18 checks and answers a line only in the worlds it is evaluated in, so the member
            # alone, with all its files, answers it (MC-E1-F1)
            return self.member_alone(qid, alias, aid, mm)
        if alias not in sub:
            sub[alias] = Asker(mm)
        out = []
        for l in sub[alias].answer(hit[0]):
            out.append(alias + '.' + l if not l.startswith((' ', 'warning')) else l)
        return out

    # ------------------------------------------------------------------------------------------
    def answer(self, a):
        self.eng_warn = []
        cur = getattr(self.m, 'currencies', None)
        if cur is not None:
            # the global currency table is this reality's (a scenario whose load was refused may have reset it;
            # L §8, §8.3: a load refusal in a scenario file refuses only that scenario)
            CURRENCIES.clear()
            CURRENCIES.update(cur)
        if getattr(a, 'refused_alone', None) is not None:
            # GD-18: this ask's LOAD-SCENARIO refuses it alone (L §6.4, §8)
            return ['%s: refused: %s' % (a.id, a.refused_alone.text())]
        try:
            return self._answer(a)
        except TallyError as e:
            if is_load(e):
                raise LoadAtRun(e)
            return ['%s: refused: %s %s' % (a.id, e.code, e.msg)]
        except (LoadAtRun, KeyboardInterrupt):
            raise
        except RecursionError:
            return ['%s: refused: REFCORE-NOT-YET nesting deeper than this interpreter\'s stack' % a.id]
        except Exception as e:
            # nothing may crash: an input this interpreter can't handle is a refusal
            import os
            if os.environ.get('REFCORE_DEBUG'):
                raise
            return ['%s: refused: REFCORE-NOT-YET an input this interpreter can\'t handle (%s: %s)'
                    % (a.id, type(e).__name__, e)]
        finally:
            for x in self.eng_warn:
                if x not in self.walk_warnings:
                    self.walk_warnings.append(x)

    def _answer(self, a):
        m = self.m
        if a.paths is not None and a.paths > MAX_PATHS:
            return ['%s: refused: REQ-PATHS %d paths for one ask; the cap is 100 000' % (a.id, a.paths)]
        eng = Engine(m)
        self.eng_warn = eng.walk_warnings
        eng.check_range_lits()
        eng.cross = getattr(self, 'cross', None)
        eng.cross_ask = a
        if a.paths is not None:
            eng.ask_paths = (a.paths, a.seed)
        base = Ctx(None, m.on)
        # as known D sets the cut K (L §4.5); law D gives the law date (L §4.3)
        K = m.on
        if a.known is not None:
            v = eng.ev(a.known, {}, base)
            if isinstance(v, Hole):
                # L §5.11: "an ask's law or as known that is a hole make ... the ask line, a hole carrying their
                # questions, and nothing inside is evaluated"
                return ['%s: %s' % (a.id, fmt_value(v))]
            K = eng.date_of(v, 'as known')
        eng.ask_K = K
        law = None
        if a.law is not None:
            v = eng.ev(a.law, {}, Ctx(None, K))
            if isinstance(v, Hole):
                return ['%s: %s' % (a.id, fmt_value(v))]
            law = eng.date_of(v, 'the law of an ask')
        # in S names the world (L §5.18, §6.4); the model's own id is the base
        w0 = a.world if (a.world is not None and a.world != m.header.id) else None
        ctx = Ctx(law, K, windows=a.windows, partial=a.partial, unc=bool(a.uncertain), world=w0)
        if a.random is not None:
            # the line's random clause, for every projection it reaches (L §5.18)
            ctx = ctx.but(rnd=eng.rnd_clause(a.random, ctx))
        each_mode = 'each' if a.windows == 'each' else ('each-read' if getattr(m, 'read_each', False) else None)
        if each_mode is not None:
            # windows each on the line, or on a projection read some line may reach (L §4.4: "for that read"): the
            # line splits by the sub-windows; under each-read only the reads that state it take the assignment
            ctx = ctx.but(windows=(each_mode, ()))
        pins_coll = Coll()
        eng.stack.append(pins_coll)
        self._outside_ctx = ctx
        try:
            if a.world is not None:
                eng.reg(('world', a.world))
            try:
                ctx = eng.apply_pins(a.pins, {}, ctx, ask=True)
            except HoleSignal as h:
                # a pin whose key or date is a hole makes the ask line that hole (L §5.11)
                return ['%s: %s' % (a.id, fmt_value(h.hole))]
        finally:
            eng.stack.pop()
        eng.ask_K = ctx.cut
        old_ws = set_world(eng.world_chain(ctx.world))
        try:
            out = self._answer_in(eng, a, ctx, pins_coll)
        finally:
            set_world(old_ws)
        # E §9 invariant 4: asks move no money; the timelines the ask walked are as they were when walked
        for w in list(eng.walks.values()):
            sig = getattr(w, 'sig', None)
            if sig is not None and getattr(w, 'failed', None) is None and w.signature() != sig:
                return ['%s: refused: ENGINE-INVARIANT asks move no money: the timeline at cut %s changed while the '
                        'ask was answered (E §9 invariant 4)' % (a.id, w.K)]
        return out

    def _answer_in(self, eng, a, ctx, pins_coll):
        m = self.m
        each_mode = ctx.windows[0] if isinstance(ctx.windows, tuple) and ctx.windows else None
        if a.form == 'diff':
            return self.answer_diff(eng, a, ctx, pins_coll)
        if a.form == 'residuals':
            return self.answer_residuals(eng, a, ctx, pins_coll)
        if a.path is not None:
            # path p replays path p of the run (L §5.16)
            ctx = ctx.but(path=(a.seed, a.path))
        pinned_axes = dict(ctx.axes)
        axes = [ax for ax in getattr(a, '_axes', []) if ax not in pinned_axes]
        combos = [()]
        for ax in axes:
            opts = m.axes[ax].options
            combos = [c + ((ax, o),) for c in combos for o in opts]
        if a.binds and axes and a.form != 'readers-of' and each_mode is None:
            return self.binds_over_axes(eng, a, ctx, pins_coll, pinned_axes, axes, combos)
        # binding tuples (L §5.18: one line per binding tuple)
        tuples = []
        tregs = {}
        if a.binds:
            # each tuple keeps what its members brought (an unfulfilled instance it binds, L §8.2), for its own line
            mem = list(eng.members(a.binds, {}, ctx, regs=[]))
            for env, st, rg in mem:
                if isinstance(st, tuple) and st[0] == 'domainhole':
                    return ['%s: %s' % (a.id, fmt_value(st[1]))]
                # L §5.18: "A tuple whose where is a hole prints its line as that hole, its value unevaluated"
                tuples.append((env, st if isinstance(st, Hole) else None))
                tregs[id(env)] = rg
        else:
            tuples.append(({}, None))
        out = []
        if a.form == 'readers-of':
            return self.readers_of(a)
        for env, wh in tuples:
            env_text = ''
            if a.binds:
                env_text = ' (%s)' % ', '.join('%s = %s' % (b.var, fmt_bind(b, env[b.var])) for b in a.binds)
            if wh is not None:
                out.append('%s%s: %s' % (a.id, env_text, fmt_value(wh)))
                continue
            if each_mode is not None:
                if axes:
                    out.extend(self.each_over_axes(eng, a, env, env_text, ctx, pins_coll, pinned_axes, axes, combos))
                    continue
                out.extend(self.each_lines(eng, a, env, env_text, ctx.but(axes=tuple(sorted(pinned_axes.items()))),
                                           pins_coll))
                continue
            results = {}
            for c in combos:
                cctx = ctx.but(axes=tuple(sorted(list(pinned_axes.items()) + list(c))))
                results[c] = self.evaluate(eng, a, env, cctx)
            lines = self.group(a, axes, combos, results)
            for ln in lines:
                for rc in tregs.get(id(env), ()):
                    ln.coll.merge(rc)
                out.extend(self.print_line(a, env_text, ln, pins_coll, ctx))
        return out

    def binds_over_axes(self, eng, a, ctx, pins_coll, pinned_axes, axes, combos):
        """An ask whose evaluation, its `for … where` included, reaches axes: every combination of them is evaluated
        (L §5.5), the binding tuples with it; one line per binding tuple and per distinct result across the
        combinations where that tuple is bound (L §5.18 Lines)."""
        order = []
        per = {}
        wholes = {}
        for c in combos:
            cctx = ctx.but(axes=tuple(sorted(list(pinned_axes.items()) + list(c))))
            mem = list(eng.members(a.binds, {}, cctx))
            prev = -1
            for env, st in mem:
                if isinstance(st, tuple) and st[0] == 'domainhole':
                    return ['%s: %s' % (a.id, fmt_value(st[1]))]
                key = tuple(vkey(env[b.var]) for b in a.binds)
                if isinstance(st, Hole):
                    # L §5.18: "A tuple whose where is a hole prints its line as that hole, its value unevaluated"
                    wholes.setdefault(key, []).append(st)
                if key not in per:
                    per[key] = (env, [])
                    order.insert(prev + 1, key)
                prev = order.index(key)
                per[key][1].append(c)
        out = []
        for key in order:
            env, cs = per[key]
            env_text = ' (%s)' % ', '.join('%s = %s' % (b.var, fmt_bind(b, env[b.var])) for b in a.binds)
            if key in wholes:
                out.append('%s%s: %s' % (a.id, env_text, fmt_value(merge(*wholes[key]))))
                continue
            results = {}
            for c in cs:
                cctx = ctx.but(axes=tuple(sorted(list(pinned_axes.items()) + list(c))))
                results[c] = self.evaluate(eng, a, env, cctx)
            for ln in self.group(a, axes, cs, results):
                out.extend(self.print_line(a, env_text, ln, pins_coll, ctx))
        return out

    # -- diff (L §5.18, §6.4) -------------------------------------------------------------------------
    def answer_diff(self, eng, a, ctx, pins_coll):
        """diff(e [for binds]) in S against B: e in world S and in world B, line by line keyed by the binding tuple
        (the expression's text without binds); `added` (only in S), `removed` (only in B), `changed` with both
        values; the unchanged lines counted.  Layout (this interpreter's): a head line with the counts, then one line
        per difference, then the register of both evaluations."""
        m = self.m
        e, bs, sw, bw = a.diff
        coll = Coll()
        coll.merge(pins_coll)
        res = {}
        only = getattr(self, '_diff_only', None)
        for w in (sw, bw):
            if only is not None and w != only:
                continue
            wv = None if w == m.header.id else w
            wctx = ctx.but(world=wv)
            old = set_world(eng.world_chain(wv))
            eng.stack.append(coll)
            try:
                eng.reg(('world', w))
                lines = []
                if bs:
                    mem = list(eng.members(bs, {}, wctx))
                    for env, st in mem:
                        if isinstance(st, tuple) and st[0] == 'domainhole':
                            lines.append((_txt(e), st[1]))
                            continue
                        key = '(%s)' % ', '.join('%s = %s' % (b.var, fmt_bind(b, env[b.var])) for b in bs)
                        if isinstance(st, Hole):
                            # "and so does a diff line in the world where its where is a hole" (L §5.18)
                            lines.append((key, st))
                            continue
                        lines.append((key, self.one(eng, e, env, wctx)))
                else:
                    lines.append((_txt(e), self.one(eng, e, {}, wctx)))
            finally:
                eng.stack.pop()
                set_world(old)
            res[w] = lines
        t = getattr(e, '_t', None)
        if only is not None:
            # one side of a diff whose worlds sit on two stacks (cli.run_worlds): the other side is another reality's
            self._diff_side = (res[only], coll, ctx, t)
            return []
        return self.diff_combine(a, res, coll, ctx, t)

    def diff_combine(self, a, res, coll, ctx, t):
        e, bs, sw, bw = a.diff
        sl = dict(res[sw])
        bl = dict(res[bw])
        out_lines = []
        counts = {'changed': 0, 'added': 0, 'removed': 0, 'unchanged': 0}
        for key, v in res[sw]:
            if key in bl:
                if rkey(v) == rkey(bl[key]):
                    counts['unchanged'] += 1
                else:
                    counts['changed'] += 1
                    out_lines.append('  changed %s: %s against %s' % (key, fmt_value(v, ctx.partial, t),
                                                                    fmt_value(bl[key], ctx.partial, t)))
            else:
                counts['added'] += 1
                out_lines.append('  added %s: %s' % (key, fmt_value(v, ctx.partial, t)))
        for key, v in res[bw]:
            if key not in sl:
                counts['removed'] += 1
                out_lines.append('  removed %s: %s' % (key, fmt_value(v, ctx.partial, t)))
        head = '%s: diff in %s against %s: %d changed, %d added, %d removed, %d unchanged' % (
            a.id, sw, bw, counts['changed'], counts['added'], counts['removed'], counts['unchanged'])
        return [head] + out_lines + ['  ' + r for r in self.register(coll, [], ctx)]

    # -- residuals (E §6.3) ----------------------------------------------------------------------------
    def answer_residuals(self, eng, a, ctx, pins_coll):
        """residuals x span [by f]: the ledger's groups, each with its totals and its lines.  Layout (this
        interpreter's): a head line, then per group `<month> <group>: model M, observed O, difference D` and its
        lines indented under it, then the register."""
        r, sp, by = a.resid
        coll = Coll()
        coll.merge(pins_coll)
        eng.stack.append(coll)
        try:
            p = eng.place_arg(r, {}, ctx)
            if isinstance(p, Hole):
                return ['%s: %s' % (a.id, fmt_value(p))]
            span = eng.span_value(sp, {}, ctx)
            if isinstance(span, Hole):
                return ['%s: %s' % (a.id, fmt_value(span))]
            try:
                pu = eng.m.place_unit(eng.m.things[p.name])
                if pu == 'any':
                    # E §6.3 (round 12): "A place of unit any gives one ledger per unit, in the order of the units'
                    # ids" (writer 5's T-101)
                    ledgers = []
                    for u in eng.residual_units(p.name, span, ctx):
                        groups, unit = eng.residuals(p.name, span, by, ctx, unit=u)
                        ledgers.append((u, groups, unit))
                else:
                    groups, unit = eng.residuals(p.name, span, by, ctx)
                    ledgers = [(None, groups, unit)]
            except TallyError as err:
                if is_load(err):
                    raise LoadAtRun(err)
                return ['%s: refused: %s %s' % (a.id, err.code, err.msg)]
        finally:
            eng.stack.pop()
        out = ['%s: residuals %s %s%s' % (a.id, p.name, _txt(sp), (' by %s' % by) if by else '')]
        for u, groups, unit in ledgers:
            ind = '  '
            if u is not None:
                out.append('  ledger %s:' % u)
                ind = '    '
            for g, mt, ot, dt, ls in groups:
                out.append('%s%s: model %s, observed %s, difference %s' % (
                    ind, ('%s %s' % (g[0].label(), g[1])) if g[0] is not None else g[1], eng.rtext(mt, unit),
                    eng.rtext(ot, unit), eng.rtext(dt, unit)))
                for ln in ls:
                    out.append(ind + '  ' + ln.text)
        out.extend('  ' + x for x in self.register(coll, [], ctx))
        return out

    # -- windows each (L §4.4, §5.18) ----------------------------------------------------------------
    def each_lines(self, eng, a, env, env_text, ctx, pins_coll):
        """One line per maximal sub-window, or combination of sub-windows, over which the answer is constant;
        one plain line when it doesn't depend on the day.  Layout: `<ask> [<window> = <first> .. <last>]: v`."""
        out = []
        for suffix, ln in self.each_rows(eng, a, env, ctx):
            out.extend(self.print_line(a, env_text + suffix, ln, pins_coll, ctx))
        return out

    def each_over_axes(self, eng, a, env, env_text, ctx, pins_coll, pinned_axes, axes, combos):
        """windows each on a line that reaches axes: each combination's sub-window lines; combinations whose lines
        are the same are printed together, split into products by the axes (L §4.4, §5.5).  Layout: the window part,
        then the axis part."""
        results = {}
        for c in combos:
            cctx = ctx.but(axes=tuple(sorted(list(pinned_axes.items()) + list(c))))
            results[c] = EachRes(self.each_rows(eng, a, env, cctx))
        out = []
        for ln in self.group(a, axes, combos, results):
            for suffix, row in ln.trace:
                l2 = Line(None, ln.axes_text, row.vals, row.coll, ln.reading, row.trace)
                out.extend(self.print_line(a, env_text + suffix, l2, pins_coll, ctx))
        return out

    def each_rows(self, eng, a, env, ctx):
        """The sub-window lines of windows each: [(window part, Line)]."""
        pending = [()]
        done = []
        seqs = []
        labels = {}
        mode = ctx.windows[0] if isinstance(ctx.windows, tuple) and ctx.windows else 'each'
        while pending:
            asg = pending.pop(0)
            from .evaluate import EachMet
            eng.each_met = EachMet(eng)
            old = set_each(eng.each_met)
            try:
                res = self.evaluate(eng, a, env, ctx.but(windows=(mode, asg)))
            finally:
                set_each(old)
                met = eng.each_met
                eng.each_met = None
            # the windows met by memo cells the line reused (an earlier axis option computed them; MC-A-F1)
            for k, v in (getattr(getattr(res, 'coll', None), 'each', None) or {}).items():
                dict.setdefault(met, k, v)
            assigned = dict(asg)
            new = [w for w in met if w not in assigned]
            if new:
                w = new[0]
                label, days = met[w]
                labels[w] = (label, days)
                pending = [asg + ((w, d),) for d in days] + pending
                if len(pending) + len(done) > 20000:
                    raise NotYet('windows each over more than 20 000 day combinations')
                continue
            done.append((assigned, res))
            seqs.append(asg)
        keys = set(r.key() for asg, r in done)
        if len(keys) == 1:
            ln = Line(None, [], done[0][1].vals, done[0][1].coll, [], done[0][1].trace)
            for asg, r in done[1:]:
                ln.coll.merge(r.coll)
            return [('', ln)]
        order = list(labels)
        if any(set(asg) != set(order) for asg, r in done):
            return self.each_nested([(sq, r) for sq, (asg, r) in zip(seqs, done)], labels)
        # runs of consecutive days with the same answer, window by window (a product of runs per line)
        by = {}
        for asg, r in done:
            by.setdefault(r.key(), []).append((asg, r))
        rows = []
        for k, items in by.items():
            cells = [tuple(asg[w] for w in order) for asg, r in items]
            for box in boxes(cells, [labels[w][1] for w in order]):
                rr = [r for asg, r in items if all(lo <= asg[w] <= hi for w, (lo, hi) in zip(order, box))]
                rows.append((box, rr))
        rows.sort(key=lambda x: tuple(lo for lo, hi in x[0]))
        out = []
        for box, rr in rows:
            parts = []
            for w, (lo, hi) in zip(order, box):
                parts.append('%s = %s' % (labels[w][0], lo if lo == hi else '%s .. %s' % (lo, hi)))
            ln = Line(None, [], rr[0].vals, rr[0].coll, [], rr[0].trace)
            for r in rr[1:]:
                ln.coll.merge(r.coll)
            out.append((' [%s]' % ', '.join(parts), ln))
        return out

    def each_nested(self, items, labels):
        """windows each where the windows a line meets depend on the day an earlier window takes: "the combinations
        nest: under each sub-window of the earlier window, the windows met there are split in turn" (L §4.4)."""
        def span(days, lo, hi):
            return days[lo] if lo == hi else '%s .. %s' % (days[lo], days[hi])

        def nest(its, depth):
            # its share their first `depth` assignments; the next window met is then one and the same
            if len(set(r.key() for sq, r in its)) == 1:
                return [((), [r for sq, r in its])]
            w = its[0][0][depth][0]
            label, days = labels[w]
            per = []
            for d in days:
                sub = [(sq, r) for sq, r in its if sq[depth][1] == d]
                if sub:
                    per.append((d, nest(sub, depth + 1)))
            out = []
            i = 0
            while i < len(per):
                sig = [(parts, rs[0].key()) for parts, rs in per[i][1]]
                j = i
                while j + 1 < len(per) and [(parts, rs[0].key()) for parts, rs in per[j + 1][1]] == sig:
                    j += 1
                dl = [d for d, _ in per]
                for k, (parts, rs) in enumerate(per[i][1]):
                    allr = [r for t in range(i, j + 1) for r in per[t][1][k][1]]
                    out.append(((('%s = %s' % (label, span(dl, i, j))),) + parts, allr))
                i = j + 1
            return out

        out = []
        for parts, rs in nest(items, 0):
            ln = Line(None, [], rs[0].vals, rs[0].coll, [], rs[0].trace)
            for r in rs[1:]:
                ln.coll.merge(r.coll)
            out.append((' [%s]' % ', '.join(parts), ln))
        return out

    def evaluate(self, eng, a, env, ctx):
        coll = Coll()
        eng.stack.append(coll)
        try:
            f = a.form
            if f == 'exprs':
                vals = tuple(self.one(eng, e, env, ctx, line=True) for e in a.exprs)
                trace = None
            elif f == 'solve':
                vals = (self.one_solve(eng, a, env, ctx),)
                trace = None
            elif f == 'check':
                vals = (self.check(eng, a, env, ctx),)
                trace = None
            elif f == 'explain' and getattr(a.ref, '_r', None) is not None and a.ref._r[0] == 'moveref':
                # a movement instance (E §10)
                v = self.one(eng, a.ref, env, ctx)
                vals = (v,)
                trace = ExplainLines(self.explain_instance(eng, v, ctx))
            elif f == 'explain' and getattr(a.ref, '_r', None) is not None and a.ref._r[0] == 'fillread':
                # a fill take (E §10)
                v, lines = self.explain_fill(eng, a, env, ctx)
                vals = (v,)
                trace = ExplainLines(lines)
            elif f == 'explain' and getattr(a.ref, '_r', None) is not None and a.ref._r[0] == 'read' and \
                    a.ref._r[1] == 'def' and a.ref._r[2] in getattr(self.m, 'walk_bound', ()):
                # a walk-bound def (E §10, L §5.18): its cell at each walk point that read it
                v, lines = self.explain_walk_bound(eng, a, env, ctx)
                vals = (v,)
                trace = ExplainLines(lines)
            elif f == 'explain' and a.ref.k in ('bal', 'low'):
                # a balance or a projection (L §5.18 explain row, E §10)
                v, lines = self.explain_balance(eng, a, a.ref, env, ctx)
                vals = (v,)
                trace = ExplainLines(lines)
            elif f == 'explain':
                eng.tracing = True
                eng.trace_cells = {}
                eng.memo_saved = eng.memo
                eng.memo = {}
                root = TraceNode('root', _txt(a.ref))
                eng.trace_stack = [root]
                try:
                    v = self.one(eng, a.ref, env, ctx)
                finally:
                    eng.tracing = False
                    eng.trace_stack = []
                    eng.memo = eng.memo_saved
                root.value = v
                if a.pins and getattr(self, '_outside_ctx', None) is not None:
                    root = self.explain_pins(eng, a, env, ctx, root, v)
                vals = (v,)
                trace = root
            elif f == 'inputs-of':
                sub = Coll()
                eng.stack.append(sub)
                try:
                    v = self.one(eng, a.ref, env, ctx)
                finally:
                    eng.stack.pop()
                coll.merge(sub)
                texts = {}
                for sid, r in sub.rows.items():
                    texts[sid] = self.row_line(r)
                vals = (RowSet(set(texts), texts),)
                trace = None
            else:
                raise NotYet('ask form %s' % f)
        finally:
            eng.stack.pop()
        return Result(vals, coll, trace)

    def one(self, eng, e, env, ctx, line=False):
        try:
            if line and is_query_line(e):
                return eng.query_line(e, env, ctx)
            return eng.ev(e, env, ctx)
        except HoleSignal as h:
            return h.hole
        except TallyError as err:
            if is_load(err):
                raise LoadAtRun(err)
            return err

    def one_solve(self, eng, a, env, ctx):
        try:
            return eng.solve_value(a.solve, env, ctx)
        except HoleSignal as h:
            return h.hole
        except TallyError as err:
            if is_load(err):
                raise LoadAtRun(err)
            return err

    def check(self, eng, a, env, ctx):
        """check e == f: ok, or differs with both values and e − f (L §5.18)."""
        l, r = a.check
        out = self._check(eng, a, env, ctx)
        # "Sides read at different law dates list each" (L §5.18): a side's own `as if {law D}`
        laws = (self.side_law(eng, l, env, ctx), self.side_law(eng, r, env, ctx))
        if laws[0] != laws[1]:
            out.laws = laws
        return out

    def side_law(self, eng, e, env, ctx):
        while e.k == 'paren':
            e = e.e
        if e.k == 'asif':
            for p in e.pins:
                if p.kind == 'law':
                    try:
                        v = eng.ev(p.value, env, ctx)
                        return eng.date_of(v, 'the law pin')
                    except (TallyError, HoleSignal):
                        return None
        return ctx.law

    def _check(self, eng, a, env, ctx):
        l, r = a.check
        lv = self.one(eng, l, env, ctx)
        rv = self.one(eng, r, env, ctx)
        lt, rt = _txt(l), _txt(r)
        if isinstance(lv, (TallyError, Hole)) or isinstance(rv, (TallyError, Hole)):
            return CheckOut('%s = %s, %s = %s' % (lt, fmt_value(lv, ctx.partial), rt, fmt_value(rv, ctx.partial)),
                            'open')
        try:
            same = eng.compare('==', lv, rv, None, ctx)
        except TallyError as err:
            if is_load(err):
                raise LoadAtRun(err)
            return CheckOut('refused: %s %s' % (err.code, err.msg), 'refused')
        if isinstance(same, Hole):
            return CheckOut('%s = %s, %s = %s' % (lt, fmt_value(lv), rt, fmt_value(rv)), 'open')
        if same:
            return CheckOut('ok', 'ok')
        diff = ''
        if isinstance(lv, Num) and isinstance(rv, Num):
            from .evaluate import arith
            try:
                d = arith('-', lv, rv, l._t, l._t, r._t)
                diff = ', difference %s' % fmt(d)
            except TallyError as err:
                if is_load(err):
                    raise LoadAtRun(err)
                diff = ', difference refused: %s %s' % (err.code, err.msg)
        else:
            # L §5.18 (round 12): "and, where e - f is defined (§3.2, §4.2), the difference e − f": of two keys of one
            # ordered index an int (L §4.2); of anything else (dates, periods off an index, text, codes) none
            ltt, rtt = getattr(l, '_t', None), getattr(r, '_t', None)
            if ltt is not None and rtt is not None and ltt.k == 'key' and rtt.k == 'key' and ltt.ix == rtt.ix \
                    and ltt.ix in eng.m.indexes:
                fake = type('N', (), {})()
                fake.l = l
                try:
                    diff = ', difference %s' % fmt(eng.key_diff(fake, lv, rv))
                except TallyError as err:
                    if is_load(err):
                        raise LoadAtRun(err)
                    diff = ', difference refused: %s %s' % (err.code, err.msg)
        return CheckOut('differs: %s = %s, %s = %s%s' % (lt, fmt(lv), rt, fmt(rv), diff), 'differs')

    # -- axes (L §5.5) --------------------------------------------------------------------------
    def group(self, a, axes, combos, results):
        """One line per distinct result; each set of combinations split into products by
        the axes in id order (L §5.5)."""
        by = {}
        order = []
        for c in combos:
            k = results[c].key()
            if k not in by:
                by[k] = []
                order.append(k)
            by[k].append(c)
        lines = []
        for k in order:
            cs = by[k]
            for prod in split(axes, [dict(c) for c in cs], self.m):
                coll = Coll()
                first = None
                for c in cs:
                    d = dict(c)
                    if all(d[ax] in opts for ax, opts in prod):
                        coll.merge(results[c].coll)
                        if first is None:
                            first = results[c]
                axes_parts = []
                reading = []
                for ax, opts in prod:
                    allo = self.m.axes[ax].options
                    if len(opts) == len(allo):
                        axes_parts.append((ax, None))
                    else:
                        ol = [o for o in allo if o in opts]
                        axes_parts.append((ax, ol))
                        reading.append('%s = %s' % (ax, ' | '.join(ol)))
                lines.append(Line(None, axes_parts, first.vals, coll, reading, first.trace))
        # order: by the axes' options in declaration order, axes in id order; * first
        def sk(ln):
            out = []
            for ax, ol in ln.axes_text:
                if ol is None:
                    out.append(-1)
                else:
                    out.append(self.m.axes[ax].options.index(ol[0]))
            return out
        lines.sort(key=sk)
        return lines

    # -- printing ---------------------------------------------------------------------------------
    def print_line(self, a, env_text, ln, pins_coll, ctx):
        ax = ''
        if ln.axes_text:
            ax = ' [%s]' % ', '.join('%s = %s' % (x, '*' if ol is None else ' | '.join(ol)) for x, ol in ln.axes_text)
        head = '%s%s%s' % (a.id, env_text, ax)
        vals = ln.vals
        out = []
        if a.form == 'inputs-of':
            rs = vals[0]
            out.append('%s:' % head)
            for sid in sorted(rs.rows):
                out.append('  %s' % rs.texts[sid])
        elif a.form == 'explain':
            out.append('%s: %s' % (head, fmt_value(vals[0], ctx.partial)))
            if a.path is not None:
                # a path value: "the seed and path number, the samples it reached in canonical order" (E §10; the
                # samples are the register's sample: lines)
                out.append('  path %d of seed %d' % (a.path, a.seed))
            out.extend(self.explain_lines(ln.trace, ctx))
        elif len(vals) == 1:
            out.append('%s: %s' % (head, fmt_value(vals[0], ctx.partial, self.etype(a, 0))))
        else:
            out.append('%s: (%s)' % (head, ', '.join(fmt_value(v, ctx.partial, self.etype(a, i))
                                                      for i, v in enumerate(vals))))
        coll = Coll()
        coll.merge(pins_coll)
        coll.merge(ln.coll)
        laws = getattr(vals[0], 'laws', None) if vals else None
        out.extend('  ' + r for r in self.register(coll, ln.reading, ctx, a.path is not None, laws))
        if a.orient is not None:
            # "Any other ask may state it, and it is printed" (L §5.15): this interpreter's layout
            out.append('  orient %s' % a.orient)
        return out

    def etype(self, a, i):
        if a.form == 'exprs' and i < len(a.exprs):
            return getattr(a.exprs[i], '_t', None)
        if a.form == 'solve':
            return getattr(a.solve, '_t', None)
        return None

    def register(self, coll, reading, ctx, samples=False, laws=None):
        ent = {}
        assumed = {}
        smp = []
        lj = {}
        for e in coll.reg:
            kind = e[0]
            if kind == 'hz':
                continue
            if kind == 'sample':
                smp.append(e)
                continue
            if kind == 'left out joint':
                # L §8.2 `left out: <move>, <n> instances, first <day>`: the joint model's own movements into a
                # member's place that nothing links (§6.1)
                lj.setdefault(e[1], set()).add(e[2])
                continue
            if kind == 'labels':
                # tally 2.1 (L §8.2 Dropped labels, §8.5 WARN-LABELS `labels dropped at <place> from <d>: <movements>`)
                lb = ent.setdefault('_labels', {})
                cur = lb.setdefault(e[1], [e[2], set()])
                cur[0] = min(cur[0], e[2])
                cur[1].add(e[3])
                continue
            if kind == 'assumed':
                assumed.setdefault(e[1], set()).add(e[2])
            elif kind == 'missing':
                ent.setdefault('missing', set()).add('missing record counted by %s' % e[1])
            else:
                ent.setdefault(kind, set()).add('%s: %s' % (kind, e[1]))
        for mid, days in lj.items():
            from .joint import shown as _jshown
            ent.setdefault('left out', set()).add('left out: %s, %d instances, first %s' % (
                _jshown(mid), len(days), min(days)))
        out = []
        for reason in sorted(assumed):
            ks = sorted(x for x in assumed[reason] if x)
            out.append('assumed: %s%s' % (reason, (' (%s)' % ', '.join(ks)) if ks else ''))
        if ent.get('pinned'):
            ent['pinned'] = merge_pin_days(ent['pinned'])
        for kind in ('given', 'missing', 'pinned', 'pinned sample'):
            out.extend(sorted(ent.get(kind, ())))
        if samples:
            # each sample the path p line reached, in canonical order (L §5.16 Order, §8.2)
            first = {}
            for e in smp:
                if e[2] not in first or e[1] < first[e[2]]:
                    first[e[2]] = e[1]
            out.extend('sample: %s' % t for t in sorted(first, key=lambda t: (first[t], t)))
        out.extend(sorted(ent.get('world', ())))
        out.extend('reading: %s' % r for r in reading)
        for kind in ('anchor', 'pad', 'adjustment', 'unfulfilled', 'open', 'left out', 'uncertain'):
            out.extend(sorted(ent.get(kind, ())))
        for place, (d0, mids) in sorted(ent.get('_labels', {}).items()):
            out.append('WARN-LABELS labels dropped at %s from %s: %s' % (place, d0, ', '.join(sorted(mids))))
        out.extend(sorted(ent.get('held', ())))
        if laws is not None:
            # a check whose sides are read at different law dates lists each (L §8.2, §5.18)
            out.append('law date %s (left), %s (right), cut %s' % (laws[0] or 'none', laws[1] or 'none', ctx.cut))
        elif ctx.law is not None:
            out.append('law date %s, cut %s' % (ctx.law, ctx.cut))
        else:
            out.append('cut %s' % ctx.cut)
        return out

    def row_line(self, r):
        """A row read, with its source (L §5.18 inputs-of)."""
        m = self.m
        v = r.version
        src = m.sources.get(v.cite) if v.cite else None
        s = '%s (source %s' % (r.sid, v.cite)
        if src is not None and getattr(src, 'title', None):
            s += ' "%s"' % src.title
        loc = getattr(r, 'locator', None)
        if loc:
            s += ', locator "%s"' % loc
        return s + ')'

    def readers_of(self, a):
        r = a.ref._r
        kind, iid = r[1], r[2]
        item = (kind, iid)
        hits = self.m.checker.readers_of(item)
        names = ['%s' % h[1] for h in hits]
        return ['%s: %s' % (a.id, ', '.join(names) if names else 'nothing reads %s' % iid)]

    # -- explain (L §5.18 Explanation order; E §10) --------------------------------------------------
    def explain_instance(self, eng, v, ctx):
        """A movement instance: its class, day, booking cut and leg derivations; for a decision, its stratum and
        walk point; each later version with its δ, the adjustment it made and what settles or waives it (E §10).
        Layout: this interpreter's."""
        from .values import Mv
        from .moves import Inst
        if not isinstance(v, Mv) or not isinstance(v.src, Inst):
            return ['(%s: no instance at the cut)' % (v.id if isinstance(v, Mv) else fmt_value(v))]
        inst = v.src
        out = []
        tctx = eng.tctx(ctx)
        run = None
        if inst.decision:
            w = eng.main_walk(ctx)
            run = w.decs.get(inst.id)
        src = run if run is not None else inst
        st = getattr(src, 'status', None) or eng.status(inst, ctx)
        if inst.decision:
            out.append('decision, day %s, status %s' % (fmt_value(inst.day), st))
            out.append('stratum %d%s' % (inst.move.stratum, (', walk point %s' % self.point_text(run.point))
                                         if run is not None and run.point is not None else ', not run'))
            if run is not None and getattr(run, 'read_places', None):
                out.append('read at its walk point: %s' % ', '.join(sorted(run.read_places)))
        else:
            out.append('plan, day %s, booking cut %s, status %s' % (fmt_value(inst.day), inst.cut, st))
        if inst.exists is not True:
            out.append('exists: %s' % fmt_value(inst.exists))
        if src.err is not None:
            out.append('refused: %s %s' % (src.err.code, src.err.msg))
        for lg in src.legs:
            amt = lg.amount
            at = fmt_value(amt) if not isinstance(amt, TallyError) else 'refused: %s %s' % (amt.code, amt.msg)
            on = (' on %s' % fmt_value(lg.day)) if lg.day is not None and lg.day != inst.day else ''
            out.append('leg %s from %s to %s amount %s%s' % (lg.name, fmt_value(lg.frm), fmt_value(lg.to), at, on))
        for k, val in sorted(src.attrs.items()):
            out.append('attr %s = %s' % (k, fmt_value(val)))
        if not inst.decision and inst.cut is not None and inst.env is not None:
            # the derivation at the booking cut, traced (E §10: re-evaluates the cell's cone with tracing on)
            eng.tracing = True
            eng.trace_cells = {}
            saved = eng.memo
            eng.memo = {}
            root = TraceNode('root', inst.id)
            eng.trace_stack = [root]
            try:
                eng.eval_inst(inst.move, inst.env, inst.keys, tctx, at_cut=inst.cut)
            finally:
                eng.tracing = False
                eng.trace_stack = []
                eng.memo = saved
            sub = self.explain_lines(root, ctx)
            if sub:
                out.append('derivation at %s:' % inst.cut)
                out.extend('  ' + l.strip() if False else l for l in sub)
        if not inst.decision:
            for adj in eng.adjustments(ctx):
                if adj.inst.id != inst.id:
                    continue
                rem = adj.remainder()
                by = ', '.join(sorted(s.id for s, sign, amt in adj.settled if amt is not None)) or 'nothing'
                out.append('adjustment %s: change %s on %s (version %s), %s; remaining %s' % (
                    adj.id, fmt_value(adj.delta), adj.day, ', '.join(adj.vids),
                    ('waived' if adj.waived else 'settled by %s' % by), 'none' if rem is None else fmt_value(rem)))
        return out

    def explain_pins(self, eng, a, env, ctx, root, v):
        """explain under an ask's `with` pins, an as if over the line (E §10): the pins with their values, the
        derivation inside them, and the one outside them.  Layout: this interpreter's."""
        from .evaluate import _tfmt
        pinned = set(p.axis for p in a.pins if getattr(p, 'pkind', None) == 'axis')
        octx = self._outside_ctx.but(axes=tuple((ax, o) for ax, o in ctx.axes if ax not in pinned))
        eng.tracing = True
        eng.trace_cells = {}
        saved = eng.memo
        eng.memo = {}
        oroot = TraceNode('root', _txt(a.ref))
        eng.trace_stack = [oroot]
        old = set_world(eng.world_chain(octx.world))
        eng.stack.append(Coll())
        try:
            vo = self.one(eng, a.ref, env, octx)
        finally:
            eng.stack.pop()
            set_world(old)
            eng.tracing = False
            eng.trace_stack = []
            eng.memo = saved
        out = TraceNode('root', _txt(a.ref))
        pn = TraceNode('group', 'the pins, evaluated outside them')
        for line in eng.pin_lines(octx, ctx):
            pn.children.append(('node', TraceNode('group', 'pin ' + line)))
        inside = TraceNode('group', 'inside the pins: %s' % _tfmt(v))
        inside.children = root.children
        outside = TraceNode('group', 'outside the pins: %s' % _tfmt(vo))
        outside.children = oroot.children
        out.children = [('node', pn), ('node', inside), ('node', outside)]
        out.value = v
        return out

    def explain_walk_bound(self, eng, a, env, ctx):
        """`explain x[k]` of a def evaluated at walk points: the cell at each walk point that read it, in walk order,
        each with its point (day, stratum, movement, keys); with none, that nothing read it at those keys (E §10,
        L §5.18).  Layout: this interpreter's."""
        from .model import _kv
        n = a.ref
        qid = n._r[2]
        info = self.m.defs[qid]
        keynodes = n.keys if n.k == 'index' else []
        try:
            keys, hole = eng.key_values(info, keynodes, n._convs, env, ctx)
            if hole is not None:
                return hole, []
            w = eng.main_walk(ctx)
        except HoleSignal as h:
            return h.hole, []
        except TallyError as e:
            if is_load(e):
                raise LoadAtRun(e)
            return e, []
        org = eng.origin(info, keys)
        cells = eng.walk_cells.get((id(w), qid, tuple(_kv(k) for k in keys)), {})
        if not cells:
            return CheckOut('walk-bound, read at no walk point', 'walk'), [
                'nothing read %s at a walk point' % org]
        # E §10 (round 14): "the cell at each walk point where a decision read it, in walk order, each with its point
        # (day, stratum, movement, keys) and explained as a def cell (above), its running reads as balances up to
        # that point"
        out = []
        byp = {r.point: r.id for r in w.decs.values() if getattr(r, 'point', None) is not None}
        for p in sorted(cells):
            import datetime as _dt
            d = _dt.date.fromordinal(p[0])
            out.append('%s at (%s, stratum %s, %s): %s' % (org, d, p[2], byp.get(p, p[3].split('|')[0]),
                                                           fmt_value(cells[p])))
            # the state just before the reader's own booking at p: "its running reads as balances up to that point"
            ctx2 = w.tctx.but(cut=min(d, w.K), walk=w, wp=p[:4] + (p[4] - 0.5,))
            eng._explaining_wb = True
            eng.tracing = True
            eng.trace_cells = {}
            saved = eng.memo
            eng.memo = {}
            root = TraceNode('root', org)
            eng.trace_stack = [root]
            sub = Coll()
            eng.stack.append(sub)
            try:
                try:
                    v2 = eng.read_def(info, keys, ctx2)
                except HoleSignal as h:
                    v2 = h.hole
                except TallyError as e:
                    if is_load(e):
                        raise LoadAtRun(e)
                    v2 = e
            finally:
                eng.stack.pop()
                eng.tracing = False
                eng.trace_stack = []
                eng.memo = saved
                eng._explaining_wb = False
            root.value = v2
            out.extend('  ' + l for l in self.explain_lines(root, ctx2))
        return CheckOut('walk-bound, read at %d walk point%s' % (len(cells), '' if len(cells) == 1 else 's'),
                        'walk'), out

    def explain_fill(self, eng, a, env, ctx):
        """`explain F.taken[s, d]`: the fill's v version if it has one, the arcs in order, the remaining supply and
        demand before and after each take (E §10).  Layout: this interpreter's."""
        from .model import _kv
        n = a.ref
        qid = n._r[1]
        eng.fill_trace = []
        saved = eng.memo
        eng.memo = {}
        try:
            v = self.one(eng, n, env, ctx)
            trace = eng.fill_trace
            fi = eng.m.checker.fill_infos[qid]
            try:
                tkeys, h = eng.fill_keys(n._tkeys, n._convs, fi['sdoms'] + fi['ddoms'], env, ctx)
                fkeys, h2 = eng.fill_keys(n._fkeys, n._fconvs, fi['fdoms'], env, ctx)
            except TallyError:
                tkeys = fkeys = None
                h = h2 = None
        finally:
            eng.fill_trace = None
            eng.memo = saved
        if isinstance(v, TallyError) and not any(t[0] == qid for t in trace):
            return v, []
        ent = None
        want = tuple(_kv(k) for k in fkeys) if fkeys is not None and h2 is None else None
        for t in trace:
            if t[0] == qid and (want is None or t[1] == want):
                ent = t
        if ent is None:
            return v, []
        _, fk, sec, log = ent
        out = []
        head = 'fill %s%s' % (qid, ('[%s]' % ', '.join(key_text(k) for k in fkeys)) if fkeys else '')
        if sec.v is not None:
            ver = self.m.versions.get(sec.v)
            head += ', v %s%s' % (sec.v, (' (source %s)' % ver.cite) if ver is not None and ver.cite else '')
        else:
            head += ', no v'
        out.append(head)
        asked = tuple(_kv(k) for k in tkeys) if tkeys is not None and h is None else None
        found = False
        out.append('arcs in order:')
        for i, (x, r, q, t, r2, q2) in enumerate(log):
            sk = ', '.join(fmt(x['se'][b.var]) for b in sec.sbinds)
            dk = ', '.join(fmt(x['de'][b.var]) for b in sec.dbinds)
            mark = ''
            if asked is not None and x['sk'] + x['dk'] == asked:
                mark = '  <- this take'
                found = True
            out.append('  %d. (%s; %s): supply %s, demand %s; takes %s; then supply %s, demand %s%s' % (
                i + 1, sk, dk, fmt_value(r), fmt_value(q), fmt_value(t), fmt_value(r2), fmt_value(q2), mark))
        if asked is not None and not found:
            out.append('this pair is not an arc: it takes 0')
        return v, out

    def explain_balance(self, eng, a, n, env, ctx):
        """`explain bal(x, d)` (a balance) and `explain bal(x, d) from t` / `explain low(x, a .. b) from t` (a
        projection), E §10.  Layout: this interpreter's."""
        form = getattr(n, '_form', None)
        if n.k == 'low' and form != 'proj':
            # L §5.18 (round 8): "explain low(x, span) without from explains the day of its lowest balance as a
            # balance"
            return self.explain_low_plain(eng, a, n, env, ctx)
        v = self.one(eng, n, env, ctx)
        if isinstance(v, TallyError) or form == 'now':
            return v, []
        sub = Coll()
        eng.stack.append(sub)
        try:
            lines = self._explain_bal_lines(eng, a, n, env, ctx)
        except HoleSignal:
            lines = []
        except TallyError as e:
            if is_load(e):
                raise LoadAtRun(e)
            lines = ['refused: %s %s' % (e.code, e.msg)]
        finally:
            eng.stack.pop()
        return v, lines

    def explain_low_plain(self, eng, a, n, env, ctx):
        """`explain low(x, a .. b)` without `from`: the lowest of the plain end-of-day balances (L §5.14.1), and the
        day it is reached, explained as a balance (L §5.18 explain row, E §10's balance row). Days of one run of that
        balance with no entry between them share one derivation, explained at the run's first day; a lowest balance
        reached in two runs, with movements between, names no one day (REFCORE-UNSPECIFIED)."""
        from .walk import K_end
        v = self.one(eng, n, env, ctx)
        if isinstance(v, (TallyError, Hole)):
            return v, []
        sub = Coll()
        eng.stack.append(sub)
        try:
            p = eng.place_arg(n.ref, env, ctx)
            unit = eng.bal_unit(n, p.name, env, ctx) if not isinstance(p, Hole) else p
            sp = eng.span_value(n.span, env, ctx)
            if isinstance(p, Hole) or isinstance(unit, Hole) or isinstance(sp, Hole):
                return v, []
            w = eng.main_walk(ctx)
            d = sp.start
            last = sp.end - datetime.timedelta(days=1)
            runs = []
            prev = None
            while d <= last:
                st = w.pstate(p.name, unit, K_end(d), raw=True)
                if isinstance(st.val, Num) and isinstance(v, Num) and st.val.q == v.q:
                    if prev is not None and prev == d - datetime.timedelta(days=1):
                        runs[-1][1] = d
                    else:
                        runs.append([d, d])
                    prev = d
                d += datetime.timedelta(days=1)
            if not runs:
                raise NotYet('explain low(%s, ...): its lowest balance %s is on no day of the span' % (p.name, fmt(v)))
            # L §5.18: "explain low(x, span) without from explains the first day its lowest balance is reached, as a
            # balance": the first run (a later run of the same balance is not explained)
            d1, d2 = runs[0]
            head = 'lowest end-of-day balance of %s on %s .. %s: %s, on %s%s' % (
                p.name, sp.start, last, fmt(v), d1, (' (through %s)' % d2) if d2 != d1 else '')
            try:
                lines = self._explain_bal_lines(eng, a, n, env, ctx, day=d1)
            except HoleSignal:
                lines = []
            except TallyError as e:
                if is_load(e):
                    raise LoadAtRun(e)
                lines = ['refused: %s %s' % (e.code, e.msg)]
            return v, [head] + lines
        finally:
            eng.stack.pop()

    def _explain_bal_lines(self, eng, a, n, env, ctx, day=None):
        from .walk import K_end, K_start, kday, gtext
        p = eng.place_arg(n.ref, env, ctx)
        if isinstance(p, Hole):
            return ['the place is a hole: %s' % fmt_hole(p)]
        place = p.name
        unit = eng.bal_unit(n, place, env, ctx)
        if isinstance(unit, Hole):
            return ['the unit is a hole: %s' % fmt_hole(unit)]
        if day is not None:
            lo_day = end = day
        elif n.k == 'low':
            sp = eng.span_value(n.span, env, ctx)
            if isinstance(sp, Hole):
                return []
            lo_day, end = sp.start, sp.end - datetime.timedelta(days=1)
        else:
            d = eng.ev(n.args[-1], env, ctx)
            if isinstance(d, Hole):
                return []
            if isinstance(d, Iv) and isinstance(d.lo, datetime.date):
                d = eng.window_choice(d, ctx)
                if isinstance(d, Iv):
                    return []
            end = lo_day = eng.date_of(d, 'the day of bal')
        if getattr(n, '_form', None) == 'proj':
            return self._explain_projection(eng, a, n, env, ctx, place, unit, lo_day, end)
        w = eng.main_walk(ctx)
        key = K_start(end) if getattr(n, 'start', False) else K_end(end)
        modelled = getattr(n, '_form', None) == 'modelled'
        out = ['balance of %s (%s) at the %s of %s%s' % (place, unit, 'start' if getattr(n, 'start', False) else 'end', end,
                                                          ', modelled' if modelled else '')]
        st, lg, i = w.state(place, unit, key)
        since = None
        lo = -1
        if lg is None or st.rst < 0:
            st0 = w.default(place, unit)
            if st0.val is None:
                out.append('from: no reading or opening before it')
            else:
                th = self.m.things[place]
                from .model import SameDiffers
                try:
                    starts = [self.m.lifespan(th, w.K).start]
                except SameDiffers:
                    # a place stated same whose sides' lifespans differ, each starting at or after S (L §6.1)
                    starts = sorted(set(x.start for x in w.side_spans(th)))
                out.append('from: 0 at the start of its lifespan, %s' % ' and '.join(str(x) for x in starts))
        else:
            e = lg.ents[st.rst]
            lo = st.rst
            since = e.key
            out.append('from: %s' % self._reading_text(w, place, e, gtext))
        ents = []
        if lg is not None:
            ents.extend(lg.ents[lo + 1:i + 1])
        wl = w.logs.get((place, '*'))
        if wl is not None:
            ents.extend(x for x in wl.ents if (since is None or x.key > since) and x.key <= key)
        from .moves import ObsMv
        rows = []
        for x in sorted(ents, key=lambda x: x.key):
            if modelled and isinstance(x.mv, ObsMv):
                continue
            t = self._entry_text(x, gtext)
            if t is not None:
                rows.append((kday(x.key), t))
        extra = []
        if modelled:
            extra.extend((x, 'modelled') for x in w.modelled)
        if n.proj and n.proj.get('uncertain'):
            extra.extend((x, 'uncertain') for x in w.uncertain)
        for x, stt in extra:
            for g in x.legs:
                gd = g.day if isinstance(g.day, datetime.date) else lin_day(x.day)
                if gd is None or gd > end or (since is not None and K_end(gd) <= since):
                    continue
                for pp, sg in ((g.frm, -1), (g.to, 1)):
                    if isinstance(pp, Thing) and pp.name == place:
                        rows.append((gd, '%s.%s (%s): %s' % (x.id, g.name, stt, self._amount_text(g.amount, sg))))
        out.extend(self._by_day(rows))
        return out

    def _explain_projection(self, eng, a, n, env, ctx, place, unit, lo_day, end):
        from .walk import K_end, kday, gtext
        t = eng.ev(n.proj['frm'], env, ctx)
        if isinstance(t, Hole):
            return []
        t = eng.date_of(t, 'the day of a projection')
        clauses = eng.proj_clauses(n.proj, ctx)
        w = eng.proj_walk(ctx, t, n._leftout, clauses)
        w.run_to(end)
        windows, rnd, unc = clauses
        out = []
        if n.k == 'low':
            out.append('projection of %s (%s) from %s, lowest end-of-day balance on %s .. %s' % (
                place, unit, t, lo_day, end))
        else:
            out.append('projection of %s (%s) from %s to the end of %s' % (place, unit, t, end))
        out.append('holder: ask %s (an ask line leaves out nothing, L §5.14.1)' % a.id)
        rtext = 'none'
        if rnd:
            rtext = 'at mean' if rnd[0] == 'mean' else 'at quantile(%s)' % fmt_q(rnd[1])
        out.append('clauses: windows %s, random %s, %s' % (windows or 'none', rtext,
                                                            'with uncertain' if unc else 'without uncertain'))
        st0 = w.default(place, unit)
        out.append('start: %s at the start of %s' % (gtext(w.st_value(st0, place, unit)), t))
        lg = w.logs.get((place, unit))
        ents = []
        if lg is not None:
            ents.extend(x for x in lg.ents if x.key <= K_end(end))
        wl = w.logs.get((place, '*'))
        if wl is not None:
            ents.extend(x for x in wl.ents if x.key <= K_end(end))
        rows = []
        for x in sorted(ents, key=lambda x: x.key):
            tx = self._entry_text(x, gtext)
            if tx is not None:
                rows.append((kday(x.key), tx))
        if unc:
            for x in w.uncertain:
                for g in x.legs:
                    gd = g.day if isinstance(g.day, datetime.date) else lin_day(x.day)
                    if gd is None or gd > end or gd < t:
                        continue
                    for pp, sg in ((g.frm, -1), (g.to, 1)):
                        if isinstance(pp, Thing) and pp.name == place:
                            rows.append((gd, '%s.%s (uncertain): %s' % (x.id, g.name, self._amount_text(g.amount, sg))))
        out.append('counted:')
        out.extend('  ' + l for l in self._by_day(rows))
        lo = sorted((dd, i) for dd, i in w.leftout_insts if dd is not None and t <= dd <= end)
        if lo:
            out.append('left out:')
            out.extend('  %s %s' % (dd, i) for dd, i in lo)
        else:
            out.append('left out: none')
        if n.k == 'low':
            best = None
            dd = lo_day
            while dd <= end:
                st = w.pstate(place, unit, K_end(dd), raw=True)
                vv = w.st_value(st, place, unit)
                if isinstance(vv, Num) and (best is None or vv.q < best[1].q):
                    best = (dd, vv)
                dd += datetime.timedelta(days=1)
            if best is not None:
                out.append('lowest: %s at the end of %s' % (fmt(best[1]), best[0]))
        return out

    def _reading_text(self, w, place, e, gtext):
        from .moves import instant_text
        rows = list(e.coll.rows.values()) if e.coll is not None else []
        val = fmt_hole(Hole(e.qs)) if e.qs else fmt(e.val)
        if not rows:
            return '%s = %s' % (e.note or 'reading', val)
        r = rows[0]
        v = r.version
        s = '%s %s = %s, %s (version %s, source %s' % (e.note, r.sid, val, instant_text(r), v.id, v.cite)
        loc = getattr(r, 'locator', None)
        if loc:
            s += ', locator "%s"' % loc
        s += ')'
        if e.note in ('anchor', 'pad'):
            from .walk import kday
            for key, gap, r2, role, u, _bc in w.gaps.get((place, kday(e.key)), ()):
                if r2 is r or getattr(r2, 'sid', None) == r.sid:
                    if role[0] == 'anchor':
                        s += ', gap role anchor, gap %s' % gtext(gap)
                    else:
                        s += ', gap role pad from %s, gap %s (%s)' % (role[1].name, gtext(gap),
                                                                    e.mv.id if e.mv is not None else 'pad')
                    break
        return s

    def _amount_text(self, amt, sign):
        if isinstance(amt, Hole):
            return 'hole: %s' % fmt_hole(amt)
        if isinstance(amt, TallyError):
            return 'refused: %s %s' % (amt.code, amt.msg)
        q = sign * amt.q
        u = amt.u[0][0] if amt.u else None
        from .moves import qty
        return ('+' if q > 0 else '') + fmt(qty(q, u))

    def _entry_text(self, x, gtext):
        """One booked entry of a place's log: its movement's id, status and what it moved (E §10)."""
        from .moves import PadMv
        mv = x.mv
        mid = mv.id if mv is not None else 'the walk'
        if x.note and x.kind in ('delta', 'hole') and mv is not None and not isinstance(mv, PadMv):
            mid += '.%s' % x.note
        st = getattr(mv, 'status', None) if mv is not None else None
        head = '%s (%s)' % (mid, st) if st else mid
        if x.kind == 'delta':
            q = x.flow.q
            return '%s: %s%s' % (head, '+' if q > 0 else '', fmt(x.flow))
        if x.kind in ('hole', 'perm'):
            return '%s: hole: %s' % (head, fmt_hole(Hole(x.qs)))
        if x.kind == 'imp_on':
            return '%s: its window %s opens: %s' % (head, x.win.window_text() if x.win is not None else '',
                                                    fmt_hole(Hole(x.qs)))
        if x.kind == 'bad':
            return '%s: refused: %s %s' % (head, x.err.code, x.err.msg)
        if x.kind == 'reset':
            return '%s' % self._reading_text_plain(x)
        return None

    def _reading_text_plain(self, x):
        rows = list(x.coll.rows.values()) if x.coll is not None else []
        val = fmt_hole(Hole(x.qs)) if x.qs else fmt(x.val)
        return '%s %s = %s' % (x.note, rows[0].sid if rows else '', val)

    def _by_day(self, rows):
        if not rows:
            return ['no movement']
        out = []
        cur = None
        for dd, t in sorted(rows, key=lambda r: r[0]):
            if dd != cur:
                out.append('%s:' % dd)
                cur = dd
            out.append('  %s' % t)
        return out

    def point_text(self, p):
        import datetime as _dt
        d = _dt.date.fromordinal(p[0])
        return '(%s, stratum %s, %s)' % (d, p[2], p[3])

    def explain_lines(self, root, ctx):
        if isinstance(root, ExplainLines):
            return ['  ' + l for l in root.lines]
        out = []
        self._no = 0
        self._seen = {}
        for kind, ch in root.children:
            self.explain_node(ch, kind, 1, out)
        return out

    def explain_node(self, tn, kind, depth, out):
        ind = '  ' * depth
        ident = ('row', tn.label) if tn.kind in ('row', 'bandrow') else id(tn)
        # a cause prints in full where it is first printed and as a reference after (L §5.18 "Each cause prints
        # once; a cause met again is referenced by its line"): the print order (rules in canonical order) need not
        # be the evaluation order (beaters first), so a cell traced first may print second
        if ident in self._seen:
            out.append('%s(%s: see [%d])' % (ind, tn.label, self._seen[ident]))
            return
        self._no += 1
        self._seen[ident] = self._no
        no = self._no
        if tn.kind == 'row':
            out.append('%s[%d] row %s = %s' % (ind, no, tn.label, fmt_value(tn.value)))
            for l in tn.lines:
                out.append('%s    %s' % (ind, l))
            return
        if tn.kind == 'bandrow':
            out.append('%s[%d] band row %s' % (ind, no, tn.label))
            return
        if tn.kind == 'def':
            law = getattr(tn, 'law', None)
            out.append('%s[%d] %s = %s%s' % (ind, no, tn.label, fmt_value(tn.value),
                                            (', law date %s' % law) if law is not None else ''))
            for l in tn.lines:
                out.append('%s    %s' % (ind, l))
            states = getattr(tn, 'rule_states', {})
            vers = getattr(tn, 'rule_vers', {})
            whens = dict(getattr(tn, 'when_children', []))
            for lab in sorted(states):
                st = states[lab]
                rv = vers.get(lab)
                extra = ''
                if st == 'won' and rv is not None:
                    vv = rv.v or rv.version
                    if vv is not None:
                        extra = ' (version %s, source %s)' % (vv.id, vv.cite)
                out.append('%s    rule %s: %s%s' % (ind, lab, st, extra))
                w = whens.get(lab)
                if w is not None:
                    for k2, ch in w.children:
                        self.explain_node(ch, k2, depth + 3, out)
                if st == 'won':
                    vc = getattr(tn, 'value_children', None)
                    if vc is not None:
                        for k2, ch in vc.children:
                            self.explain_node(ch, k2, depth + 3, out)
            for k2, ch in tn.children:
                self.explain_node(ch, k2, depth + 2, out)
            return
        out.append('%s[%d] %s' % (ind, no, tn.label))
        for k2, ch in tn.children:
            self.explain_node(ch, k2, depth + 1, out)


def boxes(cells, axes_days):
    """Cover a set of day tuples by maximal boxes of consecutive days, greedily in day order (the first
    window's runs first)."""
    left = set(cells)
    pos = [{d: i for i, d in enumerate(ds)} for ds in axes_days]
    out = []
    for c in sorted(cells):
        if c not in left:
            continue
        lo = list(c)
        hi = list(c)
        for i in range(len(c)):
            while True:
                nxt = pos[i].get(hi[i])
                if nxt is None or nxt + 1 >= len(axes_days[i]):
                    break
                cand = axes_days[i][nxt + 1]
                ok = True
                for t in _box_cells(lo, hi[:i] + [cand] + hi[i + 1:], axes_days, pos, only_axis=i, val=cand):
                    if t not in left:
                        ok = False
                        break
                if not ok:
                    break
                hi[i] = cand
        box = list(zip(lo, hi))
        for t in _box_cells(lo, hi, axes_days, pos):
            left.discard(t)
        out.append(box)
    return out


def _box_cells(lo, hi, axes_days, pos, only_axis=None, val=None):
    import itertools
    ranges = []
    for i in range(len(lo)):
        if only_axis is not None and i == only_axis:
            ranges.append([val])
        else:
            ranges.append(axes_days[i][pos[i][lo[i]]:pos[i][hi[i]] + 1])
    return itertools.product(*ranges)


def is_query_line(e):
    """An ask line's expression that is a query over random values (L §5.16), or a crps over paths."""
    from .eval2 import strip
    x = strip(e)
    if x.k == 'to' and getattr(x, '_form', None) in ('query', 'crps'):
        x = strip(x.e)
    if x.k == 'query' and x.binds is None and not getattr(x, '_dfn', False):
        return True
    if x.k == 'crps' and getattr(x, '_form', None) == 'paths':
        return True
    return False


def split(axes, combos, m):
    """Split a set of combinations (dicts axis -> option) into products, by the axes in id
    order (L §5.5 A set that isn't a product).  Returns lists of (axis, set of options)."""
    if not axes:
        return [[]]
    ax = axes[0]
    rest = axes[1:]
    groups = {}
    order = []
    for o in m.axes[ax].options:
        tails = frozenset(tuple(sorted((k, v) for k, v in c.items() if k != ax)) for c in combos if c[ax] == o)
        if not tails:
            continue
        if tails not in groups:
            groups[tails] = []
            order.append(tails)
        groups[tails].append(o)
    out = []
    for tails in order:
        opts = set(groups[tails])
        for sub in split(rest, [dict(t) for t in tails], m):
            out.append([(ax, opts)] + sub)
    return out
