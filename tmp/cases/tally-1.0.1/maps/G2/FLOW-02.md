# FLOW-02 · map

Text: `probes/flow/FLOW-02.tally`. Key kind: relation.

step 1 · owners' leg: 42000.00 − 18350.00 − 600.00 = 23050.00 · lines 27–29, 43–47, 59–61 · L §3.2, §5.13 · same: owners_amount is money − money; the leg pays it to the owners' account
relation · lender 18350.00, dealer fee 600.00, owners 23050.00, legs sum 42000.00 · lines 31–51 · L §5.13.3 (flows of a pay), §5.16 (check), §5.9 (debt) · same: each leg is a pay on sale_day; flows of each pay over September prints its amount; q_legs_sum's check prints ok; the payoff takes the debt from −18 350.00 to 0, never above (L:713)
ask returns: relation · 18350.00, 600.00, 23050.00; ok
checks: parsecheck ok · loadcheck ok
note: the legs are three pays with one date; nothing groups them or makes them settle together (O:FLOW-02-1)
