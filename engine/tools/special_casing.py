"""special-casing: T's answers, searched for in the engine's code (R-94). A report: the
review reads every hit.

    engine/tools/special-casing

It reads the suite T inherits, as engine/tests/README.md describes it:
  - the ids: every check id in checks.tsv, the case ids inside them (`I:M:CB-01` gives
    CB-01), and the case ids that name files under tests/cases/ (`A-01`, `ASK-02`);
  - the asks: in each .tally input that checks.tsv names, every `ask <name>: <text>`
    line: its text (whitespace folded, 12 characters or more) and its name (6 or more,
    with a `_` or a digit, so not a plain word);
  - the expected values: every number in the stored outputs under tests/baselines/ with 4
    or more significant digits, dates left out. A value is compared by its digits alone
    (leading and trailing zeros and the point dropped), so `1 437.00 DUK` matches the
    literal 143_700, an amount in minor units.
Paths under engine/tests/ whose name holds `hidden` or `holdout` are skipped by name and
never opened or listed.

It searches the .rs files under engine/crates/: an id in a string literal or in the code
(comments aside), an ask's text or name in a string, a value as a number literal or as a
number inside a string. A hit means the engine may know the answer instead of computing it.

Exit codes: 0 the report (hits or none; a notice when engine/tests/ holds no suite yet);
1 engine/tests/ holds a suite (checks.tsv or cases/) but the ids, the asks or the values
came out empty, so the scan would be silently off; 64 usage.
"""

import os
import re
import sys

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
ROOT = os.path.dirname(ENGINE)
sys.path.insert(0, TOOLS)
import check_rules  # noqa: E402  the lexer and the file walk

TESTS = os.path.join(ENGINE, "tests")
CHECKS = os.path.join(TESTS, "checks.tsv")
CASES = os.path.join(TESTS, "cases")
BASELINES = os.path.join(TESTS, "baselines")
SEALED = re.compile(r"hidden|holdout", re.IGNORECASE)
CASE_ID = re.compile(r"(?<![A-Za-z0-9_-])([A-Z][A-Z0-9]*-[0-9]+[a-z]?)(?![A-Za-z0-9_])")
ASK = re.compile(r"^ask[ \t]+([A-Za-z_][\w]*)[ \t]*:[ \t]*(.*\S)", re.MULTILINE)
DATE = re.compile(r"\b\d{4}-\d{2}(?:-\d{2})?\b")
# a number in an output or a string: 1 437.00, -559.5, 2000; not inside a word
NUMBER = re.compile(r"(?<![\w.])[-+]?(?:\d{1,3}(?: \d{3})+|\d+)(?:\.\d+)?(?![\w])")
LITERAL = re.compile(r"\b[0-9][0-9_]*(?:\.[0-9][0-9_]*)?")
MIN_ASK, MIN_NAME, MIN_DIGITS = 12, 6, 4


def significant(text):
    """A number's digits with the sign, separators, point and leading and trailing zeros
    dropped: '1 437.00' and '143_700' both give '1437'."""
    digits = re.sub(r"[^0-9]", "", text.split("e")[0].split("E")[0])
    return digits.strip("0")


def sealed(path):
    return bool(SEALED.search(os.path.relpath(path, TESTS)))


def walk(top):
    """check_rules.walk, without entering or opening the sealed paths."""
    for d, dirs, names in os.walk(top):
        dirs[:] = sorted(x for x in dirs if x not in check_rules.SKIP_DIRS and not SEALED.search(x))
        for n in sorted(names):
            path = os.path.join(d, n)
            if not sealed(path):
                yield path


def read(path):
    with open(path, encoding="utf-8", errors="replace") as f:
        return f.read()


