"""Household C's law at run time: Rosvald's law (RGP, RWT, RSI, RFF), the Lenia–Rosvald treaty and
its protocol, and the hidden points of the home law (PIT-16, PIT-17, PIT-18, FAM-2, IGT-2 .4).

Every function cites its rule. Money is Decimal; rounding only where a rule says. The visible home
law (fam/home.py) is reused for PIT-2..PIT-7, SOC-1, SOC-2.
"""
import csv
import pathlib
import sys
from datetime import date, timedelta
from decimal import Decimal, ROUND_DOWN, ROUND_HALF_EVEN, ROUND_HALF_UP, ROUND_UP

HOLD = pathlib.Path(__file__).resolve().parents[1]
VIS = HOLD  # cases 1.2.0: one tree, the holdout folded into the case set
sys.path.insert(0, str(VIS / "tools"))

from fam.home import CAL, PIT3, ZERO, D, duk, figure, ort, pit3_tax, scale  # noqa: E402,F401
from lib.dates import Calendar, P, add_months, end_of_month  # noqa: E402

CENT = Decimal("0.01")

# --------------------------------------------------------------------------- Rosvald: RGP

# RGP-1: Rosvald's business days: Monday to Friday, except these days.
RCAL = Calendar([{"md": md} for md in ("01-01", "05-01", "06-14", "10-03", "12-24", "12-25", "12-26", "12-31")])


def r_cent(x):
    """RGP-2: 'to the cent' is half to even."""
    return D(x).quantize(CENT, rounding=ROUND_HALF_EVEN)


def down10(x):
    """RWT-3 .2: taxable pay rounded down to a whole 10 FLR, never below 0."""
    x = max(ZERO, D(x))
    return (x / 10).quantize(Decimal(1), rounding=ROUND_DOWN) * 10


def down1(x):
    """RWT-3 .3, RWT-6 .3: tax rounded down to a whole floren."""
    return max(ZERO, D(x)).quantize(Decimal(1), rounding=ROUND_DOWN)


def half_up(x, step=CENT):
    return D(x).quantize(step, rounding=ROUND_HALF_UP)


# --------------------------------------------------------------------------- Rosvald gazette

_RG = None


def _rgaz():
    global _RG
    if _RG is None:
        _RG = list(csv.DictReader(open(HOLD / "law" / "rosvald-gazette.csv", encoding="utf-8")))
    return _RG


def rfig(fid, *, known):
    """A Rosvald figure by its id (one row per id), as published by `known` (required)."""
    rows = [r for r in _rgaz() if r["id"] == fid and P(r["published"]) <= P(known)]
    if not rows:
        raise LookupError(f"{fid} as published by {known}")
    return D(rows[-1]["value"]), f"Rosvald gazette {fid} (published {rows[-1]['published']})"


def tax_year(paid):
    """RWT-1: the tax year runs 1 April – 31 March; pay belongs to the year in which it is paid."""
    paid = P(paid)
    y = paid.year if paid.month >= 4 else paid.year - 1
    return f"{y}/{(y + 1) % 100:02d}"


# --------------------------------------------------------------------------- RSI, RWT

def rsi(gross, paid, *, known):
    """RSI-2: 13.50 % of the gross pay up to the monthly ceiling of the calendar year of payment; to the cent."""
    ceil, _ = rfig(f"RSI-CEIL-{P(paid).year}", known=known)
    return r_cent(min(D(gross), ceil) * D("0.135"))


def bands(taxable, ty, months=1, *, known):
    """RWT-3 .3: 10 % up to the first edge, 24 % up to the second, 38 % above (edges times `months`)."""
    e1, _ = rfig(f"RWT-EDGE1-{ty}", known=known)
    e2, _ = rfig(f"RWT-EDGE2-{ty}", known=known)
    e1, e2, t = e1 * months, e2 * months, D(taxable)
    out = min(t, e1) * D("0.10")
    if t > e1:
        out += (min(t, e2) - e1) * D("0.24")
    if t > e2:
        out += (t - e2) * D("0.38")
    return out


