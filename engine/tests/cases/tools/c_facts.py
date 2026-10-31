"""Household C's facts and its simulation: what the documents state (each citing its document id) and
the accounts' movements from 2024-01-01 to 2026-09-30, which gen_c.py writes as records.

The house rules are applied here as the household applied them (C-HR2's sales, C-HR3's transfers as Jan
computed them, C-HR7's repayments); where the household departed from a rule, the departure is a fact
named in a comment and in the writers' notes. Seed 20240614.
"""
import random
from datetime import date, timedelta
from decimal import Decimal, ROUND_UP

from c_law import (CAL, RCAL, ZERO, D, P, add_months, cbl_ref, coordinate, duk, end_of_month, figure, half_up,
                   ort, withhold)
from fam.home import run_payer  # noqa: E402

SEED = 20240614
T0, T1 = date(2024, 1, 1), date(2026, 9, 30)


def dd(s):
    return P(s)


def days(a, b):
    a, b = P(a), P(b)
    while a <= b:
        yield a
        a += timedelta(days=1)


def bd_on_or_before(d, cal=CAL):
    d = P(d)
    while not cal.is_business(d):
        d -= timedelta(days=1)
    return d


def bd_on_or_after(d, cal=CAL):
    return cal.on_or_next(P(d))


def ym(d):
    return f"{d.year}-{d.month:02d}"


def months(a, b):
    """'YYYY-MM' from a to b inclusive."""
    y, m = int(a[:4]), int(a[5:])
    while f"{y}-{m:02d}" <= b:
        yield f"{y}-{m:02d}"
        y, m = (y, m + 1) if m < 12 else (y + 1, 1)


def first(ymk):
    return date(int(ymk[:4]), int(ymk[5:]), 1)


# --------------------------------------------------------------------------- people, accounts

MARRIED = date(2016, 9, 17)          # C-D01: no marital contract
STAS_BORN = date(2023, 3, 9)         # C-D02
ACC_JOINT = "LN27 6100 3001 0000 4410 7765 0001"
ACC_POT = "LN27 6100 3001 0000 4410 7765 0002"
ACC_MARTA = "LN53 6100 3001 0000 2209 5148 0001"
ACC_FIRM = "LN81 6100 3001 0000 7312 0046 0001"
ACC_LAZUR = "LN88 1020 0044 9172"      # Jan's multi-currency account at Bank Lazur (DUK and FLR)
ACC_IRENA = "LN14 2090 0017 3302 1180 0001"
MORTGAGE = "HIP/2019/04471"

# --------------------------------------------------------------------------- Marta's pay (C-D05, C-D06, C-D07)

def marta_payments(year):
    """Her employer's payments (Przychodnia Pod Lipami): paid on the 28th, or the business day before."""
    out = []
    for m in range(1, 13):
        k = f"{year}-{m:02d}"
        if k < "2024-03" or "2025-09" <= k <= "2026-08" or k > "2026-09":
            continue
        if k == "2024-03":
            g = half_up(D("7350.00") * 25 / 31)       # back from 2024-03-07: 25 of 31 days (C-D06)
        elif k <= "2024-12":
            g = D("7350.00")
        elif k <= "2025-08":
            g = D("7700.00")
        else:
            g = D("7900.00")
        out.append({"date": bd_on_or_before(date(year, m, 28)).isoformat(), "kind": "pay", "gross": g})
    return out


def marta_payroll(year, *, known):
    pays = marta_payments(year)
    if not pays:
        return [], None
    return run_payer(pays, reducing=True, known=known)


# SOC-8, the SIO's decision C-D03: base, 81.5 % for 52 weeks from 2023-03-09
PARENTAL_BASE = ort(((10 * D("6800.00") + 2 * D("7100.00")) / 12) * (1 - D("0.1345")))
PARENTAL_DAY = ort(PARENTAL_BASE * D("0.815") / 30)
PARENTAL_END = STAS_BORN + timedelta(days=363)


