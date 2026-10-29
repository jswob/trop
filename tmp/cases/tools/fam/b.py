"""Household B: Piotr Sowa (sole trader) and Ola Wilk (employee), unmarried, their flat 60/40.

The facts B's documents state (each constant names its document), the home law run on them
(PIT-10, PIT-15, SOC-3, SOC-4, VAT, MTG, MOT, GEN-4) and a day-by-day simulation of 2025 that the
records are written from. Shared by gen_b.py (the dossier) and keys_b.py (the keys).

twist=True is the household as it is: the payer's bank recalls Arnika-Delta's payment of 12 Nov 2025
(the reversal); the export errors (a duplicated row, a dropped row, the half-month export) are the
writer's, in gen_b.py. twist=False removes the reversal. The keys' `without:` values are the household
without the reversal and with whole exports (no repeated row, no gap, no missing fee row), as the keys'
traces say: the grading side's proof that the twist moves 3 or more keys (README § Keys).
"""
import csv
import random
from datetime import date, timedelta
from decimal import Decimal, ROUND_DOWN

from fam.home import CAL, ROOT, ZERO, D, duk, ort, figure
from lib.dates import P, add_months, end_of_month
from lib.num import annuity

ONE = D("1")

# --------------------------------------------------------------------------- people (story)
PIOTR_BORN, OLA_BORN = date(1987, 11, 3), date(1990, 6, 21)
FLAT_SHARE = {"Piotr": D("0.60"), "Ola": D("0.40")}  # B-D04 the deed

# --------------------------------------------------------------------------- the business
BIZ_START = date(2022, 4, 11)        # B-D01 register entry; IT services (PIT-10 .4: 12 %)
FORM_C_NOTICE = date(2022, 5, 12)    # B-D02 election of form (c), holding for later years (PIT-10 .2)
SICKNESS_FROM = date(2022, 11, 1)    # B-D03 sickness insurance elected (SOC-3 .4)
VAT_GIVE_UP = date(2023, 1, 1)       # B-D05 notice giving up the small-business exemption (VAT-2 .4)
RATE_IT, RATE_ASSET = D("0.12"), D("0.10")  # PIT-10 .4


def ym_iter(a, b):
    y, m = a
    while (y, m) <= b:
        yield y, m
        y, m = (y + 1, 1) if m == 12 else (y, m + 1)


def prev_ym(y, m):
    return (y - 1, 12) if m == 1 else (y, m - 1)


def next_ym(y, m):
    return (y + 1, 1) if m == 12 else (y, m + 1)


def last_bd(y, m):
    return CAL.nth_last(y, m, 1)


def due20(y, m):
    """By the 20th of the month after (y, m), moved to the next business day (GEN-2 .2)."""
    ny, nm = next_ym(y, m)
    return CAL.on_or_next(date(ny, nm, 20))


def due25(y, m):
    ny, nm = next_ym(y, m)
    return CAL.on_or_next(date(ny, nm, 25))


# --------------------------------------------------------------------------- FX (GEN-4) and Lazur

def mid_for_event(day, *, known):
    """GEN-4 .1: the CBL mid rate of the last business day before the event's day."""
    v, src = figure("FX-FLR", CAL.before(day), known)
    return v, src


def mid_on(day, *, known):
    """B-HR4 .5: the CBL mid rate published for that day or, if none was, the last one before it."""
    return figure("FX-FLR", day if CAL.is_business(day) else CAL.before(day), known)


_LAZ = None


def lazur_buy(day, *, known):
    """Bank Lazur's buy rate of the day (market/lazur-fx-flr.csv; B-D23), as known on `known` (required)."""
    if day > P(known):
        raise LookupError(f"Lazur's buy rate of {day}: not known on {known}")
    global _LAZ
    if _LAZ is None:
        _LAZ = {}
        for ln in (ROOT / "market" / "lazur-fx-flr.csv").read_text(encoding="utf-8").splitlines()[2:]:
            d, b, s = ln.split(";")
            _LAZ[date(int(d[6:]), int(d[3:5]), int(d[:2]))] = D(b.replace(",", "."))
    return _LAZ[day]


def down_ort(x):
    return D(x).quantize(D("0.01"), rounding=ROUND_DOWN)


# --------------------------------------------------------------------------- sales (the register)

ARNIKA_NET = {2022: D("18000.00"), 2023: D("20000.00"), 2024: D("22000.00"), 2025: D("16000.00")}  # B-D06, B-D18
ARNIKA_RENAMED = date(2025, 5, 1)   # B-D18: Arnika Soft sp. merged into Arnika-Delta Group sp.
PROJECTS = [  # (issued, client, net DUK); terms 14 days
    (date(2022, 7, 15), "Studio Bez", "12000.00"),
    (date(2022, 9, 9), "Pracownia Ul", "15000.00"),
    (date(2022, 10, 14), "Studio Bez", "1300.00"),
    (date(2023, 5, 12), "Studio Bez", "9500.00"),
    (date(2023, 10, 13), "Pracownia Ul", "14200.00"),
    (date(2024, 3, 15), "Studio Bez", "12000.00"),
    (date(2024, 6, 14), "Pracownia Ul", "9800.00"),
    (date(2024, 9, 13), "Studio Bez", "8650.00"),
    (date(2024, 12, 6), "Kawiarnia Ziarno", "6000.00"),
    (date(2025, 6, 13), "Studio Bez", "8000.00"),
    (date(2025, 8, 14), "Kawiarnia Ziarno", "3500.00"),
    (date(2025, 10, 10), "Pracownia Ul", "2400.00"),
]
KESSEL_NET_FLR = D("1600.00")  # B-D21: a month's work, invoiced in florens, 30 days
CAR_SALE = (date(2025, 9, 12), "Auto Mlecz sp.", D("50400.00"))  # B-D15


