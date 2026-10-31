#!/usr/bin/env python3
"""Keys for card S1. Reads the card's records, applies the card's rules, writes keys/S1/<qid>.md.

Run from the repository root: python3 -I -B engine/tests/cases/tools/keys_s1.py
Standard library only; every amount is an exact Decimal, rounded only where a rule of the card rounds.
"""
import pathlib
import re
from decimal import Decimal as D, ROUND_HALF_UP

BASE = pathlib.Path(__file__).resolve().parents[1]
CARD = BASE / "S1" / "card.md"
OUT = BASE / "keys" / "S1"
TEXT = CARD.read_text(encoding="utf-8")
ORT = D("0.01")


# ---- shared helpers (each key script carries its own copy) ----

def need(*parts):
    """The card must say this, word for word: ties a figure or rule the script uses to the card's text."""
    for p in parts:
        assert p in TEXT, f"the card doesn't say: {p!r}"


def record(rid):
    """The rows of the fenced record block whose first line starts with rid, as lists of fields."""
    for block in re.findall(r"```csv\n(.*?)\n```", TEXT, re.S):
        lines = block.splitlines()
        if lines[0].startswith(rid + " "):
            sep = ";" if ";" in lines[1] else "|"
            return [ln.split(sep) for ln in lines[2:]]
    raise KeyError(rid)


def num(s):
    """'-12 600,00' -> Decimal('-12600.00')."""
    s = s.strip().replace(" ", "").replace(".", "").replace(",", ".")
    return D(s)


def iso(d):
    """'14.03.2025' -> '2025-03-14'."""
    dd, mm, yy = d.split(".")
    return f"{yy}-{mm}-{dd}"


def half_up(x):
    return x.quantize(ORT, rounding=ROUND_HALF_UP)


def fmt(x):
    if isinstance(x, D):
        return format(x, "f")
    return str(x)


def enc(x):
    """The key in flow form: JSON with exact decimals."""
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


# ---- the card's facts ----

need("On 2025-03-01 it held 1 180,00 DUK, all of it joint money.")
OPEN_JOINT = D("1180.00")
need("each own-money part is rounded to whole orts, half up, and the rest is joint",
     "governs: payments made up to 2025-06-30", "governs: payments made from 2025-07-01",
     "each own-money part is rounded to whole orts, half up, and the rest is taken from joint money",
     "half the joint money", "half an ort stays half an ort")
V2_FROM = "2025-07-01"

ROWS = [(int(r[0]), iso(r[1]), r[2], num(r[3]), num(r[4])) for r in record("S1-R1")]
assert [r[0] for r in ROWS] == list(range(1, len(ROWS) + 1))

# What each row is, from the card's entries and descriptions (cited by row).
need("Our salaries come in on the 10th; the household order on the 20th pays rent, bills and shopping.",
     "S1-R1 row 9 — Mirela's language course, for her alone.",
     "S1-R1 row 12 — our summer holiday, for the two of us.",
     "S1-R1 row 23 — Joran's flying lessons, for him alone.",
     "S1-R1 row 31 — 1 500,00 taken out in cash and spent within the week. Neither of us remembers on what.",
     "Mirela's part of her aunt's estate is paid in (row 3).")
KIND = {}
for no, date, text, amt, bal in ROWS:
    if text.startswith("PAY "):
        KIND[no] = ("pay", "FAM-1 v1 .1: pay earned while married is joint")
    elif text.startswith("HOUSEHOLD ORDER"):
        KIND[no] = ("common", "household order: rent, bills, shopping (entries)")
    elif text == "INTEREST":
        KIND[no] = ("interest", "FAM-1 v1 .2")
KIND[3] = ("inherit", "FAM-1 v1 .1: inherited money is Mirela's own (S1-D1)")
KIND[9] = ("alone:Mirela", "Mirela's course, for her alone (entries)")
KIND[12] = ("common", "the holiday for the two of us: a common need (entries; FAM-2 v1 .1)")
KIND[23] = ("alone:Joran", "Joran's flying lessons, for him alone (entries)")
KIND[31] = ("unknown", "cash spent on something neither remembers (entries)")
assert set(KIND) == {r[0] for r in ROWS}
assert ROWS[2][3] == D("28000.00") and "DUK 28.000,00" in TEXT  # row 3 is the notary's sum (S1-D1)


