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
