"""Records: bank dialects, replay against a balance-after column, card cycles, matching rows to invoices."""
import csv
import io
from datetime import datetime
from decimal import Decimal

from .dates import P
from .num import D


def parse(text, sep=";", decimal=",", datefmt="%d.%m.%Y", header=1, amount_cols=(), date_cols=()):
    """Rows of one institution's CSV dialect as dicts; the last header line names the columns."""
    lines = text.strip("\n").split("\n")
    names = next(csv.reader([lines[header - 1]], delimiter=sep))
    rows = []
    for rec in csv.reader(io.StringIO("\n".join(lines[header:])), delimiter=sep):
        r = dict(zip(names, rec))
        for c in amount_cols:
            r[c] = D(r[c].replace(" ", "").replace(decimal, "."))
        for c in date_cols:
            r[c] = datetime.strptime(r[c], datefmt).date()
        rows.append(r)
    return rows


def replay(opening, rows, amount="amount", balance="balance"):
    """Walk rows against the bank's balance-after column.

    A row whose balance didn't move is booked twice in the export (no-effect); any other
    disagreement is a gap of the stated size (a missing row). Returns (closing, issues).
    """
    prev, issues = D(opening), []
    for i, r in enumerate(rows):
        exp = prev + r[amount]
        if r[balance] != exp:
            if r[balance] == prev and r[amount] != 0:
                issues.append({"row": i + 1, "kind": "no-effect", "amount": r[amount]})
            else:
                issues.append({"row": i + 1, "kind": "gap", "amount": r[balance] - exp})
        prev = r[balance]
    return prev, issues


def in_cycle(rows, start, end, date="date", amount="amount"):
    """Sum of rows dated within [start, end]."""
    s, e = P(start), P(end)
    return sum((r[amount] for r in rows if s <= P(r[date]) <= e), Decimal(0))


def canonical(name, registry):
    """A counterparty's one name: registry maps old or other names to the current one."""
    seen = set()
    while name in registry and name not in seen:
        seen.add(name)
        name = registry[name]
    return name
