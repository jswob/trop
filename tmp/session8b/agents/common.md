# Session 8b writers and reader: the shared brief

You are one of the agents of session 8b (the Tally engine prep, part B1: the documents), working for the session's lead. Your own task is in `tmp/session8b/agents/<your id>/brief.md`. This file holds what every agent shares: the session brief's §1 and §2, copied word for word, then the rules for agents (where §2 speaks to the lead, part A says what it means for you), and the facts every agent needs.

---

## 1. The goal

The user, before session 8:

> "I would want to keep building this rust project properly but I have no experience in this land so I want to ensure that claude will follow the best practices and that it will keep performance and redability in mind (I think performance will be a core thing in this whole engine)."

**Session 8 settled the design.** At its gate G1 the user approved all nine items as recommended, saying "I approve" to each (`tmp/session8/decisions.md`). Those decisions stand. Your job is to write them into the documents the build reads, and into the skill that instructs it. No decision is re-opened here. A real gap goes to G2a as a question.

## 2. Hard rules

1. **Seals:**
   - Never open, list or search `tmp/holdout/`.
   - A recursive `find`, `grep` or glob over `tmp/` must prune it: `find tmp -path tmp/holdout -prune -o …`.
   - Every agent prompt says the same, word for word.
   - Don't open `engine/tests/cases/*-hidden.md`; no step here needs them.
2. **What you may write:**
   - `tmp/session8b/`, everything else of yours. Drafts go to `tmp/session8b/drafts/` first.
   - After G2a only:
     - `engine/spec/PERFORMANCE.md`, `BOUNDS.md`, `BUILD-ORDER.md`, `STALE.md`, `DONE.md` (one approvals row and gate 12's evidence wording) and `SHA256SUMS`;
     - the one stale-list line in `engine/README.md`;
     - `tmp/research_notes/engine-rust-strictness/RULEBOOK.md` and `BUILDER.md`, each copied first to `<name>.before-8b.md`;
     - `.claude/skills/engine-rust/` and `.claude/agents/engine-reviewer.md`.
   - Never: any other file under `engine/` (LANGUAGE, ENGINE, CHANGES, VERSION, the models, `engine/tests/`, `engine/tools/`); the scaffold worktree; `tmp/cases*`; `tmp/design2/`; any other `.claude/` file.
3. **Paper only.** No installs, no `cargo`, no compile, no prototype, no refcore runs. The user's rule is plans, not spikes.
4. **No new meaning.** The three documents carry no language meaning (G1 item 9). If a sentence would need to say what a construct means that LANGUAGE or ENGINE doesn't say, stop: it is a spec question (DONE rule 3). Log it in `tmp/session8b/open.md` for G2a; never write it into a document. "Discover now, decide once": collect, don't patch mid-draft.
5. **Sources:** session 8's NOTE, R1–R6 and decisions. No new web research, except to re-check a version you name (on today's date).
6. **The user's principles** (memory notes): "Engine is universal", "Engine questions", "Tally explicit", "Drift strategy", "Design process", "Heavy runs", "Memory cap", "Plan, not prototypes".
7. **Agents:**
   - at most 3 at once, closed scopes: writers Opus high, the reader Sonnet;
   - every prompt forbids sub-agents, forks, workflows and skills, and carries the seal rule;
   - save each final with `tmp/design2/tools/save_final.py` and audit each transcript with `tmp/design2/tools/audit.py`. Pass both the transcript path under `~/.claude-personal/projects/-Users-jswob-projects-trop/<your session id>/subagents/`;
   - agents can't write a file named `REPORT.md`.
8. **Chat:**
   - The user is a programmer (TypeScript, JS, ESLint, backends) who can't read Rust. Gloss every Rust term the first time, with an analogy they know.
   - About 5 plain lines per message.
   - One question at a time, problem before pick.
   - Only the user says when to move on.
9. **Compaction:**
   - `tmp/session8b/progress.md` is your memory; take each entry's time from `date` and log each budget reading (`python3 -I -B tmp/design2/scratch/lead/usage2.py <session id>`).
   - Write deliverables piece by piece. Never re-read a big input whole.
10. **Git,** on branch `engine`, only after G2a's go:
    - one commit for `engine/spec/` and `engine/README.md`;
    - one for the skill and the reviewer;
    - one for `tmp/session8b/` and the rulebook and BUILDER with their `.before-8b` copies, force-added as session 8 did (`git add -f`).
    - No AI attribution (CLAUDE.md). Never push, never `--no-verify`.


---

## A. The rules for agents

1. **Seals:**
   - Never open, list or search `tmp/holdout/`.
   - A recursive `find`, `grep` or glob over `tmp/` must prune it: `find tmp -path tmp/holdout -prune -o …`.
   - Every agent prompt says the same, word for word.
   - Don't open `engine/tests/cases/*-hidden.md`; no step here needs them.
