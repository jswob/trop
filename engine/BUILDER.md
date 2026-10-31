# Builder guidance (not part of the scaffold)

**Made:** 2026-10-22 by the phase-1 lead, from A1 §6, A2 §5, A3 §5, A4 §5 and A5 §5 (this directory). **For:** the build session's brief, later.

**Updated:** 2026-10-30 by session 8b, for tally 2.1, from session 8's gate G1 (`tmp/session8/decisions.md`: items 1 to 8, each answered "I approve" on 2026-10-30), the lead's note `tmp/research_notes/engine-prep/NOTE.md` (settlements K8-1..K8-17) and the reports R1–R6 there. Where G1 reverses a phase 1 sentence, the sentence is replaced and a line marked _Changed 2026-10-30_ says what changed and why. New: §13, performance. The copy before this update is `BUILDER.before-8b.md`.

This is how the research says the engine should be written. The scaffold doesn't encode any of it. Items marked **[Tally syntax]** rest on LANGUAGE §2 as frozen in tally 1.0.1, and change if the design round changes the syntax. _Note 2026-10-30:_ their line citations (l.71, l.198, l.228, l.233) are 1.0.1's; tally 2.1 moved the lines (`LOAD-DEPTH` is now at L l.325), and this update did not recheck the [Tally syntax] items. Rule ids (R-n) point into `RULEBOOK.md`; decision ids (D1-1, K8-1) into session 8's decisions and NOTE.

**Correction to A5 §5.2 item 6.** It says "no floats outside the Monte Carlo module". There is no such module. The spec has no floating point anywhere (LANGUAGE l.1451, l.1531), and neither does this guidance (R-18).

## 1. Order of work

The number type comes first. Wrap the big-integer crate in the engine's own `Num` and `Money` before anything else is written on top, because every later module calls them, and R-19 fires on every number operation that doesn't go through them.

## 2. Numbers (A3 §5, A1 §6; R2 Q1–Q2, D2-1, D2-2)

- **`Num`** is `enum Num { Small { n: i64, d: i64 }, Big(Box<BigRat>) }` (D2-1).
  - Two tiers, no middle one. In E1, all 1 693 results above 63 bits are also above 1 024 bits (`tmp/session8/shape.md`), so an i128 tier would hold nothing. i128 appears only as the intermediate of one cross-multiplication.
  - A whole-number fast path (`d == 1`): 77–99 % of refcore's results are whole numbers (NOTE §1 item 4).
  - A test asserts `size_of::<Num>()` is 16 bytes (R2 D2-1; an unknown until it compiles).
  - Invariants: reduced; d > 0; n ≠ i64::MIN; d ≤ i64::MAX; Small whenever the value fits. With these, derived `PartialEq` and `Hash` are correct.
  - Write `Ord` by hand, cross-multiplying in i128.
  - The Small path uses i128 intermediates and i64 `checked_*`. A `None` there means "widen", so the branch is reachable and mutation testing can kill it.
  - Demote to Small after every Big result that fits.
- **`Money`** is `{ minor: Int, cur: Currency }` with `Int = I64 | Big`, always the smallest tier that fits: a whole number of minor units (D2-2). Money plus money is one checked add, with no gcd.
  - _Changed 2026-10-30 (K8-3):_ phase 1's `I64 | I128 | Big` (A3 F-17) loses its middle tier. Every money value in the six measured checks was a whole number of minor units far inside i64 (whose limit is about 9.2·10¹⁶ in major units of a two-decimal currency), and anything larger, in any currency, goes to Big. A middle tier would only speed a range no measured check reaches, and one fewer tier is tested.
  - Methods are `checked_add`, `checked_sub`, `checked_neg` and `to_num`, plus `Num::round_to_money(step, mode, cur)`. Use `checked_*` names: inherent `add` and `sub` trip clippy's `should_implement_trait`.
  - No operator traits, `Default`, `Sum`, or `From<i64>`/`From<f64>` (R-28). With no `Mul<Num>`, the compiler refuses `money * rate`, and the only way back to money is a `round`, which is LANGUAGE §7.2.
  - Currency is checked at load. At run time a mismatch is an invariant error, never a coercion.
  - Prove the missing impls in unit tests: `static_assertions::assert_not_impl_any!`, or a hand-written trait-bound test.
