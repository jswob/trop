"""Probe keys, TIME axis."""
from datetime import date

from lib.dates import Calendar, P, add_months, end_of_month
from lib.law import cumulative_advances
from lib.num import D, half_up
from lib.rules import sweep


def TIME_01(i, t):
    y, m = map(int, i["bought_month"].split("-"))
    sold = P(i["sold"])
    gain = t("gain", D(i["proceeds"]) - D(i["cost"]), f"{i['proceeds']} − {i['cost']}")
    exempt = [d for d in range(1, 32) if _valid(y, m, d) and sold > add_months(date(y, m, d), 12 * i["years"])]
    taxed = [d for d in range(1, 32) if _valid(y, m, d) and d not in exempt]
    t("purchase days giving an exempt sale", f"{min(exempt)}–{max(exempt)}" if exempt else "none", "rule 1")
    t("purchase days giving a taxed sale", f"{min(taxed)}–{max(taxed)}" if taxed else "none", "rule 1")
    if exempt and taxed:
        return {"kind": "partial", "value": {"gain": gain}, "missing": [f"purchase day of {i['lot']} (known only as {i['bought_month']})"]}
    return {"kind": "value", "value": {"gain": gain, "taxed": bool(taxed)}}


def _valid(y, m, d):
    try:
        date(y, m, d)
        return True
    except ValueError:
        return False


def TIME_02(i, t):
    cal = Calendar(i["holidays"])
    a = t("(a) due", cal.on_or_next(i["a"]), f"{i['a']} or the next business day")
    b = t("(b) due", cal.on_or_next(i["b"]), f"{i['b']} or the next business day")
    c = t("(c) third-to-last business day", cal.nth_last(i["c"]["year"], i["c"]["month"], i["c"]["n"]), "counting business days back from the month's end")
    return {"kind": "value", "value": {"a": a.isoformat(), "b": b.isoformat(), "c": c.isoformat()}}


def _rate(versions, on):
    on = P(on)
    for v in versions:
        if P(v["from"]) <= on <= P(v["to"]):
            return D(v["rate"])
    raise ValueError(on)


def TIME_03(i, t):
    rp = t("rate on the payment day", _rate(i["versions"], i["paid"]), f"version in force {i['paid']} (rule 1)")
    rq = t("rate for the period", _rate(i["versions"], i["period"] + "-01"), f"version in force in {i['period']}")
    return {"kind": "value", "value": {"by_payment": half_up(D(i["gross"]) * rp), "by_period": half_up(D(i["gross"]) * rq)}}


def TIME_04(i, t):
    born = P(i["born"])
    y, m = map(int, i["period"][0].split("-"))
    total, first = D(0), None
    while f"{y:04d}-{m:02d}" <= i["period"][1]:
        start = date(y, m, 1)
        if (y, m) == (born.year, born.month):
            n = end_of_month(start).day
            amt = t(f"{y:04d}-{m:02d}", half_up(D(i["monthly"]) * (n - born.day + 1) / n), f"{i['monthly']} × {n - born.day + 1}/{n} (rule 2)")
            first = amt
        elif start > born:
            amt = D(i["monthly"])
        else:
            amt = D(0)
        total += amt
        y, m = (y + 1, 1) if m == 12 else (y, m + 1)
    t("period total", total, "sum of the months (rule 1)")
    return {"kind": "value", "value": {"first_month": first, "total": total}}


def TIME_05(i, t):
    a = i["a"]
    wend = t("(a) window end", add_months(a["bought"], a["months"]), "same-numbered day, clipped (rule 1)")
    b = i["b"]
    cend = t("(b) clock end", add_months(date(P(b["acquired"]).year, 12, 31), 12 * b["years"]), "end of the year of acquisition + 5 years (rule 2)")
    c = i["c"]
    eom = end_of_month(c["death"])
    dl = t("(c) deadline", end_of_month(add_months(eom, c["months"])), f"from {eom}, a month's last day: the last day of the final month (rule 3)")
    dfd = t("(c) day-for-day", add_months(eom, c["months"]), "for contrast")
    return {"kind": "value", "value": {
        "a": {"window_end": wend.isoformat(), "short_term": P(a["sold"]) <= wend},
        "b": {"clock_end": cend.isoformat(), "exempt": P(b["sold"]) > cend, "first_exempt_day": date(cend.year + 1, 1, 1).isoformat()},
        "c": {"deadline": dl.isoformat(), "in_time": P(c["reported"]) <= dl, "day_for_day_deadline": dfd.isoformat()}}}


def TIME_06(i, t):
    adv = cumulative_advances([i["monthly_base"]] * i["months"], i["bands"], D("0.01"))
    for k, a in enumerate(adv, 1):
        t(f"advance, month {k}", a, "tax on the base to date − withheld (rule 2)")
    return {"kind": "value", "value": {"september": adv[8], "october": adv[9]}}


def TIME_07(i, t):
    moved, _ = sweep(D(i["before"]) + D(i["pay"]), i["floor"], [("savings", None)])
    s = t("stated order: pay, then the rule", moved[0][1], f"{i['before']} + {i['pay']} − floor {i['floor']}")
    rev, _ = sweep(i["before"], i["floor"], [("savings", None)])
    r = t("reverse order", rev[0][1], f"{i['before']} under the floor")
    return {"kind": "value", "value": {"stated_order": s, "reverse_order": r}}
