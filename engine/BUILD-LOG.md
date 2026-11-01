# The build log

The build session's log: one line per step, newest last, and a STOP-REPORT when the
builder stops. The guard and the Stop hook append their own lines here (a blocked
settings change, "STOPPED RED"). The builder may write this file (R-79); nobody edits
a line once it is committed.

## How to stop: the STOP-REPORT

A STOP-REPORT is the builder's legal exit when a test, the spec or a rule seems wrong
(R-83). A turn can't end while `engine/tools/check fast` is red, unless a STOP-REPORT
heading was added to this file since the last commit, or by it. A rule change starts
here and goes through R-112's procedure: the lead reproduces it, the user approves it
in plain words, dated in the rulebook, and the `rules` role commits it.

Copy the template below to the end of this file, without the four-space indent (the
indent keeps the template itself from counting as a report):

    ## STOP-REPORT 2026-MM-DD: <one line: what blocks the build>

    - **Kind:** a test seems wrong | the spec seems wrong | a rule blocks correct code | other
    - **Where:** <file:line of the code, the test, the vector or the spec line>
    - **Rule or gate:** <R-n, or DONE gate n, or the spec's section>
    - **What happens:** <the command, quoted, with its exit code and the first error line>
    - **What I tried:** <the strict alternative tried, and why it doesn't work>
    - **What I ask:** <the change asked for, or the question for the lead>
    - **State:** <the last green commit; what is uncommitted>

## Log

## 2026-11-01 S plan (issue #26, gate G-S1)

**Approval.** The user, 2026-11-01: "Approve as proposed (Recommended)", to "G-S1: do you approve the S plan (tmp/coordinator-s/plan.md): 11 groups; kernel = num, read, walk, book plus hole/**; num/big.rs as the one big-integer file; explain, low and next deferred to M1 (where BUILD-ORDER's M1 builds them in full); the kernel rules commit right after this approval?" The kernel commit is 8fdd677 (rules role, R-85). The coordinator accepted the readings SQ-3 (the walk oracle is a builder test in `crates/tally-cli/tests/` that reads the S checks' inputs only, never a baseline) and SQ-4 (no thread at S; the counted depth is sized from U-15 to fit the caller's stack); SQ-5, -6, -7, -9 and -10 stand as read.

**Deferred to M1, with the checks that test them there:** the `explain` printer (`U:ask/ASK-05`, `U:val/VAL-03`; S builds the read sets and the register); the built-ins `low` and `next`, with `low`'s range minimum (`U:rule/RULE-02`, `U:rule/RULE-05`, X's changes). P8-M4 is measured at S as one point read of a written projection in E §4.3's common case.

**T's ids.** C1 `I:X:konto_balance@b_r1`, C2 `I:X:replay_september@b_r1`, C3 `I:X:replay_september@b_r1s` (run.py, byte for byte); R `R:all`; P1 `s_determinism_files_and_sections_in_any_order`, P2 `s_determinism_across_runs`, P3 `s_money_conserved`, P4 `s_no_foresight_past_the_cut`, P5 `s_no_foresight_in_a_decision`, P6 `s_no_foresight_through_the_written_projection`, P7 `s_a_hole_never_becomes_zero` (`s_properties`); H1 `s_hh4_konto_balance_at_b_r1`, H2 `s_hh4_replay_september_at_b_r1`, H3 `s_hh4_replay_september_at_b_r1s` (`s_household`, 75 vectors); MUT (gate 8 on the kernel); PERF (P8-14, P8-M1..P8-M4); ORA, the builder's own: the event walk equals the step-every-day walk on C1–C3 and the property corpus.

**Traceability.** Paths under `crates/tally/src/` unless they start `tally-cli/`; K marks the kernel.

| # | BUILD-ORDER § S item | Spec | T's ids | Lands in |
|---|---|---|---|---|
| B1 | Parser for X's base, its asks and the corpus forms | L §2.1–§2.3; the forms of L §5.1–§5.19 X writes | all | `syntax/**` |
| B2 | Evaluator: tree-walk, one `match` per node kind; counted recursion, then a work stack | D1-1, K8-1, R-16; E §3.2; L §5.9, §5.11, §7.1 | C1, H1 | `eval/**` |
| B3 | Memo: (item, key values, trimmed context); dense columns; one memo per context, no eviction; read sets | D1-2, D1-3, D1-5, D1-6, D2-7; E §3.2; PERFORMANCE §3 | C1–C3, P2, P8-14 | `eval/memo.rs` (fence F2), `eval/context.rs` |
| B4 | Plan pass; event walk over N-1..N-14; step-every-day oracle | E §4.1, §4.2; PERFORMANCE §2.1, §2.2; D1-4 | C1–C3, P1–P5, P7, H1–H3, ORA | K `walk/plan.rs`, `walk/agenda.rs` (F1), `walk/day.rs`; `oracle.rs` |
| B5 | Monthly copy-on-write checkpoints; interval measured | D2-6; PERFORMANCE §2.4, §10 U-6 | none reads them at S (SQ-5) | K `walk/state.rs` |
| B6 | Decisions with declared reads; order within a day | L §5.14.1, §5.14.2; E §3.1 steps 7–8, §4.2 step 3 | P5, P6, C1, H1 | K `walk/order.rs`, `walk/day.rs`; `load/graph.rs` |
| B7 | The written projection `bal(x, d) from t` (`low`, `next`: M1) | L §5.14.1; E §4.3; PERFORMANCE §2.3 | P6, C1, H1, P8-M4 | K `walk/projection.rs`, `walk/prefix.rs` (F5) |
| B8 | Numbers: `Num` small/big, whole-number shortcut, Lehmer; money in minor units; the cap; `round` | D2-1, D2-2, K8-2, K8-3; L §7.1, §7.2; PERFORMANCE §6 | every money line; P3; P8-M1..M3 | K `num/**` (F4; `num/big.rs` the one big-integer file) |
| B9 | The read rule with its cut | L §4.5, §4.6; E §2, §3.1 step 9 | P4, C1, K-3 | K `read/**` (F3 `read/index.rs`) |
| B10 | Booking at the cut, coverage, re-anchoring, adjustments and waivers, the residual ledger | E §5, §6.1–§6.3; L §5.14.3, §5.19, §5.18 | C1–C3, H1–H3 | K `book/**` |
| B11 | Holes, one module, one per-operator rule; the register (`explain`: M1) | L §8.1, §8.2; E §10; R-116 | P7, C1, H1 | K `hole/**`; `ask/register.rs` |
| B12 | Data layout: interner, `Table<I, T>`, B-trees, sorted slices, dense vectors | D2-3..D2-5, K8-6 | P1, P2 | `base/**` |
| B13 | Counters behind `--stats` | D4-8, K8-15 | P8-14 counts | `counters.rs`, `tally-cli/src/lib.rs` |
| B14 | Owner's kit: ARCHITECTURE.md, headers, trace, fences F1–F5 | D5-1, D5-4, D5-6; R-119, R-120, R-128, R-130 | the gate | every file; draft in `tmp/s/` |
| B15 | Release profile; overflow checks' cost measured once | D4-4 | P8 rows | measurement (the lead) |
| — | Codes (95), refusals, warnings | R-58, R-76; L §8.3–§8.5; E §11 | C1–C3 (W-1, W-2), P1 | `base/code.rs`, `base/error.rs`, `load/warn.rs` |
| — | Load as far as X needs | E §3.1 steps 1–9; L §3, §4.1–§4.3, §5.1–§5.10 | all | `load/**`, `model/**`, `time/**` |
| — | The CLI: `run(args, files)`, exit codes 0/1/2 | INTERFACE §1–§4; R-39, R-108 | all | `tally-cli/src/lib.rs`, `main.rs` |
| — | Asks and their printed form | L §5.18; E §1 | C1, P1–P7 | `ask/**` |
| — | Invariants 1, 2, 4, 5, 6 | E §9 | P3 | `invariant.rs` |

