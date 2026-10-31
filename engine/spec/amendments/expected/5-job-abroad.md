# Change 5: a move abroad, with a job there — expected outputs

**Applied as:** an `advance` of `on`, 2026-10-19 → 2029-05-01, then the `add` of the new sections and files (`abroad.tally`, `law/de/employment.tally`, `facts/2029.tally`) and their versions, all known 2029-05-01. The 1.0.1 diff adds `opening[konto_eur] = 0` to `berlin_job`, as `loans.tally` does for the laptop loan's debt. Final cut K = 2029-05-01.

**Derived by:** C5-a and C5-b, blind to each other; all cells agree under both diffs. They differed on the first day under the fixed diff; the lead took C5-b's (below).

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
| 21 | net_worth | 2031-12-31 | not computed (run-only) | **hole: `Wynagrodzenie netto z niemieckiej umowy o pracę nie jest opisane (Lohnsteuer, składki).`, `Zmiana rezydencji podatkowej w roku nie jest opisana (art. 3, umowa o unikaniu podwójnego opodatkowania).`** | hole |
| 22 | net_worth | 2041-12-31 | not computed (run-only) | **hole: `Wynagrodzenie netto z niemieckiej umowy o pracę nie jest opisane (Lohnsteuer, składki).`, `Zmiana rezydencji podatkowej w roku nie jest opisana (art. 3, umowa o unikaniu podwójnego opodatkowania).`** | hole |
| 23 | net_worth | 2059-12-31 | not computed (run-only) | **hole: `Wynagrodzenie netto z niemieckiej umowy o pracę nie jest opisane (Lohnsteuer, składki).`, `Zmiana rezydencji podatkowej w roku nie jest opisana (art. 3, umowa o unikaniu podwójnego opodatkowania).`** | hole |
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
| 35 | pit37 | wife, 2028 | −2.00 | −2.00 | exact |
| 36 | health_settlement | 2026 | 996.69 | 996.69 | exact |
| 37 | health_settlement | 2027 | 1036.56 | 1036.56 | exact |
| 38 | health_settlement | 2028 | 1078.02 | 1078.02 | exact |
| 39 | taxes_paid | 2026 | 52272.00 | 52272.00 | exact |
| 40 | taxes_paid | 2027 | 69059.00 | 69059.00 | exact |
| 41 | taxes_paid | 2028 | 68997.00 | 68997.00 | exact |

**41 cells:** 38 exact, 3 holes, 0 run-only; 3 differ from the base (new cells included), 38 are the base's value.

**The questions:**
- **Qde** `Wynagrodzenie netto z niemieckiej umowy o pracę nie jest opisane (Lohnsteuer, składki).`, at `net_wage_eur[wife_berlin, 2029-07..]`, to the cell's year;
- **Qres** `Zmiana rezydencji podatkowej w roku nie jest opisana (art. 3, umowa o unikaniu podwójnego opodatkowania).`, at `pit37_tax[wife, 2029..]`, to the year before the cell's.

## First day, and sanity

- **First day anything differs:** 2029-07-25, `wage_eur[wife_berlin, 2029-07]` into konto_eur. The rule of the 15th is a hole from walk day 2029-08-15. `room[2029-07]` is unchanged: in 2029, client_a's June invoice lands on 07-16 (07-14 is a Saturday), so every later day of the window sits above day t, whose relative value is 0, and the window's minimum stays 0 on t. (C5-a had 2029-07-15, carried from the 2027 window; it doesn't hold for 2029's calendar.) `plan_low` leaves out state pays (LANGUAGE, `plan_low`), so the change file's 2029-07-15 for the hole is wrong too.
- **Sanity:** the earliest `governs` is 2026-01 (`salary[wife_berlin] = 0 governs 2026-01..`), so it passes.

## The advance alone

No value changes. The registers' cut line moves to 2029-05-01 and the `held` entries for the months it passes drop.

## Warnings and holes

- **Holes:** konto from 2029-07-28, the first conversion of the German wage (Qde); `wcash[ike]` and `wcash[oki]` from 2029-08-15, through the rule of the 15th; the wife's pit37 from 2029 (Qres). So net worth is a hole at every asked date after 2029.
- **Warnings:** none added. Any base `WARN-NEGATIVE` or `WARN-HELD` on konto from 2029-07-28, or on `wcash[ike]` or `wcash[oki]` from 2029-08-15, is no longer tested (run-only).

