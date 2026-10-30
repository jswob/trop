# The measured shape of the work (session 8, step 1)

**Made:** 2026-10-30, 21:30–21:36, by the session 8 lead. Six checks of `engine/tests/checks.tsv` were run on the frozen refcore (tally 2.1 after 7c, 1115 tests), PyPy 3.11, from each check's cwd with run.py's argv, at most 6 jobs at once, under the 8 GB watchdog.
- **Counted** runs wrap refcore's methods with counters (`tools/m8/refcore/__main__.py`, adapted from 4.5e's `prof.py`), so they run about 1.3–1.8× slower; at 88 s they stop and write what they have (marked *partial*).
- **Plain** runs are refcore unchanged, under `/usr/bin/time -l`, 5 at once. Peak memory is macOS's "peak memory footprint" (the watchdog's measure), not RSS.
- The raw files are in `runs/count/` and `runs/plain/`.

**What the units mean (they carry over to any engine):**
- **ev**: one expression node evaluated.
- **def cell**: one read of a def at (item, key values, context), refcore's memo key (seam F). A **hit** is a cell already in the memo; a **miss** computes it.
- **read**: one read of an input by the read rule (L §4.6), keyed by (input, keys, context); **distinct** is how many different keys.
- **walk day**: one day the day walk steps through; **non-empty**: a day with at least one act (a reading, an observation, a leg, a decision).
- **legs booked**: movement legs written to a place's log.
- **parts groups**: the groups of parts work at a place that keeps parts (tally 2.1's labels).
- **Num results**: every exact value refcore makes, sized by the larger bit length of its reduced numerator and denominator.

## The six checks at a glance

| Check | What it is | Plain s | Peak GB | ev | def cells (hit %) | distinct defs (top-2 share) | reads (repeat ×) | walks × days (non-empty %) | legs | parts groups | Num results ≤ 63 bits |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `I:A:a05_ania@c05-ania` | slowest A; lots with batches (parts) | 37.7 | 0.81 | 12.7 M | 367 k (59 %) | 150 k misses over 23 defs (85 %) | 250 k (43×) | 64 × 5 477 (5.4 %) | 49 664 | 33 344 | 100 % (max 38 bits) |
| `I:E1:tx2_mtg_day` | slowest E1; a mortgage's daily decisions | 60.5 | 2.17 | 36.7 M | 1.37 M (55 %) | 619 k over 78 defs (13 %) | 3.90 M (3.6×) | 1 × 2 498 (100 %) | 1 191 | 0 | 99.92 % (max **5 049** bits) |
| `I:E2:E2-01~e2_01a` | L:18's check; joint model, 2 members | 49.1 | 0.91 | 22.5 M | 370 k (56 %) | 161 k over 70 defs (41 %) | 3.84 M (25×) | 8 × 16 070 (6.7 %) | 14 208 | 7 256 | 100 % (max 34 bits) |
| `I:E2:E2-01~e2_01b` | slowest E2, same model | 65.8 | 1.02 | ≥ 33.1 M *partial* | ≥ 547 k | — | ≥ 5.67 M | ≥ 12 × 16 070 | ≥ 19 536 | ≥ 9 977 | 100 % |
| `I:X:replay_september@b_r1` | slowest check of all (52.6 s on fast); a residual replay | — | 4.95 † | ≥ 32.6 M *partial* | ≥ 4.55 M (50 %) | ≥ 2.27 M over 57 defs (63 %; top-3 95 %) | ≥ 1.63 M (2.0×) | 20 (19 projections) × 30–317 | ≥ 1 196 | 0 | 100 % (max 25 bits) |
| `U:set/pairs/PTY-05+OWN-12/ania` | parts probe; a nested joint model with owned places | 0.87 | 0.08 | 830 | 74 (0 %) | 74 over 4 defs | 0 | 3 × 180 (2.2 %) | 18 | 18 | 100 % |

† X's peak is from its counted run on refcore_fast (the memo layer raises memory, 4.5e); a plain refcore run would sit over 60 s at 5 jobs, so it wasn't repeated. Earlier figures, tally 2.0 (4.5e `tools/count*.txt`): **LH-10** reads defs 1.45 M times over 690 k distinct cells and inputs 443 k times over 9.3 k keys (47×); **B `b03_sold`** reads inputs 813 k times over 284 k keys (2.9×), scanning 41.5 M rows.

## Per check, what stands out

**A, `a05_ania@c05-ania`** (`cg_tax`, `net_gain`, `cg_too_much` for 2024, law and known 2025-04-30, `windows each`):
- 12 engines and 64 walks, every one over the same 5 477 days. 94.6 % of those days are empty. Acts: 49 792 observations, 1 984 readings, 128 decision days (one stratum each).
- Two defs, `leave_rank` and `civ1_arrive`, give 85 % of the def misses. Reads repeat 43× per key.
- Numbers: 2.23 M results, 92.5 % integers, 7.2 % short decimals (denominator divides 10⁶), 0.3 % other.

**E1, `tx2_mtg_day`** (`mtg_pay[2027-04].day`, `bdays_due`, two years of `flows`, law 2025-12-31, known 2026-01-15):
- One walk of 2 498 days, every day non-empty: 5 077 decision instances in 2–3 strata a day.
- Defs are spread thin: the top 8 (`ov_on`, `mtg_bal`, `principal_on`, `ov_fee`, …) hold 29–42 k misses each. 1.07 M distinct read keys, read 3.6× each.
- **The only check that leaves 64 bits:** 1 693 results pass 1 024 bits, 1 109 pass 4 096 bits, the largest is 5 049 bits (about 1 520 digits; the cap is 13 606 bits). That is 0.08 % of 2.08 M results. 77 % are integers, 20 % short decimals, 3.1 % other.

**E2, L:18** (`e2_01a`: `d.payout[h]` and `d.div_paid_on` for four heirs, law 2025-06-30, known 2026-09-30):
- 8 engines for a joint model of 2 members, and 8 walks over 16 070 days (44 years). 93 % of the days are empty.
- `d·rent_amt`, a def of member D, gives 32 % of the misses: a read across the member boundary at many keys. Reads repeat 25× per key.
- `e2_01b` (three more asks on the same model) adds 4 walks; it hit the 88 s limit counted.

**X, `replay_september@b_r1`** (`residuals konto 2026-09`):
- 20 walks: 19 are written projections (`low`, `bal(x, d) from t`, D-6), each 30–317 days.
- Three defs keyed by day, `debt_before`, `principal_part` and `overpay_paid`, give 95 % of 2.27 M misses: a recurrence asked at every day from many projections. The memo held 2.27 M cells at the limit.
- 12.6 M number results, 99.2 % integers.

**Parts probe, PTY-05+OWN-12/ania:** 3 engines (a joint model inside a joint model), 3 walks of 180 days, 18 legs and 18 parts groups. Cheap by design. The parts cost shows at size in A: 33 344 groups for 49 664 legs.

## What carries over to the engine

1. **Half the def reads are first reads.** Hit rates run 50–59 %, and distinct def cells number 74 to 2.3 M per ask. A few defs keyed by day or by period carry most misses (A 85 %, X 95 %). So the memo key (seam F) and how a recurrence by day is held matter more than memo lookups.
2. **Reads repeat 2–47× per key.** An index by (input, keys) and a read memo pay. 4.5e showed refcore's row scan was 31 % of B's time.
3. **The walk dominates by repetition, not by days.** refcore steps every calendar day, but only 2–7 % have work, except E1's daily decisions. The same long walk is redone many times: 64× in A (`windows each`), 8–12× in E2 (members and asks), 20× in X (projections). Skipping empty days, and sharing a walk's prefix (checkpoints, PERFORMANCE §4.2; seam J), is where the order of magnitude is.
4. **Numbers fit 64 bits almost always.** Over 99.9 % of results fit, and **money was always a whole number of cents** (0 money values below a cent in all six). But a real household check (E1) reaches 5 049 bits, so the big tier is needed and must be correct, if rarely fast.
5. **Labels pay only where opted in.** E1 and X run 0 parts groups.
6. **Memory follows the memo.** refcore holds up to 2.3 M cells (X, 4.95 GB on the fast layer), so cell size matters at the 8 GB cap. Python's object overhead inflates every figure; the ratio, not the GB, carries over (inference).
7. **Expression nodes:** 12–37 M per ask, about 60–85 per def miss. refcore spends about 1.6–3 µs per node on PyPy. An engine at 10–50 ns per node would run E1's 36.7 M nodes in 0.4–1.8 s before any structural saving (inference: the per-node figures are assumed, not measured).

## Limits of these figures

- They are refcore's, a reference interpreter written for clarity: an engine that skips empty days or shares walks makes fewer ev and def cells, not the same number faster.
- Bit sizes are of results only; intermediates inside one operation (cross-multiplied sums) are larger.
- Two counted runs are partial at 88 s. Their counts are lower bounds, and their shape matches their complete twins.
- No Monte Carlo check was counted. LH-17-mc-light runs 256 paths in 45–65 s on the fast layer, about 0.2 s per path (stored results).