def sales(*, known):
    """Every invoice Piotr issued, 2022-04 .. 2025-12, with VAT from the supply that crossed the limit."""
    out = []
    for y, m in ym_iter((2022, 4), (2025, 12)):
        d = last_bd(y, m)
        net = D("9000.00") if (y, m) == (2022, 4) else ARNIKA_NET[y]
        cl = "Arnika-Delta Group sp." if d >= ARNIKA_RENAMED else "Arnika Soft sp."
        out.append({"issued": d, "client": cl, "cur": "DUK", "net": net, "kind": "IT", "terms": 14})
        if y == 2025:
            out.append({"issued": d, "client": "Kessel Lenia sp.", "cur": "FLR", "net": KESSEL_NET_FLR, "kind": "IT", "terms": 30})
    for d, cl, net in PROJECTS:
        out.append({"issued": d, "client": cl, "cur": "DUK", "net": D(net), "kind": "IT", "terms": 14})
    out.append({"issued": CAR_SALE[0], "client": CAR_SALE[1], "cur": "DUK", "net": CAR_SALE[2], "kind": "asset", "terms": 0})
    order = {"Studio Bez": 0, "Pracownia Ul": 0, "Kawiarnia Ziarno": 0, "Auto Mlecz sp.": 0,
             "Arnika Soft sp.": 1, "Arnika-Delta Group sp.": 1, "Kessel Lenia sp.": 2}
    out.sort(key=lambda s: (s["issued"], order[s["client"]]))
    seq = {}
    # VAT-2: exempt in 2022 until the supply with which the year's supplies first go above the limit
    days_2022 = (date(2022, 12, 31) - BIZ_START).days + 1
    limit = ort(D("200000.00") * days_2022 / 365)  # VAT-2 .2
    cum, vat_on = ZERO, False
    for s in out:
        y = s["issued"].year
        seq[y] = seq.get(y, 0) + 1
        s["id"] = f"FV {seq[y]:03d}/{y}"
        s["due"] = s["issued"] + timedelta(days=s["terms"])
        if s["cur"] == "FLR":
            rate, src = mid_for_event(s["issued"], known=known)   # PIT-15 .3, VAT-3 .3, GEN-4
            s["mid"], s["mid_src"] = rate, src
            s["net_duk"] = ort(s["net"] * rate)       # GEN-4 .2
        else:
            s["net_duk"] = s["net"]
        if y == 2022 and not vat_on and s["kind"] != "asset":
            cum += s["net_duk"]                        # VAT-2 .3: asset sales not counted
            if cum > limit:
                vat_on = True
                s["crossing"] = {"limit": limit, "cumulative": cum, "days": days_2022}
        taxable = (y == 2022 and vat_on) or y >= 2023  # 2023 on: the exemption given up (B-D05)
        if taxable:
            s["vat"] = ort(s["net"] * D("0.22"))      # VAT-3 .2 (VAT-6 for the car)
            s["vat_duk"] = ort(s["vat"] * s["mid"]) if s["cur"] == "FLR" else s["vat"]
        else:
            s["vat"] = s["vat_duk"] = ZERO
        s["gross"] = s["net"] + s["vat"]
    return out, limit


# --------------------------------------------------------------------------- purchases (input VAT)

def purchases():
    """Purchase invoices with VAT Piotr deducts, in the month he receives them (VAT-5 .1, B-D story)."""
    r = random.Random(2203)
    out = [
        {"received": date(2022, 11, 8), "supplier": "Komputery Sowiniec sp.", "what": "laptop", "net": D("7000.00"), "share": ONE},
        {"received": date(2023, 3, 15), "supplier": "Auto Grab sp.", "what": "car LN 4471K", "net": D("80000.00"), "share": D("0.5")},
        {"received": date(2024, 5, 21), "supplier": "Komputery Sowiniec sp.", "what": "monitor", "net": D("1800.00"), "share": ONE},
    ]
    for y, m in ym_iter((2023, 4), (2025, 9)):
        net = D(r.randrange(48000, 76000)) / 100
        if (y, m) == (2025, 9):
            net = D("214.60")  # fuel to 12 September
        ny, nm = next_ym(y, m)
        out.append({"received": _nth_bd(ny, nm, 2), "supplier": "Stacja Paliw Tor sp.",
                    "what": f"fuel {y}-{m:02d}", "net": net, "share": D("0.5"), "fuel_month": (y, m)})
    for p in out:
        p["vat"] = ort(p["net"] * D("0.22"))
        p["gross"] = p["net"] + p["vat"]
        p["deduct"] = p["vat"] * p["share"]   # VAT-5 .2: 50 % for a car used privately too
    out.sort(key=lambda p: p["received"])
    for i, p in enumerate(out, 1):
        p["id"] = f"P{i:03d}"
    return out


def _nth_bd(y, m, n):
    d, seen = date(y, m, 1), 0
    while True:
        if CAL.is_business(d):
            seen += 1
            if seen == n:
                return d
        d += timedelta(days=1)


# --------------------------------------------------------------------------- SOC-3, SOC-4, PIT-10 (c)

def stage(y, m):
    """SOC-3 .1-.3: stage 1 to the end of the 6th month after the start's month; stage 2 the next 24."""
    s1_end = end_of_month(add_months(date(BIZ_START.year, BIZ_START.month, 1), 6))
    s2_end = end_of_month(add_months(date(BIZ_START.year, BIZ_START.month, 1), 30))
    d = date(y, m, 1)
    return 1 if d <= s1_end else (2 if d <= s2_end else 3)


def social(y, m, proj=None, *, known):
    """SOC-3: the month's social contributions (proj: assumed figures for future years)."""
    st = stage(y, m)
    z = {"stage": st, "base": ZERO, "pension": ZERO, "disability": ZERO, "accident": ZERO, "sickness": ZERO, "labour": ZERO}
    if st == 1:
        z["total"] = ZERO
        return z
    if st == 2:
        mw, _ = figure("MIN-WAGE", date(y, 1, 1), known)
        base = mw * D("0.30")
    else:
        aw = proj[y] if proj and y in proj else figure("AVG-WAGE-PROJ", date(y, 1, 1), known)[0]
        base = aw * D("0.60")
    z["base"] = base
    z["pension"] = ort(base * D("0.195"))
    z["disability"] = ort(base * D("0.08"))
    z["accident"] = ort(base * D("0.0167"))
    z["sickness"] = ort(base * D("0.0245")) if date(y, m, 1) >= SICKNESS_FROM else ZERO
    z["labour"] = ort(base * D("0.0245")) if st == 3 else ZERO
    z["total"] = z["pension"] + z["disability"] + z["accident"] + z["sickness"] + z["labour"]
    return z


def tier(revenue):
    """SOC-4 .3: the share of AVG-WAGE-Q4 by the year's revenue."""
    if revenue <= D("60000.00"):
        return D("0.60")
    if revenue <= D("300000.00"):
        return D("1.00")
    return D("1.80")


