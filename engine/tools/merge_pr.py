"""Merge a pull request whose CI is green: the lead's only way to merge (R-132, issue #18).

    engine/tools/merge-pr <n>

Only the lead role runs it (TALLY_ROLE=lead): the guard refuses it to every other role,
and it refuses itself too. It merges pull request <n> only when every condition holds:

  - it is open, its base is master and its head is another branch of this repo;
  - every check run and status on the head commit is success, neutral or skipped, and
    none is pending (and at least one ran);
  - if it touches engine/**, the check `check full` ran on the head commit and passed,
    its step `engine/tools/check full` passed, and so did `engine/tools/commit-check`
    when it ran;
  - its body says `Closes #<issue>`;
  - GitHub reports it MERGEABLE;
  - its head branch isn't build/*: a milestone waits for the user at its gate;
  - it changes none of the walls (WALLS below), so a session can't loosen a wall and
    have the lead merge it.

Then it runs `gh pr merge <n> --merge --match-head-commit <sha>`, so a push after the
checks were read can't slip in, and prints the merge commit. Each refusal names its
reason and exits 1. Anything it can't read is a refusal: it fails closed.
"""

import json
import os
import re
import subprocess
import sys

MASTER = "master"
# the walls: a pull request changing one waits for the user (issue #18). Every other
# rule file is covered too, since a rule file's change rewrites RULES.sha256 (R-80).
WALLS = {
    "engine/tools/guard",
    "engine/tools/guard.py",
    "engine/tools/merge-pr",
    "engine/tools/merge_pr.py",
    "engine/tools/worktree-done",
    "engine/tools/worktree_done.py",
    "engine/tools/check",
    "engine/tools/check.py",
    "engine/tools/check-frozen",
    "engine/tools/check_frozen.py",
    "engine/tools/rules.sha256",
    ".claude/settings.json",
    ".github/workflows/engine.yml",
}
GREEN = {"SUCCESS", "NEUTRAL", "SKIPPED"}
CHECK_FULL = "check full"
CLOSES = re.compile(r"\bcloses #\d+\b", re.IGNORECASE)
FIELDS = "number,state,baseRefName,headRefName,headRefOid,isCrossRepository,mergeable,changedFiles,statusCheckRollup,body"


class Refused(Exception):
    """A refusal, with its reasons."""


def gh(*args):
    """gh's stdout; a failure is a refusal."""
    try:
        r = subprocess.run(["gh", *args], capture_output=True, text=True, check=False)
    except OSError as e:
        raise Refused(f"can't run gh ({e})") from e
    if r.returncode != 0:
        raise Refused(f"`gh {' '.join(args[:3])} …` failed (exit {r.returncode}): {r.stderr.strip()[:300]}")
    return r.stdout


def parse(text, what):
    try:
        return json.loads(text)
    except json.JSONDecodeError as e:
        raise Refused(f"can't read {what} ({e})") from e


def check_reasons(rollup):
    """The reasons the head commit's checks aren't all green, from gh's statusCheckRollup."""
    if not isinstance(rollup, list) or not rollup:
        return ["no check ran on the head commit"]
    reasons = []
    for c in rollup:
        kind = c.get("__typename")
        if kind == "CheckRun":
            name, status, conclusion = c.get("name"), c.get("status"), c.get("conclusion")
            if status != "COMPLETED":
                reasons.append(f"the check `{name}` is pending ({status})")
            elif conclusion not in GREEN:
                reasons.append(f"the check `{name}` ended {conclusion}")
        elif kind == "StatusContext":
            name, state = c.get("context"), c.get("state")
            if state in ("PENDING", "EXPECTED"):
                reasons.append(f"the status `{name}` is pending ({state})")
            elif state != "SUCCESS":
                reasons.append(f"the status `{name}` is {state}")
        else:
            reasons.append(f"a check of an unknown kind ({kind!r})")
    return reasons


def check_full_job(rollup):
    """The job id of the Engine workflow's `check full` run, or None when it didn't run."""
    for c in rollup if isinstance(rollup, list) else []:
        if c.get("__typename") == "CheckRun" and c.get("name") == CHECK_FULL and c.get("workflowName") == "Engine":
            m = re.search(r"/job/(\d+)", c.get("detailsUrl") or "")
            return m.group(1) if m else "unknown"
    return None


