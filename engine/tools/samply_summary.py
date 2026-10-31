"""samply-summary: a samply profile as text, the top 30 functions by self and by inclusive time
(BUILDER §13.5; D4-5).

    python3 -I -B engine/tools/samply_summary.py PROFILE.json.gz [--top N]

PROFILE is what `samply record --save-only --unstable-presymbolicate -o PROFILE.json.gz` writes
(engine/tools/perf profile runs it): the Firefox Profiler's processed format, with the symbols
in PROFILE.json.syms.json beside it. A function samply left as an address is named from that
file's symbol table, by its library and offset. Self time is the samples whose innermost frame
is the function; inclusive time is the samples with the function anywhere on the stack, counted
once per sample. Standard library only.
"""

import bisect
import collections
import gzip
import json
import os
import sys


def load(path):
    with gzip.open(path) if path.endswith(".gz") else open(path, "rb") as f:
        profile = json.load(f)
    base = path[: -len(".gz")] if path.endswith(".gz") else path
    syms_path = base + ".syms.json"
    syms = None
    if os.path.exists(syms_path):
        with open(syms_path, encoding="utf-8") as f:
            syms = json.load(f)
    return profile, syms


class Symbols:
    """Each library's symbol table from the .syms.json sidecar, looked up by offset."""

    def __init__(self, syms):
        self.tables = {}
        if syms is None:
            return
        strings = syms["string_table"]
        for lib in syms["data"]:
            table = sorted((s["rva"], s["rva"] + s["size"], strings[s["symbol"]]) for s in lib["symbol_table"])
            entry = ([t[0] for t in table], table)
            self.tables[lib["code_id"].upper()] = entry
            self.tables[lib["debug_name"]] = entry

    def name(self, lib, offset):
        entry = self.tables.get((lib.get("codeId") or "").upper()) or self.tables.get(lib.get("debugName"))
        if entry is None:
            return None
        starts, table = entry
        i = bisect.bisect_right(starts, offset) - 1
        if i >= 0 and table[i][0] <= offset < table[i][1]:
            return table[i][2]
        return None


def func_names(profile, thread, symbols):
    funcs, strings, resources = thread["funcTable"], thread["stringArray"], thread["resourceTable"]
    names = []
    for i in range(funcs["length"]):
        name = strings[funcs["name"][i]]
        resource = funcs["resource"][i]
        lib_name = ""
        if resource is not None and resource >= 0:
            lib_index = resources["lib"][resource]
            lib_name = strings[resources["name"][resource]]
            if name.startswith("0x") and lib_index is not None:
                found = symbols.name(profile["libs"][lib_index], int(name, 16))
                name = found or name
        names.append(f"{name}  [{lib_name}]" if lib_name else name)
    return names


def tally(profile, symbols):
    """(self counts, inclusive counts, samples) over every thread, by function name."""
    own, inclusive, total = collections.Counter(), collections.Counter(), 0
    for thread in profile["threads"]:
        samples, stacks, frames = thread["samples"], thread["stackTable"], thread["frameTable"]
        names = func_names(profile, thread, symbols)
        weights = samples.get("weight") or [1] * samples["length"]
        for stack, weight in zip(samples["stack"], weights, strict=True):
            if stack is None:
                continue
            weight = weight or 1
            total += weight
            seen = set()
            leaf = True
            while stack is not None:
                name = names[frames["func"][stacks["frame"][stack]]]
                if leaf:
                    own[name] += weight
                    leaf = False
                if name not in seen:
                    inclusive[name] += weight
                    seen.add(name)
                stack = stacks["prefix"][stack]
    return own, inclusive, total


def main(argv):
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__)
        return 0 if argv else 2
    top = 30
    if "--top" in argv:
        top = int(argv[argv.index("--top") + 1])
    profile, syms = load(argv[0])
    own, inclusive, total = tally(profile, Symbols(syms))
    interval = profile["meta"].get("interval", 1.0)
    print(f"samply-summary: {argv[0]}: {total} samples at {interval} ms "
          f"({'symbols from the sidecar' if syms else 'no .syms.json: addresses stay addresses'})")
    if total == 0:
        print("  no samples: the run was shorter than one interval")
        return 0
    for title, counts in (("self", own), ("inclusive", inclusive)):
        print(f"top {top} by {title} time:")
        for name, n in counts.most_common(top):
            print(f"  {n * 100 / total:5.1f} %  {n * interval:9.1f} ms  {name}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
