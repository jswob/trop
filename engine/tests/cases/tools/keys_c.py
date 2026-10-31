#!/usr/bin/env python3
"""Household C's keys: one file per question in engine/tests/cases/keys/C/, each a YAML block and the derivation.

Reads the home law (fam/home.py, with the hidden points in c_law), Rosvald's law and the treaty (c_law),
both gazettes, Bank Lazur's table, C's records where a question reads them (the Lazur export, the joint
account's export, Jan's log), and the facts C's documents state (c_facts, c_sim, c_tax). Asserts every
relation on its own output; exits 1 if one fails.

Run: python3 -I engine/tests/cases/tools/keys_c.py
"""
import csv
import pathlib
import sys
from datetime import date, timedelta

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import c_facts as F  # noqa: E402
import c_sim as S  # noqa: E402
import c_tax as X  # noqa: E402
from c_law import (CAL, RCAL, ZERO, D, P, add_months, assessment, coordinate, duk, end_of_month, gross_from_net,  # noqa: E402
                   gross_from_net_any, half_up, ort, rnb_rate, to_flr, withhold)
from lib.keys import Trace, enc  # noqa: E402
from fam.home import ALL_KNOWN  # noqa: E402

HOLD = HERE.parents[0]
REC = HOLD / "C" / "records"
FAILS = []


def check(cond, what):
    if not cond:
        FAILS.append(what)


def dirn(x):
    x = D(x)
    return {"refund": -x} if x < 0 else {"payable": x}


def way(x):
    """The questions' direction word; where nothing moves, a bare 0."""
    return ZERO if D(x) == 0 else dirn(x)


def put(qid, edges, as_of, key, t, without=None, relations=()):
    head = [f"question: {enc(qid)}", f"edges: {enc(edges)}", f"as_of: {enc(as_of)}", f"known_as_of: {enc(as_of)}",
            f"key: {enc(key)}"]
    if without is not None:
        head.append(f"without: {enc(without)}")
    if relations:
        head.append(f"relations: {enc(list(relations))}")
    body = (f"# Key · {qid}\n\nWritten by `engine/tests/cases/tools/keys_c.py`; don't edit.\n\n```yaml\n" + "\n".join(head) +
            "\n```\n\n## Derivation\n\n" + t.render() + "\n")
    p = HOLD / "keys" / "C" / f"{qid}.md"
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(body, encoding="utf-8")


def refusal(missing, reason):
    return {"kind": "refusal", "missing": missing, "reason": reason}


def by_case(cases, settled_by=None):
    """README § Keys: a by-case stands in place of the smallest value it concerns; a field every case shares stays plain."""
    first = next(iter(cases.values()))
    out = {}
    for f in first:
        vals = {label: c[f] for label, c in cases.items()}
        out[f] = first[f] if all(v == first[f] for v in vals.values()) else \
            {"kind": "by-case", "cases": vals, **({"settled by": settled_by} if settled_by else {})}
    return out


def month_after(d, n=1):
    """GEN-3.1: n months from a day."""
    return add_months(d, n)


# --------------------------------------------------------------------------- records read by the keys

def lazur_rows():
    out = []
    with open(REC / "lazur-jan.csv", encoding="utf-8") as fh:
        lines = [ln for ln in fh if not ln.startswith("#")]
    for r in csv.DictReader(lines):
        out.append({"date": P(r["data"]), "cur": r["waluta"], "type": r["typ"], "cp": r["kontrahent"], "title": r["opis"],
                    "amount": D(r["kwota"]), "balance": D(r["saldo"])})
    return out


def jodla_rows(name):
    out = []
    lines = (REC / name).read_text(encoding="utf-8").splitlines()
    opening = D(lines[4].split(": ")[1].replace(",", "."))
    for ln in lines[7:]:
        d, t, cp, ti, a, b = ln.split(";")
        out.append({"date": date(int(d[6:]), int(d[3:5]), int(d[:2])), "type": t, "cp": cp, "title": ti,
                    "amount": D(a.replace(",", ".")), "balance": D(b.replace(",", "."))})
    return opening, out


def end_balance(rows, opening, d):
    b = opening
    for r in rows:
        if r["date"] > d:
            break
        b = r["balance"]
    return b


def flr_end(rows, d):
    b = ZERO
    for r in rows:
        if r["cur"] != "FLR":
            continue
        if r["date"] > d:
            break
        b = r["balance"]
    return b

# --------------------------------------------------------------------------- C-01


