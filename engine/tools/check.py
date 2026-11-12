"""The engine's checks, the way CI runs them (R-82, R-77).

    engine/tools/check quick    after an edit, before each commit inside a group, and the Stop
                                hook: fmt, both clippy passes, the counter (`--fast`),
                                check-frozen, check-rules, headers, trace, the ast-grep scan, then
                                every test of the workspace and of the gating milestones in the
                                `quick` profile (`tests`), and the next milestone's report
    engine/tools/check commit   before a group's last commit: quick with the counter in full, and
                                build, doc, doctest, gungraun-build, both deny runs, the tree
                                probe, ast-grep's tests, special-casing, both spec load checks
                                and the fault-injection probe
    engine/tools/check full     everything CI runs on a pull request (C §6)
    engine/tools/check group G  one CI job's steps of `full` (R-77, 2026-11-13); `check list` names them
    engine/tools/check groups   fails unless every step of `full` is in exactly one group and
                                every rule with a selftest case in exactly one shard;
                                `--matrix` prints the groups as CI's job matrix (JSON)
    engine/tools/check list     the steps of each mode and group
    engine/tools/check step S   one step of `full` alone (rules-selftest runs probes this way)
    engine/tools/check walls-key   the hash of the walls' self-tests' inputs (CI's cache key)
    --fresh                     the walls' self-tests run whatever their hash says, and the memo
                                is ignored (the nightly, on demand)

`fast` is `quick`'s old name. It runs from engine/ with a clean environment: nothing from the
caller's shell changes the flags, the compiler, the profiles or the test seeds. `full`,
`group` and `step` build without incremental caches (CARGO_INCREMENTAL=0) and on every core;
`quick` and `commit` build incrementally, with 6 build jobs and 6 test threads (the Mac's
rule, R-82, 2026-11-13); the caller's CARGO_INCREMENTAL (0 or 1), CARGO_BUILD_JOBS and
NEXTEST_TEST_THREADS win. It prints one line per failed step and keeps each step's full
output in target/check-logs/<step>.log. A report step prints its one line and never fails:
`milestone-report`, the next milestone's tests (tools/milestones.py).

The memo (R-82, 2026-11-13): a green `quick`, `commit` or `full` keeps its key in
target/check-memo/<mode>: the hash of every file git sees (tracked, or untracked and not
ignored: its path, executable bit and working-tree bytes), the repository's tags, the steps'
environment, the platform and the tools' versions (memo_key). The key is taken before the run
and again after it, and kept only if the two agree. A run whose key is a kept green one of the
same or a larger mode prints one line and passes. `--fresh` ignores it, and CI never reads it.

The walls' self-tests, `guard-test` and `rules-selftest` (R-77, R-78, issue #6), are
skipped when every input they read is byte for byte as in a run of theirs that passed:
the hash of their inputs (walls_inputs) is kept in target/walls-green/<step> when they
pass, and in CI in a cache entry keyed on it. A skip prints one line with the hash. What
they read of the engine's own code, beyond the files their cases plant into, is what
check full's own steps check in the same run (fmt, clippy, ast-grep, check-rules,
check-frozen, exceptions, deny, the probes); a fresh run (nightly, on demand) runs them
whatever the hash says. On CI rules-selftest runs in shards (R-78, 2026-11-13), each kept
green under its own name.
"""

import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import time

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
ROOT = os.path.dirname(ENGINE)
sys.path.insert(0, TOOLS)
import milestones  # noqa: E402  the marker: which milestones' tests gate (issue #14)
import check_frozen  # noqa: E402  the rule files, the set the guard protects (R-80)

LOGS = os.path.join(ENGINE, "target", "check-logs")
WALLS_GREEN = os.path.join(ENGINE, "target", "walls-green")
FRESH = False  # `--fresh`: the walls' self-tests run whatever their inputs' hash says
NIGHTLY = "nightly-2026-10-21"
FUZZ_SECONDS = 20  # per target; three targets keep the step near its one-target time (R-91, 2026-11-04)
FUZZ_TIMEOUT = 10  # seconds one input may run before it counts as a hang
MODES = ("quick", "commit", "full")  # each runs at least what the one before it runs
# quick and commit's build jobs and test threads, unless the caller sets them (R-82, 2026-11-13)
LOCAL_JOBS = "6"

# The caller's environment can't change how the engine is built or tested (R-82)
CLEARED = {
    "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS",
    "RUSTC", "RUSTDOC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CLIPPY_CONF_DIR",
    "CLIPPY_ARGS", "RUSTUP_TOOLCHAIN", "CARGO_INCREMENTAL", "RUSTFMT", "RUSTC_BOOTSTRAP",
}
CLEARED_PREFIXES = ("CARGO_PROFILE_", "CARGO_BUILD_", "CARGO_UNSTABLE_", "INSTA_", "PROPTEST_", "NEXTEST_",
                    "GUNGRAUN_", "CRITERION_")
