# Milestone M3: what must turn green

The checks M3 (the operations) must turn green, from BUILD-ORDER § M3 ("What M3 builds", "Budgets", "Green") and DONE gates 8, 10 and 12, with the items the run's brief puts in scope (issue #226). Written by T for issue #226 (2026-11-18) on refcore **tally 2.2**, from T's plan approved by the coordinator (picks P2–P9, `tmp/m3/auto-picks.md`; the spec questions filed as #228). The builder reads this page; T changes it.

## The list

"Today" is the engine at master ee1e9b37 (tally 2.2 and the operations document), 2026-11-18. Every `m3_…` target has `test = false` and runs only as `check`'s report until M3's marker commit (as at M1 and M2), so T's commits stay green with no `Logged-Red:` line.

| # | What | Ids, vectors | Run by | Gate | Today |
|---|---|---|---|---|---|
| 1 | `check` in refcore (the spec step's) | none of T's: chunk C4 ties refcore's `check` to the prediction of every vector of rows 2 and 3, byte for byte (`m3/vectors.py --confirm`); a difference is a STOP-REPORT | `m3/vectors.py --confirm` | 10 | refcore prints its usage, exit 2 |
| 2 | `check` over M0's 315 text sets | `m3/check.vectors`, the vectors named by each set's id (C1): 295 `ok` (with load's warnings where the set has them), 11 a base's refusal (row 1 of M0's and `I:M:MP-10`), 9 with refused lines: a scenario refused (`I:B:b10_v3`'s `hr2_v3`, RULE-09's `inpath`, SURF-04's `redefine`) or asks refused alone (`U:law/LAW-13/LAW-13`, `U:rand/RAND-04/RAND-04`, `U:scen/SCEN-02..04`, `I:E1:tx2_cis_explain`: a probe given without its scenario's file) | `m3_check`: one test per M0 corpus group (16 with the parts) | 10 | 0 green: the engine has no `check` (a usage error, exit 2) |
| 3 | `check`'s line kinds the 315 don't reach | `m3/check/fixtures.tsv`, 35 fixtures (`fixture/<kind>/<name>`, § Row 3) | `m3_check`: one test per kind (8) | 10 | 0 green (2 held, § Holds) |
| 4 | `INTERFACE.md`'s `check` | §1's row and §9 | — | — | written (C1) |
| 5 | the run function: the library's one entry and the CLI's; the request, asks and clocks; pure; stores nothing (E §1) | C2: `m3_run` (an id asked twice, ask order, `--ask` with none after it, an unnamed file not read, usage); the library against the CLI once the builder names the entry; `m3/purity.py --engine` at the gate | `m3_run` | 10 | — |
| 6 | the seven changes through OPERATIONS §6.1 (#224's fifth item) | C2: `m3_changes` (17 runs, the law and def cells before and after each change), `m3_check` group `changes` (18: each stage `ok`, and `LOAD-KNOWN` without its advance); the walk asks held to M4 (P8-L10, load row L:10); change 5's after side held to M4 (several currencies) | `m3_changes`, `m3_check` | 10 | the after sides of c1–c4, c6, c7's three law asks green as `I:X:*@cN_fast` |
| 7 | P8-1, a conversation turn | C2: the three vectors of X's change-7 turn (pick P6) and their digests in `perf-digests.tsv`; the rules step's `turn:` recipe | `engine/tools/perf row P8-1` | 12 | the row waits |
| 8 | gate 8 ≥ 90 % | C3: kill vectors from gate 8's run on `build/m3` (44411247874), `fixes/gate8/m3_*` | `m1_fixes` `fixes-gate8` (gating) | 8 | no score for tally 2.2 |
| 9 | #192's standing items, #221's first-group items, `net_worth`'s binding (P7) | C3 | `m3_fixes` | 1, 8 | — |
| 10 | #114's remainder | C6: one refcore run at the longest R7 horizon under 8 GB against the engine | `m2/bank/pins_long.py` | 2 | 2036, 2056 net worth held |

`check`'s report line counts the `m3_…` targets' tests: `milestone-report: M3: 0/25 green (gates once M3 is tagged)` after C1 (`m3_check`'s 24 tests and the one that every group has its test, which passes).

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
| `ok` | `clean`; `unread` (an unread source, input, def and place); `outside` (an outside place nothing names); `scenario` (one that loads); `joint`; `joint-unread` (a member's unread input) | `ok`, then the warnings sorted by code then text; none for the member's (as `run`: GAPS.md, a question) |
| `base` | `dup`; `path` (a path that names nothing); `no-path`; `scenario-read` (a base item reading a scenario's); `beside-scenario`; `member`; `pin` | the one refused line: `LOAD-DUP`, `LOAD-PATH`, `LOAD-HEADER`, `LOAD-SCENARIO`, `LOAD-DUP` alone, `refused: member a: LOAD-UNIT …`, `LOAD-PIN` |
| `scenario` | `one`; `path-order` (`zulu`'s file before `alpha`'s); `over-refused`; `over-nothing`; `cycle`; `deep` (five); `twice` | each refused scenario's line in its file's full-path order; exit 1 |
| `alone` | `plain` (`q_zz` before `q_aa` in the file); `forms` (`diff … against`, `with scenario`, `as if`); `with-scenario`; `scenario-file` and `same-id` (held); `covered` (an ask in a refused scenario's world) | each `<q>: refused: LOAD-SCENARIO …` by id; the covered ask nothing |
| `sections` | `all` | a scenario line, two asks refused alone, four warnings; exit 1 |
| `notclean` | `clock`, `hole`, `pervalue`, `paths` | `ok`: `REQ-CLOCK`, a hole, a per-value refusal, `REQ-PATHS` are `run`'s |
| `paths` | `nested` (a directory whose two refusals print by full path), `household` (#226's `check household/`), `pin-bytes` (`--pin=bytes`) | `LOAD-PARSE … (nest/a/y.tally:4)`; `ok`; `ok` |
| `usage` | `ask` (`check model.tally --ask q`) | exit 2 (P3) |

## Holds

- `fixture/alone/scenario-file`, `fixture/alone/same-id` (`held.tsv`, `m3_check`, #228): refcore's `run` prints nothing for an ask in a loaded scenario's file that names a world not loaded, where L §6.4 and §8.3 refuse it alone (`GAPS.md`, 2026-11-18). The stored bytes are refcore's prediction; C4 decides with refcore's `check`.

## Keeping master green

As at S, M0, M1 and M2: every `m3_…` target has `test = false` and runs only as `check`'s report until M3 is in `engine/tools/milestones.toml`'s `gating`, which the rules step adds in `build/m3`'s commit before its gate.
