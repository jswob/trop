<!-- saved by tools/save_final.py from agent agent-a5082519dc16ec962.jsonl (R5) -->

I've written `/Users/jswob/projects/trop/tmp/research_notes/engine-prep/R5.md` (about 34.9 KB, over the ~30 KB target). My scratch notes are under `tmp/research_notes/engine-prep/scratch/R5/`.

**The three findings that matter most:**
1. **The owner should audit through the map, the spec trace and refcore, not through the Rust.**
   - rust-analyzer fails its build when a file doesn't start with a `//!` header comment (`xtask/src/tidy.rs`, `TidyDocs`), and it keeps a short ARCHITECTURE.md with invariants.
   - Cedar's readable Lean model is "an order of magnitude smaller" than its Rust and is held to it by differential tests. refcore plays that role for Tally, so each engine module should name its refcore twin and its spec sections.
2. **The strict rules aren't where speed goes, so the plain subset costs nothing.**
   - Readyset removed every bounds check from a production Rust engine and called the gain "negligible".
   - `dyn` (runtime interface dispatch) blocks inlining (Rust book); iterators and loops benchmark the same (19.23 vs 19.62 ms).
   - The order of magnitude sits in a few fenced modules: walk skipping and checkpoints, the memo, the read index, the number tiers. Each fence gets a benchmark, a plain twin and a test that they agree.
3. **K-21 should change in part.**
   - clippy's `cognitive_complexity` scores a whole `match` as +1, whatever its arm count (clippy source). Long dispatch matches aren't the problem; nesting is.
   - So `excessive_nesting` becomes an error at 5, as A1 proposed. Length stays a report at 100, because the studies conflict.

**Decisions and picks:**
- **D5-1:** an ARCHITECTURE.md of about 200 lines, plus a script-checked `//!` header in every file (purpose, Spec, Twin, Invariants), with the spec's words as names.
- **D5-2:** a subset. No `dyn` (ast-grep error), no own generics or new single-implementation traits (reported), no `impl Trait` arguments (clippy), no library macros (R-73 rises to error), short iterator chains.
- **D5-3:** nesting error at 5; length (100) and cognitive complexity (25) stay reports; each dispatch arm is one call or expression; files over 1,000 lines reported.
- **D5-4:** a ~150-line `trace` script. Unknown spec sections or lines fail; kernel functions must cite (R-75 becomes an error there); it prints the reverse map from section to code, refcore and tests.
- **D5-5:** a generated one-page milestone report, plus a readability section added to BUILDER §12's checklist.
- **D5-6:** performance code only in fenced modules, each with a `PERF:` header, a benchmark, a plain twin and an equivalence test.
- **D5-7:** newtypes with checked methods first, then named helpers in one kernel module, then iteration instead of indexing. No arithmetic macros or extension traits.

**What you need to know:**
- **Advisory thresholds disagree.** The scaffold's `tools/advisory.py` reports at 60 lines, cognitive 20 and nesting 4; A5 SC-6 said 100 and 6.
- **Two size lints are already errors.** `too_many_arguments` (7) and `type_complexity` (250) are on through the scaffold's denied `complexity` group, and the rulebook doesn't say so. `excessive_nesting` is in the same group, so setting its threshold in clippy.toml makes it an error at once.
- **The approved exception budget is 0 for the library and the parser.** The nesting threshold has to be one the code can meet without exceptions; I've marked it to re-judge at S.
- **ast-grep rules can't take an `#[expect]`** (R-109), so a real need under D5-2 has to go through a STOP-REPORT or a file the lead sanctions.
- **Rules changes:** RS-6 and D5-4 would turn R-73 and R-75 from reports into errors, which needs R-112's procedure.
- **Microsoft's Rust guidelines conflict with the rulebook in two places:** "bugs are panics" against R-3/R-4, and "use `From`, not `map_err`" against R-66. I noted both; phase 1's choices stand.
- **Versions:** Rust stable is 1.99.1 (1.100 is due 2026-11-12), and ast-grep 0.51.0 is still the latest. Both match the scaffold.
- **Sources I couldn't fully reach:**
  - Ferrocene's traceability-matrix page went over the fetch limit, so I used secondary summaries.
  - GitClear's duplication figures come via a secondary report.
  - Ruff's fixed rule-doc sections come from a search result.
