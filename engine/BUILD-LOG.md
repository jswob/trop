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
