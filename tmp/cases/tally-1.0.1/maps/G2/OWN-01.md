# OWN-01 · map

Text: `probes/own/OWN-01.tally`. Key kind: value.

step 1 · fees: 5.00 × month ends while open ['2025-03'] (rule 2) = 5.00 · lines 23–28, 30, 40, 46 · L §5.13 (when, on), §5.13.3 (flows of a pay) · same: the fee instance at each month's last day exists only where its `when` holds (L:901); opened[savings] = 2025-03-01, so only March's instance exists; flows(fee, 2025-01..2025-03) = 5.00 (L:964)
step 2 · net worth: places open on 2025-02-15: ['current'] (rule 1) = 1000.00 · lines 15–17, 32–34, 39–41 · L §5.9 (opening, kinds), §5.18 (stated anchors), §5.16 · same, through a workaround: current has no opening and its stated row anchors it at 1 000.00 at the end of 15 February (L:1141, L:1146). A place has no lifetime in L §5.9: savings exists from S, and with no opening its balance is a hole (L:721), so a plain sum over the account places is a hole and its line is no "no line". The text adds the opening day as an input, opened[x], and the asks' `where` drops places not yet open (L:611): one line (current), 1 000.00
ask returns: value · (a) 5.00 DUK; (b) 1000.00 DUK and one line, current
checks: parsecheck ok · loadcheck ok