def job_reasons(job, head):
    """The reasons `check full`'s job (from the Actions API) doesn't prove the head commit."""
    if not isinstance(job, dict):
        return [f"can't read the `{CHECK_FULL}` job"]
    reasons = []
    if job.get("head_sha") != head:
        reasons.append(f"the `{CHECK_FULL}` job ran on {job.get('head_sha')}, not on the head commit {head}")
    if job.get("conclusion") != "success":
        reasons.append(f"the `{CHECK_FULL}` job ended {job.get('conclusion')}")
    steps = {s.get("name"): s for s in job.get("steps") or []}
    full = steps.get("engine/tools/check full")
    if full is None or full.get("conclusion") != "success":
        reasons.append(f"the step `engine/tools/check full` didn't pass ({'missing' if full is None else full.get('conclusion')})")
    commit = steps.get("engine/tools/commit-check")
    if commit is not None and commit.get("conclusion") not in ("success", "skipped"):
        reasons.append(f"the step `engine/tools/commit-check` ended {commit.get('conclusion')}")
    return reasons


def changed_paths(files):
    """Every path the pull request touches, a rename's old name included."""
    paths = []
    for f in files:
        paths += [p for p in (f.get("filename"), f.get("previous_filename")) if p]
    return paths


def refusals(pr, files, job=None):
    """Every reason not to merge `pr` (gh pr view's JSON), given its changed files (the
    REST API's, one object each) and `check full`'s job (None if it wasn't read)."""
    reasons = []
    if pr.get("state") != "OPEN":
        reasons.append(f"it is {pr.get('state')}, not open")
    if pr.get("baseRefName") != MASTER:
        reasons.append(f"its base is {pr.get('baseRefName')!r}, not master")
    head = pr.get("headRefName") or ""
    if head.casefold() == MASTER:
        reasons.append("its head is master itself")
    if pr.get("isCrossRepository") is not False:
        reasons.append("its head isn't a branch of this repo")
    if head.casefold().startswith("build/"):
        reasons.append(f"its head {head} is a milestone (build/*), which waits for the user's approval at its gate")
    if not CLOSES.search(pr.get("body") or ""):
        reasons.append("its body doesn't say `Closes #<issue>`")
    if pr.get("mergeable") != "MERGEABLE":
        reasons.append(f"GitHub reports it {pr.get('mergeable')}, not MERGEABLE")
    reasons += check_reasons(pr.get("statusCheckRollup"))
    if len(files) != pr.get("changedFiles"):
        reasons.append(f"GitHub listed {len(files)} of its {pr.get('changedFiles')} changed files, so the walls can't be checked")
    paths = changed_paths(files)
    walls = sorted({p for p in paths if p.casefold() in WALLS})
    if walls:
        reasons.append(f"it changes the walls ({', '.join(walls)}), which only the user merges")
    if any(p.startswith("engine/") for p in paths):
        if check_full_job(pr.get("statusCheckRollup")) is None:
            reasons.append(f"it touches engine/ but `{CHECK_FULL}` didn't run on the head commit")
        elif job is None:
            reasons.append(f"it touches engine/ but `{CHECK_FULL}`'s job couldn't be read")
        else:
            reasons += job_reasons(job, pr.get("headRefOid"))
    return reasons


def gather(n):
    """(pr, files, job) from GitHub, for pull request `n`."""
    pr = parse(gh("pr", "view", n, "--json", FIELDS), "the pull request")
    if not isinstance(pr, dict):
        raise Refused("gh's answer isn't a pull request")
    lines = gh("api", "--paginate", f"repos/{{owner}}/{{repo}}/pulls/{n}/files?per_page=100", "--jq", ".[]")
    files = [parse(line, "the changed files") for line in lines.splitlines() if line.strip()]
    if not all(isinstance(f, dict) for f in files):
        raise Refused("gh's list of changed files isn't a list of files")
    job, job_id = None, check_full_job(pr.get("statusCheckRollup"))
    if job_id is not None and job_id != "unknown":
        job = parse(gh("api", f"repos/{{owner}}/{{repo}}/actions/jobs/{job_id}"), f"the `{CHECK_FULL}` job")
    return pr, files, job


def main(argv):
    if len(argv) != 1 or not re.fullmatch(r"[1-9][0-9]*", argv[0]):
        print("usage: engine/tools/merge-pr <pull request number>", file=sys.stderr)
        return 2
    n = argv[0]
    role = os.environ.get("TALLY_ROLE") or "unset"
    try:
        if role != "lead":
            raise Refused(f"only the lead role runs it, and this is {role} (R-132)")
        pr, files, job = gather(n)
        reasons = refusals(pr, files, job)
        if reasons:
            raise Refused("; ".join(reasons))
        gh("pr", "merge", n, "--merge", "--match-head-commit", pr["headRefOid"])
        merged = parse(gh("pr", "view", n, "--json", "mergeCommit"), "the merge commit")
        oid = (merged.get("mergeCommit") or {}).get("oid") if isinstance(merged, dict) else None
    except Refused as r:
        print(f"merge-pr: refused #{n}: {r}.", file=sys.stderr)
        return 1
    print(f"merge-pr: merged #{n} into master as {oid or '(gh named no merge commit)'}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
