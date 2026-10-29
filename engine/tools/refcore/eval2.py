"""Writer 2's evaluation, installed into evaluate.Engine.

`to u` and the transcendentals with their error bound (L §7.3), the rate built-ins (L §3.3),
distributions and random facts (L §5.16: samples, paths, closed forms, sample pins), the
statistics over paths, `crps` (L §5.15).  Each `m_<name>` function becomes the method `<name>`.
"""
import datetime
from fractions import Fraction

from . import units as U
from . import reals as R
from . import dists as D
from .errors import TallyError, Refusal, LoadError, Unspecified, NotYet, load_now, RandomLine
from .values import Num, Money, Hole, Key, Code, Special, NONE, NEVER, fmt, fmt_q, CURRENCIES, Mv, Thing
from .dates import Period, last_day


def install(E):
    for name, fn in list(globals().items()):
        if name.startswith('m_'):
            setattr(E, name[2:], fn)
    E.xsub = None
    E.ask_paths = None
    E.path_cache = None
    E.each_met = None


def _ev():
    from . import evaluate
    return evaluate


def refuse(code, msg):
    return Refusal(code, msg)


def strip(n):
    while n.k == 'paren':
        n = n.e
    return n


PERIOD_MONTHS = {'year': 12, 'quarter': 3, 'month': 1}


# -- reals -----------------------------------------------------------------------------------------

def real_bounds(x):
    """Rigorous rational bounds of a real."""
    if x.ex is not None:
        return x.ex, x.ex
    W = 96
    while W <= R.CAP:
        try:
            return x.bounds(W)
        except R.Undecided:
            W *= 2
    raise refuse('RUN-UNDECIDED', '%s not decided within the precision cap' % x.name)


def real_affine(x, a, b):
    """a * x + b for a real x and rationals a, b."""
    a = Fraction(a)
    b = Fraction(b)
    ex = x.ex * a + b if x.ex is not None else None
    G = 8 + max(abs(a.numerator).bit_length() - a.denominator.bit_length(), 0)

    def f(W):
        V = W + G
        iv = x.enclose(V)
        iv = R.imulq(iv, a)
        iv = R.iadd(iv, R.fi(b, V))
        return iv[0] >> G, -((-iv[1]) >> G)
    return R.Real(f, ex, x.name)


def widened(v, x, step):
    """The enclosure of a value correctly rounded to step: [v - u/2, v + u/2], or none when the exact
    value is a rational on the cell edge, detected exactly (L §7.3)."""
    if x is not None and x.ex is not None and x.ex == v:
        return None
    return (v - step / 2, v + step / 2)


# -- expression substitution (the coefficients of an affine form) ------------------------------------

def m_ev_sub(self, node, env, ctx, subs):
    old = self.xsub
    d = dict(old) if old else {}
    d.update(subs)
    self.xsub = d
    try:
        return self.ev(node, env, ctx)
    finally:
        self.xsub = old


# -- to u --------------------------------------------------------------------------------------------

def m_step_value(self, unode, env, ctx, vu):
    """The step of `to u` in the value's unit vu (L §7.1 precision steps)."""
    if unode.k == 'num' and getattr(unode, '_val', None) is not None:
        return Fraction(unode._val)
    v = self.ev(unode, env, ctx)
    if isinstance(v, Hole):
        return v
    if not isinstance(v, Num):
        raise refuse('LOAD-UNIT', 'the step of to is %s' % fmt(v))
    if v.q <= 0:
        raise refuse('RUN-DOMAIN', 'to %s: a step that is not positive' % fmt(v))
    if not v.u:
        return v.q
    f = U.conv(v.u, vu)
    if f is None:
        raise refuse('LOAD-UNIT', 'to %s: the step and the value (%s) have different units' % (fmt(v), U.fmt(vu)))
    return v.q * f


def m_e_to(self, n, env, ctx):
    inner = strip(n.e)
    form = n._form
    if form == 'dist':
        d = self.ev(inner, env, ctx)
        if isinstance(d, Hole):
            return d
        step = self.step_value(n.unit, env, ctx, d.unit)
        if isinstance(step, Hole):
            return step
        return d.with_step(step)
    if form == 'call':
        return self.trans_call(inner, self.call_args(inner, env, ctx), n, ctx, env)
    if form == 'distfn':
        return self.e_distfn(inner, env, ctx, to=n)
    if form == 'query':
        return self.e_query_random(inner, env, ctx, to=n)
    if form == 'crps':
        return self.e_crps(inner, env, ctx, to=n)
    raise NotYet('to u on %s' % form, n.loc)


def m_round_real(self, x, to, env, ctx, uo, what, encd_hull=None):
    """A real correctly rounded to the step of `to` (L §7.3), with its enclosure.  encd_hull: the hull of
    the exact values over the arguments' enclosures, when an argument is an interval."""
    if to is None:
        if x.ex is None:
            raise refuse('LOAD-TO', '%s needs to u' % what)
        return Num(x.ex, uo, enc=encd_hull)
    step = self.step_value(to.unit, env, ctx, uo)
    if isinstance(step, Hole):
        return step
    v, _ = R.round_to(x, step, what)
    if encd_hull is not None:
        lo, hi = encd_hull
        enc = (None, None) if lo is None else (lo - step / 2, hi + step / 2)
    else:
        enc = widened(v, x, step)
    return Num(v, uo, enc=enc)


def m_trans_call(self, n, vals, to, ctx, env=None):
    """A transcendental built-in: exp, ln, sqrt, pow, periodic, factor, accrue (L §7.3, §3.3)."""
    name = n.name
    h = self.hole_args(vals)
    if h:
        return h
    for v in vals:
        self.no_none(v, name)
    idx = [i for i, v in enumerate(vals) if isinstance(v, Num)]

    def at(qs):
        args = list(vals)
        for i, q in qs.items():
            v = vals[i]
            args[i] = Num(q, v.u, money=v.money, isint=v.isint, rate=v.rate)
        return self.real_of_call(n, name, args)
    x, uo = at({i: vals[i].q for i in idx})
    encd = [i for i in idx if vals[i].enc is not None]
    hull = None
    if encd:
        hull = self.corner_hull(at, vals, encd)
    return self.round_real(x, to, env, ctx, uo, name, hull)


def m_corner_hull(self, at, vals, encd):
    """The hull of a function's exact values over a box of arguments, from its corners: the built-ins of
    L §7.3 are monotone in each argument on their domains."""
    if any(vals[i].enc[0] is None for i in encd):
        return (None, None)
    corners = [{}]
    for i in encd:
        lo, hi = vals[i].enc
        corners = [dict(c, **{str(i): lo}) for c in corners] + [dict(c, **{str(i): hi}) for c in corners]
    los, his = [], []
    for c in corners:
        qs = {i: vals[i].q for i, v in enumerate(vals) if isinstance(v, Num)}
        for k, q in c.items():
            qs[int(k)] = q
        try:
            x, _ = at(qs)
            a, b = real_bounds(x)
        except Refusal:
            return (None, None)
        los.append(a)
        his.append(b)
    return (min(los), max(his))


