#!/usr/bin/env python3
"""Keys for card S5. Reads the card's documents and records, applies the card's rules, writes keys/S5/<qid>.md.

Run from the repository root: python3 -I -B engine/tests/cases/tools/keys_s5.py
Standard library only; every amount is an exact Decimal, rounded only where a rule of the card rounds.
"""
import datetime as dt
import pathlib
import re
from decimal import Decimal as D, ROUND_HALF_UP

BASE = pathlib.Path(__file__).resolve().parents[1]
CARD = BASE / "S5" / "card.md"
OUT = BASE / "keys" / "S5"
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


def num(s):
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


def half_up(x):
    return x.quantize(ORT, rounding=ROUND_HALF_UP)


# ---- the card's facts ----

need("the house account holds 15 500,00: 6 400,00 of it Sanna's and 9 100,00 Ivo's")
OPEN = {"Sanna": D("6400.00"), "Ivo": D("9100.00")}
need("Each of us owns what each pays in, pay included; interest is ours in proportion to what each owns just before it is credited.",
     "**v1** · from 2025-11-01 to 2026-03-31",
     "A payment takes from each of us in proportion to what each owns just before it; rows of one day go in the statement's order.",
     "**v2** · from 2026-04-01 (the annex)",
     "A payment toward something we buy together, costs included, takes half from each of us; any other payment, as v1.",
     "in shares of what each paid toward its price and the costs of buying it, unless they agreed other shares in writing",
     "The plot counts at its price until first appraised, then at its latest appraisal on or before the day",
     "until the purchase, the deposit counts at what each of us paid of it",
     "Wherever a sum is split between us by proportion or share (interest, payments, the plot's value), Sanna's part is rounded to whole orts, half up, and Ivo's is the rest.",
     "price 96 000,00 DUK. Deposit 20 000,00 DUK paid today, counted toward the price.",
     "The buyers pay the notary's fee and the transfer tax, DUK 3.150,00.",
     "our pay on the 10th, the household order (rent, bills, shopping) on the 20th",
     "each of us moves own savings in", "rows 17–18 paying the rest of the price and the costs",
     "the house account receives only interest")
V2_FROM = dt.date(2026, 4, 1)
PURCHASE = dt.date(2026, 4, 28)

ROWS = [(int(r[0]), dt.datetime.strptime(r[1], "%d/%m/%Y").date(), r[2], num(r[3]), num(r[4])) for r in record("S5-R1")]
APPR = [(dt.date.fromisoformat(r[0]), num(r[2])) for r in record("S5-R2")]
OWNACC = {(r[0].split("'")[0], dt.date.fromisoformat(r[1])): num(r[2]) for r in record("S5-R3")}
assert ROWS[5][3] == D("-20000.00") and ROWS[16][3] == D("-76000.00") and ROWS[17][3] == D("-3150.00")


def kind(no, text):
    if text.startswith(("PAY S.ILVO", "FROM S.ILVO")):
        return "in:Sanna"
    if text.startswith(("PAY I.RASK", "FROM I.RASK")):
        return "in:Ivo"
    if text == "INTEREST":
        return "interest"
    if no in (6, 17, 18):
        return "plot"     # toward the plot we buy together (S5-D2, S5-D3)
    if text == "HOUSEHOLD ORDER":
        return "other"
    raise ValueError(text)


own = dict(OPEN)
paid = {"Sanna": D(0), "Ivo": D(0)}
AFTER = {}
S = [f"2026-01-01: Sanna {fmt(own['Sanna'])}, Ivo {fmt(own['Ivo'])} of the house account (S5-R1 entry; H-1 v1 .1)"]
for no, date, text, amt, bal in ROWS:
    k = kind(no, text)
    total = own["Sanna"] + own["Ivo"]
    if k.startswith("in:"):
        who = k[3:]
        own[who] += amt
        note = f"+{fmt(amt)} {who}'s (H-1 v1 .1)"
    elif k == "interest":
        s = half_up(amt * own["Sanna"] / total)
        own["Sanna"] += s
        own["Ivo"] += amt - s
        note = (f"interest {fmt(amt)} (H-1 v1 .1, H-3 v1 .3): Sanna {fmt(amt)} × {fmt(own['Sanna'] - s)} / {fmt(total)} rounded half up = {fmt(s)}, "
                f"Ivo {fmt(amt - s)}")
    else:
        pay = -amt
        if date >= V2_FROM and k == "plot":
            s = pay / 2
            note = f"payment {fmt(pay)} toward the plot under H-2 v2 .1: half each, {fmt(s)}"
        else:
            s = half_up(pay * own["Sanna"] / total)
            v = "v1 .1" if date < V2_FROM else "v2 .1 (as v1)"
            note = (f"payment {fmt(pay)} under H-2 {v}: Sanna {fmt(pay)} × {fmt(own['Sanna'])} / {fmt(total)} rounded half up (H-3 v1 .3) "
                    f"= {fmt(s)}, Ivo {fmt(pay - s)}")
        own["Sanna"] -= s
        own["Ivo"] -= pay - s
        if k == "plot":
            paid["Sanna"] += s
            paid["Ivo"] += pay - s
            note += f"; toward the plot so far: Sanna {fmt(paid['Sanna'])}, Ivo {fmt(paid['Ivo'])}"
    assert own["Sanna"] >= 0 and own["Ivo"] >= 0
    assert own["Sanna"] + own["Ivo"] == bal, (no, own, bal)
    AFTER[no] = (date, dict(own))
    S.append(f"S5-R1 row {no} ({date}): {note} → Sanna {fmt(own['Sanna'])}, Ivo {fmt(own['Ivo'])} (sum = the row's balance {fmt(bal)})")


