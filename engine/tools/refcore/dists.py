"""Distributions (L §5.16): the closed list, their functions and quantiles, and samples.

A Dist is a distribution value: its family, its parameters (exact rationals; keys and
weights for `discrete`), the unit of its values and the step
stated with it (`to u`), which rounds its samples.  cdf, pdf, pmf and mean return
reals.Real objects (exact where the spec says so); quantile returns the exact or
correctly rounded value (L §7.3).
"""
import hashlib
import math
from fractions import Fraction

from . import reals as R
from .errors import Refusal, LoadError

CONTINUOUS = ('normal', 'lognormal', 'exponential', 'gamma', 'uniform')
DISCRETE = ('poisson', 'binomial', 'negbin', 'discrete')
NEEDS_TO_CDF = ('normal', 'lognormal', 'exponential', 'gamma', 'poisson')   # cdf (L §5.16 table, §7.3)
NEEDS_TO_PDF = ('normal', 'lognormal', 'exponential', 'gamma')
NEEDS_TO_PMF = ('poisson',)


def refuse(code, msg):
    return Refusal(code, msg)


class Dist:
    __slots__ = ('fn', 'params', 'unit', 'step', 'items')

    def __init__(self, fn, params, unit, step=None, items=None):
        self.fn = fn
        self.params = tuple(Fraction(p) for p in params)
        self.unit = unit
        self.step = step
        self.items = items          # discrete: [(key, weight)]

    def key(self):
        it = None
        if self.items is not None:
            it = tuple((repr(k), w) for k, w in self.items)
        return ('dist', self.fn, self.params, self.unit, self.step, it)

    def with_step(self, step):
        return Dist(self.fn, self.params, self.unit, step, self.items)

    def text(self):
        from .values import fmt_q
        if self.fn == 'discrete':
            return '%s(...)' % self.fn
        s = '%s(%s)' % (self.fn, ', '.join(fmt_q(p) for p in self.params))
        if self.step is not None:
            s += ' to %s' % fmt_q(self.step)
        return s


def validate_known(fn, p):
    """The parameters known when another is a hole (None): each one that is invalid by itself is refused (RUN-PARAM,
    L §5.16), since a refusal outranks a hole (L §8.4)."""
    from .values import fmt_q

    def bad(name, v):
        return refuse('RUN-PARAM', '%s parameter %s = %s is invalid' % (fn, name, fmt_q(v)))

    def at(i):
        return p[i] if i < len(p) else None
    if fn in ('normal', 'lognormal') and at(1) is not None and at(1) <= 0:
        raise bad('s', at(1))
    if fn == 'exponential' and at(0) is not None and at(0) <= 0:
        raise bad('rate', at(0))
    if fn == 'gamma':
        if at(0) is not None and at(0) <= 0:
            raise bad('shape', at(0))
        if at(1) is not None and at(1) <= 0:
            raise bad('scale', at(1))
    if fn == 'poisson' and at(0) is not None and at(0) <= 0:
        raise bad('mean', at(0))
    if fn == 'binomial':
        if at(0) is not None and (at(0).denominator != 1 or at(0) < 0):
            raise bad('n', at(0))
        if at(1) is not None and not 0 <= at(1) <= 1:
            raise bad('p', at(1))
    if fn == 'negbin':
        if at(0) is not None and at(0) <= 0:
            raise bad('mean', at(0))
        if at(1) is not None and (at(1).denominator != 1 or at(1) <= 0):
            raise bad('size', at(1))


