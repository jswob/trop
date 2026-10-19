# Tally: performance

This document covers:
- what each construct costs;
- how the engine evaluates;
- the budgets it must meet;
- how a change costs only what it touches.

It names no technology. These are design estimates, scaled from the prototypes' measurements (spec §7). They are not measurements. The build measures them and tunes (step F of the plan). Each estimate shows its arithmetic.

**Contents**
1. Units, and the evaluation strategy
2. The run: flow pass, day walk and state reads
3. Cut classes, scenarios, pins and rolling origins
4. A change costs what it touches: invalidation, the walk restart and the fitting loop
5. Monte Carlo
6. Numbers
7. Cost per construct
8. Budgets
9. What can grow, and what caps it
10. What this document doesn't settle

---

## 1. Units, and the evaluation strategy

**Units of cost.**
- An **evaluation** is one rule body computed at one key (LANGUAGE §5).
- A **read** is one lookup of an input row, a band row or a cell.
- A **walk day** is one day of the day walk with at least one movement, state pay or anchor. Days with nothing on them cost nothing.
- A **movement** is one booked or modelled-past movement.
- **P** is the number of places, **I** the number of pay instances, **M** the number of movements, **D** the number of days in [S, E], **R** the number of records, **r** the number of rows of one target.

**The strategy** follows the prior art's lessons (`prior-art/datalog-incremental.md` §3–5):

1. **Load once per reality.** Parsing, the item graph, the row indexes and the checks run once, and are cached under the reality's hash (`ENGINE.md` §3.1). Each file is also cached under its own hash, so an amendment re-parses only the files it changes.
2. **Rules are data, interpreted.** Rule bodies are planned once per load: names resolved to slots, row lookups to index positions. They are never compiled into engine code. A changed rule then costs a re-plan of that rule, not a rebuild (N-104, N-2007; DDlog's mistake, `datalog-incremental.md` §4.7).
3. **Static dependencies.** The item graph is known before anything runs. It schedules the work, refuses cycles, and bounds every cone (§4).
4. **Demand-driven.** A cell is computed when an ask, a pay or another cell needs it. Nothing an ask doesn't reach is computed (N-1602). The flow pass and the day walk run only when something reaches a place (`ENGINE.md` §3.3).
5. **Coarse tracking.** A cell is (item, keys, trimmed context): a def at a person and a year, not each arithmetic step. Each cell records its read set. Fine-grained tracking costs 1.5–3.5× when everything is dirty (Adapton), so arithmetic inside a cell isn't tracked (`datalog-incremental.md` §4.8).
6. **Paths aren't tracked.** A Monte Carlo path's layer keeps no read sets: paths share nothing but the shared layer, and are dropped after their terms (§5).
7. **Memo across amendments.** Within a session, the memo survives an amendment. Cells whose read sets the amendment doesn't touch stay valid, and a recomputed cell equal to its old value stops propagation (Salsa's early cutoff, §4).

---

## 2. The run: flow pass, day walk and state reads

### 2.1 The flow pass

Each flow pay instance costs:
- its `on` at the ask's cut K, then its `when` and `amount` at its booking cut min(t, K) (`ENGINE.md` §5): a few evaluations, mostly memo hits, since the law and the facts are shared cells;
- one more evaluation per later version that reaches its read set (back pay, `ENGINE.md` §5). Versions that don't reach it are skipped by an index lookup, with no evaluation.

Total: O(I · c), where c is the evaluations per instance, plus the movements sorted once, O(M log M).

### 2.2 The day walk

For each walk day:
1. the records of covered places, O(1) each;
2. the flow movements of that day, O(1) each;
3. the state pays due, each with its state reads;
4. one change point per place that moved;
5. the place checks, O(places that moved).

Total: O(walk days + M + state pay instances · c).

**Balance reads:**
- `bal(x)` during the walk: O(1).
- `bal(x, d)` at a past day: binary search over x's change points, O(log Mₓ).
- `flows(x, span)`: two lookups in x's prefix sums, O(log Mₓ).

**The prototype's floor** (spec §7): one run of 12 054 days with 18 144 movements took about 0.34 s. That is about 28 µs per day, or 19 µs per movement, all in:

