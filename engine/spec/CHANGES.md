# Changes: tally 2.1.1 → tally 2.1.2

tally 2.1.2 is the second patch of the frozen `tally 2.1` (`DONE.md` rule 1, `engine/README.md` § Changing it): it corrects the definition of `tally 2`, and files still state `tally 2` (`LANGUAGE.md` §9). Its gaps are the Amendments issues found while milestone S was built and gated (#23 by #13; #29, #30, #31, #32 by the S builder; #33, #34, #38, #39 by the S gate's review; #43, #44, #46 by T's STOP-REPORTs) and S's fixed budgets (#47). The user ruled them at the spec round's gate G-SP1 on 2026-11-03 (#31 on 2026-11-01, #33 and the budgets on 2026-11-02); each row quotes the ruling. Where refcore differed from the text it now follows it, with its tests.

| # | Issue | Where | The gap | The change | The ruling | Found by |
|---|---|---|---|---|---|---|
| 2.1.2-1 | #29 | L §2.3 | `LOAD-DEPTH` counted brackets and `if`s only, so a read chained by `from` or `ties by` (`bal(x, d) from bal(y, e) from …`) nested with no bound the spec named; refcore counted no link (9 links parsed, 300 crashed it with a `RecursionError`), the engine stopped at its own counter (64) with an internal `NotBuilt` | the depth at a point also counts the `from` and `ties by` terms it is inside, one each, as an `if` counts | "A: each link counts one (Recommended)", 2026-11-03 | the S builder at G3 (issue #26) |
| 2.1.2-2 | #23 | E §3.1; L §8.3 | past parsing, load printed the first refusal its checks met, and nothing ordered those checks (2.1.1's "Left open"): refcore ran its own passes and took moves and fills by id, so of a def at line 9 and a move at line 16 that both refuse, refcore printed the move's | E §3.1 "The order of the checks": the members of a joint model first, then five passes (headers; declarations; rows; the items' own checks, one sort at a time: moves, fills, defs, bands, asks; the checks across items), each walking its sections or items by full path and line; L §8.3 points to it. The residual is written: two refusals in one item, and those of a pass that walks the rows of an input or a graph, print in the order the check meets them | "C: write refcore's order (Recommended)", 2026-11-03 | issue #13's patch (tally 2.1.1) |
| 2.1.2-3 | #30 | L §8.5; E §3.1 step 6 | `WARN-UNREAD` named places nothing reads, by the example "an account no total includes", and refcore and the engine warned only inputs, defs and sources; nothing said whether a leg into a place is a read | a place no line names warns, `place <x> is read by nothing`: a read, any leg (a movement's, an observed row's, a pad's), a binding over its kind or a row whose value it is names it; a row keyed by it (`opening`, a reading), an `anchor` or a `covers` line doesn't. The example is now "an account no line names". Only `place` sections are declarations (a thing a version enters is a fact). `outside` places need no exception: one exists to be a leg's other end, so `world` in a household is named by every income and expense leg, and one no leg names is a stray line; in refcore's own tests the 55 lines this adds are all such (copied `shop`, `world` and `unexplained` lines), and the 8 others are accounts stated with at most an `opening` row | "A: warn only if never named", 2026-11-03 | the S builder at G4d (issue #26) |
| 2.1.2-4 | #33 | E §6.3 Order; E §4.1 | the residual ledger's last ties go by the movement order of §4.1, keys in index order and strata as numbers; refcore compared that order's text, so of `rent[f3]` and `rent[f11]` (key positions 2 and 10) it printed `rent[f11]` first | the movement order's terms are compared each as its own type, never as text (E §4.1, restated at §6.3); refcore sorts by one typed key (`moves.order_key`) | "Spec wins, fix after S (Recommended)", 2026-11-02 | the S gate's review (finding B4, issue #26) |
| 2.1.2-5 | #39 | L §2.5; E §4.1 | no text ordered a binding's movements: by L §2.5's letter (things in the text order of their stable ids) `fee[…]` would precede `rent[…]` whatever their days; refcore and the engine went by day, then the movement order's text | a binding over movements runs in the movement order of E §4.1: by day (a window by its first day, a hole day last), then the order's other terms, typed (#33's key). E §4.1 now places a day's observations and adjustments before its plans and its pads after its decisions, as refcore did for fewer than 10 strata (its pads' stratum 9 is now above every stratum) | "B: by day, then typed order (Recommended)", 2026-11-03 | the S gate's third pass (finding T4, issue #26) |
| 2.1.2-6 | #32 | refcore only (L §5.14.1, E §4.3 step 1, unchanged) | a projection read at a decision's point counts only the movements not yet booked; refcore compared the point `sw\|000000` with a later decision's id `sw_b` as one text (`_` sorts before `|`) and left `sw_b`'s act out. Shown by a run: `sw_b`'s refusal of every place was missing from `sw`'s projection (`q: 99.00 PLN`); named `sx`, the same model refused `q` | none: refcore compares the point's move id apart from its instance number. The engine compares walk points already | "the engine follows the spec, refcore is fixed" (the standing ruling), 2026-11-03 | the S builder at G7, by reading refcore (issue #26) |
| 2.1.2-7 | #38 | E §4.3 step 1; L §5.14.1 (`flows` row) | inside a projection's walk, `flows` of a place over a span starting before the projection's day summed only that walk's log, while `bal(x, d)` and `flows` of a decision read the walk it continues; nothing said which | the projection's walk continues the walk it starts from: a read of days before its start (`bal`, `flows` of a place or decision, a binding over a decision's movements) takes those days from that walk up to the point it starts after. A yearly cap re-checked in a projection from June sees January to May | "A: spec wins, see Jan–May (Recommended)", 2026-11-03 | the S gate's third pass (finding T3, issue #26) |
| 2.1.2-8 | #43 | refcore only (L §5.4, §5.7, §5.14, unchanged) | a declared currency is a key of the prelude's `currencies`, so a leg of unit kind `currencies` carries money in it; refcore refused it at load (`LOAD-UNIT … is money PLN, not a unit of currencies`), though its run and the engine took it | none: refcore's load check accepts money in a declared currency on a `currencies` leg, and still refuses anything else | the S-close coordinator's ruling under the user's delegation, 2026-11-03; the user at G-SP1, 2026-11-03 (row 10): "Confirm both (Recommended)" | T's STOP-REPORT for #41 (chunk 5) |
| 2.1.2-9 | #47 | P §8 (opening, §8.2, §8.4), §10; `BUILD-ORDER.md` S, M1 | every number of §8 was proposed until S; S measured P8-14 and the micro rows (`perf/S.md`, 2026-11-02), and §8.2 set C rows' instruction limits at S though no Linux instrument counts a check's instructions | fixed: P8-14 ≤ 1 s, ≤ 512 MB, ≤ 30 400 def misses (S: 760.9 ms, 154.4 MB, 25 312), days stepped reported (926, 906 non-empty), no instruction limit (the Mac's 11.86 G reported; a C row's limit waits for a Linux instrument); P8-M1 ≤ 55.5 ns and ≤ 1 048 Ir, P8-M2 ≤ 2.20 µs and ≤ 6 123 Ir, P8-M3 ≤ 7.81 µs and ≤ 87 004 Ir (× 1.2 of S's figures; Ir from CI at S's gated head cf55c5a); P8-M4's budget moves to M1 with `low`, P8-5 and F5 (S: 72.9 ms, 795 765 251 Ir). §10 gains S's answers (U-1, 3–6, 11, 12, 15–18). `perf-budgets.toml` and the CI job's `continue-on-error` are the rules step's | the user, 2026-11-02: D4-6 "Fix as proposed (Recommended)" and P8-M4 "Defer to M1 (Recommended)"; G-SP1, 2026-11-03 (row 8): "S's gated head cf55c5a (Recommended)" | S's timing run and gate (issue #47) |
| 2.1.2-10 | #31 | E §9 invariant 1 | a leg whose place is a hole books its known side (L §5.14, E §4.2), but invariant 1 named only a leg booked by its ends as left out; refcore counted the hole end as booked, summed the known end alone, and refused every ask `ENGINE-INVARIANT conservation …` | invariant 1 also leaves out a leg whose place is a hole; refcore follows. The issue expected a read of another place the hole could give to be a hole too; E §4.2 ("results … about other places, stand") and L §5.14 ("never a made-up place") keep it, so only reads through the unknown side are holes | the user, 2026-11-01: "refcore wrong, fix after S (Recommended)" | the S builder at G6 (issue #26) |
| 2.1.2-11 | #34 | E §5 steps 1, 3, 4 and 5; L §5.14.3 (Detection stays) | E §5 step 3 didn't say what a re-measurement gives when it is refused (the instance at V's cut, or a leg's amount on either side), nor when the `when` is a hole; refcore and the engine skipped the step and measured the next one from the last measured state, so such a change was dropped with no trace, against L §5.14.3's "never silently paid or silently unpaid" | a refused re-measurement makes the leg's adjustment refused, with the refusal; settled or not, it refuses its `change`, its remainder, and the `flows` and balances an unfulfilled one makes `HOLE-CHANGED` (L §8.4: a refusal outranks a hole). A hole `when` makes a hole adjustment, as a hole amount does. The next step measures against that state, so its adjustment is refused, or a hole, too, the step that ends the instance included. A booking refused at κ₀ stays refused and records nothing, as one refused at its `when` already did. L §6.5 change 3 stands (its 2026 advances and settlement, refused at booking, record nothing). Change 5's `e1` becomes the `HOLE-CHANGED` holes: `home_2026` makes the 2026-10 euro wages' `when` a hole, and `ana_berlin_job` ends them (four hole adjustments, none settled; refcore's test `L6_5_change5`, was 0.00 EUR on 07-24) | the user at G-SP1, 2026-11-03 (row 3): "A: the refusal spreads (Recommended)", with the hole-`when` sibling; and, 2026-11-03: "1: refused stays refused (Recommended)" | the S gate's review (finding O4, issue #26) |
| 2.1.2-12 | #44 | refcore only (E §5 step 5: one sentence; the rule is E §3.2's) | a settlement leg refused by the turned-round rule (`RUN-RECORD`) refused the read that first settled the adjustments; refcore left them linked with nothing recorded, so a later read in the same ask took the `HOLE-CHANGED` hole, and swapping the reads swapped the answers. The spec and the engine refuse every read | linking is all or nothing: a refused settlement leg links none of the adjustments, and every read that counts them meets the refusal, whichever is read first; E §5 step 5 says so | the S-close coordinator's ruling under the user's delegation, 2026-11-03; the user at G-SP1, 2026-11-03 (row 10): "Confirm both (Recommended)" | T's STOP-REPORT for #41 (chunk 8) |
| 2.1.2-13 | #46 | L §5.10 (The sign rule), §8.4 (`RUN-PLACE`); E §4.2 step 5 | the sign rule is checked at three points, the readings at the start and at the day's end among them, and a balance that breaks it is refused from there; refcore and the engine tested only a pad's other place, so a `stated` reading beyond its place's rule (a claim at −5.00 PLN), padded or anchored, left the place at a balance its kind forbids, unrefused (T's `g9/sign`) | a reading's own place is tested at its point like any balance, at the start (point 1, with the day's first batch), after a decision (point 2, as it is booked) and at the close (point 3): a pad that leaves it beyond its rule is refused with the point's legs, and from there so are the places it touches; an anchor, which books no leg, refuses the place's balance from there, `<place> is <b> after its reading at <instant>` (a new message in `RUN-PLACE`'s row, no new code). An `opening` always anchors (L §5.19), so one beyond its place's rule is refused the same way. The audit of points 1 and 2 found them as point 3 (the reading's own place untested, a pad's other place tested) and nothing more | the user at G-SP1, 2026-11-03 (row 4): "2: spec wins, refused (Recommended)", with the audit of the start-of-day and after-a-decision points | T's STOP-REPORT for #41 (chunk 9) |

---

# Changes: tally 2.1 → tally 2.1.1

tally 2.1.1 is the first patch of the frozen `tally 2.1` (`DONE.md` rule 1, `engine/README.md` § Changing it). It corrects the definition of `tally 2`, and files still state `tally 2` (`LANGUAGE.md` §9). Its gaps were logged by the tests' session T1 (issue #5, `engine/tests/INTERFACE.md` §5 and §8), measured on refcore, and are issue #13. The user decided the order at T1's gate G-T on 2026-10-31 ("Sort warnings") and approved the patch's scope the same day ("Yes, within scope (Recommended)"). refcore follows, with 9 new tests and one changed (1 124 tests). Every gate is re-run on it. No value, hole or register line changes. What prints changes only in the order of warnings (in one file too: by code, then by text, not as load finds them), in which load refusal prints and the place it names when the files are given in another order than their full paths, and where refcore crashed.

| # | Where | The gap | The change | Found by |
|---|---|---|---|---|
| 2.1.1-1 | L §8.5 | warnings printed in the order the files were read: of the suite's 133 multi-file checks given in another order than their full paths, 124 print their `WARN-UNREAD` lines in another order, 24 of them in a whole output that is compared; L §9 says the order of files changes nothing, and §8.5 said no order | a run's warnings print sorted by code, then by their text (code point), whatever the order of the reality's files | T1 (INTERFACE §5, gap I-5); G-T "Sort warnings" |
| 2.1.1-2 | L §8.3 | with two files that each fail to load, the refusal printed was that of the file given first | Which refusal prints: load takes the files in the order of their full paths (code point, E §3.1 step 1), reads every file before it splits any into tokens and splits every file before it parses any, and prints the first refusal it meets; so of several files refused at one of these steps, the first by full path prints, with the first refusal the step meets in it, from its first line down | T1 (INTERFACE §5, gap I-5, its synthetic pair); G-T "Sort warnings" |
| 2.1.1-3 | L §8.3 | a path the run is given that doesn't exist had no code, and refcore crashed (a traceback, exit 1) | `LOAD-PATH`: `<path> doesn't exist` | T1 (INTERFACE §8, gap I-1) |
| 2.1.1-4 | (none) | `reprint` of a file that isn't UTF-8 crashed refcore (`main` decoded without `run`'s catch) | none in the spec: `LOAD-PARSE`'s row covers bytes that don't parse; refcore's `reprint` refuses the file as `run` does, `LOAD-PARSE cannot parse: file is not UTF-8 (<path>)` | T1 (INTERFACE §8, gap I-2) |
| 2.1.1-5 | `VERSION`, `SHA256SUMS`; `DONE.md` approvals | — | `tally-2.1.1`; the approval logged | — |

**Left open.** Past parsing, load still prints the first refusal its checks meet, and the spec doesn't order those checks: `ENGINE.md` §3.1 lists its steps, and refcore runs its own order, some checks by id. So the user's rule ("the first in full-path order, then by line") is the spec's at reading, splitting and parsing only. Ordering the later checks needs E §3.1, which this patch doesn't touch.

---

# Changes: tally 2.0 → tally 2.1

tally 2.1 adds owners and batches as labels on amounts, from session 6's decided items, designed in session 7 and written in session 7b (2026-10-30). It is **opt-in**: a text with no `owned` or `batched` kind, and none of the small items' forms (notes after a one-line def or a place, a negative record field, a day key where a date is expected), means and prints what it did on 2.0, except as the rows of `MIGRATION.md` § 2.0 → 2.1 say (a declared `parties`, a kind named by a clause word, a kind test that now narrows). The reasons are `DECISIONS.md`'s "tally 2.1 records" (R-1 to R-17, G1, the four picks); the round's files are `tmp/session7/` and `tmp/session7b/`. One row per change:

| # | Where | Change | Record |
|---|---|---|---|
| 2.1-1 | L §2.2 | kind clause words `owned`, `batched`, `takes`, and `prorata` as the clause word ending a `takes` clause; `of` as a construct word; a part's built-in fields `owner`, `batch`; `parties` among the built-in ids | R-1, R-2, R-4 |
| 2.1-2 | L §2.3 | the parts clauses as kind lines (`owned by D [ties by e]`, `batched`, `step u`, `takes k for w [v V] order … [prorata [ties by e]]` or `… prorata [ties by e]`); `of q` on `bal`, `value`, a leg's amount and a reading row; `note` lines after a one-line def and a place section | R-1, R-4, R-6, R-11; picks 2, 3 |
| 2.1-3 | L §2.3 rule 7 | an `of` belongs to the read its production gives it | R-1, C-14; pick 3 |
| 2.1-4 | L §2.4 | a one-line def with notes means the long form and prints as it; no canonical form changes | R-11, C-12 |
| 2.1-5 | L §3.1 | a key of a day index stands where a date is expected, meaning `start_of(k)` | R-14, G1 Q-6 |
| 2.1-6 | L §3.5 | parties are things under the prelude kind `parties`, the subkinds the model's own; only a party owns parts; the holder is a model's own input | R-2, G1 Q-1; pick 1 |
| 2.1-7 | L §3.6 | rewritten: a place owned by shares or by parts, never both; parts, the arrival rule, the leaving rule, relabelling, net worth over `of` reads; the examples use the prelude's `parties` | R-1, R-3, R-4, R-5, R-9, R-10 |
| 2.1-8 | L §5.7 | the parts clauses on place kinds and `LOAD-PARTS`; a kind test narrows; parts and lifespans; `parties` in the prelude | R-1, R-2, R-17, C-10, RC-9 |
| 2.1-9 | L §5.10 | parts as a place's state; `bal … of q`, `value … of q`; `min 0`; a gain follows the units; the parts' cost | R-1, R-7, R-8, G1 Q-3 |
| 2.1-10 | L §5.12, §5.13 | pointers: a place's takes reuse the fill and the largest-remainder split within one outflow | R-4, RC-2, RC-3 |
| 2.1-11 | L §5.14 | legs at a place that keeps parts: the take as a fill over arcs, ties, two outflows never sharing, the batch's arrivals as supplies, undecided takes; whose parts a leg takes is the order's; the relabel; the leg read | R-3, R-4, R-5, R-10, C-1, C-4, RC-3, RC-7 |
| 2.1-12 | L §5.14.1 | the arrival rule as a reader of start-of-day parts beside decisions; the leg read's class | R-3, R-10, M-15, RC-6 |
| 2.1-13 | L §5.14.2 | takes in the first batch, in the order of their reads; a cycle broken where no take changes, else `RUN-CYCLE` with a plan in it, or a hole among observed rows | R-4, C-11, RC-6 |
| 2.1-14 | L §5.14.3 | labels are walk state at cut K; the one exception to decision 22 for a past decision that reads parts | R-3, RC-1, G1 Q-4; pick 4 |
| 2.1-15 | L §5.16, §7.1 | the 4 096-digit cap covers a value at a key, not a path statistic's own accumulation | R-13 |
| 2.1-16 | L §5.19 | readings at a place that keeps parts: stated and opening parts, pads, zero-gap anchors, gaps, a stated part's own gap | R-6, RC-4, RC-5, G1 Q-5; pick 2 |
| 2.1-17 | L §6.1 | `parties` shared by every member; two parties one only by `same`; `same` refused on a place that keeps parts; `of` across a `same` | R-2, C-9 |
| 2.1-18 | L §8.1 | `HOLE-PART`; the hole part and its possible labels; a total stays known | R-8, RC-5 |
| 2.1-19 | L §8.2, §8.5 | `WARN-LABELS`, listed in the register | R-3, M-9, RC-8 |
| 2.1-20 | L §8.3 | `LOAD-PARTS` | R-1, R-6 |
| 2.1-21 | L §8.4 | `RUN-LABEL`; `RUN-NO-RULE`, `RUN-TIE`, `RUN-CYCLE`, `RUN-DOMAIN`, `RUN-PRORATA`, `RUN-LIFESPAN` and `RUN-RECORD` widened for parts | R-1, R-3, R-4, R-6, RC-2, RC-3, RC-9 |
| 2.1-22 | L §10 | what 2.1 can't say: signed parts, `same` on a place that keeps parts, and the rest of the round's pressure list; lots may be a `batched` place | R-1, `pressure.md` |
| 2.1-23 | E §4.2 | parts in the day walk: the first batch's takes, a decision's take at its point, the arrival rule run at booking, lifespans after the day's whole order | R-3, R-4, RC-6, RC-9 |
| 2.1-24 | `DECISIONS.md` | the tally 2.1 records | all |
| 2.1-25 | `MIGRATION.md` | 2.0 → 2.1 | all |
| 2.1-26 | `VERSION`, `SHA256SUMS` | `tally-2.1` | — |
| 2.1-27 | (none) | a negative record field (item 7.2) needs no spec text: the grammar allows it, and the interpreter's refusal is a fault | R-12 |
| 2.1-28 | (none) | a joined household's cost (decision 6) is an engine requirement, `engine/BUILD-NOTES.md` § J | R-15 |

**Fixes after the spec round** (faults and contradictions only, each with a line in the round's `progress.md`):

| Row | Where | What | Found by |
|---|---|---|---|
| 2.1-F1 | L §5.7, `MIGRATION.md` | a field of a subkind read where no kind test narrows the variable stays refused as in 2.0, `LOAD-FIELD`; the spec round had named `LOAD-LEG`, which would change a 2.0 text's output against the opt-in rule | writer 11's F-1 |
| 2.1-F2 | L §5.19, §8.4 | a reading at a day's start that pads and states parts, at a place the day's first batch also moves, is refused (`RUN-RECORD`), the reading at the end of the day before written instead: its pad takes with the batch (E §4.2), so the stated parts had no one point to compare at | writer 11's F-2 |
| 2.1-F3 | E §6.3 | the residual ledger's balance line shows each stated part's gap and the unstated rest's, so "every such gap is listed with the reading" (L §5.19) has a form | writer 11's F-3 |
| 2.1-F4 | L §3.6, §8.4 | a share below 0 is `RUN-LABEL`: a part is never below zero (§10, signed parts) | writer 11's F-4 |
| 2.1-F5 | L §3.6 | in an arrival's split the hole part of hole shares ranks after every party, which `ties by` can't rank ("a known share is never lost to a hole") | writer 11's F-5 |
| 2.1-F6 | L §5.19, §8.3 | a stated part below 0, and two stated parts of one reading that overlap, are `LOAD-PARTS` | writer 11's F-6 |
| 2.1-F7 | L §8.1 | the hole part's questions are worded for a gap, an unstated rest, a cycle of observed rows, an unknown outflow and a stated part's open batches | writer 11's F-7 |
| 2.1-F9 | L §8.1 | the question of a stated batch's part whose owners are open, at a place that keeps pairs | writer 11's F-9 |
| 2.1-F10 | L §5.10 | a place alive before S has no known parts before its first anchor, as it has no balance: one hole part of the hole balance, which takes reach, and no sign checked against it | the migrator (A's joint account) |
| 2.1-F12 | L §8.1 | a read `of q` of a hole part, like an order term over it, is known where every possible label gives it one value: a part whose every possible label is q's is read as q's | the migrator (A's cycle question) |
| 2.1-C7b-1 | E §5, E §9 invariant 5, L P-8 | decision 22 restated with L §5.14.3's one exception (parts) | the contradiction read (verifier) |
| 2.1-C7b-2 | L §5.14.1, §8.3 LOAD-READS | running reads allowed in an arrival rule and its defs, as L §5.14.1 says | the contradiction read (verifier) |
| 2.1-C7b-3 | L §5.14.1, §8.4 RUN-CYCLE, RUN-FUTURE | an arrival rule's read of its day's end or later is RUN-CYCLE; RUN-FUTURE is a decision's | the contradiction read (verifier) |
| 2.1-C7b-5 | L §5.10, §5.14, §5.14.2, §5.19 | "batch" kept for the day's first batch, "part" for a label's amount (members, pieces) | the contradiction read (verifier) |
| 2.1-C7b-6 | L §3.6 | w is the outflow's movement; an outflow is one leg | the contradiction read (verifier) |
| 2.1-C7b-7 | L §5.12 | a split by origin may be written as a fill or as parts | the contradiction read (verifier) |
| 2.1-C7b-8 | L §5.7 | a late label to a party on its last day refuses that part's place, as E §4.2 step 5 says | the contradiction read (verifier) |
| 2.1-C7b-9 | L §3.6 | WARN-LABELS and the relabel at a place that keeps parts, not only an owned one | the contradiction read (verifier) |
| 2.1-C7b-11 | E §4.2, L §3.6 | the arrival rule's law at its movement's day (`at m.day`) | the contradiction read (verifier) |
| 2.1-C7b-13 | L §5.19 | a zero-gap anchor leaves the parts but for its stated parts' gaps | the contradiction read (verifier) |
| 2.1-C7b-18 | L §3.6 example | `inherited` beats `moved`, as the prose says | the contradiction read (verifier) |
| 2.1-C7b-19 | L §3.1 | a day range `by 1` is a day index | the contradiction read (verifier) |
| 2.1-C7b-20 | L §5.14.1, §8.4 `RUN-CYCLE` row | an arrival rule's `bal(y, d start) of q`, where a reading of y at d start pads (its pad takes with the day's first batch), has no one point and is refused `RUN-CYCLE`, as F2's reading is (the user's option (c); a later round may give start pads their own point) | the contradiction read (verifier), G2 Q-4 |
| 2.1-C7b-21 | L §8.1 HOLE-PART row | the row lists every source of a hole part | the contradiction read (verifier) |
| 2.1-C7b-22 | L §5.14 | a take's demands include the start pads, and a later reading's pad at its point | the contradiction read (verifier) |
| 2.1-G2-2 | L §4.5 | seeing a row known on or before K isn't applying it: an observation or reading dated on day K or later books nothing (as §5.19 and E §4.2 step 4 say); wording only, no behaviour change | the blind judge (S4-04), G2 Q-2 |

---

# Changes: tally 1.0.1 → tally 2.0

tally 2.0 is a new version of the language, not a patch. `LANGUAGE.md` §9 allows a new version only with a migration, a mechanical rewrite of every `tally 1` model, tested by running both versions on every model and case. The migration is `MIGRATION.md`, and its runs are recorded in `regress.md`. Files of the new version state `tally 2`. `tally 1` (1.0.1, tag `tally-1.0.1`) stays as released.

The definition was redesigned in one round, session 4 of the engine plan, from the evidence of tally 1.0.1's case set (session 3). The reasons are in `DECISIONS.md`. How the round was run, what it shows and what it doesn't are in the round's `REPORT.md`. This file says what changed, where the old-to-new table lives, how the contradictions were found and settled, and which gaps stay open.

**New in the package:**
- `LANGUAGE.md`, `ENGINE.md`, `DECISIONS.md`, `coverage.md` and this file, rewritten for tally 2;
- `MIGRATION.md`, the table from tally 1.0.1 to tally 2 (L §9);
- `models/`, `amendments/` and `montecarlo/` as tally 2 texts (carried by `tools/migrate.py` and the migrators);
- `VERSION` (`tally-2.0`) and `SHA256SUMS`.

**What it doesn't change:**
- The expected answers in `expected/`, `amendments/expected/` and `montecarlo/`, except the keys shown wrong. Those wait for the user (§ Open gaps, "Keys") and change only with the user's ok.
- `OPERATIONS.md`, `PERFORMANCE.md`, `BOUNDS.md` and `BUILD-ORDER.md`, which are carried as they are and marked stale, with the sections the decisions reach (`STALE.md`). Nothing in this round checked them.

## What changed

**The model of the world** (`DECISIONS.md` D-1 to D-15).
- Everything a model talks about is a declared thing of a declared kind, with a lifespan.
- Every statement about a thing is a dated fact, with when it holds (`governs`) and when it became known (`known`).
- A movement is a thing of an event kind, with named legs in any unit and declared reads.
- Law, money in places, forecasts and randomness are this one idea.
- Nothing defaults: a missing statement is refused, with a code from L §8.3.

**Concepts:** sixteen become fourteen (`regime`, drafted, was dropped at G2: GD-1).

| Concept | Change | Record |
|---|---|---|
| source, index, axis, band, given | kept (index gains the built-in period indexes; a band row can be replaced) | D-12, D-13 |
| version | changed: `governs` is required (`always` when timeless); `known` per row; an overlapping row says `replace` | D-3, D-12 |
| kind | **added**: lifespans, subkinds, dated memberships of plain kinds | D-1, D-2 |
| place | changed: a thing of a declared place kind (unit, sign rule, valuation); 1.0.1's seven place kinds are in the prelude | D-1 |
| input | changed: states over spans, readings at instants, distributions, codes, calendars | D-3, D-11, D-12, D-13 |
| def | changed: its law date is written (`at e` or `at reader`); exceptions are written pairs (`beats`) | D-4, D-12 |
| fill | kept; also lot matching (L §5.12) | D-2, D-5 |
| move | **added**: replaces `pay` and `record`; observed movements are rows of an `observed` input | D-2, D-5, D-6, D-8 |
| scenario | changed: an overlay of ordinary versions, compared by keyed difference | D-10 |
| ask | changed: a stated law date; values, partial answers, differences, readers, explanations, checks, path statistics | D-4, D-7, D-10, D-15 |
| `pay`, `record` | merged into `move` and `observed` inputs | D-2, D-8 |

**Constructs:**
- **Removed:** `draw`, `bound` and its outcome, `promised`, `free`, `holds`, `plan_low`, `next_on`, `phase`, `excepts`, `arrears` lines and `correction` versions, the `-` atom, the path key `p in path`, L §4.3's default law dates (rules 2–4), parse rule 1, inline figures other than 0, 1, −1 and key offsets, and `weeks`.
- **Merged:** `worth` into `value`; `kind()` and `currency()` into `x in K` and `unit(x)`; scenario versions into ordinary versions; rate bases and day counts into code values.
- **Added:** the full list with each construct's record is in `core.md` § "Removed, merged, added" and in `DECISIONS.md`, with A-1 to A-51 after G1 and the review's verdicts.

**The counts** (`tools/counts.py`):

| Count | 1.0.1 | 2.0 |
|---|---|---|
| Concepts | 16 | 15 |
| Keywords | 47 | 50 |
| Built-in names | 68 | 75 |
| Contextual words | 83 | 126 |
| Built-in ids | 14 | 21 |
| Constructs used, over the carried texts (`coverage.md` table 1) | 173 | 133 |
| Constructs used, over all 196 cases | — | 171 |
| Built-in words (`coverage.md` table 2) | 113 | 120 |
| Refusal and warning codes defined | 66 | 91 |

**The migration** (`MIGRATION.md`). Each changed construct is one row, with its class:
- **automatic:** rewritten by `tools/migrate.py`, with the same answers by construction;
- **warned:** rewritten, with the lines a writer checks listed;
- **manual:** the meaning changed, and the row lists the models and asks it reaches.

Of the 79 models, 67 needed only the script's warned rules and 12 were partly rewritten by hand (`regress.md`). Every change of meaning is listed in `MIGRATION.md` with the texts it reaches.

**The answers kept** (`regress.md`, judged by run on the reference core).
- The 79 models: 2 170 key rows match.
- The amendments household: 258 key cells match. 12 variant cells whose tally 1 key is a hole print otherwise under tally 2's reading (§ Open gaps, "Keys").
- The probes: all 88 keyed rows match.
- The rows that don't match are listed under § Open gaps.

## The gap log

### 1.0.1's gaps, closed

Every item tally 1.0.1 failed or was questioned on is one line of the round's trace (`trace.md`). That is 186 rows of the case set's gap log, the 29 concerns LC-1 to LC-29, and 1.0.1's 30 open gaps (this file's predecessor, § Open gaps). Each line is closed by a decision, deferred, or not a language matter, with its status in tally 2 and where it is shown.
- Concerns: 27 closed, 2 deferred.
- 1.0.1's open gaps: 23 closed, 7 not language matters (OPERATIONS or tool matters).
- Spec findings S:1–S:19: 11 closed, 3 expressed, 2 resolved, 1 awkward, 2 withdrawn.
- Probes: 83 expressed, 9 awkward.
- Questions: 20 expressed, 15 awkward, 1 breaks, 1 can't express, 2 case suspect.
- Observations: 20 expressed, 10 awkward, 3 can't express (two of them tool gaps), 1 case suspect, 2 closed.

### How the contradictions of the new text were found

1. **The review.** One ground-up review read every decision taken after G1 against four written principles (`review/principles.md`) and decided each: derive, admit or drop (`review/ledger.md`).
   - A-rows: 48 derived, 3 dropped.
   - Pressure lines: 23 derived, 11 dropped.
   - Open questions: 8 derived.
   - Word groups: 4 dropped, 1 kept.
   
   A fresh critic then checked the rewritten sections (`review/critic/`).
2. **The writers and judges of the visible cases.** They logged each place where the text couldn't say a row or said two things: the S:2xx and S:3xx rows of `rows/*.settled.md`. Each was settled by the lead in a spec round, or left on the open list.
3. **The reference interpreter's writers.** They logged each place where the spec was silent or contradicted itself, in their hand-overs. A silence stays a `REFCORE-UNSPECIFIED` refusal in the interpreter until the spec settles it.
4. **The machine checks.** Seven judges compared the interpreter's output with the keys on snap8, and four of them rechecked on snap11. Scripted reruns repeated the judged runs on the final snapshot, snap14; REPORT §5.1 names the runs left out. Each difference was triaged: an interpreter fault (fixed by the interpreter's writers, each fix with a test), a text's error, a key, or a contradiction. Spec rounds 17, 17b and 17c (R17-01 to R17-11) settled the contradictions they found. Each edit states what the spec already implied, with no new form.
5. **The contradiction read.** Three readers each read one share of the sections against all of L and E and `DECISIONS.md`, and listed 142 candidates. Three verifiers, one per report, checked every quote and decided each one: 135 confirmed (61 outcome, 69 wording, 5 stale), 7 not confirmed, and 15 that the definition doesn't settle. Round 18 (below) applied 130 edits from 108 confirmed candidates, each in its single home; duplicates were applied once. The 15 unsettled ones are on the open list.

### The settlements after the build

The rule after the interpreter's one rebuild was that only interpreter faults and contradictions are fixed, and a new design question goes to the user. Rounds 17 to 17c settled these:

| Id | Section | What was settled |
|---|---|---|
| R17-01 | L §5.16 | the example's `num` range states its step (§5.11, LOAD-DOMAIN) |
| R17-02 | L §5.19 | a link's existence is judged by its keys, `where` and `when` (E §4.1) |
| R17-03 | L §6.1 | a joint model's unlinked movement into a member's place: every read of that place from its day is a hole naming it, balances included |
| R17-04 | L §6.1 | `same` rows of different kinds, read through a kind holding both, meet the movement once |
| R17-05 | L §6.1 | a member's ask is asked by name in the joint model's run and answered by the member alone |
| R17-06 | L §8.3 | `LOAD-SCENARIO` for an ask whose world names no loaded scenario |
| R17-07 | L §5.8 | `x[*] = v` states the keys that exist at its version's `known` |
| R17-08 | L §2.1, §2.4 | the canonical number examples follow the shortest-exact rule |
| R17-09 | L §2.4 | the canonical order of a regime's and a section's clause lines and notes follows the grammar |
| R17-10 | L §6.1 | the joint model's unlinked movement: the hole and the `left out:` line (R8 item 44, MC-E1 C1) |
| R17-11 | L §4.4 | the built-ins on a window, at its endpoints (R8 item 46) |
| R18 | L, E | 130 edits from the contradiction read's 108 confirmed candidates: the literal rule's one list (§3.2 citing §7.1), `LOAD-REPLACE` and `LOAD-OVERLAP` as §4.6 gives them, refusals at load where load sees them, the sign rule after each decision's legs in E, later `opening` readings, an ask line's own projection on a path, `flows` of a move with its settlements, the law-date class bounded by lifespans, sibling scenarios' ids, the lots example's ties, a lifespan whose end is a hole, `LOAD-KNOWN` and `LOAD-PLACEKIND` in place of parse errors, and the wording and stale citations (`review/contra/verified-{1,2,3}.md`) |
| R19 | L, E | the G2 decisions (`decisions.md` GD-1 to GD-26) in one script, `scratch/lead/round19/` (`regime.py`, then `answers.py`): L §2.1–§2.3 (the month-day literal and the type word `monthday`; `regime` and `export` gone), §2.4 (`kind` sections in tree order; the canonical order's open sites), §3.1 (`monthday`), §4.2–§4.3 (a `by n` key no law date; `day_of(y, md)`; a law-date class no row covers), §4.6 (`after m` against the first batch; a period across two keys), §5.9 and §8.1 (the band hole without a date), §5.11 (a `v` naming no version), §5.14.1–§5.14.3 (`before m` only against a decision; a settlement is what `fulfils` an adjustment), §5.16 (the quantile's standard error; `discrete` over keys), §5.17 (`regime` removed, its parts in §5.10, §5.12, §5.14), §5.19 (an `opening` at S start), §6.1 (no `same`-line exception for law parts), §6.2, §7.1 (month-day literals), §8 (an ask's `LOAD-SCENARIO`), §8.3 (what load sees; `LOAD-ORDER`, `LOAD-NAME`, `LOAD-UNIT`, `LOAD-DOMAIN`, `LOAD-SAME`), §8.4 (which load code); E §1, §2 (S the header's start; `flows` before S a hole; [K, E]), §3.1 (load sees L §8.3's list; the cone through places; no refcount step), §3.2 (a class no row covers), §4.1, §4.2 (step 0's opening at S; `after m` names a decision), §5 step 2, §7, §11; then the round's contradiction read (`review/contra/verified-4.md`: 11 fixes, and C4-9 by the user, GD-27); L §2.4's last key (GD-28) and §5.14.1's `bal(x, d start)` row |

## Open gaps

These waited for the user at G2, each a question the spec didn't settle or a change the rule above didn't allow. The user settled them (`decisions.md` GD-1 to GD-26, round 19); what stays open after the freeze is the figures unjudged until the engine (`benchmarks.md`), SF-34 and SF-35 (deferred to tally 3.0), and the keys and case suspects (`freeze/KEYS.md`, the case set's errata list).

- **Regimes in the visible families (REPORT §5.2).** Households A and B write capital gains and VAT as law parts read by their own defs, not as regimes. The regime form runs only in the blind texts. The user leans toward dropping `regime` from 2.0 (`G2-regime-drop.md`).
- **`Q:A-04`, expressed in 1.0.1, now awkward** (A-49 removed `day_of`'s month-day form), and the 36 rows that stay awkward, each awaiting the user's ok (bar B2; REPORT §4).
- **The canonical form's complete rules (L §2.4).** The interpreter refuses 8 order questions as `REFCORE-UNSPECIFIED`. Pins are checked as bytes until they are settled.
- **Five more open sites:** a period read at a key position it lies across (L §4.6); a band read with no law date whose rows don't hold the argument (L §5.9); `discrete` over a key list (L §5.16); the form of a `same` line naming a law part (L §6.1); how far a law-date class reaches (E §3.2 against L §4.6 step 5).
- **Unjudged until the engine:** the figures that exist only at full scale, unjudged until the engine (`benchmarks.md`): D-10's band figures; B-12's figures at 3 000 paths; the amendments household's 170 long-horizon cells; household-bands at 1 000 paths; B-07 on the household's own span and B's balance rows after 2026-12-31; A-05's full-text `windows each` asks; the rows of LH-15, LH-16 and LH-17-mc beyond their cut-downs. The round's full runs were dropped, since they test the reference interpreter's speed, not the language. Their recipes are the engine build's size targets.
- **What load can see,** as one fixed list.
- **The contradiction read's 15 open questions** (`review/contra/verified-{1,2,3}.md`, each with both meanings quoted). The definition doesn't settle which side holds in these:
  - movements before S and `flows` (C1-9 = C2-10), and S itself against a request's later day (C1-10);
  - a negative n in `add_business` (C1-21);
  - a plain read of a reading away from its instant, a hole or `LOAD-CARRY` (C1-23);
  - `after m` inside the first batch, and `before m` ahead of a plan (C2-3, C2-4);
  - the version items a regime's law needs under `LOAD-PRIVATE` (C2-8, in part);
  - day K past or plan (C2-19);
  - a decision's day reading running state (C2-20);
  - `outside` of unit `one` or `any` (C2-21);
  - `when` in a link's existence (C2-23);
  - which settlements escape measurement (C2-26);
  - a base ask naming an absent scenario (C3-14);
  - `WARN-UNREAD`'s reach (C3-19);
  - the canonical order of subkinds (C1-31).
- **A quantile's standard error** (L §5.16). Without one, a band figure from paths can't be tested against a key.
- **A refusal and the holes under it** (L §8.4). A refusal hides the facts that are also missing (`Q:A-10`).
- **Rows still breaks or can't express:**
  - `Q:A-12`: the rounding axis prints by case;
  - `Q:D-07`, figures 6 and 7: a redemption across two versions of the act;
  - `O:A-3`: a calendar settled except one holiday.
- **Deferred:**
  - LC-7 and LC-8;
  - SF-34, one merged dated list;
  - SF-35, a per-arc cap in `fill`.
- **Keys:**
  - LH-11's three rows under `down`;
  - CMP-01's two `inputs-of` listings;
  - the amendments variants v4 and v5: 12 cells whose tally 1 key is a hole;
  - B-11's known part, against a key that leaves out the whole Wrzos balance;
  - the case suspects `Q:B-05`, `Q:B-12` and `O:B-2`, on the case set's errata list.
- **Maintenance:**
  - a law part shared by several households has up to four copies, which must be kept in step;
  - a joint model's bytes pins must be rerun when a member changes.
- **The size of the language:**
  - the contextual words and refusal codes grew with "everything written";
  - L grew by 73 %.

## Decided in `DONE.md`

Nothing yet. The changes the gates need for tally 2 are proposed in the round's `DONE-proposals.md`. They apply only once the user approves them, logged in `DONE.md`'s table.
