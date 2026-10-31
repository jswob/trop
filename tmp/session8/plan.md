# Part B's plan (session 8b), from session 8's gate G1

**Made:** 2026-10-30 by the session 8 lead, for the master, who writes 8b's brief from it. **Rests on:** `decisions.md` (G1: all nine items approved as recommended, the user's words recorded there), `tmp/research_notes/engine-prep/NOTE.md` (K8-1..K8-17 settlements, SO8-1..SO8-10, the unknowns), R1–R6, and `shape.md`.

Part B writes documents and tools, not engine code: no engine exists until S. It measures nothing of the engine. Its one engine is the scaffold's stub, shown red on every check.

## 0. Order and dependencies

```
B2.0 audit phase 1 as built ─┐
B1 documents ────────────────┼─> B2.1 merge engine into the scaffold ─> B2.2 rules ─> B2.3 harness ─> B2.4 CLAUDE.md + hook
                             │                                                                         │
                             └─> B3.1 the skill (needs B1.4) ──────────────────────────────────────────┤
                                                                                  B4 test wiring ───────┴─> B3.3 the eval ─> G2
```

- The audit (B2.0) judges what phase 1 delivered, so it runs on the scaffold **before** the merge.
- The skill (B3.1) and `engine/CLAUDE.md` (B2.4) quote the updated RULEBOOK and BUILDER (B1.4).
- The harness (B2.3) reads PERFORMANCE §8's rows (B1.1).
- The eval (B3.3) needs the merged scaffold with a working `check fast` and its fixture crate.

**Two sessions are suggested,** for checkpoints rather than savings:
- **8b-docs:** B1 and B3.1–B3.2. Paper and text, no toolchain.
- **8b-build:** B2, B4, B3.3 and G2. Needs the toolchain and the install go.

## 1. The items, with estimates and inputs

Estimates are % of the week, calibrated on this session: an Opus research agent of 70–116 calls cost about 0.2 %, and the lead's own work about 0.1 % per step.

### B1. The documents (in `engine/spec/` and `tmp/research_notes/engine-rust-strictness/`; landed as G1 item 9 says)

| # | Work | Inputs | Estimate |
|---|---|---|---|
| B1.1 | **`PERFORMANCE.md` for 2.1.** §1 strategy (D1-1..D1-7); §2 the event-driven walk, checkpoints and forks, with SO8-1's list of what makes a day non-empty derived and a step-every-day oracle named; §3 contexts as layers, trimming, projections (SO8-2); §4 read sets now, incrementality a seam; §5 paths: the parallel map, the ordered merge (D3-3), Maritz–Jarrett by intervals (D3-7); §6 numbers (D2-1, D2-2); §7 cost per construct, including 2.1's parts and labels, observations, joint members and the written projections; §8 the four groups of budget rows (D4-6, D4-7), "proposed" until S, with refcore's figures as the floor column; §9 caps for 2.1 (path statistics uncapped, 2.1-15; joint nesting with no cap, SO8-8); §10 what stays open (NOTE §4) | NOTE.md; R1–R4; shape.md; `engine/tests/load/load.tsv`, `benchmarks.md`; LANGUAGE and ENGINE 2.1; STALE.md's PERFORMANCE rows | 0.3–0.5 |
| B1.2 | **`BOUNDS.md`** where the budgets need it: the out-of-reach list and the caps, for 2.1 | STALE.md's BOUNDS rows; CHANGES.md; B1.1 | 0.1 |
| B1.3 | **`BUILD-ORDER.md` for 2.1.** S and M0–M9 restated with 2.1's constructs. Parts and labels and joint contexts get their milestone (seam J's first step at D1-7). Each milestone names its checks from `engine/tests/checks.tsv` (ids or globs) and the §8 budget rows it touches at 1×. The tracer slice keeps the household | `checks.tsv`, `map/tags.tsv`, `cases/boundaries.md`, `coverage.md`, CHANGES.md, STALE.md's BUILD-ORDER rows, B1.1 | 0.3–0.5 |
| B1.4 | **The rulebook and BUILDER.** RULEBOOK gets new rows R-114..R-118; amendments to R-16, R-37, R-40, R-42 (five points), R-47, R-58/R-76 (recount with `tools/xref_check.py`), R-73, R-74 (nesting at 5), R-75 and R-97; and a new group for readability (RD-1..RD-4, RS-1..RS-7, D5-3, D5-4, D5-6, D5-7), each with the user's G1 answer. BUILDER is updated in §2 (Lehmer now, money `I64 \| Big`, the cap per 2.1-15), §3 (the interner, `Table<I, T>`), §4 (counted recursion, K8-1), §7 (threads per D3-1, collections per D2-5), §9 (the milestone budget step), §11 (`engine/CLAUDE.md` with the SessionStart hook), §12 (the readability checklist), and gets a new section on performance (fences, counters, profiling recipes) | decisions.md; NOTE §2; R2, R3, R5, R6 §1.4 | 0.2–0.4 |
| B1.5 | **Landing:** `SHA256SUMS`, `STALE.md` (the three no longer stale), and one DONE.md approvals row that also restates gate 12's evidence and its benchmarks path (SO8-9). `VERSION` stays `tally-2.1`. The other SO8 rows stay in NOTE §3, for the next spec patch's gap log | G1 item 9 | 0.05 |

