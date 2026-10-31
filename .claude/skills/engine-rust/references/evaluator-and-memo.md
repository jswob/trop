# The evaluator, the memo and the walk

The engine is a spreadsheet with a calendar: each def at its keys is a cell, computed when something asks for it and remembered in a memo. The design is `engine/spec/PERFORMANCE.md` §1–§4 (G1 item 1, D1-1 to D1-7); the code shape is BUILDER §3–§6. Speed comes from doing less work (skip empty days, share walks, key the memo well), not from a faster node: the evaluator's form is worth about 1.5× at most.

## Contents

1. The evaluator's shape
2. Recursion to a counted depth
3. The memo
4. The day walk
5. Projections, joint members
6. Errors, codes, impossible states

## 1. The evaluator's shape (D1-1)

- A **tree-walk over a resolved tree**: names resolved to ids at load, one exhaustive `match` per node kind, close to refcore's and the spec's structure, so a reader can follow L and E beside it.
- Each `match` arm is one call or one expression (R-126): dispatch stays flat, and the work sits in a function named by the spec's word.
- Closures (pre-compiled per node) come later, only in a fence (R-130), and only where a measured ask misses its budget (E1-like day-dense asks are the candidate).
- The tree: owned enums with a span on each node; operator and postfix chains kept **flat** (`Sum { first, rest: Vec<(AddOp, Expr)> }`), because a left-nested chain of 100 000 terms is 100 000 deep and every derived `Drop`, `Clone`, `PartialEq` and `Debug` would overflow the stack.

## 2. Recursion to a counted depth (K8-1, R-16)

- A def reading a def is a plain function call while a depth counter, passed down by value, is below its limit. Past the limit, the demanded cell goes on an explicit work stack (`enum Task { Demand(Cell), Finish(Cell) }` on a `Vec`, dependencies pushed in sorted order) and the outermost call replays it.
- The depth is **counted, never measured**: no stack probing, no `stacker` crate. So a result is the same on every machine and every worker count.
- An "in progress" set gives `RUN-CYCLE` for free.
- Before a replay, demand an aggregate's domain first, so an aggregate over many uncomputed keys doesn't go quadratic.
- Recurrences are filled iteratively in key order, never by recursion (E §3.2 l.155).
- Every thread, the 1-worker run's included, is spawned with an explicit stack size sized for the count (R-42).
- **Joint-model nesting is a worklist** (a to-do list a loop pops from), never recursion: a member that is itself a joint model has no limit and no refusal code in the spec (SO8-8).
- Parser depth: a `Depth(u8)` newtype with `deeper(at) -> Result<Depth, Refusal>`, called exactly where the spec counts a level (`LOAD-DEPTH`; `as if` has its own counter, `LOAD-ASIF`).

## 3. The memo (D1-2, D1-3, D1-6, D2-7)

- **The engine's own**, a few hundred lines; not a framework crate (salsa needs hashbrown and rustc-hash, banned by R-37). salsa's _ideas_ carry over: read sets, early cut-off.
- **The key:** (item, key values, an interned and trimmed context). Key values, not index positions (seam F); walk-bound cells also carry the walk point (E §3.2).
- **Contexts as layers:** scenarios, pin sets, axis combinations, members, paths and projections each are a layer, and a cell lives in the lowest layer its cone reaches. Trimming a projection out of a cell's context is a cache choice only where it can change no byte (PERFORMANCE §3.3; its one assumption is an open G2a item).
- **A recurrence keyed by day or period is a dense column** per (def, context): refcore's heaviest misses are three day-keyed defs carrying 95 % of 2.27 M misses.
- **One memo per context, freed when the context's scope ends** (a path, a search step, a projection). No eviction, so peak memory is deterministic.
- **Read sets are built now;** incrementality across amendments (invalidation, early cut-off, the walk restart) stays a seam until a measured turn misses 1–2 s (D1-5, PERFORMANCE §4).
- No hash map in the memo (K8-6): `BTreeMap` for sparse keys, dense `Vec` columns for day-keyed ones.

## 4. The day walk (D1-4; PERFORMANCE §2)

- **Event-driven:** only days that hold an act are stepped; every other day keeps the step function's last value. PERFORMANCE §2.2 lists what makes a day non-empty (N-1 to N-14), each derived from L and E with its citation. Build the agenda from that list, nothing else.
- **The oracle:** a step-every-day walk built from the same step functions stays beside the event walk, in tests only as a comparison, never shipped. A property test runs both on every case and compares the bytes. A day missing from the agenda shows up there.
- **Monthly checkpoints** first (the interval is measured at S); **timelines forked at their first different day** second, once the walk matches refcore byte for byte.
- The stratified order within a day (D-5) is the spec's; the walk never reorders acts for speed.
- Balance reads: `bal(x)` in the walk O(1); `bal(x, d)` by binary search over x's change points; `flows(x, span)` by two prefix-sum lookups.

## 5. Projections, joint members

- The written projection (`low`, `bal(x, d) from t`, `next`, D-6) and `value`: PERFORMANCE §2.3; its common-case read is micro row P8-M4.
- **Joint members (D1-7, seam J):** a member is its own context, keyed by its pin, with its own memo; its walk runs lazily, only up to the last day a joint read asks of it. Pruning a member to the places read comes second. Load row P8-L18 counts it.

## 6. Errors, codes, impossible states (BUILDER §5–§6)

- One `enum Code`, one variant per catalogue row (L §8, E §11; the count is recounted with `tools/xref_check.py`), with `ALL`, `as_str` and the message as exhaustive matches, and a unit test comparing them with the spec's tables (R-76).
- A refusal carries code, message, file, line and item. No `anyhow`, `eyre`, `Box<dyn Error>` or `String` errors (R-58).
- **Narrow the type at the point of decision** instead of `unreachable!`: a classifier returns `Option<AddOp>`, so the builder matches `AddOp` exhaustively. Take, don't peek: `let Some(first) = iter.next() else { return Err(…) };`.
- What remains impossible (a bad `ItemId`, a missing memo slot) returns an internal error, never a panic. An invariant break (E §9, and R-115's parts check) is `ENGINE-INVARIANT`.
- Load-time cycle checks: iterative DFS or Kahn's algorithm over nodes in sorted order.
