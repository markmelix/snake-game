//! Snake game library.
//!

#![warn(missing_docs)]
//#![warn(missing_doc_code_examples)]
#![allow(dead_code)]
#![allow(clippy::unused_io_amount)]

pub mod game;
pub mod server;

/// This is an alias for standart [`Result`](std::result::Result) type which
/// represents failure.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;