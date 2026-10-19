# CMP-01 expected: eleven people from three examples

Model: `models/CMP-01.tally`. Case: `tmp/verification/cases/CMP-01.md`.

The model is the union of the three examples the case composes: MP-09 (the Düsseldorfer Tabelle Teil C Beispiel), the § 40 Exemple of MP-11 (`foyer = [ex40]`) and example (27) of MP-15, plus the NBP rate, the brochure row and the composition's facts. The published figures are the original examples' results and must stay the same. The composed questions 1 to 6 have no published answer, so their figures are derived with one-liners (below), as the case's composed questions define them.

**Readings.**
- `dt_u_work = employed` for every German figure and for every composed figure that reads the maintenance. The example prints the 1.450,00 EUR Selbstbehalt, which Anm. A VII gives for a payer in work (case "Missing facts", first bullet). The `not_employed` line is not a case figure, so it isn't worked here.
- `dt_cut_round = *`: both options give 107,60, 105,02 and 87,38 (worked in `expected/MP-09.md`).
- `unmarried[k1] = true` is an `assumed` row (case "Missing facts", third bullet). It is in the register of every maintenance figure, and of every composed figure that reads one.
- `fr_comp_order = *`: `comp_halves[ex40] = 0`, so both orders give the same figures.
- `cmp_spouses`: the case doesn't say which French spouse is friend 2 (case "Missing facts", "Composition: which of the French spouses is friend 2"). Figures for `fr_a` and `fr_b` print one line per option. Every other figure prints `*`.
- Law dates: the German defs at the ask's cut (2026-10-18) when an ask reads them, and at the pays' law date, 2026-01-01, when a pay does: the first day of the imprecise `on` 2026-01 (LANGUAGE §4.3). Both are inside `dt2026` (2026-01..) and the Kindergeld row (2026). The French defs at `start_of(income_year[ex40])` = 2025-01-01, inside `lf2026_art4` and `bareme2025` (both govern 2025).
- Barème: `part_tax[ex40] = marginal(bareme, qf[ex40])` (LANGUAGE §5.8), as in `expected/MP-11.md`. The second term is `tax[ex40] as if {parts[ex40] = base_parts}`, so `q_qf`, `q_tax_raw` and `q_tax` print two values: the first at 5 parts, the second as if 2 parts.
- Known dates (LANGUAGE §4.5), none a correction:
  - law from its publication: `bgb_2026` 2017-07-21 (the later of its two promulgations), `minuhv_2026` 2024-11-21 (BGBl. 2024 I Nr. 359), `estg66` 2024-12-30 (BGBl. 2024 I Nr. 449), `lf2026_art4` 2026-02-20 (Journal officiel);
  - sources by their own date: `verhoeff_2004` 2004-04-15, `arrondi2018` 2018-07-04, `dt2026` 2025-12-01 (the OLG's publication), the NBP versions 2025-12-31, `bareme2025` and `qf2026` 2026-04-07;
  - `brochure_ir2026` 2026-03-25, the PDF's updated stamp (its Last-Modified), since it prints no date;
  - the stipulations, from the model's `start` 2026-01-01: `cmp01_composition`, `dt2026_example`, `examples2026`, `verhoeff_27` (the debt matrix), and the assumption `mp09_assumed`, which stands for a fact of the example.
- Booking (ENGINE §5): each pay is booked at κ₀ = its January 2026 date. Every version its amount and `on` read (`dt2026`, `bgb_2026`, `dt2026_example`, `mp09_assumed`, `verhoeff_27`, `cmp01_composition`) is known by 2026-01-01. The versions known later (`lf2026_art4`, `brochure_ir2026`, `bareme2025`, `qf2026`) touch no pay's read set. So no `arrears` line is needed, no `HOLE-CHANGED` arises, and the booked amounts are the figures below.
- Flows asks: `q_c4_flows_check`, `q_c5_net` and `q_c5_sum` read `net_flow`, a def no pay reaches (LANGUAGE §5.13.1), over `flow_month` = 2026-01. They pin nothing: the timeline is per axis option (ENGINE §3.3), so they print one line per option that changes a value. The case's figures are the `employed` line. The `not_employed` lines are not case figures and are not worked, except where noted.
- Currency: the debts are money PLN (the composition reads them as złoty). Money prints with two decimals: the published "9" prints `9.00`, and "−1" prints `-1.00`. EUR figures print to the cent: 1.450,00 prints `1450.00` and 26 000 € prints `26000.00`.
- Rate: `eur_pln` = 4.2267 PLN per EUR (NBP table 251/A/NBP/2025 of 2025-12-31). Each conversion is rounded half up to the grosz, as questions 1 and 2 set.
- Signs:
  - Question 3's "difference" is Verhoeff's b = p − r, positive for an adult who must pay.
  - Question 4's "net outflow" is positive for money paid out: the maintenance converted payment by payment, plus U's b.
  - Question 5's "net flow" is `flows`: money in minus money out, so a payer is negative.
  - Only the maintenance payments and the debts of (27) are pays. The French tax and E's Kindergeld are not pays, so they are not flows (case "Missing facts", "whether the French tax … counts as flows in question 5").

## The original examples

| Ask | Keys | Case locator | Figure as stated | Reading |
|---|---|---|---|---|
| `q_eigenbedarf` | — | Published answer, German example, row "Notwendiger Eigenbedarf des U" | 1.450,00 EUR | `employed` |
| `q_masse` | — | German example, row "Verteilungsmasse": "1.750 EUR – 1.450 EUR = 300,00 EUR" | 300,00 EUR | `employed` |
| `q_einsatz` | k1 | German example, row "Einsatzbetrag K 1": "(698 – 259,00) 439,00 EUR" | 439,00 EUR | `a18`, full Kindergeld (rule `zahlbetrag.adult`) |
| `q_einsatz` | k2 | row "Einsatzbetrag K 2": "(558 – 129,50) 428,50 EUR" | 428,50 EUR | `a6_11`, half Kindergeld |
| `q_einsatz` | k3 | row "Einsatzbetrag K 3": "(486 – 129,50) 356,50 EUR" | 356,50 EUR | `a0_5`, half Kindergeld |
| `q_einsatz_sum` | — | row "Summe": "= 1.224,00 EUR" | 1.224,00 EUR | all three first-rank, K1 through the assumed row |
| `q_unterhalt` | k1 | row "Unterhalt K1": "439,00 x 300 : 1.224 = 107,60 EUR" | 107,60 EUR | Mangelfall; `employed`; `dt_cut_round = *` |
| `q_unterhalt` | k2 | row "Unterhalt K2" | 105,02 EUR | as above |
| `q_unterhalt` | k3 | row "Unterhalt K3" | 87,38 EUR | as above |
| `q_split_check` | — | Transcription notes, DE arithmetic checks: `print(D('107.60')+D('105.02')+D('87.38'))` → `300.00` | ok | `employed` |
| `q_zahlbetrag` | g1, a0_5 / a6_11 / a12_17 / a18 | Law references, German, [0] p. 6 Anhang row 1: "1. bis 2.100 356,50 428,50 523,50 439,00 100" | 356,50, 428,50, 523,50, 439,00 | group `g1` |
| `q_kindergeld_check` | — | Inputs, row "DE: Kindergeld per child": [0] p. 6 Anhang and [11] § 66 Abs. 1 EStG | ok (259,00 = 259) | — |
| `q_minuhv_check` | a0_5, a6_11, a12_17 | Inputs, rows "table amount, income group 1": 486, 558 ([0] p. 1 and [10] MinUhV § 1 Nr. 1, 2); Law references, MinUhV § 1: "653 Euro ab dem 1. Januar 2026" | ok, ok, ok | `minuhv_2026` governs 2026-01.. |
| `q_qf` | ex40, first value | Published answer, French example, row "1/ quotient familial, 5 parts": "130 000 € / 5 = 26 000 €" | 26 000 € | exact |
| `q_qf` | ex40, as if 2 parts | row "2/ quotient familial, 2 parts" | 65 000 € | exact |
| `q_tax_raw` | ex40, first value | row "1/ impôt, 5 parts (premier terme)": "soit 7 920 €" | 7 920 € | `marginal` on the exact quotient 26 000 (case check, tranche by tranche → `7920.00`) |
| `q_tax_raw` | ex40, as if 2 parts | row "2/ impôt, 2 parts": "soit 25 207,98" | 25 207,98 | before rounding |
| `q_tax` | ex40, first value | row "1/ impôt, 5 parts" | 7 920 € | round to 1, half up (no change) |
| `q_tax` | ex40, as if 2 parts | row "2/ impôt, 2 parts": "arrondi à 25 208 €" | 25 208 € | round to 1, half up |
| `q_extra_halves` | ex40 | row "2/ plafond of the 6 extra demi-parts": "(5 parts - 2 parts = 3 parts), soit 1 807 € x 6" | 6 | (5 − 2) × 2 |
| `q_benefit` | ex40 | same row: "1 807 € x 6 = 10 842 €" | 10 842 € | `fr_comp_order = *` |
| `q_second_term` | ex40 | row "2/ impôt avec plafonnement (second terme)": "25 208 € - 10 842 € = 14 366 €" | 14 366 € | `*` |
| `q_capped` | ex40 | row "3/ tax due": "Le premier terme étant inférieur au second, le plafonnement est applicable." | true (stated in words) | 7 920 < 14 366 |
| `q_tax_due` | ex40 | row "3/ tax due": "soit 14 366 €" | 14 366 € | `*` |
| `q_brochure_2parts` | — | Published answer, Independent check: [7] p. 378, row "130000": "25208" for 2 parts | ok (25 208 = 25208) | — |
| `q_brochure_5parts` | — | same: "14366" for 5 parts | ok (14 366 = 14366) | the brochure includes the plafonnement and the décote; the décote doesn't apply at this tax |
| `q_owes` | f0, f1, f2, f3 | Published answer, Debts example, (27), column pi | 9, 6, 6, 3 | (5) |
| `q_owed` | f0, f1, f2, f3 | (27), row rj | 3 7 8 6 | (6) |
| `q_balance` | f0 | (27), balance column: "9−3 = +6" | +6 | (8); prints 6.00 |
| `q_balance` | f1 | "6−7 = −1" | −1 | prints -1.00 |
| `q_balance` | f2 | "6−8 = −2" | −2 | prints -2.00 |
| `q_balance` | f3 | "3−6 = −3" | −3 | prints -3.00 |
| `q_balance_sum` | — | Law references, Debts, [2] p. 108 (10)–(11): "the grand total balance equals zero"; Transcription notes, Debts check `print(9-3, 6-7, 6-8, 3-6, (9-3)+(6-7)+(6-8)+(3-6))` → `6 -1 -2 -3 0` | 0 | prints 0.00 |
| `q_nbp_table_check` | — | Inputs, row "FX: EUR average rate": JSON "mid":4.2267 ([3] and [5]); Transcription notes, NBP | ok | — |
| `q_nbp_xml_check` | — | same row: XML "<kurs_sredni>4,2267</kurs_sredni>" with "<przelicznik>1</przelicznik>" ([4]) | ok | — |

The case's checks for the debts (Transcription notes, Debts): `print(0+3+0+6, 1+0+5+0, 2+4+0+0, 0+0+3+0)` → `9 6 6 3`; `print(0+1+2+0, 3+0+4+0, 0+5+0+3, 6+0+0+0)` → `3 7 8 6`.

## The composed questions (derived; no published answer)

| Ask | Keys | Case locator | Figure (derived) | Reading |
|---|---|---|---|---|
| `q_c1_maint_pln` | k1, k2, k3 | Story, "What is asked", question 1: "each rounded half up to the grosz" | 454.79, 443.89, 369.33 PLN | `employed`; 107,60, 105,02 and 87,38 × 4.2267, each half up to 0.01 |
| `q_c1_totals` | — | question 1: "the sum of those three złoty amounts; the total maintenance in euros converted at once and rounded the same way; and the difference" | 1268.01, 1268.01, 0.00 PLN | the difference isn't rounded: both totals are already in grosz (case "Missing facts", last bullet) |
| `q_c2_tax_pln` | — | question 2 | 60720.77 PLN | 14 366 × 4.2267 = 60 720.7722, half up |
| `q_c3_friends` | f0, f1, f2, f3 | Inputs, row "Composition: friends" | f0 de_u, f1 de_e (both `*`); f2 fr_a and f3 fr_b under `a_is_f2`, f2 fr_b and f3 fr_a under `b_is_f2` | `cmp_spouses` |
| `q_c3_adults` | de_u | question 3 | owes 9.00, owed 3.00, difference 6.00 | friend 0; `cmp_spouses = *` |
| `q_c3_adults` | de_e | question 3 | 6.00, 7.00, -1.00 | friend 1; `*` |
| `q_c3_adults` | fr_a | question 3 | `a_is_f2`: 6.00, 8.00, -2.00; `b_is_f2`: 3.00, 6.00, -3.00 | friend 2 or friend 3 |
| `q_c3_adults` | fr_b | question 3 | `a_is_f2`: 3.00, 6.00, -3.00; `b_is_f2`: 6.00, 8.00, -2.00 | friend 3 or friend 2 |
| `q_c4_outflow` | — | question 4: "the maintenance of question 1 (converted payment by payment) plus that parent's net balance from question 3" | 1274.01 PLN | `employed`; 1268.01 + 6.00 |
| `q_c4_flows_check` | — | question 4, read from the booked pays | ok, under every option | `-net_flow[de_u]`: minus U's January flows on `pln[de_u]` and on `eur[de_u]` (converted movement by movement) equals `payer_outflow`. Both sides read the same maintenance, so it holds on the `not_employed` line too |
| `q_c5_net` | de_u | question 5: "each person's net flow in złoty" | -1274.01 | `employed`; `dt_cut_round = *`; `cmp_spouses = *`. Flows in − out, euro movements converted one by one |
| `q_c5_net` | de_e | question 5 | 1.00 | `*` on every axis: E receives no maintenance |
| `q_c5_net` | de_k1, de_k2, de_k3 | question 5 | 454.79, 443.89, 369.33 | `employed`; `dt_cut_round = *`; `cmp_spouses = *` |
| `q_c5_net` | fr_a | question 5 | `a_is_f2`: 2.00; `b_is_f2`: 3.00 | friend 2 or friend 3; `*` on the other axes |
| `q_c5_net` | fr_b | question 5 | `a_is_f2`: 3.00; `b_is_f2`: 2.00 | friend 3 or friend 2 |
| `q_c5_net` | fr_c1, fr_c2, fr_c3, fr_c4 | question 5 | 0.00 | `*`: no pay reaches them |
| `q_c5_sum` | — | question 5: "the sum over all eleven people of each person's net flow in złoty" | 0.00, under every option | every pay names both ends, and each converted movement is rounded before its direction is applied, so the two ends cancel (LANGUAGE §5.13.3) |
| `q_c6_b0` … `q_c6_b3` | f0 … f3 | question 6: "the facts it depends on, and whether any fact from another example enters it" | balance[fN]: the six `owe` rows of row N and column N, all of version `verhoeff_27` (source `verhoeff`). No other example's fact | — |
| `q_c6_k1` … `q_c6_k3` | k1 … k3 | question 6 | unterhalt[kN] (each of the three reaches every child, through the Mangelfall test and the proportions): `income`, `child_age[k1..k3]`, `in_school[k1]`, `lives_with_parent[k1]`, `dt_amount[g1, a0_5 / a6_11 / a18]`, `kindergeld`, `selbstbehalt[employed]`, the band rows `dt_group` g1 and `dt_age` a0_5, a6_11, a18 (versions `dt2026`, `dt2026_example`), and `unmarried[k1]` (`mp09_assumed`, assumed). No other example's fact. Under `not_employed` the set has `selbstbehalt[not_employed]` instead | `dt_cut_round = *` |
| `q_c6_tax` | ex40 | question 6 | tax_due[ex40]: `fr_income[ex40]`, `parts[ex40]`, `income_year[ex40]`, `comp_halves[ex40]` (`examples2026`), `base_parts` (`qf2026`), `cap_half`, `cap_comp` and the `bareme` rows `[0, 11600]`, `(11600, 29579]` and `(29579, 84577]` (`lf2026_art4`): the 26 000 quotient reaches the first two, the 65 000 quotient of the second term the third. No other example's fact | `fr_comp_order = *` |

None of the three sets reaches `cmp01_composition`, the NBP rows or the brochure, and none reaches the `bareme` rows above 84 577 €. The composition's facts enter only the composed figures. The rules cite `dt2026`, `bgb_2026`, `qf2026`, `bareme2025`, `arrondi2018` and `verhoeff_2004`. `explain` shows them, and `inputs-of` doesn't.

**One-liners.**

Question 1: `python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; r=D('4.2267'); u=[D('107.60'),D('105.02'),D('87.38')]; x=[(a*r).quantize(D('0.01'),H) for a in u]; print([a*r for a in u], x, sum(x), sum(u)*r, (sum(u)*r).quantize(D('0.01'),H), sum(x)-(sum(u)*r).quantize(D('0.01'),H))"` → `[Decimal('454.792920'), Decimal('443.888034'), Decimal('369.329046')] [Decimal('454.79'), Decimal('443.89'), Decimal('369.33')] 1268.01 1268.010000 1268.01 0.00`

Question 2: `python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; print(D('14366')*D('4.2267'), (D('14366')*D('4.2267')).quantize(D('0.01'),H))"` → `60720.7722 60720.77`

Question 3 uses the case's own row and column sums and its b check (above).

Question 4: `python3 -c "from decimal import Decimal as D; print(D('1268.01')+(9-3))"` → `1274.01`

Question 5 (U's złoty in − out, then E, friend 2 and friend 3, then the sum of all eleven): `python3 -c "from decimal import Decimal as D; print(-D('1268.01')+(3-9), 7-6, 8-6, 6-3, -D('1268.01')+(3-9)+(7-6)+D('454.79')+D('443.89')+D('369.33')+(8-6)+(6-3))"` → `-1274.01 1 2 3 0.00`
