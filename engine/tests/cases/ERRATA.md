# Errata

## cases 1.0.1 (2026-10-25)

Session 3 found eight cases that looked wrong in themselves, and one oddity in a document (`gaps-1.0.1.md` part 6 and part 1). Four blind readers then answered households A, B, D and E1 from the papers alone, and every line where they differed from a key was settled. The cause was one of these:
- **key wrong**: the script was fixed and the key regenerated;
- **text unclear or incomplete**, or **two readings**: the paper was fixed so one reading stands;
- **reader wrong**: nothing changed;
- **case stands**: nothing changed.

The entries below are the changes. Each has its cause, what it settles, the fix and its files, and the keys it moved (old → new).

Rules of the change:
- Generated files (documents, records, keys, `INDEX.md`) were regenerated, never edited.
- `union_e1.py` reproduces every household byte for byte, and `keys_probes.py` says 88 agree.
- Session 3's record (`gaps-1.0.1.md`, `tally-1.0.1/`) is untouched: it was judged against cases 1.0, and the table for session 4 at the end says which of its rows to reread.

`VERSION` reads `cases 1.0.1`; `README.md`'s title and its freeze paragraph say so; `MANIFEST.sha256` holds the new hashes (no file added or removed).

Moved in all: 15 key files (11 questions, 4 probes). Figures moved in 4 of them: SCEN-02, A-05, A-12 and B-12. The other 11 moved in shape, holes or derivation only.

### ERR-1 · VAL-06: a year with no earnings stated (text incomplete)
- **Cause and what it settles:** `P:VAL-06`. Rule 1 adds "that year's earnings" every year, but the card listed none for 2024, and the code skipped a year with no entry (missing counted as 0).
- **Fix:** `probes/val/VAL-06.md` (the story says "…, 48 000.00 in 2023, and none in 2024."; the YAML gives `"2024": 0.00`); `tools/probefns/val.py` (`VAL_06` adds every year; a missing year is an error).
- **Moved:** `keys/probes/VAL-06.md`, derivation only (a 2024 line adding 0). The key stays 146 515.20.

### ERR-2 · SCEN-02: the kept credit's September instalment (key wrong)
- **Cause and what it settles:** `P:SCEN-02`. The car is sold on 15 Sep 2024. On the kept branch, the credit still pays its 20 Sep instalment, but the YAML started the kept credit in 2024-10.
- **Fix:** `probes/scen/SCEN-02.md`: the kept credit runs from 2024-09, and the worked lines count four instalments (1 600.00).
- **Moved:** `keys/probes/SCEN-02.md` cash difference −24 900.82 → −25 300.82; "added" gains "credit 2024-09".

### ERR-3 · A-10: the two rules on the 15th (text unclear; the key gains a hole)
- **Cause and what it settles:** `Q:A-10`; reader line A-10.
  - A-HR1 v2 .1 counted the balance "except the rule's own transfers". That booked A-HR4's 500.00 first and settled the order the key called unstated.
  - The papers now make the planted conflict real, so A stays a "two rules, no priority" refusal.
  - The key also missed a second hole: the joint account's bookings of 1–15 January 2026 (the records end 2025-12-31).
- **Fix:** `A/house-rules.md`:
  - A-HR1 v2 .1 now reads "…except the transfers our house rules make that day."
  - A-HR4 now reads "On the 15th, 500.00 DUK of what is above the floor (A-HR1 v2 .2) goes from the joint account to Zosia's savings."
  - `tools/keys_a.py` `q10`.
- **Moved:** `keys/A/A-10.md`. The refusal's `missing` gains "the joint account's bookings from 2026-01-01 to 2026-01-15 (the records end 2025-12-31)". Its reason is now "two rules want the same money and no priority is stated; and the day's balance they share out is not in the records".

### ERR-4 · A-11: step 17's label (case stands)
- **Cause and what it settles:** `Q:A-11` (a). The key does count Ania's 0.01 cash in Rzeka 4471-07; only the step's label left it unnamed.
- **Fix:** `tools/keys_a.py` `q11`, the label of step 17.
- **Moved:** `keys/A/A-11.md`, the derivation's step 17 label only; the key is unchanged.

### ERR-5 · A-11: the Lazur rate after June 2025 (text unclear)
- **Cause and what it settles:** `Q:A-11` (b); reader lines A-11, E1-02/Ania, E1-02/Tomek.
  - A-D31 said the bank changes its rate "by notice", and the story said Bank Lazur didn't write.
  - So a careful reader held 4.20 %, and the key's hole had no paper behind it.
  - The papers now keep the hole.
- **Fix:**
  - `tools/gen_a.py`: A-D31 now reads "The bank may change the rate; the rate in force is shown on each statement."
  - `A/story.md` line 76 now ends "…; we didn't download the second half of 2025."
  - `A/docs/A-D31.md` is regenerated.
- **Moved:** none.

### ERR-6 · A-12: where the forecast starts; the wrapped limit (text incomplete; two readings)
- **Cause and what it settles:** `Q:A-12` (a), (b); reader lines A-12 (January 2026, the start prices, items 4 and 5).
  - The forecast replaced A-HR1 v2 from February, but no paper gave January 2026's investing.
  - Item 2 ("after 2026 we take 27 300.00 and 600.00 more each year") read two ways.
- **Fix:** `A/story.md`, the forecast:
  - Item 1 says it starts from the records of 31 December 2025, at that day's prices, with January 2026's investing and Zosia's transfer left out.
  - Item 2 reads "for 2027 we take 27 900.00 DUK, and 600.00 DUK more each year after that".
  - Item 4 says what stays outside the forecast, and that Ania's Rzeka units move with the prices and are sold with the rest.
- **Moved:** none (the key already read it so).

### ERR-7 · B-12: AVG-WAGE-Q4 the household couldn't know (text incomplete; with ERR-31)
- **Cause and what it settles:** `Q:B-12` (a), `O:B-3`; reader lines B-12, the random figures.
  - The forecast is "as we see it on 2026-01-02". The key read AVG-WAGE-Q4 for February 2026 on, a row published 2026-01-20, through a lookup that ignored the known date.
  - It moves the health contributions, not the tax (SOC-4 .6).
- **Fix:**
  - `B/story.md` line 90: forecast item 6 gains "`AVG-WAGE-Q4` 9 200.00 DUK from February 2026", before the 9 800.00 from February 2027.
  - `tools/fam/b_paths.py`: `AWQ4` gains February 2026, so the gazette is read for January 2026 only.
- **Moved:** `keys/B/B-12.md` (this fix with ERR-22, from 1.0):
  - share below, with overpayments: 0.385333 → 0.386000 (se 0.008887 → 0.008890)
  - share below, without: 0.133333 → 0.134000 (se 0.006207 → 0.006220)
  - mean shortfall, with: 4 158.73 → 4 164.36
  - mean shortfall, without: 849.22 → 851.71
  - liquid 2028-12-31: 192 019.33 → 191 947.81
  - The tax is unchanged (84 081.02).

