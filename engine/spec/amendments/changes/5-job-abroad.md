# Change 5: a move abroad, with a job there

**Statement (household):** "Od lipca 2029 żona pracuje w Berlinie, 5 200 euro brutto miesięcznie, i tam się przeprowadza. Ja zostaję w Krakowie z firmą. Pracę w Acme kończy z końcem czerwca. Euro wymieniamy na złote co miesiąc."

**Operation:** amend (OPERATIONS §4) with new sections, plus add a fact (§2.1) with `add_keys`. **Kind:** two amendments: an `advance` of `on` to 2029-05-01 (OPERATIONS §4), then one `add` of the new sections and versions. As one `add` the change would be refused (`OP-EDIT`), since it edits the `on` line. **Known:** 2029-05-01.

## Diff

```diff
--- reality.tally
+++ reality.tally
-  on 2026-10-19
+  on 2029-05-01

--- abroad.tally   (new file)
+part abroad
+  tally 2
+  note "Praca żony w Niemczech od lipca 2029: konto w euro, pensja w euro, wymiana na złote 28. dnia miesiąca."
+
+unit EUR: currency minor 0.01
+
+kind exchanges: event certain
+  leg eur: EUR
+  leg pln: PLN
+
+place kantor_eur: outside lives 2026-01-01..
+
+place kantor_pln: outside lives 2026-01-01..
+
+place konto_eur: accounts EUR lives 2029-05-01..
+
+place pracodawcy_de: outside lives 2026-01-01..
+
+input fx_dom: int
+  note "lifted from an inline figure of tally 1.0.1"
+
+input fx_eur: num PLN/EUR question "Po jakim kursie wymieniamy euro na złote?"
+
+input salary_eur[j in job, m in month]: money EUR question "Ile wynosi pensja brutto w euro z tej pracy w tym miesiącu?"
+
+input wage_eur_25: int
+  note "lifted from an inline figure of tally 1.0.1"
+
+move fx[m in month]: exchanges
+  on day_of(m, fx_dom)
+  at start_of(m)
+  when bal(konto_eur) > 0
+  reads konto_eur
+  leg eur from konto_eur to kantor_eur amount bal(konto_eur)
+  leg pln from kantor_pln to konto amount round(bal(konto_eur) * fx_eur, minor(PLN), down)
+
+move wage_eur[j in job over .., m in month]: transfers
+  on day_of(m, wage_eur_25)
+  at start_of(m)
+  for m
+  when salary_eur[j, m] > 0
+  leg cash from pracodawcy_de to konto_eur amount net_wage_eur[j, m]
+
+version lifted_figures_5_job_abroad
+  cite lifted_1_0_1
+  governs always
+  known 2029-05-01
+  fx_dom = 28 "inline in tally 1.0.1: fx_in line 21 and fx_out line 29, argument 2 of day_of; one fact, the exchange day"
+  wage_eur_25 = 25 "inline in tally 1.0.1: wage_eur line 37, argument 2 of day_of"

--- law/de/employment.tally   (new file)
+part de_employment
+  tally 2
+  law
+  note "Niemiecka umowa o pracę: Lohnsteuer, Solidaritätszuschlag, składki. Zalążek: wynik jest dziurą z pytaniem, dopóki część nie zostanie napisana i nie przejdzie bramki."
+
+def net_wage_eur[j in job, m in month]: money EUR at start_of(m)
+  rule stub = ?"Wynagrodzenie netto z niemieckiej umowy o pracę nie jest opisane (Lohnsteuer, składki)."

--- facts/2029.tally   (new file)
+part facts_2029
+  tally 2
+
+source berlin_contract "Arbeitsvertrag żony, Berlin, 2029-04-28"
+
+version berlin_job
+  cite berlin_contract
+  governs always
+  known 2029-05-01
+  job + [wife_berlin] lives 2029-07-01..
+  opening[konto_eur] = 0 at 2029-05-01 start "nowe konto, saldo zerowe"
+  salary[wife_berlin] = 0 governs 2026-01.. "pensja w euro, nie w złotych"
+  salary_eur = 0 governs 2026-01.. "brak pensji w euro"
+  salary_eur[wife_berlin] = 5200 governs 2029-07.. "§ 4"
+  works_for[wife_berlin] = wife
+
+version fx_2029
+  cite modeller
+  governs always
+  known 2029-05-01
+  fx_eur = 4.3 assumed "kurs przykładowy; do zastąpienia kursem z wymiany" governs 2029-07..
+
+version wife_moves
+  cite household
+  governs always
+  known 2029-05-01
+  residence[wife] = de governs 2029-07.. "wymeldowanie; umowa najmu w Berlinie"
+  replace salary[wife_acme] = 0 governs 2029-07.. "wypowiedzenie; ostatnia pensja za czerwiec"
```

