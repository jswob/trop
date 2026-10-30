"""Probe keys, OWN axis."""
from datetime import timedelta
from decimal import Decimal
from fractions import Fraction

from lib.dates import P, add_months, days, month_key, months_back
from lib.num import UNIT, D, F, half_even, half_up
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


# --------------------------------------------------------------------------- tally 2.1: places that keep parts (session 7b)
# OWN-10 (parts that arrive), OWN-11 (parts that leave), OWN-12 (reading parts). Each card has one part per item of the
# round's probe table (P-01 .. P-24); each part is keyed separately (probes/README.md). The walk below books a card's
# rows day by day as L §3.6, §5.14 and E §4.2 say: the day's first batch (observations, plans) together, its arrivals
# from places that keep no parts first, then each place's takes over all the batch's outflows there, then the arrivals
# that read a take (kept labels, relabels); decisions after the batch, each at once.

from lib.own import PartsRefused, arrive as _arrive, split_largest, take as _take, take_batch as _take_batch  # noqa: E402

CENT = Decimal("0.01")


def _legs(row):
    if "legs" in row:
        return row["legs"]
    return [{"name": "cash", "from": row["from"], "to": row["to"], "amount": row["amount"]}]


def _fr(x):
    return x if isinstance(x, Fraction) else F(D(x)) if not isinstance(x, str) or "/" not in x else Fraction(x)


class _Book:
    """Parts per place, booked day by day. `keeps` {place: step}; `owned`, `batched`: the places that keep owners or
    batches (labels: owner, batch, or the pair); `order(x, label, row)` the leaving rule's tuple; `arrival(row, leg,
    book)` the arrival rule's shares {party: Fraction}; `prorata` and `take_rank` the takes clause's end; `arrive_rank`
    the owned clause's ties by."""

    def __init__(self, keeps, owned, batched, order, arrival, prorata=False, take_rank=None, arrive_rank=None):
        self.keeps, self.owned, self.batched = keeps, set(owned), set(batched)
        self.order, self.arrival = order, arrival
        self.prorata, self.take_rank, self.arrive_rank = prorata, take_rank, arrive_rank
        self.parts = {x: {} for x in keeps}
        self.took = {}       # (row id, leg) -> {label: amount}
        self.days = []       # [(day, {place: parts at the end of the day})]
        self.start = None    # parts at the start of the day being booked

    def snap(self):
        return {x: dict(p) for x, p in self.parts.items()}

    def at(self, d, start=False):
        """Parts at the end of day d (at its start with start=True)."""
        out = {x: {} for x in self.keeps}
        for day, s in self.days:
            if day < d or (day == d and not start):
                out = s
        return out

    def bal(self, x, d, q=None, start=False):
        ps = self.at(d, start)[x]
        if q is None:
            return sum(ps.values(), Decimal(0))
        return sum((a for l, a in ps.items() if l == q or (isinstance(l, tuple) and q in l)), Decimal(0))

    def _arrive(self, row, leg, took=None):
        x = leg["to"]
        if x in self.owned:
            shares = self.arrival(row, leg, self)
        else:
            shares = None
        step = self.keeps[x]
        if took is not None and leg["from"] in self.batched and x in self.batched:
            # batches kept: each taken part's batch arrives again, labelled anew by the arrival rule
            for lab, a in took.items():
                b = lab[1] if isinstance(lab, tuple) else lab
                if shares is None:
                    self.parts[x][b] = self.parts[x].get(b, Decimal(0)) + a
                else:
                    _arrive(self.parts[x], a, shares, step, self.arrive_rank, batch=b)
            return
        batch = row["id"] if x in self.batched else None
        if shares is None:
            self.parts[x][batch] = self.parts[x].get(batch, Decimal(0)) + D(leg["amount"])
        else:
            _arrive(self.parts[x], D(leg["amount"]), shares, step, self.arrive_rank, batch=batch)

    def _takes(self, x, outs):
        step = self.keeps[x]
        if len(outs) == 1:
            r, leg = outs[0]
            self.took[(r["id"], leg["name"])] = _take(self.parts[x], D(leg["amount"]), lambda l: self.order(x, l, r), step,
                                                      self.prorata, self.take_rank)
        elif outs:
            byname = {f"{r['id']}.{leg['name']}": (r, leg) for r, leg in outs}
            got = _take_batch(self.parts[x], [(n, D(leg["amount"])) for n, (r, leg) in byname.items()],
                              lambda l, n: self.order(x, l, byname[n][0]), step, self.prorata, self.take_rank)
            for n, g in got.items():
                r, leg = byname[n]
                self.took[(r["id"], leg["name"])] = g

    def batch(self, day, rows):
        """The day's first batch: every row dated `day` at once. A movement that takes from no place that keeps parts
        arrives first; then each place's takes over all the batch's outflows there; then the arrivals of the movements
        that took (their arrival rule may read the take, `m.leg.amount of q`), kept labels and relabels among them."""
        self.start = self.snap()
        rows = [r for r in rows if r["on"] == day]
        took = [r for r in rows if any(leg["from"] in self.keeps for leg in _legs(r))]
        for r in rows:
            if r not in took:
                for leg in _legs(r):
                    if leg["to"] in self.keeps:
                        self._arrive(r, leg)
        for x in self.keeps:
            self._takes(x, [(r, leg) for r in rows for leg in _legs(r) if leg["from"] == x])
        for r in took:
            for leg in _legs(r):
                if leg["to"] in self.keeps:
                    self._arrive(r, leg, self.took[(r["id"], leg["name"])] if leg["from"] in self.keeps else None)

    def decision(self, row):
        """A decision at its point: its takes and arrivals at once, against the parts as they stand."""
        for leg in _legs(row):
            if leg["from"] in self.keeps:
                self._takes(leg["from"], [(row, leg)])
        for leg in _legs(row):
            if leg["to"] in self.keeps:
                self._arrive(row, leg, self.took.get((row["id"], leg["name"])) if leg["from"] in self.keeps else None)

    def close(self, day):
        self.days.append((day, self.snap()))

    def walk(self, rows, decisions=None, until=None):
        """Every day with a row or a decision, in order; `decisions` {day: [fn(book, day) -> row or None]}."""
        decisions = decisions or {}
        for day in sorted({r["on"] for r in rows} | set(decisions)):
            if until and day > until:
                break
            self.batch(day, rows)
            for fn in decisions.get(day, []):
                row = fn(self, day)
                if row:
                    self.decision(row)
            self.close(day)


