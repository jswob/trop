# The mutants that survive

Every mutant that survives T's tests (missed, or timed out) is a row here, by file,
function and mutation, with its reason in one of two forms (R-87, approved 2026-11-03;
DONE gate 8), each followed by at least 20 characters:

- `R-87: equivalent: ` and why no valid input can tell the mutant from the code. A proven
  equivalent leaves the score's count.
- `R-87: missed: ` and why no test tells it apart yet, though one could: an input the
  engine at S refuses as not built, or one where refcore and the spec disagree. It stays
  counted as missed.

The score is caught ÷ (caught + missed + timed out − equivalent), where equivalent counts
the surviving mutants an `R-87: equivalent:` row matches: one row covers every mutant
with its file, function and mutation (the same mutation at two columns is two mutants),
so a key is listed once. Unviable mutants are reported, not counted. The nightly
`mutants survivors` job runs `engine/tools/mutants-survivors` over the shards' output,
prints `score S% = caught C / (caught C + missed M + timed out T - equivalent E)`, and
fails on a score under 90 %, a survivor that isn't listed, a listed one that is now
caught or gone, a key listed twice, and a row without a reason. The mutation is
cargo-mutants' description without the line and column.

The builder adds a row in the commit that leaves the mutant alive, and the review
reads every one: an equivalent row a test could tell apart is a score inflated by
bookkeeping.

| file | function | mutation | reason |
| ---- | -------- | -------- | ------ |
| crates/tally/src/walk/day.rs | Engine::book_leg_ends | replace < with <= in Engine::book_leg_ends | R-87: equivalent: a booked end's sign is -1 for the leg's from end and 1 for its to end, never 0, so `< 0` and `<= 0` pick the same other end |
