#!/usr/bin/env python3
"""Household A's keys: one file per question in keys/A/, each a YAML block and the derivation.

Reads the home law (rules as code in fam/home.py), the gazette, the market files, A's records and the
facts A's documents state (fam/a.py, each citing its document). Asserts every relation on its own
output; exits 1 if one fails.

Run: python3 -I tmp/cases/tools/keys_a.py
"""
import pathlib
import sys
from datetime import date, timedelta
from decimal import Decimal, ROUND_UP
from fractions import Fraction

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from fam import a  # noqa: E402
from fam.home import CAL, ZERO, D, duk, ort, figure, price_on, prices, units_down, run_payer, separate_return, joint_return  # noqa: E402
from lib.keys import Trace, enc  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parents[1]
REC = ROOT / "A" / "records"
FAILS = []


def check(cond, what):
    if not cond:
        FAILS.append(what)


def dirn(x):
    """A direction word for an amount the taxpayer pays (+) or gets back (−); where nothing moves, a bare 0 (README § Keys)."""
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
    body = (f"# Key · {qid}\n\nWritten by `tools/keys_a.py`; don't edit.\n\n```yaml\n" + "\n".join(head) +
            "\n```\n\n## Derivation\n\n" + t.render() + "\n")
    p = ROOT / "keys" / "A" / f"{qid}.md"
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(body, encoding="utf-8")


# --------------------------------------------------------------------------- records

def dc(s):
    # R2: only a column the row's operation leaves blank is empty (Rzeka: units and price on DEPOSIT, TAX WITHHELD, WITHDRAWAL; price on TRANSFER IN); every amount and balance read is filled
    return D(s.replace(",", ".")) if s else ZERO


def dmy(s, sep="."):
    d, m, y = s.split(sep)
    return date(int(y), int(m), int(d))


def joint_rows():
    rows = []
    for f in sorted(REC.glob("wrz-joint-*.csv")):
        lines = f.read_text(encoding="utf-8").splitlines()
        for i, ln in enumerate(lines[6:], 7):
            d, typ, cp, title, amt, bal = ln.split(";")
            rows.append({"date": dmy(d), "type": typ, "counterparty": cp, "title": title, "amount": dc(amt), "balance": dc(bal),
                         "src": f"{f.name} line {i}"})
    return rows


def rzeka_rows(name):
    f = REC / name
    out = []
    for i, ln in enumerate(f.read_text(encoding="utf-8").splitlines()[2:], 3):
        d, op, fund, units, price, value, fee, cash, note = ln.split(";", 8)
        out.append({"date": dmy(d, "/"), "op": op, "fund": fund, "units": dc(units), "price": dc(price), "value": dc(value),
                    "fee": dc(fee), "cash": dc(cash), "note": note, "src": f"{name} line {i}"})
    return out


def wrapped_rows():
    f = REC / "brz-wrapped-ania.csv"
    out = []
    for i, ln in enumerate(f.read_text(encoding="utf-8").splitlines()[1:], 2):
        acct, d, typ, fund, units, px, amt = ln.split(",")
        # R2: contribution and withdrawal rows leave fund, units and price blank (records/brz-wrapped-ania.csv); no figure is missing
        out.append({"date": date.fromisoformat(d), "type": typ, "fund": fund, "units": D(units) if units else ZERO,
                    "price": D(px) if px else ZERO, "amount": D(amt), "src": f"brz-wrapped-ania.csv line {i}"})
    return out


def wcp_units(name, upto):
    lines = (REC / name).read_text(encoding="utf-8").splitlines()
    rows = [(i, ln.split(";")) for i, ln in enumerate(lines, 1) if ln[:1].isdigit()]
    i, last = [(i, r) for i, r in rows if dmy(r[0]) <= upto][-1]
    return dc(last[5]), f"{name} line {i}"


def lots_rzeka(rows):
    """Lots from BUY rows (cost = value + fee, PIT-8 .2), less SELL rows first in, first out (PIT-8 .3)."""
    lots = []
    for r in rows:
        if r["op"] == "BUY":
            lots.append({"fund": r["fund"], "date": r["date"], "units": r["units"], "cost": r["value"] + r["fee"], "src": r["src"]})
        elif r["op"] == "SELL":
            need = r["units"]
            for l in sorted([l for l in lots if l["fund"] == r["fund"] and l["units"] > 0], key=lambda l: l["date"]):
                t = min(need, l["units"])
                l["cost"] = l["cost"] - (l["cost"] * t / l["units"] if t < l["units"] else l["cost"])
                l["units"] -= t
                need -= t
                if need == 0:
                    break
    return [l for l in lots if l["units"] > 0]


# --------------------------------------------------------------------------- A-01, A-02, A-03: returns and payroll

def payer_line(t, who, year, tot, did):
    return t(f"{who} {year}: {did}", {k: tot[k] for k in ("income", "costs", "soc", "base", "advance")},
             "income − social contributions − costs = base (PIT-2 .3)" + ("; the SIO's parental pay, taxed under PIT-4 (SOC-8 .4, .5)" if "SIO" in did else ""))