# the seeds the tests run with: fixed, so a run repeats
FIXED = {
    "PROPTEST_RNG_SEED": "20261022",
    "PROPTEST_DISABLE_FAILURE_PERSISTENCE": "1",
    "CARGO_TERM_COLOR": "never",
    "RUST_BACKTRACE": "1",
}
# clippy and cargo say these without failing; each means a rule silently does nothing
SILENT = [
    re.compile(r"does not refer to a reachable"),
    re.compile(r"expected a .* found"),
    re.compile(r"use of a deprecated field"),
    re.compile(r"will be ignored"),
    re.compile(r"unused manifest key"),
]


def clean_env():
    env = {}
    for k, v in os.environ.items():
        if k in CLEARED or k.startswith(CLEARED_PREFIXES):
            continue
        # CARGO_TARGET_<triple>_RUSTFLAGS, _LINKER and _RUNNER change the build; the dir is fine
        if k.startswith("CARGO_TARGET_") and k != "CARGO_TARGET_DIR":
            continue
        env[k] = v
    env.update(FIXED)
    # incremental builds off unless the caller sets them (the retro after M0, decision 5):
    # a worktree's target/ stays a third of the size, and the output is the same
    env["CARGO_INCREMENTAL"] = os.environ.get("CARGO_INCREMENTAL") if os.environ.get("CARGO_INCREMENTAL") in ("0", "1") else "0"
    return env


def mode_env(mode):
    """The steps' environment for `mode`: quick and commit build incrementally, with 6 build
    jobs and 6 test threads, unless the caller sets them (R-82, 2026-11-13); none of the
    three changes a result."""
    env = clean_env()
    if mode in ("quick", "commit"):
        if os.environ.get("CARGO_INCREMENTAL") not in ("0", "1"):
            env["CARGO_INCREMENTAL"] = "1"
        for name in ("CARGO_BUILD_JOBS", "NEXTEST_TEST_THREADS"):
            given = os.environ.get(name, "")
            env[name] = given if given.isdigit() and int(given) > 0 else LOCAL_JOBS
    return env


CLIPPY = ["cargo", "clippy", "--workspace", "--all-features", "--locked"]
CLIPPY_LIB = [*CLIPPY, "--lib", "--bins", "--", "-D", "warnings"]
# the test pass may use assert! and {:?} (R-109); everything else is denied as in the lib pass
CLIPPY_TESTS = [*CLIPPY, "--tests", "--benches", "--", "-D", "warnings", "-A", "clippy::disallowed_macros", "-A", "clippy::use_debug"]


# T's runner, the way every step that runs it runs it
RUNNER = ["cargo", "nextest", "run", "--locked", "-p", "tally-runner", "--all-features"]
# quick's one test run: every test target of the workspace (T's runner, tally's and the CLI's
# own), optimised with debug assertions and overflow checks on (R-82, 2026-11-13); the gating
# milestones' targets are added by name
QUICK_TESTS = ["cargo", "nextest", "run", "--locked", "--workspace", "--all-features", "--tests",
               "--cargo-profile", "quick"]


class Report(str):
    """A report step's line: printed, never a failure."""


def tool(name, *args):
    return [os.path.join(TOOLS, name), *args]


def spec_realities():
    """What the spec's load scan reads: tally 2's scan has no default run, so the 79
    models and the household (BUILD-ORDER: "over the 79 models, the household")."""
    models = os.path.join(ENGINE, "spec", "models")
    names = sorted(n for n in os.listdir(models) if n.endswith(".tally")) if os.path.isdir(models) else []
    return [f"spec/models/{n}" for n in names] + ["spec/amendments/household"]


