#!/usr/bin/env python3
"""Keys for card S6. Reads the card's documents and records, applies the card's rules, writes keys/S6/<qid>.md.

Run from the repository root: python3 -I -B engine/tests/cases/tools/keys_s6.py
Standard library only; every amount is an exact Decimal, rounded only where a rule of the card rounds.
"""
import datetime as dt
import pathlib
import re
from decimal import Decimal as D, ROUND_HALF_UP

BASE = pathlib.Path(__file__).resolve().parents[1]
CARD = BASE / "S6" / "card.md"
OUT = BASE / "keys" / "S6"
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


def dot_num(s):
    """'3,412.37' -> Decimal('3412.37')."""
    return D(s.strip().replace(",", "")) if s.strip() else D(0)


def comma_num(s):
    """'3 412,37' -> Decimal('3412.37')."""
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

need("On 2025-04-01 it held 2 600.00, all of it joint money. It pays no interest.")
OPEN = D("2600.00")
need("Pay either spouse receives while married is joint",
     "What a spouse had before marrying, inherits or is given is their own.",
     "each payment takes the oldest money still there, whoever owns it and whatever the payment is for",
     "Money that came in on one day is ordered as the statement lists it; the money in the account when the statement starts is older than any of its rows.",
     "became final on 31.10.2025", "pay either receives from that day is their own",
     "From that day each former spouse owns half of every former joint thing and of the former joint money",
     "Daria takes the car; Marek takes the former joint money in account 5150 2077; each keeps their own money.",
     "Values: the car 17 500.00 DUK; the former joint money, what the account holds of it today.",
     "pays the other the difference, rounded to whole orts, half up",
     "signed 2025-12-15", "Nothing is paid out of the joint account between 2025-10-21 and 2025-12-17.",
     "her son Marek Holm receives DUK 18 500.00", "bought with joint money",
     "The car counts at its latest valuation on or before the day.", "half an ort stays half an ort")
FINAL = dt.date(2025, 10, 31)
SIGNED = dt.date(2025, 12, 15)
CAR_AGREED = D("17500.00")

ROWS = []
for r in record("S6-R1"):
    no, date, text, debit, credit, bal = r
    ROWS.append((int(no), dt.date.fromisoformat(date), text, dot_num(credit) - dot_num(debit), dot_num(bal)))
R2 = [(dt.datetime.strptime(r[0], "%d.%m.%Y").date(), r[1], comma_num(r[2]), comma_num(r[3])) for r in record("S6-R2")]
CARV = [(dt.date.fromisoformat(r[0]), comma_num(r[2])) for r in record("S6-R3")]
b = OPEN
for no, date, text, amt, bal in ROWS:
    b += amt
    assert b == bal, no
b = D(0)
for d, t, a, bal in R2:
    b += a
    assert b == bal, d


def car_on(day):
    return max((d, v) for d, v in CARV if d <= day)


def owner_in(no, date, text):
    if text.startswith("NOTARY P.LUND"):
        return "Marek", "inherited: his own (MRG-1 v1 .1, S6-D1)"
    if text.startswith("SALARY"):
        who = "Daria" if "D.HOLM" in text else "Marek"
        if date >= FINAL:
            return who, f"{who}'s pay received after the divorce became final: {who}'s own (DIV-1 v1 .1)"
        return "joint", "pay while married: joint (MRG-1 v1 .1)"
    raise ValueError(text)


queue = [["opening", "joint", OPEN]]
S = [f"S6-R1, 2025-04-01: {fmt(OPEN)} in the account, all joint (cast), older than every row (MRG-1 v1 .2)"]
SNAP = {}


def sums(q):
    out = {"joint": D(0), "Marek": D(0), "Daria": D(0)}
    for _, o, a in q:
        out[o] += a
    return out


def qtext(q):
    return ", ".join(f"{src} {o} {fmt(a)}" for src, o, a in q)


for no, date, text, amt, bal in ROWS:
    if date >= SIGNED:
        break  # from the settlement on, everything in the account is Marek's (S6-D3 .1); handled below
    if amt > 0:
        o, why = owner_in(no, date, text)
        queue.append([f"row {no}", o, amt])
        S.append(f"S6-R1 row {no} ({date}): +{fmt(amt)}, {why}")
    else:
        need_ = -amt
        took = []
        while need_ > 0:
            take = min(need_, queue[0][2])
            took.append(f"{fmt(take)} of {queue[0][0]} ({queue[0][1]})")
            queue[0][2] -= take
            need_ -= take
            if queue[0][2] == 0:
                queue.pop(0)
        S.append(f"S6-R1 row {no} ({date}): pays {fmt(-amt)}, oldest money first (MRG-1 v1 .2): " + "; ".join(took)
                 + f" → left: {qtext(queue)}")
    s = sums(queue)
    assert s["joint"] + s["Marek"] + s["Daria"] == bal
    SNAP[no] = (date, s)


