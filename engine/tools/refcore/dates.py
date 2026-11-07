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


def past_days(after=True):
    """L §7.6 item 1 (tally 2.1.6, issue #89): a day computed outside 0001-01-01..9999-12-31 is refused, RUN-LIMIT at
    the key (load turns it into LOAD-LIMIT where it builds an index's keys or sees a move's day)."""
    return Refusal('RUN-LIMIT', 'a day passes 9999-12-31' if after else 'a day comes before 0001-01-01')


def check_year(y):
    if not 1 <= y <= 9999:
        raise past_days(y > 9999)


def add_months(d, n):
    """The same day number, clamped to the last day of the target month (L §4.2)."""
    if n == 0:
        return d
    # the end of time moves as the day 10000-01-01 would (L §2.1)
    yy, mm, dd = (10000, 1, 1) if d is END else (d.year, d.month, d.day)
    y, m = month_add(yy, mm, n)
    check_year(y)
    return D(y, m, min(dd, last_day(y, m)))


def add_days(d, n):
    try:
        return d + datetime.timedelta(days=n)
    except OverflowError:
        raise past_days(n > 0)


class EndOfTime(D):
    """The end of time (L §2.1, tally 2.1.6, issue #48): the start of the period after one that ends with 9999-12-31,
    Tally's last day, as the day 10000-01-01 would be. It follows every day, a span or a period may end there, and
    it is compared, and has days or months taken from it, as a day is (`END - ONE_DAY` is 9999-12-31,
    `add_months(END, -1)` 9999-12-01); it is no day itself, so a day after it, its calendar fields and its print are
    refused (RUN-LIMIT, L §7.6).
    A subclass of date, so Python tries its own comparisons and subtraction first; its stored day is 9999-12-31,
    which no method reads. refcore 2.1.5 built the year 9999's end as date(10000, 1, 1), which Python refuses."""
    __slots__ = ()
    _one = None

    def __new__(cls, *args):
        if cls._one is None:
            cls._one = D.__new__(cls, 9999, 12, 31)
        return cls._one

    @property
    def year(self):
        raise past_days()

    month = day = year

    def __eq__(self, o):
        return o is self if isinstance(o, D) else NotImplemented

    def __ne__(self, o):
        return o is not self if isinstance(o, D) else NotImplemented

    def __lt__(self, o):
        return False if isinstance(o, D) else NotImplemented

    def __le__(self, o):
        return o is self if isinstance(o, D) else NotImplemented

    def __gt__(self, o):
        return o is not self if isinstance(o, D) else NotImplemented

    def __ge__(self, o):
        return True if isinstance(o, D) else NotImplemented

    def __hash__(self):
        return hash('EndOfTime')

    def __add__(self, o):
        if not isinstance(o, datetime.timedelta):
            return NotImplemented
        if o.days == 0:
            return self
        if o.days < 0:
            return D.max + (o + ONE_DAY)
        raise past_days()

    __radd__ = __add__

    def __sub__(self, o):
        if isinstance(o, datetime.timedelta):
            return self + (-o)
        if o is self:
            return datetime.timedelta(0)
        if isinstance(o, D):
            return (D.max - o) + ONE_DAY
        return NotImplemented

    def __rsub__(self, o):
        if isinstance(o, D):
            return -(self - o)
        return NotImplemented

    def toordinal(self):
        return D.max.toordinal() + 1

    def no_day(self, *a):
        raise past_days()

    isoformat = __str__ = __format__ = weekday = isoweekday = isocalendar = replace = strftime = timetuple = no_day

    def __repr__(self):
        return 'END'

    def __reduce__(self):
        return (EndOfTime, ())


END = EndOfTime()


def next_day(d):
    """The day after d, or the end of time after 9999-12-31 (L §2.1)."""
    return END if d == D.max else add_days(d, 1)


def end_days(s, n):
    """The end of a period of n days from s: the day after its last, or the end of time (L §2.1); a period whose last
    day passes 9999-12-31 is refused (L §7.6)."""
    return next_day(add_days(s, n - 1))


def end_months(s, n):
    """The end of a period of n months from s, the end of time where it ends with 9999-12-31 (L §2.1); a period whose
    last day passes 9999-12-31 is refused (L §7.6)."""
    y, m = month_add(s.year, s.month, n)
    if (y, m) == (10000, 1) and s.day == 1:
        return END
    return add_months(s, n)


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
            return Period(self.kind, s, end_days(s, self.n), 'day', self.n)
        s = add_months(self.start, k * self.n)
        e = end_months(self.start, (k + 1) * self.n)
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
    return Period('day', d, next_day(d), 'day', 1)


def month_period(y, m):
    check_year(y)
    s = D(y, m, 1)
    return Period('month', s, end_months(s, 1), 'month', 1)


def quarter_period(y, q):
    check_year(y)
    s = D(y, 3 * (q - 1) + 1, 1)
    return Period('quarter', s, end_months(s, 3), 'month', 3)


def year_period(y):
    check_year(y)
    s = D(y, 1, 1)
    return Period('year', s, end_months(s, 12), 'month', 12)


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