def awq4(y, m, proj=None, *, known):
    """SOC-4 .3: AVG-WAGE-Q4 in force on the month's first day."""
    if proj:
        for (fy, fm), v in sorted(proj.items(), reverse=True):
            if (y, m) >= (fy, fm):
                return v
    return figure("AVG-WAGE-Q4", date(y, m, 1), known)[0]


def pit_c(rev_it, rev_asset, ded, rate_it=RATE_IT):
    """PIT-10 .4 with PIT-15 .4: contributions off the highest rate first; parts' bases rounded; tax rounded."""
    left = ded
    parts = []
    for rev, rate in sorted([(rev_it, rate_it), (rev_asset, RATE_ASSET)], key=lambda x: -x[1]):
        use = min(rev, left)
        left -= use
        parts.append((duk(rev - use), rate))
    tax = duk(sum((b * r for b, r in parts), ZERO))
    return tax, parts, left


def business(twist=True, *, known):
    """Month by month 2022-04 .. 2025-12: revenue, social, health, the (c) tax, VAT; yearly settlements."""
    sl, limit = sales(known=known)
    pu = purchases()
    months = {}
    carry_vat = ZERO
    for y, m in ym_iter((2022, 4), (2025, 12)):
        ms = [s for s in sl if (s["issued"].year, s["issued"].month) == (y, m)]
        rev_it = sum((s["net_duk"] for s in ms if s["kind"] == "IT"), ZERO)
        rev_as = sum((s["net_duk"] for s in ms if s["kind"] == "asset"), ZERO)
        months[(y, m)] = {"sales": ms, "rev_it": rev_it, "rev_asset": rev_as, "revenue": rev_it + rev_as,
                          "social": social(y, m, known=known)}
    for y in (2022, 2023, 2024, 2025):
        cum, carry = ZERO, ZERO
        for m in range(1, 13):
            if (y, m) not in months:
                continue
            row = months[(y, m)]
            cum += row["revenue"]
            row["rev_ytd"] = cum
            pk = prev_ym(y, m)
            row["social_paid"] = months[pk]["social"]["total"] if pk in months else ZERO  # paid by the 20th, in this month
            tax, parts, left = pit_c(row["rev_it"], row["rev_asset"], row["social_paid"] + carry)
            row["carry_in"], row["tax"], row["parts"], row["carry_out"] = carry, tax, parts, left
            carry = left if m < 12 else ZERO  # PIT-10 .4: carried within the year only
            row["awq4"] = awq4(y, m, known=known)
            row["tier"] = tier(cum)
            row["health"] = ort(D("0.09") * row["tier"] * row["awq4"])  # SOC-4 .3, .4
            row["due_tax"] = row["due_soc"] = due20(y, m)
            # VAT-4: monthly returns once VAT is due (from the crossing supply, October 2022)
            if (y, m) >= (2022, 10):
                out_v = sum((s["vat_duk"] for s in row["sales"]), ZERO)
                in_v = sum((p["deduct"] for p in pu if (p["received"].year, p["received"].month) == (y, m)), ZERO)
                res = out_v - in_v - carry_vat
                row["vat_out"], row["vat_in"], row["vat_carry_in"] = out_v, in_v, carry_vat
                if res > 0:
                    row["vat_due"], carry_vat = duk(res), ZERO   # VAT-4 .2-.3, GEN-5.2
                else:
                    row["vat_due"], carry_vat = ZERO, duk(-res)
                row["vat_carry_out"] = carry_vat
                row["due_vat"] = due25(y, m)
            else:
                row["vat_due"] = ZERO
    settle = {}
    for y in (2022, 2023, 2024, 2025):
        ms = [(y, m) for m in range(1, 13) if (y, m) in months]
        total = months[ms[-1]]["rev_ytd"]
        owed = sum((ort(D("0.09") * tier(total) * months[k]["awq4"]) for k in ms), ZERO)  # SOC-4 .5
        paid = sum((months[k]["health"] for k in ms), ZERO)
        settle[y] = {"revenue": total, "tier": tier(total), "owed": owed, "paid": paid, "difference": owed - paid,
                     "due": CAL.on_or_next(date(y + 1, 4, 20))}
    return {"months": months, "settle": settle, "sales": sl, "limit": limit, "purchases": pu}


# --------------------------------------------------------------------------- the mortgage (B-D07 contract)

MTG_PRINCIPAL = D("420000.00")
MTG_DISBURSED = date(2022, 3, 18)
MTG_N = 300
MTG_MARGIN = D("2.10")
MTG_FIRST = date(2022, 4, 5)
HOLIDAYS = (date(2022, 8, 5), date(2022, 9, 5), date(2022, 11, 5), date(2022, 12, 5), date(2023, 2, 5))  # B-D08
FEE_UNTIL = date(2025, 3, 18)  # MTG-2 .2: 36 months from disbursement (GEN-3 .1); the contract's 2 % until then

_DUKOR = None


def dukor():
    global _DUKOR
    if _DUKOR is None:
        _DUKOR = [(date.fromisoformat(r["fixing_date"]), D(r["DUKOR_6M"]))
                  for r in csv.DictReader(open(ROOT / "market" / "dukor.csv", encoding="utf-8"))]
    return _DUKOR


def fixing_before(d, *, known):
    """The last DUKOR 6M fixing before d, as known on `known` (required): a fixing dated after `known` raises."""
    fx = max((x for x in dukor() if x[0] < d), key=lambda x: x[0])
    if fx[0] > P(known):
        raise LookupError(f"DUKOR 6M fixing of {fx[0]}: not known on {known}")
    return fx


def reset_days(upto):
    out = [MTG_DISBURSED]
    y = 2022
    while True:
        for m in (1, 7):
            d = date(y, m, 1)
            if MTG_DISBURSED < d <= upto:
                out.append(d)
        y += 1
        if date(y, 1, 1) > upto:
            break
    return sorted(out)


def mtg_rate(d, assume_after=None, *, known):
    """The rate (in %) for day d: DUKOR 6M of the last fixing before the latest reset day, plus the margin."""
    r = max(x for x in reset_days(d) if x <= d)
    if assume_after and r > assume_after[0]:
        return assume_after[1] + MTG_MARGIN, r, None
    fx = fixing_before(r, known=known)
    return fx[1] + MTG_MARGIN, r, fx[0]


