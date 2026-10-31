"""Writes law/gazette.csv: the figures the home law leaves to a notice (law/acts.md, GEN-6).

Hand-set figures are listed below, each with the rule that reads it. Long series (the Central Bank's
daily FLR mid rate, the monthly CPI, the pension-capital indices, the life-expectancy table, the
savings-bond series) are generated from stated anchors and a seed. The file is the fact; this
script only wrote it. Run: python3 -I tmp/cases/tools/gen_gazette.py
"""
import csv
import math
import os
import sys
from datetime import date, timedelta
from decimal import ROUND_HALF_UP, Decimal

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from lib.dates import Calendar, end_of_month, month_key  # noqa: E402
from lib.paths import rng  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "law", "gazette.csv")
SEED = 20261024
LAST = date(2026, 9, 30)  # the recorded past ends here

# GEN-1: Spring Day, by notice per year (published by 31 October of the year before).
SPRING = {2018: "2018-04-02", 2019: "2019-04-22", 2020: "2020-04-13", 2021: "2021-04-05", 2022: "2022-04-18",
          2023: "2023-04-10", 2024: "2024-04-01", 2025: "2025-04-21", 2026: "2026-04-06"}
SPRING_PUB = {2018: "2017-10-12", 2019: "2018-10-11", 2020: "2019-10-10", 2021: "2020-10-15", 2022: "2021-10-14",
              2023: "2022-10-13", 2024: "2023-10-12", 2025: "2024-10-10", 2026: "2025-10-16"}
FIXED = ["01-01", "01-06", "05-01", "05-03", "08-15", "11-01", "11-11", "12-25", "12-26"]  # GEN-1 v1 .2


def calendar():
    hol = [{"md": md, "from": 2018, "to": None} for md in FIXED]
    hol.append({"md": "06-23", "from": 2025, "to": None})  # GEN-1 v2 .2
    hol += [{"date": d} for d in SPRING.values()]
    return Calendar(hol)


CAL = calendar()


def q(x, places):
    return Decimal(repr(x)).quantize(Decimal(1).scaleb(-places), rounding=ROUND_HALF_UP)


def interp(anchors, d):
    """Linear interpolation between dated anchors [(date, value), ...]."""
    if d <= anchors[0][0]:
        return anchors[0][1]
    for (d0, v0), (d1, v1) in zip(anchors, anchors[1:]):
        if d <= d1:
            t = (d - d0).days / (d1 - d0).days
            return v0 + (v1 - v0) * t
    return anchors[-1][1]


def A(*pairs):
    return [(date.fromisoformat(d), v) for d, v in pairs]


rows = []


def row(id_, value, unit, published, from_, to, rule, note=""):
    rows.append([id_, str(value), unit, str(published), str(from_), "" if to is None else str(to), rule, note])


# --- GEN-1: Spring Day notices
for y, d in SPRING.items():
    row(f"HOL-SPRING-{y}", d, "date", SPRING_PUB[y], f"{y}-01-01", f"{y}-12-31", "GEN-1", "Spring Day of the year")

# --- Wages and limits (spans)
MIN_WAGE = [  # SOC-3.2, SOC-4.1, WCP-1.2, WCP-3.2
    ("2080.00", "2017-09-12", "2018-01-01", "2018-12-31"), ("2230.00", "2018-09-11", "2019-01-01", "2019-12-31"),
    ("2580.00", "2019-09-10", "2020-01-01", "2020-12-31"), ("2790.00", "2020-09-15", "2021-01-01", "2021-12-31"),
    ("3020.00", "2021-09-14", "2022-01-01", "2022-12-31"), ("3450.00", "2022-09-13", "2023-01-01", "2023-06-30"),
    ("3580.00", "2022-09-13", "2023-07-01", "2023-12-31"), ("4180.00", "2023-09-12", "2024-01-01", "2024-06-30"),
    ("4260.00", "2023-09-12", "2024-07-01", "2024-12-31"), ("4620.00", "2024-09-10", "2025-01-01", "2025-12-31"),
    ("4780.00", "2025-09-16", "2026-01-01", "2026-12-31")]
for v, p, f, t in MIN_WAGE:
    row("MIN-WAGE", v, "DUK/month", p, f, t, "SOC-3", "minimum monthly wage")

PROJ = {2018: "4400.00", 2019: "4750.00", 2020: "5180.00", 2021: "5400.00", 2022: "5950.00", 2023: "6900.00",
        2024: "7800.00", 2025: "8600.00", 2026: "9300.00"}  # SOC-3.3
CAP = {2018: "132000.00", 2019: "142500.00", 2020: "155400.00", 2021: "162000.00", 2022: "178500.00",
       2023: "207000.00", 2024: "234000.00", 2025: "258000.00", 2026: "279000.00"}  # SOC-1.2
