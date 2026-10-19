# Tally: operations

These are the structured edits Claude makes to a reality, and what each one returns. Together they are the MCP surface, without the MCP technology. Each operation gives:
- its inputs;
- its validation;
- its errors.

`LANGUAGE.md` says what a reality means, and `ENGINE.md` how it runs. This document says how a reality changes, and who may change it.

**Contents**
1. The store, and the life of an amendment
2. Add a fact (and import records)
3. Add a law version
4. Amend: add, advance, restate, revise
5. Add a scenario patch
6. End
7. Plan
8. Ask a question
9. The rest of the surface
10. Errors and warnings of the operations

---

## 1. The store, and the life of an amendment

**The store** keeps a reality's files and their history. It is append-only: every accepted amendment is kept with its record time R, its author and its statement (spec §6.4, M-14). The reality as recorded at any R can be rebuilt and run, which answers "what did we think in March?" (N-1222, `ENGINE.md` §2).

**Drafts and confirmation** (spec §6.2, C-10, DR6):
- Claude may draft any amendment. A draft is kept in the store beside the reality, and no reality run reads it.
- Only `confirm`, by the household, applies an amendment to the reality. The importer's exact matches are the one exception (§2.2): they apply under a rule the household confirmed once.
- **Status lives outside the content.** The store records each file's status against the hash of its content:
  - `draft`: written, not checked;
  - `tested`: its gate passes (§3);
  - `confirmed`: the household confirmed it.

  Changing a file's content gives a new hash, which starts as `draft` again.

**Every amendment** is:

```
amendment = { id,
              base:      the reality hash it was written against,
              author:    household | claude | importer,
              statement: the household's words it comes from (required for scenarios, S-09),
              kind:      add | advance | restate | revise,
              changes:   added sections and lines, each with its file; for restate and revise, the edits,
              reason:    required for revise }
```

**Every amendment goes through the same pipeline.** Each operation below adds its own checks at step 2.

| Step | Check | Error |
|---|---|---|
| 1. Base | `base` is the current reality. If not, the store re-applies the changes on the current reality and re-validates; a conflict is reported with both texts | `OP-BASE` |
| 2. Shape | the changes parse as Tally (LANGUAGE §2.3) and name existing targets | `LOAD-PARSE`, `OP-TARGET` |
| 3. Kind | an `add` adds lines only. An `advance` moves the header's `on` forward and changes nothing else. A `restate` changes no result. A `revise` touches no version (§4) | `OP-EDIT`, `OP-ADVANCE`, `OP-RESTATE` |
| 4. Load | the reality with the amendment loads: every check of LANGUAGE §8.3 | `LOAD-*` |
| 5. Gate | every law part the amendment touches passes its gate (§3). A law part is a `part` whose header carries `law` (LANGUAGE §6.1) | `OP-GATE` |
| 6. Plan | the diff of outcomes is computed (§7) and returned with the draft | — |
| 7. Confirm | the household confirms; the amendment is appended with its R | `OP-CONFIRM` |

**Every result is typed** (spec §6.2, N-1906). It is `ok` with its data, or lists of errors, holes and warnings:
- each error has a code (LANGUAGE §8, §10 below) and its location;
- each hole has its question;
- each number has its unit and a source tag.

Results are small, and they have no prose: the MCP layer renders the Polish sentence from the codes and values.

**Names.** Every operation names its targets by stable ids (LANGUAGE §2.5). A name that matches nothing, or more than one thing ("which account?", "which Marek?"), is `OP-TARGET` with the candidates, so the ambiguity becomes a question (A-17, N-1905).

---

## 2. Add a fact (and import records)

### 2.1 Add a fact

The household states something true: a birth, a contract, a price paid, a choice made, a new account.

**Inputs:**

