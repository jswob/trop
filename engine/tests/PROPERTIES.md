# The engine's properties

DONE gate 6 holds the engine to laws, not to stored answers: things that must be true of every run. This page is the **kernel's part**, the laws milestone S must turn green (BUILD-ORDER § S, "the kernel's properties"), written by T1 (issue #5, 2026-10-31). The later parts land with the milestones that build their constructs (BUILD-ORDER § What T still writes): the parser's and the format's at M0 (`format(parse(x)) = x`, `format` idempotent), M1's and M2's (E §9.2–§9.6), and the worker sweep, 1 against 6 workers and reversed completion at M7 (D3-6).

**Restricted to S's forms**, household X's base: accounts and signed outside places, openings, transfers with stated amounts and days, a decision with declared reads, the written projection `bal(x, d) from t`, versions with their `known` dates, `as known` asks. No parts, fills, members, scenarios or random facts.

## How it runs

1. **The generator** `properties/gen.py` (standard library, seeded) writes `properties/kernel.corpus`: 24 small random realities per law, each in the variants its law compares. Every figure is a stated input (Tally explicit). `gen.py --check` (in `run.py --self-test`) fails when the file isn't what the seeds give.
2. **The proof on the oracle:** `properties/laws.py` runs every record on the frozen refcore through `run.py`'s own `run_one` (90 s budget, the watchdog) and checks each law on refcore's outputs. It writes them to `properties/kernel.refcore`. **Result, 2026-10-31: all 144 cases hold on refcore, 24 per law, and every control shows its law can fail.**
3. **The same laws in Rust:** `runner/src/laws.rs`. The runner's own tests (`every_law_holds_on_refcore`, `each_law_fails_on_a_planted_fault`) hold the Rust laws to refcore's outputs and to planted faults. They need no engine, so they run in `check full` now and stay green.
4. **On the engine:** `runner/milestone/s_properties.rs` runs every record through `tally_cli::run` in process (`INTERFACE.md` §4) and holds it to its law. It is red on the stub (7 of 7 fail, exit 69), so it is a test target with `test = false` until milestones are gated (`MILESTONE-S.md` § Keeping master green). Run it with `cargo nextest run -p tally-runner --test s_properties`.

No expected value is worked out by hand. Each law is a relation between runs, and refcore shows the relation holds on every generated reality. A control twin per case shows it isn't vacuous.

**No proptest:** proptest isn't on cargo-deny's allow-list, and adding it changes `engine/Cargo.lock`, which isn't T's. The generator is seeded and its corpus is checked in, so a run is the same everywhere; what proptest would add is shrinking, which a failure's small reality (one to six moves) mostly makes unneeded.

## The laws

| Test (`s_properties`) | Law | Source | Generator (`gen.py`) | Control |
|---|---|---|---|---|
| `s_determinism_files_and_sections_in_any_order` | the reality as one file and as the same sections spread over 2–3 files, in shuffled order, print the same bytes | L §9 (l.1784): "the order of files, and of sections in a file, changes nothing"; R-47; DONE gate 6 | `determinism`: 1–4 accounts, 1–2 outside places, 1–6 moves, half with a decision | — (equality) |
| `s_determinism_across_runs` | every record run twice prints the same bytes, and is answered (exit 0, as on refcore) | DONE gate 6 "across runs"; E §9 (l.432) | the whole corpus | an unanswered run fails |
| `s_money_conserved` | the flows of every place over the run, outside places included, sum to zero, and some aren't zero | E §9.1 | `conservation` | planted: one flow changed |
| `s_no_foresight_past_the_cut` | an ask `as known D` prints the same with and without a version known after D that replaces a move's amount | L §4.5 (l.699, the ask's cut K); DONE gate 6 "a cut never reads a version known after it" | `foresight-known` | the same ask at `on` sees the version and differs |
| `s_no_foresight_in_a_decision` | a movement into the decision's place dated after the decision's day changes nothing the decision moved | D-5; L §5.14.1 (l.1396), l.1439 "decisions are never re-measured"; DONE gate 6 | `foresight-decision`: a sweep `max(0, bal(x) - floor)` with `reads x` | the same movement dated before the decision changes it |
| `s_no_foresight_through_the_written_projection` | a decision reads `min(bal(x, d) from t for d in t .. reach)`; a movement out of x dated after the reach changes nothing it moved | L §5.14.1; D-6; DONE gate 6 "any row after the written projection's reach" | `foresight-projection` | the same movement the day after the decision, inside the reach, changes it |
| `s_a_hole_never_becomes_zero` | with one move's amount row left out, every value that read it prints a hole with the input's question, never a number; `x * 0`, `x - x` and `x + 0` stay holes | L §8.1: never 0, never a default; it stops propagating only where the outcome is decided without it | `hole` | with the row, every value is a number |

