# Tally: operations

How a caller uses the engine: its three operations, how each thing a caller does is written with them, what they leave to the store and what waits. `LANGUAGE.md` says what a text means and `ENGINE.md` how a run computes it; this document adds no meaning and no code.

**Contents**
1. The surface
2. The operations: `run`, `reprint`, `check`
3. How each interaction is carried out
4. What the surface leaves to the store
5. What waits
6. On paper: the seven changes and the kinds of card
7. The small model the examples run on

---

## 1. The surface

- **Three parties.** The **caller** (an LLM, or the app's code) writes Tally text and asks; the **engine** checks and answers; the **store** (the app's) keeps files, history, drafts, statuses and users, unknown to the engine.
- **The engine is a pure function:** files and a request in, printed text out; no clock, no state between calls, no writes (`ENGINE.md` §1, §11).
- **A reality is a set of files** (`LANGUAGE.md` §6.1), **a change a set of added files or lines** (§6.2, §6.3), **a question an ask** (§5.18), in a file handed over with the reality's when the reality doesn't hold it. Every number a caller reports comes from a printed result.

---

## 2. The operations

Each operation is a command of the engine's binary and a call of its library (`engine/tests/INTERFACE.md` §1 to §4). None adds a code: each prints only those of `LANGUAGE.md` §8 and `ENGINE.md` §11.

**For all three.** A path is a file, or a directory: every `*.tally` file under it. A joint model's members are found from its `use` lines, relative to its file, each checked against its pin (`LANGUAGE.md` §6.1). Files are taken once, in full-path order by code point (`ENGINE.md` §3.1 step 1). Everything prints to standard output. `run` and `reprint` exit 0 for an answer, 1 for a refusal (the first line starts `refused: `), 2 for a usage error (no command, a missing operand, or an option, a word that starts with `--`, that the command doesn't take); `check`'s exit is its own (§2.3).

### 2.1 `run`

`tally run <path>… [--ask <id>…] [--pin=bytes]`

- **Inputs.** The reality's files; after `--ask`, the ids of the asks to answer, a joint model's member's as `a.q`; with no `--ask`, every ask of the reality (of a joint model, its own). `--pin=bytes` makes a `use` pin the sha256 of the member's file bytes, not of its canonical text.
- **Result** (worked in §7). Each ask's result in `LANGUAGE.md` §5.18's form, in `--ask`'s order, or with none by id, full path, then line, a joint model's own before its members' in `use` order: `<id>: …`, or `<id> (k = …): …` per binding tuple, with values, holes and their questions, per-value refusals, and the register indented under it (§8.1, §8.4, §8.2). The warnings print last, `warning: CODE message`, sorted by code, then text (§8.5). Holes and per-value refusals are answers: exit 0. A reality with no ask prints load's output.
- **Refuses**, each as the one line, exit 1:
  - a reality that doesn't load: `refused: LOAD-… <message> (<path>:<line>)`, the first in full-path order (`LANGUAGE.md` §8.3); a member's as `refused: member <a>: …`;
  - after load: `REQ-ASK no ask <q>`, the first unknown id in request order; `REQ-CLOCK`; `REQ-PATHS` (`ENGINE.md` §11).

  `ENGINE-INVARIANT`, an engine fault, refuses the whole result (`ENGINE.md` §9): report it.
- **A scenario file that doesn't load** refuses that scenario and those over it, with its message; the base answers, exit 0 (`LANGUAGE.md` §6.4). Its line `scenario <s>: refused: CODE message` prints once, before every result, in §2.3's order, whatever is asked; each ask in its world prints `<q>: refused: CODE message`; the file's own asks print nothing, even when named.
- **An ask refused alone:** one that names a scenario whose file isn't given prints its line, `<q>: refused: LOAD-SCENARIO …` (`LANGUAGE.md` §8.3), and the rest answers, exit 0.

### 2.2 `reprint`

`tally reprint <path>… [--pin=bytes]`

