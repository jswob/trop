# Read verdicts, after the fix · household C · session 5r, step 2.6 (not blind)

**Not blind.** This marking comes after the triage (`triage.md`). The first score stays as recorded. Map: `maps/C.after.md`, which is `maps/C.lead.md` with C-07, C-08, C-11 and C-12 redone on the after-fix lines. Bridge: `tmp/session5/tools/answers5.py`, where a printed `none` becomes null and quoted text loses its quotes. Runs: `runs-after/C/*.out`, all 12 `ok` in its `hrun.log`. Marks: `answers/C.after.compare.txt`, compare.py with keys `tmp/cases-holdout/keys/C/` as they stand, no `--small`. The judge made no runs of its own.

| question | compare.py | read lines | final |
|---|---|---|---|
| C-01 | read | 1 agreed | **pass** |
| C-02 | pass | none | **pass** |
| C-03 | pass | none | **pass** |
| C-04 | pass | none | **pass** |
| C-05 | pass | none | **pass** |
| C-06 | pass | none | **pass** |
| C-07 | fail (2 ✗) | none | **fail** against the key as it stands (see below for the erratum proposal) |
| C-08 | read | 9 agreed | **pass** |
| C-09 | pass | none | **pass** |
| C-10 | read | 3 agreed | **pass** |
| C-11 | read | 4 agreed | **pass** |
| C-12 | read | 1 agreed | **pass** |

Totals: pass 11, fail 1, not run 0. Read lines: 18 agreed, 0 not agreed.

## C-07: the one fail, and the erratum proposal

✗ lines:
- `C-07/owed on 2025-12-31: missing 'payable'`
- `C-07/owed on 2025-12-31: labels the key doesn't have: ['cases', 'kind']`

The key's slot holds one value, `{"payable": 672.37}` (step 14). The answer is the writer's stated by-case, `c07_owed_total` place 1 for each reading of `flr_count`:
- at_lazur_buy: 672.37 (c07.out:44)
- at_cbl_mid: 3 194.65 (c07.out:47)
- at_lazur_sell: 3 259.99 (c07.out:50)

Every other slot of C-07 compares equal: the months (c07.out:1–21) and the settle-ups (c07.out:23, :25).