**Lines:** 84 added in three new files (66 non-blank), plus the `on` line. Lines edited: the `on` line only.

**Why each part is there:**
- **Currencies.** Money in two currencies is never added (N-202). The euro side needs its own account, outside places, input and pays.
- **Exchange.** Like a purchase, it is one movement with a leg in each currency (LANGUAGE §5.10, §5.14): `fx` reads the euro balance once, at its point of the day walk, so both legs see the same balance, and the rate is applied once, rounded down to the grosz (`minor(PLN)`). Tally 1.0.1 wrote it as two pays, `fx_in` (phase 4) reading the balance before `fx_out` (phase 5) emptied it.
- **A guard on the euro wage.** `wage_eur` ranges over every job, so the keyless `salary_eur = 0` row and the `when` keep `wage_acme` and the months before July free of instances. Without them, the stub's hole would reach `konto_eur` from 2026. Before 2029-05-01 no `salary_eur` row is visible, so the `when` of the earlier instances was a hole at their booking cuts; `berlin_job` makes it false, which answers them with 0 (LANGUAGE §5.13.3).
- **No Polish salary for the German job.** `salary[wife_berlin] = 0` is needed because `wage`, `emp_*` and `pit37_base` range over every job. Without it, they would read a missing salary for the new key.
- **A lifespan and an opening for the euro account.** The account is new: `place konto_eur: accounts EUR lives 2029-05-01..`, with its reading `opening[konto_eur] = 0 at 2029-05-01 start` (LANGUAGE §5.10, §6.2 "a new account"). Before its lifespan it holds nothing (`ENGINE.md` §9, invariant 6), so `fx` reads 0 on every 28th from 2026-01-28 and has no instance there; tally 1.0.1 needed a zero opening at the run's start for the same reads.

**New keys checked against closed-world rows.** The new key `wife_berlin` (index `job`) is read by:
- `salary`: stated;
- `salary_eur`: covered by the keyless row;
- `works_for`: stated.

There are no new holes from the key itself. The new place `konto_eur` has its lifespan and its `opening` reading.

## What it changes, in words

The expected outputs, cell by cell, are in `amendments/expected/5-job-abroad.md`, with the diff as first written as a variant.

