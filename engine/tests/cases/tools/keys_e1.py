#!/usr/bin/env python3
"""E1's keys: households A, B and D as one family. One file per question in keys/E1/.

Reads the family's joins as stated once (fam/e.py), D's derivation of the estate (fam/d.py), the
records of the accounts the money moved through, and the households' own keys where E1 takes a
household's answer as given (keys/A, keys/B, keys/D: the union reproduces them, `union_e1.py`).
Asserts every relation on its own output, among them that each join agrees with every household
that shows it; exits 1 if one fails.

Run: python3 -I tmp/cases/tools/keys_e1.py
"""
import csv
import pathlib
import sys
from datetime import date, datetime

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from fam import d as DF  # noqa: E402
from fam import e as E  # noqa: E402
from fam.home import ZERO, D, figure  # noqa: E402
from lib.keys import Trace, enc, load_block  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[1]
FAILS = []
ORDER = ["Piotr", "Anna", "Marta", "Jakub"]          # D-D07 .5


def check(cond, what):
    if not cond:
        FAILS.append(what)


def hkey(house, qid, root=None):
    """A household's key, taken as given."""
    return load_block(((root or ROOT / "keys") / house / f"{qid}.md").read_text(encoding="utf-8"))


def put(qid, edges, as_of, known, key, t, relations=(), out=None, by="tools/keys_e1.py"):
    head = [f"question: {enc(qid)}", f"edges: {enc(edges)}", f"as_of: {enc(as_of)}", f"known_as_of: {enc(known)}",
            f"key: {enc(key)}"]
    if relations:
        head.append(f"relations: {enc(list(relations))}")
    body = (f"# Key · {qid}\n\nWritten by `{by}`; don't edit.\n\n```yaml\n" + "\n".join(head) +
            "\n```\n\n## Derivation\n\n" + t.render() + "\n")
    p = (out or ROOT / "keys" / "E1") / f"{qid}.md"
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(body, encoding="utf-8")


def refusal(missing, reason):
    return {"kind": "refusal", "missing": missing, "reason": reason}


# --------------------------------------------------------------------------- records, each in its dialect

def comma(s):
    return D(s.replace(" ", "").replace(",", "."))


def wrzos(path):
    """Bank Wrzos: header lines, then `;` rows, dd.mm.yyyy, decimal comma, balance after."""
    lines = path.read_text(encoding="utf-8").splitlines()
    acct = next(ln.split(";", 1)[1] for ln in lines if ln.startswith("Account;"))
    start = next(i for i, ln in enumerate(lines) if ln.startswith("Booking date;"))
    rows = []
    for n, ln in enumerate(lines[start + 1:], start + 2):
        f = ln.split(";")
        rows.append({"line": n, "date": datetime.strptime(f[0], "%d.%m.%Y").date(), "type": f[1], "cp": f[2],
                     "title": ";".join(f[3:-2]), "amount": comma(f[-2])})
    return acct, rows


def cis(path):
    """Bank Cis: `#` header lines, then quoted CSV, dd-mm-yyyy, debit and credit columns."""
    lines = path.read_text(encoding="utf-8").splitlines()
    acct = next(ln.split(": ", 1)[1] for ln in lines if ln.startswith("# Account:"))
    body = [ln for ln in lines if not ln.startswith("#")]
    rows = []
    for n, f in enumerate(csv.reader(body[1:]), len(lines) - len(body) + 2):
        amt = D(f[6]) if f[6] else -D(f[5])
        rows.append({"line": n, "date": datetime.strptime(f[0], "%d-%m-%Y").date(), "type": f[2], "cp": f[3],
                     "title": f[4], "amount": amt})
    return acct, rows


def olcha(path):
    """Bank Olcha: `|` fields, yyyymmdd, decimal comma with space grouping, balance after."""
    lines = path.read_text(encoding="utf-8").splitlines()
    acct = next(ln.split("|", 1)[1] for ln in lines if ln.startswith("Account|"))
    rows = []
    for n, ln in enumerate(lines, 1):
        f = ln.split("|")
        if len(f) == 6 and f[0].isdigit() and len(f[0]) == 8:
            rows.append({"line": n, "date": datetime.strptime(f[0], "%Y%m%d").date(), "type": f[1], "cp": f[2],
                         "title": f[3], "amount": comma(f[4])})
    return acct, rows


def notary_row(rows, day):
    hit = [r for r in rows if "brzoza" in r["cp"].lower() and r["date"] == day]
    check(len(hit) == 1, f"one notary row on {day}")
    return hit[0]


# --------------------------------------------------------------------------- E1-01 the estate, paid out