## The register

- **New:** `assumed: kurs przykładowy; do zastąpienia kursem z wymiany` (`fx_eur = 4.3`, fx_2029), and, on the results that read past K, `held: fx_eur = 4.3 (fx_2029)`, `held: salary_eur[wife_berlin] = 5200 (berlin_job)`, `held: salary_eur = 0 (berlin_job)`, `held: salary[wife_berlin] = 0 (berlin_job)`, `held: salary[wife_acme] = 0 (wife_moves)` and `held: residence[wife] = de (wife_moves)`. `residence = pl (people_2026)` is now read for the wife up to 2029-06 only. Among the asked cells, these reach net_worth from 2031.
- C5-b lists all of these. C5-a lists five `held` rows and ties `fx_eur`'s two entries to whether a hole-`when` instance's amount is read, which the spec leaves open (`CHANGES.md`, C5-F4). That decides register text only.
- The stub and the new source add no entry: the stub prints as its question, the source shows in `explain`.
- **Removed:** the advance's dropped `held` entries, as above.

## The variant as first written

*At the tally 2.0 freeze (G2, keys), cells 9–15, 20 and 24 below carry what tally 2 prints. Cells 16–19, 21–23, 25–26 and 41 keep their tally 1.0.1 key: they are long-horizon walk cells, unjudged until the engine, and the m2 map's tally 2 reading clears 16–19, 25–26 and 41 too. See `KEYS.md` beside the package.*

