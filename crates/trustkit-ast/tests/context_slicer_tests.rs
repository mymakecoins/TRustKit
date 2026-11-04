// Unit tests for AstContextSlicer (TASK-008).
// Constitutional Principle III: Test-First & Anti-Cheat.

use trustkit_ast::AstContextSlicer;
use trustkit_core::entities::ContextSliceRequest;

#[test]
fn test_slice_rust_public_signatures() {
    let slicer = AstContextSlicer::new();
    let source = r#"
        pub struct User {
            pub id: u64,
            pub name: String,
        }

        impl User {
            pub fn new(id: u64, name: String) -> Self {
                let formatted = format!("{name}");
                Self { id, name: formatted }
            }

            fn internal_helper(&self) {
                println!("private");
            }
        }
    "#;

    let req = ContextSliceRequest {
        target_files: vec!["src/user.rs".to_string()],
        symbol_filters: vec!["User".to_string()],
        max_line_budget: 100,
    };

    let result = slicer.slice_code("src/user.rs", source, &req);
    assert!(result.sliced_content.contains("pub struct User"));
    assert!(result.sliced_content.contains("pub fn new"));
    assert!(!result.sliced_content.contains("fn internal_helper"));
    assert!(result.total_lines > 0);
    assert_eq!(result.preserved_symbols, vec!["User".to_string()]);
}

#[test]
fn test_slice_respects_line_budget() {
    let slicer = AstContextSlicer::new();
    let source = (1..=200)
        .map(|i| format!("pub struct Item{} {{ pub val: usize }}", i))
        .collect::<Vec<_>>()
        .join("\n");

    let req = ContextSliceRequest {
        target_files: vec!["src/items.rs".to_string()],
        symbol_filters: vec![],
        max_line_budget: 15,
    };

    let result = slicer.slice_code("src/items.rs", &source, &req);
    assert!(result.total_lines <= 16);
    assert!(
        result
            .sliced_content
            .contains("// [Context truncated to stay within budget]")
    );
}
