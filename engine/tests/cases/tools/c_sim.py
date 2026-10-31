"""Household C's accounts, 2024-01-01 .. 2026-09-30, built from c_facts and the house rules as the
household applied them. gen_c.py writes them as records; keys_c.py reads the records' facts from here.

Accounts: the joint account (Bank Jodła ...0001), the pot "Skarbonka" (...0002), Marta's own account
(...5148 0001), the workshop's account (to its closing) and Jan's account at Bank Lazur (FLR and DUK).

Departures from the house rules (facts, in the writers' notes): C-HR2's sale due on 2026-01-02 was made
on 2026-01-08; Jan applied C-HR3 v2 only from May 2025; the 2024 tenth was the household's estimate.
"""
import random
from datetime import date, timedelta
from decimal import ROUND_UP

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import c_facts as F
from c_law import CAL, VIS, ZERO, D, P, add_months, cbl_ref, coordinate, rff_amount, duk, end_of_month, half_up
from fam.home import ALL_KNOWN, run_payer
from fam import e as E  # the family's joins, stated once (E1, E2)

R = random.Random(F.SEED + 1)
T0, T1 = date(2024, 1, 1), date(2026, 9, 30)

OPEN = {"joint": D("14800.00"), "pot": D("6200.00"), "marta": D("47350.00"), "firm": D("18900.00")}
TITHE = {2024: (date(2024, 12, 16), D("21300.00")), 2025: (date(2025, 12, 15), D("5200.00"))}
GIFT_TO_JAN = (date(2024, 7, 1), D("40000.00"))
CAR = (date(2024, 7, 2), D("39500.00"), "Auto-Komis Rybak")
INHERIT = (E.PAYOUT_DAY, E.PAYOUTS["Marta"]["amount"])   # the estate payout, stated once (fam/e.py); C-D30
LOAN = {"date": date(2025, 6, 20), "amount": D("30000.00"), "repay": D("1000.00"), "forgiven": date(2026, 9, 17)}
START_DUK = D("4424.50")                                 # joint -> Lazur DUK, 2024-06-28
START_FX = (date(2024, 7, 1), D("1000.00"))              # Jan buys his first florens
VAT_REFUND_DAY = date(2024, 8, 29)                       # the June return filed 2024-07-15 (C-D12)
HEALTH_REFUND_DAY = date(2025, 5, 14)                    # SOC-4 .5 settlement filed 2025-04-14 (C-D13)
OVERPAY = [(date(2025, 6, 24), D("30000.00"))]          # from the joint account
EXTRA_FROM = date(2025, 7, 16)
EXTRA = D("800.00")
POT_LUMP = (date(2026, 7, 20), D("252000.00"))          # a lump overpayment from the pot
SWEEP = (date(2024, 6, 28), D("55000.00"))              # joint -> pot after the workshop closed
LEAVE_MONTHS = list(F.months("2025-09", "2026-08"))
LEAVE_TRANSFER = D("5000.00")
DEC_TOPUP = (date(2025, 12, 1), D("8000.00"))
NURSERY = D("1150.00")
CAR_INS = D("1480.00")
PURCHASES = [  # (date, account, payee, title, amount)
    (date(2025, 2, 6), "joint", "Elektro-Dom Zabór", "pralka", D("2899.00")),
    (date(2025, 4, 10), "flr", "Velohaus Brandt Ostrel", "Fahrrad", D("600.00")),
    (date(2025, 12, 5), "joint", "Meble Kalina", "sofa", D("3400.00")),
    (date(2025, 12, 20), "joint", "Komputery Bit", "laptop", D("4199.00")),
    (date(2026, 3, 14), "joint", "Ogród i Dom Zabór", "domek narzędziowy", D("2650.00")),
    (date(2026, 6, 6), "joint", "Płytki Zabór", "płytki", D("1400.00")),
    (date(2026, 6, 6), "joint", "Płytki Zabór", "płytki", D("1400.00")),
]
FBO_REPAY = [(date(2026, m, 15), D("325.00")) for m in range(1, 7)]   # FAM-2: the FBO's decision, 6 instalments
FBO_STOPPED = ("2025-12", "2026-08")       # the FBO paid nothing for these months (its decision of 2025-12-08)