def q01():
    t = Trace()
    out = {}
    # 2021: separately, and jointly under each reading of PIT-6 .4 against PIT-5 v1 .3
    pr = a.payroll(2021, known="2024-04-30")  # R1: A-01 known as of 2024-04-30
    tn, tk = pr["NURT"][1], pr["KALINA"][1]
    payer_line(t, "Tomek", 2021, tn, "A-D13-NURT-2021")
    payer_line(t, "Ania", 2021, tk, "A-D13-KALINA-2021")
    sa = separate_return(2021, [tk])
    st = separate_return(2021, [tn])
    t("2021 Ania alone (PIT-5 v1, PIT-3 v1)", sa["due"], f"tax {sa['tax']} on base {sa['base']} less health {tk['health_ded']:.2f}, less advances {sa['advances']}")
    t("2021 Tomek alone", st["due"], f"tax {st['tax']} on base {st['base']} less health {tn['health_ded']:.2f}, less advances {st['advances']}")
    je = joint_return(2021, [tk], [tn], health="each")
    jn = joint_return(2021, [tk], [tn], health="none")
    t("2021 jointly, reading E: each spouse's health deduction (PIT-5 v1 .3) also comes off the joint tax", je["due_each"],
      f"joint tax 2 × tax(½ × {je['combined']}) = {je['joint_tax']}; parts {je['parts']}; less health {[f'{x:.2f}' for x in je['health_ded']]} and advances {je['advances']}")
    t("2021 jointly, reading N: PIT-6 .4 deducts only the child reliefs", jn["due_each"], f"parts {jn['parts']} less advances {jn['advances']}")
    check(sum(je["parts"]) == je["joint_tax"], "A-01 2021 parts add up")
    out["2021"] = {"separately": {"Ania": dirn(sa["due"]), "Tomek": dirn(st["due"])},
                   "jointly": {"kind": "by-case", "cases": {
                       "E: the health deduction comes off the joint tax": {"Ania": dirn(je["due_each"][0]), "Tomek": dirn(je["due_each"][1])},
                       "N: only the child reliefs come off the joint tax": {"Ania": dirn(jn["due_each"][0]), "Tomek": dirn(jn["due_each"][1])}}}}
    # 2022 and 2023: joint (the household filed jointly; PIT-6 .1 holds all year: married 2016, community, residents, no business)
    without = {}
    for year in (2022, 2023):
        for law in ("enacted", "no-cut"):
            h = a.household_return(year, law, known="2024-04-30")
            j = h["joint"]
            if law == "enacted":
                for payer, (rows, tot) in h["payroll"].items():
                    payer_line(t, {"NURT": "Tomek", "KALINA": "Ania", "SIO": "Ania"}[payer], year, tot, f"A-D13-{payer}-{year}")
                t(f"{year} parents' income (PIT-7 .2)", h["parents_income"], "Ania's base + Tomek's base")
                t(f"{year} child relief (PIT-7)", h["relief"]["relief"], f"125.00 × months {h['relief']['full'] / 125} − cut {h['relief']['cut']} (income above 112 000.00), halves")
                t(f"{year} joint tax (PIT-6 .3, PIT-3 {j['version']})", j["joint_tax"], f"combined base {j['combined']}, half {j['half']}; residents, the calendar year (PIT-1 .1, .2)")
                t(f"{year} each spouse's part (PIT-6 .5)", j["parts"], f"in proportion to bases {j['bases']}, whole dukats")
                t(f"{year} each spouse's amount: part − relief − advances", j["due_each"], f"reliefs {j['reliefs']}, advances {j['advances']}")
                check(sum(j["parts"]) == j["joint_tax"], f"A-01 {year} parts add up")
                out[str(year)] = {"joint tax": j["joint_tax"], "Ania": dirn(j["due_each"][0]), "Tomek": dirn(j["due_each"][1]),
                                  "together": dirn(j["due_each"][0] + j["due_each"][1])}
                if year == 2023:
                    out["2023"]["child relief"] = h["relief"]["relief"]
            else:
                without[str(year)] = {"joint tax": j["joint_tax"], "together": dirn(j["due_each"][0] + j["due_each"][1])}
                t(f"{year} without the July 2022 cut (PIT-3 v2 throughout, advances too)", without[str(year)])
    # the causes of 2022's refund: each alone removed
    h = a.household_return(2022, known="2024-04-30")
    j = h["joint"]
    base_due = j["due_each"][0] + j["due_each"][1]
    j2 = joint_return(2022, h["ania"], h["tomek"], reliefs=h["joint"]["reliefs"], law="no-cut")
    cut_effect = (j2["after_reliefs"] - j2["advances"][0] - j2["advances"][1]) - base_due
    sa22 = separate_return(2022, h["ania"])
    st22 = separate_return(2022, h["tomek"])
    joint_effect = (sa22["due"] + st22["due"]) - base_due
    t("2022 cause 1: the July cut applied to the whole year (PIT-3 v3 governs tax year 2022)", cut_effect,
      "the same return with PIT-3 v2, the advances as withheld, less the actual")
    t("2022 cause 2: joint filing (PIT-6)", joint_effect, "separate returns less the joint return")
    causes = [{"cause": "the July 2022 rate cut applied to the whole year in the return", "refund larger by": cut_effect},
              {"cause": "joint filing, which halves the cash-out year's base", "refund larger by": joint_effect}]
    out["2022"]["causes"] = sorted(causes, key=lambda c: -c["refund larger by"])  # the larger first
    put("A-01", ["LAW-03", "LAW-04", "LAW-06", "LAW-07", "LAW-08", "PTY-02", "PTY-04", "LAW-02", "LAW-01", "REC-05", "TIME-03", "NUM-03", "LIFE-04", "ASK-05"],
        "2024-04-30", "2024-04-30", out, t, without=without,
        relations=["each year's parts add up to the joint tax (PIT-6 .5)", "2021 is the same under the cut and without it"])


def q02():
    t = Trace()
    h = a.household_return(2022, law="known-2022-05-31", cashout=False, known="2022-05-31")  # R1: A-02 known as of 2022-05-31
    j = h["joint"]
    for payer, (rows, tot) in h["payroll"].items():
        t(f"{payer} 2022 as expected on 2022-05-31 (contract pay, no cash-out: A-D03 is dated 2022-10-03)", {k: tot[k] for k in ("income", "base", "advance")})
    t("joint tax (PIT-6 .3) under PIT-3 v2, the version governing 2022 until v3 was published on 2022-06-14", j["joint_tax"], f"combined base {j['combined']}")
    due = j["due_each"][0] + j["due_each"][1]
    t("expected result: joint tax − advances (PIT-4 v2 with PIT-3 v2)", due)
    nc = a.household_return(2022, law="no-cut", cashout=False, known="2022-05-31")["joint"]
    check(nc["joint_tax"] == j["joint_tax"], "A-02 equals the no-cut reckoning on the facts known then (LAW-03 relation)")
    put("A-02", ["VAL-03", "REC-06", "LAW-03"], "2022-12-31", "2022-05-31",
        {"joint tax": j["joint_tax"], "advances": j["advances"][0] + j["advances"][1], "result": dirn(due)}, t,
        relations=["a change published 2022-06-14 changes no answer known before it"])


def q03():
    t = Trace()
    res = {}
    for law in ("enacted", "no-cut"):
        rows, tot = a.payroll(2022, law, known="2023-01-31")["NURT"]  # R1: A-03 known as of 2023-01-31
        tab = []
        for r in rows:
            if r["month"] < "2022-07":
                continue
            pay = [x for x in r["detail"] if "gross" in x][0]
            line = {"paid": pay["date"], "income": r["income"], "base": r["base"], "advance": r["advance"], "net": r["net"]}
            if law == "enacted":
                t(f"payment {pay['date']}", line, f"base = round({r['income']} − {r['soc']} − {r['costs']}); "
                  f"advance = lower of {r['candidates']} (PIT-4 v2 .4); net = gross {pay['gross']} − soc {r['soc']} − health {r['health']} − plan {r['wcp_ee']} − advance")
                check(r["net"] + r["advance"] + r["soc"] + r["health"] + r["wcp_ee"] == pay["gross"], f"A-03 net adds up {pay['date']}")
            tab.append(line)
        res[law] = tab
    # NUM-03: rounding each month vs once over the half year
    rows, _ = a.payroll(2022, known="2023-01-31")["NURT"]
    h2 = [r for r in rows if r["month"] >= "2022-07"]
    stepwise = sum((r["advance"] for r in h2), ZERO)
    t("advances withheld July–December, each rounded (GEN-5.1)", stepwise)
    put("A-03", ["NUM-03", "TIME-06", "LAW-05", "TIME-02", "TIME-03", "LAW-08"], "2022-12-31", "2023-01-31",
        res["enacted"], t, without=res["no-cut"],
        relations=["net + advance + contributions + plan = gross, each payment"])


# --------------------------------------------------------------------------- A-04 child benefit

def q04():
    t = Trace()
    birth = a.ZOSIA_BORN
    end3 = date(2023, 7, 27)
    t("3 months from the birth (GEN-3 .1)", end3, "27 April 2023 + 3 months")
    first_month = (D("400.00") * 4 / 30).quantize(D("0.01"), rounding="ROUND_DOWN")
    t("April 2023, the birth month (FAM-1 v1 .4)", first_month, "400.00 × 4 days (27–30 April) / 30, rounded down")
    in_time = {"2023-04": first_month, "2023-05": D("400.00"), "2023-06": ZERO, "2023-07": D("400.00"), "2023-08": D("400.00")}
    late = {"2023-04": ZERO, "2023-05": ZERO, "2023-06": ZERO, "2023-07": D("400.00"), "2023-08": D("400.00")}
    t("June 2023: the 2023/24 claim was filed after 30 April (FAM-1 .2), so from July", ZERO)
    t("if the claim counts as filed when posted, 2023-07-27 (A-D16): in time", sum(in_time.values()))
    t("if it counts when received, 2023-07-31: late; the 2022/23 period ended in May, so nothing for it", sum(late.values()))
    check(list(in_time) == list(late), "A-04 both readings cover the same months")

    def bc(x, y):  # README § Keys: a by-case stands in place of the smallest value it concerns; settled months stay plain
        return x if x == y else {"kind": "by-case", "cases": {"filed when posted (27 July 2023)": x, "filed when received (31 July 2023)": y},
                                 "settled by": "the FBO's decision (it arrived 2023-09-14)"}
    key = {"months": {m: bc(in_time[m], late[m]) for m in in_time}, "total": bc(sum(in_time.values()), sum(late.values()))}
    put("A-04", ["TIME-05", "TIME-04", "NUM-02", "LIFE-04", "ASK-03"], "2023-08-31", "2023-08-31", key, t)