# (name, argv or a function, extra env)
def steps(mode):
    """The steps of `quick`, `commit` or `full` (`fast` is quick), in the order they run."""
    if mode == "fast":
        mode = "quick"
    lint = [
        ("fmt", ["cargo", "fmt", "--all", "--check"], {}),
        ("clippy-lib", CLIPPY_LIB, {}),
        ("clippy-tests", CLIPPY_TESTS, {}),
        # report-only lines that differ from the lock are a report in quick, a failure otherwise (R-110)
        ("exceptions", tool("exceptions", "--check", *(["--fast"] if mode == "quick" else [])), {}),
        ("check-frozen", tool("check-frozen"), {}),
        ("check-rules", tool("check-rules"), {}),
        ("headers", tool("headers"), {}),
        ("trace", tool("trace"), {}),
        ("ast-grep-scan", ["ast-grep", "scan"], {}),
    ]
    compiled = [
        ("doc", ["cargo", "doc", "--workspace", "--no-deps", "--all-features", "--locked"], {"RUSTDOCFLAGS": "-D warnings"}),
        ("build", ["cargo", "build", "--workspace", "--all-targets", "--locked"], {}),
        # the gungraun instruments compile against today's library, so a bench can't break
        # unseen; compile only, no Valgrind (they run on Linux CI)
        ("gungraun-build", ["cargo", "check", "--locked", "--manifest-path", "tools/gungraun/Cargo.toml",
                            "--benches"], {}),
        ("doctest", ["cargo", "test", "--doc", "--workspace", "--all-features", "--locked"], {}),
        ("fault-injection-probe", fault_injection_probe, {}),
        ("deny", ["cargo", "deny", "--locked", "check", "bans", "licenses", "sources"], {}),
        ("deny-tally", ["cargo", "deny", "--locked", "--manifest-path", "crates/tally/Cargo.toml",
                        "--config", "deny-tally.toml", "check", "bans"], {}),
        ("tree-probe", tree_probe, {}),
        ("ast-grep-test", ["ast-grep", "test", "--skip-snapshot-tests"], {}),
        ("special-casing", tool("special-casing"), {}),
        ("spec-loadcheck-selftest", ["python3", "-I", "-B", "spec/tools/loadcheck.py", "--selftest"], {}),
        ("spec-loadcheck", ["python3", "-I", "-B", "spec/tools/loadcheck.py", *spec_realities()], {}),
    ]
    if mode in ("quick", "commit"):
        quick = [*lint, ("tests", tests, {}), ("milestone-report", milestone_report(["--cargo-profile", "quick"]), {})]
        return quick if mode == "quick" else quick + compiled
    return [
        *lint[:3],
        ("runner", RUNNER, {}),
        ("gating", gating([]), {}),
        ("milestone-report", milestone_report([]), {}),
        *lint[3:],
        *compiled[:3],
        ("test-dev", ["cargo", "nextest", "run", "--workspace", "--all-features", "--locked", "--profile", "ci"], {}),
        ("test-release", ["cargo", "nextest", "run", "--workspace", "--all-features", "--locked", "--profile", "ci",
                          "--cargo-profile", "release-test"], {}),
        ("gating-release", gating(["--profile", "ci", "--cargo-profile", "release-test"]), {}),
        compiled[3],
        ("release-binary", ["cargo", "build", "--release", "--locked", "-p", "tally-cli"], {}),
        *compiled[4:9],
        ("guard-test", walls("guard-test", tool("guard-test")), {}),
        ("rules-selftest", walls("rules-selftest", tool("rules-selftest/run")), {}),
        *compiled[9:],
        ("fuzz", fuzz, {}),
    ]


# --- CI's groups (R-77, R-78; 2026-11-13) ----------------------------------------------

# A pull request's `check full` runs as one job per group, in parallel: every step of `full`
# in exactly one group, and rules-selftest split by rule into the SHARDS and one more
GROUPS = {
    "lint": ["fmt", "clippy-lib", "clippy-tests", "doc", "build", "gungraun-build", "exceptions", "check-frozen",
             "check-rules", "headers", "trace", "ast-grep-scan", "ast-grep-test", "deny", "deny-tally", "tree-probe",
             "special-casing", "spec-loadcheck-selftest", "spec-loadcheck", "guard-test"],
    "dev-tests": ["runner", "gating", "milestone-report", "test-dev", "doctest"],
    "release-tests": ["test-release"],
    "release-gating": ["gating-release"],
    "release-binary": ["release-binary", "fault-injection-probe"],
    "fuzz": ["fuzz"],
}
SHARDED = "rules-selftest"
# the shards beside the last, by rule, balanced on CI's per-case times (the check-speed
# study's exp 2, with R-82's release case cut to one small binary); the last shard takes
# every other rule with a case
SHARDS = [["R-77"], ["R-51", "R-8"], ["R-9", "R-82"]]


def group_names():
    return [*GROUPS, *(f"{SHARDED}-{k}" for k in range(1, len(SHARDS) + 2))]


def rules_with_cases():
    """The rules rules-selftest has a case for, in the rulebook's order."""
    import rules_selftest  # here, not at the top: it imports this module

    return sorted({c.rule for c in rules_selftest.load_cases()}, key=lambda r: int(r[2:]))


