"""milestones: which milestones' tests gate (R-77, R-84; engine/tests/MILESTONE-S.md,
"Keeping master green"; issue #14).

    engine/tools/milestones                  each milestone: gating, next or later, and its
                                             Rust test targets
    engine/tools/milestones args             the runner's targets a mutation run selects, as
                                             nextest arguments: its default ones and every
                                             gating milestone's, but those the marker's
                                             `mutants_leave_out` names (the mutants baseline,
                                             mutants-local)
    engine/tools/milestones args --mutants   the same, each as cargo-mutants' `-C=` (the
                                             mutants shards)

T's tests are written before the engine they test, so they fail until their milestone
is built. Each milestone's Rust tests are a `tally-runner` test target named for it
(`s_properties`, then `m0_…`, `m5a_…`) with `test = false`, so a plain run leaves them
out. The marker, tools/milestones.toml, lists the milestones in build order and the
gating ones, always the first few: the rules role adds a milestone in the commit right
after the gate session tags it.

`check` runs the gating milestones' targets as failing steps (`gating`, and
`gating-release` in `check full`) and the next milestone's as a report that never
fails (`milestone-report`, a line such as `S: 3/7 green`). A runner test target with
`test = false` whose name starts with no milestone fails: it would run nowhere.

A mutation run (R-84, R-88; the user, 2026-11-06) selects the same targets but those the
marker's `mutants_leave_out` names: they are neither built nor run per mutant, and still
run in `check` and CI's test jobs. Each name there must be a gating milestone's target.
"""

import json
import os
import re
import subprocess
import sys
import tomllib

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
MARKER = os.path.join(TOOLS, "milestones.toml")
RUNNER = "tally-runner"
# the kinds of target cargo selects with --lib
LIB_KINDS = {"lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"}
# nextest's last line: "Summary [   0.012s] 7 tests run: 0 passed, 7 failed, 0 skipped", or
# "7/11 tests run: …" when a fail-fast run stopped early
SUMMARY = re.compile(r"Summary \[[^\]]*\] (?:\d+/)?(\d+) tests? run: (\d+) passed")


class MilestoneError(Exception):
    pass


def marker():
    """The marker's milestones in build order and the gating ones, checked."""
    try:
        with open(MARKER, "rb") as f:
            data = tomllib.load(f)
    except (OSError, tomllib.TOMLDecodeError) as e:
        raise MilestoneError(f"tools/milestones.toml can't be read: {e}") from e
    # `fence_catchers` is `mutants-local --fences`'s alone (fence_catchers()), and
    # `mutants_slow_tests` gate 8's 20 s check's (slow_tests()), so this reader doesn't ask
    # for them
    if sorted(set(data) - {"fence_catchers", "mutants_slow_tests"}) != ["gating", "milestones", "mutants_leave_out"]:
        raise MilestoneError("tools/milestones.toml holds exactly `milestones`, `gating` and `mutants_leave_out`, "
                             "and `mutants-local --fences`'s `fence_catchers`")
    milestones, gating, leave_out = data["milestones"], data["gating"], data["mutants_leave_out"]
    if not isinstance(leave_out, list) or not all(isinstance(t, str) for t in leave_out):
        raise MilestoneError("tools/milestones.toml: mutants_leave_out is a list of test target names")
    if len(set(leave_out)) != len(leave_out):
        raise MilestoneError("tools/milestones.toml: mutants_leave_out names a target twice")
    for key, value in (("milestones", milestones), ("gating", gating)):
        if not isinstance(value, list) or not all(isinstance(m, str) and re.fullmatch(r"[A-Z][0-9a-z]*", m) for m in value):
            raise MilestoneError(f"tools/milestones.toml: {key} is a list of milestone names such as \"S\" and \"M5a\"")
        if len({m.casefold() for m in value}) != len(value):
            raise MilestoneError(f"tools/milestones.toml: {key} names a milestone twice")
    if not milestones:
        raise MilestoneError("tools/milestones.toml: milestones is empty")
    for m in gating:
        if m not in milestones:
            raise MilestoneError(f"tools/milestones.toml: gating names {m}, which isn't a milestone")
    if gating != milestones[: len(gating)]:
        raise MilestoneError("tools/milestones.toml: gating must be the first milestones in build order, "
                             f"each added when its gate session tags it, not {', '.join(gating)}")
    return milestones, gating, leave_out


