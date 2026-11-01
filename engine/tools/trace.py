"""trace: the spec citations in the engine's code, checked, and the reverse map (R-128,
R-75, R-130).

    engine/tools/trace          check
    engine/tools/trace --map    check, then print the reverse map

Citations (R-128). In every comment of the .rs files under engine/crates/ (`//!`, `///`,
`//` and block comments; string literals are data and aren't read) it reads each spec
citation: a document, then a section, a line or both:

    L §4.6   E §4.2   P §8.1                       the short forms the spec and the
                                                   rulebook use (P is PERFORMANCE)
    LANGUAGE §4.6   ENGINE §1   PERFORMANCE §8     the long forms (`.md` may follow)
    L §4.6 l.716   L l.716                         a line, after a section or alone
    L §3.6, §5.7 and §5.10                         more sections of the same document

L is engine/spec/LANGUAGE.md, E is ENGINE.md, P is PERFORMANCE.md. A section must be a
numbered heading of its document (`### 4.6 The read rule` is §4.6, `## 1. …` is §1); a
line must be inside the document. Either failing is an error, everywhere. A cited line
outside its cited section is a report line. A numbered list item is not a section: write
`E §9 item 1`, not `E §9.1`.

Functions (R-75). A function in the kernel (the globs of tools/kernel.toml's `kernel`, empty
until the lead sets them at S, R-85) whose `///` doc comment cites no section is an
error; outside the kernel it is a report line. Tests are skipped: `#[test]` and
`#[cfg(test)]` functions, functions inside a `#[cfg(test)]` module, and the files under a
crate's tests/, benches/ and examples/.

Fences (R-130). A file whose `//!` header has a `PERF:` line must name a P8 row that
exists in PERFORMANCE §8 (`P8-10`, or a micro row: `P8-M2`) and its plain twin (`plain twin: read_plain.rs`): a
file beside it, a path under engine/, or `refcore`, the twin of last resort, when the
header's `Twin:` line names a file under engine/tools/refcore/. Otherwise an error.

The map (--map), per section of L, E and P that the code cites (a subsection's citation
doesn't cover its parent, nor the reverse): the files citing it and how (their header's
`Spec:` line, a function's doc comment, another comment), each file's `Twin:`, and T's
checks tied to the section; then the sections with no code yet. A check is tied to a
section where a row of T's map (engine/tests/map/*.tsv, the check id first) holds a
citation of it. T's map tags checks with the case set's edges (cases/boundaries.md), not
with spec sections, so today that column is nearly empty: see the map's last line.

Exit codes: 0 no error (report lines may print); 1 an error (lines starting FAIL); 64 usage.
"""

import fnmatch
import os
import re
import sys
import tomllib

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
ROOT = os.path.dirname(ENGINE)
sys.path.insert(0, TOOLS)
import check_rules  # noqa: E402  the lexer and the file walk
import headers  # noqa: E402  the //! header's fields

CRATES = os.path.join(ENGINE, "crates")
SPEC = os.path.join(ENGINE, "spec")
MAP = os.path.join(ENGINE, "tests", "map")
DOCS = {"L": "LANGUAGE.md", "E": "ENGINE.md", "P": "PERFORMANCE.md"}
LONG = {"LANGUAGE": "L", "ENGINE": "E", "PERFORMANCE": "P"}
SECTION = r"§(\d+(?:\.\d+)*)"
LINE = r"l\.(\d+)"
# a section, with the word before it if any: the document, or a connector ("and", "steps")
CITE = re.compile(rf"(?:(?<![\w-])([A-Za-z][\w-]*(?:\.md)?)[ \t]*)?{SECTION}(?:,?[ \t]+{LINE})?")
# a line alone: `L l.716`
LINE_ONLY = re.compile(rf"(?<![\w-])(LANGUAGE|ENGINE|PERFORMANCE|L|E|P)(?:\.md)?[ \t]+{LINE}")
HEADING = re.compile(r"(#{1,6})[ \t]+(\d+(?:\.\d+)*)\.?(?:[ \t]+(.*))?$")
FN = re.compile(
    r"^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:default\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?"
    r"(?:extern\s+(?:\"\"\s+)?)?fn\s+([A-Za-z_][A-Za-z0-9_]*)"
)
TEST_MOD = re.compile(r"#\[cfg\(test\)\]\s*(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+\w+\s*\{")
# a P8 row: a numbered one (`P8-10`, `P8-2a`) or a micro row (`P8-M1`, PERFORMANCE §8.4)
P8_ROW = re.compile(r"^\|\s*(P8-(?:\d+[a-z]?|M\d+))\s*\|")
P8_ID = re.compile(r"\bP8-(?:\d+[a-z]?|M\d+)\b")
TWIN_NAME = re.compile(r"\btwin\b[^:;,]*?:?[ \t]*`?([\w./-]+?)`?(?=[.;,)]?(?:\s|$))", re.IGNORECASE)
NOT_LIBRARY = ("tests", "benches", "examples")


