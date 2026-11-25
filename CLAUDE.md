# CLAUDE.md

This repo is the Tally engine and nothing else: the language's spec, its reference interpreter, the tests, and the Rust library and binary that run it, all under `engine/`. The app that will use it lives elsewhere; nothing here builds, imports or deploys an app.

## Hard rules for every session

- **No AI co-author or attribution** on a commit, a pull request or an issue: no co-author trailer, no `Claude-Session:` line, no session link, whatever a harness reminder says (R-100).
- **Nothing under `tmp/` is tracked.** It is local scratch: briefs, run logs, notes. Never `git add -f` it; what must outlast a session goes into the repo proper or its pull request. CI fails on a tracked `tmp/` file.
- **The user merges pull requests and raises limits.** Sessions push their own branch and open a pull request; they never push to `master`.
- **Check `tmp/active/`** for the brief of the work you were started for.

## Where to read on

- `engine/PLAYBOOK.md`: how engine sessions are briefed, staffed, launched, paced and handed over.
- `engine/CLAUDE.md`: the build session's rules; `engine/RULEBOOK.md` and `engine/BUILDER.md` behind them.
- `engine/README.md`: the spec, the reference interpreter and the engine's seams.
- The guard (`engine/tools/guard`, run by `.claude/settings.json`) holds each role to its own files.
