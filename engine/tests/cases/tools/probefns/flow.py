"""Probe keys, FLOW axis."""
from decimal import Decimal

from lib.num import D
from lib.own import net_balances


def FLOW_01(i, t):
    (d0, b0), (d1, b1) = i["asserted"]
    flows = sum((D(a) for _, a in i["flows"]), Decimal(0))
    bal = t("balancing flow", D(b1) - D(b0) - flows, f"{b1} − {b0} − {flows} (rule 1)")
    holds = D(b0) + flows + bal == D(b1)
    return {"kind": "relation", "value": {"balancing_flow": bal, "direction": "out of the wallet" if bal < 0 else "into the wallet", "holds": holds}}


def FLOW_02(i, t):
    owners = t("owners' leg", D(i["price"]) - D(i["payoff"]) - D(i["fee"]), f"{i['price']} − {i['payoff']} − {i['fee']}")
    legs = D(i["payoff"]) + D(i["fee"]) + owners
    return {"kind": "relation", "value": {"lender": D(i["payoff"]), "dealer_fee": D(i["fee"]), "owners": owners, "legs_sum": legs}}


def FLOW_03(i, t):
    b = net_balances(i["shares"], i["costs"], i["rents"])
    for p, v in b.items():
        t(p, v, "paid − share × costs + share × rent − received (rule 2)")
    return {"kind": "relation", "value": {"balances": b, "sum": sum(b.values(), Decimal(0))}}
