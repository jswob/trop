# Closed-form fans: Y = B - S_n, S_n = (n normal(0, s) devs) + C * Binomial(n, q). No paths are drawn.
from statistics import NormalDist
from math import comb, sqrt, lgamma, log, exp, ceil
Z = NormalDist(); q, C, s, N = 0.1, 3000.0, 400.0, 1000
def w(n): return [(comb(n, k) * q**k * (1 - q)**(n - k), C * k) for k in range(n + 1)]
def F(x, n): return sum(a * Z.cdf((x - m) / (s * sqrt(n))) for a, m in w(n))
def f(x, n): return sum(a * Z.pdf((x - m) / (s * sqrt(n))) for a, m in w(n)) / (s * sqrt(n))
def Q(b, n):
    lo, hi = -30000.0, 60000.0
    for _ in range(120):
        mid = (lo + hi) / 2
        lo, hi = (mid, hi) if F(mid, n) < b else (lo, mid)
    return hi
def ge(p, r):  # P(Binomial(N, p) >= r)
    if p <= 0: return 0.0
    if p >= 1: return 1.0
    return sum(exp(lgamma(N + 1) - lgamma(k + 1) - lgamma(N - k + 1) + k * log(p) + (N - k) * log(1 - p)) for k in range(r, N + 1))
fans = [('konto 2027-06-30', 6760.19, 1), ('konto 2027-12-31', 6656.75, 1), ('room 2027-08', 4409.93, 1),
        ('room 2027-11', 13938.05, 1), ('net worth 2027-06-30', 250001.40, 6), ('net worth 2027-12-31', 319952.22, 12)]
tot = 0.0
for name, B, n in fans:
    b, e = 0.005 * n, 0.01
    for a in (0.05, 0.25, 0.5, 0.75, 0.95):
        x = Q(1 - a, n); se = sqrt(a * (1 - a) / N) / f(x, n); tol = ceil((4 * se + b + e) * 100) / 100; r = round(a * N)
        fail = ge(1 - F(x + tol - b, n), r) + 1 - ge(1 - F(x - tol + b, n), r)
        tot += fail
        print(f'{name:21s} q{a:.2f} Q={B - x:10.2f} S-quantile={x:9.2f} f={f(x, n):.3e} SE={se:7.2f} tol={tol:7.2f} Pfail={fail:.2e}')
print(f'sum of Pfail over the 30 fan statistics: {tot:.2e}')
