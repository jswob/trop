#!/usr/bin/env python3
"""The union of all four households (E1 and E2) reproduces every household's keys, C's included.

Runs the visible union's checks (tools/union_e1.py: the hooks, D's derivation, the visible documents and
records), then C's side of the joins: C takes Marta's payout from the one statement in fam/e.py, and its
document and records agree. Then it checks that E1's keys are the same under the visible law alone and
under the visible law with the hidden supplement (law/home-supplement.md): every rule E1's keys read,
directly or through the household keys they take as given, is listed, and where the supplement touches
one, the term it adds is shown to be zero for the people E1 counts. Last, it regenerates every
household (A, B, D, C) and both parts of E from the union and shows every file is byte-identical.

Run: python3 -I engine/tests/cases/tools/union_e2.py [--no-regen]
"""
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import c_facts as F  # noqa: E402  (puts the visible tools on the path)
import c_sim as S  # noqa: E402
import union_e1 as U  # noqa: E402
from fam import b as B  # noqa: E402
from fam.home import ALL_KNOWN  # noqa: E402
from fam import e as E  # noqa: E402

HOLD = HERE.parent
VIS = HOLD  # cases 1.2.0: one tree, the holdout folded into the case set
FAILS = U.FAILS
RULE = re.compile(r"\b(GEN|PIT|SOC|FAM|WCP|CIV|IGT|TT|VAT|TSB|CC|MTG|MOT)-(\d+)\b")
KEYREF = re.compile(r"keys/([A-D])/([A-D]-\d\d)")
# The rules each household key E1 takes as given rests on, beyond those its derivation cites: the household
# scripts compute some parts (taxes owed, pensions, the joint return) in their engines, citing only the result.
DEPENDS = {
    "A-11": "PIT-2 PIT-3 PIT-4 PIT-5 PIT-6 PIT-7 SOC-1 SOC-2 WCP-1 WCP-2 CIV-1 PIT-8 PIT-9",
    "B-11": "CIV-4 PIT-2 PIT-3 PIT-4 PIT-5 PIT-10 PIT-15 SOC-1 SOC-2 SOC-3 SOC-4 VAT-1 VAT-3 VAT-4 VAT-5 GEN-4 MTG-1 MTG-2",
    "D-08": "SOC-2 SOC-5 SOC-6 SOC-7 PIT-3 PIT-5 PIT-13 TSB-3 TSB-4 TSB-5 TSB-7 CIV-4",
    "D-01": "CIV-1 CIV-2 CIV-3 CIV-5 TSB-3 TSB-6 TSB-7 PIT-5",
    "D-02": "IGT-1 IGT-2 IGT-4 GEN-2 GEN-3",
    "A-06": "IGT-1 IGT-2 IGT-3 IGT-4 GEN-3 GEN-5",
}


def check(cond, what):
    if not cond:
        FAILS.append(what)


def c_side():
    check(S.INHERIT[0] is E.PAYOUT_DAY and S.INHERIT[1] is E.PAYOUTS["Marta"]["amount"], "C takes Marta's payout from E")
    rows = (HOLD / "C" / "records" / "jodla-marta.csv").read_text(encoding="utf-8")
    pd = E.PAYOUT_DAY
    amt = f"{E.PAYOUTS['Marta']['amount']:.2f}".replace(".", ",")
    check(any(ln.startswith(pd.strftime("%d.%m.%Y")) and "Brzoza" in ln and f";{amt};" in ln for ln in rows.splitlines()),
          "C's records show Marta's payout arriving")
    c30 = (HOLD / "C" / "docs" / "C-D30.md").read_text(encoding="utf-8")
    check(U.sp(E.PAYOUTS["Marta"]["amount"]) in c30 and F.ACC_MARTA in c30 and "3/16" in c30, "C-D30 agrees with the union")


# --------------------------------------------------------------------------- E1 under both laws

def supplement_rules():
    """The rules the supplement replaces a point of, adds a point to, or adds."""
    s = (HOLD / "law" / "home-supplement.md").read_text(encoding="utf-8")
    pts = set(re.findall(r"\*\*([A-Z]+-\d+) v\d", s)) | set(re.findall(r"^### ([A-Z]+-\d+) ·", s, re.M))
    return pts


def rules_read(path, seen=None):
    """Rule ids a key's derivation cites, and those of every household key it takes as given."""
    seen = seen if seen is not None else set()
    if path in seen:
        return set()
    seen.add(path)
    text = path.read_text(encoding="utf-8")
    out = {f"{a}-{n}" for a, n in RULE.findall(text)} | set(DEPENDS.get(path.stem, "").split())
    for h, q in KEYREF.findall(text):
        out |= rules_read(VIS / "keys" / h / f"{q}.md", seen)
    return out


def scan(pattern, trees):
    rx = re.compile(pattern, re.I)
    hits = []
    for t in trees:
        for p in sorted(t.rglob("*")):
            if p.is_file() and rx.search(p.read_text(encoding="utf-8", errors="replace")):
                hits.append(str(p.relative_to(VIS)))
    return hits


