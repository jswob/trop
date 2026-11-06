"""commit-check: the commits a pull request brings, read from git (R-100, R-112). CI runs
it after `check full`, on a pull request only, against the branch it merges into.

    engine/tools/commit-check BASE      the commits in BASE..HEAD, merges left out

It fails on:
  - a commit message with an AI co-author or attribution line (R-100): a co-author, a
    "generated with Claude" line, a `Claude-Session:` line or a claude.ai session link
    (the retro after M0). The guard's tripwire reads only the command text; this reads
    the commits themselves.
  - a commit that touches engine/, .github/workflows/engine.yml or .claude/skills/engine-*
    whose subject isn't `type(scope): …` from engine/CONVENTIONS.md's lists, or is over
    72 characters (R-100, the retro after M0, decision 6). A commit whose parent has no
    engine/CONVENTIONS.md is listed, never failed.
  - a commit that changes a rule file (the guard's list, the CI workflow and the
    settings) without a trailer line `Rule-Change: R-<n> approved <YYYY-MM-DD> …`, or
    without changing tools/RULES.sha256 and a proof (a selftest case or guard-test) in
    the same commit (R-112). A rule change with no proof to change says so in its
    trailer: `Rule-Change: R-<n> approved <date>; no case: <why>`.
  - a logged red (R-77, the review after S, 2026-11-03): a commit with `Logged-Red:`
    lines, each naming one red vector or check id, may be red in `check full`, but only
    as T's re-run commit after a spec ruling: it changes nothing outside engine/tests/,
    and a later commit in BASE..HEAD changes engine/crates/ (the builder's, which turns
    those ids green). CI's `check full` on the pull request's head is the rest: green.

It always prints every rule-changing commit with its trailer, and every logged red
with its ids, for the audit to read (R-113). A commit made before this check existed (its parent has no
tools/commit_check.py) is listed, never failed.
"""

import os
import re
import subprocess
import sys

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
ROOT = os.path.dirname(ENGINE)
sys.path.insert(0, TOOLS)
import check_frozen  # noqa: E402  the list of files outside engine/ that are rule files
import guard  # noqa: E402  the one list of rule files

ATTRIBUTION = re.compile(r"co-authored-by:|noreply@anthropic\.com|generated with \[?claude|claude-session:|claude\.ai/code/session",
                         re.IGNORECASE)
# engine/CONVENTIONS.md's commit subject: `type(scope): what changes`, at most 72 characters
TYPES = ("feat", "fix", "spec", "test", "refactor", "perf", "docs", "ci", "chore")
SCOPES = ("parse", "format", "load", "eval", "walk", "book", "ask", "num", "time", "refcore", "fuzz", "rules")
SUBJECT = re.compile(rf"^({'|'.join(TYPES)})\(({'|'.join(SCOPES)})\): \S.*$")
SUBJECT_MAX = 72
CONVENTIONS = "engine/CONVENTIONS.md"
TRAILER = re.compile(r"^Rule-Change: R-([1-9][0-9]*) approved (\d{4}-\d{2}-\d{2})\b(?P<rest>.*)$", re.MULTILINE)
SELF = "engine/tools/commit_check.py"
LOGGED_RED = re.compile(r"^Logged-Red:[ \t]*(?P<ids>.*)$", re.MULTILINE)
PROOFS = ("engine/tools/rules-selftest/cases/", "engine/tools/guard_test.py")


def git(*args):
    r = subprocess.run(["git", "-C", ROOT, *args], capture_output=True, text=True, check=False)
    if r.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)}: {r.stderr.strip()}")
    return r.stdout


def is_rule(path):
    if path in check_frozen.OUTSIDE:
        return True
    return path.startswith("engine/") and guard.is_rule(path.casefold())


def logged_red(short, message, changed, later, problems, reds):
    """A commit's `Logged-Red:` lines (R-77): only T's re-run commit, its red ids listed,
    the builder's commit after it on the same branch."""
    ids = [m.group("ids").strip() for m in LOGGED_RED.finditer(message)]
    if not ids:
        return
    reds.append(f"{short}: {', '.join(i or '(none)' for i in ids)}")
    if any(not i for i in ids):
        problems.append(f"{short}: a Logged-Red line names no vector id (R-77)")
    outside = sorted(p for p in changed if p and not p.startswith("engine/tests/"))
    if outside:
        problems.append(f"{short}: a logged red changes {outside[0]}, outside engine/tests/: only T's re-run commit may be red (R-77)")
    if not any(p.startswith("engine/crates/") for c in later for p in c):
        problems.append(f"{short}: a logged red with no later commit changing engine/crates/ on the branch (R-77)")


