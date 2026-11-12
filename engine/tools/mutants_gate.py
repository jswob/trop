"""mutants-gate: gate 8's mutation run, each mutant on the T tests that enter its function
(R-84, R-88, R-133; the mutation study, approved 2026-11-12).

    engine/tools/mutants-gate baseline DIR             T's tests unmutated; DIR/times.json
    engine/tools/mutants-gate slow TIMES.json          fails on a T test slower than 20 s
    engine/tools/mutants-gate list OUT.json            cargo-mutants' mutants, with the key
    engine/tools/mutants-gate coverage OUT.json        the coverage map (R-133)
    engine/tools/mutants-gate same-map A.json B.json   two maps are equal (the nightly's twin)
    engine/tools/mutants-gate plan LIST HISTORY N OUT  N shards of even predicted cost
    engine/tools/mutants-gate shard I/N LIST MAP PLAN DIR   shard I's mutants, judged
    engine/tools/mutants-gate history OUT DIR...       a run's costs, for the next plan
    engine/tools/mutants-gate key                      this tree's key
    engine/tools/mutants-gate local [FILE…] [-F REGEX] the same on this machine, one shard

cargo-mutants makes the mutants (`cargo mutants --list --json`: names, spans, diffs). For
each one, the tests are those whose unmutated run entered the mutant's function, in the
run's coverage map (R-133; every T test when the mutant has no function span). The tool
applies the diff in place and builds as cargo-mutants does (the `mutants` cargo profile,
`--cap-lints=warn`), with the targets of those tests only, then runs them in nextest's
`mutants` profile (the memory cap, fail-fast, the slow-timeout kill), the mutant's last
killer first, then T's tests by kills per second. It judges as cargo-mutants does: the
build failed, unviable; past `--timeout`, timed out; nextest's exit 0, missed; exit 100,
caught; any other exit fails the shard. A mutant no T test enters is built and, when it
builds, missed without a test run. The file is put back after each mutant, and a mutant
left in place by a killed run (target/mutants-gate/in-place.json) is put back first.

The map, the list and the plan carry the tree's key: the hash of engine/crates' and
engine/tests' files, the workspace's Cargo.toml and Cargo.lock, the toolchain, the nextest
config, the marker, this tool and its coverage files, the proptest seed and the targets. A
shard refuses a map, a list or a plan whose key isn't its own, and a plan that doesn't hold
every listed mutant exactly once.

The plan puts the mutants into N shards of even predicted cost: each mutant's time in the
last run (its HISTORY, `history`'s output, from the run's shard directories), on the
median runner; the median for a mutant the history lacks. A mutant the last run missed or
timed out on is built with the kernel at opt-level 1 (`--config
profile.mutants.package.tally.opt-level=1`): it will likely run every selected test to the
end, 4.2x faster so (run 43127579597); overflow checks and debug assertions stay on, so
the program means the same.

`local` runs the whole method here: the baseline (a test over 20 s is a notice), the
mutants of FILE… (every kernel file when none is named) matching REGEX, a fresh map, one
shard, every mutant at opt-level 0. Its output is target/mutants-local/mutants.out/, its
exit cargo-mutants' (0 all caught, 2 some missed, 3 some timed out). tools/mutants-local
adds --next and --fences to it.

The environment is check's (R-82): only CARGO_BUILD_JOBS and NEXTEST_TEST_THREADS pass
through, which change how many jobs run at once, never a result.
"""

import concurrent.futures
import fnmatch
import hashlib
import json
import os
import re
import shutil
import signal
import statistics
import subprocess
import sys
import time
import tomllib

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
sys.path.insert(0, TOOLS)
import check  # noqa: E402  the clean environment and the fixed seeds (R-82, R-88)
import milestones  # noqa: E402  T's mutation targets (R-84)

CONFIG = os.path.join(ENGINE, ".cargo", "mutants.toml")
WORK = os.path.join(ENGINE, "target", "mutants-gate")
JOURNAL = os.path.join(WORK, "in-place.json")
COV_TARGET = os.path.join(ENGINE, "target", "mutants-cov")
LOCAL = os.path.join(ENGINE, "target", "mutants-local")
PASSED = ("CARGO_BUILD_JOBS", "NEXTEST_TEST_THREADS")
RUNNER = milestones.RUNNER
# the files the key holds, relative to engine/ (R-133)
KEYED = ["crates", "tests", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".config/nextest.toml",
         "tools/milestones.toml", "tools/mutants_gate.py", "tools/mutants-cov-wrap", "tools/mutants-cov.toml"]