for y in PROJ:
    pub = f"{y - 1}-12-{18 if y % 2 else 19}"
    row("AVG-WAGE-PROJ", PROJ[y], "DUK/month", pub, f"{y}-01-01", f"{y}-12-31", "SOC-3", "projected average monthly wage")
    row("SOC-CAP", CAP[y], "DUK/year", pub, f"{y}-01-01", f"{y}-12-31", "SOC-1", "yearly cap on the pension and disability base")

Q4 = [  # SOC-4.3: the previous year's fourth-quarter average wage, in force February to January
    ("5480.30", "2021-01-21", "2021-02-01", "2022-01-31", "Q4 2020"), ("6050.20", "2022-01-20", "2022-02-01", "2023-01-31", "Q4 2021"),
    ("6920.75", "2023-01-19", "2023-02-01", "2024-01-31", "Q4 2022"), ("7810.40", "2024-01-22", "2024-02-01", "2025-01-31", "Q4 2023"),
    ("8530.10", "2025-01-21", "2025-02-01", "2026-01-31", "Q4 2024"), ("9180.65", "2026-01-20", "2026-02-01", "2027-01-31", "Q4 2025")]
for v, p, f, t, n in Q4:
    row("AVG-WAGE-Q4", v, "DUK/month", p, f, t, "SOC-4", f"average wage in {n}")

WRAP = {2018: "13200.00", 2019: "14100.00", 2020: "14950.00", 2021: "15300.00", 2022: "16500.00", 2023: "19800.00",
        2024: "22400.00", 2025: "25100.00", 2026: "27300.00"}  # PIT-9.2
for y, v in WRAP.items():
    row("WRAP-LIMIT", v, "DUK/year", f"{y - 1}-11-{20 + y % 5}", f"{y}-01-01", f"{y}-12-31", "PIT-9", "yearly contribution limit")

for g, v1, v2 in (("I", "40000.00", "44000.00"), ("II", "30000.00", "33000.00"), ("III", "6000.00", "6600.00")):  # IGT-3.2
    row(f"IGT-FREE-{g}", v1, "DUK", "2017-10-02", "2018-01-01", "2023-12-31", "IGT-3", f"free amount, group {g}")
    row(f"IGT-FREE-{g}", v2, "DUK", "2023-10-02", "2024-01-01", None, "IGT-3", f"free amount, group {g}")

# --- Pensions: minimum, yearly indexation (SOC-5.6, SOC-6, SOC-7 v2 .3)
MINP = {2018: "1029.80", 2019: "1100.00", 2020: "1200.00", 2021: "1250.88", 2022: "1338.44", 2023: "1588.44",
        2024: "1780.96", 2025: "1878.91", 2026: "1970.00"}
for y, v in MINP.items():
    row("MIN-PENSION", v, "DUK/month", f"{y}-02-{14 + y % 3}", f"{y}-03-01", f"{y + 1}-02-{29 if (y + 1) % 4 == 0 else 28}", "SOC-5", "minimum pension")
PIDX = {2018: "1.0298", 2019: "1.0331", 2020: "1.0352", 2021: "1.0419", 2022: "1.0712", 2023: "1.1476", 2024: "1.1208",
        2025: "1.0553", 2026: "1.0491"}
for y, v in PIDX.items():
    p = f"{y}-02-{14 + y % 3}"
    row(f"PEN-IDX-{y}", v, "ratio", p, p, None, "SOC-6", f"indexation of 1 March {y}")

# --- Pension capital valorisation indices, years 1981-2025 (SOC-5.3), published in May of the next year
r = rng(SEED + 1)
for y in range(1981, 2026):
    base = 0.115 if y < 1991 else 0.095 if y < 2001 else 0.052 if y < 2020 else 0.085
    v = q(1 + base + r.uniform(-0.018, 0.018), 4)
    p = f"{y + 1}-05-{20 + (y % 7)}"
    row(f"VAL-IDX-{y}", v, "ratio", p, p, None, "SOC-5", f"valorisation index for {y}")

# --- Life expectancy at ages 58-67, months, one table a year in force 1 April - 31 March (SOC-5.4)
LE60 = {2018: 259.4, 2019: 261.0, 2020: 262.8, 2021: 259.1, 2022: 250.3, 2023: 254.0, 2024: 258.6, 2025: 260.2, 2026: 261.5}
for t, b in LE60.items():
    for age in range(58, 68):
        v = q(b - 11.4 * (age - 60) + 0.15 * (age - 60) ** 2, 1)
        row(f"LE-{age}", v, "months", f"{t}-03-{24 + t % 4}", f"{t}-04-01", f"{t + 1}-03-31", "SOC-5", f"table {t}, age {age}")

