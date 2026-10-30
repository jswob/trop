# Numbers: `Num`, `Money`, rounding, the cap

Every number in Tally is exact (L §7.1). There is no floating point anywhere: not in Monte Carlo, not in a ratio, not in a test helper the engine calls (R-18, no exception). The full guidance is BUILDER §2 and §10; the rules are RULEBOOK group 2.

## Contents
1. The two types
2. Arithmetic and gcd
3. The cap, and the one place it doesn't apply
4. Rounding, integers in dates, errors
5. The transcendental kernel
6. Tests to write

## 1. The two types

**`Num`**: `enum Num { Small { n: i64, d: i64 }, Big(Box<BigRat>) }` (D2-1).
- Two tiers, no middle one: in the heaviest measured check, every result above 63 bits was also above 1 024 bits, so a 128-bit tier would hold nothing. `i128` appears only as the intermediate of one cross-multiplication.
- A whole-number fast path (`d == 1`): 77–99 % of results are whole.
- Invariants, always: reduced; `d > 0`; `n ≠ i64::MIN`; `d ≤ i64::MAX`; Small whenever the value fits. Demote to Small after every Big result that fits. With these invariants, derived `PartialEq` and `Hash` are correct; write `Ord` by hand, cross-multiplying in `i128`.
- A unit test asserts `size_of::<Num>()` is 16 bytes.

**`Money`**: `{ minor: Int, cur: Currency }` with `Int = I64 | Big`, a whole number of minor units, always the smallest tier that fits (D2-2). Money plus money is one checked add, no gcd.
- Methods: `checked_add`, `checked_sub`, `checked_neg`, `to_num`, and `Num::round_to_money(step, mode, cur)`. Use `checked_*` names; inherent `add`/`sub` trip clippy's `should_implement_trait`.
- **No operator traits, no `Default`, no `Sum`, no `From<i64>`** (R-28). With no `Mul<Num>` for money, `money * rate` doesn't compile, and the only way back to money is a `round` (L §7.2). Prove the missing impls in a unit test (`assert_not_impl_any!` or a trait-bound test).
- A currency mismatch is caught at load; at run time it is an invariant error, never a coercion.

Only the number kernel names the big-integer types (R-29); the number stack is pinned exactly (R-30: num-bigint 0.4.8, num-integer 0.1.47, num-traits 0.2.19, default features off).

## 2. Arithmetic and gcd

- Rational arithmetic per Knuth TAOCP 4.5.1: one gcd of the denominators per addition; cross-reduction per multiplication.
- The Small path uses `i128` intermediates and `i64::checked_*`. A `None` means "widen to Big": the branch is reachable, so mutation testing can kill it.
- `gcd(big, small)`: reduce `big mod small` first, then finish in `u64`.
- **Lehmer's gcd, written now** (Knuth 4.5.2, Algorithm L) for balanced big values, about 150–250 lines, one function with its reference beside it. Test it against `num_integer::Integer::gcd` and Python's `math.gcd`. (num-bigint uses the slower binary gcd; at Tally's sizes, below the cap's 213 limbs, Lehmer is the most it needs.)
- Never call a signed primitive gcd at `i64::MIN`; the invariant excludes it.
- No `Iterator::sum`/`product`: fold with checked additions (R-24). No `BigInt::pow` without the size pre-check (R-26). No float conversions (R-25).

**Bad / good**
```rust
// Bad: silent overflow, then a float
let ratio = (a + b) as f64 / c as f64;
// Good: exact, and an overflow is a refusal
let sum = a.checked_add(&b).ok_or(NumError::Digits)?;
let ratio = sum.checked_div(&c)?;            // Num, exact
```

## 3. The cap, and the one place it doesn't apply

- `RUN-DIGITS` (L §7.1): check after every operation, on the reduced result, by bit length. ≤ 13 606 bits passes; ≥ 13 608 bits is refused; only 13 607 bits needs a comparison with 10^4096 (a lazily built constant).
- Pre-check `pow`, shifts and literal exponents before computing.
- Until the spec answers SO-12, check intermediates in the expression's written order.
- **A path statistic's accumulation is uncapped** (L §7.1 l.2103, change 2.1-15): the sum under a `mean`, a variance. It is its own type, used only by path statistics, its constructor private to the statistics module, outside the per-operation cap; its result is rounded and capped as a value (R-117). refcore caps those partial sums; the engine follows the spec (K8-17).

## 4. Rounding, integers in dates, errors

- The 7 modes of L §7.2, written once in one module (R-31), on `div_floor`/`mod_floor` plus a tie test. Test every mode with negative values and ties.
- Integers in dates and indices: `checked_div_euclid` and `checked_rem_euclid` with a positive divisor, never `/` or `%` (R-22). Write the Python equivalent (`//`, `%`) in a comment beside each: Python floors where Rust truncates, and refcore is Python (gate 10). Use `is_multiple_of` and `TryFrom` at every boundary between widths (R-21).
- `NumError { Div, Digits, Undecided, Domain, Param, Invariant }`, mapped to the `RUN-*` codes at the key. Every big `/` and `%` goes through one helper that checks for zero.
- The largest-remainder split is one function, used by `prorata … rest largest`, by every arrival's split among owners and every take ending in `prorata`; ties only by the written `ties by`, else `RUN-TIE` (R-114).

## 5. The transcendental kernel (L §7.3)

No permissive crate has rigorous erf, erfc, Φ⁻¹ or the incomplete gamma, so it is written here:
1. Dyadic intervals (m·2^e, m a big integer), rounded outward, each function's error bound written beside its code with its reference.
2. Exact paths first: sqrt to a step by integer `isqrt` and comparison; small rational powers by integer roots.
3. Quantiles certified by cdf values at the cell edges: any guess, then check cdf((k − ½)u) < p < cdf((k + ½)u).
4. Only then a heap-free fixed-width first precision for draws, checked against the big path by a differential test.
5. Exact-edge cases detected before the ladder starts; the worst case measured (time to 2^16 bits, `RUN-UNDECIDED`).

A quantile's standard error at size (D3-7): refine intervals until they settle the printed 1e-6 rounding (L §7.1 allows any method); refcore's exact Maritz–Jarrett method is the oracle at small N. It is a numeric kernel on gate 8's list.

## 6. Tests to write (BUILDER §10)

- A differential against Python's `fractions.Fraction` on random and boundary values (i64 limits, 2^63, 2^64, 10^4096 ± 1), all 7 modes, ties and negatives, every PR.
- proptest with a fixed seed: the algebraic laws; Small and Big agree on every input both can hold; `round` idempotent.
- Fixtures at the cap: 13 606, 13 607 and 13 608 bits; 10^4096 − 1 passes, 10^4096 is refused; `pow` refused before it is computed.
- Kernel benchmarks (written by the lead on request, R-91) at 20, 1 100, 1 400, 2 000 and 4 096 digits, balanced and big × small. Micro rows P8-M1..P8-M3 (PERFORMANCE §8.4).
