"""Probe keys, OWN axis."""
from decimal import Decimal
from fractions import Fraction

from lib.dates import P, add_months, days, month_key, months_back
from lib.num import UNIT, D, F, half_up
from lib.own import fifo, room
from lib.rules import balances, minimum


def OWN_01(i, t):
    sav = i["places"][1]
    q0, q1 = i["quarter"]
    months = [m for m in ("2025-01", "2025-02", "2025-03") if q0 <= m <= q1 and m >= month_key(sav["opened"])]
    fees = t("fees", D(sav["fee"]) * len(months), f"{sav['fee']} × month ends while open {months} (rule 2)")
    asked = P(i["asked"])
    lines = [p["id"] for p in i["places"] if P(p["opened"]) <= asked]
    nw = t("net worth", sum((D(p["balance_on"][i["asked"]]) for p in i["places"] if p["id"] in lines), Decimal(0)), f"places open on {asked}: {lines} (rule 1)")
    return {"kind": "value", "value": {"fees_q1": fees, "net_worth": nw, "lines": lines}}


def OWN_02(i, t):
    nw = t("net worth", D(i["cash"]) + D(i["claim"]), "cash + claim (rule 1)")
    d, b = minimum(balances(i["cash"], i["items"], counted={"planned"}))
    t("lowest ahead", b, f"planned items only, on {d} (rule 2)")
    return {"kind": "value", "value": {"net_worth": nw, "lowest_ahead": b, "lowest_on": d.isoformat()}}


def OWN_03(i, t):
    nw = t("net worth 31 December", D(i["cash"]) - D(i["tax"]), "cash − tax owed (rule 1)")
    owed = D(i["tax"]) if P(i["due"]) > P(i["run_end"]) else D(0)
    t("owed at run end", owed, f"due {i['due']} after run end {i['run_end']} (rule 2)")
    return {"kind": "value", "value": {"net_worth_dec31": nw, "owed_at_run_end": owed, "due": i["due"]}}


def OWN_04(i, t):
    out = {}
    for regime, shares in i["regimes"].items():
        out[regime] = {p: t(f"{regime}: {p}", half_up(D(i["value"]) * D(F(s))), f"{i['value']} × {s}") for p, s in shares.items()}
    return {"kind": "value", "value": out}


def OWN_05(i, t):
    keeper = i["keeper"]
    shares = {p: F(s) / 2 + (Fraction(1, 2) if p == keeper else 0) for p, s in i["estate"].items()}
    t("shares after the death", shares, "own half + half × estate share (rules 1–2)")
    pays = {p: t(f"payout {p}", half_up(D(s * F(D(i["value_at_division"])))), f"{s} × {i['value_at_division']} (rule 3)") for p, s in shares.items() if p != keeper}
    other = next(s for p, s in shares.items() if p != keeper)
    return {"kind": "value", "value": {"shares": shares, "payouts": pays, "keeper_pays": sum(pays.values(), Decimal(0)),
            "payout_at_death_value": half_up(D(other * F(D(i["value_at_death"]))))}}


def OWN_06(i, t):
    cost = t("cost", D(i["units"]) * D(i["gift_day_price"]), "units × gift-day price (rule 1)")
    proceeds = D(i["units"]) * D(i["sale_price"])
    gain = t("gain", proceeds - cost, f"{proceeds} − {cost}")
    return {"kind": "value", "value": {"cost": cost, "gain": gain, "gain_on_giver_cost": proceeds - D(i["giver_cost"])}}


def OWN_07(i, t):
    s = i["sell"]
    proceeds = D(s["units"]) * D(s["price"])
    cost, taken = fifo(i["lots"], s["units"], account=s["account"])
    t("lots taken, per account", taken, "FIFO within the account (rule 1)")
    c2, taken2 = fifo(i["lots"], s["units"])
    t("lots taken, across accounts", taken2, "contrast")
    return {"kind": "value", "value": {"cost": half_up(cost), "gain": half_up(proceeds - cost), "gain_across_accounts": half_up(proceeds - c2)}}


def OWN_08(i, t):
    dec = t("15 December 2024", room(i["used"]["2024"], i["limits"]["2024"], i["want"]), "min(want, limit − used) (rule 1)")
    jan = t("15 January 2025", room(i["used"]["2025"], i["limits"]["2025"], i["want"]), "min(want, limit − used) (rule 1)")
    w = i["withdrawal"]
    fc = min(D(w["amount"]), D(w["contributions"]))
    fe = t("from earnings", D(w["amount"]) - fc, "contributions first (rule 2)")
    tax = t("tax", half_up(fe * D(w["rate"])), f"{w['rate']} × {fe}")
    return {"kind": "value", "value": {"dec_2024": dec, "jan_2025": jan, "withdrawal": {"from_contributions": fc, "from_earnings": fe, "tax": tax}}}


def OWN_09(i, t):
    value, start, end = D(i["face"]), P(i["bought"]), P(i["redeemed"])
    year, interest = 1, Decimal(0)
    while True:
        ystart, ynext = add_months(start, 12 * (year - 1)), add_months(start, 12 * year)
        if year == 1:
            rate = D(i["first_rate"])
        else:
            ref = months_back(month_key(ystart), i["lag_months"])
            rate = t(f"year {year} rate", D(i["index"][ref]) + D(i["margin"]), f"index {ref} + margin (rule 1)")
        if ynext <= end:
            inc = t(f"year {year} interest a bond", half_up(value * rate), f"{value} × {rate} (rule 2)")
        else:
            n, of = days(ystart, end), days(ystart, ynext)
            inc = t(f"year {year} part interest a bond", half_up(value * rate * n / of), f"{value} × {rate} × {n}/{of} (rule 2)")
        value += inc
        interest += inc
        if ynext >= end:
            break
        year += 1
    fee = t("fee a bond", min(D(i["fee"]), interest), "capped at the interest (rule 3)")
    tax = t("tax", half_up(D(i["tax_rate"]) * i["bonds"] * (interest - fee), UNIT), f"{i['tax_rate']} × {i['bonds']} × ({interest} − {fee}) (rule 4)")
    payout = t("payout", i["bonds"] * value - i["bonds"] * fee - tax, f"{i['bonds']} × {value} − fees − tax")
    return {"kind": "value", "value": {"interest_per_bond": interest, "fee_per_bond": fee, "tax": tax, "payout": payout}}
