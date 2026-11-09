"""mutants-local: CI's mutation run on a few kernel files, for T's loop (issue #40).

    engine/tools/mutants-local [--next] [--fences] FILE… [-F REGEX]

FILE is a kernel file (tools/kernel.toml, .cargo/mutants.toml's examine_globs), relative to
engine/ or to the repository. -F keeps the mutants whose name matches REGEX (cargo-mutants
27.1.0 keeps a file's struct-field deletions whatever -F says). It runs what CI's mutants
jobs run, with the same flags (R-84, R-88): T's tests unmutated first, the default
targets and every gating milestone's but the marker's mutants_leave_out (tools/milestones
args); then cargo-mutants with
.cargo/mutants.toml, its kernel list narrowed to FILE, the gating milestones' targets, a
fixed proptest seed, no failure files, the baseline skipped, `--timeout 300
--build-timeout 600`, and each test under the memory cap, each mutant ended at its first
failing test (nextest's `mutants` profile, tools/test-memcap). The output is engine/target/mutants-local/mutants.out/; the exit is
cargo-mutants' (0 all caught, 2 some missed, 3 some timed out).

--next (R-88, issue #108; the M1 coordinator's pick P13) adds the next milestone's test
targets (tools/milestones: M1's `m1_…` while M1 builds), each test only if it passes in an
unmutated run of those targets just before the mutants, the batch's baseline (the mutants
profile, no fail-fast, the features their targets require). A test red there would count
as catching every mutant (pre-M1's P13), so it is left out, and listed; the rest are
kept through a nextest filter (`-E`) on the mutants' test runs. Without --next the run is
CI's, the next milestone's tests left out.

--fences (R-88, R-130; pick P55, 2026-11-09) adds the fences' equivalence tests that
tools/milestones.toml's `fence_catchers` lists (F5's: `tally-cli`'s `walk_oracle`, `tally`'s
`walk::lows::tests`), once they pass unmutated: when a fence answers T's reads, only its
equivalence test reaches its plain twin. They are the builder's tests, so a local batch only:
CI's shards and gate 8 count T's tests (R-84).

The environment is check's (R-82); only CARGO_BUILD_JOBS and NEXTEST_TEST_THREADS pass
through, which change how many jobs run at once, never a result.
"""

import fnmatch
import os
import re
import subprocess
import sys
import tomllib

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
sys.path.insert(0, TOOLS)
import check  # noqa: E402  the clean environment and the fixed seeds (R-82, R-88)
import milestones  # noqa: E402  the next milestone's targets (--next)

CONFIG = os.path.join(ENGINE, ".cargo", "mutants.toml")
OUT = os.path.join(ENGINE, "target", "mutants-local")
PASSED = ("CARGO_BUILD_JOBS", "NEXTEST_TEST_THREADS")
# CI's flags (.github/workflows/engine.yml's mutants jobs), the shard aside
FLAGS = ["--no-shuffle", "--baseline", "skip", "--timeout", "300", "--build-timeout", "600"]


# nextest's line for a finished test: `FAIL [   0.009s] (1/3) tally-runner::m1_format tests::name`
STATUS = re.compile(r"^\s*([A-Z][A-Z-]*)\s+\[\s*[\d.]+s\]\s+(?:\(\s*\d+/\d+\)\s+)?(\S+)\s+(\S+)\s*$")
GREEN = ("PASS", "LEAK")


def baseline_split(out):
    """(green, red) of an unmutated nextest run's output: each test as (binary id, name)."""
    passed = {}
    for line in out.splitlines():
        m = STATUS.match(line)
        if m:
            passed[(m.group(2), m.group(3))] = m.group(1) in GREEN
    return sorted(t for t, ok in passed.items() if ok), sorted(t for t, ok in passed.items() if not ok)


def next_filter(out, names):
    """The nextest filter for the mutants' test runs (--next, R-88): every test of the other
    targets, and of the next milestone's targets `names` only those green in `out`, the
    unmutated run; with the green and the red. None when the run's summary doesn't match
    its lines (it didn't run, or nextest's form changed): nothing can be told green."""
    green, red = baseline_split(out)
    counted = milestones.tally(out)
    if counted is None or counted[0] != len(green) + len(red):
        return None, green, red
    others = " | ".join(f"binary_id(={milestones.RUNNER}::{n})" for n in names)
    kept = "".join(f" | (binary_id(={b}) & test(={t}))" for b, t in green)
    return f"not ({others}){kept}", green, red