def validate(d):
    """An invalid parameter is refused at that key (RUN-PARAM, L §5.16)."""
    p = d.params
    fn = d.fn

    def bad(name, v):
        from .values import fmt_q
        return refuse('RUN-PARAM', '%s parameter %s = %s is invalid' % (fn, name, fmt_q(v)))
    if fn in ('normal', 'lognormal'):
        if p[1] <= 0:
            raise bad('s', p[1])
    elif fn == 'exponential':
        if p[0] <= 0:
            raise bad('rate', p[0])
    elif fn == 'gamma':
        if p[0] <= 0:
            raise bad('shape', p[0])
        if p[1] <= 0:
            raise bad('scale', p[1])
    elif fn == 'uniform':
        if not p[0] < p[1]:
            raise bad('b', p[1])
    elif fn == 'poisson':
        if p[0] <= 0:
            raise bad('mean', p[0])
    elif fn == 'binomial':
        if p[0].denominator != 1 or p[0] < 0:
            raise bad('n', p[0])
        if not 0 <= p[1] <= 1:
            raise bad('p', p[1])
    elif fn == 'negbin':
        if p[0] <= 0:
            raise bad('mean', p[0])
        if p[1].denominator != 1 or p[1] <= 0:
            raise bad('size', p[1])
    elif fn == 'discrete':
        if any(w < 0 for k, w in d.items):
            raise bad('w', min(w for k, w in d.items))
        if sum(w for k, w in d.items) <= 0:
            raise bad('w', sum(w for k, w in d.items))


# -- cdf, pdf, pmf -------------------------------------------------------------------------------

def _real_scaled(f_inner, scale_num, scale_den=1):
    """A real times a positive rational."""
    q = Fraction(scale_num, 1) / scale_den

    def f(W):
        a = f_inner(W + 4)
        b = R.imulq(a, q)
        return b[0] >> 4, -((-b[1]) >> 4)
    return f


def cdf(d, y):
    """P(X <= y) as a Real."""
    y = Fraction(y)
    fn, p = d.fn, d.params
    if fn == 'normal':
        m, s = p
        z = (y - m) / s
        return R.Real(lambda W: R.Phi_q(z, W), Fraction(1, 2) if z == 0 else None, 'cdf of normal')
    if fn == 'lognormal':
        m, s = p
        if y <= 0:
            return R.const(0)
        if y == 1 and m == 0:
            return R.const(Fraction(1, 2))

        def f(W):
            G = 16
            V = W + G
            l = R.ln_q(y, V)
            z = R.imulq(R.isub(l, R.fi(m, V)), 1 / s)
            lo = R.Phi_q(Fraction(z[0], 1 << V), V)[0]
            hi = R.Phi_q(Fraction(z[1], 1 << V), V)[1]
            return lo >> G, -((-hi) >> G)
        return R.Real(f, None, 'cdf of lognormal')
    if fn == 'exponential':
        rate = p[0]
        if y <= 0:
            return R.const(0)

        def f(W):
            e = R.exp_q(-rate * y, W)
            one = 1 << W
            return one - e[1], one - e[0]
        return R.Real(f, None, 'cdf of exponential')
    if fn == 'gamma':
        k, th = p
        if y <= 0:
            return R.const(0)
        return R.Real(lambda W: R.lower_P_q(k, y / th, W), None, 'cdf of gamma')
    if fn == 'uniform':
        a, b = p
        if y <= a:
            return R.const(0)
        if y >= b:
            return R.const(1)
        return R.const((y - a) / (b - a))
    if fn == 'poisson':
        lam = p[0]
        if y < 0:
            return R.const(0)
        kmax = math.floor(y)
        S = Fraction(0)
        t = Fraction(1)
        for i in range(kmax + 1):
            if i:
                t = t * lam / i
            S += t

        def f(W):
            e = R.exp_q(-lam, W + 8)
            r = R.imulq(e, S)
            return r[0] >> 8, -((-r[1]) >> 8)
        return R.Real(f, None, 'cdf of poisson')
    if fn == 'binomial':
        return R.const(_binom_cdf(p, y))
    if fn == 'negbin':
        return R.const(_negbin_cdf(p, y))
    raise ValueError(fn)


def cdf_key(d, k, order):
    """discrete over keys: P(X <= k) in index order."""
    tot = sum(w for kk, w in d.items)
    acc = Fraction(0)
    ok = order(k)
    for kk, w in d.items:
        if order(kk) <= ok:
            acc += w
    return acc / tot


def pmf_key(d, k):
    tot = sum(w for kk, w in d.items)
    for kk, w in d.items:
        if repr(kk) == repr(k):
            return Fraction(w) / tot
    return Fraction(0)


