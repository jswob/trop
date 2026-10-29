"""Household D · Halina Sowa: the facts her documents state, the rules as code, the joint account
simulated day by day (2024-01-01 .. 2026-09-30) with house rules D-HR1 and D-HR2, and the long view.

Every figure cites its document (D-Dnn), house rule (D-HRn), rule of the home law or the gazette.
gen_d.py writes the records and documents from this module; keys_d.py reads the records back and
uses this module only for the rules as code and the facts documents state.
"""
import math
import pathlib
import random
import sys
from datetime import date, timedelta
from decimal import Decimal, ROUND_CEILING, ROUND_DOWN
from fractions import Fraction

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

from fam.home import ALL_KNOWN, CAL, ZERO, D, duk, ort, figure, scale, PIT3  # noqa: E402
from lib.dates import P, add_months, end_of_month, days, month_key, months_back  # noqa: E402
from lib.loans import schedule  # noqa: E402
from fam import e as E  # noqa: E402  the family's joins, stated once (E1); D derives the payouts, union_e1 checks them

# --------------------------------------------------------------------------- the cast
HALINA_BORN = date(1959, 3, 8)
ZB_BORN = date(1953, 11, 20)
MARRIED = date(1980, 6, 14)            # D-D02: no marital contract, community property (CIV-1)
ADAM_BORN, ADAM_DIED = date(1981, 4, 2), date(2016, 9, 30)   # D-D03
KUBA_BORN = date(2004, 2, 11)          # D-D03: Adam's only child
DEATH = date(2024, 5, 17)              # D-D01, a Friday
CERT_ISSUED, CERT_FINAL = date(2024, 10, 17), date(2024, 11, 3)   # D-D04
IGT_REPORT = date(2025, 5, 5)          # D-D08: Halina's report, stamped by the RO
DIVISION = date(2025, 6, 12)           # D-D07
PAYOUT_DAY = E.PAYOUT_DAY              # D-D07: from the notary's deposit

ACCT = "LN73 4400 2011 0000 8817 2264 0001"      # Bank Olcha, the joint account
NOTARY_DEP = "LN30 1140 2004 0000 3402 0000 9001"
KUBA_OLD, KUBA_NEW = E.PAYOUTS["Jakub"]["to"], "LN08 1020 0044 3315 6610 0001"
ANIA_ACCT = E.PAYOUTS["Anna"]["to"]     # as A-D36
PIOTR_ACCT = E.PAYOUTS["Piotr"]["to"]   # as B-D27

FLOOR, REDEEM_MIN = D("5000.00"), D("3000.00")       # D-HR2
KUBA_SHARE, HALINA_SHARE = Fraction(3, 16), Fraction(13, 16)

# --------------------------------------------------------------------------- pensions (SOC-5, SOC-6, SOC-7)
ZB_START = date(2018, 12, 1)           # D-D09
ZB_CAPITAL = D("773412.18")            # D-D09
H_START = date(2019, 7, 1)             # D-D11
H_CLAIM = date(2019, 6, 10)            # D-D11
SURV_CLAIM = date(2024, 5, 27)         # D-D12: claim within 12 months, election of the survivor pension
SURV_DECISION = date(2024, 6, 14)      # D-D13
V2_CLAIM = date(2025, 8, 26)           # D-D14
V2_DECISION, V2_RECEIVED = date(2025, 11, 20), date(2025, 11, 24)   # D-D15
SIO_CAP_USED = D("1780.96")            # D-D15: the decision's cap reads MIN-PENSION in force on the day of the death


def _earnings(*, known):
    """D-D10: Halina's earnings 1981–2018 as the SIO's statement shows them (whole dukats)."""
    out, w = {}, D("3850")
    for y in range(1981, 2019):
        if y > 1981:
            w = w * figure(f"VAL-IDX-{y - 1}", "2019-07-01", known=known)[0]
        career = D(1) + D("0.012") * min(y - 1981, 20)
        e = (w * career).quantize(D(1))
        if y in (1987, 1991, 1992):       # at home with Piotr, with Ania
            e = D(0)
        if y == 2003:                     # unemployed from July
            e = (e / 2).quantize(D(1))
        out[y] = e
    return out


