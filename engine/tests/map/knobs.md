# The halving test: knobs of the heavy checks

Session 4.5b, 2026-10-29. Paper only: no `refcore` run. The one run made: `keys_b.py 30` and `keys_d.py 200` on a scratch copy of `engine/tests/cases` (5.7 s and 1.8 s). Both take the path count as their only setting. Both write 12 keys and pass their relations, and only B-12 and D-10 move. No key script takes a run end, a span start or any other size.

## Which checks are heavy

- **Every row of `engine/tests/load/benchmarks.md`.** Row 7 doesn't exist.
- **Every stored CPython time over 240 s:**
  - `snap16-light`: LH-16-light 347 s. LH-17-mc-light (144 s) and LH-15-light (121 s) are under 240 s.
  - `engine/tests/baselines/session5-visible`:
    - B-b12_det 906 s and B-b12_p3 852 s, both stopped (rc −15);
    - B-cs_b09_b11 1 301 s;
    - E1-02-B 597 s.
  - Nothing else in snap16's `summary.txt` files passes 240 s: the slowest are LH-10 96 s and I-6 110 s.
  - The household runs in `snap16-house` store no times.
- **4.5a's `over-budget.md`** didn't exist when this was written. `checks.tsv` stores `baseline_s` only for unit, model, light and hardset checks.
- **On the watch list, not heavy:** every single B ask, about 180 s on CPython and 47 s on PyPy (`tmp/session4_5/progress.md`). That is under 240 s but leaves a thin margin under the 60 s budget. Knob B-run-end below applies to all of them.

**The sources** of a capability setting's answer:
- **(i)** the key unchanged;
- **(ii)** a key script at that setting;
- **(iii)** none yet.

**Random figures:** the key scripts draw with Python's `random.Random` (`tools/lib/paths.py`, `fam/*_paths.py`), not with the language's draws. So a run at n paths is judged statistically against the key, which keeps its own se: the full-n key stays the target, and the run's se at n sets the tolerance. Session 4 judged D-10 this way at 2 000 paths (`rows/D.check.md` line 72). That is source (i) for any figure with an se. A quantile prints no se, so at a cut n it is (iii).

---

## K1 · `I:D:D-10`, D's forecast to death (load twin `L:1`)

| Knob | Full | Kind | Capability setting | Source |
|---|---|---|---|---|
| paths | 10 000 | scale | 200 keeps every branch (dry, alive at 2045, dead before 2035); 200 paths took 12–16 s on snap8. For judging power, 2 000 (151 s and 106 s on CPython for the two path asks, about 40 s on PyPy, estimated) | closed forms and zero volatility: (i), they don't depend on n. Whole years, tax to death, share dry: (i), statistical. **Bands 2035/2045 (6 cells): (iii)** at any n under 10 000 |
| horizon | 2064-12-31 (ages 68..105) | capability | none: "tax to death" and the lifetimes need every age the table holds, and the dry year 2055 is a key figure | – |
| band years | 2035, 2045 | capability (they are the asked dates) | – | – |

## K2 · `I:D:D-11`, the zero-volatility pair (`L:2`)

| Knob | Full | Kind | Capability setting | Source |
|---|---|---|---|---|
| run end | to the dry year (2055 base, 2052 at 8 %) | scale for figures 1–5 and 7, which end 2031-12-31; capability for figure 6 (the dry year is the answer) | split in two: D-11a to 2031-12-31 (figures 1–5, 7); D-11b, one deterministic path to 2055 | D-11a (i); D-11b (i). Not path-dependent: D-11 doesn't move under `keys_d.py 200` |

## K3 · household B as one run (`L:3`, `L:6`)

