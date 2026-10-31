"""check-frozen: the frozen files are what they were frozen as (R-80).

    engine/tools/check-frozen                 verify
    engine/tools/check-frozen --write-rules   rewrite engine/tools/RULES.sha256 (the
                                              `rules` role, in a rule change's commit, R-112)

It verifies:
  - engine/spec/SHA256SUMS: every file's checksum, and that it lists every file;
  - engine/tools/SHA256SUMS the same way over the reference interpreter (refcore/);
  - engine/tests/TESTS.sha256 the same way, once T has written it;
  - that engine/spec/ and the oracle (refcore/, refcore_fast/, tools/SHA256SUMS) match
    the commit tools/frozen.toml pins: the tag named in engine/spec/VERSION, or an
    approved amendment after it that kept the version;
  - engine/tools/RULES.sha256 over every rule file (the set the guard protects);
  - that .claude/settings.local.json (git-ignored, so no checksum covers it) doesn't
    switch off the hooks or the permission prompts: no disableAllHooks, no hooks, no
    bypassPermissions mode (R-81).
"""

import hashlib
import json
import os
import subprocess
import sys
import tomllib

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
ROOT = os.path.dirname(ENGINE)
sys.path.insert(0, TOOLS)
import guard  # noqa: E402  the guard's list of rule files is the one list

RULES = os.path.join(TOOLS, "RULES.sha256")
FROZEN = os.path.join(TOOLS, "frozen.toml")
# what the pin holds: the spec and the oracle, repo-relative
PINNED = ["engine/spec", "engine/tools/refcore", "engine/tools/refcore_fast", "engine/tools/SHA256SUMS"]
# the suite's run output (engine/tests/.gitignore), never checksummed
TESTS_OUTPUT = ("cache/", "out/", "results/")
SKIP_DIRS = {"__pycache__", "target", "corpus", "artifacts", "coverage", ".git"}
SKIP_FILES = {".DS_Store"}
# rule files outside engine/ (a folder stands for every file under it)
OUTSIDE = [
    ".github/workflows/engine.yml",
    ".claude/settings.json",
    ".claude/skills/engine-rust",
    ".claude/agents/engine-reviewer.md",
]
LOCAL_SETTINGS = ".claude/settings.local.json"


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


def verify_sums(top, sums_name, label, within=None, skip=()):
    """Problems with `top`/`sums_name`: a changed, missing or unlisted file. With
    `within`, only the files under those folders of `top` must be listed; names that
    start with one of `skip` needn't be."""
    path = os.path.join(top, sums_name)
    try:
        sums = read_sums(path)
    except (OSError, ValueError) as e:
        return [f"{label}: {e}"]
    problems = []
    if within is None:
        present = set(files_under(top)) - {sums_name}
    else:
        present = {f"{w}/{n}" for w in within for n in files_under(os.path.join(top, w))}
    present = {n for n in present if not n.startswith(skip)}
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


def verify_pin():
    """The spec and the oracle match the commit tools/frozen.toml pins, which is the
    tag spec/VERSION names or descends from it."""
    version = os.path.join(ENGINE, "spec", "VERSION")
    tag = "tally-1.0"
    if os.path.exists(version):
        with open(version, encoding="utf-8") as f:
            tag = f.read().strip()
    try:
        with open(FROZEN, "rb") as f:
            frozen = tomllib.load(f)
    except (OSError, tomllib.TOMLDecodeError) as e:
        return [f"spec: engine/tools/frozen.toml: {e}"]
    if frozen.get("tag") != tag:
        return [f"spec: spec/VERSION names {tag} but tools/frozen.toml pins {frozen.get('tag')}; a new version moves the pin through R-112"]
    if git("rev-parse", "--verify", "--quiet", f"refs/tags/{tag}").returncode != 0:
        return [f"spec: the tag {tag} (from spec/VERSION) isn't in this clone; fetch the tags"]
    pin = frozen.get("pin", tag)
    if git("rev-parse", "--verify", "--quiet", f"{pin}^{{commit}}").returncode != 0:
        return [f"spec: the pinned commit {pin} (tools/frozen.toml) isn't in this clone; fetch the history"]
    if pin != tag and git("merge-base", "--is-ancestor", tag, pin).returncode != 0:
        return [f"spec: the pinned commit {pin} doesn't descend from the tag {tag}"]
    name = tag if pin == tag else f"{tag}'s amendment {pin[:7]}"
    problems = []
    diff = git("diff", "--name-status", pin, "--", *PINNED)
    if diff.returncode != 0:
        return [f"spec: git diff against {name} failed: {diff.stderr.strip()}"]
    for line in diff.stdout.splitlines():
        problems.append(f"spec: differs from {name}: {line}")
    extra = git("ls-files", "--others", "--", *PINNED)
    for line in extra.stdout.splitlines():
        parts = line.split("/")
        if parts[-1] not in SKIP_FILES and "__pycache__" not in parts and not line.endswith(".pyc"):
            problems.append(f"spec: {line} isn't in {name}")
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
    for p in OUTSIDE:
        full = os.path.join(ROOT, p)
        if os.path.isdir(full):
            found += [f"{p}/{n}" for n in files_under(full)]
        elif os.path.isfile(full):
            found.append(p)
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


def verify_local_settings():
    """The git-ignored local settings can't switch off the next session's hooks or its
    permission prompts (R-81): a build-role script could write them, and no checksum
    or clean CI runner sees the file."""
    path = os.path.join(ROOT, LOCAL_SETTINGS)
    if not os.path.lexists(path):
        return []
    try:
        with open(path, encoding="utf-8") as f:
            data = json.load(f)
    except (OSError, ValueError) as e:
        return [f"settings: {LOCAL_SETTINGS} can't be read as JSON ({e}); it must not switch off the hooks (R-81)"]
    if not isinstance(data, dict):
        return [f"settings: {LOCAL_SETTINGS} isn't a JSON object (R-81)"]
    problems = []
    if "disableAllHooks" in data:
        problems.append(f"settings: {LOCAL_SETTINGS} sets disableAllHooks (R-81)")
    if "hooks" in data:
        problems.append(f"settings: {LOCAL_SETTINGS} sets hooks, which belong in .claude/settings.json (R-81)")
    permissions = data.get("permissions")
    if isinstance(permissions, dict) and permissions.get("defaultMode") == "bypassPermissions":
        problems.append(f"settings: {LOCAL_SETTINGS} sets the bypassPermissions mode (R-81)")
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
    problems += verify_sums(TOOLS, "SHA256SUMS", "refcore", within=["refcore"])
    tests = os.path.join(ENGINE, "tests")
    if os.path.exists(os.path.join(tests, "TESTS.sha256")):
        problems += verify_sums(tests, "TESTS.sha256", "tests", skip=TESTS_OUTPUT)
    else:
        print("check-frozen: notice: engine/tests/TESTS.sha256 isn't written yet (T writes it); skipped")
    problems += verify_pin()
    problems += verify_rules()
    problems += verify_local_settings()
    for p in problems:
        print(f"FAIL {p}")
    if problems:
        print(f"check-frozen: {len(problems)} problem{'' if len(problems) == 1 else 's'}")
        return 1
    print("check-frozen: spec, oracle, pin and rules unchanged")
    return 0


if __name__ == "__main__":
    sys.exit(main())
