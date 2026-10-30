//! TRustKit Core: Pure Domain Crate
//!
//! Constitutional Principle I: Zero external dependencies (std only).
//! Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.

pub mod ast_entities;
pub mod entities;
pub mod errors;
pub mod gate_entities;
pub mod gauntlet_entities;
pub mod gates_fsm;
pub mod graph_entities;
pub mod invariants_policy;
pub mod knowledge_entities;
pub mod port_dtos;
pub mod ports;
pub mod spec_entities;
pub mod special_cases;
pub mod use_cases;
