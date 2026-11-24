"""perf: the budget rows on the reference Mac (BUILDER §13.3, §13.5; D4-1, D4-3, D4-5).

    engine/tools/perf table                       every row of tools/perf-budgets.toml, one line each
    engine/tools/perf fingerprint                 the machine's fingerprint (D4-3)
    engine/tools/perf row ID... [--runs N] [--warmups N] [--bin PATH]
                                                  time rows: 2 warm-ups, then the median of 10
    engine/tools/perf row ID... [--runs NAME,...] [--quick] [--bin PATH]
                                                  a load row (`load:`): each of its runs once, cold, or the
                                                  named ones; --quick on the quick profile's binary
    engine/tools/perf all [--runs N] [--warmups N] [--bin PATH]
                                                  every row: the runnable ones timed, the rest say what they wait for
    engine/tools/perf counts ID... [--runs NAME,...] [--bin PATH]
                                                  the counters build's --stats against the row's count budgets
    engine/tools/perf profile ID [--runs NAME] [--bin PATH]
                                                  samply's profile of one run, summarised as text (macOS)
    engine/tools/perf limits BENCH                the gungraun hard limit of a bench, as its command-line flag
    engine/tools/perf growth [--runs N] [--warmups N] [--bin PATH]
                                                  the [growth] text at its two horizons: both rows and the ratios

Each timed run is the release binary under /usr/bin/time -l (macOS): wall time is this
script's clock around it, peak memory footprint and instructions retired are time's. A row
passes when its median wall time and its peak footprint are within budget. A run whose
resident size passes 8 GB is killed (D4-7). A fast wrong answer fails: after timing, the
output's SHA-256 is compared with T's expected digest (engine/tests/perf-digests.tsv, id and
digest per line), and a row with no digest yet is reported unjudged.

A turn row (`turn:<id> <id> …`, P8-1: a conversation turn, one `check` and two `run`s, cold)
runs vectors of M3's stores (tests/m3/check.vectors, tests/m3/runs.vectors: their cwd and
argument words) in order, each a cold process of the release binary: a run's wall time is the
sum of its processes' walls, its footprint the largest peak, its instructions the sum; the
median and the warm-ups are as for one process, and each process's output is checked against
its own vector's digest. Its line names any process that exited other than its vector's code.

A load row (`load:<row> [<run> …]`, PERFORMANCE §8.3; pick P5 (a) on #252, 2026-11-22)
runs the rows of T's run list, engine/tests/m4/load/runs.tsv, whose `row` is <row> (or only
the runs the recipe names): each run once, cold, its cwd, files and `--ask` ids as the list
has them, on the release binary under /usr/bin/time -l, with tests/tools/memcap.py watching
at 8 GB (started for the row unless one is already running, and stopped after it). Each run
has the row's budget of its own, so the row is judged by its slowest run's wall and its
largest peak; a run killed at 8 GB or 90 s misses. Each output's SHA-256 is checked against
the list's `sha256` column where it holds one; `-` is reported, not judged. One line per run,
then the row's. `--runs a,b` runs those runs alone (the row's line says how many of how
many), `--quick` the quick profile's binary, so a builder can measure one run of a stage in
the middle of a group; `--runs N` and `--warmups` don't apply. `perf counts` gives each run's
counts, `perf profile` one named run's profile.

A run row (`run:<cwd> <word> …`, P8-2's 1x twin) is one `tally run` of a text T stores that no
list names: the words after the folder (from the repository's root) are the run's own, files
first, then `--ask` and its ids. It is timed and counted as a check row is; its digest is the
row id's line of perf-digests.tsv, unjudged while it has none.

A row whose `blocks` is "report" (P8-2 until the long runs' remedy, issue #256) prints its
figures beside its budget, marked `report`, and never fails on them: no tag waits on it. A
wrong answer fails it as any row.

`perf growth` times the budget table's [growth] text (`short`, `long`: two `run:` recipes of
one text whose horizon its header sets) as a row is timed, reads each header's `start` and
`end`, and prints a Markdown table of the two and the ratios of the span, the time and the
peak, each with its exponent (1 is linear in the span), for the milestone's report
`engine/perf/M<n>.md`. It reports and never judges; it exits 1 only when a run gave no figure.

A micro row (`bench:<name>`, P8-M*) runs its criterion bench, crates/tally/benches/<name>.rs,
whose id is the row's: the wall time is criterion's median (less the median of the bench id
the row's `less` names, if any: P8-M4's run less the plain run), and the bench itself fails
the run when a result isn't its known value. Its instructions run under gungraun on Linux CI
(tools/gungraun/benches/<name>.rs), where it has one (P8-5 has none: its instruction limit
is P8-M4's); a bench with no criterion twin is left to CI.

Until the lead sets the kernel at S (tools/kernel.toml), every number is proposed
(PERFORMANCE §8): perf reports misses and exits 0. After S a miss exits 1. A wrong answer
exits 1 always. Standard library only, like run.py.
"""

import csv
import datetime
import hashlib
import json
import math
import os
import platform
import re
import signal
import statistics
import subprocess
import sys
import tempfile
import time
import tomllib

sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
import check  # noqa: E402  the cleaned environment

