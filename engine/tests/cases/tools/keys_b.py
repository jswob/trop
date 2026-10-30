#!/usr/bin/env python3
"""Household B's keys: one file per question in keys/B/, each a YAML block and the derivation.

Reads the home law (rules as code in fam/home.py and fam/b.py), the gazette, the market files, B's
records and the facts B's documents state (fam/b.py, each citing its document). Asserts every
relation on its own output; exits 1 if one fails.

Run: python3 -I tmp/cases/tools/keys_b.py [n] [--end YYYY-MM-DD]
     (n: forecast paths, default 3000; --end: B-12 alone with the forecast ending on that day instead of
      2028-12-31, written to keys-small/B/B-12.md and never to keys/, e.g. `keys_b.py 30 --end 2027-01-31`)
"""
import csv
import json
import math
import pathlib
import re
import sys
from datetime import date, timedelta
from decimal import Decimal, ROUND_CEILING, ROUND_HALF_UP

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from fam import b, b_paths  # noqa: E402
from fam.home import ALL_KNOWN, CAL, ZERO, D, duk, ort, run_payer, separate_return  # noqa: E402
from lib.dates import end_of_month  # noqa: E402
from lib.keys import Trace, enc  # noqa: E402
from lib.paths import mean_se, dec  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[1]
REC = ROOT / "B" / "records"
FAILS = []
_ARGS = sys.argv[1:]
END_AT = date.fromisoformat(_ARGS[_ARGS.index("--end") + 1]) if "--end" in _ARGS else None   # the run-end knob (B-12 only)
_ARGS = [a for i, a in enumerate(_ARGS) if a != "--end" and (i == 0 or _ARGS[i - 1] != "--end")]
N_PATHS = int(_ARGS[0]) if _ARGS else 3000
SMALL_ROOT = ROOT / "keys-small"   # where the knob's keys go; keys/ is never written with --end


def check(cond, what):
    if not cond:
        FAILS.append(what)


def dirn(x):
    x = D(x)
    return {"refund": -x} if x < 0 else {"payable": x}


def put(qid, edges, as_of, known, key, t, without=None, relations=(), small=None):
    """small: (paths, end) for a key made with the run-end knob, written under keys-small/ with both in its header."""
    head = [f"question: {enc(qid)}", f"edges: {enc(edges)}", f"as_of: {enc(as_of)}", f"known_as_of: {enc(known)}", f"key: {enc(key)}"]
    if without is not None:
        head.append(f"without: {enc(without)}")
    if relations:
        head.append(f"relations: {enc(list(relations))}")
    if small:
        head += [f"paths: {small[0]}", f"end: {enc(small[1].isoformat())}"]
        title = (f"# Key · {qid}, {small[0]} paths, the forecast ending {small[1].isoformat()}\n\n"
                 f"Written by `tools/keys_b.py {small[0]} --end {small[1].isoformat()}`; don't edit. The labels are {qid}'s: "
                 f"each figure {qid} takes on 2028-12-31 is taken on {small[1].isoformat()}, and each sum or share runs from "
                 f"2026-01 to {small[1].strftime('%Y-%m')}. Mark answers with `compare.py --small --keys-root tmp/cases/keys-small`.\n\n")
    else:
        title = f"# Key · {qid}\n\nWritten by `tools/keys_b.py`; don't edit.\n\n"
    body = (title + "```yaml\n" + "\n".join(head) + "\n```\n\n## Derivation\n\n" + t.render() + "\n")
    p = (SMALL_ROOT if small else ROOT / "keys") / "B" / f"{qid}.md"
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(body, encoding="utf-8")


# --------------------------------------------------------------------------- records

def iso(s):
    return date.fromisoformat(s)


def dmy(s, sep):
    d, m, y = s.split(sep)
    return date(int(y), int(m), int(d))


def dc(s):
    # R2: Wrzos's exports fill every amount and balance read (records/wrz-ola-*.csv); an empty cell never stands for a missing figure
    return D(s.replace(",", ".")) if s else ZERO


def cis_file(name):
    lines = (REC / name).read_text(encoding="utf-8").splitlines()
    head = {ln[2:].split(": ", 1)[0]: ln[2:].split(": ", 1)[1] for ln in lines if ln.startswith("# ") and ": " in ln}
    body = [ln for ln in lines if not ln.startswith("#")]
    rows = []
    for i, r in enumerate(csv.reader(body[1:]), 1):
        bd, vd, typ, cp, title, deb, cre, bal = r
        # R2: the Cis export fills one of Debit and Credit on each row and leaves the other blank
        rows.append({"booked": dmy(bd, "-"), "value": dmy(vd, "-"), "type": typ, "cp": cp, "title": title,
                     "amount": (D(cre) if cre else ZERO) - (D(deb) if deb else ZERO), "balance": D(bal), "raw": tuple(r),
                     "src": f"{name} row {i}"})
    p0, p1 = head["Period"].split(" - ")
    return {"opening": D(head["Opening balance"]), "start": dmy(p0, "-"), "end": dmy(p1, "-"), "rows": rows, "name": name}


def cis_months():
    return [cis_file(f"cis-5521-3380-0001-2025-{m:02d}.csv") for m in range(1, 13)]


def lazur_rows():
    out = []
    for i, ln in enumerate((REC / "laz-8813-flr-2025.csv").read_text(encoding="utf-8").splitlines()[1:], 2):
        d, desc, amt, bal = ln.split(",")
        out.append({"date": iso(d), "desc": desc, "amount": D(amt), "balance": D(bal), "src": f"laz-8813-flr-2025.csv line {i}"})
    return out


def register():
    out = []
    for ln in (REC / "faktura-sales-2022-2025.csv").read_text(encoding="utf-8").splitlines()[2:]:
        f = ln.split(";")
        out.append({"id": f[0], "issued": iso(f[1]), "due": iso(f[2]), "client": f[3], "supply": f[4], "cur": f[5], "net": D(f[6]),
                    "rate": f[7], "vat": D(f[8]), "gross": D(f[9]), "vat_duk": D(f[10]) if f[10] else None,
                    "mid": D(f[11]) if f[11] else None, "table": f[12]})
    return out


def purchases_reg():
    out = []
    for ln in (REC / "faktura-purchases-2022-2025.csv").read_text(encoding="utf-8").splitlines()[2:]:
        f = ln.split(";")
        out.append({"id": f[0], "received": iso(f[1]), "supplier": f[2], "what": f[3], "net": D(f[4]), "vat": D(f[5]), "gross": D(f[6])})
    return out


def wrzos(y, m):
    lines = (REC / f"wrz-ola-{y}-{m:02d}.csv").read_text(encoding="utf-8").splitlines()
    opening = dc(lines[4].split(";")[1])
    rows = []
    for ln in lines[6:]:
        d, typ, cp, title, amt, bal = ln.split(";")
        rows.append({"date": dmy(d, "."), "type": typ, "cp": cp, "title": title, "amount": dc(amt), "balance": dc(bal)})
    return opening, rows


def revenue_duk(s):
    return ort(s["net"] * s["mid"]) if s["cur"] == "FLR" else s["net"]   # PIT-15 .2-.3, GEN-4


# --------------------------------------------------------------------------- the Cis exports read (B-08 .. B-11)

def read_cis():
    """Each export chained: duplicates, a dropped row, a short export, reversals; the true month ends."""
    months = cis_months()
    rows, problems, gaps = [], [], []
    for k, mf in enumerate(months):
        bal, prev = mf["opening"], None
        for r in mf["rows"]:
            if prev is not None and r["raw"] == prev["raw"]:
                problems.append({"day": r["booked"].isoformat(), "amount": r["amount"],
                                 "what": f"the same row twice ({r['cp']}, {r['title']}); the balance after counts it once"})
                continue
            if bal + r["amount"] != r["balance"]:
                miss = r["balance"] - bal - r["amount"]
                problems.append({"day": None, "amount": miss, "what": "a row missing: the balance after jumps", "_before": r})
            if r["type"] == "REVERSAL":
                problems.append({"day": r["booked"].isoformat(), "amount": r["amount"],
                                 "what": f"a real reversal: {r['title']}; the invoice is open again"})
            rows.append(r)
            bal, prev = r["balance"], r
        nxt = months[k + 1]["opening"] if k + 1 < len(months) else None
        last_day = mf["rows"][-1]["booked"]
        if nxt is not None and nxt != bal:
            day = last_day + timedelta(days=1)
            gaps.append((day, mf["end"]))
            problems.append({"day": day.isoformat(), "amount": nxt - bal,
                             "what": f"the export stops at {last_day}: the rows from {day} to {mf['end']} are missing; this is their net"})
        mf["close"] = nxt if nxt is not None and nxt != bal else bal
    for p in problems:  # a missing row's day: Bank Cis's card fee (B-D36), 9 December or the next business day
        if p["day"] is None:
            fee_day = CAL.on_or_next(date(2025, 12, 9))
            check(p["amount"] == -D("249.00") and p["_before"]["booked"] >= fee_day, "B-08 the dropped row is B-D36's card fee")
            p["day"] = fee_day.isoformat()
            p["what"] = "Bank Cis's yearly card fee (B-D36) is not among the rows; the balance after shows it"
            del p["_before"]
    problems.sort(key=lambda p: p["day"])
    return {"months": months, "rows": rows, "problems": problems, "gaps": gaps}


