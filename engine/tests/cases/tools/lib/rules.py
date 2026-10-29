"""House rules: sweeps, priorities, loops, look-aheads, triggers, estimators."""
from datetime import timedelta
from decimal import Decimal

from .dates import P, months_back
from .num import D, Refuse, half_up


def sweep(balance, floor, targets):
    """Keep `floor`; pour the rest into targets in order, each up to its room (None = no limit)."""
    left = max(Decimal(0), D(balance) - D(floor))
    out = []
    for name, cap in targets:
        put = left if cap is None else min(left, D(cap))
        out.append((name, put))
        left -= put
    return out, D(balance) - sum((a for _, a in out), Decimal(0))


def by_priority(surplus, claims, priority=None):
    """Claims {rule: want} on one sum. With a priority (list of rules) each takes in turn; without one, refused."""
    if priority is None and len(claims) > 1:
        raise Refuse([f"priority between {', '.join(sorted(claims))}"], "two rules want the same money and no priority is stated")
    left, out = D(surplus), {}
    for rule in priority or list(claims):
        out[rule] = min(left, D(claims[rule]))
        left -= out[rule]
    return out, left


def find_cycle(deps):
    """A cycle in {rule: [rules it reads]} as a list, or None."""
    state, stack = {}, []

    def visit(n):
        state[n] = 1
        stack.append(n)
        for m in deps.get(n, []):
            if state.get(m) == 1:
                return stack[stack.index(m):] + [m]
            if m not in state:
                c = visit(m)
                if c:
                    return c
        stack.pop()
        state[n] = 2
        return None

    for n in sorted(deps):
        if n not in state:
            c = visit(n)
            if c:
                return c
    return None


def balances(start, items, counted=None):
    """Balance after each dated item, in date order then listing order; items: {date, amount, certainty}.
    Only certainties in `counted` (None = all) are applied."""
    bal, out = D(start), []
    for it in sorted(items, key=lambda i: P(i["date"])):
        if counted is not None and it.get("certainty", "planned") not in counted:
            continue
        bal += D(it["amount"])
        out.append((P(it["date"]), bal))
    return out


def minimum(series):
    """The lowest balance and its first date."""
    return min(series, key=lambda x: (x[1], x[0]))


def first_below(series, floor):
    for d, b in series:
        if b < D(floor):
            return d
    return None


def mean_of_last(months, decision, k, step):
    """Mean of the k full months before the decision date's month; months: {'YYYY-MM': amount}."""
    ym = P(decision).strftime("%Y-%m")
    keys = [months_back(ym, i) for i in range(1, k + 1)]
    miss = [m for m in keys if m not in months]
    if miss:
        raise Refuse([f"spending {m}" for m in miss], "the estimator needs months the history lacks")
    return half_up(sum(D(months[m]) for m in keys) / k, step), sorted(keys)