- **Rational arithmetic:** Knuth TAOCP 4.5.1. One gcd of the denominators per addition; cross-reduction per multiplication.
  - For `gcd(big, small)`, reduce `big mod small` first and finish in u64. num-bigint's binary gcd is about 10× slower than PERFORMANCE §6 assumes without this (A3 F-3).
  - Write **Lehmer's gcd now** (Knuth TAOCP 4.5.2, Algorithm L) for balanced big values: about 150–250 lines, one function with its reference beside it (D2-1). Test it against `num_integer::Integer::gcd` and Python's `math.gcd`, on random and boundary values. num-bigint 0.4.8 stays (R-30's pin).
  - _Changed 2026-10-30 (K8-2):_ phase 1 said "only if the benchmarks ask for it". E1, a measured mortgage check, reaches 5 049 bits; GMP, CPython, malachite and dashu all use Lehmer at Tally's sizes, while num-bigint alone uses the slower binary gcd. GMP switches to its faster large-number algorithm only above 330 limbs (64-bit words) on arm64, and the cap is 213 limbs, so Lehmer is the most Tally ever needs.
  - Never call a signed primitive gcd at i64::MIN; the invariant above excludes it.
- **The cap** (`RUN-DIGITS`):
  - Check it after every operation, on the reduced result, by bit length, except inside a path statistic's own accumulation (below). ≤ 13,606 bits passes, ≥ 13,608 bits is refused, and only 13,607 bits needs a comparison with 10^4096.
  - **A path statistic's accumulation** (the sum under a `mean`, a variance) is uncapped: L §7.1 l.2103 (change 2.1-15) makes it the engine's, exact by any method, as long as it gives the printed rounding exactly. It is its own type, used only by path statistics, with its constructor private to the statistics module; its result is rounded and capped as a value (R-117).
  - _Changed 2026-10-30 (K8-5):_ phase 1 said "after every operation" with no exception; BUILDER predates 2.1. refcore caps a mixed-unit or rate `mean`'s partial sums in path order; the engine follows the spec, and gate 10 differs only past 4 096 digits (K8-17).
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
- **A quantile's standard error at size** (D3-7): refcore's exact Maritz–Jarrett method is out of reach at 10 000 paths, so the engine refines intervals until they settle the printed 1e-6 rounding, which L §7.1 allows; refcore's exact method is the oracle at small N. It is a numeric kernel on gate 8's list.

## 3. The parser, the tree and the tables (A2 §5.1–§5.4, A4 §5; R2 Q3–Q6)

- **Hand-written recursive descent**, one function per grammar rule, with operator chains as loops (precedence climbing or Pratt, as rust-analyzer and ruff do). No parser generator or combinator library: cedar's generated parser needs 8 blanket allows, and `LOAD-PARSE`'s message would follow the library's wording.
- **[Tally syntax]** Lex in two steps: split into lines, measure indentation (two spaces, no tabs; refuse tabs), then tokenise each line. Tally has no comments.
- **[Tally syntax]** Binary `-` has a space on each side, because a key may contain `-` (LANGUAGE l.71). A lone `-` is the "not applicable" atom when the next token can't start an operand (l.228). Both are lexer-context rules; test them first.
- **[Tally syntax]** At most one prefix `-` (l.198) and one `not`. Operator chains and postfix chains grow without limit and add no depth (l.233).
- **The tree:** an AST of owned enums with a span on each node, not a lossless CST, since Tally has one canonical format and no comments.
  - Spans are `usize` byte offsets in an own `Span { start, end }` with checked constructors. Avoid text-size and la-arena: both panic, and la-arena's indexing is invisible to `indexing_slicing`.
  - **[Tally syntax]** Keep chains flat: `Sum { first, rest: Vec<(AddOp, Expr)> }`, and likewise `Prod`, `And`, `Or` and the postfix chain. A left-nested chain of 100,000 terms is a tree 100,000 deep, and every recursive walk over it overflows the stack, including the derived `Drop`, `Clone`, `PartialEq` and `Debug`.
  - Items live in a table: an own `ItemId(usize)`, and a table exposing only `get(id) -> Result<&Item, Internal>`.
  - **One `Table<I, T>` for every id-keyed store** (D2-4), generalising the item table above, about 40 lines: `push`, `get` and `iter`, and no `Index` impl, so `table[id]` doesn't compile at all. Indexing becomes impossible, not just linted (R-5). `get(id)` returns `Result<&T, Internal>`. No arena or typed-index crate. It is the one expected own generic (R-124 reports it once).
  - **Names and text keys go through one hand-written interner** (D2-3), about 50 lines: a `Symbol(u32)` per distinct text. Its order is the text's (`Ord` compares `as_str()`, as rustc's `StableCompare for Symbol` does), so an id never decides output order (R-47). It pays for memo keys, which hold key values (things, keys of open indices, labels; seam F): a fixed-size symbol keeps a key small and cheap to compare. No interner crate: lasso and string-interner both need hashbrown (R-37). T's property test that reorders files guards the order.
  - _Changed 2026-10-30 (K8-4):_ phase 1 said "names are `Box<str>` plus a `BTreeMap<Box<str>, ItemId>`; no interner". The interner replaces that; the `BTreeMap` from text to id stays inside it.
  - Never let an id decide output order (R-47). Don't derive `Ord` for any order that reaches output; every derived order is listed for review (R-118).
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
- **Collections** follow §7's policy (D2-5): B-trees by default, sorted slices for frozen tables, dense vectors for day-keyed data, no hash map.
- **The walk's storage** (D2-6, D1-4). Checkpoints, taken monthly at first (the interval is an unknown until S), are `Vec<Rc<PlaceState>>` copied on write. `Rc` is a shared pointer with a reference count, like several variables holding one JavaScript object; copy on write means a place's state is cloned only when a day changes it. So a checkpoint costs O(places), and a day costs O(places it changes). Pins and scenarios are small layers over a base. A place that keeps parts holds them as a sorted label vector; one that doesn't holds a plain balance (the labels' cost is paid only where a kind opts in). A day column sits beside each log. No persistent-map crate: `im` is unmaintained and `imbl` pulls rand crates into what ships (R-41).
- **The memo** (D1-2, D1-3, D1-6, D2-7) is the engine's own, not the salsa crate (it depends on hashbrown and rustc-hash, R-37, and brings about 15 crates). Its key is (item, key values, an interned trimmed context). One memo per evaluation context (a scenario, a pin set, an axis combination, a member, a path, a projection), freed when that context's scope ends; no eviction, so peak memory is deterministic. A recurrence keyed by day or period is a dense column per (def, context). Read sets are built now; the rest of incrementality stays a seam (D1-5, PERFORMANCE §4).

## 4. The evaluator (A2 §5.4; R1 Q6, D1-1 to D1-7)

