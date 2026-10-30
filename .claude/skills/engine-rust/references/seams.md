# The seams, and tally 2.1's structures

What the engine must leave room for from the start, so a later version, or a cost the spec already promises, needs no rewrite (`engine/BUILD-NOTES.md`, decided in sessions 6 and 7). Nothing here adds meaning: `engine/spec/` is the definition, and where they differ the spec wins. This file also gathers tally 2.1's structures that touch several modules: parts and labels, holes, joint members.

## Contents

1. J: joint members as their own contexts
2. U: a slot for a range on every unknown
3. N: the symbol table keyed by (path, sort, name)
4. F: the memo keyed by key values
5. R: "is it random" per evaluation context
6. Parts and labels: paid only where a kind opts in

## 1. J: joint members as their own contexts

- Each member of a joint model is its own evaluation context, keyed by its pin (the sha256 of its canonical text, E §3.1 step 1), with its own memo (D1-7).
- Its day walk runs on demand: a joint read of a member's balance asks that member's walk only for the places in its cone and only up to the day read, per (member, place set, last day). Pruning to the places read is the second step; the lazy walk to the last day read is the first.
- A member that is itself a joint model nests by the same structure, **walked as a worklist, never recursion**: the spec gives that nesting no limit and no refusal code (R-16, SO8-8).
- With parts, a member's walk state includes its parts.
- The test: load row P8-L18 (one row of a member with a long walk; its target is the cone's cost), and the nested joint pair probe `U:set/pairs/PTY-05+OWN-12/ania` (P8-15).

## 2. U: a slot for a range on every unknown

- A hole is a record from the first day: its questions, and an extensible slot for a bound, **absent everywhere in tally 2.1**. An operation that doesn't understand the slot returns a plain hole, never a range it didn't prove.
- Each unknown carries which read it came from (item, keys, cut, pins), not only its question.
- **Holes and hole parts are built only in one module** (R-116): their constructors and the per-operator rule for holes are private there (like a function a TypeScript module doesn't `export`), and every operator calls that one rule. A later version that narrows by bounds replaces that one function.
- A hole part (L §8.1, `HOLE-PART`) keeps its amount, so a total over known parts and hole parts stays known.
- A missing value is a hole, never 0 and never a default (R-27).

## 3. N: the symbol table keyed by (path, sort, name)

- The path is a member prefix today and a module tomorrow; the sort is an item, a key of index K, a thing of kind K, a field, or a built-in.
- Tally 2's one-namespace rule (L §2.5) is a **load check over that table**, a name used in two sorts refused, never a property of how names resolve.
- The prelude's `parties` kind (2.1) is one more built-in id under the same rule.

## 4. F: the memo keyed by key values

- The memo key holds key values, not index positions: (item, key values, pins, law date, cut, world). A later user function is then a def whose key domain is a type, memoised by value, with no change to the evaluator.
- Walk-bound cells also carry the walk point (E §3.2); with parts, the walk point's state includes the parts.
- Key values are interned symbols ordered by their text (data-layout.md §2).

## 5. R: "is it random" per evaluation context

- The "is it random" mark is computed per evaluation context (an item and its pins), not stored on the item. Tally 2's rule is the context with no pins (L §8.3's fixed list per item).

## 6. Parts and labels: paid only where a kind opts in

Tally 2.1's parts (L §3.6, §5.7, §5.10, §5.14; CHANGES 2.1-1 to 2.1-19) are walk state: a map from a label (an owner, a batch) to an amount per (place, unit), beside the balance.

- **A place whose kind is neither `owned` nor `batched`** keeps one number per unit, as in 2.0, and pays nothing: no map, no arrival rule, no take.
- **Owned and batched places** carry the map, one entry per live label, held as a sorted label vector; an arrival or a take costs O(L log L) for L live labels; an undecided take merges only the parts it could reach.
- **The largest-remainder split is written once** (R-114) and used by `prorata … rest largest`, every arrival's split among owners and every take ending in `prorata`. Ties go only by the written `ties by`; with none, `RUN-TIE`. Never by a map's order, an id or insertion order (R-47).
- **After every arrival, take and relabel,** each (place, unit) that keeps parts checks that its parts sum to the balance, are ≥ 0 and are whole numbers of the kind's `step`. A failure is `ENGINE-INVARIANT`, checked at run time on every run, a real check, never a `debug_assert!` (R-115).
- Labels are walk state at cut K; the cut-K walk that labels the past is the same walk that books the balances, never a second one (L §5.14.3).
- `same` is refused on a place that keeps parts; labels in ties, `same` classes and member prefixes are ordered by explicit keys (R-47).