TOOLS = check.TOOLS
ENGINE = check.ENGINE
ROOT = os.path.dirname(ENGINE)
TABLE = os.path.join(TOOLS, "perf-budgets.toml")
KERNEL = os.path.join(TOOLS, "kernel.toml")
CHECKS = os.path.join(ENGINE, "tests", "checks.tsv")
DIGESTS = os.path.join(ENGINE, "tests", "perf-digests.tsv")
M3_STORES = (os.path.join(ENGINE, "tests", "m3", "check.vectors"), os.path.join(ENGINE, "tests", "m3", "runs.vectors"))
LOAD_RUNS = os.path.join(ENGINE, "tests", "m4", "load", "runs.tsv")  # T's run list of the load rows (P5)
MEMCAP = os.path.join(ENGINE, "tests", "tools", "memcap.py")
BENCHES = os.path.join(ENGINE, "crates", "tally", "benches")
GUNGRAUN = os.path.join(TOOLS, "gungraun", "benches")
OUT = os.path.join(ENGINE, "target", "perf")
CEILING_KB = 8 * 1024 * 1024  # D4-7: 8 GB on any run
TIMEOUT_S = 90.0  # the user's hard budget for one run
COUNT_KEYS = ("def_misses", "days_stepped", "walks_shared")  # walks shared: fence F6's (PERFORMANCE §2.5)
SPAN = re.compile(r"^\s+(start|end) (\d{4}-\d{2}-\d{2})\s*$")  # a model header's `start` and `end` lines


def rows():
    with open(TABLE, "rb") as f:
        return tomllib.load(f)["row"]


def row_of(row_id):
    found = [r for r in rows() if r["id"] == row_id]
    if not found:
        sys.exit(f"perf: no row {row_id} in tools/perf-budgets.toml")
    return found[0]


def reported(r):
    """Whether the row only reports (`blocks = "report"`): its misses print, and fail nothing."""
    return r.get("blocks") == "report"


def set_at_s():
    try:
        with open(KERNEL, "rb") as f:
            return tomllib.load(f).get("set_at_s") is True
    except (OSError, ValueError):
        return False


def budget_text(r):
    parts = []
    if "wall_s" in r:
        parts.append(f"wall({r.get('workers', 1)}w) <= {seconds(r['wall_s'])}")
    if "footprint_mb" in r:
        parts.append(f"footprint <= {r['footprint_mb']} MB")
    for key, limit in r.get("counts", {}).items():
        parts.append(f"{key} <= {limit:,}".replace(",", " "))
    if "instructions" in r:
        parts.append(f"ir <= {r['instructions']:,}".replace(",", " "))
    return "; ".join(parts) or "no budget of its own"


def seconds(s):
    if s < 0:
        return f"-{seconds(-s)}"
    if s < 0.000001:
        return f"{s * 1e9:.1f} ns"
    if s < 0.001:
        return f"{s * 1e6:.2f} µs"
    if s < 1:
        return f"{s * 1e3:.2f} ms"
    return f"{s:.2f} s"


# -- the machine ---------------------------------------------------------------------------------------------------


def output_of(argv):
    try:
        return subprocess.run(argv, capture_output=True, text=True, timeout=10, check=False).stdout.strip()
    except OSError:
        return "unavailable"


def fingerprint():
    """sysctl hw.model, sw_vers, rustc -V and pmset -g, on one line (D4-3)."""
    rustc = output_of(["rustc", "-V"])
    if platform.system() != "Darwin":
        return f"fingerprint: {platform.system()} {platform.release()} {platform.machine()}; {rustc}; not the reference Mac"
    model = output_of(["sysctl", "-n", "hw.model"])
    macos = " ".join(output_of(["sw_vers", v]) for v in ("-productVersion", "-buildVersion"))
    pmset = output_of(["pmset", "-g"])
    power = output_of(["pmset", "-g", "batt"]).split("\n")[0].replace("Now drawing from ", "").strip("'")
    low = [line.split()[-1] for line in pmset.split("\n") if line.strip().startswith("lowpowermode")]
    return f"fingerprint: {model}; macOS {macos}; {rustc}; power: {power}; lowpowermode {low[0] if low else '?'}"


# -- the recipes ---------------------------------------------------------------------------------------------------


def check_row(check_id):
    with open(CHECKS, encoding="utf-8", newline="") as f:
        for c in csv.DictReader(f, delimiter="\t"):
            if c["id"] == check_id:
                return c
    sys.exit(f"perf: no check {check_id} in engine/tests/checks.tsv")


def run_words(text, who):
    """A `run:<cwd> <word> …` recipe's (argv after the binary, cwd): `run` and its words, from
    the folder the first word names under the repository's root."""
    words = text[len("run:"):].split()
    if len(words) < 2:
        sys.exit(f"perf: {who}: a run recipe names its folder and the run's words: run:<cwd> <file> … [--ask <id> …]")
    cwd = os.path.join(ROOT, words[0])
    if not os.path.isdir(cwd):
        sys.exit(f"perf: {who}: no folder {words[0]}")
    return ["run", *words[1:]], cwd