def toml_value(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, str):
        return '"' + v.replace("\\", "\\\\").replace('"', '\\"') + '"'
    if isinstance(v, list):
        return "[" + ", ".join(toml_value(x) for x in v) + "]"
    raise ValueError(f"mutants-local: .cargo/mutants.toml holds a value it can't copy: {v!r}")


def narrowed(config, files):
    """.cargo/mutants.toml with its examine_globs narrowed to `files`."""
    lines = [f"# written by tools/mutants-local from .cargo/mutants.toml; examine_globs narrowed\n"]
    for key, value in config.items():
        lines.append(f"{key} = {toml_value(files if key == 'examine_globs' else value)}\n")
    return "".join(lines)


def kernel_file(arg, globs):
    rel = arg.removeprefix("engine/")
    if not os.path.isfile(os.path.join(ENGINE, rel)):
        return None, f"{arg}: no such file under engine/"
    if not any(fnmatch.fnmatchcase(rel, g) for g in globs):
        return None, f"{arg}: not a kernel file (.cargo/mutants.toml's examine_globs)"
    return rel, None


def main(argv):
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__.strip())
        return 0 if argv else 64
    regex, nxt, fences = None, "--next" in argv, "--fences" in argv
    argv = [a for a in argv if a not in ("--next", "--fences")]
    if "-F" in argv:
        at = argv.index("-F")
        if at + 1 >= len(argv):
            print("mutants-local: -F needs a regex", file=sys.stderr)
            return 64
        regex = argv[at + 1]
        argv = argv[:at] + argv[at + 2:]
    with open(CONFIG, "rb") as f:
        config = tomllib.load(f)
    files, problems = [], []
    for arg in argv:
        rel, problem = kernel_file(arg, config.get("examine_globs", []))
        files.append(rel)
        if problem:
            problems.append(problem)
    if problems or not files:
        for p in problems or ["name at least one kernel file"]:
            print(f"mutants-local: {p}", file=sys.stderr)
        return 64
    env = check.clean_env()
    env.update({k: os.environ[k] for k in PASSED if k in os.environ})
    args = subprocess.run([os.path.join(TOOLS, "milestones"), "args"], cwd=ENGINE, env=env, capture_output=True,
                          text=True, check=True).stdout.split()
    targets = subprocess.run([os.path.join(TOOLS, "milestones"), "args", "--mutants"], cwd=ENGINE, env=env,
                             capture_output=True, text=True, check=True).stdout.split()
    print("mutants-local: T's tests, unmutated", flush=True)
    base = subprocess.run(["cargo", "nextest", "run", "--locked", "-p", "tally-runner", "--cargo-profile", "mutants",
                           "--profile", "mutants", *args], cwd=ENGINE, env=env, check=False)
    if base.returncode != 0:
        print("mutants-local: T's tests fail unmutated; no mutant is run", file=sys.stderr)
        return 1
    # one nextest filter over the mutants' test runs: the runner's tests (those of the next
    # milestone green at the baseline, with --next), then the fences' (--fences)
    runner_filter, fence_filter = None, None
    if nxt:
        added = next_targets(env)
        if added is None:
            return 1
        selected, runner_filter = added
        targets += [f"-C={a}" for a in selected]
    if fences:
        added = fence_targets(env)
        if added is None:
            return 1
        selected, fence_filter = added
        # `--lib` is the runner's already, and cargo takes it once for every package selected
        targets += [f"-C={a}" for a in selected if f"-C={a}" not in targets]
    if fence_filter is not None:
        runner = f"package({milestones.RUNNER})" + (f" & ({runner_filter})" if runner_filter else "")
        targets += ["--cargo-test-arg=-E", f"--cargo-test-arg={runner} | {fence_filter}"]
    elif runner_filter is not None:
        targets += ["--cargo-test-arg=-E", f"--cargo-test-arg={runner_filter}"]
    os.makedirs(OUT, exist_ok=True)
    local = os.path.join(OUT, "mutants.toml")
    with open(local, "w", encoding="utf-8") as f:
        f.write(narrowed(config, files))
    argv = ["cargo", "mutants", "--config", local, "-C=--locked", *targets, *FLAGS, "--output", OUT]
    if regex is not None:
        argv += ["-F", regex]
    print(f"mutants-local: {' '.join(argv[2:])}", flush=True)
    return subprocess.run(argv, cwd=ENGINE, env=env, check=False).returncode


