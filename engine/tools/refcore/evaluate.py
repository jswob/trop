"""Evaluation (L §4, §5.11, §7, §8; E §3.2).

Every value is a cell (item, keys, context).  A context is the law date,
the cut K, the pins, the axis options, the windows clause, the wording pins
and whether the line is partial.  The memo maps cells to results; a result
is a value or a refusal, with its collector: the register entries, the rows
read (inputs-of) and the axes
read.  Refusals are exceptions (TallyError); holes are values.
"""
import datetime
import re
from fractions import Fraction

from . import units as U
from . import ty as T
from .dates import (Period, day_period, month_period, quarter_period, year_period, month_of, quarter_of, year_of,
                    add_days, add_months, age, age_months, overlap_days, last_day, WEEKDAYS, ONE_DAY)
from .errors import TallyError, LoadError, Refusal, Unspecified, NotYet, load_now
from .model import (in_world, set_world, Span, visible, known_first, InputInfo, DefInfo, BandInfo, key_text, _kv,
                    wcut, WindowUndecided, row_known_sid)
from .values import MonthDay  # noqa: E402 (GD-2)
from .values import (fmt_typed, Num, Int, Money, Text, Key, Thing, Code, Rec, Lst, Tup, Iv, Cal, Hole, NONE, NEVER,
                     Special, CURRENCIES, vkey, fmt, fmt_q, Mv, fmt_hole)


def refuse(code, msg):
    return Refusal(code, msg)



# L §4.3 rule 1 and 3: a def, an ask line or a move that reads law-dated facts states at or law (LOAD-LAWDATE, checked at
# load); a read with no law date that reaches one is an interpreter fault, never an open edge of the text
LAWGUARD = '%s read with no law date, which L §4.3 refuses at load (LOAD-LAWDATE): an interpreter guard'

class Ctx:
    __slots__ = ('law', 'cut', 'pins', 'axes', 'windows', 'wording', 'partial', 'path', 'world', 'unc', 'rnd',
                 'walk', 'wp')

    def __init__(self, law, cut, pins=(), axes=(), windows=None, wording=(), partial=False, path=None, world=None,
                 unc=False, rnd=None, walk=None, wp=None):
        self.law = law
        self.cut = wcut(cut, windows)
        self.pins = pins
        self.axes = axes
        self.windows = windows
        self.wording = wording
        self.partial = partial
        self.path = path           # (seed, p): the path of a paths ask (L §5.16), or None
        self.world = world         # the scenario the ask is in (L §6.4), or None for the model
        self.unc = unc             # the line's with uncertain (L §5.18)
        self.rnd = rnd             # the line's random at clause: ('mean', None) or ('quantile', q)
        self.walk = walk           # the walk a running read reads (E §4.2), or None
        self.wp = wp               # the walk point of a decision's evaluation (E §3.2), or None

    def key(self):
        return (self.law, self.cut, self.pins, self.axes, self.windows, self.wording, self.partial, self.path,
                self.world, self.unc, self.rnd, id(self.walk) if self.walk is not None else None, self.wp)

    def tkey(self):
        """The key of the timeline this context reads: everything but the law date, partial, the walk point."""
        return (self.cut, self.pins, self.axes, self.windows, self.wording, self.path, self.world, self.unc,
                self.rnd)

    def but(self, **kw):
        c = Ctx(self.law, self.cut, self.pins, self.axes, self.windows, self.wording, self.partial, self.path,
                self.world, self.unc, self.rnd, self.walk, self.wp)
        for a, b in kw.items():
            setattr(c, a, b)
        if 'cut' in kw or 'windows' in kw:
            c.cut = wcut(c.cut, c.windows)
        return c

    def axis(self, a):
        for x, o in self.axes:
            if x == a:
                return o
        return None


EMPTY_BIND = object()     # a binding that binds nothing (L §5.7 Bindings: a hole day no member could take)


class Coll:
    __slots__ = ('reg', 'rows', 'axes', 'missing', 'trace', 'each')

    def __init__(self):
        self.reg = set()
        self.rows = {}
        self.axes = set()
        self.missing = []
        self.trace = None
        self.each = None        # the windows windows each met here, {window id: (label, days)}: kept with a memo
        #                         cell, so a line that reuses the cell still splits by them (L §4.4, §5.18; MC-A-F1)

    def merge(self, o):
        self.reg |= o.reg
        self.rows.update(o.rows)
        self.axes |= o.axes
        if o.each:
            if self.each is None:
                self.each = {}
            for k, v in o.each.items():
                self.each.setdefault(k, v)


class JournalDict(dict):
    """A cache whose insertions can be recorded (journal, a list, or None), so what a walk attempt cut short by a
    cell still being evaluated outside it cached can be taken back (mreads.m_guarded_walk)."""

    def __init__(self, *a):
        dict.__init__(self, *a)
        self.journal = None

    def __setitem__(self, k, v):
        if self.journal is not None:
            self.journal.append(k)
        dict.__setitem__(self, k, v)


class EachMet(dict):
    """The windows a windows-each run meets (ask.each_rows): each also recorded in the collector being filled, so a
    memo cell that met a window carries it to every line that reads the cell (Coll.each)."""

    def __init__(self, eng):
        dict.__init__(self)
        self.eng = eng

    def setdefault(self, k, v):
        c = self.eng.cur
        if c.each is None:
            c.each = {}
        c.each.setdefault(k, v)
        return dict.setdefault(self, k, v)


class PinVal:
    """A pin with its value (L §5.11 as if)."""
    __slots__ = ('kind', 'item', 'keys', 'fields', 'value', 'text')

    def __init__(self, kind, item, keys, fields, value, text):
        self.kind = kind
        self.item = item
        self.keys = keys          # tuple of key kv or None (*)
        self.fields = fields
        self.value = value
        self.text = text

    def key(self):
        return (self.kind, self.item, self.keys, self.fields, vkey(self.value))

    def __hash__(self):
        return hash(self.key())

    def __eq__(self, o):
        return self.key() == o.key()


class TraceNode:
    def __init__(self, kind, label, value=None):
        self.kind = kind
        self.label = label
        self.value = value
        self.lines = []           # text lines about the node itself
        self.children = []
        self.when_children = []
        self.line_no = None


NOPIN = object()


UNIT_KIND_KEYS = {}   # key -> unit kind, of every reality loaded (keys of unit kinds are units, L §3.2)


def key_clash(a, b):
    """L §3.2 A key that load can't see: values in two keys of one unit kind K meeting where load saw only K are
    refused at run, RUN-DOMAIN `<a> and <b> are in different keys <u> and <u'> of <K>`; None when the units aren't
    two keys of one unit kind."""
    ka = [x for x, e in (a.u or ()) if x in UNIT_KIND_KEYS]
    kb = [x for x, e in (b.u or ()) if x in UNIT_KIND_KEYS]
    for x in ka:
        for y in kb:
            if x != y and UNIT_KIND_KEYS[x] == UNIT_KIND_KEYS[y]:
                return refuse('RUN-DOMAIN', '%s and %s are in different keys %s and %s of %s' % (
                    fmt(a), fmt(b), x, y, UNIT_KIND_KEYS[x]))
    return None