**What the kernel's list leaves for later, and why:**
- **1 against 6 workers:** the thread module is M7's (D3-6).
- **Changing a random fact's value, a decision's later result:** random facts are M7's. A decision's later result is covered here only through the movement it books.
- **A version known after a decision's day changing what it read** (l.1439): needs a back-pay hole in X's forms. `foresight-known` covers the cut, but not this case. The first cut, if the budget needs one, is more generators beyond X's base; this one is a candidate for M1's list.

## Files

| File | What |
|---|---|
| `properties/gen.py` | the generator, seeded (`SEED = 20261031`, 24 cases per law) |
| `properties/kernel.corpus` | its output: 312 records, netstrings (law, case, variant, args, file table) |
| `properties/laws.py` | the laws on refcore; writes `properties/kernel.refcore` |
| `properties/kernel.refcore` | refcore's output and exit code per record |
| `runner/src/{corpus,output,laws,engine}.rs` | the runner's library: reading, the laws, the in-process call |
| `runner/milestone/s_properties.rs` | S's tests on the engine |
| `properties/gen_m0.py`, `properties/m0.corpus` | M0's generator (`SEED = 20261104`) and its 285 records |
| `properties/laws_m0.py`, `properties/m0.refcore` | M0's laws on refcore, and its 570 runs (the probe and the reprint of each record) |
| `runner/src/laws_m0.rs`, `runner/milestone/m0_properties.rs` | M0's laws in Rust, and M0's tests on the engine |

## § M0: the parser's and the format's laws

Milestone M0's part (BUILD-ORDER § M0, "the parser's and the formatter's properties"; `MILESTONE-M0.md` row 7), written by T for issue #56, 2026-11-04. It runs as the kernel's part does: a seeded generator, the proof on refcore, the same laws in Rust held to refcore's runs by the runner's own tests, and a milestone target on the engine.

