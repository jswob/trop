#!/usr/bin/env python3
"""Per-test function coverage of the kernel, from one LLVM profile per test.

usage: proc_cov.py PROF_DIR DEPS_DIR OUT.json [--jobs N]

PROF_DIR holds `<binary>@@<test>.<pid>.profraw` (cov-wrap.sh); DEPS_DIR is the
instrumented build's target/<profile>/deps. Writes {test key: {file: [[start, end], ...]}}:
for every function record of crates/tally/src whose entry count is above zero, the line
span of its regions. A mutant is covered by a test when one of these spans starts inside
the mutant's function span (closures inside it included).
"""
import concurrent.futures
import glob
import json
import os
import re
import subprocess
import sys

BIN = os.environ.get("LLVM_BIN") or os.path.expanduser(
    "~/.rustup/toolchains/1.99.1-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin")
KERNEL = "crates/tally/src/"


def binaries(deps):
    out = {}
    for p in glob.glob(os.path.join(deps, "*")):
        b = os.path.basename(p)
        if os.access(p, os.X_OK) and os.path.isfile(p) and "." not in b:
            out[re.sub(r"-[0-9a-f]{16}$", "", b)] = p
    return out


def one(raw, bins):
    base = os.path.basename(raw)
    binary, rest = base.split("@@", 1)
    test = rest.rsplit(".", 2)[0]
    prof = raw[:-len(".profraw")] + ".profdata"
    subprocess.run([f"{BIN}/llvm-profdata", "merge", "-sparse", raw, "-o", prof], check=True)
    exp = subprocess.run([f"{BIN}/llvm-cov", "export", "-format=text", "-skip-expansions",
                          "-instr-profile", prof, bins[binary]],
                         check=True, capture_output=True).stdout
    data = json.loads(exp)
    os.remove(prof)
    seg = {}
    for fl in data["data"][0]["files"]:
        if KERNEL not in fl["filename"]:
            continue
        name = KERNEL + fl["filename"].split(KERNEL, 1)[1]
        ss = fl["segments"]
        iv, nc = [], []
        for i, sg in enumerate(ss):
            line, col, count, has = sg[0], sg[1], sg[2], sg[3]
            nxt = ss[i + 1] if i + 1 < len(ss) else [10**9, 0]
            if has and count > 0:
                iv.append([line, col, nxt[0], nxt[1]])
            elif not has:
                nc.append([line, col, nxt[0], nxt[1]])
        seg[name] = iv
        seg["nocount:" + name] = nc
    cov = {}
    for f in data["data"][0]["functions"]:
        if f["count"] <= 0:
            continue
        names = [n for n in f["filenames"] if KERNEL in n]
        if not names:
            continue
        fname = KERNEL + names[0].split(KERNEL, 1)[1]
        regs = [r for r in f["regions"] if f["filenames"][r[5]].endswith(fname[len(KERNEL):])]
        if not regs:
            continue
        cov.setdefault(fname, []).append([min(r[0] for r in regs), max(r[2] for r in regs)])
    if "--fn-only" in sys.argv:
        seg = {}
    return f"{binary} {test}", {"fn": cov, "seg": seg}


def main():
    prof_dir, deps, out = sys.argv[1:4]
    jobs = int(sys.argv[sys.argv.index("--jobs") + 1]) if "--jobs" in sys.argv else 4
    bins = binaries(deps)
    raws = sorted(glob.glob(os.path.join(prof_dir, "*.profraw")))
    res = {}
    with concurrent.futures.ThreadPoolExecutor(jobs) as ex:
        for key, cov in ex.map(lambda r: one(r, bins), raws):
            if key in res:
                for part in ("fn", "seg"):
                    for k, v in cov[part].items():
                        res[key][part].setdefault(k, []).extend(v)
            else:
                res[key] = cov
    for v in res.values():
        for part in ("fn", "seg"):
            for k in v[part]:
                v[part][k] = sorted(set(map(tuple, v[part][k])))
    json.dump(res, open(out, "w"))
    print(f"{len(raws)} profiles, {len(res)} tests")


if __name__ == "__main__":
    main()