def recipe(r):
    """(argv after the binary, cwd), as run.py runs the check or as a `run:` recipe names the
    run, or None for a row that runs no one process."""
    if r["recipe"].startswith("run:"):
        return run_words(r["recipe"], r["id"])
    if not r["recipe"].startswith("check:"):
        return None
    c = check_row(r["recipe"][len("check:"):])
    cwd = os.path.join(ROOT, c["cwd"])
    files = [os.path.relpath(os.path.join(ROOT, f), cwd) for f in c["files"].split()]
    argv = ["run", *files, *c["options"].split()] + (["--ask", *c["asks"].split()] if c["asks"] else [])
    return argv, cwd


def m3_vectors():
    """{id: (cwd, argument words, exit code)} of M3's stores: eight netstrings a vector (id, group,
    source, cwd, the argument words joined by single spaces, the file table, the exit code, the
    output; tests/m3/vectors.py)."""
    out = {}
    for path in M3_STORES:
        with open(path, "rb") as f:
            b = f.read()
        parts, i = [], 0
        while i < len(b):
            j = b.index(b":", i)
            n = int(b[i:j])
            parts.append(b[j + 1:j + 1 + n].decode("utf-8"))
            i = j + 2 + n
        for k in range(0, len(parts) - 7, 8):
            out[parts[k]] = (parts[k + 3], parts[k + 4].split(" "), int(parts[k + 6]))
    return out


def steps(r):
    """The processes one timed run of the row starts: [(digest id, argv after the binary, cwd,
    expected exit code or None)], or None for a row that runs none (a bench, a test, a wait)."""
    plan = recipe(r)
    if plan is not None:
        digest_id = r["recipe"][len("check:"):] if r["recipe"].startswith("check:") else r["id"]
        return [(digest_id, *plan, None)]
    if not r["recipe"].startswith("turn:"):
        return None
    vectors = m3_vectors()
    out = []
    for vid in r["recipe"][len("turn:"):].split():
        if vid not in vectors:
            sys.exit(f"perf: {r['id']}: {vid} is no vector of tests/m3/check.vectors or tests/m3/runs.vectors")
        cwd, argv, code = vectors[vid]
        out.append((vid, argv, os.path.join(ROOT, cwd), code))
    return out


def load_runs(r, only=None):
    """A load row's runs (`load:<row> [<run> …]`): [(run name, argv after the binary, cwd,
    expected sha256 or None)] from T's run list, in its order, the recipe's named runs only
    when it names some, then of those the `only` names (--runs; [] when it names none of
    them); None for a row that isn't one."""
    if not r["recipe"].startswith("load:"):
        return None
    words = r["recipe"][len("load:"):].split()
    if not words:
        sys.exit(f"perf: {r['id']}: a load recipe names its row of tests/m4/load/runs.tsv: load:<row> [<run> …]")
    with open(LOAD_RUNS, encoding="utf-8", newline="") as f:
        listed = [x for x in csv.DictReader(f, delimiter="\t") if x["row"] == words[0]]
    names = [x["run"] for x in listed]
    for want in words[1:]:
        if want not in names:
            sys.exit(f"perf: {r['id']}: tests/m4/load/runs.tsv has no run {want} of {words[0]} "
                     f"(its runs: {' '.join(names) or 'none'})")
    taken = [x for x in listed if len(words) == 1 or x["run"] in words[1:]]
    if not taken:
        sys.exit(f"perf: {r['id']}: tests/m4/load/runs.tsv has no run of {words[0]}")
    chosen = [x for x in taken if not only or x["run"] in only]
    return [(x["run"], ["run", *x["files"].split(), *(["--ask", *x["asks"].split()] if x["asks"] else [])],
             os.path.join(ROOT, x["cwd"]), None if x["sha256"] == "-" else x["sha256"]) for x in chosen]


def digests():
    try:
        with open(DIGESTS, encoding="utf-8") as f:
            pairs = (line.rstrip("\n").split("\t") for line in f if line.strip() and not line.startswith("#"))
            return {p[0]: p[1] for p in pairs if len(p) == 2}
    except OSError:
        return {}


def build(features=(), quick=False):
    """The release binary, the quick profile's (--quick, a load row), or the counted one
    (debug assertions on, so the counters may exist: R-51)."""
    argv = ["cargo", "build", "--locked", "-p", "tally-cli"]
    if features:
        argv += ["--features", ",".join(features)]
    else:
        argv += ["--profile", "quick"] if quick else ["--release"]
    done = subprocess.run(argv, cwd=ENGINE, env=check.clean_env(), capture_output=True, text=True, check=False)
    if done.returncode != 0:
        sys.exit(f"perf: {' '.join(argv)} failed:\n{done.stderr[-2000:]}")
    target = os.path.join(ENGINE, os.environ.get("CARGO_TARGET_DIR") or "target")
    return os.path.join(target, "debug" if features else "quick" if quick else "release", "tally")


# -- one run -------------------------------------------------------------------------------------------------------


def resident_kb(pid):
    """The resident size of pid and its children, in KB (the watchdog's approximation of the footprint)."""
    pids = [str(pid), *output_of(["pgrep", "-P", str(pid)]).split()]
    sizes = output_of(["ps", "-o", "rss=", "-p", ",".join(pids)]).split()
    return sum(int(s) for s in sizes if s.isdigit())