def fence_catchers():
    """The fences' equivalence tests that `mutants-local --fences` adds as catchers (R-88,
    R-130; pick P55, 2026-11-09), as (package, test target or None, lib module prefix or
    None), checked: each names a package and exactly one of a test target (every test in it)
    and a module of the package's lib (`walk::lows::tests::`, its tests). Not CI's: gate 8
    counts T's tests only (R-84)."""
    try:
        with open(MARKER, "rb") as f:
            listed = tomllib.load(f).get("fence_catchers")
    except (OSError, tomllib.TOMLDecodeError) as e:
        raise MilestoneError(f"tools/milestones.toml can't be read: {e}") from e
    if not isinstance(listed, list) or not listed:
        raise MilestoneError("tools/milestones.toml: fence_catchers, `mutants-local --fences`'s, is a "
                             "non-empty list of tables")
    out = []
    for c in listed:
        ok = isinstance(c, dict) and isinstance(c.get("package"), str) and sorted(c) in (["package", "test"], ["lib", "package"])
        name = c.get("test", c.get("lib")) if isinstance(c, dict) else None
        if not ok or not isinstance(name, str) or not re.fullmatch(r"[A-Za-z0-9_-]+|([a-z0-9_]+::)+", name):
            raise MilestoneError("tools/milestones.toml: each fence_catchers entry is { package, test } or "
                                 f"{{ package, lib = \"<module>::\" }}, not {c!r}")
        out.append((c["package"], c.get("test"), c.get("lib")))
    return out


def slow_tests():
    """The T tests gate 8's 20 s check names, each a single vector T can't split (R-88; pick
    P73, issue #108, 2026-11-12): {test: (seconds, ceiling)}, its time and its ceiling on the
    median runner, checked: each entry is exactly { test, seconds, ceiling, measured }, the
    time under the ceiling, no test twice. None listed is an empty list, so every test
    keeps the 20 s."""
    try:
        with open(MARKER, "rb") as f:
            listed = tomllib.load(f).get("mutants_slow_tests", [])
    except (OSError, tomllib.TOMLDecodeError) as e:
        raise MilestoneError(f"tools/milestones.toml can't be read: {e}") from e
    if not isinstance(listed, list):
        raise MilestoneError("tools/milestones.toml: mutants_slow_tests is a list of tables")
    out = {}
    for c in listed:
        ok = isinstance(c, dict) and sorted(c) == ["ceiling", "measured", "seconds", "test"]
        ok = ok and isinstance(c["test"], str) and isinstance(c["measured"], str) and c["measured"].strip() != ""
        ok = ok and all(isinstance(c[k], (int, float)) and not isinstance(c[k], bool) for k in ("seconds", "ceiling"))
        if not ok or not 0 < c["seconds"] < c["ceiling"]:
            raise MilestoneError("tools/milestones.toml: each mutants_slow_tests entry is { test, seconds, ceiling, "
                                 f"measured }}, its seconds under its ceiling, not {c!r}")
        if c["test"] in out:
            raise MilestoneError(f"tools/milestones.toml: mutants_slow_tests names {c['test']} twice")
        out[c["test"]] = (float(c["seconds"]), float(c["ceiling"]))
    return out


def runner_package(env):
    """The runner's package, from cargo metadata."""
    r = subprocess.run(["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"], cwd=ENGINE, env=env,
                       capture_output=True, text=True, check=False)
    if r.returncode != 0:
        raise MilestoneError(f"cargo metadata failed: {(r.stderr.strip().splitlines() or ['(no output)'])[-1]}")
    runner = [p for p in json.loads(r.stdout)["packages"] if p["name"] == RUNNER]
    if len(runner) != 1:
        raise MilestoneError(f"the workspace has no {RUNNER} package")
    return runner


