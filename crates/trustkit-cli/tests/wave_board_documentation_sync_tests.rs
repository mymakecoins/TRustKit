// Integration test for TASK-006.
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use trustkit_cli::wave_board_documentation_sync::WaveBoardDocSyncValidator;

#[test]
fn test_task_006_wave_board_documentation_sync_validation() {
    let manual_md = "# User Manual\ntrustkit board up\ntk board status\ntrustkit wave board down";
    let skill_md = "# trustkit-sdd skill\nRun trustkit board down or tk board down";
    let invalid_doc = "# Other doc\nno board command here";

    let sources = [("manual.md", manual_md), ("skill.md", skill_md)];

    let missing = WaveBoardDocSyncValidator::audit_board_sources(&sources);
    assert!(
        missing.is_empty(),
        "All documentation sources should be valid"
    );

    assert!(!WaveBoardDocSyncValidator::verify_content_contains_board_and_tk(invalid_doc));
    assert!(WaveBoardDocSyncValidator::verify_content_contains_board_and_tk(manual_md));
    assert!(WaveBoardDocSyncValidator::verify_content_contains_board_and_tk(skill_md));
}

#[test]
fn test_task_006_legacy_wave_board_documentation_sync_validation() {
    let manual_md = "# User Manual\ntrustkit wave board up\ntk wave board status";
    let sources = [("manual.md", manual_md)];

    let missing = WaveBoardDocSyncValidator::audit_document_sources(&sources);
    assert!(missing.is_empty());
}
