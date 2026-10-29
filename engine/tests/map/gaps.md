# Gaps

Session 4.5b, 2026-10-29. Each gap gets one line of reason, a proposed action and a cost class.

**The actions:**
- **retag:** add edges to a key's `edges:` line where its derivation already passes them. This is a case-set change for 4.5c.
- **pair card:** a probe card with two edges, keyed by `keys_probes.py`.
- **new question:** a question in a named household.
- **accept,** with the reason.

**The cost classes:**
- **small:** a retag, a pair card, or a question keyed with functions the household's key script already has.
- **medium:** a new key-script function, or a new knob.
- **large:** a new draw family or a household text change that moves other keys.

**Rule for new questions:** add them as a scenario or as a pure ask, so that no existing key moves.

**What counts as integration here** (brief §5, step 3b): the 39 household questions plus the 79 old models (`I:<H>:*`, `I:M:*`).
- The amendments household (`I:X:*`) and the blind texts (`I:hardset/*`) are listed where they would hit a gap. They don't count: X's figures are the lead's hand arithmetic, 12 cells are interpreter prints, and the blind texts have no key (see d).

## a. Capabilities only the old models check

The visible edges cite 33 MX ids, not 37:
- 3 are named "outside the set": MX-08, 13 and 40.
- 4 sit in no visible edge: MX-01, 02, 03 and 07. `check_edges.py` counts these as the holdout's ids ("ids in no visible edge: 6").

| # | Capability | Checked by | Reason | Action | Cost |
|---|---|---|---|---|---|
| a1 | MX-08 deemed disposal and anti-deferral | CB-13, CB-14 | It composes edges the set has: an event treated as a sale (OWN-05), "as if" (LAW-05), history spread over the holding (LAW-11, NUM-02). No home-law rule taxes unsold units | accept | – |
| a2 | MX-13 required withdrawals by divisor table | LH-07, LH-08 (and LH-12..14 in `matrix.md`) | It composes a dated table (LAW-01, LAW-03), a relation (PTY-01) and a prior year-end value (VAL-02). No household draws one | accept | – |
| a3 | MX-40 scores and calibration | CH-01, CH-02, CH-03 | A real capability the catalogue lacks: judging a forecast against what happened. No household question scores a forecast, and CH-02 maps to no edge at all | **user's call.** Candidate edge `RAND-05 · Scoring a forecast` plus a probe card (closed form: the CRPS of a normal, CH-03). A question in D next to D-12's replay only if the app will score its forecasts | small (card); medium (question) |
| a4 | MX-01 relief methods, MX-02 credit limits and carryover, MX-03 mid-year residence, MX-07 insurance periods across states | CB-01..07, CB-11, CB-12 | No visible edge; by `check_edges.py` they are the holdout's ids. A visible probe would split the holdout, which is forbidden. The old models keep them in integration | accept | – |
| a5 | MX-10's wash-sale side (a loss disallowed by a 30-day window and moved into a replacement lot) | LH-01 | It composes lots (OWN-06/07), a window (TIME-05) and an exception (LAW-12); A-05 and A-06 hold the parts, but not the move of a loss into a lot's cost | accept | – |
| a6 | MX-35's named solver (a circular rule the law solves by a stated method) | UL-07, UL-08 | RULE-04 tests only the refusal side, for a household's own loop. Solving a loop by the law's named method is a different capability and has no probe | candidate edge (LAW axis) plus a probe card; the closed form can come from UL-08's iteration | small |

## b. Interacting pairs

**Counted the brief's way** (pairs with the same axis included, 120 axis pairs):

| Checks | Axis pairs met | Edge pairs met |
|---|---|---|
| households | **85** (71 across axes + 14 within one; brief: 85 ✓) | **563** (brief ✓) |
| households + old models | **91** (77 + 14) | 696 |
| + amendments household | 93 | 844 |
| + blind texts | 97 | 931 |

There are 4 186 possible edge pairs. Edge pairs are counted, not judged.

**★ edges:** all 14 meet at least 2 integration checks (households + old models). The fewest: SCEN-02 and RAND-03 with 2 each, then RULE-03 and REC-02 with 3. Households alone give CUR-01 only one (B-03), and the old models add CB-08, CB-09, CB-10 and CMP-01, each under different rules. Pass.

**The tags undercount.** A key's `edges:` line names what the question aims at, not every edge its derivation passes. Reading five derivations (B-03, B-05, B-11, B-12, D-10) and the path functions (`fam/b_paths.py`, `fam/d_paths.py`) turns 12 of the 19 unmet pairs below into retags.

