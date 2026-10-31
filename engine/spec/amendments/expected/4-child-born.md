# Change 4: a child is born — expected outputs

**Applied as:** four amendments.
1. An `advance` of `on`, 2026-10-19 → 2028-05-14.
2. The `add` of the birth: `ola_born` and `ola_home` (known 2028-05-14), with Ola's residence stated from 2026-01, the base's own start (the 1.0.1 diff).
3. An `advance` of `on`, 2028-05-14 → 2028-06-02.
4. The `add` of the claim, `ola_800` (known 2028-06-02).

Final cut K = 2028-06-02. **The optional addition** (the `ola_800_paid` ask, put in `asks/core.tally`) adds two cells; without it they are absent and the other cells are the same.

**Derived by:** C4-a and C4-b, blind to each other; every cell agrees under both diffs.

## Cells

| # | Ask | Key | Before | After | Status |
|---|---|---|---|---|---|
| 1 | wife_net | 2026-10 | 5101.67 | 5101.67 | exact |
| 2 | wife_net | 2027-01 | 5101.67 | 5101.67 | exact |
| 3 | social_and_health | 2026-10, zus_social | 456.18 | 456.18 | exact |
| 4 | social_and_health | 2026-10, health_month | 830.58 | 830.58 | exact |
| 5 | social_and_health | 2027-01, zus_social | 474.44 | 474.44 | exact |
| 6 | social_and_health | 2027-01, health_month | 518.28 | 518.28 | exact |
| 7 | social_and_health | 2028-09, zus_social | 2083.99 | 2083.99 | exact |
| 8 | social_and_health | 2028-09, health_month | 898.35 | 898.35 | exact |
| 9 | replay_september | konto 2026-09 | the HH4-b §5 ledger: balance line +32097.23 (stated 41386.27, booked 9289.04), then spend +4599.90, client_pay −3690.00, unexplained −3574.02, invest +535.30, loan_instalment +300.00, seven groups at 0.00 | unchanged | exact |
| 10 | explain_room | room[2026-11], (2026-11-15, phase 6, invest[ike, 2026-11]) | 4386.89 | 4386.89 | exact |
| 11 | explain_room | room[2026-11], (2026-11-15, phase 6, invest[mak, 2026-11]) | 4386.89 | 4386.89 | exact |
| 12 | konto_balance | 2026-10-31, bal | 6064.17 | 6064.17 | exact |
| 13 | konto_balance | 2026-10-31, bal − vat_owed | 6064.17 | 6064.17 | exact |
| 14 | konto_balance | 2026-12-31, bal | 6944.90 | 6944.90 | exact |
| 15 | konto_balance | 2026-12-31, bal − vat_owed | 6944.90 | 6944.90 | exact |
| 16 | konto_balance | 2027-06-30, bal | 6760.19 | 6760.19 | exact |
| 17 | konto_balance | 2027-06-30, bal − vat_owed | 6760.19 | 6760.19 | exact |
| 18 | konto_balance | 2027-12-31, bal | 6656.75 | 6656.75 | exact |
| 19 | konto_balance | 2027-12-31, bal − vat_owed | 6656.75 | 6656.75 | exact |
| 20 | net_worth | 2026-12-31 | 170202.16 | 170202.16 | exact |
| 21 | net_worth | 2031-12-31 | not computed (run-only) | higher than the base | run-only |
| 22 | net_worth | 2041-12-31 | not computed (run-only) | higher than the base | run-only |
| 23 | net_worth | 2059-12-31 | not computed (run-only) | higher than the base | run-only |
| 24 | ike_left | 2026 | −6000.00 | −6000.00 | exact |
| 25 | ike_left | 2027 | 0.00 | 0.00 | exact |
| 26 | ike_left | 2028 | 0.00 | 0.00 | exact |
| 27 | pit28 | 2026 | 2.00 | 2.00 | exact |
| 28 | pit28 | 2027 | 2.00 | 2.00 | exact |
| 29 | pit28 | 2028 | 1.00 | 1.00 | exact |
| 30 | pit37 | me, 2026 | 0.00 | 0.00 | exact |
| 31 | pit37 | me, 2027 | 0.00 | 0.00 | exact |
| 32 | pit37 | me, 2028 | 0.00 | 0.00 | exact |
| 33 | pit37 | wife, 2026 | −2.00 | −2.00 | exact |
| 34 | pit37 | wife, 2027 | −2.00 | −2.00 | exact |
| 35 | pit37 | wife, 2028 | −2.00 | **−743.00** | exact |
| 36 | health_settlement | 2026 | 996.69 | 996.69 | exact |
| 37 | health_settlement | 2027 | 1036.56 | 1036.56 | exact |
| 38 | health_settlement | 2028 | 1078.02 | 1078.02 | exact |
| 39 | taxes_paid | 2026 | 52272.00 | 52272.00 | exact |
| 40 | taxes_paid | 2027 | 69059.00 | 69059.00 | exact |
| 41 | taxes_paid | 2028 | 68997.00 | 68997.00 | exact |
| 42 | pit37 | ola, 2026 (new line) | — | **0.00** | exact, new |
| 43 | pit37 | ola, 2027 (new line) | — | **0.00** | exact, new |
| 44 | pit37 | ola, 2028 (new line) | — | **0.00** | exact, new |
| 45 | ola_800_paid (the optional addition) | 2028 | refused in the base (`LOAD-KEY`) | **6400.00** | exact, new |
| 46 | ola_800_paid (the optional addition) | 2029 | refused in the base (`LOAD-KEY`) | **9600.00** | exact, new |

