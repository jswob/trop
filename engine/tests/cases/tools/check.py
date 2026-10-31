#!/usr/bin/env python3
"""The freeze checklist (brief §10) on the visible set; with --hidden <root>, on the holdout too.

Sections, each printed as ok or FAIL with what it found:
- coverage: check_edges.py; every visible edge has its probe card, every keyed probe its key, every pair card
  (probes/pairs/<EDGE>+<EDGE>.md) names two visible edges and has its key, and the probes agree with their
  hand-worked answers (keys_probes.py); every edge but a SURF passage is used by a question, except an edge whose
  `where:` says no visible household meets it and it is carried in integration by the 79 cases' models (named);
  every ★ edge by questions of two households;
- keys: one key per question, well formed (question, edges, as_of, known_as_of, key; a numbered
  derivation), every edge a known one, every rule, house rule, document and record file a key cites
  existing; the self-tests of compare.py pass;
- counts: each row of the counts tables (boundaries.md, and the holdout's) used by its households'
  questions; per household 3+ house rules, 2+ holes or refusals, a forecast question, a named hole in a
  net worth, 3+ keys moved by the twist (`without:`), 2/3 of the questions fused (a law rule and a
  document or record in one derivation, two steps or more); a loop refused; every rule of the law read by a key;
- records: every CSV parses in its dialect; running balances chain within a file and from one export
  to the next, except the breaks the writers' notes plant (listed in the holdout, so only --hidden
  names them; the visible run counts them);
- regeneration (--regen): union_e1.py, or with --hidden union_e2.py, regenerates every household from
  the union and finds every file byte-identical; the key scripts assert their relations as they run;
- the holdout sealed (--hidden): the term list the holdout keeps (tools/seal-terms.txt) is matched, whole
  words, against every file of the visible tree (.pyc included) and the memory notes; no case-set file
  is left in tmp/active/;
- the reader's disagreements settled (--hidden): every ✗ and ? line of reader/compare.txt is quoted
  in reader/settled.md, and reader/compare-after.txt exists;
- the papers: INDEX.md as tools/index.py writes it, README.md; with --hidden, INDEX-hidden.md and REPORT.md.

This script holds no name, id or rule number of the holdout: it reads them only from --hidden's root.

Run: python3 -I engine/tests/cases/tools/check.py [--regen]
cases 1.2.0: the holdout's households C and E2 are folded into this tree (keys/C, keys/E2, the tools beside the visible
ones, boundaries-hidden.md and INDEX-hidden.md here), so this tree is its own hold and every check covers them; the seal
and the reader's record stay with the exam record in tmp/cases-holdout/ (--hidden only applies to a tree without C).
Exit 0 when every section passes.
"""
import csv
import json
import pathlib
import re
import subprocess
import sys
from datetime import date, timedelta
from decimal import Decimal, InvalidOperation

TOOLS = pathlib.Path(__file__).resolve().parent
VIS = TOOLS.parent
REPO = VIS.parents[2]  # the case set lives at engine/tests/cases (4.5f)
MEMORY = pathlib.Path.home() / ".claude" / "projects" / str(REPO).replace("/", "-") / "memory"
sys.dont_write_bytecode = True
sys.path.insert(0, str(TOOLS))
import index as IX  # noqa: E402

VISIBLE_HOUSES = ("A", "B", "D", "E1")
QHEAD = re.compile(r"^## ([A-Z]\d?-\d\d) · (.*)$")
HRHEAD = re.compile(r"^## ([A-Z]-HR\d+) · ")
LAWHEAD = re.compile(r"^### ([A-Z]{1,4}-\d+) · ")
RESULTS = []


def report(section, ok, msg):
    RESULTS.append(ok)
    print(f"{'ok  ' if ok else 'FAIL'} {section}: {msg}")


def run(script, *args):
    r = subprocess.run([sys.executable, "-I", "-B", str(script), *map(str, args)], cwd=REPO, capture_output=True, text=True)
    return r.returncode, (r.stdout + r.stderr).strip()


