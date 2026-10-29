use super::*;
use clap::CommandFactory;

fn is_emoji(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1F600..=0x1F64F
            | 0x1F300..=0x1F5FF
            | 0x1F680..=0x1F6FF
            | 0x1F700..=0x1F77F
            | 0x1F780..=0x1F7FF
            | 0x1F800..=0x1F8FF
            | 0x1F900..=0x1F9FF
            | 0x1FA00..=0x1FA6F
            | 0x1FA70..=0x1FAFF
            | 0x2600..=0x26FF
            | 0x2700..=0x27BF
    )
}

fn assert_clean_help(text: &str) {
    for ch in text.chars() {
        assert!(!is_emoji(ch), "Forbidden emoji '{ch}' found in help output: {text}");
    }
    let lower = text.to_lowercase();
    assert!(!lower.contains("co-authored-by: claude"), "AI trailer found in help");
    assert!(!lower.contains("co-authored-by: anthropic"), "AI trailer found in help");
    assert!(!lower.contains("claude-session"), "AI trailer found in help");
    assert!(!lower.contains("generated with"), "AI trailer found in help");
}

#[test]
fn test_all_subcommands_help_zero_emojis() {
    let mut root_cmd = Cli::command();
    let mut help_buf = Vec::new();
    root_cmd.write_help(&mut help_buf).expect("root help should render");
    let root_help = String::from_utf8(help_buf).expect("utf-8 help");
    assert_clean_help(&root_help);

    let test_invocations = [
        vec!["trustkit", "--help"],
        vec!["trustkit", "task", "--help"],
        vec!["trustkit", "task", "test", "--help"],
        vec!["trustkit", "task", "complete", "--help"],
        vec!["trustkit", "wave", "--help"],
        vec!["trustkit", "wave", "run", "--help"],
        vec!["trustkit", "gauntlet", "--help"],
        vec!["trustkit", "gauntlet", "run", "--help"],
        vec!["trustkit", "doctor", "--help"],
        vec!["trustkit", "done", "--help"],
        vec!["trustkit", "merge", "--help"],
        vec!["trustkit", "pr", "--help"],
        vec!["trustkit", "mcp", "--help"],
        vec!["trustkit", "mcp", "serve", "--help"],
    ];

    for invocation in test_invocations {
        let err = Cli::try_parse_from(invocation).expect_err("should return DisplayHelp error");
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
        assert_clean_help(&err.to_string());
    }
}

#[test]
fn test_agent_mode_env_override() {
    let previous = std::env::var("TRUSTKIT_AGENT_MODE").ok();
    
    // SAFETY: Single-threaded test setting environment variable for verification.
    unsafe {
        std::env::set_var("TRUSTKIT_AGENT_MODE", "1");
    }
    let cli = Cli::try_parse_from(["trustkit", "doctor"]).expect("parse doctor");
    assert!(cli.is_json_mode());

    // Restore env var
    unsafe {
        match previous {
            Some(val) => std::env::set_var("TRUSTKIT_AGENT_MODE", val),
            None => std::env::remove_var("TRUSTKIT_AGENT_MODE"),
        }
    }
}
