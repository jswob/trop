#!/usr/bin/env python3
"""Keys for card S3. Reads the card's records, applies the card's rules, writes keys/S3/<qid>.md.

Run from the repository root: python3 -I -B engine/tests/cases/tools/keys_s3.py
Standard library only; every amount is an exact Decimal, rounded only where a rule of the card rounds.
"""
import calendar
import datetime as dt
import pathlib
import re
from decimal import Decimal as D, ROUND_HALF_UP

BASE = pathlib.Path(__file__).resolve().parents[1]
CARD = BASE / "S3" / "card.md"
OUT = BASE / "keys" / "S3"
TEXT = CARD.read_text(encoding="utf-8")
ORT = D("0.01")


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


def comma_num(s):
    return D(s.strip().replace(" ", "").replace(",", "."))


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


def half_up(x, q=ORT):
    return x.quantize(q, rounding=ROUND_HALF_UP)


def add_years(day, n):
    """CGT-1 v1 .4: the same day number n years on, or that month's last day."""
    y = day.year + n
    return dt.date(y, day.month, min(day.day, calendar.monthrange(y, day.month)[1]))


# ---- the card's facts ----

need("each purchase is a batch, acquired on its day, costing the price paid plus the purchase fee",
     "Units received as a gift form a batch with the giver's acquisition day and the giver's cost for them",
     "Units taken from a batch cost the batch's remaining cost times the units taken divided by its remaining units, rounded to whole orts, half up; the units left keep the rest of the cost",
     "A sale takes units from the seller's batches oldest first, by acquisition day.",
     "each share but the youngest batch's is rounded to whole orts, half up, and the youngest batch takes the rest",
     "A part held more than 3 years on the day of sale is left out, gain or loss",
     "rounded to whole dukats, half up; a negative sum counts as 0",
     "The tax is 19 % of the taxed gain, rounded to whole dukats, half up",
     "the fund's price published for the day, or the latest published before it",
     "the tax the year's sales up to that day would give if the year ended then",
     "Emil holds 70 units and 4 380,00 cash", "Emil's cash account had no rows in 2026",
     "a gift from the other spouse included")
EMIL_CASH = D("4380.00")
ACCT = {"N-4471": "Nadia", "E-2290": "Emil"}

TX = []
for r in record("S3-R1"):
    no, date, acct, typ, units, price, fee, amount = r
    TX.append((int(no), dt.datetime.strptime(date, "%d-%m-%Y").date(), ACCT[acct], typ, D(units),
               D(price) if price else None, D(fee) if fee else None, D(amount) if amount else None))
PRICES = [(dt.date.fromisoformat(d), D(p)) for d, p in record("S3-R3")]
CASH = [(dt.datetime.strptime(r[0], "%d.%m.%Y").date(), r[1], comma_num(r[2]) if r[2] else None, comma_num(r[3]))
        for r in record("S3-R2")]
bal = CASH[0][3]
for d, t, a, b in CASH[1:]:  # the cash rows chain
    if a is not None:
        bal += a
    assert bal == b, (d, bal, b)


def price_on(day):
    return max((d, p) for d, p in PRICES if d <= day)


class Batch:
    def __init__(self, owner, acq, units, cost, src):
        self.owner, self.acq, self.units, self.cost, self.src = owner, acq, units, cost, src

    def take(self, n):
        """CGT-1 v1 .2: the cost of n units taken from what is left of the batch."""
        c = self.cost if n == self.units else half_up(self.cost * n / self.units)
        self.units -= n
        self.cost -= c
        return c


batches = []
STEPS = []
SALES = []  # (date, parts): parts = [(batch acq, units, proceeds share, fee share, cost, gain, left_out)]
SNAP = {}   # the batches after each transaction, by date


def holding(owner):
    return sorted([b for b in batches if b.owner == owner and b.units > 0], key=lambda b: b.acq)


