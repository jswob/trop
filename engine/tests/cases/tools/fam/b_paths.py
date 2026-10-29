"""Household B's forecast, 2026-01-01 .. 2028-12-31, as seen on 2026-01-02 (story § The forecast).

One seeded path per index: the law draw first, then each month's draws in a fixed order. Every
path runs twice on the same draws (common random numbers): with B-HR2's overpayments and without.
"""
import math
from datetime import date, timedelta
from decimal import Decimal

from fam import b
from fam.home import ALL_KNOWN, CAL, ZERO, D, duk, ort
from lib.num import annuity
from lib.paths import rng

SEED = 20260102
START, END = date(2026, 1, 1), date(2028, 12, 31)
ARNIKA = D("16000.00")                    # story F1
DELAY_A = (10, 40)                         # F1: days after issue, uniform
PROJ_RATE, PROJ_MU, PROJ_SIGMA = 0.5, math.log(8000), 0.5   # F2
DELAY_P = (14, 60)                         # F2
LAW_P, LAW_RATE, LAW_FROM = 0.30, D("0.15"), (2027, 1)      # F7
LIVING = D("2375.00")                      # F3: 9 500.00 a month in four parts
KITCHEN = (date(2026, 9, 15), D("180000.00"))  # F4: the building plot
FLR_CONVERT = (date(2026, 1, 20), D("4.15"))  # F5
AWP = {2027: D("9900.00"), 2028: D("10500.00")}            # F6 (2026 from the gazette)
AWQ4 = {(2026, 2): D("9200.00"), (2027, 2): D("9800.00"), (2028, 2): D("10400.00")}  # F6 (January 2026 from the gazette)
FLOOR_LIQUID = D("60000.00")              # B-HR3
SHARE_LIMIT = D("0.20")                    # B-HR3: more than 1 path in 5


def poisson(r, lam):
    L, k, p = math.exp(-lam), 0, 1.0
    while True:
        p *= r.random()
        if p <= L:
            return k
        k += 1


def awq4(y, m, *, known):
    if (y, m) >= min(AWQ4):  # story F6: the forecast's own figures from February 2026, never a row published later
        return b.awq4(y, m, AWQ4, known=known)
    return b.awq4(y, m, known=known)


def draws(i, zero=False):
    """The path's draws: the law change, and per month Arnika's delay and the projects."""
    if zero:
        return {"law": False, "months": {ym: (25, []) for ym in b.ym_iter((2026, 1), (2028, 12))}}
    r = rng(SEED + i)
    law = r.random() < LAW_P
    months = {}
    for ym in b.ym_iter((2026, 1), (2028, 12)):
        da = r.randint(*DELAY_A)
        n = poisson(r, PROJ_RATE)
        ps = []
        for _ in range(n):
            amt = max(1, int(r.lognormvariate(PROJ_MU, PROJ_SIGMA) + 0.5))
            ps.append((D(amt), r.randint(*DELAY_P)))
        months[ym] = (da, ps)
    return {"law": law, "months": months}


def business(dr, dec2025, *, known):
    """Invoices and each month's social, health, tax and VAT on this path (none of them read cash)."""
    inv, months = [], {}
    carry, cum = ZERO, ZERO
    prev_social = dec2025["social"]["total"]
    for (y, m) in b.ym_iter((2026, 1), (2028, 12)):
        if m == 1:
            carry, cum = ZERO, ZERO
        da, ps = dr["months"][(y, m)]
        mi = []
        d15 = CAL.on_or_next(date(y, m, 15))
        for net, dl in ps:
            mi.append({"issued": d15, "net": net, "vat": ort(net * D("0.22")), "delay": dl, "client": "project"})
        dl_ = b.last_bd(y, m)
        mi.append({"issued": dl_, "net": ARNIKA, "vat": ort(ARNIKA * D("0.22")), "delay": da, "client": "Arnika"})
        for x in mi:
            x["gross"] = x["net"] + x["vat"]
            x["paid"] = CAL.on_or_next(x["issued"] + timedelta(days=x["delay"]))
        inv += mi
        rev = sum((x["net"] for x in mi), ZERO)
        cum += rev
        soc = b.social(y, m, AWP, known=known)
        rate = LAW_RATE if (dr["law"] and (y, m) >= LAW_FROM) else b.RATE_IT
        tax, parts, left = b.pit_c(rev, ZERO, prev_social + carry, rate)
        carry = left if m < 12 else ZERO
        aw = awq4(y, m, known=known)
        health = ort(D("0.09") * b.tier(cum) * aw)
        vat = duk(sum((x["vat"] for x in mi), ZERO))
        months[(y, m)] = {"invoices": mi, "revenue": rev, "rev_ytd": cum, "social": soc["total"], "social_paid": prev_social,
                          "tax": tax, "rate": rate, "awq4": aw, "health": health, "vat": vat}
        prev_social = soc["total"]
    settle = {}
    for y in (2026, 2027):
        ks = [(y, m) for m in range(1, 13)]
        total = months[ks[-1]]["rev_ytd"]
        owed = sum((ort(D("0.09") * b.tier(total) * months[k]["awq4"]) for k in ks), ZERO)
        settle[y] = owed - sum((months[k]["health"] for k in ks), ZERO)
    return inv, months, settle