**Under the erratum proposal** (triage.md: "owed on 2025-12-31" becomes a by-case over C-08's bike readings), C-07 passes if the three cases written by `keys_c.py` equal the three printed figures above, case for case. Nothing else in C-07 differs. No file I may read holds the proposal's figures, so this stays conditional. The printed parts that would make up those cases do agree with keys I can read:
- C-HR3 alone, `c07_owed` 672.37 (c07.out:27), equals C-07 step 14.
- The bike's C-HR4 debts, `c07_hr4` 2 522.28 at CBL mid (c07.out:34) and 2 587.62 at Lazur sell (c07.out:37), each due 2025-08-31, equal C-08's key cases.
- Nothing is owed for the bike at Lazur buy (c07.out:31).

Once the proposal's case labels exist, the three case labels would be read lines.

## Read lines

### C-01
- `last VAT return/month`: key "2024-06", answer "2024-06". **Agreed.** Key step 5; c01.out:13, place 1.

### C-08: the bike's case labels (9)
The axis `flr_count` (purchases.tally:7–9) has three options:
- `at_lazur_buy`: Bank Lazur's buy rate of the purchase day.
- `at_cbl_mid`: as GEN-4 converts, the CBL mid rate of the last business day before.
- `at_lazur_sell`: Bank Lazur's sell rate of the purchase day.

Each of the key's cases is paired with the answer case of the same meaning, for "needed yes", "owed back" and "due" alike:
- key "at the CBL's mid rate, as GEN-4 converts" ↔ `flr_count = at_cbl_mid`, ×3. **Agreed.** Key step 1 (2 522.28); c08.out:6 (2 522.28 DUK; true; jan 2 522.28; 2025-08-31).
- key "at Bank Lazur's buy rate" ↔ `flr_count = at_lazur_buy`, ×3. **Agreed.** Key step 2 (2 459.58); c08.out:3 (2 459.58 DUK; false; none 0.00; due none, now null).
- key "at Bank Lazur's sell rate" ↔ `flr_count = at_lazur_sell`, ×3. **Agreed.** Key step 3 (2 587.62); c08.out:9 (2 587.62 DUK; true; jan 2 587.62; 2025-08-31).

The sofa now has one line (c08.out:12: had yes false, marta 3 400.00, 2026-02-28), which matches key step 5. The washing machine's and the shed's `due` print `none` (c08.out:1, :16) and now compare equal to the key's null.

### C-10: holes, both ways (3)
These are as in the first score. The dossier: house-rules.md:75 (C-HR9 .4) and story.md:27, :77 (neither the house nor the car has ever been valued).
- `together/missing`: the answer names the car and the house, which are the key's two facts, and nothing the dossier settles. **Agreed.** Key steps 9–11; c10.out:1, place 1.
- `Marta/missing`: the house only, at 0.5 ×. **Agreed.** The car is Jan's own (key step 11). Key step 10; c10.out:1, place 2.
- `Jan/missing`: the car at 1 × and the house at 0.5 ×. **Agreed.** Key step 11; c10.out:1, place 3.

### C-11: gift tax (4)
The axis `release_money` (loan.tally:7–8). The hole `jan_reported` (loan.tally:18, :37).
- Case label: key "the release is not a gift of money, reported by 2027-03-17" ↔ `release_money = not_money, jan_reported = true`. **Agreed.** The meanings match: not_money is the reading that IGT-4 .4 doesn't reach, and the report is pinned true. The values (report needed true, by 2027-03-17, return by none, tax 0.00) match key step 7. c11.out:17.
- Case label: key "the release is not a gift of money, not reported by 2027-03-17" ↔ `release_money = not_money, jan_reported = false`. **Agreed.** The meanings match. The values (true, 2027-03-17, 2026-10-19, 142.00) match key steps 6–7. c11.out:25.
- Case label: key "the release is a gift of money, never credited" ↔ `release_money = money`. **Agreed.** loan.tally:7 says a released debt of money is a gift of money, credited to no account of Jan's, so it is not exempt (IGT-4 .4). The values (report needed false, by none, return by 2026-10-19, tax 142.00) match key step 7 ("no report lifts that"). c11.out:7. The money lines pinned true and false print the same (c11.out:13, :21).
- `settled by`: key ["whether IGT-4 .4 reaches a released debt", "whether Jan reports the release to the RO by 2027-03-17"]. The answer's slot holds the not_money line's hole: "does Jan report Marta's release of the loan to the RO within 6 months of 2026-09-17 (IGT-4 .2)? no paper says" (jan_reported), c11.out:10, place 4. **Agreed**, by meaning:
  - Both facts are printed. The first is the axis `release_money` in the reading lines (c11.out:8, :11; loan.tally:4). No slot can take an axis as a value (I-7).
  - The second fact is the hole. Its 6 months run to the 2027-03-17 printed beside it (c11.out:10, place 3), which is key step 7's date.
  - Nothing is called missing that the dossier settles: the story says nothing about a report (loan.tally:37).

### C-12
- `October 2026 extra/missing`: key "what picks one plan among those C-HR8 .2–.3 allow: no rule says which, and none names a way to solve the loop". Answer "in which order does our plan decide October 2026's 800.00 and the later months' (C-HR8 .2 – .3 don't say)? the two orders decide October differently" (oct_extra_paid). **Agreed.**
  - Both name the same open fact: C-HR8 .2–.3 allow a plan that pays October and one that skips it, and no rule picks between them.
  - The run prints the two outcomes beside the hole: true and false (c12.out:5, places 2–3), which matches key step 4's plan 1 and plan 2.
  - The answer is a refusal with no value, which is the key's kind.
  - It calls nothing missing that the dossier settles.
