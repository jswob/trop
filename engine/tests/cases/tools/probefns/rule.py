"""Probe keys, RULE axis."""
from datetime import date
from decimal import Decimal

from lib.dates import P, add_months, end_of_month
from lib.keys import refusal
from lib.num import D, Refuse
from lib.own import room
from lib.rules import balances, by_priority, find_cycle, mean_of_last, minimum, sweep


def RULE_01(i, t):
    w = i["wrapped"]
    out, stays = sweep(i["balance"], i["floor"], [("wrapped", room(w["used"], w["limit"], i["balance"])), ("broker", None)])
    for k, v in out:
        t(k, v, "floor kept, wrapped first up to its room (rule 1)")
    return {"kind": "value", "value": {**dict(out), "stays": stays}}


def RULE_02(i, t):
    d, b = minimum(balances(i["balance"], i["items"], counted={"planned"}))
    t("lowest, planned only", b, f"on {d} (rule 1)")
    _, b2 = minimum(balances(i["balance"], i["items"]))
    return {"kind": "value", "value": {"lowest": b, "on": d.isoformat(), "if_promise_counted": b2}}


def RULE_03(i, t):
    out = {}
    for part in ("a", "b"):
        try:
            got, left = by_priority(i["surplus"], i["claims"], i[part])
            t(f"({part})", got, f"in turn {i[part]} (rule 1)")
            out[part] = {"kind": "value", "value": {**got, "left": left}}
        except Refuse as e:
            t(f"({part})", "refused", "no priority (rule 2)")
            out[part] = refusal(e)
    return out


def RULE_04(i, t):
    c = t("cycle", find_cycle(i["reads"]), "rules reading each other")
    if c:
        raise Refuse(["a solver for " + " -> ".join(c)], "rules in a loop and no law names a solver (rule 1)")
    return {"kind": "value", "value": None}


def RULE_05(i, t):
    d, bal = P(i["start"][0]), D(i["start"][1])
    while bal < D(i["threshold"]):
        d = end_of_month(add_months(date(d.year, d.month, 1), 1))
        bal += D(i["monthly"])
        t(f"savings {d}", bal, f"+ {i['monthly']}")
    nxt = add_months(date(d.year, d.month, 1), 1)
    inst = nxt.replace(day=i["instalment_day"])
    seen = nxt.replace(day=i["rule_day"]) if i["rule_day"] >= i["instalment_day"] else add_months(nxt, 1).replace(day=i["rule_day"])
    return {"kind": "value", "value": {"trigger": d.isoformat(), "first_instalment": inst.isoformat(), "first_seen": seen.isoformat()}}


def RULE_06(i, t):
    v2 = i["v2"]
    gap = {m: t(f"gap {m}", D(v2["floor"]) - D(left), f"{v2['floor']} − {left} left (rule 2)") for m, left in i["done"].items() if m >= v2["from"][:7]}
    return {"kind": "value", "value": {"gap": gap, "gap_total": sum(gap.values(), Decimal(0)), "rebooked": False, "floor_from_april": D(v2["floor"])}}


def RULE_07(i, t):
    rec = t("by records", D(i["start"]) + D(i["amount"]) * len(i["rows"]), f"{i['start']} + {len(i['rows'])} × {i['amount']}")
    rule = t("by rule", D(i["start"]) + D(i["amount"]) * len(i["months"]), f"{i['start']} + {len(i['months'])} × {i['amount']}")
    gap = {m: D(i["amount"]) for m in i["months"] if m not in i["rows"]}
    t("gap", gap, "months the rule names and the records lack (rule 1)")
    return {"kind": "value", "value": {"by_records": rec, "by_rule": rule, "gap": gap}}


def RULE_08(i, t):
    out = {}
    for d in i["decisions"]:
        v, ms = mean_of_last(i["spending"], d, i["months"], D("0.01"))
        out[d] = t(f"floor {d}", v, f"mean of {ms} (rule 1)")
    return {"kind": "value", "value": out}


def RULE_09(i, t):
    share = t("share of ruined paths", D(sum(i["ruined"])) / len(i["ruined"]), f"{sum(i['ruined'])} of {len(i['ruined'])} on {i['decided_on']}")
    dec = "stop buying" if share > D(i["limit"]) else "keep buying"
    t("decision", dec, f"more than {i['limit']}: {share > D(i['limit'])} (rule 1)")
    b = refusal(Refuse(["a statistic across paths, read inside one path"], "rule 2"))
    return {"a": {"kind": "value", "value": {"share": share, "decision": dec, "decided_on": i["decided_on"]}}, "b": b}
