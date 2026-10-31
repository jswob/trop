# Errata

## cases 1.0.1 (2026-10-25)

The visible set's errata are in `tmp/cases/ERRATA.md` (ERR-1 to ERR-32). A change to shared code (`tools/lib/`, `tools/fam/`, `tools/compare.py`) is entered there. Its effects on the hidden set are entered here, with the changes made only in this tree. The work record (the readers, the settlements, the fixer's change list, the checks) is in `plan/errata/` (`PROGRESS.md`, `rows/`).

In all: 9 hidden tool files changed for the lookahead fix, and 2 hidden keys narrowed in shape (no figure moved). One file added: `reader/compare-1.0.1.txt`. No document, record or dossier text changed, and no other hidden key moved.

### HERR-1 · the known date, in the hidden tools (ERR-31's hidden side)
- **Cause:** as ERR-31. A dated lookup with no known date read every published row, so a key could read a figure published after its question's known-as-of date.
- **Fix:** the known date is a required argument of every dated lookup:
  - in `tools/c_law.py`: `rfig` and `cbl_ref` read the rows published by `known`; `cbl_on_first` raises when the month's first day is after `known`, instead of walking back past it;
  - in `tools/c_sim.py`: `lazur_table` reads Bank Lazur's rates up to `known`;
  - with the visible `figure`, `prices`, `price_on` and `gazette`, it is threaded through `c_facts.py`, `c_sim.py`, `c_tax.py`, `keys_c.py`, `keys_e2.py`, `gen_c.py`, `gen_c_gazette.py` and `union_e2.py`.
- **Who passes what:**
  - Each key passes its question's known-as-of date: C-01 to C-04, C-06, C-08, C-10, E2-01 and E2-02.
  - The generators (`gen_c.py`, `gen_c_gazette.py`, `union_e2.py`, and the `__main__` blocks of `c_sim.py` and `c_tax.py`) pass `ALL_KNOWN`.
  - The world's replays that `keys_c.py` calls (`X.household_taxes`, `S.accounts`, and `F.jan_slips`: Jan's payslips, C-D20) pass `ALL_KNOWN`, because their rows are the records'.
- **Checked:** `plan/errata/tools/r1_perturb.py` changed every gazette row, price and rate published or dated after each question's known-as-of date (×1.37), then reran the key scripts. No key of a question known by that date moved: C-01, C-02, C-03 and C-07, C-10, the 2026-09-30 group, and E2-01 to E2-03 (`plan/errata/rows/r1-slow.txt`, `r1-fast.txt`).
- **Moved:** none (`union_e2.py`: 0 changed).

### HERR-2 · missing counted as 0, in the hidden keys (ERR-32's hidden side)
- **Fix:** `tools/keys_c.py`, C-05's `fbo.get(k, ZERO)`, now carries a comment citing its papers:
  - C-D25: "Payments stop from December 2025";
  - C-D26: they resume in September 2026;
  - the joint export covers 01.01.2024 – 30.09.2026.
- **Noted, not changed:** `tools/c_sim.py` line 288, `month_end.get(pk, ZERO)`, is in the world's C-HR2 .3 sales. A month with no month-end balance never triggers a sale, and the keys read the sales from the records.
- **Moved:** none.

### HERR-3 · placement of a by-case, in the hidden keys (ERR-21's hidden side)
- **Cause:** README § Keys now places a by-case at the smallest value it concerns (ERR-21). compare.py marks a by-case key against a table answer as a fail, so a model that follows the README would fail a hidden key that breaks the rule. ERR-21's sweep found two such keys:
  - C-08's key "bike": its three cases share "had yes": false;
  - C-11's `without:` "gift tax": its two cases share "report needed", "by" and "tax".
- **Fix:** `tools/keys_c.py` gets a helper, `by_case`, under which a field every case shares stays plain. C-08's "bike" and C-11's `without:` "gift tax" are built with it.
- **Moved (no figure moved):**
  - `keys/C/C-08.md`, "bike":
    - old: one by-case over {needed yes, had yes, owed back, due};
    - new: "had yes": false plain, with "needed yes", "owed back" and "due" each a by-case over the same three rates (CBL mid as GEN-4 converts / Lazur's buy / Lazur's sell: true / false / true; {"Jan": 2522.28} / 0 / {"Jan": 2587.62}; 2025-08-31 / null / 2025-08-31).
  - `keys/C/C-11.md`, `without:` "gift tax":
    - old: one by-case;
    - new: "report needed": false, "by": null and "tax": 0 plain, with "return by" a by-case (null / 2026-10-19), "settled by" ["whether IGT-4 .4 reaches a released debt"].
