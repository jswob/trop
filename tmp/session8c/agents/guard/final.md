<!-- saved by tools/save_final.py from agent agent-a09228ab285604be7.jsonl (guard) -->

All nine items are done and all four runs pass. Nothing is committed. The patch is `/Users/jswob/projects/trop/tmp/session8c/agents/guard/patch.diff` (8 files, +956 −74), and `git apply --check` on a clean `05103df` tree exits 0. Details, proofs, the SCAFFOLD §3 draft and the four §7 rows are in `/Users/jswob/projects/trop/tmp/session8c/agents/guard/notes.md`.

**Counts:**
- guard-test went from 770 checks to 1503, 0 failed.
- I ran the new `guard_test.py` against the old `guard.py` and `settings.json`: 391 checks failed, which covers every new refusal row.
- 7 selftest cases added: a new `R-81.case` with 5 cases (`local-settings-*`), and 2 in `R-80.case` (`builder-skill-byte`, `reviewer-agent-byte`).

**Items:** F-1, F-2 to F-5, F-8 (guard side), F-9 (guard side), F-10 with F-22 (b), F-11, F-22 (d) and F-23 are done. The SCAFFOLD §3 text is a draft in `notes.md` §2; I didn't edit `SCAFFOLD.md`.

**Runs:**
| Run | Exit | Result |
|---|---|---|
| `check-frozen --write-rules` | 0 | 160 files |
| `guard-test` | 0 | 1503 checks, 0 failed |
| `check fast` | 0 | green, 7 steps |
| `rules-selftest/run R-80 R-81` | 0 | 23 cases green, ban coverage 455/455 |

**For you to decide:**
1. **`cargo fmt` in `engine/` is now refused in every role**, because the workspace includes T's runner. The builder has to name its packages: `cargo fmt -p tally -p tally-cli`. That deserves a line in `engine/CLAUDE.md`. The looser option is to judge cargo by the cwd's manifest only and leave the runner and benches to the checksums.
2. **Some stricter choices than the brief:**
   - The build role is refused in every other checkout, not just judged alike.
   - A missing `.git` fails closed.
   - Reading a spec file with `awk` or `perl -ne` is now refused outside the spec role; that follows from F-3.
   - `-m "costs $5"` in double quotes is refused in the R-100 roles.
   - I let `-m "$(cat <<'EOF' … EOF)"` pass, since its text is in the command and gets scanned.
3. **R-81's text in the RULEBOOK** still says `.claude/hooks/**` and `.claude/agents/**` are protected. The code and my §7 draft protect only `settings.json` and the engine's own skill and reviewer agent; `settings.local.json` is checked by check-frozen. The rule text needs the same correction.
4. **Rewrite `RULES.sha256` again** in each R-112 commit you split this into, and again in step 4 once `engine/CLAUDE.md` exists. `check-frozen` will pick that file up automatically.
5. **Not done:**
   - The Stop hook string still `exec`s its shim; F-10 named only the PreToolUse and ConfigChange hooks.
   - The auditor's cargo `--target-dir`/`--manifest-path` refusals in the build role were outside my list.
   - F-6 and F-7 are logged as limits in the draft, not fixed.
   - The F-8 CI grep and F-9's check-rules side are yours.