def pdf(d, y):
    y = Fraction(y)
    fn, p = d.fn, d.params
    if fn == 'normal':
        m, s = p
        z = (y - m) / s
        return R.Real(_real_scaled(lambda W: R.phi_q(z, W), 1 / s), None, 'pdf of normal')
    if fn == 'lognormal':
        m, s = p
        if y <= 0:
            return R.const(0)

        def f(W):
            G = 16
            V = W + G
            l = R.ln_q(y, V)
            z = R.imulq(R.isub(l, R.fi(m, V)), 1 / s)
            # phi is not monotone: hull over the z interval
            a = R.phi_q(Fraction(z[0], 1 << V), V)
            b = R.phi_q(Fraction(z[1], 1 << V), V)
            lo, hi = min(a[0], b[0]), max(a[1], b[1])
            if z[0] <= 0 <= z[1]:
                c = R.phi_q(Fraction(0), V)
                hi = max(hi, c[1])
            r = R.imulq((lo, hi), 1 / (y * s))
            return r[0] >> G, -((-r[1]) >> G)
        return R.Real(f, None, 'pdf of lognormal')
    if fn == 'exponential':
        rate = p[0]
        if y < 0:
            return R.const(0)
        if y == 0:
            return R.const(rate)
        return R.Real(_real_scaled(lambda W: R.exp_q(-rate * y, W), rate), None, 'pdf of exponential')
    if fn == 'gamma':
        k, th = p
        if y < 0:
            return R.const(0)
        if y == 0:
            if k == 1:
                return R.const(1 / th)
            if k > 1:
                return R.const(0)
            raise refuse('RUN-DOMAIN', 'pdf of gamma with shape %s at 0' % k)
        x = y / th

        def f(W):
            G = 24 + 2 * int(x)
            V = W + G
            # x^(k-1) e^-x / (Gamma(k) th) = exp((k-1) ln x - x - lnGamma(k)) / th
            lg = R.lngamma_q(k, V)
            lx = R.ln_q(x, V)
            e = R.isub(R.isub(R.imulq(lx, k - 1), R.fi(x, V)), lg)
            lo = R.exp_q(Fraction(e[0], 1 << V), V)[0]
            hi = R.exp_q(Fraction(e[1], 1 << V), V)[1]
            r = R.imulq((lo, hi), 1 / th)
            return r[0] >> G, -((-r[1]) >> G)
        return R.Real(f, None, 'pdf of gamma')
    if fn == 'uniform':
        a, b = p
        if a <= y <= b:
            return R.const(1 / (b - a))
        return R.const(0)
    raise refuse('RUN-DOMAIN', '%s has no pdf (it is discrete)' % fn)


def pmf(d, y):
    y = Fraction(y)
    fn, p = d.fn, d.params
    if fn == 'poisson':
        lam = p[0]
        if y < 0 or y.denominator != 1:
            return R.const(0)
        k = int(y)
        c = lam ** k / math.factorial(k)

        def f(W):
            e = R.exp_q(-lam, W + 8)
            r = R.imulq(e, c)
            return r[0] >> 8, -((-r[1]) >> 8)
        return R.Real(f, None, 'pmf of poisson')
    if fn == 'binomial':
        n, pp = int(p[0]), p[1]
        if y < 0 or y.denominator != 1 or y > n:
            return R.const(0)
        k = int(y)
        return R.const(math.comb(n, k) * pp ** k * (1 - pp) ** (n - k))
    if fn == 'negbin':
        if y < 0 or y.denominator != 1:
            return R.const(0)
        return R.const(_negbin_pmf(p, int(y)))
    raise refuse('RUN-DOMAIN', '%s has no pmf (it is continuous)' % fn)


def _binom_cdf(p, y):
    n, pp = int(p[0]), p[1]
    if y < 0:
        return Fraction(0)
    if y >= n:
        return Fraction(1)
    k = math.floor(y)
    return sum((math.comb(n, i) * pp ** i * (1 - pp) ** (n - i) for i in range(k + 1)), Fraction(0))


def _negbin_p(p):
    mean, size = p
    return size / (size + mean)


def _negbin_pmf(p, k):
    size = int(p[1])
    q = _negbin_p(p)
    return math.comb(k + size - 1, k) * q ** size * (1 - q) ** k


