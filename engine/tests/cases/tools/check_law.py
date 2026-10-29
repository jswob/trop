"""Checks the home law, the gazette and the market files (brief §9, step 4: every rule numbered and versioned).

- acts.md: every rule has versions v1..vn, each with in force / published / governs; spans in order and
  touching; only the last may run "until replaced"; every rule a text cites exists;
- the gazette: every id a rule names in backticks has rows, and every row's id is named; dates parse;
  published on or before in force; spans of one id never overlap (a same-span row is a replacement);
  every row's rule exists;
- market/: every file parses in its own dialect, dates rise and are business days, values are positive.
Run: python3 -I tmp/cases/tools/check_law.py   (exit 1 on any failure)
"""
import csv
import os
import re
import sys
from datetime import date, timedelta
from decimal import Decimal, InvalidOperation

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gen_gazette import CAL  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ACTS = os.path.join(ROOT, "law", "acts.md")
GAZ = os.path.join(ROOT, "law", "gazette.csv")
MKT = os.path.join(ROOT, "market")
CODES = "GEN|PIT|SOC|FAM|WCP|CIV|IGT|TT|VAT|TSB|CC|MTG|MOT"
fails = []


def fail(msg):
    fails.append(msg)


def iso(s):
    return date.fromisoformat(s)


# --- acts.md
text = open(ACTS, encoding="utf-8").read()
rules, cur = {}, None
VLINE = re.compile(r"^\*\*v(\d+)\*\* · in force (\d{4}-\d\d-\d\d) – (\d{4}-\d\d-\d\d|until replaced) · published (\d{4}-\d\d-\d\d) · governs: .+$")
for line in text.splitlines():
    m = re.match(rf"^### (({CODES})-\d+) · \S", line)
    if m:
        cur = m.group(1)
        if cur in rules:
            fail(f"{cur}: numbered twice")
        rules[cur] = []
        continue
    if line.startswith("**v"):
        v = VLINE.match(line)
        if not v or cur is None:
            fail(f"bad version line: {line[:60]}")
            continue
        rules[cur].append((int(v.group(1)), iso(v.group(2)), None if v.group(3) == "until replaced" else iso(v.group(3)), iso(v.group(4))))

by_act = {}
for rid, vs in rules.items():
    act, n = rid.split("-")
    by_act.setdefault(act, []).append(int(n))
    if not vs:
        fail(f"{rid}: no version")
    for i, (n_, f, t, p) in enumerate(vs):
        if n_ != i + 1:
            fail(f"{rid}: versions not v1..vn")
        if t is not None and t < f:
            fail(f"{rid} v{n_}: ends before it starts")
        if p > f:
            fail(f"{rid} v{n_}: published after it came into force")
        if i < len(vs) - 1:
            if t is None:
                fail(f"{rid} v{n_}: 'until replaced' but not the last version")
            elif t + timedelta(days=1) != vs[i + 1][1]:
                fail(f"{rid} v{n_}: span does not touch the next version's")
for act, ns in by_act.items():
    if sorted(ns) != list(range(1, len(ns) + 1)):
        fail(f"{act}: rules not numbered 1..n: {sorted(ns)}")

for ref in set(re.findall(rf"\b(({CODES})-\d+)", text)):
    if ref[0] not in rules:
        fail(f"cited but not a rule: {ref[0]}")

# --- the gazette
named = set(re.findall(r"`([A-Z][A-Z0-9]*(?:-(?:[A-Z0-9]+|<[a-z-]+>))+)`", text))
named = {n for n in named if not re.fullmatch(rf"({CODES})-\d+", n)}
pats = {n: re.compile("^" + re.sub(r"<[a-z]+-[a-z]+>|<[a-z]+>", lambda m: {"<yyyy>": r"\d{4}", "<yymm>": r"\d{4}", "<yyyy-mm>": r"\d{4}-\d\d", "<age>": r"\d+", "<k>": r"\d{4}"}[m.group(0)], n) + "$") for n in named}
rows = list(csv.DictReader(open(GAZ, encoding="utf-8")))
hit = {n: 0 for n in named}
spans = {}
for i, r in enumerate(rows, start=2):
    ms = [n for n, p in pats.items() if p.match(r["id"])]
    if not ms:
        fail(f"gazette line {i}: id {r['id']} named by no rule")
    for n in ms:
        hit[n] += 1
    try:
        p, f = iso(r["published"]), iso(r["from"])
        t = iso(r["to"]) if r["to"] else None
    except ValueError:
        fail(f"gazette line {i}: bad date")
        continue
    if p > f:
        fail(f"gazette line {i}: published after in force")
    if t is not None and t < f:
        fail(f"gazette line {i}: ends before it starts")
    if r["unit"] == "date":
        try:
            iso(r["value"])
        except ValueError:
            fail(f"gazette line {i}: value not a date")
    else:
        try:
            Decimal(r["value"])
        except InvalidOperation:
            fail(f"gazette line {i}: value not a number")
    if r["rule"] not in rules:
        fail(f"gazette line {i}: rule {r['rule']} not in the acts")
    spans.setdefault(r["id"], []).append((f, t, p))