1. **The generator** `properties/gen_m0.py` (`SEED = 20261104`) writes `properties/m0.corpus` in `kernel.corpus`'s form. Its sources are real texts, not new ones: `checks.tsv`'s one-file sets (unit probes) that load clean on refcore (their `m0/load.refcore` probe prints the REQ-ASK line alone), at most 70 lines, no joint model (a one-file record lacks its members). Left out: any text with `schedule(`, `fulfils:` or `settles`, whose probe bytes hang on issue #72 (refcore makes those load checks only where an ask reaches the item): `U:law/LAW-04`, `U:law/LAW-10`, `U:rule/RULE-07`, `U:time/TIME-02`. No record asks a scenario's ask, so none hangs on #73. `gen_m0.py --check` fails when the file isn't what the seeds give.
2. **Each record runs three ways:** the load probe, `run <files> --ask zz_m0_probe` (`INTERFACE.md` § The load probe); `reprint <files>`; and, when that exits 0, `reprint` of the canonical sections it printed, each under the path it printed (`again`).
3. **The proof on the oracle:** `properties/laws_m0.py` runs every record on refcore 2.1.3 through `run.py`'s `run_one` and checks each law, writing the runs to `properties/m0.refcore` (four netstrings per run: case, `<variant>:probe|reprint|again`, exit code, output). `laws_m0.py --check` (`run.py --self-test` 8i) re-runs it from the cache and fails when a byte or a law differs. **Result, 2026-11-04: all 150 cases hold on refcore** (spelling 16, order 10, total 119, depth 5; each canonical text reprints as itself), from 285 records and 570 stored runs. Of total's 119 probes, 18 still load and 101 are refused: `LOAD-PARSE` 88, `-HEADER` 3, `-NAME` 3, `-DUP` 2, `-GOVERNS` 2, `-OVERLAP` 2, `-KNOWN` 1. Depth's limits fall where L §2.3 puts them: d = 8 loads, 9 is `LOAD-DEPTH expression nested deeper than 8`; `as if` 4 loads, 5 is `LOAD-ASIF as if deeper than 4`.
4. **The same laws in Rust:** `runner/src/laws_m0.rs`; its own tests (`every_m0_law_holds_on_refcore`, `each_m0_law_fails_on_a_planted_fault`, `a_mutant_agrees_by_its_code`) run in `check full` now.
5. **On the engine:** `runner/milestone/m0_properties.rs` (5 tests, `test = false` until M0 is tagged; all 5 red on S's engine, 2026-11-04: `not implemented`) runs every record in process, holds each case to its law and each run to refcore's: byte for byte, but a mutant's refusal, which agrees by its exit and code. Which check meets a broken text first is the spec's (E §3.1's passes, E §11); the words of a refusal on a random break are rows 1, 2 and 5's to pin (`m0_load`, `m0_codes`), not this law's.

| Test (`m0_properties`) | Law | Source | Generator (`gen_m0.py`) | Control |
|---|---|---|---|---|
| `m0_format_equal_models_print_identically` | a text and its rewrites that the grammar allows and L §2.4 makes equal reprint the same bytes and load clean | L §2.4 (l.329): "equal models print identically" (N-1902); L §2 (l.31) | `spelling`: 16 texts × the rewrites that change them: sections shuffled (header first), two blank lines between sections, a version's one-line items shuffled, a def's rules shuffled, `beats` lists reversed, a one-line def written long with `rule base` | one row's figure changed reprints otherwise |
| `m0_load_file_order_changes_nothing` | a text split over a model file and a `part`, in either order, loads as the whole text, and both orders reprint the same | L §9: "the order of files … changes nothing"; E §3.1 step 1 (files in path order) | `order`: 10 texts of 4 sections or more, split at a random section | a figure changed in the part reprints otherwise |
| `m0_parser_total` | a text broken by one operator gives exit 1 and one `LOAD-*` refusal line (after any warnings; the probe's REQ-ASK line if it still loads), or a reprint's sections and `sha256` line; a refused reprint (the parse, step 1) is the probe's refusal | L §8 "a load refusal stops the model before anything runs"; E §3.1; E §11 | `total`: 10 texts × 12 operators: a line deleted, duplicated, swapped with the next; a token deleted, duplicated, made the keyword `rule`; a tab; an indent off by one; the text cut short; a byte that isn't UTF-8; a bracket left open; CRLF line ends | planted: a panic's exit 101 |
| `m0_depth_stops_at_8` | an expression nested d deep loads for d ≤ 8 and is refused `LOAD-DEPTH` past it; `as if` nested d deep, `LOAD-ASIF` past 4; a chain of operators at one level never adds depth | L §2.3 (l.325) | `depth`: an invented model of three inputs; parentheses, `max(…, b)` calls and `if c then … else b`, d = 1–12; `(… as if {a = b})`, d = 1–6; chains of 2–60 terms | the variants past the limit |
| `m0_format_idempotent` | every canonical text reprints as itself | L §2 (l.31) "one canonical form"; BUILD-ORDER § M0 "`format` idempotent" | every record whose reprint exits 0; one refcore reprints, the engine must too (no vacuous pass) | planted: a changed byte |

**Not here, and why:** `format(parse(x)) = x` on every text set is row 4's (`m0_format`, the round trip of 337 texts); this part adds the texts that aren't canonical. The fuzz targets `parse`, `format` and `load` are row 8's.
