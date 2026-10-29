"""Household A: the facts its documents state, and the payroll they imply under the home law.

Shared by gen_a.py (which writes the dossier) and keys_a.py (which computes the keys). Each fact
names the document that states it; the records are generated from these facts and the market files.
"""
import calendar as _cal
from datetime import date, timedelta
from decimal import Decimal

from fam.home import CAL, ZERO, D, ort, run_payer

# --- people (story cast)
ANIA_BORN, TOMEK_BORN, ZOSIA_BORN = date(1991, 3, 14), date(1989, 7, 2), date(2023, 4, 27)
MARRIED = date(2016, 9, 10)

# --- Tomek's pay (A-D01 Nurt Software's certificate of earnings; A-D03 the option cash-out)
T_GROSS = {2020: D("6100.00"), 2021: D("6300.00"), 2022: D("6500.00"), 2023: D("6680.00"), 2024: D("7400.00"),
           2025: D("7900.00")}
T_CASHOUT = (date(2022, 10, 1), D("110000.00"))  # paid with the October 2022 pay
T_WCP_EE = (D("0.02"), D("0.01"))   # A-D06: basic 2 %, additional 1 %
T_WCP_ER = (D("0.015"), D("0.01"))  # A-D06: basic 1.5 %, the plan's top-up 1.0 %

# --- Ania's pay (A-D02 Kalina Logistyka's certificate of earnings)
A_GROSS = {2020: D("4800.00"), 2021: D("5000.00"), 2022: D("5200.00"), 2023: D("5400.00"), 2024: D("5700.00"),
           2025: D("6000.00")}
A_PARTIAL = {"2023-04": D("4680.00"), "2024-04": D("1140.00")}  # 26 days before the birth; 6 days after return
A_LEAVE = ("2023-05", "2024-03")  # no pay for these months, both included
A_WCP_EE = (D("0.02"),)
A_WCP_ER = (D("0.015"),)

# --- parental pay (A-D15 the SIO's decision)
PARENTAL_RATE = D("0.815")  # elected within 21 days (SOC-8 .2)


def ym_iter(a, b):
    y, m = a
    while (y, m) <= b:
        yield y, m
        y, m = (y + 1, 1) if m == 12 else (y, m + 1)


def bday_before_or_on(d):
    while not CAL.is_business(d):
        d -= timedelta(days=1)
    return d


def last_bday(y, m):
    return bday_before_or_on(date(y, m, _cal.monthrange(y, m)[1]))


def next_month(y, m):
    return (y + 1, 1) if m == 12 else (y, m + 1)


def t_payday(y, m):
    """A-D01: pay for a month on the 10th of the next (from the January 2025 payment, the 15th), or the business day before."""
    ny, nm = next_month(y, m)
    day = 15 if (ny, nm) >= (2025, 1) else 10
    return bday_before_or_on(date(ny, nm, day))


def t_gross(y, m, cashout=True):
    g = T_GROSS[y]
    if cashout and date(y, m, 1) == T_CASHOUT[0]:
        g += T_CASHOUT[1]
    return g


def a_gross(y, m):
    k = f"{y}-{m:02d}"
    if k in A_PARTIAL:
        return A_PARTIAL[k]
    if A_LEAVE[0] <= k <= A_LEAVE[1]:
        return None
    return A_GROSS[y]


def a_payday(y, m):
    """A-D02: the last business day of the month the pay is for."""
    return last_bday(y, m)


def a_wcp_day(y, m):
    """A-D07: Kalina pays the plan contributions on the 15th of the month after the pay, or the business day before."""
    ny, nm = next_month(y, m)
    return bday_before_or_on(date(ny, nm, 15))


def tomek_payments(cashout=True):
    """Every payment by Nurt Software: pay, and the plan contributions paid with the next pay (WCP-2 .1)."""
    out, prev = [], None
    for y, m in ym_iter((2020, 11), (2025, 12)):
        day = t_payday(y, m)
        g = t_gross(y, m, cashout)
        if prev is not None:
            er = sum((ort(prev * r) for r in T_WCP_ER), ZERO)
            out.append({"date": day, "kind": "wcp_er", "gross": er, "for": f"plan contributions on the pay of {prev_day}"})
        out.append({"date": day, "kind": "pay", "gross": g, "wcp_ee_rates": T_WCP_EE, "for": f"pay for {y}-{m:02d}"})
        prev, prev_day = g, day
    return out


def ania_payments():
    out = []
    for y, m in ym_iter((2020, 12), (2025, 12)):
        g = a_gross(y, m)
        if g is None:
            continue
        out.append({"date": a_payday(y, m), "kind": "pay", "gross": g, "wcp_ee_rates": A_WCP_EE, "for": f"pay for {y}-{m:02d}"})
        er = sum((ort(g * r) for r in A_WCP_ER), ZERO)
        out.append({"date": a_wcp_day(y, m), "kind": "wcp_er", "gross": er, "for": f"plan contributions on the pay for {y}-{m:02d}"})
    return out


