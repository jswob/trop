"""The Stop hook (R-83), in the build role only.

A build turn can't end while `engine/tools/check fast` is red, unless a STOP-REPORT
in engine/BUILD-LOG.md is newer than the last commit (added since it, or by it).
After three identical blocks the fourth stop is let through, and "STOPPED RED" is
logged, so a session that can't go green doesn't loop.
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
BLOCKS = 3
HEADING = re.compile(r"^\+#+\s*STOP-REPORT\b", re.MULTILINE)


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
        "Fix it, or write a STOP-REPORT in engine/BUILD-LOG.md if a test, the spec or a rule seems wrong (R-83).",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    sys.exit(main())
