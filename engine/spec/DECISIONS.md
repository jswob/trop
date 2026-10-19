# Tally: decisions

Every requirement of the spec (`tmp/engine-requirements.md`), every proposed change (`tmp/verification/changes.md`) and every fix the judge asked for (`judging.md` §8), each with what the design does about it and why. It also records the decisions the definition made on its own, with the alternatives weighed, and which construct each case needs.

**Statuses:**
- **taken**: Tally does what the ID asks, in the form it asks. "Taken (data)" marks law content: the constructs exist, and the law itself is data in a law part, written after the engine (principle 10).
- **merged**: the need is met by a more general construct than the one the ID names. The row says which.
- **rejected**: the design doesn't do it, on purpose. The row says why.
- **out of scope**: it belongs to a layer that isn't the engine: the MCP layer, the app, the import pipeline or the process. The row says which, and what the engine supplies to it.

**References:** L = `LANGUAGE.md`, E = `ENGINE.md`, O = `OPERATIONS.md`, P = `PERFORMANCE.md`, B = `BOUNDS.md`. Requirement texts are shortened; the spec has them in full.

**Contents**
1. Design rules DR1–DR7
2. Spec requirements: M, T, U, R, P, L, S, Q, C, A (222)
3. Proposed changes CHG-01 to CHG-14
4. The judge's fixes (judging.md §8)
5. Decisions the definition made, with the alternatives weighed
6. Departures from the spec, and the questions they raise
7. Constructs and the cases that need them

---

## 1. Design rules

| Rule | Text | Status | How, and why |
|---|---|---|---|
| DR1 | Derive, don't declare; no fact stated twice | taken | One namespace, `LOAD-DUP` (L §2.5). Instances, lots, claims and movement kinds are computed, never stated (E §1). A default row covers only keys known when it was stated, so a new key isn't silently covered (L §4.6) |
| DR2 | Checks and joins evaluated across time | taken | Rules are evaluated at each law date (L §4.3). A `check` ask binds a time index, so it runs at every period (L §5.16) |
| DR3 | Read by role or query, never by name; an exclusive role has ≤ 1 provider per window; each provider of a collective role is consumed by exactly one consumer per window | merged | Laws collect by query: sums over keys with `where` (L §5.10); asks pick places by kind (L §5.9). The role rules are `check` asks in the law part: a count per window. A built-in role system was rejected: it would be a second wiring language beside keys and sums, and checks make a violation loud |
| DR4 | Place-kind invariants | taken | `RUN-PLACE`, `LOAD-KIND` (L §5.9) |
| DR5 | Richer units: real vs nominal, certain vs promised, one money representation | taken | `prices Y by idx` (L §3.4); `promised` (L §5.13); money in minor units (L §3.1) |
| DR6 | Status outside the content, against a content hash | taken | O §1: draft, tested, confirmed, per hash |
| DR7 | Law values only through tested law templates, with the cross-check inside the gate | taken | O §3: a law part is read by the reality only when `tested`; independent checks are the cross-check |

---

## 2. Spec requirements

### M. Model and data

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| M-01 | Plain data from a closed list of block kinds, no code, serialisable for structured output | taken | A closed grammar (L §2.3), no user functions (L §10), a canonical form (L §2.4). Nesting is bounded (8 deep), so any schema of it is finite |
| M-02 | Place kinds: account, cash, holding (in lots), asset, debt (with contract state), claim, outside, memo | merged | Kinds `account`, `cash`, `holding`, `asset`, `debt`, `claim`, `outside` (L §5.9). Lots are keyed inputs and `fill`, not part of the place (CHG-01). A debt's contract state is inputs and defs. **Memo rejected:** a number no money moves through is a def |
| M-03 | A movement is dated, ≥ 0, direction only in from/to, carries why and period | taken | `pay`: `from`, `to`, `amount` ≥ 0 (`RUN-NEG`), `on`, `for` (L §5.13); every movement names its instance (E §1) |
| M-04 | Multi-leg movements: one event, several legs | merged | One event is several pays that share keys; an exchange is two pays, one per currency (L §5.9). Conservation holds per pay, so no multi-leg construct is needed. Two records of one instance are its legs (`WARN-LEGS`, L §5.18) |
| M-05 | A value: named number or table with dated entries {from, value or expression, repeat, source}, law or assumption | merged | Inputs with rows in versions (source, governs, known; L §5.3, §5.7); bands for tables (L §5.8); `assumed` rows (L §8.2). Rows are open-ended, so `repeat` isn't needed. Expressions live in defs, not rows |
| M-06 | Lookup: last entry on or before the date; `before`; `repeat` | taken | The read rule (L §4.6). "Just before" is a read at an earlier key or with `at` |
| M-07 | A template: versioned, typed parameters, lines, checks, sources, tests, roles and joins | merged | A law part (L §6.1): typed inputs with questions, defs with rules pointing at versions (`v`), `check` asks, sources. Law versions are versions of rows and rules, not of the part. Roles and joins: DR3 |
| M-08 | An instance: template, arguments, lifetime | merged | A thing is a key of an index with keyed rows. Its lifetime is its rows' spans, or a `when` on membership. There is no instantiation step |
| M-09 | A line: one formula on a grid, with a gate, a payment {from, to, fx, reverse, match} and a posting day | merged | A def keyed by a time index is the grid; `when` is the gate; a pay is the payment (L §5.13). Conversion is explicit (L §5.13.1); "reverse" is two pays or swapped ends; matching is the importer's data (O §2.2) |
| M-10 | Attached instances (law) live while what they attach to lives | merged | A law reads its subject's keys. When the subject's rows end, the law's sums see nothing. `end` checks that nothing keyed by it still moves money (`WARN-ENDED`, O §6) |
| M-11 | A parameter declares what it reads (an interface) | merged | Typed inputs and defs, unit-checked at load. The item graph is the interface (E §3.1) |
| M-12 | Party: a person with dated facts who owns places and incomes | taken | Parties are keys; their facts are keyed inputs (L §3.5); entities too (L §3.6) |
| M-13 | Ownership: dated exact fractions with kind and provenance; community property; one allocation with a declared remainder | taken | `owns[p, x]` with dated rows and a kind key; provenance by version (L §3.6); `prorata` with its remainder (L §5.12) |
| M-14 | Stored statements append-only | taken | Facts and law are add-only (L §6.2); the store is append-only (O §1). Records are movements, `stated` rows are balance assertions, notes are annotations, a confirmed match is a correction version |
| M-15 | Derived, never stored: occurrences, lots, law windows, certainty marks, claims | taken | E §1 movement kinds; instances computed; lots by `fill`; claims by pays |
| M-16 | Valid time and record time on every statement | taken | `governs` and `known` on every version (L §5.3); record time R in the store (O §1). There are three times, not two: what a row governs, when it was known, and when it was recorded |
| M-17 | Stable ids, used by patches and random keys | taken | L §2.5; draw addresses are `name[keys]` (L §5.14) |
| M-18 | Every bank account and category claimed by a block or explicitly ignored | merged | An account with no place is `OP-ACCOUNT` (O §2.2). A category's records that nothing explains are unexplained lines in `residuals … by cat` (E §6.3), so an unclaimed category is visible, biggest first |
| M-19 | Things no bank shows exist as places with a valuation | taken | `asset CUR priced v` (L §5.9) |
| M-20 | `run(model, history, clocks[, scenario]) → timeline` | taken | `run(reality, request) → result` (E §1) |
| M-21 | ASCII ids; Polish only in labels | taken | L §2.1, §10 |
| M-22 | Composite templates: one block per real-world thing expands into its wired pieces, visibly | merged | A new thing is keys and rows (L §6.5 change 1: 8 added lines). The law parts collect it by query, so nothing needs expanding; `describe` shows what reads it (O §9). **The user's answer (D8, 2026-10-19): accepted, rows plus queries, no expansion step.** A new thing no law handles is refused (`RUN-NO-RULE`) or prints a hole with its question, never a silent 0 (L §6.2) |
| M-23 | Collect by query, not by list | taken | Sums over keys with `where` (L §5.10) |
| M-24 | Law chosen from facts; one version per window; several laws in order; a coverage check refuses gaps and overlaps at load | taken | Rules with `when` and `excepts` (L §5.10); rows by the read rule. Overlaps of rows are refused at load (`LOAD-OVERLAP`); bands with gaps too (`LOAD-BAND`). **Departure:** whether rules cover every key and date can't be decided at load when `when` reads facts, so it is checked per value: `RUN-RULES` (two apply), `RUN-NO-RULE` (none) |