**The 29 unmet pairs** (households + old models): 1 within one axis and 28 across axes.

### SURF: accept all 10

ASK, CUR, FLOW, OWN, PTY, REC, SCEN, TIME, VAL × SURF, and SURF × SURF.
- **Reason:** SURF is how a text is written. Its cards have no key by design (`probes/README.md`). It holds or fails at load, for every text alike, so it changes no other edge's result.
- **What does meet:** SURF-03 (draws written) meets RAND, LAW, LIFE, NUM and RULE through B-12 and D-10.

### The other 19

| # | Pair | Can interact? (one line) | Met in fact? | Action | Cost |
|---|---|---|---|---|---|
| b1 | ASK–CUR | yes: a per-person net worth holding a foreign balance needs its rate, or a hole | yes: B-11 step 2 counts Piotr's florens (16 516.65, B-03 step 13) | retag B-11 + CUR-03 | small |
| b2 | ASK–SCEN | yes: an ask's holes and explanation under a scenario | yes: B-11 step 6 ("without the twist") | retag B-11 + SCEN-02 | small |
| b3 | ASK–RAND | yes: a quantile or a share over paths, with paths that hold a hole | yes: D-10 step 5 asks net-worth bands on the paths she is alive | retag D-10 + ASK-01 | small |
| b4 | CUR–TIME | yes: the law's rate is the business day before (TIME-02) | yes: B-03 steps 1–10 | retag B-03 + TIME-02 | small |
| b5 | CUR–VAL | yes: the rate is a gazette figure read at a date | yes: B-03 steps 14–15 (gazette FX-FLR, published dates) | retag B-03 + VAL-01, VAL-02 | small |
| b6 | CUR–REC | yes: the bank's own rate comes from its statement | yes: B-03's Lazur rates are rows of `B/records/laz-8813-flr-2025.csv` | retag B-03 + REC-05 | small |
| b7 | REC–SCEN | yes: a counterfactual past where the records are the past | yes: B-05 steps 1–2 read the exports and the days no export shows | retag B-05 + REC-01, REC-02 | small |
| b8 | RAND–SCEN | yes: a scenario compared over the same paths | yes: B-12 runs `scenarios/no_overpay.tally` on common draws | retag B-12 + SCEN-01 | small |
| b9 | RAND–TIME | yes: a drawn pay day lands in or out of a window | yes: `b_paths.draws` draws Arnika's delay each month, and `business` uses `on_or_next(15th)` | retag B-12 + TIME-03, TIME-02 | small |
| b10 | FLOW–RAND | yes, weakly: conservation per path of a drawn amount | yes: B-12's overpayments and savings transfers on each path | retag B-12 + FLOW-01 (and LIFE-03, the mortgage on each path) | small |
| b11 | PTY–RAND | yes: a person's death drawn per path ends their incomes | yes: `d_paths` draws the lifetime (D-10 step 1) | retag D-10 + PTY-01. Succession on a path (PTY-03 × RAND) stays unmet: see b19 | small |
| b12 | RAND–VAL | yes: a draw family's parameters are register rows | likely: D-10's fund draws use D-HR3's register (mean 0.05, sd 0.16, df 4; D-11 figure 7) | retag D-10 + VAL-07 after 4.5c confirms the text reads them from the register | small |
| b13 | CUR–RAND | yes: a drawn rate changes the converted law amounts per path | no: B's paths draw no rate, and D has one currency | pair card CUR-01 × RAND-01 (B's paths are too heavy, K5) | small |
| b14 | OWN–RAND | yes: a sale on a path picks lots and pays the gains tax on that path | partly: D-10 values the fund on each path; no lot is sold on a path in the key | new question in D: the gains tax of the fund units sold when the account runs dry, over D-10's paths (a pure ask if the text already sells units on a path, else a scenario; `keys_d.py` gains a lot function on paths) | medium |
| b15 | PTY–SCEN | yes: "what if one partner stops working" recomputes the joint tax | no (only X change 7, unkeyed) | **new question NQ-A** in A, a scenario: Ania stops working from 2023-07-01. Ask the joint 2023 PIT (A-01's PTY-02, LAW-05) and explain the difference (ASK-05) | small |
| b16 | SCEN–TIME | yes: a scenario starting mid-year splits a law's period | no | NQ-A as above (the 2023-07-01 start splits the tax year: TIME-04) | (in NQ-A) |
| b17 | OWN–SCEN | yes: a scenario that moves ownership or changes which lots are sold | no (only the blind I-6, I-15) | NQ-A part (b): the same scenario with Ania's units sold in December 2023 instead (OWN-07) | (in NQ-A) |
| b18 | CUR–SCEN | yes: a rate scenario moves every converted figure | no (only blind I-15) | **new question NQ-B** in B, a scenario: FLR 5 % lower from 2025-07-01. Ask B-03's six later invoices' revenue and VAT, and the Lazur DUK balance on 2025-12-31 | small |
| b19 | CUR–RULE | yes: a house rule acting on a foreign balance | partly: B-HR4 .5 values Lazur's balance (B-03 step 13) but moves nothing | NQ-B part (b): the scenario adds a rule "sell FLR above 2 000.00 on the 1st" and asks what it moves in 2025-H2 (a scenario, so no key moves) | medium (a new rule function in `keys_b.py`) |

PTY-03 × RAND (succession on a path), at edge level: a candidate D question, medium. It is left out of the order below because b11 already meets the axis pair.

**After the actions,** households plus old models would meet all 105 cross-axis pairs except the 9 SURF ones, and 14 of the 15 same-axis pairs (all but SURF × SURF): 110 of 120 in the brief's count (96 across axes + 14 within one), up from 91. The 10 left are the SURF ones.

## c. Probe-only edges

| Edge | Reason it isn't a gap | Action | Cost |
|---|---|---|---|
| SURF-01 no bare figures | a writing rule. Every integration text is written under it, and B-12 and D-10 meet SURF-03 the same way. It has no result to key | accept; 4.5c may tag SURF-01, 02 and 04 on every integration check by a load scan instead of a question | small |
| SURF-02 surface words | as SURF-01 | accept | – |
| SURF-04 stated once, checked before it runs | a load-time property: every integration text loads, or fails before it runs | accept | – |

## d. Checks without an independent key

| # | What | Where it comes from | Action | Cost |
|---|---|---|---|---|
| d1 | 88 probe cards: the hand answer equals `keys_probes.py`'s | the case set's own check | none | – |
| d2 | 4 SURF cards (`U:surf/*`, 6 unit checks): no key by design | `probes/README.md` | accept. Their baseline is snap16's print, so they catch changes, not errors. SURF-03 is keyed through B-12 and D-10 | – |
| d3 | **LH-11**: 3 rows (`a60_a144`, `a89_n`, `a133_x`) | `freeze/KEYS.md` rows 1–3, snap14's printed lines | a derivation independent of the interpreter for the `down` readings (one-line Decimal checks) | small |
| d4 | **CMP-01**: 2 rows (`q_c6_k1..k3`, `q_c6_tax`) | `freeze/KEYS.md` rows 4–5 | the same; the rows only list lifted rows, so a reading of `MIGRATION.md` keys them | small |
| d5 | **X, change 4**, cells 42–44; **X, change 5**, cells 9–16, 20 and 24 (12 cells) | `freeze/KEYS.md` rows 6–17, interpreter prints (snap14, `v5_r2` on snap8) | key them by hand arithmetic, as the rest of X was | medium |
| d6 | X's other cells | the lead's hand arithmetic, not a key script and not blind (`amendments/README.md`) | accept as integration with a weaker key, or give X a key script | large |
| d7 | 11 blind-text checks (`I:hardset/*`, I-6 … I-15) | no key: the cards are blind, and the baselines are snap16's prints | outside the brief's §4 ids, tagged anyway. They catch changes only | accept |
| d8 | LH-15 and LH-16: every success rate and Table 6 cell prints a hole (no data rows), at full size and light | the cases' own texts | accept. Their published figures are unreachable, so both twins test structure and holes (knobs.md K12–K13) | – |

## Also found

- **The join gap:** in 4.5a's `checks.tsv`, 103 household asks have no `question`: D's 69, 23 of B's (B-07, B-12 and others), 6 of A's and 5 of E1's. A per-ask tag can't be joined for them until 4.5a fills that column. Tags here are per question, as the brief says.
- **The edge list's "37 of 40" is 33 visible.** The other 4 (MX-01, 02, 03, 07) are the holdout's by the tool's own count.
- **The tags are aims.** Coverage from `edges:` lines is a floor. 4.5c's retags (b1–b12) should come from the derivations, not from new reading of the texts.
