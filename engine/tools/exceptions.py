"""The exception counter (R-104, R-109, R-110, R-111; RULEBOOK §7.1).

    engine/tools/exceptions           list every exception by area, and fail on a bad
                                      reason or an area over its budget
    engine/tools/exceptions --check   the same, and fail unless the list equals
                                      engine/EXCEPTIONS.lock
    engine/tools/exceptions --write   regenerate engine/EXCEPTIONS.lock (the builder
                                      commits it with the code)

An exception is any form that switches a rule off somewhere: #[expect], #[allow] or
#[warn], an inner lint attribute, a cfg_attr lint switch, #[ignore], #[rustfmt::skip],
#[coverage], an ast-grep-ignore comment, mutants:: text, and cargo-deny's ignore, skip,
skip-tree, bypass and licence exceptions. ast-grep's syntax tree finds the code forms.
The report-only rules' hits (ast-grep warnings) are listed apart and not counted.
"""

import fnmatch
import json
import os
import re
import subprocess
import sys
import tomllib

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
LOCK = os.path.join(ENGINE, "EXCEPTIONS.lock")
BUDGET = os.path.join(TOOLS, "exceptions-budget.toml")
KERNEL = os.path.join(TOOLS, "kernel.toml")
SCANNED = ["crates", "tests", "fuzz/fuzz_targets"]
# R-104's phrases, matched as stems (a word that starts with one) in the reason with its
# case folded and every run of hyphens, underscores and spaces made one space
BANNED = (
    "for now", "for the moment", "for the time being", "temporar", "todo", "to do", "fixme", "fix me",
    "make the test pass", "make the tests pass", "make tests pass", "backward compat", "backwards compat",
    "clippy is wrong", "silenc", "shut up",
)
ITEM_KINDS = [
    "function_item", "impl_item", "struct_item", "enum_item", "union_item", "trait_item", "mod_item",
    "const_item", "static_item", "type_item", "macro_definition",
]
INLINE_RULES = "\n---\n".join([
    "id: attr\nlanguage: rust\nrule:\n  any:\n    - kind: attribute_item\n    - kind: inner_attribute_item",
    "id: comment\nlanguage: rust\nrule:\n  any:\n    - kind: line_comment\n    - kind: block_comment",
    "id: item\nlanguage: rust\nrule:\n  any:\n" + "\n".join(f"    - kind: {k}" for k in ITEM_KINDS),
])


class Found:
    def __init__(self, area, file, line, item, kind, reason, problem=None):
        self.area, self.file, self.line, self.item = area, file, line, item
        self.kind, self.reason, self.problem = kind, reason, problem

    def lock_line(self):
        return "\t".join([self.area, self.file, self.item, self.kind, self.reason or "-"])


def load(path):
    with open(path, "rb") as f:
        return tomllib.load(f)


def ast_grep(args):
    r = subprocess.run(["ast-grep", *args, "--json=stream"], cwd=ENGINE, capture_output=True, text=True, check=False)
    if r.returncode not in (0, 1):
        raise RuntimeError(f"ast-grep failed: {r.stderr.strip()}")
    return [json.loads(line) for line in r.stdout.splitlines() if line.strip()]


def span(hit):
    b = hit["range"]["byteOffset"]
    return b["start"], b["end"]


# --- areas and reasons -------------------------------------------------------------


def areas():
    budget = load(BUDGET)
    out = []
    for a in budget["area"]:
        globs = a.get("globs", [])
        if globs == "kernel.toml":
            globs = load(KERNEL).get("kernel", [])
        out.append((a["name"], globs, a.get("budget")))
    return out, budget.get("rules", 113)


def area_of(path, in_test, table):
    if in_test:
        return "tests"
    for name, globs, _ in table:
        if any(fnmatch.fnmatchcase(path, g) for g in globs):
            return name
    return "other"


