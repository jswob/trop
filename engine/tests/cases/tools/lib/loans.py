"""Loans: annuity schedules with resets, overpayments and holidays; early-repayment cost refunds."""
from decimal import Decimal
from fractions import Fraction

from .num import CENT, D, annuity, half_up


def schedule(principal, rate, n, events=(), step=CENT):
    """Monthly annuity schedule; interest rounded half up each month.

    events: {month, kind: reset|overpay|holiday, rate|amount}. A reset applies from its month and
    re-levels the instalment over the months left; an overpayment is paid after that month's
    instalment and re-levels it (lower instalment, same end); a holiday skips the instalment and
    adds the month's interest to the balance, the end moving one month on.
    """
    bal, r, months = D(principal), D(rate), n
    pay = annuity(bal, r, months, step)
    rows, m = [], 0
    evs = list(events)
    while bal > 0:
        m += 1
        for e in evs:
            if e["month"] == m and e["kind"] == "reset":
                r = D(e["rate"])
                pay = annuity(bal, r, months - m + 1, step)
        interest = half_up(bal * r, step)
        if any(e["month"] == m and e["kind"] == "holiday" for e in evs):
            bal += interest
            months += 1
            rows.append({"month": m, "interest": interest, "instalment": Decimal(0), "principal": Decimal(0), "balance": bal})
            continue
        princ = pay - interest
        if m >= months or princ > bal:
            princ = bal
        bal -= princ
        row = {"month": m, "interest": interest, "instalment": princ + interest, "principal": princ, "balance": bal}
        for e in evs:
            if e["month"] == m and e["kind"] == "overpay":
                bal -= D(e["amount"])
                row["overpay"] = D(e["amount"])
                row["balance"] = bal
                pay = annuity(bal, r, months - m, step) if months > m and bal > 0 else Decimal(0)
        row["next_instalment"] = pay
        rows.append(row)
    return rows


def refund_by_time(cost, n, k):
    """The share of a cost for the n − k months not used."""
    return D(cost) * (n - k) / n


def refund_by_balance(cost, n, k):
    """The share of a cost carried by the balance still owed in months k+1..n (equal principal)."""
    left = (n - k) * (n - k + 1) // 2
    total = n * (n + 1) // 2
    return D(Fraction(D(cost)) * Fraction(left, total))
