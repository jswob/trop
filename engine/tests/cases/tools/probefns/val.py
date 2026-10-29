"""Probe keys, VAL axis."""
from datetime import date

from lib.dates import P, months_back
from lib.keys import refusal
from lib.law import gazette
from lib.num import D, Refuse, half_up


def VAL_01(i, t):
    # R1: the card states no known-as-of date; its story says "Nothing later is published", so it knows every entry listed
    known = date.max
    out = {}
    for part in ("a", "b"):
        try:
            v, e, held = gazette(i["entries"], i[part], known)
            t(f"({part}) RW on {i[part]}", v, f"entry from {e['from']}" + (f", past its end {e['to']}: held" if held else "") + " (rules 1–2)")
            out[part] = ({"kind": "partial", "value": v, "missing": [f"{e['id']} on {i[part]} (held at the entry from {e['from']})"]}
                         if held else {"kind": "value", "value": v})
        except Refuse as x:
            t(f"({part}) RW on {i[part]}", "refused", "no entry in force (rule 2)")
            out[part] = refusal(x)
    return out


def VAL_02(i, t):
    m = t("month read", months_back(i["starts"], i["lag_months"]), f"{i['starts']} − {i['lag_months']} months (rule 1)")
    return {"kind": "value", "value": {"read_month": m, "value": D(i["index"][m])}}


def VAL_03(i, t):
    out = {}
    for k in i["known"]:
        v, e, _ = gazette(i["entries"], i["on"], known=k)
        out[k] = {"value": t(f"known {k}", v, f"entry published {e['published']} (rule 1)"), "source": e["source"]}
    return {"kind": "value", "value": out}


def VAL_04(i, t):
    law = t("law's gain", D(i["proceeds"]) - D(i["gift_day_value"]), f"{i['proceeds']} − {i['gift_day_value']} (rule 1)")
    st = D(i["statement_gain"])
    implied = t("cost the statement used", D(i["proceeds"]) - st, f"{i['proceeds']} − {st}")
    return {"kind": "value", "value": {"statement": st, "law": law, "gap": st - law,
            "cause": {"cost_in_statement": implied, "cost_by_law": D(i["gift_day_value"])}}}


def VAL_05(i, t):
    asked = P(i["asked"])
    pd = max(d for d in i["prices"] if P(d) <= asked)
    a = t("(a) value", D(i["units"]) * D(i["prices"][pd]), f"{i['units']} × price of {pd} (rule 1)")
    m = i["sale"]["date"][:7]
    b = t("(b) sale", D(i["sale"]["units"]) * D(i["forecast"][m]), f"{i['sale']['units']} × price of {m} (rule 2)")
    return {"kind": "value", "value": {"a": {"price_date": pd, "value": a}, "b": b}}


def VAL_06(i, t):
    bal = D(0)
    y0 = min(int(y) for y in i["earnings"])
    for y in range(y0, i["to"] + 1):
        # R2: the card states indices from 2022 only; 2021 is the first year, its worked line "End 2021: 40 000." indexes nothing
        if str(y) in i["index"]:
            bal = t(f"end {y}, indexed", half_up(bal * D(i["index"][str(y)])), f"× {i['index'][str(y)]}")
        bal = t(f"end {y}, + earnings", bal + D(i["earnings"][str(y)]), f"+ {i['earnings'][str(y)]}")
    return {"kind": "value", "value": bal}


def VAL_07(i, t):
    reg, todo = [], list(i["reads"]["rent"])
    while todo:
        a = todo.pop(0)
        if a not in reg:
            reg.append(a)
            todo += i["reads"].get(a, [])
    t("register", reg, "assumptions read, transitively (rule 1)")
    a = i["assumptions"]
    rate = D(a["A-CPI"]["rate"]) + D(a["A-RENT"]["plus"])
    rent = D(i["rent"])
    for y in range(i["from"] + 1, i["to"] + 1):
        rent = t(f"rent {y}", half_up(rent * (1 + rate)), f"× (1 + {rate}) (rule 2)")
    return {"kind": "value", "value": {"rent_2027": rent, "register": reg}}
