# Tally: the engine

<!-- tally2: D-1..D-15. This file defines how tally 2 runs; each changed section names its records -->

How a Tally model runs. This document and `LANGUAGE.md` are the whole definition: a modeller writes models from them, and an evaluator works a model by hand from them. `LANGUAGE.md` says what each construct means. This document says how a run puts them together:
- in what order things are computed;
- what the past and the plan are;
- how observations are replayed;
- how paths run;
- what is checked;
- how a result is explained.

It names no technology. Costs per construct and the budgets are in `PERFORMANCE.md`, and the build milestones in `BUILD-ORDER.md`.

**Contents**
1. The run function
2. The four clocks, and the dates inside a run
3. Loading and evaluation order
4. The plan pass, the day walk and projections
5. Booking: cuts and adjustments
6. Replay against reality and the residual ledger
7. Paths
8. The evaluation layer and its scores
9. Invariants checked on every run
10. Explanation and tracing
11. What the engine refuses

---

## 1. The run function

<!-- tally2: D-2, D-6, D-8, D-10 -->

```
run(reality, request) → result

request = { asks:     the asks to answer (ids in the reality, or asks given with the request)
            clocks:   S and E are always the header's start and end (§2); K comes from each ask
            pins:     extra pins applied to every ask, as `with` }

result  = { timeline: per (cut class, world, pin set, clauses, axis options the moves reach, path): movements and end-of-day balances
            answers:  per ask, the printed result (LANGUAGE §5.18), its register (LANGUAGE §8.2) and warnings
            residuals: the residual ledger for the residual asks (§6)
            hash:     the hash of (the reality's canonical text, the request, tally version, engine build)
            version:  the tally version and the engine build that produced it (LANGUAGE §9) }
```

This is spec M-20's `run(model, history, clocks[, scenario]) → timeline`. The model and its history are one thing here, the reality: observations are rows in versions (LANGUAGE §5.19). Each ask is evaluated in its world (its `in s`, or, in a base file, the model's, LANGUAGE §5.18); the clauses are an ask line's `windows`, `random` and `with uncertain` (LANGUAGE §5.18).

**Pure.** The run reads the reality's files and the request, and nothing else: no clock, environment, locale or network. The same reality and request give the same result to the last digit (LANGUAGE §7.5).

**What a timeline holds.** For every place and every day from S to E: its movements and its end-of-day balance. Each movement is:

```
{ id, kind, day, legs: [{ name, from, to, amount ≥ 0, unit, day }], for, attributes, fulfils, status }
```
- `id` is the movement's stable id (LANGUAGE §2.5): a move instance with its keys, an observation's row, an adjustment, a pad.
- `kind` is its event kind; `for` the instance's period; each leg has its own day (LANGUAGE §5.14).
- `status` says how certain it is (N-1205):

| Status | Meaning |
|---|---|
| `observed` | it happened: an observation, booked where a statement covers its place or else from itself (§6.1) |
| `past` | a planned movement of a certain kind on a day no statement covers, before K |
| `plan` | a planned movement of a certain kind from K on: intended |
| `uncertain` | a planned movement of an uncertain kind that no observation fulfils: kept, never in the timeline's balances or `flows`; counted by a projection, a binding or a plain balance read only under `with uncertain` (LANGUAGE §5.14) |
| `adjustment` | a change a later version made to a booked plan instance (§5); it moves nothing |
| `pad` | a movement that takes up a statement's gap (§6.1) |
| `hole` | a movement whose amount, day or existence is a hole |

The **modelled past** (planned movements on covered days before K, and instances an observation the cut sees fulfils, §6.2) is computed and kept for the residual ledger, but it isn't in the timeline's balances.

**Stored as aggregates.** A result is stored under its hash. Paths are not stored; any one is recomputed on demand (§7, N-2004).

---

## 2. The four clocks, and the dates inside a run

<!-- tally2: D-3, D-13 -->

The four clocks of the spec (T-16, N-1219):