def _refused(e, t, tag):
    t(f"{tag} refused", e.code, e.reason)
    return {"kind": "refusal", "code": e.code, "missing": e.missing, "reason": e.reason}


def _value(v, **kw):
    return {"kind": "value", "value": v, **kw}


def _community(i):
    """L §3.6's example rules: an inheritance the heir's, money moved from an owned account as it was, the rest the couple's."""
    def shares(row, leg, book):
        ps = i["parties"]
        if row["kind"] in i.get("moved_kinds", ["transfers"]) and leg["from"] in book.owned:
            took = book.took[(row["id"], leg["name"])]
            tot = sum(took.values(), Decimal(0))
            return {p: Fraction(F(sum((a for l, a in took.items() if l == p), Decimal(0))), F(tot)) for p in ps}
        if row["kind"] == "inheritances":
            return {p: Fraction(int(p == row["heir"])) for p in ps}
        if i.get("community_rule", True) or row["kind"] in i.get("couple_kinds", []):
            return {p: Fraction(int(p == i["couple"])) for p in ps}
        raise PartsRefused("RUN-NO-RULE", [f"a rule of {i['rule']} for {row['id']} ({row['kind']})"],
                           f"no rule applies to {i['rule']}[{row['id']}, p] on {row['on']}")
    return shares


def _couple_first(i):
    return lambda x, lab, row: (0 if lab == i["couple"] else 1,)


def _parts_of(book, x, d, parties):
    return {p: book.bal(x, d, p) for p in parties}


def _rank(table):
    return (lambda k: D(table[k[0] if isinstance(k, tuple) else k])) if table else None


def _o10_p01(i, t, tag="(P-01)"):
    book = _Book({"sav": CENT}, ["sav"], [], _couple_first(i), _community(i))
    book.walk(i["rows"])
    parts = t(f"{tag} parts of sav at the end of {i['asked']}", _parts_of(book, "sav", i["asked"], i["parties"]),
              "an inheritance the heir's, interest and the rest the couple's; the withdrawal takes the couple's part first (rules 1-2)")
    return _value({"parts": parts, "total": book.bal("sav", i["asked"])})


def _o10_p02(i, t, tag="(P-02)"):
    book = _Book({"sav_a": CENT, "sav_b": CENT}, ["sav_a", "sav_b"], [], _couple_first(i), _community(i))
    book.walk(i["rows"])
    a = t(f"{tag} parts of sav_a at the end of {i['asked']}", _parts_of(book, "sav_a", i["asked"], i["parties"]), "the transfer took the couple's part first (rule 2)")
    b = t(f"{tag} parts of sav_b at the end of {i['asked']}", _parts_of(book, "sav_b", i["asked"], i["parties"]),
          "the transfer's arrival keeps its labels: each party's share is what the leg took of it over what it took (rule 1)")
    return _value({"sav_a": a, "sav_b": b})


def _o10_p03(i, t, tag="(P-03)"):
    def shares(row, leg, book):
        if row["kind"] == "fund_buys":
            took = [book.took[(row["id"], n)] for n in ("cash_a", "cash_b")]
            paid = sum((D(lg["amount"]) for lg in _legs(row) if lg["name"] in ("cash_a", "cash_b")), Decimal(0))
            return {p: Fraction(F(sum((tk.get(p, Decimal(0)) for tk in took), Decimal(0))), F(paid)) for p in i["parties"]}
        return _community(i)(row, leg, book)
    book = _Book({"acc_ania": CENT, "acc_tomek": CENT, "fund": D(i["unit_step"])}, ["acc_ania", "acc_tomek", "fund"], [], _couple_first(i), shares)
    book.walk(i["rows"])
    f = t(f"{tag} units of the fund by owner at the end of {i['asked']}", _parts_of(book, "fund", i["asked"], i["parties"]),
          "each owner's share is what the purchase's cash legs took of their parts over what they paid; split rest largest to the step of 1 unit (rules 1-2)")
    return _value({"fund": f, "total": book.bal("fund", i["asked"])})