def run(purpose_31=None):
    """Walk the rows. Returns the trace, the state after each row, and Mirela's part of each interest credit.
    purpose_31: None stops at row 31 (the hole); else 'common', 'alone:Mirela' or 'alone:Joran' (a branch)."""
    own = {"Mirela": D(0), "Joran": D(0)}
    joint = OPEN_JOINT
    trace, after, interest = [], {}, {}
    trace.append(f"S1-R1, 2025-03-01: opening balance {fmt(OPEN_JOINT)}, all joint (cast) → joint {fmt(joint)}, Mirela's own 0")
    for no, date, text, amt, bal in ROWS:
        kind, why = KIND[no]
        if kind == "unknown":
            if purpose_31 is None:
                trace.append(f"S1-R1 row {no} ({date}, {fmt(amt)}): {why}. FAM-2 v2 takes a payment by its purpose "
                             f"(.1 common needs, .2 one spouse alone); the purpose isn't in the dossier, so the split "
                             f"from this row on is not settled (the balance is: {fmt(bal)})")
                return trace, after, interest, False
            kind = purpose_31
        v = "v1" if date < V2_FROM else "v2"
        if kind == "pay":
            joint += amt
            note = f"+{fmt(amt)} joint ({why})"
        elif kind == "inherit":
            own["Mirela"] += amt
            note = f"+{fmt(amt)} Mirela's own ({why})"
        elif kind == "interest":
            total = own["Mirela"] + own["Joran"] + joint
            pm = half_up(amt * own["Mirela"] / total)
            pj = half_up(amt * own["Joran"] / total)
            jp = amt - pm - pj
            own["Mirela"] += pm
            own["Joran"] += pj
            joint += jp
            interest[date] = pm
            note = (f"interest {fmt(amt)} split by FAM-1 v1 .2 on the money just before it (Mirela's own "
                    f"{fmt(own['Mirela'] - pm)} of {fmt(total)}): Mirela's own part {fmt(amt)} × "
                    f"{fmt(own['Mirela'] - pm)} / {fmt(total)} rounded half up = {fmt(pm)}; joint {fmt(jp)}")
        elif kind == "common":
            pay = -amt
            if v == "v1":
                fj = min(pay, joint)
                rest = pay - fj
                town = own["Mirela"] + own["Joran"]
                assert rest <= town
                fm = rest if own["Joran"] == 0 else half_up(rest * own["Mirela"] / town)
                fjo = rest - fm
                note = (f"common need {fmt(pay)} under FAM-2 v1 .1 (joint first): {fmt(fj)} from joint"
                        + (f", the rest {fmt(rest)} from own money (Mirela {fmt(fm)})" if rest else ""))
            else:
                total = own["Mirela"] + own["Joran"] + joint
                fm = half_up(pay * own["Mirela"] / total)
                fjo = half_up(pay * own["Joran"] / total)
                fj = pay - fm - fjo
                note = (f"common need {fmt(pay)} under FAM-2 v2 .1 (in proportion just before it): Mirela's own part "
                        f"{fmt(pay)} × {fmt(own['Mirela'])} / {fmt(total)} rounded half up = {fmt(fm)}; joint {fmt(fj)}")
            joint -= fj
            own["Mirela"] -= fm
            own["Joran"] -= fjo
        elif kind.startswith("alone:"):
            who = kind.split(":")[1]
            pay = -amt
            fo = min(pay, own[who])
            fj = pay - fo
            own[who] -= fo
            joint -= fj
            note = (f"payment {fmt(pay)} for {who} alone under FAM-2 {v} .2: {fmt(fo)} from {who}'s own money, "
                    f"{fmt(fj)} from joint")
        else:
            raise ValueError(kind)
        assert joint >= 0 and own["Mirela"] >= 0 and own["Joran"] >= 0
        assert own["Mirela"] + own["Joran"] + joint == bal, (no, own, joint, bal)
        after[no] = (date, dict(own), joint)
        trace.append(f"S1-R1 row {no} ({date}): {note} → Mirela's own {fmt(own['Mirela'])}, joint {fmt(joint)}"
                     f" (sum = the row's balance {fmt(bal)})")
    return trace, after, interest, True


