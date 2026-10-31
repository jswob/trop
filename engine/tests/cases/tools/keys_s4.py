#!/usr/bin/env python3
"""Keys for card S4. Reads the card's documents and records, applies the card's rules, writes keys/S4/<qid>.md.

Run from the repository root: python3 -I -B engine/tests/cases/tools/keys_s4.py
Standard library only; every amount is an exact Decimal, rounded only where a rule of the card rounds.

Cases 1.4.0 (ERRATA ADD-10, the user's G2 Q-1): the court's ruling on the will's reason is a fact the dossier doesn't
hold, so S4-03's claims and S4-04's slots after the death are refusals naming it; the by-case figures this script
computes become the pinned twin's keys, S4-03P and S4-04P (the text run with the ruling pinned by `with`).
"""
import datetime as dt
import pathlib
import re
from decimal import Decimal as D, ROUND_HALF_UP

BASE = pathlib.Path(__file__).resolve().parents[1]
CARD = BASE / "S4" / "card.md"
OUT = BASE / "keys" / "S4"
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


def day(s):
    return dt.datetime.strptime(s, "%d.%m.%Y").date()


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


def by_case(holds, untrue):
    return {"kind": "by-case", "cases": {"the will's reason is true": holds, "the will's reason is untrue": untrue},
            "settled by": "the court's ruling on Lea's suit (hearing set for 2028-02-15)"}


# cases 1.4.0 (G2 Q-1): the ruling is a missing fact, not a reading of a rule; the card's opening clause asks for it named
def refusal(*missing):
    return {"kind": "refusal", "missing": list(missing)}


def ruling(what):
    return refusal("the court's ruling on Lea's suit: whether the will's reason for leaving Lea out is true (SUC-1 v1 .6); "
                   f"no ruling yet (hearing set for 2028-02-15); it decides {what}")


def pinned(case):
    """A pinned twin's table: the by-case's cases as plain values, one per pinned answer of the ruling."""
    return dict(case["cases"])


# ---- the card's facts ----

need("died 2027-02-14", "no marriage contract: SPS governs", "— Edvin dies.")
DEATH = dt.date(2027, 2, 14)
need("Pay and pensions received while married are joint, in whatever account",
     "a flat in Torvin (bought 1991 with joint money)", "a cottage (Edvin's own, inherited 2004)",
     "holding only pension paid in during the marriage",
     "half of each joint thing and of the joint money in each account becomes the survivor's own, half the estate's; what the survivor receives afterwards is their own",
     "My heirs are my wife Ruta and my son Pavo, in equal shares. I leave nothing to my daughter Lea",
     "with no will: spouse and children, equally",
     "each in their share of every thing, sum and debt in it",
     "The funeral is an estate debt; an estate debt paid from an account also holding other money takes the estate's money first.",
     "governs: deaths up to 2027-06-30", "governs: deaths from 2027-07-01 · As v1, but point 4 gives two thirds instead of half.",
     "half their no-will share of the estate's net value at the death (sworn appraiser's values for that day, plus money, less debts, the funeral included), rounded to whole orts, half up; the claim arises at the death",
     "the first-named heir's part rounded to whole orts, half up, the other's the rest",
     "No claim if the reason the will gives for leaving the child out is true", "no ruling yet (hearing 2028-02-15)",
     "that half of the flat 210.000,00 DUK, the cottage 160.000,00 DUK, the money as the estate holds it today",
     "who takes more pays the other the difference, rounded to whole orts, half up",
     "Ruta takes the estate's half of the flat and all its money; Pavo, the cottage",
     "deed of division 311/2027 · issued 2027-09-20", "Nobody owes anything but as the entries show.",
     "joint property half to each spouse", "latest valuation for a day on or before the day asked",
     "No rounding: half an ort stays.")
DEED_DAY = dt.date(2027, 9, 20)
DEED_FLAT_HALF, DEED_COTTAGE = D("210000.00"), D("160000.00")

R1 = {int(r[0]): (day(r[1]), r[2], num(r[3]) if r[3] else None, num(r[4])) for r in record("S4-R1")}
bal = R1[0][3]
for no in sorted(R1)[1:]:
    bal += R1[no][2]
    assert bal == R1[no][3], no
