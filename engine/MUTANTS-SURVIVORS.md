# The mutants that survive

Every mutant that survives T's tests (missed, or timed out) is a row here, by file,
function and mutation, with its reason naming its rule (R-87; DONE gate 8). The
nightly `mutants survivors` job runs `engine/tools/mutants-survivors` over the shards'
output, and fails on a survivor that isn't listed, a listed one that is now caught or
gone, and a row without a reason. The mutation is cargo-mutants' description without
the line and column.

The builder adds a row in the commit that leaves the mutant alive, and the review
reads every one.

| file | function | mutation | reason |
| ---- | -------- | -------- | ------ |