def reason_problem(reason, max_rule):
    """Why `reason` isn't an acceptable reason (R-104), or None."""
    if not reason:
        return "it has no reason"
    m = re.fullmatch(r"R-(\d+): (.+)", reason, re.DOTALL)
    if not m:
        return "its reason isn't in the form `R-<n>: …`"
    if not 1 <= int(m.group(1)) <= max_rule:
        return f"its reason names R-{m.group(1)}, which isn't a rule"
    if len(m.group(2).strip()) < 20:
        return "its reason is under 20 characters after the rule id"
    low = re.sub(r"[\s_-]+", " ", reason.casefold())
    for phrase in BANNED:
        if re.search(r"\b" + re.escape(phrase), low):
            return f"its reason says {phrase!r}"
    if "false positive" in low and "because" not in low:
        return "its reason says 'false positive' without saying why (no 'because')"
    return None


def comment_above(lines, index, marker):
    """The reason in the comment line just above line `index` (0-based), if it starts
    with an R-id: the form for exceptions without a reason field."""
    if index <= 0:
        return None
    above = lines[index - 1].strip()
    if not above.startswith(marker):
        return None
    text = above[len(marker) :].strip()
    return text if text.startswith("R-") else None


# --- the code forms ----------------------------------------------------------------


LINT_SWITCH = re.compile(r"^(expect|allow|warn)\s*\((.*)\)$", re.DOTALL)
REASON = re.compile(r'reason\s*=\s*"((?:[^"\\]|\\.)*)"', re.DOTALL)


def lint_parts(args):
    lints = re.sub(r",?\s*reason\s*=\s*\"(?:[^\"\\]|\\.)*\"", "", args, flags=re.DOTALL)
    m = REASON.search(args)
    return ", ".join(x.strip() for x in lints.split(",") if x.strip()), (m.group(1) if m else None)


def classify(text):
    """(kind, reason, reason_from_comment) for an attribute that is an exception, else
    None. An inner attribute (#![...]) counts only as one of the same forms; #![no_main]
    or #![forbid(...)] switch nothing off."""
    t = text.strip()
    inner = t.startswith("#!")
    body = (t[3:-1] if inner else t[2:-1]).strip()
    pre = "inner " if inner else ""
    head = re.match(r"[\w:]+", body)
    name = head.group(0) if head else ""
    m = LINT_SWITCH.match(body)
    if m:
        lints, reason = lint_parts(m.group(2))
        return (f"{pre}{m.group(1)} {lints}", reason, False)
    if name == "cfg_attr":
        inside = body[len("cfg_attr") :].strip()[1:-1]
        cond, _, attrs = inside.partition(",")
        attrs = attrs.strip()
        m = LINT_SWITCH.match(attrs)
        if m:
            lints, reason = lint_parts(m.group(2))
            return (f"{pre}cfg_attr({cond.strip()}) {m.group(1)} {lints}", reason, False)
        if re.match(r"(coverage|rustfmt::skip|ignore|mutants)", attrs):
            return (f"{pre}cfg_attr({cond.strip()}) {re.sub(r'\s+', ' ', attrs)[:60]}", None, True)
        return None
    if name == "ignore":
        m = re.search(r'=\s*"((?:[^"\\]|\\.)*)"', body)
        return (f"{pre}ignore", m.group(1) if m else None, False)
    if name.startswith("rustfmt::skip"):
        return (f"{pre}{name}", None, True)
    if name == "coverage":
        return (f"{pre}{re.sub(r'\s+', '', body)}", None, True)
    if name.startswith("mutants"):
        return (f"{pre}{name}", None, False)
    return None


def item_name(text):
    first = re.sub(r"\s+", " ", text.split("{", 1)[0]).strip()
    if re.match(r"(pub(\([^)]*\))? )?(unsafe )?impl\b", first):
        return re.sub(r"^.*?\bimpl\b", "impl", first)[:70]
    m = re.search(r"\b(fn|struct|enum|union|trait|mod|const|static|type)\s+([A-Za-z_][A-Za-z0-9_]*)", first)
    if m:
        return f"{m.group(1)} {m.group(2)}"
    m = re.search(r"macro_rules!\s*([A-Za-z_][A-Za-z0-9_]*)", first)
    return f"macro {m.group(1)}" if m else first[:40]


