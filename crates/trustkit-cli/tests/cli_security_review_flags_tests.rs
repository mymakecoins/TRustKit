use clap::Parser;
use trustkit_cli::cli_contract::{
    Cli, CodeReviewEffortArg, Command, DoneArgs, GauntletArgs, GauntletSubcommand, TaskArgs,
    TaskSubcommand,
};
use trustkit_core::entities::CliSecurityReviewFlag;

#[test]
fn test_parse_gauntlet_run_and_done_security_review_force_execute() {
    let cli_gauntlet =
        Cli::try_parse_from(["trustkit", "gauntlet", "run", "--security-review"]).unwrap();
    match cli_gauntlet.command {
        Command::Gauntlet(GauntletArgs {
            action: GauntletSubcommand::Run(args),
        }) => {
            assert!(args.security_review);
            assert!(!args.skip_security_review);
            assert_eq!(
                args.resolve_cli_security_flag(),
                CliSecurityReviewFlag::ForceExecute
            );
        }
        other => panic!("Expected Gauntlet Run, got {other:?}"),
    }

    let cli_done =
        Cli::try_parse_from(["trustkit", "done", "--security-review", "--json"]).unwrap();
    match cli_done.command {
        Command::Done(args) => {
            assert!(args.security_review);
            assert!(!args.skip_security_review);
            assert!(args.json);
            assert_eq!(
                args.resolve_cli_security_flag(),
                CliSecurityReviewFlag::ForceExecute
            );
        }
        other => panic!("Expected Done, got {other:?}"),
    }
}

#[test]
fn test_parse_done_and_gauntlet_run_skip_security_review() {
    let cli_done = Cli::try_parse_from(["trustkit", "done", "--skip-security-review"]).unwrap();
    match cli_done.command {
        Command::Done(DoneArgs {
            security_review,
            skip_security_review,
            json,
        }) => {
            assert!(!security_review);
            assert!(skip_security_review);
            assert!(!json);
            let args = DoneArgs {
                security_review,
                skip_security_review,
                json,
            };
            assert_eq!(
                args.resolve_cli_security_flag(),
                CliSecurityReviewFlag::ForceSkip
            );
        }
        other => panic!("Expected Done, got {other:?}"),
    }
}

#[test]
fn test_reject_conflicting_security_review_and_skip_flags() {
    let conflict_gauntlet = Cli::try_parse_from([
        "trustkit",
        "gauntlet",
        "run",
        "--security-review",
        "--skip-security-review",
    ]);
    assert!(conflict_gauntlet.is_err());

    let conflict_done = Cli::try_parse_from([
        "trustkit",
        "done",
        "--security-review",
        "--skip-security-review",
    ]);
    assert!(conflict_done.is_err());
}

#[test]
fn test_parse_task_review_and_task_complete_code_review_flag() {
    let cli_review =
        Cli::try_parse_from(["trustkit", "task", "review", "TASK-003", "--effort", "high"])
            .unwrap();
    match cli_review.command {
        Command::Task(TaskArgs {
            action: TaskSubcommand::Review(args),
        }) => {
            assert_eq!(args.task_id, "TASK-003");
            assert_eq!(args.effort, CodeReviewEffortArg::High);
        }
        other => panic!("Expected Task Review, got {other:?}"),
    }

    let cli_complete = Cli::try_parse_from([
        "trustkit",
        "task",
        "complete",
        "TASK-003",
        "--code-review",
        "medium",
        "--json",
    ])
    .unwrap();
    match cli_complete.command {
        Command::Task(TaskArgs {
            action: TaskSubcommand::Complete(args),
        }) => {
            assert_eq!(args.task_id, "TASK-003");
            assert_eq!(args.code_review, Some(CodeReviewEffortArg::Medium));
            assert!(args.json);
        }
        other => panic!("Expected Task Complete, got {other:?}"),
    }
}
