# Session 4.5c · C2's texts: the runs

Every run goes through the runner, one at a time:

```
python3 -I -B engine/tests/run.py --adhoc --cwd <cwd> -- run <recipe files…> <my files…> --ask <id> [--ask <id> …]
```

`<recipe files…>` are the files of `engine/tests/integration/recipes/<name>.txt` in that file's order (its `cwd` line is the cwd). My files go after them, in the order given. All paths below are under `.`. Seconds and rc are the runner's `# rc` line (frozen interpreter, PyPy, 60 s limit). Ask ids not listed in a run must not be added to it: the B runs sit at about 45 s with one ask.

---

## A-13 · Had Ania stopped working

- **cwd** `engine/tests/integration/cuts/A`; recipe `A.txt`, then:
  1. `engine/tests/integration/new/A-13/a13_calc.tally`
  2. `engine/tests/integration/new/A-13/a13_jul.tally`
  3. `engine/tests/integration/new/A-13/a13_apr.tally`
  4. `engine/tests/integration/new/A-13/a13_jul_sale.tally`
  5. `engine/tests/integration/new/A-13/A-13.tally`
- **Worlds.** `a13_calc` (over `lis`) is the control layer: Kalina's and the SIO's 2023 are computed from the payroll in place of their yearly information, and it holds the rules that follow from the stop day `a13_stop` (no pay from that day, so no plan contributions; SOC-8 .1, no parental pay unless insured as an employee on the birth day). There she never stops. `a13_jul` (a), `a13_apr` (c) state the day. `a13_jul_sale` (b) sits over `a13_jul`. "differs by" is the scenario's figure less the base world `lis`'s (a payment is plus, a refund minus).

| run | asks | s | rc |
|---|---|---|---|
| A1 | `a13_a a13_a_differs a13_a_differs_ania a13_a_why_paid a13_a_why_income a13_a_why_advances` | 7.4 | 0 |
| A2 | `a13_c a13_c_differs a13_c_differs_ania a13_c_why_paid a13_c_why_income a13_c_why_advances` | 7.4 | 0 |
| A3 | `a13_control a13_control_costs a13_control_soc a13_control_advances a13_why_kalina_due a13_a_why_parental a13_c_why_parental` | 6.4 | 0 |
| A4 | `a13_b_sale a13_b_lots a13_b_tax a13_b_differs_2023 a13_b_differs_2024_plain` | 12.6 | 0 |
| A5 | `a13_b_differs_2024` | over 60 s | killed |

