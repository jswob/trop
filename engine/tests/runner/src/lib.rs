//! T's test runner (`tally-runner`), the test owner's crate: the builder never edits it
//! (DONE rule 2). It calls the engine in process through `tally_cli::run`, the way
//! `engine/tests/INTERFACE.md` §4 says, so the tests need no binary and no file.
//!
//! - [`corpus`]: the kernel's property corpus and refcore's outputs for it, embedded;
//! - [`output`]: reading what an ask prints;
//! - [`laws`]: the kernel's six laws (`engine/tests/PROPERTIES.md`), proven on refcore's
//!   outputs by this crate's own tests, which run in `check full` now;
//! - [`engine`]: one corpus record run on the engine.
//!
//! The tests that run the engine are milestone S's (`milestone/s_properties.rs`), a test
//! target with `test = false`: red on the stub, so not run by default until the marker
//! that gates milestones exists (`MILESTONE-S.md` § Keeping master green). Run them with
//! `cargo nextest run -p tally-runner --test s_properties`.

#![forbid(unsafe_code)]

pub mod corpus;
pub mod engine;
pub mod laws;
pub mod output;
