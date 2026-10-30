# Readability: the map, the headers, the trace, the subset

The owner can't read Rust. They audit the engine through three things: `engine/ARCHITECTURE.md`, the `//!` header of every file, and a script-checked trace from code to spec sections (G1 item 6, D5-1 to D5-7; RULEBOOK group 8, R-119 to R-131; R-73 to R-75). The rules work like an ESLint config: some are errors, some only reports, each with a tool or a reviewer behind it. None costs speed: newtypes compile away, and bounds checks measured negligible.

## Contents

1. The map
2. The file header
3. The trace
4. The subset
5. Shape and size
6. Readable strictness
7. The review's readability section

## 1. The map (R-119, R-121)

- `engine/ARCHITECTURE.md`, at most about 200 lines: a bird's-eye view, a codemap by pipeline stage, the invariants (including what is deliberately absent), the boundaries (library against CLI; the sanctioned I/O and thread files). It names modules and never links them. Written at S, updated only at milestones.
- One directory per pipeline stage, mirroring refcore's layout where it fits; `foo.rs`, never `foo/mod.rs`.
- **Names use the spec's words** (`law_date`, `cut`, `day_walk`, `read_rule`), units last (`days_max`), no abbreviations beyond a short list (`id`, `ctx`). ASCII identifiers only (R-71).

## 2. The file header (R-120, error)

Every `.rs` file starts with:

```rust
//! Books a day's legs at their places, parts included where a kind keeps them.
//! Spec: E §4.2 steps 1–3, §5; L §5.14.
//! Twin: engine/tools/refcore/walk.py.
//! Invariants: money conserved per batch (E §9.1); parts sum to the balance (R-115).
```

- The purpose is one sentence of about 15 words, and it still says what the file does after your change.
- `Spec:` names the L and E sections the file implements; `Twin:` the refcore file doing the same work; `Invariants:` what must hold.
- A fence adds a `PERF:` line (performance.md §2).

## 3. The trace (R-75, R-128)

- Each kernel function's doc comment cites the spec section it implements (`/// L §7.2`); missing there is an **error**, elsewhere a report.
- A citation (`L §n`, `E §n`, `l.N`) that doesn't exist in the spec is an **error everywhere**. Cite the section that says what the code does, not a neighbouring one.
- The `trace` script prints, per milestone, the reverse map: section → code → refcore twin → T's tests, and the sections with no code yet.

## 4. The subset (R-122 to R-125, R-73)

| Not this                                                 | This                                                                                                    | Level                                 |
| -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- | ------------------------------------- |
| `dyn` anywhere                                           | an `enum` and an exhaustive `match`                                                                     | error (ast-grep)                      |
| `impl Trait` arguments                                   | the concrete type                                                                                       | error (clippy `impl_trait_in_params`) |
| `macro_rules!` outside the lead's list                   | a named helper or a newtype method                                                                      | error (ast-grep)                      |
| own generics, a trait with one implementation            | concrete types; a trait only with two implementations today (`Table<I, T>` is the one expected generic) | report, then review                   |
| explicit lifetimes on long-lived types                   | lifetimes only on short-lived views (a cursor over a text, a borrowed slice)                            | report, then review                   |
| iterator chains over about 4 steps, closures in closures | `.iter().filter(…).map(…).collect()` at most; each closure one expression; else a named loop            | report, review                        |

Why: each of these hides what code runs behind an indirection the owner can't follow (a trait object, a generic, a macro expansion).

## 5. Shape and size (R-74, R-126, R-127)

- **Nesting at most 5** (clippy `excessive_nesting`, error). Use early returns (`let Some(x) = … else { return Err(…) };`), push conditions up to the caller and loops down into a step named by the spec's word. Never split mechanically (`step1`, `step2`) or hide a level in a closure.
- Function length (100 lines) and cognitive complexity (25) are reports; `too_many_arguments` (7) and `type_complexity` (250) are already errors. Thresholds are re-judged at S.
- **One call or expression per dispatch arm** (R-126): a `match` over node kinds hands each case to a named function.
- A file over 1 000 code lines is reported (R-127).

## 6. Readable strictness (R-131)

- **Domain newtypes with checked methods first:** `day.next()?` rather than `day.checked_add(1).ok_or(…)?` at every use.
- **Named helpers in one kernel helper module second;** each helper's doc names the rule it serves (`R-19`), the `?` stays at the call site, and the module is in the mutation kernel.
- Iteration instead of indices.
- No arithmetic macros, and no operator-like extension traits on integer types (a trait that adds methods to `u32`, like patching `Number.prototype`).

**Bad / good**

```rust
// Bad: noise at every use, and an extension trait that reads like std
let end = start.checked_add(len).ok_or(Refusal::days(at))?;   // repeated 40 times
impl CheckedExt for u32 { fn inc(self) -> Option<u32> { self.checked_add(1) } }
// Good: the domain type owns its checked step
let end = start.plus_days(len)?;    // Day::plus_days: L §4.1; R-19
```

## 7. The review's readability section (BUILDER §12 item 9, R-129)

Headers present and true; citations name the right section; the subset held; nesting ≤ 5 and no mechanical splits; no duplicated block over about 10 lines; optimised code only in fences with twin and test; the spec's names; `ARCHITECTURE.md` updated at the milestone; newtypes, not extension traits or macros. Each milestone ends with a generated one-page report for the owner: modules added and changed with their purpose lines, the spec coverage delta, new exceptions with reasons, size and nesting deltas, benchmark deltas, new fences.