def allowance(cls, ty, *, known):
    """RWT-4: the monthly allowance of the class for the tax year."""
    v, _ = rfig(f"RWT-ALLOW{cls}-{ty}", known=known)
    return v


def withhold(gross, paid, cls, *, known):
    """RWT-3: one payment's withholding. Returns soc, allowance, taxable, tax, net."""
    ty = tax_year(paid)
    s = rsi(gross, paid, known=known)
    a = allowance(cls, ty, known=known)
    taxable = down10(D(gross) - s - a)
    tax = down1(bands(taxable, ty, known=known))
    return {"tax year": ty, "class": cls, "gross": D(gross), "soc": s, "allowance": a, "taxable": taxable,
            "tax": tax, "net": D(gross) - s - tax}


def assessment(payments, ty, cls_end, *, known):
    """RWT-6: the yearly tax on the year's pay; refund = withheld − yearly tax (negative: payable)."""
    g = sum((p["gross"] for p in payments), ZERO)
    s = sum((p["soc"] for p in payments), ZERO)
    w = sum((p["tax"] for p in payments), ZERO)
    a = allowance(cls_end, ty, known=known) * 12
    taxable = down10(g - s - a)
    tax = down1(bands(taxable, ty, 12, known=known))
    return {"gross": g, "soc": s, "withheld": w, "allowance": a, "taxable": taxable, "tax": tax, "refund": w - tax}


def gross_from_net(net, paid, cls, base, rate, max_hours=80, *, known):
    """REC-05: the grosses (base + whole overtime hours × rate, RWT contract terms) giving this net."""
    out = []
    for h in range(0, max_hours + 1):
        g = D(base) + h * D(rate)
        if withhold(g, paid, cls, known=known)["net"] == D(net):
            out.append((h, g))
    return out


def gross_from_net_any(net, paid, cls, lo, hi, *, known):
    """Every gross in whole cents between lo and hi giving this net (for the writers' notes)."""
    out = []
    g = D(lo)
    while g <= D(hi):
        if withhold(g, paid, cls, known=known)["net"] == D(net):
            out.append(g)
        g += CENT
    return out


# --------------------------------------------------------------------------- the treaty, family benefits

def rnb_rate(ym, *, known):
    """The RNB's reference rate for a month: FLR per 100 DUK (Rosvald gazette), last published on or before the 1st."""
    v, src = rfig(f"RNB-DUK-{ym.replace('-', '')}", known=known)
    return v, src


def cbl_on_first(ym, *, known):
    """The CBL's mid rate last published on or before the month's first day (treaty T-12 .2)."""
    d = date(int(ym[:4]), int(ym[5:]), 1)
    if d > P(known):  # R1: the walk back below must not stand in for a rate not yet published
        raise LookupError(f"FX-FLR on {d}: not known on {known}")
    while True:
        try:
            v, _ = figure("FX-FLR", d, known=known)
            return v, d
        except LookupError:
            d -= timedelta(days=1)


def to_flr(duk_amt, ym, *, known):
    """T-12 .2: a dukat benefit in florens at the RNB's rate, to the cent, half up."""
    r, _ = rnb_rate(ym, known=known)
    return half_up(D(duk_amt) * r / 100)


def to_duk(flr_amt, ym, *, known):
    """T-12 .2: a floren benefit in dukats at the CBL's rate, to the ort, half up."""
    r, d = cbl_on_first(ym, known=known)
    return half_up(D(flr_amt) * r), r, d


def rff_amount(ym, *, known):
    v, _ = rfig(f"RFF-CHILD-{ym[:4]}", known=known)
    return v


def fam1_amount(ym):
    """FAM-1: 400.00 DUK a month to 2023-12, 650.00 DUK from 2024-01."""
    return D("650.00") if ym >= "2024-01" else D("400.00")


