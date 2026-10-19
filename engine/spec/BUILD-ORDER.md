# Tally: build order

The Rust step (B) in milestones. Each milestone builds a slice of `LANGUAGE.md` and `ENGINE.md`, and is done when a named set of tests turns green and every earlier milestone stays green. The tests come from step T (the executable suite), which is written from this design before B starts:

- **case tests:** a case's model in `models/`, run, compared with the figures its `expected/` file points to;
- **goldens and pins:** the prototype's G1–G10 and 36 pins (`tmp/verification/goldens.md`), over the canonical projection of a result (CHG-14);
- **the household:** the synthetic household's reality and the seven changes in `amendments/`, each with the outputs it changes;
- **hindcast fixtures:** H0–H9 (`tmp/verification/hindcast.md`), each a synthetic ledger with one planted cause;
- **refusal fixtures:** one small file per code in LANGUAGE §8, ENGINE §11 and OPERATIONS §10 (87 codes: 79 refusals and holes, 8 warnings), which must give exactly that code. `ENGINE-INVARIANT` goes through a fault-injection hook, since a correct engine never gives it;
- **properties:** determinism, conservation, no foresight, canonical formatting, and the others below.

The household and replay come first (M1, M2), then the operations that change them (M3), so the product works on the household before the breadth of the 79 cases is built. Each case goes in the first milestone that has everything it needs. The assignment below comes from LANGUAGE's case lists and the case index. The D6 models confirm it, and `coverage.md` gives the final construct-by-case map. A case that moves is moved here.

**Programming analogy:** a tracer bullet, then a compiler built front end first. The tracer slice (S) runs one thin path through the whole engine on the household. Then M0 parses and type-checks every model before any runs, and each later milestone adds a part of the runtime.

---

## Summary

| M | Name | Builds | Cases turning green | Other tests |
|---|---|---|---|---|
| S | The tracer slice | the household's syntax only, exact numbers, the read rule, the day walk, booking, holes, re-anchoring, a minimal `explain` | none: the household | the household's vectors; the kernel's properties; mutation score ≥ 90 % on the kernel |
| M0 | Load and format | syntax, canonical format, names, types, units, the read rule, load refusals | none run; all 79 load | `LOAD-*` fixtures; format round-trip |
| M1 | The household forecast | exact numbers, calendar, defs, bands, axes, pins, places, pays, the walk, `plan_low`, scenarios, asks, the register | 5 | the household and changes 1, 2, 4–7; goldens G1–G3, G6–G10; the pins |
| M2 | Replay against reality | records, the import's rules, booking at the cut, corrections, back pay, the modelled past, the residual ledger, `as known` | 6 | the household's replay; change 3; G4, G5; H1–H5, H8, H9 |
| M3 | The operations | the store, the amendment pipeline, the three change kinds, `plan`, the gate, scenario life, the rest of the surface, the reference card | — | the seven changes as `plan` results; the gate on one law part with planted errors; `OP-*` fixtures |
| M4 | Law breadth | `excepts`, `missing()`, `given`, Kleene logic, `argmin`, every rounding mode, FX and other calendars | 34 | — |
| M5 | Allocation, holdings, many parties | `fill`, lots by vintage, claims, look-through, holdings in units, long histories | 25 | — |
| M6 | Exact functions and iteration | transcendentals `to u`, `dcf`, `factor`, exact distributions, scores, named iteration, `solve` in law | 9 | — |
| M7 | Monte Carlo | draws by address, paths, the shared layer, plan values, lifetimes, path statistics, replay of one path | 0: no model draws (`coverage.md` K13) | H0, H6, H7; the properties |
| M8 | The evaluation layer | rolling origins, `as if {known}`, scores of the model's own forecasts, the fitting loop | — | H0–H9 with pooled origins; the fitting step |
| M9 | Performance (step F) | the budgets of PERFORMANCE §8 measured and met | — | the benchmark suite |

