"""Probe keys, ASK axis."""
from decimal import Decimal

from lib.keys import refusal
from lib.num import D, F, Refuse, half_up, UNIT
from lib.own import fifo
from lib.rules import balances, first_below, minimum


def ASK_01(i, t):
    s = balances(i["start"], i["items"])
    t("balances", [(d.isoformat(), b) for d, b in s], "after each day's rows (rule 1)")
    d, b = minimum(s)
    return {"kind": "value", "value": {"first_below": first_below(s, i["floor"]).isoformat(), "minimum": b, "minimum_on": d.isoformat()}}


def ASK_02(i, t):
    nw, missing = {}, []
    for h in i["holdings"]:
        for p, s in h["owners"].items():
            nw.setdefault(p, Decimal(0))
            if h["value"] is None:
                missing.append(f"value of {p}'s {h['what']} on {i['asked']}")
            else:
                nw[p] += t(f"{p}: {h['what']}", half_up(D(h["value"]) * D(F(s))), f"{h['value']} × {s} (rule 1)")
    if not missing:
        return {"kind": "value", "value": nw}
    # README § Keys: a partial stands in place of the value it concerns; a person with no hole stays plain
    return {p: ({"kind": "partial", "value": v, "missing": [m for m in missing if m.startswith(f"value of {p}'s ")]}
                if any(m.startswith(f"value of {p}'s ") for m in missing) else v) for p, v in nw.items()}


def ASK_03(i, t):
    a = i["a"]
    if a["fields"].get(a["needs"]) is None:
        t("(a)", "refused", f"{a['needs']} blank on {a['form']} (rule 1)")
        ra = refusal(Refuse([f"days abroad in {a['year']} (blank on {a['form']})"], "a blank field the law needs"))
    else:
        ra = {"kind": "value", "value": a["fields"][a["needs"]] > a["limit"]}
    try:
        fifo(i["b"]["lots"], i["b"]["sell"])
        rb = {"kind": "value", "value": None}
    except Refuse as e:
        t("(b)", "refused", "rule 2")
        rb = refusal(e)
    return {"a": ra, "b": rb}


def ASK_04(i, t):
    p = D(i["principal"])
    ca = t("cost A", D(i["A"]["fee"]) + p * D(i["A"]["rate"]), "fee + interest")
    cb = t("cost B", D(i["B"]["fee"]) + p * D(i["B"]["rate"]), "fee + interest")
    be = t("breakeven rate for B", (ca - D(i["B"]["fee"])) / p, "B's rate at which the costs are equal")
    return {"kind": "refusal", "missing": [], "reason": "a choice is asked; consequences given, no winner (rule 1)",
            "values": {"cost_A": ca, "cost_B": cb, "breakeven_rate_B": be}}


def ASK_05(i, t):
    g = D(i["gross"])
    pen = t("pension", half_up(g * D(i["pension"])), "rule 1")
    hea = t("health", half_up((g - pen) * D(i["health"])), "rule 2")
    tax = t("tax advance", half_up((g - pen) * D(i["tax"]), UNIT), "rule 3")
    causes = [["gross", "contract", g], ["pension", "rule 1", -pen], ["health", "rule 2", -hea], ["tax advance", "rule 3", -tax]]
    return {"kind": "value", "value": {"net": g - pen - hea - tax, "causes": causes}}