2. **No sub-agents.** Do not spawn agents or forks, run workflows, or invoke skills. Do the work yourself. Never use the Agent, Task, Workflow, Skill or SendMessage tools. No web research: every source you need is local.
3. **Write only what your brief names:** your drafts under `tmp/session8b/drafts/`, your scratch under `tmp/session8b/agents/<your id>/`, and your questions appended to `tmp/session8b/open.md` under your own heading (append only; never edit another heading). Write nothing anywhere else: never `engine/`, `.claude/`, `tmp/research_notes/`, `tmp/cases*`, `tmp/design2/`, the scaffold worktree, or any git state. Never write a file named `REPORT.md`.
4. **Paper only.** No installs, no `cargo`, no compiling, no prototypes. Don't run refcore, `engine/tests/run.py` or any interpreter job: the figures you need are in `tmp/session8/shape.md`, `engine/tests/load/`, and the reports. `python3 -I -c '…'` for arithmetic you show in the text is fine, and so is a short `python3 -I -B` script under your scratch folder that reads text files (counting, cross-checking ids).
5. **No new meaning** (§2 rule 4 above). If a sentence would need to say what a construct means that LANGUAGE or ENGINE doesn't say, don't write it: append it to `open.md` as a question, and write the draft around it (a placeholder that names the open item, e.g. "(open: O-W1-2)"). Never answer a question by guessing. A choice the G1 decisions already made is not open: write it as decided.
6. **Write piece by piece.** Write your draft a section at a time (Write the first part, then Edit or append), so a context reset loses little. Keep a short `progress.md` in your scratch folder: what is done, what is next. If you find your context was reset, read your `progress.md` and the draft's headings, never the big inputs whole again.
7. **Read narrowly.** Big inputs are read by their summaries and the sections or line ranges you need, never whole (LANGUAGE.md is 356 KB, ENGINE.md 61 KB, DECISIONS.md 274 KB). The documents your brief says to read whole, read whole.
8. **Scope is closed.** Do your task; don't survey beyond it. Aim for about 60–110 tool calls.
9. **The user** is a programmer (TypeScript, JavaScript, ESLint, backends) who can't read Rust. The spec documents are read by the user, the builder and the gate session: plain sentences, short paragraphs, tables where they help, every Rust term glossed once with an analogy (a crate is an npm package; clippy is ESLint; `Result` is a typed error return you must handle). House style: the existing documents' (short sentences, lists, ids cited, no filler).

## B. What every agent needs to know

- **The engine** is a standalone Rust library and binary: a reality (a set of Tally text files) and an ask in, answers out. It is not built yet. The spec is `engine/spec/` (tally 2.1, frozen): `LANGUAGE.md` (L: what a construct means) and `ENGINE.md` (E: how the engine computes it). `PERFORMANCE.md`, `BOUNDS.md`, `BUILD-ORDER.md` (and `OPERATIONS.md`) are tally 1.0.1's, frozen as stale (`engine/spec/STALE.md` lists the sections 2.x reaches). This session rewrites the first three for 2.1; `OPERATIONS.md` stays stale.
- **The reference interpreter** `engine/tools/refcore/` (Python, written from the spec alone) is the oracle: the engine must print the same bytes (DONE gate 10). It is frozen.
- **The acceptance contract** is `engine/spec/DONE.md` (12 gates; gate 6 determinism with 1 and 6 workers; gate 8 mutation score on the kernel; gate 12 performance at PERFORMANCE §8's rows on the reference machine).
- **The tests** are in `engine/tests/`: `checks.tsv` (634 checks: 154 unit probes `U:`, 479 integration `I:`, refcore's own `R:all`), `load/load.tsv` and `load/benchmarks.md` (the full-size rows `L:n`, never run on refcore), `map/tags.tsv`, `cases/boundaries.md`.
- **The seams** (`engine/BUILD-NOTES.md`): J joint members as their own contexts; U a range slot on every unknown; N the symbol table keyed by (path, sort, name); F the memo keyed by key values; R "is it random" per context; the labels' cost paid only where a kind opts in.
- **Session 8 (part A) settled the design.** At gate G1 the user approved all nine items as recommended, saying "I approve" to each (`tmp/session8/decisions.md`, 2026-10-30, 22:06–22:11). Ids: D1-1..D6-6 (the decisions), K8-1..K8-17 (the lead's settlements of contradictions, NOTE §2), SO8-1..SO8-10 (spec observations, NOTE §3), R1–R6 (the six research reports in `tmp/research_notes/engine-prep/`). The measured shape of refcore's heaviest checks is `tmp/session8/shape.md` (one page; its units carry over to the engine).
- **Phase 1 (2026-10-22)** wrote `tmp/research_notes/engine-rust-strictness/RULEBOOK.md` (113 rules the user approved group by group, ids R-n) and `BUILDER.md` (how to write the engine).
- **The outlines** for every draft, and PERFORMANCE §8's fixed row ids (P8-1..P8-15, P8-L1..P8-L18, P8-M1..P8-M4), are in `tmp/session8b/outlines.md`. Read it whole; it is your plan.
- **The user's principles** (from the lead's memory notes):
  - *Engine is universal:* a standalone black box, "reality as code"; it handles every variant (many parties, currencies, any law), not one household. No household or app questions in engine design; support all variants.
  - *Tally explicit:* every meaning comes from written text, no implicit defaults; a missing statement is a refusal.
  - *Discover now, decide once, build once, check by machine:* no rules or forms added mid-batch; decisions taken once, from the evidence, then built.
  - *Lean on prior art; reduce problems to their solved forms; no unnecessary work.*
  - *Plans, not prototypes.* Separate what is proven, what is only designed, and what is a gap; be candid about design gaps.
  - *Heavy runs are engine size targets;* no interpreter job over 8 GB; at most 6 heavy jobs on the machine; a test budget of 90 s hard, 60 s soft.

## C. Your final message

About 10 lines: the files you wrote with their sizes; what each draft carries that the outline asked for, and anything it doesn't (with why); the open-list items you added, by id; anything the lead must know (a contradiction between sources, an outline instruction you couldn't follow).
