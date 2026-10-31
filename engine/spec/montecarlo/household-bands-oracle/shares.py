# Closed-form shares, means, the paired (CRN) statistics and the verdict figures. No paths are drawn.
from statistics import NormalDist
from fractions import Fraction as Fr
from math import sqrt, lgamma, log, exp, comb
Z = NormalDist(); q, C, s, N = 0.1, 3000, 400, 1000
def pE(m):  # P(E >= m + 0.01), E = dev + C*hit, dev a normal(0, s) correctly rounded to 0.01
    return (1 - q) * Z.cdf(-(m + 0.005) / s) + q * Z.cdf(-(m + 0.005 - C) / s)
def pmf(k, n, p): return exp(lgamma(n + 1) - lgamma(k + 1) - lgamma(n - k + 1) + k * log(p) + (n - k) * log(1 - p))
def fail(p, tol): return sum(pmf(k, N, p) for k in range(N + 1) if abs(k / N - p) > tol)
low = [1800] * 10 + [3197.33, 9013.95]   # the 2027 dips: ten months at 1800, Dec 29 3197.33, Sep 9 9013.95
none = 1.0
for m in low: none *= 1 - pE(m)
for name, p in [('below_zero (some day of 2027)', 1 - none), ('below_two_rents (2027-12-31 < 6400)', pE(256.75)),
                ('paired: diff = 0, P(K12 = 0)', float(Fr(9, 10) ** 12))]:
    se = sqrt(p * (1 - p) / N); tol = 4 * se + 1e-9
    print(f'{name:36s} p={p:.6f} n~{p * N:6.1f} SE={se:.6f} tol={tol:.6f} accept n in [{int(N * (p - tol)) + 1}, {int(N * (p + tol))}] Pfail={fail(p, tol):.2e}')
print('per-month P(E > 1800)', round(pE(1800), 6), 'Dec', round(pE(3197.33), 6), 'Aug', pE(9013.95), 'P(konto 2027-12-31 < 0)', pE(6656.75), 'clip Aug', pE(4409.93), 'E[dev+]*12', round(12 * s * Z.pdf(0), 2))
print('Fr(9,10)**12 =', Fr(9, 10) ** 12, '; P(K12<=1) =', float(Fr(9, 10) ** 12 + 12 * Fr(1, 10) * Fr(9, 10) ** 11))
sd = sqrt(12 * s**2 + 12 * q * (1 - q) * C**2)
print('mean net worth 2027-12-31', 319952.22 - 12 * q * C, 'SD', round(sd, 2), 'SE', round(sd / sqrt(N), 2), 'tol', round(4 * sd / sqrt(N), 2))
sdp = C * sqrt(12 * q * (1 - q))
print('paired mean', -12 * q * C, 'SD', round(sdp, 2), 'SE', round(sdp / sqrt(N), 2), 'tol', round(4 * sdp / sqrt(N), 2))
pK2, pK1 = 1 - float(Fr(9, 10) ** 12 + 12 * Fr(1, 10) * Fr(9, 10) ** 11), 1 - float(Fr(9, 10) ** 12)
print('paired median = -3000 fails with P =', sum(pmf(k, N, pK2) for k in range(500, N + 1)) + sum(pmf(k, N, pK1) for k in range(0, 500)))
for n_ in (51, 52, 1000):
    pt = Fr(2, n_ + 4); v = 1600 * pt * (1 - pt)
    print(f'0 of {n_}: 1600*pt*(1-pt) = {float(v):.4f} vs N+4 = {n_ + 4} -> {"pass" if v <= n_ + 4 else "fail"}; 2*se_v = {2 * sqrt(pt * (1 - pt) / (n_ + 4)):.6f}; P(n = 0) = 1 - {n_ * pE(6656.75):.1e}')
