# Hindcast fixtures (design only)

**What this is for.** The evaluation layer (v2 §5.6, bounds row 37) compares a model's forecast with what really happened and sorts the difference into typed residual lines. No published answer key exists for fitting a household model, so the test has to be built: a synthetic ledger made by a known model, with causes planted in it. The test passes when each planted cause lands in its own residual line.

Programming analogy: this is mutation testing for the *diagnosis*, not the code. You plant a known bug and check that the debugger points at it.

Nothing here is built. These are specifications for fixtures the engine's test suite would hold (`engine-cases`, v2 §6).

## 1. Roles

| Role | Writes | Sees | Never sees |
|---|---|---|---|
| **Generator author** (a separate agent or person, engine-blind) | the true model M\*, its seed, the planted causes and their answer key | this file | the engine's code, the modeller's work |
| **Modeller** (Claude in the fitting loop, v2 §3) | the candidate model M, its estimators, one-offs marked as facts | the synthetic ledger up to K only | M\*, the answer key, rows after K |
| **Engine** | replay, ex-post, ex-ante runs; the residual ledger; scores | M, the ledger, the eval spec (S, K, E, R) | the answer key |
| **Judge** (a script, after the run) | pass or fail per planted cause | the answer key and the residual ledger | — |

The answer key is held outside the modeller's reach (holdout custody, v2 §5.6). It is opened only for frozen model hashes.

## 2. The base fixture

- **Household:** 2 adults, 1 joint account, 1 brokerage holding, 1 rent contract, 1 employment each, spending in 6 categories.
- **Ledger:** 60 months of classified rows (§6.1 import contract: date, amount, account, counterparty id, category id, source row id, record time).
- **Generators in M\*:** pay and rent as fixed indexed lines; groceries as a compound-Poisson drain with a per-path rate; pharmacy as intermittent (rate + size); December as one Fourier pair; the holding's price as a random walk with stated drift and volatility.
- **Clocks:** S = month 0, K = month 48, E = month 60. R is set per statement by the generator.
- **Law:** one simple law template both sides share (a flat payroll tax with a stated rounding), so a planted law fault has something to break.

## 3. Planted causes

Each fixture plants one cause on top of the base, plus a null fixture with none. The answer key gives, per cause: the residual line type it must land in, the date window, the sign, and the size.

| # | Planted in M\* (not in M) | Must land in | Exact or banded |
|---|---|---|---|
| H0 | nothing (null) | market and spending lines only; no law, timing or missed-occurrence lines | banded: coverage inside the binomial range |
| H1 | a 5% raise for adult A from month 30 | "amount" on A's pay, from month 30; once fitted, a dated step in A's job | exact amount per month |
| H2 | a one-off car repair in month 41 | "unmodelled actual" in month 41; once marked a fact, it feeds the rare-cost rate for its category | exact |
| H3 | the shared law's rounding changed from half-up to truncate in M only (a law bug) | "law" on every affected month, replay − actual equal to the planted difference to the grosz; never averaged into noise | exact |
| H4 | rent paid on the 3rd instead of the 1st from month 20 | "timing" on rent from month 20, amount residual 0 | exact dates |
| H5 | a subscription cancelled in month 45 | "missed occurrence" from month 46 | exact |
| H6 | the grocery purchase rate up 20% from month 36 | "frequency" on groceries, not "severity" | banded |
| H7 | the grocery purchase size up 20% from month 36 | "severity" on groceries, not "frequency" | banded |
| H8 | a planted cause dated after K (a raise in month 52) | nothing in the fitted parameters: the fitted model's hash is identical with and without it; the raise shows only in (K, E] as "amount" | exact (hash) |
| H9 | two causes at once (H1 + H4) | each in its own line, as when planted alone | exact |

## 4. Rules the fixtures also check

- **No reads past K:** H8 is a metamorphic test. Changing rows after K must not change the fitted model (T-16, v2 §5.6).
- **One-offs can't be abused:** a run where the modeller marks every residual a one-off must score worse on H6/H7 (the rare-cost rate rises), per v2 §5.6.
- **Invalid evaluations are refused:** MAPE on a balance series that crosses zero; a band from too few paths; any read of the answer key before the model hash is frozen.
- **The three-way split:** replay − actual is 0 on every fixture except H3; ex-post − replay holds H1, H2, H4, H5; ex-ante − ex-post holds the market noise.

## 5. What passing means, and what it doesn't

**Passing shows:** the attribution machinery separates causes it was told about, on data shaped like a household's, without peeking past K.

**Passing doesn't show:**

- that a real household's model fits (spec P-17: agreement with synthetic imports isn't evidence);
- that the residual types are the right ones for real causes no fixture planted;
- that the calibration thresholds are right. v2 §7.2 calls them "a proposal, not a sourced standard", and that stands. The anchors CH-01 (Basel zones) and CH-03 (CRPS) check the score arithmetic, not the thresholds.

**Statistical power is low** (v2 §5.6: over 12 months a band covering 65% instead of 80% is caught 21% of the time). H0, H6 and H7 therefore pool walk-forward origins at months 24, 36 and 48, as v2 proposes, and each banded verdict states its own power.

## 6. Open points

- The matcher (bounds row 15) is still weak, and every line type above depends on linking a row to an occurrence. The fixtures assume the matcher works; a matcher fault would show as H4/H5 landing in "unexplained".
- The generator author must be a different agent from whoever writes the engine's eval code, and from the modeller (L-33 style blindness).
