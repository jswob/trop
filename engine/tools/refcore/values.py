"""Runtime values (L §3, §7) and how they print (L §2.4, §5.18 Output)."""
import datetime
from fractions import Fraction

from . import units as U
from .dates import Period
from .errors import Unspecified

CURRENCIES = {}   # code -> minor unit (Fraction); set by the loader


DIGIT_CAP = 4096
DIGIT_LIMIT = 10 ** DIGIT_CAP          # a value with more than 4 096 decimal digits is at least this
_CAP_BITS = DIGIT_LIMIT.bit_length() - 2


def dec_exp(a):
    """The exponent e with 10^e <= a < 10^(e+1), for a Fraction a > 0, found from bit lengths (no conversion of
    a big integer to a string)."""
    e = int((a.numerator.bit_length() - a.denominator.bit_length()) * 0.30102999566398120)
    while Fraction(10) ** e > a:
        e -= 1
    while Fraction(10) ** (e + 1) <= a:
        e += 1
    return e


class Num:
    """An exact rational with a unit.  money: a whole number of minor units of a
    currency; isint: an int; rate: an interest rate with its period (L §3.3).
    enc: for a value that rests on correctly rounded transcendentals, the interval
    (lo, hi) that holds the true real-number value (L §7.3 The error bound); (None,
    None) when it is unbounded; None for a value that rests on none."""
    __slots__ = ('q', 'u', 'money', 'isint', 'rate', 'enc')

    def __init__(self, q, u=(), money=False, isint=False, rate=False, enc=None):
        q = Fraction(q)
        # every exact value is capped at 4 096 decimal digits in its numerator and its denominator, counted on the
        # reduced fraction, where it is made (L §5.11, §7.1: RUN-DIGITS at that key); the bit length screens it
        # cheaply, with no conversion to a string
        if q.numerator.bit_length() > _CAP_BITS or q.denominator.bit_length() > _CAP_BITS:
            if abs(q.numerator) >= DIGIT_LIMIT or q.denominator >= DIGIT_LIMIT:
                from .errors import Refusal
                raise Refusal('RUN-DIGITS', 'a value passes 4 096 digits; round where the law rounds')
        self.q = q
        self.u = u
        self.money = money
        self.isint = isint
        self.rate = rate
        self.enc = enc

    def key(self):
        return ('num', self.q, self.u, self.money, self.isint, self.rate, self.enc)

    def __repr__(self):
        return 'Num(%s %s%s%s)' % (self.q, U.fmt(self.u), ' money' if self.money else '', ' int' if self.isint else '')


def Int(n):
    return Num(Fraction(n), (), isint=True)


def Money(q, cur):
    return Num(Fraction(q), ((cur, 1),), money=True)


class Text:
    __slots__ = ('s',)

    def __init__(self, s):
        self.s = s

    def key(self):
        return ('text', self.s)


class Key:
    """A key of a list index, an open index or an axis."""
    __slots__ = ('index', 'name')

    def __init__(self, index, name):
        self.index = index
        self.name = name

    def key(self):
        return ('key', self.index, self.name)

    def __repr__(self):
        return 'Key(%s:%s)' % (self.index, self.name)


class Thing:
    """A thing of a kind (L §5.7): identified by its key."""
    __slots__ = ('name',)

    def __init__(self, name):
        self.name = name

    def key(self):
        return ('thing', self.name)

    def __repr__(self):
        return 'Thing(%s)' % self.name


class Mv:
    """A movement (L §5.14, §5.19): a plan or decision instance, an observation, an adjustment or a pad,
    identified by its stable id (L §2.5).  `src` is the engine's record of it."""
    __slots__ = ('id', 'src')

    def __init__(self, id, src=None):
        self.id = id
        self.src = src

    def key(self):
        return ('mv', self.id)

    def __repr__(self):
        return 'Mv(%s)' % self.id


