// Tests for trustkit-sdd Thin Skill integrity (TASK-007).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use std::path::Path;
use trustkit_ast::InvariantLexicalScanner;

#[test]
fn test_trustkit_sdd_skill_contains_native_spec_workflows_and_zero_emojis() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let skill_path = repo_root.join(".agents/skills/trustkit-sdd/SKILL.md");
    assert!(skill_path.exists(), "Skill file must exist at .agents/skills/trustkit-sdd/SKILL.md");

    let content = fs::read_to_string(&skill_path).expect("read skill file");

    // Invariantes de conteudo nativo de especificacao
    assert!(content.contains("trustkit spec prompt"), "Must document spec prompt command");
    assert!(content.contains("trustkit spec lint"), "Must document spec lint command");
    assert!(content.contains("trustkit spec lock"), "Must document spec lock command");
    assert!(content.contains("trustkit_spec_template"), "Must document MCP spec template tool");
    assert!(content.contains("trustkit_spec_context"), "Must document MCP spec context tool");
    assert!(content.contains("trustkit_spec_lint"), "Must document MCP spec lint tool");
    assert!(content.contains("trustkit_wave_status"), "Must document MCP wave status tool");
    assert!(content.contains("trustkit_terminal_inspect"), "Must document MCP terminal inspect tool");
    assert!(content.contains("trustkit_terminal_send"), "Must document MCP terminal send tool");

    // Validacao lexica rigorosa de invariantes: zero emojis e zero trailers de IA
    let scanner = InvariantLexicalScanner;
    let violations = scanner.scan_content(skill_path.to_str().unwrap(), &content);
    assert!(violations.is_empty(), "Skill file contains lexical invariant violations: {:?}", violations);
}