5 + 6 + 34 + 25 + 9 + 0 = 79. Three steps are declared bounds and never turn green: MP-15's fewest transfers, LH-17 step 6 and CH-03 step 14 (LANGUAGE §5.17). Their cases turn green without them. LH-18's step 6, a bound in the spec, is computed exactly in Tally (BOUNDS row 49).

The cost budgets are checked from M1 on, not left to M9: each milestone's exit includes the budget rows it touches, at 1× size. M9 is where 10× size, the warm memo and the parallel paths are tuned.

---

## S. The tracer slice (before M0)

Accepted with the build plan on 2026-10-19. It is a thin path through the whole engine, in production code, so the riskiest part, the time model, runs first. Nothing in it is thrown away: M0 to M2 broaden it.

**Builds**
- A parser for the syntax the household, its records and its asks use, and no more. The full front end is M0.
- Exact rationals and `round` (L §7.1, §7.2).
- The read rule with its cut (L §4.5, §4.6).
- Indexes, places, inputs, defs, bands and pays; the flow pass, and the day walk with `plan_low`, holds and promised movements (E §4.1, §4.2; L §5.13).
- Booking at the cut, coverage and the modelled past, re-anchoring at `stated` rows, and the residual ledger (E §6; L §5.18).
- Holes with their questions (L §8.1), and a minimal `explain`: the derivation and the register (L §5.16, §8.2).

**Green**
- The household's asks match its hand-checked vectors (REPORT §13; `smoke/HH4-a.md`, `smoke/HH4-b.md`), through T's vectors.
- The kernel's properties hold under property tests: determinism, conservation, no foresight, a hole never becomes 0.
- The mutation score is at least 90 % on the kernel: the read rule, rounding, the walk and booking.

**Not in it:** the full grammar and the load catalogue (M0); scenarios, `as known`, back pay and corrections (M1, M2); everything from M3 on.

---

## M0. Load and format

**Builds**
- The lexer, the grammar and the parser (L §2.1–2.3).
- The canonical formatter, one format with no options (L §2.4).
- Names, stable ids, reserved words, one namespace (L §2.2, §2.5).
- Types and units, the unit algebra, rates, price years (L §3).
- Spans, periods and anchored indexes (L §4.1).
- Versions, the knowledge cut, the read rule with its default rows (L §4.5, §4.6, §5.3).
- The item graph, static cycles, the load catalogue (E §3.1, L §8.3).