| Clock | Comes from | What it does |
|---|---|---|
| **S**, run start | the header's `start` | the first day walked. Balances open at S from `opening` readings (LANGUAGE §5.10). Nothing before S is walked, and `flows` over days before S is a hole (`HOLE-PRERUN`, LANGUAGE §8.1): `before the run's start` |
| **K**, knowledge date | the ask's `as known D`, else the model's `on` | masks every version and row known after K: law, facts, observations, readings, entries of things. It splits the past (before K) from the plan (from K) |
| **[K, E]**, evaluation window | E is the header's `end` | the plan runs to E. Observations in [K, E] are never read by the run at cut K; only the evaluation layer compares them (§8) |
| **R**, record time | the store: the reality's files as they stood at R | "what did we think in March?" is the reality at R = March, asked as known in March (N-1222). The language has no R; the store keeps the history of the files (`OPERATIONS.md` §1) |

- **A plain forecast** has K = `on`. S is the header's start. Covered days before K replay observations, and the plan starts at K.
- **A backtest** sets K in the past with `as known`. The run sees only what was known then, and the evaluation layer judges it against what happened in [K, E] (N-1218).
- **K before S** is allowed. The whole run is then plan.
- **K after E** is allowed. The whole run is then past.
- **Movements outside [S, E]** are bound and read by bindings like any other, those after E by `flows` too, and never booked; the register lists them (`left out:`, LANGUAGE §5.14.3, D-13). `flows` over days before S is a hole whatever is dated there (`HOLE-PRERUN`, above).

**Dates inside a run.** These aren't clocks of the run, but every evaluation carries them:

| Date | Of | Defined in |
|---|---|---|
| law date d | each def evaluation, each movement's `at` | LANGUAGE §4.3 |
| booking cut min(t, K) | each plan instance dated t | LANGUAGE §5.14.3, §5 below |
| walk point | the day walk: (day, stratum, movement, keys), the "now" of running reads | §4 |
| projection day t | each `bal(x, d) from t` | LANGUAGE §5.14.1, §4.3 below |
| scenario `from` D | each scenario | LANGUAGE §6.4 |

**Cut classes.** A cut matters only through the set of versions and rows it admits. Two cuts that admit the same ones give the same values. K's own date still decides what the register prints and which rows are `held` (LANGUAGE §8.2), and what is past and what is plan (`OPERATIONS.md` §7); none of these is memoised. The distinct `known` dates of a reality, its versions' and its rows' own, split time into **cut classes**, and every memo is keyed by the class, not the date.

---

## 3. Loading and evaluation order

<!-- tally2: D-1, D-2, D-5, D-9, D-11, D-12, D-15 -->

### 3.1 Loading

Loading happens once per reality and is cached under the reality's hash; load sees what LANGUAGE §8.3 lists. When it finishes, no failure of structure remains: everything left is a per-value refusal or a hole.

1. **Parse** every file, and reprint it in the canonical format (LANGUAGE §2.4). The reality's hash is the sha256 of the UTF-8 bytes of its canonical text, its files reprinted and taken in the order of their paths in its directory tree (compared by code point), so spacing never changes a hash (a semantic hash: Dhall's is of a normal form, not of the text, D-9). A joint model's `use` line pins a member by this hash, written as 64 lower-case hexadecimal digits, so a member that is itself a joint model pins its own members through its `use` lines. A joint model's imports are checked against their hashes (`LOAD-PIN`) and loaded as closed members under their prefixes (LANGUAGE §6.1): a member's items are evaluated in its own namespace, as when it loads alone, its bindings over the prelude's kinds ranging over its own things. Only the joint model's own items range over every member's things, through the prelude's kinds, which all of them share, and through members' kinds stated `same`, which are one kind; a member's other kinds stay under its prefix. A movement of the joint model's own versions into a member's place is booked there only as LANGUAGE §6.1 says.
2. **Resolve** names in the one namespace, imports' as `a.item`, keys by position, and the visibility of scenario items (LANGUAGE §2.5, §6.4).
3. **Check types and units,** the literal rule and literal precision (LANGUAGE §3, §7.1).
4. **Build the kinds.** The kind each kind is a subkind of, its subkinds, and for each thing its base kind, its dated memberships and its lifespan, from the entries in versions and from `place` sections (LANGUAGE §5.7, §5.10). An entry at a kind with subkinds is refused (`LOAD-LEAF`), one without `lives` too (`LOAD-LIFESPAN`).
5. **Build the item graph.**
   - Its nodes are inputs, bands, defs (each rule separately), fills, moves, places, the observed inputs and asks.
   - Its edges are reads. Each edge records how the reader's keys relate to the read's: the same key, an earlier or a later key behind a guard, a computed key, a fixed key, or any key under an aggregate. **A binding over a kind** is an edge to every item that gives things or movements of that kind or its subkinds: the moves of an event kind, the observed inputs, the versions' entries. A posting (a move keyed by movements of a kind, LANGUAGE §5.14) therefore depends on every movement it binds over, and nothing names them.
6. **Check the graph:**
   - same-key cycles, and cycles in `beats` (`LOAD-CYCLE`);
   - guards on recurrence reads, a def's read of itself or of anything that depends on it at k ± n (`LOAD-GUARD`, LANGUAGE §5.11); any other shifted read is checked per key (`RUN-KEY`);
   - `beats` and `replace` labels (`LOAD-LABEL`);
   - inputs, defs and places nothing reads, and sources no version cites (`WARN-UNREAD`, LANGUAGE §8.5, decided here, whatever the asks reach).
7. **Classify the moves.** A move with `reads` is a decision, every other a plan (LANGUAGE §5.14.1). Mark every running read (`bal(x)`, `bal(x, d)` at any day, a projection `from t` at any t (the engine proves no day earlier than another, so a plan that reads a balance is written as a decision), `flows` of a place or a decision, a decision's legs, day or attributes, a binding over a decision's movements or over `pads`) and propagate the mark through defs. A plan that reaches a mark, a decision that reaches a running read or a projection of a place or movement it doesn't declare, a decision that declares a read it never makes, and a decision whose day reaches a mark are refused (`LOAD-READS`). A def that reaches a mark and that a decision reaches is **walk-bound**: its cells carry the walk point (§3.2); in an ask, or a def no movement reaches, a plain `bal(x, d)` is the balance on day d at the ask's cut and a projection is its own walk (LANGUAGE §5.14.1). For each def that holds a projection, record the movements that depend on it, which its projection leaves out (LANGUAGE §5.14.1).
8. **Stratify.** Build the decision graph: an edge from m to m′ when m reads a place m′ changes (for a leg whose place is an expression, every place it can give, LANGUAGE §5.14.2) or reads m′'s legs, day or attributes, and m′ doesn't depend on m. Its strongly connected components with more than one move need a written `before` or `after` that breaks every cycle, checked where the days and places are known at load (`LOAD-ORDER`, LANGUAGE §8.3) and in the walk otherwise (`RUN-ORDER`, §4.2 step 3), where a component no written order breaks is one stratum; a `before` that contradicts a read of legs, or names a plan or an observation, is `LOAD-ORDER` (LANGUAGE §5.14.2). The strata are the layers of the resulting acyclic graph, observations and plans below every decision, but for an observation that fulfils a decision instance of its own day, which is booked at that instance's point (LANGUAGE §5.14.2, §5.19).
9. **Index the rows.** For each target, rows are grouped by the keys they state and sorted by span, readings by instant. Versions and rows are sorted by `known`, which gives the cut classes. `replace` lines are attached to the rows they replace; an overlap without `replace` is refused (`LOAD-OVERLAP`, `LOAD-REPLACE`, LANGUAGE §4.6). A `moves` line gives the moved version one governs span per cut class: before the moving version's `known`, its own; from it, the moved one. Each version's `enters` is kept for the `wording` pin (LANGUAGE §4.3, §5.3). Band tables are checked per version (`LOAD-BAND`). A version that states balance readings of a place without a gap role is refused (`LOAD-GAPROLE`).
10. **Mark the random cone.** Every item that reaches a `dist` fact in a world, on the item graph or through a place that a random movement books into, is random in that world (LANGUAGE §5.16, §8.3): a scenario's rule or row can make an item random there alone. Each ask line is checked in the worlds it is evaluated in (LANGUAGE §5.18), and in no other. An ask line whose value is random that neither applies a query nor replays a path, a query on an ask line without `paths` that LANGUAGE §5.16's table doesn't give in closed form, and a projection a decision reaches that reads a random fact after the decision's day with neither a `random` clause (on the read or the line) nor a query around the read, where load sees the projection reach the random fact after the decision's day (LANGUAGE §5.16, §8.3), are refused (`LOAD-RANDOM`; otherwise at the read, `RUN-FUTURE`).
11. **Check the asks,** each in the worlds its line is evaluated in (LANGUAGE §5.18): the axes each one reaches, statically (`LOAD-FORKS`); orientation (`LOAD-ORIENT`); ask-only functions (`LOAD-ASKONLY`); the world (`LOAD-WORLD`); the law date (`LOAD-LAWDATE`).

### 3.2 Cells and the memo

Every value is a **cell**: an item at keys, in a context.

```
cell    = (item, keys, context)
context = (law date class, cut class, axis options, pin set, world, clauses, path [, walk point] [, projection])
```

The **walk point** (day, stratum, movement, keys) is in the context only of a walk-bound def, one that reaches a running read (LANGUAGE §5.14.1). Each decision that reads it gets a cell of its own at its point of the walk, so two decisions on one day never share a value computed before the first of them booked. The **projection** (holder, day t) is in the context of a cell evaluated inside a projection's walk (§4.3), which reads facts as known on t.

- **The context is trimmed** to what the item can depend on, which the item graph tells statically:
  - an item that no random fact reaches has no path in its context, so all paths share it;
  - an item that reaches no axis has no axis options, and one that reaches no window, random fact or uncertain kind has no clauses;
  - an item that no pin reaches shares the unpinned cell. A pin set is keyed by the pins' **values**: a pin given by an expression (LANGUAGE §5.11) is evaluated first, without that `as if`, and its value enters the key, so two expressions with one value share their cells. The register of a pin's expression goes on the ask's `pinned:` entry, not on the cells (LANGUAGE §8.2);
  - an item that reads no scenario's versions shares the base's cell;
  - an item's cut class is the interval between the `known` dates of the versions and rows in its cone, so a version that the item's cone doesn't read doesn't divide its cells (`PERFORMANCE.md` §3).
- **The law date is a class.** It is the interval between consecutive boundaries of the governs spans of the rows and rules the item reaches and of the lifespans of the things at the keys it reads (LANGUAGE §4.6 step 5), with the spans in force at the cell's cut (after `moves` and `replace`). Every law date inside one interval gives the same value. A stretch of law dates over which the rows and rules the item reaches don't change is one class even where no row covers it, so a window `at` inside such a stretch is read once, at its first day inside the window, and gives one hole, which names that day (LANGUAGE §4.6 step 5). A window `at` or `on` reads one law date per class it overlaps (LANGUAGE §4.3).
- **The memo maps cells to results.** A result is a value, a hole or a refusal, with its register and its **read set**: the rows and cells it read, band rows included. The read set is what makes adjustments (§5), invalidation (`PERFORMANCE.md` §4), `inputs-of` and `readers-of` exact.
- **Evaluation is demand-driven.** A cell is computed the first time something needs it. A recurrence is computed iteratively in key order (from the last key for reads at later keys, LANGUAGE §5.11), never by recursion on a stack, so a 12 000-day index needs no deep stack.

### 3.3 The order of a run

For each distinct (cut class, world, pin set, clauses, axis options) the asks need, where the axis options are those of the axes the moves reach (an axis no move reaches doesn't split the timeline, trimmed as cells are, §3.2):
1. **The plan pass** (§4.1): every plan instance, booked, kept as modelled past, or kept as uncertain, with the adjustments later versions make to it and the movements that settle them (§5), in dependency order.
2. **The day walk** (§4.2): readings, observations, plans and decisions, day by day, from S to E.
3. **The asks:** each reads cells and the timeline.

The plan pass and the day walk run only if something the asks need reaches a place, a move or a balance. A model with no places, such as most law cases, runs only its asks. An unpinned `flows` or `bal` ask fans out over the axis options of its timelines like any value, under the cap of `LOAD-FORKS`.

---

## 4. The plan pass, the day walk and projections

<!-- tally2: D-2, D-5, D-6, D-7, D-8, D-11, D-13; C-5, C-13, M-7 -->

### 4.1 The plan pass

For each plan move m, in the dependency order of the item graph:
1. **Enumerate** its instances over its binding tuples. A binding over a kind gives the things alive on the bind's day or span, a binding over an event kind the movements dated in its span, observed and planned (LANGUAGE §5.7). Each binding's `where` is read for its tuple at the move's `at`, at the ask's cut K, as `on` is in step 2 (LANGUAGE §5.14).
2. **For each instance,** evaluate `on` at the ask's cut K, giving its day t, then `when`, the legs' amounts, days and places, and the attributes at its booking cut min(t, K) (§5, LANGUAGE §5.14.3). A hole day or `when` makes a hole movement (LANGUAGE §5.14). An instance whose `when` is true exists, whatever its amounts; an amount of 0 books nothing.
   - **A day that is a window** (LANGUAGE §4.4) keeps its interval: the movement counts in `flows(x, span)` when the window lies inside the span, is `HOLE-IMPRECISE` when the window crosses an end of the span, and makes `bal(x, d)` a hole for d inside the window, unless the ask's `windows` clause fixes it. When its law date falls to `on`, it is the window's first day when the window lies inside one law-date class; otherwise one law date per class, and one value or `HOLE-IMPRECISE`.
3. **Classify** the instance:
   - dated outside [S, E]: kept for bindings, and after E for `flows` (`flows` over days before S is `HOLE-PRERUN`, §2), never booked, listed (`left out:`);
   - of an uncertain kind with no observation fulfilling it: kept as `uncertain` (§1);
   - on a day before K where an end of a leg is a covered place, or, on any day, fulfilled by an observation the cut sees, which stands in for it (LANGUAGE §5.19): **modelled past**, kept for the ledger, never booked (§6.2);
   - otherwise booked, as `past` before K and `plan` from K.
4. **Adjustments.** Apply later versions to a booked instance (§5).

Plans read other plans through `flows(m[keys], span)`, their legs and attributes, and `next`; never a place's running balance (`LOAD-READS`). The instance graph is acyclic, by the item graph's check plus the guard on time keys. Movements are sorted by (day, stratum, move id, keys in index order, things and an open index's keys in the text order of their stable ids, LANGUAGE §2.5); the order changes no balance and no refusal, since a day's plans read nothing running and a place's sign rule is checked after them all (§4.2 step 5).

### 4.2 The day walk

The walk is a fold over days. It starts at S from the opening readings (LANGUAGE §5.10); observed rows dated before S are bound by bindings and fills and booked by none (LANGUAGE §5.19). For each day t from S to E:
0. **Readings at the start of the day.** Each `opening` reading at `t start` that the cut sees, with t before K or t = S, anchors its place, and each `stated` reading there, with t before K, pads or anchors it (LANGUAGE §5.10, §5.19; §6.1).
1. **Observations.** Book the observations dated t, if t is before K, each end by the rule of §6.1, except one that fulfils a decision instance dated t: it is that instance's movement, booked at its point in step 3, so the decisions before it that day see the balance without it. Every other observation is booked here, on its own day, one dated before or after the instance it fulfils included (LANGUAGE §5.19).
2. **Plans.** Book the plan pass's legs dated t. A leg with its own `on` is booked on its day; each leg moves its unit from its `from` to its `to`.
3. **Decisions.** Run the decision instances due on t, stratum by stratum (§3.1 step 8): inside a stratum, the instances due on t are ordered by the places their legs give on t (LANGUAGE §5.14.2): one that reads a place another changes runs after it, and two that read what each other changes run in the `before`/`after` order or the move's `ties by` (`RUN-ORDER`, `RUN-TIE` otherwise); the others are independent, and an instance whose `when` is false orders nothing. Each one:
   - evaluates its `when` and amounts with running reads at this point of the walk, its law and facts at cut min(t, K) and its projections as of day t (§4.3); its day must be precise, or the line states `windows each` (LANGUAGE §4.4), and it is `HOLE-IMPRECISE` otherwise; a walk-bound def it reads is evaluated afresh at this point (§3.2);
   - books its legs dated t at once, so the next decision sees them, and a leg with its own later `on` on its day (LANGUAGE §5.14);
   - on a covered day before K, or where an observation the cut sees fulfils it (LANGUAGE §5.19), is modelled past instead. It is computed against the booked balances, which come from the observations ("did we follow the rule?", N-1221), less its own observations (one dated before t was booked on its own day, step 1), and it isn't booked. Its own observations dated t are booked at this point, after it is computed (LANGUAGE §5.19).
   - A reading `after m`, m a decision (one booked in the day's first batch is refused, LANGUAGE §4.6), pads or anchors its place right after m is booked; where m is modelled past (a covered day before K, LANGUAGE §5.19), right after the observations that fulfil it are booked, in step 1 or at m's point here, on the day of the last of them (LANGUAGE §4.6).
4. **Close the day.** The day's readings at `t end` that the cut sees, with t before K, pad or anchor their places (§6.1). Then each place's end-of-day balance is stored as a change point. Balances are step functions, so only days with movements or readings cost anything.
5. **Check the places.**
   - The sign rule is checked at the points LANGUAGE §5.10 states, which fall in these steps so: the legs of steps 0–2 (pads at the start, observations, plans) together, against the balance they leave; then, in step 3, each decision's legs together (or the observations booked at its point), and each reading that follows one, as they are booked; then, in step 4, the closing readings together. A balance that breaks its kind's sign rule refuses the legs at the place that brought it there (`RUN-PLACE`): in steps 0–2, each leg of that batch at the place; in step 3, that decision's legs at the place, or that reading's pad; in step 4, the closing pads at the place. A leg into or out of a thing outside its lifespan is refused (`RUN-LIFESPAN`), unless its amount is 0: a leg of 0 books nothing (LANGUAGE §5.14). A refused leg isn't booked, so no later movement or decision that day sees it, and the place's balance is refused from that point, so everything that reads it later is refused too, and so is every read of the refused leg itself (`flows` of its move over a span that holds it).
   - A place whose lifespan ends on t and whose positions aren't all zero at the close is refused from t on (`RUN-LIFESPAN`).
   - An account or cash place whose end-of-day balance goes below zero prints `WARN-NEGATIVE` on the first day of each stretch.
   - Each warning is tested only where the balance is known; a hole balance prints its hole when asked.

**Holes in the walk.** A hole movement makes both ends' balances holes from the first day it may fall on (every day, for a day that is a hole and not a window, LANGUAGE §5.14), with its questions, up to the place's next reading as §6.1 says. A leg whose place is a hole books its known side, and every read of the unknown side is a hole naming it (LANGUAGE §5.14). Decisions that read a hole balance are holes in turn. Results about earlier days, and about other places, stand.

**Running reads in the walk** (LANGUAGE §5.14.1):
- **`bal(x)`** is the running balance at this point of the day.
- **`bal(x, d)`** for d < t reads the stored change points.
- **`flows(x, span)`** of a place or a decision counts what is booked up to this point when the span ends on t.
- **A decision's legs, day and attributes** are read once it has run; the stratification puts the reader after it.

**There is no fixed point and no iteration.** A decision never sees its own result or a later decision's that depends on it. The only iteration is one the law names, written as a bounded recurrence (LANGUAGE §5.11).

### 4.3 Projections

`bal(x, d) from t`, held by item P (a def, a decision's own expression, or an ask line), is computed as a walk of its own from day t to day d (LANGUAGE §5.14.1):
1. **Start** from x's balance at the start of day t in the main walk at cut K; read at a point of the walk on day t (a decision's read), from x's balance at that point, and count in step 2 only the movements not yet booked there (LANGUAGE §5.14.1).
2. **Count** every movement dated t through d whose instance doesn't depend on P, each evaluated as known on t: its keys, `when`, day and amounts at cut min(t, K). A movement dated by a window counts under `windows each`, else the read is `HOLE-IMPRECISE` (LANGUAGE §4.4). A random amount dated after t counts at the `random` clause's value, the read's or the ask line's, or under a closed-form query around the read; else a projection a decision reaches is refused, and an ask line's own projection on a path counts that path's sample (LANGUAGE §5.14.1, §5.16). Uncertain movements count only under `with uncertain`. Observations and readings dated after t are unused. Decisions in the window run in the projection's own walk, stratum by stratum, reading its balances.
3. **Leave out** the movements that depend on P (§3.1 step 7), and list those dated in the window in the register (`left out:`).

**Nesting is finite.** Inside P's projection no movement reaches P, so a decision in it whose own projection is held by Q has Q ≠ P, and a chain of nested projections names each holder at most once.

**The common case is cheap.** When the window holds no decision that the projection counts, the projection is the start balance plus a prefix sum F of the plan movements as known on t, and `low(x, a .. b) from t` is a range minimum over F. F changes only as the cut min(t, K) rises with t, so one structure is updated in place, in walk order (`PERFORMANCE.md` §2.3). On a path, a projection that reaches random facts only through its clause is the same on every path, so F is computed once, in the shared layer (§7).

**`next(K, d)`** is a search in the sorted days of the movements of kind K, planned and observed, each once (LANGUAGE §5.19), as known at the reader's cut; two on the first day found are `RUN-TIE` (LANGUAGE §5.14.1). When none is planned yet, its binding is empty at the cut and its span reaches past it, so it is a hole naming the binding (`HOLE-NOTYET`, LANGUAGE §5.14.1).

---

## 5. Booking: cuts and adjustments

<!-- tally2: D-6, D-7; C-17, LC-23 -->

The rule is LANGUAGE §5.14.3: a booked amount never changes, and a later version that changes a booked plan instance makes a readable difference, an adjustment per changed leg, which the writer settles (P-8). Operationally, for a plan instance under an ask with cut K:

0. **Date it at K.** Its `on` is evaluated at cut K, giving t. A day learned later moves the instance under later asks only.
1. **Evaluate it at its booking cut** κ₀ = min(t, K): its `when`, then its legs. The versions and rows visible are those known by κ₀, scenario versions included by their own `known` (LANGUAGE §6.4); with t a window, a version is visible when known by the window's first day and not when known after its last, and one known inside the window leaves the amounts `HOLE-IMPRECISE` where it changes them, unless the line states `windows each` (LANGUAGE §4.4).

   The result is the booked legs A₀. An instance whose `when` was false at κ₀ didn't exist then, and counts as A₀ = 0; a read of its legs (a settlement's `a.adjusts.cash.from`) gives the places of the version that made it exist, with amount 0. An A₀ that is a hole, from its amounts or its `when`, stays a hole movement until a version answers it, under cuts from that version's `known`; a version under which the `when` is false answers it with 0, and nothing is booked.
2. **Find the later versions that can change it.** This step and the next re-evaluate the instance at cuts between κ₀ and K, the later versions' `known` dates, and they measure booked **plan** legs only:
   - A modelled-past instance (§6.2), on a covered day or fulfilled by an observation at the ask's cut, is computed at its booking cut for the ledger and never measured. A `fulfils` link is judged only at the ask's cut, by the existence of the instance it names, never by its amounts, so none of the cuts here judges it (LANGUAGE §5.19).
   - A decision instance is a decision taken on its day: its law and facts are read at κ₀ and its projections as of its day (§4.3); no later version re-measures it. Its balances are the walk's at cut K, so observations and readings that a later cut sees can change what it reads (§9 invariant 5).
   - A movement whose `fulfils` names an adjustment (a settlement) isn't measured either: a later version reaches what it settles through a new adjustment of the original instance, so the instance graph stays acyclic.

   The candidates are every instance tuple of the move in the version's cone, whether or not it existed at its cut. Take the versions and rows V with κ₀ < V.known ≤ K, in `known` order; those with one `known` are one step. Only those that touch the instance's read set can change it:
   - V has a candidate row, or a `replace`, for a read in the read set;
   - V is the `v` of a rule of a def in the cone, or replaces one;
   - V enters a key into an index or kind that the cone aggregates or binds over.

   The others are skipped without evaluation.
3. **Measure each change.** For each step, re-evaluate the instance with its versions added, giving Aᵥ, and for each leg let δ = Aᵥ − A_previous. When V makes an instance exist that didn't, δ = Aᵥ; when V ends one, δ = −A_previous.
4. **Record δ ≠ 0, or a leg whose places V changes (δ may be 0),** as an **adjustment**: a movement of the prelude kind `adjustments`, dated V.known, with id `adjustments[<move>[keys].<leg>, <V>]` (LANGUAGE §5.14.3; a step of several versions makes one adjustment, which each of them that passed step 2 names, printed under the first in ASCII order), the attributes `adjusts` (the instance) and `change` = δ, and status `adjustment`. The leg is named in the id. It moves nothing. Bindings over `adjustments`, `flows` and the register read it (LANGUAGE §5.14.3).
5. **Settle it.** The movements, planned or observed, whose `fulfils` names the adjustment are its settlements. Each settlement leg counts by the way it runs against the adjusted leg's places before and after V; LANGUAGE §5.14.3 gives the signs, and the legs it refuses (`RUN-RECORD`). The **remainder** is δ less that sum, and 0 once a movement with no legs fulfils it (a waiver).
   - A remainder ≠ 0 at the cut is listed in the register of every answer that reads the adjusted leg's places (`adjustment:`, LANGUAGE §8.2).
   - An adjustment that nothing fulfils or waives at the cut is `HOLE-CHANGED`, with the `flows` and balances LANGUAGE §5.14.3 names. A later settlement answers it only under cuts from its own `known`.
6. **A booked amount never changes afterwards** (LANGUAGE §5.14.3). Every later read of the instance sees A₀ plus the settlements dated in its span.

An instance dated after K is planned, at cut K. It has no later versions to apply.

**Example (synthetic).** A move pays a fixed sum monthly through 2027. A version known 2028-03-01 governs 2027 and cuts the sum to 0:
- Each 2027 instance is booked at its own day with the old rows.
- The new version changes each 2027 instance by δ = −amount, recorded as twelve adjustments dated 2028-03-01.
- If the source waives repayment, the model writes a move of a legless kind that fulfils each adjustment; nothing is booked and the payments stand.
- If the source claws the sum back in four instalments, the model writes a move keyed by the adjustment and the instalment, whose legs run from the receiving account to the payer, each fulfilling its adjustment; an adjustment's settlements summed against its δ give the remainder still owed.
- Had it written neither, every `flows` over the receiving account from 2028-03-01 on would be a hole asking whether repayment is due.

---

## 6. Replay against reality and the residual ledger

<!-- tally2: D-3, D-8; C-11, C-26 -->

### 6.1 The past

Before K, a place's movements on a day are:
- on a **covered** day, its observations (LANGUAGE §5.19);
- on an **uncovered** day, the model's booked plans and decisions (`past`), but for the instances an observation fulfils, whose observations stand in for them where they are booked, and the observations booked there from themselves (LANGUAGE §5.19).

The balance at S comes from `opening`, and from each `stated` reading on, by its version's gap role (below). Observed rows dated before S are seen by bindings and fills and booked by none, so history before the run is given without being walked (LANGUAGE §5.19). So the past is real wherever a statement covers it, and modelled elsewhere, and every movement says which it is (N-1201).

**Booking observations** (LANGUAGE §5.19).
- Each end of an observed leg is booked at its place from the observation of the version that covers that place on that day; an end at a place no version covers is booked from the observation itself. A transfer on two statements is two rows stated `same`, one movement, which books each end once, from its own statement.
- Each observation is booked on its own day (LANGUAGE §5.19): one that fulfils a decision instance dated that day at the instance's point in the walk, after the instance is computed (§4.2 step 3); every other in the day's first batch (§4.2 step 1), one dated before or after the instance it fulfils included.
- Two rows stated `same` that disagree print `WARN-LEGS`; each end is booked as its own statement says, the difference shows in both places' ledgers, and a read of the movement away from its places is `HOLE-SAME`. Two unjoined rows of one transfer that fulfil one instance are refused (`RUN-RECORD`).
- An observation whose `fulfils` names an instance that doesn't exist, or doesn't touch its places, is refused (`RUN-RECORD`).
- An observation that fulfils an uncertain instance makes it count: the observation is booked, and the instance leaves `uncertain`.

**Readings** (LANGUAGE §5.19). A `stated[x]` reading that the cut K sees, at an instant before K, is applied at its instant in the walk (§4.2): at the start of its day, right after the movement it names, or at the close. Its **gap** is the reading less the balance the walk reached at that instant. Its version's gap role says what follows:
- **`pad x from y`:** a movement of the prelude kind `pads`, with id `pads:<version>:<x>@<instant>`, of |gap| between y and x (into x when the gap is positive), is booked at the instant, status `pad`. `flows` and bindings count it, and the register of every figure that reads it flags it (`pad:`).
- **`anchor x`:** x's balance is set to the reading. Nothing is booked, `flows` doesn't count it, and no other place changes. The gap is the reading's balance line in the residual ledger (§6.3) and an `anchor:` entry in the register of every figure that reads x's balance from the instant on (LANGUAGE §8.2).

Either way, from the instant on x's balance is the statement's, plus what is booked since; before x's first reading, it is the opening plus what is booked since.
- **Holes.** A hole movement of x dated on or before the instant, and the hole balances it made, stop there: x's balances before it stay holes with their questions, which the gap carries (a hole gap makes a pad's amount a hole too), and from the instant x's balance is known. A movement that may fall after the instant (a hole day, or a window that runs past it) still makes x's balances holes from the first day it may fall on, since the statement may or may not hold it. The other place of a hole movement is fixed only by a reading of its own.
- **Decisions.** A reading at the close of D is applied after D's decisions, so it never changes a decision dated on or before D; a decision after it reads the stated balance through the walk, under every cut that sees the row (§9 invariant 5).
- **Cuts.** A cut before the reading's `known` doesn't see it (LANGUAGE §4.6), so its walk has no pad or anchor there and its balances are the booked ones.
- **The modelled past** on covered days up to the instant is computed against the walk's balances on those days (§6.2), which a reading doesn't change: it changes only the balances after its instant.

### 6.2 The modelled past

On covered days before K, every plan and decision instance with a leg touching a covered place is computed at its booking cut and kept as modelled past (N-1202). So is every instance that an observation the cut sees fulfils, on any day: the observations stand in for it (LANGUAGE §5.19). A decision's modelled past is computed against the real balances less its own observations (§4.2 step 3). These instances are never booked, so the model's version of the past can't double the real one.

**`bal(x, d) modelled`** (LANGUAGE §5.19) is x's balance at its last reading before d (the opening, before the first), plus every booked movement of x after it through d, with the modelled-past instances in place of the observations on covered days. It is the model's own account of the days the statement covers.

Law for the past reads observations and facts, not the modelled past (N-1203):
- A def that needs what was really paid reads `flows` of a place, or `observed(m[k])`, which on covered days are the observations.
- A def that needs a payslip reads the payslip's rows.

### 6.3 The residual ledger

`ask q: residuals x span [by f]` compares, for place x over the span, the model at the ask's cut K with the observations as known on the model's `on` date. A place of unit `any` gives one ledger per unit, in the order of the units' ids. The model there is each instance as booked or modelled at its booking cut (§6.2), plus the settlements dated in the span: the plan as it stood on that day.
- When K = `on`, this is the replay.
- When K is earlier, it is the backtest of the forecast over [K, E] (§8).
- A day no statement covers for x prints no line, since nothing states x's observations there complete (LANGUAGE §5.19); an instance dated there that an observation fulfils is compared all the same, its matched line on the instance's day.
- A pair of rows stated `same` is one movement (LANGUAGE §5.19): the ledger lists it once, on one line, and names both rows as its evidence.

Each line of the ledger is one of:

| Line | When | Shows |
|---|---|---|
| **matched** | an instance that one or more observations fulfil, unless it is partly fulfilled; an adjustment that observations settle, against its change δ on V.known | the amount difference (observed − model) and the timing difference (observation day − instance day); for a converted movement, the rate and both amounts |
| **expected but missing** | a model instance in the span that no observation fulfils (an `open:` instance, LANGUAGE §5.19) | the instance's amounts and day |
| **unexplained** | an observation that fulfils nothing, or whose instance or adjustment the model at K doesn't hold (one that only a version known after K makes: the forecast didn't expect it) | the observation |
| **partly fulfilled** | an instance of an uncertain kind whose observations sum to less than its amount | the amount difference (observed − model, as a matched line's), the remainder and a question: `when does the rest arrive, or is it written off?` (N-1206) |
| **balance** | every `stated` reading of x, at its instant, whatever its gap (0.00 included; a hole when the booked balance is one), marked `padded` or `anchored` | the gap, stated − booked: a gap or a duplicate in the import, or, where uncovered days lie between the last reading and this one, the modelled past of those days against the real one. Each reading gives its own line, so drift points at its period; the pad or the anchor then makes x's balance the statement's, so the gap isn't carried into later lines |

- **Grouping.** Lines are grouped by month and by move id (an adjustment's line by the id of the move it adjusts), or with `by f` by a field of the observed input (a category, a counterparty). A line's month is its own day's: the instance's day for a matched, expected-but-missing or partly fulfilled line (an adjustment's, V.known; for an instance whose day is a hole or a window not inside one month, the day of the last observation that fulfils it, `observed(m[k]).day`, as an observation answers a hole movement's day, LANGUAGE §5.19), the observation's day for an unexplained one, the reading's for a balance line (a reading `after m` stands on the day of its instant, right after m or the observations that stand in for m, §4.2 step 3). An instance that nothing fulfils whose day is a hole, or a window not inside one month, prints an expected-but-missing line in a group of its own named `?`, first among the groups, with its day's questions; it counts in no month's totals. Unexplained observations form one group per month named `unexplained`; with `by f`, they go by the field's value, and so does a matched line whose observations agree on it; a line with no observation, or whose observations disagree, goes by its move id, in a group printed `move <id>`. Balance lines form one group per month named `balance`.
- **Zero lines.** A matched line and a balance line print whatever their difference, 0.00 included. An instance of amount 0 with no observation moves nothing and prints no line, though it exists (LANGUAGE §5.14).
- **Signs.** Every amount is signed as a flow into x: in +, out −. A matched or partly fulfilled line's difference is observed − model; an expected but missing line's is − model; an unexplained line's is the observation; a balance line's is stated − booked. A hole on either side makes the difference a hole; a hole day or amount of an instance that observations fulfil is answered by them (LANGUAGE §5.19).
- **Totals.** Each group gives the model's total, the observations' total and the difference, so the prototype's "pay in December 2025: model 13 611.54, bank 16 611.54" is one group line. A balance group gives the booked balances as its model total and the stated ones as its observed total.
- **Order.** Groups are sorted by the absolute value of their total difference, and lines in a group by their own, biggest first (N-1223). Holes come first, since a hole may be any size, among themselves by their first question (LANGUAGE §8.1). Remaining ties go by month, then group name, then the movement order of §4.1.
- **Cost.** It is computed from the timeline and the modelled past, with no re-evaluation.

### 6.4 The fitting loop

Claude fits a parameter by replaying history many times (N-2005). Each step is:
1. pin one input with `with`;
2. replay;
3. read the residuals.

The pin's cone is the only thing recomputed:
- cells outside it keep their memo;
- the observations' booking is shared, since observations are inputs that no pin reaches;
- the day walk restarts at the first day whose movements changed.

`PERFORMANCE.md` §4 gives the cost.

---

## 7. Paths

<!-- tally2: D-11; C-10, SF-21, SF-28 -->

An ask with `paths N seed S` runs N paths (N ≤ 100 000, `REQ-PATHS`).

1. **The shared part.** Cells that no random fact reaches are computed once and shared by every path: the brackets, the indexation, every input, every plan that no random fact reaches, and the projections that reach random facts only through their `random` clause (§4.3).
2. **Per path p,** in its own memo layer over the shared one:
   - each random fact's sample is the value at its address (LANGUAGE §5.16);
   - the random-dependent plan instances are recomputed;
   - the day walk runs if any balance or decision depends on a sample;
   - the ask's per-path terms are evaluated.

   **Law per path** is just this: law defs in the random cone are cells of the path's layer (R-20).
3. **Keep the terms, drop the path.** Each path's ask terms are kept (N values per term), with the `where` of each query; its timeline is dropped. A path is recomputed alone on demand, because it is a pure function of its seed and number (`path p`, N-1406).
4. **Statistics.** `share`, `mean` and `quantile`, the standard errors of `share`, `mean` and `quantile`, the count of paths a `where` leaves, and the horizon, are computed over the kept terms as LANGUAGE §5.16 says. There is no verdict (D-11).
5. **Workers.** Paths are split into contiguous blocks, one per worker, and merged in path order. The number of workers changes nothing (LANGUAGE §7.5).

**Closed forms.** An ask without `paths` whose queries LANGUAGE §5.16's table gives in closed form runs no path: the query reads the distribution's own functions, exact or correctly rounded.

**Scenarios and `as if` share paths.** Addresses don't name the world or the pins, so a scenario and its base see the same uniforms on every path, and their difference has low variance (common random numbers, N-1302).

**Lifetimes** (CHG-08) need nothing special:
- an age at death is a random fact per person, `input death_age[p in persons]: dist year`;
- `alive[p, m]` is a def that compares it with the person's age;
- the instances that end at death have `when alive[p, m]`.

The address is keyed by the person, so it is the same in every scenario. The person's lifespan stays a fact: a thing's existence is never sampled (LANGUAGE §10).

**By hand.** Sample pins make a path exact (LANGUAGE §5.16, §7.4). A pinned path is a plain deterministic run.

---

## 8. The evaluation layer and its scores

<!-- tally2: D-11, D-15; C-10, LC-28 -->

The evaluation layer judges forecasts against what happened. It is not a separate mode. It is a set of asks that evaluate a forecast inside `as if {known K}` and the outcome outside it:

```
ask crps_sep: crps(bal(wallet, 2026-09-30) as if {known 2026-03-31}, bal(wallet, 2026-09-30)) law 2026-09-30 paths 1000 seed 7 orient lower
```

- Inside `as if {known K}`, versions and rows known after K are masked, observations in [K, E] included. **The evaluation window can't leak into the forecast;** the language enforces it, not a convention (T-16).
- Outside, the outcome is read at the ask's own cut (the run's stated cut, `on`, or the one its `as known` states), from the observations.
- **Rolling origins** are an index of cut dates. The pin takes an expression, `as if {known origin_day[o]}`, and an aggregate runs over the origins. Each origin is one cut class (`PERFORMANCE.md` §3).

**Scores** are the built-ins and expressions of LANGUAGE §5.15, each ask stating its direction as LANGUAGE §5.15 requires (`orient`):
- the CRPS of samples, path values included, is exact, and that of normal forecasts closed-form;
- interval scores, weighted interval scores and skill scores are expressions;
- coverage is judged against exact binomial ranges, through `binomial`'s exact `cdf` and `quantile`;
- actual-to-expected ratios are counts and amounts with their ranges (CHG-10).

**Oracle tests** for path statistics (CHG-13) are `check` asks:
- the statistic is an ask term, written inline in the `check`: n/N for a `share`, the value for a `mean` or a `quantile`;
- its standard error, the oracle and the tolerance (4·SE plus the step bias plus the oracle's own error) are each a def or a row, with the standard error taken at the oracle. The **step bias** is a figure, stated by the model, on how far the scheme's time step moves the statistic's mean from the oracle;
- a parameter's sample pinned for the test is listed in the register (`pinned sample:`), so a forecast can't carry it unseen.

**What the layer can't do:** the log score of paths, which the model writes as a hole with its reason (LANGUAGE §5.15).

**When "n in N futures" may be said** (spec Q-03). Tally 1's engine printed a verdict under every `share`. Tally 2 prints the figures a verdict needs, "n of N", the standard error and the horizon, and no verdict of its own (D-11, LC-28): whether a frequency may be said is a rule the household or its reader writes over those figures, as a `check` whose ask term is the statistic, its bound a def or a row (a statistic over paths stands only in an ask, LANGUAGE §5.16), and the MCP layer words its result. Tally 1's rule needed every random value's planning figure for its carry test, and the planning figure is now a question asked at a read (a `random` clause, or a `mean` or `quantile` in closed form, LANGUAGE §5.16), not a property of the fact, so the engine can't compute that test without the model saying which question it means. The precision part of the old rule (Agresti–Coull, two standard errors at most 0.05) is an expression over n and N that any model can write.

---

## 9. Invariants checked on every run

<!-- tally2: D-1, D-6, D-8, D-10 -->

These hold by construction. The engine checks them anyway, cheaply, on every run, because a failure means an engine bug. A failure refuses the whole result with `ENGINE-INVARIANT`, the invariant and the first day and place where it fails. It is never printed as an answer.

1. **Conservation.** For each unit, on each day, the two ends of each leg booked whole sum to zero, outside places and pads included; a leg booked by its ends (an observation covering one end only, `WARN-LEGS`, LANGUAGE §5.19) is left out, and its unmatched end is what the warning prints. So the balances sum to the openings plus the anchors' gaps (§6.1) wherever these are known; a place with no `opening` reading breaks nothing.
2. **Net worth moves only at the boundary, with a price, or at an anchor.** For the set of all places that aren't outside, the day's change in total worth equals the movements across the boundary (pads from an outside place included) plus the revaluation of positions that day (spec §3.2), plus that day's anchor gaps, stated − booked (§6.1), which the ledger shows as balance lines.
3. **A scenario changes nothing before its `from` that it doesn't list.** Every movement and balance before D equals the base run's, except the movements that read the scenario's facts (a decision's projection over a scenario row known by its day, a read at a law date from D, `lifespan(k)` of a thing whose lifespan the scenario changes, a binding over a span that reaches D; LANGUAGE §6.4), which the engine lists with the result, and what follows from them (LANGUAGE §6.4).
4. **Asks move no money.** The timeline's hash is the same before and after the asks are answered. Moves inside `as if` are computed and never booked.
5. **A booked plan amount never changes.** Each plan instance's A₀ is the same under every K class that books it at the same cut, and its amount at cut K is A₀ plus its adjustments (§5). A decision's amount isn't fixed that way: it reads the walk of its cut K, so observations and readings that a later cut sees can change what it reads and books (§4.2, §6.1), while the law, facts and projections it reads stay those of its day (§5, §4.3); a reading never changes a decision dated on or before its instant.
6. **Things hold nothing outside their lifespans.** Every position of a place is zero before its lifespan starts and after it ends (LANGUAGE §5.7).

The test suite adds **determinism**: every case is run twice, with 1 and with 6 workers, and the result hashes must be equal (R-16, N-1401, spec §7).

---

## 10. Explanation and tracing

<!-- tally2: D-5, D-6, D-11, D-15 -->

**`explain ref`** prints the derivation in the order of LANGUAGE §5.18 (N-1701):

| Of | It prints |
|---|---|
| a def cell | every rule of the def with its label and state: the winner with its `v` version and source, the rules beaten and what beat them, the rules not applicable, the rules of unknown applicability (a beaten rule's value is never computed, so none is printed); under the winner its causes, depth first in the order written; every row read, with version, source, locator, governs span or instant, and known date; the law date and the cut; the register |
| a `min`, `max`, `argmin` or `argmax` | its first winning term in binding order (LANGUAGE §2.5), explained as that term is (LANGUAGE §5.18) |
| a movement instance | its class (plan or decision), its day, booking cut and leg derivations; for a decision, its stratum and what it read at its walk point; each later version with its δ, the adjustment it made and what settles or waives it |
| a balance | its last reading with its gap role, version, source and gap (§6.1, §6.3), or the opening, and every movement since, up to that day, grouped by day, with each movement's status and id |
| a projection | its holder and day t, the start balance, the movements it counted with the clauses applied, and the movements it left out (§4.3) |
| a path value | the seed and path number, the samples it reached in canonical order, and the derivation on that path |
| a fill take | the fill's `v` version if it has one, the arcs in order, the remaining supply and demand before and after each take |
| a walk-bound def | the cell at each walk point where a decision read it, in walk order, each with its point (day, stratum, movement, keys) and explained as a def cell (above), its running reads as balances up to that point; with none, that nothing read it at those keys |
| an `as if` | the pins with their values (a pin given by an expression, with its derivation outside the `as if`), and the derivation inside them beside the one outside |

**Tracing isn't recorded during bulk runs.**
- `explain` re-evaluates the cell's cone with tracing on. The run is deterministic, so the traced value equals the stored one, and the memo supplies the cells it read.
- A path is replayed alone (N-2004).

**Long derivations fold.** A recurrence over 300 months prints its first and last steps and a count. Any step can be asked for with `explain x[k]`. A cause met again prints once and is referenced by its line.

**`inputs-of ref`** is the transitive read set: every row the evaluation read, the rows an `at` reads included, with its source (N-1705). A rule's `v` version isn't a row; `explain` shows it. Across an axis it fans out, one set per line.

**`readers-of x`** is the inverse of the item graph (§3.1), static: every item with a path of edges to x, the direct readers first. Of an event kind, every binding over it or over a kind it is a subkind of.

**The register** (LANGUAGE §8.2) is computed with each value, as the union of the registers of what it read. Sets are shared, so the union is cheap. A line's register is the union of its expressions'.

**Words for people.** The engine returns structured results: codes, values with units, source tags and the holes' questions, which are Polish in the household's model. A ready Polish sentence per result (spec §6.2, Q-04) is made by the MCP layer from these; the engine has no prose.

---

## 11. What the engine refuses

<!-- tally2: D-15 -->

| Refusal | When | What happens |
|---|---|---|
| **Load refusals** (LANGUAGE §8.3) | the reality doesn't load | nothing runs. A broken reality never runs in part. `OPERATIONS.md` refuses an amendment that would break loading, before it is accepted. A load error in a scenario file refuses only that scenario and the scenarios over it, and an ask's `LOAD-SCENARIO` for a scenario whose file isn't loaded refuses only that ask (LANGUAGE §6.4) |
| **Per-value refusals** (LANGUAGE §8.4) | a value can't be computed | printed at its key; it propagates and outranks holes |
| **Holes** (LANGUAGE §8.1) | a fact is missing | printed with their questions; other results stand |
| `REQ-ASK` | the request names an ask that doesn't exist (an ask given with the request that doesn't load is refused with its own load code, LANGUAGE §8.3, as `OPERATIONS.md` §8 says) | the request is refused |
| `REQ-CLOCK` | `as known` after `on` | the request is refused |
| `REQ-PATHS` | more than 100 000 paths for one ask | the request is refused |
| `ENGINE-INVARIANT` | §9 fails | the whole result is refused, with the evidence |

**What the engine never does:**
- choose an option for the household by a measure it wasn't given; a choice is an `argmin` over a finite set with a stated cost and tie-break, written by the household or the law, and a root is a `solve` (LANGUAGE §10);
- read a clock, or anything outside the reality and the request;
- write to the reality (only `OPERATIONS.md` writes: what the household confirms, statement imports and the store's advances, `OPERATIONS.md` §1);
- approximate without a stated precision;
- infer a link between an observation and a plan, or an identity between two things (LANGUAGE §5.19, §6.1);
- book a question's moves.
