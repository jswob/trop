#!/usr/bin/env python3
"""The union of households A, B and D (E1) reproduces every household's keys.

Each family event is stated once, in `fam/e.py`: Halina's gifts to Anna, the estate's shares, the
payouts to Piotr, Anna, Marta and Jakub, and the accounts they went to. This script shows:
1. the hooks are live: A's, B's and D's facts modules hold E's very objects, not copies;
2. D's derivation of the estate gives exactly the payouts stated once, and D's records show the gift;
3. every household document and record that shows a join agrees with it;
4. regenerated from the union, every household's documents, records and keys are byte-identical
   to the files on disk (A, B, D and E1), so the union changes no household's answer.

Run: python3 -I tmp/cases/tools/union_e1.py [--no-regen]
"""
import hashlib
import pathlib
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from fam import a as A  # noqa: E402
from fam import b as B  # noqa: E402
from fam import d as DF  # noqa: E402
from fam import e as E  # noqa: E402
from fam.home import ALL_KNOWN, ZERO, D  # noqa: E402

TOOLS = pathlib.Path(__file__).resolve().parent
ROOT = TOOLS.parent
FAILS = []
MONTHS = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October",
          "November", "December"]


def check(cond, what):
    if not cond:
        FAILS.append(what)


def sp(x):
    return f"{D(x):,.2f}".replace(",", " ")


def hooks():
    """The households hold E's objects themselves."""
    check(A.GIFT_DAY is E.GIFT["day"] and A.GIFT_LOTS is E.GIFT["lots"], "A takes the gift from E")
    check(A.CASH_GIFT[0] is E.CASH_GIFT["day"] and A.CASH_GIFT[1] is E.CASH_GIFT["amount"], "A takes the cash gift from E")
    check(A.ESTATE_PAYOUT[0] is E.PAYOUT_DAY and A.ESTATE_PAYOUT[1] is E.PAYOUTS["Anna"]["amount"], "A takes its payout from E")
    check(B.ESTATE_PAYOUT[0] is E.PAYOUT_DAY and B.ESTATE_PAYOUT[1] is E.PAYOUTS["Piotr"]["amount"], "B takes its payout from E")
    check(DF.PAYOUT_DAY is E.PAYOUT_DAY, "D takes the payout day from E")
    check(DF.ANIA_ACCT is E.PAYOUTS["Anna"]["to"] and DF.PIOTR_ACCT is E.PAYOUTS["Piotr"]["to"]
          and DF.KUBA_OLD is E.PAYOUTS["Jakub"]["to"], "D takes the payout accounts from E")


def derived():
    """D derives the estate; its result is what E states once."""
    div = DF.division(known=ALL_KNOWN)   # R1: the union check reads the world, so it sees every figure
    for h in ("Piotr", "Anna", "Marta"):
        check(div["child"] == E.PAYOUTS[h]["amount"], f"D derives {h}'s payout")
    check(div["kuba"] == E.PAYOUTS["Jakub"]["amount"], "D derives Jakub's payout")
    check(div["escrow"] == sum((p["amount"] for p in E.PAYOUTS.values()), ZERO), "D's deposit is the payouts' sum")
    check(DF.KUBA_SHARE == E.SHARES["Jakub"] and DF.HALINA_SHARE == E.SHARES["Halina"] + 3 * E.SHARES["Piotr"],
          "D's flat shares after the division follow from the heirs' shares")
    check(DF.DIVISION == E.DIVISION_DAY, "D's division day")


