# Session 8c, step 3: the readability scripts and special-casing (agent `scripts`)

You are a helper of session 8c (the Tally engine prep, part B2). The user approved the readability rules at gate G1 (2026-10-30) and, on 2026-10-31, re-pointing special-casing at the real test suite. You write three scripts as a **patch** in a throwaway worktree. The lead reviews it, splits it into R-112 commits, wires the scripts into `tools/check` and runs the full check. You never commit.

## 1. Hard rules

1. **Seals** (word for word from the session brief):
   - Never open, list or search `tmp/holdout/`.
   - A recursive `find`, `grep` or glob over `tmp/` must prune it: `find tmp -path tmp/holdout -prune -o …`.
   - Every agent prompt says the same, word for word.
   - Don't open `engine/tests/cases/*-hidden.md`; no step here needs them.
   - For this task also: don't open `engine/tests/defs/holdout.tsv`, `engine/tests/defs/blind7.tsv`, or any file under `engine/tests/` whose path contains `hidden` or `holdout`; the scripts you write skip such paths without reading them.
2. **No sub-agents.** Don't spawn agents or forks, run workflows or invoke skills; never use the Agent, Task, Workflow, Skill or SendMessage tools. No web research.
3. **Where you work.** `WT` = `/private/tmp/claude-501/-Users-jswob-projects-trop-engine-scaffold/10fe1c33-6637-4d53-aca9-85a7c2f9d2a8/scratchpad/scripts/wt`, a git worktree detached at `05103df` (branch `engine-scaffold`'s head). Inside WT you may write only: new scripts in `engine/tools/` (a `name.py` plus a `name` shim like the existing ones), `engine/tools/special_casing.py`, `engine/tools/rules-selftest/cases/R-*.case`, and the `//!` header lines of the stub's `.rs` files (comments only, never code). Not `check.py` (the lead wires the steps). Scratch goes in `WT/../scratch/`.
4. **What you may write outside WT:** only `/Users/jswob/projects/trop/tmp/session8c/agents/scripts/` (`patch.diff`, `notes.md`, `progress.md`). Never a file named `REPORT.md`. Nothing in `/Users/jswob/projects/trop-engine-scaffold` or elsewhere in `/Users/jswob/projects/trop`, and no git command that writes there. Inside WT: no `git commit`, no `git stash`.
5. **Your session runs under the live guard in the `rules` role;** don't try to get around it. Reading `engine/spec/` and `engine/tests/` is fine; writing them is refused.
6. **Machine:** at most one cargo build, `check` or selftest run of yours at a time. No installs. No refcore or `run.py` runs.
7. **Scope is closed:** the three items in §3. Aim for 70–110 tool calls. Keep `progress.md` short and current.
8. **Style:** stdlib Python 3, run as `python3 -I -B`, like `special_casing.py` and `check_rules.py` (reuse `check_rules`' lexer and file walk). A module docstring says what it checks, its usage and its exit codes.

## 2. Read first (narrowly)

- `/Users/jswob/projects/trop/tmp/research_notes/engine-rust-strictness/RULEBOOK.md` rows R-75, R-85, R-94, R-120, R-128, R-130 (by grep), and `BUILDER.md` §13.1 (fences).
- `/Users/jswob/projects/trop/tmp/session8/decisions.md` D5-1 and D5-4 (by grep).
- `WT/engine/SCAFFOLD.md` §4 (the kernel at S) and §7's row format; `WT/engine/tools/kernel.toml`.
- `WT/engine/tools/special_casing.py`, `check_rules.py` (its lexer and `walk`), `rules_selftest.py`'s header (the case format), `cases/R-75.case` if it exists, and `/Users/jswob/projects/trop/tmp/session8c/merge.md` row M-9.
- The spec's section headings, by grep only: `WT/engine/spec/LANGUAGE.md`, `ENGINE.md`, `PERFORMANCE.md`.
- `WT/engine/tests/README.md`, `checks.tsv`'s header and first lines, `map/README.md`, `map/tags.tsv`'s first lines, and a listing of `baselines/` (names only, skipping the sealed paths).

## 3. Your items (the closed list)

Each gets a planted violation in `tools/rules-selftest/cases/R-<n>.case` that fails with the rule's name, and the stub must pass.

1. **R-120, `tools/headers`, error.** Every `.rs` file under `engine/crates/` starts with a `//!` header: a purpose sentence (judge "about 15 words" as 6 to 30), then lines starting `Spec:`, `Twin:` and `Invariants:`, in that order. Give the stub's files such headers (comments only; `Spec: none, a stub`, and so on), so the stub passes.
2. **R-128 and R-75, `tools/trace`, error.**
   - It reads every spec citation in the code under `engine/crates/`: `L §n[.m…]`, `E §n[.m…]`, `PERFORMANCE §n` (or the short form you find the codebase's documents use; say which), each optionally followed by `l.N`. A section that isn't a heading of that document, or a line past its end, is an error everywhere.
   - A function in the kernel (the files `tools/kernel.toml` lists; empty until S) whose doc comment cites no section is an error; elsewhere it is a report line.
   - A file whose header has a `PERF:` line (a fence, R-130) must name a P8 row that exists in PERFORMANCE §8 and a twin; otherwise an error.
   - `tools/trace --map` prints the reverse map: section → files → their `Twin:` → T's tests, where T's map (`engine/tests/map/`) ties a test to a section; and the sections with no code yet. Say plainly what the tests column can and can't show from T's current map.
3. **M-9 (R-94), `tools/special_casing.py`.** Re-point it at the suite T inherits: the case and check ids (from `checks.tsv`, and the case file names), the ask texts, and the expected values of 4+ significant digits (from the stored outputs under `baselines/`, as T's README describes them), skipping the sealed paths by name. Its notice ("nothing to search for") becomes a failure once `engine/tests/` holds a suite and the script finds nothing in it. It stays a report otherwise (exit 0 with hits). Keep it fast (seconds); if a full read is slow, say what you sampled.

## 4. Before you finish

- In WT/engine: run each script on the stub (exit 0 for `headers` and `trace`; `special-casing`'s summary line), then `tools/check fast`, then `tools/rules-selftest/run R-75 R-94 R-120 R-128`. Record each exit and summary line in `notes.md`.
- `git -C WT add -A && git -C WT diff --cached > /Users/jswob/projects/trop/tmp/session8c/agents/scripts/patch.diff`.
- `notes.md`: per script, what it checks, its usage, its run time on the stub, what it can't see (honestly); the `check.py` step line each needs (name, argv, and `fast` or `full`); one SCAFFOLD §7 row per rule (R-75, R-94, R-120, R-128), as text in §7's format.

## 5. Your final message

About 10 lines: the scripts, the cases added, the exits of the runs, the known limits, the check.py lines, and anything the lead must decide.
