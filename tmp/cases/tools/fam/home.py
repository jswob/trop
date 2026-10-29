"""The home law at run time, for the family scripts: gazette figures, market prices, payroll
(SOC-1, SOC-2, WCP-1, PIT-2, PIT-4) and the yearly return (PIT-3, PIT-5, PIT-6, PIT-7).

Every function cites its rule. Money is Decimal; rounding only where GEN-5 or a rule says.
"""
import csv
import pathlib
import sys
from datetime import date, timedelta
from decimal import Decimal, ROUND_DOWN, ROUND_HALF_UP

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))

from gen_gazette import CAL  # noqa: E402  GEN-1: weekends, the fixed holidays, Spring Day
from lib.dates import P  # noqa: E402
from lib.num import CENT, UNIT, D  # noqa: E402

ZERO = Decimal(0)
ALL_KNOWN = date.max  # R1: the known date of a generator writing the world, which sees every figure and price


def ort(x):
    """GEN-5.3: whole orts, half up."""
    return D(x).quantize(CENT, rounding=ROUND_HALF_UP)


def duk(x):
    """GEN-5.1: whole dukats, under 50 orts down, 50 and over up."""
    return D(x).quantize(UNIT, rounding=ROUND_HALF_UP)


def units_down(x, places=4):
    return D(x).quantize(Decimal(1).scaleb(-places), rounding=ROUND_DOWN)


# --- the gazette (GEN-6)

_GAZ = None


def _gaz():
    global _GAZ
    if _GAZ is None:
        _GAZ = list(csv.DictReader(open(ROOT / "law" / "gazette.csv", encoding="utf-8")))
    return _GAZ


def figure(fid, on, known):
    """GEN-6.1: the figure in force on `on`, as published by `known` (required: a key passes its question's
    known-as-of date, a generator ALL_KNOWN). Returns (value, row id text)."""
    on, known = P(on), P(known)
    rows = [r for r in _gaz() if r["id"] == fid and P(r["from"]) <= on and (not r["to"] or on <= P(r["to"]))
            and P(r["published"]) <= known]
    if not rows:
        raise LookupError(f"{fid} not in force on {on} as published by {known}")  # GEN-6.2
    r = max(rows, key=lambda r: (r["from"], r["published"]))
    return D(r["value"]), f"gazette {fid} from {r['from']} (published {r['published']})"


# --- market files (facts from a private party)

_PX = {}


def prices(code, known):
    """{date: NAV} from the fund's own file, in its house's dialect: the valuation days up to `known` (required)."""
    known = P(known)
    if code in _PX:
        return {d: v for d, v in _PX[code].items() if d <= known}
    fn = {r["code"]: r["file"] for r in csv.DictReader(open(ROOT / "market" / "funds.csv", encoding="utf-8"))}[code]
    lines = (ROOT / "market" / fn).read_text(encoding="utf-8").splitlines()
    out = {}
    if lines[0].startswith("valuation_date"):
        for ln in lines[1:]:
            d, v = ln.split(",")
            out[P(d)] = D(v)
    else:
        for ln in lines[2:]:
            d, v = ln.split(";")
            out[date(int(d[6:]), int(d[3:5]), int(d[:2]))] = D(v.replace(",", "."))
    _PX[code] = out
    return {d: v for d, v in out.items() if d <= known}


def price_on(code, d, known):
    """The fund's price of the last valuation day on or before d (PIT-8.2, IGT-2.2), as known on `known`
    (required): a day after `known` has no price yet, so it raises."""
    px, d = prices(code, known), P(d)
    if d > P(known):
        raise LookupError(f"{code} on {d}: not known on {known}")
    while d not in px:
        d -= timedelta(days=1)
        if d.year < 2018:
            raise LookupError(code)
    return px[d], d


def distributions():
    """[(fund, ex, paid, per_unit)] from both houses' files."""
    out = []
    for r in csv.DictReader(open(ROOT / "market" / "brz-distributions.csv", encoding="utf-8")):
        out.append((r["fund"], P(r["ex_date"]), P(r["pay_date"]), D(r["per_unit_DUK"])))
    for ln in (ROOT / "market" / "wek-distributions.csv").read_text(encoding="utf-8").splitlines()[2:]:
        f, ex, pd, pu = ln.split(";")
        dd = lambda s: date(int(s[6:]), int(s[3:5]), int(s[:2]))  # noqa: E731
        out.append((f, dd(ex), dd(pd), D(pu.replace(",", "."))))
    return sorted(out, key=lambda x: (x[2], x[0]))


# --- the tax scale (PIT-3)

