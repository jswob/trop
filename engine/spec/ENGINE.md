# Tally: the engine

How a Tally model runs. This document and `LANGUAGE.md` are the whole definition: a modeller writes models from them, and an evaluator works a model by hand from them. `LANGUAGE.md` says what each construct means. This document says how a run puts them together:
- in what order things are computed;
- what the past and the plan are;
- how records are replayed;
- how paths run;
- what is checked;
- how a result is explained.

It names no technology. Costs per construct and the budgets are in `PERFORMANCE.md`, and the build milestones in `BUILD-ORDER.md`.

**Contents**
1. The run function
2. The four clocks, and the dates inside a run
3. Loading and evaluation order
4. The flow pass and the day walk
5. Booking: cuts, corrections and back pay
6. Replay against reality and the residual ledger
7. Monte Carlo
8. The evaluation layer and its scores
9. Invariants checked on every run
10. Explanation and tracing
11. What the engine refuses

---

## 1. The run function

```
run(reality, request) → result

request = { asks:     the asks to answer (ids in the reality, or asks given with the request)
            scenario: a scenario id, or none
            clocks:   S (default: the header's start; a request may set a later one, §2); E is always the header's end; K comes from each ask
            pins:     extra pins applied to every ask, as `with` }

result  = { timeline: per (cut class, scenario, pin set, axis options the pays reach, path): movements and end-of-day balances
            answers:  per ask, the printed result (LANGUAGE §5.16), its register (§8.2) and warnings
            residuals: the residual ledger for the residual asks (§6)
            hash:     the hash of (the reality's canonical text, the request, tally version, engine build)
            version:  the tally version and the engine build that produced it (LANGUAGE §9) }
```

This is spec M-20's `run(model, history, clocks[, scenario]) → timeline`. The model and its history are one thing here, the reality: records are rows in versions (LANGUAGE §5.18).

**Pure.** The run reads the reality's files and the request, and nothing else: no clock, environment, locale or network. The same reality and request give the same result to the last digit (LANGUAGE §7.5).

**What a timeline holds.** For every place and every day from S to E: its movements and its end-of-day balance. Each movement is:

```
{ date, from, to, amount ≥ 0, source, for, kind }
```
- `source` is a pay instance with its keys, a record id, or a version (for a correction).
- `for` is the instance's period.
- `kind` says how certain it is (N-1205):

| Kind | Meaning |
|---|---|
| `record` | it happened: a record on a covered day (§6) |
| `past` | the model's movement on a day no record covers, before K |
| `plan` | the model's movement from K on: intended |
| `promised` | a promised pay's movement: expected; counted by `bal`, and left out of `plan_low` and `free` until a record confirms it (both take a booked one back out of the balance they start from) |
| `correction` | back pay or a refund from a later version (§5) |
| `hole` | a movement whose amount, date or existence is a hole |

The **modelled past** (pays on covered days before K) is computed and kept for the residual ledger, but it isn't in the timeline's balances.

**Stored as aggregates.** A result is stored under its hash. Paths are not stored; any one is recomputed on demand (§7, N-2004).

---

## 2. The four clocks, and the dates inside a run

The four clocks of the spec (T-16, N-1219):

