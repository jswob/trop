//! The Tally engine: a reality goes in as bytes, a result comes out (ENGINE §1).
//!
//! A stub. Every entry point answers [`NotImplemented`], and the build session
//! replaces it. Nothing here is a design choice the spec owns.

#![forbid(unsafe_code)]

#[cfg(all(feature = "fault-injection", not(debug_assertions)))]
compile_error!("the fault-injection feature exists only in builds with debug assertions (R-51)");

/// The answer of every entry point until the engine is built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotImplemented;

/// Takes a source text as bytes (the library reads no files, R-39) and answers
/// [`NotImplemented`]. The fuzz target `parse` calls it.
pub fn parse(_source: &[u8]) -> Result<(), NotImplemented> {
    Err(NotImplemented)
}
