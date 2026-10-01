//! Test utilities shared across the workspace.
//!
//! A deterministic clock and in-memory fakes, kept in a dedicated crate so they
//! never ship in production builds (M-INTEGRATION-TEST-UTILS).

pub mod clock;
pub mod fake;
