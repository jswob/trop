"""milestones: which milestones' tests gate (R-77, R-84; engine/tests/MILESTONE-S.md,
"Keeping master green"; issue #14).

    engine/tools/milestones                  each milestone: gating, next or later, and its
                                             Rust test targets
    engine/tools/milestones args             the runner's targets a gated run selects, as
                                             nextest arguments: its default ones and every
                                             gating milestone's (the mutants baseline)
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
    if sorted(data) != ["gating", "milestones"]:
        raise MilestoneError("tools/milestones.toml holds exactly `milestones` and `gating`")
    milestones, gating = data["milestones"], data["gating"]
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
    return milestones, gating


def targets(env, milestones):
    """The runner's default targets, as cargo arguments, and each milestone's test targets."""
    r = subprocess.run(["cargo", "metadata", "--no-deps", "--format-version", "1", "--locked"], cwd=ENGINE, env=env,
                       capture_output=True, text=True, check=False)
    if r.returncode != 0:
        raise MilestoneError(f"cargo metadata failed: {(r.stderr.strip().splitlines() or ['(no output)'])[-1]}")
    runner = [p for p in json.loads(r.stdout)["packages"] if p["name"] == RUNNER]
    if len(runner) != 1:
        raise MilestoneError(f"the workspace has no {RUNNER} package")
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
    milestones, gating = marker()
    _, by_milestone = targets(env, milestones)
    return gating, [t for m in gating for t in by_milestone[m]]


def next_milestone(env):
    """The first milestone that doesn't gate, and its test targets; None when every one gates."""
    milestones, gating = marker()
    _, by_milestone = targets(env, milestones)
    if len(gating) == len(milestones):
        return None, []
    m = milestones[len(gating)]
    return m, by_milestone[m]


def selection(env):
    """The nextest arguments for a gated run: the runner's default targets and every gating
    milestone's (`--tests` can't be used: it leaves out every target with test = false)."""
    milestones, gating = marker()
    defaults, by_milestone = targets(env, milestones)
    return defaults + [f"--test={t}" for m in gating for t in by_milestone[m]]


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
        milestones, gating = marker()
        _, by_milestone = targets(env, milestones)
    except MilestoneError as e:
        print(f"FAIL milestones: {e}", file=sys.stderr)
        return 1
    for n, m in enumerate(milestones):
        state = "gating" if m in gating else "next" if n == len(gating) else "later"
        print(f"{m}: {state}: {', '.join(by_milestone[m]) or 'no Rust test target'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
