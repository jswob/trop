# LH-17-mc expected figures

Model: `montecarlo/LH-17-mc.tally`. It is LH-17's setting (`models/LH-17.tally`, `expected/LH-17.md`) with its randomness drawn. It is the answer key for DONE gate 7, part 1: "LH-17 run with draws matches its closed form within the oracle tolerance (4·SE plus the step bias plus the oracle's own error, ENGINE §8)".

**Load scan.** `python3 -I tools/loadcheck.py -v montecarlo/LH-17-mc.tally`, from the spec's root, gives `0 refusals (ids 101, keys 0, row keys 0, bound variables 36, rule labels 8)`, exit 0.

**Method.** Nothing here is simulated. Every figure is a closed form, evaluated by one of seven short scripts (Python standard library, `Decimal` or `Fraction` where exactness matters). Each script is shown in full with its output in § Scripts. Everything else is one-line arithmetic, shown where it is used.

## What the model is

**The setting.** Milevsky and Robinson's continuous-time model, which their Eq. 8 approximates:
- wealth w = 20 per $1 a year of real spending, spent as a continuous flow;
- X_t = (μ − σ²/2)t + σW_t, with μ = 0.05 and σ = 0.12;
- the remaining lifetime T ~ Exp(λ), with λ = 0.05, independent of W.

Wealth is W_t = e^{X_t}(w − ∫_0^t e^{−X_s} ds). By Itô this is dW = μW dt + σW dB − dt. So ruin before death is exactly the event that the SPV, S = ∫_0^T e^{−X_t} dt, is at least w. The partial integral grows in t, so there is no barrier to monitor between steps.

**Its figures are LH-17's own, with LH-17's locators:**

| Row | Value | LH-17 row | Locator |
|---|---|---|---|
| `lam` | 0.05 | `bb_lam` | p. 93, after Eq. 4c |
| `mu` | 0.05 | `mu[balanced]` | Table 3 note |
| `sigma` | 0.12 | `sigma[balanced]` | Table 3 note |
| `spend` | 1 | `spend_amt[w20]` | p. 94 |
| `wealth` | 20 | `wealth[w20]` | p. 94 |
| `s25` | 25 | `bb_s1` | p. 93 |

**Why this cell.**
- **λ = μ** makes the beta factor of Yor's law uniform (a = 1 below), so the exact ruin probability is two gamma CDFs that the language computes.
- **μw = 1** puts the CDFs' argument at their shape (z = b), where the difference is one gamma density. That gives a second, independent in-language form.
- Both figures are published, not chosen.

**The scheme.** Steps of Δ = 1/9 year, index `step = 0..2700` (300 years). Path p draws:
- its lifetime `life[p] ~ exponential(lam)`;
- per step k, `y[p, k] ~ normal(mdt, sdt)`, the log return, with mdt = (μ − σ²/2)Δ = 107/22500 and sdt = σ√Δ = 1/25 (exact, since √Δ = `rdt` = 1/3 is a row);
- per step k, `zeta[p, k] ~ normal(0, 1)`. Then `ybar = y/2 + bsd·zeta` is the step's average log level, with bsd = √(σ²Δ/12), a `sqrt … to 1e-15` def;
- one more normal, `zp[p]`, for the average over the partial last step.

The SPV is `spv[p] = acc[p, ksum[p]] + tailterm[p]`:
- `acc` sums `term[p, k] = Δ·exp(−(X_k + ybar_k))` over the K = ⌊T/Δ⌋ full steps (at most 2700);
- `tailterm` is the partial step's δ·exp(−(X_K + ȳ_p)), with δ = T − KΔ and ȳ_p = (μ − σ²/2)δ/2 + √(σ²δ/3)·zp;
- `ruined[p]` is `spend · spv[p] >= wealth`.

Only the K + 1 steps a path reaches are evaluated, because `acc` is a recurrence read at `ksum[p]`.

**No parameter draw.** The closed form takes μ and σ as known, so a parameter draw would make the target a mixture with no closed form. CHG-13's test mode (`with theta[*] = theta_mean`) therefore has nothing to pin here (finding MC1-5). The register's third verdict test, a pin on a draw, is exercised by `q_ruin_pinned`, which pins one path's uniform.

## The oracle

### Derivation