def _o10_p07(i, t, tag="(P-07)"):
    day = {r["id"]: r["on"] for r in i["rows"]}

    def shares(row, leg, book):
        who = row["buyer"] if row["kind"] == "fund_buys" else row["receiver"] if row["kind"] == "gifts" else i["couple"]
        return {p: Fraction(int(p == who)) for p in i["parties"]}

    def order(x, lab, row):
        return (0 if row["kind"] == "gifts" and lab[0] == row["giver"] else 1, day[lab[1]])
    book = _Book({"fund": D(i["unit_step"])}, ["fund"], ["fund"], order, shares)
    book.walk(i["rows"])
    d = i["asked"]
    pairs = {f"{p}, {b}": book.at(d)["fund"].get((p, b), Decimal(0)) for p in i["parties"] for b in i["batches"]}
    t(f"{tag} parts of the fund by (owner, batch) at the end of {d}", pairs,
      "the gift takes the giver's parts first, oldest batch first, and returns them labelled the receiver's, batches kept (rules 1-3)")
    tax = {}
    for g in [r for r in i["rows"] if r["kind"] == "gifts"]:
        took = book.took[(g["id"], "what")]
        for p in i["parties"]:
            units = sum((a for l, a in took.items() if l[0] == p), Decimal(0))
            v = units * D(g["unit_value"])
            tax[p] = t(f"{tag} gift tax on {g['id']} for giver {p}",
                       half_up(max(Decimal(0), v - D(i["gift_allowance"])) * D(i["gift_rate"])),
                       f"round(max(0, {units} units given × {g['unit_value']} − {i['gift_allowance']}) × {i['gift_rate']}, 0.01, half up) (rule 4)")
    return _value({"parts": pairs, "total": book.bal("fund", d), "gift_tax": tax})


def _o10_deaths(i, t, tag, extra=None):
    """P-08 and P-24: a death's relabel to the estate, then a division per asset (rules 1-5)."""
    dt, dv = i["death"], i["division"]

    def shares(row, leg, book):
        k = row["kind"]
        if k == "inheritances":
            who = row["heir"]
        elif k == "fund_buys":
            who = row["buyer"]
        elif k == "deaths":
            who = row["estate"]
        elif k == "wages_in":
            who = row["employee"]
        elif k == "divisions":
            sh = dv["shares"][leg["to"]]
            return {p: _fr(sh.get(p, "0")) for p in i["parties"]}
        else:
            raise PartsRefused("RUN-NO-RULE", [f"a rule for {row['id']}"], f"no rule applies to {row['id']}")
        return {p: Fraction(int(p == who)) for p in i["parties"]}

    def order(x, lab, row):
        return (0 if row["kind"] == "deaths" and lab == row["deceased"] else 1,
                0 if row["kind"] == "divisions" and lab == row["estate"] else 1)
    places = {"sav": CENT, "fund": D(i["unit_step"])}
    book = _Book(places, list(places), [], order, shares)

    def post(kind, x, who, extra_attrs):
        def fn(bk, day):
            amt = bk.bal_now(x, who)
            return {"id": f"{kind}_{x}", "on": day, "kind": kind, **extra_attrs,
                    "legs": [{"name": "what", "from": x, "to": x, "amount": amt}]}
        return fn
    book.bal_now = lambda x, q: sum((a for l, a in book.parts[x].items() if l == q), Decimal(0))
    dec = {dt["day"]: [post("deaths", x, dt["deceased"], {"deceased": dt["deceased"], "estate": dt["estate"]}) for x in places],
           dv["day"]: [post("divisions", x, dt["estate"], {"estate": dt["estate"], "agreement": dv["agreement"]}) for x in places]}
    if extra:
        dec[dt["day"]].append(extra)
    book.walk(i["rows"], dec)
    return book


def _o10_p08(i, t, tag="(P-08)"):
    book = _o10_deaths(i, t, tag)
    out = {}
    for d in i["asked"]:
        out[d] = {x: t(f"{tag} parts of {x} at the end of {d}", _parts_of(book, x, d, i["parties"]),
                       "the death posting relabels all the deceased's parts to the estate (rules 2-3); the division relabels the estate's parts to each heir by the agreement's share for that place, rest largest (rules 4-5)")
                  for x in ("sav", "fund")}
    return _value(out)


def _o10_p24(i, t, tag="(P-24)"):
    fp = i["final_pay"]
    refused = {}

    def late(bk, day):
        # after the death posting that day, the employer's last wage arrives, labelled the employee's (rule 6)
        if bk.bal_now("sav", i["death"]["deceased"]) == 0:
            if fp["employee"] == i["death"]["deceased"]:
                e = PartsRefused("RUN-LIFESPAN", [f"{fp['employee']}'s part labelled after his death's posting on {day}"],
                                 f"final_pay labels a part of sav to {fp['employee']} after his death's posting on {day}")
                refused["sav"] = e
                return None
            return {"id": "final_pay", "on": day, "kind": "wages_in", "employee": fp["employee"],
                    "legs": [{"name": "cash", "from": "employer", "to": "sav", "amount": fp["amount"]}]}
        return None
    book = _o10_deaths(i, t, tag, extra=late)
    out = {}
    for d in i["asked"]:
        row = {}
        for x in ("sav", "fund"):
            if x in refused and d >= i["death"]["day"]:
                row[x] = _refused(refused[x], t, f"{tag} sav at the end of {d}:")
            else:
                row[x] = _value(t(f"{tag} parts of {x} at the end of {d}", _parts_of(book, x, d, i["parties"]), "as P-08"))
        out[d] = row
    return out


