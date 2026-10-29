"""Probe keys, PTY axis."""
from decimal import Decimal

from lib.dates import P
from lib.law import progressive
from lib.num import D, half_up, split
from lib.own import estate


def PTY_01(i, t):
    m = P(i["married"])
    first = t("first year married throughout", m.year if (m.month, m.day) == (1, 1) else m.year + 1, f"married {m}")
    last = t("first year married on its last day", m.year, "contrast")
    return {"kind": "value", "value": {"throughout": first, "on_last_day": last}}


def PTY_02(i, t):
    inc, bands = i["incomes"], i["bands"]
    half = sum((D(v) for v in inc.values()), Decimal(0)) / 2
    joint = t("joint tax", half_up(2 * progressive(half, bands)), f"2 × scale({half}) (rule 2)")
    alone = {p: t(f"{p} alone", half_up(progressive(v, bands)), f"scale({v})") for p, v in inc.items()}
    parts = split(joint, list(alone.values()), remainder="largest")
    out = dict(zip(inc, parts))
    t("parts", out, "joint × alone / Σ alone, largest remainders (rule 3)")
    assert sum(parts) == joint
    return {"kind": "value", "value": {"joint": joint, "alone": alone, "parts": out}}


def PTY_03(i, t):
    s = estate(i["children"], spouse=i["spouse"])
    t("shares", s, "spouse ≥ 1/4, children equal, Adam represented by Kuba (rules 1–2)")
    assert sum(s.values()) == 1
    return {"kind": "value", "value": s}


def PTY_04(i, t):
    out = {}
    for case in ("before", "after"):
        joint = t(f"{case}: joint income", sum((D(v) for v in i[case].values()), Decimal(0)), " + ".join(str(v) for v in i[case].values()))
        ok = joint <= D(i["limit"])
        r = {"eligible": ok}
        for p, s in i["split"].items():
            r[p] = t(f"{case}: {p}", half_up(D(i["relief"]) * D(s)) if ok else D("0.00"), f"not more than {i['limit']}: {ok} (rules 1–2)")
        out[case] = r
    return {"kind": "value", "value": out}


def PTY_05(i, t):
    union = {}
    for h in ("D", "A"):  # the deriving household's statement stands (rule 1)
        for x in i[h]:
            k = (x["date"], x["from"], x["to"], D(x["amount"]))
            if k not in union or x["stated"] == "derived":
                union[k] = x
    t("union transfers", len(union), "matched on date, from, to, amount")
    change = {}
    for (d, f, to, a) in union:
        # R2: a running sum's start: every transfer the card's statements list is added, none is read as 0
        change[f] = change.get(f, Decimal(0)) - a
        change[to] = change.get(to, Decimal(0)) + a
    own = {}
    for h, person in i["people"].items():
        own[person] = sum((D(x["amount"]) * (1 if x["to"] == person else -1) for x in i[h]), Decimal(0))
    holds = all(own[p] == change[p] for p in own)
    t("each household's own key", own, "from its own statements")
    return {"kind": "relation", "value": {"union_transfers": len(union), **{p: change[p] for p in sorted(change)},
            "family_total": sum(change.values(), Decimal(0)), "keys_hold": holds}}