### ERR-8 · what `without:` is (case stands; the grading side's description)
- **Cause and what it settles:** `O:B-2`. No question asks for B-08 – B-11 without the twist. Their `without:` values rest on facts the dossier doesn't hold: the November rows after the 13th, and the dropped row.
- **Fix:**
  - `README.md` § Keys and the docstring of `tools/compare.py` now say: "`without:` is the grading side's proof that a twist moves 3 or more keys: no question asks for it, it isn't marked against a reader unless the reader gives one, and it may rest on facts the dossier doesn't hold."
  - The docstring of `tools/fam/b.py` says what B's `without:` removes: the reversal, with whole exports (no repeated row, no gap, no missing fee row).
- **Moved:** none.

### ERR-9 · D-D04: "becomes final" (a tense slip)
- **Cause and what it settles:** the dossier oddity of `gaps-1.0.1.md` part 1. A certificate received 2024-10-21 said it "became final on 3 November 2024".
- **Fix:** `tools/gen_d.py` now writes "becomes final", and `D/docs/D-D04.md` is regenerated.
- **Moved:** none.

### ERR-10 · A-D24 .5: what the commission covers (text unclear)
- **Settles:** reader lines A-07.
- **Fix:** `tools/gen_a.py` (`A/docs/A-D24.md` regenerated) now reads "Commission: 2 700.00 DUK for the credit's 48 instalment periods, paid by the borrowers on signing."
- **Moved:** none.

### ERR-11 · A-12: the car rule (key wrong; text incomplete)
- **Cause and what it settles:** reader line A-12, the car. The key's car rule sold BRZ-GLB-A units the account didn't hold, once the account (not its BRZ-GLB-A) was worth 60 000.00.
- **Fix:**
  - `A/house-rules.md` A-HR3 now has both conditions: the account is worth 60 000.00 DUK or more, and its BRZ-GLB-A is worth 45 000.00 DUK or more. Units are sold to 4 places, rounded up, oldest first, and the excess over 45 000.00 stays as cash.
  - `tools/keys_a.py` `forecast()`: a relation checks that the units sold are no more than the units held.
- **Moved:** `keys/A/A-12.md`, every entry (with ERR-12 on the crash paths). The car is bought on 2026-12-15 on every path. Old → new (car day, value, wrap tax, gains tax, after tax):

  | Path | Old | New |
  |---|---|---|
  | base | 2026-10-15, 761 496.76, 29 559, 1 291, 542 832.81 | 2026-12-15, 756 585.98, 29 559, 1 291, 539 184.35 |
  | crash | 2026-10-15, 608 167.07, 10 083, 1 291, 443 386.43 | 2026-12-15, 605 720.99, 10 083, 937, 441 832.12 |
  | flat | 2026-11-16, 557 610.68, 2 823, 0, 412 178.59 | 2026-12-15, 555 328.73, 2 823, 0, 410 483.22 |
  | switch | 2026-10-15, 573 501.73, 4 383, 0, 422 825.83 | 2026-12-15, 570 073.66, 4 383, 0, 420 278.95 |
  | base, what-if | gains tax 1 699, after tax 542 529.69 | 2026-12-15, 756 585.98, 29 559, 1 699, 538 881.23 |
  | crash, what-if | gains tax 1 699, after tax 443 083.31 | 2026-12-15, 605 720.99, 10 083, 1 233, 441 612.21 |
  | flat and switch, what-if | as flat and switch | as flat and switch |

### ERR-12 · PIT-8 .6: a loss on units held over 5 years (key wrong)
- **Cause and what it settles:** reader line A-12, PIT-8 .6 on the crash path. The law exempts a gain on units held over 5 years, and PIT-8 .4 adds every loss. `capital_tax` dropped the losses too.
- **Fix:** `tools/fam/a.py` `capital_tax`: a line held over 5 years is left out only when its gain is positive.
- **Moved:** A-12's crash paths (in ERR-11's table): gains tax 1 291 → 937, and what-if 1 699 → 1 233. A-05 and A-11 don't move.

### ERR-13 · A-01: what "taken away" keeps (text unclear)
- **Settles:** reader line A-01/2022/causes.
- **Fix:** `A/questions.md`, A-01's 2022 part now reads "…taken away alone with everything else as it was (the pay, and the advances taken from it, included)…".
- **Moved:** none.

### ERR-14 · a bare 0 where nothing moves (key wrong)
- **Cause and what it settles:** reader lines A-02/result and D-04/rent/difference. The README's rule is a bare 0 when nothing moves; the keys wrote `{"payable": 0}`.
- **Fix:** `tools/keys_a.py`, `tools/keys_d.py` (`dirn`). A sweep of every key in the set found no other.
- **Moved:**
  - `keys/A/A-02.md` result `{"payable": 0}` → `0`.
  - `keys/D/D-04.md` rent difference `{"payable": 0}` → `0`, in the key and in `without:`.

### ERR-15 · A-05: Tomek's proceeds (key wrong)
- **Cause and what it settles:** reader lines A-05/Tomek (4). A sale's proceeds are the SELL row's Value. The key multiplied the units by the export's 2-place display price.
- **Fix:** `tools/keys_a.py` (`by_value`, `sales_from_records`).
- **Moved:** `keys/A/A-05.md`, Tomek:
  - 2022 loss carried: 1 057.08 → 1 056.42
  - 2023 loss used: 390.85 → 390.73
  - 2024 loss used: 528.54 → 528.21
  - loss left after 2024: 137.69 → 137.48
  - The tax is unchanged; Ania's figures are unchanged.

### ERR-16 · A-08: counting money only (text unclear)
- **Settles:** reader line A-08.
- **Fix:** `A/questions.md`, A-08 now reads "…Counting money only — the car we'd have kept counts for nothing, and the 41 500.00 DUK the buyer paid counts as money we had — …".
- **Moved:** none (46 088.26).

### ERR-17 · A-HR1 v2 .3: rounding the average (text incomplete)
- **Settles:** reader lines A-09 (9).
- **Fix:** `A/house-rules.md` now reads "…divided by three and rounded to the ort, half up."
- **Moved:** none.

### ERR-18 · B-01: the car's share (text unclear)
- **Settles:** reader line B-01/form (b)/tax.
- **Fix:** `B/questions.md`, B-01 now reads "(a car's running costs at PIT-11 .1's share)".
- **Moved:** none.