def shard_rules(k, with_cases):
    """Shard k's rules (from 1): a named list, or for the last every other rule with a case."""
    if k <= len(SHARDS):
        return list(SHARDS[k - 1])
    named = {r for shard in SHARDS for r in shard}
    return [r for r in with_cases if r not in named]


def group_steps(name):
    """CI's group `name`: the same steps as `full`'s, in its order; a shard runs rules-selftest
    on its rules, kept green under its own name."""
    full = steps("full")
    if name in GROUPS:
        return [s for s in full if s[0] in GROUPS[name]]
    k = int(name.removeprefix(f"{SHARDED}-"))
    rules = shard_rules(k, rules_with_cases())
    if not rules:
        raise ValueError(f"the shard {name} has no rule")
    return [(SHARDED, walls(SHARDED, tool("rules-selftest/run", *rules), kept=name, salt=" ".join(rules)), {})]


def groups_problems():
    """What keeps the groups and shards from covering `full` exactly (R-77, R-78): a step in
    no group, in two, or unknown; a rule with a case in no shard or in two; a shard rule with
    no case; an empty shard."""
    full = [name for name, _, _ in steps("full")]
    held = [s for g in GROUPS.values() for s in g] + [SHARDED]
    problems = [f"the step {s} is in no group" for s in full if s not in held]
    problems += [f"the step {s} is in {held.count(s)} groups" for s in sorted(set(held)) if held.count(s) > 1]
    problems += [f"the group step {s} isn't a step of full" for s in sorted(set(held) - set(full))]
    try:
        with_cases = rules_with_cases()
    except Exception as e:  # noqa: BLE001  cases that can't be read are a red check, whatever the reason
        return problems + [f"rules-selftest's cases can't be read: {e}"]
    shards = [shard_rules(k, with_cases) for k in range(1, len(SHARDS) + 2)]
    sharded = [r for shard in shards for r in shard]
    problems += [f"the rule {r} has a case but is in no rules-selftest shard" for r in with_cases if r not in sharded]
    problems += [f"the rule {r} is in {sharded.count(r)} rules-selftest shards"
                 for r in sorted(set(sharded)) if sharded.count(r) > 1]
    problems += [f"the shard {SHARDED}-{k} names {r}, which has no case"
                 for k, shard in enumerate(shards, 1) for r in shard if r not in with_cases]
    problems += [f"the shard {SHARDED}-{k} has no rule" for k, shard in enumerate(shards, 1) if not shard]
    return problems


def groups_main(matrix):
    """`check groups`: the cover's problems, or each group's steps and shard's rules; with
    `--matrix` those go to stderr and the group names to stdout, CI's job matrix."""
    out = sys.stderr if matrix else sys.stdout
    problems = groups_problems()
    for p in problems:
        print(f"FAIL groups: {p}", file=out)
    if problems:
        print(f"check groups: red, {len(problems)} problem{'' if len(problems) == 1 else 's'}", file=out)
        return 1
    with_cases = rules_with_cases()
    for name in GROUPS:
        print(f"{name}: {' '.join(GROUPS[name])}", file=out)
    for k in range(1, len(SHARDS) + 2):
        print(f"{SHARDED}-{k}: {' '.join(shard_rules(k, with_cases))}", file=out)
    print(f"check groups: green, {len(group_names())} groups hold the {len(steps('full'))} steps of full once each, "
          f"and {len(SHARDS) + 1} of them the {len(with_cases)} rules with a case", file=out)
    if matrix:
        print(json.dumps(group_names()))
    return 0


# --- the memo (R-82, 2026-11-13) --------------------------------------------------------

MEMO = os.path.join(ENGINE, "target", "check-memo")
# the tools whose versions the key holds: every one a step runs
MEMO_VERSIONS = [["rustc", "-vV"], ["cargo", "-V"], ["cargo", "clippy", "-V"], ["rustfmt", "--version"],
                 ["rustdoc", "-V"], ["cargo", "nextest", "--version"], ["cargo", "deny", "--version"],
                 ["ast-grep", "--version"], ["cargo", f"+{NIGHTLY}", "fuzz", "--version"],
                 ["rustc", f"+{NIGHTLY}", "-vV"], ["python3", "--version"], ["git", "--version"], ["cc", "--version"]]
# variables that name the session or the terminal, which no step reads, and the three that
# change how a run builds, never its result (mode_env)
NOT_KEYED = {"_", "PWD", "OLDPWD", "SHLVL", "TERM_SESSION_ID", "TERM_PROGRAM_VERSION", "SECURITYSESSIONID",
             "TMPDIR", "COLUMNS", "LINES", "CLAUDE_CODE_SSE_PORT", "CLAUDECODE", "CLAUDE_CODE_ENTRYPOINT",
             "OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE", "GIT_EDITOR", "SSH_AUTH_SOCK", "LaunchInstanceID",
             "XPC_SERVICE_NAME", "XPC_FLAGS", "__CFBundleIdentifier", "COLORTERM", "TERM", "TERM_PROGRAM",
             "CARGO_INCREMENTAL", "CARGO_BUILD_JOBS", "NEXTEST_TEST_THREADS"}


