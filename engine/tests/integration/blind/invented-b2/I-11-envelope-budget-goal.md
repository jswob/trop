# I-11 · An envelope budget: overspends carried, a back-up lender, and the day a goal is reached

**Story.** Natalia and Oskar keep their household money in a joint account J, which holds 2 500.00 on 1 January 2027. Natalia is paid 7 200.00 on the 28th of every month and Oskar 5 400.00 on the 10th, both into J. A savings account H holds 0.00 on 1 January 2027; their goal is 30 000.00 in H. Their envelopes, and what each pays for from J (payments on one day are made in the order listed):

| Envelope | Monthly amount | Spending from J |
|---|---|---|
| Home | 3 100.00 | rent, 3 100.00 on the 3rd |
| Food | 2 000.00 | a quarter of the month's food bill on each of the 5th, 12th, 19th and 26th |
| Transport | 600.00 | 640.00 on the 15th; car insurance, 1 260.00 on 15 March 2027 |
| Fun-N (Natalia's) | 400.00 | 450.00 on the 20th |
| Fun-O (Oskar's) | 400.00 | 300.00 on the 20th |

The monthly food bill is not known. Three paths are asked: 1 800.00, 2 100.00 and 2 400.00 a month, the same in every month.

**Rules.** (The household's budget)
1. Unassigned money is J's balance minus all envelope balances minus any debt to Natalia under rule 6. Income enters unassigned.
2. A payment is charged to its envelope even if this takes the envelope below zero. An envelope's balance, positive or negative, carries into the next month.
3. On the 1st of each month, before anything else that day, each envelope is filled by its monthly amount from unassigned, in the order Home, Food, Transport, Fun-N, Fun-O. If unassigned runs short, the envelope being filled takes what is left and the later ones get nothing that month. A short fill is never made up.
4. From 1 April 2027 Transport's monthly amount is 750.00, as if since January: on 1 April, after the fills, Transport also receives 450.00 from unassigned, or as much of it as unassigned holds.
5. After its fill, a personal envelope (Fun-N, Fun-O) holds at most 600.00; anything above goes back to unassigned.
6. If a payment from J would take J's balance below zero, Natalia's own account makes the whole payment instead, and the household owes her that amount. The debt is repaid from J on the next payday, out of that day's income before the rest enters unassigned, as far as the income allows; what is left of the debt waits for the payday after.
7. On the 27th of each month, after everything else that day, unassigned money above 1 000.00 moves from J to H, but never so much that H holds more than 30 000.00.
8. Once H holds 30 000.00, rule 7 stops for good, and from the next month's fill each personal envelope's monthly amount is 500.00.

**Question.** For each food path: H's balance after each move under rule 7; the date the goal is reached; every payment Natalia makes under rule 6, with its date, amount and the date or dates it is repaid; and, at the end of 31 December 2027, every envelope's balance, unassigned money and J's balance.

**What makes it hard.** The envelopes are parts of one real balance, and unassigned money is a figure derived from the others, never stored. Envelope balances, including overspends, carry from month to month, and the fill reads a running figure in a set order, so a shortfall falls on whichever envelope comes last. An amendment "as if since January" is settled by one catch-up on a date. A back-up rule reads the real balance before each single payment and creates a debt that ranks ahead of later income. A goal reached once switches one rule off and changes another for good, and the question is the first day a condition holds, on each of three paths of an unknown figure.