def neutral():
    """For each rule the supplement touches: why its added term is zero for E1's people, checked on their papers."""
    trees = [VIS / "A", VIS / "B", VIS / "D", VIS / "E1"]
    abroad = scan(r"rosvald|(income|pay|work|tax) (from |paid |withheld )?abroad|foreign tax", trees)
    closing = scan(r"clos(es|ed|ing) (the|his|her) business|business (is )?closed|deregist", trees)
    release = scan(r"releas|forgiv|waive", trees)
    charity = scan(r"fundacj|foundation|public-benefit", trees)
    benefit = scan(r"without entitlement|repay the benefit|benefit .* repaid", trees)
    # R1: the union check reads the world, so it sees every rate
    flr_clients = {r["client"] for r in B.sales(known=ALL_KNOWN)[0] if r.get("cur") == "FLR"}
    check(bool(flr_clients), "B's florens invoices found")
    no_abroad = not abroad and all("Lenia" in c for c in flr_clients)
    why = {
        "PIT-5": (no_abroad, "the supplement's .3 adds only the foreign tax credit: no one in A, B or D has income from abroad"),
        "PIT-6": (no_abroad, "the supplement's .4 adds only foreign tax credits: none in A's joint returns"),
        "PIT-16": (no_abroad, "foreign income: none in A, B or D (B's florens come from a client in Lenia)"),
        "PIT-18": (no_abroad, "pay from work abroad: none in A, B or D"),
        "PIT-17": (not charity, "gifts to public-benefit bodies: no such gift in A, B or D"),
        "SOC-3": (not closing, "the supplement's .5 adds the closing month: no business in B closes"),
        "PIT-10": (not closing, "the supplement's .7 is for a business closed during a year: none"),
        "VAT-4": (not closing, "the supplement's .4 is for closing a business: none"),
        "IGT-2": (not release, "the supplement's .4 is a released debt: no debt in A, B or D is released"),
        "FAM-1": (not abroad, "the supplement's .6 is another state's family benefit: none in A, B or D"),
        "FAM-2": (not benefit, "benefits paid without entitlement: none in A, B or D"),
    }
    return why, {"abroad": abroad, "closing": closing, "release": release, "charity": charity, "benefit": benefit}


def both_laws():
    sup = supplement_rules()
    why, hits = neutral()
    report = []
    for p in sorted((VIS / "keys" / "E1").glob("E1-*.md")):
        read = rules_read(p)
        touched = sorted(read & sup)
        for r in touched:
            ok, reason = why.get(r, (False, "no reason given"))
            check(ok, f"{p.stem} reads {r}, which the supplement touches: {reason} — not shown ({hits})")
        report.append(f"{p.stem}: reads {len(read)} rules; the supplement touches {', '.join(touched) or 'none'}")
    return report


def regen(scripts, roots):
    """union_e1.regen with its hashes keyed by paths under the repository (session 5r, a path fix only: since the
    visible set moved to engine/tests/cases, union_e1's keys relative to its ROOT.parent no longer reach the holdout)."""
    import hashlib
    import subprocess
    top = HOLD.parents[2]

    def tree_hash():
        return {str(q.relative_to(top)): hashlib.sha256(q.read_bytes()).hexdigest()
                for r in roots for q in sorted(r.rglob("*")) if q.is_file()}
    before = tree_hash()
    for s in scripts:
        r = subprocess.run([sys.executable, "-I", "-B", str(s)], capture_output=True, text=True)
        if r.returncode:
            FAILS.append(f"{s.name} failed: {r.stdout[-300:]} {r.stderr[-300:]}")
    after = tree_hash()
    return len(after), sorted(k for k in set(before) | set(after) if before.get(k) != after.get(k))


def main(argv):
    U.checks()
    c_side()
    for line in both_laws():
        print(line)
    if "--no-regen" not in argv:
        vt, ht = VIS / "tools", HERE
        scripts = [vt / s for s in ("gen_a.py", "keys_a.py", "gen_b.py", "keys_b.py", "gen_d.py", "keys_d.py",
                                    "keys_e1.py")] + [ht / s for s in ("gen_c.py", "keys_c.py", "keys_e2.py")]
        roots = [VIS / "A", VIS / "B", VIS / "D", VIS / "keys" / "A", VIS / "keys" / "B", VIS / "keys" / "D",
                 VIS / "keys" / "E1", HOLD / "C", HOLD / "keys" / "C", HOLD / "keys" / "E2"]
        n, changed = regen(scripts, roots)
        check(not changed, f"regenerated from the union, {len(changed)} files changed: {changed[:5]}")
        print(f"regenerated from the union: {n} files of A, B, D, C, E1 and E2, {len(changed)} changed")
    if FAILS:
        print("union failed:", FAILS)
        sys.exit(1)
    print("union E2: C agrees with the union; E1's keys are the same under both laws" +
          ("" if "--no-regen" in argv else "; every household reproduced"))


if __name__ == "__main__":
    main(sys.argv[1:])
