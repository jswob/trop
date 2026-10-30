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


# --------------------------------------------------------------------------- parts (tally 2.1, session 7b)
# Places that keep parts (L §3.6, §5.10, §5.14): a map label -> amount, split and taken by named steps.

class PartsRefused(Refuse):
    """A refusal of a parts step, with its code (L §8.3, §8.4)."""

    def __init__(self, code, missing, reason=""):
        self.code = code
        super().__init__(missing, f"{code}: {reason}" if reason else code)


def split_largest(total, weights, step, rank=None):
    """L §5.13 `rest largest` at `step`: `total` split by `weights` {label: weight}; each raw share is floored to the
    step, and the steps left over go one each to the largest remainders. Labels whose remainders tie go by `rank`
    (least first), and a tie with no rank, or a rank that ties, is refused (RUN-TIE). A total that isn't a whole number
    of the step is refused (RUN-PRORATA); weights that are negative or sum to 0 too (RUN-WEIGHTS). {label: Decimal}."""
    total, step = D(total), D(step)
    units = total / step
    if units != units.to_integral_value():
        raise PartsRefused("RUN-PRORATA", [f"{total} as a whole number of {step}"], f"{total} isn't a whole number of {step}")
    units = int(units)
    ws = {k: F(D(w)) if not isinstance(w, Fraction) else w for k, w in weights.items()}
    s = sum(ws.values(), Fraction(0))
    if s <= 0 or any(w < 0 for w in ws.values()):
        raise PartsRefused("RUN-WEIGHTS", ["positive weights"], f"weights {ws} are negative or sum to 0")
    raw = {k: Fraction(units) * w / s for k, w in ws.items()}
    base = {k: r.numerator // r.denominator for k, r in raw.items()}
    left = units - sum(base.values())
    if left:
        rem = {k: raw[k] - base[k] for k in ws}
        order = sorted(ws, key=lambda k: -rem[k])
        cut = rem[order[left - 1]]
        sure = [k for k in order if rem[k] > cut]
        tied = [k for k in order if rem[k] == cut]
        need = left - len(sure)
        if need < len(tied):
            if rank is None:
                raise PartsRefused("RUN-TIE", ["a ties by for the split"], f"the split ties at {sorted(map(str, tied))}")
            rs = sorted(tied, key=rank)
            if need and rank(rs[need - 1]) == rank(rs[need]):
                raise PartsRefused("RUN-TIE", ["a ties by that orders the tied labels"], f"the split ties at {sorted(map(str, tied))}")
            tied = rs[:need]
        else:
            tied = tied[:need]
        for k in sure + tied:
            base[k] += 1
    return {k: step * b for k, b in base.items()}


def arrive(parts, amount, shares, step, rank=None, batch=None):
    """An arrival labelled by the arrival rule (L §3.6): `shares` {party: Fraction} must sum to exactly 1 (RUN-LABEL);
    the amount is split by them, rest largest at the step, ties by `rank`. With `batch`, each label is (party, batch).
    Adds to `parts` in place; returns {label: amount}."""
    total = sum(shares.values(), Fraction(0))
    if total != 1:
        raise PartsRefused("RUN-LABEL", ["shares that sum to 1"], f"the shares sum to {total}, not 1")
    got = split_largest(amount, {p: s for p, s in shares.items()}, step, rank)
    out = {}
    for p, a in got.items():
        if a:
            lab = (p, batch) if batch is not None else p
            parts[lab] = parts.get(lab, Decimal(0)) + a
            out[lab] = a
    return out


def take(parts, amount, order, step, prorata=False, rank=None):
    """One outflow's take (L §5.14): the parts sorted by `order(label)` (a tuple, least first) are taken in turn; a group
    tied on every term that can't all be taken is shared by `prorata` (rest largest at the step, weighted by the parts'
    amounts, ties by `rank`), else refused (RUN-TIE); more than the parts hold is RUN-PLACE. Takes from `parts` in place;
    returns {label: amount}."""
    amount = D(amount)
    held = sum(parts.values(), Decimal(0))
    if amount > held:
        raise PartsRefused("RUN-PLACE", [f"{amount} taken from {held} held"], f"the outflow of {amount} is more than the {held} held")
    groups = {}
    for lab, a in parts.items():
        if a:
            groups.setdefault(order(lab), []).append(lab)
    out, need = {}, amount
    for key in sorted(groups):
        if not need:
            break
        labs = groups[key]
        supply = sum((parts[l] for l in labs), Decimal(0))
        if supply <= need:
            got = {l: parts[l] for l in labs}
        elif len(labs) == 1:
            got = {labs[0]: need}
        elif prorata:
            got = split_largest(need, {l: parts[l] for l in labs}, step, rank)
        else:
            raise PartsRefused("RUN-TIE", ["an order term that ranks the tied parts"], f"parts {sorted(map(str, labs))} tie for one outflow")
        for l, a in got.items():
            if a:
                parts[l] -= a
                out[l] = out.get(l, Decimal(0)) + a
                need -= a
        for l in labs:
            if parts[l] == 0:
                del parts[l]
    return out


def take_batch(parts, outflows, order, step, prorata=False, rank=None):
    """The takes of one batch at one place (L §5.14): arcs (part k, outflow w) sorted by `order(k, w)`; `outflows` is
    [(name, amount)]. Arcs tied on every term: of one outflow, shared as `take` shares them; of two outflows, refused
    (RUN-TIE) when a part they tie for can't cover both. Returns {outflow: {label: amount}}."""
    total = sum((D(a) for _, a in outflows), Decimal(0))
    held = sum(parts.values(), Decimal(0))
    if total > held:
        raise PartsRefused("RUN-PLACE", [f"{total} taken from {held} held"], f"the batch's outflows of {total} are more than the {held} held")
    need = {w: D(a) for w, a in outflows}
    out = {w: {} for w, _ in outflows}
    groups = {}
    for lab, a in parts.items():
        if a:
            for w, _ in outflows:
                groups.setdefault(order(lab, w), []).append((lab, w))
    for key in sorted(groups):
        arcs = [(l, w) for l, w in groups[key] if parts.get(l) and need[w]]
        ws = sorted({w for _, w in arcs})
        if len(ws) > 1:
            for l in {l for l, _ in arcs}:
                if parts[l] < sum(need[w] for w in ws if (l, w) in arcs):
                    raise PartsRefused("RUN-TIE", ["an order term on the outflow"], f"outflows {ws} tie for a part of {l} that can't cover both")
            for l, w in arcs:
                a = min(parts[l], need[w])
                parts[l] -= a
                need[w] -= a
                out[w][l] = out[w].get(l, Decimal(0)) + a
        elif ws:
            w = ws[0]
            sub = {l: parts[l] for l, _ in arcs}
            got = take(sub, need[w], lambda _l: 0, step, prorata, rank) if sum(sub.values()) > need[w] else dict(sub)
            for l, a in got.items():
                parts[l] -= a
                need[w] -= a
                out[w][l] = out[w].get(l, Decimal(0)) + a
        for l in [l for l in parts if parts[l] == 0]:
            del parts[l]
    return out
