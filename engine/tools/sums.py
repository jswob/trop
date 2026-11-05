"""sums: the two checksum files, rewritten in a merge, in any role (R-80, R-112; the retro
after M0, decision 3).

    engine/tools/sums            check the merge, rewrite both files, stage them
    engine/tools/sums --check    the checks alone; writes nothing

Every merge of a branch that changed T's tests or a rule file conflicts in
engine/tests/TESTS.sha256 or engine/tools/RULES.sha256, which only the tests and the
rules roles may rewrite. This tool rewrites both, in any role, but only in a merge and
only when the merge hides no hand edit:

  - a merge is in progress (MERGE_HEAD), and its only unmerged paths are the two
    checksum files; or HEAD is a merge commit no remote branch holds yet (then
    `git commit --amend --no-edit` takes the rewritten files in);
  - every protected file the merge's result changes against a parent (the spec, the
    oracle, T's tests, the rule files) is one parent's version, or git's clean
    three-way merge of the two (`git merge-file`), byte for byte; and the working tree
    holds that result (nothing protected changed beside the index).

Then it writes TESTS.sha256 with T's own engine/tests/tools/sums.py, RULES.sha256 with
check-frozen --write-rules, runs check-frozen, and stages both files. Each refusal
names its reason and exits 1.
"""

import os
import subprocess
import sys
import tempfile

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
sys.path.insert(0, TOOLS)
import check_frozen  # noqa: E402  the rule files outside engine/
import guard  # noqa: E402  the one list of rule files and the oracle

SUMS = ("engine/tests/TESTS.sha256", "engine/tools/RULES.sha256")


class Refused(Exception):
    """A refusal, with its reason."""


def git(*args, check=True, raw=False):
    r = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=False)
    if check and r.returncode != 0:
        raise Refused(f"`git {' '.join(args)}` failed: {r.stderr.decode(errors='replace').strip()[:300]}")
    return r if raw else r.stdout.decode(errors="replace").strip()


def protected(path):
    """Whether a repo-relative path is the spec, the oracle, T's tests or a rule file."""
    if path in SUMS:
        return False
    folded = path.casefold()
    if folded.startswith(("engine/spec/", "engine/tests/")):
        return True
    if path in check_frozen.OUTSIDE or any(path.startswith(o + "/") for o in check_frozen.OUTSIDE):
        return True
    return guard.is_oracle(folded) or guard.is_rule(folded)


def blob(rev, path):
    """A file's bytes at `rev` (None when it isn't there); rev None is the working tree."""
    if rev is None:
        full = os.path.join(ROOT, path)
        if not os.path.isfile(full):
            return None
        with open(full, "rb") as f:
            return f.read()
    r = git("cat-file", "blob", f"{rev}:{path}", check=False, raw=True)
    return r.stdout if r.returncode == 0 else None


def clean_merge(base, ours, theirs):
    """git's clean three-way merge of one file's three versions, or None when it conflicts."""
    if ours == theirs:
        return ours
    if base == ours:
        return theirs
    if base == theirs:
        return ours
    if None in (base, ours, theirs):
        return None
    with tempfile.TemporaryDirectory(prefix="tally-sums-") as d:
        names = []
        for name, data in (("ours", ours), ("base", base), ("theirs", theirs)):
            names.append(os.path.join(d, name))
            with open(names[-1], "wb") as f:
                f.write(data)
        r = subprocess.run(["git", "merge-file", "-p", "--quiet", *names], capture_output=True, check=False)
    return r.stdout if r.returncode == 0 else None


def the_merge():
    """(ours, theirs, result rev or None for the working tree, what it is), or a refusal."""
    if git("rev-parse", "-q", "--verify", "MERGE_HEAD", check=False):
        theirs = git("rev-parse", "MERGE_HEAD")
        unmerged = [p for p in git("diff", "--name-only", "--diff-filter=U").splitlines() if p]
        others = [p for p in unmerged if p not in SUMS]
        if others:
            raise Refused(f"{len(others)} path(s) besides the checksum files are unmerged ({others[0]}, …): resolve them first")
        return git("rev-parse", "HEAD"), theirs, None, "the merge in progress"
    parents = git("rev-list", "--parents", "-n", "1", "HEAD").split()[1:]
    if len(parents) != 2:
        raise Refused("no merge is in progress (no MERGE_HEAD) and HEAD isn't a merge commit of two parents: it runs only in a merge")
    if git("branch", "-r", "--contains", "HEAD"):
        raise Refused("HEAD is a merge commit a remote branch already holds: a pushed merge isn't rewritten")
    return parents[0], parents[1], "HEAD", "the merge commit HEAD (not pushed)"


def problems_of(ours, theirs, result):
    base = git("merge-base", ours, theirs, check=False)
    changed = set()
    for parent in (ours, theirs):
        args = ["diff", "--name-only", "--no-renames", parent] + ([result] if result else [])
        changed |= {p for p in git(*args).splitlines() if p}
    # the working tree must hold what is committed or staged, for every protected file
    beside = git("diff", "--name-only", "--no-renames", *(["HEAD"] if result else [])).splitlines()
    problems = [f"{p}: the working tree differs from the {'commit' if result else 'index'}" for p in beside
                if p and protected(p) and p not in SUMS]
    for path in sorted(p for p in changed if protected(p)):
        now = blob(result, path)
        o, t = blob(ours, path), blob(theirs, path)
        if now in (o, t):
            continue
        b = blob(base, path) if base else None
        merged = clean_merge(b, o, t)
        if merged is None or now != merged:
            problems.append(f"{path}: neither parent's version nor git's clean merge of the two, so a hand edit (R-112)")
    return problems


def main(argv):
    global ROOT
    if argv not in ([], ["--check"]):
        print("usage: engine/tools/sums [--check]", file=sys.stderr)
        return 2
    try:
        ROOT = os.path.realpath(subprocess.run(["git", "rev-parse", "--show-toplevel"], cwd=ENGINE, capture_output=True,
                                               text=True, check=True).stdout.strip())
        ours, theirs, result, what = the_merge()
        problems = problems_of(ours, theirs, result)
    except (Refused, subprocess.CalledProcessError, OSError) as e:
        print(f"sums: refused: {e}.", file=sys.stderr)
        return 1
    if problems:
        for p in problems:
            print(f"FAIL sums: {p}")
        print(f"sums: refused: {len(problems)} protected file(s) in {what} aren't a parent's or a clean merge's")
        return 1
    print(f"sums: {what} ({ours[:12]} + {theirs[:12]}) changes no protected file by hand")
    if argv == ["--check"]:
        return 0
    for argv_ in (["python3", "-I", "-B", os.path.join(ENGINE, "tests", "tools", "sums.py")],
                  ["python3", "-I", "-B", os.path.join(TOOLS, "check_frozen.py"), "--write-rules"],
                  ["python3", "-I", "-B", os.path.join(TOOLS, "check_frozen.py")]):
        r = subprocess.run(argv_, cwd=ROOT, capture_output=True, text=True, check=False)
        out = (r.stdout + r.stderr).strip()
        print(out)
        if r.returncode != 0:
            print(f"sums: {os.path.basename(argv_[3])} failed (exit {r.returncode}); nothing staged")
            return 1
    git("add", "--", *SUMS)
    print("sums: staged " + " and ".join(SUMS) + (": now `git commit --amend --no-edit`" if result else ": commit the merge"))
    return 0


ROOT = ""

if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