def parental():
    """SOC-8: base, a day's pay, and each month's pay with its payment day."""
    months = [(y, m) for y, m in ym_iter((2022, 4), (2023, 3))]
    avg = sum((A_GROSS[y] for y, m in months), ZERO) / 12  # SOC-8 .3: the 12 months before the birth's month
    base = ort(avg * (1 - D("0.1345")))
    day_pay = ort(base * PARENTAL_RATE / 30)  # SOC-8 .4
    first, last = ZERO_DAY, ZERO_DAY + timedelta(days=363)  # 52 weeks, the birth day is day 1
    out = []
    for y, m in ym_iter((2023, 4), (2024, 4)):
        a = max(first, date(y, m, 1))
        b = min(last, date(y, m, _cal.monthrange(y, m)[1]))
        n = (b - a).days + 1
        ny, nm = next_month(y, m)
        out.append({"date": bday_before_or_on(date(ny, nm, 10)), "kind": "parental", "gross": n * day_pay,
                    "for": f"parental pay for {y}-{m:02d} ({n} days)", "days": n})
    return {"average": avg, "base": base, "day_pay": day_pay, "first": first, "last": last, "months": out}


ZERO_DAY = ZOSIA_BORN


def payer_year(payments, year):
    return [p for p in payments if p["date"].year == year]


def payroll(year, law="enacted", cashout=True, *, known):
    """Each payer's months and yearly totals for a tax year: {'NURT': (rows, tot), 'KALINA': ..., 'SIO': ...}."""
    res = {}
    t = payer_year(tomek_payments(cashout), year)
    if t:
        res["NURT"] = run_payer(t, reducing=True, law=law, known=known)
    a = payer_year(ania_payments(), year)
    if a:
        res["KALINA"] = run_payer(a, reducing=True, law=law, known=known)
    s = payer_year(parental()["months"], year)
    if s:
        res["SIO"] = run_payer(s, reducing=False, law=law, known=known)
    return res


# =====================================================================================
# Investments, accounts and the joint account: the simulation the records are written from.
# =====================================================================================
import random  # noqa: E402

from fam.home import duk, figure, price_on, distributions, units_down, joint_return, child_relief  # noqa: E402
from lib.num import split  # noqa: E402

FEE_RATE, FEE_MIN = D("0.0025"), D("5.00")  # A-D12 Rzeka's terms: purchases 0.25 %, at least 5.00; redemptions free
from fam import e as E  # noqa: E402  the family's joins, stated once (E1)
GIFT_DAY = E.GIFT["day"]                     # A-D18
GIFT_LOTS = E.GIFT["lots"]
HALINA_DAYS = {"G1": date(2018, 11, 7), "G2": date(2019, 3, 21), "G3": date(2019, 8, 12)}  # not in the dossier
CASH_GIFT = (E.CASH_GIFT["day"], E.CASH_GIFT["amount"])  # A-D19, handed over in cash
GIFT_REPORT = date(2024, 3, 14)               # A-D20
WRAP_WITHDRAWAL = (date(2024, 3, 5), date(2024, 3, 7))  # asked, paid (A-D35)
GIFT_SALE = date(2024, 3, 7)
T_SALES = [(date(2022, 10, 14), "WEK-EM-A"), (date(2023, 11, 15), "WEK-IDX-D"), (date(2024, 11, 15), "WEK-EM-A")]
ESTATE_PAYOUT = (E.PAYOUT_DAY, E.PAYOUTS["Anna"]["amount"])  # from the division of Zbigniew's estate (A-D36)
WCP_OPEN = {"T": D("56.8312"), "A": D("31.0457")}   # units on 2021-01-01 (the plans' registers)


def rule_day(y, m):
    """A-HR2: the 15th, or the next business day."""
    return CAL.on_or_next(date(y, m, 15))


class Book:
    def __init__(self):
        self.lots, self.cash, self.rows, self.contrib = [], {}, {}, []
        self.n = 0

    def row(self, acct, **kw):
        self.rows.setdefault(acct, []).append(kw)

    def lot(self, acct, fund, day, units, cost, **kw):
        self.n += 1
        lot = {"id": f"{acct}-{self.n:03d}", "account": acct, "fund": fund, "date": day, "units": units, "cost": cost}
        lot.update(kw)
        self.lots.append(lot)
        return lot

    def held(self, acct, fund=None, on=None):
        return [l for l in self.lots if l["account"] == acct and l["units"] > 0 and (fund is None or l["fund"] == fund)
                and (on is None or l["date"] < on)]

    def units(self, acct, fund):
        return sum((l["units"] for l in self.held(acct, fund)), ZERO)


def rzk_buy(book, acct, fund, day, amount, *, known):
    px, _ = price_on(fund, day, known)
    fee = max(FEE_MIN, ort(amount * FEE_RATE))
    u = units_down((amount - fee) / px)
    paid = ort(u * px)
    book.cash[acct] = book.cash.get(acct, ZERO) - paid - fee
    book.lot(acct, fund, day, u, paid + fee, price=px, fee=fee)
    book.row(acct, date=day, op="BUY", fund=fund, units=u, price=px, value=paid, fee=fee, cash=book.cash[acct])


