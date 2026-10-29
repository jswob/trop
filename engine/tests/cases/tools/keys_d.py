#!/usr/bin/env python3
"""Household D's keys: one file per question in keys/D/, each a YAML block and the derivation.

Reads the home law (rules as code in fam/home.py, fam/d.py and fam/d_paths.py), the gazette, the market
files, D's records (Bank Olcha's exports) and the facts D's documents state (fam/d.py, each citing its
document). Asserts every relation on its own output; exits 1 if one fails.

Run: python3 -I tmp/cases/tools/keys_d.py [n]   (n: forecast paths, default 10000)
"""
import csv
import math
import pathlib
import sys
from datetime import date, timedelta
from decimal import Decimal, ROUND_CEILING, ROUND_DOWN
from fractions import Fraction

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from fam import d as F, d_paths as LP  # noqa: E402
from fam.home import ALL_KNOWN, CAL, ZERO, D, duk, figure, ort  # noqa: E402
from lib.dates import add_months, months_back, month_key  # noqa: E402
from lib.keys import Trace, enc  # noqa: E402
from lib.paths import mean_se, dec  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[1]
REC = ROOT / "D" / "records"
FAILS = []
N_PATHS = int(sys.argv[1]) if len(sys.argv) > 1 else 10000


def check(cond, what):
    if not cond:
        FAILS.append(what)


def dirn(x):
    """A direction word; where nothing moves, a bare 0 (README § Keys)."""
    x = D(x)
    if x == 0:
        return ZERO
    return {"refund": -x} if x < 0 else {"payable": x}


def put(qid, edges, as_of, known, key, t, without=None, relations=()):
    head = [f"question: {enc(qid)}", f"edges: {enc(edges)}", f"as_of: {enc(as_of)}", f"known_as_of: {enc(known)}", f"key: {enc(key)}"]
    if without is not None:
        head.append(f"without: {enc(without)}")
    if relations:
        head.append(f"relations: {enc(list(relations))}")
    body = (f"# Key · {qid}\n\nWritten by `tools/keys_d.py`; don't edit.\n\n```yaml\n" + "\n".join(head) +
            "\n```\n\n## Derivation\n\n" + t.render() + "\n")
    p = ROOT / "keys" / "D" / f"{qid}.md"
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(body, encoding="utf-8")


def refusal(missing, reason):
    return {"kind": "refusal", "missing": missing, "reason": reason}


def mk(d):
    return f"{d.year}-{d.month:02d}"


# --------------------------------------------------------------------------- Bank Olcha's exports

def num(s):
    return D(s.replace(" ", "").replace(",", "."))


def ymd(s):
    return date(int(s[:4]), int(s[4:6]), int(s[6:]))


def read_olcha():
    """Every export chained by its balance-after column: a row that doesn't move the balance and repeats
    the row before is a repeat; a balance that jumps names a missing row of the jump's amount."""
    files = sorted(REC.glob("olcha-*.csv"))
    rows, problems, months = [], [], []
    prev_close = None
    for f in files:
        lines = [ln.split("|") for ln in f.read_text(encoding="utf-8").splitlines()]
        opening = num(lines[4][1])
        closing = num(lines[-1][1])
        if prev_close is not None:
            check(opening == prev_close, f"D records: {f.name} opens where the month before closed")
        bal = opening
        last = None
        for ln in lines[6:-1]:
            r = {"date": ymd(ln[0]), "type": ln[1], "cp": ln[2], "title": ln[3], "amount": num(ln[4]), "balance": num(ln[5]),
                 "raw": tuple(ln), "src": f.name}
            if bal + r["amount"] == r["balance"]:
                rows.append(r)
                bal = r["balance"]
                last = r
                continue
            if last is not None and r["raw"] == last["raw"] and r["balance"] == bal:
                problems.append({"export": f.stem[6:], "what": "the same row twice: the tenant's rent", "date": r["date"],
                                 "amount": r["amount"]})
                continue
            gap = r["balance"] - r["amount"] - bal
            problems.append({"export": f.stem[6:], "what": "a row missing: the building fee (D-D27: on the 5th)",
                             "between": (last["date"] if last else None, r["date"]), "amount": gap})
            rows.append(r)
            bal = r["balance"]
            last = r
        check(bal == closing, f"D records: {f.name} chains to its closing balance")
        months.append({"name": f.stem[6:], "opening": opening, "closing": closing})
        prev_close = closing
    return {"rows": rows, "problems": problems, "months": months}


def missing_fee_day(p):
    """D-D27: the fee is collected on the 5th, or the next business day; the one inside the gap."""
    a, b = p["between"]
    d = CAL.on_or_next(date(b.year, b.month, 5))
    check(a <= d <= b, "D-05 the missing fee's day sits inside the gap")
    return d


# --------------------------------------------------------------------------- D-01

def q01(sim, rc):
    K = "2025-06-16"   # R1: D-01's known-as-of date
    t = Trace()
    div = sim["division"]
    shares = {"Halina": Fraction(1, 4), "Piotr": Fraction(3, 16), "Anna": Fraction(3, 16), "Marta": Fraction(3, 16), "Jakub": Fraction(3, 16)}
    check(sum(shares.values()) == 1, "D-01 the shares add up to the whole")
    t("shares", shares, "CIV-3: the spouse 1/4 (with descendants); the rest equally to the children, Adam's part to Jakub by representation (D-D03, D-D04)")
    t("the flat", "Zbigniew's own", "D-D06: inherited from his mother, so it is separate (CIV-1) and wholly in the estate")
    per = {f"ISB{s}": F.bond_value(s, p, F.DIVISION, known=K) for s, p in F.SERIES}
    bonds = sum(per.values(), ZERO) * F.BONDS_EACH
    check(bonds == div["bonds"], "D-01 the bonds' value is the sum of the series' values per bond times 5")
    t("bonds on 2025-06-12", bonds, "325 bonds (D-D22; registered, TSB-1 .1) at their TSB-3 value on the agreement's day, per bond to the ort, × 5 per series")
    c = F.death_day_balance(known=K)
    end17 = [r for r in rc["rows"] if r["date"] <= F.DEATH][-1]["balance"]
    check(end17 == c, "D-01 the records' balance at the end of 2024-05-17 is the agreement's (D-D07, D-D33)")
    t("account at the end of 2024-05-17", c, "Bank Olcha's May 2024 export, the last balance after on the 17th; community, so half in the estate (CIV-2 .2)")
    t("Zbigniew's 2024 refund", F.ZB_REFUND, f"his 2024 return for the estate (D-D31): advances {F.zb_2024_return(known=K)['advances']} − tax {F.zb_2024_return(known=K)['tax']}")
    k = div["costs"]
    t("set-off: the flat's fees Halina paid", k, f"{len(F.estate_flat_costs())} fees from June 2024 to June 2025 (D-D27)")
    t("the Bank Buk loan", ZERO, "a community debt (D-D18), repaid by TU Opoka on 2024-07-12 at the principal outstanding on the day of the death; "
      "the 5 June instalment returned (D-D19, D-D20): not among the estate's debts")
    n = div["N"]
    t("the estate as divided", n, f"165 000.00 + {bonds} / 2 + {c} / 2 + {F.ZB_REFUND} − {k}")
    nk = div["N_kuba"]
    t("Jakub's part of it", nk, "the same less the flat, which he keeps 3/16 of")
    pay = {"Piotr": div["child"], "Anna": div["child"], "Marta": div["child"], "Jakub": div["kuba"]}
    t("payouts", pay, f"3/16 × {n} and 3/16 × {nk}, rounded down to whole dukats (D-D07 .4)")
    kept = (n * 3 / 16 - div["child"]) * 3 + (nk * 3 / 16 - div["kuba"])
    t("kept by the rounding", kept, "")
    row = [r for r in rc["rows"] if r["date"] == F.DIVISION and r["cp"].startswith("Notary")]
    check(len(row) == 1 and -row[0]["amount"] == div["escrow"], "D-01 the deposit row in the records is the payouts' sum")
    check(div["child"] == D("38750"), "D-01 each child's payout is A-D36's and B-D27's 38 750.00")
    t("into the notary's deposit", div["escrow"], "3 × 38 750 + Jakub's (records: 2025-06-12)")
    key = {"shares": shares, "bonds on 2025-06-12": bonds, "account at the death": c, "refund": F.ZB_REFUND,
           "set-off": k, "estate as divided": n, "payouts": pay, "paid on": F.PAYOUT_DAY, "into the deposit": div["escrow"],
           "kept by the rounding": kept, "flat after": {"Halina": Fraction(13, 16), "Jakub": Fraction(3, 16)}}
    put("D-01", ["OWN-04", "OWN-05", "PTY-01", "PTY-02", "PTY-03", "NUM-01", "NUM-05", "FLOW-03", "OWN-06", "LIFE-03"], "2025-06-16", "2025-06-16", key, t,
        relations=["the shares add up to the whole", "the deposit row is the payouts' sum", "each child's payout is the one A and B were told"])