def git_out(*args):
    r = subprocess.run(["git", "-C", ROOT, *args], capture_output=True, check=False)
    if r.returncode != 0:
        raise OSError(f"git {' '.join(args)}: {r.stderr.decode(errors='replace').strip()}")
    return r.stdout


def memo_key():
    """The tree's key: every file git sees (tracked, or untracked and not ignored) by path,
    executable bit and working-tree bytes; the tags; the steps' environment but NOT_KEYED;
    the platform; the tools' versions. An error reading any of it is an OSError."""
    h = hashlib.sha256(f"check-memo 1\n{sys.platform}\n".encode())
    listed = git_out("ls-files", "-z", "--cached", "--others", "--exclude-standard").decode()
    for rel in sorted({p for p in listed.split("\0") if p}):
        path = os.path.join(ROOT, rel)
        if os.path.islink(path):
            mode, data = "l", os.readlink(path).encode()
        elif os.path.isfile(path):
            mode = "x" if os.stat(path).st_mode & 0o111 else "-"
            with open(path, "rb") as f:
                data = f.read()
        else:
            mode, data = "missing", b""  # tracked, deleted in the working tree
        h.update(f"{rel}\t{mode}\t{hashlib.sha256(data).hexdigest()}\n".encode())
    h.update(git_out("for-each-ref", "--format=%(refname) %(objectname)", "refs/tags"))
    env = clean_env()
    for k in sorted(env):
        if k not in NOT_KEYED:
            h.update(f"env {k}={env[k]}\n".encode())
    for argv in MEMO_VERSIONS:
        try:
            r = subprocess.run(argv, cwd=ENGINE, env=env, capture_output=True, text=True, check=False)
            said = f"{r.returncode}\t{r.stdout}{r.stderr}"
        except OSError as e:
            said = f"not run: {e.strerror}"
        h.update(f"{' '.join(argv)}\t{said}\n".encode())
    return h.hexdigest()


def memo_hit(mode, key):
    """Whether `key` is a kept green one of `mode` or a larger mode."""
    for m in MODES[MODES.index(mode):]:
        try:
            with open(os.path.join(MEMO, m), encoding="ascii") as f:
                if f.read().strip() == key:
                    return True
        except OSError:
            continue
    return False


def memo_keep(mode, key):
    os.makedirs(MEMO, exist_ok=True)
    with open(os.path.join(MEMO, mode), "w", encoding="ascii") as f:
        f.write(key + "\n")


def memo_used():
    """The memo is local: never on CI, never with --fresh."""
    return not FRESH and os.environ.get("GITHUB_ACTIONS") != "true" and not os.environ.get("CI")


def tests(env, log):
    """quick's test step (R-82, R-77; 2026-11-13): every test target of the workspace and every
    gating milestone's, in one nextest run in the quick profile."""
    try:
        gates, names = milestones.gating_targets(env)
    except milestones.MilestoneError as e:
        return str(e)
    code, out = run([*QUICK_TESTS, *(f"--test={t}" for t in names)], env, log)
    if code == 0:
        return None
    which = f"the workspace's and the gating {', '.join(gates)}'s" if gates else "the workspace's"
    counted = milestones.tally(out)
    if counted is None:
        return f"the tests ({which}) don't run: {first_error(out)}"
    total, passed = counted
    return f"{total - passed} of {total} tests are red ({which})"


def run(argv, env, log, cwd=ENGINE):
    r = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, check=False)
    log.write(f"$ {' '.join(argv)}\n{r.stdout}\n[exit {r.returncode}]\n")
    return r.returncode, r.stdout


def gating(profile):
    """R-77: every gating milestone's tests (tools/milestones.toml) fail the check like the
    default targets, beside them; while none gates, the marker and the targets' names are
    still checked."""

    def step(env, log):
        try:
            gates, names = milestones.gating_targets(env)
        except milestones.MilestoneError as e:
            return str(e)
        if not names:
            log.write(f"no gating milestone has a test target (gating = {gates})\n")
            return None
        code, out = run([*RUNNER, *profile, *(f"--test={t}" for t in names)], env, log)
        if code == 0:
            return None
        who = f"{', '.join(gates)} gate{'s' if len(gates) == 1 else ''}"
        their = "its" if len(gates) == 1 else "their"
        counted = milestones.tally(out)
        if counted is None:
            return f"{who}, and {their} tests don't run: {first_error(out)}"
        total, passed = counted
        return f"{who}, and {total - passed} of {their} {total} tests are red"

    return step