def stop(child):
    """Kill a run this script started and the engine under it: /usr/bin/time's child first,
    which a kill of time alone would leave running."""
    for pid in output_of(["pgrep", "-P", str(child.pid)]).split():
        try:
            os.kill(int(pid), signal.SIGKILL)
        except (OSError, ValueError):
            pass
    child.kill()
    child.wait()


def timed(binary, argv, cwd):
    """One run under /usr/bin/time -l: (wall s, footprint bytes, instructions, exit code, output bytes)."""
    with tempfile.NamedTemporaryFile(prefix="perf-time-", suffix=".txt") as stats:
        command = ["/usr/bin/time", "-l", "-o", stats.name, binary, *argv]
        if platform.system() != "Darwin":
            command = [binary, *argv]
        start = time.perf_counter()
        with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
            child = subprocess.Popen(command, cwd=cwd, stdin=subprocess.DEVNULL, stdout=out, stderr=err)
            while True:
                try:
                    child.wait(timeout=0.25)
                    break
                except subprocess.TimeoutExpired:
                    if resident_kb(child.pid) > CEILING_KB or time.perf_counter() - start > TIMEOUT_S:
                        stop(child)
                        return None
            wall = time.perf_counter() - start
            out.seek(0)
            err.seek(0)
            output = out.read() + err.read()
        fields = {}
        for line in open(stats.name, encoding="utf-8", errors="replace"):
            words = line.split()
            if len(words) >= 2 and words[0].isdigit():
                fields[" ".join(words[1:])] = int(words[0])
    return wall, fields.get("peak memory footprint"), fields.get("instructions retired"), child.returncode, output


def criterion_row(r, name):
    """A micro row's wall time: its criterion bench (crates/tally/benches/<name>.rs, its id the
    row's), whose median is criterion's own over its samples, so --runs and --warmups don't
    apply. The bench checks its results after timing and fails the run on a wrong one."""
    argv = ["cargo", "bench", "--locked", "-p", "tally", "--bench", name]
    done = subprocess.run(argv, cwd=ENGINE, env=check.clean_env(), capture_output=True, text=True, check=False)
    if done.returncode != 0:
        return (f"BUDGET {r['id']}: WRONG or unbuilt: {' '.join(argv)} exited {done.returncode}: "
                f"{check.first_error(done.stdout + done.stderr)}"), True
    target = os.path.join(ENGINE, os.environ.get("CARGO_TARGET_DIR") or "target")
    try:
        wall, how = criterion_wall(r, target)
    except LookupError as e:
        return f"BUDGET {r['id']}: criterion wrote no median at {e}", True
    counted = ("its instructions under gungraun on Linux CI" if os.path.isfile(os.path.join(GUNGRAUN, f"{name}.rs"))
               else "no gungraun twin")
    note = f"wall(1w): median {seconds(wall)} ({how}; {counted})"
    if "wall_s" in r and wall > r["wall_s"]:
        return (f"BUDGET {r['id']} wall(1w): median {seconds(wall)} > {seconds(r['wall_s'])} "
                f"(+{(wall / r['wall_s'] - 1) * 100:.0f} %, {how})"), set_at_s()
    return f"ok {r['id']} {note} (budget: {budget_text(r)})", False


def criterion_median(target, bench_id):
    """Criterion's median, in seconds, of the bench id `bench_id` under `target`; a
    LookupError naming the file when it wrote none."""
    estimates = os.path.join(target, "criterion", bench_id, "new", "estimates.json")
    try:
        with open(estimates, encoding="utf-8") as f:
            return json.load(f)["median"]["point_estimate"] / 1e9
    except (OSError, ValueError, KeyError, TypeError):
        raise LookupError(os.path.relpath(estimates, ROOT)) from None


def criterion_wall(r, target):
    """A micro row's wall time and how it was read: its own bench id's median, less the
    median of the bench id its `less` names, when it names one (P8-M4: the run with the
    projection less the plain run, P §8.4; pick P56, 2026-11-09)."""
    wall = criterion_median(target, r["id"])
    if "less" not in r:
        return wall, "criterion"
    plain = criterion_median(target, r["less"])
    return wall - plain, f"criterion, {seconds(wall)} less {r['less']}'s {seconds(plain)}"


def timed_turn(binary, plan):
    """One timed run of a row: each of its processes in order (one for a check row), as
    (wall s summed, the largest footprint, instructions summed, [(exit code, output bytes)]),
    or None when one was killed. A footprint or count time didn't report is left out."""
    walls, footprints, instructions, outputs = [], [], [], []
    for _, argv, cwd, _ in plan:
        one = timed(binary, argv, cwd)
        if one is None:
            return None
        walls.append(one[0])
        footprints += [one[1]] if one[1] is not None else []
        instructions += [one[2]] if one[2] is not None else []
        outputs.append((one[3], one[4]))
    return (sum(walls), max(footprints) if footprints else None,
            sum(instructions) if len(instructions) == len(plan) else None, outputs)


def judged(plan, outputs):
    """Each process's output against its digest: (misses, notes) for the row's line. One process
    (a check row) reads as before; a turn's lines name the vector."""
    known, misses, notes = digests(), [], []
    for (vid, _, _, want_code), (code, output) in zip(plan, outputs):
        who = f"{vid}: " if len(plan) > 1 else ""
        expected = known.get(vid)
        if want_code is not None and code != want_code:
            notes.append(f"{who}exit {code}, its vector's {want_code}")
        if expected is None:
            notes.append(f"{who}unjudged: no digest in engine/tests/perf-digests.tsv yet (exit {code})")
        elif hashlib.sha256(output).hexdigest() != expected:
            misses.append(f"{who}WRONG: the output's digest isn't T's (exit {code}): a fast wrong answer fails")
    return misses, notes


