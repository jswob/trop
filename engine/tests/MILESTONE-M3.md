# Milestone M3: what must turn green

The checks M3 (the operations) must turn green, from BUILD-ORDER § M3 ("What M3 builds", "Budgets", "Green") and DONE gates 8, 10 and 12, with the items the run's brief puts in scope (issue #226). Written by T for issue #226 (2026-11-18) on refcore **tally 2.2**, from T's plan approved by the coordinator (picks P2–P9, `tmp/m3/auto-picks.md`; the spec questions filed as #228). The builder reads this page; T changes it.

## The list

"Today" is the engine at master ee1e9b37 (tally 2.2 and the operations document), 2026-11-18. Every `m3_…` target has `test = false` and runs only as `check`'s report until M3's marker commit (as at M1 and M2), so T's commits stay green with no `Logged-Red:` line.

| # | What | Ids, vectors | Run by | Gate | Today |
|---|---|---|---|---|---|
| 1 | `check` in refcore (the spec step's) | none of T's: chunk C4 ties refcore's `check` to the prediction of every vector of rows 2 and 3, byte for byte (`m3/vectors.py --confirm`); a difference is a STOP-REPORT | `m3/vectors.py --confirm` | 10 | done (C4, 2026-11-19): refcore's `check` (92662381, pin f62929bd) printed the prediction on 374 of 376 vectors; the two #229 fixtures print the spec's lines. Every vector now stores refcore's `check` (`source` `check`), and `--check` runs it again |
| 2 | `check` over M0's 315 text sets | `m3/check.vectors`, the vectors named by each set's id (C1): 295 `ok` (with load's warnings where the set has them), 11 a base's refusal (row 1 of M0's and `I:M:MP-10`), 9 with refused lines: a scenario refused (`I:B:b10_v3`'s `hr2_v3`, RULE-09's `inpath`, SURF-04's `redefine`) or asks refused alone (`U:law/LAW-13/LAW-13`, `U:rand/RAND-04/RAND-04`, `U:scen/SCEN-02..04`, `I:E1:tx2_cis_explain`: a probe given without its scenario's file) | `m3_check`: one test per M0 corpus group (16 with the parts) | 10 | 0 green: the engine has no `check` (a usage error, exit 2) |
| 3 | `check`'s line kinds the 315 don't reach | `m3/check/fixtures.tsv`, 44 fixtures (`fixture/<kind>/<name>`, § Row 3; 9 of them OPERATIONS' examples, § Row 5) | `m3_check`: one test per kind (9) | 10 | 0 green (none held since C4) |
| 4 | `INTERFACE.md`'s `check` | §1's row and §9 | — | — | written (C1) |
| 5 | the run function: the library's one entry and the CLI's; the request, asks and clocks; pure; stores nothing (E §1); OPERATIONS §3 and §7's examples | C2: `m3/runs.vectors` group `run` (10, § Row 5) and group `operations` (7 runs on `mini`); the library against the CLI once the builder names the entry; `m3/purity.py --engine` at the gate | `m3_run` (3 tests); `m3/purity.py` | 10 | `operations` and the library law green (C3, § C3); `run` green since G3's P18 and C5's split of `run/joint` (P21, § C5); purity: 107 of 128 sampled vectors equal on the quick binary at C2 |
| 6 | the seven changes through OPERATIONS §6.1 (#224's fifth item) | C2: `m3/changes/stages.tsv`: 17 runs (`changes/run/<stage>`, the three law asks and T's def or diff asks, before and after each change) and 17 checks (`changes/check/<stage>`: each stage with its change `ok`, each without its advance `LOAD-KNOWN`), § Row 6 | `m3_changes` (2 tests), `m3_check` group `changes` | 10 | the 17 runs green; the 17 checks wait for `check` |
| 7 | P8-1, a conversation turn | C2: X's change-7 turn (pick P6): `changes/check/c7`, `changes/run/b7`, `changes/run/c7`; their digests in `perf-digests.tsv` (§ Row 7) | `engine/tools/perf row P8-1`, once the rules step writes its `turn:` recipe | 12 | the row waits for the recipe |
| 8 | gate 8 ≥ 90 % | C3: the 98 survivors of run 44411247874 sorted (§ C3): 24 `R-87: equivalent:` rows for G3, 1 `missed:`; C5: 11 kill texts `fixes/gate8/m3_*` aimed at 25, 4 more equivalent, 10 more missed (§ C5); C6: 10 texts and `m2_counts_uncertain` aimed at 38, 5 proven caught (§ C6) | `m1_fixes` `fixes-gate8` (gating), `m2_counts` | 8 | 90.6 % on eb17e728 (run 44463157859: 2 506 caught, 417 missed, 156 equivalent); C7 wrote no kills (over 90 %) |
| 9 | #192's standing items, #221's first-group items, `net_worth`'s binding (P7) | C3: `m3/fixes/net_worth.tally` (group `last`); #221's three `ENGINE-INVARIANT` arms probed, none reached (`fixes/gate8/m3_readings`); #192's items not written in C3 (§ C3); C6: #192's N-c and P73 items (`m3/fixes/b192/`, § C6) | `m3_fixes` (3 tests), `m1_fixes` | 1, 8 | `last` green since G3's `unit(x)`; `b192` green on arrival; #192's other standing items open (§ C6) |
| 10 | #114's remainder | C6: one refcore run at the longest R7 horizon under 8 GB against the engine (`m2/bank/r7/to2032.tally`) | `m2/bank/pins_long.py` | 2 | done (C6): byte for byte to 2032-09-17; 2036 and 2056 net worth pinned by P55's route; the two digests blocked by design, for the user to strike |

`check`'s report line counts the `m3_…` targets' tests: `milestone-report: M3: 2/25 green (gates once M3 is tagged)` after C1 (`m3_check`'s 24 tests and the one that every group has its test; that one and `usage` pass: the engine already exits 2 on `check`); `milestone-report: M3: 5/31 green` after C2 (`m3_check` 27, `m3_changes` 2, `m3_run` 2; green: `m3_check`'s two, `m3_changes`' two, `m3_run_operations`).

## Rows 2 and 3: `check`'s vectors (chunk C1)

**The bytes.** Refcore has no `check` until the M3 spec step, so `m3/vectors.py` **predicts** each vector's exit code and whole output from refcore's `run` (pick P8), every line printed by a run of refcore 2.2:
1. the load probe, `run <files> --ask zz_m0_probe` (`m0/load.refcore`'s for the 315 sets; run for a fixture): a load refusal there is `check`'s whole output, exit 1;
2. else the warnings probe, `run <files> zz_m3_probe.tally --ask zz_m3_probe`, whose one ask names a world no file declares, so it is refused alone and nothing is evaluated: it prints every refused scenario's line and load's warnings;
3. a run of the asks a scan finds naming a world no file declares (`in`, `against`, `with scenario`, `as if {scenario …}`): each `<q>: refused: LOAD-SCENARIO ask <q> names …` line is an ask refused alone;
4. OPERATIONS §2.3's assembly: `ok`, or the scenario lines in their files' full-path order (P4), then the asks refused alone by id, then full path, then line (P5); then the warnings; exit 0 only after `ok`. An ask given is a usage error, exit 2 (P3), its words the engine's.

Each prediction is tied to the stored outputs of `checks.tsv` that run its set: a whole-output one that isn't refused whole holds exactly its `WARN-UNREAD` lines, and every `scenario … refused` line a stored output holds is predicted; all hold (a household's stored blocks drop warnings, so their counts are only listed). Chunk C4 runs refcore's own `check` on every vector (`--confirm`), holds it to the prediction byte for byte and stores its bytes (`source` `check`). **Done 2026-11-19:** 374 equal. The two of `RULED_229` print the spec's `LOAD-SCENARIO` lines where `run` printed nothing, as ruled. No set that `check` answers `ok` has a stored run that prints a `LOAD-` refusal (the writer's question 4, `ask.LoadAtRun`: `vectors.py` lists any as `load-at-run (#229)`; none).

**Row 3's fixtures** (`m3/check/`, every text invented: a model `plain` that loads clean, and small files each fixture adds; `nest/` for a directory; `joint/` a joint model and three members):

| Kind (test) | Fixtures | What `check` prints |
|---|---|---|
| `ok` | `clean`; `unread` (an unread source, input, def and place); `outside` (an outside place nothing names); `scenario` (one that loads); `joint`; `joint-unread` (a member's unread input) | `ok`, then the warnings sorted by code then text; none for the member's (L §8.5: each member warns alone, the joint model prints only its own; ruled 2026-11-18) |
| `base` | `dup`; `path` (a path that names nothing); `no-path`; `scenario-read` (a base item reading a scenario's); `beside-scenario`; `member`; `pin` | the one refused line: `LOAD-DUP`, `LOAD-PATH`, `LOAD-HEADER`, `LOAD-SCENARIO`, `LOAD-DUP` alone, `refused: member a: LOAD-UNIT …`, `LOAD-PIN` |
| `scenario` | `one`; `path-order` (`zulu`'s file before `alpha`'s); `over-refused`; `over-nothing`; `cycle`; `deep` (five); `twice` | each refused scenario's line in its file's full-path order; exit 1 |
| `alone` | `plain` (`q_zz` before `q_aa` in the file); `forms` (`diff … against`, `with scenario`, `as if`); `with-scenario`; `scenario-file` and `same-id` (held); `covered` (an ask in a refused scenario's world) | each `<q>: refused: LOAD-SCENARIO …` by id; the covered ask nothing |
| `sections` | `all` | a scenario line, two asks refused alone, four warnings; exit 1 |
| `notclean` | `clock`, `hole`, `pervalue`, `paths` | `ok`: `REQ-CLOCK`, a hole, a per-value refusal, `REQ-PATHS` are `run`'s |
| `paths` | `nested` (a directory whose two refusals print by full path), `household` (#226's `check household/`), `pin-bytes` (`--pin=bytes`) | `LOAD-PARSE … (nest/a/y.tally:4)`; `ok`; `ok` |
| `usage` | `ask` (`check model.tally --ask q`) | exit 2 (P3) |
| `operations` | § Row 5's checks on `mini` | `ok` ×5; CHG-7's, TXT-3's and ASK-2's one refused line; §7's scenario line and warning, exit 1 |

## Row 5: the run function and OPERATIONS' examples (chunk C2)

Every byte is refcore's run (`m3/runs.vectors`, `source` `run`; a usage error `rule`, exit 2 alone).

**Group `run`** (`m3/run/runs.tsv`; texts `m3/run/`, over `m3/check/model.tally`): `twice` (an id asked twice prints twice), `order` (the request's order), `ask-empty` (`--ask` with no id: every ask), `cuts` (two asks, two `as known` cuts, each its own `cut`), `unnamed` (a file in the table no argument names, which would refuse `LOAD-DUP` if read), `dir` (a `.txt` under a directory given, never read), `joint` (`x` and the member's `a.q`), `covered` (OPERATIONS §2.1: the refused scenario's line once, the ask in its world its own refusal, the scenario file's own ask nothing, even named), `unknown` and `none` (no command: usage, exit 2).

**Group `operations`** (`m3/mini/`: §7's `reality.tally` and the files §3 prints, verbatim; `early/` is the header with `on 2026-10-19`, `w/` the §7 `acme_stops.tally` whose row says `replace`): runs `ask` (ASK-2), `asks` (ASK-3 to ASK-7), `bolt` (CHG-4's holes), `diff` (SCN-1), `worked` (§7), `statement` (IMP-1, IMP-3's anchor line), `law` (LAW-1's `check`); checks `question`, `bolt`, `scenario`, `statement`, `law` (`ok`), `known` (CHG-7), `name` (TXT-3), `dup` (ASK-2), `worked`. **Read against the document:** refcore prints SCN-1's and §7's printed outputs byte for byte, and CHG-7's and TXT-3's refusal lines word for word. No difference.

**Purity** (`m3/purity.py --engine [release|quick|PATH]`, at the gate): the first vector of each group of S's, M1's, M2's and M3's stores (128), each run by the binary from a fresh temporary tree under `TZ`, `LC_ALL`, `LANG`, an empty `HOME` and stray `TALLY_*` variables; each must print its stored bytes and leave its tree unchanged. On the quick binary at C2: 107 equal, the 21 `check` vectors wait for `check`.

## Row 6: the seven changes (chunk C2)

`m3/changes/stages.tsv` (cwd `integration/amendments/real11`): `bN` is the household with only its `on` moved (§6.1 step 1), `cN` the household with the change (`real11/cN`, or T's stage-a texts for 4a and 6a: `m3/changes/c4a/facts/2028.tally`, `c6a/facts/2027.tally`, the first lines of the stage-b files, and `on-2028-05-14/`, `on-2027-04-02/` headers), `kN` the change without its advance. `m3/vectors.py` first asserts that every `real11` file a stage names is the spec household's but the change's own lines, and that T's texts are the household's or the stage-b files' lines.

| Change | Runs (`changes/run/`) | Asks (T's def or diff asks: `m3/changes/asks/`) | Checks (`changes/check/`) | Held to M4 |
|---|---|---|---|---|
| 1 | `b1`, `c1` | the three law asks, `c1_revenue` (revenue and health 2027-01..04) | `c1` ok; `k1` `LOAD-KNOWN version client_c_2027` | the walk cells (`pit28`, `konto_balance`, `net_worth`, `ike_left`, `taxes_paid`, the 2027-03-15 first difference) |
| 2 | `b2`, `c2` | three, `c2_wages` (minimum, projected wage, health base 2026, 2027) | `c2` ok; `k2` `LOAD-KNOWN version proj_wage_2027` (the second version alone) | the walk cells |
| 3 | `b3`, `c3` | three, `c3_health` (health 2026-12..2027-06) | `c3` ok; `k3` `LOAD-KNOWN version health_ryczalt_2027` | the back pay's `adjustment:` lines and the waiver: booked instances, a walk |
| 4 | `b4a`, `c4a`, `b4b`, `c4b` | three (`pit37` gains Ola's lines) | `c4a`, `c4b` ok; `k4a` `ola_born`, `k4b` `ola_800` | `ola_800_paid` and the May back pay: a walk |
| 5 | `b5` | three | `c5` ok; `k5` `LOAD-KNOWN version lifted_figures_5_job_abroad` | the after side's runs: several currencies (BUILD-ORDER § M4 places `I:X:*@c5_fast`, `@v5_fast`) |
| 6 | `b6a`, `c6a`, `b6b`, `c6b` | three, `c6_loan` (the annuity, instalment, principal and overpayment at k 1, 8, 9, 10, 40, 41, 47, 48) | `c6a`, `c6b` ok; `k6a` `car_loan`, `k6b` `car_overpay_2028_01` | the loan's end in `konto`, `net_worth`: a walk |
| 7 | `b7` (the household as is), `c7` | three in `wife_stops`; `c7_wife_diff`, `c7_social_diff`, `c7_health_diff`, `c7_pit37_diff` (`diff … in wife_stops against tropek`) | `c7` ok (the household's own `ok` is `fixture/paths/household`) | the walk cells `in wife_stops` |

**Refcore's cost:** the 17 runs and 17 checks took 15 s on refcore_fast (PyPy, run.py's `pypy+fast`), and none came near 8 GB or 90 s. The walk asks were not run: they are load row L:10's (its recipe names these 17 realities and their walk asks), timed as P8-L10 at M4 (pick Q8). On the engine at C2 the 17 runs are already green.

## Row 7: P8-1 (chunk C2)

The turn (pick P6): `check` of change 7's reality, `run` without it, `run` with it: `changes/check/c7`, `changes/run/b7`, `changes/run/c7`. `tools/perf_digests.py` now reads a `turn:<id> <id> <id>` recipe and writes one digest per vector from M3's stores (refcore's bytes); `perf-digests.tsv` holds the three from the recipe the rules step is asked to write, `turn:changes/check/c7 changes/run/b7 changes/run/c7`:

| Vector | sha256 of refcore's output |
|---|---|
| `changes/check/c7` | `dc51b8c96c2d745df3bd5590d990230a482fd247123599548e0632fdbf97fc22` |
| `changes/run/b7` | `135a25c2f3b37b08a2965ac77cbe979fb80a728fc0b742d1e36779556476ef31` |
| `changes/run/c7` | `2074832b3cad4473df88b3ce90ab9e4462facbb47e9cd51b639526db4c05b0d7` |

`check`'s digest is of refcore's `check` since C4 (the bytes the prediction gave). The same run of `perf_digests.py` re-stored P8-10's `I:A:a05_ania@c05-ania` digest, stale since tally 2.2's refcore fixes (8341b008, bec41e6b): it changed its hole lines after the file was last written (2026-11-15).

## C3: gate 8, the backlog, the reviews, step b (chunk C3, 2026-11-19)

**Gate 8** (run 44411247874 on 4fa66bfd, before G1: 87.5 % = 2 416 ÷ (2 416 + 494 − 150)). T mapped the 98 unlisted survivors to `build/m3` at ac551ad8. Five moved with G1's splits, none is gone, and none is now caught by a vector. The sort is `tmp/m3/t-equivalent.md`, which the coordinator copies to G3:
- **24 `R-87: equivalent:` rows**, each with its reason. They are 18 of `walk/fork.rs` (a read that doesn't continue a kept walk walks anew as the twin does, so only the counter "walks shared" differs), `covers_all` ×3 (any covering span's end gives the same union), `seen_observations` and `observed_row_seen` `> → >=` (same-day restatements in one layer are refused at load, `LOAD-OVERLAP`: T's probe on both engines) and `Uncertain::leg` `> → >=` (the sign is ±1). With them the score is 88.3 %.
- **1 `missed:`**: `reads_lowest` → `true`. The traced read of a projection's `min` is `not implemented` on the engine.
- **Killable with no text yet**: `ledger.rs` `hole_days_into` ×2, `agenda.rs` field deletions ×3, `row_place` / `version_file` ×4 (the read-time tie that load pre-empts: equivalent if `choose` never meets two rows load let through, the builder's word), `answer_apart` ×5. 69 not sorted in C3.
- **No kill text written in C3.** Each candidate T tried was pre-empted at load (the same-day restatement) or not built (explain of a projection `min`). 90 % needs k + 0.9·e ≥ 68 more.

**#221's arms** (G1's typed table): `load/rows.rs` `type_readings` (a reading at no edge is `LOAD-INSTANT`; unit `any` no longer exists), `eval/read.rs` `row_value` and `ask/walked.rs` `reading_instant` (readings read directly and explained, in the base and in a scenario with its own `stated` row). None reached `ENGINE-INVARIANT`, and both engines print the same. The probe is a gating vector, `fixes/gate8/m3_readings`.

**`net_worth`'s minimal text** (P7, `m3/fixes/net_worth.tally`, group `last`, `m3_fixes`): six places of four kinds and two units, an outside place, a wrapper's fund units priced in PLN. `last/net_worth` asks household X's binding; refcore prints 5 400.00, 10 400.00 and 30 400.00 PLN, and `kinds_only` 4. `last/unit-quantity`: `unit(x)` of a quantity unit's place is `RUN-DOMAIN` (net_worth's `and` keeps it unread). `last/places` asks the binding's other parts, which the engine answers. The engine stops at `unit(x)` of a bound place: `tally run: not implemented` (G3).

**The reviews' vectors** (`m3_check` group `reviews`, `m3_run`):
- `reviews/same-id-order` (G2 N-1): one id refused alone in `n1/a/s2.tally` and `n1/z/s1.tally`. The engine follows P5's path order; refcore's `check` orders by scenario id. Held for #228.
- `reviews/twin-placed` (N-9, P12): `twin` declared in `p12/b/` and `p12/d/`, beside `p12/c/`'s refused `alpha`. The engine places it by its first file; refcore's `check` puts it after `alpha`. Held for #228.
- `reviews/narrowed` (#232's second case, P19): a scenario's `moves register_10 to 2026-10-01..2026-10-20` leaves `bank[r2]` (10-28) standing outside. Refcore loads it (`ok`); the engine refuses `LOAD-COVERS`. Held for #232.
- `run/ask-before-pin` (N-9): `--pin=bytes` after `--ask` is the option. Red until P18 (G3).
- G1 A-4 (a walk-bound def read by a cycle member's `when` at a point with no recorded run): not probed in C3.

**Step b** (the builder's plan §5):
- `m1_lows`, `m2_counts` and `m2_forks` now call `run_with_how(Files, &Request::new(asks), How { lows | forks, ..How::FENCES })`, with no `run_with_lows`, `run_with_forks` or `Request` literal. All three pass. `m2_forks.rs` was generated by `tmp/m2/t/gen_forks.py`, which now differs.
- `m3_run_the_library_prints_what_the_cli_prints`: every M3 run vector and `check` fixture through the library's entry, on files T resolves per INTERFACE §4, prints the CLI's bytes and exit, or stops where it stops. It is green.
- INTERFACE §4's fault section now names `run_with_how`'s `How { fault, .. }`.

## C5: the survivors left, the fixes' vectors (chunk C5, 2026-11-19)

**Gate 8** (still 4fa66bfd's list, run 44411247874; 15b59db7's run was under way). Eleven kill texts, `m1/fixes/gate8/m3_*/base.tally`, gating in `fixes-gate8`, each the same on both engines (green on arrival):
- `m3_answer_apart`: a settlement bound over adjustments whose amount is a hole at its day, answered by a later version, beside one whose stated amount a later version changes (never measured): `answer_apart` ×5.
- `m3_uncertain_hole`: uncertain movements with hole amounts, in and out, read `with uncertain partial` and `with uncertain`: `Uncertain::leg` `> → <`, `> → ==`; `uncertain_sum` `|| → &&`.
- `m3_gap_outside`: `gap(x, d)` before the run's start, on x's reading day, another day, and another place's reading day: `stated_on` → `Ok(true)`, `&& → ||`.
- `m3_less_own`: a decision fulfilled two days early, read less its observation at both of the leg's places: `less_own_obs` `&& → ||`.
- `m3_own_earlier`: the same decision fulfilled on the run's first day, and one fulfilled on its own day: `own_earlier_obs` `< → <=`, `< → ==`, `|| → &&`.
- `m3_modelled_day`: `bal(x, d) modelled` on the very day of a modelled-past leg: `modelled_legs` `> → ==`, `> → >=`.
- `m3_after_run`: flows after the run's end past an instance whose `when` is false: `after_run` `|| → &&`.
- `m3_window_flows`: flows over spans after windows closed, across one, and past hole-dated movements; a move's flows over a window (`flows_hole`); a user question that ends " for a balance" (`Asked::for_read`'s guard): `place_log` ×4, `flows_hole` → `None`, `for_read`.
- `m3_two_readers`: two moves of one stratum and one day read a walk-bound def, each cell at its own point: `schedule_decision` delete `rank`.
- `m3_window_end`: a window-dated decision whose last day has an end-of-day reading (the window's end after the closing readings: #218's ground, both engines alike): `schedule_decision` delete `stratum` (and `rank` at the end point).
- `m3_ledger_days`: a residual line whose model figure is a hole carrying an unknown day (`unknown day: due_day` in the register). Aimed at `Fig::hole_days_into` and `Line::hole_days_into`, but T's proof of `Line::hole_days_into → ()` is **missed** (146 tests selected, this one among them): the register gets the day by another path. Not counted; for the builder (equivalent, or a case T hasn't found).

That is 25 survivors aimed at. T proved `stated_on → Ok(true)` caught with `mutants-local` (5 tests selected, caught in 12 s on this tree); the rest are argued from the code and wait for gate 8's next run. New rows for G3 in `tmp/m3/t-equivalent.md`: § A2, 4 `R-87: equivalent:` (`Of::counts` ×2 and the `flows_place` guard: every place holds one unit since tally 2.2; `and_list`'s guard: every caller names two or more) and § B2, 10 `R-87: missed:` (`same_booking.rs` ×7 and `planned_by_id` ×3: a `same` group holding a planned or decision instance is LOAD-SAME in one model, a joint model's, M5c's).

Estimate once G3 lists A and A2 (28) and the 25 are caught: (2 416 + 25) ÷ 2 732 = 89.3 %. 90 % needs about 18 more kills (k + 0.9·e ≥ 68: 25 + 0.9 × 28 = 50.2).

Left unsorted for C6: `walk/readings.rs` ×3 (a decision window's fallen places; the sign rule's anchor), `walk/state.rs` `imp_range`, `partial_legs` and `Hole::shares` (the sign rule's window tests), `walk/point.rs` ×2, `walk/day.rs` ×3, `walk/plan.rs` `class_depends` ×2, `LegVal::eq` ×2, `walk/tuples.rs` `link_tuple`, `walk/move_flows.rs`'s guard, `walk/reads.rs` `partial_bal`, `walk/agenda.rs` `left_out_lines` and `schedule_plan`'s `rank`, `book/adjust.rs` `measure_move`, `walk/value.rs` `priced_position` ×2 (load refuses every form T tried: the run-time check looks unreachable, for the builder's word), `book/same.rs` `stated_same` → `true` (likely equivalent: a movement in no `same` group books both ends as written), `read/rows.rs` ×4 (the builder's `choose` question).

**A load gap both engines share** (`tests/out/p-c5/lawdate_binding.*`, not a vector): a settlement move bound `a in adjustments on act_day` where `act_day` is law-dated loads; refcore then prints `REFCORE-NOT-YET … which L §4.3 refuses at load (LOAD-LAWDATE)`, the engine `ENGINE-INVARIANT unreachable: load refuses a dated read with no law date`. L §4.3 says `LOAD-LAWDATE` at load.

**The fixes' vectors** (`m3/fixes/g3/`, copies of the G3 review's probes; group `fixes`, `m3_fixes_review`):
- `fixes/stacked-covers` and `fixture/reviews/stacked-covers` (B-1): a scenario over one P19 refuses `LOAD-COVERS` is refused with it. Refcore loads both (#232's case). Held for #232. The control, `fixes/stacked-unit` and `fixture/reviews/stacked-unit` (over a `LOAD-UNIT` one), is the same on both engines.
- `fixes/member-warning-id` (B-2), `fixes/member-refused-alone` (B-3), `fixes/member-clock` (B-4), `fixes/scenario-files-clock` (B-5): refcore's bytes stored, the engine follows the spec. Held for #233.
- P21: `run/joint` is now `--ask a.q` (green); `run/ask-before-pin` asks `a.q --pin=bytes` (green). The joint model's own ask `x` over its members is M5c's, by name: no M3 vector.
- G1 A-4, `fixes/cycle-when` (`m3/fixes/a4/cycle_when.tally`): a walk-bound def read by an unordered cycle's `when`s. No `ENGINE-INVARIANT`, but the engine differs from refcore: it gives the act's point and the cycle's first instance one point (`At { n: 0, ..at }`), so `explain avail[…]` lists 2 cells where refcore lists 3 (the act's, named by its move, then each instance's), and the first instance's explain loses `read at its walk point: konto` (its re-read is a memo hit at the same point). The engine's own doc of `run_cycle` says each instance runs "at its own point after the act's", as refcore's `run_group` does. Red for G3 (report-only).

## C6: gate 8 to 90 %, the backlog, #114 (chunk C6, 2026-11-19)

**Gate 8** (run 44438723712 on 15b59db7: 87.6 %, 98 unlisted survivors, `tmp/m3/gate8-15b59db7.txt`). New kill texts, gating in `fixes-gate8` unless named, each the same on both engines:
- The builder's five (`tmp/m3/builder-texts/`, copied as `m1/fixes/gate8/m3_g3_*`): `m3_g3_counts` (an outside place's per-unit residual ledger: `Of::counts` ×2, so C5's two equivalent rows are withdrawn), `m3_g3_priced_any`, `m3_g3_priced_kind` (`RUN-DOMAIN value(market) in PLN`: `priced_position` ×2), `m3_g3_tie` (a read-time `LOAD-OVERLAP` tie: `row_place`, `version_file` ×2 each), and `m3_g3_a3`, the one of the builder's three #235 probes that loads (a settlement bound on a date fact governed always; `a1`, `a2` are refused at load).
- `m3_just_before` (the turn-4 review's N-6): two defs reading `bal(konto)` at a decision's day and at a later day, explained, beside two decisions of one day: `At::just_before`'s three `R-87: missed:` rows.
- `m3_window_class`: a plan dated by a window that starts on the run's first day, one by a window a statement covers every day of, one into a place whose life starts inside its window: `class_depends` ×2, `window_hole_ends` → `Ok(())`.
- `m3_left_out`: a def's projection whose window holds two days of two left-out instances each (`left out: put, 2 instances` then `4 instances`): `left_out_lines` `== → !=`.
- `m3_less_cov`: `m3_less_own` with jar covered in January by a version of its own, so the early observation is read back at konto alone (the sweep 800.00, not 1 500.00): `end_booked` → `true`.
- `m2_counts_uncertain` (`m2/counts/unc.tally`, gating with M2's counts; pick P26): `days_c`'s projection reads taken `with uncertain`, so they walk fence F6's projection walk, not F5's tree, and its counts differ on and off; `u4`, one read on rising days across two month ends, meets the parked walk past its plan, so a new plan pass continues it (144 days stepped on, 162 off). Aimed at G3's 18 `walk/fork.rs` `R-87: missed:` rows (`continue_planned`, `wait_to_continue`, `keep_walked`, `agrees`, `same_*`, `paused`), `step_day`'s `forks == On` (it skips `keep_walked`) and `LegVal::eq` ×2 (F6's `same_act`).

**Proofs** (`mutants-local --next`, niced, one at a time; logs `tests/out/p-c6-*.txt`): `continue_planned → Ok(())` was **missed** with unc's first four asks (480 tests selected, `m2_counts_uncertain` among them: no read continued a parked walk), hence `u4`; with it **caught** (481 selected). Then on that tree: `agrees → false` caught (38), `Walk::keep_walked → ()` caught (203), `LegVal::eq → false` caught (38), `measure_move`'s `!` at `settles && !is_hole_booking` caught (455: C5's `m3_answer_apart`, not in 15b59db7's run).

**Estimate.** Aimed at this chunk: 8 (the builder's texts) + 3 (`just_before`) + 3 (`window_class`) + 1 (`left_out_lines`) + 1 (`end_booked`) + 19 (F6's 18 and `step_day`) + 2 (`LegVal::eq`) + 1 (`measure_move`) = 38, 5 of them proven, the rest argued from the code. The coordinator's count needed about 35 past C5's 25 and G3's 8 (88.7 %), so about 90 % if gate 8's next run catches them. Equivalent rows: C5's two `Of::counts` rows withdrawn, the `flows_place` row's reason corrected (`tmp/m3/t-equivalent.md` § A2); no new ones.

**#114's remainder.** Refcore on R7 to 2033-11-30 passed the cap (8.1 GB after 45 s, stopped by `memcap.py`); to 2032-10-31 it ran in 63 s (`m2/bank/r7/to2032.tally`). Refcore and the engine print it byte for byte (520 lines): konto 12 616.43 PLN and net worth 858 027.83 PLN on 2031-10-17, 12 611.05 PLN and 999 320.71 PLN on 2032-09-17 (2032-10-17 is a hole there on both, after the run's end through the 15th's decision). So P55's route: `pins_long.tsv`'s net worth on 2036-10-17 and 2056-10-17 take the engine's figures (1 746 061.56 PLN, printed again on 2026-11-19 in 3.8 s and 0.7 GB; 10 093 756.45 PLN, from 2026-11-15, as its run takes 12.5 GB), the prototype's beside them (−15.92, −54.80), the gap named the prototype's, unjudged after 2032; no longer held. **The two digests** (`reality.digest`, `case2.digest`) are blocked by design: a hash over the prototype's own output lines, which no tally 2 run prints. The pull request asks the user to strike them from gate 2.

**#192's standing items** (T's plan §5; `m3/fixes/b192/`, group `b192`, `m3_fixes_backlog`; both engines alike):
- `b192/solve-units` (G1's fix N-c): `solve` where the unknown is a rate (`%/year`) and where it is a `num PLN`, each on a fine grid and a coarse one whose nearest point isn't the root. An off-grid bracket is `LOAD-DOMAIN` on both.
- `b192/parked-prefix` (the review of P73's fix): one projection read on falling days within a month, across a refused leg (`RUN-NEG` at jar from the 21st) and a hole (konto's), each from the parked walk's prefix.
- Left standing: G3's list on plain window values (about 6), G2's `explain` repeat lines (a spec question if they still differ), G5's `q2_order` family (about 8, texts in the main folder's `tmp/m2/review/`), G5's NB-2 (settle caches keyed without `upto`), and the kills gate 8 lists (`cross_turn`, `at_place`, `thing_key`, P60's).

## C7: the gate's vectors (chunk C7, 2026-11-19)

**#235's forms** (the gate review's B-1 to B-4 and the fix review's G-1, on `build/m3` 7660a91f; texts `m3/fixes/lawdate/`, copied from the two reviews' probes, each dated text beside its law-free twin where it has one):
- **Gating where refcore answers** (`m3/runs.vectors` group `lawdate`, `m3_fixes_lawdate`, 33): B-1's calendar read by `is_business` in a no-`at` key's `where` (five weekdays paid); a month-keyed input at its key's month; B-2's `on add_business(d_open, 1, cal)` before the `at` (loads); a dated `where` that refuses itself; and 29 of the fix review's forms × `at`/no `at`: a calendar, a def over it, a def with its own `at` and `as if {law <fixed>}` need no law date, with or without the move's `at`; in a move with no `at`, an input, `as if {law act_day}`, a def whose `at` reads a law-dated input, an aggregate's `where`, an `at reader` def and a def through one are `LOAD-LAWDATE … states no at`.
- **The spec's answers where refcore prints its guard** (`m3/spec.vectors` group `spec-lawdate`, `m3_fixes_spec_lawdate`, 37, two of them held; refcore's guard beside each row in `m3/spec.tsv`): a law-dated read in a key's `on` before its `at` is `LOAD-LAWDATE move <m>'s key <k> reads law-dated <x> with no law date: its at reads <k>` at load (B-3's `mean(… where …)` and `as if {law act_day}`, G-1 B's `cal[region]`, the fix review's input, `as if {law act_day}`, a def whose `at` is dated, an aggregate; P28; the code is L §4.3's, the words the engine's until #235's round writes the key's form into L §8); G-1 A, `is_business(d, cal[region])` in a no-`at` move, is L §8's `pay reads law-dated facts and states no at`; and a key's `where` waiting for its tuple's `at` prints its law-free twin's bytes, refcore's run of the twin (B-3's `where`, G-1 C and D, four of the fix review's `at` forms, item 1's true case, and 19 of the gate's P30 probes: true, false, refusing, two waiting, before a law-free one, a later key's refused `on`, an `at` reading only the later key, an aggregate inside, a hole `at`, keys over accounts, places and months, `unit(p)`, and the asks `flows`, `explain`, `as known`, `with`, `residuals`, `readers-of`, `for d in`).
- **Held for #235, the engine's bytes as they are** (`held.tsv`): the gate review's item 1 (`spec/lawdate/i1-atref-false`: a refused `at` on a tuple whose waiting `where` is false refuses the ask, `RUN-NONE never in the law date of shut`, where its twin answers) and item 15 (`spec/lawdate/w-hole-at-false`: a hole `at` there adds `gap_day: no value` where the twin drops the tuple). The spec doesn't say which.
- **Not stored:** P30's `def_keyed` (its twin is refused by refcore, `LOAD-LAWDATE ask reaches an at reader def`, P30's known difference), the fix review's `at reader` forms in a key's `on` with an `at` (refcore refuses them with another `LOAD-LAWDATE` form: item 13), its pinned reads (N-3, a spec question for #235) and the forms that stop `not implemented` (N-2).

**B-4** (`m3/fixes/alone/`, `m3/spec.vectors` group `spec-alone`, `m3_fixes_spec_alone`, 3): `run` of a reality whose scenario file holds an ask naming no world prints that ask's `LOAD-SCENARIO` line in its place by id and answers the rest, exit 0, as `check` prints it (OPERATIONS §2.1, §2.3): every ask, `--ask q_gone` alone, and SCN-4's diff against a dropped scenario. Refcore's run prints nothing for it (#229); `m3/vectors.py` ties each answer, less its asks refused alone, to refcore's bytes.

**#192's open rows** (the builder's plan's numbers; `m3/fixes/b192/`, group `b192`, `m3_fixes_backlog`, refcore's bytes):
- Row 50 (G5 fix NB-2, `settled` and `unsettled` keyed by cut where refcore's caches carry a prefix walk's day): `b192/prefix-settle`, an ask whose def sums event legs, so the read needs the walk up to a leg's day while a plan's `when` reads the def, before a settlement bound over adjustments is listed; then the whole walk's balance, the adjustments' count and the settlement's flows. Three adjustments and three back pays on both engines, in eight forms tried (the trigger in a key's `where`, a `when`, an `on`, planned before or after the adjusted move). No wrong number reached; without a counters build T can't show that the prefix walk lists the settlement first, so the row stands as a vector, not a proof.
- Row 22 (M2 G3 review NB-4, `qp`'s plain hole): `b192/cal-hole`, a calendar with no row and one whose row is a hole, read by `add_business` and `is_business` in asks, a def, `explain` and a key's `where`: every hole carries its `(cal)` tag on both.
- Row 51 (G5 fix NB-3, `trace_answers`' stop): `b192/trace-answers`, `explain` over two hole bookings a later version answers, the second answer `RUN-DIV`: the explain refuses as the balance does, and an explain before it answers; the same on both.
- Row 52 (G5 fix NB-4, `HOLE-SAME`'s leg follows `row.legs`): `b192/same-two-legs`, two rows stated `same` of a kind whose two legs both leave konto, disagreeing on the fee: explain, flows and the balance print the same four `HOLE-SAME` holes on both. **Red:** `b192/same-two-legs-residual`, `residuals konto 2026-03` over them: the unexplained group's line shows bank[a2]'s fee leg's hole on refcore and its cash leg's on the engine (refcore's `same_rows_hole` is the engine's; the difference is in which row or leg the ledger's line passes). Held for #192 in `held.tsv`, for the builder. Both engines also name bank[a2] twice in each hole ("bank[a2], bank[a2] and bank[b1] are one"): a row's two legs at one place form one class; a question for the round.
- Row 40 (G4 fix NB-3, `chain_pads` at a continued walk's start): `b192/chain-pads`, two decisions reading pads up to their own day, the second's walk continuing the first's: the same on both. For the round to read: both print `sum(p.cash.amount for p in pads over 2026-04-01 .. 2026-04-15)` as 350.00 PLN, with the pads −200.00 (04-10 start) and 03-15's 500.00 listed under it, where the April pads alone sum to −50.00 by T's count; T doesn't judge it by hand (the drift rule), and refcore agrees.
- Rows 21, 26, 53: not written in C7.

## Holds

- `last/net_worth` (`m3_fixes`) for **#234** (C6, pick P24): net worth sums money with a value in `num PLN` (L §3.2: `num CUR`); refcore's dyn branch prints money (`5 400.00 PLN`), the engine the spec's `5 400 PLN`; refcore is fixed at the round.
- `fixture/alone/scenario-file` and `fixture/alone/same-id` were held for **#229** (the coordinator's ruling, 2026-11-18): refcore's `run` makes this load check only while answering, so it prints nothing for an ask in a loaded scenario's file that names a world not loaded, where L §6.4 and §8.3 refuse it alone. They now store refcore's `check`, which prints the spec's lines, so the engine is held to the spec. Refcore's `run` is fixed at the round (#229).
- `fixture/reviews/same-id-order`, `fixture/reviews/twin-placed` (#228) and `fixture/reviews/narrowed` (#232), `m3_check`: § C3.
- `fixture/reviews/stacked-covers` (`m3_check`) and `fixes/stacked-covers` (`m3_fixes`) for #232; `fixes/member-warning-id`, `fixes/member-refused-alone`, `fixes/member-clock`, `fixes/scenario-files-clock` (`m3_fixes`) for #233: § C5.
- `spec/lawdate/i1-atref-false` and `spec/lawdate/w-hole-at-false` (`m3_fixes`) for **#235** (C7): the engine's bytes, held while the spec doesn't say whether a refused or hole `at` over a false waiting `where` refuses, adds its hole or drops the tuple. The 35 other `spec-lawdate` rows and the 3 `spec-alone` rows gate on the spec's bytes; refcore's guard (#235) and its silent run (#229) sit beside them in `m3/spec.tsv`, and `m3/vectors.py --check` fails when refcore's bytes move, so the round moves each row to `m3/runs.vectors`.
- `b192/same-two-legs-residual` (`m3_fixes`) for **#192** (C7, row 52): refcore's bytes; the engine's residual line shows another leg's `HOLE-SAME` hole.
- **To M4, by name:** every walk ask of the seven changes (load row L:10, P8-L10), and change 5's after-side runs (§ Row 6).

## Keeping master green

As at S, M0, M1 and M2: every `m3_…` target has `test = false` and runs only as `check`'s report until M3 is in `engine/tools/milestones.toml`'s `gating`, which the rules step adds in `build/m3`'s commit before its gate.

## tally 2.2.1, chunk C1 (#241, 2026-11-20)

T's plan for the round (the lead's copy, `tmp/spec-2.2.1/t-plan.md`), chunk C1, on today's refcore and engine:
- **Guards** (`m3/fixes/t221/`, group `t221-gd`): 24 runs (`m3_fixes_t221_guards`) and 2 checks (`m3_check_t221_guards`). They are answers the round leaves as they are, each the same on both engines, which its changes must not move: S-208.3, 208.4, 208.6, S-216's five, 228.Q12 (C-14 option A), S-231.Q4, S-window-key's day key, the sign rule, a binding over a span the window crosses, S-231.Q2's twin, 235.8, R18's cell 1, 231.Q6, S-230.Q5b, S-233.5, 229.2, 221.10, S-233.8's twin, and 233.c's load words.
- **221.C3** (`t221-g1/221-C3-arith`, `m3_fixes_t221_g1`): refcore's bytes, where the engine holes at `+`. It is held for #241 in `held.tsv` until the builder's G1, because a `Logged-Red:` commit can't be pushed before a builder commit (R-77).
- `fixes/t22/fault_paths` is re-held to M7 by name.
- `baselines/session5-visible/` is retired; its seconds are `defs/session5-seconds.tsv`.

## tally 2.2.1, chunk C2: the windows (#241, 2026-11-20)

On refcore at the window group's pin (bcc49686). Every store was re-run (P23), each script at its own job count, niced.

**Vectors** (`m3/fixes/t221/`):
- Group `t221-g1`, 13 runs, red on the engine until the builder's G1:
  - 214.1: `214-zero-leg`, `214-one-day`.
  - 218.1: `218-window-end`.
  - 220.1: `220-1-covered-end`.
  - 230.Q4: `230-4-undone`.
  - 236.1: `236-1-differs-nowait`, `236-1-agree-false-nowait`.
  - 221.C2: `221-C2-list`.
  - S-window-key: `window-key-month`, `window-key-two-months`.
  - S-from: `from-across`, `from-after`, `from-before`.

  Three of them (`window-key-two-months`, `from-after`, `from-before`) already print refcore's bytes on the engine. They hold the twins G1 must keep.
- `t221-rc/221-10-mean-quantile` is held for M4 (`held.tsv`): the engine has no `mean` or `quantile` before M4.
- Five `later/` texts: M5a's 221.R2 and S-213 with its two twins; M7's S-212.
- 221.R3 (P22): m1 `fixes/g13` `r_agent_w_{c1,c3,c5,f1,f2,f3}` and `rf_w_{w01,w11,w13}` move from `m1/spec.tsv` to `vectors.tsv`, `for` t221-G1. Their bytes are refcore's, and they match the writer's probe lines.

**The proof: what moved, by item.** Nothing else moved.

| Item | Records |
|---|---|
| 218.1 | m1 `fixes/gate8/m3_window_end` (the holes after the window's last day become 900.00 PLN; the `unknown day: save_day` lines go) |
| 214.1 | m2 `fixes/rv/g4f_zero_leg`, `g4f_zero_leg_life` (the hole becomes 0.00 PLN); I:M:CMP-01's baseline (+1 `WARN-NEGATIVE` line, in C2's first commit) |
| 221.C2 | m2 `fixes/t22/list_window_name` (`[window(4, 6), 1]` becomes the window's hole) |
| 221.R3 | the 9 m1 rows above (spec rows released) |
| R:all | 1 514 passed |

- m1 1 977 records, m2 296, kernel 166, codes m0 252, m1 38, m2 9, m0 load, m3 check 383, runs 124, spec 40, later 13, hh4 75. Integration 479: 395 same, 84 not run. Unit 154: 153 same, 1 ran.
- `m1/spec.tsv` keeps 10 rows (e1 and e2 still crash refcore). `m2/spec.tsv` keeps 9: `num_list_key` still prints the object 207.4 mends at W5.
- **The STOP-REPORT** (GAPS.md, C2): refcore at the pin passes 8 GB on m2's five R7 goldens. The cause is S-212's context per projected instance. Their records stay as stored, and `m2/vectors.py --check` is red on them alone until the writer's mend.

## tally 2.2.1, chunk C3: the second pin's proof (#241, 2026-11-20)

On refcore at the second pin (0818f488: W6, W5, W7, W2, W3, W1; 1 597 tests). Every store was written from it, each script at its own job count, niced, and diffed against HEAD record by record.

**The proof: what moved, by item.** Nothing else moved.

| Item (writer, builder) | Records |
|---|---|
| 206.1 (W6, G2) | m1 `fixes/gate8/m2_g7_p1_cuts`, `m2_g7_p2`, `m2_g8c_b1f`, `m2_g8c_b2` |
| 221.5 (W6, G2) | m1 `fixes/gate8/m3_window_class` |
| 219.2 (W6, G2) | m1 `m2_g4_adj_amt`, `m2_d512_day_rows`, `m2_d512_link_same`, `m2_d512_two_groups`; m2 `fixes/g4/adj_amt`, `fixes/t22/rv3_scen_over_base`; m3 `b192/same-two-legs`, `b192/same-two-legs-residual` (both with 237.2) |
| 208.2 (W6, G2) | m2 `fixes/rv/g2_b3_both_break`, `rc_refusal_words_differ` |
| 221.1 (W6, G2) | m2 `d77/sign`, `fixes/t22/cut_each_ask_twin`, `items/explain/from`; `fixes/t22/cut_each_read_twin` (its class, on no build-log list) |
| 220.2 (W6, G2) | kernel `g10/ledger`; m1 `fixes/g12/pm_ledger_ledger`, `pm3_…`, `pm4_g10_…`, `pm8_g10_…`, `prr_projunc_g10_…`, `r_pb_bal-ledger_…`, `v_mvq`; m3 `t221-g1/220-1-covered-end` |
| 237.1 (W6, G2) | m1 `m2_d512_alone_pad`, `m2_d512_pads_run`, `m2_g4_p_pad`; m3 `b192/chain-pads` |
| 208.5 (W6) | I:A:a10's baseline (`assumed:` under a refusal) |
| P25 (W6, G1's fix) | m1 `fixes/g13/r_agent_w_model_with_r_agent_w_c2`, `rf_w_model_with_rf_w_w15` (`round` gives the interval: "for a print") |
| P27 (W4, G1) | `t221-rc/221-10-mean-quantile` released |
| R21 (W5, G3) | m1 `fixes/g2/p4_f2`; m2 `fixes/t22/rv4_b2_pin_calendar`, `rv4_b2_pins`, `unknown_day_names`; m3 `t221-g1/221-C3-arith` (both windows named); 11 baselines |
| 237.2 (W5, G3) | m1 `m2_d512_both_cover_day`, `m2_g4_g3_amt`, `m2_g4_p_both_konto_amt`, `m2_rt_e6_3_…`, `m2_rt_l6_1_…`; m2 `s6/hole_same_order_class` |
| 221.6 (W5, G3) | m1 `fixes/gate8/m2_g3_fix2_c2`; I:E1:tx2_owes_explain's baseline |
| 233.7 (W5, G3) | codes/m1 `rules/none`; m2 `spec/g1/qn` released (with 208.10) |
| 207.4 (W5) | m2 `spec/p60/num_list_key` released |
| R20, 233.8 (W2, G4) | released: `fixes/member-warning-id` (red here); `member-refused-alone`, `member-clock`, `scenario-files-clock`, `fixture/reviews/same-id-order`, `twin-placed` (green here) |
| S-223, S-232 (W3) | released, green here: `fixture/reviews/narrowed`, `stacked-covers`, `fixes/stacked-covers` |
| R18 (W1) | the 40 `m3/spec.tsv` rows' refcore cells (their answers); `spec/lawdate/i1-atref-false`, `w-hole-at-false` released (green here) |
| the household's notes (W6) | file tables only: kernel 3, m1 20, m2 6; m0/format 24 lines and 14 hashes |

- **Byte-equal:** m0 load 327; codes m0 252 and m2 9; loadcheck 90; the laws (570, 648, 168); later 13; hh4 75. m3 check 383 and spec 40 also matched, but for the rows above.
- **The R7 goldens:** m2's five ran, byte-equal, at the pin (C2's STOP-REPORT mended by W6's 82e80319).
- **Integration:** 479: 382 same, 13 re-stored (above), 84 not run. Unit 154: 153 same, 1 ran.
- **Still held:** `fixes/t22/fault_paths` (M7) and `last/net_worth` (234.1, which W5 didn't take).
- **T's texts:** `misc2/m2.tally` loses `before top`, which S-221.9 refuses.
- **Waits for a tree with G2:** `m2_counts.rs`' five rows (206.1: `r2`, `u2`, `u4`, `r7`, `u3f`). They hold the engine's counters, not refcore's, so they can't be re-pinned on this branch.

## tally 2.2.1, chunks C4 and C7: G2's vectors and refcore's (#241, 2026-11-20)

On the build branch merged at 7e4ae092 (G1, G1's fix, G2, G2's fix) and refcore at the second pin (0818f488).

- **C4, G2 (`t221-g2`, 11 rows):** 206.1, 208.2, 208.5, 219.1, 219.2, 220.2, 221.1, 221.5 (with 208.1's `?` line), 231.Q1b, 231.R2 for a def, 237.1, refcore's bytes, green on this engine. 208.10's row (`qn`) waits for G3: 233.7 names the read (`lvl is none here`). 208.8 isn't written: no text holds `value … in PLN` under `partial` (M5a's form).
- **The counts:** `m2_counts.rs` re-pinned to this engine for 206.1 (`r2`, `u2`, `u4`: one leg more; `r7`, `u3f`: fewer walks, more shared).
- **C7, refcore alone (`t221-rc`, 16 rows and 1 check fixture):** 221.R1, 222.1, 228.Q5 (check: a cycle's refusal placed by its header's file), 228.Q9, 231.Q10 (written orders compose; the first by id not due that day), 231.R2 for a place, 235.3 (and its refused twin), 235.10 (and its twin), 235.13 (and its twin), 236.2, R18's cell 2. 235.8, R18's cell 1 and 229.2 were already guards (`t221-gd`, C1).
- **Held for refcore (`held.tsv`):** S-231.Q1a (`t221-g2/231-1b-whens`, `t221-rc/231-1a-conflict-order`: refcore names `bz and bx`, step 3's order is `bx and bz`), 231.R1 (`t221-rc/231-R1-explain-in-s`: refcore's axes not in id order), 236.3 (`t221-rc/236-3-over-world`: the writer's W8 mends it after the pin). Not written: 208.R2 (refcore refuses each ask `REFCORE-NOT-YET`; T's text waits in T's notes).
- **To G3 and G4:** 221.C1's `min(window, w)` (refcore one question per unknown day, R21: G3); 228.Q5's `run` (the refusals in path order, R20: G4).

## tally 2.2.1, chunk C8: the first review's findings (#241, 2026-11-20)

- **12 vectors, `t221-g1/rv-*`,** from the review's own probe texts (`m3/fixes/t221/rv1/`), each ask beside its pinned days: B1 (`pow` over 517 days), B2 (`day_of`, a refusal only the ends give), B3 (two windows' quotient), B5 (two cuts in one run, no `ENGINE-INVARIANT`), N2 (`round`, P25), N4 (a version known after the window), N5 (`month_of` across two months), N6 (one window's divisor), N8 (a reading's own day, P32), N9 (one name in two arguments), P25's `abs` and `pow` across 0, P26 (a scenario's move across `from`). Refcore's bytes, green on this engine.
- **Not yet (P41):** B4 and its kin, N1, N3, intervals over 100 000 values. P26's `p0430` (a pin that leaves `px` out prints `pinned:` on the engine since G2's review fix) waits for the last pin.

## tally 2.2.1, chunks C5 and C6: G3's and G4's vectors, red until built (#241, 2026-11-20)

Refcore's bytes at the second pin; one local commit with a `Logged-Red:` line per id (R-77: no engine commit follows it yet).

- **G3, the names (9):** `t221-g3` 221-2-two-windows, 221-3-two-calendars, 221-4-def-window, 221-6-held-calendar, 221-C1-min-window, 235-6-bound-key, r21-c1-move-on, r21-c2-two-moves; `t221-g2/208-10-quantile-none` (233.7's name). Not written: 221.8 (the writer's W8 calls it a guard both interpreters print, after the pin), 233.7's own text (its three stored records carry it), 233.c's run form (the load words are `t221-gd/233-c-load-words`).
- **G4 (15):** `t221-g4` 221-9-before-self, 223-replace-covers and its check, 228-2-run-path-order, 228-5-no-file-run, 228-11-member-warnings, 231-2-pin-balance, the check 232-per-cut, 233-8-known-paren, and S-usage's six `usage-*` (exit 2). Green on arrival: `t221-rc/230-5a-same-order`, `233-a-unit-literal`.
- **Not yet written:** `reprint-pin-bytes` (the runs store takes only `run` rows and usage errors), 228.Q10's member `on`, and the four form fixtures (`LOAD-ORDER`, `LOAD-COVERS`, `LOAD-INSTANT`, `LOAD-VERSION`, `codes/m0`).
- **`INTERFACE.md` §1:** S-usage's words (an option the command doesn't take is exit 2; `reprint` takes `--pin=bytes`); gap I-4 closed. **`MILESTONE-M0.md` row 5:** re-stated from `xref_check.py --gate M0` at tally 2.2.1: 101 codes, 54 `LOAD-*`, 268 forms (175 `LOAD-*`, 20 with no output). **The catalogue:** `map/codes.tsv` as `xref_check.py` writes it (no row moved in this chunk; `WARN-STALE` left at C3).

## tally 2.2.1, chunk C9: the third pin's proof, and the review vectors (#241, 2026-11-21)

On refcore at the third pin (a02d9aec: W9, W8, W10, W11; 1 648 tests) and the build branch at d3d1eba5 (G1's and G2's last fixes). Every store was written from the pin (`proof2.sh`, 3 421 s) and diffed record by record.

**What moved, by item.** Nothing else moved; m0 (load, format), codes m0 and m2, loadcheck, the laws, later and hh4 are byte-equal.

| Item | Records |
|---|---|
| P35, the register under every form and outcome (`open:`, `assumed:`, `pinned:`, `anchor:`, `unfulfilled:`) | kernel 6, codes/m1 3, m1 45, m2 12 (the five R7 goldens: `r7_g3` gains its `open:` lines), m3 `t221-g1/from-across`; 18 check baselines (37 integration checks and U:ask/ASK-03) |
| `WARN-LEGS` by a day and in one version (B5, P43) | m1 `m2_d512_both_cover_day`, `m2_d512_day_rows`, `m2_g4_s_day`, `m2_g8b_sc1`, `m2_g8b_sc2`, `m2_rt_l6_1_same_in_model_disagree`; m2 `s6/hole_same_order_class` |
| P43 S2, membership in a span | m1 `m2_d512_both_cover_day` `t_sum`, `m2_g4_s_day` `q_moved` (the hole becomes 950.00) |
| `rf_w_w15` (refcore was wrong; the hole is owed) | m1 `fixes/g13/rf_w_model_with_rf_w_w15` |
| 234.1, 236.3, S-231.Q1a, 231.R1 | m3 `last/net_worth`, `t221-rc/236-3-over-world`, `t221-g2/231-1b-whens`, `t221-rc/231-1a-conflict-order`, `t221-rc/231-R1-explain-in-s` |
| 208.R2 | m2 `spec/p60/ask_for_law` released (refcore answers) |
| `restate_g12`'s rule over refcore's new lines | m1 `fixes/g12/pb_g12_members_members`, its asks restated |

- **Released:** `last/net_worth`, `t221-g2/231-1b-whens`, `t221-rc/231-1a-conflict-order`, `t221-rc/236-3-over-world` (green); `t221-rc/208-R2-for-law` written (both print the same). **Still held:** `fixes/t22/fault_paths` (M7); `t221-rc/231-R1-explain-in-s` (a finding: refcore fans out over every axis outside the pins, the engine over the axes the value reads).
- **Vectors, none whose text holds a window:** the second review's B1 to B5, N1, N3, N4, N6, Q3 (N2 and N5 have only window texts); the fix review's F5 to F8, F10; P42's leg `to` and a span both days lie in; P45 (a) (G3); `t221-rc/221-8-same-row-line`, `228-10-member-on`; G3's 233.7 and 233.c texts; the four form fixtures of `codes/m0` (G4).
- **Red on this engine, not G3's or G4's (findings, logged red in the local commit):** a refused result lists no `unfulfilled:` line where refcore lists them as a value's (P35; the second review's N2): 39 records and codes/m1 `rules/debt-above`; `WARN-LEGS` for two observations of one instance that differ only on a day (`m2_g8b_sc1`, `sc2`); `m2_g4_s_day` `q_obs` (refcore the day's `HOLE-SAME` for an amount read, the engine 950.00: P36 per fact).

## tally 2.2.1, chunk D2: the last pin's proof, and the last vectors (#241, 2026-11-21)

On refcore at the round's last pin (b70993c2, `tally-2.2.1`: W12, W13; 1 665 tests) and the round's whole engine (build/2.2.1 at bc483495: G3, G4, F-A, F-B). Every store was written from the pin (`proof2.sh`, 3 241 s) and diffed record by record. Kernel, m0 (load, format), loadcheck, the laws, codes m1 and m2, later and hh4 are byte-equal.

| Item (CHANGES) | Records |
|---|---|
| 2.2.1-92, P48: a `same` line as written (230.Q-5a, N1); a header's `same` at its own line (221.8) | codes/m0 `joint/same-what`; m2 `cut/instance_first`, `fixes/t22/cut_same_rows_header`; m3 `t221-rc/221-8-same-row-line`, `t221-rc/230-5a-same-order` |
| 2.2.1-93, P48: `RUN-NONE` names the read (B2, N2); the residual head (P24) | m3 `t221-g3/233-7-none-arith` (`fee[2026-02] is none here: an ordering`), `t221-rc/208-R2-for-law` (`residuals konto 2026-03`) |
| 2.2.1-94, P48: the register lines of a same side's day and a false `when` (`pinned:`, `held:`); F-C; F-E | m1 `fixes/gate8/m2_g4_s_day` `q_obs` (950.00 PLN); m2 `fixes/g4/nb3_at` (three `held:` lines); m3 `t221-g3/p45-same-day-names` (`pinned: d2`, F-D), `t221-rc/231-R1-explain-in-s` (the axes the value reads); the eight `I:A:a12_*` checks (`held:` lines; `snap16-house/A-q12.out`) |
| 2.2.1-96, P50: `LOAD-VERSION` names its file and line (N3) | codes/m0 `basic/version-three` (`(base.tally:2)`) |

- **Released:** `t221-rc/231-R1-explain-in-s` (both print the axes the value reads). **Still held:** `fixes/t22/fault_paths` (paths, M7).
- **Vectors, none whose text holds a window** (`m3/fixes/t221/rv3`, `rv4`): G3's review B1, B2, N1, N2, N4, N5; G4's review B1 and its one-file case, B2, N1, B3 (two `reprint … --pin=bytes` rows: the runs runner takes `reprint`, O §2.2), P49's order, `LOAD-VERSION`'s line, its disagreement form and its member form; codes/m0 `joint/version-member` (the member form: 160 of 180 `LOAD-*` forms have an output at M0).
- **Red, logged in T's last commit:** `basic/version-three` and `t221-g4/rv4-version-line`, the builder's (the engine names the header's line 1, refcore the `tally <N>` statement's: the lead rules refcore's); two findings: `fixes/g4/nb3_at` (`flows` over instances a false `when` drops lists no `held:` line of the `when`'s reads on the engine, three on refcore: 2.2.1-94) and `s6/hole_same_order_class` (three observations of one instance stated same, two pairs disagreeing: refcore warns `WARN-LEGS` per pair, the engine once).

## tally 2.2.1, chunk D3: the moved pin's bounded proof, and the fix review's vectors (#241, 2026-11-21)

The pin moved by one commit, d7cc0526 (1 668 tests): L §5.19's phrase, and refcore in two places (`git diff b70993c2..d7cc0526 -- engine/tools/refcore engine/tools/refcore_fast`; refcore_fast unchanged). The proof is bounded by what can reach them (`bound_d3.py`, T's notes): `check_correction_covers` runs only where a row id has a correction, a `replace <input>[<row>] = <row>` (not `= none`) after an earlier statement of it; `joint_warning`'s new pattern differs from the old only on a lifted `WARN-LEGS`, a joint model whose output prints one. The texts of every store (kernel, m0 load and format, codes m0/m1/m2, m1, m2, m3 check/runs/spec, the checks run.py runs on refcore) were scanned: 478 records reach the first (151 checks counted with the 35 joint ones, whose baselines weren't read), none the second. All 478 were re-run on refcore_fast at d7cc0526 (699 s): **none moved**.

- **Vectors, none whose text holds a window** (`m3/fixes/t221/rv5`): the fix review's B-1 (`a.q: 50.00 PLN`, the warning names `a.put[a, 2026-03]`), B-2 (the base's own `LOAD-VERSION replace rule d.base in w names version v (b.tally:22)` beside `s3.tally` on tally 3), S-223's two corners (`cc_mid_moved`: `LOAD-COVERS … fix2 doesn't cover konto on 2026-02-10`; `cc_mid_drop`: loads, `q: 1 000.00 PLN`). R:all 1 668.
- **Green now:** `basic/version-three`, `t221-g4/rv4-version-line` (the builder's closing turn names the `tally <N>` statement's line). **Still red, logged, the builder's:** `fixes/g4/nb3_at`, `s6/hole_same_order_class`. **Held:** `fixes/t22/fault_paths` (paths, M7).
