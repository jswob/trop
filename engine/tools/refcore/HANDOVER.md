# refcore: hand-over from writers 1 (core), 2 (numbers, randomness, fill, prorata, solve), 3 (movements), 4 (rounds 7, 7b, 7c; faults; the NOT-YET list), 5 (rounds 8, 8b; faults), 6 (rounds 9-13; joint models; faults; UNSPECIFIED sites the current text settles), 7 (rounds 12, 12b, 13b, 14; the triage's sites; faults; writer 6's open items) and 8 (rounds 15, 16 and the redesign review; the removed forms; faults; UNSPECIFIED and NOT-YET sites; a second pass over the joint-model sites, the canonical reprint and the pin; a third pass over the judges' faults on snap8 and round 17; a fourth pass over the second batch and round 17b; a fifth pass over the third and fourth batches; a sixth pass over round 18 and MC-X-F9 to F11), 9 (MC-X-F12, MC-X-F13) and 10 (spec round 19, the G2 decisions GD-1 to GD-26, the contra edits C4; TX-F1)

refcore is a reference interpreter for Tally 2, written from `spec/LANGUAGE.md` (L) and `spec/ENGINE.md` (E) alone,
as a spec-checking artefact. Exact rationals throughout (Fraction); no floating point enters a result (floats are
used only as first guesses of a quantile, which are then verified by interval arithmetic). Where the spec is silent
or unclear it refuses `REFCORE-UNSPECIFIED <section>: <what>` (the list is `UNSPECIFIED.md`); a construct this
interpreter doesn't build refuses `REFCORE-NOT-YET <construct>`. Nothing may crash: an internal error becomes
`REFCORE-NOT-YET an input this interpreter can't handle (...)` unless `REFCORE_DEBUG` is set.

## Writer 10

