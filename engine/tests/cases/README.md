# The case set, cases 1.4.0

An exam for Tally: made-up families whose lives push the language to its edges, each question with an exact answer. If Tally handles these, it can handle a real household.

This README describes the visible set. Since cases 1.2.0 the holdout's households C and E2 are here too, unsealed after session 5r's exam: `READER.md`, `boundaries-hidden.md` (edges HID-1..5), `INDEX-hidden.md` and `ERRATA-hidden.md` describe them, and `tmp/cases-holdout/` keeps the exam's record. Since cases 1.4.0 session 7's six blind cards are here too, S1 to S6: each a self-contained card (its own cast, law, house rules, records and questions in one `card.md`), keyed by its own script (`tools/keys_s<n>.py`, keys in `keys/S<n>/`), in this README's key format; `ERRATA.md` ADD-10 says where they came from. A second holdout, `tmp/holdout/`, stays sealed; nothing here depends on it.

## The principles

- **Extreme but believable:** lives that could happen, with their extremes clustered. Nothing absurd.
- **All made up:** the country (Lenia, its money the dukat, DUK), its law, its bodies, currencies, prices and people. What isn't written doesn't exist, so nobody fills a gap from a real act.
- **Minimal witness:** every rule, record and number is there because a question needs it. Long records come from scripts.
- **Facts come from records, never from rules:** prices from price files, law figures from the gazette, purchases from the broker's export. A house rule says what was meant; where the records differ, the records win and the gap is named.
- **Past from records, future from paths:** prices, law and facts have a recorded past and a forecast future. Future law is a stated what-if or a draw, run on every path.
- **Law and house rules are one kind:** numbered rules with versions and dates.

**The parts:** the **edge list** (`boundaries.md`), every kind of tricky thing; one **probe** per edge, a card of a few lines with its answer (`probes/`, see `probes/README.md`); the **home law**, gazette and market files, shared by every family; three visible **families**, A (the Lis family: two salaries, a child born in 2023, investing, a car on credit, a gift, an inheritance), B (Piotr and Ola: a one-man business, a flat in shares on a mortgage, a client paying in florens) and D (Halina: a death, pensions, bonds, an estate), and **E1**, which joins them as one family; the **grading side**, which the language under test never reads (keys, the scripts behind them, `compare.py`, the checklist); and **`INDEX.md`**, edge → probe → questions, for the design session.

## The format

```
tmp/cases/
  README.md  INDEX.md  boundaries.md  ERRATA.md  VERSION  MANIFEST.sha256
  law/acts.md                   the home law: numbered rules, each version with in-force dates
  law/gazette.csv               published figures: id, value, published, in force from/to
  market/<series>.csv           prices, the reference rate, FX, CPI
  probes/<axis>/<edge>.md       one card per visible edge
  probes/pairs/<edge>+<edge>.md a pair card: two edges where they meet (probes/README.md)
  A/ B/ D/                      story.md, docs/, house-rules.md, records/, questions.md
  E1/                           story.md (the joins only), questions.md
  C/ E2/                        the unsealed holdout (cases 1.2.0): C as A, B, D; E2 joins A, C and D
  S1/ … S6/card.md              the blind cards (cases 1.4.0): one self-contained card each, keyed by tools/keys_s<n>.py
  keys/<H>/<qid>.md             written by the key scripts; keys/probes/<edge>.md by keys_probes.py
  keys-small/<H>/<qid>.md       keys for runs cut short, by a key script's knob (keys-small/README.md)
  tools/                        lib/, fam/, gen_*.py, keys_*.py, compare.py, check.py, index.py, usage.py
```

UTF-8 Markdown and CSV only. Every rule, document, record file and row has an id, so keys can cite it.

