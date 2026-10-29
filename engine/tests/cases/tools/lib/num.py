"""Exact numbers for the keys: Decimal money, rounding by named steps, splits, conversion.

Every rounding is a named call, made where the rule rounds and nowhere else.
"""
from decimal import Decimal, ROUND_DOWN, ROUND_HALF_EVEN, ROUND_HALF_UP, getcontext
from fractions import Fraction

getcontext().prec = 60

CENT = Decimal("0.01")
UNIT = Decimal("1")


class Refuse(Exception):
    """No value: the key names what's missing, or why it can't be computed."""

    def __init__(self, missing, reason=""):
        self.missing = list(missing)
        self.reason = reason
        super().__init__(reason or "; ".join(self.missing))


def D(x):
    """A Decimal from a Decimal, int, str or Fraction (exact where the fraction terminates)."""
    if isinstance(x, Decimal):
        return x
    if isinstance(x, Fraction):
        return Decimal(x.numerator) / Decimal(x.denominator)
    if isinstance(x, float):
        raise TypeError("floats are not money: pass a str or a Decimal")
    return Decimal(str(x))


def F(x):
    """A Fraction from a Fraction, Decimal, int or a str like '3/16'."""
    if isinstance(x, Fraction):
        return x
    if isinstance(x, Decimal):
        return Fraction(x)
    return Fraction(str(x))


def half_up(x, step=CENT):
    """Round half away from zero to a multiple of `step`."""
    return D(x).quantize(D(step), rounding=ROUND_HALF_UP)


def down(x, step=CENT):
    """Truncate toward zero to a multiple of `step`."""
    return D(x).quantize(D(step), rounding=ROUND_DOWN)


def half_even(x, step=CENT):
    """Round half to even to a multiple of `step`."""
    return D(x).quantize(D(step), rounding=ROUND_HALF_EVEN)


def add(*items):
    """Sum (amount, currency) pairs; two currencies with no stated rate are refused."""
    target = items[0][1]
    others = sorted({c for _, c in items if c != target})
    if others:
        raise Refuse([f"rate {c}->{target}" for c in others], "amounts in two currencies with no stated rate")
    return sum((D(a) for a, _ in items), Decimal(0)), target


def split(total, weights, step=CENT, remainder="order", order=None):
    """Split `total` by `weights` into multiples of `step` that add back exactly.

    Each part is first floored to `step`; the steps left over go one each either in the
    listing order (`remainder="order"`, or `order` given as indices) or to the parts with the
    largest dropped remainder (`remainder="largest"`, ties in listing order).
    """
    total, step = D(total), D(step)
    units = int(total / step)
    if D(units) * step != total:
        raise ValueError("total is not a multiple of step")
    ws = [F(w) for w in weights]
    raw = [Fraction(units) * w / sum(ws) for w in ws]
    base = [r.numerator // r.denominator for r in raw]
    left = units - sum(base)
    if remainder == "largest":
        idx = sorted(range(len(ws)), key=lambda i: (-(raw[i] - base[i]), i))
    else:
        idx = order or list(range(len(ws)))
    for i in idx[:left]:
        base[i] += 1
    return [step * b for b in base]


def annuity(principal, rate, n, step=CENT):
    """Level instalment repaying `principal` over `n` periods at `rate` per period, rounded half up."""
    p, r = D(principal), D(rate)
    if r == 0:
        return half_up(p / n, step)
    return half_up(p * r / (1 - (1 + r) ** -n), step)


def chain(amount, factors, step=None):
    """Multiply by each factor in turn, rounding after each when `step` is given."""
    x = D(amount)
    for f in factors:
        x = x * D(f)
        if step is not None:
            x = half_up(x, step)
    return x


def convert(amount, rate, step=CENT):
    """One converted transfer: amount × rate, a whole number of minor units (half up)."""
    return half_up(D(amount) * D(rate), step)