### T. Time and calendar

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| T-01 | Arrival windows, planned pessimistically, listed as an assumption | merged | An imprecise date (L §4.4) makes every span read it might fall in a hole, unless decided. Planning pessimistically is the model's own choice, written as an `assumed` row or rule. A built-in pessimism was rejected: it would be a silent default |
| T-02 | Once a windowed item happens, the forecast conditions on it | taken | Its record books on the covered day, and the instance is matched (E §6) |
| T-03 | Business-day shift over weekends and Polish holidays | taken | `next_business`, `prev_business`, `add_business` with a holiday `dates` input (L §4.2). Holidays are rows, so Wigilia from 2025 is a version |
| T-04 | Holiday calendars per country | taken | One `dates` input per country |
| T-05 | "The third-to-last business day" | taken | `add_business(end_of(m), -3, h)` |
| T-06 | Period ≠ payment date | taken | `for` and `on` on every pay (L §5.13). Card cycles are anchored months, `from 17` (L §4.1) |
| T-07 | Cumulative law: year to date minus paid; bases that carry forward | taken | Defs over month keys, `flows(p, span)` of the advances (L §5.13.1), recurrences (L §5.10) |
| T-08 | Last year as an input, from history or a stated fact | taken | A read at an earlier key behind a guard; the first year's figure is a row (L §5.10) |
| T-09 | Which date a value is read at | taken | The law date and `at` (L §4.3) |
| T-10 | Cash basis: law reads payment dates of other movements across years | taken | `flows(p, span)` sums by date; `for` by period (L §5.13.1) |
| T-11 | Relative dates from speech: anchor, phrase, window, the defaults rule, all shown | merged | The calendar built-ins (L §4.2) are the resolver. The MCP layer maps the phrase to a built-in with the statement's day as anchor; the engine evaluates it in an ask; the row keeps the phrase and anchor in its locator. So the engine computes the date, not Claude. **The user's answer (D8, 2026-10-19): the split is accepted:** the MCP layer maps the phrase, the engine computes the date, the row keeps the phrase and anchor |
| T-12 | Fixed phases within a day; a declared posting order | merged | Records and flow pays first, then state pays by `phase` 0–9, pay id and keys (L §5.13.2). The spec's eight named phases map onto numbers the model chooses |
| T-13 | The environment resolves monthly, declared as an assumption | merged | A price's rows govern months, and the read rule reads the month's row. An assumed resolution is an `assumed` row |
| T-14 | Law periods without built-in periods | taken | Anchored indexes, spans, one-off periods (L §4.1) |
| T-15 | "What did we think in March?", "what changed since last month?" | taken | The reality at R plus `as known` (E §2, O §1); `plan` between two record times (O §7, O §9 `history`) |
| T-16 | Four clocks S, K, (K, E], R; every read masked at K; a plain forecast sets S = K | taken | E §2. **Departure:** S is the header's `start`, not K, so a plain forecast replays the covered past before K. A request may set S = K (E §2) |

### U. Money, units and inflation

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| U-01 | Unit algebra | taken | L §3.2 |
| U-02 | Only small integers as literals | taken | Stricter: L §7.1 also bans date literals in rules |
| U-03 | Round where the law rounds, with its mode and unit at each step; tests tolerate published 1-grosz disagreements with a reason | taken | `round(x, u, mode)`: seven modes, any positive step (L §7.2). A published disagreement is a check with its errata (O §3, L-41) or an axis |
| U-04 | EUR and USD places; FX as dated values; NBP previous business day; a group balance at a declared FX read | taken | Places per currency; rates `num A/B` (L §3.2); `prev_business` with `at`; `flows … in CUR at fx mode` (L §5.13.1) |
| U-05 | FX applies only between different currencies | taken | Unrepresentable: `money EUR × num PLN/EUR` is PLN, and the same rate applied to PLN money is `LOAD-UNIT` |
| U-06 | Holdings in FIFO lots; value = units × price; revaluation by unit flows | merged | Holdings in units, priced (L §5.9); lots by `fill` in FIFO order (CHG-01); revaluation by an ask over `worth` and unit flows |
| U-07 | A gain split into FX, price and interaction | merged | Ask expressions with `as if` pins of the rate or the price (L §5.10) |
| U-08 | Simulate nominal; past amounts recorded, never recomputed | taken | L §3.4; booked amounts are fixed (L §5.13.3) |
| U-09 | The price index lives in the model, per scenario | taken | The index is an input; a scenario's rows override its future (L §6.4) |
| U-10 | A today's-money read side, the default on every chart | merged | `real(x, d, base, idx)` in asks (L §5.16). Chart defaults are the app's (out of scope) |
| U-11 | `today_money(x, date)` in questions | taken | `real` |
| U-12 | Spending learned from history converts each past row to today's prices before averaging | merged | The estimator is a def over record rows (L §5.18), and it writes the conversion with the index (`× idx[base] / idx[d]`). `real` itself is a lens for asks |
| U-13 | Every entered amount says which year's złoty it is in | taken | `prices Y by idx`, else nominal (L §3.4). A `prices` input is converted on every read, so a formula can't forget |
| U-14 | The deflator as a lens over any dated series | taken | `real` takes any index (L §5.16) |
| U-15 | Rates carry value, per, basis, convention | taken | The basis is the rate's type; day counts are named in `dcf` (L §3.3) |
| U-16 | Store what was stated; gross from net by the payroll law, by integer bisection under its rounding | merged | The stated net is a row. The gross is found by `solve … to 0.01` in an ask (L §5.16), and recorded as an `assumed` row that cites it (P-21). A def `solve` is allowed only where a law names the method |
| U-17 | Run-time conservation; net worth moves only at the boundary or with a price | taken | E §9 |
| U-18 | Money that isn't ours is a liability rules can't invest | taken | `holds`, `free`, `WARN-HELD` (L §5.9) |
| U-19 | Certain vs promised money | taken | `promised` is never counted by `plan_low` or `free` (L §5.13) |
| U-20 | One money representation | taken | Integer minor units (L §3.1) |

