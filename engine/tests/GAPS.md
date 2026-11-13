# T's gaps: forms the tests hold back

T's STOP-REPORTs (the guard's pointer, R-79): a form T wrote and ran where refcore and the engine differ outside a
ruled family, held out of the vectors until it is ruled. Each in `engine/BUILD-LOG.md`'s STOP-REPORT layout, newest
last. A held fixture stays written (`codes/m0/held.tsv`, `m0/format/held.tsv`); once ruled, it moves to the vectors.

**Ruled by tally 2.1.7** (T's re-run, 2026-11-13). Every section below up to "STOP-REPORT 2026-11-11" was an item of
the round (the plan's G1–G16) and is ruled: G1 (`adjust/bound-field`, `gifts/lifespan-zz`) by #90; REQ-CLOCK by R5
(#144, `codes/m1/edits.tsv` `rules/clock`); the parenthesised years by R10 (#149) and S10 (#113), `m1/carried/held.tsv`
released; `low` over no days by R6 (#145, `decide/low-empty`, `-from`); the fan-out's two refusals by R5 (#129); the
`$u` clause by B3 (#151); NB6, NB7, NB11 by R6 (G4, G5, G8), NB9 by S5 (#152), NB10 by R2 (G7); the row id by B6
(#135); `argmax … ties by` a hole by R1 (#116); the band figure by R9 (G9); N2, the prefix walk's legs by R1 (G10,
G16); `by unit_price` keyed by places by R5 (G11); end-of-time ranges by B1 (G12); `pinned:` by read order by R7
(G13); a figure in an `if` in a pin by R10 (G14); two currencies through `any` by R5 (G15); WARN-NEGATIVE through the
walk up to a day by R1 (G3). Each item is a vector named by its rule (`m1/fixes/r<n>/`, `b<n>/`, `s<n>/`); the forms
still noted "not stored, not built" stay for the milestones that build them. T's re-run's own findings are the last
section, SR-1 to SR-14.

## STOP-REPORT 2026-11-05: a field of an adjustment's `.adjusts` that no event has loads on the engine; refcore refuses it `LOAD-FIELD`

**Resolved (2026-11-05):** the coordinator's pick P19 (a); the builder's 9d9f5a1 ("M0, G12 (2 of 2)") checks a field read through `.adjusts` by name over the event kinds, and the engine prints refcore's `LOAD-FIELD zz is not a field of events`. `adjust/adjusts-field` moved from `codes/m0/held.tsv` to `edits.tsv`, with the forms 9d9f5a1 made match: a leg and an attribute read through `.adjusts` (`adjust/adjusts-leg`, `adjust/adjusts-attr`), a bound adjustment's `.adjusts.zz` (`adjust/bound-adjusts-field`), `LOAD-FIELD` through an aggregate (`adjust/aggregate-field`). The report as filed:

- **Kind:** other (an engine gap the builder left for #90; no vector holds the form)
- **Where:** L §5.14.3 (`spec/LANGUAGE.md:1432`: an adjustment's `adjusts` is "the instance, a key of its move's kind"), L §2.3 (`spec/LANGUAGE.md:146`: a movement's built-in fields are `day fulfils`), L §8.3 `LOAD-FIELD` `<field> is not a field of <type>`; the held fixture `adjust/adjusts-field` (`codes/m0/held.tsv`); the engine's `movement_field` (`crates/tally/src/load/types.rs`), whose G12 entry ("M0, G12", leftover 2) says "Not taken: … its reading by name over `events` and its `LOAD-FIELD` (`… .adjusts.zz` loads, refcore says `LOAD-FIELD zz is not a field of events`) … for #90".
- **Rule or gate:** DONE gate 5 (each `LOAD-*` code's message, exactly); L §8.3 (a load refusal stops the model before anything runs).
- **What happens:** `codes/m0/adjust/base.tally` with `def ls: date = argmax(1 for a in adjustments over ..).adjusts.zz` after `def net`, run as the load probe `run base.tally --ask zz_m0_probe`: refcore `refused: LOAD-FIELD zz is not a field of events (base.tally:29)`, exit 1; the engine (88c5249) `refused: REQ-ASK no ask zz_m0_probe`, exit 1: it loads. Its twins agree on both: `.adjusts` alone (`adjust/adjusts-type`, `LOAD-UNIT rule base of ls is key events, not date`) and `.adjusts.day` (loads clean in both).
- **What I tried:** nothing on the engine (T writes tests only). I judge refcore right: `zz` is no field of an event, and load sees the type. A vector of refcore's bytes would gate red (M0 gates), so the form is held, as asked.
- **What I ask:** options: **(a) Recommended:** the builder checks a field of `.adjusts` (an events key) as refcore's `kind_attrs` does, with #90 or before the tag; T then moves `adjust/adjusts-field` to `edits.tsv` (one row, `LOAD-FIELD`, form `<field> is not a field of <type>`). **(b)** the coordinator rules that it waits for M1 with #90; T lists it in `MILESTONE-M0.md` as held until then.
- **State:** this step's commit (T, after 88c5249) holds the form in `codes/m0/held.tsv`; no vector depends on it.

## STOP-REPORT 2026-11-05: a bare bound name's field that its kind lacks loads on the engine; refcore refuses it `LOAD-FIELD`

**Ruled for M1 (2026-11-05):** the coordinator's pick P20: #90, with a kind test's narrowing of a bound name. Held in `codes/m0/held.tsv` (`adjust/bound-field`; and, after G13's 0ea61ca, `gifts/lifespan-zz`, `lifespan(e.zz)` over `events`: refcore `LOAD-FIELD zz is not a field of events`, the engine loads it); no vector depends on them.

- **Kind:** other (an engine gap ruled for M1; no vector holds the form)
- **Where:** L §2.3 (`spec/LANGUAGE.md:146`: the built-in fields), L §5.14.3 (`spec/LANGUAGE.md:1432`: an adjustment's attributes `adjusts`, `change`), L §8.3 `LOAD-FIELD` `<field> is not a field of <type>`; the held fixture `adjust/bound-field`; the engine's `movement_field` (`crates/tally/src/load/types.rs`), whose 9d9f5a1 entry says a field whose object is a name stays `Unseen` ("the narrowing, not built, stays #90's").
- **Rule or gate:** DONE gate 5; L §8.3.
- **What happens:** `codes/m0/adjust/base.tally` with `def ls: date = argmax(1 for a in adjustments over .. where a.zz)` after `def net`, as the load probe: refcore `refused: LOAD-FIELD zz is not a field of adjustments (base.tally:29)`, exit 1; the engine (9d9f5a1) passes `a.zz` and refuses the def's type, `refused: LOAD-UNIT rule base of ls is key adjustments, not date (base.tally:29)`, exit 1. Its twin `where a.adjusts.zz` agrees on both (`adjust/bound-adjusts-field`, `LOAD-FIELD zz is not a field of events`).
- **What I tried:** nothing on the engine. refcore is right by L §8.3 (load sees `a`'s kind and that it has no `zz`).
- **What I ask:** nothing now: P20 rules it for M1 (#90). Then T moves `adjust/bound-field` to `edits.tsv`.
- **State:** held, as ruled.

## STOP-REPORT 2026-11-05: a unit past the 64-bit powers, a value past the digit cap, or an axis count past it, where the engine's capacity stop differs from refcore

**Ruled for #89 (2026-11-05):** the coordinator's ruling: the capacity limits the spec doesn't state (the builder's "M0 built" entry) go to the next spec round, which states the limits and their codes; until then the engine stops with its capacity stop ("not built") or another code, where refcore, whose Python numbers have no limit, loads or refuses otherwise. Held in `codes/m0/held.tsv` (7 rows, `basic/s1-*`); no vector depends on them.

- **Kind:** other (the spec is silent on capacity; ruled for #89)
- **Where:** L §3.2 (units and their powers), §7.1 (a figure in a rule), L §8.3 (`LOAD-UNIT`), L §8.4 (`RUN-DIGITS`, the 4 096-digit cap); the engine's `unit_stop` (`crates/tally/src/load/types.rs`) and `check_forks` (`load/pins.rs`), G13's entry (`engine/BUILD-LOG.md`, "M0, G13", S1 and S2).
- **Rule or gate:** DONE gate 5; L §8.3.
- **What happens:** each one edit of `codes/m0/basic/base.tally`, as the load probe (`run base.tally --ask zz_m0_probe`; T's `tmp/t-m0/gate/probe-g13.txt`):

  | Held fixture | The edit | refcore | The engine (0ea61ca) |
  |---|---|---|---|
  | `basic/s1-def-pct` | `def y: num = 1 %^3000` | `refused: REQ-ASK no ask zz_m0_probe` (loads) | exit 69, `tally run: not implemented` |
  | `basic/s1-def-pow` | `def y: num = 1 PLN^9223372036854775807*PLN` | `LOAD-UNIT rule base of y is num PLN^9223372036854775808, not num` | exit 69, `not implemented` |
  | `basic/s1-def-pow-neg` | `def y: num = 1 PLN^-9223372036854775808/PLN` | `LOAD-UNIT rule base of y is num 1/PLN^9223372036854775809, not num` | `LOAD-PARSE cannot parse: a power too large` |
  | `basic/s1-input-pow` | `input y: num PLN^9223372036854775807*PLN` | loads | `RUN-DOMAIN outside its operation's domain` |
  | `basic/s1-input-pct` | `input y: num %^3000` | loads | `RUN-DIGITS a number past 4096 digits` |
  | `basic/s1-row-pct` | `rate[q2] = 1 %^3000` | `RUN-DIGITS a value passes 4 096 digits; round where the law rounds` | the same, with `(base.tally:43)` |
  | `basic/s1-row-pow` | `rate[q2] = 1 PLN^9223372036854775807*PLN` | `LOAD-UNIT 1 and num have different units` | `RUN-DOMAIN outside its operation's domain` |

  **Not held, too large to store as a row:** an ask's axis combinations past the 4 096-digit cap. It takes 8 585 axes or more of three options (3^8585 has 4 097 digits; three options per axis give the shortest text), some 450 KB of text: G13's `tmp/m0/g12/s2.py`, `text(8586)`. G13's run: refcore `refused: LOAD-FORKS ask q reaches <the count, 4 097 digits> axis combinations; the cap is 64` (G13 summarised the line); the engine "not built" ("axis combinations past 4096 digits"). The counts below the cap are vectors (`basic/forks-wide`, 3^33; `basic/forks-past-64`, 3^70).
- **What I tried:** nothing on the engine. Where refcore loads a unit power past 64 bits or a value past the cap, the spec's meaning is unsettled (L §8.4 caps a value's digits at a key, at run; no text states a unit's powers or an axis count's size), so neither side is the expected value.
- **What I ask:** nothing now: #89's round states each limit and its code; T then moves the rows to `edits.tsv` (and writes the axis form by a generator if it is a vector).
- **State:** held, as ruled.

## STOP-REPORT 2026-11-08: an ask's `as known` after the model's `on` is answered by refcore; E §11 refuses the request `REQ-CLOCK`

- **Kind:** refcore and the spec disagree (M1's code fixtures, issue #108, chunk C4; no vector holds the form)
- **Where:** E §11 (`spec/ENGINE.md:498`: `REQ-CLOCK` | `as known` after `on` | the request is refused); L §5.18 (`as known D` states the line's cut K, in place of the model's `on`); `codes/m1/held.tsv` `rules/clock`.
- **Rule or gate:** DONE gate 5 (each code has its fixture; `tools/xref_check.py --gate M1` stays red on `REQ-CLOCK` alone).
- **What happens:** `codes/m1/rules/base.tally` (on 2026-03-01) with `ask q_due: due as known 2026-06-01`, run `run base.tally --ask q_due`: refcore answers `q_due: 2026-02-13`, the register's `cut 2026-06-01`, exit 0 (`ask.py:383`: the cut is set to the ask's date, never checked against `on`). E §11 refuses the request. E §11 also gives `REQ-CLOCK` no message: no form says what the refusal line prints after the code.
- **What I tried:** nothing on refcore or the engine. I judge the spec clear on the refusal and silent on its words.
- **What I ask:** **(a) Recommended:** the spec states `REQ-CLOCK`'s message (e.g. `as known <D> is after the model's on <on>`) and that it is the run's one result, as `REQ-ASK`'s row does; refcore is fixed at the next spec round; T moves `rules/clock` to `edits.tsv` with refcore's bytes then, and the engine is held to the spec's line until then "for #n". (b) `as known` after `on` is allowed (the cut moves past what the reality knows) and E §11's row is struck.
- **State:** held in `codes/m1/held.tsv`; no vector depends on it.

## STOP-REPORT 2026-11-08: refcore refuses or misreads a parenthesised year literal in six of P6's nine places; an `if` of year literals at a year key

- **Kind:** refcore and the spec disagree (M1's carried item 5 and #97's N1, issue #108, chunk C5; no vector holds the forms)
- **Where:** L §2.1 (parentheses don't change a literal); the pre-M1 run's pick P6 on #97 ("(2026) is the year wherever a bare 2026 is", built in M1) and the M1 run's P12 ("every parenthesised literal"); `m1/carried/years/` (a base and one file per form); `m1/carried/held.tsv` (each form, its files and refcore's line).
- **Rule or gate:** DONE gate 1 (the M1 brief's carried item 5); L §2.1.
- **What happens:** with the base `model.tally` beside each form's file, refcore 2.1.6 answers three forms as their bare twins do (`year_of(start_day) == (2026)` true, `wage[(2026-03)]`, `bal(konto, (2026-01-05))`: stored, `carried-5`) and refuses six: `rate[(2026)]` `LOAD-KEY (2026) (year4) is not a key of ys`; `[(2026), 2027]`, `governs (2026)` and a row `extra[(2026)]` `LOAD-PARSE expected a literal, got (`; `day_of((2026), 12-31)` `LOAD-UNIT day_of is 2 026, not a period`; `(2026)..2027` `LOAD-DOMAIN range of quantities … has no step`. And #97's N1 form `rate[if year_of(start_day) == 2026 then 2026 else 2027]`: `LOAD-KEY … (year4) is not a key of ys`, where `rate[2026 + 1 year]` and `[2026, 2027]` answer.
- **What I tried:** nothing on refcore or the engine. I judge the six refusals refcore's: P6 and P12 read L §2.1 as making each the bare literal's meaning. The `if` is less clear: L doesn't say an `if`'s arms take the key type the read wants.
- **What I ask:** **(a) Recommended:** refcore is fixed at the next spec round for the six (refcore's half of P6), and the spec says an `if`'s arms take the type of the place it stands in (or that it doesn't, and N1's `if` is struck); until then the forms stay in `m1/carried/held.tsv`, and the builder builds P6 by the spec in G1 without their vectors. (b) The six are stored now with their bare twin's output as the expected bytes (refcore's run of the same text without the parentheses), so G1 is held to L §2.1 at once; a T script change, every byte still refcore's.
- **State:** held out of `m1/vectors.tsv`; the three refcore answers are its vectors (`carried/years/p_compare`, `p_month`, `p_day`).

## STOP-REPORT 2026-11-10: refcore crashes on `low` over a window whose last day is before its first; L §5.14.1 refuses it `RUN-EMPTY` (ruled: P53, #126)

- **Kind:** refcore and the spec disagree; already ruled (the M1 coordinator's pick P53 on G8's review B1), so a record of the held fixtures, not a new question (M1, chunk C8, issue #108)
- **Where:** L §5.14.1 (`low(x, a .. b)` is `min(bal(x, d) … for d in a .. b step 1 day)`), §2.5 (a stepped range holds none here), §5.11 and §8.4 (`min` over no keys: `RUN-EMPTY`); `codes/m1/held.tsv` `decide/low-empty`, `decide/low-empty-from`.
- **What happens:** `codes/m1/decide/base.tally` with `ask q_decide: low(konto, 2026-03-31 .. 2026-03-01)`, with and without `from 2026-03-01`: refcore `q_decide: refused: REFCORE-NOT-YET an input this interpreter can't handle (IndexError: list index out of range)`; the engine `q_decide: refused: RUN-EMPTY low over no keys` since G8's fix (678d88a). Refcore prints no bytes a vector could keep (`m1/vectors.py` refuses `REFCORE-NOT-YET`), so the review's `z_rev` and `z_revf` are held here, not in `held.tsv`.
- **What I ask:** nothing new: refcore's `m_e_low` is fixed at M2's round (#126); T then moves both rows to `edits.tsv` with refcore's bytes.
- **State:** held in `codes/m1/held.tsv`; no vector depends on it.

## STOP-REPORT 2026-11-10: P57's equivalence target needs `tally` as a dependency of `tally-runner`, one line of `engine/Cargo.lock`, which is not T's file

- **Kind:** a rule blocks a correct design (R-79: the tests role writes `engine/tests/` only), M1 chunk C8 part 4, issue #108
- **Where:** pick P57 (T writes the fence's equivalence check as its own runner test, every ask of T's corpora through `tally::run_with_lows` with `Lows::Tree` and `Lows::Walk`); `tests/runner/Cargo.toml` depends on `tally-cli` alone, and `tally-cli` doesn't re-export `tally`, so the runner can't name `run_with_lows`; adding `tally = { workspace = true }` to the runner changes `engine/Cargo.lock`'s `tally-runner` entry (`dependencies = ["tally", "tally-cli"]`), and `check`'s `--locked` build refuses a stale lock.
- **What I did:** wrote the target, `runner/milestone/m1_lows.rs` (4 tests: the kernel corpus's 312 records, `m1.corpus`'s 648, every `fixes-` vector of `m1/vectors.refcore`, and F5's seven realities copied to `m1/lows/`, each ask run with the tree and with the twin and the lines compared), committed but not yet a `[[test]]` target; proved it in a scratch copy of `engine/` outside the repository (`/tmp/claude-t8`), the dependency and the target added there and cargo writing that copy's lock: 4 of 4 green in 48 s (debug). Nothing outside `engine/tests/` was written in the repository.
- **What I ask:** **(a) Recommended:** the coordinator lets T's next commit carry the lock's one generated line beside `tests/runner/Cargo.toml`'s two changes (the dependency and the `[[test]] m1_lows` entry, `test = false`), or the rules role commits the three lines; nothing else changes. (b) `tally-cli` re-exports the library (`pub use tally;`, a builder's line under R-68), and the runner calls `tally_cli::tally::run_with_lows` with no new dependency edge.
- **State:** `m1_lows.rs` and `m1/lows/` are in the tree; `MILESTONE-M1.md` row 9.

## STOP-REPORT 2026-11-10: which of two refusals an ask fanned out over an axis and a `for` meets first: refcore tuple by tuple, the engine combination by combination; the spec doesn't say

- **Kind:** the spec seems silent (G7 review NB3, probed in M1 chunk C8, issue #108)
- **Where:** L §5.5 (an ask evaluates every combination of the axes its evaluation reads), §5.11 (a `for`'s tuples in key order); refcore `ask.py` `binds_over_axes` (each tuple, then its combinations); the engine's `ask/axes.rs` (each combination, then its tuples, so the combination's cells are shared).
- **What happens:** G8's `forms` model (a place `vault` priced by a clause keyed by a kind holding neither it nor its unit; a place `gfund` priced per gold) with `index ks = [k1, k2]` and `axis ra` (`r1`, `r2`), and `ask q_order: (if (k == k1 and ra == r2) then value(vault, 2026-03-31) in PLN else if (k == k2 and ra == r1) then value(gfund, 2026-03-31) in PLN else 0 PLN) for k in ks`: both refusals are load codes the run meets. Refcore: `refused: LOAD-KEY vault is priced by bprice, keyed by boxes, which holds neither vault nor its unit` (k1 under r2 first); the engine: `refused: LOAD-UNIT value(gfund) in PLN: gfund is priced in PLN/gold, not PLN/etf` (k2 under r1 first). With the cells swapped (`q_twin`) both give `LOAD-KEY`. Texts: `tmp/m1/t/c8/nb3.tally`, `nb3_a.tally` (local).
- **What I ask:** a ruling for the round. **(a) Recommended:** both are load's refusals (L §8.4), so the order is load's, the first in the text, whichever cell the run meets first; until load finds them, no vector pins the order. (b) Tuples first, as refcore, written in §5.5; the engine follows. (c) Combinations first, as the engine.
- **State:** no vector; G7 review NB2 (two distinct values that print alike, grouped by text where refcore groups by `rkey`) has no text that reaches it: one ask's values have one type, exact rationals print in full (`1/3` and `0.333…3` print apart) and one index's keys print apart.

## STOP-REPORT 2026-11-10: a place priced by a clause keyed by a kind of units is refused `LOAD-UNIT … priced in PLN/$u` by refcore and the engine; L §3.2 reads the key's unit

- **Kind:** refcore and the engine agree, and both seem to differ from the spec (met in M1 chunk C8, writing an input for MB3's `walk/value.rs` `unit_in`, issue #108)
- **Where:** L §3.2 (a per-unit price of a kind of units, `input fxs[u in currencies]: num PLN/u`, read at a key takes that key's unit), §5.10 (a place priced per unit by its kind's clause); G8's 57d25e0 fixed the same `$u` for a plain read and a `last` (`eval/read.rs` `row_converted`), not for the clause's unit check (`walk/value.rs` `priced_position`); refcore's `m_priced_read`.
- **What happens:** G8's `keyed` model (`fxs[EUR] = 4.25 PLN/EUR`) with `kind fxheld: place one min 0 priced fxs`, `place eurcash: fxheld EUR`, `opening[eurcash] = 100.00 EUR` and `ask uk_value: value(eurcash, 2026-03-31) in PLN`: both refuse the file `LOAD-UNIT value(eurcash) in PLN: eurcash is priced in PLN/$u, not PLN/EUR` (exit 1). By L §3.2 the clause's price at eurcash's unit is `fxs[EUR]`, a `num PLN/EUR`, so 425 PLN. Text: `tmp/m1/t/c8/unitkind.tally` (local).
- **What I ask:** **(a) Recommended:** the clause takes its key's unit as a read does; refcore fixed at the round, the engine at G14; T then stores the vector with refcore's bytes. (b) A clause keyed by a kind of units is refused at load (`LOAD-KEY`), said in §5.10.
- **State:** no vector (it would pin a refusal the spec doesn't give). MB3's three `unit_in` mutants (`unit_in` → false, `&&` → `||`, `priced_read`'s `unit_in` guard → false) need this form answered, so they stay missed.

## For the round 2026-11-10: five decision-cycle readings G10's review found shared by refcore and the engine, or refcore's alone, that the spec seems to refuse; no issue rules them

- **Kind:** refcore and the engine agree (or refcore answers where the engine stops), and both seem to differ from the spec (G10 review non-blocking 6, 7, 9, 10, 11; M1 chunk C9 part 3, issue #108). No vector stores them: each would pin a reading the spec doesn't give. The review's texts are `tmp/m1/g10-review/p/` and `agent/` (local).
- **NB6, `next(K, d)` in a decision over decision members, with `reads K`** (`p/c9`, `c11`): both refuse `LOAD-READS … declares reads pays and never reads it`. L §5.14.1 makes `next` a binding, `first(m in K over d ..)`, which a decision declares by its kind. **(a) Recommended:** `next` reads K; the declaration loads.
- **NB7, a plan with no `reads` reading `next(K, d)` over decision movements** (`p/c4`, `c5`): refcore `HOLE-NOTYET` though p1 exists; the engine stops not implemented (exit 69). By L §5.14.1 a plan reading a binding over decisions is `LOAD-READS`. **(a) Recommended:** `LOAD-READS`, refcore fixed at the round (with #121's), the engine at G14 or M2.
- **NB9, a written order's transitive reach** (`p/a5`: `save < spend < top` with `save` and `top` reading each other's place; `a9`: a 3-ring with `after` on two links): both refuse `RUN-ORDER`. L §5.14.2: a cycle is ordered when its text "states `before m` or `after m` on one of them". **(a) Recommended:** a chain of written orders that orders every pair is enough; written in §5.14.2. (b) Each mutual pair needs its own clause, as both engines read it now.
- **NB10, keyed instances with no `ties by`** (`p/b2`: `pull[k in ks]` on one day, each reading `bal(konto)`): both print `flows(pull[k1]) = 400.00`, `flows(pull[k2]) = 100.00`, a silent key order, while `bal(konto)` is `RUN-TIE`; `p/b4` (an order-free amount) is refused `RUN-TIE … that changes the result`. L §5.14.2 refuses a tie that changes a result, so each flow is `RUN-TIE` too. **(a) Recommended:** every read of a tied member's result is refused, as `bal` is.
- **NB11, `gap` read in a plan's body during the main walk** (`agent/gapmemo/m`): both read a half-built walk and give `HOLE-NOREADING`. L §5.19 is silent on a plan's `gap` read mid-walk. **(a) Recommended:** a plan's `gap` reads the place's walk up to the plan's day, as a decision's does (E §4.3 step 1).
- **State:** no vectors for these eight texts; the review's other 67 groups are G10's vectors (`m1/fixes/g10/`, five held for #121).

## For the round 2026-11-10: three G11–G13 differences no issue rules, so no vector stores them

- **Kind:** refcore and the engine differ, and no ruling says which follows the spec (M1 chunk C9 part 5, issue #108). The texts are G11's, G12's and G13's probes (`tmp/m1/g11-review/`, `g12-review/`, `g13/`, local); every other ask of theirs is a vector (`m1/fixes/g11/` to `g13/`, the differing ones held for #100, #125, #132, #133, #134).
- **A row's stable id with its instant** (G12 review B1, fixed by d1fd733): two readings of one place on one day in one version, at the start and the end. The engine prints the row as `bank:stated[konto] 2026-02-20 end = 395.00 PLN, 2026-02-20 end`, refcore as `bank:stated[konto] = 395.00 PLN, 2026-02-20 end`; L §2.5 says two rows of one target in one version need distinct ids. Asks: `p/b1/b1.tally` `a`, `b1x.tally` `a`, `b`, `c`; `agent/lw/lw.tally` `e4`, `e5`, `e6`, `e13`, `l2`. B1's fix, the end-of-day reading's own instant on the `from:` line, agrees on both. **(a) Recommended:** an issue ruling the engine's id, refcore fixed at the round; T then holds these nine asks for it.
- **`argmax … ties by p[k]` whose tie a hole breaks** (G11 review non-blocking 13, `agent/mx.tally` `e2`, `v`): the engine gives a hole (`?"p at q3: no value"`), refcore `q3`. By L §5.14.2 a `ties by` key that is a hole can't order the tie. **(a) Recommended:** the hole, as the engine.
- **A band row finer than its minor unit** (G13, `sites/pz3/f1.tally`): both refuse `LOAD-PRECISION`; refcore prints the figure as a Python fraction, `601/200 is finer than the minor unit`, the engine as written, `3.005`. **(a) Recommended:** the figure as written, as #100 rules for a key.
- **Not stored, not built (G14 or M2):** `explain` of a def reading a move by name (`p/mr*.tally` `e5`, `mr2` `x6`), of `low … with uncertain` without `from` (`lw` `l1`), of a fill and an event count (`agent/mv2`, `mvs`, `p/b` `enn`), bands over dates or keys (`agent/b1`), `pow` with no `to` and an uncertain interval (`g11/un`), an extreme over a recurrence past the counted depth (`ai2` `e_pick_a`, refcore's not-yet), money ranges finer than their minor unit and rows keyed by non-integer ranges (`g13/pz/p4`, refcore's not-yet), and the 26 sites' parts over a leg of unit `any` (`sites/pz3` `x*`, `y*`, `pz2` `k4`).

## For the round 2026-11-11: G13's and G14's fix probes where refcore and the engine differ and no issue rules it

- **Kind:** refcore and the engine differ, and no ruling says which (M1 chunk C10 part 2, issue #108). The texts are the reviews' and builders' probes (`tmp/m1/g13-review/`, `g13-fix-review/`, `g14/`, `g14-review/`, local); every other ask is a vector (`m1/fixes/g13/`, `g14/`; held for #120, #121, #123, #130, #134, #135), or, where refcore crashes, the spec's answer in `m1/spec.tsv` (31, its section named, `lifespan` of a movement among them for #136).
- **G14 review N2, a false invariant 5 on refcore** (`agent/inv5/b.tally`, `c.tally` `t3`): refcore refuses `ENGINE-INVARIANT a booked plan amount changed: pay[2027-02] booked at 2027-01-01 differs between the cuts 2027-01-01 and 2027-01-01 (E §9 invariant 5)`; the engine, keying a partial walk's bookings by its limit (99c2c92), gives `RUN-PLACE acct is -1.00 PLN …` and `0.00 PLN`. E §9 item 5 compares one instance between two walks of one cut; a walk up to a day isn't the whole walk. **(a) Recommended:** an issue for refcore's round, the engine as built; T then holds the two asks. The same on T's `fixes/g14/t_mb5_pl.tally` (C10, MB5's prefix walk, a leg the walk refuses `RUN-PLACE`), where refcore's answer depends on the asks' order: `t` then `t2` (the whole file) refuses `t2` with that false invariant 5, `t2` then `t` answers `t (m = 2027-04): 935.00 PLN` (the refused leg counted); each ask alone, and `t` then `t3`, give the engine's `RUN-PLACE sav is -5.00 PLN …` in every order. Stored: `t_mb5_pl_t_t3`, `t_mb5_pl_t2`; the whole file waits with N2.
- **G14 review N7, `by unit_price` keyed by places** (`p130/one/up_pln`): the engine `RUN-KEY etf is not a key of funds`, refcore the hole `unit_price at etf: no value`. **(a) Recommended:** refused at load, L §5.10 (`by p` is a `num CUR/u` keyed by units).
- **End-of-time day ranges** (`g14/rh/m.tally` `a13`, `a14`, `a14b`; `g14-review/rh/h.tally` `b4`, `b7`): refcore steps the range to test one key and refuses `RUN-LIMIT a day passes 9999-12-31`; the engine tests the key by its offset (abec330) and answers, or refuses at load a literal key off the step (`b7`, `LOAD-KEY 9999-12-29 is not a value of …`, L §2.5). **(a) Recommended:** the engine's.
- **A `pinned:` line dropped by read order** (`g14/pt/memo.tally` `t7`, `m7.tally` `t7`, `u3`): refcore drops `pinned: a = 3` when `s[5]` is read first; the engine prints it both ways, as L §8.2 asks. **(a) Recommended:** the engine's.
- **A figure inside an `if` in an ask's pin** (G14 fix's load gap; M1 chunk C10 part 4): `ask g1: f[q2] with v[q1] = (if true then 3 else 4)` and `g3: … (if v[q2] > 0 then 3 else 4)` (texts `tmp/m1/t/c10/pw/pt_base.tally` with `pg1.tally`, `pg3.tally`, local; G14 review's `agent/pt/a.tally` `x3`). Refcore refuses the file `LOAD-LITERAL figure 3 in an ask; make it a row`, though it takes the same figure as the pin's whole value (`g2: … with v[q1] = 3`, stored, `fixes/g14/t_pin_*`): its pin scope allows literals (`check.py` `pin`, "an ask's pins hold literals") and the `if`'s branches lose that. The engine loads both and answers. L §7.1 "An ask allows every literal a def allows, plus its own: … its pins …" gives the engine's reading: a pin's value is the question's, whatever its form. **(a) Recommended:** an issue for refcore's round (its `if` keeps the ask pin's allowance); T then stores `g1`, `g3` and holds them for it.
- **Money of another currency through a leg of unit `any`, summed into a PLN def** (`tmp/m1/t/c10/pw/c.tally`, local; MB5's prefix walk, input c): refcore refuses each read `RUN-DOMAIN 1 000.00 PLN and 20.00 EUR are in different keys PLN and EUR of currencies`; the engine stops the run `LOAD-UNIT 1 000.00 PLN and 20.00 EUR have different units` (met at run, as P66's forms). L §3.2 types money by its currency, and §8.4 refuses at load what the types show; through `any` only the run shows it. **(a) Recommended:** `LOAD-UNIT` at run, as P66 rules for a day over a period; refcore at the round. With a day over a year in place of EUR (`c` as first written) refcore computes and refuses its own units later, the engine refuses `LOAD-UNIT … day is a dimension of its own` (#134).
- **A leg the walk refuses, counted by refcore through the walk up to its day** (`fixes/g14/t_mb5_plc.tally` `t`; MB5's prefix walk): `op[2027-04]`'s leg takes `sav` (`min 0`) to -32.00 PLN, so the walk refuses it `RUN-PLACE`. Read directly (`op[2027-04].cash.amount`, `bal(sav, 2027-04-30)`) both refuse it; read inside `p[2027-04]`, which sums the kind's legs over its month and is paid on that day, refcore counts it (`953.00 PLN`) and the engine refuses `RUN-PLACE sav is -32.00 PLN …`. L §5.14 (a leg the walk refused is refused) and E §4.2 step 5 give the engine's; refcore misses the refusal of a leg in the month of the cell the read is inside, the day its walk up to a day stops at: so too `tmp/m1/t/c10/pw/plm.tally` (local; `t3`, `p[2027-03] as known 2026-12-31`, whose version takes `sav` to -17.00 PLN on 2027-03-01: refcore `968.00 PLN`, the engine `RUN-PLACE`, in either order beside `ta`). In `t_mb5_pl`, whose refused leg is February's and read on the main walk's own path, refcore refuses as the engine does. `plm` is also the one input that tells `prefix_walk`'s kept walk by its cut (`reads.rs:567`, `==` → `!=`, which would print refcore's `968.00`), so that mutant stays missed until the round. **(a) Recommended:** an issue for refcore's round, the engine as built; T then stores `t` and `plm`'s `t3` and holds them. Stored now: `ta` then `t3` (`t_mb5_plc_ta_t3`), which agree.

## STOP-REPORT 2026-11-11: `WARN-NEGATIVE` is missing when the answer comes through a walk up to a day (G14 review N3), a wrong output for the builder

- **Kind:** the engine's output differs from refcore's and from the spec (M1 chunk C10 part 5, issue #108); the vector is red, `fixes/g14/r_agent_inv5_n` (`m1_fixes_the_open_findings`, group `fixes-open`, apart from G14's so that group counts under `mutants-local --next`).
- **Where:** E §4.2 step 5 (each walked day checks its places: an account or cash place whose end-of-day balance goes below zero prints `WARN-NEGATIVE` on the first day of each stretch, tested only where the balance is known); L §8.5 (a run's warnings print beside its results, sorted); L §5.10's table (`accounts` may go below zero, with `WARN-NEGATIVE`).
- **What happens:** the review's `agent/inv5/n.tally` (F4g's form: `p[m]` sums the event kind `ovs`'s legs over its month, `pay[m]` pays `p[m]` out of `acct`, `op[m]` pays 1 PLN out of it). Its asks read only `p[m]`, so the engine walks `acct` only up to each read's day (the prefix walk). `acct` closes 2027-01-01 at -1.00 PLN. Refcore prints `warning: WARN-NEGATIVE acct is below zero from 2027-01-01` after the answers; the engine prints the same answers and no warning. Where an ask reads the balance itself, both warn.
- **Why the spec asks for it:** the walk up to a day is the walk's own fold, stopped early (E §4.2, §4.3); its step 5 checks each day it walks, and the balance there is known. A warning doesn't depend on which read made the walk run.
- **What I ask:** the builder prints the warnings of the days a walk up to a day walked, as the whole walk does, once per run (L §8.5); T's vector turns green.
- **State:** the vector is in the store, red; no hold.
- **Not stored, not built (the gate or M2):** pins under `as if` naming a binding inside an aggregate (`agent/pt/x4`–`x6`, N5), a pin naming two bindings in one part (`pt/two.tally` `p3`), an ask's `law`, `as known` and `with` pins naming its own `for` (`agent/pt/w.tally`, `x1`, `x2`, `two` `p4`; refcore crashes), rows keyed by a range of fractions (`g13-review/p/sm*`, `agent/w/rq`, `x/rg8`), a scenario's parts beside `k23` (`g13-review/s/`); a leg read by name of a plan's instance a later version made exist (E §5 step 1; `fixes/g14/t_mb5_lx.tally` `x1`: refcore `0.00 PLN`, the engine stops the run "not implemented", so `leg_field`'s five mutants on that path, `members.rs:731` `||`, `:740`, `:744` ×3, have no green input yet).

## STOP-REPORTs of tally 2.1.7's re-run (T, 2026-11-13): refcore's answer needs an exception to a rule's text

Each: the smallest text, the rule's words, refcore's answer, what T stored. Where a line moved and nothing explains
it, T left it at tally 2.1.6's bytes (kept in the store; `held.tsv` where the rest of the vector moved). Texts named
`out/t217/…` are T's probes, re-made from the row's description (git-ignored); the rest are tracked fixtures.

- **SR-1 (R2, #141):** `residuals konto` is refused `RUN-TIE lean has a tie …` by instances of `lean` whose legs never
  touch konto (`kernel` `g5/order` q_res_jan, q_res_feb). R2: "a refused movement is refused for every read of it,
  exactly where a hole movement would be a hole"; the same instance refused by `RUN-DIV`, or a hole, leaves the ledger
  answered. Kept at 2.1.6's bytes: `g5/order`, m1 `fixes/g12/pm_order_order`, `pm4_g5_…`, `pm7_…`, `pm8_g5_…`,
  `r_p_ord`; held for #141 with refcore's bytes (their other lines moved by R5), nine: `prr_*` (four), `r_agent_ord_order`,
  `r_pb_*-order_order` (four).
- **SR-2 (B1, #150):** `kernel` `g14/day_range_end`: `r_past`, `7262-02-02 .. 9999-12-31 step 1 day`, 1 000 001 keys,
  is answered; L §7.6 item 2 refuses more than 1 000 000 keys, as refcore does for `r_count` (the same count ending
  9999-12-30). `r_edge` and `r_two` answer where L §7.6 item 1 refuses "a range's next key" past 9999-12-31. Kept.
- **SR-3 (R4, #143, F11a):** `value(wunits[oki], d1) in PLN by fp` on `place wunits[w in wrapper]: holdings fu priced
  fp` prints `LOAD-PRICE value(wunits[ike]) by fp: wunits[ike]'s price already gives its fu`: it names a place the
  text doesn't read (R9: a value printed as written). The refusal itself is R4's; stored as refcore gives it.
- **SR-4 (R4, F11a, the spec's texts):** `engine/spec/amendments/household/law/pl/wrappers.tally:45`, its
  `asks/core.tally` and `engine/spec/montecarlo/household-bands/scenarios/surprises.tally` write `value(wunits[w], d)
  in PLN by etf_price[month_of(d)]` on a place `priced etf_price`: refcore refuses the household `LOAD-PRICE`, while
  `spec/tools/loadcheck.py` calls it clean. The fix is T-1's: drop the `by`. `m0/loadcheck` kept at 2.1.6's bytes;
  T's 26 copies (`integration/amendments/cut11`, `real11`) were repaired (pick T-1).
- **SR-5 (R3, #142, minor):** `max(w_in, d) == d` undecided prints `?"date within window(…) not precise enough for
  max(w_in, d) == d"` with no `(w_in)`, where `add_days(w_in, 1) == d` carries it: L §8.1, "with the item and keys each
  came from". The form left out of `r3/reach_max`.
- **SR-6 (R7, #146):** `tot as if {r[k1] = 5} with r[k1] = 9` is 6 (R4 right), but its register lists both `pinned:
  r[k1] = 5` and `pinned: r[k1] = 9`; R7: a pin is listed "exactly when a read in its set gives it". `m1/fixes/r4/reach`
  ask `p_inner`, left out of `r4/reach_asif`.
- **SR-7 (R6, #145):** `next(transfers, d1)` before any member prints `?"m: no transfers yet, …"`, naming the
  definition's bound `m`, not the form as written (R6: "their messages naming the form as written"); its RUN-TIE
  prints `next(transfers, 2026-03-02)`, the day's value for the text's `d1` (stored as refcore gives it). `m1/fixes/r6/
  reach` asks `n_next`, `n_first` left out.
- **SR-8 (R7, #146):** `bal(konto, d9)` where a move's `when wanted` is false, `wanted` an `assumed` row, lists no
  `assumed:`; with it true it does. L §8.2: the set holds "the reads that decided that a movement doesn't count (a
  false when …)". `m1/fixes/r7/reach` ask `a_when` left out.
- **SR-9 (R3, #142, a crash):** `count(d in d1 .. d2 step age(w, d2) * 1 month)`, w a window: `REFCORE-NOT-YET …
  AttributeError: 'Iv' object has no attribute 'q'`; R3 makes a step that needs one value `HOLE-IMPRECISE`.
  `fixes/g13/r_agent_w_model_with_r_agent_w_g4`, `_g5`, `rf_w_model_with_rf_w_w10` kept at 2.1.6's bytes.
- **SR-10 (R8, #147):** `explain f with ax = lo`, ax an axis the ask pins, prints `REFCORE-NOT-YET … axis ax read
  outside the axes the ask reaches`; R8: every ask form fans out over the axes it reads. `m1/fixes/r8/reach` ask
  `e_pinned` left out of `r8/reach_explain`.
- **SR-11 (budget):** `fixes/g13/r_agent_w_model_with_r_agent_w_h2`, `count(d in 2026-01-01 .. 2046-01-01 step 1 day
  where t6[d] == n5)`, now runs past refcore's 90 s; tally 2.1.6's ran in budget. Kept at 2.1.6's bytes.
- **SR-12 (B6, #135):** `px = 4 at 2026-04-01`, a `num` reading at a bare day, one of two px rows of a version, prints
  its row id `card_2026:px 2026-04-01 None` (`explain`, `inputs-of`); L §2.5 gives `version:target` then its instant,
  the day, and `None` is no printed form (R9). `fixes/g11/r_p_b_elp`, `_ilp` kept at 2.1.6's bytes.
- **SR-13 (R10, #149):** `first_q == (2026-04)`, first_q a key of a month index `by 3`, is false on refcore; L §2.5
  Keys: the position reaches through parentheses, and the bare twin `first_q != 2026-04` is false. `fixes/g7/
  by3c_a_paren` stays held, re-pointed from #113 to #149.
- **SR-14 (budget):** `I:B:B-13~B3` and `~B4` run past the 90 s budget on tally 2.1.7's refcore (2.1.6: 45.1 s,
  49.7 s); their siblings take 70.6 s (`~B1`), 88.3 s (`~B2`) and 89.4 s (`~B5`) at two jobs, from 43–50 s. Their
  baselines are kept; a capability copy (TESTING §4) or a load row is the user's call.
- **Coverage lost, not a fault:** an ask's `as known` after the model's `on` now refuses the whole request (E §11
  `REQ-CLOCK`, #112): `I:E1:tx2_*` (six asks of `baselines/e1-02-B16.out`) and `I:S4:S4-03P~pinned` (six) print the
  refusal alone, stored as refcore gives it. A text repair that keeps their asks answered is for the round's owner.
