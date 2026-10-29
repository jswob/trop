"""Probe keys, NUM axis: each function reads a card's inputs and returns its key."""
from decimal import Decimal

from lib.dates import days
from lib.keys import refusal
from lib.num import CENT, UNIT, D, Refuse, add, annuity, chain, half_up, split


def NUM_01(i, t):
    try:
        total = add(*[(x["amount"], x["cur"]) for x in i["a"]["items"]])
        a = {"kind": "value", "value": total[0]}
    except Refuse as e:
        t("(a) total", "refused", f"rule 1: {'; '.join(e.missing)} not stated")
        a = refusal(e)
    b = i["b"]
    r = t("(b) monthly rate", D(b["yearly_nominal"]) / 12, f"{b['yearly_nominal']} / 12 (rule 2)")
    pay = t("(b) instalment", annuity(b["principal"], r, b["months"]), f"{b['principal']} × r / (1 − (1 + r)^−{b['months']}), half up to 0.01")
    return {"a": a, "b": {"kind": "value", "value": pay, "unit": "DUK a month"}}


def NUM_02(i, t):
    unused = t("unused days", days(i["cover_ends"], i["end"]), f"{i['cover_ends']} → {i['end']} (rule 2)")
    year = t("days in the policy year", days(i["start"], i["end"]) + 1, f"{i['start']} → {i['end']} inclusive")
    aa = t("refund, actual/actual", half_up(D(i["premium"]) * unused / year), f"{i['premium']} × {unused} / {year} (rules 1, 3)")
    a365 = t("refund, actual/365", half_up(D(i["premium"]) * unused / 365), f"{i['premium']} × {unused} / 365")
    return {"kind": "value", "value": {"actual_actual": aa, "actual_365": a365, "unused_days": unused}}


def NUM_03(i, t):
    base = t("base", half_up(D(i["income"]) - D(i["costs"]), UNIT), f"{i['income']} − {i['costs']}, half up to 1 (rule 1)")
    adv = t("advance", half_up(base * D(i["rate"]) - D(i["relief"]), UNIT), f"{i['rate']} × {base} − {i['relief']}, half up to 1 (rule 2)")
    once = t("rounded once", half_up((D(i["income"]) - D(i["costs"])) * D(i["rate"]) - D(i["relief"]), UNIT), "same formula, unrounded base")
    return {"kind": "value", "value": {"advance": adv, "rounded_once": once, "difference": adv - once}}


def NUM_04(i, t):
    from lib.loans import refund_by_balance, refund_by_time
    n, k = i["months"], i["paid"]
    rt = t("reading T", half_up(refund_by_time(i["cost"], n, k)), f"{i['cost']} × {n - k}/{n}")
    rb = t("reading B", half_up(refund_by_balance(i["cost"], n, k)), f"{i['cost']} × Σ(1..{n - k}) / Σ(1..{n})")
    return {"kind": "by-case", "cases": {"T: pro rata by time": rt, "B: by the falling balance": rb}}


def NUM_05(i, t):
    parts = t("parts", split(i["total"], [1] * len(i["heirs"]), remainder="order"), "floor to 0.01, leftover units in listing order (rule 1)")
    printed = [half_up(p, UNIT) for p in parts]
    t("printed parts", printed, "half up to 1 (rule 2)")
    return {"kind": "value", "value": {"parts": dict(zip(i["heirs"], parts)), "sum": sum(parts, Decimal(0)),
            "printed_parts": printed, "printed_sum": sum(printed, Decimal(0)), "printed_total": half_up(i["total"], UNIT)}}


def NUM_06(i, t):
    f = t("index ratio", chain(1, i["factors"]), " × ".join(str(x) for x in i["factors"]))
    v = t("2030 nominal", half_up(D(i["amount"]) * f), f"{i['amount']} × ratio, half up to 0.01 (rule 2)")
    return {"kind": "value", "value": v}


def NUM_07(i, t):
    net = t("net", sum((D(x) for x in i["results"]), Decimal(0)), " + ".join(str(x) for x in i["results"]) + " (rule 1)")
    tax = t("tax", half_up(max(Decimal(0), net) * D(i["rate"])), f"{i['rate']} × max(0, net) (rule 2)")
    carried = t("loss carried", max(Decimal(0), -net), "−net when negative (rule 2)")
    return {"kind": "value", "value": {"net": net, "tax": tax, "loss_carried": carried}}