def payouts_both_ends(t, other_end=None, *, known):
    """The deposit and each payout, from D's derivation, D's records and the recipients' records.
    `other_end`, where given, reads Marta's end from papers the family's visible households don't hold."""
    div = DF.division(known=known)
    t("the heirs' shares", dict(E.SHARES), "CIV-3.1, CIV-3.2: no will; Adam's 3/16 goes to Jakub (D-D04)")
    derived = {"Piotr": div["child"], "Anna": div["child"], "Marta": div["child"], "Jakub": div["kuba"]}
    t("the payouts, as D derives them", derived,
      "D-D07 .4: each child's 3/16 of the estate as divided, Jakub's 3/16 without the flat he keeps a share in, "
      "less the set-off of D-D07 .3, rounded down to whole dukats (CIV-5.2); keys/D/D-01")
    d01 = hkey("D", "D-01")["key"]
    for h in ORDER:
        check(derived[h] == E.PAYOUTS[h]["amount"], f"E1-01 D's derived payout to {h} is the one stated once")
        check(D(d01["payouts"][h if h != "Anna" else "Anna"]) == E.PAYOUTS[h]["amount"], f"E1-01 D-01's payout to {h}")
    acct, rows = olcha(ROOT / "D" / "records" / "olcha-2025-06.csv")
    dep = notary_row(rows, E.DIVISION_DAY)
    into = -dep["amount"]
    t("into the deposit", into, f"D-D07 .5; records/olcha-2025-06.csv line {dep['line']} ({acct}), {dep['date']}")
    check(into == D(d01["into the deposit"]), "E1-01 the deposit row is D-01's")
    out = {}
    a_acct, a_rows = wrzos(ROOT / "A" / "records" / "wrz-ania-savings-2025.csv")
    b_acct, b_rows = cis(ROOT / "B" / "records" / "cis-5521-3380-0002-2025.csv")
    ends = {"Anna": ("A", "records/wrz-ania-savings-2025.csv", a_acct, a_rows, "A-D36"),
            "Piotr": ("B", "records/cis-5521-3380-0002-2025.csv", b_acct, b_rows, "B-D27")}
    for h in ORDER:
        p = E.PAYOUTS[h]
        e = {"amount": p["amount"], "paid on": E.PAYOUT_DAY.isoformat()}
        if h in ends:
            house, f, acc, rws, doc = ends[h]
            r = notary_row(rws, E.PAYOUT_DAY)
            check(acc == p["to"], f"E1-01 {h}'s account in {house}'s records is the one D-D07 names")
            check(r["amount"] == p["amount"], f"E1-01 {h}'s records show the stated payout")
            e["to"] = p["to"]
            e["arrived"] = r["date"].isoformat()
            t(f"{h}: paid and arrived", e, f"D-D07 .5 names {p['to']}; {doc}; {house}/{f} line {r['line']}: {r['amount']}")
        elif h == "Marta" and other_end:
            e.update(other_end(t, p))
        elif h == "Marta":
            e["to"] = refusal(["the number of Marta's account: D-D07 says only \"to her account\""],
                              "no document or record of the family names it")
            e["arrived"] = refusal(["a record of Marta's account"], "no record of hers is among the family's papers")
            t("Marta: paid", e, "D-D07 .5: \"to Marta to her account\"; nothing else of hers is in the family's papers")
        else:
            e["to"] = p["to"]
            e["arrived"] = refusal(["a record of Jakub's account " + p["to"]],
                                   "no record of his is among the family's papers")
            t("Jakub: paid", e, "D-D07 .5 names the account; Jakub keeps no records in the set")
        out[h] = e
    paid = sum((E.PAYOUTS[h]["amount"] for h in ORDER), ZERO)
    left = into - paid
    t("left in the deposit", left, "what was paid in less the four payouts")
    check(left == ZERO, "E1-01 the deposit pays out exactly what was paid in")
    return {"into the deposit": into, "payouts": out, "left in the deposit": left}


def e1_01():
    t = Trace()
    key = payouts_both_ends(t, known="2026-01-15")   # R1: E1-01's known-as-of date
    return key, t


# --------------------------------------------------------------------------- E1-02 what each of us is worth

def kuba_2025():
    """Jakub on 2025-12-31, counted as Halina counts (D-HR4), from what the family's papers show of him."""
    d08 = hkey("D", "D-08")["key"]
    flat = DF.VALUATION_2025[1] * 3 / 16
    owed = -D(d08["owed to Jakub"])
    missing = ["what Jakub has and owes beyond Ogrodowa and Halina's debt: no account or record of his is in the "
               "family's papers, so not what is left of his 7 812.00 DUK payout or of the shares Halina sent him",
               "how much of his PIT-13 tax on his 3/16 of 2025's rent he had paid by 2025-12-31 (D-HR1 .4: he pays his own)"]
    return flat, owed, missing


