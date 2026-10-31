# I-6 · A joint mortgage: rate resets, two overpayments with two elections, and a relief split between the borrowers

**Story.** Hania and Igor each own half of the flat they live in. On 15 March 2027 Bank Orla lends them, jointly, 480 000.00 for it, repayable in 300 monthly instalments due on the 15th, the first on 15 April 2027.

REF6 is a reference rate the central bank publishes on the last business day of each month. It was 5.40 % on Fri 26 Feb 2027 and 5.90 % on Tue 31 Aug 2027. Later values are unknown; the bank's outlook puts each one between 4.50 % and 6.50 %.

They overpay 30 000.00 on 1 Dec 2027, electing "lower instalment", and 25 000.00 on 1 Mar 2028, electing "shorter term". Their income tax before any relief: Hania 14 000.00 for 2027 and 15 000.00 for 2028; Igor 600.00 for 2027 and 1 100.00 for 2028.

**Rules.** (Loan contract, and the Home Loan Relief Act where named)
1. Rate periods start on 15 March and 15 September. A period's rate is the REF6 value published last before it starts, plus a margin of 2.10 points.
2. An instalment's interest is the principal outstanding after the previous instalment day, times the rate of the period that held on the day before the instalment is due, divided by 12, rounded half up to whole minor units. Principal repaid is the instalment minus its interest.
3. The instalment is P × r / (1 − (1 + r)^−n), where P is the outstanding principal, r the current rate divided by 12 and n the remaining number of instalments, rounded up to whole minor units. The last instalment is whatever repays the loan.
4. The instalment is recomputed on the first day of each period, after that day's instalment, and applies from the next one.
5. An overpayment earns nothing until the next instalment day, when it is applied after that day's instalment. "Lower instalment" then recomputes the instalment as in rule 3. "Shorter term" keeps the instalment and cuts the number of remaining instalments to the fewest that repay the loan. On a day when an overpayment is applied and a period starts, the overpayment is applied first.
6. Overpayments in a contract year (15 March to 14 March) above 10 % of the original principal bear a fee of 2 % of the excess, paid with the overpayment that crosses the line. The fee is not interest.
7. (Relief Act) A person may credit 20 % of the interest they paid in a calendar year on a loan for the home they live in against their income tax of that year, at most 1 500.00 a year. Joint borrowers split the interest paid by their shares in the home. A credit above the person's tax is carried to the next year only, where it counts within that year's cap.

**Question.** The instalment in force from 15 Apr 2027, 15 Oct 2027, 15 Jan 2028, 15 Apr 2028 and 15 Oct 2028; the number of instalments left after 15 Mar 2028; the overpayment fee; the interest paid in 2027 and in 2028; and for each borrower the relief credited and carried for 2027 and 2028. Everything from 15 Mar 2028 on is asked on two named paths: both 2028 REF6 values at 4.50 %, and both at 6.50 %.

**What makes it hard.** The loan is a state carrying its principal, remaining count and instalment from month to month, rewritten by a scheduled reset that reads a published series and by overpayments whose effect is chosen by an election made with each one. Two events land on one day, in a stated order. The fee reads a running total over a contract year that is not the calendar year. Unknown future values are carried as named paths. The relief is a second regime reaching the same interest, split between two people by ownership, capped per person, limited by each one's own tax, with a carry that lives one year.