VALS = [(r[0], day(r[1]), num(r[2]), r[3]) for r in record("S4-R2")]
R3 = [(r[0], day(r[1]), num(r[2])) for r in record("S4-R3")]


def valuation(item, on):
    """H-1 v1 .2: the latest valuation for a day on or before the day asked."""
    return max((d, v, by) for i, d, v, by in VALS if i == item and d <= on)


def kept(acct_prefix, on):
    return [b for a, d, b in R3 if a.startswith(acct_prefix) and d == on][0]


J0 = R1[0][3]                                   # joint account, end of 2027-02-13; no rows until row 1
P0 = kept("2208 3390", dt.date(2027, 2, 13))
assert P0 == kept("2208 3390", dt.date(2027, 10, 4)) == R1[2][2]  # untouched until it is closed into row 2
FUNERAL = -R1[1][2]
assert R1[1][0] > DEATH and "FUNERAL" in R1[1][1]

S = []
S.append(f"at death {DEATH} (SPS-1 v1 .2): joint account {fmt(J0)} (S4-R1 row 0, no rows until 2027-02-24) → Ruta's own {fmt(J0 / 2)}, "
         f"estate {fmt(J0 / 2)}; Edvin's account {fmt(P0)} (S4-R3), all pension paid in during the marriage, so joint whatever its name "
         f"(SPS-1 v1 .1) → Ruta's own {fmt(P0 / 2)}, estate {fmt(P0 / 2)}; the flat is joint → half Ruta's own, half the estate's; "
         "the cottage is Edvin's own (SPS-1 v1 .1) → the estate's")
S.append("heirs (SUC-1 v1 .1, S4-D1): Ruta 1/2, Pavo 1/2; from the day of death they own the estate together in those shares (SUC-1 v1 .1)")
est_money_joint = J0 / 2 - FUNERAL
assert est_money_joint >= 0
S.append(f"funeral {fmt(FUNERAL)} paid {R1[1][0]} (S4-R1 row 1): an estate debt, taken from the estate's money in the account first "
         f"(SUC-1 v1 .2) → the estate's money there {fmt(J0 / 2)} − {fmt(FUNERAL)} = {fmt(est_money_joint)}; Ruta's {fmt(J0 / 2)} untouched")
S_BASE = list(S)

# S4-01: the estate's net value for the day of death
flat_d = [v for i, d, v, by in VALS if i == "flat" and d == DEATH and by.startswith("sworn appraiser")][0]
cot_d = [v for i, d, v, by in VALS if i == "cottage" and d == DEATH and by.startswith("sworn appraiser")][0]
assert flat_d == D("412000.00") and cot_d == D("156500.00")
NET = flat_d / 2 + cot_d + J0 / 2 + P0 / 2 - FUNERAL
s1 = S_BASE + [f"net value for {DEATH} (SUC-1 v1 .4; the sworn appraiser's values for that day, S4-R2): half the flat {fmt(flat_d)}/2 + cottage {fmt(cot_d)} + money "
               f"{fmt(J0 / 2)} + {fmt(P0 / 2)} − funeral {fmt(FUNERAL)} = {fmt(NET)}"]
write_key("S4-01", "2027-02-14", "2027-04-10", NET, s1)

# S4-03: Lea's claim (by-case on the pending ruling)
assert DEATH <= dt.date(2027, 6, 30)  # SUC-1 v1 governs; v2 governs only deaths from 2027-07-01
CLAIM = half_up(NET / 2 / 3)
RUTA_PART = half_up(CLAIM / 2)
PAVO_PART = CLAIM - RUTA_PART
s3 = s1 + [f"SUC-1 v1 governs (deaths up to 2027-06-30; v2, two thirds, governs deaths from 2027-07-01): with no will Lea would have 1/3 "
           f"(spouse and two children, SUC-1 v1 .1); her claim = 1/2 × 1/3 × {fmt(NET)} = {NET / 6:.6f}…, rounded to whole orts "
           f"half up = {fmt(CLAIM)}, arising {DEATH}",
           f"owed in the heirs' shares (SUC-1 v1 .5): Ruta, named first, {fmt(CLAIM)}/2 = {fmt(CLAIM / 2)} rounded half up = "
           f"{fmt(RUTA_PART)}; Pavo the rest {fmt(PAVO_PART)}",
           "the will gives a reason for leaving Lea out; if it is true she has no claim (SUC-1 v1 .6); Lea disputes it and the court "
           "has not ruled (entry of 2027-05-20): the claim is one value per branch of the pending ruling"]
