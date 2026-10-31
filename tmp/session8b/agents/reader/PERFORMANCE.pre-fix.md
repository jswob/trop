# Tally: performance

This document covers, for **tally 2.1**:
- how the engine evaluates, and what each construct costs;
- the budgets it must meet (§8), and what caps what can grow (§9);
- how a change costs only what it touches (§4, a seam for now).

It plans the engine. It carries no language meaning: what a construct means is `LANGUAGE.md` (L) and how the engine computes it is `ENGINE.md` (E). Where this document and those differ, they win (G1 item 9). It names no technology: no library and no Rust API. Those live in `tmp/research_notes/engine-rust-strictness/BUILDER.md` and `RULEBOOK.md`, cited here by section and row (BUILDER §7, R-42).

**Estimates, not measurements.** Nothing here was built or run. The figures come from refcore, the reference interpreter, as session 8 measured it (`tmp/session8/shape.md`), and from the six research reports R1–R6 (`tmp/research_notes/engine-prep/`). Each estimate shows its arithmetic in a `python3 -c` block with its output, and is marked an estimate. Nothing on paper is "proven".

**Every budget number is proposed until S** (D4-6). The tracer slice S measures it; the number is then fixed with the user; a number found too tight is changed only through a logged row (DONE.md's approvals).

Ids: D1-1..D6-6 are session 8's decisions, approved by the user at gate G1 on 2026-10-30 (`tmp/session8/decisions.md`); K8-n are the lead's settlements of contradictions and SO8-n the spec observations (`NOTE.md` §2, §3); P8-n are this document's budget rows (§8), which `BUILD-ORDER.md` cites.

**Contents**
1. Units, and the evaluation strategy
2. The run: the plan pass, the event-driven walk, projections
3. Contexts as layers: the memo, trimming, memory
4. A change costs what it touches (a seam)
5. Paths and parallel timelines
6. Numbers
7. Cost per construct
8. Budgets
9. What can grow, and what caps it
10. What stays open

---

## 1. Units, and the evaluation strategy

### 1.1 Units of cost

The units are `shape.md`'s. They are counts, so they are the same on every machine and every run, and they carry over from refcore to the engine (D4-8).

| Unit | What one is |
|---|---|
| **ev** | one expression node evaluated |
| **def cell** | one read of a def at (item, key values, context), the memo key (E §3.2, seam F). A **hit** finds the cell in the memo; a **miss** computes it |
| **read** | one read of an input by the read rule (L §4.6), keyed by (input, keys, context); a **distinct read** is one such key, counted once |
| **walk day** | one day the day walk steps through (E §4.2) |
| **non-empty day** | a day with at least one act: a reading, an observation, a leg, a decision (the full list is §2.2) |
| **leg booked** | one movement leg written to a place's log |
| **parts group** | one group of parts work (a take, an arrival) at a place that keeps parts (L §3.6) |
| **walk shared** | a timeline's prefix taken from another's checkpoint, not walked again (§2.4) |
| **cell recomputed locally** | a shared-layer cell a worker computed for itself (§5.3) |
| **Num result** | one exact value made, sized by the larger bit length of its reduced numerator and denominator |

Other symbols: **P** places, **M** movements, **Mₓ** the change points of place x, **D** the days in [S, E], **R** observed rows, **r** the rows of one target, **L** the live labels of one (place, unit), **N** paths, **W** workers.

**The counters.** The engine counts these units behind a build option `counters`, which is off in the release build, like the fault-injection hook (R-51), and prints them with `--stats` (D4-8, K8-15). The counted build and the timed build differ; their outputs are byte-identical. The counters are the first instrument of §8 and the profiler's first stop (BUILDER's performance section).

### 1.2 The strategy

The order of magnitude is in the structure, not in how fast one node runs (NOTE §1 item 1). refcore steps every calendar day though only 2–7 % carry work, and redoes the same long walk 8 to 64 times (`shape.md`, "What carries over" 3). The strategy removes that work first, and keeps the evaluator plain.

1. **Load once per reality.** Parsing, the item graph, the row indexes and the load checks run once and are cached under the reality's hash (E §3.1). Each file is also cached under its own hash, so an amendment re-parses only the files it changes.
2. **Rules are data, interpreted.** Rule bodies are resolved once per load: names to slots, row reads to index positions. They are never compiled into engine code, so a changed rule costs a re-resolve of that rule, not a rebuild.
3. **A tree-walk over the resolved tree** (D1-1). One case per node kind. Closures in place of the walk only where a measured ask misses its budget: published measurements put closures about 1.5× ahead of a plain tree-walk, and bytecode no better than closures (R1 §2 item 2).
4. **Recursion to a counted depth** (K8-1). The evaluator recurses across cells up to a fixed count, then pushes the cell on an explicit work stack and replays it. Every thread is spawned with an explicit stack size sized for the count (R-42 amended, D3-1). Joint-model nesting is walked as a worklist (R-16). Inside one expression, recursion is bounded by the language (`LOAD-DEPTH`, 8, L §2.3).
5. **Static dependencies.** The item graph is known before anything runs (E §3.1 steps 5–8). It schedules the work, refuses cycles, stratifies decisions and bounds every cone.
6. **Demand-driven.** A cell is computed the first time something needs it (E §3.2). Nothing an ask doesn't reach is computed. The plan pass and the walk run only if something the asks need reaches a place, a move or a balance (E §3.3).
7. **Coarse cells, the engine's own memo** (D1-2). A cell is (item, key values, trimmed context): a def at a person and a year, not each arithmetic step. Each cell keeps its read set, built now (D1-5): adjustments (E §5 step 2), `inputs-of` and `readers-of` need it. The memo is the engine's own, not a framework's (R1 §2 item 3).
8. **Contexts as layers** (D1-3). Scenarios, pin sets, axis combinations, members, paths and projections are layers; a cell lives in the lowest layer its cone reaches; a recurrence keyed by day or period is a dense column (§3).
9. **No eviction** (D1-6). Memory is released when its scope ends: a path, a search step, a projection, a member. Peak memory is then a function of the ask, not of timing (§3.5).
10. **The walk is event-driven** (D1-4): only non-empty days are stepped, with monthly checkpoints; timelines are forked at their first different day, built second (§2).
11. **Joint members are contexts** (D1-7, seam J): each with its own memo and a walk run lazily up to the last day read; pruning to the places read comes second (§2.6).
12. **Paths and independent timelines run in parallel; every merge is sequential** (D3-1..D3-7). Workers fill numbered slots; one thread merges them in task-index order. Paths keep no read sets (§5).
13. **Numbers in two tiers** (D2-1, D2-2): a small exact form on 64-bit integers and a big one, with the whole-number shortcut and Lehmer's gcd (§6).
14. **The memo across amendments is a seam** (D1-5): verification and early cut-off are designed (§4) and built only when a measured turn misses its budget.