def _negbin_cdf(p, y):
    if y < 0:
        return Fraction(0)
    k = math.floor(y)
    return sum((_negbin_pmf(p, i) for i in range(k + 1)), Fraction(0))


# -- quantiles ----------------------------------------------------------------------------------

def _qcheck(d, q):
    """Type 1: the smallest x with cdf(x) >= q (L §5.16).  q <= 0 has no smallest x; q >= 1 has none
    for a support unbounded above."""
    if q <= 0 or q > 1:
        from .values import fmt_q
        raise refuse('RUN-DOMAIN', 'quantile of %s at %s: no smallest value with cdf at least %s'
                     % (d.fn, fmt_q(q), fmt_q(q)))


def quantile(d, q, step=None, order=None):
    """The q-quantile of d: exact for a discrete d, correctly rounded to the step for a continuous one.
    Returns (value, enclosure) where the enclosure is that of the true value."""
    q = Fraction(q)
    _qcheck(d, q)
    fn, p = d.fn, d.params
    if fn == 'discrete':
        tot = sum(w for k, w in d.items)
        acc = Fraction(0)
        for k, w in d.items:
            acc += w
            if acc / tot >= q:
                return k, None
        return d.items[-1][0], None
    if fn == 'binomial':
        n = int(p[0])
        acc = Fraction(0)
        for k in range(n + 1):
            acc += math.comb(n, k) * p[1] ** k * (1 - p[1]) ** (n - k)
            if acc >= q:
                return Fraction(k), None
        return Fraction(n), None
    if fn == 'negbin':
        if q == 1:
            raise refuse('RUN-DOMAIN', 'quantile of negbin at 1: its support is unbounded')
        acc = Fraction(0)
        k = 0
        while True:
            acc += _negbin_pmf(p, k)
            if acc >= q:
                return Fraction(k), None
            k += 1
    if fn == 'poisson':
        if q == 1:
            raise refuse('RUN-DOMAIN', 'quantile of poisson at 1: its support is unbounded')
        lam = p[0]
        # smallest k with cdf(k) >= q; cdf(k) is irrational, so the comparison is decided (L §7.3)
        k = max(0, int(float(lam) - 10 * math.sqrt(float(lam))) - 1) if lam > 50 else 0
        while k > 0 and R.compare(cdf(d, k - 1), q, 'quantile of poisson') >= 0:
            k -= 1
        while R.compare(cdf(d, k), q, 'quantile of poisson') < 0:
            k += 1
        return Fraction(k), None
    # continuous
    if step is None:
        raise LoadError('LOAD-TO', 'continuous distribution %s needs to u' % fn)
    if fn == 'uniform':
        a, b = p
        x = a + q * (b - a)
        return R.round_to(R.const(x), step, 'quantile of uniform')
    if fn == 'exponential':
        rate = p[0]
        if q == 1:
            raise refuse('RUN-DOMAIN', 'quantile of exponential at 1: its support is unbounded')

        def f(W):
            l = R.ln_q(1 - q, W + 8)
            r = R.imulq(l, -1 / rate)
            return r[0] >> 8, -((-r[1]) >> 8)
        return R.round_to(R.Real(f, None, 'quantile of exponential'), step, 'quantile of exponential')
    if q == 1:
        raise refuse('RUN-DOMAIN', 'quantile of %s at 1: its support is unbounded' % fn)
    if q == Fraction(1, 2) and fn == 'normal':
        # the median of a symmetric distribution, detected exactly (L §7.3)
        return R.round_to(R.const(p[0]), step, 'quantile of normal')
    if q == Fraction(1, 2) and fn == 'lognormal':
        return R.round_to(R.real_exp(p[0]), step, 'quantile of lognormal')
    return _solve_quantile(d, q, step)