class Code:
    __slots__ = ('kind', 'word', 'n')

    def __init__(self, kind, word, n=None):
        self.kind = kind
        self.word = word
        self.n = n

    def key(self):
        return ('code', self.kind, self.word, self.n)


class Rec:
    __slots__ = ('fields',)

    def __init__(self, fields):
        self.fields = tuple(fields)

    def get(self, f):
        for a, b in self.fields:
            if a == f:
                return b
        raise KeyError(f)

    def has(self, f):
        return any(a == f for a, b in self.fields)

    def key(self):
        return ('rec', tuple((a, vkey(b)) for a, b in self.fields))


class Lst:
    __slots__ = ('items',)

    def __init__(self, items):
        self.items = tuple(items)

    def key(self):
        return ('list', tuple(vkey(x) for x in self.items))


class Tup:
    __slots__ = ('items',)

    def __init__(self, items):
        self.items = tuple(items)

    def key(self):
        return ('tup', tuple(vkey(x) for x in self.items))


class Special:
    __slots__ = ('name',)

    def __init__(self, name):
        self.name = name

    def key(self):
        return ('special', self.name)

    def __repr__(self):
        return self.name


NONE = Special('none')
NEVER = Special('never')


class Iv:
    """An interval of ordered values from a window (L §4.4).  `days` is the
    set of a one_of window; `src` names the window for a question; `origin`
    identifies the read it came from (a value compared with itself is decided)."""
    __slots__ = ('lo', 'hi', 'days', 'src', 'origin')

    def __init__(self, lo, hi, days=None, src=None, origin=None):
        self.lo = lo
        self.hi = hi
        self.days = days
        self.src = src
        self.origin = origin

    def key(self):
        return ('iv', vkey(self.lo), vkey(self.hi), tuple(sorted(self.days)) if self.days else None)

    def window_text(self):
        if self.src:
            return self.src
        return 'window(%s, %s)' % (fmt(self.lo), fmt(self.hi))


class Cal:
    __slots__ = ('weekends', 'yearly', 'yfrom', 'added')

    def __init__(self, weekends, yearly, yfrom, added):
        self.weekends = weekends
        self.yearly = yearly
        self.yfrom = yfrom
        self.added = added

    def key(self):
        return ('cal', tuple(sorted(self.weekends)), tuple(self.yearly), self.yfrom,
                tuple(repr(a) for a in self.added))


class Hole:
    """A missing fact (L §8.1): the questions it carries, each with the item and
    keys it came from.  `lin`, under a partial ask, is (known, {atom: coef})."""
    __slots__ = ('qs', 'lin')

    def __init__(self, qs, lin=None):
        self.qs = frozenset(qs)
        self.lin = lin

    def key(self):
        return ('hole', self.qs)

    def __repr__(self):
        return 'Hole(%s)' % sorted(self.qs)


def hole(text, origin, code='HOLE-STATED'):
    return Hole([(text, origin)])


def merge_holes(*vals):
    qs = set()
    for v in vals:
        if isinstance(v, Hole):
            qs |= v.qs
    return Hole(qs)


def vkey(v):
    if isinstance(v, bool):
        return ('bool', v)
    if isinstance(v, datetime.date):
        return ('date', v)
    if isinstance(v, Period):
        return ('period', v.key())
    if v is None:
        return ('nothing',)
    return v.key()


def is_hole(v):
    return isinstance(v, Hole)


# ---------------------------------------------------------------------------
# printing

def group3(intpart):
    s = str(intpart)
    if len(s) <= 3:
        return s
    out = []
    while len(s) > 3:
        out.insert(0, s[-3:])
        s = s[:-3]
    out.insert(0, s)
    return ' '.join(out)


def terminating(q):
    d = q.denominator
    for p in (2, 5):
        while d % p == 0:
            d //= p
    return d == 1