# --------------------------------------------------------------------------- helpers

_LZ = None


def lazur_table(*, known):
    """Bank Lazur's FLR table: {date: (buy, sell)}, the days up to `known` (required)."""
    global _LZ
    if _LZ is None:
        _LZ = {}
        for ln in (VIS / "market" / "lazur-fx-flr.csv").read_text(encoding="utf-8").splitlines()[2:]:
            d, b, s = ln.split(";")
            _LZ[date(int(d[6:]), int(d[3:5]), int(d[:2]))] = (D(b.replace(",", ".")), D(s.replace(",", ".")))
    return {d: v for d, v in _LZ.items() if d <= P(known)}


def rate_month_ok(k, *, known):
    """C-HR2 .2: Bank Lazur's buy rate at or above 4.1100 on every day of the month it published one."""
    vals = [v[0] for d, v in lazur_table(known=known).items() if F.ym(d) == k]
    return bool(vals) and min(vals) >= D("4.1100")


def first_bd(k):
    return CAL.on_or_next(F.first(k))


class Book:
    """One account's rows: events added in any order, balances run by date then by priority."""

    def __init__(self, name, opening):
        self.name, self.opening, self.ev = name, opening, []

    def add(self, d, amount, kind, cp, title, pri=5, **extra):
        self.ev.append({"date": P(d), "amount": D(amount), "type": kind, "cp": cp, "title": title, "pri": pri, **extra})

    def run(self, check=True):
        bal = self.opening
        rows = []
        for e in sorted(self.ev, key=lambda e: (e["date"], e["pri"])):
            bal += e["amount"]
            rows.append({**e, "balance": bal})
            if check:
                assert bal >= 0, (self.name, e["date"], bal)
        return rows


def balance_on(rows, opening, d):
    b = opening
    for r in rows:
        if r["date"] > d:
            break
        b = r["balance"]
    return b

# --------------------------------------------------------------------------- the workshop (to its closing)