def coordinate(ym, marta_works, jan_works_r, *, known):
    """T-12: who pays first, and the difference the other pays (child living in Lenia).

    Returns {"first": "Lenia"|"Rosvald"|None, "Lenia": DUK, "Rosvald": FLR, ...}."""
    lenia = fam1_amount(ym)
    if not jan_works_r:
        return {"first": "Lenia", "Lenia": lenia, "Rosvald": ZERO}
    ros = rff_amount(ym, known=known)
    if marta_works:  # T-12 .1 (a): a parent works in the child's state
        conv = to_flr(lenia, ym, known=known)
        return {"first": "Lenia", "Lenia": lenia, "Rosvald": max(ZERO, ros - conv), "converted": conv,
                "rnb": rnb_rate(ym, known=known)[0]}
    conv, r, d = to_duk(ros, ym, known=known)  # T-12 .1 (b): only the parent in Rosvald works
    return {"first": "Rosvald", "Lenia": max(ZERO, lenia - conv), "Rosvald": ros, "converted": conv,
            "cbl": r, "cbl day": d.isoformat()}


# --------------------------------------------------------------------------- the home law: foreign pay (PIT-16, PIT-18)

def gen4(amount, event, *, known):
    """GEN-4: the CBL mid rate of the last business day before the event; each amount to the ort, half up."""
    day = CAL.before(event)
    r, _ = figure("FX-FLR", day, known=known)
    return half_up(D(amount) * r), r, day


def foreign_base(slip, *, known):
    """PIT-18 .1: converted gross less converted contributions less 250.00 DUK (PIT-2.2); PIT-18 .2: tax converted alike."""
    g, r, day = gen4(slip["gross"], slip["paid"], known=known)
    s, _, _ = gen4(slip["soc"], slip["paid"], known=known)
    t, _, _ = gen4(slip["tax"], slip["paid"], known=known)
    return {"paid": slip["paid"], "rate": r, "rate day": day.isoformat(), "gross DUK": g, "soc DUK": s,
            "costs": D("250.00"), "base": g - s - D("250.00"), "tax DUK": t}


def pit17(gift_total, base_marta, base_jan):
    """PIT-17: a gift from community property is each spouse's half; each deducted up to 10 % of that
    spouse's base (rounded, before the deduction); what is above is lost."""
    half = D(gift_total) / 2
    cm, cj = base_marta * D("0.10"), base_jan * D("0.10")
    dm, dj = min(half, cm), min(half, cj)
    return {"half": half, "cap Marta": cm, "cap Jan": cj, "Marta": dm, "Jan": dj}


def joint_tax(combined, exempt=ZERO, v="v3"):
    """PIT-6 .3, with PIT-18 .4 where there is exempt income: twice the PIT-3 tax on half of
    (combined + exempt), times combined over that sum; rounded (GEN-5.1)."""
    combined, exempt = D(combined), D(exempt)
    if exempt == 0:
        return duk(2 * pit3_tax(combined / 2, v))
    whole = combined + exempt
    return duk(2 * pit3_tax(whole / 2, v) * combined / whole)


def child_relief(months, parents_income):
    """PIT-7: 125.00 a month, cut by the parents' income above 112 000.00; never below 0."""
    full = D("125.00") * months
    return max(ZERO, full - max(ZERO, D(parents_income) - D("112000.00")))


def credit(tax_after_relief, foreign_base_sum, whole_base, foreign_tax):
    """PIT-16 .3 with PIT-18 .5: the lower of the foreign tax and the part of the tax on that income;
    the part rounded (GEN-5.1); the rest is lost."""
    part = duk(D(tax_after_relief) * D(foreign_base_sum) / D(whole_base))
    return min(D(foreign_tax), part), part


# --------------------------------------------------------------------------- Marta's loan (C-HR7)

def cbl_ref(d, *, known):
    """The CBL's reference rate in force on a day (home-gazette-supplement.csv), as published by `known` (required)."""
    best = None
    for r in csv.DictReader(open(HOLD / "law" / "home-gazette-supplement.csv", encoding="utf-8")):
        if r["id"] == "CBL-REF" and P(r["from"]) <= P(d) and (not r["to"] or P(d) <= P(r["to"])) and P(r["published"]) <= P(known):
            best = (D(r["value"]), r["from"])
    if best is None:
        raise LookupError(d)
    return best
