"""Correctly rounded transcendental values (L §7.3), by interval arithmetic.

A real is an object with an exact value when one is known (a rational: exp(0),
ln(1), a perfect power, Phi(0) = 1/2, ...) and an enclosure function: given a
working precision W it returns integers lo <= hi with the true value inside
[lo * 2^-W, hi * 2^-W].  Every operation rounds outward, so the enclosure is
rigorous at every W; a larger W only makes it narrower.

`round_to(x, u)` refines W (64, 128, ... up to 2^16 bits) until the enclosure
lies inside one rounding cell of the step u, ties half-even (L §7.3), and
refuses RUN-UNDECIDED past the cap.  No floating point enters a result: floats
are used only as first guesses of a quantile, which are then verified.
"""
from fractions import Fraction
import math

from .errors import Refusal

CAP = 1 << 16          # bits of precision (L §7.3: 2^16 bits)
START = 64


class Undecided(Exception):
    pass


def refuse(code, msg):
    return Refusal(code, msg)


# -- fixed-point intervals: (lo, hi) integers at scale 2^-W --------------------------------------

def fi(q, W):
    """A rational as an interval at scale W."""
    q = Fraction(q)
    n = q.numerator << W
    d = q.denominator
    lo = n // d
    hi = -((-n) // d)
    return lo, hi


def iadd(a, b):
    return a[0] + b[0], a[1] + b[1]


def isub(a, b):
    return a[0] - b[1], a[1] - b[0]


def ineg(a):
    return -a[1], -a[0]


def imul(a, b, W):
    p = (a[0] * b[0], a[0] * b[1], a[1] * b[0], a[1] * b[1])
    lo = min(p) >> W
    hi = -((-max(p)) >> W)
    return lo, hi


def imulq(a, q):
    """Interval times a rational (exact scale, outward)."""
    q = Fraction(q)
    n, d = q.numerator, q.denominator
    if n >= 0:
        x, y = a[0] * n, a[1] * n
    else:
        x, y = a[1] * n, a[0] * n
    return x // d, -((-y) // d)


def idiv(a, b, W):
    if b[0] <= 0 <= b[1]:
        raise Undecided('division by an interval holding 0')
    los = []
    his = []
    for x in a:
        for y in b:
            n = x << W
            los.append(n // y)
            his.append(-((-n) // y))
    return min(los), max(his)


def isqr(a, W):
    """Square of an interval."""
    if a[0] >= 0:
        return (a[0] * a[0]) >> W, -((-(a[1] * a[1])) >> W)
    if a[1] <= 0:
        return (a[1] * a[1]) >> W, -((-(a[0] * a[0])) >> W)
    m = max(-a[0], a[1])
    return 0, -((-(m * m)) >> W)


def isqrt_iv(a, W):
    """sqrt of an interval of non-negative values."""
    lo = max(a[0], 0)
    hi = a[1]
    if hi < 0:
        raise Undecided('sqrt of a negative interval')
    l = math.isqrt(lo << W)
    h = math.isqrt(hi << W)
    if h * h < (hi << W):
        h += 1
    return l, h


def to_frac(a, W):
    return Fraction(a[0], 1 << W), Fraction(a[1], 1 << W)


def ihull(*ivs):
    return min(x[0] for x in ivs), max(x[1] for x in ivs)


# -- constants ---------------------------------------------------------------------------------

_cache = {}


def _atanh_inv(k, W):
    """atanh(1/k) for an integer k >= 2, as an interval at scale W."""
    # sum_{n>=0} 1/((2n+1) k^(2n+1)); remainder after the term n=N is below the next term / (1 - 1/k^2)
    one = 1 << (W + 8)
    s_lo = s_hi = 0
    p = one // k          # 1/k^(2n+1), floored
    pk = k * k
    n = 0
    while True:
        t = p // (2 * n + 1)
        s_lo += t
        s_hi += t + 2       # floor errors of p and of the division
        p //= pk
        n += 1
        if p == 0:
            break
    # remainder: below 2 * (next term) <= 2 ulps
    s_hi += 4
    lo = s_lo >> 8
    hi = -((-s_hi) >> 8)
    return lo, hi


def ln2(W):
    k = ('ln2', W)
    if k not in _cache:
        a = _atanh_inv(3, W + 4)
        _cache[k] = ((2 * a[0]) >> 4, -((-(2 * a[1])) >> 4))
    return _cache[k]


def _atan_inv(k, W):
    """atan(1/k) for an integer k >= 2 (alternating series, interval)."""
    G = 8
    one = 1 << (W + G)
    p = one // k
    pk = k * k
    s = 0
    n = 0
    err = 0
    while p:
        t = p // (2 * n + 1)
        if n % 2 == 0:
            s += t
        else:
            s -= t
        err += 2
        p //= pk
        n += 1
    err += 2
    lo = (s - err) >> G
    hi = -((-(s + err)) >> G)
    return lo, hi


def pi(W):
    k = ('pi', W)
    if k not in _cache:
        a = _atan_inv(5, W + 6)
        b = _atan_inv(239, W + 6)
        lo = 16 * a[0] - 4 * b[1]
        hi = 16 * a[1] - 4 * b[0]
        _cache[k] = (lo >> 6, -((-hi) >> 6))
    return _cache[k]


def sqrt2pi(W):
    k = ('sqrt2pi', W)
    if k not in _cache:
        p = pi(W + 4)
        two_pi = (2 * p[0], 2 * p[1])
        s = isqrt_iv(two_pi, W + 4)
        _cache[k] = (s[0] >> 4, -((-s[1]) >> 4))
    return _cache[k]


def sqrtpi(W):
    k = ('sqrtpi', W)
    if k not in _cache:
        p = pi(W + 4)
        s = isqrt_iv(p, W + 4)
        _cache[k] = (s[0] >> 4, -((-s[1]) >> 4))
    return _cache[k]


# -- exp and ln at a rational point --------------------------------------------------------------

def exp_q(x, W):
    """An enclosure of e^x, x rational, at scale W."""
    x = Fraction(x)
    if x == 0:
        return 1 << W, 1 << W
    # reduce: r = x / 2^s with |r| <= 1/2
    s = 0
    ax = abs(x)
    while ax > Fraction(1, 2):
        ax /= 2
        s += 1
    G = s + 12
    V = W + G
    r = fi(x / (1 << s), V)
    one = 1 << V
    # Taylor sum_{k=0}^{K} r^k / k!, |r| <= 1/2 + ulp
    acc = (one, one)
    term = (one, one)
    k = 1
    while True:
        term = imul(term, r, V)
        term = (term[0] // k, -((-term[1]) // k))
        acc = iadd(acc, term)
        k += 1
        if max(abs(term[0]), abs(term[1])) < 4:
            break
    # remainder: |sum_{j>k} r^j/j!| <= 2 |term| (ratio <= 1/2 + ulp), plus a few ulps
    acc = (acc[0] - 8, acc[1] + 8)
    if acc[0] < 0:
        acc = (0, acc[1])
    for _ in range(s):
        acc = isqr(acc, V)
    return acc[0] >> G, -((-acc[1]) >> G)


def ln_q(x, W):
    """An enclosure of ln x, x a positive rational, at scale W."""
    x = Fraction(x)
    if x <= 0:
        raise ValueError('ln of a non-positive value')
    if x == 1:
        return 0, 0
    # x = 2^k * m, m in [1, 2)
    k = x.numerator.bit_length() - x.denominator.bit_length()
    m = x / (Fraction(2) ** k)
    while m >= 2:
        m /= 2
        k += 1
    while m < 1:
        m *= 2
        k -= 1
    G = 12 + abs(k).bit_length()
    V = W + G
    z = (m - 1) / (m + 1)          # in [0, 1/3)
    zi = fi(z, V)
    z2 = imul(zi, zi, V)
    acc = (0, 0)
    p = zi
    n = 0
    while True:
        t = p
        d = 2 * n + 1
        t = (t[0] // d, -((-t[1]) // d))
        acc = iadd(acc, t)
        if t[1] < 2:
            break
        p = imul(p, z2, V)
        n += 1
    acc = (acc[0] - 4, acc[1] + 4)    # remainder below the next term / (1 - z^2), and floor errors
    lnm = (2 * acc[0], 2 * acc[1])
    l2 = ln2(V)
    kl = (k * l2[0], k * l2[1]) if k >= 0 else (k * l2[1], k * l2[0])
    tot = iadd(lnm, kl)
    return tot[0] >> G, -((-tot[1]) >> G)


def sqrt_q(x, W):
    x = Fraction(x)
    a = fi(x, 2 * W)
    lo = math.isqrt(a[0])
    hi = math.isqrt(a[1])
    if hi * hi < a[1]:
        hi += 1
    return lo, hi


def exact_root(x, b):
    """The b-th root of a non-negative rational if it is rational, else None."""
    x = Fraction(x)
    if x < 0:
        return None

    def iroot(n):
        if n == 0:
            return 0
        r = int(round(n ** (1.0 / b))) if n.bit_length() < 1000 else None
        if r is None:
            lo, hi = 0, 1 << (n.bit_length() // b + 1)
            while lo < hi:
                mid = (lo + hi) // 2
                if mid ** b < n:
                    lo = mid + 1
                else:
                    hi = mid
            r = lo
        for c in (r - 1, r, r + 1):
            if c >= 0 and c ** b == n:
                return c
        return None
    a = iroot(x.numerator)
    if a is None:
        return None
    d = iroot(x.denominator)
    if d is None:
        return None
    return Fraction(a, d)


# -- the normal distribution -------------------------------------------------------------------

def phi_q(z, W):
    """phi(z) = e^(-z^2/2) / sqrt(2 pi)."""
    z = Fraction(z)
    G = 8
    V = W + G
    e = exp_q(-z * z / 2, V)
    s = sqrt2pi(V)
    r = idiv(e, s, V)
    return r[0] >> G, -((-r[1]) >> G)


def Phi_q(z, W):
    """Phi(z) = 1/2 + phi(z) * sum_{n>=0} z^(2n+1) / (2n+1)!!  (all terms positive for z > 0)."""
    z = Fraction(z)
    if z == 0:
        h = 1 << (W - 1)
        return h, h
    if z < 0:
        a = Phi_q(-z, W)
        one = 1 << W
        return one - a[1], one - a[0]
    if z > 38:
        # Mills' ratio: 0 < 1 - Phi(z) < phi(z) / z < e^(-z^2/2) < 2^-1040
        one = 1 << W
        lo = one - 1 if W < 1000 else one - (one >> 1000)
        return lo, one
    G = 16 + int(z * z).bit_length()
    V = W + G
    zi = fi(z, V)
    z2 = Fraction(z * z)
    t = zi
    acc = zi
    n = 0
    while True:
        n += 1
        t = imulq(t, z2 / (2 * n + 1))
        acc = iadd(acc, t)
        if 2 * n + 3 > 2 * z2 and t[1] < 4:
            break
    # tail: ratio < 1/2 from here, so the rest is below the last term, plus floor errors
    acc = (acc[0] - 4 - n, acc[1] + 4 + n + t[1])
    ph = phi_q(z, V)
    prod = imul(acc, ph, V)
    half = 1 << (V - 1)
    r = (prod[0] + half, prod[1] + half)
    return r[0] >> G, -((-r[1]) >> G)


# -- the gamma function and the regularized lower incomplete gamma ------------------------------

_bern = [Fraction(1), Fraction(-1, 2)]


def bernoulli(n):
    while len(_bern) <= n:
        m = len(_bern)
        s = Fraction(0)
        for k in range(m):
            s += math.comb(m + 1, k) * _bern[k]
        _bern.append(-s / (m + 1))
    return _bern[n]


def lngamma_q(a, W):
    """ln Gamma(a), a > 0 rational: shift to z = a + N and use Stirling's series with its remainder
    bound (for real z > 0 the error is below the first omitted term)."""
    a = Fraction(a)
    G = 20
    V = W + G
    N = max(0, int(W // 2) + 10 - int(a))
    z = a + N
    # (z - 1/2) ln z - z + ln(2 pi)/2 + sum_{j=1}^{n} B_2j / (2j (2j-1) z^(2j-1))
    lz = ln_q(z, V)
    t1 = imulq(lz, z - Fraction(1, 2))
    acc = isub(t1, fi(z, V))
    p2 = pi(V)
    l2p = ln_q(Fraction(2), V)
    lp_lo = ln_q(Fraction(p2[0], 1 << V), V)[0]
    lp_hi = ln_q(Fraction(p2[1], 1 << V), V)[1]
    half = ((l2p[0] + lp_lo) // 2, -((-(l2p[1] + lp_hi)) // 2))
    acc = iadd(acc, half)
    n = max(2, W // 6)
    for j in range(1, n + 1):
        c = bernoulli(2 * j) / (2 * j * (2 * j - 1) * z ** (2 * j - 1))
        acc = iadd(acc, fi(c, V))
    rem = abs(bernoulli(2 * n + 2)) / ((2 * n + 2) * (2 * n + 1) * z ** (2 * n + 1))
    rr = fi(rem, V)
    acc = (acc[0] - rr[1] - n, acc[1] + rr[1] + n)
    # subtract ln(a (a+1) ... (a+N-1))
    if N:
        prod = Fraction(1)
        for i in range(N):
            prod *= a + i
        lp = ln_q(prod, V)
        acc = isub(acc, lp)
    return acc[0] >> G, -((-acc[1]) >> G)


def gamma_exact(a):
    """Gamma(a) when it is rational (a a positive integer), else None."""
    a = Fraction(a)
    if a.denominator == 1 and a > 0:
        return Fraction(math.factorial(int(a) - 1))
    return None


def lower_P_q(a, x, W):
    """P(a, x) = gamma(a, x) / Gamma(a) = e^-x x^a sum_{n>=0} x^n / Gamma(a+n+1), x >= 0."""
    a, x = Fraction(a), Fraction(x)
    if x <= 0:
        return 0, 0
    G = 24 + 2 * int(x) + int(x).bit_length()
    V = W + G
    # x^a e^-x / Gamma(a+1) = exp(a ln x - x - lnGamma(a+1))
    la = lngamma_q(a + 1, V)
    lx = ln_q(x, V)
    expo = isub(imulq(lx, a), fi(x, V))
    expo = isub(expo, la)
    e_lo = exp_q(Fraction(expo[0], 1 << V), V)[0]
    e_hi = exp_q(Fraction(expo[1], 1 << V), V)[1]
    pre = (e_lo, e_hi)
    # sum_{n>=0} x^n / ((a+1)...(a+n))
    one = 1 << V
    t = (one, one)
    acc = (one, one)
    n = 0
    while True:
        n += 1
        t = imulq(t, x / (a + n))
        acc = iadd(acc, t)
        if a + n + 1 > 2 * x and t[1] < 4:
            break
    acc = (acc[0] - 2 * n, acc[1] + 2 * n + t[1])
    r = imul(pre, acc, V)
    one = 1 << V
    r = (max(0, r[0]), min(one, r[1]))
    return r[0] >> G, -((-r[1]) >> G)


# -- reals and rounding ------------------------------------------------------------------------

class Real:
    """f(W) -> (lo, hi) integers at scale W; ex: the exact rational value when known."""
    __slots__ = ('f', 'ex', 'name')

    def __init__(self, f, ex=None, name='a value'):
        self.f = f
        self.ex = ex
        self.name = name

    def enclose(self, W):
        if self.ex is not None:
            return fi(self.ex, W)
        return self.f(W)

    def bounds(self, W):
        return to_frac(self.enclose(W), W)


def const(q):
    q = Fraction(q)
    return Real(lambda W: fi(q, W), q)


def real_exp(x):
    x = Fraction(x)
    return Real(lambda W: exp_q(x, W), Fraction(1) if x == 0 else None, 'exp')


def real_ln(x):
    x = Fraction(x)
    if x <= 0:
        raise refuse('RUN-DOMAIN', 'ln(%s) outside its domain' % _q(x))
    return Real(lambda W: ln_q(x, W), Fraction(0) if x == 1 else None, 'ln')


def real_sqrt(x):
    x = Fraction(x)
    if x < 0:
        raise refuse('RUN-DOMAIN', 'sqrt(%s) outside its domain' % _q(x))
    return Real(lambda W: sqrt_q(x, W), exact_root(x, 2), 'sqrt')


def real_pow(x, q):
    """x^q, q rational (a non-integer q: x >= 0)."""
    x, q = Fraction(x), Fraction(q)
    if q.denominator == 1:
        if x == 0 and q < 0:
            raise refuse('RUN-DIV', 'division by zero')
        return const(x ** int(q))
    if x < 0:
        raise refuse('RUN-DOMAIN', 'pow(%s, %s) outside its domain' % (_q(x), _q(q)))
    if x == 0:
        if q < 0:
            raise refuse('RUN-DIV', 'division by zero')
        return const(0)
    r = exact_root(x, q.denominator)
    if r is not None:
        return const(r ** q.numerator)

    def f(W):
        G = 16 + abs(q.numerator).bit_length()
        V = W + G
        l = ln_q(x, V)
        e = imulq(l, q)
        lo = exp_q(Fraction(e[0], 1 << V), V)[0]
        hi = exp_q(Fraction(e[1], 1 << V), V)[1]
        return lo >> G, -((-hi) >> G)
    return Real(f, None, 'pow')


def real_from(f, ex=None, name='a value'):
    return Real(f, ex, name)


def _q(x):
    from .values import fmt_q
    return fmt_q(x)


def round_half_even(k):
    """The nearest integer to the rational k, ties to even."""
    fl = k.numerator // k.denominator
    r = k - fl
    if r > Fraction(1, 2):
        return fl + 1
    if r < Fraction(1, 2):
        return fl
    return fl if fl % 2 == 0 else fl + 1


def round_to(x, u, what='a value'):
    """x (a Real) correctly rounded to the step u (a positive rational), ties half-even (L §7.3).
    Returns (value, enclosure) where the enclosure is (lo, hi) of the true value: [v - u/2, v + u/2],
    or the exact value as a point when it is known."""
    u = Fraction(u)
    if x.ex is not None:
        v = round_half_even(x.ex / u) * u
        return v, (x.ex, x.ex)
    W = START
    while W <= CAP:
        try:
            lo, hi = x.bounds(W)
        except Undecided:
            W *= 2
            continue
        a = lo / u
        b = hi / u
        ma = round_half_even(a)
        mb = round_half_even(b)
        # strictly inside one cell: no edge (m + 1/2) in [a, b]
        if ma == mb and a > ma - Fraction(1, 2) and b < ma + Fraction(1, 2):
            v = ma * u
            return v, (v - u / 2, v + u / 2)
        W *= 2
    raise refuse('RUN-UNDECIDED', '%s to %s not decided within the precision cap' % (what, _q(u)))


def compare(x, q, what='a comparison'):
    """The sign of x - q, decided (exact when x is known exactly)."""
    q = Fraction(q)
    if x.ex is not None:
        return (x.ex > q) - (x.ex < q)
    W = START
    while W <= CAP:
        try:
            lo, hi = x.bounds(W)
        except Undecided:
            W *= 2
            continue
        if lo > q:
            return 1
        if hi < q:
            return -1
        W *= 2
    raise refuse('RUN-UNDECIDED', '%s not decided within the precision cap' % what)


def hull_monotone(fn, lo, hi, increasing=True):
    """The hull of the exact values of a monotone function over [lo, hi], as rationals: rigorous lower
    and upper bounds (fn(q) returns a Real)."""
    a = fn(lo)
    b = fn(hi)
    if not increasing:
        a, b = b, a
    W = START
    al = a.bounds(W)[0] if a.ex is None else a.ex
    bh = b.bounds(W)[1] if b.ex is None else b.ex
    return al, bh
