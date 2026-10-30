# Builder guidance (not part of the scaffold)

**Made:** 2026-10-22 by the phase-1 lead, from A1 §6, A2 §5, A3 §5, A4 §5 and A5 §5 (this directory). **For:** the build session's brief, later.

This is how the research says the engine should be written. The scaffold doesn't encode any of it. Items marked **[Tally syntax]** rest on LANGUAGE §2 as frozen in tally 1.0.1, and change if the design round changes the syntax. Rule ids (R-n) point into `RULEBOOK.md`.

**Correction to A5 §5.2 item 6.** It says "no floats outside the Monte Carlo module". There is no such module. The spec has no floating point anywhere (LANGUAGE l.1451, l.1531), and neither does this guidance (R-18).

## 1. Order of work

The number type comes first. Wrap the big-integer crate in the engine's own `Num` and `Money` before anything else is written on top, because every later module calls them, and R-19 fires on every number operation that doesn't go through them.

## 2. Numbers (A3 §5, A1 §6)

- **`Num`** is `enum Num { Small { n: i64, d: i64 }, Big(Box<BigRat>) }`.
  - Invariants: reduced; d > 0; n ≠ i64::MIN; d ≤ i64::MAX; Small whenever the value fits. With these, derived `PartialEq` and `Hash` are correct.
  - Write `Ord` by hand, cross-multiplying in i128.
  - The Small path uses i128 intermediates and i64 `checked_*`. A `None` there means "widen", so the branch is reachable and mutation testing can kill it.
  - Demote to Small after every Big result that fits.
- **`Money`** is `{ minor: Int, cur: Currency }` with `Int = I64 | I128 | Big`, always the smallest tier that fits.
  - Methods are `checked_add`, `checked_sub`, `checked_neg` and `to_num`, plus `Num::round_to_money(step, mode, cur)`. Use `checked_*` names: inherent `add` and `sub` trip clippy's `should_implement_trait`.
  - No operator traits, `Default`, `Sum`, or `From<i64>`/`From<f64>` (R-28). With no `Mul<Num>`, the compiler refuses `money * rate`, and the only way back to money is a `round`, which is LANGUAGE §7.2.
  - Currency is checked at load. At run time a mismatch is an invariant error, never a coercion.
  - Prove the missing impls in unit tests: `static_assertions::assert_not_impl_any!`, or a hand-written trait-bound test.
- **Rational arithmetic:** Knuth TAOCP 4.5.1. One gcd of the denominators per addition; cross-reduction per multiplication.
  - For `gcd(big, small)`, reduce `big mod small` first and finish in u64. num-bigint's binary gcd is about 10× slower than PERFORMANCE §6 assumes without this (A3 F-3).
  - Write a Lehmer gcd for balanced big values only if the benchmarks ask for it. Test any hand-written gcd against `num_integer::Integer::gcd`.
  - Never call a signed primitive gcd at i64::MIN; the invariant above excludes it.
- **The cap** (`RUN-DIGITS`):
  - Check it after every operation, on the reduced result, by bit length. ≤ 13,606 bits passes, ≥ 13,608 bits is refused, and only 13,607 bits needs a comparison with 10^4096.
  - Hold 10^4096 as a lazily built constant.
  - Pre-check `pow`, shifts and literal exponents before computing (R-26).
  - Until the spec answers SO-12, check intermediates in the expression's written order.
- **Rounding:** the 7 modes of LANGUAGE §7.2, once, on `div_floor`/`mod_floor` plus a tie test. Test every mode with negative values and ties; dashu's 2026 `mod_floor` bug shows why.
- **Errors:** `NumError { Div, Digits, Undecided, Domain, Param, Invariant }`, mapped to the `RUN-*` codes at the key. Every BigInt `/` and `%` goes through one helper that checks for zero.
- **Integers in dates and indices:** `checked_div_euclid` and `checked_rem_euclid` with a positive divisor. Write the Python equivalent (`//` and `%`) beside each, because the Python reference core floors where Rust truncates (gate 10). Use `is_multiple_of` and `TryFrom` at every boundary between widths.
- **The transcendental kernel** (LANGUAGE §7.3). No permissive crate has rigorous erf, erfc, Φ⁻¹ or incomplete gamma, so it is written here:
  1. Dyadic intervals (m·2^e, m a BigInt), rounded outward, each function's error bound written beside its code with its reference.
  2. Exact paths first: sqrt to a step by integer `isqrt` and comparison; small rational powers by integer roots.
  3. Quantiles certified by cdf values at the cell edges: any guess, then check cdf((k − ½)u) < p < cdf((k + ½)u).
  4. Only then a heap-free fixed-width first precision for draws, checked against the BigInt path by a differential test.
  5. Exact-edge cases detected before the ladder starts.
  6. The worst case measured: the time to reach 2^16 bits.

