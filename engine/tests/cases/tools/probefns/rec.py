"""Probe keys, REC axis."""
from datetime import timedelta
from decimal import Decimal
from fractions import Fraction

from lib.dates import P
from lib.law import gazette
from lib.ledger import canonical, in_cycle, parse, replay
from lib.num import UNIT, D, half_up


def REC_01(i, t):
    flagged = t("flagged", [a for a in i["statement_accounts"] if a not in i["model_accounts"]], "in the records, not in the model (rule 1)")
    res = t("residual", D(i["statement_balance"]) - D(i["model_balance"]), "statement − model (rule 2)")
    return {"kind": "relation", "value": {"flagged": flagged, "residual": res, "booked": False}}


def REC_02(i, t):
    rows = parse(i["csv"], header=2, amount_cols=("Kwota", "Saldo"), date_cols=("Data",))
    closing, issues = replay(i["opening"], rows, amount="Kwota", balance="Saldo")
    t("replay", issues, "balance-after walk (rule 1)")
    naive = t("naive sum", D(i["opening"]) + sum((r["Kwota"] for r in rows), Decimal(0)), "opening + every row")
    bad = {x["row"] for x in issues}
    without = t("without faulty rows", D(i["opening"]) + sum((r["Kwota"] for k, r in enumerate(rows, 1) if k not in bad), Decimal(0)), f"rows {sorted(bad)} dropped")
    assert naive - without == sum((x["amount"] for x in issues), Decimal(0))
    return {"kind": "relation", "value": {"closing": closing, "issues": issues, "naive_sum": naive, "without_faulty": without}}


def REC_03(i, t):
    bill = t("bill", in_cycle(i["rows"], *i["cycle"]), f"rows within {i['cycle']} (rule 1)")
    return {"kind": "value", "value": {"bill": bill, "due": i["due"]}}


def REC_04(i, t):
    inv, row = i["invoice"], i["row"]
    same = t("same counterparty", canonical(inv["client"], i["registry"]) == canonical(row["from"], i["registry"]), "registry names (rule 1)")
    inside = P(inv["due"]) <= P(row["date"]) <= P(inv["due"]) + timedelta(days=i["window_days"])
    paid = min(D(row["amount"]), D(inv["amount"])) if same and inside else D(0)
    open_ = t("open", D(inv["amount"]) - paid, "rule 2")
    cert, p = i["lexicon"][i["said"]]
    return {"kind": "value", "value": {"matched": inv["id"] if paid else None, "paid": paid, "open": open_, "certainty": cert, "p": D(str(p))}}


def _net(g, i):
    c = half_up(g * D(i["pension"]))
    return g - c - half_up((g - c) * D(i["tax"]), UNIT)


def REC_05(i, t):
    net = D(i["net"])
    guess = half_up(net / (1 - D(i["pension"])) / (1 - D(i["tax"])))
    found = [g for g in (guess + D(k) / 100 for k in range(-5000, 5001)) if _net(g, i) == net]
    t("gross giving the net", found, f"every cent within 50.00 of {guess} (rule 1)")
    if len(found) == 1:
        # README § Keys: the partial stands in place of the gross alone; that it is inferred is settled
        return {"gross": {"kind": "partial", "value": found[0], "missing": [f"payslip for {i['month']} (gross inferred from net)"]}, "inferred": True}
    return {"kind": "by-case", "cases": {str(g): g for g in found}}


def REC_06(i, t):
    out = {}
    for k in i["known"]:
        r, e, _ = gazette(i["entries"], i["on"], known=k)
        out[k] = t(f"known {k}", half_up(D(i["income"]) * r), f"rate {r} published {e['published']} (rule 1)")
    return {"kind": "value", "value": out}


def REC_07(i, t):
    p, n = i["prices"], i["years"]
    ends = []
    for s in range(len(p) - n):
        units = sum((Fraction(D(i["buy"])) / p[s + k] for k in range(n)), Fraction(0))
        ends.append(t(f"start {s}", half_up(D(units * p[s + n])), f"units {units} × {p[s + n]} (rule 1)"))
    w = min(range(len(ends)), key=lambda k: ends[k])
    b = max(range(len(ends)), key=lambda k: ends[k])
    return {"kind": "value", "value": {"ends": ends, "worst": [w, ends[w]], "best": [b, ends[b]]}}
