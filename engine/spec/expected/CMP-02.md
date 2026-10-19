# CMP-02 expected: forty members of one chit, the auction month and the next

Model: `models/CMP-02.tally`. Case: `tmp/verification/cases/CMP-02.md`.

The model is MP-14, which models the illustration, with the composed questions 1 to 4 added. Every published figure is from source 0, the ICAI opinion's "Illustration of chit scheme" (pp. 2–3), as the case's "## Published answer" and its front-matter `published_answers` give them. These must stay the same. The composed questions have no published answer, so their figures are derived with one-liners (below).

**Readings.**
- The winner is the row `winner = sub_b` ("let's assume B becomes the winner"). `chit_act` reaches only the two cap checks, which are true under every option, so every line prints `*` for it.
- `pays_late` and `prized_before` are false for every ticket but D. Both are `assumed` keyless rows.
- Payments in the auction month (question 1):
  - every one of the 39 members other than the foreman pays Rs. 2500, B included ("Remaining 39 members contribute their instalment on monthly basis");
  - the foreman's ticket pays one instalment from its upfront Rs. 2500 × 40, that is the upfront divided by the 40 months of the chit (rule `paid_in.foreman_part`);
  - case "Missing facts" leaves both of these open (its third and fourth bullets). These are the readings under which the chit amount of Rs. 1,00,000, "40 subscribers @ Rs. 2500", is the month's total paid in;
  - the case's "Missing facts", first bullet, says this is not the first auction (D is already prized). The auction month's instalment is read as the full Rs. 2500, as the illustration's Chitty sala is.