## 3. The parser and the tree (A2 §5.1–§5.4, A4 §5)

- **Hand-written recursive descent**, one function per grammar rule, with operator chains as loops (precedence climbing or Pratt, as rust-analyzer and ruff do). No parser generator or combinator library: cedar's generated parser needs 8 blanket allows, and `LOAD-PARSE`'s message would follow the library's wording.
- **[Tally syntax]** Lex in two steps: split into lines, measure indentation (two spaces, no tabs; refuse tabs), then tokenise each line. Tally has no comments.
- **[Tally syntax]** Binary `-` has a space on each side, because a key may contain `-` (LANGUAGE l.71). A lone `-` is the "not applicable" atom when the next token can't start an operand (l.228). Both are lexer-context rules; test them first.
- **[Tally syntax]** At most one prefix `-` (l.198) and one `not`. Operator chains and postfix chains grow without limit and add no depth (l.233).
- **The tree:** an AST of owned enums with a span on each node, not a lossless CST, since Tally has one canonical format and no comments.
  - Spans are `usize` byte offsets in an own `Span { start, end }` with checked constructors. Avoid text-size and la-arena: both panic, and la-arena's indexing is invisible to `indexing_slicing`.
  - **[Tally syntax]** Keep chains flat: `Sum { first, rest: Vec<(AddOp, Expr)> }`, and likewise `Prod`, `And`, `Or` and the postfix chain. A left-nested chain of 100,000 terms is a tree 100,000 deep, and every recursive walk over it overflows the stack, including the derived `Drop`, `Clone`, `PartialEq` and `Debug`.
  - Items live in a table: an own `ItemId(usize)`, and a table exposing only `get(id) -> Result<&Item, Internal>`.
  - Names are `Box<str>` plus a `BTreeMap<Box<str>, ItemId>`; no interner. Never let an id decide output order (R-47). Don't derive `Ord` for any order that reaches output.
- **The cursor**, with no indexing and no raw arithmetic:

  ```rust
  pub struct Cursor<'a> { rest: &'a str, offset: usize }
  impl<'a> Cursor<'a> {
      pub fn peek(&self) -> Option<char> { self.rest.chars().next() }
      pub fn bump(&mut self) -> Option<char> {
          let c = self.rest.chars().next()?;
          self.rest = self.rest.get(c.len_utf8()..)?;
          self.offset = self.offset.checked_add(c.len_utf8())?;
          Some(c)
      }
      pub fn eat(&mut self, word: &str) -> Option<bool> {
          match self.rest.strip_prefix(word) {
              Some(rest) => { self.rest = rest; self.offset = self.offset.checked_add(word.len())?; Some(true) }
              None => Some(false),
          }
      }
  }
  ```

  A `None` from the cursor maps to one coded refusal at the call site (`ok_or(…)?`). Lines: `for (i, line) in text.split('\n').enumerate()`, displayed with `i.checked_add(1)`. **[Tally syntax]** Indentation: count leading spaces with `take_while`, check `is_multiple_of(2)`, level by `checked_div(2)`. Use `split_once`, `strip_prefix`, `char_indices`, `str::get`; slice patterns (`[first, rest @ ..]`, `split_first`) for token lists.
- **Depth**, counted and passed by value, refused before descending:

  ```rust
  #[derive(Clone, Copy)]
  pub struct Depth(u8);
  impl Depth {
      pub const LIMIT: u8 = 8; // [Tally syntax] LOAD-DEPTH, LANGUAGE l.233
      pub fn deeper(self, at: Span) -> Result<Depth, Refusal> {
          let d = self.0.checked_add(1).ok_or(Refusal::depth(at))?;
          if d > Self::LIMIT { return Err(Refusal::depth(at)); }
          Ok(Depth(d))
      }
  }
  ```

  **[Tally syntax]** Call `deeper` exactly where the spec counts a level: each `(`, `[`, `{` (calls, keys and `as if` included) and each `if`. `as if` keeps a second counter with limit 4 (`LOAD-ASIF`). Never measure the stack (R-16).
- **Progress:** every parser loop consumes a token or exits. A loop body that ends at its starting offset returns `LOAD-PARSE` instead of asserting, as rust-analyzer, ruff and biome do with a panic. A hang fails gate 9 like a crash.
- **Several refusals:** stop at the first unless the spec settles otherwise (SO-4). Whatever it settles, make the order deterministic: sort by (file, line, column, code).

