# The home law: the Republic of Lenia

Made up, like everything in the set. Shared by every household.

## How to read it

- **Acts and rules.** Each act has a code (`PIT`, `SOC`, ...). Its rules are numbered `PIT-1`, `PIT-2`, ...; a point inside a rule is `PIT-3.2`. A key cites the rule, the version and the point: `PIT-3 v3 .2`.
- **Versions.** Every rule has one or more versions. Each version line states three things:
  - **in force:** the days the version is law, `from – to`, or `from – until replaced` (in force until the next version's first day);
  - **published:** the day its text appeared in the Official Gazette; before that day nobody knew it;
  - **governs:** the events and periods it decides. A version decides only what its governs line names. A later version can govern a period that started before it was published (a retroactive version); an earlier version keeps governing the periods it names after it is replaced.
- **Points not repeated.** "Points 1–4 as v1" means those points read word for word as in v1.
- **Figures in backticks** (`MIN-WAGE`, `ISB-R1-<yymm>`) are published by notice in `law/gazette.csv`, read under GEN-6. Every other figure is in the rule's text.
- **The gazette** has one row per published figure: `id, value, unit, published, from, to, rule, note`. A figure for a span (a wage, a limit) is in force `from – to`; an empty `to` means until replaced. A figure for a period (an index for a month or a year) carries the period in its id and is in force from its publication, with an empty `to`; a later row with the same id replaces it from that row's publication (a flash estimate replaced by the final figure, GEN-6.3). A savings-bond series' rates carry the series in the id and are in force for its month of sale. A daily figure (the Central Bank's mid rate) has `from = to =` its day.
- **Figures for contracts.** The gazette also publishes a figure no rule of this law reads, for contracts to refer to: the yearly CPI average `CPI-AVG-<yyyy>` (the year before = 100).
- **Money.** The dukat (DUK) is divided into 100 orts. The floren (FLR) is a foreign currency; the Central Bank of Lenia publishes its mid rate in dukats for 1 FLR.
- **Bodies.** The Revenue Office (RO); the Social Insurance Office (SIO); the Central Bank of Lenia (CBL); Statistics Lenia (STL); the Treasury; the municipality's Family Benefits Office (FBO); the Official Gazette.
- **What isn't written doesn't exist.** A question this law doesn't settle is not settled by any other law.

---

## GEN · General Provisions Act

### GEN-1 · Business days and public holidays
**v1** · in force 2018-01-01 – 2024-12-31 · published 2017-06-01 · governs: days 2018-01-01 – 2024-12-31
1. A business day is a Monday to Friday that is not a public holiday.
2. Public holidays: 1 January, 6 January, 1 May, 3 May, 15 August, 1 November, 11 November, 25 December, 26 December, and Spring Day.
3. Spring Day is a Monday in April, set for each year by a notice (`HOL-SPRING-<yyyy>`) published by 31 October of the year before. For a year with no notice, which Monday of April is Spring Day is not settled; every other day of that year is settled by points 1–2.

**v2** · in force 2025-01-01 – until replaced · published 2024-06-18 · governs: days from 2025-01-01
1. As v1.
2. Public holidays: those of v1 point 2, and 23 June (Midsummer Day).
3. As v1.

### GEN-2 · Deadlines
**v1** · in force 2018-01-01 – until replaced · published 2017-06-01 · governs: every deadline in this law
1. What is due "by" a day may be done on that day or before it.
2. A deadline that falls on a day that is not a business day moves to the next business day.
3. A payment is made on the day the payer's account is debited.

### GEN-3 · Periods and ages
**v1** · in force 2018-01-01 – until replaced · published 2017-06-01 · governs: every period and age in this law
1. A period of n months (or n years) from a day ends on the day of the n-th month (year) after it that has the same number; where that month has no such day, on its last day.
2. A period of n months from the end of a month ends on the last day of the n-th month after that month.
3. "Within" a period means on or before its last day.
4. A person reaches age n on the n-th anniversary of their birth; a person born on 29 February reaches it on 28 February in a year that has no 29 February.
5. A month, unless a rule says otherwise, is a calendar month.

### GEN-4 · Amounts in a foreign currency
**v1** · in force 2018-01-01 – until replaced · published 2017-06-01 · governs: every conversion this law requires
1. An amount in a foreign currency is converted into dukats at the Central Bank's mid rate (`FX-FLR`) of the last business day before the day of the event the rule names.
2. Each amount is converted separately (each invoice, each payment), and the result is rounded to whole orts, half up.
3. A rate a bank applied to the money is not the rate of this law, and is never used under it.

### GEN-5 · Rounding
**v1** · in force 2018-01-01 – until replaced · published 2017-06-01 · governs: every amount this law computes
1. Under PIT, IGT and TT, tax bases, taxes and advances are rounded to whole dukats: under 50 orts down, 50 orts and over up.
2. VAT on an invoice is rounded to whole orts, half up. VAT due or carried in a monthly return is rounded to whole dukats as in point 1.
3. Contributions, benefits, pensions, refunds and fees are rounded to whole orts, half up.
4. A rule may name another rounding for an amount; it then replaces points 1–3 for that amount.
5. Only the amounts named in points 1–4 are rounded, at the step that produces them. Every other amount is carried exactly.

### GEN-6 · Figures set by notice
**v1** · in force 2018-01-01 – until replaced · published 2017-06-01 · governs: every figure this law leaves to a notice
1. A figure this law leaves to a notice is the one the gazette shows in force on the day the rule names.
2. Where no notice is in force on that day, the figure is not set, and what this law computes from it is not settled.
3. A figure for a period (an index for a month or a year) is, on a given day, its latest value published by that day.

---

## PIT · Personal Income Tax Act

### PIT-1 · Taxpayers and the tax year
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: tax years from 2018
1. A resident is a person who stays in Lenia more than 183 days in a tax year, or whose centre of personal or economic interests is in Lenia. A resident is taxed on income from everywhere.
2. The tax year is the calendar year.
3. Each person is taxed on their own income. Spouses are taxed apart, unless they file jointly (PIT-6).

### PIT-2 · Employment income and its base
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: tax years from 2018, and advances paid from 2018-01-01
1. Employment income is gross pay, parental pay (SOC-8) and pensions (SOC-5, SOC-7), counted on the day each is paid, and an employer's contributions to a workplace plan (WCP-2), counted on the day the employer pays them to the plan.
2. Costs: 250.00 DUK for each month in which pay from an employment is paid, for each employment. Parental pay and pensions have no costs.
3. The base is the income, less the social contributions the payer deducted from it (SOC-1), less the costs, and is never below 0.

### PIT-3 · The tax scale
**v1** · in force 2018-01-01 – 2021-12-31 · published 2017-09-01 · governs: the tax for tax years 2018–2021; advances paid 2018-01-01 – 2021-12-31
1. The scale: 18 % of the base up to 90 000.00 DUK, and 33 % of the part above 90 000.00 DUK.
2. The reducing amount: 1 530.00 DUK a year.
3. The tax is the scale on the base less the reducing amount, and is never below 0.

**v2** · in force 2022-01-01 – 2022-06-30 · published 2021-11-15 · governs: advances paid 2022-01-01 – 2022-06-30, and, in PIT-4 v2 point 4, advances paid 2022-07-01 – 2022-12-31; the tax for tax year 2022 until v3 was published, v3 governing it from then on
1. The scale: 18 % of the base up to 120 000.00 DUK, and 33 % of the part above 120 000.00 DUK.
2. The reducing amount: 5 400.00 DUK a year.
3. As v1.

**v3** · in force 2022-07-01 – until replaced · published 2022-06-14 · governs: the tax for tax years from 2022, the whole of 2022 included; advances paid from 2022-07-01
1. The scale: 13 % of the base up to 120 000.00 DUK, and 33 % of the part above 120 000.00 DUK.
2. The reducing amount: 3 900.00 DUK a year.
3. As v1.

### PIT-4 · Advances on employment income
**v1** · in force 2018-01-01 – 2021-12-31 · published 2017-09-01 · governs: advances on payments made 2018-01-01 – 2021-12-31
1. The payer (the employer; the SIO for parental pay and pensions) computes an advance for each month in which it pays employment income. The month's base is the month's PIT-2 base from this payer, rounded (GEN-5.1).
2. The advance is: the PIT-3 scale on this payer's bases of the year to date, this month's included, less the scale on its bases before this month; less 1/12 of the reducing amount, where the employee has filed a reducing statement with this payer for the year; less the month's health contribution to the extent of 7.75 % of its base (SOC-2 v1 .2). The PIT-3 version is the one in force on the payment day. The advance is never below 0 and is rounded (GEN-5.1).
3. The payer pays the advance to the RO by the 20th of the next month.

**v2** · in force 2022-01-01 – until replaced · published 2021-11-15 · governs: advances on payments made from 2022-01-01
1. As v1.
2. As v1, without the health contribution.
3. As v1.
4. For payments made 2022-07-01 – 2022-12-31, the payer computes the advance under point 2 once with PIT-3 v2 and once with PIT-3 v3, and withholds the lower of the two.

### PIT-5 · The yearly return
**v1** · in force 2018-01-01 – 2021-12-31 · published 2017-09-01 · governs: returns for tax years 2018–2021
1. A taxpayer with income taxed by the scale files a return by 30 April of the next year.
2. The base for the year is the sum of the person's PIT-2 bases from every payer, plus business profit taxed under PIT-10 form (a), rounded (GEN-5.1).
3. The tax is the PIT-3 tax on that base (the version governing the tax year), less the child relief (PIT-7), less the year's health contributions to the extent of 7.75 % of their bases (SOC-2 v1 .2), never below 0; rounded (GEN-5.1).
4. The tax less the advances withheld or paid for the year (PIT-4, PIT-10.5 form (a)) is payable by the return's deadline, or, where negative, refunded by the RO within 45 days of the day the return is filed.

**v2** · in force 2022-01-01 – until replaced · published 2021-11-15 · governs: returns for tax years from 2022
1. As v1.
2. As v1.
3. As v1, without the health contributions.
4. As v1.

### PIT-6 · Joint filing by spouses
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: returns for tax years from 2018
1. Spouses may file jointly for a tax year when, for the whole of that year (1 January to 31 December): they are married, they have community property (CIV-1), both are residents, and neither has business income taxed under PIT-10 form (b) or (c).
2. The election is made in the return, for that year only.
3. The joint tax is twice the PIT-3 tax on half of the spouses' combined base. The combined base is rounded (GEN-5.1); its half is carried exactly; the joint tax is rounded (GEN-5.1).
4. Both spouses' child reliefs are deducted from the joint tax, never below 0, and both spouses' advances are deducted from the result.
5. Each spouse's part of the joint tax is in proportion to their own base, in whole dukats; a dukat left over goes to the spouse with the larger base. Each spouse's amount payable or refunded is their part less their own reliefs and advances.

### PIT-7 · The child relief
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: returns for tax years from 2018
1. For each child under 18, the parents together have a relief of 125.00 DUK for each month of the tax year in which the child was alive and under 18 on at least one day.
2. The parents' income is the sum of both parents' PIT-5 bases, PIT-10 form (b) profits and PIT-10 form (c) bases for the year. A parent who has died or is unknown counts with 0.
3. Where the parents' income exceeds 112 000.00 DUK, the relief is reduced by the excess, and is never below 0.
4. The relief is split between the parents in halves, unless both state another split in their returns.
5. Each parent's part is deducted from that parent's tax (PIT-5.3; from the joint tax under PIT-6.4), never below 0. A part not used is lost.

### PIT-8 · Capital income
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: tax years from 2018; payments from 2018-01-01
1. Capital income is: gains from selling fund units; distributions from funds; interest on deposits; bond interest (TSB-5). Units held in a wrapped account (PIT-9) or a workplace plan (WCP) are not under this rule.
2. The gain from a sale is its proceeds (price times units, less the sale fee) less the cost of the units sold. A lot's cost is the price paid for it with the purchase fee. A lot acquired by gift or inheritance costs its units times the fund's price of the last valuation day on or before the day of the gift or of the death, and its acquisition day is that day.
3. Units sold are matched to lots first in, first out, by acquisition day, within the account where they are held. A lot moved to another account of the same holder keeps its cost and acquisition day. Lots in different accounts are never matched together.
4. All of a person's gains and losses of a tax year are added. Tax: 19 % of the year's net gain, less the losses carried in under point 5, base and tax rounded (GEN-5.1), stated in the return by 30 April of the next year, with no advances.
5. A year whose sales net to a loss has no tax under point 4. The loss is carried into the 5 following tax years: in any one year at most half of that loss is used, and only against a net gain; older losses are used first.
6. A gain on units held more than 5 years since their acquisition is exempt. For a lot acquired by gift or inheritance, the holding counts from the day the giver (or the deceased) acquired it.
7. Distributions and deposit interest: the payer withholds 19 % of each payment, rounded (GEN-5.1). That ends the tax on them.

### PIT-9 · The wrapped account
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: contributions and withdrawals from 2018-01-01
1. A person may hold one wrapped account, with a fund house or a broker.
2. Contributions in a calendar year are allowed up to that year's `WRAP-LIMIT`. The provider returns the part above it, without what it earned, within 30 days.
3. Gains and distributions inside the account are not taxed while they stay in it.
4. A withdrawal, in full or in part, takes first the contributions not yet withdrawn, earliest year first, and then the earnings (the account's value less those contributions).
5. The earnings withdrawn are taxed at 19 %, withheld by the provider, rounded (GEN-5.1), unless on the withdrawal day the holder is 60 or older and has made contributions in at least 5 calendar years.

### PIT-10 · Business income
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: tax years from 2018; advances for months from January 2018
1. Business income is taxed under one form for each tax year: (a) the scale (PIT-3) on profit, together with the person's other income taxed by the scale; (b) 19 % of profit; (c) a flat rate on revenue.
2. The form is elected by notice to the RO by 20 February of the year, or by the 20th of the month after the month of the year's first revenue. An election holds for the following years until a new notice. Without any notice, form (a) applies.
3. Under (a) and (b), profit is revenue (on the invoice's issue day) less costs (on the cost invoice's day) less the social contributions paid in the period (SOC-3). A loss under (a) or (b) is not carried.
4. Under (c), the base of a month is the month's revenue less the social contributions paid in the month; contributions not used in a month are carried to later months of the same year. The rates by the kind of supply: IT services 12 %; other services 8.5 %; the sale of a business asset 10 %. Each month's base and tax are rounded (GEN-5.1).
5. Advances, by the 20th of the next month: under (a), the PIT-3 tax (the version in force on the month's last day) on the year's profit to date, less the advances already paid for the year; under (b), 19 % of the year's profit to date less the advances already paid; under (c), the month's tax. Never below 0; rounded (GEN-5.1).
6. Under (a) the year is settled in the return (PIT-5). Under (b) and (c), a separate return by 30 April of the next year states the year's tax and the advances paid; the difference is payable by that day, or refunded within 45 days of filing.

### PIT-11 · Cars in a business
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: tax years from 2018
1. Under PIT-10 (a) and (b), a car used in the business and also privately counts 75 % of its running costs (fuel, repairs, insurance) as costs.
2. Its depreciation is a cost: 20 % a year of its initial value (the price net of VAT, plus the VAT not deducted), 1/12 of it for each month, from the month after its entry in the business's asset register to the month of its sale, both included; each month's amount rounded to whole orts, half up.
3. On its sale under (a) or (b): the price net of VAT is revenue, and the initial value less the depreciation counted is a cost.
4. On its sale under (c): the price net of VAT is revenue at the rate for the sale of a business asset.

### PIT-12 · The sale of a dwelling
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: sales from 2018-01-01
1. Income from selling a dwelling, or a share in one, is the price less the sale's costs less the acquisition cost. The acquisition cost is the price paid with the transfer tax and the notary's fees; for a share acquired by inheritance, the inheritance tax paid on that share.
2. The income is taxed only where the sale is before the end of the 5th year after the year of acquisition. A share acquired by inheritance counts as acquired on the day the deceased acquired it; a share from dissolved community property, on the day the spouses acquired it.
3. The tax is 19 % of the income less the exempt part, base and tax rounded (GEN-5.1), stated in the return for the year of the sale.
4. The exempt part is the income times the part of the price spent on the seller's own housing within 3 years after the end of the year of the sale, over the price.
5. Where money is spent under point 4 after the return was filed, the seller corrects the return, and the tax paid in excess is refunded within 45 days of the correction.

### PIT-13 · Rent
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: rent received from 2018-01-01
1. Rent from a dwelling let outside a business is taxed at a flat rate on the rent received: 8.5 % of the rent received in the tax year up to 100 000.00 DUK, and 12.5 % of the part above it; per person.
2. Each co-owner is taxed on the rent times their share. Spouses may state that one of them pays the tax on the rent of both.
3. An advance by the 20th of the next month: the tax on the year's rent to date less the advances already paid; rounded (GEN-5.1). Costs are not deducted.
4. The year is settled in a return by 30 April of the next year.

### PIT-14 · Exempt income
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: tax years from 2018
1. Exempt from PIT: the child benefit (FAM-1); a sum an insurer pays on a person's death, a loan it pays off included; what is acquired by inheritance or gift (it falls under IGT); what a co-owner receives from another to settle their shares (CIV-4); payments to the workplace plan by the state (WCP-3).

### PIT-15 · Business revenue
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: tax years from 2018; advances for months from January 2018; SOC-4 contributions for months from January 2022
1. Under every form of PIT-10, and for the tiers and the settlement of SOC-4 .3 and .5, revenue is counted on the day the invoice is issued.
2. Revenue is the invoice's net price, without the VAT charged on it.
3. An invoice in a foreign currency is converted under GEN-4; the event is the issue day.
4. Where a month's revenue under PIT-10 form (c) falls under more than one rate, the social contributions are deducted from the part at the highest rate first, then from the next. Each part's base is rounded (GEN-5.1); the month's tax is the sum of each part's base times its rate, rounded (GEN-5.1).

---

## SOC · Social Insurance Act

### SOC-1 · An employee's social contributions
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: pay paid from 2018-01-01
1. The employer deducts from an employee's gross pay: pension 9.50 %, disability 1.50 %, sickness 2.45 %. Parental pay and pensions bear none.
2. Pension and disability contributions are due only on the year's gross pay from one employer up to that year's `SOC-CAP`; in the month the cap is crossed, only on the part up to it.
3. Each contribution of a month is rounded (GEN-5.3).
4. The employer adds its own pension contribution of 10.00 % on the same base; it is not the employee's money, but it is credited to the employee's pension account (SOC-5).

### SOC-2 · The health contribution of employees and pensioners
**v1** · in force 2018-01-01 – 2021-12-31 · published 2017-09-01 · governs: pay and pensions paid 2018-01-01 – 2021-12-31
1. 9 % of the base: gross pay less the SOC-1 contributions; for a pension, the gross pension. Rounded (GEN-5.3). Parental pay bears none.
2. 7.75 % of the base is deducted from the tax: from the advance (PIT-4 v1) and in the return (PIT-5 v1).

**v2** · in force 2022-01-01 – until replaced · published 2021-11-15 · governs: pay and pensions paid from 2022-01-01
1. As v1.
2. Nothing is deducted from the tax.

### SOC-3 · A sole trader's social contributions
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: months of activity from January 2018
1. Stage 1: from the day the business starts to the end of the 6th month after the month it starts, no social contributions are due.
2. Stage 2: the next 24 months. The base is 30 % of the `MIN-WAGE` in force on 1 January of the year.
3. Stage 3: every month after. The base is 60 % of that year's `AVG-WAGE-PROJ`.
4. Rates on the base: pension 19.50 %, disability 8.00 %, accident 1.67 %; sickness 2.45 % only where the trader has elected sickness insurance; labour fund 2.45 % only in stage 3. Each rounded (GEN-5.3).
5. In the month the business starts, the base is pro-rated: base times the days of activity in the month over the month's days.
6. They are paid by the 20th of the next month.

### SOC-4 · A sole trader's health contribution
**v1** · in force 2022-01-01 – until replaced · published 2021-11-15 · governs: months of activity from January 2022
1. Under PIT-10 form (a): 9 % of the month's income (revenue less costs), never below 9 % of the `MIN-WAGE` in force on 1 January of the year.
2. Under form (b): 4.9 % of the month's income, with the same floor.
3. Under form (c), by the tier of the year's revenue to the end of the month: up to 60 000.00 DUK, 9 % of 60 % of `AVG-WAGE-Q4`; above 60 000.00 DUK up to 300 000.00 DUK, 9 % of 100 % of it; above 300 000.00 DUK, 9 % of 180 % of it. `AVG-WAGE-Q4` is the figure in force on the month's first day.
4. Each month's contribution is rounded (GEN-5.3) and paid by the 20th of the next month.
5. Settlement by 20 April of the next year. Under (a) and (b): the rate on the year's income, never below the floor for each month of activity, less what was paid. Under (c): for each month of activity, the tier amount of the year's whole revenue at that month's `AVG-WAGE-Q4`, summed, less what was paid. A positive difference is paid by that day; a negative one is refunded on request.
6. The health contribution is never deducted from a tax.

### SOC-5 · The old-age pension
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: pensions starting from 2018-01-01
1. The retirement age is 60 for women and 65 for men.
2. Each year's pension contribution credited to a person's account is 19.50 % of their earnings of the year, as the SIO's statement of earnings shows.
3. The capital on retirement in year R is the sum, over each year y up to R − 1, of the year's contribution times the product of `VAL-IDX-<k>` for every year k from y + 1 to R − 1 (no factor where there is none). The products are carried exactly; the capital is rounded (GEN-5.3).
4. The monthly pension is the capital over the `LE-<age>` in force on the day the pension starts, age in whole years on that day; rounded (GEN-5.3).
5. The pension is paid from the first day of the month after the claim is filed, at the earliest from the month after the age is reached; for each whole month, by the payment day the pensioner chose.
6. Where the person has at least 20 insured years (women) or 25 (men), the pension is topped up to the `MIN-PENSION` in force for the month.

### SOC-6 · Indexation of pensions
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: indexations from 1 March 2018
1. On 1 March each year, every pension that started before 1 March of the year before is multiplied by that year's `PEN-IDX-<yyyy>`; a pension that started later is multiplied too, from the first 1 March after a full year of payment.
2. The indexed pension is rounded (GEN-5.3). A top-up (SOC-5.6) is recomputed on the new `MIN-PENSION` and is not indexed.

### SOC-7 · The survivor pension
**v1** · in force 2018-01-01 – 2025-06-30 · published 2017-09-01 · governs: months 2018-01 – 2025-06
1. A person married to the deceased on the day of the death, and aged 50 or over on that day (or younger and raising the deceased's child under 16), is entitled to a survivor pension: 85 % of the pension the deceased was entitled to on the day of the death, rounded (GEN-5.3), indexed under SOC-6 as if it had started when the deceased's pension started.
2. It is paid from the first day of the month after the death, where the claim is filed within 12 months of the death; otherwise from the month after the claim. The deceased's pension for the month of the death is due in full; where not paid before the death, it falls into the estate.
3. A person entitled to two pensions is paid one: the one they elect in the claim. A new election takes effect from the month after it.

**v2** · in force 2025-07-01 – until replaced · published 2024-12-12 · governs: months from 2025-07
1. As v1.
2. As v1.
3. A person entitled both to an old-age pension and to a survivor pension is paid the one they elect in full, and a part of the other: 15 % for months July 2025 – December 2026, 25 % for months from January 2027. The part is rounded (GEN-5.3). Both together are never more than 3 times the `MIN-PENSION` in force for the month; the part is cut to fit.
4. Point 3 applies on a claim. A claim filed by 30 September 2025 takes effect from July 2025; a later one, from the month after it is filed. Amounts for the months before the SIO's decision are paid as arrears with the first payment after the decision.

### SOC-8 · Parental pay
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: births from 2018-01-01
1. A parent insured as an employee on the day of the birth, or a sole trader with sickness insurance (SOC-3.4), is entitled to parental pay for 52 weeks from the day of the birth (the birth day is day 1).
2. The rate: 100 % of the base for the first 20 weeks and 70 % for the next 32; or, where the parent elects it within 21 days of the birth, 81.5 % for all 52 weeks.
3. The base is the average monthly gross pay of the 12 calendar months before the month of the birth, less 13.45 %, rounded (GEN-5.3).
4. A day's pay is the base times the rate over 30, rounded (GEN-5.3). A month's pay is the days of the period in the month times a day's pay.
5. The SIO pays each month's pay by the 10th of the next month. It is taxed under PIT-4 with the SIO as payer and bears no contributions.

---

## FAM · Family Benefits Act

### FAM-1 · The child benefit
**v1** · in force 2018-01-01 – 2023-12-31 · published 2017-09-01 · governs: months 2018-01 – 2023-12
1. A parent who lives in Lenia with a child under 18 is entitled to 400.00 DUK a month for the child.
2. It is granted by the FBO for benefit periods running 1 June – 31 May. A claim for a period filed by 30 April before it gives the benefit from June; a later claim, from the month the claim is filed.
3. For a child born during a period, a claim filed within 3 months of the birth gives the benefit from the month of the birth; a later claim, from the month the claim is filed.
4. For the month of the birth, the amount is the month's amount times the days from the birth day to the month's end, both included, over the month's days, rounded down to whole orts.
5. It is paid for each month by that month's last day.

**v2** · in force 2024-01-01 – until replaced · published 2023-08-10 · governs: months from 2024-01
1. A parent who lives in Lenia with a child under 18 is entitled to 650.00 DUK a month for the child.
2. to 5. As v1.

---

## WCP · Workplace Capital Plans Act

### WCP-1 · Contributions
**v1** · in force 2019-07-01 – until replaced · published 2019-01-15 · governs: pay paid from 2019-07-01
1. An employee aged 18 to 55 is enrolled in the employer's plan unless they have opted out by notice.
2. The employee's basic contribution is 2 % of gross pay; 0.5 % for a month whose gross pay is at most 1.2 times the `MIN-WAGE`, where the employee has asked for it. An additional contribution of up to 2 % may be declared.
3. The employer's basic contribution is 1.5 % of gross pay; its plan may add up to 2.5 %.
4. Parental pay bears none. Each contribution is rounded (GEN-5.3). The employer pays them to the plan by the 15th of the next month; units are bought at the fund's price of the day the money is credited.

### WCP-2 · Tax on contributions
**v1** · in force 2019-07-01 – until replaced · published 2019-01-15 · governs: pay paid from 2019-07-01
1. The employer's contributions are the employee's employment income (PIT-2.1) on the day the employer pays them to the plan. They are not part of the base of SOC-1 or SOC-2.
2. The employee's contributions are deducted from pay after tax and contributions.
3. Nothing is taxed while units stay in the plan.

### WCP-3 · Payments by the state
**v1** · in force 2019-07-01 – until replaced · published 2019-01-15 · governs: participation from 2019-07-01
1. A welcome payment of 250.00 DUK, once, after the participant's first 3 full months in a plan.
2. A yearly payment of 240.00 DUK for each year in which the participant's own and the employer's contributions together are at least 3.5 % of 6 times the `MIN-WAGE` in force on 1 January of that year.
3. The state credits the yearly payment by 31 March of the next year.

---

## CIV · Civil and Family Code

### CIV-1 · Property of spouses
**v1** · in force 2018-01-01 – until replaced · published 2017-03-01 · governs: marriages from any date, from 2018-01-01
1. Spouses have community property from the day they marry, unless a marital contract sets separate property.
2. Community property is what either spouse acquires during the marriage, their earnings included, and the income (interest, distributions) from either spouse's separate assets.
3. Separate property is: what a spouse owned before the marriage; what a spouse acquires by inheritance or gift, unless the giver states it is for both; and what is acquired in exchange for separate assets.
4. People who are not married have no community property. What they buy together they own in the shares their deed states.

### CIV-2 · The end of community property
**v1** · in force 2018-01-01 – until replaced · published 2017-03-01 · governs: ends of community from 2018-01-01
1. Community property ends on the death of a spouse, on divorce, or by a marital contract, on that day.
2. From that day each spouse (or the estate of the deceased spouse) holds half of every community asset and owes half of every community debt.

### CIV-3 · Succession by law
**v1** · in force 2018-01-01 – until replaced · published 2017-03-01 · governs: deaths from 2018-01-01
1. Where the deceased left no will, the spouse and the children inherit in equal shares, but the spouse's share is never less than 1/4; the children then share the rest equally.
2. A child who died before the deceased is replaced by that child's own children, who take that child's share in equal parts.
3. The heirs hold the estate in their shares from the day of the death, until it is divided (CIV-5). The estate's debts are borne by the same shares.

### CIV-4 · Co-ownership
**v1** · in force 2018-01-01 – until replaced · published 2017-03-01 · governs: co-ownership from 2018-01-01
1. Each co-owner bears the costs and is entitled to the income (rent) of the thing in proportion to their share.
2. A co-owner who paid more than their share of the costs has a claim on the others for the excess, each for their share. Rent collected by one co-owner is owed to the others for their shares.
3. These claims bear no interest unless the co-owners agree otherwise.

### CIV-5 · Division and buyouts
**v1** · in force 2018-01-01 – until replaced · published 2017-03-01 · governs: divisions from 2018-01-01
1. Co-owners (heirs, or the spouses after community property ended) may divide what they own together by an agreement, on any day.
2. A co-owner who takes an asset, or a larger share of it, pays each of the others the value of what they give up: the asset's value on the day of the division, unless the agreement states another value, times the share given up.
3. The payments are due by the day the agreement states.

---

## IGT · Inheritance and Gift Tax Act

### IGT-1 · What is taxed, and the groups
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: acquisitions from 2018-01-01
1. Tax is due on what a person acquires by inheritance or by gift, where the person or the asset is in Lenia.
2. Group I: the spouse, descendants, ascendants, stepchildren, siblings, step-parents, children-in-law and parents-in-law. Group II: siblings' descendants, the parents' siblings, and the spouses of siblings and of stepchildren. Group III: everyone else.

### IGT-2 · When the tax arises, and the base
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: acquisitions from 2018-01-01
1. The tax arises: for a gift, on the day of the gift contract, or without one, on the day the gift is performed (money credited, units transferred); for an inheritance, on the day the court's certificate of inheritance becomes final.
2. The base is the market value of what is acquired on the day the tax arises, less the debts and costs the acquirer takes over (an heir: the estate's debts for their share). Fund units are valued at the fund's price of the last valuation day on or before that day; savings bonds at their value on that day (TSB-3); a dwelling at the value the acquirer declares, or the RO sets.
3. The base is rounded (GEN-5.1).

### IGT-3 · Adding up, the free amount and the scale
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: acquisitions from 2018-01-01
1. What a person acquired from the same person in the 5 years before the last acquisition is added to the last one. The free amount is deducted once; the tax already paid on the earlier ones is deducted from the tax.
2. The free amount is that of the acquirer's group, in force on the day the last acquisition's tax arises: `IGT-FREE-I`, `IGT-FREE-II`, `IGT-FREE-III`.
3. The tax on the amount above the free amount: Group I: 3 % up to 12 000.00 DUK; 360.00 DUK and 5 % of the part above 12 000.00 up to 24 000.00 DUK; 960.00 DUK and 7 % of the part above 24 000.00 DUK. Group II: 7 %; 840.00 DUK and 9 %; 1 920.00 DUK and 12 %, at the same edges. Group III: 12 %; 1 440.00 DUK and 16 %; 3 360.00 DUK and 20 %, at the same edges.
4. The tax is rounded (GEN-5.1).

### IGT-4 · The close family's exemption
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: acquisitions from 2018-01-01
1. What a spouse, a descendant, an ascendant, a stepchild, a sibling or a step-parent acquires is exempt.
2. Point 1 applies only where the acquirer reports the acquisition to the RO within 6 months of the day the tax arose. Without that report, the acquisition is taxed as Group I.
3. The report under point 2 is not required where the acquisition, added up under IGT-3.1, is not above `IGT-FREE-I`.
4. A gift of money is exempt under point 1 only where it was credited to the acquirer's payment account. Point 3 does not lift this condition.

### IGT-5 · Returns and payment
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: acquisitions from 2018-01-01
1. Where an acquisition is not exempt, the acquirer files a return within 1 month of the day the tax arose.
2. The RO sets the tax by a decision; it is paid within 14 days of the day the decision is delivered.

---

## TT · Transfer Tax Act

### TT-1 · Transfer tax on a dwelling
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: purchases from 2018-01-01
1. The buyer of a dwelling from a seller who is not a VAT taxpayer pays 2 % of its market value (the price, unless it is below the market value).
2. Co-buyers each pay on their share of the value.
3. The notary collects the tax on the day of the deed. It is rounded (GEN-5.1).

### TT-2 · The first-home exemption
**v1** · in force 2023-01-01 – until replaced · published 2022-09-29 · governs: purchases from 2023-01-01
1. A buyer's share is exempt where, on the day of the purchase and at any time in the 5 years before it, the buyer owned no dwelling and no share in one.
2. A share of 50 % or less in a dwelling, acquired by inheritance, does not count under point 1.
3. Each co-buyer is judged alone.

---

## VAT · Value Added Tax Act

### VAT-1 · Taxable supplies and the rate
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: supplies from 2018-01-01
1. VAT is due on supplies of goods and services in Lenia by a taxpayer, at 22 % of the net price.
2. Letting a dwelling for living in is exempt.

### VAT-2 · The small-business exemption
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: supplies from 2018-01-01
1. A taxpayer whose supplies in the previous year were not above 200 000.00 DUK is exempt in the current year, until the supply with which the year's supplies first go above 200 000.00 DUK; VAT is due from that supply on, that supply included.
2. For a business that starts during the year, the limit for that year is 200 000.00 DUK times the days of activity in the year over the year's days, rounded (GEN-5.3).
3. Exempt supplies and sales of business assets are not counted.
4. A taxpayer may give up the exemption by notice, from the first day of a month.

### VAT-3 · The tax point and the invoice
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: supplies from 2018-01-01
1. VAT is due on the invoice's issue day. An invoice is issued by the 15th of the month after the supply.
2. The VAT on an invoice is the net price times the rate, rounded (GEN-5.2).
3. On an invoice in a foreign currency, the VAT is also stated in dukats, converted under GEN-4; the event is the issue day.

### VAT-4 · The monthly return
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: months from January 2018
1. A taxpayer files a return for each month by the 25th of the next month: the VAT on the month's invoices issued, less the input VAT deducted in the month, less the excess carried from the month before.
2. A positive result is paid by the same day. A negative one is carried into the next month, or, on request in the return, refunded within 60 days of the return.
3. The amounts are rounded (GEN-5.2).

### VAT-5 · Input VAT
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: purchase invoices received from 2018-01-01
1. The VAT on a purchase for the business is deducted in the month its invoice is received, or in either of the two months after.
2. For a car used in the business and also privately: 50 % of the VAT on its purchase, fuel, repairs and leasing. 100 % where it is used only in the business and a mileage log is kept.

### VAT-6 · Selling a business asset
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: sales from 2018-01-01
1. A taxpayer selling a business asset charges VAT at 22 % of the price, whatever share of the purchase VAT it deducted.

---

## TSB · Treasury Savings Bonds: terms of issue

### TSB-1 · The bonds
**v1** · in force 2018-01-01 – until replaced · published 2017-11-20 · governs: series issued from January 2018
1. The Treasury issues indexed ten-year savings bonds in monthly series named `ISB<yymm>`, sold during that month at 100.00 DUK each. They are registered in the holder's name.
2. Each bond's interest years run from its purchase day; the bond matures 10 years after it (GEN-3.1).

### TSB-2 · The interest rate
**v1** · in force 2018-01-01 – until replaced · published 2017-11-20 · governs: series issued from January 2018
1. In the first interest year, the series' first-year rate `ISB-R1-<yymm>`.
2. In each later interest year: the `CPI-YY-<yyyy-mm>` of the month two months before the month in which that interest year starts, less 100, plus the series' margin `ISB-MARGIN-<yymm>`; never below 0. The index is read as published on the interest year's first day.

### TSB-3 · The bond's value
**v1** · in force 2018-01-01 – until replaced · published 2017-11-20 · governs: series issued from January 2018
1. At the end of each interest year, the interest is added to the bond: the new value is the old value times one plus the year's rate, rounded to whole orts, half up, for each bond.
2. On a day inside an interest year, the value is the value at the year's start times one plus the year's rate times the days elapsed over the interest year's days, rounded to whole orts, half up, for each bond.

### TSB-4 · Redemption before maturity
**v1** · in force 2018-01-01 – until replaced · published 2017-11-20 · governs: series issued from January 2018
1. A holder may ask for any number of bonds of a series to be redeemed early. The Treasury pays them on the third business day after the request.
2. Each bond is paid its value on the day of the request (TSB-3.2) less a fee of 2.00 DUK; the fee is never more than the bond's value less 100.00 DUK.
3. At maturity, the value is paid with no fee.

### TSB-5 · Tax on the interest
**v1** · in force 2018-01-01 – until replaced · published 2017-11-20 · governs: redemptions from 2018-01-01
1. On each redemption, the Treasury withholds 19 % of the amount paid per bond less 100.00 DUK, times the number of bonds redeemed, rounded (GEN-5.1).

### TSB-6 · Inherited bonds
**v1** · in force 2018-01-01 – until replaced · published 2017-11-20 · governs: deaths from 2018-01-01
1. Bonds pass to the heirs with their series' terms.
2. For an heir, TSB-4.2 and TSB-5 read the bond's value on the day of the death in place of 100.00 DUK.
3. Bonds are divided between co-owners only in whole bonds.

### TSB-7 · Bonds of spouses
**v1** · in force 2018-01-01 – until replaced · published 2017-11-20 · governs: deaths from 2018-01-01
1. Bonds registered to one spouse but held in community (CIV-1) are, on that spouse's death, half the survivor's and half in the estate.
2. The heir who takes them in the division of the estate holds all of them as an heir: TSB-6.2 applies to each of them, the survivor's half included.

### TSB-8 · The tax withheld is final
**v1** · in force 2018-01-01 – until replaced · published 2017-11-20 · governs: redemptions from 2018-01-01
1. The tax withheld under TSB-5 ends the tax on the bond's interest. The interest is not stated in the holder's return.

---

## CC · Consumer Credit Act

### CC-1 · Early repayment
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: repayments from 2018-01-01
1. A consumer may repay a consumer credit, in full or in part, at any time.
2. On early repayment, the total cost of the credit is reduced by the costs that relate to the period by which the term is shortened, even where the consumer paid them before.
3. The lender settles the repayment within 14 days.

### CC-2 · The early repayment fee
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: repayments from 2018-01-01
1. The lender may charge a fee only where the agreement sets one and the rate was fixed for the period repaid.
2. The fee is at most 1 % of the amount repaid early where more than one year is left until the agreed end, and at most 0.5 % otherwise; never more than the interest the consumer would have paid between the repayment and the agreed end.

---

## MTG · Mortgage Credit Act

### MTG-1 · Payment holidays
**v1** · in force 2022-07-29 – 2023-12-31 · published 2022-07-14 · governs: instalments falling due 2022-08-01 – 2023-12-31
1. A borrower with a mortgage loan in dukats for their own home, agreed before 1 July 2022, may suspend up to 8 instalments: up to 2 in the third quarter of 2022, up to 2 in the fourth, and up to 1 in each quarter of 2023. One loan per borrower.
2. A suspension is asked for by notice, for an instalment falling due after the notice.
3. For a suspended month the borrower pays nothing, and no interest is charged for it. Every later instalment moves by one month, and the term grows by one month for each suspension.

### MTG-2 · Overpayment
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: overpayments from 2018-01-01
1. A borrower may pay off a mortgage loan in part or in full at any time.
2. After 36 months from the loan's disbursement, no fee may be charged. Before that, the fee is the one the agreement sets, at most 3 % of the amount overpaid and never more than one year's interest on it at the rate in force on the day.

---

## MOT · Motor Vehicles Act

### MOT-1 · Compulsory insurance on a sale
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: sales from 2018-01-01
1. The holder of a car keeps a compulsory liability policy for it.
2. When the car is sold, the seller's policy ends on the day of the sale.
3. The insurer refunds the premium for the period from the day after the end to the policy's own end, as the policy's terms compute it.

### MOT-2 · Road tax
**v1** · in force 2018-01-01 – until replaced · published 2017-09-01 · governs: tax years from 2018
1. The yearly road tax of a car: engine up to 1 600 cm³, 360.00 DUK; up to 2 500 cm³, 720.00 DUK; above, 1 200.00 DUK.
2. It is paid by 31 March for a car registered to the holder on 1 January; for a car registered later, by the end of the month after the registration, for the whole months left in the year, at 1/12 a month.
3. When a car is sold, the seller is refunded 1/12 of the yearly tax for each whole month of the year after the month of the sale, rounded (GEN-5.3).