# --- CPI, year on year (same month a year before = 100), monthly 2018-01 .. 2026-08 (TSB-2.2)
CPI = A(("2018-01-15", 101.9), ("2018-06-15", 102.0), ("2018-12-15", 101.1), ("2019-06-15", 102.4), ("2019-12-15", 103.2),
        ("2020-02-15", 104.5), ("2020-06-15", 103.2), ("2020-12-15", 102.5), ("2021-06-15", 104.4), ("2021-12-15", 108.4),
        ("2022-03-15", 110.9), ("2022-06-15", 115.1), ("2022-10-15", 117.7), ("2023-02-15", 118.2), ("2023-06-15", 112.8),
        ("2023-12-15", 106.3), ("2024-03-15", 102.1), ("2024-07-15", 104.0), ("2024-12-15", 104.8), ("2025-06-15", 104.1),
        ("2025-12-15", 103.2), ("2026-08-15", 102.9))
FLASH = {"2024-12": "-0.1", "2025-06": "0.2", "2025-12": "-0.1"}  # flash estimates that the final figure replaced
r = rng(SEED + 2)
cpi_year = {}
m = date(2018, 1, 15)
while m <= date(2026, 8, 15):
    v = q(interp(CPI, m) + r.uniform(-0.2, 0.2), 1)
    ym = month_key(m)
    cpi_year.setdefault(m.year, []).append(v)
    nm = date(m.year + (m.month == 12), m.month % 12 + 1, 15)
    pub = CAL.on_or_next(nm)
    if ym in FLASH:
        fp = CAL.nth_last(m.year, m.month, 1)
        row(f"CPI-YY-{ym}", v + Decimal(FLASH[ym]), "index", fp, fp, None, "TSB-2", f"CPI {ym}, flash estimate")
    row(f"CPI-YY-{ym}", v, "index", pub, pub, None, "TSB-2", f"CPI {ym}")
    m = nm
for y, vs in cpi_year.items():
    if len(vs) == 12:
        p = CAL.on_or_next(date(y + 1, 1, 15))
        row(f"CPI-AVG-{y}", q(float(sum(vs)) / 12, 1), "index", p, p, None, "GEN-6", f"CPI average of {y} (previous year = 100), for contracts that index by it")

# --- Savings bonds: first-year rate and margin of each monthly series, 2018-01 .. 2026-09 (TSB-2)
R1 = [("2018-01", "2.70"), ("2020-04", "2.40"), ("2020-06", "1.70"), ("2022-01", "2.40"), ("2022-02", "3.40"),
      ("2022-03", "4.40"), ("2022-04", "5.40"), ("2022-05", "6.00"), ("2022-06", "6.50"), ("2022-07", "7.00"),
      ("2022-08", "7.25"), ("2023-10", "6.85"), ("2024-01", "6.80"), ("2024-07", "6.55"), ("2025-01", "6.25"),
      ("2025-07", "5.90"), ("2025-10", "5.60"), ("2026-01", "5.35")]
MARGIN = [("2018-01", "1.50"), ("2020-04", "1.00"), ("2022-03", "1.25"), ("2024-01", "2.00")]


def step(table, ym):
    v = None
    for k, x in table:
        if k <= ym:
            v = x
    return v


m = date(2018, 1, 1)
while m <= LAST:
    ym = month_key(m)
    yymm = ym[2:4] + ym[5:7]
    prev = date(m.year - (m.month == 1), (m.month - 2) % 12 + 1, 1)
    pub = CAL.nth_last(prev.year, prev.month, 3)
    f, t = m, end_of_month(m)
    row(f"ISB-R1-{yymm}", step(R1, ym), "% a year", pub, f, t, "TSB-2", f"series ISB{yymm}, first-year rate")
    row(f"ISB-MARGIN-{yymm}", step(MARGIN, ym), "% a year", pub, f, t, "TSB-2", f"series ISB{yymm}, margin")
    m = date(m.year + (m.month == 12), m.month % 12 + 1, 1)

# --- The Central Bank's FLR mid rate, every business day 2021-01-04 .. LAST (GEN-4)
FX = A(("2021-01-04", 4.55), ("2021-12-31", 4.60), ("2022-03-07", 4.88), ("2022-06-30", 4.70), ("2022-10-14", 4.87),
       ("2023-03-31", 4.66), ("2023-10-31", 4.45), ("2024-06-28", 4.31), ("2025-03-31", 4.22), ("2025-12-31", 4.26),
       ("2026-09-30", 4.24))
r = rng(SEED + 3)
dev, d = 0.0, date(2021, 1, 4)
while d <= LAST:
    if CAL.is_business(d):
        dev = 0.97 * dev + r.gauss(0, 0.006)
        row("FX-FLR", q(interp(FX, d) + dev, 4), "DUK/FLR", d, d, d, "GEN-4", "")
    d += timedelta(days=1)

def write():
    rows.sort(key=lambda x: (x[0], x[4], x[3]))
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", newline="", encoding="utf-8") as fh:
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(["id", "value", "unit", "published", "from", "to", "rule", "note"])
        w.writerows(rows)
    print(f"wrote {OUT}: {len(rows)} rows")


if __name__ == "__main__":
    write()