class Mortgage:
    """B-D07: interest by days (actual/365) on the day's balance, rounded per instalment; annuity
    re-levelled at the first instalment after each reset and after each overpayment (same end);
    MTG-1 holidays: nothing paid, no interest for that month, later instalments a month on."""

    def __init__(self, holidays=HOLIDAYS, assume_after=None, *, known):
        self.known = known
        self.holidays = set(holidays)
        self.assume = assume_after
        self.slots = [add_months(MTG_FIRST, k) for k in range(MTG_N + len(self.holidays))]
        self.bal = MTG_PRINCIPAL
        self.cursor = MTG_DISBURSED
        self.accr = ZERO
        self.k = 0
        r, _, _ = mtg_rate(MTG_DISBURSED, known=known)
        self.rate = r
        self.pay = annuity(self.bal, r / 1200, MTG_N)
        self.last_reset = MTG_DISBURSED
        self.rows = []
        self.overpays = []

    def left(self):
        return sum(1 for s in self.slots[self.k:] if s not in self.holidays)

    def accrue_to(self, d):
        while self.cursor < d:
            r, rd, _ = mtg_rate(self.cursor, self.assume, known=self.known)
            nxt = min([x for x in reset_days(d + timedelta(days=400)) if x > self.cursor] + [d])
            nxt = min(nxt, d)
            n = (nxt - self.cursor).days
            self.accr += self.bal * r / 100 * n / 365
            self.cursor = nxt

    def overpay(self, d, amount):
        self.accrue_to(d)
        amount = D(amount)
        fee = ort(amount * D("0.02")) if d <= FEE_UNTIL else ZERO
        cap = ort(amount * D("0.03"))
        fee = min(fee, cap, ort((amount - fee) * self.rate_on(d) / 100))  # MTG-2 .2
        self.bal -= amount - fee
        self.pay = annuity(self.bal, self.rate_on(d) / 1200, self.left())
        rec = {"date": d, "paid": amount, "fee": fee, "principal": amount - fee, "balance": self.bal, "next_instalment": self.pay}
        self.overpays.append(rec)
        return rec

    def rate_on(self, d):
        return mtg_rate(d, self.assume, known=self.known)[0]

    def next_slot(self):
        return self.slots[self.k] if self.k < len(self.slots) else None

    def instalment(self):
        d = self.slots[self.k]
        self.accrue_to(d)
        r, rd, fx = mtg_rate(d - timedelta(days=1), self.assume, known=self.known)
        if d in self.holidays:
            row = {"slot": d, "holiday": True, "interest": ZERO, "principal": ZERO, "instalment": ZERO, "balance": self.bal,
                   "forgiven": ort(self.accr)}
            self.accr = ZERO
            self.k += 1
            self.rows.append(row)
            return row
        if rd != self.last_reset:
            self.pay = annuity(self.bal, r / 1200, self.left())
            self.last_reset = rd
        interest = ort(self.accr)
        self.accr = ZERO
        princ = self.pay - interest
        last = self.left() == 1
        if last or princ > self.bal:
            princ = self.bal
        self.bal -= princ
        row = {"slot": d, "holiday": False, "rate": r, "interest": interest, "principal": princ, "instalment": princ + interest,
               "balance": self.bal, "pay_day": CAL.on_or_next(d)}
        self.k += 1
        self.rows.append(row)
        return row

    def run_to(self, d):
        """Instalments whose slot is on or before d."""
        while self.k < len(self.slots) and self.slots[self.k] <= d:
            self.instalment()


# --------------------------------------------------------------------------- the car loan (B-D12)

CAR_LOAN = {"principal": D("60000.00"), "rate": D("0.095"), "n": 36, "first": date(2023, 4, 15)}
CAR_REPAID = date(2025, 9, 16)  # B-D16


def car_loan():
    pay = annuity(CAR_LOAN["principal"], CAR_LOAN["rate"] / 12, CAR_LOAN["n"])
    bal, rows = CAR_LOAN["principal"], []
    for k in range(CAR_LOAN["n"]):
        d = add_months(CAR_LOAN["first"], k)
        if d > CAR_REPAID:
            break
        i = ort(bal * CAR_LOAN["rate"] / 12)
        p = pay - i if k < CAR_LOAN["n"] - 1 else bal
        bal -= p
        rows.append({"slot": d, "pay_day": CAL.on_or_next(d), "interest": i, "principal": p, "instalment": p + i, "balance": bal})
    last = rows[-1]["slot"]
    days = (CAR_REPAID - last).days
    interest = ort(bal * CAR_LOAN["rate"] * days / 365)
    end = add_months(CAR_LOAN["first"], CAR_LOAN["n"] - 1)          # the agreed end, 2026-03-15
    ahead, b2 = ZERO, bal                                               # interest the schedule would still have charged
    for k in range(CAR_LOAN["n"]):
        if add_months(CAR_LOAN["first"], k) > CAR_REPAID:
            i2 = ort(b2 * CAR_LOAN["rate"] / 12)
            ahead += i2
            b2 -= (pay - i2) if k < CAR_LOAN["n"] - 1 else b2
    cap_rate = D("0.01") if add_months(CAR_REPAID, 12) < end else D("0.005")   # CC-2 .2
    fee = min(ort(bal * D("0.01")), ort(bal * cap_rate), ahead)          # B-D12: 1 %; CC-2 caps it
    payoff = {"date": CAR_REPAID, "balance": bal, "days": days, "interest": interest, "fee": fee, "total": bal + interest + fee,
              "instalment": pay, "agreed end": end, "fee by the agreement": ort(bal * D("0.01")), "fee cap": ort(bal * cap_rate),
              "interest ahead": ahead}
    return rows, payoff


CAR_INS_2025 = {"from": date(2025, 3, 15), "to": date(2026, 3, 14), "premium": D("2190.00"), "paid": date(2025, 3, 14),
                "fee": D("15.00")}  # B-D13
CAR_INS_2024_PREMIUM = D("2060.00")
ROAD_TAX = D("720.00")  # MOT-2 .1: 1 998 cm³ (B-D11)