- **story.md:** the cast first (people with birth dates and relations; each couple's property regime; accounts and assets at the start, with owners and shares), then one dated entry per event: `date · known <date> · <document id>`. A fact with a document is stated there and the story points at it; one without (an agreement, a forecast assumption) is stated in the story, in the household's words. A correction is a new entry naming the one it corrects.
- **docs/:** contracts, letters, decisions, forms, payslips, in the issuer's own number format, headed by id, issuer, date issued, date received. Planted errors are not marked.
- **house-rules.md:** the household's own rules, numbered, each with its start date and versions, in the household's words.
- **records/:** each institution's own CSV dialect (separator, decimal comma, date format, header lines, a balance-after column where the bank gives one), one file per account and period. Full exports where a question reads them; elsewhere only what the household kept, as the story says. A gap in the records is a fact.
- **questions.md:** at most 12 per household and 8 for E1; cases 1.1 lifted the cap for its three additions (A-13, B-13, D-13). Each has an id, an as-of date, a known-as-of date, the question in the household's words, and the unit. One clause at the top: "If the dossier doesn't settle an answer, name what's missing instead of a value. If a rule can be read more than one way, answer under each reading." Questions ask for consequences, never for a choice (no advice), and never point at a document or a rule. A forecast question states its scenario.
- **probes:** a few lines of story, the rule, one question, a hand-worked answer, and a YAML block of inputs; `tools/keys_probes.py` computes each key with the library into `keys/probes/<edge>.md`, and it must equal the hand-worked answer. A SURF edge's probe is a passage to write, with no key.

## Keys

**One key per question,** in `keys/<H>/<qid>.md`, written by the household's key script: a YAML block (the key, machine-readable), then the derivation as the script's trace.

**Five kinds:**
- **value:** an amount, a date, a list or a table;
- **partial:** a value plus the named things it leaves out;
- **refusal:** no value; it names what's missing;
- **by-case:** one value per reading of an unclear rule, or per branch of a pending result, with the date that settles it;
- **relation:** a statement the answers must satisfy, checked without a derivation: an invariant (a transfer between own accounts never changes the total; shares sum to 1; the people's net worths add up to the household's), a metamorphic check (same seed, same answer; swap two identical people and their answers swap; zero volatility gives the plain projection; drop the duplicate row and the balance moves by exactly its amount; a change from date D changes no answer before D), or a closed form.

**Placement:** a partial, a refusal or a by-case stands in place of the smallest value it concerns; the settled values beside it stay plain, and a total that leaves the hole out is itself a partial.

**Values:** exact decimals (`Decimal`); money with its currency, to the smallest unit; rounded where and as the rule rounds, step by step, citing it, and nowhere else. A direction word says which way money goes (refund, payable, owed to, owed by); where nothing moves, a bare 0.

**Comparison:** exact for everything deterministic. A random key gives a reference `r`, its standard error `se_r` and a target `p`: `r` is a closed form where one exists (`se_r` = 0); otherwise the script runs enough seeded paths for `se_r ≤ p/4`. An answer passes if its own error is at most `p` and it lies within 4 combined standard errors of `r`.

**Holes are judged both ways:** an answer must name every missing fact the key lists, and must not call missing a fact the dossier settles.

**Derivation:** numbered steps, each with what it computes, the formula and its inputs (a rule id, a document, a record row, an earlier step), readable without running anything. **Also recorded:** the as-of and known-as-of dates; for a key a twist moves, `without:` its value without the twist. `without:` is the grading side's proof that a twist moves 3 or more keys: no question asks for it, it isn't marked against a reader unless the reader gives one, and it may rest on facts the dossier doesn't hold.

**Why:** typed outcomes let a later session mark pass or fail with no judgement calls, and keep a hole from passing as a number. Relations test whole classes of errors at once, like property tests.

**Marking:** `python3 -I tmp/cases/tools/compare.py <answers.md> --keys <H>` marks each question pass, fail, read or not run; `--self-test <H>` checks that every key passes against itself. A run cut in paths is marked with `--small` against a key made for it (`--keys-root tmp/cases/keys-small`): a random answer then needs only to lie within 4 combined standard errors of `r`, with no precision bar. Prose (holes, reasons, case labels) is listed as **read**: a question passes only once those lines are read and agreed.

## Tools

All Python runs as `python3 -I` from the repository root, standard library only.

- `tools/check.py` runs the freeze checklist on this tree; its last output is `check-output.txt`. `--regen` also regenerates every household from the union (`union_e1.py`) and finds every file byte-identical.
- `tools/index.py` writes `INDEX.md`.
- `tools/usage.py` measures a session's cost, as a share of the week.

## How the cases are used later

**Session 3, the gap log** (`tmp/cases/gaps-1.0.1.md`): the probes and the visible families written in tally 1.0.1, one row per probe, per question, and per modelling observation (a fact or rule that is hard to write although no question asks about it). Statuses, worst first: **breaks** (written, but the answer is wrong by the spec, or the spec is silent; a value where the key has a hole is the worst case), **can't express** (the reason names what's missing), **awkward** (right only through a workaround; the reason names it), **expressed**. Outside the scale, **case suspect** goes to `ERRATA.md`.