- **The shape** (D1-1): a tree-walk over a resolved tree, one exhaustive `match` per node kind, close to refcore's and the spec's. Closures come later, and only in a fence (R-130), where a measured ask misses its budget; the evaluator's form is worth about 1.5× at most, while the structure (the walk, the memo's key) is worth the order of magnitude (R1 §2.1). PERFORMANCE §1–§5 hold the design; this section holds the code shape.
- **Recursion to a counted depth, then an explicit work stack** (D1-1, K8-1). A def reading a def is a plain function call while a counter, passed down by value like `Depth` in §3, is below its limit. Past the limit, the demanded cell is pushed on an explicit stack (`enum Task { Demand(Cell), Finish(Cell) }` on a `Vec`, dependencies pushed in sorted order), and the outermost call replays the stacked cells, as IronCalc does (it counts to 64). An "in progress" set gives `RUN-CYCLE` for free.
  - The depth is counted, never measured (R-16), so a result is the same on every machine and every worker count. The limit and the thread stack size it needs are measured at S (NOTE §4: the deepest stack a case needs).
  - Before a replay, demand an aggregate's domain first, so an aggregate over many uncomputed keys doesn't turn quadratic (R1 Q6).
  - Recurrences are filled iteratively in key order, never by recursion on a stack (E §3.2 l.155).
  - Every thread, the 1-worker run's included, is spawned with an explicit stack size sized for the count (R-42, §7).
  - **Joint-model nesting is a worklist, never recursion** (R-16): a member that is itself a joint model pins its own members (E §3.1 l.118), and the spec gives that nesting no limit and no refusal code (SO8-8), so no counted depth can refuse it.
  - _Changed 2026-10-30 (K8-1):_ phase 1 said "an explicit work stack, not Rust recursion". Counted recursion reads like the spec and like refcore; the count makes it safe; the work stack stays as the fallback past the count.
- **Joint members** (D1-7, seam J): a member is its own context with its own memo; its walk runs lazily, up to the last day read. Pruning a member to the places read comes second.
- **Load-time cycle checks** (`LOAD-CYCLE`): iterative DFS or Kahn's algorithm, over nodes in sorted order.

## 5. Impossible states without `unreachable!` (A2 §5.5)

- **Narrow the type at the point of decision.** The token classifier returns `Option<AddOp>`, so the node builder matches `AddOp` exhaustively and never re-matches `TokenKind`. This removes cedar's most common exception ("`op` is checked by the caller").
- **Take, don't peek:** `let Some(first) = iter.next() else { return Err(…) };`.
- **Classify exhaustively** instead of `matches!`: `fn is_comparison(op: BinOp) -> bool { match op { BinOp::Eq | … => true, BinOp::Add | … => false } }`, so a new variant must be decided (R-63).
- **What remains** (a bad `ItemId`, a missing memo slot) returns an internal error. Which code it prints is a spec gap (SO-6).
- **Mutation:** an unreachable error branch is a mutant nobody can kill. In the gate-8 kernel, prefer types that make the branch not exist.

## 6. Errors and codes (A2 §5.6, A1 §6)

- One `enum Code`, one variant per row of LANGUAGE §8, ENGINE §11 and OPERATIONS §10 (87 codes). `const ALL: [Code; 87]`, `fn as_str(self) -> &'static str` and the message function, each an exhaustive `match`.
  - _Changed 2026-10-30 (K8-13):_ "87" is 1.0.1's count; 2.0 has 91 and 2.1 adds four (R-76). Recount at 8c with `tools/xref_check.py`; `ALL`'s length follows the recount, and the test below compares with the spec's tables either way.
- A unit test reads the spec's tables (read only) and compares them with `Code::ALL`, as rustc's tidy check does for its error codes.
- A refusal value carries code, message, file, line and item (LANGUAGE l.1582). The CLI's JSON shape is `{ code, message, … }` (engine-tests §4).
- No `anyhow`, `eyre`, `Box<dyn Error>` or `String` errors (R-58). `thiserror` is optional; a hand-written `Display` is fine. No rendering crate (miette, ariadne): the spec fixes the message text.

## 7. Determinism, I/O and output (A1 §6, A4 §5; R2 D2-5, R3 D3-1 to D3-6)

- **Collections** (D2-5, K8-6):
  - `BTreeMap`/`BTreeSet` (maps and sets kept in key order) by default;
  - sorted boxed slices (`Box<[T]>`, a fixed-size array) for tables frozen after load, searched by binary search;
  - dense `Vec`s for day- or position-keyed data: a recurrence's column per (def, context), logs;
  - `IndexMap` only where insertion order is the meaning, each use with a reason (R-37);
  - no small-vector crate, no collection or arena crate.
  - **No hash map, the memo's included.** _Changed 2026-10-30 (K8-6):_ session 8 weighed one sanctioned, never-iterated hash memo (R2 D2-5 b); G1 chose none now. The heavy misses are day-keyed, which a dense column serves. It is revisited only if the first engine profile shows sparse memo lookups dominating, and then as a rules change (R-112).
  - Sort with `sort_by_key` on a total key, never `sort_unstable`; a comparator that isn't a total order may crash since Rust 1.81.
