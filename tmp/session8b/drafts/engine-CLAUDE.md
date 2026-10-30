# The Tally engine: rules for the build session

Printed by the build role's SessionStart hook at the start and after each compaction. Invoke the skill `engine-rust` now; it holds the how. The walls (lints, the guard, the Stop hook, CI) hold whatever you read here.

## 1. What this is

- The spec is `engine/spec/` (tally 2.1, frozen, read-only): `LANGUAGE.md` says what a construct means, `ENGINE.md` how it is computed; `PERFORMANCE.md` and `BUILD-ORDER.md` plan the build.
- T's tests are `engine/tests/` (read-only). They decide "done"; refcore (`engine/tools/refcore/`) is the oracle, byte for byte.
- You write `engine/crates/**` and `engine/BUILD-LOG.md`. Nothing else.

## 2. Hard rules, each with the check that enforces it

1. Never edit a test, a vector, an expected value, the spec, a rule file, the skill or this file. *The guard refuses (R-79, R-80, R-81); check-frozen fails.*
2. Never special-case an input: no branch on a case id, an ask text, a path or an expected value. *The special-casing scan (R-94); review.*
3. Never add a crate or a feature. *cargo-deny's allow-lists (R-52).*
4. Never weaken a check: no `#[allow]`, no config edit, no `cfg(not(clippy))`, no `ast-grep-ignore`, no mutation skip, no raised timeout, no retry. *R-103 to R-109, R-86, R-90; CI fails.*
5. A STOP-REPORT is a good outcome. When a test, the spec or a rule seems to conflict with correct behaviour, stop and file one. *The Stop hook lets a turn end only when `check fast` is green or a STOP-REPORT is newer than the last commit (R-83).*

**The STOP-REPORT protocol:**
- Stop when a T test looks wrong; the spec is unclear or silent; a rule blocks a correct design and the budget is used up; you need a crate or a kernel file; a mutation survivor isn't equivalent and T has no test for it; `check fast` stays red on the same failure after three honest tries; or a budget looks wrong.
- Leave the code as it was. No half-fix, no workaround.
- Copy the template at the top of `engine/BUILD-LOG.md` to its end. Give the smallest input, what the engine gives and what the test expects, which of model, spec, test, rule or code you think is wrong and why, what you tried, and what you did not do.
- Then take other red vectors, or end the session.
- The spec is fixed first, never only the code (DONE rule 3).

## 3. The loop

- Start: read the last `engine/BUILD-LOG.md` entries, the milestone's section of `engine/spec/BUILD-ORDER.md`, the plan's traceability table; run `engine/tools/check fast`; note the red vector ids.
- Work: the next group of red vectors in plan order; implement; `engine/tools/check fast`; commit when green.
- Before each commit: `engine/tools/check full`. Never commit red.
- Only `engine/tools/check`, never raw `cargo` with your own flags. Read the one-line failures; open a log only when needed. A failure that doesn't reproduce is reported, not "fixed".
- The Stop hook runs `check fast` at the end of every turn.
- Count budgets and instruction limits are red on every PR, like a failing test. Wall time and peak footprint are judged on the reference Mac at the milestone tag (PERFORMANCE §8).
- End: append to `engine/BUILD-LOG.md` what turned green, what's next, open STOP-REPORTs, the exception delta, the check's last line.

## 4. Exceptions

- Restructure first. Then, only where the area's budget allows, one `#[expect(clippy::<lint>, reason = "R-<n>: why the safe form doesn't fit")]` on the smallest expression or item, with `engine/tools/exceptions --write` in the same commit. Over budget, or no strict form: a STOP-REPORT.
- The library's and the parser's budgets are 0.
- Forbidden: `#[allow]` in any form, inner `#![expect]`, `#[expect]` on a module or a whole `impl`, a reason with "for now", "temporary" or "TODO". The counter's lock must equal the code (R-110).

## 5. Write this instead

- An index `v[i]` becomes `v.get(i).ok_or(…)?`; an id-keyed store is a `Table<I, T>` with `get`.
- `a + b` on integers becomes `a.checked_add(b).ok_or(…)?`, or a newtype method (`day.next()?`).
- `as` becomes `TryFrom` with a refusal.
- `unwrap` becomes `?` or `ok_or(…)?`; `unwrap_or_default` becomes a hole.
- `/` and `%` on integers become `checked_div_euclid` and `checked_rem_euclid`.
- `unreachable!` becomes a narrower type at the point of decision.
- `let _ =` and `.ok()` become handling, or `?`.
- `HashMap` becomes `BTreeMap`, or a dense `Vec` for day-keyed data.
- `dyn` becomes an `enum` and an exhaustive `match`.
- An `impl Trait` argument becomes the concrete type.
- A macro becomes a helper function or a newtype method.
- Nesting past 5 becomes an early return or a step named by the spec's word.
- Clocks and environment variables are never read; the library takes the reality as bytes.

## 6. Numbers

- Exact rationals only, through `Num` (Small `i64/i64` or Big) and `Money` (`I64 | Big` minor units).
- **No floats anywhere** (R-18, no exception).
- Overflow and the 4 096-digit cap are refusals with their codes, except a path statistic's own accumulator, which is its own uncapped type (R-117).

## 7. Holes and refusals

- A missing value is a hole with its question, never 0 (R-27). Holes are built only in the hole module (R-116).
- An error carries its catalogue code (R-76). No `anyhow`, no `Box<dyn Error>`, no `String` errors (R-58).
- An invariant break returns `ENGINE-INVARIANT`, never a panic. Parts are checked against the balance after every arrival, take and relabel (R-115).

## 8. Determinism

- Sorted maps; output order by explicit keys, never by an id, an address, insertion order or a derived `Ord` (R-47, R-118).
- No clock, no environment, no I/O in the library (R-38, R-39).
- Threads only in the sanctioned thread module: each spawned through `Builder::spawn_scoped` with a stack size passed in; results merged in task-index order; no lock, channel or atomic outside it (R-42).
- Full relative paths sorted by code point (R-40).

## 9. Tests

- T's vectors define done; your own tests don't count toward a gate.
- Fixed seeds; no snapshots; no `cfg(test)` behaviour in engine code (R-93).
- Reproduce a failure before you fix it.

## 10. Code shape

- Simple first. Enums with exhaustive `match`; one call or expression per dispatch arm.
- No `dyn`, no `impl Trait` arguments, no library macros (errors); no own generics or one-implementation traits (reported) (R-122 to R-125, R-73).
- Nesting at most 5 (R-74). Recursion only to a counted depth; joint nesting as a worklist (R-16).
- Every `.rs` file starts with its `//!` header: purpose, `Spec:`, `Twin:`, `Invariants:` (R-120).
- Each kernel function's doc comment cites its spec section; a missing or wrong citation there is an error (R-75, R-128).
- Optimised code only in fence modules, each with a plain twin and an equivalence test, opened when a budget row asks (R-130).

## 11. Hand-off

- Append to `engine/BUILD-LOG.md` at each step's end.
- When compacting, keep: the milestone, the red vector ids, the open STOP-REPORTs, the files modified.

## 12. Commits

- One green step per commit, naming the vector ids.
- Never an AI co-author or attribution line (R-100).