**First day anything differs:** 2029-07-25, when the first German wage reaches `konto_eur` (a hole, below). That is not before the earliest `governs`, 2026-01 (`salary[wife_berlin] = 0` and `salary_eur = 0`), so the sanity check holds.
- On 2029-07-15 the rule of the 15th measures room up to 2029-08-14, and `room[2029-07]` is unchanged. Its window doesn't reach the hole on `konto` from 2029-07-28: that comes from the exchange, a state pay, and tally 1.0.1's `plan_low` left state pays out. (Under tally 2 the projection counts `fx`, a decision that doesn't read `room` (LANGUAGE §5.14.1), so `room[2029-07]` meets that hole; `amendments/expected/5-job-abroad.md` keeps the 1.0.1 answer, and `regress/maps/m2/cells.md` lists the step.)
- The missing wage of 2029-08-10 is in that window but doesn't move its minimum, which stays 0 on day t: client A's June pay lands on 2029-07-16 (07-14 is a Saturday), so every later day sits above it.
- The June wage, paid 2029-07-10, is unchanged.

**Movements**
- **The wife's Polish job ends:**
  - `wage[wife_acme, m]` from 2029-07 is 0: salary 0 gives net 0. The first such instance is dated 2029-08-10.
  - Her employee ZUS, health and PIT advances are 0 from July.
- **The German wage:**
  - `wage_eur[wife_berlin, m]` from 2029-07 is a **hole** on each 25th, from 2029-07-25: "Wynagrodzenie netto z niemieckiej umowy o pracę nie jest opisane".
  - `konto_eur` is a hole from that day.
  - So is `fx` on the 28th, both its legs, and `konto` from **2029-07-28**.
- **Everything that reads `konto` afterwards** is a hole with the same question, joined from 2030-04-30 by the `abroad` question of the wife's PIT-37 (below):
  - the rule of the 15th from walk day 2029-08-15, whose `bal(konto)` at t is a hole;
  - the wrappers `wcash[ike]` and `wcash[oki]` from 2029-08-15 (`invest[mak]` stays 0, and known);
  - `room`'s projection, from walk day 2029-08-15.
- **After a statement** (D8, J70): a later import whose `stated[konto]` row governs a month-end after 2029-07-28 re-anchors `konto` there. From that day `konto` is known again, up to the next hole: the next 15th, since the IKE cap reads the year's `flows(invest[ike], …)`, which hold its hole instances. Only after a December anchor is it the next 28th. The decisions taken before the anchor stay holes, and so do the wrappers they fill, which have no statement; the questions stay listed on that statement's balance line. Under this change's own `on`, no such statement exists yet, so the outputs below are as stated.
- **Unchanged:**
  - my pays: business, ZUS, health, ryczałt, VAT;
  - the rent;
  - spending, which the household didn't restate. Its budget is the couple's, and a separate statement would split it.

**Asks**
- `pit37`: the ask covers 2026–2028, unchanged. Beyond the ask, the wife's 2029 PIT-37 is a hole from the `abroad` rule, "Zmiana rezydencji podatkowej w roku nie jest opisana…". So are 2030 and later, since every month is `de`. So `pit37_pay` and `pit37_refund` of each return day from 2030-04-30 are holes in `konto` too.
- `net_worth`:
  - the 2026-12-31 row is unchanged;
  - the 2031, 2041 and 2059 rows are holes, because `konto` is.
  - It sums PLN places only. Once the German part is written, `konto_eur` is empty after each 28th, so the PLN sum is still the household's whole net worth. EUR held over a month-end would need the ask to convert, with `in PLN at fx_eur`.
- **Unchanged:** `konto_balance` (all rows are before 2029), `taxes_paid`, `health_settlement`, `pit28`, `social_and_health`, `wife_net`, `ike_left`, `explain_room` and `replay_september`.
- `explain room[2029-08]` shows the hole and its question at the root.

**When the German part is written** and passes its gate, the holes close. Then:
- `konto` receives about 4.3 × the German net wage on each 28th;
- the 15th invests it.

The Polish PIT for the year of the move still needs its own rule, the `abroad` hole. That is a separate law version: residence, the double-tax treaty, and the method of avoiding double taxation.

**Register:**
- one assumed rate: `assumed: kurs przykładowy; do zastąpienia kursem z wymiany` (`fx_eur = 4.3`, modeller);
- new `held` entries on the results that read past 2029-05 (among the asked cells, `net_worth` from 2031): `fx_eur`, the `berlin_job` rows and the `wife_moves` rows;
- the stub def and the new source are not register entries: the stub shows as its question on every result it reaches, and the source in `explain`.

**Warnings:** none new. The holes are results, not warnings.

**Composes with change 4.** If Ola was born (change 4) and moves with her mother, the household adds `residence[ola] = de governs 2029-07..`.
- Her 800+ stops from 2029-07: `b800_due` reads `residence[c, m] = pl`.
- The wife's child relief for 2029 is inside the `abroad` hole.

If Ola stays in Kraków with me, nothing changes for her.

**Depends on:**
- L1: a two-leg exchange, each pay in one currency;
- L8: an instance exists only where its `when` is true;
- L9: a state-reading `when` is evaluated at the walk point;
- L10: the `on` advance;
- L11: a stub def with a hole rule;
- L19: whether a part under `law/` faces the gate, and whether a hole-only part can pass it.