### ERR-19 · B-08: what to list (text unclear)
- **Settles:** reader line B-08/problems.
- **Fix:** `B/questions.md`, B-08 now reads "…a row repeated, a row undoing an earlier one, a gap, a movement with no row — in date order: the day; the amount (…); and what it is."
- **Moved:** none.

### ERR-20 · B-HR6 .2: a probable day is no promise (text unclear)
- **Settles:** reader line B-09, FV 026/2025's status.
- **Fix:** `B/house-rules.md` B-HR6 .2 now reads "(a day given only as likely, "probably on …", isn't one)" and "without promising a day".
- **Moved:** none.

### ERR-21 · where a partial, a refusal or a by-case stands (key wrong)
- **Cause and what it settles:** reader lines B-10/2025 and B-11. The keys wrapped settled values inside a hole. README § Keys now says: "**Placement:** a partial, a refusal or a by-case stands in place of the smallest value it concerns; the settled values beside it stay plain, and a total that leaves the hole out is itself a partial."
- **Fix:** `README.md`; `tools/keys_b.py` (B-10, B-11); `tools/keys_a.py` (A-04); `tools/probefns/ask.py`, `probes/ask/ASK-02.md`; `tools/probefns/rec.py`, `probes/rec/REC-05.md`; `tools/keys_e1.py` (reads B-11's new shape). The sweep left A-01, A-05, A-11, B-09 and TIME-01 as they were: every value inside them turns on the hole.
- **Moved (no figure moved):**
  - `B-10`: "2025" is a plain table whose 2025-11-26 entry is the refusal (the day's balance).
  - `B-11`: "flat costs" is plain, and "Piotr", "Ola" and "household" are each a partial.
  - `A-04`: each of 2023-04 and 2023-05 is a by-case, and so is the total; the other months are plain.
  - `ASK-02`: Piotr is plain and Ola is the partial.
  - `REC-05`: "gross" is the partial and "inferred" is plain.

### ERR-22 · B-12: the mortgage re-levels at each reset (key wrong)
- **Cause and what it settles:** reader line B-12/mortgage 2028-12-31. B-D07 .4 sets the instalment again at the first instalment after each reset. The forecast's fast mortgage never did after 2026-01-05.
- **Fix:** `tools/fam/b_paths.py` `FastMortgage`.
- **Moved:** `keys/B/B-12.md`:
  - mortgage 2028-12-31: 347 244.50 → 347 190.44
  - with ERR-7: the share below, the mean shortfall and the liquid money (in ERR-7)
  - The tax is unchanged.

### ERR-23 · compare.py: the documented random answer (grading tool wrong)
- **Cause and what it settles:** reader lines B-12, the random figures, and D-10 (10). The questions ask for `{"value": …, "se": …}`, but compare.py failed that form.
- **Fix:** `tools/compare.py` `kinded` accepts it, with or without `"kind": "random"`.
- **Moved:** none.

### ERR-24 · B-12: savings below zero (text incomplete)
- **Settles:** reader line B-12, savings below zero.
- **Fix:** `B/story.md` forecast item 3 now reads "If B-HR5 needs more than the savings account holds, the savings account goes below zero by the rest, and the liquid money counts it so."
- **Moved:** none (the key already does this).

### ERR-25 · B-10 v3: the loop, and the balance it needs (text incomplete; key wrong)
- **Cause and what it settles:** reader line B-10/v3 on 2026-01-26.
  - The set refuses a loop no rule says how to solve (RULE-04). B's questions now say so.
  - The refusal also missed the unknown Cis balance of 2026-01-26.
- **Fix:**
  - `B/questions.md`, the opening now reads "Where two rules are each sized from the other's result and no rule says how to solve them, name the loop instead of solving it."
  - `tools/keys_b.py` `q10`.
- **Moved:** `keys/B/B-10.md`, v3's `missing` (key and `without:`) gains "the Cis balance on 2026-01-26: …".

### ERR-26 · D-01: what the set-off is taken from (text unclear)
- **Settles:** reader lines D-01 (2).
- **Fix:** `D/questions.md` now reads "…the set-off for the flat's fees (in all), and the estate's value as the agreement divides it, less that set-off."
- **Moved:** none.

### ERR-27 · D-HR4 .4: the house's value (text unclear)
- **Settles:** reader lines D-08 (2) and E1-02/Halina.
- **Fix:** `D/house-rules.md` now reads "4. The house counts at its latest valuation, as a flat does (point 2)." The house stays a hole.
- **Moved:** none.

### ERR-28 · B-HR4 .5: each amount on its own (text unclear)
- **Settles:** reader line E1-02/Piotr (the 0.01).
- **Fix:** `B/house-rules.md` now reads "…each amount on its own, to the ort."
- **Moved:** none.

### ERR-29 · B-HR4 .1: overpayments since the last instalment (text unclear)
- **Settles:** reader lines E1-02/Ola.
- **Fix:** `B/house-rules.md` now reads "…the mortgage counts at its balance after the last instalment and any overpayment since (interest since the last instalment left out)…".
- **Moved:** none.

### ERR-30 · Piotr's holes at the end of 2025 (key wrong)
- **Cause and what it settles:** reader lines B-11, E1-02/Piotr and E1-02/together.
  - Whether October 2025's contributions, health contribution, PIT advance and VAT were paid sits in the missing Cis rows of 14–30 November.
  - In E1-02, whether Piotr owes inheritance tax is also unknown.
  - The keys took both as settled.
- **Fix:** `tools/keys_b.py` `q11`; `tools/keys_e1.py`.
- **Moved (no figure moved):**
  - `keys/B/B-11.md`: Piotr becomes a partial, and the household carries his hole.
  - `keys/E1/E1-02.md`: Piotr becomes a partial with both holes, and "together" carries them.
  - B-11's `without:` (whole exports) keeps Piotr plain.

### ERR-31 · a key reads only what was known by its known-as-of date (key code)
- **Cause:** with no known date, the dated lookups skipped the publication filter, so a key could read a figure published after its question's known-as-of date (ERR-7).
- **Fix:** the known date is a required argument of every dated lookup:
  - in `tools/fam/home.py`: `figure`, `prices`, `price_on`, `run_payer`;
  - in `tools/lib/law.py`: `gazette`;
  - in `tools/fam/b.py`: `lazur_buy`, `fixing_before` and the figures built on them.
  - It is threaded through `fam/a.py`, `fam/b.py`, `fam/b_paths.py`, `fam/d.py`, `fam/d_paths.py`, `keys_*.py`, `gen_*.py`, `union_e1.py` and `probefns/val.py`.
- **Who passes what:**
  - Each key passes its question's known-as-of date.
  - A generator writing the world passes `ALL_KNOWN` (`fam/home.py`), with a comment.
  - The world's replays that a key calls (`b.business`, `b.simulate`, D's `sim`) pass `ALL_KNOWN` too, because their rows are the records'.
- **Checked:**
  - The replays were checked by changing every figure, price and rate published or dated after each question's known-as-of date and rerunning the keys. No key of a question known by that date moved, in A, B, D or E1.
- **Moved:** none.

### ERR-32 · missing counted as 0 (key and probe code)
- **Cause:** VAL-06 (ERR-1) counted a year with no entry as 0.
- **Fix:** the key and probe code was swept for the same pattern. Each place that counts a blank or missing input as 0 now cites the paper line that states it, in `tools/keys_a.py`, `tools/keys_b.py`, `tools/probefns/pty.py` and `tools/probefns/val.py`. No place without a paper behind it was found.
- **Moved:** none.

## For session 4

### The nine items

| Item | 1.0 status | Settled | Errata |
|---|---|---|---|
| `P:VAL-06` | case suspect | text incomplete (2024's earnings now stated as none) | ERR-1, ERR-32 |
| `P:SCEN-02` | case suspect | key wrong (−24 900.82 → −25 300.82) | ERR-2 |
| `Q:A-10` | case suspect | text unclear: the papers now make the conflict real; the key stays a refusal and gains a hole | ERR-3 |
| `Q:A-11` | case suspect | (a) case stands, label fixed; (b) text unclear, the papers now keep the hole | ERR-4, ERR-5 |
| `Q:A-12` | case suspect | (a) text incomplete, (b) two readings: the forecast now says where it starts and what 2027's limit is; the key moved for the car rule (ERR-11) and PIT-8 .6 (ERR-12) | ERR-6, ERR-11, ERR-12 |
| `Q:B-12` | case suspect | (a) text incomplete and key wrong: AVG-WAGE-Q4 for 2026 now stated; the key no longer reads the later gazette row; (b) about session 3's text, no case change | ERR-7, ERR-22, ERR-24, ERR-31 |
| `O:B-2` | case suspect | case stands; what `without:` is now written | ERR-8 |
| `O:B-3` | case suspect | as `Q:B-12` (a) | ERR-7, ERR-31 |
| D-D04's "became final" | (part 1, no row) | a tense slip: "becomes final" | ERR-9 |

### The rows to reread

Session 3 judged these rows against cases 1.0. Each case's key or papers have changed since:
- **Key:** the key moved.
- **Papers:** a house rule, document, story line or question it reads changed in meaning.
- **Wording:** a change of words only.

Two README changes reach every question: the placement rule (ERR-21) and what `without:` is (ERR-8). So do the shared lookups' required known date (ERR-31), which moved no key.

| Row | 1.0 status | Changed | Errata |
|---|---|---|---|
| `P:VAL-06` | case suspect | papers (card), key derivation | ERR-1 |
| `P:SCEN-02`, `O:SCEN-02-1` | case suspect, awkward | papers (card), key | ERR-2 |
| `P:ASK-02` | awkward | key (placement), card's `expect` | ERR-21 |
| `P:REC-05` | awkward | key (placement), card's `expect` | ERR-21 |
| `Q:A-01` | breaks | papers (question) | ERR-13 |
| `Q:A-02` | breaks | key (bare 0) | ERR-14 |
| `Q:A-04` | expressed | key (placement) | ERR-21 |
| `Q:A-05` | can't express | key (Tomek's losses) | ERR-15 |
| `Q:A-07` | expressed | papers (A-D24 .5) | ERR-10 |
| `Q:A-08` | awkward | papers (question; A-D24 .5) | ERR-16, ERR-10 |
| `Q:A-09` | can't express | papers (A-HR1 v2 .1 and .3) | ERR-3, ERR-17 |
| `Q:A-10` | case suspect | papers (A-HR1 v2 .1, A-HR4), key | ERR-3 |
| `Q:A-11` | case suspect | papers (A-D31, story line 76), key derivation | ERR-4, ERR-5 |
| `Q:A-12` | case suspect | papers (forecast, A-HR3), key | ERR-6, ERR-11, ERR-12 |
| `Q:B-01` | breaks | papers (question) | ERR-18 |
| `Q:B-03` | awkward | papers (B-HR4 .1 and .5) | ERR-28, ERR-29 |
| `Q:B-08` | breaks | papers (question) | ERR-19 |
| `Q:B-09` | breaks | papers (B-HR6 .2, B-HR4 .5) | ERR-20, ERR-28 |
| `Q:B-10` | awkward | papers (questions' opening), key (placement, v3's hole) | ERR-21, ERR-25 |
| `Q:B-11` | awkward | papers (B-HR4 .1 and .5), key (placement, Piotr's hole) | ERR-21, ERR-28, ERR-29, ERR-30 |
| `Q:B-12` | case suspect | papers (forecast items 3 and 6), key | ERR-7, ERR-22, ERR-24 |
| `O:B-2` | case suspect | README, docstrings | ERR-8 |
| `O:B-3` | case suspect | papers (forecast item 6), key | ERR-7, ERR-31 |
| `Q:D-01` | awkward | papers (question); wording (D-D04) | ERR-26, ERR-9 |
| `Q:D-02` | awkward | wording (D-D04) | ERR-9 |
| `Q:D-04` | awkward | key (bare 0) | ERR-14 |
| `Q:D-08` | awkward | papers (D-HR4 .4) | ERR-27 |
| `Q:D-09` | awkward | wording (D-D04) | ERR-9 |
| `Q:E1-01` | awkward | wording (D-D04) | ERR-9 |
| `Q:E1-02` | awkward | papers (A-D31, story line 76, B-HR4 .1 and .5, D-HR4 .4), key (Piotr's holes) | ERR-5, ERR-27, ERR-28, ERR-29, ERR-30 |
| `Q:E1-03` | awkward | wording (D-D04) | ERR-9 |

The other rows' cases and keys are as session 3 judged them.

## cases 1.0.2 (2026-10-29)

The tally 2 design round proposed two errata (`tmp/design2/case-errata-proposals.md`, CE-1 and CE-2), after the user's choices at G2 (items `keys` and `a12`). An exam-side session applied them. Each figure below comes from the key scripts working from the papers. A tally 2 print was read only as evidence of a reading. The rules of the change are those of cases 1.0.1, above.

`VERSION` reads `cases 1.0.2`; `README.md`'s title and its freeze paragraph say so; `MANIFEST.sha256` holds the new hashes (no file added or removed).

Moved in all: 3 key files (B-11, E1-02, A-12), each with figures moved. No paper changed. README is unchanged: Placement settles CE-1's open edge (ERR-33).

### ERR-33 · B-11: Ola's Wrzos balance has a known part (key wrong)
- **Cause and what it settles:** CE-1; `Q:B-11`. The key left Ola's whole Wrzos balance out of her partial. By Placement ("a partial … stands in place of the smallest value it concerns"), the smallest missing value is what she spent after her last statement, not the balance:
  - the March 2025 statement reads 10 933.10 on 2025-03-31 (`records/wrz-ola-2025-03.csv`);
  - after it the papers settle her pay coming in (B-D25b: on the 28th, or the last business day before; its net by PIT-4, SOC-1, SOC-2), 49 390.20 for April to December;
  - and her order to Piotr going out (B-HR1; the nine rows in Cis), 15 300.00;
  - so the known part is 45 023.30, and the hole is what she spent from the account from 2025-04-01 to 2025-12-31.
- **The open edge (J-TX2):** an answer that names the spending in parts, such as one unknown per month, names the key's missing fact. Each part is unknown and together they are the key's, so "holes are judged both ways" passes it: every listed fact is named and nothing settled is called missing. No README sentence is added.
- **Fix:** `tools/keys_b.py`: `wrzos_known` (the known part, step 3), `OLA_SPEND` (the hole), `worth` (Ola's "Wrzos"), `q11` (the household counted whole).
- **Moved:** `keys/B/B-11.md`:
  - Ola 97 763.03 → 142 786.33; household 519 623.38 → 564 646.68;
  - the missing fact "Ola's Wrzos balance on 2025-12-31: her last statement is March 2025's" → "what Ola spent from her Wrzos account from 2025-04-01 to 2025-12-31 (her pay in and her order to Piotr out are known): her last statement is March 2025's", in Ola and in household;
  - `without:` Ola 97 765.15 → 142 788.45, household 519 628.68 → 564 651.98;
  - a derivation step added (step 3, the known part).
- **Against tally 2's print:** the same known parts (142 786.33 and 564 646.68) with the spends as nine monthly terms.

### ERR-34 · E1-02: B-11's known part carried in (key wrong)
- **Cause and what it settles:** CE-1's second half; `Q:E1-02`. Step 2 takes Ola's figure and hole from B-11's key, so ERR-33 carries in.
- **Fix:** `tools/keys_e1.py`, step 2's wording only. The figures come from `keys/B/B-11.md` as before.
- **Moved:** `keys/E1/E1-02.md`:
  - Ola 97 763.03 → 142 786.33, with ERR-33's missing fact;
  - together 1 148 588.50 → 1 193 611.80, with the same fact in its list.
- **Against tally 2's print:** Ola as B-11's (`scratch/lead/check/e1-02-B16.out`).

### ERR-35 · A-12: a fund's price, rounded by no stated mode (text incomplete; three readings)
- **Cause and what it settles:** CE-2; `Q:A-12`.
  - The forecast's item 3 says only that each price is "then rounded to the places its fund quotes in". GEN-5 names no rounding for a price, and GEN-5.5 rounds nothing it doesn't name. No document of A states one either.
  - The dossier names three modes: half up (GEN-5, A-HR1 v2 .3), down (units, A-D08 .4, A-D09 .3, A-D11 .1, A-D12 .1) and up (A-HR3).
  - Half even appears nowhere, so it isn't a reading. The user chose these three readings on 2026-10-29.
  - Every price step rounds by the reading's mode. Under each reading, the key runs the forecast in full.
- **Placement:** a figure the mode moves is a by-case (`half up`, `down`, `up`). A figure it doesn't move stays plain: the car's day everywhere, every figure of the flat path (a price times 1 is itself under any mode), and the switch path's gains tax of 0.
- **Fix:** `tools/keys_a.py`: `MODES`, `forecast(path, whatif, mode)` (the price step, and the wrapped limit's running total kept per mode), `q12`.
- **Moved:** `keys/A/A-12.md`: in base, crash and switch and their 2030 what-ifs, the value, wrap tax, gains tax (base and crash) and after-tax figure become by-cases. Their half-up cases are ERR-36's figures. For example, base:
  - value {half up 756 586.06, down 755 116.69, up 757 996.21};
  - wrap tax {29 559, 29 383, 29 729}; gains tax {1 291, 1 286, 1 297};
  - after tax {539 184.41, 538 227.22, 540 101.32}.
- **Against tally 2's print** (`scratch/lead/check/snap14-house/A-q12.out`): its `half-up` and `down` cases equal the key's in all 8 worlds, every figure. It has no `up` case.

### ERR-36 · A-12: the sale on 2035-12-31 is one per holding (key wrong)
- **Cause and what it settles:** CE-2's few orts. Item 5 sells everything in the three accounts on 2035-12-31.
  - The case set's sale is one amount per sale: the ort of units × the day's price, shared over the lots by units (`by_value`, as every sale in the records is read).
  - The forecast instead rounded each lot's units × price and added them up.
  - Now each account's holding of a fund is one sale. Its proceeds make the value and the wrapped account's worth, and they are shared over the lots for the gains (PIT-8 .2, .6).
- **Fix:** `tools/keys_a.py` `forecast`.
- **Moved:** the half-up figures (the 1.0.1 key's single values were half up):
  - base 756 585.98 → 756 586.06 and 539 184.35 → 539 184.41;
  - crash 605 720.99 → 605 721.05 and 441 832.12 → 441 832.17;
  - flat 555 328.73 → 555 328.60 and 410 483.22 → 410 483.13;
  - switch 570 073.66 → 570 073.63 and 420 278.95 → 420 278.92;
  - the 2030 what-ifs: base 538 881.23 → 538 881.29, crash 441 612.21 → 441 612.25; flat and switch as above.
  - No tax moved.

### ERR-37 · A-12: Spring Day from 2027 (case stands; the key now shows it)
- **Cause and what it settles:** CE-2's related note; `O:A-3`.
  - GEN-1 .3 leaves Spring Day unsettled for each year without a notice, 2027 to 2035. The key's calendar simply had none after 2026.
  - Under each Monday of April, April's investing day (A-HR2: the 15th or the next business day) stays before the month's last day, where prices move. So it buys at the same price.
  - No lot crosses PIT-8 .6's 5 years before the sale, and the car's day (2026-12-15) is in a settled year.
  - So every reading gives the same figures: the key takes none, and states this as step 1.
  - An answer that makes a figure a hole for the Spring Day's sake calls missing a fact the dossier settles.
- **Fix:** `tools/keys_a.py` `spring_moves`, checked as the key runs.
- **Moved:** `keys/A/A-12.md`, derivation step 1 added; no figure.

### For session 5 · the rows to rejudge

Rejudge these rows against cases 1.0.2, on `engine/tools/refcore`:

| Row | Changed | Errata |
|---|---|---|
| `Q:B-11` | key: Ola's and the household's partials, their hole, `without:` | ERR-33 |
| `Q:E1-02` | key: Ola and together | ERR-34 |
| `Q:A-12` | key: by-cases over the price's rounding, the sale per holding, the Spring Day step | ERR-35, ERR-36, ERR-37 |
| `O:A-3` | A-12's key now says the Spring Day moves no figure | ERR-37 |

Differences from tally 2's prints that remain: A-12 has an `up` case, which `px_round` lacks. B-11 and E1-02 give no figure that differs. No other key or paper changed.

## cases 1.0.3 (2026-10-29)

Session 5's exam took up the three case suspects of household B that the tally 2 design round left open (`tmp/design2/G2-list.md` §4: `Q:B-05`, `Q:B-12`, `O:B-2`; `O:B-6` is B-05's and B-12's observation). An errata session settled them. Each figure below comes from the key scripts working from the papers; a tally 2 print was read only as evidence of a reading. The rules of the change are those of cases 1.0.1, above.

`VERSION` reads `cases 1.0.3`; `README.md`'s title and its freeze paragraph say so; `MANIFEST.sha256` holds the new hashes (no file added or removed).

Moved in all: 1 key file (B-05), with figures moved. No paper changed.

### ERR-38 · B-05: Bank Jodła's list comes after the question's date (key wrong)
- **Cause and what it settles:** `Q:B-05`, `O:B-6`. B-05 is known as of 2026-01-05. Its key took 2025's overpayments from Bank Jodła's list (B-D28), received 2026-01-07: against ERR-31, a key reads only what was known by its known-as-of date.
  - On 2026-01-05 the overpayments are the Cis exports' rows "loan 7710-22 overpayment", each month exported in the first days of the next (story): the same nine.
  - But the November export stops at 2025-11-13 (B-08), and B-HR2's run of 2025-11-26 falls on the days it lacks. Whether anything was overpaid from 2025-11-14 to 2025-11-30, and how much, no paper known on 2026-01-05 says. B-D28 says none, on 2026-01-07.
  - Nothing else settles it by then. The savings export shows no transfer on 2025-11-26, so B-HR2's surplus that day was at most 3 000.00, and it may have gone to the mortgage. December's instalment, 2 855.21 as November's, doesn't settle it either: by the key script's own mortgage (B-D07 .4 re-levels after each overpayment), an overpayment of about 56.00 on 2025-11-26 leaves it at 2 855.21. B-10's key already holds that day's run unknown.
- **Placement:** an overpayment on those days moves the interest of the instalment of 2025-12-05, the balance on 2025-12-31 and the instalment of 2026-01-05, in both runs and in the difference. It moves neither the 2022–2024 interest, nor the interest of 2025's instalments to 2025-11-05, nor the last instalment's day (B-D07 .4 keeps the end). So:
  - 2025's interest is a partial: the instalments of 2025-01-05 .. 2025-11-05, missing the December instalment's interest;
  - the balance and the instalment are refusals;
  - the key checks this with trial overpayments on 2025-11-14, 2025-11-26 and 2025-11-30.
- **Fix:** `tools/keys_b.py` `q05` (with `received`): the overpayments from the Cis exports; a check that B-D28 is received after the known-as-of date; the holes and their placement.
- **Moved:** `keys/B/B-05.md`, in "as it was", "without the holidays" and "difference":
  - interest 2025: 29 806.08, 29 637.00, −169.08 → partials 27 584.48, 27 429.15, −155.33;
  - balance 2025-12-31: 373 346.69, 370 993.16, −2 353.53 → refusals;
  - instalment 2026-01-05: 2 653.38, 2 660.73, 7.35 → refusals;
  - each hole names "whether anything was overpaid on the mortgage from 2025-11-14 to 2025-11-30, and how much (B-HR2's run of 2025-11-26 falls on those days)…" (a partial: the December instalment's interest, which turns on it);
  - unchanged: the 2022–2024 interest, the last instalment (2047-08-05, 2047-03-05, −5);
  - the derivation gains steps 1–3 (the exports' overpayments, the days no export shows, what they reach) and a relation.
- **Against tally 2's print** (`tmp/design2/scratch/mc-b/out11/cs_b05.out`): at cut 2026-01-05 the plain figures are the key's, and the holes sit in the same cells, named by the November gap's own questions (`gap_ziarno`, `gap_cards`, `gap_card_pays`), from which B-HR2's run would follow. Its 2025 interest is a whole hole where the key has a partial. At cut 2026-01-07 it gives the 1.0.2 figures, B-D28's.

### For session 5 · the rows to rejudge (cases 1.0.3)

Rejudge these rows against cases 1.0.3:

| Row | Changed | Errata |
|---|---|---|
| `Q:B-05` | key: 2025's interest a partial, the balance and the instalment of 2026-01-05 refusals, in each run and the difference | ERR-38 |
| `O:B-6` | B-05's half settled; B-12's half not (below) | ERR-38 |

Not changed: `Q:B-12`. Its key reads the same fact on 2026-01-02, through the forecast's start (the mortgage as it stood on 2025-12-31). The fact reaches the mortgage on 2028-12-31 and, through the instalments, the shares below, the mean shortfalls, the liquid money and B-HR3's verdict: by the rules, 7 of its 9 figures. Its fix waits for a paper change made in both trees, as ERR-7's was. `O:B-2` stands as ERR-8 settled it.

## cases 1.1 (2026-10-29)

Additions for session 4.5c, made by the case writer C1 from the brief of the user and the master session; no erratum. Each figure below comes from a key script's run (`keys_a.py`, `keys_b.py`, `keys_d.py`, `keys_probes.py`) and is copied from its key. The entries are numbered ADD-n, apart from the errata. The rules of the change are those of cases 1.0.1, and one more: nothing that exists moves.

`VERSION` reads `cases 1.1`; `README.md`'s title, its freeze paragraph, its format tree, its marking paragraph and its questions line say so (the cap of 12 questions per household is lifted for the three additions, by the user's and the master session's decision); `MANIFEST.sha256` holds the new hashes, with the files added below.

Moved in all: no key, no paper of a household, no record. Every file under `keys/` that existed is byte-identical: `check.py --regen` regenerates 253 files of A, B, D and E1 (the 250 of cases 1.0.3 and the three new keys), 0 changed, and `keys_probes.py` rewrites every old probe key unchanged. Added: 3 questions with their keys, 2 probe cards with their keys, 1 edge, 1 knob with its key under `keys-small/`, and `compare.py --small`.

### ADD-1 · A-13: had Ania stopped working (a scenario)
- **What it asks:** `A/questions.md` A-13, as of 2024-12-31, known as of 2025-04-30. Ania stops working for Kalina from 2023-07-01 (no pay for any time from then, no plan contributions on it; the rest as it was unless the law makes it follow). (a) the joint 2023 return as A-01 asks it, each line's difference from A-01's 2023 and why; (b) the same with her 200 gifted units sold at DM Kopia on 2023-12-15 at that day's price, no fee, instead of 2024-03-07: the lots, each lot's gain and whether taxed, her fund tax for 2023 and 2024 and their differences; (c) as (a), stopping from 2023-04-01.
- **Why (c):** the brief's date moves nothing. Ania is on parental leave from 2023-04-27 (A-D06-2023b), so Kalina's last 2023 payment is on 2023-05-15. Her parental pay runs on: SOC-8 .1 reads her insurance on the birth day. So (a)'s key is A-01's 2023, every difference 0, with the reasons. An honest answer, and a trap for a model that cuts the parental pay. (c) keeps the edges the brief meant: from 2023-04-01 her April pay goes, and the law takes her parental pay with it (SOC-8 .1: not insured on the birth day); the child relief is no longer cut, and the joint tax and each part move (joint tax 6837 → 2835).
- **(b):** the November 2018 lot is held more than 5 years since Halina's purchase on any day of that month, so it is exempt; the March and August 2019 lots are taxed. The test settles by month, so no day is needed (2023 tax 237). The 2024 difference is a by-case: A-05's March 2019 day is still missing.
- **Edges:** SCEN-02, SCEN-01, LAW-05, TIME-04, PTY-02, PTY-04, LAW-07, ASK-05, OWN-06, OWN-07, TIME-05, TIME-01.
- **Files:** `tools/keys_a.py`: `key_of`, `a13_return` (A-01's return with the stop; with no stop it equals `household_return(2023)`, checked), `a13_lines`, `a13_part`, `kopia_lots`, `q13`; `main` runs it. `keys/A/A-13.md`.

### ADD-2 · B-13: had the floren been 5 % cheaper (a scenario)
- **What it asks:** `B/questions.md` B-13, as of 2025-12-31, known as of 2026-01-07. Every floren rate dated from 2025-07-01, the CBL's mid rates and Bank Lazur's table alike, is times 0.95, rounded to 4 places, half up (the question states it). (a) the revenue and VAT of the six Kessel invoices issued from 2025-07-01 (FV 015 .. FV 028/2025, B-03's; checked), and the Lazur florens on 2025-12-31 with their worth; (b) the same with a rule of the scenario's own, **B-13 rule 1**: on the 1st (or the next business day), the florens above 2 000.00 FLR at the day's end go to Cis. Asked: each sale, the Lazur account at the year's end, and the currency gain or loss against the florens' worth on the sale's day as B-HR4 .5 counts it.
- **Why "B-13 rule 1", not B-HR7:** it is the scenario's, not B's, so `house-rules.md` is unchanged, and `check.py` requires every `B-HRn` a key cites to be in it.
- **What the rule does:** one sale, on 2025-10-01: 1 904.00 FLR, credited 2025-10-02, a loss of 191.35. On 2025-09-01 the day's arrival is counted and leaves 1 952.00, not above 2 000.00. On 2025-12-01 the story's own sale of that day's florens comes first. Both are TIME-07's order, as the rule states it.
- **Edges:** CUR-01, CUR-02, CUR-03, SCEN-01, SCEN-03, RULE-01, TIME-02, TIME-07.
- **Files:** `tools/keys_b.py`: `FLR_LOW`, `B13_KEEP`, `flr_low`, `key_of`, `b13_rule` (the rule function), `q13`; `main` runs it. `keys/B/B-13.md`.

### ADD-3 · D-13: had the fund been taxed (a scenario on D-10's paths)
- **Pure ask or scenario:** D's long view already sells fund units when the account goes below 0 (point 8), but its point 7 says the fund's gains are not taxed. A pure ask would give 0 by assumption, so D-13 is a scenario that replaces point 7 with the law (PIT-8). It states what the law needs: units carried exactly; the 45 000.00 of 2027 bought on 2026-12-31 as 45 000 units; purchases and sales at the unit's worth on 31 December; each year's tax paid with the next year's money.
- **What it asks:** on D-10's paths (seed 20261001 + i, 10 000 paths, the same lifetimes, checked path by path), the gains tax to her death, the gains on the units sold (taxed or not), the share of paths selling units; with zero volatility, the first sale year and closed forms of the expected gains and gains tax. Every figure is in the keys' random form.
- **The result:** the gains tax is 0 on every path. Units are first sold in 2039, when care starts, and the oldest lots go first (PIT-8 .3), so every gain sold is on units held more than 5 years (PIT-8 .6). On the 42 paths where a younger lot is sold at a gain, that year's losses or a loss carried (PIT-8 .5, on 30 of them) cover it. So the tax answers "0", and the gains (mean 93 973.21, closed form 134 048.52 with zero volatility) carry the lots' cost basis and their order.
- **Checks:** every path untaxed (rate 0) reproduces D-10's dry year and net worths; the fund never refills once emptied, so no rule for a unit's worth at zero units is needed.
- **Edges:** OWN-06, OWN-07, RAND-01, RAND-02, RAND-03, RAND-04, SCEN-01, VAL-07, TIME-05, LAW-09.
- **Files:** `tools/keys_d.py`: `GAINS_RATE`, `FUND_BOUGHT`, `FUND_UNITS`, `d10_draws`, `whole_duk`, `lot_run` (the lot function on paths: which lots go, at what cost, each year's tax), `F_add_years`, `F_`, `show`, `q13`; `main` runs it. `keys/D/D-13.md`.

### ADD-4 · The pair card CUR-01 × RAND-01, and the pair convention
- **The card:** `probes/pairs/CUR-01+RAND-01.md`. A floren invoice whose law-rate conversion reads a drawn mid rate: a random walk from 4.2000, one draw per business day, seed 20260301, 20 000 paths. It asks the mean revenue at the law's rate and the mean cash at the bank's (random, against the exact means 4 250.00 and 4 400.00), and whether the same seed gives the same answer.
- **The convention**, described in `probes/README.md`:
  - the file is `probes/pairs/<EDGE>+<EDGE>.md`;
  - its `edge:` line lists both edges;
  - its key is `keys/probes/<EDGE>+<EDGE>.md`, headed by the pair's name (`lib/keys.write` is untouched);
  - its function is `probefns/pairs.py` `CUR_01__RAND_01`;
  - a pair is no edge, so `boundaries.md` doesn't list it.
- **The tools, each with the smallest change:**
  - `keys_probes.py`: `+` maps to `__` in the function's name;
  - `index.py`: `probe_cell` lists the pair beside both edges' cards;
  - `check.py`: `pair_cards`, so each pair card names two visible edges and has its key;
  - `check_edges.py`: unchanged, since it reads only `boundaries.md`, which lists no pair.

### ADD-5 · LAW-20: a circular rule the law solves by its own named method (a new edge), and check.py's exemption
- **The edge:** `boundaries.md` LAW-20, in the entry format: hard, where, key, probe, ids MX-35, which RULE-04 also carries. The axes table reads LAW 14 and 93 in all. RULE-04's note now names LAW-20 as the solver side. The paragraph "Two mechanics are carried in part" says that MX-35's refusal side is RULE-04, its named solver LAW-20, and its integration stays with the 79 cases (UL-07, UL-08).
- **Why LAW-20, not LAW-14:** the holdout keeps edges of its own, whose ids the visible side can't see. A number well past the visible ones avoids a clash, and the entry says so.
- **The card:** `probes/law/LAW-20.md`, with `probefns/law.py` `LAW_20`. A deduction and a premium credit are each sized from the other. The law's four steps give (a), with the method's closed form checked as a relation. The iteration the law also allows gives (b): 7 rounds, within 1.00 of the closed-form fixed point. The law's method is the key.
- **Used by a question?** No visible question uses LAW-20 honestly. D-13's tax is paid the next year (PIT-8 .4), so its sale doesn't cover a tax of its own, and no visible law or house rule names a solver. So `check.py` has a narrow exemption (`carried_by_79`): an edge whose `where:` reads "no visible household; carried in integration by the 79 cases' models (…)" needs its probe but no question, and the coverage line names it, with its ids: "LAW-20 (UL-07, UL-08)". Only LAW-20 has that wording.

### ADD-6 · The run-end knob in keys_b.py, and keys-small/
- **The knob:** `python3 -I tmp/cases/tools/keys_b.py 30 --end 2027-01-31` writes only `keys-small/B/B-12.md`, never `keys/`. Without `--end`, the script does exactly what it did; `keys_b.py 30` still sets only the path count.
- **The code:**
  - `fam/b_paths.py`: `run`, `plan_tax_closed_form` and `forecast` take `end=END`, so the default is the old run;
  - `keys_b.py`: argument parsing, `put(small=…)`, `q12(end)`, and `main`'s knob branch.
- **What the end-date key states, against B-12's:**
  - its header and YAML give the path count (30) and the end date, and its `as_of` is 2027-01-31;
  - the labels are B-12's, so the same answers file can be marked;
  - "mortgage 2028-12-31" is the balance on 2027-01-31, after the instalment of 2027-01-05, with both 2026 resets and the 2027-01-01 reset re-levelled;
  - "liquid 2028-12-31" is Cis plus savings at the end of January 2027;
  - "tax 2026-2028" sums the months 2026-01 .. 2027-01, so January 2027 carries the path's 2027 law draw;
  - "share below", "mean shortfall" and "overpayments stopped" (B-HR3's test) read the month ends 2026-01 .. 2027-01;
  - "zero variance tax" is the closed form over those 13 months.
- **What is left out:** what falls due after the end (the 2026 health settlement on 2027-04-20, January 2027's payments due in February).
- **The paths:** each is the first part of the same path of B-12's run. The draws are made for all 36 months as B-12 makes them, and the paths are B-12's first 30. At 30 paths the decision is the same as B-12's (stopped), with shares and means of their own.
- **The folder:** `keys-small/README.md` says how each file there was made.

### ADD-7 · compare.py --small
- **What it does:** with `--small`, a random answer passes if `|value − r| ≤ 4 × √(se² + se_r²)`; the precision bar `se ≤ p` is dropped, and everything else is as before.
- **Its self-test:** `self_test_small`, run inside every `--self-test <H>`, checks every random figure of the house's keys three ways: an answer with se = 2p fails without `--small` and passes with it; one 5 combined standard errors or more away fails both ways.
- **`--keys-root`:** it already took any folder of house folders. `compare.py --self-test B --keys-root tmp/cases/keys-small` passes.

### For session 5 · cases 1.1

- **No existing key moved:** every file of cases 1.0.3 under `keys/` is byte-identical, and so is every paper.
- **The hidden households:** their key scripts share `lib/` and `fam/home.py` with the visible ones, and neither was edited. `fam/b_paths.py` changed only by options that default to the old run. Session 5 confirms the hidden keys with its own run (`check.py --hidden … --regen`); the case writer never ran it.
- **The new rows to model and judge:**

| Row | What | Entry |
|---|---|---|
| `Q:A-13` | a scenario on A: the stop in (a) moves nothing; (c) moves; (b) the lots | ADD-1 |
| `Q:B-13` | a scenario on B: every FLR rate 5 % lower, and a rule of its own | ADD-2 |
| `Q:D-13` | a scenario on D's paths: the fund taxed under PIT-8 | ADD-3 |
| `P:CUR-01+RAND-01` | the first pair card | ADD-4 |
| `P:LAW-20` | a new edge's card; no question | ADD-5 |
| `Q:B-12` (small) | `keys-small/B/B-12.md`, for a run cut to 30 paths and 2027-01-31, marked with `--small` | ADD-6, ADD-7 |

## cases 1.1.1 (2026-10-29)

Session 4.5f: tools only. No key, card, paper or record moved; every key file is byte-identical. The case set moved from `tmp/cases/` to `engine/tests/cases/` by `git mv` (its bytes unchanged); `tally-1.0.1/` stayed in `tmp/cases/`. `VERSION` reads `cases 1.1.1`; `README.md`'s title and freeze paragraph say so; `MANIFEST.sha256` holds the new hashes of the three tools (no file added or removed).

### TOOL-1 · `compare.py` marks the probe keys (J's I-3)
- **Cause:** `compare.py` read keys by `question:` and knew no kinds `value` / `relation`, so the probe keys (`edge:`, values wrapped) couldn't be marked; J unwrapped copies by hand-run script (`tmp/session4_5/c/judge/probe_keys.py`).
- **Fix:** `tools/compare.py`: the house `P` reads `keys/probes/`; the key of edge E is question `P-E`, and each `{"kind": "value" | "relation", "value": X}` is marked as X (J's `unwrap`, folded in). Its self-test: the nudge of one number leaves a refusal's numbers and a random figure unmoved (neither is marked by moving 0.01: a refusal's side values aren't marked, a random figure has `self_test_small`), and takes a bare-number key's moved value (it dropped it). The households' self-tests are unchanged: A 12, B 13, D 13, E1 3 moved numbers fail. P: 90 keys pass as answers, 80 moved numbers fail.
- **`tools/check.py`:** runs `compare.py --self-test P` with the households'.
- **Moved:** nothing.

### TOOL-2 · The tools find the case set at `engine/tests/cases/`
- **Cause:** `check.py` took the repository as the case set's grandparent, and `check_edges.py` as its great-grandparent, which held at `tmp/cases/` only.
- **Fix:** `tools/check.py` (`REPO`), `tools/check_edges.py` (`ROOT`, and `boundaries.md` read beside the tools). The other tools find their files from their own place and need nothing.
- **Moved:** nothing.