def in_gap(d, gaps):
    return any(a <= d <= z for a, z in gaps)


# --------------------------------------------------------------------------- B-01

def q01():
    t = Trace()
    bz = b.business(True, known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    reg = [s for s in register() if s["issued"].year == 2024]
    months = {}
    for m in range(1, 13):
        row = bz["months"][(2024, m)]
        rev = sum((revenue_duk(s) for s in reg if s["issued"].month == m), ZERO)
        check(rev == row["revenue"], f"B-01 the register's revenue is the facts' ({m})")
        months[f"2024-{m:02d}"] = {"revenue": rev, "revenue to date": row["rev_ytd"], "social": row["social"]["total"],
                                   "health": row["health"], "tax": row["tax"], "paid on": b.due20(2024, m).isoformat()}
        t(f"2024-{m:02d}", months[f"2024-{m:02d}"],
          f"stage {row['social']['stage']}, base {row['social']['base']} (SOC-3); tier {row['tier']} × AVG-WAGE-Q4 {row['awq4']} × 9 % (SOC-4 .3); "
          f"tax: (revenue − contributions paid {row['social_paid']} − carried {row['carry_in']}) × 12 %, base and tax rounded (PIT-10 .4); paid by the 20th (SOC-3 .6, PIT-10 .5, GEN-2)")
    st = bz["settle"][2024]
    settle = {"revenue": st["revenue"], "owed": st["owed"], "paid": st["paid"], "difference": dirn(st["difference"]), "due": st["due"].isoformat()}
    t("settlement 2024", settle, f"each month: tier of the whole year's revenue ({st['tier']}) × that month's AVG-WAGE-Q4 × 9 %, summed, less paid (SOC-4 .5)")
    check(st["revenue"] > D("300000.00") and all(bz["months"][(2024, m)]["tier"] <= D("1.00") for m in range(1, 12)) and bz["months"][(2024, 12)]["tier"] == D("1.80"),
          "B-01 the cliff: paid at 60 % or 100 % until December, owed at 180 % for every month")
    # form (b): PIT-10 .3, PIT-11 .1-.2
    costs = ZERO
    for p in [p for p in purchases_reg() if p["received"].year == 2024]:
        car = p["supplier"] == "Stacja Paliw Tor sp."
        not_deducted = p["vat"] / 2 if car else ZERO           # VAT-5 .2
        c = (p["net"] + not_deducted) * (D("0.75") if car else 1)  # PIT-11 .1
        costs += c
    dep = ort(D("88800.00") * D("0.20") / 12) * 12            # PIT-11 .2: 80 000 + 8 800 not deducted (B-D11)
    t("costs under (b)", {"invoices": costs, "depreciation": dep}, "fuel at (net + half the VAT) × 75 % (PIT-11 .1, VAT-5 .2); the monitor at net; depreciation 88 800.00 × 20 % / 12 a month")
    costs += dep
    soc_paid = sum((bz["months"][k]["social"]["total"] for k in [(2023, 12)] + [(2024, m) for m in range(1, 12)]), ZERO)
    profit = st["revenue"] - costs - soc_paid
    tax_b = duk(profit * D("0.19"))
    check(tax_b == duk(duk(profit) * D("0.19")), "B-01 the (b) tax doesn't hang on rounding the profit first")
    tax_c = sum((bz["months"][(2024, m)]["tax"] for m in range(1, 13)), ZERO)
    form_b = {"contributions deducted": soc_paid, "tax": tax_b}
    t("form (b)", dict(form_b, costs=costs, profit=profit), "revenue − costs − contributions paid in 2024 (December 2023 .. November 2024's); × 19 %, rounded (PIT-10 .3, .5)")
    t("form (c)", tax_c, "the months' taxes")
    key = {"months": months, "settlement": settle, "form (b)": form_b, "form (c)": tax_c}
    put("B-01", ["LIFE-05", "LAW-01", "LAW-02", "LAW-06", "LAW-08", "LAW-11", "TIME-02", "TIME-03", "TIME-04", "TIME-06", "VAL-02", "ASK-01"],
        "2025-04-30", "2025-04-30", key, t, relations=["the settlement is the year's tier over every month less what was paid"])


# --------------------------------------------------------------------------- B-02

def q02():
    t = Trace()
    bz = b.business(True, known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    cross = [s for s in bz["sales"] if "crossing" in s][0]
    days = cross["crossing"]["days"]
    t("limit", bz["limit"], f"200 000.00 × {days} days (2022-04-11 .. 12-31) / 365, rounded (VAT-2 .2)")
    t("crossing", (cross["id"], cross["crossing"]["cumulative"]), "the first invoice with which 2022's supplies go above the limit; VAT from it on (VAT-2 .1)")
    rets = {}
    for m in (10, 11, 12):
        r = bz["months"][(2022, m)]
        rets[f"2022-{m}"] = {"output": r["vat_out"], "input": r["vat_in"], "carried in": r["vat_carry_in"], "payable": r["vat_due"],
                             "carried out": r["vat_carry_out"], "due": r["due_vat"].isoformat()}
        t(f"return 2022-{m}", rets[f"2022-{m}"], "output − input − carried; positive paid by the 25th, else carried (VAT-4)")
    key = {"limit": bz["limit"], "invoice": cross["id"], "supplies with it": cross["crossing"]["cumulative"], "returns": rets}
    put("B-02", ["LAW-07", "TIME-02", "NUM-01"], "2023-02-28", "2023-02-28", key, t)


# --------------------------------------------------------------------------- B-03

def q03():
    known = "2026-01-07"   # R1: B-03's known-as-of date
    t = Trace()
    reg = [s for s in register() if s["client"] == "Kessel Lenia sp."]
    laz = lazur_rows()
    cis = read_cis()["rows"]
    inv = {}
    for s in reg:
        mid, src = b.mid_for_event(s["issued"], known=known)
        check(mid == s["mid"], f"B-03 the register's rate is GEN-4's ({s['id']})")
        line = {"revenue": ort(s["net"] * mid), "VAT": ort(s["vat"] * mid)}
        check(line["VAT"] == s["vat_duk"], f"B-03 the register's VAT in DUK ({s['id']})")
        paid = [r for r in laz if s["id"] in r["desc"]]
        if paid:
            pd = paid[0]["date"]
            sold = [r for r in laz if r["date"] == pd and r["desc"].startswith("FX sale")]
            if sold:
                rate = b.lazur_buy(pd, known=known)
                got = ort(s["gross"] * rate)     # B-D23
                cred = CAL.on_or_next(pd + timedelta(days=1))
                rows = [r for r in cis if r["booked"] == cred and r["cp"] == "PIOTR SOWA (BANK LAZUR)"]
                check(rows and rows[0]["amount"] == got, f"B-03 the dukats reached Cis ({s['id']})")
                line.update({"dukats": got, "credited": cred.isoformat()})
                t(s["id"], line, f"revenue and VAT at CBL mid {mid} of {b.CAL.before(s['issued'])}, the business day before (GEN-4 .1, .2; PIT-15 .3, VAT-3 .3); "
                                 f"{s['gross']} FLR sold at Lazur's buy {rate} on {pd}, to the ort (B-D23); "
                                 f"the money differs from the gross at the law's rate by {got - line['revenue'] - line['VAT']}")
            else:
                line["florens"] = "kept in the Lazur account"
                t(s["id"], line, f"at CBL mid {mid}; paid {pd} into Lazur, kept")
        else:
            line["florens"] = "owed by Kessel"
            t(s["id"], line, f"at CBL mid {mid}; unpaid at the year's end")
        inv[s["id"]] = line
    flr = laz[-1]["balance"]
    m31, _ = b.mid_on(date(2025, 12, 31), known=known)
    lz = {"FLR": flr, "DUK": ort(flr * m31)}
    t("Lazur at 2025-12-31", lz, f"FLR × CBL mid {m31} (B-HR4 .5)")
    nov = [s for s in reg if s["issued"].month == 11][0]
    nv = {}
    for d in (date(2025, 11, 30), date(2025, 12, 31)):
        r, src = b.mid_on(d, known=known)
        nv[d.isoformat()] = {"FLR owed": nov["gross"], "DUK": ort(nov["gross"] * r)}
        t(f"November invoice at {d}", nv[d.isoformat()], f"{nov['gross']} FLR × {r} ({src})")
    check(nv["2025-11-30"]["FLR owed"] == nv["2025-12-31"]["FLR owed"], "B-03 a revaluation never moves the floren amount (CUR-03)")
    key = {"invoices": inv, "Lazur at 2025-12-31": lz, "November invoice": nv}
    put("B-03", ["CUR-01", "CUR-02", "CUR-03", "NUM-01"], "2025-12-31", "2026-01-07", key, t,
        relations=["a revaluation never moves the floren amount"])


# --------------------------------------------------------------------------- B-04

def q04():
    t = Trace()
    fp = b.flat_purchase()
    deed = {w: {"price": fp[w]["price paid"], "transfer tax": fp[w]["transfer tax"], "notary": fp[w]["notary"]} for w in ("Piotr", "Ola")}
    t("at the deed", deed, "TT-1: 2 % of 600 000.00 × the share, rounded (GEN-5.1); the fee by share (B-D04). TT-2 governs purchases from 2023 only")
    spent = fp["Ola"]["at the deed"]
    now = b.studio_tax(spent)
    later = b.studio_tax(spent + b.KITCHEN_QUOTE[1] * D("0.40"))
    t("studio income", now["income"], "312 000.00 − 7 800.00 − (250 000.00 + 5 000.00 + 2 100.00) (PIT-12 .1); taxed: sold before the end of 2024 (PIT-12 .2)")
    t("in the 2021 return", now, "spent by then: 72 000.00 + 4 800 + 1 968.00; exempt = income × spent / 312 000.00; base and tax rounded (PIT-12 .3-.4)")
    t("if the kitchen is ordered", later, "spent + 40 % of 45 000.00 by 2024-12-31; the return corrected (PIT-12 .5)")
    total, lines = b.rent_overlap()
    t("rent for owned days", total, " + ".join(f"{x['rent']} ({x['days']}/{x['of']} of {x['month']})" for x in lines))
    dep = {"back": b.RENT["returned"][1], "on": b.RENT["returned"][0].isoformat(), "kept": b.RENT["kept"]}
    t("deposit", dep, "B-D32")
    key = {"at the deed": deed,
           "Ola's studio": {"income": now["income"], "in the 2021 return": now["tax"],
                            "in the end": {"kind": "by-case", "cases": {"nothing more spent on her housing by 2024-12-31": now["tax"],
                                                                        "her 40 % of the kitchen paid by 2024-12-31": later["tax"]},
                                           "settled by": "what Ola spends on her housing by 2024-12-31 (PIT-12 .4: 3 years after 2021)"}},
           "rent for owned days": total, "deposit": dep}
    put("B-04", ["LAW-10", "LIFE-02", "OWN-01", "OWN-04", "NUM-02", "PTY-01"], "2022-07-31", "2022-07-31", key, t)


# --------------------------------------------------------------------------- B-05

def mortgage(holidays, overpays, *, known):
    m = b.Mortgage(holidays=holidays, known=known)
    for d, amt in overpays:
        m.run_to(d)
        m.overpay(d, amt)
    m.run_to(date(2025, 12, 31))
    bal = m.bal
    m.run_to(date(2026, 1, 5))
    jan = m.rows[-1]
    paid = [s for s in m.slots if s not in m.holidays]
    interest = {str(y): sum((r["interest"] for r in m.rows if not r["holiday"] and r["slot"].year == y), ZERO) for y in range(2022, 2026)}
    return {"interest": interest, "balance 2025-12-31": bal, "instalment 2026-01-05": jan["instalment"], "last instalment": paid[-1].isoformat()}, m


def received(doc):
    """A document's received date, from its header."""
    text = (ROOT / "B" / "docs" / f"{doc}.md").read_text(encoding="utf-8")
    return iso(re.search(r"^- received: (\d{4}-\d\d-\d\d)$", text, re.M).group(1))


def q05_listed(known, t, sim, got):
    """ERR-40: B-D28 is received on 2026-01-02, by B-05's known-as-of date, so the overpayments are its list's (the key
    of cases 1.0.2, before ERR-38)."""
    ovs = [(o["date"], o["paid"]) for o in sim["mtg"].overpays]
    jod = (ROOT / "B" / "docs" / "B-D28.md").read_text(encoding="utf-8")
    check(all(f"| {o['date']:%d.%m.%Y} | overpayment |" in jod for o in sim["mtg"].overpays), "B-05 the overpayments are Bank Jodła's list's")
    t("overpayments (B-D28)", ovs, f"B-D28 is received {got}, by the known-as-of date {known}")
    was, m1 = mortgage(b.HOLIDAYS, ovs, known=known)
    check(was["balance 2025-12-31"] == sim["mtg"].bal, "B-05 the replay equals the simulation")
    t("as it was", was, "B-D07 on market/dukor.csv: resets 1 Jan and 1 Jul; interest by days / 365; holidays carry none (MTG-1 .3); each overpayment's fee B-D07's 2 % until 36 months from disbursement, none after (MTG-2 .2)")
    wo, m2 = mortgage((), ovs, known=known)
    t("without the holidays", wo, "the same, every instalment from 2022-04-05 on, the same overpayments")
    diff = {"interest": {y: wo["interest"][y] - was["interest"][y] for y in was["interest"]},
            "balance 2025-12-31": wo["balance 2025-12-31"] - was["balance 2025-12-31"],
            "instalment 2026-01-05": wo["instalment 2026-01-05"] - was["instalment 2026-01-05"],
            "last instalment": (iso(wo["last instalment"]).year - iso(was["last instalment"]).year) * 12 + iso(wo["last instalment"]).month - iso(was["last instalment"]).month}
    t("difference", diff, "without less as it was; months")
    check(diff["last instalment"] == -len(b.HOLIDAYS), "B-05 each holiday moved the end by a month")
    key = {"as it was": was, "without the holidays": wo, "difference": diff}
    put("B-05", ["LIFE-03", "VAL-05", "NUM-02", "SCEN-01", "SCEN-02"], "2026-01-05", "2026-01-05", key, t,
        relations=["the difference is the two runs' exact difference", "the replay equals the simulation"])


def q05():
    """ERR-38: a key reads B-D28 only when it is received by the known-as-of date (ERR-40: it is, on 2026-01-02;
    q05_listed). Else the overpayments are the Cis exports' rows; the exports lack 14 .. 30 November, so what was
    overpaid then is a hole, and so is every figure it reaches."""
    known = "2026-01-05"   # R1: B-05's known-as-of date
    t = Trace()
    sim = b.simulate(True, known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    got = received("B-D28")
    if got <= iso(known):
        return q05_listed(known, t, sim, got)
    rc = read_cis()
    rows = [r for r in rc["rows"] if r["cp"] == "Bank Jodła S.A." and r["title"].endswith("overpayment")]
    ovs = [(r["booked"], -r["amount"]) for r in rows]
    world = {(o["date"], o["paid"]) for o in sim["mtg"].overpays}
    check(set(ovs) <= world and all(in_gap(d, rc["gaps"]) for d, _ in world - set(ovs)), "B-05 the exports show every overpayment but those on days they lack")
    check(len(rc["gaps"]) == 1, "B-05 one stretch of days no export shows")
    gap0, gap1 = rc["gaps"][0]
    rule_day = CAL.on_or_next(date(gap0.year, gap0.month, 26))   # B-HR2 .1: the 26th, or the next business day
    check(gap0 <= rule_day <= gap1, "B-05 B-HR2's November run falls on the days the export lacks")
    gap = (f"whether anything was overpaid on the mortgage from {gap0} to {gap1}, and how much (B-HR2's run of {rule_day} falls on those days): "
           f"the November Cis export stops at {gap0 - timedelta(days=1)}, and Bank Jodła's list (B-D28) is received only on {got}")
    t("overpayments the exports show", [[d.isoformat(), a, r["src"]] for (d, a), r in zip(ovs, rows)],
      f"the Cis rows to Bank Jodła \"loan 7710-22 overpayment\", each month exported in the first days of the next (story, 2025-12-31); "
      f"B-D28 is received {got}, after the known-as-of date {known}, so it isn't read")
    t("the days no export shows", [gap0.isoformat(), gap1.isoformat()],
      f"the November export stops at {gap0 - timedelta(days=1)} (B-08); B-HR2's run of {rule_day} falls on them, and what it sent to the mortgage isn't shown")

    def cells(holidays, extra):
        c, m = mortgage(holidays, sorted(ovs + extra), known=known)
        early = sum((r["interest"] for r in m.rows if not r["holiday"] and r["slot"].year == 2025 and r["slot"] < gap0), ZERO)
        return {"2022": c["interest"]["2022"], "2023": c["interest"]["2023"], "2024": c["interest"]["2024"], "2025 before": early,
                "last": c["last instalment"], "2025": c["interest"]["2025"], "balance": c["balance 2025-12-31"],
                "instalment": c["instalment 2026-01-05"]}, m

    def runs(extra):
        (a, m), (z, _) = cells(b.HOLIDAYS, extra), cells((), extra)
        months = (iso(z["last"]).year - iso(a["last"]).year) * 12 + iso(z["last"]).month - iso(a["last"]).month
        return a, z, {k: (months if k == "last" else z[k] - a[k]) for k in a}, m

    *base, m1 = runs([])
    check(base[0]["balance"] == sim["mtg"].bal, "B-05 the replay equals the simulation")
    # README § Keys, Placement: a trial overpayment on the gap's days moves exactly the holes, in each run and in the difference
    trials = [runs([(d, D(x))]) for x in ("0.01", "3000.00") for d in (gap0, rule_day, gap1)]
    for i in range(3):
        moved = {k for tr in trials for k, v in tr[i].items() if v != base[i][k]}
        check(moved == {"2025", "balance", "instalment"}, f"B-05 an overpayment on the days the exports lack moves exactly the holes (run {i}: {sorted(moved)})")
    nxt = min(s for s in m1.slots if s >= gap0)
    before = [s for s in m1.slots if s.year == 2025 and s < gap0 and s not in m1.holidays]
    t("what those days reach", [f"interest 2025: the instalment of {nxt}", "balance 2025-12-31", "instalment 2026-01-05"],
      f"an overpayment lowers the balance from its day, so it moves the interest of the instalment of {nxt}, and the balance and instalments after; "
      f"not the instalments before, nor the last instalment's day (B-D07 .4: re-levelled so that the loan ends on the same day); "
      f"trial overpayments of 0.01 and 3 000.00 on {gap0}, {rule_day} and {gap1} move exactly these, in both runs and the difference")

    def holed(c, what):
        return {"interest": {"2022": c["2022"], "2023": c["2023"], "2024": c["2024"],
                             "2025": {"kind": "partial", "value": c["2025 before"],
                                      "missing": [f"{what} interest of the instalment of {nxt}: it turns on {gap}"]}},
                "balance 2025-12-31": {"kind": "refusal", "missing": [gap],
                                       "reason": f"{what} balance from {gap0} on turns on what was overpaid on the days the November export lacks"},
                "instalment 2026-01-05": {"kind": "refusal", "missing": [gap],
                                          "reason": f"{what} instalment is set on the balance at the reset of 2026-01-01 (B-D07 .4), which turns on what was overpaid "
                                                    "on the days the November export lacks"},
                "last instalment": c["last"]}

    was, wo, diff = holed(base[0], "the"), holed(base[1], "the"), holed(base[2], "each run's")
    t("as it was", was, "B-D07 on market/dukor.csv: resets 1 Jan and 1 Jul; interest by days / 365; holidays carry none (MTG-1 .3); each overpayment's fee "
      f"B-D07's 2 % until 36 months from disbursement, none after (MTG-2 .2); with step 1's overpayments; 2025's known part: the instalments of {before[0]} .. {before[-1]}")
    t("without the holidays", wo, "the same, every instalment from 2022-04-05 on, the same overpayments")
    t("difference", diff, "without less as it was; months")
    check(diff["last instalment"] == -len(b.HOLIDAYS), "B-05 each holiday moved the end by a month")
    key = {"as it was": was, "without the holidays": wo, "difference": diff}
    put("B-05", ["LIFE-03", "VAL-05", "NUM-02", "SCEN-01", "SCEN-02"], "2026-01-05", "2026-01-05", key, t,
        relations=["the difference is the two runs' exact difference", "the replay equals the simulation",
                   "an overpayment on the days the exports lack moves only the holes"])


# --------------------------------------------------------------------------- B-06

def q06():
    t = Trace()
    legs = b.car_sale_legs()
    bz = b.business(True, known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    sep = bz["months"][(2025, 9)]
    without, _, _ = b.pit_c(sep["rev_it"], ZERO, sep["social_paid"] + sep["carry_in"])
    tax = sep["tax"] - without
    check(tax == legs["tax"], "B-06 the sale's tax is the September tax with it less without it")
    L = {"price": legs["price"], "VAT": -legs["vat"], "tax": -tax, "loan": -legs["payoff"]["total"], "loan fee": -legs["payoff"]["fee"],
         "insurance": legs["insurance"], "road tax": legs["road_tax"]}
    t("legs", L, f"VAT 22 % of the price (VAT-1 .1) whatever was deducted (VAT-6); tax: 10 % of the net under (c), contributions off the 12 % part first (PIT-11 .4, PIT-15 .4); "
                 f"loan: {legs['payoff']['balance']} + {legs['payoff']['interest']} + fee min(1 %, 0.5 % with under a year left) (B-D12, B-D16, CC-2); "
                 f"insurance {legs['unused_days']}/{legs['policy_days']} of 2 190.00 rounded down − 15.00 (B-D13, MOT-1); road tax 3/12 of 720.00 (MOT-2 .3)")
    left = sum((v for k, v in L.items() if k != "loan fee"), ZERO)
    t("left", left, "the legs added (the fee is inside the loan)")
    pv = {"share": D("0.5"), "deducted": ort(D("17600.00") * D("0.5"))}
    t("purchase VAT", pv, "VAT-5 .2: a car used privately too, no log")
    st = bz["settle"][2025]
    ks = [(2025, m) for m in range(1, 13)]
    rev_wo = st["revenue"] - b.CAR_SALE[2]
    owed_wo = sum((ort(D("0.09") * b.tier(rev_wo) * bz["months"][k]["awq4"]) for k in ks), ZERO)
    h = {"with the sale": st["owed"], "without it": owed_wo, "difference": st["owed"] - owed_wo}
    t("health 2025", h, f"the year's revenue {st['revenue']} with, {rev_wo} without: tier {b.tier(st['revenue'])} vs {b.tier(rev_wo)} for every month (SOC-4 .3, .5)")
    check(left == L["price"] + L["VAT"] + L["tax"] + L["loan"] + L["insurance"] + L["road tax"], "B-06 the legs add up")
    key = {"legs": L, "left": left, "purchase VAT": pv, "health 2025": h}
    put("B-06", ["LIFE-02", "FLOW-02", "LAW-02", "LAW-07"], "2025-12-31", "2025-12-31", key, t, relations=["the legs add up"])


# --------------------------------------------------------------------------- B-07

def q07():
    known = "2025-04-30"   # R1: B-07's known-as-of date
    t = Trace()
    _, rows = wrzos(2024, 10)
    net = [r for r in rows if r["cp"] == "MEWA SP." and r["title"] == "pay 10/2024"][0]["amount"]
    before = b.ola_payments(2024)[:9]
    lo, hi = D("7200.00"), D("30000.00")
    while hi - lo > D("0.01"):
        mid = ((lo + hi) / 2).quantize(D("0.01"))
        if b.oct_net(mid, before, known=known) < net:
            lo = mid
        else:
            hi = mid
    g = hi if b.oct_net(hi, before, known=known) == net else lo
    check(b.oct_net(g, before, known=known) == net, "B-07 a gross gives the October net")
    hits = [x for x in (g + D(k) / 100 for k in range(-2000, 2001)) if b.oct_net(x, before, known=known) == net]
    check(hits == [g], f"B-07 only one gross within 20.00 gives the net ({len(hits)})")
    check(g == b.OLA_GROSS[2024] + b.OLA_BONUS, "B-07 the inferred gross is the facts'")
    t("October 2024 gross", g, f"the gross whose net, after January-September's pay (B-D25), is {net} (records/wrz-ola-2024-10.csv); the only one within ±20.00")
    rws, tot = run_payer(b.ola_payments(2024, g - b.OLA_GROSS[2024]), known=known)
    ret = separate_return(2024, [tot])
    r = {"income": tot["income"], "base": ret["base"], "tax": ret["tax"], "advances": ret["advances"], "result": dirn(ret["due"])}
    t("2024 return", r, f"PIT-5: income − contributions − costs, rounded; PIT-3 {ret['version']}; less the advances")
    joint = {"kind": "refusal", "missing": ["a marriage: joint filing is for spouses (PIT-6), and Piotr and Ola aren't married"],
             "reason": "there is no joint unit for an unmarried couple; and Piotr's business income is taxed under form (c), not with the scale"}
    _, feb = wrzos(2025, 2)
    b27 = [r for r in feb if r["date"] <= date(2025, 2, 27)][-1]["balance"]
    end = feb[-1]["balance"]
    exp = b.expected_feb_2025_net(known=known)
    w = {"as seen on 2025-02-27": b27 + exp, "as it was": end}
    t("Wrzos 2025-02-28", w, f"on the 27th: the balance {b27} + February's net {exp}, due on the 28th (B-D25b); it came on 7 March (B-D26, known 3 March)")
    key = {"October 2024 gross": g, "2024 return": r, "jointly": joint, "Wrzos 2025-02-28": w}
    put("B-07", ["REC-05", "NUM-07", "PTY-02", "LAW-08", "REC-06", "ASK-03"], "2025-04-30", "2025-04-30", key, t)


# --------------------------------------------------------------------------- B-08

def card_spending(rows):
    card = sum((-r["amount"] for r in rows if r["type"] == "CARD PAYMENT"), ZERO)
    refunds = sum((r["amount"] for r in rows if r["type"] == "CARD REFUND"), ZERO)
    friends = sum((r["amount"] for r in rows if r["type"] == "TRANSFER IN" and "my half" in r["title"]), ZERO)
    return card - refunds - friends, (card, refunds, friends)


def q08():
    t = Trace()
    rc = read_cis()
    sim = b.simulate(True, known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    ms = rc["months"]
    bal = {"2025-10-31": ms[9]["close"], "2025-11-30": ms[10]["close"], "2025-12-31": ms[11]["close"]}
    true = {}
    for k in bal:
        d = iso(k)
        true[k] = [r for r in sim["rows"]["cis"] if r["booked"] <= d][-1]["balance"]
    check(bal == true, "B-08 the records' month ends equal the simulation's")
    t("balances", bal, "October: the last balance after (the repeated row doesn't move it); November: December's opening balance; December: the last balance after")
    for p in rc["problems"]:
        t("problem", p, "")
    # REC-02 relation: chaining the amounts with the duplicate counts it twice
    oct_ = ms[9]
    chained = oct_["opening"] + sum((r["amount"] for r in oct_["rows"]), ZERO)
    dup = [p for p in rc["problems"] if p["what"].startswith("the same row twice")][0]
    check(chained - bal["2025-10-31"] == dup["amount"], "B-08 dropping the duplicate moves the replay by exactly its amount")
    t("October chained with the repeat", chained, f"opening + every amount; less the true end = {chained - bal['2025-10-31']}, the repeated row")
    spend, parts = card_spending(rc["rows"])
    full, fparts = card_spending([dict(r, type=r["desc"]) for r in sim["rows"]["cis"]])
    gap_rows = [dict(r, type=r["desc"]) for r in sim["rows"]["cis"] if in_gap(r["booked"], rc["gaps"])]
    check(spend == full - card_spending(gap_rows)[0], "B-08 the known card spending is the year's less the missing half-month's")
    t("card spending (rows shown)", spend, f"card payments {parts[0]} − refunds {parts[1]} − friends' halves {parts[2]}")
    key = {"balances": bal, "problems": rc["problems"],
           "card spending": {"kind": "partial", "value": spend,
                             "missing": [f"Piotr's card payments of {rc['gaps'][0][0]} .. {rc['gaps'][0][1]}: the November export stops before them"]}}
    s0 = b.simulate(False, known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    wb = {k: [r for r in s0["rows"]["cis"] if r["booked"] <= iso(k)][-1]["balance"] for k in bal}
    without = {"balances": wb, "problems": [], "card spending": card_spending([dict(r, type=r["desc"]) for r in s0["rows"]["cis"]])[0]}
    t("without the twist", without, "the reversal not made; the exports whole")
    put("B-08", ["REC-02", "REC-01", "REC-03", "TIME-07", "ASK-03"], "2025-12-31", "2026-01-07", key, t, without=without,
        relations=["dropping the duplicate moves the balance by exactly its amount", "the records' replay equals the simulation"])
    return rc


# --------------------------------------------------------------------------- B-09

def receivables(twist, rc=None, *, known):
    """What is owed to Piotr on 2025-12-31, by the records (twist) or the simulation (without)."""
    reg = register()
    if twist:
        cis, gaps = rc["rows"], rc["gaps"]
    else:
        s0 = b.simulate(False, known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
        cis, gaps = [dict(r, type=r["desc"]) for r in s0["rows"]["cis"]], []
    laz = lazur_rows()
    end = date(2025, 12, 31)
    m31, _ = b.mid_on(end, known=known)
    out, missing, lines = [], [], []
    for s in reg:
        if s["issued"] < date(2024, 12, 1) or s["client"] == "Auto Mlecz sp.":
            continue  # paid before 2025 (story); the car paid on the day
        # R2: no payment row in the exports reads as unpaid; the Cis export's missing rows (read_cis, from its period) are a hole below
        if s["cur"] == "FLR":
            got = sum((r["amount"] for r in laz if s["id"] in r["desc"]), ZERO)
        else:
            got = sum((r["amount"] for r in cis if s["id"] in r["title"]), ZERO)
        left = s["gross"] - got
        if left <= D("1.00"):          # B-HR6 .1
            continue
        status = None
        if s["due"] > end:
            status = "not yet due"
        elif s["client"].startswith("Arnika"):
            status = "promised"         # B-D19: "we will pay it on 2026-01-08"
        elif s["client"] == "Studio Bez":
            status = "expected"         # B-D20 (2025-12-29): "probably next week"
        elif s["client"] == "Kessel Lenia sp.":
            status = "expected"         # B-D22 (2025-12-30): "probably on the 5th"
        elif (end - s["due"]).days > 30:
            if any(a >= s["due"] for a, z in gaps):
                missing.append(f"whether {s['client']} paid {s['id']}: the Cis rows of {gaps[0][0]} .. {gaps[0][1]} are missing")
                lines.append((s["id"], "a hole: it may have been paid in the missing rows"))
                continue
            status = "missed"
        out.append({"who": s["client"], "what": s["id"], "currency": s["cur"], "amount": left, "status": status, "_day": s["issued"]})
        lines.append((s["id"], f"{left} {s['cur']}, {status}"))
    lent = b.BARTEK["lent"][1] - sum((r["amount"] for r in cis if r["cp"] == "BARTŁOMIEJ KOS"), ZERO)
    out.append({"who": "Bartłomiej Kos", "what": "loan", "currency": "DUK", "amount": lent, "status": "promised", "_day": b.BARTEK["lent"][0]})
    lines.append(("Bartek", f"{lent}, promised for {b.BARTEK['promised']} (B-D24)"))
    out.sort(key=lambda x: (x["_day"], x["what"]))
    total = sum(((ort(x["amount"] * m31) if x["currency"] == "FLR" else x["amount"]) for x in out if x["status"] != "missed"), ZERO)
    for x in out:
        del x["_day"]
    return out, total, missing, lines, m31


def q09(rc):
    known = "2026-01-07"   # R1: B-09's known-as-of date
    t = Trace()
    out, total, missing, lines, m31 = receivables(True, rc, known=known)
    for a, bb in lines:
        t(a, bb, "")
    t("total DUK", total, f"FLR at CBL mid {m31} of 2025-12-31 (B-HR4 .5); missed debts count nothing (B-HR4 .2)")
    key = {"kind": "partial", "value": {"owed": out, "total DUK": total}, "missing": missing}
    w_out, w_total, w_missing, _, _ = receivables(False, known=known)
    check(not w_missing, "B-09 without the twist nothing is missing")
    without = {"owed": w_out, "total DUK": w_total}
    t("without the twist", without, "Arnika-Delta's October payment stands; the November export is whole, so Ziarno's invoice is plainly missed")
    put("B-09", ["REC-04", "OWN-02", "TIME-02", "ASK-03"], "2025-12-31", "2026-01-07", key, t, without=without)
    return total, w_total


# --------------------------------------------------------------------------- B-10

def replay_hr2(rc, *, known):
    """B-HR2 from the records: each rule's day's Cis end before the rule's rows, the Lazur florens, the register."""
    reg = register()
    laz = lazur_rows()
    out, missing = {}, {}
    for d in b.rule_days():
        if in_gap(d, rc["gaps"]):
            missing[d.isoformat()] = f"the Cis balance at the end of {d}: the November export stops before it, so that day's run"
            continue
        own = [r for r in rc["rows"] if r["booked"] == d and (r["title"] == "loan 7710-22 overpayment" or r["title"].startswith("to savings"))]
        rest = [r for r in rc["rows"] if r["booked"] <= d and r not in own]
        cis = rest[-1]["balance"]
        flr = ([r for r in laz if r["date"] <= d] or [{"balance": ZERO}])[-1]["balance"]
        sl = [{"id": s["id"], "issued": s["issued"], "vat_duk": s["vat_duk"] if s["cur"] == "FLR" else s["vat"], "net_duk": revenue_duk(s)} for s in reg]
        run = b.hr2(d, cis, flr, sl, "v1" if d < b.V2_STATED else "v2", known=known)
        recorded = (-sum((r["amount"] for r in own if r["title"] == "loan 7710-22 overpayment"), ZERO),
                    -sum((r["amount"] for r in own if r["title"].startswith("to savings")), ZERO))
        check(recorded == (run["overpay"], run["save"]), f"B-10 the records show what the rule called for on {d}")
        out[d.isoformat()] = (run, {"overpayment": run["overpay"], "to savings": run["save"]})
    return out, missing


def q10(rc):
    known = "2026-01-07"   # R1: B-10's known-as-of date
    t = Trace()
    rep, missing = replay_hr2(rc, known=known)
    sim = b.simulate(True, known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    for run in sim["runs"]:
        if run["day"].isoformat() in rep:
            check(rep[run["day"].isoformat()][0]["overpay"] == run["overpay"] and rep[run["day"].isoformat()][0]["save"] == run["save"],
                  f"B-10 the records' replay equals the simulation on {run['day']}")
    for d, (run, v) in rep.items():
        t(d, v, f"{run['version']}: Cis {run['cis']} + {run['flr']} FLR × {run['lazur_buy']} rounded down = {run['have']}; "
                f"reserve {run['reserve']} (VAT {run['vat']}, 12 % {run['net12']}); surplus {run['surplus']}"
                + ("; Cis keeps 1 000.00, so cut" if run["bound"] else ""))
    # README § Keys: a refusal stands in place of the one value it concerns, the day's run; the other days stay plain
    v25 = {d.isoformat(): (rep[d.isoformat()][1] if d.isoformat() in rep else
                           {"kind": "refusal", "missing": [missing[d.isoformat()]], "reason": "the day's balance is not in the exports"})
           for d in b.rule_days() if d.isoformat() in rep or d.isoformat() in missing}
    cf = b.simulate(True, v2_from=date(2025, 1, 1), known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    since = {}
    for r0, r1 in zip(sim["runs"][:3], cf["runs"][:3]):
        since[r1["day"].isoformat()] = {"overpayment": r1["overpay"], "to savings": r1["save"]}
        t(f"v2 since January, {r1['day']}", since[r1["day"].isoformat()], f"Cis {r1['cis']} (really {r0['cis']}): v2's floor on the balance as it would have been")
    after = date(2025, 3, 26)
    c_real = [r for r in sim["rows"]["cis"] if r["booked"] <= after][-1]["balance"]
    c_cf = [r for r in cf["rows"]["cis"] if r["booked"] <= after][-1]["balance"]
    short = c_cf - c_real
    check(short == sim["runs"][0]["save"] - cf["runs"][0]["save"], "B-10 the shortfall is January's extra transfer to savings, carried")
    since["short after 2025-03-26"] = short
    t("short after 2025-03-26", short, "v2 since January keeps 3 000.00 more in January; with it February's and March's runs move the same; the 3 000.00 stays short")
    v3 = {"kind": "refusal",
          "missing": ["what breaks the loop: v3's reserve needs the instalments after the day's overpayment, and the overpayment is what's above the reserve",
                      "the Cis balance on 2026-01-26: on 2026-01-07 January's bookings are known only as the forecast's assumptions (story item 5), with its project invoices random"],
          "reason": "a loop: the overpayment and the reserve are each sized from the other"}
    key = {"2025": v25, "v2 since January": since, "v3 on 2026-01-26": v3}
    s0 = b.simulate(False, known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    without = {"2025": {r["day"].isoformat(): {"overpayment": r["overpay"], "to savings": r["save"]} for r in s0["runs"]},
               "v2 since January": since, "v3 on 2026-01-26": v3}
    t("without the twist", without["2025"], "no reversal; the November export whole")
    put("B-10", ["RULE-01", "RULE-02", "RULE-03", "RULE-04", "RULE-06", "REC-07", "TIME-07", "ASK-03"], "2026-01-26", "2026-01-07", key, t,
        without=without, relations=["the records' replay equals the simulation", "the shortfall is January's extra transfer, carried"])


# --------------------------------------------------------------------------- B-11

def flat_costs(sim):
    mt = sim["mtg"]
    inst = sum((r["instalment"] for r in mt.rows if not r["holiday"] and r["pay_day"].year == 2025), ZERO)
    over = sum((o["paid"] for o in mt.overpays if o["date"].year == 2025), ZERO)
    fees = b.FLAT_FEE * 12
    ins = b.FLAT_INS_2025[1]
    total = inst + over + fees + ins
    ola_paid = b.OLA_ORDER * 11 + ins
    ola_share = ort(total * D("0.40"))
    return {"total": total, "Piotr paid": total - ola_paid, "Ola paid": ola_paid, "Piotr's share": total - ola_share,
            "Ola's share": ola_share, "Ola owes Piotr": ola_share - ola_paid}, (inst, over, fees, ins)


WRZ_LAST = date(2025, 3, 31)   # the end of the last statement Ola kept (records/wrz-ola-2025-03.csv)


def wrzos_known(*, known):
    """Ola's Wrzos balance on 2025-12-31, its known part: the March statement's reading, then what the papers
    settle after it, her pay (B-D25b) in and her order to Piotr (B-HR1, its rows in Cis) out. What she spent
    from April on is the hole (README § Keys, Placement: the smallest value missing)."""
    _, mar = wrzos(2025, 3)
    pays = [n for n in b.ola_nets(2025, known=known) if WRZ_LAST < n["date"] <= date(2025, 12, 31)]
    orders = [r for r in read_cis()["rows"] if r["cp"] == "ALEKSANDRA WILK" and r["booked"] > WRZ_LAST]
    check(len(pays) == 9 and len(orders) == 9, "B-11 Wrzos after March: nine pays in, nine orders out")
    part = mar[-1]["balance"] + sum((n["net"] for n in pays), ZERO) - sum((r["amount"] for r in orders), ZERO)
    return part, {"March 2025 statement": mar[-1]["balance"], "pay April .. December": sum((n["net"] for n in pays), ZERO),
                  "orders to Piotr": -sum((r["amount"] for r in orders), ZERO)}


def worth(sim, recv_total, *, known):
    bz = sim["bz"]
    bal = sim["bal"]
    m31, _ = b.mid_on(date(2025, 12, 31), known=known)
    fc, _ = flat_costs(sim)
    flat = b.VALUATION[1]
    debt = sim["mtg"].bal
    dec_ = bz["months"][(2025, 12)]
    owed = dec_["social"]["total"] + dec_["health"] + dec_["tax"] + dec_["vat_due"] + bz["settle"][2025]["difference"]
    o_flat, o_debt = ort(flat * D("0.40")), ort(debt * D("0.40"))
    _, tot = run_payer(b.ola_payments(2025), known=known)
    ret = separate_return(2025, [tot])
    piotr = {"Cis": bal["cis"], "savings": bal["sav"], "florens": ort(bal["laz"] * m31), "flat": flat - o_flat, "mortgage": -(debt - o_debt),
             "owed to him": recv_total, "Ola owes him": fc["Ola owes Piotr"], "taxes owed": -owed}
    ola = {"Wrzos": wrzos_known(known=known)[0], "flat": o_flat, "mortgage": -o_debt, "owes Piotr": -fc["Ola owes Piotr"],
           "2025 return": -ret["due"]}
    return piotr, ola, ret


PIOTR_OCT = ("whether October 2025's contributions, health contribution, PIT advance (due 2025-11-20) and VAT (due 2025-11-25) were paid: "
             "the Cis rows of 2025-11-14 .. 2025-11-30 are missing")


OLA_SPEND = ("what Ola spent from her Wrzos account from 2025-04-01 to 2025-12-31 (her pay in and her order to Piotr out are known): "
             "her last statement is March 2025's")


def q11(recv):
    known = "2026-01-07"   # R1: B-11's known-as-of date
    t = Trace()
    sim = b.simulate(True, known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    fc, parts = flat_costs(sim)
    jod = (ROOT / "B" / "docs" / "B-D28.md").read_text(encoding="utf-8")
    listed = sum((D(ln.split("|")[3].strip().replace(" ", "")) for ln in jod.splitlines() if ln.startswith("| ") and ln[2].isdigit()), ZERO)
    check(listed == parts[0] + parts[1], "B-11 the instalments and overpayments are Bank Jodła's list's")
    rc_ola = [r for r in read_cis()["rows"] if r["cp"] == "ALEKSANDRA WILK"]
    check(len(rc_ola) == 11, "B-11 Ola's order: eleven in Cis, March's rejected (RULE-07)")
    t("flat costs", fc, f"instalments {parts[0]} + overpayments {parts[1]} (B-D28) + fees {parts[2]} (B-D09) + insurance {parts[3]} (B-D10); "
                        f"Ola: 11 × 1 700.00 (March rejected, records/wrz-ola-2025-03.csv) + the insurance; 40 % to the ort (B-HR1)")
    check(fc["Piotr paid"] - fc["Piotr's share"] + fc["Ola paid"] - fc["Ola's share"] == 0, "B-11 the netting balances add up to zero")
    piotr, ola, ret = worth(sim, recv[0], known=known)
    t("Piotr", piotr, "B-HR4")
    _, wz = wrzos_known(known=known)
    t("Ola's Wrzos, the known part", wz, "the balance after the last row of records/wrz-ola-2025-03.csv; her nets for April .. December, "
      "each paid on the 28th or the last business day before (B-D25b, PIT-4, SOC-1, SOC-2); 9 × 1 700.00 to Piotr on the 5th (B-HR1, the Cis rows)")
    t("Ola", ola, f"B-HR4; her 2025 return: tax {ret['tax']} less advances {ret['advances']}; what she spent from Wrzos after March unknown")
    P, O = sum(piotr.values(), ZERO), sum(ola.values(), ZERO)
    whole = (piotr["Cis"] + piotr["savings"] + piotr["florens"] + b.VALUATION[1] - sim["mtg"].bal + piotr["owed to him"]
             + piotr["taxes owed"] + ola["2025 return"] + ola["Wrzos"])   # the household counted whole: the claim between us cancels
    check(P + O == whole, "B-11 the people's net worths add up to the household's")
    t("household, counted whole", whole, "the flat and the mortgage unsplit; Ola's debt to Piotr cancels")
    # README § Keys: a partial stands in place of the values it concerns; the flat costs stay plain
    hole_o = [OLA_SPEND]
    hole_p = [PIOTR_OCT]
    key = {"flat costs": fc, "Piotr": {"kind": "partial", "value": P, "missing": hole_p},
           "Ola": {"kind": "partial", "value": O, "missing": hole_o},
           "household": {"kind": "partial", "value": P + O, "missing": hole_p + hole_o}}
    s0 = b.simulate(False, known=ALL_KNOWN)  # R1: the world's replay, whose rows are the records' (checked against them); the key's own lookups pass its known
    fc0, _ = flat_costs(s0)
    p0, o0, _ = worth(s0, recv[1], known=known)
    P0, O0 = sum(p0.values(), ZERO), sum(o0.values(), ZERO)
    without = {"flat costs": fc0, "Piotr": P0, "Ola": O0, "household": P0 + O0}
    t("without the twist", without, "the reversal not made: more overpaid in November and December")
    put("B-11", ["ASK-02", "OWN-03", "OWN-04", "FLOW-03", "RULE-07", "OWN-02", "PTY-01"], "2025-12-31", "2026-01-07", key, t, without=without,
        relations=["the netting balances add up to zero", "the people's net worths add up to the household's"])


# --------------------------------------------------------------------------- B-12

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


def q12(end=None):
    """end: the run-end knob; None is B-12 as asked (to 2028-12-31), written to keys/."""
    known = "2026-01-02"   # R1: B-12's known-as-of date
    t = Trace()
    stop_at = end or b_paths.END
    on_ = stop_at.isoformat()
    fc = b_paths.forecast(N_PATHS, known=known, end=stop_at)
    on, off = fc["on"], fc["off"]
    check(all(a["tax"] == c["tax"] for a, c in zip(on, off)), "B-12 common random numbers: the same tax on and off, path by path")
    share_on = rnd([1 if r["breach"] else 0 for r in on], 6)
    share_off = rnd([1 if r["breach"] else 0 for r in off], 6)
    stop = share_on["r"] > b_paths.SHARE_LIMIT
    morts = {r["mortgage"] for r in off}
    check(len(morts) == 1, "B-12 with nothing overpaid the mortgage is the same on every path")
    mort = morts.pop()
    z = fc["zero"]
    check(z[0]["tax"] == z[1]["tax"] == fc["closed_tax"], "B-12 zero variance equals the closed form, run twice")
    t("paths", N_PATHS, f"seed {b_paths.SEED} + i; each run with B-HR2 v2's overpayments and without, on the same draws"
      + (f"; the forecast ends on {on_}, each path being the first part of the full run's (the same draws)" if end else ""))
    t("share below 60 000.00", {"with": share_on, "without": share_off}, "paths whose lowest month end of Cis + savings is under 60 000.00 (B-HR3)"
      + (f", month ends 2026-01 .. {stop_at.strftime('%Y-%m')}" if end else ""))
    t("overpayments stopped", stop, f"{share_on['r']} > 0.20")
    sf_on = rnd([r["shortfall"] for r in on], 2)
    sf_off = rnd([r["shortfall"] for r in off], 2)
    t("mean shortfall", {"with": sf_on, "without": sf_off}, "60 000.00 − the lowest month end, 0 where it isn't below")
    dec_ = off if stop else on
    tax = rnd([r["tax"] for r in dec_], 2)
    liq = rnd([r["liquid_end"] for r in dec_], 2)
    if end:
        t(f"mortgage {on_} (labelled 2028-12-31)", mort, "no overpayments: B-D07 at the rate of the 2026-01-01 reset (F8), re-levelled after each reset")
        t(f"tax 2026-01 .. {stop_at.strftime('%Y-%m')} (labelled 2026-2028)", tax, "the months' PIT-10 (c) tax, 12 % or 15 % from 2027 by the path's law draw (F7)")
        t(f"liquid {on_} (labelled 2028-12-31)", liq, f"Cis + savings at the end of {stop_at.strftime('%Y-%m')}")
        t(f"zero variance tax 2026-01 .. {stop_at.strftime('%Y-%m')}", fc["closed_tax"], "16 000.00 a month, no projects, 12 %: Σ duk(duk(16 000 − contributions paid − carried) × 12 %) (F9)")
    else:
        t("mortgage 2028-12-31", mort, "no overpayments: B-D07 at the rate of the 2026-01-01 reset (F8)")
        t("tax 2026-2028", tax, "the months' PIT-10 (c) tax, 12 % or 15 % from 2027 by the path's law draw (F7)")
        t("liquid 2028-12-31", liq, "Cis + savings at the end of December 2028")
        t("zero variance tax", fc["closed_tax"], "16 000.00 a month, no projects, 12 %: Σ duk(duk(16 000 − contributions paid − carried) × 12 %) (F9)")
    key = {"overpayments stopped": stop, "mortgage 2028-12-31": mort,
           "share below": {"with overpayments": share_on, "without": share_off},
           "mean shortfall": {"with overpayments": sf_on, "without": sf_off},
           "tax 2026-2028": tax, "liquid 2028-12-31": liq, "zero variance tax": fc["closed_tax"]}
    put("B-12", ["RAND-01", "RAND-02", "RAND-03", "RAND-04", "LAW-13", "RULE-09", "SURF-03"], on_, "2026-01-02", key, t,
        relations=["common random numbers: both runs on the same paths", "zero variance equals the closed form"],
        small=(N_PATHS, end) if end else None)


# --------------------------------------------------------------------------- B-13: the floren 5 % lower from 2025-07-01 (a scenario)

FLR_LOW = (date(2025, 7, 1), D("0.95"))   # B-13: every FLR rate dated from that day, times 0.95
B13_KEEP = D("2000.00")                   # B-13 rule 1: the florens kept in the Lazur account


def flr_low(rate, day):
    """B-13's scenario rate: a rate dated `day` (a CBL mid rate's day, or Bank Lazur's table's day), from 2025-07-01 times
    0.95, rounded to 4 decimal places, half up, as both sources publish theirs; before that day, as it was."""
    return (rate * FLR_LOW[1]).quantize(D("0.0001"), rounding=ROUND_HALF_UP) if day >= FLR_LOW[0] else rate


def key_of(qid):
    """An earlier key of this run, read back from its file (keys/B/<qid>.md)."""
    for ln in (ROOT / "keys" / "B" / f"{qid}.md").read_text(encoding="utf-8").splitlines():
        if ln.startswith("key: "):
            return json.loads(ln[5:], parse_float=Decimal)


def b13_rule(laz, t, *, known):
    """B-13 rule 1, the scenario's own house rule (not one of B's): from 2025-07-01, on the 1st of each month or the next
    business day, the florens in the Lazur account above 2 000.00 FLR at the end of that day (everything else booked that day
    counted) are sold to Cis: at Bank Lazur's buy rate of the day (the scenario's), the dukats to the ort, half up, credited
    the next business day (B-D23 §4). Returns the sales and the florens sold in all."""
    sales, sold = [], ZERO
    for m in range(7, 13):
        d = CAL.on_or_next(date(2025, m, 1))
        rows = [r for r in laz if r["date"] <= d]
        today = [r for r in laz if r["date"] == d]
        bal = (rows[-1]["balance"] if rows else ZERO) - sold
        booked = ("; booked that day first: " + ", ".join(
            (f"{r['desc'].split('SP. ')[1]} in {r['amount']}" if r["amount"] > 0 else f"the story's sale of that day's florens {r['amount']}")
            for r in today)) if today else ""
        if bal > B13_KEEP:
            x = bal - B13_KEEP
            rate = flr_low(b.lazur_buy(d, known=known), d)
            mid, src = b.mid_on(d, known=known)
            mid_l = flr_low(mid, d if CAL.is_business(d) else CAL.before(d))
            sale = {"day": d.isoformat(), "FLR sold": x, "DUK": ort(x * rate), "credited": CAL.on_or_next(d + timedelta(days=1)).isoformat()}
            worth = ort(x * mid_l)
            t(f"B-13 rule 1 on {d}", sale, f"Lazur at the day's end {bal} FLR{booked}; sells {bal} − {B13_KEEP}; at Lazur's buy "
              f"{b.lazur_buy(d, known=known)} × 0.95 = {rate}, to the ort (B-D23 §4); worth that day, counting as we count: × CBL mid "
              f"{mid} × 0.95 = {mid_l} (B-HR4 .5) = {worth}; gain or loss {sale['DUK'] - worth}")
            sales.append(dict(sale, worth=worth))
            sold += x
        else:
            t(f"B-13 rule 1 on {d}", ZERO, f"Lazur at the day's end {bal} FLR{booked}; not above {B13_KEEP}: nothing sold")
    check(all(r["balance"] - sum((x["FLR sold"] for x in sales if x["day"] <= r["date"].isoformat()), ZERO) >= 0 for r in laz),
          "B-13 the rule never sells florens the account hasn't")
    return sales, sold


def q13():
    known = "2026-01-07"   # R1: B-13's known-as-of date, as B-03's
    t = Trace()
    reg = [s for s in register() if s["client"] == "Kessel Lenia sp."]
    laz = lazur_rows()
    b03 = key_of("B-03")["invoices"]
    inv = {}
    for s in reg:
        mid, src = b.mid_for_event(s["issued"], known=known)
        day = CAL.before(s["issued"])
        ml = flr_low(mid, day)
        line = {"revenue": ort(s["net"] * ml), "VAT": ort(s["vat"] * ml)}
        if s["issued"] < FLR_LOW[0]:
            check(line == {k: b03[s["id"]][k] for k in ("revenue", "VAT")}, f"B-13 an invoice before 2025-07-01 is as in B-03 ({s['id']})")
            continue
        inv[s["id"]] = line
        t(f"(a) {s['id']}, issued {s['issued']}", line, f"CBL mid of {day}, the business day before (GEN-4 .1, .2; PIT-15 .3, VAT-3 .3): "
                                                      f"{mid} × 0.95 = {ml}; {s['net']} FLR net, {s['vat']} FLR VAT, each to the ort")
    check(len(inv) == 6 and set(inv) == {k for k in b03 if k in inv}, "B-13 B-03's six invoices issued from 2025-07-01")
    flr = laz[-1]["balance"]
    m31, src = b.mid_on(date(2025, 12, 31), known=known)
    m31l = flr_low(m31, date(2025, 12, 31))
    lz = {"FLR": flr, "DUK": ort(flr * m31l)}
    t("(a) Lazur at 2025-12-31", lz, f"the florens of laz-8813-flr-2025.csv's last row × CBL mid {m31} × 0.95 = {m31l} (B-HR4 .5)")
    sales, sold = b13_rule(laz, t, known=known)
    left = flr - sold
    lzb = {"FLR": left, "DUK": ort(left * m31l)}
    check(left + sold == flr, "B-13 the florens sold plus those left are the account's")
    t("(b) Lazur at 2025-12-31", lzb, f"{flr} − {sold} FLR sold by the rule, × {m31l} (B-HR4 .5)")
    gl = sum((x["DUK"] - x["worth"] for x in sales), ZERO)
    t("(b) currency gain or loss of the rule's sales", gl, "Σ the dukats got − the florens' worth on the sale's day (B-HR4 .5); a loss negative")
    key = {"(a)": {"invoices": inv, "Lazur at 2025-12-31": lz},
           "(b)": {"sales": [{k: v for k, v in x.items() if k != "worth"} for x in sales], "Lazur at 2025-12-31": lzb,
                   "currency gain or loss": gl}}
    put("B-13", ["CUR-01", "CUR-02", "CUR-03", "SCEN-01", "SCEN-03", "RULE-01", "TIME-02", "TIME-07"], "2025-12-31", known, key, t,
        relations=["a change from 2025-07-01 changes no figure before it: the invoices issued before then are B-03's",
                   "the florens the rule sells and those left add up to the account's"])


def main():
    if END_AT:   # the run-end knob: B-12 alone, to keys-small/
        q12(END_AT)
        if FAILS:
            print("relations failed:", *FAILS, sep="\n  ")
            sys.exit(1)
        print(f"B-12 written to keys-small/B/ ({N_PATHS} paths, the forecast ending {END_AT}), relations pass")
        return
    q01()
    q02()
    q03()
    q04()
    q05()
    q06()
    q07()
    rc = q08()
    recv = q09(rc)
    q10(rc)
    q11(recv)
    q12()
    q13()
    if FAILS:
        print("relations failed:", *FAILS, sep="\n  ")
        sys.exit(1)
    print("B: 13 keys written, relations pass")


if __name__ == "__main__":
    main()