### R. Run semantics

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| R-01 | The flow pass | taken | E §4.1 |
| R-02 | The day walk | taken | E §4.2 |
| R-03 | Strata inferred; questions are state | taken | State pays are inferred (L §5.13.1); asks may make state reads |
| R-04 | No foresight by construction: look-ahead sees only planned flows, never a rule's own future, drawn spending, random keys or the agenda | taken | `plan_low` counts only flow pays as known that day, each draw at its plan value (L §5.13.1, §5.14); state pays can't read later days (`LOAD-FUTURE`); `next_on` can't name a state pay. The draw's `plan` closes the path case, found while writing this file |
| R-05 | Look-ahead primitives (`project`, `min_over`) with the literal sentence as default | taken | `plan_low` over a place or a group (L §5.13.1); the household's planning figures are draws' `plan` values |
| R-06 | The forecast subtracts the household's planned transfers, including those made by other rules | merged | Planned transfers that are flow pays are subtracted. A state rule's future result isn't, by design: no rule reads another rule's future (N-1102, spec §4#1). A transfer planned in advance is written as a flow pay. **The user's answer (D8, 2026-10-19): accepted.** A planned transfer is a flow pay; a rule never reads another rule's future |
| R-07 | Measured pool vs paying pool; defined behaviour when one goes negative | taken | `plan_low` over a group of places, the lowest sum (L §5.13.1, added while writing this file). An account below zero prints `WARN-NEGATIVE`; a debt, claim or holding past its limit is refused |
| R-08 | Ordered allocation, the limit re-checked after each posting | taken | State pays in phase order; a same-day `flows` total counts what is booked so far (L §5.13.1, added while writing this file) |
| R-09 | Competing rules by priority, then stable id | taken | `phase`, then pay id, then keys (L §5.13.2) |
| R-10 | A fallback (sell to cover) reads realised state only, counts T+2 proceeds, counts its firings | merged | A state pay; its proceeds are a second state pay two business days later reading the sale's history. The rule counts its own unsettled proceeds from `flows` of past days. Firings are an ask: `count(d in day where flows(sell, d) > 0)` |
| R-11 | Every run reports a rule's breach rate and mean shortfall, and a run with the rule off | merged | Asks (`share`, `mean`); "the rule off" is an axis the rule's `when` reads, pinned with `with` (L §5.5). Not automatic: the model asks for it |
| R-12 | Reacting to published signals is allowed; to the model's own forecast is not | taken | A rule reads inputs (published series, masked at K) and the past. Asks can't be read by defs or pays, so a forecast can't be a signal. A typed lagged signal was rejected: `plan_low` is the one sanctioned look-ahead |
| R-13 | "Once, when …" triggers on a state condition; the firing date anchors the template's schedules | merged | A state pay with `when` on state and on its own history (`flows(buy, ..d - 1) = 0`) fires once. Dependents read the firing day from `flows` of past days; they are state pays, inferred |
| R-14 | Goal-seek only at query level, never inside a run | taken | `solve` in asks; in a def only where the law names the method (L §5.16) |
| R-15 | Back-testing a rule on real history; fits use data up to each decision; "did we follow it?" | taken | Masking at K and rolling origins (E §8); a state pay's modelled past is computed against the real balances (E §4.2, §6.2) |
| R-16 | Deterministic to the grosz | taken | L §7.5 |
| R-17 | Fast | taken | P §8 |
| R-18 | Bands, "N in 1 000", common random numbers by stable id | taken | `quantile`, `share` with SE (L §5.16); addresses (L §5.14) |
| R-19 | Runs stored as aggregates by hash; paths replayed on demand | taken | E §1, §7; P §1 |
| R-20 | Law per path at an acceptable cost | taken | E §7; the cost is estimated in P §5 and measured at step F |
| R-21 | Defaults are preferences: the literal rule is the default; caution is the household's choice | merged | No built-in caution. Plan values and window choices are the household's rows |
| R-22 | Solving inside a law only with a solver the law names; every other cycle refused | taken | A bounded recurrence with `first` (L §5.10); `solve` in a rule with `v` (L §5.16); the previous period's value is an earlier key. Same-key cycles are `LOAD-CYCLE` |

### P. Past meets future

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| P-01 | Before today bank rows; from today generated rows; start from real balances | taken | Records on covered days, the plan from K (E §6.1); `opening` (L §5.9) |
| P-02 | The modelled past computed, never booked, reconciled with the bank | taken | E §6.2, §6.3 |
| P-03 | Law for the past reads records, not the modelled past | taken | `flows` of a place reads records on covered days (E §6.2) |
| P-04 | An earner's past comes from dated history | taken | Rows govern dated spans (L §4.6) |
| P-05 | Certainty marks derived, plus a dated missed state | merged | Movement kinds `record`, `past`, `plan`, `promised`, `correction`, `hole`, and the modelled past (E §1). "Missed" is the ledger's "expected but missing" line (E §6.3) |
| P-06 | Partial fulfilment: the remainder's date and chance are asked | taken | The "partly fulfilled" ledger line, with its question (E §6.3) |
| P-07 | A missed window: keep open, move or write off; the household decides | merged | The ledger line asks; each answer is an amendment: a row moving `on`, or a `-` row (O §2) |
| P-08 | Exact matches link automatically; fuzzy ones proposed; a window per prediction; holds replaced; repricing as `replace` with a valid-from | taken | O §2.2: the household's confirmed rule (`match_days`, `match_amount`); holds restated by key. Repricing is a version that governs from its date |
| P-09 | A chance item not happened by its window's end is missed; later chances update | merged | A covered day before K shows what happened; an instance with no record is "expected but missing", and the modelled past is never booked. A chance is an input; updating it is an addition |
| P-10 | Author kinds; only confirmed ones count | merged | Authors in the store (O §1); provenance per version (L §3.7) |
| P-11 | A stated number beats a probability word; an editable lexicon | out of scope | The MCP layer. The engine takes numbers only |
| P-12 | Uncounted cash: a balance assertion with a pad flow | merged | A `stated` row for a cash count (L §5.18). It re-anchors the place's balance from its day (D8, J70), with no movement; the difference is a ledger balance line (E §6.3), and the household may add the record that explains it. An automatic pad was rejected: it would be a figure nobody stated |
| P-13 | Claims derived from open commitments | merged | A claim place booked by pays (L §5.9) |
| P-14 | Refunds, split bills, own-account transfers, card cycles, a 13th salary, re-categorisation, a disagreeing forecast, 0% instalments | merged | Each is existing constructs: a refund pay or an unexplained record; a claim; two legs of one instance; anchored months; a yearly pay; a correction restating a record's field; the ledger; a debt place |
| P-15 | Money to a new counterparty: still ours (a claim) or gone (outside)? | taken | O §2.2 asks it for a counterparty with no `party_place` row |
| P-16 | Spending fitted from history; parameter uncertainty drawn once per path, unreadable by forecasts | merged | Generators are draws and defs (L §5.14). Fitting is Claude's loop (E §6.4); the engine never fits (the user's decision). A parameter's draw has a `plan` value, so forecasts never read the drawn parameter |
| P-17 | Agreement with the bank counts as evidence only on real imports | out of scope | The process. Every ledger line carries its records' source, so a synthetic import is visible as one |
| P-18 | The import interface: classifier, normalisation, clustering, broker history, holds and duplicates | out of scope | The user's pipeline (spec §2.3), except holds and duplicates, which the import operation handles (O §2.2) |
| P-19 | Today's state settled before what-ifs | merged | At K, missing expected records are ledger lines with questions (E §6.3). A scenario runs over whatever the reality says (L §6.4) |
| P-20 | Each bank row linked to the instance and period it explains | taken | `explains` (L §5.18) |
| P-21 | Gross pay history from payslips; inferred from net by running the law backwards, marked inferred, checked against PIT-11 | merged | Payslip rows; `solve` in an ask; an `assumed` row citing it; a `check` against PIT-11 |
| P-22 | A pay trajectory fitted from history, marked modelled, a draft until confirmed | merged | Claude's fitting loop writes `assumed` rows (E §6.4); drafts until confirmed (O §1) |
| P-23 | Reconcile timing too, with a suggested day or window | merged | The ledger's timing difference (E §6.3). The suggestion is Claude's, from the ledger |
| P-24 | The import contract: counterparty resolved to a place; matching on places, period, window, tolerance; category a hint | taken | `account_place`, `party_place`, `match_days`, `match_amount` (L §5.18, O §2.2) |
| P-25 | One registry of entities with stable ids; renames keep ids; dated merges and splits; parent categories | merged | `bank_party` keys are stable; `party_place` rows are dated, so a merge is two keys mapping to one place from a date. A parent category is a relation input. The registry itself is the pipeline's (out of scope) |

### L. Law and tax

