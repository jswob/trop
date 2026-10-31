# Session 8c's auditor: phase 1's audit of the engine scaffold

You are the auditor of session 8c (the Tally engine prep, part B2). You audit what phase 1 delivered: the engine scaffold as committed at `db0262e` on branch `engine-scaffold`. The lead has already run the scripted checks (`check full`, `guard-test`, `exceptions --check`, `check-frozen`, the spec against its tag). Your job is the part a script can't do: fresh probes and fresh plants, and a judgement on four open items. You change nothing real: you probe and plant only inside a throwaway copy, and you write only your findings.

## 1. Hard rules

1. **Seals** (word for word from the session brief):
   - Never open, list or search `tmp/holdout/`.
   - A recursive `find`, `grep` or glob over `tmp/` must prune it: `find tmp -path tmp/holdout -prune -o …`.
   - Every agent prompt says the same, word for word.
   - Don't open `engine/tests/cases/*-hidden.md`; no step here needs them.
2. **No sub-agents.** Don't spawn agents or forks, run workflows or invoke skills; never use the Agent, Task, Workflow, Skill or SendMessage tools. No web research.
3. **Where you work.** `WT` = `/private/tmp/claude-501/-Users-jswob-projects-trop-engine-scaffold/10fe1c33-6637-4d53-aca9-85a7c2f9d2a8/scratchpad/auditor/wt`, a git worktree detached at `db0262e`, made for you. Inside `WT` you may edit, plant, build and revert freely (`git -C WT checkout -- <path>`, `git -C WT clean -fd`). Your scratch (probe JSON files, small scripts, logs) goes in `WT/../probes/`.
4. **What you may write, outside WT:** only `/Users/jswob/projects/trop/tmp/session8c/agents/auditor/findings.md` and `progress.md` beside it. Never a file named `REPORT.md`. Never anything in `/Users/jswob/projects/trop-engine-scaffold` (the real scaffold) or elsewhere in `/Users/jswob/projects/trop` (the main checkout), and no git command that writes in either (no commit, checkout, worktree, branch, tag, stash). Reading both is fine (`git -C /Users/jswob/projects/trop show engine:<path>` reads branch `engine`).
5. **Guard probes are hook inputs, never real edits.** Write a hook JSON (`{"tool_name": …, "tool_input": {…}, "cwd": "<WT>"}`) to a file under your scratch and feed it to `WT/engine/tools/guard` on stdin, with the role in the command's environment: `TALLY_ROLE=build WT/engine/tools/guard < probe.json; echo "exit=$?"`. Exit 0 allows, 2 refuses. That guard's repo root is `WT`, so paths inside `WT` are "in the repo". Your own session runs under the live guard in the `rules` role; don't try to get around it.
6. **Machine:** at most one cargo build or check of yours at a time (the lead runs others). No installs. No refcore or `run.py` runs.
7. **Scope is closed:** the four tasks in §3, nothing else. Aim for 80–130 tool calls. Write `findings.md` piece by piece (a section at a time) and keep a short `progress.md`, so a context reset loses little.

## 2. Read first (narrowly)