# --------------------------------------------------------------------------- A-05 capital tax, A-06 gift tax

def by_value(lines, value):
    """A sale's proceeds are the SELL row's Value (ort of units × the day's price, as the export writes it), not units × the
    export's 2-place display price: shared over the lots by units, to the ort, the last lot taking the rest."""
    units = sum((l["units"] for l in lines), ZERO)
    left = value
    for i, l in enumerate(lines):
        p = left if i == len(lines) - 1 else ort(value * l["units"] / units)
        left -= p
        l["proceeds"], l["gain"] = p, p - l["cost"]
    return lines


def sales_from_records(*, known):
    """Each person's sales with their lots, from the Rzeka exports (Tomek: BUY lots; Ania: the gifted lots, cost by PIT-8 .2)."""
    sales = []
    tr = rzeka_rows("rzk-4471-02-tomasz.csv")
    lots = []
    for r in tr:
        if r["op"] == "BUY":
            lots.append({"id": r["src"], "fund": r["fund"], "date": r["date"], "units": r["units"], "cost": r["value"] + r["fee"]})
        elif r["op"] == "SELL":
            need, taken = r["units"], []
            for l in sorted([l for l in lots if l["fund"] == r["fund"] and l["units"] > 0], key=lambda l: (l["date"], l["id"])):
                u = min(need, l["units"])
                taken.append((dict(l), u))
                l["units"] -= u
                need -= u
                if need == 0:
                    break
            sales.append({"person": "T", "day": r["date"], "fund": r["fund"], "price": r["price"], "src": r["src"],
                          "lines": by_value(a.gain_lines(taken, r["date"], r["price"]), r["value"])})
    ar = rzeka_rows("rzk-4471-07-anna.csv")
    gpx, gday = price_on("BRZ-GLB-A", a.GIFT_DAY, known)
    glots = []
    for r in ar:
        if r["op"] == "TRANSFER IN":
            month = r["note"].split("acquired ")[1]
            glots.append({"id": f"gift lot {month}", "fund": r["fund"], "date": a.GIFT_DAY, "units": r["units"],
                          "cost": ort(r["units"] * gpx), "month": month, "giver_cost": r["value"]})
        elif r["op"] == "SELL":
            taken = [(dict(l), l["units"]) for l in glots]
            sales.append({"person": "A", "day": r["date"], "fund": r["fund"], "price": r["price"], "src": r["src"], "taken": taken,
                          "lines": [dict(x, since=date(*map(int, l["month"].split("-")), 1)) for x, (l, _) in zip(by_value(a.gain_lines(taken, r["date"], r["price"], giver_day=a.GIFT_DAY), r["value"]), taken)]})
    return sales, gpx, gday


def q05():
    t = Trace()
    sales, gpx, gday = sales_from_records(known="2025-04-30")  # R1: A-05 known as of 2025-04-30
    for s in sales:
        t(f"sale {s['day']} {s['fund']} ({s['src']})", sum((l["gain"] for l in s["lines"]), ZERO),
          f"{len(s['lines'])} lots first in, first out; gain = units × {s['price']} − lot cost")
    tom = a.capital_tax(sales, "T")
    key = {"Tomek": {}}
    for y in (2022, 2023, 2024):
        r = tom[y]
        t(f"Tomek {y} (PIT-8 .4, .5)", {"net": r["net"], "loss used": r["used"], "base": r["base"], "tax": r["tax"]})
        key["Tomek"][str(y)] = {"tax": r["tax"], "loss used": r["used"]}
    paid = {r["title"].split()[-2]: -r["amount"] for r in joint_rows() if r["title"].startswith("PIT capital gains 2024")}
    t("paid on 2025-04-28 for 2024 (wrz-joint-2025-04)", paid)
    key["Tomek"]["2024"]["paid too much by"] = paid["T."] - tom[2024]["tax"]
    key["Tomek"]["2022"]["loss carried"] = -tom[2022]["net"]
    key["Tomek"]["loss left after 2024"] = sum((x for _, x in tom[2024]["losses_left"]), ZERO)
    # Ania: the gifted lots' holding counts from Halina's purchase (PIT-8 .6), known only by month (A-D18)
    s = [x for x in sales if x["person"] == "A"][0]
    t("gifted lots' cost: units × the price on the gift day (PIT-8 .2)", gpx, f"BRZ-GLB-A on {gday}")
    sale = s["day"]
    cases = {}
    for label, g2_exempt in (("bought 1–6 March 2019", True), ("bought 7–31 March 2019", False)):
        net = ZERO
        for (lot, u), ln in zip(s["taken"], s["lines"]):
            y, m = map(int, lot["month"].split("-"))
            if lot["month"] == "2019-03":
                exempt = g2_exempt
            else:
                last_day = date(y, m, 28) + timedelta(days=4)
                last_day = last_day - timedelta(days=last_day.day)
                exempt = sale > a._add_years(last_day, 5)  # held more than 5 years even if bought on the month's last day
                if not exempt:
                    check(not sale > a._add_years(date(y, m, 1), 5), "A-05 a gifted lot's test settles by month")
            if not exempt:
                net += ln["gain"]
        base = duk(net)
        cases[f"the March 2019 lot {label}"] = {"tax": duk(base * D("0.19")), "net gain": net, "paid too much by": paid["A."] - duk(base * D("0.19"))}
        t(f"Ania 2024 if the March 2019 lot was {label}", cases[f"the March 2019 lot {label}"],
          f"sold {sale}; exempt if held more than 5 years since Halina's purchase (GEN-3 .1)")
    st = a.broker_statement_2024(type("B", (), {"sales": [dict(s, person="A")]})())
    t("the broker's statement (A-D22)", st, "cost from the transfer form (Halina's prices, A-D21), no holding test")
    w = [r for r in wrapped_rows() if r["type"] == "withdrawal"][0]
    contrib = sum((r["amount"] for r in wrapped_rows() if r["type"] == "contribution" and r["date"] < w["date"]), ZERO)
    t("wrapped withdrawal 2024 (PIT-9 .4): contributions not yet withdrawn come first", {"withdrawn": -w["amount"], "contributions": contrib, "earnings taken": ZERO, "tax": ZERO})
    key["Ania"] = {"2024": {"kind": "by-case", "cases": cases, "missing": ["the day in March 2019 Halina bought the 60 units"]},
                   "wrapped withdrawal tax 2024": ZERO,
                   "why Ania's payment differs": ["cost taken from the giver's purchases, not the gift-day price (PIT-8 .2)",
                                                                                   "no 5-year test for gifted units held since the giver's purchase (PIT-8 .6)"]}
    check(st["tax"] == paid["A."], "A-05 Ania paid the broker's figure")
    check(len({v["tax"] for v in cases.values()}) == 2, "A-05 the two cases differ")
    put("A-05", ["OWN-06", "OWN-07", "LAW-09", "NUM-07", "VAL-04", "TIME-01", "OWN-04", "OWN-08", "VAL-05", "LAW-12"],
        "2024-12-31", "2025-04-30", key, t)