# --- the spec --------------------------------------------------------------------------


def load_doc(name):
    """A spec document's sections, {"4.6": (first line, last line, title)}, its line count
    and its P8 rows (for PERFORMANCE)."""
    with open(os.path.join(SPEC, name), encoding="utf-8") as f:
        lines = f.read().splitlines()
    heads, fence = [], None
    for n, line in enumerate(lines, 1):
        m = re.match(r"\s*(`{3,}|~{3,})", line)
        if m:
            fence = None if fence and m.group(1).startswith(fence) else (fence or m.group(1))
            continue
        h = None if fence else HEADING.match(line)
        if h:
            heads.append((n, len(h.group(1)), h.group(2), (h.group(3) or "").strip()))
    sections = {}
    for i, (n, level, num, title) in enumerate(heads):
        end = next((m - 1 for m, lv, _, _ in heads[i + 1 :] if lv <= level), len(lines))
        sections[num] = (n, end, title)
    # a top-level numbered list item is its section's subsection when no heading has that
    # number: the spec's own documents cite ENGINE §9's first item as `E §9.1`
    for i, (n, level, num, title) in enumerate(heads):
        end = next((m - 1 for m, _, _, _ in heads[i + 1 :]), len(lines))
        fence = None
        for k in range(n + 1, end + 1):
            line = lines[k - 1]
            f = re.match(r"\s*(`{3,}|~{3,})", line)
            if f:
                fence = None if fence and f.group(1).startswith(fence) else (fence or f.group(1))
                continue
            item = None if fence else re.match(r"(\d+)\. ", line)
            if item and f"{num}.{item.group(1)}" not in sections:
                sections[f"{num}.{item.group(1)}"] = (k, sections[num][1], "(list item)")
    rows = set()
    if "8" in sections:
        first, last, _ = sections["8"]
        rows = {m.group(1) for line in lines[first - 1 : last] if (m := P8_ROW.match(line))}
    return sections, len(lines), rows


def load_spec():
    return {doc: load_doc(name) for doc, name in DOCS.items()}


def citations(text):
    """The spec citations in a piece of text: (offset, doc, section or None, line or None).
    A section with no document before it (`, §5`, `and §5`, `–§5`) belongs to the last
    document named before it: `E §4.2 steps 1–3, §5` cites E §5, and `BUILDER §13.1, §13.2`
    cites no spec section."""
    found, doc = [], None
    for m in CITE.finditer(text):
        word = m.group(1)
        if word and (word.removesuffix(".md") in LONG or word in DOCS):
            doc = LONG.get(word.removesuffix(".md"), word)
        elif word and (word[0].isupper() or "-" in word or word.endswith(".md") or any(c.isdigit() for c in word)):
            doc = None  # another document's section: SCAFFOLD §4, engine-tests §4, R5 §2
        if doc:
            found.append((m.start(), doc, m.group(2), int(m.group(3)) if m.group(3) else None))
    for m in LINE_ONLY.finditer(text):
        found.append((m.start(), LONG.get(m.group(1), m.group(1)), None, int(m.group(2))))
    return sorted(found)


def judge(spec, doc, section, line):
    """(error, report) about one citation; each None when it holds."""
    sections, length, _ = spec[doc]
    name = f"{doc} §{section}" if section else f"{doc} l.{line}"
    if section and section not in sections:
        return f"{name} isn't a numbered heading of {DOCS[doc]}", None
    if line and line > length:
        return f"{name} l.{line}: the line is past the end of {DOCS[doc]} ({length} lines)", None
    if section and line:
        first, last, _ = sections[section]
        if not first <= line <= last:
            return None, f"{name} l.{line}: the line is outside the section (lines {first}–{last})"
    return None, None


# --- the code --------------------------------------------------------------------------


