---
name: engine-rust
description: How to write, fix and review the Tally engine's Rust under its approved rulebook (R-1 to R-131) and builder guidance, so the code stays exact, deterministic, crash-free, fast by structure and readable by an owner who doesn't read Rust. Invoke it by name at the start of every engine build or review session (TALLY_ROLE=build). Also use it whenever you edit or review code under engine/crates/, when clippy, ast-grep, cargo-deny, the guard or the Stop hook blocks you, when you are tempted to add an allow or expect, when a test or the spec looks wrong, when a count or time budget misses, or when you write numbers, the memo, the day walk, threads, parts and labels, holes or joint members.
---

# engine-rust: writing the Tally engine

The engine is a standalone Rust library and CLI: a reality (Tally text files) and an ask in, answers out, the same bytes the reference interpreter `engine/tools/refcore/` prints (DONE gate 10). It makes four promises, and every rule below serves one of them:

- **Exact.** No floating point anywhere. Every number is an exact rational; money is a whole number of minor units.
- **Deterministic.** The same bytes on every machine, every run, with 1 or 6 workers (gate 6).
- **Never a crash.** A missing value is a hole with its question; a bad input is a refusal with its catalogue code; a broken invariant is `ENGINE-INVARIANT`. A crash on a user's model is always a bug.
- **Readable by its owner,** who audits through `engine/ARCHITECTURE.md`, each file's `//!` header and the trace to the spec, not through the Rust.

Speed comes from the structure (skip empty days, share walks, a well-keyed memo, dense columns), not from clever code. Plain code first; optimised code only in fences.

**Where the rules live.** The lints, the guard, the Stop hook and CI are the walls: they hold whatever you read. `engine/CLAUDE.md` (printed at session start and after compaction) holds the never-miss rules. This skill holds the how. The full texts: `engine/RULEBOOK.md` (every rule with its reason) and `engine/BUILDER.md` (§1–§13). The design: `engine/spec/PERFORMANCE.md`, `BUILD-ORDER.md`; the meaning: `engine/spec/LANGUAGE.md` (L) and `ENGINE.md` (E). Where this skill and the rulebook differ, the rulebook wins; where either differs from the spec, the spec wins, and you stop.

## 1. The hard rules

Each names the check that enforces it, so a red line tells you which rule you met.

1. Never edit tests, vectors, expected values, the spec, a rule file, this skill or `engine/CLAUDE.md`. _Check:_ the guard (R-79, R-80, R-81), `check-frozen`.
2. Never special-case an input: no branch on a case id, an ask text, a path or an expected value. _Check:_ the special-casing scan (R-94), review.
3. Never add a crate or a feature. _Check:_ cargo-deny's allow-lists (R-52, R-56).
4. Never weaken a check: no `#[allow]`, no config edit, no `cfg(not(clippy))`, no `ast-grep-ignore`, no mutation skip, no raised timeout, no retry. _Check:_ R-103 to R-109, R-70, R-86, R-82, R-90.
5. No `unsafe`, no floats, no `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!` outside tests, no `assert!` outside tests. _Check:_ `forbid` lints with no exception (R-1, R-18, R-2, R-3, R-4, R-106).
6. Integers only through checked calls; no `as` casts, no `/` or `%`, no `x[i]` indexing or `s[a..b]` slicing. _Check:_ clippy errors (R-19, R-21, R-22, R-5, R-6).
7. A missing value is a hole, never 0 or a default; an error keeps its catalogue code; no error is dropped. _Check:_ R-27, R-66, R-76, R-116.
8. Determinism: no hash map or set, no clock, no environment, no I/O in the library, no derived order reaching output, threads only in the thread module. _Check:_ R-36 to R-47, R-118.
9. The subset: no `dyn`, no `impl Trait` arguments, no library macros, nesting at most 5. _Check:_ R-122, R-123, R-73, R-74.
10. Every `.rs` file starts with its `//!` header; every kernel function cites its spec section; every citation exists. _Check:_ the header script (R-120), the `trace` script (R-75, R-128).
11. Recursion only to a counted depth; the stack is never measured. _Check:_ R-16, review.
12. Optimised code only in fence modules, each with a plain twin and an equivalence test. _Check:_ R-130, review.
13. One green step per commit, naming the vector ids; never an AI co-author or attribution line. _Check:_ R-100's tripwire.

**A STOP-REPORT is a good outcome** (R-83, R-112). Stopping well beats passing by the wrong means.

## 2. When a check blocks you

1. **Restructure first:** a slice pattern, an iterator, a checked operation, a narrower type, a newtype method, an early return.
2. **Then, only where the area's budget allows,** one `#[expect(clippy::<lint>, reason = "R-<n>: why the safe form doesn't fit")]` on the smallest expression or item, and `engine/tools/exceptions --write` in the same commit. The library's and the parser's budgets are 0, so there this step doesn't exist. The reason names the domain, never the tool, and never says "for now", "temporary" or "TODO" (R-104).
3. **Over budget, or no strict form:** a STOP-REPORT (§5).

