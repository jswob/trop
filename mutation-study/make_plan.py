#!/usr/bin/env python3
"""A gate-8 plan from an earlier run of the method: shards balanced by cost, survivors at O1.

usage: make_plan.py CI_DIR LIST.json SHARDS OUT.json [--o1 2.9 --o1-build 15]
Each mutant's predicted cost is its time in CI_DIR on the median runner (sim_wall.py's
runner speeds); a mutant the earlier run didn't judge gets the median cost. The mutants
the earlier run missed are built with the kernel at opt-level 1 ("o1"), their predicted
cost adjusted. Shards are filled longest first onto the least loaded and numbered by load,
largest first, so GitHub's queue starts the longest first. Writes
{"shards": [[name, ...], ...], "o1": [name, ...], "load": [minutes, ...]}.
"""
import json
import statistics
import sys

from sim_wall import arg, load


def main():
    a = sys.argv[1:]
    ci, lst, n, out = a[0], a[1], int(a[2]), a[3]
    o1, o1b = arg(a, "--o1", 2.9), arg(a, "--o1-build", 15.0)
    rows, speed = load(ci)
    cost, missed = {}, []
    for i, rs in rows.items():
        for r in rs:
            b, t = r.get("build", 0) / speed[i], r.get("test", 0) / speed[i]
            if r["summary"] == "MissedMutant":
                missed.append(r["name"])
                b, t = b + o1b, t / o1
            cost[r["name"]] = b + t + 0.3
    names = [m["name"] for m in json.load(open(lst))]
    med = statistics.median(cost.values())
    bins = [[0.0, k, []] for k in range(n)]
    for name in sorted(names, key=lambda x: -cost.get(x, med)):
        b = min(bins, key=lambda x: (x[0], x[1]))
        b[0] += cost.get(name, med)
        b[2].append(name)
    bins.sort(key=lambda x: -x[0])
    plan = {"shards": [b[2] for b in bins], "o1": sorted(set(missed) & set(names)),
            "load": [round(b[0] / 60, 1) for b in bins]}
    json.dump(plan, open(out, "w"), indent=0)
    print(f"{len(names)} mutants in {n} shards; o1 {len(plan['o1'])}; "
          f"load min {min(plan['load'])} max {max(plan['load'])} sum {sum(plan['load']):.0f} min")


if __name__ == "__main__":
    main()
