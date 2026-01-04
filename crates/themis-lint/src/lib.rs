//! # Themis Lint
//!
//! Linting rules for Themis contract governance.
//!
//! This crate provides configurable lint rules that enforce best practices
//! and Themis conventions on API contracts.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod reporter;
pub mod rules;

pub use reporter::{LintReport, LintReporter};