# each mutant's tests and build, in seconds (R-88)
TIMEOUT, BUILD_TIMEOUT = 600, 600
# the baseline's limit on one T test (R-88): about 60 s on a runner 3x slower, half the kill
SLOW = 20.0
O1 = ["--config", "profile.mutants.package.tally.opt-level=1"]
# run 43127579597, all 301 survivors: tests 4.17x faster at opt-level 1, the build 27.5 s longer
O1_TEST, O1_BUILD = 4.17, 27.5
# what the tool adds to each mutant, seconds; and a mutant's cost when nothing is known
OVERHEAD, DEFAULT_COST = 0.3, 30.0
# cargo-mutants' environment for a mutant's build and tests
MUTANT_ENV = {"CARGO_ENCODED_RUSTFLAGS": "--cap-lints=warn", "INSIDE_CARGO_MUTANTS": "true"}
SURVIVED = ("MissedMutant", "Timeout")
GREEN = ("PASS", "LEAK")
# nextest's line for a finished test: `PASS [   0.009s] (1/3) tally-runner::m1_format tests::name`
STATUS = re.compile(r"^\s*([A-Z][A-Z-]*)\s+\[\s*([\d.]+)s\]\s+(?:\(\s*\d+/\d+\)\s+)?(\S+)\s+(\S+)\s*$")
# a hunk header of the mutant's diff
HUNK = re.compile(r"^@@ -(\d+)(?:,\d+)? \+\d+(?:,\d+)? @@.*$", re.M)


class GateError(Exception):
    pass


# --- the environment, the key ----------------------------------------------------------


def run_env():
    env = check.clean_env()
    env.update({k: os.environ[k] for k in PASSED if k in os.environ})
    return env


def selection():
    """T's mutation targets (R-84): `tools/milestones args`, the baseline's and every shard's."""
    try:
        return milestones.selection(dict(os.environ))
    except milestones.MilestoneError as e:
        raise GateError(f"tools/milestones: {e}") from e


def tree_key(args):
    """The key of this tree for the targets `args` (R-133): every keyed file's path and bytes,
    tracked or new (never an ignored one, nor a stray profile), the seed and the targets."""
    out = subprocess.run(["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *KEYED],
                         cwd=ENGINE, capture_output=True, check=False)
    if out.returncode != 0:
        raise GateError(f"git ls-files failed: {out.stderr.decode(errors='replace').strip()}")
    h = hashlib.sha256()
    for rel in sorted({p for p in out.stdout.decode().split("\0") if p and not p.endswith(".profraw")}):
        path = os.path.join(ENGINE, rel)
        if not os.path.isfile(path):
            continue  # tracked, deleted here
        with open(path, "rb") as f:
            h.update(rel.encode() + b"\0" + hashlib.sha256(f.read()).digest())
    h.update(json.dumps({"seed": check.FIXED["PROPTEST_RNG_SEED"], "args": args}).encode())
    return h.hexdigest()[:16]


def load_json(path):
    try:
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    except (OSError, ValueError) as e:
        raise GateError(f"{path}: {e}") from e


def dump_json(path, data):
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=0, sort_keys=True)
        f.write("\n")


def same_key(what, data, key):
    if data.get("key") != key:
        raise GateError(f"the {what}'s key {data.get('key')} isn't this tree's {key}: it was made from another "
                        "tree, so it can't select this tree's tests (R-133)")


def nextest_lines(text):
    """[(status, seconds, test key)] of nextest's finished-test lines."""
    out = []
    for line in text.splitlines():
        m = STATUS.match(line)
        if m:
            out.append((m.group(1), float(m.group(2)), f"{m.group(3)} {m.group(4)}"))
    return out


def tee(argv, env, log, timeout=None):
    """Run argv from engine/, its output to the terminal and to `log`; (exit or "timeout", text, seconds)."""
    start = time.monotonic()
    os.makedirs(os.path.dirname(log), exist_ok=True)
    with open(log, "w", encoding="utf-8") as f:
        f.write(f"*** {' '.join(argv)}\n")
        f.flush()
        p = subprocess.Popen(argv, cwd=ENGINE, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                             text=True, errors="replace", start_new_session=True)
        try:
            for line in p.stdout:
                sys.stdout.write(line)
                sys.stdout.flush()
                f.write(line)
            code = p.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(p.pid, signal.SIGKILL)
            p.wait()
            code = "timeout"
    with open(log, encoding="utf-8") as f:
        return code, f.read(), time.monotonic() - start


# --- the baseline ------------------------------------------------------------------------


def baseline(out_dir, extra=(), expr=None):
    """T's tests unmutated in the mutants profile (R-88); False when one fails. Each test's
    time goes to OUT_DIR/times.json, for the 20 s check."""
    argv = ["cargo", "nextest", "run", "--locked", "-p", RUNNER, "--cargo-profile", "mutants", "--profile", "mutants",
            *selection(), *extra]
    if expr:
        argv += ["-E", expr]
    code, text, _ = tee(argv, run_env(), os.path.join(out_dir, "baseline.log"))
    times = {t: s for st, s, t in nextest_lines(text) if st in GREEN}
    dump_json(os.path.join(out_dir, "times.json"), times)
    if code != 0:
        print(f"mutants-gate: T's tests fail unmutated (exit {code}); no mutant is run", file=sys.stderr)
        return False
    print(f"mutants-gate: baseline: {len(times)} tests pass unmutated")
    return True


