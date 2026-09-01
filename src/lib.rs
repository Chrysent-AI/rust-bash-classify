//! Library scaffold for the Rust port of bash-classify.
//!
//! Parity modules will mirror the Python implementation (`models`, `parser`,
//! `matcher`, `classifier`, `database`, and `cli`). The `commands_data` module
//! is generated from `vendor/commands/*.yaml`, the counterpart of the Python
//! package's `commands/` directory. It is owned by `scripts/gen_commands_data.sh`
//! and must not be edited by hand.

#[allow(dead_code)]
pub mod commands_data;
pub mod database;
pub mod matcher;
pub mod models;
pub mod parser;

pub use database::{load_database, CommandDatabase};
pub use matcher::{match_command, MatchError};
pub use models::{Classification, CommandResult, ExpressionResult, Risk};
pub use parser::{parse_expression, ParseError};
