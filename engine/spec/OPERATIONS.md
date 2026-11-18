# Tally: operations

How a caller uses the engine: its three operations, how each thing a caller does is written with them, what they leave to the store and what waits. `LANGUAGE.md` says what a text means and `ENGINE.md` how a run computes it; this document adds no meaning and no code.

**Contents**
1. The surface
2. The operations: `run`, `reprint`, `check`
3. How each interaction is carried out
4. What the surface leaves to the store
5. What waits
6. On paper: the seven changes and the kinds of card

---

## 1. The surface

- **Three parties.** The **caller** (an LLM, or the app's code) writes Tally text and asks. The **engine** checks and answers. The **store** (the app's) keeps files, history, drafts, statuses and users; the engine doesn't know it exists.
- **The engine is a pure function.** Files and a request in, printed text out. It reads no clock, keeps no state between calls and writes nothing (`ENGINE.md` §1, §11).
- **A reality is a set of files** (`LANGUAGE.md` §6.1); **a change is a set of added files or lines** (§6.2, §6.3). The caller writes both as Tally text, and load checks them.
- **A question is an ask** (`LANGUAGE.md` §5.18). One the reality doesn't hold is written in a file handed over with the reality's. Every number a caller reports comes from a printed result.

---

## 2. The operations

Each operation is a command of the engine's binary and a call of its library (`engine/tests/INTERFACE.md` §1 to §4). None adds a code: each prints only those of `LANGUAGE.md` §8 and `ENGINE.md` §11.

**For all three.** A path is a file, or a directory: every `*.tally` file under it. A joint model's members are found from its `use` lines, relative to its file, each checked against its pin (`LANGUAGE.md` §6.1). The files are taken in full-path order, by code point, whatever order they are given in (`ENGINE.md` §3.1 step 1). Standard input and the environment are never read; everything prints to standard output. Exit 0 is an answer, 1 a refusal (the first line starts `refused: `), 2 a usage error.

### 2.1 `run`

`tally run <path>… [--ask <id>…]`

- **Inputs.** The reality's files; after `--ask`, the ids of the asks to answer, a joint model's member's as `a.q`; with no `--ask`, every ask. `--pin=bytes` makes a `use` pin the sha256 of the member's file bytes, not of its canonical text.
- **Result.** Each ask's printed result in `LANGUAGE.md` §5.18's form: values per binding and per reading, holes with their questions, per-value refusals with their codes, and under each result its register (`LANGUAGE.md` §8.1, §8.4, §8.2). The run's warnings print sorted by code, then by text (§8.5). Holes and per-value refusals are answers: exit 0.
- **Refuses**, each as the one line, exit 1:
  - a reality that doesn't load: `refused: LOAD-… <message>`, the first in full-path order (`LANGUAGE.md` §8.3); a member's as `refused: member <a>: …`;
  - after load: `REQ-ASK no ask <q>`, the first unknown id in request order; `REQ-CLOCK`, a cut an ask states after `on`; `REQ-PATHS`, more than 100 000 paths for one ask (`ENGINE.md` §11).
- **A scenario file that doesn't load** refuses that scenario and those over it; the base still answers, and each ask in such a world prints `scenario <s>: refused: …`, exit 0 (`LANGUAGE.md` §6.4).
- **`ENGINE-INVARIANT`**, an engine fault, refuses every result that rests on the broken walk (`ENGINE.md` §9).

### 2.2 `reprint`

`tally reprint <path>…`

