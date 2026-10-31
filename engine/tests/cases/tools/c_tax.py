"""Household C's joint returns for 2024 (as filed, and corrected after Rosvald's refund) and 2025, and
the tenth (C-HR6). Used by c_sim (what the household paid or got back) and keys_c (the answers).

Law: PIT-2, PIT-3 v3, PIT-5 v2 (and v1 .3 of the home supplement), PIT-6, PIT-7, PIT-10 (a), PIT-16,
PIT-17, PIT-18, GEN-4, GEN-5; the treaty's T-7 and T-20 and its protocol for 2025.
"""
from datetime import date
from decimal import Decimal

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import c_facts as F
from c_law import ZERO, D, child_relief, duk, foreign_base, gen4, half_up, joint_tax, pit3_tax, pit17
from fam.home import ALL_KNOWN, run_payer

FILED = {"2024": date(2025, 4, 28), "2024 corrected": date(2025, 10, 20), "2025": date(2026, 4, 27)}
REFUND_DAY = {"2024": date(2025, 5, 22), "2025": date(2026, 5, 21)}


def marta(year, *, known):
    out = {"employer": ZERO, "sio": ZERO, "adv employer": ZERO, "adv sio": ZERO}
    pays = F.marta_payments(year)
    if pays:
        _, t = run_payer(pays, reducing=True, known=known)
        out["employer"], out["adv employer"] = t["base"], t["advance"]
    if year == 2024:
        _, t = run_payer(F.parental_2024(), reducing=False, known=known)
        out["sio"], out["adv sio"] = t["base"], t["advance"]
    out["base"] = duk(out["employer"] + out["sio"])
    return out


def workshop_2024(*, known):
    import c_sim as S
    fm = S.firm(known=known)
    rev = sum((m["revenue"] for m in fm["months"].values()), ZERO)
    cst = sum((m["costs"] for m in fm["months"].values()), ZERO)
    soc = sum((a for d, a in fm["soc paid"] if d.year == 2024 and d <= F.CLOSED), ZERO)  # PIT-10 .3, v1 .7
    adv = sum((a["advance"] for a in fm["advances"]), ZERO)
    return {"revenue": rev, "costs": cst, "soc paid": soc, "profit": rev - cst - soc, "advances": adv}


def refund_shares(*, known):
    """PIT-18 .3: Rosvald's refund for 2024/25 shared among the year's payments by gross, to the cent, half up;
    each share converted (GEN-4) at the day the refund was received."""
    from c_law import assessment
    slips = [s for s in F.jan_slips(known=ALL_KNOWN) if s["tax year"] == "2024/25"]  # R1: Jan's payslips (C-D20), papers the household holds: the employer's withholding is the world's
    a = assessment(slips, "2024/25", 2, known=known)
    out = {}
    for s in slips:
        sh = half_up(a["refund"] * s["gross"] / a["gross"])
        out[s["work"]] = {"flr": sh, "duk": gen4(sh, F.ASSESS_2425["paid"], known=known)[0]}
    return out, a


def year_2024(gift, use17=True, corrected=False, relief_first=True, *, known):
    m = marta(2024, known=known)
    w = workshop_2024(known=known)
    fs = [foreign_base(s, known=known) | {"work": s["work"]} for s in F.jan_slips(known=ALL_KNOWN) if s["paid"].year == 2024]  # R1: Jan's payslips (C-D20), papers the household holds: the employer's withholding is the world's
    fb = sum((f["base"] for f in fs), ZERO)
    ft = sum((f["tax DUK"] for f in fs), ZERO)
    if corrected:
        sh, _ = refund_shares(known=known)
        ft -= sum((sh[f["work"]]["duk"] for f in fs), ZERO)
    jb = duk(w["profit"] + fb)
    return settle(m["base"], jb, fb, ft, ZERO, gift, use17, relief_first,
                  m["adv employer"] + m["adv sio"] + w["advances"], extra={"marta": m, "workshop": w, "foreign": fs})


def year_2025(gift, reading="paid", use17=True, use18=True, *, known):
    """reading: 'paid' — the pay for December 2024 (paid 2025-01-03) is income derived in 2025 and exempt;
    'earned' — it was derived in December 2024 and stays under the credit (PIT-16)."""
    m = marta(2025, known=known)
    sh, _ = refund_shares(known=known)
    fs = [foreign_base(s, known=known) | {"work": s["work"]} for s in F.jan_slips(known=ALL_KNOWN) if s["paid"].year == 2025]  # R1: Jan's payslips (C-D20), papers the household holds: the employer's withholding is the world's
    cred = [f for f in fs if reading == "earned" and f["work"] == "2024-12"]
    ex = [f for f in fs if f not in cred]
    fb = sum((f["base"] for f in cred), ZERO)
    ft = sum((f["tax DUK"] - sh[f["work"]]["duk"] for f in cred), ZERO)
    exempt = duk(sum((f["base"] for f in ex), ZERO))
    jb = duk(fb)
    if not use18:   # without PIT-18 .4: the exempt income counted in the PIT-7 and PIT-17 bases
        return settle(m["base"], jb, fb, ft, exempt, gift, use17, True, m["adv employer"],
                      extra={"marta": m, "foreign": fs}, exempt_in_bases=True)
    return settle(m["base"], jb, fb, ft, exempt, gift, use17, True, m["adv employer"], extra={"marta": m, "foreign": fs})