def parental_2024():
    """The SIO's payments in 2024 (C-D04): each month's days in the period times a day's pay, paid by the 10th."""
    out = []
    for k, n in (("2023-12", 31), ("2024-01", 31), ("2024-02", 29), ("2024-03", (PARENTAL_END - date(2024, 3, 1)).days + 1)):
        nxt = add_months(first(k), 1)
        out.append({"for": k, "date": bd_on_or_before(nxt.replace(day=10)).isoformat(), "kind": "parental",
                    "gross": n * PARENTAL_DAY, "days": n})
    return out


# --------------------------------------------------------------------------- Jan's workshop (C-D08 .. C-D13, records)

OPENED, CLOSED = date(2021, 2, 1), date(2024, 6, 14)   # C-D08, C-D09
CLIENTS = ["Kowal Meble Domowe", "Pensjonat Pod Świerkiem", "Gabinet dr Rudy", "Hanna Sikora", "Restauracja Stara Kuźnia",
           "Wiesław Mróz", "Szkoła Podstawowa nr 2 w Zaborze", "Ewa i Karol Dąb", "Kawiarnia Pod Zegarem", "Joanna Wilczek"]
SUPPLIERS = [("Tartak Borówka sp.j.", "drewno"), ("Okucia Zając", "okucia"), ("Farby Lazurowe Zabór", "lakiery"),
             ("Narzędziownia Kos", "narzędzia")]


def workshop_books():
    """Sales and cost registers 2023-07 .. 2024-06 (invoice issue day, net, VAT)."""
    r = random.Random(SEED)
    sales, costs = [], []
    no_s, no_c = 0, 0
    for k in months("2023-07", "2024-06"):
        y, m = int(k[:4]), int(k[5:])
        last = CLOSED if k == "2024-06" else end_of_month(date(y, m, 1))
        n = 2 if k == "2024-06" else r.choice([3, 4, 4, 5])
        for i in range(n):
            day = min(last.day, 4 + i * 6 + r.randint(0, 3))
            d = bd_on_or_before(date(y, m, day))
            if k == "2024-06":
                d = [date(2024, 6, 4), date(2024, 6, 12)][i]
            net = D(r.randint(180, 950) * 10)
            if k == "2024-06":
                net = [D("3800.00"), D("2650.00")][i]
            no_s += 1
            sales.append({"no": f"FV/{y}/{no_s:03d}", "date": d, "client": CLIENTS[r.randrange(len(CLIENTS))],
                          "net": net, "vat": ort(net * D("0.22")), "paid": bd_on_or_after(d + timedelta(days=r.randint(7, 14)))})
        # rent of the workshop (C-D10): invoice on the 1st business day, 2 600.00 net
        no_c += 1
        rd = bd_on_or_after(date(y, m, 1))
        costs.append({"no": f"K/{y}/{no_c:03d}", "date": rd, "supplier": "Hala Przemysłowa Zabór sp. z o.o.",
                      "what": "najem warsztatu", "net": D("2600.00"), "vat": D("572.00"), "paid": rd + timedelta(days=2)})
        for j in range(r.choice([2, 3, 3, 4])):
            sup, what = SUPPLIERS[r.randrange(len(SUPPLIERS))]
            d = bd_on_or_before(date(y, m, min(last.day, 3 + j * 7 + r.randint(0, 4))))
            net = D(r.randint(4000, 32000)) / 10
            if k == "2024-06":
                net = net + D("2400.00") if j == 0 else net
            no_c += 1
            costs.append({"no": f"K/{y}/{no_c:03d}", "date": d, "supplier": sup, "what": what, "net": net,
                          "vat": ort(net * D("0.22")), "paid": d + timedelta(days=3)})
    return sales, costs


SOC3_RATES = (("pension", "0.195"), ("disability", "0.08"), ("accident", "0.0167"), ("sickness", "0.0245"),
              ("labour fund", "0.0245"))


def soc3(k, days_active=None, days_in_month=None, *, known):
    """SOC-3: stage 3 from 2023-09 (opened 2021-02-01): base 60 % of the year's AVG-WAGE-PROJ; stage 2 before
    it: 30 % of the MIN-WAGE on 1 January; sickness elected (C-D08); pro-rated in the month of closing."""
    y = int(k[:4])
    if k >= "2023-09":
        aw, _ = figure("AVG-WAGE-PROJ", date(y, 1, 1), known=known)
        base = aw * D("0.60")
        rates = SOC3_RATES
    else:
        mw, _ = figure("MIN-WAGE", date(y, 1, 1), known=known)
        base = mw * D("0.30")
        rates = SOC3_RATES[:4]
    if days_active is not None:
        base = base * days_active / days_in_month  # SOC-3 v1 .5 (home supplement): the month of closing
    parts = {n: ort(base * D(r)) for n, r in rates}
    return {"base": base, **parts, "total": sum(parts.values(), ZERO)}