The 1.0 diff gives `konto_eur` no `opening` row. By LANGUAGE §5.9, `bal(konto_eur)` is a hole from S until its first anchor, and it has none. `fx_in` and `fx_out` read it from 2026-01-28, so konto is a hole from 2026-01-28. The 2026-09-30 anchor stops that, but the IKE cap's `flows` over the February–August hole invests make konto a hole again from 2026-10-15.

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
| 9 | replay_september | konto 2026-09 | the HH4-b §5 ledger: balance line +32097.23 (stated 41386.27, booked 9289.04), then spend +4599.90, client_pay −3690.00, unexplained −3574.02, invest +535.30, loan_instalment +300.00, seven groups at 0.00 | **the base's ledger with no hole group: balance gap 32 097.23 (booked 9 289.04, stated 41 386.27), spend 4 599.90, client_pay −3 690.00, unexplained −3 574.02, invest 535.30, loan_instalment 300.00, seven moves at 0.00 (tally 2)** | exact |
| 10 | explain_room | room[2026-11], (2026-11-15, phase 6, invest[ike, 2026-11]) | 4386.89 | **4386.89 (tally 2)** | exact |
| 11 | explain_room | room[2026-11], (2026-11-15, phase 6, invest[mak, 2026-11]) | 4386.89 | **4386.89 (tally 2)** | exact |
| 12 | konto_balance | 2026-10-31, bal | 6064.17 | **6064.17 (tally 2)** | exact |
| 13 | konto_balance | 2026-10-31, bal − vat_owed | 6064.17 | **6064.17 (tally 2)** | exact |
| 14 | konto_balance | 2026-12-31, bal | 6944.90 | **6944.90 (tally 2)** | exact |
| 15 | konto_balance | 2026-12-31, bal − vat_owed | 6944.90 | **6944.90 (tally 2)** | exact |
| 16 | konto_balance | 2027-06-30, bal | 6760.19 | **hole: `Po jakim kursie wymieniamy euro na złote?`, `opening at konto_eur: no value at 2026-01-01`** | hole |
| 17 | konto_balance | 2027-06-30, bal − vat_owed | 6760.19 | **hole: `Po jakim kursie wymieniamy euro na złote?`, `opening at konto_eur: no value at 2026-01-01`** | hole |
| 18 | konto_balance | 2027-12-31, bal | 6656.75 | **hole: `Po jakim kursie wymieniamy euro na złote?`, `opening at konto_eur: no value at 2026-01-01`** | hole |
| 19 | konto_balance | 2027-12-31, bal − vat_owed | 6656.75 | **hole: `Po jakim kursie wymieniamy euro na złote?`, `opening at konto_eur: no value at 2026-01-01`** | hole |
| 20 | net_worth | 2026-12-31 | 170202.16 | **170202.16 (tally 2)** | exact |
| 21 | net_worth | 2031-12-31 | not computed (run-only) | **hole: `Po jakim kursie wymieniamy euro na złote?`, `Wynagrodzenie netto z niemieckiej umowy o pracę nie jest opisane (Lohnsteuer, składki).`, `Zmiana rezydencji podatkowej w roku nie jest opisana (art. 3, umowa o unikaniu podwójnego opodatkowania).`, `opening at konto_eur: no value at 2026-01-01`** | hole |
| 22 | net_worth | 2041-12-31 | not computed (run-only) | **hole: `Po jakim kursie wymieniamy euro na złote?`, `Wynagrodzenie netto z niemieckiej umowy o pracę nie jest opisane (Lohnsteuer, składki).`, `Zmiana rezydencji podatkowej w roku nie jest opisana (art. 3, umowa o unikaniu podwójnego opodatkowania).`, `opening at konto_eur: no value at 2026-01-01`** | hole |
| 23 | net_worth | 2059-12-31 | not computed (run-only) | **hole: `Po jakim kursie wymieniamy euro na złote?`, `Wynagrodzenie netto z niemieckiej umowy o pracę nie jest opisane (Lohnsteuer, składki).`, `Zmiana rezydencji podatkowej w roku nie jest opisana (art. 3, umowa o unikaniu podwójnego opodatkowania).`, `opening at konto_eur: no value at 2026-01-01`** | hole |
| 24 | ike_left | 2026 | −6000.00 | **−6000.00 (tally 2)** | exact |
| 25 | ike_left | 2027 | 0.00 | **hole: `Po jakim kursie wymieniamy euro na złote?`, `opening at konto_eur: no value at 2026-01-01`** | hole |
| 26 | ike_left | 2028 | 0.00 | **hole: `Po jakim kursie wymieniamy euro na złote?`, `opening at konto_eur: no value at 2026-01-01`** | hole |
| 27 | pit28 | 2026 | 2.00 | 2.00 | exact |
| 28 | pit28 | 2027 | 2.00 | 2.00 | exact |
| 29 | pit28 | 2028 | 1.00 | 1.00 | exact |
| 30 | pit37 | me, 2026 | 0.00 | 0.00 | exact |
| 31 | pit37 | me, 2027 | 0.00 | 0.00 | exact |
| 32 | pit37 | me, 2028 | 0.00 | 0.00 | exact |
| 33 | pit37 | wife, 2026 | −2.00 | −2.00 | exact |
| 34 | pit37 | wife, 2027 | −2.00 | −2.00 | exact |
| 35 | pit37 | wife, 2028 | −2.00 | −2.00 | exact |
| 36 | health_settlement | 2026 | 996.69 | 996.69 | exact |
| 37 | health_settlement | 2027 | 1036.56 | 1036.56 | exact |
| 38 | health_settlement | 2028 | 1078.02 | 1078.02 | exact |
| 39 | taxes_paid | 2026 | 52272.00 | 52272.00 | exact |
| 40 | taxes_paid | 2027 | 69059.00 | 69059.00 | exact |
| 41 | taxes_paid | 2028 | 68997.00 | **hole: `Po jakim kursie wymieniamy euro na złote?`, `opening at konto_eur: no value at 2026-01-01`** | hole |

**41 cells:** 22 exact, 19 holes, 0 run-only; 19 differ from the base (new cells included), 22 are the base's value.

- **The questions:** Qfx `Po jakim kursie wymieniamy euro na złote?` (fx_in[2026-01..08]) and **Qop** `opening at konto_eur: no value at 2026-01-01`; the later net worth cells add Qde and Qres.
- **First day:** 2026-01-01 for konto_eur, 2026-01-28 for konto.
- **Warnings:** the base's `WARN-NEGATIVE` on konto of 2026-03-01 and 2026-03-17 is cleared, since konto is a hole on both days, and so is any later warning on those places.
- **replay_september:** hole groups print first; all three carry Qfx first, so they tie and go by group name (`ENGINE.md` §6.3).

## Readings it rests on

- The pre-move `wage_eur[wife_acme, 2026-01..2029-04]` instances had a hole `when` at their booking cuts (no `salary_eur` row before 2029-05-01). `berlin_job` makes that `when` false, which answers the hole with 0: nothing is booked (LR-5). Under the other reading, 40 hole movements into konto_eur would remain.
- `held` and the printed cut are the ask's K: LR-1.
- The law-part gate premise, `law/de/` included (`README.md` here).