| Input | Required | Note |
|---|---|---|
| `facts` | yes | one or more `(target, keys, value)`, each target an input; a value may be `? "question"` or `-` |
| `source` | yes | an existing source id, or a new source (id, title, location) |
| `known` | yes | the day the household learned it; today if they just did |
| `governs` | no | the span the facts govern; a row may carry its own |
| `locator` | no | per fact: the page, clause or line |
| `assumed` | no | per fact: the reason, when the household gives a figure the source doesn't |
| `add_keys` | no | keys to add to list indexes: a person, a job, a loan |
| `correction` | no | true when the fact fixes a record of something already true (LANGUAGE §4.5) |

**It becomes** one new version: `cite source`, `governs`, `known`, the key additions, then the rows. One statement or one document is one version, so its facts share a source and a date and are diffed as a unit.

**Validation:**
- Every target is an input. A fact aimed at a def is refused (`OP-KIND`): a def is computed, and a stated result is a given, which needs a source that states it (§4, `add` of a given row).
- Values have the input's type, unit and precision (`LOAD-UNIT`, `LOAD-PRECISION`).
- `known` ≤ the model's `on` (`LOAD-KNOWN`). If the household learned it after `on`, an `advance` to that day comes first (§4): its plan shows what time passing did, and the add follows it.
- The rows tie with no other row (`LOAD-OVERLAP`).
- **New keys are checked against closed-world rows.** For each added key, the operation lists the reads that become holes because the default rows don't cover the new key (LANGUAGE §6.2), with their questions. Answering them is the household's next step.

**Returns:** the draft version, the plan (§7), the new holes with their questions, and the warnings.

**Errors:** `OP-TARGET`, `OP-KIND`, `OP-SOURCE` (no source given), and the `LOAD-*` codes above.

### 2.2 Import records

The import pipeline delivers bank rows (spec §6.1). Each row carries:
- its date, amount and currency;
- the account;
- the counterparty and category, as registry ids;
- the source's own row id;
- its record time.

A batch also says which account and period it covers: a statement.

**It becomes** one version per statement:
- `cite` the bank's export, `governs` the statement's period, `known` the day the statement was available, `covers` its accounts;
- one row of the movement input per bank row (LANGUAGE §5.18);
- the statement's closing balances as `stated` rows.

**The engine does the bookkeeping:**
- **Ids.** The account maps to a place, through rows of the built-in input `account_place[a]`. The source row id becomes the record key, so a correction keeps the row's id.
- **Duplicates.** A row whose id is already there with the same content is dropped. With different content, it becomes a `correction` version restating that key.
- **Card holds.** A pending hold has its own key. When it books, the pipeline names the hold it replaces, and the booked row restates the hold's key (N-1207).
- **Matching** (spec §6.1, P-08, P-20, P-23). For each record, the candidate instances are those of pays that:
  1. touch the record's place, in the record's direction;
  2. have their other end at the outside place that the record's counterparty maps to (rows of the built-in input `party_place[c]`), when that end is an outside place;
  3. are dated within `match_days[p]` days of the record;
  4. have an amount within `match_amount[p]` of the record's.

  `match_days` and `match_amount` are built-in inputs per pay. Their rows are data with a source, confirmed once by the household; a pay with no rows gets proposals only. Then:
  - **exact:** one candidate, the same day and the same amount. The link is written in the record's `explains` field, with the provenance "matched exactly by the confirmed rule";
  - **near:** one best candidate within the window and tolerance. It is proposed, never written;
  - **ambiguous:** several candidates. They are proposed together, for the household to choose;
  - **none:** the record stays unexplained.
  - **a new counterparty** (no `party_place` row): the record is unexplained, and the import asks whether the money is still ours (a claim place) or gone (an outside place) (P-15).

  A proposal the household confirms becomes a `correction` version that restates the record with its link: the record was always that instance's.

**Returns:** counts of rows booked, linked exactly, proposed, unexplained, duplicate and corrected; the proposals; the stated-balance check for each covered account (`ENGINE.md` §6.3).

