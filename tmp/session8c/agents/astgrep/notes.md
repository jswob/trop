# Session 8c step 3, agent `astgrep`: notes

The patch is `patch.diff` (against `05103df`): 12 ast-grep rules, 12 test files, 5 new case files and 4 extended ones. Nothing else in WT changed. Nothing committed.

## 1. The rules

Common to all: ast-grep sees source, not expanded code, so **a macro's arguments and bodies are token trees it can't parse** (`format!("{}", v[0])`, `vec![x.unwrap()]`). Only `no-dyn` looks into token trees (for the `dyn` keyword). Names reached through an alias or a `use … as` aren't seen either.

"Test modules" means a `mod` item preceded by `#[cfg(test)]`. Other attributes and comments may sit between them; the house rules (`no-debug-format`) accept only the attribute right before the `mod`. Not exempt: `#[cfg(all(test, …))]`, a `#[cfg(test)] fn` outside a test module, and **a test module kept in its own file** (`#[cfg(test)] mod tests;` with the body in `src/…/tests.rs`). The same limit already applies to `no-debug-format`. See §4 item 6.

| # | Rule id, file | Severity | What it matches | Cases |
|---|---|---|---|---|
| 1 | `no-dyn`, `rules/no-dyn.yml` | error | every `dynamic_type` (`&dyn T`, `Box<dyn T>`, `dyn Fn`, `impl dyn T`), plus a `dyn` token inside a macro's arguments. Files `crates/**`, `tests/runner/**`, tests included | R-122 `dyn-trait-object`, `dyn-in-test-module` |
| 2 | `no-macro-definition`, `rules/no-macro-definition.yml` | error | every `macro_rules!` in `crates/*/src/**`. The lead's list is `ignores: []`, with a comment. Test modules are not exempt (the brief gives no exemption). `macro-definition-review` is unchanged and still lists every macro, listed files included | R-73 `macro-definition-outside-list` (+ the old `macro-definition`) |
| 3 | `derive-order-review`, `rules/derive-order-review.yml` | warning | an attribute whose text has `derive(` … `PartialOrd` or `Ord` as a whole word, `cfg_attr(…, derive(Ord))` and paths like `core::cmp::Ord` included | R-118 `derive-ord` |
| 4a | `type-parameter-review`, `rules/type-parameter-review.yml` | warning | each type or const parameter in `<…>` on a fn, struct, enum, union, type alias, impl, trait or associated type; **also `impl Trait` in a parameter's type** (a type parameter without a name; see §4 item 4) | R-124 `type-parameter` |
| 4b | `trait-definition-review`, `rules/trait-definition-review.yml` | warning | every `trait` item | R-124 `trait-definition` |
| 4c | `lifetime-review`, `rules/lifetime-review.yml` | warning | each named lifetime **where it is declared**: `<'a>` on an item or impl, and `for<'a>`. `'static` and `'_` aren't listed; the workspace denies `elided_lifetimes_in_paths`, so `Formatter<'_>` is required. Loop labels are a different node. Each use of a named lifetime refers to a declaration, so every explicit lifetime is listed once, through its declaration, not once per use | R-124 `lifetime` |
| 5a | `long-chain-review`, `rules/long-chain-review.yml` | warning | 5 or more `.m(…)` calls along one receiver chain. `?`, turbofish and field accesses between them don't break the chain; one hit per chain, on its last call | R-125 `long-chain`, `four-step-chain-not-listed` (pass) |
| 5b | `nested-closure-review`, `rules/nested-closure-review.yml` | warning | every closure with a closure among its ancestors | R-125 `nested-closure` |
| 6 | `inline-hint-review`, `rules/inline-hint-review.yml` | warning | `#[inline(always)]`, `#[cold]`, also inside `cfg_attr`. Not `#[inline]` or `#[inline(never)]`. The fences are `ignores: []`, with a comment | R-130 `inline-always`, `cold` |
| 7 | `no-index`, `rules/no-index.yml` | error | every `index_expression` (`v[i]`, `&v[1..]`, `m[&k]`, `v[0] = x`) on any type, in `crates/**` minus `crates/*/tests/**`, `crates/*/benches/**` and test modules | R-5 `map-index`, `index-ast-grep`, `tests-may-index-ast-grep` (pass) |
| 8 | `no-unwrap-path`, `rules/no-unwrap-path.yml` | error | a `scoped_identifier` naming `Option`/`Result` then `unwrap`, `expect`, `unwrap_err` or `expect_err`: with or without a leading `::`, a `std::`/`core::` `option`/`result` prefix, a turbofish (`Option::<u8>::unwrap`) or a bracketed type (`<Option<u8>>::unwrap`). Both `.map(Option::unwrap)` and `Option::unwrap(x)`. Same files and exemptions as `no-index` | R-2 `unwrap-method-ref`, `result-expect-path`, `tests-may-unwrap-path` (pass) |
| 9 | `no-underscore-discard`, `rules/no-underscore-discard.yml` | error | `_ = <call>` and `let _<name> = <call>` (`mut` and a type annotation allowed), where the right side is a call, a method call or a macro call. In `crates/*/src/**`, test modules exempt. A call ending in `?` isn't a discard (its error is returned) and isn't listed | R-66 `underscore-assign`, `underscore-named-binding`, `tests-may-discard` (pass) |

