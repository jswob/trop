"""special-casing: T's answers, searched for in the engine's code (R-94). A report: it
never fails the build, and the review reads every hit.

    engine/tools/special-casing

From T's vectors (engine/tests/**/*.json) it takes the case ids (the file names under
tests/cases/), the ask texts (every string under a key named `ask`) and every expected
value with 4 or more significant digits. It searches the code under engine/crates/ for
them: an id or an ask text in a string literal or the code, a value's digits in a number
literal or a string. A hit means the engine may know the answer instead of computing it.

Until T's vectors exist it prints a notice. It always exits 0.
"""

import json
import os
import re
import sys

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
sys.path.insert(0, TOOLS)
import check_rules  # noqa: E402  the lexer and the file walk

TESTS = os.path.join(ENGINE, "tests")
CASES = os.path.join(TESTS, "cases")
NUMBER = re.compile(r"^[+-]?(?:[0-9][0-9_]*)?(?:\.[0-9_]*)?(?:[eE][+-]?[0-9]+)?$")
LITERAL = re.compile(r"\b[0-9][0-9_]*(?:\.[0-9][0-9_]*)?")
MIN_ASK = 12


def significant(text):
    """The significant digits of a decimal written as text, or '' if it isn't one."""
    t = text.strip()
    if not t or not NUMBER.match(t) or not any(c.isdigit() for c in t):
        return ""
    mantissa = re.split(r"[eE]", t)[0].lstrip("+-").replace("_", "")
    digits = mantissa.replace(".", "").lstrip("0")
    if "." not in mantissa:
        digits = digits.rstrip("0")
    return digits


def gather():
    ids, asks, values = set(), set(), set()
    if os.path.isdir(CASES):
        ids = {os.path.splitext(n)[0] for n in os.listdir(CASES) if n.endswith(".json")}

    def visit(node, key=None):
        if isinstance(node, dict):
            for k, v in node.items():
                visit(v, k)
        elif isinstance(node, list):
            for v in node:
                visit(v, key)
        elif isinstance(node, bool):
            return
        elif isinstance(node, (int, float)):
            d = significant(repr(node))
            if len(d) >= 4:
                values.add(d)
        elif isinstance(node, str):
            if key == "ask" and len(node) >= MIN_ASK:
                asks.add(node)
            d = significant(node)
            if len(d) >= 4:
                values.add(d)

    for path in check_rules.walk(TESTS):
        if path.endswith(".json"):
            try:
                with open(path, encoding="utf-8") as f:
                    visit(json.load(f))
            except (OSError, ValueError):
                print(f"special-casing: {check_rules.rel(path)}: not JSON; skipped")
    return ids, asks, values


def main():
    ids, asks, values = gather()
    if not ids and not asks and not values:
        print("special-casing: notice: T's vectors don't exist yet (engine/tests/cases/); nothing to search for")
        return 0
    print(f"special-casing: {len(ids)} case ids, {len(asks)} ask texts, {len(values)} values of 4+ significant digits")
    hits = []
    id_pattern = re.compile(r"(?<![A-Za-z0-9_-])(" + "|".join(sorted(map(re.escape, ids), key=len, reverse=True)) + r")(?![A-Za-z0-9_])") if ids else None
    for path in check_rules.rust_files(os.path.join(ENGINE, "crates")):
        text = open(path, encoding="utf-8").read()
        code, strings = check_rules.lex(text)
        r = check_rules.rel(path)
        for n, s in strings:
            if id_pattern:
                hits += [f"{r}:{n}: case id {m.group(1)} in a string" for m in id_pattern.finditer(s)]
            hits += [f"{r}:{n}: an ask text in a string: {a!r}" for a in asks if a.lower() in s.lower()]
            hits += [f"{r}:{n}: the value {d} in a string" for d in values if d in s.replace("_", "").replace(".", "")]
        for n, line in enumerate(code.splitlines(), 1):
            if id_pattern:
                hits += [f"{r}:{n}: case id {m.group(1)} in the code" for m in id_pattern.finditer(line)]
            for lit in LITERAL.findall(line):
                d = significant(lit)
                if len(d) >= 4 and d in values:
                    hits.append(f"{r}:{n}: the value {d} as the literal {lit}")
    for h in sorted(set(hits)):
        print(f"special-casing: {h}")
    print(f"special-casing: {len(set(hits))} hits, for the review")
    return 0


if __name__ == "__main__":
    sys.exit(main())