PIT3 = {
    "v1": {"edge": D("90000.00"), "low": D("0.18"), "high": D("0.33"), "reducing": D("1530.00")},
    "v2": {"edge": D("120000.00"), "low": D("0.18"), "high": D("0.33"), "reducing": D("5400.00")},
    "v3": {"edge": D("120000.00"), "low": D("0.13"), "high": D("0.33"), "reducing": D("3900.00")},
}


def scale(base, v):
    """PIT-3 .1: the scale on a base (no reducing amount, no rounding)."""
    s = PIT3[v]
    base = D(base)
    if base <= s["edge"]:
        return base * s["low"]
    return s["edge"] * s["low"] + (base - s["edge"]) * s["high"]


def pit3_tax(base, v):
    """PIT-3 .3: the scale less the reducing amount, never below 0 (rounded by the caller's rule)."""
    return max(ZERO, scale(base, v) - PIT3[v]["reducing"])


def year_version(year, law="enacted"):
    """The PIT-3 version governing the tax for a tax year (PIT-3 governs lines).

    law: 'enacted' (v3 from 2022), 'known-2022-05-31' (v2 for 2022: v3 not yet published),
    'no-cut' (the twist removed: v3 never enacted, v2 from 2022 on).
    """
    if year <= 2021:
        return "v1"
    if law in ("known-2022-05-31", "no-cut"):
        return "v2"
    return "v3"


def advance_versions(paid, law="enacted"):
    """PIT-3 versions an advance on a payment day is computed with (PIT-4 .2, PIT-4 v2 .4)."""
    paid = P(paid)
    if paid < date(2022, 1, 1):
        return ["v1"]
    if law in ("known-2022-05-31", "no-cut") or paid < date(2022, 7, 1):
        return ["v2"]
    if paid <= date(2022, 12, 31):
        return ["v2", "v3"]  # PIT-4 v2 .4: the lower of the two
    return ["v3"]


# --- payroll

def soc1(gross, gross_before, cap):
    """SOC-1: pension 9.50 %, disability 1.50 % up to the year's SOC-CAP (from this employer); sickness 2.45 %."""
    gross, gross_before = D(gross), D(gross_before)
    capped = max(ZERO, min(gross, cap - gross_before))
    pen = ort(capped * D("0.095"))
    dis = ort(capped * D("0.015"))
    sick = ort(gross * D("0.0245"))
    return {"pension": pen, "disability": dis, "sickness": sick, "total": pen + dis + sick, "capped_base": capped}


def run_payer(payments, reducing=True, law="enacted", *, known):
    """PIT-2, PIT-4, SOC-1, SOC-2 and WCP-1 for one payer's payments of one calendar year.

    payments: [{date, kind: pay|parental|wcp_er, gross, wcp_ee_rates}] (wcp_er = the employer's
    plan contributions, income on the day paid to the plan, WCP-2.1). Returns one row per month in
    which the payer pays employment income, and the year's totals (the payer's yearly statement).
    """
    by_month = {}
    for p in sorted(payments, key=lambda p: P(p["date"])):
        by_month.setdefault(P(p["date"]).month, []).append(p)
    year = P(payments[0]["date"]).year
    rows, cum_base, gross_ytd = [], ZERO, ZERO
    tot = {"income": ZERO, "costs": ZERO, "soc": ZERO, "health": ZERO, "health_ded": ZERO, "advance": ZERO, "wcp_ee": ZERO}
    for m in sorted(by_month):
        income = soc_total = health = health_ded = wcp_ee = ZERO
        net = ZERO
        has_pay = False
        detail = []
        for p in by_month[m]:
            g = D(p["gross"])
            income += g
            if p["kind"] == "pay":
                has_pay = True
                cap, _ = figure("SOC-CAP", p["date"], known)
                s = soc1(g, gross_ytd, cap)
                gross_ytd += g
                h = ort((g - s["total"]) * D("0.09"))  # SOC-2 .1
                hd = min(h, (g - s["total"]) * D("0.0775")) if P(p["date"]).year <= 2021 else ZERO  # SOC-2 v1 .2
                ee = sum((ort(g * D(r)) for r in p.get("wcp_ee_rates", ())), ZERO)  # WCP-1 .2 and .4, WCP-2 .2
                soc_total += s["total"]
                health += h
                health_ded += hd
                wcp_ee += ee
                net += g - s["total"] - h - ee
                detail.append({"date": P(p["date"]).isoformat(), "gross": g, "soc": s["total"], "health": h, "wcp_ee": ee})
            elif p["kind"] == "parental":
                net += g  # SOC-8 .5: no contributions
                detail.append({"date": P(p["date"]).isoformat(), "parental": g})
            else:
                detail.append({"date": P(p["date"]).isoformat(), "wcp_er": g})
        costs = D("250.00") if has_pay else ZERO  # PIT-2 .2: a month in which pay is paid
        base_exact = max(ZERO, income - soc_total - costs)
        base = duk(base_exact)  # PIT-4 .1, GEN-5.1
        pay_day = max(P(p["date"]) for p in by_month[m])
        cands = {}
        for v in advance_versions(pay_day, law):
            a = scale(cum_base + base, v) - scale(cum_base, v)
            if reducing:
                a -= PIT3[v]["reducing"] / 12
            a -= health_ded  # PIT-4 v1 .2
            cands[v] = duk(max(ZERO, a))
        adv = min(cands.values())
        cum_base += base
        net -= adv
        rows.append({"month": f"{year}-{m:02d}", "income": income, "soc": soc_total, "costs": costs, "base": base,
                     "advance": adv, "candidates": cands, "health": health, "wcp_ee": wcp_ee, "net": net,
                     "detail": detail})
        tot["income"] += income
        tot["costs"] += costs
        tot["soc"] += soc_total
        tot["health"] += health
        tot["health_ded"] += health_ded
        tot["advance"] += adv
        tot["wcp_ee"] += wcp_ee
    tot["base"] = max(ZERO, tot["income"] - tot["soc"] - tot["costs"])  # PIT-2 .3 for the year
    return rows, tot