def q06():
    t = Trace()
    g = a.gift_tax(known="2024-04-30")  # R1: A-06 known as of 2024-04-30
    t("the units' value on the gift day (IGT-2 .2)", g["value"], f"200 × {g['price']} ({g['price_day']})")
    t("added to the 2021 cash gift from the same person within 5 years (IGT-3 .1; A-D19)", g["base"], "35 280.00 + 15 000.00, rounded (IGT-2 .3)")
    t("the cash gift was never exempt: not credited to a payment account (IGT-4 .4)", "taxable")
    t("report needed: the added-up sum is above IGT-FREE-I (IGT-4 .3)", g["free"], g["free_src"])
    t("report deadline: 6 months from 2023-09-12 (IGT-4 .2, GEN-3 .1)", g["deadline"])
    t("reported 2024-03-14 (A-D20): late, so taxed as Group I", g["over"], f"{g['base']} − {g['free']}")
    t("tax: 3 % up to 12 000.00 (IGT-3 .3), rounded", g["tax"])
    put("A-06", ["LAW-11", "LAW-12", "OWN-05", "TIME-05"], "2024-04-30", "2024-04-30",
        {"gift tax": g["tax"]}, t)


# --------------------------------------------------------------------------- A-07, A-08 the car

def q07():
    t = Trace()
    c = a.car_settlement()
    t("sale price (A-D27)", a.CAR_SALE["price"])
    t("payoff on 2024-08-22 (A-D28)", c["payoff"], f"balance {c['balance']} + interest {c['interest']} ({c['days']} days, A-D24 .6) + fee {c['fee']} (0.5 %, CC-2 .2; below the {c['left_interest']} of interest left)")
    t("insurance refund (A-D25 §9, MOT-1 .3)", c["oc_refund"], f"1 104.00 × {c['unused']} / {c['pol_days']} days, down, − 20.00")
    t("road tax refund (MOT-2 .3)", c["road_refund"], "720.00 / 12 × 4 whole months after August")
    t("commission refund, reading T: by time (CC-1 .2)", c["refund_time"], "2 700.00 × 9 / 48")
    t("commission refund, reading B: by the falling balance", c["refund_balance"], "2 700.00 × Σ balances owed in months 40–48 / Σ in months 1–48")
    base = a.CAR_SALE["price"] - c["payoff"] + c["oc_refund"] + c["road_refund"]
    key = {"kind": "by-case", "cases": {"T: the commission refunded by time": base + c["refund_time"],
                                        "B: the commission refunded by the falling balance": base + c["refund_balance"]},
           "settled by": "the bank's settlement, due within 14 days of 2024-08-22 (CC-1 .3)"}
    put("A-07", ["LIFE-02", "NUM-02", "NUM-04", "FLOW-02", "LIFE-03"], "2024-12-31", "2024-08-25", key, t,
        relations=["the legs add up: price − payoff + refunds"])


def q08():
    t = Trace()
    c = a.car_settlement()
    sold = a.CAR_SALE["price"] - c["payoff"] + c["refund_time"] + c["oc_refund"] + c["road_refund"]
    t("sold: what came in and went out because of the sale, by 2025-05-14 (records)", sold, "41 500.00 − payoff + commission 506.25 (paid 2024-09-03) + 13.18 + 240.00")
    inst = sum((r["instalment"] for r in c["schedule"] if r["month"] >= 40), ZERO)
    t("kept: instalments 40–48, 2024-09-14 .. 2025-05-14 (A-D24)", inst)
    t("kept: the renewed insurance on 2024-09-01 (A-D26)", a.OC["renewal_premium"])
    t("kept: road tax 2025 (MOT-2 .1–.2)", a.ROAD_TAX)
    garage = D("180.00") * 9
    t("kept: the garage, September 2024 – May 2025 (A-D29)", garage, "9 × 180.00")
    kept = -(inst + a.OC["renewal_premium"] + a.ROAD_TAX + garage)
    diff = kept - sold
    t("kept less sold", diff)
    put("A-08", ["SCEN-02", "LIFE-02", "FLOW-02"], "2025-05-14", "2025-05-14",
        {"less by": -diff}, t)


# --------------------------------------------------------------------------- A-09, A-10 the 15th

RULE_TO = ("Brzask TFI – wrapped account A. Lis", "Rzeka Securities – client 4471-02")


def rule_on(rows, wrap, d, order=None, *, known):
    day = [r for r in rows if r["date"] == d]
    if order == "reversed":
        day = list(reversed(day))
    before = [r for r in rows if r["date"] < d]
    moved = -sum((r["amount"] for r in day if r["type"] == "OUTGOING TRANSFER" and r["counterparty"] in RULE_TO), ZERO)
    flows = sum((r["amount"] for r in day), ZERO)
    end_balance = before[-1]["balance"] + flows  # the order of the day's rows doesn't matter (A-HR1 v2 .1)
    b = end_balance + moved
    nxt = a.rule_day(*a.next_month(d.year, d.month))
    due = a.standing_due(d, nxt)
    avg, ms = a.card_average(rows, d)
    floor = D("1000.00") + sum((x[2] for x in due), ZERO) + avg
    rule = max(ZERO, b - floor)
    lim, _ = figure("WRAP-LIMIT", d, known)
    used = sum((r["amount"] for r in wrap if r["type"] == "contribution" and r["date"].year == d.year and r["date"] < d), ZERO)
    w = max(ZERO, min(rule, lim - used))
    return {"balance": b, "due": sum((x[2] for x in due), ZERO), "card": avg, "floor": floor, "rule": rule,
            "to wrapped": w, "to Rzeka": rule - w, "moved": moved, "gap": moved - rule}


def q09():
    t = Trace()
    rows, wrap = joint_rows(), wrapped_rows()
    tab = {}
    for m in range(1, 13):
        d = a.rule_day(2025, m)
        r = rule_on(rows, wrap, d, known="2026-01-15")  # R1: A-09 known as of 2026-01-15
        r2 = rule_on(rows, wrap, d, order="reversed", known="2026-01-15")
        check(r == r2, f"A-09 order within {d} changes nothing")
        t(f"{d}", r, "floor = 1 000.00 + known payments before the next 15th (A-D33) + card average of the 3 months before (A-HR1 v2 .3)")
        tab[d.isoformat()] = {"rule": r["rule"], "to wrapped": r["to wrapped"], "to Rzeka": r["to Rzeka"], "moved": r["moved"], "gap": r["gap"]}
    # REC-03: the own transfer to Bank Lazur moves no household total
    laz = [r for r in rows if r["counterparty"].startswith("Bank Lazur")]
    for r in laz:
        t(f"own transfer {r['date']} ({r['src']})", r["amount"], "out of the joint account, into the Lazur savings: the household's total doesn't move")
    put("A-09", ["RULE-01", "RULE-02", "RULE-07", "RULE-08", "OWN-08", "VAL-02", "TIME-07", "TIME-02", "REC-03", "FLOW-01"],
        "2025-12-31", "2026-01-15", tab, t,
        relations=["reordering a day's rows changes nothing", "an own transfer moves no household total"])


def q10():
    t = Trace()
    t("A-HR1 v2: on the 15th, everything above the floor goes to investing", "claims all of the surplus")
    t("A-HR4: from January 2026, 500.00 DUK to Zosia's savings on the 15th", "claims 500.00 of the same money")
    t("neither rule says which comes first; the amounts depend on the order", "refused")
    t("the balance of 2026-01-15 needs the joint account's bookings of 1-15 January 2026", "the records end 2025-12-31 (story)")
    put("A-10", ["RULE-03", "ASK-03"], "2026-01-15", "2026-01-15",
        {"kind": "refusal", "missing": ["which comes first on the 15th: investing everything above the floor, or Zosia's 500.00",
                                        "the joint account's bookings from 2026-01-01 to 2026-01-15 (the records end 2025-12-31)"],
         "reason": "two rules want the same money and no priority is stated; and the day's balance they share out is not in the records"}, t)


