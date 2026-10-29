# Edge list

Every kind of tricky thing the case set must hit, one **edge** per hard part, grouped by **axis**. Each edge carries the ids it answers for: the household needs of `tmp/design/inventory.md` (rows marked H), the language concerns `LC-1`..`LC-29` of `tmp/learning/CONCERNS.md`, and the mechanics `MX-01`..`MX-40` of `tmp/design/hard-set.md`. A few edges carry no inventory id: they come from the set's own counts (the counts table below).

**How it's used:** each visible edge gets one probe card, `probes/<axis>/<edge>.md`; each family's questions aim at edges and name them; `INDEX.md` maps edge → probe → questions. The holdout carries a few edges of its own; they are listed there, not here.

**Checked by** `python3 -I tmp/cases/tools/check_edges.py`: every id sits in an edge or in "Outside the set" below, no id is unknown, every edge is well formed.

**Everything in an edge is made up:** the home law, its rates, bodies and figures come in step 4. Where an edge names a period ("from July 2022") or a figure, it is the set's own invention.

## Entry format

```
### <EDGE> · <name>          ★ marks the axis's hardest edge: hit twice, under different rules
- hard: the tricky part
- where: the households that hit it, and how
- key: the key kinds its questions take (value, partial, refusal, by-case, relation)
- probe: the probe's sketch, for step 3
- ids: inventory needs, LC concerns, mechanics (or "none")
- counts: the set's counts it meets (optional; see the counts table)
```

## Axes

| Axis | About | Edges |
|---|---|---|
| NUM | money, units, rounding | 7 |
| TIME | dates, calendars, periods, windows | 7 |
| PTY | people, relations, groups | 5 |
| OWN | places, ownership, holdings | 9 |
| FLOW | transfers and conservation | 3 |
| CUR | two currencies | 3 |
| VAL | figures, sources, versions, prices | 7 |
| LAW | law arithmetic and how laws fit | 14 |
| LIFE | contracts, loans, things with a lifetime | 5 |
| RULE | the household's own rules | 9 |
| REC | records, replay, knowledge dates | 7 |
| SCEN | scenarios and amendments | 4 |
| RAND | uncertainty over paths | 4 |
| ASK | questions, holes, refusals | 5 |
| SURF | how a model is written (shown by writing, not by a key) | 4 |
| | **Total** | **93** |

**The user's emphases** are marked `emphasis` in their edges: future law (LAW-13, SCEN-03, RAND-03), prices and figures from records (VAL-01, VAL-05, REC-05), house rules (RULE-01..09).

---

## NUM: money, units, rounding

### NUM-01 · Amounts, quantities and rates carry their units
- hard: money is exact in minor units and always carries its currency; adding two currencies without a stated conversion is refused; fund units, bonds and shares of a flat are quantities with units; a rate never appears without its period and compounding
- where: all; B (a foreign-currency invoice beside home-currency accounts); A (fund units); D (bond counts, a 13/16 share)
- key: value, refusal
- probe: a home-currency amount plus a foreign-currency amount with no rate: refusal naming the missing conversion; a monthly instalment from a yearly nominal rate
- ids: N-201 N-202 N-204 N-206

