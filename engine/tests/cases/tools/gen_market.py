"""Writes market/: the private parties' price files (brief §4: who names a figure decides where it lives).

- two fund houses' daily valuations (each in its own CSV dialect) and their distributions;
- the reference rate DUKOR (3M, 6M), fixed by the Lenian Rate Administrator;
- Bank Lazur's FLR exchange table (its buy and sell rates, around the Central Bank's mid rate).

Every series comes from stated anchors, stated parameters and a seed; the files are the facts.
Reads law/gazette.csv (the FLR mid rate), so run gen_gazette.py first.
Run: python3 -I tmp/cases/tools/gen_market.py
"""
import csv
import math
import os
import sys
from datetime import date, timedelta
from decimal import ROUND_HALF_UP, Decimal

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gen_gazette import A, CAL, LAST, SEED, interp, q  # noqa: E402
from lib.paths import rng  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MKT = os.path.join(ROOT, "market")
START = date(2018, 1, 2)

EQUITY = A(("2018-01-02", 100), ("2018-12-28", 92), ("2019-12-31", 108), ("2020-02-21", 112), ("2020-03-23", 76),
           ("2020-12-31", 112), ("2021-12-31", 133), ("2022-09-30", 98), ("2022-12-30", 104), ("2023-12-29", 122),
           ("2024-12-31", 139), ("2025-04-08", 128), ("2025-12-31", 150), ("2026-09-30", 158))
BONDS = A(("2018-01-02", 100), ("2019-12-31", 105), ("2020-12-31", 108), ("2021-12-31", 104), ("2022-10-31", 94),
          ("2023-12-29", 104), ("2024-12-31", 110), ("2025-12-31", 116), ("2026-09-30", 119))
MONEY = A(("2018-01-02", 100), ("2021-12-31", 101.5), ("2022-12-30", 107.4), ("2023-12-29", 114.0),
          ("2024-12-31", 119.9), ("2025-12-31", 125.1), ("2026-09-30", 128.4))

# code, house, name, kind, start day, start NAV, weights on (equity, bonds, money), idiosyncratic daily sd,
# distribution: (yield, months) or None
FUNDS = [
    ("BRZ-GLB-A", "BRZ", "Brzask Global Equity", "accumulating", "2018-01-02", 152.30, (1.0, 0, 0), 0.0035, None),
    ("BRZ-BND-A", "BRZ", "Brzask Treasury Bond", "accumulating", "2018-01-02", 118.40, (0, 1.0, 0), 0.0008, None),
    ("BRZ-DIV-D", "BRZ", "Brzask Dividend Equity", "distributing", "2018-01-02", 64.10, (0.8, 0, 0), 0.0040, (0.030, (5,))),
    ("BRZ-MM-A", "BRZ", "Brzask Money Market", "accumulating", "2018-01-02", 103.25, (0, 0, 1.0), 0.0001, None),
    ("WEK-EM-A", "WEK", "Wektor Emerging Markets", "accumulating", "2018-01-02", 41.2730, (1.25, 0, 0), 0.0060, None),
    ("WEK-IDX-D", "WEK", "Wektor Lenia Index", "distributing", "2018-01-02", 210.5520, (1.1, 0, 0), 0.0045, (0.016, (3, 9))),
    ("WEK-WCP-2055", "WEK", "Wektor Workplace Plan 2055", "accumulating", "2019-07-01", 100.0000, (0.7, 0.3, 0), 0.0020, None),
]
PLACES = {"BRZ": 2, "WEK": 4}


def business_days(a, b):
    d = a
    while d <= b:
        if CAL.is_business(d):
            yield d
        d += timedelta(days=1)


def factor_paths():
    """Log levels of the three factors on each business day: the anchors plus an AR(1) wobble."""
    r = rng(SEED + 10)
    out, de, db = {}, 0.0, 0.0
    for d in business_days(START, LAST):
        de = 0.98 * de + r.gauss(0, 0.009)
        db = 0.98 * db + r.gauss(0, 0.0025)
        out[d] = (math.log(interp(EQUITY, d)) + de, math.log(interp(BONDS, d)) + db, math.log(interp(MONEY, d)))
    return out


def distribution_day(y, month):
    """Ex-day: the 15th or the next business day; paid on the 5th business day after."""
    ex = CAL.on_or_next(date(y, month, 15))
    pay, n = ex, 0
    while n < 5:
        pay += timedelta(days=1)
        if CAL.is_business(pay):
            n += 1
    return ex, pay


def fmt(v, places, comma):
    s = f"{v:.{places}f}"
    return s.replace(".", ",") if comma else s