Labels cost only where a kind opts in (`engine/BUILD-NOTES.md`, the labels' cost): a place whose kind is neither `owned` nor `batched` pays nothing for parts (§7).

---
## 2. The run: the plan pass, the event-driven walk, projections

A run, for each distinct (cut class, world, pin set, clauses, axis options the moves reach) the asks need, is the plan pass, then the day walk, then the asks (E §3.3). Both passes run only when something the asks need reaches a place, a move or a balance; a model with no places runs only its asks.

### 2.1 The plan pass

Tally 2.1 keeps the plan pass (E §4.1). For each plan move, in the item graph's dependency order, it enumerates the instances over their binding tuples, evaluates each `on` at the ask's cut K, then `when`, legs and attributes at the booking cut min(t, K), classifies the instance (outside [S, E], uncertain, modelled past, or booked) and applies later versions' adjustments (E §5).

- **Per instance:** a few evaluations, mostly memo hits, since law and facts are shared cells; one more per later version that touches its read set. A version that touches nothing in the read set is skipped by an index lookup, with no evaluation (E §5 step 2).
- **Total:** O(I · c) for I instances and c evaluations each, plus one sort of the booked movements by (day, stratum, move id, keys), O(M log M) (E §4.1, last paragraph).
- **What it hands the walk:** the booked legs by day, the decision instances by day (a decision's day reads no running state, L §5.14.1, so every decision day is known before the walk), the hole and window movements, and the instance lists that `flows`, `next` and the projections read.

### 2.2 The event-driven walk

E §4.2 states the walk as a fold "for each day t from S to E", and says "Balances are step functions, so only days with movements or readings cost anything" (E §4.2 step 4, l.198). The engine steps only the days that hold an act, in the same order, and leaves every other day's balances as the step function's last value (D1-4). This is where `shape.md`'s 93–98 % of empty days go.

No section of L or E lists what makes a day non-empty (SO8-1, open item O-1). The list below is derived from E §4.2's steps and the sections they cite. Each row names the construct that puts work on day t, and what the walk does that day. A day is non-empty if any row holds for it; every other day has no act, and the walk's state on it is the previous day's.

| # | Day t holds | Construct | What the walk does on t | Source |
|---|---|---|---|---|
| N-1 | the run's start, t = S | the header's `start` | opens every place from its `opening` readings; places alive before S without one stay `HOLE-ANCHOR` until their first anchor | E §4.2 (opening), step 0, l.189–190; L §5.10 l.1088 |
| N-2 | a reading at `t start` the cut sees: an `opening` with t before K or t = S, a `stated` with t before K | a `reading` input's row, `at D start` | anchors or pads the place by its version's gap role (§6.1), parts included at a place that keeps them | E §4.2 step 0, l.190; E §6.1 l.294–298; L §4.6 l.728 |
| N-3 | an observation dated t, with t before K | an `observed` input's row, `on: t` | books each end by E §6.1's rule in the day's first batch, or, if it fulfils a decision instance dated t, at that instance's point (N-5) | E §4.2 step 1, l.191; E §6.1 l.287–289; L §4.5 l.699 |
| N-4 | a booked plan leg dated t: a plan instance's day, or a leg's own `on` | a plan move (no `reads`), status `past` or `plan` | books the leg in the day's first batch; at a place that keeps parts, its take and its arrival (with the arrival rule run at booking) | E §4.2 step 2, l.192; E §4.2 parts, l.205–206; L §5.14 l.1348 |
| N-5 | a decision instance due on t, whatever its `when` turns out to be | a move with `reads` | runs it at its stratum and point, books its legs dated t at once, and its own observations dated t after it; on a covered day before K, or where an observation fulfils it, computes it as modelled past against the booked balances | E §4.2 step 3, l.193–196; L §5.14.1 l.1374 |
| N-6 | a decision's leg with its own later `on`, dated t | a decision's leg `on d` | books the leg on t; this day enters the agenda only when the decision books, so the agenda takes insertions | E §4.2 step 3, l.195 |
| N-7 | a reading `after m`, m a decision on t (or, where m is modelled past, on the day of the last observation that fulfils m) | a `reading` row, `after m` | pads or anchors right after m, or after those observations | E §4.2 step 3, l.197; L §4.6 l.728 |
| N-8 | a reading at `t end` the cut sees, with t before K | a `reading` row, `at D end` | pads or anchors at the close | E §4.2 step 4, l.198 |
| N-9 | the last day of a place's lifespan | an entry's `lives` span, a `place` section's | refuses the place from t on if a position isn't zero at the close (`RUN-LIFESPAN`) | E §4.2 step 5, l.201; L §5.7 l.942 |
| N-10 | the last day of a party's lifespan, where some place keeps parts labelled to that party | `parties` and a parts kind | checks, after the day's whole order, that the party holds no part at the close; refuses that part's place from t on otherwise | E §4.2 parts, l.208; L §5.7 l.943 |
| N-11 | the first day a hole movement may fall on: a window's first day, or S (or the place's first day) for a day that is a hole and not a window | a movement whose day is a hole or a window | makes both ends' balances holes from t, with the movement's questions, up to each place's next reading | E §4.2 (holes), l.211; E §4.1 step 2, l.177; L §5.14 l.1348 |
| N-12 | the day after a window's last day, for a movement dated by a window and no `windows each` | a planned day `window(a, b)` | ends the stretch of hole balances N-11 began: `bal(x, d)` is a hole only for d inside the window | E §4.1 step 2, l.177 |
| N-13 | V.known of an adjustment that nothing fulfils or waives at the cut | `adjustments` (E §5) | makes the adjusted leg's places' balances holes from t on, past later readings too (`HOLE-CHANGED`); the adjustment itself moves nothing | E §5 steps 4–5, l.258–261; L §5.14.3 l.1435 |
| N-14 | the day of a joint model's own movement into a member's place that no `same` or `fulfils` links | a joint model's versions | books nothing there; makes the place's balance, read in the joint model, a hole from t until the member's next reading of it | L §6.1 l.1805 |

**What is not on the list, and why:**
- **Movements outside [S, E], uncertain instances no observation fulfils, modelled-past plan instances, observations dated before S, and observations or readings dated on K or later** book nothing (E §4.1 step 3, l.179–182; E §4.2 steps 0–1; L §4.5 l.699). A projection or a `with uncertain` read counts the uncertain ones from the plan pass's lists, not from the walk (L §5.14 l.1356).
- **An adjustment that is settled or waived** moves nothing on its day (E §5 step 4); its settlements are ordinary movements, on the list by N-3 or N-4.
- **A lifespan's first day** puts no work in the walk: a place's balance is zero there (L §5.10 l.1088), and a leg into a thing before its lifespan is refused on the leg's own day (N-4, N-5). Putting it on the agenda changes nothing.
- **A `known` date** is not a walk act. What it changes is what a projection read on a later day sees (the cut min(t, K) rises with t, E §4.3 l.230); the projection structure is updated before its next read (§2.5).
- **The sign rule, `WARN-NEGATIVE` and the invariants** are checked at points that fall on listed days: the sign rule after each batch, decision and reading (L §5.10 l.1081–1084), and a stretch below zero starts on a day the balance changes (E §4.2 step 5, l.202). On a day with no act, no leg is booked (invariant 1 has nothing to sum), no position changes (invariant 6), and invariant 2 reduces to the revaluation equalling itself (E §9).

**Inside a projection's own walk** (E §4.3) the same list applies to the window [t, d], with the projection's own sources: the plan movements as known on t, uncertain ones only under `with uncertain`, decisions in the window, and no observation or reading after t (E §4.3 step 2, l.225).

**The oracle.** The engine keeps a step-every-day walk beside the event-driven one, built from the same step functions, that steps every day from S to E. It must print the same bytes; it steps more days. A property test runs both on every case and on T's generated realities, and compares the outputs byte for byte (R1 §3 Q3). A row missing from the list above shows as a difference there. The step-every-day walk is a test oracle, never the shipped path.

**What the event walk costs:**
- building the agenda: one sort of the acts, O(A log A) for A acts, plus an ordered insert for each N-6 leg;
- per non-empty day: the acts of N-1..N-14, each O(1) or the parts work of §7; then one change point per place that moved;
- per empty day: nothing.

`shape.md` gives the size of the saving for the heaviest checks (estimate: refcore's walks × days, times its non-empty share):

```
$ python3 -I -c "print('A', 64*5477, round(64*5477*0.054), 'E2', 8*16070, round(8*16070*0.067), 'U', 3*180, round(540*0.022))"
A 350528 18929 E2 128560 8614 U 540 12
```

So A steps about 18 900 days instead of 350 528, and E2-01a about 8 600 instead of 128 560. E1 is the exception: all 2 498 of its days hold a decision, so its floor is node speed (§10).

**Balance reads:**
- `bal(x)` during the walk: O(1).
- `bal(x, d)` at a past day: a binary search over x's change points, O(log Mₓ).
- `flows(x, span)`: two lookups in x's prefix sums, O(log Mₓ) (L §5.14 cost).

### 2.3 Monthly checkpoints

At the start of each month the walk keeps a **checkpoint**: every place's state (its balance, and its parts per (place, unit) where its kind keeps them), the legs booked ahead (N-6), and the projection structures of §2.5 (D1-4, D2-6).
- A checkpoint is a list of shared place states, copied on write: a place that didn't move since the last checkpoint costs one shared reference, a place that moved costs one copy (D2-6; BUILDER §3).
- Pins and scenarios are small layers over the base's checkpoints, not copies (D2-6).
- **Size for 33 years:** 396 checkpoints of P place states, most of them shared. The interval is a guess and an unknown (§10): it trades memory against the length of a walk restarted or forked from a checkpoint (R1 §6).

The checkpoints serve the forks (§2.4) and the walk restart (§4.3).

### 2.4 Timelines forked at their first different day (second)

Two timelines whose acts agree up to day d share the walk to d: the engine walks the shared prefix once, takes the last checkpoint before d, and walks each timeline on from it (D1-4, second step; R1 §3 Q3).
- **Which timelines:** the `windows each` sub-windows of one line, axis combinations the moves reach, rolling origins, the cuts of `as known`, scenario worlds, and a member's walk asked twice.
- **The first different day** comes from the plan pass's outputs: the first act whose day, legs or amounts differ between the two. For a scenario it is not its `from` alone, since movements before `from` can differ through what reads the scenario's facts, which E §9 invariant 3 lists (L §6.4); the fork takes the earliest of these.
- **Built second,** after the event-driven walk matches refcore on every check. Its oracle is the unforked walk: the same bytes, fewer walks. The counter "walk shared" (§1.1) shows the saving.

`shape.md`'s repetition is the target: 64 walks in A (`windows each`), 8–12 in E2 (members and asks), 20 in X (19 of them projections). How late those timelines diverge is unknown, so the fork's saving is an unknown until the engine runs (§10).

### 2.5 The written projection, `next` and `value`

Tally 2's look-ahead is written: `bal(x, d) from t`, `low(x, a .. b) from t` and `next(K, d)` (L §5.14.1, D-6). They replace 1.0.1's `plan_low`, `next_on` and `worth` (`value(x, d) in C by p`, L §5.10). E §4.3 computes them.

**The general case.** `bal(x, d) from t`, held by item P, is a walk of its own from day t to d: it starts from x's balance at the start of t (or at the reader's point on t), counts every movement dated t..d that doesn't depend on P, each as known on t, and runs the decisions in the window stratum by stratum (E §4.3 steps 1–3). It is event-driven like the main walk (§2.2). Its cost is one walk of its window, memoised per (item, day, cut, clauses) (L §5.14 cost, l.1454). Nesting is finite: a chain of nested projections names each holder at most once (E §4.3 l.228).

**The common case is cheap** (E §4.3 l.230). When the window holds no decision the projection counts, the projection is the start balance plus a prefix sum F of the plan movements as known on t, and `low(x, a .. b) from t` is a range minimum over F.
- F is held in a segment tree over the days on which x has a counted movement: a query is O(log n), and adding δ to every F(u) from day u₀ on is O(log n) by a lazy range add.
- F changes only as the cut min(t, K) rises with t: when t passes a version's `known`, the instances that version changes get new amounts, and F gets one range add per changed instance. The cut never goes back during a walk, so one tree is updated in place, in walk order, before the next read.
- A sparse table would answer in O(1) but must be rebuilt in O(n log n) when F changes, which it does during the walk.
- On a path, a projection that reaches random facts only through its `random` clause is the same on every path, so its F is built once in the shared layer (E §4.3 l.230, E §7 step 1).
- n is at most the run's days, 12 054 over 33 years, so a query is at most 14 steps (estimate):

```
$ python3 -I -c "import math; print(math.ceil(math.log2(12054)))"
14
```

- **Cost:** O((n + Δ) log n) per (place, holder) for the whole walk, where Δ is the number of instance changes that versions known before K cause.

**Groups of places** are written as sums in 2.1, `min(sum(bal(x, d) from t for x in liquid on d) for d in a .. b step 1 day)`, which isn't the sum of the members' lows (L §5.14.1 l.1409). Evaluated as written, it costs (b − a + 1) days × the members' reads. A sum of step functions is a step function, so the minimum can be taken over the members' change points instead; that is an optimisation that changes no value, taken only where a measured ask needs it.

**`next(K, d)`** is a binary search in the sorted days of K's movements, planned and observed, as known at the reader's cut (E §4.3 l.232): O(log I_K).

**`value(x, d) in C by p`** is a balance read and a price read, O(log Mₓ + log r); with `of q`, the parts read of §7 (L §5.10 l.1090, l.1099).

### 2.6 Joint members as their own contexts (seam J)

Each member of a joint model is its own evaluation context, keyed by its pin (the sha256 of its canonical text, E §3.1 step 1), with its own memo and its own walk (D1-7; `engine/BUILD-NOTES.md` § J).
- **First:** a member's walk runs lazily, up to the last day a joint read asks of it. A read of one row of a member with a long walk then costs that member's walk to that day, not to its end.
- **Second:** pruning the member's walk to the places in the reading cone, per (member, place set, last day).
- A member that is itself a joint model nests by the same structure, walked as a worklist, never by native recursion (R-16, SO8-8). With parts, a member's walk state includes its parts.

The target is load row L:18 (§8): session 5 measured `e2_01_payouts` at 67.7 s with a third member against 17.1 s without (fast layer), and 73.1 s for one row of a member with a long walk (`engine/tests/load/load.tsv`).

---
## 3. Contexts as layers: the memo, trimming, memory

### 3.1 The memo key

A cell is (item, keys, context) and its context is E §3.2's: (law date class, cut class, axis options, pin set, world, clauses, path [, walk point] [, projection]). The engine's memo key is (item, **key values**, context id) (D1-3):
- key values, not index positions, so a later function-like def needs no change to the evaluator (seam F, `engine/BUILD-NOTES.md`);
- things, open-index keys and labels as interned symbols, so a key is fixed-size and cheap to compare; output order is by the symbol's text, never its id (D2-3, R-47);
- the context **interned** once as a small id, after it is **trimmed** to what the item can depend on, which the item graph tells statically (E §3.2).

Half of all def reads are first reads (hit rates 50–59 %, `shape.md` "What carries over" 1), so the number of distinct cells matters more than lookup speed. Trimming is what makes cells fewer.

### 3.2 The layers

Each context dimension is a layer over the one below. A cell lives in the lowest layer its cone reaches, and every context above shares it (D1-3; PERFORMANCE 1.0.1 §3).

| Layer | A cell joins it when its cone reaches | Shared by | Freed when (§3.5) | Source |
|---|---|---|---|---|
| cut class | a version or row whose `known` splits the cone's interval | every cut in the interval | the run ends | E §2 l.106, E §3.2 l.152 |
| law date class | a governs boundary or a lifespan in the cone | every law date in the class | the run ends | E §3.2 l.153 |
| world (scenario) | a scenario's versions | the base and other worlds otherwise | the run ends | E §3.2 l.151; L §6.4 cost |
| pin set | a pin, keyed by the pins' values | the unpinned layer otherwise | its `as if`, `solve` step or fitting step ends | E §3.2 l.150 |
| axis options | an axis | every combination otherwise | the run ends | E §3.2 l.149 |
| clauses | a window, a random fact or an uncertain kind under the line's clauses | every clause set otherwise | the run ends | E §3.2 l.149 |
| member | a joint model's member (§2.6) | — | the joint run ends | D1-7; seam J |
| path | a random fact | every path otherwise | its terms are kept (§5) | E §3.2 l.148; E §7 step 3 |
| walk point | a running read (a walk-bound def) | nothing: one cell per reader's point | the decision books | E §3.2 l.145; L §5.14.1 l.1393 |
| projection (holder, t) | a running read or a projection inside the window, or a random fact (§3.3) | its cut class otherwise (§3.3) | the projection's read ends | E §3.2 l.145 |

1.0.1's sections restated as layers:
- **Cut classes.** Monthly statements are versions known every month. Only the observed inputs' cone reads them (booking on covered days, residuals, defs over observations), so a monthly cut doesn't split a tax def's cells.
- **Booking cuts.** An instance dated t is evaluated at min(t, K) (E §5 step 1). Most instances in a year share the law's cut class, so a 33-year run costs about one evaluation set per law version, not one per day.
- **Scenarios** cost one candidate more per read of a target they write; every cell outside that cone is the base's (L §6.4 cost). Stacks are at most 4 deep (`LOAD-SCENARIO`).
- **Pins** (`as if`, `with`) give each distinct pin set a layer over the unpinned one, keyed by values; a pin given by an expression is evaluated once, outside its `as if` (E §3.2). `as if` nests at most 4 deep (`LOAD-ASIF`).
- **Axes in the timeline.** The plan pass and the walk run once per combination of the axes the moves reach; an axis only defs and asks read doesn't split the timeline (E §3.3). The timeline's cost multiplies by the product of those axes' option counts, at most 64 per ask (`LOAD-FORKS`).
- **Walk-bound defs** are evaluated afresh at each reader's point (E §3.2), one evaluation per decision instance that reads them, which the walk already counts. The cell is evaluation-local and is dropped when the decision books (R1 §3 Q2, an inference from E §3.2).
- **Rolling origins** are an index of cut dates, one cut class each (E §8). Cells whose cut class holds two origins are shared, and the walks fork at their first different day (§2.4).
- **`as known`** costs one evaluation per distinct cut class reached, never one per date.

### 3.3 Trimming a projection out of a cell's context (SO8-2)

E §3.2 puts the projection (holder P, day t) in the context of a cell evaluated inside P's walk, and doesn't say whether a cell whose cone never reaches that walk keeps it (SO8-2, open item O-2). R1 §3 Q3 finds that it matters: X's 2.27 M misses are 95 % three day-keyed defs asked from 19 projections. This section derives what L and E allow.

**The test a trimming must pass.** The memo maps a cell to its result: a value, a hole or a refusal, with its register and read set (E §3.2 l.154), and the result is a pure function of the cell (seam F). A trimming makes two contexts share one cell. It can change a printed byte only if the two contexts give that cell different results. If they can't, the trimming is a cache choice, not a meaning, and needs no spec text.

**What being inside P's walk changes** (E §4.3 steps 1–3; L §5.14.1 l.1396–1399):
1. the cut: each counted movement is evaluated as known on t, at cut min(t, K);
2. the running reads: a balance, `flows` of a place or a decision, a decision's legs, and a binding over a decision's movements or `pads` read P's walk, not the main walk;
3. what is left out: the movements that depend on P;
4. a random amount dated after t is read at the `random` clause's value or by the query around the read, not by its sample;
5. uncertain movements and windows count only under their clauses.

Everything else a cell reads goes through the read rule (L §4.6), whose result depends only on the item, its keys, the law date, the cut, the scenario layer and a `wording` pin (L §4.6 steps 1–5), all of them context dimensions already.

**The derivation.** Take a cell C evaluated inside P's walk whose cone, on the item graph (E §3.1 step 5), reaches no running read, no projection and no random fact, and whose clauses are trimmed as E §3.2 l.149 says.
- Change 1 is the cut min(t, K), which the cut class already carries (E §3.2 l.152).
- Changes 2 and 4 can't reach C, by its cone.
- Change 5 is the clauses, already in the context or trimmed.
- Change 3 can't reach C either. C is reached from a counted movement M′, and M′ doesn't depend on P, or it would be left out. A binding over a kind is an edge to every movement it binds (E §3.1 step 5). So if C's cone held a movement that depends on P, M′ would depend on P through C.

So C has the same result inside P's walk as at cut min(t, K) outside any projection. Dropping (P, t) from its context, and keeping the cut class of min(t, K), changes no value, register or read set, and so no printed byte. **It is a cache choice**, and the engine takes it.

**What keeps (P, t):** a cell whose cone reaches a running read or a projection (it is walk-bound inside the window), or a random fact (its value turns on whether its movement falls after t, change 4).

**One assumption, logged as O-W1-1.** L §5.14.1 l.1400 and §5.14.3 l.1444 say a fact a projection reads before the fact is known makes the decision "a hole that nothing later answers". The derivation reads that hole as the read rule's own at cut min(t, K) (`HOLE-UNGOVERNED` with its question, or the `assumed` row known from the start), spreading to the decision. If the projection gave a different hole there, a trimmed cell would print another question, and the trimming would be a meaning.

**The checks.** Byte equality with refcore on every check (DONE gate 10), and a property test that runs every case with the trimming on and off and compares the bytes. Whether X's three defs reach a running read is a fact about X's text, decided statically by the trimming; R1 left it unchecked.

### 3.4 Dense columns and collections

- **A recurrence keyed by day or period** is a dense column per (def, context): a vector indexed by the key's position, filled forward in key order from the last filled key on a miss, never by recursion (E §3.2 l.155; D1-3, D2-5). It stores no key per cell. `shape.md`'s day-keyed defs (A 85 %, X 95 % of misses) become columns.
- **Sparse cells** are in ordered trees; **frozen tables** (row indexes, the agenda once sorted) are sorted slices searched by bisection; **day-keyed data** is in dense vectors (D2-5).
- **No hash map now** (K8-6). It is revisited only if the first engine profile shows sparse memo lookups dominating.
- **A read memo** per (input, keys, law date class, cut class): reads repeat 2–47× per key (`shape.md` "What carries over" 2), and refcore's row scan was 31 % of B's time (session 4.5e).

### 3.5 Scope, and memory per cell

**One memo per context, freed when its scope ends; no eviction** (D1-6, D2-7). The scopes are a path, a `solve` step's or fitting step's pin layer, a projection's read, a member, and the run. What is freed depends only on the ask's structure, so peak memory is the same on every run and with any number of workers. An eviction policy (least recently used) would depend on access order, which isn't fixed under 6 workers; the results would stay equal but peak memory, and a hit on the 8 GB ceiling, could differ (R1 §3 Q4). Eviction is decided only after an engine run of B-07 (D2-7).

**Memory per cell** (estimates, NOTE §1 item 11):
- R1: 60–100 B a cell without its read set, 150–300 B with it;
- R2: 70–90 B a sparse cell, 24–32 B a dense one (the key is the position);
- refcore: about 2.2 KB a cell (X: 4.95 GB over 2.27 M cells). Python's object overhead inflates it; the ratio carries over, not the bytes (`shape.md` "What carries over" 6).

X's 2.27 M cells (estimate): at 60–300 B each, 0.14–0.68 GB; with 95 % of them in dense columns, 60–80 MB:

```
$ python3 -I -c "print(4.95e9/2.27e6); print(2.27e6*60/1e9, 2.27e6*300/1e9); print((2.27e6*0.95*24 + 2.27e6*0.05*70)/1e6, (2.27e6*0.95*32 + 2.27e6*0.05*90)/1e6)"
2180.616740088106
0.1362 0.681
59.701 79.223
```

**B-07** (L:4) stopped in refcore at 83 GB after 4 635 s, with nothing printed. Divided by refcore's ratio of 7–35×, it lands at 2–12 GB if every cell were kept (R1 §3 Q4). Its 4 001 search steps each pin a layer, which is freed after its step, so the kept total is far lower; only the engine can measure it (§10).

---
## 4. A change costs what it touches (a seam)

**Built now:** read sets on every memo cell (E §3.2 l.154), the item graph's static cones, the checkpoints of §2.3 and the pin layers of §3.2. Adjustments (E §5 step 2), `explain`, `inputs-of` and `readers-of` need them anyway (D1-5).

**Built only when measured:** verification across amendments, early cut-off and durability tiers, below. They are built when a measured conversation turn misses 1–2 s (P8-1), or a cold 33-year run misses 1 s at 10× (P8-2); then gate 12 requires every output byte-identical with and without them (D1-5; R1 §3 Q5). Until then, a turn is a cold run of the amended reality: results are cached by the reality's hash (E §1), and a scenario shares the base's memo within one run (L §6.4).

The design the seam leaves room for is 1.0.1's §4, restated for 2.1:

### 4.1 Invalidation

An amendment (`OPERATIONS.md` §1, stale) has a **direct set**: the rows it adds or restates (target, keys, spans, `known`), the rules it adds (def, label, `v`), the keys and entries it adds to indexes and kinds, the items it adds or revises.
1. **The static cone:** the items reachable from the direct set on the item graph. Anything outside it is untouched by construction.
2. **Verify downward, from what the asks need:** a cell in the cone is checked against its read set; if none of its reads has a new candidate row, a new rule or a new key, it stays valid, at O(its read set) with no evaluation; otherwise it is recomputed.
3. **Early cut-off:** a recomputed cell equal to its old result (value, holes, refusals, register) leaves its readers valid.

**Durability tiers** cut the verification walk: each row is tagged by what changes it.

| Tier | Holds | Changes |
|---|---|---|
| law | law parts' rows, bands and rules | yearly |
| facts | the reality's own facts, entries and observations | weekly to monthly |
| pins | pins, fitting parameters, scenario rows | each request |
| paths | samples | each path |

A cell that reads only the law tier skips verification when an amendment touches only facts.

### 4.2 The walk restart

The walk is a fold, so a change on day d* changes the walk from d* on. The new walk starts at the last checkpoint before d*, the first day whose acts (§2.2) differ, and runs to E. It stops early at a month's checkpoint where every place state (balance and parts) and every leg booked ahead is equal to the old walk's; that is rare, since balances usually differ for good once money moves differently. A scenario's walk is the same restart from its first different day (§2.4).

### 4.3 What one turn costs

A turn, "validate a patch, run reality and the scenario, compare, answer" (N-2001), rests on `OPERATIONS.md`, which stays stale; its steps are restated in 2.1's terms and wait for that rewrite.

| Step | Cost |
|---|---|
| Re-parse the changed files | O(their size); milliseconds |
| Reload | the graph checks, O(items + edges) |
| The law gate, if a law part changed | its `check` asks' cones (P8-7) |
| The reality run | memo hits outside the amendment's cone (with the seam), or a cold run (without it, P8-2) |
| The scenario run | the cone of what it writes, and a walk from its first different day (§2.4) |
| The difference (`plan`, `diff`) | O(M) over two timelines, plus the asks |

A change to next year's law re-walks from the first day whose acts differ: next year, or earlier where a projection from an earlier day reaches into it.

### 4.4 The fitting loop

E §6.4: each step pins one input with `with`, replays, and reads the residuals. Shared across steps: every cell outside the pin's cone, and the observations' booking (no pin reaches them). Recomputed each step: the pin's cone, the walk from its first changed day (§4.2), and the residual ledger, O(lines) (E §6.3). This rides on pin layers and checkpoints, which the semantics need anyway, not on the seam.

**Estimate.** P8-8's budget, ≤ 20 ms for two years of history, allows about 27 µs per day all in. 200 steps are then about 4 s: a background task, not one turn.

```
$ python3 -I -c "print(20e-3/730*1e6, 200*0.020)"
27.397260273972602 4.0
```

A parameter that touches every day can't be cut short: the whole replay window is walked each step, in one pass, with no fixed point (E §4.2 l.219).

---

## 5. Paths and parallel timelines

### 5.1 What runs in parallel

Every split in Tally's work is a set of independent evaluation contexts over a shared layer, each a pure function of itself (E §3.2, §7; L §7.4). So workers only map tasks into numbered slots; every merge runs on one thread, in task-index order (D3-1, R3 §2 item 1).
- **First:** Monte Carlo paths (E §7), and the independent timelines: axis combinations the moves reach, `windows each` sub-windows, rolling origins, scenario worlds (D3-2). This reaches A's 64× and X's 20× repetition at up to 6×.
- **Later:** joint members (§2.6) and independent asks of one request.
- **Never:** work inside one timeline or one path; the walk is a fold.

### 5.2 The merge rule

refcore's rule, logged as spec observations SO8-3 and SO8-6 (D3-3):
- per line, the terms in term order;
- the first refusing path, in path order, wins; with none, the union of the paths' holes;
- quantiles over the values sorted stably by (value, path).

Since 2.1 a path statistic's accumulation is uncapped (L §7.1 R-13, CHANGES 2.1-15), so the merge order no longer matters for the sums' digits (K8-16). It still matters for which refusal prints. The engine follows L §7.1: refcore caps a mixed-unit or rate `mean`'s partial sums in path order (K8-17), so gate 10 can differ only where a partial sum passes 4 096 digits.

### 5.3 The shared layer

Cells no random fact reaches are computed once and shared by every path (E §7 step 1). The engine runs the first task alone, freezes the shared cells it made, then runs the rest against the frozen layer, which workers read by plain borrow with no lock (D3-4). A cell a later task needs that isn't in the frozen layer is computed in the task's own layer, and the counter **cells recomputed locally** (§1.1) shows the duplication.

### 5.4 Workers

- **The worker count W is an argument.** The CLI's default is 6, a flag lowers it, and it is never read from the machine (D3-5). Lowering W changes no byte.
- **Every thread is spawned through the one thread module, with an explicit stack size** passed in, so no thread reads its stack size from the environment, and a deep expression can't fail on 6 workers and not on 1 (D3-1; R-42 amended; BUILDER §7). The 1-worker run goes through the same module.
- **No lock, channel or atomic outside that module** (R-42 amended).
- E §7 step 5's "contiguous blocks, one per worker" is a means; the guarantee is the ordered merge (SO8-5).

### 5.5 Costs

- **The shared layer:** once, the same as a run without paths.
- **Per path:** the random cone only, in its own memo layer: the samples, the random-dependent plan instances, the walk if a balance or decision depends on a sample, the ask's terms (E §7 step 2). **Law per path** is just this: law defs in the random cone are cells of the path's layer.
- **A sample:** one SHA-256 and one quantile, about 0.3–1 µs; a discrete quantile O(k) (L §5.16 cost). The address is hashed as UTF-8, refcore's encoding (SO8-4).
- **No tracking:** a path keeps no read sets and no verification; its layer is freed after its terms are kept (E §7 step 3; §3.5).
- **Terms kept:** one value per term per path, about 3.2 MB at 10 000 paths × 10 terms, 32 MB at the 100 000-path cap (estimate, 32 B a value):

```
$ python3 -I -c "print(10000*10*32/1e6, 100000*10*32/1e6)"
3.2 32.0
```

- **Peak memory** ≈ the shared layer + W × (one live task layer and its walk state) + the kept terms (R3 Q5). The per-path layer at B-12's size is unknown (§10).

**The rate N-2003 sets.** 1 000 paths × 30 years daily in ≤ 1 s on 6 workers is about 550 ns per path-day per worker, law on every path. 1.0.1's prototype ran the same without law at about 250 ns. refcore runs LH-17-mc-light at about 0.18–0.25 s per path (`shape.md`, Limits). Estimates:

```
$ python3 -I -c "from fractions import Fraction as F; print(float(F(1)*6/(1000*10958)*10**9), float(F(45,100)*6/(1000*10958)*10**9), 45/256, 65/256)"
547.5451724767294 246.39532761452818 0.17578125 0.25390625
```

### 5.6 Statistics

- **`mean` and `share`** are exact sums over the kept terms, their standard errors correctly rounded to 1e-6 (L §5.16). The accumulator is its own type, outside the 4 096-digit cap check (R-117, 2.1-15).
- **`quantile`** sorts the N terms, O(N log N), and prints the type-1 value with its Maritz–Jarrett standard error (L §5.16). Computed exactly, as refcore does, that needs integers the size of N^N and about N²/2 steps: about 40 000 digits and 5·10⁷ steps at 10 000 paths, 500 000 digits and 5·10⁹ steps at 100 000 (R3 §2 item 6). L §7.1 lets the engine use any method that gives the printed rounding exactly, so it encloses the standard error in intervals refined until they round to one value at 1e-6, and refcore's exact method is its oracle at small N (D3-7). This kernel is on gate 8's list (numbers and rounding).
- **A path replayed alone** (`path p`, an explanation) costs one path's random cone over the cached shared layer (E §7 step 3, N-2004).

### 5.7 Testing gate 6

Gate 6: equal result hashes with 1 and 6 workers, and across runs (DONE.md). The tests (D3-6):
- **a worker sweep,** W ∈ {1, 2, 3, 5, 6, 7}, on the cases and on T's generated realities;
- **a reversed completion order,** test-only, so a merge that leaks finishing order fails;
- **planted violations,** a merge that takes the first finisher's refusal, to show the tests catch it;
- **a deep-stack case,** an ask that nests to the counted depth on a worker thread.

---
## 6. Numbers

Every number is exact (L §7.1), so its cost must be bounded and its common case cheap.

**What refcore made** (`shape.md`, "What carries over" 4): over 99.9 % of results fit 64 bits; 77–99 % are whole numbers; money was always a whole number of cents. E1 is the exception that sizes the big tier: 1 693 results pass 1 024 bits, 1 109 pass 4 096 bits, and the largest is 5 049 bits, about 1 520 digits, against the cap's 13 606 bits. No result fell between 64 and 1 024 bits.

**The tiers** (D2-1, D2-2):
- **`Num`** is small (a 64-bit numerator and denominator) or big, with **no middle tier**: in E1 every result above 63 bits is also above 1 024, so a 128-bit tier would hold almost nothing. A 128-bit intermediate stays only inside one cross-multiplication. The target size is 16 bytes, settled by a size assertion at the first compile (R2 Q1).
- **The whole-number shortcut:** when both denominators are 1, `+`, `−` and `×` are one checked machine operation with no gcd.
- **Short decimals** (law's rates; A 7.2 %, E1 20 % of results) have denominators that divide 10⁶, whose gcds end in a few steps.
- **Money** is a whole number of minor units by type (L §3.1), 64-bit or big; 1.0.1's 128-bit money tier is dropped (K8-3). Money + money is one checked add. A rational appears only where money leaves the money type: money × num, money ÷ anything, shares, `prorata` and the parts splits, path statistics, and a `num CUR` def never rounded (L §7.2; R2 Q2). 64 bits of grosze reach about 9.2·10¹⁶ zł.

**Lehmer's gcd, written now** (K8-2). GMP, CPython and the other big-number libraries R2 read use Lehmer's algorithm at Tally's sizes; the big tier's library (R-30) uses the binary one, which is slower on two balanced big values, as when two compounded sums meet. GMP's own switch to a faster large-number algorithm comes above 330 limbs (arm64) or 618 (x86-64); the cap is 213 limbs of 64 bits and E1's largest result 79, so **Lehmer is the most Tally ever needs** (R2 Q1). A big × small gcd takes one short division, then a 64-bit gcd.

```
$ python3 -I -c "import math; print(4096*math.log2(10), 13606/64, 5049/64, 5049/math.log2(10), (2**63-1)/100)"
13606.617476658635 212.59375 78.890625 1519.9004481074412 9.223372036854776e+16
```

**Where denominators grow.** A recurrence that compounds without rounding, 360 months of (1 + r/12) with a 2-decimal r, has a denominator of about 1 200³⁶⁰, about 1 100 digits. A schoolbook operation at the cap costs about 14 times one at that size (estimates):

```
$ python3 -I -c "import math; print(360*math.log10(1200), (4096/1108.5)**2)"
1108.5052485771448 13.653641673629918
```

**The cap** is `RUN-DIGITS`: a value at a key whose reduced numerator or denominator passes 4 096 digits is refused there (L §7.1). It is a cost bound, not a precision rule. It covers a value at a key, not a path statistic's own accumulation (2.1-15, §5.6): BUILDER §2's "after every operation" predates 2.1, and the spec wins (K8-5, R-117). Law rounds where it says, which resets the size at each `round`.

**Transcendentals** (L §7.3: `exp`, `ln`, `sqrt`, `pow` with a non-integer exponent, the distributions' functions that need `to`, a normal forecast's `crps`, and `periodic`, `factor` and `accrue` with a non-integer power) are decided by interval refinement: start about 64 bits above what the unit needs, and double until the interval lies in one rounding cell, or 2¹⁶ bits are reached (`RUN-UNDECIDED`). The typical cost is one evaluation at the first precision.

**Rounding** is O(size of the number), once per `round`; on an already reduced value it needs no gcd (R2 Q2).

---

## 7. Cost per construct

Each construct's section in L states its cost; collected, with 2.1's constructs and the engine's form of each.

| Construct | L | Cost | Memo key, or state |
|---|---|---|---|
| header, source, unit, literal | §5.1, §5.2, §5.4, §7.1 | none at run time | — |
| version, row read | §5.3, §4.6 | O(log r) a read: rows grouped by (target, stated keys), sorted by span | read memo (§3.4) |
| axis | §5.5 | multiplies only the cone that reaches it; ≤ 64 combinations an ask | axis options |
| index, key | §5.6 | O(1) a key | — |
| kind, lifespan, binding | §5.7 | a lifespan test O(1); a binding `on d` O(things of the kind), memoised per day; a kind test O(1) | (kind, day) |
| `parties` | §3.5, §5.7 | a prelude kind like any other: no cost of its own | — |
| input | §5.8 | O(log r) a read | (keys, period, cut class) |
| band | §5.9 | O(log rows) a call; `marginal` O(rows) | (table version, keys, value) |
| place, balance | §5.10 | O(1) in the walk; O(log Mₓ) at a past day | change points per place |
| def, given | §5.11 | O(rules) tests a key, only the winner evaluated; a given is one candidate more; a recurrence over n keys O(n), iterative, a dense column | (keys, contexts, §3.1) |
| `as if` | §5.11 | one layer per distinct pin set; only the pinned cone | pin set, by values |
| fill | §5.12 | O(A log A) for A arcs | (fill key, supplies, demands, order) |
| prorata | §5.13 | O(n log n) once per (total, weights) | (total, weights) |
| plan move | §5.14 | O(1) an instance; one evaluation more per later version reaching it (§2.1) | instance |
| decision, keyed decision | §5.14.1–§5.14.2 | its reads at its point; the instances due on a day ordered by the places their legs give, O(k log k) for k on the day; a walk-bound def once per reader's point | walk point (§3.2) |
| `flows` | §5.14 | O(log Mₓ) by prefix sums | — |
| adjustments | §5.14.3 | one re-evaluation per later version that touches the read set (E §5 step 2) | instance |
| projection `bal(x, d) from t` | §5.14.1 | one walk of its window, event-driven; in the common case a prefix sum (§2.5) | (item, day, cut, clauses) |
| `low` | §5.14.1 | O(log n) a query in the common case; O(log n) per changed instance | segment tree per (place, holder) |
| `next` | §5.14.1 | O(log I_K) | — |
| `value` | §5.10 | O(log Mₓ + log r); with `of q`, a parts read | — |
| **parts: arrival** | §3.6, §5.14 | the arrival rule, one def cell per (movement, party) it reaches; the split O(L log L) by `rest largest` | parts per (place, unit) |
| **parts: take** | §3.6, §5.14 | a fill over (part, outflow) arcs, O(L log L) for L live labels | parts per (place, unit) |
| **parts: relabel** | §3.6, §5.14 | a take and an arrival on one place | parts per (place, unit) |
| **parts: the split** (R-114) | §3.6, §5.13 | the largest-remainder split, written once, O(n log n) | — |
| **parts: the check** (R-115) | §3.6, §5.10 | parts sum to the balance, ≥ 0 and whole steps, after every arrival, take and relabel: O(L) a booking | — |
| **parts: reads** `bal … of q`, `m.leg.amount of q` | §5.10, §5.14 | O(L) at a day; O(log Mₓ) at a past day with stored part change points | — |
| observations | §5.19 | sorted once per cut class, O(R log R); booking O(1) each | (place, day) |
| residuals | §5.18, E §6.3 | O(lines + log) from the timeline and the modelled past; no re-evaluation | — |
| random fact, sample | §5.16 | one SHA-256 and one quantile, 0.3–1 µs; discrete O(k) | (address, path) |
| path ask | §5.16, §5.18 | N × the random cone, plus O(N log N) for statistics | — |
| scores | §5.15 | O(n log n) a sample score | — |
| ask | §5.18 | its cone | result under (reality hash, request) |
| `solve` | §5.18 | ⌈log₂((hi − lo)/u)⌉ + 3 evaluations of its cone, each in a pin layer freed after its step | (bracket, unit, pins) |
| `as known` | §5.18 | one evaluation per distinct cut class reached | cut class |
| scenario | §6.4 | one candidate more per read of a target it writes | world |
| joint member | §6.1 | its own context; its walk to the last day read (§2.6) | member |

**Labels cost only where a kind opts in** (`engine/BUILD-NOTES.md`, the labels' cost; `shape.md` "What carries over" 5). A place whose kind is neither `owned` nor `batched` keeps one number per unit, with no map, no arrival rule and no take; E1 and X run 0 parts groups. The cut-K walk that labels the past (L §5.14.3) is the walk that books the balances, not a second one. At size, A runs 33 344 parts groups for 49 664 legs; the parts probe `U:set/pairs/PTY-05+OWN-12/ania` runs 18 for 18. Load row L:18 carries the labels into the budgets (`STALE.md`'s 2.1 note).

**A hole in the parts** merges only the parts an undecided take could reach into one hole part (L §5.14), so a hole costs the reach, not the place.

**An aggregate over an open index** (a def summing observed rows of one category for the year before y) costs O(R) per key, so O(R · Y) over Y years, memoised per cut class. Where its `where` compares a field of the row with a function of the def's key, the engine may group the rows once by that function, O(R + Y): an optimisation that changes no result.

---
## 8. Budgets

The rows are DONE gate 12's: every budget here is met at the size its row states, on the reference machine. Four groups (D4-6): the interactive targets kept from 1.0.1 (I), the six measured checks at 1× (C), the load rows at full size (L), and micro rows (M). The ids are fixed; `BUILD-ORDER.md` cites them.

**Every number is proposed until S**: measured at S, then fixed with the user; a too-tight number changes only through a logged row.

**The columns.**
- *refcore's figure* replaces 1.0.1's "Floor": refcore's own time, footprint or counts (`shape.md`, `engine/tests/load/`). It is an upper reference, not a target, since refcore is written for clarity (`shape.md`, Limits). 1.0.1's prototype figures are kept where 2.1 has nothing better, and marked so.
- *Wall*, *footprint* and *count* are the budget. Footprint is peak memory footprint, the watchdog's measure (R4 Q7).
- *Where and when* names the instrument's place only; the instruments, the release and profiling profiles and the recipes are BUILDER's performance section (D4-1, D4-4, D4-5).
- *Blocks* is what a miss stops (D4-2): **counts and instruction hard limits block every PR, on Linux CI; wall time and footprint block only a milestone's tag, run by the gate session on the reference machine.** Wall time on hosted CI is too noisy to gate (R4 §2 item 1).
- **The reference machine** is the user's MacBook Pro M2 Pro (32 GB, macOS 26), on AC power, idle, never during fuzzing; 2 warm-up runs, then the median of 10; a fingerprint in every report (D4-3).
- **Memory** (D4-7): 8 GB is a hard ceiling on any run; ≤ 4 GB a load row, so 6 heavy jobs fit the 32 GB machine; ≤ 512 MB a 1× check. The ceiling is the runner's, enforced as `memcap.py` enforces it on refcore; the engine itself reads nothing of the machine (E §1).

```
$ python3 -I -c "print(32/6)"
5.333333333333333
```

### 8.1 Group I: the interactive targets kept

1.0.1 §8's targets (N-2001..N-2007), kept as budgets. Their sizes were "the prototype's instances", a tally 1 notion; in 2.1's models they need T's generator (SO8-10), which scales a reality's instances and observations tenfold. Footprint ≤ 1 GB each (R4 Q6; O-W1-6). Measured on the reference machine at each milestone that builds their constructs (`BUILD-ORDER.md`).

| Id | What | Size | refcore's figure | Wall | Footprint | Count | Where and when | Blocks |
|---|---|---|---|---|---|---|---|---|
| P8-1 | **A conversation turn** (N-2001): validate a patch, run the reality and one scenario, compare, give the deterministic answer (§4.3) | the household's reality and one scenario, at T's generated size | — | **1–2 s** end to end | ≤ 1 GB | — | Mac, at the milestone tags from the one that builds amendments. **Rests on `OPERATIONS.md`, which stays stale**: the row's steps wait for its rewrite | the tag |
| P8-2 | **A deterministic run, cold** (N-2002) | 33 years daily, 10× the household's instances and observations (1.0.1: 181 440 movements, so ≤ 5.5 µs a movement all in) | — (1.0.1's prototype: 0.34 s at 1×) | **≤ 1 s** | ≤ 1 GB | days stepped ≤ the non-empty days (§2.2) | Mac, at the tags from S. A miss also decides the seam (§4, D1-5) | the tag |
| P8-3 | **A deterministic run, warm** | as P8-2, memo kept from the cold run | — | **≤ 50 ms** | ≤ 1 GB | def misses 0 | Mac, at the tags from S | the tag |
| P8-4 | **Monte Carlo** (N-2003): law on every path, after the deterministic answer or from the cache | 1 000 paths × 30 years daily, 6 workers | — (prototype: 0.45 s, no law; refcore LH-17-mc-light ≈ 0.2 s a path) | **≤ 1 s** | ≤ 1 GB | cells recomputed locally reported | Mac, at the tags from the Monte Carlo milestone | the tag |
| P8-5 | **One look-ahead decision**, in 2.1 one read of the written projection `low(x, a .. b) from t` in E §4.3's common case: one range minimum over F (§2.5, about 14 steps) | a 33-year place, n ≤ 12 054 | — (prototype: 0.1–0.3 µs, compiled) | **≤ 0.3 µs** | — | its instruction limit (P8-M4) | its micro form P8-M4 | as P8-M4 |
| P8-6 | **One path replayed** for an explanation (N-2004) | one path of P8-4's ask, shared layer cached | — (prototype ≈ 20 ms) | **≤ 20 ms** | ≤ 1 GB | — | Mac, at the tags from the Monte Carlo milestone | the tag |
| P8-7 | **The gate on one law part** (N-2007): its `check` asks and the threshold table | about 9 checks, 61 planted errors | — (prototype ≈ 0.9 s) | **≤ 1 s** | ≤ 1 GB | — | Mac. **Rests on `OPERATIONS.md` §3, which stays stale**: the row waits for its rewrite | the tag |
| P8-8 | **One fitting step** (N-2005): pin, replay, residuals (§4.4) | two years of history | — | **≤ 20 ms** | ≤ 1 GB | — | Mac, at the tags from the milestone that builds the residual ledger | the tag |
| P8-9 | **Determinism** (gate 6) | every case and T's generated realities | — | equal result hashes across runs, with W ∈ {1, 2, 3, 5, 6, 7} (§5.7), and on the Mac and Linux | — | — | Linux CI every PR; the Mac at each tag | every PR |

```
$ python3 -I -c "print(18144*10, 1/181440*1e6)"
181440 5.511463844797179
```

### 8.2 Group C: the six measured checks at 1×

The six heaviest checks `shape.md` measured on refcore, at their `checks.tsv` recipes. Each: **≤ 1 s, cold, 1 worker; ≤ 512 MB; no more def-cell misses than refcore's distinct def cells** (D4-6, D4-7). Counts and instruction counts are measured on Linux CI on every PR; wall time and footprint on the reference machine at the milestone tag, and nightly from the milestone that builds the check's constructs (`BUILD-ORDER.md`). Instruction hard limits are set at S from the first measurement, then only lowered (D4-2).

The days-stepped budget is refcore's walks × days times its non-empty share (§2.2): what the event walk steps before any fork. A partial count is a lower bound of refcore's (`shape.md`, Limits); as a budget it is stricter than refcore's true figure (O-W1-4).

| Id | Check | refcore: plain s, peak GB | Wall | Footprint | Def misses ≤ | Days stepped ≤ | Blocks |
|---|---|---|---|---|---|---|---|
| P8-10 | `I:A:a05_ania@c05-ania` | 37.7 s, 0.81 GB | ≤ 1 s | ≤ 512 MB | 150 k | 18 929 (64 × 5 477 at 5.4 %) | counts: every PR; wall, footprint: the tag |
| P8-11 | `I:E1:tx2_mtg_day` | 60.5 s, 2.17 GB | ≤ 1 s | ≤ 512 MB | 619 k | 2 498 (every day holds a decision) | as P8-10 |
| P8-12 | `I:E2:E2-01~e2_01a` | 49.1 s, 0.91 GB | ≤ 1 s | ≤ 512 MB | 161 k | 8 614 (8 × 16 070 at 6.7 %) | as P8-10 |
| P8-13 | `I:E2:E2-01~e2_01b` | 65.8 s, 1.02 GB | ≤ 1 s | ≤ 512 MB | 547 k (partial: cells, not misses) | 12 920 (≥ 12 × 16 070 at 6.7 %, partial) | as P8-10 |
| P8-14 | `I:X:replay_september@b_r1` | 52.6 s on refcore's fast layer, 4.95 GB † | ≤ 1 s | ≤ 512 MB | 2.27 M (partial) | reported: refcore's non-empty share isn't measured | as P8-10 |
| P8-15 | `U:set/pairs/PTY-05+OWN-12/ania` | 0.87 s, 0.08 GB | ≤ 1 s | ≤ 512 MB | 74 | 12 (3 × 180 at 2.2 %) | as P8-10 |

† From the counted run on refcore's fast layer, whose memo layer raises memory (`shape.md`).

```
$ python3 -I -c "print(64*5477*0.054, 8*16070*0.067, 12*16070*0.067, 540*0.022)"
18928.512 8613.52 12920.28 11.879999999999999
```

**Why ≤ 1 s** (estimate): E1's 36.7 M nodes at 10–50 ns a node are 0.4–1.8 s before any structural saving (`shape.md` "What carries over" 7; R4 Q6). E1 is the one check whose every day holds work, so it is the check most likely to need closures (D1-1, §10).

**Further counts, reported and not blocking at first:** legs booked, parts groups (A: 33 344; E1 and X: 0, §7), walks shared (§2.4), reads and distinct reads. Each becomes a blocking count budget once S has measured it.

### 8.3 Group L: the load rows at full size

One row per `engine/tests/load/load.tsv` row, the id's number being the row's `L:n`. The file has **17 rows, numbered L:1–L:18 with no L:7** (open item O-3); G1's "18 load rows" counts the numbering. These are never run on refcore; DONE gate 12 judges their full-scale-only figures here first.

**The budgets:**
- **≤ 4 GB each; 8 GB a hard ceiling** (D4-7).
- **A row that draws no paths: ≤ 10 s** for each run of its recipe (a recipe of several runs budgets each).
- **A Monte Carlo row:** its deterministic part as a row without paths (≤ 10 s), then its paths at N-2003's rate, **1 s per 1 000 paths × 30 years on 6 workers**, scaled by paths and by the years its paths walk. This split of a mixed row is W1's reading of D4-6 ("Monte Carlo at N-2003's rate"), logged as O-W1-2; the rate is P8-4's.
- **Count:** the 1× twin's counts (its capability checks in `load.tsv`), scaled, reported; not blocking.
- **Where and when:** the reference machine, nightly from the milestone that completes the row's constructs, at that milestone's exit, and at M9 (`BUILD-ORDER.md`). **Blocks:** the milestone tag.

Which rows draw paths (from `load.tsv` and `benchmarks.md`): L:1, L:5, L:11, LH-17-mc in L:12, and part of L:14. LH-15 and LH-16 draw none (no case model samples, L §5.16 cost).

The path-walk years H of L:1, L:11 and LH-17-mc are not stated in `load.tsv`; T fixes them from the recipe (O-W1-3). B's span (L:5) is 2022-03-01..2028-12-31, 2 498 days. Estimates:

```
$ python3 -I -c "
from datetime import date
b=(date(2028,12,31)-date(2022,3,1)).days+1
print(b, 3000/1000*b/10958, 3/1000*b/10958)
for H in (20,30,40): print(H, 10000/1000*H/30, 1000/1000*H/30)"
2498 0.6838839204234349 0.0006838839204234349
20 6.666666666666667 0.6666666666666666
30 10.0 1.0
40 13.333333333333334 1.3333333333333333
```

| Id | What (`load.tsv`) | Paths | refcore's figure (`benchmarks.md`) | Wall | Footprint |
|---|---|---|---|---|---|
| P8-L1 | household D, D-10 at 10 000 paths | yes | about 10–12 min an ask on snap8 | deterministic asks ≤ 10 s; each path ask 10 × H/30 s (10 s at H = 30) | ≤ 4 GB |
| P8-L2 | household D, D-11's zero-volatility pair to the dry year | no | — | ≤ 10 s | ≤ 4 GB |
| P8-L3 | household B as one run over its full span (to 2028-12-31) | no | stopped after 12 min 37 s, nothing printed | ≤ 10 s, every question together | ≤ 4 GB |
| P8-L4 | household B, B-07 on its own span (a search of 4 001 steps) | no | stopped at 83 GB after 4 635 s, nothing printed | ≤ 10 s | ≤ 4 GB; each step's pin layer freed after it (§3.5) |
| P8-L5 | household B, B-12 at 3 000 paths to 2028-12-31 | yes | about 5 min a path on snap8; `b12_liquid` at 1 path stopped at 18 GB after 1 587 s | deterministic asks ≤ 10 s; each path ask 3 × 2 498/10 958 ≈ 0.7 s | ≤ 4 GB |
| P8-L6 | household B, the balance rows past 2026-12-31 | no | — | ≤ 10 s | ≤ 4 GB |
| P8-L8 | household E1, the full joint model's walks | no | over 5 min a walk (a walk of B about 3 min) | ≤ 10 s | ≤ 4 GB |
| P8-L9 | household A, A-05's `windows each` asks for Ania on the full text | no | stopped at 330 s on snap8 and snap11 | ≤ 10 s | ≤ 4 GB |
| P8-L10 | the amendments household's long-horizon walk cells, `c1_r2`, `v5_r2` | no | — (four snap11 runs passed 8 GB, L:14) | ≤ 10 s each run | ≤ 4 GB |
| P8-L11 | household-bands at 1 000 paths, and its cut-down | yes | the 5 test-mode asks over 60 s alone | deterministic part ≤ 10 s; each path ask 1 × H/30 s (1 s at H = 30) | ≤ 4 GB |
| P8-L12 | LH-15, LH-16, LH-17-mc at their full axes, cohorts, paths and span | LH-17-mc only | — | LH-15, LH-16: ≤ 10 s each; LH-17-mc: deterministic part ≤ 10 s, each path ask at the rate for its N and H | ≤ 4 GB |
| P8-L13 | the household's `explain_room`, M2's cut-downs at full span, F2's `windows each` | no | — | ≤ 10 s each run | ≤ 4 GB |
| P8-L14 | session 4's snap11 runs past 8 GB: `b12_det`, `b12_p3`, household-bands' `cut11/mc`, `c1_r2` | `b12_p3` (3 paths), `cut11/mc` | all four stopped at 8 GB on snap14; `b12_p3` finished on snap15 | `b12_det`, `c1_r2`: ≤ 10 s; `b12_p3`: ≤ 10 s plus 3 paths at the rate (under 1 ms); `cut11/mc`: as P8-L11 | ≤ 4 GB each |
| P8-L15 | O:A-3's run: A's `q12` asks with the `one_of` forecast copy | no | stopped at 11.0 GB | ≤ 10 s | ≤ 4 GB |
| P8-L16 | household B, B-05's three asks known 2026-01-05 | no | 86.5 s, 90.6 s, 85.2 s on PyPy | ≤ 10 s | ≤ 4 GB |
| P8-L17 | household A, A-13 (b)'s 2024 difference with `windows each` | no | over 60 s for C2 | ≤ 10 s | ≤ 4 GB |
| P8-L18 | decision 6: a joint model reading one row of a member whose walk is long; the three-member E2 | no | 1× twin `e2_01a` 33.5 s on refcore; session 5: 67.7 s with the third member against 17.1 s without, 73.1 s for one row of a long-walk member (fast layer) | ≤ 10 s | ≤ 4 GB |

**L:18's count is the seam's own test** (§2.6, seam J): a member's walk stops at the last day a joint read asks of it, and the counters show that member's days stepped against its walk's length. Reported at first, a count budget once S has measured it.

### 8.4 Group M: micro rows

Fixed inputs; the micro harness on the reference machine, and instruction hard limits on Linux CI (BUILDER's performance section). **Each is set at S from its first measurement × 1.2, then only lowered by the rules role** (R4 Q6 group IV; D4-2). They block every PR on Linux by their instruction limit; their wall time is reported at each tag.

| Id | Row | Size | Budget |
|---|---|---|---|
| P8-M1 | `Num` add and multiply on the small tier | two small values, whole and short-decimal | first measurement × 1.2 |
| P8-M2 | gcd of a big and a small number | a big value at 5 049 bits and a 64-bit value | first measurement × 1.2 |
| P8-M3 | one big operation at 5 049 bits (E1's largest result): a multiply and its reduction | two values at 5 049 bits | first measurement × 1.2 |
| P8-M4 | one read of a written projection in E §4.3's common case (P8-5's micro form) | a 33-year place, n ≤ 12 054 | ≤ 0.3 µs (P8-5), and first measurement × 1.2 for its instruction limit |

R1 and R4 name no other micro row. A further one would come from S's first profile, through a logged row.

---
## 9. What can grow, and what caps it

Every quantity that can grow without bound has a cap, a refusal, or a cost the author sees. Each cap below is checked against the code and number L and E state in tally 2.1.

| Grows with | Cap, refusal or cost | Where |
|---|---|---|
| axis combinations in one ask | 64, after its `with` pins (`LOAD-FORKS`) | L §5.5 l.868; L §8.3 |
| Monte Carlo paths | 100 000 an ask (`REQ-PATHS`) | E §7, E §11 |
| the size of an exact value at a key | 4 096 digits in its reduced numerator or denominator (`RUN-DIGITS`) | L §7.1 l.2102; L §8.4 |
| **a path statistic's accumulation** | **uncapped**: the sums under a mean or a variance are the engine's, exact by any method that gives the printed rounding (2.1-15); its own type, outside the cap check (R-117); its size grows with N ≤ 100 000. refcore caps a mixed-unit or rate `mean`'s partial sums; the engine follows the spec (K8-17, §5.2) | L §5.16, §7.1 (R-13); §5.6 |
| the precision of a transcendental | 2¹⁶ bits (`RUN-UNDECIDED`) | L §7.3 |
| scenario stacks | 4 deep, acyclic (`LOAD-SCENARIO`) | L §6.4 |
| `as if` nesting | 4 deep (`LOAD-ASIF`) | L §2.3 l.325, §5.11 |
| expression nesting | 8 deep (`LOAD-DEPTH`) | L §2.3 l.325 |
| a recurrence | bounded by its index, computed iteratively in key order | E §3.2 l.155; L §5.11 |
| a law's own iteration | written as a bounded recurrence; no fixed point | E §4.2 l.219; L §5.11 |
| a `solve` | its bracket and unit fix the number of steps, ⌈log₂((hi − lo)/u)⌉ + 3 | L §5.18 cost |
| projection nesting | finite: a chain names each holder at most once | E §4.3 l.228 |
| **joint-model nesting** | **uncapped**: no limit and no code (SO8-8); walked as a worklist, never by native recursion (R-16) | E §3.1 l.118; L §6.1 |
| rolling origins | the origins index the author writes, one cut class each | E §8 |
| observations | O(R log R) per cut class; their real volume is unknown | L §5.19 cost |
| fill arcs | O(A log A); the author's arcs | L §5.12 |
| live labels at a place that keeps parts | no cap: one per live owner, batch or pair; a take or an arrival O(L log L), a check O(L); the counters show the parts groups | L §5.10 cost; §7 |
| a run's memory | **8 GB a hard ceiling on any run**, the test runner's, not a code: the engine reads nothing of the machine | D4-7; E §1 |
| workers | an argument; the CLI's default 6; never read from the machine | D3-5 |

**The out-of-reach steps.** Tally 2 has no `bound` keyword. A step no row can give is an ordinary hole with its reason (P-17, D-15): the model writes it, and it prints its question. They are rows 37, 40–43 and 49 of `BOUNDS.md`:
- the fewest transfers to settle a group: "choose the best" can't be written (L §10); a choice the model makes is an `argmin` over a finite set with a stated cost and tie-break (E §11), and a proposed settlement is checked by a `check` ask;
- "eventual" ruin over an infinite horizon;
- actuarial values that need a mortality table the case doesn't give (LH-17's step 6; LH-18's annuity values are computed exactly, `to u`);
- the log score of path forecasts (E §8 l.413, L §5.15);
- legal interpretation, advice and macro forecasts, and a library of all laws (rows 41–43): outside the engine by nature.

None of them is a slow computation: each costs nothing at run time beyond printing its hole.

---

## 10. What stays open

Each unknown of NOTE §4 that a number here rests on, with the section it moves and the milestone that measures it. All are measurements, never settled on paper.

| # | Unknown | Moves | Measured at |
|---|---|---|---|
| U-1 | nanoseconds per node of the tree-walk with exact arithmetic on the small tier (`shape.md` assumed 10–50 ns) | §1.2 item 3; P8-10..P8-15; P8-2 | S |
| U-2 | whether day-dense asks like E1 need closures (D1-1's trigger) | §1.2 item 3; P8-11 | the milestone that builds E1's constructs |
| U-3 | the read-set overhead on a cold run (1.0.1 cited 1.27× for a published provenance system) | §3.5; P8-2 | S |
| U-4 | bytes per memo cell, with and without read sets, sparse and dense | §3.5; every footprint | S |
| U-5 | how many cells remain once empty days are skipped and prefixes shared | §3.5; the C rows' def-miss budgets | S, then each milestone |
| U-6 | the checkpoint interval (monthly is a guess) | §2.3, §2.4, §4.2 | S |
| U-7 | how late A's and E2's timelines diverge: the fork's saving | §2.4; P8-10, P8-12, P8-13 | the milestone that builds forks (`BUILD-ORDER.md`) |
| U-8 | whether a cold 33-year run meets ≤ 1 s at 10×, which decides the seam (D1-5) | §4; P8-1, P8-2 | S, then each tag |
| U-9 | B-07's and B-12's peaks with per-context memos (L:4, L:5, L:14) | §3.5, §5.5; P8-L4, P8-L5, P8-L14 | the milestone that completes B, and M9 |
| U-10 | the size of one path's layer at B-12, D-10 and household-bands; decides the worker default's memory (D3-5) | §5.5; P8-L1, P8-L5, P8-L11 | the Monte Carlo milestone, and M9 |
| U-11 | the share of balanced big gcds in E1 (what Lehmer buys), and whether `Num` is 16 bytes | §6; P8-M2, P8-M3 | S |
| U-12 | how often 64-bit intermediates overflow inside one operation | §6; P8-M1 | S |
| U-13 | the 1 → 6 worker speed-up, and the cells duplicated under the shared layer's freeze | §5.3, §5.5; P8-4, the MC load rows | the Monte Carlo milestone |
| U-14 | the cost of the interval-refined Maritz–Jarrett standard error at 10 000 paths | §5.6; P8-L1 | the Monte Carlo milestone |
| U-15 | the deepest stack a case needs, which sizes the counted depth and the threads' stack | §1.2 item 4; §5.4 | S |
| U-16 | the overflow checks' cost, in the big tier's library in particular (D4-4: above 3 % there, the user decides an override for that library only) | every wall budget | S, once |
| U-17 | instruction-count stability across CI machines, and the instruction counter's slowdown on a 1 s run | §8's blocking columns | S |
| U-18 | the reference machine's thermals over a 10-run median | §8's wall columns | S |
| U-19 | the cost of law per path | §5.5; P8-4 | the Monte Carlo milestone. First remedy if over: hoist more into the shared layer |
| U-20 | the real import volume, which 10× (N-2002) stands for; the user may change it | P8-2 | T's generator, then S |
| U-21 | whether a persisted memo across sessions is worth it, beyond results stored by hash (N-2004); the design doesn't need one | §4 | after M9, if a turn misses |

**Not a budget lever:** profile-guided optimisation comes only at M9, as a release-only extra, and no budget may need it (D4-4).

**Open items this document logged** (`tmp/session8b/open.md`, for G2a): O-1 (§2.2's derived list, for review), O-2 (§3.3's derivation, for review), O-3 (the 17 load rows), and W1's own items O-W1-1 onward.