class FastMortgage:
    """B-D07 at one rate from 2026-01-05 on (DUKOR 6M assumed flat at its last fixing, F8)."""

    def __init__(self, m):
        self.bal, self.cursor, self.accr, self.pay = m.bal, m.cursor, m.accr, m.pay
        self.slots = [s for s in m.slots[m.k:] if s not in m.holidays]
        self.rate = m.rate_on(date(2026, 1, 5))
        self.last_reset = m.last_reset

    def accrue(self, d):
        if d > self.cursor:
            self.accr += self.bal * self.rate / 100 * (d - self.cursor).days / 365
            self.cursor = d

    def overpay(self, d, x):
        self.accrue(d)
        self.bal -= x
        self.pay = annuity(self.bal, self.rate / 1200, len(self.slots))

    def instalment(self):
        d = self.slots[0]
        rd = max(x for x in b.reset_days(d - timedelta(days=1)) if x < d)
        if rd != self.last_reset:  # B-D07 .4: re-levelled at the first instalment after each reset, as b.Mortgage
            self.pay = annuity(self.bal, self.rate / 1200, len(self.slots))
            self.last_reset = rd
        self.slots.pop(0)
        self.accrue(d)
        i = ort(self.accr)
        self.accr = ZERO
        p = self.pay - i
        if not self.slots or p > self.bal:
            p = self.bal
        self.bal -= p
        return d, i + p


def start_state(*, known):
    """The state on 2025-12-31 (the records, twist included) and January 2026's known items."""
    sim = b.simulate(True, known=ALL_KNOWN)  # R1: the world's replay to 2025-12-31, the records' rows
    bz = sim["bz"]
    mt = sim["mtg"]
    mt.known = known  # R1: from here on the mortgage reads the fixings as known on the forecast's known-as-of date
    mt.run_to(date(2026, 1, 5))  # the 2026-01-05 instalment: the 1 January reset, re-levelled (B-D07)
    jan = mt.rows[-1]
    dec = bz["months"][(2025, 12)]
    flr = sim["bal"]["laz"] + sum((s["gross"] for s in bz["sales"] if s["client"] == "Kessel Lenia sp." and s["issued"].month in (11, 12)), ZERO)
    known = [  # (day, amount into the Cis account), story F5
        (date(2026, 1, 8), D("19520.00")), (date(2026, 1, 14), D("19520.00")), (date(2026, 1, 15), D("4760.00")),
        (CAL.on_or_next(FLR_CONVERT[0] + timedelta(days=1)), ort(flr * FLR_CONVERT[1])),
        (date(2026, 3, 31), b.BARTEK["lent"][1] - b.BARTEK["back"][1]),
        (b.due20(2025, 12), -(dec["social"]["total"] + dec["health"] + dec["tax"])),
        (b.due25(2025, 12), -dec["vat_due"]),
        (bz["settle"][2025]["due"], -bz["settle"][2025]["difference"]),
    ]
    return {"cis": sim["bal"]["cis"], "sav": sim["bal"]["sav"], "mtg": mt, "jan_instalment": jan, "dec": dec, "known": known,
            "flr": flr}


