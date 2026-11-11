#!/usr/bin/env python3
"""Prototype of a coverage-selected gate-8 run (the study's method, for measurement).

cargo-mutants still makes the mutants: their names, spans and diffs come from
`cargo mutants --list --json` (the same set, the same text). For each mutant this:
  1. selects T's tests whose unmutated run enters the mutant's function (COV.json,
     proc_cov.py's function spans), and their test targets;
  2. applies the mutant's diff in place;
  3. builds as cargo-mutants does (`cargo nextest run --no-run`, the mutants profile,
     `--cap-lints=warn`), only the selected targets (the runner's lib alone when none);
  4. runs the selected tests in the nextest `mutants` profile (fail-fast, the memory cap),
     killers first (a tool config of nextest priorities), `--no-tests=pass`;
  5. restores the file and classifies like cargo-mutants: build failed -> Unviable,
     test phase over --timeout -> Timeout, tests passed -> MissedMutant, else CaughtMutant.

usage: gate8_select.py --engine ENGINE/crates --mutants LIST.json --cov COV.json --names NAMES.txt
                       --out DIR [--prio PRIO.json] [--timeout 600] [--build-timeout 600]
"""
import argparse
import collections
import json
import os
import re
import signal
import subprocess
import sys
import time

ENV = {"CARGO_ENCODED_RUSTFLAGS": "--cap-lints=warn", "INSIDE_CARGO_MUTANTS": "true",
       "PROPTEST_RNG_SEED": "20261022", "PROPTEST_DISABLE_FAILURE_PERSISTENCE": "1"}
FEATURES = "--features=tally-runner/fault-injection"
LINE = re.compile(r"^\s+(PASS|FAIL|SIGKILL|SIGTERM|SIGSEGV|SIGABRT|TIMEOUT|ABORT|LEAK)\s+\[\s*([\d.]+)s\]\s+\(\s*\d+/\d+\)\s+(\S+)\s+(\S+)")


def apply(text, diff):
    """The mutated text: the diff's one hunk, checked against the original lines."""
    lines = text.split("\n")
    out, pos = [], 0
    hunks = re.split(r"^@@ -(\d+)(?:,\d+)? \+\d+(?:,\d+)? @@.*$", diff, flags=re.M)
    for i in range(1, len(hunks), 2):
        start = int(hunks[i]) - 1
        body = hunks[i + 1].split("\n")[1:]
        out.extend(lines[pos:start])
        pos = start
        for b in body:
            if b.startswith("\\") or b == "":
                continue
            tag, s = b[0], b[1:]
            if tag in " -":
                if lines[pos] != s:
                    raise ValueError(f"diff doesn't match line {pos + 1}: {s!r} != {lines[pos]!r}")
                pos += 1
                if tag == " ":
                    out.append(s)
            elif tag == "+":
                out.append(s)
    out.extend(lines[pos:])
    return "\n".join(out)


def run(argv, cwd, timeout, log):
    t0 = time.time()
    env = dict(os.environ, **ENV)
    with open(log, "a") as fh:
        fh.write(f"\n*** {' '.join(argv)}\n")
        fh.flush()
        p = subprocess.Popen(argv, cwd=cwd, env=env, stdout=fh, stderr=subprocess.STDOUT,
                             start_new_session=True)
        try:
            code = p.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(p.pid, signal.SIGKILL)
            p.wait()
            code = "timeout"
        fh.write(f"*** result: {code}\n")
    return code, time.time() - t0


def target_args(binaries):
    out = [FEATURES]
    for b in sorted(binaries):
        out.append("--lib" if b == "tally_runner" else f"--test={b}")
    return out or ["--lib"]


def filterset(tests):
    parts = []
    for t in tests:
        b, name = t.split(" ", 1)
        bid = "tally-runner" if b == "tally_runner" else f"tally-runner::{b}"
        parts.append(f"(binary_id({bid}) & test(={name}))")
    return " | ".join(parts) if parts else "none()"