def firm(*, known):
    sales, costs = F.workshop_books()
    months6 = list(F.months("2024-01", "2024-06"))
    mr = F.workshop_months(sales, costs, months6, "2024-06", known=known)
    dec = F.workshop_months(sales, costs, ["2023-12"], known=known)["2023-12"]
    vat = F.vat_returns(sales, costs, ["2023-12"] + months6)
    b = Book("firm", OPEN["firm"])
    after = []   # what fell due after the account closed (paid from the joint account)
    close = date(2024, 6, 28)

    def put(d, *a):
        (b.add(d, *a) if d <= close else after.append((d,) + a))
    for s in sales:
        if T0 <= s["paid"]:
            put(s["paid"], s["net"] + s["vat"], "PRZELEW PRZYCH.", s["client"], f"zapłata {s['no']}", 1)
    for c in costs:
        if T0 <= c["paid"]:
            put(c["paid"], -(c["net"] + c["vat"]), "PRZELEW WYCH.", c["supplier"], f"zapłata {c['no']}", 3)
    soc_paid = [(F.due20("2023-12"), dec["soc3"])] + [(F.due20(k), mr[k]["soc3"]) for k in months6]
    adv = F.pit10a_advances(mr, soc_paid)
    pay = [("2023-12", F.due20("2023-12"), dec["soc3"], dec["health"], D("2914.00"))]
    for a, k in zip(adv, months6):
        pay.append((k, F.due20(k), mr[k]["soc3"], mr[k]["health"], a["advance"]))
    for k, d, s3, h, a in pay:
        put(d, -s3, "PRZELEW WYCH.", "SIO", f"składki społeczne {k[5:]}/{k[:4]}", 2)
        put(d, -h, "PRZELEW WYCH.", "SIO", f"składka zdrowotna {k[5:]}/{k[:4]}", 2)
        if a:
            put(d, -a, "PRZELEW WYCH.", "Urząd Skarbowy Zabór", f"zaliczka PIT {k[5:]}/{k[:4]}", 2)
    for v in vat:
        if v["result"] > 0:
            put(P(v["due"]), -v["result"], "PRZELEW WYCH.", "Urząd Skarbowy Zabór", f"VAT {v['month'][5:]}/{v['month'][:4]}", 2)
    for k in months6:
        b.add(F.bd_on_or_before(date(2024, int(k[5:]), 10)), -D("6500.00"), "PRZELEW WYCH.", "Marta i Jan Wrona",
              "wypłata właściciela", 4, to="joint")
        if k != "2024-06":
            b.add(F.bd_on_or_before(end_of_month(F.first(k))), -D("19.00"), "OPŁATA", "Bank Jodła", "prowadzenie rachunku", 6)
    # the last VAT return (June 2024) asked for its excess back (VAT-4 .2, VAT-4 v1 .4); SOC-4 .5's settlement
    jun = [v for v in vat if v["month"] == "2024-06"][0]
    after.append((VAT_REFUND_DAY, -jun["result"], "PRZELEW PRZYCH.", "Urząd Skarbowy Zabór", "zwrot VAT 06/2024", 1))
    inc = sum((m["income"] for m in mr.values()), ZERO)
    hdue = max(half_up(inc * D("0.09")), sum((m["health floor"] for m in mr.values()), ZERO))
    hpaid = sum((m["health"] for m in mr.values()), ZERO)
    after.append((HEALTH_REFUND_DAY, hpaid - hdue, "PRZELEW PRZYCH.", "SIO", "zwrot nadpłaty składki zdrowotnej 2024", 1))
    last = b.run()[-1]["balance"]
    b.add(close, -last, "PRZELEW WYCH.", "Marta i Jan Wrona", "zamknięcie rachunku — przelew salda", 9, to="joint")
    return {"health due": hdue, "health paid": hpaid, "rows": b.run(), "sales": sales, "costs": costs, "months": mr, "dec23": dec, "vat": vat, "advances": adv,
            "after": after, "closing": last, "soc paid": soc_paid}

# --------------------------------------------------------------------------- benefits


def works(k):
    """T-11: Marta paid for work or parental pay in month k."""
    return k not in LEAVE_MONTHS


PART2_FROM = "2024-12"   # T-20 .2: Part II has effect from the first day of the month after 2024-11-28


def benefits(*, known):
    """T-12 from December 2024; before it each state pays under its own law alone (RFF-1 .1, FAM-1)."""
    out = {}
    for k in F.months("2024-07", "2026-09"):
        if k < PART2_FROM:
            out[k] = {"first": None, "Lenia": D("650.00"), "Rosvald": rff_amount(k, known=known)}
        else:
            out[k] = coordinate(k, works(k), True, known=known)
    return out


def rff_payments(ben, *, known):
    def lastbd(y, m):
        return F.bd_on_or_before(end_of_month(date(y, m, 1)), F.RCAL)

    def q(ks):
        return sum((ben[k]["Rosvald"] for k in ks), ZERO)
    old_sep = coordinate("2025-09", True, True, known=known)["Rosvald"]
    out = [(date(2024, 12, 10), q(["2024-07", "2024-08", "2024-09"]), "Q3/2024"),
           (lastbd(2025, 1), q(["2024-10", "2024-11", "2024-12"]), "Q4/2024"),
           (lastbd(2025, 4), q(["2025-01", "2025-02", "2025-03"]), "Q1/2025"),
           (lastbd(2025, 7), q(["2025-04", "2025-05", "2025-06"]), "Q2/2025"),
           (lastbd(2025, 10), ben["2025-07"]["Rosvald"] + ben["2025-08"]["Rosvald"] + old_sep, "Q3/2025"),
           (date(2025, 12, 19), ben["2025-09"]["Rosvald"] - old_sep, "Korrektur 09/2025"),
           (lastbd(2026, 1), q(["2025-10", "2025-11", "2025-12"]), "Q4/2025"),
           (lastbd(2026, 4), q(["2026-01", "2026-02", "2026-03"]), "Q1/2026"),
           (lastbd(2026, 7), q(["2026-04", "2026-05", "2026-06"]), "Q2/2026")]
    return out, old_sep