Spec round 19 (the G2 decisions, `decisions.md` § "The G2 decisions", `GD-n`; the round's diffs are
`scratch/refcore-r10/L-round19.diff` and `E-round19.diff`), the coordinator's contra edits of round 19 part 4
(`scratch/lead/round19/contra_edits.json`: C4-1, C4-2, C4-4, C4-5, C4-6, C4-11; C4-9 as GD-27, below), and the
faults TX-F1 (`scratch/tx/pty05-fault/`), TX2-F1 and TX2-F2 (W-TX2, `scratch/tx2/`). The brief is `scratch/refcore-r10/faults5.md`; each item's outcome and every
test it names are in `scratch/refcore-r10/items.md`. **950 tests, all pass**; 12.6 s, peak 0.26 GB (snap14's 898: 12.4 s;
runs in between took 19 s on a loaded machine, snap14's code timing the same there).

- **GD-1, regimes gone (fixed).** No `regime` section (the grammar's `expected a section, got regime`), no `export`,
  no LOAD-PRIVATE, no `R.x`, no regime counts; `x.f` with x unknown is LOAD-NAME. 8 namespace tests removed (copies in
  `scratch/refcore-r10/removed/`), 6 unboxed (5 renamed `L5_12_*`, `L5_9_band_call_def_and_ask`), 5 new `L2_*_regime_*`.
- **GD-2, month-day (fixed).** `values.MonthDay`, `ty.MD` (`monthday`); `MM-DD` in rows, pins, call arguments and
  `yearly`; `02-29` LOAD-PARSE; `==`/`!=` only (LOAD-UNIT); `day_of(y, md)`, of an anchored key LOAD-DOMAIN; a
  literal in a rule LOAD-LITERAL; `schedule` months only. 17 tests (`L3_1_monthday_*`, `L4_2_*monthday*`, ...).
- **GD-3, quantile's SE (fixed).** `eval2.m_path_stat` / `m_se_quantile`: Maritz-Jarrett, weights by the Beta(m,
  N - m + 1) distribution function's polynomial (exact), correctly rounded as `mean`'s; `=== options mj=check` (not
  language) also computes them as binomial tails and prints both and their sum (`L5_16_quantile_se_check`).
- **GD-4, what load sees (fixed).** `check3.m_seen_day` (L §8.3 item 9: literals and keys of a declared, non-open
  index folded through built-ins that read no fact); `m_check_seen_days` (a decision's plain `bal(x, d)` of its own
  or a later day, LOAD-CYCLE at load, folded per key); `m_check_proj_seen` (a decision's own projection that load
  sees take a random movement after t whose random fact is read after t: LOAD-RANDOM at load; the run gave the same
  instance and fact as RUN-FUTURE, `E3_1_seen_day_projection_random`); `check2.m_random_cone` through every place a
  random movement books, and `m_line_random` (a balance, `low`, `flows` or projection of such a place on an ask line
  without a query is LOAD-RANDOM at load: `L8_3_cone_*`).
- **GD-5, GD-6, GD-13 follow; GD-9 fixed** (`canon`: kinds in tree order, `L2_4_reprint_kind_tree`).
- **GD-10, GD-11, C4-1, C4-2 (fixed).** `check3`: `before m` with m a plan or an observation, and a reading `after m`
  with m a plan or an observation (by class), are LOAD-ORDER with L §8.3's messages.
- **GD-17 (fixed)**: `moves.settlement_move` is a movement whose `fulfils` names an adjustment. **GD-18 (fixed)**:
  `check.run` refuses an ask naming an unloaded scenario alone (`m.asks_alone`), `ask.Asker` prints it.
- **GD-20 follows; GD-21 fixed** (`walk`: an opening at S start anchors whatever K, one before S only before K);
  **GD-22 fixed** (`evaluate.def_window`: one class, one hole naming its first day); **GD-23 follows** (two tests).
- **GD-27 (C4-9, the user, 2026-10-28; fixed).** `check3.m_check_period_keys` (candidates from `check.period_conv`):
  a period read at a key of a time index whose keys are declared, written as a literal or folded from the reader's
  own keys (L §8.3 item 9), that lies across two keys is LOAD-KEY `<period> lies across keys <a> and <b> of <index>`
  at load; one computed from a row stays RUN-DOMAIN at the value (`evaluate`). Tests `L4_6_period_across_keys`
  (a row: RUN-DOMAIN, comment updated), new `L4_6_period_across_keys_literal`, `_folded`, `L4_6_period_held_by_key`.
- **GD-24 (fixed).** The 13 sites (below, REFCORE-UNSPECIFIED places) now give the stated outcome; `canon` has
  `Tie`/`Rev` sort keys. C4-4 (`v` naming no version LOAD-NAME), C4-5 (LOAD-SCHEDULE only where load sees the day),
  C4-6 (LOAD-UNIT, follows), C4-11 (`discrete` over a key list LOAD-DOMAIN).
- **TX-F1 (fixed).** `walk.consol`: a modelled-past plan (`same` with an observed row) books no place of its own, and
  an observation's end is booked by the version that covers that place, so `stmt[r1]` books acc; the open register
  skips same-observed instances. fault.tally prints 5 000.00 (flows, bal; `explain` names stmt[r1].cash), the three
  contrasts 5 000.00, PTY-05 (`--pin=bytes`) `(5 000.00, -5 000.00)` (snap14: 0.00, 0.00). 5 tests `L6_1_*`.
- **TX2-F1, TX2-F2 (fixed).** L §5.14 on d: a day that is a hole and not a window makes every `flows` and binding
  a hole "whatever the instance's `when` says". `moves.m_inst_body`: a false `when` on such a day leaves the
  instance's existence the day's hole, its legs read as for any hole day (snap15 dropped it: `flows` 0.00, a known
  balance). `mreads.m_flows_move`: a decision instance with such a day has no walk point and was skipped; it now
  counts as its hole. TX2-F2 (household B's `flows(mtg_pay, 2025)` 36 326.10 DUK) was this second path, not the
  Cis observations: reduced to `dec2` in `L5_14_hole_day_when_false`; one run of B (182 s, 2.1 GB) now prints the
  hole. Tests `L5_14_hole_day_when_false`, `L5_14_hole_day_holiday2027` (W-TX2's text).
- **GD-28, L §2.4's ties (the user, 2026-10-29; fixed).** "Two rules, or two items, that every key above leaves tied
  ... sort by their own canonical line, compared as ASCII": `canon.ascii_key` is the last component of
  `rule_key` and `item_key` (UTF-8 byte order), and `Tie` no longer raises. The four §2.4 sites are gone; their
  tests `L2_4_reprint_tie_no_start`, `_one_start`, `_after`, `_same_row` now print that order. `refcore reprint`
  gives a sha256 for household B (`cases/households/B/*.tally records/*.tally`: `38500231c4594f8f8235fc4045b121fe
  65e6c7ea4b8b11c0bf750c7aab1e49b7`) and D (`*.tally law/*.tally`: `fcb76e41cb8d7eb232a7a701590082bd11d88c3551
  9416c3dead08c7892eabe1`); outputs in `scratch/refcore-r10/runs/`.
- **L §5.14.1 `bal(x, d start)` (round 19's new row; fixed in part).** On the decision's day it is no cycle
  (follows: `L5_14_1_bal_start_own_day`, the decision's result equal to the read on an ask line); for d after the
  day load now refuses it where it sees the day (`check3`: the syntactic and the folded LOAD-CYCLE checks no longer
  skip a `start` read; `L5_14_1_bal_start_later_day_load`; it was RUN-FUTURE), otherwise RUN-FUTURE
  (`L5_14_1_bal_start_later_day_run`). `reads x` unchanged.

**Files touched:** `ask.py`, `canon.py`, `check.py`, `check2.py`, `check3.py`, `cli.py`, `eval2.py`, `evaluate.py`,
`joint.py`, `loader.py`, `model.py`, `moves.py`, `mreads.py`, `parser.py`, `ty.py`, `values.py`, `walk.py`, `UNSPECIFIED.md`, this
file; 65 new tests, 27 changed, 13 removed or renamed (the list: `scratch/refcore-r10/items.md`).

**UNSPECIFIED** (1 site, generated by `scratch/refcore-r10/gen_unspecified.py`, a copy of writer 8's with the texts
of the sites that left removed): §5.16, the Maritz-Jarrett SE of samples that are no numbers
(`L5_16_quantile_se_keys`). The four §2.4 ties of round 19 are settled by GD-28 (above).

**Left open:** `flows(gift)` and `observed(gift)` of a same-observed modelled plan give 0.00 and none
(`scratch/refcore-r10/t/tx1.tally`); a decision projection whose day is seen but whose reach isn't (a random
movement's day from a row) stays RUN-FUTURE, as L §5.16 and now E §3.1 step 10 say (contradiction 50, settled); a
projection in a def a decision reaches is left to the run; the "seen" test of LOAD-SCENARIO and LOAD-ORDER uses
`moveon` edges; GD-22 is still NotYet where a reality's span boundary falls inside the window; kind roots whose parent
isn't printed sort among the roots; the residual line of a reading after a decision has no test.

## Writer 9

One fault, MC-X-F12 (`scratch/refcore-r9/faults4.md`): a band whose every row governs `always` was taken as
law-dated. **896 tests, all pass** (24 new); 12.7 s, peak 0.26 GB. Minimal texts and probes: `scratch/refcore-r9/F12/`,
`scratch/refcore-r9/probes/`; the tests are made by `scratch/refcore-r9/gen/mk.py`.

**The fix.** L §4.3: law-dated is "an input, band, def or rule with a row or version whose span isn't `always`".
- `check.check_lawdates`: `banddated` (also `Checker.banddated`) holds, per band, a table in a version whose governs
  isn't always or that a `moves` line moves (`ver_timeless`), or a member read. A band whose rows read a law-dated
  input, band or def (an `at reader` one, one whose `at` reads law-dated facts) is law-dated too: bands are "read at
  the law date of their reader" (L §4.3), so are their rows' reads, and P-6's chain goes on through the band
  (`reads_dated`, taken to a fixed point with the defs; `at_memo` dropped when a band turns law-dated). A def's read,
  an `at e`'s read (`at_dated`) and an ask line's direct read of a band now go by `banddated`.
- `check3.m_check_move_lawdates`: a move with no `at` reads a band by `banddated`; and a move's `at` that reads a
  law-dated input or band directly is LOAD-LAWDATE `move <m>'s at reads law-dated <x>` (L §4.3 "The at ends every
  chain, so it reads nothing law-dated itself", L §8.3). snap12 left that to the interpreter guard at the read
  (REFCORE-NOT-YET), for inputs too.
- `evaluate.band_table`: a band read with no law date takes the tables that govern always (a dated one is the guard
  still); `band_rows`: no table gives `b: no value` / `b at k: no value` (L §4.6 step 5 "For an input or a band ...
  Read with no law date ..., the question ends at no value"; §8.1); `band_no_row`: with no law date,
  REFCORE-UNSPECIFIED §5.9 (below).

**Tests.** `L4_3_band_always_def`, `_always_ask`, `L4_3_input_always_control` (the three minimal texts);
`L4_3_band_dated_def`, `_dated_ask` (a dated band refused without `at` and without `law`), `_dated_law`;
`L4_3_band_later_dated` (rule 2's last clause: a later version's dated `replace` row), `_moved` (a `moves` line);
`L4_3_band_row_reads_dated`, `_dated_ask`, `_dated_law`, `_timeless`, `_at_reader`, `_at_reader_law`;
`L4_3_band_move`, `_move_dated`, `_at_reads`, `_at_reads_dated`, `_move_at`, `_move_at_dated`,
`L4_3_input_move_at_dated`; `L5_9_band_no_law_no_row`; `L4_6_band_no_table_no_law`; `L6_4_scenario_row_band_always`.

**Files touched:** `check.py`, `check3.py`, `evaluate.py`, `UNSPECIFIED.md`, this file, the 24 tests (F13: `moves.py`, 2 tests).

**UNSPECIFIED**: one new site, §5.9 (`evaluate.band_no_row`): a band read with no law date whose rows don't hold x;
L §5.9 and §8.1 give the question only as `<b>: no row holds <x> at <law date>`. Added to `UNSPECIFIED.md` by hand
(13 rows): the generator in `scratch/refcore-r8/` has no minimal text for it, so a rerun drops the row until it gets one.
*Settled by round 19 (GD-24): the hole is `<b>: no row holds <x>` (`L5_9_band_no_law_no_row`); the 13 rows are
settled and `UNSPECIFIED.md` is regenerated (writer 10).*

**Left open (seen, not fixed).** A scenario's expression row (L §6.4) that reads a law-dated base fact (a band or an
input) in a row governing `always`: by L §4.3 the input's value changes with the law date, but `InputInfo.lawless()`
looks at row spans only, so its def reader states no `at` and the read meets the interpreter guard (REFCORE-NOT-YET),
for inputs as for bands, as in snap12 (`scratch/refcore-r9/probes/sc/d`, `sc/i`). The always-band case now answers
(`L6_4_scenario_row_band_always`).

### Writer 9: MC-X-F13

`refcore run scratch/refcore-r9/F13/I-8.tally --ask c_paydays` (and `c_overpayment`, `c_owed_after`) gave
REFCORE-NOT-YET `(AttributeError: 'NoneType' object has no attribute 'unit')`. The cause is in `moves.adjust_move_compute`,
the per-step measure of E §5 step 3. The loop took the legs to compare from the booked instance, else from the step's.
When one step ended a booked instance and a later step found it still ended, neither state had the leg, and
`(b or a).unit` read None. Now (`moves.py`, the `names`/`pool` lines and the `a is None and b is None` skip):
- the legs come from the two states compared, the previous step's (when it existed) and this one's;
- a leg that neither has is skipped: δ = 0 and no places change, so E §5 step 4 records nothing;
- the adjusted leg's places come from the booked instance or the step's, as before, else the previous step's.

The same loop also lost the ending adjustment of an instance that a later step made exist and a step after that
ended (no legs at κ₀, none under the ending step).

Tests: `E5_adjust_ended_twice` (snap13: the AttributeError) and `E5_adjust_made_then_ended` (snap13: one adjustment,
not two). Minimal texts: `scratch/refcore-r9/F13/ended_twice.tally`, `made_then_ended.tally`. The three asks of I-8
now answer; their figures are not judged here. No UNSPECIFIED site. **898 tests, all pass**; 13.7 s, peak 0.26 GB.

## Writer 8, sixth pass

The coordinator's sixth pass (`scratch/refcore-r8/faults3/faults3.md`): spec round 18's outcome edits (the
contradiction read; reasons in `review/contra/verified-{1,2,3}.md`), MC-X-F9 to F11 on snap11, and the register note.
**872 tests, all pass** (57 new); speed untouched; the suite peaks at 0.25 GB (the 8 GB cap of 12:05 is far off).
Where this section and the older ones below differ, this one holds. Per-id notes: `scratch/refcore-r8/sixth/notes.md`;
minimal texts: `scratch/refcore-r8/sixth/r18/`.

**Round 18's outcome edits.** F: snap11 already did what the edited text says (a test now names it). X: fixed.
- C1-1 F `L4_3_end_of_last_day`. C1-2/3 F `L7_1_inline_figures_def`. C1-6 F `L3_3_factor_three_args`. C1-7 F
  `L7_2_mode_axis` (a conversion `in CUR at fx mode` still needs `law`, LOAD-LAWDATE, as before).
- C1-8 X `L5_7_lifespan_hole_end`: `lives 1950-01-01..?"date of death"` parses (`parser.life_span`), the span keeps
  its question (`Span.end_q`), the thing is an unknown member on the days it may be alive (`model.LifeHole`,
  `walk.alive_day`), `lifespan(k).to` is the hole (`evaluate.c_lifespan`).
- C1-11 F/X `L4_6_replace_same_day_disjoint` (loads), `_overlap` (LOAD-OVERLAP, step 4; rows, entries and rule
  versions: `loader`, `evaluate.rule_in_force`; no LOAD-REPLACE for two same-day `replace` lines).
- C1-12 F `L5_3_same_keys_one_version`, `_tie`. C1-14 X `L5_11_first_no_ties_load` (`check.t_agg`; `L5_7_order`'s and
  `L5_11_first_notyet`'s q1 moved there). C1-15 X `L4_2_day_of_anchored_load` (`check.anchored_key`, in `day_of` and
  `schedule`; `L4_2_day_of_anchored` keeps q3).
- C1-16 F `L4_3_at_reads_lawdated`, `_no_law`. C1-17 F `L4_1_by_n_no_law_date` (contradiction 47). C1-18/C1-35 F
  `L4_6_lifespan_key_period`, `L4_6_movement_facts_no_lapse`. C1-20 F (`E6_3_residuals_adjustment`,
  `L6_1_same_rows_settled`). C1-24 F `L5_3_version_no_known`, `L5_7_placekind_no_sign`. C1-25 F
  `L7_1_literal_replace_rule`. C1-27 F `L2_2_unit_bound_variable`, `L2_2_unit_rule_label`. C1-30 F
  (`L5_14_3_adjustment_name_observed`'s e).
- C1-38 X `L5_6_fill_key_bind_kind`: a fill's key bind over a kind needs no `on`/`over` (`check.bind(key_bind=True)`,
  `check3.m_bind_movements`, `check2.m_fill_sides`).
- C1-47 X `L5_16_mean_of_dist_value` (a def), `_ask`: `mean(d)` of a distribution value is LOAD-RANDOM `mean of a
  distribution value; ask it of a random fact` (C3-3's message); `L5_16_mean_of_discrete` now LOAD-RANDOM;
  `L5_16_query_closed_in_rule` lost md; the table's means are read of random facts in
  `L5_16_mean_table_random_facts` (`check2.m_dist_value_query`; `eval2.dist_value_mean` removed).
- C2-1 X `L5_10_sign_rule_decision` (a decision's legs together: leaving and re-entering a claim passes; breaking it
  refuses that decision's legs at the place), `L5_10_sign_rule_close`, `_close_refused` (the closing readings
  together). `walk.check_group` tests every place of a group against the balance the group leaves, then refuses the
  legs at the broken places (so the first batch no longer depends on place names: `L5_10_sign_rule_batch_all_places`,
  snap11 tested in name order and took back as it went); `run_one` wraps `run_one_body` and tests the decision's legs,
  its own observations and the `same` ends it books, then applies its `after` readings, each tested as booked.
  Beside it: a pad refused as it is booked counted the reading's end in conservation, ENGINE-INVARIANT
  (`apply_reading`; `E4_2_refused_pad_after_decision`).
- C2-2 F/X `E4_2_opening_after_start`: an opening after S anchors at its instant (F); one at or after K is not applied
  (X: "a reading applies only before the cut K", L §5.19; E §4.2 steps 0 and 4), but one at S start or before S is,
  whatever K (contradiction 48). `L6_4_base_binding_reaches_scenario` changed: lokata's opening at 2027-01-01 was
  after K; the bonus now goes into lokata.
- C2-7 F (`L5_16_ask_projection_path`). C2-8 F `L5_17_version_band_row_private_band` (the refcount is
  `tools/refcount.py`'s, not built here). C2-9 F `L5_14_3_flows_move_settlement_span`.
- C2-13 X `L5_19_modelled_from_reading_before`: `bal(x, d) modelled` starts from the last reading before day d, else
  an opening on d, and skips the readings on d (`mreads.m_bal_modelled`).
- C2-14 F `L5_14_1_pads_binding_plan`, `_reads`. C2-15 F `L4_6_star_row_own_known`. C2-24 F
  `L5_19_uncertain_covered_not_open`. C2-32 F `L4_4_known_inside_window`, `_changes`. C2-42 F
  `L5_14_1_run_cycle_own_day`.
- C3-1 F (`L5_11_new_label`). C3-2 F `L2_5_sibling_scenarios_same_id`. C3-4 F `L5_16_cdf_discrete_lower_end`.
- C3-7 X/U `L4_3_at_window_lifespan`: a window `at` across a lifespan's end is HOLE-IMPRECISE (snap11 let a hole win
  over HOLE-IMPRECISE; `evaluate.def_window` now: one value, else HOLE-IMPRECISE where some day's value is no hole);
  refcore reads every day of the window, so every class boundary is one. Where every day is the gap rule's hole,
  naming its own law date, the class's extent decides: REFCORE-UNSPECIFIED (`L4_3_at_window_gap_holes`,
  contradiction 49). The instance path (`moves.body_over_law`) still calls any difference HOLE-IMPRECISE.
- C3-8 X `L5_14_1_bal_def_no_movement_explain`: walk-bound needs a decision to reach the def (`check3`: `walk_bound`
  after `from_decisions`); explain gives such a def's ordinary derivation.
- C3-13 F `L5_17_lots_two_sales` (FIFO, 600.00 and 700.00), `_old_order` (RUN-TIE). `L5_17_regime_example` follows
  the example's current lines (arc with `unit(…) == unit(…)`, tax's `where … in ordinary`, the new order). Beside it:
  `unit(q)` of a quantity was unbuilt (`check3.m_unit_of_quantity`, `mreads.m_c_unit`; `L5_10_unit_of_quantity`,
  `_plain_number`), and a fill's take of a pair it never matched, "a 0 with no key", now takes the key of what it
  meets in `+`, `-`, `*` and `/` (`evaluate.arith`: `keyless_zero`, `adopt_key`, `cancel_kind`; snap11: LOAD-UNIT
  `600 PLN and 0 PLN*notes/fund_x have different units`).
- C3-16 F `L5_9_whole_table_replaces_nothing`. Beside it, L §5.9's older sentences: a whole replacing table stands on
  a day where no table is in force (`evaluate._table_of`; `L5_9_whole_table_no_table_in_force`, snap11 LOAD-REPLACE at
  the read), and a patch whose span holds such a day is LOAD-REPLACE at load, `replace rows of <b> in <V> patch no
  table on <span>` (`loader.check_bands`, `_uncovered`; `L5_9_patch_no_table_load`, `L5_9_patch_covered`).
- Grammar: C1-8, C1-24 and C1-7 as above.

**Faults.**
1. **MC-X-F9** X `L5_14_movement_equality`: `==` on two movements compares their stable ids (`evaluate.compare`); an
   order on movements is RUN-DOMAIN `an order on movements (…) has no meaning` (`cmp3`).
2. **MC-X-F10** X `L5_12_fill_dynamic_unit`, `_run`: a side of unit `dyn` (an adjustment's `change`, "of the leg's
   type", L §5.14.3) is checked per value at the take (`check2.m_check_fill_body`; `eval2.m_fill_compute`: RUN-DOMAIN
   `fill <f>: the supply … and the demand … of the arc (…) have different units`). The spec states `change`'s unit
   (per value), so no UNSPECIFIED site.
3. **MC-X-F11** X `L5_11_term_order_window_move`: a prefix walk evaluated a window-dated plan whose window starts after
   the walk's day; `moves.m_eval_inst` now skips it by the window's first day, so the answer no longer depends on the
   order of terms.
4. **Register note.** konto_balance's `unfulfilled: health_pay[2026-07]` is right by L §8.2: the balance books
   r20260921b, judging its link tests `ryczalt_pay[2026-08]`'s `when`, whose `flows(health_pay, 2026-08)` counts that
   instance ("each `when` it tested"). The fifth pass's "its register" meant konto_balance's, not the replay's. The
   replay (`residuals konto 2026-09`) lists no register entry at all, only "law date, cut": `resid.m_residuals` reads
   cached instances, observations and the walk without merging their collectors. By L §8.2 and E §10 ("A line's
   register is the union of its expressions'") it would list it too, through the matched `ryczalt_pay[2026-08]`'s
   booking-cut amount and the link. **Not built** (a gap: the residual ledger's register).

**Readings taken:** first/last over things without `ties by` at load is LOAD-UNIT (as `min` over things), `day_of`
of an anchored key LOAD-DOMAIN: L §8.4 gives "(`LOAD-UNIT`, `LOAD-DOMAIN`)" without saying which; an opening at S
start applies whatever K (48); a refused leg's places are both refused from the check (as before, E4_2_refused_leg_reads
f4), so taking a group's legs back leaves no further balance to test; the cut's `as known` aside, a patch's load check
uses the spans in force at `on`.

**Not fixed, seen:** an ask `law w` with w a window input gives REFCORE-NOT-YET `(HoleSignal: )`
(`scratch/refcore-r8/sixth/r18/c3_7/m.tally` q3), an internal leak; the residual ledger's register (above).

**UNSPECIFIED**: one new site, E §3.2 (`evaluate.def_window`, `L4_3_at_window_gap_holes`); `UNSPECIFIED.md`
regenerated, 12 rows. **Contradictions**: 47, 48 and 49 are new (below).

## Writer 8, fifth pass

The coordinator's fifth pass: the fault round's third and last batch (`scratch/refcore-r8/faults2/faults2.md`, "Third
batch": six faults from the judge of the amendments household, household-bands and the blind cards), with the
"Fourth batch" added to it (W-B-F4, W-B-Q1). **815 tests, all pass** (15 new); speed untouched. Where this section and
the older ones below differ, this one holds.

**Faults** (each with a test naming its section):

1. **MC-X-F1, ENGINE-INVARIANT (E §9 invariant 5) on the replay, depending on the order of the asks** (L §5.19, E §4.1
   step 3, E §9). The cause, found in the code: the table of observation links was built in one go
   (`moves.obs_links`), and judging one link tests its target's `when` (`ryczalt_pay[m]`: `ryczalt_adv[m] > 0`), which
   reads `flows(health_pay, m)`, which takes the status of `health_pay`'s instances, which read the link table again,
   re-entrantly, while it was half built. That status then saw no observation for an instance a later row fulfils
   (`ryczalt_pay[2026-08]` modelled without its 1 990.00), so `pit28_due` read 1 887 instead of -103, and
   `pit28_pay[2026]`'s booked body differed between two reads at one cut. Which ask ran first decided where the build
   started. Now each target's links are judged on their own, when first read (`moves.obs_links_for`, `link_targets`;
   an observation's `target` and `err` are judged lazily, `ObsMv._judge_link`); judging that reads the same target's
   links again is RUN-CYCLE. Minimal text written (`scratch/refcore-r8/f5x/f1/m.tally`): `probe`'s `when` reads
   `flows(adv_pay[m], m)`, one statement links both; snap10's link judging gives RUN-RECORD and an `open:` line, now
   f is 150.00 PLN and q 640.00 PLN in either order (`L5_19_link_reads_another_link`, `_order`). The household's
   replay now runs: konto 6 064.17 PLN as before, and its register now also lists `unfulfilled: health_pay[2026-07]`,
   because judging `ryczalt_pay[2026-08]`'s link tests its `when`, whose `flows(health_pay, 2026-08)` counts that
   instance (L §8.2: "The list is transitive: whatever its evaluation read, through any chain (N-713), each `when` it
   tested"). What judging a link read is now kept with the link: every reader of the cached verdict takes it, and so
   does whatever counts an observation it judged (the observation's collector). Before, it went to whichever reader
   built the table first. `L8_2_register_link_when` (an ask judges the link first), `_alone`.
2. **MC-X-F2, a scenario's `replace` of an entry known the same day as the base's** (L §6.4, §4.6 step 2). An entry's
   `replace` looked for an earlier entry by `known` alone. In a scenario a base entry is an earlier row whatever its
   known ("a base row known on the scenario's own known day is replaced too"), as rows already were: the loader now
   compares the versions' layers (`loader.version_layer`, `row_layer` built on it), and the lifespan a cut reads takes
   a scenario's replace over the base's whatever their known dates, then the later known (`model.ThingInfo.span_in`,
   `version._layer`). The check "two replace lines known the same day" is per layer. `L6_4_replace_entry_same_known`
   (the fault's text, §6.4's example), `_later_base` (a base replace known later than the scenario's).
3. **MC-X-F5, RUN-TIE when one instance's `when` is false** (E §4.2 step 3; L §5.14.2). Two instances of one move on
   one day were a tie when either read a place the other changes, and the places a decision read included plain
   balances of earlier days, which no decision of the day changes. A running decision now records apart the reads that
   see its point (`walk.live`, `run.live_places`: `bal(x)`, `flows` through its day, a projection); a plain
   `bal(x, d)` reads a point before the day's decisions (one at or after it is refused), so it orders nothing
   (`mreads.note_read`). The tie test and `run_group`'s settled `when` use those. Explain's "read at its walk point"
   still lists every place read. `E4_2_tie_when_false` (the fault's text: one instance a month, its `when` false for
   the other).
4. **MC-X-F6, a fill ordered by a window day** (L §4.4, §5.12 Sort). The fill's sort compared the order terms with the
   plain three-way compare, which refuses a window (LOAD-UNIT). It now compares a window as L §4.4 decides: before or
   after where the whole interval lies on one side, equal for a value compared with itself, and RUN-ORDER `fill <f>:
   the order of the arc (…) is a hole` where it can't decide (`eval2` fill sort, `cmp1` through `iv_compare`).
   `L5_12_fill_order_window` (the fault's text), `_undecided` (two overlapping windows).
5. **MC-X-F7, "fill read inside its own supply" on a fill its supply doesn't read** (L §5.12, §5.14.1). The check
   types moves before fills, so a plan's leg reading `f[y].taken` met the fill untyped and took that for a read inside
   its supply. A fill is now checked once, where it is first read if that comes before its turn
   (`check2.check_fill`, `check_fill_body`); a read while the fill's own supply is being typed is still REFCORE-NOT-YET.
   `L5_12_fill_read_by_move` (the fault's text).
6. **MC-X-F8, an exported keyed fill read from outside its regime** (L §2.3 `post`, §5.12, §5.17, §2.5). `R.F.taken[…]`
   and `R.F[y].taken[…]` are now fill reads (`check2.fill_ref`); from outside R in anything but an ask, F must be in
   R's `export` (LOAD-PRIVATE `<item> names <R.F>, which regime <R> doesn't export`). Minimal text written:
   `L5_17_export_keyed_fill` (a def and ask lines read `gains.matched[marek].taken[l, c]`; takes hand-checked from L
   §5.12), `_private` (no export line). *Settled by round 19 (GD-1): regimes are gone; the read is
   `L5_12_keyed_fill_read`, `_private` is removed (writer 10).*
7. **W-B-F4, `windows each` on a line with a binding over a kind `over` a span** (L §4.4, §5.18). It crashed
   (TypeError in `WCut`): date arithmetic on the cut built its result through the WCut class. Arithmetic on a cut now
   gives a plain day (`model.WCut`). `L4_4_windows_each_cut_arithmetic` (minimal text: the line meets no window and
   prints once, as without the clause).
8. **W-B-Q1, `schedule` over a calendar whose `added` holds a window** (L §4.2, §4.4, §5.14, §8.1). Built as the lead
   reads it; the sections state it. snap10 took any business-day read of a day inside an added window as a hole with
   HOLE-CALENDAR's old question, so a scheduled day there was a hole with no bound (L §5.14), a hole in every span. L
   §4.2: "every day known but one. Such a window is one the line meets, so a result that depends on its day is
   HOLE-IMPRECISE unless the line states `windows each` (§4.4)"; L §4.4: built-ins on a window give the interval of
   their values, and "only a comparison decides an interval"; L §5.14: a movement's day may be a window. The
   business-day functions (`schedule`, `add_business`, `last_business`, `is_business`) are now evaluated at each day
   the window can take (`evaluate.over_added`: one pass per day of each window met, nested where a pass meets
   another). One value is that value. Dates that differ give the interval of the days: a window, so a movement dated by
   it counts in a span that holds it, not at all in a span wholly outside it, and a span that crosses it or a balance
   inside it is HOLE-IMPRECISE. An `is_business` that differs is HOLE-IMPRECISE, `date within <window> not precise
   enough for is_business(…)` (L §8.1's HOLE-IMPRECISE row, "a window in a calendar's `added` list"; its
   HOLE-CALENDAR row now names only an added day that is a hole). `windows each` still splits by the added window.
   Minimal texts written: `L4_2_schedule_added_window` (one_of: the day, `windows each`, flows of a span inside, outside
   and crossing, two balances), `_interior` (a window(a, b) whose inner day moves the result: contradiction 46);
   `L4_2_added_window` takes the HOLE-IMPRECISE message.

**Readings taken:** a running decision's plain `bal(x, d)` orders nothing among the day's decisions (it can't see
them); a scenario's entry `replace` outranks the base's at every known date, then the later known within a layer; what
judging an observation's link read belongs to every result that reads the verdict or counts the observation (L §8.2);
a business-day function over an added window is taken at every day the window can take, not only its ends (46).

**UNSPECIFIED**: no new site (`UNSPECIFIED.md` regenerated, 11 rows). **Contradictions**: 46 is new (below); 19 is
settled by the current L §8.1 and marked so.

## Writer 8, fourth pass

The coordinator's fourth pass: the fault round's second batch (`scratch/refcore-r8/faults2/faults2.md`, "Second
batch": three faults from the joint household's judge and W8-SEEN, which writer 8 saw in the third pass) and spec
round 17b (`faults2/L-round17b.diff`). **800 tests, all pass** (8 new); speed untouched. Where this section and the
older ones below differ, this one holds. No fault came by message during the pass.

**Faults** (each fixed, with a test naming its section):

1. **MC-E1-F1, a member's base ask naming one of its own scenarios** (L §6.1, §6.4, §5.18). Each member is loaded
   alone from its base files, and its ask `q: x with scenario s1` was checked there with s1 unknown, so every joint
   model over it was refused (`member m: LOAD-SCENARIO …`). The member's base now loads knowing its scenarios' ids,
   as `run_worlds` loads a base (`joint.joint_texts`, `member_worlds`), and a base ask that names one of them (`in`,
   a diff's worlds, a scenario pin, `as if {scenario s}`) is answered by the member alone with all its files, as an
   ask in its scenario file already was (`ask.member_names_world`, `Asker.member_alone`). `L6_1_member_ask_own_scenario`
   (the fault's text), `_in` (`in s1`, a diff, a base ask).
2. **MC-E1-F2, the period word in `periodic(r, month, …)` renamed by the merge** (L §2.2 period words, §2.3 Calls,
   §6.1). The parser marked the lone word in `periodic`'s and `factor`'s period position as an id, so a member that
   declares `index month` had it renamed `m.month`. It is now a word of the language there
   (`parser.period_word_arg`), and the merge leaves it. `L6_1_member_period_word` (the fault's text),
   `L6_1_member_period_words` (`factor`, a duration `1 day`, indexes named `day` and `year`).
3. **MC-E1-F3, no `left out:` after the member's reading** (L §6.1, §8.2, round 17b). The joint movement's register
   entry rode on the walk's hole entry, which the member's reading resets, so a balance after the reading lost it. The
   walk now keeps each place's unlinked joint movements (`Walk.joint_lo`), and every balance, `low` and `flows` that
   reads the place at or after one's day lists it (`mreads.joint_left_out`, from `read_state` and `flows_place`).
   `L6_1_joint_move_until_reading` (the fault's text: a balance before the movement's day, between it and the reading,
   after the reading, and flows over January and February).
4. **W8-SEEN, a line that isn't random refused in a model with a random-day move** (E §7 step 1, §3.1 step 10; L
   §5.16). Outside a path, a plan whose day reaches a sample was an instance refused before its legs were read, so its
   refusal held for the whole walk. Now `moves.eval_inst` catches the random line at the day (outside a path only),
   reads the legs at the cut as for a hole day, and keeps the refusal: `walk.sched_refused` books it at the places the
   legs name, from the walk's start (the day has no bound refcore can see), so only their balances meet it. A decision
   whose day reaches a sample is refused at its legs' places the same way (`walk.dec_bad`, `placebad`); where the
   places can't be read, every place is, as before. A line reading such a place without a query is still LOAD-RANDOM.
   `E7_1_shared_line_random_day`, `_decision`, `E7_1_random_day_decision_place`.

**Round 17b, checked:**
- L §6.1 The joint model's own movements: "A read of that place … from the movement's day until the member's next
  reading of the place, a balance included, is a hole naming it …; from that reading on, the reading states the
  balance, as after any hole movement (§5.19)": agrees (the walk's reading resets the hole; `after: 10.00 DUK` with its
  anchor line). "Every result that reads the place on or after the movement's day lists it under `left out:`":
  **changed** (fault 3); a read before the day lists nothing, as before. Contradiction 44 settled.
- L §8.2 `left out:` "under a result of a joint model that reads a member's place on or after their day": agrees
  with the above; the count is of the instances dated at or before the read.
- `L6_1_joint_move_in_member_place` and `L6_1_joint_move_reads`: outputs unchanged (no member reads konto after the
  movement); comments cite 17b.

**Readings taken:** a member's base ask names its own scenario through `in`, a diff's world, a scenario pin or an `as
if {scenario s}` in its line or a def it reaches (the worlds `run_worlds` uses); outside a path, a random-day
instance refuses its places from the walk's start (its day's support isn't read); a lone `day`, `month`, `quarter` or
`year` in `periodic`'s second or `factor`'s third argument is the period word, whatever the model declares.

**UNSPECIFIED**: no new site (`UNSPECIFIED.md` regenerated, 11 rows). **Contradictions**: none new (44 settled by
round 17b; the third pass's "Seen" note is fault 4, fixed).

## Writer 8, third pass

The coordinator's third pass: the faults the judges found on snap8 (`scratch/refcore-r8/faults2/faults2.md`, seven,
each with its minimal text) and spec round 17 (`faults2/L-round17.diff`, eleven edits, already in `spec/LANGUAGE.md`).
**792 tests, all pass** (18 new); speed untouched. Where this section and the older ones below differ, this one holds.
No further fault came by message during the pass.

**Faults** (each fixed, with a test from its own minimal text naming its section):

1. **MC-A-F1, `windows each` under an axis fan-out** (L §4.4, §5.5, §5.18). The windows a line meets were collected
   only on the first evaluation of a memoised cell, so under the second axis option the line printed the first
   sub-window's value for the whole window. The windows met now travel with the cell's collector (`Coll.each`,
   merged like the rest) and `ask.each_rows` takes them from the result (`EachMet`). `L4_4_windows_each_axis_memo`.
2. **MC-D-F1, a RUN-CYCLE that hung on the order of asks** (L §5.11; E §3.1 step 5, §3.2, §4.1). A plan's leg check
   asked for the main walk while that walk was being built for another cell, so the answer depended on which ask ran
   first (RUN-CYCLE, or ENGINE-INVARIANT naming one cut twice). Now a leg check made while a walk is running tries
   the main walk under a journal of the memo and the walk caches (`mreads.guarded_walk`, `evaluate.JournalDict`);
   if it meets a cell of the outer walk it rolls the journal back and reads a walk that stops at the leg's day
   (`mreads.prefix_walk`: the plan instances through that day only, `moves.plan_list`, `upto`). A cycle is still
   RUN-CYCLE where the chain returns to a cell being evaluated. `E3_2_demand_driven_walk`, `_order` (the same outputs
   in either ask order).
3. **MC-M-F1, a key list over an anchored index** (L §2.5 Bare names, Keys). A variable bound over a key list used at
   a key position of `tmonth` (months from the 6th) read its literals as calendar months; it now takes its index
   from the position (`check.key_conv`, `_klvar`), so `2022-06` is the key that starts 2022-06-06.
   `L2_5_keylist_anchored`. The case left behind is a new REFCORE-UNSPECIFIED site, below.
4. **MC-B-F5, a def reading a randomly moved balance inside a query over paths** (E §3.1 step 10, §7 step 2; L §5.16).
   The random cone stopped at inputs, defs and fills, so `low_w` (a def over `bal(w, …)`, `w` booked by a move with
   a random day) shared one cell across paths and its read of the sample refused the load. `check2.random_cone` now
   lets moves (`('move', id)`, `('moveon', id)`) into the cone, records the places a random move's legs book (a
   literal place by name, any other as every place), and makes random each def, fill or decision that reads one of
   them (`bal`, `low` at a day, `bal(x)`, `flows`, a decision's legs), to a fixed point. So `share(low_w < amt) …
   paths 3 seed 1` runs, a bare `ask q: low_w` is refused at load (`LOAD-RANDOM ask q: low_w is random; apply a
   query`), and `mean(low_w)` without paths has no closed form (it was "a value that is not random").
   Projections stay out of the cone (a projection reaches a random amount through its clause, the line's query or
   the path's sample; one with none is refused at the read, as before). A balance read directly in an ask line is
   still found random at the read (contradiction 36). `L5_16_random_balance_def`, `_bare`, `_closed`, `_amount`.
5. **MC-B-F2, a `partial` balance with a hole leg** (L §5.18 Partial answers, §5.10, §5.19). A balance whose walk met
   a hole amount printed the plain hole. Each hole entry now keeps its sign and leg (`walk.Ent.sgn`, `.leg`,
   `moves.Leg.src`), and a balance read under `partial` adds the known part to each hole leg's amount re-read as a
   partial value (`mreads.partial_bal`), giving `100.00 DUK + -1 × ?"…" (sp[2025-03])`; it falls back to the plain
   hole where the form isn't linear or the place has an anchor, a refusal or an imprecise window.
   `L5_18_partial_balance`, `_when`.
6. **MC-B-F3, the difference of two month values** (L §4.2 Differences). Period minus period of one period kind now
   types `int` and evaluates as the number of months (quarters: months / 3; years) from the second period's start to
   the first's (`check` `_datearith = 'pdiff'`, `evaluate.date_arith`). `L4_2_period_difference`.
7. **MC-B-F1, an ask whose world's scenario file isn't loaded** (L §6.4; L §8.3 LOAD-SCENARIO, round 17). `ask q …
   in other`, a `diff` naming one, and a scenario pin naming one, with no such world loaded, are `LOAD-SCENARIO ask q
   names other, neither the model nor a loaded scenario` (they were LOAD-WORLD); LOAD-WORLD keeps its own case. A pin
   on an unknown id outside an ask stays LOAD-ASIF. `L6_4_ask_world_not_loaded`, `_diff_world_not_loaded`,
   `_asif_world_not_loaded`.

**Round 17, checked edit by edit:**
- §2.1 and §2.4 numbers (`96 000`, `1 190.05`): agree (refcore printed the shortest decimal already; contradiction 39
  settled). `L2_4_reprint_expressions`' comment updated.
- §2.4 "A regime prints its items …, then its `export` line and its notes in the grammar's order": **changed**
  (`canon.sec_regime`: items, `export`, notes; notes were first). *Gone at round 19 (GD-1, writer 10).*
- §2.4 "Inside a section: its clause lines and notes first, in the grammar's order (§2.2)": **changed**
  (`canon.BODY_RANK`): an index's `v`, `assumed`, notes; a fill's `v`, notes, `supply` …; a move's notes last, after
  `fulfils`; a def's notes before its rules and a version's after its clause lines, as before. A kind's `leg`, `attr`
  and `note` lines are one repeated choice in §2.2 (`kline`), so they keep the file's order (contradiction 45).
  Contradiction 40 settled. `L2_4_reprint_bodies` updated (a fill `v` line added). No pinned test's hash moved.
- §5.8 "`x[*] = v` states every key of x that exists at its version's `known`": agrees (`evaluate.unstated_ok`).
  New `L5_8_star_row_later_thing` (a thing entering later); `L4_6_closed_world` has an index's key.
- §5.16 `1 .. n[m] step 1`: agrees (contradiction 32 settled). New `L5_16_random_items_no_step` (LOAD-DOMAIN without
  the step); `L5_16_random_items_load`'s comment updated.
- §5.19 a link judged "by its keys, `where` and `when`": agrees (contradiction 33 settled; `moves.link_tuple` read
  `when`, its and `obs_links`' docstrings updated; `L5_19_run_record`).
- §6.1 "A binding over a kind that holds both kinds meets the one movement once": agrees (contradiction 35 settled;
  `L6_1_same_rows_kinds`' comment updated).
- §6.1 The joint model's own movements ("every read of that place in the joint model from the movement's day on, a
  balance included, is a hole naming it, and every result that reads the place lists it under `left out:`"):
  balances and `low` agreed; **`flows` changed**: it printed the hole with no `left out:` line, and now takes a hole
  entry's register as a balance does (`mreads` flows). New `L6_1_joint_move_reads` (flows, low, a balance on the
  day). A read before the movement's day lists nothing (contradiction 44). Contradiction 34 settled.
- §6.1 a member's ask "answered by the member alone; as for every ask, no line reads one": agrees
  (`L6_1_member_ask_not_value`, `L6_1_member_scenario_ask`).
- §8.3 LOAD-SCENARIO `ask <q> names <w>, neither the model nor a loaded scenario`: built (fault 7).

**New REFCORE-UNSPECIFIED site** (`UNSPECIFIED.md` now lists 11):
- §4.6 (`evaluate.to_key`): a period read at a key position of an index whose keys it is no longer than, but none of
  which holds it (a calendar month across two months of an anchored index, `first_day[month_of(d)]` over `tmonth`).
  §4.6 gives the key whose period holds a date or a shorter period, RUN-KEY outside the index and LOAD-KEY for a
  longer period; this case is none of them. snap8 refused it RUN-DOMAIN, a message no row of L §8.4 has.
  `L4_6_period_across_keys`.

**Seen, not a fault given, left as it is** (fixed in the fourth pass, W8-SEEN): a line that isn't random, over a model with a move whose day is random,
reading a place that move doesn't book (`ask q: low_v`, `v` booked by a plain move) is refused LOAD-RANDOM at the
read, though E §7 step 1 shares such cells across paths: the main walk has no path, can't evaluate the random move's
day, and that instance's refusal holds for the whole walk (its places aren't known before its day is), so every
balance meets it. Building it means booking a random-day instance's literal places before its day, or a walk of the
non-random instances only.

**Readings taken:** the random cone's place test (a place booked through a binding or an expression matches every
place; a read of a place through a binding or a section matches any random place); flows takes a hole entry's
register (the walk's balances already did, `Walk.cum`).

**Contradictions**: 44, 45 under Contradictions and gaps; 32-35, 39 and 40 are settled by round 17; 36 is narrowed
(fault 4).

## Writer 8, second pass

The coordinator's second pass: the joint-model sites the spec states (L §6.1, §6.4, §2.4, E §3.1 step 1), in its
order. **774 tests, all pass** (42 new; `L6_1_pin_unspecified` removed, the pin being built); speed untouched. Where
this section and the older ones below differ, this one holds.

**Built**, each with a test naming its section:

1. **Members of several files, nested members** (L §6.1 "A model is one or more .tally files in one directory tree";
   E §3.1 step 1). `use a = "dir"` names a directory tree, every `.tally` file under it (`joint.member_texts`; the
   files given first, then the disk); its parts are merged as parts `a·<id>` (a law part stays a law part); its
   scenario files are hashed but are no part of its base (L §6.4). A member that is itself a joint model is loaded
   alone (recursive `joint.joint_texts`) and merged under `a·b·x`, read `a.b.x`, asked `a.b.q`; a member's
   bindings range over what it holds (`joint.owner_deep`, `owns`). Shared law: two members copying different
   versions of one law part are LOAD-SAME `members a and c copy different versions of law part pit`.
   `L6_1_member_directory`, `_nested`, `_nested_closed`, `_nested_pin`, `L6_1_shared_law_differs`,
   `_same_versions`, `L6_1_two_model_headers`.
2. **The canonical reprint and the pin** (L §2.4, E §3.1 step 1). `canon.py` reprints a reality (`refcore reprint
   <paths>`; test option `=== options reprint`); a member's pin is the sha256 of its files' reprints in the order of
   their paths, 64 lower-case hex digits or LOAD-PIN (`L6_1_pin_uppercase`); `--pin=bytes` stays a test
   convenience. A joint model is reprinted with its members' indexes (`a.ks` keys in a's order; `joint.member_hash`,
   `use_member_files`). The lexer and parser mark what the reprint needs (token roles `bin`/`u`/`lo`/`hi`, a
   number's canonical text, parenthesis tokens). `L2_4_reprint_*` (16), `E3_1_pin_canonical`, `_spacing`,
   `_mismatch_canonical`; `L2_2_rule_governs_open_end` (a fault the reprint tests found: `rule a governs 2027.. = 2`
   was LOAD-PARSE).
3. **A member's own units** (L §6.1 "Any other unit a member declares is its own (`a.hour`)"): renamed `a·hour`,
   unit-kind keys included; currencies are one by code, LOAD-SAME for two minor units. `L6_1_member_own_unit`,
   `_units_differ`.
4. **`same` of things and places whose entries differ, keyed places at keys** (L §6.1 Identities, What same does).
   Every side's entries are kept and read together: on a day they disagree a binding over the kind has an unknown
   member, HOLE-SAME `<p> and <q> are one, but the entry into <kind> differs on <d>: <span> (<row>) and <span>
   (<row>)` (`model.SameDiffers`); a member's own binding reads its own side (`b._owner`). A keyed place at its keys
   is joined at load (`loader.joint_place_sames`); a place stated same whose lifespans differ or that a side enters
   more than once keeps the other side whole (`J.place_others`, `loader.joint_place_sides`, `same_other`,
   `th.same_others`). In the walk: a leg on a day the sides disagree is a HOLE-SAME hole movement at that place; a
   start that is zero on one side and needs an anchor on the other is a hole until a reading; an end only one side
   states makes a nonzero balance a HOLE-SAME hole instead of RUN-LIFESPAN (`walk.alive_day`, `default`, `ending`).
   LOAD-SAME for a keyed place named without keys and for a thing written with keys that isn't a keyed place;
   RUN-RECORD for keys with no instance. `L6_1_same_thing_entries_differ`, `_entered_twice`,
   `L6_1_same_keyed_place`, `_no_keys`, `_lifespans_differ`, `L6_1_same_place_lifespans_differ`, `_ends_differ`,
   `_entered_twice`.
5. **A decision in a `same` group** (L §6.1: "a place two of them book … takes it at the point of the last of them in
   the day's order (§5.14.2), since a decision's amount is known only at its point"): an observation's ends wait for
   the decision's point (`walk.same_wait_for`, `same_release`); amounts or days that differ are a HOLE-SAME hole
   movement; a decision stated same as an observation lists no `unfulfilled:`. `L6_1_same_decision_waits`,
   `_disagree`.
6. **Asks across stacks** (L §6.4 "Inside an expression, `x as if {scenario s}` is x in s"; §5.18 "each `as if
   {scenario s}` in it"): a line's `as if {scenario s}` where s sits on another stack than the line's world is
   evaluated in the reality that holds s (`cli.make_cross`, `evaluate.e_asif` -> `self.cross`).
   `L6_4_asif_two_stacks`.
7. **`a.q` in an expression and a bare fill**: the refusal is kept, since L states it: "nothing reads an ask (P-12)"
   (L §5.18, §6.4), and a fill's output is `F.taken[…]` (L §5.12 Outputs). LOAD-PARSE, as for an ask of the model
   itself (`L6_1_member_ask_not_value`, `L5_12_fill_bare`).
8. **Other §6.1 sites**: a member's ask in one of its scenario files is asked as `a.q` by the member alone with all
   its files (`ask.member_ask` -> `cli.run_texts`; a refused scenario prints `scenario a.s: refused: …`),
   `L6_1_member_scenario_ask`. A member's place, movement instance, observed row and keyed band read by the joint
   model already work through the merge (every name a member declares is renamed `a·x`).

**Left REFCORE-NOT-YET**, by section:
- **§6.4, §5.18** (`cli.py:141`, `:301`, `:372`, `:376`, `evaluate.py:2381`): an ask written in a scenario file
  whose `diff` or `as if` names a world on another stack; an ask line whose `in` and pins sit on two stacks in other
  ways; an `as if` reached through a def from another stack. refcore loads one reality per stack (two stacks may
  declare one id: "Two scenarios over one base never meet", L §6.4) and checks a line in the reality that holds its
  file; answering it in another stack needs the line checked there, which refcore's loader can't do.
- **§6.1**: two places of one member stated same whose lifespans differ (`loader.py:1749`, `:2402`: the sides are
  kept per member). Guards: two entered places at keys (`loader.py:1734`, unreachable), a representative with no
  lifespan (`:1773`), a model loaded without the joint pass (`:128`, `:132`), the old member-read path
  (`check.py:1252`, `:1258`, `:1279`, `:2818`, unreachable through `run` and `test`).
- **§2.4** (`canon.py:210`): the order of keys of a member's index when the member's files aren't given to the
  reprint.

**New REFCORE-UNSPECIFIED sites** (9; `UNSPECIFIED.md` lists 10 with §5.16's, each with a test):
- §2.4, the order of: a rule whose `v` names no version of the reality; two rules of one label and v, one without
  its own governs; two such rules, one governing with no start (the version items' "a row with no span of its own
  first, then a span with no start" is said of version items only); a version's `same` item among its other items
  (it has no target id); version items keyed `*`, by a range or by a place keyed by things, against other keys; a key
  its index doesn't hold; a row governing `always` among rows of one target and keys with spans (`always` is no
  lspan: rowmeta `"governs" (lspan | "always")`); a row read `after m` among rows of one target and keys (m's day is
  a value, not text).
- §6.1: a header `same` line stating which version of a law part both members follow (contradiction 43).

**Readings taken** (one reading each; listed to be checked):
- E §3.1 step 1: a reality's canonical text is its files' reprints, each ending with its newline, concatenated in the
  order of their paths (by code point) with nothing between; the paths are relative to the member's root (a
  one-file member: its base name).
- L §6.1: a `use` path is relative to the joint model's file; the root model file is the one no use line names, and
  every file under a member's path is that member's.
- L §2.4, where it names no form: spacing beyond operators, commas and brackets is taken from the spec's examples
  (`f(x)`, `x[k]`, `a: T`, `?"q"`, `8 PLN/hour`, `19 %`, a version's span `2026-01..` touching, `a .. b` spaced in an
  expression); the order of repeated lines of one clause (legs, attributes, notes, `before`/`after`) and the spelling
  of a word the grammar takes in two forms are kept; kinds print parents before subkinds, the least id whose parent
  is printed next; no blank line inside a section or a regime; a four-digit integer with no unit is kept as written
  (a year and an int are one token before typing).
- L §2.4 instants: "by … their instant" in time, `at D start` before `at D end` (L §4.6: "before the day's movements",
  "after them").
- L §6.1, places stated same (item 4): the reads that depend on a place being alive are its legs, the zero at its
  start and its end-empty check; "an entry only one side states stands" is read per cut (a side whose entries the cut
  doesn't see yet states none); the `<row>` of a place section's side is that place.
- A member's ask in a scenario file prints the member's register unprefixed, as every `a.q`.
- L §5.17's example is not in the canonical form (no blank line between its kinds; its regime's places after its
  inputs): examples need not be, and refcore reprints them. *Round 19 (GD-1): §5.17 is a stub; the example,
  unboxed, is `L5_12_lots_example` (writer 10).*

**Contradictions**: 39-43 under Contradictions and gaps.

## Writer 8 in short

- **Works**: the grammar of L §2.3 with the removed forms refused (pending, in_force, meta, reasons, windows
  latest/earliest, within/HOLE-STALE, `split by days`, empirical, the day counts but act365, of_leg, first_business,
  real, paths(), `calendar removed`, a month-day in `added`, defeaters, a movement's `kind` field, the built-in ids
  freed: party_place, match_days, match_amount, bank_party, quarters); every item of the redesign review's list
  (`scratch/refcore-r8/changes.md`) whose text the spec states, listed below; the 18 faults of the queue (17 fixed,
  F12 not reproducible, each with a test); every UNSPECIFIED site but one; 10 NOT-YET sites built.
- **Refuses REFCORE-NOT-YET** (after the second pass, above): asks across stacks written in a scenario file; a priced
  reference with fields; fills over kinds or with a hole span; interpreter guards. The list is under Writer 8: still
  NOT-YET.
- **REFCORE-UNSPECIFIED**: 10 sites: §2.4 (8, the reprint's open orders), §5.16 (1, `discrete` over a key list or a
  range), §6.1 (1, a `same` line choosing a law part's version) (`UNSPECIFIED.md`).
- **Tests**: 774, all pass (`$PY -I -B refcore test`, about 12 s). Speed untouched.

## Writer 8: what changed

Every change has a test naming its section; tests writer 8 wrote or converted say so in their comments ("Writer 8").
By area (the spec's text is authoritative over the review's list where they differ; differences under Contradictions):

- **Removed forms** (L §2.2, §2.3, W-1..W-4, A-5, A-15, A-29): each refused as the grammar now reads (LOAD-NAME for a
  gone built-in call, LOAD-PARSE for a gone production, LOAD-FIELD for `kind`); `*_gone` tests. `pending` is replaced
  by HOLE-NOTYET (first, last, next over a binding empty at the cut whose span passes it). Freed ids are ordinary ids
  (`L2_2_freed_builtin_ids`). Quarters are a model's own index (`index qs = month 2026-01..2027-12 by 3`, L §4.1).
- **Order** (L §2.5, the fresh critic): things and an open index have no order a rule reads (`<` etc. LOAD-UNIT at
  load, RUN-DOMAIN at run; `==`/`!=` stand); binding order is the text order of stable ids; `first`/`last` over things,
  movements or an open index need `ties by` (RUN-TIE); `next` breaks no tie by keys.
- **Units on places** (L §5.10): a unit on a place whose kind isn't of unit `one` is LOAD-UNIT (`outside PLN` is gone;
  124 tests rewritten to `outside`, flows of an outside place through a `place PLN signed outside` kind).
- **Windows** (L §4.4): `windows each` reaches version `known` windows (entries, moves lines, statements,
  observations), move `at` windows and a projection's own clause; a window in `added` is a window the line meets.
- **Readings and lots** (L §5.17, §5.19, PL-8, PL-15): observed rows before S are bound and booked by none; HOLE-LOTS
  `<place>: the lots held before <S> sum to <a>, the opening reading states <b>` (`eval2.m_lots_holes`).
- **Observations** (L §5.19, the fresh critic): every observation is booked on its own day; one that fulfils a decision
  instance of the same day at that instance's point; a modelled-past decision is computed against the balances less
  its own observations (`walk.less_obs`, `mreads.less_own_obs`). An observed row is in its kind and the kind's parents
  (`o in K`, PL-18); a binding over an observed input ranges over every row, a `same` pair included (OQ-3); rows
  stated `same` with different kinds are an unknown member HOLE-SAME in a binding over either kind (OQ-2). A `fulfils`
  link is judged at the ask's cut by the instance's keys, `where`, `when` and leg places, never amounts (OQ-1, W-D-F2).
- **The sign rule** (L §5.10 point 1): checked once after the day's first batch, taken together; RUN-PLACE `<place> is
  <b> after the day's observations and plans of <d>`; a refused batch's deltas are rolled back.
- **Decisions** (E §4.2 step 3, F11): within an unordered cycle, instances are ordered by the places their legs give;
  a `when` false orders nothing; mutual reads are RUN-ORDER/RUN-TIE. A hole-day decision is a hole only at its legs'
  places (F10).
- **Randomness** (L §5.16, E §3.1 step 10): `mean(d)` of a distribution value is the table's mean; a closed-form query
  stands in a rule (one with no closed form is LOAD-ASKONLY `<query> has no closed form outside an ask`); an ask line's
  own projection reads samples on a path; a decision's projection with neither clause nor query is RUN-FUTURE; a line
  random with no query is LOAD-RANDOM, found at the read (`errors.RandomLine`: a value's refusal inside a walk, a load
  refusal for the line that meets it, since refcore's load marks the random cone over inputs and defs only); a random
  number of items (LOAD-RANDOM `the draw <x> can exceed the last slot of <index>` at load, RUN-KEY on the path);
  `to u` on a closed-form quantile of a·X + b and on a mean resting on rounded means, each rounded once from the real.
- **Codes and days** (PL-17, PL-11): a code position takes any expression of the code's type (RUN-DOMAIN for a wrong
  family at run); a def keyed by days is read at a computed day (RUN-KEY outside its range).
- **Versions** (L §5.3, §5.11): a row enters into force at the later of its version's `known` and the start of its own
  time (wording pins per row); LOAD-VERSION `replace rule <d>.<r> in <V> names version <W>`.
- **Adjustments** (L §5.14.3, E §5): settlements are never measured; a settlement of a moved leg signs by the way it
  runs against the leg's places before and after (turned round: RUN-RECORD); an adjustment's ends include the new
  places; versions known the same day make one adjustment named under the first in ASCII order.
- **Residuals** (E §6.3): an adjustment or instance the cut doesn't hold is unexplained; an undated fulfilled instance
  is matched in its observation's month; `same` rows are listed once, naming both.
- **Explain** (A-10): a `min`, `max`, `argmin`, `argmax` is explained by its first winning term in binding order
  (`evaluate.trace_winner`). **Pins** (OQ-8): the pins one def forms under many `as if` print once, days merged into
  spans (`ask.merge_pin_days`; the layout `= a .. b, c` is refcore's).
- **Scenarios** (L §6.4): lifespans may change only from `from` (PL-13, `loader.check_scenario_lifespans`, LOAD-SCENARIO
  `<s> changes who is alive in <K> on <d>, before <from>`); two scenarios over one base may declare one id, asks
  included (PL-16, `cli` keys asks by id and world); a scenario over a joint model replaces members' rows by their
  prefixes (`cli.joint` runs worlds over the merged lines). The row message is L §8.3's.
- **Joint models** (L §6.1): a member's bindings range over its own things and movements, the joint model's over all
  (OQ-5, `b._owner`, `evaluate.member_owns/member_moves`); qualified refs, prefs and binds and member-row pins (OQ-4);
  the joint model's own movement into a member's place is booked only where `same` links it to that member's own
  movement, otherwise a hole `<m> crosses to <member> with no link` with `left out:` (OQ-7, SF-32); the merge renames
  only tokens the member's parse read as ids (W-E1-F1); a member's own calendar where its run differs (W-E1-F3).
- **Faults** (`scratch/refcore-r8/faults.md`): F1, F7, F8, F9, F10, F11, J-A-1, J-A-2, W-B-1, W-B-2, W-B-3, W-D-F1,
  W-D-F2, J-D-1, W-E1-F1, W-E1-F2, W-E1-F3 fixed (tests named after them, or after the section for F11 and J-D-1);
  F12 doesn't reproduce (`F12_windows_each_axis`).
- **NOT-YET sites built**: a member's ask read in an expression (LOAD-PARSE); a fill read bare (LOAD-PARSE); a priced
  reference with keys (`check3.resolve_price`); arithmetic on a distribution value (LOAD-UNIT); the two `to u` sites;
  scenarios of a joint model; SF-32; a sample outside a path (LOAD-RANDOM); a member's keyed band (works through the
  merge's renaming, `L6_1_member_keyed_band`).

## Writer 8: still NOT-YET

- **Joint models, asks across stacks**: see Writer 8, second pass (Left REFCORE-NOT-YET); everything else of the
  first pass's list here is built.
- **Fills**: a keyed fill or a side over a kind or movements (`check2.py:1325`, `:1332`), a fill read in its own supply
  (`check2.py:1419`), a side over an index whose span is a hole (`eval2.py:1744`).
- **Numbers**: `to u` on a closed form resting on rounded values other than a mean or an affine quantile
  (`eval2.py:895`); a crps bound over more than 4 000 rounded values (`eval2.py:1194`).
- **Other**: a priced reference with fields (`check3.py:826`); the value of an observed row of no declared type
  (`loader.py:2452`); `explain low` with no day (`ask.py:1297`, unreachable); `tally N` other than 2 (`loader.py:120`).
- **Guards and dispatch fallbacks**: LAWGUARD (six places), the axis guard (`evaluate.py:367`), the million-key range
  (`evaluate.py:2637`, `:2660`), `windows each` over 20 000 combinations (`ask.py:617`), the stack (`cli.py:122`),
  unexpected inputs (`cli.py:127`), and dispatcher fallbacks no checked text reaches.

## Writer 7 in short

- **Works**: rounds 12, 12b, 13b and 14 as far as one-file members go; every triage site but T-74; the faults found;
  writer 6's open items (per-line `unfulfilled:`, known windows as holes, `same` groups). Details under What works.
- **Refuses REFCORE-NOT-YET**: the canonical pin (`--pin=bytes` is a test convenience, not language); directory and
  nested members; scenarios over a joint model; `same` of a keyed place; things whose entries differ (G-ENTRIES); a
  decision in a `same` group; a joint movement in a member's place (SF-32); a member's own units and keyed bands;
  asks across stacks other than a `diff`; interpreter guards. The list is under Refused REFCORE-NOT-YET.
- **REFCORE-UNSPECIFIED**: 6 sites (from 72): §5.14.3 (2), §5.16 (1), §5.19 (1), E §6.3 (2). Contradictions 24-31.
- **Speed**: dropped by the coordinator; no slow text met (657 tests in about 8 s).

## Running

    PY=/Users/jswob/projects/trop/engine/tools/parsecheck/.venv/bin/python
    cd /Users/jswob/projects/trop/tmp/design2
    $PY -I -B refcore run a.tally b.tally [--pin=bytes] [--ask id ...]
    $PY -I -B refcore test [substring ...]
    $PY -I -B refcore reprint a.tally b.tally      (each file's canonical text, L §2.4, then the reality's sha256)

(`scratch/refcore-r4/py` is a two-line wrapper for `$PY -I -B`: `scratch/refcore-r4/py refcore test`;
`scratch/refcore-r4/show.py <test or .tally> [ids]` prints what refcore answers.) `__main__.py` puts the parent
directory on `sys.path` (python `-I` hides the cwd); `cli.py` runs everything in a thread with a large stack (deep
recurrences). Standard library only; lark is not used. The whole suite (657 tests) runs in about 8 s. Writer 5's
helpers: `scratch/refcore-r5/show <test or .tally ...> [--ask ids] [--debug]` prints what refcore answers;
`scratch/refcore-r5/tb.py <test> <substring> [asks]` prints the stack where a refusal holding the substring is raised.
Writer 6's: `$PY -I -B scratch/refcore-r6/show.py <test or .tally ...> [--pin=bytes] [--ask ids] [--debug] [--tb
text]` (a test's `=== options` apply); `scratch/refcore-r6/mk.py` (`mk(name, comment, files, expect, options=)`).
(`-I` hides the cwd, so `$PY -I -B -m refcore` fails from inside `tmp/design2`; run `refcore` as above from there, or
`$PY -I -B -c "import sys; sys.path.insert(0, '/Users/jswob/projects/trop/tmp/design2'); from refcore import cli;
cli.main(['test'])"`.)

`UNSPECIFIED.md` is generated: `$PY -I -B scratch/refcore-r7/gen_unspecified.py` (writer 4's script, updated by
writers 5, 6 and 7: its AST scan is `scratch/refcore-r7/unspec_sites.py`, and a minimal text per site is kept in the
script; it runs `scratch/refcore-r7/show.py` on each test that expects a REFCORE-UNSPECIFIED refusal; rerun it after
moving code, since it records file:line). Writer 7's helpers: `scratch/refcore-r7/show.py` (writer 6's, same use),
`mk.py` (`mk(name, comment, files, expect, ids=None, options=None)`) and `mkjoint.py` (`mkjoint(name, comment, sames,
body, expect, …)`: a joint model over two members `ana` and `ben`, run with `pin=bytes`); `PROGRESS.md` there.

`--pin=bytes` is **not language**: L §6.1 and E §3.1 step 1 pin a member by the sha256 of its canonical text, which
refcore computes by default since writer 8's second pass (`canon.py`). The option (sha256 of the member files' UTF-8
bytes, in path order) is kept so the older joint-model tests keep their pins; a test that states it says so in `===
options`. Writer 8's `scratch/refcore-r8/pin.py <test names>` fills a test's use-line pins (canonical, or bytes under
`pin=bytes`; it skips a hash of one repeated digit other than 0 and a test marked `# keep-pin`, both meant not to
match). `UNSPECIFIED.md` is now generated by `$PY -I -B scratch/refcore-r8/gen_unspecified.py`; since writer 10
by its copy, `$PY -I -B scratch/refcore-r10/gen_unspecified.py`.

## Layout

| file | what it holds |
|---|---|
| `errors.py` | `TallyError` (code, msg, loc); `LoadError`, `Refusal` (`refuse`), `Unspecified`, `NotYet` |
| `lexer.py` | tokens, indentation (L §2.1: tabs, id length, number forms, `%`, dates, month-days) |
| `node.py` | the AST node (`Node(k, **fields)`, `.loc`), `walk` |
| `parser.py` | hand-written recursive descent over the whole grammar of L §2.3, header first (a scenario header too); `explain bal(…)`/`explain low(…)`; a scenario row's expression value (`rowexpr`, L §6.4) |
| `units.py` | units as sorted (base, power) tuples; `%` folded to 1/100; year/quarter/month one dimension, day separate |
| `dates.py` | dates, months, years (anchored `from MM-DD`), periods, calendar arithmetic |
| `values.py` | `Num` (with `enc`, the enclosure of the true value, L §7.3; the 4 096-digit cap of L §7.1 checked by bit length, never by printing), `Money`, `Code`, `Key`, `Tup`, `Rec`, `Iv` (date windows), `Hole`, `NONE`/`PENDING`/`NEVER`; `fmt` |
| `ty.py` | static types for the checker (`dist` carries `u` and, for `discrete`, `ix`) |
| `model.py` | the loaded model: indexes (open ones too), kinds and things (keyed places as `PlaceThing` named `konto[ana]`), inputs and rows, defs/rules, bands, axes, versions, sources, fills, moves, observations, readings, `joint_members`; the world filter of scenarios |
| `loader.py` | files → model: header, scenario headers, names, versions (in known order), rows (a scenario's expression row as `('sexpr', e)`), entries, band tables (`BandKey`, `order_key`: bands over keys and periods), observed rows, `covers`/`pad`/`anchor`, `moves V to SPAN`; `prescan_open_keys` (an open index's keys from its rows, before places are built); `load_texts(texts, members)` |
| `check.py` | the load-time checker: types and units (L §3.2), the literal rule (L §7.1), law dates (L §4.3, an input whose rows all govern always is timeless; a band too, `banddated`, writer 9), guards, cycles, LOAD-ALIVE, LOAD-FORKS, asks (LOAD-WORLD, diff, residuals, `explain bal/low`), scenario reads (LOAD-SCENARIO), key lists (`kl_value`, a key list of an int index compares with int), `member_read` (a joint model's `a.item`); at its end `check2.install`, `check3.install` |
| `check2.py` | writer 2's checks (Checker methods `m_<name>`): `to u`, transcendentals, rates, distributions, random reads, closed forms (LOAD-RANDOM), LOAD-TO, LOAD-ORIENT, sample pins, crps, prorata (over lists and ranges too), fill (list and range domains), solve, `paths()`; `check_scen_rows` (L §6.4 expression rows: a prefixed name is rewritten to the base's item, read in the base world) |
| `check3.py` | writer 3's checks: places, moves, `bal`/`flows`/`value`/`low`/`next`/`covered`/`gap`/`observed`/`unit`, LOAD-READS (E §3.1 step 7 marks through defs), LOAD-ORDER, LOAD-COVERED, LOAD-COVERS, LOAD-GAPROLE, LOAD-PRICE, `at` reader defs reached from a move (`at_defs`, LOAD-LAWDATE), `value … by <expr>`, the unit of a keyed place, decisions and strata |
| `evaluate.py` | the evaluator (`Engine.ev`): memo per (item, keys, context) with axis trimming (`def_axes`), the read rule (L §4.6), defs and rules (L §5.11), aggregates and bindings, calendars and windows, `as if` (also traced, `asif_traced`), pins (`pin_lines`), holes as values, the error-bound arithmetic; the world switch of a scenario's expression row (`_world`), `fill_trace`/`walk_cells` (for `explain`), `member_read` (a member's item, evaluated by the member's own `Engine` at the reader's law date and cut); at its end `eval2`, `moves`, `walk`, `mreads`, `resid` install themselves |
| `eval2.py` | writer 2's evaluation: `to u`, transcendentals, rates, distributions, samples (SHA-256), sample pins, closed-form queries, path statistics, crps, prorata, fill (`step` and a log per take, for `explain`), solve |
| `moves.py` | movement values (`Mv`), plan instances (`Inst`; a day that is `none` or `never` gives no instance, L §4.2), statuses, observations and `fulfils`, readings, coverage, adjustments per move (`adjust_move`, so a plan whose amount reads `flows` of another plan has no cycle) and their settlements (`settle`, idempotent) and waivers, HOLE-CHANGED; `check_schedule` (LOAD-SCHEDULE); `inv5` (E §9 invariant 5) |
| `walk.py` | the day walk (E §4.2): logs per (place, unit), entry keys, openings, pads and anchors, decisions in strata; E §9 invariants 1 (conservation, legs booked by their ends left out), 2 (`inv2_add`/`inv2_check`: net worth inside changes only by boundary legs, pads, anchors and openings), 3 (`inv3`: a scenario equals its base before `from`), the timeline `signature` for invariant 4; WARN-NEGATIVE, WARN-LEGS, RUN-LIFESPAN, RUN-PLACE; `main_walk`, `proj_walk` |
| `mreads.py` | reads of movements and places: `bal`, `low`, `flows` (place, move, a named leg `flows(buy.eur, …)`, `for`, `in CUR`; a plan's own adjustments settled per move), `value`, `next`, `covered`, `gap`, `observed`, `unit` (a key of the unit kind), projection clauses |
| `resid.py` | the residual ledger (E §6.3) |
| `reals.py` | correctly rounded reals: fixed-point intervals refined W = 64, 128, … 2^16 (RUN-UNDECIDED past it) |
| `dists.py` | the closed list of distributions (L §5.16) |
| `ask.py` | asks (L §5.18): fan-out over axes (also the axes a `for … where` reads, `binds_over_axes`), `for`, `check` (a law per side), `explain` (defs; movement instances; a balance and a projection, `explain_balance`; a fill take, `explain_fill`; a walk-bound def, `explain_walk_bound`; an ask's `with` pins and `as if`, `explain_pins`), `inputs-of`, `readers-of`, `solve`, `partial`, `law`, `as known`, `with`, `windows each` (axes too, `EachRes`), `paths`/`path`, `orient`, `in S`, `diff`, `residuals`, a member's ask `a.q`; the register (L §8.2); the invariant 4 check |
| `joint.py` | writer 6: a joint model (L §6.1) merged into one model. `pin_check` (E §3.1 step 1); each member's declared names renamed `alias·name` (U+00B7, printed `.`), its header turned into `part alias·member` + `tally`, its asks dropped (asked as `a.q` by loading the member alone); header `same` lines of kinds, things and places joined before parsing (`plan_joins`: the second side's declaration and entries dropped, references rewritten; `LOAD-SAME` for places of two kinds); `JointInfo(groups, owner_place)` for consolidation. Writer 8's second pass: `split_reality` (the root model file), `member_texts` (a file or a directory tree), `pin_check`/`member_hash` (the canonical hash, `canon.py`), nested members (recursive `joint_texts`, `owner_deep`, `owns`), `shared_law`, a member's own units, things kept with every side's entries (`JointInfo.things`), places whose lifespans may differ (`JointInfo.place_others`) |
| `canon.py` | writer 8: the canonical format (L §2.4) and the reality's hash (E §3.1 step 1): `reprint(files, members)`, `reality_hash(files, mode, members)`; `Table` (indexes, kinds, versions, a member's indexes), `FilePrinter` (header, sections in kind order, versions by known, bodies, version items, rules, the short form, numbers, expressions with the parentheses the parse needs); `Open` sort components whose order L §2.4 doesn't give (REFCORE-UNSPECIFIED when an order rests on them) |
| `cli.py` | `run_one`, `run_worlds` (L §6.4), `joint` (L §6.1: the pin check, `joint.joint_texts`, `loader.load_lines`, then the asks; a member's `a.q` from the member loaded alone), `run_texts(texts, ids, options)` |
| `testrun.py` | the test runner (`parse_test(path, with_options=True)` reads `=== options k=v`) |
| `tests/*.test` | 774 tests, named by section (`L5_14_move.test`, `E6_3_residuals.test`) |

Writer 2's tests are generated by `scratch/refcore-r2/gen/w2tests.py`; writer 3's scratch models are in
`scratch/refcore-r3/`; writer 4's probes and helpers in `scratch/refcore-r4/` (`tb.py`: the stack of the first
refusal matching a text; `inv_probe.py`: shows invariant 2 firing on a simulated engine bug; `PROGRESS.md`).

### Test file format

    # comment: the section and the spec text the expected value comes from
    === options pin=bytes        (optional, before the files: run as with --pin=bytes)
    === options reprint          (optional: print the files' canonical text, `--- path` before each, then
                                  `sha256 <hex>`, instead of answering asks)
    === name.tally
    <file text>
    === expect [ask ids]
    <expected output>

Several `=== x.tally` blocks make one reality (a file whose header is `scenario s over b from D` is a scenario; a
file named by a `use` line is a joint model's member). An expected line ending in ` ...` matches one line by prefix.
A bare unindented `...` line matches any number of lines (including none). Warnings are compared only when the
expected text has a `warning:` line. A test's text is each block right-stripped plus a newline (what `pin=bytes`
hashes).

### Output format (presentational, not fixed by the spec)

- `<ask> (var = key, ...) [axis = options]: value`; a tuple prints `(v1, v2)`; `*` for all options of an axis; a
  movement prints its id `wage[2026-03]`; `pending`, `none` as words.
- register lines indented two spaces in this order: `assumed`, `given`, `missing`, `pinned`, `pinned sample`,
  `sample`, `world`, `reading`, `anchor`, `pad`, `adjustment`, `unfulfilled`, `open`, `left out`, `uncertain`, `held`;
  then `law date D, cut K` (a `check` with a law per side: `law date L (left), R (right), cut K`), then `orient`. A
  joint model's register names a member's items `a.item`.
- `≈ <value> ± <bound>`; queries over paths as in writer 2's notes.
- `diff`, `residuals`: as writer 3 left them.
- `explain` of a movement instance (E §10): class, day, booking cut, status, legs, attributes, derivation,
  adjustments; for a decision its stratum, walk point and the places it read.
- `explain bal(x, d)`: `balance of x (U) at the end of d`, `from:` the opening, reading, pad or anchor it starts at
  (with its row, version, source, locator and gap), then per day the entries (`inst.leg (status): amount`), then the
  register. `explain bal(x, d) from t` / `explain low(x, span) from t`: `projection of x (U) from t …`, `holder:`,
  `clauses:`, `start:`, `counted:` per day, `left out:` (E §10 rows for a balance and a projection).
  `explain low(x, a .. b)` without `from` (writer 5): `lowest end-of-day balance of x on a .. b: V, on d1 (through
  d2)`, then the balance explanation at d1.
- `explain` of a def: a cause prints in full where it is first printed (rules in canonical order) and as `(x: see
  [n])` after, whatever order it was evaluated in (L §5.18).
- `explain` of a fill read: `fill f, v …`, `arcs in order:` each arc with supply and demand before and after its
  take, the asked take marked `<- this take`.
- `explain` of a walk-bound def: `walk-bound, read at n walk point(s)`, one line per walk point (day, stratum,
  decision instance) with its value.
- `explain` with pins (an ask's `with`, an `as if`): numbered nodes `[n]`: the pins evaluated outside them, the
  value inside, the value outside; a repeated derivation prints `(x: see [n])`.
- `windows each` whose windows depend on an earlier window's day (writer 6): one line per nested combination,
  `q [a = d1 .. d2, b[k] = e1 .. e2]: v`, the earlier window first; consecutive days of a window merge when everything
  under them is the same.
- a joint model prints a member's items `a.item` (the merged name `a·item` with `·` printed `.`); a member's
  `WARN-UNREAD` lines are dropped (the member is checked alone).
- a hole prints `?"question" (origins)`; a refused line `refused: CODE message`; a load refusal is the single line
  `refused: CODE message (file:line)`; a scenario's load refusal is `scenario s: refused: ...`; a member's load
  refusal `refused: member a: ...`.

## What works

Writer 1 (core): the whole grammar; header; names, shadowing, key literals; types, units, money, rates, prices,
inline figures; indexes, kinds and things, lifespans, `on`/`over`; versions, rows, `replace`, entries, closed world;
bands; the read rule; law dates; cuts and known windows; windows, `one_of`, `latest`/`earliest`; calendars and
schedules; defs, aggregates, `missing`, `as if`; axes; asks; the register; rounding (regimes and export until
round 19, GD-1).

Writer 2: `to u` and transcendentals with their error bound; rates; distributions; random facts, samples, sample pins;
closed-form queries and queries over paths; crps; prorata; fill; solve.

Writer 3: places, moves, decisions and the walk, projections, adjustments, observations and readings, residuals,
scenarios and diff, the joint model's pin check, E §9 invariant 1, `explain` of a movement instance, the household.

Writer 4 (this round):
- **Rounds 7, 7b, 7c of the text.** `explain bal(x, d)`, `explain bal(x, d) from t`, `explain low(…) from t` (E §10);
  date arithmetic (a period moves by an int or a whole number of its periods); L §4.3 rules 1–2 (an input whose rows
  all govern always needs no law date, in a def or a move); L §4.6 step 1; `LOAD-LEAF` for entries only (a movement
  may be of a kind with subkinds); L §5.9 replaced rows (all match earlier intervals, or a whole new table; a mix is
  `LOAD-BAND`); `value … by <ref or expression>`; `unit(x)` is a key of the unit kind (`unit(x) == PLN`), refused for a
  place of unit any; `prices Y by idx` with an input or a def keyed by one time index; L §5.14's `pay_lead` example,
  a move's window `at` read once per law-date class, `reads` of a keyed place or a move named without keys,
  `at` reader defs reached from a move (and `LOAD-LAWDATE` for one reached from `at e` itself); L §5.14.1's
  `RUN-CYCLE` on the decision's own day and `RUN-FUTURE` after it, `LOAD-CYCLE`; L §5.14.3's adjustment attribute
  types, `flows(m[keys], span)` of one leg (`flows(buy.eur, 2026)`, `LOAD-LEG` for an unnamed leg of a multi-leg
  kind; on covered days the fulfilling observations stand in), `next(K, d)`'s same-day order; L §5.19 `covered`
  (`LOAD-COVERED` only for an aggregate of an observed input; a binding over an event kind needs none); L §6.4
  scenario expression rows (a prefixed name reads the base, a bare name the scenario; `WARN-STALE` dropped); the
  household (L §6.5) base and changes to the new text; E §3.1 step 7; E §5's clawback; E §9 invariant 1.
- **Faults found by running texts**, each a test: an axis read in an ask's `where` (L5_5_axis_in_where); a key-list
  variable compared with an int (L2_5_keylist_compare_int, …_year_span, …_not_int); a unit load can't show checked at
  the key (L3_2_dyn_round_at_key); the 4 096-digit cap (L7_1_digits_product, L7_1_digits_print); `value … by
  etf_price[month_of(d)]` (L5_10_value_by_expr); speed (axis trimming in `read_def`); key `*` int (L4_2_key_times,
  L8_4_period_times); a refusal outranks a hole (L8_4_refusal_outranks_hole); `≈` on a keyed mean
  (L7_3_bound_mean_keys); `assumed:` on every keyed read of an assumed index (L5_6_assumed_hole); the unit of a keyed
  place's balance (L5_10_keyed_place_unit); `prices … by` a def (L3_4_prices_def); a plan whose amount reads `flows`
  of another plan (L5_14_3_flows_of_plan, …_adjusted).
- **Built from the NOT-YET list**: `explain` of a balance, a projection, a fill take, a walk-bound def and an `as if`;
  scenario expression rows; a move keyed by a stepped range; a place keyed by an open index (and an open index's keys
  from its rows, `LOAD-KEY` for a key no row brings); E §9 invariants 2–5 checked on every run (`ENGINE-INVARIANT`;
  invariant 3 compares a scenario with its base at the end of `from` − 1 when no decision falls before `from`);
  `LOAD-SCHEDULE` for a move whose `on` is a schedule with no date in [S, E]; writers 1–2's leftovers: `check` with a
  law per side, `windows each` on a line that reaches an axis, prorata over a key list or a range, fills over a key
  list or a range, a band whose argument is a key or a period; `use` joint models (a member's inputs and defs read
  as `a.item[keys]`, a member's asks as `a.q`).

Writer 5 (rounds 8 and 8b, and faults found by running texts):
- **Round 8.** L §5.7 Bindings: a binding `on`/`over` a hole binds nothing where no thing of the kind known at the
  cut could be in it (`Engine.bind_day`, `kind_has_member`; a member its `where` rules out whatever the day doesn't
  count, L §8.1 "stops propagating"); this gives L §6.5 change 3's old view at `as known 2028-06-14` (`old: 0.00 PLN`,
  `L6_5_change3`; `L5_7_bind_hole_day`). L §2.3 `row`: an expression row outside a scenario's version is
  `LOAD-SCENARIO` (`L6_4_expression_row_base`). L §6.4 `from D`: a scenario's move books nothing before D, its earlier
  instances left out and listed (`moves.m_before_scen_from`, status `leftout`; `L6_4_scenario_move_before_from`),
  `LOAD-SCENARIO` where its day reads only its keys and it has no `when` (`L6_4_scenario_move_load_day`). L §5.18
  `explain low(x, span)` without `from` explains the day of its lowest plain balance as a balance
  (`Asker.explain_low_plain`; `L5_18_explain_balance` e5, `L5_18_explain_low_plain`). L §5.14 `on d` none: no instance
  (`L5_14_on_none`). L §8.3 LOAD-BAND's x: the first point two rows share or the first uncovered point between rows
  (`loader.band_defect`; `L8_3_band_x_*`). Parse rule 4: keys and fields after `value(…) in CUR by p` belong to p
  (`L2_3_rule4_value_by_field`).
- **Round 8b.** L §4.6 step 1: only an input keyed by one period index is tested at its time key; a def (its givens and
  rules), a band and an input keyed by none or by two are tested at the law date (`Engine.time_point`;
  `L4_6_two_time_keys`). L §8.1 "It carries": a read at a hole key carries the key's questions only (already so;
  `L8_1_hole_key_carries`).
- **Faults**, each a test: an axis reached through `flows`/`bal` of a place (the cone, `Checker.reach_axes`: the
  `('balances',)` node reaches every move, E §3.3 "the axes the moves reach"; `L5_5_axis_through_flows`,
  `L5_5_axis_picks_leg_place`); a movement refused before its places are known was left out of a place's flows in
  silence (`flows_place` now reads the walk's global log, as `pstate` does; `sched_refused` refuses every place when a
  leg's place itself is refused; `L8_4_refused_leg_place_flows`); a settlement whose leg ends are
  `a.adjusts.cash.from`/`.to` for an instance a later version made exist (`m_made_to_exist_leg`: the adjusted leg's
  places, booked at A0 = 0; `L5_14_3_settle_adjusts_ends`); `explain` crashed (KeyError) when a cause was evaluated
  under one rule and printed first under another (`Asker.explain_node`; `E10_explain_cause_order`; projections to the
  run's last day, `E10_explain_projection_run_end`).

Writer 6 (rounds 9, 10, 11, 11b, 13; faults; the current text's settlements of old UNSPECIFIED sites; 96 new tests):
- **Round 9.** LOAD-BAND's x (`loader.band_defect`: an open end, the first row out of order, the smaller point;
  `L8_3_band_x_*`); LOAD-SCENARIO's catalogue messages for an expression row and a move before `from`.
- **Round 10.** Prices (L §5.10): `priced` on a kind, a place and an entry, `price = ref | last(ref) [within …]`
  (`parser.price`, `model.price_of`, `check3.check_priced/resolve_price`, `mreads.priced_read`, `value` rewritten);
  an asset whose price nothing names is `LOAD-PRICE` (`L5_10_asset_no_price`). E §4.2 step 5: a refused leg refuses
  the reads of its places (`walk.bad_legs`, `mreads.leg_refused`; `E4_2_refused_leg_reads`).
- **Round 11: joint models (L §6.1), built** (`joint.py`, see Layout). The pin: a `use` line's sha256 that isn't 64
  hex digits is `LOAD-PIN`; otherwise what the hash is taken over is `REFCORE-UNSPECIFIED E §3.1`, unless the run says
  `--pin=bytes` (sha256 of the member file's UTF-8 bytes). Member paths are relative to the joint file. `same` in the
  header joins kinds, things and places (one item, bound once); `same` in a joint version joins rows (read together:
  the joint model's own row settles them, two that agree are one, two that disagree are `HOLE-SAME` naming both) and
  move instances (`loader.joint_sames`: `same_rows`, `same_moves`, `same_settle`); a joint row on a member's item that
  no `same` joins is UNSPECIFIED §6.1. Consolidation in the walk (`walk.consol`): of two instances stated `same`, the
  left counts and the right's ends at places the left touches are skipped (equal amount and day) or `HOLE-SAME`; a
  member's movement whose end sits at a place another member owns, with no `same`, makes the balances it reaches a
  hole "`<mv>` crosses to `<member>` with no link". Prelude kinds range over every member; bindings count a `same`
  pair once (`mreads.once_by_same`). Tests `L6_1_*` (24).
- **Round 11b.** (a) L §4.6 step 2 "the same keys" are keys stated alike; step 3.1 a scenario's higher layer, 3.2 the
  narrower row (`loader.narrower`, `row_layer`; `evaluate.choose`; `check_overlap` skips what precedence orders); (b)
  L §6.4 `from D` bars only base-declared items' rows before D; (c) a base item reaching scenario things through a
  binding is not LOAD-SCENARIO (`check_scenario_reads`); (d) L §8.2 `pinned:` per binding tuple prints the pin as
  written with its binding (`L8_2_pinned_per_tuple`).
- **Round 13.** Defeaters: a rule with `beats` and no value (`parser.rulebody`; it takes part in steps 4–5 only;
  `L5_11_defeater*`, `L2_3_rulebody_no_value`); L §6.3 item 2: two same-day rows, a span over `always`, load and
  precedence orders them (`L6_3_same_day_span_over_always`).
- **Faults found by running texts**, each a test: a plan deferred before a decision (`walk.defer`), a priced fault,
  a lawless input with no rows, key-list days in `bal`, a unit key compared, `flows` of an adjusted plan
  (`moves.settle_candidates`), a refusal outranks a hole in `crps`/`cdf`, the `crps` enclosure, the gap's register
  (`walk.booked_coll`), a scenario move's `on`, `WARN-UNREAD` across realities, a key times money, a world named
  after an ask's `for` (`parser` stops the binding at a closing `in ID`), `as if {scenario s}` in the base.
- **The current text settles old sites** (each a test): L §4.6 step 5's no-law question `x: no value`; L §6.4
  expression rows read inputs, bands and defs, and a place, balance, movement or observed input, directly or through a
  def, is LOAD-SCENARIO (`L6_4_expression_row_reads`); a scenario move whose window's first day or leg's own day is
  before `from` is left out; `last`/`since` with no reading is HOLE-NOREADING; a band with no row for its argument is
  HOLE-UNGOVERNED `<b>: no row holds <x> at <d>`; HOLE-ANCHOR with no reading `<place> has no opening or stated
  reading`; a law, known or wording pin, or a pin key, that is a hole makes the line that hole (and a sample pin's);
  a binding over planned instances and their observations (L §5.19); the `for … where` hole line (also in `diff`);
  nested `windows each`; known windows over rules and band tables; LOAD-LAWDATE for an ask line reading a law-dated
  input or band directly; LOAD-INSTANT; LOAD-KNOWN for a row known before its version; LOAD-PARSE for a weekday word,
  for `first`/`last` with a term and for `prorata` with two bindings; LOAD-MISSING for `missing` of a def;
  LOAD-KIND for a move or observed row of `adjustments`; LOAD-FIELD for meta's fields on a movement; LOAD-LEG for a
  window in an observation's or a leg's `on`; quarters print `2026-Q2`; HOLE-CALENDAR for a window or `one_of` added
  day; `day_of` of an anchored key is RUN-DOMAIN; prorata's left-out key (0) and negative total (`rest largest`,
  symmetric); a stepped range's written key (LOAD-KEY outside it) and computed key (RUN-KEY); `meta`'s `version` and
  `source` as text; `paths()` without paths (LOAD-ASKONLY), a query on a `path p` line and a closed form of a pinned
  sample (LOAD-RANDOM); `observed()` with legs between different places (`none` there); a balance reading of `none`
  (no balance; with `replace` it removes the reading).

Writer 7 (rounds 12, 12b, 13b, 14; writer 5's triage sites; faults; writer 6's open items; the suite 573 → 657 tests):
- **Round 12, 12b, 13b.** The quarter literal `2026-Q1` (`lexer` QUARTER, node `quarter`; `L2_1_quarter_literal*`);
  `bond30_360`, `us30_360` (February's last day, L §3.3), `e30_360`, and negative spans (`L3_3_dcf_us_feb`); weekday
  words `mon` … `sun` (`L4_2_weekday_words_fri_sat`); a period's last day is its end less a day (`L4_2_period_last_day`).
  Adjustments are named `adjustments[m[k].leg, V]` (`moves.AdjMv.id`, `loader.adj_ref`): a planned or observed
  movement's `fulfils` settles one; a settlement leg counts by the adjusted leg's places, RUN-RECORD when it touches
  neither (E §5 step 5); a version that changes a leg's places makes an adjustment with δ = 0; a made-to-exist leg is
  the last `made` step's (`L5_14_3_adjustment_*`, `L5_14_3_settle_*`). Stepped-range keys: LOAD-KEY for a written
  key outside a range whose ends are literals, RUN-KEY otherwise (`loader.range_literal`, `evaluate.range_is_literal`,
  `check_range_lits`; `L2_5_stepped_range_*`). L §6.4: a scenario `replace`s any row known in its base, an overlap
  without `replace` is refused, a `replace` of nothing too (`L6_4_scenario_*`).
- **The triage's sites** (writer 5's numbering, `scratch/refcore-r7/triage-sites.md`), each a test: T-6 to T-9 records
  (LOAD-FIELD for legs or a missing field, never compared: LOAD-UNIT/RUN-DOMAIN); T-10 a row's unit at `*`; T-13 an
  anchored `02-29` (LOAD-PARSE); T-14 a calendar argument (LOAD-UNIT); T-29 a known window ending after `on`
  (LOAD-KNOWN, rows too); T-44 a lifespan with no start; T-45, T-46 the base kind (LOAD-DUP); T-51 a band over things
  (LOAD-UNIT); T-53 `unit()` of a quantity unit; T-59 a binding over a movement whose day is a hole
  (`L5_11_binding_hole_day_member`); T-61 a fill side whose members' days are holes (possible members, every arc a
  hole condition; `L5_12_fill_side_hole_day`); T-65 a decision's binding over walk-booked movements, booked up to its
  point, RUN-FUTURE past its day (`L5_14_1_bind_decisions_*`); T-66 HOLE-PRERUN; T-69 a movement's `for`; T-70 to
  T-72 the unit of `flows` (`check3.m_t_flows` LOAD-UNIT, `mreads` RUN-DOMAIN; `L5_14_3_flows_unit_*`); T-73 a
  random fact's sample type; T-75 `path p` (LOAD-PARSE); T-79 the standard error over one path; T-86 `check`'s
  difference; T-87 `explain low`'s first run; T-93 `observed()` lacking a leg (RUN-RECORD); T-96 a `diff` across two
  scenario stacks (`cli.cross_stack_diff`, `L6_4_diff_two_stacks`); T-98 `pow` with a unit; T-100 to T-104 the
  residual ledger (a LOAD-UNIT guard; one ledger per unit, printed `ledger U:`; the `?` group; an adjustment's matched
  line against δ on V.known, grouped by the adjusted move; a reading `after m` on m's day; `E6_3_residuals_*`).
  G-KNOWN (T-32, T-33, T-39): `model.WCut` carries the cut with the line's `windows earliest`/`latest`; something
  known in a window that holds the cut (a row, a rule, an entry, a `moves` line, an observation, a reading, a
  `covers`) is chosen by the clause, else HOLE-IMPRECISE (`model.WindowUndecided`, turned into the hole by
  `Engine.ev`; an entry is an unknown member of a binding over its kind; `L4_5_known_window_*`).
- **Round 14** (`scratch/refcore-r7/round14-sites.md`). A movement's `kind` is `text` (`L2_5_movement_kind_field`);
  `meta(…).governs` of a row that governs always is `(from: none, to: never)` (`L5_18_meta_governs_always`);
  LOAD-PRICE for a price named on a kind and its place (`L5_10_priced_twice`); a `windows` clause chooses a
  calendar's `added` window (`L4_2_added_window_windows`); `first`/`last` take no term (`L5_11_first_takes_no_term`);
  an input keyed by one period index is not law-dated (`InputInfo.undated_read`; `L4_3_period_keyed_input_def`);
  prefixed band calls `R.b(x)`, `R.b[k](x)`, `a.b(x)`, `household.b(x)` (`parser._name_chain`,
  `check.prefixed_call`; `L2_3_prefixed_band_*`); G-SORT's LOAD-SAME (`joint.same_joins`, `plan_joins`,
  `loader.joint_sames`: a version's `same` naming a kind or thing, sides of two sorts, kinds declared differently
  (notes aside they join), things of two kinds, rows of two types, places of two units or two price inputs, a joint
  row on a member's item no `same` joins; `L6_1_same_*`); a currency two members declare with different minor units
  (`joint.dedupe_units`, `L6_1_currency_minor_differs`); G-LINK `same p = q: p doesn't exist` (RUN-RECORD,
  `L6_1_same_side_missing`); G-GROUP: `same` is an equivalence (`walk.same_pairs` groups, `mreads.once_by_same`
  union-find), a group books once at each place two members touch, the latest member's leg in the day's order, and
  members that disagree there make it HOLE-SAME `<a> and <b> are one, but the leg at p differs on d: …`; a plan stated
  `same` as an observation isn't `unfulfilled` (`L6_1_same_group`, three lines of one statement joined by two `same`
  lines); a walk-bound def's cells are explained as def
  cells, at a key just before the reader's point, a running `bal(x)` read printed as a node (`ask.explain_walk_bound`,
  `E10_explain_walk_bound`).
- **Faults**, each a test: a scenario's load refusal reset the currency table under the base's asks (an internal
  KeyError; `L8_3_scenario_load_refusal_base`); a four-digit figure where no period is expected was refused LOAD-UNIT
  as a year, not LOAD-LITERAL (`L7_1_year4_number_literal`); `low … from t` dropped `with uncertain`
  (`L5_14_1_low_with_uncertain`); `next` of a decision counted an instance whose `when` was false at its point (it read
  the plan, not the walk's runs; `L5_14_1_next_decision_when`);
  a law, known or wording pin written as an expression now prints `pinned: law <expr> = d` and merges its register
  (`L8_2_pinned_expression`); writer 6's `w_on` shape refuses nothing: before any adjustment exists the binding binds
  nothing, between the raise's known day and `w_on`'s version it is a hole, L §5.7 (`L5_7_bind_hole_day_w_on`).
- **Writer 6's open items.** An ask's `for` lists `unfulfilled:` per line (`members(regs=)`;
  `L8_2_for_unfulfilled_per_line`); known windows over observations, readings and coverage give holes, not NOT-YET
  (`L4_5_known_window_statement`); writer 6's "the left side counts" is replaced by G-GROUP; U-44 keeps its marker
  (`moves.settle_sign`, `L5_14_3_settle_place_change`); U-43 has no site (a member is one file, so two members share no
  law part; a directory member is NOT-YET).
- **Tests changed**: `L8_2_gap_pad_register` (`spent` is now `outside DUK`: round 12 refuses `flows` of a unit-any
  place); `L2_5_stepped_range_fulfils_out` (RUN-RECORD, G-LINK) and `_literal_out` (RUN-KEY: its ends are rows);
  `L2_3_rule4_ask_for_world` (`unfulfilled: gift` on its line); `L6_1_pin_unspecified` (NOT-YET);
  `L6_1_joint_row_unjoined`, `L6_1_same_in_model_thing`, `L6_1_same_things_two_kinds` (LOAD-SAME);
  `L5_10_priced_twice` (LOAD-PRICE); `E10_explain_walk_bound` (def cells).
- **Speed** (E §3.2 memos) was dropped by the coordinator; no text in the suite is slow (657 tests in about 8 s), and
  writer 7 met no text too slow to finish.

## Refused REFCORE-NOT-YET (not built)

Writer 7's list, kept for history; the current one is under Writer 8: still NOT-YET (writer 8 built scenarios of a
joint model, SF-32, a member's keyed band, the two `to u` sites, arithmetic on a distribution value, a sample outside
a path, a priced reference with keys). Line numbers below are writer 7's. Each is a construct the text settles that
refcore doesn't build, or a guard of its own.

- **The pin** (L §6.1, E §3.1 step 1, round 14's G-PIN): the sha256 of a member's canonical text needs the canonical
  reprinter (L §2.4); `joint.pin_check` refuses unless the run says `--pin=bytes` (a test convenience, see Running).
- **Joint models**, settled by round 14 and not built: a member of several files (a directory, so also U-43's shared
  law parts) and a member that is itself a joint model (`joint.py:254`, `:272`); scenarios of a joint model (G-OWNROWS,
  U-35; `joint.py:281`); `same` of a keyed place or a thing at keys (U-29; `joint.py:516`, `loader.py:1579`); a thing
  entered more than once, and two things whose entries or lifespans differ (G-ENTRIES, U-30, U-31; `joint.py:521`,
  `:573`); a unit other than a currency that two members both declare (a member's own unit `a.hour`; `joint.py:714`);
  a decision in a `same` group (U-37, booked at the group's latest point; `walk.py:738`, `:751`); a movement of the
  joint model in a member's place (SF-32, U-38; `walk.py:802`); a member's keyed band called by the joint model
  (`check.py:2736`). The old member-read path (`check.py` `member_read`, a model loaded without the merge) is
  unreachable from `run`/`test`, which merge.
- **Asks across stacks** (L §6.1, §6.4): an ask line whose worlds sit on different stacks of scenarios, other than a
  `diff`, and a `diff` asked where one side's stack isn't loaded (`cli.py:143`, `:278`; refcore loads one reality
  per stack); `as if {scenario s}` reached from a reality that doesn't load s (`evaluate.py:2386`).
- **Known windows** (G-KNOWN): something known in a window that holds the cut, read where no hole is kept (outside a
  read, a binding over a kind, an observation, a reading or a `covers`), stays `model.WindowUndecided`, a NOT-YET.
- **Numbers and randomness**: `to u` on a closed-form quantile of a·X + b and on statistics resting on rounded values
  (`eval2.py:813`, `:877`); the error bound of a `crps` over more than 4 000 values resting on rounded values
  (`eval2.py:1176`); arithmetic on a random value outside the forms listed (`check.py:1316`); a sample outside a path
  (`eval2.py:575`); `discrete` over the things of a kind, a key list or a range (`check2.py:360`, `:366`); a keyed
  fill over a kind or a binding other than an index, list or range, and a fill side over one (`check2.py:1233`,
  `:1240`); a fill read inside its own supply (`check2.py:1327`); a fill side over an index whose span is a hole
  (`eval2.py:1655`).
- **Movements**: a built-in movement field outside legs, `day`, `kind`, `fulfils` and attributes (`mreads.py:196`,
  unreachable past LOAD-FIELD); the value of an observed row of no declared type (`loader.py:2381`); a priced
  reference with keys or fields (`check3.py:818`); `explain low` whose lowest balance falls on no day of the span
  (`ask.py:1248`).
- **Guards of this interpreter** (each behind a load refusal the text states, or a size limit the text doesn't set):
  a read with no law date that L §4.3 refuses at load (`evaluate.LAWGUARD`, six places); an axis read outside the
  axes an ask reaches (L §5.5: the cone holds every axis; `evaluate.py:352`); a stepped range of more than a million
  keys (L §5.6 sets no cap; `evaluate.py:2657`, `:2680`); a lifespan with no start (LOAD-LIFESPAN at load;
  `evaluate.py:3454`); `windows each` over more than 20 000 day combinations (`ask.py:580`); nesting deeper than the
  stack (`cli.py:114`).
- Long-derivation folding in `explain` (E §10); `LOAD-SCHEDULE` for a schedule outside a move's `on` (what "the run"
  of a schedule is there; see the gaps).
- WARN-BACKPAY, WARN-DRAFT, WARN-ENDED, REQ-ASK and REQ-CLOCK belong to operations and requests, outside a
  reality's text; WARN-STALE is raised by the store (L §6.4).

## REFCORE-UNSPECIFIED places

**Writer 10: settled.** Round 19 (GD-24, GD-22) states an outcome at each of the 13 sites listed by writers 8 and 9
(§2.4 (8, the reprint's orders), §4.6, §5.9, §5.16 `discrete` over a key list, §6.1, E §3.2), and each of their
tests now expects it. `UNSPECIFIED.md` now holds 1 site, the Maritz-Jarrett SE of samples that are no numbers
(§5.16); the ties L §2.4 left are settled by GD-28; see Writer 10. What follows is the history.

Writer 8, second pass: 10 sites, 9 new: §2.4 (8, the reprint's open orders) and §6.1 (1); listed under Writer 8,
second pass. Writer 8's first pass: 1 site, listed with a minimal text in `UNSPECIFIED.md` (generated by
`scratch/refcore-r8/gen_unspecified.py`). By section: §5.16 (1), `check2.py:370`, `discrete` over a key list or a stepped range (the table states
`discrete(w for k in IDX)` over an index; `L5_16_discrete_key_list`). Settled by writer 8: §5.14.3 (2: same-day
versions make one adjustment named under the first in ASCII order; a place-change settlement signs by the leg's
places before and after), §5.16 (1: `mean(d)` is the table's mean), §5.19 (1: a `fulfils` of an adjustment no version
made is RUN-RECORD), E §6.3 (2: unheld adjustments and instances are unexplained; an undated matched line falls in
its observation's month). Writer 7's list follows for history.

6 sites (writer 7), listed with a minimal text each in `UNSPECIFIED.md` (generated; see Running). By section: §5.14.3 (2),
§5.16 (1), §5.19 (1), E §6.3 (2).

- §5.14.3 `moves.py:1363`: an adjustment made by two versions known the same day (one step, E §5 step 4), which an
  observation names by one of them: `adjustments[m[keys].leg, V]` names one version.
- §5.14.3 `moves.py:1386`: a settlement of an adjustment for a change of its leg's places (round 14's U-44, left open
  by the coordinator: which places sign it, and its remainder; `L5_14_3_settle_place_change`).
- §5.16 `check2.py:614`: `mean` of a distribution value written inline (`mean(normal(mu, sd))`; `mean(g)` of a random
  fact is the closed form, T-74 narrowed).
- §5.19 `moves.py:1184`: a `fulfils` naming `adjustments[…, V]` that no version made at the cut (L §5.19's RUN-RECORD
  names "an instance that doesn't exist").
- E §6.3 `resid.py:263`: an observation settling an adjustment the model at the cut doesn't hold, in the ledger.
- E §6.3 `resid.py:286`: a matched line of an instance whose day is a hole or a window across months: its month
  (the `?` group is stated only for an instance "that nothing fulfils").

Writer 7: of writer 6's 72 sites only T-74 is left, narrowed. Settled by the text into refusal codes, holes or
values, each with a test (What works): §2.5 (4), §3.1 (4), §3.2 (1), §3.3 (1), §3.7 (1), §4.1 (1), §4.2 (1), §4.5
(3), §5.3 (1), §5.7 (2), §5.9 (1), §5.10 (2), §5.11 (1), §5.12 (1), §5.14.1 (2), §5.14.3 (6), §5.16 (3), §5.18 (2),
§5.19 (3), §6.1 (12, LOAD-SAME or RUN-RECORD), §6.4 (1, a `diff` across two stacks), §7.1 (1), E §6.3 (5), E §10 (1).
Turned into NOT-YET, the text settling them and refcore not building them or guarding behind a load refusal: §4.3
(6, the LAWGUARD guards), §5.5 (1, the axis guard), §5.6 (2, refcore's million-key limit), §5.7 (1, a lifespan with
no start), E §3.1 (1, the pin); an ask across stacks other than a `diff` (§6.4) is NOT-YET too. New: the five
above other than §5.16.

Writer 6: 50 of writer 5's 104 sites settled by the current text (listed under What works; each now a refusal code, a hole or a value
the spec states, with a test), the joint model's NOT-YET replaced by 14 new sites (§6.1 twelve: `same` sides of two
sorts, kinds declared differently, places in different units, things of two kinds, a joint row on a member's item no
`same` joins, a `same` instance with no movement; E §3.1 the pin's bytes; §2.5 a stepped-range key in a `same` side),
and new: §5.10 a place and its kind both stating a price, §2.5 the type of a movement's `kind` field, §3.7 `meta`'s
`governs` of a row that governs always, E §10 the cells of a walk-bound def. The §4.3 no-law-date sites in
`evaluate.py` are now guards (an ask line reading a law-dated input or band without `law` is LOAD-LAWDATE at load).

Writer 5: settled by rounds 8/8b: §4.6 an item keyed by more than one time index (removed); §5.18 `explain low` without
`from` (now built; a narrower site remains: a lowest balance reached on two separate runs of days); §6.4 a scenario
move before `from` (left out; a narrower site remains: a window day or a leg's own day before `from`); the §4.6
HOLE-UNGOVERNED-without-a-law-date site no longer fires for change 3 (a binding that binds nothing needs no question;
it stays for a direct read). New: §5.14.3 the leg of an instance made to exist by versions that give it different
places.

Settled by the round-7 text (writer 3's sites removed): §4.3 a def without `at` reading an input whose rows all govern
always (L §4.3 rule 2); §4.3 a move whose `at` is a window (L §5.14 `at e`); §5.9 a band mixing replaced and plain rows
(L §5.9, `LOAD-BAND`); §5.10 `unit(x)` of a place of unit any (refused, L §5.10); §5.14.3 `flows(m, span)` of a
multi-leg kind (L §5.14.3, `LOAD-LEG`); §5.14.3 `flows` over a modelled-past instance (its observations stand in).
New sites this round: §3.1 legs in a record value; §4.6 HOLE-UNGOVERNED read with no law date (its question names
`<law date>`); §5.9 a band over things (their order depends on the cut); §5.10 `unit(x)` of a unit no unit kind
holds; §5.13 prorata over several bindings; §5.18 `explain low` without `from`; §5.18 the hole `where` on a line that
reads an axis (a second site of an old question); §6.4 a prefixed name of a place, band or move in an expression row;
§6.4 a scenario-declared move before `from` (E §9 invariant 3).

## Contradictions and gaps between places of the spec (what refcore does)

Writer 3's thirteen are settled by the round-7 text (1 `pit … at start_of(y)`; 2 change 3 names `act_day` and the
§5.14 example uses `pay_lead`; 3 a move whose day is none has no instance; 4 `salary_eur … governs 2026-10..`; 5 E §9
invariant 1 leaves out legs booked by their ends; 6 WARN-STALE belongs to the store; 7 `explain bal`/`low` in the
grammar; 8 L §4.6 step 1 tests rules at d; 9 prefixed and bare names in expression rows; 10 RUN-CYCLE/RUN-FUTURE; 11
LOAD-COVERED; 12 `governs always` inputs; 13 LOAD-LEAF), and so are the gaps on `flows(m)` legs, modelled instances,
`next()`'s order, adjustment attribute types and E §3.1 step 7. Writer 2's W2-1..W2-7 are unchanged. Found by writer 4 (1, 2, 3, 4, 5 and 7 settled by round 8, see below):

1. **L §6.5 change 3 vs L §4.6 step 5 and §4.3 rule 2.** Change 3: "`ask … as known 2028-06-14` still shows the old
   view". Its version (known 2028-06-15) adds `act_day`, which the moves `keep_advances` and `refund_2027` bind `on`;
   at cut 2028-06-14 `act_day` has no row, and as an input whose rows all govern always it is read with no law date,
   so L §4.6 gives HOLE-UNGOVERNED, whose question "no value at `<law date>`" names a law date there isn't. refcore:
   `old` refuses `REFCORE-UNSPECIFIED §4.6` (test `L6_5_change3`); the old view would need the domain of a binding
   that reads an input with no row to be empty, which no rule states.
2. **L §2.3 `row … "=" value` vs L §6.4 expression rows.** The grammar's `value` holds literals, records, codes,
   distributions, calendars and holes, no expression; L §6.4 writes scenario rows whose value is an expression over
   the base (`household.valuation[flat] * keep_share`). refcore parses an expression value in a scenario version only
   (`LOAD-PARSE` elsewhere).
3. **L §6.4 scenario-declared moves vs E §9 invariant 3.** L §6.4 lets a scenario declare moves, and nothing confines
   their days to `from` on; E §9 invariant 3 says a scenario and its base agree before `from`. refcore: a scenario move
   that books before `from` refuses `REFCORE-UNSPECIFIED §6.4` (test `L6_4_scenario_move_before_from`).
4. **L §5.18 `explain low(…)` vs L §5.14.1 and E §10.** L §5.18 lists `explain low(…)` with the projections, but
   `low(x, span)` without `from` is plain balances (L §5.14.1 table) and E §10's projection row needs a day t. refcore:
   `explain low(x, span) from t` is a projection; without `from` `REFCORE-UNSPECIFIED §5.18`.
5. **L §4.2 schedules vs L §5.14 `on`.** L §4.2: "a move whose day is none has no instance"; L §5.14's `on d` lists a
   date, a window or a hole, and RUN-NONE elsewhere refuses reading a none. refcore follows L §4.2 for a move's day.
6. **LOAD-SCHEDULE's "run".** The refusal of a schedule that yields no date in the run is stated for a move's `on`;
   what "the run" of a schedule read by a def (with a law date, perhaps outside [S, E]) is, is not. refcore checks a
   move's `on` only.
7. **L §6.4's example `replace sale_price[flat] = household.valuation[flat] * keep_share`.** `sale_price` is declared
   nowhere in the text. If it is money (as a price of a flat would be), the value is a `num PLN` (money times a num),
   and L §3.2 "Back to money" ("A `num CUR` becomes money only through `round(x, u, mode)`") refuses it `LOAD-UNIT`.
   refcore checks an expression row as a value assigned to the input's type, so the example needs a `round`.

Round 8 settles writer 4's 1 (L §5.7: the bindings bind nothing, so change 3's old view prints 0.00 PLN), 2 (L §2.3
row: `expr` in a scenario's version, LOAD-SCENARIO otherwise), 3 (L §6.4: a scenario's move books nothing before D),
4 (L §5.18: `explain low` without `from` explains a balance), 5 (L §5.14 `on d` lists none) and 7 (L §6.4 declares
`sale_price` a `num PLN` input). Found by writer 5:

8. **L §5.7 Bindings (round 8) vs L §6.5 change 3's old view.** The rule binds nothing "where no thing of the kind,
   known at the cut, could be alive on one of its possible days". At cut 2028-06-14 the household's base already has
   adjustments (`wage[2026-10, …].cash@home_2026`), alive on a possible day of the unconstrained hole `act_day`; only
   the bindings' `where a.adjusts in pit_…` rules them out. refcore reads the `where` in (a member it rules out whatever
   the day can't be bound, L §8.1 "It stops propagating only where the outcome is decided without it"), which gives the
   old view; the round-8 sentence alone names the kind, not the `where`.
9. **L §8.3 LOAD-SCENARIO messages vs L §2.3 and L §6.4.** The catalogue lists three messages; L §2.3 (an expression
   row outside a scenario's version) and L §6.4 (a scenario's move dated before `from`, where load sees the day) refuse
   LOAD-SCENARIO with none of them. refcore words its own.
10. **L §8.3 LOAD-BAND's x.** "The first point two rows share, or the first point between rows that no row covers":
    among the reals an open end has no first point (`[0, 4]`, `[5, 18)` leave (4, 5) uncovered), rows merely written
    out of order (`[5, 18)`, `[0, 5)`) have neither, and a table with an overlap and a gap has two. refcore prints the
    open end, the first row out of order's lower end, and the smaller point.
11. **L §5.18 `explain low(x, span)` without `from`: "the day of its lowest balance".** A balance is usually lowest on
    a run of days (it holds until the next movement), and may be lowest on two runs. refcore explains a single run at
    its first day (the derivation is the same on each) and refuses `REFCORE-UNSPECIFIED §5.18` for two runs.
12. **L §6.4 "its instances dated earlier are left out".** For an instance dated by a window that starts before D, or
    with a leg's own `on` before D, "dated earlier" is unclear; refcore leaves out an instance whose day (its window's
    last day) is before D, and the invariant-3 guard refuses `REFCORE-UNSPECIFIED §6.4` if one books before D anyway.
13. **E §5 step 1 vs a read of a made-to-exist instance's leg.** An instance whose `when` was false at κ₀ "counts as
    A0 = 0"; L §6.5 change 3 reads `a.adjusts.cash.from`. Of such an instance, refcore reads the adjusted leg (its
    places as the version that made it exist gives them; amount A0 = 0); the text states the places of the adjusted
    leg (E §5 step 5) but not a leg read of an instance that didn't exist at its cut.

Found by writer 6:

14. **L §6.1 Kinds across members vs L §6.1 `same` and L §2.3 `sref`.** "Two members' kinds are one only by a `same`
    line", but a `same` side "names a thing, a row or a move instance" (`sref`'s note says the same). refcore joins two
    kinds by a header `same` (`L6_1_same_things_kinds`); a `same` naming any other sort is UNSPECIFIED §6.1.
15. **L §5.1 vs L §6.1 on things of two kinds.** L §5.1: `same` lines state "two things of one kind"; L §6.1 refuses
    only places of two kinds (`LOAD-SAME`). refcore: `LOAD-SAME` for places, UNSPECIFIED §6.1 for other things.
16. **E §3.1 step 1 vs L §6.1 on the pin.** "The reality's hash is over the canonical text" (E) vs a `use` line
    "pinned by its content" (L). refcore: UNSPECIFIED E §3.1 unless the run says `--pin=bytes`.
17. **L §6.4 vs L §2.3 `call`.** L §6.4 lets a scenario's expression row read the base's items by prefix, bands
    included; `call = (ID ["[" expr "]"] | BUILTIN) "("` has no prefixed band call (`household.fee(x)`). refcore parses
    none.
18. **E §10 vs L §5.18 on a walk-bound def.** L §5.18: "the cell at each walk point that read it, in walk order, each
    with its point"; E §10's row doesn't say whether each cell is explained as a def cell (rules, causes, rows,
    register) or by its point and value. refcore: UNSPECIFIED E §10 (`E10_explain_walk_bound` w1); a def read at no
    walk point prints as stated.
19. **L §4.4 `windows` vs L §8.1 HOLE-CALENDAR.** (Settled, writer 8's fifth pass, W-B-Q1: L §8.1's HOLE-CALENDAR row
    now names only an added day that is a hole, and HOLE-IMPRECISE "a window in a calendar's added list", as L §4.2
    and §4.4 do; refcore follows them, `L4_2_added_window`, `L4_2_schedule_added_window`.) Was: L §4.4: a line's `windows` clause "applies to every window the line
    meets", HOLE-IMPRECISE without one; L §8.1: an `added` day that is a window "decides the result" as HOLE-CALENDAR,
    `calendar <c> adds a day within <window>: is it <d>?`, fixed by stating the day. refcore follows L §8.1; a
    `windows` clause doesn't choose the added day.
20. **L §4.3 vs L §4.6 step 1 on an ask line.** L §4.3: "an ask line that reads a law-dated input ... directly" states
    `law` (LOAD-LAWDATE), law-dated meaning a row whose span isn't `always`; L §4.6 step 1 reads an input keyed by one
    period index at its time key, so its value needs no law date. refcore: LOAD-LAWDATE for a direct read of a
    law-dated input keyed by no period index or by two, and of a law-dated band (writer 9: a band whose tables all
    govern always, and whose rows read nothing law-dated, needs none, MC-X-F12); an input keyed by one period index needs none
    (`L4_3_ask_reads_lawdated`; the suite's `salary[ben_cogs, 2028-03]` asks).
21. **L §2.5 "LOAD-KEY for a literal" at a stepped range's position.** The range's ends may be rows, known only when
    read; refcore checks at run and refuses a written key outside the range `LOAD-KEY` (the whole model), a computed
    one `RUN-KEY` (`L2_5_stepped_range_key`, `..._literal_out`).
22. **L §5.11 `first`/`last` "take no term (LOAD-PARSE)" vs L §2.3 `agg`.** The grammar parses `first(t for k in K)`;
    the refusal is the prose's. refcore refuses LOAD-PARSE after parsing (was LOAD-DOMAIN).
23. **L §2.5 a movement's field `kind`.** Named a built-in field (§2.2, §2.5), its type and value are given nowhere:
    UNSPECIFIED §2.5. `version`, `source`, `known`, `governs`, `held`, `assumed`, `at`, `from`, `to` are meta's or a
    lifespan's, so on a movement they are LOAD-FIELD.

Round 14 settles writer 6's 14 and 15 (G-SORT, LOAD-SAME), 16 (G-PIN: the canonical text's hash; NOT-YET in
refcore), 17 (prefixed band calls in the grammar), 18 (def cells), 19 (a `windows` clause chooses the added day), 20
(an input keyed by one period index is not law-dated), 21 (LOAD-KEY only where the range's ends are literals), 22
(`first`/`last` parse no term) and 23 (`kind` is `text`). Found by writer 7:

24. **L §5.14.3 vs E §5 step 5 on a change of places (round 14's U-44, left open).** L §5.14.3: a version that
    changes "the places of one of its legs (then even with δ = 0: the booked movement stays where it was booked, and
    the change is never silent)" makes an adjustment. E §5 step 5: "A settlement leg counts +amount when it runs the
    adjusted leg's way, out of its `from` or into its `to` … the **remainder** is δ less that sum". The adjusted leg
    has two sets of places, and a settlement that moves the amount to the new place leaves a remainder ∓A with δ = 0.
    refcore: `REFCORE-UNSPECIFIED §5.14.3` (`moves.settle_sign`, `L5_14_3_settle_place_change`); the adjustment itself
    (δ = 0, HOLE-CHANGED until waived) is built (`L5_14_3_adjustment_place_change`).
25. **L §5.14.3 one step vs one version in the name.** "taken in known order (versions with one `known` together, as
    one step, `ENGINE.md` §5)" against "Its name is `adjustments[m[keys].leg, V]`, the adjusted leg and the version".
    Two versions known the same day that both change a leg make one step, and so one adjustment, with two names.
    refcore: an observation naming one of them is `REFCORE-UNSPECIFIED §5.14.3` (`moves.py:1363`).
26. **L §5.19 RUN-RECORD vs a `fulfils` naming an adjustment.** L §5.19: "A `fulfils` naming something that is neither
    a planned instance nor an adjustment … is refused (`LOAD-KIND`). A `fulfils` naming an instance that doesn't exist
    … is refused (`RUN-RECORD`)"; L §8.3's RUN-RECORD row says the same of an instance. An adjustment name whose
    version makes no adjustment of that leg at the cut is named by neither. refcore: `REFCORE-UNSPECIFIED §5.19`
    (`moves.py:1184`).
27. **E §6.3 Grouping, a matched instance whose day is a hole.** "A line's month is its own day's: the instance's day
    for a matched, expected-but-missing or partly fulfilled line"; "An instance that nothing fulfils whose day is a
    hole, or a window not inside one month, prints an expected-but-missing line in a group of its own named `?`". An
    instance that observations fulfil whose day is a hole has no month and no group. refcore:
    `REFCORE-UNSPECIFIED E §6.3` (`resid.py:286`).
28. **E §6.3 an adjustment's line and its "move id".** The matched row: "an adjustment that observations settle,
    against its change δ on V.known"; Grouping: "a line with no observation, or whose observations disagree, goes by its
    move id, in a group printed `move <id>`". An adjustment is a movement of the prelude kind `adjustments`, made by no
    move. refcore groups it by the adjusted move (`move wage`; `E6_3_residuals_adjustment`). An observation that
    settles an adjustment the model at the ledger's cut doesn't hold has no line the text names:
    `REFCORE-UNSPECIFIED E §6.3` (`resid.py:263`).
29. **E §4.2 a reading `after m` vs L §5.19's modelled past.** E §4.2: "A reading `after m` pads or anchors its place
    right after m is booked"; L §5.19: on a covered day the planned movements are the modelled past, "never booked"
    (their observations stand in). refcore applies such a reading right after m's observation legs too (`walk.after_mv`,
    once), and E §6.3's balance line stands on m's day (`E6_3_residuals_after_reading`).
30. **L §5.16 the closed list vs `mean` of a distribution value.** "Each has `cdf`, `pdf` or `pmf`, `quantile` … and a
    mean. Called on a distribution value, `cdf(d, x)`, `pdf(d, x)`, `pmf(d, k)` and `quantile(d, q)` are these
    functions"; Closed forms: "`mean(e)`, e a sum of known multiples of random facts plus a known constant". `mean` of a
    distribution value written inline (`mean(normal(mu, sd))`) is neither. refcore: `REFCORE-UNSPECIFIED §5.16`
    (`check2.py:614`); `mean(g)` of a random fact is the closed form (`L5_16_mean_random_fact_def`).
31. **L §5.14.3 (round 12) vs writer 6's fault test `L8_2_gap_pad_register`.** Not two places of the spec: the test
    read `flows` of a place of unit `any`, which round 12 refuses ("a place of unit `any` … is refused (`LOAD-UNIT`
    where load sees it …)"); its place is now `outside DUK`, and the test's point (the gap's register) is kept.

Writer 8 (24-30 are settled by the current text; 25-28 and 30 by statements writer 8 built, see REFCORE-UNSPECIFIED
places). New:

32. **L §5.16 A random number of items vs L §8.3 LOAD-DOMAIN.** §5.16 writes the range `1 .. n[m]` with no step; a draw
    of a `dist` with no unit is a `num`, and §8.3's LOAD-DOMAIN refuses `range of quantities <a> .. <b> has no step`.
    refcore refuses the example as written; texts state `step 1` (`L5_16_random_items_ok`). *Settled by round 17:
    the example states `step 1`.*
33. **L §5.19 Coverage vs E §4.1 on a `fulfils` link's existence.** §5.19 judges the link "by the existence of the
    instance it names (its keys and `where`)"; E §4.1's instance exists only where its `when` holds too. refcore reads
    keys, `where` and `when` at the ask's cut, with its legs' places, never its amounts (`W_D_F2_link_reads_no_amount`).
    *Settled by round 17: "by its keys, `where` and `when`".*
34. **L §6.1 The joint model's own movements vs L §8.2 `left out:`.** §6.1: an unlinked movement into a member's place
    is "listed in the joint model's register (`left out:`), and a joint total over that place is a hole naming it";
    §8.2's row lists it "under a result of a joint model that reads a member's place", not booked there. Whether a
    plain read of that place is the member's balance or the hole isn't said. refcore: every joint read of the place
    from the movement's day is the hole `<m> crosses to <member> with no link`, with the `left out:` line
    (`L6_1_joint_move_in_member_place`). *Settled by round 17 (every read of the place from the day on is the hole);
    what remains is contradiction 44.*
35. **L §6.1 rows of different kinds stated `same`, over a parent kind.** "a binding over either kind that meets the
    movement is a `HOLE-SAME` hole"; a binding over a kind holding both meets it whichever kind it has. refcore counts it
    once there, as a member (`L6_1_same_rows_kinds`). *Settled by round 17: it "meets the one movement once".*
36. **E §3.1 step 10 vs refcore's random cone.** Load refuses a random ask line with no query, and a decision's
    projection reaching a random fact after its day, "where load sees that day". refcore's load marks the random cone
    over inputs and defs, not through balances, so it finds a line random through a balance at the read and stops the
    reality as a load refusal (`errors.RandomLine`, `L5_16_ask_random_balance`), and a decision's projection is
    RUN-FUTURE at the read (`L5_16_decision_projection_future`). *Narrowed in writer 8's third pass (MC-B-F5): the
    cone now runs through moves and the balances they book into defs, fills and decisions, so a def read bare is
    refused at load; a balance read directly in an ask line, and a projection, are still found at the read.*
    *Settled by round 19 (GD-4, L §8.3 items 9 and 10; writer 10): the cone runs through every place a random
    movement books, so such a line is refused at load (`L8_3_cone_*`); a decision's projection is LOAD-RANDOM at
    load where load sees it whole (`E3_1_seen_day_projection_random`), otherwise RUN-FUTURE (contradiction 50).*
37. **The review's list vs the spec** (the spec built): W-4 writes `index q = quarter …`, L §4.1 has the period kinds
    day, month and year and writes quarters `index qs = month 2026-01..2027-12 by 3`; PL-15 names HOLE-ANCHOR for lots
    that disagree with the opening reading, L §8.1 and the list's own head name HOLE-LOTS (refcore: HOLE-LOTS on the
    supply's lots, propagating to takes, a reading of where the hole lands that is refcore's); W-2/W-3/A-3 drop
    `exponential`, the rolls and the weekdays, which the head's corrections and the spec keep.
38. **L §8.2 `pinned:` merged spans.** The pins one def forms under many `as if` "print once, their days merged into
    maximal spans"; the printed form isn't given. refcore prints `pinned: known d = 2026-03-01 .. 2026-03-03,
    2026-03-10` and merges pins of one written expression (`L8_2_pinned_merged_spans`).
39. **L §2.4 numbers: "the shortest exact decimal" vs `96 000.00`.** "Numbers print as the shortest exact decimal, with
    a leading `0` before the point (`0.05`) and thousands grouped by a space (`96 000.00`)": the shortest exact decimal
    of that value is `96 000`. refcore prints the shortest, so `5.00 PLN` reprints `5 PLN` (`L2_4_reprint_instants`).
    *Settled by round 17: `96 000`.*
40. **L §2.4 "notes come first" vs L §2.2's productions.** §2.4: "Inside a section: notes come first", "Clause order is
    the order in the grammar", a regime's "`export` line last"; §2.2 puts notes after the clause lines: `index …
    [IN "v" ID NL] [IN "assumed" STR NL] {IN "note" STR NL}`, `fill … [IN "v" ID NL] {IN "note" STR NL}`, `move …
    [IN "fulfils" ref NL] {IN "note" STR NL}`, `version … IN "known" known NL … {IN "note" STR NL} {IN vitem NL}`,
    `regime … [IN "export" ID {"," ID} NL] {IN "note" STR NL}`. refcore: notes first in kind, index, input, band, fill
    and move bodies (its parser takes a note line anywhere in them); in a version after its clause lines (the
    grammar); in a regime first, `export` last (§2.4) (`L2_4_reprint_bodies`). *Settled by round 17: clause lines and
    notes in the grammar's order, a regime's `export` then its notes; refcore follows it (third pass), and what remains
    is contradiction 45.* *Round 19 (GD-1): regimes are gone (writer 10).*
41. **L §2.4 "so equal models print identically (N-1902)" vs what it normalises.** Two equal models that differ in a
    word the grammar takes in two forms (durations are "an integer and a period word", `1 day`, `5 years`, §4.1), in
    the order of repeated lines of one clause (legs, attributes, notes), or in spacing §2.4 names nowhere, print
    differently: §2.4 gives no rule for them. refcore keeps the file's (Writer 8, second pass, Readings).
42. **L §2.4 "thousands grouped by a space" vs "Dates print in ISO" for a year.** `2026` as a year and `2026` as an
    int are one token before typing, and the reprint precedes load (E §3.1 step 1: "Parse every file, and reprint
    it"). refcore keeps a four-digit integer with no unit as written.
43. **L §6.1 Shared law vs `same` lines.** "Two members that copy different versions of one law part … are refused
    (`LOAD-SAME`) unless the joint model states, by a `same` line, which one both follow"; but a side names "a kind, a
    thing …, a row … or a movement …; any other pair is refused (`LOAD-SAME`; a law part both members copy: Shared
    law, below)", and `sref = ID {"." FIELD | "[" lit {"," lit} "]"} (a kind, thing, row or movement …)`: no form
    names a law part's version. refcore: REFCORE-UNSPECIFIED §6.1 (`L6_1_shared_law_same_line`).

Writer 8, third pass (32-35, 39 and 40 are settled by round 17). New:

44. **L §6.1 (round 17) "every result that reads the place lists it under `left out:`" vs a read before the
    movement's day.** §6.1: "every read of that place in the joint model from the movement's day on … is a hole naming
    it, and every result that reads the place lists it under `left out:` (§8.2)"; §8.2's row: "under a result of a
    joint model that reads a member's place, the joint model's own movements into that place …"; but §5.18's register
    is what a result "reached" ("the planned instances it counted unfulfilled or left out"), and a balance on a day
    before the movement doesn't reach it. Taken word for word, `bal(b.konto, 2026-04-30)` lists a movement of
    2026-05-05. refcore lists it only under results that meet it, from its day on (`L6_1_joint_move_in_member_place`'s
    `kb0`, `ka0`; `L6_1_joint_move_reads`), as snap8 did. *Settled by round 17b: "on or after the movement's day".*
45. **L §2.4 (round 17) "its clause lines and notes first, in the grammar's order (§2.2)" vs a kind's lines.** §2.2:
    `kind = "kind" ID [":" kspec] NL {IN kline NL}`, `kline = "leg" … | "attr" … | "note" STR`: one repeated choice,
    so the grammar's order says nothing between a kind's notes, legs and attributes (the order of the alternatives
    isn't an order of lines, as `vitem`'s isn't). refcore keeps the file's order, as for repeated lines of one clause
    (41) (`L2_4_reprint_bodies`); two kinds that differ only there print differently.
46. **L §4.4 "the interval of their values at its first and last day" vs a calendar's `added` window.** §4.4 gives a
    built-in on a window the interval of its values at the window's ends ("two endpoint evaluations per operation, with
    no enumeration of days"), and states it for a window argument. A business-day function over a calendar whose
    `added` holds a window (§4.2, "a window the line meets") isn't monotone in that day: `schedule(2026-06, pay_dom,
    following, len)` with `pay_dom` 22 and `added [window(2026-06-19, 2026-06-23)]` is 06-22 at both ends of the
    window and 06-23 where the added day is 06-22. refcore takes every day the window can take and gives the interval
    of the values (`evaluate.over_added`; `L4_2_schedule_added_window_interior`).
47. **L §4.1 `by n` vs "A key of a year, month or day index is a value of that period type".** Round 18 (C1-17)
    removed "Its law date is its first day". §4.1: "A key of a range `by n` stands for the n periods from it to the
    next key: `day 2026-01-05..2026-12-28 by 7` gives weeks", and §4.3: "A period is not a law date"; but §4.1's Keys as
    values: "A key of a year, month or day index is a value of that period type", so a key of a day range `by 7` is a
    day, a law date. refcore takes the key as the period it stands for and refuses `at w` (LOAD-LAWPERIOD); the law
    date of such a key is now unstated (`L4_1_by_n_no_law_date` uses a month range by 3, which is a month either way).
    *Settled by round 19 (GD-20): a key of a year, month or day range `by n`, n above 1, is a period, never a law
    date (LOAD-LAWPERIOD, L §4.2 Keys as values); refcore follows (`L4_1_day_by_7_no_law_date`, writer 10).*
48. **E §4.2 "It starts at S from the opening readings" vs step 0's "with t before K".** E §4.2 (round 18, C2-2):
    "0. Each `opening` reading at `t start` that the cut sees, with t before K, anchors its place"; L §5.19: "Like an
    observation, a reading applies only before the cut K". Against: E §4.2's "It starts at S from the opening
    readings" and L §5.19's "`opening[x]` gives its balance where the walk starts". A forecast from today (K = S =
    `on`) with today's opening at S start meets both: by step 0 the opening is not applied. refcore applies an opening
    at S start, or before S, whatever K, and one after S only before K (`walk.prepare`; `E4_2_opening_after_start`).
    *Settled by round 19 (GD-21): step 0 reads "with t before K or t = S"; refcore applies an opening at S start
    whatever K, and one before or after S only before K (writer 10).*
49. **E §3.2 "Every law date inside one interval gives the same value" vs L §4.6 step 5's question.** The gap rule's
    hole names its law date ("with no time key, `x at k: no value at <law date>`"), so a read with no candidate gives a
    different hole at each law date of one class. L §4.3 reads a window `at` "once per law-date class the window
    overlaps, each at its first day inside the window", and the class's extent ("the rows and rules the item reaches":
    those at every key?) then decides whether a window inside such a stretch is one class (one hole) or several
    (HOLE-IMPRECISE). refcore reads every day, and refuses that case REFCORE-UNSPECIFIED E §3.2
    (`L4_3_at_window_gap_holes`); `windows each` splits it day by day, as each day's answer differs.
    *Settled by round 19 (GD-22): a stretch where the rows and rules the item reaches don't change is one class even
    where no row covers it, read once at its first day inside the window, one hole naming that day; refcore
    follows (`evaluate.def_window`, `L4_3_at_window_gap_holes`) and the site E §3.2 is gone (writer 10).*
50. **E §3.1 step 10 "where load sees that day" vs L §5.16 "where load sees the projection reach the random fact
    after the decision's day"** (writer 10). A decision whose day load sees (L §8.3 item 9) and whose projection
    takes a random movement whose day comes from a row: E refuses at load, L at the read (load doesn't see the
    movement inside the projection's span). refcore takes L's, the narrower, and refuses at load only what it sees
    whole (`check3.m_check_proj_seen`); the rest is RUN-FUTURE (`L5_16_decision_projection_future_path`).
    *Settled: E §3.1 step 10 now takes L §5.16's condition ("where load sees the projection reach the random fact
    after the decision's day", otherwise RUN-FUTURE), as refcore builds it.*

Gaps (silent, refused or decided as noted): the diff and residual output layouts; the timing of a residual line with
several observations; E §5's "four instalments" fulfilling twelve adjustments. (HOLE-ANCHOR's `<d>` and an
adjustment's attributes in a declared move or an observed row are settled: L §8.1's question for a place with no
reading, and LOAD-KIND for a move or observed row of `adjustments`.)

## Decisions taken where the spec allowed one reading only after thought

- Writers 1-2: a rule beaten by a rule whose `when` is false is not beaten; `as if` value pins listed under `pinned`
  when read; an input whose rows all govern `always` is timeless (now L §4.3); a query in a def is closed-form; a
  random fact read at no law date adds nothing to the horizon.
- An instance whose `when` was false at κ₀ counts as A₀ = 0 and a later version that makes it exist adjusts by Aᵥ
  (E §5 steps 1, 3). A plan's adjustments are measured per move (`adjust_move`): a plan reading `flows` of another
  plan depends on that plan's instances, not on every move's adjustments.
- A binding's per-member registers count only for members its `where` keeps.
- Writer 5: a binding over a kind whose day is a hole binds nothing when every member known at the cut fails its
  `where` (gap 8); a leg read of an instance a later version made exist gives the adjusted leg at A0 = 0 (gap 13); a
  movement whose leg's place is refused refuses every balance and flows (it could book anywhere); the explanation
  prints a cause in full where it first prints.
- Bindings over event kinds leave out the modelled past (a covered day's plan instances): its observations stand in.
- An ask in a scenario file with `in household` is answered in that file's reality; an ask whose worlds sit on
  different stacks is UNSPECIFIED §6.4.
- A joint model's member is read as it is, at the reader's law date and cut, by its own engine; a key of a joint
  period or int index keys the member's period or int index (L §4.2 Keys as values).
- Writer 6: a joint model is one merged model (members renamed `a·x`), so a member's facts are read under the joint
  model's cut and law date like any other; a member's own asks are answered by the member loaded alone. A joint row
  on a member's item that no `same` joins is UNSPECIFIED (L §6.1: the joint model "never changes an imported one").
  Two `same` rows that agree are one row; a `same` pair of instances counts its left side. A balance reading of `none`
  is grouped with the readings of its place's unit, so `replace … = none` finds the reading it removes. `observed()`
  sets `from`/`to` to `none` separately. A `for … where` hole line prints `q (k = …): ?"…"`, and in `diff` that world's
  value is the hole (the line is `changed`).
- E §9 invariant 2 is checked per unit on each walked day (net change inside equals boundary legs + pads + anchor and
  opening gaps); invariant 4 compares the timeline signature before and after an ask; invariant 5 records each booked
  plan amount per instance and cut class and compares later bookings.
- Writer 7: a version known the same day as another, both changing one leg, is one step (E §5 step 4): one adjustment,
  printed `adjustments[m[k].leg, V1, V2]`; a settlement naming it by either is UNSPECIFIED (contradiction 25). A
  made-to-exist leg's places are the last version step that made it exist. An after-reading applies once, after m's
  plan legs or its observations' legs, whichever books (contradiction 29). A decision's binding over walk-booked
  movements counts what is booked before its point (`Ctx.wp`), pads and decisions included. A known window that holds
  the cut makes an observation's legs, a reading's value and a `covers` span holes (HOLE-IMPRECISE), each chosen by
  `windows earliest`/`latest`. A `same` group books at a shared place the latest member's leg (`same_book_key`, the
  walk key); the others there are skipped when they agree on amount and day, and make one HOLE-SAME when they don't.
  A walk-bound def's cell is re-evaluated at a key just before the reader's point (so it doesn't see the reader's own
  booking), with cell recording off (`_explaining_wb`).

## Notes for a later writer

- Contexts: `evaluate.Ctx(law, cut, pins, axes, windows, wording, partial, path, world, unc, rnd, walk, wp)`; the walk
  key is (cut, pins, axes, windows, wording, path, world, rnd).
- The world filter is global (`model._WSET`): `ask._answer` sets it; `as if {scenario s}` and a scenario's
  expression row (`_world` names) switch it around their body.
- Registers: reads record only on the top collector (`Engine.cur`); bindings keep per-member collectors.
- `ENGINE-INVARIANT` has no test: it fires only on an engine bug (`scratch/refcore-r4/inv_probe.py` simulates one).
- A `LOAD-*` refusal raised at run (`refuse('LOAD-…')`, e.g. a written stepped-range key, `paths()` without paths)
  refuses the whole model, as a load refusal does.
- An ask's `for` lists `unfulfilled:` per line (writer 7, `evaluate.members(regs=[])`).
- Speed (E §3.2 memos) was not worked on by writers 6 or 7 (the coordinator dropped it from writer 7's brief); the
  suite runs in about 8 s, and writer 7 met no slow text.
- Writer 7's walk keys: observations `(d, 1, 0, oid + ' ' + leg, 0)`, plans `(d, 2, 0, osort, 0)`; a pad's `point`
  is its key; `walk.after_keys` maps a reading to its key. The cut is a `model.WCut` (a date carrying the line's
  `windows earliest`/`latest`), made by `Ctx.__init__` and `but()`; `model.visible` reads it.
  `model.WindowUndecided` is a NOT-YET that `Engine.ev` turns into HOLE-IMPRECISE (`.hole()`).
- To write a test: `scratch/refcore-r7/mk.py` and `mkjoint.py`; every expected value names its section in the
  comment, and comes from the text or a hand-checkable consequence of a stated rule.