k3 = by_case({"from Ruta": D(0), "from Pavo": D(0)}, {"from Ruta": RUTA_PART, "from Pavo": PAVO_PART})
# cases 1.4.0 (G2 Q-1): S4-03 names the missing ruling; the by-case figures are the pinned twin S4-03P's
need("If the dossier doesn't settle an answer, name what's missing instead of a value.",
     "Lea sues Ruta and Pavo, calling the will's reason untrue; no ruling yet (hearing 2028-02-15)")
WHY_REFUSE = ("Lea disputes it and the court has not ruled (entry of 2027-05-20); the ruling is a fact the dossier doesn't hold, not a "
              "way of reading a rule, and the card "
              "says: if the dossier doesn't settle an answer, name what's missing instead of a value. So each figure that turns on it "
              "names the missing ruling (cases 1.4.0, G2 Q-1)")
s3r = s3[:-1] + ["the will gives a reason for leaving Lea out; if it is true she has no claim (SUC-1 v1 .6); " + WHY_REFUSE + f"; the figures of steps {len(s3) - 2}–{len(s3) - 1} hold once "
                 "it is known, and are the pinned twin's, S4-03P"]
write_key("S4-03", "2027-12-31", "2027-12-31",
          {"from Ruta": ruling("Lea's claim from Ruta"), "from Pavo": ruling("Lea's claim from Pavo")}, s3r)
PIN = ("pinned twin of S4-03 (cases 1.4.0, G2 Q-1): the text run with the ruling pinned by `with` (L §5.18), once as true and once as "
       "untrue; each pinned answer is one plain value")
write_key("S4-03P", "2027-12-31", "2027-12-31", pinned(k3),
          s3[:-1] + ["the will gives a reason for leaving Lea out; if it is true she has no claim (SUC-1 v1 .6): with the reason pinned "
                     f"true, 0 from each; pinned untrue, steps {len(s3) - 2}–{len(s3) - 1}'s figures", PIN])

# S4-02: the division's payment
est_money = est_money_joint + P0 / 2
total = DEED_FLAT_HALF + DEED_COTTAGE + est_money
half = total / 2
ruta_takes = DEED_FLAT_HALF + est_money
pavo_takes = DEED_COTTAGE
assert ruta_takes > half
PAY = half_up(ruta_takes - half)
assert R1[3][2] == -PAY and "P.MALEK" in R1[3][1]
s2 = S_BASE + [f"the estate's money on the deed's day {DEED_DAY}: in the joint account {fmt(est_money_joint)} (no rows between the funeral "
               f"and the deed) + half of Edvin's account {fmt(P0 / 2)} = {fmt(est_money)}",
               f"at the deed's values (S4-D2 .2): {fmt(DEED_FLAT_HALF)} + {fmt(DEED_COTTAGE)} + {fmt(est_money)} = {fmt(total)}; "
               f"each heir's part {fmt(half)} (S4-D2 .3)",
               f"Ruta takes {fmt(DEED_FLAT_HALF)} + {fmt(est_money)} = {fmt(ruta_takes)}; Pavo takes {fmt(pavo_takes)}; Ruta takes more "
               f"and pays Pavo {fmt(ruta_takes - half)}, rounded to whole orts half up = {fmt(PAY)}, by 2027-10-31; "
               f"paid {R1[3][0]} (S4-R1 row 3)"]
write_key("S4-02", "2027-10-31", "2027-11-05", {"paid by": "Ruta", "paid to": "Pavo", "amount": PAY}, s2)