def block(text):
    m = re.search(r"```yaml\n(.*?)\n```", text, re.S)
    out = {}
    for ln in (m.group(1).splitlines() if m else []):
        k, _, v = ln.partition(": ")
        try:
            out[k] = json.loads(v, parse_float=Decimal)
        except json.JSONDecodeError:
            out[k] = v
    return out


def house_dir(h, hold):
    return (VIS / h) if h in VISIBLE_HOUSES else (hold / h)


def keys_root(h, hold):
    return (VIS / "keys") if h in VISIBLE_HOUSES else (hold / "keys")


class Scope:
    def __init__(self, hold):
        self.hold = hold
        self.houses = list(VISIBLE_HOUSES) + (sorted(p.name for p in (hold / "keys").iterdir() if p.is_dir()
                                                     and p.name not in VISIBLE_HOUSES and (hold / p.name / "questions.md").exists())
                                              if hold else [])
        self.vis_edges = IX.edges(VIS / "boundaries.md")
        self.hid_edges = IX.edges(hold / "boundaries-hidden.md") if hold else []
        self.edge_ids = {e[2] for e in self.vis_edges + self.hid_edges}
        self.keys = {}      # qid -> (house, text, block)
        for h in self.houses:
            for p in sorted(keys_root(h, hold).joinpath(h).glob("*.md")):
                t = p.read_text(encoding="utf-8")
                self.keys[p.stem] = (h, t, block(t))
        self.use = {}
        for q, (h, _, b) in self.keys.items():
            for e in b.get("edges") or []:
                self.use.setdefault(e, []).append((h, q))
        laws = list(dict.fromkeys([VIS / "law" / "acts.md"] + (sorted((hold / "law").glob("*.md")) if hold else [])))
        self.rules = {}
        for p in laws:
            for ln in p.read_text(encoding="utf-8").splitlines():
                m = LAWHEAD.match(ln)
                if m:
                    self.rules[m.group(1)] = p.name
        self.prefixes = sorted({r.split("-")[0] for r in self.rules}, key=len, reverse=True)
        self.rule_re = re.compile(r"\b(" + "|".join(self.prefixes) + r")-(\d+)\b")
        self.files = set()
        for root in [VIS] + ([hold] if hold and hold != VIS else []):
            for p in root.rglob("*"):
                if p.is_file() and "__pycache__" not in p.parts:
                    self.files.add(p.name)


# --------------------------------------------------------------------------- coverage

CARRIED = re.compile(r"^- where: no visible household; carried in integration by the 79 cases' models \(((?:[A-Z]{2}-\d\d)(?:, [A-Z]{2}-\d\d)*)\)")


def carried_by_79(path):
    """{edge: its 79-case ids} for each edge whose `where:` says no visible household meets it and the 79 cases' models
    carry it in integration (cases 1.1: LAW-20, the named solver). Such an edge needs its probe, not a question."""
    out, cur = {}, None
    for ln in path.read_text(encoding="utf-8").splitlines():
        m = IX.EDGE_HEAD.match(ln)
        if m:
            cur = m.group(1)
            continue
        m = CARRIED.match(ln)
        if m and cur:
            out[cur] = m.group(1)
    return out


def pair_cards():
    """Pair cards (probes/pairs/<EDGE>+<EDGE>.md): ([(stem, its edges)], problems)."""
    from lib.keys import load_block
    cards, bad = [], []
    for p in sorted((VIS / "probes" / "pairs").glob("*.md")):
        parts = p.stem.split("+")
        y = load_block(p.read_text(encoding="utf-8")) or {}
        cards.append((p.stem, parts))
        if len(parts) != 2 or y.get("edge") != parts:
            bad.append(f"{p.name}: its edge line must name its two edges")
        if not (VIS / "keys" / "probes" / f"{p.stem}.md").exists():
            bad.append(f"{p.name}: no key")
    return cards, bad