def child_relief(year, months, parents_income):
    """PIT-7: 125.00 a month, cut by the parents' income above 112 000.00, never below 0; halves."""
    full = D("125.00") * months
    cut = max(ZERO, D(parents_income) - D("112000.00"))
    r = max(ZERO, full - cut)
    return {"full": full, "cut": cut, "relief": r, "half": r / 2}


def separate_return(year, payer_totals, relief=ZERO, law="enacted", health_ded=True):
    """PIT-5 for one person: base rounded, tax by the governing PIT-3 version, less reliefs; advances."""
    v = year_version(year, law)
    base = duk(sum((t["base"] for t in payer_totals), ZERO))  # PIT-5 .2
    tax = pit3_tax(base, v) - relief
    if year <= 2021 and health_ded:
        tax -= sum((t["health_ded"] for t in payer_totals), ZERO)  # PIT-5 v1 .3
    tax = duk(max(ZERO, tax))
    adv = sum((t["advance"] for t in payer_totals), ZERO)
    return {"version": v, "base": base, "tax": tax, "advances": adv, "due": tax - adv}


def joint_return(year, a_totals, b_totals, reliefs=(ZERO, ZERO), law="enacted", health="each"):
    """PIT-6: twice the tax on half the combined base; reliefs; each spouse's part by base, whole dukats.

    health (tax years to 2021 only): 'each' = each spouse's PIT-5 v1 .3 deduction from the joint tax;
    'none' = PIT-6 .4 lists only the child reliefs, so nothing else is deducted.
    """
    v = year_version(year, law)
    ba = duk(sum((t["base"] for t in a_totals), ZERO))
    bb = duk(sum((t["base"] for t in b_totals), ZERO))
    combined = duk(ba + bb)  # PIT-6 .3
    half = combined / 2
    joint = duk(2 * pit3_tax(half, v))
    hd = [ZERO, ZERO]
    if year <= 2021 and health == "each":
        hd = [sum((t["health_ded"] for t in a_totals), ZERO), sum((t["health_ded"] for t in b_totals), ZERO)]
    # PIT-6 .5: parts in proportion to own base, whole dukats, the dukat left over to the larger base
    total_units = int(joint)
    pa = (total_units * ba // combined) if combined else 0
    pb = (total_units * bb // combined) if combined else 0
    left = total_units - int(pa) - int(pb)
    if ba >= bb:
        pa += left
    else:
        pb += left
    parts = [D(int(pa)), D(int(pb))]
    adv = [sum((t["advance"] for t in a_totals), ZERO), sum((t["advance"] for t in b_totals), ZERO)]
    after = duk(max(ZERO, joint - reliefs[0] - reliefs[1] - hd[0] - hd[1]))
    each = [parts[i] - reliefs[i] - hd[i] - adv[i] for i in range(2)]
    if year <= 2021 and health == "each":  # like PIT-5: the tax after the health deduction, whole dukats (GEN-5)
        each = [duk(max(ZERO, parts[i] - reliefs[i] - hd[i])) - adv[i] for i in range(2)]
    return {"version": v, "bases": [ba, bb], "combined": combined, "half": half, "joint_tax": joint,
            "after_reliefs": after, "parts": parts, "reliefs": list(reliefs), "health_ded": hd, "advances": adv,
            "due_each": each, "due": after - adv[0] - adv[1]}


def gains_tax(net_gain, carried_in=ZERO, rate=D("0.19")):
    """PIT-8 .4: tax on the year's net gain less carried-in losses; base and tax in whole dukats."""
    base = duk(max(ZERO, D(net_gain) - D(carried_in)))
    return base, duk(base * rate)
