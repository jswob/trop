"""mutants-local: CI's mutation run on a few kernel files, for T's loop (issue #40).

    engine/tools/mutants-local FILE… [-F REGEX]

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

The environment is check's (R-82); only CARGO_BUILD_JOBS and NEXTEST_TEST_THREADS pass
through, which change how many jobs run at once, never a result.
"""

import fnmatch
import os
import subprocess
import sys
import tomllib

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
sys.path.insert(0, TOOLS)
import check  # noqa: E402  the clean environment and the fixed seeds (R-82, R-88)

CONFIG = os.path.join(ENGINE, ".cargo", "mutants.toml")
OUT = os.path.join(ENGINE, "target", "mutants-local")
PASSED = ("CARGO_BUILD_JOBS", "NEXTEST_TEST_THREADS")
# CI's flags (.github/workflows/engine.yml's mutants jobs), the shard aside
FLAGS = ["--no-shuffle", "--baseline", "skip", "--timeout", "300", "--build-timeout", "600"]


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
    regex = None
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
    os.makedirs(OUT, exist_ok=True)
    local = os.path.join(OUT, "mutants.toml")
    with open(local, "w", encoding="utf-8") as f:
        f.write(narrowed(config, files))
    argv = ["cargo", "mutants", "--config", local, "-C=--locked", *targets, *FLAGS, "--output", OUT]
    if regex is not None:
        argv += ["-F", regex]
    print(f"mutants-local: {' '.join(argv[2:])}", flush=True)
    return subprocess.run(argv, cwd=ENGINE, env=env, check=False).returncode


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