# --- the walls' self-tests' inputs (issue #6) -------------------------------------------

# guard-test copies these into a throwaway repository and reads settings.json's hooks
GUARD_TEST_FILES = ["engine/tools/guard-test", "engine/tools/guard_test.py", "engine/tools/guard", "engine/tools/guard.py",
                    "engine/tools/stop-check", "engine/tools/stop_check.py", "engine/tools/merge-pr",
                    "engine/tools/merge_pr.py", "engine/tools/worktree-done", "engine/tools/worktree_done.py",
                    ".claude/settings.json", "engine/tools/check", "engine/tools/check.py",
                    # the guard runs commit-check before a push (R-100, issue #108)
                    "engine/tools/commit-check", "engine/tools/commit_check.py", "engine/tools/check_frozen.py",
                    # merge-pr reads CI's groups from check (R-77, 2026-11-13)
                    "engine/tools/milestones.py"]
GUARD_TEST_VERSIONS = [["python3", "--version"], ["git", "--version"]]
# rules-selftest builds, lints and plants into a copy of the engine with these tools
SELFTEST_VERSIONS = [*GUARD_TEST_VERSIONS, ["rustc", "-vV"], ["cargo", "-V"], ["cargo", "clippy", "-V"],
                     ["rustfmt", "--version"], ["rustdoc", "-V"], ["cargo", "deny", "--version"],
                     ["cargo", "nextest", "--version"], ["ast-grep", "--version"]]
# beyond the rule files: the manifests and lock, the module list the ban coverage plants
# into, the spec's and the oracle's checksums (the copy's pin), and the root package.json
# the copy holds; the cases' own targets are added from the case files
SELFTEST_FILES = ["engine/Cargo.lock", "engine/Cargo.toml", "engine/crates/tally/Cargo.toml",
                  "engine/crates/tally-cli/Cargo.toml", "engine/tests/runner/Cargo.toml",
                  "engine/crates/tally/src/lib.rs", "engine/spec/VERSION", "engine/spec/SHA256SUMS",
                  "engine/tools/SHA256SUMS", "package.json"]
PLANTS = re.compile(r"^--- (?:replace|append|delete|write) (\S+)", re.M)


def case_targets():
    """The real files the selftest's cases plant into, repo-relative."""
    cases = os.path.join(TOOLS, "rules-selftest", "cases")
    found = set()
    for name in sorted(os.listdir(cases)):
        with open(os.path.join(cases, name), encoding="utf-8") as f:
            for rel in PLANTS.findall(f.read()):
                path = os.path.normpath(os.path.join(ENGINE, rel))
                if os.path.isfile(path):
                    found.add(os.path.relpath(path, os.path.dirname(ENGINE)))
    return found


def walls_inputs(step):
    """(files, version commands) that decide `step`'s verdict, beyond the engine code
    check full's own steps check in the same run."""
    if step == "guard-test":
        return sorted(GUARD_TEST_FILES), GUARD_TEST_VERSIONS
    return sorted(set(check_frozen.rule_files()) | set(SELFTEST_FILES) | case_targets()), SELFTEST_VERSIONS


def walls_hash(step, env, salt=""):
    """The hash of `step`'s inputs; `salt` (a shard's rules) is hashed with them."""
    files, versions = walls_inputs(step)
    h = hashlib.sha256(f"{step}\n{sys.platform}\n".encode())
    if salt:
        h.update(f"salt {salt}\n".encode())
    root = os.path.dirname(ENGINE)
    for rel in files:
        try:
            with open(os.path.join(root, rel), "rb") as f:
                h.update(f"{rel}\t{hashlib.sha256(f.read()).hexdigest()}\n".encode())
        except OSError:
            h.update(f"{rel}\tmissing\n".encode())
    for argv in versions:
        r = subprocess.run(argv, cwd=ENGINE, env=env, capture_output=True, text=True, check=False)
        h.update(f"{' '.join(argv)}\t{r.returncode}\t{r.stdout}{r.stderr}\n".encode())
    return h.hexdigest(), len(files), len(versions)