```
$ python3 -c "from fractions import Fraction as F; print(float(F(34,100)/12054*10**6), float(F(34,100)/18144*10**6))"
28.206404513024722 18.738977072310405
```

### 2.3 `plan_low`, the look-ahead

`plan_low(x, a, b)` = bal(x) − P(t) + min over u in [a, b] of (F(u) − F(t) − h(u)). F is the cumulative sum of x's planned flow movements, as known on walk day t at cut min(t, K); h(u) is x's hold on day u, and P(t) the promised movements booked so far that no record confirms (`ENGINE.md` §4.2). The tree holds F − h.

**Range minimum.** F is held in a segment tree over the days with movements:
- a query is O(log n);
- adding δ to every F(u) from day u₀ on is O(log n), by a lazy range add.

n is the number of days on which x has a planned movement or its hold changes: at most 12 054 over 33 years, so log₂ n ≤ 14.

```
$ python3 -c "import math; print(math.ceil(math.log2(12054)))"
14
```

**Why a segment tree, not a sparse table:**
- A sparse table answers in O(1), but it is rebuilt from scratch when F changes, in O(n log n). That would be about 169 000 entries per rebuild for one place over 33 years.
- F changes during the walk. Before K, the cut min(t, K) rises with t. Each time t passes a version's `known` date, the instances that version changes get new amounts, and F gets one range add per changed instance.
- The cut never goes backwards during a walk, so one tree is updated in place, in walk order.
- On Monte Carlo paths, F counts every draw at its plan value (LANGUAGE §5.14), so the tree is built once in the shared layer and read by every path.
- A group of places (LANGUAGE §5.13.1) gets one tree over the sum of its members' F. Groups are fixed at load, so their number is the number of distinct groups the model writes.

**Cost:** O((n + Δ) log n) per place for the whole walk, where Δ is the number of instance changes the versions known before K cause. A query costs about 14 steps, which fits the prototype's look-ahead floor of 0.1–0.3 µs per decision.

### 2.4 `next_on` and `worth`

- `next_on(p[keys], d)` is a binary search over p's instance dates as known on the walk day, O(log I_p).
- `worth(x, d)` is a balance read and a price read, O(log Mₓ + log r).

---

## 3. Cut classes, scenarios, pins and rolling origins

**Cut classes per item.** A cut matters only through the versions it admits (`ENGINE.md` §2). The engine refines this per item:
- The **cut class of a cell** is the interval, between consecutive `known` dates, of the versions that hold candidate rows, rules or key additions in the item's cone. The cone is found statically from the item graph.
- Two cuts in the same interval give the cell the same result. So K is trimmed out of the context like the path and the axes are (`ENGINE.md` §3.2).
- **The case that matters most:** monthly bank statements are versions known every month. They are read only by the movement input's cone: booking on covered days, the residuals and defs over history. A tax def reads none of them, so a monthly cut doesn't split the tax's cells.

**Booking cuts.** Each instance dated t has its `when` and amount evaluated at min(t, K); its `on` is read at K. For instances before K, the booking cut of a cell is also a class, and most instances in one year share the law's class. So a 33-year run costs about one evaluation set per law version, not one per day.

**Scenarios.**
- A scenario's layer adds one candidate per read of a target it writes, O(1).
- Its cells outside the cone of what it writes are the base's cells, shared (N-2004).
- A stack of up to 4 scenarios shares in the same way, layer by layer.

**Pins** (`as if`, `with`) work like a scenario: each distinct pin set has its own memo layer over the unpinned one, and only the pinned items' cone is computed in it. A pin set is keyed by its values (`ENGINE.md` §3.2), so a pin given by an expression costs one evaluation of that expression, outside the `as if`, before its layer is found.

**Axes in the timeline** (`ENGINE.md` §3.3). The flow pass and the day walk run once per combination of the options of the axes the pays reach. An axis that only defs and asks read doesn't split the timeline. The timeline's cost therefore multiplies by the product of those axes' option counts: three two-option axes reached by pays make 8 timelines. `LOAD-FORKS` caps any one ask at 64 combinations, and a household's pays usually reach no axis at all, so its timeline stays one. A case like CMP-01, whose pays read three axes, pins them in its asks or pays the 8 runs.

