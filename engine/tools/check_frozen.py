"""check-frozen: the frozen files are what they were frozen as (R-80).

    engine/tools/check-frozen                 verify
    engine/tools/check-frozen --write-rules   rewrite engine/tools/RULES.sha256 (the
                                              `rules` role, in a rule change's commit, R-112)

It verifies:
  - engine/spec/SHA256SUMS: every file's checksum, and that it lists every file;
  - engine/tests/TESTS.sha256 the same way, once T has written it;
  - that engine/spec/ matches the git tag named in engine/spec/VERSION;
  - engine/tools/RULES.sha256 over every rule file (the set the guard protects).
"""

import hashlib
import os
import subprocess
import sys

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
ROOT = os.path.dirname(ENGINE)
sys.path.insert(0, TOOLS)
import guard  # noqa: E402  the guard's list of rule files is the one list

RULES = os.path.join(TOOLS, "RULES.sha256")
SKIP_DIRS = {"__pycache__", "target", "corpus", "artifacts", "coverage", ".git"}
SKIP_FILES = {".DS_Store"}
OUTSIDE = [".github/workflows/engine.yml", ".claude/settings.json"]


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 16), b""):
            h.update(block)
    return h.hexdigest()


def files_under(top):
    """Every file under `top`, relative to it, with '/' separators, sorted."""
    found = []
    for d, dirs, names in os.walk(top):
        dirs[:] = sorted(x for x in dirs if x not in SKIP_DIRS)
        for n in names:
            if n not in SKIP_FILES:
                found.append(os.path.relpath(os.path.join(d, n), top).replace(os.sep, "/"))
    return sorted(found)


def read_sums(path):
    sums = {}
    with open(path, encoding="utf-8") as f:
        for n, line in enumerate(f, 1):
            line = line.rstrip("\n")
            if not line:
                continue
            digest, sep, name = line.partition("  ")
            if not sep or len(digest) != 64:
                raise ValueError(f"{os.path.relpath(path, ROOT)}:{n}: not `<sha256>  <path>`")
            sums[name] = digest
    return sums


def verify_sums(top, sums_name, label):
    """Problems with `top`/`sums_name`: a changed, missing or unlisted file."""
    path = os.path.join(top, sums_name)
    try:
        sums = read_sums(path)
    except (OSError, ValueError) as e:
        return [f"{label}: {e}"]
    problems = []
    present = set(files_under(top)) - {sums_name}
    for name, digest in sorted(sums.items()):
        full = os.path.join(top, name)
        if not os.path.isfile(full):
            problems.append(f"{label}: {name} is listed but missing")
        elif sha256(full) != digest:
            problems.append(f"{label}: {name} changed (checksum differs)")
    for name in sorted(present - set(sums)):
        problems.append(f"{label}: {name} is not listed")
    return problems


def git(*args):
    return subprocess.run(["git", "-C", ROOT, *args], capture_output=True, text=True, check=False)


def verify_tag():
    version = os.path.join(ENGINE, "spec", "VERSION")
    tag = "tally-1.0"
    if os.path.exists(version):
        with open(version, encoding="utf-8") as f:
            tag = f.read().strip()
    if git("rev-parse", "--verify", "--quiet", f"refs/tags/{tag}").returncode != 0:
        return [f"spec: the tag {tag} (from spec/VERSION) isn't in this clone; fetch the tags"]
    problems = []
    diff = git("diff", "--name-status", tag, "--", "engine/spec")
    if diff.returncode != 0:
        return [f"spec: git diff against {tag} failed: {diff.stderr.strip()}"]
    for line in diff.stdout.splitlines():
        problems.append(f"spec: differs from {tag}: {line}")
    extra = git("ls-files", "--others", "--", "engine/spec")
    for line in extra.stdout.splitlines():
        if os.path.basename(line) not in SKIP_FILES and "__pycache__" not in line.split("/"):
            problems.append(f"spec: {line} isn't in {tag}")
    return problems


def rule_files():
    """Every rule file, repo-relative, sorted: what the guard refuses to every role but `rules`."""
    found = []
    for name in files_under(ENGINE):
        rel = f"engine/{name}"
        if rel.startswith(("engine/spec/", "engine/tests/")) or rel == "engine/tools/RULES.sha256":
            continue
        if guard.is_rule(rel.casefold()):
            found.append(rel)
    found += [p for p in OUTSIDE if os.path.isfile(os.path.join(ROOT, p))]
    return sorted(found)


def verify_rules():
    try:
        sums = read_sums(RULES)
    except OSError:
        return ["rules: engine/tools/RULES.sha256 is missing"]
    except ValueError as e:
        return [f"rules: {e}"]
    problems = []
    current = rule_files()
    for name in current:
        if name not in sums:
            problems.append(f"rules: {name} is a rule file but not in RULES.sha256")
        elif sha256(os.path.join(ROOT, name)) != sums[name]:
            problems.append(f"rules: {name} changed (checksum differs); a rule changes only through R-112")
    for name in sorted(set(sums) - set(current)):
        problems.append(f"rules: {name} is in RULES.sha256 but missing")
    return problems


def write_rules():
    with open(RULES, "w", encoding="utf-8") as f:
        for name in rule_files():
            f.write(f"{sha256(os.path.join(ROOT, name))}  {name}\n")


def main():
    args = sys.argv[1:]
    if args == ["--write-rules"]:
        write_rules()
        print(f"check-frozen: wrote {os.path.relpath(RULES, ROOT)} ({len(rule_files())} files)")
        return 0
    if args:
        print("usage: engine/tools/check-frozen [--write-rules]", file=sys.stderr)
        return 64
    problems = verify_sums(os.path.join(ENGINE, "spec"), "SHA256SUMS", "spec")
    tests = os.path.join(ENGINE, "tests")
    if os.path.exists(os.path.join(tests, "TESTS.sha256")):
        problems += verify_sums(tests, "TESTS.sha256", "tests")
    else:
        print("check-frozen: notice: engine/tests/TESTS.sha256 isn't written yet (T writes it); skipped")
    problems += verify_tag()
    problems += verify_rules()
    for p in problems:
        print(f"FAIL {p}")
    if problems:
        print(f"check-frozen: {len(problems)} problem{'' if len(problems) == 1 else 's'}")
        return 1
    print("check-frozen: spec, tag and rules unchanged")
    return 0


if __name__ == "__main__":
    sys.exit(main())