# --------------------------------------------------------------------------- A-11 net worth

def holdings():
    """Units per account and fund at the end of 2025, and cash, from the records."""
    out = {}
    wr = wrapped_rows()
    u = {}
    cash = ZERO
    for r in wr:
        if r["type"] == "contribution":
            cash += r["amount"]
        elif r["type"] == "purchase":
            # R2: the account was opened empty (story, the places table: BRZ-WRAP-0091-A "opened, empty")
            u[r["fund"]] = u.get(r["fund"], ZERO) + r["units"]
            cash -= r["amount"]
        elif r["type"] == "redemption":
            u[r["fund"]] += r["units"]
            cash += r["amount"]
        elif r["type"] == "withdrawal":
            cash += r["amount"]
    out["wrapped"] = (u, cash, "brz-wrapped-ania.csv")
    for name, key in (("rzk-4471-02-tomasz.csv", "Rzeka Tomek"), ("rzk-4471-07-anna.csv", "Rzeka Ania")):
        rows = rzeka_rows(name)
        u = {}
        for r in rows:
            if r["op"] in ("BUY", "TRANSFER IN"):
                # R2: 4471-02 was opened empty (story, the places table), 4471-07 is new on 2024-02-12 (story, A-D21)
                u[r["fund"]] = u.get(r["fund"], ZERO) + r["units"]
            elif r["op"] == "SELL":
                u[r["fund"]] -= r["units"]
        out[key] = ({k: v for k, v in u.items() if v}, rows[-1]["cash"], name)
    return out