def coverage(s):
    code, out = run(TOOLS / "check_edges.py", *(["--hidden", s.hold / "boundaries-hidden.md"] if s.hold else []))
    report("coverage", code == 0, "check_edges.py: " + out.splitlines()[-1] if out else "no output")
    missing_probe, missing_key = [], []
    for axis, _, eid, _, _ in s.vis_edges:
        card = VIS / "probes" / axis.lower() / f"{eid}.md"
        if not card.exists():
            missing_probe.append(eid)
        elif axis != "SURF" and not (VIS / "keys" / "probes" / f"{eid}.md").exists():
            missing_key.append(eid)
    pairs, pair_bad = pair_cards()
    vis_ids = {e[2] for e in s.vis_edges}
    pair_bad += [f"{st}: {e} is no visible edge" for st, es in pairs for e in es if e not in vis_ids]
    report("coverage", not missing_probe and not missing_key and not pair_bad,
           f"{len(s.vis_edges)} visible edges, each with its probe card; keyed probes without a key: {missing_key or 'none'}; without a card: {missing_probe or 'none'}"
           + (f"; pair cards {', '.join(st for st, _ in pairs)}, each naming two visible edges, with its key: {pair_bad or 'no problem'}" if pairs or pair_bad else ""))
    code, out = run(TOOLS / "keys_probes.py")
    m = re.search(r"(\d+) probes agree, (\d+) disagree", out)
    report("coverage", code == 0 and m is not None and m.group(2) == "0", f"keys_probes.py: {m.group(0) if m else out[-200:]}")
    carried = carried_by_79(VIS / "boundaries.md")
    unused = [e[2] for e in s.vis_edges if e[0] != "SURF" and e[2] not in s.use and e[2] not in carried]
    unused_h = [e[2] for e in s.hid_edges if e[2] not in s.use]
    report("coverage", not unused and not unused_h,
           f"every edge but a SURF passage used by a question ({len(s.use)} edges used); unused: {unused + unused_h or 'none'}"
           + (f"; carried in integration by the 79 cases' models instead: {', '.join(f'{e} ({ids})' for e, ids in carried.items())}" if carried else ""))
    weak = []
    for axis, _, eid, _, star in s.vis_edges + s.hid_edges:
        if star:
            hs = {h for h, _ in s.use.get(eid, []) if not h.startswith("E")}
            if len(hs) < 2:
                weak.append(f"{eid} ({', '.join(sorted(hs)) or 'none'})")
    nstar = sum(1 for e in s.vis_edges + s.hid_edges if e[4])
    if s.hold:
        report("coverage", not weak, f"{nstar} ★ edges, each used by questions of two households (under their different rules); short: {weak or 'none'}")
    else:
        report("coverage", True, f"{nstar} ★ edges; used by two visible households: {nstar - len(weak)}; by one so far: {weak or 'none'} "
                                 "(a ★ edge may take its second household from the holdout, so only the full run fails on it)")


# --------------------------------------------------------------------------- keys