**Modules.** Kernel (gate 8, `tools/kernel.toml`): `crates/tally/src/{num,read,walk,book,hole}/**`, with `num/big.rs` the one file that names the big-integer types (R-29, R-108). Other library files: `lib.rs`, `api.rs`, `oracle.rs`, `counters.rs`, `invariant.rs`, `base/**`, `time/**`, `model/**`, `load/**`, `eval/**`, `ask/**`. Lexer and parser: `syntax/**` (exception budget 0). CLI: `crates/tally-cli/**` (its third exception: reading the reality's files). No thread module at S.

**Groups, in order:** G1 numbers; G2 foundations and the CLI's way in; G3 lexer and parser; G4 load and the read rule; G5 evaluator, memo, holes; G6 plan pass and event walk (P1, P3, P4, P5, P7); G7 written projection (P6, P2); G8 observations, readings, adjustments; G9 register and konto_balance (C1, H1); G10 residual ledger (C2, C3, H2, H3); G11 checkpoints, counters, fences, measurements.

**Risk noted for the gate (row 5):** T's S tests reach only small numbers, so the big tier's mutants (`num/big.rs`, `num/gcd.rs`) may survive T's tests at S. If they do, they are a T gap reported through the lead, not equivalent mutants.

## 2026-11-01 S, G1: numbers (issue #26)

**Built.** `crates/tally/src/num/` (kernel): `rational.rs` (`Num`: a reduced fraction, small `i64`/`NonZeroU64` or big, 16 bytes; demotion after every big result; the 4 096-digit cap by bit length, 10^4096 compared only at 13 607 bits), `small.rs` (128-bit intermediates, the whole-number shortcut), `big.rs` (the one big-integer file: `Big`, `BigRat`), `gcd.rs` (Euclid on words; Lehmer, Knuth 4.5.2 Algorithm L, with Collins's test; big by small), `round.rs` (the seven modes of L §7.2 at any positive step), `money.rs` (`Money` in minor units, `Word | Big`; currencies never mix), `text.rs` (literals read exactly; printing as refcore's `values.py`: groups of three, the shortest exact decimal, the sig15 aid). `lib.rs` names the files in an inline `pub mod num { … }`, since `mod.rs` is denied (`mod_module_files`) and the code stays under the kernel's `num/**`.

**Green.** No T id: G1 has none of its own; it feeds every money line, P3 and P8-M1..M3. The builder's 33 unit tests (dev and release), with Lehmer against a plain Euclid twin on 200 fixed-seed cases.

**Deviations to review.** `Num` has no `Ord` (BUILDER §2 names one): comparing two big values multiplies and can refuse at the cap's edge, so `compare` returns a `Result`. `Big` has no derived `Ord` either; `Big::order` is a named call (R-118).

**For the lead.** `tools/trace.py`'s `P8_ID` is `P8-\d+[a-z]?`, so a fence's `PERF:` line can't name P8-M1..P8-M4; F4 (`num/**`) would name a micro row. No fence is open at G1.

**Next:** G2, foundations and the CLI's way in. **STOP-REPORTs:** none. **Exceptions:** +1, kernel `num/big.rs` `struct Big`, `clippy::disallowed_types` (R-29); count 5, report-only 0. **Check:** `check full: green, 30 steps (320s)`.

## STOP-REPORT 2026-11-01: rules-selftest's R-111/cli-over-budget assumes the CLI holds 2 exceptions; its sanctioned third makes it count 5

- **Kind:** a rule blocks correct code (the rule is right; its selftest case is stale)
- **Where:** `tools/rules-selftest/cases/R-111.case`, case `cli-over-budget` (last changed in 045355d); the code that trips it, `crates/tally-cli/src/main.rs` `fn read_reality`
- **Rule or gate:** R-111 (area budgets), R-39 and R-108 (the CLI's sanctioned I/O file); `tools/cli-io.toml` names the three CLI exceptions: "the arguments, the reality's files …, and the output"
- **What happens:** `engine/tools/check full` → `check full: red, 1 of 30 steps failed (317s)`, the one failure `FAIL R-111/cli-over-budget: the output lacks 'area cli has 4 exceptions, over its budget of 3 (R-111)'`. The case plants two exceptions in `tally-cli` ("Planted too: the CLI holds two since its stdin read went (audit C1)") and expects the count 4. G2 adds the CLI's third, the reading of the reality's files that INTERFACE §4 gives the binary (`#[expect(clippy::disallowed_methods, reason = "R-39: the reality's files are read here, once, before the pure run takes them")]`, lock written, `exceptions: … cli 3/3`), so the planted run counts 5 and prints `area cli has 5 exceptions, over its budget of 3 (R-111)`. The budget check itself works.
- **What I tried:** nothing that changes the count honestly. Folding the reading into `arguments` or `write_output` would hide an I/O call under another exception's reason; leaving the binary to pass an empty table would make it refuse every real path `LOAD-PATH`. Both are workarounds, so neither is done.
- **What I ask:** a rules-role fix to the case: plant one exception (the CLI now holds its three) and expect `area cli has 4 exceptions`, or keep two and expect 5. Then G2 commits as it stands.
- **State:** last green commit 5c29cf0. Uncommitted, all green but this step (`check fast` green, 12 steps; `check full` 29 of 30): G2, foundations and the way in: `crates/tally/src/base/code.rs` (the catalogue, 99 codes), `base/refusal.rs`, `api.rs` (the read step, `run`), `lib.rs`; `crates/tally/tests/catalogue.rs` (the builder's test against L §8 and E §11); `crates/tally-cli/src/lib.rs` (arguments, the file table, paths and directories, exits 0/1/2/74, usage), `main.rs` (reads the files); `EXCEPTIONS.lock` (+1, cli). The release binary prints M0's fixtures `LOAD-PATH/alone` and `LOAD-PATH/before-parse` byte for byte, exit 1.

**Resolved** 2026-11-01 by `eb91a56` (rules role, issue #28; the user: "Rules commit on build/s (Recommended)"): `R-111/cli-over-budget` now plants 4 and holds from any live CLI count. G2 commits as it stood.

## 2026-11-01 S, G2: foundations and the way in (issue #26)

**Built.** `crates/tally/src/base/` (non-kernel): `code.rs`, the catalogue, one enum per family (`Hole`, `Load`, `Run`, `Warn`, `Req`, and `ENGINE-INVARIANT`), 99 codes: L §8's tables hold 95 (R-76's recount), E §11 adds `REQ-ASK`, `REQ-CLOCK`, `REQ-PATHS`, `ENGINE-INVARIANT`; `Code::ALL` in the tables' order; `refusal.rs`, a refusal with its code, message and place, printed as refcore's `TallyError.text`. `api.rs`: `run(files, request)` and its read step (every file by full path, the first `LOAD-PATH` or not-UTF-8 `LOAD-PARSE` in that order, L §8.3); past the read step it answers `NotImplemented` until G3. `crates/tally/tests/catalogue.rs`: the builder's test, `Code::ALL` against L §8.1, §8.3–§8.5 and E §11 (R-76). `crates/tally-cli/src/lib.rs`: the arguments as refcore's `command_lines` reads them, the netstring file table, a path as a key or a directory prefix of `.tally` keys, exits 0, 1, 2, 74 (69 while a run passes the part built), the usage text; `reprint` answers "not implemented" until its milestone. `main.rs`: reads every path into the table, walking directories as `os.walk` does (no links followed); a path it can't read is left out, so it is refused `LOAD-PATH`.

**Deferred, accepted by the coordinator:** `Table<I, T>`, the interner and `Span` land in G3 and G4, where they are first used.

**Green.** No T id. The release binary prints M0's fixtures `LOAD-PATH/alone` and `LOAD-PATH/before-parse` byte for byte, exit 1; `LOAD-PATH/control` needs the parser (G3).

**Gap noted:** a directory given with no `.tally` file under it can't be told from a missing path in the file table, so the engine refuses it `LOAD-PATH` where refcore reads no files; no check reaches it.

**Next:** G3, the lexer and parser. **STOP-REPORTs:** the one above, resolved. **Exceptions:** +1, cli `main.rs` `fn read_reality`, `clippy::disallowed_methods` (R-39); cli 3/3, count 6, report-only 0.  **Check:** `check full: green, 30 steps (310s)`.

## 2026-11-01 S, G3: the lexer and the parser (issue #26)

**Built.** `crates/tally/src/syntax/` (the lexer-parser area, budget 0), each file the twin of a part of refcore's `lexer.py` and `parser.py`, its error texts word for word: `lex.rs` (lines and their indentation, tokens: grouped numbers, dates, months, quarters, month-days, keys, currency codes, strings, operators); `tree.rs` (the tree: one enum per choice of the grammar, every node with its line); `parse.rs` (the cursor over one line's tokens, units, literals, spans, the depth count); `expr.rs` (expressions as flat chains, postfixes, atoms, binds, `bal`, `value`, `flows`, `covered`, aggregates, calendars, types); `item.rs` (the header and its checks, the sections, a version's lines, items and rows, the units prescan, `parse_reality`). `time/day.rs`: `Day`, a checked calendar day. `api::run` now splits every file, then parses every file (L §8.3), answers a parse refusal, and answers `NotImplemented` past the parse; `tally::parse`, the fuzz target's entry, runs the parser on one file.

**Green.** No T id of S's list. M0's fixture `LOAD-PATH/control` now prints byte for byte, exit 1 (with `alone` and `before-parse`, all three of M0's `LOAD-PATH` rows). X's 13 files (`b_r1`, and `b_r1s`'s asks) and all 312 realities of the kernel corpus parse, checked once with a temporary local test, not committed: the builder's committed tests read no T file (R-93). `crates/tally/tests/parse_cases.rs`: a reality in the corpus's forms parses to the expected sections, rows and asks; seven refusals print refcore's text (`LOAD-HEADER`, `LOAD-PARSE`, `LOAD-KNOWN`, `LOAD-RESERVED`, `LOAD-PLACEKIND`); a form built later is `NotImplemented`.

**Deviations to review.**
- Nodes carry their line number, not a byte span (BUILDER: a span on each node): refcore's messages name lines only. The interner and `Table<I, T>` come with the names they key, in G4.
- A construct the engine doesn't build yet stops the parse with `Stop::NotBuilt`, exit 69, never a guessed refusal: axes, fills, queries (`mean`, `share`, `quantile`), `low`, `crps`, `cdf`/`pdf`/`pmf`, distributions, `prorata`, `as if` (so `LOAD-ASIF` too), pins (`with` other than `with uncertain`), random asks and `random at`, the asks `explain`, `check`, `solve`, `diff`, `inputs-of`, `readers-of`, a version's `moves`, a kind's parts clauses, parts in a reading's row.
- `tally::parse` returns `Result<Vec<File>, Stop>` (it was `Result<(), NotImplemented>`); the fuzz target asks only `is_err()`, the stub bench only calls it.

**Spec gap, for the lead:** L §2.3's depth limit (`LOAD-DEPTH`, 8) counts brackets and `if`s; a read chained by `from` or `ties by` nests without either, so the grammar's recursion has no bound. The parser counts its own recursion there and stops at 64 with `NotBuilt` (R-16), not a refusal the spec doesn't name.

**Next:** G4, load and the read rule. **STOP-REPORTs:** none. **Exceptions:** +0; count 6; report-only 2, `lifetime-review` on the parser's cursor `P<'a>` (`syntax/parse.rs`, the struct and its `impl`): R-124's own case, a cursor over a text. **Check:** `check full: green, 30 steps (371s)`.

## 2026-11-01 S, G4a: time, units, types and the model's shapes (issue #26)

**Built.** `crates/tally/src/time/`: `day.rs` rewritten as refcore's ordinal (0001-01-01 is 1), ordered by its number, with weekdays, day and month steps; `period.rs` (days, months, quarters, years, their steps, shifts, labels, `age`, `age_months`, `overlap_days`); `calendar.rs` (weekends, fixed and yearly days, business-day steps). `crates/tally/src/model/`, the shapes load fills: `unit.rs` (units as products of powers, conversions, currencies), `ty.rs` (the types), `key.rs` (key values, patterns, spans), `kind.rs` (the prelude's kinds, entries, things), `index.rs` (indexes, versions, a version's entry into force), `decl.rs` (inputs, defs, bands, places, moves, rows, observations), `reality.rs` (the reality, its namespace, kind chains, lifespans under a cut). `crates/tally/src/base/table.rs`: `Table<T>`, items by name in the order added, no `Index` impl (BUILDER §3). `syntax/item.rs`: the header's range check compares days. Inline tests for each new file.

**Green.** No T id. **Next:** G4b, load's builders. **STOP-REPORTs:** none. **Exceptions:** +0; count 6; report-only 5 (+3, `type-parameter-review` on `Table<T>`, its `Default` and its `impl`: BUILDER §3's one expected own generic).  **Check:** `check full: green, 30 steps (396s)`.

## 2026-11-01 S, G4b: load, the parsed files become the reality (issue #26)

**Built.** `crates/tally/src/load/` (non-kernel, E §3.1 steps 1–4, refcore's `loader.py` its twin, messages word for word), in refcore's order: `steps.rs` (the header: one model header, one tally version; the built-in inputs `opening`, `stated`, `account_place`; names and `LOAD-DUP`; units and currencies), `kinds.rs` (the prelude and declared kinds, subkind chains and `LOAD-CYCLE` on a kind, entries of things and list keys in known order, `replace`, lifespans, base kinds), `indexes.rs` (the built-in calendars, list, open and int/day/month/year ranges, the open keys' prescan), `spans.rs` (spans, `governs`, versions with `cite` and `known`), `decls.rs` (inputs, defs, bands, places, moves: binds, `LOAD-SHADOW`, domains, types and units), `places.rs` (places, keyed places' tuples, a place reference), `keys.rs` (a key resolved in its domain, patterns), `rows.rs` (items: rules, band tables, rows with their pattern, governs and known; covers and gap roles; stable row ids, L §4.6), `observed.rs` (observations: fields, legs, amounts at the minor unit, `fulfils`, adjustments). `model/decl.rs`: a row's stable id, `InputInfo::new`; `model/reality.rs`: a kind's attributes through its chain. `crates/tally/tests/load_cases.rs`: the builder's test, a small reality loads to refcore's indexes, places, things with bases and lifespans, row ids, observation and `WARN-UNREAD` for a source; six refusals print refcore's text (`LOAD-SHADOW`, `LOAD-NAME`, `LOAD-KNOWN`, `LOAD-DUP`, `LOAD-KEY`, `LOAD-PRECISION`).

**Green.** No T id. Checked once against refcore with a temporary local test, not committed (R-93): X (`b_r1`) loads to the same indexes, places, things, lifespans, rows, givens, observations and moves, 237 normalized lines identical; all 312 realities of the kernel corpus load with refcore's things and bases, row ids and source warnings.

**Deviations to review.**
- Load is constructive: the refusals S makes are those construction meets; the rest of E §3.1's checks are M0's. So `api::run` still answers `NotImplemented` past the parse: a load refusal printed now could stand where refcore prints an earlier check's.
- Row values and observation attributes stay as written until G5's value types.
- `Stop::NotBuilt`, never a guessed refusal: scenarios, joint models, known windows, axes, a member's prefix, `same` lines, ranges of quarters (or any word but int, day, month, year), a band over values that aren't numbers or keyed by more than one key, a computed key of a move instance, a keyed place's key that isn't a literal; calendar arithmetic past 9999-12-31.

**Next:** G4c, the read rule (`read/`). **STOP-REPORTs:** none. **Exceptions:** +0; count 6; report-only 5.  **Check:** `check full: green, 30 steps (433s)`.

## 2026-11-01 S, G4c: the read rule (issue #26)

**Built.** `crates/tally/src/read/` (the kernel's, a plain scan; the row index is fence F3, opened when a budget row asks), refcore's `evaluate.py` its twin, messages word for word: `rows.rs` (the time point, a read with no law date, candidates: keys stated, `*` and ranges in index order, keys unstated that were there when the row was known, known by the cut, the span or instant; the choice: `replace` at the same keys, the narrower row, a span over `always`, ties `LOAD-OVERLAP`; `held`), `gap.rs` (no candidate: `none` outside a lifespan by the thing's membership hull, else `HOLE-UNGOVERNED`'s questions: the input's `question`, `no value for <period>`, `at <d>`, with no law date `no value`, a calendar's; a reading's plain read and `HOLE-NOREADING`; `read_input`), `rules.rs` (the version of a rule in force: `LOAD-OVERLAP`, `LOAD-LABEL`), `bands.rs` (the table a band read takes, refcore's recursion through earlier tables as a loop, `replace` rows patched in; the band's `held` mark and questions). A read returns the row, `none` or the question; the hole itself is built in the hole module (G5, R-116). `Lit::text` moved into the syntax tree (load's `lit_text` calls it). `crates/tally/tests/read_cases.rs`: 29 reads of a small reality and 5 tie refusals, each refcore's line from the same text (the refusals from the same change to refcore's loaded model, since its load refuses them first); a local probe, `tmp/s/refread.py`, prints refcore's lines.

**Green.** No T id.

**Deviations to review.**
- `last` and `since` are left for the evaluator: neither X nor the corpus reads them.
- A band `replace` table that stands as a whole table (no table under it, or rows that don't match the earlier intervals) is `NotBuilt`: telling it needs load's band checks (`whole_ok`), which are M0's.
- A dated read with no law date is `NotBuilt` (`LOAD-LAWDATE`, M0's), where refcore's interpreter guard raises.
- Cut classes come with the memo (G5), where they trim cells: an item's class is per its cone, which the item graph (G4d) gives. The calendar built-ins are `time/`'s (G4a); the evaluator binds them (G5).

**Next:** G4d, the item graph. **STOP-REPORTs:** none. **Exceptions:** +0; count 6; report-only 5.  **Check:** `check full: green, 30 steps (450s)`.

## 2026-11-01 S, G4d: the item graph (issue #26)

**Built.** The item graph, E §3.1 step 6 and L §8.5, refcore's checker its twin (`check.py`'s edges, reads, relations, binds, aggregates, band calls and cycles; `check3.py`'s move checks, flows, balances, the move graph and the strata), messages word for word. `model/graph.rs`: nodes (a kind word and an id, ordered by the word, then the id, as refcore sorts them), the relation of a read (`same`, `shift:<var>:-/+`, `computed`, `any`, `bind`, `named`, `flows`, `bal`, `explicit`, `calendar`, `next`, `observed`), marks (`balnow`, `bal`, `proj`, `flows`, `legs`, `legof`) and their targets, and the `Graph`: edges, marks, the projection holders and what each leaves out, the items read from decisions, the walk-bound defs, the strata's `after`, the written order and the order groups. `load/graph.rs`: the walk over every item in refcore's order (price clauses, inputs' ranges, moves by id, defs, bands, asks, `prices … by`), a variable's range and the keys of the item it sits in deciding each read's relation; recursion counted to 512 (`ENGINE-INVARIANT` past it). `load/strata.rs`: the graph built last in load: same-key cycles (`LOAD-CYCLE`), `WARN-UNREAD`, the decisions' reads and changes, `after` from what a decision reads that another changes, the written `before`/`after` (`LOAD-NAME`, `LOAD-ORDER`), cycles among decisions merged into order groups or refused (`LOAD-ORDER` when every one is on a known day), the levels and `self_reads`; every search an explicit stack. `MoveInfo` gains `read_targets`, `changes`, `self_reads`; `Reality` gains `graph`. `crates/tally/tests/graph_cases.rs`: the builder's test, a small reality's graph as refcore's 62 lines (three decisions in strata 1–3 by a written `after` and a read of legs, a projection in a def and in an ask, a recurrence, `next`, two unread items), what a projection leaves out, and 5 refusals with refcore's text.

**Green.** No T id. Checked once against refcore with a temporary local test, not committed (R-93): X (`b_r1`), 401 normalized graph lines identical (edges, marks, holders, left-out sets, items from decisions, walk-bound defs, strata, warnings); all 312 realities of the kernel corpus, 14 058 lines identical, none refused on either side.

**Deviations to review.**
- The graph's refusals are those building it meets: a same-key `LOAD-CYCLE`, `LOAD-ORDER`, and `LOAD-NAME` for an unknown move in an order line. `LOAD-READS`, `LOAD-GUARD`, a cycle through `beats`, and the checker's type and unit refusals are M0's.
- A leg's place written as an `if` gives an unknown target, where refcore's checker unifies the branches' types.
- A membership test's list reads no item; a key that doesn't resolve names no literal place (M0 refuses it, `LOAD-KEY`).
- The parser bounds expression depth; the walk counts its own to 512 as an invariant.

**Spec gap for the lead.** L §8.5 and E §3.1 step 6 name places nothing reads among `WARN-UNREAD`'s items; refcore warns only inputs and defs. The engine follows refcore.

**Next:** G5, the evaluator, the memo and holes. **STOP-REPORTs:** none. **Exceptions:** +0; count 6; report-only 5.  **Check:** `check full: green, 30 steps (476s)`.

## 2026-11-01 S, G5a: the evaluator, holes and the memo's cells (issue #26)

**Built.** The evaluator, E §3.2, refcore's `evaluate.py` its twin (with `eval2.py`'s rates and `check.py`'s static types where the run needs them). `hole/questions.rs` (kernel, R-116): a hole as the sorted set of its questions with the figure each was reached at, the one per-operator rule (`spread`), L §8.1's print, the empty bound slot (seam U). `model/value.rs`: runtime values; a number carries the class refcore's checker gives the expression that made it (`int`, `num u` shown plainly or with `%`, `rate`, a bare figure), money is whole minor units; how each prints. `eval/state.rs`: contexts (law date, cut), scopes that keep each key variable's domain, the register's collectors, the memo of cells (def, keys, context) holding a value or a refusal with what it met, `RUN-CYCLE` from the cells in progress, a counted depth of 400 def cells. `eval/arith.rs`: arithmetic deciding the result's class from its operands' as check.py's `_arith`; time and key arithmetic; comparison. `eval/read.rs`: keys made keys of their positions (`RUN-KEY`, keys added after the cut), rows' values with `assumed` and `held`, `prices Y by`, rows' records and calendars. `eval/rules.rs`: the law date by `at`, a bind's `where`, givens, rules in force, the `beats` order, `when` under Kleene logic with unknown rules' holes, `RUN-CONFLICT`, `RUN-NO-RULE`, the `assumed`, `given` and `missing` entries, the def's type. `eval/expr.rs`: one arm per node kind. `eval/builtin.rs`: the calendar and date built-ins, business days read from the calendar in force on each day, `round` with its result's type (L §7.2), `min`, `max`, `abs`, `pos`, `missing`, `periodic` and `factor` exact for an integer power. `eval/band.rs`: the table and the row holding the argument, the band's `assumed` and `held`, the row's value in the band's type. `eval/aggregate.rs`: `sum`, `count`, `min`, `max`, `any`, `all` over bindings (an index's keys at the cut, a kind's things `on` a day or `over` a span by `kind:key`, lists, ranges), a `where` that is a hole making an unknown member, tuples depth first in refcore's order. `eval/ty.rs`: a sample value of an expression's class where the run can't see one (a sum over no keys, a bare figure that is an `if`'s branch). `crates/tally/tests/eval_cases.rs`: the builder's test, 25 reads of a small reality as refcore's (holes in sums and logic, a band's ends, `round` to a num, an int and money, a rate's period and factor, `missing` in a cited rule, `beats` with a `when` that is a hole, `RUN-EMPTY`), holes that stay holes, a cell computed once.

**Green.** No T id. Checked once against refcore with a temporary local test, not committed (R-93): X (`b_r1`), every def keyed by indexes and kinds read at up to three keys (law = cut = `on`), 138 reads: 109 values and registers identical; 4 need the walk (`bal`, `flows`: G6); 25 refcore itself didn't finish (Python's recursion limit on the loan's 241- and 480-deep chains; a deeper stack ran past 15 minutes and was stopped), where the engine gives 4 values, the walk's `NotBuilt` for 17 and the depth limit's for 4.

**Deviations to review.**
- Plan §3 G5 not yet built, now G5b: the memo's dense day- and period-keyed columns and context trimming (PERFORMANCE §3.3; the key is the full context now); the explicit work stack past the counted depth (past 400 def cells a read stops `NotBuilt`, as `debt_before[laptop, 480]` does); asks that print values (L §5.18).
- The depth and the stack (U-15): a debug test thread's 2 MB overflows on the loan's 240-deep chain; the probe ran in a 1 GB thread. The limit is to be measured against the stack the thread module passes.
- Static types come from sample values (`ty.rs`), not a checker: where a sample's class can't be seen the run stops `NotBuilt`, never guessing a zero.
- `NotBuilt` until their groups: reads of the walk (`bal`, `flows`, `value`, `covered`, bindings over movements and observations), `first`, `last`, `argmin`, `argmax`, `to u`, `last(x, d)`, windows, a power that isn't an integer, a kind's `on`/`over` that reads its own variable.
- The checker's load refusals (`LOAD-UNIT`, `LOAD-LITERAL`, `LOAD-MISSING`, …) are M0's; where the run meets one it refuses with the value's text where refcore names a type.

**Next:** G5b (the memo's columns and trimming, the work stack, asks that print values), then G6. **STOP-REPORTs:** none. **Exceptions:** +0; count 6; report-only 5.  **Check:** `check full: green, 31 steps (529s)`.

## 2026-11-01 S, G5b: the work stack and the asks that print values (issue #26)

**Built.** The rest of plan §3 G5 that S's checks need. `eval/stack.rs`: the outermost reads (`Engine::value`, `read_def`, `tuples`, `line`); past the counted depth a def cell waits on the work stack (BUILDER §4): `Stop::Deep` unwinds the read, the cells that unwound stay in progress under the task they wait on, the outermost read computes the tasks from depth 0, the latest first, then starts again on the memo; a ring of computed keys deeper than the limit is `RUN-CYCLE` at the cell refcore names. `eval/state.rs`: the task and the work stack, `DEPTH_LIMIT` 32 (U-15 below); `ev` and `read_def_at` are the evaluator's own. `ask/answer.rs`, `ask/register.rs` (refcore's `ask.py` their twin): the asks in id order or as the run names them (`LOAD-NAME no ask <id>`), each with its own engine; `as known`, `law`, `in <the model>` (`world:`), `for … where` tuples (a domain or a `where` that is a hole prints as that hole), a line's values with a run-time refusal standing as its value, the register (`assumed` merged by reason, `given`, `missing`, `world`, the walk's words, `held`, then `law date <d>, cut <K>` or `cut <K>`), the warnings last, sorted by code and text. `api::run`: read, parse, load, then the asks. Literal keys: `load/keys.rs` `index_key_of` (refcore's `resolve_key_literal` on an index); the evaluator's key literals carry the literal and resolve through it, where before an integer key (`acc[3000]`) or a period by its label was refused `LOAD-KEY`. Tests: `deep_cases.rs` (a 3 000-deep recurrence, a loan schedule's three defs 600 deep through `round`, `min` and `max`, a 150-cell ring of computed keys, each as refcore prints it); `ask_cases.rs` (nine asks of a small reality and `--ask`'s order, as refcore prints them); `api.rs` and `register.rs` inline.

**Green.** No T id: every S check reads the walk.

**Measured, U-15.** A debug build uses about 19 KB of stack per def cell: 19 025 to 19 057 bytes on the three chains, from the lowest stack address `def_cell` reached on a 1 GB thread (a probe, not committed). T's runner gives a run a 2 MiB thread, so about 110 cells fit; the limit is 32 (about 0.6 MB), leaving the rest for expressions between cells and, from G6, the walk. `deep_cases.rs` runs on libtest's 2 MiB thread: it aborts with a stack overflow at a limit of 1 000 (checked) and passes at 32. To measure again on X once the walk runs.

**Deviations to review.**
- Plan §3 G5's dense day- and period-keyed columns are fence F2, whose row is P8-14 (plan §4); R-130 and BUILDER §13.1 open a fence when a budget row asks, and P8-14 runs once C2 does (G10). Until then a recurrence goes through the work stack: no deep stack, a chain's cells computed about twice. The fill in key order (E §3.2 l.155) comes with F2.
- Trimming: PERFORMANCE §3.3 trims a projection out of a cell's context; projections come with G7, and the trimming with them. The cut and law-date classes of E §3.2 are cache choices measured at G11 (U-5). Two facts for whoever builds them: `cut` reads the cut itself, so a cone that reads it shares no class; a hole's question and `RUN-CONFLICT` name the law date, so sharing a cell across a law-date class would print another day than refcore, whose memo is keyed by the exact law date and cut.
- Read sets per cell: G8, with adjustments, their first reader.
- A load refusal, at load or met at run, makes the run `NotImplemented`: load's checks are M0's, and refcore may refuse first with another.
- An ask's `with uncertain` is taken and not yet read: the walk reads it (G6, G7).
- BUILDER §4's "demand an aggregate's domain first, before a replay" is not done: an aggregate whose terms each pass the depth replays once per term, its earlier terms memo hits. Measured at G11 if a row asks.

**Next:** G6, the plan pass and the event walk. **STOP-REPORTs:** none. **Exceptions:** +0; count 6; report-only 5. **Check:** `check full: green, 31 steps (473s)`.

## 2026-11-01 S, G6: the plan pass and the event walk (issue #26)

**Built.** Plan §3 G6, E §4–§5, refcore's `walk.py`, `moves.py` and `mreads.py` the twins. `walk/point.rs`: a point of the walk, `At` (day, step, stratum, rank, instance, sub), ordered as refcore's walk key; the batch's and the closing's check points. `walk/plan.rs`: the plan pass, a movement's instances from its tuples and its law (cached, `RUN-CYCLE` on a movement reached again), each leg's ends and amount, the status (past, plan, hole), the order of plan instances as refcore's text (`<move>|<key tuple>`), units of a place, `ENGINE-INVARIANT`. `walk/state.rs`: the walk's log of changes per place and unit, state and cumulative sums, unbooking, the opening balances, `Progress` (planning, walking, done, failed). `walk/agenda.rs`: the agenda (readings, plan legs, decisions, adjustments, lifespan ends), the next day with work, and the oracle's every day (`oracle.rs`, `Days::Every`). `walk/day.rs`: a day's steps in order, the sign rule's groups (the first batch, a running decision, the closing), `RUN-LIFESPAN`, conservation at the close (`ENGINE-INVARIANT`), `WARN-NEGATIVE` on the accounts and cash a day touched. `walk/order.rs`: decisions on a day, ordered by `ties by`, `RUN-TIE` where a tie changes the result, each run's reads at its own point (`WalkAt` in the context's key). `walk/reads.rs`: `bal` (with `with uncertain`), `flows` of a place or a movement, a decision's runs, the main walk to the cut, resumable after the work stack's `Stop::Deep` (the day's cursor resumes at its next act). `book/adjust.rs`: E §5's adjustments, a past or hole instance before the cut measured at each version's known date, its δ per leg, `HOLE-CHANGED` and the remainder's text. `eval`: the context's walk point, versions read, the walk's state on the engine. `api::Request.days`. `crates/tally-cli/tests/walk_oracle.rs`: the walk oracle (ORA).

**Green.** P1 `s_determinism_files_and_sections_in_any_order`, P3 `s_money_conserved`, P4 `s_no_foresight_past_the_cut`, P5 `s_no_foresight_in_a_decision`, P7 `s_a_hole_never_becomes_zero`. ORA: the event walk and the every-day walk print the same bytes on all 312 records of `tests/properties/kernel.corpus`; the 72 `foresight-projection` records are `NotImplemented` on both (G7). Checked once against refcore with a temporary local test, not committed (R-93): the 240 records answered equal `kernel.refcore` byte for byte, warnings compared as a sorted block. `kernel.refcore` lists warnings in the order met, from before 2.1.1; refcore today sorts them, as the engine does (a note for T: the stored file is stale in that one respect).

**Deviations to review.**
- Kept as refcore does it: a leg whose place is a hole and whose amount is known books one end, and the close's conservation check refuses `ENGINE-INVARIANT` (the STOP-REPORT below).
- Plan §3 G8's adjustments are built here as far as the corpus reaches them (no settlements, no waivers): the corpus's later versions change past instances, and the walk can't book those without them.
- `NotBuilt` until their groups: observations and coverage, stated readings, a written order beside a plan, a cycle of decisions on one day, a decision whose day is a hole, settlements (`fulfils`), a version answering a hole movement, `flows … for k` and converted flows, projections, modelled and window reads (G7), `bal` of a quantity, partial movement keys, a leg booked outside every sign-rule group (pads, G8), a leg read whose walk needs a cell still being computed (refcore's prefix walk).
- E §9 invariant 2 (net worth) is not checked: it has no effect on output; with G11's checks.
- The memo's dense columns (fence F2) stay G11 and per-cell read sets G8, as G5b left them. The walk's agenda is a sorted list per walk; checkpoints are G11's.

**Next:** G7, the written projection (P6 `s_no_foresight_through_the_written_projection`, P2 `s_determinism_across_runs`: the corpus's projection records answer). **STOP-REPORTs:** one, below; it doesn't block the build. **Exceptions:** +0; count 6; report-only 5. **Check:** `check full: green, 31 steps (654s)`.

## STOP-REPORT 2026-11-01: a leg whose place is a hole refuses ENGINE-INVARIANT in refcore, where L §5.14 says it books its known side

- **Kind:** other: the oracle and the spec disagree, on an input no T check reaches.
- **Where:** refcore `tools/refcore/walk.py:1234` (`whole = all(on for p, s, on in sides)`: an end whose place is a `Hole` counts as on, so the leg is taken as booked whole and only its known end joins `cons`); the engine's twin, `crates/tally/src/walk/day.rs` (`book_leg`, then `close_day`'s conservation check); the spec, L §5.14 Legs (l.1353: "the leg books its known side, and every read of the unknown side is a hole naming it") and E §9 invariant 1 (l.425: "the two ends of each leg booked whole sum to zero …; a leg booked by its ends … is left out").
- **Rule or gate:** E §9 invariant 1; DONE gate 10 (refcore's bytes); the skill's "never a crash: a broken invariant is a bug".
- **What happens:** the smallest input (`tmp/s/hp/reality.tally` in the worktree): places `acc_0`, `acc_1` (`accounts DUK`), `input dest: key accounts` with no row, `move mv_0: transfers` on 2025-01-10, `leg cash from acc_0 to dest amount amt` (`amt = 100.00`), openings 500.00 and 0.00, asks `bal(acc_0, 2025-01-31)` and `bal(acc_1, 2025-01-31)`. `python3 -I -B engine/tools/refcore run reality.tally` prints, exit 0, for both asks: `refused: ENGINE-INVARIANT conservation: the legs booked on 2025-01-10 in DUK sum to -100.00 DUK, not 0 (E §9 invariant 1)`; the engine (`target/release/tally run reality.tally`) prints the same bytes. By L §5.14 `a0` reads 400.00 and the unknown side's reads are holes naming `dest`; nothing in the spec refuses.
- **What I tried:** nothing in code: the engine keeps refcore's bytes (gate 10), and no corpus record or T check reaches a hole place (no `ENGINE-INVARIANT` in `tests/properties/kernel.refcore`).
- **What I ask:** the lead's ruling: either refcore leaves a leg with a hole place out of conservation, as invariant 1 leaves out a leg booked by its ends (then T adds a check, and the engine follows: one line in `book_leg`), or the spec says such a leg refuses, and with which code. What `bal` of a place that may be the unknown side reads (here `acc_1`) is the same question's second half.
- **State:** the last green commit is this G6 commit; nothing uncommitted under `engine/`.

## 2026-11-01 S, G7: the written projection (issue #26)

**Built.** Plan §3 G7, L §5.14.1 and E §4.3, refcore's `mreads.py` (`proj_bal`, `proj_regs`, `proj_uncertain`) and `walk.py` (`proj_walk`) the twins. `walk/projection.rs`: `bal(x, d) from t` as a walk of its own from day t: from the main walk at the start of t (an ask, or a def no decision reaches), from the reading decision's walk at its point on t, or at the start of an earlier t; a later t is `RUN-FUTURE`, d before t `RUN-DOMAIN`, t before S `HOLE-PRERUN`; its cut min(t, K); one walk per (walk continued, point, cut, moves left out), walked only as far as the read's day, as refcore's `run_to`; `left out:` for each move it leaves out with an instance dated in its window; `with uncertain`, the read's or the ask's. `walk/state.rs`: a projection's walk starts each place as the walk it continues stood at its point (`Projected`, the chain's states nearest first, the recursion counted by the chain's level). `walk/agenda.rs`: its plan pass reads no reading and makes no adjustment, counts the instances of the moves that depend on the holder instead of booking them, skips plans dated before t, keeps decision ranks global so points compare across walks, and on a decision's day keeps only the decisions after its point. `walk/day.rs`: `advance_to` (a walk paused after a day), the plan pass in its own collector, no `WARN-NEGATIVE` in a projection's walk. `walk/reads.rs`: a read of a projection's walk gathers the register and `left out:` of the walks it continues, while no reset is seen (refcore's `merge_cum`). `eval`: the holder of a projection (the ask, def, band or move whose expression is evaluated; the item graph gives what it leaves out), the ask's `with uncertain`. `crates/tally/tests/projection_cases.rs`: the builder's test, 16 asks of a small reality as refcore prints them, under both day sources (a def's projection read by a decision and by an ask, a decision's own projection, one from an earlier day, nested projections, a row known between t and K, `RUN-FUTURE`, `RUN-DOMAIN`, `HOLE-PRERUN`, a hole start, `with uncertain`, a projection below zero that warns nothing, a refusal after the read's day that refuses nothing). `crates/tally-cli/tests/walk_oracle.rs` now requires every corpus record answered.

**Green.** P6 `s_no_foresight_through_the_written_projection`, P2 `s_determinism_across_runs`. ORA: all 312 records of `tests/properties/kernel.corpus` answered, the event walk and the every-day walk alike. Checked once against refcore with the release binary over the corpus's realities, not committed (R-93): all 312 records equal `kernel.refcore` byte for byte, warnings compared as a sorted block (the stored file's order predates 2.1.1, as G6 noted).

**Deviations to review.**
- Fence F5 (the common case: start balance plus a prefix sum F) is not opened: R-130 and BUILDER §13.1 open a fence when its row asks, and P8-M4's bench is the lead's (R-91). Every projection is the plain walk of its window, which F5 will keep as its twin.
- `low` and `next` are M1's (G-S1). A projection's `windows each` and `random` clause are `NotBuilt`.
- PERFORMANCE §3.3's trimming needs no code: the context has no (holder, t) slot; a cell evaluated inside a projection's walk is keyed by its cut and, when walk-bound, by the walk's id and point, which is the trimming E §3.2 and §3.3 derive.
- A projection whose holder the evaluator doesn't track (a def's `at` or a key bind's `where` are evaluated outside the def's cell) takes the outer item's holder; load's `LOAD-READS` (M0's) refuses a projection in a move's `on`.

**Gap for the lead** (no check reaches it). When refcore cuts day t of a projection's walk at a decision's point, it compares the run's key `'<move>|%06d'` with each decision act's `<move>` as text, so of two decision moves in one stratum whose ids differ only by a suffix starting with a character before `|` (`sw`, `sw_b`), it drops the later one's act; the engine compares walk points (moves ranked by id) and keeps it, as L §5.14.1 says ("it counts only the movements not yet booked"). Two moves in one stratum read nothing the other changes, so only a hole or a refusal of the later one that holds for every place could show the difference.

**Next:** G8, observations, readings, adjustments and settlements (X's base needs them before C1 and H1). **STOP-REPORTs:** issue #31 (the hole leg place, above) waits for the user; refcore's bytes are kept. **Exceptions:** +0; count 6; report-only 5. **Check:** `check full: green, 31 steps (682s)`.