def settle(mb, jb, fb, ft, exempt, gift, use17, relief_first, advances, extra=None, exempt_in_bases=False):
    jb_cap = jb + (exempt if exempt_in_bases else ZERO)
    if use17 and gift:
        g = pit17(gift, mb, jb_cap)
        dm, dj = g["Marta"], min(g["Jan"], jb)
    else:
        g, dm, dj = None, ZERO, ZERO
    combined = duk(mb - dm + jb - dj)
    jt = joint_tax(combined, exempt)
    parents = mb + jb + (exempt if exempt_in_bases else ZERO)
    rel = child_relief(12, parents)
    if fb and combined:
        if relief_first:
            after = max(ZERO, jt - rel)
            part = duk(after * fb / combined)
            cr = duk(min(ft, part))
            due = max(ZERO, after - cr)
        else:
            part = duk(jt * fb / combined)
            cr = duk(min(ft, part))
            due = max(ZERO, jt - cr - rel)
    else:
        part, cr, due = ZERO, ZERO, max(ZERO, jt - rel)
    return {"marta base": mb, "jan base": jb, "foreign base": fb, "foreign tax": ft, "exempt": exempt, "pit17": g,
            "ded marta": dm, "ded jan": dj, "combined": combined, "joint tax": jt, "parents income": parents,
            "relief": rel, "credit part": part, "credit": cr, "due": due, "advances": advances,
            "payable": due - advances, **(extra or {})}


def tenth(fn, **kw):
    """C-HR6: the smallest whole-dukat gift G with G >= a tenth of (B - T(G)): B the spouses' bases before the
    gift is deducted, T(G) the tax due on the return with G deducted."""
    r0 = fn(ZERO, **kw)
    b = r0["marta base"] + r0["jan base"]
    g = duk(b / 10)
    while g > 0:
        t = fn(g - 1, **kw)["due"]
        if g - 1 >= (b - t) / 10:
            g -= 1
            continue
        break
    # g is now the smallest with g >= (b - T(g))/10, scanning down from a start above it
    while g < (b - fn(g, **kw)["due"]) / 10:
        g += 1
    return {"B": b, "G": g, "T": fn(g, **kw)["due"], "check": (b - fn(g, **kw)["due"]) / 10}


def household_taxes(*, known):
    """What the household paid or got back (the joint account): the 2024 return, its correction, the 2025 return
    (filed on the 'paid' reading)."""
    import c_sim as S
    out = []
    r24 = year_2024(S.TITHE[2024][1], known=known)
    c24 = year_2024(S.TITHE[2024][1], corrected=True, known=known)
    r25 = year_2025(S.TITHE[2025][1], reading="paid", known=known)
    for key, r, filed, refund_day in (("2024", r24, FILED["2024"], REFUND_DAY["2024"]),
                                      ("2025", r25, FILED["2025"], REFUND_DAY["2025"])):
        if r["payable"] > 0:
            out.append((filed, -r["payable"], "Urząd Skarbowy Zabór", f"PIT za {key} — dopłata"))
        elif r["payable"] < 0:
            out.append((refund_day, -r["payable"], "Urząd Skarbowy Zabór", f"zwrot PIT za {key}"))
    diff = c24["due"] - r24["due"]
    if diff > 0:
        out.append((FILED["2024 corrected"], -diff, "Urząd Skarbowy Zabór", "PIT za 2024 — korekta, dopłata"))
    return out, {"2024": r24, "2024 corrected": c24, "2025": r25}


if __name__ == "__main__":
    import c_sim as S
    for k, r in household_taxes(known=ALL_KNOWN)[1].items():  # R1: a debug run over the world, so it sees every figure
        print(k, {x: r[x] for x in ("marta base", "jan base", "foreign base", "foreign tax", "exempt", "ded marta", "ded jan",
                                    "combined", "joint tax", "relief", "credit part", "credit", "due", "advances", "payable")})
    print("tenth 2024", tenth(year_2024, known=ALL_KNOWN))  # R1: a debug run over the world, so it sees every figure
    print("tenth 2024 corrected", tenth(year_2024, corrected=True, known=ALL_KNOWN))  # R1: a debug run over the world, so it sees every figure
    print("tenth 2024 no PIT-17", tenth(year_2024, use17=False, known=ALL_KNOWN))  # R1: a debug run over the world, so it sees every figure
    print("tenth 2025 paid", tenth(year_2025, reading="paid", known=ALL_KNOWN))  # R1: a debug run over the world, so it sees every figure
    print("tenth 2025 earned", tenth(year_2025, reading="earned", known=ALL_KNOWN))  # R1: a debug run over the world, so it sees every figure
    print(household_taxes(known=ALL_KNOWN)[0])  # R1: a debug run over the world, so it sees every figure