def _o10_p09(i, t, tag="(P-09)"):
    dv = i["divorce"]

    def shares(row, leg, book):
        if row["kind"] == "divorces":
            return {p: _fr(i["stake"].get(p, "0")) for p in i["parties"]}
        return _community(i)(row, leg, book)

    def order(x, lab, row):
        return (0 if row["kind"] == "divorces" and lab == row["couple"] else 1,)
    book = _Book({"sav": CENT}, ["sav"], [], order, shares, arrive_rank=_rank(i["odd_step_rank"]))

    def post(bk, day):
        amt = sum((a for l, a in bk.parts["sav"].items() if l == i["couple"]), Decimal(0))
        return {"id": "divorce_sav", "on": day, "kind": "divorces", "couple": i["couple"],
                "legs": [{"name": "what", "from": "sav", "to": "sav", "amount": amt}]}
    book.walk(i["rows"], {dv["day"]: [post]})
    parts = t(f"{tag} parts of sav at the end of {i['asked']}", _parts_of(book, "sav", i["asked"], i["parties"]),
              "the divorce posting relabels the couple's parts to each spouse by stake; the odd step to the lower odd_step_rank (rules 2-4)")
    return _value({"parts": parts, "total": book.bal("sav", i["asked"])})


def _o10_p11(i, t, tag="(P-11)"):
    def shares(row, leg, book):
        return {p: Fraction(int(p == row["buyer"])) for p in i["parties"]}
    book = _Book({"fund": D(i["unit_step"])}, ["fund"], [], _couple_first(i), shares)
    book.walk(i["rows"])
    out = {}
    for d in i["asked"]:
        px = D([v for day, v in sorted(i["px"].items()) if day <= d][-1])
        units = _parts_of(book, "fund", d, i["parties"])
        out[d] = {"units": units, "price": px,
                  "value": t(f"{tag} value by owner at the end of {d}", {p: u * px for p, u in units.items()}, f"units × the last price on or before {d}, {px} (rules 2-3)")}
    first, last = i["asked"][0], i["asked"][-1]
    out["gain"] = t(f"{tag} gain by owner", {p: out[last]["value"][p] - out[first]["value"][p] for p in i["parties"]}, f"value at {last} − value at {first}: units × the price's rise; nothing booked (rule 3)")
    return _value(out)


def _reading_parts(book, x, booked, total, stated):
    """L §5.19: an anchor at a reading sets the total; a stated part is q's balance, its difference from q's booked part
    q's own gap; the unstated rest has its own gap. Returns (parts, gap, own gaps, rest gap); a rest gap ≠ 0 with no
    stated parts is unknown money, returned as None (a hole part)."""
    gap = D(total) - sum(booked.values(), Decimal(0))
    own = {q: D(v) - booked.get(q, Decimal(0)) for q, v in stated.items()}
    rest_booked = sum((a for q, a in booked.items() if q not in stated), Decimal(0))
    rest_gap = (D(total) - sum((D(v) for v in stated.values()), Decimal(0))) - rest_booked
    parts = {q: a for q, a in booked.items() if q not in stated}
    parts.update({q: D(v) for q, v in stated.items()})
    return parts, gap, own, rest_gap


def _o10_readings(i, t, tag):
    book = _Book({"sav": CENT}, ["sav"], [], _couple_first(i), _community(i))
    op = i["opening"]
    book.parts["sav"] = {q: D(v) for q, v in op["parts"].items()}
    assert sum(book.parts["sav"].values(), Decimal(0)) == D(op["total"])
    t(f"{tag} opening parts of sav at {op['day']} start", dict(book.parts["sav"]), f"stated beside the opening total {op['total']} (rule 1)")
    book.close((P(op["day"]) - timedelta(days=1)).isoformat())
    book.walk(i["rows"])
    r = i["reading"]
    booked = dict(book.at(r["day"])["sav"])
    t(f"{tag} booked parts of sav at the end of {r['day']}", booked, "the opening parts, the deposit the couple's, the withdrawal the couple's part first (rules 1-3)")
    parts, gap, own, rest_gap = _reading_parts(book, "sav", booked, r["total"], r.get("parts", {}))
    t(f"{tag} the total's gap", gap, f"stated {r['total']} − booked (rule 4)")
    t(f"{tag} each stated part's own gap", own, "stated part − booked part (rule 5)")
    t(f"{tag} the unstated rest's gap", rest_gap, "(stated total − stated parts) − the rest's booked parts (rule 5)")
    assert rest_gap == 0, "the probe keeps the rest's gap at 0"
    after = t(f"{tag} parts of sav after the reading", {p: parts.get(p, Decimal(0)) for p in i["parties"]}, "an anchor whose gaps are explained keeps the parts (rules 4-5)")
    return {"booked": {p: booked.get(p, Decimal(0)) for p in i["parties"]}, "gap": gap, "own_gaps": own, "parts": after}