## 4. The evaluator (A2 §5.4)

- **An explicit work stack**, not Rust recursion: `enum Task { Demand(Cell), Finish(Cell) }` on a `Vec`, dependencies pushed in sorted order, and an "in progress" set, which also gives `RUN-CYCLE` for free. ENGINE l.141 makes recurrences iterative; demand chains through computed keys and chains of defs would otherwise recurse (SO-10).
- **Load-time cycle checks** (`LOAD-CYCLE`): iterative DFS or Kahn's algorithm, over nodes in sorted order.

## 5. Impossible states without `unreachable!` (A2 §5.5)

- **Narrow the type at the point of decision.** The token classifier returns `Option<AddOp>`, so the node builder matches `AddOp` exhaustively and never re-matches `TokenKind`. This removes cedar's most common exception ("`op` is checked by the caller").
- **Take, don't peek:** `let Some(first) = iter.next() else { return Err(…) };`.
- **Classify exhaustively** instead of `matches!`: `fn is_comparison(op: BinOp) -> bool { match op { BinOp::Eq | … => true, BinOp::Add | … => false } }`, so a new variant must be decided (R-63).
- **What remains** (a bad `ItemId`, a missing memo slot) returns an internal error. Which code it prints is a spec gap (SO-6).
- **Mutation:** an unreachable error branch is a mutant nobody can kill. In the gate-8 kernel, prefer types that make the branch not exist.

## 6. Errors and codes (A2 §5.6, A1 §6)

- One `enum Code`, one variant per row of LANGUAGE §8, ENGINE §11 and OPERATIONS §10 (87 codes). `const ALL: [Code; 87]`, `fn as_str(self) -> &'static str` and the message function, each an exhaustive `match`.
- A unit test reads the spec's tables (read only) and compares them with `Code::ALL`, as rustc's tidy check does for its error codes.
- A refusal value carries code, message, file, line and item (LANGUAGE l.1582). The CLI's JSON shape is `{ code, message, … }` (engine-tests §4).
- No `anyhow`, `eyre`, `Box<dyn Error>` or `String` errors (R-58). `thiserror` is optional; a hand-written `Display` is fine. No rendering crate (miette, ariadne): the spec fixes the message text.

## 7. Determinism, I/O and output (A1 §6, A4 §5)

- **Collections:** `BTreeMap`/`BTreeSet`, or `IndexMap` where insertion order is the meaning, each use with a reason. Sort with `sort_by_key` on a total key, never `sort_unstable`; a comparator that isn't a total order may crash since Rust 1.81.
- **Threads:** one module, `std::thread::scope` over contiguous path blocks, joined in path order (ENGINE §7.5), worker count passed in. No global pool, no `available_parallelism`. Ask the lead to sanction the module (R-108).
- **The library takes the reality as in-memory files** (path and bytes). It reads no clock, environment or file system.
- **The CLI's `main`**, in the sanctioned file:
  - reads `std::env::args_os()`, turning a non-UTF-8 argument into the usage-error exit code;
  - reads the reality directory, sorting entries by name bytes (R-40), and decodes UTF-8 strictly (R-49);
  - calls `tally_cli::run(args, stdin) -> Output`, which is pure;
  - writes `stdout` and `stderr` with `std::io::Write::write_all` on locked handles, handling the error;
  - returns `std::process::ExitCode::from(code)`.
- **Output:** structs (fields serialize in declaration order) and `Vec`s already in the spec's canonical order, or a small hand-written writer. Exact numbers as strings: a decimal where the value terminates, else `p/q`. Never `serde_json::Value` or `Map`, and never a map keyed by strings.
- **Hashing:** `sha2` 0.11.1. Check it with NIST known-answer vectors in a unit test, run on both machines. The result hash's function and encoding are a spec gap (SO-2).
- **Machine integers** only for indices, counts and day numbers; fixed-width types, not `usize`, in anything hashed or printed (R-44).
- **The fault-injection hook** for gate 5 compiles only under `#[cfg(all(feature = "fault-injection", debug_assertions))]`, with `#[cfg(all(feature = "fault-injection", not(debug_assertions)))] compile_error!("…");` beside it (R-51).

## 8. When a lint blocks real work (A1 §6, A4 §5)

