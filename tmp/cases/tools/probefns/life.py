"""Probe keys, LIFE axis."""
from datetime import date
from decimal import Decimal

from lib.dates import P, add_months, days, end_of_month
from lib.law import table
from lib.loans import schedule
from lib.num import UNIT, D, half_up


def LIFE_01(i, t):
    def pay(ym):
        return D([a for m, a in i["pay"] if m <= ym][-1])
    total = t("2025 in all", sum((pay(f"2025-{m:02d}") for m in range(1, 13)), Decimal(0)), "Σ months by the contract's terms")
    jan = t("January 2026", half_up(pay("2025-12") * (1 + D(i["index"]["2025"])), UNIT), f"× (1 + {i['index']['2025']}), half up to 1")
    return {"kind": "value", "value": {"total_2025": total, "jan_2026": jan}}


def LIFE_02(i, t):
    sold = P(i["sold"])
    ins = i["insurance"]
    unused, year = days(sold, ins["end"]), days(ins["start"], ins["end"]) + 1
    refund = t("insurance refund", half_up(D(ins["premium"]) * unused / year), f"{ins['premium']} × {unused}/{year} (rule 1)")
    pend = t("parking ends", end_of_month(add_months(date(sold.year, sold.month, 1), 1)), "end of the month after notice (rule 3)")
    still = t("parking still paid", D(i["parking"]["monthly"]) * (pend.month - sold.month), "months after the sale's month until the end")
    return {"kind": "value", "value": {"insurance": {"ends": i["sold"], "refund": refund}, "credit": {"ends": i["sold"], "repaid": D(i["credit"])},
            "parking": {"ends": pend.isoformat(), "still_paid": still}, "car_tax": {"ends": i["sold"], "refund": D("0.00")}}}


def LIFE_03(i, t):
    rows = schedule(i["principal"], i["rate"], i["months"], i["events"])
    for r in rows[:9]:
        t(f"month {r['month']}", (r["interest"], r["principal"], r["balance"]), "interest, principal, balance (rule 1)")
    return {"kind": "value", "value": {"start": rows[0]["instalment"], "after_reset": rows[6]["instalment"],
            "after_overpay": rows[8]["next_instalment"], "balance_after_month_6": rows[5]["balance"]}}


def LIFE_04(i, t):
    total = Decimal(0)
    for m in range(1, 13):
        ym = f"{i['year']}-{m:02d}"
        total += D([a for f, a in i["amounts"] if f <= ym][-1])
    return {"kind": "value", "value": t("2025 in all", total, "each month at the amount in force (rule 1)")}


def LIFE_05(i, t):
    y, m = map(int, i["month"].split("-"))
    rev = sum((D(a) for _, a, _ in i["invoices"]), Decimal(0))
    vat = t("VAT", half_up(rev * D(i["vat"])), f"{i['vat']} × {rev} (rule 1)")
    due = date(y + (m == 12), m % 12 + 1, 25)
    sy, sm = map(int, i["started"].split("-"))
    n = (y - sy) * 12 + (m - sm) + 1
    s = i["social"]
    social = t("social", D(s["reduced"] if n <= s["months"] else s["full"]), f"business month {n} (rule 2)")
    todate = sum((D(a) for a in i["earlier"]), Decimal(0)) + rev
    health = t("health", table(todate, i["health"]), f"revenue to date {todate} (rule 3)")
    cash = t("cash in March", sum((half_up(D(a) * (1 + D(i["vat"]))) for _, a, p in i["invoices"] if p[:7] == i["month"]), Decimal(0)), "gross of invoices paid in the month")
    late = [no for no, _, p in i["invoices"] if P(p) > P(i["due"])]
    return {"kind": "value", "value": {"vat": vat, "vat_due": due.isoformat(), "social": social, "health": health, "cash_march": cash, "late": late}}
