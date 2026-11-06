// Tests for Gauntlet Run headless and non-interactive behavior (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_cli::commands::gauntlet_run_cmd::load_project_security_policy;
use trustkit_core::entities::{
    CliSecurityReviewFlag, SecurityReviewDecision, SecurityReviewMode, SecuritySkipReason,
};
use trustkit_core::security_review_policy::{
    SecurityReviewPolicyConfig, resolve_security_review_decision,
};

#[test]
fn test_task_003_load_project_security_policy_from_workspace_root() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let root = temp.path();
    let toml_path = root.join("trustkit.toml");
    std::fs::write(
        &toml_path,
        "[governance.gauntlet]\nsecurity_review_mode = \"always\"\nmin_confidence_threshold = 9\n",
    )
    .expect("write toml");

    let sub_dir = root.join("docs").join("specs").join("cycle-123");
    std::fs::create_dir_all(&sub_dir).expect("create sub_dir");

    let config = load_project_security_policy(&sub_dir);
    // Should inspect parent/workspace root and find the custom configuration
    assert_eq!(config.mode, SecurityReviewMode::Always);
    assert_eq!(config.min_confidence_threshold, 9);
}

#[test]
fn test_task_003_headless_non_interactive_skips_security_review_prompt() {
    let policy = SecurityReviewPolicyConfig {
        mode: SecurityReviewMode::Prompt,
        min_confidence_threshold: 8,
    };

    // When is_interactive_tty is false (headless/agent environment), decision should automatically skip without blocking
    let decision = resolve_security_review_decision(
        CliSecurityReviewFlag::Unspecified,
        &policy,
        false, // non-interactive tty
        false, // not json mode
    );

    assert_eq!(
        decision,
        SecurityReviewDecision::Skip {
            reason: SecuritySkipReason::HeadlessNonInteractiveDefault,
        }
    );
}
