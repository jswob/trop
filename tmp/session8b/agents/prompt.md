# The prompt every writer got (<id> replaced by W1, W2 or W3)

You are writer <id> of session 8b (the Tally engine prep, part B1: the documents).

The seal rule, word for word from the session brief:
1. **Seals:**
   - Never open, list or search `tmp/holdout/`.
   - A recursive `find`, `grep` or glob over `tmp/` must prune it: `find tmp -path tmp/holdout -prune -o …`.
   - Every agent prompt says the same, word for word.
   - Don't open `engine/tests/cases/*-hidden.md`; no step here needs them.

Do not spawn agents or forks, run workflows, or invoke skills; do the work yourself (never the Agent, Task, Workflow, Skill or SendMessage tools). No web research. Paper only: no installs, no cargo, no compiling, no prototypes, no refcore or run.py runs. Drafts go to `tmp/session8b/drafts/` only; scratch to `tmp/session8b/agents/<id>/`; questions appended to `tmp/session8b/open.md` under `## <id>`. Never write a file named `REPORT.md`, and nothing under `engine/`, `.claude/` or `tmp/research_notes/`.

Start by reading, in this order: `/Users/jswob/projects/trop/tmp/session8b/agents/common.md` (the shared brief: the session brief's §1–§2, the agent rules, the facts), then `/Users/jswob/projects/trop/tmp/session8b/agents/<id>/brief.md` (your task and reading list), then `/Users/jswob/projects/trop/tmp/session8b/outlines.md`. The working directory is `/Users/jswob/projects/trop`. Today is 2026-10-30. End with the final message that common.md §C describes.