def dec_digits(q):
    """Number of decimals of a terminating fraction: the larger power of 2 or 5 in its denominator."""
    d = Fraction(q).denominator
    a2 = (d & -d).bit_length() - 1
    d >>= a2
    a5 = 0
    while d % 5 == 0:
        d //= 5
        a5 += 1
    return max(a2, a5)


def fmt_dec(q, digits=None):
    neg = q < 0
    a = -q if neg else q
    if digits is None:
        digits = dec_digits(a)
    scaled = a * 10 ** digits
    whole = int(scaled)  # exact for terminating with these digits
    ip, fp = divmod(whole, 10 ** digits)
    s = group3(ip)
    if digits:
        s += '.' + str(fp).rjust(digits, '0')
    return ('-' if neg and whole != 0 else '') + s


def sig15(q):
    """A 15-significant-digit display, rounded half to even (a reading aid)."""
    if q == 0:
        return '0'
    neg = q < 0
    a = abs(q)
    # find exponent e with 10^e <= a < 10^(e+1)
    e = dec_exp(a)
    scale = Fraction(10) ** (14 - e)
    x = a * scale
    n = x.numerator // x.denominator
    r = x - n
    if r > Fraction(1, 2) or (r == Fraction(1, 2) and n % 2 == 1):
        n += 1
    val = Fraction(n) / scale
    digits = max(0, 14 - e)
    s = fmt_dec(val, digits if digits else 0)
    if '.' in s:
        s = s.rstrip('0').rstrip('.')
    return ('-' if neg else '') + s


def fmt_q(q):
    """A number: the shortest exact decimal, or a reduced a/b (L §2.4, §5.18)."""
    q = Fraction(q)
    if terminating(q):
        s = fmt_dec(q)
        sig = len(str(abs(q.numerator)).rstrip('0')) if q.denominator == 1 else len(
            str(abs((q * 10 ** dec_digits(q)).numerator)).lstrip('0'))
        if sig > 15:
            return '%s (%s)' % (s, sig15(q))
        return s
    return '%d/%d (%s)' % (q.numerator, q.denominator, sig15(q))


def minor_digits(cur):
    m = CURRENCIES.get(cur)
    if m is None:
        return 2
    return dec_digits(m)


class MonthDay:
    """A month-day (L §3.1 `monthday`, GD-2, round 19): a day of the year without its year, `04-30`. 02-29 is none
    (not every year has it). Month-days compare with == and != only (L §8.4)."""
    __slots__ = ('m', 'd')

    def __init__(self, m, d):
        self.m, self.d = m, d

    def __eq__(self, o):
        return isinstance(o, MonthDay) and (self.m, self.d) == (o.m, o.d)

    def __ne__(self, o):
        return not self.__eq__(o)

    def __hash__(self):
        return hash(('monthday', self.m, self.d))

    def __repr__(self):
        return '%02d-%02d' % (self.m, self.d)

    def key(self):
        return ('monthday', self.m, self.d)


def monthday_ok(mm, dd):
    """L §3.1 monthday: a day every year has (02-29 is no month-day)."""
    try:
        datetime.date(2001, mm, dd)
        return True
    except ValueError:
        return False