| Knob | Full | Kind | Capability setting | Source |
|---|---|---|---|---|
| run end (`reality.tally:5`) | 2028-12-31 | scale for B-01..B-11, whose last dates are ≤ 2026-01-26 (B-10) | 2026-01-31, as `scratch/mc-b/cs` did at 2026-12-31. The copy also shortens `market.tally:53`'s row, which names keys past the end (`rows/B.check.md` line 9) | (i) for B-01..B-11. The cells after the cut are load only: `L:6` |
| span start | 2022-03-01 | capability: every B question reads the history, which holds B-04 (2022-07), B-02 (2023-02), the mortgage's 36-month fee clock and the records | – | – |
| asks per run | all of B's | scale; 4.5a already splits per ask | one ask | (i) |
| law files | 33 | capability (B's law) | – | – |

## K4 · `I:B:B-07`, the gross-up search (`L:4`)

| Knob | Full | Kind | Capability setting | Source |
|---|---|---|---|---|
| run end | 2028-12-31 | scale (B-07 ends 2025-04-30) | 2025-12-31 (the writer's span; 9 of 9 figures judged there) | (i) |
| search steps | 4 001 (`gross_lo`/`gross_hi` 10 340.00/10 380.00, the key's ±20.00) | scale | ±0.50 (101 steps) around the key's answer: it keeps the step, the stopping rule and `b07_unique`'s count | (i) |
| cuts | 2025-04-30, 2025-02-27 | capability (two knowledge dates are the question) | – | – |
| `windows each` (both Wrzos asks) | every window to the run end | scale, through the run end | follows the run end | (i) |

## K5 · `I:B:B-12`, B's paths (`L:5`, `L:14`)

| Knob | Full | Kind | Capability setting | Source |
|---|---|---|---|---|
| paths | 3 000 (about 5 min a path on snap8) | scale | 30 (`keys_b.py 30`: 5.5 s, relations pass) | (ii) `keys_b.py 30` or (i) statistical. The share figures have se; the mortgage and the zero-variance tax are deterministic |
| run end | 2028-12-31 | **scale**, down to 2027-01-31: that keeps the 2027 law draw (RAND-03, LAW-13, 12 % or 15 % from 2027), both mortgage resets (1 Jan, 1 Jul 2026), the 26th rule and the forecast rule. It also drops the Spring Day holes of 2027–2028 (`rows/B.check.md` line 150) after January | 2027-01-31 | **(iii):** `keys_b.py` has no run-end setting, and every B-12 figure is dated 2028-12-31 or sums to it. 4.5c adds a run-end knob to the key script, then (ii) |
| span start | 2022-03-01 | capability (the state at the cut 2026-01-02 is the history's) | – | – |
| paired runs | with and without overpayments (`no_overpay` scenario) | capability (common random numbers is a key relation) | – | – |

Even 1 path at full span is over budget (5 min CPython, about 75 s PyPy, estimated). So B-12's capability check exists only once the run end shrinks: (iii) until `keys_b.py` takes an end date.

## K6 · `B-cs_b09_b11` (`I:B:B-09`, `I:B:B-11`; `L:6`)

1 301 s for one run holding both questions' asks, on `cs11`, which is cut to 2026-12-31.

| Knob | Full | Kind | Capability setting | Source |
|---|---|---|---|---|
| asks per run | B-09's and B-11's | scale | one ask per check (4.5a) | (i) |
| run end | 2026-12-31 (`cs11`) | scale (both end 2025-12-31, known 2026-01-07) | 2026-01-31 | (i) |

## K7 · `I:E1:*`, the joint model's walks (`L:8`; E1-02-B 597 s)

| Knob | Full | Kind | Capability setting | Source |
|---|---|---|---|---|
| B inside E1 | B's whole text (a walk about 3 min on CPython) | scale if a stub states B's keyed figures as readings. E1's own rules (PTY-05's joins) all stay, and B's rules are B's own checks | the `scratch/mc-e1/cdB/` stub, B stubbed (the walk cells judged there) | (i): `keys_e1.py` builds on B's keys (it reads B-11's shape), so the stub's figures are the keys' |
| B's run end | 2028-12-31 | scale (E1 ends ≤ 2026-01-15) | 2026-01-31 | (i) |

## K8 · `I:A:A-05`, `windows each` for Ania (`L:9`)

| Knob | Full | Kind | Capability setting | Source |
|---|---|---|---|---|
| walk end | A's full text | scale (A-05 ends 2024-12-31, known 2025-04-30) | the walk stops at 2025-12 (22 s on snap8, every figure matching) | (i) |

## K9 · `I:A:A-12` under O:A-3's `one_of` forecast (`L:15`)

The ordinary A-q12 takes 133 s on CPython and 32 s on PyPy, so it is not heavy. O:A-3's copy reached 11 GB.

| Knob | Full | Kind | Capability setting | Source |
|---|---|---|---|---|
| `one_of` options | the calendar settled except one Monday in April | capability at 2 options (each option is a branch); scale above 2 | 2 | **(iii):** O:A-3 is a gap-log item with no key. It was judged on its minimal text `scratch/oa3/oa3.tally` |
| horizon | 2035-12-31 (A-12's date) | capability (the asked date) | – | – |
| asks | 8 (`q12`) | scale | one ask | (iii), as above |

## K10 · the amendments household's walk cells (`I:X:*@c*_r2`, `@b_r1*`, `explain_room`; `L:10`, `L:13`, `L:14` c1_r2)

| Knob | Full | Kind | Capability setting | Source |
|---|---|---|---|---|
| horizon | each cell's date. 170 cells lie past 2027-01-14; `pit28`, `health_settlement` and `ola_800_paid` walk at any horizon | capability per cell (the cell's date is its question); scale only past the change's own first date | per change, the run end at the first date its rules all occur: c1 2027-03-22, c2 2027-01, c3 the settlement 2028-05-22, c4 the claim 2028-06-20, c5 2029-07, c6 the overpayment 2028-01, c7 2027-01 | (i) for the cells `amendments/expected/` holds up to those dates. The cells past them are load (`L:10`). **(iii)** for any rule whose first cell isn't in the expected file; 4.5c lists them |
| span start | 2026-01-01 (the reality's) | capability (`on` and the openings) | – | – |

Caveat: the amendments' expected figures are the lead's hand arithmetic (`amendments/README.md`), and 12 cells of changes 4 and 5 are interpreter prints (`freeze/KEYS.md` rows 6–17). See `gaps.md` d.

## K11 · household-bands (`I:X:*@mc`; `L:11`, `L:14` mc)

| Knob | Full | Kind | Capability setting | Source |
|---|---|---|---|---|
| paths | 1 000 | scale | the cut-down's (`regress/mc-x.md` § 7) | test-mode checks (tolerance 0) and `base_values`: (i), they hold at any n (8 of 8). **Bands, shares, means and paired statistics: (iii)**: no key script for X, and the quantiles have no se |

## K12 · `I:M:LH-15` and `I:light/LH-15-light` (`L:12`)

| Knob | Full | Kind | Capability setting | Source |
|---|---|---|---|---|
| horizons | 30–60 years | scale | 1–4 years (the light cut; step 0..47) | the success rates are holes until a version supplies data (`ret`, CPI have no rows), so: holes (i); period counts (ii), the light's one-line checks (`regress/light/LH-15-light.md`) |
| cohorts | monthly from 1871-02 to 2015-12 | scale (the cells are holes either way) | as the light cut | (i) |
| axis combinations | all | scale | one pinned combination (`with`) | (i) |

The full model prints holes, so the published table cells can't be reached at any size. Both the load twin and the capability check test structure and holes, not ERN's figures. That is a fact about the case, not a knob.

## K13 · `I:M:LH-16` and `I:light/LH-16-light` (347 s; `L:12`)

The light twin is itself over 240 s, about 87 s on PyPy (estimated at ÷4), so it needs one more cut.

| Knob | Full / light | Kind | Capability setting | Source |
|---|---|---|---|---|
| payout periods (`hz_years`) | 15–30 / 1–4 | scale | 1–2 (step 0..24) | holes (i); the period counts (ii), a one-line count as in the light key's part (b) |
| success-rate asks | 72 / 6 | scale (the same construct at other keys) | 3: one era each, both withdrawal kinds | (i) |
| Table 6 asks | 11 / 6 | scale | panel A's `t6a_mean` and `t6_text` | (i) |
| `wr` rates | 10 per ask | scale | 2 | (i), holes |
| axes (`cpi_base` …) | all six with every option / pinned | capability is "every option exists", which the text keeps; pinning a value is scale | pinned, as light | (i) |

## K14 · `I:M:LH-17` (montecarlo `LH-17-mc`) and `I:light/LH-17-mc-light` (`L:12`)

| Knob | Full | Kind | Capability setting | Source |
|---|---|---|---|---|
| paths | 10 000 / 256 | scale. Detection power drops below 10 000: at 256, `chk_ruin` still catches a drift-sign error and unseeded paths, but no longer Eq. 8 used as the oracle (`regress.md` line 143) | 256 (144 s CPython, so under 60 s on PyPy is likely but unmeasured) | (i), statistical against the exact law (Yor 1992), as the light key does |
| steps | 2 700 (dt = 1/9 year, 300 years) | capability (the horizon is infinite for the endowment; ruin needs the long run) | – | – |

## Answer sources, in short

| Source | Checks |
|---|---|
| (i) only | K2, K3, K4, K6, K7, K8, K12, K14, and K10 up to each change's first date |
| (ii) | K5's path cut (`keys_b.py 30`); K12's and K13's period counts (the light's one-line checks) |
| **(iii)** | **K1:** D-10's 6 band cells below 10 000 paths. **K5:** B-12 whole, since it fits the budget only with a run end the key script lacks. **K9:** O:A-3's `one_of` run. **K10:** any change rule with no expected cell before its first date (unlisted). **K11:** household-bands' bands, shares, means and paired statistics |