def keys(s):
    bad = []
    for h in s.houses:
        qs = []
        for ln in (house_dir(h, s.hold) / "questions.md").read_text(encoding="utf-8").splitlines():
            m = QHEAD.match(ln)
            if m:
                qs.append(m.group(1))
        ks = sorted(q for q, v in s.keys.items() if v[0] == h)
        if sorted(qs) != ks:
            bad.append(f"{h}: questions {qs} vs keys {ks}")
    report("keys", not bad, f"one key per question in {', '.join(s.houses)} ({len(s.keys)} keys); {bad or 'no mismatch'}")
    bad = []
    for q, (h, t, b) in s.keys.items():
        for f in ("question", "edges", "as_of", "known_as_of", "key"):
            if f not in b:
                bad.append(f"{q}: no {f}")
        if b.get("question") != q:
            bad.append(f"{q}: names {b.get('question')}")
        for e in b.get("edges") or []:
            if e not in s.edge_ids:
                bad.append(f"{q}: unknown edge {e}")
        der = t.split("## Derivation", 1)[1] if "## Derivation" in t else ""
        if not re.search(r"^1\. ", der, re.M):
            bad.append(f"{q}: no numbered derivation")
    report("keys", not bad, f"every key well formed, with known edges and a numbered derivation; {bad[:8] or 'none wrong'}")
    bad, n = [], 0
    for q, (h, t, b) in s.keys.items():
        hdir = house_dir(h, s.hold)
        for m in s.rule_re.finditer(t):
            n += 1
            if f"{m.group(1)}-{m.group(2)}" not in s.rules:
                bad.append(f"{q}: rule {m.group(0)}")
        for m in re.finditer(r"\b([A-D])-HR(\d+)\b", t):
            n += 1
            hr = house_dir(m.group(1), s.hold) / "house-rules.md"
            if not hr.exists() or f"## {m.group(0)} · " not in hr.read_text(encoding="utf-8"):
                bad.append(f"{q}: house rule {m.group(0)}")
        for m in re.finditer(r"\b([A-D])-D(\d+[a-z]?)\b", t):
            n += 1
            d = house_dir(m.group(1), s.hold) / "docs"
            if not (d / f"{m.group(0)}.md").exists() and not list(d.glob(f"{m.group(0)}[a-z].md")) and not list(d.glob(f"{m.group(0)}-*.md")):
                bad.append(f"{q}: document {m.group(0)}")
        for m in re.finditer(r"\b([\w.-]+\.(?:csv|tsv|txt))\b", t):
            n += 1
            if m.group(1) not in s.files:
                bad.append(f"{q}: file {m.group(1)}")
        del hdir
    report("keys", not bad, f"{n} citations of rules, house rules, documents and record files, all found; {bad[:8] or 'none missing'}")
    fails = []
    for h in s.houses + ["P"]:  # P: the probe keys (cases 1.1.1)
        args = ["--self-test", h] + (["--keys-root", s.hold / "keys"] if h not in VISIBLE_HOUSES + ("P",) else [])
        code, out = run(TOOLS / "compare.py", *args)
        if code != 0:
            fails.append(f"{h}: {out[-200:]}")
    report("keys", not fails, f"compare.py's self-test passes for {', '.join(s.houses + ['P'])}; {fails or 'no failure'}")


# --------------------------------------------------------------------------- counts

def count_rows(path):
    rows, on = [], False
    for ln in path.read_text(encoding="utf-8").splitlines():
        if ln.startswith("## Counts"):
            on = True
            continue
        if on and ln.startswith("## "):
            break
        m = re.match(r"^\| ([LMQR]\d+) \| (.*?) \| (.*?) \| (.*?) \|$", ln) if on else None
        if m:
            rows.append((m.group(1), m.group(2), re.findall(r"[A-Z]{3,4}-\d\d", m.group(3)), m.group(4)))
    return rows


