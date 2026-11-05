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

**Ruled for M1 (2026-11-05):** the coordinator's pick P20: #90, with a kind test's narrowing of a bound name. Held in `codes/m0/held.tsv` (`adjust/bound-field`); no vector depends on it.

- **Kind:** other (an engine gap ruled for M1; no vector holds the form)
- **Where:** L §2.3 (`spec/LANGUAGE.md:146`: the built-in fields), L §5.14.3 (`spec/LANGUAGE.md:1432`: an adjustment's attributes `adjusts`, `change`), L §8.3 `LOAD-FIELD` `<field> is not a field of <type>`; the held fixture `adjust/bound-field`; the engine's `movement_field` (`crates/tally/src/load/types.rs`), whose 9d9f5a1 entry says a field whose object is a name stays `Unseen` ("the narrowing, not built, stays #90's").
- **Rule or gate:** DONE gate 5; L §8.3.
- **What happens:** `codes/m0/adjust/base.tally` with `def ls: date = argmax(1 for a in adjustments over .. where a.zz)` after `def net`, as the load probe: refcore `refused: LOAD-FIELD zz is not a field of adjustments (base.tally:29)`, exit 1; the engine (9d9f5a1) passes `a.zz` and refuses the def's type, `refused: LOAD-UNIT rule base of ls is key adjustments, not date (base.tally:29)`, exit 1. Its twin `where a.adjusts.zz` agrees on both (`adjust/bound-adjusts-field`, `LOAD-FIELD zz is not a field of events`).
- **What I tried:** nothing on the engine. refcore is right by L §8.3 (load sees `a`'s kind and that it has no `zz`).
- **What I ask:** nothing now: P20 rules it for M1 (#90). Then T moves `adjust/bound-field` to `edits.tsv`.
- **State:** held, as ruled.
