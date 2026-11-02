# The Tally engine's architecture

The map of `engine/crates/`, for the owner, who audits the engine through this page, each
file's `//!` header and the trace from code to spec (`engine/tools/trace --map`). It names
modules and never links them. Written at milestone S (2026-11-02); updated only at a
milestone (R-119). The spec is the authority: L is `spec/LANGUAGE.md`, E `spec/ENGINE.md`,
P `spec/PERFORMANCE.md`. The twin of each file, refcore's file it mirrors, is on its
header's `Twin:` line.

## Bird's-eye view

The engine is a pure function. A reality (Tally text files) and a request go in, as bytes;
the printed answer comes out, the same bytes refcore prints (DONE gate 10). Think of it as
a compiler and an interpreter in one, with one twist: time. Every value is read "as known
at a cut, under the law of a law date", and the world moves day by day.

A run is a pipeline:

1. **read** the files, in the order of their full paths;
2. **parse** each into a tree (`syntax`);
3. **load** the trees into one resolved, checked reality (`load` building `model`);
4. **answer** each ask (`ask`), which pulls values on demand: the evaluator (`eval`)
   computes def cells into the memo, reads inputs through the read rule (`read`), and asks
   the walk (`walk`) for balances and flows; the walk books legs, observations and
   readings (`book`); numbers are exact (`num`); a missing fact is a hole (`hole`);
5. **print** the lines, the register under each, then the warnings.

Nothing is computed ahead of a read: an ask reads a def, the def reads a balance, the
balance needs the walk up to its day, the walk needs the plan's amounts, which are defs
again. The memo and the walk's own caches make each piece computed once per context.

## Codemap, by pipeline stage

Paths are under `crates/tally/src/` unless they say otherwise. **K** marks the kernel, the
code DONE gate 8's mutation score covers (`tools/kernel.toml`).

**The way in.**
- `lib.rs`: the library's face; each module block names its files and its spec section.
- `api.rs`: `run` and `run_with_memo`: read, parse, load, answer (E §1; L §8.3 for which
  refusal prints first).
- `oracle.rs`: the walk's day source: the event walk, or every day (the test oracle).
- `counters.rs`: the `--stats` counts, in the `counters` build only (BUILDER §13.2).
- `invariant.rs`: E §9 item 4, asks move no money: each main walk's signature, kept
  once it is walked and checked again after each ask.
- `crates/tally-cli/src/lib.rs`: the `tally` command as a function, `run(args, files)`:
  arguments, paths, exit codes 0/1/2 (INTERFACE §1–§4).
- `crates/tally-cli/src/main.rs`: the binary; the one file that reads files and writes
  output.

**Foundations** (`base/`). `code.rs`: the catalogue's 99 codes (L §8's 95 and E §11's
four), one enum. `refusal.rs`: a refusal carries its code, message and place. `table.rs`:
the one id-keyed store, with a checked `get` and no indexing.

**Text** (`syntax/`, the lexer-parser area, exception budget 0). `lex.rs`: lines, indent
levels, tokens. `parse.rs`: the parser's cursor over one line. `expr.rs`: expressions by
precedence climbing. `item.rs`: the file grammar, header, sections, version items. `tree.rs`:
the tree, one type per production.

**Time** (`time/`). `day.rs`: a checked calendar day. `period.rs`: months, quarters, years,
their steps. `calendar.rs`: business days.

