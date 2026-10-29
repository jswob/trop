# Household bands: the answer key (DONE gate 7, part 2)

Gate 7, part 2: "The household's bands give their predicted statistics within the same tolerance". The tolerance is 4·SE plus the step bias plus the oracle's own error (ENGINE §8). This file is the key. Every predicted value below is a closed form; no path is drawn, by script or by hand.

Files:
- `montecarlo/household-bands/scenarios/surprises.tally`: the draws, the pays and 35 asks.
- `montecarlo/household-bands/scenarios/costly_repairs.tally`: a scenario `over surprises`, for the paired (common random numbers) statistics.
- `montecarlo/household-bands/asks/bands_oracle.tally`: a reality part holding the oracle values and tolerances as rows, which the `check` asks read.
- `montecarlo/household-bands-oracle/fans.py` and `shares.py`: the closed forms (28 and 27 lines). They evaluate formulas and bisect a CDF; neither draws.
- The load scan (§2.2) runs on a directory that holds `amendments/household/` and `household-bands/` together. It is made by copying and isn't shipped.

K = `on` = 2026-10-19 throughout. Everything the asks read lies in 2026–2027.

---

## 1. The addition

**What the forecaster writes.** "What if, in 2027, the month's spending wobbles around the learned amount, and now and then an unexpected repair comes (car, washing machine, dentist)? How low can konto go, how many futures in 1 000 fall below zero, and what is August's room?" That is a scenario: it changes nothing before 2027-01-01, and it can be compared with the base.

**The draws** (`surprises.tally:29–35`):
```
def repair_hit[m in surprise_month]: dist at day_of(m, surprise_dom)
  note "czy w miesiącu m przychodzi niespodziewana naprawa (1) czy nie (0)"
  rule base = binomial(1, repair_chance)

def spend_dev[m in surprise_month]: dist PLN at day_of(m, surprise_dom)
  note "odchylenie wydatków miesiąca m od kwoty wyuczonej"
  rule base = normal(0 PLN, spend_sd) to minor(PLN)
```

In tally 1.0 the binomial draw carried `to 1`, since the grammar asked a `to` of every draw (MC2-10). tally 1.0.1 gives a discrete draw none (V1-P7), so it is written without one. Nothing else in this key changes.
with `repair_chance = 0.1`, `repair_cost = 3000`, `spend_sd = 400`, all `assumed` (`surprises.tally:129–133`). `surprise_month` is 2027-01..2027-12, so no instance is dated before `from`.

**The pays** (`surprises.tally:37–59`), all on the 25th of month m, `for m`:
- `repair`: konto → sklepy, `repair_cost`, when `repair_hit[p, m] = 1`;
- `spend_over`: konto → sklepy, `round(spend_dev[p, m], 0.01, half-up)`, when the deviation is > 0;
- `spend_under`: sklepy → konto, `round(-spend_dev[p, m], 0.01, half-up)`, when it is < 0.

Two pays carry the deviation because a negative amount is refused (`RUN-NEG`, L:888). The `round` is a no-op on a value already on the 0.01 grid, and it makes the amount money (L:292).

**Plan values.** Both draws have `plan 0`, the median. That is "the household's own planning figure, such as the median spending" (L:974). So every look-ahead (`plan_low`, `next_on`) sees no surprise instance (`repair_hit = 1` is false at 0, and so are both deviation tests). The rule of the 15th therefore plans exactly as in the base. A surprise already booked before the walk day is on the path with its drawn value (L:974), so it lowers konto from the 25th to the next 15th, and the next room absorbs it.

**Defs** (`surprises.tally:21–31`):
- `invested[m]` is the month's room: the sum over wrappers of `flows(invest[w, m], m)`;
- `net_worth_on[d]` is the core `net_worth` expression as a def (`asks/core.tally:13`);
- `year_low[p in path]` is the lowest end-of-day konto in 2027.

**The asks** (household questions first):

| Ask | Asks | N |
|---|---|---|
| `band_konto` | konto quantile fan 0.05/0.25/0.5/0.75/0.95 at 2027-06-30 and 2027-12-31 | 1000 |
| `band_net_worth` | the same fan of net worth | 1000 |
| `band_room` | the fan of the month's room (`invested`) for 2027-08 (the tight month) and 2027-11 | 1000 |
| `below_zero` | "N in 1 000 futures konto falls below 0 on some day of 2027" | 1000 |
| `below_two_rents` | "N in 1 000 futures konto ends 2027 below two rents" | 1000 |
| `mean_net_worth` | the mean net worth at 2027-12-31 | 1000 |
| `paired_cost` (costly_repairs) | what a 6 000 zł repair instead of 3 000 zł costs by 2027-12-31: median, mean, share of futures it costs nothing | 1000 |
| `base_values` | the base net worth and konto, no paths | — |
| `test_mode` | the fans' q05 and the zero-dip share with both draws pinned | 1000 |
| `verdict_51`, `verdict_52` | the "0 of N" verdict edge | 51, 52 |
| `replay_one`, `replay_path` | determinism (§6) | 1, 1000 |
| `check_*` (28 asks) | the acceptance rules (§4) | as above |

All path asks use `seed 7` and say `with scenario …` explicitly (MC2-11).

**Why the thresholds are a rule and not a row.** `2 * rent` is read at the ask's law date K (L:405), where `rent` = 3200 (`facts/2026.tally:26`, prices 2026 with `inflation = 0` through 2027, `facts/2026.tally:14`), so the threshold is 6400 under any law date in 2026–2027. A threshold or an oracle written as a scenario row would be a hole: the row governs 2027-01-01.. and the ask reads it at K (MC2-9). That is why the oracles sit in a reality part.

---

## 2. The base

### 2.1 The household's 2027 windows (HH4-b §10, `smoke/HH4-b.md:570–584`; HH4-a agrees)

The rule of the 15th: room = max(0, plan_low(konto, 15th, 14th of the next month) − 1000). plan_low subtracts P = 800 (`marek_last`, promised and unconfirmed, HH4-b ambiguity A(a)) and the VAT hold. So after each invest, konto + the window minimum = 1800.