- **The hidden reader's comparison, rerun** (`reader/compare-1.0.1.txt`, beside 1.0's `compare-after.txt`):
  - C-08: read → fail, on shape only. The reader gave one by-case over "bike", the 1.0 key's form. Its three cases' values are the key's (1.0's comparison read the case labels and agreed).
  - C-11's `without:` is not marked.
  - Totals: C 5 pass, 3 read, 4 fail → 5 pass, 2 read, 5 fail; E2 unchanged (2 read, 1 fail).
  - Every other line is 1.0's.

### HERR-4 · the visible errata's effects on the hidden set
- **Shared code:**
  - `fam/home.py`, `lib/law.py`, `fam/a.py` (`capital_tax`, ERR-12), `fam/b.py`, `fam/b_paths.py` (ERR-7, ERR-22), `fam/d.py`, `fam/d_paths.py`: no hidden key moved by any of them (`union_e2.py` after each group of fixes: no hidden file changed).
  - ERR-14's sweep for a direction word with 0 found no hidden key; C's `way()` already writes a bare 0.
- **compare.py (ERR-8, ERR-23):** before HERR-3, the hidden reader's comparison under the new `compare.py` reproduced 1.0's `reader/compare-after.txt` byte for byte.
- **Dossier:** of the visible papers the errata changed, the hidden keys cite only D-D04 (E2-01, E2-02, E2-03), whose change is a tense (ERR-9); the stories cited are C's (C-07, C-12) and D's (E2-02), both unchanged. Session 5 rereads nothing hidden for the visible errata; E2's keys didn't move.

## cases 1.0.2 (2026-10-29)

The visible set's errata are ERR-33 to ERR-37 in `tmp/cases/ERRATA.md`. They changed only visible scripts (`keys_a.py`, `keys_b.py`, `keys_e1.py`), which no hidden script imports. E2 reads no key they moved. This tree was swept for the same two patterns. The record is in `plan/errata/PROGRESS-1.0.2.md`.

### HERR-4 · the sweep for ERR-33's and ERR-35's patterns (case stands)
- **(a) A partial whose missing fact is a whole balance, where a reading of it exists before the gap:**
  - none. The hidden holes in a net worth are the house and the car, which C-HR9 .4 says were never valued, so no earlier reading exists.
  - The other holes are refusals of a different kind: a missing account record, and the value the inheritance tax needs.
- **(b) A single-valued key resting on a rounding step the papers don't state:**
  - none. Every rounding step in `keys_c.py`, `keys_e2.py`, `c_tax.py`, `c_sim.py` and `c_facts.py` cites its paper: GEN-5, GEN-4, RGP-2, RWT-3 .2–.3, PIT-16 .3 and PIT-18 .3 (home supplement), C-D27 .2, C-HR3 .1, C-HR6 .2, C-HR7 .2, C-HR9 .1.
  - The one figure not covered by a paper is C-06's "dukats due" for the sale not made on 2026-01-02. It follows Bank Lazur's practice, which the records show:
    - the pot's rows round 6 365.5627 to 6 365.56 and 21 403.4062 to 21 403.41, so the bank rounds to the nearest ort;
    - 10 420.70 × 4.1204 = 42 937.4523 is no tie, so every nearest mode gives 42 937.45.
- **Moved:** none.

## cases 1.2.0 (2026-10-30)

Session 5r's close, from the user's calls at gate G5 (`tmp/session5/g5/items.json`). The visible side is ERR-39, ERR-40, TOOL-3 and ADD-8 in `engine/tests/cases/ERRATA.md`. This tree is now open: households C and E2 are copied into `engine/tests/cases/` and stay here as the exam's record (`README.md`).

### HERR-5 · C-07: the bike is owed back too (G5 K-1; ERR-39)
- **Cause:** the exam's one case fault. C-07's "owed on 2025-12-31" counted only C-HR3's settle-ups; C-HR4 .4 makes the bike payable back under the readings that need a yes, C-08's three.
- **Fix:** `tools/keys_c.py` `q07` (original: `tally-2.0/scratch/regen/keys_c.py.orig`).
- **Moved:** `keys/C/C-07.md`, "owed on 2025-12-31": 672.37 → by-case: CBL mid 3 194.65, Lazur buy 672.37, Lazur sell 3 259.99. No other key (`union_e2.py`: only C-07 changed).

### HERR-6 · The tool fixes of step 2.7, accepted (G5 M-1; TOOL-3)
- `tools/c_law.py`, `tools/gen_c_gazette.py`, `tools/union_e2.py` (3 path lines and a regeneration function) and `INDEX-hidden.md` (by `index.py --hidden`); originals and diffs in `tally-2.0/scratch/regen/`. `VERSION` reads `cases 1.2.0`; `MANIFEST.sha256` is re-recorded over this tree's files.