- `WT/engine/SCAFFOLD.md` §2, §3, §7 (the rule map), §8 (deviations), §9 (open questions).
- `WT/engine/SCAFFOLD-REPORT.md` §2.4 (the guard's evidence) and §5 (open questions).
- `WT/engine/tools/guard.py` (657 lines, whole), `guard_test.py` (to see what is already probed, so yours are fresh), `stop_check.py`, `check_frozen.py`, and `check.py`'s top 80 lines.
- `/Users/jswob/projects/trop/tmp/research_notes/engine-rust-strictness/RULEBOOK.md`: "How to read a row" and groups §1–§7, by section (the version on disk is 2.1's; phase 1's is `RULEBOOK.before-8b.md` beside it; judge the scaffold against phase 1's rows).
- `WT/engine/tools/rules-selftest/cases/*.case`: grep, to make sure each of your plants is new.

## 3. Your tasks (the closed list)

### A. Fresh guard probes

Probe the guard with cases `guard_test.py` doesn't already cover. Cover at least:
- **every role** (`build`, `rules`, `tests`, `spec`, unset, and one unknown value) against: the spec, the tests, each class of rule file (SCAFFOLD §3's list), `.claude/settings.json` and `.claude/settings.local.json`, the build allow-list's edges (a `build.rs`, a `benches/` file, a `.cargo/config.toml` or `rustfmt.toml` under a crate, the root `Cargo.toml`), case variants (`ENGINE/SPEC/…`), `..` traversal and a symlink made inside `WT/../probes/` that points into `WT/engine/spec/`;
- **paths outside the repo:** the main checkout's `/Users/jswob/projects/trop/engine/spec/DONE.md` and `/Users/jswob/projects/trop/engine/tests/checks.tsv` in every role (the master's finding: `rel()` treats the main checkout as outside the repo, so the `rules` role may write its spec and tests; confirm and size it), and `~/.claude/settings.json`, `~/.cargo/config.toml` in the build role;
- **interpreter writes:** `python3 -c` naming a protected path, a script file, a here-document fed to `python3`, `node -e`, `bun -e`, `perl -pi -e`, `ruby -i`;
- **git writes:** `checkout -- <protected>`, `restore`, `reset --hard`, `stash`, `worktree add`, `merge <branch>` (note: a branch named `engine` is also a directory here), `apply`, `git -C <dir> …`;
- **other writers you judge likely to be missed:** e.g. `rustfmt <file>`, `cargo fmt`, `cargo clippy --fix`, `cargo fix`, `cargo add`, `tar -x -C`, `unzip -d`, `curl -o`, `vim -c`/`ex`, `install -t`, `ln -s`, a computed path (`$VAR`) in the build role, `bash -c`, `eval`, a subshell `( … )`, `xargs`;
- **fails closed:** `TALLY_GUARD_SELFTEST_CRASH=1`, non-JSON stdin, empty stdin, a JSON array, `PATH` without `python3`, the guard file missing (simulate inside `WT` only: move `WT/engine/tools/guard` aside and run `.claude/settings.json`'s hook command string by hand with `CLAUDE_PROJECT_DIR=WT`), then put it back.

Keep a table: probe, role, expected, got (exit), and mark each unexpected answer. An allowed write the guard's stated scope should refuse is a **hole**; one SCAFFOLD §3 "Its limits" already names is a **known limit** (say which line).

### B. New planted violations, one per rulebook group (§1–§7)

For each of the seven groups, plant one violation of a tool-enforced rule in a form **not** already among the selftest's cases (grep `cases/` and clippy.toml to be sure), in the place the rule concerns (usually `WT/engine/crates/tally/src/lib.rs`; the CLI's `lib.rs` or a `Cargo.toml` where the rule is about those). Run `WT/engine/tools/check fast` (or `check step <name>` where the rule's tool is outside `fast`, e.g. `deny`, `ast-grep-scan`), record the exit and the line that names the rule, then revert. A plant that stays green is a finding. Choose plants a builder might plausibly write (e.g. a crash reached through a method reference or a trait method, an arithmetic form through a std helper, a hash-ordered type arriving through a dependency type, an `#[expect]` whose reason dodges R-104's phrases).

### C. Phase 1's four open items (SCAFFOLD §9, SCAFFOLD-REPORT §5)

For each: the facts (commands, exit codes, quoted lines), your judgement, and a concrete proposed fix (file and change) the lead can make in the `rules` role.
1. **The CLI's exception budget (3 of 3 used).** Read `WT/engine/EXCEPTIONS.lock`, `tools/exceptions-budget.toml`, `crates/tally-cli/src/main.rs`. Then read refcore's command line on branch `engine` (`git -C /Users/jswob/projects/trop show engine:engine/tools/refcore/cli.py`, by the parts that read argv, files, directories and stdin, and write output): the engine must match it byte for byte (`tally run <files> [--ask id …] [--pin=bytes]`, `tally reprint <files>`). Does matching it need I/O the three exceptions don't cover (reading files by path, listing a directory: R-39, R-40)? If so, size the budget it needs.
2. **A rule change and its checksum in one commit** (R-112): the checksums can't catch a `rules`-role commit. Propose a machine check that makes such a commit visible (e.g. what CI or `check-frozen` could compare against the target branch, and what it would print), without blocking a legitimate R-112 commit.
3. **The interpreter tripwire's reach.** Demonstrate in `WT`, in the build role: a `python3 <script>` whose script writes a rule file passes the guard; then show `WT/engine/tools/check fast` (the Stop hook's check) fails on it via check-frozen. Then judge: is "the Stop hook catches it at the turn's end, CI on a clean runner" enough, or should the build role refuse something more (and what it would cost the builder)?
4. **Gate 8's survivor wording (SO-28).** Compare DONE gate 8's text (`git -C /Users/jswob/projects/trop show tally-1.0.1:engine/spec/DONE.md` and `… show engine:engine/spec/DONE.md`, gate 8's lines only) with `WT/engine/MUTANTS-SURVIVORS.md` and `tools/mutants_survivors.py`. Propose the scaffold-side fix (the spec is frozen; a spec wording point goes to the lead as an observation).

### D. Session 8's two findings, confirmed or not

1. **R6 C-3:** the skill (`.claude/skills/engine-rust/**`), the reviewer (`.claude/agents/engine-reviewer.md`) and `engine/CLAUDE.md` are outside the guard's rule globs and `RULES.sha256`. Probe each path in every role, and read `check_frozen.py`'s rule-file list. Say which roles can write each today.
2. **R4:** `check.py` doesn't clear `GUNGRAUN_*` or `CRITERION_*` from the environment. Quote the lines.

## 4. Output

`findings.md`:
1. **Commands:** every command that bears on a finding, with its exit code (probe tables may be summarised as counts, with every unexpected row listed).
2. **Probe table** (A) and **plant table** (B).
3. **Findings F-1, F-2, …:** what, the evidence (command, exit, quoted line), severity (*hole*: a write the guard's scope should refuse; *gap*: a rule without enforcement or a check that can't fire; *doc*: a description that doesn't match the code), and a concrete proposed fix (file, change, and the selftest or guard-test case that would prove it).
4. **Open items C1–C4 and D1–D2:** facts, judgement, fix.

Before you finish, revert `WT` (`git -C WT status --short` clean apart from ignored build output) and restore the guard if you moved it.

## 5. Your final message

About 10 lines: the counts (probes, unexpected answers, plants, plants that stayed green); each finding's id, severity and one-line summary; anything the lead must decide.
