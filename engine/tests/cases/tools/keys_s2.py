#!/usr/bin/env python3
"""Keys for card S2. Reads the card's documents and records, applies the card's rules, writes keys/S2/<qid>.md.

Run from the repository root: python3 -I -B engine/tests/cases/tools/keys_s2.py
Standard library only; every amount is an exact Decimal, rounded only where a rule of the card rounds.
"""
import calendar
import datetime as dt
import pathlib
import re
from decimal import Decimal as D, ROUND_HALF_UP

BASE = pathlib.Path(__file__).resolve().parents[1]
CARD = BASE / "S2" / "card.md"
OUT = BASE / "keys" / "S2"
TEXT = CARD.read_text(encoding="utf-8")


# ---- shared helpers (each key script carries its own copy) ----

def need(*parts):
    for p in parts:
        assert p in TEXT, f"the card doesn't say: {p!r}"


def record(rid):
    for block in re.findall(r"```csv\n(.*?)\n```", TEXT, re.S):
        lines = block.splitlines()
        if lines[0].startswith(rid + " "):
            head = lines[1]
            sep = ";" if ";" in head else ("|" if "|" in head else ",")
            return [ln.split(sep) for ln in lines[2:]]
    raise KeyError(rid)


def num(s):
    s = s.strip().replace(" ", "").replace(",", ".")
    return D(s)


def fmt(x):
    return format(x, "f") if isinstance(x, D) else str(x)


def enc(x):
    if isinstance(x, dict):
        return "{" + ", ".join(f"{enc(str(k))}: {enc(v)}" for k, v in x.items()) + "}"
    if isinstance(x, list):
        return "[" + ", ".join(enc(v) for v in x) + "]"
    if isinstance(x, D):
        return format(x, "f")
    if isinstance(x, bool):
        return "true" if x else "false"
    if x is None:
        return "null"
    s = str(x).replace("\\", "\\\\").replace('"', '\\"')
    return f'"{s}"'


def write_key(qid, as_of, known, key, steps, relations=None):
    lines = [f"# Key · {qid}", "", f"Written by `tools/keys_{qid.split('-')[0].lower()}.py`; don't edit.", "",
             "```yaml", f"question: {enc(qid)}", "edges: []", f"as_of: {enc(as_of)}", f"known_as_of: {enc(known)}",
             f"key: {enc(key)}"]
    if relations:
        lines.append(f"relations: {enc(relations)}")
    lines += ["```", "", "## Derivation", ""]
    lines += [f"{i}. {s}" for i, s in enumerate(steps, 1)]
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / f"{qid}.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def refusal(*missing):
    return {"kind": "refusal", "missing": list(missing)}


def add_months(day, n):
    """GFT-1 v1 .3: the same day number n months on, or that month's last day."""
    y, m = divmod(day.month - 1 + n, 12)
    y, m = day.year + y, m + 1
    return dt.date(y, m, min(day.day, calendar.monthrange(y, m)[1]))


# ---- the card's facts ----

need("**Helga Brant** (born 1941-07-03), Talia's grandmother", "close family (spouse, children, grandchildren, parents, grandparents, siblings)")
need("issued 2026-08-31 · received 2026-08-31", "handed over today", "2026-08-31, 2027-03-01, 2027-04-19 · known the same days · S2-D1, S2-D2, S2-D3")
POSSESSION = dt.date(2026, 8, 31)
need("sent and received 01/03/2027 08:14")
REPORTED = dt.date(2027, 3, 1)
need("decision 2027/SD/0388 · issued 2027-04-12 · received 2027-04-19", "the tax decision is delivered")
DELIVERED = dt.date(2027, 4, 19)
need("within 6 months of possession", "A decision assesses the tax at the amount points 1–2 give, due 14 days after the day it is delivered.",
     "No day moves for weekends or holidays.",
     "5 % of the value from 10 000,00 to 30 000,00 DUK and 9 % of the value above 30 000,00 DUK, rounded to whole dukats, half up",
     "by the same-numbered day six months on, or that month's last day if it has none",
     "at 10 % a year divided by 365", "both included", "rounded to whole dukats, half up, when the tax is paid",
     "No interest on interest.", "goes to the tax first, then to interest",
     "their own taxes, with the interest on them, are their own debts",
     "for joint money so used they owe the other spouse half",
     "A spouse's own debt takes their own money first, then joint money", "Common needs take joint money first.",
     "1 500,00 of Talia's own", "never fell below 9 000,00",
     "all other money out for common needs", "No rounding: half an ort stays.")
