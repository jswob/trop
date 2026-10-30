# W-A progress (session 3, household A)
Brief: /private/tmp/claude-501/-Users-jswob-projects-trop/2b7f7638-5a75-4bdf-aa64-86242ca7cb30/scratchpad/W-A.prompt
law dir was empty at start (I am first writer).

## Done
- (nothing yet)

## Next
- read dossier: story, house-rules, questions, docs, keys, acts, LANGUAGE.md

## Read so far (don't re-read whole)
- brief, story, house-rules, questions, all 12 keys, all docs, acts.md, LANGUAGE.md whole (2026-10-25 session).
- key spec facts: grammar L §2.3 (lines 119-222); literal rule L §7.1 (ints 0-31,100,360,365,366,1000 only in defs);
  read rule L §4.6; def/rules L §5.10; pay/state L §5.13; records L §5.18 (movement input, open index, covers, stated);
  ask L §5.16 (as known, with pins, axes fan out L §5.5, real() ask-only); scenarios L §6.4; holes L §8.1.

## Design (settled before compaction 1)
- files: households/A/reality.tally (model lis, tally 1, on 2026-01-15, start 2021-01-01, end 2035-12-31) + part files; scenarios/ for A-12.
- law files law/<rule-id>.tally (PIT-2..9, SOC-1/2/8, FAM-1, IGT-*, CC-1/2, MOT-1/2, GEN-1 holidays+HOL-SPRING, GEN-3, CIV-1, WCP-1/2 as needed).
- PIT-3 band versions v1 (2018-2021, known 2017-09-01), v2 (2022, known 2021-11-15), v3 (2022.., known 2022-06-14, enters 2022-07-01); advances read via `wording <payday> for [pit3_scale]`; A-01 cause via as-if wording 2022-06-30; A-02 as known 2022-05-31.
- payroll per payer x month of payment; SOC-1 13.45% (11% capped at SOC-CAP on yearly gross per employer, 2.45% uncapped); health 9% of (gross-soc); costs 250; base rounded; advance cumulative scale - 1/12 reducing.
- A-01 axis E/N 2021 readings; A-04 axis posted/received; A-05 lots few rows (awkward), March 2019 lot imprecise -> HOLE-IMPRECISE; A-07 axis T/B; A-08 scenario can't remove covered records -> defs (awkward); A-10 refusal via `?` row; A-11 partial -> second ask; A-12 case suspect (January 2026 investing), variant pinning January to 0, holidays 2027-35 assumed.
- O:A-1 CSV records not readable as they stand (OPERATIONS §2.2 dialect unspecified).
- gazette lines: WRAP-LIMIT 1985-1993; IGT-FREE-I 1596/1597; SOC-CAP 1934-1938; HOL-SPRING 1590-1595.

## After compaction 1 (re-read keys, docs, spec 4.x/5.x/6.x/7.1/8.1 again)
- law files = figures, bands, axes, versions only (no household defs); household defs point at law versions with `v`.
- payroll verified: Nurt pay for k paid 10th of k+1 (15th from Dec-2024 pay), er plan 2.5% paid on next payday (period k+2), ee 3%; Kalina paid last bd of k, er 1.5% paid 15th k+1, ee 2%; SIO day pay 123.44 x leave days of k paid by 10th k+1.
- A-01 uses yearly-info rows (info record input); payroll calc for A-02/A-03/A-11 and the no-cut scenario (rule `nocut excepts doc v nocut` points at scenario version).
- PIT-6 .5 parts: floor shares + leftover to larger base (not prorata rest largest).
- A-10 case suspect (Zosia 500 doesn't depend on order); A-12 case suspect (January 2026 amount); A-11 without (−213) unexplained.
- O ideas: unit records impossible (movement CUR only); holiday list composite (fixed days + notice); CSV dialects.
- market prices: GLB 2023-09-12 176.40 l.1447; 2025-12-31 (line 2027): GLB 230.56, BND 138.99, DIV 71.02, MM 129.18; IDX 254.7653 l.2028; WCP 134.2798 l.1650.
- records lines: sale 41500 aug l.32, payoff aug l.34, commission sep l.10, insurance sep l.20, road tax oct-2024 l.15; lazur 2000 dec-2024 l.30; lazur 1500 oct-2025 l.14; joint close dec-2025 l.45 11579.98; PIT cap 2025-04 l.47/48; laz last line 41 24223.47; ania sav last 39162.33 (l.21).

## After compaction 2
- law files ALL written + parse ok (24): GEN-1, GEN-6, PIT-2..9, SOC-1, SOC-2, SOC-8, FAM-1 (benefit_period index, axis fam1_filed posted/received), WCP-1, WCP-2, CIV-1, IGT-2, IGT-3 (index igt_group, keyed band igt3_scale[g], igt_free[g] gazette), IGT-4, CC-1 (axis cc1_refund by_time/by_balance), CC-2, MOT-1, MOT-2 (band mot2_tax(cc)).
- next: household files, then checks, then maps.

## After compaction 3
- written + parse ok: reality, people (earner[e,y]), places, prices (forecast rules v fc_moves = scenario version), payroll, tax, benefit, gift.
- design: no `with law` pins in asks (law pin would re-date undated inputs like pit3_rate/holidays); defs w/ notes need `rule base =`; `part` is keyword (def jpart); if-values with `=` need parens; end_of(m) = first day AFTER m.
- next: car.tally (A-07/A-08), records.tally, invest.tally (A-05, A-09, A-10), worth.tally (A-11), forecast + scenarios (A-12, no_cut), asks/, checks, maps.

## After compaction 4
- written: car, records, invest, rules, plan, worth (A-07/08 car; A-05/A-09/A-10 invest+rules; A-11 worth; walk in plan from 2026-01-01).
- fixed: open-index additions removed (LOAD-KIND, L:562; keys come from rows); lot_reported rows one key each; tomek lots split (samples known 2021-08-16, rest `?` known 2024-11-15); num DUK types (tax y_hded/hded_p/relief_half/res_joint/together_joint/gain; payroll red12/adv_scale/adv_w; gift value/added_up); units -> single fund_u (O:A-4).
- next: forecast.tally + scenarios/ (A-12, no_cut), asks/A-01..12, checks, maps/A, report.
- after: forecast.tally (part sell_all), scenarios/ (forecast, base, crash, flat, switch, t25_*, no_cut), asks/A-01..12 written; card pay (hole) in plan; `at` on rule_day/so_day/pay_on/lot_cost; parsecheck 61 ok; loadcheck (household+law as one dir in scratchpad) 0 refusals.
- statuses: A-01..04, A-06, A-07 expressed; A-05 can't express (unit registers); A-08 awkward; A-09 can't express (records); A-10 case suspect; A-11 case suspect (key omits rza_cash 0.01; A-D31 by notice) + partial; A-12 case suspect (Jan 2026 vs A-10 refusal).
- O:A-1 dialect, O:A-2 unit registers, O:A-3 holiday list, O:A-4 fund_u, O:A-5 holidays law date, O:A-6 owns by origin, O:A-7 no str field; S:601 L:1120 vs L:188; S:602 covered(x, span) vs call L:215.
- next: maps/A/*.md then SubagentHandback.
- DONE: maps/A/A-01..A-12 written; final checks parsecheck 61 ok, loadcheck 0 refusals; report handed back.