**B1 total: 0.95–1.55.**

### B2. The scaffold (worktree `/Users/jswob/projects/trop-engine-scaffold`, branch `engine-scaffold`)

| # | Work | Inputs | Estimate |
|---|---|---|---|
| B2.0 | **Phase 1's audit** (`tmp/active/engine-phase1-scaffold.md` §9), on the scaffold as built: re-run every check, probe the guard with fresh cases, plant new violations, confirm `engine/spec/` untouched. Then its open items: the CLI's exception budget (3 of 3 used); a rule change and its checksum in one commit; the interpreter tripwire; gate 8's survivor wording (SO-28). Add R6's finding that the guard misses the skill and `engine/CLAUDE.md` (C-3), and R4's that `check.py` doesn't clear `GUNGRAUN_*` and `CRITERION_*` | `engine/SCAFFOLD.md`, `SCAFFOLD-REPORT.md`; phase 1's NOTE and RULEBOOK | 0.3–0.5 |
| B2.1 | **Merge `engine` into `engine-scaffold` locally**, with conflicts logged. Merge base 6f5e676 (tally-1.0.1), 53 commits on `engine`, 168 files on the scaffold. Only `.prettierignore` overlaps by text. The meaning-level conflicts: check-frozen's `VERSION` (tally-1.0.1 → tally-2.1) and its `SHA256SUMS`; the guard's allow-lists against the Python suite now in `engine/tests/` and refcore in `engine/tools/`; T's runner stub at `engine/tests/runner/`; the CI workflow's paths; root `package.json` scripts | `git diff 6f5e676..engine` | 0.15–0.3 |
| B2.2 | **The new rules, each with its planted violation in `rules-selftest`.** clippy.toml: `excessive-nesting-threshold = 5`; bans on `std::thread::spawn`, `Scope::spawn`, channels, locks and atomics outside the thread module (R-42). clippy: `impl_trait_in_params`. ast-grep: `dyn` (error), `macro_rules!` outside the lead's list (R-73), derived orders (R-118, report), own generics and one-implementation traits (reports). Scripts: the `//!` header check (RD-2) and the `trace` script (D5-4, R-75 in the kernel). The guard's rule globs and `RULES.sha256` gain the skill and `engine/CLAUDE.md` (D6-3). Every change goes through R-112's procedure | B1.4; the scaffold's `tools/rules-selftest` | 0.4–0.7 |
| B2.3 | **The benchmark harness and profiling.** A lead-written `engine/tools/perf`: wall time, peak footprint and instructions retired by `/usr/bin/time -l`, 2 warm-ups then the median of 10, and a machine fingerprint. A samply summariser to text. gungraun bench skeletons and a Linux CI job for count and instruction limits. A protected budget table read from §8's rows. The `counters` feature and the `--stats` contract on the stub. CI: wall-time rows run on the reference Mac at milestones (R-97 amended), never on hosted runners | B1.1 §8; R4 D4-1..D4-8 | 0.3–0.5 |
| B2.4 | **`engine/CLAUDE.md`** (≤ 150 lines, BUILDER §11 updated) and the build-role **SessionStart hook** (startup and compact) that prints it, through the rules role | B1.4; R6 D6-2 | 0.1 |