1. Restructure first: a slice pattern, an iterator, a checked operation, a narrower type.
2. Then one `#[expect(clippy::<lint>, reason = "R-<n>: why the safe form doesn't fit")]` on the smallest expression or item, within the area's budget. Run `engine/tools/exceptions --write` in the same commit.
3. Over budget, or no strict alternative: a STOP-REPORT.

Never edit `clippy.toml`, the lint table or the rules (the guard refuses). Never use `cfg(not(clippy))`, an inner `#![expect]`, an `#[expect]` on a module, or an `ast-grep-ignore` comment. Each fails CI by design.

| The workaround an LLM reaches for | The rule that answers it |
|---|---|
| `unwrap_or_default()`, `unwrap_or(0)`, `.get(i).unwrap_or(&0)` | R-27 (ban plus report), review |
| `saturating_*`, `wrapping_*` | R-23 |
| a macro that hides `unwrap` or `panic!` | R-13 |
| `#[allow]`, `#![allow]`, `cfg_attr(…, allow(…))` | R-103, R-107 |
| an `#[expect]` on a whole `impl` or module | R-107; review (the counter shows its line) |
| floats for a division | R-18 (no exception) |
| `a.add(b)` instead of `a + b` | R-20 |
| `.ok()`, `let _ =` | R-66 |
| `Box<dyn Error>`, `anyhow` | R-58 |
| `cfg(test)` behaviour, reading answer keys | R-93 |
| `#[mutants::skip]` | R-86 |
| turning a hook or lint off through config | R-79–R-82 |
| `clone()` storms to dodge the borrow checker | R-74 (report), review |

## 9. The loop (A5 §5.1, §5.3)

| When | What runs | Fails the turn? |
|---|---|---|
| after each edit | rust-analyzer diagnostics (plugin) | no: a speed aid |
| a group of T's vectors looks green | `engine/tools/check fast`: fmt, clippy, T's runner filtered to the milestone plus a fixed sample of earlier ones, the exception counter | yes, through the Stop hook |
| end of turn | the same, as a Stop hook | yes, unless a STOP-REPORT is newer than the last commit |
| before each commit | `engine/tools/check full`: the whole T suite in dev and release, cargo-deny, check-frozen, check-rules, `exceptions --check`; advisory `cargo mutants --in-diff` on kernel changes | never commit red |
| nightly | the full kernel mutation run, advisory lints, coverage, long fuzz, benchmarks | reported in the morning |
| milestone end | the fresh review (§12), then the gate session's re-run and tag | the gate session decides |

- Run only through `engine/tools/check`, never raw `cargo` with your own flags. Read the one-line failures; open a log only when needed. Never raise a timeout or add a retry. If a failure doesn't reproduce, report it; don't "fix" it.
- **One session per milestone step**, started at the repo root as `TALLY_ROLE=build`:
  - start: read the last `engine/BUILD-LOG.md` entries, the milestone's BUILD-ORDER section and the plan's traceability table; run `check fast` and note the red vector ids;
  - work: next group of red vectors in plan order; implement; `check fast`; commit when green; on any stop trigger, a STOP-REPORT, then move on or end;
  - end: append to BUILD-LOG what turned green, what's next, open STOP-REPORTs, the exception delta, and the check's last line.
- **Plan each milestone** in plan mode: map each BUILD-ORDER item to its spec sections and T's vector ids. At S, propose the kernel module list; the lead writes it into the protected mutants config (R-85).
- **STOP-REPORT** (A5 §2.3): when a T test looks wrong, the spec is unclear, a rule blocks a correct design with the budget used up, a crate or kernel file is needed, a survivor isn't equivalent and T has no test for it, or `check fast` stays red on the same failure after three honest tries. Leave the code as it was. Record the item, the smallest input, what the engine gives and what the test expects, which of model, spec, test, rule or code is wrong and why, what was tried, and what was *not* done. The prompt line: "If a test, the spec or a rule seems to conflict with correct behaviour, stop and file a STOP-REPORT. Never edit tests, never special-case inputs, never weaken a check. A STOP-REPORT is a good outcome."

## 10. Testing style (A5 §5.5, A3 §4–§5)

