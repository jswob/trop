<!-- saved by tools/save_final.py from agent agent-a778b42d78b221561.jsonl (auditor) -->

## Auditor, session 8c: phase 1 audit of the engine scaffold at db0262e

**The harness blocked my findings file.** It refused the Write of `/Users/jswob/projects/trop/tmp/session8c/agents/auditor/findings.md` with "Subagents should return findings as text". I did not try another way round it, so the full findings are below (sections 1–5); please save them as `findings.md`. `progress.md` beside it was written. `WT` was reverted: `git -C WT status --short` is empty and the guard is restored. No file in either real checkout was touched.

### Summary (the §5 message)
- **Guard probes:** 113 fresh probes (43 paths, 70 Bash), each in 6 roles, so 678 hook calls, plus 18 fails-closed probes.
  - 174 allows that the stated scope refuses (3 noise rows left out). By role: build 43, rules 41, tests 33, spec 14, unset 43.
  - 12 refusals the scope allows. These go the safe way.
  - 1 fail-open (F-10). The unknown role refused all 113 probes.
- **Plants:** 14 planted forms across rulebook groups §1–§7. 12 stayed green in `check fast` or in the cargo-deny steps: P1, P1c, P2, P2b, P3, P3b, P4, P5, P5b, P6, P6b, P7. Two fired: P1b (`clippy::panic`) and P7b (the ast-grep rule for R-108, but only in `full`).
- **Findings:**
  - holes: F-1, F-2, F-3, F-4, F-5, F-6, F-7, F-8, F-9, F-11
  - gaps: F-10, F-12 to F-21, F-23
  - doc: F-22
- **Biggest:**
  - F-11: a build-role script can write `disableAllHooks` into `.claude/settings.local.json`. No checksum covers that file, it is git-ignored, and CI never sees it.
  - F-1: every role can write the main checkout's spec and tests through Bash, the build role included.
  - F-9: a cargo config above the repo with an `[env]` section can turn off clippy's bans in `check fast`.
  - F-12 to F-18: everyday forms the lints miss.
- **For the lead to decide:** see the end of this message.

### 1. Commands (cwd is `WT`, or `P = WT/../probes`)

| # | Command | Exit | Bears on |
|---|---|---|---|
| 1 | `python3 P/probe.py > P/results.tsv` (113 probes × 6 roles) | 0 | A |
| 2 | Fails-closed probes F01–F18 (table 2.3), run against `engine/tools/guard`, and the settings.json hook string run under `/bin/sh -c` with `CLAUDE_PROJECT_DIR=WT` | see 2.3 | F-10 |
| 3 | Real write: `perl -pi -e 's/the/THE/; s/zz/zz/' engine/spec/DONE.md`, then `engine/tools/check-frozen` prints `FAIL spec: DONE.md changed (checksum differs)` | 0, then 1 | F-3 |
| 4 | Real write: `ln engine/spec/DONE.md engine/crates/tally/src/h.md && echo x >> …/h.md`, then `check-frozen` prints the same two FAIL lines | 0, then 1 | F-5 |
| 5 | Plants in batch 1, then `tools/check fast`: `` `panic_any` should not be present in production code `` (`-F clippy::panic`), plus 2 noise errors | 1 | B |
| 6 | Noise and P1b removed, then `check fast` | 1 | `tests_outside_test_module` on p7.rs only |
| 7 | p7 wrapped in `#[cfg(test)] mod tests`, then `tools/exceptions --write`, then `check fast` → `check fast: green, 7 steps` | 0 | F-12 to F-15, F-18, F-19 |
| 8 | Planted files touched, `check fast` again: green, and the log shows tally, tally-cli and tally-runner re-checked | 0 | confirms 7 |
| 9 | `tools/check step ast-grep-scan`: `error[disallowed-exception-outside-sanctioned-file]` at `crates/tally/tests/p7.rs:17` | 1 | P7b, F-20 |
| 10 | `tools/exceptions --check` before the write: `FAIL not in EXCEPTIONS.lock` for both p7 rows. After `--write`: exit 0, `cli 3/3` | 1, then 0 | F-18, F-19 |
| 11 | P4 planted, `cargo metadata --offline`, then `check step deny-tally` (`bans ok`), `check step deny`, `check step tree-probe`, `check fast`: all green | 0 | F-16 |
| 12 | `cargo tree --offline --locked -p tally -e normal --target i686-unknown-linux-gnu` lists `oorandom v11.1.5`. On the host the count is 0. | 0 | F-16 |
| 13 | Baseline for P6: `v.remove(0)` in lib.rs, then `check fast`: `use of a disallowed method std::vec::Vec::remove` | 1 | P6 |
| 14 | `WT/.cargo/config.toml` with `[env] CLIPPY_CONF_DIR = { value = ".cargo", relative = true }` and an empty `clippy.toml`, then `check fast`: the ban is gone; red only on the CLI's now-unfulfilled `#[expect]`s. `check-rules` exits 0. | 1 / 0 | F-9 |
| 15 | Same config, but `.cargo/clippy.toml` = `engine/clippy.toml` minus the `Vec::remove` line, then `check fast` → green | 0 | F-9 |
| 16 | `tools/rules-selftest/run R-7` with that config in place: `FAIL ban coverage: std::vec::Vec::remove (R-7) didn't fire`, `ban coverage 454/455` | 1 | F-9 (full catches it only when the config is inside the tree) |
| 17 | P6b planted, then `tools/check-rules`, then `check fast`; and the guard in the build role on an Edit of `crates/tally/src/p6b_bench.rs` | 0, 0, 0 | F-17 |
| 18 | C3: guard in the build role on Bash `python3 ../probes/c3.py` | 0 | C3 |
| 19 | C3, real run: `python3 ../probes/c3.py`, then `check fast`: `FAIL check-frozen: FAIL rules: engine/clippy.toml changed`. Then `TALLY_ROLE=build engine/tools/stop-check` prints `block 1 of 3` | 0, 1, 2 | C3 |
| 20 | C3: `clippy.toml` restored, leaving only `.claude/settings.local.json` (`{"disableAllHooks": true}`, shown `!!`, ignored). `check fast` green, `check-frozen` prints `spec, tag and rules unchanged` | 0 | F-11 |
| 21 | An unformatted plant, then `check step fmt` (exit 1). Then `RUSTFMT=/usr/bin/true check step fmt` → green | 1, then 0 | F-21 |
| 22 | `git -C M show tally-1.0.1:engine/spec/DONE.md` (l.25) and `engine:engine/spec/DONE.md` (l.27), gate 8's lines | 0 | C4 |

