"""Probe keys, CUR axis."""
from decimal import Decimal

from lib.dates import Calendar
from lib.num import D, convert


def CUR_01(i, t):
    d = t("law's rate date", Calendar().before(i["issued"]), f"last business day before {i['issued']} (rule 1)")
    rate = D(i["mid"][d.isoformat()])
    rev = t("revenue", convert(i["amount"], rate), f"{i['amount']} × {rate}")
    cash = t("cash", convert(i["amount"], i["bank_rate"]), f"{i['amount']} × {i['bank_rate']}")
    return {"kind": "value", "value": {"law_rate_date": d.isoformat(), "revenue": rev, "cash": cash, "fx_loss": rev - cash}}


def CUR_02(i, t):
    one = t("one conversion", convert(i["amount"], i["rate"]), f"{i['amount']} × {i['rate']} (rule 1)")
    booked = t("booked", one * i["count"], f"{i['count']} × {one}")
    view = t("view", convert(D(i["amount"]) * i["count"], i["rate"]), "the total converted at once (rule 2)")
    return {"kind": "value", "value": {"booked": booked, "view": view, "difference": view - booked}}


def CUR_03(i, t):
    vals = {d: t(f"value {d}", convert(i["balance"], r), f"{i['balance']} × {r}") for d, r in i["rates"].items()}
    v = list(vals.values())
    return {"kind": "relation", "value": {"values": vals, "revaluation": v[1] - v[0], "flr_balance": [D(i["balance"])] * 2, "transfers": 0}}