# S4-04: worth
s4 = s2[:len(S_BASE)] + s2[len(S_BASE):] + s3[len(s1):]
k4 = {}
d0 = dt.date(2027, 2, 13)
(fd, fv, _), (cd, cv, _) = valuation("flat", d0), valuation("cottage", d0)
edvin = fv / 2 + cv + J0 / 2 + P0 / 2
ruta = fv / 2 + J0 / 2 + P0 / 2
LEA0, PAVO0 = kept("Lea's own", d0), kept("Pavo's own", d0)
k4["2027-02-13"] = {"Edvin": edvin, "Ruta": ruta, "Lea": LEA0, "Pavo": PAVO0}
s4.append(f"2027-02-13: flat {fmt(fv)} (valuation for {fd}), cottage {fmt(cv)} (for {cd}); joint property counts half to each "
          f"(H-1 v1 .1). Edvin = {fmt(fv)}/2 + {fmt(cv)} + {fmt(J0)}/2 + {fmt(P0)}/2 = {fmt(edvin)}; Ruta = {fmt(fv)}/2 + {fmt(J0)}/2 + "
          f"{fmt(P0)}/2 = {fmt(ruta)}; Lea {fmt(LEA0)}, Pavo {fmt(PAVO0)} (S4-R3)")
d1 = dt.date(2027, 6, 30)
(fd, fv, _), (cd, cv, _) = valuation("flat", d1), valuation("cottage", d1)
ruta_acct = kept("2208 7702", d1)
LEA1, PAVO1 = kept("Lea's own", d1), kept("Pavo's own", d1)
ruta_own = fv / 2 + J0 / 2 + P0 / 2 + ruta_acct
estate = fv / 2 + cv + est_money_joint + P0 / 2
r_hold, p_hold, l_hold = ruta_own + estate / 2, PAVO1 + estate / 2, LEA1
k4["2027-06-30"] = {"Ruta": by_case(r_hold, r_hold - RUTA_PART), "Lea": by_case(l_hold, l_hold + CLAIM),
                    "Pavo": by_case(p_hold, p_hold - PAVO_PART)}
s4.append(f"2027-06-30: flat {fmt(fv)} (for {fd}), cottage {fmt(cv)} (for {cd}). Ruta's own: half the flat {fmt(fv / 2)} + her halves of the "
          f"accounts {fmt(J0 / 2)} + {fmt(P0 / 2)} + her account {fmt(ruta_acct)} (S4-R3) = {fmt(ruta_own)}. The estate, held 1/2 each: "
          f"{fmt(fv / 2)} + {fmt(cv)} + {fmt(est_money_joint)} + {fmt(P0 / 2)} = {fmt(estate)}. Ruta = {fmt(ruta_own)} + {fmt(estate / 2)} = "
          f"{fmt(r_hold)}; Pavo = {fmt(PAVO1)} (S4-R3) + {fmt(estate / 2)} = {fmt(p_hold)}; Lea = {fmt(LEA1)} (S4-R3); if the will's reason is untrue, "
          f"Lea + {fmt(CLAIM)}, Ruta − {fmt(RUTA_PART)}, Pavo − {fmt(PAVO_PART)} (H-1 v1 .1)")
d2 = dt.date(2027, 12, 31)
(fd, fv, _), (cd, cv, _) = valuation("flat", d2), valuation("cottage", d2)
joint_bal = R1[max(R1)][3]
ruta_acct = kept("2208 7702", d2)
pavo_acct = kept("Pavo's own", d2)
LEA2 = kept("Lea's own", d2)
assert pavo_acct == PAVO1 + PAY  # row 3 is the only change to Pavo's money after 2027-06-30
r_hold = fv + joint_bal + ruta_acct
p_hold = cv + pavo_acct
k4["2027-12-31"] = {"Ruta": by_case(r_hold, r_hold - RUTA_PART), "Lea": by_case(LEA2, LEA2 + CLAIM),
                    "Pavo": by_case(p_hold, p_hold - PAVO_PART)}
s4.append(f"2027-12-31: after the deed (SUC-1 v1 .3) Ruta alone owns the flat and all the money in account 2208 1145 (Edvin's account was "
          f"paid into it, S4-R1 row 2); Pavo the cottage. Flat {fmt(fv)} (for {fd}), cottage {fmt(cv)} (for {cd}). Ruta = {fmt(fv)} + "
          f"{fmt(joint_bal)} (S4-R1 row 3) + {fmt(ruta_acct)} (S4-R3) = {fmt(r_hold)}; Pavo = {fmt(cv)} + {fmt(pavo_acct)} (S4-R3) = "
          f"{fmt(p_hold)}; Lea = {fmt(LEA2)} (S4-R3); if the will's reason is untrue, Lea + {fmt(CLAIM)}, Ruta − {fmt(RUTA_PART)}, Pavo − {fmt(PAVO_PART)}")
