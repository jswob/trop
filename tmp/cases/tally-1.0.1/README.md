# tally-1.0.1: the case set written in Tally

Session 3 of the case-set plan: the visible probes and households of cases 1.0 written in tally 1.0.1 (`engine/spec/`), each step mapped and judged against the spec, line by line (nothing runs Tally yet). The result is the gap log, **`tmp/cases/gaps-1.0.1.md`**: one row per probe, question, observation and spec finding, worst first, with each edge's status for `INDEX.md`.

## What the folder holds

- `probes/<axis>/` — one reality per probe card: `<EDGE>.tally`, or a directory `<EDGE>/` for a multi-file card (LAW-13, RAND-04, RULE-09, SCEN-01 … 04, SURF-04). 92 cards, axes as in `INDEX.md`.
- `households/A`, `B`, `D`, `E1` — one reality each, read together with `law/`. E1 is the joined family; its `a/`, `b/`, `d/` hold the copies of the other households' files it needs (O:E1-1). `PROGRESS.md` files are the writers' notes, not Tally.
- `law/` — the home law, one file per rule of `law/acts.md` (`PIT-2.tally`, `VAT-1.tally`, …), shared by every household.
- `maps/<chunk>/` — the step maps: every step of a card's or key's derivation, marked same, workaround or missing, with the text's lines.
- `rows/` — per chunk (P0, G1 … G5, A, B, D, E1): the writer's final message (`.writer.md`, `P0.md`), the judge's (`.judge.md`), the supervisor's decisions (`.decisions.tsv`) and the settled rows (`.settled.md`); `G5.recovered.md` (P:RULE-08, recovered from its map); `agents.tsv` (each agent's cost, time and audit).
- `tools/`
  - `merge_gaps.py` — writes the gap log from `rows/` and `INDEX.md`; `--check` runs check 1 (ids and counts).
  - `final_checks.py` — checks 2–5: parsecheck, loadcheck on every reality, `tmp/cases/tools/check.py`, no `__pycache__`.
  - `settle.py` — builds `rows/<chunk>.settled.md` from the writer's rows, the judge's verdicts and the decisions.
  - `rowcheck.py` — checks a rows file: ids, statuses, reasons, spec references, Text paths.
  - `save_final.py` — saves an agent's final message from its transcript, never retyped.
  - `audit.py` — audits an agent's transcript: tools, paths read and written, holdout mentions.
  - `fill.py` — fills a prompt template from files.
  - `pc` — parsecheck, run from the repository root.
  - `prompts/` — every agent's prompt and the three templates.
- `scratch/` — judges' scratch and `scratch/CO/<H>/`, the household-plus-law copies loadcheck reads. Not part of the result.

## Rerunning the checks

From the repository root:

```
python3 -I -B tmp/cases/tally-1.0.1/tools/merge_gaps.py            # rewrites the gap log, prints check 1
python3 -I -B tmp/cases/tally-1.0.1/tools/final_checks.py          # checks 2-5, about 1.5 minutes
```

parsecheck needs its venv (`engine/tools/parsecheck/README.md`). Neither tool checks units, names, scenario rules or run-time refusals, so a text that passes both can still be refused (gap log, part 1). The one loadcheck refusal, SURF-04's LOAD-DUP, is the refusal its card asks for.