def q01(A):
    known = "2024-12-31"   # R1: C-01's known-as-of date
    t = Trace()
    fm = A["firm"]
    jun = fm["months"]["2024-06"]
    t("June 2024's activity", F.CLOSED.day, "C-D09: closed 2024-06-14, so 14 of June's 30 days")
    soc = t("June social", jun["soc3"], f"SOC-3 .3 base 60 % of AVG-WAGE-PROJ 2024 pro-rated 14/30 (SOC-3 v1 .5), rates .4 with sickness (C-D09), each to the ort: {jun['soc3 detail']}")
    h = t("June health", jun["health"], f"SOC-4 .1: 9 % of June's income {jun['income']} (registers) is below the floor 9 % of MIN-WAGE 2024-01-01, so the floor")
    due = t("due", F.due20("2024-06"), "SOC-3 .6, SOC-4 .4: by 2024-07-20, a Saturday, so 2024-07-22 (GEN-2.2)")
    v = [x for x in fm["vat"] if x["month"] == "2024-06"][0]
    t("last VAT return", v, "VAT-4 v1 .4: the closing month's return is the last; output VAT on June's invoices less June's input VAT (GEN-5.2)")
    t("refunded on", S.VAT_REFUND_DAY, "C-D12: refund asked in the return (VAT-4 .2); the joint account's row of 2024-08-29")
    adv = sum((a["advance"] for a in fm["advances"]), ZERO)
    t("advances", adv, "PIT-10 .5 (a): the PIT-3 v3 tax on the year's profit to date less advances paid, each month; contributions paid by the month's end or the closing day (PIT-10 v1 .7): " +
      "; ".join(f"{a['month']} {a['advance']}" for a in fm["advances"]))
    w = X.workshop_2024(known=known)
    prof = t("profit", w["profit"], f"PIT-10 .3, v1 .7: revenue {w['revenue']} less costs {w['costs']} less social contributions paid to the closing day {w['soc paid']} (December 2023's, paid 2024-01-22, and January–April 2024's; May's, paid 2024-06-20, and June's, paid 2024-07-22, fall after it)")
    after = [a for a in fm["after"] if a[0] <= date(2024, 12, 31)]
    out_ = t("after closing: paid out", -sum((a[1] for a in after if a[1] < 0), ZERO), "the joint account's rows for the workshop from 2024-06-29: " + "; ".join(f"{a[0]} {a[4]} {a[1]}" for a in after))
    in_ = t("after closing: paid in", sum((a[1] for a in after if a[1] > 0), ZERO))
    key = {"June social": soc, "June health": h, "due": due.isoformat(),
           "last VAT return": {"month": v["month"], "result": dirn(v["result"]), "refunded on": S.VAT_REFUND_DAY.isoformat()},
           "advances": adv, "profit": prof, "after closing": {"paid out": out_, "paid in": in_}}
    check(prof == w["revenue"] - w["costs"] - w["soc paid"], "C-01 profit")
    check(out_ == soc + h, "C-01 paid out = June's contributions")
    full = F.soc3("2024-06", known=known)["total"]
    t("without the switch", full, "the workshop stays open: June is a whole month of activity, and its contributions are paid from the business account")
    without = {"June social": full, "June health": h, "due": due.isoformat(), "after closing": {"paid out": ZERO, "paid in": ZERO}}
    put("C-01", ["LIFE-02", "LIFE-05", "NUM-02", "LAW-08", "TIME-03", "OWN-03"], "2024-12-31", key, t, without,
        ["profit = revenue − costs − contributions paid to the closing day", "paid out after closing = June's social + health"])

# --------------------------------------------------------------------------- C-02


def q02(A):
    known = "2026-03-31"   # R1: C-02's known-as-of date
    t = Trace()
    slips = {s["work"]: s for s in A["lazur"]["slips"]}
    lr = lazur_rows()
    row = [r for r in lr if r["title"] == "Lohn 10/2025"][0]
    t("October 2025's net", row["amount"], f"records: lazur-jan.csv, {row['date']} 'Lohn 10/2025' (the payslip is lost)")
    cp = F.contract_period("2025-10")
    cands = gross_from_net(row["amount"], row["date"], F.jan_class(row["date"]), F.BASE[cp], F.OT_RATE[cp], known=known)
    t("grosses under the contract", cands, f"C-D14, C-D16: base {F.BASE[cp]} + whole hours × {F.OT_RATE[cp]}; net = gross − contribution − tax (RWT-2 .2); RSI-2, RWT-3, class 2 (C-D17)")
    check(len(cands) == 1, "C-02 one gross fits")
    h, g = cands[0]
    anyg = gross_from_net_any(row["amount"], row["date"], 2, D("5000.00"), D("6500.00"), known=known)
    t("grosses in whole cents from 5 000 to 6 500", anyg, "without the contract's terms the net doesn't settle it")
    months = {}
    for k in F.months("2024-07", "2026-02"):
        s = slips[k]
        if k == "2025-10":
            check(s["gross"] == g and s["net"] == row["amount"], "C-02 the inferred slip is the employer's")
        months[k] = {"paid": s["paid"].isoformat(), "gross": s["gross"], "contribution": s["soc"], "tax": s["tax"], "net": s["net"]}
        check(s["net"] == s["gross"] - s["soc"] - s["tax"], f"C-02 net {k}")
    t("months", months, "C-D20, and October 2025 as above; each payment: RSI-2 13.50 % up to RSI-CEIL of the year paid, to the cent half-even (RGP-2); RWT-3 taxable = gross − RSI − allowance of the class (RWT-4: class 2 for pay paid after the notice reached the employer, C-D15, C-D17), down to 10, bands, down to whole; tax year by the day paid (RWT-1 .2); Rosvald's tax in full, no relief for Lenia's (T-8); paid on the 5th of the next month or the Rosvald business day before (RGP-1 .2); insured in Rosvald for each month of the work (RSI-1, T-10)")
    log = F.travel_log()
    p1 = t("presence 2024/25", F.presence(log, "2024-04-01", "2025-03-31"), "RWT-5: days in Rosvald at any time (records: dziennik-jana.csv, 'w Rosvaldzie' = tak)")
    p2 = t("presence 2025/26", F.presence(log, "2025-04-01", "2026-03-31"))
    a1 = assessment([s for s in A["lazur"]["slips"] if s["tax year"] == "2024/25"], "2024/25", 2, known=known)
    a2 = assessment([s for s in A["lazur"]["slips"] if s["tax year"] == "2025/26"], "2025/26", 2, known=known)
    t("assessment 2024/25", a1, "RWT-6 .2–.3: class 2 on 2025-03-31; agrees with C-D19")
    t("assessment 2025/26", a2, "RWT-6: 236 days ≥ 183; pay paid 2025-04-04 .. 2026-03-05 (October 2025 inferred), class 2 on 2026-03-31")
    check(p1 >= 183 and p2 >= 183, "C-02 presence")
    key = {"months": months, "October 2025 gross": g, "presence": {"2024/25": p1, "2025/26": p2},
           "assessment": {"2024/25": dirn(-a1["refund"]), "2025/26": dirn(-a2["refund"])}}
    put("C-02", ["HID-03", "REC-05", "TIME-04", "NUM-03", "LAW-07", "TIME-03"], "2026-03-31", key, t,
        relations=["each month's net = gross − contribution − tax", "October 2025's net is the Lazur row", "the yearly refund = withheld − yearly tax"])
    return a2

