"""Days and periods (L §4.1, §4.2).

A day is a datetime.date.  A period is a Period: a kind (day, month,
quarter, year), its first day, the first day after it, and the step its
index advances by (for `by n` and anchored ranges, L §4.1).
"""
import calendar as _cal
import datetime

from .errors import Refusal, Unspecified

D = datetime.date
ONE_DAY = datetime.timedelta(days=1)


def month_add(y, m, n):
    k = y * 12 + (m - 1) + n
    return k // 12, k % 12 + 1


def last_day(y, m):
    return _cal.monthrange(y, m)[1]


def add_months(d, n):
    """The same day number, clamped to the last day of the target month (L §4.2)."""
    y, m = month_add(d.year, d.month, n)
    return D(y, m, min(d.day, last_day(y, m)))


def add_days(d, n):
    return d + datetime.timedelta(days=n)


class Period:
    __slots__ = ('kind', 'start', 'end', 'unit', 'n')

    def __init__(self, kind, start, end, unit, n):
        self.kind = kind      # 'day' 'month' 'quarter' 'year'
        self.start = start
        self.end = end        # first day after it
        self.unit = unit      # 'day' or 'month': what the step counts
        self.n = n            # step length in units

    def key(self):
        return (self.kind, self.start, self.end)

    def __eq__(self, o):
        return isinstance(o, Period) and self.key() == o.key()

    def __hash__(self):
        return hash(self.key())

    def __lt__(self, o):
        return self.start < o.start

    def shift(self, k):
        """The period k steps later (key arithmetic on its index)."""
        if self.unit == 'day':
            s = add_days(self.start, k * self.n)
            return Period(self.kind, s, add_days(s, self.n), 'day', self.n)
        s = add_months(self.start, k * self.n)
        e = add_months(self.start, (k + 1) * self.n)
        return Period(self.kind, s, e, 'month', self.n)

    def steps_to(self, o):
        """How many steps from self to o (o - self), or None if not on the grid."""
        if self.unit == 'day':
            dd = (o.start - self.start).days
            if dd % self.n:
                return None
            return dd // self.n
        dm = (o.start.year - self.start.year) * 12 + (o.start.month - self.start.month)
        if dm % self.n or o.start.day != self.start.day:
            return None
        return dm // self.n

    def contains(self, d):
        return self.start <= d < self.end

    def label(self):
        if self.kind == 'day':
            return self.start.isoformat()
        if self.kind == 'month':
            return '%04d-%02d' % (self.start.year, self.start.month)
        if self.kind == 'year':
            return '%04d' % self.start.year
        if self.kind == 'quarter':
            # L §2.1: "A quarter ... is 2026-Q1 to 2026-Q4"; L §2.4: "a quarter as 2026-Q1 (§2.1)"
            return '%04d-Q%d' % (self.start.year, (self.start.month - 1) // 3 + 1)
        raise AssertionError(self.kind)

    def __repr__(self):
        return 'Period(%s %s..%s)' % (self.kind, self.start, self.end)


def day_period(d):
    return Period('day', d, d + ONE_DAY, 'day', 1)


def month_period(y, m):
    s = D(y, m, 1)
    return Period('month', s, add_months(s, 1), 'month', 1)


def quarter_period(y, q):
    s = D(y, 3 * (q - 1) + 1, 1)
    return Period('quarter', s, add_months(s, 3), 'month', 3)


def year_period(y):
    s = D(y, 1, 1)
    return Period('year', s, D(y + 1, 1, 1), 'month', 12)


def month_of(d):
    return month_period(d.year, d.month)


def quarter_of(d):
    return quarter_period(d.year, (d.month - 1) // 3 + 1)


def year_of(d):
    return year_period(d.year)


def age(b, d):
    """Completed years from b to d; the anniversary of 29 February in a common
    year is 1 March; before b it is none (None here)."""
    if d < b:
        return None
    n = d.year - b.year
    if b.month == 2 and b.day == 29 and not _cal.isleap(d.year):
        ann = D(d.year, 3, 1)
    else:
        ann = D(d.year, b.month, b.day)
    if d < ann:
        n -= 1
    return n


def age_months(b, d):
    """The largest n with add_months(b, n) <= d; None before b."""
    if d < b:
        return None
    n = (d.year - b.year) * 12 + (d.month - b.month)
    while n > 0 and add_months(b, n) > d:
        n -= 1
    while add_months(b, n + 1) <= d:
        n += 1
    return n


def overlap_days(a, b, k):
    """Days x with a <= x <= b that lie in period k."""
    lo = max(a, k.start)
    hi = min(b, k.end - ONE_DAY)
    if hi < lo:
        return 0
    return (hi - lo).days + 1


WEEKDAYS = {'mon': 1, 'tue': 2, 'wed': 3, 'thu': 4, 'fri': 5, 'sat': 6, 'sun': 7}