def comments(text):
    """Every comment of Rust source as (line, text), a block comment one item per line;
    strings and characters are skipped as check_rules.lex skips them."""
    out, i, n = [], 0, len(text)
    raw = re.compile(r'(?:b|c)?r(#*)"')
    char = re.compile(r"'(?:[^'\\\n]|\\(?:u\{[0-9a-fA-F]+\}|x[0-9a-fA-F]{2}|.))'")
    while i < n:
        if text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            out.append((text.count("\n", 0, i) + 1, text[i:j]))
            i = j
            continue
        if text.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif text.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            first = text.count("\n", 0, i) + 1
            out += [(first + k, part) for k, part in enumerate(text[i:j].split("\n"))]
            i = j
            continue
        m = raw.match(text, i)
        if m and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
            end = text.find('"' + m.group(1), m.end())
            i = n if end < 0 else end + 1 + len(m.group(1))
            continue
        if text[i] == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            i = j + 1
            continue
        m = char.match(text, i)
        i = m.end() if m else i + 1
    return out


def test_lines(code):
    """The lines inside `#[cfg(test)]` modules, from the lexed code."""
    inside = set()
    for m in TEST_MOD.finditer(code):
        depth, j = 1, m.end()
        while j < len(code) and depth:
            depth += {"{": 1, "}": -1}.get(code[j], 0)
            j += 1
        first = code.count("\n", 0, m.start()) + 1
        inside.update(range(first, first + code.count("\n", m.start(), j) + 1))
    return inside


def functions(text):
    """The functions of a file that aren't tests: (line, name, doc comment text, the doc
    comment's lines)."""
    code, _ = check_rules.lex(text)
    raw, lexed, skip = text.splitlines(), code.splitlines(), test_lines(code)
    found = []
    for i, line in enumerate(lexed):
        m = FN.match(line)
        if not m or i + 1 in skip:
            continue
        doc, attrs, j, in_attr = [], [], i - 1, False
        while j >= 0:
            r, c = raw[j].strip(), lexed[j].strip() if j < len(lexed) else ""
            if c.startswith("#!"):
                break
            if in_attr or c.startswith("#["):
                attrs.append(c)
                in_attr = not c.startswith("#[")
            elif c.endswith("]"):
                attrs.append(c)
                in_attr = True
            elif r.startswith("///"):
                doc.append((j + 1, r[3:]))
            elif c:
                break
            j -= 1
        if any(re.search(r"#\[(?:\w+::)*test\b|\bcfg\(test\)", a) for a in attrs):
            continue
        found.append((i + 1, m.group(1), " ".join(t for _, t in reversed(doc)), {n for n, _ in doc}))
    return found


def kernel_globs():
    try:
        with open(check_rules.KERNEL, "rb") as f:
            return tomllib.load(f).get("kernel", [])
    except OSError:
        return []


def fence_problems(path, head, spec):
    """R-130: a PERF: line names an existing P8 row and its plain twin."""
    if "PERF" not in head:
        return []
    n, perf = head["PERF"]
    rows, problems = spec["P"][2], []
    ids = P8_ID.findall(perf)
    if not ids:
        problems.append((n, "the PERF: line names no P8 row of PERFORMANCE §8"))
    problems += [(n, f"the PERF: line's {i} isn't a row of PERFORMANCE §8") for i in ids if i not in rows]
    twin = TWIN_NAME.search(perf)
    if not twin or not twin_exists(path, twin.group(1), head):
        problems.append((n, "the PERF: line names no plain twin that exists (`plain twin: <file>`)"))
    return problems


def twin_exists(path, name, head):
    if name.lower() == "refcore":
        twin = head.get("Twin", (0, ""))[1]
        return any(os.path.isfile(os.path.join(ROOT, p)) for p in re.findall(r"engine/tools/refcore/[\w./-]*\w", twin))
    here = os.path.dirname(path)
    candidates = [os.path.join(here, name), os.path.join(here, name + ".rs"), os.path.join(here, name, "mod.rs"),
                  os.path.join(ENGINE, name), os.path.join(ROOT, name)]
    return any(os.path.isfile(c) for c in candidates)