def slow(times):
    """The T tests slower than 20 s in the baseline, slowest first (R-88)."""
    return sorted(((s, t) for t, s in times.items() if s > SLOW), reverse=True)


# --- the mutants -------------------------------------------------------------------------


def toml_value(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, str):
        return '"' + v.replace("\\", "\\\\").replace('"', '\\"') + '"'
    if isinstance(v, list):
        return "[" + ", ".join(toml_value(x) for x in v) + "]"
    raise GateError(f".cargo/mutants.toml holds a value that can't be copied: {v!r}")


def narrowed(config, files):
    """.cargo/mutants.toml's text with its examine_globs narrowed to `files`."""
    lines = ["# written by tools/mutants-gate from .cargo/mutants.toml; examine_globs narrowed\n"]
    for key, value in config.items():
        lines.append(f"{key} = {toml_value(files if key == 'examine_globs' else value)}\n")
    return "".join(lines)


def narrowed_config(files):
    """The mutants config to list from: .cargo/mutants.toml, its examine_globs narrowed to
    `files` (None: the whole kernel)."""
    if files is None:
        return CONFIG
    with open(CONFIG, "rb") as f:
        config = tomllib.load(f)
    os.makedirs(WORK, exist_ok=True)
    path = os.path.join(WORK, "mutants.toml")
    with open(path, "w", encoding="utf-8") as f:
        f.write(narrowed(config, files))
    return path


def list_mutants(out, args, files=None, regex=None):
    """cargo-mutants' mutants (`--list --json`), keyed; REGEX keeps those whose name matches it,
    struct-field deletions included (cargo-mutants 27.1.0's -F skips them)."""
    argv = ["cargo", "mutants", "--config", narrowed_config(files), "--list", "--json", "-C=--locked"]
    p = subprocess.run(argv, cwd=ENGINE, env=run_env(), capture_output=True, text=True, check=False)
    if p.returncode != 0:
        raise GateError(f"cargo mutants --list failed:\n{p.stderr.strip()}")
    try:
        mutants = json.loads(p.stdout)
    except ValueError as e:
        raise GateError(f"cargo mutants --list --json: {e}") from e
    if regex is not None:
        mutants = [m for m in mutants if re.search(regex, m["name"])]
    names = [m["name"] for m in mutants]
    if len(set(names)) != len(names):
        raise GateError("cargo mutants --list gives a name twice, so a verdict can't be told to its mutant")
    # every diff applies to its file and changes it, so no shard stops on one mid-run
    texts = {}
    for m in mutants:
        if m["file"] not in texts:
            with open(os.path.join(ENGINE, m["file"]), encoding="utf-8") as f:
                texts[m["file"]] = f.read()
        try:
            same = apply_diff(texts[m["file"]], m["diff"]) == texts[m["file"]]
        except GateError as e:
            raise GateError(f"{m['name']}: {e}") from e
        if same:
            raise GateError(f"{m['name']}: its diff changes nothing")
    dump_json(out, {"key": tree_key(args), "mutants": mutants})
    print(f"mutants-gate: {len(mutants)} mutants")
    return mutants


def apply_diff(text, diff):
    """The mutated text: the diff's hunks, each checked against the original lines."""
    lines = text.split("\n")
    out, pos = [], 0
    parts = HUNK.split(diff)
    if len(parts) < 3:
        raise GateError("the mutant's diff has no hunk")
    for k in range(1, len(parts), 2):
        start = int(parts[k]) - 1
        if start < pos:
            raise GateError("the mutant's diff has hunks out of order")
        out.extend(lines[pos:start])
        pos = start
        for b in parts[k + 1].split("\n")[1:]:
            if b.startswith("\\") or b == "":
                continue
            tag, s = b[0], b[1:]
            if tag in " -":
                if pos >= len(lines) or lines[pos] != s:
                    raise GateError(f"the mutant's diff doesn't match line {pos + 1}")
                pos += 1
                if tag == " ":
                    out.append(s)
            elif tag == "+":
                out.append(s)
            else:
                raise GateError(f"the mutant's diff has a line {b!r}")
    out.extend(lines[pos:])
    return "\n".join(out)


def put_back():
    """A mutant a killed run left in place goes back to its original text first."""
    if not os.path.isfile(JOURNAL):
        return
    j = load_json(JOURNAL)
    with open(os.path.join(ENGINE, j["file"]), "w", encoding="utf-8") as f:
        f.write(j["text"])
    os.remove(JOURNAL)
    print(f"mutants-gate: notice: put {j['file']} back, a mutant a killed run left in place")


