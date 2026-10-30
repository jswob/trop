# The law of Rosvald: what a worker from Lenia meets

Rosvald is Lenia's neighbour; its money is the floren (FLR, 100 cents). This file holds the parts of its law that household C meets, in the home law's format (`tmp/cases/law/acts.md` § How to read it). Its figures are in `law/rosvald-gazette.csv`, as Rosvald's gazette published them; a figure's id is named in the rule. Bodies: the Rosvald National Bank (RNB), the Rosvald Tax Board (RTB), the Rosvald Social Insurance (RSI office) and the Rosvald Family Fund (RFF).

## RGP · General Provisions

### RGP-1 · Business days
**v1** · in force 2015-01-01 – until replaced · published 2014-06-02 · governs: every deadline and pay day under Rosvald's law
1. Business days are Monday to Friday, except the public holidays: 1 January, 1 May, 14 June, 3 October, 24, 25, 26 and 31 December.
2. A day Rosvald's law or a contract under it fixes for a payment, where it is not a business day, moves to the business day before it, unless the rule says otherwise.

### RGP-2 · Rounding
**v1** · in force 2015-01-01 – until replaced · published 2014-06-02 · governs: every amount under Rosvald's law
1. "To the cent" means to the nearest cent, and an amount exactly half a cent from two cents goes to the even one.
2. Only the amounts a rule names are rounded, at the step that produces them.

### RGP-3 · The dukat reference rate
**v1** · in force 2015-01-01 – until replaced · published 2014-06-02 · governs: reference rates for months from January 2015
1. The RNB publishes, on the last business day of each month, its reference rate of the dukat for the next month, in florens for 100 dukats (`RNB-DUK-yyyymm`).

## RWT · Wage Tax Act

### RWT-1 · Who pays it, and the tax year
**v1** · in force 2015-01-01 – until replaced · published 2014-09-01 · governs: pay paid from 2015-01-01
1. Pay for work done in Rosvald is taxed in Rosvald, whoever the worker is and wherever they live.
2. The tax year runs from 1 April to 31 March. Pay belongs to the tax year in which it is paid, whatever month it is for.

### RWT-2 · Withholding
**v1** · in force 2015-01-01 – until replaced · published 2014-09-01 · governs: pay paid from 2015-01-01
1. The employer withholds the wage tax and the worker's social contribution (RSI-2) from each payment of pay, and gives the worker a payslip stating the gross, each amount withheld and the net.
2. The net is the gross less the contribution and the tax.

### RWT-3 · The tax on a payment
**v1** · in force 2015-01-01 – until replaced · published 2014-09-01 · governs: pay paid from 2015-01-01
1. Each payment is taxed as one month's pay, by the figures of the tax year in which it is paid.
2. The taxable pay is the gross less the contribution withheld from it (RSI-2) less the class's monthly allowance (RWT-4), rounded down to a whole 10 FLR, and never below 0.
3. The tax: 10 % of the taxable pay up to `RWT-EDGE1-<year>`; 24 % of the part above it up to `RWT-EDGE2-<year>`; 38 % of the part above that; rounded down to a whole floren.

### RWT-4 · Classes
**v1** · in force 2015-01-01 – until replaced · published 2014-09-01 · governs: pay paid from 2015-01-01
1. Class 1 is every worker's class. Class 2 is the class of a worker who is married and lives with a child under 16, for a tax year for which the worker has given the employer a class notice.
2. A class notice holds for one tax year. It applies to pay paid after the day it reaches the employer; pay paid on that day or before is taxed in class 1.
3. The monthly allowances: class 1 `RWT-ALLOW1-<year>`, class 2 `RWT-ALLOW2-<year>`.

### RWT-5 · Presence
**v1** · in force 2015-01-01 – until replaced · published 2014-09-01 · governs: tax years from 2015/16
1. A day of presence is a day on which the worker is in Rosvald at any time, whether working or not.

### RWT-6 · The yearly assessment
**v1** · in force 2015-01-01 – until replaced · published 2014-09-01 · governs: tax years from 2015/16
1. A worker with at least 183 days of presence in a tax year may claim a yearly assessment of that year's pay, within two years of the year's end. A worker with fewer days may not.
2. The yearly taxable pay is the year's gross less the year's contributions less 12 times the monthly allowance of the worker's class on the year's last day, rounded down to a whole 10 FLR, never below 0.
3. The yearly tax is RWT-3 .3 with each edge taken 12 times, rounded down to a whole floren.
4. The RTB decides the claim. Where the tax withheld in the year is more than the yearly tax, the difference is refunded within 14 days of the decision; where it is less, the difference is payable within 30 days of it.

## RSI · Social Insurance Act

### RSI-1 · Who is insured
**v1** · in force 2015-01-01 – until replaced · published 2014-09-01 · governs: work from 2015-01-01
1. A worker employed in Rosvald is insured in Rosvald for the months of that employment.

### RSI-2 · The worker's contribution
**v1** · in force 2015-01-01 – until replaced · published 2014-09-01 · governs: pay paid from 2015-01-01
1. The worker's contribution is 13.50 % of each payment's gross, up to the monthly ceiling `RSI-CEIL-<year>` of the calendar year in which it is paid, to the cent.

## RFF · Family Fund Act

### RFF-1 · The child benefit
**v1** · in force 2015-01-01 – until replaced · published 2014-09-01 · governs: months from January 2015
1. A person insured in Rosvald (RSI-1) is entitled, for each month of insurance, to the child benefit for each child under 18 who lives with them: `RFF-CHILD-<year>` a month.
2. Where an agreement with another state gives that state's benefit first, the RFF pays only what the agreement says.

### RFF-2 · Claims and payment
**v1** · in force 2015-01-01 – until replaced · published 2014-09-01 · governs: claims filed from 2015-01-01
1. The benefit is claimed from the RFF. A claim filed within 6 months of a month gives the benefit for that month.
2. The RFF pays each calendar quarter's benefit by the last business day of the month after the quarter; on a first claim, the months of the quarters already ended are paid within 30 days of its decision.
3. The person reports to the RFF, within one month, any change that alters what it pays. The RFF corrects what it paid by a further payment, or a recovery, within 60 days of the report.
