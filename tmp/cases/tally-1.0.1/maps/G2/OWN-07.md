# OWN-07 · map

Text: `probes/own/OWN-07.tally`. Key kind: value.

step 1 · lots taken, per account: FIFO within the account (rule 1) = L1 10 (1000), L3 5 (450) · lines 30, 34, 38, 40–46, 57, 62–76 · L §5.11 (fill), §10 (lots matched by fill) · same: arcs are the lots of account X (L:850); sorted by lot_bought (L:851); takes min(10, 15) from L1 and min(10, 5) from L3 (L:853); cost 10 × 100 + 5 × 90 = 1 450, gain 1 950 − 1 450 = 500
step 2 · lots taken, across accounts: contrast = L1 10 (1000), L2 5 (600) · lines 32, 36, 48–55 · L §5.11 · same: arc true; L1 then L2; cost 1 600, gain 350
ask returns: value · 1450, 500; 1600, 350 DUK
checks: parsecheck ok · loadcheck ok (after renaming the index account, which collided with the placekind option account, L:257)