def state_on(after, day):
    """The state at the end of a day: after the last row dated on or before it."""
    nos = [no for no, (d, _, _) in after.items() if d <= day]
    return after[max(nos)]


TRACE, AFTER, INTEREST, DONE = run()
assert not DONE

# S1-01
d, own, joint = state_on(AFTER, "2025-06-30")
k1 = {"Mirela's own": own["Mirela"], "joint": joint}
steps = TRACE[:1 + max(no for no in AFTER if AFTER[no][0] <= "2025-06-30")]
steps.append(f"on 2025-06-30, after row 17: Mirela's own {fmt(own['Mirela'])}, joint {fmt(joint)}; "
             "every payment to this day falls under FAM-2 v1 (governs payments made up to 2025-06-30)")
write_key("S1-01", "2025-06-30", "2025-07-10", k1, steps,
          ["Mirela's own + joint = the account's balance on 2025-06-30 (S1-R1 row 17: 22 580,79)"])
assert own["Mirela"] + joint == D("22580.79")

# S1-02
k2 = {}
for day in ("2025-03-31", "2025-06-30", "2025-09-30"):
    k2[day] = INTEREST[day]
k2["2025-12-31"] = refusal("what the 1 500,00 taken out in cash on 2025-10-17 (S1-R1 row 31) was spent on: "
                           "the family's common needs, Mirela alone or Joran alone; it decides the split of the "
                           "account's money before the interest of 2025-12-31")
steps = list(TRACE)
for day in ("2025-03-31", "2025-06-30", "2025-09-30"):
    steps.append(f"Mirela's own part of the interest credited {day} = {fmt(INTEREST[day])} (the row's step above)")
branches = {}
for p in ("common", "alone:Mirela", "alone:Joran"):
    _, aft, intr, ok = run(p)
    assert ok
    branches[p] = (intr["2025-12-31"], state_on(aft, "2025-12-31"))
vals = sorted({b[0] for b in branches.values()})
assert len(vals) == 3, "the hole must change the answer"
steps.append("the interest of 2025-12-31 is not settled: by the purpose of row 31 Mirela's part would be "
             + "; ".join(f"{p.replace('alone:', 'for ').replace('common', 'common needs')} {fmt(b[0])}"
                         for p, b in branches.items())
             + " (shown only to prove the hole moves the answer; the dossier doesn't say which)")
write_key("S1-02", "2025-12-31", "2026-01-20", k2, steps)

# S1-03
k3 = {}
steps = list(TRACE)
for day in ("2025-06-30", "2025-09-30"):
    d, own, joint = state_on(AFTER, day)
    m = own["Mirela"] + joint / 2
    j = own["Joran"] + joint / 2
    k3[day] = {"Mirela": m, "Joran": j}
    steps.append(f"worth on {day} (H-1 v1 .1, nothing owed): Mirela = own {fmt(own['Mirela'])} + joint "
                 f"{fmt(joint)} / 2 = {fmt(m)}; Joran = own {fmt(own['Joran'])} + joint {fmt(joint)} / 2 = {fmt(j)}")
hole = ("what the 1 500,00 taken out in cash on 2025-10-17 (S1-R1 row 31) was spent on: the family's common needs, "
        "Mirela alone or Joran alone; it decides the split of the account's money from that day")
k3["2025-12-31"] = {"Mirela": refusal(hole), "Joran": refusal(hole)}
steps.append("worth on 2025-12-31: not settled, by row 31's purpose; the two together are the balance, 31 835,65. "
             "By purpose Mirela would be: "
             + "; ".join(f"{p.replace('alone:', 'for ').replace('common', 'common needs')} "
                         f"{fmt(b[1][1]['Mirela'] + b[1][2] / 2)}" for p, b in branches.items()))
write_key("S1-03", "2025-12-31", "2026-01-20", k3, steps,
          ["on each date Mirela + Joran = the account's balance that day (2025-06-30: 22 580,79; 2025-09-30: 27 037,60)"])
for day, bal in (("2025-06-30", D("22580.79")), ("2025-09-30", D("27037.60"))):
    assert k3[day]["Mirela"] + k3[day]["Joran"] == bal
print("S1: 3 keys written")