def required_features(env, names):
    """The features the runner's test targets `names` require (`required-features`, such
    as M1's m1_invariant and fault-injection), sorted: a run that selects one by name
    has to turn them on, or cargo refuses it."""
    wanted = set(names)
    return sorted({f for t in runner_package(env)[0]["targets"] if t["name"] in wanted for f in t.get("required-features", [])})


def targets(env, milestones):
    """The runner's default targets, as cargo arguments, and each milestone's test targets."""
    runner = runner_package(env)
    by_prefix = {m.casefold(): m for m in milestones}
    defaults, by_milestone = [], {m: [] for m in milestones}
    for t in sorted(runner[0]["targets"], key=lambda t: t["name"]):
        kind, name = t["kind"][0], t["name"]
        if t.get("test", True):
            if kind in LIB_KINDS:
                defaults.append("--lib")
            elif kind in ("bin", "test", "bench", "example"):
                defaults.append(f"--{kind}={name}")
        elif kind == "test":
            prefix = name.split("_", 1)[0].casefold() if "_" in name else ""
            if prefix not in by_prefix:
                raise MilestoneError(f"{RUNNER}'s test target {name} has test = false but names no milestone, "
                                     "so it would run nowhere (name it <milestone>_…, such as s_properties)")
            by_milestone[by_prefix[prefix]].append(name)
    return defaults, by_milestone


def gating_targets(env):
    """The gating milestones and their test targets, in build order."""
    milestones, gating, _ = marker()
    _, by_milestone = targets(env, milestones)
    return gating, [t for m in gating for t in by_milestone[m]]


def next_milestone(env):
    """The first milestone that doesn't gate, and its test targets; None when every one gates."""
    milestones, gating, _ = marker()
    _, by_milestone = targets(env, milestones)
    if len(gating) == len(milestones):
        return None, []
    m = milestones[len(gating)]
    return m, by_milestone[m]


def selection(env):
    """The nextest arguments for a mutation run: the runner's default targets and every gating
    milestone's but the marker's `mutants_leave_out` (`--tests` can't be used: it leaves out
    every target with test = false), with the runner's features those targets require
    (M1's m1_invariant: fault-injection), or cargo would refuse them (issue #108, R2)."""
    milestones, gating, leave_out = marker()
    defaults, by_milestone = targets(env, milestones)
    gated = [t for m in gating for t in by_milestone[m]]
    for t in leave_out:
        if t not in gated:
            raise MilestoneError(f"tools/milestones.toml: mutants_leave_out names {t}, which is no gating "
                                 "milestone's test target")
    chosen = [t for t in gated if t not in leave_out]
    features = [f"--features={RUNNER}/{f}" for f in required_features(env, chosen)]
    return defaults + features + [f"--test={t}" for t in chosen]


def tally(out):
    """(tests, passed) from nextest's summary line, or None when it has none; a test a
    fail-fast run didn't reach counts, not green."""
    found = SUMMARY.findall(out)
    return (int(found[-1][0]), int(found[-1][1])) if found else None


def main():
    args = sys.argv[1:]
    env = dict(os.environ)
    try:
        if args in (["args"], ["args", "--mutants"]):
            chosen = selection(env)
            print(" ".join(f"-C={a}" for a in chosen) if "--mutants" in args else " ".join(chosen))
            return 0
        if args:
            print("usage: engine/tools/milestones [args [--mutants]]", file=sys.stderr)
            return 64
        milestones, gating, leave_out = marker()
        _, by_milestone = targets(env, milestones)
    except MilestoneError as e:
        print(f"FAIL milestones: {e}", file=sys.stderr)
        return 1
    for n, m in enumerate(milestones):
        state = "gating" if m in gating else "next" if n == len(gating) else "later"
        names = [f"{t} (not in mutation runs)" if t in leave_out else t for t in by_milestone[m]]
        print(f"{m}: {state}: {', '.join(names) or 'no Rust test target'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