# R1: the world's own figure (the SIO decided it on its day, the papers state it), so it sees every figure
H_EARNINGS = _earnings(known=ALL_KNOWN)


def capital(earnings, retire_year, *, known):
    """SOC-5.2 and .3: 19.50 % a year (rounded, GEN-5.3), times the VAL-IDX products, exactly; rounded."""
    tot = ZERO
    for y, e in sorted(earnings.items()):
        c = ort(e * D("0.195"))
        f = D(1)
        for k in range(y + 1, retire_year):
            f *= figure(f"VAL-IDX-{k}", "2019-07-01", known=known)[0]
        tot += c * f
    return ort(tot)


def le(age, on, *, known):
    return figure(f"LE-{age}", on, known=known)[0]


def idx_years(start, upto):
    """SOC-6.1: the years whose 1 March indexation a pension started on `start` gets, up to `upto`."""
    out = []
    for y in range(start.year + 1, P(upto).year + 1):
        mar = date(y, 3, 1)
        if mar > P(upto):
            break
        if start < date(y - 1, 3, 1) or add_months(start, 12) <= mar:
            out.append(y)
    return out


def indexed(amount, start, on, *, known):
    """SOC-6: the pension on day `on`, each indexation rounded (SOC-6.2)."""
    a = D(amount)
    for y in idx_years(start, on):
        a = ort(a * figure(f"PEN-IDX-{y}", f"{y}-03-01", known=known)[0])
    return a


# R1: the world's own figure (the SIO decided it on its day, the papers state it), so it sees every figure
H_INITIAL = ort(capital(H_EARNINGS, 2019, known=ALL_KNOWN) / le(60, H_START, known=ALL_KNOWN))      # SOC-5.4: age 60 on 2019-07-01
ZB_INITIAL = ort(ZB_CAPITAL / le(65, ZB_START, known=ALL_KNOWN))                    # SOC-5.4: age 65 on 2018-12-01


def own(on, *, known):
    return indexed(H_INITIAL, H_START, on, known=known)


def zb(on, *, known):
    return indexed(ZB_INITIAL, ZB_START, on, known=known)


# R1: the world's own figure (the SIO decided it on its day, the papers state it), so it sees every figure
SURV_BASE = ort(D("0.85") * zb(DEATH, known=ALL_KNOWN))      # SOC-7 .1: 85 % of his pension on the day of the death


def surv(on, *, known):
    """SOC-7 .1: indexed as if it had started when his pension started; first month June 2024."""
    a = SURV_BASE
    for y in idx_years(ZB_START, on):
        if date(y, 3, 1) > DEATH:
            a = ort(a * figure(f"PEN-IDX-{y}", f"{y}-03-01", known=known)[0])
    return a


def min_pension(month_first, *, known):
    return figure("MIN-PENSION", month_first, known=known)[0]


def part_law(m, *, known):
    """SOC-7 v2 .3 for month m (its first day): 15 % of her own pension, cut so both ≤ 3 × MIN-PENSION."""
    if m < date(2025, 7, 1):
        return ZERO
    pct = D("0.15") if m < date(2027, 1, 1) else D("0.25")
    p = ort(pct * own(m, known=known))
    room = 3 * min_pension(m, known=known) - surv(m, known=known)
    return max(ZERO, min(p, room))


def part_sio(m, *, known):
    """D-D15: the SIO's part, with the cap read at 3 × 1 780.96 (the planted error)."""
    if m < date(2025, 7, 1):
        return ZERO
    p = ort(D("0.15") * own(m, known=known))
    return max(ZERO, min(p, 3 * SIO_CAP_USED - surv(m, known=known)))


def pay_day(y, m):
    """D-D09, D-D11: the SIO pays on the 5th, or the last business day before it."""
    d = date(y, m, 5)
    return d if CAL.is_business(d) else CAL.before(d)


# --------------------------------------------------------------------------- payroll on pensions (PIT-4 v2, SOC-2 v2)

def sio_month(gross_parts, cum_base):
    """One month of SIO payments to one person: health per payment, the month's advance on the month's base."""
    total = sum(gross_parts, ZERO)
    base = duk(total)  # PIT-4 .1: no contributions, no costs (PIT-2 .2)
    adv = duk(max(ZERO, scale(cum_base + base, "v3") - scale(cum_base, "v3") - PIT3["v3"]["reducing"] / 12))
    return base, adv


