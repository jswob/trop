# R2: data and numbers at speed

Read `tmp/session8/agents/common.md` first; it holds the session brief's §1–§2, the agent rules, and the output format. You write `tmp/research_notes/engine-prep/R2.md`.

## Your question

How the engine should hold its numbers and its data so it is fast, exact, deterministic, and within the 8 GB cap. Speed is the user's core concern; the answer must stay buildable under phase 1's strict rules and readable by its owner.

1. **Exact rationals with a small-value fast path.** How GMP (`mpq_t`, `mpz` small values), CPython's ints (compact ints, 30-bit digits) and `fractions`, and the Rust crates (num-bigint and num-rational, dashu, malachite, ibig, rug, and any other maintained candidate) hold small values inline and fall back to big integers; their gcd algorithms (binary, Lehmer, half-gcd) and where each wins; licences (malachite and rug/GMP are LGPL: what that means for a shipped binary), maintenance, `unsafe`, panics, and checked APIs. Set this against phase 1's pick (A3; RULEBOOK R-30: num-bigint 0.4 under the engine's own `Num` with an i64 tier) and A3's finding that num-bigint's gcd makes a reduced operation about 10× slower than PERFORMANCE §6 assumes.
   - **shape.md's evidence:** over 99.9 % of refcore's results fit 64 bits; 77–99 % are integers; one household check (E1, a mortgage) reaches 5 049 bits (1 109 results over 4 096 bits; the cap is 13 606 bits). Say what representation fits that distribution best (i64/i64 inline, an i128 tier, the big tier), and whether the pick of R-30 should change.
2. **Money at a fixed minor unit.** Whether money can be scaled integers (a whole number of minor units, i64 then i128 then big) where the spec's exactness allows (LANGUAGE §3.1 the money type, §7.1–§7.2; PERFORMANCE §6). shape.md found every money value in six checks to be a whole number of cents. Say exactly where a rational must appear (a rate times money before its `round`) and what that costs.
3. **String interning** (seam N, the symbol table keyed by (path, sort, name)). BUILDER.md §3 said "names are `Box<str>` plus a `BTreeMap`; no interner", and RULEBOOK R-47 says an id must never decide output order. How rustc, rust-analyzer, ruff and the interning crates (lasso, string-interner, others) do it, whether interning pays here, and how to keep it deterministic.
4. **Arenas and index-based graphs instead of pointers.** Typed index vectors and arenas (index_vec, typed-index-collections, id-arena, slotmap, la-arena; phase 1 found la-arena panics and its indexing is invisible to the `indexing_slicing` lint), and how rustc and rust-analyzer do it. Under the rule "no indexing" (R-5), what the safe accessor costs.
5. **Compact storage** for versions, rows, observations, the walk's logs and change points, and tally 2.1's parts (a map from a label to an amount per place and unit, LANGUAGE §3.6; BUILD-NOTES "the labels' cost"): struct-of-arrays against array-of-structs, small vectors, sorted vectors against B-trees, and persistent maps for overlays (scenarios, pins, checkpoints: im, imbl, rpds) with their determinism.
6. **Maps with a deterministic order.** BTreeMap, IndexMap, sorted Vec, and hash maps with a fixed hasher whose iteration never reaches output (phase 1 bans hash maps in what ships, `deny-tally.toml`; R-39–R-47). Speed trade-offs with sources.
7. **Memory against the 8 GB cap** at the sizes of `engine/tests/load/benchmarks.md` (refcore reached 83 GB on B-07's full span; 10 000 paths for D-10): an estimate per memo cell, per row, per movement and per path, marked as inference where it is.

## Read (by range; don't read whole)

- `tmp/session8/shape.md` (whole, one page).
- `tmp/research_notes/engine-rust-strictness/A3.md`: §2 (takeaway), the §3 findings on crates, gcd and the fast path (F-1 to F-16 as you need them), §5, §7. `NOTE.md` §1 items 3 and 14, §2 rows K-11 and K-12, §4 rows SO-12 to SO-24. `RULEBOOK.md` group 2 (lines 60–84) and group 3 (85–107). `BUILDER.md` §2, §3 and §7.
- `engine/spec/PERFORMANCE.md` §6 (lines 239–273), §9 (335–360); `LANGUAGE.md` §3.1 (361–392), §3.6 (466–540), §7 (2090–2197); `engine/BUILD-NOTES.md` (whole).
- `engine/tests/load/benchmarks.md` and `load.tsv`.

## Decisions to frame (merge or split as the evidence shows)

The number representation and the big-integer crate (may change R-30); money as scaled integers; interning; arenas and ids; the deterministic collections policy; overlays (persistent maps or layers).
