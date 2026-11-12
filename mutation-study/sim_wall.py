#!/usr/bin/env python3
"""Wall time of the method's CI run under other shardings, from its measured per-mutant costs.

usage: sim_wall.py CI_DIR [--trials 2000] [--o1 2.9 --o1-build 15]
CI_DIR holds study-shard-<i>/ (results.jsonl and NNNN.log). Each shard's runner speed is
the median, over the tests it ran, of the test's time there over its median in all shards
(1.0 = the median runner; above 1 slower). A mutant's cost on the median runner is its
measured build + test over its shard's speed, plus 0.3 s of driver overhead.
Simulated, with runner speeds drawn from the 24 measured ones:
  - static: N shards balanced by cost (longest first onto the least loaded), 20 at once;
  - queue: N shards, the matrix in order, GitHub starting a queued shard when a job ends.
Each job adds JOB min of setup; the run adds COV min of coverage before the shards.
--o1 F --o1-build S: the survivors' tests run F times faster and their build S s longer
(the kernel at opt-level 1, option E).
--scale K --beta B: mutants x K, survivors' and caught tests x K, builds x K**B (project.py).
"""
import json
import os
import random
import re
import statistics
import sys

LINE = re.compile(r"^\s+(PASS|FAIL)\s+\[\s*([\d.]+)s\]\s+\(\s*\d+/\d+\)\s+(\S+\s+\S+)")
COV, JOB, CONC = 7.7, 0.5, 20


def arg(a, name, default):
    return type(default)(a[a.index(name) + 1]) if name in a else default


def load(ci):
    shards = sorted(int(f.split("-")[-1]) for f in os.listdir(ci) if f.startswith("study-shard-"))
    times, rows = {}, {}
    for i in shards:
        d = os.path.join(ci, f"study-shard-{i}")
        rows[i] = [json.loads(l) for l in open(os.path.join(d, "results.jsonl"))]
        for g in os.listdir(d):
            if g.endswith(".log"):
                for line in open(os.path.join(d, g), errors="replace"):
                    m = LINE.match(line)
                    if m and m.group(1) == "PASS" and float(m.group(2)) > 0.5:
                        times.setdefault(m.group(3), {}).setdefault(i, []).append(float(m.group(2)))
    med = {t: statistics.median(x for v in per.values() for x in v) for t, per in times.items()}
    speed = {}
    for i in shards:
        r = [statistics.median(per[i]) / med[t] for t, per in times.items() if i in per]
        speed[i] = statistics.median(r)
    return rows, speed


def lpt(costs, n):
    bins = [[0.0, k] for k in range(n)]
    for c in sorted(costs, reverse=True):
        b = min(bins)
        b[0] += c
    return sorted((b[0] for b in bins), reverse=True)


def queue(loads, speeds):
    """GitHub: CONC jobs at once, the matrix in order, a queued job starts when one ends"""
    free = [0.0] * CONC
    end = 0.0
    for load, s in zip(loads, speeds):
        free.sort()
        start = free[0]
        free[0] = start + JOB + load / 60 * s
        end = max(end, free[0])
    return end


def main():
    a = sys.argv[1:]
    ci = a[0]
    trials = arg(a, "--trials", 2000)
    o1, o1b = arg(a, "--o1", 1.0), arg(a, "--o1-build", 0.0)
    k, beta = arg(a, "--scale", 1.0), arg(a, "--beta", 0.75)
    rows, speed = load(ci)
    print("runner speeds:", " ".join(f"{speed[i]:.2f}" for i in sorted(speed)))
    sp = sorted(speed.values())
    print(f"   min {sp[0]:.2f} median {statistics.median(sp):.2f} max {sp[-1]:.2f}")
    costs, check = [], []
    for i, rs in rows.items():
        tot = 0.0
        for r in rs:
            b, t = r.get("build", 0) / speed[i], r.get("test", 0) / speed[i]
            if r["summary"] == "MissedMutant" and o1 > 1:
                b, t = b + o1b, t / o1
            c = b * k ** beta + t * k + 0.3
            tot += r.get("build", 0) + r.get("test", 0)
            costs.extend([c] * (int(k) if k >= 1 else 1))
        check.append(tot / 60)
    total = sum(costs) / 60
    print(f"mutants {len(costs)}; work on the median runner {total:.0f} runner-min "
          f"(measured on the run's own runners {sum(check):.0f})")
    rnd = random.Random(20261112)
    print(f"\nwall from the run's start, coverage {COV} min first; {trials} draws of runner speeds")
    print(f"{'sharding':28} {'shards':>6} {'median':>7} {'p90':>6} {'worst draw':>10}")
    for label, n in [("round-robin-like (random)", 24), ("static, balanced", 20), ("static, balanced", 24),
                     ("queue, balanced", 40), ("queue, balanced", 60), ("queue, balanced", 100)]:
        if label.startswith("round"):
            res = []
            for _ in range(trials):
                cs = costs[:]
                rnd.shuffle(cs)
                loads = [sum(cs[j::n]) for j in range(n)]
                res.append(COV + queue(loads, [rnd.choice(sp) for _ in range(n)]))
        else:
            loads = lpt(costs, n)
            res = [COV + queue(loads, [rnd.choice(sp) for _ in range(n)]) for _ in range(trials)]
        res.sort()
        print(f"{label:28} {n:6} {res[len(res)//2]:7.0f} {res[int(len(res)*0.9)]:6.0f} {res[-1]:10.0f}")


if __name__ == "__main__":
    main()
