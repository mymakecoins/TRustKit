// Handler for `trustkit update` and `tk update` (TASK-003).
// Constitutional Principle I: Strict Hexagonal Isolation.
// Constitutional Principle II: Absolute Prohibition of NULL.
// Constitutional Principle IV: Zero Panic.
// Constitutional Principle VI: Focused Modular Clean Code (< 250 LOC).
// Constitutional Principle VII: Zero Emojis and Zero AI Trailers.

use crate::cli_contract::UpdateArgs;
use crate::error_handler::DomainOutcome;
use crate::presenter::Presenter;
use trustkit_core::entities::SelfUpdateRequest;
use trustkit_core::errors::DomainViolation;
use trustkit_core::ports::SelfUpdatePort;
use trustkit_orch::SelfUpdateOrchestrator;

pub fn execute(args: &UpdateArgs, json_mode: bool) -> Result<DomainOutcome, DomainViolation> {
    let orchestrator = SelfUpdateOrchestrator::for_current_dir();
    execute_with_port(args, json_mode, &orchestrator)
}

pub fn execute_with_port(
    args: &UpdateArgs,
    json_mode: bool,
    port: &dyn SelfUpdatePort,
) -> Result<DomainOutcome, DomainViolation> {
    let req = SelfUpdateRequest::new(args.check, args.force, "main");
    let report = port.execute_update(&req)?;
    println!(
        "{}",
        Presenter::render_update(&report, args.check, args.force, json_mode || args.json)
    );
    Ok(DomainOutcome::Success)
}
