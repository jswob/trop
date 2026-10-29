"""Household D's long view (D-HR3): Halina's money year by year from 2027 to a random death.

Everything the law or a contract indexes grows by the year's CPI assumption (3 %; D-11 overrides it):
PEN-IDX on 1 March of year y reads the CPI of y-1, so does MIN-PENSION from March, the rent (D-D25:
CPI-AVG of y-1) and the building fee from January; spending in year y is in year-y prices. The tax law
stays as in force in 2026 (PIT-3 v3, PIT-13, SOC-2 v2). The SIO pays the SOC-7 v2 part as the law says.
The fund's gains are not taxed (an assumption of the register). Flows are booked at the year's end.

A path draws, from one seeded generator, her whole years still lived (the life table, ages 68..105) and
then one fund return a year: mean 5 %, standard deviation 16 %, Student-t with 4 degrees of freedom.
"""
import math
import random
from datetime import date
from decimal import Decimal
from fractions import Fraction

from fam.home import ZERO, D, PIT3, duk, ort, scale
from lib.num import F
from lib.paths import curtate_expectation, draw_curtate

from fam import d as FD

START_YEAR = 2027
START_ACCOUNT = D("10000.00")
START_FUND = D("45000.00")
CARE_2026 = D("36000.00")        # a year, in 2026 money, from the year she turns 80
CARE_FROM_AGE = 80
QS = FD.qs()                      # q for the year she turns 68 (2027) .. 105 (2064)
YEARS = len(QS)                   # 38: 2027 .. 2064


def cpi_path(over=None):
    """The year's CPI factor for 2026 .. 2065: 1.03, or the override's."""
    c = {y: 1 + FD.INFL for y in range(2026, 2066)}
    for y, v in (over or {}).items():
        c[y] = D(v)
    return c


def flows(c, *, known):
    """The deterministic year flows 2027 .. 2064 under CPI path c: pensions, tax, rent, spending."""
    surv, own, minp = FD.surv(date(2026, 3, 1), known=known), FD.own(date(2026, 3, 1), known=known), FD.min_pension(date(2026, 3, 1), known=known)
    rent, fee = FD.rent_on(date(2026, 3, 1), known=known), FD.FEES[2026]
    spend, care = FD.SPEND_2026, CARE_2026
    out = []
    for y in range(START_YEAR, START_YEAR + YEARS):
        fee = ort(fee * c[y - 1])
        spend = spend * c[y]
        care = care * c[y]
        gross = health = rents = fees = ZERO
        for mo in range(1, 13):
            if mo == 3:
                surv, own, minp = ort(surv * c[y - 1]), ort(own * c[y - 1]), ort(minp * c[y - 1])
                rent = ort(rent * c[y - 1])
            part = max(ZERO, min(ort(D("0.25") * own), 3 * minp - surv))
            g = surv + part
            gross += g
            health += ort(g * D("0.09"))
            rents += rent
            fees += fee
        tax = duk(max(ZERO, scale(duk(gross), "v3") - PIT3["v3"]["reducing"]))
        rent_tax = duk(rents * 13 / 16 * D("0.085"))
        rent_net = ort((rents - fees) * 13 / 16) - rent_tax
        age = y - FD.HALINA_BORN.year
        sp = ort(spend + (care if age >= CARE_FROM_AGE else ZERO))
        out.append({"year": y, "gross": gross, "health": health, "tax": tax, "rent": rents, "rent net": rent_net,
                    "spending": sp, "flow": gross - health - tax + rent_net - sp, "surv_mar": surv, "rent_mar": rent})
    return out


def t_draw(r):
    z = r.gauss(0.0, 1.0)
    v = sum(r.gauss(0.0, 1.0) ** 2 for _ in range(FD.FUND_DF))
    return z / math.sqrt(v / FD.FUND_DF)


def ret(r):
    t = t_draw(r) / math.sqrt(FD.FUND_DF / (FD.FUND_DF - 2))
    return max(-0.95, float(FD.FUND_MU) + float(FD.FUND_S) * t)


def run(fl, k, rets):
    """One path: k whole years lived (years 0..k-1), fund returns rets[j]. Returns the yearly states."""
    a, f = START_ACCOUNT, START_FUND
    dry, tax, ends = None, ZERO, {}
    for j in range(min(k, YEARS)):
        y = fl[j]
        f = ort(f * (1 + Decimal(f"{rets[j]:.6f}")))
        a = a + y["flow"]
        tax += y["tax"]
        if a > FD.SWEEP_ABOVE:
            f, a = f + (a - FD.SWEEP_ABOVE), FD.SWEEP_ABOVE
        elif a < 0:
            sold = min(f, -a)
            f, a = f - sold, a + sold
            if a < 0 and dry is None:
                dry = y["year"]
        ends[y["year"]] = a + f
    return {"k": k, "dry": dry, "tax": tax, "ends": ends}


def path(i, fl, zero=False):
    r = random.Random(FD.SEED + i)
    k = draw_curtate(r, QS)
    rets = [float(FD.FUND_MU) if zero else ret(r) for _ in range(YEARS)]
    return run(fl, k, rets)


def survival():
    """P(she lives through year j), j = 0 .. YEARS-1, exact."""
    alive, out = Fraction(1), []
    for q in QS:
        alive *= 1 - F(q)
        out.append(alive)
    return out


def closed(fl):
    """The closed forms: expected whole years, expected tax to death; the zero-volatility dry year and its share."""
    s = survival()
    e_k = curtate_expectation(QS)
    e_tax = sum((s[j] * F(fl[j]["tax"]) for j in range(YEARS)), Fraction(0))
    z = run(fl, YEARS, [float(FD.FUND_MU)] * YEARS)
    share = s[z["dry"] - START_YEAR] if z["dry"] else Fraction(0)
    return {"e_k": e_k, "e_tax": e_tax, "dry_zero": z["dry"], "share_zero": share}