def soc4_floor(y, *, known):
    mw, _ = figure("MIN-WAGE", date(y, 1, 1), known=known)
    return ort(mw * D("0.09"))


def workshop_months(sales, costs, months_list, closing_month=None, *, known):
    """Monthly revenue, costs, income, SOC-3 and SOC-4 (form a) for the months listed."""
    out = {}
    for k in months_list:
        rev = sum((s["net"] for s in sales if ym(s["date"]) == k), ZERO)
        cst = sum((c["net"] for c in costs if ym(c["date"]) == k), ZERO)
        inc = rev - cst
        if k == closing_month:
            sc = soc3(k, CLOSED.day, 30, known=known)
        else:
            sc = soc3(k, known=known)
        fl = soc4_floor(int(k[:4]), known=known)
        h = max(ort(inc * D("0.09")), fl)
        out[k] = {"revenue": rev, "costs": cst, "income": inc, "soc3": sc["total"], "soc3 detail": sc, "health": h,
                  "health floor": fl}
    return out


def due20(k):
    """The 20th of the next month, moved to the next business day (SOC-3 .6, SOC-4 .4, PIT-10 .5; GEN-2.2)."""
    return CAL.on_or_next(add_months(first(k), 1).replace(day=20))


def due25(k):
    return CAL.on_or_next(add_months(first(k), 1).replace(day=25))


def pit10a_advances(month_rows, paid_soc):
    """PIT-10 .5 (a): the PIT-3 tax (in force on the month's last day: v3) on the year's profit to date, less
    the advances already paid; profit = revenue − costs − social contributions paid to date (PIT-10 .3)."""
    from fam.home import pit3_tax
    out, paid, rev, cst = [], ZERO, ZERO, ZERO
    for k, r in month_rows.items():
        rev += r["revenue"]
        cst += r["costs"]
        last = end_of_month(first(k))
        soc_paid = sum((a for d0, a in paid_soc if d0 <= min(last, CLOSED)), ZERO)
        profit = duk(rev - cst - soc_paid)
        tax = duk(pit3_tax(profit, "v3"))
        adv = max(ZERO, duk(tax - paid))
        paid += adv
        out.append({"month": k, "profit to date": profit, "tax to date": tax, "advance": adv, "due": due20(k).isoformat()})
    return out


def vat_returns(sales, costs, months_list):
    """VAT-4: output VAT on invoices issued less input VAT on invoices received in the month; whole dukats."""
    out, carry = [], ZERO
    for k in months_list:
        o = sum((s["vat"] for s in sales if ym(s["date"]) == k), ZERO)
        i = sum((c["vat"] for c in costs if ym(c["date"]) == k), ZERO)
        res = duk(o) - duk(i) - carry
        carry = -res if res < 0 else ZERO
        out.append({"month": k, "output": duk(o), "input": duk(i), "result": res, "due": due25(k).isoformat()})
    return out


# --------------------------------------------------------------------------- Jan's job in Rosvald (C-D14 .. C-D18)

JOB_START = date(2024, 7, 1)
BASE = {"2024/25": D("4800.00"), "2025/26": D("5000.00"), "2026/27": D("5150.00")}   # C-D14, C-D16
OT_RATE = {"2024/25": D("37.50"), "2025/26": D("39.00"), "2026/27": D("40.25")}
# overtime hours of each month's work, from the payslips (C-D20 ..)
OT = {"2024-07": 6, "2024-08": 0, "2024-09": 14, "2024-10": 9, "2024-11": 12, "2024-12": 4,
      "2025-01": 10, "2025-02": 0, "2025-03": 7, "2025-04": 11, "2025-05": 3, "2025-06": 8, "2025-07": 5,
      "2025-08": 0, "2025-09": 16, "2025-10": 13, "2025-11": 12, "2025-12": 2, "2026-01": 9, "2026-02": 6,
      "2026-03": 8, "2026-04": 10, "2026-05": 4, "2026-06": 7, "2026-07": 3, "2026-08": 0}