def house_on(day):
    return AFTER[max(no for no, (d, _) in AFTER.items() if d <= day)][1]


TOTAL = paid["Sanna"] + paid["Ivo"]
assert TOTAL == D("99150.00")  # 96 000,00 price + 3 150,00 costs (S5-D3)

# S5-01
k1 = {"Sanna": paid["Sanna"], "Ivo": paid["Ivo"]}
s1 = S[:19] + [f"paid toward the plot, its price and costs (PRP-1 v1 .2; rows 6, 17, 18): Sanna {fmt(paid['Sanna'])}, Ivo {fmt(paid['Ivo'])}, "
               f"together {fmt(TOTAL)} = 96 000,00 + 3 150,00 (S5-D3)"]
write_key("S5-01", "2026-04-28", "2026-05-05", k1, s1, ["Sanna + Ivo = 99 150,00, the price and the costs"])


def plot_value(day):
    ap = [(d, v) for d, v in APPR if d <= day]
    if not ap:
        return None, D("96000.00")
    return max(ap)


def parts(day):
    ad, v = plot_value(day)
    s = half_up(v * paid["Sanna"] / TOTAL)
    return ad, v, s, v - s


# S5-02
k2 = {}
s2 = list(s1)
s2.append(f"shares (PRP-1 v1 .2, no other shares agreed): Sanna {fmt(paid['Sanna'])}/{fmt(TOTAL)}, Ivo {fmt(paid['Ivo'])}/{fmt(TOTAL)}; "
          "a rise in value falls on each in their share (PRP-1 v1 .2)")
for day in (dt.date(2026, 12, 31), dt.date(2027, 6, 30)):
    ad, v, s, i = parts(day)
    k2[day.isoformat()] = {"Sanna": s, "Ivo": i}
    s2.append(f"{day}: the plot at its latest appraisal on or before the day, {fmt(v)} ({ad}, S5-R2; H-3 v1 .2); Sanna "
              f"{fmt(v)} × {fmt(paid['Sanna'])} / {fmt(TOTAL)} = {v * paid['Sanna'] / TOTAL:.6f}…, rounded half up = {fmt(s)}; "
              f"Ivo the rest {fmt(i)} (H-3 v1 .3)")
write_key("S5-02", "2027-06-30", "2027-07-10", k2, s2)

# S5-03
k3 = {}
s3 = list(S) + s2[len(s1):len(s1) + 1]
# the deposit's parts: what each paid of row 6 (H-2 v1 .1 on the money just before it)
DEP = {"Sanna": None, "Ivo": None}
_own = dict(OPEN)
for no, date, text, amt, bal in ROWS:
    if no == 6:
        tot = _own["Sanna"] + _own["Ivo"]
        DEP["Sanna"] = half_up(-amt * _own["Sanna"] / tot)
        DEP["Ivo"] = -amt - DEP["Sanna"]
        break
    _own = AFTER[no][1]
for day in (dt.date(2026, 3, 31), dt.date(2026, 12, 31), dt.date(2027, 6, 30)):
    h = house_on(day)
    row = {}
    if day < PURCHASE:
        for who in ("Sanna", "Ivo"):
            row[who] = OWNACC[(who, day)] + h[who] + DEP[who]
        s3.append(f"{day}: before the purchase. Sanna = own account {fmt(OWNACC[('Sanna', day)])} (S5-R3) + house account {fmt(h['Sanna'])} "
                  f"+ her part of the deposit {fmt(DEP['Sanna'])} = {fmt(row['Sanna'])}; Ivo = {fmt(OWNACC[('Ivo', day)])} + {fmt(h['Ivo'])} "
                  f"+ {fmt(DEP['Ivo'])} = {fmt(row['Ivo'])} (H-3 v1 .1–.2)")
    else:
        ad, v, ps, pi = parts(day)
        row["Sanna"] = OWNACC[("Sanna", day)] + h["Sanna"] + ps
        row["Ivo"] = OWNACC[("Ivo", day)] + h["Ivo"] + pi
        s3.append(f"{day}: Sanna = own account {fmt(OWNACC[('Sanna', day)])} (S5-R3) + house account {fmt(h['Sanna'])} + her part of the plot "
                  f"{fmt(ps)} (of {fmt(v)}, appraised {ad}) = {fmt(row['Sanna'])}; Ivo = {fmt(OWNACC[('Ivo', day)])} + {fmt(h['Ivo'])} + "
                  f"{fmt(pi)} = {fmt(row['Ivo'])} (H-3 v1 .1–.3)")
        assert row["Sanna"] + row["Ivo"] == OWNACC[("Sanna", day)] + OWNACC[("Ivo", day)] + h["Sanna"] + h["Ivo"] + v
    k3[day.isoformat()] = row
write_key("S5-03", "2027-06-30", "2027-07-10", k3, s3,
          ["on each date Sanna + Ivo = both own accounts + the house account's balance + the plot (or, before the purchase, the deposit)"])
print("S5: 3 keys written")