# --------------------------------------------------------------------------- bonds (TSB-1 .. TSB-3, TSB-7)

def _series():
    out = []
    y, m = 2019, 1
    while (y, m) <= (2024, 5):
        out.append((f"{y % 100:02d}{m:02d}", CAL.on_or_next(date(y, m, 10))))
        y, m = (y + 1, 1) if m == 12 else (y, m + 1)
    return out


SERIES = _series()           # D-D22: five bonds of each series, bought on these days, in Zbigniew's name
BONDS_EACH = 5


def r1(s, *, known):
    y, m = 2000 + int(s[:2]), int(s[2:])
    return figure(f"ISB-R1-{s}", date(y, m, 1), known=known)[0] / 100


def margin(s, *, known):
    y, m = 2000 + int(s[:2]), int(s[2:])
    return figure(f"ISB-MARGIN-{s}", date(y, m, 1), known=known)[0] / 100


def gazette_cpi(ref, on, *, known):
    return figure(f"CPI-YY-{ref}", on, known)[0]


def year_rate(s, p, k, cpi=None, *, known):
    """TSB-2: year 1 the first-year rate; later the CPI two months before the year's start month, as
    published on the year's first day, less 100, plus the margin; never below 0."""
    if k == 1:
        return r1(s, known=known)
    ys = add_months(p, 12 * (k - 1))
    ref = months_back(month_key(ys), 2)
    c = cpi(ref, ys) if cpi else gazette_cpi(ref, ys, known=known)
    return max(ZERO, (D(c) - 100) / 100 + margin(s, known=known))


def bond_value(s, p, on, cpi=None, *, known):
    """TSB-3: capitalised at each interest year's end, pro rata by days inside one; per bond, to the ort."""
    on = P(on)
    v, k = D(100), 1
    while k <= 10:
        ys, yn = add_months(p, 12 * (k - 1)), add_months(p, 12 * k)
        r = year_rate(s, p, k, cpi, known=known)
        if on >= yn:
            v = ort(v * (1 + r))
            k += 1
            continue
        return ort(v * (1 + r * days(ys, on) / days(ys, yn)))
    return v


def heir_base(s, p, *, known):
    """TSB-6 .2 with TSB-7: for Halina every bond she took in the division reads its value on the death day."""
    return bond_value(s, p, DEATH, known=known)


def redeem(s, p, request, n, base, *, known):
    """TSB-4 .2 and TSB-5: per bond the value on the request day less the fee (capped at value − base); the tax."""
    v = bond_value(s, p, request, known=known)
    fee = min(D("2.00"), max(ZERO, v - base))
    paid = v - fee
    tax = duk(D("0.19") * (paid - base) * n)
    return {"series": f"ISB{s}", "bonds": n, "value": v, "fee": fee, "base": base, "gross": paid * n, "tax": tax,
            "net": paid * n - tax}


def nth_business_after(d, n):
    for _ in range(n):
        d = CAL.on_or_next(d + timedelta(days=1))
    return d


# --------------------------------------------------------------------------- the loan and the insurer (D-D18 .. D-D20)
LOAN = {"principal": D("90000.00"), "monthly": D("0.0045"), "n": 120, "first": date(2020, 10, 5)}
LOAN_ROWS = schedule(LOAN["principal"], LOAN["monthly"], LOAN["n"])
INSURER_PAID_ON, BANK_REFUND_ON = date(2024, 7, 12), date(2024, 7, 16)


def loan_due(k):
    return CAL.on_or_next(add_months(LOAN["first"], k - 1))


LAST_BEFORE_DEATH = max(k for k in range(1, 121) if loan_due(k) <= DEATH)          # instalment 44, 2024-05-06
LOAN_AT_DEATH = LOAN_ROWS[LAST_BEFORE_DEATH - 1]["balance"]                       # D-D19: principal outstanding
JUNE_INSTALMENT = LOAN_ROWS[LAST_BEFORE_DEATH]["instalment"]                       # collected 2024-06-05, refunded
GROUP_LIFE = D("90000.00")                                                         # D-D21, paid 2024-06-28

