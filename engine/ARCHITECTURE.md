# The Tally engine's architecture

The map of `engine/crates/`, for the owner, who audits the engine through this page, each
file's `//!` header and the trace from code to spec (`engine/tools/trace --map`). It names
modules and never links them. Written at milestone S (2026-11-02), brought to M0
(2026-11-05); updated only at a milestone (R-119). The spec is the authority: L is
`spec/LANGUAGE.md`, E `spec/ENGINE.md`, P `spec/PERFORMANCE.md`. The twin of each file,
refcore's file and functions it mirrors, is on its header's `Twin:` line; the load tables
below repeat the main ones.

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
   the walk (`walk`) for balances and flows; the walk books legs, observations and
   readings (`book`); numbers are exact (`num`); a missing fact is a hole (`hole`);
6. **print** the lines, the register under each, then the warnings.

A second entry, **`reprint`**, stops after step 3: it prints each file in the canonical
format (L §2.4), the one spelling every reality has, then the reality's hash, the sha256 of
those texts in path order. A joint model's members are pinned by that hash.

Nothing is computed ahead of a read: an ask reads a def, the def reads a balance, the
balance needs the walk up to its day, the walk needs the plan's amounts, which are defs
again. The memo and the walk's own caches make each piece computed once per context.

**What M0 changed, in one sentence:** at S, load did only what the evaluator needed to run
X's base and answered "not implemented" on any refusal; at M0, load is the whole of L §8.3's
catalogue, in refcore's order, and the parser and the formatter cover all of L §2. The run
past load (`eval`, `walk`, `book`, `ask`) is S's, with "not built" stops for the forms later
milestones build.

## Codemap, by pipeline stage

Paths are under `crates/tally/src/` unless they say otherwise. **K** marks the kernel, the
code DONE gate 8's mutation score covers (`tools/kernel.toml`). M0 added no kernel file and
changed none: all its code is in `syntax/`, `load/`, `model/`, `ask/`, `eval/`, `api.rs`,
`lib.rs`, `base/refusal.rs` and the CLI.

