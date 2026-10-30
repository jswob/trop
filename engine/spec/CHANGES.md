# Changes: tally 2.0 → tally 2.1

tally 2.1 adds owners and batches as labels on amounts, from session 6's decided items, designed in session 7 and written in session 7b (2026-10-30). It is **opt-in**: a text with no `owned` or `batched` kind, and none of the small items' forms (notes after a one-line def or a place, a negative record field, a day key where a date is expected), means and prints what it did on 2.0, except where it declares the new built-in id `parties` (`MIGRATION.md`, 2.0 → 2.1). The reasons are `DECISIONS.md`'s "tally 2.1 records" (R-1 to R-17, G1, the four picks); the round's files are `tmp/session7/` and `tmp/session7b/`. One row per change:

| # | Where | Change | Record |
|---|---|---|---|
| 2.1-1 | L §2.2 | kind clause words `owned`, `batched`, `takes`, and `prorata` as the clause word ending a `takes` clause; `of` as a construct word; a part's built-in fields `owner`, `batch`; `parties` among the built-in ids | R-1, R-2, R-4 |
| 2.1-2 | L §2.3 | the parts clauses as kind lines (`owned by D [ties by e]`, `batched`, `step u`, `takes k for w [v V] order … [prorata [ties by e]]` or `… prorata [ties by e]`); `of q` on `bal`, `value`, a leg's amount and a reading row; `note` lines after a one-line def and a place section | R-1, R-4, R-6, R-11; picks 2, 3 |
| 2.1-3 | L §2.3 rule 7 | an `of` belongs to the read its production gives it | R-1, C-14; pick 3 |
| 2.1-4 | L §2.4 | a one-line def with notes means the long form and prints as it; no canonical form changes | R-11, C-12 |
| 2.1-5 | L §3.1 | a key of a day index stands where a date is expected, meaning `start_of(k)` | R-14, G1 Q-6 |
| 2.1-6 | L §3.5 | parties are things under the prelude kind `parties`, the subkinds the model's own; only a party owns parts; the holder is a model's own input | R-2, G1 Q-1; pick 1 |
| 2.1-7 | L §3.6 | rewritten: a place owned by shares or by parts, never both; parts, the arrival rule, the leaving rule, relabelling, net worth over `of` reads; the examples use the prelude's `parties` | R-1, R-3, R-4, R-5, R-9, R-10 |
| 2.1-8 | L §5.7 | the parts clauses on place kinds and `LOAD-PARTS`; a kind test narrows; parts and lifespans; `parties` in the prelude | R-1, R-2, R-17, C-10, RC-9 |
| 2.1-9 | L §5.10 | parts as a place's state; `bal … of q`, `value … of q`; `min 0`; a gain follows the units; the parts' cost | R-1, R-7, R-8, G1 Q-3 |
| 2.1-10 | L §5.12, §5.13 | pointers: a place's takes reuse the fill and the largest-remainder split within one outflow | R-4, RC-2, RC-3 |
| 2.1-11 | L §5.14 | legs at a place that keeps parts: the take as a fill over arcs, ties, two outflows never sharing, the batch's arrivals as supplies, undecided takes; whose parts a leg takes is the order's; the relabel; the leg read | R-3, R-4, R-5, R-10, C-1, C-4, RC-3, RC-7 |
| 2.1-12 | L §5.14.1 | the arrival rule as a reader of start-of-day parts beside decisions; the leg read's class | R-3, R-10, M-15, RC-6 |
| 2.1-13 | L §5.14.2 | takes in the first batch, in the order of their reads; a cycle broken where no take changes, else `RUN-CYCLE` with a plan in it, or a hole among observed rows | R-4, C-11, RC-6 |
| 2.1-14 | L §5.14.3 | labels are walk state at cut K; the one exception to decision 22 for a past decision that reads parts | R-3, RC-1, G1 Q-4; pick 4 |
| 2.1-15 | L §5.16, §7.1 | the 4 096-digit cap covers a value at a key, not a path statistic's own accumulation | R-13 |
| 2.1-16 | L §5.19 | readings at a place that keeps parts: stated and opening parts, pads, zero-gap anchors, gaps, a stated part's own gap | R-6, RC-4, RC-5, G1 Q-5; pick 2 |
| 2.1-17 | L §6.1 | `parties` shared by every member; two parties one only by `same`; `same` refused on a place that keeps parts; `of` across a `same` | R-2, C-9 |
| 2.1-18 | L §8.1 | `HOLE-PART`; the hole part and its possible labels; a total stays known | R-8, RC-5 |
| 2.1-19 | L §8.2, §8.5 | `WARN-LABELS`, listed in the register | R-3, M-9, RC-8 |
| 2.1-20 | L §8.3 | `LOAD-PARTS` | R-1, R-6 |
| 2.1-21 | L §8.4 | `RUN-LABEL`; `RUN-NO-RULE`, `RUN-TIE`, `RUN-CYCLE`, `RUN-DOMAIN`, `RUN-PRORATA`, `RUN-LIFESPAN` and `RUN-RECORD` widened for parts | R-1, R-3, R-4, R-6, RC-2, RC-3, RC-9 |
| 2.1-22 | L §10 | what 2.1 can't say: signed parts, `same` on a place that keeps parts, and the rest of the round's pressure list; lots may be a `batched` place | R-1, `pressure.md` |
| 2.1-23 | E §4.2 | parts in the day walk: the first batch's takes, a decision's take at its point, the arrival rule run at booking, lifespans after the day's whole order | R-3, R-4, RC-6, RC-9 |
| 2.1-24 | `DECISIONS.md` | the tally 2.1 records | all |
| 2.1-25 | `MIGRATION.md` | 2.0 → 2.1 | all |
| 2.1-26 | `VERSION`, `SHA256SUMS` | `tally-2.1` | — |
| 2.1-27 | (none) | a negative record field (item 7.2) needs no spec text: the grammar allows it, and the interpreter's refusal is a fault | R-12 |
| 2.1-28 | (none) | a joined household's cost (decision 6) is an engine requirement, `engine/BUILD-NOTES.md` § J | R-15 |