def counts(s):
    families = [h for h in s.houses if not h.startswith("E")]
    rows = count_rows(VIS / "boundaries.md") + (count_rows(s.hold / "boundaries-hidden.md") if s.hold else [])
    bad = []
    for tag, text, es, where in rows:
        users = {h for e in es for h, _ in s.use.get(e, [])}
        named = families if "all" in where else [h for h in re.findall(r"\b([A-D])\b", where) if h in s.houses]
        if not users:
            bad.append(f"{tag}: no question uses {es}")
        for h in named:
            if h not in users:
                bad.append(f"{tag}: {h} names it but no question of {h} uses {es}")
    report("counts", not bad, f"{len(rows)} count rows, each used by its households' questions; {bad or 'none short'}")
    bad, lines = [], []
    for h in families:
        ks = {q: v for q, v in s.keys.items() if v[0] == h}
        qtext = (house_dir(h, s.hold) / "questions.md").read_text(encoding="utf-8")
        nhr = len([ln for ln in (house_dir(h, s.hold) / "house-rules.md").read_text(encoding="utf-8").splitlines() if HRHEAD.match(ln)])
        holes = [q for q, (_, _, b) in ks.items() if re.search(r'"kind": "(refusal|partial)"', json.dumps(b.get("key"), default=str))]
        forecast = [q for q, (_, _, b) in ks.items() if any(re.match(r"(RAND|SCEN)-|RULE-09|LAW-13", e) for e in b.get("edges") or [])]
        worth = []
        for q in holes:
            sec = qtext.split(f"## {q} · ", 1)[1].split("\n## ", 1)[0]
            if "worth" in sec:
                worth.append(q)
        moved = [q for q, (_, _, b) in ks.items() if "without" in b and b["without"] != b["key"]]
        fused, onestep = 0, []
        for q, (_, t, _) in ks.items():
            der = t.split("## Derivation", 1)[1]
            law = bool(s.rule_re.search(der) or re.search(r"\b[A-D]-HR\d", der))
            fact = bool(re.search(r"\b[A-D]-D\d|\.(csv|tsv|txt)\b|\b(records?|story|export|statement|register|invoice|payslip|balance|"
                                  r"units|price|gazette|FX-FLR|CPI|DUKOR)\b", der, re.I))
            fused += law and fact
            if len(re.findall(r"^\d+\. ", der, re.M)) < 2:
                onestep.append(q)
        ok = nhr >= 3 and len(holes) >= 2 and forecast and worth and len(moved) >= 3 and 3 * fused >= 2 * len(ks) and not onestep
        if not ok:
            bad.append(h)
        lines.append(f"{h}: {nhr} house rules, {len(holes)} holes or refusals, {len(forecast)} forecast, net-worth hole in {worth or 'none'}, "
                     f"{len(moved)} moved by the twist, {fused}/{len(ks)} fused, one-step {onestep or 'none'}")
    report("counts", not bad, "per household: " + "; ".join(lines))
    loops = [q for q, (_, t, _) in s.keys.items() if re.search(r'"kind": "refusal"[^}]*"reason": "a loop', t)]
    report("counts", bool(loops), f"a loop refused: {loops or 'none'}")
    cited = set()
    texts = [t for _, t, _ in s.keys.values()] + [p.read_text(encoding="utf-8") for p in (VIS / "keys" / "probes").glob("*.md")]
    for t in texts:
        cited |= {f"{m.group(1)}-{m.group(2)}" for m in s.rule_re.finditer(t)}
    unread = [r for r in s.rules if r not in cited]
    report("counts", not unread, f"{len(s.rules)} rules of the law ({len(set(s.rules.values()))} files), each read by a key or a probe; unread: {unread or 'none'}")


# --------------------------------------------------------------------------- records

def num(x):
    x = x.strip().replace(" ", "").replace(" ", "")
    if not x:
        return None
    if "," in x and "." not in x:
        x = x.replace(",", ".")
    try:
        return Decimal(x)
    except InvalidOperation:
        return None


HEADWORD = re.compile(r"^\W*(date|data|dzień|booking|number|nr|account)\b", re.I)
SEPS = (";", ",", "\t", "|")


def parse(path):
    """(preamble lines, header, rows, postamble) in the file's own dialect. The header is the first line
    whose first field names a date, number or account column, split by the separator that gives it the
    most fields; a row longer than the header holds the separator in its last field (a note); a
    closing-balance line ends the rows."""
    lines = path.read_text(encoding="utf-8").splitlines()
    for i, ln in enumerate(lines):
        head, d = max(((next(csv.reader([ln], delimiter=sep)), sep) for sep in SEPS), key=lambda x: len(x[0]))
        if len(head) >= 3 and HEADWORD.match(head[0]):
            break
    else:
        raise ValueError(f"{path.name}: no header line")
    m, data, post = len(head), [], []
    for r in csv.reader(lines[i + 1:], delimiter=d):
        if not any(c.strip() for c in r):
            continue
        if post or re.match(r"(closing balance|saldo końcowe)", r[0], re.I):
            post.append(r)
            continue
        if len(r) < m:
            raise ValueError(f"{path.name}: a row of {len(r)} fields under a header of {m}: {r}")
        if len(r) > m:
            r[m - 1:] = [d.join(r[m - 1:])]
        data.append(r)
    return lines[:i], head, data, post