**Green**
- Every model in `models/` and the household's files load with no refusal. True on paper since D8: the 8 models the audit found refused were renamed, and the static scan `tools/loadcheck.py` (keywords, one namespace, shadowing, built-in ids and band names, ask names against keys) gives zero refusals over the 79 models, the household and the changes' added lines. Its first run found three more models refused (RC-06's and UL-04's asks named for keys, LH-09's source named like an axis option) and two changes (an added key named like its version, an optional ask named like a version); all were renamed. M0 runs it again (`--selftest` first), and then the real loader, which also checks what the scan can't: names an expression resolves, units, intervals.
- `format(parse(x)) = x` for every one of them, and `format` is idempotent.
- Each `LOAD-*` code has a fixture that gives exactly that code and message.

**Properties:** the parser is total: no input crashes it, and every rejected input gets a code (fuzzed); nesting stops at 8 levels (`LOAD-DEPTH`).

## M1. The household forecast

The deterministic core, enough for the synthetic household: a B2B earner, an employee, Polish law, a mortgage, savings.

**Builds**
- Exact rationals in i64, i128 and bignum, with the `RUN-DIGITS` cap; the literal rule; `round` with its seven modes (L §7.1, §7.2; P §6).
- The calendar built-ins, holiday inputs, the law date and its propagation, imprecise dates (L §4.2–4.4).
- `index`, `input`, `def` and `rule` with `when`, recurrences with `first`, `band`, `axis` with the 64-fork cap, `prorata` (L §5.5–5.10, §5.12).
- Pins: `with` and `as if`, including the law and known pins (L §5.10).
- Places and their kinds, `holds` and `free`, `owns` (L §5.9, §3.6).
- Flow pays and state pays, `flows`, `bal`, `worth`, `next_on`; the flow pass, the day walk, the order within a day (L §5.13; E §4).
- `plan_low` over a place or a group, by a segment tree (E §4.2; P §2.3).
- Booking at the cut, with a booked amount fixed (E §5), without records.
- Scenarios as layers (L §6.4).
- Asks, including `solve` and `check`; `real`; holes, the register, warnings (L §5.16, §8).
- `explain`, provenance on every movement (E §10).
- Conservation and the other run invariants (E §9).
- The canonical projection of a result: movements and end-of-day balances in a fixed order (CHG-14).

**Green**
- Cases: UL-01, UL-02, UL-05, LH-05, LH-10.
- The household's forecast, and changes 1, 2, 4, 5, 6 and 7, each giving the outputs `amendments/` lists.
- Goldens G1, G2, G3, G6, G7, G8, G9 and G10, and the pins, re-stated on the projection. Four CHG-14 tasks come first:
  - ownership stated as facts (G1, G3);
  - G6's raise says how it meets the wage index (L-03);
  - the pins that the joint-return eligibility (L-26) may move are re-derived, and each move is explained;
  - the digests are taken over the projection.

**Properties**
- **Determinism:** the same files give the same bytes on every run, thread count and platform.
- **Conservation:** money is conserved on every movement, outside places included.
- **No foresight:** changing a draw's value, a state pay's later result or any row after the look-ahead's window never changes a state pay's decision before it (C-13).
- **Holes:** no hole ever becomes 0. A fixture per hole kind checks it stays a hole.

**Budgets touched:** a cold deterministic run at 1× (≤ 0.1 s, a tenth of the 10× budget), one look-ahead decision ≤ 0.3 µs.

## M2. Replay against reality

**Builds**
- Records: movement inputs, `covers`, `stated`, `explains`, `missing()` of a record (L §5.18; L §5.10).
- Anchors: a `stated` row the cut sees re-anchors its place's balance at the close of its day, with the difference on the ledger's balance line (E §4.2, §6.1, §6.3; J70, D8).
- The import's rules as data, `account_place`, `party_place`, `match_days`, `match_amount`, and the import operation's matching, holds and duplicates (L §5.18; O §2.2).
- Booking with records: the cut per class, the covered past (E §5, §6.1).
- `correction` versions; `arrears on D | none [of V] for <pays>`; `HOLE-CHANGED`; `enters`, `moves V to SPAN` and the `wording` pin (L §4.3, §5.3, §5.13.3).
- The modelled past, computed and never booked (E §6.2).
- The residual ledger, its lines and their questions (E §6.3).
- The four clocks in full: S, K, (K, E], R; `as known` (E §2).

**Green**
- Cases: LH-14, RC-08, UL-06, UL-09, UL-15, CH-04.
- The household's replay over its synthetic ledger, and change 3 (the retroactive law).
- Goldens G4 and G5 (G5 with the fixture's flat price index pinned).
- Hindcast fixtures H1, H2, H3, H4, H5, H8 and H9, each landing in the line its answer key names. H0, H6 and H7 are banded, so they wait for M7.

**Properties**
- **Masking at K:** changing any record or version known after K leaves every result as of K unchanged (H8 is the metamorphic case).
- **The three-way split:** replay minus actual is 0 on every fixture but H3.
- **Fixed past:** no later version changes a booked amount; it adds a correction movement (E §5).

**Budgets touched:** one replay of two years ≤ 20 ms (the fitting step's floor, M8).

## M3. The operations

This is the surface Claude uses through MCP, built without fixing any MCP technology. After M3, the household can be kept by amendments.

**Builds**
- The store: drafts, `tested`, `confirmed` against a content hash, authors, record time R (O §1).
- The amendment pipeline and its typed results (O §1).
- Add a fact, import records, add a law version, amend with the four kinds (add, advance of `on`, restate proven by equal results, revise with a reason), end, scenario patches and their life (O §2–6).
- `plan`, the outcome diff (O §7), and saved analyses (O §8).
- The gate in its seven steps, with planted errors and the threshold table (O §3).
- The rest of the surface: snapshot, catalog, describe, record, confirm, run, query, explain, history, reconcile, drill, trial (O §9).
- Language versioning and migrations (L §9).
- The reference card: the catalogue, cut from LANGUAGE §2, §5 and §8 to about the prototype's 13k tokens (A-01).

**Green**
- The seven changes, each sent as an amendment: `plan` lists exactly the outputs `amendments/` says it changes, and none other.
- The gate on one law part (the household's Polish PIT part with its published checks): it passes, its planted errors are caught at ≥ 80%, and one errata check (L-41) reports its gap.
- Each `OP-*` code and warning has a fixture.

**Properties**
- **Restate proven:** a restate whose results differ is refused (`OP-RESTATE`).
- **Status follows content:** any edit to a confirmed part's content drops it back to draft.
- **Append-only:** no operation removes a stored statement.

**Budgets touched:** a conversation turn on the household, 1–2 s; the gate on one law part, ≤ 1 s.

## M4. Law breadth

The constructs that tax and benefit law needs beyond the household's: exceptions, branches, eligibility, other countries' calendars and money.

**Builds**
- `excepts`; `missing()` in law with `v`; `given` (L §5.10).
- Kleene `and` and `or`, the aggregates table, `argmin`, `argmax` (L §5.10).
- Every rounding mode and step, and the rounding axis (L §7.2).
- Several currencies: FX rates as `num A/B`, `flows … in CUR at fx mode`, exchanges as two pays (L §3.2, §5.9, §5.13.1).
- Calendars per country (L §4.2).

**Green (34)**
- Cross-border: CB-01, CB-02, CB-03, CB-04, CB-06, CB-07, CB-08, CB-11, CB-12.
- Lifetime: LH-07, LH-08, LH-12, LH-13.
- Many parties: MP-05, MP-06, MP-07, MP-08, MP-10, MP-11, MP-13, MP-16.
- Reference calculators: RC-02, RC-04, RC-05, RC-07.
- Unusual law: UL-03, UL-04, UL-10, UL-11, UL-12, UL-14.
- Checks: CH-05.
- From M7, since their models don't draw: MP-14 (the lot's winner is stipulated) and CMP-02, its composed case.

## M5. Allocation, holdings and many parties

**Builds**
- `fill`: greedy allocation in a stated order, with its refusals (L §5.11).
- Lots as vintage-keyed inputs and defs, matched by `fill` (CHG-01).
- Carry-forward and carryback by `fill` over years (CHG-04).
- Holdings in units, priced; claims; debts with contract state (L §5.9).
- Entities, `stake`, look-through by a bounded depth (L §3.6).
- Groups and relations with dates (L §3.5).
- Long histories as input rows (LH-15's 145 years of monthly data).

**Green (25)**
- Cross-border: CB-05, CB-09, CB-10, CB-13, CB-14.
- Lifetime: LH-01, LH-02, LH-03, LH-04, LH-06, LH-09, LH-15, LH-16.
- Many parties: MP-01, MP-02, MP-03, MP-04, MP-09, MP-12, MP-15 (without the declared bound).
- Reference calculators: RC-01, RC-03, RC-06.
- Unusual law: UL-13.
- Composed: CMP-01.

## M6. Exact functions and iteration

**Builds**
- Transcendentals `to u` by interval refinement, up to 2^16 bits, and the `≈` bound when no `round` follows (L §7.3).
- `dcf` with its day counts and `factor` (L §3.3).
- Exact distribution functions without paths: CDFs, quantiles, the exact binomial (L §5.14).
- Scores: `crps`, interval scores, skill, `orient` (L §5.15).
- A named iteration, and `solve` in a def where the law names the method (L §5.10, §5.16).

**Green (9):** UL-07, UL-08, LH-11, CH-01, CH-02, CH-03 (without step 14); from M7, since their models compute exactly and don't draw: LH-17 (without step 6), LH-18, CH-06.

**Properties:** every transcendental's printed digits are correct (checked against a second method at a higher precision); the refinement stops or refuses, never returns a wrong digit.

## M7. Monte Carlo

**Builds**
- Draws by SHA-256 address, the canonical draw order, every distribution (L §5.14, §7.4).
- Paths, the shared layer and the per-path layer (E §7).
- Draws' plan values in the look-ahead (`LOAD-PLAN`).
- Lifetimes per party per path, `alive` (CHG-08).
- Path statistics with standard errors, `quantile`, `share`, paired paths (L §5.16).
- The `share` verdict, sayable or not (spec Q-03, E §8; D8): the standard error test (Agresti–Coull), the carry of `held` and `assumed` figures over the plan movements at plan values, the horizon, and a pin on a draw. It reads only what the shared layer holds, so it adds no evaluation.
- One path replayed for an explanation (E §7, §10).

**Green**
- Cases: none. No model draws (`coverage.md` K13): LH-17, LH-18 and CH-06 compute exactly and go to M6, MP-14 and CMP-02 stipulate the lot and go to M4. The household draws nothing either.
- Hindcast fixtures H0, H6 and H7, on a single origin: the only tests of Monte Carlo before step P adds its own.

**Properties**
- **Common random numbers:** two scenarios share every draw whose address they share (S-11).
- **Order independence:** the result doesn't depend on the number of workers or the order paths finish.
- **No foresight on paths:** the look-ahead's result is the same on every path (it reads plan values only).

**Budgets touched:** 1 000 paths × 30 years ≤ 1 s on 6 workers, at 1×; one path replayed ≤ 20 ms.

## M8. The evaluation layer

**Builds**
- Rolling origins: `as if {known origin_day[o]}` over an index (E §8).
- Scores of the model's own forecasts against records, refused where invalid (E §8).
- The test mode for statistics: a parameter draw pinned to its mean, in the register (CHG-13).
- The fitting loop's cycle: pin, replay, residuals (E §6.4).

**Green**
- Hindcast fixtures H0–H9 with pooled origins at months 24, 36 and 48, each banded verdict stating its power.
- The refusals of `hindcast.md` §4: MAPE on a series that crosses zero, a band from too few paths, a read of the answer key before the model hash is frozen.

**Budgets touched:** one fitting step ≤ 20 ms.

## M9. Performance (step F)

Every budget in PERFORMANCE §8, at 10× the prototype's size: the turn, the cold and warm runs, Monte Carlo with law on every path, the look-ahead, path replay, the gate and the fitting step. The strategies in PERFORMANCE §3–5 come in here where M1–M8 didn't need them:
- cut classes trimmed per item;
- verifying traces with early cutoff, and durability tiers;
- the walk restarted from monthly checkpoints;
- the memo kept across amendments.

Optimisations stay invisible: every earlier test, the canonical projection and the digests are unchanged by them. PERFORMANCE §10 lists what this step must settle, such as grouping aggregates over open indexes.

---

## What every milestone keeps

- **Regression tests from the prototype's bugs** (C-16): no rounding helper that loses a cent; FX only between different currencies, by the unit types.
- **The refusal catalogue:** each code added to LANGUAGE §8 or OPERATIONS §10 gets its fixture in the milestone that builds its construct.
- **No silent default:** a test per construct that a missing row gives a hole, never 0 or a guess.
- **The bounds:** BOUNDS.md rows marked "designed untested" or "gap designed" are re-marked when their milestone's tests pass, and only then.
