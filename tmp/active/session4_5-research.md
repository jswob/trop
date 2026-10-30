# Session 4.5 research: a test architecture for Tally

Written 2026-10-29, after session 5 was stopped. The question: how to bring the test set into a standard layered architecture that runs quickly, keeps the reference interpreter as the judge, and gives every future check one way to be broken down. This note holds the evidence and the options; the brief comes after the user picks.

## 1. The problem, in general terms

A language with a frozen spec, an executable reference semantics that is obviously right but slow (`engine/tools/refcore`), an independent oracle that knows nothing of the language (the case set's key scripts), a fast engine still to come, and a test corpus of mixed sizes. Three needs:

1. the reference stays the judge of correctness;
2. every check is small enough to run in a fixed budget, and its full-size twin is kept for the engine;
3. one rule decides, for anything new, which layer it goes to and what it must cover.

## 2. Measured today (2026-10-29)

| Fact | Figure | How |
|---|---|---|
| Loading B is not the cost | syntax check of B and its 33 law files 8 s; load scan 0.1 s; refcore's own load ~2 % of a run | `parsecheck.py`, `loadcheck.py`, a 60 s stack sample |
| Where B's time goes | 95 % inside answering one ask; in 45 s, 284 087 fact lookups (`read_input`) for 691 plan instances, 13.5 M row comparisons (`row_matches`), still in `walk.prepare` (the day walk not yet started); exact fractions under 1 % | cProfile, 45 s, ask `tx2_worth` (scratch tools in the session's scratchpad `prof/`) |
| B is small | the case set's Python model computes all 12 of B's keys in 5.5 s (30 paths; 3 000 paths takes over 100 s, only B-12 changes) | `keys_b.py 30` on a scratch copy of `tmp/cases` |
| A single B ask on refcore | about 3 min for any walk (session 4: `bal(cis, 2024-05-17)` 2 min 59 s) | `tmp/design2/rows/E1.check.md` |
| Unit layer is already fast | 103 probe texts in 101 s in all, slowest 39 s; 79 old models: 77 run in 131 s, slowest 96 s; refcore's 950 tests 12–13 s | `scratch/lead/check/snap16-*/summary.txt`; `refcore test` |
| refcore can run on PyPy, likely unchanged | standard library only; compiles at the Python 3.11 language level; the 950 tests pass on CPython 3.11 (12.4 s); `uv` offers `pypy-3.11.13` for download | import scan, `python3.11 refcore test`, `uv python list` |
| Integration coverage, today | 39 household questions name 2–14 capabilities each (median 5); 89 of 92 capabilities appear in some question (3 are probe-only); 563 capability pairs co-occur in a question; 85 of 120 axis pairs | the keys' `edges:` lines |
| parsecheck lags tally 2 | rejects the `replace` band rows of B's `law/PIT-3.tally` (line 40) that refcore accepts | `parsecheck.py -v` |

## 3. Prior art