**Fixes after the spec round** (faults and contradictions only, each with a line in the round's `progress.md`):

| Row | Where | What | Found by |
|---|---|---|---|
| 2.1-F1 | L §5.7, `MIGRATION.md` | a field of a subkind read where no kind test narrows the variable stays refused as in 2.0, `LOAD-FIELD`; the spec round had named `LOAD-LEG`, which would change a 2.0 text's output against the opt-in rule | writer 11's F-1 |
| 2.1-F2 | L §5.19, §8.4 | a reading at a day's start that pads and states parts, at a place the day's first batch also moves, is refused (`RUN-RECORD`), the reading at the end of the day before written instead: its pad takes with the batch (E §4.2), so the stated parts had no one point to compare at | writer 11's F-2 |
| 2.1-F3 | E §6.3 | the residual ledger's balance line shows each stated part's gap and the unstated rest's, so "every such gap is listed with the reading" (L §5.19) has a form | writer 11's F-3 |
| 2.1-F4 | L §3.6, §8.4 | a share below 0 is `RUN-LABEL`: a part is never below zero (§10, signed parts) | writer 11's F-4 |
| 2.1-F5 | L §3.6 | in an arrival's split the hole part of hole shares ranks after every party, which `ties by` can't rank ("a known share is never lost to a hole") | writer 11's F-5 |
| 2.1-F6 | L §5.19, §8.3 | a stated part below 0, and two stated parts of one reading that overlap, are `LOAD-PARTS` | writer 11's F-6 |
| 2.1-F7 | L §8.1 | the hole part's questions are worded for a gap, an unstated rest, a cycle of observed rows, an unknown outflow and a stated part's open batches | writer 11's F-7 |
| 2.1-F9 | L §8.1 | the question of a stated batch's part whose owners are open, at a place that keeps pairs | writer 11's F-9 |

---

# Changes: tally 1.0.1 → tally 2.0

tally 2.0 is a new version of the language, not a patch. `LANGUAGE.md` §9 allows a new version only with a migration, a mechanical rewrite of every `tally 1` model, tested by running both versions on every model and case. The migration is `MIGRATION.md`, and its runs are recorded in `regress.md`. Files of the new version state `tally 2`. `tally 1` (1.0.1, tag `tally-1.0.1`) stays as released.

The definition was redesigned in one round, session 4 of the engine plan, from the evidence of tally 1.0.1's case set (session 3). The reasons are in `DECISIONS.md`. How the round was run, what it shows and what it doesn't are in the round's `REPORT.md`. This file says what changed, where the old-to-new table lives, how the contradictions were found and settled, and which gaps stay open.

**New in the package:**
- `LANGUAGE.md`, `ENGINE.md`, `DECISIONS.md`, `coverage.md` and this file, rewritten for tally 2;
- `MIGRATION.md`, the table from tally 1.0.1 to tally 2 (L §9);
- `models/`, `amendments/` and `montecarlo/` as tally 2 texts (carried by `tools/migrate.py` and the migrators);
- `VERSION` (`tally-2.0`) and `SHA256SUMS`.

**What it doesn't change:**
- The expected answers in `expected/`, `amendments/expected/` and `montecarlo/`, except the keys shown wrong. Those wait for the user (§ Open gaps, "Keys") and change only with the user's ok.
- `OPERATIONS.md`, `PERFORMANCE.md`, `BOUNDS.md` and `BUILD-ORDER.md`, which are carried as they are and marked stale, with the sections the decisions reach (`STALE.md`). Nothing in this round checked them.

## What changed

**The model of the world** (`DECISIONS.md` D-1 to D-15).
- Everything a model talks about is a declared thing of a declared kind, with a lifespan.
- Every statement about a thing is a dated fact, with when it holds (`governs`) and when it became known (`known`).
- A movement is a thing of an event kind, with named legs in any unit and declared reads.
- Law, money in places, forecasts and randomness are this one idea.
- Nothing defaults: a missing statement is refused, with a code from L §8.3.

**Concepts:** sixteen become fourteen (`regime`, drafted, was dropped at G2: GD-1).

| Concept | Change | Record |
|---|---|---|
| source, index, axis, band, given | kept (index gains the built-in period indexes; a band row can be replaced) | D-12, D-13 |
| version | changed: `governs` is required (`always` when timeless); `known` per row; an overlapping row says `replace` | D-3, D-12 |
| kind | **added**: lifespans, subkinds, dated memberships of plain kinds | D-1, D-2 |
| place | changed: a thing of a declared place kind (unit, sign rule, valuation); 1.0.1's seven place kinds are in the prelude | D-1 |
| input | changed: states over spans, readings at instants, distributions, codes, calendars | D-3, D-11, D-12, D-13 |
| def | changed: its law date is written (`at e` or `at reader`); exceptions are written pairs (`beats`) | D-4, D-12 |
| fill | kept; also lot matching (L §5.12) | D-2, D-5 |
| move | **added**: replaces `pay` and `record`; observed movements are rows of an `observed` input | D-2, D-5, D-6, D-8 |
| scenario | changed: an overlay of ordinary versions, compared by keyed difference | D-10 |
| ask | changed: a stated law date; values, partial answers, differences, readers, explanations, checks, path statistics | D-4, D-7, D-10, D-15 |
| `pay`, `record` | merged into `move` and `observed` inputs | D-2, D-8 |

**Constructs:**
- **Removed:** `draw`, `bound` and its outcome, `promised`, `free`, `holds`, `plan_low`, `next_on`, `phase`, `excepts`, `arrears` lines and `correction` versions, the `-` atom, the path key `p in path`, L §4.3's default law dates (rules 2–4), parse rule 1, inline figures other than 0, 1, −1 and key offsets, and `weeks`.
- **Merged:** `worth` into `value`; `kind()` and `currency()` into `x in K` and `unit(x)`; scenario versions into ordinary versions; rate bases and day counts into code values.
- **Added:** the full list with each construct's record is in `core.md` § "Removed, merged, added" and in `DECISIONS.md`, with A-1 to A-51 after G1 and the review's verdicts.

**The counts** (`tools/counts.py`):

| Count | 1.0.1 | 2.0 |
|---|---|---|
| Concepts | 16 | 15 |
| Keywords | 47 | 50 |
| Built-in names | 68 | 75 |
| Contextual words | 83 | 126 |
| Built-in ids | 14 | 21 |
| Constructs used, over the carried texts (`coverage.md` table 1) | 173 | 133 |
| Constructs used, over all 196 cases | — | 171 |
| Built-in words (`coverage.md` table 2) | 113 | 120 |
| Refusal and warning codes defined | 66 | 91 |

**The migration** (`MIGRATION.md`). Each changed construct is one row, with its class:
- **automatic:** rewritten by `tools/migrate.py`, with the same answers by construction;
- **warned:** rewritten, with the lines a writer checks listed;
- **manual:** the meaning changed, and the row lists the models and asks it reaches.

Of the 79 models, 67 needed only the script's warned rules and 12 were partly rewritten by hand (`regress.md`). Every change of meaning is listed in `MIGRATION.md` with the texts it reaches.

**The answers kept** (`regress.md`, judged by run on the reference core).
- The 79 models: 2 170 key rows match.
- The amendments household: 258 key cells match. 12 variant cells whose tally 1 key is a hole print otherwise under tally 2's reading (§ Open gaps, "Keys").
- The probes: all 88 keyed rows match.
- The rows that don't match are listed under § Open gaps.

## The gap log

### 1.0.1's gaps, closed

Every item tally 1.0.1 failed or was questioned on is one line of the round's trace (`trace.md`). That is 186 rows of the case set's gap log, the 29 concerns LC-1 to LC-29, and 1.0.1's 30 open gaps (this file's predecessor, § Open gaps). Each line is closed by a decision, deferred, or not a language matter, with its status in tally 2 and where it is shown.
- Concerns: 27 closed, 2 deferred.
- 1.0.1's open gaps: 23 closed, 7 not language matters (OPERATIONS or tool matters).
- Spec findings S:1–S:19: 11 closed, 3 expressed, 2 resolved, 1 awkward, 2 withdrawn.
- Probes: 83 expressed, 9 awkward.
- Questions: 20 expressed, 15 awkward, 1 breaks, 1 can't express, 2 case suspect.
- Observations: 20 expressed, 10 awkward, 3 can't express (two of them tool gaps), 1 case suspect, 2 closed.

