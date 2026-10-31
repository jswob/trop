<!-- saved by tools/save_final.py from agent agent-a8cd0e86a542dfff2.jsonl (scripts) -->

The three scripts and their cases are written. The patch is staged in WT and not committed. The two required runs in WT end red for one reason: the new and changed rule files aren't in RULES.sha256 yet. That clears when you run `check-frozen --write-rules` in the R-112 commit.

**Scripts** (stdlib, `python3 -I -B`, each with a shim):
- `tools/headers` (R-120, error): exit 0 on the stub in 0.06 s. I added comment-only headers to the 4 stub files (`Spec: none, a stub.` and so on).
- `tools/trace [--map]` (R-128, R-75, error): exit 0 on the stub in 0.07 s, with 6 report lines for the stub's uncited functions. It reads citations in comments only, not in strings. It accepts `L`, `E` and `P §n` (P is the spec's short form for PERFORMANCE, as in DONE.md's `P §5.2`), the long forms, and `l.N`. It also checks a fence's `PERF:` line (R-130).
- `tools/special-casing` (M-9, R-94): re-pointed at checks.tsv, the case file names, the `ask` lines of the .tally inputs and every file under baselines/ (read whole, no sampling). Paths holding hidden or holdout are skipped by name. It found 901 ids, 1967 ask texts, 1660 ask names and 10749 values, with 0 hits, exit 0, in about 1.3 s. It now exits 1 when the suite exists but any one of ids, ask texts or values comes out empty.

**Cases (15):** R-120 (3), R-128 (3), R-75 (2), R-94 (5) and R-130 (2). All 15 pass when judged with rules_selftest's own `plant`/`run_tool`/`judge`, in a copy made without the sealed paths: `scratch/simcases.py`, next to WT.

**Runs in WT:**
- `check fast`: exit 1, `red, 1 of 7 steps failed`. Only check-frozen fails, on RULES.sha256.
- `rules-selftest/run R-75 R-94 R-120 R-128 R-130`: exit 1, `the clean run fails, so no case can be judged`. Same cause; every other clean tool passed.

**check.py lines:**
- fast: `("headers", tool("headers"), {})` and `("trace", tool("trace"), {})`
- full: `("special-casing", tool("special-casing"), {})`

**Limits:**
- The tests column of `trace --map` is nearly empty. T's map tags checks with case-set edges, not spec sections, so only 1 check is tied: `U:time/TIME-08-before` to L §5.14.2.
- Values are matched by their digits alone (leading and trailing zeros dropped), so `1 437.00` matches `143_700`, but `1 400.00` counts as 2 digits and isn't searched.
- The scripts can't judge whether a valid citation is the right one, and don't check `Twin:` paths or anything outside `engine/crates/`.

**For you to decide:**
1. The trace refuses `E §9.1`, because ENGINE §9's items are a list, not headings. But spec/DONE.md, spec/BUILD-ORDER.md and the engine-rust skill's header example all cite `E §9.1`. Either keep it strict (and fix the skill's example to `E §9 item 1`) or accept list items as sections.
2. Keep the fence cases in R-130.case, or fold them into R-128.
3. Keep special-casing failing on any one empty source, or only when all three are empty.
4. The RULES.sha256 rewrite must include `crates/tally/benches/stub.rs`, which counts as a rule file.
5. rules_selftest.py stops at R-113 and still lists R-75 and R-94 as proven elsewhere. Consider adding headers and trace to its clean run.
6. advisory.py's `uncited()` now duplicates the trace's R-75 check, and SCAFFOLD §2's tools table needs updating.
7. Whether R-120 also covers `fuzz/`.

notes.md has the §7 rows for R-75, R-94, R-120 and R-128, plus a draft row for R-130.

Files are in /Users/jswob/projects/trop/tmp/session8c/agents/scripts/:
- patch.diff
- notes.md
- progress.md
