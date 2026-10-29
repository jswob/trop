"""Dates: month arithmetic with month-end wording, day counts, business-day calendars."""
import calendar as _cal
from datetime import date, timedelta


def P(s):
    """A date from an ISO string (or a date)."""
    return s if isinstance(s, date) else date.fromisoformat(s)


def end_of_month(d):
    d = P(d)
    return date(d.year, d.month, _cal.monthrange(d.year, d.month)[1])


def add_months(d, n):
    """The same day `n` months on; where that month lacks the day, its last day."""
    d = P(d)
    y, m = divmod(d.month - 1 + n, 12)
    y, m = d.year + y, m + 1
    return date(y, m, min(d.day, _cal.monthrange(y, m)[1]))


def days(a, b):
    """Days from a to b (b − a)."""
    return (P(b) - P(a)).days


def month_key(d):
    d = P(d)
    return f"{d.year:04d}-{d.month:02d}"


def months_back(ym, n):
    """'YYYY-MM' n months before ym."""
    y, m = map(int, ym.split("-"))
    k = y * 12 + (m - 1) - n
    return f"{k // 12:04d}-{k % 12 + 1:02d}"


class Calendar:
    """Weekends plus listed holidays: {"md": "06-22", "from": 2026, "to": null} or {"date": "YYYY-MM-DD"}."""

    def __init__(self, holidays=()):
        self.holidays = list(holidays)

    def is_holiday(self, d):
        d = P(d)
        for h in self.holidays:
            if "date" in h and P(h["date"]) == d:
                return True
            if "md" in h and d.strftime("%m-%d") == h["md"]:
                if h.get("from") is not None and d.year < h["from"]:
                    continue
                if h.get("to") is not None and d.year > h["to"]:
                    continue
                return True
        return False

    def is_business(self, d):
        d = P(d)
        return d.weekday() < 5 and not self.is_holiday(d)

    def on_or_next(self, d):
        d = P(d)
        while not self.is_business(d):
            d += timedelta(days=1)
        return d

    def before(self, d):
        """The last business day strictly before d."""
        d = P(d) - timedelta(days=1)
        while not self.is_business(d):
            d -= timedelta(days=1)
        return d

    def nth_last(self, year, month, n):
        """The n-th last business day of a month (1 = the last)."""
        d = date(year, month, _cal.monthrange(year, month)[1])
        seen = 0
        while True:
            if self.is_business(d):
                seen += 1
                if seen == n:
                    return d
            d -= timedelta(days=1)