def q11(law="enacted"):
    K = "2026-01-15"  # R1: A-11 known as of 2026-01-15
    t = Trace()
    d = date(2025, 12, 31)
    rows = joint_rows()
    comm = []
    joint = [r for r in rows if r["date"] <= d][-1]
    comm.append(("joint account", joint["balance"], joint["src"]))
    lz = (REC / "laz-savings-1020-0044-7310.csv").read_text(encoding="utf-8").splitlines()
    laz_bal = D(lz[-1].split(",")[3])
    laz_in = -sum((r["amount"] for r in rows if r["counterparty"].startswith("Bank Lazur") and r["date"] > date(2025, 6, 30)), ZERO)
    comm.append(("Bank Lazur savings: balance 2025-06-30 + deposit 2025-10-06", laz_bal + laz_in, "laz-savings last line; wrz-joint-2025-10"))
    h = holdings()
    for acct in ("wrapped", "Rzeka Tomek"):
        units, cash, src = h[acct]
        v = cash
        for f, u in units.items():
            px, pd_ = price_on(f, d, K)
            v += ort(u * px)
            t(f"{acct}: {u} {f} × {px} ({pd_})", ort(u * px), src)
        comm.append((acct, v, src))
    for who, name in (("T", "wek-wcp-lis-tomasz.csv"), ("A", "wek-wcp-lis-anna.csv")):
        u, src = wcp_units(name, d)
        px, _ = price_on("WEK-WCP-2055", d, K)
        comm.append((f"workplace plan {who}", ort(u * px), f"{src}; units bought with the contributions and the state's payments (WCP-1 .4, WCP-3)"))
    comm.append(("rent deposit (A-D30)", D("6400.00"), "A-D30"))
    rows_t, _ = a.payroll(2026, law, known=K)["NURT"] if False else (None, None)
    pays = [p for p in a.tomek_payments() if p["kind"] == "pay" and p["date"] == a.t_payday(2025, 12)]
    dec = run_payer([p for p in a.tomek_payments() if p["date"].year == 2026] or pays, law=law, known=K)[0][0]
    comm.append(("Tomek's December 2025 pay, net, paid 2026-01-15 (A-HR5)", dec["net"], "A-D02-2025, A-D04; PIT-4, SOC-1, SOC-2, WCP-1 .2, WCP-2 .1, .2"))
    ret = a.household_return(2025, law, known=K)["joint"]
    comm.append(("the 2025 joint return, refund (+) or payable (−), next spring (A-HR5)", -(ret["due_each"][0] + ret["due_each"][1]), "PIT-6"))
    # Ania's savings: the estate payout is hers alone (CIV-1 .3); the interest on it is community (CIV-1 .2)
    sav = (REC / "wrz-ania-savings-2025.csv").read_text(encoding="utf-8").splitlines()
    sav_bal = dc(sav[-1].split(";")[5])
    interest = sav_bal - a.ESTATE_PAYOUT[1]
    comm.append(("interest on Ania's savings, net (community, CIV-1 .2)", interest, "wrz-ania-savings-2025.csv"))
    for name, v, src in comm:
        t(f"community: {name}", v, src)
    C = sum((v for _, v, _ in comm), ZERO)
    t("community property, total", C)
    units, cash, src = h["Rzeka Ania"]
    sep = cash
    for f, u in units.items():
        px, _ = price_on(f, d, K)
        sep += ort(u * px)
    t(f"Ania alone: her Rzeka account, the units bought with the gifted units' proceeds plus the account's cash ({cash}) (CIV-1 .3)", sep, src)
    t("Ania alone: the estate payout (CIV-1 .3)", a.ESTATE_PAYOUT[1], "A-D36")
    half_cents = int(C * 100)
    ania_half = D(half_cents // 2) / 100
    tomek_half = C - ania_half  # A-HR5: the odd ort to Tomek
    ania = ania_half + sep + a.ESTATE_PAYOUT[1]
    t("each person's share (A-HR5)", {"Ania": ania, "Tomek": tomek_half})
    check(ania + tomek_half == C + sep + a.ESTATE_PAYOUT[1], "A-11 the people add up to the household")
    return {"kind": "partial", "value": {"Ania": ania, "Tomek": tomek_half, "household": ania + tomek_half},
            "missing": ["the Bank Lazur savings rate from 2025-07-01, so the interest credited 2025-09-30 and 2025-12-31"]}, t


def q11_put():
    key, t = q11()
    without, _ = q11("no-cut")
    put("A-11", ["ASK-02", "OWN-04", "OWN-03", "NUM-05", "OWN-01", "ASK-03"], "2025-12-31", "2026-01-15", key, t,
        without=without["value"], relations=["the people's net worths add up to the household's"])


# --------------------------------------------------------------------------- A-12 the forecast

PATHS = {"base": lambda ym: D("0.004"),
         "crash": lambda ym: D("-0.5") if ym == (2027, 3) else D("0.004"),
         "flat": lambda ym: ZERO,
         "switch": lambda ym: D("0.004") if ym < (2030, 1) else D("-0.001")}
PLACES = {"BRZ-GLB-A": 2, "BRZ-BND-A": 2, "BRZ-DIV-D": 2, "BRZ-MM-A": 2, "WEK-EM-A": 4, "WEK-IDX-D": 4}


def forecast(path, whatif, mode):
    """The household's forecast (story, 2026-01-15): monthly prices by the path, investing, the car (A-HR3), the sale at the end."""
    K = "2026-01-15"  # R1: A-12 known as of 2026-01-15
    start = date(2025, 12, 31)
    px = {f: price_on(f, start, K)[0] for f in PLACES}
    h = holdings()
    # lots at the start, from the records
    lots = []
    for r in wrapped_rows():
        if r["type"] == "purchase":
            lots.append({"acct": "W", "fund": r["fund"], "date": r["date"], "units": r["units"], "cost": r["amount"]})
        elif r["type"] == "redemption":
            need = -r["units"]
            for l in sorted([l for l in lots if l["acct"] == "W" and l["fund"] == r["fund"] and l["units"] > 0], key=lambda l: l["date"]):
                u = min(need, l["units"])
                l["cost"] -= l["cost"] * u / l["units"]
                l["units"] -= u
                need -= u
                if need == 0:
                    break
    for l in lots_rzeka(rzeka_rows("rzk-4471-02-tomasz.csv")):
        lots.append(dict(l, acct="T"))
    for r in rzeka_rows("rzk-4471-07-anna.csv"):
        if r["op"] == "BUY":
            lots.append({"acct": "A", "fund": r["fund"], "date": r["date"], "units": r["units"], "cost": r["value"] + r["fee"]})
    cash = {"W": h["wrapped"][1], "T": h["Rzeka Tomek"][1], "A": h["Rzeka Ania"][1]}
    wr = wrapped_rows()
    contrib_left = sum((r["amount"] for r in wr if r["type"] == "contribution"), ZERO) + sum((r["amount"] for r in wr if r["type"] == "withdrawal"), ZERO)
    invested = fees = ZERO
    car = None
    car_tax_lines = []
    y, m = 2026, 1
    while (y, m) <= (2035, 12):
        if (y, m) >= (2026, 2):
            d = a.rule_day(y, m)
            amount = D("3000.00") + D("100.00") * (y - 2026)
            limit = figure("WRAP-LIMIT", d, K)[0] if y == 2026 else D("27300.00") + D("600.00") * (y - 2026)
            used = getattr(forecast, "_used", {}).get((path, whatif, mode, y), ZERO)
            w = max(ZERO, min(amount, limit - used))
            forecast.__dict__.setdefault("_used", {})[(path, whatif, mode, y)] = used + w
            invested += amount
            if w > 0:
                contrib_left += w
                cash["W"] += w
                for f, part in zip(("BRZ-GLB-A", "BRZ-BND-A"), a.split(w, [1, 1])):
                    u = units_down(part / px[f])
                    paid = ort(u * px[f])
                    cash["W"] -= paid
                    lots.append({"acct": "W", "fund": f, "date": d, "units": u, "cost": paid})
            b = amount - w
            if b > 0:
                fee = max(a.FEE_MIN, ort(b * a.FEE_RATE))
                u = units_down((b - fee) / px["BRZ-GLB-A"])
                paid = ort(u * px["BRZ-GLB-A"])
                cash["T"] += b - fee - paid
                fees += fee
                lots.append({"acct": "T", "fund": "BRZ-GLB-A", "date": d, "units": u, "cost": paid + fee})
            if car is None:
                worth = cash["T"] + sum((ort(l["units"] * px[l["fund"]]) for l in lots if l["acct"] == "T" and l["units"] > 0), ZERO)
                glb = sorted([l for l in lots if l["acct"] == "T" and l["fund"] == "BRZ-GLB-A" and l["units"] > 0], key=lambda l: l["date"])
                glb_worth = ort(sum((l["units"] for l in glb), ZERO) * px["BRZ-GLB-A"])  # A-HR3: its BRZ-GLB-A's worth, to the ort
                if worth >= D("60000.00") and glb_worth >= D("45000.00"):
                    need = (D("45000.00") / px["BRZ-GLB-A"]).quantize(D("0.0001"), rounding=ROUND_UP)
                    check(need <= sum((l["units"] for l in glb), ZERO), "A-12 the car's sale sells only units the account holds")
                    proceeds = ort(need * px["BRZ-GLB-A"])
                    cash["T"] += proceeds - D("45000.00")  # A-HR3: what the sale brings above 45 000.00 stays as cash
                    for l in glb:
                        u = min(need, l["units"])
                        c = l["cost"] if u == l["units"] else l["cost"] * u / l["units"]
                        car_tax_lines.append({"year": d.year, "gain": ort(u * px["BRZ-GLB-A"]) - c, "since": l["date"], "day": d})
                        l["cost"] -= c
                        l["units"] -= u
                        need -= u
                        if need == 0:
                            break
                    car = {"date": d, "proceeds": proceeds, "worth": worth}
        # the month's last day: prices move by the path
        g = PATHS[path]((y, m))
        for f in px:
            px[f] = (px[f] * (1 + g)).quantize(Decimal(1).scaleb(-PLACES[f]), rounding=mode)
        y, m = a.next_month(y, m)
    end = date(2035, 12, 31)
    # selling everything on 2035-12-31 (item 5): each account's holding of a fund is one sale, its proceeds the ort of
    # units × the day's price, shared over the lots by units, as the records' sales are (by_value)
    held = {}
    for l in lots:
        if l["units"] > 0:
            held.setdefault((l["acct"], l["fund"]), []).append(l)
    sold = {k: ort(sum((l["units"] for l in ls), ZERO) * px[k[1]]) for k, ls in held.items()}
    value = sum(cash.values(), ZERO) + sum(sold.values(), ZERO)
    wrap_value = cash["W"] + sum((v for k, v in sold.items() if k[0] == "W"), ZERO)
    earnings = max(ZERO, wrap_value - contrib_left)
    wrap_tax = duk(earnings * D("0.19"))  # PIT-9 .4, .5 (Ania is 44)
    rate = (lambda yy: D("0.25") if (whatif and yy >= 2030) else D("0.19"))
    sales = []
    for acct, person in (("T", "T"), ("A", "A")):
        lines = []
        for k, ls in held.items():
            if k[0] == acct:
                lines += by_value([{"lot": f"{l['fund']} {l['date']}", "units": l["units"], "cost": l["cost"], "since": l["date"]} for l in ls], sold[k])
        sales.append({"person": person, "day": end, "lines": lines})
    if car:
        sales.append({"person": "T", "day": car["date"], "lines": [{"lot": "car", "units": 0, "proceeds": 0, "cost": 0, "gain": x["gain"], "since": x["since"]} for x in car_tax_lines]})
    hist, _, _ = sales_from_records(known=K)
    tt = a.capital_tax(hist + sales, "T", years=range(2021, 2036), rate=rate)
    ta = a.capital_tax(hist + sales, "A", exempt_lots=(), years=range(2021, 2036), rate=rate)
    gains_tax = tt[2035]["tax"] + ta[2035]["tax"]
    after = value - wrap_tax - gains_tax
    deflator = Fraction(10025, 10000) ** 119
    q = Fraction(after) / deflator * 100
    today = D((q.numerator * 2 + q.denominator) // (2 * q.denominator)) / 100  # half up, exact (GEN-5)
    return {"car": car["date"].isoformat() if car else None, "value": value, "wrap tax": wrap_tax, "gains tax": gains_tax,
            "after tax, January-2026 money": today, "_invested": invested, "_fees": fees, "_car": car, "_cash": cash, "_px": px, "_lots": lots}


MODES = {"half up": "ROUND_HALF_UP", "down": "ROUND_DOWN", "up": "ROUND_UP"}  # item 3 names no mode: each one the papers name (GEN-5; A-D08 .4, A-HR3)


def spring_moves():
    """GEN-1 .3: no notice for 2027-2035, so which Monday of April is Spring Day is not settled. Under each, the 15th's
    investing day (A-HR2) stays before April's last day, where the price moves, and on the same side of every 5-year line."""
    end = date(2035, 12, 31)
    for y in range(2027, 2036):
        mondays = [date(y, 4, d) for d in range(1, 31) if date(y, 4, d).weekday() == 0]
        days = {a.rule_day(y, 4)}   # the day the forecast runs on, its calendar holding no Spring Day after 2026
        for sd in mondays:
            d = date(y, 4, 15)
            while d.weekday() >= 5 or CAL.is_holiday(d) or d == sd:
                d += timedelta(days=1)
            days.add(d)
        check(all(d.month == 4 and d < date(y, 4, 30) for d in days), f"A-12 Spring Day {y}: the investing day stays before the price moves")
        check(len({end > a._add_years(d, 5) for d in days}) == 1, f"A-12 Spring Day {y}: no lot crosses a 5-year line")


def q12():
    t = Trace()
    key = {}
    spring_moves()
    t("Spring Day 2027-2035", "the same figures under each", "GEN-1 .3 leaves each year's Monday unsettled; under each, April's investing day "
      "moves within the days before the month's last day, at the same price, and no lot crosses PIT-8 .6's 5 years")
    for whatif in (False, True):
        for path in PATHS:
            label = f"{path}{', gains tax 25 % from 2030' if whatif else ''}"
            runs = {}
            for name, mode in MODES.items():
                r = forecast(path, whatif, mode)
                runs[name] = {k: v for k, v in r.items() if not k.startswith("_")}
                t(f"{label}, prices rounded {name}", runs[name],
                  f"each month's last day: price × (1 + the path's rate), rounded {name} to the fund's places (item 3); "
                  "on 2035-12-31 each account's holding of a fund sold as one sale, the ort of units × price (item 5); "
                  "tax on selling all then (PIT-8, PIT-9); ÷ 1.0025^119")
            # README § Keys, Placement: a figure the rounding moves is a by-case; one it doesn't stays plain
            key[label] = {k: (runs["half up"][k] if len({str(x[k]) for x in runs.values()}) == 1
                              else {"kind": "by-case", "cases": {n: x[k] for n, x in runs.items()}})
                          for k in runs["half up"]}
            if path == "flat":
                check(all(not isinstance(v, dict) for v in key[label].values()), "A-12 the flat path keeps every price, under each rounding")
    for path in PATHS:
        b, w = key[path], key[f"{path}, gains tax 25 % from 2030"]
        check(b["car"] == w["car"] and b["value"] == w["value"], f"A-12 the what-if changes nothing before 2030 ({path})")
    put("A-12", ["LIFE-01", "NUM-06", "VAL-06", "SCEN-04", "LAW-13", "RULE-05"], "2035-12-31", "2026-01-15", key, t,
        relations=["the 2030 what-if changes nothing before 2030", "the flat path keeps every price"])


# --------------------------------------------------------------------------- A-13: had Ania stopped working (a scenario)

A13_SALE = date(2023, 12, 15)   # (b): the gifted units sold at DM Kopia that day, at that day's price, no fee (stated in the question)


def key_of(qid):
    """An earlier key of this run, read back from its file (keys/A/<qid>.md)."""
    import json
    for ln in (ROOT / "keys" / "A" / f"{qid}.md").read_text(encoding="utf-8").splitlines():
        if ln.startswith("key: "):
            return json.loads(ln[5:], parse_float=Decimal)


def a13_return(stop, *, known):
    """2023's joint return (PIT-6, PIT-7), had Ania stopped working for Kalina from `stop` (the 1st of a month; None: as it
    was). Kalina pays nothing for any month from then, and no plan contribution on it; pay for an earlier month and the
    contribution on it (owed by WCP-1 .4) stay. Parental pay comes only to a parent insured as an employee on the birth
    day (SOC-8 .1). Everything else as a.household_return(2023)."""
    from fam.home import child_relief
    assert stop is None or stop.day == 1
    cut = f"{stop.year}-{stop.month:02d}" if stop else None
    kal_all = [p for p in a.ania_payments() if p["date"].year == 2023]
    kal = [p for p in kal_all if cut is None or p["for"].split()[-1] < cut]
    insured = stop is None or stop > a.ZOSIA_BORN
    sio_all = [p for p in a.parental()["months"] if p["date"].year == 2023]
    sio = sio_all if insured else []
    pr = {"NURT": run_payer([p for p in a.tomek_payments() if p["date"].year == 2023], reducing=True, known=known)}
    if kal:
        pr["KALINA"] = run_payer(kal, reducing=True, known=known)
    if sio:
        pr["SIO"] = run_payer(sio, reducing=False, known=known)
    t_ = [pr["NURT"][1]]
    a_ = [pr[k][1] for k in ("KALINA", "SIO") if k in pr]
    pinc = duk(sum((x["base"] for x in t_), ZERO)) + duk(sum((x["base"] for x in a_), ZERO))  # PIT-7 .2
    cr = child_relief(2023, 9, pinc)
    j = joint_return(2023, a_, t_, reliefs=(cr["half"], cr["half"]))
    return {"j": j, "relief": cr["relief"], "pinc": pinc, "income": sum((x["income"] for x in a_), ZERO), "base": j["bases"][0],
            "advances": j["advances"][0], "dropped": [p for p in kal_all if p not in kal], "sio_dropped": [] if insured else sio_all,
            "kal": kal, "insured": insured}


def a13_lines(r):
    j = r["j"]
    return {"joint tax": j["joint_tax"], "Ania": j["due_each"][0], "Tomek": j["due_each"][1], "together": j["due_each"][0] + j["due_each"][1],
            "child relief": r["relief"], "Ania's income": r["income"], "Ania's base": r["base"], "parents' income": r["pinc"],
            "Ania's advances": r["advances"]}


def a13_part(label, stop, base, t, *, known):
    r = a13_return(stop, known=known)
    j, here, ours = r["j"], a13_lines(r), a13_lines(base)
    moved = {k: here[k] - ours[k] for k in here}
    for p in r["dropped"]:
        t(f"({label}) dropped: Kalina's {p['for']}, paid {p['date']}", p["gross"], f"for a month from {stop} (the stop)")
    if not r["dropped"]:
        last = max(p["date"] for p in r["kal"])
        t(f"({label}) Kalina's payments of 2023 on or after the stop", [], f"none: the last is on {last}; parental leave from {a.ZOSIA_BORN} (A-D06-2023b)")
    if r["sio_dropped"]:
        t(f"({label}) the SIO's parental pay of 2023", sum((p["gross"] for p in r["sio_dropped"]), ZERO),
          f"dropped: on the birth day, {a.ZOSIA_BORN}, she is no longer insured as an employee (SOC-8 .1)")
    else:
        t(f"({label}) the SIO's parental pay of 2023 stays", sum((p["gross"] for p in a.parental()["months"] if p["date"].year == 2023), ZERO),
          f"insured as an employee on the birth day, {a.ZOSIA_BORN}: 52 weeks of it (SOC-8 .1, A-D15)")
    t(f"({label}) parents' income (PIT-7 .2)", r["pinc"], "Ania's base + Tomek's base")
    t(f"({label}) child relief (PIT-7)", r["relief"], f"125.00 × 9 months − the income above 112 000.00, never below 0; halves")
    t(f"({label}) joint tax (PIT-6 .3, PIT-3 {j['version']})", j["joint_tax"], f"combined base {j['combined']}, half {j['half']}")
    t(f"({label}) each spouse's part (PIT-6 .5) and amount: part − relief − advances", j["due_each"],
      f"parts {enc(j['parts'])} by bases {enc(j['bases'])}; reliefs {enc(j['reliefs'])}; advances {enc(j['advances'])}")
    t(f"({label}) each line less A-01's 2023 (what we pay plus, a refund minus)", moved)
    why = []
    if r["dropped"]:
        why.append("Kalina's pay for the months from the stop, and the plan contributions on it, are gone: "
                   + "; ".join(f"{p['for']} {p['gross']} (paid {p['date']})" for p in r["dropped"]))
    else:
        why.append(f"no Kalina payment of 2023 falls from the stop on: on parental leave from {a.ZOSIA_BORN}, she was paid nothing by Kalina after {max(p['date'] for p in r['kal'])}")
    if r["sio_dropped"]:
        why.append(f"no parental pay: on the birth day she isn't insured as an employee (SOC-8 .1); {sum((p['gross'] for p in r['sio_dropped']), ZERO)} of 2023 gone, and the SIO's advances on it")
    else:
        why.append("her parental pay runs on: it is hers for 52 weeks from the birth, being insured as an employee on the birth day (SOC-8 .1)")
    if moved["child relief"]:
        why.append(f"the parents' income falls to {r['pinc']}, so the child relief is cut by {max(ZERO, r['pinc'] - D('112000.00'))} instead of {max(ZERO, base['pinc'] - D('112000.00'))} (PIT-7 .3)")
    if moved["joint tax"]:
        why.append(f"the joint tax follows the combined base, {j['combined']} instead of {base['j']['combined']}; each part follows its spouse's base (PIT-6 .5), "
                   f"so Tomek's part moves by {j['parts'][1] - base['j']['parts'][1]} though his pay doesn't")
    if not any(moved.values()):
        why.append("so no line of the 2023 return moves")
    out = {"joint tax": j["joint_tax"], "Ania": dirn(j["due_each"][0]), "Tomek": dirn(j["due_each"][1]),
           "together": dirn(j["due_each"][0] + j["due_each"][1]), "child relief": r["relief"], "differs by": moved, "why": why}
    return out, moved


def kopia_lots(gpx):
    lines = (REC / "kop-30-118-552-ania.csv").read_text(encoding="utf-8").splitlines()
    out = []
    for i, ln in enumerate(lines[2:], 3):
        d, ev, fund, qty, note = ln.split("\t")
        if ev == "TRANSFER IN (gift)":
            u = D(qty)
            out.append({"id": f"gift lot {note.split('bought ')[1]}", "month": note.split("bought ")[1], "units": u, "cost": ort(u * gpx),
                        "src": f"kop-30-118-552-ania.csv line {i}"})
    return out


def q13():
    known = "2025-04-30"   # R1: A-13's known-as-of date
    t = Trace()
    base = a13_return(None, known=known)
    h = a.household_return(2023, known=known)
    check(base["j"] == h["joint"] and base["relief"] == h["relief"]["relief"], "A-13 with no stop, the scenario's return is A-01's 2023")
    a01 = key_of("A-01")["2023"]
    check(a01["joint tax"] == base["j"]["joint_tax"] and a01["child relief"] == base["relief"], "A-13 the base figures are A-01's key")
    t("A-01's 2023 return (as it was)", a13_lines(base), "the same payroll as A-01 (A-D13-NURT-2023, A-D13-KALINA-2023, A-D13-SIO-2023)")
    pa, ma = a13_part("a", date(2023, 7, 1), base, t, known=known)
    check(not any(ma.values()), "A-13 (a) moves nothing: Kalina paid nothing after June 2023 and the parental pay stays")
    pc, mc = a13_part("c", date(2023, 4, 1), base, t, known=known)
    check(sum((mc[k] for k in ("Ania", "Tomek")), ZERO) == mc["together"], "A-13 (c) the spouses' moves add up to the couple's")
    # (b): the gifted units sold at DM Kopia on 2023-12-15 (all 200: every lot of the account goes)
    gpx, gday = price_on("BRZ-GLB-A", a.GIFT_DAY, known)
    px, pday = price_on("BRZ-GLB-A", A13_SALE, known)
    lots = kopia_lots(gpx)
    t("(b) gifted lots' cost: units × the price on the gift day; acquired that day (PIT-8 .2)", {l["month"]: l["cost"] for l in lots}, f"BRZ-GLB-A {gpx} on {gday} (A-D18)")
    value = ort(sum((l["units"] for l in lots), ZERO) * px)
    taken = [dict(l) for l in lots]
    by_value(taken, value)
    t("(b) the sale: every unit of the account (PIT-8 .3: the lots of one account; all go)", value, f"{sum((l['units'] for l in lots), ZERO)} × {px} ({pday}), no fee; shared over the lots by units")
    net, out_lots = ZERO, []
    for l in taken:
        y, m = map(int, l["month"].split("-"))
        first = date(y, m, 1)
        last = date(y, m, 28) + timedelta(days=4)
        last = last - timedelta(days=last.day)
        held_all, held_none = A13_SALE > a._add_years(last, 5), not A13_SALE > a._add_years(first, 5)
        check(held_all or held_none, f"A-13 (b) the {l['month']} lot's 5-year test settles by its month alone")
        exempt = held_all and l["gain"] > 0
        if not exempt:
            net += l["gain"]
        t(f"(b) lot bought {l['month']} by Halina ({l['src']})", {"units": l["units"], "proceeds": l["proceeds"], "cost": l["cost"], "gain": l["gain"], "taxed": not exempt},
          f"held since Halina's purchase (PIT-8 .6): 5 years from any day of {l['month']} end {'before' if held_all else 'after'} {A13_SALE} (GEN-3 .1)")
        out_lots.append({"bought": l["month"], "units": l["units"], "gain": l["gain"], "taxed": not exempt})
    from fam.home import gains_tax
    base23, tax23 = gains_tax(net)
    t("(b) Ania 2023 (PIT-8 .4): the taxed lots' net gain, base and tax in whole dukats", {"net gain": net, "base": base23, "tax": tax23}, "no loss carried in")
    sales, _, _ = sales_from_records(known=known)
    story = {y: sum((sum((l["gain"] for l in s["lines"]), ZERO) for s in sales if s["person"] == "A" and s["day"].year == y), ZERO) for y in (2023, 2024)}
    check(story[2023] == 0 and not [s for s in sales if s["person"] == "A" and s["day"].year == 2023], "A-13 as things happened, Ania sold nothing in 2023")
    a05 = key_of("A-05")["Ania"]["2024"]
    t("(b) Ania 2024: no sale in 2024 (the units are gone in December 2023)", ZERO, "PIT-8 .4: no gain, no tax")
    moved24 = {"kind": "by-case", "cases": {k: ZERO - v["tax"] for k, v in a05["cases"].items()}, "missing": a05["missing"]}
    t("(b) 2024 less what she owes as things happened (A-05: the March 2019 lot's day is missing, A-D18)", moved24)
    key = {"(a)": pa, "(b)": {"lots": out_lots, "2023": {"tax": tax23, "differs by": tax23}, "2024": {"tax": ZERO, "differs by": moved24}}, "(c)": pc}
    put("A-13", ["SCEN-02", "SCEN-01", "LAW-05", "TIME-04", "PTY-02", "PTY-04", "LAW-07", "ASK-05", "OWN-06", "OWN-07", "TIME-05", "TIME-01"],
        "2024-12-31", known, key, t,
        relations=["with no stop, the return is A-01's 2023", "each spouse's part follows their own base (PIT-6 .5): the moves add up to the couple's"])


def main():
    for q in (q01, q02, q03, q04, q05, q06, q07, q08, q09, q10, q11_put, q12, q13):
        q()
    if FAILS:
        print("relations failed:", *FAILS, sep="\n  ")
        sys.exit(1)
    print(f"A: 13 keys written, relations pass")


if __name__ == "__main__":
    main()