def fmt(v):
    if isinstance(v, bool):
        return 'true' if v else 'false'
    if isinstance(v, Num) and v.enc is not None:
        b = bound_of(v)
        if b is not None and b == 0:
            return fmt(Num(v.q, v.u, v.money, v.isint, v.rate))
        base = fmt(Num(v.q, v.u, v.money, v.isint, v.rate))
        return '≈ %s ± %s' % (base, fmt_bound(b, v))
    if isinstance(v, Num):
        if v.money:
            cur = v.u[0][0]
            q = v.q
            m = CURRENCIES.get(cur)
            if m is not None and (q / m).denominator == 1:
                return '%s %s' % (fmt_dec(q, minor_digits(cur)), cur)
            return '%s %s' % (fmt_q(q), cur)
        if v.isint:
            return fmt_dec(v.q)
        if v.rate:
            p = U.period_of(v.u)
            return '%s /%s' % (fmt_q(v.q), p)
        if v.u:
            return '%s %s' % (fmt_q(v.q), U.fmt(v.u))
        return fmt_q(v.q)
    if isinstance(v, datetime.date):
        return v.isoformat()
    if isinstance(v, MonthDay):
        return repr(v)
    if isinstance(v, Period):
        return v.label()
    if isinstance(v, Key):
        return v.name
    if isinstance(v, Thing):
        return v.name
    if isinstance(v, Mv):
        return v.id
    if isinstance(v, Text):
        return '"%s"' % v.s.replace('\\', '\\\\').replace('"', '\\"')
    if isinstance(v, Code):
        return 'nominal %d' % v.n if v.word == 'nominal' else v.word
    if isinstance(v, Rec):
        return '(%s)' % ', '.join('%s: %s' % (a, fmt(b)) for a, b in v.fields)
    if isinstance(v, Lst):
        return '[%s]' % ', '.join(fmt(x) for x in v.items)
    if isinstance(v, Tup):
        return '(%s)' % ', '.join(fmt(x) for x in v.items)
    if isinstance(v, Special):
        return v.name
    if isinstance(v, Iv):
        if v.days is not None:
            return 'one_of([%s])' % ', '.join(d.isoformat() for d in sorted(v.days))
        return 'window(%s, %s)' % (fmt(v.lo), fmt(v.hi))
    if isinstance(v, Hole):
        return fmt_hole(v)
    if isinstance(v, Cal):
        parts = ['weekends [%s]' % ', '.join(v.weekends)]
        if v.yearly:
            s = 'yearly [%s]' % ', '.join('%02d-%02d' % md for md in v.yearly)
            if v.yfrom is not None:
                s += ' from %d' % v.yfrom
            parts.append(s)
        return 'calendar(%s)' % ', '.join(parts)
    if v is None:
        return '-'
    if type(v).__name__ == 'Dist':
        return v.text()
    return repr(v)


def bound_of(v):
    """The distance bound of a value from its true real value (L §7.3); None when unbounded."""
    lo, hi = v.enc
    if lo is None or hi is None:
        return None
    return max(v.q - lo, hi - v.q, Fraction(0))


def round_up_2sig(b):
    """b rounded up to two significant digits (L §7.3: the bound prints to two significant digits)."""
    if b <= 0:
        return Fraction(0)
    e = dec_exp(b)
    scale = Fraction(10) ** (1 - e)
    x = b * scale
    n = -((-x.numerator) // x.denominator)
    return Fraction(n) / scale


def fmt_bound(b, v):
    if b is None:
        return 'unbounded'
    r = round_up_2sig(b)
    s = fmt_dec(r)
    if v.money:
        return '%s %s' % (s, v.u[0][0])
    if v.rate:
        return '%s /%s' % (s, U.period_of(v.u))
    if v.u:
        return '%s %s' % (s, U.fmt(v.u))
    return s


def fmt_typed(v, t):
    """A value printed by its static type: a value of the unit % prints with it (L §2.4 Numbers, §3.2:
    `num %` differs from `num` only in how it prints, `19 %` against `0.19`)."""
    if t is not None and t.k == 'num' and not t.u and t.n == 'pct' and isinstance(v, Num) and not v.money \
            and not v.u and v.enc is None:
        return '%s %%' % fmt_q(v.q * 100)
    return fmt(v)


def fmt_hole(h):
    by = {}
    for text, origin in h.qs:
        by.setdefault(text, set()).add(origin)
    out = []
    for text in sorted(by):
        origins = sorted(o for o in by[text] if o)
        s = '?"%s"' % text.replace('\\', '\\\\').replace('"', '\\"')
        if origins:
            s += ' (%s)' % ', '.join(origins)
        out.append(s)
    return ' '.join(out) if out else '?'
