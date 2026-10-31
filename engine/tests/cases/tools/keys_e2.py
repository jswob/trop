#!/usr/bin/env python3
"""E2's keys: household C joins households A, B and D. One file per question in engine/tests/cases/keys/E2/.

Reads what E1 reads (through tools/keys_e1.py: the joins stated once in fam/e.py, D's derivation, the
records of the accounts the money moved through, the visible households' keys as given) and, of C,
its records and facts (c_facts) and its own keys as given (keys/C). Asserts every relation on its own
output, among them that every join agrees with each household that shows it; exits 1 if one fails.

Run: python3 -I engine/tests/cases/tools/keys_e2.py
"""
import pathlib
import sys
from datetime import datetime

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import c_facts as F  # noqa: E402  (puts the visible tools on the path)
import c_sim as S  # noqa: E402
import keys_e1 as K1  # noqa: E402
from fam import e as E  # noqa: E402
from fam.home import D  # noqa: E402
from lib.keys import Trace  # noqa: E402

HOLD = HERE.parent
KEYS = HOLD / "keys"
FAILS = K1.FAILS


def check(cond, what):
    if not cond:
        FAILS.append(what)


def jodla(path):
    """Bank Jodła: header lines, a column line, then `;` fields, dd.mm.yyyy, decimal comma, balance after."""
    lines = path.read_text(encoding="utf-8").splitlines()
    acct = lines[1].split(": ", 1)[1]
    rows = []
    for n, ln in enumerate(lines, 1):
        f = ln.split(";")
        if not (len(f) >= 6 and len(f[0]) == 10 and f[0][2] == "." and f[0][:2].isdigit()):
            continue
        rows.append({"line": n, "date": datetime.strptime(f[0], "%d.%m.%Y").date(), "type": f[1], "cp": f[2],
                     "title": ";".join(f[3:-2]), "amount": K1.comma(f[-2])})
    return acct, rows


def marta_end(t, p):
    """Marta's payout at her end: her account in C's records, and C-D30."""
    acct, rows = jodla(HOLD / "C" / "records" / "jodla-marta.csv")
    r = K1.notary_row(rows, E.PAYOUT_DAY)
    c30 = (HOLD / "C" / "docs" / "C-D30.md").read_text(encoding="utf-8")
    check(acct == F.ACC_MARTA and F.ACC_MARTA in c30, "E2-01 C-D30 and C's records name Marta's account")
    check(r["amount"] == p["amount"] and K1.sp(p["amount"]) in c30, "E2-01 C's records show the stated payout")
    check(S.INHERIT[0] is E.PAYOUT_DAY and S.INHERIT[1] is p["amount"], "E2-01 C takes the payout from E")
    e = {"to": acct, "arrived": r["date"].isoformat()}
    t("Marta: paid and arrived", e,
      f"D-D07 .5 says \"to her account\"; C-D30 names {acct}, her own; C/records/jodla-marta.csv line {r['line']}: {r['amount']}")
    return e


def e2_01():
    t = Trace()
    return K1.payouts_both_ends(t, marta_end, known="2026-09-30"), t   # R1: E2-01's known-as-of date


def ckey(qid):
    return K1.hkey("C", qid, KEYS)["key"]