The law content (L-01 to L-23) is data: law parts the engine runs, written after it (principle 10, the user's "universal engine"). For each, the row names the constructs that express it.

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| L-01 | Employee: gross to net, the 30× ZUS cap, the 120k threshold, KUP, PIT-2, the annual return, the refund | taken (data) | Year-to-date sums with `min` for the cap; cumulative bases (T-07); rows for KUP and PIT-2; joint or separate as an axis or a fact; the refund as a pay dated by the return |
| L-02 | PPK legs; reliefs | taken (data) | Pays per leg; reliefs as rules with `when`; ordered reliefs by `fill` |
| L-03 | A raise says how it meets the wage index | merged | The raise row says its basis: `prices Y by wage_index`, or nominal (L §3.4). CHG-14 gives G6 its statement |
| L-04 | Sole trader: forms, ZUS stages, health, FP, ryczałt tiers, a mixed car, the settlement | taken (data) | Rules on the form (a fact or an axis); bands by months since start; shares as `num`; the settlement reads `flows` of advances |
| L-05 | Sole-trader gaps: cash-basis deduction, health true-up, health as a cost, refusing a mid-year form change, sick pay | taken (data) | `flows` by date; a settlement pay; defs; a `check` or a hole rule for a mid-year change; `when` on voluntary insurance |
| L-06 | A fair B2B vs employment comparison | taken (data) | Scenarios or an axis side by side; no winner (L §10) |
| L-07 | VAT: output minus input; a mixed car at 50% | taken (data) | Defs |
| L-08 | VAT extras | taken (data) | Anchored quarters; a carry-forward recurrence; `flows` for the cash method; year-to-date thresholds; the due date apart from the payment (`for` and `on`); the split-payment account left out of the liquid group by kind or id; KSeF amounts as record fields |
| L-09 | Belka / PIT-38 | taken (data) | Yearly sums; FIFO by `fill`; `max(0, …)`; a pay on 30 April |
| L-10 | Belka extras | taken (data) | `prev_business` with the NBP rows; losses by `fill` with a 50% cap per year; opening lots as rows; "if we sold today" by `as if`; withholding as pays |
| L-11 | IKE: dated limit never going down, exceeding refused, the qualified-withdrawal gate | taken (data) | Rows, with a `check` that the limit doesn't fall; the investing rule takes `min` with the room; a direct pay over the limit fails a `check`; the gate compares dates with `age` |
| L-12 | IKZE and PPK | taken (data) | The refund at the marginal rate by `as if` |
| L-13 | OKI from 2027 | taken (data) | The average daily value as a state read over days (a state pay); the reference rate row on 31 October; `round(…, 0.0001, down)`; `max` for the floor; `min` for the exempt amount |
| L-14 | IKE vs OKI vs an account as scenarios, with a breakeven, never a choice | taken | Scenarios side by side and `solve` (L §5.16, §6.4) |
| L-15 | Bonds: capitalisation, CPI with a lag and a vintage, early-redemption fee | taken (data) | A recurrence; the index read at `m - lag`; a vintage key; `when` |
| L-16 | An annuity mortgage | taken (data) | `factor` and a recurrence (L §3.3) |
| L-17 | Loan extras: resets, WIBOR vs POLSTR by signing date, overpayment, 0% instalments | taken (data) | Rate rows; the POLSTR start date is a row, read by `when signed[l] >= polstr_from` (no date literal in a rule, L §7.1); overpayment as a fact or an axis; a debt place |
| L-18 | Property: PCC with the first-home exemption, fees, running costs; rent ends at purchase or a warning | taken (data) | Defs; the household's `check` "one housing cost at a time" (S-08) |
| L-19 | The 5-year PIT-39 clock and its relief | taken (data) | `add_years` and year ends; a def |
| L-20 | Zasiłek | taken (data) | Rules with `when` |
| L-21 | Benefits | taken (data) | Defs and pays keyed by child and month |
| L-22 | German employment (PAP), all classes, a joint return with Poland | taken (data) | A law part. The tariff used three times is one def read `as if` with each argument (A-06) |
| L-23 | Polish social insurance for a German job | taken (data) | A classification def (CHG-05) that the ZUS rules read |
| L-24 | A published law change is a dated entry with its applies and known dates; an unpublished one only a scenario | taken | Versions with `governs` and `known` (L §5.3); scenario versions have no `known` (L §6.4) |
| L-25 | Unverified points | out of scope | Law research. Each is an `assumed` row or an axis until checked |
| L-26 | Eligibility, not only arithmetic: the joint return checked against its conditions | taken (data) | Rules with `when` over facts; an ineligible return is not applicable (`-`) |
| L-27 | Retroactive law books the difference as dated arrears or a refund; with no date, a hole | taken | `arrears on D | none for <pays>`; no line gives `HOLE-CHANGED` (L §5.13.3, E §5) |
| L-28 | Several laws on one income in a stated order; trying orders only where the law says, with a cap | taken | The order is the read graph (L §5.10); `fill` for shared caps; orders tried as an axis, capped at 64 (`LOAD-FORKS`) |
| L-29 | "As if" inside a law | taken | `as if` (L §5.10) |
| L-30 | The gate, in order | taken | O §3, seven steps. Coverage and planted errors were added to O §3 while writing this file |
| L-31 | Tests from published figures, collected by a non-author | taken | O §3 step 3 |
| L-32 | Both kinds of test kept | taken | O §3 step 3 |
| L-33 | The gate's scripts and figures hidden from the author | taken | O §3 step 3 (process) |
| L-34 | The cross-check inside the gate | taken | O §3: the independent checks are the cross-check |
| L-35 | Payslips and real returns become tests | taken | O §3 step 3 |
| L-36 | An independent oracle agrees to the grosz | taken | An oracle is a check. D7's evaluators and the build's tests are this process |
| L-37 | Rounding errors caught | taken | O §3 step 5 plants rounding errors |
| L-38 | Behaviour between and beyond tested points probed | taken | O §3 step 6, the threshold table |
| L-39 | Expected answers are themselves checked | merged | Every expected figure has a source and locator (O §3 step 2); a figure two sources disagree on is an errata check (L-41). Collecting it twice is process |
| L-40 | Who writes law doesn't matter | taken | O §3 (the user's decision) |
| L-41 | A published figure that disagrees with its law: match the law, keep both, report the gap | taken | O §3 |

### S. Scenarios and their life

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| S-01 | A scenario: base, `from`, operations; reality never changed | taken | Scenario files (L §6.4) |
| S-02 | Stacking | taken | `over`, at most 4 deep |
| S-03 | Patches on stable ids: add, remove, replace, end, override | merged | Rows override, `-` and 0 rows end, new sections add. "Remove" is `-` rows from `from`: nothing is deleted, so the base's history stays readable |
| S-04 | `replace` keeps draws only if numbering is preserved | taken | Addresses are keys: the same keys keep their draws (L §5.14) |
| S-05 | Rebase, a gone target, stale, promote | taken | O §5. A target can't be gone, since ids and keys are never removed |
| S-06 | Counterfactual pasts: a branch date, plus which later facts survive | merged | A scenario `from` a past date; every later reality version still applies unless the scenario overrides it (L §6.4). `as known` for what was known then |
| S-07 | Linked overrides | taken | The model's own derivations carry the override (O §5) |
| S-08 | Valid but wrong warns | taken | The model's `check` asks run in the scenario (O §5) |
| S-09 | Scenarios only from household statements | taken | `OP-STATEMENT` (O §5) |
| S-10 | A saved analysis | taken | O §8 |
| S-11 | Shared futures; environment counters in absolute months | taken | Draw addresses don't name the scenario; time keys are calendar keys, not offsets from `from` |
| S-12 | Isolation: new pieces don't move proven results | taken | `plan` lists every result that moves (O §7); checks re-run in the gate |

