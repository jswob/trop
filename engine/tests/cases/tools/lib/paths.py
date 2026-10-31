"""Uncertainty: seeded paths, life tables and their closed forms, standard errors."""
import math
import random
from decimal import Decimal
from fractions import Fraction
from math import comb

from .num import F


def rng(seed):
    """The one source of randomness: a stated seed."""
    return random.Random(seed)


def curtate_expectation(qs):
    """Expected whole years still lived, from death probabilities year by year (the last is 1)."""
    alive, e = Fraction(1), Fraction(0)
    for q in qs:
        alive *= 1 - F(q)
        e += alive
    return e


def draw_curtate(r, qs):
    """One drawn lifetime: whole years lived before the year of death."""
    for k, q in enumerate(qs):
        if r.random() < float(F(q)):
            return k
    return len(qs)


def mean_se(xs):
    """Sample mean and its standard error."""
    n = len(xs)
    m = sum(xs) / n
    v = sum((x - m) ** 2 for x in xs) / (n - 1)
    return m, math.sqrt(v / n)


def binom_tail(n, p, kmin):
    """P(K >= kmin) for K ~ Binomial(n, p), exact."""
    p = F(p)
    return sum(Fraction(comb(n, k)) * p ** k * (1 - p) ** (n - k) for k in range(kmin, n + 1))


def dec(x, places=6):
    """A float statistic as a Decimal with fixed places (for keys)."""
    return Decimal(f"{x:.{places}f}")