CLASS2_NOTICE = {"2024/25": date(2024, 7, 1), "2025/26": date(2025, 5, 20), "2026/27": date(2026, 4, 1)}   # C-D15, C-D17, C-D18
MISSING_SLIP = "2025-10"   # the payslip for October 2025 was never kept (story)


def contract_period(work_month):
    """The pay terms by the month worked (C-D14: raises from the pay for April)."""
    y, m = int(work_month[:4]), int(work_month[5:])
    y0 = y if m >= 4 else y - 1
    return f"{y0}/{(y0 + 1) % 100:02d}"


def pay_day(work_month):
    """C-D14: pay for a month is paid on the 5th of the next, or the Rosvald business day before."""
    return bd_on_or_before(add_months(first(work_month), 1).replace(day=5), RCAL)


def jan_class(paid):
    """RWT-4: class 2 for pay paid after the day the year's notice reached the employer; class 1 otherwise."""
    from c_law import tax_year
    ty = tax_year(paid)
    n = CLASS2_NOTICE.get(ty)
    return 2 if n is not None and P(paid) > n else 1


def jan_slips(*, known):
    out = []
    for k in months("2024-07", "2026-08"):
        cp = contract_period(k)
        g = BASE[cp] + OT[k] * OT_RATE[cp]
        pd = pay_day(k)
        w = withhold(g, pd, jan_class(pd), known=known)
        out.append({"work": k, "paid": pd, "hours": OT[k], **w})
    return out


ASSESS_2425 = {"claimed": date(2025, 6, 12), "decided": date(2025, 9, 10), "paid": date(2025, 9, 24)}  # C-D19
ASSESS_2526 = {"claimed": date(2026, 7, 14)}                                                          # no decision by 2026-09-30

# --------------------------------------------------------------------------- the travel log (records/jan-dziennik.csv)

LEAVE = ([date(2024, 8, d) for d in range(19, 24)] + [date(2024, 12, 27), date(2024, 12, 30)] +
         [date(2025, 2, d) for d in range(17, 22)] + [date(2025, 5, 2)] + [date(2025, 8, d) for d in range(4, 16)] +
         [date(2025, 10, d) for d in range(27, 32)] + [date(2025, 12, 22), date(2025, 12, 23), date(2025, 12, 29),
                                                       date(2025, 12, 30)] +
         [date(2026, 2, d) for d in range(16, 21)] + [date(2026, 7, d) for d in range(20, 32)])
AWAY = ([date(2025, 2, 15) + timedelta(days=i) for i in range(7)] +       # nights in the mountains, 15–21 Feb 2025
        [date(2025, 8, 2) + timedelta(days=i) for i in range(14)] +       # by the sea, 2–15 Aug 2025
        [date(2026, 7, 18) + timedelta(days=i) for i in range(14)])       # by the lake, 18–31 Jul 2026
SATURDAYS = [date(2024, 9, 14), date(2024, 10, 12), date(2024, 11, 16), date(2025, 1, 25), date(2025, 3, 8),
             date(2025, 5, 17), date(2025, 6, 21), date(2025, 10, 18), date(2025, 11, 15), date(2026, 1, 24),
             date(2026, 3, 14), date(2026, 4, 18), date(2026, 6, 13)]
OUTINGS = [date(2024, 7, 28), date(2024, 12, 1), date(2025, 3, 30), date(2025, 12, 7), date(2026, 5, 24)]  # family Sundays in Ostrel
PROJECT = (date(2025, 9, 8), date(2025, 11, 27))   # Marrenburg: Monday to Thursday nights there