class Selector:
    """The tests that enter each mutant's function, in the map (R-133)."""

    def __init__(self, cov):
        self.tests = sorted(cov["tests"])
        self.starts = {}
        for test, files in cov["tests"].items():
            for file, spans in files.items():
                for start, _end in spans:
                    self.starts.setdefault(file, []).append((start, test))

    def __call__(self, mutant):
        span = (mutant.get("function") or {}).get("span")
        if not span:
            return list(self.tests)  # no function span: every T test
        lo, hi = span["start"]["line"], span["end"]["line"]
        return sorted({t for s, t in self.starts.get(mutant["file"], []) if lo <= s <= hi})


def target_args(tests, features):
    """cargo's target arguments for the binaries that hold `tests` (nextest binary ids:
    `pkg` for a lib's tests, `pkg::name` for a test target), the runner's lib alone when none."""
    packages, names, lib = set(), set(), False
    for t in tests:
        binary = t.split(" ", 1)[0]
        package, _, name = binary.partition("::")
        if name.startswith("bin/"):
            raise GateError(f"{t}: a binary's tests aren't T's mutation targets")
        packages.add(package)
        if name:
            names.add(name)
        else:
            lib = True
    if not tests:
        lib = True
    # the runner is always built: the features name it, and cargo refuses an unselected package's
    packages.add(RUNNER)
    return ([f"--package={p}" for p in sorted(packages)] + list(features) + (["--lib"] if lib else [])
            + [f"--test={n}" for n in sorted(names)])


def filterset(tests):
    out = []
    for t in tests:
        binary, name = t.split(" ", 1)
        out.append(f"(binary_id(={binary}) & test(={name}))")
    return " | ".join(out)


def judge(code):
    """cargo-mutants' verdict of a mutant's test run (R-88): nextest's 0 all passed, 100 a test
    failed (a memory-cap or slow-timeout kill included); any other exit is the run's error."""
    if code == "timeout":
        return "Timeout"
    if code == 0:
        return "MissedMutant"
    if code == 100:
        return "CaughtMutant"
    raise GateError(f"nextest exited {code}, which is no verdict: the shard fails (R-88)")


def run_mutant(m, tests, features, opt, log):
    """One mutant: its diff applied, built with its tests' targets, its tests run, the file put
    back. The result's row for results.jsonl."""
    rec = {"name": m["name"], "selected": len(tests), "o1": bool(opt)}
    path = os.path.join(ENGINE, m["file"])
    with open(path, encoding="utf-8") as f:
        original = f.read()
    mutated = apply_diff(original, m["diff"])
    env = dict(run_env(), **MUTANT_ENV)
    base = ["cargo", "nextest", "run", "--cargo-profile=mutants", "--locked", *opt, *target_args(tests, features)]
    os.makedirs(WORK, exist_ok=True)
    dump_json(JOURNAL, {"file": m["file"], "text": original})
    try:
        with open(path, "w", encoding="utf-8") as f:
            f.write(mutated)
        with open(log, "w", encoding="utf-8") as f:
            f.write(f"{m['name']}\n{len(tests)} T tests enter its function{':' if tests else ''} {' '.join(tests)}\n")
        code, build = timed(base[:3] + ["--no-run"] + base[3:], env, log, BUILD_TIMEOUT)
        rec["build"] = round(build, 1)
        if code != 0:
            rec["summary"] = "Timeout" if code == "timeout" else "Unviable"
        elif not tests:
            rec["summary"] = "MissedMutant"
            with open(log, "a", encoding="utf-8") as f:
                f.write("*** no T test enters the mutated function: missed without a test run (R-84)\n")
        else:
            cfg = os.path.join(os.path.dirname(log), os.path.basename(log)[:-4] + ".toml")
            with open(cfg, "w", encoding="utf-8") as f:
                for k, t in enumerate(tests[:200]):
                    f.write(f"[[profile.mutants.overrides]]\nfilter = '{filterset([t])}'\npriority = {100 - k}\n\n")
            code, test = timed(base + ["--profile=mutants", "--no-tests=fail", f"--tool-config-file=mutants-gate:{cfg}",
                                       "-E", filterset(tests)], env, log, TIMEOUT)
            rec["test"] = round(test, 1)
            try:
                rec["summary"] = judge(code)
            except GateError as e:
                raise GateError(f"{m['name']}: {e}; see {log}") from e
            with open(log, encoding="utf-8") as f:
                text = f.read()
            lines = nextest_lines(text)
            if rec["summary"] == "MissedMutant" and len({t for _, _, t in lines}) != len(tests):
                raise GateError(f"{m['name']}: nextest passed but ran {len({t for _, _, t in lines})} of the "
                                f"{len(tests)} tests selected; see {log}")
            rec["fails"] = [[st, t] for st, _, t in lines if st not in (*GREEN, "SIGTERM", "SIGKILL")][:5]
    finally:
        with open(path, "w", encoding="utf-8") as f:
            f.write(original)
        os.remove(JOURNAL)
    return rec


