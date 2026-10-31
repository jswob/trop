# The household's 7 changes: expected outputs

These are the answer keys for DONE gate 3's second half: "each of the 7 changes gives the outputs predicted for it, by hand, before the build". They are new in tally 1.0.1. The change files (`amendments/changes/`) keep their "in words" sections as illustrations; where a change file and its expected file differ, the expected file holds.

## How each key was made

- Two evaluators worked each change by hand, blind to each other, from the frozen spec (tally 1.0) and the household files alone. Every figure is a one-line `python3 -c` with `Decimal` or `Fraction`, printed with its output in the evaluator's file.
- The clean-side lead compared the pair cell by cell and settled each disagreement in writing. The comparisons and the evaluators' files are the provenance; they sit outside the spec, in `tmp/design/clean/changes/` (`C<n>-a.md`, `C<n>-b.md`, `C<n>-compare.md`).
- No engine ran. Nothing under `tmp/holdout/` was read.

## What a key holds

1. **How the change is applied:** the amendments in order. Every change but the scenario is an `advance` of `on` followed by one or more `add`s (`amendments/README.md`, "Advancing `on`").
2. **The cells.** The household's asks (`asks/core.tally`) give 41 cells:

   | Ask | Cells |
   |---|---|
   | wife_net | 2 |
   | social_and_health | 6 (three months, two values each) |
   | replay_september | 1 (the ledger) |
   | explain_room | 2 (two walk points) |
   | konto_balance | 8 (four dates, two expressions) |
   | net_worth | 4 |
   | ike_left, pit28, health_settlement, taxes_paid | 3 each |
   | pit37 | 6 (two people, three years) |

   A change that adds a person or an ask adds cells, marked "new". **Before** is the base's value, `smoke/HH4-b.md` §10. **After** is the value at the change's final cut.
3. **Status:**
   - **exact:** the printed value, to the cent.
   - **hole:** the cell prints `?`, with the questions listed.
   - **run-only:** the walk to 2059 is too long to work by hand. The key gives the direction against the base, and a bound where one was derived. An engine's figure is checked against that direction, not against a number.
4. **The first day anything differs** (over movements and balances, `OPERATIONS.md` §7 step 6) and the sanity verdict.
5. **Warnings and holes** that the change adds or clears.
6. **The register:** the entries the change adds or removes, as a delta. The full register text of each line is in the evaluators' files, under the readings named in the key.
7. **Variants.** A change whose first-written diff had a defect keeps that version as a second variant, "as first written", because it tests how a hole spreads. Its diff is the 1.0 diff; the change file holds the fixed one.
8. **The readings it rests on:** the 1.0.1 patches that settle a question the cells depend on (`CHANGES.md`).

## The premise for law versions

Changes 2, 3 and 5 add law versions, and a law part is read only once it is `tested` (`OPERATIONS.md` §3). For these keys, the store holds every part under `law/pl/`, and change 5's `law/de/`, as `tested` against its hash (`amendments/README.md`, "Not gated law"). That is a fixture premise; the status lives outside the content (`OPERATIONS.md` §1). The gate itself is tested on its own fixture (`BUILD-ORDER.md`).
