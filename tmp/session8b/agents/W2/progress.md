# W2 progress
- [x] read common.md, brief.md, outlines.md, BUILD-ORDER.md, STALE rows
- [ ] DONE.md, decisions, NOTE §1-2, plan §2
- [ ] tests columns, tags, README, TESTING, load.tsv, boundaries headings
- [ ] coverage, CHANGES 1-140, BUILD-NOTES
- [ ] placement TSV + script
- [ ] draft sections
- [x] inputs read (DONE, decisions, NOTE 1-2, plan 2, tests README/TESTING, load.tsv, tags, boundaries headings, CHANGES 2.1, BUILD-NOTES, L/E headings)
- design: S X base; M0 load refusals; M1 X forecast; M2 replay + forks step; M3 ops (no checks); M4 law breadth; M5.1 fill/many parties, M5.2 parts, M5.3 joint members; M6 exact fns; M7 MC + thread module; M8 eval; M9 perf + not-run load checks
- placement rule: unit=edge map; household=max(floor, question edges w/o PTY-05, file scan); I:M = 1.0.1 case lists raised by scan; not-run -> M9
- scratch: scan_baselines.py; next: gen_placement.py
- [x] gen_placement.py writes drafts/BUILD-ORDER-placement.tsv (634 rows): S=4 M0=10 M1=90 M2=26 M3=0 M4=77 M5a=166 M5b=79 M5c=40 M6=36 M7=21 M8=0 M9=85
- [x] placement_check.py written (needs the draft's appendix globs + Counts: line); try_globs.py for pure globs
- decisions: forks = M2's last step; thread module + parallel paths/timelines = M7; members/asks parallel = M9; D1-7 step 2 (pruning) = M5c's last step
- [ ] draft BUILD-ORDER.md: header, rules, summary, S, M0..M9, T's, keeps, appendix
- [x] draft complete (   47588 B); open items O-W2-1..4 appended to tmp/session8b/open.md
- placement_check.py output (2026-10-30 22:54):
```
checks.tsv: 634 checks; placement: 634 rows
per milestone: S=4 M0=10 M1=90 M2=26 M3=0 M4=77 M5a=166 M5b=79 M5c=40 M6=36 M7=21 M8=0 M9=85 | total 634
draft globs: 138, naming 569 of 634 checks
faults: 0
exit 0
```
