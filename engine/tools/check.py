"""The engine's checks, the way CI runs them (R-82).

    engine/tools/check fast    fmt, both clippy passes, the runner, the counter, and the
                               two cheap rule checks (check-frozen, check-rules)
    engine/tools/check full    everything CI runs on a pull request (C §6)
    engine/tools/check list    the steps of each
    engine/tools/check step S  one step of `full` alone (rules-selftest runs probes this way)

It runs from engine/ with a clean environment: nothing from the caller's shell changes
the flags, the compiler, the profiles or the test seeds. It prints one line per failed
step and keeps each step's full output in target/check-logs/<step>.log.
"""

import os
import re
import shutil
import subprocess
import sys
import time

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
LOGS = os.path.join(ENGINE, "target", "check-logs")
NIGHTLY = "nightly-2026-10-21"
FUZZ_SECONDS = 60

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
    return env


CLIPPY = ["cargo", "clippy", "--workspace", "--all-features", "--locked"]
CLIPPY_LIB = [*CLIPPY, "--lib", "--bins", "--", "-D", "warnings"]
# the test pass may use assert! and {:?} (R-109); everything else is denied as in the lib pass
CLIPPY_TESTS = [*CLIPPY, "--tests", "--benches", "--", "-D", "warnings", "-A", "clippy::disallowed_macros", "-A", "clippy::use_debug"]


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
    fast = [
        ("fmt", ["cargo", "fmt", "--all", "--check"], {}),
        ("clippy-lib", CLIPPY_LIB, {}),
        ("clippy-tests", CLIPPY_TESTS, {}),
        ("runner", ["cargo", "nextest", "run", "--locked", "-p", "tally-runner", "--all-features"], {}),
        ("exceptions", tool("exceptions", "--check"), {}),
        ("check-frozen", tool("check-frozen"), {}),
        ("check-rules", tool("check-rules"), {}),
        ("headers", tool("headers"), {}),
        ("trace", tool("trace"), {}),
        ("ast-grep-scan", ["ast-grep", "scan"], {}),
    ]
    if mode == "fast":
        return fast
    return fast + [
        ("doc", ["cargo", "doc", "--workspace", "--no-deps", "--all-features", "--locked"], {"RUSTDOCFLAGS": "-D warnings"}),
        ("build", ["cargo", "build", "--workspace", "--all-targets", "--locked"], {}),
        ("test-dev", ["cargo", "nextest", "run", "--workspace", "--all-features", "--locked", "--profile", "ci"], {}),
        ("test-release", ["cargo", "nextest", "run", "--workspace", "--all-features", "--locked", "--profile", "ci",
                          "--cargo-profile", "release-test"], {}),
        ("doctest", ["cargo", "test", "--doc", "--workspace", "--all-features", "--locked"], {}),
        ("release-binary", ["cargo", "build", "--release", "--locked", "-p", "tally-cli"], {}),
        ("fault-injection-probe", fault_injection_probe, {}),
        ("deny", ["cargo", "deny", "--locked", "check", "bans", "licenses", "sources"], {}),
        ("deny-tally", ["cargo", "deny", "--locked", "--manifest-path", "crates/tally/Cargo.toml",
                        "--config", "deny-tally.toml", "check", "bans"], {}),
        ("tree-probe", tree_probe, {}),
        ("ast-grep-test", ["ast-grep", "test", "--skip-snapshot-tests"], {}),
        ("guard-test", tool("guard-test"), {}),
        ("rules-selftest", tool("rules-selftest/run"), {}),
        ("special-casing", tool("special-casing"), {}),
        ("spec-loadcheck-selftest", ["python3", "-I", "-B", "spec/tools/loadcheck.py", "--selftest"], {}),
        ("spec-loadcheck", ["python3", "-I", "-B", "spec/tools/loadcheck.py", *spec_realities()], {}),
        ("fuzz", fuzz, {}),
    ]


def run(argv, env, log, cwd=ENGINE):
    r = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, check=False)
    log.write(f"$ {' '.join(argv)}\n{r.stdout}\n[exit {r.returncode}]\n")
    return r.returncode, r.stdout


def fault_injection_probe(env, log):
    """R-51: no test-only feature (fault injection, the counters, the reversed order)
    can reach a release build, and the CLI never turns one on."""
    sys.path.insert(0, TOOLS)
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
    """Each fuzz target for FUZZ_SECONDS, its regressions replayed first (gate 9's PR part)."""
    fuzz_dir = os.path.join(ENGINE, "fuzz")
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
        corpus = os.path.join(fuzz_dir, "corpus", target)
        os.makedirs(corpus, exist_ok=True)
        code, _ = run(["cargo", f"+{NIGHTLY}", "fuzz", "run", target, corpus, regressions, "--",
                       f"-max_total_time={FUZZ_SECONDS}"], env, log)
        if code != 0:
            return f"{target}: the fuzzer found a crash (see fuzz/artifacts/{target}/)"
    return None


def first_error(out):
    for line in out.splitlines():
        if re.match(r"(error(\[\w+\])?:|FAIL|ERROR|panicked|thread '.*' panicked)", line.strip()):
            return line.strip()
    lines = [l.strip() for l in out.splitlines() if l.strip()]
    return lines[-1] if lines else "(no output)"


def main():
    args = sys.argv[1:]
    if args == ["list"]:
        for m in ("fast", "full"):
            print(f"{m}: {' '.join(name for name, _, _ in steps(m))}")
        return 0
    if args in (["fast"], ["full"]):
        mode, chosen = args[0], steps(args[0])
    elif len(args) == 2 and args[0] == "step" and any(s[0] == args[1] for s in steps("full")):
        mode, chosen = f"step {args[1]}", [s for s in steps("full") if s[0] == args[1]]
    else:
        print("usage: engine/tools/check fast|full|list|step <name>", file=sys.stderr)
        return 64
    for needed in ("cargo", "python3"):
        if shutil.which(needed) is None:
            print(f"FAIL setup: {needed} is not on PATH")
            return 1
    os.makedirs(LOGS, exist_ok=True)
    env = clean_env()
    failed = []
    began = time.monotonic()
    for name, what, extra in chosen:
        path = os.path.join(LOGS, f"{name}.log")
        with open(path, "w", encoding="utf-8") as log:
            step_env = {**env, **extra}
            if callable(what):
                problem = what(step_env, log)
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
    print(f"check {mode}: green, {total} step{'' if total == 1 else 's'} ({took:.0f}s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
