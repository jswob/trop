"""Units and the unit rules (L §3.2).

A unit is a tuple of (base, power) pairs, sorted by base, with no zero
power.  Bases are currencies, declared units, keys of unit kinds and the
period words.  `%` is folded into the value as 1/100.  `year`, `quarter`
and `month` are one dimension (1 year = 4 quarters = 12 months); `day` is a
dimension of its own.  A base written `$v` stands for the bound variable v
of a declaration (`num PLN/u`) until a read substitutes the key.
"""
from fractions import Fraction

from .errors import LoadError

PERIOD_MONTHS = {'year': 12, 'quarter': 3, 'month': 1}
PERIODS = ('year', 'quarter', 'month', 'day')

ONE = ()


POWER_LIMIT = 64


def power_limit(b, e):
    """L §7.6 item 4 (tally 2.1.6, issue #89): every power of a unit, `%` included, written or reached by `*`, `/` and
    `pow`, is at most 64 in size. Powers are literals (L §7.1), so load meets each one; the caller adds the place."""
    if abs(e) > POWER_LIMIT:
        raise LoadError('LOAD-LIMIT', 'unit %s has a power past 64' % b)


def at(loc, fn, *args):
    """fn(*args), a refusal it raises without a place given loc."""
    try:
        return fn(*args)
    except LoadError as e:
        if e.loc is None:
            e.loc = loc
        raise


def make(terms):
    d = {}
    for b, e in terms:
        d[b] = d.get(b, 0) + e
    for b, e in d.items():
        power_limit(b, e)
    return tuple(sorted((b, e) for b, e in d.items() if e != 0))


def from_syntax(terms):
    """Unit syntax (from the parser) -> (unit, factor).  `%` gives 1/100 per power."""
    if not terms:
        return ONE, Fraction(1)
    for b, e in terms:
        power_limit(b, e)
    f = Fraction(1)
    out = []
    for b, e in terms:
        if b == '%':
            f *= Fraction(1, 100) ** e
        else:
            out.append((b, e))
    u, g = simplify(make(out))
    return u, f * g


def simplify(u):
    """Bring the period bases of the one dimension year/quarter/month to one base."""
    pbs = [b for b, e in u if b in PERIOD_MONTHS]
    if len(pbs) <= 1:
        return u, Fraction(1)
    return _to_base(u, pbs[0])


def _to_base(u, target):
    f = Fraction(1)
    out = []
    for b, e in u:
        if b in PERIOD_MONTHS and b != target:
            f *= Fraction(PERIOD_MONTHS[b], PERIOD_MONTHS[target]) ** e
            out.append((target, e))
        else:
            out.append((b, e))
    return make(out), f


def mul(a, b):
    """Units multiply: -> (unit, factor)."""
    pa = [x for x, e in a if x in PERIOD_MONTHS]
    pb = [x for x, e in b if x in PERIOD_MONTHS]
    terms = list(a) + list(b)
    u = make(terms)
    if pa and pb and set(pa + pb) != {pa[0]}:
        target = pa[0]
        return _to_base(u, target)
    return simplify(u)


def inv(a):
    return tuple((b, -e) for b, e in a)


def div(a, b):
    return mul(a, inv(b))


def power(a, n):
    for b, e in a:
        power_limit(b, e * n)
    return make([(b, e * n) for b, e in a])


def canon(u):
    """(month-normalised unit, factor) for comparing units of one dimension."""
    f = Fraction(1)
    out = []
    for b, e in u:
        if b in PERIOD_MONTHS:
            f *= Fraction(PERIOD_MONTHS[b]) ** e
            out.append(('month', e))
        else:
            out.append((b, e))
    return make(out), f


def conv(frm, to):
    """The factor f with x [frm] = x*f [to], or None when the units differ."""
    if frm == to:
        return Fraction(1)
    cf, ff = canon(frm)
    ct, ft = canon(to)
    if cf != ct:
        return None
    return ff / ft


def is_one(u):
    return not u


def currencies(u):
    return [(b, e) for b, e in u if len(b) == 3 and b.isupper()]


def is_currency(u):
    return len(u) == 1 and u[0][1] == 1 and len(u[0][0]) == 3 and u[0][0].isupper()


def subst(u, env):
    """Replace `$v` bases by the keys env gives."""
    if not any(b.startswith('$') for b, e in u):
        return u
    out = []
    for b, e in u:
        if b.startswith('$'):
            out.append((env.get(b[1:], b), e))
        else:
            out.append((b, e))
    return make(out)


def period_of(u):
    """For a rate unit 1/P: P, else None."""
    if len(u) == 1 and u[0][1] == -1 and u[0][0] in PERIODS:
        return u[0][0]
    return None


def fmt(u):
    if not u:
        return ''
    pos = [(b, e) for b, e in u if e > 0]
    neg = [(b, -e) for b, e in u if e < 0]

    def t(b, e):
        return b if e == 1 else '%s^%d' % (b, e)
    s = '*'.join(t(b, e) for b, e in pos) if pos else '1'
    for b, e in neg:
        s += '/' + t(b, e)
    return s


def day_and_period(u):
    """A unit mixing day with year/quarter/month: refused (L §3.2, `1 day / 1 year`)."""
    bs = set(b for b, e in u)
    return 'day' in bs and bool(bs & set(PERIOD_MONTHS))
