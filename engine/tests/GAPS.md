# T's gaps: forms the tests hold back

T's STOP-REPORTs (the guard's pointer, R-79): a form T wrote and ran where refcore and the engine differ outside a
ruled family, held out of the vectors until it is ruled. Each in `engine/BUILD-LOG.md`'s STOP-REPORT layout, newest
last. A held fixture stays written (`codes/m0/held.tsv`, `m0/format/held.tsv`); once ruled, it moves to the vectors.

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