# --------------------------------------------------------------------------- D-02

def q02():
    t = Trace()
    arose = F.CERT_FINAL
    t("the tax arises", arose, "IGT-2.1: the day the certificate became final (D-D04)")
    six = date(2025, 5, 3)
    check(not CAL.is_business(six), "D-02 the six months end on a day that isn't a business day")
    deadline = CAL.on_or_next(six)
    t("report by", deadline, "IGT-4.2: 6 months (GEN-3.1) = 2025-05-03, a Saturday and a holiday (3 May, GEN-1 v2 .2), moved to the next business day (GEN-2.2)")
    check(F.IGT_REPORT <= deadline, "D-02 Halina's report is in time")
    t("Halina", {"reported": F.IGT_REPORT, "tax": 0}, "D-D08: reported on 2025-05-05, in time: exempt (IGT-4.1)")
    kuba = {"kind": "by-case",
            "cases": {"reported by 2025-05-05": {"tax": 0},
                      "not reported": refusal(["the flat's value for the tax, as Jakub declares it or the RO sets it (IGT-2.2): "
                                               "whether his acquisition is above IGT-FREE-I (44 000.00), so that he had to report (IGT-4.3), "
                                               "and the Group I tax both turn on it"], "the dossier has no value the law takes")},
            "settled by": ["whether Jakub reported to the RO by 2025-05-05", "the flat's value as declared or set"]}
    t("Jakub", kuba, "nothing in the dossier says whether he reported; not exempt, he would file a return within a month (IGT-5 .1), and the RO would set the tax (IGT-5 .2)")
    t("the group life benefit", False, "D-D21: paid to the named beneficiary; neither inherited nor a gift (IGT-1), exempt from PIT (PIT-14)")
    key = {"tax arises": arose, "report by": deadline, "Halina": {"reported": F.IGT_REPORT, "tax": 0}, "Jakub": kuba,
           "group life taxed": False}
    put("D-02", ["LAW-12", "TIME-05", "PTY-03"], "2025-12-31", "2025-12-31", key, t,
        relations=["the report is in time only through the move to the next business day"])


# --------------------------------------------------------------------------- D-03

def pension_months(twist, *, known):
    out = {}
    m = date(2024, 1, 1)
    while m <= date(2026, 9, 1):
        if not twist or m <= date(2024, 5, 1):
            ent = paid = F.own(m, known=known)
        else:
            ent = F.surv(m, known=known) + F.part_law(m, known=known)
            paid = F.surv(m, known=known) + F.part_sio(m, known=known)
        out[mk(m)] = {"entitled": ent, "paid": paid}
        m = add_months(m, 1)
    return out


def q03(sim, rc):
    K = "2026-09-30"   # R1: D-03's known-as-of date
    t = Trace()
    cap = F.capital(F.H_EARNINGS, 2019, known=K)
    t("Halina's capital", cap, "SOC-5.3: Σ 19.5 % of each year's earnings (D-D10) × the VAL-IDX products to 2018")
    t("her pension from 2019-07-01", F.H_INITIAL, f"{cap} / LE-60 {F.le(60, F.H_START, known=K)} (SOC-5.4)")
    check(F.H_INITIAL == D("1918.31"), "D-03 the pension from the history is the decision's (D-D11)")
    zbd = F.zb(F.DEATH, known=K)
    t("Zbigniew's pension on 2024-05-17", zbd, f"{F.ZB_INITIAL} (D-D09) indexed each 1 March from 2020 (SOC-6)")
    t("survivor pension", F.SURV_BASE, f"85 % × {zbd}, from June 2024, indexed as his (SOC-7 v1 .1, .2)")
    months = pension_months(True, known=K)
    for k in ("2024-05", "2024-06", "2025-02", "2025-03", "2025-07", "2025-12", "2026-03", "2026-09"):
        t(f"month {k}", months[k])
    law_cap = 3 * F.min_pension(date(2025, 7, 1), known=K)
    sio_cap = 3 * F.SIO_CAP_USED
    t("the cap, July 2025", {"law": law_cap, "SIO": sio_cap}, "SOC-7 v2 .3: 3 × MIN-PENSION in force for the month (GEN-6 .1); the SIO read the figure in force at the death (D-D15)")
    owed = sum((v["entitled"] - v["paid"] for v in months.values()), ZERO)
    t("owed by the SIO on 2026-09-30", owed, "Σ entitled − paid, July 2025 – September 2026")
    arrears = 5 * F.part_sio(date(2025, 7, 1), known=K)
    t("arrears", arrears, "D-D15: July–November 2025's parts, paid with December's pension")
    # relation: the records' SIO credits are the law's nets as the SIO paid them
    got = sorted((r["date"], r["amount"]) for r in rc["rows"] if r["cp"] == "Social Insurance Office" and "Z. SOWA" not in r["title"])
    want = sorted((p["day"], p["net"]) for y in sim["sio"]["H"].values() for p in y)
    check(got == want, "D-03 the records' pension credits equal the SIO's nets (gross − health − advance)")
    check(owed == sum((F.part_law(add_months(date(2025, 7, 1), i), known=K) - F.part_sio(add_months(date(2025, 7, 1), i), known=K) for i in range(15)), ZERO),
          "D-03 the gap is the part's gap only")
    key = {"capital": cap, "pension from 2019-07-01": F.H_INITIAL, "Zbigniew's pension at the death": zbd,
           "Zbigniew's May 2024": {"paid on": F.pay_day(2024, 5), "due in full": True},
           "survivor pension from": date(2024, 6, 1), "months": months, "cap 2025-07": {"law": law_cap, "SIO": sio_cap},
           "arrears": {"paid on": date(2025, 12, 5), "amount": arrears}, "owed by the SIO": owed}
    wm = pension_months(False, known=K)
    without = {"months": wm, "owed by the SIO": ZERO}
    t("without the twist", "her own pension each month, paid as due", "")
    put("D-03", ["LIFE-04", "PTY-04", "LAW-03", "LAW-04", "LAW-07", "VAL-04", "VAL-06", "TIME-04"], "2026-09-30", "2026-09-30", key, t,
        without=without, relations=["the records' credits are the law's nets", "the gap is the part's gap only"])
    return owed