def time_row(r, binary, runs, warmups, only=None):
    """One line for one row, and whether it failed (a wrong answer, or a miss after S)."""
    if r["recipe"].startswith("load:"):
        return time_load(r, binary, only)
    plan = steps(r)
    name = r["recipe"][len("bench:"):]
    if plan is None and r["recipe"].startswith("bench:") and os.path.isfile(os.path.join(BENCHES, f"{name}.rs")):
        return criterion_row(r, name)
    if plan is None and r["recipe"].startswith("bench:"):
        return f"{r['id']}: {r['recipe']}, its instructions under gungraun on Linux CI", False
    if plan is None:
        return f"{r['id']}: {r['recipe']}", False
    results = []
    for _ in range(warmups + runs):
        one = timed_turn(binary, plan)
        if one is None and reported(r):
            return f"report {r['id']}: a run passed 8 GB or {TIMEOUT_S:.0f} s and was killed (D4-7); no tag waits on it", False
        if one is None:
            return f"BUDGET {r['id']}: a run passed 8 GB or {TIMEOUT_S:.0f} s and was killed (D4-7)", True
        results.append(one)
    measured = results[warmups:]
    wall = statistics.median(m[0] for m in measured)
    footprints = [m[1] for m in measured if m[1] is not None]
    peak = max(footprints) if footprints else None
    instructions = [m[2] for m in measured if m[2] is not None]
    misses, notes = [], []
    if "wall_s" in r and wall > r["wall_s"]:
        misses.append(f"wall({r.get('workers', 1)}w): median {seconds(wall)} > {seconds(r['wall_s'])} "
                      f"(+{(wall / r['wall_s'] - 1) * 100:.0f} %, {runs} runs)")
    else:
        notes.append(f"wall({r.get('workers', 1)}w): median {seconds(wall)}")
    if peak is not None and "footprint_mb" in r and peak > r["footprint_mb"] * 1024 * 1024:
        misses.append(f"footprint: peak {peak / 1048576:.1f} MB > {r['footprint_mb']} MB")
    elif peak is not None:
        notes.append(f"footprint: peak {peak / 1048576:.1f} MB")
    if instructions:
        notes.append(f"instructions: median {int(statistics.median(instructions)):,}".replace(",", " "))
    if len(plan) > 1:
        notes.append(f"{len(plan)} processes a run: {' '.join(s[0] for s in plan)}")
    wrong_lines, judged_notes = judged(plan, measured[-1][3])
    wrong = bool(wrong_lines)
    misses += wrong_lines
    notes += judged_notes
    if reported(r) and not wrong:
        return (f"report {r['id']} " + "; ".join(misses + notes)
                + f" ({runs} runs; beside the budget: {budget_text(r)}; reported, no tag waits on it)"), False
    if misses:
        return f"BUDGET {r['id']} " + "; ".join(misses + notes), wrong or set_at_s()
    return f"ok {r['id']} " + "; ".join(notes) + f" ({runs} runs; budget: {budget_text(r)})", False


# -- a load row ----------------------------------------------------------------------------------------------------


def memcap_running():
    """Whether a memcap.py watchdog runs already (anyone's: it watches every engine run): a
    Python process whose script is memcap.py, not a command line that merely names it."""
    for line in output_of(["ps", "-axo", "command="]).split("\n"):
        words = line.split()
        if not words or not os.path.basename(words[0]).startswith(("python", "pypy", "Python")):
            continue
        script = next((w for w in words[1:] if not w.startswith("-")), "")
        if os.path.basename(script) == "memcap.py" and "--self-test" not in words:
            return True
    return False


