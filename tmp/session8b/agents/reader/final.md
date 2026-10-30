<!-- saved by tools/save_final.py from agent agent-a55dcc7d04e02e1a2.jsonl (reader) -->

Reader done. I wrote `/Users/jswob/projects/trop/tmp/session8b/reader.md` (21 113 bytes, findings only) and scratch under `tmp/session8b/agents/reader/`. No draft was changed and nothing else was written.

**Counts.** 22 distinct findings: 4 must, 9 should, 9 note.
- PERFORMANCE: 3 must, 5 should, 4 note.
- BOUNDS: 1 note.
- BUILD-ORDER (and the TSV): 1 must, 4 should, 5 note.
- RULEBOOK: 2 should, 2 note.
- BUILDER: 3 should, 1 note.
- A cross-draft finding is counted once per draft it names.
- No draft contradicts a G1 pick.

**Sound:**
- The placement check reports 634/634 placed once, 0 faults. 15 spot checks and my own feature scan agree with the milestones.
- All 36 P8 ids match outline §C.
- The RULEBOOK counts reconcile by script: 131 rows, keep 115, report 9, drop 7. All 11 amended rows and R-114..R-131 carry their enforcement and "I approve" (2026-10-30).
- Every R-n that BUILDER cites exists with the same meaning.
- The §2 non-empty-day list, §3.3 trimming, §5 merge rule and §9 caps all check against L and E, including line numbers.

**The three that matter most:**
1. **F-19 (must).** ENGINE.md l.230 sends the reader to `PERFORMANCE.md` §2.3 for the structure that is "updated in place, in walk order". In the draft §2.3 is the monthly checkpoints and that structure is in §2.5. E is frozen, so PERFORMANCE has to keep the structure at §2.3. Every other incoming reference to PERFORMANCE holds.
2. **F-20 (must).** In PERFORMANCE §2.2, N-11 says a window movement's balances are holes "up to each place's next reading". N-12 and E §4.1 l.177 say a window's hole lasts only inside the window. N-11 should cover hole days only.
3. **F-6 (must).** BUILD-ORDER says the placement TSV sits "beside this file" and cites `placement_check.py`. Session rule 2 does not let the session land either file in `engine/spec/`. The frozen document would cite files that are not there.

**Two smaller points worth a look:**
- **F-5 (must).** PERFORMANCE l.147 cites §4.3 for the walk restart; it is §4.2.
- **F-7 (should).** PERFORMANCE measures P8-2 and P8-3 "from S", while BUILD-ORDER defers the 10x run to M9. P8-2 is the row that triggers the D1-5 seam.
