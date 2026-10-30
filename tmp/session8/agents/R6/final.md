<!-- saved by tools/save_final.py from agent agent-aab4adb40a1ed0a94.jsonl (R6) -->

I've written `/Users/jswob/projects/trop/tmp/research_notes/engine-prep/R6.md` (about 35 KB, a little over the ~30 KB target). Raw skill texts and the skills.sh search results are in `tmp/research_notes/engine-prep/scratch/R6/`. Everything was read as data: nothing installed, compiled or run.

**The three findings that matter most:**
1. **Nothing exists to adopt.** Anthropic publishes no Rust skill; its marketplace's only Rust entry is the `rust-analyzer-lsp` 1.0.0 plugin. The registry the `skills` CLI uses (skills.sh, CLI 1.7.4) lists about 100 Rust skills. I judged 12 candidates and read 5 in depth: Apollo, leonardomso, actionbook, Trail of Bits and Microsoft's guidelines.
   - Each recommends something the rulebook bans: `anyhow`, `unwrap_or_default`, `saturating_*`, fast hashers, rayon, `insta`, or `expect`.
   - actionbook's pack is the clearest injection case. It has a UserPromptSubmit hook that injects "MANDATORY" text on nearly every prompt, an unpinned `npx -y …@latest` MCP server, and no licence.
   - The 2026-10-22 decision stands: create our own skill and borrow ideas only.
2. **`CLAUDE.md` and skills are advice, not enforcement.** Claude Code's docs say a rule that must always hold belongs in a hook.
   - `engine/CLAUDE.md` is a nested file. A session started at the repo root doesn't load it at launch, and after compaction it reloads only on demand.
   - A skill's body loads only when the skill is called, and only its first 5,000 tokens come back after compaction.
   - So the plan is a build-role SessionStart hook (startup and compact) that prints `engine/CLAUDE.md`, plus `/engine-rust` at session start and a `paths:` frontmatter.
   - The skill's `paths:` doesn't fire for files in a sibling worktree, so build sessions must start inside the scaffold worktree.
3. **The guard has a gap.** The scaffold guard's `RULE_GLOBS` and `RULES.sha256` don't cover `.claude/skills/engine-rust/` or `engine/CLAUDE.md`.
   - The build role still can't edit them directly, because they're off its allow-list.
   - But `check-frozen` misses a change made through an interpreter, and app-role sessions can edit both freely.

**Decisions:**
- **D6-1:** create a local `engine-rust` skill. Borrow, without copying text: leonardomso's rule-per-file Bad/Good format, Apollo's and Microsoft's "measure first, name hot paths" method, Trail of Bits' bug shapes for the reviewer, and turso's and zed's readability rules.
- **D6-2:** the never-miss rules stay in the lints and hooks. `engine/CLAUDE.md` is printed by the SessionStart hook, and the skill is invoked by name.
- **D6-3:** add the skill and `engine/CLAUDE.md` to the rule globs and checksums, changed only by the `rules` role under R-112.
- **D6-4:** the skill's eval is 9 planted cases from BUILDER §8 (two must end in a STOP-REPORT), graded by scripts, run with and without the skill, 3 runs each, in Part B. The user sets the bar.
- **D6-5:** a read-only `engine-reviewer` subagent that preloads the skill. Install `rust-analyzer-lsp` in Part B as a speed aid only.
- **D6-6:** approve the 2.1 rows. That is new rows R-114 (the largest-remainder split written once), R-115 (parts sum to the balance, ⚖), R-116 (holes built in one module), R-117 (a path statistic's accumulator outside the cap) and R-118 (a report of derived orderings). Plus amendments to R-16, R-40, R-47 and R-76.

**What you need to know:**
- **C-1, a contradiction:** BUILDER §2 checks the 4,096-digit cap "after every operation". Tally 2.1's change 2.1-15 (LANGUAGE l.2103) exempts a path statistic's accumulation, so BUILDER is stale there.
- **C-2:** R-76 and R-58 still say 87 codes. 2.1 has about 95–98 (my regex count, an inference).
- **R-40 changes meaning.** The engine hash orders files by full path code point (ENGINE l.118), which differs from walking each directory sorted by name: `a/b.tally` and `a.x.tally` swap places. It needs rewording.
- **Spec observations:**
  - **O-1:** ENGINE §9 doesn't list "parts sum to the balance" as an invariant, though `ENGINE-INVARIANT` is defined as "§9 fails".
  - **O-2:** joint models nest with no limit and no refusal code, so a walk over members must be a worklist, not a counted-depth recursion.
- **Pressure P-1, for the performance agents:** R-36/R-37 ban even fixed-seed hash maps, while the memo reaches 2.3 M cells. A sanctioned, never-iterated memo map is enforceable through the already-listed `iter_over_hash_type` lint. I sketched it; I didn't pick it.
- **Versions** were re-checked today: `skills` 1.7.4, Apollo 1.1.2, leonardomso 1.5.1, Trail of Bits 1.1.2, the LSP plugin 1.0.0, rust-analyzer 2026-10-26. Every source I tried was reachable.
- The 2026-10-22 memory note says the guard "already refuses" these files to the build role. That holds only for direct edits; neither file is checksummed (C-3).
