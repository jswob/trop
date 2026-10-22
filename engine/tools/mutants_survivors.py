"""mutants-survivors: the surviving mutants are the listed ones (R-87; DONE gate 8).

    engine/tools/mutants-survivors DIR...

Each DIR is a cargo-mutants output directory (holding mutants.out/, or mutants.out
itself); the CI shards' directories are read together. A mutant survives when it is
missed or times out (R-88). It prints the score, caught / (caught + missed + timed out),
and the unviable count, which the score leaves out.

Every survivor must be a row of engine/MUTANTS-SURVIVORS.md:

    | crates/tally/src/x.rs | parse | replace parse -> bool with true | R-21: why it lives |

by file, function and mutation (cargo-mutants' description, without the line and
column, which move), with a reason naming its rule. It fails on a survivor that isn't
listed, a listed one that is now caught or unviable, a listed one no run produced, and
a row without a reason, and on a score under DONE gate 8's 90 %. Until the lead sets
the kernel at S (tools/kernel.toml), it reports the same problems and exits 0, since the
stubs' mutants test nothing (R-85).
"""

import json
import os
import re
import sys
import tomllib

TOOLS = os.path.dirname(os.path.realpath(__file__))
ENGINE = os.path.dirname(TOOLS)
SURVIVORS = os.path.join(ENGINE, "MUTANTS-SURVIVORS.md")
KERNEL = os.path.join(TOOLS, "kernel.toml")
REASON = re.compile(r"^R-([1-9][0-9]*): \S")
POSITION = re.compile(r"^(?P<file>[^:]+):[0-9]+:[0-9]+: (?P<mutation>.+)$")
SURVIVED = {"MissedMutant": "missed", "Timeout": "timed out"}
GATE = 90  # DONE gate 8: at least 90 % of the kernel's mutants killed


def outcomes_files(dirs):
    found = []
    for d in dirs:
        for path in (os.path.join(d, "mutants.out", "outcomes.json"), os.path.join(d, "outcomes.json")):
            if os.path.isfile(path):
                found.append(path)
                break
        else:
            raise SystemExit(f"mutants-survivors: no outcomes.json in {d}")
    return found


def mutants(paths):
    """{(file, function, mutation): summary} over every run, plus the counts."""
    seen, counts = {}, {"caught": 0, "missed": 0, "timeout": 0, "unviable": 0}
    for path in paths:
        with open(path, encoding="utf-8") as f:
            data = json.load(f)
        for outcome in data["outcomes"]:
            m = outcome["scenario"].get("Mutant") if isinstance(outcome["scenario"], dict) else None
            if m is None:
                continue  # the baseline
            pos = POSITION.match(m["name"])
            mutation = pos["mutation"] if pos else m["name"]
            function = (m.get("function") or {}).get("function_name") or "-"
            key = (m["file"], function, mutation)
            summary = outcome["summary"]
            seen[key] = summary
            counts[{"CaughtMutant": "caught", "MissedMutant": "missed", "Timeout": "timeout",
                     "Unviable": "unviable"}.get(summary, "unviable")] += 1
    return seen, counts


def listed(problems):
    """{(file, function, mutation): reason} from the table's rows."""
    rows = {}
    if not os.path.isfile(SURVIVORS):
        problems.append("engine/MUTANTS-SURVIVORS.md is missing (R-87)")
        return rows
    with open(SURVIVORS, encoding="utf-8") as f:
        for n, line in enumerate(f, 1):
            cells = [c.strip() for c in line.strip().strip("|").split("|")]
            if not line.startswith("|") or len(cells) != 4 or cells[0] in ("file", "") or set(cells[0]) <= set("-: "):
                continue
            file, function, mutation, reason = cells
            if not REASON.match(reason):
                problems.append(f"engine/MUTANTS-SURVIVORS.md:{n}: the reason must start with its rule, 'R-<n>: ' (R-87)")
            rows[(file, function, mutation)] = reason
    return rows


def kernel_set():
    try:
        with open(KERNEL, "rb") as f:
            return tomllib.load(f).get("set_at_s") is True
    except OSError:
        return False


def main(argv):
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__.strip())
        return 0 if argv else 64
    seen, counts = mutants(outcomes_files(argv))
    problems = []
    rows = listed(problems)
    for key, summary in sorted(seen.items()):
        if summary in SURVIVED and key not in rows:
            problems.append(f"{key[0]}: {key[2]} ({SURVIVED[summary]}) is not in MUTANTS-SURVIVORS.md (R-87)")
    for key in sorted(rows):
        summary = seen.get(key)
        if summary is None:
            problems.append(f"{key[0]}: {key[2]} is listed in MUTANTS-SURVIVORS.md and no run produced it: remove the row (R-87)")
        elif summary not in SURVIVED:
            state = "caught" if summary == "CaughtMutant" else "unviable"
            problems.append(f"{key[0]}: {key[2]} is listed in MUTANTS-SURVIVORS.md and is now {state}: remove the row (R-87)")
    scored = counts["caught"] + counts["missed"] + counts["timeout"]
    score = f"{100 * counts['caught'] / scored:.1f}%" if scored else "n/a"
    print(f"mutants-survivors: score {score} = caught {counts['caught']} / (caught {counts['caught']} + missed "
          f"{counts['missed']} + timed out {counts['timeout']}); unviable {counts['unviable']}, not counted")
    if not scored:
        problems.append("no mutant was caught, missed or timed out: nothing was scored")
    elif 100 * counts["caught"] < GATE * scored:
        problems.append(f"the score {score} is under DONE gate 8's {GATE} %")
    for p in problems:
        print(f"mutants-survivors: {p}")
    if not problems:
        print("mutants-survivors: every survivor is listed, with its reason")
        return 0
    if not kernel_set():
        print("mutants-survivors: notice: the kernel isn't set yet (the lead sets it at S, R-85); reported, not failed")
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