def worth_2025(t):
    a11, b11, d08 = hkey("A", "A-11")["key"], hkey("B", "B-11")["key"], hkey("D", "D-08")["key"]
    av = a11["value"]
    bv = {k: (v["value"] if isinstance(v, dict) and "kind" in v else v) for k, v in b11.items()}  # B-11's partials stand per person
    t("Ania and Tomek, as A counts (A-HR5)", {"Ania": av["Ania"], "Tomek": av["Tomek"]},
      "keys/A/A-11; the missing savings rate touches the Lazur savings, community, so both halves")
    t("Piotr and Ola, as B counts (B-HR4)", {"Piotr": bv["Piotr"], "Ola": bv["Ola"]},
      "keys/B/B-11; what Ola spent from Wrzos after March is hers alone")
    hv = d08["net worth"]
    t("Halina, as she counts (D-HR4)", hv["value"], "keys/D/D-08; the house is never valued")
    flat, owed, kmiss = kuba_2025()
    t("Jakub: his 3/16 of Ogrodowa", flat, "D-HR4 .2: the latest valuation, 240 000.00 (D-D29, 2025-10-06), for his share; "
      "the agreed 165 000.00 (D-D07 .2) is not one")
    t("Jakub: owed to him by Halina", owed, "D-HR4 .3: 2025-Q4's share under D-HR1, held in her account (keys/D/D-08); rent one co-owner collects is owed to the others (CIV-4 .2)")
    kv = flat + owed
    people = {
        "Ania": {"kind": "partial", "value": D(av["Ania"]), "missing": list(a11["missing"])},
        "Tomek": {"kind": "partial", "value": D(av["Tomek"]), "missing": list(a11["missing"])},
        "Piotr": {"kind": "partial", "value": D(bv["Piotr"]), "missing": list(b11["Piotr"]["missing"]) + [
            "whether Piotr owes inheritance tax on Zbigniew's estate: nothing says whether he reported by 2025-05-05 (E1-03)"]},
        "Ola": {"kind": "partial", "value": D(bv["Ola"]), "missing": list(b11["Ola"]["missing"])},
        "Halina": {"kind": "partial", "value": D(hv["value"]), "missing": list(hv["missing"])},
        "Jakub": {"kind": "partial", "value": kv, "missing": kmiss},
    }
    val = lambda x: x["value"] if isinstance(x, dict) else x  # noqa: E731
    total = sum((val(v) for v in people.values()), ZERO)
    missing = []
    for v in people.values():
        for m in (v.get("missing", []) if isinstance(v, dict) else []):
            if m not in missing:
                missing.append(m)
    t("together", total, "the six add up; a debt between two of them is one's minus and the other's plus")
    owes = [{"by": "Ola", "to": "Piotr", "amount": D(bv["flat costs"]["Ola owes Piotr"])},
            {"by": "Halina", "to": "Jakub", "amount": owed}]
    t("owed within the family", owes, "B-HR1 (keys/B/B-11); D-HR1 (keys/D/D-08)")
    check(D(av["Ania"]) + D(av["Tomek"]) == D(av["household"]), "E1-02 A's people add up to A-11's household")
    check(D(bv["Piotr"]) + D(bv["Ola"]) == D(bv["household"]), "E1-02 B's people add up to B-11's household")
    check(total == D(av["household"]) + D(bv["household"]) + D(hv["value"]) + kv, "E1-02 the people add up to the family")
    check(-D(d08["owed to Jakub"]) == owed, "E1-02 Halina's debt to Jakub is his claim")
    people["together"] = {"kind": "partial", "value": total, "missing": missing}
    return people, owes


def e1_02():
    t = Trace()
    people, owes = worth_2025(t)
    key = dict(people)
    key["owed within the family"] = owes
    return key, t


# --------------------------------------------------------------------------- E1-03 gifts and inheritances

def sp(x):
    return f"{D(x):,.2f}".replace(",", " ")


def heir_cases(name, pron, subj, *, known):
    free, _ = figure("IGT-FREE-I", "2024-11-03", known)
    return {"kind": "by-case", "cases": {
        "reported by 2025-05-05": 0,
        "not reported": refusal([f"the flat's value for the tax, as {name} declares it or the RO sets it (IGT-2.2): "
                                 f"whether {pron} acquisition is above IGT-FREE-I ({sp(free)}), so that {subj} had to "
                                 "report (IGT-4.3), and the Group I tax both turn on it"],
                                "the dossier has no value the law takes")},
        "settled by": [f"whether {name} reported to the RO by 2025-05-05", "the flat's value as declared or set"]}


