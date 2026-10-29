"""Probe keys, SCEN axis."""
from datetime import date
from decimal import Decimal

from lib.dates import P, add_months
from lib.num import D, half_up


def SCEN_01(i, t):
    l, o, asked = i["loan"], i["overpay"], P(i["asked"])
    cash, debt = D(l["amount"]), D(l["amount"])
    d = P(l["from"] + f"-{l['day']:02d}")
    while d <= asked and debt > 0:
        pay = min(D(l["instalment"]), debt)
        cash, debt = cash - pay, debt - pay
        t(f"instalment {d}", debt, f"− {pay}")
        if d.strftime("%Y-%m") == o["month"]:
            cash, debt = cash - D(o["amount"]), debt - D(o["amount"])
            t(f"overpayment {d}", debt, f"− {o['amount']}")
        d = add_months(d, 1)
    return {"kind": "relation", "value": {"cash_difference": cash, "debt": debt, "net_worth_difference": cash - debt}}


def SCEN_02(i, t):
    asked, br = P(i["asked"]), P(i["branch"])
    dropped = [e["id"] for e in i["actual"] if e["needs_sale"] and P(e["date"]) >= br]
    kept = [e["id"] for e in i["actual"] if not e["needs_sale"]]
    added, extra = [], Decimal(0)
    for c in i["if_kept"]:
        d = P(c["from"] + f"-{c['day']:02d}")
        while d <= asked:
            added.append(f"{c['id']} {d.strftime('%Y-%m')}")
            extra += D(c["amount"])
            d = add_months(d, 1)
    t("dropped", dropped, "need the sale (rule 1)")
    t("added", added, "run on if kept (rule 2)")
    diff = t("cash difference", extra - sum((D(e["amount"]) for e in i["actual"] if e["id"] in dropped), Decimal(0)), "added − dropped")
    return {"kind": "value", "value": {"dropped": dropped, "kept": kept, "added": sorted(added), "cash_difference": diff}}


def _rent(i, override):
    r = D(i["rent"])
    for y in range(i["from"] + 1, i["to"] + 1):
        prev = y - 1
        o = i["override"]
        rate = D(o["rate"]) if override and o["years"][0] <= prev <= o["years"][1] else D(i["base"])
        r = half_up(r * (1 + rate))
    return r


def SCEN_03(i, t):
    o = t("rent 2029, override", _rent(i, True), "previous year's index, override years then base (rules 1–2)")
    b = t("rent 2029, base", _rent(i, False), "base path")
    moved = sorted(k for k, v in i["readers"].items() if v == "CPI")
    return {"kind": "value", "value": {"rent_2029": o, "base_rent_2029": b, "moved": moved}}


def SCEN_04(i, t):
    bal, first = D(i["start"]), None
    for y in range(1, i["years"] + 1):
        r = D(i["returns"].get(str(y), i["returns"]["default"]))
        bal = t(f"year {y}", half_up((bal + D(i["yearly"])) * (1 + r)), f"(+ {i['yearly']}) × (1 + {r}) (rule 1)")
        if first is None and bal < D(i["floor"]):
            first = y
    return {"kind": "value", "value": {"end": bal, "first_below_floor": first}}