class Engine:
    def __init__(self, m):
        self.m = m
        UNIT_KIND_KEYS.update(getattr(m, 'unit_kind_keys', None) or {})
        self.memo = {}
        self.fill_trace = None      # explain of a fill take: the fills computed, with their takes (E §10)
        self.walk_cells = {}        # (walk, def, keys) -> {walk point: value}: a walk-bound def's cells (E §10)
        self.member_engines = {}    # a joint model's members (L §6.1), one engine each
        self.stack = [Coll()]
        self.active = set()
        self.ask_K = m.on
        self.tracing = False
        self.trace_stack = []
        self.trace_cells = {}
        self.cal_choice = None      # the days chosen for calendars' added windows while a business-day function is
        self.cal_met = None         # evaluated at each day they can take (over_added); the windows met there

    # -- collectors ---------------------------------------------------------
    @property
    def cur(self):
        return self.stack[-1]

    def reg(self, entry):
        self.cur.reg.add(entry)

    def push(self):
        c = Coll()
        self.stack.append(c)
        return c

    def pop(self):
        c = self.stack.pop()
        self.stack[-1].merge(c)
        return c

    def isolated(self, fn):
        """Run fn in a fresh collector that is not merged; return (value or exception, coll)."""
        c = Coll()
        self.stack.append(c)
        try:
            v = fn()
            err = None
        except TallyError as e:
            v = None
            err = e
        finally:
            self.stack.pop()
        return v, err, c

    # -- expressions ----------------------------------------------------------
    def ev(self, n, env, ctx):
        if self.xsub is not None:
            v = self.xsub.get(id(n))
            if v is not None:
                return v
        f = getattr(self, 'e_' + n.k, None)
        if f is None:
            raise NotYet('evaluation of %s' % n.k, n.loc)
        try:
            v = f(n, env, ctx)
            if ctx.partial and isinstance(v, Hole) and v.lin is not False and plain_node(n):
                # a rounding, min, max, a band or a comparison meeting a hole makes a plain hole (L §5.18)
                return Hole(v.qs, False)
            return v
        except HoleSignal as h:
            # one value needed from an undecided interval: the operation is the hole (L §4.4)
            return h.hole
        except WindowUndecided as w:
            # something known in a window that holds the cut, with no windows clause: HOLE-IMPRECISE (L §4.5)
            return w.hole()

    def e_paren(self, n, env, ctx):
        return self.ev(n.e, env, ctx)

    def e_num(self, n, env, ctx):
        t = n._t
        if t.k == 'flex':
            target = getattr(n, '_flexto', None)
            if target is not None:
                return self.typed(Fraction(n._val), target)
            return Num(Fraction(n._val), (), isint=(Fraction(n._val).denominator == 1))
        if t.k == 'money':
            return Money(n._val, t.cur)
        if t.k == 'rate':
            return Num(n._val, n._u, rate=True)
        if t.k == 'int':
            return Int(n.v)
        if t.k == 'key':
            return self.keyval(n._r[1])
        if t.k == 'period':
            return year_period(int(n.v))
        if getattr(n, '_year', None):
            return year_period(int(n.v))
        return Num(n._val, n._u)

    def typed(self, q, target):
        if target.k == 'money':
            return Money(q, target.cur)
        if target.k == 'int':
            return Int(q)
        if T.is_pct(target):
            return Num(q / 100, ())
        if target.k == 'num':
            return Num(q, target.u)
        if target.k == 'rate':
            raise refuse('LOAD-UNIT', 'a bare figure as a rate: a rate states its period (L §3.3)')
        return Num(q, ())

    def e_neg(self, n, env, ctx):
        if n._t.k == 'flex':
            v = self.ev(n.e, env, ctx)
            return Num(-v.q, v.u, money=v.money, isint=v.isint, rate=v.rate)
        v = self.ev(n.e, env, ctx)
        if isinstance(v, Hole):
            if ctx.partial and lin_of(v) is not None:
                return Hole(v.qs, lin_scale(lin_of(v), Num(-1, (), isint=True)))
            return Hole(v.qs)
        self.no_none(v, 'arithmetic')
        if isinstance(v, Num):
            return Num(-v.q, v.u, money=v.money, isint=v.isint, rate=v.rate, enc=enc_neg(v.enc))
        if isinstance(v, Iv):
            return Iv(self.neg1(v.hi), self.neg1(v.lo), None, v.src)
        raise refuse('LOAD-UNIT', 'unary minus on %s' % fmt(v))

    def neg1(self, x):
        return Num(-x.q, x.u, money=x.money, isint=x.isint, rate=x.rate)

    def e_date(self, n, env, ctx):
        return n.v

    def e_month(self, n, env, ctx):
        r = getattr(n, '_r', None)
        if r is not None and r[0] == 'key':
            return self.keyval(r[1])
        return month_period(*n.v)

    def e_quarter(self, n, env, ctx):
        r = getattr(n, '_r', None)
        if r is not None and r[0] == 'key':
            return self.keyval(r[1])
        return quarter_period(*n.v)

    def e_mmdd(self, n, env, ctx):
        return MonthDay(*n.v)

    def e_bool(self, n, env, ctx):
        return n.v

    def e_str(self, n, env, ctx):
        return Text(n.v)

    def e_none(self, n, env, ctx):
        return NONE

    def e_never(self, n, env, ctx):
        return NEVER

    def e_cut(self, n, env, ctx):
        return ctx.cut

    def e_code(self, n, env, ctx):
        return n._r[1]

    def e_cur(self, n, env, ctx):
        return Thing(n.v)

    def e_key(self, n, env, ctx):
        r = n._r
        if r[0] == 'key':
            return self.keyval(r[1])
        if r[0] == 'code':
            return r[1]
        raise LoadError('LOAD-KEY', 'ambiguous key %s' % r[1], n.loc)

    def keyval(self, k):
        """A key as a value (L §4.2 Keys as values)."""
        if isinstance(k, int) and not isinstance(k, bool):
            return Int(k)
        return k

    def e_record(self, n, env, ctx):
        out = []
        for f, v in n.fields:
            if v.k == 'hole':
                out.append((f, Hole([(v.text, '')])))
            else:
                out.append((f, self.ev(v, env, ctx)))
        return Rec(out)

    def e_list(self, n, env, ctx):
        return Lst([self.ev(x, env, ctx) for x in n.items])

    def e_name(self, n, env, ctx):
        r = n._r
        k = r[0]
        if k == 'var':
            v = env[r[1]]
            return v
        if k == 'read':
            return self.read(n, r, [], env, ctx)
        if k == 'axisval':
            return Key(r[1], self.axis_option(r[1], ctx))
        if k == 'key':
            return self.keyval(r[1])
        if k == 'code':
            return r[1]
        if k == 'cut':
            return ctx.cut
        if k == 'keylit':
            raise LoadError('LOAD-KEY', 'ambiguous key %s' % r[1], n.loc)
        if k == 'place':
            return self.e_place(n, env, ctx)
        if k == 'moveref':
            return self.e_moveref(n, env, ctx)
        raise NotYet('the name %s' % n.id, n.loc)

    def axis_option(self, a, ctx):
        o = ctx.axis(a)
        if o is None:
            raise NotYet('axis %s read outside the axes the ask reaches: an interpreter guard (L §5.5: the cone holds every '
                         'axis a run reads)' % a)
        self.cur.axes.add(a)
        return o

    def member_read(self, n, env, ctx):
        """a.item[keys] in a joint model: the member's item, evaluated in the member at the reader's law date and cut
        (L §6.1 Joint models: the member is read as it is, unchanged)."""
        _, alias, kind, iid = n._r
        mm = self.m.joint_members[alias]
        me = self.member_engines.get(alias)
        if me is None:
            me = self.member_engines[alias] = Engine(mm)
        me.ask_K = self.ask_K
        info = mm.inputs[iid] if kind == 'input' else mm.defs[iid]
        keynodes = n.keys if n.k == 'index' else []
        keys = []
        for kn, conv, b in zip(keynodes, n._convs, info.binds):
            if conv[0] == 'lit':
                keys.append(conv[1])
                continue
            v = self.ev(kn, env, ctx)
            if isinstance(v, Hole):
                return v
            keys.append(me.to_key(v, conv, b.dom, kn))
        c = Coll()
        me.stack.append(c)
        try:
            mctx = Ctx(ctx.law, ctx.cut)
            v = me.read_input(info, keys, mctx) if kind == 'input' else me.read_def(info, keys, mctx)
        finally:
            me.stack.pop()
            # the member's register, its items named a.item (L §6.1: they are read as a.item)
            c2 = Coll()
            for e in c.reg:
                if e[0] == 'assumed' and len(e) > 2 and e[2]:
                    c2.reg.add((e[0], e[1], '%s.%s' % (alias, e[2])))
                elif e[0] in ('held', 'unfulfilled', 'open', 'uncertain', 'anchor', 'pad', 'adjustment', 'left out',
                              'missing') and isinstance(e[1], str):
                    c2.reg.add((e[0], '%s.%s' % (alias, e[1])) + tuple(e[2:]))
                else:
                    c2.reg.add(e)
            for sid, r in c.rows.items():
                c2.rows['%s.%s' % (alias, sid)] = r
            self.cur.merge(c2)
        return v

    def member_band(self, n, env, ctx):
        """a.b(x) in a joint model: the member's band, read in the member at the reader's law date and cut."""
        _, alias, bid = n._r
        mm = self.m.joint_members[alias]
        me = self.member_engines.get(alias)
        if me is None:
            me = self.member_engines[alias] = Engine(mm)
        me.ask_K = self.ask_K
        info = mm.bands[bid]
        x = self.ev(n.args[0], env, ctx)
        if isinstance(x, Hole):
            return Hole(x.qs)
        self.no_none(x, 'a band argument')
        c = Coll()
        me.stack.append(c)
        try:
            mctx = Ctx(ctx.law, ctx.cut)
            rows, hole = me.band_rows(info, None, mctx)
            if hole is not None:
                v = hole
            else:
                xk = band_x(x, mm)
                v = None
                for r, t in rows:
                    if r.holds(xk):
                        v = me.band_value(info, None, r, t, x, mctx)
                        break
                if v is None:
                    v = me.band_no_row(info, None, x, mctx)
        finally:
            me.stack.pop()
            c2 = Coll()
            for sid, r in c.rows.items():
                c2.rows['%s.%s' % (alias, sid)] = r
            c2.reg = set(c.reg)
            self.cur.merge(c2)
        return v

    def e_index(self, n, env, ctx):
        k = n._r[0]
        if k == 'member':
            return self.member_read(n, env, ctx)
        if k == 'fillread':
            return self.e_fillread(n, env, ctx)
        if k == 'place':
            return self.e_place(n, env, ctx)
        if k == 'moveref':
            return self.e_moveref(n, env, ctx)
        if k == 'obsrow':
            return self.e_obsrow(n, env, ctx)
        return self.read(n, n._r, n.keys, env, ctx)

    def e_field(self, n, env, ctx):
        r = getattr(n, '_r', None)
        if r is not None and r[0] == 'member':
            return self.member_read(n, env, ctx)
        if r is not None and r[0] == 'read':
            return self.read(n, r, [], env, ctx)
        if r is not None and r[0] == 'fillread':
            return self.e_fillread(n, env, ctx)
        if r is not None and r[0] == 'mvfield':
            return self.mv_field(n, env, ctx)
        if r is not None and r[0] == 'place':
            return self.e_place(n, env, ctx)
        if r is not None and r[0] == 'moveref':
            return self.e_moveref(n, env, ctx)
        v = self.ev(n.obj, env, ctx)
        if isinstance(v, Hole):
            return v
        if v is NONE:
            raise refuse('RUN-NONE', '%s is none here: a field read of it' % _ntxt(n.obj))
        if isinstance(v, Rec):
            return v.get(n.name)
        raise refuse('LOAD-FIELD', '%s is not a field of %s' % (n.name, fmt(v)))

    # -- reads ------------------------------------------------------------------
    def key_values(self, info, keynodes, convs, env, ctx):
        """Evaluate a read's keys; a hole key makes the read a hole."""
        keys = []
        holes = []
        benv = {}
        for kn, conv, b in zip(keynodes, convs, info.binds):
            if conv[0] == 'lit':
                k = conv[1]
            elif conv[0] == 'axis':
                k = Key(conv[1], self.axis_option(conv[1], ctx))
            else:
                v = self.ev(kn, env, ctx)
                if isinstance(v, Hole):
                    holes.append(v)
                    keys.append(None)
                    continue
                try:
                    k = self.to_key(v, conv, b.dom, kn)
                except TallyError as e:
                    if e.code != 'RUN-KEY':
                        raise
                    dr = getattr(kn, '_draw', None)
                    if dr is not None:
                        # L §5.16, §8.4: "draw <n> of <x> passes the last slot of <index>"
                        n = self.ev(dr[0], env, ctx)
                        raise refuse('RUN-KEY', 'draw %s of %s passes the last slot of %s' % (fmt(n), dr[1], dr[2]))
                    if info.__class__.__name__ == 'DefInfo' and b.dom[0] == 'index' and \
                            self.m.indexes[b.dom[1]].pk == 'day' and isinstance(v, datetime.date):
                        # L §8.4: "<d> is outside the days of <def>" (a def keyed by an index of days, read at a
                        # computed day, §5.11)
                        raise refuse('RUN-KEY', '%s is outside the days of %s' % (v, info.id))
                    raise
            if b.dom[0] == 'range':
                h = self.range_member(k, b, benv, ctx, kn)
                if h is not None:
                    holes.append(h)
                    keys.append(None)
                    continue
            else:
                try:
                    self.check_key(k, b.dom, ctx)
                except TallyError as e:
                    dr = getattr(kn, '_draw', None)
                    if e.code != 'RUN-KEY' or dr is None:
                        raise
                    # L §5.16, §8.4: "draw <n> of <x> passes the last slot of <index>"
                    n = self.ev(dr[0], env, ctx)
                    raise refuse('RUN-KEY', 'draw %s of %s passes the last slot of %s' % (fmt(n), dr[1], dr[2]))
            benv[b.var] = k
            keys.append(k)
        if holes:
            return None, Hole(set().union(*[h.qs for h in holes]))
        return keys, None

    def check_range_lits(self):
        """The keys written at a stepped range's position in rows, fulfils and same lines (loader.register_range_lit):
        each "must be one of the range's values ... (LOAD-KEY for a literal)" (L §2.5). Checked once, at the model's
        on date; ends that can't be read there (a hole, a law-dated row) leave the check to the key's own read."""
        m = self.m
        lits = getattr(m, 'range_lits', None)
        if not lits or getattr(m, '_range_lits_done', False):
            return
        m._range_lits_done = True
        ctx = Ctx(None, m.on)
        for b, value, kn, loc, earlier in lits:
            d = b.dom[1] if len(b.dom) > 1 else b.node.dom
            if not range_is_literal(d):
                # L §2.5 (round 14): LOAD-KEY "for a literal where the range's ends and step are literals, which load
                # sees; otherwise RUN-KEY where the key is read, and a fulfils or a same side at a key the range
                # doesn't hold names an instance that doesn't exist, RUN-RECORD"
                continue
            try:
                self.range_member(value, b, dict(earlier), ctx, kn)
            except TallyError as e:
                if e.code == 'LOAD-KEY':
                    raise LoadError('LOAD-KEY', e.msg, loc)
                continue

    def range_member(self, k, b, benv, ctx, kn):
        """A key at a position keyed by a stepped range "must be one of the range's values, a + i·s <= b for a whole
        number i >= 0 (LOAD-KEY for a literal, RUN-KEY at run)" (L §2.5). A hole end makes the read that hole."""
        # a move's bind keeps the range node on its syntax (check3 sets its dom to ('range',))
        d = b.dom[1] if len(b.dom) > 1 else b.node.dom
        a = self.ev(d.a, benv, ctx)
        z = self.ev(d.b, benv, ctx)
        s = self.ev(d.step, benv, ctx) if d.step is not None else None
        hs = [x for x in (a, z, s) if isinstance(x, Hole)]
        if hs:
            return merge(*hs)
        memo = self.__dict__.setdefault('_range_memo', {})
        mk = (id(d), vkey(a), vkey(z), vkey(s) if s is not None else None)
        vals = memo.get(mk)
        if vals is None:
            vals = memo[mk] = set(vkey(x) for x in self.range_values(a, z, s, b))
        kk = Int(k) if isinstance(k, int) and not isinstance(k, bool) else k
        if vkey(kk) not in vals:
            code = 'LOAD-KEY' if getattr(kn, '_rangelit', False) and range_is_literal(d) else 'RUN-KEY'
            raise refuse(code, '%s is not a value of %s .. %s%s' % (fmt(kk), fmt(a), fmt(z),
                                                                     (' step %s' % fmt(s)) if s is not None else ''))
        return None

    def to_key(self, v, conv, dom, kn):
        m = self.m
        c = conv[0]
        if v is NONE:
            raise refuse('RUN-NONE', '%s is none here: a key' % _ntxt(kn))
        if isinstance(v, Iv):
            raise self.imprecise(v, 'a key')
        if c == 'same':
            if isinstance(v, Num):
                return self.int_of(v)
            if isinstance(v, OutKey):
                raise refuse('RUN-KEY', '%s is outside %s' % (v.text, v.index))
            return v
        if c == 'int':
            if isinstance(v, OutKey):
                return v.value
            return self.int_of(v)
        if c == 'holding' or c == 'period':
            ix = m.indexes[dom[1]]
            if isinstance(v, OutKey):
                v = v.value
            if isinstance(v, datetime.date):
                k = ix.holding(v)
                if k is None:
                    raise refuse('RUN-KEY', '%s is outside %s' % (v, ix.id))
                return k
            if isinstance(v, Period):
                for k in ix.keys:
                    if isinstance(k, datetime.date):
                        if v.kind == 'day' and k == v.start:
                            return k
                        continue
                    if k == v:
                        return k
                    if k.start <= v.start and v.end <= k.end:
                        return k
                if ix.keys and any(isinstance(k, Period) and k.start < v.end and v.start < k.end for k in ix.keys):
                    if all(isinstance(k, Period) and (k.end - k.start) < (v.end - v.start) for k in ix.keys):
                        raise refuse('LOAD-KEY', 'the period %s is longer than the keys of %s: it holds no key'
                                     % (fmt(v), ix.id))
                    # L §4.6 Reading at a date (round 19, GD-24, GD-27): "A period no longer than the keys that no
                    # key's period holds (it lies across two keys, ...) is refused at load where load sees the period
                    # and the keys (LOAD-KEY, §8.3), and otherwise at that value (RUN-DOMAIN)": what load sees is
                    # refused there (check3.m_check_period_keys), the rest here
                    raise refuse('RUN-DOMAIN', 'the period %s lies across two keys of %s: no key\'s period holds it'
                                 % (fmt(v), ix.id))
                raise refuse('RUN-KEY', '%s is outside %s' % (fmt(v), ix.id))
        if c == 'thing':
            if isinstance(v, Mv) and conv[1] in m.kinds and m.kinds[conv[1]].spec == 'event':
                # a movement as the key of an event kind (L §5.7: a movement is a thing)
                return v
            if not isinstance(v, Thing):
                raise refuse('RUN-KEY', '%s is not a key of %s' % (fmt(v), conv[1]))
            if not m.thing_in(v.name, conv[1]):
                raise refuse('RUN-KEY', '%s is not a key of %s' % (v.name, conv[1]))
            return v
        if c == 'keylist':
            from .loader import resolve_key_literal
            node = v.node if isinstance(v, ListKey) else None
            if node is None:
                # a key-list variable bound to an integer key takes the index where it is used (L §2.5)
                if isinstance(v, Num):
                    return self.int_of(v)
                return v
            return resolve_key_literal(m, node, conv[1], None)
        if c == 'value':
            return v
        raise NotYet('key conversion %s' % c)

    def int_of(self, v):
        if v is NONE:
            # none can only be tested (L §3.1)
            raise refuse('RUN-NONE', 'a value is none here: an integer argument')
        if isinstance(v, Num):
            if v.q.denominator != 1:
                raise refuse('RUN-NOT-INT', '%s is not an integer' % fmt(v))
            return int(v.q)
        return v

    def check_key(self, k, dom, ctx):
        m = self.m
        if dom[0] == 'index':
            ix = m.indexes[dom[1]]
            if not ix.has(k):
                raise refuse('RUN-KEY', '%s is outside %s' % (key_text(k), ix.id))
            if not ix.visible(k, ctx.cut):
                raise refuse('RUN-KEY', '%s was added after the cut' % key_text(k))
        elif dom[0] == 'kind' and isinstance(k, Mv):
            # a key of an event kind is a movement (L §5.7, §5.11): one of the kind or its subkinds
            src = self.mv_record(k, ctx) if k.src is None else k.src
            if src is None or src.kind not in m.kinds[dom[1]].subtree(m):
                raise refuse('RUN-KEY', '%s is not a movement of %s' % (k.id, dom[1]))
        elif dom[0] == 'kind':
            t = m.things.get(k.name) if isinstance(k, Thing) else None
            if t is None:
                raise refuse('RUN-KEY', '%s is not a key of %s' % (key_text(k), dom[1]))
            if not m.thing_seen(t, ctx.cut):
                raise refuse('RUN-KEY', '%s was added after the cut' % k.name)
        elif dom[0] == 'axis':
            pass

    def read(self, n, r, keynodes, env, ctx):
        kind, iid = r[1], r[2]
        info = self.m.inputs[iid] if kind == 'input' else self.m.defs[iid]
        keys, hole = self.key_values(info, keynodes, n._convs, env, ctx)
        if hole is not None:
            return hole
        if getattr(n, '_sample', False):
            return self.sample_read(info, keys, ctx)
        bw = getattr(n, '_world', None)
        if bw is None and n.k == 'index':
            bw = getattr(n.obj, '_world', None)
        if bw is not None and bw[1] != ctx.world:
            # a scenario row's prefixed name is read in the base (L §6.4); its keys were evaluated here
            ctx = ctx.but(world=bw[1])
            old = set_world(self.world_chain(bw[1]))
            try:
                if kind == 'input':
                    return self.read_input(info, keys, ctx)
                return self.read_def(info, keys, ctx)
            finally:
                set_world(old)
        if kind == 'input':
            return self.read_input(info, keys, ctx)
        return self.read_def(info, keys, ctx)

    def origin(self, info, keys):
        if not keys:
            return info.id
        return '%s[%s]' % (info.id, ', '.join(key_text(k) for k in keys))

    def pin_value(self, info, keys, ctx):
        for p in ctx.pins:
            if p.kind not in ('set', 'solve') or p.item != info.id:
                continue
            if p.keys is not None:
                ok = True
                for pk, k in zip(p.keys, keys):
                    if pk is not None and pk != _kv(k):
                        ok = False
                        break
                if not ok:
                    continue
            if p.fields:
                return ('fields', p)
            if p.kind == 'set':
                self.reg(('pinned', p.text))
            return p.value
        return NOPIN

    def apply_field_pins(self, info, keys, ctx, v):
        """A pin on a record field pins that field (L §5.11)."""
        for p in ctx.pins:
            if p.kind != 'set' or p.item != info.id or not p.fields:
                continue
            if p.keys is not None and any(pk is not None and pk != _kv(k) for pk, k in zip(p.keys, keys)):
                continue
            if not isinstance(v, Rec):
                continue
            v = set_field(v, p.fields, p.value)
            self.reg(('pinned', p.text))
        return v

    def time_point(self, info, keys, ctx):
        """The day a read's spans are tested at: the first day of the period of the time
        key (L §4.6 step 1), else the law date."""
        tp = info.time_pos
        # L §4.6 step 1 (round 8b): "the period of x's time key when x is an input keyed by one period index, or d
        # otherwise (a rule, a band, and an input keyed by no period index or by more than one, are tested at the law
        # date d)": a def's givens and rules, and an input keyed by two period indexes, are tested at d
        from .model import InputInfo
        if tp == 'multi' or not isinstance(info, InputInfo):
            return ctx.law, None
        if tp is not None:
            k = keys[tp]
            if isinstance(k, datetime.date):
                return k, k
            return k.start, k
        return ctx.law, None

    def need_law(self, d, info):
        if d is None and not info.lawless():
            raise NotYet(LAWGUARD % ('a read of %s' % info.id))

    # -- the read rule (L §4.6) --------------------------------------------------------
    def row_matches(self, r, keys, info):
        for p, k, b in zip(r.pattern, keys, info.binds):
            if p[0] == '*':
                continue
            if p[0] == 'k':
                if _kv(p[1]) != _kv(k):
                    return False
            else:
                lo, hi = self.key_ord(p[1], b.dom), self.key_ord(p[2], b.dom)
                x = self.key_ord(k, b.dom)
                if not (lo <= x <= hi):
                    return False
        return True

    def key_ord(self, k, dom):
        m = self.m
        if dom[0] == 'index':
            return m.indexes[dom[1]].pos(k)
        if dom[0] == 'axis':
            return m.axes[dom[1]].options.index(k.name)
        if dom[0] == 'kind':
            return m.kind_order_key(m.things[k.name], dom[1], self.ask_K)
        return k

    def unstated_ok(self, r, keys, info):
        """A key a row leaves unstated, or covers with *, is one its index or kind had on the
        row's known date (L §4.6 step 1, §6.2)."""
        m = self.m
        rk = known_first(r.known)
        for p, k, b in zip(r.pattern, keys, info.binds):
            if p[0] != '*':
                continue
            dom = b.dom
            if dom[0] == 'index':
                ix = m.indexes[dom[1]]
                v = ix.added.get(_kv(k))
                if v is not None and known_first(v.known) > rk:
                    return False
            elif dom[0] == 'kind':
                t = m.things.get(k.name)
                if t is None:
                    return False
                fk = None
                sub = m.kinds[dom[1]].subtree(m)
                for e in t.entries:
                    if e.kind in sub and not e.replace:
                        kk = known_first(e.version.known) if e.version is not None else datetime.date.min
                        fk = kk if fk is None else min(fk, kk)
                if fk is None or fk > rk:
                    return False
        return True

    def row_visible(self, r, ctx):
        if not in_world(r.version):
            return False
        return self.known_vis(r.known, row_known_sid(r), 'a row', ctx)

    def known_vis(self, known, sid, what, ctx):
        """Whether something known on `known` is seen at the cut. A window that holds the cut "states a windows clause
        or is HOLE-IMPRECISE ... This holds for everything the version or row brings: its rows, band tables and rules"
        (L §4.5)."""
        vis = visible(known, ctx.cut, sid or what)
        if vis is None:
            lo, hi = known
            raise HoleSignal(Hole([('date within window(%s, %s) not precise enough for the cut %s' % (
                lo, hi, ctx.cut), sid)]))
        return vis

    def wording_ok(self, version, item, ctx, own=None):
        """Inside a `wording D for [...]` pin naming the item: the candidate entered into force by D (L §4.3), row
        by row (L §5.3: from the row's own time where it states one)."""
        for it, d in ctx.wording:
            if it == item:
                return version.enters_at(ctx.cut, own) <= d
        return True

    def choose(self, cands, info, what):
        """Replacement and precedence among candidates (L §4.6 steps 2-4)."""
        if not cands:
            return None
        if getattr(self.m, 'joint', None) is not None:
            from .moves import row_owner
            owners = sorted(set(row_owner(self.m, c) for c in cands))
            if len(owners) > 1:
                # facts about one thing from two members are read together; the joint model's own row settles
                # them; two that disagree are a hole naming both rows (L §6.1, HOLE-SAME)
                if '' in owners:
                    return self.choose([c for c in cands if row_owner(self.m, c) == ''], info, what)
                wins = [self.choose([c for c in cands if row_owner(self.m, c) == o], info, what) for o in owners]
                if all(vkey_row(w) == vkey_row(wins[0]) for w in wins):
                    return wins[0]
                import copy
                from .joint import shown
                a, b = wins[0], wins[1]
                q = '%s and %s are one, but %s differs: %s (%s) and %s (%s)' % (
                    shown(a.sid or ''), shown(b.sid or ''), shown(info.id), fmt_rowval(a), shown(a.sid or ''),
                    fmt_rowval(b), shown(b.sid or ''))
                r = copy.copy(a)
                r.val = ('hole', q)
                return r
        from .loader import pattern_norm, row_layer
        keep = []
        for c in cands:
            pc = pattern_norm(c.pattern)
            beaten = False
            for o in cands:
                if o is not c and o.replace and pattern_norm(o.pattern) == pc and \
                        (row_layer(self.m, o) > row_layer(self.m, c) or (row_layer(self.m, o) == row_layer(self.m, c)
                                                                         and known_first(o.known) > known_first(c.known))):
                    # in a scenario, its replace beats the base's row whatever their known dates (L §4.6 step 2)
                    beaten = True
                    break
            if not beaten:
                keep.append(c)
        from .loader import narrower, row_layer
        # 3.1 inside a scenario, the row from the higher layer (L §4.6)
        top = max(row_layer(self.m, c) for c in keep)
        keep = [c for c in keep if row_layer(self.m, c) == top]
        # 3.2 the narrower row (round 11b): a row another remaining row is narrower than drops out
        if len(keep) > 1 and not isinstance(info, BandInfo):
            keep = [c for c in keep if not any(o is not c and narrower(self.m, info, o.pattern, c.pattern)
                                               for o in keep)]
        if len(keep) > 1:
            spans = [c for c in keep if c.instant is not None or not c.span(self._K).always]
            if spans:
                keep = spans
        if len(keep) > 1:
            raise refuse('LOAD-OVERLAP', 'rows %s and %s of %s tie' % (keep[0].loc, keep[1].loc, info.id))
        return keep[0]

    def candidates(self, rows, keys, d, ctx, info, instant=None, last_before=None):
        out = []
        self._K = ctx.cut
        for r in rows:
            if not self.row_matches(r, keys, info):
                continue
            if not self.unstated_ok(r, keys, info):
                continue
            if not self.row_visible(r, ctx):
                continue
            own = r.own_span if r.own_span is not None else (r.instant[0] if r.instant is not None and
                                                                 isinstance(r.instant[0], datetime.date) else None)
            if not self.wording_ok(r.version, info.id, ctx, own):
                continue
            if r.instant is not None:
                if instant is not None and r.instant[0] != instant:
                    continue
                if last_before is not None and not (isinstance(r.instant[0], datetime.date)
                                                     and r.instant[0] <= last_before):
                    continue
            else:
                sp = r.span(ctx.cut)
                if sp is None:
                    continue
                if d is None and last_before is None:
                    if not sp.always:
                        raise NotYet(LAWGUARD % ('a read of %s' % info.id))
                    out.append(r)
                    continue
                if last_before is not None:
                    if sp.always or sp.start is None or sp.start > last_before:
                        continue
                elif not sp.contains(d):
                    continue
            out.append(r)
        return out

    def row_value(self, r, info, keys, ctx, d, period):
        """A chosen row's value, with its register entries (L §8.2)."""
        self.cur.rows[r.sid] = r
        if self.tracing:
            self.trace_row(r, info, keys)
        v = r.val
        org = self.origin(info, keys)
        reason = r.assumed
        if reason is not None:
            self.reg(('assumed', reason, org))
        if r.version.assumed is not None:
            self.reg(('assumed', r.version.assumed, org))
        for b, k in zip(info.binds, keys):
            if b.dom[0] == 'index':
                ix = self.m.indexes[b.dom[1]]
                if ix.assumed is not None:
                    self.reg(('assumed', '%s: %s' % (ix.id, ix.assumed), ''))  # L §5.6: assumed: <index>: <reason>
        if isinstance(v, tuple) and v and v[0] == 'hole':
            return Hole([(v[1], org)])
        if isinstance(v, tuple) and v and v[0] == 'node':
            # a dist row gives the distribution by an expression (L §5.16)
            v = self.ev(v[1], {}, ctx)
        elif isinstance(v, tuple) and v and v[0] == 'sexpr':
            # a scenario row over the base's facts, evaluated at the row's keys and law date (L §6.4)
            ak = ('sexpr', r.sid, tuple(vkey(k) for k in keys), d, ctx.key())
            act = self.__dict__.setdefault('_sexpr_active', set())
            if ak in act:
                raise refuse('RUN-CYCLE', '%s reads itself through its scenario row %s' % (org, r.sid))
            act.add(ak)
            try:
                v = self.ev(v[1], {}, ctx.but(law=d) if d is not None else ctx)
            finally:
                act.discard(ak)
            if isinstance(v, TallyError):
                raise v
        # held (L §8.2): a row whose span has a start and no end, read at a law date after K
        if r.instant is None and d is not None:
            sp = r.span(ctx.cut)
            if not sp.always and sp.start is not None and sp.end is None and d > self.ask_K:
                per = fmt(period) if period is not None else d.isoformat()
                self.reg(('held', '%s = %s (%s), read for %s' % (org, fmt(v) if not isinstance(v, Hole) else '?',
                                                                  r.version.id, per)))
        if isinstance(v, Iv):
            v = Iv(v.lo, v.hi, v.days, v.src, origin=(r.sid, org))
            v = self.window_choice(v, ctx)
        if isinstance(v, Num) and any(b.startswith('$') for b, e in v.u):
            # a per-unit price num PLN/u takes the key it is read at (L §3.2)
            env = {b.var: key_text(k) for b, k in zip(info.binds, keys)}
            v = Num(v.q, U.subst(v.u, env), money=v.money, isint=v.isint, rate=v.rate)
        return v

    def window_choice(self, v, ctx):
        ws = ctx.windows
        if not (isinstance(ws, tuple) and ws and ws[0] == 'each'):
            # the line's windows each rides on its cut into contexts made afresh (a walk's, an instance's)
            ws = getattr(ctx.cut, 'windows', None)
        if isinstance(ws, tuple) and ws and ws[0] == 'each':
            # windows each: the day the run assigns to this window (L §4.4, §5.18)
            wi = window_id(v)
            if wi is None:
                return v
            wid, label, days = wi
            for x, d in ws[1]:
                if x == wid:
                    return d
            if self.each_met is not None:
                self.each_met.setdefault(wid, (label, days))
            return days[0]
        return v

    def no_candidate(self, info, keys, ctx, d, period, calendar=False):
        """The gap rule: a hole, or none outside a lifespan (L §4.6 step 5)."""
        m = self.m
        for b, k in zip(info.binds, keys):
            if b.dom[0] == 'kind' and isinstance(k, Thing):
                t = m.things.get(k.name)
                if t is None:
                    continue
                if d is not None and m.same_sides(t, m.kinds[b.dom[1]].subtree(m)) is not None:
                    # a thing stated same (L §6.1): outside the lifespan both sides state, none; where they disagree,
                    # HOLE-SAME (raised by alive_on, a hole at the read)
                    if not m.alive_on(t, b.dom[1], d, ctx.cut, getattr(b, '_owner', None)):
                        return NONE
                    continue
                sp = m.member_span(t, b.dom[1], ctx.cut)
                if d is not None and (sp is None or not sp.contains(d)):
                    return NONE
        org = self.origin(info, keys)
        if calendar:
            return Hole([('calendar %s doesn\'t cover %s' % (info.id, d.year), org)])
        if info.question:
            return Hole([(info.question, org)])
        ks = ', '.join(key_text(k) for k in keys)
        if period is None and d is None:
            # "Read with no law date (an input whose rows all govern always, §4.3 rule 2), the question ends at no
            # value: x at k: no value, x: no value" (L §4.6 step 5)
            if keys:
                return Hole([('%s at %s: no value' % (info.id, ks), org)])
            return Hole([('%s: no value' % info.id, org)])
        if period is not None:
            return Hole([('%s at %s: no value for %s' % (info.id, ks, fmt(period)), org)])
        if keys:
            return Hole([('%s at %s: no value at %s' % (info.id, ks, d), org)])
        return Hole([('%s: no value at %s' % (info.id, d), org)])

    def index_assumed(self, info):
        """An index with `assumed`: every figure keyed by it, and every result that reaches one, lists
        `assumed: <index>: <reason>` (L §5.6, §8.2), whatever the figure's value (a hole included)."""
        for b in info.binds:
            dom = b.dom
            if dom[0] == 'index':
                ix = self.m.indexes.get(dom[1])
                if ix is not None and ix.assumed is not None:
                    self.reg(('assumed', '%s: %s' % (ix.id, ix.assumed), ''))

    def read_input(self, info, keys, ctx, calendar=False, probe=False):
        self.index_assumed(info)
        pv = self.pin_value(info, keys, ctx)
        if pv is not NOPIN and not (isinstance(pv, tuple) and pv[0] == 'fields'):
            return pv
        d, period = self.time_point(info, keys, ctx)
        if info.reading:
            return self.read_reading(info, keys, ctx, probe)
        sr = getattr(self.m, 'same_rows', None)
        if sr:
            skey = (info.id, tuple(_kv(k) for k in keys))
            if skey in sr:
                return self.read_same_row(info, keys, ctx, skey, period)
        self.need_law(d, info)
        try:
            cands = self.candidates(info.rows, keys, d, ctx, info)
        except HoleSignal as h:
            return h.hole
        r = self.choose(cands, info, 'input')
        if r is None:
            v = self.no_candidate(info, keys, ctx, d, period, calendar)
            if probe and isinstance(v, Hole):
                return ('missing',)
            return v
        v = self.row_value(r, info, keys, ctx, d, period)
        if probe and isinstance(v, Hole) and isinstance(r.val, tuple) and r.val[0] == 'hole':
            return ('missing',)
        if info.prices is not None:
            v = self.prices(info, keys, ctx, v, d)
        v = self.apply_field_pins(info, keys, ctx, v)
        return v

    def read_same_row(self, info, keys, ctx, skey, period):
        """Two rows stated same (L §6.1): read together.  A row of the joint model's own states their value; else
        both sides are read; where only one states the fact, it is the fact; where they disagree, a hole naming both
        rows (HOLE-SAME)."""
        from .joint import shown
        m = self.m
        pinf, pkeys, ta, tb = m.same_rows[skey]
        pkey = (pinf.id, tuple(_kv(k) for k in pkeys))
        own = list(m.same_settle.get(skey, [])) + list(m.same_settle.get(pkey, []))

        def side(inf, ks, rows):
            d, per = self.time_point(inf, ks, ctx)
            self.need_law(d, inf)
            cands = self.candidates(rows, ks, d, ctx, inf)
            return self.choose(cands, inf, 'input'), d, per
        try:
            if own:
                d, per = self.time_point(info, keys, ctx)
                cands = []
                for r in own:
                    ks = keys if r.target == info.id or getattr(r.target, 'id', None) == info.id else pkeys
                    tinf = info if ks is keys else pinf
                    cands += self.candidates([r], ks, d, ctx, tinf)
                r = self.choose(cands, info, 'input') if cands else None
                if r is not None:
                    return self.row_value(r, info, keys, ctx, d, per)
            ra, da, pa = side(info, keys, info.rows)
            rb, db, pb = side(pinf, pkeys, pinf.rows)
        except HoleSignal as h:
            return h.hole
        if ra is None and rb is None:
            return self.no_candidate(info, keys, ctx, da, pa)
        va = self.row_value(ra, info, keys, ctx, da, pa) if ra is not None else None
        vb = self.row_value(rb, pinf, pkeys, ctx, db, pb) if rb is not None else None
        if va is None:
            return vb
        if vb is None:
            return va
        if isinstance(va, Hole) or isinstance(vb, Hole):
            return merge(*[x for x in (va, vb) if isinstance(x, Hole)])
        from .values import vkey
        if vkey(va) == vkey(vb):
            return va
        day = da if da is not None else db
        q = '%s and %s are one, but the value differs on %s: %s (%s) and %s (%s)' % (
            shown(ta), shown(tb), day if day is not None else 'every day', fmt(va), shown(ra.sid or ''), fmt(vb),
            shown(rb.sid or ''))
        return Hole([(q, '%s, %s' % (shown(ta), shown(tb)))])

    def prices(self, info, keys, ctx, v, d):
        """A figure in Y prices (L §3.4)."""
        if isinstance(v, Hole) or v is NONE:
            return v
        Y = info.prices
        if info.prices_by is None:
            if d.year == Y:
                return v
            return Hole([('%s is in %d prices; no index moves it to %s' % (info.id, Y, d), self.origin(info, keys))])
        hit = self.m.lookup(info.prices_by, info.regime)
        if hit is None or hit[0] not in ('input', 'def') or len(hit[1].binds) != 1:
            raise LoadError('LOAD-NAME', 'prices by %s: not an input or a def keyed by one time index' %
                            info.prices_by)
        idx = hit[1]
        b = idx.binds[0]
        if b.dom[0] != 'index' or self.m.indexes[b.dom[1]].pk is None:
            raise LoadError('LOAD-KEY', 'prices by %s: not keyed by a time index' % idx.id)
        ix = self.m.indexes[b.dom[1]]
        ky = None
        for k in ix.keys:
            if isinstance(k, Period) and k.kind == 'year' and k.label() == '%04d' % Y:
                ky = k
        if ky is None:
            raise LoadError('LOAD-KEY', 'prices %d by %s needs an index with a key for the year %d' % (Y, idx.id, Y))
        kd = ix.holding(d)
        if kd is None:
            raise refuse('RUN-KEY', '%s is outside %s' % (d, ix.id))
        if hit[0] == 'def':
            # a level built from an inflation row is a def (L §3.4)
            a = self.read_def(idx, [kd], ctx)
            bb = self.read_def(idx, [ky], ctx)
        else:
            a = self.read_input(idx, [kd], ctx)
            bb = self.read_input(idx, [ky], ctx)
        if isinstance(a, Hole) or isinstance(bb, Hole):
            for x in (a, bb):
                if not isinstance(x, Hole):
                    self.no_none(x, 'arithmetic')
            return merge(v, a, bb)
        self.no_none(a, 'arithmetic')
        self.no_none(bb, 'arithmetic')
        if bb.q == 0:
            raise refuse('RUN-DIV', 'division by zero')
        # value x idx[d] / idx[Y], exactly, as num CUR (L §3.4); an enclosure rides with it (L §7.3)
        num = arith('*', Num(v.q, v.u), a, None, None, None)
        r = arith('/', num, bb, None, None, None)
        return Num(r.q, v.u, enc=r.enc)

    def read_reading(self, info, keys, ctx, probe=False):
        """A plain read of a reading takes the reading at that instant (L §4.6 Readings)."""
        d = ctx.law
        self.need_law(d, info)
        try:
            cands = self.candidates(info.rows, keys, d, ctx, info, instant=d)
        except HoleSignal as h:
            return h.hole
        r = self.choose(cands, info, 'reading')
        org = self.origin(info, keys)
        if r is None:
            if probe:
                return ('missing',)
            return Hole([('%s has no reading at %s' % (org, d), org)])
        v = self.row_value(r, info, keys, ctx, None, None)
        if probe and isinstance(v, Hole):
            return ('missing',)
        return v

    def e_lastx(self, n, env, ctx):
        """last(x[k], d): the written carry (L §4.6)."""
        r = n.ref
        info = self.m.inputs[r._r[2]]
        keys, hole = self.key_values(info, r.keys if r.k == 'index' else [], r._convs, env, ctx)
        if hole is not None:
            return hole
        dv = self.ev(n.d, env, ctx)
        if isinstance(dv, Hole):
            return dv
        d = self.date_of(dv, 'the day of last')
        pv = self.pin_value(info, keys, ctx)
        if pv is not NOPIN and not isinstance(pv, tuple):
            return pv
        org = self.origin(info, keys)
        try:
            cands = self.candidates(info.rows, keys, d, ctx, info, last_before=d)
        except HoleSignal as h:
            return h.hole
        if not cands:
            if info.reading:
                # L §4.6 Readings: "With no reading on or before d, last and since are holes (HOLE-NOREADING)"; L §8.1:
                # "<x>[k] has no reading on or before <d>"
                return Hole([('%s has no reading on or before %s' % (org, d), org)])
            return self.no_candidate(info, keys, ctx, d, None)
        if info.reading:
            latest = max(c.instant[0] for c in cands)
            cands = [c for c in cands if c.instant[0] == latest]
        else:
            latest = max(c.span(ctx.cut).start for c in cands)
            cands = [c for c in cands if c.span(ctx.cut).start == latest]
        row = self.choose(cands, info, 'last')
        v = self.row_value(row, info, keys, ctx, None, None)
        self.reg(('held', '%s = %s (%s), read for %s' % (org, fmt(v), row.version.id, d)))
        return v

    def e_call_since(self, n, env, ctx):
        r = n.args[0]
        info = self.m.inputs[r._r[2]]
        keys, hole = self.key_values(info, r.keys if r.k == 'index' else [], r._convs, env, ctx)
        if hole is not None:
            return hole
        dv = self.ev(n.args[1], env, ctx)
        if isinstance(dv, Hole):
            return dv
        d = self.date_of(dv, 'the day of since')
        try:
            cands = self.candidates(info.rows, keys, d, ctx, info, last_before=d)
        except HoleSignal as h:
            return h.hole
        if not cands:
            # L §4.6 Readings / L §8.1 HOLE-NOREADING: "<x>[k] has no reading on or before <d>"
            org = self.origin(info, keys)
            return Hole([('%s has no reading on or before %s' % (org, d), org)])
        latest = max(c.instant[0] for c in cands)
        return Int((d - latest).days)

    # -- defs (L §5.11) ----------------------------------------------------------------
    def def_axes(self, qid):
        """The axes a def's cone reaches, or None when the cone reaches movements or balances (whose timeline
        carries every axis): E §3.2, the context is trimmed to what the item can depend on."""
        cache = self.m.__dict__.setdefault('_def_axes', {})
        if qid in cache:
            return cache[qid]
        ch = getattr(self.m, 'checker', None)
        out = None
        if ch is not None:
            seen = set()
            stack = [('def', qid)]
            axes = set()
            plain = True
            while stack and plain:
                x = stack.pop()
                if x in seen:
                    continue
                seen.add(x)
                if x[0] not in ('def', 'input', 'band', 'axis'):
                    plain = False
                    break
                if x[0] == 'input':
                    inf = self.m.inputs.get(x[1])
                    if inf is None or inf.observed or inf.reading or getattr(inf, 'builtin', False):
                        plain = False
                        break
                axes |= ch.axes_of.get(x, set())
                if x[0] == 'axis':
                    axes.add(x[1])
                for t in ch.edges.get(x, {}):
                    stack.append(t)
            if plain:
                out = frozenset(axes)
        cache[qid] = out
        return out

    def read_def(self, info, keys, ctx):
        w = ctx.walk
        if ctx.wp is not None and w is not None and not w.proj and info.id in getattr(self.m, 'walk_bound', ()) and \
                not getattr(self, '_explaining_wb', False):
            # a walk-bound def's cell at each walk point of the main walk, kept for explain (E §10)
            rec = self.walk_cells.setdefault((id(w), info.id, tuple(_kv(k) for k in keys)), {})
            try:
                v = self._read_def(info, keys, ctx)
            except TallyError as e:
                rec[ctx.wp] = e
                raise
            rec[ctx.wp] = v
            return v
        return self._read_def(info, keys, ctx)

    def _read_def(self, info, keys, ctx):
        self.index_assumed(info)
        if ctx.path is not None and info.id not in self.m.random_items:
            # a cell no random fact reaches is shared by every path (E §7 step 1)
            ctx = ctx.but(path=None)
        if ctx.axes:
            # an item that reaches no axis has no axis options in its context (E §3.2)
            ra = self.def_axes(info.id)
            if ra is not None and any(a not in ra for a, o in ctx.axes):
                ctx = ctx.but(axes=tuple((a, o) for a, o in ctx.axes if a in ra))
        if ctx.wp is not None and info.id not in getattr(self.m, 'walk_bound', ()):
            # only a walk-bound def carries the walk point (E §3.2)
            ctx = ctx.but(walk=None, wp=None)
        pv = self.pin_value(info, keys, ctx)
        if pv is not NOPIN and not (isinstance(pv, tuple) and pv[0] == 'fields'):
            return pv
        for b, k in zip(info.binds, keys):
            if b.node.where is not None or b.dom[0] == 'list':
                pass
        env = {b.var: self.keyval(k) for b, k in zip(info.binds, keys)}
        for b in info.binds:
            if b.node.where is not None:
                w = self.ev(b.node.where, env, ctx.but(law=None))
                if w is not True:
                    # the def is defined where its key bind's where holds (L §5.6) (L §8.4 What isn't defined is refused)
                    raise refuse('RUN-DOMAIN', '%s read at a key its key bind\'s where leaves out' % info.id)
        at = info.at
        if at is None:
            lctx = ctx.but(law=None)
            return self.def_cell(info, keys, env, lctx)
        if at.k == 'law_reader':
            if ctx.law is None:
                raise refuse('LOAD-LAWDATE', 'ask reaches an at reader def (%s) without law' % info.id)
            return self.def_cell(info, keys, env, ctx)
        if at.k == 'law_expr':
            lv = self.law_value(at.e, env, ctx, info)
            if isinstance(lv, Hole):
                return lv
            if isinstance(lv, Iv):
                return self.def_window(info, keys, env, ctx, lv)
            return self.def_cell(info, keys, env, ctx.but(law=lv))
        raise NotYet('a law date of the form %s' % at.k)

    def law_value(self, e, env, ctx, info):
        # the reads inside `at e` are made at the law date of the read that asks for the def: its reader's,
        # or the ask's `law` (L §4.3)
        v = self.ev(e, env, ctx)
        if isinstance(v, Hole):
            return v
        if isinstance(v, datetime.date):
            return v
        if isinstance(v, Iv):
            return self.window_choice(v, ctx)
        if v is NONE:
            raise refuse('RUN-NONE', 'the law date of %s is none here' % info.id)
        if v is NEVER:
            # never is later than every date (L §3.1): no law governs it (L §8.4 What isn't defined is refused)
            raise refuse('RUN-DOMAIN', 'the law date of %s is never' % info.id)
        if isinstance(v, Period):
            raise LoadError('LOAD-LAWPERIOD', 'at %s: a period is not a law date' % fmt(v))
        raise refuse('LOAD-UNIT', 'the law date of %s is %s' % (info.id, fmt(v)))

    def def_window(self, info, keys, env, ctx, w):
        """An at that gives a window reads the def once per law-date class the window overlaps
        (L §4.3); every day of the window gives the same values as its classes."""
        days = sorted(w.days) if w.days else [w.lo + datetime.timedelta(days=i) for i in range((w.hi - w.lo).days + 1)]
        vals = []
        for dd in days:
            vals.append(self.def_cell(info, keys, env, ctx.but(law=dd)))
        imprecise = Hole([('date within %s not precise enough for the law date of %s' % (w.window_text(), info.id),
                           self.origin(info, keys))])

        def same(a, b):
            if isinstance(a, Hole) or isinstance(b, Hole):
                return isinstance(a, Hole) and isinstance(b, Hole) and set(a.qs) == set(b.qs)
            return vkey(a) == vkey(b)
        if all(same(v, vals[0]) for v in vals):
            return vals[0]
        # L §4.3: "If all give one value, that is the value; otherwise it is HOLE-IMPRECISE" (round 18, C3-7: the
        # classes are bounded by lifespans too, E §3.2). refcore reads every day of the window, so every boundary is
        # one; a day whose value is no hole and differs from another day's lies in another class, so the classes
        # differ (snap11 let any hole win over HOLE-IMPRECISE)
        if not all(isinstance(v, Hole) for v in vals):
            return imprecise
        # every day a hole, not all alike: holes of rows or questions that differ lie in different classes; but the
        # gap rule's hole names its law date (L §4.6 step 5, `no value at <law date>`), which E §3.2 says is the same
        # value on every day of a class, and the class's extent ("the rows and rules the item reaches") then decides
        # whether the window holds one class or two
        dated = re.compile(r': no value at \d{4}-\d{2}-\d{2}$')
        if any(dated.search(q[0]) for v in vals for q in v.qs):
            # E §3.2 (round 19, GD-22): "A stretch of law dates over which the rows and rules the item reaches don't
            # change is one class even where no row covers it, so a window at inside such a stretch is read once, at
            # its first day inside the window, and gives one hole, which names that day". refcore has no class
            # computation: it takes the window as one stretch where no span of the reality (a version's governs, a
            # row's or rule's own span, a lifespan or membership) starts or ends inside it, a superset of the item's
            # boundaries, and the days' holes agree but for the dates they name
            def norm(v):
                return frozenset((dated.sub(': no value at D', q[0]),) + tuple(q[1:]) for q in v.qs)
            if all(norm(v) == norm(vals[0]) for v in vals):
                if not self.window_bounded(days):
                    return vals[0]
                raise NotYet('the law-date classes of a window at (%s) whose days give the gap rule\'s holes, with a '
                             'span boundary of the reality inside it (E §3.2)' % w.window_text())
        return imprecise

    def window_bounded(self, days):
        """Whether a span boundary of the reality falls inside the days (after the first): a version's governs, a row's
        or a rule's own span, a thing's entry (lifespan or membership)."""
        m = self.m
        lo, hi = days[0], days[-1]
        bs = set()

        def add(sp):
            if sp is None or getattr(sp, 'always', False):
                return
            for d in (sp.start, sp.end):
                if d is not None:
                    bs.add(d)
        for v in m.versions.values():
            add(getattr(v, 'governs', None))
            for mv, sp in getattr(v, 'moved_by', ()):
                add(sp)
        for inf in m.inputs.values():
            for r in getattr(inf, 'rows', ()):
                add(getattr(r, 'own_span', None))
        for d in m.defs.values():
            for rvs in getattr(d, 'rules', {}).values():
                for rv in rvs:
                    add(getattr(rv, 'span', None))
        for th in m.things.values():
            for e in th.entries:
                add(getattr(e, 'span', None))
        return any(lo < b <= hi for b in bs if isinstance(b, datetime.date))

    def def_cell(self, info, keys, env, ctx):
        ck = ('def', info.id, tuple(_kv(k) for k in keys), ctx.key())
        hit = self.memo.get(ck)
        if hit is not None and not self.tracing:
            v, err, coll = hit
            self.cur.merge(coll)
            if err is not None:
                raise err
            return v
        if ck in self.active:
            if ck in getattr(self, '_walk_outer', ()):
                # a walk started inside this cell's evaluation needs it (mreads.m_guarded_walk): from here on, what
                # the attempt caches is taken back if it is abandoned
                self.outer_hit()
            raise refuse('RUN-CYCLE', '%s reads itself through a computed key' % self.origin(info, keys))
        self.active.add(ck)
        tn = None
        if self.tracing:
            if ck in self.trace_cells:
                prev = self.trace_cells[ck]
                if self.trace_stack:
                    self.trace_stack[-1].children.append(('ref', prev))
                self.active.discard(ck)
                hit = self.memo.get(ck)
                if hit is not None:
                    v, err, coll = hit
                    self.cur.merge(coll)
                    if err is not None:
                        raise err
                    return v
            tn = TraceNode('def', self.origin(info, keys))
            tn.law = ctx.law
            if self.trace_stack:
                self.trace_stack[-1].children.append(('node', tn))
            self.trace_cells[ck] = tn
            self.trace_stack.append(tn)
        c = Coll()
        self.stack.append(c)
        err = None
        v = None
        try:
            v = self.eval_rules(info, keys, env, ctx, tn)
            v = self.coerce_def(info, v)
        except TallyError as e:
            err = e
        finally:
            self.stack.pop()
            self.active.discard(ck)
            if tn is not None:
                self.trace_stack.pop()
                tn.value = v if err is None else err
        self.memo[ck] = (v, err, c)
        self.cur.merge(c)
        if err is not None:
            raise err
        return v

    def coerce_def(self, info, v):
        t = info.type
        if isinstance(v, (Hole, Special)):
            return v
        if t.k == 'int' and isinstance(v, Num):
            if v.q.denominator != 1:
                raise refuse('RUN-NOT-INT', '%s is not an integer' % fmt(v))
            return Int(v.q)
        if t.k == 'num' and isinstance(v, Num) and (v.money or v.isint):
            return Num(v.q, v.u)
        if t.k == 'money' and isinstance(v, Num) and not v.money:
            if v.isint and (v.q == 0):
                return Money(0, t.cur)
            raise refuse('LOAD-UNIT', 'a num stored as money in %s' % info.id)
        if t.k == 'key' and isinstance(v, (Period, datetime.date)):
            ix = self.m.indexes.get(t.ix)
            if ix is not None and not ix.has(v):
                raise refuse('RUN-KEY', '%s is outside %s' % (fmt(v), ix.id))
            return v
        if t.k == 'key' and isinstance(v, Num) and v.isint:
            ix = self.m.indexes.get(t.ix)
            if ix is not None and ix.isint:
                k = int(v.q)
                if not ix.has(k):
                    raise refuse('RUN-KEY', '%d is outside %s' % (k, ix.id))
                return v
        return v

    def rule_exists(self, rv, p, ctx, info):
        """Whether a version of a rule exists at day p under the cut (L §5.11 step 2)."""
        K = ctx.cut
        if rv.version is not None and not in_world(rv.version):
            return False
        if rv.v is not None and not in_world(rv.v):
            # pointing at a scenario version: outside the scenario the rule is never a candidate (L §6.4)
            return False
        if rv.version is not None:
            vis = self.known_vis(rv.version.known, rv.version.id, 'a version', ctx)
            if not vis:
                return False
            if not self.wording_ok(rv.version, info.id, ctx, rv.span):
                return False
            sp = rv.span if rv.span is not None else rv.version.span_at(K)
            if sp is None:
                return False
            if p is None:
                if sp.always:
                    # a rule whose version governs always isn't law-dated (L §4.3 rule 1)
                    return True
                raise NotYet(LAWGUARD % ('a dated rule of %s' % info.id))
            return sp.contains(p)
        ok = True
        if rv.v is not None:
            vis = self.known_vis(rv.v.known, rv.v.id, 'a version', ctx)
            if not vis:
                return False
            if not self.wording_ok(rv.v, info.id, ctx, rv.span):
                return False
            sp = rv.v.span_at(K)
            if sp is None:
                return False
            if p is None and sp.always:
                # its version governs always: the rule's value doesn't change with the law date (L §4.3 rules 1-2)
                ok = True
            elif p is None:
                raise NotYet(LAWGUARD % ('a rule with v of %s' % info.id))
            else:
                ok = sp.contains(p)
        if ok and rv.span is not None:
            if p is None:
                raise NotYet(LAWGUARD % 'a rule with its own governs')
            ok = rv.span.contains(p)
        return ok

    def rule_in_force(self, info, label, p, ctx):
        vers = [rv for rv in info.rules[label] if self.rule_exists(rv, p, ctx, info)]
        if not vers:
            return None
        reps = [rv for rv in vers if rv.replace]
        if reps:
            best = max(reps, key=lambda rv: known_first(rv.known()))
            same = [rv for rv in reps if known_first(rv.known()) == known_first(best.known())]
            if len(same) > 1:
                # two versions of the rule known the same day whose spans both hold p: neither replaces the other, a
                # tie (L §4.6 steps 2 and 4; round 18 C1-11)
                vs = ' and '.join(sorted(rv.version.id if rv.version is not None else '?' for rv in same))
                raise refuse('LOAD-OVERLAP', 'rules %s.%s of %s overlap, neither replacing the other' % (
                    info.id, label, vs))
            return best
        if len(vers) > 1:
            raise refuse('LOAD-LABEL', 'rule %s already exists in %s' % (label, info.id))
        return vers[0]

    def beats_order(self, info, labels):
        done = []
        left = set(labels)
        while left:
            ready = sorted(l for l in left if not (info.beaters.get(l, set()) & left))
            if not ready:
                raise LoadError('LOAD-CYCLE', 'a beats cycle in %s' % info.id)
            for l in ready:
                done.append(l)
                left.discard(l)
        return done

    def eval_rules(self, info, keys, env, ctx, tn):
        m = self.m
        org = self.origin(info, keys)
        d, period = self.time_point(info, keys, ctx)
        # 1. givens
        if info.givens:
            try:
                cands = self.candidates(info.givens, keys, d, ctx, info)
            except HoleSignal as h:
                return h.hole
            g = self.choose(cands, info, 'given')
            if g is not None:
                v = self.row_value(g, info, keys, ctx, d, period)
                self.reg(('given', '%s%s = %s (%s)' % (info.id, g.keys_text(), fmt(g.val) if not isinstance(
                    g.val, tuple) else '?', g.version.id)))
                if tn is not None:
                    tn.lines.append('given %s%s (version %s)' % (info.id, g.keys_text(), g.version.id))
                return v
        # 2. the versions of each rule in force at the law date d (L §5.11 step 2: "Among the versions of one
        # rule that exist at d and K"; L §4.3: the written `at` sets it, a time key doesn't)
        d = ctx.law
        live = {}
        for lab in info.labels:
            rv = self.rule_in_force(info, lab, d, ctx)
            if rv is not None:
                live[lab] = rv
        order = self.beats_order(info, list(live))
        applicable = []
        unknown = {}
        state = {}
        missing_true = {}
        for lab in order:
            rv = live[lab]
            beaters = info.beaters.get(lab, set())
            by = sorted(b for b in beaters if b in applicable and lab in live[b].node.beats)
            if by:
                state[lab] = 'beaten by %s' % ', '.join(by)
                continue
            w = rv.node.when
            if w is None:
                applicable.append(lab)
                continue
            if tn is not None:
                sub = TraceNode('when', lab)
                self.trace_stack.append(sub)
            c = Coll()
            self.stack.append(c)
            try:
                wv = self.ev(w, env, ctx)
            finally:
                self.stack.pop()
                if tn is not None:
                    self.trace_stack.pop()
                    tn.when_children.append((lab, sub))
            self.cur.merge(c)
            if c.missing:
                missing_true[lab] = list(c.missing)
            if wv is True:
                applicable.append(lab)
            elif wv is False:
                state[lab] = 'not applicable'
            elif isinstance(wv, Hole):
                unknown[lab] = wv
                state[lab] = 'unknown'
            elif wv is NONE:
                raise refuse('RUN-NONE', 'the when of %s.%s is none here' % (info.id, lab))
            else:
                raise refuse('LOAD-UNIT', 'the when of %s.%s is not bool' % (info.id, lab))
        # L §5.11 step 4: "The candidates are the applicable rules that no applicable rule beats"
        cands = [l for l in applicable if not any(l in live[o].node.beats for o in applicable if o != l)]
        for l in applicable:
            if l not in cands:
                state[l] = 'beaten by %s' % ', '.join(sorted(o for o in applicable if l in live[o].node.beats))
        if len(cands) >= 2:
            raise refuse('RUN-CONFLICT', 'rules %s and %s both win for %s on %s' % (
                cands[0], cands[1], org, d if d is not None else 'no law date'))
        relevant = []
        for u, h in unknown.items():
            beats_cand = any(c in live[u].node.beats for c in cands)
            beaten_by_cand = any(u in live[c].node.beats for c in cands)
            # L §5.11 step 5: "when it would beat a candidate, be a candidate beside the winner, or be the only rule
            # left"
            if beats_cand or not beaten_by_cand or not cands:
                relevant.append(h)
        if tn is not None:
            tn.rule_states = dict(state)
            tn.rule_vers = live
        if relevant:
            if tn is not None and cands:
                tn.rule_states[cands[0]] = 'candidate'
            return merge(*relevant)
        if not cands:
            raise refuse('RUN-NO-RULE', 'no rule applies to %s on %s' % (org, d if d is not None else 'no law date'))
        win = cands[0]
        rv = live[win]
        if tn is not None:
            tn.rule_states[win] = 'won'
        val = rv.node.value
        for vv in (rv.v, rv.version):
            if vv is not None and vv.assumed is not None:
                self.reg(('assumed', vv.assumed, '%s.%s' % (info.id, win)))
        if win in missing_true and rv.v is not None:
            self.reg(('missing', rv.v.id))
        if val.k == 'hole':
            return Hole([(val.text, org)])
        if tn is not None:
            sub = TraceNode('value', win)
            self.trace_stack.append(sub)
            try:
                v = self.ev(val, env, ctx)
            finally:
                self.trace_stack.pop()
                tn.value_children = sub
            return v
        return self.ev(val, env, ctx)

    # -- bands (L §5.9) -------------------------------------------------------------------
    def band_table(self, info, key, ctx):
        """Choose the table by the read rule, a version's table as one row (L §5.9)."""
        d = ctx.law
        cands = []
        for t in info.tables:
            if key is not None and _kv(t.key) != _kv(key):
                continue
            if key is None and t.key is not None:
                continue
            if not in_world(t.version):
                continue
            vis = self.known_vis(t.version.known, t.version.id, 'a version', ctx)
            if not vis or not self.wording_ok(t.version, info.id, ctx):
                continue
            sp = t.version.span_at(ctx.cut)
            if sp is None:
                continue
            if d is None:
                # read with no law date: a band whose every table governs always isn't law-dated (L §4.3, MC-X-F12),
                # so each of its tables governs whatever day; a dated table here is refused at load (LOAD-LAWDATE)
                if not sp.always:
                    raise NotYet(LAWGUARD % ('a read of band %s' % info.id))
            elif not sp.contains(d):
                continue
            cands.append(t)
        return self._table_of(cands, info, key, d)

    def _table_of(self, cands, info, key, d):
        if not cands:
            return None
        best = max(cands, key=lambda t: known_first(t.version.known))
        top = [t for t in cands if known_first(t.version.known) == known_first(best.version.known)]
        if len(top) > 1:
            raise refuse('LOAD-OVERLAP', 'band %s: two tables known the same day' % info.id)
        t = top[0]
        if not t.replace:
            others = [o for o in cands if o is not t]
            if others:
                raise refuse('LOAD-OVERLAP', 'band %s: tables overlap without replace' % info.id)
            return [(r, t) for r in t.rows]
        base = self._table_of([o for o in cands if known_first(o.version.known) < known_first(t.version.known)],
                              info, key, d)
        if base is None:
            # L §5.9 Rows replaced: "On a day of its span where no table is in force, only a whole table stands, as an
            # input's replace row is its value where no earlier row governs; a patch there is refused (LOAD-REPLACE)"
            # (a whole table that replaces no table on any day of its span is refused at load, round 18 C3-16)
            if getattr(t, 'whole_ok', False):
                return [(r, t) for r in t.rows]
            raise refuse('LOAD-REPLACE', 'replace rows of %s in %s patch no table on %s' % (info.id, t.version.id, d))
        # L §5.9 Rows replaced: the earlier table with the matching rows' values replaced, when every replace row's
        # interval is one of its; otherwise the replace rows form a whole table that replaces it wholly
        if all(any(x.same_interval(r) for r, src in base) for x in t.rows):
            out = []
            for r, src in base:
                rep = [x for x in t.rows if x.same_interval(r)]
                out.append((rep[0], t) if rep else (r, src))
            return out
        if getattr(t, 'whole_ok', False):
            return [(r, t) for r in t.rows]
        raise refuse('LOAD-BAND', 'band %s in %s: the replace rows neither match the earlier table\'s intervals nor '
                     'form a whole table' % (info.id, t.version.id))

    def band_key(self, n, info, env, ctx):
        if not info.binds:
            return None, None
        conv = n._bconv
        kn = n.bkey
        b = info.binds[0]
        if conv[0] == 'lit':
            k = conv[1]
        elif conv[0] == 'axis':
            k = Key(conv[1], self.axis_option(conv[1], ctx))
        else:
            v = self.ev(kn, env, ctx)
            if isinstance(v, Hole):
                return None, v
            k = self.to_key(v, conv, b.dom, kn)
        self.check_key(k, b.dom, ctx)
        return k, None

    def band_origin(self, info, key):
        return info.id if key is None else '%s[%s]' % (info.id, key_text(key))

    def band_rows(self, info, key, ctx):
        rows = self.band_table(info, key, ctx)
        if rows is None:
            org = self.band_origin(info, key)
            # L §4.6 step 5 (an input or a band): "Read with no law date ..., the question ends at no value"
            at = '' if ctx.law is None else ' at %s' % ctx.law
            if key is None:
                return None, Hole([('%s: no value%s' % (info.id, at), org)])
            return None, Hole([('%s at %s: no value%s' % (info.id, key_text(key), at), org)])
        return rows, None

    def call_band(self, n, env, ctx):
        info = self.m.bands[n._r[1]]
        key, hk = self.band_key(n, info, env, ctx)
        if hk is not None:
            return hk
        x = self.ev(n.args[0], env, ctx)
        if isinstance(x, Hole):
            return Hole(x.qs)
        self.no_none(x, 'a band argument')
        rows, hole = self.band_rows(info, key, ctx)
        if hole is not None:
            return hole
        xk = band_x(x, self.m)
        for r, t in rows:
            if r.holds(xk):
                return self.band_value(info, key, r, t, x, ctx)
        return self.band_no_row(info, key, x, ctx)

    def band_no_row(self, info, key, x, ctx):
        """L §8.1 HOLE-UNGOVERNED: "no row of the band's table holds its argument (§5.9)", question "<b>: no row holds
        <x> at <law date>"."""
        org = self.band_origin(info, key)
        if ctx.law is None:
            # a band whose every table governs always is read with no law date (L §4.3, MC-X-F12); L §5.9 and §8.1
            # (round 19, GD-24): "`<b>: no row holds <x>` read with no law date"
            return Hole([('%s: no row holds %s' % (org, fmt(x)), org)])
        return Hole([('%s: no row holds %s at %s' % (org, fmt(x), ctx.law), org)])

    def band_value(self, info, key, r, t, x, ctx):
        org = self.band_origin(info, key)
        if self.tracing and self.trace_stack:
            tn = TraceNode('bandrow', '%s: row %s (version %s%s)' % (
                org, interval_text(r), t.version.id, (', "%s"' % r.locator) if r.locator else ''))
            self.trace_stack[-1].children.append(('node', tn))
        self.cur.rows['%s:%s %s' % (t.version.id, org, interval_text(r))] = BandRowRef(t, r, org)
        if t.version.assumed is not None:
            self.reg(('assumed', t.version.assumed, org))
        sp = t.version.span_at(ctx.cut)
        if sp is not None and not sp.always and sp.start is not None and sp.end is None and ctx.law > self.ask_K:
            self.reg(('held', '%s row %s (%s), read for %s' % (org, interval_text(r), t.version.id, ctx.law)))
        if r.value.k == 'hole':
            return Hole([(r.value.text, org)])
        env = {info.decl.arg: x}
        v = self.ev(r.value, env, ctx)
        t_ = info.type
        if isinstance(v, Num) and t_.k == 'money' and not v.money:
            if v.isint and v.q == 0:
                return Money(0, t_.cur)
            raise refuse('LOAD-UNIT', 'a num stored as money in band %s' % info.id)
        if isinstance(v, Num) and t_.k == 'num' and v.money:
            return Num(v.q, v.u)
        return v

    def band_ref_key(self, n, env, ctx):
        bid, kn, conv = n._band
        info = self.m.bands[bid]
        if kn is None:
            return info, None, None
        b = info.binds[0]
        if conv[0] == 'lit':
            k = conv[1]
        elif conv[0] == 'axis':
            k = Key(conv[1], self.axis_option(conv[1], ctx))
        else:
            v = self.ev(kn, env, ctx)
            if isinstance(v, Hole):
                return info, None, v
            k = self.to_key(v, conv, b.dom, kn)
        self.check_key(k, b.dom, ctx)
        return info, k, None

    def c_band_lo(self, n, env, ctx):
        info, key, hk = self.band_ref_key(n, env, ctx)
        if hk is not None:
            return hk
        x = self.ev(n.args[1], env, ctx)
        if isinstance(x, Hole):
            return Hole(x.qs)
        self.no_none(x, 'a band argument')
        rows, hole = self.band_rows(info, key, ctx)
        if hole is not None:
            return hole
        xk = band_x(x, self.m)
        for r, t in rows:
            if r.holds(xk):
                end = r.lo if n.name == 'band_lo' else r.hi
                if end is None:
                    return NONE
                return self.band_end_value(info, end)
        return self.band_no_row(info, key, x, ctx)

    c_band_hi = c_band_lo

    def band_end_value(self, info, end):
        at = info.argtype
        if isinstance(end, datetime.date):
            return end
        if hasattr(end, 'orig'):
            return end.orig
        if at.k == 'money':
            return Money(end, at.cur)
        if at.k == 'int':
            return Int(end)
        return Num(end, at.u)

    def c_marginal(self, n, env, ctx):
        """marginal(b, x): the sum over rows whose lower end is below x of value × (min(x, hi) − lo)."""
        info, key, hk = self.band_ref_key(n, env, ctx)
        if hk is not None:
            return hk
        x = self.ev(n.args[1], env, ctx)
        if isinstance(x, Hole):
            return Hole(x.qs)
        self.no_none(x, 'a band argument')
        rows, hole = self.band_rows(info, key, ctx)
        if hole is not None:
            return hole
        if rows and rows[0][0].lo is None:
            raise refuse('LOAD-BAND', 'marginal(%s, x) over a first row from -inf' % info.id)
        xk = band_x(x, self.m)
        if hasattr(xk, 'orig'):
            # keys and periods are ordered, not measured: a row's width is no quantity (L §8.4)
            raise refuse('LOAD-UNIT', 'marginal(%s, x) over keys: a row\'s width is no quantity' % info.id)
        if not any(r.holds(xk) for r, t in rows):
            return self.band_no_row(info, key, x, ctx)
        acc = None
        holes = []
        xu = x.u if isinstance(x, Num) else ()
        for r, t in rows:
            if r.lo is None or not (r.lo < xk):
                continue
            hi = xk if r.hi is None or xk < r.hi else r.hi
            width = hi - r.lo
            v = self.band_value(info, key, r, t, x, ctx)
            if isinstance(v, Hole):
                holes.append(v)
                continue
            self.no_none(v, 'marginal')
            if not isinstance(v, Num):
                raise refuse('LOAD-UNIT', 'marginal over rows that are not numbers')
            u, f = U.mul(xu, v.u)
            term = v.q * width * f
            acc = term if acc is None else acc + term
            ures = u
        if holes:
            return merge(*holes)
        if acc is None:
            u, f = U.mul(xu, info.type.u if info.type.k in ('num', 'money') else ())
            return Num(0, u)
        return Num(acc, ures)

    # -- operators ------------------------------------------------------------------------
    def no_none(self, v, what):
        if v is NONE:
            raise refuse('RUN-NONE', 'a value is none here: %s' % what)

    def e_bin(self, n, env, ctx):
        a = self.ev(n.l, env, ctx)
        b = self.ev(n.r, env, ctx)
        return self.binop(n, a, b, ctx)

    def binop(self, n, a, b, ctx):
        op = n.op
        if isinstance(a, Hole) or isinstance(b, Hole):
            # a refusal outranks a hole (L §8.4): none or never in arithmetic is refused whatever the hole holds
            for v in (a, b):
                if v is NEVER:
                    raise refuse('RUN-NONE', 'never in arithmetic')
                if not isinstance(v, Hole):
                    self.no_none(v, 'arithmetic')
            if ctx.partial:
                return lin_bin(op, a, b, n)
            return merge(a, b)
        if a is NEVER or b is NEVER:
            raise refuse('RUN-NONE', 'never in arithmetic')
        self.no_none(a, 'arithmetic')
        self.no_none(b, 'arithmetic')
        if getattr(n, '_keyarith', False):
            return self.key_arith(n, a, b)
        if getattr(n, '_keydiff', False):
            return self.key_diff(n, a, b)
        if isinstance(a, Iv) or isinstance(b, Iv):
            return self.iv_arith(n, a, b, ctx)
        da = getattr(n, '_datearith', None)
        if da is not None:
            return self.date_arith(n, a, b)
        if isinstance(a, (datetime.date, Period)) or isinstance(b, (datetime.date, Period)):
            return self.date_arith(n, a, b)
        v = arith(op, a, b, n._t, n.l._t, n.r._t)
        if getattr(n, '_rate_duration', False) and v.u:
            raise refuse('LOAD-UNIT', '%s %s %s: a rate times a duration of another period dimension (%s)'
                         % (fmt(a), op, fmt(b), U.fmt(v.u)))
        return v

    def key_arith(self, n, a, b):
        ix_id = n._t.ix
        ix = self.m.indexes[ix_id]
        k = b if isinstance(b, int) else self.int_of(b)
        if n.op == '-':
            k = -k
        if isinstance(a, OutKey):
            raise refuse('RUN-KEY', '%s is outside %s' % (a.text, a.index))
        if ix.isint:
            return Int(self.int_of(a) + k)
        if ix.pk is not None:
            if isinstance(a, datetime.date):
                return add_days(a, k)
            return a.shift(k)
        p = ix.pos(a)
        q = p + k
        if 0 <= q < len(ix.keys):
            return ix.keys[q]
        return OutKey(ix_id, '%s %s %d' % (key_text(a), n.op, abs(k)), None)

    def key_diff(self, n, a, b):
        ix = self.m.indexes.get(n.l._t.ix)
        if ix is None:
            raise refuse('LOAD-UNIT', 'the difference of two things of a kind')
        if ix.isint:
            return Int(self.int_of(a) - self.int_of(b))
        if ix.pk is not None:
            if isinstance(a, datetime.date):
                return Int((a - b).days)
            s = b.steps_to(a)
            if s is None:
                # the difference of two keys of one index is defined (L §4.2); these periods are off its grid
                raise refuse('RUN-DOMAIN', 'the difference of periods %s and %s' % (fmt(a), fmt(b)))
            return Int(s)
        return Int(ix.pos(a) - ix.pos(b))

    def date_arith(self, n, a, b):
        op = n.op
        sign = 1 if op == '+' else -1
        if getattr(n, '_datearith', None) == 'pdiff' and isinstance(a, Period) and isinstance(b, Period):
            # L §4.2 Differences: the int number of periods from b to a (a key of an anchored index is the period
            # in which it starts, L §4.1)
            months = (a.start.year * 12 + a.start.month) - (b.start.year * 12 + b.start.month)
            if a.kind == 'year':
                return Int(a.start.year - b.start.year)
            if a.kind == 'quarter':
                return Int(months // 3)
            return Int(months)
        if isinstance(a, datetime.date):
            if isinstance(b, Num):
                if b.u:
                    if len(b.u) != 1 or b.u[0][1] != 1:
                        raise refuse('LOAD-UNIT', 'a date moves by an int or a duration')
                    per = b.u[0][0]
                    k = b.q * sign
                    if k.denominator != 1:
                        raise refuse('RUN-NOT-INT', '%s is not an integer' % fmt(b))
                    k = int(k)
                    if per == 'day':
                        return add_days(a, k)
                    if per == 'month':
                        return add_months(a, k)
                    if per == 'quarter':
                        return add_months(a, 3 * k)
                    return add_months(a, 12 * k)
                k = self.int_of(b) * sign
                return add_days(a, k)
        if isinstance(a, Period) and isinstance(b, Num) and b.u and len(b.u) == 1 and b.u[0][1] == 1 and \
                b.u[0][0] in ('day', 'month', 'quarter', 'year'):
            # a duration that is a whole number of the period's own periods (L §4.2 Date arithmetic: `y + 5 years`
            # is `y + 5`, `m + 1 year` is `m + 12`); any other is refused at the key (L §8.4)
            per = b.u[0][0]
            q = b.q
            if q.denominator != 1:
                raise refuse('RUN-NOT-INT', '%s is not an integer' % fmt(b))
            unit, amount = ('day', int(q)) if per == 'day' else ('month', int(q) * {'month': 1, 'quarter': 3,
                                                                                     'year': 12}[per])
            if unit != a.unit or amount % a.n:
                raise refuse('RUN-DOMAIN', '%s %s %s: not a whole number of its periods (L §4.2)' % (
                    fmt(a), op, fmt(b)))
            b = Int(amount // a.n)
        if isinstance(a, Period) and isinstance(b, Num) and not b.u:
            k = self.int_of(b) * sign
            if a.kind == 'day':
                return a.shift(k)
            if a.kind == 'month':
                s = add_months(a.start, k)
                return Period(a.kind, s, add_months(s, 1), 'month', a.n) if a.n == 1 else a.shift(k)
            if a.kind == 'quarter':
                return a.shift(k)
            # a year moves by years
            s = add_months(a.start, 12 * k)
            return Period('year', s, add_months(a.start, 12 * (k + 1)) if a.n == 12 else add_months(s, a.n), 'month',
                          a.n)
        raise refuse('LOAD-UNIT', '%s %s %s' % (fmt(a), op, fmt(b)))

    def iv_arith(self, n, a, b, ctx):
        """Arithmetic on a window gives an interval (L §4.4)."""
        if isinstance(a, Iv) and not isinstance(b, Iv):
            lo = self.binop_plain(n, a.lo, b, ctx)
            hi = self.binop_plain(n, a.hi, b, ctx)
            src = a.src
        elif isinstance(b, Iv) and not isinstance(a, Iv):
            lo = self.binop_plain(n, a, b.lo, ctx)
            hi = self.binop_plain(n, a, b.hi, ctx)
            src = b.src
        else:
            raise self.imprecise(a, 'arithmetic on two windows')
        if order_lt(hi, lo):
            lo, hi = hi, lo
        org = a.origin if isinstance(a, Iv) else b.origin
        return Iv(lo, hi, None, src, origin=org)

    def binop_plain(self, n, a, b, ctx):
        da = getattr(n, '_datearith', None)
        if getattr(n, '_keyarith', False):
            return self.key_arith(n, a, b)
        if da is not None or isinstance(a, (datetime.date, Period)) or isinstance(b, (datetime.date, Period)):
            return self.date_arith(n, a, b)
        return arith(n.op, a, b, n._t, n.l._t, n.r._t)

    def imprecise(self, v, use):
        return HoleSignal(Hole([('date within %s not precise enough for %s' % (v.window_text(), use),
                                 v.origin[1] if v.origin else '')]))

    def e_not(self, n, env, ctx):
        v = self.ev(n.e, env, ctx)
        if isinstance(v, Hole):
            return Hole(v.qs)
        self.no_none(v, 'not')
        return not v

    def e_and(self, n, env, ctx):
        nar = getattr(n, '_narrow', None)
        if nar:
            # L §5.7 (tally 2.1, R-17): "A conjunct that reads them is evaluated only where the test holds, and the
            # conjunction is false where it doesn't"
            for t in nar:
                if self.ev(t, env, ctx) is False:
                    return False
        a = self.ev(n.l, env, ctx)
        self.no_none(a, 'and')
        if a is False:
            return False
        b = self.ev(n.r, env, ctx)
        self.no_none(b, 'and')
        if b is False:
            return False
        if isinstance(a, Hole) or isinstance(b, Hole):
            return merge(a, b)
        return True

    def e_or(self, n, env, ctx):
        a = self.ev(n.l, env, ctx)
        self.no_none(a, 'or')
        if a is True:
            return True
        b = self.ev(n.r, env, ctx)
        self.no_none(b, 'or')
        if b is True:
            return True
        if isinstance(a, Hole) or isinstance(b, Hole):
            return merge(a, b)
        return False

    def e_if(self, n, env, ctx):
        c = self.ev(n.c, env, ctx)
        if c is True:
            return self.ev(n.a, env, ctx)
        if c is False:
            return self.ev(n.b, env, ctx)
        self.no_none(c, 'if')
        # a hole condition: both branches are evaluated, and the if is a hole unless they are equal
        # (L §5.11 Logic, §8.1).  Equal is equal as values: a bare 0 branch takes the type of the whole
        # (L §3.2 Inline figures), so 0 and a num 0 are equal.
        a = self.ev(n.a, env, ctx)
        b = self.ev(n.b, env, ctx)
        if not isinstance(a, Hole) and not isinstance(b, Hole) and same_value(a, b):
            fa = getattr(n.a, '_t', None) is not None and n.a._t.k == 'flex'
            return b if fa else a
        return merge(c, a, b)

    def e_cmp(self, n, env, ctx):
        a = self.ev(n.l, env, ctx)
        b = self.ev(n.r, env, ctx)
        return self.compare(n.op, a, b, n, ctx)

    def compare(self, op, a, b, n, ctx):
        eq = op in ('==', '!=')
        # none can only be tested (L §3.1)
        for x in (a, b):
            if x is NONE:
                if not eq:
                    raise refuse('RUN-NONE', 'a value is %s here: an ordering' % x.name)
                same = (a is b)
                if isinstance(a, Hole) or isinstance(b, Hole):
                    other = b if x is a else a
                    if isinstance(other, Hole):
                        return Hole(other.qs)
                return same if op == '==' else not same
        if isinstance(a, Hole) or isinstance(b, Hole):
            return merge(a, b)
        if a is NEVER or b is NEVER:
            if eq:
                return (a is b) == (op == '==')
            if a is NEVER and b is NEVER:
                return op in ('<=', '>=')
            x = 1 if a is NEVER else -1
            return {'<': x < 0, '<=': x < 0, '>': x > 0, '>=': x > 0}[op]
        if isinstance(a, Iv) or isinstance(b, Iv):
            return self.iv_compare(op, a, b, n)
        if isinstance(a, MonthDay) and isinstance(b, MonthDay):
            # L §3.1 monthday (GD-2): == and != only (an order is refused at load, L §8.4)
            if not eq:
                raise refuse('RUN-DOMAIN', 'an order on month-days')
            return (a == b) == (op == '==')
        if eq and ((isinstance(a, Thing) and isinstance(b, Thing)) or (isinstance(a, Key) and isinstance(b, Key)) or
                   (isinstance(a, Mv) and isinstance(b, Mv))):
            # things and keys compare by == and != whatever their order (L §5.7, §5.6); a movement is a thing of its
            # kind (L §1, §5.14: "A movement ... is a thing of its kind"), one by its stable id (L §2.5) (MC-X-F9)
            return (a.key() == b.key()) == (op == '==')
        c = self.cmp3(a, b, n)
        return {'==': c == 0, '!=': c != 0, '<': c < 0, '<=': c <= 0, '>': c > 0, '>=': c >= 0}[op]

    def cmp3(self, a, b, n):
        """-1, 0 or 1: numbers by value (one unit), dates, periods, keys in index order, bools (false < true), text
        and codes by equality; things and an open index's keys only by equality (L §5.7, §5.6)."""
        if isinstance(a, Num) and isinstance(b, Num):
            if a.rate and b.rate and a.u != b.u:
                raise refuse('LOAD-UNIT', 'rates of different periods compared')
            f = U.conv(b.u, a.u) if a.u != b.u else Fraction(1)
            if f is None:
                if not a.u or not b.u:
                    f = Fraction(1)
                else:
                    kc = key_clash(a, b)
                    if kc is not None:
                        raise kc
                    raise refuse('LOAD-UNIT', '%s and %s have different units' % (fmt(a), fmt(b)))
            x, y = a.q, b.q * f
            return (x > y) - (x < y)
        if isinstance(a, bool) and isinstance(b, bool):
            return (a > b) - (a < b)
        if isinstance(a, datetime.date) and isinstance(b, datetime.date):
            return (a > b) - (a < b)
        if isinstance(a, Period) and isinstance(b, Period):
            if a == b:
                return 0
            return (a.start > b.start) - (a.start < b.start) or (a.end > b.end) - (a.end < b.end)
        if isinstance(a, Key) and isinstance(b, Key):
            if a.key() == b.key():
                return 0
            nl = getattr(n, 'l', None)
            ixid = nl._t.ix if nl is not None and nl._t.k == 'key' else a.index
            if ixid in self.m.indexes and self.m.indexes[ixid].form == 'open':
                # L §5.6: an open index has no order a rule can read; L §8.4: refused at the key (RUN-DOMAIN)
                raise refuse('RUN-DOMAIN', 'an order on keys of the open index %s has no meaning' % ixid)
            if ixid in self.m.indexes:
                ix = self.m.indexes[ixid]
                pa, pb = ix.pos(a), ix.pos(b)
            elif ixid in self.m.axes:
                ax = self.m.axes[ixid]
                pa, pb = ax.options.index(a.name), ax.options.index(b.name)
            else:
                return 0 if a.name == b.name else 1
            return (pa > pb) - (pa < pb)
        if isinstance(a, Thing) and isinstance(b, Thing):
            if a.name == b.name:
                return 0
            # L §5.7 Things have no order a rule can read; L §8.4: refused at the key (RUN-DOMAIN)
            raise refuse('RUN-DOMAIN', 'an order on things (%s, %s) has no meaning' % (a.name, b.name))
        if isinstance(a, Mv) and isinstance(b, Mv):
            if a.id == b.id:
                return 0
            # a movement is a thing (L §5.14), and things have no order a rule can read (L §5.7; L §8.4: RUN-DOMAIN)
            raise refuse('RUN-DOMAIN', 'an order on movements (%s, %s) has no meaning' % (a.id, b.id))
        if isinstance(a, Text) and isinstance(b, Text):
            return 0 if a.s == b.s else 1
        if isinstance(a, Code) and isinstance(b, Code):
            return 0 if a.key() == b.key() else 1
        if isinstance(a, OutKey) or isinstance(b, OutKey):
            o = a if isinstance(a, OutKey) else b
            raise refuse('RUN-KEY', '%s is outside %s' % (o.text, o.index))
        if isinstance(a, Rec) or isinstance(b, Rec):
            # L §3.1 (round 12): "Two records are never compared" (L §8.4: RUN-DOMAIN at run)
            raise refuse('RUN-DOMAIN', 'a comparison of two records has no meaning (compare their fields)')
        raise refuse('LOAD-UNIT', '%s and %s compared' % (fmt(a), fmt(b)))

    def iv_compare(self, op, a, b, n):
        """A comparison on a window is decided when the whole interval lies on one side;
        a value compared with itself is decided (L §4.4)."""
        if isinstance(a, Iv) and isinstance(b, Iv) and a.origin is not None and a.origin == b.origin:
            return {'==': True, '!=': False, '<': False, '<=': True, '>': False, '>=': True}[op]
        alo, ahi = (a.lo, a.hi) if isinstance(a, Iv) else (a, a)
        blo, bhi = (b.lo, b.hi) if isinstance(b, Iv) else (b, b)
        # outcomes at the extremes
        res = set()
        if op in ('<', '<='):
            strict = op == '<'
            if (self.cmp3(ahi, blo, None) < 0) or (not strict and self.cmp3(ahi, blo, None) <= 0):
                return True
            if (self.cmp3(alo, bhi, None) >= 0) if strict else (self.cmp3(alo, bhi, None) > 0):
                return False
        elif op in ('>', '>='):
            strict = op == '>'
            if (self.cmp3(alo, bhi, None) > 0) or (not strict and self.cmp3(alo, bhi, None) >= 0):
                return True
            if (self.cmp3(ahi, blo, None) <= 0) if strict else (self.cmp3(ahi, blo, None) < 0):
                return False
        else:
            disjoint = self.cmp3(ahi, blo, None) < 0 or self.cmp3(bhi, alo, None) < 0
            if isinstance(a, Iv) and a.days is not None and not isinstance(b, Iv):
                disjoint = b not in a.days
            if isinstance(b, Iv) and b.days is not None and not isinstance(a, Iv):
                disjoint = a not in b.days
            if disjoint:
                return op == '!='
        w = a if isinstance(a, Iv) else b
        return Hole([('date within %s not precise enough for %s' % (w.window_text(), _ntxt(n) if n else 'a comparison'),
                      w.origin[1] if w.origin else '')])

    def e_in(self, n, env, ctx):
        x = self.ev(n.l, env, ctx)
        if isinstance(x, Hole):
            return Hole(x.qs)
        self.no_none(x, 'a membership test')
        r = n.r
        if getattr(r, '_r', None) and r._r[0] == 'kind':
            kind = r._r[1]
            if isinstance(x, Mv):
                return self.mv_kind_in(x, kind, ctx)
            if not isinstance(x, Thing):
                raise refuse('LOAD-UNIT', 'membership of %s in a kind' % fmt(x))
            t = self.m.things.get(x.name)
            if t is None:
                return False
            if n.on is not None:
                d = self.ev(n.on, env, ctx)
                if isinstance(d, Hole):
                    return Hole(d.qs)
                d = self.date_of(d, 'the day of a membership test')
                return self.m.alive_on(t, kind, d, ctx.cut)
            base = t.base
            chain = [base] + self.m.kinds[base].ancestors(self.m)
            if kind in chain:
                return True
            sub = self.m.kinds[kind].subtree(self.m)
            if any(e.kind in sub for e in t.entries):
                # "x in K tests membership of a kind whose memberships aren't dated"; a dated one needs `on d`
                # (L §5.7) (L §8.4 What isn't defined is refused)
                raise refuse('RUN-DOMAIN', '%s in %s with no on: a membership with a lifespan tested without a day'
                             % (x.name, kind))
            return False
        coll = self.ev(r, env, ctx)
        if isinstance(coll, Hole):
            return Hole(coll.qs)
        if isinstance(coll, Lst):
            return any(vkey(x) == vkey(y) for y in coll.items)
        raise refuse('LOAD-UNIT', 'membership in %s' % fmt(coll))

    def date_of(self, v, what):
        if isinstance(v, datetime.date):
            return v
        if isinstance(v, Iv):
            raise self.imprecise(v, what)
        if v is NONE:
            raise refuse('RUN-NONE', 'a value is none here: %s' % what)
        if v is NEVER:
            raise refuse('RUN-NONE', 'never in %s' % what)
        if isinstance(v, Period) and v.kind == 'day':
            return v.start
        raise refuse('LOAD-UNIT', '%s is %s, not a date' % (what, fmt(v)))

    def pin_lines(self, ctx, ctx2):
        """What a set of pins changed, as text: each pin with its value (E §10 an as if)."""
        out = []
        old = set(q.key() for q in ctx.pins)
        for q in ctx2.pins:
            if q.key() not in old:
                out.append(getattr(q, 'text', None) or repr(q.key()))
        if ctx2.law != ctx.law:
            out.append('law %s' % ctx2.law)
        if ctx2.cut != ctx.cut:
            out.append('known %s' % ctx2.cut)
        for it, d in ctx2.wording:
            if (it, d) not in ctx.wording:
                out.append('wording %s for %s' % (d, it))
        for ax, o in ctx2.axes:
            if (ax, o) not in ctx.axes:
                out.append('%s = %s' % (ax, o))
        if ctx2.world != ctx.world:
            out.append('scenario %s' % (ctx2.world or self.m.header.id))
        return out

    def eval_in(self, e, env, ctx, ctx2):
        """e evaluated in ctx2, the world switched when ctx2 names another (L §6.4)."""
        if ctx2.world != ctx.world:
            old = set_world(self.world_chain(ctx2.world))
            try:
                return self.ev(e, env, ctx2)
            finally:
                set_world(old)
        return self.ev(e, env, ctx2)

    def asif_traced(self, n, env, ctx):
        """explain of an as if (E §10): the pins with their values (a pin given by an expression with its derivation
        outside the as if), and the derivation inside them beside the one outside."""
        tn = TraceNode('asif', 'as if {...}')
        self.trace_stack[-1].children.append(('node', tn))
        pn = TraceNode('group', 'the pins, evaluated outside the as if')
        tn.children.append(('node', pn))
        self.trace_stack.append(pn)
        try:
            ctx2 = self.apply_pins(n.pins, env, ctx)
        except HoleSignal as h:
            return h.hole
        finally:
            self.trace_stack.pop()
        for line in self.pin_lines(ctx, ctx2):
            pn.children.append(('node', TraceNode('group', 'pin ' + line)))
        inside = TraceNode('group', 'inside the as if')
        tn.children.append(('node', inside))
        self.trace_stack.append(inside)
        try:
            v = self.eval_in(n.e, env, ctx, ctx2)
        finally:
            self.trace_stack.pop()
        inside.label = 'inside the as if: %s' % _tfmt(v)
        outside = TraceNode('group', 'outside the as if')
        tn.children.append(('node', outside))
        self.trace_stack.append(outside)
        self.stack.append(Coll())   # the outside derivation adds nothing to the result's register
        try:
            try:
                vo = self.ev(n.e, env, ctx)
            except HoleSignal as h:
                vo = h.hole
            except TallyError as e:
                if load_now(e):
                    raise
                vo = e
        finally:
            self.stack.pop()
            self.trace_stack.pop()
        outside.label = 'outside the as if: %s' % _tfmt(vo)
        return v

    def e_asif(self, n, env, ctx):
        """e as if {pins}: the pins' values are evaluated first, without that as if (L §5.11)."""
        if self.tracing and self.trace_stack:
            return self.asif_traced(n, env, ctx)
        try:
            ctx2 = self.apply_pins(n.pins, env, ctx, foreign_ok=True)
        except HoleSignal as h:
            return h.hole
        if ctx2.world != ctx.world:
            # x as if {scenario s} is x in s (L §6.4)
            chain = self.world_chain(ctx2.world, foreign_ok=True)
            if chain is None:
                # s sits on another stack of scenarios: the as if is evaluated in the reality that holds it (L §5.18
                # "an ask is checked and answered only in the worlds its line is evaluated in (... each as if
                # {scenario s} in it)"; cli.run_worlds)
                return self.cross(self, n, env, ctx)
            old = set_world(chain)
            try:
                return self.ev(n.e, env, ctx2)
            finally:
                set_world(old)
        return self.ev(n.e, env, ctx2)

    def world_chain(self, w, foreign_ok=False):
        """The scenarios seen in world w: w and the scenarios it stacks on (L §6.4); None for a world of another stack
        an ask's as if reaches (foreign_ok, with a cross evaluator)."""
        over = getattr(self.m, 'scenario_over', {})
        out = []
        while w is not None:
            if w not in over:
                if foreign_ok and getattr(self, 'cross', None) is not None and w in getattr(self.m, 'all_worlds', ()):
                    return None
                raise NotYet('the world %s from a reality that doesn\'t load it (an as if {scenario} reached from '
                             'another stack)' % w)
            out.append(w)
            w = over[w]
        return out

    def apply_pins(self, pins, env, ctx, ask=False, foreign_ok=False):
        """The context inside pins (L §5.11).  "A pin whose key is a hole, a law, known or wording pin whose date is a
        hole ... make the value inside the as if, or the ask line, a hole carrying their questions, and nothing inside
        is evaluated": HoleSignal with their holes."""
        holes = []
        old = getattr(self, '_foreign_ok', False)
        self._foreign_ok = foreign_ok
        try:
            out = self.apply_pins_(pins, env, ctx, ask, holes)
        finally:
            self._foreign_ok = old
        if holes:
            raise HoleSignal(merge(*holes))
        return out

    def apply_pins_(self, pins, env, ctx, ask, holes):
        pinsv = list(ctx.pins)
        law = ctx.law
        cut = ctx.cut
        axes = dict(ctx.axes)
        wording = list(ctx.wording)
        world = ctx.world
        for p in pins:
            if p.kind == 'scenario':
                world = None if p.id == self.m.header.id else p.id
                self.world_chain(world, foreign_ok=getattr(self, '_foreign_ok', False))
                self.reg(('world', p.id))
                continue
            if p.kind in ('law', 'known', 'wording'):
                # L §8.2 pinned: "A pin given by an expression prints its value, pinned: <pin> = v, and with it the
                # register of what the expression read" (a literal pin prints as written)
                c = Coll()
                self.stack.append(c)
                try:
                    v = self.ev(p.value, env, ctx)
                finally:
                    self.stack.pop()
                if isinstance(v, Hole):
                    holes.append(v)
                    continue
                d = self.date_of(v, 'the %s pin' % p.kind)
                expr = p.value.k not in ('date', 'month', 'quarter', 'year', 'num')
                if expr:
                    self.cur.merge(c)
                if p.kind == 'law':
                    law = d
                elif p.kind == 'known':
                    cut = d
                else:
                    for i in p.ids:
                        item = self.qualify_id(i)
                        wording.append((item, d))
                if p.kind == 'wording':
                    written = 'wording %s for [%s]' % (_ntxt(p.value) if expr else d, ', '.join(p.ids))
                else:
                    written = '%s %s' % (p.kind, _ntxt(p.value) if expr else d)
                self.reg(('pinned', ('%s = %s' % (written, d)) if expr else written))
            elif getattr(p, 'pkind', None) == 'axis':
                axes[p.axis] = p.option
                self.reg(('pinned', '%s = %s' % (p.axis, p.option)))
            elif getattr(p, 'pkind', None) == 'sample':
                try:
                    pinsv = self.sample_pin(p, env, ctx, pinsv)
                except HoleSignal as h:
                    holes.append(h.hole)
            else:
                info = p.info
                keys = []
                ktexts = []
                keyhole = False
                for kn, conv, b in zip(p.target.keys or [], p.keyconv, info.binds):
                    if conv is None:
                        keys.append(None)
                        ktexts.append('*')
                        continue
                    if conv[0] == 'lit':
                        k = conv[1]
                    elif conv[0] == 'axis':
                        k = Key(conv[1], self.axis_option(conv[1], ctx))
                    else:
                        kv = self.ev(kn, env, ctx)
                        if isinstance(kv, Hole):
                            holes.append(kv)
                            keyhole = True
                            break
                        k = self.to_key(kv, conv, b.dom, kn)
                    keys.append(_kv(k))
                    ktexts.append(key_text(k))
                if keyhole:
                    continue
                c = Coll()
                self.stack.append(c)
                try:
                    v = self.ev(p.value, env, ctx)
                finally:
                    self.stack.pop()
                v = coerce_to(v, p.ty)
                text = '%s%s%s' % (info.id, ('[%s]' % ', '.join(ktexts)) if ktexts else '',
                                   ''.join('.' + f for f in p.fields))
                if p.value.k not in ('num', 'neg', 'date', 'month', 'str', 'bool', 'key', 'name') or (
                        p.value.k == 'name' and p.value._r[0] == 'read'):
                    text = '%s = %s' % (text, fmt_typed(v, p.ty))
                    self.cur.merge(c)
                else:
                    text = '%s = %s' % (text, fmt_typed(v, p.ty))
                if getattr(p, '_pertuple', None):
                    # L §8.2 (round 11b): a pin formed per binding tuple prints once, as written, with its binding
                    # (`pinned: gross = g for g in gross_lo .. gross_hi step minor(DUK)`), never one line per tuple
                    text = '%s for %s' % (p._src, ', '.join(b._src for b in p._pertuple))
                pv = PinVal('set', info.id, tuple(keys) if p.target.keys is not None else None, tuple(p.fields), v,
                            text)
                pinsv = [q for q in pinsv if q.key()[:4] != pv.key()[:4]]
                pinsv.append(pv)
                if ask:
                    # an ask's with pins are printed with every line (L §5.18 The world of an ask)
                    self.reg(('pinned', text))
        return ctx.but(law=law, cut=cut, pins=tuple(sorted(pinsv, key=lambda q: repr(q.key()))),
                       axes=tuple(sorted(axes.items())), wording=tuple(wording), world=world)

    def qualify_id(self, i):
        if '.' in i:
            return i
        hit = self.m.lookup(i)
        if hit is not None and hit[1] is not None and hasattr(hit[1], 'id'):
            return hit[1].id
        return i

    # -- aggregates (L §5.11) ---------------------------------------------------------------
    def bind_day(self, b, node, env, ctx, span=False):
        """The day (or span) of a binding `on d` / `over s` over a kind. A day that is a hole binds nothing where no
        thing of the kind, known at the cut, could be alive on one of its possible days (L §5.7 Bindings, round 8: "a
        binding over `adjustments` before any adjustment exists"); a hole is unconstrained, so that is where the kind
        has no member at the cut. Returns EMPTY_BIND then."""
        try:
            v = self.span_value(node, env, ctx) if span else self.ev(node, env, ctx)
        except Unspecified as e:
            if not getattr(e, 'hole_like', False):
                raise
            if not self.kind_has_member(b, ctx, env):
                return EMPTY_BIND
            raise
        if isinstance(v, Hole) and not self.kind_has_member(b, ctx, env):
            return EMPTY_BIND
        return v

    def kind_has_member(self, b, ctx, env=None):
        """Whether any thing or movement of the binding's kind, known at the cut, could be in it on some day: one its
        `where` doesn't rule out (a member whose `where` is false is outside the binding whatever its day, so the
        domain is then decided without the day, L §8.1 "It stops propagating")."""
        m = self.m
        dom = b._dom
        if dom[0] in ('event', 'obs'):
            cands = [x.mv() for x in self.movements_of(dom, ctx) if member_moves(m, b, x)]
        elif dom[0] == 'kind':
            cands = [Thing(t.name) for t in m.things.values() if m.thing_seen(t, ctx.cut) and m.thing_in(t.name, dom[1])
                     and member_owns(m, b, t)]
        else:
            return True
        if b.where is None or env is None:
            return bool(cands)
        for v in cands:
            e2 = dict(env)
            e2[b.var] = v
            self.push()
            try:
                w = self.ev(b.where, e2, ctx)
            except TallyError:
                return True
            finally:
                self.stack.pop()
            if w is not False:
                return True
        return False

    def bind_values(self, b, env, ctx):
        """The values a binding ranges over, in binding order, and whether its span reaches
        past the cut."""
        m = self.m
        dom = b._dom
        past = False
        if dom[0] == 'index':
            ix = m.indexes[dom[1]]
            vals = [self.keyval(k) for k in ix.keys_at(ctx.cut)]
        elif dom[0] in ('event', 'obs'):
            res = self.movement_members(b, env, ctx)
            if isinstance(res, Hole):
                return None, res, False
            holes = [st for v, st in res if isinstance(st, Hole)]
            if holes:
                # L §5.14 "A movement whose day is a hole makes every flows and binding over a span it might fall in
                # a hole"; L §5.11 "A key whose where is a hole is an unknown member". Readers that keep each
                # member's status go through bind_each; here, with none kept, the unknown member leaves the
                # binding's values a hole (writer 5's T-59).
                return None, merge(*holes), False
            vals = [v for v, st in res]
        elif dom[0] == 'kind':
            kind = dom[1]
            things = [t for t in m.things.values() if m.thing_seen(t, ctx.cut) and member_owns(m, b, t)]
            unknown = {}

            def alive(t, test):
                # L §4.5: a thing whose entry is known in a window holding the cut is an unknown member, a key whose
                # where is a hole (§5.11)
                try:
                    return test(t)
                except WindowUndecided as w:
                    unknown[t.name] = w.hole()
                    return True
            if b.on is not None:
                d = self.bind_day(b, b.on, env, ctx)
                if d is EMPTY_BIND:
                    return [], None, False
                if isinstance(d, Hole):
                    return None, d, False
                d = self.date_of(d, 'the day of a binding')
                ow = getattr(b, '_owner', None)
                sel = [t for t in things if alive(t, lambda t: m.alive_on(t, kind, d, ctx.cut, ow))]
                past = d > ctx.cut
            else:
                sp = self.bind_day(b, b.over, env, ctx, span=True)
                if sp is EMPTY_BIND:
                    return [], None, False
                if isinstance(sp, Hole):
                    return None, sp, False
                ow = getattr(b, '_owner', None)
                sel = [t for t in things if alive(t, lambda t: m.alive_over(t, kind, sp, ctx.cut, ow))]
                past = sp.end is None or sp.end > ctx.cut + ONE_DAY
            sel.sort(key=lambda t: self.safe_order_key(t, kind, ctx.cut))
            vals = [Thing(t.name) for t in sel]
            b._unknown = unknown
        elif dom[0] == 'list':
            fixed = b._var.fixed
            vals = []
            for txt, kn in zip(dom[1], dom[2]):
                if fixed is not None:
                    from .loader import resolve_key_literal
                    vals.append(self.keyval(resolve_key_literal(m, kn, fixed, b.loc)))
                elif kn.k == 'num' and kn.isint:
                    vals.append(Int(int(kn.v)))
                elif kn.k == 'date':
                    vals.append(kn.v)
                elif kn.k == 'month':
                    vals.append(month_period(*kn.v))
                elif kn.k == 'quarter':
                    vals.append(quarter_period(*kn.v))
                else:
                    vals.append(ListKey(txt, kn))
        else:
            d = b.dom
            a = self.ev(d.a, env, ctx)
            z = self.ev(d.b, env, ctx)
            s = self.ev(d.step, env, ctx) if d.step is not None else None
            hs = [x for x in (a, z, s) if isinstance(x, Hole)]
            if hs:
                return None, merge(*hs), False
            vals = self.range_values(a, z, s, b)
        return vals, None, past

    def range_values(self, a, z, s, b):
        out = []
        if isinstance(a, datetime.date):
            z = self.date_of(z, 'a range end')
            cur = a
            fake = type('N', (), {})()
            fake.op = '+'
            while cur <= z:
                out.append(cur)
                nxt = self.date_arith(fake, cur, s)
                if nxt <= cur:
                    raise refuse('RUN-DOMAIN', 'a stepped range whose step is not positive')
                cur = nxt
                if len(out) > 1000000:
                    raise NotYet('a stepped range of more than a million keys (a size guard of this interpreter; L §5.6 sets no cap)')
            return out
        self.no_none(a, 'a range end')
        self.no_none(z, 'a range end')
        if s is None:
            lo, hi = self.int_of(a), self.int_of(z)
            return [Int(i) for i in range(lo, hi + 1)]
        if s.q <= 0:
            # "a step or a count that isn't positive" (L §8.4 What isn't defined is refused)
            raise refuse('RUN-DOMAIN', 'a stepped range whose step %s is not positive' % fmt(s))
        f = U.conv(z.u, a.u) if a.u != z.u else Fraction(1)
        g = U.conv(s.u, a.u) if a.u != s.u else Fraction(1)
        if f is None or g is None:
            raise refuse('LOAD-UNIT', 'a range of quantities in different units')
        cur = a.q
        stop = z.q * f
        step = s.q * g
        n = 0
        while cur <= stop:
            out.append(Num(cur, a.u, money=a.money, isint=a.isint and step.denominator == 1))
            cur += step
            n += 1
            if n > 1000000:
                raise NotYet('a stepped range of more than a million keys (a size guard of this interpreter; L §5.6 sets no cap)')
        return out

    def span_value(self, sp, env, ctx):
        a = b = None
        if sp.a is not None:
            a = self.ev(sp.a, env, ctx)
            if isinstance(a, Hole):
                return a
            if getattr(sp.a, '_year', False) and isinstance(a, Num):
                # a key-list variable of four-digit keys where a period is expected is a year (L §2.1, §2.5)
                a = year_period(int(a.q))
        if sp.b is not None:
            b = self.ev(sp.b, env, ctx)
            if isinstance(b, Hole):
                return b
            if getattr(sp.b, '_year', False) and isinstance(b, Num):
                b = year_period(int(b.q))
        start = None if a is None else self.span_start(a)
        if not sp.dots:
            return Span(start, self.span_end(a))
        end = None if b is None else self.span_end(b)
        return Span(start, end)

    def span_start(self, v):
        if isinstance(v, datetime.date):
            return v
        if isinstance(v, Period):
            return v.start
        if isinstance(v, Iv):
            raise self.imprecise(v, 'a span end')
        raise refuse('LOAD-UNIT', 'a span end is %s' % fmt(v))

    def span_end(self, v):
        if isinstance(v, datetime.date):
            return v + ONE_DAY
        if isinstance(v, Period):
            return v.end
        if isinstance(v, Iv):
            raise self.imprecise(v, 'a span end')
        raise refuse('LOAD-UNIT', 'a span end is %s' % fmt(v))

    def members(self, binds, env, ctx, i=0, status=None, possible=False, regs=None):
        """Yield (env, status) over the binding tuples in binding order; status is True, or a
        Hole when a where is a hole (an unknown member). With possible, a binding whose day or span is a hole
        yields every member it could hold, each an unknown member carrying that hole (a fill side, L §5.12). With
        regs (a list), each tuple's members' registers are kept for it, not merged: (env, status, [Coll]) (an ask's
        for, one line per tuple, L §8.2)."""
        if i == len(binds):
            if regs is not None:
                yield env, (status if status is not None else True), regs
            else:
                yield env, (status if status is not None else True)
            return
        b = binds[i]
        res = self.bind_each(b, env, ctx)
        pc = getattr(b, '_colls', None)
        b._colls = None
        if isinstance(res, Hole):
            cands = self.possible_members(b, ctx) if possible else None
            if cands is None:
                if regs is not None:
                    yield env, ('domainhole', res), regs
                else:
                    yield env, ('domainhole', res)
                return
            res = [(v, res) for v in cands]
        for v, s0 in res:
            if isinstance(s0, TallyError):
                raise s0
            e2 = dict(env)
            e2[b.var] = v
            st = status
            if isinstance(s0, Hole):
                st = s0 if st is None or st is True else merge(st, s0)
            if b.where is not None:
                w = self.ev(b.where, e2, ctx)
                if w is False:
                    continue
                if w is NONE:
                    raise refuse('RUN-NONE', 'a where is none here')
                if isinstance(w, Hole):
                    st = w if st is None or st is True else merge(st, w)
            r2 = regs
            if pc:
                mc = pc.get(v.id if isinstance(v, Mv) else getattr(v, 'name', None))
                if mc is not None:
                    if regs is not None:
                        r2 = regs + [mc]
                    else:
                        self.cur.merge(mc)
            for x in self.members(binds, e2, ctx, i + 1, st, possible, r2):
                yield x

    def safe_order_key(self, t, kind, K):
        """A thing's place in binding order: the text order of its stable id (L §2.5)."""
        return self.m.kind_order_key(t, kind, K)

    def possible_members(self, b, ctx):
        """Every member a binding over a kind, an event kind or an observed input could hold whatever its day or span:
        the things of the kind known at the cut, or the timeline's movements of the kind; None for another domain."""
        m = self.m
        dom = b._dom
        if dom[0] == 'kind':
            things = [t for t in m.things.values() if m.thing_seen(t, ctx.cut) and m.thing_in(t.name, dom[1]) and
                      member_owns(m, b, t)]
            things.sort(key=lambda t: m.kind_order_key(t, dom[1], ctx.cut))
            return [Thing(t.name) for t in things]
        if dom[0] in ('event', 'obs'):
            return [x.mv() for x in self.movements_of(dom, ctx) if member_moves(m, b, x)]
        return None

    def zero_of(self, t):
        if t.k == 'dyn':
            # 0 of a unit known only at run: it takes its unit from the terms, or its target's (L §7.1 bare 0)
            return Int(0)
        if t.k == 'money':
            return Money(0, t.cur)
        if t.k in ('int', 'flex', 'key'):
            return Int(0)
        if t.k == 'rate':
            # 0 in the term's type (L §5.11): a rate's period is in its value, and there is none (L §8.4 What isn't defined is refused)
            raise refuse('RUN-DOMAIN', 'the sum of rates over no keys (a rate carries its period)')
        m = self.m
        if any(b in m.kinds and m.kinds[b].spec == 'unit' for b, e in (t.u or ())):
            # L §3.2 A key that load can't see: "A 0 with no key (a bare 0, a sum over no keys, ...) takes the key of
            # what it meets" (fault F1)
            return Int(0)
        return Num(0, t.u or ())

    def e_agg(self, n, env, ctx):
        fn = n.fn
        if fn in ('first', 'last'):
            return self.agg_first(n, env, ctx)
        mem = list(self.members(n.binds, env, ctx))
        for e2, st in mem:
            if isinstance(st, tuple) and st[0] == 'domainhole':
                return st[1]
        if fn == 'count' and n.term is None:
            holes = [st for e2, st in mem if isinstance(st, Hole)]
            if holes:
                return merge(*holes)
            return Int(len(mem))
        if fn in ('any', 'all'):
            return self.agg_anyall(n, mem, ctx)
        terms = []
        holes = []
        snap = self.trace_snap() if fn in ('min', 'max', 'argmin', 'argmax') and n.term is not None else None
        for e2, st in mem:
            if n.term is None:
                t = self.bare_key(n, e2)
            else:
                t = self.ev(n.term, e2, ctx)
            if isinstance(st, Hole):
                if fn == 'sum' and isinstance(t, Num) and t.q == 0:
                    continue
                holes.append(st)
                if isinstance(t, Hole):
                    holes.append(t)
                continue
            if isinstance(t, Hole):
                holes.append(t)
                continue
            if t is NONE:
                raise refuse('RUN-NONE', 'a term of %s is none here' % fn)
            terms.append((e2, t))
        if fn == 'count':
            if holes:
                return merge(*holes)
            return Int(sum(1 for e2, t in terms if t is True))
        if fn == 'sum':
            if holes:
                if ctx.partial:
                    acc = self.zero_of(n._t)
                    for e2, t in terms:
                        acc = arith('+', acc, t, n._t, n._t, n.term._t)
                    out = Hole(set())
                    lin = (acc, ())
                    ok = True
                    for h in holes:
                        lh = lin_of(h)
                        if lh is None:
                            ok = False
                        else:
                            lin = lin_add(lin, lh)
                    qs = set().union(*[h.qs for h in holes])
                    return Hole(qs, lin if ok else False)
                return merge(*holes)
            acc = self.zero_of(n._t)
            for e2, t in terms:
                acc = arith('+', acc, t, n._t, n._t, n.term._t)
            if n._t.k == 'money' and not acc.money:
                acc = Money(acc.q, n._t.cur)
            return acc
        if holes:
            return merge(*holes)
        if not terms:
            raise refuse('RUN-EMPTY', '%s over no keys' % fn)
        if fn in ('min', 'max'):
            res = self.agg_minmax(fn, [t for e2, t in terms], n)
            if snap is not None:
                self.trace_winner(snap, res, [(lambda e2=e2: self.ev(n.term, e2, ctx), t) for e2, t in terms])
            return res
        if fn in ('argmin', 'argmax'):
            res = self.agg_arg(n, terms, ctx)
            if snap is not None:
                self.trace_winner(snap, res, [(lambda e2=e2: self.ev(n.term, e2, ctx), self.bare_key(n, e2))
                                              for e2, t in terms])
            return res
        raise NotYet(fn)

    def trace_snap(self):
        """Where the trace stands before a min, max, argmin or argmax evaluates its terms (explain only)."""
        if not self.tracing or not self.trace_stack:
            return None
        parent = self.trace_stack[-1]
        return (parent, len(parent.children), set(self.trace_cells))

    def trace_winner(self, snap, res, terms):
        """L §5.18 Explanation order: "A min, max, argmin or argmax is explained by its first winning term in
        binding order" (E §10). terms: [(re-evaluate the term, what it gives the result: its value, or its key for
        argmin/argmax)], in binding order. The causes the terms traced are dropped and the first term whose value is
        the result is traced again, alone."""
        if snap is None or isinstance(res, (Hole, Iv, TallyError)):
            return
        win = None
        for f, v in terms:
            if not isinstance(v, (Hole, Iv)) and vkey(v) == vkey(res):
                win = f
                break
        if win is None:
            return
        parent, n0, cells0 = snap
        del parent.children[n0:]
        for k in [k for k in self.trace_cells if k not in cells0]:
            del self.trace_cells[k]
        try:
            win()
        except (TallyError, HoleSignal):
            pass

    def bare_key(self, n, e2):
        vs = [e2[b.var] for b in n.binds]
        return vs[0] if len(vs) == 1 else Tup(vs)

    def agg_minmax(self, fn, ts, n):
        best = ts[0]
        anyiv = any(isinstance(t, Iv) for t in ts)
        if anyiv:
            los = [t.lo if isinstance(t, Iv) else t for t in ts]
            his = [t.hi if isinstance(t, Iv) else t for t in ts]
            pick = (lambda xs: self.extreme(xs, fn))
            lo, hi = pick(los), pick(his)
            src = [t for t in ts if isinstance(t, Iv)][0].src
            if self.cmp3(lo, hi, None) == 0:
                return lo
            return Iv(lo, hi, None, src)
        return self.extreme(ts, fn, n)

    def extreme(self, ts, fn, n=None):
        best = ts[0]
        for t in ts[1:]:
            c = self.cmp3(t, best, n if n is not None and getattr(n, 'term', None) is None else None)
            if (fn == 'min' and c < 0) or (fn == 'max' and c > 0):
                best = t
        if isinstance(best, Num) and any(isinstance(t, Num) and t.enc is not None for t in ts):
            # the interval of a min or max (L §7.3 The error bound)
            encs = [enc_of(t, best.u) for t in ts if isinstance(t, Num)]
            if any(e[0] is None for e in encs):
                e = (None, None)
            elif fn == 'min':
                e = (min(a for a, b in encs), min(b for a, b in encs))
            else:
                e = (max(a for a, b in encs), max(b for a, b in encs))
            best = Num(best.q, best.u, best.money, best.isint, best.rate, enc=e)
        return best

    def agg_arg(self, n, terms, ctx):
        fn = n.fn
        best = None
        ivs = [t for e2, t in terms if isinstance(t, Iv)]
        if ivs:
            # windows: "argmin and argmax give the key whose value is decided below (above) every other key's,
            # and otherwise HOLE-IMPRECISE" (L §5.11, §4.4)
            def lo(t):
                return t.lo if isinstance(t, Iv) else t

            def hi(t):
                return t.hi if isinstance(t, Iv) else t
            wins = []
            for i, (e2, t) in enumerate(terms):
                ok = True
                for j, (e3, u) in enumerate(terms):
                    if i == j:
                        continue
                    if fn == 'argmin' and not self.cmp3(hi(t), lo(u), None) < 0:
                        ok = False
                        break
                    if fn == 'argmax' and not self.cmp3(lo(t), hi(u), None) > 0:
                        ok = False
                        break
                if ok:
                    wins.append(e2)
            if len(wins) == 1:
                return self.bare_key(n, wins[0])
            raise self.imprecise(ivs[0], fn)
        for e2, t in terms:
            if best is None:
                best = [(e2, t)]
                continue
            c = self.cmp3(t, best[0][1], None)
            if (fn == 'argmin' and c < 0) or (fn == 'argmax' and c > 0):
                best = [(e2, t)]
            elif c == 0:
                best.append((e2, t))
        if len(best) > 1:
            best = self.tie_break(n, best, ctx)
        return self.bare_key(n, best[0][0])

    def tie_break(self, n, tied, ctx):
        if not n.tie:
            raise refuse('RUN-TIE', '%s has a tie at %s that changes the result, and no ties by' % (
                n.fn, ', '.join(fmt(self.bare_key(n, e2)) for e2, t in tied)))
        scored = []
        for e2, t in tied:
            vals = [self.ev(x, e2, ctx) for x in n.tie]
            for v in vals:
                if isinstance(v, Hole):
                    return [(e2, v)]
            scored.append((vals, e2, t))
        best = [scored[0]]
        for s in scored[1:]:
            c = 0
            for a, b in zip(s[0], best[0][0]):
                c = self.cmp3(a, b, None)
                if c:
                    break
            if c < 0:
                best = [s]
            elif c == 0:
                best.append(s)
        if len(best) > 1:
            raise refuse('RUN-TIE', '%s has a tie at %s that changes the result, and its ties by doesn\'t break it' % (
                n.fn, ', '.join(fmt(self.bare_key(n, s[1])) for s in best)))
        return [(best[0][1], best[0][2])]

    def unordered_what(self, b):
        """What a binding ranges over when it has no order a rule can read (L §5.7, §5.6): 'things', 'movements',
        'open-index keys', else None."""
        dom = b._dom
        if dom[0] == 'kind':
            return 'things'
        if dom[0] in ('event', 'obs'):
            return 'movements'
        if dom[0] == 'index' and self.m.indexes[dom[1]].form == 'open':
            return 'open-index keys'
        return None

    def agg_first(self, n, env, ctx):
        """first and last (L §5.11): the first (last) binding tuple whose where is true, in binding order; over things,
        movements or an open index's keys in the order of their ties by, without which they are refused (RUN-TIE).
        Empty: HOLE-NOTYET when a binding of things or movements reaches past the cut (L §3.1), else RUN-EMPTY."""
        assert n.term is None   # refused at load, LOAD-PARSE (L §5.11)
        un = [w for w in (self.unordered_what(b) for b in n.binds) if w]
        if un and not n.tie:
            # L §5.11 Ties are stated; L §8.4 RUN-TIE: "<first or last> over <things, movements or open-index keys>
            # has no ties by"
            raise refuse('RUN-TIE', '%s over %s has no ties by' % (n.fn, un[0]))
        mem = list(self.members(n.binds, env, ctx, regs=[]))
        for e2, st, rg in mem:
            if isinstance(st, tuple) and st[0] == 'domainhole':
                return st[1]
        if un:
            groups = self.tie_groups(n, mem, ctx)
            if isinstance(groups, Hole):
                return groups
        else:
            groups = [[x] for x in mem]
        if n.fn == 'last':
            groups = list(reversed(groups))
        holes = []
        for g in groups:
            for e2, st, rg in g:
                for c in rg:
                    self.cur.merge(c)
            trues = [e2 for e2, st, rg in g if st is True]
            holes.extend(st for e2, st, rg in g if isinstance(st, Hole))
            if not trues:
                continue
            if holes:
                return merge(*holes)
            if len(trues) > 1:
                raise refuse('RUN-TIE', "%s has a tie at %s that changes the result, and its ties by doesn't break it" % (
                    n.fn, ', '.join(fmt(self.bare_key(n, e2)) for e2 in trues)))
            return self.bare_key(n, trues[0])
        if holes:
            return merge(*holes)
        b = self.notyet_bind(n.binds, env, ctx)
        if b is not None:
            # L §3.1, §5.11, §8.1 HOLE-NOTYET
            return Hole([('%s: no %s yet, and its span reaches past the cut' % (b.var, b._dom[1]), b._dom[1])])
        raise refuse('RUN-EMPTY', '%s over no keys' % n.fn)

    def tie_groups(self, n, mem, ctx):
        """The members in the order of the ties by term, those with equal terms in one group (L §5.11); a Hole when a
        member's term is one (its place in the order is unknown)."""
        import functools
        scored = []
        for e2, st, rg in mem:
            vals = [self.ev(x, e2, ctx) for x in n.tie]
            hs = [v for v in vals if isinstance(v, Hole)]
            if hs:
                return merge(*(hs + ([st] if isinstance(st, Hole) else [])))
            for v in vals:
                if v is NONE:
                    raise refuse('RUN-NONE', 'the ties by of %s is none here' % n.fn)
            scored.append((vals, (e2, st, rg)))

        def cmp(a, b):
            for x, y in zip(a[0], b[0]):
                c = self.cmp3(x, y, None)
                if c:
                    return c
            return 0
        scored.sort(key=functools.cmp_to_key(cmp))
        groups = []
        for s in scored:
            if groups and cmp(groups[-1][0], s) == 0:
                groups[-1][1].append(s[1])
            else:
                groups.append((s, [s[1]]))
        return [g[1] for g in groups]

    def notyet_bind(self, binds, env, ctx):
        """The first binding of things or movements whose day or span reaches past the cut (L §3.1), or None."""
        from .mreads import reads_var
        earlier = []
        for b in binds:
            dep = any(reads_var(x, v) for v in earlier for x in (b.on, b.over) if x is not None)
            earlier.append(b.var)
            if dep or b._dom[0] not in ('kind', 'event', 'obs'):
                continue
            if b.on is not None:
                d = self.bind_day(b, b.on, env, ctx)
                if d is EMPTY_BIND or isinstance(d, Hole) or d is NONE:
                    continue
                if self.date_of(d, 'the day of a binding') > ctx.cut:
                    return b
            elif b.over is not None:
                sp = self.bind_day(b, b.over, env, ctx, span=True)
                if sp is EMPTY_BIND or isinstance(sp, Hole):
                    continue
                if sp.end is None or sp.end > ctx.cut + ONE_DAY:
                    return b
        return None

    def agg_anyall(self, n, mem, ctx):
        fn = n.fn
        holes = []
        for e2, st in mem:
            t = self.ev(n.term, e2, ctx)
            if t is NONE:
                raise refuse('RUN-NONE', 'a term of %s is none here' % fn)
            if isinstance(st, Hole):
                holes.append(st)
                if isinstance(t, Hole):
                    holes.append(t)
                continue
            if isinstance(t, Hole):
                holes.append(t)
                continue
            if fn == 'any' and t is True:
                return True
            if fn == 'all' and t is False:
                return False
        if holes:
            return merge(*holes)
        return fn == 'all'

    def e_query(self, n, env, ctx):
        if n.binds is None:
            return self.e_query_random(n, env, ctx)
        mem = list(self.members(n.binds, env, ctx))
        for e2, st in mem:
            if isinstance(st, tuple) and st[0] == 'domainhole':
                return st[1]
        holes = []
        terms = []
        for e2, st in mem:
            t = self.ev(n.arg, e2, ctx)
            if isinstance(st, Hole):
                holes.append(st)
            if isinstance(t, Hole):
                holes.append(t)
                continue
            if t is NONE:
                raise refuse('RUN-NONE', 'a term of %s is none here' % n.fn)
            terms.append(t)
        if holes:
            return merge(*holes)
        if not terms:
            raise refuse('RUN-EMPTY', '%s over no keys' % n.fn)
        if n.fn == 'share':
            return Num(Fraction(sum(1 for t in terms if t is True), len(terms)), ())
        if n.fn == 'mean':
            acc = terms[0]
            for t in terms[1:]:
                acc = arith('+', acc, t, n.arg._t, n.arg._t, n.arg._t)
            # the enclosure rides with the value (L §7.3 The error bound)
            nn = Num(len(terms), (), isint=True)
            return Num(acc.q / len(terms), acc.u, enc=enc_arith('/', acc, nn, 1) if acc.enc is not None else None)
        q = self.ev(n.q, env, ctx)
        if isinstance(q, Hole):
            return q
        srt = sorted(terms, key=lambda t: t.q if isinstance(t, Num) else t)
        need = q.q * len(srt)
        for i, t in enumerate(srt, 1):
            if i >= need:
                return t
        return srt[-1]

    # -- calls ----------------------------------------------------------------------------------
    def e_call(self, n, env, ctx):
        r = n._r
        if r[0] == 'band':
            return self.call_band(n, env, ctx)
        if r[0] == 'memberband':
            return self.member_band(n, env, ctx)
        name = r[1]
        f = getattr(self, 'c_' + name, None)
        if f is None:
            if name == 'since':
                return self.e_call_since(n, env, ctx)
            raise NotYet(name, n.loc)
        return f(n, env, ctx)

    def args(self, n, env, ctx):
        return [self.ev(a, env, ctx) for a in n.args]

    def hole_args(self, vals, strict=None):
        hs = [v for v in vals if isinstance(v, Hole)]
        if hs:
            if strict is not None:
                # a refusal outranks a hole (L §8.4): a none argument is refused whatever the hole holds
                for v in vals:
                    if not isinstance(v, Hole):
                        self.no_none(v, strict)
            return merge(*hs)
        return None

    def as_date_or_iv(self, v, what):
        if isinstance(v, (datetime.date, Iv)):
            return v
        if isinstance(v, Period) and v.kind == 'day':
            return v.start
        if v is NONE:
            raise refuse('RUN-NONE', 'a value is none here: %s' % what)
        if v is NEVER:
            raise refuse('RUN-NONE', 'never in %s' % what)
        raise refuse('LOAD-UNIT', '%s is %s' % (what, fmt(v)))

    def lift(self, fn, *vals, src=None):
        """A built-in on a window gives the interval of its values at the first and last day (L §4.4)."""
        ivs = [v for v in vals if isinstance(v, Iv)]
        if not ivs:
            return fn(*vals)
        if len(ivs) > 1:
            raise self.imprecise(ivs[0], 'a built-in of two windows')
        lo_args = [v.lo if isinstance(v, Iv) else v for v in vals]
        hi_args = [v.hi if isinstance(v, Iv) else v for v in vals]
        a, b = fn(*lo_args), fn(*hi_args)
        if a is None or b is None or a is NONE or b is NONE:
            raise self.imprecise(ivs[0], 'a built-in that is none on part of the window')
        if self.cmp3(a, b, None) == 0:
            return a
        if order_lt(b, a):
            a, b = b, a
        return Iv(a, b, None, ivs[0].src, origin=ivs[0].origin)

    def period_arg(self, v, what):
        if isinstance(v, Period):
            return v
        if isinstance(v, datetime.date):
            return day_period(v)
        if v is NONE:
            raise refuse('RUN-NONE', 'a value is none here: %s' % what)
        raise refuse('LOAD-UNIT', '%s is %s, not a period' % (what, fmt(v)))

    def c_start_of(self, n, env, ctx):
        v, = self.args(n, env, ctx)
        if isinstance(v, Hole):
            return v
        if isinstance(v, Iv):
            return self.lift(lambda d: d, v)
        p = self.period_arg(v, 'start_of')
        return p.start

    def c_end_of(self, n, env, ctx):
        v, = self.args(n, env, ctx)
        if isinstance(v, Hole):
            return v
        if isinstance(v, Iv):
            return self.lift(lambda d: d + ONE_DAY, v)
        p = self.period_arg(v, 'end_of')
        return p.end

    def _of(self, n, env, ctx, f):
        v, = self.args(n, env, ctx)
        if isinstance(v, Hole):
            return v
        if isinstance(v, Iv):
            return self.lift(f, v)
        if isinstance(v, Period):
            return f(v.start)
        return f(self.date_of(v, n.name))

    def c_year_of(self, n, env, ctx):
        return self._of(n, env, ctx, year_of)

    def c_quarter_of(self, n, env, ctx):
        return self._of(n, env, ctx, quarter_of)

    def c_month_of(self, n, env, ctx):
        return self._of(n, env, ctx, month_of)

    def c_day_of(self, n, env, ctx):
        a, b = self.args(n, env, ctx)
        h = self.hole_args([a, b])
        if h:
            return h
        if n._form == 'month':
            p = self.period_arg(a, 'day_of')
            k = self.int_of(b)
            y, mo = p.start.year, p.start.month
            if p.kind != 'month' or p.start.day != 1:
                # L §4.2: "Of a key of an anchored index (§4.1) it is refused (RUN-DOMAIN)"
                raise refuse('RUN-DOMAIN', 'day_of of %s, a key of an anchored index' % fmt(p))
            return datetime.date(y, mo, max(1, min(k, last_day(y, mo)))) if k >= 1 else self._bad_day(k)
        p = self.period_arg(a, 'day_of')
        if p.kind == 'year' and (p.start.month, p.start.day) != (1, 1):
            # L §4.2: "Of a key of an anchored index (§4.1) it is refused (RUN-DOMAIN)"
            raise refuse('RUN-DOMAIN', 'day_of of %s, a key of an anchored index' % fmt(p))
        mm, dd = b.m, b.d
        try:
            return datetime.date(p.start.year, mm, dd)
        except ValueError:
            # only day_of(m, n) clamps (L §4.2) (L §8.4 What isn't defined is refused)
            raise refuse('RUN-DOMAIN', 'day_of(y, %02d-%02d) in a year without that day' % (mm, dd))

    def _bad_day(self, k):
        raise refuse('RUN-DOMAIN', 'day_of(m, %d): a day number below 1' % k)

    def c_days_between(self, n, env, ctx):
        a, b = self.args(n, env, ctx)
        h = self.hole_args([a, b])
        if h:
            return h
        a = self.as_date_or_iv(a, 'days_between')
        b = self.as_date_or_iv(b, 'days_between')
        return self.lift(lambda x, y: Int((y - x).days), a, b)

    def c_add_days(self, n, env, ctx):
        a, k = self.args(n, env, ctx)
        h = self.hole_args([a, k])
        if h:
            return h
        a = self.as_date_or_iv(a, 'add_days')
        kk = self.int_of(k)
        return self.lift(lambda x: add_days(x, kk), a)

    def months_arg(self, k, per, mult):
        if isinstance(k, Num) and k.u:
            if k.u != ((per, 1),):
                raise refuse('LOAD-UNIT', 'a duration of %s for add_%ss' % (U.fmt(k.u), per))
            return self.int_of(Num(k.q, ())) * mult
        return self.int_of(k) * mult

    def c_add_months(self, n, env, ctx):
        a, k = self.args(n, env, ctx)
        h = self.hole_args([a, k])
        if h:
            return h
        a = self.as_date_or_iv(a, 'add_months')
        mm = self.months_arg(k, 'month', 1)
        return self.lift(lambda x: add_months(x, mm), a)

    def c_add_years(self, n, env, ctx):
        a, k = self.args(n, env, ctx)
        h = self.hole_args([a, k])
        if h:
            return h
        a = self.as_date_or_iv(a, 'add_years')
        mm = self.months_arg(k, 'year', 12)
        return self.lift(lambda x: add_months(x, mm), a)

    def c_overlap_days(self, n, env, ctx):
        a, b, k = self.args(n, env, ctx)
        h = self.hole_args([a, b, k])
        if h:
            return h
        a = self.as_date_or_iv(a, 'overlap_days')
        b = self.as_date_or_iv(b, 'overlap_days')
        p = self.period_arg(k, 'overlap_days')
        if isinstance(a, Iv) and isinstance(b, Iv):
            raise self.imprecise(a, 'overlap_days of two windows')
        return self.lift(lambda x, y: Int(overlap_days(x, y, p)), a, b)

    def c_age(self, n, env, ctx):
        b, d = self.args(n, env, ctx)
        h = self.hole_args([b, d])
        if h:
            return h
        b = self.as_date_or_iv(b, 'age')
        d = self.as_date_or_iv(d, 'age')
        fn = age if n.name == 'age' else age_months

        def f(x, y):
            r = fn(x, y)
            return NONE if r is None else Int(r)
        if not isinstance(b, Iv) and not isinstance(d, Iv):
            return f(b, d)
        return self.lift(f, b, d)

    c_age_months = c_age

    def c_weekday(self, n, env, ctx):
        d, = self.args(n, env, ctx)
        if isinstance(d, Hole):
            return d
        d = self.as_date_or_iv(d, 'weekday')
        if isinstance(d, Iv):
            if d.days is None and (d.hi - d.lo).days >= 6:
                return Iv(Int(1), Int(7), None, d.src)
            days = sorted(d.days) if d.days else [d.lo + datetime.timedelta(days=i)
                                                  for i in range((d.hi - d.lo).days + 1)]
            ws = sorted(set(x.isoweekday() for x in days))
            if len(ws) == 1:
                return Int(ws[0])
            return Iv(Int(min(ws)), Int(max(ws)), None, d.src)
        return Int(d.isoweekday())

    def c_pos(self, n, env, ctx):
        v, = self.args(n, env, ctx)
        if isinstance(v, Hole):
            return v
        ix = self.m.indexes.get(n.args[0]._t.ix) or (self.m.axes.get(n.args[0]._t.ix))
        if isinstance(v, OutKey):
            raise refuse('RUN-KEY', '%s is outside %s' % (v.text, v.index))
        if hasattr(ix, 'pos'):
            k = self.int_of(v) if isinstance(v, Num) else v
            p = ix.pos(k)
            if p is None:
                raise refuse('RUN-KEY', '%s is outside %s' % (key_text(k), ix.id))
            return Int(p + 1)
        return Int(ix.options.index(v.name) + 1)

    def c_window(self, n, env, ctx):
        a, b = self.args(n, env, ctx)
        h = self.hole_args([a, b])
        if h:
            return h
        a, b = self.date_of(a, 'window'), self.date_of(b, 'window')
        if a > b:
            # named in L §8.4 What isn't defined is refused
            raise refuse('RUN-DOMAIN', 'window(%s, %s) with a after b' % (a, b))
        return self.window_choice(Iv(a, b, None, 'window(%s, %s)' % (a, b)), ctx)

    def c_one_of(self, n, env, ctx):
        v, = self.args(n, env, ctx)
        if isinstance(v, Hole):
            return v
        ds = frozenset(self.date_of(x, 'one_of') for x in v.items)
        if not ds:
            raise refuse('RUN-DOMAIN', 'one_of of no days')
        return self.window_choice(Iv(min(ds), max(ds), ds, 'one_of([%s])' % ', '.join(
            d.isoformat() for d in sorted(ds))), ctx)

    def c_min(self, n, env, ctx):
        snap = self.trace_snap()
        vals = self.args(n, env, ctx)
        h = self.hole_args(vals, strict=n.name)
        if h:
            return h
        for v in vals:
            self.no_none(v, n.name)
        vals = [coerce_flex(v, a._t, vals, n) for v, a in zip(vals, n.args)]
        res = self.agg_minmax(n.name, vals, None)
        if snap is not None:
            # L §5.18: explained by its first winning term, the arguments in the order written
            self.trace_winner(snap, res, [(lambda a=a: self.ev(a, env, ctx), v) for a, v in zip(n.args, vals)])
        return res

    c_max = c_min

    def c_abs(self, n, env, ctx):
        v, = self.args(n, env, ctx)
        if isinstance(v, Hole):
            return v
        self.no_none(v, 'abs')
        return Num(abs(v.q), v.u, money=v.money, isint=v.isint, rate=v.rate, enc=enc_abs(v.enc))

    def c_pow(self, n, env, ctx):
        x, k = self.args(n, env, ctx)
        h = self.hole_args([x, k], strict='pow')
        if h:
            return h
        self.no_none(x, 'pow')
        self.no_none(k, 'pow')
        if getattr(n, '_realpow', False) and (k.q.denominator != 1 or k.enc is not None or x.enc is not None):
            return self.trans_call(n, [x, k], None, ctx)
        kk = self.int_of(k)
        big = max(abs(x.q.numerator).bit_length(), x.q.denominator.bit_length())
        if big > 1 and (big - 1) * abs(kk) * 0.30102999 > DIGIT_CAP + 10:
            raise refuse('RUN-DIGITS', 'pow(%s, %d) passes 4 096 digits; round where the law rounds' % (fmt(x), kk))
        if x.q == 0 and kk < 0:
            raise refuse('RUN-DIV', 'division by zero: pow(0, %d)' % kk)
        q = x.q ** kk
        check_digits(q)
        return Num(q, U.power(x.u, kk) if x.u else (), isint=(x.isint and kk >= 0) and False,
                   enc=enc_pow(x.enc, kk))

    def c_minor(self, n, env, ctx):
        c = n.args[0].v
        return Money(CURRENCIES[c], c)

    def c_round(self, n, env, ctx):
        """round(x, u, mode): the multiple of u the mode chooses (L §7.2)."""
        x = self.ev(n.args[0], env, ctx)
        u = self.ev(n.args[1], env, ctx)
        mn = n.args[2]
        if getattr(mn, '_r', None) and mn._r[0] == 'axismode':
            mode = Code('rounding', self.axis_option(mn._r[1], ctx))
        else:
            mode = self.ev(mn, env, ctx)
        h = self.hole_args([x, u, mode])
        if h:
            return h
        self.no_none(x, 'round')
        self.no_none(u, 'round')
        self.no_none(mode, 'round')
        if isinstance(x, Iv):
            raise self.imprecise(x, 'round')
        if not isinstance(mode, Code) or mode.kind != 'rounding':
            raise refuse('LOAD-UNIT', 'round: the mode is %s' % fmt(mode))
        if u.q <= 0:
            # "a step or a count that isn't positive" is refused (L §8.4 What isn't defined is refused)
            raise refuse('RUN-DOMAIN', 'round with a step %s that is not positive' % fmt(u))
        if getattr(n.args[1], '_stepflex', False):
            u = Num(u.q, x.u)
        f = U.conv(u.u, x.u) if u.u != x.u else Fraction(1)
        if f is None:
            if getattr(n, '_dyn', False):
                # a unit load couldn't show differs at this key (L §3.2, §8.4)
                raise refuse('RUN-DOMAIN', 'round: the step %s and %s have different units' % (fmt(u), fmt(x)))
            raise refuse('LOAD-UNIT', 'round: the step %s and %s have different units' % (fmt(u), fmt(x)))
        step = u.q * f
        k = round_mode(x.q / step, mode.word)
        q = k * step
        cur = getattr(n, '_money', None)
        if cur is not None:
            minor = CURRENCIES[cur]
            if (q / minor).denominator != 1:
                raise refuse('LOAD-UNIT', 'round to money: the step %s is not a whole number of minor units' % fmt(u))
            return Money(q, cur)
        if getattr(n, '_int', False):
            # round(x, 1, mode) of a dimensionless num is an int (L §7.2 Its type)
            return Int(q)
        return Num(q, x.u)

    def c_lifespan(self, n, env, ctx):
        v, = self.args(n, env, ctx)
        if isinstance(v, Hole):
            return v
        self.no_none(v, 'lifespan')
        t = self.m.things.get(v.name)
        sp = self.m.lifespan(t, ctx.cut)
        if sp is None:
            raise refuse('RUN-KEY', '%s was added after the cut' % v.name)
        if sp.start is None:
            # unreachable: a lifespan has a start (L §5.7, LOAD-LIFESPAN at load)
            raise NotYet('lifespan(%s) of a lifespan with no start' % v.name)
        to = NEVER if sp.end is None else sp.end - ONE_DAY
        if sp.end_q is not None:
            # an end not yet known: the stated hole (L §5.7, round 18 C1-8; HOLE-STATED)
            to = Hole([(sp.end_q, v.name)])
        return Rec([('from', sp.start), ('to', to)])

    def c_missing(self, n, env, ctx):
        """missing(r): true when r is a hole because no row gives it, or its row is a stated ?;
        never itself a hole (L §5.11)."""
        r = n.args[0]
        info = self.m.inputs[r._r[2]]
        c = Coll()
        self.stack.append(c)
        try:
            keys, hole = self.key_values(info, r.keys if r.k == 'index' else [], r._convs, env, ctx)
            if hole is not None:
                res = False
            else:
                v = self.read_input(info, keys, ctx, probe=True)
                res = isinstance(v, tuple) and v[0] == 'missing'
        finally:
            self.stack.pop()
        self.cur.merge(c)
        if res:
            self.cur.missing.append(info.id)
        return res

    # -- calendars (L §4.2) -------------------------------------------------------------------------
    def cal_at(self, node, env, ctx, day):
        """Read the calendar input at the day asked about."""
        r = node._r
        info = self.m.inputs[r[2]]
        keys, hole = self.key_values(info, node.keys if node.k == 'index' else [], node._convs, env, ctx)
        if hole is not None:
            return hole
        return self.read_input(info, keys, ctx.but(law=day), calendar=True)

    def business(self, node, env, ctx, day):
        cal = self.cal_at(node, env, ctx, day)
        if isinstance(cal, Hole):
            return cal
        if cal is NONE:
            raise refuse('RUN-NONE', 'the calendar is none here')
        wd = day.isoweekday()
        if any(WEEKDAYS[w] == wd for w in cal.weekends):
            return False
        if (day.month, day.day) in cal.yearly and (cal.yfrom is None or day.year >= cal.yfrom):
            return False
        unknown = []
        for a in cal.added:
            if a[0] == 'day' and a[1] == day:
                return False
            if a[0] == 'hole':
                unknown.append(Hole([(a[1], node._r[2])]))
            if (a[0] == 'window' and a[1] <= day <= a[2]) or (a[0] == 'oneof' and day in a[1]):
                # L §4.2: a day in added may be a window, "every day known but one. Such a window is one the line
                # meets, so a result that depends on its day is HOLE-IMPRECISE unless the line states windows each"
                if a[0] == 'window':
                    wt = 'window(%s, %s)' % (a[1], a[2])
                    iv = Iv(a[1], a[2], None, wt, origin=('calendar %s' % node._r[2], 'added %s' % wt))
                    days = [a[1] + datetime.timedelta(days=i) for i in range((a[2] - a[1]).days + 1)]
                else:
                    wt = 'one_of([%s])' % ', '.join(str(x) for x in sorted(a[1]))
                    iv = Iv(min(a[1]), max(a[1]), set(a[1]), wt, origin=('calendar %s' % node._r[2], 'added %s' % wt))
                    days = sorted(a[1])
                if ctx.windows in ('earliest', 'latest') or (isinstance(ctx.windows, tuple) and ctx.windows and
                                                             ctx.windows[0] == 'each'):
                    # L §4.4: windows each splits the line by it, "a day in a calendar's added list (§4.2)"
                    if self.window_choice(iv, ctx) == day:
                        return False
                    continue
                if self.cal_choice is not None:
                    # evaluated at each day the window can take (over_added): this pass's day, or the first while
                    # the window is newly met (that pass is run again per day)
                    k = (node._r[2], wt)
                    if k not in self.cal_choice:
                        self.cal_met.setdefault(k, days)
                    if self.cal_choice.get(k, days[0]) == day:
                        return False
                    continue
                unknown.append(Hole([('date within %s not precise enough for is_business(%s, %s)' % (
                    wt, day, _ntxt(node)), node._r[2])]))
        if unknown:
            return merge(*unknown)
        return True

    def over_added(self, n, fn):
        """A business-day function over a calendar whose added list holds a window, evaluated at each day the window
        can take (L §4.2 "every day known but one"; one pass per day of each window it meets, nested where a pass meets
        another). One value: that value. Dates that differ: the interval of the days it can take, a window like any
        other (L §4.4: "Built-ins on a window ... give the interval of their values"; "Only a comparison decides an
        interval"), so a movement dated by it counts in a span that holds the whole interval (L §4.4, §5.14).
        is_business that differs: HOLE-IMPRECISE (L §4.2: "a result that depends on its day is HOLE-IMPRECISE unless
        the line states windows each"; L §8.1)."""
        outer, outer_met = self.cal_choice, self.cal_met
        todo = [dict(outer) if outer is not None else {}]
        out = []
        first_met = None
        try:
            while todo:
                ch = todo.pop(0)
                self.cal_choice = ch
                self.cal_met = {}
                v = fn()
                new = sorted(k for k in self.cal_met if k not in ch)
                if new:
                    k = new[0]
                    if first_met is None:
                        first_met = k
                    for d in self.cal_met[k]:
                        c2 = dict(ch)
                        c2[k] = d
                        todo.append(c2)
                    continue
                out.append(v)
        finally:
            self.cal_choice, self.cal_met = outer, outer_met
        holes = [v for v in out if isinstance(v, Hole)]
        if holes:
            return merge(*holes)
        v0 = out[0]
        if all(vkey(v) == vkey(v0) for v in out[1:]):
            return v0
        if isinstance(v0, bool):
            cal_id, wt = first_met
            return Hole([('date within %s not precise enough for %s' % (wt, _ntxt(n)), cal_id)])
        return Iv(min(out), max(out), None, None)

    def c_is_business(self, n, env, ctx):
        d = self.ev(n.args[0], env, ctx)
        if isinstance(d, Hole):
            return d
        d = self.date_of(d, 'is_business')
        return self.over_added(n, lambda: self.business(n.args[1], env, ctx, d))

    def c_add_business(self, n, env, ctx):
        d = self.ev(n.args[0], env, ctx)
        k = self.ev(n.args[1], env, ctx)
        h = self.hole_args([d, k])
        if h:
            return h
        d = self.date_of(d, 'add_business')
        k = self.int_of(k)
        if k == 0:
            raise refuse('RUN-ZERO', 'add_business(%s, 0, %s)' % (d, _ntxt(n.args[2])))
        step = 1 if k > 0 else -1

        def count():
            left = abs(k)
            cur = d
            while left:
                cur = cur + datetime.timedelta(days=step)
                b = self.business(n.args[2], env, ctx, cur)
                if isinstance(b, Hole):
                    return b
                if b:
                    left -= 1
            return cur
        return self.over_added(n, count)

    def c_last_business(self, n, env, ctx):
        """the n-th last business day of period p (L §4.2)"""
        p = self.ev(n.args[0], env, ctx)
        k = self.ev(n.args[1], env, ctx)
        h = self.hole_args([p, k])
        if h:
            return h
        p = self.period_arg(p, n.name)
        k = self.int_of(k)
        if k < 1:
            # "a step or a count that isn't positive" (L §8.4 What isn't defined is refused)
            raise refuse('RUN-DOMAIN', '%s with n = %d below 1' % (n.name, k))

        def count():
            cur, step = p.end - ONE_DAY, -1
            left = k
            while p.contains(cur):
                b = self.business(n.args[2], env, ctx, cur)
                if isinstance(b, Hole):
                    return b
                if b:
                    left -= 1
                    if left == 0:
                        return cur
                cur = cur + datetime.timedelta(days=step)
            # an argument outside the domain its row states (L §8.4 What isn't defined is refused)
            raise refuse('RUN-DOMAIN', '%s(%s, %d): the period has fewer business days' % (n.name, fmt(p), k))
        return self.over_added(n, count)

    def c_schedule(self, n, env, ctx):
        p = self.ev(n.args[0], env, ctx)
        e = self.ev(n.args[1], env, ctx)
        roll = self.ev(n.args[2], env, ctx)
        h = self.hole_args([p, e, roll])
        if h:
            return h
        p = self.period_arg(p, 'schedule')
        # L §4.2 schedule (round 19): p is a month and e an int day of the month (a year's form went with A-49)
        k = self.int_of(e)
        y, mo = p.start.year, p.start.month
        if k < 1 or k > last_day(y, mo):
            return NONE
        day = datetime.date(y, mo, k)
        if not p.contains(day):
            return NONE
        w = roll.word
        if w == 'unadjusted':
            return day
        if w not in ('following', 'preceding', 'modified_following'):
            raise refuse('LOAD-UNIT', 'schedule: %s is not a roll' % w)
        return self.over_added(n, lambda: self.rolled(n, env, ctx, day, w))

    def rolled(self, n, env, ctx, day, w):
        """schedule's day moved by its roll (L §4.2, ISDA 2006 section 4.12)."""
        b = self.business(n.args[3], env, ctx, day)
        if isinstance(b, Hole):
            return b
        if b:
            return day

        def walk(start, step):
            cur = start
            while True:
                cur = cur + datetime.timedelta(days=step)
                bb = self.business(n.args[3], env, ctx, cur)
                if isinstance(bb, Hole):
                    return bb
                if bb:
                    return cur
        if w == 'following':
            return walk(day, 1)
        if w == 'preceding':
            return walk(day, -1)
        if w == 'modified_following':
            f = walk(day, 1)
            if isinstance(f, Hole):
                return f
            if f.month != day.month:
                return walk(day, -1)
            return f
        raise refuse('LOAD-UNIT', 'schedule: %s is not a roll' % w)

    # -- tracing for explain ---------------------------------------------------------------------
    def trace_row(self, r, info, keys):
        if not self.trace_stack:
            return
        tn = TraceNode('row', r.sid)
        src = self.m.sources.get(r.version.cite)
        bits = ['version %s' % r.version.id, 'source %s' % r.version.cite]
        if src is not None:
            bits.append('"%s"' % src.title)
        if r.locator:
            bits.append('locator "%s"' % r.locator)
        if r.instant is not None:
            bits.append('at %s%s' % (r.instant[0], (' ' + r.instant[1]) if r.instant[1] else ''))
        else:
            sp = r.span(self.ask_K)
            bits.append('governs %s' % (sp.text() if sp is not None else '?'))
        kn = r.known
        bits.append('known %s' % (('window(%s, %s)' % kn) if isinstance(kn, tuple) else kn))
        tn.lines.append(', '.join(bits))
        val = r.val
        tn.value = Hole([(val[1], self.origin(info, keys))]) if isinstance(val, tuple) and val[0] == 'hole' else val
        self.trace_stack[-1].children.append(('node', tn))



def range_is_literal(d):
    """Whether a stepped range's ends and step are literals (L §2.5, round 14: then load sees its values)."""
    def lit(n):
        if n is None:
            return True
        if n.k == 'neg':
            return lit(n.e)
        if n.k == 'paren':
            return lit(n.e)
        return n.k in ('num', 'date', 'month', 'quarter', 'year', 'dur', 'duration') or (
            n.k == 'num' and not getattr(n, 'unit', None))
    return lit(d.a) and lit(d.b) and lit(getattr(d, 'step', None))

def window_id(v):
    """A window's identity under windows each: its row (or its written text), its label and its days."""
    if not isinstance(v.lo, datetime.date):
        return None
    if v.days:
        days = sorted(v.days)
    else:
        days = [v.lo + datetime.timedelta(days=i) for i in range((v.hi - v.lo).days + 1)]
    if v.origin:
        return ('row',) + tuple(str(x) for x in v.origin), str(v.origin[1]), days
    return ('src', v.src), str(v.src), days


def _tfmt(v):
    if isinstance(v, TallyError):
        return 'refused: %s %s' % (v.code, v.msg)
    if isinstance(v, Hole):
        return fmt_hole(v)
    return fmt(v)


def vkey_row(r):
    from .values import vkey
    v = r.val
    if isinstance(v, tuple):
        return repr(v)
    return vkey(v)


def fmt_rowval(r):
    v = r.val
    if isinstance(v, tuple):
        return '?'
    return fmt(v)


def thing_owners(m, t):
    """The members whose things t is (L §6.1): the owners of the versions that enter it, of its name, and of the places
    joined into it; None for a thing every member has (a currency, an unowned prelude thing)."""
    from .joint import owner_deep
    J = getattr(m, 'joint', None)
    cache = m.__dict__.setdefault('_thing_owners', {})
    if t.name in cache:
        return cache[t.name]
    owners = set(owner_deep(e.version.id) for e in t.entries if getattr(e, 'version', None) is not None)
    if SEP_CHAR in t.name:
        owners.add(owner_deep(t.name))
    for (o, n, out) in (J.groups.get(t.name, ()) if J is not None else ()):
        owners.add(o)
    if not owners or owners == {''}:
        owners = None if not owners else owners
    cache[t.name] = owners
    return owners


SEP_CHAR = '\u00b7'


def member_owns(m, b, t):
    """Whether a binding b over a kind holds thing t: a binding inside a member ranges over that member's things only
    (L §6.1, OQ-5)."""
    ow = getattr(b, '_owner', None)
    if ow is None:
        return True
    from .joint import owns
    owners = thing_owners(m, t)
    return owners is None or any(owns(ow, o) for o in owners)


def move_owner(x):
    """The member whose movement x is ('' for the joint model's own; the innermost one of a nested member)."""
    from .joint import owner_deep as owner_of
    mv = getattr(x, 'move', None)
    if mv is not None and getattr(mv, 'id', None):
        return owner_of(mv.id)
    row = getattr(x, 'row', None)
    if row is not None and getattr(row, 'inf', None) is not None:
        return owner_of(row.inf.id)
    inst = getattr(x, 'inst', None)
    if inst is not None:
        return move_owner(inst)
    xid = getattr(x, 'id', '') or ''
    if xid.startswith('pads:'):
        return owner_of(xid[5:].split(':', 1)[0])
    return owner_of(xid)


def member_moves(m, b, x):
    """Whether a binding b over an event kind holds movement x: inside a member, that member's movements only (L
    §6.1, OQ-5: a member's items mean what they mean alone)."""
    ow = getattr(b, '_owner', None)
    if ow is None:
        return True
    from .joint import owns
    return owns(ow, move_owner(x))


class HoleSignal(Exception):
    """Raised inside a read to return a hole at once (an imprecise visibility)."""

    def __init__(self, hole):
        Exception.__init__(self)
        self.hole = hole


class OutKey:
    """A key arithmetic result outside its index: RUN-KEY where it stands in a key position."""

    def __init__(self, index, text, value):
        self.index = index
        self.text = text
        self.value = value

    def key(self):
        return ('outkey', self.index, self.text)


def same_value(a, b):
    """Equal as values: two numbers of one quantity are equal whatever their int or money mark."""
    if isinstance(a, Num) and isinstance(b, Num):
        f = U.conv(a.u, b.u)
        return f is not None and a.q * f == b.q and a.rate == b.rate
    return vkey(a) == vkey(b)


class ListKey:
    """A key of a key-list binding whose index no key position fixed."""

    def __init__(self, text, node):
        self.text = text
        self.node = node
        self.name = text

    def key(self):
        return ('listkey', self.text)


class IRef:
    def __init__(self, kind, id):
        self.kind = kind
        self.id = id

    def key(self):
        return ('iref', self.kind, self.id)


class BandRowRef:
    def __init__(self, table, row, origin):
        self.table = table
        self.row = row
        self.origin = origin
        self.version = table.version
        self.sid = '%s:%s %s' % (table.version.id, origin, interval_text(row))
        self.locator = row.locator


def interval_text(r):
    def e(x, inf):
        if x is None:
            return inf
        if isinstance(x, Fraction):
            return fmt_q(x)
        if hasattr(x, 'orig'):
            return key_text(x.orig) if not isinstance(x.orig, Period) else fmt(x.orig)
        return str(x)
    return '%s%s, %s%s' % ('[' if r.lo_closed else '(', e(r.lo, '-inf'), e(r.hi, 'inf'), ']' if r.hi_closed else ')')


def band_x(x, m=None):
    if isinstance(x, Num):
        return x.q
    if isinstance(x, datetime.date):
        return x
    if isinstance(x, (Period, Key)):
        from .loader import order_key
        return order_key(x, m)
    raise refuse('LOAD-UNIT', 'a band argument %s' % fmt(x))


def order_lt(a, b):
    if isinstance(a, Num) and isinstance(b, Num):
        return a.q < b.q
    if isinstance(a, datetime.date) and isinstance(b, datetime.date):
        return a < b
    if isinstance(a, Period) and isinstance(b, Period):
        return a.start < b.start
    return False


def set_field(rec, fields, value):
    f = fields[0]
    out = []
    for a, b in rec.fields:
        if a == f:
            out.append((a, value if len(fields) == 1 else set_field(b, fields[1:], value)))
        else:
            out.append((a, b))
    return Rec(out)


def merge(*vals):
    # an operation that met a hole: under partial, a plain hole (L §5.18 Partial answers)
    qs = set()
    for v in vals:
        if isinstance(v, Hole):
            qs |= v.qs
    return Hole(qs, False)


DIGIT_CAP = 4096
DIGIT_LIMIT = 10 ** DIGIT_CAP     # a value with more than 4 096 decimal digits is at least this


def check_digits(q):
    """RUN-DIGITS: an exact value capped at 4 096 decimal digits (L §7.1)."""
    if abs(q.numerator) >= DIGIT_LIMIT or q.denominator >= DIGIT_LIMIT:
        raise refuse('RUN-DIGITS', 'a value passes 4 096 digits; round where the law rounds')


def coerce_to(v, ty):
    """A pin's value takes its target's type (money, int, num)."""
    if not isinstance(v, Num) or ty is None:
        return v
    if ty.k == 'money' and not v.money and v.enc is None:
        if v.isint or v.q.denominator == 1 or (v.q / CURRENCIES[ty.cur]).denominator == 1:
            return Money(v.q, ty.cur)
    if ty.k == 'num' and (v.money or v.isint):
        return Num(v.q, ty.u if not v.u else v.u, enc=v.enc)
    return v


def coerce_flex(v, t, vals, n):
    if t.k != 'flex' or not isinstance(v, Num):
        return v
    for o in vals:
        if isinstance(o, Num) and o is not v and (o.u or o.money):
            return Num(v.q, o.u, money=o.money, rate=o.rate, enc=v.enc)
    return v


def round_mode(x, mode):
    """The integer multiple the mode chooses (L §7.2)."""
    import math
    fl = math.floor(x)
    ce = math.ceil(x)
    if fl == ce:
        return Fraction(fl)
    if mode == 'down':
        return Fraction(fl if x > 0 else ce)
    if mode == 'up':
        return Fraction(ce if x > 0 else fl)
    if mode == 'floor':
        return Fraction(fl)
    if mode == 'ceil':
        return Fraction(ce)
    frac = x - fl
    half = Fraction(1, 2)
    if frac < half:
        return Fraction(fl)
    if frac > half:
        return Fraction(ce)
    if mode == 'half-up':
        return Fraction(ce if x > 0 else fl)
    if mode == 'half-down':
        return Fraction(fl if x > 0 else ce)
    if mode == 'half-even':
        return Fraction(fl if fl % 2 == 0 else ce)
    raise refuse('LOAD-UNIT', '%s is not a rounding mode' % mode)


def keyless_zero(x):
    """A 0 in a unit kind K with no key, as a fill's taken of a pair it never matched is: its unit names K itself."""
    return x.q == 0 and any(n in UNIT_KIND_KEYS.values() for n, e in (x.u or ()))


def adopt_key(z, other):
    """L §3.2: "A 0 with no key (a bare 0, a sum over no keys, a fill's taken of a pair it never matched) takes the key
    of what it meets": each unit kind K in z's unit becomes the key of K in other's unit."""
    u = []
    for n, e in z.u:
        if n in UNIT_KIND_KEYS.values():
            ks = [k for k, f in (other.u or ()) if UNIT_KIND_KEYS.get(k) == n]
            if len(ks) == 1:
                n = ks[0]
        u.append((n, e))
    return Num(0, tuple(u), money=z.money, isint=z.isint, rate=z.rate)


def cancel_kind(u):
    """A product or quotient of a keyless 0 (its unit names a unit kind K) and a quantity in a key k of K: the 0 takes
    k, so K^e and k^-e cancel (L §3.2)."""
    u = list(u)
    for i, (n, e) in enumerate(u):
        if n in UNIT_KIND_KEYS.values():
            for j, (k, f) in enumerate(u):
                if UNIT_KIND_KEYS.get(k) == n and f == -e:
                    return cancel_kind(tuple(x for t, x in enumerate(u) if t not in (i, j)))
    return tuple(u)


def arith(op, a, b, rt, lt, rtt):
    """Exact arithmetic on numbers (L §3.2, §7.1); rt is the static result type."""
    if not isinstance(a, Num) or not isinstance(b, Num):
        raise refuse('LOAD-UNIT', '%s %s %s' % (fmt(a), op, fmt(b)))
    if op in '+-':
        if keyless_zero(a) and not keyless_zero(b):
            a = adopt_key(a, b)
        elif keyless_zero(b) and not keyless_zero(a):
            b = adopt_key(b, a)
    if lt is not None and lt.k == 'flex':
        a = Num(a.q, b.u, money=b.money, isint=b.isint, rate=b.rate) if op in '+-' else Num(a.q, (), isint=True)
    if rtt is not None and rtt.k == 'flex':
        b = Num(b.q, a.u, money=a.money, isint=a.isint, rate=a.rate) if op in '+-' else Num(b.q, (), isint=True)
    if op in '+-':
        if a.rate or b.rate:
            if a.u != b.u:
                raise refuse('LOAD-UNIT', 'rates of different periods: %s %s %s' % (fmt(a), op, fmt(b)))
        f = U.conv(b.u, a.u) if a.u != b.u else Fraction(1)
        if f is None:
            if not a.u and a.q == 0:
                a, f = Num(0, b.u), Fraction(1)
            elif not b.u and b.q == 0:
                f = Fraction(1)
            else:
                kc = key_clash(a, b)
                if kc is not None:
                    raise kc
                raise refuse('LOAD-UNIT', '%s and %s have different units' % (fmt(a), fmt(b)))
        q = a.q + b.q * f if op == '+' else a.q - b.q * f
        u = a.u
    elif op == '*':
        u, f = U.mul(a.u, b.u)
        q = a.q * b.q * f
        if keyless_zero(a) or keyless_zero(b):
            u = cancel_kind(u)
    else:
        if b.q == 0:
            raise refuse('RUN-DIV', 'division by zero')
        u, f = U.div(a.u, b.u)
        q = a.q / b.q * f
        if keyless_zero(a):
            u = cancel_kind(u)
    check_digits(q)
    e = enc_arith(op, a, b, f) if (a.enc is not None or b.enc is not None) else None
    k = rt.k if rt is not None else 'num'
    if k == 'money':
        return Num(q, u if u else ((rt.cur, 1),), money=True, enc=e)
    if k == 'int':
        return Num(q, (), isint=True, enc=e)
    if k == 'rate':
        return Num(q, a.u if a.rate else b.u, rate=True, enc=e)
    if k == 'dyn' and (a.money or b.money) and len(u) == 1 and u[0][1] == 1 and u[0][0] in CURRENCIES:
        # a quantity of a unit known only at run (a leg of unit any): money stays money (L §3.2)
        return Num(q, u, money=True, enc=e)
    return Num(q, u, enc=e)


# -- the error bound (L §7.3): interval arithmetic over the enclosures of rounded values ------------

def enc_of(x, u=None):
    """The enclosure of a Num (a point when it rests on no rounded transcendental), in unit u."""
    e = x.enc if x.enc is not None else (x.q, x.q)
    if u is not None and u != x.u and e[0] is not None:
        f = U.conv(x.u, u)
        if f is not None:
            e = (e[0] * f, e[1] * f)
    return e


def enc_neg(e):
    if e is None:
        return None
    if e[0] is None:
        return e
    return (-e[1], -e[0])


def enc_abs(e):
    if e is None or e[0] is None:
        return e
    lo, hi = e
    if lo >= 0:
        return e
    if hi <= 0:
        return (-hi, -lo)
    return (Fraction(0), max(-lo, hi))


def enc_pow(e, k):
    if e is None or e[0] is None:
        return e
    lo, hi = e
    if k < 0 and lo <= 0 <= hi:
        return (None, None)
    c = [lo ** k, hi ** k]
    if k % 2 == 0 and lo <= 0 <= hi:
        c.append(Fraction(0))
    return (min(c), max(c))


def enc_arith(op, a, b, f):
    ea = a.enc if a.enc is not None else (a.q, a.q)
    eb = b.enc if b.enc is not None else (b.q, b.q)
    if ea[0] is None or eb[0] is None:
        return (None, None)
    if op == '+':
        return (ea[0] + eb[0] * f, ea[1] + eb[1] * f)
    if op == '-':
        return (ea[0] - eb[1] * f, ea[1] - eb[0] * f)
    if op == '*':
        ps = [x * y * f for x in ea for y in eb]
        return (min(ps), max(ps))
    if eb[0] <= 0 <= eb[1]:
        return (None, None)
    ps = [x / y * f for x in ea for y in eb]
    return (min(ps), max(ps))


# -- partial answers (L §5.18 Partial answers) --------------------------------------------------

# the operations a partial answer can't see through (L §5.18 Partial answers): a rounding, min, max, a band, a comparison
_PLAIN_CALLS = frozenset(('round', 'min', 'max', 'abs', 'pow', 'marginal', 'band_lo'))
_PLAIN_AGGS = frozenset(('min', 'max', 'argmin', 'argmax', 'count', 'all', 'any'))


def plain_node(n):
    k = n.k
    if k in ('cmp', 'not', 'and', 'or', 'in'):
        return True
    if k == 'call':
        r = getattr(n, '_r', None)
        if r is not None and r[0] in ('band', 'memberband'):
            return True
        return getattr(n, 'name', None) in _PLAIN_CALLS
    if k == 'agg':
        return n.fn in _PLAIN_AGGS
    return False


def lin_of(v):
    if isinstance(v, Hole):
        if v.lin is False:
            # a plain hole (L §5.18)
            return None
        if v.lin is not None:
            return v.lin
        if len(v.qs) == 1:
            atom = list(v.qs)[0]
            return (None, ((atom, Num(1, (), isint=True)),))
        return None
    return (v, ())


def lin_add(a, b, sign=1):
    ka, ta = a
    kb, tb = b
    if ka is None:
        k = kb if sign == 1 or kb is None else Num(-kb.q, kb.u, money=kb.money, isint=kb.isint)
    elif kb is None:
        k = ka
    else:
        k = arith('+' if sign == 1 else '-', ka, kb, None, None, None)
        if ka.money and kb.money:
            k = Num(k.q, k.u, money=True)
    terms = dict(ta)
    for atom, c in tb:
        c2 = c if sign == 1 else Num(-c.q, c.u, isint=c.isint)
        if atom in terms:
            terms[atom] = Num(terms[atom].q + c2.q, terms[atom].u, isint=terms[atom].isint and c2.isint)
        else:
            terms[atom] = c2
    return (k, tuple(sorted(terms.items(), key=lambda x: x[0])))


def lin_scale(l, s, div=False):
    k, ts = l
    def sc(x):
        if div:
            if s.q == 0:
                raise refuse('RUN-DIV', 'division by zero')
            return arith('/', x, s, None, None, None)
        return arith('*', x, s, None, None, None)
    k2 = None if k is None else sc(k)
    if k is not None and k.money and s.isint and not div:
        k2 = Num(k2.q, k.u, money=True)
    return (k2, tuple((atom, sc(c)) for atom, c in ts))


def lin_bin(op, a, b, n):
    qs = set()
    for v in (a, b):
        if isinstance(v, Hole):
            qs |= v.qs
    la, lb = lin_of(a), lin_of(b)
    if la is None or lb is None:
        return Hole(qs, False)
    if op in '+-':
        return Hole(qs, lin_add(la, lb, 1 if op == '+' else -1))
    if op == '*':
        if isinstance(a, Hole) and isinstance(b, Hole):
            return Hole(qs, False)
        known, l = (b, la) if isinstance(a, Hole) else (a, lb)
        if not isinstance(known, Num):
            return Hole(qs, False)
        return Hole(qs, lin_scale(l, known))
    if op == '/':
        if isinstance(b, Hole):
            return Hole(qs, False)
        if not isinstance(b, Num):
            return Hole(qs, False)
        return Hole(qs, lin_scale(la, b, div=True))
    return Hole(qs, False)


def _ntxt(n):
    from .check import _txt
    try:
        return _txt(n)
    except Exception:
        return n.k


from . import eval2 as _eval2  # noqa: E402
_eval2.install(Engine)
from . import moves as _moves  # noqa: E402
from . import walk as _walk  # noqa: E402
from . import mreads as _mreads  # noqa: E402
_moves.install(Engine)
_walk.install(Engine)
_mreads.install(Engine)
from . import resid as _resid  # noqa: E402
_resid.install(Engine)
from . import partswalk as _partswalk  # noqa: E402
_partswalk.install(_walk.Walk, Engine)
