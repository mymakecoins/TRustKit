use super::*;

#[test]
fn test_parse_wave_plan() {
    let args = [
        "trustkit",
        "wave",
        "plan",
        "--tasks-dir",
        "specs/001-feat/tasks",
        "--json",
    ];
    let cli = Cli::try_parse_from(args).expect("parse should succeed");
    assert!(cli.is_json_mode());
    match cli.command {
        Command::Wave(WaveArgs {
            action: WaveSubcommand::Plan(args),
        }) => {
            assert_eq!(args.tasks_dir.as_deref(), Some("specs/001-feat/tasks"));
            assert!(args.json);
        }
        _ => panic!("Expected wave plan command"),
    }
}

#[test]
fn test_parse_spec_lock() {
    let args = [
        "trustkit",
        "spec",
        "lock",
        "--spec",
        "specs/001-feat/feat-spec.md",
        "--json",
    ];
    let cli = Cli::try_parse_from(args).expect("parse should succeed");
    assert!(cli.is_json_mode());
    match cli.command {
        Command::Spec(SpecArgs {
            action: SpecSubcommand::Lock(args),
        }) => {
            assert_eq!(args.spec.as_deref(), Some("specs/001-feat/feat-spec.md"));
            assert!(args.json);
        }
        _ => panic!("Expected spec lock command"),
    }
}

#[test]
fn test_parse_spec_init() {
    let args = [
        "trustkit",
        "spec",
        "init",
        "--from-prd",
        "docs/PRD.md",
        "--name",
        "my-feature",
        "--output-dir",
        "specs/001-feat",
        "--json",
    ];
    let cli = Cli::try_parse_from(args).expect("parse should succeed");
    assert!(cli.is_json_mode());
    match cli.command {
        Command::Spec(SpecArgs {
            action: SpecSubcommand::Init(args),
        }) => {
            assert_eq!(args.from_prd, "docs/PRD.md");
            assert_eq!(args.name.as_deref(), Some("my-feature"));
            assert_eq!(args.output_dir.as_deref(), Some("specs/001-feat"));
            assert!(args.json);
        }
        _ => panic!("Expected spec init command"),
    }
}

#[test]
fn test_parse_init_with_issue() {
    let args = ["trustkit", "init", "--issue", "45", "--json"];
    let cli = Cli::try_parse_from(args).expect("parse should succeed");
    assert!(cli.is_json_mode());
    assert_eq!(
        cli.command,
        Command::Init(InitArgs {
            issue: Some("45".to_string()),
            from_prd: None,
            json: true,
        })
    );
}

#[test]
fn test_parse_status() {
    let args = ["trustkit", "status"];
    let cli = Cli::try_parse_from(args).expect("parse should succeed");
    assert!(!cli.is_json_mode());
    assert_eq!(cli.command, Command::Status(StatusArgs { json: false }));
}

#[test]
fn test_parse_hook_pre_tool() {
    let args = [
        "trustkit",
        "hook",
        "pre-tool",
        "--command",
        "git add .",
        "--json",
    ];
    let cli = Cli::try_parse_from(args).expect("parse should succeed");
    assert!(cli.is_json_mode());
    match cli.command {
        Command::Hook(HookArgs {
            action: HookSubcommand::PreTool(pre_tool),
        }) => {
            assert_eq!(pre_tool.command, "git add .");
            assert!(pre_tool.json);
        }
        _ => panic!("Expected hook pre-tool command"),
    }
}