- **Threads** (D3-1 to D3-6; R-42 as amended):
  - One module, the only one with threads, sanctioned by the lead (R-108). Everything else is single-threaded: one context is evaluated on one thread.
  - Every thread is started as `std::thread::Builder::new().stack_size(s).spawn_scoped(scope, f)` inside `std::thread::scope`, with `s` passed in. Never `Scope::spawn` or `thread::spawn`: a plain spawn reads the `RUST_MIN_STACK` environment variable (R-39) and gets 2 MiB against the main thread's 8 MiB, so a deep case could pass on 1 worker and crash on 6. _Changed 2026-10-30 (K8-8):_ phase 1 named `thread::scope` but not the stack.
  - The 1-worker run goes through the same module, on one spawned thread with the same stack.
  - What runs in parallel first (D3-2): Monte Carlo paths, and independent timelines (axis combinations, `windows each`, rolling origins, scenarios). Joint members and parallel asks come later.
  - Tasks are cut into contiguous blocks, one per worker. Each worker owns a `Vec` and fills its numbered slots; results travel only through `join`; one thread merges them in **task-index order** (D3-3), never in finishing order. In TypeScript terms: `Promise.all` writing `results[i]`, then one loop reducing in index order. _Changed 2026-10-30 (K8-8, K8-16):_ "path order" became task-index order, since a task is a path or a timeline; ENGINE §7 step 5's blocks are a means, the guarantee is the ordered merge (SO8-5).
  - The merge rule is refcore's (D3-3): per line, terms in order; the first refusing path wins, else the union of holes; quantiles sorted stably by (value, path). Which refusal prints is logged as SO8-3.
  - The shared layer (D3-4): run the first task alone, freeze the cells it shares, then run the rest. The frozen layer is handed out as `&` (a read-only borrow, with no interior mutability), which is what lets the compiler enforce the freeze. Count the cells a worker recomputes locally.
  - The worker count is an argument (D3-5): the CLI defaults to 6, a flag lowers it, and it is never read from the machine (no `available_parallelism`). No global pool, no rayon.
  - No channel, lock or atomic outside the module (R-42 d). While each worker owns its `Vec`, there is no interleaving to explore, so no schedule explorer is needed (shuttle only if an atomic ever enters the module, R3 Q7).
  - Gate 6's tests (D3-6): the worker sweep {1, 2, 3, 5, 6, 7}; a test-only switch that runs the blocks in reverse completion order and merges as usual; planted violations (a merge in finishing order, `Scope::spawn` without a stack size, a `HashMap` in a path layer, the refusal taken from the highest path); the deepest-expression case at 1 and 6 workers on the same explicit stack.
- **The library takes the reality as in-memory files** (path and bytes). It reads no clock, environment or file system.
- **The CLI's `main`**, in the sanctioned file:
  - reads `std::env::args_os()`, turning a non-UTF-8 argument into the usage-error exit code;
  - reads the reality's directory tree, collecting every file's relative path first and then sorting the full paths by code point, as E §3.1 l.118 orders them for the hash (R-40), and decodes UTF-8 strictly (R-49). _Changed 2026-10-30 (K8-12):_ phase 1 sorted each directory's entries by name bytes, which orders `a/b.tally` and `a.x.tally` the other way round and changes the hash;
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

| The workaround an LLM reaches for                              | The rule that answers it                   |
| -------------------------------------------------------------- | ------------------------------------------ |
| `unwrap_or_default()`, `unwrap_or(0)`, `.get(i).unwrap_or(&0)` | R-27 (ban plus report), review             |
| `saturating_*`, `wrapping_*`                                   | R-23                                       |
| a macro that hides `unwrap` or `panic!`                        | R-13                                       |
| `#[allow]`, `#![allow]`, `cfg_attr(…, allow(…))`               | R-103, R-107                               |
| an `#[expect]` on a whole `impl` or module                     | R-107; review (the counter shows its line) |
| floats for a division                                          | R-18 (no exception)                        |
| `a.add(b)` instead of `a + b`                                  | R-20                                       |
| `.ok()`, `let _ =`                                             | R-66                                       |
| `Box<dyn Error>`, `anyhow`                                     | R-58                                       |
| `cfg(test)` behaviour, reading answer keys                     | R-93                                       |
| `#[mutants::skip]`                                             | R-86                                       |
| turning a hook or lint off through config                      | R-79–R-82                                  |
| `clone()` storms to dodge the borrow checker                   | R-74 (report), review                      |

**The bans added on 2026-10-30** (G1 items 3, 4 and 6), one row each, so the skill and its eval (D6-4) can quote them. The library's and the parser's exception budgets are 0 (RULEBOOK §7.1), so for every row below an `#[expect]` is not a way out there.

