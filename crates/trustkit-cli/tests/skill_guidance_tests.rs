// Tests for trustkit-sdd skill guidance on RED test authoring (TASK-004).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle III: Test-First & Anti-Cheat.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use std::fs;
use std::path::Path;
use trustkit_ast::InvariantLexicalScanner;

#[test]
fn test_gate4_skill_guidance_documents_red_test_authoring_step() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let skill_path = repo_root.join(".agents/skills/trustkit-sdd/SKILL.md");
    assert!(
        skill_path.exists(),
        "Skill file must exist at .agents/skills/trustkit-sdd/SKILL.md"
    );

    let content = fs::read_to_string(&skill_path).expect("read skill file");

    let gate4_pos = content
        .find("### Gate 4:")
        .expect("Gate 4 section must exist");
    let gate5_pos = content.find("### Gate 5:").unwrap_or(content.len());
    let gate4_section = &content[gate4_pos..gate5_pos];

    assert!(
        gate4_section.contains("target_test_file"),
        "Gate 4 documentation must reference target_test_file for RED authoring"
    );

    assert!(
        gate4_section.contains("autoria")
            || gate4_section.contains("redigir")
            || gate4_section.contains("escrever"),
        "Gate 4 documentation must explicitly guide authoring the failing test before task test --phase red"
    );

    let wave_run_pos = gate4_section
        .find("trustkit wave run")
        .expect("wave run command must be in Gate 4");
    let task_test_red_pos = gate4_section
        .find("trustkit task test <TASK_ID> --phase red")
        .expect("task test --phase red command must be in Gate 4");

    assert!(
        wave_run_pos < task_test_red_pos,
        "wave run must precede task test --phase red in Gate 4"
    );

    let intermediate = &gate4_section[wave_run_pos..task_test_red_pos];
    assert!(
        intermediate.contains("target_test_file"),
        "The step between wave run and task test --phase red must document authoring in target_test_file"
    );

    let scanner = InvariantLexicalScanner;
    let violations = scanner.scan_content(skill_path.to_str().unwrap(), &content);
    assert!(
        violations.is_empty(),
        "Skill file contains lexical invariant violations: {:?}",
        violations
    );
}
