# Tally engine

Tally is a language for a household's money and the law that governs it, plus the engine that runs it: a standalone Rust library and binary, input to output. It is built and proven here before the app, the store or an MCP server uses it. Nothing in `apps/` or `packages/` imports it.

## `spec/`: tally 1.0, frozen

The commit tagged `tally-1.0` froze it. `spec/SHA256SUMS` lists every file's checksum.

- **The definition:** `LANGUAGE.md`, `ENGINE.md`, `OPERATIONS.md`, `PERFORMANCE.md`, `BOUNDS.md`, `DECISIONS.md`, `BUILD-ORDER.md`, `coverage.md`.
- **The contract:** `DONE.md`, the gates the engine must pass.
- **The answer keys:** `models/` and `expected/` (the 79 cases), and `amendments/` (the synthetic household and its 7 changes). `smoke/HH4-a.md` and `smoke/HH4-b.md` are the household's hand checks.
- **Inputs for the tests:** `verification/goldens.md` and `verification/hindcast.md`. Under `verification/blocks/` are the prototype's synthetic fixtures that the goldens are checked against.
- **`tools/loadcheck.py`:** the static load scan. Run it with `python3 -I spec/tools/loadcheck.py` (`--selftest` first).

**Changing it.** After the freeze, a change needs three things: a failing test or a logged contradiction, a row in the gap log, and a patch that names both. Each patch raises the version (`tally 1.0.1`, …) and updates `SHA256SUMS`, and every gate in `DONE.md` is run again.

**Paths.** The spec was written in `tmp/design/`, and its paths are relative to that folder, so `models/`, `expected/`, `smoke/` and `tools/` resolve here. Two kinds of path don't:

- `tmp/verification/goldens.md` and `tmp/verification/hindcast.md` are `verification/` here, and `tmp/blocks/cases/` is `verification/blocks/cases/`.
- Provenance references, such as `REPORT.md`, `gaps/`, `log/`, `candidates/`, `prior-art/`, other `smoke/` files and the rest of `tmp/verification/`, point at the design record. That record stays in the gitignored `tmp/` and isn't part of the spec.

**Never here:** the held-out cases, and any real household's data. The household in `amendments/` is synthetic.
