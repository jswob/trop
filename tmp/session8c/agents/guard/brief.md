# Session 8c, step 3: the guard and the walls (agent `guard`)

You are a helper of session 8c (the Tally engine prep, part B2). The user approved the audit's fix list on 2026-10-31. You implement the guard-side fixes as a **patch** in a throwaway worktree. The lead reviews it, splits it into R-112 commits and runs the full check. You never commit.

## 1. Hard rules

1. **Seals** (word for word from the session brief):
   - Never open, list or search `tmp/holdout/`.
   - A recursive `find`, `grep` or glob over `tmp/` must prune it: `find tmp -path tmp/holdout -prune -o …`.
   - Every agent prompt says the same, word for word.
   - Don't open `engine/tests/cases/*-hidden.md`; no step here needs them.
2. **No sub-agents.** Don't spawn agents or forks, run workflows or invoke skills; never use the Agent, Task, Workflow, Skill or SendMessage tools. No web research.
3. **Where you work.** `WT` = `/private/tmp/claude-501/-Users-jswob-projects-trop-engine-scaffold/10fe1c33-6637-4d53-aca9-85a7c2f9d2a8/scratchpad/guard/wt`, a git worktree detached at `05103df` (branch `engine-scaffold`'s head: phase 1's scaffold merged with branch `engine`). Edit freely inside `WT`. Scratch goes in `WT/../scratch/`.
4. **What you may write outside WT:** only `/Users/jswob/projects/trop/tmp/session8c/agents/guard/` (`patch.diff`, `notes.md`, `progress.md`). Never a file named `REPORT.md`. Nothing in `/Users/jswob/projects/trop-engine-scaffold` or elsewhere in `/Users/jswob/projects/trop`, and no git command that writes there (no commit, checkout, worktree, branch, tag, stash). Inside WT: no `git commit`, no `git stash`.
5. **Your session runs under the live guard in the `rules` role;** don't try to get around it. Probe the guard only by feeding hook JSON to `WT/engine/tools/guard` on stdin (`TALLY_ROLE=build WT/engine/tools/guard < probe.json`), and through `WT/engine/tools/guard-test`.
6. **Machine:** at most one cargo build, `check` or selftest run of yours at a time. No installs. No refcore or `run.py` runs.
7. **Scope is closed:** the items in §3. Aim for 70–110 tool calls. Keep `progress.md` short and current, so a context reset loses little.

## 2. Read first (narrowly)

- `/Users/jswob/projects/trop/tmp/session8c/audit.md` §3 (the table rows named in §3 below) and `agents/auditor/findings.md` (the same ids' evidence and proposed fixes).
- `WT/engine/SCAFFOLD.md` §3 (roles, the rule files, what the guard checks, its limits) and §6 (R-112's procedure).
- `WT/engine/tools/guard.py` (whole), `guard_test.py` (its tables), `check_frozen.py`, and `rules_selftest.py`'s header and `OUTSIDE` list. The oracle area (`ORACLE_GLOBS`) and `frozen.toml` are new at the merge (SCAFFOLD §8 items 29–33).
- `WT/.claude/settings.json` (the hook strings).

## 3. Your items (the closed list)

Each fix needs proof: a guard-test row, or a rules-selftest case in `tools/rules-selftest/cases/R-<n>.case` (the case format is in `rules_selftest.py`'s header).

1. **F-1:** the guard knows every checkout of the repo. Without running git, find the worktree roots from `ROOT/.git` (a file `gitdir: …` in a linked worktree, a directory in the main one) → the common dir → `worktrees/*/gitdir`, plus the main checkout. `rel()` and `watched()` judge a path against the longest root that holds it, so the main checkout's `engine/spec/` is the spec in every role. Fails closed if the git files can't be read. guard-test: a linked worktree whose writes onto the other checkout's spec, tests, oracle and rule files are refused (build it inside guard-test's throwaway repo with `git worktree add`).
2. **F-2, F-3, F-4, F-5:** the tripwire's holes, as the audit's table proposes: reserved words, `{ }`, `!`, backticks, `>(`/`<(` as separators; prefixes with their valued options (`nice -n`, `env -u`, `timeout -s`, `xargs -n`), `stdbuf`, `timeout`; inline code counts as the script (`perl -pi -e '…'`, `ruby -i`, `python3 -c`, `node -e`), `sed`'s attached and long forms; new writers (`tar -C`/`-x`, `unzip -d`, `curl -o`, `sort -o`, `ditto`, `vim -c`/`ex`, `rustfmt`, `cargo fmt`, `cargo clippy --fix`, `cargo fix`, `cargo add`/`remove`) with their targets; `patch` with no file is whole-tree in the build role; `ln` without `-s` and `cp -l` check their sources too. One guard-test row per form.
3. **F-8 (guard side):** R-100 also reads the file `git commit -F <file>` names, and `-m "$(…)"` with a command substitution is refused in the roles R-100 covers. (The CI step that greps the commit messages is the lead's.)
4. **F-9 (guard side):** in the build role, `watched()` also covers every `.cargo/` directory from the repo root up to `/`, and `$CARGO_HOME` (default `~/.cargo`). (check-rules' side is the lead's.)
5. **F-10:** the PreToolUse and ConfigChange hook strings in `.claude/settings.json` run the guard as `/bin/bash "$g" …; c=$?; [ "$c" -eq 0 ] || exit 2` instead of `exec "$g"`, so a missing `bash` on `PATH` still refuses. **F-22 (b):** the message for a guard that exists but isn't executable says so, not "missing". guard-test rows run the real hook strings.
6. **F-11:** check-frozen fails when `.claude/settings.local.json` (git-ignored, at the repo root) holds `disableAllHooks`, any `hooks` key, or a `permissions.defaultMode` of `bypassPermissions`. A selftest case (`R-81.case`, new file).
7. **F-22 (d):** the build role watches `$CLAUDE_CONFIG_DIR` (when set) by name, beside `~/.claude`.
8. **F-23 / D6-3:** the rule globs gain `.claude/skills/engine-rust/*`, `.claude/agents/engine-reviewer.md` and `engine/CLAUDE.md` (it doesn't exist yet; the lead writes it in step 4). check-frozen's `OUTSIDE` list and `rules_selftest.py`'s `OUTSIDE` copy list gain the existing ones, so `RULES.sha256` covers them. Not the rest of `.claude/agents/**` or `.claude/hooks/**`: the toolkit syncs those. guard-test rows for each path in every role.
9. **SCAFFOLD §3** (draft text in `notes.md`, don't edit SCAFFOLD.md): the rule-file list as it now is; "What the guard checks" for the new forms; "Its limits" gains F-6 (git pathspec magic, `--work-tree=`) and F-7 (a computed `cd "$D"`), and loses the overstatement F-22 (e) names.

## 4. Before you finish

- In WT: `engine/tools/check-frozen --write-rules`, then `engine/tools/guard-test`, then `engine/tools/check fast`, then `engine/tools/rules-selftest/run R-80 R-81`. Record each exit and summary line in `notes.md`.
- `git -C WT add -A && git -C WT diff --cached > /Users/jswob/projects/trop/tmp/session8c/agents/guard/patch.diff` (the lead applies it with `git apply`).
- `notes.md`: per item, what changed (file, function) and its proof (the guard-test rows or the case names); the SCAFFOLD §3 draft; one §7 row update per rule touched (R-79, R-80, R-81, R-100), as text; anything you chose not to do and why.

## 5. Your final message

About 10 lines: the counts (guard-test checks before and after, selftest cases added), each item done or not, the exits of the four runs, and anything the lead must decide.