### NUM-02 · Day counts and pro-rating by days
- hard: the contract names its day count (actual/365, actual/actual); a refund by unused days across a 29 February; a month pro-rated by its own length
- where: A (the car insurance refunded pro rata at the sale; the consumer credit's interest); B (mortgage interest by days)
- key: value
- probe: a policy year spanning a leap day, cancelled mid-year: the refund by unused days under the contract's convention, and under the other
- ids: N-207 N-308 MX-09

### NUM-03 · Rounding where and as the law rounds ★
- hard: each step rounds by its own rule (to whole units, per bond, to the minor unit) in the stated order; truncating before or after a percentage changes the result; the residue of rounding over decades
- where: A (the payroll advance: the base rounded, the tax rounded, in the law's order); D (bond interest per bond, then the tax on it)
- key: value; relation (the stated rounding vs rounding once at the end differ by a known amount)
- probe: a base and a tax rounded at two steps; the same inputs with the steps swapped give a different key
- ids: N-208 N-210 N-211 MX-09

### NUM-04 · An unclear convention, keyed under each reading
- hard: the law leaves a refund basis or a rounding unstated; the model states each reading; the engine never picks one
- where: A (the consumer credit repaid early: its cost refund read pro rata by time, or by the falling balance)
- key: by-case
- probe: a cost refund on early repayment under two stated readings: one value per reading
- ids: N-212 N-109
- counts: L8

### NUM-05 · Parts and wholes
- hard: a split that must add back to the whole leaves an odd minor unit that a stated rule gives to someone; 100 split three ways; a printed total isn't the sum of printed parts, and display rounding never feeds a computation
- where: D (rent split 13/16 : 3/16; the estate in shares); all (net worth totals vs rounded per-person parts)
- key: value; relation (parts add back to the whole exactly)
- probe: 100.00 split in thirds by a stated remainder rule; the printed total vs the sum of printed parts
- ids: N-213 N-214

### NUM-06 · Which year's money
- hard: every entered amount says whether it is nominal or in a stated year's prices; today's money is a view through a price index; bank and tax amounts stay nominal
- where: D (rent stated in one year's prices and indexed; her spending in today's money over her lifetime); A (the ten-year forecast in today's money)
- key: value
- probe: spending stated in 2025 prices, read as 2030 nominal under a stated index path
- ids: N-216 N-1604

### NUM-07 · Gross, net, sign and zero
- hard: what was stated (gross or net) is kept as stated and the other side derived by law; direction words, not signs; a negative result only where the law allows it (a loss year), elsewhere a floor or a carry; zero is not "not applicable"
- where: A (gross pay stated, net derived; a year whose fund sales net to a loss); B (a month known only as net)
- key: value, partial
- probe: a year whose gains net to a loss: tax 0, the loss carried, never a negative tax
- ids: N-217 N-218

## TIME: dates, calendars, periods, windows

### TIME-01 · A date known only to the month
- hard: a source gives only a month; the day is a hole, never the 1st or the 15th; a key that needs the day names it missing
- where: A (gifted fund units whose purchase the giver recorded only by month)
- key: refusal, partial
- probe: a lot "bought in March 2019" on a 5-year clock ending in that month: partial, the day named missing
- ids: N-301

### TIME-02 · Calendars and dates stated as rules
- hard: business days under a stated holiday calendar (one holiday added from a given year); "by the 20th", "the 10th, or the next business day", "the third-to-last business day"; arrival windows (paid between the 3rd and the 7th) planned at the window's pessimistic end
- where: B (contributions by the 20th, VAT by the 25th, a client paying within a window); A (pay on the 10th); all (deadlines)
- key: value (a date)
- probe: a due date on a weekend before the holiday new that year: the shifted date, with the year before for contrast
- ids: N-302 N-303 N-304 N-1004

### TIME-03 · The period an amount is for, and the day it is paid
- hard: pay for June paid on 10 July falls under July's law where the law keys on payment, under June's where it keys on the period; a time key must never set the law date silently
- where: A (June 2022 pay paid in July, after the rate cut); B (a month's contributions paid the next month)
- key: value
- probe: one pay item for June paid in July across a rate change: the tax under a law keyed on payment
- ids: N-305 LC-9

### TIME-04 · Periods each law declares, split by facts
- hard: no built-in month or year; a benefit period running June to May, a contribution year; a fact changing inside a period (a birth, a death mid-month) splits it
- where: A (a child benefit's June–May period, a birth inside it); D (a death mid-month ends one pension and starts another); B (a contribution year)
- key: value
- probe: a June–May benefit with a birth in October: the first month's amount and the period's total
- ids: N-103 N-306 N-315 MX-33

### TIME-05 · Clocks, windows and deadlines ★
- hard: whole months and years between dates under the rule's own month-end wording (within 6 months of the end of the month of purchase; a 5-year clock from the end of the year of acquisition); windows relative to an event, looking back or forward; an obligation due by a date, not on it; a missed deadline changes the tax
- where: D (the inheritance reported within 6 months of the death, or its exemption is lost); A (a benefit claimed within 3 months of the birth; ages in months)
- key: value; by-case (filed in time or not, when the dossier doesn't say)
- probe: bought 31 August, sold 28 February: inside or outside the window, under the rule's month-end wording
- ids: N-309 N-310 N-311 MX-23
- counts: L6

### TIME-06 · Running totals and repricing within a period
- hard: the tax to date minus what was already withheld; revenue to date sets this month's tier; an input taken from an earlier period of another kind (last year's fourth quarter sets this year's base)
- where: A (the cumulative payroll advance crossing a bracket mid-year); B (revenue to date moving the contribution tier; a base from last year's fourth quarter)
- key: value
- probe: monthly withholding crossing a bracket in October: October's advance
- ids: N-313 N-314 MX-34

### TIME-07 · Order within one day
- hard: pay lands and a house rule runs on the same day; which comes first is stated, never implied by file order; same-day bank rows keep the bank's order
- where: A (pay and the 15th rule on one day, when payday shifts onto the 15th); B (same-day rows)
- key: value; relation (reordering events stated as simultaneous changes nothing)
- probe: pay and the sweep rule both on the 15th: the swept amount under the stated order
- ids: N-318 LC-18

## PTY: people, relations, groups

### PTY-01 · People and their dated relations
- hard: people with birth dates, residence, work status, marital status and death; relations with dates (partners, then spouses; a grandson by representation)
- where: all; D (widowhood; a grandson); B (an unmarried couple)
- key: value
- probe: a couple who marry mid-year: which year's joint rule first reads them as spouses
- ids: N-401 N-404

### PTY-02 · Groups the law computes on
- hard: a tax unit or an estate is a group with dated membership that differs from the household; a joint return is open in some years only; a group's result is traced back to each member
- where: A (the joint return, its tax traced to each spouse); B (unmarried: no joint unit); D (the estate as a party)
- key: value; relation (the members' parts add up to the group's result)
- probe: a joint return's tax split back to each spouse under the stated attribution rule
- ids: N-405 N-409 MX-26

### PTY-03 · Succession down the family tree
- hard: an estate passes in stated shares; a child who died first is represented by his own child; marital property is split before the estate is
- where: D (Zbigniew's death; Adam died first; Kuba takes Adam's share)
- key: value
- probe: a four-branch estate with one branch predeceased: each heir's share as an exact fraction
- ids: N-407 MX-21 MX-27

### PTY-04 · A law reads another party ★
- hard: one person's result reads another's facts or results (an income limit over both parents; a survivor pension from the deceased's record); an allowance passes between people (a relief split between parents)
- where: A (the child relief's income limit reads both parents; its split between them); D (the survivor pension reads Zbigniew's record)
- key: value, by-case
- probe: a relief with a joint income limit; one parent's raise crosses it
- ids: N-408 MX-30 MX-24
- counts: L3

### PTY-05 · Joins across households
- hard: a family event is stated once: one household derives it, the others state it as an outside fact; adding a household changes no answer of another; a result lists only the facts it depends on
- where: E1 (A, B and D as one family: the gift, the estate payouts)
- key: relation (the union reproduces each household's keys); partial (Kuba, whose only known fact is his estate share)
- probe: two households stating one transfer from each side: the union states it once, and both keys hold
- ids: N-105 N-106 N-410 N-1308 N-1705

## OWN: places, ownership, holdings

### OWN-01 · Places, their kinds and lifetimes
- hard: accounts, cash, units, assets valued on dated occasions, debts and claims are places with rules of their own; a place exists only from its opening to its closing; a foreign-currency balance or a pile of units has no single value without a stated rate or price; one shared model of what is owned, not three
- where: all; A (an account opened in 2023: no line before it, a fee only while open); B (the car, a deposit); D (flats valued on dated occasions)
- key: value, refusal
- probe: an account opened 1 March with a monthly fee: the fees in the first quarter; net worth on 15 February
- ids: N-501 N-502 LC-12 LC-17 LC-29

### OWN-02 · Claims and money that isn't ours
- hard: money lent is a claim, not gone; a promised remainder isn't cash for a look-ahead; VAT collected sits in our account but is a debt; a co-owner's share of rent in our account is owed on
- where: B (lent to a friend, paid back in part, the rest promised; VAT collected); D (the co-owners' rent share in Halina's account)
- key: value, partial
- probe: 2 000 lent, 1 200 back, the rest promised: net worth counts the claim; the look-ahead doesn't count the 800
- ids: N-503 N-504 N-505 N-2109 LC-20 LC-21

### OWN-03 · Owed but not yet paid
- hard: accrued interest, next spring's tax on this year's income and this month's VAT belong in net worth on any day; an amount due after the run's end still shows as owed
- where: all (each household has one on an asked date); B (VAT, the year's tax); D (accrued bond interest)
- key: value
- probe: net worth on 31 December, with the year's tax due 30 April, in a run ending 31 March
- ids: LC-11 LC-16
- counts: M8

### OWN-04 · Four ownership regimes
- hard: community property, separate property, 60/40 co-ownership and an estate in shares; ownership is dated and fractional; per-person net worth reads it
- where: A (community property, and the gifted units as separate property); B (the flat 60/40, unmarried); D (community until the death, then the estate in shares)
- key: value, by-case
- probe: one asset under each regime: each person's share
- ids: N-506 MX-20
- counts: M2

### OWN-05 · Events that move ownership ★
- hard: a gift, a death and a buyout move shares on a date; community becomes shares at a death; an item valued at the death and paid out at the division at another value
- where: D (the death, the estate divided, buyouts); A (the gift of fund units)
- key: value
- probe: community dissolved by a death, then a buyout at a later valuation: each heir's payout
- ids: N-508 N-928 MX-21
- counts: M2, twist D

### OWN-06 · Lots
- hard: each lot has its cost, date, currency and owner; a gifted lot's cost is set by law, not by what was paid; lots move between owners and between brokers; accumulating and distributing funds; five or more funds and a hundred or more lots
- where: A (funds in a tax-wrapped account and at a broker; the gifted units moved brokers); D (bonds)
- key: value
- probe: a gifted lot whose cost the law sets at its value on the gift's day, sold later: the gain
- ids: N-509 MX-10
- counts: M1

### OWN-07 · Matching sales to lots
- hard: the law matches, not the holding: first in, first out per account; bonds redeemed by chosen series; partial sales; gains netted per year; a lot bought in a foreign currency at the law's rate
- where: A (FIFO per account across two brokers); D (bonds redeemed by series)
- key: value
- probe: three lots in two accounts, one partial sale: FIFO per account vs across accounts
- ids: N-510 N-2103
- counts: M1

### OWN-08 · Kept-apart parts and their limits
- hard: money inside a place kept apart by kind or year (contributions vs earnings in a wrapped account) and drawn in the law's order and cap; a yearly contribution limit; contribution vintages with clocks of their own
- where: A (the wrapped account's yearly limit, filled first on the 15th; its withdrawal test)
- key: value
- probe: a wrapped account filled across a year change against its limit; a withdrawal's taxed part
- ids: N-513 N-514 N-2104 MX-12 MX-14

### OWN-09 · Indexed bonds in monthly series
- hard: each series has its own first-year rate, then an index read with a lag plus a margin; capitalised yearly; interest per bond, then tax; an early-redemption fee per bond, capped at the interest
- where: D (bonds bought every month; redeemed below a floor)
- key: value
- probe: one series redeemed in its third year: interest per bond, the fee, the tax
- ids: N-2105
- counts: M1

## FLOW: transfers and conservation

### FLOW-01 · Transfers and conservation
- hard: every transfer has its dates, source, destination, amount and cause; each place's change equals its transfers; a transfer between own accounts never changes the total; uncounted cash is an asserted balance with a balancing flow; residuals appear only where an error was planted
- where: all
- key: relation
- probe: a wallet balance asserted twice: the balancing flow between
- ids: N-601 N-603 N-607 N-1807

### FLOW-02 · One event, several legs
- hard: a conversion has one leg per currency; a car sale pays off the lender from the price, with a fee leg; the legs settle together or the event isn't booked
- where: A (the private car sold, its credit repaid from the price); B (the business car's sale)
- key: value; relation
- probe: a sale whose price goes to the lender, the fee and the owner: each leg
- ids: N-602

### FLOW-03 · Shared costs and netting ★
- hard: co-owners' costs and rent net to balances that add up to zero; who owes whom after a shortfall; one obligation shared by several
- where: D (the co-owned flat's costs and rent netted between heirs); B (the 60/40 flat's costs, paid by one)
- key: value; relation (the balances add up to zero)
- probe: three co-owners, one pays all costs, rent comes into one account: the net balances
- ids: N-605 MX-25 MX-28

## CUR: two currencies

### CUR-01 · Which rate for each leg ★
- hard: the law names its rate (the central bank's mid rate of the last business day before the event); the bank converts at its own rate; one transfer read at both; the gap is a currency gain or loss
- where: B (invoices in the foreign currency: revenue and VAT at the law's rate, the money at the bank's)
- key: value
- probe: one invoice issued, paid a month later: revenue at the law's rate, cash at the bank's, the difference
- ids: N-932 MX-06
- counts: M6

### CUR-02 · Converted amounts are whole minor units
- hard: each converted transfer is rounded to whole minor units by a declared rule; a total converted at once is only a view; a foreign amount paid into a foreign-currency account is no conversion at all
- where: B (a foreign-currency invoice paid into a foreign-currency account)
- key: value; relation
- probe: ten conversions vs their total converted once: the stated difference
- ids: N-606 N-1808

### CUR-03 · Revaluation is not a transfer
- hard: a foreign-currency balance or receivable moves in home-currency value with the rate; net worth shows a gain or a loss, and no transfer happens
- where: B (an unpaid foreign-currency invoice at a month end)
- key: value; relation (a revaluation never moves the foreign-currency balance)
- probe: a foreign balance held across a rate change: home-currency net worth before and after
- ids: N-604 MX-06

## VAL: figures, sources, versions, prices

### VAL-01 · Figures from the gazette
- hard (emphasis): law figures live in the gazette with publication and in-force dates; the last entry on or before the date is read; no entry is refused, never read as 0; a value read after its last entry is held there and listed; figures set by a separate notice
- where: all; D (an index not yet published on an asked date)
- key: value, refusal, partial
- probe: one figure asked before its first entry (refusal) and after its last (held, listed)
- ids: N-701 N-702 N-703 N-707 N-712

### VAL-02 · Which date a value is read at
- hard: each value states its reading date: the period's start, a month with a lag, last year's fourth quarter
- where: D (the bond's index read with a lag); B (a contribution base from last year's fourth quarter); A (the wrapped account's limit by year)
- key: value
- probe: an index read with a two-month lag for a series starting in March
- ids: N-704

### VAL-03 · Two dates on every figure
- hard: when a figure governs vs when it was known; a year as known on 31 May vs on 1 August; one figure with a draft source, then a final one
- where: A (the 2022 law as known before and after the mid-year change was published)
- key: value (one per known-as-of date)
- probe: a limit published in draft, then final and different: the answer known before and after
- ids: N-705 N-706

### VAL-04 · A published figure that disagrees with its law ★
- hard: a record states a figure; the law computes another; both are kept, the gap is named with its cause, and neither is silently corrected
- where: A (the broker's yearly tax statement after the gifted units moved brokers); D (a pension-office decision that misapplies the law)
- key: value with a named gap; partial
- probe: a statement's figure vs the law's: both, and the difference with its cause
- ids: N-715 MX-29
- counts: L9, twist 5

### VAL-05 · Prices come from records
- hard (emphasis): fund prices, the reference rate's fixings and the bank's exchange rates come from their issuers' files, never from rules; a holding is units × that day's recorded price; a day with no price reads the last one, stated; a forecast resolves prices by month
- where: A (fund prices from the market files); B (the reference rate for the mortgage's reset)
- key: value
- probe: units valued on a day with no price; a forecast sale mid-month at the month's price
- ids: N-512 N-319

### VAL-06 · Indexed values and histories
- hard: a price index is a history plus an assumed future, per scenario; a stated amount indexed yearly; decades of stated earnings, each multiplied by an index ratio, into a pension
- where: D (her pension from a stated history; rent indexed; the bond index); A (the price index in the forecast)
- key: value
- probe: three years of earnings indexed into a base by stated yearly indices
- ids: N-718 N-810 N-1204 MX-17

### VAL-07 · The assumption register
- hard: every default, assumption and held value a result read is listed, transitively
- where: D (the register behind a lifetime forecast)
- key: value (a list)
- probe: a forecast reading an assumed index that reads an assumed rate: both listed
- ids: N-713 N-1704

## LAW: law arithmetic and how laws fit

### LAW-01 · Edges: thresholds, brackets and tables
- hard: "more than" vs "not less than"; piecewise formulas with stated edges; tables keyed by bands with open and closed edges, and by age in months; caps, floors, the smaller of; answers just below, at and just above an edge
- where: A (the tax scale; a benefit by the child's age in months); B (contribution tiers by revenue)
- key: value
- probe: one scale at the edge minus one minor unit, at it, and plus one
- ids: N-801 N-802 N-709 N-711 N-809 N-1806

### LAW-02 · Cliffs and tapers
- hard: one unit more loses a fixed amount (a cliff); a relief cut unit for unit above a limit; where the taper applies matters
- where: B (contribution tiers: a cliff at a revenue edge); A (the child relief cut above its income limit)
- key: value
- probe: income at the limit, and one unit above: the relief before and after
- ids: N-804 MX-31

### LAW-03 · Law versions by date ★
- hard: a mid-year change with transitional rules (the payroll compares old and new and keeps the better); a superseded version still run for the year it governs; "is this law in force on this date?"; a version with no stated span is refused, never read as forever; a rule with a span of its own
- where: A (the twist: a rate cut from July 2022; the 2021 rules still run in 2022); D (a survivor pension from July 2025)
- key: value; relation (a change from a date changes no answer before it)
- probe: a cut from 1 July with a transitional comparison: June's and July's advance
- ids: MX-04 N-908 LC-6 LC-10 LC-14
- counts: L1, L4, twist A

### LAW-04 · Retroactive law
- hard: a version covering a closed period is booked as a refund or as arrears on the day it is known, never by rewriting the past; back pay paid late or in instalments
- where: A (the July cut applied to the whole year in the return: a refund); D (pension arrears paid late)
- key: value with its direction
- probe: a cut enacted in July for the whole year: the refund in the return
- ids: N-921 MX-05 LC-23
- counts: L2, twist A

### LAW-05 · "As if"
- hard: the law runs itself on changed facts: old and new rules on the same pay; keep the better, or the capped
- where: A (the payroll's old-vs-new comparison in 2022)
- key: value
- probe: a tax under two rule sets, the lower kept
- ids: N-907 MX-32

### LAW-06 · Elections
- hard: a choice is a stated fact with a scope (joint filing per year, a tax form per year); each side can be run and shown, none picked; an election left unstated is refused
- where: A (joint filing per year); B (the tax form per year, both forms side by side)
- key: value per side; refusal
- probe: a year with no filing election stated: refusal; with one: the tax
- ids: N-912 N-1313 N-109 LC-24
- counts: L5

### LAW-07 · Eligibility and classification from facts
- hard: a joint return checked against its conditions; a survivor pension's conditions; the activity picks the flat-tax rate; a VAT exemption lost past a revenue threshold mid-year
- where: A (the joint return's conditions); B (the flat-tax rate by activity; the VAT threshold); D (the survivor pension's conditions)
- key: value, refusal
- probe: revenue crossing the VAT-exemption threshold in October: from which invoice VAT applies
- ids: N-902 N-913 N-914

### LAW-08 · Pipelines across laws and forms
- hard: several laws on one income, in a stated order, across forms; a law applies to what it taxes with no hand wiring; a job and a business side by side; gross to net with a yearly contribution cap
- where: A (gross to net: contributions capped at a multiple of the average wage, the monthly advance, the return); B (Piotr's business and Ola's job)
- key: value
- probe: the contribution cap crossed in November: December's net pay
- ids: N-901 N-903 N-905 N-2102 N-1608 LC-13 MX-36

### LAW-09 · Carry-forwards with caps
- hard: a loss carried for 5 years, at most half of its original size in any one year; the original size of each year's loss is read, not what is left
- where: A (a fund loss carried forward)
- key: value
- probe: a loss of 10 000 used against gains of 3 000, 8 000 and 9 000
- ids: N-919 MX-11

### LAW-10 · A result pending until a window closes
- hard: a sale's tax depends on a later window (reinvestment within 3 years); until the window closes the key gives each branch and the date that settles it; purchase tax with an exemption, and the 5-year clock
- where: D (the rental flat's sale and its reinvestment relief); B (the flat's purchase)
- key: by-case
- probe: a sale in year 1, part reinvested in year 2, the window closing in year 4: the tax per branch
- ids: N-923 N-2106
- counts: L7

### LAW-11 · A law reads history
- hard: an exemption reads how a gift moved (by bank transfer or in cash); gifts from one giver cumulated over 5 years; a yearly settlement reads what was already paid; results of earlier years
- where: A (the gift's exemption reads its transfer and earlier gifts); B (the yearly contribution settlement reads the monthly payments)
- key: value
- probe: two gifts four years apart from one giver: the second's taxable part
- ids: N-926 N-927 MX-22

### LAW-12 · An exception through a chain
- hard: an exemption, its exception, and an exception to that; where the middle rule doesn't apply, the outer one must not let the case through
- where: D (the inheritance exemption, its reporting condition, and that condition's exception); A (the gift)
- key: value
- probe: three nested rules with the middle one inapplicable
- ids: LC-2

### LAW-13 · Future law
- hard (emphasis): future law is a stated what-if from a date, or a change as a draw with stated odds, run on every path; a change from a date moves no answer before it
- where: A (the gains tax at 25 % from 2030); B (a law change as a draw)
- key: value per path; relation
- probe: a rate change from 2030 on a forecast to 2035: every answer before 2030 unchanged
- ids: N-1405
- counts: L10

### LAW-20 · A circular rule the law solves by its own named method
- hard: two amounts each sized from the other (a deduction lowers the income a credit is computed on, and the credit lowers what may be deducted); the law names its own way out, a fixed sequence of steps (an interim deduction, an interim credit, then the final deduction and the final credit), and also allows an iteration until the change is under one unit; the result is the named method's, not the point where both rules hold; a loop the law names no method for stays refused (RULE-04)
- where: no visible household; carried in integration by the 79 cases' models (UL-07, UL-08): a self-employed person's health-insurance deduction and premium credit
- key: value (the method's result); relation (a closed form where the rule is linear; the iteration within one unit of the fixed point)
- probe: a deduction and a credit sized from each other: the law's four steps, then its iteration, each against its closed form
- ids: MX-35

Its number stands well past the visible LAW edges because the holdout keeps edges of its own.

## LIFE: contracts, loans, things with a lifetime

### LIFE-01 · A thing written once, with a lifetime
- hard: a job, a contract, a loan or a lease is written once, with blanks that change on dates (a raise from a date; rent indexation); the same blocks serve one day and a lifetime
- where: all; D (a run over her lifetime); A (a ten-year forecast)
- key: value
- probe: a contract with a raise on a date and an indexation clause
- ids: N-1001 N-1002 N-102

### LIFE-02 · When a thing ends ★
- hard: a car's sale ends its insurance (refunded pro rata), its credit (repaid early) and its tax; closing a contract ends what it carries; reading an ended thing, leaving an end out, or two things in one exclusive role (rent and an owned flat as the home) warns; a what-if lists what it ends
- where: A (the private car: credit, insurance, tax); B (the business car: the VAT share, tax on the sale; rent overlapping the owned flat)
- key: value; refusal (a warning)
- probe: a car sold with an insurance, a credit and a parking contract: each end and its money
- ids: N-1003 N-1010 N-1011 N-1012 N-1702
- counts: M3

### LIFE-03 · Loans
- hard: annuity or equal principal; the day count; a rate reset on a reference rate; a payment holiday; an overpayment that shortens the term or lowers the instalment; an early payoff; a payoff by a life insurer
- where: B (the mortgage: reset, holiday, overpayment); D (paid off by the insurer after the death); A (the consumer credit repaid early)
- key: value
- probe: an annuity loan with a reset at month 12 and an overpayment at month 18: the instalment after each
- ids: N-1005 MX-15 MX-16
- counts: M4

### LIFE-04 · Benefits recalculated
- hard: a running benefit recalculated when an input changes or the law raises it; parental pay, a child benefit, unemployment, a state pension
- where: A (parental pay, the child benefit); D (her pension, the survivor pension)
- key: value
- probe: a benefit raised by law mid-period: the period's total
- ids: N-1007 N-2107

### LIFE-05 · A sole trader's month and year
- hard: invoices on payment terms, paid late or in part; monthly VAT; contributions in stages (reduced in the first years); tiers by revenue to date; the year's settlement
- where: B (Piotr)
- key: value
- probe: one month of three invoices, one paid late: VAT due, the contribution, the tier
- ids: N-2101

## RULE: the household's own rules

### RULE-01 · A rule on the real balance on a date
- hard (emphasis): "on the 15th, keep a floor and put the rest into the wrapped account first, up to its limit"; the literal rule is the default
- where: A (the 15th rule); B (a tax reserve first, then overpay the mortgage)
- key: value
- probe: the 15th rule with a floor and a limit nearly reached
- ids: N-101 N-1101 N-1109
- counts: R1

### RULE-02 · What a rule may see
- hard (emphasis): a look-ahead sees only planned money, never promised; room measured over a group of accounts, taken from one; a pay that reads a balance must be visible to the look-ahead; a fallback reads only realised state, with money that settles days later; a rule may react to a published signal, never to the model's own forecast
- where: A (how much can we invest on the 15th, given the instalments ahead); B (the reserve over two accounts); D (bonds redeemed below a floor, the cash arriving days later)
- key: value
- probe: the lowest balance ahead over planned items only, one promised inflow left out
- ids: N-1102 N-1103 N-1105 N-1107 LC-15 LC-19

### RULE-03 · Rules competing for the same money ★
- hard (emphasis): two rules want the same money; with a stated priority it decides (ties by a stable id); with none, a refusal naming both
- where: B (reserve, overpayment and investing, with a stated priority); A (two rules with no priority: a refusal)
- key: value, refusal
- probe: two sweeps of one surplus, first with a priority, then without
- ids: N-1104
- counts: R2

### RULE-04 · A loop
- hard (emphasis): rules that depend on each other in a loop are refused unless the law names a solver; no rule of the home law or the house rules names one (the solver side is LAW-20)
- where: one household's step places it (a refusal naming the cycle)
- key: refusal
- probe: one rule sized from a second's result, the second from the first's
- ids: N-918 MX-35
- counts: R3

### RULE-05 · "Once, when…" on a state
- hard (emphasis): a rule triggered by a state (buy the car once savings reach a sum), whose consequences (its instalments) the 15th rule must then see; a rule on a state (redeem when below a floor)
- where: A (the car bought once savings reach a sum); D (bonds redeemed below a floor)
- key: value (a date)
- probe: savings growing monthly: the trigger date, and the first instalment the 15th rule sees
- ids: N-1108
- counts: R1

### RULE-06 · A rule changed "as if since January"
- hard (emphasis): a new version of a house rule, stated in April to apply from January; what was done stays done; the gap between what was done and what the rule now says is named, never rebooked
- where: one household's step places it
- key: value, partial
- probe: a sweep floor raised in April "from January": the shortfall for January to March, named
- ids: none
- counts: R4

### RULE-07 · What a rule meant vs what the records show
- hard (emphasis): the rules say a standing order ran; the records show it failed one month; the records win and the gap is named; "did we follow our rule?"
- where: A (did we follow the 15th rule?); one more household's step places a failed standing order
- key: value with a named gap
- probe: a monthly order with one month missing from the bank rows: the balance from the records, the gap named
- ids: N-1221
- counts: R5

### RULE-08 · Parameters estimated from history
- hard: a rule's parameter is an estimator over history up to the decision date (the floor is three months' mean spending); spending fitted by a stated estimator; the fitting is the modeller's, the estimator is stated
- where: A (the floor from spending up to each 15th)
- key: value
- probe: a floor as the mean of the last three months' spending, on two dates
- ids: N-1211 N-1212 N-1214 MX-39

### RULE-09 · A rule on a forecast
- hard (emphasis): a rule that reads the forecast ("stop buying if any path runs dry before 85") is a decision on a stated date over that date's paths; a statistic across paths read inside a path is refused
- where: each household's forecast rule (one per household)
- key: value, refusal
- probe: a rule reading the share of ruined paths, decided on a stated date; the same rule read inside a path: refused
- ids: N-1409
- counts: R1

## REC: records, replay, knowledge dates

### REC-01 · Records are the past; the model is the rest
- hard: before today the records; after it the model; the model's own version of the past is computed and compared with the statements, never booked; every account in the records is covered by the model or flagged; agreement is evidence only on real imports (these are all made up, and say so)
- where: B (the model's past against the bank statements; an account the model doesn't name)
- key: value; relation
- probe: a statement with an account the model doesn't name: flagged
- ids: N-1201 N-1202 N-1217 N-608

### REC-02 · A statement with errors ★
- hard: a missing row, a duplicate, a reversal, an export covering half a month; the balance-after column exposes them; residuals by cause; each institution's own CSV dialect
- where: B (the twist: all four kinds, in the bank's dialect); D (a second instance, in Bank Olcha's dialect: a repeated row, a missing row)
- key: value; relation (dropping the duplicate moves the balance by exactly its amount)
- probe: five rows with a duplicate and a balance-after column
- ids: LC-25 N-1218
- counts: M5, twist B

### REC-03 · Awkward rows
- hard: a refund, a split bill, a transfer between own accounts, a card's billing cycle, a 13th pay, 0 % instalments, each read for what it is
- where: B; A
- key: value; relation (an own transfer moves no total)
- probe: a card cycle closing on the 20th with a refund inside it
- ids: N-1215
- counts: M5

### REC-04 · Matching and certainty
- hard: rows matched to expected events on counterparty, window and tolerance; one registry of counterparties (a client renamed after a merger); a part payment asks for the remainder; each amount's certainty: happened, expected, promised, missed; probability words through a stated lexicon
- where: B (client invoices matched to rows; a renamed client; a client's "probably next week")
- key: value
- probe: an invoice of 2 000 paid 1 200: matched, the remainder asked for, its certainty
- ids: N-110 N-1205 N-1206 N-1207 N-1208 N-1210

### REC-05 · Records as the law's inputs
- hard (emphasis): past years' law reads payslips, filed returns and institutions' statements as inputs; where only net is known, gross is inferred by the law and marked so
- where: A (past returns read the payslips and the employer's yearly statement); B (Ola's missing payslip: gross from net)
- key: value, partial
- probe: a missing payslip whose net is known: gross inferred under the stated rates
- ids: N-1203 N-719 N-1213

### REC-06 · Clocks and knowledge dates
- hard: booking date vs value date; what was known when; "what did we think in May 2022?"; what changed since last month; today's state settled before any what-if
- where: A (the year as known in May 2022 and after); B (is today's pay in?)
- key: value (one per known-as-of date)
- probe: one question as of one date, known as of two dates
- ids: N-1216 N-1219 N-1222

### REC-07 · Replaying over recorded history
- hard: a fixed plan replayed over the recorded market history from every start month; runs from a past start that use only what was known then; residuals by cause
- where: D (a withdrawal plan replayed over the market files); B (a run from a past start)
- key: value (a table); relation
- probe: a 3-year plan over 6 years of monthly prices, from each start month: the worst and the best
- ids: N-1220 MX-18 MX-38

## SCEN: scenarios and amendments

### SCEN-01 · Scenarios and amendments
- hard: a scenario is a base, a start and changes on stable ids; the common changes (a new job, next year's law, a child, a new loan with an overpayment) stay local; an amendment's effect is shown before it is applied; an exact difference between two runs; only the household's own scenarios; a draft counts only once a person confirms it
- where: A (an unconfirmed car sale doesn't count; an amendment's effect); B (an exact difference between two runs)
- key: value; relation
- probe: one amendment (a new loan and an overpayment): its difference from the base
- ids: N-107 N-111 N-1209 N-1301 N-1306 N-1310 N-1311 N-1703

### SCEN-02 · A counterfactual past ★
- hard: a branch date, and which later facts survive it (had they kept the car: the insurance renews, the credit runs on, the sale never happens, the 15th rule sees other money)
- where: A (had they kept the car); B (Piotr employed instead, from a past date)
- key: value
- probe: a branch at a sale's date: which later facts survive
- ids: N-1304

### SCEN-03 · Linked overrides
- hard (emphasis): "8 % inflation for 3 years" moves everything that reads the index (rent, pension, bonds) and says what follows it
- where: D
- key: value
- probe: an index overridden for 3 years, then back on the base path: the rent in year 5
- ids: N-1305

### SCEN-04 · Named extreme paths
- hard: a −50 % crash, a flat decade, a regime switch, each a stated path run with the law and the house rules
- where: A
- key: value per path
- probe: a crash in year 2 of a ten-year plan: the end balance, and the first year below a floor
- ids: none
- counts: M7

## RAND: uncertainty over paths

### RAND-01 · Random families
- hard: fat-tailed returns; two correlated funds; client payments as a compound Poisson stream with random dates; a random lifetime from a stated life table; seeds stated, and the same seed gives the same answer
- where: B (client payments); D (returns, her lifetime)
- key: value (random: reference, its standard error, a target); relation
- probe: a lifetime drawn from a four-age table: its mean over seeded paths
- ids: N-1402
- counts: M7

### RAND-02 · Closed forms
- hard: the expected lifetime is an exact sum over the table; zero volatility gives the plain projection; a lognormal mean; an exact expectation is computed, or a named bound is given
- where: D (her expected lifetime; zero volatility); B (zero variance in client payments gives the plan)
- key: relation (closed form)
- probe: a four-age life table: the expected remaining lifetime, exactly
- ids: MX-19 LC-22
- counts: M7

### RAND-03 · Law on every path ★
- hard (emphasis): tax and benefits computed on every path; a rule over a random stream says whether it reads each event or the period (a deductible per claim or per year); a law change as a draw
- where: B (tax on every path of random client payments; a law change with stated odds); D (pension and tax on every path to a random death)
- key: value (random)
- probe: random payments over a year against a tax threshold: the share of paths above it
- ids: N-1403
- counts: M7, L10

### RAND-04 · What a run may say
- hard: bands and quantiles by one stated rule; n of N, the share, its standard error, the horizon; no verdict built in; scenarios on common random numbers; two plans compared path by path, no winner; a rule's breach rate and mean shortfall, and the same with the rule off
- where: B (the reserve rule's breach rate); D (bands of her net worth)
- key: value (random); relation (common random numbers: both scenarios on the same paths)
- probe: two plans on the same seeded paths: the median path-wise difference
- ids: N-1106 N-1302 N-1404 N-1605 LC-28

## ASK: questions, holes, refusals

### ASK-01 · Question primitives
- hard: the value on a date, the first date something happens, the minimum and when, sums over windows; questions read places by kind or role, so a new account counts without editing the question; a question never moves money; the product's eight questions across the set
- where: all
- key: value
- probe: the first date a balance drops below a floor; the minimum and its date
- ids: N-1601 N-1602 N-1606 N-1612

### ASK-02 · Net worth per person, with holes ★
- hard: net worth over chosen people, weighted by dated ownership, at market value and after liquidation; each household names at least one hole in it
- where: A, B, D; E1 (Kuba: only his estate share is known)
- key: partial
- probe: two people, one asset co-owned, one with no valuation: each person's net worth and the hole
- ids: N-1603
- counts: Q3

### ASK-03 · Holes, refusals and warnings
- hard: a missing fact is a hole with its question, never 0; a blank on a form isn't 0; a contradiction is refused, naming both sides; an ambiguous name becomes a question; holes are judged both ways; run warnings (an account below zero, selling more than is held)
- where: all (two or more per household)
- key: refusal, partial
- probe: a form with a blank field the law needs; a sale of more units than are held
- ids: N-916 N-1607 N-1905 N-1906 MX-37
- counts: Q3

### ASK-04 · No advice
- hard: "should she sell the flat?" is refused; both consequences and the breakeven are shown, no winner
- where: D
- key: refusal with values
- probe: "which loan is better?": both costs and the breakeven rate
- ids: N-108 N-1802

### ASK-05 · Explain
- hard: every number traces to its causes: this transfer, this rule, this fact
- where: A
- key: value (the causes it must name)
- probe: a net pay and its causes, in order
- ids: N-1701

## SURF: how a model is written

These edges show in the writing (session 3's gap log, session 5's reading test), not in a key. Their probes are small passages to write.

### SURF-01 · No bare figures
- hard: every figure in a rule is a named fact from the gazette, a document or the house rules; only small integers (days of the month, 12, 100) may appear bare; a figure nothing reads is flagged
- where: all
- key: none (shown by writing)
- probe: a rule with a floor of 1 000 and the 15th: which are facts, which may stay bare
- ids: N-215 N-1013 LC-1

### SURF-02 · Surface words
- hard: "is" vs "equals?"; minus vs "not applicable"; a condition vs its value; one order for the language; aggregates over lots, categories and heirs
- where: all (the reading test)
- key: none (shown by reading)
- probe: a rule whose condition compares two amounts, sums over heirs, and has a not-applicable branch beside a negative amount
- ids: LC-3 LC-4 LC-5 LC-7 LC-8

### SURF-03 · Draws written
- hard: how a draw, its path and a plan over paths are written
- where: B, D
- key: none (shown by writing)
- probe: RAND-01's lifetime, written out
- ids: LC-26 LC-27

### SURF-04 · Stated once, checked before it runs
- hard: each thing stated once, its consequences derived; changes by amendment only; ids in ASCII; a redefined id refused; everything checked before it runs; formula macros for long law formulas; a fresh modeller writes the model from the household's own sentences
- where: all
- key: none (shown by writing)
- probe: a long law formula used in three places, written once
- ids: N-104 N-1309 N-1611 N-1801 N-1903 N-1904

---

## Outside the set

Ids no question or model here can test. Each is named so the checklist can tell "outside" from "forgotten".

| Id | Why it's outside |
|---|---|
| N-317 | speech dates are resolved before modelling; the dossiers state dates |
| N-1223 | tuning tools: a tool, not something a model says |
| N-1303 | weekly rebasing: a workflow over weeks |
| N-1307 | saved analyses: a tool |
| N-1401 | bit-identical paths across threads and machines: an engine test (the set's same-seed relations check one machine) |
| N-1609 | the numbers in Claude's sentences |
| N-1610 | forms that describe themselves: the app |
| N-1803 | the law gate: a process |
| N-1804 | the law gate's rounding probes: a process |
| N-1805 | every author passes the gate: a process |
| N-1809 | what holds by construction: a design document |
| N-1901 | the catalogue Claude reads |
| N-1902 | the schema and canonical format |
| N-1907 | the surface chosen by evidence |
| N-1908 | Claude's tools |
| N-2001 | speed of a conversation turn |
| N-2002 | headroom |
| N-2003 | Monte Carlo speed |
| N-2004 | incremental runs |
| N-2005 | cheap replays |
| N-2007 | the gate's speed |
| N-2110 | the prototype's own goldens |
| MX-08 | deemed disposal: no home-law rule here taxes unsold units (the 79 cases' CB-14) |
| MX-13 | required withdrawals by a divisor table: no household here draws one (LH-14) |
| MX-40 | scores and calibration: no household question scores a forecast (CH-03) |

Two mechanics are carried in part. MX-10's wash-sale side stays with the 79 cases (LH-01); its lot side is in OWN-06. MX-35's refusal side is in RULE-04, and its named solver in LAW-20, whose probe is the set's own; no visible household meets a named solver, so its integration stays with the 79 cases (UL-07, UL-08).

## Counts (brief §2–§3), visible part

| Tag | Count | Edges | Where |
|---|---|---|---|
| L1 | 2 mid-year law changes, 1 with transitional rules | LAW-03 | A, D |
| L2 | 1 retroactive change booked as a refund or arrears | LAW-04 | A |
| L3 | 3 rules that read another party | PTY-04 | A, D (the third is in the holdout) |
| L4 | 1 superseded version still run | LAW-03 | A |
| L5 | 1 election per year | LAW-06 | A |
| L6 | 1 deadline whose miss changes the tax | TIME-05 | D |
| L7 | 1 result pending until a window closes | LAW-10 | D |
| L8 | 1 unclear rule keyed under each reading | NUM-04 | A |
| L9 | 2 published figures that disagree with their law | VAL-04 | A, D |
| L10 | future law: 1 what-if and 1 change as a draw, both on every path | LAW-13, RAND-03 | A, B |
| M1 | 5+ funds and 100+ lots; bonds; a lot moving between owners and between brokers | OWN-06, OWN-07, OWN-09 | A, D |
| M2 | 4 ownership regimes and 3 events that move ownership | OWN-04, OWN-05 | A, B, D |
| M3 | 2 things bought and ended, each with 3+ things tied to it | LIFE-02 | A, B |
| M4 | 3 loans of different kinds: a reset, a holiday, an overpayment, an early payoff | LIFE-03 | A, B, D |
| M5 | 1 statement with 4 kinds of error, and awkward rows | REC-02, REC-03 | B |
| M6 | 2 currencies; one transfer at the law's rate and the bank's | CUR-01 | B |
| M7 | paths in B and D and A's named paths; 3 random families; law on every path; 2+ closed forms | RAND-01, RAND-02, RAND-03, SCEN-04 | A, B, D |
| M8 | owed but not yet paid, inside net worth | OWN-03 | all |
| Q3 | per household: 2+ holes or refusals, 1+ forecast question, 1+ named hole in per-person net worth | ASK-02, ASK-03 | all |
| R1 | per household 3+ house rules: one on dates, one on state, one on a forecast | RULE-01, RULE-05, RULE-09 | all |
| R2 | two rules wanting the same money, with a priority and without | RULE-03 | B, A |
| R3 | a loop (a refusal) | RULE-04 | placed by a family's step |
| R4 | a rule changed "as if since January" | RULE-06 | placed by a family's step |
| R5 | a standing order whose records show it failed | RULE-07 | A, and one more |

The other per-question counts (fused questions, each twist moving 3+ keys, every home-law rule read) are checked by `tools/check.py` on the questions and keys, not by edges.

## Twists, visible part

| Household | Twist | Edges it moves |
|---|---|---|
| A | a rate cut from July 2022, applied by the return to the whole year | LAW-03, LAW-04, TIME-03, VAL-03 |
| B | wrong bank rows: missing, duplicate, reversal, half-month export | REC-02, REC-01, REC-04 |
| D | a death (Zbigniew, 2024) | OWN-05, PTY-03, TIME-04, LIFE-03, LAW-12 |
| A, D | a published figure that disagrees with its law | VAL-04 |