def car_sale_legs():
    rows, payoff = car_loan()
    sale_day, buyer, net = CAR_SALE
    vat = ort(net * D("0.22"))  # VAT-6 .1
    unused = (CAR_INS_2025["to"] - sale_day).days  # 13 Sep 2025 .. 14 Mar 2026 (MOT-1 .2-.3)
    pol_days = (CAR_INS_2025["to"] - CAR_INS_2025["from"]).days + 1
    ins = down_ort(CAR_INS_2025["premium"] * unused / pol_days) - CAR_INS_2025["fee"]
    road = ort(ROAD_TAX / 12 * (12 - sale_day.month))  # MOT-2 .3
    return {"price": net + vat, "net": net, "vat": vat, "payoff": payoff, "insurance": ins, "unused_days": unused,
            "policy_days": pol_days, "road_tax": road, "tax": duk(duk(net) * RATE_ASSET)}


# --------------------------------------------------------------------------- Ola's pay (B-D25, B-D26)

OLA_GROSS = {2024: D("7200.00"), 2025: D("7600.00")}


def ola_payday(y, m):
    d = date(y, m, 28)
    while not CAL.is_business(d):
        d -= timedelta(days=1)
    return d


def ola_payments(year, bonus=ZERO):
    out = []
    for m in range(1, 13):
        g = OLA_GROSS[year] + (bonus if (year, m) == (2024, 10) else ZERO)
        d = ola_payday(year, m)
        if (year, m) == (2025, 2):
            d = date(2025, 3, 7)  # B-D26: February's pay late
        out.append({"date": d, "kind": "pay", "gross": g})
    return out


def oct_net(gross_oct, before_rows, *, known):
    """Ola's October 2024 net for a given gross, her January-September bases being known (PIT-4 v2, SOC-1, SOC-2)."""
    from fam.home import run_payer
    pays = ola_payments(2024)[:9] + [{"date": ola_payday(2024, 10), "kind": "pay", "gross": gross_oct}]
    rows, _ = run_payer(pays, known=known)
    return rows[9]["net"]


OLA_BONUS = D("3160.00")  # the October 2024 bonus (story: the payslip is lost); its gross is the only one giving that net


# --------------------------------------------------------------------------- 2025: the accounts, day by day

PIOTR_CIS = "LN45 2090 0017 5521 3380 0001"
PIOTR_SAV = "LN45 2090 0017 5521 3380 0002"
PIOTR_LAZ = "LN88 1020 0044 8813 FLR"
OLA_WRZ = "LN27 1140 2004 0000 3402 8830 1177"
CIS_OPEN_2025 = D("18412.37")
SAV_OPEN_2025 = D("52000.00")
from fam import e as E  # noqa: E402  the family's joins, stated once (E1)
ESTATE_PAYOUT = (E.PAYOUT_DAY, E.PAYOUTS["Piotr"]["amount"])   # B-D27, into Cis savings
FLOOR = {"v1": D("5000.00"), "v2": D("8000.00")}      # B-HR2
V2_STATED = date(2025, 4, 14)

# Arnika's payments of its invoices (Dec 2024 .. Nov 2025): (invoice month, paid on)
ARNIKA_PAID = {(2024, 12): date(2025, 1, 14), (2025, 1): date(2025, 2, 14), (2025, 2): date(2025, 3, 14),
               (2025, 3): date(2025, 4, 14), (2025, 4): date(2025, 5, 14), (2025, 5): date(2025, 6, 13),
               (2025, 6): date(2025, 7, 14), (2025, 7): date(2025, 8, 28), (2025, 8): date(2025, 9, 12),
               (2025, 9): date(2025, 10, 14), (2025, 10): date(2025, 11, 12), (2025, 11): date(2025, 12, 12)}
RECALL = (date(2025, 11, 13), (2025, 10))  # the twist: the payer's bank recalls the payment of 12 Nov
ARNIKA_REPAY = date(2026, 1, 8)  # B-D19 (promised)
KESSEL_PAID = {1: date(2025, 3, 3), 2: date(2025, 3, 28), 3: date(2025, 4, 30), 4: date(2025, 5, 29), 5: date(2025, 6, 30),
               6: date(2025, 7, 30), 7: date(2025, 9, 1), 8: date(2025, 9, 29), 9: date(2025, 10, 30), 10: date(2025, 12, 1)}
KESSEL_KEPT = (7, 8)  # July's and August's florens stay in the Lazur account (story)
PROJECT_PAID = {("Kawiarnia Ziarno", date(2024, 12, 6)): [(date(2025, 1, 9), None)],
                ("Studio Bez", date(2025, 6, 13)): [(date(2025, 7, 21), D("5000.00"))],
                ("Pracownia Ul", date(2025, 10, 10)): [(date(2025, 10, 24), D("-0.40"))]}  # None: in full; -x: short by x
BARTEK = {"lent": (date(2025, 2, 10), D("6000.00")), "back": (date(2025, 7, 1), D("3500.00")), "promised": date(2026, 3, 31)}
MISSING_ROW = (date(2025, 12, 9), D("249.00"), "CARD FEE", "Bank Cis S.A.", "yearly card fee")
FLAT_FEE = D("690.00")      # B-D09 housing association, 2025
ENERGY = D("175.00")
OLA_ORDER = D("1700.00")    # B-HR1
OLA_ORDER_FAILED = (2025, 3)
FLAT_INS_2025 = (date(2025, 3, 10), D("520.00"))  # B-D10, paid by Ola
MERCHANTS = ["Sklep Jagoda Kłodzin", "Market Groszek Kłodzin", "Apteka Mięta", "Kino Zorza", "Restauracja Dąb", "Drogeria Malwa", "Piekarnia Kłos",
             "Sklep internetowy Kram", "Stacja Paliw Iskra", "Sklep Sport Kret"]


def rule_days(year=2025):
    return [CAL.on_or_next(date(year, m, 26)) for m in range(1, 13)]