def scan(spec):
    """Every file's citations, functions and header, checked: (files, errors, reports)."""
    kernel, files, errors, reports = kernel_globs(), [], [], []
    for path in check_rules.rust_files(CRATES):
        r = check_rules.rel(path)
        local = os.path.relpath(path, ENGINE).replace(os.sep, "/")
        with open(path, encoding="utf-8") as f:
            text = f.read()
        head = headers.parse(text) or {}
        parts = local.split("/")
        library = len(parts) < 4 or parts[2] not in NOT_LIBRARY
        in_kernel = any(fnmatch.fnmatchcase(local, g) for g in kernel)
        fns = functions(text) if library else []
        spec_first = head.get("Spec", (0, ""))[0]
        spec_last = head.get("Twin", (spec_first + 1, ""))[0] - 1 if spec_first else 0
        doc_of = {n: name for _, name, _, lines in fns for n in lines}
        cites = []
        for n, comment in comments(text):
            for _, doc, section, line in citations(comment):
                error, report = judge(spec, doc, section, line)
                if error:
                    errors.append(f"{r}:{n}: {error} (R-128)")
                if report:
                    reports.append(f"{r}:{n}: {report} (R-128)")
                if section and not error:
                    how = "Spec:" if spec_first <= n <= spec_last else f"fn {doc_of[n]}" if n in doc_of else f"a comment, l.{n}"
                    cites.append((doc, section, how))
        for n, name, doc, _ in fns:
            if any(section for _, _, section, _ in citations(doc)):
                continue
            msg = f"{r}:{n}: fn {name}: its doc comment cites no spec section"
            if in_kernel:
                errors.append(f"{msg}, and the file is in the kernel (R-75)")
            else:
                reports.append(f"{msg} (R-75; an error only in the kernel)")
        errors += [f"{r}:{n}: {p} (R-130)" for n, p in fence_problems(path, head, spec)]
        twin = head.get("Twin", (0, "none"))[1]
        files.append({"rel": r, "cites": cites, "fns": fns, "twin": twin, "kernel": in_kernel})
    return files, errors, reports


# --- the map ---------------------------------------------------------------------------


def t_checks():
    """{(doc, section): [check ids]} from T's map: a citation anywhere in a row of a .tsv
    under engine/tests/map/ ties the row's check (its first column) to that section."""
    tied = {}
    if not os.path.isdir(MAP):
        return tied
    for name in sorted(os.listdir(MAP)):
        if not name.endswith(".tsv") or re.search(r"hidden|holdout", name, re.IGNORECASE):
            continue
        with open(os.path.join(MAP, name), encoding="utf-8") as f:
            for row in f.read().splitlines()[1:]:
                check = row.split("\t", 1)[0]
                for _, doc, section, _ in citations(row):
                    if section:
                        tied.setdefault((doc, section), []).append(check)
    return tied


def print_map(spec, files):
    tied = t_checks()
    order = [(d, s) for d in DOCS for s in spec[d][0]]
    by_section = {}
    for f in files:
        for doc, section, how in f["cites"]:
            by_section.setdefault((doc, section), {}).setdefault(f["rel"], set()).add(how)
    print("trace: the reverse map: spec section -> code -> twin -> T's checks")
    for key in order:
        if key not in by_section:
            continue
        doc, section = key
        print(f"{doc} §{section} {spec[doc][0][section][2]}")
        for rel, hows in sorted(by_section[key].items()):
            twin = next(f["twin"] for f in files if f["rel"] == rel)
            print(f"  code: {rel} ({'; '.join(sorted(hows))})  twin: {twin}")
        checks = tied.get(key, [])
        shown = ", ".join(checks[:8]) + (f" and {len(checks) - 8} more" if len(checks) > 8 else "")
        print(f"  T's checks: {shown or "none in T's map"}")
    bare = [f"{d} §{s}" for d, s in order if (d, s) not in by_section]
    print(f"trace: the sections with no code yet, {len(bare)} of {len(order)}:")
    for doc in DOCS:
        row = [b for b in bare if b.startswith(doc + " ")]
        if row:
            print(f"  {', '.join(row)}")
    n_checks = len({c for cs in tied.values() for c in cs})
    print(f"trace: T's map ties {n_checks} check{'s' if n_checks != 1 else ''} to {len(tied)} section"
          f"{'s' if len(tied) != 1 else ''}; it tags checks with the case set's edges, not with spec sections")


def main():
    args = sys.argv[1:]
    if args not in ([], ["--map"]):
        print("usage: engine/tools/trace [--map]", file=sys.stderr)
        return 64
    try:
        spec = load_spec()
    except OSError as e:
        print(f"FAIL the spec can't be read: {e} (R-128)")
        return 1
    files, errors, reports = scan(spec)
    for line in reports:
        print(f"trace: report: {line}")
    for line in errors:
        print(f"FAIL {line}")
    if args == ["--map"]:
        print_map(spec, files)
    n_cites = sum(len(f["cites"]) for f in files)
    n_fns = sum(len(f["fns"]) for f in files)
    n_kernel = sum(1 for f in files if f["kernel"])
    summary = (f"{len(files)} files ({n_kernel} in the kernel), {n_cites} section citations, "
               f"{n_fns} functions, {len(reports)} report lines")
    if errors:
        print(f"trace: {len(errors)} error{'' if len(errors) == 1 else 's'}; {summary}")
        return 1
    print(f"trace: every citation is in the spec; {summary}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