def rzk_deposit(book, acct, day, amount, op="DEPOSIT"):
    book.cash[acct] = book.cash.get(acct, ZERO) + amount
    book.row(acct, date=day, op=op, fund="", units="", price="", value=amount, fee=ZERO, cash=book.cash[acct])


def fifo_sell(book, acct, fund, day, units):
    """Take units oldest first (PIT-8 .3, by acquisition day within the account); returns [(lot, units)]."""
    need, taken = units, []
    for l in sorted(book.held(acct, fund), key=lambda l: (l["date"], l["id"])):
        if need == 0:
            break
        t = min(need, l["units"])
        taken.append((dict(l), t))
        l["units"] -= t
        need -= t
    assert need == 0
    return taken


def wrap_buy(book, day, amount, *, known):
    """Brzask wrapped account: half to BRZ-GLB-A, half to BRZ-BND-A; units to 4 places, down (A-D11)."""
    book.contrib.append((day, amount))
    book.cash["WRAP"] = book.cash.get("WRAP", ZERO) + amount
    book.row("WRAP", date=day, type="contribution", fund="", units="", price="", amount=amount)
    for fund, part in zip(("BRZ-GLB-A", "BRZ-BND-A"), split(amount, [1, 1])):
        px, _ = price_on(fund, day, known)
        u = units_down(part / px)
        paid = ort(u * px)
        book.cash["WRAP"] -= paid
        book.lot("WRAP", fund, day, u, paid, price=px)
        book.row("WRAP", date=day, type="purchase", fund=fund, units=u, price=px, amount=paid)


def wrap_room(book, day, *, known):
    lim, _ = figure("WRAP-LIMIT", day, known)
    used = sum((a for d, a in book.contrib if d.year == day.year and d < day), ZERO)
    return lim - used, lim, used


def invest(book, day, amount, v, *, known):
    """A-HR1: the wrapped account first up to the year's limit; the rest to Tomek's Rzeka account."""
    room, _, _ = wrap_room(book, day, known=known)
    w = max(ZERO, min(amount, room))
    if w > 0:
        wrap_buy(book, day, w, known=known)
    b = amount - w
    if b > 0:
        rzk_deposit(book, "RZK-T", day, b)
        if v == 1:
            for fund, part in zip(("WEK-EM-A", "WEK-IDX-D", "BRZ-DIV-D"), split(b, [1, 1, 1])):
                rzk_buy(book, "RZK-T", fund, day, part, known=known)
        else:
            rzk_buy(book, "RZK-T", "BRZ-GLB-A", day, b, known=known)
    return w, b


def pay_distributions(book, upto):
    for fund, ex, paid, per in distributions():
        if paid > upto or paid < date(2021, 1, 1) or getattr(book, "_dist_done", set()).__contains__((fund, ex)):
            continue
        u = sum((l["units"] for l in book.held("RZK-T", fund, on=ex)), ZERO)
        if u == 0:
            continue
        book.__dict__.setdefault("_dist_done", set()).add((fund, ex))
        gross = ort(u * per)
        tax = duk(gross * D("0.19"))  # PIT-8 .7, GEN-5.1
        book.cash["RZK-T"] += gross - tax
        book.row("RZK-T", date=paid, op="DISTRIBUTION", fund=fund, units=u, price=per, value=gross, fee=ZERO, cash=book.cash["RZK-T"] + tax)
        book.row("RZK-T", date=paid, op="TAX WITHHELD", fund=fund, units="", price="", value=-tax, fee=ZERO, cash=book.cash["RZK-T"])


# --- the joint account's known payments (A-D33 standing orders; A-D25 the credit; A-D30 the garage; A-D34 instalments)
STANDING = [
    # name, counterparty, day of month, amount, first due, last due
    ("rent", "Jerzy Grab", 1, D("3200.00"), date(2020, 1, 1), date(2024, 12, 31)),
    ("rent", "Jerzy Grab", 1, D("3350.00"), date(2025, 1, 1), None),
    ("nursery", "Żłobek Pszczółka", 5, D("1250.00"), date(2024, 9, 5), None),
    ("internet and phone", "Fala Telekom", 20, D("139.99"), date(2020, 1, 1), None),
    ("garage", "Garaże Kos", 10, D("180.00"), date(2021, 6, 10), date(2024, 8, 10)),
    ("instalments, washing machine and fridge", "Elektro Bór Raty", 25, D("380.00"), date(2024, 10, 25), date(2025, 9, 25)),
]
CAR_CREDIT = {"principal": D("45000.00"), "annual": D("0.079"), "n": 48, "first": date(2021, 6, 14), "commission": D("2700.00")}
FRIENDS = ("Katarzyna Mrówka", "Bartłomiej Wilk")
CARD_REFUNDS = [(date(2024, 11, 4), "Drogeria Mak", D("89.99")), (date(2025, 2, 11), "Księgarnia Pióro", D("45.50")),
                (date(2025, 6, 17), "Sklep Sport Kret", D("249.00")), (date(2025, 10, 2), "Market Kłos", D("12.40"))]
