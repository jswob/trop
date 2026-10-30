# Tally engine

Tally is a language for a household's money and the law that governs it, plus the engine that runs it: a standalone Rust library and binary, input to output. It is built and proven here before the app, the store or an MCP server uses it. Nothing in `apps/` or `packages/` imports it.

## `spec/`: tally 2.1

The commit tagged `tally-2.0` froze tally 2.0. Tally 2.1 adds owners and batches as labels on amounts, opted into per place kind (`CHANGES.md`, `MIGRATION.md` § 2.0 → 2.1); a 2.0 text runs unchanged. Tally 2 is a new version of the language, with a migration from tally 1 (`spec/MIGRATION.md`, `LANGUAGE.md` §9). `tally 1.0.1` stays reachable by its tag `tally-1.0.1`. `spec/VERSION` names the tag the tree matches, and `spec/SHA256SUMS` lists every file's checksum.

- **The definition:** `LANGUAGE.md` (what a construct means) and `ENGINE.md` (how the engine computes it), with `DECISIONS.md` (why: tally 2's records in Part I, tally 1.0.1's decisions carried in Part II) and `coverage.md`.
- **The migration:** `MIGRATION.md`, each tally 1.0.1 construct and its tally 2 form, automatic, warned or manual.
- **Stale until rewritten for tally 2:** `OPERATIONS.md`, `PERFORMANCE.md`, `BOUNDS.md`, `BUILD-ORDER.md`, `smoke/` and `verification/` are tally 1.0.1's, carried as they are. `STALE.md` lists the sections that tally 2's decisions reach. A gate that cites one of them waits until it is rewritten.
- **The contract:** `DONE.md`, the gates the engine must pass, restated for tally 2.0 with the changes the user approved at G2 (logged in its table).
- **The gap log:** `CHANGES.md`, what tally 2.0 changed from 1.0.1, how its contradictions were found and settled, and the gaps still open.
- **The answer keys:** `models/` and `expected/` (the 79 cases, carried into tally 2 with their keys kept), and `amendments/` (the synthetic household and its 7 changes as tally 2 texts, with `amendments/expected/`). `montecarlo/` holds the Monte Carlo models and keys.
- **`tools/loadcheck.py`:** the static load scan for tally 2. Run it with `python3 -I spec/tools/loadcheck.py` (`--selftest` first).

**Changing it.** After the freeze, a change needs three things: a failing test or a logged contradiction, a row in the gap log (`spec/CHANGES.md`), and a patch that names both. Each patch raises the version (`tally 2.0.1`, …), updates `SHA256SUMS`, and every gate in `DONE.md` is run again.

**Paths.** The spec was written in `tmp/design2/`, and its paths are relative to that folder, so `models/`, `amendments/`, `montecarlo/` and `tools/` resolve here. Provenance references (`REPORT.md`, `trace.md`, `regress.md`, `review/`, `rows/`, `cases/`, `refcore/`, `prior-art/`, `problems.md`, `core.md`) point at the design record. That record stays in the gitignored `tmp/` and isn't part of the spec. Part II of `DECISIONS.md` and the stale documents keep tally 1.0.1's paths (`tmp/design/`).

## `tools/refcore/`: the reference interpreter

The Python reference interpreter of tally 2.1, written from `spec/LANGUAGE.md` and `spec/ENGINE.md` alone by writers who never saw an engine. It is the oracle of `DONE.md`'s gate 10 (differential testing). It uses the standard library only and was built and tested under Python 3.14.3; from `engine/`, `python3 -I -B tools/refcore test` runs its 1105 tests, `… run <files of one reality> [--ask id …]` answers a reality's asks, and `… reprint <files>` prints each file's canonical form. Where the spec is silent it refuses `REFCORE-UNSPECIFIED <section>: <question>` (`refcore/UNSPECIFIED.md`); `refcore/HANDOVER.md` describes how it is built. `tools/SHA256SUMS` lists its files' checksums.

## `BUILD-NOTES.md`: the engine's seams

What the Rust engine must leave room for from the start: a joint model's members as their own contexts, a slot for a range on every unknown, the symbol table, the memo keyed by key values, "is it random" per context, and the labels' cost paid only where a kind opts in. Not normative: where it and `spec/` differ, the spec wins.

**Never here:** the held-out cases, and any real household's data. The household in `amendments/` is synthetic.
