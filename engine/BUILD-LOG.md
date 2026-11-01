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