def documents():
    """Every document and record that shows a join agrees with E."""
    def doc(h, i):
        return (ROOT / h / "docs" / f"{i}.md").read_text(encoding="utf-8")
    g = E.GIFT
    a18 = doc("A", "A-D18")
    for _, month, units in g["lots"]:
        y, m = month.split("-")
        check(f"| {MONTHS[int(m) - 1]} {y} | {units} |" in a18, f"A-D18 lists the lot of {month}")
    n = sum((u for _, _, u in g["lots"]), D(0))
    check(f"{n} units of Brzask Global Equity ({g['fund']})" in a18 and g["into"] in a18, "A-D18: the units and the account")
    kop = (ROOT / "A" / "records" / "kop-30-118-552-ania.csv").read_text(encoding="utf-8").splitlines()
    ins = [ln.split("\t") for ln in kop if "\tTRANSFER IN (gift)\t" in ln]
    check([(r[0], r[2], D(r[3])) for r in ins] == [(g["day"].isoformat(), g["fund"], u) for _, _, u in g["lots"]],
          "A's Kopia records show the lots arriving on the gift day")
    hk = (ROOT / "D" / "records" / "kopia-30-118-117-halina.tsv").read_text(encoding="utf-8")
    check(f"{g['day'].isoformat()}\tTRANSFER OUT (gift)\t{g['fund']}\t-{n}\tto {g['into']} (ANNA LIS)" in hk
          and g["from"] in hk, "D's Kopia record shows the units leaving")
    c = E.CASH_GIFT
    check(f"{c['day'].day} {MONTHS[c['day'].month - 1]} {c['day'].year}" in doc("A", "A-D19")
          and f"{int(c['amount']):,}".replace(",", " ") + " DUK" in doc("A", "A-D19"), "A-D19: the cash gift")
    d07 = doc("D", "D-D07")
    for h, p in E.PAYOUTS.items():
        if p["to"]:
            check(p["to"] in d07, f"D-D07 names {h}'s account")
    check("to Marta to her account" in d07, "D-D07 names no account of Marta's")
    pd = E.PAYOUT_DAY
    a36, b27 = doc("A", "A-D36"), doc("B", "B-D27")
    check(sp(E.PAYOUTS["Anna"]["amount"]) in a36 and f"{pd.day} {MONTHS[pd.month - 1]} {pd.year}" in a36, "A-D36")
    check(sp(E.PAYOUTS["Piotr"]["amount"]) in b27 and E.PAYOUTS["Piotr"]["to"] in b27 and pd.strftime("%d.%m.%Y") in b27,
          "B-D27")
    d01 = (ROOT / "keys" / "D" / "D-01.md").read_text(encoding="utf-8")
    for h, p in E.PAYOUTS.items():
        check(f'"{h}": {int(p["amount"])}' in d01, f"D-01's key pays {h} the stated payout")


def tree_hash(roots):
    out = {}
    for r in roots:
        for p in sorted(r.rglob("*")):
            if p.is_file():
                out[str(p.relative_to(ROOT.parent))] = hashlib.sha256(p.read_bytes()).hexdigest()
    return out


def regen(scripts, roots):
    """Run each script under -I; return the files whose bytes changed, appeared or vanished."""
    before = tree_hash(roots)
    for s in scripts:
        r = subprocess.run([sys.executable, "-I", "-B", str(s)], capture_output=True, text=True)
        if r.returncode:
            FAILS.append(f"{s.name} failed: {r.stdout[-300:]} {r.stderr[-300:]}")
    after = tree_hash(roots)
    return len(after), sorted(k for k in set(before) | set(after) if before.get(k) != after.get(k))


def checks():
    hooks()
    derived()
    documents()
    return FAILS


def main(argv):
    checks()
    if "--no-regen" not in argv:
        scripts = [TOOLS / s for s in ("gen_a.py", "keys_a.py", "gen_b.py", "keys_b.py", "gen_d.py", "keys_d.py",
                                       "keys_e1.py")]
        roots = [ROOT / "A", ROOT / "B", ROOT / "D", ROOT / "keys" / "A", ROOT / "keys" / "B", ROOT / "keys" / "D",
                 ROOT / "keys" / "E1"]
        n, changed = regen(scripts, roots)
        check(not changed, f"regenerated from the union, {len(changed)} files changed: {changed[:5]}")
        print(f"regenerated from the union: {n} files of A, B, D and E1, {len(changed)} changed")
    if FAILS:
        print("union failed:", FAILS)
        sys.exit(1)
    print("union E1: the hooks are live, D derives what is stated once, every document and record agrees")


if __name__ == "__main__":
    main(sys.argv[1:])
