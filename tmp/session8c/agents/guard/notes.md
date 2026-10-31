# Session 8c step 3, agent `guard`: notes for the lead

Patch: `patch.diff` (beside this file), made with `git add -A && git diff --cached` in WT at `05103df`. `git apply --check` on a clean `05103df` tree: exit 0. 8 files, +956 −74:
`.claude/settings.json`, `engine/tools/{guard.py, guard_test.py, check_frozen.py, rules_selftest.py, RULES.sha256}`, `engine/tools/rules-selftest/cases/R-80.case` (2 cases added), `R-81.case` (new, 5 cases).

## 0. The four runs (final state, in WT)

| Run | Exit | Summary line |
|---|---|---|
| `engine/tools/check-frozen --write-rules` | 0 | `check-frozen: wrote engine/tools/RULES.sha256 (160 files)` |
| `engine/tools/guard-test` | 0 | `guard-test: 1503 checks, 0 failed` (before: `770 checks, 0 failed`) |
| `engine/tools/check fast` | 0 | `check fast: green, 7 steps (1s)` (19 s cold, earlier) |
| `engine/tools/rules-selftest/run R-80 R-81` | 0 | `rules-selftest: green, 23 cases over 2 rules (17s)`; clean run passes, ban coverage 455/455 |

**The rows bite.** The new `guard_test.py` run against the old `guard.py` and the old `settings.json` (both from `05103df`), at 1493 checks: **391 failed**. Every new refusal row failed there; the rows that stay allowed passed on both.

`RULES.sha256` changes: it gains `.claude/agents/engine-reviewer.md` and the 8 files of `.claude/skills/engine-rust/`, and it has new sums for `.claude/settings.json`, the four tool scripts and the two case files. The lead splits it into R-112 commits, so `RULES.sha256` has to be rewritten again in each commit.

## 1. Per item: what changed, and its proof