# --------------------------------------------------------------------------- C-03, C-04


def ret(r):
    return {"Marta base": r["marta base"], "Jan base": r["jan base"], "deductions": {"Marta": r["ded marta"], "Jan": r["ded jan"]},
            "combined": r["combined"], "joint tax": r["joint tax"], "relief": r["relief"], "credit": r["credit"], "due": r["due"],
            "advances": r["advances"], "difference": dirn(r["payable"])}


def adj(x):
    x = D(x)
    return {"add": x} if x >= 0 else {"take off": -x}


def q03():
    known = "2025-12-31"   # R1: C-03's known-as-of date
    t = Trace()
    gift = S.TITHE[2024][1]
    f = X.year_2024(gift, known=known)
    c = X.year_2024(gift, corrected=True, known=known)
    t("Marta's base", f["marta base"], f"PIT-2, PIT-5.2: employer's base {f['marta']['employer']} (C-D05) + the SIO's {f['marta']['sio']} (C-D04), rounded")
    t("Jan's foreign pay", [(x["work"], x["paid"].isoformat(), x["rate"], x["base"], x["tax DUK"]) for x in f["foreign"]],
      "PIT-16 .1, PIT-18 .1–.2: each payment paid in 2024 converted at FX-FLR of the last business day before the pay day, each to the ort; base = gross − RSI − 250.00")
    t("Jan's base", f["jan base"], f"PIT-5.2: business profit {f['workshop']['profit']} (PIT-10 (a), C-01) + foreign base {f['foreign base']}, rounded; no treaty for 2024 (T-20 .2, protocol 2)")
    t("deductions", (f["ded marta"], f["ded jan"]), f"PIT-17: the gift {gift} from the joint account, half each; capped at 10 % of each base: Marta's {f['pit17']['cap Marta']}, Jan's {f['pit17']['cap Jan']}; Marta's excess lost")
    t("combined", f["combined"], "PIT-6.3: rounded")
    t("joint tax", f["joint tax"], "PIT-6.3: twice PIT-3 v3 on half")
    t("relief", f["relief"], f"PIT-7: parents' income {f['parents income']} (before PIT-17, .4) far above 112 000.00, so 0")
    t("credit as filed", f["credit"], f"PIT-16 .3, PIT-18 .5: the lower of the Rosvald tax {f['foreign tax']} and the part {f['credit part']} = tax × foreign base / combined; rounded")
    t("as filed", ret(f))
    sh, a = X.refund_shares(known=known)
    t("the refund's shares", sh, f"PIT-18 .3: the 2024/25 refund {a['refund']} FLR (C-D19) shared by gross, each to the cent, converted at FX-FLR of 2025-09-23 (received 2025-09-24)")
    t("credit as corrected", c["credit"], f"the 2024 payments' Rosvald tax less their shares = {c['foreign tax']}, now below the part, so the credit")
    t("as corrected", ret(c))
    paid = c["due"] - f["due"]
    t("paid with the correction", paid, "the corrected tax due less the filed; by the corrected return's day (PIT-18 .3), the joint account's row of 2025-10-20")
    tf = X.tenth(X.year_2024, known=known)
    tc = X.tenth(X.year_2024, corrected=True, known=known)
    t("tenth as filed", tf, "C-HR6 .2: the smallest whole dukat G ≥ (bases before the gift − tax due with G deducted) / 10")
    t("tenth as corrected", tc, "the same on the corrected return (the return last filed before December 2025's gift, C-HR6 .3)")
    nxt = t("for next December", tc["G"] - gift, "C-HR6 .3: the tenth less the 21 300.00 given")
    check(tc["G"] >= tc["check"] and tc["G"] - 1 < (tc["B"] - X.year_2024(tc["G"] - 1, corrected=True, known=known)["due"]) / 10, "C-03 smallest tenth")
    check(c["credit"] <= f["credit"], "C-03 the correction lowers the credit")
    key = {"filed": ret(f), "corrected": ret(c) | {"paid with it": way(paid)},
           "tenth": {"as filed": tf["G"], "as corrected": tc["G"], "given": gift, "for next December": adj(nxt)}}
    put("C-03", ["HID-02", "LAW-08", "CUR-01", "CUR-02", "PTY-04", "HID-05", "LAW-10"], "2025-12-31", key, t,
        relations=["difference = due − advances", "the corrected credit is never above the filed", "the tenth is the smallest whole dukat that meets C-HR6 .2"])
    return tc["G"] - gift