| Month | Room R | konto after invest | Window minimum (day) |
|---|---|---|---|
| Jan | 11410.06 | 12135.16 | −10335.16 (Feb 9) |
| Feb | 9835.50 | 13520.33 | −11720.33 (Mar 1) |
| Mar | 13685.82 | 11719.79 | −9919.79 (Apr 11) |
| Apr | 12253.87 | 11391.28 | −9591.28 (May 9) |
| May | 10606.19 | 12823.74 | −11023.74 (Jun 9) |
| Jun | 11464.51 | 11944.38 | −10144.38 (Jul 11) |
| Jul | 11889.02 | 11771.49 | −9971.49 (Aug 9) |
| Aug | 4409.93 | 1800.00 | 0 (Aug 15 itself) |
| Sep | 18791.75 | 11829.09 | −10029.09 (Oct 10) |
| Oct | 9494.21 | 14051.01 | −12251.01 (Nov 1) |
| Nov | 13938.05 | 11713.80 | −9913.80 (Dec 9) |
| Dec | 11531.30 | 11898.63 | −10098.63 (2028-01-09) |

- Base konto: 2027-06-30 = 6760.19 and 2027-12-31 = 6656.75 (HH4-b ambiguity A(a)); `vat_owed` is 0 on both.
- The lowest konto after each month's 25th, within 2027:
  - ten months (Jan–Jul, Sep–Nov) have their window minimum after the 25th, so the lowest is 1800;
  - Aug's minimum is Aug 15 itself, before the 25th; after Aug 25 the lowest is Sep 9, at 1800 + 7213.95 = 9013.95;
  - Dec's window minimum falls in 2028; within 2027 the lowest is Dec 29, at 11898.63 − 8701.30 = 3197.33 (checked day by day: it reaches 6656.75 on Dec 31).

### 2.2 Load scan

```
$ python3 -I tools/loadcheck.py -v <scan>
loadcheck (D8 W3): a line scan of ids and keywords; no expression is parsed and no figure computed.
arguments: 1 reality, 16 files: 0 refusals (ids 323, keys 49, row keys 183, bound variables 307, rule labels 62)
  <scan>: 16 files, 0 refusals (ids 323, keys 49, row keys 183, bound variables 307, rule labels 62)
total: 0 refusals
exit 0
```

**0 refusals.** The household alone gives 255 ids, 35 keys, 75 row keys, 201 bound variables and 59 rule labels, also with 0 refusals. The 13 household files in the copy are byte-equal to `amendments/household/` (checked with `cmp`), and the 3 addition files are byte-equal to their sources.

**What the scan can't see, checked by reading** (L §2.2, L:80–116):
- No id, key or bound variable is a keyword. `plan`, `to`, `from`, `over`, `paths` and `seed` are used only in their clause positions.
- No built-in name is redeclared. `invested` and `net_worth_on` differ from the household's `invest` pay and `net_worth` ask.
- Key names differ from ask ids: `zero_dip`, `two_rents`, `no_repair` and `nw_1231_mean`, never `below_zero`.
- No new place, so the built-in `place` index is unchanged; the surprises go to the household's `sklepy` (outside PLN). A scenario place would join `place`, which `net_worth`'s sum reads in the base too (LOAD-SCENARIO doubt).
- `costly_repairs` reads `net_worth_on`, a surprises item. It may, being `over surprises` (L:1237).
- Parse rules: `check A <= B = true` splits at the only depth-0 ` = ` (parse rule 1). In `X - X as if {scenario surprises}`, the `as if` binds to the second reference only (parse rule 3), which is the difference intended.

### 2.3 Base net worth NW_B

NW = konto + gotowka + marek + loan_debt + Σ wcash + Σ units × price.
- In 2027: gotowka 600, marek 0, loan_debt −300 (−1200 plus the Jan–Mar 2027 instalments, HH4-b B(a)).
- Units 431 + 869 = 1300 and wcash 19.00 + 38.26, at the 2027 price 133.56 (`etf_price` = 126 × 1.06).
- Every 2027 invest adds its room to the wrappers' worth. A buy is whole units at 133.56, so it is value-neutral.

```
$ python3 -I -c "from decimal import Decimal as D; ..."   (full command in §3.3)
held 2027-01-01 173685.26 reval 9828.00 check 2026-12-31 170202.16
rooms Jan-Jun 69255.95 all 139310.21
NW_B 2027-06-30 250001.40 | NW_B 2027-12-31 319952.22
Dec 29 low 3197.33 | Sep 9 low 9013.95
```

- **NW_B(2027-06-30) = 250001.40** = 6760.19 + 600 − 300 + 173685.26 + 69255.95.
- **NW_B(2027-12-31) = 319952.22** = 6656.75 + 600 − 300 + 173685.26 + 139310.21.
- The check reproduces HH4-b's net_worth[2026-12-31] = 170202.16 from the same parts.

---

## 3. The model in closed form

### 3.1 One month's surprise

E_m = dev_m + 3000·hit_m, where dev_m is normal(0, 400) correctly rounded to 0.01 and hit_m is Bernoulli(0.1), all independent.
- Uniforms: u = (k + 1/2)/2^64. The binomial(1, 0.1) quantile is 1 exactly when u > 0.9. The grid's effect is about 2^−64, which is neglected.
- Rounding: dev_r ≥ m + 0.01 ⇔ dev ≥ m + 0.005 (a tie has measure 0). So P(E > m) = pE(m) = 0.9·Φ(−(m + 0.005)/400) + 0.1·Φ(−(m + 0.005 − 3000)/400), exactly for the rounded draw.
- Symmetry: half-even rounding is odd, round(−x) = −round(x), so E[dev_r] = 0 exactly and E[E_m] = 300 exactly.

### 3.2 Absorption and linearity