| What blocks                                                                                                                     | The workaround it tempts                                                                                                                                                                                             | Write instead                                                                                                                                                                             | Stop and report when                                                                                                                            |
| ------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| no `dyn`: ast-grep error (R-122)                                                                                                | `Box<dyn Trait>` or `&dyn Trait` to share code between node kinds; a wrapper trait plus a generic `<T: Trait>` to get the same thing without the word `dyn` (R-124 reports it); an `ast-grep-ignore` comment (R-109) | an `enum` with one variant per case and an exhaustive `match` (R-62), each arm calling a named function (R-126)                                                                           | a design truly needs values of open-ended type: a STOP-REPORT, for a rules change (R-112)                                                       |
| no `impl Trait` arguments: clippy `impl_trait_in_params`, error (R-123)                                                         | `fn f(xs: impl Iterator<Item = Day>)`; the same as a named generic `fn f<I: Iterator<Item = Day>>(xs: I)` (R-124 reports it)                                                                                         | the concrete type: `&[Day]`, a `Table<I, T>`, or a named iterator struct                                                                                                                  | the only concrete type is unnameable (a closure's type): a STOP-REPORT                                                                          |
| no library macros outside the lead's list: ast-grep error (R-73)                                                                | a `macro_rules!` that shortens `checked_add(…).ok_or(…)?`, or stamps out repeated `match` arms                                                                                                                       | a newtype method (`day.next()?`) or a named helper in the kernel helper module (R-131); repeated arms stay written out, which R-62 wants anyway                                           | the same code would be written out over about 10 times with no helper shape: a STOP-REPORT asking for the lead's list                           |
| nesting deeper than 5: clippy `excessive_nesting`, error (R-74)                                                                 | `#[expect(clippy::excessive_nesting)]`; splitting a function into `step1`, `step2` mechanically; hiding a level inside a closure                                                                                     | an early return (`let Some(x) = … else { return Err(…) };`); conditions pushed up to the caller and loops down into a named step; a helper named by the spec's word for that step (R-121) | the lexer, the parser or the day walk can't meet 5 without an exception: a STOP-REPORT with the function, since the threshold is re-judged at S |
| no lock, channel or atomic outside the thread module: clippy.toml (R-42 d)                                                      | a `Mutex<Vec<…>>` that workers push into; an `mpsc` channel to collect results; an `AtomicUsize` to count cells across workers                                                                                       | each worker owns its `Vec` of numbered slots, results return through `join`, and one thread merges in task-index order (§7); per-worker counts are added up after the merge               | a design needs shared mutable state between workers (dynamic chunks, cancellation): a STOP-REPORT, since it would also need shuttle (R3 Q7)     |
| threads only through `Builder::spawn_scoped` with a stack size: clippy.toml bans `thread::spawn` and `Scope::spawn` (R-42 a, e) | `s.spawn(…)` inside `thread::scope`, which compiles and reads `RUST_MIN_STACK`; a stack size taken from an environment variable                                                                                      | `Builder::new().stack_size(s).spawn_scoped(scope, f)` with `s` passed in from the caller                                                                                                  | no stack size the count allows fits the deepest case: a STOP-REPORT with the case                                                               |
| no hash map, the memo's included: clippy.toml and cargo-deny (R-36, R-37; K8-6)                                                 | a `HashMap` or an FxHash map in the memo "for speed"; an `IndexMap` used as a hash map                                                                                                                               | a dense `Vec` column for day- or position-keyed cells; a `BTreeMap` for the rest (§7)                                                                                                     | a profile of the engine shows sparse memo lookups dominating a budget row: a STOP-REPORT with the profile (K8-6's revisit)                      |
| no operator-like extension traits or arithmetic macros: review (R-131)                                                          | `trait CheckedExt for u32 { fn inc(self) -> … }`, which reads like a std method                                                                                                                                      | a newtype for the domain value with checked methods, or a free helper function in the kernel helper module                                                                                | —                                                                                                                                               |
| optimised code only in fences: review, the trace (R-130)                                                                        | `#[inline(always)]`, a hand-unrolled loop or a cache added inside the evaluator "while I'm here"                                                                                                                     | plain code; a fence module with its `PERF:` header, benchmark, twin and equivalence test, once a budget row asks for it (§13)                                                             | a budget row misses and the fix would spread outside one module: a STOP-REPORT                                                                  |
| no derived order reaching output: review, ast-grep report (R-47, R-118)                                                         | `#[derive(PartialOrd, Ord)]` on a label, member or party type, then sorting output by it                                                                                                                             | an explicit sort key: the text order of stable ids, or the written `ties by` (R-47)                                                                                                       | —                                                                                                                                               |

## 9. The loop (A5 §5.1, §5.3; R4 Q5, D4-2)

| When                               | What runs                                                                                                                                                                                                                                                                                                                                    | Fails the turn?                                                      |
| ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| after each edit                    | rust-analyzer diagnostics (plugin)                                                                                                                                                                                                                                                                                                           | no: a speed aid                                                      |
| a group of T's vectors looks green | `engine/tools/check fast`: fmt, clippy, T's runner filtered to the milestone plus a fixed sample of earlier ones, the exception counter                                                                                                                                                                                                      | yes, through the Stop hook                                           |
| end of turn                        | the same, as a Stop hook                                                                                                                                                                                                                                                                                                                     | yes, unless a STOP-REPORT is newer than the last commit              |
| before each commit                 | `engine/tools/check full`: the whole T suite in dev and release, cargo-deny, check-frozen, check-rules, `exceptions --check`; advisory `cargo mutants --in-diff` on kernel changes                                                                                                                                                           | never commit red                                                     |
| every PR, Linux CI                 | `check full`, plus the budgets that count (D4-2): T's runner checks each vector's count budgets at 1 worker (the `counters` build, §13); gungraun benches at 1× with hard instruction limits from the budget table, passed on the command line by `engine/tools/perf`; instruction drift against the `engine` branch's last run, as a report | yes: a count or a hard limit over budget is red, like a failing test |
| nightly                            | the full kernel mutation run, advisory lints, coverage, long fuzz, benchmarks; on the reference Mac (never during a fuzz run), every budget row's wall time and peak footprint                                                                                                                                                               | reported in the morning                                              |
| milestone end                      | the fresh review (§12); the milestone report (R-129); the gate session's `engine/tools/perf milestone M<n>` on the reference Mac: wall time and peak footprint of the budget rows the milestone touches, at 1× size (BUILD-ORDER); then the re-run and tag                                                                                   | the gate session decides; a budget miss blocks the tag (D4-2)        |