**Errors:** `OP-ACCOUNT` (an account with no place), `OP-CURRENCY` (a row in another currency than its place), `LOAD-COVERS`, and the load codes.

---

## 3. Add a law version

A law changes: a new act, a new year's figures, a court's reading, a retroactive amendment.

**Inputs:**

| Input | Required | Note |
|---|---|---|
| `part` | yes | the law part (file) it belongs to: a `part` whose header carries `law`; a new act is a new law part |
| `source` | yes | the act and its location: Dz.U. position, URL, paragraph |
| `governs` | yes | the span it applies to (`applies` in the spec) |
| `known` | yes | the day it was published or enacted (`known_from`, L-24) |
| `enters` | when it differs from the default | the day it enters into force, where that isn't the later of `known` and the start of `governs` (LANGUAGE §5.3) |
| `arrears` | when it reaches back | for each pay it changes after booking: `on D` or `none`; in a later version, `on D of V` or `none of V` answers back pay an earlier version V left open (LANGUAGE §5.13.3) |
| `moves` | when it postpones or extends an earlier version | `V to SPAN` (LANGUAGE §5.3) |
| `assumed` | when the version stands in for a text it doesn't cite | the reason, naming the text (LANGUAGE §4.5, §5.3) |
| `rows` | no | the changed figures, each with its locator |
| `bands` | no | each band that changes, as a whole table (LANGUAGE §5.8) |
| `rules` | no | new rule lines, with `v` set to this version: the same label to replace a formula from this version on, or a new label with `excepts` for a new exception |

**It becomes** one version in the law part, with any rule lines added to their defs.

**Validation:**
- All load checks. In particular, every band table is complete (`LOAD-BAND`), and each rule's label and `excepts` exist (`LOAD-EXCEPTS`).
- **Back pay is listed.** When `governs` starts before `known`, the operation lists every booked pay instance the version changes, with its δ, under each `arrears` line. Instances that no line covers will be `HOLE-CHANGED` holes, until a later version answers them with `arrears … of` this version. They are returned as `WARN-BACKPAY`, so the author sees them before confirming.
- **The gate** (DR7, C-12, C-24, L-30). A law part is read by the reality only once it is `tested`. The gate runs these steps in order, and stops at the first that fails:
  1. **Load.** The part loads with the reality (every check of LANGUAGE §8.3).
  2. **Sources.** Every row and every rule with a figure points at a version with a source and a locator. Rows can't exist without a version, so this checks only the locators.
  3. **Checks.** Every `check` ask in the part passes. Two kinds are kept (L-32):
     - **independent checks:** the law's own published examples, payslips and real returns (L-35), collected by someone who isn't the author (L-31). The author doesn't see them before the gate runs (L-33). They are the cross-check (L-34);
     - **the author's own checks**, which add coverage but aren't evidence.
  4. **Coverage.** Every rule of every def in the part, and every pay's amount, is reached by at least one check, and every rule wins in at least one check's `explain`. A rule no check reaches is listed, and the gate fails. Steps 4 to 6 skip rules whose value is `?`: a hole rule asserts nothing, so a part of stubs alone (a law not yet written, LANGUAGE §6.5 change 5) is `tested` at once, and every result that reaches it is a hole with its question.
  5. **Planted errors** (mutation tests). The engine makes copies of the part, each with one planted error: a comparison flipped, `<` made `≤`, a rounding mode or unit changed (L-37), a band edge moved by one unit, an `excepts` dropped, a rule's `v` moved to the previous version. At least 80% of the copies must fail a check. The surviving ones are listed, since each shows a part of the law that nothing tests.
  6. **Threshold table** (C-24, L-38). Every band edge and every `when` comparison on an input is evaluated just below, at and just above the edge, over the part's checks' inputs. The table is printed for review. It holds no expected answers; it shows where the law jumps.
  7. **Status.** The part is `tested`, recorded against its hash (§1).

  **A published figure that disagrees with its own law** (L-41) is a check that expects the law's figure. Its row cites the errata or a second source, and its note keeps the published figure, so the gate prints both.

  **Who writes law doesn't matter** (L-40): Claude's parts, the household's and the importer's pass the same gate. A failing gate refuses the version in the reality (`OP-GATE`). The draft may still be used in a scenario, with `WARN-DRAFT` on every result that reaches it (C-09).

