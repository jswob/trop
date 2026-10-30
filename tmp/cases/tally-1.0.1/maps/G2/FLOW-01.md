# FLOW-01 · map

Text: `probes/flow/FLOW-01.tally`. Key kind: relation.

step 1 · balancing flow: 250.00 − 400.00 − 300.00 (rule 1) = −450.00 · lines 23–31, 33, 42–47 · L §5.18 (stated anchors), §5.13.3 (flows), §5.9 · same: the counts are stated rows; the wallet has no opening, so its balance is known from its first anchor (L:1146): 400.00 at the end of 1 January, 250.00 at the end of 31 January (L:1141); flows over 2 to 31 January counts the cash machine's 300.00 and no anchor (L:1142); balancing = 250 − 400 − 300 = −450.00, the sign giving the direction (out of the wallet). The spec's own balance line, stated − booked, gives the same −450.00 (q_ledger, E:253, E:285)
relation · start + recorded flows + balancing flow = end · line 35 · L §5.16 (check) · same: q_holds prints ok; it holds by the def's arithmetic, the card's rule 2
ask returns: relation · −450.00 DUK; ok
checks: parsecheck ok · loadcheck ok
note: Tally books no balancing movement (an anchor isn't a movement, L:1142), so flows(wallet, 2025-01) is +300.00, not −150.00 (O:FLOW-01-1)