def chain(path, parsed):
    """Breaks in the running balance: ([(row number or 'closing', expected, shown)], opening, closing)."""
    pre, head, data, post = parsed
    low = [h.casefold() for h in head]
    bal = next((i for i, h in enumerate(low) if re.match(r"(balance|saldo)", h)), None)
    amt = next((i for i, h in enumerate(low) if re.match(r"(amount|kwota)", h)), None)
    deb = next((i for i, h in enumerate(low) if h == "debit"), None)
    cre = next((i for i, h in enumerate(low) if h == "credit"), None)
    cur = next((i for i, h in enumerate(low) if h in ("waluta", "currency")), None)
    if bal is None or (amt is None and (deb is None or cre is None)):
        return None
    opening = None
    for ln in pre:
        mo = re.search(r"(opening balance|saldo początkowe)\W*([-\d\s., ]+)$", ln, re.I)
        if mo:
            opening = num(mo.group(2))
    last, breaks = {}, []
    for i, r in enumerate(data):
        a = num(r[amt]) if amt is not None else (num(r[cre]) or 0) - (num(r[deb]) or 0)
        b, c = num(r[bal]), (r[cur] if cur is not None else "")
        if a is None or b is None:
            continue
        prev = last.get(c, opening if cur is None else None)
        if prev is not None and prev + a != b:
            breaks.append((i + 1, prev + a, b))
        last[c] = b
    closing = last.get("", opening)
    for r in post:
        shown = num(r[-1])
        if shown is not None and closing is not None and shown != closing:
            breaks.append(("closing", closing, shown))
    return breaks, opening, closing


def period(pre):
    """(first day, last day) of an export's period line, in any of the dialects' date forms, or None."""
    ln = next((ln for ln in pre if re.match(r"\W*(period|okres)\b", ln, re.I)), "")
    days = []
    for m in re.finditer(r"\b(\d{4})-?(\d\d)-?(\d\d)\b|\b(\d\d)[.-](\d\d)[.-](\d{4})\b", ln):
        y, mo, d = (m.group(1), m.group(2), m.group(3)) if m.group(1) else (m.group(6), m.group(5), m.group(4))
        days.append(date(int(y), int(mo), int(d)))
    return (days[0], days[-1]) if len(days) >= 2 else None


def records(s):
    roots = [VIS / h / "records" for h in ("A", "B", "D")] + ([s.hold / h / "records" for h in s.houses if h not in VISIBLE_HOUSES and (s.hold / h / "records").exists()] if s.hold else [])
    nfiles, nrows, found, errors = 0, 0, [], []
    accounts = {}
    for root in roots:
        for p in sorted(root.iterdir()):
            if p.suffix not in (".csv", ".tsv"):
                p.read_text(encoding="utf-8")
                nfiles += 1
                continue
            try:
                parsed = parse(p)
            except (ValueError, StopIteration) as e:
                errors.append(str(e))
                continue
            nfiles += 1
            nrows += len(parsed[2])
            c = chain(p, parsed)
            if c is None:
                continue
            breaks, opening, closing = c
            found += [f"{root.parent.name}/{p.name} row {i}" for i, _, _ in breaks]
            acct = next((ln for ln in parsed[0] if re.match(r"\W*(account|rachunek)\b", ln, re.I)), None)
            if acct and opening is not None:
                accounts.setdefault((root, acct), []).append((p.name, period(parsed[0]), opening, closing))
    for (root, _), files in accounts.items():
        for (n1, p1, _, c1), (n2, p2, o2, _) in zip(files, files[1:]):
            follows = p1 and p2 and p2[0] - p1[1] == timedelta(days=1)
            if follows and c1 is not None and c1 != o2:
                found.append(f"{root.parent.name}/{n2} opening")
    report("records", not errors, f"{nfiles} files, {nrows} rows, each parsed in its own dialect; {errors[:3] or 'no error'}")
    if not s.hold:
        report("records", True, f"running balances chain within each export and from each month's export to the next, except {len(found)} breaks; "
                                "the full run (--hidden) matches them against the planted ones")
        return
    planted = [ln.split("  #")[0].strip() for ln in (s.hold / "tools" / "planted-breaks.txt").read_text(encoding="utf-8").splitlines()
               if ln.strip() and not ln.startswith("#")]
    extra, short = sorted(set(found) - set(planted)), sorted(set(planted) - set(found))
    report("records", not extra and not short, f"running balances chain except the {len(planted)} planted breaks; unplanted: {extra or 'none'}; planted but not found: {short or 'none'}")


# --------------------------------------------------------------------------- regeneration, seal, reader, papers