def _o10_p13(i, t, tag="(P-13)"):
    return _value(_o10_readings(i, t, tag))


def _o10_p14(i, t, tag="(P-14)"):
    return _value(_o10_readings(i, t, tag))


def _o10_p15(i, t, tag="(P-15)"):
    def run(cut):
        sep = dict(i["separate"]["rows"])
        for v in i["separate"]["later"]:
            if v["known"] <= cut:
                sep.update(v["rows"])

        def shares(row, leg, book):
            if row["kind"] == "family_gifts" and sep[row["deed"]]:
                return {p: Fraction(int(p == row["recipient"])) for p in i["parties"]}
            return _community(i)(row, leg, book)

        def order(x, lab, row):
            return (0 if row["kind"] == "sweeps" and lab == row["for_whom"] else 1, 0 if lab == i["couple"] else 1)
        book = _Book({"sav": CENT, "acc_ania": CENT}, ["sav", "acc_ania"], [], order, shares)
        sw = i["sweep"]

        def sweep(bk, day):
            amt = sum((a for l, a in bk.parts["sav"].items() if l == sw["for_whom"]), Decimal(0))
            return {"id": "sweep", "on": day, "kind": "sweeps", "for_whom": sw["for_whom"],
                    "legs": [{"name": "cash", "from": "sav", "to": "acc_ania", "amount": amt}]}
        book.walk(i["rows"], {sw["day"]: [sweep]})
        d = i["asked"]
        return {"separate": sep,
                "sweep": book.took.get(("sweep", "cash"), {}),
                "sav": _parts_of(book, "sav", d, i["parties"]), "acc_ania": _parts_of(book, "acc_ania", d, i["parties"])}
    out = {}
    for name, cut in (("as_known_on", i["on"]), ("as_known_before", i["as_known_before"])):
        r = run(cut)
        t(f"{tag} cut {cut}: the deed rows seen", r["separate"], "the ruling, known later, replaces the row (rule 3)")
        t(f"{tag} cut {cut}: the sweep took", r["sweep"], "the past decision reads Ania's part at the walk's cut, so it follows the ruling (rule 4)")
        out[name] = {"sav": t(f"{tag} cut {cut}: parts of sav at the end of {i['asked']}", r["sav"], "labels are the walk's at the cut (rules 1-3)"),
                     "acc_ania": t(f"{tag} cut {cut}: parts of acc_ania at the end of {i['asked']}", r["acc_ania"], "the sweep's arrival keeps its labels (rule 2)")}
    return _value(out)


def _o10_p16(i, t, tag="(P-16)"):
    def shares(row, leg, book):
        if row["kind"] == "distributions":
            st = book.start[row["paid_by"]]
            tot = sum(st.values(), Decimal(0))
            return {p: Fraction(F(st.get(p, Decimal(0))), F(tot)) for p in i["parties"]}
        if row["kind"] == "fund_buys":
            return {p: Fraction(int(p == row["buyer"])) for p in i["parties"]}
        return {p: Fraction(int(p == i["couple"])) for p in i["parties"]}
    book = _Book({"fund": D(i["unit_step"]), "sav": CENT}, ["fund", "sav"], [], _couple_first(i), shares)
    book.walk(i["rows"])
    d = i["asked"]
    inc = next(r for r in i["rows"] if r["kind"] == "distributions")
    st = t(f"{tag} units of the fund at the start of {inc['on']}", {p: book.bal("fund", inc["on"], p, start=True) for p in i["parties"]},
           "before that day's movements (rule 2)")
    end = {p: book.bal("fund", inc["on"], p) for p in i["parties"]}
    t(f"{tag} for contrast: units at the end of {inc['on']}", end, "the same day's purchase is in them; the rule doesn't read them")
    sav = t(f"{tag} parts of sav at the end of {d}", _parts_of(book, "sav", d, i["parties"]),
            f"the income of {inc['amount']} split by the start-of-day units, rest largest to 0.01 (rules 1-2)")
    return _value({"fund_at_start": st, "sav": sav, "fund": _parts_of(book, "fund", d, i["parties"])})


def _o10_p18(i, t, tag="(P-18)"):
    out = {}
    for d in i["asked"]:
        book = _Book({"sav": CENT}, ["sav"], [], _couple_first(i), _community(i))
        try:
            book.walk(i["rows"], until=d)
            out[d] = _value(t(f"{tag} parts of sav at the end of {d}", _parts_of(book, "sav", d, i["parties"]), "rules 1-2 reach every arrival so far"))
        except PartsRefused as e:
            out[d] = _refused(e, t, f"{tag} parts of sav at the end of {d}:")
    return out


