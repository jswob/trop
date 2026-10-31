# Change 6: a new loan, and an overpayment

**Statement (household):** "Bierzemy kredyt na samochód: 40 000 zł, 48 rat równych, 8,99% rocznie, pierwsza rata 10 maja 2027. Bank wypłaca pieniądze salonowi. Umowa z 2 kwietnia." Later: "Przy racie w styczniu 2028 nadpłacimy 5 000 zł; umowa mówi, że nadpłata skraca okres."

**Operation:** add a fact (OPERATIONS §2.1), twice. The loan adds `car_dealer` to `seller` with `add_keys`, and enters the loan `car` in the kind `loan`; the overpayment adds no key. **Kind:** four amendments: an `advance` of `on` to 2027-04-02 (OPERATIONS §4), the `add` of the loan, an `advance` to 2028-01-08, then the `add` of the overpayment. Each `add` alone would be refused (`OP-EDIT`), since it would edit the `on` line. **Known:** 2027-04-02, then 2028-01-08.

## Diff

```diff
--- reality.tally
+++ reality.tally
-  on 2026-10-19
+  on 2027-04-02

--- facts/2027.tally   (new file, or added to change 1's)
+part facts_2027
+  tally 2
+
+source car_contract "Umowa kredytu samochodowego, 2027-04-02"
+
+version car_loan
+  cite car_contract
+  governs always
+  known 2027-04-02
+  loan + [car] lives 2027-04-02..
+  seller + [car_dealer] lives 2027-04-02..
+  loan_seller[car] = car_dealer "§ 1: wypłata na rachunek sprzedawcy"
+  loan_terms[car] = (principal: 40000, n: 48, apr: 0.0899 1/year, first_month: 2027-05, due: 10) "§ 2"
+  overpay[car] = 0 "brak nadpłat"
+  overpay_choice[car] = shorten "§ 9: nadpłata skraca okres kredytowania"
```

Then, at the overpayment (the `on` line moves to 2028-01-08):

```diff
--- facts/2027.tally
+++ facts/2027.tally
+
+version car_overpay_2028_01
+  cite household
+  governs always
+  known 2028-01-08
+  overpay[car, 9] = 5000 "nadpłata przy 9. racie, styczeń 2028"
```

**Lines:** 15 added for the loan and 6 for the overpayment (13 and 5 non-blank), plus the `on` line each time. Lines edited: the `on` line only.

**New keys checked against closed-world rows.** Two keys are added:
- **`car`**, in the kind `loan`: `loan + [car] lives 2027-04-02..`, from the contract's date, as `laptop_loan` enters `laptop`. The place `loan_debt[car]` lives from then too (a keyed place lives on its key thing's lifespan, LANGUAGE §5.10), so it needs no opening: its balance is zero at its lifespan's start. It is read by `loan_seller`, `loan_terms`, `overpay` and `overpay_choice`. The base rows are keyed to `laptop`, or keyless in `laptop_loan`'s version (`overpay = 0`), so none covers `car`. All four are stated.
- **`car_dealer`**, in `seller`, for the place `sellers[car_dealer]`. It is an outside place, which needs no opening.

There are no new holes.

## What it changes, in words

The expected outputs, cell by cell, are in `amendments/expected/6-loan-and-overpayment.md`.

The loan's defs are recurrences over `inst` (1..480), so `car` gets its own schedule with no new def.

**At the loan** (`on` = 2027-04-02):
- **The rate:** `inst_rate[car]` = periodic(8.99% nominal, month) = 0.0899 / 12.
- **The instalment:** `annuity[car]` = round(40 000 × i × (1 + i)^48 / ((1 + i)^48 − 1), 0.01) = **995.21**.
- **The first instalment** (2027-05-10, a Monday) is 299.67 interest and 695.54 principal.
- **Movements:**
  - `loan_out[car]` on 2027-04-10 moves 40 000 from `loan_debt[car]` to `sellers[car_dealer]`.
  - `loan_instalment[car, k]`, one movement with its `principal` leg to `loan_debt[car]`, its `interest` leg to `kredytodawcy` and its `overpay` leg (0 here), runs on the 10th of each month, from 2027-05-10 to 2031-04-10.
  - The `last` rule makes the 48th instalment repay the remainder exactly, rounding grosze included.
- **First day anything differs:** 2027-04-10, the disbursement. The version `governs always`, so it governs every date and the sanity bound holds trivially (OPERATIONS §7). The payout date is the model's convention, one month before the first instalment, on its day; the statement dates only the contract.
  - `room[2027-04]`, measured on 2027-04-15, is unchanged: its window's minimum is 2027-05-09, the day before the wage, and instalment 1 falls on the wage's day.
  - Later rooms fall by the instalment A = 995.21 on average, but by 0, A or 2A month by month, as instalments that fall on a weekend, before a delayed wage, move a window's minimum: May −A, June −2A, July 0, August −A, September −2A, October 0, November and December −A.
  - The rule of the 15th puts that much less into the wrappers: 7 961.68 less into OKI in 2027.