**`let _ = …` (brief §3 item 9):** it is caught by clippy's `let_underscore_must_use` (deny in `Cargo.toml` l.120, R-66's existing case `let-underscore-result`) for every `#[must_use]` value, `Result` included. Rustc's `let_underscore_drop` (deny, l.44) catches values that have a destructor. `no-underscore-discard` leaves `let _ =` alone.

**Scope of the report rules (3–6):** `crates/*/src/**`, test modules exempt. Benches, `crates/*/tests/`, `tests/runner/` and `fuzz/` aren't scanned. The reason: `tools/exceptions` writes every report hit into `EXCEPTIONS.lock`, and the bench stub's criterion call `|b| b.iter(|| …)` is a closure inside a closure. That shape is criterion's API. See §4 item 3.

## 2. Known limits (what each rule can't see)

- **no-dyn:** none in parsed code. A `dyn` inside a macro's arguments is caught as a token; a `dyn` inside a string literal is not (correct). Bare trait objects (`Box<Trait>`) are a hard error in edition 2021, so they never reach the rule.
- **no-macro-definition:** `macro_rules!` only; `macro` (macros 2.0) is unstable. Proc macros need a proc-macro crate, which the deny lists govern.
- **derive-order-review:** derives only, matched as text. A hand-written `impl Ord`/`impl PartialOrd` isn't listed (R-118 names derives). A derive generated by another macro isn't seen.
- **type-parameter-review:** `-> impl Trait` (return position) isn't listed, since the function picks that type itself. Generics inside macros aren't seen.
- **trait-definition-review:** it lists every trait; it can't count implementations ("two today" is review's).
- **lifetime-review:** bounds like `T: 'a` and `'b: 'a` aren't listed apart (they refer to declared lifetimes). It can't tell a short-lived view from any other use; review judges.
- **long-chain-review:** it counts method calls only. The chain's first receiver, even when it is a call (`Foo::new().a().b().c().d()`), isn't counted. A parenthesised receiver, `&`/`*`, an index, a cast or a macro call ends the chain. R-125's "each closure a single expression" stays review only.
- **nested-closure-review:** none in parsed code. A closure in a `fn` item written inside a closure also counts (it is still nested).
- **inline-hint-review:** none in parsed code. A fence is a whole file (an `ignores:` glob), not a module inside a file.
- **no-index:** none in parsed code: every index expression is banned, as R-5 asks. `Index::index(v, i)` written as a path isn't an index expression (the audit's clippy.toml part of F-12 covers it). Indexing inside macro arguments isn't seen; clippy's `indexing_slicing` still sees slices and arrays there after expansion, not maps.
- **no-unwrap-path:** name-based. Not seen: an alias (`type O = Option<u8>; O::unwrap`), `use std::option::Option as O`, and a path built inside a macro. The audit's clippy.toml entries (`std::option::Option::unwrap` etc., outside this patch) resolve paths and would see those. A crate-local type that is itself named `Option`/`Result` and has an `unwrap` would be flagged; the engine has none.
- **no-underscore-discard:** text-level, no types. **It flags any call, fallible or not:** `let _len = v.len();` is an error too (§4 item 5). It misses a discard whose right side isn't a call directly: `_ = (f(), g())`, `_ = if c { f() } else { g() }`, `_ = match …`, and tuple patterns like `let (_a, _b) = f()`. A `#[cfg(test)] mod tests;` kept in its own file isn't exempt.

## 3. The runs (in WT/engine, unless said)

