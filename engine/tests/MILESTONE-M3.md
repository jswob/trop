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