def _guess(d, q):
    """A first guess of the q-quantile (a float; only a start for the verified search)."""
    fn, p = d.fn, d.params
    try:
        from statistics import NormalDist
        if fn == 'normal':
            return float(p[0]) + float(p[1]) * NormalDist().inv_cdf(float(q))
        if fn == 'lognormal':
            return math.exp(float(p[0]) + float(p[1]) * NormalDist().inv_cdf(float(q)))
        if fn == 'gamma':
            return _gamma_guess(float(p[0]), float(p[1]), float(q))
    except (ValueError, OverflowError, ZeroDivisionError):
        return None
    return None


def _gamma_cdf_float(k, x):
    if x <= 0:
        return 0.0
    # series for P(k, x)
    if x < k + 1:
        s = t = 1.0 / k
        n = 0
        while abs(t) > 1e-17 * abs(s) and n < 10000:
            n += 1
            t *= x / (k + n)
            s += t
        return s * math.exp(-x + k * math.log(x) - math.lgamma(k))
    # continued fraction for Q(k, x)
    b = x + 1 - k
    c = 1e300
    dd = 1 / b
    h = dd
    for i in range(1, 10000):
        an = -i * (i - k)
        b += 2
        dd = an * dd + b
        dd = 1e-300 if abs(dd) < 1e-300 else dd
        c = b + an / c
        c = 1e-300 if abs(c) < 1e-300 else c
        dd = 1 / dd
        de = dd * c
        h *= de
        if abs(de - 1) < 1e-16:
            break
    return 1 - math.exp(-x + k * math.log(x) - math.lgamma(k)) * h


def _gamma_guess(k, th, q):
    lo, hi = 0.0, max(1.0, k * 2)
    while _gamma_cdf_float(k, hi) < q:
        hi *= 2
        if hi > 1e300:
            return None
    for _ in range(200):
        mid = (lo + hi) / 2
        if _gamma_cdf_float(k, mid) < q:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2 * th


def quantile_affine(d, q, step, a, b):
    """quantile(a * X + b, q) of a continuous d, correctly rounded to the step (L §5.16 Closed forms: "a · quantile(D,
    q) + b for a > 0; a · quantile(D, 1 - q) + b for a < 0"): the cell of Y = aX + b holding its q-quantile, each cell
    edge y taken back to X's scale, (y - b)/a, and compared there with F (F(x) against q, or 1 - F(x) against q for
    a < 0), decided by interval refinement."""
    q = Fraction(q)
    a, b = Fraction(a), Fraction(b)
    if a > 0:
        _qcheck(d, q)
    else:
        _qcheck(d, 1 - q)
    if (a > 0 and q == 1) or (a < 0 and q == 0):
        raise refuse('RUN-DOMAIN', 'quantile of %s at %s: its support is unbounded' % (d.fn, 1 if a > 0 else 0))
    qq = q if a > 0 else 1 - q
    if d.fn == 'uniform':
        x = d.params[0] + qq * (d.params[1] - d.params[0])
        return R.round_to(R.const(a * x + b), step, 'quantile of uniform')
    if qq == Fraction(1, 2) and d.fn == 'normal':
        # the median of a symmetric distribution, detected exactly (L §7.3)
        return R.round_to(R.const(a * d.params[0] + b), step, 'quantile of normal')
    gx = _guess(d, q if a > 0 else 1 - q)
    if gx is None and d.fn == 'uniform':
        gx = float(d.params[0] + (q if a > 0 else 1 - q) * (d.params[1] - d.params[0]))
    g = float(a) * gx + float(b) if gx is not None and math.isfinite(gx) else None

    def side(edge):
        x = (edge - b) / a
        if a > 0:
            return R.compare(cdf(d, x), q, 'quantile of %s' % d.fn)
        return -R.compare(cdf(d, x), 1 - q, 'quantile of %s' % d.fn)
    return _solve_quantile(d, q, step, side=side, guess=g)