def travel_log():
    """One row a day from 2024-07-01: where Jan slept (the night beginning that day) and whether he was in Rosvald."""
    out = []
    for d in days("2024-07-01", "2026-09-30"):
        work = RCAL.is_business(d) and d not in LEAVE
        in_r = work or d in SATURDAYS or d in OUTINGS
        night = "dom"
        if d in AWAY:
            night = "wyjazd"
        elif PROJECT[0] <= d <= PROJECT[1] and d.weekday() <= 3 and work and RCAL.is_business(d + timedelta(days=1)) \
                and (d + timedelta(days=1)) not in LEAVE:
            night = "Marrenburg"
        out.append({"date": d, "rosvald": in_r, "night": night, "work": work})
    return out


def presence(log, a, b):
    return sum(1 for r in log if P(a) <= r["date"] <= P(b) and r["rosvald"])


def nights_home(log, k):
    return sum(1 for r in log if ym(r["date"]) == k and r["night"] == "dom")


# --------------------------------------------------------------------------- the house: mortgage (C-D24, C-D25)

LOAN = {"principal": D("400000.00"), "rate": D("0.062"), "n": 300, "paid out": date(2019, 5, 20), "first": date(2019, 6, 15)}


def instalment():
    r = LOAN["rate"] / 12
    return half_up(LOAN["principal"] * r / (1 - (1 + r) ** -LOAN["n"]))


def mortgage(extra, upto=date(2040, 12, 31)):
    """C-D24: a level instalment on the 15th (the next business day where it isn't one); each instalment's
    interest is 1/12 of 6.20 % of the principal owed the day before, to the ort; an overpayment cuts the
    principal on its day and shortens the term (C-D25). extra: [(date, amount)]."""
    inst = instalment()
    bal = LOAN["principal"]
    rows = []
    ex = sorted(extra)
    k = 0
    while bal > 0:
        day = CAL.on_or_next(add_months(LOAN["first"], k))
        while ex and ex[0][0] < day:
            d0, a = ex.pop(0)
            bal -= a
            rows.append({"date": d0, "kind": "overpayment", "amount": a, "balance": bal})
        if day > upto:
            break
        intr = ort(bal * LOAN["rate"] / 12)
        pay = min(inst, bal + intr)
        bal = bal + intr - pay
        rows.append({"date": day, "kind": "instalment", "amount": pay, "interest": intr, "principal": pay - intr, "balance": bal})
        k += 1
    return rows, inst


# --------------------------------------------------------------------------- bills (joint account)

def bills(k):
    """The house's bills paid in a month (joint account): (day, payee, title, amount)."""
    y, m = int(k[:4]), int(k[5:])
    winter = m in (1, 2, 3, 11, 12)
    out = [(3, "Sieć Wola sp. z o.o.", "internet", D("89.00")),
           (12, "Wodociągi Zabór", "woda", D("118.40") + D((y * 7 + m * 13) % 29)),
           (20, "Energia Zabór S.A.", "prąd", D("296.00") + D((y * 11 + m * 17) % 61) + D("0.35")),
           (22, "Gazownia Graniczna", "gaz", (D("418.00") if winter else D("112.00")) + D((y + m * 5) % 37))]
    return [(CAL.on_or_next(date(y, m, d)), p, t, a) for d, p, t, a in out]


V1_BILLS = ("woda", "prąd", "gaz")
V2_BILLS = ("woda", "prąd", "gaz", "internet")


def hr3_due(k, log, inst_paid, version):
    """C-HR3: half of (instalment + bills + food), times nights at home over the month's nights; to the ort."""
    names = V1_BILLS if version == "v1" else V2_BILLS
    food = D("2400.00") if version == "v1" else D("2800.00")
    b = sum((a for d0, p, t, a in bills(k) if t in names), ZERO)
    nh = nights_home(log, k)
    n = end_of_month(first(k)).day
    costs = inst_paid + b + food
    return {"instalment": inst_paid, "bills": b, "food": food, "costs": costs, "nights home": nh, "nights": n,
            "due": ort(costs / 2 * nh / n)}


def hr3_version(k):
    """C-HR3 under C-HR1: v2 signed 2025-03-20 / 2025-04-02 reaches back to January 2025, but never past the
    settle-up of 2025-02-28: it governs months from 2025-03."""
    return "v2" if k >= "2025-02" else "v1"


def hr3_version_as_jan(k):
    """How Jan applied it (the records): v2 from the month it came into force, May 2025."""
    return "v2" if k >= "2025-05" else "v1"