# --------------------------------------------------------------------------- Ogrodowa 2/11 (D-D06, D-D25, D-D27, D-D28)
OLD_RENT = D("1780.00")              # the old lease, ended 2024-05-31 (D-D26)
NEW_RENT = D("1900.00")              # D-D25, from 2025-07-01; indexed each 1 March by CPI-AVG of the year before
FEES = {2024: D("420.00"), 2025: D("445.00"), 2026: D("468.00")}   # D-D27, paid on the 5th or the next business day
BOILER = (date(2026, 2, 16), date(2026, 2, 18), D("6840.00"))       # D-D28: issued, paid, amount
INVENTORY_FLAT = D("230000.00")      # D-D05: the agent's value at the death
AGREED_FLAT = D("165000.00")         # D-D07
VALUATION_2025 = (date(2025, 10, 6), D("240000.00"))               # D-D29
OFFER = (date(2026, 9, 15), D("201500.00"))                         # D-D30: for Halina's 13/16


def rent_on(d, *, known):
    """D-D25: 1 900.00 from July 2025; on each 1 March the rent times CPI-AVG of the year before / 100, to the ort."""
    r = NEW_RENT
    for y in range(2026, P(d).year + 1):
        if P(d) >= date(y, 3, 1):
            r = ort(r * figure(f"CPI-AVG-{y - 1}", f"{y}-03-01", known=known)[0] / 100)
    return r


def fee_day(y, m):
    return CAL.on_or_next(date(y, m, 5))


# --------------------------------------------------------------------------- the estate and the division

def estate_flat_costs():
    """Ogrodowa's fees paid after the death up to the division day (D-D07 lists them)."""
    out = []
    y, m = 2024, 6
    while True:
        d = fee_day(y, m)
        if d > DIVISION:
            break
        out.append((d, FEES[y]))
        y, m = (y + 1, 1) if m == 12 else (y, m + 1)
    return out


def bonds_total(on, *, known):
    return sum((bond_value(s, p, on, known=known) * BONDS_EACH for s, p in SERIES), ZERO)


def zb_2024_return(*, known):
    """Zbigniew's 2024 PIT (PIT-5 v2): five pensions January–May, the SIO's advances; the estate's refund."""
    cum, adv, inc = ZERO, ZERO, ZERO
    for m in range(1, 6):
        g = zb(date(2024, m, 1), known=known)
        base, a = sio_month([g], cum)
        cum += base
        adv += a
        inc += g
    base = duk(inc)
    tax = duk(max(ZERO, scale(base, "v3") - PIT3["v3"]["reducing"]))
    return {"income": inc, "base": base, "tax": tax, "advances": adv, "refund": adv - tax}


# R1: the world's own figure (the SIO decided it on its day, the papers state it), so it sees every figure
ZB_REFUND = zb_2024_return(known=ALL_KNOWN)["refund"]
ZB_REFUND_PAID = date(2025, 5, 21)      # D-D31
N_TARGET = D("206668.00")               # chosen so that 3/16 of it rounds down to 38 750 (A-D36, B-D27)


def death_day_balance(*, known):
    """The joint account at the end of 2024-05-17, set so the agreement's figures meet A-D36 and B-D27."""
    b_half = bonds_total(DIVISION, known=known) / 2
    k = sum((a for _, a in estate_flat_costs()), ZERO)
    return 2 * (N_TARGET - AGREED_FLAT - b_half + k - ZB_REFUND)


def division(*, known):
    """D-D07: the values Halina takes, the payouts (3/16 each, rounded down to whole dukats)."""
    bonds = bonds_total(DIVISION, known=known)
    c = death_day_balance(known=known)
    k = sum((a for _, a in estate_flat_costs()), ZERO)
    n_all = AGREED_FLAT + bonds / 2 + c / 2 + ZB_REFUND - k
    n_kuba = n_all - AGREED_FLAT
    child = (n_all * 3 / 16).quantize(D(1), rounding=ROUND_DOWN)
    kuba = (n_kuba * 3 / 16).quantize(D(1), rounding=ROUND_DOWN)
    return {"bonds": bonds, "bonds_half": bonds / 2, "account": c, "account_half": c / 2, "refund": ZB_REFUND,
            "costs": k, "N": n_all, "N_kuba": n_kuba, "child": child, "kuba": kuba, "flat_part": AGREED_FLAT * 3 / 16,
            "escrow": 3 * child + kuba}