def _solve_quantile(d, q, step, side=None, guess=None):
    """The cell of the step holding F^-1(q), F continuous and increasing: F((j - 1/2)u) < q < F((j + 1/2)u),
    each comparison decided by interval refinement; F(edge) = q exactly is a tie (half-even)."""
    u = Fraction(step)
    g = _guess(d, q) if side is None else guess
    if g is not None and math.isfinite(g):
        j = int(round(g / float(u))) if abs(g / float(u)) < 1e300 else 0
    else:
        j = 0
    half = Fraction(1, 2)

    if side is None:
        def side(edge):
            """-1: F(edge) < q; +1: F(edge) > q; 0: equal (exact)."""
            return R.compare(cdf(d, edge), q, 'quantile of %s' % d.fn)
    # bracket: find j with side((j-1/2)u) < 0 and side((j+1/2)u) > 0
    lo_s = side((j - half) * u)
    hi_s = side((j + half) * u)
    if lo_s < 0 < hi_s:
        v = j * u
        return v, (v - u / 2, v + u / 2)
    # search outward by doubling, then bisect
    if lo_s >= 0:
        # the quantile is at or below the lower edge
        if lo_s == 0:
            return _tie(j - half, u, d, q)
        step_j = 1
        a = j
        while True:
            b = a - step_j
            s = side((b - half) * u)
            if s < 0:
                break
            if s == 0:
                return _tie(b - half, u, d, q)
            a = b
            step_j *= 2
        lo_j, hi_j = b, a     # side((lo_j - 1/2)u) < 0, side((hi_j - 1/2)u) > 0
    else:
        if hi_s == 0:
            return _tie(j + half, u, d, q)
        step_j = 1
        a = j
        while True:
            b = a + step_j
            s = side((b + half) * u)
            if s > 0:
                break
            if s == 0:
                return _tie(b + half, u, d, q)
            a = b
            step_j *= 2
        lo_j, hi_j = a + 1, b + 1    # side((lo_j - 1/2)u) < 0 ... side((hi_j - 1/2)u) > 0
    # invariant: side((lo_j - 1/2)u) < 0 and side((hi_j - 1/2)u) > 0; the answer is lo_j <= j < hi_j
    while hi_j - lo_j > 1:
        mid = (lo_j + hi_j) // 2
        s = side((mid - half) * u)
        if s == 0:
            return _tie(mid - half, u, d, q)
        if s < 0:
            lo_j = mid
        else:
            hi_j = mid
    v = lo_j * u
    return v, (v - u / 2, v + u / 2)


def _tie(k_edge, u, d, q):
    """F^-1(q) is exactly on the cell edge k_edge * u: half-even."""
    x = k_edge * u
    j = R.round_half_even(k_edge)
    v = j * u
    return v, (x, x)


def mean(d, step=None):
    """The mean as a Real (L §5.16 table)."""
    fn, p = d.fn, d.params
    if fn == 'normal':
        return R.const(p[0])
    if fn == 'lognormal':
        m, s = p
        return R.real_exp(m + s * s / 2)
    if fn == 'exponential':
        return R.const(1 / p[0])
    if fn == 'gamma':
        return R.const(p[0] * p[1])
    if fn == 'uniform':
        return R.const((p[0] + p[1]) / 2)
    if fn == 'poisson':
        return R.const(p[0])
    if fn == 'binomial':
        return R.const(p[0] * p[1])
    if fn == 'negbin':
        return R.const(p[0])
    raise refuse('RUN-DOMAIN', 'discrete has no mean: its values are keys, not numbers (L §5.16)')


# -- samples ------------------------------------------------------------------------------------

def uniform_of(address, seed, path):
    """u = (k + 1/2) / 2^64, k the first 8 bytes of SHA-256(address|S|p) big-endian (L §5.16)."""
    a = '%s|%d|%d' % (address, seed, path)
    k = int.from_bytes(hashlib.sha256(a.encode('utf-8')).digest()[:8], 'big')
    return Fraction(2 * k + 1, 1 << 65)


_sample_cache = {}


def sample(d, u):
    """quantile(D, u), correctly rounded to D's step for a continuous D, exact for a discrete one."""
    key = (d.key(), u)
    hit = _sample_cache.get(key)
    if hit is not None:
        return hit
    if d.fn in CONTINUOUS:
        if d.step is None:
            raise LoadError('LOAD-TO', 'continuous distribution %s needs to u' % d.fn)
        r = quantile(d, u, d.step)
    else:
        r = quantile(d, u)
    if len(_sample_cache) > 2000000:
        _sample_cache.clear()
    _sample_cache[key] = r
    return r