1. **Time change.** Put t = 4s/σ². Then σW_t =d 2B_s, and −X_t =d 2(B_s + νs) with ν = 1 − 2μ/σ². In s-time the lifetime is T' = σ²T/4 ~ Exp(λ'), λ' = 4λ/σ², and S = (4/σ²)·A_{T'}, where A_t = ∫_0^t e^{2(B_s + νs)} ds.
2. **Yor (1992)** (source `yor1992`). For T' ~ Exp(λ') independent of B, A_{T'} =d Z_{1,a} / (2γ_b), with:
   - Z_{1,a} ~ Beta(1, a) and γ_b ~ Gamma(b, 1), independent;
   - μ_Y = √(2λ' + ν²), a = (μ_Y + ν)/2, b = (μ_Y − ν)/2.

   The locator wasn't checked against the paper in this session. The proof in § The ruin equation doesn't rest on it.
3. **Its moments agree with the exponential-functional formula** E[A_{T'}^k] = k! / Π_{j=1..k}(λ' − 2j² − 2νj). Since λ' = 2ab and b − a = −ν, each factor is λ' − 2j² − 2νj = 2(a + j)(b − j). That gives k!·Γ(a+1)Γ(b−k) / (2^k Γ(a+k+1) Γ(b)), the moments of Z/(2γ). As λ' → 0 (with ν < 0), a → 0 and b → 2μ/σ² − 1, and the law becomes Dufresne's 1/S ~ Gamma(2μ/σ² − 1, scale σ²/2). That is Eq. 8 at λ = 0.
4. **At λ = μ.** 2λ' + ν² = 8μ/σ² + 1 − 4μ/σ² + 4μ²/σ⁴ = (1 + 2μ/σ²)². So μ_Y = 1 + 2μ/σ², which gives **a = 1** and **b = 2μ/σ² = 125/18**. Z_{1,1} is uniform, and S = 2U/(σ²γ_b).
5. **The ruin probability.** With c = σ²w/2 = 0.144 and z = 1/c = 2/(σ²w):

   P(S ≥ w) = P(U ≥ cγ_b) = E[(1 − cγ_b)+] = F_b(z) − c·E[γ_b; γ_b ≤ z] = **F_b(z) − μw·F_{b+1}(z)**

   Here F_k is the Gamma(k, 1) CDF, and E[γ_b; γ_b ≤ z] = b·F_{b+1}(z), with cb = μw.
6. **At w = 20.** μw = 1 and z = b. The identity F_b(x) − F_{b+1}(x) = x^b e^{−x}/Γ(b+1) then gives p* = b^b e^{−b} / Γ(b+1), the Gamma(b+1, 1) density at b. Stirling: p* ≈ 1/√(2πb) = 0.151.
7. **The density of S** is f_S(s) = μ·F_{b+1}(2/(σ²s)) (from ψ' below). It is decreasing in s, at most μ, and w·f_S(w) = μw·F_{b+1}(b) = 0.400906. The step bias uses it.

In the model: `p_oracle` is form 5 (rule `base v yor1992_v`), `p_oracle_pdf` is form 6, and `chk_lam_mu` and `chk_mw` assert the two preconditions.

### The ruin equation (the proof that doesn't rest on the citation)

Let ψ(w) = F_b(z) − μw·F_{b+1}(z), with z = 2/(σ²w) and dz/dw = −z/w. Write f_k for the Gamma(k, 1) density, and use f_{b+1}(z) = z·f_b(z)/b.
- **ψ'(w) = −μF_{b+1}(z).** The f_b terms cancel: −(z/w)f_b + μz·f_{b+1} = f_b·(−z/w + μz²/b), and μz²/b = σ²z²/2 = z/w.
- **ψ''(w) = μ(z/w)·f_{b+1}(z).** So (σ²w²/2)ψ'' = μ(σ²wz/2)·f_{b+1} = μf_{b+1}(z), since σ²wz/2 = 1.
- **The equation holds.** (σ²w²/2)ψ'' + (μw − 1)ψ' − λψ = μf_{b+1} − μ(μw − 1)F_{b+1} − μF_b + μ²wF_{b+1} = μ(f_{b+1} + F_{b+1} − F_b) = 0, using λ = μ and F_b − F_{b+1} = f_{b+1}.
- **The boundary values hold.** ψ(0+) = 1 and ψ(∞) = 0. The equation's bounded solutions near ∞ form one dimension (the Euler roots at ∞ are w^1 and w^{−b}), so ψ is the only bounded solution with ψ(0+) = 1.
- **ψ is the ruin probability.** e^{−λt}ψ(W_t) is a bounded martingale up to ruin (the generator gives Lψ − λψ = 0). At ruin it is e^{−λτ}. Without ruin, W_t → ∞ (the drift 0.0428 is positive) and it tends to 0. So ψ(w) = E[e^{−λτ}] = P(τ < T).

Script A2 checks this numerically at w = 5, 20 and 60:
- ψ' + μF_{b+1} = 0 and the equation's residual is 0, both to 18 decimals (central differences, h = 1e−12, 60 digits);
- ψ(0.001) = 0.99995 and ψ(10⁴) = 3.5e−18.

### Values (scripts A, A2, E)

| Figure | Value |
|---|---|
| b = z | 125/18 = 6.94444… |
| F_b(z) | 0.550489809375652488332721153800… |
| F_{b+1}(z) | 0.400906423547225256998136773291… |
| **p\*** = F_b(z) − F_{b+1}(z) | **0.149583385828427231334584380509** |
| p\* as the density form | agrees to 6e−59 |
| w·f_S(w) | 0.400906423547225257 |
| Eq. 8 at this cell: α = 589/161, β = 0.0322 | 0.105697756271381007, which is **0.0439 below p\*** |
| E[S] = 1/(λ + μ − σ²) | 1250/107 = 11.6822429906542 |
| E[S²] = 2/((λ + μ − σ²)(λ + 2μ − 3σ²)) | 6250000/28569 = 218.768595330603 |
| E[S³], E[S⁴] | 5777.34, 218012.8 (moments exist below order b = 6.94) |

**In-language values** (`q_oracle`, script E):

| Ask | Value |
|---|---|
| `p_oracle` | 0.149583385829 ≈, bound 1.0e-12 |
| `p_oracle_pdf` | 0.149583385828 ≈, bound 5.0e-13 |
| `eq8` | 0.105697756271 ≈ |
| `eq8 - p_oracle` | −0.043885629558 ≈ |
| `spv_mean_c` | 1250/107, plain |
| `spv_mean_d` | display 11.6806793010923 ≈ |
| `spv_mean_c - spv_mean_d` | display 0.00156368956189805 ≈ |

`chk_oracle_key` prints `ok`: |0.149583385829 − 0.149583385828427231| = 5.73e−13 ≤ 1e−12.

### Cross-checks against the published figures (`expected/LH-17.md`)

- **`t3_endow`, all nine cells.** These are Eq. 8 at λ = 0, which is the exact Dufresne member of the same family. Script A2 computes 100·F_{b−1}(2k/(100σ²)) for k = 2..10: 6.6508 | 24.9189 | 48.9968 | 70.0073 | 84.2773 | 92.4729 | 96.6482 | 98.5925 | 99.4369. At 1 dp half-up that is 6.7 | 24.9 | 49.0 | 70.0 | 84.3 | 92.5 | 96.6 | 98.6 | 99.4, the published row (`expected/LH-17.md` l.81), 9 of 9. This validates the gamma-CDF code on published numbers.
- **`t3_geo`.** `gdrift` = μ − σ²/2 = 0.0428 (l.89, derivation 5).
- **The lifetime figures.** They are the lifetime draw's own law (l.32–35, derivation 2): `surv_25` e^{−1.25} = 0.2865047969, `surv_40` e^{−2} = 0.1353352832, `e_life` 20, `med_life` ln 2/0.05 = 13.8629436112.
- **`spv_mean`'s formula.** l.58 uses 1/(μ − σ² + λ), which is the first moment of the exact law: E[S] = (2/σ²)·E[U]·E[1/γ_b] = (1/σ²)/(b − 1) = 1/(2μ − σ²).

### The oracle's own error

- **The model's closed form is exact**, with no model-form error. The law is proven by the ruin equation, and the identity of forms 5 and 6 is exact algebra.
- **The in-language value** `p_oracle` has two CDFs, each correctly rounded to 1e-12 (±5e−13), with coefficient μw = 1. So |p_oracle − p\*| ≤ 1e−12, and in fact it is 5.73e−13. That is the row `err_ruin` = 1e-12.
- **p\* is certain to 30 decimals,** 0.149583385828427231334584380509:
  - the series for F stops when a term falls below 1e−55 of the sum, and its terms fall geometrically;
  - Stirling's series at z ≥ 80 with 8 Bernoulli terms leaves at most |B₁₈|/(18·17·80¹⁷) = 8e−34, and the check Γ(1/2)² = π gives −1.4e−33, consistent with that;
  - the arithmetic carries 60 digits;
  - the density form agrees to 6e−59. It checks the series and the identity, not lnΓ.

## The step bias

### Why it has a sign

On a step of length ℓ ≤ Δ, write the log level as X_k + Y_u, with Y_u = mu + σB_u (m = μ − σ²/2). Let Ȳ = (1/ℓ)∫Y and Z_u = Y_u − Ȳ. The step's true SPV is

I_k = ℓ·e^{−X_k − Ȳ}·(1 + R_k), where R_k = avg_u e^{−Z_u} − 1 ≥ 0, by Jensen (avg Z = 0).

The model adds ℓ·e^{−X_k − Ȳ}. Its pair (y, ȳ) has the exact joint law of (Y_ℓ, Ȳ):
- the means are mℓ and mℓ/2; Var Y_ℓ = σ²ℓ, Var Ȳ = σ²ℓ/3, Cov = σ²ℓ/2;
- so Ȳ = Y_ℓ/2 + ζ, with ζ ~ N(0, σ²ℓ/12) independent of Y_ℓ;
- the steps are independent of each other and of T.

The partial step needs only Ȳ ~ N(mδ/2, σ²δ/3).

Given the drawn pairs, a Brownian path with those values exists (its conditional law), so one probability space holds the draws and a true (W, T). On it, **S_d ≤ S on every path**. So the share is biased low only, and E[p̂] = p_L ≤ p\* up to the two-sided roundings below. A plain left Riemann sum, e^{−X_k}·Δ, has a first-order two-sided error per step and no such pathwise order. Its bias can't be bounded this way.

### The bound

Let M be the largest R_k on the path, and ρ > 0.
1. {S ≥ w} \ {S_d ≥ w} ⊆ {w ≤ S < w(1 + M)} ⊆ {w ≤ S < w(1 + ρ)} ∪ {M > ρ}.
2. Since f_S is decreasing, P(w ≤ S < w(1 + ρ)) ≤ ρ·w·f_S(w) = 0.400906·ρ.
3. **The gap on one step.** e^{−z} − 1 + z ≤ ½z²e^{|z|} (Lagrange remainder) and avg Z = 0, so R ≤ ½·e^{sup|Z|}·avg Z². By Minkowski, √(avg Z²) ≤ σ√Q + |m|ℓ/√12, with Q = avg_u (B_u − B̄)².
4. **The tail of Q.** The demeaned Brownian motion has covariance eigenfunctions cos(nπu/ℓ), with eigenvalues ℓ²/(n²π²). This was checked by integrating the covariance against cos(nπu) on [0, 1]: ∫ min(s,t)cos(nπt)dt = (cos(nπs) − 1)/(nπ)², and ∫(t²/2 − t)cos(nπt)dt = 1/(nπ)².
   - So Q = Σ_{n≥1} ℓ·ξ_n²/(n²π²), and E e^{θQ} = Π(1 − x²/(n²π²))^{−1/2} = √(x/sin x), with x = √(2θℓ) < π (Euler's product).
   - Chernoff: P(Q > κℓ) ≤ e^{−x²κ/2}·√(x/sin x).
5. **The tail of sup|Z|.** sup|Z| ≤ 2σ·sup|B| + |m|ℓ/2, and by reflection P(sup_{u≤ℓ}|B_u| > a) ≤ 4Φ̄(a/√ℓ). So P(sup|Z| > s) ≤ 4Φ̄((s − |m|ℓ/2)/(2σ√ℓ)).
6. **Per step.** P(R_k > ρ) ≤ P(Q > κℓ) + P(sup|Z| > s), with ρ = ½e^s·(σ√(κℓ) + |m|ℓ/√12)². Both bounds grow with ℓ, so ℓ = Δ covers the partial step too.
7. **Over the steps.** A path has min(K, 2700) + 1 ≤ K + 1 steps, independent of the R_k. So P(M > ρ) ≤ E[K + 1]·(…) = (…)/(1 − e^{−λΔ}) = 180.5·(…).
8. **The cut at 300 years** lowers S only when T > 300: e^{−15} = 3.06e−7.
9. **The roundings,** which are two-sided:
   - the exponent of a term is off by at most 2700·5e−13 (the y draws) + 2.6e−13 (the ybar draw, with |Φ⁻¹(u)| ≤ 9.2 on the 2⁻⁶⁴ grid) ≤ 1.36e−9, so a relative 1.4e−9;
   - each exp is off by ±5e−13, and Σℓ ≤ 300, so ±1.5e−10 absolute;
   - the 1e-9 lifetime rounding moves ruin only if the ruin time lies within 5e−10 of T: ≤ 2λ·5e−10 = 5e−11 per use (T is independent of the path, with density ≤ λ);
   - the uniform grid (k + ½)/2⁶⁴: the ruin event is monotone in each normal draw (rounding is monotone), and its lifetime section has at most 2701 pieces, so ≤ 6e−16 in all.

   Together: ≤ f_S(w)·(w·2.8e−9 + 3e−10) + 2e−10 + 6e−16 < 1e−9.

**Values** (script B2, at Δ = 1/9; optimum s = 0.434, κ = 3.44, x = 3.094):

| Term | Value |
|---|---|
| ρ | 4.406136e−3 |
| ρ·w·f_S(w) | 1.766448e−3 |
| 180.5·P(Q > κΔ) | 1.028659e−4 |
| 180.5·P(sup\|Z\| > s) | 2.469883e−5 |
| cut at 300 years | 3.06e−7 |
| roundings | < 1e−9 |
| **total, lower side** | **1.894320e−3 ≤ 0.0019** = `bias_ruin` |
| upper side (roundings only) | < 1e−9 |

So **p\* − 0.0019 ≤ p_L ≤ p\* + 1e−9**. A heuristic size of the true bias is σ²Δ/12·w·f ≈ 5.3e−5. The bound is about 36 times that, and it is rigorous.

**At other steps** (script B), with E[K] + 1 and the bound:

| Δ | E[K] + 1 | Bound |
|---|---|---|
| 1/4 | 80.5 | 4.67e−3 |
| 1/9 | 180.5 | 1.90e−3 |
| 1/16 | 320.5 | 1.03e−3 |
| 1/36 | 720.5 | 4.46e−4 |
| 1/100 | 2000.5 | 1.61e−4 |
| 1/365 | 7300.5 | 4.58e−5 |

### The other statistics, and one with bias 0

- **The lifetime statistics have no step.** The only bias is the 1e-9 rounding of the exponential quantile:
  - for P(T > 25), ≤ λe^{−1.25}·5e−10 = 7.2e−12, so `bias_surv` = 1e-11;
  - for the mean of T, ≤ 5e−10, so `bias_life` = 1e-9;
  - **the type-1 median** of N = 10 000 paths is the 5000th order statistic, whose mean is 20·(H₁₀₀₀₀ − H₅₀₀₀) = ln 2/λ − 0.001000 (script C). So `bias_median` = 0.0011.
- **The mean of the SPV: a design with bias 0.** Its oracle is the scheme's own exact mean, not the continuous one (script D). With g = E e^{−y}, q = e^{−λΔ}, h = E e^{−ȳ}, a = m/2 − σ²/6 + λ and J = ∫_0^Δ δe^{−aδ}dδ:

  E[S_d] = (Δhq + λJ)·(1 − (gq)^{2700}) / (1 − gq) = **11.680679301092120**

  - The full steps give Σ_{k<2700} Δh·g^k·q^{k+1}. The partial step gives Σ_k (gq)^k·λJ, since given K = k, (T − kΔ) has density λe^{−λ(kΔ+δ)} and E e^{−ȳ_p} = e^{−(m/2 − σ²/6)δ}.
  - It is the def `spv_mean_d`, whose own error (six exps to 1e-15) is ≤ 2e−12, so `err_spv` = 1e-11.
  - Its gap to the continuous E[S] = 1250/107 is the exact step bias of the mean, **0.0015636896**, which `q_oracle` prints. Only the roundings remain: ≤ 1.4e−9·11.7 + 1.5e−10 = 1.7e−8, so `bias_spv` = 1e-7.

## The acceptance rule

|stat − oracle| ≤ 4·SE + bias + oracle error. The SE is a def at the oracle, since the statistic's printed SE can't be read (finding MC1-1). N = 10 000 throughout.

| Check | Statistic | Oracle | SE | 4·SE | Bias | Oracle's error | Tolerance (`q_tol`) | Passes when |
|---|---|---|---|---|---|---|---|---|
| `chk_ruin` | `share(p in path: ruined[p])` | `p_oracle` 0.149583385829 | `se_ruin` 0.003566626 | 0.014266504 | 0.0019 | 1e-12 | **0.016166504001** | n ∈ [1335, 1657] |
| `chk_surv` | `share(p in path: life[p] > s25)` | `surv_oracle` 0.28650479686 | `se_surv` 0.004521281 | 0.018085124 | 1e-11 | 5e-13 | 0.0180851240105 | n ∈ [2685, 3045] |
| `chk_median` | `quantile(p in path: life[p], 0.5)` | `med_oracle` 13.8629436112 | `se_life` 0.2 = 1/(λ√N) | 0.8 | 0.0011 | 1e-11 | 0.80110000001 | [13.06184361119, 14.66404361121] |
| `chk_life_mean` | `mean(p in path: life[p])` | `e_life` 20 | `se_life` 0.2 | 0.8 | 1e-9 | 0 | 0.800000001 | [19.199999999, 20.800000001] |
| `chk_spv_mean` | `mean(p in path: spv[p])` | `spv_mean_d` 11.6806793010923 | `se_spv` 0.09073606 | 0.36294424 | 1e-7 | 1e-11 | 0.36294434001 | [11.31773496, 12.04362364] |

Notes on the SE column:
- `se_spv` is an upper bound. It uses Var S_d ≤ E[S²] − E[S_d]² = 82.330, since S_d ≤ S (script D), against Var S = 82.294.
- The median's SE is the asymptotic √(q(1 − q))/(f(ξ)√N) with f(ξ) = λ/2.

## N = 10 000, and why

**Decisive.** SE = 0.003567 and the tolerance is 0.01617. The check fails a correct engine with chance ≤ 2.4e−5 (§ That the check bites), and catches with chance ≥ 1 − 3e−5 any engine whose mean share is off by more than tolerance + 4·SE = 0.0305 (8.5 SE):
- a drift sign error is 93 SE off;
- unseeded paths are at least 42 SE off;
- even Eq. 8 as an oracle is 12.3 SE away from a correct engine.

**Within budget.**
- A path evaluates E[K] + 1 = 180.5 steps, each with 2 normal quantiles and 1 exp, plus the lifetime, `zp`, a sqrt and the partial exp: 3·179.5 + 4 = 542.5 transcendental evaluations at 0.3–1 µs (PERFORMANCE §5). With about 6 cells a step at the prototype's ≈ 250 ns per path-day, that is 0.21–0.59 ms a path.
- At N = 10 000 that is 2.1–5.9 worker-seconds: **0.35–0.98 s on 6 workers per path ask that reaches `spv`**, against the Monte Carlo budget of ≤ 1 s on 6 workers (PERFORMANCE §8).
- `q_ruin`, `chk_ruin` and `chk_spv_mean` are three such asks, since ENGINE §7 shares no path layer between asks. The rest touch only the lifetime or a few paths.
- One path replayed costs ≤ 0.6 ms on average, and 2700·3 µs = 8 ms at the 300-year cut, within ≤ 20 ms.

**Not 100 000.** 3.5–9.8 s per ask breaks the budget, for 4·SE = 0.0045. If the gate measures more than 1 s, Δ = 1/4 halves the cost (80.5 steps), with bias 0.0047 and tolerance 0.0190, and N stays.

**Not 1 000.** 4·SE = 0.045, so Eq. 8's gap (0.044) and any error under 0.09 pass.

## The `share` verdict (ENGINE §8)

**At N = 10 000, `q_ruin` is `sayable`.**
- Precision: 1600·p̃(1 − p̃) ≤ 400 < N + 4 = 10004 for every n. At n ≈ 1496, p̃ = 1498/10004 and 2·se_v = 0.007135 (script C).
- Carry: the term reads no balance and no flow. Its register's only `assumed` entry, `scheme:rdt`, is read at the law date of its readers, which is the ask's cut K, so c = K − K = 0 ≤ 2 years. The horizon has no day to read (finding MC1-4).
- No pin on a draw.
- `q_life`'s share (life > 25) and `q_ruin_head` (n ≈ 15 of 10 000: 1600·p̃(1 − p̃) = 2.7 ≤ 10004) are `sayable` for the same reasons.

**At N = 100, `q_ruin_100` is `not sayable`.**
- It passes precision only for n ≤ 5 or n ≥ 95: 1600·p̃(1 − p̃) ≤ 104. Under p_L ∈ [0.1477, 0.1496] that chance is 1.6e−3 (script C), so the predicted print is `not sayable: 2·se 0.072521 > 0.05 (15 of 100)` at n = 15.
- Whatever n the seed gives, the verdict and its figure follow from n by the rule. That is the exact assertion.

**Where it flips.** For a share near p\*, the first N that passes is **207** at s = p\* exactly (script C), and 208 at the nearest integer n (n = 31). Every n passes from N = 396 (1600/4 = 400 ≤ N + 4). For comparison, 0 of 10 gives 2·se 0.187044, the ENGINE §8 example, reproduced.

**`q_ruin_pinned` is `not sayable`.** At N = 1000 precision always passes, and the carry is 0, so the only reason is the third test: `pinned: u(life[1]) = 0.5` is a pin on a draw. Its reason has no specified text (finding MC1-6).

## What is asserted exactly

1. **A seeded run replays byte for byte.** The whole reality, run twice and on two machines, prints identical bytes for every ask (LANGUAGE §7.5).
2. **The worker count changes nothing.** 1 and 6 workers print identical bytes (ENGINE §7 step 5).
3. **One path replays alone.**
   - `q_path7` (paths 7) and `q_path7_wide` (paths 100) print the same `life[7]`, `spv[7]`, `ruined[7]`.
   - A one-path explanation of path 7 reproduces them.
   - `q_ruin_head`'s n of 10 000 equals `q_ruin_100`'s n of 100: the first 100 paths of a large run are the small run. The `p <= 100 and …` guard stops at the `and`, so it is cheap.
4. **Pinned paths are plain runs.** `chk_hand_u`, `chk_hand_u_seed7` and `chk_hand_v` print `ok`, with T = 0.25: K = ⌊2.25⌋ = 2 full steps and δ = 0.25 − 2/9 = 1/36 (script D).
   - **`hand_u`** pins the uniforms to 1/2. Then y = mdt rounded = 0.004755555556, and ζ = zp = 0 exactly. The terms are (1/9)·r(e^{−y/2}) = (1/9)·0.997625046896 and (1/9)·r(e^{−3y/2}) = (1/9)·0.992892048500, plus (1/36)·r(e^{−(2y + 107/180000)}) = (1/36)·0.989945334003. So **spv[1] = 8952013715587/36000000000000** (0.248667047655194).
   - The same pins under seed 7 give the same value, since every draw reached is pinned.
   - **`hand_v`** pins the normal draws to 0. Then **spv[1] = 2/9 + (1/36)·0.999405732203 = 8999405732203/36000000000000** (0.249983492561194).
5. **The closed forms print exactly** the values of § The oracle (`q_oracle`), and the tolerances of § The acceptance rule (`q_tol`).
   - `chk_lam_mu`, `chk_mw` and `chk_oracle_key` print `ok`.
   - The ≈ bounds of `p_oracle`, `p_oracle_pdf` and `eq8` are 1.0e-12, 5.0e-13 and 5.0e-13. Those of values that rest on draws have two readings (finding MC1-8).
6. **The printed standard error of each share** is √(s(1 − s)/N) correctly rounded to 1e-6, at the printed n.
7. **`q_distinct` prints `false`, `false`, `false`.** Two paths' lifetimes, two paths' SPVs, and one path's two steps differ. Each is equal by chance with probability below 1e−10.

## What is asserted statistically

- `chk_ruin`, `chk_surv`, `chk_median`, `chk_life_mean` and `chk_spv_mean` print `ok`. Equivalently, the statistics of `q_ruin` and `q_life` lie in the bands of § The acceptance rule.
- `q_ruin_100` prints `not sayable`, with chance 1 − 1.6e−3.

## That the check bites

**The chance that a correct engine fails** at a fixed seed, over seeds (script C; exact binomial, Poisson–gamma and order-statistic tails):

| Check | False-fail chance |
|---|---|
| `chk_ruin` | 6.1e−6 at p_L = p\*, 2.44e−5 at p_L = p\* − 0.0019, the worst end |
| `chk_surv` | 6.55e−5 |
| `chk_median` | 6.37e−5 |
| `chk_life_mean` | 6.44e−5 (the sum of the lifetimes is exactly gamma) |
| `chk_spv_mean` | about 6.3e−5 under the normal approximation, less since `se_spv` is an upper bound. The only rigorous bound is Markov on the fourth moment, 1.17 % (kurtosis 10.53, one-line arithmetic in § Scripts) |
| the five together | ≤ 2.2e−4 rigorous for the first four; ≈ 2.8e−4 with the fifth |

**What wrong engines give:**

| Wrong engine | Mean of the ruin share | Caught by |
|---|---|---|
| **Drift sign error** (the normal quantile adds −m) | μ' = σ² − μ = −0.0356 maps to Yor with a = 125/18, b = 1 (λ = σ² − μ'). p = ∫_0^{1/c}(1 − cg)^{a}e^{−g}dg = **0.481387**, 93 SE off (script C). Also E[S] = 1/(λ + μ' − σ²) = ∞ | `chk_ruin` certainly; `chk_spv_mean` (an infinite mean); exactly by `chk_hand_u`: 0.251308730859 against 0.248667047656 |
| **Lifetime one step off** | One step long: P(τ < T + Δ) ∈ [p\*, e^{λΔ}p\*], a shift ≤ +8.33e−4. One step short, or dropping the partial step: a shift ≥ −8.29e−4. That is 0.23 SE: **not caught statistically** | **exactly** by `chk_hand_u` and `chk_hand_v`: K = 3 with no partial gives 0.330966511122, K = 2 with the partial dropped 0.221168566155, K = 1 with a 5/36 partial 0.248666950312, all against 0.248667047656 |
| **Draws not re-seeded per path** (the path key missing from the address) | every path the same: n ∈ {0, N}, a share of 0 or 1, at least 42 SE off | `chk_ruin` and `chk_surv` certainly. `chk_median` and `chk_life_mean` with chance 0.96 and 0.97: the single lifetime lies within the tolerance with chance 0.040 and e^{−0.96} − e^{−1.04} = 0.029. Exactly by `q_distinct` (`life[1] = life[2]` true) |
| The step key missing from the address | every step the same return: the log level is linear in t with slope 0.0428 + 0.36z, a gross change | `chk_ruin`; exactly by `q_distinct` (`y[1, 0] = y[1, 1]` true) |
| Eq. 8 taken as the oracle (a wrong key, not engine) | a correct engine's 0.1477–0.1496 against 0.1057: off by ≥ 0.0420 | fails a correct engine with near certainty: the reason for MC1-3 |

## Asks and their expected prints

| Ask | Prints |
|---|---|
| `chk_hand_u`, `chk_hand_u_seed7`, `chk_hand_v`, `chk_lam_mu`, `chk_mw`, `chk_oracle_key` | `ok` (exact) |
| `chk_ruin`, `chk_surv`, `chk_median`, `chk_life_mean`, `chk_spv_mean` | `ok` (statistical, chances above) |
| `q_distinct` | `false`, `false`, `false` |
| `q_life` | the median in [13.0618, 14.6640], with no SE (a `quantile` over paths prints none); the mean in [19.2, 20.8], with its SE √(Σ(x − x̄)²/(N(N − 1))) correctly rounded to 1e-6 (MC1-2, LU-1); carry 0.00 years and horizon 0.00 years (MC1-4, LM-2); the share n of 10000 with n ∈ [2685, 3045], `sayable` |
| `q_oracle` | 0.149583385829 ≈ · 0.149583385828 ≈ · 0.105697756271 ≈ · −0.043885629558 ≈ · 1250/107 · 11.6806793010923 ≈ · 0.00156368956189805 ≈ |
| `q_path7`, `q_path7_wide` | the same three values, unknown in advance |
| `q_ruin` | n of 10000 with n ∈ [1335, 1657], its SE, `sayable`; the mean of `spv` in [11.3177, 12.0436] with its SE |
| `q_ruin_100` | n of 100, `not sayable: 2·se … > 0.05 (n of 100)` unless n ≤ 5 or n ≥ 95 |
| `q_ruin_head` | n of 10000, with n equal to `q_ruin_100`'s; `sayable` |
| `q_ruin_pinned` | n of 1000, `not sayable` with the test-mode reason only; `pinned: u(life[1]) = 0.5` |
| `q_tol` | 0.003566626 · 0.016166504001 · 0.004521281 · 0.0180851240105 · 0.2 · 0.80110000001 · 0.800000001 · 0.09073606 · 0.36294434001 |

## Findings

Paths are relative to the spec's root. Line numbers are tally 1.0's; each finding's place in tally 1.0.1 is in `CHANGES.md`.

| Id | File:line | What | What it blocks | Readings |
|---|---|---|---|---|
| MC1-1 | ENGINE.md:353; LANGUAGE.md:1047, 1595 | ENGINE §8 has "the statistic, its standard error, the oracle and the tolerance" each "a def or a row". But a path aggregate is ask-only (`LOAD-ASKONLY`), so the statistic can only be an ask term, and its printed SE is read by no expression. §8 also doesn't say whether the tolerance's SE is the statistic's printed one or the oracle's | Writing an oracle test as §8 says | (a) the statistic in the `check`; SE and tolerance as defs at the oracle (this model) (b) the printed SE becomes readable in the ask (c) §8 says "the statistic is an ask term" |
| MC1-2 | LANGUAGE.md:1049–1050; ENGINE.md:318 | `mean` and `quantile` over paths "print a standard error" with no formula. ENGINE §7 says SEs are "computed exactly", but a square root needs a `to` (only `share`'s is "correctly rounded to 1e-6") | The goldens of `q_ruin`'s mean line and `q_life`'s two SE lines. The checks don't read them | mean: √(Σ(x − x̄)²/(N(N − 1))) or /N², to 1e-6. Quantile: the asymptotic form with a density estimate, an order-statistic (binomial) interval, or none |
| MC1-3 | DONE.md:24; models/LH-17.tally:5; PERFORMANCE.md:356 | "LH-17 … matches its closed form": LH-17's closed form, Eq. 8, is exact only at λ = 0 (eventual ruin, out of reach for paths) and a moment match otherwise. At a reachable cell it is 0.1057 against the exact 0.1496 (script A), so a correct engine fails against it with near certainty (12.3 SE) | Gate 7 part 1 as written | (a) the exact law of Yor (1992) at λ = μ, as here (b) Eq. 8 with a widened tolerance (no longer a closed form of the drawn model) (c) a λ = 0 oracle with a finite-horizon bound |
| MC1-4 | ENGINE.md:362 | Carry and horizon of a term that reads no balance and no flow. c = "the latest law date at which a `held` or `assumed` entry … is read, minus K" gives 0 when every read is at K, but the horizon h = d − K has no d: the term reads no day | The verdict line's golden for `q_ruin`, `q_life`, `q_ruin_head`. The verdict is `sayable` under every reading | h = 0 and printed `horizon 0.00 years`; h omitted; h from the header's `end` |
| MC1-5 | LANGUAGE.md:1495 vs 175, 206 (and 167) | The test mode's syntax `with theta[*] = theta_mean` doesn't parse: a pin is `ref "=" expr`, a ref's keys are expressions, and `*` exists only in a version row's keys | CHG-13's test mode in any model. This one has no parameter draw (§ What the model is) | (a) `*` allowed in a pin's keys, meaning every key (b) a keyless pin of a keyed draw (c) a pin per path through `for` |
| MC1-6 | ENGINE.md:363–365 | The third test's reason, a pin on a draw, has no text. §8 gives the precision and carry texts only | Gate 7's "with its reasons" golden for `q_ruin_pinned` | e.g. `not sayable: test mode (pinned: u(life[1]) = 0.5)` |
| MC1-7 | LANGUAGE.md:1047, 768, 601 | An ask can't read its own N. `count` and `sum` over `path` are neither allowed nor refused: the ask-only list names `share`, `mean` and `quantile`, and §5.10 says only those exist over `path`, in asks. So the SE defs read a row `npaths` that matches `paths 10000` only by hand, and a mismatch gives a wrong tolerance in silence | Tying the SE to N | (a) `count(p in path)` allowed in asks, plus a `check` against the row (b) a built-in for N (c) the row, as here |
| MC1-8 | LANGUAGE.md:1485, 969 | §7.3's ≈ bound widens "each rounded value". A draw is a correctly rounded quantile (§5.14 step 4), and an exact cell-edge value (exp(0), Φ⁻¹(1/2) = 0, sqrt(10000)) is "detected exactly". Whether either is widened is unstated | The ≈ text of values on paths (`q_path7`) and of `se_life`. The checks compare exact values and don't read it | For `hand_u`'s spv: draws not widened gives 1.3e-13; widened gives 2.7e-13 |
| MC1-9 | LANGUAGE.md:504 | `start` and `end` are required when a model has draws, though no draw here is on a time index and nothing reads the run span. The model sets start = end = on | Nothing; it is noise in the header | required as written; required only for places, pays and time-keyed draws |

## Scripts

### A: the oracle, Eq. 8 and `t3_endow`

```python
from decimal import Decimal as D, getcontext
from fractions import Fraction as Fr
getcontext().prec = 60
def atan_inv(n):
    x = D(1) / n; t = s = x; k = 1
    while abs(t) > D(10) ** -58: t *= -x * x; k += 2; s += t / k
    return s
PI = 16 * atan_inv(5) - 4 * atan_inv(239)
BERN = [Fr(1, 6), Fr(-1, 30), Fr(1, 42), Fr(-1, 30), Fr(5, 66), Fr(-691, 2730), Fr(7, 6), Fr(-3617, 510)]
def lngamma(z):
    sh = D(0)
    while z < 80: sh -= z.ln(); z += 1
    s = (z - D('0.5')) * z.ln() - z + (2 * PI).ln() / 2
    for i, b in enumerate(BERN, 1): s += D(b.numerator) / D(b.denominator) / (2 * i * (2 * i - 1) * z ** (2 * i - 1))
    return s + sh
def P(a, x):
    t = s = 1 / a; n = 0
    while t > D(10) ** -55 * s: n += 1; t *= x / (a + n); s += t
    return (a * x.ln() - x - lngamma(a)).exp() * s
mu, sig, lam, w = D('0.05'), D('0.12'), D('0.05'), D(20)
b = 2 * mu / sig ** 2; z = 2 / (sig ** 2 * w)
p_star = P(b, z) - mu * w * P(b + 1, z)
p_pdf = (b * z.ln() - z - lngamma(b + 1)).exp()
al = (2 * mu + 4 * lam) / (sig ** 2 + lam) - 1; be = (sig ** 2 + lam) / 2
print('b =', b, ' z =', z, ' F_b(z) =', P(b, z), ' F_b+1(z) =', P(b + 1, z))
print('p* =', p_star); print('pdf form =', p_pdf)
print('density f_S(w) = mu F_b+1(z) =', mu * P(b + 1, z), ' w f =', w * mu * P(b + 1, z))
print('Eq. 8: alpha =', al, ' beta =', be, ' P =', P(al, 1 / (w * be)), ' minus p* =', P(al, 1 / (w * be)) - p_star)
print('t3_endow % =', [round(float(100 * P(b - 1, 2 * D(k) / (100 * sig ** 2))), 2) for k in range(2, 11)])
print('Gamma(1/2)^2 / pi - 1 =', (2 * lngamma(D('0.5'))).exp() / PI - 1)
```

```
b = 6.94444444444444444444444444444444444444444444444444444444444  z = 6.94444444444444444444444444444444444444444444444444444444444  F_b(z) = 0.550489809375652488332721153800250512227349684942906408856403  F_b+1(z) = 0.400906423547225256998136773291244525813252460613761230694959
p* = 0.149583385828427231334584380509005986414097224329145178161444
pdf form = 0.149583385828427231334584380509005986414097224329145178161387
density f_S(w) = mu F_b+1(z) = 0.0200453211773612628499068386645622262906626230306880615347480  w f = 0.400906423547225256998136773291244525813252460613761230694959
Eq. 8: alpha = 3.65838509316770186335403726708074534161490683229813664596273  beta = 0.0322  P = 0.105697756271381006665013403168753675536726841936992416886749  minus p* = -0.043885629557046224669570977340252310877370382392152761274695
t3_endow % = [6.65, 24.92, 49.0, 70.01, 84.28, 92.47, 96.65, 98.59, 99.44]
Gamma(1/2)^2 / pi - 1 = -1.433489926450567642057047618E-33
```

### A2: the ruin equation, and `t3_endow` to 4 dp

The first 19 lines (imports, `atan_inv`, `PI`, `BERN`, `lngamma`, `P`) are script A's, verbatim.

```python
from decimal import Decimal as D, getcontext
from fractions import Fraction as Fr
getcontext().prec = 60
def atan_inv(n):
    x = D(1) / n; t = s = x; k = 1
    while abs(t) > D(10) ** -58: t *= -x * x; k += 2; s += t / k
    return s
PI = 16 * atan_inv(5) - 4 * atan_inv(239)
BERN = [Fr(1, 6), Fr(-1, 30), Fr(1, 42), Fr(-1, 30), Fr(5, 66), Fr(-691, 2730), Fr(7, 6), Fr(-3617, 510)]
def lngamma(z):
    sh = D(0)
    while z < 80: sh -= z.ln(); z += 1
    s = (z - D('0.5')) * z.ln() - z + (2 * PI).ln() / 2
    for i, b in enumerate(BERN, 1): s += D(b.numerator) / D(b.denominator) / (2 * i * (2 * i - 1) * z ** (2 * i - 1))
    return s + sh
def P(a, x):
    t = s = 1 / a; n = 0
    while t > D(10) ** -55 * s: n += 1; t *= x / (a + n); s += t
    return (a * x.ln() - x - lngamma(a)).exp() * s
mu, sig, lam = D('0.05'), D('0.12'), D('0.05'); b = 2 * mu / sig ** 2
psi = lambda w: P(b, 2 / (sig ** 2 * w)) - mu * w * P(b + 1, 2 / (sig ** 2 * w))
for w in (D(5), D(20), D(60)):
    h = D(10) ** -12; f0, fp, fm = psi(w), psi(w + h), psi(w - h)
    d1, d2 = (fp - fm) / (2 * h), (fp - 2 * f0 + fm) / h ** 2
    print('w', w, ' psi', f0.quantize(D(10) ** -15), ' psi\' + mu F_b+1 =', (d1 + mu * P(b + 1, 2 / (sig ** 2 * w))).quantize(D(10) ** -18),
          ' ODE residual =', (sig ** 2 * w ** 2 / 2 * d2 + (mu * w - 1) * d1 - lam * f0).quantize(D(10) ** -18))
print('psi(1e-3) =', psi(D('0.001')).quantize(D(10) ** -15), ' psi(1e4) =', psi(D(10000)).quantize(D(10) ** -25))
print('t3_endow % =', [(100 * P(b - 1, 2 * D(k) / (100 * sig ** 2))).quantize(D('0.0001')) for k in range(2, 11)])
```

```
w 5  psi 0.750000028666421  psi' + mu F_b+1 = 0E-18  ODE residual = -0E-18
w 20  psi 0.149583385828427  psi' + mu F_b+1 = -0E-18  ODE residual = -0E-18
w 60  psi 0.001655092612944  psi' + mu F_b+1 = -0E-18  ODE residual = -0E-18
psi(1e-3) = 0.999950000000000  psi(1e4) = 3.4932051E-18
t3_endow % = [Decimal('6.6508'), Decimal('24.9189'), Decimal('48.9968'), Decimal('70.0073'), Decimal('84.2773'), Decimal('92.4729'), Decimal('96.6482'), Decimal('98.5925'), Decimal('99.4369')]
```

### B: the step-bias bound per step length

`wf` is w·f_S(w) from script A.

```python
import math
mu, sig, lam, w = 0.05, 0.12, 0.05, 20.0
m = mu - sig ** 2 / 2; wf = 0.400906423547225257   # w f_S(w) = mu w F_b+1(b), script A
def pq(kq):      # P(Q > kq*dt) <= min_x exp(-x^2 kq / 2) sqrt(x / sin x), 0 < x < pi
    return min(math.exp(-x * x * kq / 2) * math.sqrt(x / math.sin(x)) for x in [i / 1000 * math.pi for i in range(1, 1000)])
def bound(dt, s, kq):
    rho = 0.5 * math.exp(s) * (sig * math.sqrt(kq * dt) + m * dt / math.sqrt(12)) ** 2
    psup = 2 * math.erfc((s - m * dt / 2) / (2 * sig * math.sqrt(dt)) / math.sqrt(2))   # 4 Phibar(.)
    steps = 1 / (1 - math.exp(-lam * dt))                                               # E[K] + 1
    return rho * wf * (1 + 1e-6) + steps * (pq(kq) + psup), rho, steps, pq(kq), psup
def best(dt):
    return min((bound(dt, s / 1000, kq / 20) + (s / 1000, kq / 20)) for s in range(20, 1500, 10) for kq in range(10, 400, 5))
for dt in (1 / 4, 1 / 9, 1 / 16, 1 / 36, 1 / 100, 1 / 365):
    t, rho, steps, a, c, s, kq = best(dt)
    print('dt 1/%d: bound %.3e  rho %.3e  s %.3f  q/dt %.2f  steps %.1f  P(Q) %.2e  P(sup) %.2e  heuristic %.1e' % (round(1 / dt), t, rho, s, kq, steps, a, c, sig ** 2 * dt / 12 * wf))
print('exp(-lam*300) = %.3e' % math.exp(-lam * 300))
```

```
dt 1/4: bound 4.674e-03  rho 1.043e-02  s 0.600  q/dt 3.00  steps 80.5  P(Q) 4.66e-06  P(sup) 1.44e-06  heuristic 1.2e-04
dt 1/9: bound 1.899e-03  rho 4.464e-03  s 0.430  q/dt 3.50  steps 180.5  P(Q) 4.28e-07  P(sup) 1.81e-07  heuristic 5.3e-05
dt 1/16: bound 1.028e-03  rho 2.434e-03  s 0.340  q/dt 3.75  steps 320.5  P(Q) 1.29e-07  P(sup) 3.32e-08  heuristic 3.0e-05
dt 1/36: bound 4.458e-04  rho 1.035e-03  s 0.240  q/dt 4.00  steps 720.5  P(Q) 3.88e-08  P(sup) 4.32e-09  heuristic 1.3e-05
dt 1/100: bound 1.610e-04  rho 3.839e-04  s 0.160  q/dt 4.50  steps 2000.5  P(Q) 3.50e-09  P(sup) 5.56e-11  heuristic 4.8e-06
dt 1/365: bound 4.577e-05  rho 1.084e-04  s 0.090  q/dt 5.00  steps 7300.5  P(Q) 3.13e-10  P(sup) 1.62e-12  heuristic 1.3e-06
exp(-lam*300) = 3.059e-07
```

### B2: the bound at Δ = 1/9, refined

```python
import math
mu, sig, lam, w, dt = 0.05, 0.12, 0.05, 20.0, 1 / 9
m = mu - sig ** 2 / 2; wf = 0.400906423547225257
xs = [i / 10000 * math.pi for i in range(1, 10000)]
pq = lambda kq: min(math.exp(-x * x * kq / 2) * math.sqrt(x / math.sin(x)) for x in xs)
def bound(s, kq):
    rho = 0.5 * math.exp(s) * (sig * math.sqrt(kq * dt) + m * dt / math.sqrt(12)) ** 2
    psup = 2 * math.erfc((s - m * dt / 2) / (2 * sig * math.sqrt(dt)) / math.sqrt(2))
    steps = 1 / (1 - math.exp(-lam * dt))
    return rho * wf + steps * (pq(kq) + psup), rho, rho * wf, steps * pq(kq), steps * psup
best = min(bound(s / 1000, kq / 100) + (s / 1000, kq / 100) for s in range(400, 460, 2) for kq in range(320, 380, 2))
print('bound %.6e  rho %.6e  rho*w*f %.6e  steps*P(Q) %.6e  steps*P(sup) %.6e  s %.3f  q/dt %.2f' % best)
x = min(xs, key=lambda x: math.exp(-x * x * best[6] / 2) * math.sqrt(x / math.sin(x))); print('x* = %.4f' % x)
print('with truncation e^-15 and rounding 1e-9: %.6e' % (best[0] + math.exp(-15) + 1e-9))
```

```
bound 1.894013e-03  rho 4.406136e-03  rho*w*f 1.766448e-03  steps*P(Q) 1.028659e-04  steps*P(sup) 2.469883e-05  s 0.434  q/dt 3.44
x* = 3.0938
with truncation e^-15 and rounding 1e-9: 1.894320e-03
```

### C: false-fail chances, the verdict's arithmetic, wrong engines

```python
import math
lg = math.lgamma
def binom_cdf(k, n, p):     # P(X <= k), X ~ Bin(n, p), summed in log space
    return sum(math.exp(lg(n + 1) - lg(i + 1) - lg(n - i + 1) + i * math.log(p) + (n - i) * math.log1p(-p)) for i in range(0, k + 1))
def pois_cdf(k, x):         # P(Pois(x) <= k)
    return sum(math.exp(i * math.log(x) - x - lg(i + 1)) for i in range(0, k + 1))
N, ps, bias = 10000, 0.149583385828427, 0.0019
se = math.sqrt(ps * (1 - ps) / N); tol = 4 * se + bias + 1e-12
def ff_share(pl, po, t):    # P(|X/N - po| > t)
    lo, hi = math.ceil(N * (po - t)) - 1, math.floor(N * (po + t))
    return binom_cdf(lo, N, pl) + 1 - binom_cdf(hi, N, pl)
print('ruin: se %.7f 4se %.7f tol %.7f  ff at p* %.2e  at p*-bias %.2e' % (se, 4 * se, tol, ff_share(ps, ps, tol), ff_share(ps - bias, ps, tol)))
s25 = math.exp(-1.25); se2 = math.sqrt(s25 * (1 - s25) / N); tol2 = 4 * se2 + 1e-11 + 5e-13
print('surv25: se %.7f tol %.7f  ff %.2e' % (se2, tol2, ff_share(s25, s25, tol2)))
xi = 20 * math.log(2); hb = 20 * (math.log(2) - sum(1 / i for i in range(5001, 10001))); tm = 0.8 + 0.0011 + 1e-9
F = lambda x: 1 - math.exp(-0.05 * x)
print('median: E T(5000) - xi = %.6f  tol %.6f  ff %.2e' % (-hb, tm, binom_cdf(4999, N, F(xi + tm)) + 1 - binom_cdf(4999, N, F(xi - tm))))
tl = 0.8 + 1e-9
print('mean life: tol %.9f  ff %.2e' % (tl, pois_cdf(N - 1, N * (20 + tl) * 0.05) + 1 - pois_cdf(N - 1, N * (20 - tl) * 0.05)))
print('N=100: P(n<=5 or n>=95) = %.2e' % (binom_cdf(5, 100, ps) + 1 - binom_cdf(94, 100, ps)))
for n, NN in ((15, 100), (0, 10), (15, 196), (1496, 10000)):
    pt = (n + 2) / (NN + 4); print('  n %d of %d: 2 se_v = %.6f  1600 pt(1-pt) = %.3f vs N+4 = %d' % (n, NN, 2 * math.sqrt(pt * (1 - pt) / (NN + 4)), 1600 * pt * (1 - pt), NN + 4))
flip = min(NN for NN in range(1, 1000) if 1600 * ((ps * NN + 2) / (NN + 4)) * (1 - (ps * NN + 2) / (NN + 4)) <= NN + 4)
print('  at s = p*: first N that passes = %d' % flip)
c, a1 = 0.144, 125 / 18; nq = 200000; hq = (1 / c) / nq
fs = [(1 - c * i * hq) ** a1 * math.exp(-i * hq) for i in range(nq + 1)]
print('drift sign error: p = %.6f' % (hq / 3 * (fs[0] + fs[-1] + 4 * sum(fs[1:-1:2]) + 2 * sum(fs[2:-1:2]))))
print('lifetime one step long: shift <= %.3e, short: >= -%.3e' % ((math.exp(0.05 / 9) - 1) * ps, (1 - math.exp(-0.05 / 9)) * ps))
print('one path for all, |T1 - xi| <= tol_median: %.4f' % (math.exp(-0.05 * (xi - tm)) - math.exp(-0.05 * (xi + tm))))
```

```
ruin: se 0.0035666 4se 0.0142665 tol 0.0161665  ff at p* 6.09e-06  at p*-bias 2.44e-05
surv25: se 0.0045213 tol 0.0180851  ff 6.55e-05
median: E T(5000) - xi = -0.001000  tol 0.801100  ff 6.37e-05
mean life: tol 0.800000001  ff 6.44e-05
N=100: P(n<=5 or n>=95) = 1.61e-03
  n 15 of 100: 2 se_v = 0.072521  1600 pt(1-pt) = 218.787 vs N+4 = 104
  n 0 of 10: 2 se_v = 0.187044  1600 pt(1-pt) = 195.918 vs N+4 = 14
  n 15 of 196: 2 se_v = 0.039440  1600 pt(1-pt) = 124.440 vs N+4 = 200
  n 1496 of 10000: 2 se_v = 0.007135  1600 pt(1-pt) = 203.709 vs N+4 = 10004
  at s = p*: first N that passes = 207
drift sign error: p = 0.481387
lifetime one step long: shift <= 8.333e-04, short: >= -8.287e-04
one path for all, |T1 - xi| <= tol_median: 0.0401
```

The SPV's moments and the fourth-moment bound, one line:

`python3 -I -c "from fractions import Fraction as F; l=m=F(1,20); s2=F(9,625); d=[l+j*m-F(j*(j+1),2)*s2 for j in (1,2,3,4)]; E=[1,1/d[0],2/(d[0]*d[1]),6/(d[0]*d[1]*d[2]),24/(d[0]*d[1]*d[2]*d[3])]; v=E[2]-E[1]**2; m4=E[4]-4*E[1]*E[3]+6*E[2]*E[1]**2-3*E[1]**4; N=10000; print(float(E[3]), float(E[4]), float(v), float(m4), float(m4/v**2), float((m4/v**2+3*(N-1))/(256*N)))"` → `5777.339665420856 218012.81756305118 82.29379403791378 71305.96361639774 10.529110478297357 0.011721691058780586`

### D: the scheme's exact mean, and the pinned paths

```python
from decimal import Decimal as D, getcontext, ROUND_HALF_EVEN
from fractions import Fraction as F
getcontext().prec = 50
mu, sig, lam, dt, kmax = D('0.05'), D('0.12'), D('0.05'), D(1) / 9, 2700
m = mu - sig ** 2 / 2
g = (-m * dt + sig ** 2 * dt / 2).exp(); q = (-lam * dt).exp(); h = (-m * dt / 2 + sig ** 2 * dt / 6).exp()
a = m / 2 - sig ** 2 / 6 + lam; J = (1 - (-a * dt).exp() * (1 + a * dt)) / a ** 2
trunc = 1 - (-kmax * (lam + m - sig ** 2 / 2) * dt).exp()
e_sd = (dt * h * q + lam * J) * trunc / (1 - g * q)
e_s = 1 / (lam + mu - sig ** 2); e_s2 = 2 / ((lam + mu - sig ** 2) * (lam + 2 * mu - 3 * sig ** 2))
print('E[S] =', e_s, ' E[S_d] =', e_sd, ' gap =', e_s - e_sd)
print('E[S^2] =', e_s2, ' Var S =', e_s2 - e_s ** 2, ' bound on Var S_d =', e_s2 - e_sd ** 2)
print('SE bound at N = 1e4 =', ((e_s2 - e_sd ** 2) / 10000).sqrt(), ' E[K] + 1 =', 1 / (1 - q))
dec = lambda x: D(x.numerator) / D(x.denominator) if isinstance(x, F) else D(x)
r12 = lambda x: F(dec(x).quantize(D('1e-12'), rounding=ROUND_HALF_EVEN))
def path(y, k, delta, part):     # spv of a pinned path: k full steps (zeta = 0), then a partial step of length delta
    xs, s = F(0), F(0)
    for _ in range(k): s += F(1, 9) * r12((-(dec(xs) + dec(y) / 2)).exp()); xs += y
    return s + delta * r12((-(dec(xs) + part)).exp()) if delta else s
y = r12(m * dt); ypart = m * (D(1) / 36) / 2             # u = 1/2: y = the median, rounded; zeta = zp = 0; delta = 1/36
hu = path(y, 2, F(1, 36), ypart); hv = path(0, 2, F(1, 36), ypart)
print('y =', dec(y), ' hand_u =', hu, '=', dec(hu)); print('hand_v =', hv, '=', dec(hv))
print('wrong engines on hand_u: drift sign', dec(path(-y, 2, F(1, 36), ypart)), ' K = 3, no partial', dec(path(y, 3, 0, 0)),
      ' K = 2, partial dropped', dec(path(y, 2, 0, 0)), ' K = 1, partial 1/9 + 1/36', dec(path(y, 1, F(5, 36), m * D(5) / 72)))
```

```
E[S] = 11.682242990654205607476635514018691588785046728972  E[S_d] = 11.680679301092120150230050974098641327328555941292  gap = 0.001563689562085457246584539920050261456490787680
E[S^2] = 218.76859533060310126360740662956351289859638069236  Var S = 82.29379403791378341925418800784982611372433946605  bound on Var S_d = 82.33032639564120079860458690482733196936057331460
SE bound at N = 1e4 = 0.090736060304402240890775588179368619516982827454737  E[K] + 1 = 180.50046296272481346566388482852502494443297449316
y = 0.004755555556  hand_u = 8952013715587/36000000000000 = 0.24866704765519444444444444444444444444444444444444
hand_v = 8999405732203/36000000000000 = 0.24998349256119444444444444444444444444444444444444
wrong engines on hand_u: drift sign 0.25130873085933333333333333333333333333333333333333  K = 3, no partial 0.33096651112244444444444444444444444444444444444444  K = 2, partial dropped 0.22116856615511111111111111111111111111111111111111  K = 1, partial 1/9 + 1/36 0.24866695031205555555555555555555555555555555555556
```

### E: the in-language values, as the engine rounds them

The CDF and density values are script A's.

```python
from decimal import Decimal as D, getcontext, ROUND_HALF_EVEN
from fractions import Fraction as F
getcontext().prec = 50
dec = lambda x: D(x.numerator) / D(x.denominator) if isinstance(x, F) else x
r = lambda x, u: F(dec(x).quantize(D(u), rounding=ROUND_HALF_EVEN))      # correctly rounded, half-even
sq = lambda x, u: r(dec(x).sqrt(), u); ex = lambda x, u: r((-dec(x)).exp(), u)
Fb, Fb1 = D('0.550489809375652488332721153800'), D('0.400906423547225256998136773291')   # script A
pdf, e8 = D('0.149583385828427231334584380509'), D('0.105697756271381006665013403169')   # script A
lam, mu, sig, n, dt = F(1, 20), F(1, 20), F(3, 25), 10000, F(1, 9)
po = r(Fb, '1e-12') - mu * 20 * r(Fb1, '1e-12')
se_r = sq(po * (1 - po) / n, '1e-9'); su = ex(F(5, 4), '1e-12'); se_s = sq(su * (1 - su) / n, '1e-9')
print('p_oracle', dec(po), ' p_oracle_pdf', dec(r(pdf, '1e-12')), ' eq8', dec(r(e8, '1e-12')), ' eq8 - p_oracle', dec(r(e8, '1e-12') - po))
print('se_ruin', dec(se_r), ' tol_ruin', dec(4 * se_r + F('0.0019') + F('1e-12')))
print('surv_oracle', dec(su), ' se_surv', dec(se_s), ' tol_surv', dec(4 * se_s + F('1e-11') + F('5e-13')))
print('med_oracle', dec(r(D(2).ln(), '1e-12') / lam), ' tol_median', dec(4 * F(1, 5) + F('0.0011') + F('1e-11')), ' tol_life', dec(4 * F(1, 5) + F('1e-9')))
m = mu - sig ** 2 / 2
gq, q, h = ex((lam + m - sig ** 2 / 2) * dt, '1e-15'), ex(lam * dt, '1e-15'), ex(m * dt / 2 - sig ** 2 * dt / 6, '1e-15')
a = m / 2 - sig ** 2 / 6 + lam; J = (1 - ex(a * dt, '1e-15') * (1 + a * dt)) / a ** 2; tr = 1 - ex(2700 * (lam + m - sig ** 2 / 2) * dt, '1e-15')
sd = (dt * h * q + lam * J) * tr / (1 - gq); m2 = 2 / ((lam + mu - sig ** 2) * (lam + 2 * mu - 3 * sig ** 2)); sc = 1 / (lam + mu - sig ** 2)
se_v = sq((m2 - sd ** 2) / n, '1e-9')
print('spv_mean_d', dec(sd), ' spv_mean_c', sc, ' gap', dec(sc - sd))
print('spv_m2', m2, ' se_spv', dec(se_v), ' tol_spv', dec(4 * se_v + F('1e-7') + F('1e-11')))
print('bsd', dec(sq(sig ** 2 * dt / 12, '1e-15')), ' mdt', m * dt, ' sdt', sig / 3, ' shape_b', 2 * mu / sig ** 2, ' zarg', 2 / (sig ** 2 * 20))
print('eq8_alpha', (2 * mu + 4 * lam) / (sig ** 2 + lam) - 1, ' eq8_beta', (sig ** 2 + lam) / 2, ' kmax * dt', 2700 * dt)
```

```
p_oracle 0.149583385829  p_oracle_pdf 0.149583385828  eq8 0.105697756271  eq8 - p_oracle -0.043885629558
se_ruin 0.003566626  tol_ruin 0.016166504001
surv_oracle 0.28650479686  se_surv 0.004521281  tol_surv 0.0180851240105
med_oracle 13.8629436112  tol_median 0.80110000001  tol_life 0.800000001
spv_mean_d 11.680679301092307557607476982951277276990047687786  spv_mean_c 1250/107  gap 0.0015636895618980498691585310674143117949990411858803
spv_m2 6250000/28569  se_spv 0.09073606  tol_spv 0.36294434001
bsd 0.011547005383793  mdt 107/22500  sdt 1/25  shape_b 125/18  zarg 125/18
eq8_alpha 589/161  eq8_beta 161/5000  kmax * dt 300
```
