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
| `plan_low(x, a, b)` | `low(x, a .. b) from t`, t the state pay's day (the written projection) | manual: the projection counts more than flow pays | household | D-6 |
| `promised` | an `uncertain` event kind | manual | household | D-6 |
| `next_on(p, d)` | `next(K, d)` | automatic | none used (K7) | D-6 |
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
| a state pay ordered before a flow pay of its day, or a statement balance read right after a flow pay or an observation | refused (`LOAD-ORDER`): plans and observations come first on their day, so `before m` and a reading `after m` name only a decision (L §4.6, §5.14.2); the start-of-day balance is `bal(x, d start)` | manual: **a change of meaning** | to count | GD-10, GD-11 |
| `flows` over a span that reaches before `start` | a hole, `HOLE-PRERUN`, whatever movements are dated there (L §4.1, E §2) | manual: **a change of meaning** | to count | GD-5 |
| a day a source fixes in every year (two int rows under A-49) | an input of type `monthday` (`MM-DD`), read in year y by `day_of(y, md)` (L §3.1, §4.2) | none (new form; the two int rows still work) | `Q:A-04` and any like it | GD-2 |
| `at k`, k a key of a range `by n` with n above 1 | refused, `LOAD-LAWPERIOD`: such a key is the n periods it stands for, not a day (L §4.2, §4.3); write `at start_of(k)` or another day | manual: **a change of meaning** | to count | GD-20 |

Every ask either prints the same or its change is listed in `regress.md` (phase 5).
