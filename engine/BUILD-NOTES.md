# Build notes for the engine (not normative)

What the Rust engine must leave room for, decided in sessions 6 and 7 (2026-10-30) without a language change. Each section names the seam: the one structure the engine should build from the start so that a later version, or a cost the spec already promises, needs no rewrite. `spec/` is the definition; nothing here adds to it or overrides it. Where this file and the spec differ, the spec wins.

## J. A joint model's members are their own evaluation contexts

- **Why.** LANGUAGE §5.18 says an ask costs its cone. refcore pays a member's whole day walk to read one of its rows (73.1 s on the fast layer), and a third member made one measured ask about four times slower (67.7 s against 17.1 s). The user's decision 6 (session 6): "ok, make it an engine requirement." No language change (tally 2.1's record R-15).
- **The seam.** Each member of a joint model is its own evaluation context, keyed by its pin (the sha256 of its canonical text, ENGINE §3.1 step 1), with its own memo, and its day walk is run on demand: a joint read of a member's balance asks that member's walk only for the places in its cone and only up to the day read, per (member, place set, last day). A member that is itself a joint model nests by the same structure. With parts (tally 2.1), a member's walk state includes its parts.
- **The test.** The load row shaped like E-1 (one row of a member with a long walk), its target the cone's cost (`tests/load/load.tsv`), and the nested joint pair card (`tests/cases/probes/pairs/PTY-05+OWN-12.md`).

## U. A slot for a range on every unknown

- **Why.** Item 4 of session 6 (ranges on unknowns) is research only, pending the user's choice: "we should research later the most optimal way to handle it." The research is `tmp/session7/research-ranges.md` (presented at gate G2 of session 7b).
- **The seam.** The hole value is a record from the first day: its questions, and an extensible slot for a bound, absent everywhere in tally 2.1, where an operation that doesn't understand the slot returns a plain hole, never a range it didn't prove. Each unknown carries which read it came from (item, keys, cut, pins), not only its question. Every operator's rule for holes is one function the operator calls, so a later version that narrows by bounds replaces that function. A hole part (LANGUAGE §8.1) keeps its amount for the same reason: a bound on one party's part is then its known parts plus the hole parts' amounts.

## N. The symbol table keyed by (path, sort, name)

- **Why.** Item 7.5 (one namespace) is parked for tally 3.0, with namespaces and modules. Relaxing tally 2's one-namespace rule later breaks no text.
- **The seam.** The symbol table is keyed by (path, sort, name): the path is a member prefix today and a module tomorrow; the sort is an item, a key of index K, a thing of kind K, a field, or a built-in. Tally 2's one-namespace rule (LANGUAGE §2.5) is a load check over that table, a name used in two sorts refused, never a property of how names resolve. The prelude's `parties` (tally 2.1) is one more built-in id under the same rule.

## F. The memo keyed by key values

- **Why.** Item 7.6 (user functions) is parked. A def keyed by a stepped range is already a total function of a bounded argument.
- **The seam.** The memo key of an evaluation holds key values, not index positions: (item, key values, pins, law date, cut, world). A later function is a def whose key domain is a type, memoised by value, with no change to the evaluator. Walk-bound cells also carry the walk point (ENGINE §3.2), and with parts the walk point's state includes the parts.

## R. "Is it random", per evaluation context

- **Why.** Item 7.7 (randomness under pins and the other path limits) is parked for 3.0. A pin that replaces every random input of a chain doesn't make the chain plain today (LANGUAGE §8.3's fixed list per item).
- **The seam.** The "is it random" mark is computed per evaluation context (an item and its pins), not stored on the item; tally 2's rule is the context with no pins.

## The labels' cost, paid only where a kind opts in

- **Why.** Tally 2.1's parts (LANGUAGE §3.6, §5.10) are walk state: a map from a label to an amount per (place, unit), beside the balance.
- **The seam.** A place whose kind is neither `owned` nor `batched` keeps one number per unit, as in 2.0, and pays nothing for labels: no map, no arrival rule, no take. Only owned and batched places carry the map, one entry per live label; an arrival or a take there costs O(L log L) for L live labels, and an undecided take merges only the parts it could reach. The memo and the walk must not make every place pay for the few that opt in, and the cut-K walk that labels the past (LANGUAGE §5.14.3) is the same walk that books the balances, not a second one.
