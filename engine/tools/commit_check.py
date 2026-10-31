"""commit-check: the commits a pull request brings, read from git (R-100, R-112). CI runs
it after `check full`, on a pull request only, against the branch it merges into.

    engine/tools/commit-check BASE      the commits in BASE..HEAD, merges left out

It fails on:
  - a commit message with an AI co-author or attribution line (R-100). The guard's
    tripwire reads only the command text; this reads the commits themselves.
  - a commit that changes a rule file (the guard's list, the CI workflow and the
    settings) without a trailer line `Rule-Change: R-<n> approved <YYYY-MM-DD> …`, or
    without changing tools/RULES.sha256 and a proof (a selftest case or guard-test) in
    the same commit (R-112). A rule change with no proof to change says so in its
    trailer: `Rule-Change: R-<n> approved <date>; no case: <why>`.

It always prints every rule-changing commit with its trailer, for the audit to read
(R-113). A commit made before this check existed (its parent has no
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

ATTRIBUTION = re.compile(r"co-authored-by:|noreply@anthropic\.com|generated with \[?claude", re.IGNORECASE)
TRAILER = re.compile(r"^Rule-Change: R-([1-9][0-9]*) approved (\d{4}-\d{2}-\d{2})\b(?P<rest>.*)$", re.MULTILINE)
SELF = "engine/tools/commit_check.py"
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


def check(base):
    problems, listed = [], []
    for sha in git("rev-list", "--no-merges", "--reverse", f"{base}..HEAD").split():
        message = git("show", "-s", "--format=%B", sha)
        title = message.splitlines()[0] if message else ""
        short = f"{sha[:7]} {title}"
        for line in message.splitlines():
            if ATTRIBUTION.search(line):
                problems.append(f"{short}: an AI co-author or attribution line: {line.strip()!r} (R-100)")
        changed = git("show", "--format=", "--name-only", "--no-renames", sha).split("\n")
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
    return problems, listed


def main():
    if len(sys.argv) != 2:
        print("usage: engine/tools/commit-check BASE", file=sys.stderr)
        return 64
    try:
        problems, listed = check(sys.argv[1])
    except RuntimeError as e:
        print(f"FAIL commit-check: {e}")
        return 1
    for line in listed:
        print(f"commit-check: rule change: {line}")
    for p in problems:
        print(f"FAIL {p}")
    if problems:
        print(f"commit-check: {len(problems)} problem{'' if len(problems) == 1 else 's'}")
        return 1
    print(f"commit-check: {len(listed)} rule-changing commit{'' if len(listed) == 1 else 's'}, no attribution line")
    return 0


if __name__ == "__main__":
    sys.exit(main())