### 2. Probe table (task A)

**2.1 Counts.**
- 678 hook calls. Expected answers come from the stated scope: SCAFFOLD §3's table and its rule-file list.
- 177 rows were allowed where the scope refuses. 3 of them are noise:
  - E43: an Edit of the directory `engine/tools` in the unset role (a directory can't be written as a file).
  - B61 in `tests` and `spec`: Bash outside the repo is free by design in every role but `build`.
- That leaves **174 unexpected allows**, plus **12 safe-direction refusals**.
- These fresh probes all answered correctly:
  - Every rule-file class: `engine/fuzz/Cargo.lock`, `crates/tally-cli/benches/b.rs`, `crates/tally/benches/sub/deep.rs`, `crates/tally/src/.cargo/config`, `crates/tally/src/rustfmt.toml`, `.GitHub/Workflows/Engine.yml`, `.claude/../.claude/settings.json`.
  - Case variants: `ENGINE/Spec/Done.MD`, `engine/TESTS/x`, `ENGINE/TOOLS/GUARD.PY`, `engine/CARGO.TOML`.
  - `..` traversal: `engine/target/../spec/DONE.md`, `crates/tally/src/../../../tools/guard.py`, `P/../wt/engine/spec/DONE.md`.
  - Symlinks in `P` that point into `WT/engine/spec` and `WT/engine`.
  - In the build role: `~/.claude/settings.json`, `~/.claude-personal/settings.json`, `~/.cargo/config.toml`, a repo-root `.cargo/config.toml` and `.claude/settings.local.json`.
  - Bash forms: `bun -e` naming the spec; `python3 <<EOF` naming a rule file; `( cd engine/spec && cp … )`; `echo $(rm …)`; `exec 3> …`; `install -t`; `ln -s`; `git restore -s HEAD~1 -- engine/tests`.

**2.2 Every unexpected row.** "Allowed" lists the roles the guard let through.

| Probe | Payload | Allowed | Class |
|---|---|---|---|
| E33, E34, E35 | Edit, Write, MultiEdit of `M/engine/spec/DONE.md`, `M/engine/tests/checks.tsv`, `M/engine/clippy.toml` | rules, unset | hole F-1 |
| B18, B59, B60 | `git -C M checkout -- engine/spec/DONE.md`; `cp P/x M/engine/spec/DONE.md`; `echo x > M/engine/tests/checks.tsv` | all five | hole F-1 |
| B38–B41 | `{ cp … engine/spec/DONE.md; }`, `if true; then cp …; fi`, `for f in engine/spec/*.md; do rm "$f"; done`, `! rm engine/tests/checks.tsv` | all five | hole F-2 |
| B42 | ``echo `cp P/x engine/spec/DONE.md` `` | all five | hole F-2 |
| B44 | `echo x > >(tee engine/spec/DONE.md)` | build, rules, unset | hole F-2 |
| B45–B49, B56 | `nice -n 5 rm …`, `env -u HOME rm …`, `timeout -s KILL 5 rm …`, `… \| xargs -n 1 rm`, `xargs -I {} cp …`, `stdbuf -o0 cp …` | all five | hole F-2 |
| B01–B04 | `perl -pi -e 's/a/b/; s/c/d/' engine/spec/DONE.md` (real write: command 3); `ruby -i -pe 'gsub(/a/, "b")' engine/tests/checks.tsv`; `python3 -c "import sys; open(sys.argv[1], 'w')" engine/clippy.toml`; `node -e "…process.argv[1]…" engine/spec/DONE.md` | all five | hole F-3 |
| B51, B52 | `sed -i --expression='s/a/b/' engine/spec/DONE.md`, `sed -i -es/a/b/ engine/spec/DONE.md` | all five | hole F-3 |
| B07 | `python3 - <<'EOF'` with `pathlib.Path('eng' + 'ine/spec/DONE.md').write_text('x')` | build, rules, unset | known limit (§3 "Its limits" l.1, tripwire not wall) |
| B22–B29, B53, B54 | `rustfmt engine/fuzz/fuzz_targets/parse.rs`; `cargo fmt --manifest-path fuzz/Cargo.toml`; `cargo clippy --fix --allow-dirty --all-targets`; `tar -xf … -C engine/spec`; `unzip -o … -d engine/tests`; `curl -sSo engine/spec/DONE.md …`; `vim -es -c … engine/spec/DONE.md`; `ex -sc '%s/a/b/\|x' engine/clippy.toml`; `sort -o engine/spec/DONE.md …`; `ditto P/x engine/spec/DONE.md` | all five | hole F-4 (rustfmt, fmt and fix are low) |
| B50 | `patch -p1 < P/x.diff` | build too | hole F-4: the build role refuses `git apply` as whole-tree but not `patch` |
| B55 | `cargo build --target-dir spec` (cwd engine) | build too | hole F-4, low |
| B67 | `cargo add serde --manifest-path crates/tally/Cargo.toml` | + tests, spec | hole F-4, low |
| B32, B33 | `ln engine/spec/DONE.md engine/crates/tally/src/h.md` (real write: command 4); `ln engine/spec/DONE.md P/h && echo x >> P/h` | build, rules, unset / all five | hole F-5 |
| B10 | `git checkout tally-1.0 -- 'engine/sp*'` | rules, unset | hole F-6 |
| B11, B12, B13 | `git checkout HEAD~1 -- ':(top)engine/spec'`; `git restore --source=HEAD~1 -- '*.md'`; `git --work-tree=engine checkout HEAD -- spec/DONE.md` | build, rules, unset | hole F-6 |
| B35, B36 | `D=engine/spec; cd "$D" && echo x > DONE.md`; `cd "$(dirname engine/spec/DONE.md)" && cp P/x DONE.md` | build, rules, unset | hole F-7 |
| B34 | `F=engine/spec/DONE.md; echo x > "$F"` | all but build | hole F-7 (deliberate in guard.py l.303–307, but §3 doesn't say so) |
| B19, B65 | `git commit -F P/msg.txt`; `git commit -m "$(cat msg.txt)"` (the message holds `Co-Authored-By: Claude …`) | all five | hole F-8 |
| B20 | `git merge -m 'Merge … Co-Authored-By: Claude …' feature` | rules, tests, spec, unset | hole F-8 |
| B62 | `mkdir -p ../.cargo && cp P/x ../.cargo/config.toml` | build too | hole F-9 |
| B14 | `git merge engine` | none (scope: every role but build) | false refusal, F-22 (`git merge refs/heads/engine` passes, B15) |
| 8 rows in the spec role | B07, B10, B11, B13, B32, B35, B36, B44 | refused | safe direction |

Known limits behaved as §3 says: B06 (`python3 P/w.py`, a script that writes the spec) and B57 (`bash P/s.sh`) were allowed in every role.

**2.3 Fails closed.**
- F01: a baseline build-role Edit of lib.rs → 0.
- F02: `TALLY_GUARD_SELFTEST_CRASH=1` → 2, `internal error (exit 1); refusing the call`.
- F03–F07: refused with exit 2 for `{oops`, empty stdin, a JSON array, `tool_input: []` and `file_path: 7`.
- F08: tool `Read` → 2. The hook's matcher keeps it from firing.
- F09, F10: `TALLY_ROLE=' build'` and `TALLY_ROLE=BUILD` → 2.
- F11: `PATH=/bin` (no python3) → 2, `internal error (exit 127)`.
- **F12: the hook string under `env -i PATH=/nonexistent` → 127**, `env: bash: No such file or directory`. This fails open (F-10).
- F13: the hook string with the normal PATH → 0.
- F14: the guard moved aside → 2, `engine/tools/guard is missing`.
- F15: `chmod -x` on the guard → 2, but the message says "missing".
- F16: `guard.py` missing → 2, only because Python's "can't open file" exits 2.
- F17: `guard.py` with a syntax error → 2.
- F18: no `engine/` folder → 0.

### 3. Plant table (task B)
Every plant is fresh: I grepped `cases/*.case` and `clippy.toml`'s 455 paths for each form first.

| Group | Plant (where) | Command → exit | Result |
|---|---|---|---|
| §1 R-5 | P1 `m[&k]` on a `BTreeMap` (tally lib.rs) | check fast → 0 | **green** (F-12) |
| §1 R-3 | P1b `std::panic::panic_any(0_u8)` | check fast → 1, `` `panic_any` should not be present in production code `` (clippy::panic) | fired |
| §1 R-2 | P1c `v.iter().copied().map(Option::unwrap)` | → 0 | **green** (F-12) |
| §2 R-24 | P2 `<u64 as std::iter::Sum<&u64>>::sum(v.iter())` | → 0 | **green** (F-12) |
| §2 R-20, R-5 | P2b `*std::ops::Index::index(v, i)` | → 0 | **green** (F-12) |
| §3 R-45 | P3 `impl Display` whose body is `std::fmt::Debug::fmt(self, f)` | fast → 0; ast-grep-scan doesn't flag it | **green** (F-13) |
| §3 R-44, R-50 | P3b `format!("{dir}{}{file}", std::path::MAIN_SEPARATOR)` | → 0 | **green** (F-14) |
| §4 R-41, R-52 | P4 `[target.'cfg(target_pointer_width = "32")'.dependencies] oorandom = { workspace = true }`, with `oorandom = "=11.1.5"` added to `[workspace.dependencies]` | deny-tally, deny, tree-probe, fast → 0 | **green** (F-16) |
| §5 R-66 | P5 `_ = parse(v);` | → 0 | **green** (F-15) |
| §5 R-66 | P5b `let _ignored = parse(v);` | → 0 | **green** (F-15) |
| §6 R-80, R-82 | P6 repo-root `.cargo/config.toml` `[env] CLIPPY_CONF_DIR` pointing at a clippy.toml copy without one ban, plus `v.remove(0)` | fast → 0; check-rules → 0; selftest R-7 → 1 | **green in fast** (F-9) |
| §6 R-91 | P6b `[[bench]] path = "benches/../src/p6b_bench.rs"` | check-rules → 0, fast → 0, guard allows the build role to edit the source | **green** (F-17) |
| §7 R-104 | P7 `#[expect(clippy::needless_return, reason = "R-67: kept for backwards compatibility with the old fixture format")]` in `crates/tally/tests/p7.rs`, after `exceptions --write` (which the builder may run) | exceptions --check → 0, fast → 0 | **green** (F-18) |
| §7 R-108 | P7b `#[expect(clippy::disallowed_methods, …)]` around `std::fs::read` in the same test file | fast → 0; ast-grep-scan → 1 | fires only in `full` (F-20) |

### 4. Findings

Each finding gives the problem, its evidence and a proposed fix (the file and change, and the guard-test or selftest case that would prove it). Every fix is for the lead to make in the `rules` role.

**F-1 (hole). The guard treats other checkouts of the same repo as "outside".**
- `rel()` (guard.py l.89–98) measures every path against the guard's own root.
- In the main checkout `M`, these were allowed:
  - the spec, the tests and `clippy.toml`, by Edit/Write in rules and unset (E33–E35);
  - the same files by Bash in all five roles, the build role included, because `watched()` (l.167–180) skips anything outside the repo apart from `~/.claude` and `~/.cargo` (B18, B59, B60).
- The master's finding is confirmed and is larger than stated: it is not just the rules role.
- Today `M` (branch engine) has no guard of its own. After the merge, its own sessions are guarded, but cross-checkout writes stay open.
- **Fix:** in guard.py, collect the repo's worktree roots at start-up without running git: follow `ROOT/.git` (a directory, or a `gitdir:` file) to the common dir and read `<common>/worktrees/*/gitdir`. Then make `rel()` and `watched()` match a path against the longest root that contains it, so every checkout gets the same verdicts.
- **guard-test:** a throwaway repo with `git worktree add`. From the linked tree, Edit and `cp` onto `<main>/engine/spec/DONE.md` must be refused in every role but spec.

**F-2 (hole). Shell syntax hides the command word.**
- `check_segment` takes the first word after leading assignments and known prefixes as the command. These all get through in all five roles:
  - reserved words and grouping (`{`, `then`, `do`, `!`);
  - backticks;
  - `>(…)`;
  - prefixes with options that take a value (`nice -n`, `env -u`, `timeout -s`, `xargs -n`, `xargs -I`);
  - unknown wrappers (`stdbuf`).
- **Fix in guard.py:**
  - treat `then do else elif {` and `!` as separators, and skip `if while until fi done }` as words;
  - check backtick contents as a nested command, the way `$(` already splits;
  - treat `>(` and `<(` as separators;
  - give each prefix its value-taking options (env `-u -C -S`, nice `-n`, timeout `-s -k` plus the duration, xargs `-n -I -L -P -s -E -d -a`, sudo `-u -g`);
  - add `stdbuf`, `ionice`, `flock`, `caffeinate` and `unbuffer` to `PREFIXES`.
- **guard-test:** the 12 commands B38–B49 and B56, allowed to the owner role only.

**F-3 (hole). A file operand after inline code is taken for "the script".**
- In `interpreter_paths` (l.340–355), inline code doesn't set `script_seen`. So the first operand after `-e '…'` or `-c "…"` is skipped as if it were the script.
- That passes B01–B04. The real write in command 3 confirms it.
- The existing case `perl -pi -e 's/a/b/' engine/deny.toml` passes only because its code has no space and no `;`.
- sed has the same flaw (l.522–528). With `--expression=…` or an attached `-es/…/`, no bare `-e` is seen, so the file is dropped as "the script" (B51, B52).
- **Fix:** set `script_seen = True` once inline code has been seen. For sed, count `--expression=*`, `--file=*` and attached `-e…`/`-f…` as the script.
- **guard-test:** B01–B04, B51 and B52, allowed to the owner role only.

**F-4 (hole; low for rustfmt, cargo fmt/fix/add and `--target-dir`). Writers the tripwire doesn't know.**
- These pass in all five roles: B22–B29, B53, B54, B67.
- In the build role, `patch -p1 < diff` (B50) passes even though `git apply` is refused as whole-tree.
- **Fix:** add these to `WRITERS`, each with its targets:
  - rustfmt, vim, vi, nvim, ex, ed: all operands;
  - tar: the `-C`/`--directory` value;
  - unzip: the `-d` value;
  - curl: `-o`/`--output`; wget: `-O`/`-P`;
  - sort, uniq: `-o`;
  - ditto, cpio, pax: the last operand.
- In the build role, refuse `patch` with no file operand as whole-tree, and refuse cargo's `--manifest-path` outside `crates/` and `--target-dir` other than `target/`.
- **guard-test:** a row for each.

**F-5 (hole). Hard links.**
- `ln` checks only its destination (l.514–516). A hard link to the spec inside `crates/` (build role), or outside the repo, gives a writable second name.
- Command 4: writing through the link changed `engine/spec/DONE.md`, and check-frozen exited 1.
- **Fix:** for `ln` without `-s`, and for `cp -l`/`--link`, also check the source operands.
- **guard-test:** B32 refused to every role.

**F-6 (hole). Git pathspecs are resolved as literal paths.**
- Globs, pathspec magic (`:(top)`, `:/`) and `--work-tree=` get through (l.406–439).
- The build role allows `git restore --source=HEAD~1 -- '*.md'` (B12), `:(top)engine/spec` (B11) and `--work-tree=engine … spec/DONE.md` (B13).
- **Fix:**
  - a pathspec with `*?[` or a leading `:` is expanded read-only (`git ls-files -- <spec>`) and each hit checked, or refused outright in the build role;
  - `--work-tree`/`--git-dir`, with or without `=`, and `GIT_WORK_TREE=`/`GIT_DIR=` are refused in the build role and resolved as the root in the others.
- **guard-test:** B10–B13.

**F-7 (hole). Computed directories.**
- A `cd` to a computed target isn't checked, and later relative paths resolve against a bogus cwd.
- So `cd "$D" && echo x > DONE.md` passes even in the build role (B35, B36), despite its rule against computed paths (l.302–307).
- In the other roles, a computed write target (B34) is allowed on purpose, but §3 doesn't say so.
- **Fix:** after a dynamic `cd`/`pushd`, mark the cwd unknown. In the build role, refuse any later relative write in that command; elsewhere, treat it as computed.
- **Doc:** name computed paths in §3 "Its limits".
- **guard-test:** B35, B36.

**F-8 (hole). R-100's tripwire reads only the command text.**
- `git commit -F msgfile`, `git commit -m "$(cat msgfile)"` and a `git merge -m` (no word "commit") pass with an AI co-author line (l.558–560).
- **Fix:**
  - for `commit`, `merge`, `commit-tree`, `tag -m` and `notes`, read the `-F` file and scan it;
  - in the other roles, refuse a computed `-m`;
  - add a CI step that greps `git log base..HEAD --format=%B` for `co-authored-by|generated with`. The base is available because `engine.yml` fetches with `fetch-depth: 0`.
- **guard-test:** B19, B20, B65.

**F-9 (hole plus gap). A cargo config above the repo.**
- The build role may write `../.cargo/config.toml` (B62).
- check-rules' `cargo_config_problems` (check_rules.py l.232) flags `build.*`, `target.*.{rustflags,linker,runner}`, `profile`, `patch`, `source` and `unstable`, but not `[env]`.
- P6 (commands 14–15): `[env] CLIPPY_CONF_DIR` makes clippy read another clippy.toml, and `check fast` goes green on a banned call.
- `full` catches it (rules-selftest coverage 454/455) only when the config is inside the tree CI checks out. A config above the repo stays local, so only the Stop hook is fooled, and CI on a clean runner fails the banned call.
- **Fix:**
  - check-rules: fail on any `[env]` entry, on an `[alias]` named after build, check, clippy, fmt, nextest, test or doc, and on `include`;
  - guard, build role: watch every `.cargo/` on ROOT's ancestor chain and `$CARGO_HOME`.
- **selftest:** R-82 `cargo-config-env`.

**F-10 (gap, low). The hook wiring fails open on a broken shebang.**
- The settings.json hook `exec`s the shim, whose shebang is `#!/usr/bin/env bash`. With no bash on the hook's PATH it exits 127 (F12), and Claude Code treats that as non-blocking.
- F16 refuses only because Python's own exit code happens to be 2.
- The builder can't reach this, since it can't change the hook's PATH.
- **Fix (settings.json, PreToolUse and ConfigChange):** `/bin/bash "$g"; c=$?; [ "$c" -eq 0 ] || exit 2` in place of `exec "$g"`.
- **guard-test:** the hook string run with `PATH=/nonexistent` must exit 2.

**F-11 (hole). A build-role script can switch off the next session's hooks.**
- The guard allows `python3 ../probes/c3.py` (command 18), a script that writes `{"disableAllHooks": true}` to `.claude/settings.local.json`.
- `check fast` and check-frozen stay green (command 20).
- The file is git-ignored globally (`/Users/jswob/.config/git/ignore:1`), so CI never sees it. RULEBOOK R-81's risk column names exactly this line.
- §3's "the wall is the checksums" doesn't cover it.
- **Fix in check-frozen (so `check fast` and the Stop hook):** fail when `ROOT/.claude/settings.local.json` exists and holds `disableAllHooks`, `hooks` or `defaultMode: bypassPermissions`. The same scan could cover `$CLAUDE_CONFIG_DIR` (or `~/.claude`)`/settings.json`, as check-rules already scans `~/.cargo`.
- Add `.claude/hooks/**` and `.claude/agents/**` (R-81's text) to check_frozen's `OUTSIDE` list and to `RULE_GLOBS`.
- **guard-test and selftest:** a Stop hook test with that file present must block.

**F-12 (gap). Crash and arithmetic forms the bans miss.**
- P1 (R-5: a `BTreeMap` index; `indexing_slicing` ignores maps), P1c (R-2, a forbid-level rule, reached through a method reference), P2 (R-24 through the `Sum` trait) and P2b (R-20/R-5 through `Index::index`) are all green.
- **Fix in clippy.toml's `disallowed-methods`:** add `std::option::Option::{unwrap,expect}`, `std::result::Result::{unwrap,expect,unwrap_err,expect_err}`, `std::iter::Sum::sum`, `std::iter::Product::product`, `std::ops::Index::index` and `std::ops::IndexMut::index_mut`. These also catch method references and fully qualified calls; tests keep their allowances through the test pass.
- Add an ast-grep rule `no-map-index` that errors on `$A[&$B]`, since a map lookup takes `&key`.
- **selftest:** `map-index`, `unwrap-method-ref`, `sum-trait`, `index-method`.

**F-13 (gap). R-45 through a trait method.**
- P3: a `Display` that calls `std::fmt::Debug::fmt(self, f)` is invisible to `use_debug` and to `no-debug-format`.
- **Fix:** ban `std::fmt::Debug::fmt` in clippy.toml.
- **selftest:** R-45 `debug-fmt-call`.

**F-14 (gap, low). A platform constant in the library.**
- P3b: `std::path::MAIN_SEPARATOR` passes R-50's `platform-cfg`, which sees only `cfg` forms.
- **Fix:** an ast-grep rule `platform-const` for `MAIN_SEPARATOR`, `MAIN_SEPARATOR_STR`, `std::env::consts::` and `usize::BITS` in `crates/**`.
- **selftest:** R-50 `main-separator`.

**F-15 (gap). R-66: dropped errors.**
- P5 `_ = parse(v);` and P5b `let _ignored = parse(v);` are green.
- **Fix:** an ast-grep rule `no-underscore-discard`, an error in library code, for an assignment to `_` and for `let _<name> = <call>`.
- **selftest:** `underscore-assign`, `underscore-named-binding`.

**F-16 (gap). A dependency gated to a target the deny configs don't check.**
- P4: `deny.toml` and `deny-tally.toml` set `[graph] targets = ["aarch64-apple-darwin", "x86_64-unknown-linux-gnu"]`.
- CI's nightly job builds `i686-unknown-linux-gnu` (engine.yml l.230), where `oorandom` (a random-number crate) now ships in tally's graph (command 12).
- **Fix:**
  - add `i686-unknown-linux-gnu`, and every target CI builds, to both `targets` lists (or drop the lists);
  - check-rules: refuse `[target.*.dependencies]` in `engine/crates/*` manifests.
- **selftest:** R-52 `target-gated-dependency`.

**F-17 (gap). A bench path can escape `benches/`.**
- check_rules.py l.211–212 tests `p.startswith("benches/")` on the raw string, so `benches/../src/p6b_bench.rs` passes. The build role may then edit the bench source (P6b, R-91).
- **Fix:** `os.path.normpath(p)`, then require it to start with `benches/` and contain no `..`.
- **selftest:** R-91 `bench-path-escapes`.

**F-18 (gap). R-104's phrase check misses variants.**
- `BANNED` (exceptions.py l.31–33) is matched word by word. "backwards compatibility" passes P7, and so do "false-positive" (hyphenated), "to do", "for the moment", "for the time being" and "clippy is mistaken".
- **Fix:** normalise the reason (lowercase, `-` and `_` to spaces, collapse spaces) and match stems:
  - `backwards? compat`
  - `false positive`
  - `to ?do`
  - `fix ?me`
  - `for (now|the moment|the time being)`
  - `temporar(y|ily)`
  - `(clippy|the lint) (is|'s) (wrong|mistaken|broken)`
  - `make (the )?tests? (pass|green)`
- **selftest:** R-104 `banned-phrase-variant`, one row per stem.

**F-19 (gap). One `#[expect]` covers every banned call in its function.**
- The counter counts attributes, not capabilities.
- C1's demonstration: `std::fs::read_dir` and `std::fs::read` added inside `read_input` stay green, and the counter still says `cli 3/3`. The same function could hold a process spawn.
- **Fix:**
  - a protected `tools/cli-io.toml` that lists, for each sanctioned function in `main.rs`, the std paths it may call;
  - check-rules fails on any other `std::fs`, `io`, `env`, `process` or `net` path inside `main.rs`.
- **selftest:** R-111 `exception-widened`.

**F-20 (gap). `check fast` (the Stop hook's check) leaves out ast-grep-scan.**
- Rules enforced only by ast-grep fail in CI, not at the end of a turn. These include R-13, R-18's literals, R-23's calls, R-45's `format!`, R-48, R-50, R-65, R-70, R-103's and R-104's forms, R-107 and R-108. P7b shows it: green in fast, red in full.
- The scan ran in 0 s.
- **Fix:** add `ast-grep-scan` to `fast` in check.py, and update SCAFFOLD §2's line.

**F-21 (gap, low). R-82's clean environment misses some variables.**
- `RUSTFMT=/usr/bin/true` turns the fmt step green on unformatted code (command 21).
- Also not cleared: `RUSTC_BOOTSTRAP`, and D2's `CRITERION_*` and `GUNGRAUN_*` (gungraun isn't in `Cargo.lock` today).
- **Fix (check.py):** add `"RUSTFMT"` and `"RUSTC_BOOTSTRAP"` to `CLEARED`, and `"CRITERION_"` and `"GUNGRAUN_"` to `CLEARED_PREFIXES`.
- **selftest:** R-82 `rustfmt-env-cleared`.

**F-22 (doc).**
- (a) `git merge engine` is refused in every role (B14) because `engine` is also a folder. SCAFFOLD §3 should say to use `refs/heads/engine`, or `check_git` should prefer a ref when `git rev-parse --verify` succeeds.
- (b) The hook prints "missing" for a guard that isn't executable (F15).
- (c) §3's rule-file list protects only `.claude/settings.json`. R-81 protects `.claude/settings*.json`, `hooks/**` and `agents/**`; guard-test makes those app-writable.
- (d) `~/.claude-personal` (this user's `CLAUDE_CONFIG_DIR`) is watched only because `startswith(HOME + "/.claude")` has no separator. It should watch `$CLAUDE_CONFIG_DIR` by name.
- (e) §3 "Its limits" overstates the checksum wall; see F-1, F-9, F-11 and F-7.

**F-23 (gap, D1).** The builder's instruction files can be written by app work. Details and fix are under D1 below.

### 5. Open items

**C1. The CLI's exception budget (3 of 3).**
- *Facts:*
  - `EXCEPTIONS.lock` lists three CLI exceptions, all in `main.rs`: `fn arguments` (args_os), `fn read_input` (stdin) and `fn write_output`, each an `#[expect(clippy::disallowed_methods)]`.
  - `exceptions-budget.toml` caps the CLI at 3 ("arguments, reading the reality, writing output").
  - refcore's `cli.py` on branch `engine`:
    - reads argv: `run <paths> [--ask id …] [--pin=bytes]` and `reprint <paths>`;
    - for each path, tests `os.path.isdir` and walks it with `os.walk` (`dirs.sort()`, `sorted(fs)`, `.tally` only, in `loader.collect`), or reads the file with `open(f,'rb')`;
    - during loading, a joint model's `use` lines make it read member files and directories from disk (`joint.member_texts`: `isdir`, then `collect` and `_read`, or `isfile` and `_read`, whenever `on_disk = os.path.isfile(jfile)`);
    - prints lines to stdout and exits 0, 1 or 2;
    - **never reads stdin**.
- *Judgement:* matching refcore needs file reads by path, a recursive directory listing (R-39, R-40) and member reads in the middle of a load. The library can't do I/O, so the last needs either a two-pass interface or a reader closure passed in from `main.rs`. All of it fits under one exception if every filesystem access sits in one function, since one `#[expect]` covers every banned call in its function (F-19). The stdin exception has no counterpart in refcore. So:
  - **3** is enough if stdin is dropped;
  - **4** if T's interface keeps stdin;
  - **5** if the lead wants the directory listing in an exception of its own so the R-40 review sees it.
- *Also:* refcore's walk lists a directory's files before its subdirectories, which is not a byte sort of full paths. T should confirm where the order matters for `run` (`reprint` sorts by path).
- *Fix:* keep 3. Settle with T whether stdin stays; if it does, raise the budget to 4 through R-112 with the user's dated approval before the builder needs it. Adopt F-19's `cli-io.toml`, so the 3 stay honest.

**C2. A rule change and its checksum land in one commit (R-112).**
- *Facts:*
  - `RULES.sha256` is rewritten in the rule change's own commit; the last two were fe1ac0d and db0262e. So check-frozen passes such a commit by construction.
  - CI checks out with `fetch-depth: 0` and runs on pull requests into master and engine.
  - The rulebook, where the dated approval goes, is tracked on `engine` (force-added under the ignored `tmp/`) but not on `engine-scaffold`.
- *Proposal:* a CI-only step in `check full`, `rule-changes`, running `tools/check-frozen --since origin/$GITHUB_BASE_REF`. It:
  - lists the rule files whose bytes differ between the merge-base and HEAD (rule_files() on both sides, plus `RULES.sha256` entries);
  - requires each commit that touches one to carry a trailer `Rule-Change: R-<n> approved <YYYY-MM-DD>`, to also change `tools/RULES.sha256`, a `tools/rules-selftest/cases/R-<n>.case` and `engine/SCAFFOLD.md`, and to touch nothing in `crates/`, `tests/` or `spec/`;
  - requires a new row in a new rule file, `engine/RULE-CHANGES.md` (date, rule, files, the user's words), which is append-only and tracked on every branch, unlike the rulebook.
- It always prints, for example: `rule-changes: abc1234 (R-7, approved 2026-10-31) changes engine/clippy.toml, engine/tools/RULES.sha256; the audit reads this commit`. It fails with, for example: `FAIL rule-changes: abc1234 changes engine/clippy.toml without a Rule-Change trailer (R-112)`.
- A legitimate R-112 commit passes and is listed in the PR's log.
- *selftest:* `rule-change-without-trailer`, in a temporary repo.

**C3. The interpreter tripwire's reach.**
- *Facts* (commands 18–20):
  - the guard allows `python3 ../probes/c3.py` in the build role (exit 0);
  - the script appends to `engine/clippy.toml`;
  - `check fast` exits 1 on `FAIL check-frozen: FAIL rules: engine/clippy.toml changed`;
  - `stop-check` exits 2 with `block 1 of 3`.
- *Judgement:* for the spec, the tests and the rule files, the Stop hook at the end of the turn plus CI on a clean runner is enough. STOPPED RED is logged after three blocks and CI fails.
- It is not enough for what no checksum covers: `.claude/settings.local.json` (F-11), a cargo config above the repo (F-9), `~/.claude*` and the other checkout (F-1).
- Refusing interpreters outright would cost the builder its own scripts and `bun engine:*`, which goes through `bun`, an interpreter. I don't recommend it.
- *Fix:* extend the wall instead: F-11, F-9 and F-1. If the Claude Code version in use offers OS-level Bash sandboxing with a write allow-list, the lead should check whether the build role can use it; that would be a wall, not a tripwire.

**C4. Gate 8's survivor wording (SO-28).**
- *Facts:*
  - DONE gate 8 (`tally-1.0.1` l.25; `engine` l.27): "At least 90 % of mutants killed in the kernel … Every surviving mutant is listed, each with the reason it is equivalent".
  - `MUTANTS-SURVIVORS.md`: "with its reason naming its rule (R-87; DONE gate 8)".
  - `mutants_survivors.py` l.32: `REASON = re.compile(r"^R-([1-9][0-9]*): \S")`.
  - The score is caught ÷ (caught + missed + timed out), with a 90 % floor (l.114–121).
- *Judgement:* the scaffold lets a non-equivalent survivor sit inside the 10 % with any `R-n` reason. DONE allows only equivalent survivors.
- *Fix (scaffold side):*
  - change `REASON` to `^R-87: equivalent: .{20,}`;
  - reword the header of `MUTANTS-SURVIVORS.md` to "with the reason it is equivalent (DONE gate 8), written `R-87: equivalent: …`; a survivor that isn't equivalent needs a test: T's GAPS.md or a STOP-REPORT";
  - **selftest:** R-87 `survivor-not-equivalent`, which must fail.
- *Observation for the spec:* whether equivalent survivors count against the 90 % is left open (SO-28).

**D1. The skill, the reviewer and `engine/CLAUDE.md` (R6 C-3) — confirmed. This is F-23.**
- Probes E39–E41 and B68–B70: `engine/CLAUDE.md`, `.claude/skills/engine-rust/SKILL.md` and `.claude/agents/engine-reviewer.md` can be written by `rules` and unset, by Edit and by Bash. `build`, `tests` and `spec` are refused.
- `check_frozen.rule_files()` walks `engine/` through `guard.is_rule`, plus `OUTSIDE = [".github/workflows/engine.yml", ".claude/settings.json"]`. None of the three files is checksummed. None exists yet at db0262e.
- *Fix:* when they land, in an R-112 commit:
  - add them to `RULE_GLOBS` (rules only) and to check_frozen's `OUTSIDE` list;
  - add guard-test rows that allow only the rules role;
  - rewrite `RULES.sha256`.

**D2. `GUNGRAUN_*` and `CRITERION_*` (R4) — confirmed.**
- check.py l.28–33: `CLEARED = { "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS", "RUSTC", "RUSTDOC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CLIPPY_CONF_DIR", "CLIPPY_ARGS", "RUSTUP_TOOLCHAIN", "CARGO_INCREMENTAL", }` and `CLEARED_PREFIXES = ("CARGO_PROFILE_", "CARGO_BUILD_", "CARGO_UNSTABLE_", "INSTA_", "PROPTEST_", "NEXTEST_")`.
- No `GUNGRAUN_` or `CRITERION_` appears anywhere in check.py.
- `RUSTFMT` is a live bypass of the same kind; see F-21 for the fix.

### For the lead to decide
1. Whether to fix F-1 by making the guard aware of other checkouts of the repo. This is a rules-role change.
2. C1: ask T whether stdin stays. If it does, the budget goes to 4 with the user's dated approval. Separately, whether to adopt F-19's per-function I/O allow-list.
3. C2: whether to adopt the `Rule-Change:` trailer and the `engine/RULE-CHANGES.md` check (a new rule file, so an R-112 change).
4. C3: accept "extend the wall" (F-11, F-9) rather than refusing interpreters, and whether to look into OS sandboxing for the build role.
5. C4: the strict reading (survivors must be equivalent); the counting question goes to the user as SO-28.
6. Priority among the rest:
   - first, the three with no backstop: F-11, F-1, F-9;
   - then the rule gaps a builder could hit by accident: F-12 to F-18 and F-20;
   - the tripwire holes F-2 to F-8 last, since check-frozen catches them for files inside the repo at the end of the turn.

Scratch files (probe script and results, plant logs) are in `/private/tmp/claude-501/-Users-jswob-projects-trop-engine-scaffold/10fe1c33-6637-4d53-aca9-85a7c2f9d2a8/scratchpad/auditor/probes/`, with `probe.py` and `results.tsv` among them.