def e2_02():
    t = Trace()
    key = K1.gifts(t, known="2026-09-30")   # R1: E2-02's known-as-of date
    acq = key["acquisitions"]
    for p in sorted((HOLD / "C" / "docs").glob("*.md")):
        s = p.read_text(encoding="utf-8").lower()
        if "revenue office" in s and ("estate" in s or "inherit" in s) and "report" in s:
            FAILS.append(f"E2-02 {p.name} may report Marta's inheritance")
    acq["Marta from Zbigniew"] = {"tax": K1.heir_cases("Marta", "her", "she", known="2026-09-30")}
    t("Marta from Zbigniew", acq["Marta from Zbigniew"],
      "C's papers show the payout (C-D30) and no report to the RO; as for every child: 3/16 of the estate on 2024-11-03 "
      "at the law's values (IGT-2.2)")
    c11 = ckey("C-11")["gift tax"]
    acq["Jan from Marta"] = {"tax": {"kind": "by-case", "cases": {k: v["tax"] for k, v in c11["cases"].items()}, "settled by": c11["settled by"]}}
    t("Jan from Marta", acq["Jan from Marta"],
      "the 40 000.00 of 2024-07-01 and the released half of the loan (IGT-2 v1 .4), added up (IGT-3.1); keys/C/C-11")
    c09 = ckey("C-09")["gift tax"]
    acq["Staś from Irena"] = {"tax": D(c09["tax"])}
    t("Staś from Irena", acq["Staś from Irena"], "300.00 a month into the parents' account, added up below IGT-FREE-I; keys/C/C-09")
    check(c11["cases"]["the release is a gift of money, never credited"]["tax"] > 0, "E2-02 one case of Jan's taxes him")
    return key, t


def e2_03():
    t = Trace()
    c11 = ckey("C-11")
    share = E.SHARES["Marta"]
    t("Marta's share", share, "D-D04; CIV-3.1")
    t("hers alone", True, "CIV-1.3: an inheritance is separate property; C-D30 pays it into her own account (C-D35)")
    lent = S.LOAN["amount"]
    c31 = (HOLD / "C" / "docs" / "C-D31.md").read_text(encoding="utf-8")
    check(K1.sp(lent) in c31 and "inheritance" in c31, "E2-03 C-D31 lends from her inheritance")
    t("lent to the community", lent, f"C-D31, {S.LOAN['date']}: from her inheritance, under C-HR7")
    owed = D(c11["owed at the forgiveness"])
    t("owed to her at the forgiveness", owed, "C-HR7: lent + interest − repaid; keys/C/C-11")
    gift = D(c11["gift"]["Jan"]).quantize(D("1"), rounding="ROUND_HALF_UP")
    t("the forgiveness: a gift to Jan", gift, "IGT-2 v1 .4: half of the community's debt released, in whole dukats as the question asks (GEN-5.1's way); keys/C/C-11")
    check(D(c11["gift"]["Marta"]) == 0, "E2-03 the forgiveness gives Marta nothing")
    check(abs(gift - owed / 2) <= D("0.5"), "E2-03 the gift is half of what was released, in whole dukats")
    check(owed == lent + D(c11["interest"]) - D(c11["repaid"]), "E2-03 owed = lent + interest − repaid")
    key = {"share": share, "paid": E.PAYOUTS["Marta"]["amount"], "hers alone": True, "lent to the community": lent,
           "owed to her at the forgiveness": owed, "gift to Jan": gift}
    t("paid", key["paid"], "D-D07 .4; C-D30")
    return key, t


def main():
    out = KEYS / "E2"
    by = "tools/keys_e2.py"
    k, t = e2_01()
    K1.put("E2-01", ["PTY-05", "FLOW-01", "FLOW-02", "REC-04", "REC-06"], "2025-06-30", "2026-09-30", k, t,
           ["the deposit pays out exactly what was paid in",
            "each payout as D derives it is the one its heir's household states and its account shows"], out, by)
    k, t = e2_02()
    K1.put("E2-02", ["PTY-05", "LAW-11", "LAW-12", "PTY-03", "PTY-04", "TIME-05", "OWN-05"], "2026-09-30", "2026-09-30",
           k, t, ["Halina's and Jakub's taxes are D-02's, Anna's is A-06's, Jan's is C-11's, Staś's is C-09's",
                  "nothing given by one person is added to what came from another"], out, by)
    k, t = e2_03()
    K1.put("E2-03", ["PTY-05", "OWN-04", "OWN-05", "LIFE-03"], "2026-09-30", "2026-09-30", k, t,
           ["owed = lent + interest − repaid", "the gift is half of what was released"], out, by)
    if FAILS:
        print("relations failed:", FAILS)
        sys.exit(1)
    print("E2: 3 keys written, relations pass")


if __name__ == "__main__":
    main()
