"""Remove a finished worktree and its branch: the lead's clean-up (R-132, issue #18).

    engine/tools/worktree-done <path>

Only the lead role runs it (TALLY_ROLE=lead): the guard refuses it to every other role,
and it refuses itself too. The guard refuses a raw `git worktree remove`, since every
worktree holds the spec, so this is the one way a session removes one. It removes the
worktree at <path> only when:

  - <path> is the top of a linked worktree of this repo: not the main checkout, and not
    the checkout it is run from;
  - a branch is checked out there (not a detached HEAD), and it isn't master;
  - `git status` there is empty: nothing changed, nothing untracked;
  - the branch's commit is in origin/master, fetched first: its pull request merged.

Then it runs `git worktree remove <path>` (never --force) and deletes the local branch.
Each refusal names its reason and exits 1.
"""

import os
import subprocess
import sys

MASTER = "master"


class Refused(Exception):
    """A refusal, with its reason."""


def git(*args, cwd):
    """(exit code, stdout, stderr) of a git command run in `cwd`."""
    try:
        r = subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, check=False)
    except OSError as e:
        raise Refused(f"can't run git ({e})") from e
    return r.returncode, r.stdout.strip(), r.stderr.strip()


def must(*args, cwd):
    code, out, err = git(*args, cwd=cwd)
    if code != 0:
        raise Refused(f"`git {' '.join(args)}` failed in {cwd} (exit {code}): {err[:300]}")
    return out


def worktrees(cwd):
    """The repo's worktrees' paths, realpaths, the main checkout first."""
    out = must("worktree", "list", "--porcelain", cwd=cwd)
    return [os.path.realpath(line[len("worktree ") :]) for line in out.splitlines() if line.startswith("worktree ")]


def done(path, here):
    """Removes the worktree at `path`, judged from the checkout `here`; returns what it did."""
    if not os.path.isdir(path):
        raise Refused(f"{path} isn't a folder")
    path = os.path.realpath(path)
    home = os.path.realpath(must("rev-parse", "--show-toplevel", cwd=here))
    common = os.path.realpath(os.path.join(home, must("rev-parse", "--git-common-dir", cwd=home)))
    code, top, _ = git("rev-parse", "--show-toplevel", cwd=path)
    if code != 0 or os.path.realpath(top) != path:
        raise Refused(f"{path} isn't the top of a git checkout")
    theirs = os.path.realpath(os.path.join(path, must("rev-parse", "--git-common-dir", cwd=path)))
    if theirs != common:
        raise Refused(f"{path} is a checkout of another repo")
    trees = worktrees(home)
    if path not in trees:
        raise Refused(f"{path} isn't one of this repo's worktrees")
    if path == trees[0]:
        raise Refused(f"{path} is the main checkout, which is never removed")
    if path == home:
        raise Refused(f"{path} is the checkout this runs from")
    code, branch, _ = git("symbolic-ref", "-q", "--short", "HEAD", cwd=path)
    if code != 0 or not branch:
        raise Refused(f"{path} has a detached HEAD, so no branch says what was merged")
    if branch.casefold() == MASTER:
        raise Refused(f"{path} has master checked out")
    status = must("status", "--porcelain", cwd=path)
    if status:
        first = status.splitlines()[0]
        raise Refused(f"{path} isn't clean: `git status` lists {len(status.splitlines())} paths ({first.strip()}, …)")
    tip = must("rev-parse", "--verify", f"refs/heads/{branch}^{{commit}}", cwd=home)
    must("fetch", "-q", "origin", MASTER, cwd=home)
    code, _, err = git("merge-base", "--is-ancestor", tip, f"refs/remotes/origin/{MASTER}", cwd=home)
    if code == 1:
        raise Refused(f"its branch {branch} ({tip[:12]}) isn't merged into origin/master")
    if code != 0:
        raise Refused(f"can't tell whether {branch} is merged into origin/master: {err[:300]}")
    must("worktree", "remove", path, cwd=home)
    must("branch", "-D", branch, cwd=home)
    return f"removed {path} and deleted its branch {branch} ({tip[:12]}, merged into origin/master)"


def main(argv):
    if len(argv) != 1 or not argv[0]:
        print("usage: engine/tools/worktree-done <worktree path>", file=sys.stderr)
        return 2
    role = os.environ.get("TALLY_ROLE") or "unset"
    try:
        if role != "lead":
            raise Refused(f"only the lead role runs it, and this is {role} (R-132)")
        what = done(os.path.abspath(argv[0]), os.getcwd())
    except Refused as r:
        print(f"worktree-done: refused {argv[0]}: {r}.", file=sys.stderr)
        return 1
    print(f"worktree-done: {what}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