# --------------------------------------------------------------------------- the joint account, day by day

SHOPS = ["Sklep Pod Lipą", "Piekarnia Złoty Kłos", "Apteka Rumianek", "Market Brzezie", "Drogeria Konwalia",
         "Warzywniak u Janka", "Stacja Olcha-Paliwa"]


def _spending(rng, y, m, extra):
    """Halina's own spending in a month: the house's bills, cash and card rows (seeded)."""
    rows = []
    rows.append((CAL.on_or_next(date(y, m, 15)), "STANDING ORDER", "Volta Energia", f"Leśna 4 energy {m:02d}/{y}", -D("385.00")))
    rows.append((CAL.on_or_next(date(y, m, 20)), "STANDING ORDER", "Telekom Lenia", f"phone {m:02d}/{y}", -D("59.00")))
    for day in (3, 17):
        d = CAL.on_or_next(date(y, m, day))
        rows.append((d, "CASH", "ATM Olcha Brzezie", "cash withdrawal", -D("500.00")))
    for _ in range(9 + rng.randint(0, 4)):
        d = date(y, m, rng.randint(1, end_of_month(date(y, m, 1)).day))
        amt = D(rng.randint(2400, 26000)) / 100
        rows.append((d, "CARD", rng.choice(SHOPS), "card ...2264", -amt))
    rows.append((CAL.on_or_next(date(y, m, 25)), "TRANSFER", "Opieka Dom sp.", f"home help {m:02d}/{y}", -extra))
    return rows