def scan_code(table, max_rule):
    paths = [p for p in SCANNED if os.path.isdir(os.path.join(ENGINE, p))]
    if not paths:
        return []
    by_file = {}
    for hit in ast_grep(["scan", "--inline-rules", INLINE_RULES, *paths]):
        by_file.setdefault(hit["file"], []).append(hit)
    found = []
    for file in sorted(by_file):
        hits = by_file[file]
        with open(os.path.join(ENGINE, file), "rb") as f:
            raw = f.read()
        lines = raw.decode("utf-8").split("\n")
        attrs = [h for h in hits if h["ruleId"] == "attr"]
        comments = [h for h in hits if h["ruleId"] == "comment"]
        items = [h for h in hits if h["ruleId"] == "item"]
        skippable = sorted(span(h) for h in attrs + comments)

        def only_trivia(a, b):
            """Whether raw[a:b] holds nothing but whitespace, comments and attributes."""
            pos = a
            for s, e in skippable:
                if e <= pos or s >= b:
                    continue
                if raw[pos:s].strip():
                    return False
                pos = max(pos, e)
            return not raw[pos:b].strip()

        def attributed(s, e):
            after = [h for h in items if span(h)[0] >= e]
            if not after:
                return None
            nxt = min(after, key=lambda h: span(h)[0])
            return nxt if only_trivia(e, span(nxt)[0]) else None

        def enclosing(s, e):
            around = [h for h in items if span(h)[0] <= s and span(h)[1] >= e]
            return min(around, key=lambda h: span(h)[1] - span(h)[0]) if around else None

        tests = []
        for h in attrs:
            if re.sub(r"\s+", "", h["text"]) == "#[cfg(test)]":
                target = attributed(*span(h))
                if target is not None and target["text"].lstrip().startswith(("mod", "pub mod", "pub(crate) mod")):
                    tests.append(span(target))
                elif target is not None:
                    tests.append(span(target))

        def in_test(s):
            return any(a <= s < b for a, b in tests)

        def place(h):
            s, e = span(h)
            target = None if h["text"].startswith("#!") else attributed(s, e)
            target = target or enclosing(s, e)
            return item_name(target["text"]) if target else "(file)"

        for h in attrs:
            c = classify(h["text"])
            if c is None:
                continue
            kind, reason, from_comment = c
            line = h["range"]["start"]["line"]
            if from_comment:
                reason = comment_above(lines, line, "//")
            area = area_of(file, in_test(span(h)[0]), table)
            found.append(Found(area, file, line + 1, place(h), kind, reason, reason_problem(reason, max_rule)))
        for h in comments:
            if "ast-grep-ignore" in h["text"] or "mutants::" in h["text"]:
                what = "ast-grep-ignore" if "ast-grep-ignore" in h["text"] else "mutants:: comment"
                line = h["range"]["start"]["line"]
                area = area_of(file, in_test(span(h)[0]), table)
                found.append(Found(area, file, line + 1, place(h), what, None, "a tool-suppression comment, never allowed (R-109)"))
    return found


# --- cargo-deny's entries ----------------------------------------------------------


def scan_deny(table, max_rule):
    found = []
    for name in ("deny.toml", "deny-tally.toml"):
        path = os.path.join(ENGINE, name)
        if not os.path.isfile(path):
            continue
        cfg = load(path)
        lines = open(path, encoding="utf-8").read().split("\n")
        area = area_of(name, False, table)

        def line_of(needle, *also):
            for i, l in enumerate(lines):
                if f'"{needle}"' in l and all(a in l for a in also):
                    return i
            return -1

        entries = []
        for e in cfg.get("advisories", {}).get("ignore", []):
            ident = e if isinstance(e, str) else e.get("id", "?")
            reason = None if isinstance(e, str) else e.get("reason")
            entries.append(("advisories.ignore", ident, reason, line_of(ident)))
        bans = cfg.get("bans", {})
        for key in ("skip", "skip-tree"):
            for e in bans.get(key, []):
                ident = e if isinstance(e, str) else e.get("crate") or e.get("name", "?")
                reason = None if isinstance(e, str) else e.get("reason")
                entries.append((f"bans.{key}", ident, reason, line_of(ident)))
        for e in bans.get("build", {}).get("bypass", []):
            ident = e.get("crate") or e.get("name", "?")
            i = next((i for i, l in enumerate(lines) if f'"{ident}"' in l and re.search(r"allow-globs|allow\s*=|build-script|required-features", l)), -1)
            entries.append(("bans.build.bypass", ident, comment_above(lines, i, "#"), i))
        for e in cfg.get("licenses", {}).get("exceptions", []):
            ident = e.get("crate") or e.get("name", "?")
            i = line_of(ident, "allow")
            entries.append(("licenses.exceptions", ident, comment_above(lines, i, "#"), i))
        for kind, ident, reason, i in entries:
            found.append(Found(area, name, i + 1, ident, kind, reason, reason_problem(reason, max_rule)))
    return found