**The model** (`model/`): what load leaves. `reality.rs` holds the whole: the namespace,
kinds and things (`kind.rs`), indexes and versions (`index.rs`), keys and spans
(`key.rs`), declarations and what versions state for them (`decl.rs`), units (`unit.rs`),
static types (`ty.rs`), runtime values (`value.rs`), and the item graph (`graph.rs`: who
reads what, the running reads, what each projection leaves out, the decisions' strata).

**Load** (`load/`, E §3.1). `steps.rs` runs the steps in refcore's order: the built-ins,
names and units (`steps.rs`), kinds (`kinds.rs`), indexes (`indexes.rs`), versions and their
spans (`spans.rs`), the sources no version reads, things (`kinds.rs`), declarations
(`decls.rs`), open keys (`indexes.rs`, `keys.rs`), places (`places.rs`), the version items:
rows, givens and observations as movements (`rows.rs`, `observed.rs`), the asks, then the
item graph (`graph.rs`) and what is decided from it (`strata.rs`: cycles, unread items,
walk-bound defs, strata).

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

**The asks** (`ask/`, L §5.18). `answer.rs`: each ask's cut, law date, tuples and lines.
`register.rs`: the register under a line, then the warnings. `residuals.rs`: a `residuals`
ask's groups and lines.

## Cross-cutting ideas

- **Contexts.** Every value is computed in a context: a law date, a cut, and inside the
  walk a walk and a point; an ask line's own reads also carry its `with uncertain`, which
  no decision or timeline sees (L §5.14). The memo keys a def cell by (def, keys, cut, walk
  point, `with uncertain`) and keeps one cell per law-date class: the law dates whose
  answer can't differ, found from the spans of the rows and rules the cell actually read
  (E §3.2). A text that names the day (a hole's question, a `held` entry) keeps the cell at
  its own day.
- **Read sets.** Each def cell records what it read: the inputs, rows, bands, band rows and
  cells (E §3.2, P §4). What a walk, a plan, an adjustment or a link reads is shared by
  every reader and is not in a cell's set. At S nothing skips work by them yet; they are the
  seam for `explain`, `inputs-of`, `readers-of` and the change-cost work.
- **The walk is a fold over days,** stepped only on days with an act (P §2.2). One main
  walk per cut; a projection walks its own window from the state of the walk it continues.
  Balances are logs of change points, so `bal(x, d)` is a binary search.
- **Checkpoints.** At each month's start a main walk keeps every place's state, shared
  with the month before where it didn't change, and the decision legs booked ahead. They
  are for the forks (P §2.5) and the walk restart (P §4.2); at S nothing starts from one.
- **Holes are values,** carried by every operator by one rule, and printed with their
  questions. A missing value is never 0.
- **Refusals** are values too: a code from the catalogue and a message; a broken
  invariant is `ENGINE-INVARIANT`, never a crash.
- **Recursion** is counted (`DEPTH_LIMIT` 32); deeper chains wait on an explicit work stack.

## Invariants, and what is deliberately absent

- **Exact.** No floating point anywhere (R-18). Integers only through checked calls; an
  overflow or the 4 096-digit cap is a refusal with its code.
- **Deterministic.** No hash map or set (B-trees, sorted slices, dense vectors); no clock,
  environment, file or network in the library; output ordered by explicit keys, never by
  an address or insertion order; paths sorted by code point.
- **Never a crash.** No `unsafe`, `unwrap`, `expect`, `panic!`, indexing or slicing in
  the library; `ENGINE-INVARIANT` where the engine finds itself wrong. Checked after each
  day: money conserved (E §9 item 1), net worth explained (item 2), nothing held outside a
  lifespan (item 6); a plan instance the same under every cut that books it (item 5); asks
  move no money (item 4, `invariant.rs`, after each ask).
- **A small Rust.** No `dyn`, no `impl Trait` arguments, no macros of our own, no own
  generics, nesting at most 5 (R-122 to R-125, R-73, R-74). Enums and exhaustive `match`.
- **No threads at S.** The run happens on the caller's thread. The thread module comes
  with Monte Carlo (M7): scoped threads, explicit stack size, merge in task order.
- **Absent at S, on purpose:** `explain`, `low` and `next` (M1); parts and labels (M5b);
  joint members (M5c); paths and timelines (M7); the walk restart (the change-cost work,
  P §4). Deferred to M1 by the user (G-S2, 2026-11-02): the interner of names, the
  id-keyed `Table<I, T>` (today `Table<T>` keyed by name) and the context's layer slots.
  Fences F2's dense columns, F3 (the read index) and F5 stay closed until a budget row
  asks (R-130).
  A construct S doesn't build answers `NotImplemented`, never a guess.

## Boundaries

- **Library against CLI.** `crates/tally` is pure: bytes in, lines out. `crates/tally-cli`
  turns arguments and files into a `Given` list and the answer into stdout and an exit
  code. Only `crates/tally-cli/src/main.rs` reads files, arguments and writes output (R-39,
  R-108); it holds the CLI's three exceptions.
- **The kernel** (`num`, `read`, `walk`, `book`, `hole`): the code gate 8's mutants run
  over, with an exception budget of its own; one exception (the big-integer wrapper in
  `num/big.rs`, R-29).
- **The big-integer crate** is named in one file, `num/big.rs`.
- **The thread file:** none at S.
- **Fences** (R-130): optimised code only where a budget row asked, each with a `PERF:`
  line, a plain twin and an equivalence test:
  - F1 the event walk (`walk/agenda.rs`, P8-14): twin `oracle.rs`'s every-day walk;
    checkpoints in `walk/state.rs`, twin the log itself;
  - F2 the memo's law-date classes (`eval/memo.rs`, P8-14): twin `Memo::Exact`, refcore's
    keying;
  - F4 the number tiers (`num/small.rs`, P8-M1; `num/gcd.rs`, P8-M2): twins the big tier
    and Euclid.

## Tests, and who decides "done"

- **T's tests** in `engine/tests/` decide: the checks against refcore's bytes, the
  household vectors, the kernel's properties. The engine never reads them (R-93).
- **The builder's tests** make the internals easy to get right and count toward no gate:
  inline `tests` modules, `crates/tally/tests/*_cases.rs` (small realities, refcore's lines),
  `crates/tally/tests/catalogue.rs` (the codes against the spec's tables), and
  `crates/tally-cli/tests/walk_oracle.rs`, the twins compared on the kernel corpus and on
  X's three S checks: event walk against every day, memo classes against exact days.
- **refcore** (`engine/tools/refcore/`) is the oracle of last resort, byte for byte.

## Where the engine does X

| To find | Look in |
|---|---|
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
| rounding, money, printing a number | `num/round.rs`, `num/money.rs`, `num/text.rs` |
| an error code's text | `base/code.rs` |
