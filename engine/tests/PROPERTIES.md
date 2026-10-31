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