for n, k in hit.items():
    if k == 0:
        fail(f"named in the acts, not in the gazette: {n}")
for id_, ss in spans.items():
    ss.sort()
    for (f0, t0, p0), (f1, t1, p1) in zip(ss, ss[1:]):
        if (f0 == f1 and t0 == t1) or (t0 is None and t1 is None and f0 == p0 and f1 == p1):
            if p1 <= p0:
                fail(f"{id_}: replacement not published later")
        elif t0 is None or t0 >= f1:
            fail(f"{id_}: spans overlap at {f1}")

# --- market files
DIALECTS = {  # file: (header lines, separator, date format, decimal comma)
    "brz": (1, ",", "%Y-%m-%d", False), "wek": (2, ";", "%d.%m.%Y", True),
}


def parse_date(s, fmt):
    from datetime import datetime
    return datetime.strptime(s, fmt).date()


def check_series(path, heads, sep, dfmt, comma, ncols):
    last = None
    with open(path, encoding="utf-8") as fh:
        lines = fh.read().splitlines()
    for j, line in enumerate(lines[heads:], start=heads + 1):
        parts = line.split(sep)
        if len(parts) != ncols:
            fail(f"{os.path.basename(path)} line {j}: {len(parts)} fields")
            continue
        d = parse_date(parts[0], dfmt)
        if last is not None and d <= last:
            fail(f"{os.path.basename(path)} line {j}: dates not rising")
        if not CAL.is_business(d):
            fail(f"{os.path.basename(path)} line {j}: {d} is not a business day")
        for x in parts[1:]:
            v = Decimal(x.replace(",", ".") if comma else x)
            if v <= 0:
                fail(f"{os.path.basename(path)} line {j}: value not positive")
        last = d
    return len(lines) - heads


funds = list(csv.DictReader(open(os.path.join(MKT, "funds.csv"), encoding="utf-8")))
counts = {}
for fd in funds:
    h, sep, dfmt, comma = DIALECTS[fd["house"].lower()]
    counts[fd["file"]] = check_series(os.path.join(MKT, fd["file"]), h, sep, dfmt, comma, 2)
counts["dukor.csv"] = check_series(os.path.join(MKT, "dukor.csv"), 1, ",", "%Y-%m-%d", False, 3)
counts["lazur-fx-flr.csv"] = check_series(os.path.join(MKT, "lazur-fx-flr.csv"), 2, ";", "%d.%m.%Y", True, 3)
fx_days = sorted(iso(r["from"]) for r in rows if r["id"] == "FX-FLR")
lz = [parse_date(l.split(";")[0], "%d.%m.%Y") for l in open(os.path.join(MKT, "lazur-fx-flr.csv"), encoding="utf-8").read().splitlines()[2:]]
if lz != fx_days:
    fail("lazur-fx-flr.csv: days differ from the Central Bank's FX-FLR days")
for name, (h, sep, dfmt, comma) in (("brz-distributions.csv", DIALECTS["brz"]), ("wek-distributions.csv", DIALECTS["wek"])):
    for j, line in enumerate(open(os.path.join(MKT, name), encoding="utf-8").read().splitlines()[h:], start=h + 1):
        code, ex, pay, amt = line.split(sep)
        if code not in {fd["code"] for fd in funds}:
            fail(f"{name} line {j}: unknown fund {code}")
        if not (CAL.is_business(parse_date(ex, dfmt)) and parse_date(pay, dfmt) > parse_date(ex, dfmt)):
            fail(f"{name} line {j}: bad dates")
        counts[name] = counts.get(name, 0) + 1
expected = set(counts) | {"funds.csv"}
for f in os.listdir(MKT):
    if f not in expected:
        fail(f"market/{f}: not checked by any dialect")

nv = sum(len(v) for v in rules.values())
print(f"acts: {len(rules)} rules in {len(by_act)} acts, {nv} versions; gazette: {len(rows)} rows, {len(spans)} ids, {len(named)} names in the acts")
print("market: " + ", ".join(f"{k} {v}" for k, v in sorted(counts.items())))
if fails:
    print(f"FAIL ({len(fails)}):")
    for f in fails[:40]:
        print(" -", f)
    sys.exit(1)
print("PASS")