def next_targets(env):
    """--next: the cargo arguments that add the next milestone's targets, and the nextest
    filter that keeps of them the tests green at the batch's baseline (none when no test is:
    the targets are then left out); None when they can't be told (R-88)."""
    try:
        m, names = milestones.next_milestone(env)
        features = milestones.required_features(env, names) if names else []
    except milestones.MilestoneError as e:
        print(f"mutants-local: --next: {e}", file=sys.stderr)
        return None
    if not names:
        print(f"mutants-local: --next: {m or 'no milestone'} has no test target to add", file=sys.stderr)
        return None
    selected = [*(f"--features={f}" for f in features), *(f"--test={n}" for n in names)]
    print(f"mutants-local: {m}'s tests, unmutated (the batch's baseline)", flush=True)
    run = subprocess.run(["cargo", "nextest", "run", "--locked", "-p", "tally-runner", "--cargo-profile", "mutants",
                          "--profile", "mutants", "--no-fail-fast", *selected], cwd=ENGINE, env=env,
                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, check=False)
    os.makedirs(check.LOGS, exist_ok=True)
    with open(os.path.join(check.LOGS, "mutants-local-next.log"), "w", encoding="utf-8") as f:
        f.write(run.stdout)
    expr, green, red = next_filter(run.stdout, names)
    if expr is None:
        print(f"mutants-local: --next: {m}'s tests didn't run, so none can be told green "
              "(target/check-logs/mutants-local-next.log)", file=sys.stderr)
        return None
    print(f"mutants-local: {m}: {len(green)} of {len(green) + len(red)} tests green at the baseline, kept as catchers")
    for b, t in red:
        print(f"mutants-local: left out, red at the baseline (R-88): {b} {t}")
    if not green:
        return [], None
    return selected, expr


def fence_filter_of(catchers):
    """The nextest filter that selects the fences' tests: each test target whole, each lib
    module's tests by their path (R-88, R-130)."""
    parts = []
    for package, test, lib in catchers:
        if test is not None:
            parts.append(f"binary_id(={package}::{test})")
        else:
            parts.append(f"(binary_id(={package}) & test(/^{lib}/))")
    return " | ".join(parts)


def fence_targets(env):
    """--fences (pick P55, 2026-11-09): the cargo arguments that add the fences' equivalence
    tests (tools/milestones.toml's `fence_catchers`) as catchers, and their nextest filter,
    once they pass unmutated; None when they can't be read or fail there. The builder's own
    tests: advisory, never gate 8's (R-84)."""
    try:
        catchers = milestones.fence_catchers()
    except milestones.MilestoneError as e:
        print(f"mutants-local: --fences: {e}", file=sys.stderr)
        return None
    if not catchers:
        print("mutants-local: --fences: tools/milestones.toml lists no fence catcher", file=sys.stderr)
        return None
    packages = sorted({p for p, _, _ in catchers})
    selected = [*(f"--package={p}" for p in packages), *(f"--test={t}" for _, t, _ in catchers if t is not None)]
    if any(lib is not None for _, _, lib in catchers):
        selected.append("--lib")
    expr = fence_filter_of(catchers)
    print(f"mutants-local: the fences' tests, unmutated ({expr})", flush=True)
    run = subprocess.run(["cargo", "nextest", "run", "--locked", "--cargo-profile", "mutants", "--profile", "mutants",
                          *selected, "-E", expr], cwd=ENGINE, env=env, check=False)
    if run.returncode != 0:
        print("mutants-local: --fences: the fences' tests fail unmutated; no mutant is run", file=sys.stderr)
        return None
    print(f"mutants-local: kept as catchers, the builder's, not counted at gate 8 (R-84): {expr}")
    return selected, expr


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
