// Integration tests for AstGraphExtractor (TASK-002, RF-026, RN-002).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle IV: Zero Panic.

use std::time::Instant;
use trustkit_ast::AstGraphExtractor;
use trustkit_core::entities::{EdgeConfidence, EdgeRelationKind, GraphNodeKind};

#[test]
fn test_extract_rust_public_symbols_and_imports() {
    let extractor = AstGraphExtractor::new();
    let file_path = "crates/trustkit-core/src/spec_validator.rs";
    let rust_code = r#"
        use crate::ports::SpecValidatorPort;
        use std::sync::Arc;

        pub struct SpecValidator {
            pub port: Arc<dyn SpecValidatorPort>,
        }

        struct PrivateInternalHelper;

        impl SpecValidator {
            pub fn validate_now(&self) -> bool {
                true
            }
        }
    "#;

    let start = Instant::now();
    let (nodes, edges) = extractor.extract_from_source(file_path, rust_code);
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 5, "Extraction took {}ms (budget < 5ms)", elapsed.as_millis());

    // Verify CodeFile node
    let file_node = nodes.iter().find(|n| n.node_id == file_path);
    assert!(file_node.is_some(), "File node must be present");
    let file_node = file_node.unwrap();
    assert_eq!(file_node.kind, GraphNodeKind::CodeFile);
    assert_eq!(file_node.architectural_layer, "CORE");

    // Verify PublicSymbol node for SpecValidator
    let struct_node = nodes.iter().find(|n| n.label == "SpecValidator");
    assert!(struct_node.is_some(), "SpecValidator public symbol must be extracted");
    let struct_node = struct_node.unwrap();
    assert_eq!(struct_node.kind, GraphNodeKind::PublicSymbol);
    assert_eq!(struct_node.source_file, file_path);

    // Private struct must NOT be extracted as PublicSymbol
    let private_node = nodes.iter().find(|n| n.label == "PrivateInternalHelper");
    assert!(private_node.is_none(), "PrivateInternalHelper must not be extracted as PublicSymbol");

    // Verify Imports edge for SpecValidatorPort
    let import_edge = edges.iter().find(|e| e.target_id.contains("SpecValidatorPort"));
    assert!(import_edge.is_some(), "Import edge for SpecValidatorPort must be present");
    let import_edge = import_edge.unwrap();
    assert_eq!(import_edge.relation, EdgeRelationKind::Imports);
    assert_eq!(import_edge.confidence, EdgeConfidence::Extracted);
    assert_eq!(import_edge.source_file, file_path);

    // Verify DefinesSymbol edge
    let defines_edge = edges.iter().find(|e| e.relation == EdgeRelationKind::DefinesSymbol && e.target_id == struct_node.node_id);
    assert!(defines_edge.is_some(), "DefinesSymbol edge from file to public struct must be present");
}

#[test]
fn test_extract_rust_multiple_public_kinds() {
    let extractor = AstGraphExtractor::new();
    let file_path = "crates/trustkit-ast/src/demo_models.rs";
    let rust_code = r#"
        pub mod submod;
        pub enum ExecutionStatus { Active, Done }
        pub trait WorkerTask { fn execute(&self); }
        pub fn run_worker() {}
        pub type WorkerId = String;
        fn private_worker() {}
    "#;

    let (nodes, edges) = extractor.extract_from_source(file_path, rust_code);

    let labels: Vec<&str> = nodes.iter().filter(|n| n.kind == GraphNodeKind::PublicSymbol).map(|n| n.label.as_str()).collect();
    assert!(labels.contains(&"ExecutionStatus"), "Missing ExecutionStatus");
    assert!(labels.contains(&"WorkerTask"), "Missing WorkerTask");
    assert!(labels.contains(&"run_worker"), "Missing run_worker");
    assert!(labels.contains(&"WorkerId"), "Missing WorkerId");
    assert!(labels.contains(&"submod"), "Missing submod");
    assert!(!labels.contains(&"private_worker"), "Private fn must not be extracted");

    // Verify mod submod imports edge
    let mod_edge = edges.iter().find(|e| e.relation == EdgeRelationKind::Imports && e.target_id == "submod");
    assert!(mod_edge.is_some(), "mod submod must generate Imports edge");
}

