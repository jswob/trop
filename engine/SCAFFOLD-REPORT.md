# Phase 1: the scaffold report

For the clean-side lead's audit (phase-1 brief §9). The phase-1 lead built the scaffold on 2026-10-22 and 2026-10-23 in the worktree `trop-engine-scaffold`, on branch `engine-scaffold`, off `engine`. On 2026-10-23, at the user's request, `engine` (with the tags `tally-1.0` and `tally-1.0.1`) and `engine-scaffold` were pushed, and a pull request from `engine-scaffold` into `engine` was opened. `engine/SCAFFOLD.md` describes what is set up; this report gives the evidence.

## 1. What was built

- **The Cargo workspace:** `tally` (the library; a stub `parse` answering "not implemented"), `tally-cli` (`run(args, stdin) -> Output`, and the binary `tally`, the one file that does I/O), and `tally-runner` (T's test package, a stub test). The workspace pins the number stack and sha2, holds the lints, and holds the profiles. No engine logic.
- **The lints and bans:** the workspace lints (rustc, clippy with pedantic and cargo, rustdoc); `clippy.toml`'s three ban lists, 455 entries; 15 ast-grep rules with their tests; two cargo-deny configs (the workspace, and what ships).
- **The instruments:** a fuzz workspace with the `parse` target and its regressions folder, a criterion stub, the cargo-mutants and nextest configs.
- **The guard and its hooks:** PreToolUse, ConfigChange and Stop, wired in `.claude/settings.json`, with `guard-test`.
- **The checks:** `check fast|full|step`, `check-frozen`, `check-rules`, the exception counter with its lock and budgets, `rules-selftest` (219 cases), `mutants-survivors`, and two reports (`advisory`, `special-casing`).
- **CI:** `.github/workflows/engine.yml`. Every PR into `master` or `engine` runs `check full`. Nightly jobs run mutants in four shards, then the survivors check, plus coverage, benchmarks, the reports, macOS and a 32-bit target. Advisories run daily, clippy on beta weekly, and the long fuzz runs by hand on the self-hosted Mac.
- **Root scripts:** `engine:check`, `engine:test`, `engine:spec-check`, `engine:fuzz`, `engine:mutants`, `engine:cov`, `engine:bench`. `engine` is added to `toolkit.agnostic.paths`.
- **The docs:** `engine/SCAFFOLD.md`, this report, `engine/BUILD-LOG.md` with its STOP-REPORT template, and `engine/MUTANTS-SURVIVORS.md`.

## 2. The checks, with their output

All commands were run in the worktree with Rust 1.99.1, nightly-2026-10-21, cargo-nextest 0.9.149, cargo-deny 0.20.2, ast-grep 0.51.0, cargo-fuzz 0.13.2, cargo-mutants 27.1.0, cargo-llvm-cov 0.9.1 and actionlint 1.7.12. Commands are quoted as run, each with its exit code.

### 2.1 `engine/tools/check full` (C §7: build, test, both clippy passes, doc, fmt, both cargo-deny runs, ast-grep, the fuzz run, the selftest, the guard's tests)

```
$ engine/tools/check full
check full: green, 24 steps (160s)
[exit 0]
```

Each step's log is kept in `engine/target/check-logs/<step>.log`. The key line of each:

```
fmt                     $ cargo fmt --all --check                                        [exit 0]
clippy-lib              $ cargo clippy --workspace --all-features --locked --lib --bins -- -D warnings   [exit 0]
clippy-tests            $ cargo clippy --workspace --all-features --locked --tests --benches -- -D warnings
                          -A clippy::disallowed_macros -A clippy::use_debug             [exit 0]
runner                  1 test run: 1 passed (tally-runner tests::the_stub_answers_not_implemented)
exceptions              $ tools/exceptions --check   (5 exceptions, equal to EXCEPTIONS.lock)   [exit 0]
check-frozen            check-frozen: spec, tag and rules unchanged                      [exit 0]
check-rules             check-rules: the builder-edited files keep the rules             [exit 0]
doc                     $ cargo doc --workspace --no-deps --all-features --locked        [exit 0]
build                   $ cargo build --workspace --all-targets --locked                 [exit 0]
test-dev                3 tests run: 3 passed, 0 skipped                                 [exit 0]
test-release            3 tests run: 3 passed, 0 skipped (profile release-test)          [exit 0]
doctest                 test result: ok. 0 passed (three crates)                         [exit 0]
release-binary          $ cargo build --release --locked -p tally-cli                    [exit 0]
fault-injection-probe   error: the fault-injection feature exists only in builds with debug assertions (R-51)
                          [exit 101, which the probe requires]; cargo tree -p tally-cli -e features: no fault-injection
deny                    $ cargo deny --locked check bans licenses sources  ->  bans ok, licenses ok, sources ok
deny-tally              $ cargo deny --locked --manifest-path crates/tally/Cargo.toml --config deny-tally.toml check bans
                          ->  bans ok (warning[unused-wrapper] for hashbrown and indexmap: harmless)
tree-probe              $ cargo tree --locked -p tally -e normal,build: no rand, no getrandom   [exit 0]
ast-grep-test           test result: ok. 15 passed; 0 failed;                            [exit 0]
ast-grep-scan           $ ast-grep scan                                                  [exit 0]
guard-test              guard-test: 725 checks, 0 failed                                 [exit 0]
rules-selftest          rules-selftest: green, 219 cases over 77 rules (37s)             [exit 0]
spec-loadcheck-selftest $ python3 -I -B spec/tools/loadcheck.py --selftest               [exit 0]
spec-loadcheck          $ python3 -I -B spec/tools/loadcheck.py   (total: 0 refusals)     [exit 0]
fuzz                    Done 103027307 runs in 61 second(s)                              [exit 0]
```

After that run, one doc warning was fixed: "output filename collision", because the library and the binary are both `tally`. The binary now has `doc = false`, and `check step doc` runs without the warning. Section 2.12 is the final run.

### 2.2 `rules-selftest` (brief §7.3)

```
$ engine/tools/rules-selftest/run
rules-selftest: the clean run passes every tool, nothing is silently off
rules-selftest: ban coverage 455/455 entries fire; fakes for chrono
rules-selftest: green, 219 cases over 77 rules (38s)
[exit 0]
```

Every tool-enforced rule has at least one planted violation that must fail with the rule's own name. The ban coverage plants one use of each of clippy.toml's 455 entries. `rules-selftest/run --list` maps all 113 rules. None prints "NOTHING": every rule has a case, the ban coverage, another named proof, "review only", or "dropped at gate 1". The first full run had 5 of 214 cases failing, and two of those were real tool gaps:

- **R-45:** clippy's `use_debug` doesn't see `format!`.
- **R-71:** clippy's `missing_assert_message` skips tests.

Both are now closed by checks of their own (SCAFFOLD.md §8, items 5 to 7). A third gap, **R-59**, needed a check-rules rule. The other two failures were mistakes in my own cases.

### 2.3 The exception counter (brief §7.4; C §7 wants 0)

```
$ engine/tools/exceptions
exceptions: 5 (tests 0, fuzz-benches 0, kernel 0/20, lexer-parser 0/0, thread 0/1, cli 3/3, library 0/0, deny 2); report-only hits: 0
  cli  crates/tally-cli/src/main.rs:28  fn arguments  expect clippy::disallowed_methods  "R-12: the arguments are read once, here, through args_os, which can't crash"
  cli  crates/tally-cli/src/main.rs:37  fn read_input  expect clippy::disallowed_methods  "R-39: the CLI reads the reality from standard input in this one sanctioned file"
  cli  crates/tally-cli/src/main.rs:51  fn write_output  expect clippy::disallowed_methods  "R-39: the CLI writes its output bytes in this one sanctioned file"
  deny  deny.toml:94  syn@2.0.119  bans.skip  "R-55: dev only; zerocopy-derive (criterion via half) is still on syn 2 while serde_derive is on syn 3"
  deny  deny.toml:156  zerocopy  bans.build.bypass  "R-56: zerocopy (criterion only, dev) ships its maintainers' CI shell scripts; nothing runs them"
[exit 0]
```

It prints 5, not 0. The stubs can't do their I/O or build their dev tools without these five. Each one is reasoned, locked and within its area's budget (SCAFFOLD.md §8, item 1).

### 2.4 The guard (C §7: every role, every protected path, each Bash form, ConfigChange, Stop, a forced failure)

```
$ engine/tools/guard-test
guard-test: 725 checks, 0 failed
[exit 0]
```

It runs the guard and the Stop hook in a throwaway git repo, against hook inputs for every role (`build`, `rules`, `tests`, `spec`, unset, and an unknown role), every protected path, and each Bash form. It also covers the ConfigChange hook in every role, the Stop hook's green, red, STOP-REPORT and three-blocks paths, and a forced internal failure (`TALLY_GUARD_SELFTEST_CRASH=1`, which must refuse). The wiring in `.claude/settings.json` was also simulated by hand:

| Hook input                                        | Exit                      |
| ------------------------------------------------- | ------------------------- |
| build role, Edit `engine/clippy.toml`             | 2 (refused)               |
| build role, Edit `engine/crates/tally/src/lib.rs` | 0                         |
| `engine/` present, guard missing                  | 2 (refused: fails closed) |
| no `engine/`                                      | 0                         |
| ConfigChange in the build role                    | 2 (refused, logged)       |
| Stop with no role                                 | 0                         |

### 2.5 check-frozen: passes on `tally-1.0.1`, fails on one changed byte

This ran in a scratch clone of the branch with the working files copied in, so the real spec was never touched:

```
$ tools/check-frozen   # the scratch clone, unchanged
check-frozen: notice: engine/tests/TESTS.sha256 isn't written yet (T writes it); skipped
check-frozen: spec, tag and rules unchanged
exit 0
flipped byte 200 of engine/spec/ENGINE.md: b'd' -> b'e'
$ tools/check-frozen   # one byte changed
check-frozen: notice: engine/tests/TESTS.sha256 isn't written yet (T writes it); skipped
FAIL spec: ENGINE.md changed (checksum differs)
FAIL spec: differs from tally-1.0.1: M	engine/spec/ENGINE.md
check-frozen: 2 problems
exit 1
flipped byte 300 of engine/clippy.toml: b'#' -> b'"'
$ tools/check-frozen   # one rule-file byte changed
check-frozen: notice: engine/tests/TESTS.sha256 isn't written yet (T writes it); skipped
FAIL rules: engine/clippy.toml changed (checksum differs); a rule changes only through R-112
check-frozen: 1 problems
exit 1
```

The selftest also proves it, in its R-80 cases: `spec-byte`, `spec-new-file`, `rule-file-byte`, `ast-grep-rule-byte` and `new-rule-file`. (The message has since been changed to "1 problem".)

### 2.6 check-rules: passes, and fails on a planted `[lints]` edit

This also ran in the scratch clone:

```
$ tools/check-rules   # unchanged
check-rules: notice: the kernel files aren't set yet (the lead sets them at S, R-85); skipped
check-rules: the builder-edited files keep the rules
exit 0
$ git diff   # the planted [lints] edits
-unsafe_code = "forbid"
+unsafe_code = "warn"
-[lints]
-workspace = true
+[lints.clippy]
+unwrap_used = "allow"
$ tools/check-rules   # planted
check-rules: notice: the kernel files aren't set yet (the lead sets them at S, R-85); skipped
FAIL engine/Cargo.toml [workspace.lints] differs from the frozen copy (R-80): rust.unsafe_code is 'warn', frozen as 'forbid'
FAIL engine/crates/tally/Cargo.toml: [lints] must be exactly `workspace = true` (R-80)
check-rules: 2 problems
exit 1
```

### 2.7 actionlint

```
$ actionlint .github/workflows/engine.yml
[exit 0]
```

### 2.8 Fuzzing, mutants and the criterion stub (C §7)

```
$ bun engine:fuzz -max_total_time=20
Running `fuzz/target/aarch64-apple-darwin/release/parse -artifact_prefix=…/engine/fuzz/artifacts/parse/ -max_total_time=20 fuzz/corpus/parse fuzz/regressions/parse`
Done 35735492 runs in 21 second(s)
[exit 0]
```

The 60-second run is `check full`'s `fuzz` step, above: 103,027,307 runs in 61 s.

```
$ cd engine && cargo mutants --list
crates/tally/src/lib.rs:18:5: replace parse -> Result<(), NotImplemented> with Ok(())
[exit 0]

$ bun engine:mutants
     Summary [   0.007s] 1 test run: 1 passed, 0 skipped
Found 1 mutant to test
MISSED   crates/tally/src/lib.rs:18:5: replace parse -> Result<(), NotImplemented> with Ok(()) in 2s build + 0s test
1 mutant tested in 3s: 1 missed
mutants-survivors: score 0.0% = caught 0 / (caught 0 + missed 1 + timed out 0); unviable 0, not counted
mutants-survivors: crates/tally/src/lib.rs: replace parse -> Result<(), NotImplemented> with Ok(()) (missed) is not in MUTANTS-SURVIVORS.md (R-87)
mutants-survivors: the score 0.0% is under DONE gate 8's 90 %
mutants-survivors: notice: the kernel isn't set yet (the lead sets it at S, R-85); reported, not failed
[exit 0]
```

The stub's one mutant is missed, as it must be, because nothing tests `parse` yet. `mutants-survivors` reports it, and will fail on it once the kernel is set at S.

```
$ cd engine && cargo bench --locked -p tally --bench stub
parse_stub              time:   [288.91 ps 289.25 ps 289.62 ps]
[exit 0]
```

### 2.9 Root scripts

```
$ bun engine:check        check full: green, 24 steps                    [exit 0]
$ bun engine:test         3 tests run: 3 passed; doctests ok              [exit 0]
$ bun engine:spec-check   load check: total: 0 refusals; check-frozen    [exit 0 once RULES.sha256 is current]
$ bun engine:bench        (2.8)                                           [exit 0]
$ bun engine:cov          3 tests run: 3 passed; report in engine/target/llvm-cov/html   [exit 0]
$ bun engine:fuzz         (2.8)                                           [exit 0]
$ bun engine:mutants      (2.8)                                           [exit 0]
```

Section 2.12 is the final run.

### 2.10 `bun lint` (C §7): passes, in a built checkout too, since the ignore lines of section 2.13

Before the user's go, it failed in a built checkout:

```
$ bun lint
…/engine/target/rules-selftest/target/doc/trait.impl/core/marker/trait.Copy.js
  0:0  error  Parsing error: … was not found by the project service. …
✖ 37 problems (37 errors, 0 warnings)
[exit 1]
```

All 37 errors were in JavaScript that rustdoc wrote under `engine/target/`, which the root `eslint.config.js` didn't ignore (SCAFFOLD.md §8, item 2). Section 2.13 has the run after the fix.

**`toolkit.agnostic` with `engine`** (C §6: "check that `bun toolkit check` handles a path that isn't a package"): it does. The word check lists files with `git ls-files -- <paths>`, which works on any folder; built output is gitignored, so it isn't listed. A throwaway repo with `paths: ["engine"]` and `words: ["tropek"]` reported the planted word:

```
$ bun t.ts <tmp repo>
[{"file":"engine/tools/x.rs","line":2,"word":"tropek"}]
```

In this repo `toolkit.agnostic.words` is empty, so the check finds nothing anywhere until the words are filled in.

### 2.11 Nothing in the scaffold depends on Tally's syntax (C §7, brief §8)

Checked over the scaffold's 74 code and config files, everything outside `engine/spec/`:

- **No parsing.** No file parses Tally text, and no file holds a Tally construct (a section kind, an ask, a model). `parse` is a stub over bytes. The fuzz target feeds it raw bytes. The runner's one test asserts "not implemented".
- **Two places touch the language at all, and neither reads syntax.**
  - `guard_test.py` uses a path name, `engine/spec/models/new.tally`, as a protected path.
  - `special-casing` reads T's JSON vectors: their file names, an `ask` key and the numbers. That depends on T's vector format, not on Tally's grammar.
- **The CLI's command names** (`load`, `ask`, `run`, `project`, `format`) come from `engine-tests.md` §4, the test interface, not from the grammar.

### 2.12 The final run

The last edits were the doc fix, explicit mutants timeouts, singular messages, the map and the docs. After them, the rule checksums were rewritten last and everything was run again:

```
$ (cd engine && tools/check-rules --write-frozen && tools/check-frozen --write-rules)
check-rules: wrote engine/tools/frozen-lints.toml
check-frozen: wrote engine/tools/RULES.sha256 (149 files)
$ bun engine:check
check full: green, 24 steps (133s)
[exit 0]
    of which: rules-selftest: ban coverage 455/455 entries fire; green, 219 cases over 77 rules (37s)
              guard-test: 725 checks, 0 failed
              fuzz: Done 105157252 runs in 61 second(s)
              doc: no warnings
$ bun engine:spec-check
total: 0 refusals
check-frozen: notice: engine/tests/TESTS.sha256 isn't written yet (T writes it); skipped
check-frozen: spec, tag and rules unchanged
[exit 0]
$ bun engine:test
     Summary [   0.008s] 3 tests run: 3 passed, 0 skipped
[exit 0]
$ bun engine:bench
parse_stub              time:   [288.65 ps 288.95 ps 289.29 ps]
[exit 0]
$ bun engine:cov
     Summary [   0.013s] 3 tests run: 3 passed, 0 skipped
    Finished report saved to …/engine/target/llvm-cov/html
[exit 0]
$ bun engine:fuzz -max_total_time=10
Done 19094852 runs in 11 second(s)
[exit 0]
$ bun engine:mutants
MISSED   crates/tally/src/lib.rs:18:5: replace parse -> Result<(), NotImplemented> with Ok(()) in 3s build + 0s test
mutants-survivors: score 0.0% = caught 0 / (caught 0 + missed 1 + timed out 0); unviable 0, not counted
[exit 0]
$ actionlint .github/workflows/engine.yml
[exit 0]
```

### 2.13 After the build: the ignore lines and CI on `engine` (2026-10-23)

The user answered open questions 1 and 2 yes (section 3). The change, in one commit on top of the build:

- **`eslint.config.js`:** `'engine/'` in `ignores`.
- **`.prettierignore`:** `engine/target/`, `engine/fuzz/target/`, `engine/mutants.out/` and `engine/mutants.out.old/`. The question named the first two. The run showed cargo-mutants' two output folders failing Prettier the same way, so they were added too.
- **`.github/workflows/engine.yml`:** `branches: [master, engine]`. The workflow is a rule file, so this went through R-112: the approval is dated in R-77's rulebook row, the map entry for R-77 in SCAFFOLD.md names both branches, and `RULES.sha256` was rewritten last.
- **`rules_selftest.py`:** R-77's check also fails when the trigger doesn't name both branches. Proof that the check fires, run against the real workflow and against one with `[master]` planted:

```
real: []
planted: ["R-77: .github/workflows/engine.yml doesn't run on pull requests into master and engine"]
```

The run:

```
$ (cd engine && tools/check-frozen --write-rules)
check-frozen: wrote engine/tools/RULES.sha256 (149 files)
 engine/tools/RULES.sha256 | 4 ++--
$ bun engine:check
check full: green, 24 steps (132s)
[exit 0]
$ (cd engine && tools/check-frozen)
check-frozen: spec, tag and rules unchanged
[exit 0]
$ actionlint .github/workflows/engine.yml
[exit 0]
$ bun lint        (engine/target/doc present)
Every copy in .claude/ is its package's file, every patch is in order, nothing is linked, and Bun is pinned.
[exit 0]
$ bunx prettier --check .
[warn] .claude/skills/react-frontend-best-practices/STATE.md
[warn] docs/index.md (in the drizzle-fixtures package)
[warn] docs/traits.md (in the drizzle-fixtures package)
[warn] claude/skills/react-frontend-best-practices/STATE.md (in the toolkit package)
[exit 1]
```

The four files Prettier still flags are outside `engine/` and fail on `master` too. They were there before this work and are left alone.

## 3. The user's answers at both gates, and after

**Gate 1, the rulebook (2026-10-22).** The user answered group by group. Each of the 113 rows holds its answer, dated, in `tmp/research_notes/engine-rust-strictness/RULEBOOK.md`. Their words:

- Group 1 (panics): "keep all, ban debug_assert, abort on crash". R-4 also bans the `debug_assert*` macros, and R-9 aborts on a crash, with no catching.
- Group 2 (numbers): first a question: how is an exchange rate handled when floats are forbidden? Answer: it is entered as a decimal and held exactly as a rational number, since the spec has no float. Then: "keep all of group 2".
- Groups 3 and 4 (determinism, supply chain): "keep all of group 3", "keep all of group 4".
- Group 5 (style): "keep all of group 5, allow map_err". R-66 keeps `map_err` allowed.
- Group 6 (tests and gates): "keep all of group 6, allow-list, T's tests, lead, approve, b". In order:
  - the build guard is an allow-list that fails closed (R-79);
  - the mutation score counts T's tests only (R-84);
  - the lead writes the fuzz targets and benchmarks on request (R-91);
  - the score's definition is approved: caught ÷ (caught + missed + timed out), with unviable mutants reported, not counted (R-87);
  - the long fuzz runs on the user's Mac as a self-hosted runner, option b (R-92).
- Group 7 (exceptions): "keep all of group 7, keep those budgets". The budgets of RULEBOOK §7.1 are approved as proposed (R-111).
- The six drops (R-17, R-32 to R-35, R-101, R-102) were dropped as recommended.

**Gate 2, the revised C and the installs (2026-10-22): "install yes, actionlint yes, pins yes".**

- **Installs:** rustup with stable 1.99.1 (clippy, rustfmt, llvm-tools-preview, rust-analyzer) and nightly-2026-10-21, then the six cargo tools at the versions above, with `cargo install --locked`.
- **actionlint:** 1.7.12, via Homebrew.
- **Pins:** num-bigint `=0.4.8`, num-integer `=0.1.47`, num-traits `=0.2.19` and sha2 `=0.11.1`, default features off.

The self-hosted runner agent waits until the user fetches a registration token from GitHub.

**After the build (2026-10-23): "#1 -> yes, #2 -> yes"**, to open questions 1 and 2 (section 2.13), then "Also afterwards we should open PR with these changes".

## 4. Deviations

**From C:** `engine/SCAFFOLD.md` §8 lists all 27, each with its reason. The ones an auditor should weigh first:

- the counter prints 5, not 0 (item 1);
- the ignore lines in two files outside C's write list, added with the user's go (item 2);
- C's cargo-deny command was wrong (item 3);
- the cargo-mutants baseline workaround and its explicit timeouts (item 4);
- the four checks added where a tool turned out blind (items 5 to 8).

**From the phase-1 brief:**

1. **§8, "the exception counter prints 0":** it prints 5. The reason is above.
2. **§7.3, `engine/tools/rules-selftest`:** it is a directory, holding `run` and `cases/`, because one file per rule keeps 219 cases reviewable.
3. **§8, "everything in C §7":** `bun lint` passed only where the engine hadn't been built, until the user approved the ignore lines. It passes everywhere now (sections 2.10 and 2.13).

## 5. Open questions

1. **Two ignore lines outside C's write list.** Answered yes (2026-10-23) and done, with cargo-mutants' two output folders added to `.prettierignore` too (section 2.13).
2. **CI on the `engine` branch.** Answered yes (2026-10-23). `engine.yml` now runs on pull requests into `master` and `engine` (section 2.13).
3. **The CLI's exception budget, 3, is full at the scaffold.** Any further I/O in the CLI needs a budget raise through R-112.
4. **A rule change and its checksum are committed together** (R-112), so the checksums can't catch a `rules`-role commit. The wall for those is the user's dated approval, plus the audit reading every such commit. Is that enough?
5. **The tripwire can't see what an interpreter's script writes.** The checksums catch it at the next `check fast`, which the Stop hook runs on every build turn, and on CI. Is that acceptable, or should the build role refuse interpreters outright? Refusing them would also refuse the builder's own scripts.
6. **Merging.** The build sits in the worktree C §2 rule 3 asks for, `/Users/jswob/projects/trop-engine-scaffold`, on `engine-scaffold`. The user asked why it isn't in the monorepo: it is, on a branch of its own. After the audit, `engine-scaffold` merges into `engine`, through the pull request opened on 2026-10-23. The user's uncommitted files in the main checkout (`apps/workbench/BANS.md`, `apps/workbench/DEFINITION.md`, `playground.md`) were never touched.
7. **Gate 8's survivor rule (SO-28 below).** `MUTANTS-SURVIVORS.md` asks every survivor for a reason naming a rule. DONE says the reason must be that the mutant is equivalent. Is a survivor that isn't equivalent allowed within the 10 %?

## 6. Spec observations (not fixed; brief §2 rule 2)

These come from the research (`NOTE.md` §4). The lead checked every line against `engine/spec/` on 2026-10-22. The build found nothing new in the spec. `engine/spec/` is untouched:

```
$ git diff --stat tally-1.0.1 -- engine/spec
[exit 0, no output]
$ git status --short -- engine/spec
[exit 0, no output]
```

- **SO-1** (ENGINE l.41–42; LANGUAGE l.1673). "engine build" is undefined. If it holds a toolchain, target or build time, the result hash differs between machines and rebuilds, against "the same output … on any machine" (LANGUAGE §7.5) and PERFORMANCE l.326. From A1 S-1, A4 2.
- **SO-2** (ENGINE l.41, l.108; LANGUAGE l.977). The result hash names no hash function and no byte encoding of its tuple. SHA-256 is named only for draws. From A4 1.
- **SO-3** (LANGUAGE l.1182, l.1194). How file paths enter the reality hash is undefined: spelling, separators, case, Unicode normalisation, sort order. From A4 3.
- **SO-4** (LANGUAGE l.1582). With several load refusals, which are reported and in what order is undefined. Gate 5 wants "exactly that code and message, and nothing else". From A2 O4, A4 4.
- **SO-5** (LANGUAGE l.1586). `LOAD-PARSE`'s `<what was expected>` has no fixed wording; tabs, bad indentation and chained comparisons share the code with no message text. From A2 O5.
- **SO-6** (ENGINE l.387, l.437). `ENGINE-INVARIANT` is defined for §9 failures only. An impossible internal state (a bad item id, a missing memo slot) has no code. From A2 O1.
- **SO-7** (DONE l.23 (gate 6); BUILD-ORDER l.74). `format(parse(x)) = x` holds only for canonical x. For generated input: `format(parse(x))` is canonical, `parse(format(t)) = t`, `format` idempotent. M0 needs every shipped file byte-canonical. From A2 O2.
- **SO-8** (PERFORMANCE §9). No input-size cap and no code for a file too large to handle (fine with `usize` offsets). From A2 O3.
- **SO-9** (LANGUAGE l.233). Operator and postfix chains are unbounded and add no depth, so the AST must be flat or every walk iterative, derived traits included. Not a defect. From A2 O6.
- **SO-10** (ENGINE l.141; LANGUAGE l.759). "Never by recursion on a stack" covers recurrences, not computed-key chains or chains of defs; PERFORMANCE §9 has no row for item-graph depth. From A2 O7.
- **SO-11** (LANGUAGE l.233). An `else if` ladder counts toward `LOAD-DEPTH`, so about 9 arms are refused. An observation. From A2 O8.
- **SO-12** (LANGUAGE l.1450, l.761). Whether intermediate results inside one expression count toward `RUN-DIGITS`, and in what order, is unstated: a gate-10 divergence risk. From A3 O-1.
- **SO-13** (PERFORMANCE l.249–256). The growth example uses a 2-decimal rate; LANGUAGE's own rates give 1,406–1,945 digits after 360 months. From A3 O-2.
- **SO-14** (LANGUAGE l.1499). The exact-edge list reads as examples, not a closed list; one missed case costs the whole 2^16-bit ladder, then `RUN-UNDECIDED` where another implementation prints. From A3 O-3.
- **SO-15** (PERFORMANCE l.256, l.265). The cost model counts multiplications, not the gcds of a reduced rational operation: about 10× optimistic unless the gcd is fast. From A3 O-4.
- **SO-16** (LANGUAGE l.1505). The `≈` bound: a widened interval can contain a pole or a domain edge; what prints then is unsaid. The bound's own printing is a decision by refinement with no cap or refusal named. From A3 O-5, O-6.
- **SO-17** (LANGUAGE l.1500). "Within 2^16 bits of precision" depends on the implementation; "within 2^−65536 of a cell edge" doesn't say relative or absolute. From A3 O-7.
- **SO-18** (LANGUAGE l.999). `poisson`'s quantile "is decided exactly", yet each test carries e^(−mean); the cap and `RUN-UNDECIDED` aren't mentioned for it. From A3 O-8.
- **SO-19** (LANGUAGE l.1006; PERFORMANCE l.215, l.297). "0.3–1 µs" per draw needs a heap-free first precision; with BigInt intervals about 5–10 µs. It is an estimate, not a §8 budget row. From A1 S-7, A3 O-9.
- **SO-20** (PERFORMANCE l.245; BUILD-ORDER l.84). One says i64 then bignum, the other i64, i128 and bignum. Wording only. From A3 O-10.
- **SO-21** (LANGUAGE l.1000; ENGINE l.350). `binomial` is "exact", but its cdf denominator passes the cap at large n (`binomial(10 000, 0.9)` has 10,001 digits). From A3 O-11.
- **SO-22** (PERFORMANCE l.219). "Exact integer sums are order-free" should say rationals, and the path-order merge is required, not merely sufficient, because `RUN-DIGITS` is checked on partial sums (LANGUAGE l.1531). From A1 S-4.
- **SO-23** (PERFORMANCE l.218). "Standard errors are exact" needs a square root, which LANGUAGE §7.3 makes a transcendental. From A1 S-5.
- **SO-24** (PERFORMANCE l.217). "16 bytes × 100 000 paths × 10 terms" assumes fixed-size values; exact rationals are variable-size. From A1 S-6.
- **SO-25** (PERFORMANCE l.326). "to the grosz" names one currency in a universal engine's spec. Wording only. From A1 S-8.
- **SO-26** (DONE l.26, l.44). Gate 9's "8 hours" doesn't say continuous or cumulative; a hosted job stops at 6 h. From A1 S-2.
- **SO-27** (DONE l.29). "The reference machine" is defined nowhere. From A1 S-3.
- **SO-28** (DONE l.25). Gate 8 doesn't say whose tests kill mutants, how unviable and timed-out mutants count, who maps "the kernel" to files, or how non-equivalent survivors fit: "every surviving mutant is listed, each with the reason it is equivalent" leaves no room for them within the 10 %. From A5 1.
- **SO-29** (BUILD-ORDER l.55 vs DONE l.25). S's mutation scope omits `plan_low` and the ledger, which gate 8 includes and S builds (BUILD-ORDER l.47–48). From A5 2.
- **SO-30** (DONE l.10, l.26, l.29). No deliverable owns the fuzz targets and benchmarks, the evidence of gates 9 and 12. From A5 3.
- **SO-31** (DONE l.33, l.3). Who runs and tags a milestone gate is unsaid; l.3 says the final gates are run by a gate session. From A5 4.
- **SO-32** (BUILD-ORDER l.274). "A test per construct that a missing row gives a hole" has no owner; if it is the builder's, it sits uneasily with DONE rule 2. From A5 5.
- **SO-33** (DONE l.25). cargo-mutants mutates only workspace code, so "the numbers and rounding" measures the engine's wrapper, not the external big-integer crate. Probably intended; worth saying. From A5 6.
