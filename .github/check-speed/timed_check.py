"""`engine/tools/check`, step by step with each step's wall time (measurement only).

WT=WORKTREE python3 -I -B timed_check.py fast|full [--fresh] [--steps a,b,...] --out FILE.jsonl

It imports WORKTREE/engine/tools/check.py and runs its steps exactly as its main() does
(same clean environment, same argv), but writes one JSON line per step: name, seconds,
result. The brief's limits are put back after the clean environment: CARGO_BUILD_JOBS=6
and NEXTEST_TEST_THREADS=6 (check.py clears both; neither changes an output).
CARGO_INCREMENTAL passes through as check.py allows (0 or 1).
"""

import json
import os
import sys
import time


def main():
    args = sys.argv[1:]
    worktree, mode = os.environ["WT"], args[0]
    out = args[args.index("--out") + 1]
    only = args[args.index("--steps") + 1].split(",") if "--steps" in args else None
    sys.path.insert(0, os.path.join(worktree, "engine", "tools"))
    import check  # noqa: E402

    check.FRESH = "--fresh" in args
    env = check.clean_env()
    env["CARGO_BUILD_JOBS"] = os.environ.get("CARGO_BUILD_JOBS", "6")
    env["NEXTEST_TEST_THREADS"] = os.environ.get("NEXTEST_TEST_THREADS", "6")
    os.makedirs(check.LOGS, exist_ok=True)
    chosen = [s for s in check.steps(mode) if only is None or s[0] in only]
    began = time.monotonic()
    with open(out, "a", encoding="utf-8") as rec:
        rec.write(json.dumps({"start": time.strftime("%Y-%m-%d %H:%M:%S"), "mode": mode, "fresh": check.FRESH,
                              "incremental": env["CARGO_INCREMENTAL"], "steps": [s[0] for s in chosen]}) + "\n")
        for name, what, extra in chosen:
            t0 = time.monotonic()
            with open(os.path.join(check.LOGS, f"{name}.log"), "w", encoding="utf-8") as log:
                step_env = {**env, **extra}
                if callable(what):
                    problem = what(step_env, log)
                    report = isinstance(problem, check.Report)
                else:
                    code, outp = check.run(what, step_env, log)
                    problem = None if code == 0 else check.first_error(outp)
                    report = False
            took = time.monotonic() - t0
            row = {"step": name, "s": round(took, 1), "ok": problem is None or report,
                   "line": str(problem)[:300] if problem else ""}
            rec.write(json.dumps(row) + "\n")
            rec.flush()
            print(f"{name}\t{took:.1f}s\t{'ok' if row['ok'] else 'FAIL'}\t{row['line'][:120]}", flush=True)
        total = time.monotonic() - began
        rec.write(json.dumps({"total_s": round(total, 1)}) + "\n")
        print(f"total\t{total:.1f}s", flush=True)


if __name__ == "__main__":
    main()