- Run only through `engine/tools/check`, never raw `cargo` with your own flags. Read the one-line failures; open a log only when needed. Never raise a timeout or add a retry. If a failure doesn't reproduce, report it; don't "fix" it.
- **One session per milestone step**, started at the repo root as `TALLY_ROLE=build`:
  - start: the build role's SessionStart hook prints `engine/CLAUDE.md` (§11); invoke the skill `engine-rust` by name; read the last `engine/BUILD-LOG.md` entries, the milestone's BUILD-ORDER section and the plan's traceability table; run `check fast` and note the red vector ids;
  - work: next group of red vectors in plan order; implement; `check fast`; commit when green; on any stop trigger, a STOP-REPORT, then move on or end;
  - end: append to BUILD-LOG what turned green, what's next, open STOP-REPORTs, the exception delta, and the check's last line.
- **Plan each milestone** in plan mode: map each BUILD-ORDER item to its spec sections and T's vector ids. At S, propose the kernel module list; the lead writes it into the protected mutants config (R-85).
- **STOP-REPORT** (A5 §2.3): when a T test looks wrong, the spec is unclear, a rule blocks a correct design with the budget used up, a crate or kernel file is needed, a survivor isn't equivalent and T has no test for it, or `check fast` stays red on the same failure after three honest tries, or a budget looks wrong (a count or limit that a correct design can't meet; R4 Q5). Leave the code as it was. Record the item, the smallest input, what the engine gives and what the test expects, which of model, spec, test, rule or code is wrong and why, what was tried, and what was _not_ done. The prompt line: "If a test, the spec or a rule seems to conflict with correct behaviour, stop and file a STOP-REPORT. Never edit tests, never special-case inputs, never weaken a check. A STOP-REPORT is a good outcome."

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

## 11. `engine/CLAUDE.md` outline (A5 §5.2, corrected; R6 D6-2, D6-3)

At most 150 lines. Every hard rule appears twice, once as a sentence and once as the check that enforces it, which the sentence names.

**How it reaches the builder** (D6-2, D6-3; _changed 2026-10-30_):

- A nested `engine/CLAUDE.md` isn't loaded when a session starts at the repo root, nor after compaction, and only a skill's first 5 000 tokens survive compaction (R6 §1.2). So the build role's **SessionStart hook prints `engine/CLAUDE.md`** at the session's start and again after each compaction (`startup` and `compact`). The hook only prints; it fails open, since the walls are elsewhere.
- The skill **`engine-rust`** (`.claude/skills/engine-rust/`) is **invoked by name at each build session's start**. It carries the how (the write-this-instead tables, the references on numbers, layout, the evaluator, parallelism, performance and readability); `engine/CLAUDE.md` carries the never-miss rules; the lints, the guard and the Stop hook are the walls. A skill and a CLAUDE.md are advice, not enforcement.
- **Both are rule files:** the skill and `engine/CLAUDE.md` are in the guard's rule globs and in `RULES.sha256`, and only the `rules` role changes them, through R-112's procedure (K8-14), so the builder can't rewrite its own instructions by any route.

The outline:

1. What this is (3 lines): the spec in `engine/spec/` (frozen, read-only); T's tests in `engine/tests/` (read-only); you write `engine/crates/**`.
2. Hard rules, each with its enforcement: never edit tests, the spec or a rule file; never special-case an input; never add a crate; never weaken a check; a STOP-REPORT is a good outcome; the protocol in 10 lines.
3. The loop: §9's table, shortened; only `engine/tools/check`; what the Stop hook does; count budgets and instruction limits are red on every PR, wall time and footprint judged at the tag (§13).
4. Exceptions: `#[expect(clippy::x, reason = "R-n: …")]` only; the forbidden forms; the lock and the budget.
5. Write this instead (about 16 lines): an index becomes `.get(i).ok_or(…)?`, and a table is a `Table<I, T>` with `get`; `a + b` on integers becomes `checked_add(…).ok_or(…)?`, or a newtype method (`day.next()?`); `as` becomes `TryFrom`; `unwrap` becomes `?` or `ok_or`; `unwrap_or_default` becomes a hole; `/` and `%` become `checked_div_euclid`; `unreachable!` becomes a narrower type; `let _ =` becomes handling; `HashMap` becomes `BTreeMap` or a dense `Vec`; `dyn` becomes an `enum` and a `match`; an `impl Trait` argument becomes the concrete type; a macro becomes a helper function; nesting past 5 becomes an early return or a named step; clocks and env are never read.
6. Numbers: exact rationals only, through `Num` (Small or Big) and `Money` (`I64 | Big` minor units); overflow and the cap are refusals with their codes, except a path statistic's accumulator (R-117); **no floats anywhere**.
7. Holes and refusals: a missing value is a hole with its question, never 0; an error carries its catalogue code; an invariant break returns `ENGINE-INVARIANT`, never a panic.
8. Determinism: sorted maps; no time, env or threads except the sanctioned thread module, where every thread is spawned with a stack size passed in and results merge in task-index order; full relative paths sorted by code point (R-40).
9. Tests: T's vectors define done; your tests don't count; fixed seeds; no snapshots; no `cfg(test)` behaviour; reproduce before you fix.
10. Code shape: simple; enums with exhaustive `match`; no new trait without two implementations today; the `//!` header on every file (R-120); cite the spec section in each kernel function's doc comment, an error there (R-75); the subset (no `dyn`, no `impl Trait` arguments, no macros, generics reported; R-122 to R-125); recursion only to a counted depth (R-16); optimised code only in fences (R-130).
11. Hand-off: append to `engine/BUILD-LOG.md`; when compacting, keep the milestone, the red vector ids, open STOP-REPORTs and the files modified.
12. Commits: one green step each, naming the vector ids; never an AI co-author or attribution line.

## 12. The review checklist (A5 §5.4)