def fbo_payments():
    """The FBO's payments to the joint account, for each month by its last day (FAM-1 .5)."""
    out = []
    for k in F.months("2024-01", "2026-09"):
        if FBO_STOPPED[0] <= k <= FBO_STOPPED[1]:
            continue
        out.append((F.bd_on_or_before(end_of_month(F.first(k))), D("650.00"), k))
    return out

# --------------------------------------------------------------------------- the mortgage and C-HR3


def extras(pot_extras=True, upto=T1):
    ex = list(OVERPAY)
    if pot_extras:
        d = EXTRA_FROM
        while d <= upto:
            ex.append((CAL.on_or_next(d), EXTRA))
            d = add_months(d, 1)
        ex.append(POT_LUMP)
    return sorted(ex)


def instalments_by_month():
    rows, inst = F.mortgage(extras(), upto=T1)
    return {F.ym(r["date"]): r for r in rows if r["kind"] == "instalment"}, rows


def hr3(version_fn):
    log = F.travel_log()
    inst, _ = instalments_by_month()
    return {k: F.hr3_due(k, log, inst[k]["amount"], version_fn(k)) for k in F.months("2024-07", "2026-08")}

# --------------------------------------------------------------------------- Bank Lazur


def lazur(*, known):
    slips = F.jan_slips(known=known)
    ben = benefits(known=known)
    rff, old_sep = rff_payments(ben, known=known)
    tab = lazur_table(known=known)
    from c_law import assessment
    a2425 = assessment([s for s in slips if s["tax year"] == "2024/25"], "2024/25", 2, known=known)
    dues = hr3(F.hr3_version_as_jan)
    ev = {}

    def at(d, *e):
        ev.setdefault(d, []).append(e)
    for s in slips:
        at(s["paid"], 1, "WYNAGRODZENIE", "Montagewerk Ostrel AG", f"Lohn {s['work'][5:]}/{s['work'][:4]}", s["net"])
    for d, a, t in rff:
        at(d, 1, "PRZELEW PRZYCH.", "Rosvald Family Fund", f"Kindergeld {t}", a)
    at(F.ASSESS_2425["paid"], 1, "PRZELEW PRZYCH.", "Rosvald Tax Board", "Erstattung Steuerjahr 2024/25", a2425["refund"])
    for d, acct, payee, title, amt in PURCHASES:
        if acct == "flr":
            at(d, 4, "KARTA", payee, title, -amt)
    for r in F.travel_log():
        d = r["date"]
        if r["work"]:
            at(d, 5, "KARTA", "Kantine Montagewerk", "Mittagessen", -D("11.80"))
            if d.weekday() == 0:
                at(d, 5, "KARTA", "Tankhof Ostrel", "Diesel", -(D(58 + (d.toordinal() * 7) % 17) + D("0.40")))
        if d in F.OUTINGS:
            at(d, 5, "KARTA", "Markthalle Ostrel", "Einkauf", -D(140 + (d.toordinal() * 3) % 60))
    share_day = {CAL.on_or_next(add_months(F.first(k), 1).replace(day=10)): (v["due"], k) for k, v in dues.items()}
    rows, sales, month_end = [], [], {}
    sell0 = tab[START_FX[0]][1]
    bal = START_FX[1]
    rows.append({"date": START_FX[0], "type": "KUPNO WALUTY", "cp": "Jan Wrona (DUK)", "title": f"kupno {START_FX[1]} FLR po {sell0}",
                 "amount": START_FX[1], "balance": bal, "rate": sell0})
    sold_q = set()
    late = None
    for d in F.days(date(2024, 7, 1), T1):
        todays = []
        if d in tab and d == first_bd(F.ym(d)) and d > date(2024, 7, 31):
            pk = F.ym(d - timedelta(days=d.day))
            q = (d.year, (d.month - 1) // 3)
            if rate_month_ok(pk, known=known) and q not in sold_q and bal > D("1500.00"):
                if d == date(2026, 1, 2):
                    late = q
                else:
                    todays.append(("C-HR2 .2", bal - D("1500.00")))
                    sold_q.add(q)
            elif month_end.get(pk, ZERO) > D("14000.00"):
                todays.append(("C-HR2 .3", bal - D("6000.00")))
        if late and d == date(2026, 1, 8):
            todays.append(("C-HR2 .2", bal - D("1500.00")))
            sold_q.add(late)
            late = None
        for rule, amt in todays:
            if amt <= 0:
                continue
            r = tab[d][0]
            dk = half_up(amt * r)
            bal -= amt
            rows.append({"date": d, "type": "SPRZEDAŻ WALUTY", "cp": "Skarbonka " + F.ACC_POT, "title": f"sprzedaż {amt} FLR po {r}",
                         "amount": -amt, "balance": bal, "rate": r, "duk": dk, "rule": rule})
            sales.append({"date": d, "rule": rule, "flr": amt, "rate": r, "duk": dk, "to": "pot"})
        if d in share_day:
            due, k = share_day[d]
            r = tab[d][0]
            fl = (due / r).quantize(D("0.01"), rounding=ROUND_UP)
            bal -= fl
            rows.append({"date": d, "type": "SPRZEDAŻ WALUTY", "cp": "Marta i Jan Wrona " + F.ACC_JOINT,
                         "title": f"udział {k[5:]}/{k[:4]}: {due} DUK po {r}", "amount": -fl, "balance": bal, "rate": r,
                         "duk": due, "rule": "C-HR2 .1"})
            sales.append({"date": d, "rule": "C-HR2 .1", "flr": fl, "rate": r, "duk": due, "to": "joint", "for": k})
        for e in sorted(ev.get(d, []), key=lambda e: e[0]):
            _, t, cp, ti, a = e
            bal += a
            rows.append({"date": d, "type": t, "cp": cp, "title": ti, "amount": a, "balance": bal})
        assert bal >= 0, (d, bal)
        if d == end_of_month(d):
            month_end[F.ym(d)] = bal
    db = Book("lazur DUK", ZERO)
    db.add(date(2024, 6, 28), START_DUK, "PRZELEW PRZYCH.", "Marta i Jan Wrona " + F.ACC_JOINT, "na start", 1)
    db.add(START_FX[0], -half_up(START_FX[1] * sell0), "KUPNO WALUTY", "Jan Wrona (FLR)", f"kupno {START_FX[1]} FLR po {sell0}", 3)
    db.add(GIFT_TO_JAN[0], GIFT_TO_JAN[1], "PRZELEW PRZYCH.", "Marta Wrona " + F.ACC_MARTA, "darowizna", 1)
    db.add(CAR[0], -CAR[1], "PRZELEW WYCH.", CAR[2], "zakup samochodu, umowa z 2024-07-02", 3)
    duk_rows = db.run()
    return {"flr": rows, "duk": duk_rows, "sales": sales, "rff": rff, "old_sep": old_sep, "a2425": a2425, "slips": slips,
            "month_end": month_end, "dues": dues, "benefits": ben}

# --------------------------------------------------------------------------- Marta's loan (C-HR7)


def loan(*, known):
    """Daily interest at CBL-REF/365 on what is owed at the day's start, added at each month's end and on the
    debt's last day (to the ort, half up); 1 000.00 repaid on the 10th (the next business day) from August 2025;
    released at the start of 2026-09-17."""
    owed = LOAN["amount"]
    acc = ZERO
    led = [{"date": LOAN["date"], "kind": "paid out", "amount": owed, "owed": owed}]
    pays = []
    d = date(2025, 8, 10)
    while d < LOAN["forgiven"]:
        pays.append(CAL.on_or_next(d))
        d = add_months(d, 1)
    day = LOAN["date"]
    end = LOAN["forgiven"] - timedelta(days=1)
    while day <= end:
        rate, _ = cbl_ref(day, known=known)
        acc += owed * rate / 100 / 365
        if day in pays:
            owed -= LOAN["repay"]
            led.append({"date": day, "kind": "repaid", "amount": -LOAN["repay"], "owed": owed})
        if day == end_of_month(day) or day == end:
            i = half_up(acc)
            owed += i
            led.append({"date": day, "kind": "interest", "amount": i, "owed": owed})
            acc = ZERO
        day += timedelta(days=1)
    return {"ledger": led, "repayments": pays, "released": owed}

# --------------------------------------------------------------------------- the joint account, the pot, Marta's own


def spend(b):
    """Everyday card spending from the joint account."""
    shops = [("Supermarket Zorza", "zakupy"), ("Sklep Pod Kasztanem", "zakupy"), ("Piekarnia Kłos", "pieczywo"),
             ("Apteka Rumianek", "apteka"), ("Stacja Paliw Kmieć", "paliwo"), ("Drogeria Lilia", "drogeria")]
    for d in F.days(T0, T1):
        if d.weekday() in (1, 4):
            b.add(d, -D(R.randint(14000, 26000)) / 100, "KARTA", *shops[R.randrange(2)], 7)
        if d.weekday() == 5:
            b.add(d, -D(R.randint(9000, 21000)) / 100, "KARTA", *shops[0], 7)
        if d.weekday() == 2 and R.random() < 0.6:
            b.add(d, -D(R.randint(800, 3500)) / 100, "KARTA", *shops[2], 7)
        if d.day in (7, 21) and R.random() < 0.7:
            b.add(d, -D(R.randint(2500, 12000)) / 100, "KARTA", *shops[R.choice([3, 5])], 7)
        if d.weekday() == 3 and (d.day <= 7 and d < date(2024, 7, 1) or 14 < d.day <= 21):
            b.add(d, -D(R.randint(18000, 26000)) / 100, "KARTA", *shops[4], 7)


def interest_own(b):
    """Bank Jodła pays 1.20 % a year on Marta's own account, on each day's closing balance, credited at each
    quarter's end; it withholds 19 % (PIT-8 .7), rounded to whole dukats (GEN-5.1)."""
    rows = b.run()
    acc = ZERO
    for d in F.days(T0, T1):
        acc += balance_on(rows, b.opening, d) * D("0.012") / 365
        if d == end_of_month(d) and d.month in (3, 6, 9, 12):
            i = half_up(acc)
            t = duk(i * D("0.19"))
            b.add(d, i, "ODSETKI", "Bank Jodła", "kapitalizacja odsetek", 8)
            b.add(d, -t, "PODATEK", "Bank Jodła", "podatek od odsetek", 9)
            rows = b.run()
            acc = ZERO


def accounts(taxes=(), *, known):
    """taxes: [(date, amount, payee, title)] the household's tax payments and refunds after 2024 (from c_tax)."""
    fm = firm(known=known)
    lz = lazur(known=known)
    ln = loan(known=known)
    _, mrows = instalments_by_month()
    joint, pot, marta = Book("joint", OPEN["joint"]), Book("pot", OPEN["pot"]), Book("marta", OPEN["marta"])
    for y in (2024, 2025, 2026):
        pays = F.marta_payments(y)
        if pays:
            rws, _ = F.marta_payroll(y, known=known)
            for p, r in zip(pays, rws):
                joint.add(p["date"], r["net"], "WYNAGRODZENIE", "Przychodnia Pod Lipami", f"wynagrodzenie {p['date'][5:7]}/{y}", 1)
    par = F.parental_2024()
    prow, _ = run_payer(par, reducing=False, known=known)
    for p, r in zip(par, prow):
        joint.add(p["date"], r["net"], "PRZELEW PRZYCH.", "SIO", f"zasiłek rodzicielski {p['for'][5:]}/{p['for'][:4]}", 1)
    joint.add(date(2024, 6, 28), -START_DUK, "PRZELEW WYCH.", "Jan Wrona " + F.ACC_LAZUR, "na start", 3)
    for r in fm["rows"]:
        if r.get("to") == "joint":
            joint.add(r["date"], -r["amount"], "PRZELEW PRZYCH.", "Warsztat Stolarski Jan Wrona", r["title"], 1)
    for d, amt, t, cp, ti, pri in fm["after"]:
        joint.add(d, amt, t, cp, ti, 2)
    for s in lz["sales"]:
        if s["to"] == "joint":
            joint.add(s["date"], s["duk"], "PRZELEW PRZYCH.", "Jan Wrona (Bank Lazur)", f"udział {s['for'][5:]}/{s['for'][:4]}", 1)
        else:
            pot.add(s["date"], s["duk"], "PRZELEW PRZYCH.", "Jan Wrona (Bank Lazur)", "sprzedaż FLR", 1)
    for d, a, k in fbo_payments():
        joint.add(d, a, "PRZELEW PRZYCH.", "FBO Zabór", f"świadczenie na dziecko {k[5:]}/{k[:4]}", 1)
    for d, a in FBO_REPAY:
        joint.add(CAL.on_or_next(d), -a, "PRZELEW WYCH.", "FBO Zabór", "zwrot nienależnego świadczenia — rata", 3)
    d = date(2024, 1, 3)
    while d <= T1:
        joint.add(CAL.on_or_next(d), D("300.00"), "PRZELEW PRZYCH.", "Irena Wrona", "dla Stasia", 1)
        d = add_months(d, 1)
    for r in mrows:
        if r["kind"] == "instalment" and r["date"] >= T0:
            joint.add(r["date"], -r["amount"], "SPŁATA KREDYTU", "Bank Jodła", f"rata kredytu {F.MORTGAGE}", 3)
    for d0, a in OVERPAY:
        joint.add(d0, -a, "SPŁATA KREDYTU", "Bank Jodła", f"nadpłata kredytu {F.MORTGAGE}", 3)
    for d0, a in extras():
        if (d0, a) not in OVERPAY:
            pot.add(d0, -a, "SPŁATA KREDYTU", "Bank Jodła", f"nadpłata kredytu {F.MORTGAGE}", 3)
    for k in F.months("2024-01", "2026-09"):
        for d0, p, t, a in F.bills(k):
            joint.add(d0, -a, "PRZELEW WYCH.", p, t, 3)
        if "2024-03" <= k <= "2025-08":
            joint.add(CAL.on_or_next(F.first(k).replace(day=5)), -NURSERY, "PRZELEW WYCH.", "Żłobek Bajka", f"opłata {k[5:]}/{k[:4]}", 3)
    for y in (2024, 2025, 2026):
        joint.add(CAL.on_or_next(date(y, 7, 2)), -CAR_INS, "PRZELEW WYCH.", "Ubezpieczenia Lipa", "OC/AC samochodu", 3)
    spend(joint)
    for d0, acct, payee, title, amt in PURCHASES:
        if acct == "joint":
            joint.add(d0, -amt, "KARTA", payee, title, 6)
    for y, (d0, a) in TITHE.items():
        joint.add(d0, -a, "PRZELEW WYCH.", "Fundacja Pod Dębem", f"darowizna {y}", 4)
    for d0, a, payee, title in taxes:
        joint.add(d0, a, "PRZELEW PRZYCH." if a > 0 else "PRZELEW WYCH.", payee, title, 2)
    for k in LEAVE_MONTHS:
        d0 = first_bd(k)
        pot.add(d0, -LEAVE_TRANSFER, "PRZELEW WYCH.", "Marta i Jan Wrona", "na bieżące", 2)
        joint.add(d0, LEAVE_TRANSFER, "PRZELEW PRZYCH.", "Skarbonka", "na bieżące", 1)
    joint.add(SWEEP[0], -SWEEP[1], "PRZELEW WYCH.", "Skarbonka", "oszczędności", 9)
    pot.add(SWEEP[0], SWEEP[1], "PRZELEW PRZYCH.", "Marta i Jan Wrona", "oszczędności", 1)
    pot.add(DEC_TOPUP[0], -DEC_TOPUP[1], "PRZELEW WYCH.", "Marta i Jan Wrona", "na święta", 2)
    joint.add(DEC_TOPUP[0], DEC_TOPUP[1], "PRZELEW PRZYCH.", "Skarbonka", "na święta", 1)
    marta.add(GIFT_TO_JAN[0], -GIFT_TO_JAN[1], "PRZELEW WYCH.", "Jan Wrona " + F.ACC_LAZUR, "darowizna", 3)
    marta.add(INHERIT[0], INHERIT[1], "PRZELEW PRZYCH.", "Kancelaria Notarialna Ewa Brzoza — depozyt",
              "wypłata z depozytu notarialnego", 1)
    marta.add(LOAN["date"], -LOAN["amount"], "PRZELEW WYCH.", "Marta i Jan Wrona", "pożyczka wg umowy z 2025-06-20", 3)
    joint.add(LOAN["date"], LOAN["amount"], "PRZELEW PRZYCH.", "Marta Wrona", "pożyczka wg umowy z 2025-06-20", 1)
    for d0 in ln["repayments"]:
        joint.add(d0, -LOAN["repay"], "PRZELEW WYCH.", "Marta Wrona", "spłata pożyczki", 3)
        marta.add(d0, LOAN["repay"], "PRZELEW PRZYCH.", "Marta i Jan Wrona", "spłata pożyczki", 1)
    interest_own(marta)
    return {"firm": fm, "lazur": lz, "loan": ln, "joint": joint, "pot": pot, "marta": marta, "mortgage": mrows}


if __name__ == "__main__":
    import c_tax
    A = accounts(c_tax.household_taxes(known=ALL_KNOWN)[0], known=ALL_KNOWN)  # R1: a debug run over the world, so it sees every figure
    for n in ("joint", "pot", "marta"):
        rows = A[n].run(check=False)
        lo = min(rows, key=lambda r: r["balance"])
        print(n, "end", rows[-1]["balance"], "low", lo["date"], lo["balance"])
        for k in ("2024-06-30", "2024-12-31", "2025-06-30", "2025-12-31", "2026-06-30"):
            print("   ", k, balance_on(rows, A[n].opening, P(k)))
    jr = A["joint"].run(check=False)
    short = []
    for d in F.days(T0, T1):
        n = sum(1 for k in F.months("2023-04", F.ym(d)) if CAL.on_or_next(F.first(k).replace(day=3)) <= d)
        b = balance_on(jr, A["joint"].opening, d)
        if b < 300 * n:
            short.append((d.isoformat(), b, 300 * n))
    print("stas short days", len(short), short[:3], short[-3:])
    lz = A["lazur"]
    print("lazur end", lz["flr"][-1]["balance"], [(s["date"].isoformat(), s["rule"], s["flr"]) for s in lz["sales"] if s["to"] == "pot"])
    print("loan released", A["loan"]["released"])
    print("ok months", [k for k in F.months("2024-07", "2026-09") if rate_month_ok(k, known=ALL_KNOWN)])  # R1: a debug run over the world, so it sees every figure
    print("firm closing", A["firm"]["closing"], "after", [(a[0].isoformat(), a[1], a[4]) for a in A["firm"]["after"]])