PAYBACKS = [(date(2024, 9, 9), "Katarzyna Mrówka", "for the pizza", D("64.00")),
            (date(2025, 3, 3), "Bartłomiej Wilk", "cinema", D("38.00")),
            (date(2025, 5, 19), "Katarzyna Mrówka", "Ola's birthday gift, my part", D("120.00")),
            (date(2025, 9, 15), "Bartłomiej Wilk", "concert tickets", D("150.00"))]
POWER = [(date(2023, 11, 12), D("286.40")), (date(2024, 1, 12), D("341.10")), (date(2024, 3, 12), D("318.75")),
         (date(2024, 5, 13), D("244.20")), (date(2024, 7, 12), D("201.65")), (date(2024, 9, 12), D("219.30")),
         (date(2024, 11, 12), D("297.85")), (date(2025, 1, 13), D("356.40")), (date(2025, 3, 12), D("329.10")),
         (date(2025, 5, 12), D("251.75")), (date(2025, 7, 14), D("208.90")), (date(2025, 9, 12), D("226.45")),
         (date(2025, 11, 12), D("311.20"))]
OWN_TO_LAZUR = [(date(2024, 12, 20), D("2000.00")), (date(2025, 10, 6), D("1500.00"))]
CAR_SALE = {"day": date(2024, 8, 20), "price": D("41500.00"), "payoff_day": date(2024, 8, 22)}
OC = {"start": date(2023, 9, 1), "end": date(2024, 8, 31), "premium": D("1104.00"), "fee": D("20.00"),
      "renewal_premium": D("1236.00")}
ROAD_TAX = D("720.00")  # MOT-2 .1: 1 798 cm³
JOINT_OPEN = (date(2023, 10, 1), D("9800.00"))
SEED = 20260115


def standing_due(a, b):
    """Known payments due after day a and before day b (A-HR1 v2 .2): [(day, name, amount)]."""
    out = []
    d = a + timedelta(days=1)
    while d < b:
        for name, cp, dom, amt, first, last in STANDING:
            if d.day == dom and first <= d and (last is None or d <= last):
                out.append((d, name, amt))
        cc = CAR_CREDIT
        if d.day == 14 and cc["first"] <= d <= date(2024, 8, 14):
            out.append((d, "car credit instalment", None))
        d += timedelta(days=1)
    return out


def credit_schedule():
    from lib.loans import schedule
    cc = CAR_CREDIT
    return schedule(cc["principal"], cc["annual"] / 12, cc["n"])


def car_settlement():
    """The sale's legs (A-D27, A-D28, A-D26, MOT-2 .3) and the credit's early repayment (CC-1, CC-2)."""
    sch = credit_schedule()
    paid = [r for r in sch if r["month"] <= 39]  # instalments 2021-06-14 .. 2024-08-14
    bal = paid[-1]["balance"]
    days = (CAR_SALE["payoff_day"] - date(2024, 8, 14)).days
    interest = ort(bal * CAR_CREDIT["annual"] * days / 365)  # A-D25 .6: a broken period by days, actual/365
    left_interest = sum((r["interest"] for r in sch if r["month"] > 39), ZERO)
    fee = min(ort(bal * D("0.005")), left_interest)  # A-D25 .8, CC-2 .2: one year or less left
    n, k = CAR_CREDIT["n"], 39
    by_time = ort(CAR_CREDIT["commission"] * (n - k) / n)
    starts = [CAR_CREDIT["principal"]] + [r["balance"] for r in sch[:-1]]
    by_balance = ort(CAR_CREDIT["commission"] * sum(starts[k:], ZERO) / sum(starts, ZERO))
    pol_days = (OC["end"] - OC["start"]).days + 1
    unused = (OC["end"] - CAR_SALE["day"]).days
    oc_refund = (OC["premium"] * unused / pol_days).quantize(D("0.01"), rounding="ROUND_DOWN") - OC["fee"]
    road = ort(ROAD_TAX / 12 * (12 - CAR_SALE["day"].month))
    return {"schedule": sch, "balance": bal, "days": days, "interest": interest, "fee": fee, "left_interest": left_interest,
            "payoff": bal + interest + fee, "refund_time": by_time, "refund_balance": by_balance, "pol_days": pol_days,
            "unused": unused, "oc_refund": oc_refund, "road_refund": road, "instalment": sch[0]["instalment"]}


def card_rows(rng, day, car):
    out = []
    shops = ["Market Kłos", "Sklep Pod Lipą", "Piekarnia Kłosek", "Apteka Rumianek", "Drogeria Mak", "Restauracja Rydz",
             "Księgarnia Pióro", "Kino Iskra", "Sklep Sport Kret", "Warzywniak Burak"]
    k = 1 if rng.random() < 0.72 else 0
    k += 1 if rng.random() < (0.45 if day.weekday() >= 5 else 0.22) else 0
    mult = D("1.45") if day.month == 12 and day.day < 24 else D("1")
    for _ in range(k):
        shop = shops[min(int(rng.random() * len(shops)), len(shops) - 1)]
        amt = (D(15) + D(int(rng.random() * 16000)) / 100) * mult
        out.append(("CARD TRANSACTION", shop, "card ...4471", -ort(amt)))
    if car and day.weekday() == 4:
        out.append(("CARD TRANSACTION", "Stacja Paliw Tor", "card ...4471", -ort(D(180) + D(int(rng.random() * 9000)) / 100)))
    return out