A fresh-context, read-only reviewer (Read, Grep, Glob, read-only Bash), given the diff range, the plan and the spec sections. It reports only gaps in correctness or the stated requirements, to `engine/reviews/<milestone>.md`. _Since 2026-10-30:_ it is the `engine-reviewer` subagent, which loads the skill `engine-rust` (D6-5), and item 9 below is the readability section (D5-5).

1. Traceability: each plan item maps to code and green vectors; each cited spec section says what the code does; nothing outside the milestone changed.
2. Passing by the wrong means: literal-scan hits; branches on names, paths or ask texts; reads of `engine/tests/`; `cfg(test)` or `debug_assertions` behaviour; output assembled from expected strings; guard-log entries outside the allow-list.
3. Workaround patterns (§8).
4. Exception reasons: about the domain, not about the tool.
5. Holes and refusals: every missing-input path ends in a hole or a catalogue code; no invariant break panics.
6. Determinism: no new hash-ordered collection, clock, environment read or thread.
7. Mutation: each new survivor's equivalence argument holds; others go to T.
8. Evidence: the builder's report quotes commands, exit codes and counts; the reviewer re-runs `check full` once.
9. Readability (D5-5, RULEBOOK group 8):
   - Headers: every new or changed `.rs` file starts with its `//!` header (purpose, `Spec:`, `Twin:`, `Invariants:`), and the purpose line still says what the file does (R-120).
   - The trace: each citation names the section that says what the code does, not a neighbouring one; the milestone's uncovered sections are expected ones (R-75, R-128).
   - The subset: no `dyn`; no unlisted generic, macro or trait; no `impl Trait` argument; lifetimes only on short-lived views; iterator chains of about 4 steps, single-expression closures (R-73, R-122 to R-125).
   - Nesting and shape: nothing deeper than 5; one call or expression per dispatch arm; no function split mechanically to meet the limit (`step1`, `step2`); no duplicated block over about 10 lines; files over 1 000 lines noted (R-74, R-126, R-127).
   - Fences: optimised code only in fence modules, each with its `PERF:` header, its benchmark, its plain twin and an equivalence test; no `#[inline(always)]` or `#[cold]` outside one (R-130).
   - Names: the spec's words, units last, only the listed abbreviations; the module layout matches `ARCHITECTURE.md`, and `ARCHITECTURE.md` is updated at the milestone (R-119, R-121).
   - Strictness: newtypes with checked methods and the kernel helper module, not extension traits or arithmetic macros (R-131).

## 13. Performance (R4; D4-1 to D4-8, D5-6; added 2026-10-30)

Performance is core, and it comes from the structure (§3, §4, §7; PERFORMANCE §1–§5), not from clever code: refcore steps every calendar day though only 2–7 % have work, and redoes the same long walk up to 64 times (NOTE §1 item 1). This section says how speed is kept readable, measured, guarded and profiled. The budgets are PERFORMANCE §8's rows (P8-1 onwards), every number **proposed until S**. Versions are the ones R4 checked on 2026-10-30; each tool is pinned and run only through `engine/tools/` (R-82).

### 13.1 Fences (D5-6, R-130)

- **Plain code first.** A fence module is opened only when a budget row asks for it, by its P8 id.
- A fence has:
  - a `PERF:` line in its `//!` header: what it speeds up, the P8 row that asked, and the name of its plain twin;
  - a benchmark (written by the lead on request, R-91);
  - a plain twin: the simple version of the same code, kept beside it;
  - an equivalence test: twin and fence agree on generated input (proptest with a fixed seed), and the whole run still prints refcore's bytes.