def simulate(twist=True, v2_from=V2_STATED, *, known):
    """2025 on the Cis current account, the Cis savings account and the Lazur floren account.

    Returns rows per account (booking date order, with balance after), the B-HR2 runs and the
    mortgage, which the rule's overpayments feed."""
    bz = business(twist, known=known)
    sl = bz["sales"]
    mtg = Mortgage(known=known)
    mtg.run_to(date(2024, 12, 31))
    rows = {"cis": [], "sav": [], "laz": []}
    bal = {"cis": CIS_OPEN_2025, "sav": SAV_OPEN_2025, "laz": ZERO}
    ev = {}  # day -> list of (prio, acct, desc, cp, title, amount, value_date)

    def add(day, prio, acct, desc, cp, title, amount, vd=None):
        ev.setdefault(day, []).append((prio, acct, desc, cp, title, D(amount), vd or day))

    # clients
    for s in sl:
        key = (s["issued"].year, s["issued"].month)
        if s["client"].startswith("Arnika") and key in ARNIKA_PAID:
            pd = ARNIKA_PAID[key]
            payer = "ARNIKA-DELTA GROUP SP." if pd >= ARNIKA_RENAMED else "ARNIKA SOFT SP."
            add(pd, 1, "cis", "TRANSFER IN", payer, f"{s['id']}", s["gross"])
            if twist and key == RECALL[1]:
                add(RECALL[0], 1, "cis", "REVERSAL", payer, f"recall by the payer's bank: {s['id']}", -s["gross"])
        for (cl, iss), pays in PROJECT_PAID.items():
            if s["client"] == cl and s["issued"] == iss:
                for pd, amt in pays:
                    a = s["gross"] if amt is None else (s["gross"] + amt if amt < 0 else amt)
                    add(pd, 1, "cis", "TRANSFER IN", cl.upper(), f"{s['id']}", a)
        if s["client"] == "Kessel Lenia sp." and s["issued"].month in KESSEL_PAID:
            pd = KESSEL_PAID[s["issued"].month]
            add(pd, 1, "laz", "incoming transfer", "KESSEL LENIA SP.", s["id"], s["gross"])
            if s["issued"].month not in KESSEL_KEPT:
                rate = lazur_buy(pd, known=known)
                duk_amt = ort(s["gross"] * rate)  # B-D23
                add(pd, 2, "laz", "FX sale", "own account at Bank Cis", f"FX sale {s['gross']:.2f} FLR at {rate} to {PIOTR_CIS[-4:]}", -s["gross"])
                add(CAL.on_or_next(pd + timedelta(days=1)), 1, "cis", "TRANSFER IN", "PIOTR SOWA (BANK LAZUR)",
                    f"FX {s['gross']:.2f} FLR at {rate}", duk_amt)
        if s["kind"] == "asset":
            add(s["issued"], 1, "cis", "TRANSFER IN", "AUTO MLECZ SP.", s["id"], s["gross"])
    # Ola's standing order (B-HR1); failed in March (her Wrzos statement)
    for m in range(1, 13):
        if (2025, m) != OLA_ORDER_FAILED:
            add(CAL.on_or_next(date(2025, m, 5)), 1, "cis", "TRANSFER IN", "ALEKSANDRA WILK", "flat share", OLA_ORDER)
    # fixed debits
    rows_car, payoff = car_loan()
    for r in rows_car:
        if r["pay_day"].year == 2025:
            add(r["pay_day"], 5, "cis", "LOAN REPAYMENT", "Bank Cis S.A.", "loan 5521-K instalment", -r["instalment"])
    add(CAR_REPAID, 5, "cis", "LOAN REPAYMENT", "Bank Cis S.A.", "loan 5521-K early repayment", -payoff["total"])
    for m in range(1, 13):
        add(CAL.on_or_next(date(2025, m, 10)), 4, "cis", "STANDING ORDER", "WM Wrzosowa 12", "building fee", -FLAT_FEE)
        add(CAL.on_or_next(date(2025, m, 12)), 4, "cis", "STANDING ORDER", "Energa Lenia", "electricity", -ENERGY)
    pu = bz["purchases"]
    for p in pu:
        if "fuel_month" in p and p["received"].year == 2025:
            add(CAL.on_or_next(date(2025, p["received"].month, 10)), 4, "cis", "TRANSFER OUT", "Stacja Paliw Tor sp.",
                f"fleet invoice {p['what']}", -p["gross"])
    for y, m in [(2024, 12)] + [(2025, k) for k in range(1, 12)]:
        row = bz["months"][(y, m)]
        add(row["due_soc"], 6, "cis", "TRANSFER OUT", "SIO", f"social contributions {y}-{m:02d}", -row["social"]["total"])
        add(row["due_soc"], 6, "cis", "TRANSFER OUT", "SIO", f"health contribution {y}-{m:02d}", -row["health"])
        if row["tax"] > 0:
            add(row["due_tax"], 6, "cis", "TRANSFER OUT", "Revenue Office", f"PIT advance {y}-{m:02d}", -row["tax"])
        if row["vat_due"] > 0:
            add(row["due_vat"], 6, "cis", "TRANSFER OUT", "Revenue Office", f"VAT {y}-{m:02d}", -row["vat_due"])
    st = bz["settle"][2024]
    if st["difference"] > 0:
        add(st["due"], 6, "cis", "TRANSFER OUT", "SIO", "health contribution settlement 2024", -st["difference"])
    add(CAR_INS_2025["paid"], 4, "cis", "TRANSFER OUT", "TU Bocian S.A.", "policy MTPL 88-4471 2025/26", -CAR_INS_2025["premium"])
    add(date(2025, 3, 20), 4, "cis", "TRANSFER OUT", "Gmina Kłodzin", "road tax 2025 LN 4471K", -ROAD_TAX)
    legs = car_sale_legs()
    add(date(2025, 9, 29), 1, "cis", "TRANSFER IN", "TU BOCIAN S.A.", "refund policy MTPL 88-4471", legs["insurance"])
    add(date(2025, 10, 15), 1, "cis", "TRANSFER IN", "GMINA KŁODZIN", "road tax refund LN 4471K", legs["road_tax"])
    add(BARTEK["lent"][0], 4, "sav", "TRANSFER OUT", "Bartłomiej Kos", "loan", -BARTEK["lent"][1])
    add(BARTEK["back"][0], 1, "cis", "TRANSFER IN", "BARTŁOMIEJ KOS", "part of the loan back", BARTEK["back"][1])
    # awkward rows: a split bill, a refund
    add(date(2025, 6, 9), 3, "cis", "CARD PAYMENT", "Restauracja Pod Dębem", "card ...3380", D("-412.00"), date(2025, 6, 7))
    add(date(2025, 6, 9), 1, "cis", "TRANSFER IN", "MAREK ŻUK", "dinner, my half", D("206.00"))
    add(date(2025, 4, 3), 3, "cis", "CARD PAYMENT", "Sklep Elektro Kos", "card ...3380", D("-349.99"), date(2025, 4, 2))
    add(date(2025, 4, 11), 1, "cis", "CARD REFUND", "Sklep Elektro Kos", "card ...3380 refund", D("349.99"), date(2025, 4, 10))
    add(MISSING_ROW[0], 4, "cis", MISSING_ROW[2], MISSING_ROW[3], MISSING_ROW[4], -MISSING_ROW[1])
    add(ESTATE_PAYOUT[0], 1, "sav", "TRANSFER IN", "KANCELARIA NOTARIALNA BRZOZA", "estate of Zbigniew Sowa: payout", ESTATE_PAYOUT[1])
    # card spending (seeded)
    rc = random.Random(2501)
    d = date(2025, 1, 1)
    while d <= date(2025, 12, 31):
        for _ in range(2):
            if rc.random() < 0.38:
                amt = D(rc.randrange(1500, 26000)) / 100
                bd = CAL.on_or_next(d + timedelta(days=1))
                if bd.year == 2025:
                    add(bd, 3, "cis", "CARD PAYMENT", rc.choice(MERCHANTS), "card ...3380", -amt, d)
        d += timedelta(days=1)

    runs, topups = [], []
    rdays = set(rule_days())
    seq = 0
    d = date(2025, 1, 1)
    while d <= date(2025, 12, 31):
        # the mortgage instalment due (paid on its pay day)
        while mtg.next_slot() and CAL.on_or_next(mtg.next_slot()) == d:
            r = mtg.instalment()
            if not r["holiday"]:
                add(d, 5, "cis", "TRANSFER OUT", "Bank Jodła S.A.", f"loan 7710-22 instalment {r['slot']:%m/%Y}", -r["instalment"])
        day_cis = sum((e[5] for e in ev.get(d, []) if e[1] == "cis"), ZERO)
        if bal["cis"] + day_cis < 0:  # B-HR5: back up to 1 000.00 from savings, rounded up to the next 100.00
            need = D("1000.00") - (bal["cis"] + day_cis)
            top = (need / 100).to_integral_value(rounding="ROUND_CEILING") * 100
            add(d, 0, "cis", "OWN TRANSFER", "PIOTR SOWA", f"from savings {PIOTR_SAV[-4:]}", top)
            add(d, 0, "sav", "OWN TRANSFER", "PIOTR SOWA", f"to {PIOTR_CIS[-4:]}", -top)
            topups.append((d, D(top)))
        for prio, acct, desc, cp, title, amt, vd in sorted(ev.get(d, []), key=lambda e: e[0]):
            bal[acct] += amt
            seq += 1
            rows[acct].append({"booked": d, "value": vd, "desc": desc, "cp": cp, "title": title, "amount": amt, "balance": bal[acct], "seq": seq})
        if d in rdays:
            run = hr2(d, bal["cis"], bal["laz"], sl, "v1" if d < v2_from else "v2", known=known)
            runs.append(run)
            if run["overpay"] > 0:
                rec = mtg.overpay(d, run["overpay"])
                run["fee"] = rec["fee"]
                bal["cis"] -= run["overpay"]
                seq += 1
                rows["cis"].append({"booked": d, "value": d, "desc": "TRANSFER OUT", "cp": "Bank Jodła S.A.", "title": "loan 7710-22 overpayment",
                                    "amount": -run["overpay"], "balance": bal["cis"], "seq": seq})
            if run["save"] > 0:
                bal["cis"] -= run["save"]
                bal["sav"] += run["save"]
                seq += 1
                rows["cis"].append({"booked": d, "value": d, "desc": "OWN TRANSFER", "cp": "PIOTR SOWA", "title": f"to savings {PIOTR_SAV[-4:]}",
                                    "amount": -run["save"], "balance": bal["cis"], "seq": seq})
                seq += 1
                rows["sav"].append({"booked": d, "value": d, "desc": "OWN TRANSFER", "cp": "PIOTR SOWA", "title": f"from {PIOTR_CIS[-4:]}",
                                    "amount": run["save"], "balance": bal["sav"], "seq": seq})
        assert bal["cis"] >= 0, (d, bal["cis"])
        d += timedelta(days=1)
    mtg.accrue_to(date(2026, 1, 1))
    return {"rows": rows, "bal": bal, "runs": runs, "mtg": mtg, "bz": bz, "topups": topups}