**Session 4** starts from `INDEX.md`, works on the probes of one edge at a time, and uses the families as the proof that everything works together. A trace table maps every row that isn't "expressed", and every LC concern, to the change that resolves it, or to "deferred" with the user's ok. Every change states its general rule and shows it in two places (two families, a family and a probe, or a case of the 79), so nothing is fitted to one case; "expressed" rows are re-written (no regressions). The seven qualities, in order, settle trade-offs.

**Session 5:** the probes, the visible families, then the holdout, modelled in the new spec and marked **pass**, **fail** or **not run** by `compare.py`; each fail with its cause (the language, the modeller, or the case, which is an erratum); the holdout's first score reported apart; the reading test (about 20 passages from the models, explained by the user). **The bar:** everything passes, and so does the reading test.

## Changes

Frozen as `cases 1.0` (`VERSION`; `MANIFEST.sha256` holds every file's hash). After the freeze a file changes only through `ERRATA.md` and a version bump. Now `cases 1.0.1` (2026-10-25): `ERRATA.md` lists each change, and ends with the rows of `gaps-1.0.1.md` whose case or key changed since session 3 judged them. Then `cases 1.0.2` (2026-10-29): the errata from the tally 2 design round, with the rows for session 5 to rejudge; and `cases 1.0.3` (2026-10-29): an erratum from session 5's exam, with its rows to rejudge. Then `cases 1.1` (2026-10-29): additions only, no existing key moved: three questions (A-13, B-13, D-13), a pair card (CUR-01 × RAND-01), a new edge LAW-20 with its card, a run-end knob for B-12 (`keys-small/`) and `compare.py --small`; `ERRATA.md` lists them. Then `cases 1.1.1` (2026-10-29): tools only, no key or paper moved: `compare.py` marks the probe keys (house `P`), and the tools find the case set at its new home, `engine/tests/cases/`. Then `cases 1.2.0` (2026-10-30): the holdout unsealed, households C and E2 added with their keys and tools, and two errata from gate G5 (C-07's key, B-D28's date with B-05's key); `ERRATA.md` lists them. Then `cases 1.3.0` (2026-10-30): tally 2.1's additions only, no existing key moved: three edges OWN-10, OWN-11 and OWN-12 (parts that arrive, parts that leave, reading parts) with their cards of 24 parts, SURF-05..07, the pair card PTY-05 × OWN-12 (a nested joint model), and A's two questions A-14 and A-15 (Ania's withdrawal by case, and under the house rule "in proportion"); `ERRATA.md` lists them. Then `cases 1.4.0` (2026-10-30): additions only: session 7's six blind cards S1 to S6 with their keys and key scripts, and, by the user's G2 Q-1, S4-03's and S4-04's keys in a new form, naming the court's missing ruling, their by-case figures moved to a pinned twin (S4-03P, S4-04P); no other key moved; `ERRATA.md` lists them (ADD-10, TOOL-4).