def walls(name, argv, kept=None, salt=""):
    """A wall's self-test, skipped when its inputs are those of a green run of it; a shard's
    green is kept under its group's name (`kept`), its rules hashed with the inputs."""

    def step(env, log):
        key, files, versions = walls_hash(name, env, salt)
        green = os.path.join(WALLS_GREEN, kept or name)
        try:
            with open(green, encoding="ascii") as f:
                last = f.read().strip()
        except OSError:
            last = ""
        log.write(f"inputs: sha256 {key} ({files} files, {versions} tool versions); last green: {last or 'none'}\n")
        if not FRESH and last == key:
            return Report(f"skipped: its {files} input files and {versions} tool versions (sha256 {key[:16]}) are "
                          f"those of a green run; `check full --fresh` runs it")
        code, out = run(argv, env, log)
        if code != 0:
            return first_error(out)
        os.makedirs(WALLS_GREEN, exist_ok=True)
        with open(green, "w", encoding="ascii") as f:
            f.write(key + "\n")
        return None

    return step


def milestone_report(profile):
    """The next milestone's tests, the builder's work list: a line such as `S: 3/7 green`,
    never a failure (issue #14); in quick's profile there (R-82, 2026-11-13)."""

    def step(env, log):
        return next_report(profile, env, log)

    return step


def next_report(profile, env, log):
    try:
        m, names = milestones.next_milestone(env)
    except milestones.MilestoneError as e:
        return Report(f"no report: {e}")
    if m is None:
        return Report("every milestone gates")
    if not names:
        return Report(f"{m}: no test target (gates once {m} is tagged)")
    _, out = run([*RUNNER, *profile, *(f"--test={t}" for t in names)], env, log)
    counted = milestones.tally(out)
    if counted is None:
        return Report(f"{m}: its tests don't run: {first_error(out)} (gates once {m} is tagged)")
    total, passed = counted
    return Report(f"{m}: {passed}/{total} green (gates once {m} is tagged)")


def fault_injection_probe(env, log):
    """R-51: no test-only feature (fault injection, the counters, the reversed order)
    can reach a release build, and the CLI never turns one on."""
    import check_rules  # the list of test-only features and which tally declares

    for feature in check_rules.declared_test_only():
        code, out = run(["cargo", "build", "--release", "--locked", "-p", "tally", "--features", feature], env, log)
        if code == 0 or feature not in out or "debug assertions" not in out:
            return f"a release build with the {feature} feature compiled; the compile_error! guard is missing"
    code, out = run(["cargo", "tree", "--locked", "-p", "tally-cli", "-e", "features"], env, log)
    if code != 0:
        return "cargo tree failed"
    for feature in check_rules.TEST_ONLY:
        if f'feature "{feature}"' in out:
            return f"tally-cli turns on tally's {feature} feature"
    return None


def tree_probe(env, log):
    """R-36: no randomness in what tally ships."""
    code, out = run(["cargo", "tree", "--locked", "-p", "tally", "-e", "normal,build", "--prefix", "none"], env, log)
    if code != 0:
        return "cargo tree failed"
    hits = sorted({line.split()[0] for line in out.splitlines() if re.match(r"(rand|rand_core|getrandom|fastrand) ", line)})
    return f"tally depends on {', '.join(hits)}" if hits else None


def fuzz(env, log):
    """Each fuzz target for FUZZ_SECONDS, its regressions replayed first (gate 9's PR part); its
    corpus seeded with M0's texts (fuzz/seeds.py), the dictionary of L §2.2-§2.3's tokens, and
    an input that runs FUZZ_TIMEOUT seconds a finding, a hang (R-91). The corpus is fresh on
    every run, the seeds alone in target/fuzz-corpus/<target>/, as on CI: fuzz/corpus/, which
    long runs grow, is never read, so its size can't exhaust libFuzzer's memory (issue #78)."""
    fuzz_dir = os.path.join(ENGINE, "fuzz")
    dictionary = os.path.join(fuzz_dir, "tally.dict")
    code, out = run(["cargo", f"+{NIGHTLY}", "fuzz", "list"], env, log, cwd=ENGINE)
    targets = out.split() if code == 0 else []
    if not targets:
        return "cargo fuzz list found no target"
    for target in targets:
        regressions = os.path.join(fuzz_dir, "regressions", target)
        cases = sorted(
            os.path.join(regressions, f) for f in os.listdir(regressions) if not f.startswith(".") and f != "README.md"
        ) if os.path.isdir(regressions) else []
        if not os.path.isdir(regressions):
            return f"fuzz/regressions/{target}/ is missing"
        if cases:
            code, _ = run(["cargo", f"+{NIGHTLY}", "fuzz", "run", target, *cases], env, log)
            if code != 0:
                return f"{target}: a saved regression fails again"
        corpus = os.path.join(ENGINE, "target", "fuzz-corpus", target)
        shutil.rmtree(corpus, ignore_errors=True)
        code, _ = run(["python3", "-I", "-B", os.path.join(fuzz_dir, "seeds.py"), target, "--into", corpus], env, log)
        if code != 0:
            return f"{target}: fuzz/seeds.py couldn't seed its corpus"
        code, _ = run(["cargo", f"+{NIGHTLY}", "fuzz", "run", target, corpus, regressions, "--",
                       f"-max_total_time={FUZZ_SECONDS}", f"-timeout={FUZZ_TIMEOUT}", f"-dict={dictionary}"], env, log)
        if code != 0:
            return f"{target}: the fuzzer found a crash, a hang or a failed assertion (see fuzz/artifacts/{target}/)"
    return None