def hr2(d, cis_eod, flr, sl, version, *, known):
    """B-HR2 on day d: what we have, the reserve, the overpayment and the savings transfer."""
    rate = lazur_buy(d, known=known)
    flr_duk = down_ort(flr * rate)
    have = cis_eod + flr_duk
    ms = [s for s in sl if s["issued"].year == d.year and s["issued"].month == d.month and s["issued"] <= d]
    vat = sum((s["vat_duk"] for s in ms), ZERO)
    net12 = ort(sum((s["net_duk"] for s in ms), ZERO) * D("0.12"))
    reserve = FLOOR[version] + vat + net12 + D("4000.00")
    surplus = have - reserve
    over = save = ZERO
    if surplus > 0:
        over = min(D("3000.00"), surplus)
        save = surplus - over
        avail = cis_eod - D("1000.00")
        if over + save > avail:
            save = max(ZERO, min(save, avail - over))
            over = max(ZERO, min(over, avail))
    return {"day": d, "version": version, "cis": cis_eod, "flr": flr, "lazur_buy": rate, "flr_duk": flr_duk, "have": have,
            "invoices": [s["id"] for s in ms], "vat": vat, "net12": net12, "reserve": reserve, "surplus": surplus,
            "overpay": over, "save": save, "bound": (surplus > 0 and over + save < surplus)}


# --------------------------------------------------------------------------- 2022: the flat bought, the rent ended, Ola's studio

FLAT_DEED = {"date": date(2022, 3, 18), "price": D("600000.00"), "seller": "Jan Nowik", "notary_fee": D("4920.00"),
             "address": "ul. Wrzosowa 12/5, Kłodzin"}  # B-D04: shares 60/40; the notary's fee shared by the shares
OWN_FUNDS = {"Piotr": D("108000.00"), "Ola": D("72000.00")}  # B-D04: paid at the deed; the rest the mortgage (B-D07)
RENT = {"monthly": D("3200.00"), "deposit": D("6400.00"), "deposit_paid": date(2019, 9, 1), "from": date(2019, 9, 1),
        "notice": date(2022, 3, 21), "ends": date(2022, 6, 30), "returned": (date(2022, 7, 15), D("6050.00")),
        "kept": D("350.00")}  # B-D31 contract (Piotr the tenant), B-D32 notice and hand-over