def q04(adj24):
    known = "2026-09-30"   # R1: C-04's known-as-of date
    t = Trace()
    gift = S.TITHE[2025][1]
    given = gift - adj24
    t("given for 2025", given, f"C-HR6 .3: December 2025's {gift} less what it carried for 2024 ({adj24})")
    r = X.year_2025(gift, reading="paid", known=known)
    tn = X.tenth(X.year_2025, reading="paid", known=known)
    t("return as filed", ret(r) | {"exempt": r["exempt"]},
      "C-D34: the return states Jan's pay paid from 2025-01-03 on as exempt (resident of Lenia, his home there: T-2; taxed where the work is done: T-5 .1; T-7, T-20 .2), with progression (PIT-18 .4), out of the PIT-7 and PIT-17 bases")
    t("tenth", tn, "C-HR6 .2 on the return as filed, the one C-HR6 .3 measures by")
    key = ret(r) | {"exempt": r["exempt"], "tenth": tn["G"], "for December 2026": adj(tn["G"] - given)}
    check(r["payable"] == r["due"] - r["advances"], "C-04 difference")
    put("C-04", ["HID-01", "LAW-08", "LAW-02", "TIME-03", "HID-05"], "2026-09-30", key, t,
        relations=["difference = due − advances", "the exempt pay is in no base of PIT-7.2 or PIT-17"])

# --------------------------------------------------------------------------- C-05


def q05(A):
    t = Trace()
    ben = A["lazur"]["benefits"]
    fbo = {k: a for d, a, k in S.fbo_payments()}
    months, over = {}, ZERO
    for k in F.months("2024-07", "2026-09"):
        b = ben[k]
        months[k] = {"Lenia": b["Lenia"], "Rosvald": b["Rosvald"]}
        # R2: no FBO payment 2025-12 .. 2026-08: C-D25 "Payments stop from December 2025", C-D26 resumes September 2026; the joint export covers 01.01.2024 - 30.09.2026
        over += fbo.get(k, ZERO) - b["Lenia"]
    t("months", {k: (ben[k]["first"], v) for k, v in months.items()},
      "T-11, T-12 .1: Lenia first while Marta is paid for work in Lenia; Rosvald first in her unpaid leave 2025-09 .. 2026-08 (C-D07); the difference converted by T-12 .2 (RNB-DUK, RGP-3, for Rosvald, FX-FLR on or before the 1st for Lenia), half up; the RFF's benefit for each month Jan is insured, only what T-12 gives (RFF-1 .1, .2); each change of who pays first from its month (T-13 .1)")
    t("overpaid", over, "the FBO's rows in the joint account less what FAM-1 .6 / T-12 gave: September–November 2025; recovered under the home law (T-13 .2)")
    reps = [CAL.on_or_next(d) for d, a in S.FBO_REPAY]
    t("repaid", (sum((a for d, a in S.FBO_REPAY), ZERO), reps), "FAM-2 .2, C-D25: six instalments of 325.00, the joint account's rows")
    q3 = sum((ben[k]["Rosvald"] for k in ("2026-07", "2026-08", "2026-09")), ZERO)
    by = F.bd_on_or_before(date(2026, 10, 31), RCAL)
    t("owed by Rosvald on 2026-09-30", q3, f"RFF-2 .2: the third quarter of 2026 (July, August in full, September's difference), by {by}")
    rff_paid = sum((a for d, a, _ in A["lazur"]["rff"]), ZERO)
    due_paid = sum((ben[k]["Rosvald"] for k in F.months("2024-07", "2026-06")), ZERO)
    check(rff_paid == due_paid, "C-05 the RFF's payments to June 2026 are what was due")
    check(over == D("1950.00"), "C-05 overpaid")
    key = {"months": months, "overpaid": {"amount": over, "repaid": sum((a for d, a in S.FBO_REPAY), ZERO), "last instalment": reps[-1].isoformat()},
           "owed on 2026-09-30": {"Rosvald": {"amount": q3, "by": by.isoformat()}, "Lenia": ZERO}}
    without = {"months": {k: {"Lenia": D("650.00"), "Rosvald": ZERO} for k in months}, "overpaid": {"amount": ZERO, "repaid": ZERO},
               "owed on 2026-09-30": {"Rosvald": {"amount": ZERO}, "Lenia": ZERO}}
    t("without the switch", "no parent works in Rosvald: Lenia alone pays, 650.00 a month (FAM-1 v2); nothing is overpaid")
    put("C-05", ["HID-04", "PTY-04", "LIFE-04", "CUR-01", "CUR-02", "OWN-03", "TIME-05"], "2026-09-30", key, t, without,
        ["overpaid = Lenia's payments − Lenia's dues", "the RFF's payments to June 2026 equal its dues"])

# --------------------------------------------------------------------------- C-06


