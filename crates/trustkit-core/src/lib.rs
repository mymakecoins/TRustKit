//! TRustKit Core: Pure Domain Crate
//!
//! Constitutional Principle I: Zero external dependencies (std only).
//! Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.

pub mod entities;
pub mod errors;
pub mod gates_fsm;
pub mod invariants_policy;
pub mod ports;
pub mod special_cases;
pub mod use_cases;