- "Paid out or set aside" (question 1) uses the prize before GST and charges (Rs. 70,000). The GST and the charges come out of B's prize, so they don't change the total.
- Net position (question 2) is what the ticket received, plus the share credited to its next instalment, minus what it paid in. Every ticket's Rs. 625 is credited at the end of the auction month, D's included. D forfeits it only by paying the next instalment late, which is question 3.
- Known dates (LANGUAGE §4.5), as in `expected/MP-14.md`: `act1982` 1982-08-20 and `act2001` 2001-05-09 (Gazette publication, from the Puducherry copy's footnotes, pages 1 and 31), `act2019` 2019-12-05 (Gazette), `icai_opinion` 2023-11-09 (the opinion's own date, holding its rules), `icai_illustration` and `mp14_assumed` 2026-10-18 (the illustration's stipulations and the assumptions standing for its facts, from `on`; the model has no `start`). All are known by the cut, so no figure changes.
- Money prints to the paisa: Rs. 68,125 prints `68125.00`.

## The illustration's figures

| Ask | Case locator | Figure as stated | Reading |
|---|---|---|---|
| `q_chit_amount` | Published answer, row "Chitty sala" (p. 3, bullet 2): "Chitty sala will be Rs. 1,00,000 i.e., 40 subscribers @ Rs. 2500/-." | Rs. 1,00,000 | 40 × 2500; prints 100000.00 |
| `q_members_check` | Inputs, row "Members": "Assume a chit fund scheme with 40 members" | ok | the index has 40 tickets |
| `q_foreman_upfront` | row "Foreman's upfront contribution": "(i.e., Rs. 2500 * 40 months)" (no total printed) | Rs. 2500 * 40 months (case check `print(2500*40)` → `100000`) | duration 40 |
| `q_prize` | row "B's prize": "he will be entitled to Rs. 70,000 (less applicable GST on foreman commission and other incidental charges)" | Rs. 70,000 | before GST and charges |
| `q_discount` | row "Amount foregone": "Rs. 30,000 (i.e. 30% foregone by “B”)" | Rs. 30,000 | bid[sub_b] = 30% |
| `q_commission` | row "Foreman commission": "Rs. 5,000 (5%) will be taken by the Company as foreman commission" | Rs. 5,000 | 5% of the chit amount |
| `q_distributable` | row "Amount shared": "remaining Rs. 25,000 will get divided equally among all 40 subscribers" | Rs. 25,000 | — |
| `q_dividend` | row "Dividend per subscriber": "Rs. 625 (i.e. Rs. 25,000/40 subscribers)" | Rs. 625 | exact |
| `q_next_instalment` | row "Next month instalment": "next month instalment for all subscribers would be Rs. 1,875" | Rs. 1,875 | for the 39 tickets that keep the share. `sub_d` prints 2500.00 (derived from the next row, not printed) |
| `q_forfeited_d` | row "D's forfeiture": "his dividend (veethapalisa) of Rs. 625 will be forfeited" | Rs. 625 | rule `dividend_kept.forfeit` |
| `q_prize_net` | row "B's prize", the parenthesis; Missing facts, fifth bullet | not stated | a hole: the questions of `gst` and `charges` |
| `q_due_d` | Inputs, row "D pays late": "He will also have to pay interest for delayed remittance at applicable rate." | not stated | a hole: the interest of `late_interest` |
| `q_winner_tied` | Inputs, rows "The bids" and "The winner" | ok | B's bid equals the highest, 30% |
| `q_bid_ok` | Inputs, rows "Bid cap under the Act" (40%) and "Bid cap set by the Company" (30%) | true (not a figure) | `chit_act = *` |
| `q_commission_ok` | Law references, s. 21(1)(b) (five per cent) and the 2019 amendment s. 8(i) (seven per cent) | true (not a figure) | `chit_act = *` |

The case's checks (Transcription notes): `print(40*2500)` → `100000`; `print(100000*30/100, 100000-30000)` → `30000.0 70000`; `print(100000*5/100, 30000-5000)` → `5000.0 25000`; `print(25000/40, 2500-625)` → `625.0 1875`.

## The composed questions (derived; no published answer)

| Ask | Keys | Case locator | Figure (derived) | Reading |
|---|---|---|---|---|
| `q_c1_paid_in` | — | Story, "What is asked", question 1: "the total paid in by all forty tickets (the foreman's included, from its upfront payment)" | 100000.00 | 39 × 2500 + 100000 / 40 |
| `q_c1_paid_out` | — | question 1: "the prize to the winner, the foreman's commission, the amount shared" | 70000.00, 5000.00, 25000.00; total 100000.00 | prize before GST and charges |
| `q_c1_check` | — | question 1: "whether the amounts paid out or set aside add up to the total paid in" | ok | — |
| `q_c2_position` | sub_b | question 2: "the winner B" | received 70000.00, credited 625.00, paid 2500.00; position 68125.00 | — |
| `q_c2_position` | sub_a (and every other ticket that did not win: sub_c, sub_d, t06 … t40) | question 2: "a member that did not win" | received 0.00, credited 625.00, paid 2500.00; position -1875.00 | D included: the share is credited at the end of the auction month |
| `q_c2_position` | foreman | question 2: "the foreman's own ticket" | received 0.00, credited 625.00, paid 2500.00; position -1875.00 | paid from the upfront |
| `q_commission` | — | question 2: "the foreman's commission as the company's income" | 5000.00 | the illustration's Rs. 5,000 |
| `q_c2_sum` | — | question 2: "the sum over all forty tickets plus the commission" | 0.00 | — |
| `q_c2_winner_net` | — | questions 2 and 4: B's position after "GST and charges on the prize" | not stated | a hole: the questions of `gst` and `charges` |
| `q_next_instalment` | each ticket | question 3: "the instalment due from each ticket after the adjustment" | 1875.00 for every ticket but sub_d; sub_d 2500.00 | the foreman's too, from its upfront |
| `q_c3_total` | — | question 3: "the total due from all forty tickets" | 75625.00 | 39 × 1875 + 2500, with D late as in the illustration |
| `q_due_d` | — | question 3: "the instalment due from a prized subscriber D who pays late and forfeits the share" | instalment 2500.00 (`q_next_instalment` at sub_d); with the interest, not stated | `q_due_d` is a hole: the interest |
| `q_c4_forfeit` | sub_d | question 4: "where a forfeited share goes" | not stated | a hole: the Company holds it as a liability until termination, the Act is silent, the EAC puts the income at legal extinguishment |

**Question 4, stated and open.**
- **Stated by the opinion**, as rows of `icai_illustration` with locators: the instalment, members, duration, bids, winner, commission rate and the Company's cap. The Act's caps are rows of `act1982`, `act2001` and `act2019`. From them come the figures of the first table: 1,00,000, 70,000, 30,000, 5,000, 25,000, 625, 1,875, and D's forfeited 625.
- **Left open**, each a hole that prints its question:
  - the GST and the charges on the prize (`q_prize_net`, `q_c2_winner_net`);
  - the interest on D's late payment (`q_due_d`);
  - where a forfeited share goes (`q_c4_forfeit`).
- **Left open, carried by readings**, which the readings above list:
  - which text of the Act governs (`chit_act`; no figure depends on it);
  - how the foreman's upfront payment counts in the month;
  - whether B pays in the auction month;
  - the auction month's number.
- **Derived here, not stated:** every figure of questions 1 to 3.

**One-liners.**

Question 1: `python3 -c "from fractions import Fraction as F; print(39*2500 + F(2500*40, 40), 70000+5000+25000)"` → `100000 100000`

Question 2 (B, a ticket that did not win, the sum of all forty plus the commission): `python3 -c "print(70000+625-2500, 0+625-2500, 68125 + 39*(-1875) + 5000)"` → `68125 -1875 0`

Question 3 (the adjusted instalment, the total with D late, and the total if nobody were late, which isn't asked): `python3 -c "print(2500-625, 39*1875 + 2500, 40*1875)"` → `1875 75625 75000`