| Run | Exit | Summary line |
|---|---|---|
| `ast-grep test --skip-snapshot-tests` | 0 | `test result: ok. 27 passed; 0 failed;` (15 old + 12 new rules) |
| `ast-grep scan` | 0 | no output: no error and no warning on the stub |
| `tools/exceptions --check` | 0 | `exceptions: 5 (tests 0, fuzz-benches 0, kernel 0/20, lexer-parser 0/0, thread 0/1, cli 3/3, library 0/0, deny 2); report-only hits: 0`: total unchanged at 5 |
| `tools/check-frozen` (not asked; it explains the next row) | 1 | `check-frozen: 33 problems`: exactly the patch's 33 rule files (29 new, 4 changed), "not in RULES.sha256" / "checksum differs". The lead's R-112 commit runs `check-frozen --write-rules` |
| `tools/rules-selftest/run R-2 R-5 R-66 R-73 R-118 R-122 R-124 R-125 R-130` | see below | see below |

**Why the selftest ran in a mirror:** the selftest's clean run calls `tools/check-frozen`, which fails in WT because of the row above. WT may not hold a rewritten `RULES.sha256` (brief §1.3), so I copied `WT/engine` (minus `target/`), plus `package.json`, `.github/workflows/engine.yml` and `.claude/settings.json`, to `WT/../scratch/mirror/`. There I ran `engine/tools/check-frozen --write-rules`: 179 files, and the only lines that differ from WT's file are the patch's 33. Then the selftest ran in the mirror. A selftest in WT itself would fail at the clean run, with only check-frozen's 33 problems.

Selftest in the mirror: **exit 0**, 37 s. Its three summary lines:

```
rules-selftest: the clean run passes every tool, nothing is silently off
rules-selftest: ban coverage 455/455 entries fire; fakes for chrono
rules-selftest: green, 31 cases over 9 rules (37s)
```

The clean run includes `ast-grep scan`, `exceptions --check` and `check-frozen` on the whole engine with the 12 new rules on, plus `--inspect entity`, which shows every rule on. The 31 cases are R-2 6, R-5 6, R-66 6, R-73 2, R-118 1, R-122 2, R-124 3, R-125 3, R-130 2. Five of them are `pass` cases: the test-module exemptions and the 4-step chain.

I also checked how the new rules interact with the other rules' cases. I ran the 12 rules over every planted Rust block in all `cases/*.case` (a throwaway copy, removed afterwards). They fire only in positive cases of other rules: `macro_rules!` plants in R-13, R-18 and R-103, and index plants in R-5, R-6 and R-9. Those cases expect their own id, and R-9's is a `cargo build` run, so none of them changes. No `pass` case is hit.

**Exceptions with the new rules on:** today's stub gives no report hit, so the lock is unchanged. From now on, every report hit in `crates/*/src/**` becomes a `report-only` line in `EXCEPTIONS.lock`, by design. The builder's first generic, trait, lifetime or long chain changes the lock and needs `tools/exceptions --write`.

## 4. For the lead to decide

1. **`rules_selftest.py` stops at R-113.** `completeness()` and `list_rules()` use `range(1, 114)`. `--list` doesn't show R-118 … R-130, and the completeness check wouldn't notice if their cases were missing. The cases still run when the rules are named, as here. Raise the bound to the rulebook's last row (`range(1, 132)`) in the same R-112 commit. That file isn't mine to edit.
2. **`RULES.sha256`:** the commit(s) need `tools/check-frozen --write-rules` (the patch's 33 files).
3. **Scope of the report rules:** `crates/*/src/**`, test modules exempt (§1). Widen it to benches, `tests/runner` or test modules if review wants those listed. The bench stub would then add a `nested-closure-review` line to `EXCEPTIONS.lock`.
4. **`impl Trait` in argument position** is in `type-parameter-review`, since it is a type parameter without a name. To drop it, delete the `abstract_type` branch and its two invalid tests.
5. **`no-underscore-discard` flags every call**, not only fallible ones (no type information). Today's stub has none. If an infallible `let _x = f();` appears, the fix is plain code (`f();`, or a used binding). RAII guards (`let _guard = …`) would also be refused; the engine has no locks.
6. **Test modules in their own file** (`#[cfg(test)] mod tests;`) aren't exempt from `no-index`, `no-unwrap-path`, `no-underscore-discard` (or the existing `no-debug-format`). Either the builder keeps test modules inline (BUILDER can say so) or the rules get an `ignores:` glob for such files.
7. **The test-module helper** is a local `utils:` entry repeated in each of the 9 rules that exempt tests: `sgconfig.yml` has no `utilDirs`, and that file isn't mine. It accepts other attributes and comments between `#[cfg(test)]` and `mod`. `no-debug-format` keeps the stricter form. If one global util is wanted, add `utilDirs` to `sgconfig.yml`.
8. **R-73 in test modules:** `no-macro-definition` errors there too, as the brief gave no exemption. Add a test-module exemption if test macros should pass.
9. **F-12's other half** (clippy.toml's `disallowed-methods` entries for `Option`/`Result` unwrap and expect, `Sum::sum`, `Product::product`, `Index::index`, `IndexMut::index_mut`) isn't in this patch. With the unwrap entries added, `no-unwrap-path` and clippy overlap on plain paths; the ast-grep rule stays the one that works without type resolution, and in `ast-grep test`.

