# reader scratch notes (running)
PERF read whole, BOUNDS read whole.
- PERF l.147 "walk restart (§4.3)" -> restart is §4.2 (l.315); §4.3 is "What one turn costs". broken xref (must)
- PERF l.382 "address hashed as UTF-8 (SO8-4)" states a choice L leaves unsaid -> check L 5.16
- PERF l.51 vs BUILDER: check 'counters' naming
- PERF 8.2 P8-14 def misses "2.27 M (partial)" vs outline ">= 2.27M" ok
- loadtsv: 17 rows L1-6,8-18 OK matches P8-L list
- BOUNDS: 53 rows in table; header says 49+4 ok. row 54 not added (O-W1-5).
TODO: check E l.198 quote, LOAD-DEPTH 8 (L 2.3), LOAD-FORKS 64, LOAD-SCENARIO 4, LOAD-ASIF 4, REQ-PATHS 100000, RUN-DIGITS, RUN-UNDECIDED 2^16, N-1..N-14 cites
BUILD-ORDER read whole. placement_check: faults 0, 634/634, 138 globs name 569. Spot 15 placements ok (VAL-01 M1, REC-05 M2, PTY-04 M5a, OWN-10 M5b, PTY-05 M5c, LH-16-light M6, D-13~D1 M7, b05_balance M9, LAW-02 M4, d05 M5a, a05_ania M5b...). own scan of U:/I:M/I:X/hardset/light/I:S for owned by/use/fill/paths/to-u vs milestone: only 2 expected M0 load refusals.
- BO l.147? no. PERF l.147 "(§4.3)" wrong.
- BO names num-bigint l.94 (crate) ; std's scoped threads l.316; Symbol(u32), Table<I,T> l.91 (Rust API) - convention says none  (should/note)
- PERF P8-2/P8-3 "Mac, at the tags from S" vs BO: P8-2 first at M1 (1x), 10x at M9; P8-3 only M9. (should)
- PERF P8-8 "from the milestone that builds the residual ledger" vs BO M8 (fitting loop). (should)
- BO l.3 "D1-1..D6-5" vs PERF l.14 "D1-1..D6-6" minor
- edge table puts OWN-08/REC-04/RULE-03 in M1/M2 but placed M5a (explained l.29/252) note
RULEBOOK read (header, groups 2,5,8 + amended rows). counts: 131 rows; keep 115/report 9/drop 7 match summary & per-group table (script rb_count.py; R-66 has escaped pipes so cell count 9 but is keep); 99 tool+16 review = 115 reconciles; original 98/8/7. amended list of 11 all present with 'I approve' 2026-10-30. R-114..118, R-119..131 all have enforcement + answer.
- R-122 adds 'sanctioned file' escape route not in D5-2 (note)
SPEC CHECKS PERF:
- MUST: ENGINE.md l.230 cites 'PERFORMANCE.md §2.3' for F updated in place; draft §2.3 = Monthly checkpoints, F structure is §2.5. (L l.1102,1454 cite §2.2 ok (balance reads/prefix sums in 2.2); E l.154,351 §4 ok; E l.400 §3 ok; L l.2102 §6 ok; DONE gate12 §8 ok)
- PERF l.147 '(§4.3)' should be §4.2
- N-11 says window hole 'up to each place's next reading' but N-12 + E 4.1 l.177 say hole only inside window. N-11 'or the place's first day' not in E (E: every day).
- l.382 UTF-8 note (SO8-4) - states a choice L leaves unsaid
- l.257 O-W1-1 misquote: L l.1400 says 'makes it [the projection] a hole', draft says 'the decision' 
- caps all verified (LOAD-DEPTH 8, ASIF 4, SCENARIO 4, FORKS 64, REQ-PATHS 100000, RUN-DIGITS 4096, UNDECIDED 2^16)
- E l.189-211 N-list line cites verified exact. L l.942,943,1805,1088 verified.
- 5.6 MJ/ type-1 / 1e-6 verified (L l.1523-1525)
