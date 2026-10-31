# Session 8c, step 3: the new ast-grep rules (agent `astgrep`)

You are a helper of session 8c (the Tally engine prep, part B2). The user approved the new rules at gates G1 and G2a (2026-10-30) and the audit's fix list on 2026-10-31. You write the ast-grep side as a **patch** in a throwaway worktree. The lead reviews it, splits it into R-112 commits and runs the full check. You never commit.

## 1. Hard rules

1. **Seals** (word for word from the session brief):
   - Never open, list or search `tmp/holdout/`.
   - A recursive `find`, `grep` or glob over `tmp/` must prune it: `find tmp -path tmp/holdout -prune -o …`.
   - Every agent prompt says the same, word for word.
   - Don't open `engine/tests/cases/*-hidden.md`; no step here needs them.
2. **No sub-agents.** Don't spawn agents or forks, run workflows or invoke skills; never use the Agent, Task, Workflow, Skill or SendMessage tools. No web research.
3. **Where you work.** `WT` = `/private/tmp/claude-501/-Users-jswob-projects-trop-engine-scaffold/10fe1c33-6637-4d53-aca9-85a7c2f9d2a8/scratchpad/astgrep/wt`, a git worktree detached at `05103df` (branch `engine-scaffold`'s head). Inside WT you may write only `engine/tools/ast-grep/rules/*.yml`, `engine/tools/ast-grep/tests/*.yml` and `engine/tools/rules-selftest/cases/R-*.case`. Plants for your own trials go in WT and are reverted. Scratch goes in `WT/../scratch/`.
4. **What you may write outside WT:** only `/Users/jswob/projects/trop/tmp/session8c/agents/astgrep/` (`patch.diff`, `notes.md`, `progress.md`). Never a file named `REPORT.md`. Nothing in `/Users/jswob/projects/trop-engine-scaffold` or elsewhere in `/Users/jswob/projects/trop`, and no git command that writes there. Inside WT: no `git commit`, no `git stash`.
5. **Your session runs under the live guard in the `rules` role;** don't try to get around it.
6. **Machine:** at most one cargo build, `check` or selftest run of yours at a time. No installs. No refcore or `run.py` runs.
7. **Scope is closed:** the rules in §3. Aim for 60–100 tool calls. Keep `progress.md` short and current.
8. **The engine's code stays the stub.** If a rule fires as an error on the stub's code (`engine/crates/*/src/**`, `engine/tests/runner/`), don't edit the code: narrow the rule if the hit is wrong, or report it in `notes.md`.

## 2. Read first (narrowly)

- `/Users/jswob/projects/trop/tmp/research_notes/engine-rust-strictness/RULEBOOK.md` rows R-2, R-5, R-66, R-73, R-118, R-122, R-124, R-125, R-130 (by grep; group 8 is "Readability").
- `/Users/jswob/projects/trop/tmp/session8c/audit.md` §3, rows F-12 and F-15, and `agents/auditor/findings.md` for the same ids.
- `WT/engine/sgconfig.yml`; three existing rules and their tests for the house style: `macro-definition-review.yml` (a report), `no-debug-format.yml` (test modules, `tests/` and `benches/` exempt), `no-float.yml`.
- `WT/engine/tools/rules_selftest.py`'s header (the case format) and two cases: `cases/R-73.case`, `cases/R-45.case`.
- `WT/engine/SCAFFOLD.md` §7 (the rule map's row format).

## 3. The rules (the closed list)

Severity `error` fails `ast-grep scan` and CI; `warning` is a report (never fails; the exception counter lists the hits). Messages start with the rule id, like the existing ones. Every rule gets valid and invalid snippets in `tools/ast-grep/tests/` and a planted violation in `tools/rules-selftest/cases/R-<n>.case`, failing with the rule's id.

1. **R-122, error:** no `dyn` anywhere in the engine's code (`crates/**`, `tests/runner/**`): trait objects in any position (`&dyn T`, `Box<dyn T>`, `dyn Fn`).
2. **R-73, error:** no `macro_rules!` in `crates/*/src/**` outside the lead's list of files. The list is the rule's own `ignores:` (empty today, with a comment saying it is the lead's list, changed through R-112). `macro-definition-review` stays as the report for listed files.
3. **R-118, warning:** every `#[derive(…)]` that names `PartialOrd` or `Ord` on the engine's own types.
4. **R-124, warnings (one rule each):** own type parameters (a `<T>` on a fn, struct, enum, impl or trait); every `trait` definition (review judges "two implementations today"); explicit lifetimes (O-W3-1 item 1: a lifetime parameter or annotation other than `'static`).
5. **R-125, warning** (O-W3-1 item 2): an iterator or method chain of 5 or more calls; a closure inside a closure.
6. **R-130, warning** (O-W3-1 item 3): `#[inline(always)]` and `#[cold]` outside a fence. The fences are the rule's `ignores:` (empty today; the lead adds each fence's file through R-112).
7. **F-12 (R-5), error:** every index or slice expression (`a[i]`, `a[1..]`, `m[&k]` on any type) outside tests (test modules, `tests/`, `benches/`): R-5 bans them all, maps included, which clippy's `indexing_slicing` misses.
8. **F-12 (R-2), error:** `Option::unwrap`, `Option::expect`, `Result::unwrap`, `Result::expect`, `Result::unwrap_err`, `Result::expect_err` written as paths (`.map(Option::unwrap)`, `Option::unwrap(x)`), with or without `std::option::`/`core::option::` prefixes, outside tests.
9. **F-15 (R-66), error:** `_ = <call>;` and `let _<name> = <call>;` (a discarded result) in library code (`crates/*/src/**`, test modules exempt). `let _ = …` is already caught elsewhere; check which rule, and say so.

## 4. Before you finish

- In WT/engine: `ast-grep test --skip-snapshot-tests`, `ast-grep scan` (no error on the stub), `tools/exceptions --check` (the counter's total unchanged at 5, or say why), then `tools/rules-selftest/run R-2 R-5 R-66 R-73 R-118 R-122 R-124 R-125 R-130`. Record each exit and summary line in `notes.md`.
- `git -C WT add -A && git -C WT diff --cached > /Users/jswob/projects/trop/tmp/session8c/agents/astgrep/patch.diff`.
- `notes.md`: per rule, the file, the pattern in a sentence, the cases; what each rule can't see (a known limit), honestly; one SCAFFOLD §7 row per rule (new or updated), as text in §7's format.

## 5. Your final message

About 10 lines: the rules written (id, severity), the cases added, the exits of the four runs, the known limits, and anything the lead must decide.