- **Inputs.** As `run`; no asks.
- **Result.** For each file in full-path order, `--- <path>` (a file given alone as given, a directory's files relative to it) and its canonical text (`LANGUAGE.md` §2.4). Last, `sha256 <hex>`, the reality's hash over those files (`ENGINE.md` §3.1 step 1): what a `use` line pins.
- **Refuses** what `run`'s reading of the files refuses (`LOAD-PATH`, `LOAD-PARSE`, `LOAD-HEADER`), a text with no canonical form (`LOAD-DUP`, `LOAD-NAME` under an unknown prefix, `LANGUAGE.md` §2.4) and a member off its pin (`LOAD-PIN`). Any other text that parses reprints, loaded or not.

### 2.3 `check`

`tally check <path>… [--pin=bytes]`

- **Inputs** as `run`'s, no asks. It **does** everything load does, every ask's own checks included (`ENGINE.md` §3.1; `LANGUAGE.md` §8.3), and evaluates and walks nothing.
- **Result**, its lines in this order:
  - `ok` alone on the first line when everything given loads;
  - else a base that doesn't load: its one `refused: CODE message` line, as `run` prints it, and nothing more;
  - or, with a base that loads: each refused scenario's `scenario <s>: refused: …` line in the full-path order of the files that declare them (one declared twice, by its first), then each ask refused alone as `<q>: refused: …`, by id, then full path, then line (today `LOAD-SCENARIO`; an ask in a refused scenario's world is covered by its scenario's line);
  - then the warnings load decides (`WARN-UNREAD`), in `LANGUAGE.md` §8.5's order.
- **Exit**, its own rule: 0 only when the first line is `ok`, else 1; 2 for a usage error.
- **When.** Before a caller keeps a change; a question handed over for one run needs none.
- **`ok` isn't a clean run.** `REQ-ASK`, `REQ-CLOCK`, holes and per-value refusals come only from `run`.

---

## 3. How each interaction is carried out

Each recipe is a few lines of Tally text or one command. Every example runs on the small model of §7, with the files it names.

### 3.1 Asking

- **ASK-2, a question not saved.** Write the ask line in a new file; name its id in the run:
  ```
  part question
    tally 2

  ask q_oct: bal(konto, 2026-10-31)
  ```
  `tally run reality.tally question.tally --ask q_oct`. Its id must be new (`LOAD-DUP`). A line that doesn't load refuses the whole run, or the ask alone (§2.1); `run` prints the refusal, so it needs no `check`.
- **ASK-3, a past view, a what-if, paths, a fixed figure.** The ask's clauses (`LANGUAGE.md` §5.18): `as known D`, `in s`, `paths N seed S`, `with x = v`. `ask q_then: bal(konto, 2026-12-31) as known 2026-10-05`.
- **ASK-4, the same pins on every ask.** A `with` on each line: `ask q_raise: bal(konto, 2026-12-31) with salary[acme, 2026-12] = 9 000.00 PLN`. The register prints each pin (`pinned:`).
- **ASK-5, explain a number.** `ask q_why: explain bal(konto, 2026-10-30)`; `explain ref` for a cell (`ENGINE.md` §10).
- **ASK-6, describe an item.** Its text (`reprint`); `readers-of salary`, `inputs-of salary[acme, 2026-10]`.
- **ASK-7, compare the model with the bank.** `ask q_resid: residuals konto 2026-10` prints the residual ledger by month and move (`ENGINE.md` §6.3); `explain bal(konto, 2026-10-30)` lists every movement since the last reading before that day.
- **ASK-10, save a question.** Keep the ask line in the reality's `asks/<topic>.tally` (`LANGUAGE.md` §6.1).
- **ASK-12, fit a parameter; replay one path.** Per trial value, pin it with `with` and read the residuals (`ENGINE.md` §6.4); `path p` replays one path (`LANGUAGE.md` §5.16).

### 3.2 Names and members

- **TXT-3, which thing was meant.** A wrong or ambiguous name is refused at load: `refused: LOAD-KEY kotno is not a key of places (<path>:<line>)`, `LOAD-NAME unknown id <x>`, `LOAD-KEY ambiguous key <key>`. The caller asks the person which.
- **TXT-6, pin a joint model's member.** `tally reprint` the member, and put the last line's hash in the joint model's header. The `use` path is the member's file, or its directory when it has more than one file; `reprint`'s hash is the same for a directory and for its files given one by one:
  ```
  use a = "../mini" sha256 "47bb95f3…"
  ```
  Its items are then `a.konto`, its asks `a.q`.

### 3.3 Changes

- **CHG-1, add a fact.** A version with its rows (`LANGUAGE.md` §5.3), in a new file, `bolt.tally`:
  ```
  part facts_bolt
    tally 2

  version bolt_job
    cite s
    governs 2026-11..
    known 2026-11-03
    jobs + [bolt] lives 2026-11-01..
    pay_dom[bolt] = 12 "umowa, § 5"
  ```
  A figure that changes, or was wrong, is a `replace` row in a new version where an earlier row states the same keys (`LOAD-OVERLAP` without it; at other keys none is needed, `LOAD-REPLACE` with it). Where it reaches instances already booked, CHG-3 follows.
- **CHG-2, add a law version.** A version with `replace` rows where a figure changes, and `replace rule d.r` where a formula does (`LANGUAGE.md` §6.2, §6.5 change 2).
- **CHG-3, the back pay a change opens.** Run after a version that reaches back (`raise_oct`: `salary[acme] = 7 500.00 PLN`, `governs 2026-10`, `known 2026-11-04`). Each booked instance it changed prints as a hole, `?"wage[2026-10, acme] on 2026-10-10 changed after it was paid (version raise_oct); is back pay due, and when?"`, with the register line `adjustment: wage[2026-10, acme] by 500.00 PLN on 2026-11-04 (version raise_oct), remaining 500.00 PLN`. The answer is a move that fulfils the adjustment, or one with no legs that waives it (`LANGUAGE.md` §6.5 change 3).
- **CHG-4, what a new thing leaves unanswered.** Run after the entry. The reads at the new keys print as holes with their questions (`HOLE-UNGOVERNED`; after `bolt.tally`, `?"Ile wynosi pensja z tej pracy w tym miesiącu?" (salary[bolt, 2026-11], salary[bolt, 2026-12])`) or as `RUN-NO-RULE`; a closed-world row is restated with `replace` (`LANGUAGE.md` §6.2).
- **CHG-5, end something.** A thing a version entered ends by its lifespan, `replace jobs + [acme] lives 2020-02-01..2026-11-30`, with rows `none` where a fact stops. A place a `place` section declares ends by rewriting that section's `lives`, an edit the store holds (§4, CHG-10). A place ends only empty (else `RUN-LIFESPAN konto ends on 2026-11-30 holding 15 000.00 PLN`), and a leg outside a thing's lifespan is `RUN-LIFESPAN` too.
- **CHG-6, add structure.** New sections in a new file (`LANGUAGE.md` §6.5 change 5).
- **CHG-7, move `on` forward.** Rewrite the header's `on` line. With `on 2026-10-19`, IMP-1's statement is refused until it moves: `refused: LOAD-KNOWN version statement_2026_10 is known after the model's on date (statement.tally:6)`.
- **CHG-11, what does this change?** Two runs, without and with the added files, the same asks, their lines compared. No run prints movements, so the first day anything differs comes from asks of each place's balance by day. `diff(e) in s against b` compares a scenario with its base.

### 3.4 Scenarios

- **SCN-1, write a what-if and compare it.** A scenario file (`LANGUAGE.md` §6.4), `acme_stops.tally`, asked in its world or against its base. Its row says `replace` where the base states a row at the same keys, as in CHG-1; §7's base states `salary` with no key, so this one needs none:
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
  `tally run reality.tally acme_stops.tally --ask q_diff` prints:
  ```
  q_diff: diff in acme_stops against mini: 1 changed, 0 added, 0 removed, 0 unchanged
    changed bal(konto, 2026-12-31): 15 000.00 PLN against 22 000.00 PLN
    world: acme_stops
    world: mini
    unfulfilled: wage[2026-10, acme]
    held: pay_dom[acme] = 10 (home), read for 2026-12-01
    held: salary[acme, 2026-12] = 7 000.00 PLN (home), read for 2026-12
    cut 2026-11-05
  ```
- **SCN-3, make a what-if real.** Copy its versions into a file of the reality with `known` the day it became real (`on` moved there if needed). The scenario file goes (SCN-4, the store's): its rows are the reality's now. An expression row is refused outside a scenario (`LOAD-SCENARIO`): write its value, read from a run in it. A later `known` gives adjustments the scenario didn't (`LANGUAGE.md` §6.4).

### 3.5 Bank statements

- **IMP-1, a bank export as rows.** One version per statement: `covers`, `anchor` or `pad`, a row per bank row keyed by its id, the closing balance as a `stated` reading (`LANGUAGE.md` §5.19). `statement.tally`:
  ```
  part statements_2026_10
    tally 2

  source bank_2026_10 "Wyciąg z konta, październik 2026"

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
- **IMP-2, duplicates, corrections, card holds.** The key is the bank's row id, so a row already written isn't written again. A correction restates the key in a later version that also `covers` the place (`LOAD-COVERS` otherwise): `replace bank[r1] = (…)`; a row that never happened, or a hold that never booked, `replace bank[r2] = none`; a hold that books restates the hold's key.
- **IMP-3, a statement's closing balance.** The `stated` reading. Every balance read after it prints `anchor: konto 2026-10-31 end, stated 7 900.00 PLN, difference 20.00 PLN`; `residuals` prints the same gap.
- **IMP-4, match bank rows to planned movements.** The caller reads `residuals` (`expected but missing wage[2026-10, acme]`, `unexplained bank[r1]`) and the register's `open:` lines, and writes `fulfils` on the row: in the statement, or later by a `replace` of the row (IMP-2). The engine checks each link at the ask's cut (`RUN-RECORD`) and proposes none.

### 3.6 Law

- **LAW-1, run a law file's checks.** `check e == f` asks in the law part, each side a def or a row with its source, never a bare figure (`LOAD-LITERAL`). `pit.tally`:
  ```
  part pit_rules
    tally 2
    law

  source pit_act "Ustawa o PIT, art. 27"
  source pit_example "Przykład z objaśnień ministra finansów"

  input pit_rate: num
  input pit_expected: money PLN

  def pit[j in jobs, m in months]: money PLN at start_of(m)
    rule base = round(salary[j, m] * pit_rate, 1 PLN, half-up)

  version pit_2026
    cite pit_act
    governs 2026..
    known 2026-01-01
    pit_rate = 12 % "art. 27 ust. 1"

  version pit_2026_example
    cite pit_example
    governs always
    known 2026-01-01
    pit_expected = 840.00 PLN "przykład 1"

  ask pit_check: check pit[acme, 2026-11] == pit_expected law 2026-11-01
  ```
  `run` answers them like any ask: `ok`, or `differs: e = …, f = …` with the difference (`LANGUAGE.md` §5.18).

---

## 4. What the surface leaves to the store

Each needs a store, a user or a clock, so it is the app's; named here, specified nowhere.

| Id | The store's |
|---|---|
| ASK-11 | saved analyses |
| TXT-4 | a snapshot: `reprint`'s hash, `run`'s open holes, the store's files and statuses |
| CHG-8 | a check that a change only adds |
| CHG-9 | a restatement, proved by two runs that print the same |
| CHG-10 | a revision, with its reason |
| SCN-2 | whether the base moved under a what-if (`LANGUAGE.md` §6.4) |
| SCN-4 | dropping a what-if |
| LAW-3 | keeping untested law out of real answers |
| STO-1 | the files and their history; record time R (`ENGINE.md` §2) |
| STO-2 | drafts, statuses, confirmation |
| STO-3 | who wrote a change, from whose words |
| STO-4 | two changes written against one base |

---

## 5. What waits

Not in the first version, with what a caller does meanwhile.

| Id | What waits | Meanwhile |
|---|---|---|
| ASK-8 | the whole timeline as data | asks over `bal` and `flows` |
| ASK-9 | answers as typed fields | the printed form, fixed to the byte |
| TXT-5 | the reference card | `LANGUAGE.md` is the contract |
| TXT-7 | migration to a newer tally version | `MIGRATION.md`, applied by a writer |
| LAW-2 | a gate on a law part: coverage, planted errors, the threshold table | the law's checks are run (LAW-1) |
| STO-5 | results kept under their hash | every call computes; a caller may keep a printed result under the reality's hash, the request and the binary |
| STO-6 | the engine kept warm between calls | every call loads |
| VER-1 | which engine this is: tally version and build (`LANGUAGE.md` §9) | the store knows its binary; another tally version is refused at load |

---

## 6. On paper

### 6.1 The seven changes (`engine/spec/amendments/changes/`)

A change file's diff shows its lines in place; the caller writes them in a new file, since the order of files and sections changes nothing (`LANGUAGE.md` §6.1). Then three steps:
1. Rewrite `on` to the change's `known` date where it is later (CHG-7), and add the change's file.
2. `check` the reality with it: `ok`, or the refusal to fix.
3. `run` the household's asks with `on` moved, without and with the change's file, and compare the printed lines. For a scenario, the asks are written `in s` or as `diff` in an ask file (ASK-2, SCN-1).

| # | Added | What is read off the run with it |
|---|---|---|
| 1 | `facts/2027.tally`: client C's entry, fee and terms; `on` 2027-01-20 | the changed values; no new hole: client C's rows are stated |
| 2 | a law part: two sources, two versions, one known before `on`; `on` 2026-12-18 | the values that read the 2027 minimum and projected wage |
| 3 | a law part: the source, an input, the waiving move, the retroactive health version; `on` 2027-06-15 | the changed health payments; no `HOLE-CHANGED`: the waiver fulfils them |
| 4 | `facts/2028.tally`: Ola's birth; later the 800+ claim and the move settling its back pay; `on` 2028-05-14, then 2028-06-02 | Ola's keys in every ask over persons; the May benefit's back pay, settled |
| 5 | `abroad.tally`, the stub law part `law/de/employment.tally`, `facts/2029.tally`; `on` 2029-05-01 | the stub's question at every result that needs the German law |
| 6 | `facts/2027.tally` (new, or change 1's): the car loan's version, later the overpayment's; `on` 2027-04-02, then 2028-01-08 | the loan's instalments and its end; the shorter term |
| 7 | `scenarios/wife_stops.tally`, and asks `in wife_stops` or `diff(e) in wife_stops against tropek`; `on` stays | the asks' results; nothing before 2027-01-15 differs |

### 6.2 The kinds of visible card (`engine/tests/cases/README.md`)

| Card | Added | Called | Read off |
|---|---|---|---|
| a value | the card's ask in an ask file | `run --ask <q>` | the value line, with its unit |
| a partial | the ask, marked `partial` | `run --ask <q>` | the known part, each named hole with its coefficient (`LANGUAGE.md` §5.18) |
| a refusal | the ask | `run --ask <q>` | the hole's questions, or `refused: CODE message` at the value |
| by-case | the ask; the reading as an axis (`LANGUAGE.md` §5.5) | `run --ask <q>` | one result per option, each with its `reading:` line |
| a relation | its asks, or a `check e == f` ask | `run`; two for a metamorphic one | `ok` or `differs:`; or two results compared |
| a random figure | the ask with its query and `paths N seed S` (`LANGUAGE.md` §5.16) | `run --ask <q>` | the statistic and its standard error |
| a joint household | a joint model, its members pinned (TXT-6) | `reprint` per member, `check`, `run --ask a.q …` | each member's answers as `a.q`, the joint model's own |
| a writing passage | the passage as Tally text | `check`, then `reprint` | `ok` and the canonical text; no key |

---

## 7. The small model the examples run on

`reality.tally`:
```
model mini
  tally 2
  on 2026-11-05
  start 2026-10-01
  end 2026-12-31

source s "Oświadczenia gospodarstwa"

unit PLN: currency minor 0.01

kind jobs
kind wages: event certain
  leg cash: PLN

place konto: accounts PLN lives 2026-01-01..
place world: outside lives 1900-01-01..

input bank: observed
input pay_dom[j in jobs]: int
input salary[j in jobs, m in months]: money PLN question "Ile wynosi pensja z tej pracy w tym miesiącu?"

move wage[m in months, j in jobs over m]: wages
  on day_of(m, pay_dom[j])
  at start_of(m)
  leg cash from world to konto amount salary[j, m]

version home
  cite s
  governs 2026-10..
  known 2026-10-01
  jobs + [acme] lives 2020-02-01..
  pay_dom = 10 "umowa, § 5"
  salary = 7 000.00 PLN "umowa, § 4"
  opening[konto] = 1 000.00 PLN at 2026-10-01 start "wyciąg"
```

Beside it: `question.tally` (§3.1's asks in one part), and the files §3 prints.

**A worked output.** A part `more.tally` with `input unused: int` and the asks `q_base: bal(konto, 2026-10-31)`, `q_dec: bal(konto, 2026-12-31)`, `q_in: bal(konto, 2026-12-31) in acme_stops`, beside an `acme_stops.tally` whose row says `replace`; `tally run reality.tally more.tally acme_stops.tally --ask q_base q_dec q_in`, exit 0:
```
scenario acme_stops: refused: LOAD-REPLACE replace salary[acme]: no earlier row to replace (acme_stops.tally:9)
q_base: 8 000.00 PLN
  unfulfilled: wage[2026-10, acme]
  cut 2026-11-05
q_dec: 22 000.00 PLN
  unfulfilled: wage[2026-10, acme]
  held: pay_dom[acme] = 10 (home), read for 2026-12-01
  held: salary[acme, 2026-12] = 7 000.00 PLN (home), read for 2026-12
  cut 2026-11-05
q_in: refused: LOAD-REPLACE replace salary[acme]: no earlier row to replace (acme_stops.tally:9)
warning: WARN-UNREAD input unused is read by nothing
```
Not shown by it: `unfulfilled:` is a planned instance nothing observed fulfils yet, `held:` a row carried past the cut K, `cut` K itself (`LANGUAGE.md` §8.2); an id given twice prints twice; a `differs:` is an answer, exit 0.
