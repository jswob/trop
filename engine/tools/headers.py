"""headers: every .rs file under engine/crates/ starts with its `//!` header (R-120).

    engine/tools/headers

The header is the file's first lines, each starting `//!`, in this order:
  1. the purpose: one sentence, before `Spec:`, of 6 to 30 words (the rulebook's "about
     15"), ending in a full stop; it may wrap onto several lines;
  2. a line starting `Spec:`: the L and E sections the file implements;
  3. a line starting `Twin:`: the refcore file that does the same work;
  4. a line starting `Invariants:`: what must hold.
A field may wrap onto the next `//!` lines, and none may be empty (`Spec: none, a stub.`
is fine). A fence adds a `PERF:` line after them (R-130). Free text may follow after a
blank `//!` line. The trace (engine/tools/trace, R-128) checks the citations the header
holds and a fence's `PERF:` line; this script checks only the form.

Exit codes: 0 every file has its header; 1 a file hasn't (one line per problem, each
naming R-120); 64 usage.
"""

import os
import re
import sys

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
sys.path.insert(0, TOOLS)
import check_rules  # noqa: E402  the file walk

CRATES = os.path.join(ENGINE, "crates")
FIELDS = ("Spec", "Twin", "Invariants")
KEY = re.compile(r"(Spec|Twin|Invariants|PERF):(.*)")
MIN_WORDS, MAX_WORDS = 6, 30
# a sentence's end inside the text: a stop, then a space and a capital
INNER_END = re.compile(r"[.!?][\"')\]`]*\s+[A-Z]")


def parse(text):
    """The `//!` header of a file's text, or None when its first line isn't `//!`: a dict
    with "purpose" (its text), "order" (the keys in the order they come), "end" (the last
    header line) and, per key, (line, text), a wrapped field's lines joined."""
    lines = text.splitlines()
    if not lines or not lines[0].startswith("//!"):
        return None
    head, purpose, current, end = {"order": []}, [], None, 0
    for end, line in enumerate(lines, 1):
        if not line.startswith("//!"):
            end -= 1
            break
        body = line[3:].strip()
        m = KEY.fullmatch(body)
        if m:
            current = m.group(1)
            head["order"].append(current)
            head.setdefault(current, (end, m.group(2).strip()))
        elif not body:
            current = "free text" if current else None
        elif current is None and not head["order"]:
            purpose.append(body)
        elif current in head and head[current][0] < end:
            ln, t = head[current]
            head[current] = (ln, f"{t} {body}".strip())
    head["purpose"], head["end"] = " ".join(purpose), end
    return head


def problems_of(text):
    head = parse(text)
    if head is None:
        return [(1, "no //! header: a purpose sentence, then Spec:, Twin: and Invariants: lines")]
    found = []
    purpose = head["purpose"]
    words = len(purpose.split())
    if not purpose:
        found.append((1, "the header has no purpose sentence before Spec:"))
    elif not MIN_WORDS <= words <= MAX_WORDS:
        found.append((1, f"the purpose is {words} words, not one sentence of {MIN_WORDS} to {MAX_WORDS}"))
    elif INNER_END.search(purpose) or not re.search(r"[.!?][\"')\]`]*$", purpose):
        found.append((1, "the purpose isn't one sentence ending in a full stop"))
    order = head["order"]
    if order[: len(FIELDS)] != list(FIELDS) or any(order.count(k) > 1 for k in FIELDS):
        missing = [k + ":" for k in FIELDS if k not in head]
        what = f"no {', '.join(missing)} line" if missing else "the fields are out of order or repeated"
        found.append((1, f"{what}: the header is the purpose, then Spec:, Twin:, Invariants:"))
    for key in FIELDS + ("PERF",):
        if key in head and not head[key][1]:
            found.append((head[key][0], f"the {key}: line is empty"))
    return found


def main():
    if sys.argv[1:]:
        print("usage: engine/tools/headers", file=sys.stderr)
        return 64
    problems, files = [], check_rules.rust_files(CRATES)
    for path in files:
        with open(path, encoding="utf-8") as f:
            text = f.read()
        problems += [f"{check_rules.rel(path)}:{n}: {p} (R-120)" for n, p in problems_of(text)]
    for p in problems:
        print(f"FAIL {p}")
    if problems:
        print(f"headers: {len(problems)} problem{'' if len(problems) == 1 else 's'} in {len(files)} files")
        return 1
    print(f"headers: {len(files)} files, each with its //! header (R-120)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