def card_spend(rows, y, m):
    """A-HR1 v2 .3: one month's card spending, less card refunds and friends' paybacks for shared bills."""
    s = ZERO
    for r in rows:
        if r["date"].year == y and r["date"].month == m:
            if r["type"] in ("CARD TRANSACTION", "CARD REFUND"):
                s -= r["amount"]
            elif r["type"] == "INCOMING TRANSFER" and r["counterparty"] in FRIENDS:
                s -= r["amount"]
    return s


def card_average(rows, d):
    ms = []
    y, m = d.year, d.month
    for _ in range(3):
        y, m = (y - 1, 12) if m == 1 else (y, m - 1)
        ms.append((y, m))
    total = sum((card_spend(rows, yy, mm) for yy, mm in ms), ZERO)
    return ort(total / 3), ms


def behaviour(d, amount):
    """What the household actually moved (the writers' notes): round hundreds down; July 2025 skipped; November 2025 + 1 000."""
    if d == date(2025, 7, 15):
        return ZERO
    moved = (amount / 100).to_integral_value(rounding="ROUND_FLOOR") * 100 if amount > 0 else ZERO
    if d == date(2025, 11, 17):
        moved += D("1000.00")
    return D(moved).quantize(D("0.01"))


def net_pays(*, known):
    """{day: [(type, counterparty, title, amount)]} for pay credited to the joint account, 2023-10 .. 2025-12."""
    out = {}
    for y in (2023, 2024, 2025):
        pr = payroll(y, known=known)
        for payer, (rows, _) in pr.items():
            for r in rows:
                pays = [x for x in r["detail"] if "gross" in x or "parental" in x]
                if not pays:
                    continue
                day = date.fromisoformat(pays[-1]["date"])
                if day < JOINT_OPEN[0]:
                    continue
                cp = {"NURT": "Nurt Software", "KALINA": "Kalina Logistyka", "SIO": "Social Insurance Office"}[payer]
                who = "ANIA LIS" if payer != "NURT" else "TOMASZ LIS"
                title = f"{'parental pay' if payer == 'SIO' else 'salary'} {r['month']} {who}"
                out.setdefault(day, []).append(("INCOMING TRANSFER", cp, title, r["net"]))
    return out


def child_benefit_days():
    out = {}
    for y, m in ym_iter((2023, 10), (2025, 12)):
        day = bday_before_or_on(date(y, m, 25))
        out[day] = D("400.00") if y < 2024 else D("650.00")  # FAM-1 v1 .1 / v2 .1
    return out


def household_return(year, law="enacted", health="each", cashout=True, *, known):
    """The joint return (PIT-6) with the child relief (PIT-7), from the payroll."""
    pr = payroll(year, law, cashout, known=known)
    t = [pr["NURT"][1]]
    a = [pr[k][1] for k in ("KALINA", "SIO") if k in pr]
    pinc = duk(sum((x["base"] for x in t), ZERO)) + duk(sum((x["base"] for x in a), ZERO))  # PIT-7 .2
    months = {2023: 9}.get(year, 12 if year > 2023 else 0)
    cr = child_relief(year, months, pinc)
    rel = (cr["half"], cr["half"])  # PIT-7 .4: halves
    j = joint_return(year, a, t, reliefs=rel, law=law, health=health)
    return {"payroll": pr, "parents_income": pinc, "relief": cr, "joint": j, "ania": a, "tomek": t}


def gain_lines(taken, sale_day, sale_px, giver_day=None):
    """Per lot: proceeds, cost, gain, and whether held more than 5 years (PIT-8 .2, .6)."""
    out = []
    for lot, u in taken:
        proceeds = ort(u * sale_px)
        cost = lot["cost"] if u == lot["units"] else ort(lot["cost"] * u / lot["units"])
        start = lot.get("giver_day", lot["date"]) if giver_day is None else giver_day
        out.append({"lot": lot["id"], "units": u, "proceeds": proceeds, "cost": cost, "gain": proceeds - cost, "since": start})
    return out


