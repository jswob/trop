"""The Stop hook (R-83), in the build role only.

A build turn can't end while `engine/tools/check fast` is red, unless a STOP-REPORT
in engine/BUILD-LOG.md is newer than the last commit (added since it, or by it).
After three identical blocks the fourth stop is let through, and "STOPPED RED" is
logged, so a session that can't go green doesn't loop.

Logged red doesn't hold a turn (R-83 and R-77, issue #108, 2026-11-08): when the only
red steps are the ones that run T's tests (`runner`, `gating`) and every failing test
there is logged red, the turn ends. A failing test is logged red when it is named on a
`Logged-Red:` line of a commit on this branch (origin/master..HEAD), or when its message
counts its failing vectors ("2 of 9 vectors of g6 differ from refcore:") and that many
of them, each at the start of its line ("g6/when: …"), are named on such lines. Any other
red blocks as before: another step, a test that isn't logged, a vector that isn't, or a
log it can't read. Which group a logged id is for is the group review's to check.
"""

import datetime
import hashlib
import json
import os
import re
import subprocess
import sys

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
ROOT = os.path.dirname(ENGINE)
LOG = os.path.join(ENGINE, "BUILD-LOG.md")
STATE = os.path.join(ENGINE, "target", "check-logs", "stop-state.json")
LOGS = os.path.join(ENGINE, "target", "check-logs")
BLOCKS = 3
HEADING = re.compile(r"^\+#+\s*STOP-REPORT\b", re.MULTILINE)
# logged red (R-77): the commits on this branch, and check fast's steps that run T's tests
BASE = "origin/master"
LOGGED_RED = re.compile(r"^Logged-Red:[ \t]*(\S.*?)[ \t]*$", re.MULTILINE)
TEST_STEPS = ("runner", "gating")
FAILED_STEP = re.compile(r"^FAIL ([\w-]+): ")
# nextest's line for a finished test: `FAIL [   0.009s] (1/3) tally-runner::s_kernel tests::name`
STATUS = re.compile(r"^\s*([A-Z][A-Z-]*)\s+\[\s*[\d.]+s\]\s+(?:\(\s*\d+/\d+\)\s+)?(\S+)\s+(\S+)\s*$")
GREEN = ("PASS", "LEAK")
SUMMARY = re.compile(r"Summary \[[^\]]*\]\s+(\d+) tests? run: (\d+) passed")
# a runner test's count of its failing vectors, its first line (`2 of 9 … fail:`)
COUNTED = re.compile(r"^(?:\S+: )?(\d+) (?:of|failures)\b.*:$")


def git(*args):
    r = subprocess.run(["git", "-C", ROOT, *args], capture_output=True, text=True, check=False)
    return r.stdout if r.returncode == 0 else ""


def stop_report_is_new():
    rel = os.path.relpath(LOG, ROOT)
    if HEADING.search(git("diff", "HEAD", "--", rel)):
        return True
    if HEADING.search(git("show", "HEAD", "--format=", "--", rel)):
        return True
    tracked = git("ls-files", "--", rel).strip()
    if not tracked and os.path.exists(LOG):
        with open(LOG, encoding="utf-8") as f:
            return re.search(r"^#+\s*STOP-REPORT\b", f.read(), re.MULTILINE) is not None
    return False


def logged_ids():
    """Every id on a `Logged-Red:` line of a commit on this branch; none when the base
    can't be read."""
    return set(LOGGED_RED.findall(git("log", "--format=%B", f"{BASE}..HEAD")))


def red_tests(text):
    """{test: [its output blocks]} for every test nextest's log shows red, or None when
    the log's summary counts a different number of red tests than its lines show."""
    red, current = {}, None
    for line in text.splitlines():
        m = STATUS.match(line)
        if m:
            current = None
            if m.group(1) not in GREEN:
                current = []
                red.setdefault(f"{m.group(2)} {m.group(3)}", []).append(current)
        elif line.startswith("─") or SUMMARY.search(line):
            current = None
        elif current is not None:
            current.append(line.strip())
    counted = SUMMARY.findall(text)
    if not counted or int(counted[-1][0]) - int(counted[-1][1]) != len(red):
        return None
    return red


def is_logged(test, blocks, logged):
    """Whether a red test is logged red: named itself (`s_kernel_g6_when`, or with its
    target, `s_kernel::s_kernel_g6_when`), or each vector its message counts is."""
    binary, name = test.split(" ", 1)
    short = name.split("::")[-1]
    if {short, f"{binary.split('::')[-1]}::{short}"} & logged:
        return True
    for block in blocks:
        count, named = None, set()
        for line in block:
            if count is None:
                m = COUNTED.match(line)
                count = int(m.group(1)) if m else None
                continue
            named.update(i for i in logged if line.startswith(f"{i}: "))
        if count and len(named) == count:
            return True
    return False


def not_logged(out, logged):
    """What of check fast's red isn't logged red: [] when every failure is a test of a
    test step that is logged red."""
    left = []
    for line in [l for l in out.splitlines() if l.startswith("FAIL")] or ["FAIL (no failing step named)"]:
        m = FAILED_STEP.match(line)
        step = m.group(1) if m else None
        if step not in TEST_STEPS or not logged:
            left.append(line)
            continue
        try:
            with open(os.path.join(LOGS, f"{step}.log"), encoding="utf-8", errors="replace") as f:
                red = red_tests(f.read())
        except OSError:
            red = None
        if not red:
            left.append(line)
            continue
        left += [f"{step}: {t}" for t, blocks in sorted(red.items()) if not is_logged(t, blocks, logged)]
    return left


def load():
    try:
        with open(STATE, encoding="utf-8") as f:
            return json.load(f)
    except (OSError, ValueError):
        return {}


def save(state):
    os.makedirs(os.path.dirname(STATE), exist_ok=True)
    with open(STATE, "w", encoding="utf-8") as f:
        json.dump(state, f)


def log(line):
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(f"\n- {stamp} {line}\n")


def main():
    if (os.environ.get("TALLY_ROLE") or "unset") != "build":
        return 0
    sys.stdin.read()
    run = subprocess.run([os.path.join(TOOLS, "check"), "fast"], capture_output=True, text=True, cwd=ENGINE, check=False)
    if run.returncode == 0:
        save({})
        return 0
    if stop_report_is_new():
        save({})
        return 0
    left = not_logged(run.stdout, logged_ids())
    if not left:
        save({})
        print("engine stop-check: `check fast` is red only on logged red (a Logged-Red line of this branch names "
              "each failing vector); the turn may end (R-83, R-77).")
        return 0
    failures = [l for l in run.stdout.splitlines() if l.startswith("FAIL")] or run.stdout.splitlines()[-5:]
    summary = "\n".join(failures)
    digest = hashlib.sha256(summary.encode()).hexdigest()
    state = load()
    count = state.get("count", 0) + 1 if state.get("digest") == digest else 1
    if count > BLOCKS:
        log(f"STOPPED RED after {BLOCKS} identical blocks: " + "; ".join(failures)[:500])
        save({})
        return 0
    save({"digest": digest, "count": count})
    print(
        f"engine stop-check (block {count} of {BLOCKS}): `engine/tools/check fast` is red:\n{summary}\n"
        + "".join(f"not logged red: {l}\n" for l in left[:20] if l not in failures)
        + "Fix it, or write a STOP-REPORT in engine/BUILD-LOG.md if a test, the spec or a rule seems wrong (R-83).",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    sys.exit(main())