def no_report(house, name):
    """No document of `house` reports an inheritance to the RO."""
    for p in sorted((ROOT / house / "docs").glob("*.md")):
        s = p.read_text(encoding="utf-8").lower()
        if "revenue office" in s and ("estate" in s or "inherit" in s) and "report" in s:
            FAILS.append(f"E1-03 {p.name} may report {name}'s inheritance")


def gifts(t, *, known):
    a06, d02 = hkey("A", "A-06")["key"], hkey("D", "D-02")["key"]
    t("the inheritance tax arises", d02["tax arises"], "IGT-2.1: the day the certificate became final (D-D04); keys/D/D-02")
    t("report by", d02["report by"], "IGT-4.2: 6 months (GEN-3.1), moved to the next business day (GEN-2.2); keys/D/D-02")
    acq = {}
    acq["Anna from Halina"] = {"tax": D(a06["gift tax"])}
    t("Anna from Halina", acq["Anna from Halina"], "A-D19 (2021, cash, never credited) added to A-D18 (2023, the units) "
      "within 5 years (IGT-3.1); reported late (A-D20, IGT-4.2): Group I; keys/A/A-06")
    acq["Halina from Zbigniew"] = {"tax": D(d02["Halina"]["tax"])}
    t("Halina from Zbigniew", acq["Halina from Zbigniew"], "reported on 2025-05-05 (D-D08), in time: exempt (IGT-4.1); keys/D/D-02")
    for name, house in (("Anna", "A"), ("Piotr", "B")):
        no_report(house, name)
        acq[f"{name} from Zbigniew"] = {"tax": heir_cases(name, *(("her", "she") if name == "Anna" else ("his", "he")), known=known)}
        t(f"{name} from Zbigniew", acq[f"{name} from Zbigniew"],
          f"{house}'s papers show the payout (CIV-5) but no report to the RO; the acquisition is 3/16 of the estate on "
          "2024-11-03 at the law's values (IGT-2.2), not the payout; nothing from Halina is added to it (IGT-3.1: another giver)")
    jc = d02["Jakub"]
    acq["Jakub from Zbigniew"] = {"tax": {"kind": "by-case", "cases": {k: v["tax"] if "tax" in v else v
                                                                        for k, v in jc["cases"].items()},
                                          "settled by": jc["settled by"]}}
    check(acq["Jakub from Zbigniew"]["tax"] == heir_cases("Jakub", "his", "he", known=known), "E1-03 Jakub's cases are D-02's")
    t("Jakub from Zbigniew", acq["Jakub from Zbigniew"], "keys/D/D-02")
    in_estate = False
    t("Halina's gifts in Zbigniew's estate", in_estate,
      "D's story: her mother's estate (2018) was hers alone, and bought the units (CIV-1.3); D-D07 .1 lists what Halina takes from the estate: no units")
    return {"inheritance tax arises": d02["tax arises"], "report by": d02["report by"], "acquisitions": acq,
            "Halina's gifts in Zbigniew's estate": in_estate}


def e1_03():
    t = Trace()
    return gifts(t, known="2025-12-31"), t   # R1: E1-03's known-as-of date


def main():
    k, t = e1_01()
    put("E1-01", ["PTY-05", "FLOW-01", "FLOW-02", "REC-04", "REC-06"], "2025-06-30", "2026-01-15", k, t,
        ["the deposit pays out exactly what was paid in",
         "each payout as D derives it is the one its heir's household states and its account shows"])
    k, t = e1_02()
    put("E1-02", ["ASK-02", "PTY-05", "OWN-02", "OWN-03", "OWN-04"], "2025-12-31", "2026-01-15", k, t,
        ["the people add up to the family", "each household's people are its own key's",
         "a debt within the family is one person's minus and another's plus"])
    k, t = e1_03()
    put("E1-03", ["PTY-05", "LAW-11", "LAW-12", "PTY-03", "TIME-05", "OWN-05"], "2025-12-31", "2025-12-31", k, t,
        ["Halina's and Jakub's taxes are D-02's, Anna's gift tax is A-06's",
         "nothing given by one person is added to what came from another"])
    if FAILS:
        print("relations failed:", FAILS)
        sys.exit(1)
    print("E1: 3 keys written, relations pass")


if __name__ == "__main__":
    main()