Never: `#[allow]` in any form, an inner `#![expect]`, an `#[expect]` on a module or a whole `impl`, `cfg(not(clippy))`, an `ast-grep-ignore` comment, an edit to `clippy.toml`, the lint table, `deny*.toml` or the budget file. Each fails CI by design, and the guard refuses the edits.

## 3. Write this instead

Each line: the workaround that tempts, what to write instead, and the rule that says why. The references give the longer form.

- **Instead of** `v[i]`, `&s[a..b]`: `v.get(i).ok_or(…)?`, a cursor, a slice pattern `[first, rest @ ..]`, `str::get`, `split_once` (R-5, R-6).
- **Instead of** `unwrap_or_default()`, `unwrap_or(0)`, `.get(i).unwrap_or(&0)`: a hole with its question, or `ok_or(…)?` to a coded refusal (R-27).
- **Instead of** `opt.unwrap()`, `expect("…")`: `?`, `ok_or(…)?`, `let Some(x) = … else { return Err(…) };` (R-2).
- **Instead of** `n + 1`, `a * b` on integers: `n.checked_add(1).ok_or(…)?`, or a newtype method: `day.next()?` (R-19, R-131).
- **Instead of** `saturating_*`, `wrapping_*`, `overflowing_*`: `checked_*(…).ok_or(…)?`: an overflow is a refusal, never a clamp (R-23).
- **Instead of** `x as u32`: `u32::try_from(x)` mapped to a refusal (R-21).
- **Instead of** `a / b`, `a % b` on day numbers or indices: `checked_div_euclid`, `checked_rem_euclid` (Python's `//` and `%` beside it in a comment) (R-22).
- **Instead of** floats for a division or a ratio: `Num`, the exact rational (R-18).
- **Instead of** `a.add(b)`, `Add::add(a, b)`: a checked call; `Num` and `Money` have named checked methods, no operators (R-20, R-28).
- **Instead of** `.ok()`, `let _ = write(…)`: handle the error, or return it with `?` (R-66).
- **Instead of** `Box<dyn Error>`, `anyhow`, `String` errors: the catalogue's error enum, one variant per code (R-58, R-76).
- **Instead of** `unreachable!()` after a check: a narrower type at the point of decision, so the branch doesn't exist (R-3).
- **Instead of** a macro that hides `unwrap` or `panic!`; any `macro_rules!` outside the lead's list: a named helper in the kernel helper module, or a newtype method (R-13, R-73).
- **Instead of** `#[allow]`, `#![allow]`, `cfg_attr(…, allow(…))`: restructure; at most an `#[expect]` with a reason, in budget (R-103, R-107).
- **Instead of** an `#[expect]` on a whole `impl` or module: one `#[expect]` on the smallest item (R-107).
- **Instead of** `cfg(test)` behaviour, `debug_assertions` checks, reading answer keys or `engine/tests/`: the same code path in test and release (R-93).
- **Instead of** `#[mutants::skip]`: a test at the public level (`run`), or the survivor listed with its reason (R-86, R-87).
- **Instead of** turning a hook or a lint off through config: nothing: a STOP-REPORT (R-79 to R-82).
- **Instead of** `clone()` storms to dodge the borrow checker: restructure ownership: borrow, return the value, or an `Rc` for shared walk state (R-74 (report), review).
- **Instead of** `HashMap`/`HashSet`, an FxHash map "for speed", an `IndexMap` used as a hash map: `BTreeMap`/`BTreeSet`; a dense `Vec` for day- or position-keyed cells; a sorted boxed slice for frozen tables (R-36, R-37).
- **Instead of** `table[id]` on an id-keyed store: `Table<I, T>`: `table.get(id)?`; there is no `Index` impl, so indexing doesn't compile (R-5, BUILDER §3).
- **Instead of** `Box<dyn Trait>`, `&dyn Trait`; a wrapper trait plus a generic to avoid the word `dyn`; an `ast-grep-ignore` comment: an `enum` with one variant per case and an exhaustive `match`, each arm calling a named function; a real need for open-ended types is a STOP-REPORT, for a rules change (R-122, R-124, R-62, R-126, R-109, R-112).
- **Instead of** an `impl Trait` argument, `fn f(xs: impl Iterator<Item = Day>)`, or the same as `<I: Iterator>`: the concrete type: `&[Day]`, a `Table<I, T>`, or a named iterator struct (R-123, R-124).
- **Instead of** nesting past 5; `#[expect(clippy::excessive_nesting)]`; mechanical `step1`/`step2` splits; a level hidden in a closure: an early return; conditions pushed up, loops pushed down into a step named by the spec's word (R-74, R-121).
- **Instead of** a `Mutex<Vec<…>>` workers push into; an `mpsc` channel; an `AtomicUsize` counter: each worker owns a `Vec` of numbered slots; results come back through `join`; one thread merges in task-index order (R-42).
- **Instead of** `s.spawn(…)` inside `thread::scope`; `thread::spawn`; a stack size from an env variable: `Builder::spawn_scoped` only: `Builder::new().stack_size(s).spawn_scoped(scope, f)` with `s` passed in, in the thread module (R-42, R-39).
- **Instead of** `trait CheckedExt for u32 { fn inc(self) … }`: a domain newtype with checked methods, or a free helper in the kernel helper module (R-131).
- **Instead of** `#[inline(always)]`, an unrolled loop or a cache added in the evaluator "while I'm here": plain code; a fence with its `PERF:` header, benchmark, twin and equivalence test, once a budget row asks (R-130).
- **Instead of** `#[derive(PartialOrd, Ord)]` on a label, member or party type, then sorting output by it: an explicit sort key: the text order of stable ids, or the written `ties by` (R-47, R-118).
- **Instead of** `SystemTime::now()`, `std::env::var`, file reads in the library: the library takes the reality as in-memory bytes; the CLI's `main` does all I/O (R-38, R-39).

## 4. The loop

- **Start:** the SessionStart hook prints `engine/CLAUDE.md`; invoke this skill; read the last entries of `engine/BUILD-LOG.md`, the milestone's section of `engine/spec/BUILD-ORDER.md` and the plan's traceability table; run `engine/tools/check fast` and note the red vector ids.
- **Work:** take the next group of red vectors in plan order; read the spec sections they cite; implement; `engine/tools/check fast`; commit when green, one green step per commit.
- **Before each commit:** `engine/tools/check full`. Never commit red.
- **Always** through `engine/tools/check`, never raw `cargo` with your own flags. Read the one-line failures; open a log only when needed. If a failure doesn't reproduce, report it; don't "fix" it.
- **Budgets:** count budgets and instruction limits are red on every PR, like a failing test; wall time and peak footprint are judged on the reference Mac at the milestone tag (PERFORMANCE §8, BUILDER §13). A miss is one line, e.g. `COUNT E1/tx2_mtg_day def_miss: 701 203 > 619 000`.
- **End:** append to `engine/BUILD-LOG.md` what turned green, what's next, open STOP-REPORTs, the exception delta and the check's last line. When compacting, keep the milestone, the red vector ids, the open STOP-REPORTs and the files modified.

## 5. The STOP-REPORT

**Stop when** a T test looks wrong; the spec is unclear or silent; a rule blocks a correct design and the budget is used up; you need a crate or a new kernel file; a mutation survivor isn't equivalent and T has no test for it; `check fast` stays red on the same failure after three honest tries; or a budget looks wrong (a count or limit no correct design can meet).

**Then:** leave the code as it was (no half-fix, no workaround), and copy the template at the top of `engine/BUILD-LOG.md` to its end. Fill it with the smallest input, what the engine gives and what the test expects, which of the model, the spec, the test, the rule or the code you think is wrong and why, what you tried, and what you did **not** do. A STOP-REPORT heading newer than the last commit lets the turn end (R-83). Then move on to other red vectors, or end the session.

Never edit a test, special-case an input, or weaken a check to get past a stop. The spec is fixed first, never only the code (DONE rule 3).

## 6. Performance, in one paragraph

Measure before you optimise, and count before you time: most slowness here is more work, not slower work. The hot paths are known from refcore's counts (`tmp/session8/shape.md`): the day walk (only 2–7 % of days have work), repeated walks (64× in one ask), the memo's misses (a few day-keyed defs carry 85–95 %), the read index, and the number tiers. Each becomes a fence when a budget row asks, by its P8 id. Read `references/performance.md` before any optimisation, and `references/evaluator-and-memo.md` before touching the walk or the memo.

## 7. References: read the one you need

- `references/numbers.md`: read it when you write or change `Num`, `Money`, rounding, the cap, gcd, the transcendental kernel, path statistics.
- `references/data-layout.md`: read it when you add a table, a map, an id, an interned name, checkpoints or walk storage.
- `references/evaluator-and-memo.md`: read it when you write the evaluator, the memo, the day walk, recursion, cycles, errors and codes.
- `references/parallelism.md`: read it when you touch threads, paths, timelines, the merge or gate 6's tests.
- `references/performance.md`: read it when you open a fence, add counters, read a budget miss, profile.
- `references/readability.md`: read it when you start a file, name things, cite the spec, shape a function, review a diff.
- `references/seams.md`: read it when you build parts and labels, holes, joint members, the symbol table or "is it random".