def gather():
    """(ids, asks, names, values) from T's suite; each a set."""
    ids, asks, names, values, inputs = set(), set(), set(), set(), set()
    if os.path.isfile(CHECKS):
        rows = [r.split("\t") for r in read(CHECKS).splitlines()]
        head = rows[0] if rows else []
        col = {c: head.index(c) for c in ("id", "files", "asks") if c in head}
        for row in rows[1:]:
            if "id" in col and len(row) > col["id"] and row[col["id"]]:
                ids.add(row[col["id"]])
                ids.update(CASE_ID.findall(row[col["id"]]))
            if "files" in col and len(row) > col["files"]:
                inputs.update(row[col["files"]].split())
            if "asks" in col and len(row) > col["asks"]:
                names.update(a for a in re.split(r"[ ,]+", row[col["asks"]]) if a)
    if os.path.isdir(CASES):
        for path in walk(CASES):
            stem = os.path.splitext(os.path.basename(path))[0]
            if CASE_ID.fullmatch(stem):
                ids.add(stem)
    for rel in sorted(inputs):
        path = os.path.join(ROOT, rel)
        if rel.endswith(".tally") and not SEALED.search(rel) and os.path.isfile(path):
            for name, text in ASK.findall(read(path)):
                names.add(name)
                folded = " ".join(text.split())
                if len(folded) >= MIN_ASK:
                    asks.add(folded.lower())
    names = {n for n in names if len(n) >= MIN_NAME and re.search(r"[_0-9]", n)}
    if os.path.isdir(BASELINES):
        for path in walk(BASELINES):
            for m in NUMBER.finditer(DATE.sub(" ", read(path))):
                d = significant(m.group())
                if len(d) >= MIN_DIGITS:
                    values.add(d)
    return ids, asks, names, values


def has_suite():
    return os.path.isfile(CHECKS) or (os.path.isdir(CASES) and any(True for _ in walk(CASES)))


def search(ids, asks, names, values):
    hits = set()
    id_pattern = None
    if ids:
        alternatives = "|".join(sorted(map(re.escape, ids), key=len, reverse=True))
        id_pattern = re.compile(r"(?<![A-Za-z0-9_-])(" + alternatives + r")(?![A-Za-z0-9_])")
    name_pattern = None
    if names:
        name_pattern = re.compile(r"(?<![\w])(" + "|".join(sorted(map(re.escape, names), key=len, reverse=True)) + r")(?![\w])")
    for path in check_rules.rust_files(os.path.join(ENGINE, "crates")):
        code, strings = check_rules.lex(read(path))
        r = check_rules.rel(path)
        for n, s in strings:
            if id_pattern:
                hits.update(f"{r}:{n}: case id {m.group(1)} in a string" for m in id_pattern.finditer(s))
            if name_pattern:
                hits.update(f"{r}:{n}: the ask name {m.group(1)} in a string" for m in name_pattern.finditer(s))
            folded = " ".join(s.split()).lower()
            hits.update(f"{r}:{n}: an ask text in a string: {a!r}" for a in asks if a in folded)
            for m in NUMBER.finditer(DATE.sub(" ", s)):
                d = significant(m.group())
                if d in values:
                    hits.add(f"{r}:{n}: the value {d} in a string, as {m.group()!r}")
        for n, line in enumerate(code.splitlines(), 1):
            if id_pattern:
                hits.update(f"{r}:{n}: case id {m.group(1)} in the code" for m in id_pattern.finditer(line))
            for lit in LITERAL.findall(line):
                d = significant(lit)
                if len(d) >= MIN_DIGITS and d in values:
                    hits.add(f"{r}:{n}: the value {d} as the literal {lit}")
    return hits


def main():
    if sys.argv[1:]:
        print("usage: engine/tools/special-casing", file=sys.stderr)
        return 64
    ids, asks, names, values = gather()
    counts = f"{len(ids)} ids, {len(asks)} ask texts, {len(names)} ask names, {len(values)} values of {MIN_DIGITS}+ significant digits"
    if not has_suite():
        print("special-casing: notice: engine/tests/ holds no suite yet (no checks.tsv, no cases/); nothing to search for")
        return 0
    empty = [what for what, got in (("ids", ids), ("ask texts", asks), ("values", values)) if not got]
    if empty:
        print(f"FAIL special-casing: engine/tests/ holds a suite but gave no {', no '.join(empty)} ({counts}): "
              "the scan would be silently off; see engine/tests/README.md for where they moved (R-94)")
        return 1
    print(f"special-casing: {counts}")
    hits = search(ids, asks, names, values)
    for h in sorted(hits):
        print(f"special-casing: {h} (R-94)")
    print(f"special-casing: {len(hits)} hits, for the review (R-94)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