### Q. Questions, explanation and the read side

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| Q-01 | Questions read-only; answers are queries over the run | taken | Asks have no pay (L §5.16); aggregates `first`, `last`, `min`, `max`, `argmin` over days |
| Q-02 | Measures: futures, first passage with censoring, the lowest day, lenses, IRR/TWR, price/FX split, change, sensitivity | merged | `share`; `first` with a test of `any` for censoring; `argmin`; sums by field or week; `solve` for IRR; `as if` for splits and sensitivity; `plan` for change |
| Q-03 | One rule for when "N in 1 000 futures" may be said | taken | **The user's answer (D8, 2026-10-19): the engine owns the rule.** Every `share` over paths prints `sayable` or `not sayable` with its reasons (E §8, L §5.16): two standard errors ≤ 0.05 (Agresti–Coull, so "0 of 10" isn't sayable; clean-side audit F1, `AUDIT-D8.md`), the carry (the money-weighted years the term's movements rest on `held` or `assumed` figures) ≤ 2 years, and no pin on a draw. On the household, 3 years of mostly known money gives a carry ≤ 1.50 years (sayable); a 30-year flat-price bet about 15 (not sayable). Cost O(movements in the window), no refusals of its own. The MCP layer words the verdict. Build: M7 (BUILD-ORDER) |
| Q-04 | Every number from a tool result: small, units, source tag, Polish sentence | merged | Typed results with units and source tags (O §1). The Polish sentence is the MCP layer's |
| Q-05 | Templates describe themselves | taken | Inputs carry questions; `describe` (O §9); `check` asks as sanity checks |
| Q-06 | Difference ledger; attribution; representative paths; Shapley | merged | `plan` (O §7); `as if` differences; a path chosen by `quantile` and replayed by `explain`. Shapley is an ask over `as if` with up to 6 causes (2⁶ = 64, the fork cap) |
| Q-07 | Net worth at market value and after liquidation, neither the default | merged | Two asks; liquidation by `as if` a sale |
| Q-08 | No weighting, no winner; breakevens; the median path-wise difference | taken | L §10; `solve`; paired paths (L §5.14) give the path-wise difference |
| Q-09 | Provenance on every movement | taken | E §1, L §3.7 |
| Q-10 | A derivation tree | taken | `explain` (E §10) |
| Q-11 | An assumption register | taken | L §8.2 |
| Q-12 | Transitive, uncapped, with falsifiers | merged | The register is transitive and uncapped (L §8.2). A falsifier is a `check` the model writes against records to come |
| Q-13 | A known-through horizon, so "held" separates known from guessed | taken | `held` in the register (L §8.2) |
| Q-14 | Today's money by default on charts | out of scope | The app; `real` is the engine's part |
| Q-15 | A warning when a reader outlives what it reads | taken | Stricter: a read past the last row is a hole (the gap rule); an open-ended row read past K is `held` |
| Q-16 | Read places by kind or role | taken | `kind`, `currency`, `owns` (L §5.9) |
| Q-17 | Warnings: below zero, no further pay, selling more than held | taken | `WARN-NEGATIVE`; `next_on` with no instance left is a hole (L §5.13.1); `RUN-PLACE` |
| Q-18 | "What does my job really pay?" | merged | Asks over the linked records and the payslip rows |

### C. Built-in correctness

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| C-01 | The validator runs first | taken | Load (E §3.1, L §8.3) |
| C-02 | The language is total; templates nest a fixed depth | taken | L §10; 8 levels of expression, 4 of `as if` |
| C-03 | No value on a date is refused, never 0 | taken | The gap rule (L §4.6, §8.1) |
| C-04 | Every employment sits in some annual return | merged | A job read by nothing is `WARN-UNREAD`; the role rule is a `check` in the law part (DR3) |
| C-05 | Joins checked per year, with "any of" | merged | `check` asks over years with `any` |
| C-06 | Checks run on every date the arguments change | taken | A check binds a time index (DR2) |
| C-07 | Two instances in one role at once warn | merged | A `check` (a count ≤ 1 per month), which also runs in scenarios (S-08) |
| C-08 | Debt ≤ 0; a payment into a debt capped at its balance | merged | A departure from the spec's "capped": `RUN-PLACE` refuses rather than caps, since a cap would hide the error. A model that wants a cap writes it, with `min` or a `last` rule (the household's loans: `principal_part` takes `min(debt_before, …)` and its `last` rule the remainder, and `overpay_paid` is `min(overpay, debt_before − principal_part)`, so no pay into `loan_debt` can overshoot; checked in D8 for the laptop loan and change 6's car loan). **The user's answer (D8, 2026-10-19): refuse; a cap is written explicitly** |
| C-09 | Draft law refused in reality, warns in a scenario | taken | O §3, `WARN-DRAFT` |
| C-10 | Status outside the content; Claude's parts load as draft; only the user confirms | taken | O §1 |
| C-11 | Derive, don't declare | taken | DR1 |
| C-12 | Law values only through tested templates | taken | DR7 |
| C-13 | Property tests for no foresight | taken | Build tests (`BUILD-ORDER.md`); the design makes foresight unrepresentable (R-04) |
| C-14 | Conservation at run time | taken | E §9 |
| C-15 | The engine frozen by hash; data-only additions provably data-only | taken | `tally N` (L §9); an amendment never changes the engine |
| C-16 | Regression tests for the prototype's bugs | taken | Exact arithmetic has no rounding helper to lose a cent; FX by unit types (U-05); both are build tests |
| C-17 | An earner nobody reads is flagged | taken | `WARN-UNREAD` |
| C-18 | Redefining an id refused unless versioned | taken | `LOAD-DUP`; `revise` with a reason (O §4) |
| C-19 | A question read-only by construction | taken | L §5.16 |
| C-23 | The scenario suite | taken | Process: D6's `amendments/`, the build's tests |
| C-24 | A combinatorial matrix and every threshold | merged | Checks bound over a matrix of inputs, and the threshold table (O §3 step 6) |

### A. Claude and MCP

| ID | Requirement | Status | How, and why |
|---|---|---|---|
| A-01 | One catalogue is the whole contract, at a size that fits | merged | LANGUAGE §2, §5 and §8 are the catalogue. A reference card cut from them, about the prototype's 13k tokens, is a build task (`BUILD-ORDER.md`) |
| A-02 | A fresh Claude writes correct blocks | taken | Evidence: D4's fresh modellers; D6's modellers |
| A-03 | The validator is the last line; drafts run on a fixed environment; no LLM judge | taken | Load, the gate, `plan` (O §1). No LLM judge anywhere |
| A-04 | The schema Claude writes is non-recursive | merged | Claude writes text, checked by the parser. Bounded nesting keeps any structured schema finite |
| A-05 | The formula surface chosen by evidence; a change keeps pinned results and gate tests | taken | D3–D4 (`judging.md`); migrations (L §9) |
| A-06 | Named formula macros | merged | No user functions (L §10). A formula used with several arguments is one def read `as if` each argument (N-1904) |
| A-07 | Sign or direction: legs only, kinds checked | taken | L §5.13 |
| A-08 | Rate period or basis: every field required | taken | L §3.3 |
| A-09 | Nominal vs real: a required price year | taken | L §3.4 |
| A-10 | Gross vs net: store what was stated; the gross is a hole | taken | An input with no row is a hole |
| A-11 | Relative dates: anchor and phrase, resolved by the engine | merged | T-11. **The user's answer (D8, 2026-10-19): the split is accepted** |
| A-12 | Probability words: the lexicon; stated numbers win | out of scope | The MCP layer (P-11) |
| A-13 | Silent defaults: a named default entry, shown | taken | `assumed` rows (L §8.2) |
| A-14 | LLM arithmetic: expressions; only stated literals | taken | L §7.1 |
| A-15 | Tax or market literals banned | taken | L §7.1 |
| A-16 | Omitted consequence: a warning and a question | merged | New keys list the holes they open (O §2.1); `WARN-ENDED` (O §6); checks in scenarios (S-08) |
| A-17 | Entity confusion: resolve against ids; ambiguity a question | taken | `OP-TARGET` (O §1) |
| A-18 | Duplication: replace, not duplicate | taken | `LOAD-DUP`; duplicate records (O §2.2); `restate` |
| A-19 | Scenarios only from statements | taken | `OP-STATEMENT` |
| A-20 | An answer key of 80–150 Polish utterances on the eval harness | out of scope | The MCP layer's evaluation (`apps/evals`) |
| A-21 | The MCP surface | taken | O §9 |
| A-22 | API data retention | out of scope | The app's operations |
| A-23 | Tuning tools: `reconcile`, `drill`, `explain`, `trial` | taken | O §9 |

---

## 3. Proposed changes

| Change | Status | How, and why |
|---|---|---|
| CHG-01 Sub-ledgers by vintage inside a place | taken differently | No built-in lots or pooling. Lots are vintage-keyed inputs and defs, matched by `fill` in the law's own order (L §5.11, §10). Each jurisdiction pools differently (FIFO, a UK pool, wash sales, a deemed sale), so a built-in rule would be wrong for most. A vintage-keyed def keeps the original size (LH-04) |
| CHG-02 History queries in rules | taken | `flows(p, span[, for k])` of a flow pay is not a state read (L §5.13.1); `bal(x, d)` and `worth(x, d)` in state pays; reads at earlier keys; aggregates over record rows (L §5.18). The look-ahead never reads the future (the change's risk) |
| CHG-03 Group units and relations with dates | taken | Membership and relation inputs with dated rows; sums over keys (L §3.5). No built-in unit type |
| CHG-04 Carryback | taken differently | The later year's loss is matched to earlier years by `fill`; the refund is its own pay, dated when claimed. The closed year's booked amount stays fixed (L §5.13.3), so history isn't rewritten |
| CHG-05 Classifications from facts | taken | Defs, read by other laws like any def; a court's ruling is a given (L §5.10) |
| CHG-06 Conventions: FX per leg, day count, compounding | taken | `dcf` with five day counts; the rate basis as type (L §3.3); FX per movement with a mode (L §5.13.1); per leg as rows or axes. No default can be written (L §10) |
| CHG-07 Entity parties and look-through | taken | Entities are keys with places; `stake`; look-through by a bounded depth recurrence (L §3.6) |
| CHG-08 Stochastic lifetimes | taken | A draw per party per path, `alive[p, m]`, `when alive` (E §7) |
| CHG-09 Ranked allocation | taken | `fill` in a stated order; `prorata` with its remainder; an unstated order is a hole (L §5.11, §5.12) |
| CHG-10 Scores with exact statistical functions | taken | `crps` exact over samples, closed form for normals; exact `binomial`; `orient` (L §5.15) |
| CHG-11 A law's own rule for a missing record | taken | `missing(r)`, only in a `when` of a rule with `v`; listed in the register; `LOAD-MISSING` elsewhere (L §5.10) |
| CHG-12 Estimators over history with what they carry | taken | Defs over record rows, masked at K; price years and indexes as rows (L §5.18, §3.4) |
| CHG-13 A test mode for Monte Carlo statistics | taken | Pins of a parameter draw to its mean, listed in the register; oracle `check` asks with SE and tolerance (L §7.4, E §8) |
| CHG-14 What the goldens need | taken | The goldens are pinned on a canonical projection of the result: movements and end-of-day balances (E §1). Ownership is stated as facts. L-26 joins the fixes that may move a pin. G6 gets its raise statement (L-03). All four are build tasks (`BUILD-ORDER.md`) |

---

## 4. The judge's fixes

`judging.md` §8 listed 21 fixes for B. Each, with where the definition settles it:

| # | Fix | Decision |
|---|---|---|
| 1 | A correction at or before K when the pay is a hole | A hole instance stays a hole; a later version is applied to it like any other (E §5). A correction known by K counts as known on t (L §5.13.3) |
| 2 | A later-known change to a booked pay with no `correct on` | C's `arrears` on the version, `HOLE-CHANGED` with no line, now per pay with `for` (L §5.13.3) |
| 3 | The forced `round(…, 1e-30)` for carried ratios | Exact rationals with the `RUN-DIGITS` cap at 4 096 digits, which refuses (L §7.1) |
| 4 | `solve`: which root; flag the edges | The grid point nearest the root, a tie going up; no sign change is `RUN-SOLVE` (L §5.16) |
| 5 | The error of composite transcendentals | Composites are exact on the rounded values; `≈` with an error bound when no `round` follows (L §7.3) |
| 6 | Age before birth; the child-born pattern on a monthly base | `age` is `-` before birth (L §4.2). The pattern is change 4 of L §6.5 |
| 7 | The empty `sum`; short-circuit `and` / `or` | `sum` over no keys is 0; `and` / `or` left to right with Kleene values (L §5.10) |
| 8 | Whether a `with` pin reaches an input read through `at` | It replaces every read of the input at those keys, `at` included (L §5.10) |
| 9 | A cite per row | Every row's version cites a source; a row may add a locator (L §3.7) |
| 10 | A law change without restating figures | Open-ended rows and the `version` header (L §5.3) |
| 11 | Name clashes | One namespace, every reserved word listed, `LOAD-RESERVED`, `LOAD-NAME`, `LOAD-SHADOW` (L §2.2, §2.5) |
| 12 | The single `=` after `when` and in `solve` | C's parse rule (L §2.3) |
| 13 | Types: a bare 0, `int × ratio`, `round`'s result, `year_of` on a month, `m + 1` past the end | L §3.1, §3.2, §4.2; `RUN-KEY` past the end |
| 14 | Whether the precision rule covers a recurrence's start rule | Every rule, the start rule included, is one rule body under the same rules (L §5.10, §7.1) |
| 15 | A declared bound: per value or at load | At load for defs, pays and fills (`LOAD-BOUND`); printed in asks (L §5.17) |
| 16 | The order of draws; `*` in keys | The canonical draw order (L §5.14); `*` covers the keys known at the row's date (L §4.6) |
| 17 | A pay whose date is a hole; a "nobody" key | A hole date makes every `flows` it might fall in a hole (L §5.13); not applicable is `-` |
| 18 | Poisson; argmin / argmax; an assertion ask; a lattice span | `poisson` (L §5.14); `argmin`, `argmax` (L §5.10); `check` (L §5.16); a span over an index needs no input (L §4.1) |
| 19 | Guidance on `given` against `input` | A given needs a source that states the result; an assumption is an `assumed` input row (L §5.10, `OP-GIVEN`) |
| 20 | A def without a time key evaluates at the model date | Law-date propagation (L §4.3 rule 3), not a refusal: §5 decision 1 |
| 21 | The cost of imprecise dates; a cap on axes | Two endpoint evaluations per operation (L §4.4); 64 forks (`LOAD-FORKS`) |

---

## 5. Decisions the definition made, with the alternatives weighed

Decisions taken in D5, beyond the candidates and the judge. Each weighs the alternatives against the qualities in order (spec §2.4: never wrong in silence; stated once; explainable; deterministic; fast; expressive; writable).

| # | Decision | Alternatives weighed | Why this one |
|---|---|---|---|
| 1 | **Law-date propagation:** a def with no `at` and no time key takes its reader's law date (L §4.3) | (a) B: the model's `on` date; (b) the lead's §9: refuse such a def when it reads dated rows | (a) is wrong in silence: a child's needs read by a July pay used `on`'s law. (b) is loud but forces a snapshot model to be restructured when it grows a time axis (quality 2). Propagation is right, and `explain` prints the law date it ran at (quality 3) |
| 2 | **K defaults to the model's `on`, which is required**; a version known after `on` is refused (L §4.5) | (a) no default cut, every version visible; (b) the latest known date | (a) lets a forecast read tomorrow's news. (b) moves with every addition. `LOAD-KNOWN` also catches a mistyped year |
| 3 | **Scenario versions are visible by date**, from `from`, not masked by K (L §6.4) | Scenario versions known on `from`, masked by K like reality's | With K = `on`, a scenario starting in the future was invisible to every forecast: a contradiction found in drafting |
| 4 | **A default row covers only the keys known at its date** (L §4.6) | C's `complete`, covering every key | A child added later would silently get the default 0 (quality 1). Instead the new key makes holes with questions (O §2.1) |
| 5 | **Records as a construct** (L §5.18): movement inputs, `covers`, `stated`, `explains`, the modelled past | (a) Records as plain input rows, as in the cases; (b) records in a separate ledger outside the language | (a) can't book the past or reconcile it, and replay is the core point (the user's decision). (b) splits one reality in two. Records as rows keep masking at K and provenance for free |
| 6 | **Four kinds of change:** add, advance, restate, revise (L §6.2, O §4; `advance` added in D6, gap L10) | (a) Add-only (B); (b) free edits (A's `replace`); (c) moving `on` as a restate of the header (D5) | (a) can't fix the model's own wiring. (b) can edit history in silence. (c) changes results, so it can't be a restate. Restate is proven by equal results; revise never touches a version; advance moves only `on`, forward, and shows its plan |
| 7 | **`arrears … for <pays>`** (L §5.13.3) | One arrears choice per version (C) | A retroactive version that changes both the monthly advances and the settlement would correct both: double counting |
| 8 | **A booked amount is fixed**; later versions give correction movements (L §5.13.3, E §5) | Re-evaluate past instances on every read | Re-evaluation rewrites history and double-counts with the corrections |
| 9 | **`held` in the register** (L §8.2) | No mark; or a warning per read | Q-13: a figure carried past K unchanged is a guess and must say so, without a warning per read |
| 10 | **`flows` of a flow pay is not a state read** (L §5.13.1) | Every history read is a state read | Law would need the day walk to read the year's advances; annual settlements would become state pays for no reason |
| 11 | **Rounding to any positive step, seven modes, the mode an axis** (L §7.2) | Fixed decimal units only | 5 Rappen and whole-złoty bases are steps; an unstated mode is the commonest hole (B row 25), so it must be an axis |
| 12 | **No date literals in rules** (L §7.1) | Allowed, as small integers are | A date in a rule is law data in code: a law change would edit a formula (A-15) |
| 13 | **Draws carry a `plan` value** for look-aheads (L §5.14) | (a) Look-ahead counts drawn values; (b) look-ahead skips draw-dependent flows | (a) is foresight on every path (R-04). (b) silently treats drawn spending as 0. The household's planning figure is their stated sentence (R-05) |
| 14 | **Same-day `flows` totals and groups of places** (L §5.13.1) | A memo place counting contributions; a sum of per-place lows | A memo place is a second kind of thing (M-02, rejected). A sum of lows is a different, wrong number |
| 15 | **The importer's rules are data** (`account_place`, `party_place`, `match_days`, `match_amount`) | Rules inside the importer's code | As data they have sources, the household confirms them once, and changing a tolerance is an addition |
| 16 | **Look-through by a bounded depth recurrence** (L §3.6) | Open recursion over entities; a fixed point | Open recursion breaks totality; a cross-holding has no finite answer, so a `check` compares the last two depths |
| 17 | **An exchange is two pays** (L §5.9) | A multi-currency pay | Conservation per currency holds per pay; the rate per leg is explicit |
| 18 | **The look-ahead is a segment tree** (P §2.3) | A sparse table | F changes as the cut rises during the walk; a sparse table would be rebuilt each time |
| 19 | **The gate has spec L-30's steps**, planted errors included (O §3) | Checks alone | Independent figures alone caught 75% of planted errors (L-32); coverage and mutants find the untested parts |
| 20 | **Pins `law` and `known` take expressions** (L §5.10) | Date literals only | Rolling origins need `as if {known origin_day[o]}` over an index (E §8) |
| 21 | **Matches the importer is sure of are written by the household's confirmed rule** (O §2.2) | Every match confirmed one by one | P-08 links exact matches automatically. A rule the household confirmed once keeps "drafts count only once confirmed" |

**Decided in D6**, from the round-1 gaps (`gaps/resolution.md`, `gaps/lead-household.md`):

| # | Decision | Alternatives weighed | Why this one |
|---|---|---|---|
| 22 | **A state pay instance is never re-measured** by a later version that isn't a `correction`; `arrears` covers flow pays only (L §5.13.3, E §5; L17) | Re-measure every booked instance, state pays included (D5) | A retroactive law made every look-ahead decision since its start a `HOLE-CHANGED`: loud and wrong. A decision is taken with what was known that day |
| 23 | **A def that reaches a state read is evaluated at each reader's walk point** (L §5.13.1, E §3.2; L9) | Memoise it per cell, as other defs | Two state pays on one day reading one `room` def would invest the same money twice, in silence |
| 24 | **An instance's `on` is read at the ask's cut; its `when` and amount at min(t, K)** (L §5.13.3; L18) | Read all three at min(t, K) (D5) | The cut needs t, and t came from `on`: circular |
| 25 | **Three classes of reserved words**: keywords, built-in names (only before `(`), contextual words (L §2.2; G54, G1, L4) | One flat list of about 180 words (D5) | Every group renamed domain words (`age`, `rate`, `share`, `start`); the definition's own examples didn't load |
| 26 | **Recurrences at later keys, and at computed keys with `RUN-CYCLE`** (L §5.10; G14, G35) | Earlier keys only (D5) | Annuities are computed back from the last age; an earlier acquisition is found by `where`. Termination still holds: one direction per index, and a chain is no longer than the keys |
| 27 | **A pin's value is any expression, evaluated unpinned** (L §5.10; G55); `law D for [x, y]` (G64; replaced by the wording in force on D, decision 34, and removed in D7: no case used it, `coverage.md` R1) | Literal pins (D5) | "As if the parts were the base parts" and "the scale in its 2022 wording" are questions the law asks; evaluating outside the `as if` keeps them free of cycles |
| 28 | **Band rows hold any literal and read their own ends; `marginal`; keyed bands** (L §5.8; L16, G4, G45, G63) | Literal-rule band rows; edges restated as inputs (D5) | "10 800 + 32% over 120 000" wrote the edge twice and nothing checked they agreed (quality 2) |
| 29 | **A bound ranks between a refusal and a hole, and may appear anywhere in an ask; a hole is a fact a row could give, a bound a step no row can give** (L §5.17; G39, G52) | Bounds only as a whole ask body (D5) | A comparison with a bound is a question about the bound; data a row could give must stay a hole, so a later row fills it |
| 30 | **`covered(x, span)` guards every aggregate over a movement input** (L §5.18; L2) | Sum what is there | A month with no statement summed to 0: a silent undercount |
| 31 | **A part is a law part only when its header says `law`; the gate skips hole rules** (L §6.1, O §3; L19) | Law parts by directory (D5's layout) | The engine checks only headers; an honest stub was refused by the coverage step |

**Decided in D6 round 2**, from the round-2 gaps (`gaps/resolution.md` § Round 2):

| # | Decision | Alternatives weighed | Why this one |
|---|---|---|---|
| 32 | **Known dates follow a fixed order of evidence** (L §4.5; H1, H10, H11): law from the publication that binds it, else its assent or making; the act that gave the wording; other sources from the earliest day they surely existed in the form read | (a) the modeller's judgement per source; (b) always the retrieval day | (a) gave eleven groups eleven readings of "publication", "updated stamp" and month dating. (b) hides every source from views before the fetch. The earliest sure day never shows a text too early, and dating a row by the act that gave its wording dates it once |
| 33 | **Back pay may be answered by a later version, `arrears … of V`** (L §5.13.3, E §5; H2) | (a) add the line to V once the answer is known (the old `HOLE-CHANGED` fix); (b) answer it with a `correction` | (a) breaks §6.2: a confirmed version is never edited, and a cut between the two dates would see an answer no one knew. (b) makes the answer known all along, the same error |
| 34 | **`enters` and the `wording D` pin** (L §4.3, §4.6, §5.3; H17) | (a) `law D for` alone (round 1, decision 27; removed in D7); (b) `known D for [x]`, a cut for named items | (a) reads the latest-known version governing D, so a retroactive act known before D but in force after it beats the text in force on D (UL-15, art. 26 ust. 3). (b) still passes a text published before D and not yet in force. Entry into force is a third date, apart from governs and known |
| 35 | **`moves V to SPAN`** (L §5.3, §4.6, E §3.1; H18) | (a) a new version per postponement, copying the rules (RC-01: three postponements of about 15 rules); (b) edit V's span | (a) copies law, which then drifts apart. (b) breaks §6.2, and an earlier cut would see a date no one knew yet |

**Decided in D6 round 3**, from the round-3 gaps (`gaps/resolution.md` § Round 3):

| # | Decision | Alternatives weighed | Why this one |
|---|---|---|---|
| 36 | **An imprecise `at` or `on` reads one law date per law-date class its period overlaps; an `at` that is a hole makes a hole** (L §4.3, E §3.2; I3, I4) | (a) the period's first day (round 2); (b) both ends of the period | (a) picks the older law in silence when a governs start falls inside the period (MP-08: deaths "in 2015" against regulations from 2015-06-12). (b) misses a change inside the period that later reverts. The classes are what the memo is already keyed by, so the cost is one evaluation per class, usually one |
| 37 | **A stand-in is a version marked `assumed`, dated as the text it stands for** (L §4.5, §5.3, §8.2; I1, I2) | (a) rows marked `assumed` only (round 2); (b) the later text's dates | (a) leaves the rules that point at the stand-in out of every register (MP-16: Lines 5 and 10, the rounding, the minimum order). (b) hides the earlier text from every view before the later one, though it was public: `RUN-NO-RULE` where an assumed rule belongs |
| 38 | **Guidance is a dated issue; a document kept up to date is an other source, dated by its earliest public stamp (a printed date, a server date, a dated capture); a file's own stamp only where no public stamp exists, marked `assumed`** (L §4.5, §5.3; I5, I6; D8) | (a) every publication of an authority as guidance; (b) the earliest sure stamp, file records included (D6 round 3 to D7) | (a) gives no date to a leaflet or form instructions with no release notice (LH-14, CB-13, CB-14). (b) can show a text a few days before its public release, since a file is saved before it is posted. **The user's answer (D8, 2026-10-19): public stamps only, with a visible fallback.** The fallback's date is in the register, so a view that rests on it says so |
| 39 | **A commencement "to be fixed, at the latest D" is a `moves` to D** (L §5.3; I11) | (a) a hole start; (b) the original start until the fixing act | A governs end is a literal (§7.1), so (a) can't be written. (b) applies the act early under cuts before the fixing act (RC-05, décret 2019-1574). The latest date never shows law too early |
| 40 | **A `stated` balance re-anchors the walk** (E §4.2, §6.1, §6.3, §9; L §5.18; J70, D8) | (a) no re-anchoring: a balance is the opening plus everything booked (D5 to D7); (b) re-anchor and drop the difference | (a) lets one hole movement in an uncovered past spoil every later balance, through every statement (the household, D7). (b) hides the drift. Re-anchoring keeps stated − booked on the ledger's balance line, one line per statement, against the last anchor. **The user's answer (D8, 2026-10-19).** And every figure that reads an anchored balance lists the anchor, with its difference, in its register (`anchor:`, L §8.2): the user's answer to REPORT §13's question (clean side, 2026-10-19) |

---

## 6. Departures from the spec, and the questions they raise

Each departure is above with its reason. Those that touch a decision the user made, or a requirement where the evidence points another way, went to `REPORT.md` as questions. **All five were answered on 2026-10-19 (D8), each "yes" to the recommendation; the rows above carry the answers** (REPORT §12):

1. **M-22, composite templates.** The user decided that one block per real thing should expand into its wired pieces. In Tally a new thing is rows, and the law parts find it by query, so nothing expands (L §6.5: a new job is 8 added lines, no edits). Is "rows plus derived wiring" the "one block" you meant, or do you want a visible expansion step?
2. **R-06, other rules' transfers in the look-ahead.** Tally's look-ahead counts planned flows only. A rule never sees another rule's future result, which is what keeps the walk free of fixed points (spec §4#1). A transfer planned in advance is written as a flow pay, and then it is counted. Is that enough?
3. **T-11 and A-11, relative dates.** The engine computes every date, through its calendar built-ins. Mapping a Polish phrase to a built-in is the MCP layer's job, and the row keeps the phrase. Is a phrase map outside the engine acceptable?
4. **C-08, a payment into a debt capped at its balance.** Tally refuses the payment instead (`RUN-PLACE`), since a cap hides an error; a model that wants a cap writes it. Refuse, or cap?
5. **Q-03, when "N in 1 000 futures" may be said.** Left to the MCP layer, with the engine supplying the standard error and the register. Should the rule be the engine's?

Departures that touch no user decision and need no answer: M-24 (coverage checked per value, not at load), T-16 (S is the header's `start`), U-12 (the estimator writes its own conversion), A-06 (no macros), M-02 (no memo places), P-12 (no automatic pad).

---

## 7. Constructs and the cases that need them

From the models (D7, 2026-10-19): `coverage.md` counts every construct and built-in word over the 79 models, the household and the seven changes. The D5 version of this table listed the cases from the case files before the models were written; where the models differ, the last column says so. `coverage.md` §3 has the constructs no case uses, each removed or kept with its reason.

| Construct | LANGUAGE | Cases (models) | Note |
|---|---|---|---|
| header, source, version, input, index, def and rule, ask | §5.1–5.3, §5.6, §5.7, §5.10, §5.16 | all |  |
| a row with its own `governs` | §4.6 | CB-03, CH-05, CMP-01, LH-05, LH-07, MP-09, MP-11, RC-03, RC-08, UL-05, UL-06, UL-08, UL-13, the household, chg1, chg4, chg5 |  |
| `enters` (entry into force) | §5.3 | RC-08, UL-04, UL-15 |  |
| `moves` | §5.3 | RC-01, RC-05 |  |
| `arrears` | §5.13.3 | LH-14, the household, chg3, chg4 | planned: LH-14, UL-09, RC-08, UL-06 |
| `correction` versions | §4.5 | CH-04 | planned: CH-04, LH-05 |
| unit | §5.4 | CB-06, CH-02, CH-04, CH-06, LH-01, LH-02, the household |  |
| axis | §5.5 | CB-02, CB-03, CB-05, CB-06, CB-07, CB-10, CB-11, CB-12, CB-14, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-02, LH-03, LH-04, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-09, MP-11, MP-12, MP-13, MP-14, MP-16, RC-01, RC-02, RC-04, RC-05, RC-08, UL-01, UL-02, UL-03, UL-05, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-14, UL-15 |  |
| `index … assumed`, `index …: open` | §5.6 | CB-14, the household |  |
| band | §5.8 | CB-01, CB-02, CB-03, CB-04, CB-07, CH-05, CMP-01, MP-05, MP-06, MP-08, MP-09, MP-10, MP-11, MP-16, RC-07, RC-08, UL-01, UL-03, UL-07, UL-09, UL-10, UL-15, the household |  |
| place | §5.9 | CMP-01, the household, chg5 | planned: also CMP-02, MP-14, MP-15, LH-15, LH-16, MP-01, MP-04, CB-10 |
| `excepts` | §5.10 | CB-02, CB-03, CB-06, CB-07, CB-09, CB-11, CB-13, CB-14, CH-02, CH-03, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, the household |  |
| recurrences | §5.10 | CH-02, CH-06, LH-02, LH-04, LH-05, LH-07, LH-10, LH-13, LH-15, LH-16, LH-18, MP-15, RC-01, the household |  |
| `as if` and `with` pins | §5.10, §5.16 | CB-03, CB-07, CB-09, CH-04, CH-05, CMP-01, LH-03, LH-04, LH-06, LH-12, LH-15, MP-03, MP-04, MP-05, MP-07, MP-08, MP-10, MP-11, MP-16, RC-03, RC-05, RC-06, RC-07, RC-08, UL-01, UL-03, UL-11, UL-13, UL-14, UL-15 | planned: 18 models |
| `missing()` | §5.10 | the household | planned: CH-01 |
| fill | §5.11 | CB-05, CB-09, LH-01, LH-02, LH-04, LH-06, LH-09, MP-15, UL-13 |  |
| prorata | §5.12 | CMP-01, MP-01, MP-09, MP-12 |  |
| pay | §5.13 | CMP-01, the household, chg5 | planned: 10, the others hold their money as defs |
| state reads (`bal`, `plan_low`, `worth`) | §5.13.1 | the household, chg5 |  |
| `flows` | §5.13.3 | CMP-01, the household |  |
| draw | §5.14 | none | planned: LH-17, LH-18, CH-06, MP-14, CMP-02, CH-03, CH-01 (coverage.md K13) |
| exact distributions | §5.14 | CH-01, CH-03, LH-17 |  |
| scores | §5.15 | CH-03 | planned: also CH-01, CH-02, CH-06, which evaluate with exact distributions and arithmetic (backtest zones, actual-to-expected ratios), not §5.15 scores |
| path statistics | §5.16 | none | planned: LH-17, LH-18, CH-06 |
| `solve` | §5.16 | LH-10, LH-11, LH-17, MP-07, UL-08 |  |
| `check` | §5.16 | CB-01, CB-03, CB-05, CB-08, CB-09, CH-02, CH-04, CH-05, CH-06, CMP-01, CMP-02, MP-02, MP-09, MP-12, MP-13, MP-14, MP-15, MP-16, UL-03, UL-04, UL-08, UL-11, UL-13 | planned: CH-06, the household |
| `explain`, `inputs-of`, `residuals` | §5.16 | CB-07, CMP-01, the household |  |
| bound | §5.17 | CH-03, LH-17, MP-15 | planned: also LH-18 |
| records | §5.18 | the household |  |
| scenario | §6.4 | chg7 | planned: LH-10, CB-01, LH-04, UL-10, MP-14, the household |
| transcendentals `to u` | §7.3 | CH-01, CH-03, CH-06, LH-10, LH-11, LH-15, LH-16, LH-17, LH-18, MP-07, UL-08 |  |
| rounding | §7.2 | CB-02, CB-06, CB-10, CB-11, CB-12, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-02, LH-07, LH-08, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-03, MP-04, MP-07, MP-09, MP-11, MP-12, MP-13, MP-14, MP-16, RC-01, RC-04, RC-05, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-15, the household, chg5 |  |