def state(day):
    return SNAP[max(no for no, (d, _) in SNAP.items() if d <= day)][1]


# S6-01: the day before the judgment became final, while the marriage still stands
need("as of 2025-10-30 · known as of 2025-11-10 · On 2025-10-30, the day before the divorce became final")
EVE = FINAL - dt.timedelta(days=1)
s_eve = state(EVE)
s_final = state(FINAL)
assert s_eve == s_final  # no row between 2025-10-21 and 2025-10-31
k1 = {"Marek's own": s_eve["Marek"], "joint": s_eve["joint"]}
cut = 1 + sum(1 for no, (d, _) in SNAP.items() if d <= EVE)
s1 = S[:cut] + [f"on {EVE}, the day before the judgment became final (S6-D2), the marriage still stands and the money in the account is joint "
                f"or own (MRG-1 v1 .1): Marek's own {fmt(s_eve['Marek'])}, joint {fmt(s_eve['joint'])}; together the balance "
                f"{fmt(s_eve['Marek'] + s_eve['joint'])} (S6-R1 row 23, the last row before that day)"]
write_key("S6-01", EVE.isoformat(), "2025-11-10", k1, s1, ["Marek's own + joint = the account's balance on 2025-10-30 (40,100.85)"])
assert s_eve["Marek"] + s_eve["joint"] == D("40100.85")

# S6-02
s_sign = state(SIGNED)
assert s_sign["joint"] == s_final["joint"]  # nothing paid out between the judgment and the signing
JM = s_sign["joint"]
total = CAR_AGREED + JM
half = total / 2
assert JM > half
PAY = half_up(JM - half)
row26 = [r for r in ROWS if r[0] == 26][0]
assert row26[3] == -PAY and [a for d, t, a, _ in R2 if t.startswith("FROM M.HOLM")] == [PAY]
s2 = S + [f"on {SIGNED} the former joint money in the account is {fmt(JM)} (Marek's own rows 24–25 are not part of it, DIV-1 v1 .1)",
          f"at the settlement's values (S6-D3 .2): car {fmt(CAR_AGREED)} + money {fmt(JM)} = {fmt(total)}; half = {fmt(half)}",
          f"Daria takes the car, {fmt(CAR_AGREED)}; Marek takes the money, {fmt(JM)}, more than half; he pays Daria {fmt(JM)} − {fmt(half)} = "
          f"{fmt(JM - half)}, rounded to whole orts half up = {fmt(PAY)} (S6-D3 .3), paid {row26[1]} (S6-R1 row 26, S6-R2)"]
write_key("S6-02", "2025-12-31", "2026-01-10", {"paid by": "Marek", "paid to": "Daria", "amount": PAY}, s2)

# S6-03
k3 = {}
s3 = list(s2)
for day in (dt.date(2025, 6, 30), FINAL):
    st = state(day)
    cd, cv = car_on(day)
    daria = st["joint"] / 2 + cv / 2 + st["Daria"]
    marek = st["Marek"] + st["joint"] / 2 + cv / 2
    k3[day.isoformat()] = {"Daria": daria, "Marek": marek}
    s3.append(f"{day}: car {fmt(cv)} (valued {cd}, H-1 v1 .2); in the account joint {fmt(st['joint'])}, Marek's own {fmt(st['Marek'])}. "
              f"Daria = {fmt(st['joint'])}/2 + {fmt(cv)}/2 = {fmt(daria)}; Marek = {fmt(st['Marek'])} + {fmt(st['joint'])}/2 + {fmt(cv)}/2 = "
              f"{fmt(marek)} (H-1 v1 .1; from {FINAL} the halves are the former joint property's, DIV-1 v1 .1)")
day = dt.date(2025, 12, 31)
cd, cv = car_on(day)
d_acct = max((d, bal) for d, _, _, bal in R2 if d <= day)[1]
m_acct = [r for r in ROWS if r[1] <= day][-1][4]
k3[day.isoformat()] = {"Daria": cv + d_acct, "Marek": m_acct}
s3.append(f"{day}: after the settlement (DIV-1 v1 .2) Daria alone owns the car, {fmt(cv)} (valued {cd}), and her account {fmt(d_acct)} (S6-R2): "
          f"{fmt(cv + d_acct)}; Marek alone owns all the money in account 5150 2077, {fmt(m_acct)} (S6-R1 row 26)")
write_key("S6-03", "2025-12-31", "2026-01-10", k3, s3,
          ["on 2025-06-30 and 2025-10-31, Daria + Marek = the account's balance + the car"])
for dd, bal in (("2025-06-30", D("30355.40")), ("2025-10-31", D("40100.85"))):
    assert k3[dd]["Daria"] + k3[dd]["Marek"] == bal + car_on(dt.date.fromisoformat(dd))[1]
print("S6: 3 keys written")
