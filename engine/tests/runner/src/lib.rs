//! T's test runner (`tally-runner`), the test owner's crate: the builder never edits it
//! (DONE rule 2). It calls the engine in process through `tally_cli::run`, the way
//! `engine/tests/INTERFACE.md` §4 says, so the tests need no binary and no file.
//!
//! - [`corpus`]: the kernel's property corpus and refcore's outputs for it, embedded;
//! - [`output`]: reading what an ask prints;
//! - [`laws`]: the kernel's six laws (`engine/tests/PROPERTIES.md`), proven on refcore's
//!   outputs by this crate's own tests, which run in `check full` now;
//! - [`household`]: household X's base as milestone S runs it, and HH4's hand-checked
//!   vectors restated on it (`hh4/vectors.tsv`), proven on refcore's outputs the same way;
//! - [`codes`]: T's fixtures of the codes no check prints (`codes/fixtures.tsv`), `LOAD-PATH`
//!   first;
//! - [`engine`]: a run on the engine, in process.
//!
//! The tests that run the engine are each milestone's test targets (`milestone/s_properties.rs`
//! and `milestone/s_household.rs` for S, `milestone/m0_codes.rs` for M0), with `test = false`:
//! red until their milestone is built. The marker `engine/tools/milestones.toml` gates a
//! milestone's targets once it is tagged, and `check` reports the next milestone's, never
//! failing (`MILESTONE-S.md` § Keeping master green). Run one by name:
//! `cargo nextest run -p tally-runner --test s_household`.

#![forbid(unsafe_code)]

pub mod codes;
pub mod corpus;
pub mod engine;
pub mod household;
pub mod laws;
pub mod output;
