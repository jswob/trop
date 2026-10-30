# Session 4.5e: the plan of the layer

Written 2026-10-29, after step 2's profiles and before any layer code.

## The profiles

Tool `tools/prof.py` (PyPy, from the check's cwd; argv in `tools/argv.txt`). Three modes:
- `count`: calls and distinct keys;
- `count2`: distinct keys under the layer's own keys, and why a read is ineligible;
- `sample`: a stack sample every 5 ms from a side thread.

cProfile is useless here. It sees only the main thread (refcore works in `big_stack`'s thread), and inside that thread it slows PyPy so much that a 50 s run barely leaves load. The master's 45 s figure was the B ask `tx2_worth`, which is also the E1 check `I:E1:tx2_worth`.

The three checks:
- B: `I:B:b03_sold`, the slowest active B ask, 58.7 s;
- E1: `I:E1:tx2_worth`, 57.0 s;
- LH-10: `I:M:LH-10`, 55.2 s.

| In 40 s (count) | B b03_sold | E1 tx2_worth | LH-10 |
|---|---|---|---|
| `read_input` calls / distinct, strict ctx key (law, partial, walk, wp included) | 813 420 / 283 925 | 867 403 / 302 180 | 442 676 / 9 345 |
| `read_input` distinct, the layer's key (law replaced by the read's day `d`, see below) | 1.51 M / 509 529 (count2, in 40 s) | 1.58 M / 514 933 | 505 436 / 9 775 |
| `candidates` (rows, keys, info) distinct | **649** in 1.51 M | **649** in 1.58 M | 8 295 in 499 460 |
| `row_matches` calls / distinct (row, keys) | 41.5 M / 6 963 | 44.4 M / 6 963 | 12.3 M / 165 616 |
| `read_def` calls / distinct | 119 343 / 70 326 | 124 570 / 72 572 | 1.45 M / 690 582 |
| `read_input` eligible for the memo (count2) | 100 % | 100 % | 95 % (5 % have pins) |

Sampled (70 s, share of samples; the JIT inlines `row_matches` into `candidates`):

| | B | E1 | LH-10 |
|---|---|---|---|
| answering the ask (inclusive) | 95 % | 97.6 % | 99.4 % |
| `read_input` incl. | 45.8 % | 46.0 % | 15.9 % |
| `candidates` incl. (≈ the row scan) | 31.2 % | 31.2 % | 11.7 % |
| `lawless` self (from `need_law`, a scan of all rows) | 4.1 % | 4.4 % | small |
| `IndexInfo.holding` self (a scan of an index's keys) | 3.5 % | 3.4 % | small |
| `choose` self | 4.0 % | 3.8 % | 1.3 % |
| `_read_def` / `def_cell` / `key_values` self | 2 % each | 2 % each | 12 / 9 / 9 % |

**What it says:**
- B's and E1's lookups are dominated by the row scan: 1.5 M scans over only 649 distinct (rows, keys) pairs.
- Their remembered reads repeat only 3× (`d` is a day, and a walk reads many days).
- LH-10 is different. Its reads repeat 50×, but 84 % of its time is outside `read_input`, in the def cells, which refcore already memoises in `Engine.memo`.

## Expected gain (Amdahl, before measuring)

- B/E1: the index removes most of the 31 %. `lawless` and `holding` take about 8 %, and the read memo takes about 2/3 of what is left of `read_input`'s 15 %. If all of that went, the time would drop about 50 %: about 2×, borderline for the stop rule.
- LH-10: about 15 % saved, 1.2×.
- An honest reading: the 2× bar is at the edge for the B-shaped checks, which are most of the 12 slowest. Measure after round 1 (rule 5).

## What is wrapped, and what each one reads and writes

All of it lives in `engine/tools/refcore_fast/`, wrapping `refcore.evaluate.Engine` and `refcore.model` classes from outside. Each memo is per engine instance (an `Engine` lives for one ask), so memory is bounded by one ask.

1. **`Engine.candidates`, the row index.**
   - **What it reads:** `row_matches(r, keys, info)` and `unstated_ok(r, keys, info)` read `r.pattern`, `r.known`, `info.binds`, and the model: `indexes[].pos`, `axes[].options`, `kind_order_key(…, self.ask_K)`, `things`, `kinds`, `entries`. The model is only changed by the loader (and joint's build), and both are done before any ask.
   - **Its effects:** none.
   - **The index:**
     - **Key:** `(id(rows), len(rows), id(info), _kv of keys, ask_K, model._WSET[0])`. The entry holds `rows` itself, so the id can't be reused, and the hit is checked with `is`.
     - **Value:** the sub-list of rows passing both tests, in order.
   - **The call:** the original `candidates` is then called on that sub-list, so the order, the cut tests, `self._K` and every exception are the original's.
   - **Exceptions:** if a filter test raises, the layer calls the original on all rows. The original's first raise may come from a different row (`row_visible` comes before the next row's `row_matches`).
2. **`Engine.read_input`, the read memo.**
   - **Eligible only when:**
     - the engine is not tracing, and `each_met` is None;
     - the context has no pins;
     - the input has no `prices`, and is not a reading;
     - the model has no `same_rows` and no `joint`;
     - none of the input's rows has an expression value (`node`, `sexpr`).

     So no `ev` runs inside it, and nothing touches `Engine.memo`, its journal or `_sexpr_active`.
   - **What it reads then** (the path `index_assumed` → `pin_value` → `time_point` → `need_law` → `candidates` → `choose` → `no_candidate` | `row_value` → `apply_field_pins`):
     - `info`, `keys`, and the read's day `d` from `time_point`, which is `ctx.law` unless the input is keyed by one period index;
     - `ctx.cut` and its `windows` (a `WCut` compares as a date, so its windows are keyed apart), `ctx.windows` (`window_choice`) and `ctx.wording` (`wording_ok`);
     - `model._WSET[0]` (`in_world`), `self.ask_K` (`held`, `key_ord`), `calendar`, `probe`;
     - for safety, also `ctx.axes`, `ctx.path`, `ctx.world`, `ctx.unc`, `ctx.rnd`.
   - **What it doesn't read:** `ctx.law` (apart from through `d`), `ctx.partial`, `ctx.walk`, `ctx.wp`. Each of these is read only in `ev`, and no `ev` runs here.
   - **Its effects:**
     - register entries, `cur.rows`, `cur.axes`, `cur.each` and `cur.missing`: all go to the current `Coll`, and the layer records them in a child `Coll`, refcore's own push idiom;
     - `self._K`, set by `candidates`;
     - nothing else (`trace_row` needs tracing, `each_met` needs it set, and both are excluded).
   - **The answer:** the value object itself. refcore's values are immutable (`Hole.qs` is a frozenset).
   - **An exception:** the read is not remembered, and the notes it wrote are still merged, as the original would have left them.
   - **A replay:** merges the recorded `Coll` into the current one, sets `_K`, and returns the value.
3. **`InputInfo.lawless`**, memoised per info. It is pure over static `rows`, `reading` and `observed`, and keyed by the info and `len(rows)`.
4. **`IndexInfo.holding(d)`**, memoised per (index, d) for a plain date `d`. The index's keys are static after load.

**Not wrapped:**
- `read_def` and the def cells. refcore already memoises them in `Engine.memo` (a `JournalDict` that walks roll back), and their effects include that journal.
- `row_matches` alone. It is cheaper to index the whole filter.

## `--verify N` and the self-test

- **`--verify N`:**
  - One in N memo hits (each wrapper counts its own) is recomputed the slow way, with the layer bypassed below it.
  - The comparison covers the answer (deep structural equality over slots and dicts, identity for rows) and the notes (`reg`, `rows` with its order, `axes`, `each`, `missing`, `trace`, `_K`).
  - Any difference prints `refcore_fast: VERIFY MISMATCH …` to stderr and exits 70.
- **The entry:** `python -I -B engine/tools/refcore_fast [--verify N] [--plant] <refcore argv>`.
- **`--plant`** drops `d` from the read memo's key: a planted omission.
- **The self-test:** a check that reads one input at two days must stop under `--plant --verify 1`, and pass under `--verify 1` without the plant. The check is picked by running the unit layer under the plant.

## Round 1, then the stop rule

Build items 1–4, check that `refcore_fast test` gives `950 passed`, and time the 12 slowest integration checks at `--jobs 3`. If the median gain is under 2×, stop and report.