# --------------------------------------------------------------------------- D-04

def ret_of(pays):
    inc = sum((p["gross"] for p in pays), ZERO)
    adv = sum((p["advance"] for p in pays), ZERO)
    base = duk(inc)
    tax = duk(max(ZERO, F.scale(base, "v3") - F.PIT3["v3"]["reducing"]))
    return {"income": inc, "base": base, "tax": tax, "advances": adv, "difference": dirn(tax - adv)}


def q04(sim, rc):
    t = Trace()
    pen = ret_of(sim["sio"]["H"][2025])
    r25 = sim["returns"][2025]
    check(pen["tax"] - pen["advances"] == r25["due"], "D-04 the return's pension part is the simulation's")
    t("pensions 2025", pen, "PIT-5 v2: the SIO's gross paid in 2025, the December arrears included (paid in 2025); the scale v3 less 3 900")
    adv = [r for r in sim["rent_adv"] if r["month"].startswith("2025")]
    rent = adv[-1]["rent"]
    share = rent * 13 / 16
    tax = duk(share * D("0.085"))
    paid = sum((r["advance"] for r in adv), ZERO)
    rentk = {"received": rent, "her share": share, "tax": tax, "advances": paid, "difference": dirn(tax - paid)}
    t("rent 2025", rentk, "PIT-13: the rent received July–December, January 2026's included (paid 2025-12-30); 13/16 of it, her share (CIV-4 .1); 8.5 %")
    rec_adv = -sum((r["amount"] for r in rc["rows"] if r["title"].startswith("PIT-13") and r["title"].endswith("/2025 H. SOWA")), ZERO)
    check(rec_adv == paid, "D-04 the records' PIT-13 advances equal the advances the law asks")
    total = pen["tax"] - pen["advances"] + tax - paid
    t("total", dirn(total), "due 2026-04-30 (PIT-5); the bond interest isn't stated (TSB-8)")
    pay = [r for r in rc["rows"] if r["title"] == "PIT 2025 H. SOWA"]
    check(len(pay) == 1 and -pay[0]["amount"] == total, "D-04 the records' payment is the return's")
    key = {"pensions": pen, "rent": rentk, "bond interest stated": False, "total": dirn(total), "due": date(2026, 4, 30)}
    s0 = F.sim(False, known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    pen0 = ret_of(s0["sio"]["H"][2025])
    zero = {"received": ZERO, "her share": ZERO, "tax": 0, "advances": 0, "difference": dirn(0)}
    without = {"pensions": pen0, "rent": zero, "bond interest stated": False,
               "total": dirn(pen0["tax"] - pen0["advances"]), "due": date(2026, 4, 30)}
    t("without the twist", without, "her own pension only; the flat Zbigniew's and empty")
    put("D-04", ["LAW-04", "LIFE-04", "NUM-05"], "2026-04-30", "2026-04-13", key, t, without=without,
        relations=["the records' advances and payment are the return's"])


# --------------------------------------------------------------------------- D-05

def q05(sim, rc):
    t = Trace()
    probs = []
    for p in rc["problems"]:
        if "between" in p:
            d = missing_fee_day(p)
            probs.append({"export": p["export"], "what": p["what"], "date": d, "amount": p["amount"]})
            check(p["amount"] == -F.FEES[d.year], "D-05 the missing row is the month's fee")
        else:
            probs.append(p)
    t("the exports' problems", probs, "chaining each row's balance after")
    # the repaired records equal the simulation
    rows = list(rc["rows"]) + [{"date": p["date"], "amount": p["amount"], "title": "fee (restored)", "cp": "WM Ogrodowa 2"}
                               for p in probs if "missing" in p["what"]]
    got = sorted((r["date"], r["amount"]) for r in rows)
    want = sorted((r["date"], r["amount"]) for r in sim["rows"])
    check(got == want, "D-05 the repaired records equal the simulation")
    feb = [m for m in rc["months"] if m["name"] == "2026-02"][0]
    lines = [ln.split("|") for ln in (REC / "olcha-2026-02.csv").read_text(encoding="utf-8").splitlines()[6:-1]]
    chained = feb["opening"] + sum((num(ln[4]) for ln in lines), ZERO)
    dup = [p for p in probs if "twice" in p["what"]][0]
    check(chained - feb["closing"] == dup["amount"], "D-05 chaining the February amounts with the repeat is off by exactly its amount")
    t("February chained with the repeat", chained, f"less the closing {feb['closing']} = {chained - feb['closing']}")
    qs = {}
    for q in sim["quarters"]:
        e = {"rent": q["rent"], "costs": q["costs"], "share": q["share"], "carried in": q["carried in"], "to send": q["sent"],
             "due": q["due"]}
        qs[q["quarter"]] = e
    # rent and costs by quarter from the repaired records
    for qk, e in qs.items():
        y, n = int(qk[:4]), int(qk[-1])
        q0, q1 = date(y, 3 * n - 2, 1), add_months(date(y, 3 * n - 2, 1), 3) - timedelta(days=1)
        r_rent = sum((r["amount"] for r in rows if q0 <= r["date"] <= q1 and r.get("cp") == "Oskar Wilczek"), ZERO)
        r_cost = -sum((r["amount"] for r in rows if q0 <= r["date"] <= q1 and r.get("cp") in ("WM Ogrodowa 2", "Termo-Kłodzin sp.")), ZERO)
        check((r_rent, r_cost) == (e["rent"], e["costs"]), f"D-05 {qk}: the repaired records' rent and costs are the rule's")
    sent = {}
    for r in rc["rows"]:
        if r["cp"] == "Jakub Sowa":
            sent.setdefault(r["title"][9:16], []).append(r)
    for qk, e in qs.items():
        rs = sent.get(qk, [])
        if rs:
            last = rs[-1]
            check(-last["amount"] == e["to send"], f"D-05 {qk}: the transfer is the rule's")
            e["reached him"] = last["date"]
    t("quarters", qs, "D-HR1: 3/16 of (rent received − the flat's costs paid) in the quarter, to the ort; a minus carried; Q4 2025 returned and resent (D-D32)")
    owed = [e["to send"] for k, e in qs.items() if "reached him" not in e and e["to send"] > 0]
    t("owed to Jakub on 2026-09-30", sum(owed, ZERO), "2026-Q3's, due 2026-10-12")
    key = {"problems": probs, "quarters": qs, "owed to Jakub": sum(owed, ZERO)}
    put("D-05", ["REC-02", "FLOW-03", "OWN-02", "NUM-05"], "2026-09-30", "2026-09-30", key, t,
        relations=["dropping the repeat moves the replay by exactly its amount", "the repaired records equal the simulation",
                   "the records' rent and costs are the rule's"])
    return qs


# --------------------------------------------------------------------------- D-06

def held(sim, on):
    left = {f"ISB{s}": F.BONDS_EACH for s, _ in F.SERIES}
    for red in sim["redemptions"]:
        if red["request"] <= on:
            for x in red["lines"]:
                left[x["series"]] -= x["bonds"]
    return left


def year_start(p, on):
    k = 0
    while add_months(p, 12 * (k + 1)) <= on:
        k += 1
    return add_months(p, 12 * k)


def q06(sim):
    K = "2026-09-30"   # R1: D-06's known-as-of date
    t = Trace()
    on = date(2025, 12, 31)
    left = held(sim, on)
    n = sum(left.values())
    t("bonds held", n, "325 less those redeemed under D-HR2 (D-D24a..c)")
    val = acc = fee = tax = net = ZERO
    for s, p in F.SERIES:
        k = left[f"ISB{s}"]
        if not k:
            continue
        v = F.bond_value(s, p, on, known=K)
        val += v * k
        acc += (v - F.bond_value(s, p, year_start(p, on), known=K)) * k
        r = F.redeem(s, p, on, k, F.heir_base(s, p, known=K), known=K)
        fee += r["fee"] * k
        tax += r["tax"]
        net += r["net"]
    check(val - fee - tax == net, "D-06 value − fees − taxes = what a full redemption pays")
    t("value 2025-12-31", val, "TSB-3 per bond, to the ort, × bonds")
    t("accrued interest", acc, "per bond: the value less the value at the current interest year's start (TSB-3.2)")
    t("if all redeemed that day", {"fees": fee, "tax": tax, "paid": net}, "TSB-4.2, TSB-5 with TSB-6.2/TSB-7: the death-day value in place of 100.00; the tax rounded per series")
    on2, known = date(2026, 12, 31), date(2026, 9, 30)
    left2 = held(sim, date(2026, 9, 30))

    def cpi_known(ref, ys):
        return figure(f"CPI-YY-{ref}", ys, known)[0]

    v2, holes = ZERO, {}
    for s, p in F.SERIES:
        k = left2[f"ISB{s}"]
        if not k:
            continue
        try:
            v2 += F.bond_value(s, p, on2, cpi_known, known=K) * k
        except LookupError:
            ys = year_start(p, on2)
            ref = months_back(month_key(ys), 2)
            holes.setdefault(ref, []).append(f"ISB{s}")
    missing = [f"CPI-YY-{ref}, not published by 2026-09-30: the interest years from {ref[:4]}-{int(ref[5:]) + 2:02d} of {', '.join(v)}"
               for ref, v in sorted(holes.items())]
    t("value 2026-12-31", {"bonds": sum(left2.values()), "known": v2, "missing": missing}, "no redemption after 2026-09-30")
    key = {"bonds held": n, "value": val, "accrued interest": acc, "if all redeemed": {"fees": fee, "tax": tax, "paid": net},
           "2026-12-31": {"bonds": sum(left2.values()), "value": {"kind": "partial", "value": v2, "missing": missing}}}
    put("D-06", ["OWN-06", "OWN-03", "NUM-01", "NUM-03", "VAL-01", "VAL-02"], "2025-12-31", "2026-09-30", key, t,
        relations=["value − fees − taxes = what a full redemption pays"])
    return {"value": val, "paid": net}


# --------------------------------------------------------------------------- D-07

def q07(sim, rc):
    t = Trace()
    reds = {}
    for red in sim["redemptions"]:
        bonds = {x["series"]: x["bonds"] for x in red["lines"]}
        gross = sum((x["gross"] for x in red["lines"]), ZERO)
        last = red["lines"][-1]
        check(gross >= F.REDEEM_MIN and gross - (last["gross"] / last["bonds"]) < F.REDEEM_MIN, f"D-07 {red['request']}: just enough bonds")
        row = [r for r in rc["rows"] if r["date"] == red["paid"] and r["cp"] == "Treasury ISB"]
        check(len(row) == 1 and row[0]["amount"] == red["net"], f"D-07 {red['request']}: the records' credit is the net")
        reds[red["request"].isoformat()] = {"low day": red["low_day"], "balance": red["low_balance"], "paid on": red["paid"],
                                            "bonds": bonds, "tax": red["tax"], "net": red["net"]}
    for a, b in zip(sim["redemptions"], sim["redemptions"][1:]):
        check(b["request"] > a["paid"] or b["low_day"] >= a["paid"], "D-07 no request while one is in flight")
    t("redemptions", reds, "D-HR2: the day's end below 5 000.00, the next business day's request, the earliest series first, paid on the 3rd business day (TSB-4.1)")
    after = [r for r in rc["rows"] if r["date"] > F.DIVISION]
    days = {}
    for r in after:
        days[r["date"]] = r["balance"]
    low_d = min(days, key=lambda d: (days[d], d))
    t("lowest end of day after 2025-06-12", {"day": low_d, "balance": days[low_d]}, "")
    key = {"redemptions": reds, "bonds left": sum(held(sim, date(2026, 9, 30)).values()),
           "lowest end of day": {"day": low_d, "balance": days[low_d]}}
    s0 = F.sim(False, known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    d0 = {}
    for r in s0["rows"]:
        if r["date"] > F.DIVISION:
            d0[r["date"]] = r["balance"]
    l0 = min(d0, key=lambda d: (d0[d], d))
    without = {"redemptions": {}, "bonds left": 325, "lowest end of day": {"day": l0, "balance": d0[l0]}}
    check(d0[l0] >= F.FLOOR, "D-07 without the death the floor is never reached")
    t("without the twist", without, "no escrow, no widow's costs: the account never goes below the floor")
    put("D-07", ["RULE-02", "RULE-05", "OWN-07", "OWN-09"], "2026-09-30", "2026-09-30", key, t, without=without,
        relations=["just enough bonds each time", "the records' credits are the nets", "no request while one is in flight"])


# --------------------------------------------------------------------------- D-08

def q08(sim, rc, bonds, qs):
    K = "2026-01-07"   # R1: D-08's known-as-of date
    t = Trace()
    on = date(2025, 12, 31)
    acct = [r for r in rc["rows"] if r["date"] <= on][-1]["balance"]
    t("account", acct, "Bank Olcha, the last balance after in December 2025")
    t("bonds", bonds["paid"], "D-HR4: what a redemption requested that day would pay (D-06)")
    flat = F.VALUATION_2025[1] * 13 / 16
    t("Ogrodowa 13/16", flat, "D-HR4: the latest valuation, 240 000.00 (D-D29, 2025-10-06); the agreed 165 000.00 is not one")
    sio = sum((F.part_law(date(2025, m, 1), known=K) - F.part_sio(date(2025, m, 1), known=K) for m in range(7, 13)), ZERO)
    t("owed by the SIO", sio, "D-HR4: the part's gap July–December 2025, gross")
    kuba = -qs["2025-Q4"]["to send"]
    t("owed to Jakub", kuba, "2025-Q4's share, held in her account until 2026-01-12")
    pit13 = -[r for r in sim["rent_adv"] if r["month"] == "2025-12"][0]["advance"]
    t("PIT-13 advance for December", pit13, "due 2026-01-20")
    pen = ret_of(sim["sio"]["H"][2025])
    pit = -(pen["tax"] - pen["advances"])
    t("PIT 2025 on the pensions", pit, "the year's pensions are all paid: tax − advances, due 2026-04-30")
    total = acct + bonds["paid"] + flat + sio + kuba + pit13 + pit
    check(total == sum([acct, bonds["paid"], flat, sio, kuba, pit13, pit], ZERO), "D-08 the parts add up")
    hole = ["the house ul. Leśna 4, Brzezie: her own, never valued"]
    key = {"account": acct, "bonds": bonds["paid"], "Ogrodowa 13/16": flat, "owed by the SIO": sio, "owed to Jakub": kuba,
           "PIT-13 December": pit13, "PIT 2025": pit, "house": {"kind": "refusal", "missing": hole, "reason": "no valuation"},
           "net worth": {"kind": "partial", "value": total, "missing": hole}}
    put("D-08", ["OWN-01", "OWN-02", "OWN-03", "ASK-02", "NUM-05", "VAL-04"], "2025-12-31", "2026-01-07", key, t,
        relations=["the parts add up"])


# --------------------------------------------------------------------------- D-09

def q09():
    t = Trace()
    price = F.OFFER[1]
    fee = ort(price * D("0.02"))
    inh, bought = price * 4 / 13, price * 9 / 13
    t("price", {"inherited 4/16": inh, "bought 9/16": bought}, "13/16 = 4/16 inherited (D-D04) + 9/16 bought in the division (D-D07)")
    t("the inherited 4/16", "not taxed", "PIT-12.2: counts as acquired when Zbigniew's mother did (D-D06), long before 2021")
    costs = fee * 9 / 13
    acq = 3 * D("30937.50") + D("1230.00")
    income = bought - costs - acq
    t("the bought 9/16's income", income, f"{bought} − its part of the fee {costs} − the three buyouts' flat parts and the notary's fee {acq} (PIT-12.1)")
    tax_none = duk(D("0.19") * duk(income))
    t("tax, nothing spent on her housing", tax_none, "PIT-12.3, in the 2026 return; PIT-12.4 can lift it until 2029-12-31")
    net = {"none": price - fee - tax_none, "all": price - fee}
    tax = {"kind": "by-case", "cases": {"nothing spent on her own housing by 2029-12-31": tax_none,
                                        "the whole price spent on it by 2029-12-31": 0},
           "settled by": ["what she spends on her own housing by 2029-12-31 (PIT-12.4, .5)"]}
    rent, fees = F.rent_on(date(2026, 9, 30), known="2026-09-30") * 12, F.FEES[2026] * 12  # R1: D-09's known-as-of date
    keep = ort((rent - fees) * 13 / 16) - duk(rent * 13 / 16 * D("0.085"))
    t("keep: net rent a year", keep, f"13/16 × ({rent} − {fees}) less PIT-13 on 13/16 of {rent}, at today's rent and fee")
    yrs = {k: (v / keep).quantize(D("0.01")) for k, v in net.items()}
    t("years of that rent the proceeds equal", yrs, "")
    key = {"advice": refusal([], "whether to sell is Halina's choice; both sides are shown"),
           "sell": {"price": price, "agent's fee": fee, "inherited 4/16 taxed": False, "income of the 9/16": income,
                    "tax": tax,
                    "net": {"kind": "by-case", "cases": {"nothing spent": net["none"], "all spent": net["all"]}, "settled by": ["as for the tax"]}},
           "keep": {"value": F.VALUATION_2025[1] * 13 / 16, "net rent a year": keep},
           "breakeven years": {"kind": "by-case", "cases": {"nothing spent": yrs["none"], "all spent": yrs["all"]}, "settled by": ["as for the tax"]}}
    put("D-09", ["ASK-04", "LAW-10", "OWN-01", "NUM-05"], "2026-09-30", "2026-09-30", key, t,
        relations=["the parts of the price add up to it"])
    check(inh + bought == price, "D-09 the parts of the price add up to it")


# --------------------------------------------------------------------------- D-10, D-11

def nice(se):
    """A target precision at least 4 times the standard error: 1, 2 or 5 times a power of ten."""
    x = D(se) * 4
    e = D(1).scaleb(x.adjusted())
    for f in (1, 2, 5, 10):
        if e * f >= x:
            v = e * f
            return D(int(v)) if v >= 1 else v


def rnd(xs, places):
    m, se = mean_se([float(x) for x in xs])
    return {"kind": "random", "r": dec(m, places), "se_r": dec(se, places), "p": nice(dec(se, places))}


def quant(xs, p):
    s = sorted(xs)
    return s[max(0, math.ceil(p * len(s)) - 1)]


def band(paths, year, p):
    alive = [(i, x["ends"][year]) for i, x in enumerate(paths) if year in x["ends"]]
    r = quant([v for _, v in alive], p)
    bs = [quant([v for i, v in alive if i % 20 == b], p) for b in range(20)]
    m = sum(float(b) for b in bs) / 20
    se = math.sqrt(sum((float(b) - m) ** 2 for b in bs) / 19 / 20)
    return {"kind": "random", "r": r, "se_r": dec(se, 2), "p": nice(dec(se, 2))}


def frac(x, places):
    return (Decimal(x.numerator) / Decimal(x.denominator)).quantize(Decimal(1).scaleb(-places))


def q10():
    t = Trace()
    fl = LP.flows(LP.cpi_path(), known="2026-09-30")   # R1: D-10's known-as-of date
    cf = LP.closed(fl)
    paths = [LP.path(i, fl) for i in range(N_PATHS)]
    zero = [LP.path(i, fl, zero=True) for i in range(N_PATHS)]
    check([p["k"] for p in paths] == [p["k"] for p in zero], "D-10 the same lifetimes with and without volatility")
    ek = frac(cf["e_k"], 4)
    t("expected whole years", ek, "Σ_j Π_{i≤j} (1 − q_i), ages 68..105 (D-HR3's table)")
    yrs = rnd([p["k"] for p in paths], 4)
    check(abs(yrs["r"] - ek) <= 4 * yrs["se_r"], "D-10 the paths' lifetimes agree with the closed form")
    etax = frac(cf["e_tax"], 2)
    tx = rnd([p["tax"] for p in paths], 2)
    check(abs(tx["r"] - etax) <= 4 * tx["se_r"], "D-10 the paths' tax to death agrees with the closed form")
    t("tax to death", {"closed form": etax, "paths": tx}, "the pensions' PIT each year she lives, as the 2026 law has it")
    sz = frac(cf["share_zero"], 6)
    zs = rnd([1 if p["dry"] else 0 for p in zero], 6)
    check(abs(zs["r"] - sz) <= 4 * zs["se_r"], "D-10 with zero volatility the share running dry is the chance of living through the dry year")
    t("zero volatility", {"dry in": cf["dry_zero"], "share": sz, "paths": zs}, "every return 5 %: the money runs out in that year; the share is P(alive through it)")
    sh = rnd([1 if p["dry"] else 0 for p in paths], 6)
    t("share running dry", sh, f"{N_PATHS} paths, seed {F.SEED} + i")
    bands = {str(y): {f"p{int(q * 100)}": band(paths, y, q) for q in (D("0.1"), D("0.5"), D("0.9"))} for y in (2035, 2045)}
    t("bands", bands, "net worth (account + fund) at the year's end, paths she is alive at; batch standard errors")
    key = {"expected whole years": ek, "tax to death closed form": etax,
           "zero volatility": {"dry in": cf["dry_zero"], "share running dry": sz},
           "whole years": yrs, "tax to death": tx, "share running dry": sh, "share running dry, zero volatility": zs,
           "bands": bands}
    put("D-10", ["LIFE-01", "RAND-01", "RAND-02", "RAND-03", "RAND-04", "SURF-03", "NUM-06"], "2064-12-31", "2026-09-30", key, t,
        relations=["lifetimes agree with the closed form", "tax to death agrees with the closed form",
                   "zero volatility: the share is the chance of living through the dry year"])
    return fl, cf


def q11(fl, cf):
    t = Trace()
    over = {2027: D("1.08"), 2028: D("1.08"), 2029: D("1.08")}
    fo = LP.flows(LP.cpi_path(over), known="2026-09-30")   # R1: D-11's known-as-of date
    cfo = LP.closed(fo)
    by = {f["year"]: f for f in fl}
    bo = {f["year"]: f for f in fo}
    check(by[2027]["rent_mar"] == bo[2027]["rent_mar"], "D-11 the 2027 rent reads 2026's CPI: unmoved")
    check(by[2032]["rent_mar"] / by[2031]["rent_mar"] - bo[2032]["rent_mar"] / bo[2031]["rent_mar"] < D("0.0001"),
          "D-11 back on the base path from 2030: the same growth after")

    def cpi_base(ref, ys):
        try:
            return figure(f"CPI-YY-{ref}", ys, "2026-09-30")[0]
        except LookupError:
            return D("103")

    def cpi_over(ref, ys):
        if "2027-01" <= ref <= "2029-12":
            return D("108")
        return cpi_base(ref, ys)

    s, p = [x for x in F.SERIES if x[0] == "2405"][0]
    bond = {"base": F.bond_value(s, p, date(2029, 12, 31), cpi_base, known="2026-09-30"),   # R1: D-11's known-as-of date
            "8 %": F.bond_value(s, p, date(2029, 12, 31), cpi_over, known="2026-09-30")}
    t("one ISB2405 bond on 2029-12-31", bond, "TSB-2: the CPI-YY two months before each interest year, 103 where not yet published")
    rows = {"rent from 2031-03": ("rent_mar", 2031), "survivor pension from 2031-03": ("surv_mar", 2031),
            "spending 2031": ("spending", 2031), "pension tax 2031": ("tax", 2031)}
    key = {}
    for lab, (f, y) in rows.items():
        key[lab] = {"base": by[y][f], "8 %": bo[y][f]}
        t(lab, key[lab], "")
    key["ISB2405 on 2029-12-31"] = bond
    key["zero volatility dry in"] = {"base": cf["dry_zero"], "8 %": cfo["dry_zero"]}
    t("zero volatility dry in", key["zero volatility dry in"], "")
    key["follows the index"] = ["the rent (D-D25)", "the pensions and the minimum pension (SOC-6, SOC-7)", "the building fee", "her spending",
                                "the bonds' rates (TSB-2)"]
    key["does not follow it"] = ["the tax scale and its reducing amount (PIT-3 v3)", "the fund's returns", "the life table"]
    key["register"] = {"CPI": F.INFL, "fund mean": F.FUND_MU, "fund sd": F.FUND_S, "fund df": F.FUND_DF, "q at 68": F.LIFE_TABLE[68],
                       "q growth": D("1.105"), "spending 2026": F.SPEND_2026, "care 2026": LP.CARE_2026, "care from age": LP.CARE_FROM_AGE,
                       "sweep above": F.SWEEP_ABOVE, "start account": LP.START_ACCOUNT, "start fund": LP.START_FUND, "seed": F.SEED}
    t("register", key["register"], "D-HR3")
    put("D-11", ["SCEN-03", "VAL-06", "VAL-07", "VAL-02", "NUM-06"], "2031-12-31", "2026-09-30", key, t,
        relations=["the 2027 rent unmoved", "the same growth after 2029"])


# --------------------------------------------------------------------------- D-12

def market():
    px = []
    with open(ROOT / "market" / "brz-div-d.csv", encoding="utf-8") as f:
        for r in csv.DictReader(f):
            px.append((date.fromisoformat(r["valuation_date"]), D(r["nav_DUK"])))
    dist = []
    with open(ROOT / "market" / "brz-distributions.csv", encoding="utf-8") as f:
        for r in csv.DictReader(f):
            if r["fund"] == "BRZ-DIV-D":
                dist.append((date.fromisoformat(r["ex_date"]), date.fromisoformat(r["pay_date"]), D(r["per_unit_DUK"])))
    return px, dist


def first_day(px, y, m):
    return [x for x in px if (x[0].year, x[0].month) == (y, m)][0]


def replay(px, dist, y, m):
    """D-12: 100 000.00 into whole units on the month's first valuation day; 900.00 out on the first valuation day
    of each of the next 36 months, cash first, then whole units sold; distributions net of 19 % into cash."""
    d0, nav0 = first_day(px, y, m)
    units = int(D("100000.00") / nav0)
    cash = D("100000.00") - units * nav0
    days = []
    for k in range(1, 37):
        yy, mm = divmod(12 * y + (m - 1) + k, 12)
        days.append(first_day(px, yy, mm + 1))
    end = days[-1][0]
    evs = []
    for ex, pay, per in dist:
        if d0 < ex <= end:
            evs.append(("ex", ex, per))
            if pay <= end:
                evs.append(("pay", pay, per))
    netd, pend = ZERO, {}
    moves = ZERO
    last_nav = nav0
    timeline = sorted([(d, 1, ("w", nav)) for d, nav in days] + [(d, 0, (kind, per)) for kind, d, per in evs])
    for d, _, (kind, v) in timeline:
        if kind == "ex":
            g = units * v
            pend[d.year] = g - duk(g * D("0.19"))
            continue
        if kind == "pay":
            got = pend.pop(d.year)
            cash += got
            netd += got
            continue
        moves += units * (v - last_nav)
        last_nav = v
        if cash < D("900.00"):
            n = int(((D("900.00") - cash) / v).to_integral_value(rounding=ROUND_CEILING))
            units -= n
            cash += n * v
        cash -= D("900.00")
    value = units * last_nav + cash
    return {"start": d0, "end": end, "units": units, "cash": cash, "value": value, "moves": moves, "dist": netd}


def q12():
    t = Trace()
    px, dist = market()
    starts = {}
    for y in range(2018, 2024):
        for m in range(1, 13):
            if (y, m) > (2023, 9):
                break
            r = replay(px, dist, y, m)
            starts[f"{y}-{m:02d}"] = r
    tab = {k: {"units left": r["units"], "end value": r["value"]} for k, r in starts.items()}
    for k, r in starts.items():
        check(r["value"] == D("100000.00") - 36 * D("900.00") + r["moves"] + r["dist"], f"D-12 {k}: the end value is explained by the moves and the distributions")
    worst = min(starts, key=lambda k: starts[k]["value"])
    best = max(starts, key=lambda k: starts[k]["value"])
    t("starts", len(tab), "2018-01 .. 2023-09, each run to its 36th withdrawal")
    t("worst", {"start": worst, "end value": starts[worst]["value"]}, "")
    t("best", {"start": best, "end value": starts[best]["value"]}, "")
    key = {"starts": tab, "worst": {"start": worst, "end value": starts[worst]["value"]},
           "best": {"start": best, "end value": starts[best]["value"]}}
    put("D-12", ["REC-07", "NUM-01"], "2026-09-30", "2026-09-30", key, t,
        relations=["end value = 100 000 − the withdrawals + the market's moves + the net distributions"])


# --------------------------------------------------------------------------- D-13: had the fund been taxed (a scenario on D-10's paths)

GAINS_RATE = D("0.19")                 # PIT-8 .4
FUND_BOUGHT = date(2026, 12, 31)       # D-13: the fund's 45 000.00 bought that day, as 45 000 units for 45 000.00
FUND_UNITS = Fraction(45000)


def d10_draws(i):
    """D-10's path i, drawn exactly as fam/d_paths.path draws it: seed F.SEED + i, her whole years, then 38 returns."""
    import random
    from lib.paths import draw_curtate
    r = random.Random(F.SEED + i)
    k = draw_curtate(r, LP.QS)
    return k, [LP.ret(r) for _ in range(LP.YEARS)]


def whole_duk(x):
    """GEN-5.1 on an exact gain (x ≥ 0): whole dukats, under 50 orts down, 50 and over up."""
    return D(math.floor(Fraction(x) + Fraction(1, 2)))


def lot_run(fl, k, rets, rate=GAINS_RATE):
    """One path of the long view (fam/d_paths.run), with the fund held in lots (D-13). Units are carried exactly; at each
    year's end, after the return, a unit is worth the fund's value over the units held. Money put in buys a lot on
    31 December at that worth; units sold to bring the account back to 0 go first in, first out (PIT-8 .3), each part of a
    lot costing its share of the lot's cost (PIT-8 .2); a gain on units held more than 5 years is exempt (PIT-8 .6, GEN-3 .1).
    The year's gains and losses are added; a loss is carried 5 years, at most half of it used in a year and only against a
    net gain, oldest first (PIT-8 .5); base and tax in whole dukats (PIT-8 .4), paid with the next year's money (by 30 April;
    a year she doesn't live counts none of its money). rate 0 is the long view as told (point 7: not taxed).
    Returns what D-10's run returns, and each year's sales and tax."""
    a, f = LP.START_ACCOUNT, LP.START_FUND
    lots = [{"bought": FUND_BOUGHT, "units": FUND_UNITS, "cost": Fraction(LP.START_FUND)}]
    units = FUND_UNITS
    dry, ends, owed, total, losses, years, refills = None, {}, ZERO, ZERO, [], [], 0
    for j in range(min(k, LP.YEARS)):
        y = fl[j]
        yr, day = y["year"], date(y["year"], 12, 31)
        f = ort(f * (1 + Decimal(f"{rets[j]:.6f}")))
        a = a + y["flow"] - owed
        owed = ZERO
        lines = []
        if a > F.SWEEP_ABOVE:
            x = a - F.SWEEP_ABOVE
            if units == 0:
                refills += 1   # never on D-10's paths (checked): the unit's worth would need a rule
                worth = Fraction(1)
            else:
                worth = Fraction(f) / units
            u = Fraction(x) / worth
            lots.append({"bought": day, "units": u, "cost": Fraction(x)})
            units += u
            f, a = f + x, F.SWEEP_ABOVE
        elif a < 0:
            sold = min(f, -a)
            if sold > 0:
                worth = Fraction(f) / units
                need = Fraction(sold) / worth
                for L in lots:
                    if need == 0:
                        break
                    if L["units"] == 0:
                        continue
                    take = min(need, L["units"])
                    cost = L["cost"] * take / L["units"]
                    gain = take * worth - cost
                    lines.append({"bought": L["bought"].isoformat(), "units": take, "cost": cost, "proceeds": take * worth, "gain": gain,
                                  "exempt": gain > 0 and day > F_add_years(L["bought"], 5)})
                    L["units"] -= take
                    L["cost"] -= cost
                    need -= take
                units -= Fraction(sold) / worth
                check(sum((l["proceeds"] for l in lines), Fraction(0)) == Fraction(sold), "D-13 a sale's lots bring exactly what is sold")
            f, a = f - sold, a + sold
            if a < 0 and dry is None:
                dry = yr
        tax = ZERO
        if lines:
            net = sum((l["gain"] for l in lines if not l["exempt"]), Fraction(0))
            used = Fraction(0)
            if net > 0:
                room = net
                for L in losses:
                    if room <= 0:
                        break
                    if yr - L["year"] > 5 or L["left"] <= 0:
                        continue
                    u = min(L["left"], L["amount"] / 2, room)
                    L["left"] -= u
                    used += u
                    room -= u
                base = whole_duk(net - used)
                tax = whole_duk(F_(base) * F_(rate))
            elif net < 0:
                losses.append({"year": yr, "amount": -net, "left": -net})
            years.append({"year": yr, "lines": lines, "net": net, "loss used": used, "tax": tax,
                          "realised": sum((l["gain"] for l in lines), Fraction(0))})
            owed = tax
            total += tax
        ends[yr] = a + f
    return {"k": k, "dry": dry, "ends": ends, "gains tax": total, "years": years, "sold": bool(years), "refills": refills,
            "realised": sum((y["realised"] for y in years), Fraction(0)), "taxed part": any(not l["exempt"] for y in years for l in y["lines"]),
            "carried": any(y["loss used"] > 0 for y in years), "exempt": any(l["exempt"] for y in years for l in y["lines"])}


def F_add_years(d, n):
    return d.replace(year=d.year + n)


def F_(x):
    return Fraction(x)


def show(x):
    """An exact amount shown in a derivation, to the ort (carried exactly)."""
    return ort(D(Fraction(x)))


def q13():
    t = Trace()
    fl = LP.flows(LP.cpi_path(), known="2026-09-30")   # R1: D-13's known-as-of date, as D-10's
    s_ = LP.survival()
    runs, zeros = [], []
    for i in range(N_PATHS):
        k, rets = d10_draws(i)
        p10 = LP.path(i, fl)
        check(k == p10["k"], f"D-13 path {i}: D-10's lifetime")
        r = lot_run(fl, k, rets)
        u = lot_run(fl, k, rets, rate=ZERO)
        check(u["dry"] == p10["dry"] and u["ends"] == p10["ends"], f"D-13 path {i}: untaxed, the run is D-10's")
        runs.append(r)
        zeros.append(lot_run(fl, k, [float(F.FUND_MU)] * LP.YEARS))
    check(all(r["refills"] == 0 for r in runs + zeros), "D-13 the fund never fills again once emptied on these paths")
    full0 = lot_run(fl, LP.YEARS, [float(F.FUND_MU)] * LP.YEARS)
    by_year = {y["year"]: y for y in full0["years"]}
    e_tax = sum((s_[j] * F_(by_year[LP.START_YEAR + j]["tax"]) for j in range(LP.YEARS) if LP.START_YEAR + j in by_year), Fraction(0))
    e_real = sum((s_[j] * by_year[LP.START_YEAR + j]["realised"] for j in range(LP.YEARS) if LP.START_YEAR + j in by_year), Fraction(0))
    first0 = full0["years"][0]["year"] if full0["years"] else None
    t("paths", N_PATHS, f"D-10's: seed {F.SEED} + i, her whole years then the 38 returns; the same lifetimes as D-10's on every path")
    t("untaxed, the lots change nothing", True, "every path run with rate 0 gives D-10's dry year and net worths")
    ex = next(r for r in runs if r["sold"])
    y0 = ex["years"][0]
    t(f"the first path with a sale: path {runs.index(ex)}, its first sale, {y0['year']}-12-31",
      [{"lot bought": l["bought"], "proceeds": show(l["proceeds"]), "cost": show(l["cost"]), "gain": show(l["gain"]), "exempt": l["exempt"]} for l in y0["lines"]],
      "units sold first in, first out (PIT-8 .3), each part at its share of the lot's cost (PIT-8 .2); a gain on units held more than "
      "5 years exempt (PIT-8 .6, GEN-3 .1); shown to the ort, carried exactly")
    t(f"its tax for {y0['year']} (PIT-8 .4)", y0["tax"], f"the year's taxed gains and losses {show(y0['net'])}, less losses used {show(y0['loss used'])}; base and tax in whole dukats, paid with {y0['year'] + 1}'s money")
    zl = [{"year": y["year"], "gains": show(y["realised"]), "exempt parts": sum(1 for l in y["lines"] if l["exempt"]), "tax": y["tax"]} for y in full0["years"]]
    t("zero volatility, living to 105: each year units are sold", zl, "every return 5 %: the oldest lots go first, each held more than 5 years")
    nt, nc, nl = sum(1 for r in runs if r["gains tax"] > 0), sum(1 for r in runs if r["carried"]), sum(1 for r in runs if r["taxed part"])
    ng = sum(1 for r in runs if any(not l["exempt"] and l["gain"] > 0 for y in r["years"] for l in y["lines"]))
    fy = min(r["years"][0]["year"] for r in runs if r["sold"])
    t("why the tax is 0", {"first year any path sells": fy, "paths with a tax": nt, "paths with a loss (never exempt: PIT-8 .6 exempts gains)": nl,
                           "paths with a gain on units held 5 years or less": ng, "paths where a carried loss covers a year's taxed gain (PIT-8 .5)": nc},
      f"of {N_PATHS}: the oldest lots go first (PIT-8 .3), and by the first sale they are more than 5 years old; "
      "where a younger lot is reached at a gain, the same year's losses or a loss carried from an earlier year cover it")
    check(0 < nc <= ng, "D-13 a carried loss covers a taxed gain only where a young lot is sold at a gain")
    check(nt == 0 and nc > 0, "D-13 no path pays a gains tax, and on some a carried loss is what makes it so")
    etax, ereal = frac(e_tax, 2), frac(e_real, 2)
    t("zero volatility, closed forms: the expected gains tax and gains to her death", {"tax": etax, "gains": ereal},
      "Σ_j P(she lives through year j) × that year's figure (a path is the same as this one up to her death)")
    zg = rnd([show(r["realised"]) for r in zeros], 2)
    check(abs(zg["r"] - ereal) <= 4 * zg["se_r"], "D-13 with zero volatility the paths' gains agree with the closed form")
    check(all(r["gains tax"] == 0 for r in zeros) and etax == 0, "D-13 with zero volatility no gains tax")
    t("zero volatility on the paths: gains to her death", zg, "the same lifetimes, every return 5 %")
    gt = rnd([r["gains tax"] for r in runs], 2)
    t("gains tax: each year's tax on the units sold while she lives, summed to her death", gt, f"{N_PATHS} paths")
    gr = rnd([show(r["realised"]) for r in runs], 2)
    t("gains: proceeds less cost of every part sold while she lives, exempt or not, summed to her death", gr, f"{N_PATHS} paths; each path's sum to the ort")
    sh = rnd([1 if r["sold"] else 0 for r in runs], 6)
    t("share selling units: paths on which units are sold while she lives", sh, f"{N_PATHS} paths")
    key = {"gains tax": gt, "gains": gr, "share selling units": sh,
           "zero volatility": {"first sale in": first0, "gains closed form": ereal, "gains tax closed form": etax}}
    put("D-13", ["OWN-06", "OWN-07", "RAND-01", "RAND-02", "RAND-03", "RAND-04", "SCEN-01", "VAL-07", "TIME-05", "LAW-09"],
        "2064-12-31", "2026-09-30", key, t,
        relations=["D-10's draws: the same lifetimes, path by path", "untaxed, the run is D-10's, path by path",
                   "zero volatility: the paths' gains agree with the closed form"])

def main():
    sim = F.sim(known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    rc = read_olcha()
    q01(sim, rc)
    q02()
    q03(sim, rc)
    q04(sim, rc)
    qs = q05(sim, rc)
    bonds = q06(sim)
    q07(sim, rc)
    q08(sim, rc, bonds, qs)
    q09()
    fl, cf = q10()
    q11(fl, cf)
    q12()
    q13()
    if FAILS:
        print("relations failed:", *FAILS, sep="\n  ")
        sys.exit(1)
    print("D: 13 keys written, relations pass")


if __name__ == "__main__":
    main()
