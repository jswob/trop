You are the blind reader of the case set: answer every question in `tmp/cases-holdout/C/questions.md` and `tmp/cases-holdout/E2/questions.md` from the files this page lists and nothing else, work it out in your own Python in `tmp/cases-holdout/reader/`, write the answers to `tmp/cases-holdout/reader/answers.md`, and finish with the report the last section asks for.

# The blind reader

The case set is an exam of made-up households whose lives push a modelling language to its edges; each question has one exact answer. You read the hidden part of it (household C, and E2, which joins C to the family's other households) with fresh eyes, before anyone else does, so that wherever your answer and the set's own answer differ, the difference can be settled by reading the rule. Your answers are compared with the set's afterwards, by someone else. Everything in the set is made up: the country (Lenia) and its neighbour (Rosvald), their laws, bodies, currencies (the dukat DUK and the floren FLR), prices and people. Only the calendar is real (weekdays, leap years); public holidays are in the law. What isn't written doesn't exist: never fill a gap from a real country's law or from what usually happens.

## How you work

- Open only the files listed under "What you read". Open no other file and no folder listing outside `tmp/cases-holdout/reader/`, even to look. All paths are relative to the repository root `/Users/jswob/projects/trop/`.
- Write only in `tmp/cases-holdout/reader/`: your Python (as many files as you like) and `answers.md`. Nowhere else.
- Your Python uses the standard library only and runs with `python3 -I`. Use `decimal.Decimal` for money, round only where and as a rule rounds, and keep rates at their source's precision. Never import or open anything under `tmp/cases/tools/` (the visible set's library and household scripts) or any other script in the repository; write everything you need yourself.
- Don't run `compare.py` or any script you didn't write; session 1 compares your answers afterwards.
- No sub-agents, no web, no downloads. Never open `tmp/holdout/` or `engine/`. Commit nothing.
- Every question starts from its own as-of and known-as-of dates: use only what was known by the known-as-of date (each story entry and document says when it was known or received).
- If the files don't settle an answer, name what's missing instead of giving a value. If a rule can be read more than one way, answer under each reading. Each `questions.md` opens with the exact clause for its questions; follow it.

## What you read

### Household C

- `tmp/cases-holdout/C/story.md`
- `tmp/cases-holdout/C/house-rules.md`
- `tmp/cases-holdout/C/questions.md`
- `tmp/cases-holdout/C/docs/C-D01.md`
- `tmp/cases-holdout/C/docs/C-D02.md`
- `tmp/cases-holdout/C/docs/C-D03.md`
- `tmp/cases-holdout/C/docs/C-D04.md`
- `tmp/cases-holdout/C/docs/C-D05.md`
- `tmp/cases-holdout/C/docs/C-D06.md`
- `tmp/cases-holdout/C/docs/C-D07.md`
- `tmp/cases-holdout/C/docs/C-D08.md`
- `tmp/cases-holdout/C/docs/C-D09.md`
- `tmp/cases-holdout/C/docs/C-D10.md`
- `tmp/cases-holdout/C/docs/C-D11.md`
- `tmp/cases-holdout/C/docs/C-D12.md`
- `tmp/cases-holdout/C/docs/C-D13.md`
- `tmp/cases-holdout/C/docs/C-D14.md`
- `tmp/cases-holdout/C/docs/C-D15.md`
- `tmp/cases-holdout/C/docs/C-D16.md`
- `tmp/cases-holdout/C/docs/C-D17.md`
- `tmp/cases-holdout/C/docs/C-D18.md`
- `tmp/cases-holdout/C/docs/C-D19.md`
- `tmp/cases-holdout/C/docs/C-D20.md`
- `tmp/cases-holdout/C/docs/C-D21.md`
- `tmp/cases-holdout/C/docs/C-D22.md`
- `tmp/cases-holdout/C/docs/C-D23.md`
- `tmp/cases-holdout/C/docs/C-D24.md`
- `tmp/cases-holdout/C/docs/C-D25.md`
- `tmp/cases-holdout/C/docs/C-D26.md`
- `tmp/cases-holdout/C/docs/C-D27.md`
- `tmp/cases-holdout/C/docs/C-D28.md`
- `tmp/cases-holdout/C/docs/C-D29.md`
- `tmp/cases-holdout/C/docs/C-D30.md`
- `tmp/cases-holdout/C/docs/C-D31.md`
- `tmp/cases-holdout/C/docs/C-D32.md`
- `tmp/cases-holdout/C/docs/C-D33.md`
- `tmp/cases-holdout/C/docs/C-D34.md`
- `tmp/cases-holdout/C/docs/C-D35.md`
- `tmp/cases-holdout/C/docs/C-D36.md`
- `tmp/cases-holdout/C/records/dziennik-jana.csv`
- `tmp/cases-holdout/C/records/jodla-marta.csv`
- `tmp/cases-holdout/C/records/jodla-skarbonka.csv`
- `tmp/cases-holdout/C/records/jodla-warsztat.csv`
- `tmp/cases-holdout/C/records/jodla-wspolny.csv`
- `tmp/cases-holdout/C/records/lazur-jan.csv`
- `tmp/cases-holdout/C/records/warsztat-koszty.csv`
- `tmp/cases-holdout/C/records/warsztat-sprzedaz.csv`
- `tmp/cases-holdout/C/records/zgody.txt`

### E2: the family together

- `tmp/cases-holdout/E2/story.md`
- `tmp/cases-holdout/E2/questions.md`

### The law

