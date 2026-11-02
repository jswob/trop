# The mutants that survive

Every mutant that survives T's tests (missed, or timed out) is a row here, by file,
function and mutation, with the reason it is equivalent (DONE gate 8; R-87): the form
`R-87: equivalent: ` and at least 20 characters saying why no test can tell the mutant
from the code. The
nightly `mutants survivors` job runs `engine/tools/mutants-survivors` over the shards'
output, and fails on a survivor that isn't listed, a listed one that is now caught or
gone, and a row without a reason. The mutation is cargo-mutants' description without
the line and column.

The builder adds a row in the commit that leaves the mutant alive, and the review
reads every one.

| file | function | mutation | reason |
| ---- | -------- | -------- | ------ |
| crates/tally/src/walk/day.rs | Engine::book_leg_ends | replace < with <= in Engine::book_leg_ends | R-87: equivalent: a booked end's sign is -1 for the leg's from end and 1 for its to end, never 0, so `< 0` and `<= 0` pick the same other end |