- **T's vectors decide "done".** The builder's tests make internals easy to get right and never count toward a gate.
- **Unit tests:** small, deterministic, no time or sleep; each cites the spec section it checks; exact values, never `assert!(r.is_ok())`; no golden outputs the builder wrote itself.
- **proptest** with a fixed seed in CI and persistence files only in the builder's crate: the number type's algebraic laws; the Small and Big tiers agree on every input both can hold; `round` idempotent; parse and format round-trip on generated input, in the form SO-7 gives.
- **A3's number tests**, which the builder writes once `Num` exists:
  - a differential against Python's `fractions.Fraction`, on random and boundary values (i64 limits, 2^63, 2^64, 10^4096 ± 1), all 7 modes, ties and negatives, on every PR;
  - the transcendental oracle: `decimal` (exp, ln) and `isqrt` per PR; Arb through python-flint, or mpmath, nightly (an install, so the user's go); a generator of inputs near cell edges;
  - fixtures at the cap: 13,606, 13,607 and 13,608 bits; 10^4096 − 1 passes, 10^4096 is refused; `pow` refused before it is computed;
  - kernel benchmarks at 20, 1,100, 1,400, 2,000 and 4,096 digits, balanced and big × small; the 360-step recurrence at r = 0.07 and 0.0585; a normal draw to 1e-9 on both paths.
- **Mutation:** expect unviable mutants under strict lints. Hunt survivors with tests at the public level (`run`), not tests aimed at the mutant; if only an internal test can kill it, ask whether the code is reachable. Survivors not killed by T are dead code (delete), a T gap (report through the lead), or equivalent (list with a reason).
- **Fuzz targets** (written by the lead on request, R-91): the parser never panics or hangs, every rejection carries a code, `parse(format(t)) = t`, and `format` is idempotent. cargo-fuzz builds with debug assertions and overflow checks on by default.

## 11. `engine/CLAUDE.md` outline (A5 §5.2, corrected)

At most 150 lines. Every hard rule appears twice, once as a sentence and once as the check that enforces it, which the sentence names.

1. What this is (3 lines): the spec in `engine/spec/` (frozen, read-only); T's tests in `engine/tests/` (read-only); you write `engine/crates/**`.
2. Hard rules, each with its enforcement: never edit tests, the spec or a rule file; never special-case an input; never add a crate; never weaken a check; a STOP-REPORT is a good outcome; the protocol in 10 lines.
3. The loop: §9's table, shortened; only `engine/tools/check`; what the Stop hook does.
4. Exceptions: `#[expect(clippy::x, reason = "R-n: …")]` only; the forbidden forms; the lock and the budget.
5. Write this instead (about 12 lines): an index becomes `.get(i).ok_or(…)?`; `a + b` on integers becomes `checked_add(…).ok_or(…)?`; `as` becomes `TryFrom`; `unwrap` becomes `?` or `ok_or`; `unwrap_or_default` becomes a hole; `/` and `%` become `checked_div_euclid`; `unreachable!` becomes a narrower type; `let _ =` becomes handling; `HashMap` becomes `BTreeMap`; clocks and env are never read.
6. Numbers: exact rationals only, through `Num` and `Money`; overflow and the cap are refusals with their codes; **no floats anywhere**.
7. Holes and refusals: a missing value is a hole with its question, never 0; an error carries its catalogue code; an invariant break returns `ENGINE-INVARIANT`, never a panic.
8. Determinism: sorted maps; no time, env or threads except the sanctioned merge module.
9. Tests: T's vectors define done; your tests don't count; fixed seeds; no snapshots; no `cfg(test)` behaviour; reproduce before you fix.
10. Code shape: simple; enums with exhaustive `match`; no new trait without two implementations today; cite the spec section in each kernel function's doc comment.
11. Hand-off: append to `engine/BUILD-LOG.md`; when compacting, keep the milestone, the red vector ids, open STOP-REPORTs and the files modified.
12. Commits: one green step each, naming the vector ids; never an AI co-author or attribution line.

## 12. The review checklist (A5 §5.4)

A fresh-context, read-only reviewer (Read, Grep, Glob, read-only Bash), given the diff range, the plan and the spec sections. It reports only gaps in correctness or the stated requirements, to `engine/reviews/<milestone>.md`.

1. Traceability: each plan item maps to code and green vectors; each cited spec section says what the code does; nothing outside the milestone changed.
2. Passing by the wrong means: literal-scan hits; branches on names, paths or ask texts; reads of `engine/tests/`; `cfg(test)` or `debug_assertions` behaviour; output assembled from expected strings; guard-log entries outside the allow-list.
3. Workaround patterns (§8).
4. Exception reasons: about the domain, not about the tool.
5. Holes and refusals: every missing-input path ends in a hole or a catalogue code; no invariant break panics.
6. Determinism: no new hash-ordered collection, clock, environment read or thread.
7. Mutation: each new survivor's equivalence argument holds; others go to T.
8. Evidence: the builder's report quotes commands, exit codes and counts; the reviewer re-runs `check full` once.