def first_error(out):
    for line in out.splitlines():
        if re.match(r"(error(\[\w+\])?:|FAIL|ERROR|panicked|thread '.*' panicked)", line.strip()):
            return line.strip()
    lines = [l.strip() for l in out.splitlines() if l.strip()]
    return lines[-1] if lines else "(no output)"


USAGE = "usage: engine/tools/check quick|commit|full|group <name>|groups [--matrix]|list|step <name>|walls-key [--fresh]"


def choose(args):
    """(mode, steps) for the arguments, or None when they aren't a known request."""
    if args in (["fast"], ["quick"], ["commit"], ["full"]):
        mode = "quick" if args[0] == "fast" else args[0]
        return mode, steps(mode)
    if len(args) == 2 and args[0] == "step" and any(s[0] == args[1] for s in steps("full")):
        return f"step {args[1]}", [s for s in steps("full") if s[0] == args[1]]
    if len(args) == 2 and args[0] == "group" and args[1] in group_names():
        return f"group {args[1]}", group_steps(args[1])
    return None


def main():
    global FRESH  # one flag for the whole run
    args = sys.argv[1:]
    if "--fresh" in args:
        FRESH = True
        args = [a for a in args if a != "--fresh"]
    if args == ["walls-key"]:
        env = clean_env()
        keys = "".join(walls_hash(s, env)[0] for s in ("guard-test", "rules-selftest"))
        print(hashlib.sha256(keys.encode()).hexdigest())
        return 0
    if args in (["groups"], ["groups", "--matrix"]):
        return groups_main(args[-1] == "--matrix")
    if args == ["list"]:
        for m in MODES:
            print(f"{m}: {' '.join(name for name, _, _ in steps(m))}")
        for g in group_names():
            print(f"group {g}: {' '.join(name for name, _, _ in group_steps(g))}")
        return 0
    try:
        chosen = choose(args)
    except ValueError as e:
        print(f"FAIL setup: {e}")
        return 1
    if chosen is None:
        print(USAGE, file=sys.stderr)
        return 64
    mode, chosen = chosen
    for needed in ("cargo", "python3", "git"):
        if shutil.which(needed) is None:
            print(f"FAIL setup: {needed} is not on PATH")
            return 1
    key = None
    if mode in MODES and memo_used():
        try:
            key = memo_key()
        except OSError as e:
            print(f"check {mode}: no memo, the tree's key can't be read ({e})")
        if key is not None and memo_hit(mode, key):
            print(f"check {mode}: green on this tree already (sha256 {key[:16]})")
            return 0
    os.makedirs(LOGS, exist_ok=True)
    env = mode_env(mode)
    failed = []
    began = time.monotonic()
    for name, what, extra in chosen:
        path = os.path.join(LOGS, f"{name}.log")
        with open(path, "w", encoding="utf-8") as log:
            step_env = {**env, **extra}
            if callable(what):
                problem = what(step_env, log)
                if isinstance(problem, Report):
                    print(f"{name}: {problem}")
                    problem = None
            else:
                code, out = run(what, step_env, log)
                problem = None if code == 0 else first_error(out)
                if problem is None and name.startswith("clippy"):
                    noisy = [p.pattern for p in SILENT if p.search(out)]
                    if noisy:
                        problem = f"clippy said \"{noisy[0]}\": a rule silently does nothing"
        if problem:
            failed.append(name)
            print(f"FAIL {name}: {problem}  (target/check-logs/{name}.log)")
    total = len(chosen)
    took = time.monotonic() - began
    if failed:
        print(f"check {mode}: red, {len(failed)} of {total} steps failed ({took:.0f}s)")
        return 1
    if key is not None:
        try:
            after = memo_key()
        except OSError:
            after = None
        if after == key:
            memo_keep(mode, key)
        else:
            print(f"check {mode}: the tree changed during the run, so this green isn't remembered")
    print(f"check {mode}: green, {total} step{'' if total == 1 else 's'} ({took:.0f}s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