**The way in.**
- `lib.rs`: the library's face; each module block names its files and its spec section.
  Three entry points for the fuzz targets: `parse` (one file's bytes to trees) and
  `format_file` (one file reprinted) here, `load_report` (load alone) from `api.rs`.
- `api.rs`: `run`, `run_in` (`run` with the disk's member files, the CLI's entry) and
  `run_with_memo`: read, split, load the reality or its worlds, answer (E §1; L §8.3 for
  which refusal prints first); `reprint`; `load_report` (what load prints when no ask is
  answered).
- `oracle.rs`: the walk's day source: the event walk, or every day (the test oracle).
- `counters.rs`: the `--stats` counts, in the `counters` build only (BUILDER §13.2).
- `invariant.rs`: E §9 item 4, asks move no money: each main walk's signature, kept
  once it is walked and checked again after each ask.
- `crates/tally-cli/src/lib.rs`: the `tally` command as a function, `run(args, files)`:
  arguments, paths, exit codes 0/1/2 (INTERFACE §1–§4); `tally reprint`, its files named as
  given and taken by full path (INTERFACE §1, issue #60).
- `crates/tally-cli/src/main.rs`: the binary; the one file that reads files and writes
  output. It also reads a joint model's members from the disk, beside the files it is given,
  round by round (INTERFACE §2): the library never touches the disk.

**Foundations** (`base/`). `code.rs`: the catalogue's 99 codes (L §8's 95 and E §11's
four), one enum. `refusal.rs`: a refusal carries its code, message and place; a member's
refusal is printed `member a: …`. `table.rs`: the one name-keyed store, items in the order
they were added (the reality's section order), with a checked `get` and no indexing.

**Text** (`syntax/`, the lexer-parser area, exception budget 0).
- The parser: `lex.rs` (lines, indent levels, tokens; each token keeps the role the parser
  gave it, for the formatter), `parse.rs` (the cursor over one line; nesting counted to
  L §2.3's depth of 8), `expr.rs` (expressions by precedence climbing), `reads.rs` (`low`,
  `crps`, queries, distributions), `ask.rs` (ask lines, pins, `as if`, `solve`), `item.rs`
  (the file grammar: header, sections, version items; `as if` nested at most 4 deep),
  `tree.rs` (the tree, one type per production). The parser reads every form of L §2.3; a
  text it can't read is a coded load refusal (`LOAD-PARSE`, `LOAD-DEPTH`, `LOAD-ASIF`,
  `LOAD-RESERVED`, …), never a crash.
- The formatter: `canon.rs` (each file reprinted: header first, sections in kind order,
  kinds in tree order, versions last by `known`; the reality's hash), `canon_order.rs` (the
  sort keys: an index's keys in order, a version's items by target and keys, ties by the
  canonical line), `spacing.rs` (a line's tokens and the spaces between them; numbers in
  their canonical spelling), `parens.rs` (a pair of parentheses stays only where the parse
  needs it: the section is parsed again without it and compared).

**Time** (`time/`). `day.rs`: a checked calendar day. `period.rs`: months, quarters, years,
their steps. `calendar.rs`: business days.

**The model** (`model/`): what load leaves. `reality.rs` holds the whole: the namespace,
the `same` pairs, a joint model's members; kinds and things (`kind.rs`), indexes and
versions (`index.rs`, a version's `known` day or window), keys and spans (`key.rs`),
declarations and what versions state for them (`decl.rs`), units (`unit.rs`), static types
(`ty.rs`), runtime values (`value.rs`), the scenarios a reality loads (`world.rs`), and the
item graph (`graph.rs`: who reads what, with how the keys relate; the running reads; the
random reads by role; what each projection leaves out; the decisions' strata).

**Load** (`load/`, E §3.1): see the next section, pass by pass.

**The read rule** (`read/`, K, L §4.6). `rows.rs`: the candidate rows of a read and the one
it takes. `gap.rs`: the gap rule, `none` outside a lifespan, the hole's question. `rules.rs`:
the rule version in force at a law date. `bands.rs`: a band's table and row. A plain scan:
the row index (fence F3) is opened when a budget row asks.

**The evaluator** (`eval/`, E §3.2). `expr.rs`: one arm per node kind. `rules.rs`: a def
cell, its law date set by `at`, its value from a given or the winning rule. `memo.rs`: the
memo (fence F2). `stack.rs`: the outermost reads; past the counted depth a cell waits on the
work stack and the read is replayed. `read.rs`: an input read's keys and value. `state.rs`:
the context (law date, cut, walk point, the line's `with uncertain`), scopes, the register
a cell collects, the `Engine` of one ask. `arith.rs`, `aggregate.rs`, `band.rs`,
`builtin.rs`, `ty.rs`: arithmetic, aggregates, bands, built-ins, sample values.

**Holes** (`hole/`, K, L §8.1). `questions.rs`: a hole is its set of questions; the one
per-operator rule. Holes are built only here (R-116).

**Numbers** (`num/`, K, L §7). `rational.rs`: `Num`, a reduced fraction, small or big, 16
bytes. `small.rs`: the small tier (fence F4). `big.rs`: the big tier, the one file that
names the big-integer crate. `gcd.rs`: Euclid, big by small, Lehmer (fence F4). `money.rs`:
whole minor units. `round.rs`: the seven modes. `text.rs`: literals read and values printed.

**The walk** (`walk/`, K, E §4).
- `plan.rs`: the plan pass: each move's instances over its binding tuples, dated and
  valued at the cut, classified (past, plan, hole, uncertain); invariant 5 across asks.
- `agenda.rs`: the acts handed to the day walk, each at its point; the next day to step
  (fence F1: only days with an act).
- `point.rs`: a point of the walk's order (day, step, stratum, rank, instance).
- `day.rs`: one day: acts in order, legs booked at both ends, the sign rule after each
  batch, the close with invariants 1, 2 and 6.
- `order.rs`: a day's decisions, each run at its point against the balances as they stand.
- `state.rs`: each place's log of entries and the state after each; the monthly
  checkpoints (P §2.4).
- `reads.rs`: `bal`, `flows`, a move's flows, each with its register.
- `projection.rs`: `bal(x, d) from t`, a walk of its own from day t.

**The book** (`book/`, K, E §5, §6). `observe.rs`: observations the cut sees, coverage,
links to the instances they fulfil. `reading.rs`: `opening` and `stated` readings, the
anchor and the pad. `adjust.rs`: a booked instance measured again at each later version.
`settle.rs`: settlements and waivers. `members.rs`: movements as values in bindings.
`ledger.rs`: the residual ledger.

**The asks** (`ask/`, L §5.18). `answer.rs`: each ask's cut, law date, tuples and lines;
`REQ-ASK` for an ask no world has. `worlds.rs`: the asks of a reality with scenarios: each
refused scenario's line first, then the asks by id and world, the worlds' warnings merged.
`register.rs`: the register under a line, then the warnings. `residuals.rs`: a `residuals`
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
| the members merged under their prefixes (`a.x` becomes one name), the header's `same` lines of kinds, places and things joined | `load/merge.rs` | `joint.py` `joint_texts`, `collect_decls`, `same_joins`, `plan_joins` |
| scenarios: the base alone, then each scenario with the base and its stack (at most 4 deep) | `load/worlds.rs` | `cli.py` `run_worlds`; `loader.py` `load_texts` |

**Passes 1 and 2: what the reality declares** (E §3.1 steps 1–3).

| Step | Files | refcore twin (`loader.py` unless named) |
|---|---|---|
| one model header, one `tally`, the scenarios' `over` | `load/steps.rs`, `model/world.rs` | `Loader.build` pass 1, `scenario_over` |
| the one namespace: the built-in ids (`parties` among them), every section's id once (`LOAD-DUP`, `LOAD-RESERVED`) | `load/names.rs` | `register_builtins`, `register_names` |
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
| every row's value converted to its target's type | `load/values.rs` | `check_rows` |
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
| the walk, its scopes and variables; asks; prices | `load/check.rs` | `check.py` `Checker.run`, `check_def`, `check_band`, `check_ask`, `check_prices` |
| names, literals, reads, keys, fields; a prefix that names nothing, a member's ask read as a value | `load/types.rs` | `check.py` `tc`, `t_name`, `t_index`, `t_field`, `keylit`, `read` |
| arithmetic and the unit algebra, comparisons, `if`, aggregates | `load/ops.rs` | `check.py` `_arith`, `fx_check`, `unify`, `compare`, `t_agg` |
| built-in calls, bands | `load/calls.rs` | `check.py` `t_call`, `c_*`, `band_call`, `prefixed_call` |
| bindings, a value assigned to a declared type | `load/binds.rs` | `check.py` `bind`, `assign` |
| moves and legs | `load/moves.rs` | `check3.py` `m_check_move`, `m_check_leg` |
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
refuses later, with another code or not at all (issues #72, #73, #75): those checks have no
twin, or a twin outside refcore's load, and their headers say so.

## The ideas M0 added

- **The symbol table** is the reality's one namespace (`Reality.names`): every id once, each
  with its sort (input, def, band, place, kind, index, unit, …). A joint model's member items
  enter it under their prefix as one name (`a·x`, printed `a.x`).
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

## Cross-cutting ideas

- **Contexts.** Every value is computed in a context: a law date, a cut, and inside the
  walk a walk and a point; an ask line's own reads also carry its `with uncertain`, which
  no decision or timeline sees (L §5.14). The memo keys a def cell by (def, keys, cut, walk
  point, `with uncertain`) and keeps one cell per law-date class: the law dates whose
  answer can't differ, found from the spans of the rows and rules the cell actually read
  (E §3.2). A text that names the day (a hole's question, a `held` entry) keeps the cell at
  its own day.
- **Read sets.** Each def cell records what it read: the inputs, rows, a def's givens at its
  keys, bands, band rows and cells (E §3.2, P §4). What a walk, a plan, an adjustment or a
  link reads is shared by every reader and is not in a cell's set. Nothing skips work by them
  yet; they are the seam for `explain`, `inputs-of`, `readers-of` and the change-cost work.
- **The walk is a fold over days,** stepped only on days with an act (P §2.2). One main
  walk per cut; a projection walks its own window from the state of the walk it continues.
  Balances are logs of change points, so `bal(x, d)` is a binary search.
- **Checkpoints.** At each month's start a main walk keeps every place's state, shared
  with the month before where it didn't change, and the decision legs booked ahead. They
  are for the forks (P §2.5) and the walk restart (P §4.2); nothing starts from one yet.
- **Holes are values,** carried by every operator by one rule, and printed with their
  questions. A missing value is never 0.
- **Refusals** are values too: a code from the catalogue and a message; a broken
  invariant is `ENGINE-INVARIANT`, never a crash. A load refusal names its file and line; a
  refusal met inside an expression takes the line its check names.
- **Recursion** is counted: `DEPTH_LIMIT` 32 in the evaluator, 512 in the checker's walk
  (far above the parser's depth of 8, which every input meets first); deeper chains wait on
  an explicit work stack.
- **"Not built."** A form the engine doesn't build stops the run with "not implemented"
  (exit 69), never a guess. At M0 that is mostly the run of forms later milestones build.
  Load itself stops so only where the spec and refcore disagree and a STOP-REPORT is open, at
  a capacity limit the spec doesn't state, or at a form a later milestone owns. The list,
  site by site with its owner, is `BUILD-LOG.md`'s "M0 built" entry.

## Invariants, and what is deliberately absent

- **Exact.** No floating point anywhere (R-18). Integers only through checked calls; an
  overflow or the 4 096-digit cap is a refusal with its code.
- **Deterministic.** No hash map or set (B-trees, sorted slices, dense vectors); no clock,
  environment, file or network in the library; output ordered by explicit keys, never by
  an address or insertion order; paths sorted by code point. Where refcore's order is a
  dict's insertion order (the reads an item made first, the asks' reads), the engine keeps
  that order in an explicit list beside the sorted map.
- **Never a crash.** No `unsafe`, `unwrap`, `expect`, `panic!`, indexing or slicing in
  the library; `ENGINE-INVARIANT` where the engine finds itself wrong. Checked after each
  day: money conserved (E §9 item 1), net worth explained (item 2), nothing held outside a
  lifespan (item 6); a plan instance the same under every cut that books it (item 5); asks
  move no money (item 4, `invariant.rs`, after each ask). The parser is total: the `parse`,
  `format` and `load` fuzz targets hold every input to a result or a coded refusal.
- **A small Rust.** No `dyn`, no `impl Trait` arguments, no macros of our own, no own
  generics, nesting at most 5 (R-122 to R-125, R-73, R-74). Enums and exhaustive `match`.
- **No threads yet.** The run happens on the caller's thread. The thread module comes
  with Monte Carlo (M7): scoped threads, explicit stack size, merge in task order.
- **Absent at M0, on purpose:** the run of what M0 loads but S's evaluator doesn't answer.
  M1: asks in a scenario's world, pins, axes and `as if`, `explain`, the written projections
  (`low`, `last`, `value`, `windows each`), known windows and `moves` lines at run,
  `prorata`, written orders of decisions. M2: observations, `same` pairs and readings at
  run, the residual ledger in full. M4: the law-breadth forms. M5a: fills, holdings,
  ownership. M5b: parts and labels at run. M5c: joint models' asks and a member's reads at
  run. M6: `to u`, the transcendental calls, `solve`, `cdf`/`pdf`/`pmf`/`crps` at run. M7:
  distributions, random asks, queries, paths and timelines. Deferred to M1 by the user
  (G-S2, 2026-11-02): the interner of names, the id-keyed `Table<I, T>` (today `Table<T>`
  keyed by name) and the context's layer slots. Fences F2's dense columns, F3 (the read
  index) and F5 stay closed until a budget row asks (R-130).

## Boundaries

- **Library against CLI.** `crates/tally` is pure: bytes in, lines out. `crates/tally-cli`
  turns arguments and files into a `Given` list and the answer into stdout and an exit
  code. Only `crates/tally-cli/src/main.rs` reads files, arguments and writes output (R-39,
  R-108), a joint model's member files included; it holds the CLI's three exceptions.
- **The kernel** (`num`, `read`, `walk`, `book`, `hole`): the code gate 8's mutants run
  over, with an exception budget of its own; one exception (the big-integer wrapper in
  `num/big.rs`, R-29). M0 left it as it was.
- **The parser and the library** have an exception budget of 0; M0 kept both at 0.
- **The big-integer crate** is named in one file, `num/big.rs`; the hash uses the `sha2`
  crate, already a dependency, in `syntax/canon.rs`.
- **The thread file:** none yet.
- **Fences** (R-130): optimised code only where a budget row asked, each with a `PERF:`
  line, a plain twin and an equivalence test:
  - F1 the event walk (`walk/agenda.rs`, P8-14): twin `oracle.rs`'s every-day walk;
    checkpoints in `walk/state.rs`, twin the log itself;
  - F2 the memo's law-date classes (`eval/memo.rs`, P8-14): twin `Memo::Exact`, refcore's
    keying;
  - F4 the number tiers (`num/small.rs`, P8-M1; `num/gcd.rs`, P8-M2): twins the big tier
    and Euclid.

  M0 opened none.

## Where the next milestones plug in

- **M1, the household forecast.** Load already types and graphs everything M1 runs; M1's
  work is in the run: the evaluator's arms that stop "not built" today (`eval/expr.rs`:
  `as if`, `low`, `last`, `value`; `eval/read.rs`: a scenario's expression row;
  `ask/answer.rs`: the ask forms beyond values and residuals, a partial answer, known
  windows), asks in a scenario's world (`ask/worlds.rs`, `ask/answer.rs`), decisions with
  written orders (`walk/agenda.rs`, `walk/order.rs`), and the pins and axes as layers of the
  memo's context (`eval/state.rs`). The decisions' strata, walk-bound defs and left-out moves
  that M1's walk reads are load's already (`model/graph.rs`).
- **M2, replay against reality.** Load already builds observations as movements, readings
  and `same` pairs (`Reality.sames`); M2 reads them at run: `book/` for booking at the cut,
  `walk/reads.rs` for the modelled balance, `book/ledger.rs` for the residual ledger in full.
  Each stops "not built" at run today, at the line named in the "M0 built" entry.
- **Later milestones** find their load side done too: fills (M5a) are typed with their
  sides and arcs, parts clauses (M5b) checked, joint models (M5c) merged with their members
  loaded alone and their asks known, distributions and queries (M6, M7) typed and their
  random cone computed. Their run is theirs.

## Tests, and who decides "done"

- **T's tests** in `engine/tests/` decide: the checks against refcore's bytes, the
  household vectors, the kernel's properties. For M0: `m0_format` (every text reprints as
  refcore's), `m0_load` (every text loads with refcore's refusals), `m0_codes` (a fixture
  for each `LOAD-*` code and message form), `m0_loadcheck` (the scan of `tools/loadcheck.py`
  agrees), `m0_properties` (equal spellings print identically, file order changes nothing,
  the parser is total, depth stops at 8, `format` is idempotent). The engine never reads
  them (R-93).
- **The builder's tests** make the internals easy to get right and count toward no gate:
  inline `tests` modules, `crates/tally/tests/*_cases.rs` (small realities, refcore's lines;
  `parse_cases.rs`, `format_cases.rs` and `load_cases.rs` hold M0's),
  `crates/tally/tests/catalogue.rs` (the codes against the spec's tables), and
  `crates/tally-cli/tests/walk_oracle.rs`, the twins compared on the kernel corpus and on
  X's three S checks.
- **The fuzz targets** (`engine/fuzz/`): `parse`, `format` (a text it prints reprints as
  itself) and `load` (one to three files): every input a result or a coded refusal, never a
  crash, a hang or `ENGINE-INVARIANT`; each crash found is kept in `fuzz/regressions/`.
- **refcore** (`engine/tools/refcore/`) is the oracle of last resort, byte for byte.

## Where the engine does X

| To find | Look in |
|---|---|
| how a file is split into lines and tokens | `syntax/lex.rs` |
| how an expression parses | `syntax/expr.rs`, `syntax/reads.rs`, `syntax/ask.rs` |
| the canonical format and the hash | `syntax/canon.rs`, `syntax/canon_order.rs`, `syntax/spacing.rs`, `syntax/parens.rs` |
| the order of load's steps | `load/steps.rs` (`load_among`), then `load/strata.rs` (`build_graph`) |
| scenarios as worlds; joint models | `load/worlds.rs`; `load/joint.rs`, `load/merge.rs` |
| the namespace, `LOAD-DUP` | `load/names.rs` |
| an expression's type, a load refusal inside it | `load/types.rs`, `load/ops.rs`, `load/calls.rs` |
| who reads what | `load/check.rs` (`edge`), `model/graph.rs` |
| a load refusal's text | the check's file in the tables above; the code in `base/code.rs` |
| which row a read takes | `read/rows.rs`, `read/gap.rs` |
| which rule version is in force | `read/rules.rs`, then `eval/rules.rs` |
| a def's value and its memo cell | `eval/rules.rs`, `eval/memo.rs` |
| a hole's question | `hole/questions.rs`; where it is made, `read/gap.rs` |
| a plan instance's day and amount | `walk/plan.rs` |
| what happens on a day | `walk/day.rs`, `walk/order.rs` |
| a balance or a flow | `walk/reads.rs`, `walk/state.rs` |
| a projection `from t` | `walk/projection.rs` |
| observations, readings, anchors | `book/observe.rs`, `book/reading.rs` |
| adjustments and settlements | `book/adjust.rs`, `book/settle.rs` |
| the residual ledger | `book/ledger.rs`, printed by `ask/residuals.rs` |
| the register under a line | `ask/register.rs` |
| an ask in a reality with scenarios | `ask/worlds.rs` |
| rounding, money, printing a number | `num/round.rs`, `num/money.rs`, `num/text.rs` |
| an error code's text | `base/code.rs` |