def sim(twist=True, *, known):
    """The joint account 2024-01-01 .. 2026-09-30; returns rows, redemptions and statements' facts.

    twist=False is the world without the death (for the `without:` keys): both pensions, the old lease
    goes on as the tenant gave notice anyway, no estate, no division, no rules D-HR1/D-HR2 money moves.
    """
    rng = random.Random(20240517)
    ev = []   # (date, order, type, counterparty, title, amount, tag)

    def add(d, typ, cp, title, amt, tag="", order=5):
        ev.append((d, order, typ, cp, title, D(amt), tag))

    # pensions and their payroll
    sio = {"H": {}, "Z": {}}     # (year) -> list of payments
    cum = {"H": {}, "Z": {}}

    def pension_pay(who, d, parts, label, month):
        y = d.year
        c = cum[who].setdefault(y, ZERO)
        prev = [p for p in sio[who].setdefault(y, []) if p["month"] == month]
        gross_all = sum(parts, ZERO) + sum((p["gross"] for p in prev), ZERO)
        base_all, adv_all = sio_month([gross_all], c - sum((p["base_inc"] for p in prev), ZERO))
        base_prev = sum((p["base_inc"] for p in prev), ZERO)
        adv_prev = sum((p["advance"] for p in prev), ZERO)
        g = sum(parts, ZERO)
        h = ort(g * D("0.09"))
        adv = adv_all - adv_prev
        base_inc = base_all - base_prev
        cum[who][y] = c + base_inc
        net = g - h - adv
        sio[who][y].append({"day": d, "month": month, "gross": g, "health": h, "advance": adv, "base_inc": base_inc, "net": net, "label": label})
        add(d, "CREDIT", "Social Insurance Office", label, net, f"pension-{who}", order=1)

    m = date(2024, 1, 1)
    while m <= date(2026, 9, 1):
        y, mo = m.year, m.month
        d = pay_day(y, mo)
        alive = (not twist) or m <= date(2024, 5, 1)
        if alive:
            pension_pay("Z", d, [zb(m, known=known)], f"pension {mo:02d}/{y} Z. SOWA", m)
            pension_pay("H", d, [own(m, known=known)], f"pension {mo:02d}/{y} H. SOWA", m)
        elif m == date(2024, 6, 1):
            pension_pay("H", d, [own(m, known=known)], f"pension {mo:02d}/{y} H. SOWA", m)
            pension_pay("H", date(2024, 6, 24), [surv(m, known=known) - own(m, known=known)], "survivor pension 06/2024, difference (decision 2024-06-14)", m)
        else:
            parts = [surv(m, known=known)]
            if m >= date(2025, 12, 1):
                parts.append(part_sio(m, known=known))
            label = f"survivor pension {mo:02d}/{y}"
            if m == date(2025, 12, 1):
                arrears = sum((part_sio(date(2025, k, 1), known=known) for k in range(7, 12)), ZERO)
                parts.append(arrears)
                label += " + arrears 07-11/2025 (decision 2025-11-20)"
            pension_pay("H", d, parts, label, m)
        m = add_months(m, 1)

    # Ogrodowa: the old lease to May 2024, fees, the new lease from July 2025
    m = date(2024, 1, 1)
    while m <= date(2026, 9, 1):
        y, mo = m.year, m.month
        add(fee_day(y, mo), "STANDING ORDER", "WM Ogrodowa 2", f"fee Ogrodowa 2/11 {mo:02d}/{y}", -FEES[y], "fee")
        if m <= date(2024, 5, 1):
            add(CAL.on_or_next(date(y, mo, 8)), "CREDIT", "Igor Mazur", f"rent {mo:02d}/{y} Ogrodowa 2/11", OLD_RENT, "rent-old")
        if not twist and date(2024, 6, 1) <= m:
            pass  # the tenant gave notice for May either way (D-D26); the flat stands empty
        m = add_months(m, 1)
    if twist:
        tenant_days = {}
        m = date(2025, 7, 1)
        while m <= date(2026, 9, 1):
            y, mo = m.year, m.month
            pd = CAL.on_or_next(date(y, mo, 7 + (mo % 3)))
            if (y, mo) == (2026, 1):
                pd = date(2025, 12, 30)     # January's rent paid early, before the New Year
            tenant_days[m] = pd
            add(pd, "CREDIT", "Oskar Wilczek", f"rent {mo:02d}/{y} Ogrodowa 2/11", rent_on(m, known=known), "rent")
            m = add_months(m, 1)
        add(BOILER[1], "TRANSFER", "Termo-Kłodzin sp.", "invoice FV/2026/0216 Ogrodowa 2/11 boiler", -BOILER[2], "boiler")

    # bonds bought (to May 2024) and the loan
    for s, p in SERIES:
        if p >= date(2024, 1, 1):
            add(p, "TRANSFER", "Treasury ISB", f"ISB{s} 5 bonds reg. 55-1904-2271", -D("500.00"), "bonds")
    for k in range(1, 121):
        dd = loan_due(k)
        if date(2024, 1, 1) <= dd <= date(2026, 9, 30):
            if twist and dd > date(2024, 6, 30):
                break
            add(dd, "DIRECT DEBIT", "Bank Buk", f"loan 3310-20 instalment {k}", -LOAN_ROWS[k - 1]["instalment"], "loan", order=2)
    if twist:
        add(BANK_REFUND_ON, "CREDIT", "Bank Buk", "loan 3310-20 repaid by TU Opoka; instalment 45 returned", JUNE_INSTALMENT, "loan-refund")
        add(date(2024, 6, 28), "CREDIT", "TU Opoka S.A.", "group life GZ-7741 benefit, insured Z. Sowa", GROUP_LIFE, "life")
        add(ZB_REFUND_PAID, "CREDIT", "Revenue Office Kłodzin", "PIT 2024 refund Z. SOWA (estate)", ZB_REFUND, "zb-refund")

    # spending
    m = date(2024, 1, 1)
    while m <= date(2026, 9, 1):
        extra = (D("2150.00") if m >= date(2025, 7, 1) else D("1100.00")) if (twist and m >= date(2024, 6, 1)) else D("600.00")
        for r in _spending(rng, m.year, m.month, extra):
            ev.append((r[0], 6, r[1], r[2], r[3], r[4], "spend"))
        m = add_months(m, 1)
    if twist:
        add(date(2024, 9, 20), "TRANSFER", "Kamieniarz Bazalt", "headstone Z. Sowa", -D("8600.00"), "spend")
        add(date(2026, 8, 3), "CARD", "Biuro Podróży Mewa-Tour", "card ...2264", -D("3480.00"), "spend")

    div = division(known=known) if twist else None
    if twist:
        add(DIVISION, "TRANSFER", "Notary Ewa Brzoza (deposit)", "estate of Z. Sowa: payments to the heirs, agreement 12.06.2025", -div["escrow"], "escrow")

    # taxes: Halina's returns, the rent advances (PIT-13)
    rets = {}

    def returns_of(year):
        pays = sio["H"].get(year, [])
        inc = sum((p["gross"] for p in pays), ZERO)
        adv = sum((p["advance"] for p in pays), ZERO)
        base = duk(inc)
        tax = duk(max(ZERO, scale(base, "v3") - PIT3["v3"]["reducing"]))
        return {"income": inc, "base": base, "tax": tax, "advances": adv, "due": tax - adv,
                "health": sum((p["health"] for p in pays), ZERO)}

    # the 2024 return needs 2024's pensions only: they are already in sio
    r24 = returns_of(2024)
    rets[2024] = r24
    if r24["due"] > 0:
        add(date(2025, 4, 28), "TRANSFER", "Revenue Office Kłodzin", "PIT 2024 H. SOWA", -r24["due"], "tax")
    elif r24["due"] < 0:
        add(date(2025, 5, 12), "CREDIT", "Revenue Office Kłodzin", "PIT 2024 refund H. SOWA", -r24["due"], "tax")

    rent_adv = []
    if twist:
        # PIT-13: Halina's 13/16 of the rent received, cumulative, advances by the 20th of the next month
        rec = sorted((pd, rent_on(mm, known=known)) for mm, pd in tenant_days.items())
        for yy in (2025, 2026):
            paid = ZERO
            cumr = ZERO
            for mo in range(1, 13):
                got = [a for pd, a in rec if pd.year == yy and pd.month == mo]
                if not got and cumr == 0:
                    continue
                cumr += sum(got, ZERO)
                tax = duk(cumr * D(HALINA_SHARE.numerator) / HALINA_SHARE.denominator * D("0.085"))
                adv = tax - paid
                due = CAL.on_or_next(date(yy + (mo == 12), 1 if mo == 12 else mo + 1, 20))
                if due > date(2026, 9, 30):
                    rent_adv.append({"month": f"{yy}-{mo:02d}", "rent": cumr, "tax": tax, "advance": adv, "due": due, "paid": None})
                    paid = tax
                    continue
                if adv > 0:
                    add(due, "TRANSFER", "Revenue Office Kłodzin", f"PIT-13 advance {mo:02d}/{yy} H. SOWA", -adv, "rent-tax")
                rent_adv.append({"month": f"{yy}-{mo:02d}", "rent": cumr, "tax": tax, "advance": adv, "due": due, "paid": due})
                paid = tax

    # D-HR1: Kuba's quarters; the 2025 return (filed 2026-04-13) is booked inside the day loop below
    quarters = []
    if twist:
        carry = ZERO
        for (qy, qn) in [(2025, 3), (2025, 4), (2026, 1), (2026, 2), (2026, 3)]:
            q0 = date(qy, 3 * qn - 2, 1)
            q1 = end_of_month(date(qy, 3 * qn, 1))
            rent = sum((rent_on(mm, known=known) for mm, pd in tenant_days.items() if q0 <= pd <= q1), ZERO)
            costs = sum((-e[5] for e in ev if e[6] in ("fee", "boiler") and q0 <= e[0] <= q1), ZERO)
            share = ort((rent - costs) * 3 / 16)
            due_base = date(qy + (qn == 4), 1 if qn == 4 else 3 * qn + 1, 10)
            due = CAL.on_or_next(due_base)
            send = share + carry
            entry = {"quarter": f"{qy}-Q{qn}", "rent": rent, "costs": costs, "share": share, "carried in": carry, "due": due}
            if send <= 0:
                entry["sent"] = ZERO
                carry = send
            else:
                entry["sent"] = send
                carry = ZERO
                if due <= date(2026, 9, 30):
                    if (qy, qn) == (2025, 4):
                        add(due, "TRANSFER", "Jakub Sowa", f"Ogrodowa {qy}-Q{qn} 3/16 to {KUBA_OLD}", -send, "kuba", order=3)
                        add(date(2026, 1, 14), "RETURN", "Jakub Sowa", "returned: account closed (transfer of 12.01.2026)", send, "kuba", order=3)
                        add(date(2026, 1, 15), "TRANSFER", "Jakub Sowa", f"Ogrodowa {qy}-Q{qn} 3/16 to {KUBA_NEW}", -send, "kuba", order=3)
                    else:
                        add(due, "TRANSFER", "Jakub Sowa", f"Ogrodowa {qy}-Q{qn} 3/16", -send, "kuba", order=3)
            quarters.append(entry)

    # the day loop: balances, the 2025 return, D-HR2
    ev.sort(key=lambda e: (e[0], e[1]))
    opening = ZERO
    pre = [e for e in ev if e[0] <= DEATH]
    if twist:
        opening = death_day_balance(known=known) - sum((e[5] for e in pre), ZERO)
    else:
        opening = sim(True, known=known)["opening"]
    remaining = {s: BONDS_EACH for s, _ in SERIES}
    pdays = dict(SERIES)
    rows, redemptions = [], []
    bal = opening
    in_flight = None
    pending = sorted(ev, key=lambda e: (e[0], e[1]))
    d = date(2024, 1, 1)
    i = 0
    r25_done = False
    while d <= date(2026, 9, 30):
        if twist and not r25_done and d == date(2026, 4, 13):
            r25 = returns_of(2025)
            rets[2025] = r25
            r25_done = True
            if r25["due"] > 0:
                pending.append((date(2026, 4, 30), 5, "TRANSFER", "Revenue Office Kłodzin", "PIT 2025 H. SOWA", -r25["due"], "tax"))
            elif r25["due"] < 0:
                pending.append((date(2026, 5, 20), 5, "CREDIT", "Revenue Office Kłodzin", "PIT 2025 refund H. SOWA", -r25["due"], "tax"))
            pending.sort(key=lambda e: (e[0], e[1]))
        while i < len(pending) and pending[i][0] == d:
            e = pending[i]
            bal += e[5]
            rows.append({"date": d, "type": e[2], "cp": e[3], "title": e[4], "amount": e[5], "balance": bal, "tag": e[6]})
            i += 1
        if in_flight and in_flight["paid"] == d:
            bal += in_flight["net"]
            rows.append({"date": d, "type": "CREDIT", "cp": "Treasury ISB", "title": f"redemption request {in_flight['request'].isoformat()}",
                         "amount": in_flight["net"], "balance": bal, "tag": "redeem"})
            in_flight = None
        if twist and d > DIVISION and bal < FLOOR and in_flight is None:
            req = CAL.on_or_next(d + timedelta(days=1))
            lines, got = [], ZERO
            for s, p in SERIES:
                while remaining[s] > 0 and got < REDEEM_MIN:
                    v = bond_value(s, p, req, known=known)
                    per = v - min(D("2.00"), max(ZERO, v - heir_base(s, p, known=known)))
                    need = int(((REDEEM_MIN - got) / per).to_integral_value(rounding=ROUND_CEILING))
                    n = min(remaining[s], need)
                    r = redeem(s, p, req, n, heir_base(s, p, known=known), known=known)
                    lines.append(r)
                    got += r["gross"]
                    remaining[s] -= n
                if got >= REDEEM_MIN:
                    break
            red = {"low_day": d, "low_balance": bal, "request": req, "paid": nth_business_after(req, 3), "lines": lines,
                   "net": sum((x["net"] for x in lines), ZERO), "tax": sum((x["tax"] for x in lines), ZERO)}
            redemptions.append(red)
            in_flight = red
        d += timedelta(days=1)
    assert i == len(pending), "events after the end"
    return {"opening": opening, "rows": rows, "redemptions": redemptions, "sio": sio, "returns": rets,
            "rent_adv": rent_adv, "quarters": quarters, "division": div, "remaining": remaining}


# --------------------------------------------------------------------------- the long view (D-HR3, story "assumptions")

LIFE_TABLE = {a: (D("0.0120") * D("1.105") ** (a - 68)).quantize(D("0.0001")) for a in range(68, 105)}
LIFE_TABLE[105] = D("1")
INFL = D("0.03")
FUND_MU, FUND_S, FUND_DF = D("0.05"), D("0.16"), 4
SPEND_2026 = D("54000.00")          # a year, in 2026 money (D-HR3)
SWEEP_ABOVE = D("15000.00")         # D-HR3
SEED = 20261001


def qs():
    return [LIFE_TABLE[a] for a in range(68, 106)]