def simulate(*, known):
    """Everything the records show, 2021-01-01 .. 2025-12-31 (the world: gen_a.py passes ALL_KNOWN)."""
    book = Book()
    book.sales = []
    rng = random.Random(SEED)
    pays = net_pays(known=known)
    cb = child_benefit_days()
    car = car_settlement()
    joint, bal = [], JOINT_OPEN[1]
    rules = []
    ret23 = household_return(2023, known=known)["joint"]
    ret24 = household_return(2024, known=known)["joint"]
    wcp = {"T": [], "A": []}
    wunits = dict(WCP_OPEN)
    tp, ap = tomek_payments(), ania_payments()

    def post(day, typ, cp, title, amount):
        nonlocal bal
        bal += amount
        joint.append({"date": day, "type": typ, "counterparty": cp, "title": title, "amount": amount, "balance": bal})

    # WCP registers (A-D06, A-D07; WCP-1 .4, WCP-3)
    prev = None
    for p in tp:
        if p["kind"] == "pay":
            prev = p
        elif p["kind"] == "wcp_er" and p["date"].year >= 2021:
            pass
    for i, p in enumerate(tp):
        if p["kind"] == "wcp_er" and p["date"] >= date(2021, 1, 1):
            g = [q for q in tp[:i] if q["kind"] == "pay"][-1]["gross"]
            ee = sum((ort(g * r) for r in T_WCP_EE), ZERO)
            px, _ = price_on("WEK-WCP-2055", p["date"], known)
            for src, amt in (("employee", ee), ("employer", p["gross"])):
                u = units_down(amt / px)
                wunits["T"] += u
                wcp["T"].append({"date": p["date"], "source": src, "amount": amt, "price": px, "units": u, "after": wunits["T"]})
    for p in ap:
        if p["kind"] == "wcp_er" and p["date"] >= date(2021, 1, 1):
            g = [q for q in ap if q["kind"] == "pay" and a_wcp_day(q["date"].year, q["date"].month) == p["date"]][0]["gross"]
            ee = sum((ort(g * r) for r in A_WCP_EE), ZERO)
            px, _ = price_on("WEK-WCP-2055", p["date"], known)
            for src, amt in (("employee", ee), ("employer", p["gross"])):
                u = units_down(amt / px)
                wunits["A"] += u
                wcp["A"].append({"date": p["date"], "source": src, "amount": amt, "price": px, "units": u, "after": wunits["A"]})
    for who in ("T", "A"):
        for y in range(2021, 2026):
            day = bday_before_or_on(date(y, 3, 25))
            px, _ = price_on("WEK-WCP-2055", day, known)
            u = units_down(D("240.00") / px)
            wunits[who] += u
            wcp[who].append({"date": day, "source": f"state yearly payment {y - 1}", "amount": D("240.00"), "price": px, "units": u, "after": wunits[who]})
        wcp[who].sort(key=lambda r: (r["date"], r["source"]))
        run = WCP_OPEN[who]
        for r in wcp[who]:
            run += r["units"]
            r["after"] = run

    d = date(2021, 1, 1)
    while d <= date(2025, 12, 31):
        pay_distributions(book, d)
        # Tomek's sales: all units of the fund, proceeds to the joint account (A-D13)
        for sd, fund in T_SALES:
            if sd == d:
                u = book.units("RZK-T", fund)
                px, _ = price_on(fund, d, known)
                taken = fifo_sell(book, "RZK-T", fund, d, u)
                proceeds = ort(u * px)
                book.cash["RZK-T"] += proceeds
                book.row("RZK-T", date=d, op="SELL", fund=fund, units=u, price=px, value=proceeds, fee=ZERO, cash=book.cash["RZK-T"])
                book.sales.append({"person": "T", "day": d, "fund": fund, "price": px, "lines": gain_lines(taken, d, px)})
                book.cash["RZK-T"] -= proceeds
                out_day = CAL.on_or_next(d + timedelta(days=4))
                book.row("RZK-T", date=out_day, op="WITHDRAWAL", fund="", units="", price="", value=-proceeds, fee=ZERO, cash=book.cash["RZK-T"])
                book.__dict__.setdefault("to_joint", []).append((out_day, proceeds, fund))
        if d == GIFT_DAY:
            px, _ = price_on("BRZ-GLB-A", d, known)
            for gid, month, u in GIFT_LOTS:
                hp, _ = price_on("BRZ-GLB-A", HALINA_DAYS[gid], known)
                book.lot("KOP-A", "BRZ-GLB-A", d, u, ort(u * px), giver_month=month, giver_day=HALINA_DAYS[gid],
                         giver_cost=ort(u * hp), gid=gid)
                book.row("KOP-A", date=d, op="TRANSFER IN (gift)", fund="BRZ-GLB-A", units=u, note=f"from H. Sowa, bought {month}")
        if d == date(2024, 2, 12):
            for l in book.held("KOP-A"):
                book.row("KOP-A", date=d, op="TRANSFER OUT", fund=l["fund"], units=l["units"], note="to Rzeka Securities 4471-07")
                book.row("RZK-A", date=d, op="TRANSFER IN", fund=l["fund"], units=l["units"], price="", value=l["giver_cost"], fee=ZERO, cash=ZERO,
                         note=f"cost basis as reported by the transferring broker; acquired {l['giver_month']}")
                l["account"] = "RZK-A"  # PIT-8 .3: keeps its cost and acquisition day
        if d == GIFT_SALE:
            u = book.units("RZK-A", "BRZ-GLB-A")
            px, _ = price_on("BRZ-GLB-A", d, known)
            taken = fifo_sell(book, "RZK-A", "BRZ-GLB-A", d, u)
            proceeds = ort(u * px)
            book.cash["RZK-A"] = proceeds
            book.row("RZK-A", date=d, op="SELL", fund="BRZ-GLB-A", units=u, price=px, value=proceeds, fee=ZERO, cash=proceeds)
            book.sales.append({"person": "A", "day": d, "fund": "BRZ-GLB-A", "price": px, "lines": gain_lines(taken, d, px), "gift": True,
                               "taken": taken})
            rzk_buy(book, "RZK-A", "BRZ-MM-A", d, proceeds, known=known)
        if d == WRAP_WITHDRAWAL[0]:
            px, _ = price_on("BRZ-BND-A", d, known)
            u = units_down(D("20000.00") / px)
            fifo_sell(book, "WRAP", "BRZ-BND-A", d, u)
            paid = ort(u * px)
            book.row("WRAP", date=d, type="redemption", fund="BRZ-BND-A", units=-u, price=px, amount=paid)
            book.row("WRAP", date=WRAP_WITHDRAWAL[1], type="withdrawal", fund="", units="", price="", amount=-paid)
            book.withdrawal = (d, paid)

        if d >= JOINT_OPEN[0]:
            # the day's rows, in the bank's booking order
            for name, cp, dom, amt, first, last in STANDING:
                due = date(d.year, d.month, dom)
                if CAL.on_or_next(due) == d and first <= due and (last is None or due <= last):
                    post(d, "STANDING ORDER", cp, name, -amt)
            if d.day == 14 or CAL.on_or_next(date(d.year, d.month, 14)) == d:
                due = date(d.year, d.month, 14)
                if CAR_CREDIT["first"] <= due <= date(2024, 8, 14) and CAL.on_or_next(due) == d:
                    m = (due.year - 2021) * 12 + due.month - 5
                    post(d, "STANDING ORDER", "Bank Wrzos – consumer credit 77-0412", f"instalment {m}/48", -car["schedule"][m - 1]["instalment"])
            for t in pays.get(d, []):
                post(d, *t)
            if d in cb:
                post(d, "INCOMING TRANSFER", "Family Benefits Office", f"child benefit {d:%Y-%m} Zofia Lis", cb[d])
            for pd, amt in POWER:
                if pd == d:
                    post(d, "OUTGOING TRANSFER", "Prąd Lenia", f"electricity invoice {d:%m/%Y}", -amt)
            for rd, shop, amt in CARD_REFUNDS:
                if rd == d:
                    post(d, "CARD REFUND", shop, "card ...4471", amt)
            for pd, who, title, amt in PAYBACKS:
                if pd == d:
                    post(d, "INCOMING TRANSFER", who, title, amt)
            for od, amt in OWN_TO_LAZUR:
                if od == d:
                    post(d, "OUTGOING TRANSFER", "Bank Lazur – savings LN88 1020 0044 7310", "own transfer", -amt)
            for od, amt, fund in getattr(book, "to_joint", []):
                if od == d:
                    post(d, "INCOMING TRANSFER", "Rzeka Securities", f"payout client 4471-02 ({fund} sold)", amt)
            if d == WRAP_WITHDRAWAL[1]:
                post(d, "INCOMING TRANSFER", "Brzask TFI", "wrapped account withdrawal A. Lis", book.withdrawal[1])
            if d == date(2024, 3, 12):
                post(d, "OUTGOING TRANSFER", "Meble Jodła", "kitchen, order 2024/118", D("-18500.00"))
            if d == date(2024, 3, 28):
                post(d, "OUTGOING TRANSFER", "Municipality of Dębowo", "road tax 2024 car WD 4471K", -ROAD_TAX)
            if d == date(2024, 6, 20):
                post(d, "OUTGOING TRANSFER", "Revenue Office", "gift tax, decision IGT/2024/0611, A. Lis", -gift_tax(known=known)["tax"])
            if d == date(2024, 5, 20):
                post(d, "INCOMING TRANSFER", "Revenue Office", "PIT 2023 refund A. and T. Lis", -ret23["due"])
            if d == CAR_SALE["day"]:
                post(d, "INCOMING TRANSFER", "Marek Dzięcioł", "car Tarpan Kombi WD 4471K, contract of 20.08.2024", CAR_SALE["price"])
            if d == CAR_SALE["payoff_day"]:
                post(d, "OUTGOING TRANSFER", "Bank Wrzos – consumer credit 77-0412", "early repayment in full", -car["payoff"])
            if d == date(2024, 9, 3):
                post(d, "INCOMING TRANSFER", "Bank Wrzos – consumer credit 77-0412", "settlement of early repayment: commission refund", car["refund_time"])
            if d == date(2024, 9, 10):
                post(d, "INCOMING TRANSFER", "Orlik Insurance", "refund, policy OC/2023/55102", car["oc_refund"])
            if d == date(2024, 10, 7):
                post(d, "INCOMING TRANSFER", "Municipality of Dębowo", "road tax 2024 refund WD 4471K", car["road_refund"])
            if d == date(2025, 4, 28):
                for who, amt in capital_payments(book):
                    post(d, "OUTGOING TRANSFER", "Revenue Office", f"PIT capital gains 2024 {who}", -amt)
            if d == date(2025, 4, 30) and ret24["due"] > 0:
                post(d, "OUTGOING TRANSFER", "Revenue Office", "PIT 2024 A. and T. Lis", -ret24["due"])
            if d == date(2025, 5, 26) and ret24["due"] < 0:
                post(d, "INCOMING TRANSFER", "Revenue Office", "PIT 2024 refund A. and T. Lis", -ret24["due"])
            for r in card_rows(rng, d, d < CAR_SALE["day"]):
                post(d, *r)

        if d.day >= 15 and d == rule_day(d.year, d.month):
            if d < date(2024, 1, 15):
                amount, info = D("2500.00"), {"v": 1}
            else:
                nxt = rule_day(*next_month(d.year, d.month))
                due = [(x, n, a if a is not None else car["instalment"]) for x, n, a in standing_due(d, nxt)]
                avg, ms = card_average(joint, d)
                floor = D("1000.00") + sum((a for _, _, a in due), ZERO) + avg
                amount = max(ZERO, bal - floor)
                info = {"v": 2, "balance": bal, "due": due, "card": avg, "floor": floor, "rule": amount}
            moved = amount if info["v"] == 1 else behaviour(d, amount)
            room, lim, used = wrap_room(book, d, known=known)
            if d >= JOINT_OPEN[0] and moved > 0:
                w = max(ZERO, min(moved, room))
                if w > 0:
                    post(d, "OUTGOING TRANSFER", "Brzask TFI – wrapped account A. Lis", "contribution", -w)
                if moved - w > 0:
                    post(d, "OUTGOING TRANSFER", "Rzeka Securities – client 4471-02", "deposit", -(moved - w))
            if moved > 0:
                invest(book, d, moved, info["v"], known=known)
            info.update({"day": d, "moved": moved, "room": room})
            rules.append(info)
        d += timedelta(days=1)
    return {"book": book, "joint": joint, "rules": rules, "wcp": wcp, "car": car, "ret23": ret23, "ret24": ret24}