- **Balances:**
  - `loan_debt[car]` is −40 000 from 2027-04-10 and climbs to 0.
  - `konto` is not lower month by month: the instalments come out of the wrappers. After each 15th konto is the floor less the window's minimum, so it is 995.21 lower only from an instalment's 10th to the next 15th, 995.21 higher from 2027-06-15 to 07-10 and from 09-15 to 10-10, and equal otherwise.
- **Net worth:**
  - It falls by 40 000 on 2027-04-10. The car isn't a place: the household didn't ask to value it.
  - After that, each instalment's principal moves money between two of our own places, so net worth falls only by the interest, and by the wrappers' lost growth.
  - To count the car, the household would add a holding place priced by a depreciation input. That is a separate amendment, and the plan would show its effect.

**At the overpayment** (`on` = 2028-01-08):
- **Instalment 9** is due 2028-01-10, after K. It is planned with the overpayment known.
  - `overpay_paid[car, 9]` = min(5 000, debt before 9 less its principal) = 5 000.
  - So the `overpay` leg of `loan_instalment[car, 9]` moves 5 000 from `konto` to `loan_debt[car]` on 2028-01-10.
- **Shorten:** `instalment` stays 995.21 and the debt falls faster. The loan ends at instalment 42, on 2030-10-10, a part instalment of about 416.65: six instalment dates early, since 43 to 48 book nothing. The interest saved, about 1 550, adds more than one instalment to the 5 000's five.
  - The last instalment is the remainder, by `min(debt_before, …)`.
  - Instances after it have no movement, since `when debt_before > 0` is false.
  - The plan prints the last amount; the date, 2030-10-10, follows from the schedule.
- **Total interest falls** by the interest the 5 000 would have carried to the old end.
- **First day anything differs:** 2028-01-10, the overpayment itself.
  - **The December room stands.** `invest[*, 2027-12]` was booked on 2027-12-15, before the version was known. Its room window (2027-12-15 to 2028-01-14) reaches the overpayment, but its look-ahead read at its own cut, 2027-12-15 (`ENGINE.md` §4.2), which doesn't see the overpayment, and a booked state pay isn't re-measured by a later version (L17).
  - **The next difference** is `room[2028-01]` on 2028-01-15, which is about 5 000 lower.
  - Later instalments are unchanged until the early end, when room rises: by about 578.56 at instalment 42, then by 995.21 in each of the six months after. In 2028 the IKE fills in April instead of March.
- **Choosing `reduce` instead** (`overpay_choice[car] = reduce`) gives a hole wherever `instalment[car, k]` is read, from instalment 1, since the `reduce` rule (`loans.tally`) has no guard on k: "Nadpłata ze zmniejszeniem raty: bank przelicza ratę; zasada przeliczenia nie jest wpisana." The bank's rule is not in the model, and the model doesn't guess it.

**Asks**
- `konto_balance`: 2027-06-30 is higher by one instalment, 7 755.40 (instalment 3 falls on Saturday 2027-07-10, before the wage of the 12th, and moves June's window minimum); 2027-12-31 is unchanged.
- `net_worth`: lower at every date from 2027-04-10. After the overpayment it is above the loan alone from 2028-02-10.
- `ike_left`: unchanged, 0 for 2027 (the IKE filled on 2027-03-15, before the loan was known) and for 2028.
- `taxes_paid`: 2028 is 24 zł lower, 68 973.00, since `oki_tax[2027]` falls from 368 to 344 with less in OKI.
- **Unchanged:** ZUS, health and PIT. A consumer car loan's interest isn't deductible, and no law part reads the loan.

**Register:** nothing new. `car_loan` cites the contract and `car_overpay_2028_01` the household, and nothing is assumed. Each advance moves every register's cut line and drops the `held` entries for the months it passes.

**Warnings:**
- None. `WARN-ENDED` belongs to an `end` (OPERATIONS §10), and there is none here; after the last instalment every pay of `car` books nothing, and `loan_debt[car]` reaches 0.
- Within 2028-01-10 konto passes through −4 195.21, after the overpayment and before the wage, but `WARN-NEGATIVE` tests end-of-day balances (787.71 that day). konto is below `keep_floor` from 2028-01-10 to 01-13, which no warning covers: December's room was taken before the overpayment was known.

**Depends on:**
- L8: no instance where `when` is false;
- L10: the `on` advances;
- L13: `k >= loan_terms[l].n` and `k > 1` as guards;
- L17: booked state pays aren't re-measured.