**Returns:** the draft version, the gate's results and the threshold table, the back pay list, and the plan (§7).

**Errors:** `OP-GATE`, `LOAD-ARREARS`, `LOAD-BAND`, `LOAD-EXCEPTS`, `LOAD-MOVES`, `LOAD-OVERLAP`, `LOAD-KNOWN`.

---

## 4. Amend: add, advance, restate, revise

The general change, for anything the specific operations don't cover. Examples: a new place, a new pay, a new def, a new ask, a new rule, a given, a fix of the model's own wiring.

**Inputs:** `base`, `kind`, `changes` (as in §1), `reason` (required for `revise`).

**The four kinds** (LANGUAGE §6.2):

| Kind | What it may change | Validation |
|---|---|---|
| `add` | adds sections and lines, anywhere. Removes and edits nothing | the text diff against the base has only added lines (`OP-EDIT`) |
| `advance` | the header's `on`, forward, and nothing else: time passing. It changes results, since `on` is the default cut and the plan's start | the new `on` is later than the old (`OP-ADVANCE`), and no other line changes (`OP-EDIT`); the plan is shown: what moved from plan to past, and what the versions known by the new `on` change |
| `restate` | edits that change no result: a note, a locator, a source's title, moving items between files, reordering | the reality is run before and after over every saved ask and the whole timeline. Every result and every movement must be equal (`OP-RESTATE`) |
| `revise` | edits of defs, pays, places, fills, draws and asks: the model was wrong, not the world | it touches no version (`OP-EDIT`); `reason` is given; every law part it touches passes its gate again; the plan is shown |

- A version is never edited by `revise`. A wrong fact is a `correction`. A changed law is a new version.
- A revised law def returns to `draft` until its gate passes.

**Given rows.** A row aimed at a def (LANGUAGE §5.10) is added with `add`, inside a version whose source states that result. Example: a court's figure, or a tax office's decision. The operation refuses a given whose source is the household's own assumption (`OP-GIVEN`). A guessed result is an `assumed` input row, not a given.

**Returns:** the draft, the plan (§7) and, for `restate`, the proof that nothing changed. An `advance` is the store's to make: it follows the calendar, or an add whose `known` is after `on`.

**Errors:** `OP-EDIT`, `OP-ADVANCE`, `OP-RESTATE`, `OP-GIVEN`, `OP-GATE`, and the load codes.

---

## 5. Add a scenario patch

The household asks "what if …" (S-01 to S-12, LANGUAGE §6.4).

**Inputs:**

| Input | Required | Note |
|---|---|---|
| `statement` | yes | the household's words. Claude never proposes a scenario (S-09, A-19, spec §4#34) |
| `scenario` | yes | a new id, or an existing scenario to extend |
| `from` | for a new scenario | the day from which it changes anything |
| `over` | no | the scenario it stacks on (S-02) |
| `changes` | yes | versions of rows (no `known`), key additions, and new sections (places, inputs, defs, pays, asks) |

**It becomes** the scenario's file, or an `add` to it.

**Validation:**
- `statement` is present (`OP-STATEMENT`).
- **The scenario rules hold** (`LOAD-SCENARIO`):
  - no row starts before `from`;
  - no version has `known`, `arrears` or `correction`;
  - the stack is at most 4 deep;
  - nothing in the reality reads the scenario's items.
- A law part that is only `draft` may be used, with `WARN-DRAFT`.
- **Linked overrides** (S-07) need no preset. A scenario that pins the price index changes whatever the model derives from it: wages, rents, limits. The plan lists what moved.
- **Valid but wrong** (S-08): the model's own `check` asks run in the scenario, so a scenario that buys the flat while the rent goes on fails the household's check "one housing cost at a time". The failure is returned as a warning on the scenario.

