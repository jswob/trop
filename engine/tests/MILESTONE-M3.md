# Milestone M3: what must turn green

The checks M3 (the operations) must turn green, from BUILD-ORDER § M3 ("What M3 builds", "Budgets", "Green") and DONE gates 8, 10 and 12, with the items the run's brief puts in scope (issue #226). Written by T for issue #226 (2026-11-18) on refcore **tally 2.2**, from T's plan approved by the coordinator (picks P2–P9, `tmp/m3/auto-picks.md`; the spec questions filed as #228). The builder reads this page; T changes it.

## The list

"Today" is the engine at master ee1e9b37 (tally 2.2 and the operations document), 2026-11-18. Every `m3_…` target has `test = false` and runs only as `check`'s report until M3's marker commit (as at M1 and M2), so T's commits stay green with no `Logged-Red:` line.

| # | What | Ids, vectors | Run by | Gate | Today |
|---|---|---|---|---|---|
| 1 | `check` in refcore (the spec step's) | none of T's: chunk C4 ties refcore's `check` to the prediction of every vector of rows 2 and 3, byte for byte (`m3/vectors.py --confirm`); a difference is a STOP-REPORT | `m3/vectors.py --confirm` | 10 | refcore prints its usage, exit 2 |
| 2 | `check` over M0's 315 text sets | `m3/check.vectors`, the vectors named by each set's id (C1): 295 `ok` (with load's warnings where the set has them), 11 a base's refusal (row 1 of M0's and `I:M:MP-10`), 9 with refused lines: a scenario refused (`I:B:b10_v3`'s `hr2_v3`, RULE-09's `inpath`, SURF-04's `redefine`) or asks refused alone (`U:law/LAW-13/LAW-13`, `U:rand/RAND-04/RAND-04`, `U:scen/SCEN-02..04`, `I:E1:tx2_cis_explain`: a probe given without its scenario's file) | `m3_check`: one test per M0 corpus group (16 with the parts) | 10 | 0 green: the engine has no `check` (a usage error, exit 2) |
| 3 | `check`'s line kinds the 315 don't reach | `m3/check/fixtures.tsv`, 44 fixtures (`fixture/<kind>/<name>`, § Row 3; 9 of them OPERATIONS' examples, § Row 5) | `m3_check`: one test per kind (9) | 10 | 0 green (2 held, § Holds) |
| 4 | `INTERFACE.md`'s `check` | §1's row and §9 | — | — | written (C1) |
| 5 | the run function: the library's one entry and the CLI's; the request, asks and clocks; pure; stores nothing (E §1); OPERATIONS §3 and §7's examples | C2: `m3/runs.vectors` group `run` (10, § Row 5) and group `operations` (7 runs on `mini`); the library against the CLI once the builder names the entry; `m3/purity.py --engine` at the gate | `m3_run` (2 tests); `m3/purity.py` | 10 | `operations` green; `run` red on `run/joint` alone (`not implemented`); purity: 107 of 128 sampled vectors equal on the quick binary, the 21 `check` ones wait |
| 6 | the seven changes through OPERATIONS §6.1 (#224's fifth item) | C2: `m3/changes/stages.tsv`: 17 runs (`changes/run/<stage>`, the three law asks and T's def or diff asks, before and after each change) and 17 checks (`changes/check/<stage>`: each stage with its change `ok`, each without its advance `LOAD-KNOWN`), § Row 6 | `m3_changes` (2 tests), `m3_check` group `changes` | 10 | the 17 runs green; the 17 checks wait for `check` |
| 7 | P8-1, a conversation turn | C2: X's change-7 turn (pick P6): `changes/check/c7`, `changes/run/b7`, `changes/run/c7`; their digests in `perf-digests.tsv` (§ Row 7) | `engine/tools/perf row P8-1`, once the rules step writes its `turn:` recipe | 12 | the row waits for the recipe |
| 8 | gate 8 ≥ 90 % | C3: kill vectors from gate 8's run on `build/m3` (44411247874), `fixes/gate8/m3_*` | `m1_fixes` `fixes-gate8` (gating) | 8 | no score for tally 2.2 |
| 9 | #192's standing items, #221's first-group items, `net_worth`'s binding (P7) | C3 | `m3_fixes` | 1, 8 | — |
| 10 | #114's remainder | C6: one refcore run at the longest R7 horizon under 8 GB against the engine | `m2/bank/pins_long.py` | 2 | 2036, 2056 net worth held |

`check`'s report line counts the `m3_…` targets' tests: `milestone-report: M3: 2/25 green (gates once M3 is tagged)` after C1 (`m3_check`'s 24 tests and the one that every group has its test; that one and `usage` pass: the engine already exits 2 on `check`); `milestone-report: M3: 5/31 green` after C2 (`m3_check` 27, `m3_changes` 2, `m3_run` 2; green: `m3_check`'s two, `m3_changes`' two, `m3_run_operations`).

## Rows 2 and 3: `check`'s vectors (chunk C1)

**The bytes.** Refcore has no `check` until the M3 spec step, so `m3/vectors.py` **predicts** each vector's exit code and whole output from refcore's `run` (pick P8), every line printed by a run of refcore 2.2:
1. the load probe, `run <files> --ask zz_m0_probe` (`m0/load.refcore`'s for the 315 sets; run for a fixture): a load refusal there is `check`'s whole output, exit 1;
2. else the warnings probe, `run <files> zz_m3_probe.tally --ask zz_m3_probe`, whose one ask names a world no file declares, so it is refused alone and nothing is evaluated: it prints every refused scenario's line and load's warnings;
3. a run of the asks a scan finds naming a world no file declares (`in`, `against`, `with scenario`, `as if {scenario …}`): each `<q>: refused: LOAD-SCENARIO ask <q> names …` line is an ask refused alone;
4. OPERATIONS §2.3's assembly: `ok`, or the scenario lines in their files' full-path order (P4), then the asks refused alone by id, then full path, then line (P5); then the warnings; exit 0 only after `ok`. An ask given is a usage error, exit 2 (P3), its words the engine's.

Each prediction is tied to the stored outputs of `checks.tsv` that run its set: a whole-output one that isn't refused whole holds exactly its `WARN-UNREAD` lines, and every `scenario … refused` line a stored output holds is predicted; all hold (a household's stored blocks drop warnings, so their counts are only listed). Chunk C4 runs refcore's own `check` on every vector (`--confirm`), holds it to the prediction byte for byte and stores its bytes (`source` `check`).

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

`check`'s digest is the prediction's until C4 confirms it. The same run of `perf_digests.py` re-stored P8-10's `I:A:a05_ania@c05-ania` digest, stale since tally 2.2's refcore fixes (8341b008, bec41e6b): it changed its hole lines after the file was last written (2026-11-15).

## Holds

- `fixture/alone/scenario-file`, `fixture/alone/same-id` (`held.tsv`, `m3_check`, **#229**, the coordinator's ruling 2026-11-18): refcore's `run` makes this load check only while answering, so it prints nothing for an ask in a loaded scenario's file that names a world not loaded, where L §6.4 and §8.3 refuse it alone (`GAPS.md`, 2026-11-18). The engine follows the spec. The stored bytes are refcore's `run`'s prediction until C4 stores refcore's `check`.
- **To M4, by name:** every walk ask of the seven changes (load row L:10, P8-L10), and change 5's after-side runs (§ Row 6).

## Keeping master green

As at S, M0, M1 and M2: every `m3_…` target has `test = false` and runs only as `check`'s report until M3 is in `engine/tools/milestones.toml`'s `gating`, which the rules step adds in `build/m3`'s commit before its gate.