STUDIO = {"bought": date(2019, 6, 14), "price": D("250000.00"), "tt": D("5000.00"), "notary": D("2100.00"),
          "sold": date(2021, 11, 22), "sale_price": D("312000.00"), "agent": D("7800.00")}  # B-D33, B-D34 (Ola's)
KITCHEN_QUOTE = (date(2022, 4, 20), D("45000.00"))  # B-D35, to be fitted in spring 2023, shared 60/40
VALUATION = (date(2024, 11, 5), D("640000.00"))     # B-D29


def flat_purchase():
    """TT-1 per share, the notary's fee by share, what each paid at the deed."""
    out = {}
    for who, sh in FLAT_SHARE.items():
        tt = duk(FLAT_DEED["price"] * sh * D("0.02"))         # TT-1 .1-.3
        fee = ort(FLAT_DEED["notary_fee"] * sh)
        out[who] = {"share": sh, "transfer tax": tt, "notary": fee, "price paid": OWN_FUNDS[who], "at the deed": OWN_FUNDS[who] + tt + fee}
    return out


def rent_overlap():
    """Rent for the days from the deed's day to the rent's end, each month's rent spread evenly over its days."""
    d0, d1 = FLAT_DEED["date"], RENT["ends"]
    tot, lines = ZERO, []
    y, m = d0.year, d0.month
    while (y, m) <= (d1.year, d1.month):
        first, last = date(y, m, 1), end_of_month(date(y, m, 1))
        a, b_ = max(first, d0), min(last, d1)
        days = (b_ - a).days + 1
        amt = ort(RENT["monthly"] * days / last.day)
        lines.append({"month": f"{y}-{m:02d}", "days": days, "of": last.day, "rent": amt})
        tot += amt
        y, m = next_ym(y, m)
    return tot, lines


def studio_tax(spent):
    """PIT-12: Ola's studio sale (2021), the exempt part by what she spent on her own housing."""
    s = STUDIO
    acq = s["price"] + s["tt"] + s["notary"]
    income = s["sale_price"] - s["agent"] - acq
    exempt = income * spent / s["sale_price"]
    base = duk(income - exempt)
    return {"income": income, "spent": spent, "exempt": exempt, "base": base, "tax": duk(base * D("0.19"))}


# --------------------------------------------------------------------------- Ola's Wrzos account (her statements)

OLA_WRZ_OPEN = {(2024, 10): D("4127.55"), (2025, 2): D("3412.18")}
OLA_SHOPS = ["Sklep Jagoda Kłodzin", "Kiosk Ruta", "Apteka Mięta", "Drogeria Malwa", "Kawiarnia Pod Lipą", "Księgarnia Kaganek", "Perfumeria Lilia"]


def ola_nets(year, bonus=ZERO, *, known):
    """Each pay's net as paid (cumulative withholding: a second pay in a month takes the month's rest)."""
    from fam.home import run_payer
    pays = ola_payments(year, bonus)
    out = []
    for i, p in enumerate(pays):
        rows, _ = run_payer(pays[:i + 1], known=known)
        month = f"{p['date'].year}-{p['date'].month:02d}"
        before = sum((o["net"] for o in out if o["month"] == month), ZERO)
        row = [r for r in rows if r["month"] == month][0]
        out.append({"for": (year, i + 1), "date": p["date"], "gross": p["gross"], "net": row["net"] - before, "month": month})
    return out


def expected_feb_2025_net(*, known):
    """February 2025's pay as B-D25 says it is paid (on 28 February), seen on 27 February."""
    from fam.home import run_payer
    pays = ola_payments(2025)[:2]
    pays[1] = dict(pays[1], date=ola_payday(2025, 2))
    rows, _ = run_payer(pays, known=known)
    return rows[1]["net"]


def ola_wrzos(*, known):
    """Ola's rows for 2024-10 and 2025-02 .. 2025-03 (the statements she kept)."""
    r = random.Random(1990)
    nets = {2024: ola_nets(2024, OLA_BONUS, known=known), 2025: ola_nets(2025, known=known)}
    out = {}
    for months, opening in (([(2024, 10)], OLA_WRZ_OPEN[(2024, 10)]), ([(2025, 2), (2025, 3)], OLA_WRZ_OPEN[(2025, 2)])):
        bal = opening
        for (y, m) in months:
            rows, ev = [], []
            for n in nets[y]:
                if (n["date"].year, n["date"].month) == (y, m):
                    ev.append((n["date"], 0, "TRANSFER IN", "MEWA SP.", f"pay {n['for'][1]:02d}/{y}", n["net"]))
            d5 = CAL.on_or_next(date(y, m, 5))
            ev.append((d5, 1, "STANDING ORDER", "PIOTR SOWA", "flat share", -OLA_ORDER))
            if (y, m) == FLAT_INS_2025[0].timetuple()[:2]:
                ev.append((FLAT_INS_2025[0], 2, "TRANSFER OUT", "TU Bocian S.A.", "policy flat WRZ-12/5 2025/26", -FLAT_INS_2025[1]))
            d = date(y, m, 1)
            while d.month == m:
                if CAL.is_business(d) and d.day not in (27, 28) and r.random() < 0.45:
                    ev.append((d, 3, "CARD PAYMENT", r.choice(OLA_SHOPS), "card ...1177", -(D(r.randrange(900, 9800)) / 100)))
                d += timedelta(days=1)
            for d, prio, typ, cp, title, amt in sorted(ev, key=lambda e: (e[0], e[1])):
                if typ == "STANDING ORDER" and bal < OLA_ORDER:
                    rows.append({"date": d, "type": "STANDING ORDER REJECTED", "cp": cp, "title": "flat share - insufficient funds",
                                 "amount": ZERO, "balance": bal})
                    continue
                if typ == "CARD PAYMENT" and bal + amt < D("150.00"):
                    continue
                bal += amt
                rows.append({"date": d, "type": typ, "cp": cp, "title": title, "amount": amt, "balance": bal})
            out[(y, m)] = {"opening": rows and (rows[0]["balance"] - rows[0]["amount"]), "rows": rows, "closing": bal}
    return out