| ask | answers |
|---|---|
| `a13_a` | (a): joint tax, Ania, Tomek, together, child relief (in `a13_jul`) |
| `a13_a_differs` | (a) differs by: those five |
| `a13_a_differs_ania` | (a) differs by: Ania's income, Ania's base, parents' income, Ania's advances |
| `a13_a_why_paid`, `a13_a_why_income`, `a13_a_why_advances` | (a) why: per payer and payment month of 2023, gross paid, income (with the plan contributions), advance, `a13_jul` against the control |
| `a13_why_kalina_due` | (a), (c) why: Kalina's pay due per month of 2023, actual, (a), (c) |
| `a13_a_why_parental`, `a13_c_why_parental` | (a), (c) why: explain of June's parental pay, showing SOC-8 .1's rule not applying in (a) and winning in (c) |
| `a13_c`, `a13_c_differs`, `a13_c_differs_ania`, `a13_c_why_*` | (c), the same labels |
| `a13_control*` | check of the method: the control layer against `lis`, per payer, for 2023 income, costs, contributions and advances (all 0 changed: the payroll gives the yearly information's figures) |
| `a13_b_sale` | (b): the sale at DM Kopia (day, units, money, fee) |
| `a13_b_lots` | (b) lots: bought (month), units, gain, taxed |
| `a13_b_tax` | (b): Ania's fund-sale tax for 2023 and 2024 in `a13_jul_sale` |
| `a13_b_differs_2023` | (b): 2023 differs by |
| `a13_b_differs_2024` | (b): 2024 differs by, `windows each` (the actual 2024 sale meets the March 2019 lot's month) — **over 60 s** |
| `a13_b_differs_2024_plain` | (b): the same line without `windows each`, which prints the `HOLE-IMPRECISE` naming that window |

**Over 60 s:** `a13_b_differs_2024`. Its cost is the actual 2024 tax with `windows each`: the judged ask `a05_ania` (A-05) alone is over 60 s in this recipe too. The scenario's own 2024 figure (`a13_b_tax`) is fast.

**Awkward rows and gaps**
- `A-13/a13_calc.tally:3` — the scenarios diverge from 2023-01-01, not from the stop day: `info` is keyed by the tax year, and a scenario row may govern no day before its `from`.
- `A-13/a13_calc.tally:15-16` — the yearly information of the changed payers is replaced by `none` so their year is computed from the payroll; the control layer (A3) is what shows that method changes nothing by itself.
- `A-13/a13_calc.tally:19` — SOC-8 .1 (insured as an employee on the birth day) isn't in the judged payroll, which reads the SIO's decision; it's added as a scenario rule citing the question's version, since a `replace rule` may cite only its own version (it can't point `v` at `soc8_v1`).
- `A-13/a13_calc.tally:21-22` — T-1 fix: the judged `res_joint` and `together_joint` (`households/A/tax.tally:83-101`) floor each spouse's part less relief at 0 and sum the floored lines; PIT-6 .4, .5 and PIT-7 .5 put the floor on the joint tax less both reliefs, then take both advances (together), and give each spouse their part less their own relief and advances with no floor. Two `replace rule` items in the scenario layer (`a13_law`, beating the judged rules) write it so in every A-13 world; the judged text is untouched. The floor binds only in (c), so (a) and the actual 2023 agree under both readings, and in (c) Ania plus Tomek no longer sums to together, as the law reads. Awkward: the base world `lis` keeps the judged rules, so "differs by" compares the law's reading in the scenario with the judged reading in the base, which agree for 2023 only because the floor doesn't bind there.
- `A-13/a13_calc.tally:17` — a stop inside a month would need A-D05 §7's day count: written as a hole rule, not reached by either day.
- `A-13/a13_jul_sale.tally:28,59` — the hypothetical sale is a planned move on a day the DM Kopia register covers. A planned movement there is modelled past, never booked, and the interpreter left it out of the `fund_sales` binding too, so the lots fill didn't see it. Fixed by shrinking the register's coverage to the day before the sale (`moves kop_register to …`, in its own version known 2023-09-13: `LOAD-MOVES` refuses a `moves` naming a version known after its own).
- `A-13/a13_jul_sale.tally:53` — the gifted lots at DM Kopia (`gifts_in`) aren't reached by the base's PIT-8 sums, which take only `fund_buys` and `lot_transfers`; `net_gain` gets a rule that restates the base rule's two sums and adds the third (R-once strained).
- `A-13/a13_jul_sale.tally:49` — the BRZ-MM-A purchase of 2024-03-07 is removed with the sale whose proceeds paid for it; what she does with the December money isn't asked (it sits in `kop_cash`).
- `A-13/a13_jul_sale.tally:66` — price times units needs a rounding mode to become money; the question names none: an `assumed` input (no mode changes a 2-place price times whole units).
- The gifted lot's cost repeats `gift_cost`'s workaround: a price can't be keyed by a leg's unit, so it's the gift-day value of the holding in the lot's share of its units.

---

## B-13 · Had the floren been 5 % cheaper

- **cwd** `engine/tests/integration/cuts/B/cs11`; recipe `B.txt`, then:
  1. `engine/tests/integration/new/B-13/b13_rows.tally`
  2. `engine/tests/integration/new/B-13/b13_fx.tally`
  3. `engine/tests/integration/new/B-13/b13_rule.tally`
  4. `engine/tests/integration/new/B-13/B-13.tally`
- **Worlds.** `b13_rows.tally` is a base part adding the real rates of the rule's days (gazette FX-FLR and Lazur buy rates the judged text didn't hold). `b13_fx` (a) replaces every floren rate the reality holds from 2025-07-01 by an expression row: the base's own row times 0.95, rounded to 4 places half up. `b13_rule` (b) is over `b13_fx` and holds B-13 rule 1 only.

| run | asks | s | rc |
|---|---|---|---|
| B1 | `b13_a_invoices b13_a_lazur` | 45.2 | 0 |
| B2 | `b13_b_sales` | 44.2 | 0 |
| B3 | `b13_b_days` | 45.0 | 0 |
| B4 | `b13_b_lazur` | 43.8 | 0 |
| B5 | `b13_b_gain` | 45.5 | 0 |

| ask | answers |
|---|---|
| `b13_a_invoices` | (a) invoices: revenue and VAT of each Kessel invoice issued from 2025-07-01 |
| `b13_a_lazur` | (a) Lazur at 2025-12-31: FLR and DUK (B-HR4 .5's mid rate, as B-03) |
| `b13_b_sales` | (b) sales: day, FLR sold, DUK, credited |
| `b13_b_days` | (b) support: each rule day with the booked Lazur balance and what the rule sells |
| `b13_b_lazur` | (b) Lazur at 2025-12-31: FLR and DUK |
| `b13_b_gain` | (b) currency gain or loss of the rule's sales (B-HR4 .5) |

**Over 60 s:** none alone. Any two of the (b) asks in one run go over (the recipe's load is about 41 s).

**Awkward rows and gaps**
- `B-13/b13_rule.tally:34` (and the file's second note) — B-13 rule 1 is a decision, but the Lazur and Cis exports cover the whole second half of 2025, so a decision there is modelled past: computed against the booked balances, never booked, and a second sale wouldn't see the first. Written as defs over the booked Lazur balance less the rule's earlier sales. So the sales move nothing: Cis and B-HR2's count of the Lazur florens on the 26th don't see them (not asked).
- `B-13/b13_rule.tally:8` — the rule's months are a scenario index 2025-07..2025-12, the question's half-year; the rule itself has no end.
- `B-13/b13_fx.tally:17` onward — one expression row per day the reality holds a rate for: a `*` row can't name its key in its expression, and an open-span row over the keyless `fx_mid` would be read by `last()` on days with no published rate.
- `B-13/b13_fx.tally:6` — known 2025-06-30, not each rate's own day: a replace known the same day as the row it replaces is refused by the reference interpreter (fault F5).
- `B-13/b13_rows.tally:5,41` — rates for the rule's days added as base facts, since the judged text held only the days it read.

---

## D-13 · Had the fund been taxed

- **cwd** `engine/tests/integration/cuts/D/p200_11`; recipe `D-p200.txt`, then:
  1. `engine/tests/integration/new/D-13/longview_taxed.tally`
  2. `engine/tests/integration/new/D-13/D-13.tally`
- **Paths: 200**, seed 20261001: the 200-path copy. The question names D-10's number of paths, and D-10's own judged run is 10 000 paths; the text is written at 200, to be judged statistically. Same random facts as D-10 (`life_u`, `zr`), so the same samples.

| run | asks | s | rc |
|---|---|---|---|
| D1 | `d13_paths` | 25.9 | 0 |
| D2 | `d13_zero d13_stream` | 4.1 | 0 |

| ask | answers |
|---|---|
| `d13_paths` | gains tax (mean of the PIT-8 tax on units sold in years she lives, summed to her death); gains (mean, each path's gains to the ort); share selling units (paths with a sale in a year she lives) |
| `d13_zero` | zero volatility: first sale in; gains closed form and gains tax closed form, rounded to the ort and exact |
| `d13_stream` | the hole for Python's `random.Random` stream (the judged `py_stream`) |

**Over 60 s:** none. A second reading of "share selling units" (a sale in any year to 2064, lived or not) went over 60 s and was dropped: the text reads the question's "while she lives".

**Awkward rows and gaps**
- `D-13/longview_taxed.tally:81` — PIT-8 .3's first in, first out over 38 yearly lots is written as the overlap of each lot's run of units with each year's run of units sold, a def over (lot, year) pairs: there's no lot construct for units that aren't movements.
- `D-13/longview_taxed.tally:108` — the exact gains of 200 paths have denominators whose mean passes the 4 096-digit cap (`RUN-DIGITS`); each path's gains are rounded to the ort for the mean, mode the household's axis `hh_round` (no source rounds them).
- `D-13/longview_taxed.tally:130` onward — the zero-volatility chain (`z_` defs) is a copy of the path chain with every R at 5 %: `t_tax[y] as if {r_path[*] = fund_mu}` stays random to load (`LOAD-RANDOM`), so a closed form can't read it (D-10 does the same with `zv_`).
- `D-13/longview_taxed.tally:12` — PIT-8 .5's five carry years as an index `int 1..5` beside the row `pit8_carry_years`; the rule reads the row too.
- `D-13/longview_taxed.tally:104,216` — a unit's worth with no unit held is a hole (the question defines worth as value over units held); no path reached it.
- PIT-8 .6's 5-year exemption and PIT-8 .5's loss carry apply ("as the law taxes fund units"); the gains tax mean comes out 0 with standard error 0 on these paths, and 0 in closed form.

---

## CUR-01 × RAND-01 · A drawn rate on the law's leg

- **cwd** `engine/tests/integration/new/probes/pairs`; files: `engine/tests/integration/new/probes/pairs/CUR-01+RAND-01.tally` (standalone).

| run | asks | s | rc |
|---|---|---|---|
| P1 | `q_days q_python_stream q_means` | 27.6 | 0 |
| P2 | `q_means_again` | 28.0 | 0 |

| ask | answers |
|---|---|
| `q_days` | the law's day (last business day before the invoice) and the day paid |
| `q_means` | mean revenue and mean cash received, each with its standard error, 20 000 paths, seed 20260301 |
| `q_means_again` | the same line again: same seed, same answer (compare P1 and P2) |
| `q_python_stream` | the hole for Python's `random.Random(20260301)` stream |

**Over 60 s:** none alone; `q_means` and `q_means_again` together would be about 56 s, so they run apart.

**Awkward rows and gaps**
- `probes/pairs/CUR-01+RAND-01.tally:50` — rule 4's Python stream is no row a text can give (L 5.16): a hole def; the draws are the engine's own under the card's seed and path count.
- `probes/pairs/CUR-01+RAND-01.tally:58` — `binomial(1, 1/2)` gives 1 for a uniform above 1/2, where the card's stream calls below 0.5 up: the same law, not the same draws.
- `probes/pairs/CUR-01+RAND-01.tally:37` — `cash` is a built-in id (the prelude kind), refused as a def name (`LOAD-DUP`): `cash_received`.

---

## LAW-20 · A circular rule the law solves by its own named method

- **cwd** `engine/tests/integration/new/probes/law`; files: `engine/tests/integration/new/probes/law/LAW-20.tally` (standalone).

| run | asks | s | rc |
|---|---|---|---|
| L1 | `a_rule3 a_closed_form_agrees a_closed_form_deduction b_rule4 b_rounds b_fixed_point` | 0.5 | 0 |

| ask | answers |
|---|---|
| `a_rule3` | (a): rule 3's deduction and credit |
| `a_closed_form_agrees`, `a_closed_form_deduction` | (a): check against the card's closed form, credit and deduction |
| `b_rule4` | (b): rounds taken, the deduction and the credit it ends at |
| `b_rounds` | (b): every round's deduction and credit |
| `b_fixed_point` | (b): the point where rules 1 and 2 hold at once, and whether rule 4's deduction ends within the tolerance of it |

**Over 60 s:** none.

**Awkward rows and gaps**
- `probes/law/LAW-20.tally:12` — rule 4 has no bound on its rounds; the iteration index is 0..50, the text's own bound (`first` refuses if none stops it).
- `probes/law/LAW-20.tally:25` (and every def with a note) — a short def can't carry a note (`LOAD-PARSE`), so commented one-line defs take the long form with a `base` rule.
- Rule 3 is read as keys 0 and 1 of the same iteration as rule 4 (rule 2 is one keyed def), since there are no user functions to apply rule 2 to two deductions.
