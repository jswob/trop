# Phase 1's audit of the engine scaffold (session 8c, step 1)

**Made:** 2026-10-30/31 by the 8c lead (`rules` role) and one Opus auditor, on the scaffold as phase 1 delivered it: `db0262e` on `engine-scaffold`, before the merge of `engine` (brief §5; `tmp/active/engine-phase1-scaffold.md` §9). Every check below gives its command and exit code. The fixes go in at step 3, through R-112.

Where it ran:
- **in place**: the scaffold worktree `/Users/jswob/projects/trop-engine-scaffold` at `db0262e`, whose working tree holds one change that isn't phase 1's: the user's `rust-analyzer-lsp` install (2026-10-30) rewrote `.claude/settings.json` (finding A-1);
- **clean**: a throwaway worktree detached at `db0262e` (session scratchpad `audit/wt`), so phase 1's own result is judged without that change;
- **the auditor's probes and plants**: a second throwaway worktree at `db0262e` (scratchpad `auditor/wt`); never the real scaffold.

## 1. The lead's scripted checks

| # | Command (from `engine/`) | Where | Exit | Result |
|---|---|---|---|---|
| 1 | `tools/check full` | in place | 1 | 22 of 24 steps green; red: `check-frozen` ("FAIL rules: .claude/settings.json changed (checksum differs); a rule changes only through R-112") and `rules-selftest` ("the clean run fails, so no case can be judged": the same check-frozen line). 100 s warm |
| 2 | `tools/check full` | clean | 0 | "check full: green, 24 steps (251s)" (cold build) |
| 3 | `tools/rules-selftest/run` (step `rules-selftest` of #2) | clean | 0 | "the clean run passes every tool, nothing is silently off"; "ban coverage 455/455 entries fire"; "green, 219 cases over 77 rules (89s)" |
| 4 | `tools/guard-test` | in place | 0 | "guard-test: 725 checks, 0 failed" |
| 5 | `tools/exceptions --check` | in place | 0 | "exceptions: 5 (tests 0, fuzz-benches 0, kernel 0/20, lexer-parser 0/0, thread 0/1, cli 3/3, library 0/0, deny 2); report-only hits: 0" |
| 6 | `tools/check-frozen` | in place | 1 | "FAIL rules: .claude/settings.json changed (checksum differs)"; notice: `engine/tests/TESTS.sha256` not written yet (T's) |
| 7 | `git show db0262e:.claude/settings.json \| shasum -a 256` | — | 0 | `1082ead2…87650`, equal to `RULES.sha256`'s line: phase 1's committed file is the checksummed one; only the working copy differs |
| 8 | `git diff --stat tally-1.0.1 -- engine/spec` and `git status --short -- engine/spec` | in place | 0, 0 | no output: `engine/spec/` untouched against tag `tally-1.0.1` |

Step logs: `engine/target/check-logs/<step>.log` in each worktree; every green step ends `[exit 0]`.

**Phase 1's scaffold, as committed, is green:** `check full` 24 of 24 on a clean checkout of `db0262e`, the selftest 219 cases over 77 rules and 455 of 455 bans, the guard's tests 725 of 725, the counter at 5 (CLI 3 of 3), the spec untouched. The only red in place is finding A-1, which isn't phase 1's.

## 2. The auditor's probes and plants (task A–D)

One Opus auditor, 78 tool calls, on a throwaway worktree at `db0262e`; its full record, with every command and exit code, is `tmp/session8c/agents/auditor/findings.md` (saved from its transcript with `save_final.py`; the harness refused its own Write of the file). Transcript audit (`audit.py`): **78 calls, 0 holdout, 0 breaches** (`agents/auditor/audit.txt`). Nothing left behind: no `settings.local.json`, no `.cargo/` config in or above any checkout (checked by the lead).

- **Guard probes:** 113 fresh probes (43 paths, 70 Bash) × 6 roles = 678 hook calls, plus 18 fails-closed probes. 174 allows that the stated scope refuses, 12 refusals on the safe side, 1 fail-open (F-10). The unknown role refused all 113.
- **Plants:** 14 fresh forms across rulebook groups §1–§7; 12 stayed green in `check fast` (2 fired: `panic_any`, and R-108's ast-grep rule only in `full`).
- **The lead re-ran four probes against the real scaffold's guard** (`scratchpad/audit/spot.py`, hook input only), build role: `cp … /Users/jswob/projects/trop/engine/spec/DONE.md` exit 0 (F-1), `perl -pi -e 's/a/b/; s/c/d/' engine/spec/DONE.md` exit 0 (F-3), `{ cp … engine/spec/DONE.md; }` exit 0 (F-2), control `cp … engine/spec/DONE.md` exit 2. Confirmed.

## 3. Findings and the proposed fixes

Severity: **hole** (a write the guard's scope should refuse), **gap** (a rule a tool should enforce and doesn't), **doc**. "Pick" is the lead's proposal for step 3; every fix goes through R-112 with its selftest or guard-test case.

### 3.1 The lead's own

| id | Sev. | Finding | Proposed fix | Pick |
|---|---|---|---|---|
| A-1 | gap | The user's `rust-analyzer-lsp` install (2026-10-30, D6-5) rewrote `.claude/settings.json` (added `enabledPlugins`, re-indented to 2 spaces): `check-frozen` and so `check full` are red in place. Any project-scope settings write by the user does the same, by design (settings.json is a rule file) | one rules commit **before the merge**: keep `enabledPlugins` (D6-5 approved the plugin), restore the repo's formatting, rewrite `RULES.sha256` | fix first |
| A-2 | hole | The master's finding: `rel()` treats the main checkout as outside the repo. Larger than stated: see F-1 | F-1 | fix |
| A-3 | — | The guard refuses `git merge engine` in the `rules` role ("it holds engine/spec/DONE.md"), since a merge of `engine` writes the spec and the tests. That is right; the user runs the merge (brief §6). `git merge refs/heads/engine` passes the tripwire (auditor B15): the same class as F-6, caught by check-frozen | document in SCAFFOLD §3 | doc |

### 3.2 The auditor's (full evidence in `agents/auditor/findings.md`)

**Walls with no backstop** (files no checksum covers, or outside the repo):

| id | Sev. | Finding | Proposed fix | Pick |
|---|---|---|---|---|
| F-1 | hole | Other checkouts of the repo are "outside": the main checkout's spec, tests and rule files are writable by Bash in **every role, build included** (`watched()` skips them), and by Edit in `rules` and unset | guard.py finds the repo's worktree roots without running git (`.git` → common dir → `worktrees/*/gitdir`); `rel()`/`watched()` use the longest root holding the path. guard-test: a linked worktree's writes onto the other's spec refused | fix |
| F-11 | hole | A build-role script can write `{"disableAllHooks": true}` to `.claude/settings.local.json` (git-ignored, unchecksummed): the next session runs without hooks; `check fast` stays green | check-frozen fails when `.claude/settings.local.json` holds `disableAllHooks`, `hooks`, or a `bypassPermissions` mode. (Not the auditor's "all of `.claude/agents/**` and `hooks/**`": the toolkit syncs those; only the engine's own files, D1) | fix |
| F-9 | hole + gap | A cargo config above the repo with `[env] CLIPPY_CONF_DIR` swaps clippy's ban list; `check fast` goes green on a banned call; check-rules doesn't look at `[env]`; the build role may write `../.cargo/config.toml` | check-rules fails on `[env]`, on an `[alias]` shadowing a cargo verb, on `include`, in every `.cargo/config*` from the root up and in `$CARGO_HOME`; the build role's guard watches those dirs. selftest R-82 `cargo-config-env` | fix |
| F-10 | gap | The hook string `exec`s a `#!/usr/bin/env bash` shim: with no bash on PATH it exits 127, which Claude Code treats as non-blocking (fails open; not reachable by the builder) | settings.json: `/bin/bash "$g"; c=$?; [ "$c" -eq 0 ] \|\| exit 2` instead of `exec "$g"` (PreToolUse and ConfigChange); guard-test row | fix |
| F-20 | gap | `check fast` (the Stop hook's check) has no ast-grep scan, so ast-grep-only rules (R-13, R-18 literals, R-23, R-45, R-48, R-50, R-65, R-70, R-103/104 forms, R-107, R-108, and step 3's new ones) fail only in CI | add `ast-grep-scan` to `fast` (it runs in about 0 s) | fix |
| F-21 / D2 | gap | `check.py` clears neither `GUNGRAUN_*`/`CRITERION_*` (R4) nor `RUSTFMT` (`RUSTFMT=/usr/bin/true` makes `fmt` green on unformatted code) nor `RUSTC_BOOTSTRAP` | add them to `CLEARED`/`CLEARED_PREFIXES`; selftest R-82 `rustfmt-env-cleared` | fix (brief item) |
| F-23 / D1 | gap | The skill, the reviewer and `engine/CLAUDE.md` are writable by `rules` and app work and not checksummed (R6 C-3) | brief step 3 (D6-3): rule globs and check-frozen's `OUTSIDE` list; guard-test rows | fix (brief item) |

**Rule gaps a builder could hit by accident** (plants that stayed green):

| id | Sev. | Finding | Proposed fix | Pick |
|---|---|---|---|---|
| F-12 | gap | `m[&k]` on a `BTreeMap` (R-5; `indexing_slicing` skips maps), `.map(Option::unwrap)` (R-2 through a method reference), `<u64 as Sum>::sum` (R-24), `Index::index(v, i)` (R-20/R-5) all green | an ast-grep error on **every index expression** outside tests (R-5 bans all of them, maps included); an ast-grep error on `Option::`/`Result::` `unwrap`/`expect`/`unwrap_err`/`expect_err` as paths outside tests (clippy.toml can't: tests may unwrap); clippy.toml bans `std::iter::Sum::sum`, `std::iter::Product::product`, `std::ops::Index::index`, `std::ops::IndexMut::index_mut` | fix |
| F-13 | gap | `std::fmt::Debug::fmt(self, f)` inside a `Display` dodges R-45 | clippy.toml bans `std::fmt::Debug::fmt` | fix |
| F-15 | gap | `_ = parse(v);` and `let _ignored = parse(v);` drop an error (R-66) | ast-grep error on both forms in library code | fix |
| F-16 | gap | a dependency under `[target.'cfg(target_pointer_width = "32")'.dependencies]` escapes cargo-deny (its `targets` omit i686, which CI builds nightly): a random-number crate shipped | add every CI target to both deny configs' `targets`; check-rules refuses `[target.*.dependencies]` in `engine/crates/*` | fix |
| F-17 | gap | `[[bench]] path = "benches/../src/x.rs"` passes check-rules' prefix test, so the builder edits a bench (R-91) | check-rules normalises the path | fix |
| F-18 | gap | R-104's phrase list misses "backwards compatibility", "false-positive", "to do", "for the moment"… | normalise the reason and match stems | fix |
| F-19 | gap | one `#[expect]` covers every banned call in its function: `read_input` could also list directories or spawn a process and the counter still says `cli 3/3` | check-rules: in `main.rs`, `std::process`/`std::net` never, `std::fs`/`std::env`/`std::io` only from a protected list (`tools/cli-io.toml`) | fix |
| F-14 | gap, low | `std::path::MAIN_SEPARATOR` in the library passes R-50 | the library does no path work (R-39); log | log |

**Tripwire holes** (the checksums catch them for files inside the repo at the turn's end, and CI on a clean runner):

| id | Sev. | Finding | Proposed fix | Pick |
|---|---|---|---|---|
| F-2 | hole | shell syntax hides the command word: `{ … }`, `if …; then`, `for … do`, `!`, backticks, `>(…)`, prefixes with valued options (`nice -n`, `env -u`, `timeout -s`, `xargs -n`), `stdbuf` | treat reserved words and `>(`/`<(` as separators; check backticks as nested; give prefixes their valued options | fix |
| F-3 | hole | inline code doesn't count as "the script", so the file after `perl -pi -e '…; …'`, `ruby -i`, `python3 -c "…"`, `node -e` is skipped; `sed -i --expression=` likewise | set `script_seen` on inline code; sed's attached and long forms | fix |
| F-5 | hole | `ln engine/spec/DONE.md <writable>` makes a writable second name (hard link) | `ln` without `-s` and `cp -l` check their sources too | fix |
| F-8 | hole | R-100 reads only the command text: `git commit -F file`, `-m "$(cat f)"`, `git merge -m` pass with an AI attribution | a CI step greps `git log base..HEAD --format=%B` for attribution lines (the wall); the guard reads `-F` files | fix |
| F-4 | hole | writers the tripwire doesn't know: `tar -C`, `unzip -d`, `curl -o`, `sort -o`, `ditto`, `vim -c`/`ex`, `patch` (whole-tree in build), `rustfmt`/`cargo fmt`/`clippy --fix`/`cargo add` (low) | add them to `WRITERS` with their targets; `patch` without a file whole-tree in build | fix |
| F-6 | hole | git pathspec globs, `:(top)`, `--work-tree=` | log as a tripwire limit (check-frozen is the wall) | log |
| F-7 | hole | a computed `cd "$D"` then a relative write | log as a tripwire limit | log |

**Docs:** F-22 (a) `git merge engine` (A-3); (b) the hook says "missing" for a non-executable guard; (c) SCAFFOLD §3's rule-file list against R-81's text; (d) `~/.claude-personal` (`CLAUDE_CONFIG_DIR`) watched only by a prefix accident: watch `$CLAUDE_CONFIG_DIR` by name; (e) §3 "Its limits" overstates the checksum wall (F-1, F-9, F-11). Pick: fix in SCAFFOLD §3 and guard.py (d).

## 4. Phase 1's open items and session 8's findings

| Item | Facts | Pick |
|---|---|---|
| C1. The CLI's budget, 3 of 3 | refcore never reads stdin; it reads files by path, walks directories (`os.walk`, sorted), and a joint model's `use` lines read member files mid-load. One `#[expect]` covers a whole function (F-19) | **keep 3**: arguments, the reality's files (one function: files, directories, members), the output. The stub's stdin read goes; `INTERFACE-draft.md` says so for T. F-19 keeps the three honest |
| C2. A rule change and its checksum in one commit | `RULES.sha256` is rewritten in the rule change's own commit, so check-frozen passes it by construction; CI fetches full history | a CI-only `check full` step, `rule-changes`: every commit since the base that touches a rule file must carry a trailer `Rule-Change: R-<n> approved <date>` and also change `RULES.sha256` and a selftest case; it always prints the list for the audit. selftest in a temporary repo |
| C3. The interpreter tripwire's reach | a build-role `python3 script.py` writing `clippy.toml` passes the guard; `check fast` then fails on check-frozen and the Stop hook blocks (`block 1 of 3`) | **extend the wall, don't refuse interpreters** (the builder's own scripts and `bun engine:*` need them): F-1, F-9, F-11. Open for G2: whether Claude Code's OS-level Bash sandbox (a write allow-list) could bind the build role, a real wall |
| C4. Gate 8's survivors (SO-28) | DONE gate 8: each survivor listed "with the reason it is equivalent"; `mutants_survivors.py` accepts any `R-n: …` reason | strict: reason `R-87: equivalent: …` (20+ chars), header reworded; selftest `survivor-not-equivalent`. Whether equivalent survivors count against the 90 % stays SO-28's, for the spec |
| D1 (R6 C-3) | confirmed (F-23) | brief step 3 |
| D2 (R4) | confirmed (F-21) | brief step 3 |

**Cost of the picks:** about 0.4–0.5 % of the week on top of step 3's 0.4–0.7 %, so step 3 lands near 1.5× its estimate. The cheapest cut: the tripwire group (F-2–F-5, F-8 guard side) logged as limits instead, about 0.1 %.