def main():
    os.makedirs(MKT, exist_ok=True)
    fac = factor_paths()
    days = sorted(fac)
    dists = {"BRZ": [], "WEK": []}
    for code, house, name, kind, start, nav0, w, sd, dist in FUNDS:
        start = date.fromisoformat(start)
        r = rng(SEED + 20 + sum(map(ord, code)))
        idio, adj, rows = 0.0, 1.0, []
        base0 = None
        ex_days = {}
        if dist:
            for y in range(start.year, LAST.year + 1):
                for mth in dist[1]:
                    ex, pay = distribution_day(y, mth)
                    if start < ex <= LAST:
                        ex_days[ex] = pay
        for d in days:
            if d < start:
                continue
            idio = 0.9 * idio + r.gauss(0, sd)
            lv = sum(wi * fi for wi, fi in zip(w, fac[d]))
            if base0 is None:
                base0 = lv
            raw = nav0 * math.exp(lv - base0 + idio)
            if d in ex_days:
                before = raw * adj
                amount = q(before * dist[0], 2)
                adj *= 1 - float(amount) / before
                dists[house].append((code, d, ex_days[d], amount))
            nav = q(raw * adj, PLACES[house])
            rows.append((d, nav))
        path = os.path.join(MKT, f"{code.lower()}.csv")
        with open(path, "w", newline="", encoding="utf-8") as fh:
            if house == "BRZ":
                fh.write("valuation_date,nav_DUK\n")
                for d, v in rows:
                    fh.write(f"{d.isoformat()},{fmt(v, 2, False)}\n")
            else:
                fh.write(f"Wektor Funds - daily valuation - {name}\n")
                fh.write("Date;NAV (DUK)\n")
                for d, v in rows:
                    fh.write(f"{d.strftime('%d.%m.%Y')};{fmt(v, 4, True)}\n")

    with open(os.path.join(MKT, "brz-distributions.csv"), "w", newline="", encoding="utf-8") as fh:
        fh.write("fund,ex_date,pay_date,per_unit_DUK\n")
        for code, ex, pay, amt in dists["BRZ"]:
            fh.write(f"{code},{ex.isoformat()},{pay.isoformat()},{amt}\n")
    with open(os.path.join(MKT, "wek-distributions.csv"), "w", newline="", encoding="utf-8") as fh:
        fh.write("Wektor Funds - distributions\n")
        fh.write("Fund;Ex-date;Paid;Per unit (DUK)\n")
        for code, ex, pay, amt in dists["WEK"]:
            fh.write(f"{code};{ex.strftime('%d.%m.%Y')};{pay.strftime('%d.%m.%Y')};{fmt(amt, 2, True)}\n")

    with open(os.path.join(MKT, "funds.csv"), "w", newline="", encoding="utf-8") as fh:
        wr = csv.writer(fh, lineterminator="\n")
        wr.writerow(["code", "house", "name", "kind", "currency", "first_valuation", "file", "distributions"])
        for code, house, name, kind, start, *_rest in FUNDS:
            dist = _rest[-1]
            wr.writerow([code, house, name, kind, "DUK", start, f"{code.lower()}.csv",
                         "" if not dist else f"{dist[0] * 100:.1f} % of NAV, ex-day the 15th (or next business day) of month(s) {','.join(map(str, dist[1]))}"])

    # DUKOR: fixed every business day from 2021-01-04, % a year, two decimals
    D6 = A(("2021-01-04", 0.25), ("2021-09-30", 0.30), ("2021-12-31", 2.60), ("2022-03-31", 4.90), ("2022-06-30", 7.00),
           ("2022-10-14", 7.35), ("2023-06-30", 6.95), ("2023-10-31", 5.85), ("2024-06-28", 5.85), ("2024-12-31", 5.80),
           ("2025-05-30", 5.20), ("2025-09-30", 4.70), ("2025-12-31", 4.30), ("2026-09-30", 4.10))
    r = rng(SEED + 30)
    with open(os.path.join(MKT, "dukor.csv"), "w", newline="", encoding="utf-8") as fh:
        fh.write("fixing_date,DUKOR_3M,DUKOR_6M\n")
        for d in business_days(date(2021, 1, 4), LAST):
            six = interp(D6, d) + r.uniform(-0.02, 0.02)
            three = six - 0.08 + r.uniform(-0.02, 0.02)
            fh.write(f"{d.isoformat()},{q(max(three, 0.01), 2)},{q(max(six, 0.01), 2)}\n")

    # Bank Lazur's FLR table: buy and sell around the Central Bank's mid of the same day
    mids = []
    with open(os.path.join(ROOT, "law", "gazette.csv"), encoding="utf-8") as fh:
        for rw in csv.DictReader(fh):
            if rw["id"] == "FX-FLR":
                mids.append((date.fromisoformat(rw["from"]), float(rw["value"])))
    r = rng(SEED + 40)
    with open(os.path.join(MKT, "lazur-fx-flr.csv"), "w", newline="", encoding="utf-8") as fh:
        fh.write("Bank Lazur - exchange table - FLR\n")
        fh.write("Date;Buy;Sell\n")
        for d, mid in mids:
            spread = 0.025 + r.uniform(-0.002, 0.002)
            fh.write(f"{d.strftime('%d.%m.%Y')};{fmt(q(mid * (1 - spread), 4), 4, True)};{fmt(q(mid * (1 + spread), 4), 4, True)}\n")
    print(f"wrote {MKT}: {len(os.listdir(MKT))} files")


if __name__ == "__main__":
    main()
