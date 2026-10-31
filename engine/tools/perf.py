"""perf: the budget rows on the reference Mac (BUILDER §13.3, §13.5; D4-1, D4-3, D4-5).

    engine/tools/perf table                       every row of tools/perf-budgets.toml, one line each
    engine/tools/perf fingerprint                 the machine's fingerprint (D4-3)
    engine/tools/perf row ID... [--runs N] [--warmups N] [--bin PATH]
                                                  time rows: 2 warm-ups, then the median of 10
    engine/tools/perf all [--runs N] [--warmups N] [--bin PATH]
                                                  every row: the runnable ones timed, the rest say what they wait for
    engine/tools/perf counts ID... [--bin PATH]   the counters build's --stats against the row's count budgets
    engine/tools/perf profile ID [--bin PATH]     samply's profile of one run, summarised as text (macOS)
    engine/tools/perf limits BENCH                the gungraun hard limit of a bench, as its command-line flag

Each timed run is the release binary under /usr/bin/time -l (macOS): wall time is this
script's clock around it, peak memory footprint and instructions retired are time's. A row
passes when its median wall time and its peak footprint are within budget. A run whose
resident size passes 8 GB is killed (D4-7). A fast wrong answer fails: after timing, the
output's SHA-256 is compared with T's expected digest (engine/tests/perf-digests.tsv, id and
digest per line), and a row with no digest yet is reported unjudged.

Until the lead sets the kernel at S (tools/kernel.toml), every number is proposed
(PERFORMANCE §8): perf reports misses and exits 0. After S a miss exits 1. A wrong answer
exits 1 always. Standard library only, like run.py.
"""

import csv
import hashlib
import os
import platform
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
OUT = os.path.join(ENGINE, "target", "perf")
CEILING_KB = 8 * 1024 * 1024  # D4-7: 8 GB on any run
TIMEOUT_S = 90.0  # the user's hard budget for one run
COUNT_KEYS = ("def_misses", "days_stepped")


def rows():
    with open(TABLE, "rb") as f:
        return tomllib.load(f)["row"]


def row_of(row_id):
    found = [r for r in rows() if r["id"] == row_id]
    if not found:
        sys.exit(f"perf: no row {row_id} in tools/perf-budgets.toml")
    return found[0]


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


def recipe(r):
    """(argv after the binary, cwd), as run.py runs the check, or None for a row that waits."""
    if not r["recipe"].startswith("check:"):
        return None
    c = check_row(r["recipe"][len("check:"):])
    cwd = os.path.join(ROOT, c["cwd"])
    files = [os.path.relpath(os.path.join(ROOT, f), cwd) for f in c["files"].split()]
    argv = ["run", *files, *c["options"].split()] + (["--ask", *c["asks"].split()] if c["asks"] else [])
    return argv, cwd


def digests():
    try:
        with open(DIGESTS, encoding="utf-8") as f:
            pairs = (line.rstrip("\n").split("\t") for line in f if line.strip() and not line.startswith("#"))
            return {p[0]: p[1] for p in pairs if len(p) == 2}
    except OSError:
        return {}


def build(features=()):
    """The release binary, or the counted one (debug assertions on, so the counters may exist: R-51)."""
    argv = ["cargo", "build", "--locked", "-p", "tally-cli"]
    if features:
        argv += ["--features", ",".join(features)]
    else:
        argv += ["--release"]
    done = subprocess.run(argv, cwd=ENGINE, env=check.clean_env(), capture_output=True, text=True, check=False)
    if done.returncode != 0:
        sys.exit(f"perf: {' '.join(argv)} failed:\n{done.stderr[-2000:]}")
    return os.path.join(ENGINE, "target", "debug" if features else "release", "tally")


# -- one run -------------------------------------------------------------------------------------------------------