OWN_START = D("1500.00")

LIST = [(dt.date.fromisoformat(r[0]), D(r[3])) for r in record("S2-R2")]
BANK = {int(r[0]): (dt.date(int(r[1][6:]), int(r[1][3:5]), int(r[1][:2])), r[2], num(r[3]) if r[3] else None, num(r[4]))
        for r in record("S2-R1")}


def car_value(day):
    """GFT-1 v1 .1 and H-1 v1 .2: the latest list published on or before the day."""
    pub, val = max((p, v) for p, v in LIST if p <= day)
    return pub, val


steps0 = []
pub, value = car_value(POSSESSION)
steps0.append(f"taxed value (GFT-1 v1 .1): the list of {pub} (latest on or before possession {POSSESSION}, S2-R2) = {fmt(value)}")
assert "DUK 41,300.00" in TEXT and value == D("41300")
tax_raw = D("0.05") * (min(value, D(30000)) - D(10000)) + D("0.09") * max(value - D(30000), D(0))
TAX = tax_raw.quantize(D(1), rounding=ROUND_HALF_UP)
steps0.append(f"tax (GFT-1 v1 .2) = 5 % × ({fmt(min(value, D(30000)))} − 10000) + 9 % × ({fmt(value)} − 30000) = {fmt(tax_raw)}, "
              f"rounded to whole dukats half up = {fmt(TAX)}")
LAST_DAY = add_months(POSSESSION, 6)
steps0.append(f"last day to report (GFT-1 v1 .3): 6 months from {POSSESSION}; February 2027 has no 31st, so its last day "
              f"= {LAST_DAY}; not moved for the weekend (GFT-1 v1 .6)")
assert REPORTED > LAST_DAY
steps0.append(f"the report was received {REPORTED} (S2-D2), after {LAST_DAY}: Helga is close family "
              f"(grandmother, GFT-1 v1 .3) but the report was late, so the tax obligation stands from {POSSESSION} (GFT-1 v1 .1, .3)")

# S2-01: known as of 2027-03-15 — the decision hasn't been delivered, so the due date isn't known.
k1 = {"tax": TAX, "last day to report": LAST_DAY.isoformat(),
      "pay by": refusal("the day the Revenue Office's decision is delivered: the tax is due 14 days after it, "
                        "and on 2027-03-15 no decision has been delivered")}
write_key("S2-01", "2027-03-15", "2027-03-15", k1,
          steps0 + ["due date (GFT-1 v1 .4): 14 days after the decision's delivery; known as of 2027-03-15 no decision "
                    f"exists (S2-D3 is delivered {DELIVERED}), so the day is not settled"])

# The payments
DUE = DELIVERED + dt.timedelta(days=14)
d3, t3, a3, b3 = BANK[3]
assert -a3 == TAX and d3 == dt.date(2027, 6, 10) and b3 - a3 == BANK[2][3]  # row 3 follows row 2's closing balance
days = (d3 - DUE).days  # from DUE+1 to d3, both included
int_raw = TAX * D("0.10") * days / 365
INTEREST = int_raw.quantize(D(1), rounding=ROUND_HALF_UP)
d5, t5, a5, b5 = BANK[5]
assert -a5 == INTEREST
steps = list(steps0)
steps.append(f"due date (GFT-1 v1 .4): delivered {DELIVERED} (S2-D3) + 14 days = {DUE}")
steps.append(f"tax paid {d3} (S2-R1 row 3, {fmt(-a3)}; GFT-1 v1 .6): the whole tax")
steps.append(f"interest (GFT-1 v1 .5): days {DUE + dt.timedelta(days=1)} – {d3} both included = {days}; "
             f"{fmt(TAX)} × 10 % × {days} / 365 = {int_raw:.6f}…, rounded to whole dukats half up when the tax is paid = {fmt(INTEREST)}")