def OWN_10(i, t):
    """OWN-10: parts that arrive, labelled by a law written once (L §3.6, §5.10, §5.14, §5.19)."""
    fns = {"p01": _o10_p01, "p02": _o10_p02, "p03": _o10_p03, "p07": _o10_p07, "p08": _o10_p08, "p09": _o10_p09,
           "p11": _o10_p11, "p13": _o10_p13, "p14": _o10_p14, "p15": _o10_p15, "p16": _o10_p16, "p18": _o10_p18,
           "p24": _o10_p24}
    return {k: fns[k](i[k], t, f"({k[0].upper()}-{k[1:]})") for k in i}


def _takes_of(book, rows, parties):
    return {r["id"]: {p: book.took.get((r["id"], "cash"), {}).get(p, Decimal(0)) for p in parties} for r in rows}


def _o11_tax(i, t, tag, w_term):
    def order(x, lab, row):
        o = (0 if lab == i["couple"] else 1,)
        return o + ((0 if row["kind"] == "interest_taxes" else 1,) if w_term else ())
    book = _Book({"sav": CENT}, ["sav"], [], order, _community(i))
    book.walk(i["rows"])
    d = i["asked"]
    outs = [r for r in i["rows"] if r["on"] == d and r["from"] == "sav"]
    took = t(f"{tag} what each outflow of {d} took", _takes_of(book, outs, i["parties"]),
             "one batch: arcs sorted by the couple's part first, then the interest tax before the other outflow (rules 2-3)")
    parts = t(f"{tag} parts of sav at the end of {d}", _parts_of(book, "sav", d, i["parties"]), "")
    return _value({"took": took, "parts": parts})


def _o11_p04(i, t, tag="(P-04)"):
    return _o11_tax(i, t, tag, True)


def _o11_p19(i, t, tag="(P-19)"):
    try:
        return _o11_tax(i, t, tag, False)
    except PartsRefused as e:
        return _refused(e, t, f"{tag} the batch of {i['asked']}:")


def _o11_p21(i, t, tag="(P-21)"):
    out = {}
    s = i["signed"]
    if s["sign"] != "min 0":
        out["signed"] = _refused(PartsRefused("LOAD-PARTS", [f"kind {s['kind']} as min 0"], f"kind {s['kind']} keeps parts and is {s['sign']}"), t,
                                 f"{tag} the text whose owned kind is {s['sign']}:")
    try:
        out["below_zero"] = _o11_tax(i["below_zero"], t, tag, True)
    except PartsRefused as e:
        out["below_zero"] = _refused(e, t, f"{tag} the batch of {i['below_zero']['asked']}:")
    return out


def _o11_p05(i, t, tag="(P-05)"):
    book = _Book({"sav": CENT}, ["sav"], [], _couple_first(i), _community(i), prorata=True, take_rank=_rank(i["ties_rank"]))
    book.walk(i["rows"])
    d = i["asked"]
    outs = [r for r in i["rows"] if r["from"] == "sav"]
    took = t(f"{tag} what the withdrawal took", _takes_of(book, outs, i["parties"]),
             "the couple's part first; the rest shared by the tied parts in proportion to their amounts, rest largest to 0.01, the odd step by ties by (rules 2-3)")
    return _value({"took": took, "parts": t(f"{tag} parts of sav at the end of {d}", _parts_of(book, "sav", d, i["parties"]), "")})


def _o11_p06(i, t, tag="(P-06)"):
    day = {r["id"]: r["on"] for r in i["rows"]}
    book = _Book({"lots": D(i["unit_step"])}, [], ["lots"], lambda x, lab, row: (day[lab],), None)
    book.walk(i["rows"])
    buys = [r for r in i["rows"] if r["kind"] == "fund_buys"]
    gains = {}
    for s in [r for r in i["rows"] if r["kind"] == "fund_sales"]:
        lg = {l["name"]: l for l in _legs(s)}
        price = D(lg["cash"]["amount"]) / D(lg["units"]["amount"])
        took = book.took[(s["id"], "units")]
        for b in buys:
            bl = {l["name"]: l for l in _legs(b)}
            cost = D(bl["cash"]["amount"]) / D(bl["units"]["amount"])
            u = took.get(b["id"], Decimal(0))
            gains[f"{s['id']}, {b['id']}"] = t(f"{tag} gain of {s['id']} on batch {b['id']}", u * (price - cost),
                                               f"{u} units taken × ({price} sale price − {cost} cost a unit) (rules 2-3)")
    d = i["asked"]
    left = {b["id"]: book.bal("lots", d, b["id"]) for b in buys}
    t(f"{tag} units left per batch at the end of {d}", left, "first in, first out by the batch's day (rule 1)")
    return _value({"gain": gains, "total_gain": sum(gains.values(), Decimal(0)), "left": left})


def _o11_paid(i, paid_from):
    def order(x, lab, row):
        return (0 if row["kind"] == "withdrawals" and lab == paid_from(row) else 1, 0 if lab == i["couple"] else 1)
    return order