**B2 total: 1.25–2.1.**

### B3. The skill (G1 item 7: create)

| # | Work | Inputs | Estimate |
|---|---|---|---|
| B3.1 | **`.claude/skills/engine-rust/`**, written with `skill-creator`. `SKILL.md` ≤ 5 000 tokens: the hard rules with their enforcement, the "write this instead" table, the loop, the STOP-REPORT. `references/`: numbers, data layout, the evaluator and the memo, parallelism, performance (fences, counters, profiling recipes), readability (the subset, headers, trace), the seams. Borrowed ideas only (R6 D6-1; Trail of Bits is CC-BY-SA, so no text). Paper checks: every cited R-n exists; every BUILDER §8 row appears; `quick_validate.py` | B1.4; R6; decisions.md | 0.3–0.4 |
| B3.2 | **`engine-reviewer`**, a read-only subagent that loads the skill, with BUILDER §12's checklist plus readability (D5-5) | B1.4 | 0.05 |
| B3.3 | **The eval:** a fixture crate with E1–E9 (R6 §1.3), graded by scripts, with and without the skill. The user's approved bar: no banned form in any run, and STOP-REPORTs in at least 5 of 6 runs of E8 and E9. **Staged:** 1 run per arm first (18 runs), then the remaining 36 only for the cases where an arm fails or the arms differ, unless the user wants all 54 at once | B2.1–B2.4 merged and green; B3.1 | 18 runs 0.5–1.0; all 54 1.5–3.0 |

**B3 total: 0.85–1.45 staged; 1.85–3.45 with all 54 runs.**

### B4. The test wiring (`engine/tests/`, the guard)

