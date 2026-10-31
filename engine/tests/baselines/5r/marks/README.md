# The marks behind the 5r baselines (cases 1.2.0)

The 18 outputs in `..` are refcore's runs of the unsealed holdout's checks (`defs/holdout.tsv`), made 2026-10-30 by
`run.py --interp refcore`. Each is byte-equal to the exam's after-fix run of the same recipe
(`tmp/cases-holdout/tally-2.0/runs-after/`), which session 5r's judges marked.

- `C.map.md`, `E2.map.md`: the slot maps (C: the after-fix map with C-07's slot reshaped for the corrected key, ERR-39;
  E2: the after-fix map).
- `C.answers.md`, `E2.answers.md`: `engine/tests/tools/answers.py` over these outputs.
- `C.compare.txt`, `E2.compare.txt`: `compare.py --keys-root engine/tests/cases/keys` over them: C 6 pass, 6 read;
  E2 3 read; no fail.
- `C-reads.md`, `E2-reads.md`: the judges' verdicts on the read lines (every read line agreed). C-07 is new under
  ERR-39: its three cases match the key's figures (672.37, 3194.65, 3259.99) with no ✗; the printed labels
  `at_cbl_mid`, `at_lazur_buy`, `at_lazur_sell` read as the key's three readings (agreed at the 5r close).

So all 15 questions pass. A check here compares the whole output with its baseline; a change is a `differ` to judge
against these marks.
