# The Tally engine's architecture

The map of `engine/crates/`, for the owner, who audits the engine through this page, each
file's `//!` header and the trace from code to spec (`engine/tools/trace --map`). It names
modules and never links them. Written at milestone S (2026-11-02), brought to M0
(2026-11-05) and to M1 (2026-11-11); updated only at a milestone (R-119). The spec is the
authority: L is `spec/LANGUAGE.md`, E `spec/ENGINE.md`, P `spec/PERFORMANCE.md`. The twin of
each file, refcore's file and functions it mirrors, is on its header's `Twin:` line; the
load tables below repeat the main ones.

## Bird's-eye view

The engine is a pure function. A reality (Tally text files) and a request go in, as bytes;
the printed answer comes out, the same bytes refcore prints (DONE gate 10). Think of it as
a compiler and an interpreter in one, with one twist: time. Every value is read "as known
at a cut, under the law of a law date", and the world moves day by day.

A run is a pipeline:

1. **read** the files, in the order of their full paths;
2. **split** them into lines and read each file's header first: is this one model, a model
   with scenarios (each loads as its own *world*), or a *joint model* whose `use` lines name
   member models (each found, pinned by a hash and loaded alone, then merged);
3. **parse** each file into a tree (`syntax`);
4. **load** the trees into one resolved, checked reality (`load` building `model`): names,
   units, kinds, indexes, versions, declarations, rows, then **the checker**, which types
   every expression and records who reads what, then the checks over that graph. A load
   refusal is printed and the run stops; in a reality with scenarios, a scenario's refusal
   stops only that scenario;
5. **answer** each ask (`ask`), which pulls values on demand: the evaluator (`eval`)
   computes def cells into the memo, reads inputs through the read rule (`read`), and asks
   the walk (`walk`) for balances, flows, values and projections; the walk books legs,
   observations and readings (`book`); numbers are exact (`num`); a missing fact is a hole
   (`hole`). Every value is computed in a *context*: a law date, a cut, the pins in force,
   the world the ask is in and, for an ask over axes, one combination of their options;
6. **print** the lines, the register under each, then the warnings.

A second entry, **`reprint`**, stops after step 3: it prints each file in the canonical
format (L §2.4), the one spelling every reality has, then the reality's hash, the sha256 of
those texts in path order. A joint model's members are pinned by that hash.

Nothing is computed ahead of a read: an ask reads a def, the def reads a balance, the
balance needs the walk up to its day, the walk needs the plan's amounts, which are defs
again. The memo and the walk's own caches make each piece computed once per context.

**What M0 changed:** at S, load did only what the evaluator needed to run X's base and
answered "not implemented" on any refusal; at M0, load is the whole of L §8.3's catalogue,
in refcore's order, and the parser and the formatter cover all of L §2.

**What M1 changed:** at M0, the run past load was S's. At M1 it answers the household
forecast: the asks of L §5.18 but the random ones (values, `check`, `diff`, `solve`,
`explain`, `inputs-of`, `readers-of`); pins, `as if`, worlds and axes as layers of one
context; the written projections (`bal(x, d) from t`, `low`, `next`, `value`, `flows`);
decisions in their written order; windows and the calendar built-ins at run; `prorata`.
Between the two, the pre-M1 run gave names one interner and tables typed ids, and split the
longest files along the spec's steps; the map below names the parts.

## Codemap, by pipeline stage

Paths are under `crates/tally/src/` unless they say otherwise. **K** marks the kernel, the
code DONE gate 8's mutation score covers (`tools/kernel.toml`: `num/`, `read/`, `walk/`,
`book/`, `hole/`). Pre-M1 split `walk/plan.rs` and `walk/reads.rs` into five files; M1 added
three kernel files: `num/split.rs`, `walk/value.rs` and `walk/lows.rs`.

