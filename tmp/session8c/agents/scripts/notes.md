# Session 8c step 3, the `scripts` helper: notes

Patch: `tmp/session8c/agents/scripts/patch.diff` (14 files, against `05103df`). WT stays staged, never committed.

## The runs (in WT/engine, on the stub)

| Run | Exit | Summary line |
|---|---|---|
| `tools/headers` | 0 | `headers: 4 files, each with its //! header (R-120)` (0.06 s) |
| `tools/trace` | 0 | `trace: every citation is in the spec; 4 files (0 in the kernel), 1 section citations, 6 functions, 6 report lines` (0.07 s; the 6 report lines are the stub's uncited functions, R-75 outside the kernel) |
| `tools/trace --map` | 0 | `trace: T's map ties 1 check to 1 section; it tags checks with the case set's edges, not with spec sections`; 123 of 124 sections have no code yet |
| `tools/special-casing` | 0 | `special-casing: 901 ids, 1967 ask texts, 1660 ask names, 10749 values of 4+ significant digits` then `special-casing: 0 hits, for the review (R-94)` (1.2–1.5 s) |
| `tools/check fast` | 1 | `check fast: red, 1 of 7 steps failed (24s)`: only check-frozen, and only on RULES.sha256 (below). fmt, clippy-lib, clippy-tests, runner, exceptions, check-rules green |
| `tools/rules-selftest/run R-75 R-94 R-120 R-128 R-130` | 1 | `rules-selftest: red: the clean run fails, so no case can be judged`; its one clean problem: `clean check-frozen: … benches/stub.rs changed (checksum differs)` (32 s). Every other clean tool (clippy, clippy-tests, rustdoc, fmt, ast-grep, check-rules, exceptions, the probes, deny) passed |
| the 15 new cases, judged by `rules_selftest`'s own `plant()`, `run_tool()` and `judge()` in a sealed-free copy (`scratch/simcases.py`) | 0 | all 15 `ok`: each fails with its rule's message, the two report cases print theirs with exit 0 |

**Why red:** every new or changed rule file needs `tools/check-frozen --write-rules`, which the R-112 commit does and I may not: `tools/headers`, `headers.py`, `trace`, `trace.py`, `special_casing.py`, the five cases, and `crates/tally/benches/stub.rs`, which the guard counts as a rule file (the benches are the lead's, R-91); its header is a comment-only change. After that, the selftest's clean run should go green and judge the cases for real.

The copy for the case check was made with `rsync` excluding every name holding `hidden` or `holdout` and `blind7.tsv`, so nothing sealed was read; it had no cargo build, which the 15 `run` cases don't need.

## The scripts

### `tools/headers` (R-120), error

- **Checks:** every `.rs` under `engine/crates/` starts with `//!` lines: the purpose (all text before `Spec:`) is one sentence of 6 to 30 words ending in a stop, then lines starting `Spec:`, `Twin:`, `Invariants:`, first three keys in that order, none repeated, none empty. A field may wrap; a `PERF:` line may follow; free text after a blank `//!`. One `FAIL <file>:<line>: … (R-120)` per problem.
- **Usage:** `engine/tools/headers`. Exits 0 / 1 / 64 (usage).
- **Stub:** the four files got headers, comments only: `Spec: none, a stub.`, `Twin: none, a stub.`, an `Invariants:` line from each file's own text; `main.rs` and `benches/stub.rs` had two-sentence purposes, rewritten as one, the rest kept below a blank `//!`.
- **Can't see:** whether the purpose is true (review, BUILDER §12); "one sentence" is a heuristic (a stop, a space, a capital); whether the `Twin:` file exists or does the same work; anything outside `engine/crates/` (`fuzz/fuzz_targets/parse.rs`, T's `tests/runner/`).

### `tools/trace` (R-128, R-75, and R-130's fence line), error

- **Citations:** read in comments only (`//!`, `///`, `//`, block comments), not in string literals (they are data: Tally texts in tests cite sections too). Forms: `L §n`, `E §n`, and **`P §n` for PERFORMANCE**, the short form the spec uses (`DONE.md`: `P §5.2`); the long forms `LANGUAGE`, `ENGINE`, `PERFORMANCE` (`.md` optional); `l.N` after a section or alone (`L l.325`). A section with no document before it (`, §5`, `and §5`, `–§5`) belongs to the last document named in the comment, so `E §4.2 steps 1–3, §5` cites E §5, and `BUILDER §13.1, §13.2` or `engine-tests §4` cite nothing of the spec. A section must be a numbered heading of its document (outside code fences); a line must be ≤ the document's length: either is `FAIL … (R-128)`. A cited line outside its cited section is a report line.
- **Functions:** a `fn` (found by a line regex on rustfmt'd code) whose `///` doc comment, above any attributes, holds no section citation: `FAIL … (R-75)` when the file matches `tools/kernel.toml`'s `kernel` globs (used whatever `set_at_s` says; empty until S), else `trace: report: …`. Skipped: `#[test]`, `#[cfg(test)]` functions, functions in a `#[cfg(test)] mod … { }`, and files under a crate's `tests/`, `benches/`, `examples/`.
- **Fences:** a header `PERF:` line must name a `P8-<n>` that is a row of PERFORMANCE §8 (P8-1..P8-15 today) and a plain twin (`plain twin: <file>`) that exists beside the file, under `engine/` or the repo root, or `refcore` when the `Twin:` line names an existing file under `engine/tools/refcore/`. `FAIL … (R-130)`.
- **The map (`--map`):** per cited section, in spec order: the files and how they cite it (`Spec:`, `fn <name>`, `a comment, l.N`), each file's `Twin:`, then T's checks; then the sections with no code (a parent and its subsections count separately). **The tests column, honestly:** T's map (`engine/tests/map/tags.tsv`) tags checks with the case set's *edges* (`ASK-01`, `NUM-03`, from `cases/boundaries.md`), not spec sections. The script ties a check to a section only where a citation stands in its map row: today exactly one, `U:time/TIME-08-before` → L §5.14.2 (in a `sources` note). So the column shows almost nothing now and can't show coverage until T's map gains a section per edge or per check (T's interface; the probe texts cite sections in only 11 of ~110 files, so they are no substitute).
- **Usage:** `engine/tools/trace [--map]`. Exits 0 (reports may print) / 1 / 64.
- **Can't see:** whether a valid citation is the right one (coarse citations; BUILDER §12 item 1); `#[doc = "…"]` and `/** */` as function docs; functions from macros; `mod tests;` declared in another file; bare `§n` with no document; strings.

### `tools/special-casing` (R-94, M-9), a report that can fail

- **Gathers** (from `engine/tests/`, as its README describes the suite; every path holding `hidden` or `holdout` is skipped by name, its folders never entered): check ids from `checks.tsv`'s `id` column and the case ids inside them (`I:M:CB-01` → `CB-01`); case ids from file names under `cases/` (`A-01`, `ASK-02`, `S2-02`); the asks: each `ask <name>: <text>` line of the `.tally` inputs `checks.tsv` names (one sealed name in the `files` column is skipped unread): the text (whitespace folded, ≥ 12 chars) and the name (≥ 6 chars with `_` or a digit; plus `checks.tsv`'s `asks` column); values: every number in every file under `baselines/` (14 MB, read whole, no sampling), dates dropped, kept when it has 4+ significant digits.
- **Value matching is by digits alone:** sign, separators, point and leading and trailing zeros dropped, so `1 437.00 DUK` matches the literal `143_700` (minor units). Consequence: `1 400.00` counts as 2 digits and isn't searched; false positives across scales are possible (it's a report).
- **Searches** `engine/crates/**/*.rs`: ids in strings and code (comments aside), ask texts (case-insensitive) and names in strings, values as number literals and as numbers inside strings. Hit lines end `(R-94)`.
- **Exit:** 0 with hits or none; the old notice (exit 0) only when `engine/tests/` holds no suite (no `checks.tsv`, no `cases/`); **1** when the suite exists but the ids, the ask texts or the values come out empty (a stricter reading of "finds nothing": any one empty source means the scan is partly off, as M-9 found); 64 usage.
- **Can't see:** an answer computed from parts, split, encoded in hex or summed; English question texts (`cases/*/questions.md`); `load/` and `examples.tsv`; anything T stores outside the three sources.

## The `check.py` step lines

```python
# fast (the Stop hook's check), each well under a second:
("headers", tool("headers"), {}),
("trace", tool("trace"), {}),
# full (and CI, where R-94 asks for it):
("special-casing", tool("special-casing"), {}),
```

## SCAFFOLD §7 rows

- **R-75** Each kernel function's doc comment cites the spec section it implements (`/// L §7.2`): an error in the kernel (`tools/kernel.toml`), a report line elsewhere. **Enforced by:** `engine/tools/trace` (R-128's script), error in the kernel, report elsewhere. **Proven by:** selftest: `uncited-kernel-function`, `uncited-elsewhere-is-a-report`.
- **R-94** A special-casing scan: T's case and check ids, ask texts and every expected value with 4 or more significant digits, searched for in the engine's code; each hit is reviewed. **Enforced by:** `engine/tools/special-casing`, a report (exit 0 with hits) that fails when `engine/tests/` holds a suite but gives it nothing to search for. **Proven by:** selftest: `case-id-in-a-string`, `ask-text-in-a-string`, `ask-name-in-a-string`, `value-as-a-literal`, `suite-gives-nothing-to-search`.
- **R-120** Every `.rs` file under `engine/crates/` starts with a `//!` header: a purpose sentence (6 to 30 words), then `Spec:`, `Twin:` and `Invariants:` lines, in that order. **Enforced by:** `engine/tools/headers`, error. **Proven by:** selftest: `no-header`, `header-without-twin`, `purpose-not-one-sentence`.
- **R-128** A spec citation in the code (`L §n`, `E §n`, `P §n`, `l.N`) must exist: a section that isn't a numbered heading of its document, or a line past its end, is an error everywhere; `--map` prints section → code → twin → T's checks and the sections with no code yet. **Enforced by:** `engine/tools/trace`, error. **Proven by:** selftest: `unknown-section`, `line-past-the-end`, `list-item-is-not-a-section`.
- (if kept) **R-130** … **Enforced by:** review; `engine/tools/trace` checks a fence's `PERF:` line names an existing P8 row and an existing plain twin, error. **Proven by:** selftest: `fence-names-no-budget-row`, `fence-names-no-twin`.

