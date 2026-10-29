"""Probe keys, LAW axis."""
from datetime import timedelta
from decimal import Decimal

from lib.dates import P, add_months
from lib.keys import refusal
from lib.law import carry_forward, chain, progressive, table, taper
from lib.num import UNIT, D, Refuse, half_up


def LAW_01(i, t):
    return {"kind": "value", "value": [t(f"benefit at {x}", table(x, i["rows"]), "rule 1") for x in i["at"]]}


def LAW_02(i, t):
    tp, cl = i["taper"], i["cliff"]
    relief = [t(f"relief at {x}", half_up(taper(x, tp["full"], tp["limit"])), "rule 1") for x in tp["at"]]
    contrib = [t(f"contribution at {x}", table(x, cl["rows"]), "rule 2") for x in cl["at"]]
    return {"kind": "value", "value": {"relief": relief, "contribution": contrib}}


def _advance(base, v):
    return half_up(D(base) * D(v["rate"]) - D(v["relief"]))


def LAW_03(i, t, with_change=True):
    old, new = i["old"], i["new"]
    ts, te = map(P, i["transitional"])
    out = {}
    for who, month, base in i["cases"]:
        d = P(month + "-01")
        if not with_change or d <= P(old["to"]):
            a = _advance(base, old)
            how = "old version (rule 1)"
        elif ts <= d <= te:
            a = min(_advance(base, old), _advance(base, new))
            how = f"lower of old {_advance(base, old)} and new {_advance(base, new)} (rule 3)"
        else:
            a = _advance(base, new)
            how = "new version (rule 2)"
        out[f"{who} {month}"] = t(f"{who} {month}", a, how) if with_change else a
    if with_change:
        before = LAW_03(i, t, with_change=False)
        june = [k for k in out if k.endswith(old["to"][:7])]
        out["june_unchanged"] = all(out[k] == before[k] for k in june)
        t("June without the change", {k: before[k] for k in june}, "the law as before 1 July")
        return {"kind": "relation", "value": out}
    return out


def LAW_04(i, t):
    withheld = t("withheld", sum((half_up(D(i["monthly"]) * D(r)) for r in i["withheld_rates"]), Decimal(0)), "Σ monthly × rate of the month")
    due = t("due", half_up(D(i["monthly"]) * len(i["withheld_rates"]) * D(i["year_rate"])), f"{i['year_rate']} × year's income (rule 1)")
    s = withheld - due
    return {"kind": "value", "value": {"withheld": withheld, "due": due, "settlement": abs(s),
            "direction": "refund to Ania" if s > 0 else "arrears from Ania", "booked": i["filed"]}}


def LAW_05(i, t):
    taxes = {k: t(f"set {k}", half_up(D(i["income"]) * D(v["rate"]) - D(v["relief"])), f"{v['rate']} × {i['income']} − {v['relief']}") for k, v in i["sets"].items()}
    k = min(taxes, key=lambda s: (taxes[s], s))
    return {"kind": "value", "value": {**taxes, "kept": taxes[k], "set": k}}


def LAW_06(i, t):
    inc, bands = i["incomes"], i["bands"]
    sides = {"joint": half_up(2 * progressive(sum(D(v) for v in inc.values()) / 2, bands)),
             "separate": sum((half_up(progressive(v, bands)) for v in inc.values()), Decimal(0))}
    out = {}
    for part, year in (("a", "2023"), ("b", "2024")):
        e = i["elections"].get(year)
        if e is None:
            t(f"({part}) {year}", "refused", "no election stated (rule 1)")
            out[part] = refusal(Refuse([f"filing election for {year}"], "an election left unstated"))
        else:
            t(f"({part}) {year} sides", sides, "rule 2")
            out[part] = {"kind": "value", "value": {**sides, "elected": e, "tax": sides[e]}}
    return out


def LAW_07(i, t):
    cum, first, vat = D(i["before"]), None, Decimal(0)
    for no, day, amt in i["invoices"]:
        cum += D(amt)
        t(f"revenue after {no}", cum, f"+ {amt}")
        if first is None and cum > D(i["threshold"]):
            first = no
        if first is not None:
            vat += D(amt)
    vat = t("VAT", half_up(vat * D(i["rate"])), f"{i['rate']} × invoices from {first} (rules 1–2)")
    return {"kind": "value", "value": {"first_with_vat": first, "vat": vat}}


def LAW_08(i, t):
    cum, nets = Decimal(0), []
    for m in range(1, 13):
        base = max(Decimal(0), min(D(i["gross"]), D(i["cap"]) - cum))
        cum += D(i["gross"])
        c = half_up(base * D(i["pension"]))
        tax = half_up((D(i["gross"]) - c) * D(i["tax"]), UNIT)
        net = D(i["gross"]) - c - tax
        if m >= 10:
            t(f"month {m}", net, f"contribution {c} on {base}, advance {tax} (rules 1–3)")
        nets.append(net)
    return {"kind": "value", "value": {"november": nets[10], "december": nets[11]}}


def LAW_09(i, t):
    used, taxable, left = carry_forward(i["loss"], i["gains"])
    t("used", used, "min(half the original loss, the gain, what's left) (rule 1)")
    return {"kind": "value", "value": {"used": used, "taxable": taxable, "left": left}}


