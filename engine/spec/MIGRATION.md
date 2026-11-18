# Migration from tally 1.0.1 to tally 2

Started in phase 3 of the design round (2026-10-25); this file is the one home of the old-to-new table. Each 1.0.1 construct that changes is sorted three ways (brief, phase 4): **automatic** (a rewrite rule that `tools/migrate.py` applies, with the same answers by construction), **warned** (automatic, but the writer checks one named thing), or **manual** (the meaning changed; the models and asks it reaches are listed). Reach is from `engine/spec/coverage.md` (the 79 models, the household, chg1–chg7). Constructs not listed are unchanged (`core.md`, "Kept unchanged").

| 1.0.1 construct | Tally 2 form | Class | Reaches | Record |
|---|---|---|---|---|
| comparison `=` in an expression | `==` | automatic | nearly every model | D-14 |
| `-` as a value or a test | `none` | automatic | 19 models (value), 16 (test) | D-7 |
| version or row without `governs` | `governs always` | automatic (the tie rule and `held` are unchanged for `always`) | 36 models | D-3 |
| def relying on L §4.3 rule 3 (the reader's law date) | `at reader` | automatic | most models (H-1: 1 092 of 1 454 defs) | D-4 |
| time-keyed def relying on rule 2 | `at start_of(k)` | automatic | models with time-keyed defs and no `at` | D-4 |
| ask relying on rule 4 (law date K) | `law <the model's on date>` | warned: the answers are the same today and stop moving when `on` advances; the writer checks this was meant | dateless asks | D-4 |
| `rule r excepts a, b` | `rule r beats` the transitive closure of `a, b` written as pairs | warned: every pair the closure adds (LC-2's chain links) is listed for the writer to check against the source | 67 models | D-12, C-19 |
| a later rule reusing a label | `replace rule d.r …` | automatic | models with rule versions | D-12 |
| version row aimed at a def | `given …` | automatic | 14 models | D-12 |
| rate type with a basis (`rate nominal 12/year`, …) | `rate` with a `compounding` code | automatic | LH-10, LH-11, CH-05, LH-18 | D-12 |
| `dcf` day-count words (`act365`, …) | `daycount` codes | automatic | none used (K1) | D-12 |
| record value (positional tuple) | named record | automatic, by the type's field order | LH-10, LH-13, LH-14, MP-13, RC-02, household, chg6 | D-14 |
| bare literal 100, 360, 365, 366 or 1000 in arithmetic | a unit, a code or a sourced input | manual, per occurrence (counted in phase 5) | to count | D-14 |
| any inline figure but 0, 1, −1 and key offsets in a def, rule, move, fill or band formula, day numbers and dates included (`day_of(m, 15)`) | an input with a row in the model's facts | warned: the script lifts each distinct figure into one input row per meaning, named from its use; the writer checks the name and merges figures that are one fact (the household's six 15s become one row) | most models; the household (LC-1) | D-14, C-3 |
| `bound X "…" because "…"` | an ordinary hole whose question is the reason | automatic | CH-03, LH-17, MP-15 | D-15, C-20 |
| `explain` order | the stated order | warned: printed lines may reorder | CB-07, household | D-15 |
| `place x: account|cash|holding|asset|debt|claim|outside …` | a thing of the prelude kind of that name, plural (`accounts`, `cash`, `holdings`, `assets`, `debts`, `claims`, `outside`; unit `one`), entered with its unit and a lifespan: `accounts + [konto] PLN lives …`, or a `place` section | automatic for the kind; manual for `holds` | CMP-01, household, chg5 | D-1, D-2 |
| `holds h`, `free(x)` | an owed place, a place of a kind with a sign rule (L §5.10) | manual | household (`free`: none) | D-2 |
| `worth(x, d)`, `kind(x)`, `currency(x)` | `value(x, d) in CUR by p`, `x in K`, `unit(x)` | automatic | household | D-1 |
| index addition `X + [k]` of things | `K + [k] lives span`, X becoming a kind | warned: the writer states the lifespan (1.0.1's "from the version's date" is offered) | chg1, chg4, chg5, chg6 | D-1 |
| zero rows that start or end a thing (L §6.5 change 1) | a lifespan | manual | household | D-1 |
| `pay` | a one-leg `move` of the prelude kind `transfers`, with `reads` where 1.0.1 inferred a state pay (a state pay becomes a decision) | automatic | CMP-01, household, chg5 | D-2, D-5 |
| several pays of one event | one `move` with several legs | manual | household | D-2 |
| `phase n` and pay id order | `before`/`after` only on same-day cycles | automatic (1.0.1's order is one valid order of the graph) | household, chg5 | D-5 |
| `argmin`/`argmax`, `prorata rest largest`, `fill` order with possible ties | `ties by` | warned: the writer checks the tie-break against the source | 5, 3 and 9 models | D-5 |
| `arrears on D` (or `none`) | a settlement move on D of every adjustment of the pay, each `fulfils` its adjustment, its leg against the adjusted leg; `none`: a move of a legless kind that fulfils (waives) each adjustment | automatic | models with arrears lines, household | D-6, C-17 |
| `correction` version | a later version with the same `governs` and a later `known`, and a settlement move per changed instance dated on the instance's day | automatic | models with correction versions | D-6, C-17 |
| `plan_low(x, a, b)` | `min(bal(x, d) from t for d in a .. b step 1 day)`, t the state pay's day (the written projection) | manual: the projection counts more than flow pays | household | D-6 |
| `promised` | an `uncertain` event kind | manual | household | D-6 |
| `next_on(p, d)` | `(first(m in K over d ..) ties by m.day)` | automatic | none used (K7) | D-6 |
| records (movement input), `explains` | `observed` input with named fields, `fulfils` | manual | household | D-8 |
| `stated[x]` with a span | a reading `at D end`, D the span's last day, in a version with `anchor x` (1.0.1 re-anchored) | manual | household | D-3, D-8 |
| `opening[x]` | a reading `at S start` (an opening always anchors) | warned: the writer checks the instant | household, chg6 | D-3 |
| `prices` and price inputs read on other days | `reading` input with a stated carry | warned: the writer checks whether the input is a reading | CH-05, household | D-3 |
| `dates` holiday lists, business-day functions | a `calendar` value with coverage | warned: the coverage is the list's years | household | D-13 |
| `scenario` file versions | ordinary versions with `known` and `governs` | warned: the writer checks the values taken from the old defaults | chg7 | D-10 |
| `draw r[p in path, …] ~ D to u plan v` | an input or def of type `dist` whose distribution states `to u`; the planning figure as a query at the read (`random at mean` or `quantile(q)`); items keyed `p in path` lose that key | manual | `montecarlo/` (two models) | D-11 |
| distributions in `cdf`, `pdf`, `pmf`, `crps` | the same over `dist` values | automatic | CH-01, CH-03, LH-17 | D-11 |
| `share` verdict | `share` prints n of N, its standard error and the horizon; no verdict and no carry (L §5.16, E §8) | manual for any ask that prints a verdict | LH-15, LH-16 | D-11 |
| rule line `rule L … when c = v` | `rule L … = v when c` | automatic (a reprint) | 511 rules | D-14, C-18 |
| short def `def x: T at E = v` | `def x: T = v at E` | automatic | 18 defs | D-14 |
| aggregate `sum(b : t)`, `argmin(b : t)`, … | `sum(t for b)`, `argmin(t for b)` | automatic | 376 uses in 64 files | D-14 |
| `solve x in [a, b] to u: l = r` | `solve l == r for x in [a, b] to u` | automatic | 18 uses | D-14 |
| a model with no `start`/`end` | `start` and `end` written | warned: the script writes the year of `on` as the frame; the writer checks it | 46 of the 79 | D-13, C-21 |
| a four-digit range `2026..2030` | `year 2026..2030` or an integer range, as the index says | warned where the index is untyped (LH-14's ambiguity A1) | LH-14 and any like it | D-14, C-21 |
| a statement version over a covered place | the same with `anchor x` (1.0.1 re-anchors) | automatic | household, chg6 | D-8, C-11 |
| a model using `PLN` (or any currency) without declaring it | `unit PLN: currency minor 0.01` | automatic | every model with money | D-12, C-31 |
| a total over places relying on `outside` being left out | the same with `where not x in outside` | automatic | household, CMP-01 | D-1, C-9 |
| reads of a movement's legs through defs (1.0.1 inferred) | `reads m` on the reader | automatic (computed from 1.0.1's inference) | household, chg5 | D-5, C-5 |
| `match_days[p]`, `match_amount[p]` keyed by pay | keyed by place (L §5.19); then dropped with the importer's other built-in ids (W-4, after G1): no text uses them | manual | household | D-8 (phase 4 log 21), W-4 |
| a base item naming a scenario's item, or a rule's `v` naming a scenario version (1.0.1's N-931/L-24 allowed it) | refused `LOAD-SCENARIO`: the base loads and answers without its scenarios, and an act not yet law brings its rules in the scenario's own version (L §6.4) | manual: **a change of meaning** | none of the 79 models, the household or its changes (none prints `LOAD-SCENARIO` in the snap12 reruns); the SCEN probes were written for it | A-36 (G2 list) |
| `crps(x for p in path, y)` | `crps(x, y)` in a paths ask | automatic | none of the 79 | D-11 |
| `with u(z[k]) = q`, `with z[k] = v` on a draw | `with sample z[k].u = q`, `with sample z[k] = v` | automatic | `montecarlo/` | D-11 |
| a `round` step other than 1 or a minor unit in a def (`round(x, 0.05, …)`) | `minor(CUR)` when it is the minor unit, else an input row | warned, as the literal rule's row above | to count | D-14 |
| a def keyed by a range that tally 1 wrote as an index of steps (`argmin` over an index of cents) | unchanged; `min(v for v in a .. b step s where c)` is offered | none (optional rewrite) | Q-B-07-like models | D-15 |
| a `regime R` block (a tally 2 design-round text; 1.0.1 has none) | its items at the top level: the `regime` line, its `export` line and its own notes go, the items are dedented one level, and `R.x` becomes `x`, or `R_x` where x clashes with a top-level id or another regime's item (`scratch/lead/round19/unbox.py`) | automatic | the blind texts I-6, I-9, I-10 and I-12 | GD-1 |
| a state pay ordered before a flow pay of its day, or a statement balance read right after a flow pay or an observation | refused (`LOAD-ORDER`): plans and observations come first on their day, so `before m` names only a decision (L §5.14.2); a statement balance is stated `at D end` (L §4.6); the start-of-day balance is `bal(x, d start)` | manual: **a change of meaning** | to count | GD-10, GD-11 |
| `flows` over a span that reaches before `start` | a hole, `HOLE-PRERUN`, whatever movements are dated there (L §4.1, E §2) | manual: **a change of meaning** | to count | GD-5 |
| a day a source fixes in every year (two int rows under A-49) | an input of type `monthday` (`MM-DD`), read in year y by `day_of(y, md)` (L §3.1, §4.2) | none (new form; the two int rows still work) | `Q:A-04` and any like it | GD-2 |
| `at k`, k a key of a range `by n` with n above 1 | refused, `LOAD-LAWPERIOD`: such a key is the n periods it stands for, not a day (L §4.2, §4.3); write `at start_of(k)` or another day | manual: **a change of meaning** | to count | GD-20 |

Every ask either prints the same or its change is listed in `regress.md` (phase 5).

## 2.0 → 2.1

tally 2.1 is opt-in (`CHANGES.md`, 2.0 → 2.1). A 2.0 text declares `tally 2`, as a 2.1 text does, and loads and means the same on 2.1, except as the rows below say.

| 2.0 construct | 2.1 form | Class | Reaches | Record |
|---|---|---|---|---|
| a text that declares an item named `parties` | renamed: `parties` is a prelude kind, a built-in id (`LOAD-DUP` otherwise) | manual | probes OWN-04 and OWN-05, and L §3.6's own example (a grep over every `.tally` under `engine/`) | R-2, G1 Q-1 |
| a kind named `owned`, `batched` or `takes` | renamed: kind clause words (`LOAD-RESERVED`'s positions, L §2.2) | manual | none found | R-1 |
| a subkind's field read where only a kind test says the variable is of that subkind, refused before (`LOAD-FIELD`) | loads, and reads the field (L §5.7) | none (a refusal becomes a value) | probes whose stored output is that refusal | R-17 |
| a day key where a date is expected, refused before (`LOAD-UNIT`) | loads, meaning `start_of(k)`; `start_of(k)` stays valid (L §3.1) | none | the visible cases write `start_of(k)` | R-14 |
| a one-line def followed by `note` lines, a parse failure before | loads, and prints as the long form (L §2.4) | none | none (the long form stays) | R-11 |
| a record row with a negative field, refused by the interpreter | loads, as the grammar always allowed | none (an interpreter fault) | none visible | R-12 |
| a house rounding written only to keep a path statistic under 4 096 digits | may be dropped (L §7.1) | manual, optional | D-13 | R-13 |
| a split by owner or by lot written by hand (`owns` rows per place, a fill per holding, a def that subtracts one party's money) | an `owned` or `batched` place kind with its arrival rule and `takes` clause; its `owns` rows go (L §3.6) | manual, optional, per text | households A and E1, in session 7b | R-1 to R-10 |

Every 2.0 text that uses none of these prints byte for byte as on 2.0 (the round's hard rule 5, checked by the suite).

## 2.1 → 2.2

tally 2.2 is a new version of the language (`CHANGES.md`, 2.1.8 → 2.2): the user cut nine forms on 2026-11-17. A 2.1 text declares `tally 2`, as a 2.2 text does, and loads and means the same on 2.2, except as the rows below say. A text that still writes a cut form is refused, by the grammar (`LOAD-PARSE`) unless its row names another form.

| 2.1 construct | 2.2 form | Class | Reaches | Record |
|---|---|---|---|---|
| a leg's own day, `leg l from A to B amount q on d`, in a move or an observed row | every leg is on its movement's day; cash that lands later is a second move keyed by the first, `move paid[x in sales over ..]: …` with `on add_days(x.day, lag)`; on a statement, its own row | manual | no household, model, blind text, amendment or probe card; T's vectors | 2.2-6 (S-6) |
| a converted total, `flows(x, span) in CUR at fx mode` | a def keyed by the movement that rounds it at its day, `def in_cur[t in K]: money CUR = round(t.l.amount * fx, minor(CUR), mode) at t.day`, summed: `sum(in_cur[t] for t in K over span where t.l.to == x) - sum(in_cur[t] for t in K over span where t.l.from == x)` | manual | model CMP-01 (`net_flow`) | 2.2-7 (S-7) |
| a place kind of unit `any` that isn't `outside`, `kind brokerages: place any min 0`, with readings keyed by unit and `bal(broker, n7, d)` | one place per unit under a kind of unit `one`: `kind brokerages: place one min 0`, `ordinary + [broker_n7] n7 lives …`, read `bal(broker_n7, d)`; a fill over the kind needs no unit test in its arc. The kind is refused `LOAD-PLACEKIND place kind <K> is of unit any and isn't outside`; a balance reading of an `outside` place of unit `any` is refused `LOAD-UNIT` (a reading of an `outside` place of a stated unit loads as before) | manual | no household, model, blind text, amendment or probe card (their broker accounts are of unit `one`); L §5.7's example; T's vectors | 2.2-1 (S-1) |
| a balance reading `after m`, `stated[konto] = 900.00 PLN after pay[2026-01] "s"` | `at D end` (`at D start` for the balance before the day's movements): a running balance printed row by row is stated once a day, as its last row's balance. `at D` alone is refused `LOAD-INSTANT … needs start or end` | manual | no household, model, blind text, amendment or probe card (all 336 real `stated` rows are `at D start` or `at D end`); T's vectors | 2.2-3 (S-3) |
| a `known` window, `known window(2026-10-15, 2026-11-01)` on a version or a row, read seen and unseen at a cut inside it | `known 2026-11-01`, the range's last day, with a note saying so (L §4.5: the latest it can be); the earlier reading is a scenario whose version is `known` on the range's first day, compared with `diff(e) in early against base`. A `known window` is `LOAD-PARSE cannot parse: expected a day, got window` | manual | no household, model, blind text, amendment or probe card; T's vectors | 2.2-2 (S-2) |
| a version's `same` line between two observed rows, known after the rows | the same text; the line has no day of its own and holds at every cut that sees both rows (L §5.19): asked between the rows' `known` and the line's, the two rows are one movement where 2.1 counted two, and a row whose twin the cut doesn't see is read alone where 2.1 refused (`RUN-RECORD same <p> = <q>: <q> doesn't exist`). A line with a planned instance on a side keeps its version's `known` | manual: **a change of meaning** | none: each of the 23 real lines sits in the version of one of its rows; T's vectors | 2.2-5 (S-5) |
| `same a.fee[2026-03] = b.fee_in[2026-03]`, two rows of an input, in a header or a version | inside one model, a def that reads both inputs and states their disagreement, `rule clash beats agreed = ?"…" when fee_bank != fee_broker`; in a joint model, a pin, `with a.fee[2026-03] = b.fee_in[2026-03]` on the ask or `as if {a.fee[2026-03] = b.fee_in[2026-03]}` in a def of the joint model (L §6.1, §5.18). The line is `LOAD-SAME same <p> = <q>: a row of an input and a row of an input` | manual | no household, model, blind text, amendment or probe card (PTY-05's line joins an observed row with a movement and stays); T's vectors | 2.2-8 (S-8) |
| `low(x, a .. b) [from t] [clauses]` and `next(K, d)`; `explain low(…)` | `min(bal(x, d) [from t] [clauses] for d in a .. b step 1 day)`, d a name the item doesn't use, the clauses on the `bal` read; `(first(m in K over d ..) ties by m.day)`; the lowest balance explained in two asks, its day by `argmin(bal(x, d) from t for d in a .. b step 1 day) ties by d` and then `explain bal(x, <that day>) from t`. Every value as before. A text that writes them: `low(…)` is `LOAD-PARSE cannot parse: expected ")", got ..`, `next(…)` is `LOAD-NAME unknown id next`, `explain low(…)` is `LOAD-PARSE cannot parse: expected end of line, got (` | automatic | the probe cards RULE-02, OWN-02 (`low`) and RULE-05 (`next`), byte-identical respelled; T's vectors and generator | 2.2-9 (S-9) |
| a line that states `windows each`, on an ask line or a projection read, `ask q: bal(konto, 2026-03-31) law 2026-03-31 windows each` | the plain line, whose answer is the hole `HOLE-IMPRECISE` where the day matters, its register naming each unknown day (`unknown day: w`); to take the window as one day, pin the cell that holds the unknown day itself, never a value computed from it, one ask per day: `ask q_d07: bal(konto, 2026-03-31) with w = 2026-03-07 law 2026-03-31`. A window in a calendar's `added` list is one scenario per choice (`replace holidays = calendar(…, added [2027-04-12])`). A window the caller is to pin is the value of a row or a def, so that it has a cell; a window written in place (`on window(a, b)`) becomes an input's row | manual | household A (A-05, A-13), B (B-07, B-12); T's vectors | D-7, SF-6, SF-23 |

Every 2.1.8 text that uses none of these prints byte for byte as on 2.1.8, but for the lines the round names: the register's `unknown day:` line under a hole that carries an unknown day (2.2-11), the `clauses:` line of a projection's `explain` (2.2-4), `LOAD-INSTANT`'s words (2.2-3), and the lines of a def keyed by a movement, which now split by an axis or take a pin (2.2-10).