def capital_payments(book):
    """What each paid for 2024 on 2025-04-28: Tomek his own reckoning (right), Ania the broker's statement (A-D22)."""
    out = []
    st = broker_statement_2024(book)
    out.append(("A. Lis", st["tax"]))
    t = capital_tax(book.sales, "T")
    out.append(("T. Lis", t[2024]["tax"]))
    return out


def broker_statement_2024(book):
    """A-D22 as Rzeka wrote it: cost from the transfer form (Halina's cost), no holding-period test."""
    s = [x for x in book.sales if x["person"] == "A"][0]
    proceeds = sum((l["proceeds"] for l in s["lines"]), ZERO)
    cost = sum((lot["giver_cost"] for lot, u in s["taken"]), ZERO)
    base = duk(proceeds - cost)
    return {"proceeds": proceeds, "cost": cost, "gain": proceeds - cost, "base": base, "tax": duk(base * D("0.19"))}


def capital_tax(sales, person, exempt_lots=(), years=range(2021, 2026), rate=lambda y: D("0.19")):
    """PIT-8 .4–.6 for one person: each year's net gain, the losses carried (half a year, 5 years), the tax."""
    out, losses = {}, []
    for y in years:
        net = ZERO
        lines = []
        for s in sales:
            if s["person"] != person or s["day"].year != y:
                continue
            for ln in s["lines"]:
                held5 = s["day"] > _add_years(ln["since"], 5)
                ex = (held5 and ln["gain"] > 0) or ln["lot"] in exempt_lots  # PIT-8 .6 exempts a gain; a loss is added (.4)
                lines.append(dict(ln, exempt=ex))
                if not ex:
                    net += ln["gain"]
        used = ZERO
        if net > 0:
            room = net
            for L in losses:
                if room <= 0:
                    break
                if y - L["year"] > 5 or L["left"] <= 0:
                    continue
                u = min(L["left"], L["amount"] / 2, room)
                L["left"] -= u
                used += u
                room -= u
        base = duk(max(ZERO, net - used))
        tax = duk(base * rate(y))
        if net < 0:
            losses.append({"year": y, "amount": -net, "left": -net})
        out[y] = {"net": net, "used": used, "base": base, "tax": tax, "lines": lines,
                  "losses_left": [(L["year"], L["left"]) for L in losses]}
    return out


def _add_years(d, n):
    try:
        return d.replace(year=d.year + n)
    except ValueError:
        return d.replace(year=d.year + n, day=28)


def gift_tax(*, known):
    """IGT-2 .. IGT-4: the units (A-D18) added to the 2021 cash gift (A-D19); reported late (A-D20), so not exempt."""
    px, pday = price_on("BRZ-GLB-A", GIFT_DAY, known)
    units = sum((u for _, _, u in GIFT_LOTS), ZERO)
    value = units * px
    base = duk(value + CASH_GIFT[1])  # IGT-3 .1, IGT-2 .3
    free, src = figure("IGT-FREE-I", GIFT_DAY, known)
    over = max(ZERO, base - free)
    if over <= D("12000.00"):
        t = over * D("0.03")
    elif over <= D("24000.00"):
        t = D("360.00") + (over - D("12000.00")) * D("0.05")
    else:
        t = D("960.00") + (over - D("24000.00")) * D("0.07")
    return {"price": px, "price_day": pday, "units": units, "value": value, "base": base, "free": free, "free_src": src,
            "over": over, "tax": duk(t), "deadline": date(2024, 3, 12)}