def LAW_10(i, t):
    closes = t("window closes", P(i["sold"]).replace(month=12, day=31).replace(year=P(i["sold"]).year + 3), "end of the third year after the sale (rule 1)")
    cases = {}
    for name, more in i["branches"].items():
        share = min(Decimal(1), (D(i["spent"]) + D(more)) / D(i["proceeds"]))
        taxable = D(i["gain"]) - D(i["gain"]) * share
        cases[name] = t(f"branch: {name}", half_up(taxable * D(i["rate"])), f"{i['rate']} × (gain − gain × {share})")
    return {"kind": "by-case", "cases": cases, "settles": closes.isoformat()}


def LAW_11(i, t):
    *earlier, (gday, gamt) = i["gifts"]
    since = add_months(gday, -12 * i["years"])
    window = [D(a) for d, a in earlier if P(d) > since]
    cum = t("sum within 5 years", sum(window, Decimal(0)) + D(gamt), f"gifts after {since} + this one (rule 1)")
    used_before = max(Decimal(0), sum(window, Decimal(0)) - D(i["free"]))
    taxable = t("taxable", max(Decimal(0), cum - D(i["free"])) - used_before, f"above {i['free']}, less earlier taxed {used_before} (rule 2)")
    return {"kind": "value", "value": {"taxable": taxable, "tax": half_up(taxable * D(i["rate"]))}}


def LAW_12(i, t):
    out = {}
    for who, f in i["heirs"].items():
        ex = chain([f["close"], f["unreported"], f["minor"]])
        out[who] = t(who, D("0.00") if ex else half_up((D(i["amount"]) - D(i["free"])) * D(i["rate"])),
                     f"exempt = close {f['close']} and not (unreported {f['unreported']} and not minor {f['minor']})")
    return {"kind": "value", "value": out}


def LAW_13(i, t):
    y0, y1 = i["years"]
    base = {y: half_up(D(i["gain"]) * D(i["base_rate"])) for y in range(y0, y1 + 1)}
    w = i["whatif"]
    wi = {y: half_up(D(i["gain"]) * D(w["rate"] if y >= w["from"] else i["base_rate"])) for y in range(y0, y1 + 1)}
    bt = t("base total", sum(base.values(), Decimal(0)), "Σ yearly tax")
    wt = t("what-if total", sum(wi.values(), Decimal(0)), f"{w['rate']} from {w['from']} (rule 1)")
    same = all(base[y] == wi[y] for y in base if y < w["from"])
    return {"kind": "relation", "value": {"base_total": bt, "whatif_total": wt, "whatif_2030": wi[2030], "before_unchanged": same}}


def LAW_20(i, t):
    """A deduction and a credit sized from each other, solved by the law's own named method (rule 3), and by the
    iteration it also allows (rule 4); the rule is linear, so closed forms check both."""
    inc, prem, thr, rate = D(i["income"]), D(i["premium"]), D(i["threshold"]), D(i["rate"])

    def credit(ded):
        return half_up(min(prem, max(Decimal(0), rate * (inc - ded - thr))))   # rule 2

    def deduction(cr):
        return max(Decimal(0), prem - cr)                                        # rule 1

    d1 = t("(a) (i) interim deduction", prem, "the premium (rule 3)")
    c1 = t("(a) (ii) interim credit", credit(d1), f"{rate} × ({inc} − {d1} − {thr}) (rule 2)")
    d2 = t("(a) (iii) final deduction", deduction(c1), f"{prem} − {c1} (rule 1)")
    c2 = t("(a) (iv) final credit", credit(d2), f"{rate} × ({inc} − {d2} − {thr}) (rule 2)")
    k = rate * (inc - prem - thr)
    closed_a = {"deduction": prem - k, "credit": k * (1 + rate)}
    agree_a = closed_a == {"deduction": d2, "credit": c2}
    t("(a) closed form of the method (rule linear, no bound reached)", {k_: half_up(v) for k_, v in closed_a.items()},
      f"credit = r(I − P − T)(1 + r), deduction = P − r(I − P − T), r {rate}, I {inc}, P {prem}, T {thr}")
    t("(a) the method's result equals its closed form", agree_a)
    ded, steps = prem, 0
    while True:
        steps += 1
        new = deduction(credit(ded))
        t(f"(b) round {steps}", {"deduction": new, "change": new - ded}, "the credit from the deduction, then the deduction from the credit (rule 4)")
        if abs(new - ded) < D("1.00"):
            ded = new
            break
        ded = new
    cb = credit(ded)
    fixed = {"deduction": prem - k / (1 - rate), "credit": k / (1 - rate)}
    t("(b) closed form: the fixed point both rules hold at", {k_: half_up(v) for k_, v in fixed.items()}, "credit = r(I − P − T)/(1 − r), deduction = P − credit")
    within = abs(ded - fixed["deduction"]) < D("1.00") and abs(cb - fixed["credit"]) < D("1.00")
    t("(b) the iteration within 1.00 of the fixed point", within)
    return {"a": {"kind": "relation", "value": {"deduction": d2, "credit": c2, "closed_form_agrees": agree_a}},
            "b": {"kind": "relation", "value": {"deduction": ded, "credit": cb, "rounds": steps, "within_1_of_fixed_point": within}}}
