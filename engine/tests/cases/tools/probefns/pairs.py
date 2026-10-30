"""Probe keys, pair cards: two edges meeting in one card (probes/pairs/<EDGE>+<EDGE>.md, see probes/README.md).

A pair card's function is named after both edges, `-` written `_` and `+` written `__`: CUR-01+RAND-01 → CUR_01__RAND_01.
"""
from datetime import timedelta
from fractions import Fraction

from lib.dates import Calendar, P
from lib.num import D, convert, half_up
from lib.paths import dec, mean_se, rng


def _walk(i, r):
    """One path's mid rates: from the start day's rate, each business day after it the day before's plus one step,
    drawn in day order: up if r.random() < chance, else down."""
    cal, d, end = Calendar(), P(i["start"]), P(i["paid"])
    rate, out = D(i["start_rate"]), {}
    out[d] = rate
    while d < end:
        d = cal.on_or_next(d + timedelta(days=1))
        rate += D(i["up"]) if r.random() < float(i["chance"]) else D(i["down"])
        out[d] = rate
    return out


def CUR_01__RAND_01(i, t):
    cal = Calendar()
    law_day = t("the law's rate day", cal.before(P(i["issued"])), f"the last business day before {i['issued']} (rule 1)")
    pay_day = P(i["paid"])
    steps = {}
    d, n = P(i["start"]), 0
    while d < pay_day:
        d = cal.on_or_next(d + timedelta(days=1))
        n += 1
        steps[d] = n
    step = (Fraction(str(i["up"])) + Fraction(str(i["down"]))) / 2 if Fraction(str(i["chance"])) == Fraction(1, 2) else None
    e_rev = t("closed form: the mean revenue", half_up(Fraction(str(i["amount"])) * (Fraction(str(i["start_rate"])) + steps[law_day] * step)),
              f"{i['amount']} × ({i['start_rate']} + {steps[law_day]} steps × {D(step)}, the mean step) (rules 1, 3)")
    e_cash = t("closed form: the mean cash", half_up(Fraction(str(i["amount"])) * (Fraction(str(i["start_rate"])) + steps[pay_day] * step - Fraction(str(i["spread"])))),
               f"{i['amount']} × ({i['start_rate']} + {steps[pay_day]} steps × {D(step)} − {i['spread']}) (rules 2, 3)")

    def run():
        r = rng(i["seed"])
        rev, cash = [], []
        for _ in range(i["paths"]):
            w = _walk(i, r)
            rev.append(convert(i["amount"], w[law_day]))
            cash.append(convert(i["amount"], w[pay_day] - D(i["spread"])))
        return rev, cash

    rev, cash = run()
    out = {}
    for name, xs, p in (("revenue", rev, i["p"]["revenue"]), ("cash", cash, i["p"]["cash"])):
        m, se = mean_se([float(x) for x in xs])
        out[name] = {"kind": "random", "r": dec(m, 2), "se_r": dec(se, 2), "p": D(p)}
        t(f"seeded mean {name}", out[name], f"{i['paths']} paths, seed {i['seed']} (rule 4); each path's amount to the ort (rule 3)")
    same = run() == (rev, cash)
    t("same seed, same answer", same, "run twice")
    return {"kind": "value", "value": {**out, "same_seed_same_answer": same}}


def PTY_05__OWN_12(i, t):
    """A nested joint model (PTY-05 × OWN-12): an outer joint model whose member is itself a joint model reads one
    party's parts across every member, the party stated `same` at each level (L §6.1, §3.6; E §3.1 step 1)."""
    from decimal import Decimal
    from probefns.own import _Book, _community, CENT
    parts = {}
    for m, mi in i["members"].items():
        book = _Book({mi["place"]: CENT}, [mi["place"]], [], lambda x, lab, row: (0,), _community(mi))
        book.walk(mi["rows"])
        parts[m] = {p: book.bal(mi["place"], i["asked"], p) for p in mi["parties"]}
        t(f"member {m}: parts of {mi['place']} at the end of {i['asked']}", parts[m], "the member alone (its own rules 1-2)")
    inner = i["inner"]
    in_sum = t(f"the inner joint model {inner['name']}: {inner['party']}'s parts across its members",
               sum((parts[m].get(q, Decimal(0)) for m, q in inner["same"].items()), Decimal(0)),
               " + ".join(f"{m}: {q}" for m, q in inner["same"].items()) + " (stated same in the inner header, L §6.1)")
    outer = i["outer"]
    b = t(f"the outer joint model: {outer['party']}'s parts at member {outer['member']}",
          parts[outer["member"]].get(outer["same"], Decimal(0)), f"{outer['party']} = {outer['member']}.{outer['same']} (stated same in the outer header)")
    total = t("the party's parts across the whole family", in_sum + b, "inner + outer member (rule 3)")
    return {"kind": "value", "value": {"members": parts, "inner": in_sum, "outer_member": b, "total": total}}