def regen(s):
    script = (s.hold / "tools" / "union_e2.py") if s.hold else (TOOLS / "union_e1.py")
    code, out = run(script)
    m = re.search(r"regenerated from the union: (\d+) files.*?, (\d+) changed", out)
    report("regeneration", code == 0 and m is not None and m.group(2) == "0", f"{script.name}: {m.group(0) if m else out[-300:]}")


def seal(s):
    terms, allow = [], set()
    for ln in (s.hold / "tools" / "seal-terms.txt").read_text(encoding="utf-8").splitlines():
        if not ln.strip() or ln.startswith("#"):
            continue
        if ln.startswith("allow: "):
            allow.add(ln[7:].strip())
        elif ln.startswith("re: "):
            terms.append(re.compile(ln[4:].strip()))
        else:
            terms.append(re.compile(r"(?<!\w)" + re.escape(ln.strip()) + r"(?!\w)"))
    hits, n = [], 0
    for root in (VIS, MEMORY):
        for p in sorted(root.rglob("*")):
            if not p.is_file():
                continue
            n += 1
            text = p.read_bytes().decode("utf-8", errors="ignore")
            for t in terms:
                for m in t.finditer(text):
                    where = f"{p.relative_to(root.parent) if root == VIS else p.name}:{m.group(0)}"
                    if where not in allow:
                        hits.append(where)
    report("sealed", not hits, f"{len(terms)} terms, whole words, over {n} files of the visible tree and the memory notes; hits: {sorted(set(hits))[:10] or 'none'}")
    left = sorted(p.name for p in (REPO / "tmp" / "active").glob("case-set-*"))
    report("sealed", not left, f"case-set files left in tmp/active/: {left or 'none'}")


def reader(s):
    cmp_ = s.hold / "reader" / "compare.txt"
    settled = (s.hold / "reader" / "settled.md").read_text(encoding="utf-8") if (s.hold / "reader" / "settled.md").exists() else ""
    lines = [ln.strip() for ln in cmp_.read_text(encoding="utf-8").splitlines() if ln.strip().startswith(("✗", "?"))]
    open_ = [ln for ln in lines if ln not in settled]
    after = (s.hold / "reader" / "compare-after.txt").exists()
    report("reader", not open_ and after, f"{len(lines)} lines of compare.txt, each quoted in settled.md; unsettled: {len(open_)}; compare-after.txt {'there' if after else 'missing'}")


def papers(s):
    ok = (VIS / "INDEX.md").exists() and (VIS / "INDEX.md").read_text(encoding="utf-8") == IX.visible_index()
    report("papers", ok, "INDEX.md is what index.py writes" if ok else "INDEX.md missing or stale: run tools/index.py")
    rd = VIS / "README.md"
    report("papers", rd.exists() and rd.stat().st_size > 0, "README.md there" if rd.exists() else "README.md missing")
    if s.hold:
        hi = s.hold / "INDEX-hidden.md"
        ok = hi.exists() and hi.read_text(encoding="utf-8") == IX.hidden_index(s.hold)
        report("papers", ok, "INDEX-hidden.md is what index.py --hidden writes" if ok else "INDEX-hidden.md missing or stale")
        if s.hold != VIS:
            rp = s.hold / "REPORT.md"
            report("papers", rp.exists(), "REPORT.md there" if rp.exists() else "REPORT.md missing")


def main(argv):
    hold = pathlib.Path(argv[argv.index("--hidden") + 1]).resolve() if "--hidden" in argv else None
    if (VIS / "keys" / "C").is_dir():  # cases 1.2.0: the holdout's households live in this tree; it is its own hold
        hold = VIS
    s = Scope(hold)
    print(f"# check.py: {'the case set with the unsealed households (cases 1.2.0)' if hold == VIS else 'the visible set and the holdout' if hold else 'the visible set'}")
    coverage(s)
    keys(s)
    counts(s)
    records(s)
    if "--regen" in argv:
        regen(s)
    if hold and hold != VIS:  # the seal and the reader's record belong to the sealed exam (tmp/cases-holdout/)
        seal(s)
        reader(s)
    papers(s)
    print(f"# {sum(RESULTS)} of {len(RESULTS)} checks pass: {'PASS' if all(RESULTS) else 'FAIL'}")
    return 0 if all(RESULTS) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