- **Inputs.** As `run`; no asks.
- **Result.** For each file in full-path order, `--- <path>` (a file given alone as given, a directory's files relative to it) and its canonical text (`LANGUAGE.md` §2.4). Last, `sha256 <hex>`, the reality's hash over those files (`ENGINE.md` §3.1 step 1): what a `use` line pins.
- **Refuses** what `run`'s reading of the files refuses: a path that doesn't exist (`LOAD-PATH`), bytes that aren't UTF-8 or text that doesn't parse (`LOAD-PARSE`). A text that parses but doesn't load still reprints.

### 2.3 `check`

`tally check <path>…`, specified here and built at M3 (`BUILD-ORDER.md` § M3).

- **Inputs.** As `run` (the same paths, order and members with their pins); no asks.
- **Does** everything load does (`ENGINE.md` §3.1; `LANGUAGE.md` §8.3), every ask's own load checks included, and stops: it evaluates and walks nothing.
- **Result.** `ok` on the first line when the base and every scenario file given load, then the warnings load decides (`WARN-UNREAD`) in `LANGUAGE.md` §8.5's order. Exit 0.
- **Refuses.** A base that doesn't load: its one `refused: CODE message` line, as `run` prints it. A base that loads beside a scenario file that doesn't: `scenario <s>: refused: …` for each such file in full-path order, then the warnings. Exit 1 when the first line isn't `ok`.
- **It adds no code:** every line it prints is one `run` can print.
- **`ok` isn't a clean run.** `REQ-ASK`, `REQ-CLOCK`, holes and per-value refusals come only from `run`.

---

## 3. How each interaction is carried out

Each recipe is a few lines of Tally text or one command; the ids are the design map's (issue #7). Most examples run on one small model: a job `acme`, an account `konto`, an outside place `world`.

### 3.1 Asking

- **ASK-2, a question not saved.** Write the ask line in a new file, and name its id in the run:
  ```
  part question
    tally 2

  ask q_oct: bal(konto, 2026-10-31)
  ```
  `tally run <the reality's files> question.tally --ask q_oct`. Its id must be new (`LOAD-DUP`). A line that doesn't load refuses the whole run, so `check` it first.
- **ASK-3, a past view, a what-if, paths, a fixed figure.** The ask's clauses (`LANGUAGE.md` §5.18): `as known D`, `in s`, `paths N seed S`, `with x = v`. `ask q_then: bal(konto, 2026-12-31) as known 2026-10-05`.
- **ASK-4, the same pins on every ask.** Write the `with` clause on each line: `ask q_raise: bal(konto, 2026-12-31) with salary[acme, 2026-12] = 9 000.00 PLN`. The register prints each pin (`pinned:`).
- **ASK-5, explain a number.** `ask q_why: explain bal(konto, 2026-10-31)`; `explain ref` for a cell (`ENGINE.md` §10).
- **ASK-6, describe an item.** The text itself, canonical from `reprint`; `readers-of x` lists what reads it, `inputs-of ref` the rows a cell read.
- **ASK-7, compare the model with the bank.** `ask q_resid: residuals konto 2026-10` prints the residual ledger by month and move (`ENGINE.md` §6.3); `explain bal(konto, 2026-10-31)` lists every movement since the anchor.
- **ASK-10, save a question.** Keep the ask line in a file of the reality, `asks/<topic>.tally` (`LANGUAGE.md` §6.1).
- **ASK-12, fit a parameter; replay one path.** Pin the parameter with `with`, run, read the residuals, once per trial value (`ENGINE.md` §6.4); `path p` replays one path (`LANGUAGE.md` §5.16).

### 3.2 Names and members

- **TXT-3, which thing was meant.** Write the name. A wrong or ambiguous one is refused at load, naming it: `refused: LOAD-KEY kotno is not a key of places`, `LOAD-NAME unknown id <x>`, `LOAD-KEY ambiguous key <key>`. The caller asks the person which one was meant.
- **TXT-6, pin a joint model's member.** `tally reprint <the member's files>`, and put its last line's hash in the joint model's header:
  ```
  use a = "../base/reality.tally" sha256 "a08b16d3…"
  ```
  The member's items are then read as `a.konto`, its asks asked as `a.q`.

### 3.3 Changes

- **CHG-1, add a fact.** A version with its rows (`LANGUAGE.md` §5.3, §6.5 change 6), in a new file or at the end of one:
  ```
  version house_overpay_2028_05
    cite stmt_2028_05
    governs 2028-05
    known 2028-05-20
    overpay[house, 2028-05] = (amount: 10 000.00 PLN, choice: shorter) "nadpłata 10 000 zł, skrócenie okresu"
  ```
- **CHG-2, add a law version.** A version with `replace` rows where a figure changes, and `replace rule d.r` where a formula does (`LANGUAGE.md` §6.2, §6.5 change 2).
- **CHG-3, the back pay a change opens.** Run after a version that reaches back. Each booked instance it changed prints as a hole, `?"wage[2026-10, acme] on 2026-10-10 changed after it was paid (version raise_oct); is back pay due, and when?"`, with the register line `adjustment: wage[2026-10, acme] by 500.00 PLN on 2026-11-04 (version raise_oct), remaining 500.00 PLN`. The answer is a move that fulfils the adjustment, or one with no legs that waives it (`LANGUAGE.md` §6.5 change 3).
- **CHG-4, what a new thing leaves unanswered.** Run after the entry. The reads at the new keys print as holes with their questions (`HOLE-UNGOVERNED`) or as `RUN-NO-RULE`; a closed-world row is restated with `replace` (`LANGUAGE.md` §6.2).
- **CHG-5, end something.** Its lifespan, `replace jobs + [acme] lives 2020-02-01..2026-11-30`, and rows `none` where a fact stops. A leg that still moves into or out of it is refused (`RUN-LIFESPAN`).
- **CHG-6, add structure.** New sections (kinds, places, inputs, defs, moves) in a new file (`LANGUAGE.md` §6.5 change 5).
- **CHG-7, move `on` forward.** Rewrite the header's `on` line. A version known after `on` is refused until it moves: `refused: LOAD-KNOWN version statement_2026_10 is known after the model's on date`.
- **CHG-11, what does this change?** Two runs, without and with the added files, the same asks; the caller compares the printed lines. Nothing prints a run's movements, so the first day anything differs comes from asks of each place's balance by day. Within one reality, `diff(e) in s against b` compares a scenario with its base.

### 3.4 Scenarios

- **SCN-1, write a what-if and compare it.** A scenario file (`LANGUAGE.md` §6.4), asked in its world or against its base:
  ```
  scenario acme_stops over mini
    tally 2
    from 2026-12-01

  version acme_stops_pay
    cite s
    governs 2026-12-01..
    known 2026-10-19
    salary[acme] = 0 PLN "założenie scenariusza"

  ask q_diff: diff(bal(konto, 2026-12-31)) in acme_stops against mini
  ```
  It prints `changed bal(konto, 2026-12-31): 15 000.00 PLN against 22 000.00 PLN`.
- **SCN-3, make a what-if real.** Copy its versions into a file of the reality with `known` the day it became real (`on` moved there if needed), and drop the scenario file or rename the copies (one version id in both is `LOAD-DUP`). An expression row is refused outside a scenario (`LOAD-SCENARIO`): write its value, read from a run in the scenario. A later `known` gives adjustments the scenario didn't (`LANGUAGE.md` §6.4).

### 3.5 Bank statements

- **IMP-1, a bank export as rows.** One version per statement: `covers` its place, `anchor` or `pad`, one observed row per bank row, keyed by the bank's own row id, and the closing balance as a `stated` reading (`LANGUAGE.md` §5.19):
  ```
  version statement_2026_10
    cite bank_2026_10
    governs 2026-10
    known 2026-11-02
    covers konto
    anchor konto
    bank[r1] = (on: 2026-10-10, kind: wages, leg cash from world to konto amount 7 000.00 PLN, fulfils: wage[2026-10, acme]) "line 1"
    bank[r2] = (on: 2026-10-12, kind: transfers, leg cash from konto to world amount 120.00 PLN) "line 2"
    stated[konto] = 7 900.00 PLN at 2026-10-31 end "saldo końcowe"
  ```
- **IMP-2, duplicates, corrections, card holds.** The row's key is the bank's id, so a row already written isn't written again. A correction restates the key in a later version, `replace bank[r1] = (…)`; a row that never happened, or a hold that never booked, is `replace bank[r2] = none`; a hold that books restates the hold's key. The correcting version states `covers` the place too, so the row is booked there.
- **IMP-3, a statement's closing balance.** The `stated` reading. Every balance read after it prints `anchor: konto 2026-10-31 end, stated 7 900.00 PLN, difference 20.00 PLN`; `residuals` prints the same gap.
- **IMP-4, match bank rows to planned movements.** The caller reads `residuals` (`expected but missing wage[2026-10, acme]`, `unexplained bank[r1]`) and the register's `open:` lines, and writes `fulfils` on the row: in the statement, or later by a `replace` of the row (IMP-2). The engine checks each link at the ask's cut (`RUN-RECORD`) and proposes none.

### 3.6 Law

- **LAW-1, run a law file's checks.** `check e == f` asks in the law part, each side a def or a row with its source, never a bare figure (`LOAD-LITERAL`):
  ```
  ask pit_check: check pit[acme, 2026-11] == pit_expected law 2026-11-01
  ```
  `run` answers them like any ask: `ok`, or `differs: e = …, f = …` with the difference (`LANGUAGE.md` §5.18).

---

## 4. What the surface leaves to the store

Each needs a store, a user or a clock, so it is the app's. They are named so the app's builder knows what the surface leaves; none is specified here.

| Id | The store's |
|---|---|
| ASK-11 | saved analyses |
| TXT-4 | a snapshot: the hash is `reprint`'s, the open holes `run`'s, the files and statuses the store's |
| CHG-8 | a check that a change only adds |
| CHG-9 | a restatement, proved by two runs that print the same |
| CHG-10 | a revision, with its reason |
| SCN-2 | whether the base moved under a what-if (`WARN-STALE`, `LANGUAGE.md` §8.5): a run has no file times |
| SCN-4 | dropping a what-if |
| LAW-3 | keeping untested law out of real answers, by statuses kept against `reprint`'s hash |
| STO-1 | the files and their history; record time R (`ENGINE.md` §2) |
| STO-2 | drafts, statuses, confirmation |
| STO-3 | who wrote a change, from whose words |
| STO-4 | two changes written against one base |

---

## 5. What waits

Real, and not in the first version; each with what a caller does meanwhile.

| Id | What waits | Meanwhile |
|---|---|---|
| ASK-8 | the whole timeline as data | asks over `bal` and `flows` |
| ASK-9 | answers as typed fields | the printed form, fixed to the byte |
| TXT-5 | the reference card | `LANGUAGE.md` is the contract |
| TXT-7 | migration to a newer tally version | `MIGRATION.md`, applied by a writer |
| LAW-2 | a gate on a law part: coverage, planted errors, the threshold table | the law's checks are run (LAW-1) |
| STO-5 | results kept under their hash | every call computes; a caller may keep a printed result under the reality's hash |
| STO-6 | the engine kept warm between calls | every call loads |
| VER-1 | which engine this is: the tally version and the build (`LANGUAGE.md` §9) | the store knows which binary it runs; a file that states another tally version is refused at load |

---

## 6. On paper

### 6.1 The seven changes (`amendments/changes/`)

Each change file holds its diff, and each is carried out in three steps:
1. Add the change's files, and rewrite `on` to the change's `known` date where it is later.
2. `check` the reality with them: `ok`, or the refusal to fix.
3. `run` the household's asks without and with the change, and compare the printed lines.

| # | Added | What is read off the run with it |
|---|---|---|
| 1 | `facts/2027.tally`: client C's entry, fee and terms; `on` 2027-01-20 | the asks' changed values; no new hole, since client C's rows are stated |
| 2 | two law versions in `law/pl/common.tally`, one known before `on` (a late entry); `on` 2026-12-18 | the values that read the 2027 minimum and projected wage |
| 3 | in `law/pl/business.tally`: the retroactive health version, an input and the move that waives its adjustments; `on` 2027-06-15 | the health payments the version changed, and no `HOLE-CHANGED`: the waiver fulfils their adjustments |
| 4 | `facts/2028.tally`: Ola's birth; later the 800+ claim with the move that settles its back pay; `on` 2028-05-14, then 2028-06-02 | Ola's keys in every ask over persons; the May benefit's back pay, settled by the added move |
| 5 | `abroad.tally` (a kind, places, inputs, moves), the stub law part `law/de/employment.tally`, `facts/2029.tally`; `on` 2029-05-01 | the holes with the stub's question, at every result that needs the German law |
| 6 | `facts/2027.tally`: the car loan's version, later the overpayment's; `on` 2027-04-02, then 2028-01-08 | the loan's instalments and its end; the shorter term |
| 7 | `scenarios/wife_stops.tally`; `on` stays | each ask `in wife_stops`, or `diff(e) in wife_stops against tropek`; nothing before 2027-01-15 differs |

### 6.2 The kinds of visible card (`engine/tests/cases/README.md`)

| Card | Added | Called | Read off |
|---|---|---|---|
| a value | the card's ask in an ask file | `check`, then `run --ask <q>` | the value line, with its unit |
| a partial | the ask, marked `partial` | `run --ask <q>` | the known part and each named hole with its coefficient (`LANGUAGE.md` §5.18) |
| a refusal | the ask | `run --ask <q>` | the hole's questions, or `refused: CODE message` at the value |
| by-case | the ask; the reading as an axis (`LANGUAGE.md` §5.5) | `run --ask <q>` | one result per option, each with its `reading:` line |
| a relation | the asks it relates, or a `check e == f` ask | `run`, or two runs for a metamorphic relation | `ok` or `differs:`; or the two printed results compared |
| a random figure | the ask with its query and `paths N seed S` (`LANGUAGE.md` §5.16) | `run --ask <q>` | the statistic and its standard error |
| a joint household | a joint model, each member pinned by `reprint`'s hash | `reprint` per member, `check`, `run --ask a.q …` | each member's answers as `a.q`, the joint model's own asks |
| a writing passage | the passage, written as Tally text | `check`, then `reprint` | `ok` and the canonical text; it has no key |
