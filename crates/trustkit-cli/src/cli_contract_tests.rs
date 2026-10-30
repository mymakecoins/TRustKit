use super::*;

#[test]
fn test_parse_task_test_red_and_green() {
    let args_red = ["trustkit", "task", "test", "TASK-001", "--phase", "red"];
    let cli_red = Cli::try_parse_from(args_red).expect("parse red should succeed");
    assert!(!cli_red.is_json_mode());
    match cli_red.command {
        Command::Task(TaskArgs {
            action: TaskSubcommand::Test(args),
        }) => {
            assert_eq!(args.task_id, "TASK-001");
            assert_eq!(args.phase, TddPhaseArg::Red);
            assert_eq!(trustkit_core::gauntlet_entities::TddPhase::from(args.phase), trustkit_core::gauntlet_entities::TddPhase::Red);
            assert!(!args.json);
        }
        _ => panic!("Expected task test command"),
    }

    let args_green = ["trustkit", "--json", "task", "test", "TASK-002", "--phase", "green"];
    let cli_green = Cli::try_parse_from(args_green).expect("parse green should succeed");
    assert!(cli_green.is_json_mode());
    match cli_green.command {
        Command::Task(TaskArgs {
            action: TaskSubcommand::Test(args),
        }) => {
            assert_eq!(args.task_id, "TASK-002");
            assert_eq!(args.phase, TddPhaseArg::Green);
            assert_eq!(trustkit_core::gauntlet_entities::TddPhase::from(args.phase), trustkit_core::gauntlet_entities::TddPhase::Green);
        }
        _ => panic!("Expected task test command"),
    }
}

#[test]
fn test_parse_task_complete() {
    let args = ["trustkit", "task", "complete", "TASK-001", "--json"];
    let cli = Cli::try_parse_from(args).expect("parse should succeed");
    assert!(cli.is_json_mode());
    match cli.command {
        Command::Task(TaskArgs { action: TaskSubcommand::Complete(args) }) => {
            assert_eq!(args.task_id, "TASK-001");
            assert!(args.json);
        }
        _ => panic!("Expected task complete command"),
    }
}

#[test]
fn test_parse_wave_run() {
    let args = ["trustkit", "wave", "run", "--wave", "1"];
    let cli = Cli::try_parse_from(args).expect("parse should succeed");
    assert!(!cli.is_json_mode());
    match cli.command {
        Command::Wave(WaveArgs { action: WaveSubcommand::Run(args) }) => {
            assert_eq!(args.wave, 1);
            assert!(!args.json);
        }
        _ => panic!("Expected wave run command"),
    }
}

#[test]
fn test_parse_gauntlet_run() {
    let args = ["trustkit", "gauntlet", "run", "--json"];
    let cli = Cli::try_parse_from(args).expect("parse should succeed");
    assert!(cli.is_json_mode());
    match cli.command {
        Command::Gauntlet(GauntletArgs { action: GauntletSubcommand::Run(args) }) => {
            assert!(args.json);
        }
        _ => panic!("Expected gauntlet run command"),
    }
}

#[test]
fn test_parse_doctor_and_done() {
    let cli_doc = Cli::try_parse_from(["trustkit", "doctor"]).expect("doctor parse");
    assert_eq!(cli_doc.command, Command::Doctor(DoctorArgs { json: false }));

    let cli_done = Cli::try_parse_from(["trustkit", "done", "--json"]).expect("done parse");
    assert!(cli_done.is_json_mode());
    assert_eq!(
        cli_done.command,
        Command::Done(DoneArgs {
            security_review: false,
            skip_security_review: false,
            json: true,
        })
    );
}

#[test]
fn test_parse_merge_and_pr() {
    let cli_merge = Cli::try_parse_from(["trustkit", "merge", "--target", "develop"]).expect("merge parse");
    assert!(!cli_merge.is_json_mode());
    assert_eq!(cli_merge.command, Command::Merge(MergeArgs { target: "develop".to_string(), json: false }));

    let cli_pr = Cli::try_parse_from(["trustkit", "pr", "--json"]).expect("pr parse");
    assert!(cli_pr.is_json_mode());
    assert_eq!(cli_pr.command, Command::Pr(PrArgs { target: None, json: true }));
}

#[test]
fn test_parse_mcp_serve() {
    let cli = Cli::try_parse_from(["trustkit", "mcp", "serve"]).expect("mcp serve parse");
    assert!(!cli.is_json_mode());
    match cli.command {
        Command::Mcp(McpArgs { action: McpSubcommand::Serve(args) }) => assert!(!args.json),
        _ => panic!("Expected mcp serve command"),
    }
}

#[test]
fn test_parse_wave_plan() {
    let args = ["trustkit", "wave", "plan", "--tasks-dir", "specs/tasks", "--json"];
    let cli = Cli::try_parse_from(args).expect("parse should succeed");
    assert!(cli.is_json_mode());
    match cli.command {
        Command::Wave(WaveArgs { action: WaveSubcommand::Plan(args) }) => {
            assert_eq!(args.tasks_dir.as_deref(), Some("specs/tasks"));
            assert!(args.json);
        }
        _ => panic!("Expected wave plan command"),
    }
}

#[test]
fn test_parse_spec_lock() {
    let args = ["trustkit", "spec", "lock", "--spec", "specs/feat.md", "--json"];
    let cli = Cli::try_parse_from(args).expect("parse should succeed");
    assert!(cli.is_json_mode());
    match cli.command {
        Command::Spec(SpecArgs { action: SpecSubcommand::Lock(args) }) => {
            assert_eq!(args.spec.as_deref(), Some("specs/feat.md"));
            assert!(args.json);
        }
        _ => panic!("Expected spec lock command"),
    }
}

#[test]
fn test_parse_spec_init() {
    let args = ["trustkit", "spec", "init", "--from-prd", "docs/PRD.md", "--name", "feat", "--output-dir", "specs/feat", "--json"];
    let cli = Cli::try_parse_from(args).expect("parse should succeed");
    assert!(cli.is_json_mode());
    match cli.command {
        Command::Spec(SpecArgs { action: SpecSubcommand::Init(args) }) => {
            assert_eq!(args.from_prd, "docs/PRD.md");
            assert_eq!(args.name.as_deref(), Some("feat"));
            assert_eq!(args.output_dir.as_deref(), Some("specs/feat"));
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
    assert_eq!(cli.command, Command::Init(InitArgs { issue: Some("45".to_string()), from_prd: None, json: true }));
}

#[test]
fn test_parse_status() {
    let cli = Cli::try_parse_from(["trustkit", "status"]).expect("parse status");
    assert!(!cli.is_json_mode());
    assert_eq!(cli.command, Command::Status(StatusArgs { json: false }));
}

#[test]
fn test_parse_hook_pre_tool() {
    let args = ["trustkit", "hook", "pre-tool", "--command", "git add .", "--json"];
    let cli = Cli::try_parse_from(args).expect("parse hook pre-tool");
    assert!(cli.is_json_mode());
    match cli.command {
        Command::Hook(HookArgs { action: HookSubcommand::PreTool(pre_tool) }) => {
            assert_eq!(pre_tool.command, "git add .");
            assert!(pre_tool.json);
        }
        _ => panic!("Expected hook pre-tool command"),
    }
}