#[test]
fn test_extract_typescript_exports_and_imports() {
    let extractor = AstGraphExtractor::new();
    let file_path = "crates/frontend/src/api_client.ts";
    let ts_code = r#"
        import { Config } from './config';
        import axios from 'axios';

        export interface ApiResponse {
            status: number;
            data: unknown;
        }

        export type ApiToken = string;

        export class ApiClient {
            fetchData(): void {}
        }

        export function createClient(): ApiClient {
            return new ApiClient();
        }

        const internalVar = 42;
    "#;

    let start = Instant::now();
    let (nodes, edges) = extractor.extract_from_source(file_path, ts_code);
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 5, "TS extraction took {}ms (budget < 5ms)", elapsed.as_millis());

    let file_node = nodes.iter().find(|n| n.node_id == file_path).expect("File node must exist");
    assert_eq!(file_node.kind, GraphNodeKind::CodeFile);

    let public_labels: Vec<&str> = nodes.iter().filter(|n| n.kind == GraphNodeKind::PublicSymbol).map(|n| n.label.as_str()).collect();
    assert!(public_labels.contains(&"ApiResponse"), "Missing ApiResponse interface");
    assert!(public_labels.contains(&"ApiToken"), "Missing ApiToken type");
    assert!(public_labels.contains(&"ApiClient"), "Missing ApiClient class");
    assert!(public_labels.contains(&"createClient"), "Missing createClient function");
    assert!(!public_labels.contains(&"internalVar"), "Internal variable must not be public symbol");

    // Check imports
    let relative_import = edges.iter().find(|e| e.target_id.contains("config")).expect("Relative import must exist");
    assert_eq!(relative_import.relation, EdgeRelationKind::Imports);
    assert_eq!(relative_import.confidence, EdgeConfidence::Inferred);

    let pkg_import = edges.iter().find(|e| e.target_id == "axios").expect("Package import must exist");
    assert_eq!(pkg_import.relation, EdgeRelationKind::Imports);
    assert_eq!(pkg_import.confidence, EdgeConfidence::Extracted);
}

#[test]
fn test_extract_markdown_wikilinks_unification() {
    let extractor = AstGraphExtractor::new();
    let file_path = ".trustkit/wiki/architecture/overview.md";
    let md_content = r#"---
title: "Architecture Overview"
tags: [arch, rust]
updated_at: "2026-10-29"
---

# Architecture Overview

See [[hexagonal-core]] and [[specs/gate-lifecycle|Gate Lifecycle]].
Also inline `[[ignored-code]]`.
"#;

    let (nodes, edges) = extractor.extract_from_source(file_path, md_content);

    let note_node = nodes.iter().find(|n| n.node_id == file_path).expect("KnowledgeNote node must exist");
    assert_eq!(note_node.kind, GraphNodeKind::KnowledgeNote);
    assert_eq!(note_node.label, "Architecture Overview");
    assert_eq!(note_node.architectural_layer, "WIKI");

    let wiki_edges: Vec<&str> = edges.iter()
        .filter(|e| e.relation == EdgeRelationKind::WikiLinksTo)
        .map(|e| e.target_id.as_str())
        .collect();

    assert!(wiki_edges.contains(&"hexagonal-core"), "Missing hexagonal-core link");
    assert!(wiki_edges.contains(&"specs/gate-lifecycle"), "Missing specs/gate-lifecycle link");
    assert!(!wiki_edges.contains(&"ignored-code"), "Ignored code link must not be extracted");
}

#[test]
fn test_extract_from_files_batch() {
    let extractor = AstGraphExtractor::new();
    let files = vec![
        ("crates/trustkit-core/src/model_a.rs".to_string(), "pub struct Alpha;".to_string()),
        ("crates/trustkit-core/src/model_b.rs".to_string(), "use crate::model_a::Alpha; pub struct Beta;".to_string()),
    ];

    let (nodes, edges) = extractor.extract_from_in_memory_files(&files);
    assert_eq!(nodes.iter().filter(|n| n.kind == GraphNodeKind::PublicSymbol).count(), 2);
    assert!(edges.iter().any(|e| e.relation == EdgeRelationKind::Imports && e.target_id.contains("Alpha")));
}
