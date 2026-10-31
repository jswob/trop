# I-10 · A car shared by two households: shares that change, a running ledger, and an accident paid for in two years

**Story.** Sisters Pola and Rena keep separate households. On 1 January 2027 they buy a car together, Pola 60 %, Rena 40 %, and keep a shared ledger of what each household owes the other. Pola is the policy holder.

- Fuel: every month of 2027 Pola's household buys 300.00 of fuel and Rena's 200.00. The log shows Pola's household driving 60 % of each month's kilometres from January to June and 40 % from July to December. In 2028 each household buys 250.00 a month and drives half.
- 1 Jan 2027: Pola pays the 2027 premium, 2 400.00.
- 12 Apr 2027: Rena pays for a service, 1 500.00.
- 1 Jul 2027: Rena buys 10 % of the car from Pola for 5 400.00. From then on they own it half and half.
- 18 Nov 2027: an accident; Rena was driving and caused it. Pola pays the repair, 7 000.00, on 25 Nov 2027.
- 1 Jan 2028: Rena pays the 2028 premium.
- 14 Feb 2028: the insurer pays Pola 6 000.00 for the repair (7 000.00 less the policy excess of 1 000.00).

**Rules.** (The sisters' agreement, and the insurer's terms where named)
1. Every cost of the car except fuel is shared by the ownership shares on the day it is paid.
2. Fuel is shared by the kilometres each household drove in the month.
3. Of a repair after an accident, the policy excess is charged to the household whose driver caused it; the rest is a cost under rule 1. A payment from the insurer is shared by the ownership shares on the day it is received.
4. The extra premium a claim causes is charged to the household whose driver caused it; the rest of the premium is a cost under rule 1.
5. The price of a share is paid through the ledger.
6. Whatever a household pays, or receives, for the car goes into the ledger, and the other household then owes or is owed its part.
7. If, at the end of a month, one household owes the other more than 800.00, it pays the whole balance on the 10th of the next month. At the end of December the whole balance is paid on 10 January, whatever its size.
8. (Insurer) The premium for 2028 is 3 000.00 less a discount: 30 % if no accident in 2027 led to a claim, 10 % if one did.

**Question.** Every payment between the households from January 2027 to 10 March 2028: its date, who pays, and the amount. Then the 2028 premium, the part charged to Rena under rule 4, and each household's share of the car's 2027 costs, fuel included and the price of the share left out.

**What makes it hard.** Two households hold one thing in shares that change on a date, and each cost is split by the shares on its own day, or by a log, or charged to one side by fault. A running balance between the households is carried month to month and a payment is triggered by reading it against a threshold at month end, with a different rule at year end. One accident reaches three things in two years: a repair split by fault and shares, a receipt from a third party months later, and a premium that depends on whether a claim was paid. A purchase of a share is itself a ledger entry.
