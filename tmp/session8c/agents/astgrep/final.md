<!-- saved by tools/save_final.py from agent agent-a1b6f7ade0c7e5e54.jsonl (astgrep) -->

All 12 ast-grep rules are written and every run passes, but the selftest only passes in a scratch copy of the engine: in WT, check-frozen fails on the 33 new or changed rule files until `RULES.sha256` is rewritten. The patch is `/Users/jswob/projects/trop/tmp/session8c/agents/astgrep/patch.diff` (33 files, nothing committed), and the details are in `notes.md` beside it.

**Rules:**
- Errors: `no-dyn` (R-122), `no-macro-definition` (R-73, the lead's list is `ignores: []`), `no-index` (R-5), `no-unwrap-path` (R-2), `no-underscore-discard` (R-66).
- Reports: `derive-order-review` (R-118), `type-parameter-review`, `trait-definition-review` and `lifetime-review` (R-124), `long-chain-review` and `nested-closure-review` (R-125), `inline-hint-review` (R-130, the fences are `ignores: []`).
- `let _ = …` is already caught by clippy's `let_underscore_must_use` (R-66's existing case) and rustc's `let_underscore_drop`.

**Cases:** new files R-118, R-122, R-124, R-125 and R-130. R-2, R-5, R-66 and R-73 are extended, using the audit's names (`map-index`, `unwrap-method-ref`, `underscore-assign`, `underscore-named-binding`), with pass cases for the test-module exemptions and the 4-step chain.

**Runs, in WT/engine:**
- `ast-grep test`: exit 0, 27 passed.
- `ast-grep scan`: exit 0, no errors or warnings on the stub.
- `exceptions --check`: exit 0, total still 5, report-only hits 0.
- `check-frozen` (not on your list): exit 1, 33 problems, exactly the patch's files.
- Selftest for the nine rules: run in `WT/../scratch/mirror` with a rewritten `RULES.sha256`, since WT may not hold one. Exit 0, clean run green, 31 cases over 9 rules.

**Known limits:**
- No rule sees inside macros, except `no-dyn`, which catches a `dyn` token.
- `no-unwrap-path` matches names, so aliases and `use … as` escape it.
- `no-underscore-discard` flags any call, fallible or not, and misses tuple and `if`/`match` forms.
- A test module kept in its own file (`#[cfg(test)] mod tests;`) isn't exempt.
- `long-chain-review` counts method calls only, not a first call like `Foo::new()`.

**For you to decide:**
1. Rewrite `RULES.sha256` in the R-112 commit (`check-frozen --write-rules`).
2. `rules_selftest.py` only knows rules up to R-113, so `--list` and the completeness check skip R-118 to R-130 (their cases still run when named). Raise the bound to `range(1, 132)`.
3. The report rules scan `crates/*/src/**` only, test modules exempt. Benches are left out because criterion's `|b| b.iter(|| …)` would add a line to `EXCEPTIONS.lock`.
4. I counted `impl Trait` in a parameter as an own type parameter in `type-parameter-review`. Keep it or drop it.
5. `no-macro-definition` gives no exemption to test modules.
6. The test-module exemption is repeated in each of the 9 rules that exempt tests, because `sgconfig.yml` has no `utilDirs`.
7. F-12's clippy.toml half isn't in this patch.