def timed(argv, env, log, timeout):
    start = time.monotonic()
    with open(log, "a", encoding="utf-8") as f:
        f.write(f"\n*** {' '.join(argv)}\n")
        f.flush()
        p = subprocess.Popen(argv, cwd=ENGINE, env=env, stdout=f, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code = p.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(p.pid, signal.SIGKILL)
            p.wait()
            code = "timeout"
        f.write(f"*** exit: {code}\n")
    return code, time.monotonic() - start


def outcome(m, rec, log_rel):
    return {
        "scenario": {"Mutant": {k: m.get(k) for k in ("name", "file", "package", "function", "span", "replacement", "genre")}},
        "summary": rec["summary"],
        "log_path": log_rel,
        "phase_results": [{"phase": "Build", "duration": rec.get("build", 0)},
                          {"phase": "Test", "duration": rec.get("test", 0)}],
    }


def judge_all(names, mutants, cov, plan, features, out, label):
    """The mutants `names`, each judged; DIR/mutants.out/outcomes.json as cargo-mutants writes
    it, the logs beside it, and DIR/results.jsonl (each mutant's times, for the history)."""
    select = Selector(cov)
    o1, killers = set(plan.get("o1", [])), plan.get("killers", {})
    rank = {t: k for k, t in enumerate(plan.get("order", []))}
    mo = os.path.join(out, "mutants.out")
    shutil.rmtree(mo, ignore_errors=True)
    if os.path.exists(os.path.join(out, "results.jsonl")):
        os.remove(os.path.join(out, "results.jsonl"))
    os.makedirs(os.path.join(mo, "log"))
    outcomes, counts = [], {}
    for k, name in enumerate(names):
        m = mutants[name]
        tests = select(m)
        # killers first: the mutant's last killer, then T's tests by kills per second
        tests.sort(key=lambda t: (t != killers.get(name), rank.get(t, len(rank)), t))
        log_rel = f"log/{label}-{k:04d}.log"
        rec = run_mutant(m, tests, features, O1 if name in o1 else [], os.path.join(mo, log_rel))
        rec["log"] = f"mutants.out/{log_rel}"
        with open(os.path.join(out, "results.jsonl"), "a", encoding="utf-8") as f:
            f.write(json.dumps(rec, sort_keys=True) + "\n")
        outcomes.append(outcome(m, rec, log_rel))
        counts[rec["summary"]] = counts.get(rec["summary"], 0) + 1
        print(f"{k + 1}/{len(names)} {rec['summary']:12} build {rec.get('build')} s, tests {rec.get('test', '-')} s, "
              f"{len(tests)} selected{', opt-level 1' if rec['o1'] else ''}: {name}", flush=True)
    with open(os.path.join(mo, "outcomes.json"), "w", encoding="utf-8") as f:
        json.dump({"outcomes": outcomes, "total_mutants": len(outcomes)}, f, indent=1)
    print("mutants-gate: " + ", ".join(f"{v} {k}" for k, v in sorted(counts.items())))
    return counts


def features_of(args):
    return sorted({a for a in args if a.startswith("--features=")})


def shard(spec, list_path, map_path, plan_path, out):
    """Shard I of N: the plan's shard I, judged, after every key and the plan are checked."""
    try:
        i, n = (int(x) for x in spec.split("/"))
    except ValueError as e:
        raise GateError(f"{spec}: a shard is I/N") from e
    put_back()
    args = selection()
    key = tree_key(args)
    listed, cov, plan = load_json(list_path), load_json(map_path), load_json(plan_path)
    same_key("mutants list", listed, key)
    same_key("coverage map", cov, key)
    same_key("plan", plan, key)
    mutants = {m["name"]: m for m in listed["mutants"]}
    planned = [x for s in plan["shards"] for x in s]
    if len(planned) != len(set(planned)) or set(planned) != set(mutants):
        raise GateError("the plan doesn't hold every listed mutant exactly once (R-88)")
    if not set(plan.get("o1", [])) <= set(mutants):
        raise GateError("the plan's opt-level 1 list names a mutant that isn't listed")
    if n != len(plan["shards"]) or not 0 <= i < n:
        raise GateError(f"the plan has {len(plan['shards'])} shards; there is no shard {spec}")
    judge_all(plan["shards"][i], mutants, cov, plan, features_of(args), out, f"{i:03d}")
    return 0


# --- the coverage map (R-133) ------------------------------------------------------------


def llvm_bin():
    sysroot = subprocess.run(["rustc", "--print", "sysroot"], cwd=ENGINE, capture_output=True, text=True,
                             check=True).stdout.strip()
    host = re.search(r"^host: (\S+)$", subprocess.run(["rustc", "-vV"], cwd=ENGINE, capture_output=True, text=True,
                                                       check=True).stdout, re.M).group(1)
    return os.path.join(sysroot, "lib", "rustlib", host, "bin")


def kernel_file(path, globs):
    rel = os.path.relpath(path, ENGINE).replace(os.sep, "/") if os.path.isabs(path) else path
    return rel if any(fnmatch.fnmatchcase(rel, g) for g in globs) else None


def entered(profdata, binary, tools, globs):
    """{kernel file: [[first line, last line], ...]} of the functions a profile entered: every
    function record with an entry count above zero, closures included."""
    exp = subprocess.run([os.path.join(tools, "llvm-cov"), "export", "-format=text", "-skip-expansions",
                          "-instr-profile", profdata, binary], capture_output=True, check=False)
    if exp.returncode != 0:
        raise GateError(f"llvm-cov export failed on {binary}: {exp.stderr.decode(errors='replace')[:500]}")
    data = json.loads(exp.stdout)
    spans = {}
    for fn in data["data"][0]["functions"]:
        if fn["count"] <= 0:
            continue
        files = [kernel_file(f, globs) for f in fn["filenames"]]
        if files[0] is None:
            continue
        regions = [r for r in fn["regions"] if files[r[5]] == files[0]]
        if regions:
            spans.setdefault(files[0], set()).add((min(r[0] for r in regions), max(r[2] for r in regions)))
    return {f: sorted([list(s) for s in v]) for f, v in sorted(spans.items())}


def coverage(out, extra=(), expr=None):
    """The coverage map (R-133): T's mutation targets built with `-C instrument-coverage` and
    run once, each test with its own LLVM profile (tools/mutants-cov-wrap); for each test the
    kernel functions it entered. Fails unless every test passes and leaves exactly one profile."""
    args = [*selection(), *extra]
    key = tree_key(args)
    work = os.path.join(WORK, "cov")
    shutil.rmtree(work, ignore_errors=True)
    profiles, stray = os.path.join(work, "profiles"), os.path.join(work, "stray")
    os.makedirs(profiles)
    os.makedirs(stray)
    env = run_env()
    env.update({"CARGO_ENCODED_RUSTFLAGS": "-Cinstrument-coverage", "CARGO_TARGET_DIR": COV_TARGET,
                "MUTANTS_COV_OUT": profiles, "LLVM_PROFILE_FILE": os.path.join(stray, "%p-%m.profraw")})
    argv = ["cargo", "nextest", "run", "--locked", "-p", RUNNER, "--cargo-profile", "mutants",
            "--config-file", os.path.join(TOOLS, "mutants-cov.toml"), "--profile", "cov", *args]
    if expr:
        argv += ["-E", expr]
    start = time.monotonic()
    code, text, _ = tee(argv, env, os.path.join(work, "run.log"))
    lines = nextest_lines(text)
    ran = {t for _, _, t in lines}
    if code != 0 or any(st not in GREEN for st, _, _ in lines):
        raise GateError(f"the coverage run failed (exit {code}): every T test must pass in it (R-133)")
    counted = milestones.tally(text)
    if counted is None or counted[0] != len(ran):
        raise GateError("the coverage run's summary doesn't match its tests' lines (R-133)")
    seen = {}
    for name in os.listdir(profiles):
        if name.endswith(".test"):
            with open(os.path.join(profiles, name), encoding="utf-8") as f:
                binary, test = f.read().split("\n")[:2]
            raws = [os.path.join(profiles, r) for r in os.listdir(profiles)
                    if r.startswith(name[:-5] + ".") and r.endswith(".profraw")]
            seen[test] = (binary, raws)
    problems = [f"{t}: no profile" for t in sorted(ran - set(seen))]
    problems += [f"{t}: {len(r)} profiles" for t, (_, r) in sorted(seen.items()) if len(r) != 1]
    problems += [f"{t}: a profile, but no test line" for t in sorted(set(seen) - ran)]
    if problems:
        raise GateError("every T test must leave exactly one profile (R-133): " + "; ".join(problems[:10]))
    print(f"mutants-gate: coverage run {time.monotonic() - start:.0f} s, {len(ran)} tests; reading their profiles")
    tools = llvm_bin()
    with open(CONFIG, "rb") as f:
        globs = tomllib.load(f).get("examine_globs", [])

    def one(test):
        binary, (raw,) = seen[test]
        binary = binary if os.path.isabs(binary) else os.path.join(ENGINE, binary)
        prof = raw[:-len(".profraw")] + ".profdata"
        subprocess.run([os.path.join(tools, "llvm-profdata"), "merge", "-sparse", raw, "-o", prof], check=True)
        try:
            return test, entered(prof, binary, tools, globs)
        finally:
            os.remove(prof)

    jobs = int(os.environ.get("NEXTEST_TEST_THREADS") or os.cpu_count() or 4)
    with concurrent.futures.ThreadPoolExecutor(max(1, jobs)) as ex:
        tests = dict(ex.map(one, sorted(seen)))
    dump_json(out, {"key": key, "args": args, "tests": tests})
    print(f"mutants-gate: coverage map {out}: {len(tests)} tests, {time.monotonic() - start:.0f} s, key {key}")
    return 0


# --- the plan and the history -----------------------------------------------------------


def predicted(entry, o1):
    """A mutant's cost in seconds on the median runner, at the opt-level it is planned at."""
    build, test = entry.get("build", 0.0), entry.get("test", 0.0)
    if o1 and not entry.get("o1"):
        build, test = build + O1_BUILD, test / O1_TEST
    elif entry.get("o1") and not o1:
        build, test = max(build - O1_BUILD, 0.0), test * O1_TEST
    return build + test + OVERHEAD


def make_plan(listed, history, n):
    """N shards of even predicted cost (longest first onto the least loaded), numbered by load,
    largest first, so GitHub's queue starts the longest first; the last run's survivors at
    opt-level 1; their killers, and T's tests by kills per second (R-88)."""
    names = [m["name"] for m in listed["mutants"]]
    past = history.get("mutants", {})
    o1 = sorted(x for x in names if past.get(x, {}).get("summary") in SURVIVED)
    cost = {x: predicted(past[x], x in o1) for x in names if x in past}
    median = statistics.median(cost.values()) if cost else DEFAULT_COST
    bins = [[0.0, k, []] for k in range(n)]
    for x in sorted(names, key=lambda x: (-cost.get(x, median), x)):
        b = min(bins, key=lambda b: (b[0], b[1]))
        b[0] += cost.get(x, median)
        b[2].append(x)
    bins.sort(key=lambda b: (-b[0], b[1]))
    tests = history.get("tests", {})
    order = sorted(tests, key=lambda t: (-tests[t]["kills"] / max(tests[t]["seconds"], 0.01), t))
    killers = {x: past[x]["killer"] for x in names if past.get(x, {}).get("killer")}
    return {"key": listed["key"], "shards": [b[2] for b in bins], "o1": o1, "killers": killers, "order": order,
            "load": [round(b[0] / 60, 1) for b in bins], "new": sorted(set(names) - set(past))}


def shard_rows(d):
    """A shard directory's rows and its logs' test lines: [(row, [(status, seconds, test)])]."""
    res = os.path.join(d, "results.jsonl")
    rows = []
    with open(res, encoding="utf-8") as f:
        for k, line in enumerate(f):
            row = json.loads(line)
            # the study's shards (run 43127579597) wrote NNNN.log beside results.jsonl
            log = os.path.join(d, row.get("log") or f"{k:04d}.log")
            try:
                with open(log, encoding="utf-8", errors="replace") as g:
                    lines = nextest_lines(g.read())
            except OSError:
                lines = []
            rows.append((row, lines))
    return rows


def history(out, dirs):
    """A run's costs for the next plan: each mutant's build and test time on the median runner
    (each shard's speed is the median, over the tests its opt-level 0 mutants passed, of the
    test's time there over its median in the whole run), its verdict and its first failing
    test; each test's kills and median time at opt-level 0."""
    shards = {d: shard_rows(d) for d in dirs}
    times = {}
    for d, rows in shards.items():
        for row, lines in rows:
            if not row.get("o1"):
                for st, s, t in lines:
                    if st == "PASS" and s > 0.5:
                        times.setdefault(t, {}).setdefault(d, []).append(s)
    median = {t: statistics.median(x for v in per.values() for x in v) for t, per in times.items()}
    speed = {}
    for d in shards:
        ratios = [statistics.median(per[d]) / median[t] for t, per in times.items() if d in per]
        speed[d] = statistics.median(ratios) if ratios else 1.0
    mutants, kills, seconds = {}, {}, {}
    for d, rows in shards.items():
        for row, lines in rows:
            fails = row.get("fails") or []
            killer = fails[0][1] if row["summary"] == "CaughtMutant" and fails else None
            mutants[row["name"]] = {"build": round(row.get("build", 0.0) / speed[d], 1),
                                    "test": round(row.get("test", 0.0) / speed[d], 1),
                                    "o1": bool(row.get("o1")), "summary": row["summary"], "killer": killer}
            if killer:
                kills[killer] = kills.get(killer, 0) + 1
            if not row.get("o1"):
                for st, s, t in lines:
                    if st in ("PASS", "FAIL"):
                        seconds.setdefault(t, []).append(s / speed[d])
    tests = {t: {"kills": kills.get(t, 0), "seconds": round(statistics.median(seconds.get(t) or [1.0]), 2)}
             for t in sorted(set(kills) | set(seconds))}
    dump_json(out, {"mutants": mutants, "tests": tests,
                    "speeds": {os.path.basename(os.path.normpath(d)): round(s, 2) for d, s in sorted(speed.items())}})
    print(f"mutants-gate: history {out}: {len(mutants)} mutants from {len(dirs)} shards; "
          f"runner speed {min(speed.values()):.2f}-{max(speed.values()):.2f}")
    return 0


# --- on this machine -------------------------------------------------------------------


def local(files=None, regex=None, extra=(), expr=None):
    """The whole method here, one shard (tools/mutants-local, the root engine:mutants)."""
    put_back()
    args = [*selection(), *extra]
    work = os.path.join(WORK, "local")
    print("mutants-gate: T's tests, unmutated", flush=True)
    if not baseline(work, extra, expr):
        return 1
    for s, t in slow(load_json(os.path.join(work, "times.json"))):
        print(f"mutants-gate: notice: {t} took {s:.1f} s here, over the {SLOW:.0f} s CI's baseline allows (R-88)")
    listed = list_mutants(os.path.join(work, "mutants.json"), args, files, regex)
    if not listed:
        print("mutants-gate: no mutant to run")
        return 0
    coverage(os.path.join(work, "map.json"), extra, expr)
    cov = load_json(os.path.join(work, "map.json"))
    mutants = {m["name"]: m for m in listed}
    counts = judge_all([m["name"] for m in listed], mutants, cov, {}, features_of(args), LOCAL, "local")
    return 3 if counts.get("Timeout") else 2 if counts.get("MissedMutant") else 0


def kernel_files(argv):
    with open(CONFIG, "rb") as f:
        globs = tomllib.load(f).get("examine_globs", [])
    files, problems = [], []
    for arg in argv:
        rel = arg.removeprefix("engine/")
        if not os.path.isfile(os.path.join(ENGINE, rel)):
            problems.append(f"{arg}: no such file under engine/")
        elif not any(fnmatch.fnmatchcase(rel, g) for g in globs):
            problems.append(f"{arg}: not a kernel file (.cargo/mutants.toml's examine_globs)")
        files.append(rel)
    return files, problems


def main(argv):
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__.strip())
        return 0 if argv else 64
    cmd, rest = argv[0], argv[1:]
    try:
        if cmd == "key" and not rest:
            print(tree_key(selection()))
            return 0
        if cmd == "baseline" and len(rest) == 1:
            return 0 if baseline(rest[0]) else 1
        if cmd == "slow" and len(rest) == 1:
            over = slow(load_json(rest[0]))
            for s, t in over:
                print(f"FAIL R-88: {t} took {s:.1f} s in the baseline, over {SLOW:.0f} s: T splits it, so a "
                      "slow-timeout kill means a hang, not a slow runner")
            print(f"mutants-gate: {len(over)} T test{'' if len(over) == 1 else 's'} over {SLOW:.0f} s")
            return 1 if over else 0
        if cmd == "list" and len(rest) == 1:
            list_mutants(rest[0], selection())
            return 0
        if cmd == "coverage" and len(rest) == 1:
            return coverage(rest[0])
        if cmd == "same-map" and len(rest) == 2:
            a, b = load_json(rest[0]), load_json(rest[1])
            if a != b:
                differ = sorted(t for t in set(a.get("tests", {})) | set(b.get("tests", {}))
                                if a.get("tests", {}).get(t) != b.get("tests", {}).get(t))
                print(f"FAIL R-133: the twin coverage maps differ (keys {a.get('key')}, {b.get('key')}): "
                      f"{len(differ)} tests, such as {', '.join(differ[:5])}")
                return 1
            print(f"mutants-gate: the twin coverage maps are equal ({len(a.get('tests', {}))} tests)")
            return 0
        if cmd == "plan" and len(rest) == 4:
            listed = load_json(rest[0])
            past = load_json(rest[1])
            plan = make_plan(listed, past, int(rest[2]))
            dump_json(rest[3], plan)
            print(f"mutants-gate: {len(listed['mutants'])} mutants in {len(plan['shards'])} shards, "
                  f"{min(plan['load'])}-{max(plan['load'])} min each; {len(plan['o1'])} at opt-level 1; "
                  f"{len(plan['new'])} new to the history")
            return 0
        if cmd == "shard" and len(rest) == 5:
            return shard(*rest)
        if cmd == "history" and len(rest) >= 2:
            return history(rest[0], rest[1:])
        if cmd == "local":
            regex = None
            if "-F" in rest:
                at = rest.index("-F")
                if at + 1 >= len(rest):
                    print("mutants-gate: -F needs a regex", file=sys.stderr)
                    return 64
                regex, rest = rest[at + 1], rest[:at] + rest[at + 2:]
            files, problems = kernel_files(rest)
            for p in problems:
                print(f"mutants-gate: {p}", file=sys.stderr)
            return 64 if problems else local(files or None, regex)
    except GateError as e:
        print(f"FAIL mutants-gate: {e}", file=sys.stderr)
        return 1
    print(__doc__.strip(), file=sys.stderr)
    return 64


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
