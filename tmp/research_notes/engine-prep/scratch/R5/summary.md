## Summary

1. **The owner reads the map, the trace and the reports, not the Rust.**
   - Strict projects explain themselves in two places: a short `ARCHITECTURE.md` (a codemap plus invariants, rust-analyzer [S1][S2]), and a `//!` header that every file must start with, enforced by rust-analyzer's tidy test [S4].
   - Tally adds what Cedar has, a readable twin: refcore. Cedar's Lean model is "an order of magnitude smaller" than its Rust and is held to it by differential tests [S8].
   - Each engine module names its refcore twin and its spec sections.
2. **A small Rust subset reads better at no loss of speed.**
   - Closed `enum`s with exhaustive `match`, never trait objects (`dyn`), which prevent inlining [S13].
   - No own generics, `impl Trait` arguments, library macros, or new traits without a counted reason.
   - Short iterator chains: loops and iterators benchmark alike, 19.62 against 19.23 ms [S16].
   - ast-grep matches every one of these constructs [S11].
3. **Strictness is not where speed goes.**
   - Readyset removed all bounds checks from a production Rust engine and called the cost "negligible" [S31].
   - `shape.md` puts the order of magnitude in structure: empty days, shared walks, the memo, the read index.
   - That code goes in fenced modules, each with a benchmark, a plain twin and an explanation (inference).
4. **K-21 changes in part.**
   - Clippy's own complexity score adds **1 for a whole `match`, whatever its arm count** [S18]. A long dispatch `match` isn't the problem; nesting is.
   - `excessive_nesting` becomes an **error at 5**, as A1 proposed. Length stays a report at 100 lines; the studies conflict [S23][S24].
   - `too_many_arguments` (7) and `type_complexity` (250) are **already errors**, through the scaffold's denied `complexity` group.
5. **The spec trace is cheap and checkable.**
   - The spec is frozen, so `L §5.11` and `L l.1451` never go stale.
   - A ~150-line script fails on cited sections that don't exist, requires citations in the kernel (R-75 becomes an error there), and prints the reverse map: section → code → refcore → tests.
   - refcore already cites 69 sections this way. Prior art: the Rust Reference's `//@ reference:` test headers [S27]; rustc tidy [S22].
6. **Readable strictness:**
   - Domain newtypes with checked methods (`DayNo::next()?`), which cost nothing ("elided at compile time" [S33]).
   - Named helpers in one kernel module.
   - Iteration instead of index arithmetic.
   - No macros or operator-like extension traits, which hide checks from clippy and mutation testing.
7. **Decisions D5-1 to D5-7:** layout and names; the subset; size limits; the trace; the owner's audit kit; performance fences; readable strictness. Each is made of rows the owner can approve one by one, like the rulebook's.

---