def _o11_p12(i, t, tag="(P-12)"):
    book = _Book({"sav": CENT}, ["sav"], [], _o11_paid(i, lambda r: r["paid_from"]), _community(i))
    book.walk(i["rows"])
    d = i["asked"]
    outs = [r for r in i["rows"] if r["from"] == "sav"]
    took = t(f"{tag} what each outflow took", _takes_of(book, outs, i["parties"]),
             "a withdrawal takes the party its row says paid first, the fee (no such field) the couple's part first; a shortfall goes on down the order (rules 2-3)")
    return _value({"took": took, "parts": t(f"{tag} parts of sav at the end of {d}", _parts_of(book, "sav", d, i["parties"]), ""),
                   "total": book.bal("sav", d)})


def _o11_p20(i, t, tag="(P-20)"):
    """The withdrawal's paid_from is a hole: its take is tried under every reading (each person); the parts it could
    reach under some reading merge into one hole part, the rest stay known, and the total stays known (L §5.14, §8.1)."""
    rows = i["rows"]
    w = next(r for r in rows if r["kind"] == "withdrawals")
    before = [r for r in rows if r["on"] < w["on"]]
    book = _Book({"sav": CENT}, ["sav"], [], _o11_paid(i, lambda r: r["paid_from"]), _community(i))
    book.walk(before)
    start = dict(book.parts["sav"])
    t(f"{tag} parts of sav before the withdrawal", start, "as P-12 (rules 1-3)")
    reach, outcomes = set(), {}
    for reading in i["hole_readings"]:
        ps = dict(start)
        got = _take(ps, D(w["amount"]), lambda lab, rd=reading: _o11_paid(i, lambda r: rd)("sav", lab, w), CENT)
        outcomes[reading] = got
        reach |= set(got)
    t(f"{tag} the take under each reading of paid_from", outcomes, "")
    decided = len({tuple(sorted(o.items())) for o in outcomes.values()}) == 1
    assert not decided
    merged = sum((start[l] for l in reach), Decimal(0)) - D(w["amount"])
    known = {l: a for l, a in start.items() if l not in reach}
    t(f"{tag} parts the take could reach (merged into one hole part)", sorted(reach), f"their amounts less the {w['amount']} taken = {merged}")
    total = sum(start.values(), Decimal(0)) - D(w["amount"])
    hole = {"kind": "refusal", "code": "HOLE-PART", "missing": [w["question"]],
            "reason": f"HOLE-PART: sav: {merged} may be {', '.join(sorted(reach))}'s"}
    parts = {p: (hole if p in reach else _value(known.get(p, Decimal(0)))) for p in i["parties"]}
    return {"total": _value(t(f"{tag} total of sav at the end of {i['asked']}", total, "the total stays known")), "parts": parts}


def _o11_p22(i, t, tag="(P-22)"):
    out = {}
    # (a) the observed rows: two transfers between two owned places in one batch, each arrival keeping labels
    h = i["hole"]
    book = _Book({"sav": CENT, "acc2": CENT}, ["sav", "acc2"], [], lambda x, lab, row: (0,), _community(h), prorata=True,
                 take_rank=_rank(h["ties_rank"]))
    cyc = [r for r in h["rows"] if r["on"] == h["cycle_day"]]
    book.walk([r for r in h["rows"] if r["on"] < h["cycle_day"]])
    start = {x: dict(p) for x, p in book.parts.items()}
    a, b = cyc
    tries = {}
    for lab in h["parties"]:
        ps = dict(start[a["from"]])
        ps[lab] = ps.get(lab, Decimal(0)) + D(b["amount"])
        tries[lab] = _take(ps, D(a["amount"]), lambda l: (0,), CENT, True, _rank(h["ties_rank"]))
    t(f"{tag}(a) {a['id']}'s take at {a['from']}, with {b['id']}'s arrival read as each party's", tries, "the outcome differs, so the take is undecided (L §5.14.2)")
    assert len({tuple(sorted(v.items())) for v in tries.values()}) > 1
    totals = {x: sum(start[x].values(), Decimal(0)) for x in start}
    for r in cyc:
        totals[r["from"]] -= D(r["amount"])
        totals[r["to"]] += D(r["amount"])
    mid = h["between"]
    # the hole part's question as L §8.1 words it for a cycle of observed rows (tally 2.1-F7)
    q = f"which of {', '.join(r['id'] for r in cyc)} went first on {h['cycle_day']}?"
    hole = {"kind": "refusal", "code": "HOLE-PART", "missing": [q], "reason": "HOLE-PART: a cycle of observed rows in one batch"}
    res = {mid: {x: {"total": _value(totals[x]), "parts": {p: hole for p in h["parties"]}} for x in ("sav", "acc2")}}
    rd = h["reading"]
    healed = {}
    for x in ("sav", "acc2"):
        st = rd[x]
        assert sum((D(v) for v in st["parts"].values()), Decimal(0)) == D(st["total"]) == totals[x]
        healed[x] = {"total": _value(D(st["total"])), "parts": {p: _value(D(st["parts"].get(p, "0"))) for p in h["parties"]}}
    t(f"{tag}(a) parts at the end of {rd['day']}", {x: st["parts"] for x, st in rd.items() if x != "day"}, "the reading's stated parts heal the hole (L §5.19)")
    res[rd["day"]] = healed
    out["hole"] = res
    # (b) the same with the returning transfer a decision: the day's order places it after the plan, so it computes exactly
    dcs = i["decision"]
    book = _Book({"sav": CENT, "acc2": CENT}, ["sav", "acc2"], [], lambda x, lab, row: (0,), _community(dcs), prorata=True,
                 take_rank=_rank(dcs["ties_rank"]))
    back = dcs["back"]

    def decide(bk, day):
        amt = min(D(back["amount"]), sum(bk.parts[back["from"]].values(), Decimal(0)))
        return {"id": back["id"], "on": day, "kind": "transfers", "legs": [{"name": "cash", "from": back["from"], "to": back["to"], "amount": amt}]}
    book.walk(dcs["rows"], {back["on"]: [decide]})
    d = dcs["asked"]
    out["decision"] = _value({x: t(f"{tag}(b) parts of {x} at the end of {d}", _parts_of(book, x, d, dcs["parties"]),
                                   "the plan's take first, then the decision's against the parts as they stand, in proportion (rules 1-3)")
                              for x in ("sav", "acc2")})
    return out


