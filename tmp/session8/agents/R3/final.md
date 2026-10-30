<!-- saved by tools/save_final.py from agent agent-a1cc9a5b0f99638ab.jsonl (R3) -->

R3's report is done: `/Users/jswob/projects/trop/tmp/research_notes/engine-prep/R3.md`, about 34 KB (a little over the 30 KB target). Scratch notes are in `tmp/research_notes/engine-prep/scratch/R3/progress.md`. The seal held: I never opened or searched `tmp/holdout/`, and nothing ran except one `python3 -I` arithmetic line.

**The three findings that matter most:**
1. **Parallel work only fills numbered slots; one thread merges in order.** Every split (paths, axis combinations, `windows each` leaves, rolling origins, scenario timelines, joint members) is an independent context over a shared layer. Workers write results into slots by task number, and one thread does every merge in index order, as refcore does. ENGINE §7 already keeps all N terms, so this costs no extra memory.
2. **The merge order is now forced by refusals, not arithmetic.** Tally 2.1 (LANGUAGE §7.1, R-13) took the digit cap off path-statistic sums, so phase 1's SO-22 no longer applies to sums. When several paths refuse, refcore prints the first term in line order, then the lowest path, ahead of any hole. The spec doesn't state this. rayon documents its own multi-error choice as "not deterministic".
3. **New hole in R-42 and R-39:** a thread spawned the plain way in std (1.99.1) takes its stack size from the `RUST_MIN_STACK` environment variable. It also gets 2 MiB, against 8 MiB on the macOS main thread, so a deep expression could crash with 6 workers and not with 1.

**Decisions, with my pick for each:**
- **D3-1, threading tool:** keep R-42 (`std::thread::scope`, no rayon), with five amendments:
  - spawn only through `Builder::spawn_scoped` with a stack size passed in;
  - the 1-worker run goes through the same module;
  - "path order" becomes "task-index order";
  - ban channels, locks and atomics in the library outside the module;
  - ban `Scope::spawn`.
- **D3-2, what runs in parallel first:** paths plus independent timelines in the first build; joint members and parallel asks later.
- **D3-3, merge rule:** copy refcore's rule, including a stable sort by (value, path) for quantiles; no early cancellation in the first build.
- **D3-4, shared layer:** run task 1 alone, then freeze the layer; cells a task misses go into its own layer and are folded back afterwards; count the duplicated work.
- **D3-5, memory:** the worker count is an argument, the CLI defaults to 6, and a flag lowers it; never read from the machine.
- **D3-6, gate 6 tests:** a worker sweep {1, 2, 3, 5, 6, 7}, a test-only reversed or shuffled completion mode, planted violations and a deep-stack case. Add shuttle only if an atomic enters the module.
- **D3-7, Maritz–Jarrett:** refined interval evaluation to the 1e-6 rounding; refcore's exact method is the oracle at small N.

**What the lead must know:**
- **refcore departs from the spec.** In `eval2.py:952–958`, a `mean` over mixed units or rates caps its partial sums in path order (`evaluate.py:4340`), which LANGUAGE §7.1 R-13 forbids. The engine should follow the spec. A gate 10 mismatch would only appear past 4 096 digits.
- **Maritz–Jarrett is a performance trap.** refcore's exact method works on integers the size of N^N: about 5×10⁷ steps on 40 000-digit integers at D-10's 10 000 paths, and out of reach at 100 000. These are estimates, not measurements.
- **Four spec observations, SO-paths-1 to 4:**
  - which refusal prints across paths;
  - the byte encoding of the sample address is unnamed (refcore uses UTF-8);
  - ENGINE §7.5's "one block per worker" is a means, not the guarantee;
  - which of several equal quantile values carrying different error bounds prints.
- **Versions checked today:** rayon 1.12.0, std 1.99.1, sha2 0.11.1, proptest 1.12.0, loom 0.7.2 (has no `thread::scope`), shuttle 0.9.9 (has it), crossbeam-utils 0.8.23, bumpalo 3.20.3, chili 0.2.1, salsa 0.28.5.
- **Sources I couldn't use:** no primary page for typst's or cargo's job ordering was researched, and none of the newer schedulers (chili, forte) documents determinism.