def main():
    ap = argparse.ArgumentParser()
    for a in ("--engine", "--mutants", "--cov", "--out"):
        ap.add_argument(a, required=True)
    ap.add_argument("--names", help="the mutants to run, one name a line")
    ap.add_argument("--shard", help="i/n: the list's mutants i, i+n, ... (cargo-mutants' round-robin)")
    ap.add_argument("--all-tests", action="store_true",
                    help="a full run's emulation: every test of every target, no selection")
    ap.add_argument("--prio")
    ap.add_argument("--timeout", type=float, default=600)
    ap.add_argument("--build-timeout", type=float, default=600)
    o = ap.parse_args()
    o.out = os.path.abspath(o.out)
    os.makedirs(o.out, exist_ok=True)
    muts = {m["name"]: m for m in json.load(open(o.mutants))}
    cov = json.load(open(o.cov))
    # cov keys use cov-wrap.sh's names: `<binary> tests__x` with `::` as `__`; nextest
    # needs the real names: the coverage file name maps back through the log's names
    real = json.load(open(o.prio)).get("real", {}) if o.prio else {}
    prio = json.load(open(o.prio))["order"] if o.prio else []
    killers = json.load(open(o.prio)).get("killers", {}) if o.prio else {}
    rank = {t: i for i, t in enumerate(prio)}
    idx = collections.defaultdict(list)
    for t, parts in cov.items():
        for f, spans in parts["fn"].items():
            for s, _e in spans:
                idx[f].append((s, t))
    dirty = subprocess.run(["git", "status", "--porcelain", "--", "."], cwd=o.engine,
                           capture_output=True, text=True).stdout.strip()
    if dirty:
        raise SystemExit(f"the crates aren't clean, a mutant may be left in place:\n{dirty}")
    signal.signal(signal.SIGTERM, lambda *_: (_ for _ in ()).throw(SystemExit("SIGTERM")))
    if o.names:
        names = [l.rstrip("\n") for l in open(o.names) if l.strip()]
    else:
        i, n = map(int, o.shard.split("/"))
        names = [m["name"] for k, m in enumerate(json.load(open(o.mutants))) if k % n == i]
    every = sorted(real.values()) if o.all_tests else None
    results = []
    resf = os.path.join(o.out, "results.jsonl")
    done = set()
    if os.path.exists(resf):
        done = {json.loads(l)["name"] for l in open(resf)}
    for n, name in enumerate(names):
        if name in done:
            continue
        m = muts[name]
        f = m["file"]
        fs = (m.get("function") or {}).get("span") or m["span"]
        lo, hi = fs["start"]["line"], fs["end"]["line"]
        S = sorted({t for s, t in idx.get(f, []) if lo <= s <= hi})
        tests = every if every is not None else [real.get(t, t) for t in S]
        # killers first: this mutant's last killer, then the tests by kills per second
        tests.sort(key=lambda t: (0 if t == killers.get(name) else 1, rank.get(t, 1e9), t))
        bins = {t.split(" ", 1)[0] for t in tests}
        path = os.path.join(o.engine, f[len("crates/"):])  # --engine is engine/crates
        orig = open(path).read()
        log = os.path.join(o.out, f"{n:04d}.log")
        open(log, "w").write(f"{name}\nselected {len(tests)}: {tests}\n")
        rec = {"name": name, "selected": len(tests), "targets": sorted(bins)}
        try:
            open(path, "w").write(apply(orig, m["diff"]))
            base = ["cargo", "nextest", "run", "--cargo-profile=mutants",
                    "--package=tally-runner@0.1.0", "--locked"] + target_args(bins)
            code, bt = run(base[:3] + ["--no-run"] + base[3:], o.engine, o.build_timeout, log)
            rec["build"] = round(bt, 1)
            if code != 0:
                rec["summary"] = "Timeout" if code == "timeout" else "Unviable"
            else:
                cfg = os.path.join(o.out, f"{n:04d}.toml")
                with open(cfg, "w") as fh:
                    for i, t in enumerate([] if every is not None else tests[:200]):
                        fh.write(f"[[profile.mutants.overrides]]\nfilter = '{filterset([t])}'\n"
                                 f"priority = {100 - i}\n\n")
                code, tt = run(base + ["--profile=mutants", "--no-tests=pass",
                                       f"--tool-config-file=gate8:{cfg}", "-E", filterset(tests)],
                               o.engine, o.timeout, log)
                rec["test"] = round(tt, 1)
                # nextest: 0 all passed, 100 a test failed (a slow-timeout kill and the
                # memory cap included); any other exit is the run's error, never a verdict
                if code not in ("timeout", 0, 100):
                    raise SystemExit(f"nextest exited {code} on {name}: see {log}")
                rec["summary"] = ("Timeout" if code == "timeout" else
                                  "MissedMutant" if code == 0 else "CaughtMutant")
                fails = []
                for line in open(log):
                    x = LINE.match(line)
                    if x and x.group(1) not in ("PASS", "SIGKILL", "SIGTERM"):
                        fails.append([x.group(1), f"{x.group(3)} {x.group(4)}"])
                rec["fails"] = fails[:5]
        finally:
            open(path, "w").write(orig)
        with open(resf, "a") as fh:
            fh.write(json.dumps(rec) + "\n")
        print(f"{n + 1}/{len(names)} {rec['summary']:12} b {rec.get('build')} t {rec.get('test')} "
              f"sel {len(tests)} {name[:90]}", flush=True)


if __name__ == "__main__":
    main()