**46 cells:** 43 exact, 0 holes, 3 run-only; 9 differ from the base (new cells included), 37 are the base's value.

- **pit37 wife 2028** is −743.00: the child relief for 2028's months from May (741.36), on top of the base's −2.00. The relief runs 17 full years plus 741.36 in 2028 and 370.68 in 2046; the wife's refunds are +741, then +1112 for each full year, then +371, the first on 2029-06-14.
- **ola_800_paid:** the 800+ benefit from 2028-05, the May instance corrected from 05-22 to 06-20 (it was dated before the claim was known), so 1600 on 2028-06-20; 6400.00 for 2028 and 9600.00 for 2029, to the last month 2046-04.
- **net_worth, run-only:** higher than the base. C4-a gives Δ = I + R − T with I = 38165 / 145285 / 192816 at 2031 / 2041 / 2059, exact, and R, T run-only.

## First day, and sanity

- **First day anything differs:** 2028-06-20, the 800+ movement and zus's balance (−1600.00 that day, −172800.00 at E). C4-b computes konto's 2028-06-15 invest as unchanged; C4-a leaves konto's first day open between 06-15 and 06-20 as run-only. The birth's own first movement difference is 2029-06-14 (`urzad_skarbowy`, the 2028 return).
- **Sanity:** the earliest `governs` is 2026-01 (Ola's residence), so it passes.

## The advances alone

No value, hole, ledger line or anchor changes. The registers do change: the cut line, the law date of the dateless ask lines, and the `held` entries that drop.

## Warnings and holes

- **Added:** at the claim, `WARN-BACKPAY` lists `b800_pay[ola, 2028-05]`, δ +800, covered by `arrears on 2028-06-20` (LR-10, LR-8). The birth changes no booked pay: the pit37 ola instances re-measure to δ 0.
- **Cleared:** none. **Holes:** none.

## The register

New `held` entries: `residence[ola] = pl (ola_home)` and the three `b800_2024` rows, read from 2028-07. The advances' dropped `held` entries and the new cut line, as above.

## The variant as first written

*At the tally 2.0 freeze (G2, keys), cells 42–44 below carry tally 2's question; see `KEYS.md` beside the package.*

The 1.0 diff states `residence[ola] = pl governs 2028-05..`. `pit37_tax`'s `abroad` rule (`employment.tally`:56) reads `residence[p, m]` for every month of the year, for every person, and the base's keyless residence row doesn't cover a key added later. So `pit37_due[ola, 2026..2028]` are holes, and they reach konto, room and the wrappers through her pay and refund instances.

| # | Ask | Key | Before | After | Status |
|---|---|---|---|---|---|
| 1 | wife_net | 2026-10 | 5101.67 | 5101.67 | exact |
| 2 | wife_net | 2027-01 | 5101.67 | 5101.67 | exact |
| 3 | social_and_health | 2026-10, zus_social | 456.18 | 456.18 | exact |
| 4 | social_and_health | 2026-10, health_month | 830.58 | 830.58 | exact |
| 5 | social_and_health | 2027-01, zus_social | 474.44 | 474.44 | exact |
| 6 | social_and_health | 2027-01, health_month | 518.28 | 518.28 | exact |
| 7 | social_and_health | 2028-09, zus_social | 2083.99 | 2083.99 | exact |
| 8 | social_and_health | 2028-09, health_month | 898.35 | 898.35 | exact |
| 9 | replay_september | konto 2026-09 | the HH4-b §5 ledger: balance line +32097.23 (stated 41386.27, booked 9289.04), then spend +4599.90, client_pay −3690.00, unexplained −3574.02, invest +535.30, loan_instalment +300.00, seven groups at 0.00 | unchanged | exact |
| 10 | explain_room | room[2026-11], (2026-11-15, phase 6, invest[ike, 2026-11]) | 4386.89 | 4386.89 | exact |
| 11 | explain_room | room[2026-11], (2026-11-15, phase 6, invest[mak, 2026-11]) | 4386.89 | 4386.89 | exact |
| 12 | konto_balance | 2026-10-31, bal | 6064.17 | 6064.17 | exact |
| 13 | konto_balance | 2026-10-31, bal − vat_owed | 6064.17 | 6064.17 | exact |
| 14 | konto_balance | 2026-12-31, bal | 6944.90 | 6944.90 | exact |
| 15 | konto_balance | 2026-12-31, bal − vat_owed | 6944.90 | 6944.90 | exact |
| 16 | konto_balance | 2027-06-30, bal | 6760.19 | 6760.19 | exact |
| 17 | konto_balance | 2027-06-30, bal − vat_owed | 6760.19 | 6760.19 | exact |
| 18 | konto_balance | 2027-12-31, bal | 6656.75 | 6656.75 | exact |
| 19 | konto_balance | 2027-12-31, bal − vat_owed | 6656.75 | 6656.75 | exact |
| 20 | net_worth | 2026-12-31 | 170202.16 | 170202.16 | exact |
| 21 | net_worth | 2031-12-31 | not computed (run-only) | **hole: `W jakim kraju ta osoba mieszka w tym miesiącu?` (residence[ola, …])** | hole |
| 22 | net_worth | 2041-12-31 | not computed (run-only) | **hole: `W jakim kraju ta osoba mieszka w tym miesiącu?` (residence[ola, …])** | hole |
| 23 | net_worth | 2059-12-31 | not computed (run-only) | **hole: `W jakim kraju ta osoba mieszka w tym miesiącu?` (residence[ola, …])** | hole |
| 24 | ike_left | 2026 | −6000.00 | −6000.00 | exact |
| 25 | ike_left | 2027 | 0.00 | 0.00 | exact |
| 26 | ike_left | 2028 | 0.00 | **hole: `W jakim kraju ta osoba mieszka w tym miesiącu?` (residence[ola, …])** | hole |
| 27 | pit28 | 2026 | 2.00 | 2.00 | exact |
| 28 | pit28 | 2027 | 2.00 | 2.00 | exact |
| 29 | pit28 | 2028 | 1.00 | 1.00 | exact |
| 30 | pit37 | me, 2026 | 0.00 | 0.00 | exact |
| 31 | pit37 | me, 2027 | 0.00 | 0.00 | exact |
| 32 | pit37 | me, 2028 | 0.00 | 0.00 | exact |
| 33 | pit37 | wife, 2026 | −2.00 | −2.00 | exact |
| 34 | pit37 | wife, 2027 | −2.00 | −2.00 | exact |
| 35 | pit37 | wife, 2028 | −2.00 | **−743.00** | exact |
| 36 | health_settlement | 2026 | 996.69 | 996.69 | exact |
| 37 | health_settlement | 2027 | 1036.56 | 1036.56 | exact |
| 38 | health_settlement | 2028 | 1078.02 | 1078.02 | exact |
| 39 | taxes_paid | 2026 | 52272.00 | 52272.00 | exact |
| 40 | taxes_paid | 2027 | 69059.00 | 69059.00 | exact |
| 41 | taxes_paid | 2028 | 68997.00 | **hole: `W jakim kraju ta osoba mieszka w tym miesiącu?` (residence[ola, …])** | hole |
| 42 | pit37 | ola, 2026 (new line) | — | **hole: `Zmiana rezydencji podatkowej w roku nie jest opisana (art. 3, umowa o unikaniu podwójnego opodatkowania).` (pit37_tax[ola, 2026])** (tally 2; the 1.0.1 key's question `W jakim kraju ta osoba mieszka w tym miesiącu?` (residence[ola, …]) can't arise in tally 2) | hole, new |
| 43 | pit37 | ola, 2027 (new line) | — | **hole: `Zmiana rezydencji podatkowej w roku nie jest opisana (art. 3, umowa o unikaniu podwójnego opodatkowania).` (pit37_tax[ola, 2027])** (tally 2; the 1.0.1 key's question `W jakim kraju ta osoba mieszka w tym miesiącu?` (residence[ola, …]) can't arise in tally 2) | hole, new |
| 44 | pit37 | ola, 2028 (new line) | — | **hole: `Zmiana rezydencji podatkowej w roku nie jest opisana (art. 3, umowa o unikaniu podwójnego opodatkowania).` (pit37_tax[ola, 2028])** (tally 2; the 1.0.1 key's question `W jakim kraju ta osoba mieszka w tym miesiącu?` (residence[ola, …]) can't arise in tally 2) | hole, new |
| 45 | ola_800_paid (the optional addition) | 2028 | refused in the base (`LOAD-KEY`) | **6400.00** | exact, new |
| 46 | ola_800_paid (the optional addition) | 2029 | refused in the base (`LOAD-KEY`) | **9600.00** | exact, new |

**46 cells:** 38 exact, 8 holes, 0 run-only; 11 differ from the base (new cells included), 35 are the base's value.

- **The hole questions:** `? W jakim kraju ta osoba mieszka w tym miesiącu?` at residence[ola, 2026-01..2026-12], [2027-01..2027-12] and [2028-01..2028-04] for the three pit37 ola lines; the cells reached through her pay instances carry the same question.
- **First day:** 2028-05-14, when the `HOLE-CHANGED` movements of the re-measured past pit37 ola instances land: the key was added after their booking cuts, so each counted A₀ = 0 there (LC-1), and its hole δ is dated on or after the version's `known` (V4-P4).
- **Warnings:** at the birth, `WARN-BACKPAY` lists the three re-measured pit37 ola instances as covered by no line: they become `HOLE-CHANGED`. At the claim, as above.
- **Sanity:** `ola_born` has no `governs`, so its spanless rows (`born[ola]`, the `parent` rows) govern every date and the bound is the run's first day (OPERATIONS §7). The first day is after it, so it passes. The 1.0 words' "2028-05" counted Ola's residence row only.

## Readings it rests on

- A key added after a booking cut counts A₀ = 0 at that cut (LC-1); a hole δ is `HOLE-CHANGED`, dated on or after V.known (V4-P4). These decide the variant's first day only.
- A fact version lists its back pay as a law version does (LR-10), covered instances included (LR-8).
- `held` and the printed cut are the ask's K: LR-1.
