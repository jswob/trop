# Parallelism: paths, timelines, the ordered merge

Using 6 cores must not change one printed byte (DONE gate 6: 1 and 6 workers give equal hashes). In TypeScript terms: `Promise.all` writing each result into `results[i]`, then one loop reducing them in index order, never "whoever finishes first". The design is PERFORMANCE §5 (G1 item 4, D3-1 to D3-7); the code shape is BUILDER §7; the rule is R-42 as amended.

## Contents
1. The thread module
2. What runs in parallel
3. The merge
4. The shared layer
5. Testing gate 6

## 1. The thread module (R-42 a–e, D3-1)

- **One module, sanctioned by the lead (R-108), is the only one with threads.** Everything else is single-threaded: one context is evaluated on one thread.
- Every thread is started as `std::thread::Builder::new().stack_size(s).spawn_scoped(scope, f)` inside `std::thread::scope`, with `s` passed in by the caller. Never `thread::spawn` or `Scope::spawn`: a plain spawn reads `RUST_MIN_STACK` (an environment read, R-39) and gets 2 MiB against the main thread's 8 MiB, so a deep case could pass on 1 worker and crash on 6.
- The 1-worker run goes through the same module, on one spawned thread with the same stack.
- The worker count is an argument; the CLI defaults to 6, a flag lowers it; never read from the machine (no `available_parallelism`). No global pool, no rayon (its `reduce` and multi-error choice aren't deterministic).
- **No channel, lock or atomic outside the module** (R-42 d). Inside it, each worker owns its `Vec`, so there is no interleaving to explore.

**Bad / good**
```rust
// Bad: shared mutable state, finishing order decides the result
let out = Mutex::new(Vec::new());
scope.spawn(|| out.lock()?.push(run_path(p)));
// Good: numbered slots, results through join, one merge in task-index order
// (a sketch: a failed spawn or join maps to ENGINE-INVARIANT, never a panic)
let handle = Builder::new().stack_size(stack).spawn_scoped(scope, move || run_block(block))?;
let slots: Vec<Slot> = handle.join().map_err(|_| Invariant::Worker)?; // one block's slots
// ... then one loop merges every block's slots by task index
```

## 2. What runs in parallel (D3-2)

- **First:** Monte Carlo paths, and independent timelines: axis combinations, `windows each`, rolling origins, scenarios.
- **Later:** joint members, parallel asks.
- Tasks are cut into contiguous blocks, one per worker (a means, E §7 step 5; the guarantee is the ordered merge, SO8-5).

## 3. The merge (D3-3)

- One thread merges the numbered slots in **task-index order** (a task is a path or a timeline).
- refcore's rule: per line, terms in order; the first refusing path wins, else the union of holes; quantiles sorted stably by (value, path). Which refusal prints when several refuse is logged for the spec (SO8-3).
- A path statistic's accumulation is uncapped and has its own type (R-117); its order is still the task order, so which refusal prints is fixed.

## 4. The shared layer (D3-4)

- Run the first task alone, freeze the cells it shares, then run the rest. The frozen layer is handed out as `&` (a read-only borrow with no interior mutability), so the compiler enforces the freeze.
- Count the cells a worker recomputes locally (the `counters` build reports it).
- Paths keep no read sets; a path's layer is freed after its terms.

## 5. Testing gate 6 (D3-6)

- The worker sweep {1, 2, 3, 5, 6, 7}: equal result hashes.
- A test-only switch that runs the blocks in reverse completion order and merges as usual. It never reaches the shipped binary: R-51, amended at session 8c to cover it and the `counters` feature (G2a, 2026-10-30), keeps both out, as it keeps the fault-injection hook.
- Planted violations: a merge in finishing order, `Scope::spawn` without a stack size, a `HashMap` in a path layer, the refusal taken from the highest path.
- The deepest-expression case at 1 and 6 workers on the same explicit stack.