1. **A reference model as judge, checked on many small inputs (AWS).**
   - **Cedar** (AWS's authorization language) is built by "verification-guided development": an executable model in Lean with proofs, a production engine in Rust, and differential random testing of the two on millions of generated inputs. A release waits until the model, the proofs and the differential tests are up to date. AWS names the model's *fast runtime* as one reason for choosing Lean, because differential testing against a slow model is impractical.
   - **ShardStore** (S3's storage node) has an executable reference model per component, "the same interface … a much simpler implementation". Property-based tests compare the implementation against the models, and correctness is split into independent properties, each checked by the tool that suits it.
   - **For Tally:** the shape is DONE gate 10's already. The lesson is that an obviously right model must still be fast enough on small inputs to be used as the judge; its job is many small inputs, and scale belongs to the fast implementation.
2. **Small scope.** Jackson's small scope hypothesis (Alloy, *Software Abstractions*), and SmallCheck's version of it: a program that fails its specification almost always fails on some simple case. **For Tally:** the capability version of a check is the smallest world that still holds every moving part.
3. **Interaction coverage (NIST combinatorial testing; Kuhn, Wallace, Gallo, IEEE TSE 2004).** In the systems studied, 42 % of failures were triggered by one parameter, 70 % by two, 89 % by three, and all by six or fewer. **For Tally:** the integration layer gets a measurable goal. It doesn't need more households; it needs every pair (perhaps every triple) of capabilities *that can interact* to meet in some integration check. Today's map: 563 pairs, 85 of 120 axis pairs.
4. **Size and scope as separate labels (Bazel, Google).** Every test has a size (its resource and time budget: small, medium, large, enormous, each with a default timeout; a large test gets 900 s) independent of what it tests. A test past its timeout fails. **For Tally:** layer (unit, integration, acceptance, load) × budget (seconds, 60 s, gate-only, engine-only). A capability check over its budget is a fault to fix, not a check to move.
5. **One generator, a scale knob, answers per scale (TPC-H).** The same 22 queries run at any scale factor. DuckDB ships expected answers at scale 0.01, 0.1 and 1 for correctness, and benchmarks at larger scales. **For Tally:** the key scripts are our generator and answer set. B's already takes its path count; each scale number becomes a knob, the capability version runs at the small setting and the load version at full size, and the text is the same.
6. **Feature tags on every test (test262, ECMAScript's conformance suite).** Each test carries YAML front matter with the features it needs, drawn from one `features.txt`. **For Tally:** the edge list and the keys' `edges:` lines already do this for the case set; extending them to the probes and the 79 old models makes the map machine-checked.
7. **Speed without touching the obviously right code.**
   - A faster runtime runs the same code: PyPy reports about 3–4× over CPython 3.11 on its own benchmark suite. It's unmeasured on refcore, and deep recursion (refcore's 512 MB thread stack) is the known risk.
   - A memo layer's known hazard is the *hidden dependency*: a lookup that reads something its cache key doesn't hold. Build systems answer it with checks that recompute and compare: Shake's lint, Bazel's sandbox, `nix-build --check`. That is the spot-check mode.
8. **Never rerun what didn't change (content-addressed test caching, Bazel).** A test result is keyed by the hash of all its inputs, and a test whose inputs are unchanged is not run again. **For Tally:** key every run by the hashes of the interpreter, the texts and the asks. Session 5's 1.5 hours of reruns would have been cut down to the runs whose inputs had changed.

## 4. The architecture these point to

- **One capability list** (the edge list) is the catalogue. Every test names the capabilities it uses (tags), and a machine check keeps the map whole.
- **Layers:**
  - **Unit:** probes; one capability, seconds each, about 2 min for the whole layer.
  - **Integration:** households plus the old models, each check in its capability version, under 60 s each, under 10 min for the layer on 6 cores. Coverage goal: every interacting pair.
  - **Acceptance:** the holdout; blind, exam only, never split into visible probes.
  - **Load:** the full-size twins; engine only.
  - **The interpreter's own tests** (950) stay with the interpreter; the both-ways check of any speed change runs there and on the unit layer.
- **The judge:** refcore as frozen. A faster run (PyPy, or a layer behind a switch) is accepted only if the 950 tests and every stored output match it byte for byte. Heavy runs use spot checks.
- **The runner:** a budget per layer (past it = fail), results cached by input hash, a before/after table.
- **The rule for anything new:** its capabilities go on the list, with a probe for each new one; its combinations go into an integration check, aimed at uncovered pairs; its scale goes into a load twin. The halving test decides what is scale.

## 5. Options

| | What | Keeps "obviously right" | Cost and risk |
|---|---|---|---|
| **A (pick)** | Runtime first. Measure refcore on PyPy: the 950 tests, the stored outputs byte for byte, one B ask's time. Then session 4.5 restructures. The memo layer is built only if PyPy falls short. | fully: the frozen code, unchanged | one download, minutes; risk that recursion or a speed-up under ~5× doesn't get B's asks under 60 s |
| B | The memo and index layer first (as discussed before this research), then session 4.5 | mostly: a switch, both-ways proof, spot checks | an agent job on the interpreter; the hidden-dependency risk |
| C | Session 4.5 restructures only; B-sized capability checks run in the background at gates | fully | no speed work, but slow integration runs forever |

**How 4.5 proves the new place is better:** one before/after table:
- each layer's wall time, before (snap16's summaries, today's runs) and after;
- the counts: capabilities, probes, integration checks, load twins;
- coverage: capabilities with a probe, interacting pairs met, every load twin with its capability version;
- the byte-for-byte equality on every stored output.

Session 5 then uses the same runner and the same breakdown on the holdout, with no new tooling.

## 6. Sources

- Cedar: "How We Built Cedar: A Verification-Guided Approach", arXiv 2407.01688 (FSE 2024 industry); AWS Open Source Blog, "Lean into verified software development"; Amazon Science, "How we built Cedar with automated reasoning and differential testing".
- ShardStore: Bornholt et al., "Using Lightweight Formal Methods to Validate a Key-Value Storage Node in Amazon S3", SOSP 2021.
- Kuhn, Wallace, Gallo, "Software Fault Interactions and Implications for Software Testing", IEEE TSE 30(6), 2004; NIST ACTS.
- Bazel Test Encyclopedia (test sizes and timeouts).
- DuckDB tpch extension (`tpch_answers()` at scale 0.01, 0.1, 1).
- test262 CONTRIBUTING.md (front matter, `features`).
- PyPy benchmark notes (pypy.org, speed.pypy.org).
- Jackson, *Software Abstractions* (small scope hypothesis); Runciman, Naylor, Lindblad, "SmallCheck and Lazy SmallCheck", Haskell Symposium 2008.