def m_real_of_call(self, n, name, args):
    if name == 'exp':
        return R.real_exp(args[0].q), ()
    if name == 'ln':
        return R.real_ln(args[0].q), ()
    if name == 'sqrt':
        x = args[0]
        return R.real_sqrt(x.q), U.make([(b, e // 2) for b, e in x.u])
    if name == 'pow':
        x, k = args
        if k.q.denominator == 1 and x.u:
            return R.real_pow(x.q, k.q), U.power(x.u, int(k.q))
        return R.real_pow(x.q, k.q), ()
    if name == 'periodic':
        r, _, c = args
        return self.periodic_real(r, n.args[1]._period, c, None), ()
    if name == 'factor':
        r, k, _, c = args
        return self.periodic_real(r, n.args[2]._period, c, k), ()
    if name == 'accrue':
        return self.accrue_real(*args), (args[0].u if not args[0].isint else ())
    raise NotYet(name, n.loc)


# -- rates (L §3.3) ------------------------------------------------------------------------------------

def m_rate_parts(self, r, P, c, what):
    """periodic(r, P, c) as ('lin', value) or ('pow', base, exponent): the rate per period P is value,
    or base^exponent - 1."""
    if not isinstance(r, Num) or not r.rate:
        raise refuse('LOAD-UNIT', '%s: %s is not a rate' % (what, fmt(r)))
    per = U.period_of(r.u)
    if per is None:
        raise refuse('LOAD-UNIT', '%s: %s states no period' % (what, fmt(r)))
    if P == 'day' or per == 'day':
        if not (P == 'day' and per == 'day'):
            # "LOAD-UNIT where load sees the rate's period, RUN-DOMAIN otherwise" (L §3.3, round 6)
            raise refuse('RUN-DOMAIN', '%s: a %s rate over a %s: a day\'s share of a longer period is a day count '
                         '(dcf), never a default (L §3.3)' % (what, per, P))
        t = Fraction(1)
    else:
        t = Fraction(PERIOD_MONTHS[P], PERIOD_MONTHS[per])
    if not isinstance(c, Code) or c.kind != 'compounding':
        raise refuse('LOAD-UNIT', '%s: %s is not a compounding code' % (what, fmt(c)))
    w = c.word
    rq = r.q
    if w == 'simple':
        return ('lin', rq * t)
    if w == 'effective':
        return ('pow', 1 + rq, t)
    if w == 'discount':
        if rq == 1:
            raise refuse('RUN-DIV', 'division by zero: a rate of discount of 1')
        return ('pow', 1 + rq / (1 - rq), t)
    if w == 'nominal':
        m = Fraction(c.n)
        return ('pow', 1 + rq / m, m * t)
    raise refuse('LOAD-UNIT', '%s: %s is not a compounding code' % (what, w))


def m_periodic_real(self, r, P, c, k):
    """periodic(r, P, c), or factor(r, k, P, c) = (1 + periodic)^k when k is given."""
    what = 'periodic' if k is None else 'factor'
    parts = self.rate_parts(r, P, c, what)
    if k is not None:
        if not isinstance(k, Num) or k.u:
            raise refuse('LOAD-UNIT', 'factor: n is %s' % fmt(k))
        if parts[0] == 'lin':
            return R.real_pow(1 + parts[1], k.q)
        return R.real_pow(parts[1], parts[2] * k.q)
    if parts[0] == 'lin':
        return R.const(parts[1])
    return real_affine(R.real_pow(parts[1], parts[2]), 1, -1)


def m_accrue_real(self, p, r, a, b, dc, c):
    if not isinstance(r, Num) or not r.rate:
        raise refuse('LOAD-UNIT', 'accrue: %s is not a rate' % fmt(r))
    if U.period_of(r.u) != 'year':
        raise refuse('LOAD-UNIT', 'accrue takes a yearly rate, not %s (L §3.3)' % fmt(r))
    y = self.dcf_q(a, b, dc)
    if not isinstance(c, Code) or c.kind != 'compounding':
        raise refuse('LOAD-UNIT', 'accrue: %s is not a compounding code' % fmt(c))
    w = c.word
    rq = r.q
    pq = p.q
    if w == 'simple':
        return R.const(pq * rq * y)
    if w == 'effective':
        return real_affine(R.real_pow(1 + rq, y), pq, -pq)
    if w == 'nominal':
        m = Fraction(c.n)
        return real_affine(R.real_pow(1 + rq / m, m * y), pq, -pq)
    # accrue's row gives simple, effective and nominal m (L §3.3); discount is outside it (L §8.4)
    raise refuse('RUN-DOMAIN', 'accrue under %s: its row states simple, effective and nominal m' % w)


def m_dcf_q(self, a, b, dc):
    """The year fraction from a to b under the day count (L §3.3)."""
    a = self.as_day(a, 'dcf')
    b = self.as_day(b, 'dcf')
    if not isinstance(dc, Code) or dc.kind != 'daycount':
        raise refuse('LOAD-UNIT', 'dcf: %s is not a daycount code' % fmt(dc))
    w = dc.word
    if b < a:
        # L §3.3: "b before a gives the negative of dcf(b, a)"
        return -self.dcf_q(b, a, dc)
    if w == 'act365':
        # L §3.3: "With n the days from a to b, a counted and b not, act365 gives n/365"; the review removed the
        # other day counts ("A text that needs another day count writes a def over the two dates")
        return Fraction((b - a).days, 365)
    raise refuse('LOAD-UNIT', 'dcf: %s is not a daycount code' % w)


def m_as_day(self, v, what):
    from .values import Iv
    if isinstance(v, Iv):
        raise self.imprecise(v, what)
    if isinstance(v, Period) and v.kind == 'day':
        return v.start
    if isinstance(v, datetime.date):
        return v
    if v is NONE:
        raise refuse('RUN-NONE', 'a value is none here: %s' % what)
    raise refuse('LOAD-UNIT', '%s: %s is not a date' % (what, fmt(v)))


def m_c_dcf(self, n, env, ctx):
    vals = self.args(n, env, ctx)
    h = self.hole_args(vals)
    if h:
        return h
    return Num(self.dcf_q(*vals), ())


def m_call_args(self, n, env, ctx):
    """A call's arguments; a period word (periodic, factor) stands for itself."""
    return [a._period if getattr(a, '_period', None) else self.ev(a, env, ctx) for a in n.args]


def m_c_exp(self, n, env, ctx):
    return self.trans_call(n, self.call_args(n, env, ctx), None, ctx, env)


m_c_ln = m_c_exp
m_c_sqrt = m_c_exp
m_c_periodic = m_c_exp
m_c_factor = m_c_exp
m_c_accrue = m_c_exp


# -- distributions (L §5.16) -------------------------------------------------------------------------

def m_e_dist(self, n, env, ctx):
    fn = n.fn
    unit = n._unit
    if fn == 'discrete':
        mem = list(self.members([n.bind], env, ctx))
        items = []
        holes = []
        for e2, st in mem:
            if isinstance(st, tuple) and st[0] == 'domainhole':
                return st[1]
            if isinstance(st, Hole):
                holes.append(st)
            t = self.ev(n.term, e2, ctx)
            if isinstance(t, Hole):
                holes.append(t)
                continue
            self.no_none(t, fn)
            if not isinstance(t, Num):
                raise refuse('LOAD-UNIT', 'discrete: the weight %s is not a number' % fmt(t))
            items.append((e2[n.bind.var], t.q))
        if holes:
            return _ev().merge(*holes)
        if any(w < 0 for k, w in items) or sum(w for k, w in items) <= 0:
            raise refuse('RUN-PARAM', 'discrete parameter w = [%s] is invalid' % ', '.join(
                fmt_q(w) for k, w in items))
        return D.Dist('discrete', (), (), None, items)
    vals = [self.ev(a, env, ctx) for a in n.args]
    h = self.hole_args(vals)
    for v in vals:
        if isinstance(v, Hole):
            continue
        self.no_none(v, fn)
        if not isinstance(v, Num):
            raise refuse('LOAD-UNIT', '%s: %s is not a number' % (fn, fmt(v)))

    def inu(v, u):
        if not v.u:
            return v.q
        f = U.conv(v.u, u)
        if f is None:
            raise refuse('LOAD-UNIT', '%s: %s is not of unit %s' % (fn, fmt(v), U.fmt(u) or 'num'))
        return v.q * f
    if h:
        # a refusal outranks a hole (L §8.4): a parameter known invalid by itself is refused although another is
        # a hole
        units = {'normal': (unit, unit), 'lognormal': ((), ()), 'uniform': (unit, unit), 'exponential': (U.inv(unit),),
                 'gamma': ((), unit)}.get(fn)
        ps = []
        for i, v in enumerate(vals):
            if isinstance(v, Hole):
                ps.append(None)
            else:
                ps.append(inu(v, units[i] if units is not None and i < len(units) else ()))
        D.validate_known(fn, ps)
        return h
    if fn in ('normal', 'uniform'):
        ps = [inu(vals[0], unit), inu(vals[1], unit)]
    elif fn == 'exponential':
        ps = [inu(vals[0], U.inv(unit))]
    elif fn == 'gamma':
        ps = [inu(vals[0], ()), inu(vals[1], unit)]
    else:
        ps = [inu(v, ()) for v in vals]
    d = D.Dist(fn, ps, unit)
    D.validate(d)
    return d


def m_address_law(self, info, keys, ctx):
    """The law date a random fact is read at (L §5.16 Order; the horizon)."""
    if hasattr(info, 'rules'):
        at = info.at
        if at is None:
            return None
        if at.k == 'law_expr':
            env = {b.var: self.keyval(k) for b, k in zip(info.binds, keys)}
            try:
                v = self.law_value(at.e, env, ctx, info)
            except TallyError:
                return None
            return v if isinstance(v, datetime.date) else None
        return ctx.law
    if info.timeless():
        return None
    return ctx.law


def m_dist_value(self, info, keys, ctx):
    if hasattr(info, 'rules'):
        return self.read_def(info, keys, ctx)
    return self.read_input(info, keys, ctx)


def m_sample_read(self, info, keys, ctx):
    """The sample of a random fact on the path (L §5.16 Samples)."""
    from .model import _kv, key_text
    org = self.origin(info, keys)
    law = self.address_law(info, keys, ctx)
    self.reg(('hz', law))
    unit = info.type.u or ()
    upin = None
    for p in ctx.pins:
        if p.kind not in ('sample', 'sampleu') or p.item != info.id:
            continue
        if p.keys is not None and any(pk is not None and pk != _kv(k) for pk, k in zip(p.keys, keys)):
            continue
        self.reg(('pinned sample', p.text))
        if p.kind == 'sample':
            v = p.value
            if isinstance(v, Num) and v.u and unit and v.u != unit:
                f = U.conv(v.u, unit)
                v = Num(v.q * f, unit, enc=v.enc)
            elif isinstance(v, Num) and not v.u:
                v = Num(v.q, unit, enc=v.enc)
            return v
        upin = p.value.q
        break
    d = self.dist_value(info, keys, ctx)
    if isinstance(d, Hole):
        return d
    if not isinstance(d, D.Dist):
        raise refuse('LOAD-UNIT', '%s is not a distribution' % org)
    r = ctx.rnd
    if ctx.wp is not None and law is not None and not (r is not None and r[0] == 'proj'):
        # on a path a decision reads a sample only at a law date on or before its own day (L §5.16)
        dday = datetime.date.fromordinal(ctx.wp[0])
        if law > dday:
            raise refuse('RUN-FUTURE', 'a decision on %s reads the sample of %s at %s, after its day' % (dday, org,
                                                                                                          law))
    if upin is None and r is not None and r[0] == 'proj' and law is not None and law > r[3]:
        # a projection from t reads a fact after t at its random clause's value, the same on every path (L §5.16
        # Decisions and projections); without a clause it is refused
        if r[1] is None and r[4] is not None:
            # L §5.16: a decision's projection with neither a random clause nor a query around the read "is refused:
            # at load where load sees the projection reach the random fact after the decision's day (LOAD-RANDOM),
            # otherwise at the read (RUN-FUTURE)"; L §8.4 `<decision> on <d> reads <x> at <d'> after its day`
            raise refuse('RUN-FUTURE', 'a decision on %s reads %s at %s after its day (a projection from %s with no '
                                       'random clause or query)' % (r[4], org, law, r[3]))
        if r[1] is None and ctx.path is None:
            # L §5.16: "An ask line's own projection is no decision: it reads an amount after t by its random clause,
            # by the line's query, or on a path by that path's sample"; with none of them the line is random with no
            # query (L §8.3 LOAD-RANDOM `<projection> reaches random <x> after <t> with no random clause or query`;
            # refcore's load doesn't see which movements a projection reaches, so it refuses at the read)
            raise RandomLine('a projection from %s reaches random %s at %s, after %s, with no random clause or '
                             'query' % (r[3], org, law, r[3]))
    if upin is None and r is not None and r[0] == 'proj' and law is not None and law > r[3] and r[1] is not None:
        if d.fn == 'discrete':
            if r[1] == 'mean':
                raise refuse('RUN-DOMAIN', 'random at mean of %s, a draw of keys (L §5.16)' % org)
            k, _ = D.quantile(d, r[2])
            return k
        if r[1] == 'mean':
            mr = D.mean(d)
            if mr.ex is not None:
                return Num(mr.ex, d.unit)
            if d.step is None:
                raise refuse('LOAD-TO', 'random at mean of %s needs its distribution\'s to u' % org)
            v, _ = R.round_to(mr, d.step, 'mean of %s' % d.fn)
            return Num(v, d.unit, enc=widened(v, mr, d.step))
        v, enc = D.quantile(d, r[2], step=d.step)
        return Num(v, d.unit) if not isinstance(v, Num) else v
    if upin is not None:
        u = upin
    else:
        if ctx.path is None:
            # E §3.1 step 10: "An ask line whose value is random that neither applies a query nor replays a path
            # ... [is] refused (LOAD-RANDOM)"; L §8.3 `<line> is random; apply a query`. refcore's load marks the
            # random cone through moves into the defs, fills and decisions that read the balances they book
            # (check2.random_cone), not into an ask line's own balance read, so such a line is refused at the read;
            # outside a path, a plan or decision whose day reaches a sample refuses only the places it books
            # (moves.eval_inst, walk.dec_bad; E §7 step 1).
            raise RandomLine('the line is random (it reaches the sample of %s); apply a query' % org)
        seed, p = ctx.path
        u = D.uniform_of(org, seed, p)
    v = self.sample_value(d, u)
    if upin is None:
        tx = fmt(Num(v.q, v.u)) if isinstance(v, Num) else fmt(v)
        order = tuple(self.key_ord(k, b.dom) for b, k in zip(info.binds, keys)) if keys else ()
        self.reg(('sample', (law is not None, law or datetime.date.min, info.id, repr(order)),
                  '%s = %s (u = %s)' % (org, tx, fmt_q(u))))
    return v


def m_sample_value(self, d, u):
    r, enc = D.sample(d, u)
    if d.fn == 'discrete':
        return r
    if d.fn in D.CONTINUOUS:
        x = R.const(enc[0]) if enc is not None and enc[0] == enc[1] else None
        return Num(r, d.unit, enc=widened(r, x, d.step) if x is not None else (r - d.step / 2, r + d.step / 2))
    return Num(r, d.unit)


# -- cdf, pdf, pmf ------------------------------------------------------------------------------------

def m_affine_coef(self, node, x, env, ctx, unit):
    """a and b of the affine form node = a * X + b, by evaluating it at X = 0 and X = 1 (exact)."""
    f0 = self.ev_sub(node, env, ctx, {id(x): Num(0, unit)})
    f1 = self.ev_sub(node, env, ctx, {id(x): Num(1, unit)})
    for v in (f0, f1):
        if isinstance(v, Hole):
            return v, None
    a = Num(f1.q - f0.q * (U.conv(f0.u, f1.u) or 1), f1.u)
    return a, f0


def m_prob(self, d, op, t, what):
    """P(X op t) as a real (L §5.16 share's closed form)."""
    if op == '<=':
        return D.cdf(d, t)
    cont = d.fn in D.CONTINUOUS
    if op == '<':
        if cont:
            return D.cdf(d, t)
        return self.real_sub(D.cdf(d, t), self.pmf_real(d, t))
    if op == '>':
        return real_affine(D.cdf(d, t), -1, 1)
    if op == '>=':
        return real_affine(self.prob(d, '<', t, what), -1, 1)
    if op == '==':
        return R.const(0) if cont else self.pmf_real(d, t)
    if op == '!=':
        return R.const(1) if cont else real_affine(self.pmf_real(d, t), -1, 1)
    raise refuse('LOAD-UNIT', '%s: no closed form for %s' % (what, op))


def m_pmf_real(self, d, t):
    return D.pmf(d, t)


def m_real_sub(self, x, y):
    ex = x.ex - y.ex if x.ex is not None and y.ex is not None else None

    def f(W):
        return R.isub(x.enclose(W + 2), y.enclose(W + 2))[0] >> 2, -((-R.isub(x.enclose(W + 2),
                                                                             y.enclose(W + 2))[1]) >> 2)
    return R.Real(f, ex, x.name)


def m_e_distfn(self, n, env, ctx, to=None):
    fn = n.fn
    x = n._x
    dn, yn = n.args
    d = self.ev(x, env, ctx)
    y = self.ev(yn, env, ctx)
    # both operands are evaluated: a refusal of either outranks a hole of the other (L §5.11, §8.4)
    if isinstance(d, Hole) or isinstance(y, Hole):
        return _ev().merge(*[h for h in (d, y) if isinstance(h, Hole)])
    self.no_none(y, fn)
    if not isinstance(d, D.Dist):
        raise refuse('LOAD-UNIT', '%s of %s' % (fn, fmt(d)))
    if n._keys is not None:
        order = lambda k: self.key_ord(k, ('index', n._keys))
        if fn == 'cdf':
            return Num(D.cdf_key(d, y, order), ())
        return Num(D.pmf_key(d, y), ())
    unit = d.unit
    if strip(dn) is strip(x):
        a, b = Num(1, ()), None
        yq = y.q * (U.conv(y.u, unit) if y.u and unit and y.u != unit else 1)
        t = yq
        au = Fraction(1)
    else:
        a, b = self.affine_coef(dn, x, env, ctx, unit)
        if isinstance(a, Hole):
            return a
        if a.q == 0:
            raise refuse('RUN-DOMAIN', '%s of a * X + b with a = 0 (L §5.16: a is not 0)' % fn)
        f = U.conv(y.u, b.u) if y.u != b.u and y.u and b.u else Fraction(1)
        t = (y.q * (f or 1) - b.q) / a.q
        au = abs(a.q)
    if fn == 'cdf':
        if a.q > 0:
            r = D.cdf(d, t)
        else:
            r = real_affine(self.prob(d, '<', t, fn), -1, 1)
        uo = ()
    elif fn == 'pmf':
        r = self.pmf_real(d, t)
        uo = ()
    else:
        r = D.pdf(d, t)
        if au != 1:
            r = real_affine(r, 1 / au, 0)
        yu = y.u if y.u else (b.u if b is not None else unit)
        uo = U.inv(yu or unit)
    return self.round_real(r, to, env, ctx, uo, fn)


# -- queries over random values (L §5.16) ------------------------------------------------------------

def m_e_query_random(self, n, env, ctx, to=None):
    if getattr(n, '_dfn', False):
        return self.dist_value_quantile(n, env, ctx, to)
    mode = getattr(n, '_qmode', 'closed')
    if mode == 'paths':
        st = self.path_stat(n, env, ctx, to)
        return st['value']
    return self.closed_query(n, env, ctx, to)[0]


def m_dist_value_quantile(self, n, env, ctx, to):
    """quantile(d, q) of a distribution value: type 1, the smallest x with cdf(x) >= q (L §5.16)."""
    d = self.ev(n.arg, env, ctx)
    if isinstance(d, Hole):
        return d
    q = self.ev(n.q, env, ctx)
    if isinstance(q, Hole):
        return q
    self.no_none(q, 'quantile')
    if not isinstance(d, D.Dist):
        raise refuse('LOAD-UNIT', 'quantile of %s, which is not a distribution' % fmt(d))
    if d.fn == 'discrete':
        k, _ = D.quantile(d, q.q)
        return k
    if d.fn in D.CONTINUOUS:
        if d.fn == 'uniform' and to is None:
            D._qcheck(d, q.q)
            return Num(d.params[0] + q.q * (d.params[1] - d.params[0]), d.unit)
        if to is None:
            raise refuse('LOAD-TO', 'quantile of %s needs to u' % d.fn)
        step = self.step_value(to.unit, env, ctx, d.unit)
        if isinstance(step, Hole):
            return step
        v, enc = D.quantile(d, q.q, step)
        x0 = R.const(enc[0]) if enc is not None and enc[0] == enc[1] else None
        return Num(v, d.unit, enc=widened(v, x0, step) if x0 is not None else (v - step / 2, v + step / 2))
    v, _ = D.quantile(d, q.q)
    Q = Num(v, d.unit)
    if to is not None:
        return self.round_exactish(Q, to, env, ctx, 'quantile')
    return Q


def m_fact_dist(self, x, env, ctx):
    """The distribution of a random fact read directly (a closed form's X)."""
    r = x._r
    info = self.m.inputs[r[2]] if r[1] == 'input' else self.m.defs[r[2]]
    keynodes = x.keys if x.k == 'index' else []
    keys, hole = self.key_values(info, keynodes, x._convs, env, ctx)
    if hole is not None:
        return hole, None
    self.reg(('hz', self.address_law(info, keys, ctx)))
    for p in ctx.pins:
        if p.kind in ('sample', 'sampleu') and p.item == info.id:
            # L §5.16: a query "that reaches a fact whose sample is pinned ... states paths N seed S"; closed forms
            # are "refused otherwise (LOAD-RANDOM: the ask states paths N seed S)"
            raise refuse('LOAD-RANDOM', 'a query of %s, whose sample is pinned, has no closed form: the ask states '
                                        'paths N seed S' % info.id)
    d = self.dist_value(info, keys, ctx)
    return d, info


def m_closed_query(self, n, env, ctx, to=None):
    """A query without paths, from the distributions' own functions (L §5.16 Closed forms).
    Returns (value, horizon facts)."""
    kind = n._closed[0]
    xs = n._closed[1]
    ds = []
    for x in xs:
        d, info = self.fact_dist(x, env, ctx)
        if isinstance(d, Hole):
            return d, None
        if not isinstance(d, D.Dist):
            raise refuse('LOAD-UNIT', 'a query of %s, which is not a distribution' % fmt(d))
        ds.append(d)
    arg = n.arg
    if kind == 'mean' and to is not None and any(D.mean(d).ex is None for d in ds):
        # L §5.16 Closed forms: "the same sum of their means ..., exact or correctly rounded": with the line's to u,
        # Σ a_i · mean_i + b rounded once, from the means as reals (not each mean rounded first)
        zero = {id(x): Num(0, d.unit) for x, d in zip(xs, ds)}
        b0 = self.ev_sub(arg, env, ctx, zero)
        if isinstance(b0, Hole):
            return b0, None
        terms = []
        for x, d in zip(xs, ds):
            one = dict(zero)
            one[id(x)] = Num(1, d.unit)
            f1 = self.ev_sub(arg, env, ctx, one)
            if isinstance(f1, Hole):
                return f1, None
            terms.append((f1.q - b0.q * (U.conv(b0.u, f1.u) or 1), D.mean(d)))
        bq = b0.q

        def fsum(W):
            G = 16 + len(terms).bit_length()
            V = W + G
            lo, hi = R.fi(bq, V)
            for a_, mr in terms:
                e = R.imulq(mr.enclose(V), a_)
                lo += e[0]
                hi += e[1]
            return lo >> G, -((-hi) >> G)
        ystep = self.step_value(to.unit, env, ctx, b0.u)
        if isinstance(ystep, Hole):
            return ystep, None
        real = R.Real(fsum, None, 'mean')
        v, _ = R.round_to(real, ystep, 'mean')
        return Num(v, b0.u, enc=widened(v, real, ystep)), None
    if kind == 'mean':
        subs = {}
        for x, d in zip(xs, ds):
            mr = D.mean(d)
            if mr.ex is not None:
                subs[id(x)] = Num(mr.ex, d.unit)
            else:
                if d.step is None:
                    raise refuse('LOAD-TO', 'mean of %s needs to u' % d.fn)
                v, _ = R.round_to(mr, d.step, 'mean of %s' % d.fn)
                subs[id(x)] = Num(v, d.unit, enc=widened(v, mr, d.step))
        v = self.ev_sub(arg, env, ctx, subs)
        if to is not None and isinstance(v, Num):
            return self.round_exactish(v, to, env, ctx, 'mean'), None
        return v, None
    if kind == 'quantile':
        x, d = xs[0], ds[0]
        q = self.ev(n.q, env, ctx)
        if isinstance(q, Hole):
            return q, None
        self.no_none(q, 'quantile')
        trivial = strip(arg) is x
        if trivial:
            a = Num(1, ())
        else:
            a, b = self.affine_coef(arg, x, env, ctx, d.unit)
            if isinstance(a, Hole):
                return a, None
            if a.q == 0:
                raise refuse('RUN-DOMAIN', 'quantile of a * X + b with a = 0')
        qq = q.q if a.q > 0 else 1 - q.q
        if d.fn == 'discrete':
            if not trivial:
                raise refuse('LOAD-UNIT', 'quantile of an affine form of a draw of keys')
            k, _ = D.quantile(d, qq)
            return k, None
        if d.fn in D.CONTINUOUS:
            step = d.step
            if to is not None:
                if not trivial:
                    # L §5.16 Closed forms: quantile(a * X + b, q) = a · quantile(D, q) + b (a > 0), a · quantile(D,
                    # 1 - q) + b (a < 0), "correctly rounded" to the line's to u: the cells of Y's step, each edge
                    # compared through F
                    ystep = self.step_value(to.unit, env, ctx, b.u)
                    if isinstance(ystep, Hole):
                        return ystep, None
                    v, enc = D.quantile_affine(d, q.q, ystep, a.q, b.q)
                    x0 = R.const(enc[0]) if enc is not None and enc[0] == enc[1] else None
                    return Num(v, b.u, enc=widened(v, x0, ystep) if x0 is not None else
                               (v - ystep / 2, v + ystep / 2)), None
                step = self.step_value(to.unit, env, ctx, d.unit)
            if d.fn == 'uniform' and to is None:
                D._qcheck(d, qq)
                Q = Num(d.params[0] + qq * (d.params[1] - d.params[0]), d.unit)
            else:
                if step is None:
                    raise refuse('LOAD-TO', 'quantile of %s needs to u' % d.fn)
                v, enc = D.quantile(d, qq, step)
                x0 = R.const(enc[0]) if enc is not None and enc[0] == enc[1] else None
                Q = Num(v, d.unit, enc=widened(v, x0, step) if x0 is not None else (v - step / 2, v + step / 2))
        else:
            v, _ = D.quantile(d, qq)
            Q = Num(v, d.unit)
            if to is not None and trivial:
                return self.round_exactish(Q, to, env, ctx, 'quantile'), None
        if trivial:
            return Q, None
        return self.ev_sub(arg, env, ctx, {id(x): Q}), None
    # share
    x, d = xs[0], ds[0]
    c = strip(arg)
    side = n._closed[2]
    S = c.l if side == 'l' else c.r
    other = c.r if side == 'l' else c.l
    y = self.ev(other, env, ctx)
    if isinstance(y, Hole):
        return y, None
    self.no_none(y, 'share')
    op = c.op
    flip = {'<': '>', '>': '<', '<=': '>=', '>=': '<=', '==': '==', '!=': '!='}
    if side == 'r':
        op = flip[op]
    if d.fn == 'discrete':
        if strip(S) is not x:
            raise refuse('LOAD-UNIT', 'share of an affine form of a draw of keys')
        ixid = x._t.ix
        order = lambda k: self.key_ord(k, ('index', ixid))
        pk = D.pmf_key(d, y)
        le = D.cdf_key(d, y, order)
        p = {'<=': le, '<': le - pk, '>': 1 - le, '>=': 1 - le + pk, '==': pk, '!=': 1 - pk}[op]
        return Num(p, ()), None
    if strip(S) is x:
        f = U.conv(y.u, d.unit) if y.u and d.unit and y.u != d.unit else Fraction(1)
        t = y.q * (f or 1)
    else:
        a, b = self.affine_coef(S, x, env, ctx, d.unit)
        if isinstance(a, Hole):
            return a, None
        if a.q == 0:
            raise refuse('RUN-DOMAIN', 'share of a * X + b op y with a = 0')
        f = U.conv(y.u, b.u) if y.u and b.u and y.u != b.u else Fraction(1)
        t = (y.q * (f or 1) - b.q) / a.q
        if a.q < 0:
            op = flip[op]
    r = self.prob(d, op, t, 'share')
    if r.ex is None and to is None:
        raise refuse('LOAD-TO', 'share of %s needs to u (a cdf of %s needs to)' % (d.fn, d.fn))
    return self.round_real(r, to, env, ctx, (), 'share'), None


def m_round_exactish(self, v, to, env, ctx, what):
    """`to u` on an exact value, or on one that rests on one rounded value: correctly rounded."""
    if v.enc is not None:
        raise NotYet('to u on a closed-form %s that rests on rounded values' % what)
    return self.round_real(R.const(v.q), to, env, ctx, v.u, what)


# -- paths (L §5.16; E §7) -----------------------------------------------------------------------------

def m_path_terms(self, n, env, ctx):
    """The query's argument (and where) on every path: [(kept, value)]."""
    from .evaluate import Coll
    if ctx.path is not None:
        # L §5.16: a path p line "holds no query over paths (LOAD-RANDOM)"
        raise refuse('LOAD-RANDOM', 'a query over paths on a path p line')
    N, S = self.ask_paths
    key = (id(n), tuple(sorted((k, repr(v)) for k, v in env.items())), ctx.key())
    if self.path_cache is None:
        self.path_cache = {}
    hit = self.path_cache.get(key)
    if hit is not None:
        terms, coll = hit
        self.cur.merge(coll)
        return terms
    terms = []
    coll = Coll()
    self.stack.append(coll)
    try:
        for p in range(1, N + 1):
            pctx = ctx.but(path=(S, p))
            c = Coll()
            self.stack.append(c)
            try:
                keep = True
                if getattr(n, 'where', None) is not None:
                    w = self.ev(n.where, env, pctx)
                    if isinstance(w, Hole):
                        keep = w
                    elif w is NONE:
                        raise refuse('RUN-NONE', 'a where is none here')
                    else:
                        keep = bool(w)
                v = self.ev(n.arg if n.k == 'query' else n.e, env, pctx) if keep is not False else None
            finally:
                self.stack.pop()
            c.reg = set(e for e in c.reg if e[0] != 'sample')
            coll.merge(c)
            terms.append((keep, v))
    finally:
        self.stack.pop()
    self.path_cache[key] = (terms, coll)
    self.cur.merge(coll)
    return terms


def m_path_stat(self, n, env, ctx, to=None):
    """share, mean or quantile over the paths, with the counts and standard errors (L §5.16)."""
    terms = self.path_terms(n, env, ctx)
    N = len(terms)
    holes = [k for k, v in terms if isinstance(k, Hole)] + [v for k, v in terms if k is True and isinstance(v, Hole)]
    if holes:
        h = _ev().merge(*holes)
        return {'value': h, 'hole': True}
    kept = [v for k, v in terms if k is True]
    M = len(kept)
    if M == 0:
        raise refuse('RUN-EMPTY', '%s over no paths: the where leaves none' % n.fn)
    for v in kept:
        if v is NONE:
            raise refuse('RUN-NONE', 'a term of %s is none on a path' % n.fn)
    st = {'fn': n.fn, 'N': N, 'M': M, 'where': getattr(n, 'where', None) is not None}
    if n.fn == 'share':
        cnt = sum(1 for v in kept if v is True)
        s = Fraction(cnt, M)
        st.update(count=cnt, value=Num(s, ()), se=self.se_share(s, M))
        return st
    if n.fn == 'mean':
        acc = None
        for v in kept:
            if not isinstance(v, Num):
                raise refuse('LOAD-UNIT', 'mean of %s' % fmt(v))
            acc = v if acc is None else _ev().arith('+', acc, v, None, None, None)
        mean = Num(acc.q / M, acc.u, enc=None if acc.enc is None else
                   ((None, None) if acc.enc[0] is None else (acc.enc[0] / M, acc.enc[1] / M)))
        st.update(value=mean, se=lambda: self.se_mean([v.q * (U.conv(v.u, acc.u) or 1) for v in kept], mean.q))
        return st
    q = self.ev(n.q, env, ctx)
    if isinstance(q, Hole):
        return {'value': q, 'hole': True}
    self.no_none(q, 'quantile')
    if q.q <= 0 or q.q > 1:
        raise refuse('RUN-DOMAIN', 'quantile at %s: no smallest value with at least %s of the samples at or '
                     'below it' % (fmt_q(q.q), fmt_q(q.q)))
    if all(isinstance(v, Num) for v in kept):
        srt = sorted(kept, key=lambda v: v.q * (U.conv(v.u, kept[0].u) or 1))
    else:
        srt = sorted(kept, key=lambda v: self.cmp_key(v))
    need = q.q * M
    rank = M
    for i, v in enumerate(srt, 1):
        if i >= need:
            rank = i
            break
    st.update(value=srt[rank - 1])
    # L §5.16 (round 19, GD-3): the standard error by the Maritz–Jarrett method, over the kept samples (E §7 step 4);
    # m = ⌈q·N⌉ is the rank of the type-1 value
    if all(isinstance(v, Num) for v in srt):
        xs = [v.q * (U.conv(v.u, srt[0].u) or 1) for v in srt]
        st.update(se=lambda: self.se_quantile(xs, rank))
    else:
        st.update(se=lambda: _mj_unspecified(srt))
    return st


def _mj_unspecified(srt):
    raise Unspecified('§5.16', 'the Maritz-Jarrett standard error of a quantile over paths whose samples are no numbers '
                      '(%s): C_k = sum W_i x(i)^k has no meaning for them, and L §5.16 states no other' % fmt(srt[0]))


# Writer 10, GD-3: a test may run with `=== options mj=check` (not language): the weights are then also computed as
# the binomial tail, compared with the Beta distribution function's polynomial, their sum checked to be 1 exactly,
# and the line prints them.
MJ_CHECK = [False]


def mj_beta_S(N, m):
    """S(i) = N^N · I(i/N), i = 0..N, I the distribution function of Beta(m, N - m + 1) as its polynomial
    I(x) = sum_{k=m..N} (-1)^(k-m) C(N, k) C(k-1, m-1) x^k (integers: Horner in i with r_k = p_k N^(N-k))."""
    from math import comb
    r = [0] * (N + 1)
    for k in range(m, N + 1):
        r[k] = (-1) ** (k - m) * comb(N, k) * comb(k - 1, m - 1) * N ** (N - k)
    out = []
    for i in range(N + 1):
        acc = r[N]
        for k in range(N - 1, m - 1, -1):
            acc = acc * i + r[k]
        out.append(acc * i ** m)
    return out


def mj_tail_S(N, m):
    """S(i) = N^N · P(Binomial(N, i/N) >= m) = sum_{j=m..N} C(N, j) i^j (N - i)^(N - j), i = 0..N."""
    from math import comb
    return [sum(comb(N, j) * i ** j * (N - i) ** (N - j) for j in range(m, N + 1)) for i in range(N + 1)]


def m_se_quantile(self, xs, m):
    """L §5.16 (round 19): with x(1) <= ... <= x(N) and m the rank of the type-1 value, W_i = I(i/N) - I((i-1)/N), I
    the distribution function of Beta(m, N - m + 1) (equivalently the binomial tail, exact for whole N), C_k =
    sum W_i x(i)^k, the standard error sqrt(C2 - C1^2), correctly rounded to 1e-6 as mean's; none with N = 1."""
    N = len(xs)
    if N < 2:
        return None
    S = mj_beta_S(N, m)
    D = N ** N
    if MJ_CHECK[0]:
        T = mj_tail_S(N, m)
        if T != S:
            raise AssertionError('Maritz-Jarrett: the binomial tail and the Beta polynomial differ (N %d, m %d)' % (N, m))
    W = [Fraction(S[i] - S[i - 1], D) for i in range(1, N + 1)]
    if MJ_CHECK[0]:
        if sum(W) != 1:
            raise AssertionError('Maritz-Jarrett: the weights sum to %s' % sum(W))
        self.mj_seen.append('N %d, m %d, weights [%s] (binomial tail = Beta polynomial), sum %s' % (
            N, m, ', '.join(fmt_q(w) for w in W), fmt_q(sum(W))))
    c1 = sum(w * x for w, x in zip(W, xs))
    c2 = sum(w * x * x for w, x in zip(W, xs))
    var = c2 - c1 * c1
    if var < 0:
        raise AssertionError('Maritz-Jarrett: a negative variance')
    x = R.real_sqrt(var)
    v, _ = R.round_to(x, Fraction(1, 10 ** 6), 'the standard error')
    return v


def m_cmp_key(self, v):
    if isinstance(v, Key):
        ix = v.index
        return self.key_ord(v, ('index', ix))
    return v


def m_se_share(self, s, M):
    """sqrt(s(1 - s)/N), correctly rounded to 1e-6 (L §5.16)."""
    x = R.real_sqrt(s * (1 - s) / M)
    v, _ = R.round_to(x, Fraction(1, 10 ** 6), 'the standard error')
    return v


def m_se_mean(self, xs, mean):
    """s / sqrt(N), s^2 the sample variance over N - 1, correctly rounded to 1e-6 (L §5.16)."""
    N = len(xs)
    if N < 2:
        # L §5.16 (round 12): "with N = 1 the standard error is none"
        return None
    var = sum((x - mean) ** 2 for x in xs) / (N - 1)
    x = R.real_sqrt(var / N)
    v, _ = R.round_to(x, Fraction(1, 10 ** 6), 'the standard error')
    return v


# -- the printed form of a query on an ask line ----------------------------------------------------------

class QOut:
    """A query printed with its figures (L §5.16): this interpreter's layout."""

    def __init__(self, text):
        self.text = text

    def key(self):
        return ('qout', self.text)


def _horizon(coll):
    ds = [e[1] for e in coll.reg if e[0] == 'hz' and e[1] is not None]
    return max(ds).isoformat() if ds else 'none'


def m_query_line(self, e, env, ctx):
    """An ask line's expression that is a query over random values, printed with its figures."""
    from .evaluate import Coll
    x = strip(e)
    to = None
    if x.k == 'to' and getattr(x, '_form', None) in ('query', 'crps'):
        to = x
        x = strip(x.e)
    c = Coll()
    self.stack.append(c)
    try:
        if x.k == 'crps':
            v = self.e_crps(x, env, ctx, to)
            st = None
        elif getattr(x, '_qmode', None) == 'paths':
            st = self.path_stat(x, env, ctx, to)
            v = st['value']
            if to is not None and isinstance(v, Num) and not st.get('hole'):
                v = self.round_exactish(v, to, env, ctx, x.fn)
        else:
            v, _ = self.closed_query(x, env, ctx, to)
            st = None
    finally:
        self.stack.pop()
    hz = _horizon(c)
    c.reg = set(r for r in c.reg if r[0] != 'hz')
    self.cur.merge(c)
    if isinstance(v, Hole):
        return v
    from .values import fmt_typed
    vt = fmt_typed(v, getattr(x, '_t', None)) if not isinstance(v, Key) else fmt(v)
    if st is None or st.get('hole'):
        return QOut('%s (horizon %s)' % (vt, hz))
    kept = '%d paths' % st['N'] if not st['where'] else '%d of %d paths' % (st['M'], st['N'])
    if st['fn'] == 'share':
        return QOut('%d of %d, %s (standard error %s; %s; horizon %s)' % (
            st['count'], st['M'], vt, fmt_q(st['se']), kept, hz))
    if st['fn'] in ('mean', 'quantile'):
        # L §5.16: mean's standard error; quantile's by the Maritz-Jarrett method (round 19, GD-3), printed as mean's
        self.mj_seen = []
        se = st['se']()
        extra = ''.join('; maritz-jarrett %s' % t for t in self.mj_seen)
        return QOut('%s (standard error %s; %s; horizon %s%s)' % (vt, 'none' if se is None else fmt_q(se), kept, hz,
                                                                 extra))
    return QOut('%s (%s; horizon %s)' % (vt, kept, hz))


# -- crps (L §5.15) ----------------------------------------------------------------------------------

def m_e_crps(self, n, env, ctx, to=None):
    form = n._form
    y = self.ev(n.y, env, ctx)
    # a hole observation doesn't stop the evaluation: the forecast is evaluated too, so a refusal there outranks the
    # hole ("Refusals outrank holes: when both reach a result, the result is refused", L §5.11), and a hole there
    # adds its questions (L §8.1 It carries)
    yh = y if isinstance(y, Hole) else None
    if yh is None:
        self.no_none(y, 'crps')
    if form == 'normal':
        d = self.ev(strip(n.e), env, ctx)
        if isinstance(d, Hole) or yh is not None:
            return _ev().merge(*[h for h in (yh, d) if isinstance(h, Hole)])
        m, s = d.params
        f = U.conv(y.u, d.unit) if y.u and d.unit and y.u != d.unit else Fraction(1)
        z = (y.q * (f or 1) - m) / s
        r = crps_normal_real(z, s)
        return self.round_real(r, to, env, ctx, d.unit, 'crps')
    if form == 'paths':
        terms = self.path_terms(n, env, ctx)
        vals = [v for k, v in terms]
        ws = None
    else:
        mem = list(self.members([n.bind], env, ctx))
        vals = []
        ws = [] if form == 'weighted' else None
        wvals = []
        holes = []
        for e2, st in mem:
            if isinstance(st, tuple) and st[0] == 'domainhole':
                return st[1]
            if isinstance(st, Hole):
                holes.append(st)
            t = self.ev(n.e, e2, ctx)
            if ws is not None:
                w = self.ev(n.weight, e2, ctx)
                if isinstance(w, Hole):
                    holes.append(w)
                    continue
                self.no_none(w, 'crps')
                ws.append(w.q)
                wvals.append(w)
            vals.append(t)
        if holes:
            return _ev().merge(*(holes + ([yh] if yh is not None else [])))
    hs = [v for v in vals if isinstance(v, Hole)]
    if yh is not None:
        hs.append(yh)
    if hs:
        return _ev().merge(*hs)
    if not vals:
        raise refuse('RUN-EMPTY', 'crps over no keys')
    u = None
    xs = []
    for v in vals:
        self.no_none(v, 'crps')
        if not isinstance(v, Num):
            raise refuse('LOAD-UNIT', 'crps of %s' % fmt(v))
        if u is None:
            u = v.u
        xs.append(v.q * (U.conv(v.u, u) or 1))
    yq = y.q * (U.conv(y.u, u) if y.u and u and y.u != u else 1)
    if ws is None:
        val = crps_sample(xs, yq)
    else:
        tot = sum(ws)
        if tot == 0:
            raise refuse('RUN-DIV', 'division by zero: crps weights that sum to 0')
        val = crps_weighted(xs, [w / tot for w in ws], yq)
    # the error bound (L §7.3): a crps over values or weights that rest on rounded transcendentals is computed by
    # interval arithmetic over L §5.15's formula, each rounded value widened to its enclosure
    xe = [_ev().enc_of(v, u) for v in vals]
    we = [_ev().enc_of(w) for w in wvals] if ws is not None else None
    ye = _ev().enc_of(y, u)
    enc = None
    if any(v.enc is not None for v in vals) or y.enc is not None or (
            ws is not None and any(w.enc is not None for w in wvals)):
        enc = crps_enclosure(xe, we, ye)
    out = Num(val, u or (), enc=enc)
    if to is not None:
        return self.round_exactish(out, to, env, ctx, 'crps')
    return out


def _iv_abs(a):
    lo, hi = a
    if lo >= 0:
        return a
    if hi <= 0:
        return (-hi, -lo)
    return (Fraction(0), max(-lo, hi))


def _iv_sub(a, b):
    return (a[0] - b[1], a[1] - b[0])


def _iv_mul(a, b):
    ps = [a[0] * b[0], a[0] * b[1], a[1] * b[0], a[1] * b[1]]
    return (min(ps), max(ps))


def crps_enclosure(xe, we, ye):
    """The enclosure of L §5.15's crps, by interval arithmetic over its formula as written (L §7.3): xe, we, ye the
    enclosures of the values, the weights (None: unweighted) and the observation. A term |x_i - x_i| is 0 (a value
    compared with itself, L §4.4). (None, None) when unbounded (weights whose sum may be 0)."""
    n = len(xe)
    if n * n > 16000000:
        raise NotYet('the error bound of a crps over more than 4 000 values that rest on rounded values')
    if we is None:
        a = (Fraction(0), Fraction(0))
        for x in xe:
            t = _iv_abs(_iv_sub(x, ye))
            a = (a[0] + t[0], a[1] + t[1])
        p = (Fraction(0), Fraction(0))
        for i in range(n):
            for j in range(n):
                if i != j:
                    t = _iv_abs(_iv_sub(xe[i], xe[j]))
                    p = (p[0] + t[0], p[1] + t[1])
        return (a[0] / n - p[1] / (2 * n * n), a[1] / n - p[0] / (2 * n * n))
    W = (sum(w[0] for w in we), sum(w[1] for w in we))
    if W[0] <= 0 <= W[1]:
        return (None, None)
    hw = []
    for w in we:
        ps = [w[0] / W[0], w[0] / W[1], w[1] / W[0], w[1] / W[1]]
        hw.append((min(ps), max(ps)))
    a = (Fraction(0), Fraction(0))
    for x, h in zip(xe, hw):
        t = _iv_mul(h, _iv_abs(_iv_sub(x, ye)))
        a = (a[0] + t[0], a[1] + t[1])
    p = (Fraction(0), Fraction(0))
    for i in range(n):
        for j in range(n):
            if i != j:
                t = _iv_mul(_iv_mul(hw[i], hw[j]), _iv_abs(_iv_sub(xe[i], xe[j])))
                p = (p[0] + t[0], p[1] + t[1])
    return (a[0] - p[1] / 2, a[1] - p[0] / 2)


def crps_sample(xs, y):
    """(1/n) sum |x_i - y| - (1/(2n^2)) sum sum |x_i - x_j|, exact, through order statistics (L §5.15)."""
    n = len(xs)
    s = sorted(xs)
    a = sum(abs(x - y) for x in s)
    # sum_{i,j} |x_i - x_j| = 2 * sum_i (2i - n - 1) x_(i), i from 1
    pair = 2 * sum((2 * i - n - 1) * x for i, x in enumerate(s, 1))
    return Fraction(a, 1) / n - Fraction(pair, 1) / (2 * n * n)


def crps_weighted(xs, ws, y):
    """sum w_i |x_i - y| - 1/2 sum sum w_i w_j |x_i - x_j| with normalised weights (L §5.15)."""
    a = sum(w * abs(x - y) for x, w in zip(xs, ws))
    s = sorted(zip(xs, ws))
    W = Fraction(0)
    S = Fraction(0)
    pair = Fraction(0)
    for x, w in s:
        pair += w * (x * W - S)
        W += w
        S += w * x
    return a - pair     # 1/2 * 2 * sum_{i<j}


def crps_normal_real(z, s):
    """s [z (2 Phi(z) - 1) + 2 phi(z) - 1/sqrt(pi)] (L §5.15)."""
    z = Fraction(z)
    s = Fraction(s)

    def f(W):
        G = 16 + max(abs(z.numerator).bit_length() - z.denominator.bit_length(), 0) + \
            max(abs(s.numerator).bit_length() - s.denominator.bit_length(), 0)
        V = W + G
        P = R.Phi_q(z, V)
        p = R.phi_q(z, V)
        sp = R.sqrtpi(V)
        t = R.imulq(R.isub(R.imulq(P, 2), R.fi(1, V)), z)
        t = R.iadd(t, R.imulq(p, 2))
        inv = R.idiv(R.fi(1, V), sp, V)
        t = R.isub(t, inv)
        t = R.imulq(t, s)
        return t[0] >> G, -((-t[1]) >> G)
    return R.Real(f, None, 'crps')


# -- sample pins (L §5.16 Pins) ----------------------------------------------------------------------------

def m_sample_pin(self, p, env, ctx, pinsv):
    """with sample x[k] = v, x[*] = e, x[k].u = 0.37: the pinned expression is evaluated unpinned and must be
    known (L §5.16)."""
    from .model import _kv, key_text
    from .evaluate import PinVal, Coll
    from .values import fmt_typed
    info = p.info
    keys = []
    ktexts = []
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
                # L §5.11: "A pin whose key is a hole ... make[s] the value inside the as if, or the ask line, a hole
                # carrying their questions, and nothing inside is evaluated"
                from .evaluate import HoleSignal
                raise HoleSignal(kv)
            k = self.to_key(kv, conv, b.dom, kn)
        keys.append(_kv(k))
        ktexts.append(key_text(k))
    c = Coll()
    self.stack.append(c)
    try:
        v = self.ev(p.value, env, ctx)
    finally:
        self.stack.pop()
    self.cur.merge(c)
    addr = '%s%s' % (info.id, ('[%s]' % ', '.join(ktexts)) if ktexts else '')
    if isinstance(v, Hole):
        # L §5.16 Pins: "The pinned expression is evaluated unpinned; a hole pins the hole (§5.11)"; L §5.11: "A value
        # or sample pin whose value is a hole pins that hole" (a uniform that is a hole makes the sample that hole)
        from .values import fmt_hole
        text = '%s%s = %s' % (addr, '.u' if p.ufield else '', fmt_hole(v))
        pv = PinVal('sample', info.id, tuple(keys) if p.target.keys is not None else None, (), v, text)
        pinsv = [q for q in pinsv if not (q.kind in ('sample', 'sampleu') and q.item == pv.item and q.keys == pv.keys)]
        pinsv.append(pv)
        self.reg(('pinned sample', text))
        return pinsv
    if p.ufield:
        if not isinstance(v, Num) or not 0 < v.q < 1:
            raise refuse('RUN-DOMAIN', 'sample %s.u = %s: a uniform lies strictly between 0 and 1 (L §5.16 step 3)'
                         % (addr, fmt(v)))
        text = '%s.u = %s' % (addr, fmt_q(v.q))
        kind = 'sampleu'
    else:
        text = '%s = %s' % (addr, fmt_typed(v, p.ty))
        kind = 'sample'
    pv = PinVal(kind, info.id, tuple(keys) if p.target.keys is not None else None, (), v, text)
    pinsv = [q for q in pinsv if not (q.kind in ('sample', 'sampleu') and q.item == pv.item and q.keys == pv.keys)]
    pinsv.append(pv)
    self.reg(('pinned sample', text))
    return pinsv


# -- solve (L §5.18) ------------------------------------------------------------------------------------

def m_e_solve(self, n, env, ctx):
    return self.solve_value(n, env, ctx)


def m_solve_value(self, s, env, ctx):
    """The grid point nearest the root of F(v) = f - g, a tie going up (L §5.18 steps 1-3)."""
    from .model import _kv
    from .evaluate import PinVal, arith
    kind, info, keyconv = s._target
    vt = s._vt
    keys = []
    for kn, conv, b in zip(s.ref.keys if s.ref.k == 'index' else [], keyconv, info.binds):
        if conv[0] == 'lit':
            k = conv[1]
        elif conv[0] == 'axis':
            k = Key(conv[1], self.axis_option(conv[1], ctx))
        else:
            kv = self.ev(kn, env, ctx)
            if isinstance(kv, Hole):
                return kv
            k = self.to_key(kv, conv, b.dom, kn)
        keys.append(_kv(k))
    lo = self.ev(s.lo, env, ctx)
    hi = self.ev(s.hi, env, ctx)
    for v in (lo, hi):
        if isinstance(v, Hole):
            return v
        self.no_none(v, 'solve')
    if vt.k == 'rate':
        uu = s.unit._u
        if lo.u != uu or hi.u != uu:
            raise refuse('LOAD-UNIT', 'solve: the bracket and the step are rates of different periods')
        step = s.unit._val
        unit = uu
    else:
        unit = vt.u if vt.k in ('num', 'money') else ()
        step = self.step_value(s.unit, env, ctx, unit)
        if isinstance(step, Hole):
            return step
    lq = lo.q * (U.conv(lo.u, unit) if lo.u and lo.u != unit else 1)
    hq = hi.q * (U.conv(hi.u, unit) if hi.u and hi.u != unit else 1)
    if hq < lq or ((hq - lq) / step).denominator != 1:
        raise refuse('LOAD-DOMAIN', 'solve: [lo, hi] is not on the grid lo + i u')

    def mk(q):
        if vt.k == 'rate':
            return Num(q, unit, rate=True)
        if vt.k == 'money':
            return Num(q, unit, money=True)
        if vt.k == 'int':
            return Num(q, (), isint=q.denominator == 1)
        return Num(q, unit)
    holes = []

    def F(q):
        pv = PinVal('solve', info.id, tuple(keys), (), mk(q), '')
        pins = tuple(sorted([p for p in ctx.pins if not (p.item == info.id and p.kind in ('set', 'solve'))] +
                            [pv], key=lambda p: repr(p.key())))
        c2 = ctx.but(pins=pins)
        fv = self.ev(s.f, env, c2)
        gv = self.ev(s.g, env, c2)
        if isinstance(fv, Hole) or isinstance(gv, Hole):
            holes.append(_ev().merge(fv, gv))
            raise _SolveHole()
        self.no_none(fv, 'solve')
        self.no_none(gv, 'solve')
        d = arith('-', fv, gv, None, s.f._t, s.g._t)
        return (d.q > 0) - (d.q < 0)
    try:
        flo = F(lq)
        if flo == 0:
            return mk(lq)
        fhi = F(hq)
        if fhi == 0:
            return mk(hq)
        if flo == fhi:
            raise refuse('RUN-SOLVE', 'no sign change on [%s, %s]' % (fmt(lo), fmt(hi)))
        a, b = lq, hq
        fa = flo
        while b - a > step:
            mid = a + step * ((b - a) // (2 * step))
            fm = F(mid)
            if fm == 0:
                return mk(mid)
            if fm == fa:
                a, fa = mid, fm
            else:
                b = mid
        c = a + step / 2
        fc = F(c)
        if fc == 0 or fc == fa:
            return mk(b)
        return mk(a)
    except _SolveHole:
        return _ev().merge(*holes)


class _SolveHole(Exception):
    pass


# -- prorata (L §5.13) -------------------------------------------------------------------------------------

def m_e_prorata(self, n, env, ctx):
    from .evaluate import round_mode, merge
    total = self.ev(n.total, env, ctx)
    if isinstance(total, Hole):
        return total
    self.no_none(total, 'prorata')
    b = n.binds[0]
    mem = list(self.members([b], env, ctx))
    keys = []
    ws = []
    holes = []
    for e2, st in mem:
        if isinstance(st, tuple) and st[0] == 'domainhole':
            return st[1]
        if isinstance(st, Hole):
            holes.append(st)
        w = self.ev(n.w, e2, ctx)
        if isinstance(w, Hole):
            holes.append(w)
            continue
        self.no_none(w, 'prorata')
        keys.append(e2[b.var])
        ws.append(w)
    if holes:
        return merge(*holes)
    k = self.ev(n.key, env, ctx)
    if isinstance(k, Hole):
        return k
    from .values import vkey
    pos = [i for i, x in enumerate(keys) if vkey(x) == vkey(k)]
    # L §5.13 step 1: "A key k that the binding leaves out (its where false) has share 0"; the split is still made, so
    # its refusals stand
    out_k = not pos
    i_k = pos[0] if pos else 0
    u0 = ws[0].u if ws else ()
    wq = []
    for w in ws:
        f = U.conv(w.u, u0) if w.u != u0 else Fraction(1)
        if f is None:
            raise refuse('LOAD-UNIT', 'prorata: weights of different units')
        wq.append(w.q * f)
    tot_w = sum(wq)
    if any(w < 0 for w in wq) or tot_w <= 0:
        raise refuse('RUN-WEIGHTS', 'prorata weights [%s] are negative or sum to 0' % ', '.join(fmt_q(w) for w in wq))
    u = self.ev(n.u, env, ctx)
    if isinstance(u, Hole):
        return u
    self.no_none(u, 'prorata')
    if getattr(n.u, '_stepflex', False):
        u = Num(u.q, total.u)
    f = U.conv(u.u, total.u) if u.u != total.u else Fraction(1)
    if f is None:
        raise refuse('LOAD-UNIT', 'prorata: the step %s and the total %s have different units' % (fmt(u), fmt(total)))
    step = u.q * f
    if step <= 0:
        raise refuse('RUN-DOMAIN', 'prorata with a step %s that is not positive' % fmt(u))
    T_ = total.q
    raw = [T_ * w / tot_w for w in wq]
    if n.rest in ('none', 'last'):
        mn = n.mode
        if getattr(mn, '_r', None) and mn._r[0] == 'axismode':
            mode = Code('rounding', self.axis_option(mn._r[1], ctx))
        else:
            mode = self.ev(mn, env, ctx)
        if isinstance(mode, Hole):
            return mode
        if not isinstance(mode, Code) or mode.kind != 'rounding':
            raise refuse('LOAD-UNIT', 'prorata: the mode is %s' % fmt(mode))
        if n.rest == 'none' or i_k != len(raw) - 1:
            q = round_mode(raw[i_k] / step, mode.word) * step
        else:
            q = T_ - sum(round_mode(r / step, mode.word) * step for r in raw[:-1])
    else:
        if (T_ / step).denominator != 1:
            raise refuse('RUN-PRORATA', '%s isn\'t a whole number of %s' % (fmt(total), fmt(u)))
        # L §5.13 rest largest: each raw share "rounded toward zero to u (down)"; the leftover units "each of the
        # total's sign" go to the largest remainders |raw_j - share_j|, "so a negative total's shares are the negatives
        # of its absolute value's": split |total| and give back the sign
        sign = -1 if T_ < 0 else 1
        T_ = abs(T_)
        raw = [T_ * w / tot_w for w in wq]
        import math
        down = [math.floor(r / step) for r in raw]
        rems = [r - d * step for r, d in zip(raw, down)]
        left = int(T_ / step) - sum(down)
        order = sorted(range(len(raw)), key=lambda i: -rems[i])
        give = set(order[:left])
        if 0 < left < len(raw):
            edge = rems[order[left - 1]]
            tied = [i for i in range(len(raw)) if rems[i] == edge]
            inside = [i for i in tied if i in give]
            if len(inside) != len(tied):
                # the tie changes which keys take a unit: ties by decides it (D-5)
                if not n.tie:
                    raise refuse('RUN-TIE', 'prorata has a tie at %s that changes the result, and no ties by' %
                                 ', '.join(fmt(keys[i]) for i in tied))
                scored = []
                for i in tied:
                    e2 = dict(env)
                    e2[b.var] = keys[i]
                    vals = [self.ev(x, e2, ctx) for x in n.tie]
                    for v in vals:
                        if isinstance(v, Hole):
                            return v
                    scored.append((vals, i))
                import functools

                def cmp(a, c):
                    for x, y in zip(a[0], c[0]):
                        r = self.cmp3(x, y, None)
                        if r:
                            return r
                    return 0
                scored.sort(key=functools.cmp_to_key(cmp))
                need = len(inside)
                if need < len(scored) and cmp(scored[need - 1], scored[need]) == 0:
                    raise refuse('RUN-TIE', 'prorata has a tie at %s that changes the result, and its ties by doesn\'t '
                                 'break it' % ', '.join(fmt(keys[i]) for i in tied))
                give = (give - set(tied)) | set(i for v, i in scored[:need])
        q = sign * (down[i_k] + (1 if i_k in give else 0)) * step
    if out_k:
        q = Fraction(0)
    cur = n._money
    if cur is not None:
        if (q / CURRENCIES[cur]).denominator != 1:
            raise refuse('LOAD-UNIT', 'prorata to money: the step %s is not a whole number of minor units' % fmt(u))
        return Money(q, cur)
    return Num(q, total.u)


# -- fill (L §5.12) -----------------------------------------------------------------------------------------

def m_fill_keys(self, nodes, convs, doms, env, ctx):
    keys = []
    for kn, conv, dom in zip(nodes, convs, doms):
        if conv[0] == 'lit':
            k = conv[1]
        elif conv[0] == 'axis':
            k = Key(conv[1], self.axis_option(conv[1], ctx))
        else:
            kv = self.ev(kn, env, ctx)
            if isinstance(kv, Hole):
                return None, kv
            if kv is NONE:
                raise refuse('RUN-NONE', 'a key is none here')
            k = self.to_key(kv, conv, dom, kn)
        keys.append(k)
    return keys, None


def m_e_fillread(self, n, env, ctx):
    from .model import _kv
    qid = n._r[1]
    fi = self.m.checker.fill_infos[qid]
    fkeys, h = self.fill_keys(n._fkeys, n._fconvs, fi['fdoms'], env, ctx)
    if h is not None:
        return h
    tkeys, h = self.fill_keys(n._tkeys, n._convs, fi['sdoms'] + fi['ddoms'], env, ctx)
    if h is not None:
        return h
    takes = self.fill_cell(qid, fkeys, ctx)
    hit = takes.get(tuple(_kv(k) for k in tkeys))
    if hit is None:
        t = fi['taken']
        return Money(0, t.cur) if t.k == 'money' else Num(0, t.u or ())
    v, coll = hit
    self.cur.merge(coll)
    return v


def m_fill_cell(self, qid, fkeys, ctx):
    from .model import _kv
    from .evaluate import Coll
    if ctx.path is not None and qid not in getattr(self.m, 'random_fills', ()):
        ctx = ctx.but(path=None)
    ck = ('fill', qid, tuple(_kv(k) for k in fkeys), ctx.key())
    hit = self.memo.get(ck)
    if hit is not None:
        v, err, coll = hit
        if err is not None:
            self.cur.merge(coll)
            raise err
        return v
    if ck in self.active:
        raise refuse('RUN-CYCLE', 'fill %s reads itself through a computed key' % qid)
    self.active.add(ck)
    c = Coll()
    self.stack.append(c)
    err = None
    v = None
    try:
        v = self.fill_compute(qid, fkeys, ctx)
    except TallyError as e:
        err = e
    finally:
        self.stack.pop()
        self.active.discard(ck)
    self.memo[ck] = (v, err, c)
    if err is not None:
        self.cur.merge(c)
        raise err
    return v


def m_lots_holes(self, sups, dems, s, ctx):
    """{supply key: HOLE-LOTS hole} for the lots held before the run (observed acquisitions dated before S that the
    fill binds and nothing books) at a place whose opening reading at S states another sum (L §5.17 Lots, §8.1:
    `<place>: the lots held before <S> sum to <a>, the opening reading states <b>`). The lots held are the pre-run
    observations a side binds, into the place less out of it, unit by unit."""
    from .model import _kv
    from .values import Thing
    S = self.m.S
    pre = {}
    members = []
    for k, e2, v, c in sups:
        members.append(('supply', k, e2))
    for k, e2, v, c in dems:
        members.append(('demand', k, e2))
    seen = set()
    by_key = {}
    for what, k, e2, in members:
        for var, x in e2.items():
            if not isinstance(x, Mv):
                continue
            src = x.src if x.src is not None else None
            if src is None:
                try:
                    src = self.mv_record(x, ctx)
                except TallyError:
                    continue
            if getattr(src, 'status', None) != 'observed' or not isinstance(getattr(src, 'day', None), datetime.date) \
                    or not src.day < S:
                continue
            for lg in src.legs:
                for p, sg in ((lg.to, 1), (lg.frm, -1)):
                    if not isinstance(p, Thing) or not isinstance(lg.amount, Num):
                        continue
                    if what == 'supply':
                        by_key.setdefault(k, set()).add(p.name)
                    if (src.id, lg.name, p.name) in seen:
                        continue
                    seen.add((src.id, lg.name, p.name))
                    u = lg.unit or '*'
                    pre[(p.name, u)] = pre.get((p.name, u), Fraction(0)) + sg * lg.amount.q
    if not pre:
        return {}
    opening = {}
    for iid, place, unit, r in self.readings(self.tctx(ctx)):
        if iid != 'opening' or r.instant[0] == 'after' or r.instant[0] != S:
            continue
        val = r.val
        if isinstance(val, Num):
            u = unit or '*'
            opening[(place, u)] = val
    bad = {}
    for (place, u), a in sorted(pre.items()):
        ov = opening.get((place, u))
        if ov is None and u != '*':
            ov = opening.get((place, '*'))
        if ov is None:
            continue
        if ov.q != a:
            unit = u if u != '*' else None
            atxt = fmt(Num(a, ov.u, money=ov.money)) if ov.money else fmt(Num(a, ov.u))
            q = ('%s: the lots held before %s sum to %s, the opening reading states %s' % (place, S, atxt, fmt(ov)),
                 place)
            bad[place] = q
    out = {}
    for k, places in by_key.items():
        qs = [bad[p] for p in sorted(places) if p in bad]
        if qs:
            out[k] = Hole(qs)
    return out


def m_fill_compute(self, qid, fkeys, ctx):
    """The procedure of L §5.12: arcs, sort, start, take; with the holes rule."""
    import functools
    from .model import _kv
    from .evaluate import Coll, merge
    s = self.m.fills[qid]
    fi = self.m.checker.fill_infos[qid]
    env0 = {b.var: self.keyval(k) for b, k in zip(s.binds, fkeys)}

    def run(fn):
        c = Coll()
        self.stack.append(c)
        try:
            v = fn()
        finally:
            self.stack.pop()
        return v, c

    def side(expr, binds, what):
        out = []
        if binds:
            # L §5.12 Sides: "A key whose membership in a side is a hole (its binding's day, span or where a hole,
            # §5.7) takes part with every arc of it a hole condition carrying those questions"
            mem = list(self.members(binds, env0, ctx, possible=True))
        else:
            mem = [(env0, True)]
        for e2, st in mem:
            if isinstance(st, tuple) and st[0] == 'domainhole':
                raise NotYet('fill %s: a side whose binding over an index has a hole span' % qid)
            v, c = run(lambda: self.ev(expr, e2, ctx))
            if v is NONE and isinstance(st, Hole):
                v = st
            if v is NONE:
                raise refuse('RUN-NONE', 'a %s of fill %s is none here' % (what, qid))
            if isinstance(v, Num) and v.q < 0:
                raise refuse('RUN-NEG', 'negative amount %s for fill %s' % (fmt(v), qid))
            key = tuple(_kv(e2[b.var]) for b in binds)
            out.append((key, e2, v, c))
            if isinstance(st, Hole):
                held[key, what] = st
        return out
    held = {}
    sups = side(s.supply, s.sbinds, 'supply')
    dems = side(s.demand, s.dbinds, 'demand')
    lots = self.lots_holes(sups, dems, s, ctx)
    if lots:
        # HOLE-LOTS (L §5.17 Lots, §8.1): the lots held before the run disagree with the opening reading that anchors
        # their sum: each such lot is a hole carrying the question
        sups = [(k, e2, lots.get(k, v) if not isinstance(v, Hole) else v, c) for k, e2, v, c in sups]
    tt = fi['taken']
    unit = tt.u or ()
    arcs = []
    takes = {}
    for sk, se, sv, sc_ in sups:
        for dk, de, dv, dc_ in dems:
            env = dict(se)
            env.update(de)

            def arc_eval():
                try:
                    return self.ev(s.arc, env, ctx)
                except TallyError as e:
                    if load_now(e):
                        raise
                    raise refuse(e.code, 'fill %s, the arc of (%s; %s): %s' % (
                        qid, ', '.join(fmt(se[b.var]) for b in s.sbinds), ', '.join(fmt(de[b.var]) for b in s.dbinds),
                        e.msg))
            a, ac = run(arc_eval)
            if a is False:
                takes[sk + dk] = (Num(0, unit, money=tt.k == 'money'), ac)
                continue
            if a is NONE:
                raise refuse('RUN-NONE', 'the arc of fill %s is none here' % qid)
            hm = [h for h in (held.get((sk, 'supply')), held.get((dk, 'demand'))) if h is not None]
            if hm:
                # a key whose membership is a hole: every arc of it is a hole condition (L §5.12 Sides)
                a = merge(*(hm + ([a] if isinstance(a, Hole) else [])))
            ords, oc = run(lambda: [self.ev(o, env, ctx) for o in s.order])
            for o in ords:
                if isinstance(o, Hole):
                    raise refuse('RUN-ORDER', 'fill %s: the order of the arc (%s; %s) is a hole' % (
                        qid, ', '.join(fmt(se[b.var]) for b in s.sbinds), ', '.join(fmt(de[b.var]) for b in s.dbinds)))
            ties, tc = run(lambda: [self.ev(o, env, ctx) for o in (s.tie or [])])
            for o in ties:
                if isinstance(o, Hole):
                    raise refuse('RUN-ORDER', 'fill %s: the ties by of an arc is a hole' % qid)
            ac.merge(oc)
            ac.merge(tc)
            arcs.append({'sk': sk, 'dk': dk, 'se': se, 'de': de, 'sv': sv, 'dv': dv, 'a': a, 'ord': ords,
                         'tie': ties, 'c': ac, 'sc': sc_, 'dc': dc_})

    def cmp1(a, b, arc):
        # L §4.4: a comparison on a window "is decided when the whole interval lies on one side", and "a value
        # compared with itself is decided" (x <= x even for a window); L §5.12 Sort: "An order that can't be decided
        # (a hole) is refused (RUN-ORDER)"
        from .values import Iv
        if isinstance(a, Iv) or isinstance(b, Iv):
            lt = self.iv_compare('<', a, b, None)
            gt = self.iv_compare('>', a, b, None)
            if lt is True:
                return -1
            if gt is True:
                return 1
            if lt is False and gt is False:
                return 0
            raise refuse('RUN-ORDER', 'fill %s: the order of the arc (%s; %s) is a hole' % (
                qid, ', '.join(fmt(arc['se'][bb.var]) for bb in s.sbinds),
                ', '.join(fmt(arc['de'][bb.var]) for bb in s.dbinds)))
        return self.cmp3(a, b, None)

    def cmpv(x, y, k):
        for a, b in zip(x[k], y[k]):
            r = cmp1(a, b, x)
            if r:
                return r
        return 0
    arcs.sort(key=functools.cmp_to_key(lambda x, y: cmpv(x, y, 'ord') or cmpv(x, y, 'tie')))
    groups = []
    for x in arcs:
        if groups and cmpv(groups[-1][0], x, 'ord') == 0 and cmpv(groups[-1][0], x, 'tie') == 0:
            groups[-1].append(x)
        else:
            groups.append([x])

    def simulate(seq, log=None):
        R = {sk: (sv, sc_) for sk, se, sv, sc_ in sups}
        Q = {dk: (dv, dc_) for dk, de, dv, dc_ in dems}
        out = {}

        def step(x):
            (r, rc), (q, qc) = R[x['sk']], Q[x['dk']]
            c = Coll()
            c.merge(x['c'])
            c.merge(rc)
            c.merge(qc)
            zero = lambda v: isinstance(v, Num) and v.q == 0
            if zero(r) or zero(q):
                t = Num(0, unit, money=tt.k == 'money')
                out[x['sk'] + x['dk']] = (t, c)
                return
            if isinstance(r, Hole) or isinstance(q, Hole) or isinstance(x['a'], Hole):
                h = merge(*[v for v in (r, q, x['a']) if isinstance(v, Hole)])
                out[x['sk'] + x['dk']] = (h, c)
                R[x['sk']] = (h, c)
                Q[x['dk']] = (h, c)
                return
            f = U.conv(q.u, r.u) if q.u != r.u else Fraction(1)
            if f is None:
                # supply and demand "share one unit" (L §5.12); one load couldn't see is checked here, value by value
                # (L §3.2, §8.4: RUN-DOMAIN) (MC-X-F10)
                raise refuse('RUN-DOMAIN', 'fill %s: the supply %s and the demand %s of the arc (%s; %s) have '
                                           'different units' % (
                                               qid, fmt(r), fmt(q), ', '.join(fmt(x['se'][b.var]) for b in s.sbinds),
                                               ', '.join(fmt(x['de'][b.var]) for b in s.dbinds)))
            qq = q.q * f
            t = min(r.q, qq)
            tv = Num(t, r.u, money=r.money)
            out[x['sk'] + x['dk']] = (tv, c)
            R[x['sk']] = (Num(r.q - t, r.u, money=r.money), c)
            Q[x['dk']] = (Num(q.q - t / f, q.u, money=q.money), c)
        for x in seq:
            before = (R[x['sk']][0], Q[x['dk']][0])
            step(x)
            if log is not None:
                log.append((x, before[0], before[1], out[x['sk'] + x['dk']][0], R[x['sk']][0], Q[x['dk']][0]))
        return out
    seq = [x for g in groups for x in g]
    log = [] if getattr(self, 'fill_trace', None) is not None else None
    res = simulate(seq, log)
    if log is not None:
        # explain of a fill take (E §10): the arcs in order, the remaining supply and demand before and after each
        self.fill_trace.append((qid, tuple(_kv(k) for k in fkeys), s, log))
    for i, g in enumerate(groups):
        if len(g) < 2:
            continue
        rev = [x for gg in groups[:i] for x in gg] + list(reversed(g)) + [x for gg in groups[i + 1:] for x in gg]
        alt = simulate(rev)
        if any(_vk(res[k][0]) != _vk(alt[k][0]) for k in res):
            raise refuse('RUN-TIE', 'fill %s has a tie at %s that changes the result, and %s' % (
                qid, '; '.join('(%s; %s)' % (', '.join(fmt(x['se'][b.var]) for b in s.sbinds),
                                             ', '.join(fmt(x['de'][b.var]) for b in s.dbinds)) for x in g),
                'its ties by doesn\'t break it' if s.tie else 'no ties by'))
    takes.update(res)
    return takes


def _vk(v):
    from .values import vkey
    return vkey(v)
