//! TRustKit Core: Pure Domain Crate
//!
//! Constitutional Principle I: Zero external dependencies (std only).
//! Constitutional Principle II: Absolute Prohibition of NULL & Special Cases.

pub mod ast_entities;
pub mod certification_merge_pr_use_case;
pub mod dual_axis_entities;
pub mod dual_axis_scope_creep;
pub mod entities;
pub mod errors;
pub mod gate_entities;
pub mod gates_fsm;
pub mod gauntlet_entities;
pub mod glossary_entities;
pub mod graph_entities;
pub mod harness_retro_engine;
pub mod invariants_policy;
pub mod knowledge_entities;
pub mod milestone_entities;
pub mod module_depth_entities;
pub mod next_step_guidance;
pub mod next_step_status_policy;
pub mod osc7501_encoder;
pub mod port_dtos;
pub mod ports;
pub mod security_review_policy;
pub mod semantic_review_entities;
pub mod spec_entities;
pub mod spec_prompt_builder;
pub mod spec_scaffold_entities;
pub mod special_cases;
pub mod terminal_multiplexer_entities;
pub mod tool_caching_policy;
pub mod update_entities;
pub mod use_cases;
