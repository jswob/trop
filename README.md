# Tally

Tally is a language for a household's money and the law that governs it: accounts, flows, taxes and forecasts written as plain text that a person can read and an LLM can write. The engine runs a Tally text and answers its questions, input to output, exactly (rational numbers, no floats) and the same way every time.

This repo holds the engine alone, built and proven before anything uses it:

- **The spec:** `engine/spec/` (`LANGUAGE.md` what a construct means, `ENGINE.md` how it is computed, `DONE.md` the gates the engine must pass); `engine/spec/VERSION` names the frozen tag.
- **The reference interpreter:** `engine/tools/refcore/`, Python written from the spec alone, the oracle the engine is compared with byte for byte.
- **The tests:** `engine/tests/`, which decide when a milestone is done.
- **The engine:** `engine/crates/`, a Rust library (`tally`) and its command line (`tally-cli`).

## Build and check

From the repo's root, with the toolchain `engine/rust-toolchain.toml` pins:

```
engine/tools/check quick     # after an edit: the lints and the unit tests, about 45 s
engine/tools/check commit    # before a commit: adds the gating tests and the build
engine/tools/check full      # what CI runs on every pull request
```

How the work is run, and the rules every session keeps: `engine/PLAYBOOK.md` and `engine/RULEBOOK.md`.