**Scenario life:**

| Operation | What it does |
|---|---|
| `scenario.patch` | the above |
| `scenario.stale` | lists `WARN-STALE`: where the reality has changed under what the scenario overrides (S-05) |
| `scenario.promote` | turns the scenario's versions into one reality amendment, with `known` = today and the household's statement. It goes through the pipeline and needs `confirm`. The scenario file stays, or is dropped |
| `scenario.drop` | removes the scenario file. Scenarios aren't reality, so this is allowed; the store keeps the history |

Rebasing is automatic: a scenario is always read over the current reality (LANGUAGE §6.4). A target can't vanish under it, because ids and keys are never removed. A thing that ended has rows that say so.

**Returns:** the scenario's file, the plan against its base (§7), and the warnings.

**Errors:** `OP-STATEMENT`, `LOAD-SCENARIO`, and the load codes.

---

## 6. End

Something stops: a job ends, a loan is repaid early, a rent contract ends, an account is closed, a person dies, an act is repealed.

**Inputs:** `target` (a key of a list index, or an act's rule label), `end` (the first day it no longer holds), `source`, `known`.

**It becomes** one version whose rows make the thing 0 or `-` from `end`.
- The operation finds every input keyed by the target with a row open past `end`.
- It proposes the ending row for each: 0 for money and counts, `-` for everything else, `false` for a membership or a flag.
- Claude or the household confirms each proposal, or changes it. For example, a loan's terms stay, and its balance ends by its own schedule or by a final overpayment row.
- **For a repeal,** the operation proposes a rule with the same label, `v` set to the repealing version and value 0 or `-`, and asks about back pay as in §3.

**Validation:**
- After the version, every pay touching the target books nothing from `end`, and every place it owns can reach 0. Otherwise the operation lists what still runs (`WARN-ENDED`): the "loan that kept paying after its last instalment" (spec §3.9).
- **A pay that data can't stop is refused** (`OP-END`): its amount and `when` read no input keyed by the target. The fix is a `revise` that gives the pay its data, such as a `when` on the target's membership. Then the end is an `add`.

**Returns:** the draft version, the proposals, the plan.

**Errors:** `OP-END`, `OP-TARGET`, and the load codes.

---

## 7. Plan

"What does this change?" Given a draft amendment or a scenario, `plan` gives the diff of outcomes (N-1802, the brief's "plan").

**Inputs:** `amendment` (a draft id) or `scenario`; `asks` (default: every saved ask); `paths` (optional, for bands).

**Output**, sorted by size within each part:
1. **Movements:** added, removed and changed, by pay instance, with the first day of difference.
2. **Balances:** per place, the first day that differs, the largest difference, and the difference at E.
3. **Ask results:** before and after. This includes holes and refusals that appear or disappear, with their questions.
4. **The register:** new assumptions, givens, held figures, readings and bounds.
5. **Warnings:** new and cleared.
6. **Sanity:** the first day anything differs. It must not be before the amendment's earliest `governs` or the scenario's `from` (for a scenario, `ENGINE.md` §9 checks it).

**Validation:** the same as the amendment's own pipeline, steps 1–5.

**Cost:** the cone of the change only (`PERFORMANCE.md` §4). Bands run after the deterministic answer, or come from the cache (spec §7).

**Errors:** those of the amendment.

---

## 8. Ask a question

**Inputs:**
- an `ask`: the text of an ask (LANGUAGE §5.16), or a saved ask's id;
- `scenario`, `as_known`, `paths`, `seed`: optional, the same as the ask's own options.

**Validation:**
- The ask parses and loads against the reality: `LOAD-ASKONLY`, `LOAD-FORKS`, `LOAD-ORIENT`, `LOAD-BOUND`.
- **It can't move money.** Its grammar has no pay, so this is unrepresentable (C-19, F27).
- An ask isn't saved unless `save` is given. Then it is an `add` to an `asks/` file.

**Returns:** the printed result of LANGUAGE §5.16:
- the values per binding and per reading, with their units;
- the holes, with their questions;
- the refusals, with their codes;
- the register;
- the warnings;
- for paths, the statistics with their standard errors.

Every number in Claude's answer comes from here (Q-04).

**Saved analyses** (S-10): `analysis.save` stores an ask file, the scenarios it uses and the result's hash. It reopens by reading the stored result, or by re-running it, which gives the same result.

**Errors:** `REQ-ASK`, `REQ-CLOCK`, `REQ-PATHS` (`ENGINE.md` §11), and the load codes.

---

## 9. The rest of the surface

The other tools of A-21 and A-23 are thin wrappers over the operations above and the engine:

| Tool | Is |
|---|---|
| `snapshot` | the current reality's hash, its files and their statuses, the counts of items, the open holes with their questions |
| `catalog` | `LANGUAGE.md` and `ENGINE.md`, the whole contract (A-01) |
| `describe(id)` | the item's declaration, its rows by version with sources, its rules with their versions, its questions, and what reads it |
| `record` | §2.1 and §2.2 |
| `confirm(id)` | step 7 of the pipeline; the household only |
| `scenario.patch` | §5 |
| `run` | `ENGINE.md` §1 over the saved asks |
| `query` | §8 |
| `explain(ref)` | LANGUAGE §5.16, `ENGINE.md` §10, for a cell, a movement, a balance or a path value |
| `history` | the store's amendments by record time, with authors and statements |
| `reconcile(from, to, by)` | the ask `residuals x from..to by f` for each covered place (`ENGINE.md` §6.3) |
| `drill(line)` | one ledger line: the records and the instances side by side, matched pairs and leftovers, with dates |
| `trial(patch)` | §7 on a draft, with the residuals before and after: mismatches fixed and mismatches created |

---

## 10. Errors and warnings of the operations

These are added to the language's catalogue (LANGUAGE §8). Every error names the amendment, the file and the line.

| Code | Message | What Claude does |
|---|---|---|
| `OP-BASE` | `written against <hash>, the reality is now <hash>; conflict at <item>` | re-reads the item and rewrites the change |
| `OP-EDIT` | `an add changes line <n>`; `a revise touches version <v>` | rewrites it as an addition, or as a correction version |
| `OP-RESTATE` | `restate changes <results>` | makes it a `revise` with a reason, or an `add` |
| `OP-ADVANCE` | `advance moves on from <a> to <b>, not forward` | states a later day |
| `OP-TARGET` | `<name> matches nothing` or `matches <a>, <b>` | asks the household which |
| `OP-KIND` | `<target> is a def; a fact is aimed at an input` | adds the input, or a given with its source |
| `OP-SOURCE` | `a fact without a source` | asks where it comes from |
| `OP-GIVEN` | `a given from an assumption` | writes an `assumed` input row instead |
| `OP-GATE` | `law part <p>: check <q> fails: <expected> vs <got>` | fixes the law, or uses it only in a scenario as a draft |
| `OP-STATEMENT` | `a scenario without the household's statement` | doesn't create it |
| `OP-CONFIRM` | `confirm by <author>` | asks the household |
| `OP-END` | `<pay> can't be stopped by data` | revises the pay to read the target's data, then ends it |
| `OP-ACCOUNT` | `account <a> has no place` | adds an `account_place` row |
| `OP-CURRENCY` | `row <r> is in <cur>, its place in <cur'>` | fixes the mapping |

| Warning | Means |
|---|---|
| `WARN-BACKPAY` | a law version changes booked pays; the list says which lines cover them and which become holes (§3) |
| `WARN-DRAFT` | a result reaches a law part that isn't `tested` (scenarios only, §3) |
| `WARN-ENDED` | after an end, something keyed by the target still moves money (§6) |