for no, date, who, typ, units, price, fee, amount in TX:
    if typ == "BUY":
        cost = units * price + fee
        assert amount == -cost
        batches.append(Batch(who, date, units, cost, f"S3-R1 row {no}"))
        STEPS.append(f"S3-R1 row {no} ({date}): {who} buys {fmt(units)} units: a batch acquired {date}, cost "
                     f"{fmt(units)} × {fmt(price)} + fee {fmt(fee)} = {fmt(cost)} (CGT-1 v1 .1)")
    elif typ.startswith("TRANSFER OUT"):
        n = -units
        src = holding(who)
        assert len(src) == 1  # the giver has one batch: no rule is needed for which of the giver's units a gift takes
        b = src[0]
        c = b.take(n)
        gift = (b.acq, n, c)
        STEPS.append(f"S3-R1 row {no} ({date}): {who} gives {fmt(n)} units of the batch acquired {b.acq}; their cost "
                     f"(CGT-1 v1 .2) = {fmt(b.cost + c)} × {fmt(n)} / {fmt(b.units + n)} rounded half up = {fmt(c)}; "
                     f"{who} keeps {fmt(b.units)} units at {fmt(b.cost)}; no tax on a gift (CGT-1 v1 .1)")
    elif typ.startswith("TRANSFER IN"):
        acq, n, c = gift
        assert n == units
        batches.append(Batch(who, acq, n, c, f"S3-R1 row {no}"))
        STEPS.append(f"S3-R1 row {no} ({date}): {who} receives the {fmt(n)} units as a batch with the giver's day {acq} "
                     f"and cost {fmt(c)} (CGT-1 v1 .1); hers alone (S3-D1, SEP-1 v1 .1)")
    elif typ == "SELL":
        n = -units
        assert amount == n * price - fee
        need_n, parts = n, []
        for b in holding(who):  # CGT-1 v1 .3: oldest first
            if need_n == 0:
                break
            k = min(need_n, b.units)
            parts.append([b, k])
            need_n -= k
        assert need_n == 0
        youngest = max(range(len(parts)), key=lambda i: parts[i][0].acq)
        shares = [None] * len(parts)
        for i, (b, k) in enumerate(parts):
            if i != youngest:
                shares[i] = half_up(fee * k / n)
        shares[youngest] = fee - sum(s for s in shares if s is not None)
        rows = []
        STEPS.append(f"S3-R1 row {no} ({date}): {who} sells {fmt(n)} units at {fmt(price)}, fee {fmt(fee)}; proceeds "
                     f"{fmt(n)} × {fmt(price)} − {fmt(fee)} = {fmt(amount)}; oldest batches first (CGT-1 v1 .3)")
        for i, (b, k) in enumerate(parts):
            c = b.take(k)
            gross = k * price
            gain = gross - shares[i] - c
            ends = add_years(b.acq, 3)
            left_out = date > ends
            rows.append((b.acq, k, gross, shares[i], c, gain, left_out))
            STEPS.append(f"  part from the batch acquired {b.acq}: {fmt(k)} units; fee share "
                         + ("(youngest batch: the rest) " if i == youngest else f"{fmt(fee)} × {fmt(k)} / {fmt(n)} rounded half up ")
                         + f"= {fmt(shares[i])}; cost (CGT-1 v1 .2) {fmt(c)}; gain {fmt(k)} × {fmt(price)} − {fmt(shares[i])} − {fmt(c)} "
                         f"= {fmt(gain)} (CGT-1 v1 .3); its 3 years end {ends}: "
                         + ("the sale is after that day, so it is left out (CGT-1 v1 .4)" if left_out else "not more than 3 years, so it is taxed (CGT-1 v1 .4)"))
        SALES.append((date, rows))
    else:
        raise ValueError(typ)
    SNAP[date] = [(b.owner, b.acq, b.units, b.cost) for b in batches]


def tax_up_to(day):
    taxed = sum((g for d, rows in SALES if d <= day and d.year == day.year for (_, _, _, _, _, g, lo) in rows if not lo), D(0))
    untaxed = sum((g for d, rows in SALES if d <= day and d.year == day.year for (_, _, _, _, _, g, lo) in rows if lo), D(0))
    tg = max(half_up(taxed, D(1)), D(0))
    tax = half_up(tg * D("0.19"), D(1))
    return taxed, tg, untaxed, tax


def units_on(owner, day):
    snaps = [d for d in SNAP if d <= day]
    return [(acq, u, c) for o, acq, u, c in SNAP[max(snaps)] if o == owner and u > 0]


# S3-01
taxed, tg, untaxed, tax = tax_up_to(dt.date(2026, 12, 31))
k1 = {"taxed gain": tg, "untaxed gain": untaxed, "tax": tax}
s1 = list(STEPS)
s1.append(f"2026: gains not left out sum to {fmt(taxed)}, rounded to whole dukats half up = {fmt(tg)} (CGT-1 v1 .5); "
          f"left-out gains and losses sum to {fmt(untaxed)}")
s1.append(f"tax = 19 % × {fmt(tg)} = {fmt(tg * D('0.19'))}, rounded to whole dukats half up = {fmt(tax)} (CGT-1 v1 .5), due by 2027-04-30")
write_key("S3-01", "2026-12-31", "2027-01-20", k1, s1)

# S3-02
left = units_on("Nadia", dt.date(2026, 12, 31))
k2 = {acq.isoformat(): {"units": u, "cost": c} for acq, u, c in sorted(left)}
s2 = list(STEPS)
s2.append("left on 2026-12-31: " + "; ".join(f"{fmt(u)} units acquired {acq} at cost {fmt(c)}" for acq, u, c in sorted(left)))
write_key("S3-02", "2026-12-31", "2027-01-20", k2, s2,
          ["the cost of the units left + the cost of the units sold = the cost of all the units Nadia acquired"])
all_cost = D("4135.00") + D("3540.00") + D("3185.00")
gift_cost = [r for d, rows in SALES for r in rows if r[0] == dt.date(2022, 11, 15)][0][4]
sold_cost = sum(r[4] for d, rows in SALES for r in rows)
assert sum(c for _, _, c in left) + sold_cost == all_cost + gift_cost

# S3-03
k3 = {}
s3 = list(STEPS)
for day in (dt.date(2026, 6, 30), dt.date(2026, 9, 30), dt.date(2026, 12, 31)):
    pd, p = price_on(day)
    cash_n = max((d, b) for d, _, _, b in CASH if d <= day)[1]
    un = sum(u for _, u, _ in units_on("Nadia", day))
    ue = sum(u for _, u, _ in units_on("Emil", day))
    _, tgd, _, taxd = tax_up_to(day)
    nadia = cash_n + un * p - taxd
    emil = EMIL_CASH + ue * p
    k3[day.isoformat()] = {"Nadia": nadia, "Emil": emil}
    s3.append(f"{day}: price {fmt(p)} (published {pd}; H-1 v1 .1). Nadia = cash {fmt(cash_n)} (S3-R2) + {fmt(un)} units × {fmt(p)} "
              f"− tax {fmt(taxd)} (the year's sales to this day: taxed gain {fmt(tgd)}; H-1 v1 .2) = {fmt(nadia)}; "
              f"Emil = cash {fmt(EMIL_CASH)} + {fmt(ue)} units × {fmt(p)} = {fmt(emil)} (H-1 v1 .1)")
write_key("S3-03", "2026-12-31", "2027-01-20", k3, s3)
print("S3: 3 keys written")