| Clock | Comes from | What it does |
|---|---|---|
| **S**, run start | the header's `start`; a request may set a later one | the first day walked. Balances open at S from `opening` rows (LANGUAGE §5.9). Nothing before S is walked, and `flows` over days before S is a hole (`HOLE-MISSING`, LANGUAGE §8.1): `before the run's start` |
| **K**, knowledge date | the ask's `as known D`, else the model's `on` | masks every version known after K: law, facts, records, valuations, index additions. It splits the past (before K) from the plan (from K) |
| **(K, E]**, evaluation window | E is the header's `end` | the plan runs to E. Records in (K, E] are never read by the run at cut K; only the evaluation layer compares them (§8) |
| **R**, record time | the store: the reality's files as they stood at R | "what did we think in March?" is the reality at R = March, asked as known in March (N-1222). The language has no R; the store keeps the history of the files (`OPERATIONS.md` §1) |

- **A plain forecast** has K = `on`. S is the header's start. Covered days before K replay records, and the plan starts at K.
- **A backtest** sets K in the past with `as known`. The run sees only what was known then, and the evaluation layer judges it against what happened in (K, E] (N-1218).
- **K before S** is allowed. The whole run is then plan.
- **K after E** is allowed. The whole run is then past.

**Dates inside a run.** These aren't clocks of the run, but every evaluation carries them:

| Date | Of | Defined in |
|---|---|---|
| law date d | each def evaluation | LANGUAGE §4.3 |
| booking cut min(t, K) | each pay instance dated t | LANGUAGE §5.13.3, §5 below |
| walk day | the day walk; the "now" of state reads | §4 |
| scenario `from` D | each scenario; visibility by date | LANGUAGE §6.4 |

**Cut classes.** A cut matters only through the set of versions it admits. Two cuts that admit the same versions give the same values. K's own date still decides what the register prints and which rows are `held` (LANGUAGE §8.2), and what is past and what is plan (`OPERATIONS.md` §7); none of these is memoized. The distinct `known` dates of a reality split time into **cut classes**, and every memo is keyed by the class, not the date.

---

## 3. Loading and evaluation order

### 3.1 Loading

Loading happens once per reality and is cached under the reality's hash. When it finishes, no failure of structure remains: everything left is a per-value refusal or a hole.

1. **Parse** every file, and reprint it in the canonical format (LANGUAGE §2.4). The reality's hash is over the canonical text, so spacing never changes a hash.
2. **Resolve** names in the one namespace, keys by position, and the visibility of scenario items (LANGUAGE §2.5, §6.4).
3. **Check types and units,** the literal rule and literal precision (LANGUAGE §3, §7.1).
4. **Build the item graph.**
   - Its nodes are inputs, bands, defs (each rule separately), fills, draws, pays, places and asks.
   - Its edges are reads. Each edge records how the reader's keys relate to the read's: the same key, an earlier key behind a guard, a fixed key, or any key under an aggregate.
5. **Check the graph:**
   - same-key cycles (`LOAD-CYCLE`);
   - guards on recurrence reads, a def's read of itself or of anything that depends on it at k ± n (`LOAD-GUARD`, LANGUAGE §5.10); any other shifted read is checked per key (`RUN-KEY`);
   - `excepts` labels and cycles (`LOAD-EXCEPTS`);
   - inputs, defs and places nothing reads, and sources no version cites (`WARN-UNREAD`, LANGUAGE §8.5, decided here, whatever the asks reach).
6. **Infer state pays.** Mark every state read (LANGUAGE §5.13.1) and propagate the mark to every def that reaches it. A pay whose amount, date or `when` reaches a mark is a state pay (LANGUAGE §5.13.1); a pay that reaches one anywhere else (its `for`, its places' keys, its bindings' `where`) is a flow pay that makes a state read, refused (`LOAD-STATE`). A def no pay reaches may hold only the dated forms (`bal(x, d)`, `worth(x, d)`, `flows`); `bal(x)`, `free(x)` and `plan_low` anywhere but under a state pay are `LOAD-STATE`. A def that reaches a state read is marked **walk-bound**: its cells carry the walk point (§3.2).
7. **Index the rows.** For each target, rows are grouped by the keys they state and sorted by span. Versions are sorted by `known`, which gives the cut classes. A `moves` line gives the moved version one governs span per cut class: before the moving version's `known`, its own; from it, the moved one. Each version's `enters` is kept for the `wording` pin (LANGUAGE §4.3, §5.3). Band tables are checked per version (`LOAD-BAND`), and ties are found (`LOAD-OVERLAP`), across groups too: rows that state as many keys, but different ones, tie where their keys meet (`x[a, *]` and `x[*, b]` at `(a, b)`, LANGUAGE §4.6).
8. **Check the asks:** the axes each one reaches, statically (`LOAD-FORKS`); orientation (`LOAD-ORIENT`); bounds (`LOAD-BOUND`); ask-only functions (`LOAD-ASKONLY`).

### 3.2 Cells and the memo

Every value is a **cell**: an item at keys, in a context.

```
cell    = (item, keys, context)
context = (law date class, cut class, axis options, pin set, scenario, path [, walk point])
```

The **walk point** (day, phase, pay, keys) is in the context only of a walk-bound def, one that reaches a state read (LANGUAGE §5.13.1). Each state pay that reads it gets a cell of its own at its point of the walk, so two state pays on one day never share a value computed before the first of them booked.

- **The context is trimmed** to what the item can depend on, which the item graph tells statically:
  - an item that no draw reaches has no path in its context, so all paths share it;
  - an item that reaches no axis has no axis options;
  - an item that no pin reaches shares the unpinned cell. A pin set is keyed by the pins' **values**: a pin given by an expression (LANGUAGE §5.10) is evaluated first, without that `as if`, and its value enters the key, so two expressions with one value share their cells. The register of a pin's expression goes on the ask's `pinned:` entry, not on the cells (LANGUAGE §8.2);
  - an item's cut class is the interval between the `known` dates of the versions in its cone, so a version that the item's cone doesn't read doesn't split its cells (`PERFORMANCE.md` §3).
- **The law date is a class.** It is the interval between consecutive governs boundaries of the rows and rules the item reaches, with the spans in force at the cell's cut (after `moves`). Every law date inside one interval gives the same value. An imprecise `at` or `on` reads one law date per class its period overlaps (LANGUAGE §4.3).
- **The memo maps cells to results.** A result is a value, a hole, a bound or a refusal, with its register and its **read set**: the rows and cells it read, band rows included. The read set is what makes back pay (§5), invalidation (`PERFORMANCE.md` §4) and `inputs-of` exact.
- **Evaluation is demand-driven.** A cell is computed the first time something needs it. A recurrence is computed iteratively in key order, never by recursion on a stack, so a 12 000-day index needs no deep stack.

### 3.3 The order of a run

For each distinct (cut class, scenario, pin set, axis options) the asks need, where the axis options are those of the axes the pays reach (an axis no pay reaches doesn't split the timeline, trimmed as cells are, §3.2):
1. **The flow pass** (§4.1): every flow pay instance in [S, E], booked or kept as modelled past.
2. **The day walk** (§4.2): records, flow movements and state pays, day by day, from S to E.
3. **The asks:** each reads cells and the timeline.

The flow pass and the day walk run only if something the asks need reaches a place, a pay or a balance. A model with no places, such as most law cases, runs only its asks. An unpinned `flows` or `bal` ask fans out over the axis options of its timelines like any value, under the cap of `LOAD-FORKS`.

---

## 4. The flow pass and the day walk

### 4.1 The flow pass

For each flow pay p:
1. **Enumerate** its instances over its binding tuples.
2. **For each instance,** evaluate `on` at the ask's cut K, then `when` and `amount` at its booking cut min(t, K) (§5, LANGUAGE §5.13.3). An instance dated outside [S, E] isn't part of the run. A hole date or `when` makes a hole movement (LANGUAGE §5.13). An instance whose `when` is true exists, whatever its amount; an amount of 0 books nothing.
   - **An imprecise `on`** (a month or a year) keeps its interval (LANGUAGE §4.4). The movement is booked as an interval movement over its period: it counts in `flows(x, span)` when the period lies inside the span, is a `HOLE-IMPRECISE` when the period crosses an end of the span, and makes `bal(x, d)` a hole for d inside the period. When its law date falls to `on` (no time key and no `for`, LANGUAGE §4.3), it is the period's first day when the period lies inside one law-date class; otherwise one law date per class, and one value or `HOLE-IMPRECISE`.
3. **Classify** the instance:
   - on a day before K where either end is covered by records, it is **modelled past**: kept for the ledger, never booked (§6);
   - otherwise it is booked, as `past` before K and `plan` from K, or as `promised`.
4. **Corrections.** Apply later versions to the instance (§5).

Flow pays may read other flow pays through `flows(p, span)` and `next_on`. The instance graph is acyclic, by the item graph's check plus the guard on time keys. Movements are sorted by (date, pay id, keys in index order).

### 4.2 The day walk

The walk is a fold over days. For each day t from S to E:
1. **Records.** Book the records of every covered place dated t, if t is before K (§6), except those that explain a state pay instance dated t or later: each of those is booked at its instance's point in step 3 (one dated after its instance is booked on its own day, here), so the state pays before it see the balance without it, and a modelled-past state pay never sees its own record.
2. **Flow movements.** Book the flow pass's movements dated t, in pay id order.
3. **State pays.** Run the state pay instances due on t in order of phase, then pay id, then keys in index order (LANGUAGE §5.13.2). Each one:
   - evaluates its `when`, amount and date with state reads at this point of the day; its date must be precise (`HOLE-IMPRECISE` otherwise), and a walk-bound def it reads is evaluated afresh at this point (§3.2);
   - books its movement at once, so the next state pay sees it;
   - on a covered day before K, is modelled past instead. It is computed against the booked balances, which come from the records ("did we follow the rule?", N-1221), and it isn't booked. Its own record, if it has one, is booked at this point, after it is computed.
4. **Close the day.** First the day's **anchors** (§6.1): if t is before K and the cut sees a `stated[x]` row governing t as its last day, x's end-of-day balance is the stated value, and the difference from the balance booked so far goes to the ledger (§6.3). Then each place's end-of-day balance is stored as a change point. Balances are step functions, so only days with movements or anchors cost anything.
5. **Check the places** (LANGUAGE §5.9):
   - Each movement is checked as steps 1-3 book it, against the balance at that point of the day. One that takes a debt above 0, a claim below 0, or a holding below 0 units unless the holding is short (`short[x]`, LANGUAGE §5.9), is refused (`RUN-PLACE`). The movement isn't booked, so no later movement or state pay that day sees it, and the place's balance is refused from that point, so everything that reads it later is refused too.
   - An account or cash place whose end-of-day balance goes below zero prints `WARN-NEGATIVE` on the first day of each stretch. The order of movements within a day is the engine's, so a balance between two of them isn't tested.
   - A free balance below zero on a day whose hold is above 0 prints `WARN-HELD`.
   - Each warning is tested only where the balance is known; a hole balance prints its hole when asked.

**Holes in the walk.** A hole movement makes both ends' balances holes from its day on, with its questions, up to the place's next anchor (§6.1). State pays that read them are holes in turn. Results about earlier days, and about other places, stand.

**State reads in the walk** (LANGUAGE §5.13.1):
- **`bal(x)`** is the running balance at this point of the day.
- **`bal(x, d)`** for d < t reads the stored change points.
- **`plan_low(x, a, b)`** = bal(x) − P(t) + min over days u in [a, b] of (F(u) − F(t) − h(u)), where F is the cumulative sum of x's planned flow movements, P(t) the promised movements of x booked so far that no record confirms (LANGUAGE §5.13), and h(u) x's hold on day u (0 on a place with no hold).
  - The planned flow movements are the flow pass's instances as known on day t: their keys (index additions, and open-index keys known by then), then each instance's `when`, then its date and its amount, all at cut min(t, K), whether booked or modelled past. This differs from booking, which dates an instance at K (§4.1): the look-ahead is what was known on day t. A `when` that is false drops the instance, its date unread; then a date known to fall outside (t, b], t the walk's day, drops it, its amount unread, whatever its `when` (such a date can't move the balance on the window). A hole in the `when`, the date or the amount of an instance that is left makes `plan_low` a hole, with the questions of every hole so read (LANGUAGE §8.1): a hole date may fall anywhere in the window. Records and anchors after t are never used, nor state pays or promised pays.
  - On a place with a hold, the hold on each day of [a, b] is read at the same cut; a hole there makes `plan_low` a hole.
  - On a path, each future instance is counted with every draw at its plan value (LANGUAGE §5.14), so the look-ahead is the same on every path and never sees the path's drawn future. F is therefore computed once, in the shared layer (§7).
  - A range-minimum structure over F answers each query in O(log n). F changes only as the cut min(t, K) rises with t, so one structure is updated in place, in walk order (`PERFORMANCE.md` §2.3).
- **`next_on(p[keys], d)`** is a search in the sorted dates of p's instances as known on day t.

**There is no fixed point and no iteration.** A state pay never sees its own result or a later state pay's. The only iteration is one the law names, written as a bounded recurrence (LANGUAGE §5.10).

---

## 5. Booking: cuts, corrections and back pay

The rule is LANGUAGE §5.13.3. Operationally, for an instance under an ask with cut K:

0. **Date it at K.** Its `on` is evaluated at cut K, giving t. A date learned later moves the instance under later asks only.
1. **Evaluate it at its booking cut** κ₀ = min(t, K): its `when`, then its amount. The versions visible are:
   - those known by κ₀;
   - `correction` versions known by K;
   - scenario versions, if t ≥ the scenario's `from`;
   - with t imprecise, a version is visible when known by the interval's first day and not when known after its last; one known inside the interval leaves the amount `HOLE-IMPRECISE` (LANGUAGE §4.4, §5.13.3).

   The result is the booked amount A₀. An instance whose `when` was false at κ₀ didn't exist then, and counts as A₀ = 0. An A₀ that is a hole, from its amount or its `when`, stays a hole movement until a version answers it: a `correction` answers it at t, and a change in the world measures δ from 0 under its `arrears` line. A version under which the `when` is false answers it with 0 (δ = 0, nothing booked).
2. **Find the later versions that can change it.** This step and the next apply to booked **flow pay** instances only. A modelled-past instance (§6.2) is computed at its booking cut for the ledger and never measured. A state pay instance is a decision taken on its day: its law and facts are read at κ₀ (step 1) and its look-ahead as known then (§4.2). It is never re-measured by a later version that isn't a `correction`, and no `arrears` line covers it (a `correction` reaches it through the walk at cut K, step 1). Its balances are the walk's at cut K, so records and anchors that a later cut sees can change it (§9 invariant 5). The candidates are every instance tuple of the pay in the version's cone, whether or not it existed at its cut. Take the versions V with κ₀ < V.known ≤ K that are neither corrections nor scenario versions, in `known` order; versions with one `known` are one step. Only those that touch the instance's read set can change it:
   - V has a candidate row for a read in the read set;
   - V is the `v` of a rule of a def in the cone;
   - V adds a key to an index that the cone aggregates.

   The others are skipped without evaluation.
3. **Measure each change.** For each step, re-evaluate the instance with its versions added, giving Aᵥ, and let δ = Aᵥ − A_previous. Versions with one `known` give one δ; if more than one of them has an `arrears` line covering the pay, the lines must agree (`LOAD-ARREARS`). When V makes an instance exist that didn't, δ = Aᵥ; when V ends one, δ = −A_previous.
4. **Act on δ ≠ 0** by the `arrears` line that covers this pay for V's change: V's own line, else a line `of V` in a later version W with W.known ≤ K. One line at most covers a pay for one version's change, V's own lines and the later `of V` lines together (`LOAD-ARREARS`, LANGUAGE §5.3):
   - **`on D`:** a correction movement of |δ| on D, between the pay's places, reversed when δ < 0;
   - **`none`:** nothing;
   - **no line:** a hole movement `HOLE-CHANGED`, dated on or after V.known, a hole date (LANGUAGE §5.13.3): every `flows` that can count the movement, over a span that reaches V.known or later, is a hole (LANGUAGE §5.13.3), and so are the pay's places' balances from V.known on, past a later anchor too (§4.2, §6.1). A later version that answers it does so only under cuts from its own `known`.
5. **A booked amount never changes afterwards** (LANGUAGE §5.13.3). Every later read of the instance sees A₀ plus the corrections dated in its span.

An instance dated after K is planned, at cut K. It has no later versions to apply.

**Example (synthetic).** A benefit is paid monthly through 2027. A version known 2028-03-01 governs 2027 and cuts the benefit to 0, with no repayment of what was paid:
- Each 2027 instance is booked at its own date with the old law.
- The new version changes each 2027 instance by δ = −amount.
- The version says `arrears none`, so the payments stand and nothing is booked.
- Had it said nothing, every `flows` over the benefit's account from 2028-03-01 on would be a hole asking whether repayment is due.

---

## 6. Replay against reality and the residual ledger

### 6.1 The past

Before K, a place's movements on a day are:
- on a **covered** day, its records (LANGUAGE §5.18);
- on an **uncovered** day, the model's booked pays (`past`).

The balance at S comes from `opening`, and from each anchor on, from its `stated` row (below). So the past is real wherever a statement covers it, and modelled elsewhere, and every movement says which it is (N-1201).

**Booking records.**
- A record that explains a pay instance is booked as that instance's movement, with the record's date and amount. One that explains a state pay instance, dated on or before the instance's day, is booked at the instance's point in the walk (§4.2 step 3), not at the start of its day; one dated after it is booked on its own day, in step 1.
- If the instance's two ends are both covered, its two records are its legs. It books once, and legs that differ print `WARN-LEGS`.
- An unexplained record is booked between its place and the outside place of its currency, so conservation holds.
- An explained record whose instance doesn't touch its place, or doesn't exist, is refused (`RUN-RECORD`).

**Anchors** (LANGUAGE §5.18). A `stated[x]` row that the cut K sees, governing a day D before K, re-anchors x: at the close of D (§4.2 step 4), x's end-of-day balance is set to the stated value, and later days build on it. So from D, x's balance is the bank's balance at its last anchor plus what is booked since; before x's first anchor, it is the opening plus what is booked since.
- **It isn't a movement.** Nothing is booked, `flows` doesn't count it, and no other place changes. The difference, stated − booked, where booked is the balance the walk reached at D before the anchor (the last anchor, or the opening, plus the movements since), is never hidden: it is the statement's balance line in the residual ledger (§6.3), one line per `stated` row, and an `anchor:` entry in the register of every figure that reads x's balance from D on (LANGUAGE §8.2).
- **Holes.** A hole movement of x dated on or before D, and the hole balances it made, stop at D: x's balances before D stay holes with their questions, which the balance line carries (it is a hole too), and from D x's balance is known. A movement that may fall after D (a hole date, or an imprecise date whose period runs past D) still makes x's balances holes from the first day after D on which it may fall, since the statement may or may not hold it. The other place of a hole movement is anchored only by a `stated` row of its own.
- **Decisions.** The anchor is applied at the close of D, after D's state pays, so it never changes a state pay dated on or before D; a state pay after D reads the anchored balance through the walk, under every cut that sees the row (§9 invariant 5).
- **Cuts.** A cut before the statement's `known` doesn't see the row (LANGUAGE §4.6), so its walk has no anchor there and its balances are the booked ones.
- **The modelled past** on covered days up to D is computed against the walk's balances on those days (§6.2), which an anchor at D doesn't change: it changes only the balances after D.

### 6.2 The modelled past

On covered days before K, every pay instance touching a covered place is computed at its booking cut and kept as modelled past (N-1202). This includes state pays: a state pay's modelled past is computed against the real balances. These instances are never booked, so the model's version of the past can't double the real one.

Law for the past reads records and facts, not the modelled past (N-1203):
- A def that needs what was really paid reads `flows` of a place, which on covered days are the records.
- A def that needs a payslip reads the payslip's rows.

### 6.3 The residual ledger

`ask q: residuals x span [by f]` compares, for place x over the span, the model at the ask's cut K with the records as known on the model's `on` date. The model there is each instance as booked or modelled at its booking cut (§6.2), plus the corrections dated in the span: the plan the household had on that day.
- When K = `on`, this is the replay.
- When K is earlier, it is the backtest of the forecast over (K, E] (§8).

Each line of the ledger is one of:

| Line | When | Shows |
|---|---|---|
| **matched** | an instance that one or more records explain, unless it is partly fulfilled | the amount difference (records − model) and the timing difference (record day − instance day) |
| **expected but missing** | a model instance in the span that no record explains | the instance's amount and date |
| **unexplained** | a record with `explains -` | the record |
| **partly fulfilled** | a promised instance whose records sum to less than its amount | the amount difference (records − model, as a matched line's), the remainder and a question: `when does the rest arrive, or is it written off?` (N-1206) |
| **balance** | every `stated` row of x, on its day D, whatever its difference (0.00 included; a hole when the booked balance is one). The booked balance is the last anchor before D (the previous `stated` row of x as known on `on`, else the opening) plus x's movements in the timeline at cut K on the days after it, through D; when K = `on`, that is the balance the walk reached at D before the anchor (§6.1) | the difference, stated − booked (a hole when the booked balance is one, with its questions): a gap or a duplicate in the import, or, where uncovered days lie between the last anchor and D, the modelled past of those days against the real one. Each `stated` row gives its own line, so drift points at its period; the anchor then resets x's balance, so the difference isn't carried into later lines |

- **Grouping.** Lines are grouped by month and by pay id, or with `by f` by a field of the movement input (a category, a counterparty). A line's month is its own day's: the instance's date for a matched, expected-but-missing or partly fulfilled line, the record's date for an unexplained one, D for a balance line. Unexplained records, which have no pay id, form one group per month named `unexplained`; with `by f`, they go by the field's value, and so does a matched line whose records agree on it; a line with no record, or whose records disagree, goes by its pay id, in a group printed `pay <id>`. Balance lines form one group per month named `balance`.
- **Zero lines.** A matched line and a balance line print whatever their difference, 0.00 included. An instance of amount 0 with no record is no movement and prints nothing.
- **Signs.** Every amount is signed as a flow into x: in +, out −. A matched or partly fulfilled line's difference is records − model; an expected but missing line's is − model; an unexplained line's is the record; a balance line's is stated − booked. A hole on either side makes the difference a hole.
- **Totals.** Each group gives the model's total, the records' total and the difference, so the prototype's "pay in December 2025: model 13 611.54, bank 16 611.54" is one group line. A balance group gives the booked balances as its model total and the stated ones as its records total.
- **Order.** Groups are sorted by the absolute value of their total difference, and lines within a group by their own, biggest first (N-1223). Holes come first, since a hole may be any size, among themselves by their first question (LANGUAGE §8.1). Remaining ties go by month, then group name, then the movement order of §4.1.
- **Cost.** It is computed from the timeline and the modelled past, with no re-evaluation.

### 6.4 The fitting loop

Claude fits a parameter by replaying history many times (N-2005). Each step is:
1. pin one input with `with`;
2. replay;
3. read the residuals.

The pin's cone is the only thing recomputed:
- cells outside it keep their memo;
- the records' booking is shared, since records are inputs that no pin reaches;
- the day walk restarts at the first day whose movements changed.

`PERFORMANCE.md` §4 gives the cost.

---

## 7. Monte Carlo

An ask with `paths N seed s` runs N paths (N ≤ 100 000, `REQ-PATHS`).

1. **The shared part.** Cells that no draw reaches are computed once and shared by every path: the brackets, the indexation, every input, every flow pay that no draw reaches, and the plan-valued look-ahead sums of §4.2.
2. **Per path p,** in its own memo layer over the shared one:
   - the draws are the values at their addresses (LANGUAGE §5.14);
   - the draw-dependent flow instances are recomputed;
   - the day walk runs if any balance or state pay depends on a draw;
   - the ask's per-path terms are evaluated.

   **Law per path** is just this: law defs in the draw cone are cells of the path's layer (R-20).
3. **Keep the terms, drop the path.** Each path's ask terms are kept (N values per term); its timeline is dropped. A path is recomputed alone on demand, because it is a pure function of its key (N-1406).
4. **Statistics.** `share`, `mean` and `quantile`, and the standard errors of `share` and `mean`, are computed over the kept terms as LANGUAGE §5.16 says, and each `share` gets its verdict, sayable or not (§8).
5. **Workers.** Paths are split into contiguous blocks, one per worker, and merged in path order. The number of workers changes nothing (LANGUAGE §7.5).

**Scenarios and `as if` share paths.** Addresses don't name the scenario or the pins, so a scenario and its base see the same uniforms on every path, and their difference has low variance (common random numbers, N-1302).

**Lifetimes** (CHG-08) need nothing special:
- a death is a draw per party per path;
- `alive[p, m]` is a def;
- the instances that end at death have `when alive[p, m]`.

The draw is keyed by the party, so it is the same in every scenario.

**By hand.** Pins of uniforms or values make a path exact (LANGUAGE §5.14, §7.4). A pinned path is a plain deterministic run.

---

## 8. The evaluation layer and its scores

The evaluation layer judges forecasts against what happened. It is not a separate mode. It is a set of asks that evaluate a forecast inside `as if {known K}` and the outcome outside it:

```
ask crps_sep: crps(bal(konto, 2026-09-30) as if {known 2026-03-31} for p in path, bal(konto, 2026-09-30)) paths 1000 seed 7 orient lower
```

- Inside `as if {known K}`, versions known after K are masked, records in (K, E] included. **The evaluation window can't leak into the forecast;** the language enforces it, not a convention (T-16).
- Outside, the outcome is read at the ask's own cut, normally `on`, from the records.
- **Rolling origins** are an index of cut dates. The pin takes an expression, `as if {known origin_day[o]}`, and an aggregate runs over the origins. Each origin is one cut class (`PERFORMANCE.md` §3).

**Scores** are the built-ins of LANGUAGE §5.15, and every score's direction is stated (`orient`):
- the CRPS of samples is exact, and that of normal forecasts closed-form;
- interval scores, weighted interval scores and skill scores are expressions;
- coverage is judged against exact binomial ranges, through `binomial`'s exact `cdf` and `quantile`;
- actual-to-expected ratios are counts and amounts with their ranges (CHG-10).

**Oracle tests** for Monte Carlo statistics (CHG-13) are `check` asks:
- the statistic is an ask term, written inline in the `check` (a path aggregate is ask-only, `LOAD-ASKONLY`): n/N for a `share`, the value for a `mean` or a `quantile`;
- its standard error, the oracle and the tolerance (4·SE plus the step bias plus the oracle's own error) are each a def or a row, with the standard error taken at the oracle. The **step bias** is a bound, stated by the model, on how far the scheme's time step moves the statistic's mean from the oracle;
- a parameter draw pinned to its mean is listed in the register as test mode, so a forecast can't carry it unseen.

**What the layer can't do** is a declared bound: the log score of paths (LANGUAGE §5.15, `BOUNDS.md`).

**When "n in N futures" may be said** (spec Q-03). Every `share` over paths prints, under its "n of N", a verdict: `sayable`, or `not sayable` with its reasons. The engine owns the rule because it has the register; the MCP layer only words the verdict. It is one deterministic rule over outputs the run already has, with three tests:
1. **Precision.** For n of N, the test's standard error is se_v = √(p̃(1 − p̃)/(N + 4)), with p̃ = (n + 2)/(N + 4) (Agresti–Coull). The printed standard error √(s(1 − s)/N) (LANGUAGE §5.16) is 0 at s = 0 or 1 whatever N is, so it can't be the test: "0 of 10" would pass. The test is 2·se_v ≤ 0.05, two standard errors at most 5 points either side, compared exactly as 1600·p̃(1 − p̃) ≤ N + 4. With N = 1 000 it always passes (se_v ≤ 0.0158, at p̃ = 1/2). With N = 100 it passes only for n ≤ 5 or n ≥ 95 (2·se_v = 0.098 at n = 50). 0 of N first passes at N = 52.
2. **Carry.** The carry c, in years, is how long, on average, the money the term reads rests on figures carried into the future. Take the plan movements into and out of the places the term reads, dated in (K, d], where d is the last day the term reads, each at its plan value (every draw at its plan value, as the look-ahead counts it, §4.2). Then
   c = Σ |a|·τ / Σ |a|,
   where a is a movement's amount and τ is its date minus K, in days / 365, when its amount's register holds a `held` or an `assumed` entry (LANGUAGE §8.2), and 0 otherwise. A term that reads no balance and no flow (a def over paths) takes c = the latest law date at which a `held` or `assumed` entry of its register is read, minus K, in years. The test is c ≤ 2 years. A movement whose amount reads a draw with no `plan` makes the test fail, with the reason `carry needs a plan value for draw <z>`. The horizon h = (d − K) in days / 365 bounds c (c ≤ h), and is printed beside it. A term that reads no day takes d as the latest law date it reads, so h = c.
3. **No test mode.** A share whose register holds a pin on a draw (the test mode of LANGUAGE §7.4, CHG-13, such as a parameter draw pinned to its mean) is a test, not a forecast. Its reason is `not sayable: test mode (pinned: <pin>)`.

The share is sayable when all three pass. The reasons name each test that fails, with its figures: `not sayable: carry 14.98 years > 2 (horizon 29.97 years)`, `not sayable: 2·se 0.187044 > 0.05 (0 of 10)`. The figures print rounded half to even, se_v to 1e-6 and c and h to 0.01 years; the tests compare exact values. The thresholds, 0.05 and 2 years, are the engine's for `tally 1`, and a model can't change them. `mean` and `quantile` over paths print c and h beside their standard error, with no verdict: the rule is about frequencies.

**Why these inputs.** The standard error says whether N paths are enough. The carry says how much of the result is the model's own assumptions rather than the world: a `held` figure is today's, carried unchanged, and an `assumed` one is nobody's statement. The further they are carried, and the more money rests on them, the more a frequency is the assumptions' and the less it is a fact; yet "n in 1 000 futures" reads as a fact.

**The spec's two examples, on the household** (by hand, at the rule's level). The household has no draws, so N = 1 000 paths over some draw, and a share s, are supposed; everything else is the household's.
- **"3 years of mostly known money":** `share(p in path: bal(konto, 2029-09-30) >= 0) paths 1000 seed 7` at K = `on` = 2026-10-19. The horizon is h = 1 077 / 365 = 2.95 years (`365 + 365 + 366 − 19 = 1077`).
  - Precision: se_v ≤ √(1/4 / 1004) = 0.01578, so 2·se_v ≤ 0.0316 ≤ 0.05, whatever n is.
  - Carry: konto's plan movements are monthly (wages, fees, taxes, spending, investing). Take every one as carried, the worst case: each reads a held tax or contribution row, or an assumed growth row. Equal months give c = h/2. Amounts growing by at most 4 % a year, 1.04³ − 1 = 0.1249 over the three years, give c ≤ h · (1/2 + 0.1249/3) / (1 + 0.1249/2) = 1.504 years (printed 1.50), which is ≤ 2.
  - **Sayable.**
- **"A 30-year flat-price bet":** the same share at 2056-09-30, with prices held flat (the price rows read as `held`). The horizon is h = 10 939 / 365 = 29.97 years (`30 · 365 + 8 − 19 = 10939`, eight leap days).
  - Precision passes, as above.
  - Carry: with all the money carried, c ≈ h/2 = 14.98 years. Even with half of it known (τ = 0) and spread like the rest, c ≈ h/4 = 7.49 years. Either way c > 2.
  - **Not sayable:** `carry 14.98 years > 2 (horizon 29.97 years)`.

**Cost.** One pass over the plan movements of the term's places in (K, d], which the shared layer already holds at plan values with their registers (§7 step 1): O(M) additions per share, with no evaluation. **Refusals:** none of its own. Where the share prints, the verdict prints; where the share is a hole, refused or a bound, that prints, with no verdict.

---

## 9. Invariants checked on every run

These hold by construction. The engine checks them anyway, cheaply, on every run, because a failure means an engine bug. A failure refuses the whole result with `ENGINE-INVARIANT`, the invariant and the first day and place where it fails. It is never printed as an answer.

1. **Conservation.** For each currency, on each day, the movements' two ends sum to zero, outside places included. So the balances sum to the openings plus the anchors' differences (§6.1) wherever these are known; a place with no `opening` row breaks nothing.
2. **Net worth moves only at the boundary, with a price, or at an anchor.** For the set of all places that aren't outside, the day's change in total worth equals the movements across the boundary plus the revaluation of holdings and assets that day (spec §3.2), plus that day's anchor differences, stated − booked (§6.1), which the ledger shows as balance lines.
3. **A scenario changes nothing before its `from`.** Every movement and balance before D equals the base run's (LANGUAGE §6.4).
4. **Asks move no money.** The timeline's hash is the same before and after the asks are answered. Pays inside `as if` are computed and never booked.
5. **A booked flow amount never changes.** Each flow pay instance's A₀ is the same under every K class that books it at the same cut. A state pay's amount isn't fixed that way: it reads the walk of its cut K, so records and anchors that a later cut sees can change what it reads and books (§4.2, §6.1), while the law, facts and look-ahead it reads stay those of its day (§5, §4.2); an anchor never changes a state pay dated on or before its day.

The test suite adds **determinism**: every case is run twice, with 1 and with 6 workers, and the result hashes must be equal (R-16, N-1401, spec §7).

---

## 10. Explanation and tracing

**`explain ref`** prints the derivation (LANGUAGE §5.16, N-1701):

| Of | It prints |
|---|---|
| a def cell | every rule of the def with its label and state: the winner with its `v` version and source, the rules beaten and what beat them, the rules not applicable, the rules of unknown applicability (a beaten rule's value is never computed, so none is printed); every row read, with version, source, locator, governs span and known date; the law date and the cut; the register |
| a pay instance | its date, booking cut, amount derivation, and each later version with its δ and what was done (correction, nothing, hole) |
| a balance | its last anchor, the opening or the `stated` row (with its version, source and its balance line's difference, §6.1, §6.3), and every movement since, up to that day, grouped by day, with each movement's kind and source |
| a path value | the path key, its draws in the canonical order, and the derivation on that path |
| a fill take | the fill's `v` version if it has one, the arcs in order, the remaining supply and demand before and after each take |
| a walk-bound def | the cell at each walk point where a state pay read it, in walk order, each with its point (day, phase, pay); with none, that nothing read it at those keys |
| an `as if` | the pins with their values (a pin given by an expression, with its derivation outside the `as if`), and the derivation inside them beside the one outside |

**Tracing isn't recorded during bulk runs.**
- `explain` re-evaluates the cell's cone with tracing on. The run is deterministic, so the traced value equals the stored one, and the memo supplies the children.
- A path is replayed alone (N-2004).

**Long derivations fold.** A recurrence over 300 months prints its first and last steps and a count. Any step can be asked for with `explain x[k]`. Repeated subtrees print once.

**`inputs-of ref`** is the transitive read set: every row the evaluation read, the rows an `at` reads included, with its source (N-1705). A rule's `v` version isn't a row; `explain` shows it. Across an axis it fans out, one set per line.

**The register** (LANGUAGE §8.2) is computed with each value, as the union of the registers of what it read. Sets are shared, so the union is cheap. A line's register is the union of its expressions'.

**Words for people.** The engine returns structured results: codes, values with units, source tags and the holes' questions, which are Polish in the household's model. A ready Polish sentence per result (spec §6.2, Q-04) is made by the MCP layer from these; the engine has no prose.

---

## 11. What the engine refuses

| Refusal | When | What happens |
|---|---|---|
| **Load refusals** (LANGUAGE §8.3) | the reality doesn't load | nothing runs. A broken reality never runs in part. `OPERATIONS.md` refuses an amendment that would break loading, before it is accepted. A load error in a scenario file refuses only that scenario and the scenarios over it |
| **Per-value refusals** (LANGUAGE §8.4) | a value can't be computed | printed at its key; it propagates and outranks holes |
| **Holes** (LANGUAGE §8.1) | a fact is missing | printed with their questions; other results stand |
| `REQ-ASK` | the request names an ask that doesn't exist (an ask given with the request that doesn't load is refused with its own load code, LANGUAGE §8.3, as `OPERATIONS.md` §8 says) | the request is refused |
| `REQ-CLOCK` | S outside the header's [start, end], or `as known` after `on` | the request is refused |
| `REQ-PATHS` | more than 100 000 paths for one ask | the request is refused |
| `ENGINE-INVARIANT` | §9 fails | the whole result is refused, with the evidence |

**What the engine never does:**
- choose an option for the household, or optimise, except a `solve` the law names;
- read a clock, or anything outside the reality and the request;
- write to the reality (only `OPERATIONS.md` writes: what the household confirms, statement imports and the store's advances, `OPERATIONS.md` §1);
- approximate without a stated precision;
- book a question's pays.