### How the contradictions of the new text were found

1. **The review.** One ground-up review read every decision taken after G1 against four written principles (`review/principles.md`) and decided each: derive, admit or drop (`review/ledger.md`).
   - A-rows: 48 derived, 3 dropped.
   - Pressure lines: 23 derived, 11 dropped.
   - Open questions: 8 derived.
   - Word groups: 4 dropped, 1 kept.
   
   A fresh critic then checked the rewritten sections (`review/critic/`).
2. **The writers and judges of the visible cases.** They logged each place where the text couldn't say a row or said two things: the S:2xx and S:3xx rows of `rows/*.settled.md`. Each was settled by the lead in a spec round, or left on the open list.
3. **The reference interpreter's writers.** They logged each place where the spec was silent or contradicted itself, in their hand-overs. A silence stays a `REFCORE-UNSPECIFIED` refusal in the interpreter until the spec settles it.
4. **The machine checks.** Seven judges compared the interpreter's output with the keys on snap8, and four of them rechecked on snap11. Scripted reruns repeated the judged runs on the final snapshot, snap14; REPORT §5.1 names the runs left out. Each difference was triaged: an interpreter fault (fixed by the interpreter's writers, each fix with a test), a text's error, a key, or a contradiction. Spec rounds 17, 17b and 17c (R17-01 to R17-11) settled the contradictions they found. Each edit states what the spec already implied, with no new form.
5. **The contradiction read.** Three readers each read one share of the sections against all of L and E and `DECISIONS.md`, and listed 142 candidates. Three verifiers, one per report, checked every quote and decided each one: 135 confirmed (61 outcome, 69 wording, 5 stale), 7 not confirmed, and 15 that the definition doesn't settle. Round 18 (below) applied 130 edits from 108 confirmed candidates, each in its single home; duplicates were applied once. The 15 unsettled ones are on the open list.