for dd in ("2027-06-30", "2027-12-31"):
    tots = [sum(k4[dd][p]["cases"][c] for p in ("Ruta", "Lea", "Pavo")) for c in k4[dd]["Ruta"]["cases"]]
    assert tots[0] == tots[1]
assert edvin + ruta == valuation("flat", d0)[1] + valuation("cottage", d0)[1] + J0 + P0
# cases 1.4.0 (G2 Q-1): S4-04's slots after the death name the missing ruling; the by-case figures are S4-04P's
n_claim = len(s2) + 1  # the number (from 1) of s4's step that computes Lea's claim; the next is her claim's parts
k4r = {"2027-02-13": k4["2027-02-13"]}
for dd in ("2027-06-30", "2027-12-31"):
    k4r[dd] = {p: ruling(f"{p}'s worth on {dd} (Lea's claim, SUC-1 v1 .4-.5, is owed by Ruta and Pavo to Lea)") for p in k4[dd]}
s4r = s4[:n_claim + 1] + ["the will gives a reason for leaving Lea out; if it is true she has no claim (SUC-1 v1 .6); " + WHY_REFUSE] + s4[n_claim + 2:]
s4r = [s.replace("; if the will's reason is untrue, ", "; the claim (pinned twin S4-04P): if the will's reason is untrue, ") for s in s4r]
write_key("S4-04", "2027-12-31", "2027-12-31", k4r, s4r,
          ["on 2027-02-13, Edvin + Ruta = the flat + the cottage + both accounts"])
# S4-04P: the pinned twin. Ruta on 2027-12-31 is a slot of its own, asked as known 2028-01-01 (compare.py marks no cut,
# so the cut is in the slot's label)
RUTA_LATE = "Ruta, as known 2028-01-01"
ruta_reading = [(a, d, b) for a, d, b in R3 if a.startswith("2208 7702") and d == d2]
assert len(ruta_reading) == 1 and ruta_reading[0][2] == ruta_acct
k4p = {dd: {(RUTA_LATE if (dd, p) == ("2027-12-31", "Ruta") else p): pinned(k4[dd][p]) for p in k4[dd]}
       for dd in ("2027-06-30", "2027-12-31")}
ruta_before = kept("2208 7702", d1)
assert LEA2 == LEA1 and pavo_acct == PAVO1 + PAY and ruta_acct != ruta_before
WHY_LATE = (f"Ruta on 2027-12-31 is asked as known 2028-01-01, a slot of its own: her worth that day counts her account at the balance "
            f"kept for 2027-12-31 (S4-R3, {fmt(ruta_acct)}), {fmt(ruta_acct - ruta_before)} above the one kept for {d1} "
            f"({fmt(ruta_before)}); no row of the card shows the difference, only that balance kept for the day's end. By L §4.5 as applied at G2 Q-2, a "
            "reading at the cut's own day is seen but applied only before the cut, so at the card's cut 2027-12-31 it is not applied; "
            "the cut that applies it is the next day, 2028-01-01 (session 7b's judge, causes.md S4-04). The card's \"known as of "
            "2027-12-31\" means knowledge at that day's end, the same facts. Lea's and Pavo's slots stay as known 2027-12-31: their "
            f"kept balances for 2027-12-31 are what their earlier ones give (Lea {fmt(LEA1)} unchanged; Pavo {fmt(PAVO1)} + the "
            f"division's payment {fmt(PAY)}, S4-R1 row 3)")
write_key("S4-04P", "2027-12-31", "2027-12-31", k4p,
          s4[:n_claim + 1] + ["the will gives a reason for leaving Lea out; if it is true she has no claim (SUC-1 v1 .6): with the reason "
                              "pinned true, each slot's first figure; pinned untrue, Lea gains the claim and Ruta and Pavo owe their parts"]
          + s4[n_claim + 3:] + [WHY_LATE, PIN.replace("S4-03", "S4-04")],  # the twin has no slot for 2027-02-13
          ["on 2027-06-30 and 2027-12-31, Ruta + Lea + Pavo is the same under both pinned answers of the ruling"])
print("S4: 6 keys written")
