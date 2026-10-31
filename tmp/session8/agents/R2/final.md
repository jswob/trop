<!-- saved by tools/save_final.py from agent agent-a4651ade0b99e73ac.jsonl (R2) -->

I wrote `/Users/jswob/projects/trop/tmp/research_notes/engine-prep/R2.md` (34 384 bytes, a little over the ~30 KB guide). It has every section common.md §C asks for. I made no scratch notes, ran nothing but `python3 -I -c` arithmetic, and never touched `tmp/holdout/`.

**The three findings that matter most:**
1. **Write the Lehmer gcd now, not "only if benchmarks ask" (BUILDER §2).** The cap is 213 limbs (64-bit words) and E1 reaches 79. GMP only switches to its faster large-number gcd above 618 limbs (Skylake) or 330 (arm64), so Lehmer is the ceiling for Tally. GMP, CPython, malachite and dashu all use Lehmer; num-bigint alone still uses the slower binary one. On E1's large values, an engine on num-bigint's gcd would likely reduce fractions slower than refcore's own Python (inference).
2. **No i128 tier for `Num`.** In E1, about 1 664 results pass 63 bits and 1 693 pass 1 024 bits, so a value that leaves 64 bits is almost always far past it. Use i64/i64 inline plus big, with i128 only for intermediate products. Add a `d == 1` shortcut, since 77–99 % of results are integers. A 16-byte `Num` is possible; the first compile confirms it.
3. **The memo is the only memory risk, and it fits 8 GB.** Estimates (inference): 70–90 B per general memo cell, 24–32 B per cell in an array indexed by day or period, against refcore's 2 180 B per cell. For X that is 0.07–0.2 GB instead of 4.95 GB. B-07 would be about 3.3 GB if nothing were freed, but that run never finished, so the real figure is unknown. Hence one memo per context, freed when the context ends.

**Decisions and picks:**
- **D2-1:** keep num-bigint 0.4.8 (R-30 stands); `Num` = Small(i64/i64) or Big; add our own Lehmer plus the cheap path for big × small.
- **D2-2:** money is whole minor units, i64 or Big; drop phase 1's i128 money tier.
- **D2-3:** one hand-written interner (names stored once and compared as `u32` ids), output ordered by text as rustc does. This replaces BUILDER §3's "no interner".
- **D2-4:** our own `Table<I, T>` with `get` only and no `Index` impl; no arena crate.
- **D2-5:** sorted B-tree maps by default, sorted slices for frozen tables, arrays for memo cells keyed by position, no small-vector crate. A no-iteration hash memo (rustc's `UnordMap` design) waits for G1, because it needs an R-36 exception.
- **D2-6:** walk checkpoints as `Vec<Rc<PlaceState>>` with copy-on-write; pins and scenarios as at most 8 layers; no persistent-map crate.
- **D2-7:** one memo per evaluation context, dense where the key is a position, freed when the context ends; decide on eviction after an engine run of B-07.

**What you should know:**
- **Contradictions with phase 1:** BUILDER §2 (Lehmer timing, money's i128 tier) and BUILDER §3 (no interner).
- **R-37's text needs one clause:** it bans hashbrown outright, but the scaffold's `deny-tally.toml` lets indexmap wrap it and BUILDER §7 allows IndexMap. It should say "except under indexmap".
- **Crates checked today:**
  - num-bigint's newest is 0.5.1; the 0.4.8 pin is still current in the 0.4 line.
  - malachite is at 0.14.0 and depends on libm unconditionally, which phase 1's K-8 bans.
  - lasso and string-interner both need hashbrown, so they are blocked.
  - `im` is flagged unmaintained (RUSTSEC-2026-0248).
  - imbl pulls `rand_core` and `rand_xoshiro` into what ships.
  - smallvec has five advisories.
- **Sources I couldn't reach:**
  - gmplib.org's repository returned 403, so the GMP thresholds come from the GitHub mirror alisw/GMP.
  - gnu.org's FAQ returned 429, so the LGPL reading comes from the SPDX licence text.