steps.append(f"interest paid {d5} (S2-R1 row 5, {fmt(-a5)})")
k2 = {"tax": D(0), "interest": INTEREST}
write_key("S2-02", "2027-06-30", "2027-07-05", k2,
          steps[:-1] + [f"owed to the Revenue Office on 2027-06-30: tax 0 (paid {d3}); interest {fmt(INTEREST)} "
                        f"(no row pays it until {d5}; no interest runs on interest, GFT-1 v1 .5)"])

# S2-03: worth
own_T = OWN_START
claim = D(0)  # Talia owes Oskar (MAR-1 v1 .2)
pay_steps = []
for no in (3, 5):
    d, t, a, b = BANK[no]
    p = -a
    from_own = min(p, own_T)
    from_joint = p - from_own
    own_T -= from_own
    claim += from_joint / 2
    pay_steps.append(f"S2-R1 row {no} ({d}, {fmt(p)}): Talia's own debt (MAR-1 v1 .1: her tax, and the interest on it), taken from her own money first "
                     f"(MAR-1 v1 .2): {fmt(from_own)} own, {fmt(from_joint)} joint; she owes Oskar half the joint part, "
                     f"{fmt(from_joint / 2)} (MAR-1 v1 .2) → her own money {fmt(own_T)}, owed to Oskar {fmt(claim)}")
    if no == 3:
        claim_after_3, own_after_3 = claim, own_T
claim_after_5 = claim

steps = list(steps0) + steps[len(steps0):len(steps0) + 4]
steps.append("between 2026-08-01 and 2027-07-31 the balance never fell below 9 000,00 and every other payment was for common "
             "needs, taken from joint money first (MAR-1 v1 .2), so Talia's own 1 500,00 is untouched until row 3")
steps += pay_steps
k3 = {}
points = [
    ("2026-12-31", BANK[1][3], OWN_START, TAX, D(0), D(0), "tax unpaid, owed since possession (H-1 v1 .2); no interest yet (due later)"),
    ("2027-06-30", BANK[4][3], own_after_3, D(0), INTEREST, claim_after_3, "tax paid; interest owed for the days run (H-1 v1 .2)"),
    ("2027-07-31", BANK[6][3], own_T, D(0), D(0), claim_after_5, "tax and interest paid"),
]
rel = []
for day, bal, ownT, tax_owed, int_owed, cl, why in points:
    pub, car = car_value(dt.date.fromisoformat(day))
    joint = bal - ownT
    talia = car + ownT + joint / 2 - tax_owed - int_owed - cl
    oskar = joint / 2 + cl
    k3[day] = {"Talia": talia, "Oskar": oskar}
    assert talia + oskar == car + bal - tax_owed - int_owed
    steps.append(f"{day}: car {fmt(car)} (list of {pub}); balance {fmt(bal)} (S2-R1), Talia's own {fmt(ownT)}, joint {fmt(joint)}; "
                 f"{why}. Talia = {fmt(car)} + {fmt(ownT)} + {fmt(joint)}/2 − tax {fmt(tax_owed)} − interest {fmt(int_owed)} "
                 f"− owed to Oskar {fmt(cl)} = {fmt(talia)}; Oskar = {fmt(joint)}/2 + {fmt(cl)} = {fmt(oskar)} (H-1 v1 .1–.2)")
write_key("S2-03", "2027-07-31", "2027-08-15", k3, steps,
          ["on each date Talia + Oskar = the car + the account's balance − what Talia owes the Revenue Office",
           "the debt between Talia and Oskar moves worth between them and never changes their total"])
print("S2: 3 keys written")
