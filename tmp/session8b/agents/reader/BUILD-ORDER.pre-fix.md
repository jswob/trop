# Tally: build order

The Rust step (B) in milestones, for **tally 2.1**. Each milestone builds a slice of `LANGUAGE.md` (L) and `ENGINE.md` (E), and is done when its named checks are green and every earlier milestone's stay green. It plans the engine and carries no language meaning (G1 item 9): what a construct means is L's and E's, and a sentence that would need more went to the open list instead (DONE rule 3). Rewritten in session 8b (2026-10-30) from session 8's G1 decisions (`tmp/session8/decisions.md`: D1-1..D6-5, K8-1..K8-17) and the spec observations (SO8-n, `tmp/research_notes/engine-prep/NOTE.md` §3). It replaces 1.0.1's text, which `STALE.md` listed.

**The tests the milestones name** exist now, in `engine/tests/` (written from the frozen spec and the answer keys, never the builder's: DONE rule 2):
- **`checks.tsv`, 634 checks**, each run on the frozen reference interpreter `engine/tools/refcore/` and stored as its output; the engine must print the same bytes (DONE gate 10):
  - 154 **unit probes** `U:`, one card per edge of the capability catalogue `cases/boundaries.md` (99 edges on 15 axes), plus pair cards and probe sets;
  - 479 **integration** checks `I:`: the old models (`I:M:`: 77 of the 79, and two key scripts), the households A, B, C, D, E1, E2 one check per ask, the amendments household X (`I:X:`), the light twins of the heavy models, the blind texts (`I:hardset/`) and session 7's blind cards S1–S6;
  - **`R:all`**, refcore's own tests: the oracle's health, not the engine's.
- **The load rows** `L:n` (`load/load.tsv` and `load/benchmarks.md`: 17 rows numbered L:1–L:18, with no L:7), the full-size runs no refcore job finishes. They are the engine's size targets, budgeted as PERFORMANCE §8's rows P8-L1..P8-L18, numbered as their rows (none for the missing L:7). 85 rows of `checks.tsv` whose note starts `not-run:` are such runs: listed so their ids exist, never run on refcore.
- **What step T still writes** (`tmp/session8/plan.md` §2), marked below as *T's, not yet in `checks.tsv`*: the goldens G1–G10 and the 36 pins restated in tally 2 (gate 2); the hindcast fixtures H0–H9 restated (gate 4); the codes' missing fixtures, one per refusal, hole and warning code of L §8 and E §11, their count taken by `tools/xref_check.py` (gate 5; K8-13); `PROPERTIES.md` with its generators (gate 6); and the generator of 10 000 random realities (gate 10).

**Programming analogy.** A tracer bullet, then a compiler built front end first. The tracer slice (S) runs one thin path through the whole engine on the amendments household, in production code. Then M0 parses and checks every text before anything runs, and each later milestone adds a part of the runtime, like adding features behind a test suite that already exists (TDD, where the tests were written first by someone else).

**Terms used once below.** A *crate* is an npm package. *clippy* is Rust's ESLint. A *fence* is a module that may hold optimised code, next to a plain twin that a test proves equal (D5-6), like keeping a readable reference implementation beside a hand-tuned one. A *worker* is a thread; the engine's threads only fill numbered slots, like `Promise.all` writing `results[i]`, and one loop merges them in index order (D3-1, D3-3).

---

## How a check is placed

Every one of the 634 checks is placed in exactly one milestone. The exact placement is `BUILD-ORDER-placement.tsv` beside this file (columns `id`, `milestone`, `why`); this document names checks by id or glob, and the appendix lists the globs a script checks against it (`placement_check.py`: every check placed once, every glob's checks in its milestone, the counts below). Placing a check needs only what it exercises, read from `checks.tsv` (`question`, `note`, `files`), `map/tags.tsv` (each question's edges) and the case lists:

1. **A check goes in the first milestone that has everything it exercises**, as 1.0.1's cases did.
2. **A unit probe** goes where its edge's construct is built (the edge table in the summary below). A pair card goes with the later of its two edges. A probe whose whole stored output is one `refused: LOAD-…` line goes to M0, since load alone gives it.
3. **A model check** (`I:M:`) keeps 1.0.1's case lists (§ Summary): a text scan of the 2.1 models finds none of them using a construct built later than its milestone (no parts, no joint model, fills only in M5a's cases, transcendentals only in M6's). LH-15 and LH-16 have no model check: their light twins stand for them (`I:light/`), and their full size is load row L:12. The two key scripts (`I:M:LH-11~key`, `I:M:CMP-01~key`) run no engine; they go with their case.
4. **A household check** goes to the later of the household's **floor** and its question's edges. The floor is what every walk of that household carries, whatever is asked: a place that keeps parts carries them in every walk (L §5.10; `engine/BUILD-NOTES.md`, the labels' cost), a joint model walks its members (L §6.1), a fill that the household's monthly tax reads, and the law parts beyond X's forms. Random facts and searches don't raise the floor: they are reached only by the asks that read them, which their question's edges name (RAND-n, RULE-09). The floors: X by its run (S, M1, M2 or M4), A M5b (its places keep parts), B M5a (its tax reads a fill), C and D M4 (their law parts), E1 and E2 M5c (joint models); a check of E1 or E2 that runs one household alone (E1-02's B side `I:E1:tx2_*`, E2's C side `I:E2:E2-01~e2c`) takes that household's floor.
5. **A `not-run:` check** is a load row's run: its output has no stored baseline, and the figures it holds are unjudged until the engine (`load.tsv`'s last column). It goes to M9, where every load row is run at its size and those figures are judged. Its row's wall time and footprint are measured earlier, at the exit of the milestone that completes its constructs (§ Summary, the P8-L column).

The scan behind rules 3 and 4 looks for six things in a check's text files, strings and comments blanked: a `fill`, a kind's `owned by` or `batched` line, a `use` line, an `fx` read, a transcendental call or `to u`, and an ask's `paths N`. It raised 21 checks above their edge's milestone (each says so in its `why`): three probes and three blind texts that write a fill, six of X's minimal texts that write a fill, two probes with a `use` line, and the seven blind-card checks of S1–S6, whose places keep parts.

---

## Summary

| M | Name | Builds | Checks | Models (`I:M:`) | DONE gates advanced | P8 rows at 1× |
|---|---|---|---|---|---|---|
| S | The tracer slice | the kernel, on X's base: evaluator, memo, event walk, checkpoints, numbers, booking, holes; counters, map, trace | 4 | — | 3 (X's base), 6 (the kernel's), 8, 10 | P8-14; P8-M1..P8-M4 set |
| M0 | Load and format | grammar with 2.1's clauses, canonical format, symbol table, types, load catalogue | 10 | none run; all load | 1 (all load), 5 (`LOAD-*`), 6 (format), 9 | — |
| M1 | The household forecast | X's forecast: rules, axes, pins, decisions, the written projections, scenarios, asks | 90 | 3 | 1, 2, 3, 5, 6 | P8-2, P8-5, P8-9 |
| M2 | Replay against reality | observations, booking, the residual ledger, the clocks; then forks | 26 | 7 | 1, 2, 3, 4, 6, 8 | P8-14 kept |
| M3 | The operations | E §1's run function; the rest waits for `OPERATIONS.md` | 0 | — | 5 (OP codes, later), 12 | P8-1, P8-7 (when rewritten) |
| M4 | Law breadth | `beats`, `given`, Kleene logic, rounding modes, FX, calendars | 77 | 34 | 1, 3 | P8-L2, P8-L10 |
| M5a | Allocation, holdings, many parties | `fill`, lots, holdings, ownership by shares, groups | 166 | 23 | 1, 8 (the fill) | P8-11; P8-L4, P8-L6, P8-L16 |
| M5b | Parts and labels | owned and batched kinds, arrivals, takes, relabels, R-115, hole parts | 79 | — | 5, 6 | P8-10; P8-L9, P8-L15, P8-L17 |
| M5c | Joint members as contexts | D1-7's two steps: a member's own context, then pruning | 40 | — | 6 | P8-12, P8-13, P8-15; P8-L8, P8-L18 |
| M6 | Exact functions and iteration | `to u`, `dcf`, exact distributions, scores, `solve` in law | 36 | 10 (+ LH-15, LH-16 by light twins) | 1 | — |
| M7 | Monte Carlo and parallel timelines | draws, paths, path statistics; the thread module | 21 | 0 (no model draws) | 4, 6, 7 | P8-4, P8-6, P8-9 (6 workers); P8-L1, P8-L3, P8-L5, P8-L11..P8-L14 |
| M8 | The evaluation layer | rolling origins, own-forecast scores, the fitting loop | 0 | — | 4 | P8-8 |
| M9 | Performance (step F) | every P8 row at size; seams on a measured miss; PGO | 85 | — | 12 | all, at size |

Counts: S=4 M0=10 M1=90 M2=26 M3=0 M4=77 M5a=166 M5b=79 M5c=40 M6=36 M7=21 M8=0 M9=85

**The arithmetic.** The cases: 3 + 7 + 34 + 23 + 12 = 79, as in 1.0.1, since no 2.1 model text uses parts or a joint model (M5a's 23 is 1.0.1's M5; M6's 12 counts LH-15 and LH-16 through their light twins). The checks:

```sh
python3 -c "print(4 + 10 + 90 + 26 + 0 + 77 + 166 + 79 + 40 + 36 + 21 + 0 + 85)"   # 634, every check once
python3 -c "print(3 + 7 + 34 + 23 + 10 + 2)"   # 79 I:M checks: 77 models and the 2 key scripts
```

**The edge table** (rule 2). An edge used by X's base or changes goes where X needs it; the edge's full breadth is what its probe checks.

| Milestone | Edges (`cases/boundaries.md`) |
|---|---|
| M1 | NUM-01..07; TIME-01..07 (and the TIME-08 probe); PTY-01; OWN-01, -02, -03, -08; FLOW-01, -02; VAL-01..04, -06, -07; LAW-01, -03, -05, -06, -08, -13; LIFE-01..05; RULE-01..05; REC-06; SCEN-01, -03, -04; ASK-01, -03, -04, -05; SURF-01, -02, -04, -05, -07 |
| M2 | REC-01..05, -07; VAL-05; LAW-04; RULE-06..08; SCEN-02; SURF-06 |
| M4 | CUR-01..03; LAW-02, -07, -10, -11, -12 |
| M5a | PTY-02..04; OWN-04..07, -09; FLOW-03; LAW-09; ASK-02 |
| M5b | OWN-10, -11, -12 |
| M5c | PTY-05 |
| M6 | RAND-02; LAW-20 |
| M7 | RAND-01, -03, -04; RULE-09; SURF-03 |

The cost budgets are checked from S on, not left to M9: each milestone's exit includes the P8 rows it touches, at 1× size for P8-10..P8-15 (cold, 1 worker), and at full size for the load rows whose constructs it completes. M9 is where the 10× sizes, the warm memo and the remaining parallel work are tuned. Every number is **proposed until S**: measured at S, then fixed with the user; a number found too tight changes only through a logged row (PERFORMANCE §8).

---

## S. The tracer slice (before M0)

A thin path through the whole engine, in production code, on the base of the amendments household X (its files in `integration/amendments/cut11/b_r1/`), so the riskiest parts, the time model and the cost of the walk, run first. Nothing in it is thrown away: M0 to M2 broaden it. X stays the tracer's household, as in 1.0.1: it is the one household whose base reads a written projection, decisions with declared reads, observations and law versions, yet keeps no parts, writes no fill and joins no member, so it is the thinnest path through every stage.

**Builds**
- A parser for the syntax X's base and its asks use, and no more. The full front end is M0.
- **The evaluator** (D1-1): a tree-walk over a resolved tree, one `match` per node kind (a `match` is a `switch` the compiler checks for missing cases). Recursion to a counted depth, then an explicit work stack; the depth is counted, never measured (K8-1, R-16).
- **The memo** (D1-2, D1-3; seam F): the engine's own, keyed by (item, key values, an interned and trimmed context); a recurrence keyed by day or period held as a dense column per (def, context); one memo per context, freed when its scope ends, never evicted (D1-6, D2-7). Read sets are recorded per cell now; the rest of PERFORMANCE §4 stays a seam (D1-5). The context's layer slots exist from S; X's base fills only the base layer and its projections' (D1-3).
- **The walk** (E §4.1, §4.2; D1-4): the plan pass, then a day walk driven by events, which steps only non-empty days; PERFORMANCE §2 derives what makes a day non-empty (SO8-1, open item O-1). The **step-every-day walk** is kept as the event walk's test oracle: equal output, more days. Monthly checkpoints are copied-on-write place states (D2-6); their interval is an unknown measured here (PERFORMANCE §10). Decisions with declared reads and the stratified order within a day, in X's forms (L §5.14.1, §5.14.2; D-5).
- **The written projection** in X's forms, `bal(x, d) from t`, `low` and `next` (L §5.14.1; E §4.3).
- **Numbers** (D2-1, D2-2; K8-2, K8-3): `Num` small (64-bit numerator and denominator) or big, with the whole-number shortcut and the Lehmer gcd written now; money a whole number of minor units, 64-bit or big; the 4 096-digit cap (`RUN-DIGITS`, L §7.1); `round` in X's forms (L §7.2).
- The read rule with its cut (L §4.5, §4.6).
- Booking at the cut, coverage, re-anchoring at X's statement rows and the residual ledger, in X's base forms (E §5, §6.1, §6.3; L §5.19).
- **Holes** with their questions (L §8.1), built in one module with one per-operator rule, the hole value a record with an empty slot for a later bound (seam U; R-116); the register (L §8.2) and a minimal `explain` (L §5.18; E §10).
- **The data layout:** one interner, `Symbol(u32)`, whose output order is by text (D2-3); an own `Table<I, T>` with `get` only, so indexing is impossible (D2-4); B-trees, sorted slices for frozen tables and dense vectors for day-keyed data, no hash map (D2-5, K8-6).
- **The counters** behind `--stats`, in shape.md's units, a cargo feature that is off in release (D4-8, K8-15).
- **The owner's kit** (D5-1, D5-4, D5-6): `engine/ARCHITECTURE.md` (about 200 lines), the `//!` header in every file (purpose, `Spec:`, `Twin:` the refcore file, `Invariants:`), the `trace` script from code to spec sections and back, and the fences, each with a `PERF:` header, a benchmark, a plain twin and an equivalence test: the walk, the memo, the read index and the number tiers.
- The release profile as the scaffold has it; the overflow checks' cost measured once (D4-4; above 3 % inside num-bigint, the user decides).

**Green**
- `I:X:konto_balance@b_r1`, `I:X:replay_september@b_r1`, `I:X:replay_september@b_r1s`, byte for byte; X's base asks against the hand-checked vectors through T's household vectors (gate 3, `smoke/HH4-*`).
- `R:all` stays green (the oracle's own tests; the engine runs nothing in it).
- The event walk equals the step-every-day walk on every S check.
- The kernel's properties: determinism, conservation, no foresight, a hole never becomes 0 (gate 6; T's `PROPERTIES.md` where written).
- The mutation score is at least 90 % on the kernel as S builds it: the numbers and rounding, the read rule, the walk and the written projection, booking and the residual ledger, the order within a day (gate 8).

**Measured at S** (and the budgets then fixed with the user, D4-6): P8-14, X's replay at 1× (refcore 52.6 s on the fast layer, 4.95 GB, ≥ 2.27 M def misses, partial), against its proposed ≤ 1 s and ≤ 512 MB; the micro rows P8-M1, P8-M2, P8-M3 and P8-M4, each set at the first measurement × 1.2 and then only ratcheted down by the lead; the unknowns of NOTE §4 that PERFORMANCE §10 assigns to S. This is the first figure from an engine, the evidence D1-1's closures, D1-4's forks and D1-5's seam wait for.

**Not in it:** the full grammar and the load catalogue (M0); X's changes, scenarios and the rest of its forecast (M1); records in full, back pay and the four clocks (M2); everything from M3 on.

---

## M0. Load and format

**Builds**
- The lexer, the grammar and the parser (L §2.1–§2.3), with 2.1's forms: the kind clause words and the parts clauses as kind lines, `of q` on a read, `note` lines after a one-line def and a place (2.1-1..2.1-3).
- The canonical formatter, one format with no options (L §2.4; 2.1-4: a one-line def with notes prints as the long form).
- Names, stable ids, reserved words. The symbol table is keyed by (path, sort, name), and the one-namespace rule is a load check over it (seam N; L §2.2, §2.5); the prelude's `parties` is one more built-in id.
- Types and units, the unit algebra, rates, price years (L §3.1–§3.4), including a day key where a date is expected (2.1-5).
- Spans, periods and indexes (L §4.1, §5.6); versions, `known` dates and the read rule's rows (L §4.5, §4.6, §5.3).
- Kinds: lifespans, subkinds, the prelude kinds (`parties` among them), and the parts clauses checked at load (`LOAD-PARTS`, L §5.7; 2.1-8, 2.1-20). Declared reads checked (`LOAD-READS`, L §5.14.1); axis combinations counted (`LOAD-FORKS`, L §5.5).
- The item graph, static cycles, file order by full relative path (E §3.1; R-40 as amended, K8-12), and the load catalogue (L §8.3).

**Green**
- The 10 load refusals: `U:law/LAW-13/rate25`, `U:rand/RAND-04/plan_b`, `U:rule/RULE-09/inpath`, `U:scen/SCEN-0*/{amend, kept, infl8, crash}`, `U:surf/SURF-04/redefine` (each a scenario file loaded alone, `LOAD-HEADER`), `U:rule/RULE-04` (`LOAD-CYCLE`) and `U:own/OWN-11/p21-signed` (`LOAD-PARTS`).
- Every text that `checks.tsv` names loads with no refusal but those its stored output shows, and `tools/loadcheck.py` (`--selftest` first) agrees with the loader over the 79 models, the household and the changes.
- `format(parse(x)) = x` for every one of them, and `format` is idempotent (gate 6).
- Each `LOAD-*` code has its fixture, giving exactly that code and message (gate 5; *T's, not yet in `checks.tsv`*, for the codes no probe prints).

**Properties:** the parser is total: no input crashes it, and every rejected input gets a code (fuzzed; gate 9, 8 hours per target at the end); nesting stops at L §8.3's depth (`LOAD-DEPTH`).

---

## M1. The household forecast

The deterministic core, enough for X's forecast and its changes 1, 2, 4, 6 and 7: a B2B earner, an employee, Polish law, a mortgage, savings.

**Builds**
- Exact numbers in full: the literal rule, `round` with every mode X's law uses (L §7.1, §7.2). M4 adds the rest.
- The calendar built-ins, holiday inputs, the law date and its propagation, windows and `windows each` (L §4.2–§4.4).
- `index`, `input`, `def` and `rule` with `when` and `beats` in X's forms, recurrences, `band`, `axis` with its 64-combination cap (L §5.5, §5.6, §5.8, §5.9, §5.11).
- `prorata` with its three rests; its `rest largest`, the largest-remainder split, is written once here and is every later split (L §5.13; R-114).
- Pins: `with` and `as if`, the law and known pins (L §5.11), as **layers** of the memo's context; axis combinations and scenarios too (D1-3).
- Kinds and things with lifespans; places and their kinds, the prelude's place kinds (L §3.6, §5.7, §5.10); parties as things under the prelude kind `parties`, the subkinds the model's own (L §3.5; 2.1-6).
- Moves: plans in dependency order over the whole run, decisions in the day walk with declared reads, **keyed decisions** (a keyed move that declares `reads`), ties by the move's `ties by`, written orders (`before`, `after`), the stratified order within a day (L §5.14, §5.14.1, §5.14.2; E §4.1, §4.2).
- **The written projections** in full: `bal(x, d) from t`, `low`, `next` and their clauses (`windows each`, `with uncertain`), `value`, `flows` (L §5.10, §5.14.1; E §4.3). They replace 1.0.1's `plan_low`, `next_on` and `worth`.
- Scenarios as overlays of ordinary versions, compared by keyed difference (L §6.4), and amendments as added files (L §6.1–§6.3).
- Asks: values, checks, partial answers, differences, `solve` in an ask, `explain` with provenance on every movement (L §5.18; E §10); the register and warnings (L §8.2, §8.5).
- The run invariants (E §9) and the canonical projection of a result.

**Green (90)**
- X: `I:X:*@c1_fast`, `@c2_fast`, `@c4_fast`, `@c6_fast`, `@c7_fast`, `@v4_fast` (18 checks), and X's minimal texts without a fill (`I:X:t@min_m13`, `…_m13b`, `…_F4g`, `…_F4h`, `…_m10l`, `I:X:n@min_m2`, `n2@min_m2`, and the same two at `min_m2b`: 9).
- Cases: `I:M:UL-01`, `I:M:UL-02`, `I:M:UL-05`.
- Probes (60): `U:num/*`, `U:time/*`, `U:life/*`; ASK-01, -03, -04, -05; FLOW-01, -02; LAW-01, -03, -05, -06, -08, LAW-13 and its set; OWN-01, -02, -03; PTY-01; REC-06; RULE-01, -02, -05; SCEN-01, -03, -04 and their sets; SURF-01, -02, -04 and its set, -05, -07; VAL-01..04, -06, -07.
- *T's, not yet in `checks.tsv`:* goldens G1, G2, G3, G6, G7, G8, G9 and G10 and the 36 pins, on tally 2's canonical form (gate 2); the hole and run codes of M1's constructs, one fixture each (gate 5).

**Properties** (gate 6): determinism; money conserved (E §9.1); net worth moves only at the boundary, a price or an anchor (§9.2); a scenario changes nothing before its `from` (§9.3); asks move no money (§9.4); things hold nothing outside their lifespans (§9.6); no foresight through declared reads and the written projection (D-5, D-6); a hole never becomes 0.

**Budgets touched:** P8-5, one read of a written projection (its micro form P8-M4 set at S); P8-2's cold run at 1× on X's forecast (the 10× run is M9's, its size from T's generator, SO8-10); P8-9, equal hashes across runs with 1 worker.

---

## M2. Replay against reality

**Builds**
- Observations in full (L §5.19): observed inputs, readings at instants, statements with their anchors, pads and gaps, `same` between observed rows, and the plans an observation fulfils.
- Booking at the cut per class, the covered past, back pay and adjustments as ordinary movements (E §5; L §5.14.3). L §5.14.3's one exception for a past decision that reads parts is M5b's.
- The modelled past, computed and never booked (E §6.2); the residual ledger in full, its lines and their questions, and `residuals` asks (E §6.3).
- The four clocks S, K, (K, E] and R, and the dates inside a run (E §2); an ask's known date.
- Prices and figures read from records (L §5.8, §5.19).

**Then forks** (D1-4's second step), M2's last step: timelines forked at their first different day, once the walk matches refcore on every check from S to M2. It is a fence (D5-6) whose plain twin is the unforked walk, and the equivalence test runs both on every later check. Why here: from M2 on, the timelines that share a start multiply (X's changes and scenarios, axis combinations, `windows each`, pin sets, and later A's 64 repetitions behind P8-10), and each later milestone's budgets read them. Parts (M5b) and members (M5c) add place state, which the copied-on-write checkpoints carry (D2-6); the fork itself doesn't change.

**Green (26)**
- X: `I:X:*@c3_fast` and `I:X:*@v3_fast`, change 3, the retroactive law (6).
- Cases: `I:M:LH-05`, `LH-14`, `RC-08`, `UL-06`, `UL-09`, `UL-15`, `CH-04`.
- Probes (13): LAW-04; REC-01, -02, -03, -05, -07; RULE-06, -07, -08; SCEN-02 and its set; SURF-06; VAL-05.
- *T's, not yet in `checks.tsv`:* goldens G4 and G5 (G5 with the fixture's flat price index pinned); hindcast fixtures H1, H2, H3, H4, H5, H8 and H9, each landing in the ledger line its answer key names (gate 4). H0, H6 and H7 are banded and wait for M7.

**Properties**
- **Masking at K:** changing any record or version known after K leaves every result as of K unchanged (H8 is the metamorphic case).
- **The three-way split:** replay minus actual is 0 on every fixture but H3.
- **Fixed past:** a booked plan amount never changes; a later version adds an adjustment (E §5, §9.5).
- **Forks:** every check gives the same bytes with forks on and off.

**Budgets touched:** P8-14 kept at its fixed number; with forks, the repeated walks of X's changes are re-measured.

---

## M3. The operations

This is the surface Claude uses, built without fixing any MCP technology. **`OPERATIONS.md` stays stale** (`STALE.md`): it is tally 1.0.1's, and no session has rewritten it for tally 2. M3 therefore splits.

**What M3 builds from E alone**
- The run function `run(reality, request) → result` as the library's one entry and the CLI's (E §1): the request (asks, clocks, pins), the result (timelines, answers with their registers and warnings, residuals, the hash and the version).
- Purity: the run reads its files and its request and nothing else, no clock, environment, locale or network (E §1; L §7.5).
- The result stored as aggregates under its hash; a path never stored, recomputed on demand (E §1, §7).
- A reality as a set of files, and a change as added files (L §6.1–§6.3), already exercised by X's changes in M1, M2 and M4.

**What waits for the rewrite of `OPERATIONS.md`**
- The store and its statuses, authors and record time; the amendment pipeline and its typed results; the change kinds as operations; scenario life.
- `plan` and its outcome diff; saved analyses.
- The gate on a law part, with its steps, planted errors and thresholds.
- The rest of the surface, the OP codes and their fixtures (gate 5: they join when `OPERATIONS.md` is rewritten), and the reference card.
- The budgets that lean on them: P8-1 (a conversation turn, 1–2 s) and P8-7 (the gate on one law part, ≤ 1 s).

No operation's meaning is written here. **Order:** M3's first part is tagged after M2. Its second part is planned when `OPERATIONS.md` is rewritten (open: O-W2-1). M4 to M8 don't wait for it: none of their checks runs an operation, since every engine check in `checks.tsv` is a run of the run function.

**Green:** no check of `checks.tsv`. The seven changes as amendments through `plan`, and the gate's fixture, come with the rewrite.

---

## M4. Law breadth

The constructs that tax and benefit law needs beyond X's: exceptions, branches, eligibility, other countries' calendars and money.

**Builds**
- `beats` chains and `given` beyond X's forms; Kleene `and` and `or`, the aggregates, `argmin` and `argmax` (L §5.11).
- Every rounding mode and step, and the rounding axis (L §7.2).
- Several currencies: rates as `num A/B`, `flows … in CUR at fx mode`, an exchange's two legs (L §3.2, §5.10, §5.14).
- Calendars per country (L §4.2).
- The household law parts that C and D run on (GEN, PIT, SOC, IGT, CC, CIV and the rest), each in full.

**Green (77)**
- X: `I:X:*@c5_fast`, `I:X:*@v5_fast`, change 5, a job abroad with euro places (6).
- Cases (34): CB-01, -02, -03, -04, -06, -07, -08, -11, -12; LH-07, -08, -12, -13; MP-05, -06, -07, -08, -10, -11, -13, -16; RC-02, -04, -05, -07; UL-03, -04, -10, -11, -12, -14; CH-05; and MP-14 and CMP-02, whose lot is stipulated, so they don't draw.
- Household C (6): `I:C:C-01~c01`, `C-02~c02`, `C-04~c04`, `C-06~c06`, `C-08~c08`, and E2's C side `I:E2:E2-01~e2c`.
- Household D (23): `I:D:d04_*`, `I:D:d09_*`, `I:D:d11_*`, `I:D:d12_*`.
- Probes (8): `U:cur/*`; LAW-02, -07, -10, -11, -12.

**Budgets touched:** P8-L2 (D-11's pair, its constructs complete here) and P8-L10 (X's long-horizon walk cells, change 1's and change 5's walk runs), each at full size at M4's exit.

---

## M5. Allocation, parts and many parties

Three named steps, each tagged as a milestone (M5a, M5b, M5c) when its checks are green: allocation first, since a place's takes reuse the fill (L §5.12; 2.1-10); then parts, since household A keeps them in every walk; then joint members, since E1 and E2 join A and its parts.

### M5a. Allocation, holdings and many parties

**Builds**
- `fill`: greedy allocation in a stated order, its supplies, demands and arcs, and its refusals (L §5.12); lots matched by a fill; carry-forwards and carrybacks by a fill over years.
- Holdings in units, priced, beyond X's; claims and debts beyond X's forms (L §5.10).
- Ownership by shares, `owns[p, x]` with dated rows, and net worth per person over it (L §3.6; ASK-02).
- Groups and relations with dated spans; a law that reads another party (L §3.5).
- Long histories as input rows.

**Green (166)**
- Household B (55): every B check that runs, but B-12's (M7): `I:B:b01_*` to `b11x_*` on their cut copies, `I:B:*@k4` (B-07's search at 101 steps, J marks), `I:B:B-13~*`.
- Household D (43): `I:D:d01_*`, `d02_*`, `d03_*`, `d05_*` (with the known fault KF-01: `I:D:d05_quarters` guarded, `I:D:d05_quarters~alone` reading `known` until fixed), `d06_*`, `d07_*`, `d08_*`.
- Household C (6): C-03, C-05, C-07, C-09, C-10 and C-11 (`I:C:C-0n~c0n`).
- E1-02's B side (6): `I:E1:tx2_*`, B alone.
- The blind texts (11): `I:hardset/*`.
- Cases (23) and a key script: CB-05, -09, -10, -13, -14; LH-01, -02, -03, -04, -06, -09; MP-01, -02, -03, -04, -09, -12, -15 (MP-15 has no declared bound in tally 2: its step is an ordinary hole with its reason, D-15); RC-01, -03, -06; UL-13; CMP-01 and `I:M:CMP-01~key`.
- X (7): the minimal texts that write a fill (`I:X:t@min_F3w`, `…_F3w3`, `…_F3k`, `…_F7a`, `…_m12d`, `…_m12e`) and card I-10's `I:X:q1` (shares that change, a shared ledger).
- Probes (14): ASK-02; FLOW-03; LAW-09; OWN-04, -05, -06, -07, -09; PTY-02, -03, -04; and OWN-08, REC-04 and RULE-03, whose texts write a fill.

**Budgets touched:** P8-11 (`I:E1:tx2_mtg_day`, refcore 60.5 s, 2.17 GB, 619 k def misses; E1's largest number, 5 049 bits, is P8-M3's size); P8-L4 (B-07 at 4 001 steps), P8-L6 (B's balance rows past 2026-12-31), P8-L16 (B-05 known 2026-01-05), at full size. Gate 8's kernel list gains the fill.

### M5b. Parts and labels

**Builds** (L §3.6, §5.7, §5.10, §5.12–§5.14.3, §5.19, §8.1; E §4.2, §6.3; the rows 2.1-2 to 2.1-23 of `CHANGES.md`)
- Kinds that keep parts: `owned by D [ties by e]`, `batched`, `step u`, `takes … order … [prorata]` (L §5.7). A place's parts are walk state beside its balance, per unit a map from a label to an amount, paid only where a kind opts in: a place of any other kind keeps one number per unit and pays nothing (`engine/BUILD-NOTES.md`, the labels' cost; PERFORMANCE §7).
- The **arrival rule**, a reader of start-of-day parts beside decisions, with its law at its movement's day (L §3.6, §5.14.1); the **split**, L §5.13's `rest largest` at the kind's step, the one function written at M1 (R-114), ties by `ties by`, else `RUN-TIE`.
- The **leaving rule** (`takes`) as a fill over the place's parts, reusing M5a's fill (L §5.12, §5.14): ties, two outflows never sharing, the batch's arrivals as supplies, undecided takes; takes in the day's first batch in the order of their reads, and a cycle broken where L §5.14.2 says, else `RUN-CYCLE`.
- **Relabels** and the leg read `m.leg.amount of q` (L §5.14).
- Reads `bal(x, d) of q` and `value(x, d) of q`, and net worth over `of` reads (L §3.6, §5.10); `parties` as the only owner of parts (L §3.5).
- Labels as walk state at the cut K, with L §5.14.3's one exception for a past decision that reads parts (E §5).
- Readings at a place that keeps parts: stated and opening parts, pads, zero-gap anchors, gaps, and the ledger's line per stated part (L §5.19; E §6.3).
- **Hole parts** (`HOLE-PART`, L §8.1) in seam U's one module (R-116), and `WARN-LABELS` in the register (L §8.2, §8.5); `RUN-LABEL` and the run codes widened for parts (L §8.4).
- The parts in the day walk, in E §4.2's order: the first batch's takes, a decision's take at its point, the arrival rule at booking, lifespans after the day's whole order.
- **The parts check** (R-115): after every arrival, take and relabel, parts sum to the balance, are at least 0 and whole steps; a failure refuses as an invariant. It is checked at run time, at O(labels) per booking (G1 item 8). E §9 doesn't list it (SO8-7), so it is the engine's own check, not a DONE gate 6 property.

**Green (79)**
- Household A (46): every A check that runs (`I:A:*` but the 3 `not-run:` ones), A-13's, A-14's and A-15's included.
- Session 7's blind cards (7): `I:S?:*`, S4's pinned twin `I:S4:S4-03P~pinned` among them.
- Probes (26): `U:own/OWN-10/*` (13); OWN-11's but `p21-signed` (9); OWN-12's `p10`, `p17/ania`, `p17/halina`, `p23`.
- *T's, not yet in `checks.tsv`:* the fixtures of `RUN-LABEL`, `HOLE-PART`, `WARN-LABELS` and the run codes 2.1 widened, where no probe prints them (gate 5).

**Budgets touched:** P8-10 (`I:A:a05_ania@c05-ania`, A-05's `windows each`: refcore 37.7 s, 0.81 GB, 150 k def misses); P8-L9 (A-05 on the full text), P8-L15 (A's q12 asks), P8-L17 (A-13 (b) with `windows each`), at full size.

### M5c. Joint members as contexts

**Builds**
- Joint models: `use` lines importing closed models pinned by the sha256 of their canonical text (E §3.1 step 1), `same` lines, a member's items read as `a.item`, `parties` shared by every member, `of` across a `same` (L §5.1, §6.1).
- **D1-7's first step (seam J):** each member is its own evaluation context, keyed by its pin, with its own memo, a layer of the memo's context (D1-3); its day walk runs lazily, up to the last day a read reaches; with parts, its walk state includes them. A member that is itself a joint model nests by the same structure, walked as a worklist, since nesting has no limit and no code (R-16 as amended; SO8-8).
- **D1-7's second step**, M5c's last: a member's walk pruned to the places the reads reach, per (member, place set, last day) (`engine/BUILD-NOTES.md` § J). A fence whose plain twin is the first step's walk.

**Green (40)**
- E1 on its joint cut (23): `I:E1:e1_*`.
- E2 as a joint model of A and D (5): `I:E2:E2-01~e2_01a`, `~e2_01b`, `~e2_01c`, `I:E2:E2-02~e2_02a`, `~e2_02b`.
- Probes (12): `U:pty/PTY-05/*` and its set; `U:pairs/PTY-05+OWN-12/*` and its set; `U:own/OWN-12/p17/joint` and `U:set/own/OWN-12/p17/ania`, whose texts join members.

**Budgets touched:** P8-12 and P8-13 (E2-01's two asks: refcore 49.1 s and 65.8 s), P8-15 (`U:set/pairs/PTY-05+OWN-12/ania`, a nested joint model: 0.87 s, 74 def misses), at 1×; P8-L8 (E1's full joint walks) and P8-L18 (one row read from a member with a long walk: the cone's cost, decision 6), at full size.

---

## M6. Exact functions and iteration

**Builds**
- Transcendentals `to u` by interval refinement, and the `≈` bound when no `round` follows (L §7.3).
- `dcf` with its day counts, and `factor` and `periodic` beyond X's forms, with a non-integer power (L §3.3).
- Exact distribution functions without paths: CDFs, quantiles of a distribution, the exact binomial (L §5.16).
- Scores: `crps`, interval scores, skill, `orient` (L §5.15).
- A named iteration, and `solve` in a def where the law names the method (L §5.11; LAW-20).

**Green (36):** cases UL-07, UL-08, LH-10, LH-11 (and `I:M:LH-11~key`), CH-01, CH-02, CH-03, LH-17, LH-18, CH-06; LH-15 and LH-16 through their light twins `I:light/LH-15-light` and `I:light/LH-16-light~*` (22); probes RAND-02 and `U:law/LAW-20~L1`.

**Properties:** every transcendental's printed digits are correct, checked against a second method at a higher precision; the refinement stops or refuses, never prints a wrong digit.

---

## M7. Monte Carlo and parallel timelines

**Builds**
- Random facts (a distribution is a fact's value), draws by address, the canonical draw order, every distribution (L §5.16, §7.4).
- Paths, the shared layer and the per-path layer (E §7); paths as a layer of the memo's context, each path's memory freed when it ends (D1-3, D1-6).
- Path statistics with their counts, standard errors and horizon, `quantile`, `share`, paired paths (L §5.16); a quantile's standard error by intervals refined to the printed 1e-6 rounding, refcore's exact method the oracle at small N (D3-7); a path statistic's accumulator its own type, outside the cap check (2.1-15; R-117; K8-16, K8-17).
- One path replayed for an explanation (E §7, §10).
- **The thread module** (D3-1..D3-7), the only place threads, channels and locks live: std's scoped threads, each spawned with a stack size passed in (R-42 as amended); workers fill numbered slots and one thread merges them in task-index order, the 1-worker run through the same module; the merge rule (per line, terms in order; the first refusing path wins, else the union of holes; quantiles sorted stably by value, then path; SO8-3); the shared layer run alone first, then frozen, with a counter of locally recomputed cells (D3-4); the worker count an argument, the CLI's default 6, never read from the machine (D3-5). A fence, its plain twin the 1-worker run.
- **In parallel through it:** Monte Carlo paths, and the independent timelines that exist since M1 (axis combinations, `windows each`, scenarios) and from M8 (rolling origins), as D3-2 puts them in the first set.

**Why the thread module lands here.** Every budget before M7 is single-worker: P8-10..P8-15 are cold runs on 1 worker, and P8-4 is the first row that names workers. Gate 6's worker sweep and the merge rule are about paths (which refusal prints, the union of holes, a quantile's order), so the module is tested where its hardest case first exists. And by M7 every single-threaded output from S to M6 is fixed by its check, so a difference between worker counts can only be the module's. Parallel joint members and parallel asks come later (D3-2), at M9, and only for a row that misses.

**Green (21)**
- B-12's split runs (4): `I:B:o3_jan@k5`, `o3_feb@k5`, `b12_zero_tax@k5`, `b12_tax@k5` (J marks).
- D-10 and D-13 (6): `I:D:d10_*`, `I:D:D-13~D1`, `~D2`; C-12 (`I:C:C-12~c12`, its edge RULE-09).
- `I:light/LH-17-mc-light`.
- Probes (9): RAND-01, RAND-03, RAND-04 and its set, RULE-09 and its set, SURF-03, and the pair `U:pairs/CUR-01+RAND-01~P1`, `~P2`.
- *T's, not yet in `checks.tsv`:* hindcast H0, H6 and H7 on a single origin; the Monte Carlo keys of `montecarlo/` (LH-17 against its exact closed form, the household's bands) run at full size as load rows L:11 and L:12 (gate 7).

**Properties** (gate 6, D3-6): common random numbers (two scenarios share every draw whose address they share); equal hashes across the worker sweep {1, 2, 3, 5, 6, 7}; a test-only reversed completion order; planted violations; a deep-stack case; no foresight on paths.

**Budgets touched:** P8-4 (1 000 paths × 30 years, 6 workers) at 1×; P8-6 (one path replayed, ≤ 20 ms); P8-9 with 1 and 6 workers; at full size, P8-L1 (D-10 at 10 000 paths), P8-L3 (B's full span), P8-L5 (B-12 at 3 000 paths), P8-L11 (household-bands at 1 000 paths), P8-L12 (LH-15, LH-16, LH-17-mc at full size), P8-L13 and P8-L14 (X's and session 4's runs past 8 GB).

---

## M8. The evaluation layer

**Builds**
- Rolling origins, `as if {known origin_day[o]}` over an index (E §8), as independent timelines through M7's module.
- Scores of the model's own forecasts against records, refused where invalid (E §8).
- The fitting loop's cycle: pin, replay, residuals (E §6.4).

**Green:** no check of `checks.tsv`. *T's, not yet in `checks.tsv`:* hindcast H0–H9 with pooled origins at months 24, 36 and 48, each banded verdict stating its power (gate 4); the refusals of `hindcast.md` §4, each where it lives (the judge script's, not the engine's).

**Budgets touched:** P8-8, one fitting step over two years of history.

---

## M9. Performance (step F)

Every row of PERFORMANCE §8 at the size it states, on the reference machine (DONE gate 12; D4-3): P8-2 and P8-3 at 10× (their sizes from T's generator, SO8-10), every load row P8-L1..P8-L18 at full size, nightly, and the 85 `not-run:` checks of `checks.tsv` run as their rows: `I:A:*@q05-*` and `A-13~A5` (A), B-05, B-07, B-12 and K3's retired runs (B), `I:light/LH-16-light`, and X's c1_r2, mc, @k10 and @k11 runs.

The seams built only on a measured miss come in here, each when its row misses, never before:
- PERFORMANCE §4's verification and early cut-off, when a measured turn misses 1–2 s or the cold 33-year run misses 1 s at 10× (D1-5);
- closures in place of the tree walk, for an ask whose node count misses its budget (D1-1);
- parallel joint members and parallel asks (D3-2);
- a hash map, only if the first engine profile shows sparse memo lookups dominating (K8-6).

PGO only here, and no budget may need it (D4-4). Every optimisation leaves every output byte-identical: the gate 1–7 hashes before and after. PERFORMANCE §10 lists what this step settles.

---

## What T still writes, and where it lands

*T's, not yet in `checks.tsv`*, each placed where 1.0.1 placed its tally 1 form (`tmp/session8/plan.md` §2):

| T's item | DONE gate | Lands at |
|---|---|---|
| Goldens G1, G2, G3, G6–G10 and the 36 pins, restated in tally 2 with refcore baselines | 2 | M1 |
| Goldens G4, G5 | 2 | M2 |
| Hindcast H1–H5, H8, H9 | 4 | M2 |
| Hindcast H0, H6, H7 on a single origin | 4 | M7 |
| Hindcast H0–H9 with pooled origins | 4 | M8 |
| The codes' missing fixtures: a scan of which codes the baselines print against L §8 and E §11, their count by `tools/xref_check.py` (K8-13), and a fixture for each code no check prints; `ENGINE-INVARIANT` through the fault-injection hook | 5 | each in the milestone that builds its construct: `LOAD-*` at M0, the parts codes at M5b, the rest with their construct; OPERATIONS §10's when it is rewritten |
| `PROPERTIES.md` with its generators | 6 | the kernel's at S, the parser's and the format's at M0, M1's and M2's lists above, the worker sweep and reversed completion at M7 (D3-6) |
| The generator of 10 000 random realities | 10 | the full differential run is gate 10's, after M9; whether earlier exits can run it over the constructs built so far is T's call (open: O-W2-3) |
| `TESTS.sha256`, the guard over the tests | rule 2 | before S |

T's sizes for P8-1..P8-8 in 2.1's models also come from the generator (SO8-10).

---

## What every milestone keeps

- **Every earlier check stays green**, byte for byte against refcore's stored output (gate 10 on the checks), and `R:all` stays green.
- **Regression tests from the prototype's bugs** (C-16): no rounding helper that loses a cent; FX only between different currencies, by the unit types.
- **The code catalogue:** each code of L §8 and E §11 gets its fixture in the milestone that builds its construct.
- **No silent default:** a test per construct that a missing row gives a hole or a refusal, never 0 or a guess (*Tally explicit*).
- **The bounds:** `BOUNDS.md` rows are re-marked when their milestone's checks pass, and only then.
- **The budgets** (D4-2): the counters' counts and Valgrind's instruction counts, with hard limits, on every PR on Linux; wall time and peak footprint on the reference Mac (D4-3) at the milestone's tag, run by the gate session, for the P8 rows the milestone touches. A number is proposed until S; after S, a miss is fixed in the code, or changed only through a logged row.
- **The memory ceiling:** 8 GB on any run, 4 GB per load row, 512 MB per 1× check (D4-7); at most 6 heavy jobs at once on the machine.
- **The mutation score** of gate 8 on the kernel as it stands, at least 90 %, every surviving mutant listed with its reason.
- **The milestone report** (D5-5): one generated page, modules added, the spec trace's coverage delta, exceptions, size and benchmark deltas, for the owner; and `engine/ARCHITECTURE.md` updated (D5-1).
- **The fences** (D5-6): no optimised code outside a fence; each fence keeps its plain twin and its equivalence test green.

---

## Appendix: the placement by glob

`BUILD-ORDER-placement.tsv` is the exact placement, one row per check. The globs below name 569 of the 634 checks; the rest, mostly unit probes in axes that span milestones, are named in their milestone's section and placed by the TSV. `placement_check.py` expands each glob against `checks.tsv` (`*` any run of characters, `?` one, `[…]` a set, `[!…]` its complement) and fails if a glob's checks aren't all in its milestone or its count differs, if any check is placed zero times or twice, or if the `Counts:` line above disagrees with the TSV.

| M | Glob | Checks |
|---|---|---|
| S | `R:all` | 1 |
| S | `I:X:*@b_r1` | 2 |
| S | `I:X:*@b_r1s` | 1 |
| M0 | `U:law/LAW-13/rate25` | 1 |
| M0 | `U:rand/RAND-04/plan_b` | 1 |
| M0 | `U:rule/RULE-09/inpath` | 1 |
| M0 | `U:scen/SCEN-01/amend` | 1 |
| M0 | `U:scen/SCEN-02/kept` | 1 |
| M0 | `U:scen/SCEN-03/infl8` | 1 |
| M0 | `U:scen/SCEN-04/crash` | 1 |
| M0 | `U:surf/SURF-04/redefine` | 1 |
| M0 | `U:rule/RULE-04` | 1 |
| M0 | `U:own/OWN-11/p21-signed` | 1 |
| M1 | `U:num/*` | 7 |
| M1 | `U:time/*` | 9 |
| M1 | `U:life/*` | 5 |
| M1 | `I:X:*@c[12467]_fast` | 15 |
| M1 | `I:X:*@v4_fast` | 3 |
| M1 | `I:X:n*@min_m2*` | 4 |
| M1 | `I:X:t@min_m13*` | 2 |
| M1 | `I:X:t@min_F4?` | 2 |
| M1 | `I:X:t@min_m10l` | 1 |
| M1 | `I:M:UL-0[125]` | 3 |
| M2 | `I:X:*@[cv]3_fast` | 6 |
| M2 | `U:rec/REC-0[12357]` | 5 |
| M2 | `U:rule/RULE-0[678]` | 3 |
| M2 | `I:M:CH-04` | 1 |
| M2 | `I:M:LH-05` | 1 |
| M2 | `I:M:LH-14` | 1 |
| M2 | `I:M:RC-08` | 1 |
| M2 | `I:M:UL-06` | 1 |
| M2 | `I:M:UL-09` | 1 |
| M2 | `I:M:UL-15` | 1 |
| M4 | `U:cur/*` | 3 |
| M4 | `I:X:*@[cv]5_fast` | 6 |
| M4 | `I:C:C-0[12468]~*` | 5 |
| M4 | `I:E2:E2-01~e2c` | 1 |
| M4 | `I:D:d04_*` | 8 |
| M4 | `I:D:d09_*` | 6 |
| M4 | `I:D:d11_*` | 7 |
| M4 | `I:D:d12_*` | 2 |
| M4 | `I:M:CB-0[1-46-8]` | 7 |
| M4 | `I:M:CB-1[12]` | 2 |
| M4 | `I:M:LH-0[78]` | 2 |
| M4 | `I:M:LH-1[23]` | 2 |
| M4 | `I:M:MP-0[5-8]` | 4 |
| M4 | `I:M:MP-1[01346]` | 5 |
| M4 | `I:M:RC-0[2457]` | 4 |
| M4 | `I:M:UL-0[34]` | 2 |
| M4 | `I:M:UL-1[0124]` | 4 |
| M4 | `I:M:CH-05` | 1 |
| M4 | `I:M:CMP-02` | 1 |
| M5a | `I:hardset/*` | 11 |
| M5a | `I:E1:tx2_*` | 6 |
| M5a | `I:B:*@k4` | 6 |
| M5a | `I:B:B-13~*` | 5 |
| M5a | `I:B:b0[1246]_*` | 17 |
| M5a | `I:B:b03_*[!3]` | 4 |
| M5a | `I:B:b08_*[!3]` | 7 |
| M5a | `I:B:b09_*[!3]` | 4 |
| M5a | `I:B:b1[01]*[!3]` | 7 |
| M5a | `I:B:b10_v3` | 1 |
| M5a | `I:B:b05_*_k07` | 3 |
| M5a | `I:B:b05_last` | 1 |
| M5a | `I:C:C-0[3579]~*` | 4 |
| M5a | `I:C:C-1[01]~*` | 2 |
| M5a | `I:D:d0[123678]_*` | 38 |
| M5a | `I:D:d05_*` | 5 |
| M5a | `I:M:CMP-01*` | 2 |
| M5a | `I:X:t@min_F3*` | 3 |
| M5a | `I:X:t@min_F7a` | 1 |
| M5a | `I:X:t@min_m12?` | 2 |
| M5a | `I:X:q1` | 1 |
| M5a | `U:own/OWN-0[4-79]` | 5 |
| M5a | `U:pty/PTY-0[234]` | 3 |
| M5a | `I:M:CB-0[59]` | 2 |
| M5a | `I:M:CB-1[034]` | 3 |
| M5a | `I:M:LH-0[123469]` | 6 |
| M5a | `I:M:MP-0[1-49]` | 5 |
| M5a | `I:M:MP-1[25]` | 2 |
| M5a | `I:M:RC-0[136]` | 3 |
| M5a | `I:M:UL-13` | 1 |
| M5b | `I:A:a0[1-46-9]*` | 18 |
| M5b | `I:A:mca*` | 6 |
| M5b | `I:A:a1*` | 11 |
| M5b | `I:A:A-1[45]~*` | 2 |
| M5b | `I:A:A-13~A[1-4]` | 4 |
| M5b | `I:A:a05_*@c05-*` | 2 |
| M5b | `I:A:a05_tomek*` | 2 |
| M5b | `I:A:a05_wrapped` | 1 |
| M5b | `I:S?:*` | 7 |
| M5b | `U:own/OWN-10/*` | 13 |
| M5b | `U:own/OWN-11/p[0-1]*` | 5 |
| M5b | `U:own/OWN-11/p2[02]*` | 3 |
| M5b | `U:own/OWN-11/p21-below` | 1 |
| M5c | `I:E1:e1_*` | 23 |
| M5c | `I:E2:E2-0?~e2_0*` | 5 |
| M5c | `U:pty/PTY-05/*` | 3 |
| M5c | `U:pairs/PTY-05+OWN-12/*` | 5 |
| M5c | `U:set/pty/*` | 1 |
| M5c | `U:set/pairs/*` | 1 |
| M5c | `U:own/OWN-12/p17/joint` | 1 |
| M5c | `U:set/own/*` | 1 |
| M6 | `I:light/LH-16-light~*` | 22 |
| M6 | `I:light/LH-15-light` | 1 |
| M6 | `I:M:LH-11*` | 2 |
| M6 | `U:law/LAW-20~L1` | 1 |
| M6 | `U:rand/RAND-02` | 1 |
| M6 | `I:M:UL-0[78]` | 2 |
| M6 | `I:M:LH-10` | 1 |
| M6 | `I:M:LH-1[78]` | 2 |
| M6 | `I:M:CH-0[1236]` | 4 |
| M7 | `I:D:d10_*` | 4 |
| M7 | `I:D:D-13~*` | 2 |
| M7 | `I:B:o3_*@k5` | 2 |
| M7 | `I:B:b12_*tax@k5` | 2 |
| M7 | `I:light/LH-17-mc-light` | 1 |
| M7 | `U:rand/RAND-0[13]` | 2 |
| M7 | `U:pairs/CUR-01+RAND-01~*` | 2 |
| M7 | `I:C:C-12~*` | 1 |
| M9 | `I:B:*@k3` | 27 |
| M9 | `I:B:*@cd7full*` | 10 |
| M9 | `I:B:b12_*[!5]` | 9 |
| M9 | `I:B:b12_[lms]*@k5` | 7 |
| M9 | `I:B:o3_[fj]*[bn]` | 2 |
| M9 | `I:B:b05_[abi]*[sel]` | 2 |
| M9 | `I:B:b07_wrzos_seen` | 1 |
| M9 | `I:X:*@k1?` | 11 |
| M9 | `I:X:*@c1_r2` | 2 |
| M9 | `I:X:check_tm_*[!1]` | 4 |
| M9 | `I:X:base_values` | 1 |
| M9 | `I:X:explain_room` | 1 |
| M9 | `I:X:ike_left` | 1 |
| M9 | `I:X:net_worth` | 1 |
| M9 | `I:X:taxes_paid` | 1 |
| M9 | `I:light/LH-16-light` | 1 |
| M9 | `I:A:*@q05-*` | 2 |
| M9 | `I:A:A-13~A5` | 1 |
