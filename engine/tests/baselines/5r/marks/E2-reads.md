# Read verdicts · household E2 · session 5r, after-fix marking (step 2.6) · **not blind**

The first score stays as recorded (`maps/E2-reads.md`). This marking uses the texts fixed after triage (`triage.md`: E2-02, cause **text**; `TEXTS-after.sha256`), the runs `runs-after/E2/` (only `e2c.out` differs from `runs/E2/`; the other five outputs are byte-identical, checked with cmp), the map `maps/E2.after.md` (`maps/E2.md` with only E2-02's "Jan from Marta" slots changed), the bridge `tmp/session5/tools/answers5.py` (I-4, I-5), and `answers/E2.after.compare.txt`. Cited lines are `runs-after/E2/<run>.out:<line>`. No run of my own was made.

| question | compare.py | read lines (agreed / not agreed) | final mark (after fix, not blind) |
|---|---|---|---|
| E2-01 | read | 5 / 0 | **pass** |
| E2-02 | read (no ✗) | 20 / 0 | **pass** |
| E2-03 | read | 1 / 0 | **pass** |

## E2-01 · read (unchanged slots)

compare.txt:3–7 are the first score's five read lines. The four account numbers now come through without quotes (I-5), each equal to the key's (key steps 4–7; e2_01a.out:21, :23, :25, :27): **agreed** ×4. Jakub's arrived/missing (e2_01a.out:33, key step 7): **agreed**, as in `E2-reads.md`.

## E2-02 · read

The four heirs from Zbigniew (compare.txt:9–24, key steps 5, 6, 7, 9): the same 16 lines from the same unchanged lines (e2_02a.out, e2_02b.out). **Agreed** ×16, for the reasons given in `E2-reads.md` (pinned report on 2025-05-05 = the key's "reported by 2025-05-05"; world e2_not_reported = "not reported"; one hole, the flat's IGT-2 .2 value = the key's missing fact; "settled by" holes = the report day and the flat's value).

Jan from Marta (key step 10, keys/C/C-11; the tax is place 9 of each line):

- case label "the release is not a gift of money, reported by 2027-03-17" ↔ "reading: release_money = not_money; pinned: jan_reported = true" (compare.txt:25): **agreed**. e2c.out:15–17: under not_money with the report pinned made, report needed true, deadline 2027-03-17 (the key's date), exempt true, tax 0.00 = the key's 0.
- case label "the release is not a gift of money, not reported by 2027-03-17" ↔ "reading: release_money = not_money; pinned: jan_reported = false" (compare.txt:26): **agreed**. e2c.out:23–25: deadline 2027-03-17, exempt false, tax 142.00 = the key's 142.
- case label "the release is a gift of money, never credited" ↔ "reading: release_money = money" (compare.txt:27): **agreed**. e2c.out:5–6: the money reading, which C states as a released debt credited to no account of Jan's, so not exempt (C/full/loan.tally:7); report not needed, deadline none, exempt false, tax 142.00 = the key's 142. It is the same under both report pins (e2c.out:11, :19). compare.py paired this case rightly: the map lists the cases in the key's order (I-6).
- "settled by" (compare.txt:28): key ["whether IGT-4 .4 reaches a released debt", "whether Jan reports the release to the RO by 2027-03-17"]; the answer's slot is the tax as the papers stand under not_money (e2c.out:8, place 9), a refusal with one hole: whether Jan reports the release within 6 months of 2026-09-17 (`jan_reported`; place 7 of the same line gives that deadline as 2027-03-17). **Agreed**, by the answer as a whole: the slot names the second fact, and the first fact, the axis `release_money` (C's question whether IGT-4 .4 reaches a released debt, loan.tally:4), is named in all three case labels. A slot can't take it as a value (instrument I-7, `triage.md`). I note for the lead that this verdict counts on the case labels for the first fact. The answer calls nothing missing that the dossier settles: the gift, the release, the amounts and the deadline are printed as values (e2c.out:8, places 1–7).

No ✗: every E2-02 value slot matches its key value. The rest of E2-02 is unchanged from the first score.

## E2-03 · read (unchanged)

`E2-03/share` (compare.txt:30): key "3/16", answer 0.1875 (e2_02a.out:57, place 1): **agreed**, as in `E2-reads.md`.
