"""mutants-local: gate 8's mutation run on a kernel file, for a single-mutant proof (issue #40).

    engine/tools/mutants-local [--next] [--fences] FILE… [-F REGEX]

Locally it is for a single-mutant proof of a few minutes, `-F` naming one mutant; a batch
of mutants goes to a gate-8 dispatch on GitHub (the user, 2026-11-12: "Why on earth are we
running this locally instead of on github?"). T's baseline and the coverage map are paid
once per tree: a second proof on the same tree, with the same targets, reuses them (R-88,
2026-11-13; tools/mutants-gate's local mode).

FILE is a kernel file (tools/kernel.toml, .cargo/mutants.toml's examine_globs), relative to
engine/ or to the repository. -F keeps the mutants whose name matches REGEX, a struct
field's deletion included. It is tools/mutants-gate's local mode, CI's method on this
machine (R-84, R-88, R-133; 2026-11-12): T's tests unmutated first (the default targets
and every gating milestone's but the marker's mutants_leave_out, tools/milestones args; a
test over the 20 s CI's baseline allows is a notice here); the mutants of FILE; a fresh
coverage map; then each mutant built with the targets of the T tests that enter its
function and run on those tests, under `--timeout 600 --build-timeout 600`, a fixed
proptest seed, no failure files and the memory cap, ended at its first failing test
(nextest's `mutants` profile, tools/test-memcap). A mutant no test enters is built and, if
viable, missed. The output is engine/target/mutants-local/mutants.out/; the exit is
cargo-mutants' (0 all caught, 2 some missed, 3 some timed out).

--next (R-88, issue #108; the M1 coordinator's pick P13) adds the next milestone's test
targets (tools/milestones: the first milestone not gating, M1's `m1_…` while M1 built; M2's,
once it has one, since M1 gates), each test only if it passes in an
unmutated run of those targets just before the mutants, the batch's baseline (the mutants
profile, no fail-fast, the features their targets require). A test red there would count
as catching every mutant (pre-M1's P13), so it is left out, and listed; the rest are
kept through a nextest filter (`-E`) on the baseline and the coverage run, so the map, and
with it each mutant's tests, holds only them. Without --next the run is CI's, the next
milestone's tests left out, the gating ones' required features on (R2).

--fences (R-88, R-130; pick P55, 2026-11-09) adds the fences' equivalence tests that
tools/milestones.toml's `fence_catchers` lists (F5's: `tally-cli`'s `walk_oracle`, `tally`'s
`walk::lows::tests`), once they pass unmutated: when a fence answers T's reads, only its
equivalence test reaches its plain twin. They are the builder's tests, so a local batch only:
CI's shards and gate 8 count T's tests (R-84).

The environment is check's (R-82); only CARGO_BUILD_JOBS and NEXTEST_TEST_THREADS pass
through, which change how many jobs run at once, never a result.
"""

import os
import re
import subprocess
import sys

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
sys.path.insert(0, TOOLS)
import check  # noqa: E402  the clean environment and the fixed seeds (R-82, R-88)
import milestones  # noqa: E402  the next milestone's targets (--next)
import mutants_gate  # noqa: E402  gate 8's method, its local mode (R-88)


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
    files, problems = mutants_gate.kernel_files(argv)
    if problems or not files:
        for p in problems or ["name at least one kernel file"]:
            print(f"mutants-local: {p}", file=sys.stderr)
        return 64
    env = mutants_gate.run_env()
    # the targets added to T's (with --next, --fences) and one nextest filter over the
    # baseline and the coverage run: the runner's tests (those of the next milestone green at
    # the batch's baseline, with --next), then the fences' (--fences)
    extra, runner_filter, fence_filter = [], None, None
    if nxt:
        added = next_targets(env)
        if added is None:
            return 1
        selected, runner_filter = added
        extra += selected
    if fences:
        added = fence_targets(env)
        if added is None:
            return 1
        selected, fence_filter = added
        # `--lib` is the runner's already, and cargo takes it once for every package selected
        extra += [a for a in selected if a not in extra and a != "--lib"]
    expr = runner_filter
    if fence_filter is not None:
        runner = f"package({milestones.RUNNER})" + (f" & ({runner_filter})" if runner_filter else "")
        expr = f"{runner} | {fence_filter}"
    try:
        return mutants_gate.local(files, regex, extra, expr)
    except mutants_gate.GateError as e:
        print(f"FAIL mutants-local: {e}", file=sys.stderr)
        return 1


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