- **konto.** A surprise on the 25th of month j lowers konto from that day to the next 15th. plan_low on the 15th of j+1 starts from the lower balance, so room[j+1] = R[j+1] − E_j. After that invest, konto is back to the base. Hence:
  - konto(2027-06-30) = 6760.19 − E_Jun;
  - konto(2027-12-31) = 6656.75 − E_Dec (Dec's surprise is absorbed in Jan 2028);
  - room[2027-08] = 4409.93 − E_Jul;
  - room[2027-11] = 13938.05 − E_Oct.
- **Clipping.** Clipping (R < E, so room = 0 and the deficit carries) needs E_Jul > 4409.93: P = pE(4409.93) = 2.1e-5. Every other month needs E > 9494.21: P ≈ 0. Clipping moves only levels below 2.1e-5, none of the asked ones.
- **Net worth is exactly linear, whatever the absorption.** A surprise moves money to or from an outside place (sklepy). The changed invest is internal: whole-unit buys at a price that is constant through 2027, with the rest kept in wcash. oki_tax and the other 2027 taxes are paid in 2028. So NW(d) = NW_B(d) − Σ E_m over the months whose 25th is ≤ d: n = 6 for 2027-06-30 and n = 12 for 2027-12-31.
- With S_n = Σ_{1..n} E_m = N(0, 400²·n) + 3000·Binomial(n, 0.1), each fan is B − S_n, a finite normal mixture.

### 3.3 The scripts and their output

`fans.py` evaluates the mixture's CDF and density, bisects for the quantile (120 halvings) and computes the binomial tails of the type-1 order statistic. The tolerance is ceil((4·SE + 0.005·n + 0.01)·100)/100.

```
$ python3 -I montecarlo/household-bands-oracle/fans.py
konto 2027-06-30      q0.05 Q=   3760.19 S-quantile=  3000.00 f=9.974e-05 SE=  69.10 tol= 276.43 Pfail=5.03e-04
konto 2027-06-30      q0.25 Q=   6373.22 S-quantile=   386.97 f=5.622e-04 SE=  24.36 tol=  97.45 Pfail=1.26e-04
konto 2027-06-30      q0.50 Q=   6704.31 S-quantile=    55.88 f=8.889e-04 SE=  17.79 tol=  71.17 Pfail=6.77e-05
konto 2027-06-30      q0.75 Q=   6995.97 S-quantile=  -235.78 f=7.545e-04 SE=  18.15 tol=  72.62 Pfail=6.35e-05
konto 2027-06-30      q0.95 Q=   7397.48 S-quantile=  -637.29 f=2.523e-04 SE=  27.32 tol= 109.29 Pfail=6.30e-05
konto 2027-12-31      q0.05 Q=   3656.75 S-quantile=  3000.00 f=9.974e-05 SE=  69.10 tol= 276.43 Pfail=5.03e-04
konto 2027-12-31      q0.25 Q=   6269.78 S-quantile=   386.97 f=5.622e-04 SE=  24.36 tol=  97.45 Pfail=1.26e-04
konto 2027-12-31      q0.50 Q=   6600.87 S-quantile=    55.88 f=8.889e-04 SE=  17.79 tol=  71.17 Pfail=6.77e-05
konto 2027-12-31      q0.75 Q=   6892.53 S-quantile=  -235.78 f=7.545e-04 SE=  18.15 tol=  72.62 Pfail=6.35e-05
konto 2027-12-31      q0.95 Q=   7294.04 S-quantile=  -637.29 f=2.523e-04 SE=  27.32 tol= 109.29 Pfail=6.30e-05
room 2027-08          q0.05 Q=   1409.93 S-quantile=  3000.00 f=9.974e-05 SE=  69.10 tol= 276.43 Pfail=5.03e-04
room 2027-08          q0.25 Q=   4022.96 S-quantile=   386.97 f=5.622e-04 SE=  24.36 tol=  97.45 Pfail=1.26e-04
room 2027-08          q0.50 Q=   4354.05 S-quantile=    55.88 f=8.889e-04 SE=  17.79 tol=  71.17 Pfail=6.77e-05
room 2027-08          q0.75 Q=   4645.71 S-quantile=  -235.78 f=7.545e-04 SE=  18.15 tol=  72.62 Pfail=6.35e-05
room 2027-08          q0.95 Q=   5047.22 S-quantile=  -637.29 f=2.523e-04 SE=  27.32 tol= 109.29 Pfail=6.30e-05
room 2027-11          q0.05 Q=  10938.05 S-quantile=  3000.00 f=9.974e-05 SE=  69.10 tol= 276.43 Pfail=5.03e-04
room 2027-11          q0.25 Q=  13551.08 S-quantile=   386.97 f=5.622e-04 SE=  24.36 tol=  97.45 Pfail=1.26e-04
room 2027-11          q0.50 Q=  13882.17 S-quantile=    55.88 f=8.889e-04 SE=  17.79 tol=  71.17 Pfail=6.77e-05
room 2027-11          q0.75 Q=  14173.83 S-quantile=  -235.78 f=7.545e-04 SE=  18.15 tol=  72.62 Pfail=6.35e-05
room 2027-11          q0.95 Q=  14575.34 S-quantile=  -637.29 f=2.523e-04 SE=  27.32 tol= 109.29 Pfail=6.30e-05
net worth 2027-06-30  q0.05 Q= 243614.80 S-quantile=  6386.60 f=3.761e-05 SE= 183.26 tol= 733.10 Pfail=1.62e-04
net worth 2027-06-30  q0.25 Q= 246710.62 S-quantile=  3290.78 f=1.397e-04 SE=  98.03 tol= 392.15 Pfail=9.00e-05
net worth 2027-06-30  q0.50 Q= 248677.03 S-quantile=  1324.37 f=1.202e-04 SE= 131.52 tol= 526.14 Pfail=1.19e-04
net worth 2027-06-30  q0.75 Q= 250075.52 S-quantile=   -74.12 f=2.168e-04 SE=  63.15 tol= 252.66 Pfail=8.76e-05
net worth 2027-06-30  q0.95 Q= 251290.85 S-quantile= -1289.45 f=9.103e-05 SE=  75.71 tol= 302.89 Pfail=6.28e-05
net worth 2027-12-31  q0.05 Q= 310186.69 S-quantile=  9765.53 f=2.439e-05 SE= 282.58 tol=1130.39 Pfail=1.42e-04
net worth 2027-12-31  q0.25 Q= 314196.26 S-quantile=  5755.96 f=8.184e-05 SE= 167.32 tol= 669.37 Pfail=6.49e-05
net worth 2027-12-31  q0.50 Q= 316705.70 S-quantile=  3246.52 f=1.211e-04 SE= 130.51 tol= 522.13 Pfail=7.69e-05
net worth 2027-12-31  q0.75 Q= 318893.59 S-quantile=  1058.63 f=1.015e-04 SE= 134.94 tol= 539.82 Pfail=6.42e-05
net worth 2027-12-31  q0.95 Q= 321243.30 S-quantile= -1291.08 f=5.358e-05 SE= 128.64 tol= 514.63 Pfail=6.89e-05
sum of Pfail over the 30 fan statistics: 4.23e-03
```

`shares.py` gives the shares, means, paired statistics and verdict figures:

```
$ python3 -I montecarlo/household-bands-oracle/shares.py
below_zero (some day of 2027)        p=0.661666 n~ 661.7 SE=0.014962 tol=0.059848 accept n in [602, 721] Pfail=5.99e-05
below_two_rents (2027-12-31 < 6400)  p=0.334426 n~ 334.4 SE=0.014919 tol=0.059677 accept n in [275, 394] Pfail=5.73e-05
paired: diff = 0, P(K12 = 0)         p=0.282430 n~ 282.4 SE=0.014236 tol=0.056944 accept n in [226, 339] Pfail=6.27e-05
per-month P(E > 1800) 0.099868 Dec 0.031089 Aug 2.1689232693645893e-52 P(konto 2027-12-31 < 0) 3.0685831038855937e-21 clip Aug 2.1187132219186038e-05 E[dev+]*12 1914.92
Fr(9,10)**12 = 282429536481/1000000000000 ; P(K12<=1) = 0.659002251789
mean net worth 2027-12-31 316352.22 SD 3411.74 SE 107.89 tol 431.56
paired mean -3600.0000000000005 SD 3117.69 SE 98.59 tol 394.36
paired median = -3000 fails with P = 3.669675179440459e-25
0 of 51: 1600*pt*(1-pt) = 56.0661 vs N+4 = 55 -> fail; 2*se_v = 0.050482; P(n = 0) = 1 - 1.6e-19
0 of 52: 1600*pt*(1-pt) = 55.1020 vs N+4 = 56 -> pass; 2*se_v = 0.049598; P(n = 0) = 1 - 1.6e-19
0 of 1000: 1600*pt*(1-pt) = 3.1809 vs N+4 = 1004 -> pass; 2*se_v = 0.002814; P(n = 0) = 1 - 3.1e-18
```

(The script prints the mean's tolerance rounded half-even, 394.36. The row is ceil(394.3602…·100)/100 = 394.37, by the rule of §3.4.)

The base figures and the tolerances to full precision:

```
$ python3 -I -c "
from decimal import Decimal as D; from datetime import date
R=[D(x) for x in '11410.06 9835.50 13685.82 12253.87 10606.19 11464.51 11889.02 4409.93 18791.75 9494.21 13938.05 11531.30'.split()]
held=1300*D('133.56')+D('19.00')+D('38.26'); print('held 2027-01-01', held, 'reval', 1300*(D('133.56')-126), 'check 2026-12-31', D('6944.90')+600+0-1200+1300*126+D('19.00')+D('38.26'))
print('rooms Jan-Jun', sum(R[:6]), 'all', sum(R))
print('NW_B 2027-06-30', D('6760.19')+600+0-300+held+sum(R[:6]), '| NW_B 2027-12-31', D('6656.75')+600+0-300+held+sum(R))
print('Dec 29 low', D('11898.63')-D('8701.30'), '| Sep 9 low', 1800+D('7213.95'))
K=date(2026,10,19)
for d in [date(2027,6,30),date(2027,8,31),date(2027,11,30),date(2027,12,31)]: print(d, (d-K).days, round((d-K).days/365,4))"
held 2027-01-01 173685.26 reval 9828.00 check 2026-12-31 170202.16
rooms Jan-Jun 69255.95 all 139310.21
NW_B 2027-06-30 250001.40 | NW_B 2027-12-31 319952.22
Dec 29 low 3197.33 | Sep 9 low 9013.95
2027-06-30 254 0.6959
2027-08-31 316 0.8658
2027-11-30 407 1.1151
2027-12-31 438 1.2

$ python3 -I -c "... 4*sqrt(p*(1-p)/1000) for each share; the two means' 4*SE"
zero_dip 0.6616659325761982 0.05984835853979865
two_rents 0.33442580392395005 0.05967715617805792
no_repair 0.2824295364810001 0.056943915341904974
mean 431.55532669635767 paired 394.36024140371967
```

### 3.4 The tolerance, term by term

The step bias b is my reading of ENGINE §8's undefined term (MC2-6): the gap between the statistic of the discretised draws and the closed form of the continuous model.

| Statistic kind | 4·SE | Step bias b | Oracle's own error e | Row |
|---|---|---|---|---|
| quantile of a sum of n surprises | 4·√(a(1−a)/N)/f(Q) | 0.005·n zł: each rounded deviation is within 0.005 of its continuous value | 0.01 zł (Q printed to the cent) | ceil to 0.01 |
| share | 4·√(p(1−p)/N) | 0: pE models the rounded draw exactly | 1e-6 where p is printed to 6 places; 0 for 0.9^12 (exact) | ceil to 1e-6 |
| mean | 4·SD/√N | 0: rounding is odd, so it adds no mean | 0: the closed form is exact | ceil to 0.01 |
| paired median, test-mode values | — | 0 | 0 | exact equality |

The uniform grid (2^−64) shifts nothing at these scales.

---

## 4. The statistics

For each statistic: the value, its derivation, the SE at N = 1000, the tolerance, the verdict, the acceptance rule as the test applies it, P(a correct engine fails) and one wrong engine. Horizons h = (d − K)/365: 2027-06-30 is 0.70, 2027-08-31 is 0.87, 2027-11-30 is 1.12 and 2027-12-31 is 1.20 (254, 316, 407 and 438 days).

`mean` and `quantile` print an SE, c and h with no verdict (ENGINE §8). h is as above, and c is only bounded, c ≤ h (MC2-7). The engine's SE formula for a quantile is unstated (MC2-5), so the test never compares the printed SE; the tolerances are rows built from the oracle's SE.

### 4.1 konto fan (`band_konto`; checks `check_konto_q05..q95`)

konto(d) = B − E, so Q_konto(a) = B − Q_E(1 − a), with B = 6760.19 (06-30) and 6656.75 (12-31).

| Level | 2027-06-30 | 2027-12-31 | SE | tol | Pfail |
|---|---|---|---|---|---|
| 0.05 | 3760.19 | 3656.75 | 69.10 | 276.43 | 5.03e-4 |
| 0.25 | 6373.22 | 6269.78 | 24.36 | 97.45 | 1.26e-4 |
| 0.50 | 6704.31 | 6600.87 | 17.79 | 71.17 | 6.77e-5 |
| 0.75 | 6995.97 | 6892.53 | 18.15 | 72.62 | 6.35e-5 |
| 0.95 | 7397.48 | 7294.04 | 27.32 | 109.29 | 6.30e-5 |

- **q05 sits exactly at 3000.** F_E(3000) = 0.9·Φ(7.5) + 0.1·Φ(0) = 0.95 − 2.9e-14. So E's 0.95-quantile is 3000.00 to the cent, and q05 = B − 3000.
  - The density there is only the repair component's peak, 0.1·φ(0)/400 = 9.974e-5. The SE is therefore large, and the order statistic is skewed: Pfail is 5.03e-4, not the normal 6.3e-5.
- **Acceptance.** Each `check_konto_qNN` prints `ok` on both lines (d = 2027-06-30 and d = 2027-12-31), that is |Q̂ − oracle_konto[d, qNN]| ≤ oracle_konto_tol[*, qNN]. `band_konto` prints the five money values per date.
- **Wrong engine (foresight).** If the look-ahead uses the drawn values instead of the plans, the 15th invests R − E before the surprise lands. konto then returns to B on every path, and every quantile is B. q05 is off by 3000.00 (> 276.43) and q25 by 386.97 (> 97.45); q50 alone (55.88 < 71.17) wouldn't catch it.

### 4.2 Net-worth fan (`band_net_worth`; checks `check_net_worth_q05..q95`)

NW(d) = NW_B − S_n, with n = 6 (06-30, NW_B 250001.40) and n = 12 (12-31, NW_B 319952.22).

| Level | 2027-06-30 | SE | tol | 2027-12-31 | SE | tol |
|---|---|---|---|---|---|---|
| 0.05 | 243614.80 | 183.26 | 733.10 | 310186.69 | 282.58 | 1130.39 |
| 0.25 | 246710.62 | 98.03 | 392.15 | 314196.26 | 167.32 | 669.37 |
| 0.50 | 248677.03 | 131.52 | 526.14 | 316705.70 | 130.51 | 522.13 |
| 0.75 | 250075.52 | 63.15 | 252.66 | 318893.59 | 134.94 | 539.82 |
| 0.95 | 251290.85 | 75.71 | 302.89 | 321243.30 | 128.64 | 514.63 |

- The step bias is 0.03 at n = 6 and 0.06 at n = 12.
- Pfail is 1.62e-4, 9.00e-5, 1.19e-4, 8.76e-5 and 6.28e-5 at 06-30, and 1.42e-4, 6.49e-5, 7.69e-5, 6.42e-5 and 6.89e-5 at 12-31.
- **Acceptance.** Each `check_net_worth_qNN` prints `ok` on both lines.
- **Wrong engine (path key left out of the address).** Every path then gets the same uniforms and the fan collapses to one value v. The five oracles span 251290.85 − 243614.80 = 7676.05 at 06-30 and 11056.61 at 12-31. So some level misses by at least 3838.03 (06-30) or 5528.30 (12-31), far beyond every tolerance.

### 4.3 Room band (`band_room`; checks `check_room_q05..q95`)

room[2027-08] = 4409.93 − E_Jul and room[2027-11] = 13938.05 − E_Oct. This is the same law as §4.1, so the SEs, tolerances and Pfail are those of §4.1.

| Level | 2027-08 | 2027-11 | tol |
|---|---|---|---|
| 0.05 | 1409.93 | 10938.05 | 276.43 |
| 0.25 | 4022.96 | 13551.08 | 97.45 |
| 0.50 | 4354.05 | 13882.17 | 71.17 |
| 0.75 | 4645.71 | 14173.83 | 72.62 |
| 0.95 | 5047.22 | 14575.34 | 109.29 |

- `invested[m]` is money (it is flows), so it prints to the cent.
- August clips (room 0) with P = 2.1e-5. That is below every asked level, so it moves no quantile.
- **Acceptance.** Each `check_room_qNN` prints `ok` for m = 2027-08 and m = 2027-11.
- **Wrong engine (the walk not re-run per path, ENGINE §7 step 2).** If the rule of the 15th is computed once in the shared layer, every path invests R, and every quantile is R. q05 is off by 3000.00 (> 276.43) and q25 by 386.97 (> 97.45).

### 4.4 "konto falls below 0 on some day of 2027" (`below_zero`; check `check_zero_dip`)

year_low < 0 ⇔ some month's E exceeds that month's post-25th minimum: 1800 for ten months, 3197.33 for Dec and 9013.95 for Aug (§2.1). Different months' E are independent, and absorption keeps each dip separate.
- p = 1 − (1 − pE(1800))^10 · (1 − pE(3197.33)) · (1 − pE(9013.95)) = 1 − (1 − 0.099868)^10 · (1 − 0.031089) · (1 − 2.2e-52) = **0.661666** (0.66166593…).
- n ≈ 662 of 1000. SE = 0.014962. tol = ceil6(4·0.0149621 + 1e-6) = **0.059850**.
- **Acceptance.** `check_zero_dip` prints `ok` ⇔ |n/1000 − 0.661666| ≤ 0.059850 ⇔ n ∈ [602, 721]. P(a correct engine fails) = 6.0e-5 (exact binomial).
- **Verdict at n = 662.** Precision: p̃ = 664/1004 = 0.661355 and 1600·p̃(1 − p̃) = 358.3435 ≤ 1004, so it passes (2·se_v = 0.029871; it passes for every n at N = 1000). Carry: c ≤ h = 1.20 ≤ 2, so it passes. No pin. **`sayable`.** Printed: "662 of 1000", 0.662, se 0.014958 (√(s(1 − s)/N) at s = 0.662), then `sayable`.
- **Wrong engine (the month key dropped from the address).** Then one E per path hits all twelve months, and year_low < 0 ⇔ E > 1800: p = 0.099868, n ≈ 100, outside [602, 721].

### 4.5 "konto ends 2027 below two rents" (`below_two_rents`; check `check_two_rents`)

2·rent = 6400 (§1). konto(2027-12-31) < 6400 ⇔ E_Dec > 6656.75 − 6400 = 256.75.
- p = pE(256.75) = 0.9·Φ(−256.755/400) + 0.1·Φ(2743.245/400) = **0.334426** (0.33442580…).
- n ≈ 334. SE = 0.014919. tol = ceil6(4·0.0149193 + 1e-6) = **0.059679**.
- **Acceptance.** `check_two_rents` prints `ok` ⇔ n ∈ [275, 394]. Pfail = 5.7e-5.
- **Verdict at n = 334.** p̃ = 336/1004 = 0.334661, 1600·p̃(1 − p̃) = 356.2610 ≤ 1004 (2·se_v 0.029784). c ≤ h = 1.20. No pin. **`sayable`.**
- **Cross-check with the fan, free of chance.** In the same run, type 1 gives Q̂(a) < 6400 ⇔ n ≥ ⌈1000a⌉. So `band_konto`'s 2027-12-31 q25 < 6400 ⇔ n ≥ 250, and its q50 < 6400 ⇔ n ≥ 500. At the predicted n ≈ 334, q25 = 6269.78 < 6400 < q50 = 6600.87.
- **Wrong engine (foresight, as in §4.1).** konto(12-31) = 6656.75 ≥ 6400 on every path, so 0 of 1000, outside [275, 394].

### 4.6 The paired statistics (`costly_repairs`: `check_no_repair`, `check_paired_mean`, `check_paired_median`, `paired_cost`)

costly_repairs sets `repair_cost = 6000` over surprises. Draw addresses don't name the scenario (L:1246, E:321), so path p sees the same deviations and hits in both. The deviations cancel exactly, and diff = NW_costly(12-31) − NW_surprises(12-31) = −3000·K, with K = Σ hit_m ~ Binomial(12, 0.1). NW is linear, so absorption doesn't matter (§3.2), even where 6000 zł clips August's room.
- **share(diff = 0)** = P(K = 0) = 0.9^12 = **282429536481/10^12 = 0.282429536481**, exact. SE = 0.014236, tol = ceil6(4·0.0142360) = **0.056944**. `check_no_repair` passes ⇔ n ∈ [226, 339]; Pfail = 6.3e-5.
  - **Verdict at n = 282:** p̃ = 284/1004, 1600·p̃(1 − p̃) = 324.5663 ≤ 1004; c ≤ 1.20; no pin. **`sayable`.**
- **mean(diff)** = −3000·1.2 = **−3600**, exact. SD = 3000·√(12·0.09) = 3117.69, SE = 98.59, tol = ceil2(394.3602) = **394.37**.
  - The mean is −3·ΣK with ΣK ~ Binomial(12000, 0.1) exactly. So `check_paired_mean` passes ⇔ ΣK ∈ [1069, 1331], and P(fail) = 6.41e-5 exactly:

    ```
    $ python3 -I -c "... sum of Binomial(12000, 0.1) pmf over |-3k+3600| > 394.37"
    paired mean Pfail exact 6.414305336108995e-05 reject sumK <= 1068 or >= 1332
    ```
- **quantile(diff, 0.5)** = **−3000** exactly. The type-1 median is the 500th smallest, which is −3000 ⇔ #(K ≥ 2) ≤ 499 and #(K ≥ 1) ≥ 500. P(K ≥ 2) = 0.341 and P(K ≥ 1) = 0.718, so it fails with P = 3.7e-25. `check_paired_median` compares with `=` (tol 0).
- **Determinism by-product.** Every path's diff is a multiple of −3000, so the printed mean is −3·ΣK exactly (a multiple of 0.003 PLN).
- **Wrong engine (unpaired: the scenario name in the address, or a stream per scenario).** diff then has SD √(12(400² + 0.09·6000²) + 12(400² + 0.09·3000²)) = 7241.55 and is continuous. share(diff = 0) ≈ 0 (outside [226, 339]), and the median is not −3000.

### 4.7 Mean net worth 2027-12-31 (`mean_net_worth`; check `check_mean_net_worth`)

- E[NW] = 319952.22 − 12·0.1·3000 − 12·E[dev_r] = 319952.22 − 3600 − 0 = **316352.22**, exact by §3.1.
- SD = √(12·400² + 12·0.09·3000²) = 3411.74. SE = 107.89. tol = ceil2(431.5553) = **431.56**.
- **Acceptance.** `check_mean_net_worth` prints `ok`. Pfail ≈ 6.3e-5: the sum of 12 000 terms is close to normal.
- **Wrong engine (negative deviations dropped, as when `spend_under` never books or the normal is clipped at 0).** The mean is less 12·400·φ(0) = 1914.92, so 314437.30, off by 1914.92 > 431.56.

### 4.8 Test mode and the collapse (`test_mode`, `check_tm_konto`, `check_tm_net_worth`, `check_tm_room`, `check_tm_zero_dip`)

`with scenario surprises, repair_hit[*, *] = 0, spend_dev[*, *] = 0`: both draws are pinned for every path (L:1495; the `[*, *]` form is MC2-3). With the pins, every surprise instance is off and every path is the base:

| Check | Statistic | Exact value |
|---|---|---|
| `check_tm_konto` | quantile(konto 2027-12-31, 0.05) | **6656.75** (HH4-b) |
| `check_tm_net_worth` | quantile(NW 2027-12-31, 0.05) | **319952.22** (§2.3) |
| `check_tm_room` | quantile(invested[2027-11], 0.05) | **13938.05** (HH4-b room 2027-11) |
| `check_tm_zero_dip` | share(year_low < 0) | **0** |

- Each is compared with `=`, tol 0, and must print `ok`. Pfail = 0: the run is deterministic.
- **Verdict of `test_mode`'s share.** "0 of 1000", fraction 0, se 0.000000.
  - Precision: p̃ = 2/1004, 1600·p̃(1 − p̃) = 3.1809 ≤ 1004, so it passes (2·se_v 0.002814).
  - Carry passes (c ≤ 1.20).
  - Test mode fails: the register holds a pin on a draw.
  - **`not sayable`**, with a reason naming the test mode. Its wording is unspecified (MC2-4).
- The register lists `pinned: repair_hit[*, *] = 0`, `pinned: spend_dev[*, *] = 0` and `scenario: surprises`.
- **Wrong engine (the pin not in the register).** It prints `sayable` for a pinned run.

### 4.9 The verdict edge and the base values

- **`verdict_51`**: share(konto 2027-12-31 < 0) over 51 paths.
  - P(konto(12-31) < 0) = pE(6656.75) = 3.07e-21, so n = 0 with P = 1 − 1.6e-19.
  - p̃ = 2/55; 1600·p̃(1 − p̃) = 56.0661 > 55.
  - Prints "0 of 51", 0, se 0.000000, then `not sayable: 2·se 0.050482 > 0.05 (0 of 51)`. Carry passes (h = 1.20) and is not named.
- **`verdict_52`**: p̃ = 2/56; 55.1020 ≤ 56, so it passes (2·se_v 0.049598). Prints "0 of 52" and `sayable`. This matches ENGINE §8: "0 of N first passes at N = 52".
- **Wrong engines.**
  - Wald (the printed se, 0) makes `verdict_51` sayable.
  - Agresti–Coull taken over N instead of N + 4 gives 2·√((2/56)(54/56)/52) = 0.051470 > 0.05, which makes `verdict_52` not sayable.
- **`base_values`** (no `paths`), under reading MC2-1(a): the path index is empty, so no surprise pay has an instance, and the ask prints the base. NW 2027-06-30 = **250001.40**, konto 2027-06-30 = **6760.19**, NW 2027-12-31 = **319952.22**, konto 2027-12-31 = **6656.75**. Under the other readings it is refused (`RUN-NO-PATH`, L:1493, L:1626).

### 4.10 The chance that a correct engine fails at 4·SE

The union bound over every check ask:
- the 30 fan checks, 4.23e-3;
- the three share checks, 6.0e-5 + 5.7e-5 + 6.3e-5;
- the two mean checks, 6.3e-5 + 6.4e-5;
- the paired median, 3.7e-25;
- test mode, 0.

That totals **4.5e-3, about 1 run in 220**. Almost half (2.0e-3) is the four one-month q05 checks. They sit in the mixture's valley, where the asymptotic SE understates the order statistic's spread. If the suite wants less, those four tolerances are the ones to widen; the rule as given (4·SE) is kept here.

```
$ python3 -I -c "... union of the Pfail above"
union bound all check asks 0.004537365692687967 = 1 in 220
```

### 4.11 Under HH4-b ambiguity A(b) (a planned promise counts as kept: P = 0 after 2026-12-31)

The base konto is 800 lower (5960.19 and 5856.75) and the post-25th minimum is 1000. Rooms from Feb on are unchanged, and NW is unchanged (the 800 is invested in Jan, which is internal).

```
$ python3 -I -c "... pE as in §3.1; lows 1000 x10, 2397.33, 8213.95"
A(b): per-month P(E>1000) 0.105588 below_zero 0.702979 below_two_rents P(E>-543.25) 0.921507
```

- konto fans: every value 800 lower, with the same tolerances.
- below_zero: 0.702979 (SE 0.014450, tol 0.057801, n ∈ [646, 760] at the same rule).
- below_two_rents: 0.921507.
- NW fan, room band, the paired statistics and the mean: unchanged.

The key's rows are A(a), HH4-b's first choice (MC2-14).

---

## 5. Determinism assertions

These are exact: a correct engine meets each with probability 1.

1. **Byte-for-byte replay at a seed.** Running the whole reality's asks twice with `seed 7` gives byte-identical output, registers and verdicts included. The result hashes are equal.
2. **Workers.** The same output with 1 and with 6 workers (ENGINE §7 step 5, the suite in §9). 6 doesn't divide 1000 (blocks of 167 and 166) or 51, so uneven blocks are covered.
3. **One path replays alone.** `replay_path` (`explain year_low[17] … paths 1000 seed 7`) prints:
   - the path key 17;
   - exactly 24 draws in the canonical order (L:978: path, then time key, then draw id in ASCII): `repair_hit[17, 2027-01]|7`, `spend_dev[17, 2027-01]|7`, `repair_hit[17, 2027-02]|7`, …, `spend_dev[17, 2027-12]|7`;
   - no draw of any other path;
   - a value equal to path 17's kept term (E §10, "a path value").

   The test may recompute these 24 uniforms from SHA-256 itself; this key doesn't.
4. **Prefix.** A path is a pure function of its key (L:973, L:1492), so paths 1..M are the same in every run with N ≥ M:
   - `replay_one` (paths 1) gives mean(year_low) = year_low[1] of the 1000-path run, and likewise for net worth;
   - verdict_52's n − verdict_51's n ∈ {0, 1}, and it is 1 only if path 52 has konto(12-31) < 0.
5. **Asks in one run share paths.** In particular, `band_konto`'s 12-31 quantiles and `below_two_rents`' n agree as in §4.5 (Q̂(a) < 6400 ⇔ n ≥ ⌈1000a⌉).
6. **Common random numbers.** In costly_repairs every path's diff is in {0, −3000, …, −36000}, so the median is −3000 and the mean is a multiple of 0.003 (§4.6).
7. **Invariant 3 (ENGINE §9).** Every movement and balance before 2027-01-01 equals the base run's. In fact nothing differs before 2027-01-25, the first surprise date (a Monday).

---

## 6. Design limits

- `plan 0` is the median of both draws and the mean of `spend_dev`. For `repair_hit`, an engine that plans with the mean (0.1) still fires no instance, since `= 1` is false. So mean-for-plan confusion is invisible here.
- No price draw: the ETF price can't be drawn from new files (MC2-12). The fans are of konto, net worth and room only.
- The fans are one-month or n-month sums; nothing compounds. That keeps every oracle closed-form.

---

## 7. Findings

| Id | Where | What | What it blocks | Readings |
|---|---|---|---|---|
| MC2-1 | L:886, L:601, L:966, L:1493; E:311–316, E:323–326 | How a pay or def reads a path-keyed draw, and which instances a `p in path` pay has on path q. L:886 says "one per path"; E:325's `alive[p, m]` and ENGINE §8's `share(p in path: bal(konto, …))` read per-path values with no path key. | Every pay here; `base_values` with no paths | (a) on path q only the instance with p = q exists, and with no `paths` there are none (mine; `base_values` prints the base); (b) the path is implicit in the per-path layer, but a draw's address needs the key (L:966) and nothing names it; (c) literally N instances per path, which books every path's surprises on each |
| MC2-2 | L:1237, L:1591; grammar L:127–128 | A scenario file's items list versions, index additions, places, inputs, defs, pays and asks, but not `draw` or `index`. | `surprises.tally`'s two draws and two indexes | (a) allowed, as sections a file may hold (mine); (b) refused, so the draws move to a reality part and only the rows stay in the scenario |
| MC2-3 | L:175–177, L:206 vs L:1495 | A pin is `ref "=" expr` and a ref's keys are exprs, so `theta[*]` (L:1495) has no grammar; nor is it said whether two keys need `[*, *]`. | The five test-mode asks | (a) `*` per key, as in rowkeys (mine); (b) one `[*]` covers all keys |
| MC2-4 | E:363, E:365 | No reason wording for the test-mode failure; and does a pin to the plan or median (`repair_hit = 0`; the mean is 0.1) count as "pinned to its mean"? | `test_mode`'s printed reason | Any pin on a draw makes it not sayable (mine, by E:363's "a pin on a draw"); wording e.g. `not sayable: test mode (pinned: repair_hit[*, *] = 0)` |
| MC2-5 | L:1047–1050; L:1007 | `mean` and `quantile` over paths print an SE, but no formula is given; the asymptotic quantile SE needs a density, and "a sample has no density". | Printing and testing the SE | The test doesn't compare the printed SE; the tolerances are rows from the oracle's SE (√(a(1−a)/N)/f(Q)) |
| MC2-6 | DONE.md:24, E:353 | "Step bias" is undefined. | How every tolerance is built | Mine (§3.4): 0.005·n zł for a sum of n cent-rounded draws; 0 for shares (rounding modelled exactly) and means (odd rounding); the 2^−64 grid ignored |
| MC2-7 | E:360–362 | Carry: (1) the places a term reads mix unit-counted holdings with PLN; (2) do state pays (`invest`) and promised movements count as "plan movements"; (3) the places of a `flows(pay)` term; (4) a def over paths reading balances (`year_low`). | The printed c (h is settled: 0.70, 0.87, 1.12, 1.20) | c ≤ h ≤ 1.20 < 2 under every reading, so no verdict here depends on it |
| MC2-8 | L:399–410 | A draw's law date is unstated: which date reads `repair_chance` and `spend_sd`. | Every draw here | (a) the draw's time key (2027-mm-01) (mine); (b) its reader's; (c) K, where the scenario rows aren't visible yet, so every draw is a hole |
| MC2-9 | L:1240–1242 + L:405 | Scenario rows govern D..; an ask with no date reads at K < D, so a scenario row read directly in an ask is a hole. | Thresholds and oracles as scenario rows | Worked around: thresholds as `2 * rent`, oracles in a reality part |
| MC2-10 | L:155, L:969, L:988 | `to UNIT` is mandatory on every draw, even an exact one (`binomial … to 1`); `uniform` is "exact" in the table but continuous by rule 4. | Minor | `to` ignored for discrete D (mine) |
| MC2-11 | L:1237, L:1246, L:1408 | Does an ask declared in a scenario file run in that scenario by default? | Every ask here | I wrote `with scenario …` on each, which is right under both readings |
| MC2-12 | `amendments/household/pays.tally:48–53`; L:1172; grammar L:136–137, L:169 | An ETF price draw can't be added in new files: `etf_price` and `etf_growth` live in pays.tally (an item lives wholly in one file), and rows and givens are literals, so `etf_return` can't be set to a draw. | Price fans and worth bands from returns | An amend to pays.tally (not new files), or a `v` hook in the household |
| MC2-13 | E:317, E:406, E:413; DONE.md:24 | "A path is replayed alone", but there is no syntax for it, and the kept terms aren't exposed. | Determinism assertion 3 | `explain year_low[17] … paths 1000` (mine); a `paths 17..17` form would be cleaner |
| MC2-14 | E:62; HH4-b.md:741–755 (ambiguity A) | A promised movement after K is never confirmed, so P = 800 for the run. | Every konto statistic (±800) and below_zero | A(a) in the rows; A(b) values in §4.11 |
| MC2-15 | E:353 vs L:1047, L:1595; L:1048 | E:353 says the statistic and its SE are "each a def or a row", but a path aggregate outside an ask is `LOAD-ASKONLY`, and the printed SE has no expression form. Nor is it said what `share(…)` is as an operand (n/N?), or whether a verdict prints inside a `check`. | Every `check_*` ask | Statistic inline in the check, its value n/N or the quantile's value; tolerance from a row; the verdict read from the plain share asks |

---

## 8. Summary of the predicted statistics (N = 1000, seed 7, scenario surprises)

| Statistic | Predicted | tol | Accept |
|---|---|---|---|
| konto 06-30 q05/q25/q50/q75/q95 | 3760.19 / 6373.22 / 6704.31 / 6995.97 / 7397.48 | 276.43 / 97.45 / 71.17 / 72.62 / 109.29 | `check_konto_*` ok |
| konto 12-31 | 3656.75 / 6269.78 / 6600.87 / 6892.53 / 7294.04 | same | ok |
| room 2027-08 | 1409.93 / 4022.96 / 4354.05 / 4645.71 / 5047.22 | same | `check_room_*` ok |
| room 2027-11 | 10938.05 / 13551.08 / 13882.17 / 14173.83 / 14575.34 | same | ok |
| NW 06-30 | 243614.80 / 246710.62 / 248677.03 / 250075.52 / 251290.85 | 733.10 / 392.15 / 526.14 / 252.66 / 302.89 | `check_net_worth_*` ok |
| NW 12-31 | 310186.69 / 314196.26 / 316705.70 / 318893.59 / 321243.30 | 1130.39 / 669.37 / 522.13 / 539.82 / 514.63 | ok |
| below_zero | 0.661666 (≈ 662 of 1000), sayable | 0.059850 | n ∈ [602, 721] |
| below_two_rents | 0.334426 (≈ 334), sayable | 0.059679 | n ∈ [275, 394] |
| costly − surprises = 0 | 0.282429536481 (≈ 282), sayable | 0.056944 | n ∈ [226, 339] |
| mean NW 12-31 | 316352.22 | 431.56 | ok |
| paired mean / median | −3600 / −3000 | 394.37 / 0 | ok |
| test mode | 6656.75 / 319952.22 / 13938.05 / 0 of 1000, not sayable | 0 | ok |
| verdict_51 / verdict_52 | 0 of 51 not sayable (2·se 0.050482) / 0 of 52 sayable | — | exact text |
| base_values | 250001.40, 6760.19, 319952.22, 6656.75 | 0 | exact (MC2-1(a)) |
