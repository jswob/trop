"""mutants-survivors: the surviving mutants are the listed ones (R-87; DONE gate 8).

    engine/tools/mutants-survivors DIR...

Each DIR is a cargo-mutants output directory (holding mutants.out/, or mutants.out
itself); the CI shards' directories are read together. A mutant survives when it is
missed or times out (R-88). Every survivor must be a row of engine/MUTANTS-SURVIVORS.md:

    | crates/tally/src/x.rs | parse | replace parse -> bool with true | R-87: equivalent: why |

by file, function and mutation (cargo-mutants' description, without the line and
column, which move), with its reason in one of two forms, each with at least 20
characters after it (R-87, approved 2026-11-03):

    R-87: equivalent: <why>   a proven equivalent: no valid input tells it from the code
    R-87: missed: <why>       a test could tell it apart but can't yet (an input the
                              engine refuses as not built; refcore and the spec disagree)

The score is caught / (caught + missed + timed out - equivalent): a surviving mutant
whose row is `R-87: equivalent:` leaves the count; every other survivor stays counted
as missed. The equivalent count is of mutants, not rows: one row covers every mutant
with its key, such as the same mutation at two columns. Unviable mutants are reported,
not counted. It fails on a survivor that isn't listed, a listed one that is now caught
or unviable, a listed one no run produced, a row without a reason, a key listed twice,
and a score under DONE gate 8's 90 %. Until the lead sets
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
REASON = re.compile(r"^R-87: (?P<form>equivalent|missed): \S.{19,}$")
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
    """{(file, function, mutation): [summary, ...]} over every run, plus the counts.

    A key can hold several mutants: the same mutation at different columns."""
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
            seen.setdefault(key, []).append(summary)
            counts[{"CaughtMutant": "caught", "MissedMutant": "missed", "Timeout": "timeout",
                     "Unviable": "unviable"}.get(summary, "unviable")] += 1
    return seen, counts


def listed(problems):
    """{(file, function, mutation): form} from the table's rows: "equivalent", "missed",
    or None for a row without a reason."""
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
            form = REASON.match(reason)
            if not form:
                problems.append(f"engine/MUTANTS-SURVIVORS.md:{n}: the reason must say why the mutant is equivalent "
                                "or missed: 'R-87: equivalent: ' or 'R-87: missed: ' and 20 characters or more "
                                "(R-87, DONE gate 8)")
            key = (file, function, mutation)
            if key in rows:
                problems.append(f"engine/MUTANTS-SURVIVORS.md:{n}: {file}: {mutation} is listed twice: one row covers "
                                "every mutant with its file, function and mutation (R-87)")
            rows[key] = form["form"] if form else None
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
    equivalent = 0
    for key, summaries in sorted(seen.items()):
        survived = [s for s in summaries if s in SURVIVED]
        if survived and key not in rows:
            problems.append(f"{key[0]}: {key[2]} ({SURVIVED[survived[0]]}) is not in MUTANTS-SURVIVORS.md (R-87)")
        elif rows.get(key) == "equivalent":
            equivalent += len(survived)
    for key in sorted(rows):
        summaries = seen.get(key)
        if summaries is None:
            problems.append(f"{key[0]}: {key[2]} is listed in MUTANTS-SURVIVORS.md and no run produced it: remove the row (R-87)")
        elif not any(s in SURVIVED for s in summaries):
            state = "caught" if "CaughtMutant" in summaries else "unviable"
            problems.append(f"{key[0]}: {key[2]} is listed in MUTANTS-SURVIVORS.md and is now {state}: remove the row (R-87)")
    scored = counts["caught"] + counts["missed"] + counts["timeout"] - equivalent
    score = f"{100 * counts['caught'] / scored:.1f}%" if scored else "n/a"
    print(f"mutants-survivors: score {score} = caught {counts['caught']} / (caught {counts['caught']} + missed "
          f"{counts['missed']} + timed out {counts['timeout']} - equivalent {equivalent}); "
          f"unviable {counts['unviable']}, not counted")
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
