"""Holdings and ownership: lots, FIFO, succession shares, co-owners' netting, wrapped-account room."""
from decimal import Decimal
from fractions import Fraction

from .dates import P
from .num import D, F, Refuse


def fifo(lots, units, account=None):
    """Take `units` from lots oldest first (only `account`'s lots when given).

    lots: [{id, account, date, units, cost}]; returns (cost taken, [(lot id, units, cost)]).
    Selling more than is held is refused.
    """
    pool = sorted((l for l in lots if account is None or l["account"] == account), key=lambda l: (P(l["date"]), l["id"]))
    need, cost, taken = F(D(units)), Fraction(0), []
    held = sum(F(D(l["units"])) for l in pool)
    if need > held:
        raise Refuse([f"units held: {D(held)} < {D(need)} sold"], "selling more units than are held")
    for l in pool:
        if need == 0:
            break
        take = min(need, F(D(l["units"])))
        c = F(D(l["cost"])) * take / F(D(l["units"]))
        taken.append((l["id"], D(take), D(c)))
        cost += c
        need -= take
    return D(cost), taken


def estate(children, spouse=None, spouse_min="1/4"):
    """Statutory shares: spouse and children equal, the spouse at least `spouse_min`;
    a child who died first is represented by his own children in equal parts.

    children: [{name, alive, issue: [names]}]; returns {name: Fraction}.
    """
    heads = len(children) + (1 if spouse else 0)
    each = Fraction(1, heads)
    shares = {}
    if spouse:
        s = max(each, F(spouse_min))
        shares[spouse] = s
        each = (1 - s) / len(children)
    for c in children:
        if c.get("alive", True):
            shares[c["name"]] = shares.get(c["name"], 0) + each
        else:
            issue = c.get("issue") or []
            if not issue:
                raise Refuse([f"heirs of {c['name']}"], "a predeceased child with no stated issue")
            for g in issue:
                shares[g] = shares.get(g, 0) + each / len(issue)
    return shares


def net_balances(shares, costs, rents):
    """Co-owners' balances: + is owed to the person, − owed by them; they add up to zero.

    shares: {name: fraction}; costs: [[payer, amount]]; rents: [[receiver, amount]].
    """
    tc = sum((D(a) for _, a in costs), Decimal(0))
    tr = sum((D(a) for _, a in rents), Decimal(0))
    out = {}
    for p, s in shares.items():
        s = D(F(s))
        paid = sum((D(a) for q, a in costs if q == p), Decimal(0))
        got = sum((D(a) for q, a in rents if q == p), Decimal(0))
        out[p] = paid - s * tc + s * tr - got
    return out


def room(used, limit, want):
    """What fits under a yearly limit: min(want, limit − used), never below 0."""
    return min(D(want), max(Decimal(0), D(limit) - D(used)))