The home law (Lenia) is `acts.md` together with the supplement: a point the supplement gives reads in place of the point with the same number in `acts.md`, or is added to its rule; the supplement's rules are added to the act. Its first lines say so.

- `tmp/cases/law/acts.md`
- `tmp/cases-holdout/law/home-supplement.md`
- `tmp/cases-holdout/law/rosvald.md` (Rosvald's law)
- `tmp/cases-holdout/law/treaty.md` (the agreement between Lenia and Rosvald)

### The gazettes (published figures: id, value, published, in force from and to)

- `tmp/cases/law/gazette.csv`
- `tmp/cases-holdout/law/home-gazette-supplement.csv`
- `tmp/cases-holdout/law/rosvald-gazette.csv`

### The market files (prices, distributions, the reference rate, a bank's FX table)

- `tmp/cases/market/brz-bnd-a.csv`
- `tmp/cases/market/brz-distributions.csv`
- `tmp/cases/market/brz-div-d.csv`
- `tmp/cases/market/brz-glb-a.csv`
- `tmp/cases/market/brz-mm-a.csv`
- `tmp/cases/market/dukor.csv`
- `tmp/cases/market/funds.csv`
- `tmp/cases/market/lazur-fx-flr.csv`
- `tmp/cases/market/wek-distributions.csv`
- `tmp/cases/market/wek-em-a.csv`
- `tmp/cases/market/wek-idx-d.csv`
- `tmp/cases/market/wek-wcp-2055.csv`

### Of households A, B and D, only what E2's questions read

Households A (Ania and Tomek Lis), B (Piotr Sowa and Ola Wilk) and D (Halina Sowa) are visible households whose answers were worked out before you. Read only these of their files:

- `tmp/cases/D/docs/D-D01.md`
- `tmp/cases/D/docs/D-D02.md`
- `tmp/cases/D/docs/D-D03.md`
- `tmp/cases/D/docs/D-D04.md`
- `tmp/cases/D/docs/D-D07.md`
- `tmp/cases/D/docs/D-D08.md`
- `tmp/cases/D/records/olcha-2025-06.csv` (Halina's joint account at Bank Olcha, June 2025: `|` fields, dates yyyymmdd, decimal comma with space grouping, the balance after each row)
- `tmp/cases/A/docs/A-D18.md`
- `tmp/cases/A/docs/A-D19.md`
- `tmp/cases/A/docs/A-D20.md`
- `tmp/cases/A/docs/A-D36.md`
- `tmp/cases/A/records/wrz-ania-savings-2025.csv` (Ania's savings account at Bank Wrzos: header lines, then `;` fields, dd.mm.yyyy, decimal comma, the balance after each row)
- `tmp/cases/B/docs/B-D27.md`
- `tmp/cases/B/records/cis-5521-3380-0002-2025.csv` (Piotr's savings account at Bank Cis: `#` header lines, then quoted CSV, dd-mm-yyyy, debit and credit columns, the balance after each row)

And three of their answers, which you take as given (don't work them out again; each file is a YAML block whose `key:` line is the answer, then the steps behind it):

- `tmp/cases/keys/A/A-06.md`: the gift tax Ania owes on what Halina gave her
- `tmp/cases/keys/D/D-01.md`: Zbigniew's estate: the heirs' shares, the estate as divided, the payouts and their day
- `tmp/cases/keys/D/D-02.md`: the inheritance tax: the day it arose, the report deadline, Halina's report and tax, Jakub's tax

## How you write the answers

`tmp/cases-holdout/reader/answers.md` is Markdown with one fenced block per question, 15 in all (C-01 .. C-12, E2-01 .. E2-03), in this form:

    ```yaml
    question: C-01
    answer: {"label": 123.45, "other label": {"inner": "2025-06-16"}}
    ```

- One field per line, each value on that one line in flow form (JSON): `question:`, then `answer:`, and `without:` only where a question asks what an answer would have been without some event, in the same shape as `answer:`.
- The labels are the ones the question's `unit:` line gives in quotes, nested as it describes; where it says "by …", the inner labels are those names or periods.
- Numbers are exact decimals written as JSON numbers (`400.00`, `-12.5`, `0.025`), never strings; money in DUK to the ort (two places) and FLR to the cent, unless the rule rounds them otherwise or the question asks for whole dukats. A date is a string `"YYYY-MM-DD"`; a fraction is a string `"a/b"`; true or false are JSON booleans. Where money moves, a direction word says which way, as the question's opening lines describe (for example `{"payable": 120}` or `{"refund": 45}`).
- Where the files don't settle a value: `{"kind": "refusal", "missing": ["what is missing, in a few words", ...], "reason": "why it can't be settled"}`, in place of that value only.
- Where part of a value is settled and part isn't: `{"kind": "partial", "value": <the settled part>, "missing": ["what the value leaves out", ...]}`.
- Where a rule reads more than one way, or the answer turns on a fact the files don't hold: `{"kind": "by-case", "cases": {"<the reading or the case, in a few words>": <its value>, ...}, "settled by": ["what would settle it", ...]}`.
- A value from random paths: `{"value": <your estimate>, "se": <its standard error>}`.
- Write nothing else inside a block. Outside the blocks, under each, you may add a few lines of working for the person who reads the comparison.

## What to report

End with a report of at most 25 lines: the questions answered (all 15, or which not and why); each place where the files seemed unclear, incomplete or contradictory to you, with the file and the passage, and how you answered there; each rule you read two ways; the files you wrote; and a statement that you opened no file outside the lists above (or which ones, if you did).