def start_memcap():
    """The memcap.py watchdog at 8 GB for a load row (macOS: it reads top's footprint), or
    None when one runs already or the machine isn't the Mac; its caller stops it."""
    if platform.system() != "Darwin" or memcap_running():
        return None
    return subprocess.Popen([sys.executable, "-I", "-B", MEMCAP, "8", "2"], stdin=subprocess.DEVNULL,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def megabytes(b):
    return f"{b / 1048576:.1f} MB"


def load_run_line(r, name, one, wall, want):
    """One run's line and its (wall, peak or None, killed, wrong)."""
    head = f"  {r['id']} {name}:"
    if one is None:
        return f"{head} killed after {seconds(wall)}: passed 8 GB resident or {TIMEOUT_S:.0f} s", (wall, None, True, False)
    wall, peak, instructions, code, output = one
    killed = code < 0
    parts = [f"wall {seconds(wall)}", f"peak {megabytes(peak)}" if peak is not None else "peak unreported"]
    if instructions is not None:
        parts.append(f"instructions {instructions:,}".replace(",", " "))
    parts.append(f"killed by signal {-code} (memcap at 8 GB)" if killed else f"exit {code}")
    digest = hashlib.sha256(output).hexdigest()
    wrong = want is not None and not killed and digest != want
    if killed:
        parts.append("no output judged")
    elif want is None:
        parts.append(f"sha256 {digest[:12]} reported (runs.tsv holds -)")
    else:
        parts.append(f"WRONG: sha256 {digest[:12]}, runs.tsv's {want[:12]}: a fast wrong answer fails" if wrong
                     else "sha256 runs.tsv's")
    return f"{head} " + ", ".join(parts), (wall, peak, killed, wrong)


def time_load(r, binary, only):
    """A load row (pick P5 (a)): each run once, cold, a line each, then the row's line, judged by
    the slowest run's wall and the largest peak; and whether it failed (a wrong answer or a
    killed run, or a miss after S)."""
    plan, whole = load_runs(r, only), len(load_runs(r))
    if not plan:
        return f"{r['id']}: --runs names none of its {whole} runs", False
    watchdog = start_memcap()
    results = []
    try:
        for name, argv, cwd, want in plan:
            start = time.perf_counter()
            one = timed(binary, argv, cwd)
            line, result = load_run_line(r, name, one, time.perf_counter() - start, want)
            print(line, flush=True)
            results.append((name, *result))
    finally:
        if watchdog is not None:
            watchdog.terminate()
            watchdog.wait()
    slowest = max(results, key=lambda x: x[1])
    peaks = [x for x in results if x[2] is not None]
    largest = max(peaks, key=lambda x: x[2]) if peaks else None
    killed = [x[0] for x in results if x[3]]
    wrong = [x[0] for x in results if x[4]]
    misses, notes = [], []
    over = [x[0] for x in results if "wall_s" in r and x[1] > r["wall_s"]]
    wall_text = f"slowest {slowest[0]} {seconds(slowest[1])}"
    if over:
        misses.append(f"wall({r.get('workers', 1)}w): {wall_text} > {seconds(r['wall_s'])} "
                      f"(+{(slowest[1] / r['wall_s'] - 1) * 100:.0f} %; {len(over)} over: {' '.join(over)})")
    else:
        notes.append(f"wall({r.get('workers', 1)}w): {wall_text}")
    if largest is not None and "footprint_mb" in r and largest[2] > r["footprint_mb"] * 1048576:
        heavy = [x[0] for x in peaks if x[2] > r["footprint_mb"] * 1048576]
        misses.append(f"footprint: largest {largest[0]} {megabytes(largest[2])} > {r['footprint_mb']} MB "
                      f"({len(heavy)} over: {' '.join(heavy)})")
    elif largest is not None:
        notes.append(f"footprint: largest {largest[0]} {megabytes(largest[2])}")
    if killed:
        misses.append(f"killed at 8 GB or {TIMEOUT_S:.0f} s (D4-7): {' '.join(killed)}")
    if wrong:
        misses.append(f"WRONG output: {' '.join(wrong)}")
    unjudged = sum(1 for _, _, _, want in plan if want is None)
    count = f"{len(plan)} of {whole} runs (--runs)" if len(plan) < whole else f"{len(plan)} run{'s' * (len(plan) > 1)}"
    notes.append(f"{count}, each once, cold" + (f"; {unjudged} unjudged (sha256 -)" if unjudged else ""))
    if reported(r) and not wrong:
        return (f"report {r['id']} " + "; ".join(misses + notes)
                + f" (beside the budget: {budget_text(r)}, each run; reported, no tag waits on it)"), False
    if misses:
        return f"BUDGET {r['id']} " + "; ".join(misses + notes), bool(wrong or killed) or set_at_s()
    return f"ok {r['id']} " + "; ".join(notes) + f" (budget: {budget_text(r)}, each run)", False


# -- growth with the horizon ----------------------------------------------------------------------------------------


def growth_table():
    """The budget table's [growth]: `what`, and `short` and `long`, two `run:` recipes of one text."""
    with open(TABLE, "rb") as f:
        g = tomllib.load(f).get("growth")
    if not g or not all(isinstance(g.get(k), str) and g[k].startswith("run:") for k in ("short", "long")):
        sys.exit("perf: tools/perf-budgets.toml's [growth] names `short` and `long`, each a `run:` recipe")
    return g


def span_of(cwd, header, who):
    """(start, end, days from start through end) from the model header's own `start` and `end`
    lines: the run's first file."""
    found = {}
    try:
        with open(os.path.join(cwd, header), encoding="utf-8") as f:
            for line in f:
                m = SPAN.match(line)
                if m and m.group(1) not in found:
                    found[m.group(1)] = m.group(2)
    except OSError:
        sys.exit(f"perf: growth {who}: no header {header}")
    if set(found) != {"start", "end"}:
        sys.exit(f"perf: growth {who}: {header} states no `start` and `end` lines: the run's first file is its header")
    days = (datetime.date.fromisoformat(found["end"]) - datetime.date.fromisoformat(found["start"])).days + 1
    return found["start"], found["end"], days


def ratio(a, b, span):
    """b / a with its exponent over the span's ratio (1 is linear in the span)."""
    if a is None or b is None or a <= 0:
        return "unreported"
    return f"x{b / a:.2f} (exponent {math.log(b / a) / math.log(span):.2f})"


def cmd_growth(opts):
    """The [growth] text at its two horizons, each timed as a row is (the warm-ups, then the
    median), a Markdown table and the ratios for engine/perf/M<n>.md."""
    g = growth_table()
    print(fingerprint())
    binary = opts["bin"] or build()
    measured = []
    for key in ("short", "long"):
        argv, cwd = run_words(g[key], f"[growth] {key}")
        start, end, days = span_of(cwd, argv[1], key)
        kept = []
        for _ in range(opts["warmups"] + opts["runs"]):
            one = timed(binary, argv, cwd)
            if one is None or one[3] != 0:
                why = f"passed 8 GB or {TIMEOUT_S:.0f} s and was killed (D4-7)" if one is None else f"exited {one[3]}"
                print(f"growth {key}: {' '.join(argv)} {why}: no figure")
                return 1
            kept.append(one)
        kept = kept[opts["warmups"]:]
        peaks = [k[1] for k in kept if k[1] is not None]
        counted = [k[2] for k in kept if k[2] is not None]
        measured.append((key, " ".join(argv[1:]), start, end, days, statistics.median(k[0] for k in kept),
                         max(peaks) if peaks else None, statistics.median(counted) if counted else None))
    print(f"\n{g.get('what', 'the [growth] text')}, {opts['runs']} runs after {opts['warmups']} warm-ups each:\n")
    print("| Horizon | Span | Days | Wall, median | Peak | Instructions |")
    print("| --- | --- | --- | --- | --- | --- |")
    for key, words, start, end, days, wall, peak, ir in measured:
        print(f"| {key}: `{words.split()[0]}` | {start} .. {end} | {days} | {seconds(wall)} | "
              f"{megabytes(peak) if peak is not None else 'unreported'} | "
              f"{f'{ir / 1e9:.2f} G' if ir is not None else 'unreported'} |")
    (_, _, _, _, d1, w1, p1, i1), (_, _, _, _, d2, w2, p2, i2) = measured
    span = d2 / d1
    print(f"\ngrowth: the span x{span:.2f}; time {ratio(w1, w2, span)}; peak {ratio(p1, p2, span)}; "
          f"instructions {ratio(i1, i2, span)}; linear in the span is x{span:.2f}, exponent 1")
    return 0


# -- the commands --------------------------------------------------------------------------------------------------


def options(args):
    """--runs N (or a load row's --runs NAME,…), --warmups N, --bin PATH, --quick out of args;
    the rest are ids."""
    opts = {"runs": 10, "warmups": 2, "bin": None, "only": None, "quick": False}
    ids = []
    it = iter(args)
    for a in it:
        if a == "--runs":
            value = next(it, "0")
            if value.isdigit():
                opts["runs"] = int(value)
            else:
                opts["only"] = [n for n in value.split(",") if n]
        elif a == "--warmups":
            opts["warmups"] = int(next(it, "0"))
        elif a == "--bin":
            opts["bin"] = next(it, None)
        elif a == "--quick":
            opts["quick"] = True
        else:
            ids.append(a)
    if opts["runs"] < 1 or opts["warmups"] < 0:
        sys.exit("perf: --runs is at least 1 and --warmups at least 0")
    return opts, ids


def runs_by_name(selected, opts):
    """--runs NAME,… and --quick are a load row's: refused beside any other row, and a name
    that is no run of the rows asked for is refused."""
    if (opts["only"] or opts["quick"]) and not all(r["recipe"].startswith("load:") for r in selected):
        sys.exit("perf: --runs NAME,… and --quick take load rows (`load:`) only")
    if opts["only"]:
        names = {x[0] for r in selected for x in load_runs(r)}
        unknown = [n for n in opts["only"] if n not in names]
        if unknown:
            sys.exit(f"perf: --runs: {' '.join(unknown)} is no run of {' '.join(r['id'] for r in selected)} "
                     f"in tests/m4/load/runs.tsv")


def cmd_time(selected, opts):
    runs_by_name(selected, opts)
    print(fingerprint())
    if not set_at_s():
        print("perf: every number is proposed until S (tools/kernel.toml): misses are reported, not failed")
    runnable = any(steps(r) or r["recipe"].startswith("load:") for r in selected)
    binary = opts["bin"] or (build(quick=opts["quick"]) if runnable else None)
    failed = False
    for r in selected:
        line, bad = time_row(r, binary, opts["runs"], opts["warmups"], opts["only"])
        print(line)
        failed = failed or bad
    return 1 if failed else 0


def stats_of(output):
    """The counters' line, `stats: key=value ...`, from a run's output."""
    for line in output.decode("utf-8", "replace").split("\n"):
        if line.startswith("stats: "):
            return {k: int(v) for k, v in (w.split("=", 1) for w in line[len("stats: "):].split()) if v.isdigit()}
    return None


def load_counts(r, binary, only):
    """A load row's counts: each run's `stats:` line, a line each, and the row's against its
    count budgets (each run's own); whether it failed."""
    over, shown = [], []
    for name, argv, cwd, _ in load_runs(r, only):
        try:
            done = subprocess.run([binary, *argv, "--stats"], cwd=cwd, capture_output=True, timeout=TIMEOUT_S,
                                  check=False)
        except subprocess.TimeoutExpired:
            print(f"  {r['id']} {name}: killed at {TIMEOUT_S:.0f} s, no counts")
            over.append(f"{name} killed at {TIMEOUT_S:.0f} s")
            continue
        stats = stats_of(done.stdout + done.stderr)
        if stats is None:
            print(f"  {r['id']} {name}: no `stats:` line (exit {done.returncode})")
            over.append(f"{name}: no `stats:` line (D4-8)")
            continue
        print(f"  {r['id']} {name}: " + ", ".join(f"{k} {stats.get(k, 0)}" for k in COUNT_KEYS)
              + f" (exit {done.returncode})")
        shown.append(name)
        over += [f"{name} {k} {stats.get(k, 0):,} > {v:,}".replace(",", " ")
                 for k, v in r.get("counts", {}).items() if stats.get(k, 0) > v]
    if over:
        print(f"COUNT {r['id']} " + "; ".join(over))
        return True
    print(f"ok {r['id']} counts(1w): {len(shown)} runs (budget: {budget_text(r)}, each run)")
    return False


def cmd_counts(selected, opts):
    runs_by_name(selected, opts)
    binary = opts["bin"] or build(features=("counters",))
    failed = False
    for r in selected:
        if r["recipe"].startswith("load:"):
            failed = load_counts(r, binary, opts["only"]) and set_at_s() or failed
            continue
        plan = recipe(r)
        if plan is None:
            timed_only = " (a turn of several processes, timed only)" if r["recipe"].startswith("turn:") else ""
            print(f"{r['id']}: {r['recipe']}{timed_only}")
            continue
        argv, cwd = plan
        done = subprocess.run([binary, *argv, "--stats"], cwd=cwd, capture_output=True, timeout=TIMEOUT_S, check=False)
        stats = stats_of(done.stdout + done.stderr)
        if stats is None:
            print(f"COUNT {r['id']}: no `stats:` line (exit {done.returncode}): the counters build prints one (D4-8)")
            failed = True
            continue
        over = [f"{k} {stats.get(k, 0):,} > {v:,}".replace(",", " ")
                for k, v in r.get("counts", {}).items() if stats.get(k, 0) > v]
        shown = ", ".join(f"{k} {stats.get(k, 0)}" for k in COUNT_KEYS)
        if over:
            print(f"COUNT {r['id']} " + "; ".join(over) + f" (exit {done.returncode})")
            failed = failed or set_at_s()
        else:
            print(f"ok {r['id']} counts(1w): {shown} (exit {done.returncode}; budget: {budget_text(r)})")
    return 1 if failed else 0


def cmd_profile(r, opts):
    runs_by_name([r], opts)
    plan = recipe(r)
    if r["recipe"].startswith("load:"):
        runs = load_runs(r, opts["only"])
        if len(runs) != 1:
            sys.exit(f"perf: {r['id']} is a load row of {len(runs)} runs: name one with --runs NAME")
        plan = (runs[0][1], runs[0][2])
    if plan is None and r["recipe"].startswith("turn:"):
        sys.exit(f"perf: {r['id']} is a turn of several processes, timed only: {r['recipe']}")
    if plan is None:
        sys.exit(f"perf: {r['id']} has nothing to run: {r['recipe']}")
    if platform.system() != "Darwin":
        sys.exit("perf: on Linux, `perf record -e cpu-clock -g -- <bin>` then `perf report --stdio --no-children` "
                 "(BUILDER §13.5)")
    binary = opts["bin"] or build(quick=opts["quick"])
    argv, cwd = plan
    os.makedirs(OUT, exist_ok=True)
    profile = os.path.join(OUT, f"{r['id']}.json.gz")
    done = subprocess.run(["samply", "record", "--save-only", "--unstable-presymbolicate", "-o", profile, "--",
                           binary, *argv], cwd=cwd, capture_output=True, text=True, check=False)
    if not os.path.exists(profile):
        sys.exit(f"perf: samply recorded nothing (exit {done.returncode}):\n{done.stderr[-2000:]}")
    return subprocess.run([sys.executable, "-I", "-B", os.path.join(TOOLS, "samply_summary.py"), profile],
                          check=False).returncode


def cmd_limits(bench):
    found = [r for r in rows() if r["recipe"] == f"bench:{bench}"]
    if not found:
        sys.exit(f"perf: no row runs bench {bench}")
    if "instructions" in found[0]:
        print(f"--callgrind-limits=ir={found[0]['instructions']}")
    return 0


def main(argv):
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__)
        return 0 if argv else 2
    command, opts_ids = argv[0], options(argv[1:])
    opts, ids = opts_ids
    if command == "table":
        for r in rows():
            print(f"{r['id']:<7} {r['group']}  {budget_text(r)}  [{r['blocks']}]  {r['recipe']}")
        return 0
    if command == "fingerprint":
        print(fingerprint())
        return 0
    if command == "all":
        return cmd_time(rows(), opts)
    if command == "row" and ids:
        return cmd_time([row_of(i) for i in ids], opts)
    if command == "counts" and ids:
        return cmd_counts([row_of(i) for i in ids], opts)
    if command == "profile" and len(ids) == 1:
        return cmd_profile(row_of(ids[0]), opts)
    if command == "limits" and len(ids) == 1:
        return cmd_limits(ids[0])
    if command == "growth" and not ids:
        return cmd_growth(opts)
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