## 5. SCAFFOLD §7 rows (new or updated), in §7's format

- **R-2** No `unwrap` or `expect` outside tests. Tests may use them. **Enforced by:** clippy, error with no exception (`unwrap_used`, `expect_used`); tests exempted in clippy.toml; ast-grep (`no-unwrap-path`, error) for the forms written as paths (`map(Option::unwrap)`, `Result::expect(r, …)`), tests exempt. **Proven by:** selftest: `unwrap`, `expect`, `tests-may-unwrap`, `unwrap-method-ref`, `result-expect-path`, `tests-may-unwrap-path`.
- **R-5** No `x[i]` indexing or `x[a..b]` slicing on lists, arrays or maps; use `.get` and handle the miss. **Enforced by:** clippy, error (`indexing_slicing`); tests exempted; ast-grep (`no-index`, error) on every index expression outside tests, maps included, which `indexing_slicing` skips. **Proven by:** selftest: `index`, `slice`, `tests-may-index`, `map-index`, `index-ast-grep`, `tests-may-index-ast-grep`.
- **R-66** Errors are never dropped silently: no `let _ = fallible`, no `_ = fallible`, no `let _name = fallible`, no discarded `.ok`. `map_err` stays allowed. **Enforced by:** clippy, error (`let_underscore_must_use`, `unused_result_ok`); rustc (`let_underscore_drop`); ast-grep (`no-underscore-discard`, error) on `_ = call()` and `let _name = call()` in library code, test modules exempt. **Proven by:** selftest: `let-underscore-result`, `discarded-ok`, `map-err-allowed`, `underscore-assign`, `underscore-named-binding`, `tests-may-discard`.
- **R-73** No `macro_rules!` in the library outside the lead's list of files; a listed macro is still reported for review. **Enforced by:** ast-grep (`no-macro-definition`, error; the list is its `ignores:`, empty at the scaffold, changed through R-112); ast-grep (`macro-definition-review`, report). **Proven by:** selftest: `macro-definition`, `macro-definition-outside-list`.
- **R-118** Every derived `PartialOrd` or `Ord` on the engine's own types is listed for review. **Enforced by:** ast-grep (`derive-order-review`, report). **Proven by:** selftest: `derive-ord`.
- **R-122** No `dyn` anywhere in the engine, tests included. **Enforced by:** ast-grep (`no-dyn`, error). **Proven by:** selftest: `dyn-trait-object`, `dyn-in-test-module`.
- **R-124** No own type parameters; no new trait without two implementations today; explicit lifetimes only on short-lived views. Each hit is listed for review. **Enforced by:** ast-grep (`type-parameter-review`, `trait-definition-review`, `lifetime-review`, reports), then review. **Proven by:** selftest: `type-parameter`, `trait-definition`, `lifetime`.
- **R-125** Iterator chains of about 4 steps at most, each closure a single expression, no closure inside a closure. **Enforced by:** review; ast-grep (`long-chain-review` for 5 or more method calls, `nested-closure-review`, reports). **Proven by:** selftest: `long-chain`, `nested-closure`, `four-step-chain-not-listed`.
- **R-130** Optimised code lives only in named fence modules, each with a `PERF:` header, a benchmark, a plain twin and an equivalence test. `#[inline(always)]` and `#[cold]` outside a fence are listed for review. **Enforced by:** review; the `trace` script; ast-grep (`inline-hint-review`, report; the fences are its `ignores:`, empty at the scaffold, added through R-112). **Proven by:** selftest: `inline-always`, `cold`.
