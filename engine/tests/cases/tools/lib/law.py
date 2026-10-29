"""Law arithmetic: scales, banded tables, tapers, cumulative advances, carry-forwards, the gazette, exception chains."""
from decimal import Decimal
from fractions import Fraction

from .dates import P
from .num import D, F, Refuse, half_up


def progressive(base, bands):
    """Tax on `base` by bands [[upper, rate], ..., [None, rate]]; each upper is the band's last amount, inclusive."""
    base, tax, lower = D(base), Decimal(0), Decimal(0)
    for upper, rate in bands:
        top = base if upper is None else min(base, D(upper))
        if top > lower:
            tax += (top - lower) * D(rate)
        if upper is None or base <= D(upper):
            break
        lower = D(upper)
    return tax


def table(x, rows):
    """The value of the row whose band holds x; rows: {lo, lo_incl, hi, hi_incl, value} (lo/hi null = open)."""
    x = D(x)
    for r in rows:
        lo_ok = r["lo"] is None or x > D(r["lo"]) or (r["lo_incl"] and x == D(r["lo"]))
        hi_ok = r["hi"] is None or x < D(r["hi"]) or (r["hi_incl"] and x == D(r["hi"]))
        if lo_ok and hi_ok:
            return D(r["value"])
    raise Refuse([f"band for {x}"], "no row of the table holds the amount")


def taper(x, full, limit, per_unit=1):
    """`full`, cut by `per_unit` for each unit of x above `limit`, never below 0."""
    return max(Decimal(0), D(full) - max(Decimal(0), D(x) - D(limit)) * D(per_unit))


def cumulative_advances(bases, bands, step):
    """Each period's advance: the tax on the base to date (rounded to `step`) minus what was already withheld."""
    cum, withheld, out = Decimal(0), Decimal(0), []
    for b in bases:
        cum += D(b)
        due = half_up(progressive(cum, bands), step) - withheld
        out.append(due)
        withheld += due
    return out


def carry_forward(loss, gains, cap_fraction="1/2", years=5):
    """A loss used against later years' gains: at most cap_fraction of the original loss a year, for `years` years."""
    cap = D(F(D(loss)) * F(cap_fraction))
    left, used, taxable = D(loss), [], []
    for i, g in enumerate(gains):
        g = D(g)
        u = min(cap, g, left) if (g > 0 and i < years) else Decimal(0)
        left -= u
        used.append(u)
        taxable.append(g - u)
    return used, taxable, left


def gazette(entries, on, known):
    """Read a published figure on a date, as known on `known` (required: no read past the known-as-of date).

    The entry in force on `on` with the latest start (and, at equal starts, the latest publication)
    wins; nothing in force is refused; a date past the last entry's end reads that entry, held.
    """
    on = P(on)
    cands = [e for e in entries if P(e["from"]) <= on and P(e["published"]) <= P(known)]
    if not cands:
        fid = entries[0]["id"] if entries else "figure"
        raise Refuse([f"{fid} on {on.isoformat()}"], "no gazette entry in force on the date")
    e = max(cands, key=lambda e: (P(e["from"]), P(e["published"])))
    held = e.get("to") is not None and on > P(e["to"])
    return D(e["value"]), e, held


def chain(flags):
    """A rule and its exceptions, outermost first: [grants, removes, restores, ...]."""
    if len(flags) == 1:
        return bool(flags[0])
    return bool(flags[0]) and not chain(flags[1:])