**The way in.**
- `lib.rs`: the library's face; each module block names its files and its spec section.
  Three entry points for the fuzz targets: `parse` (one file's bytes to trees) and
  `format_file` (one file reprinted) here, `load_report` (load alone) from `api.rs`.
- `api.rs`: `run`, `run_in` (`run` with the disk's member files, the CLI's entry),
  `run_with_memo` and `run_with_lows` (fence F5 on or off, for its equivalence tests): read,
  split, load the reality or its worlds, answer (E §1; L §8.3 for which refusal prints
  first); `reprint`; `load_report` (what load prints when no ask is answered). `run_faulted`
  exists only in the `fault-injection` build: a run with a planted fault, so gate 5's
  `ENGINE-INVARIANT` fixture has a text that reaches it.
- `oracle.rs`: the walk's day source: the event walk, or every day (the test oracle).
- `counters.rs`: the `--stats` counts, in the `counters` build only (BUILDER §13.2).
- `invariant.rs`: E §9 items 3 and 4: a scenario changes nothing before its `from`, and asks
  move no money (each main walk's signature, kept once it is walked and checked again after
  each ask).
- `crates/tally-cli/src/lib.rs`: the `tally` command as a function, `run(args, files)`:
  arguments, paths, exit codes 0/1/2 (INTERFACE §1–§4); `tally reprint`, its files named as
  given and taken by full path (INTERFACE §1, issue #60). It re-exports the library, so T's
  tests can call `run_with_lows`, and `run_faulted` in the `fault-injection` build.
- `crates/tally-cli/src/main.rs`: the binary; the one file that reads files and writes
  output. It also reads a joint model's members from the disk, beside the files it is given,
  round by round (INTERFACE §2): the library never touches the disk.

**Foundations** (`base/`). `code.rs`: the catalogue's 101 codes (L §8's 97 and E §11's
four), one enum. `refusal.rs`: a refusal carries its code, message and place, and its
*reach*: what it names that a later load rule acts on. That is an ask naming a world the run
doesn't load, which is refused alone, or the id an `unknown id` or `ambiguous key` refusal names,
which a scenario's id read outside it renames `LOAD-SCENARIO` (L §6.4). The reach is set where
the refusal is made, so no rule reads it back from the message's words. A member's refusal is
printed `member a: …`. `symbol.rs`: the interner, each distinct name's text kept once as a
small number; nothing is ever ordered by that number. `table.rs`: the one store of items,
`Table<I, T>`: items in the order they were added (the reality's section order), found by
their typed id or their name with a checked lookup, never by indexing.

**Text** (`syntax/`, the lexer-parser area, exception budget 0).
- The parser: `lex.rs` (lines, indent levels, tokens; each token keeps the role the parser
  gave it, for the formatter), `parse.rs` (the cursor over one line; nesting counted to
  L §2.3's depth of 8), `expr.rs` (expressions by precedence climbing: postfixes,
  references, atoms, records, legs, types), `binds.rs` (spans, bindings, `ties by`),
  `calls.rs` (the built-in calls and the reads with clauses: `bal`, `value`, `flows`,
  `covered`, the aggregates, `calendar`, the day sets, `price`), `reads.rs` (`low`, `crps`,
  queries, distributions), `ask.rs` (ask lines, pins, `as if`, `solve`), `item.rs` (the
  file: header, sections, a version's items and rows; `as if` nested at most 4 deep),
  `decls.rs` (the declaring sections, `source` to `move`), `tree.rs` (the tree, one type
  per production). The parser reads every form of L §2.3; a text it can't read is a coded
  load refusal (`LOAD-PARSE`, `LOAD-DEPTH`, `LOAD-ASIF`, `LOAD-RESERVED`, …), never a crash.
- The formatter: `canon.rs` (each file reprinted: header first, sections in kind order,
  kinds in tree order, versions last by `known`; the reality's hash), `canon_order.rs` (the
  sort keys: an index's keys in order, keys of different literal kinds by kind first, a
  version's items by target and keys, ties by the canonical line), `spacing.rs` (a line's
  tokens and the spaces between them; numbers in their canonical spelling), `parens.rs` (a
  pair of parentheses stays only where the parse needs it: the section is parsed again
  without it and compared).

**Time** (`time/`). `day.rs`: a checked calendar day. `period.rs`: months, quarters, years,
their steps. `calendar.rs`: business days, and the days a window added to a day can take.

**The model** (`model/`): what load leaves. `reality.rs` holds the whole: the namespace,
the `same` pairs, a joint model's members, the priced places; `ids.rs` the typed ids and
the namespace that gives them (each name with its sort and its id); kinds and things
(`kind.rs`), indexes and versions (`index.rs`, a version's `known` day or window), keys,
spans and a stepped range's values as keys (`key.rs`), declarations and what versions state
for them (`decl.rs`), units (`unit.rs`), static types (`ty.rs`), runtime values (`value.rs`;
a window keeps the read it is the value of, so two reads of one item compare as one value),
the scenarios a reality loads (`world.rs`), and the item graph (`graph.rs`: who reads what,
with how the keys relate; the running reads; the random reads by role; what each projection
leaves out; the decisions' strata; the kinds a binding ranges over).

**Load** (`load/`, E §3.1): see the next section, pass by pass.

**The read rule** (`read/`, K, L §4.6). `rows.rs`: the candidate rows of a read and the one
it takes. `gap.rs`: the gap rule, `none` outside a lifespan, the hole's question. `rules.rs`:
the rule version in force at a law date. `bands.rs`: a band's table and row. A plain scan:
the row index (fence F3) is opened when a budget row asks.

**The evaluator** (`eval/`, E §3.2), in five parts.
- *The core.* `expr.rs`: one arm per node kind. `rules.rs`: a def cell, its law date set by
  `at`, its value from a given or the winning rule. `memo.rs`: the memo (fence F2).
  `stack.rs`: the outermost reads; past the counted depth a cell waits on the work stack and
  the read is replayed. `state.rs`: the context (law date, cut, walk point, the line's
  `with uncertain`, the layer, the world), scopes, the register a cell collects, the
  `Engine` of one ask, the walks up to a day it keeps.
- *The context's layers* (E §3.2, §3.3; L §5.5, §5.11, §5.18, §6.4). `context.rs`: a layer,
  the pins, wording pins and axis options in force, interned as one small id, and trimmed
  for each def to what it can depend on. `pins.rs`: `as if` and an ask's `with` applied,
  each pin's keys, date and value, and the `pinned:` lines. `timelines.rs`: one timeline per
  cut, world and pin set the moves reach, parked and swapped at a scope's edge. `srows.rs`:
  a scenario's expression row at run, the base's items it names read in the base.
  `solve.rs`: `solve`'s exact bisection on its grid, each step a pin layer freed after it.
- *Reads and keys.* `read.rs`: an input read, the row the read rule takes and its value with
  its register entries, `last` and `since`. `keys.rs`: a read's keys, each made a key of its
  position's index, kind, axis, list or range, and checked against what the cut and the
  world see. `convert.rs`: a row's value converted to its target's type. `ranges.rs`: a
  binding's stepped ranges of numbers, money or days, counted against the key limit before a
  value is built. `bindings.rs`: what a binding ranges over (an index's keys the cut sees, a
  key list's, a kind's things alive on a day or over a span). `years.rs`: a four-digit figure
  the checker read as a year, met at run as that year. `windows.rs`: a day not yet known (a
  window) through comparisons and arithmetic, as an interval; where one value is needed, an
  undecided interval is `HOLE-IMPRECISE`.
- *Operations.* `arith.rs` (arithmetic and units at run), `aggregate.rs` (`sum`, `count`,
  `any`, `all`, the unknown members a hole `where` makes), `extremes.rs` (`min` to `last`,
  and the `ties by` that breaks a tie), `band.rs` (bands), `builtin.rs` (the built-ins'
  dispatch and the number built-ins), `dates.rs` (the date built-ins, each over a window
  too), `business.rs` (the calendar built-ins: business days, rolls, a period's n-th last),
  `prorata.rs` (`prorata`'s reading; each share from `num/split.rs`), `ty.rs` (a sample
  value of a type, for the zero of an empty sum).
- *Explanation.* `trace.rs`: the derivation `explain` prints, recorded while the value's
  cone is evaluated again with tracing on (E §10).

**Holes** (`hole/`, K, L §8.1). `questions.rs`: a hole is its set of questions; the one
per-operator rule. Holes are built only here (R-116).

**Numbers** (`num/`, K, L §7). `rational.rs`: `Num`, a reduced fraction, small or big, 16
bytes. `small.rs`: the small tier (fence F4). `big.rs`: the big tier, the one file that
names the big-integer crate. `gcd.rs`: Euclid, big by small, Lehmer (fence F4). `money.rs`:
whole minor units. `round.rs`: the seven modes. `text.rs`: literals read and values printed.
`split.rs`: a total split by weights to a step, `prorata`'s three rests; the
largest-remainder split is written here once, for every later split that takes it (R-114).

**The walk** (`walk/`, K, E §4).
- *The plan pass* (E §4.1). `plan.rs`: each move's instances, dated at the cut, valued at
  their booking cut, classified (past, plan, hole, uncertain), in the movement order;
  invariant 5 across asks. `tuples.rs`: a move's binding tuples, bound depth first at its law
  date, and the tuple and places a link names. `legs.rs`: each leg's two places, its day and
  its amount as a quantity of its unit.
- *The day walk* (E §4.2). `agenda.rs`: the acts handed to the day walk, each at its point;
  the next day to step (fence F1: only days with an act); the days a hole movement may fall
  on. `point.rs`: a point of the walk's order (day, step, stratum, rank, instance). `day.rs`:
  one day: acts in order, legs booked at both ends, the sign rule after each batch, the close
  with invariants 1, 2 and 6; in the `fault-injection` build, the planted fault. `order.rs`:
  a day's decisions, each run at its point against the balances as they stand, in their
  order: strata, `before` and `after`, `ties by`, a cycle no written order breaks refused.
  `state.rs`: each place's log of entries and the state after each; the monthly checkpoints
  (P §2.4).
- *The reads.* `reads.rs`: `bal` (plain, running, `with uncertain`) with its register, and
  the walk up to a day that a decision's leg read needs while the main walk is still being
  computed. `flows.rs`: `flows` of a place and of a move. `value.rs`: `value(x, d) in CUR
  [by p]`, a place's positions at the end of a day, each at its price.
- *The written projection* (E §4.3). `projection.rs`: `bal(x, d) from t`, a walk of its own
  from day t without its holder's dependents, and `low`, its least end-of-day balance.
  `lows.rs`: fence F5, the same answers in the common case (no decision the projection
  counts) read from running sums of x's counted plan legs kept in a tree.

**The book** (`book/`, K, E §5, §6). `observe.rs`: observations the cut sees, coverage,
links to the instances they fulfil. `reading.rs`: `opening` and `stated` readings, the
anchor and the pad. `adjust.rs`: a booked instance measured again at each later version.
`settle.rs`: settlements and waivers. `members.rs`: movements as values in bindings, a
decision's movements among them; `next`; a movement's leg read by name. `ledger.rs`: the
residual ledger.

**The asks** (`ask/`, L §5.18).
- `answer.rs`: each ask's cut, law date and `with` pins, its binding tuples, its lines'
  values, `check` or `solve`, and each line's register; `REQ-ASK` for an ask no world has.
- `worlds.rs`: the asks of a reality with scenarios: each ask in the world it names (`in s`,
  `diff … in s against b`, a `{scenario s}` pin); each refused scenario's line first, the
  worlds' warnings merged.
- `axes.rs`: an ask's fan-out over the axes it reaches: every combination of their options
  evaluated, the combinations that give one result printed as one line, each line a product
  of options (`*` for an axis whose every option gives it).
- `check.rs`: `check e == f`: `ok`, or `differs` with both sides and the difference.
- `diff.rs`: a `diff` ask's lines, key by key changed, added, removed or unchanged.
- `explain.rs`: `explain`'s derivation, `inputs-of`'s rows with their sources,
  `readers-of`'s readers; it dispatches `explain`'s other forms to `walked.rs` (a balance, a
  projection, a `low`) and `instance.rs` (a movement instance: class, day, booking cut, legs,
  a decision's walk point and reads, adjustments).
- `register.rs`: the register under a line, then the warnings. `residuals.rs`: a `residuals`
  ask's groups and lines.

## Load, pass by pass

Load is a list of steps run in one order, refcore's (`load/steps.rs`, E §3.1 "The order of
the checks"). The order matters as much as the checks: a text with two faults must print the
same first refusal refcore prints, so each step is the twin of a refcore function, met in
the same sequence. Each step reads only what earlier steps built.

**Before load: worlds and members** (E §3.1 step 1).

| Step | Files | refcore twin |
|---|---|---|
| every file split, headers read, by full path | `load/worlds.rs` | `cli.py` `_run_texts`, `run_worlds` |
| a joint model: members found (the files given, then the disk), pinned, each loaded alone as a frame of a worklist, never by recursion | `load/joint.rs` | `joint.py` `split_reality`, `member_texts`, `pin_check`, `member_hash` |
| the members merged: their declarations, shared law, the header's clauses | `load/merge.rs` | `joint.py` `joint_texts`, `collect_decls` |
| each member's lines under its prefix (`a.x` becomes one name), its currencies and calendar | `load/prefixed.rs` | `joint.py` `member_lines`, `drop_blocks`, `dedupe_units`, `rename_tokens`, `rewrite_member_refs` |
| the header's `same` lines of kinds, places and things, grouped and planned as one item each | `load/joins.rs` | `joint.py` `same_joins`, `plan_joins` |
| scenarios: the base alone, then each scenario with the base and its stack (at most 4 deep) | `load/worlds.rs` | `cli.py` `run_worlds`; `loader.py` `load_texts` |

**Passes 1 and 2: what the reality declares** (E §3.1 steps 1–3).

| Step | Files | refcore twin (`loader.py` unless named) |
|---|---|---|
| one model header, one `tally`, the scenarios' `over` | `load/steps.rs`, `model/world.rs` | `Loader.build` pass 1, `scenario_over` |
| the one namespace: the built-in ids (`parties` among them), every section's id once (`LOAD-DUP`, `LOAD-RESERVED`) | `load/names.rs`, `model/ids.rs` | `register_builtins`, `register_names` |
| units and currencies | `load/steps.rs` | `build_units` |
| kinds, subkinds, the prelude | `load/kinds.rs` | `build_kinds` |
| indexes: built-in periods, lists, ranges; then the axes | `load/indexes.rs`, `load/steps.rs` | `build_indexes`, `build_range` |
| versions: spans, `known` dates and windows, `moves`, sources | `load/spans.rs` | `model.py` `lit_period`, `lspan_to_span`, `VersionInfo` |
| things entering kinds, with their lifespans | `load/kinds.rs` | `build_entries`, `entry` |
| declarations: inputs, defs, bands, moves, their binds, types and units | `load/decls.rs` | `build_decls` |
| open indexes' keys, then places | `load/indexes.rs`, `load/places.rs` | `prescan_open_keys`, `build_places` |
| a joint model's places stated `same`; keys that are also ids | `load/same.rs`, `load/names.rs` | `joint_sames`, `check_key_ids` |

**Pass 3: what versions state** (E §3.1 step 4).

| Step | Files | refcore twin (`loader.py`) |
|---|---|---|
| rows, givens, band rows, replaced rules, `covers`, gap roles; keys resolved | `load/rows.rs`, `load/keys.rs` | `build_items`, `row_item`, `band_item`, `rule_item`, `resolve_key_literal` |
| observations as real movements, their legs, attributes and what they fulfil | `load/observed.rs` | `obs_item`, `obs_fields`, `check_mv_refs` |
| every row's value converted to its target's type (the conversion is `eval/convert.rs`, which the run shares) | `load/values.rs` | `check_rows` |
| overlaps and `replace`; each observation one movement on a covered day | `load/overlap.rs` | `check_overlap` |
| `same` lines of rows and movements | `load/same.rs` | `joint_sames`, `sref` |
| scenario rows against `from`; lifespans a scenario changes | `load/scenarios.rs` | `check_scenarios`, `check_scenario_lifespans` |
| band tables whole and in order | `load/bands.rs` | `check_bands` |

**Pass 4: the checker** (E §3.1 step 5). One walk over every expression: the parts clauses
and price clauses first, then one sort at a time (moves, fills, defs, bands, asks), each in
section order, then `prices … by`, the random rows and a scenario's expression rows. Each
expression is **typed** (a static type: money in a currency, a number with a unit, a date, a
key of an index, a record, …) and each **read recorded** as an edge of the item graph, with
how the reader's keys relate to the read's. Types and edges come from the same walk, as in
refcore's `Checker.tc`, so within one item the first refusal is refcore's.

| Part | Files | refcore twin |
|---|---|---|
| the walk, its scopes and variables, edges and marks; bands; asks; prices | `load/check.rs` | `check.py` `Checker.run`, `check_band`, `check_ask`, `check_prices` |
| a def's rules: `when`, the value, labels, `beats` (never in a cycle), the key offsets of its arithmetic | `load/defs.rs` | `check.py` `check_def`, `_beats_cycle`, `item_offsets` |
| names and literals | `load/types.rs` | `check.py` `tc`, `t_name`, `keylit` |
| an input, a def, a place or a move read at keys, each key at its position; a postfix's index or field; a prefix that names nothing, a member's ask read as a value | `load/keyed.rs` | `check.py` `read`, `key_pos`, `key_conv`, `t_index`, `t_field` |
| an expression as a load refusal names it | `load/texts.rs` | `check.py` `_txt`; `check3.py` `span_txt` |
| arithmetic and the unit algebra, `if`, `and`, `or`, `not`, aggregates | `load/ops.rs` | `check.py` `_arith`, `fx_check`, `unify`, `t_agg` |
| comparisons with their guards, key-list variables and period literals as keys, membership | `load/compare.rs` | `check.py` `t_cmp`, `kl_value`, `compare`, `t_in` |
| built-in calls, bands | `load/calls.rs` | `check.py` `t_call`, `c_*`, `band_call`, `prefixed_call` |
| bindings, a value assigned to a declared type | `load/binds.rs` | `check.py` `bind`, `assign` |
| a move's day, legs and attributes against its kind's, and the place or read each target names | `load/targets.rs` | `check3.py` `m_check_move`, `m_read_target`, `m_place_target`, `m_move_changes` |
| a leg's places, kinds, units and amount | `load/moves.rs` | `check3.py` `m_check_leg` |
| `bal`, `low`, `flows`, `gap`, `observed`, `next`, projections | `load/reads.rs` | `check3.py` `m_t_bal` and its siblings |
| priced places, `value(x, d) in CUR` | `load/prices.rs` | `check3.py` `m_check_priced`, `m_t_value` |
| fills and `prorata` | `load/fills.rs` | `check2.py` `m_check_fill`, `m_t_prorata` |
| distributions, queries, `crps`, `cdf`/`pdf`/`pmf` | `load/random.rs` | `check2.py` `m_tc_dist`, `m_t_query_random`, `m_check_dist_rows` |
| `to u`, the transcendental and rate built-ins | `load/exact.rs` | `check2.py` `m_t_to`, `m_c_exp`, … |
| pins, `as if`, an ask's axes | `load/pins.rs` | `check.py` `pin`, `pin_literal` |
| parts clauses, arrival and leaving rules, stated parts | `load/parts.rs` | `parts.py` `chk_check_parts` and its siblings |
| a scenario's expression rows; which asks a scenario's reality checks | `load/srows.rs` | `check2.py` `m_check_scen_rows`; `check.py` `ask_worlds` |

**Pass 5: the checks over the graph** (E §3.1 steps 6–10), run by `build_graph` in
`load/strata.rs`, in this order, refcore's:

| Check | Files | refcore twin |
|---|---|---|
| same-key cycles, recurrence reads and their guards (`LOAD-CYCLE`, `LOAD-GUARD`) | `load/strata.rs` | `check.py` `check_graph`, `_cycles` |
| law dates (`LOAD-LAWDATE`, `LOAD-LAWPERIOD`) | `load/lawdates.rs` | `check.py` `check_lawdates` |
| an ask's axis combinations, at most 64 (`LOAD-FORKS`) | `load/pins.rs` | `check_graph`'s forks |
| unread items (`WARN-UNREAD`) | `load/strata.rs` | `check_graph`'s end |
| running reads, declared reads, the decisions' reach, walk-bound defs, left-out moves, the strata (`LOAD-READS`, `LOAD-ORDER`) | `load/strata.rs`, `load/parts.rs` | `check3.py` `m_check_moves_graph`, `m_strata`; `parts.py` `chk_parts_walk_bound` |
| an arrival rule's reads of its own day | `load/parts.rs` | `parts.py` `chk_check_arrival_reads` |
| readings `after` a plan or an observation | `load/strata.rs` | `check3.py` `m_check_after_readings` |
| days and periods load sees; periods across keys | `load/across.rs` | `check3.py` `m_check_seen_days`, `m_check_period_keys` |
| moves' law dates | `load/lawdates.rs` | `check3.py` `m_check_move_lawdates` |
| the random cone, draws, queries, orientation (`LOAD-RANDOM`, `LOAD-ORIENT`, `LOAD-ASKONLY`); projections that reach a random amount | `load/cone.rs`, `load/across.rs` | `check2.py` `m_random_cone`, `m_finish_random`; `check3.py` `m_check_proj_seen` |
| what a scenario's expression row reads (`LOAD-SCENARIO`) | `load/srows.rs` | `check2.py` `m_check_srow_reads` |
| schedules that give no date; a scenario's move before its `from`; a query over paths on a `path p` line | `load/across.rs`, `load/cone.rs` | `moves.py` `m_check_schedule`; none; `eval2.py` `m_path_terms` |

Where the spec and refcore disagreed, the spec won and the engine refuses at load what refcore
refuses later, with another code or not at all (issues #72, #73, #75, and at M1 #130 and
#136): those checks have no twin, or a twin outside refcore's load, and their headers say so.

## The ideas M0 added

- **The symbol table** is the reality's one namespace (`Reality.names`, `model/ids.rs`):
  every id once, each with its sort (input, def, band, place, kind, index, unit, …). A joint
  model's member items enter it under their prefix as one name (`a·x`, printed `a.x`).
  Since pre-M1 each name is interned once (`base/symbol.rs`) and every table is found by a
  typed id of its sort, so an id of one sort can't find an item of another.
- **Spans and versions.** A version states rows for a span of days (`governs`) and is known
  from a date, or across a known window; a row may carry its own span and known date. Load
  resolves every span to days, groups rows by target and keys, and refuses overlaps that no
  `replace` explains. The read rule (`read/`, S's) then picks the row at a read.
- **Worlds.** A reality with scenarios loads several times: the base alone, then each
  scenario with the base's files and the scenarios under it. Each is a full reality. A
  scenario's reality keeps only the asks its world evaluates (L §6.4).
- **Joint models.** A member is a closed model, found by path, pinned by the hash of its
  canonical text, loaded alone first (its own refusals print `member a: …`), then merged
  into the joint model under its prefix. Nesting is a worklist, never recursion (R-16).
- **The item graph** is built by the checker: nodes are items (an input, a def, a band, a
  move, a move's day, a place, an ask, a price, a fill, an axis, a scenario's expression
  row, …), edges are reads with how their keys relate (same key, a shifted key, a computed
  key, …). Every check of pass 5 is a search over it, each an explicit stack.
- **The canonical format** is computed on tokens, as refcore's `canon.py` does: blocks of
  lines reordered by the keys L §2.4 states, each line's tokens respaced, each pair of
  parentheses kept only where taking it out changes the parse.

## The ideas M1 added

- **One context, in layers** (E §3.2, D1-3). What a value depends on beyond its law date and
  cut is one small interned id, a *layer*: the pins in force (`with`, `as if`, `law`,
  `wording`) and the options of the axes it reaches (`eval/context.rs`). Each def keeps
  only the part of a layer it can reach, so a cell that no pin reaches is shared with the
  unpinned run instead of computed again. A layer made for one scope (an `as if`, a step of
  `solve`) is freed at its end.
- **Worlds at run.** The world an ask is in (its scenario stack, or one under it) is part of
  every cell's key and is never trimmed: no value is shared across worlds. `in s`,
  `diff … in s against b` and `as if {scenario s}` read through it (`ask/worlds.rs`,
  `ask/diff.rs`).
- **Timelines.** The walk's state is kept apart per (cut, world, pin set) that the moves
  reach (`eval/timelines.rs`, E §3.3): a pin that changes a plan's amount gets a walk of its
  own; one that doesn't shares the main walk. Moves inside `as if` are computed and never
  booked; each timeline's signature is checked after the asks (E §9 item 4).
- **Fan-out over axes** (`ask/axes.rs`, L §5.5). An ask evaluates every combination of the
  axes it reaches (at most 64, `LOAD-FORKS` at load) and prints one line per distinct
  result, each line naming the options that give it.
- **Windows at run** (`eval/windows.rs`, L §4.4). A day not yet known is an interval; a
  comparison is decided when the whole interval lies on one side; a window compared with
  itself is decided because it remembers which read it is the value of; where one day is
  needed and the interval is undecided, the value is `HOLE-IMPRECISE`.
- **The written projections** (`walk/projection.rs`, E §4.3). `bal(x, d) from t` and `low`
  walk their own window from day t, counting what is known on t and leaving out the
  movements that depend on x (listed `left out:`), so a decision never sees its own future.
  In the common case fence F5 (`walk/lows.rs`) answers the same from a tree of running sums
  (O(log n) per read, P8-5).
- **Decisions in their order** (`walk/order.rs`, L §5.14.2). A day's decisions run by
  stratum, then by their written `before` and `after`, then by `ties by`; a cycle that no
  written order breaks is refused. A decision's leg read that needs the walk up to its own
  day, while the main walk is still being computed, gets a walk up to that day of its own
  (`walk/reads.rs`).
- **Provenance.** `explain` evaluates the value's cone again with tracing on and prints the
  derivation (`eval/trace.rs`, `ask/explain.rs`, `ask/walked.rs`, `ask/instance.rs`;
  E §10). `inputs-of` and `readers-of` read the cells' read sets and the item graph.
- **One split** (`num/split.rs`, R-114): `prorata`'s three rests, and the largest-remainder
  split written once for every later split.
- **A planted fault** (`fault-injection` build only): one minor unit added to a day's
  conservation sum, so the `ENGINE-INVARIANT` path has a fixture (gate 5). The feature
  exists only in builds with debug assertions (R-51): a release build with it doesn't
  compile, which `check full`'s `fault-injection-probe` step shows.

## Cross-cutting ideas

- **Contexts.** Every value is computed in a context: a law date, a cut, inside the walk a
  walk and a point, the layer of pins and axis options, and the world; an ask line's own
  reads also carry its `with uncertain`, which no decision or timeline sees (L §5.14). The
  memo keys a def cell by (def, keys, cut, walk point, `with uncertain`, layer, world) and
  keeps one cell per law-date class: the law dates whose answer can't differ, found from the
  spans of the rows and rules the cell actually read (E §3.2). A text that names the day (a
  hole's question, a `held` entry) keeps the cell at its own day.
- **Read sets.** Each def cell records what it read: the inputs, rows, a def's givens at its
  keys, bands, band rows and cells (E §3.2, P §4). What a walk, a plan, an adjustment or a
  link reads is shared by every reader and is not in a cell's set. `explain`, `inputs-of`
  and `readers-of` read them; nothing skips work by them yet: they are the seam for P §4,
  a change that costs what it touches.
- **The walk is a fold over days,** stepped only on days with an act (P §2.2). One main
  walk per cut and timeline; a projection walks its own window from the state of the walk it
  continues. Balances are logs of change points, so `bal(x, d)` is a binary search.
- **Checkpoints.** At each month's start a main walk keeps every place's state, shared
  with the month before where it didn't change, and the decision legs booked ahead. They
  are for the forks (P §2.5, M2) and the walk restart (P §4.2); nothing starts from one yet.
- **Holes are values,** carried by every operator by one rule, and printed with their
  questions. A missing value is never 0.
- **Refusals** are values too: a code from the catalogue and a message; a broken
  invariant is `ENGINE-INVARIANT`, never a crash. A load refusal names its file and line; a
  refusal met inside an expression takes the line its check names.
- **Recursion** is counted: `DEPTH_LIMIT` 32 in the evaluator, 512 in the checker's walk
  (far above the parser's depth of 8, which every input meets first); deeper chains wait on
  an explicit work stack.
- **"Not built."** A form the engine doesn't build stops the run with "not implemented"
  (exit 69), never a guess. At M1 that is the run of forms later milestones build (below).
  Load itself stops so only where the spec and refcore disagree and a STOP-REPORT is open, at
  a capacity limit the spec doesn't state, or at a form a later milestone owns. Each site and
  its owner is in `BUILD-LOG.md`'s entries: "M0 built", and for M1 the groups' entries
  (G13's for the 26 run sites).

## Invariants, and what is deliberately absent

- **Exact.** No floating point anywhere (R-18). Integers only through checked calls; an
  overflow or the 4 096-digit cap is a refusal with its code.
- **Deterministic.** No hash map or set (B-trees, sorted slices, dense vectors); no clock,
  environment, file or network in the library; output ordered by explicit keys, never by
  an address, an interned number or insertion order; paths sorted by code point. Where
  refcore's order is a dict's insertion order (the reads an item made first, the asks'
  reads), the engine keeps that order in an explicit list beside the sorted map.
- **Never a crash.** No `unsafe`, `unwrap`, `expect`, `panic!`, indexing or slicing in
  the library; `ENGINE-INVARIANT` where the engine finds itself wrong. Checked after each
  day: money conserved (E §9 item 1), net worth explained (item 2), nothing held outside a
  lifespan (item 6); a plan instance the same under every cut that books it (item 5); asks
  move no money (item 4) and a scenario changes nothing before its `from` (item 3), both in
  `invariant.rs`, after the asks. The parser is total: the `parse`, `format` and `load` fuzz
  targets hold every input to a result or a coded refusal.
- **A small Rust.** No `dyn`, no `impl Trait` arguments, no macros of our own, no own
  generics, nesting at most 5 (R-122 to R-125, R-73, R-74). Enums and exhaustive `match`.
- **No threads yet.** The run happens on the caller's thread. The thread module comes
  with Monte Carlo (M7): scoped threads, explicit stack size, merge in task order.
- **Absent at M1, on purpose:** the run of what load checks but no M1 vector reaches, each
  stopping "not implemented": `windows each` (an ask's and a balance read's; pick P52, M2),
  a partial answer, a known window or a `moves` line at run, a `diff` or `residuals` ask
  over axes. M2: observations, `same` pairs and readings at run, the four clocks, the
  residual ledger in full, forks from the checkpoints. M4: the law-breadth forms. M5a:
  fills, holdings, ownership. M5b: parts and labels at run. M5c: joint models' asks and a
  member's reads at run. M6: `to u`, the transcendental calls, `cdf`/`pdf`/`pmf`/`crps` at
  run. M7: distributions, random asks, queries, paths and timelines of paths. Fences F2's
  dense columns and F3 (the read index) stay closed until a budget row asks (R-130).

## Boundaries

- **Library against CLI.** `crates/tally` is pure: bytes in, lines out. `crates/tally-cli`
  turns arguments and files into a `Given` list and the answer into stdout and an exit
  code. Only `crates/tally-cli/src/main.rs` reads files, arguments and writes output (R-39,
  R-108), a joint model's member files included; it holds the CLI's three exceptions.
- **The kernel** (`num`, `read`, `walk`, `book`, `hole`): the code gate 8's mutants run
  over, with an exception budget of its own; one exception (the big-integer wrapper in
  `num/big.rs`, R-29). M1 added three kernel files and no exception.
- **The parser and the library** have an exception budget of 0; M1 kept both at 0.
- **The big-integer crate** is named in one file, `num/big.rs`; the hash uses the `sha2`
  crate, already a dependency, in `syntax/canon.rs`.
- **Features.** `counters` (the `--stats` counts) and `fault-injection` (the planted fault)
  are test and measurement builds, never the shipped CLI (R-51).
- **The thread file:** none yet.
- **Fences** (R-130): optimised code only where a budget row asked, each with a `PERF:`
  line, a plain twin and an equivalence test:
  - F1 the event walk (`walk/agenda.rs`, P8-14): twin `oracle.rs`'s every-day walk;
    checkpoints in `walk/state.rs`, twin the log itself;
  - F2 the memo's law-date classes (`eval/memo.rs`, P8-14): twin `Memo::Exact`, refcore's
    keying;
  - F4 the number tiers (`num/small.rs`, P8-M1; `num/gcd.rs`, P8-M2): twins the big tier
    and Euclid;
  - F5 the written projection's common case (`walk/lows.rs`, P8-5, P8-M4), opened at M1:
    twin the projection's own walk (`Lows::Walk`), compared with the tree (`Lows::Tree`) on
    T's corpora by `m1_lows` and on F5's realities by `tally-cli/tests/walk_oracle.rs`.

## Where the next milestones plug in

- **M2, replay against reality.** Load already builds observations as movements, readings
  and `same` pairs (`Reality.sames`); M2 reads them at run: `book/` for booking at the cut,
  `walk/reads.rs` for the modelled balance, `book/ledger.rs` for the residual ledger in full.
  Each stops "not built" at run today. The forks start from the checkpoints `walk/state.rs`
  already keeps, a fence whose twin is the unforked walk. `windows each` is M2's too (P52).
- **Later milestones** find their load side done too: fills (M5a) are typed with their
  sides and arcs, parts clauses (M5b) checked, joint models (M5c) merged with their members
  loaded alone and their asks known, distributions and queries (M6, M7) typed and their
  random cone computed. Their run is theirs.

## Tests, and who decides "done"

- **T's tests** in `engine/tests/` decide: the checks against refcore's bytes, the
  household vectors, the kernel's properties. For M0: `m0_format`, `m0_load`, `m0_codes`,
  `m0_loadcheck`, `m0_properties`. For M1 (`tests/MILESTONE-M1.md`): `m1_checks` (the 90
  checks, X's forecast and changes among them), `m1_goldens` (the goldens and pins),
  `m1_codes` and `m1_invariant` (a fixture per code, `ENGINE-INVARIANT` under the planted
  fault), `m1_properties` (the laws of gate 6), `m1_carried` (what pre-M1 left), `m1_fixes`
  (a vector per review fix and per mutant left for T), `m1_format` (#109), `m1_lows` (F5
  against its twin). The engine never reads them (R-93).
- **The builder's tests** make the internals easy to get right and count toward no gate:
  inline `tests` modules, `crates/tally/tests/*_cases.rs` (small realities, refcore's lines),
  `crates/tally/tests/catalogue.rs` (the codes against the spec's tables),
  `crates/tally-cli/tests/walk_oracle.rs` (the twins compared on the kernel corpus, on X's
  checks and on F5's realities in `tally-cli/tests/lows/`) and
  `crates/tally-cli/tests/fault.rs` (the planted fault).
- **The benches** (`crates/tally/benches/`): `p8_m1` to `p8_m4`, the micro rows; `p8_5`, a
  read of a written projection; `b1_list`, a 100 000-key list.
- **The fuzz targets** (`engine/fuzz/`): `parse`, `format` (a text it prints reprints as
  itself) and `load` (one to three files): every input a result or a coded refusal, never a
  crash, a hang or `ENGINE-INVARIANT`; each crash found is kept in `fuzz/regressions/`.
- **refcore** (`engine/tools/refcore/`) is the oracle of last resort, byte for byte.

## Where the engine does X

| To find | Look in |
|---|---|
| how a file is split into lines and tokens | `syntax/lex.rs` |
| how an expression parses | `syntax/expr.rs`, `syntax/calls.rs`, `syntax/binds.rs`, `syntax/reads.rs`, `syntax/ask.rs` |
| how a section parses | `syntax/item.rs`, `syntax/decls.rs` |
| the canonical format and the hash | `syntax/canon.rs`, `syntax/canon_order.rs`, `syntax/spacing.rs`, `syntax/parens.rs` |
| the order of load's steps | `load/steps.rs` (`load_among`), then `load/strata.rs` (`build_graph`) |
| scenarios as worlds; joint models | `load/worlds.rs`; `load/joint.rs`, `load/merge.rs`, `load/prefixed.rs`, `load/joins.rs` |
| the namespace, `LOAD-DUP` | `load/names.rs`, `model/ids.rs` |
| an expression's type, a load refusal inside it | `load/types.rs`, `load/keyed.rs`, `load/ops.rs`, `load/compare.rs`, `load/calls.rs`; its text, `load/texts.rs` |
| who reads what | `load/check.rs` (`edge`), `model/graph.rs` |
| a load refusal's text | the check's file in the tables above; the code in `base/code.rs` |
| which row a read takes | `read/rows.rs`, `read/gap.rs` |
| which rule version is in force | `read/rules.rs`, then `eval/rules.rs` |
| a def's value and its memo cell | `eval/rules.rs`, `eval/memo.rs` |
| a read's keys | `eval/keys.rs`, `eval/ranges.rs`, `eval/bindings.rs` |
| a pin, `as if`, `with` | `eval/pins.rs`, `eval/context.rs` |
| a scenario's world at run | `ask/worlds.rs`, `eval/srows.rs`; the timeline, `eval/timelines.rs` |
| an ask over axes | `ask/axes.rs`, `eval/context.rs` |
| a window's day | `eval/windows.rs`, `eval/dates.rs`, `eval/business.rs` |
| `prorata` | `eval/prorata.rs`, `num/split.rs` |
| `solve`, `check`, `diff` | `eval/solve.rs`, `ask/check.rs`, `ask/diff.rs` |
| a hole's question | `hole/questions.rs`; where it is made, `read/gap.rs` |
| a plan instance's day and amount | `walk/plan.rs`, `walk/tuples.rs`, `walk/legs.rs` |
| what happens on a day | `walk/day.rs`, `walk/order.rs` |
| the order of a day's decisions | `walk/order.rs` |
| a balance or a flow | `walk/reads.rs`, `walk/flows.rs`, `walk/state.rs` |
| `value(x, d) in CUR` | `walk/value.rs` |
| a projection `from t`, `low` | `walk/projection.rs`, `walk/lows.rs` (F5) |
| observations, readings, anchors | `book/observe.rs`, `book/reading.rs` |
| adjustments and settlements | `book/adjust.rs`, `book/settle.rs` |
| movements in a binding, `next` | `book/members.rs` |
| the residual ledger | `book/ledger.rs`, printed by `ask/residuals.rs` |
| `explain`, `inputs-of`, `readers-of` | `ask/explain.rs`, `ask/walked.rs`, `ask/instance.rs`; the trace, `eval/trace.rs` |
| the register under a line | `ask/register.rs` |
| an ask in a reality with scenarios | `ask/worlds.rs` |
| rounding, money, printing a number | `num/round.rs`, `num/money.rs`, `num/text.rs` |
| an error code's text | `base/code.rs` |
