#!/usr/bin/env python3
"""Write the hidden figures: Rosvald's gazette (`law/rosvald-gazette.csv`) and the CBL's reference rate
(`law/home-gazette-supplement.csv`). Run before gen_c.py and keys_c.py.

Rosvald's figures are stated here as its gazette published them. The RNB's monthly reference rate for
the dukat (FLR per 100 DUK) is set from the CBL's mid rate of the day before its publication, with a
small stated spread, to 4 places: it is the RNB's own figure, not the CBL's.

Run: python3 -I engine/tests/cases/tools/gen_c_gazette.py
"""
import pathlib
import sys
from datetime import date, timedelta
from decimal import Decimal, ROUND_HALF_UP

HOLD = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(HOLD / "tools"))  # cases 1.2.0: one tree

from fam.home import ALL_KNOWN, figure  # noqa: E402
from lib.dates import Calendar, end_of_month  # noqa: E402

RCAL = Calendar([{"md": md} for md in ("01-01", "05-01", "06-14", "10-03", "12-24", "12-25", "12-26", "12-31")])


def last_business(y, m):
    d = end_of_month(date(y, m, 1))
    while not RCAL.is_business(d):
        d -= timedelta(days=1)
    return d


def write():
    rows = [("id", "value", "unit", "published", "from", "to", "rule", "note")]
    years = {"2024/25": ("2024-02-15", "2024-04-01", "2025-03-31", "950.00", "1700.00", "1200.00", "4000.00"),
             "2025/26": ("2025-02-14", "2025-04-01", "2026-03-31", "980.00", "1750.00", "1250.00", "4100.00"),
             "2026/27": ("2026-02-13", "2026-04-01", "2027-03-31", "1000.00", "1790.00", "1280.00", "4200.00")}
    for ty, (pub, f, t, a1, a2, e1, e2) in years.items():
        rows.append((f"RWT-ALLOW1-{ty}", a1, "FLR/month", pub, f, t, "RWT-4", "class 1 allowance"))
        rows.append((f"RWT-ALLOW2-{ty}", a2, "FLR/month", pub, f, t, "RWT-4", "class 2 allowance"))
        rows.append((f"RWT-EDGE1-{ty}", e1, "FLR/month", pub, f, t, "RWT-3", "upper edge of the 10 % band"))
        rows.append((f"RWT-EDGE2-{ty}", e2, "FLR/month", pub, f, t, "RWT-3", "upper edge of the 24 % band"))
    for y, v, pub in ((2024, "7500.00", "2023-12-14"), (2025, "7700.00", "2024-12-12"), (2026, "7900.00", "2025-12-11")):
        rows.append((f"RSI-CEIL-{y}", v, "FLR/month", pub, f"{y}-01-01", f"{y}-12-31", "RSI-2", "monthly contribution ceiling"))
    for y, v, pub in ((2024, "210.00", "2023-11-30"), (2025, "220.00", "2024-11-28"), (2026, "230.00", "2025-11-27")):
        rows.append((f"RFF-CHILD-{y}", v, "FLR/month", pub, f"{y}-01-01", f"{y}-12-31", "RFF-1", "child benefit per child"))
    # RNB-DUK: published on the last Rosvald business day of the month before, for the month
    spread = [Decimal(x) for x in ("0.0004", "-0.0006", "0.0009", "-0.0003", "0.0007", "-0.0008")]
    y, m, i = 2024, 1, 0
    while (y, m) <= (2026, 10):
        py, pm = (y, m - 1) if m > 1 else (y - 1, 12)
        pub = last_business(py, pm)
        d = pub - timedelta(days=1)
        while True:
            try:
                mid, _ = figure("FX-FLR", d, ALL_KNOWN)  # R1: the generator writes the world, so it sees every figure
                break
            except LookupError:
                d -= timedelta(days=1)
        v = (Decimal(100) / mid * (1 + spread[i % len(spread)])).quantize(Decimal("0.0001"), rounding=ROUND_HALF_UP)
        ld = end_of_month(date(y, m, 1))
        rows.append((f"RNB-DUK-{y}{m:02d}", str(v), "FLR/100 DUK", pub.isoformat(), f"{y}-{m:02d}-01", ld.isoformat(),
                     "RGP-3", "RNB reference rate of the dukat for the month"))
        y, m, i = (y, m + 1, i + 1) if m < 12 else (y + 1, 1, i + 1)
    (HOLD / "law" / "rosvald-gazette.csv").write_text("\n".join(",".join(r) for r in rows) + "\n", encoding="utf-8")

    ref = [("id", "value", "unit", "published", "from", "to", "rule", "note"),
           ("CBL-REF", "5.75", "%/year", "2023-10-04", "2023-10-05", "2025-03-05", "", "CBL reference rate"),
           ("CBL-REF", "5.25", "%/year", "2025-03-05", "2025-03-06", "2025-10-01", "", "CBL reference rate"),
           ("CBL-REF", "4.75", "%/year", "2025-10-01", "2025-10-02", "2026-05-06", "", "CBL reference rate"),
           ("CBL-REF", "4.50", "%/year", "2026-05-06", "2026-05-07", "", "", "CBL reference rate")]
    (HOLD / "law" / "home-gazette-supplement.csv").write_text("\n".join(",".join(r) for r in ref) + "\n", encoding="utf-8")


if __name__ == "__main__":
    write()