def q06(A):
    known = "2026-09-30"   # R1: C-06's known-as-of date
    t = Trace()
    lr = lazur_rows()
    tab = S.lazur_table(known=known)
    sales = {}
    for r in lr:
        if r["cur"] == "FLR" and r["type"] == "SPRZEDAŻ WALUTY" and r["cp"].startswith("Skarbonka"):
            rate = D(r["title"].split(" po ")[1])
            check(rate == tab[r["date"]][0], f"C-06 rate {r['date']}")
            sales[r["date"].isoformat()] = {"florens": -r["amount"], "rate": rate, "dukats": half_up(-r["amount"] * rate)}
    t("sales into the pot", sales, "records: lazur-jan.csv, the sales to the pot; dukats = florens × Bank Lazur's buy rate, to the ort (the pot's rows agree)")
    rules = {s["date"].isoformat(): s["rule"] for s in A["lazur"]["sales"] if s["to"] == "pot"}
    t("under", rules, "C-HR2 .2: every published buy rate of the month ≥ 4.1100 (market/lazur-fx-flr.csv), at most once a quarter; .3 otherwise, month-end above 14 000.00")
    due = date(2026, 1, 2)
    fl = flr_end(lr, due - timedelta(days=1)) - D("1500.00")
    r0 = tab[due][0]
    t("the sale due on 2026-01-02", (fl, r0, half_up(fl * r0)), "C-HR2 .2: December 2025's buy rates all ≥ 4.1100; first business day 2026-01-02 (Jan 1 a holiday); florens above 1 500.00 at the end of 2026-01-01")
    made = sales["2026-01-08"]
    held = flr_end(lr, S.T1)
    hd = half_up(held * tab[S.T1][0])
    t("held on 2026-09-30", (held, tab[S.T1][0], hd), "the FLR side's balance; C-HR9 .1: Bank Lazur's buy rate of the day")
    tot = sum((r["amount"] for r in lr if r["cur"] == "FLR"), ZERO)
    check(tot == held, "C-06 the FLR rows add up to the balance")
    qs = {}
    for d0, rl in rules.items():
        if rl == "C-HR2 .2":
            q = (d0[:4], (int(d0[5:7]) - 1) // 3)
            check(q not in qs, "C-06 one point-2 sale a quarter")
            qs[q] = d0
    key = {"sales": sales, "off-rule": {"made on": "2026-01-08", "florens made": made["florens"], "due on": due.isoformat(),
                                        "florens due": fl, "dukats due": half_up(fl * r0)},
           "held": {"florens": held, "dukats": hd}}
    put("C-06", ["CUR-01", "CUR-03", "RULE-07", "RULE-05", "RULE-01", "CUR-02"], "2026-09-30", key, t,
        relations=["the FLR rows add up to the balance held", "at most one point-2 sale a calendar quarter", "dukats = florens × buy rate, to the ort"])

# --------------------------------------------------------------------------- C-07


def q07(A):
    t = Trace()
    log = F.travel_log()
    inst, _ = S.instalments_by_month()
    rule = {k: F.hr3_due(k, log, inst[k]["amount"], F.hr3_version(k)) for k in F.months("2024-08", "2025-11")}
    paid = {s["for"]: s["duk"] for s in A["lazur"]["sales"] if s.get("for")}
    t("C-HR3 v2 from", "2025-02", "C-HR1: second signature 2025-04-02 → May 2025, but v2 names January 2025; the settle-up of 2025-02-28 closed August 2024 – January 2025, so February 2025")
    months = {}
    for k in F.months("2025-01", "2025-11"):
        r = rule[k]
        months[k] = {"part": r["due"], "paid": paid[k]}
        t(f"{k}", (F.hr3_version(k), r["costs"], r["nights home"], r["nights"], r["due"], paid[k]),
          "half of (instalment + bills + food) × nights at home / nights, to the ort; paid: the Lazur sale for the month")
    s1 = sum((rule[k]["due"] - paid[k] for k in F.months("2024-08", "2025-01")), ZERO)
    s2 = sum((rule[k]["due"] - paid[k] for k in F.months("2025-02", "2025-07")), ZERO)
    t("settle-up 2025-02-28", s1, "C-HR3 .5: August 2024 – January 2025")
    t("settle-up 2025-08-31", s2, "February – July 2025: Jan used v1 to April; due within 7 days, by 2025-09-07; nothing transferred (story)")
    check(s1 == 0, "C-07 the first settle-up is even")
    # cases 1.2.0, ERR-39 (session 5r's K-1): C-HR4 .4 makes a purchase made without a needed yes the buyer's own, paid
    # back into the joint account at the next settle-up; the bike (C-08) needed one under two of its three readings, its
    # settle-up was 2025-08-31 and nothing was paid back (story), so on 2025-12-31 Jan owes it too, by reading
    from c_law import gen4
    tab = S.lazur_table(known="2025-12-31")
    b, s = tab[date(2025, 4, 10)]
    lim = D("2500.00")
    owed = {}
    for label, v in (("at the CBL's mid rate, as GEN-4 converts", gen4(D("600.00"), date(2025, 4, 10), known="2025-12-31")[0]),
                     ("at Bank Lazur's buy rate", half_up(D("600.00") * b)),
                     ("at Bank Lazur's sell rate", half_up(D("600.00") * s))):
        bike = v if v > lim else ZERO
        owed[label] = way(s1 + s2 + bike)
        t(f"owed on 2025-12-31, the bike {label}", (s1 + s2, bike, s1 + s2 + bike),
          "the settle-ups (C-HR3 .5) plus the bike where it needed a yes it never had, due at the 2025-08-31 settle-up (C-HR4 .4), nothing paid back")
    key = {"months": months, "settle-ups": {"2025-02-28": way(s1), "2025-08-31": way(s2)}, "owed on 2025-12-31": by_case(owed)}
    wo = {}
    for k in F.months("2025-01", "2025-11"):
        nh = sum(1 for r in log if F.ym(r["date"]) == k and r["night"] != "wyjazd")
        r = rule[k]
        wo[k] = {"part": ort(r["costs"] / 2 * nh / r["nights"])}
    t("without the switch", {k: v["part"] for k, v in wo.items()}, "no job in Rosvald, so no nights in Marrenburg: only the holidays away")
    put("C-07", ["RULE-06", "FLOW-03", "OWN-03", "TIME-03", "HID-05"], "2025-12-31", key, t, {"months": wo},
        ["owed = the settle-ups' sum, plus the bike under the readings where it needed a yes (C-HR4 .4)", "a month's part is never above half its costs"])

# --------------------------------------------------------------------------- C-08


def q08():
    known = "2026-09-30"   # R1: C-08's known-as-of date
    t = Trace()
    tab = S.lazur_table(known=known)
    from c_law import gen4
    b, s = tab[date(2025, 4, 10)]
    lim = D("2500.00")

    def bike(rate, label, v=None):
        v = half_up(D("600.00") * rate) if v is None else v
        t(f"bike at {label}", v, f"600.00 FLR × {rate}" if rate else "GEN-4: the CBL mid rate of the last business day before 2025-04-10, to the ort")
        return {"needed yes": v > lim, "had yes": False, "owed back": {"Jan": v} if v > lim else ZERO,
                "due": "2025-08-31" if v > lim else None}
    key = {
        "washing machine": {"needed yes": True, "had yes": True, "owed back": ZERO, "due": None},
        "bike": by_case({"at the CBL's mid rate, as GEN-4 converts": bike(None, "the CBL mid of 2025-04-09", gen4(D("600.00"), date(2025, 4, 10), known=known)[0]),
                         "at Bank Lazur's buy rate": bike(b, "Lazur's buy"),
                         "at Bank Lazur's sell rate": bike(s, "Lazur's sell")}),
        "sofa": {"needed yes": True, "had yes": False, "owed back": {"Marta": D("3400.00")}, "due": "2026-02-28"},
        "laptop": {"needed yes": True, "had yes": False, "owed back": {"Jan": D("4199.00")}, "due": "2026-02-28"},
        "shed": {"needed yes": True, "had yes": True, "owed back": ZERO, "due": None},
        "tiles": {"needed yes": True, "had yes": False, "owed back": {"Marta": D("2800.00")}, "due": "2026-08-31"},
    }
    t("washing machine", "2 899.00 > 2 500.00; asked 2025-02-03, 'Tak.' 2025-02-04 (zgody.txt)")
    t("sofa", "asked 2025-11-27, no answer; the 7 days end on 2025-12-04, in December, so the silence is no yes (C-HR4 .2); bought 2025-12-05 by Marta; next settle-up 2026-02-28 (C-HR4 .4)")
    t("laptop", "asked 2025-12-10; 'Nie' on 2025-12-11; bought 2025-12-20 by Jan; next settle-up 2026-02-28 (C-HR4 .4)")
    t("shed", "asked 2026-03-02; silence is a yes from 2026-03-09, 'Tak' on 2026-03-12; bought 2026-03-14")
    t("tiles", "one invoice of 2 800.00 (C-D33) paid as two card rows of 1 400.00: one purchase (C-HR4 .1); no ask; next settle-up 2026-08-31")
    put("C-08", ["NUM-04", "CUR-01", "TIME-05", "RULE-02", "HID-05", "REC-04"], "2026-09-30", key, t,
        relations=["nothing is owed back where a yes was had or none was needed"])

# --------------------------------------------------------------------------- C-09


def gift_days():
    out = []
    d = date(2023, 4, 3)
    while d <= S.T1:
        out.append(CAL.on_or_next(d))
        d = add_months(d, 1)
    return out


def q09():
    t = Trace()
    op, rows = jodla_rows("jodla-wspolny.csv")
    gd = gift_days()
    short_days, worst, worst_day = 0, ZERO, None
    for d in F.days(S.T0, S.T1):
        sent = 300 * sum(1 for g in gd if g <= d)
        b = end_balance(rows, op, d)
        if b < sent:
            short_days += 1
            if sent - b > worst:
                worst, worst_day = sent - b, d
    sent = D(300 * len(gd))
    b = end_balance(rows, op, S.T1)
    held = min(sent, b)
    t("sent", sent, f"{len(gd)} gifts of 300.00 from 2023-04-03 (C-D36, the joint account's rows from 2024)")
    t("balance on 2026-09-30", b, "records: jodla-wspolny.csv")
    t("held", held, "C-HR5 .2: Staś's money first, up to all he has been sent")
    t("days short", short_days, "day-ends from 2024-01-01 with the balance below all he had been sent by then")
    t("largest shortfall", (worst, worst_day))
    due = [CAL.on_or_next(month_after(g)) for g in gd]
    over = sum(1 for x in due if x <= S.T1)
    nxt = min(x for x in due if x > S.T1)
    t("gift tax", (over, nxt), "IGT-4.4: money credited to the parents' account, not Staś's, so no exemption; IGT-5.1: a return within 1 month of each gift (GEN-3.1, GEN-2.2); IGT-3: 5 years' gifts added, at most 12 600.00, below IGT-FREE-I 44 000.00, so no tax")
    check(held + (sent - held) == sent, "C-09 held + short = sent")
    key = {"sent": sent, "held": held, "short": sent - held, "days short": short_days,
           "largest shortfall": {"amount": worst, "on": worst_day.isoformat()},
           "gift tax": {"returns overdue": over, "next return due": nxt.isoformat(), "tax": ZERO}}
    put("C-09", ["OWN-02", "RULE-01", "PTY-04", "TIME-05", "LAW-07", "LAW-11"], "2026-09-30", key, t,
        relations=["held + short = sent", "held = the lower of sent and the balance"])

# --------------------------------------------------------------------------- C-10


def q10(A, a2526):
    known = "2026-08-31"   # R1: C-10's known-as-of date
    t = Trace()
    on = date(2026, 8, 31)
    tab = S.lazur_table(known=known)
    buy = tab[on][0]
    op, rows = jodla_rows("jodla-wspolny.csv")
    j = end_balance(rows, op, on)
    sent = D(300 * sum(1 for g in gift_days() if g <= on))
    stas = min(sent, j)
    pop, prow = jodla_rows("jodla-skarbonka.csv")
    pot = end_balance(prow, pop, on)
    lr = lazur_rows()
    flr = flr_end(lr, on)
    flr_d = half_up(flr * buy)
    mop, mrows = jodla_rows("jodla-marta.csv")
    macc = end_balance(mrows, mop, on)
    net_int = sum((r["amount"] for r in mrows if r["date"] <= on and r["type"] in ("ODSETKI", "PODATEK")), ZERO)
    mcomm = D("2104.37") + net_int
    led = A["loan"]["ledger"]
    owed = [x for x in led if x["date"] <= on][-1]["owed"]
    mort = [r for r in A["mortgage"] if r["date"] <= on][-1]["balance"]
    rff = A["lazur"]["benefits"]["2026-07"]["Rosvald"] + A["lazur"]["benefits"]["2026-08"]["Rosvald"]
    claims = half_up(rff * buy) + half_up(a2526["refund"] * buy)
    duk_own = [r for r in lr if r["cur"] == "DUK"][-1]["balance"]
    t("joint account", (j, stas), "end of 2026-08-31; less Staś's money (C-HR9 .6, C-HR5)")
    t("pot", pot)
    t("florens", (flr, buy, flr_d), "C-HR9 .1")
    t("claims", (rff, a2526["refund"], claims), "C-HR9 .3: the RFF's benefit for July and August 2026, set by its decisions; the 2025/26 refund, claimed 2026-07-14 (C-02); C-HR9 .1: each at the buy rate on its own, to the ort")
    t("Marta's account", (macc, mcomm), "C-D35: 2 104.37 of interest since the wedding, plus the net interest since (records): community (CIV-1.2); the rest her own")
    t("the loan", owed, "C-HR7, after August's interest: the community owes it to Marta, set by the agreement (C-HR9 .3); C-HR9 .7 reaches only C-HR3 and C-HR4")
    t("the mortgage", mort, "C-HR9 .5: principal after the instalment and the extra of 2026-08-17")
    t("Jan's dukats at Lazur", duk_own, "what is left of Marta's gift after the car: his own")
    comm = (j - stas) + pot + flr_d + claims + mcomm - mort - owed
    t("community", comm)
    m = comm / 2 + (macc - mcomm) + owed
    jn = comm / 2 + duk_own
    t("Marta", m, "half the community, her own account's own part, the loan owed to her")
    t("Jan", jn, "half the community, his dukats; the car is his own, never valued")
    check(m + jn == comm + (macc - mcomm) + owed + duk_own, "C-10 the two add up")
    house = "the house ul. Graniczna 12, Zabór: community, never valued (C-HR9 .4)"
    car = "the car Jan bought on 2024-07-02 with Marta's gift: his own, never valued (C-HR9 .4)"
    key = {"together": {"kind": "partial", "value": m + jn, "missing": [house, car]},
           "Marta": {"kind": "partial", "value": m, "missing": [house]},
           "Jan": {"kind": "partial", "value": jn, "missing": [house, car]}}
    put("C-10", ["OWN-04", "OWN-02", "OWN-03", "CUR-03", "NUM-01", "OWN-01", "OWN-05", "ASK-02", "ASK-03"], "2026-08-31", key, t,
        relations=["Marta + Jan = together", "Staś's money is in no one's"])

# --------------------------------------------------------------------------- C-11


def q11(A):
    t = Trace()
    led = A["loan"]["ledger"]
    rel = A["loan"]["released"]
    intr = sum((x["amount"] for x in led if x["kind"] == "interest"), ZERO)
    rep = -sum((x["amount"] for x in led if x["kind"] == "repaid"), ZERO)
    t("interest", intr, "C-HR7 .2: daily at CBL-REF (home-gazette-supplement) / 365 on what was owed at the day's start, added at each month's end and on 2026-09-16, to the ort: " +
      "; ".join(f"{x['date']} {x['amount']}" for x in led if x["kind"] == "interest"))
    t("repaid", rep, "C-HR7 .3: 14 payments from 2025-08-11 to 2026-09-10 (the joint account's rows)")
    t("owed at the forgiveness", rel, "30 000.00 + interest − repaid")
    h = rel / 2
    t("gift to Jan", h, "IGT-2 v1 .4: a community debt to Marta released, so a gift to Jan of half, carried exactly (GEN-5.5)")
    tot = D("40000.00") + duk(h)
    t("added up", tot, "IGT-2.3: the release's base rounded; IGT-3.1: Marta's gift of 40 000.00 on 2024-07-01 is within 5 years")
    tax_b = duk((tot - D("44000.00")) * D("0.03"))
    t("tax as Group I", tax_b, "IGT-3.3: 3 % of the part above IGT-FREE-I 44 000.00")
    rb = CAL.on_or_next(month_after(S.LOAN["forgiven"], 6))
    rt = CAL.on_or_next(month_after(S.LOAN["forgiven"], 1))
    key = {"owed at the forgiveness": rel, "interest": intr, "repaid": rep, "gift": {"Marta": ZERO, "Jan": h},
           "gift tax": {"kind": "by-case", "cases": {
               "the release is not a gift of money, reported by 2027-03-17": {"report needed": True, "by": rb.isoformat(), "return by": None, "tax": ZERO},
               "the release is not a gift of money, not reported by 2027-03-17": {"report needed": True, "by": rb.isoformat(), "return by": rt.isoformat(), "tax": tax_b},
               "the release is a gift of money, never credited": {"report needed": False, "by": None, "return by": rt.isoformat(), "tax": tax_b}},
               "settled by": ["whether IGT-4 .4 reaches a released debt", f"whether Jan reports the release to the RO by {rb.isoformat()}"]}}
    t("the cases", (rb, rt), "IGT-4.1–4.3: exempt if reported within 6 months, as the sum is above 44 000.00; without the report, taxed as Group I (IGT-4.2), with a return within 1 month (IGT-5.1; 2026-10-17 a Saturday); "
      "IGT-4.4: a gift of money is exempt only if credited to his account, and no report lifts that; the report is pending until 2027-03-17")
    check(rel == S.LOAN["amount"] + intr - rep, "C-11 owed = lent + interest − repaid")
    without = {"owed at the forgiveness": rel, "interest": intr, "repaid": rep, "gift": {"Marta": ZERO, "Jan": h},
               "gift tax": by_case({
                   "the release is not a gift of money": {"report needed": False, "by": None, "return by": None, "tax": ZERO},
                   "the release is a gift of money, never credited": {"report needed": False, "by": None, "return by": rt.isoformat(), "tax": ZERO}},
                   ["whether IGT-4 .4 reaches a released debt"])}
    t("without the switch", "no car for the commute, so no gift of 2024-07-01: the half alone is below 44 000.00")
    put("C-11", ["LIFE-03", "OWN-05", "LAW-11", "TIME-05", "NUM-04", "PTY-04"], "2026-09-30", key, t, without,
        ["owed = lent + interest − repaid", "the gift is half of what was released"])

# --------------------------------------------------------------------------- C-12


def q12(A):
    t = Trace()
    pop, prow = jodla_rows("jodla-skarbonka.csv")
    pot0 = end_balance(prow, pop, S.T1)
    t("pot on 2026-09-30", pot0, "records")
    ex = S.extras()
    d, pot, n, last = date(2026, 10, 16), pot0, 0, None
    while pot >= S.EXTRA:
        ex.append((CAL.on_or_next(d), S.EXTRA))
        pot -= S.EXTRA
        if CAL.on_or_next(d) <= date(2027, 9, 30):
            n, last = n + 1, CAL.on_or_next(d)
        d = add_months(d, 1)
    rows_w, _ = F.mortgage(ex)
    rows_o, _ = F.mortgage(S.extras())
    pay_w = [r for r in rows_w if r["kind"] == "instalment"][-1]
    pay_o = [r for r in rows_o if r["kind"] == "instalment"][-1]
    # extras after the payoff aren't made
    made = [d0 for d0, a in ex if date(2026, 10, 1) <= d0 <= min(date(2027, 9, 30), pay_w["date"])]
    n, last = len(made), max(made)
    low = pot0 - S.EXTRA * n
    t("with extras", (n, low, last, pay_w["date"], pay_w["amount"]), "the scenario: each month's 800.00 while the pot holds it; C-D27 instalments, interest on the principal of the day before")
    t("without extras", (pot0, pay_o["date"], pay_o["amount"]))
    k = int((pot0 - D("5000.00")) // S.EXTRA)
    dip = CAL.on_or_next(add_months(date(2026, 10, 16), k))
    check(1 <= k < 12, "C-12 the dip falls inside October's 12 months")
    check(not any(r["amount"] > 0 for r in prow if r["date"] > S.T1), "C-12 nothing into the pot after September 2026")
    key = {"with extras": {"pot lowest": low, "on": last.isoformat(), "paid off": pay_w["date"].isoformat(), "last instalment": pay_w["amount"]},
           "without extras": {"pot lowest": pot0, "paid off": pay_o["date"].isoformat(), "last instalment": pay_o["amount"]},
           "October 2026 extra": refusal(["what picks one plan among those C-HR8 .2–.3 allow: no rule says which, and none names a way to solve the loop"],
                                         f"a loop: our plan is the months C-HR8 pays, and C-HR8 pays by our plan; the plan paying the {k} months from October 2026 meets the rule, and so does the one skipping October and paying the {k} months after it")}
    t("C-HR8 .2–.3", f"the story's forecast: nothing into the pot to 2027-09-30, nothing out but the extras, so at most {k} extras keep it at or above 5 000.00 ({pot0} − {k} × 800.00 = {pot0 - 800 * k}); a {k + 1}th leaves {pot0 - 800 * (k + 1)}. "
      f"Plan 1, the {k} months from October 2026: each paid month's plan stays at {pot0 - 800 * k}, so pay; each later month paid would leave {pot0 - 800 * (k + 1)} on its own day, so skip: it meets the rule. "
      f"Plan 2, skip October, pay the {k} months after it: October paid with them would leave {pot0 - 800 * (k + 1)} on {dip}, inside its 12 months, so skip; the rest as plan 1: it meets the rule too. "
      "October is paid in one and skipped in the other: a loop the rule doesn't settle (RULE-04)")
    check(low == pot0 - 800 * n, "C-12 pot")
    put("C-12", ["RULE-09", "RULE-04", "HID-05", "LIFE-03", "REC-01", "ASK-03"], "2026-09-30", key, t,
        relations=["pot lowest with extras = pot − 800.00 × extras paid", "the extras bring the payoff forward"])
    check(pay_w["date"] < pay_o["date"], "C-12 payoff order")


def main():
    taxes, _ = X.household_taxes(known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    A = S.accounts(taxes, known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    q01(A)
    a2526 = q02(A)
    adj24 = q03()
    q04(adj24)
    q05(A)
    q06(A)
    q07(A)
    q08()
    q09()
    q10(A, a2526)
    q11(A)
    q12(A)
    if FAILS:
        print("relations failed:", FAILS)
        sys.exit(1)
    print("C keys written: 12, relations hold")


if __name__ == "__main__":
    main()