## For the lead to decide or carry

1. **`E §9.1` is an error, but the spec itself writes it.** ENGINE §9's invariants are a numbered list, not headings, so the trace refuses `E §9.1` (case `list-item-is-not-a-section`), as the brief's "a heading of that document" says. Yet `engine/spec/DONE.md` and `BUILD-ORDER.md` cite `E §9.1`, and so does the engine-rust skill's header example (`references/readability.md`: `Invariants: money conserved per batch (E §9.1)`), so a builder will copy it. Either keep strict (fix the skill's example to `E §9 item 1`; the frozen spec keeps its wording), or let a numbered item `n.` under heading §k count as §k.n when no such heading exists (about 8 lines in `load_doc`, and the case flips to a pass).
2. **R-130.case:** the fence check is part of the trace (item 2's third point), but its cases name R-130, so I put them in `R-130.case`; fold into R-128 if you prefer.
3. **special-casing fails on any empty source** (ids, asks or values), slightly stricter than "finds nothing". Relax to all-empty if you want the brief's letter.
4. **rules_selftest.py** (not mine to edit): `completeness()` and `list_rules()` stop at R-113, so `--list` doesn't show R-120, R-128, R-130; `ELSEWHERE` still maps R-75 and R-94 to the advisory and the report (now they have cases); consider adding `tools/headers` and `tools/trace` to `clean_run()`'s tool list.
5. **advisory.py**'s `uncited()` duplicates the trace's R-75 check; SCAFFOLD §2's tools table needs `headers` and `trace` rows, and its "the two reports, which always exit 0" no longer holds for special-casing.
6. **RULES.sha256:** rewrite in the commit (list above), including `benches/stub.rs`.
7. **R-120's reach:** `fuzz/` and `tests/runner/` (T's) aren't checked; say whether R-120 covers `fuzz/`.