def run(st, dr, overpay=True, *, known, end=END):
    """One path: day by day over the days something happens. Returns what the questions read.

    end: the forecast's last day (default END, 2028-12-31). An earlier end (keys_b.py --end) stops the run there:
    the month ends, the shortfall and the tax run to it, and the liquid money and the mortgage are taken on it.
    The draws are the full run's, so a path cut short is the first part of the same path."""
    end_ym = (end.year, end.month)
    inv, months, settle = business(dr, st["dec"], known=known)
    fm = FastMortgage(st["mtg"])
    ev = {}

    def add(d, prio, amt, what):
        ev.setdefault(d, []).append((prio, D(amt), what))

    add(date(2026, 1, 5), 5, -st["jan_instalment"]["instalment"], "mortgage")
    for d, a in st["known"]:
        add(d, 1 if a > 0 else 6, a, "known")
    for x in inv:
        if x["paid"] <= end:
            add(x["paid"], 1, x["gross"], "client")
    for (y, m), row in months.items():
        add(CAL.on_or_next(date(y, m, 5)), 1, b.OLA_ORDER, "Ola")
        for day in (1, 8, 15, 22):
            add(CAL.on_or_next(date(y, m, day)), 3, -LIVING, "living")
        add(CAL.on_or_next(date(y, m, 10)), 4, -b.FLAT_FEE, "fee")
        add(CAL.on_or_next(date(y, m, 12)), 4, -b.ENERGY, "energy")
        if (y, m) < (2028, 12):
            add(b.due20(y, m), 6, -(row["social"] + row["health"] + row["tax"]), "SIO, RO")
            add(b.due25(y, m), 6, -row["vat"], "VAT")
    for y, diff in settle.items():
        add(CAL.on_or_next(date(y + 1, 4, 20)), 6, -diff, "settlement")
    add(KITCHEN[0], 4, -KITCHEN[1], "kitchen")
    rdays = {CAL.on_or_next(date(y, m, 26)): (y, m) for (y, m) in months}
    slots = [s for s in fm.slots if s <= end]
    for s in slots:
        ev.setdefault(CAL.on_or_next(s), [])
    for d in rdays:
        ev.setdefault(d, [])
    cis, sav = st["cis"], st["sav"]
    month_end, overpaid = {}, ZERO
    for d in sorted(ev):
        if d > end:
            break
        items = list(ev[d])
        while fm.slots and CAL.on_or_next(fm.slots[0]) == d:
            _, amt = fm.instalment()
            items.append((5, -amt, "mortgage"))
        net = sum((a for _, a, _ in items), ZERO)
        if cis + net < 0:  # B-HR5
            need = D("1000.00") - (cis + net)
            top = (need / 100).to_integral_value(rounding="ROUND_CEILING") * 100
            cis += top
            sav -= top
        cis += net
        if d in rdays:
            y, m = rdays[d]
            mi = [x for x in months[(y, m)]["invoices"] if x["issued"] <= d]
            reserve = b.FLOOR["v2"] + sum((x["vat"] for x in mi), ZERO) + ort(sum((x["net"] for x in mi), ZERO) * D("0.12")) + D("4000.00")
            surplus = cis - reserve
            if surplus > 0:
                ov = min(D("3000.00"), surplus) if overpay else ZERO
                sv = surplus - ov
                avail = cis - D("1000.00")
                if ov + sv > avail:
                    sv = max(ZERO, min(sv, avail - ov))
                    ov = max(ZERO, min(ov, avail))
                if ov > 0:
                    fm.overpay(d, ov)
                    overpaid += ov
                cis -= ov + sv
                sav += sv
        month_end[(d.year, d.month)] = cis + sav
    liquid = {}
    last = st["cis"] + st["sav"]
    for ym in b.ym_iter((2026, 1), end_ym):
        last = month_end.get(ym, last)
        liquid[ym] = last
    fm.accrue(date(2029, 1, 1))
    low = min(liquid.values())
    return {"tax": sum((r["tax"] for ym, r in months.items() if ym <= end_ym), ZERO), "law": dr["law"], "liquid_end": liquid[end_ym],
            "breach": low < FLOOR_LIQUID, "shortfall": max(ZERO, FLOOR_LIQUID - low), "low": low,
            "mortgage": fm.bal, "overpaid": overpaid}


def plan_tax_closed_form(st, *, known, end=END):
    """Zero variance (F9): no projects, no law change; the tax as a sum over months, written out (to the end's month)."""
    tot, carry, prev = ZERO, ZERO, st["dec"]["social"]["total"]
    for (y, m) in b.ym_iter((2026, 1), (end.year, end.month)):
        if m == 1:
            carry = ZERO
        ded = prev + carry
        use = min(ARNIKA, ded)
        tot += duk(duk(ARNIKA - use) * b.RATE_IT)
        carry = (ded - use) if m < 12 else ZERO
        prev = b.social(y, m, AWP, known=known)["total"]
    return tot


def forecast(n, *, known, end=END):
    st = start_state(known=known)
    on, off = [], []
    for i in range(n):
        dr = draws(i)
        on.append(run(st, dr, True, known=known, end=end))
        off.append(run(st, dr, False, known=known, end=end))
    z = draws(0, zero=True)
    zero = [run(st, z, False, known=known, end=end), run(st, z, False, known=known, end=end)]
    zero_on = run(st, z, True, known=known, end=end)
    return {"state": st, "on": on, "off": off, "zero": zero, "zero_on": zero_on,
            "closed_tax": plan_tax_closed_form(st, known=known, end=end)}