### The settlements after the build

The rule after the interpreter's one rebuild was that only interpreter faults and contradictions are fixed, and a new design question goes to the user. Rounds 17 to 17c settled these:

| Id | Section | What was settled |
|---|---|---|
| R17-01 | L §5.16 | the example's `num` range states its step (§5.11, LOAD-DOMAIN) |
| R17-02 | L §5.19 | a link's existence is judged by its keys, `where` and `when` (E §4.1) |
| R17-03 | L §6.1 | a joint model's unlinked movement into a member's place: every read of that place from its day is a hole naming it, balances included |
| R17-04 | L §6.1 | `same` rows of different kinds, read through a kind holding both, meet the movement once |
| R17-05 | L §6.1 | a member's ask is asked by name in the joint model's run and answered by the member alone |
| R17-06 | L §8.3 | `LOAD-SCENARIO` for an ask whose world names no loaded scenario |
| R17-07 | L §5.8 | `x[*] = v` states the keys that exist at its version's `known` |
| R17-08 | L §2.1, §2.4 | the canonical number examples follow the shortest-exact rule |
| R17-09 | L §2.4 | the canonical order of a regime's and a section's clause lines and notes follows the grammar |
| R17-10 | L §6.1 | the joint model's unlinked movement: the hole and the `left out:` line (R8 item 44, MC-E1 C1) |
| R17-11 | L §4.4 | the built-ins on a window, at its endpoints (R8 item 46) |
| R18 | L, E | 130 edits from the contradiction read's 108 confirmed candidates: the literal rule's one list (§3.2 citing §7.1), `LOAD-REPLACE` and `LOAD-OVERLAP` as §4.6 gives them, refusals at load where load sees them, the sign rule after each decision's legs in E, later `opening` readings, an ask line's own projection on a path, `flows` of a move with its settlements, the law-date class bounded by lifespans, sibling scenarios' ids, the lots example's ties, a lifespan whose end is a hole, `LOAD-KNOWN` and `LOAD-PLACEKIND` in place of parse errors, and the wording and stale citations (`review/contra/verified-{1,2,3}.md`) |
| R19 | L, E | the G2 decisions (`decisions.md` GD-1 to GD-26) in one script, `scratch/lead/round19/` (`regime.py`, then `answers.py`): L §2.1–§2.3 (the month-day literal and the type word `monthday`; `regime` and `export` gone), §2.4 (`kind` sections in tree order; the canonical order's open sites), §3.1 (`monthday`), §4.2–§4.3 (a `by n` key no law date; `day_of(y, md)`; a law-date class no row covers), §4.6 (`after m` against the first batch; a period across two keys), §5.9 and §8.1 (the band hole without a date), §5.11 (a `v` naming no version), §5.14.1–§5.14.3 (`before m` only against a decision; a settlement is what `fulfils` an adjustment), §5.16 (the quantile's standard error; `discrete` over keys), §5.17 (`regime` removed, its parts in §5.10, §5.12, §5.14), §5.19 (an `opening` at S start), §6.1 (no `same`-line exception for law parts), §6.2, §7.1 (month-day literals), §8 (an ask's `LOAD-SCENARIO`), §8.3 (what load sees; `LOAD-ORDER`, `LOAD-NAME`, `LOAD-UNIT`, `LOAD-DOMAIN`, `LOAD-SAME`), §8.4 (which load code); E §1, §2 (S the header's start; `flows` before S a hole; [K, E]), §3.1 (load sees L §8.3's list; the cone through places; no refcount step), §3.2 (a class no row covers), §4.1, §4.2 (step 0's opening at S; `after m` names a decision), §5 step 2, §7, §11; then the round's contradiction read (`review/contra/verified-4.md`: 11 fixes, and C4-9 by the user, GD-27); L §2.4's last key (GD-28) and §5.14.1's `bal(x, d start)` row |

