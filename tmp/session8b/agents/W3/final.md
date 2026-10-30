<!-- saved by tools/save_final.py from agent agent-ac529aa6db53c81a0.jsonl (W3) -->

W3 has finished. Both drafts are complete full copies, and neither original was touched.

**Files written**
- `/Users/jswob/projects/trop/tmp/session8b/drafts/RULEBOOK.md` (about 80 KB; the original is 55 KB)
- `/Users/jswob/projects/trop/tmp/session8b/drafts/BUILDER.md` (about 51 KB; the original is 22 KB)
- `/Users/jswob/projects/trop/tmp/session8b/agents/W3/changes.md` (9 KB): one line per changed or added row and section, each with its source.
- Scratch: `progress.md` and `count_rows.py` in `/Users/jswob/projects/trop/tmp/session8b/agents/W3/`.
- Two questions appended to `tmp/session8b/open.md` under `## W3`.

**RULEBOOK: 131 rules in 8 groups**
- **Totals:** 115 keep (about 99 by a tool, 16 by review), 9 report-only, 7 dropped. The script reproduced the original's 98/8/7 before counting the new file. The Summary has a per-group table.
- **New rows:** R-114 to R-117 go in group 2 (numbers and money), and R-118 (a report) in group 3. R-115 carries the ⚖ mark and "checked at run time".
- **Amended rows (11):** R-16, R-37, R-40, R-42 (all five points), R-47, R-58, R-73, R-74, R-75, R-76 and R-97. Each keeps its old text and adds "**Amended 2026-10-30 (G1 item n; source):**". Its answer cell keeps the old answer and adds "I approve" (2026-10-30, item n).
- **Rows raised from report-only:** R-73, R-74 (for nesting only) and R-75 now show "report → keep" in the Rec. column. R-58 and R-76 say "recount at 8c with `tools/xref_check.py`", as asked.
- **Group 8, readability (R-119 to R-131):** one row per D5 decision, split where enforcement differs. Each cites R5's RD and RS ids and records its G1 group (A, B or C). RS-6, D5-3's nesting limit and D5-4's kernel citations are not repeated as rows, since they are the amended R-73, R-74 and R-75.

**BUILDER**
- Updated §2, §3, §4, §7, §8, §9, §11 and §12 as the outline lists, and added §13 on performance (fences, counters and `--stats`, instruments and what blocks, profiles, profiling recipes, with R4's tool versions).
- Each sentence G1 reversed now has a "*Changed 2026-10-30 (K8-n)*" line: K8-1, K8-2, K8-3, K8-4, K8-5, K8-6, K8-8, K8-12 and K8-13.
- §8 keeps its two-column table and gains a second, four-column table for the ten new bans: what blocks, the workaround it tempts, what to write instead, and when to stop and report.
- Beyond the outline, I fixed three places that would otherwise have contradicted the amended rows:
  - §6's "87 codes" now points to the recount (K8-13).
  - §7's CLI paragraph now sorts full relative paths by code point (R-40, K8-12).
  - §4's "ENGINE l.141" now cites E §3.2 l.155.

**Open items**
- **O-W3-1:** three ast-grep reports that R5 picked have no planned tool. They are explicit lifetimes (RS-5), long iterator chains (RS-7), and `#[inline(always)]` / `#[cold]` outside a fence. Plan B2.2 lists none of them, and decisions.md never names RS-5. R-124 and R-125 say "ast-grep report", as R5 does; R-130 says "listed for review".
- **O-W3-2:** no rulebook row keeps two things out of the release build:
  - the `counters` feature (D4-8, K8-15), and
  - gate 6's test-only switch that reverses or shuffles the order the blocks run in (D3-6).

  The question is whether to amend R-51 or R-93 through R-112's procedure. For now no row changed.

**For the lead**
- RS-2 ("no own generics", R-124) will flag `Table<I, T>` (D2-4). The row names it as the one expected hit, to be reviewed once.
- BUILDER's [Tally syntax] items cite tally 1.0.1 line numbers (l.71, l.198, l.228, l.233), and 2.1 has moved them (`LOAD-DEPTH` is now at L l.325). I did not recheck those items; the header says so.
- `tools/xref_check.py` doesn't exist yet: `engine/tools/` holds only parsecheck, refcore and refcore_fast.
- R4's gap goes into §13.3 as a note. `check` doesn't clear `GUNGRAUN_*` or `CRITERION_*` environment variables, so one of them could loosen a limit. It's for the scaffold step (B2), not a rule change.
- In R-117's evidence, "R-13" means the language decision L calls R-13, not this rulebook's R-13; the row now says so. The two id spaces collide.
- No sources contradicted each other beyond what NOTE §2 already settles.
- `tmp/holdout/` was never opened, listed or searched.