- A twin is kept where it is cheap; refcore is the twin of last resort.
- The fences expected from `tmp/session8/shape.md`: the walk (skipping empty days, checkpoints), the memo, the read index, the number tiers, the threads. Closures for day-dense asks (D1-1) would be one more, only if a measured ask misses.
- `#[inline(always)]` and `#[cold]` (hints that steer the compiler's inlining) belong inside fences.

### 13.2 Counters and `--stats` (D4-8, K8-15)

- A cargo feature `counters` (a compile-time switch, like a build flag that strips code from a bundle), **off in release**, like R-51's fault injection. When on, `--stats` prints `shape.md`'s units on stderr: expression nodes evaluated (ev), def cells (hits and misses), reads and distinct reads, walk days and non-empty days, legs booked, parts groups, `Num` results by bit size, walks shared (a timeline's prefix taken from a checkpoint, PERFORMANCE §2.5), and the cells a worker recomputed locally (D3-4).
- **Outputs stay byte-identical** with the feature on and off (gate 12). The counted build differs from the timed one; that is fine, since counts are deterministic.
- Counts are exact at 1 worker. With a shared layer, two workers may compute one cell, so 6-worker rows check only the result hash (R4 Q5, inference).
- The spec defines no stats output (R4 §5 item 7). `--stats` is a build-side instrument and never part of the shipped CLI; putting it there would be a spec addition (DONE rule 1).

### 13.3 The instruments and what blocks (D4-1, D4-2, D4-3, D4-7)

Wall time on hosted CI is like a flaky Jest timing test: about 45 % false alarms at a 2 % threshold (CodSpeed 2025). Counts give the same number every run. So only counts block a PR, and wall time blocks only the milestone tag, on one fixed machine.

| Instrument                                                                                                            | What it measures                                                   | Where                                             | When                                                                                                                                               | Blocks?                                                                                               |
| --------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| the engine's counts (§13.2), checked by T's runner                                                                    | `shape.md`'s units against each vector's count budget              | Linux CI                                          | every PR, at 1 worker                                                                                                                              | yes                                                                                                   |
| **gungraun 0.20.0** (the renamed iai-callgrind) under **Valgrind 3.28.0**, with `gungraun-runner` at the same version | instructions, cache hits and allocations (its DHAT mode) per bench | Linux CI only: Valgrind has no Apple Silicon port | every PR, with hard limits from the budget table passed on the command line by `engine/tools/perf`, so a bench file's own limits can't loosen them | yes (a hard limit missed exits with code 3); drift against the `engine` branch's last run is a report |
| `engine/tools/perf` (the lead's, stdlib Python, like `run.py`), running the release binary under `/usr/bin/time -l`   | wall time, peak memory footprint, instructions retired             | the reference Mac                                 | nightly as a report; at each milestone tag, by the gate session                                                                                    | at the tag                                                                                            |
| **criterion 0.8.2**                                                                                                   | wall time of the micro rows (P8-M*)                                | the reference Mac                                 | with the micro rows; their instruction limits run under gungraun on Linux                                                                          | no (it reports; it can't fail a run)                                                                  |

- **The reference machine** (D4-3, settling SO-27): the user's MacBook Pro M2 Pro (`Mac14,9`, 32 GB, macOS 26), as the self-hosted runner phase 1 chose for fuzzing (R-92). On AC power, not in Low Power Mode, with no other heavy job and never during a fuzz run; the pinned toolchain and the frozen release profile; 1 worker unless the row says 6; 2 warm-ups, then the median of 10 runs, and a row passes when its median is within budget. Every report records `sysctl hw.model`, `sw_vers`, `rustc -V` and `pmset -g`. A new Mac means a re-measure and the user's re-approval.
- **Memory** (D4-7): 8 GB peak footprint is a hard ceiling on any run; each load row is budgeted at ≤ 4 GB (6 heavy jobs fit the 32 GB machine); each 1× check at ≤ 512 MB. Mac rows are judged by peak footprint, Linux by max RSS.
- **Where things live** (R4 Q5): the budgets in PERFORMANCE §8 (the `spec` role, with the user's approval); the same rows machine-readable in a protected budget table that session 8c builds from the P8 rows (plan B2.3; its path is 8c's), which T's runner reads for the count budgets; the benches in `crates/tally/benches/*` and the perf runner (the lead, R-91); instruction baselines as CI artifacts, never local files; milestone reports in `engine/perf/M<n>.md` (the gate session). The builder edits none of them; the guard refuses.
- **A miss** is one line, e.g. `BUDGET E1/tx2_mtg_day wall(1w): median 1.42 s > 1.00 s (+42 %, 10 runs)` or `COUNT E1/tx2_mtg_day def_miss: 701 203 > 619 000`. It is red in `check`, so the Stop hook holds the turn (R-83); the builder's legal exit is a STOP-REPORT (§9, "a budget looks wrong").
- **A fast wrong answer fails:** every benchmark checks its output hash against T's expected digest after timing.
- **A gap to close when the benches are written** (R4 §5 item 4): `check` clears `CARGO_PROFILE_*` and similar variables but not `GUNGRAUN_*` or `CRITERION_*`, so an environment variable could loosen a limit (R-82).

### 13.4 The profiles (D4-4)

- **release** stays the scaffold's: `panic = "abort"`, `overflow-checks = true`, `lto = "fat"`, `codegen-units = 1`, `debug = "line-tables-only"` (uv ships the same, plus overflow checks). **bench** inherits it, with overflow checks on. Benches ignore `panic = "abort"` (Cargo's test harness needs unwinding), so library benches measure with unwinding, and the end-to-end rows time the release binary itself.
- **Overflow checks** are measured once at S: the lead builds twice (checks on and off) and reports the difference on the six checks (P8-10 to P8-15). If num-bigint's inner loops cost over 3 %, the user decides on an override for num-bigint only (`[profile.release.package.num-bigint] overflow-checks = false`), since it changes R-8's reach.
- **profiling**: release plus full debug info, for the builder's profiling only. Gates always measure `release`.
- **PGO** (profile-guided optimisation: the compiler tuning itself from a recorded run, like a JIT's warm-up done ahead of time) only at M9, as a release-only step; cargo-pgo 0.3.0 wraps it. Every budget must be met without it, and it is never trained on the gate's own inputs. No BOLT (ELF binaries only, so not on the Mac), no `target-cpu=native`. Allocators (mimalloc, jemalloc) are an M9 measurement, not a decision now.
- The profiles live in protected files; `check-rules` holds the manifest's profile sections equal to a frozen copy (R-80).

### 13.5 Profiling recipes (D4-5)

The engine's counters come first: most slowness here is more work, not slower work.

1. **Is it slower?** `engine/tools/perf row <id>`: one line per budget, with the counts.
2. **Why more work?** The `counters` build with `--stats`; diff the counts against the row's count budget and against refcore's counted run (`shape.md`'s units).
3. **Where does the time go?** `engine/tools/perf profile <id>`:
   - on the Mac, **samply 0.13.1**, `samply record --save-only -o prof.json.gz -- <bin> <args>` (no sudo, no Xcode, no browser), then the lead's summariser, which turns its Firefox Profiler JSON into text: the top 30 functions by self and inclusive time;
   - on Linux, `perf record -e cpu-clock -g -- <bin>` then `perf report --stdio --no-children`, or gungraun's callgrind output through `callgrind_annotate`.

None of these needs sudo, a GUI or Xcode. Not used: cargo-flamegraph 0.6.14, which needs Xcode's `xctrace` on macOS, and Instruments (Xcode, many gigabytes, the user's go). samply's last release is from 2025-02, and whether it records correctly on macOS 26 is an unknown until S.
