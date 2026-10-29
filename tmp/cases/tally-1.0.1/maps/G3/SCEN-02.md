# SCEN-02 · step map (text: probes/scen/SCEN-02/, files SCEN-02.tally and kept.tally)

step 1 · dropped: what needs the sale = [sale, insurance refund] · SCEN-02.tally lines 50–62, 68, 70–76, 94–98; kept.tally lines 1–8 · L §6.4 (a scenario row wins inside it; an instance dated t ≥ from sees it), L §4.3 (an unkeyed instance's law date is its `on`), L §5.13 (`when`) · same: inside `kept`, `sold` is false on 2024-09-15 and 2024-09-20, so both instances don't exist
step 2 · added: what runs on if kept = credit 2024-10..12, parking 2024-11..12 · SCEN-02.tally lines 36–48, 66, 78–92 · L §4.3 (a month-keyed instance's law date is the month's first day, after from), L §5.13 · same, with `credit_from` = 2024-10 as the card's inputs give it; the October parking is kept in both (`m <= park_last`)
step 3 · cash difference: added − dropped = −24900.82 · SCEN-02.tally line 64 · L §6.4 (Asking), L §5.13.3 (`flows`) · same: −1650.00 in the branch against +23250.82 in the base
ask returns: value · the per-pay and per-instance flows in both worlds, and −24900.82
checks: parsecheck ok · loadcheck ok
