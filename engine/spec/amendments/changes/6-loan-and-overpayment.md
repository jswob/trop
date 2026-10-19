# Change 6: a new loan, and an overpayment

**Statement (household):** "Bierzemy kredyt na samochód: 40 000 zł, 48 rat równych, 8,99% rocznie, pierwsza rata 10 maja 2027. Bank wypłaca pieniądze salonowi. Umowa z 2 kwietnia." Later: "Przy racie w styczniu 2028 nadpłacimy 5 000 zł; umowa mówi, że nadpłata skraca okres."

**Operation:** add a fact (OPERATIONS §2.1), with `add_keys`, twice. **Kind:** `add`. **Known:**
- 2027-04-02, then 2028-01-08;
- `on` advances to each in turn.

## Diff

```diff
--- reality.tally
+++ reality.tally
-  on 2026-10-19
+  on 2027-04-02

--- facts/2027.tally   (new file, or added to change 1's)
+part facts_2027
+  tally 1
+
+source car_contract "Umowa kredytu samochodowego, 2027-04-02"
+
+version car_loan
+  cite car_contract
+  known 2027-04-02
+  seller + [car_dealer]
+  loan_seller[car] = car_dealer "§ 1: wypłata na rachunek sprzedawcy"
+  loan_terms[car] = (40000, 48, 0.0899, 2027-05, 10) "§ 2"
+  opening[loan_debt[car]] = 0
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
+  known 2028-01-08
+  overpay[car, 9] = 5000 "nadpłata przy 9. racie, styczeń 2028"
```

**Lines:** 14 added for the loan and 5 for the overpayment, plus the `on` line each time. Lines edited: the `on` line only.

**New keys checked against closed-world rows.** Two keys are added:
- **`car`**, in the open index `loan`: its rows below bring it from the version's `known`, as `laptop_loan`'s rows bring `laptop` (an open index takes no `loan + [car]` line, `LOAD-KIND`). It is read by `loan_seller`, `loan_terms`, `overpay`, `overpay_choice` and the place `loan_debt[car]`'s opening. The base rows are keyed to `laptop`, or keyless in `laptop_loan`'s version (`overpay = 0`), so none covers `car`. All five are stated.
- **`car_dealer`**, in `seller`, for the place `sellers[car_dealer]`. It is an outside place, which needs no opening.

There are no new holes.

## What it changes, in words

The loan's defs are recurrences over `inst` (1..480), so `car` gets its own schedule with no new def.

**At the loan** (`on` = 2027-04-02):
- **The rate:** `inst_rate[car]` = periodic(8.99% nominal, month) = 0.0899 / 12.
- **The instalment:** `annuity[car]` = round(40 000 × i × (1 + i)^48 / ((1 + i)^48 − 1), 0.01) = **995.21**.
- **The first instalment** (2027-05-10, a Monday) is 299.67 interest and 695.54 principal.
- **Movements:**
  - `loan_out[car]` on 2027-04-10 moves 40 000 from `loan_debt[car]` to `sellers[car_dealer]`.
  - `loan_instalment[car, k]` (principal, to `loan_debt[car]`) and `loan_interest[car, k]` (to `kredytodawcy`) run on the 10th of each month, from 2027-05-10 to 2031-04-10.
  - The `last` rule makes the 48th instalment repay the remainder exactly, rounding grosze included.
- **First day anything differs:** 2027-04-10, not before the earliest `governs`. The `known` is 2027-04-02, with no `governs` span; the rows hold from the start of time. So the earliest movement is the disbursement.
  - `room[2027-04]`, measured on 2027-04-15, falls by the instalment due 2027-05-10.
  - Every later room falls by about 995 a month.
  - The rule of the 15th puts that much less into the wrappers.
- **Balances:**
  - `loan_debt[car]` is −40 000 from 2027-04-10 and climbs to 0.
  - `konto` is about 995 lower each month than before.
- **Net worth:**
  - It falls by 40 000 on 2027-04-10. The car isn't a place: the household didn't ask to value it.
  - After that, each instalment's principal moves money between two of our own places, so net worth falls only by the interest, and by the wrappers' lost growth.
  - To count the car, the household would add a holding place priced by a depreciation input. That is a separate amendment, and the plan would show its effect.

**At the overpayment** (`on` = 2028-01-08):
- **Instalment 9** is due 2028-01-10, after K. It is planned with the overpayment known.
  - `overpay_paid[car, 9]` = min(5 000, debt before 9 less its principal) = 5 000.
  - So `loan_overpay[car, 9]` moves 5 000 from `konto` to `loan_debt[car]` on 2028-01-10.
- **Shorten:** `instalment` stays 995.21 and the debt falls faster. `debt_before` reaches 0 several instalments before the 48th, about five (5 000 / 995.21), plus a little from the interest saved.
  - The last instalment is the remainder, by `min(debt_before, …)`.
  - Instances after it have no movement, since `when debt_before > 0` is false.
  - The plan prints the exact last date and amount.
- **Total interest falls** by the interest the 5 000 would have carried to the old end.
- **First day anything differs:** 2028-01-10, the overpayment itself.
  - **The December room stands.** `invest[*, 2027-12]` was booked on 2027-12-15, before the version was known. Its room window (2027-12-15 to 2028-01-14) reaches the overpayment, but a booked state pay isn't re-measured by a later version (L17). Without L17, it would be re-measured, and with no `arrears` line it would be a `HOLE-CHANGED`.
  - **The next difference** is `room[2028-01]` on 2028-01-15, which is about 5 000 lower.
  - Later instalments are unchanged until the early end, when room rises by 995 a month.
- **Choosing `reduce` instead** (`overpay_choice[car] = reduce`) gives a hole from instalment 10 on: "Nadpłata ze zmniejszeniem raty: bank przelicza ratę; zasada przeliczenia nie jest wpisana." The bank's rule is not in the model, and the model doesn't guess it.

**Asks**
- `konto_balance` (2027-06-30 and 2027-12-31): lower.
- `net_worth`: lower at every date from 2027.
- `ike_left`: the 2027 row may stay above 0 if the IKE fills later with less room.
- **Unchanged:** taxes, ZUS, health, PIT. A consumer car loan's interest isn't deductible, and no law part reads the loan.

**Register:** two household versions and one new source. Nothing is assumed.

**Warnings:**
- `WARN-ENDED` doesn't fire: after the last instalment every pay of `car` books nothing, and `loan_debt[car]` reaches 0.
- The same check fires on a loan whose schedule leaves a balance.

**Depends on:**
- L8: no instance where `when` is false;
- L10: the `on` advances;
- L13: `k >= loan_terms[l].n` and `k > 1` as guards;
- L17: booked state pays aren't re-measured.
