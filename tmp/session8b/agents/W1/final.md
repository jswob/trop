<!-- saved by tools/save_final.py from agent agent-a561416b1afa7c8b5.jsonl (W1) -->

W1 final report (session 8b, PERFORMANCE and BOUNDS for tally 2.1)

**Files written**
- `/Users/jswob/projects/trop/tmp/session8b/drafts/PERFORMANCE.md` (73 KB, §1–§10)
- `/Users/jswob/projects/trop/tmp/session8b/drafts/BOUNDS.md` (22 KB; 53 rows)
- My section in `/Users/jswob/projects/trop/tmp/session8b/open.md` (`## W1`, appended only)
- Scratch: `/Users/jswob/projects/trop/tmp/session8b/agents/W1/progress.md`

**What PERFORMANCE carries.** Every row of outline §A:
- **§1:** shape.md's units and the counters behind `--stats`, plus the strategy as 14 numbered items.
- **§2.2, the non-empty-day list (SO8-1/O-1):** 14 items, N-1..N-14, each derived from E or L with a section and line citation. Beyond SO8-1's list, the derivation found these:
  - the day after a window's last day (E l.177);
  - the V.known date of an unsettled adjustment (L l.1435);
  - a joint model's own unlinked movement into a member's place (L l.1805);
  - a hole movement's first possible day (E l.211).
- Things judged not to be acts are also covered: `known` dates and lifespan starts. The step-every-day walk is named as the oracle.
- **§3.3, the trimming derivation (SO8-2/O-2):** a cell whose cone reaches no running read, no projection and no random fact can drop (holder, t) and keep only the cut class of min(t, K). This changes no byte, so it is a cache choice. It rests on one assumption, logged as O-W1-1.
- **§4–§7:** the incrementality seam, paths and the merge rule, numbers, and the cost table, which has the new 2.1 rows (parts, R-114, R-115, observations, `parties`, keyed decisions, joint members).
- **§8:** all fixed ids P8-1..15, P8-L1..L18 (17 rows, no L:7) and P8-M1..M4. There is no P8-L7 and no extra micro row.
  - P8-5 and P8-M4 are restated as one common-case read of `low(x, a..b) from t`. That counterpart exists, so neither is open.
  - P8-1 and P8-7 say in the row that they rest on OPERATIONS.md, which is stale.
- **§9:** caps checked against L and E codes. It adds the uncapped path accumulator, the uncapped joint nesting, live labels and the 8 GB runner ceiling. Out-of-reach steps are restated as holes with their reasons.
- **§10:** 21 unknown rows, U-1..U-21, each with the section it moves and the milestone that measures it.
- **Conventions:** every estimate has a `python3 -I -c` block whose output was run and pasted. The draft names no crate and no Rust API.

**What BOUNDS carries.** All 53 rows. A script check shows every status is unchanged. The Tally column and the notes now use 2.1's constructs and section numbers. A new "Changed for tally 2.1" section lists the renames. I checked LH-17's step 6 in `models/LH-17.tally`: it is a hole rule with its reason.

**Open items added:**
- **O-W1-1:** is the hole a projection gives for a fact not yet known on t the read rule's own hole? The §3.3 trimming assumes it is.
- **O-W1-2:** a mixed Monte Carlo load row is budgeted as its deterministic part at ≤ 10 s plus its paths at N-2003's rate. This is my reading of D4-6, for the user to confirm.
- **O-W1-3:** the path-walk years H for L:1, L:11 and LH-17-mc, and LH-17-mc's path count, are not stated anywhere; T fixes them. Meanwhile §8.3 gives the formula and its values at H = 20, 30 and 40 years.
- **O-W1-4:** P8-13 and P8-14 use refcore's partial counts as budgets, which is stricter than refcore's true figures. P8-14's days-stepped budget is only reported, because X's non-empty share was never measured.
- **O-W1-5:** I propose a BOUNDS row 54 for 2.1's labels (ownership by origin); I did not add it.
- **O-W1-6:** Group I's ≤ 1 GB footprint comes from R4 Q6's table, not from D4-7's list.

**For the lead:**
- PERFORMANCE is about twice the brief's 30–40 KB. The outline asked for a lot, and I didn't trim, to avoid dropping required rows. It can be cut if you want.
- §8.3's split of mixed rows (O-W1-2) is my interpretation, not a G1 text.
- To find D-10's horizon I read `engine/tests/cases/keys/D/D-10.md` (an answer key, not sealed). No figure from it was used.
- I never opened `tmp/holdout/` or any `*-hidden.md` file.
- I found no contradiction between the sources.
