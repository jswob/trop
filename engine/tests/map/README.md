# The map of the test set

Session 4.5b, 2026-10-29, on paper.
- **Files:** `tags.tsv` (the tags), `gaps.md` (the gaps and their actions), `knobs.md` (the halving test), `findings.md` (one line each, outside the task).
- **The ids** are 4.5a's (`engine/tests/checks.tsv`), except per household question (`I:<H>:<qid>`) and per amendment (`I:X:base`, `I:X:1`..`7`).

## Counts

| | Count |
|---|---|
| **Edges** (the catalogue: `engine/tests/cases/boundaries.md`) | 92 on 15 axes, 14 ★ |
| **Unit checks** | 112: 103 probe texts and 9 sets, from 92 cards; 88 cards keyed |
| **Integration checks, tagged** | 79 old models, 39 household questions, 8 amendments (base + 7 changes), 3 light twins, 11 blind texts |
| **Integration checks, in 4.5a's per-ask runner** | 206 household asks and 60 X asks |
| **Tags** | 714 on 252 checks |

**Where the tags come from:**
- 184 from card `edge:` lines and the blind cards;
- 223 from the keys' `edges:` lines;
- 136 through the mechanics (MX → edge);
- 115 from the old models' story headers;
- 49 from the amendments' descriptions;
- 7 from twins.

## Coverage now

| Measure | Households | + old models |
|---|---|---|
| edges with a probe | 92 / 92 (88 keyed; the 4 SURF cards have no key by design) | – |
| edges met in integration | 89 / 92 (SURF-01, 02, 04 are probe-only) | 89 / 92 |
| axis pairs met (with same-axis pairs, of 120) | 85 (brief ✓) | 91 |
| edge pairs met (of 4 186) | 563 (brief ✓) | 696 |
| ★ edges met in at least 2 checks | 13 / 14 (CUR-01 once: B-03) | 14 / 14 |

**The tags are a floor.** A key's `edges:` line names the question's aim. Five derivations already pass 12 of the 19 unmet non-SURF pairs (gaps.md b1–b12).

## Gaps, in the order proposed (cheapest high-value first)

| # | Action | Fills | Cost |
|---|---|---|---|
| 1 | Retag 5 keys from their derivations: B-03, B-05, B-11, B-12, D-10 | 12 axis pairs (b1–b12) | small |
| 2 | NQ-A: a scenario in A where Ania stops working from 2023-07-01; the joint PIT, its explanation, and a part (b) on lots | PTY–SCEN, SCEN–TIME, OWN–SCEN | small |
| 3 | NQ-B (a): a scenario in B with FLR 5 % lower from 2025-07-01 | CUR–SCEN | small |
| 4 | Pair card CUR-01 × RAND-01 | CUR–RAND | small |
| 5 | Probe card for a circular rule solved by the law's named method (candidate LAW edge) | MX-35's solver side (a6) | small |
| 6 | Independent derivations of LH-11's 3 rows and CMP-01's 2 rows | d3, d4 | small |
| 7 | A run-end knob in `keys_b.py` | makes B-12's capability check possible (K5, (iii) → (ii)) | medium |
| 8 | NQ-B (b): the scenario adds a rule on the FLR balance | CUR–RULE | medium |
| 9 | A D question: the gains tax of fund units sold on D-10's paths | OWN–RAND | medium |
| 10 | Hand keys for X's 12 interpreter cells | d5 | medium |
| 11 | MX-40, scoring a forecast: candidate edge RAND-05 plus a card, a D question only if the app scores forecasts | a3 | user's call; small or medium |
| 12 | A key script for the amendments household | d6 | large; optional |

- **Accepted, with reasons in `gaps.md`:** a1, a2, a4, a5; the 10 SURF pairs; c (the 3 probe-only SURF edges); d1, d2, d7, d8.
- **Total:** small for items 1–6; medium with 7–10. Without item 12, the whole list is **medium**.
- **After items 1–9:** 110 of 120 axis pairs (all but the 10 SURF ones), and every non-SURF cross-axis pair.

## Heavy checks with no answer source at their capability setting (iii)

From `knobs.md`, 14 heavy checks:

| Check | Why it has no source |
|---|---|
| K1, D-10 | its 6 band cells below 10 000 paths: a quantile prints no se, and the keys' draws aren't the language's |
| K5, B-12 | as a whole: it fits the budget only with a run end, and the key script has none (item 7) |
| K9, O:A-3's `one_of` run | no key at all |
| K10, the amendments' walk cells | any change rule with no expected cell before its first date; 4.5c lists them |
| K11, household-bands | its bands, shares, means and paired statistics at a cut path count |

Every other heavy check keeps its key unchanged at the capability setting (i), or has a key-script or one-line source (ii).

## After 4.5c (2026-10-29)

`coverage.py` computes this table from `tags.tsv`. It first reproduced 4.5b's figures above exactly.

| Measure | 4.5b | 4.5c |
|---|---|---|
| edges in the catalogue | 92 | 93 (LAW-20 added) |
| axis pairs met, households + old models (of 120) | 91 | **115**; 116 with the pair card CUR-01 × RAND-01, a unit check |
| edge pairs met, households + old models | 696 | 892 |
| ★ edges met at least twice, households alone | 13 of 14 | 14 of 14 |
| edges met in integration | 89 / 92 | 90 / 93 (SURF-01, 02, 04 are probe-only) |

**Unmet:** CUR–SURF, OWN–SURF, REC–SURF and SURF–SURF (SURF is accepted: `gaps.md` b).

**What changed:**
- **Retags:** the b1–b12 retags are `reach` tags in `tags.tsv`, each citing its derivation step.
- **New tags:** A-13, B-13 and D-13 come from their keys' `edges:` lines; the two cards from their `edge:` lines.
- **LAW-20** is tagged on `I:M:UL-07` and `I:M:UL-08`.
- **Weaker keys** are labelled `key:` in the note column.

The brief expected 110. The retags alone gave 114, because a retag on a check with many edges creates many cross-axis pairs: D-10's SURF-03, for instance, meets every edge added to it.
