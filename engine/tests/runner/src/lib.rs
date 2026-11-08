//! T's test runner (`tally-runner`), the test owner's crate: the builder never edits it
//! (DONE rule 2). It calls the engine in process through `tally_cli::run`, the way
//! `engine/tests/INTERFACE.md` §4 says, so the tests need no binary and no file.
//!
//! - [`corpus`]: the kernel's property corpus and refcore's outputs for it, embedded;
//! - [`output`]: reading what an ask prints;
//! - [`laws`]: the kernel's six laws (`engine/tests/PROPERTIES.md`), proven on refcore's
//!   outputs by this crate's own tests, which run in `check full` now;
//! - [`laws_m0`]: M0's four laws and idempotence (`PROPERTIES.md` § M0), proven the same way
//!   on refcore's runs of `properties/m0.corpus`;
//! - [`household`]: household X's base as milestone S runs it, and HH4's hand-checked
//!   vectors restated on it (`hh4/vectors.tsv`), proven on refcore's outputs the same way;
//! - [`codes`]: T's fixtures of the codes no check prints (`codes/fixtures.tsv`), `LOAD-PATH`
//!   first;
//! - [`kernel`]: milestone S's kernel vectors, refcore's whole output for each
//!   (`kernel/vectors.refcore`, issue #41);
//! - [`m0`]: milestone M0's vectors, refcore's reprint of every text set
//!   (`m0/format.refcore`) and its run of the load probe on each (`m0/load.refcore`, issue
//!   #56) and of row 3's realities (`m0/loadcheck.refcore`), and [`m0_texts`] and
//!   [`m0_spec_texts`], the texts they name;
//! - [`m1`]: milestone M1's vectors, refcore's whole output for each (`m1/vectors.refcore`,
//!   issue #108), and #109's text with the law it is held to, a canonical text reprints as
//!   itself;
//! - [`engine`]: a run on the engine, in process;
//! - [`held`]: the gating vectors T holds "for #n" (`held.tsv`), which the targets leave out.
//!
//! The tests that run the engine are each milestone's test targets (`milestone/s_properties.rs`,
//! `milestone/s_household.rs` and `milestone/s_kernel.rs` for S, `milestone/m0_codes.rs`,
//! `milestone/m0_format.rs`, `milestone/m0_load.rs`, `milestone/m0_loadcheck.rs` and
//! `milestone/m0_properties.rs` for M0, `milestone/m1_checks.rs`, `milestone/m1_format.rs`,
//! `milestone/m1_goldens.rs` and `milestone/m1_codes.rs` for M1),
//! with `test = false`:
//! red until their milestone is built. The marker `engine/tools/milestones.toml` gates a
//! milestone's targets once it is tagged, and `check` reports the next milestone's, never
//! failing (`MILESTONE-S.md` § Keeping master green). Run one by name:
//! `cargo nextest run -p tally-runner --test s_household`.

#![forbid(unsafe_code)]

pub mod codes;
pub mod corpus;
pub mod engine;
pub mod held;
pub mod household;
pub mod kernel;
pub mod laws;
pub mod laws_m0;
pub mod m0;
pub mod m0_spec_texts;
pub mod m0_texts;
pub mod m1;
pub mod output;