## Open gaps

These waited for the user at G2, each a question the spec didn't settle or a change the rule above didn't allow. The user settled them (`decisions.md` GD-1 to GD-26, round 19); what stays open after the freeze is the figures unjudged until the engine (`benchmarks.md`), SF-34 and SF-35 (deferred to tally 3.0), and the keys and case suspects (`freeze/KEYS.md`, the case set's errata list).

- **Regimes in the visible families (REPORT §5.2).** Households A and B write capital gains and VAT as law parts read by their own defs, not as regimes. The regime form runs only in the blind texts. The user leans toward dropping `regime` from 2.0 (`G2-regime-drop.md`).
- **`Q:A-04`, expressed in 1.0.1, now awkward** (A-49 removed `day_of`'s month-day form), and the 36 rows that stay awkward, each awaiting the user's ok (bar B2; REPORT §4).
- **The canonical form's complete rules (L §2.4).** The interpreter refuses 8 order questions as `REFCORE-UNSPECIFIED`. Pins are checked as bytes until they are settled.
- **Five more open sites:** a period read at a key position it lies across (L §4.6); a band read with no law date whose rows don't hold the argument (L §5.9); `discrete` over a key list (L §5.16); the form of a `same` line naming a law part (L §6.1); how far a law-date class reaches (E §3.2 against L §4.6 step 5).
- **Unjudged until the engine:** the figures that exist only at full scale, unjudged until the engine (`benchmarks.md`): D-10's band figures; B-12's figures at 3 000 paths; the amendments household's 170 long-horizon cells; household-bands at 1 000 paths; B-07 on the household's own span and B's balance rows after 2026-12-31; A-05's full-text `windows each` asks; the rows of LH-15, LH-16 and LH-17-mc beyond their cut-downs. The round's full runs were dropped, since they test the reference interpreter's speed, not the language. Their recipes are the engine build's size targets.
- **What load can see,** as one fixed list.
- **The contradiction read's 15 open questions** (`review/contra/verified-{1,2,3}.md`, each with both meanings quoted). The definition doesn't settle which side holds in these:
  - movements before S and `flows` (C1-9 = C2-10), and S itself against a request's later day (C1-10);
  - a negative n in `add_business` (C1-21);
  - a plain read of a reading away from its instant, a hole or `LOAD-CARRY` (C1-23);
  - `after m` inside the first batch, and `before m` ahead of a plan (C2-3, C2-4);
  - the version items a regime's law needs under `LOAD-PRIVATE` (C2-8, in part);
  - day K past or plan (C2-19);
  - a decision's day reading running state (C2-20);
  - `outside` of unit `one` or `any` (C2-21);
  - `when` in a link's existence (C2-23);
  - which settlements escape measurement (C2-26);
  - a base ask naming an absent scenario (C3-14);
  - `WARN-UNREAD`'s reach (C3-19);
  - the canonical order of subkinds (C1-31).
- **A quantile's standard error** (L §5.16). Without one, a band figure from paths can't be tested against a key.
- **A refusal and the holes under it** (L §8.4). A refusal hides the facts that are also missing (`Q:A-10`).
- **Rows still breaks or can't express:**
  - `Q:A-12`: the rounding axis prints by case;
  - `Q:D-07`, figures 6 and 7: a redemption across two versions of the act;
  - `O:A-3`: a calendar settled except one holiday.
- **Deferred:**
  - LC-7 and LC-8;
  - SF-34, one merged dated list;
  - SF-35, a per-arc cap in `fill`.
- **Keys:**
  - LH-11's three rows under `down`;
  - CMP-01's two `inputs-of` listings;
  - the amendments variants v4 and v5: 12 cells whose tally 1 key is a hole;
  - B-11's known part, against a key that leaves out the whole Wrzos balance;
  - the case suspects `Q:B-05`, `Q:B-12` and `O:B-2`, on the case set's errata list.
- **Maintenance:**
  - a law part shared by several households has up to four copies, which must be kept in step;
  - a joint model's bytes pins must be rerun when a member changes.
- **The size of the language:**
  - the contextual words and refusal codes grew with "everything written";
  - L grew by 73 %.

## Decided in `DONE.md`

Nothing yet. The changes the gates need for tally 2 are proposed in the round's `DONE-proposals.md`. They apply only once the user approves them, logged in `DONE.md`'s table.