def scan_report_only():
    paths = [p for p in SCANNED if os.path.isdir(os.path.join(ENGINE, p))]
    hits = ast_grep(["scan", *paths]) if paths else []
    out = []
    for h in hits:
        if h.get("severity") in ("warning", "info", "hint"):
            out.append(Found("report-only", h["file"], h["range"]["start"]["line"] + 1, "-", h["ruleId"], None))
    return out


# --- output ------------------------------------------------------------------------


def lock_text(found, report):
    head = [
        "# engine/EXCEPTIONS.lock: every exception, by area (R-110). Generated by",
        "# `engine/tools/exceptions --write`; commit it with the code. Columns: area, file,",
        "# item, kind and lints, reason. Lines carry no line numbers, so moving code doesn't",
        "# change the lock; adding or removing an exception does.",
        f"# count: {len(found)}; report-only hits: {len(report)}",
    ]
    body = sorted(f.lock_line() for f in found) + sorted(f.lock_line() for f in report)
    return "\n".join(head + body) + "\n"


def main():
    args = sys.argv[1:]
    if args not in ([], ["--check"], ["--write"]):
        print("usage: engine/tools/exceptions [--check|--write]", file=sys.stderr)
        return 64
    table, max_rule = areas()
    found = scan_code(table, max_rule) + scan_deny(table, max_rule)
    report = scan_report_only()
    counts = {}
    for f in found:
        counts[f.area] = counts.get(f.area, 0) + 1
    parts = []
    for name, _, budget in table:
        n = counts.get(name, 0)
        parts.append(f"{name} {n}/{budget}" if budget is not None else f"{name} {n}")
    if counts.get("other"):
        parts.append(f"other {counts['other']}")
    print(f"exceptions: {len(found)} ({', '.join(parts)}); report-only hits: {len(report)}")
    for name in [a[0] for a in table] + ["other"]:
        mine = sorted((f for f in found if f.area == name), key=lambda f: (f.file, f.line))
        for f in mine:
            print(f"  {name}  {f.file}:{f.line}  {f.item}  {f.kind}  {json.dumps(f.reason) if f.reason else '(no reason)'}")
    for f in sorted(report, key=lambda f: (f.file, f.line)):
        print(f"  report-only  {f.file}:{f.line}  {f.kind}")
    problems = [f"{f.file}:{f.line} ({f.item}, {f.kind}): {f.problem}" for f in found if f.problem]
    for name, _, budget in table:
        if budget is not None and counts.get(name, 0) > budget:
            problems.append(f"area {name} has {counts[name]} exceptions, over its budget of {budget} (R-111); file a STOP-REPORT")
    if counts.get("other"):
        problems.append("exceptions outside every area in tools/exceptions-budget.toml")
    text = lock_text(found, report)
    if args == ["--write"]:
        with open(LOCK, "w", encoding="utf-8") as f:
            f.write(text)
        print(f"exceptions: wrote engine/EXCEPTIONS.lock")
    elif args == ["--check"]:
        try:
            with open(LOCK, encoding="utf-8") as f:
                old = f.read()
        except OSError:
            old = None
        if old is None:
            problems.append("engine/EXCEPTIONS.lock is missing; run engine/tools/exceptions --write and commit it")
        elif old != text:
            was = {l for l in old.splitlines() if not l.startswith("#")}
            now = {l for l in text.splitlines() if not l.startswith("#")}
            for l in sorted(now - was):
                problems.append(f"not in EXCEPTIONS.lock: {l}")
            for l in sorted(was - now):
                problems.append(f"in EXCEPTIONS.lock but gone: {l}")
            if was == now:
                problems.append("EXCEPTIONS.lock differs (its header or an exception's count); rewrite it with --write")
    for p in problems:
        print(f"FAIL {p}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