**F-1: every checkout of the repo** (`guard.py`)
- `find_roots()` finds the checkouts without running git. It reads `ROOT/.git`, which is a folder or a `gitdir:` file. From there it goes to `commondir` and the common folder. The main checkout is the common folder's parent when that folder is named `.git`. Each `worktrees/*/gitdir` adds another checkout. The result is cached by `roots()` and ordered longest first.
- `locate(path)` returns `(root, rel)` for the longest root that holds the path, so a worktree inside another checkout (`.claude/worktrees/x`) is judged as its own checkout. `rel()` wraps it.
- `verdict()`, `watched()`, `bash_protected()` (its anchors now sit under the matched root) and `say()` (another checkout's path is shown in full) all use `locate`.
- **Fails closed:** a missing `.git`, a `.git` with no `gitdir:` line, a gitdir that isn't a folder, or an unreadable `commondir` or `worktrees/*/gitdir` raises `Refused`, so every call is refused.
- **Stricter than the brief, for the lead to confirm:** the build role writes only in its own checkout. Another checkout's `engine/crates/**` gives "in another checkout of the repo, which the build role never writes (R-79)". The other roles judge every checkout alike.
- **Proof (guard-test):** "Edit of … in another checkout as <role>" and "Bash cp onto … in another checkout as <role>". The throwaway repo gets a linked worktree beside it and a nested one at `.claude/worktrees/nested`, both made with `git worktree add`. The paths checked across checkouts are the spec, the tests, the oracle, `clippy.toml`, `settings.json`, the skill, `guard.py` and `lib.rs`, every role, in both directions: 110 checks. Two more rows prove the fail-closed case: "no .git: refuses" and "an unreadable .git file: refuses".

**F-2: shell syntax** (`guard.py`)
- `RESERVED` words (`if then else elif fi do done while until { } ! esac`) are skipped before the command word.
- `for NAME in WORDS` records `LOOPS[NAME]`. `check_target` then checks `$NAME` and `${NAME}` once for each word, one level deep. `LOOPS` is cleared per call.
- `substitutions()` finds `$(…)` and backtick bodies outside single quotes, including inside double quotes. Each body is checked as its own command.
- `split_operators()` splits the lexer's operator runs (`>(`, `);`). `>(` and `<(` are separators.
- `PREFIXES` is now a table of each prefix's options that take a value and the operands it takes before the command: `timeout`'s duration and `flock`'s lock file. `prefix_option()` handles bundled (`-iu`, `-n5`) and `=` forms. `env -S` splits its string into the command, `env -C` changes the cwd, and `flock … -c` is checked as a nested command. New prefixes: `stdbuf`, `ionice`, `flock`, `caffeinate`, `unbuffer`. The old `timeout` special case is gone. `strip_prefixes()` holds this logic.
- **Proof:** 28 BASH rows under "F-2" (26 refusals, 2 allowed), for example `{ cp …; }`, `if …; then`, `else`, `for f in engine/spec/*.md; do rm "$f"; done`, `while … do`, `!`, backticks, `"$(…)"`, `x=$(…);`, `>(tee …)`, `<(rm …)`, `nice -n`, `env -u`, `env -S`, `env -C`, `timeout -s`, `timeout --signal=`, `xargs -n 1 rm`, `xargs -I {}`, `stdbuf -o0`, `sudo -u`, `flock`, `flock -c`, `ionice`, `caffeinate`, `unbuffer`. Two rows stay allowed: ``echo `date` > /tmp/…`` and `echo "$(cat spec)" | head`.

**F-3: inline code is the script** (`guard.py`, `interpreter_paths(name, args)`, `code_letters()`)
- After inline code, every operand is checked. Inline code is:
  - the word after a code option (`-e`, `-E`, `-c`, `-p`/`--eval`/`--print` for node, bun and deno, `-r` for php, `-c`/`-Command` for pwsh);
  - a bundle that ends in a code letter (`-ne`, `-pe`, `-Bc`);
  - attached code (`-es/a/b/`);
  - any code-like word.
- `python -W`/`-X` take a value, so their value isn't taken for the script.
- sed counts `--expression=`, `--file=` and attached `-e…`/`-f…` as the script.
- **Proof:** 8 refusal rows (B01–B04, `perl -i -ne`, `perl -pi -es/…/`, B51, B52) and 2 that stay allowed (`python3 -c 'print(1)'`, `perl -ne 'print' /tmp/x`).

**F-4: new writers** (`guard.py`)
- New `WRITERS`: `ditto` (destination only, like `cp`), `rustfmt` (skipped with `--check` or `--emit stdout`), `vim`/`vi`/`nvim`/`ex`/`ed` (their files, plus the paths their `-c`/`--cmd`/`+cmd` commands name) and `uniq` (its second operand).
- New target functions:
  - `tar_targets()`: `-x` writes the `-C`/`--directory` folder, or the cwd; `-c`/`-r`/`-u` writes the `-f` archive. The old bundled form `tar xzf` is handled.
  - `unzip`: the `-d` folder or the cwd, unless it lists, tests or pipes.
  - `download_targets()`: curl's `-o` (also `-sSo`), `--output`, `-D`, `-c` and the trace files; curl `-O` writes the URL's last name in the `--output-dir` or the cwd; wget's `-O`, `-P`, `-o` and `-a`, and its default name.
  - `sort -o`/`--output`.
  - `cargo_targets()`: see below.
- `patch` with no file (and no `-o`) is refused in the build role as whole-tree.
- `option_targets()` and `short_value()` are the shared option parsers.
- **cargo** (`fmt` without `--check`, `fix`, `clippy --fix`, `add`, `remove`/`rm`) is judged by the manifest of each package it acts on:
  - each `-p` package's manifest, found through the workspace's members;
  - or `--manifest-path`;
  - or the nearest `Cargo.toml` from the cwd;
  - every member's manifest for `--all`, `--workspace` or a workspace with no package of its own.
- **Proof:** 48 rows under "F-4": every form above, plus the reads that stay allowed (`tar -tf`, `tar -cf /tmp/… engine/spec`, `unzip -l`, `curl … > /tmp`, `sort spec | head`, `uniq spec`, `rustfmt --check`, `cargo fmt --check`, `cargo clippy` without `--fix`). guard-test's throwaway repo now has a cargo workspace like the engine's: three members, and fuzz excluded with its own `[workspace]`.

**F-5: hard links** (`guard.py`, `check_segment`, the `COPIERS` branch)
- `ln` without `-s`/`--symbolic` (or a bundle holding `s`), and `cp -l`/`--link` (or a bundle holding `l`), check their sources too.
- **Proof:** B32 (refused to every role), `ln spec /tmp/h`, `cp -l`, `cp --link`. `ln -s` and `ln -sf` stay allowed.

**F-8, guard side** (`guard.py`, `check_message()` called from `check_git`, `expands()`)
- In every role but unset, for `commit`, `merge`, `tag`, `notes` and `commit-tree`:
  - the file `-F`/`--file`/`-aF` names is read and scanned; an unreadable or computed file name is refused;
  - `-F -` is refused unless the command has a here-document or a here-string;
  - `-m` with a `$` or a backtick that the shell expands is refused. "Expands" means outside single quotes, so `-m 'Rename `x`; $0'` passes.
- One exception: `-m "$(cat <<'EOF' … EOF)"` passes. That is Claude Code's usual commit form, and its text is in the command, which is scanned.
- The text scan in `check_command` now fires on `git` with `commit|merge|tag|notes|commit-tree`, so it covers B20.
- **Proof:** 18 rows under "F-8": B19, `-aF`, `--file=`, `-F` from a subfolder, an unreadable file, B65, `M=$(…); -m "$M"`, `cat f | git commit -F -`, B20, `tag -F`, `notes add -m`, the `$(cat <<'EOF')` form with and without an attribution, and the allowed ones (a clean `-F` file, a single-quoted backtick message, `-F - <<EOF`, `tag -m`, `merge -m` clean).

**F-9, guard side** (`guard.py`, `outside_watched()`)
- In the build role, the guard watches every `.cargo/` from each checkout's parent up to `/`, plus `~/.cargo` and `$CARGO_HOME`. The repo's own `.cargo/` was already watched.
- **Proof:** B62 (`mkdir -p ../.cargo && cp … ../.cargo/config.toml`), `echo x > ../../.cargo/config.toml` from `engine/`, and a `CARGO_HOME` row in the environment section.

**F-10 and F-22 (b)** (`.claude/settings.json`)
- The PreToolUse and ConfigChange strings now read `… then /bin/bash "$g"[ --config-change]; c=$?; [ "$c" -eq 0 ] || exit 2; elif [ -e "$g" ]; then echo "…guard isn't executable; refusing…" >&2; exit 2; elif [ -e "$CLAUDE_PROJECT_DIR/engine" ]; then echo "…missing…"; exit 2; else exit 0; fi`.
- **Proof:** guard-test's new `hook_command()` and `run_hook()` read the real `settings.json` and run its two strings with `/bin/sh -c`:
  - a harmless edit gives 0, and the spec as build gives 2;
  - `PATH=/nonexistent` gives 2 for both hooks (the old strings gave 127, which fails open);
  - ConfigChange gives 0 as rules and 2 as build;
  - a guard that isn't executable gives 2, saying "isn't executable", for both hooks;
  - a missing guard gives 2, saying "missing", for both;
  - a project without `engine/` gives 0.

**F-11** (`check_frozen.py`, `verify_local_settings()`)
- When `.claude/settings.local.json` exists, check-frozen fails if:
  - the file isn't JSON or isn't an object;
  - it has a `disableAllHooks` key, with any value;
  - it has a `hooks` key;
  - it has `permissions.defaultMode == "bypassPermissions"`.
- The problems are printed as `settings: …`.
- **Proof:** new `R-81.case` with `local-settings-disable-all-hooks`, `local-settings-hooks`, `local-settings-bypass-permissions`, `local-settings-unreadable`, and `local-settings-allowed` (`exit: any`; an allow list and `acceptEdits` pass).
- `rules_selftest.list_rules` now prints `ELSEWHERE` after the cases, so `--list` shows R-81's cases "and engine/tools/guard-test (the guard and the hook strings)".
- None of the three checkouts has a `settings.local.json` today, so nothing turns red.

**F-22 (d)** (`guard.py`, `outside_watched()`)
- The build role watches `~/.claude`, `~/.claude.json` and `$CLAUDE_CONFIG_DIR`, each by name. The old `startswith(HOME + "/.claude")` prefix is gone. `~/.claude.json` used to be covered only by that prefix, so it is now named on purpose.
- **Proof:** environment rows with `CLAUDE_CONFIG_DIR` set to a temporary folder (refused in build only), and the same path with it unset (free). There is also an `echo x > ~/.claude.json` row.

**F-23 / D6-3** (`guard.py` `RULE_GLOBS`, `ANCHORS`; `check_frozen.py` `OUTSIDE`, `rule_files()`; `rules_selftest.py` `OUTSIDE`, `build_scratch()`)
- `RULE_GLOBS` gains `engine/CLAUDE.md`, `.claude/skills/engine-rust/*` and `.claude/agents/engine-reviewer.md`. The skill and the agent become anchors, so `rm -rf .claude/agents` is refused.
- check-frozen's `OUTSIDE` gains the skill folder (a folder entry stands for every file under it) and the agent. `engine/CLAUDE.md` is found by the walk over `engine/` once it exists. **The lead must rerun `--write-rules` in step 4's commit.**
- The selftest scratch copies a folder entry with `copytree`.
- **Proof:** EDITS rows for `engine/CLAUDE.md`, the skill's `SKILL.md` and `references/numbers.md`, and `engine-reviewer.md` (RULES only), plus `.claude/skills/design/SKILL.md` (still APP). Two R-80 cases, `builder-skill-byte` and `reviewer-agent-byte`, show the checksum covers them.

## 2. SCAFFOLD §3 draft (replaces "The rule files", "What the guard checks", "It fails closed" and "Its limits"; "The oracle" and "The other two hooks" stay)

**The rule files** are refused to every role but `rules`: `clippy.toml`, `deny*.toml`, `sgconfig.yml`, `rust-toolchain*`, `.cargo/*`, `.config/*`, `tools/*` (with the ast-grep rules, but not the oracle), `fuzz/*`, `crates/*/benches/*`, `engine/CLAUDE.md`, `.github/workflows/engine.yml`, `.claude/settings.json`, the builder's skill `.claude/skills/engine-rust/` and its reviewer `.claude/agents/engine-reviewer.md`, and any `clippy.toml`, `rustfmt.toml`, `rust-toolchain*` or `.cargo/` anywhere under `engine/`. Their checksums are in `tools/RULES.sha256`. The rest of `.claude/` (other skills, agents, hooks) belongs to the toolkit, which syncs it. `.claude/settings.local.json` is app work's, but `check-frozen` fails when it holds `disableAllHooks`, a `hooks` key or the `bypassPermissions` mode (R-81). A role the guard doesn't know refuses everything.

**Every checkout alike.** The guard finds every checkout of the repo from its git files (`.git`, and the common folder's `worktrees/*/gitdir`) without running git. It judges a path by the checkout that holds it; for a worktree inside another, that is the inner one. So the main checkout's `engine/spec/` is the spec from every worktree. The build role writes only in its own checkout. If the git files can't be read, the guard refuses everything.

**What the guard checks.** `Edit`, `Write`, `MultiEdit` and `NotebookEdit` by their path. `Bash` by a tripwire: it refuses a command that names a protected path together with a form that writes:

- a writing command (`cp`, `mv`, `rm`, `sed -i`, `tee`, `truncate`, `ditto`, `rustfmt`, `vim`/`ex`/`ed`, …), judged by what it writes:
  - `tar -x`: its `-C` folder or the cwd; `tar -c`: its archive;
  - `unzip`: its `-d` folder or the cwd;
  - `curl -o`/`-O` and `wget`: the file they save;
  - `sort -o`: its output;
  - `ln` without `-s` and `cp -l`: the source too, since a hard link is a second name for it;
- cargo's writers (`fmt`, `fix`, `clippy --fix`, `add`, `remove`), judged by the manifest of each package they act on. `cargo fmt` in `engine/` also formats T's runner, so it is refused; the builder names its packages: `cargo fmt -p tally -p tally-cli`;
- a redirect;
- an interpreter given a path, or inline code that names one; after inline code every operand counts (`perl -pi -e '…' file`);
- a git command that writes the tree (`checkout`, `restore`, `reset`, `stash`, …);
- `find` with `-delete` or `-exec`.

It reads past shell syntax to the command word:

- reserved words: `if … then`, `for … do`, `{ }`, `!`;
- `$(…)` and backticks, also inside double quotes;
- process substitutions `>(…)` and `<(…)`;
- prefixes with their options: `env -u`/`-S`/`-C`, `nice -n`, `timeout -s`, `xargs -n`/`-I`, `sudo -u`, `stdbuf`, `flock`, `ionice`, `caffeinate`, `unbuffer`.

A `for` loop's variable stands for each of its words.

In the build role it also:

- watches `~/.claude`, `~/.claude.json`, `$CLAUDE_CONFIG_DIR`, `~/.cargo`, `$CARGO_HOME`, and every `.cargo/` from the repo up to `/`;
- refuses `disableAllHooks` and a nested `claude` run (R-81);
- refuses a command naming `TALLY_ROLE` (R-79);
- refuses whole-tree git operations and `patch` without a file.

In every role but app work it refuses a commit message with an AI co-author or attribution line (R-100):

- it looks in the command's text (`git commit`, `merge`, `tag`, `notes`, `commit-tree`) and in the file `-F` names;
- it refuses a message the shell computes (`-m "$(cat f)"`, `-m "$M"`) or pipes in (`-F -` without a here-document), since it can't read either;
- `-m "$(cat <<'EOF' … EOF)"` passes, because its text is in the command.

**It fails closed.** The shim `tools/guard` turns any exit but 0 into 2, so a crash refuses the call. The hook strings in `settings.json` run the shim with `/bin/bash` and also turn any exit but 0 into 2, so a missing `bash` or `python3` refuses too. When `engine/` exists and the guard is missing or isn't executable, they refuse every call and say which (R-79). `guard-test` runs the real hook strings and forces an internal failure to prove it.

**Its limits.**

- The Bash check is a tripwire, not a wall. It can't see what a script run by an interpreter writes (`python3 x.py`, where `x.py` writes a rule file). It reads paths as written, so it doesn't follow:
  - a computed directory: `D=engine/spec; cd "$D" && echo x > DONE.md` (F-7);
  - outside the build role, a computed target: `F=…; echo x > "$F"`;
  - git's pathspec magic and globs: `git checkout HEAD~1 -- ':(top)engine/spec'`, `git restore -s HEAD~1 -- '*.md'`, `git --work-tree=engine checkout HEAD -- spec/DONE.md` (F-6);
  - a commit message reused from another commit (`-C`, `-c`) or written in an editor.
- The wall is the checksums and CI, and it covers this checkout only:
  - `check-frozen` fails on any changed byte of the spec, the oracle, T's tests or a rule file, and on a local settings file that switches off the hooks;
  - CI runs `check full` on a clean runner and checks the branch's commit messages (R-100).
- Three things are outside the wall: another checkout's files (its own `check-frozen` sees them), a cargo config above the repo or in `$CARGO_HOME` (`check-rules` reads it), and the user's Claude settings (`~/.claude`, `$CLAUDE_CONFIG_DIR`). For the user's Claude settings, the build role's tripwire is the only guard.
- It guards Claude Code sessions only. A person's own shell isn't guarded, and neither is a session started with a different `TALLY_ROLE`: the role is trusted as set at launch.
- A worktree's `.claude/settings.json` isn't loaded by the session that wrote it, so the guard binds the next session started there.
- Each role's reach is by path. The `rules` role can write every rule file, which is why a rule change takes the user's dated approval and its own commit (R-112), and the audit reads every such commit.
- `git merge engine` is refused in every role, because `engine` is also a folder. The user merges with `git merge refs/heads/engine` (A-3).

(The draft's "CI … checks the branch's commit messages" and "`check-rules` reads it" describe the lead's F-8 and F-9 halves; drop them if those slip.)

## 3. §7 row updates (text)

- **R-79** The guard works by role, set by `TALLY_ROLE`:
  - `build` may write only an allow-list, in its own checkout;
  - `rules` may write the lint and tool configs;
  - `tests` writes `engine/tests/`;
  - `spec` writes `engine/spec/` and the oracle (refcore);
  - unset keeps C's deny-list for app work.

  Every checkout of the repo is judged alike. The guard fails closed: the edit is refused if it crashes, if `bash` or `python3` is missing, or if it can't read the repo's git files. **Enforced by:** the guard (a PreToolUse hook, run through `/bin/bash`), tested by its own script. **Proven by:** engine/tools/guard-test (every role against every path class, other checkouts, the real hook strings).
- **R-80** Every rule file is protected and checksummed: clippy.toml, deny*.toml, the ast-grep rules, rust-toolchain.toml, `.cargo/**`, the mutants and nextest configs, `engine/tools/**`, the CI workflow, `.claude/settings.json`, `engine/CLAUDE.md`, the builder's skill (`.claude/skills/engine-rust/`) and its reviewer (`.claude/agents/engine-reviewer.md`). In the build role, every `.cargo/` above the repo and `$CARGO_HOME` are watched too. *(rest unchanged)* **Proven by:** selftest: *(existing 16)*, `builder-skill-byte`, `reviewer-agent-byte`.
- **R-81** Hooks can't be switched off from a build session:
  - `.claude/settings.json` and the builder's instructions are rule files;
  - `.claude/settings.local.json` may not hold `disableAllHooks`, `hooks` or the `bypassPermissions` mode;
  - a ConfigChange hook blocks settings changes and logs them;
  - the Bash tripwire refuses `disableAllHooks` and nested `claude` runs, and in the build role it watches `~/.claude`, `~/.claude.json` and `$CLAUDE_CONFIG_DIR`.

  **Enforced by:** the guard; a ConfigChange hook; check-frozen. **Proven by:** engine/tools/guard-test; selftest: `local-settings-disable-all-hooks`, `local-settings-hooks`, `local-settings-bypass-permissions`, `local-settings-unreadable`, `local-settings-allowed`.
- **R-100** Small commits, one green step each; no AI co-author or attribution line, ever. **Enforced by:** a Bash tripwire on `git commit`, `merge`, `tag`, `notes` and `commit-tree`. It reads the command's text and the file `-F` names, and refuses a computed or piped message. CI greps the branch's commit messages (the wall; the lead's step). **Proven by:** engine/tools/guard-test (the commit tripwire); *(the CI step's own proof is the lead's)*.

## 4. For the lead to decide, and what I didn't do

1. **R-81's rulebook text** still says `.claude/settings*.json`, `.claude/hooks/**` and `.claude/agents/**` are protected. In fact only `settings.json` and the engine's own skill and agent are; `settings.local.json` is checked by check-frozen, not protected. The §7 row above matches the code. The RULEBOOK line needs the same correction (F-22 (c)).
2. **`cargo fmt` at `engine/` is refused in every role**, because the workspace holds T's runner. The builder writes `cargo fmt -p tally -p tally-cli`. Worth a line in `engine/CLAUDE.md` (step 4). The alternative would be to judge cargo by the cwd's manifest only, which allows `cargo fmt` in build and leaves the runner and the benches to the checksums. I chose the stricter reading of "with their targets".
3. **F-3's consequence:** `awk '…' spec-file` and `perl -ne '…' spec-file` (reads) are now refused in every role but spec. The builder reads with `grep`, `sed -n` or `cat`.
4. **The build role is refused in other checkouts**, which is stricter than "judged alike" (§1, F-1).
5. **A missing `.git` fails closed**, so a `git archive` export can't run a guarded session.
6. **Small additions beyond the brief's list:** `wget`, `uniq`, `ed`, `nvim` and `~/.claude.json`; the `-m "$(cat <<'EOF')"` exception; and `--list` showing R-81's guard-test proof.
7. **Not done (closed scope):**
   - the auditor's extra F-4 build-role refusals of cargo `--target-dir`/`--manifest-path` outside `crates/` (B55, low);
   - F-6 and F-7 (logged as limits in the draft);
   - plain `VAR=value` assignments are still "computed" (B34);
   - `$(…)` inside a here-document with an unquoted delimiter isn't checked;
   - `xargs` feeding `tar`/`curl`/`sort`/`cargo` isn't treated as blind;
   - the Stop hook string still `exec`s its shim: F-10 named PreToolUse and ConfigChange only, and a missing bash there skips the Stop check;
   - the F-8 CI grep and F-9's check-rules side (the lead's).
8. **`-m` with a `$` inside double quotes** (`-m "costs $5"`) is refused in the R-100 roles. The message says to use `-F`; single quotes pass.
