# PTY-03 · map

Text: `probes/pty/PTY-03.tally`. Key kind: value.

step 1 · shares: spouse ≥ 1/4, children equal, Adam represented by Kuba (rules 1–2) = Halina 1/4, Ania 3/16, Piotr 3/16, Marta 3/16, Kuba 3/16 · lines 20–35, 44–59 · L §3.5 (relations, bounded aggregates), §5.10 (rules, excepts, count, any, first), §7.1 · same: branch_count counts the decedent's children, Adam's branch included (4, L:779); spouse_share = max(0.25, 1/5) = 1/4 (L:788); branch_share = (1 − 1/4)/4 = 3/16; share_of gives the spouse rule to Halina, child_share to the surviving children, represent to Kuba with 3/16 / count(Adam's surviving children = 1), and `other` = 0 to Zbigniew and Adam (L:741–746, one winner each). The values are exact rationals and print 0.25 and 0.1875 (L:1074), the key's 1/4 and 3/16; q_shares_sum prints ok
ask returns: value · five exact shares
checks: parsecheck ok · loadcheck ok