def OWN_11(i, t):
    """OWN-11: parts that leave, in the order an outflow takes them (L §3.6, §5.12, §5.13, §5.14)."""
    fns = {"p04": _o11_p04, "p05": _o11_p05, "p06": _o11_p06, "p12": _o11_p12, "p19": _o11_p19, "p20": _o11_p20,
           "p21": _o11_p21, "p22": _o11_p22}
    return {k: fns[k](i[k], t, f"({k[0].upper()}-{k[1:]})") for k in i}


def _o12_worth(i, t, tag):
    def shares(row, leg, book):
        if row["kind"] == "fund_buys":
            return {p: Fraction(int(p == row["buyer"])) for p in i["parties"]}
        return _community(i)(row, leg, book)
    book = _Book({"sav": CENT, "fund": D(i["unit_step"])}, ["sav", "fund"], [], _couple_first(i), shares)
    book.walk(i["rows"])
    d = i["asked"]
    plain = {x: D(v["opening"]) for x, v in i["plain"].items()}
    dropped = []
    for r in i["rows"]:
        for leg in _legs(r):
            if leg["from"] in plain:
                plain[leg["from"]] -= D(leg["amount"])
            if leg["to"] in plain:
                plain[leg["to"]] += D(leg["amount"])
                if leg["from"] in book.owned:
                    dropped.append((leg["to"], r["on"], r["id"]))
    t(f"{tag} the places counted by owns, at the end of {d}", plain, "opening plus the rows")
    px = D([v for day, v in sorted(i["px"].items()) if day <= d][-1])
    own = {}
    for o in i["parties"]:
        parts = book.bal("sav", d, o) + book.bal("fund", d, o) * px
        shared = sum((plain[x] * D(_fr(i["plain"][x]["owns"].get(o, "0"))) for x in plain), Decimal(0))
        own[o] = t(f"{tag} own worth of {o}", parts + shared, f"its parts in sav + its units × {px} + its owns shares of {sorted(plain)} (rule 1)")
    nw = {}
    for o in i["parties"]:
        through = sum((D(_fr(i["stake"].get(o, {}).get(e, "0"))) * own[e] for e in i["parties"]), Decimal(0))
        nw[o] = t(f"{tag} net worth of {o}", half_even(own[o] + through), f"own worth + Σ stake × the entity's own worth, half-even to 0.01 (rule 2)")
    return {"own_worth": own, "net_worth": nw}, dropped


def _o12_p10(i, t, tag="(P-10)"):
    v, dropped = _o12_worth(i, t, tag)
    assert not dropped
    return _value(v)


def _o12_p23(i, t, tag="(P-23)"):
    v, dropped = _o12_worth(i, t, tag)
    x, day, rid = dropped[0]
    w = t(f"{tag} the warning", f"WARN-LABELS: labels dropped at {x} from {day}: {rid}",
          f"{rid} took labelled parts to {x}, a place of the model that keeps none and isn't outside (rule 3)")
    return _value(v, warning=w)


def _o12_p17(i, t, tag="(P-17)"):
    out = {}
    for m, mi in i["members"].items():
        book = _Book({mi["place"]: CENT}, [mi["place"]], [], lambda x, lab, row: (0,), _community(mi))
        book.walk(mi["rows"])
        out[m] = {p: book.bal(mi["place"], i["asked"], p) for p in mi["parties"]}
        t(f"{tag} member {m}: parts of {mi['place']} at the end of {i['asked']}", out[m], "the member alone (its own rules)")
    same = i["same"]  # {joint name: {member: its party}}
    joint = {}
    for name, sides in same.items():
        joint[name] = {m: out[m].get(q, Decimal(0)) for m, q in sides.items()}
        joint[name]["total"] = sum(joint[name].values(), Decimal(0))
        t(f"{tag} {name} across the members", joint[name], "of p reads the parts labelled with either side of the same line, in every member (L §6.1)")
    return _value({"members": out, "same": joint})


def OWN_12(i, t):
    """OWN-12: reading parts, each party's figure (L §3.6, §5.10, §5.14, §6.1, §8.2)."""
    fns = {"p10": _o12_p10, "p17": _o12_p17, "p23": _o12_p23}
    return {k: fns[k](i[k], t, f"({k[0].upper()}-{k[1:]})") for k in i}