| # | Work | Estimate |
|---|---|---|
| B4.1 | **The engine's command-line contract, matching refcore's,** so baselines compare byte for byte. `tally run <files> [--ask id …] [--pin=bytes]` and `tally reprint <files>` print what refcore prints, with exit codes and the error form. Written as `engine/tests/INTERFACE.md` (T's deliverable 1, restated for 2.1) | 0.1 |
| B4.2 | **`run.py --interp engine`:** the release binary with the same argv; the cache keyed by the binary's hash; an engine output never becomes a baseline (`ran` is refused for the engine) | 0.1 |
| B4.3 | **The build role's guard over the suite:** `engine/tests/**`, `engine/tools/refcore*`, `engine/spec/**` and the keys refused to the build role, and a `TESTS.sha256` manifest checked by check-frozen | 0.05–0.1 |
| B4.4 | **The stub shown red:** `run.py --interp engine` on the stub gives every check `differ` or `error` (not implemented), none skipped silently; the load rows listed as not-run | 0.05 |

**B4 total: 0.3–0.35.**

### G2 (the end of Part B)

The user reviews:
- the documents (PERFORMANCE, BOUNDS, BUILD-ORDER, the RULEBOOK rows, BUILDER, the DONE row);
- the scaffold's diff, the skill and its eval result.

Then the user gives the install go: samply (`cargo install --locked`), the `rust-analyzer-lsp` plugin, gungraun's runner in CI only. Last, the user pushes and merges PR #1 (`engine-scaffold` → `engine`). Estimate 0.1–0.2.

### Total

| | Low | High |
|---|---|---|
| B1 + B2 + B3 (staged eval) + B4 + G2 | 3.45 | 5.65 |
| with all 54 eval runs at once | 4.45 | 7.65 |

**Against the master's figure.** The whole prep was estimated at about 4–6 %. Part A (this session) ends near 1.7 %, so with the staged eval the prep comes to about 5.2–7.4 %. That is above the master's range, by the eval and the scaffold's audit, neither of which the master's figure separated. The cuts, in order:
1. the eval staged (already assumed);
2. the phase 1 audit by script where the scaffold has a check, with one agent only for the fresh guard probes;
3. `BOUNDS.md` to its caps rows only;
4. the samply summariser deferred to S.

## 2. Where T fits

T's brief (`tmp/active/engine-tests.md`) was written for tally 1.0.1. Today's suite in `engine/tests/` covers part of it, against refcore's baselines, byte for byte:

| DONE gate | Covered by the suite now | Still T's |
|---|---|---|
| 1, the 79 cases | 79 `I:M:` checks (every model) | none, beyond the engine interface |
| 2, goldens and pins | no check | G1–G10 and the 36 pins restated in tally 2, with refcore baselines |
| 3, the household and its 7 changes | 71 `I:X:` checks (the long-horizon cells are load rows L:10, L:13) | none |
| 4, hindcast H0–H9 | no check | the fixtures restated in tally 2 |
| 5, the codes | unit probes by capability edge (154), not one fixture per code | a scan of which codes the baselines print against the catalogue (about 98 names, K8-13); fixtures for the rest |
| 6, properties | none | `PROPERTIES.md` with generators, including D3-6's worker sweep and reversed completion |
| 7, Monte Carlo | 8 checks on cut-downs; full size at L:11, L:12 | none beyond the load rows |
| 10, differential | refcore is the oracle | the generator of 10 000 random realities |

The runner is `run.py --interp engine` (B4.2) for byte comparison, plus the scaffold's Rust runner crate (`engine/tests/runner/`) for property tests that call `tally_cli::run` in process. **T becomes a smaller session after Part B:** gates 2, 4, 5's missing fixtures, 6 and 10's generator, plus `TESTS.sha256`. The master rewrites T's brief for 2.1. A rough size is 2–4 % (inference; the generator and the properties are the bulk).

## 3. What the master carries into 8b's brief

- **The G1 answers:** all nine items approved as recommended (`decisions.md`).
- **The settlements K8-1..K8-17,** and the spec observations SO8-1..SO8-10 (NOTE §2–§3). SO8-1 (the list of what makes a day non-empty) and SO8-2 (trimming a projection's context) are design work for B1.1, not spec edits.
- **The unknowns** (NOTE §4) are measured at S, not in Part B. B1.1 names each as an open row.
- **The toolchain stays at Rust 1.99.1** through Part B. 1.100.0 (due 2026-11-12) stabilises `[lints.cargo]`; a bump is a rules-role change with re-baselined counts.
- **Seal:** Part B never opens `tmp/holdout/`. `engine/tests/cases/` holds files named `*-hidden.md`. Session 8 didn't open them; 8b's brief should say whether the build role may.
- **Rules every agent prompt carries:** no sub-agents, closed scopes, the seal rule word for word, saved finals and transcript audits, as in this session.

## Addendum (2026-10-30, after G1): the briefs

The user asked session 8's lead to write the briefs: `tmp/active/session8b-brief.md` (8b-docs: B1, B3.1–B3.2, gate G2a, landing) and `tmp/active/session8c-brief.md` (8b-build, named 8c: B2, B3.3, G2). One change from §1: B4's suite-side work (`engine/tests/INTERFACE.md`, `run.py --interp engine`, `TESTS.sha256`, the stub shown red) moves to T. The scaffold's guard gives `engine/tests/` to the `tests` role only (SCAFFOLD §3), and 8c runs as `rules`. 8c drafts the contract (`tmp/session8c/INTERFACE-draft.md`) and does the guard's side. 8b also drafts `engine/CLAUDE.md`'s text and the eval's design, so 8c only places them. The eval's runs are headless build-role sessions (`TALLY_ROLE=build claude -p`), not agents, so the guard binds them as it will bind the builder.
