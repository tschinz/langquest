//! LangQuest (`lq`) — interactive programming exercises in the terminal.
//!
//! This library crate re-exports the public modules so that integration tests
//! (and any future downstream consumers) can access the core types and
//! functions without reaching into the binary crate.

#![deny(clippy::all)]

pub mod app;
pub mod config;
pub mod error;
pub mod exercise;
pub mod runner;
pub mod ui;