**Walk-bound defs** (`ENGINE.md` §3.2). A def that reaches a state read is evaluated afresh at each reader's point of the walk, not memoised across readers. Its cost is one evaluation per state pay instance that reads it, which the day walk already counts (§2.2).

**Rolling origins** (`ENGINE.md` §8). A backtest over n origins is n cuts. For each origin:
- cells whose cut class contains that origin, and the previous one, are shared;
- the flow pass recomputes only the instances whose read sets hold a version known between the two origins;
- the walk restarts at the first day whose movements differ (§4.2).

Example: 24 monthly origins over one year of law. They share every law cell, and differ only in the records and facts known month by month. The cost is about one full run, plus 24 walks from their first changed day.

**`as known` costs** one evaluation per distinct cut class reached, never one per date.

---

## 4. A change costs what it touches

This answers the spec's Q15 (§7, N-2004): caching by hash *and* incremental re-evaluation, both coarse.

### 4.1 Invalidation

An amendment (OPERATIONS §1) has a **direct set**:
- the rows it adds or restates: target, keys, spans, known;
- the rules it adds: def, label, v;
- the keys it adds to indexes;
- the items it adds or revises.

**Three steps:**
1. **The static cone.** The items reachable from the direct set in the item graph. Anything outside it is untouched by construction: no read set check is needed (N-1705).
2. **Verify downward, from what the asks need** (Salsa's verifying traces):
   - A memo cell in the cone is checked against its read set.
   - If none of its reads has a new candidate row, a new rule or a new key, it stays valid. This is O(its read set), with no evaluation.
   - Otherwise it is recomputed.
3. **Early cutoff.** A recomputed cell equal to its old value (the same value, holes, refusals and register) leaves its readers valid. Example: a raise that leaves a person in the same band changes the tax, but the band tests above it stay valid.

**Durability tiers** cut the verification walk (`datalog-incremental.md` §3.7). Each row is tagged by the kind of thing that changes it:

| Tier | Holds | Changes |
|---|---|---|
| law | law parts' rows, bands, rules | yearly |
| facts | household facts, records | weekly to monthly |
| pins | pins, fitting parameters, scenario rows | each request |
| paths | draws | each path |

A cell that reads only the law tier skips verification entirely when an amendment touches only facts. That is O(1).

### 4.2 The walk restart

The walk is a fold, so a change on day d* changes balances from d* on. The engine keeps a **checkpoint**: every place's balance and the walk's state (the segment trees of §2.3) at the start of each month.
- The new walk starts at the last checkpoint before d*, the first day whose flow movements, records or state pay inputs differ.
- From there it runs to E. Balances usually differ for good once money has moved differently, so early cutoff inside the walk is rare. It is still checked at each month's checkpoint: if every balance and every pending movement is equal, the rest of the old walk stands.
- **Checkpoints cost** P balances plus the trees' changed nodes per month. For 33 years: 396 checkpoints of a few dozen balances, small.

### 4.3 What one turn costs

A conversation turn "validate a patch, run reality and the scenario, compare, answer" (N-2001) costs:

| Step | Cost |
|---|---|
| Re-parse the changed files | O(their size); milliseconds |
| Reload | the graph checks are O(items + edges): hundreds of items in a household reality |
| The gate, if a law part changed | its `check` asks' cones (§8: about 1 s budget) |
| The reality run | memo hits, outside the amendment's cone |
| The scenario run | the cone of what the scenario writes, plus a walk from its `from` date (rows can't start before it) |
| `plan` | a diff of two timelines, O(M), plus the asks' results |

So a change to next year's law re-walks from the first day whose inputs differ (§4.2): next year, or earlier where a look-ahead window from a day before next year reaches into it; never from 1993. A change to a household fact dated today re-walks from today.

### 4.4 The fitting loop

Claude fits a parameter by replaying history hundreds of times (N-2005, `ENGINE.md` §6.4). Each step pins one input:
- **Shared across steps:** every cell outside the pinned input's cone (law cells are durable), and the records' booking (records are inputs no pin reaches).
- **Recomputed each step:** the pinned input's cone, the walk from the first day it changes, and the residual ledger, which is O(lines).

**Estimate.** Replaying two years of history (730 days) at the prototype's 28 µs per day is about 20 ms per step. 200 steps are then about 4 s. That is a background task, not one turn, and the Rust engine should beat the prototype's per-day cost.

```
$ python3 -c "print(730*28.2/1000, 200*730*28.2/10**6)"
20.586 4.1172
```

A spending parameter that touches every day can't be cut short: the whole replay window is walked each step. The design keeps that walk cheap: no fixed point, no iteration, one pass (`ENGINE.md` §4).

---

## 5. Monte Carlo

`ENGINE.md` §7. Costs:
- **The shared layer:** computed once, the same as a deterministic run without draws.
- **Per path:** the draw-dependent cone only, its flow instances, and a walk if a balance depends on a draw. Each draw is one SHA-256 and one quantile, about 0.3–1 µs (LANGUAGE §5.14).
- **No tracking in paths:** no read sets and no memo verification. A path's memo layer is a flat table, dropped after the path.
- **Terms kept:** N values per ask term, for example 16 bytes × 100 000 paths × 10 terms = 16 MB at the cap.
- **Statistics:** O(N log N) for quantiles. Standard errors are exact over the kept terms.
- **Workers:** contiguous blocks, merged in path order. Exact integer sums are order-free, so merging is exact (N-1401).

**The prototype's floor:** 1 000 paths × 30 years daily in 0.45 s on 6 workers, with no law per path. That is about 250 ns per path-day per worker:

```
$ python3 -c "from fractions import Fraction as F; print(float(F(45,100)*6/(1000*10958)*10**9))"
246.39532761452818
```

**Law per path** (R-20) adds the law cells in the draw cone: per path, about one tax computation per person per year. Its cost is unmeasured (spec §7). The design keeps it small:
- law cells are planned once and shared, except their values;
- a path's law cells read the shared layer's bands and rates directly;
- only cells whose inputs depend on a draw are recomputed per path.

The budget (§8) is the prototype's floor plus the law, with path statistics streamed after the deterministic answer.

**A path on demand** (an explanation, N-1406) is one path: its shared layer is cached, so it costs the draw cone of one path. That is the prototype's ≈ 20 ms floor or less.

---

## 6. Numbers

Exact arithmetic is the default (LANGUAGE §7), so its cost must be bounded.

**Fast paths:**
- **Money** is a whole number of minor units: a 64-bit integer, widened to 128 bits on overflow, then to a big integer. The household's money stays in 64 bits.
- **`num`** is a rational with 64-bit numerator and denominator, widened to big integers when they overflow.
- **Rates and fractions** from the law are short decimals: small denominators.

**Where denominators grow:**
- A recurrence that compounds without rounding: 360 months of (1 + r/12) with a 2-decimal r. Its denominator is about 1 200³⁶⁰, about 1 100 digits:

  ```
  $ python3 -c "import math; print(360*math.log10(1200))"
  1108.5052485771448
  ```

  Big-integer arithmetic at that size is a few µs per operation, so one such recurrence costs milliseconds.
- **The cap:** `RUN-DIGITS` refuses a value past 4 096 digits in its reduced numerator or denominator, so no value grows without bound. Law rounds where it says to, which resets the size at each `round`.
- **Why 4 096.** The cap is a cost bound, not a precision rule. At 4 096 digits (about 13 600 bits) a schoolbook multiplication costs about 14 times one at 1 100 digits:

  ```
  $ python3 -c "import math; print((4096/1108.5)**2, 4096*math.log2(10))"
  13.653641673629918 13606.617476658635
  ```

  So an operation at the cap is tens of µs, and a recurrence of a few hundred keys at the cap stays within a turn's budget (§8). Past it, cost grows with the square of the size. A long product of market data (LH-15's and LH-16's 720 monthly ratios, each with several digits) can reach the cap: the model then rounds where its note says, a reading of the modeller that `explain` shows, never a silent truncation.

**Transcendentals** (the `to u` list of LANGUAGE §7.3: `exp`, `ln`, `sqrt`, `pow` with a non-integer exponent, the continuous distributions' functions and `poisson`'s, a normal forecast's `crps`, and `periodic` with a non-integer power) are computed by interval refinement (LANGUAGE §7.3):
- Start at about 64 bits more than the target unit needs. Double the precision until the interval rounds to one value at the unit, or 2¹⁶ bits are reached (`RUN-UNDECIDED`).
- The typical cost is one evaluation at the first precision. A second round is needed only when the value lies very near a rounding boundary.

**Rounding** is O(size of the number), once per `round`.

---

## 7. Cost per construct

Each construct in LANGUAGE §5 states its cost there. Collected:

| Construct | LANGUAGE | Cost | Memo key |
|---|---|---|---|
| header, source, unit, literal | §5.1, §5.2, §5.4, §7.1 | none at run time | — |
| version, row read | §5.3, §5.7 | O(log r) per read: rows grouped by (target, keys), sorted by span | (keys, period, cut class) |
| axis | §5.5 | multiplies only the cone of defs that reach it; ≤ 64 combinations per ask (`LOAD-FORKS`) | axis options in the context |
| index, key | §5.6 | O(1) per key | — |
| input | §5.7 | O(log r) per read | (keys, period, cut class) |
| band | §5.8 | O(log rows) per call; `marginal(b, x)` O(rows below x); `band_lo`, `band_hi` O(log rows) | (table version, key for a keyed band, value) |
| place, balance | §5.9 | O(1) in the walk; O(log Mₓ) at a past day | change points per place |
| given | §5.10 | one candidate row more per read of the def | as a row |
| def | §5.10 | O(rules) tests per key (1–6 rules in the cases), only the winner's value evaluated; a recurrence over n keys is O(n), iterative, in either direction; a read at a computed key keeps a stack of the (def, keys) being evaluated, for `RUN-CYCLE` | (keys, law date class, axis values, cut class, pin set; the walk point for a walk-bound def) |
| `as if` | §5.10 | one memo layer per distinct pin set; only the pinned items' cone; a pin's expression evaluated once, outside | pin set, by values |
| fill | §5.11 | O(A log A) for A arcs (at most 1 320 arcs for 30 years of CB-05's `spouses`: a year back and ten forward, between two spouses each way); a keyed fill, that per key | (fill key, supplies, demands, order) |
| prorata | §5.12 | O(n log n), once per (total, weights) | (total, weights) |
| flow pay | §5.13 | O(1) per instance; one evaluation more per version reaching a booked instance | instance |
| state pay | §5.13.1 | its state reads, in the walk | none: it is the walk |
| `flows` | §5.13.1 | O(log Mₓ) by prefix sums | — |
| `plan_low` | §5.13.1 | O(log n) per query; O(log n) per changed instance (§2.3) | segment tree per place |
| draw | §5.14 | one SHA-256 and one quantile, 0.3–1 µs; discrete quantiles O(k) | (address, path) |
| scores | §5.15 | O(n log n) per sample score | — |
| ask | §5.16 | its cone | ask result under (reality hash, request) |
| path ask | §5.16 | N × the draw cone, plus O(N log N) for statistics | — |
| `solve` | §5.16 | ⌈log₂((hi − lo)/u)⌉ + 3 evaluations of F's cone | (bracket, unit, pins) |
| `as known` | §5.16 | one evaluation per distinct cut class reached | cut class |
| bound | §5.17 | none | — |
| records | §5.18 | sorted once per cut class, O(R log R); booking O(1) each; `covered(x, span)` O(log V) over the covering versions' spans | (place, day) |
| residuals | §5.16, ENGINE §6.3 | O(lines + log) from the timeline and the modelled past; no re-evaluation | — |
| scenario | §6.4 | one candidate more per read of a target it writes; everything else shared | scenario |

**An aggregate over an open index** (a def summing bank rows of one category for the year before y) costs O(R) per key, so O(R · Y) over Y years. It is memoised per cut class, which changes monthly at most. When the `where` compares a field of the row with a function of the def's key, the engine may group the rows once by that function, which gives O(R + Y). That is an optimisation, not a semantic: the result is the same (`BUILD-ORDER.md`, step F).

---

## 8. Budgets

The budgets are the spec's §7 targets. "Floor" is the prototype's measurement, and "budget" is what the engine must meet, with the 10× headroom of N-2002 where it applies.

| What | Floor | Budget | Where the cost goes |
|---|---|---|---|
| **A conversation turn** (N-2001): validate a patch, run reality and the scenario, compare, give the deterministic answer | — | **1–2 s** end to end | §4.3: the cone of the change, the walk from its first day, the diff |
| **A deterministic run, cold** (N-2002): 33 years daily, 10× the prototype's instances and bank rows | 0.34 s at 1× | **≤ 1 s** | the walk and the flow pass. 10× the movements is 181 440, so ≤ 5.5 µs per movement all in |
| **A deterministic run, warm** | — | memo hits: **≤ 50 ms** | the asks' reads |
| **Monte Carlo** (N-2003): 1 000 paths × 30 years daily, law on every path | 0.45 s on 6 workers, no law | **≤ 1 s on 6 workers**, after the deterministic answer, or from the cache | the draw cone per path, no tracking (§5) |
| **One look-ahead decision** | 0.1–0.3 µs compiled | **≤ 0.3 µs** | a segment tree query, about 14 steps (§2.3) |
| **One path replayed for an explanation** (N-2004) | ≈ 20 ms | **≤ 20 ms** | one path's draw cone over the cached shared layer |
| **The gate on one law part** (N-2007): about 9 checks and the threshold table | ≈ 0.9 s | **≤ 1 s** | each check's cone, once per planted error (61 copies × 9 checks in the floor), plus three evaluations per edge for the table |
| **One fitting step** (N-2005): pin, replay a few years, residuals | — | **≤ 20 ms** for two years of history | §4.4 |
| **Determinism** | — | the same result to the grosz across runs, threads and machines | exact arithmetic; ordered merges (LANGUAGE §7.5) |

```
$ python3 -c "from fractions import Fraction as F; print(18144*10, float(F(1)/181440*10**6))"
181440 5.511463844797178
```

---

## 9. What can grow, and what caps it

Every quantity that can grow without bound has a cap, a refusal, or a cost that the author sees.

| Grows with | Cap or refusal | Where |
|---|---|---|
| axis combinations in one ask | 64 (`LOAD-FORKS`) | LANGUAGE §5.5 |
| Monte Carlo paths | 100 000 (`REQ-PATHS`) | ENGINE §11 |
| the size of an exact number | 4 096 digits (`RUN-DIGITS`) | LANGUAGE §7.1 |
| the precision of a transcendental | 2¹⁶ bits (`RUN-UNDECIDED`) | LANGUAGE §7.3 |
| scenario stacks and `as if` nesting | 4 deep (`LOAD-SCENARIO`, `LOAD-ASIF`) | LANGUAGE §5.10, §6.4 |
| a recurrence | bounded by its index, O(n), iterative | LANGUAGE §5.10 |
| expression nesting | 8 deep (`LOAD-DEPTH`) | LANGUAGE §2.3 |
| a law's own iteration | written as a bounded recurrence, never a fixed point | LANGUAGE §5.10 |
| a `solve` | its bracket and unit fix the number of steps | LANGUAGE §5.16 |
| rolling origins | the origins index the author writes: one cut class each | §3 |
| records | O(R log R) per cut class; real volume unknown (spec §7) | §7 |
| fill arcs | O(A log A); the author's arcs | LANGUAGE §5.11 |

**The out-of-reach steps** are declared bounds, not slow computations (N-2008, `BOUNDS.md`):
- the fewest transfers to settle a group;
- "eventual" ruin over an infinite horizon;
- actuarial values that need a mortality table the case doesn't give (LH-17's step 6; LH-18's annuity values are computed exactly, `to u`, BOUNDS row 49);
- the log score of path forecasts.

---

## 10. What this document doesn't settle

- **The cost of law per path.** It is estimated in §5 and measured at step F. If it is over budget, the first remedy is to hoist more into the shared layer: bands and rates are already shared, so only the values that depend on a draw are recomputed.
- **The real import volume** (spec §7). The 10× headroom is the spec's proposal; the user can change it.
- **The overhead of read sets** on a cold run. Soufflé's provenance costs about 1.27× (`datalog-incremental.md` §3.11). Cells are coarse, and paths keep no read sets, which keeps it small. Step F measures it.
- **The checkpoint interval** (§4.2). Monthly is a guess. It trades memory against the length of a restarted walk.
- **Whether a persisted memo is worth it** across sessions, beyond the results stored by hash (N-2004). The design doesn't need one: load and a warm run are within budget from the stored reality.
