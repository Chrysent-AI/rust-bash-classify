//! Library scaffold for the Rust port of bash-classify.
//!
//! Later milestones will fill in the parity modules that mirror the Python
//! implementation (`models`, `parser`, `matcher`, `classifier`, `database`,
//! and `cli`). M0 only establishes the crate and vendored data plumbing.

#[allow(dead_code)]
pub mod commands_data;