def resident_kb(pid):
    """The resident size of pid and its children, in KB (the watchdog's approximation of the footprint)."""
    pids = [str(pid), *output_of(["pgrep", "-P", str(pid)]).split()]
    sizes = output_of(["ps", "-o", "rss=", "-p", ",".join(pids)]).split()
    return sum(int(s) for s in sizes if s.isdigit())


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
                        child.kill()
                        child.wait()
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


def time_row(r, binary, runs, warmups):
    """One line for one row, and whether it failed (a wrong answer, or a miss after S)."""
    plan = recipe(r)
    if plan is None and r["recipe"].startswith("bench:"):
        return f"{r['id']}: {r['recipe']}, its instructions under gungraun on Linux CI", False
    if plan is None:
        return f"{r['id']}: {r['recipe']}", False
    argv, cwd = plan
    results = []
    for _ in range(warmups + runs):
        one = timed(binary, argv, cwd)
        if one is None:
            return f"BUDGET {r['id']}: a run passed 8 GB or {TIMEOUT_S:.0f} s and was killed (D4-7)", True
        results.append(one)
    measured = results[warmups:]
    wall = statistics.median(m[0] for m in measured)
    footprints = [m[1] for m in measured if m[1] is not None]
    peak = max(footprints) if footprints else None
    instructions = [m[2] for m in measured if m[2] is not None]
    code, output = measured[-1][3], measured[-1][4]
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
    expected = digests().get(r["recipe"][len("check:"):])
    got = hashlib.sha256(output).hexdigest()
    if expected is None:
        notes.append(f"unjudged: no digest in engine/tests/perf-digests.tsv yet (exit {code})")
        wrong = False
    else:
        wrong = got != expected
        if wrong:
            misses.append(f"WRONG: the output's digest isn't T's (exit {code}): a fast wrong answer fails")
    if misses:
        return f"BUDGET {r['id']} " + "; ".join(misses + notes), wrong or set_at_s()
    return f"ok {r['id']} " + "; ".join(notes) + f" ({runs} runs; budget: {budget_text(r)})", False


# -- the commands --------------------------------------------------------------------------------------------------


def options(args):
    """--runs N, --warmups N, --bin PATH out of args; the rest are ids."""
    opts = {"runs": 10, "warmups": 2, "bin": None}
    ids = []
    it = iter(args)
    for a in it:
        if a in ("--runs", "--warmups"):
            opts[a[2:]] = int(next(it, "0"))
        elif a == "--bin":
            opts["bin"] = next(it, None)
        else:
            ids.append(a)
    if opts["runs"] < 1 or opts["warmups"] < 0:
        sys.exit("perf: --runs is at least 1 and --warmups at least 0")
    return opts, ids


def cmd_time(selected, opts):
    print(fingerprint())
    if not set_at_s():
        print("perf: every number is proposed until S (tools/kernel.toml): misses are reported, not failed")
    binary = opts["bin"] or (build() if any(recipe(r) for r in selected) else None)
    failed = False
    for r in selected:
        line, bad = time_row(r, binary, opts["runs"], opts["warmups"])
        print(line)
        failed = failed or bad
    return 1 if failed else 0


def stats_of(output):
    """The counters' line, `stats: key=value ...`, from a run's output."""
    for line in output.decode("utf-8", "replace").split("\n"):
        if line.startswith("stats: "):
            return {k: int(v) for k, v in (w.split("=", 1) for w in line[len("stats: "):].split()) if v.isdigit()}
    return None


def cmd_counts(selected, opts):
    binary = opts["bin"] or build(features=("counters",))
    failed = False
    for r in selected:
        plan = recipe(r)
        if plan is None:
            print(f"{r['id']}: {r['recipe']}")
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
    plan = recipe(r)
    if plan is None:
        sys.exit(f"perf: {r['id']} has nothing to run: {r['recipe']}")
    if platform.system() != "Darwin":
        sys.exit("perf: on Linux, `perf record -e cpu-clock -g -- <bin>` then `perf report --stdio --no-children` "
                 "(BUILDER §13.5)")
    binary = opts["bin"] or build()
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
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