def engine_commit(changed):
    return any(p.startswith(("engine/", ".claude/skills/engine-")) or p == ".github/workflows/engine.yml" for p in changed if p)


def subject(sha, short, title, changed, problems, listed):
    """engine/CONVENTIONS.md's subject, for a commit that touches the engine (decision 6)."""
    if not engine_commit(changed):
        return
    if SUBJECT.match(title) and len(title) <= SUBJECT_MAX:
        return
    before = subprocess.run(["git", "-C", ROOT, "cat-file", "-e", f"{sha}^:{CONVENTIONS}"], capture_output=True, check=False)
    if before.returncode != 0:
        listed.append(f"{short}: its subject is off the conventions (made before {CONVENTIONS})")
    elif not SUBJECT.match(title):
        problems.append(f"{short}: the subject isn't `type(scope): …` with a type from {', '.join(TYPES)} "
                        f"and a scope from {', '.join(SCOPES)} ({CONVENTIONS}, R-100)")
    else:
        problems.append(f"{short}: the subject is {len(title)} characters, over {SUBJECT_MAX} ({CONVENTIONS}, R-100)")


def check(base):
    problems, listed, reds, early = [], [], [], []
    shas = git("rev-list", "--no-merges", "--reverse", f"{base}..HEAD").split()
    files = {sha: git("show", "--format=", "--name-only", "--no-renames", sha).split("\n") for sha in shas}
    for k, sha in enumerate(shas):
        message = git("show", "-s", "--format=%B", sha)
        title = message.splitlines()[0] if message else ""
        short = f"{sha[:7]} {title}"
        for line in message.splitlines():
            if ATTRIBUTION.search(line):
                problems.append(f"{short}: an AI co-author or attribution line: {line.strip()!r} (R-100)")
        changed = files[sha]
        subject(sha, short, title, changed, problems, early)
        logged_red(short, message, changed, [files[s] for s in shas[k + 1:]], problems, reds)
        rules = sorted(p for p in changed if p and is_rule(p))
        if not rules:
            continue
        trailers = TRAILER.findall(message)
        listed.append(f"{short}: {len(rules)} rule file(s); " + ("; ".join(f"R-{n} approved {d}" for n, d, _ in trailers) or "no trailer"))
        before = subprocess.run(["git", "-C", ROOT, "cat-file", "-e", f"{sha}^:{SELF}"], capture_output=True, check=False)
        if before.returncode != 0:
            listed[-1] += " (made before this check)"
            continue
        if not trailers:
            problems.append(f"{short}: changes {rules[0]} with no Rule-Change trailer (R-112)")
            continue
        if "engine/tools/RULES.sha256" not in changed:
            problems.append(f"{short}: changes rule files but not tools/RULES.sha256 (R-112)")
        no_case = any("no case:" in rest for _, _, rest in trailers)
        if not no_case and not any(p.startswith(PROOFS) for p in changed):
            problems.append(f"{short}: changes rule files with no selftest case or guard-test row, and its trailer gives no 'no case:' reason (R-112)")
    return problems, listed, reds, early


def main():
    if len(sys.argv) != 2:
        print("usage: engine/tools/commit-check BASE", file=sys.stderr)
        return 64
    try:
        problems, listed, reds, early = check(sys.argv[1])
    except RuntimeError as e:
        print(f"FAIL commit-check: {e}")
        return 1
    for line in listed:
        print(f"commit-check: rule change: {line}")
    for line in reds:
        print(f"commit-check: logged red: {line}")
    for line in early:
        print(f"commit-check: listed: {line}")
    for p in problems:
        print(f"FAIL {p}")
    if problems:
        print(f"commit-check: {len(problems)} problem{'' if len(problems) == 1 else 's'}")
        return 1
    print(f"commit-check: {len(listed)} rule-changing commit{'' if len(listed) == 1 else 's'}, no attribution line")
    return 0


if __name__ == "__main__":
    sys.exit(main())
