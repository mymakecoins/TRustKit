# Feature Specification: Deterministic SDD Lifecycle Foundation, Fast Hooks & State Persistence

**Feature Branch**: `001-core-db-cli`

**Created**: 2026-10-27

**Status**: Draft

**Input**: User description: "/home/mmc/00_code/00_tools/TRustKit/docs/PRD_TRustKit.md"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Deterministic Cycle Initialization & Board Synchronization (Priority: P1)

As a senior software engineer or host AI agent orchestrating a Spec-Driven Development (SDD) cycle,
I need a deterministic CLI command that inspects the current repository state, validates operator identity, preserves pre-existing modified files without discarding work, initializes a dedicated cycle directory, moves the associated card on the project board to "In progress", and opens an immutable event session,
So that development starts instantly, safely, and with zero manual setup overhead.

**Why this priority**: Without a deterministic and safe cycle initialization, subsequent gates (spec authoring, task execution, testing, and certification) lack isolation, state tracking, and repository safety guarantees.

**Independent Test**: Can be tested independently by running the initialization command against a repository with an issue number, verifying that pre-existing modified files remain intact, the cycle directory is created, the issue card is updated on the project board, and the session is recorded in the local event store with an exit code of 0 in under 250ms.

**Acceptance Scenarios**:

1. **Given** a clean or dirty Git repository and an authenticated operator account, **When** the initialization command is executed with an active issue identifier (`--issue 45`), **Then** pre-existing dirty files are recorded in an audit list without executing stash commands, the issue card is moved to "In progress" on the project board, a dedicated cycle directory `docs/specs/issue-45-AAAAMMDDHHMM/` is created, and the cycle state is persisted in the event store.
2. **Given** a repository where the initialization command is invoked without an issue identifier or outside an issue branch, **When** the initialization command executes, **Then** the system activates an unlinked local cycle, creates an unlinked directory `docs/specs/no-issue-AAAAMMDDHHMM/`, skips project board modifications without throwing errors (`SKIPPED_UNLINKED_CYCLE`), and emits an explicit unlinked-cycle notice (without blocking `stdin` in `--json` or non-interactive mode).
3. **Given** an invalid operator account or missing repository permissions, **When** the initialization command is executed, **Then** the process terminates cleanly with an explicit error diagnostics payload, without creating partial directories or corrupting repository state.

---

### User Story 2 - Sub-Millisecond Pre-Tool Hook Interception (Priority: P1)

As a host AI coding agent and project tech lead,
I need automated pre-tool command interception that evaluates shell actions before execution in under 3 milliseconds,
So that destructive, forbidden, or non-compliant operations (such as `git stash`, unstaged sweeping additions `git add .`, hard resets, direct commits to protected branches, or autonomous pull request creation) are blocked in real time before damage occurs.

**Why this priority**: Guarding against destructive commands prevents agent hallucinations from bypassing project rules, wiping unsaved work, or violating repository governance during autonomous coding cycles.

**Independent Test**: Can be tested independently by passing forbidden shell commands to the pre-tool hook inspector and verifying that it blocks forbidden patterns with an informative diagnostic message and exit code in under 3 milliseconds, while allowing safe developer commands to pass unimpeded.

**Acceptance Scenarios**:

1. **Given** an active development session, **When** the pre-tool hook inspects a command attempting `git stash`, `git add .`, `git add -A`, or `git reset --hard`, **Then** the command is immediately blocked with an explanatory policy message identifying the violated invariant, finishing inspection in under 3 milliseconds.
2. **Given** an active development session, **When** the pre-tool hook inspects an unauthorized attempt to commit directly to protected branches (`main`, `staging`, `develop`) without explicit overrides, **Then** the execution is rejected with guidance on using dedicated task branches.
3. **Given** an approved, scoped command targeting an assigned task file, **When** the pre-tool hook inspects the command, **Then** the command receives an immediate approval verdict (`CleanPass`) in under 3 milliseconds.

---

### User Story 3 - Resilient Local Event Persistence & Dual-Audience Output (Priority: P2)

As a developer monitoring task progression and an AI agent parsing execution results,
I need a resilient local state repository operating concurrently with transactional guarantees, providing compact, null-free JSON output for machine consumers alongside clean, emoji-free textual output for human terminals,
So that event history is audit-proof and agents consume minimal context tokens without null-dereference crashes.

**Why this priority**: Machine agents pay token and cognitive costs when parsing verbose, unstructured terminal output or null-ridden schemas. Humans require readable ASCII feedback. Both require concurrency-safe, corrupt-proof state storage.

**Independent Test**: Can be tested independently by recording concurrent command execution events into the event repository, querying status in both human and JSON modes, and asserting that JSON responses contain zero null values while human output formats tabular data cleanly without Unicode emojis.

**Acceptance Scenarios**:

1. **Given** multiple concurrent state recording operations during task execution, **When** events are logged to the local repository, **Then** all writes succeed without database lock collisions, foreign key violations, or partial row writes.
2. **Given** an agent inspecting cycle status using the `--json` flag, **When** the status payload is emitted, **Then** all missing relationships or uninitialized attributes are represented as explicit Special Case objects or empty collections `[]`, with zero occurrences of `: null`.
3. **Given** a human operator inspecting status in a terminal, **When** the command executes without the `--json` flag, **Then** output is presented as structured ASCII tables with status indicators (`[PASS]`, `[FAIL]`, `[GATE-0]`) without Unicode emojis or terminal corruption.

---

### User Story 4 - Cycle Inspection & Health Diagnostic Querying (Priority: P3)

As a quality assurance engineer or technical coordinator,
I need a status inspection command that evaluates the current cycle phase, gate milestone, branch compliance, and active task progress,
So that the development status and health of any ongoing feature cycle can be audited at a glance.

**Why this priority**: Providing full transparency into the current gate milestone and state transitions ensures operators and reviews can immediately identify bottlenecks or failed gates without manually inspecting raw files.

**Independent Test**: Can be tested independently by invoking the status command at various lifecycle stages (Gate 0, Gate 1, etc.) and verifying accurate reporting of current gate stage, branch name, and dirty file tracking.

**Acceptance Scenarios**:

1. **Given** an ongoing cycle at Gate 0, **When** the status command is executed, **Then** the output displays the active cycle ID, current gate stage, associated branch, and list of tracked pre-existing modified files.
2. **Given** a repository with no active cycle initialized, **When** the status command is executed, **Then** the system gracefully reports the absence of an active cycle with guidance on running the initialization command, returning an exit code of 0.

---

### Edge Cases

- **Concurrent Execution Lock:** What happens when two tool processes attempt to record execution evidence simultaneously? The persistent storage layer must handle concurrent access gracefully with automated retry timeouts without throwing busy errors or aborting.
- **Detached HEAD or Missing Git Repository:** How does the system handle initialization when run outside a Git work tree or in detached HEAD state? The system must fail fast with a descriptive error before creating any directories or touching external project boards.
- **Offline / GitHub API Network Flake:** How does the system handle temporary network failure during project board updates? The system must capture the pending transition as an offline deferred action in the local event store, allowing local work to proceed without blocking the operator.
- **Corrupted Local State Database:** What happens if the local state file is corrupted or locked by an external process? The system must provide a diagnostic message directing the user to recovery options without crashing with unhandled panic exceptions.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide an initialization capability (`init`) that accepts an optional issue identifier (`--issue`) and an optional source PRD path (`--from-prd <PATH>`, defaulting to `<spec_dir>/spec.md`), discovers local repository context, creates a timestamped cycle directory, and registers the cycle in the event store.
- **FR-002**: When an issue identifier is provided during initialization, the system MUST verify the operator identity against configured team credentials and update the corresponding card on the project board to "In progress".
- **FR-003**: If no issue identifier is supplied and the current branch is unlinked, the system MUST initialize an unlinked local cycle with a dedicated directory structure and skip project board updates without error.
- **FR-004**: During initialization, the system MUST inspect and record any pre-existing dirty files in the working tree, and MUST NOT execute `git stash`, `git reset`, or discard uncommitted changes.
- **FR-005**: The system MUST provide a high-speed pre-tool hook capability (`hook pre-tool`) that inspects shell commands and completes evaluation in under 3 milliseconds.
- **FR-006**: The pre-tool hook MUST block prohibited commands including `git stash`, sweeping git additions (`git add .`, `git add -A`), hard resets (`git reset --hard`), unauthorized direct commits to protected branches (`main`, `staging`, `develop`), and autonomous pull request creation.
- **FR-007**: The system MUST persist all cycle lifecycle transitions (`SddCycle`), artifact seals (`ArtifactSeal`), traceability links (`TraceabilityLink`), atomic tasks (`AtomicTask`), and execution evidence (`ExecutionEvidence`) in an embedded, transactional local database operating with write-ahead logging and relational integrity.
- **FR-008**: All state tables in the persistence layer MUST enforce `NOT NULL` constraints across all columns, representing absent or optional attributes using explicit domain Special Case default values.
- **FR-009**: The system MUST support a machine-readable mode (`--json`) that outputs strictly typed JSON payloads with zero `null` values, encoding uninitialized relationships as typed objects or empty collections.
- **FR-010**: The system MUST support a human-readable terminal mode that displays structured tabular ASCII data and formal textual prefixes (`[PASS]`, `[FAIL]`, `[GATE-N]`), with an absolute prohibition of Unicode emojis or graphic emoticons.
- **FR-011**: The system MUST provide a status query capability (`status`) that inspects and displays the current cycle identifier, active gate, associated branch, and execution summary.
- **FR-012**: The system MUST provide centralized boundary error handling that translates domain violations into standard POSIX exit codes (`0` through `6`) and structured diagnostic outputs without unhandled program aborts or panics.

### Key Entities *(include if feature involves data)*

- **SddCycle**: Represents a single isolated feature delivery cycle; attributes include cycle identifier, issue linkage (linked with board details or unlinked local cycle), methodology profile, current gate milestone, lifecycle status, branch reference, and pre-existing dirty file records.
- **ArtifactSeal**: Represents a cryptographic BLAKE3 signature and structural AST assertion count captured for specification and test artifacts.
- **TraceabilityLink**: Represents a verified end-to-end traceability tuple linking business rules, functional requirements, atomic tasks, Mermaid nodes, and E2E verification evidence.
- **AtomicTask**: Represents an atomic unit of TDD implementation assigned to an execution wave with explicit dependencies, expected files, and TDD lifecycle state.
- **ExecutionEvidence**: Represents an immutable record of an executed command or verification; attributes include evidence identifier, cycle association, gate stage, actor role, executed command string, exit code, execution duration in milliseconds, sanitized output summary, and verification verdict.
- **IssueLinkage**: Represents the association between the cycle and external project tracking; variants include linked issue (with number, URL, and board column) or unlinked local cycle (with local directory slug).
- **GateMilestone**: Enumerates the formal delivery gates from Gate 0 (Discovery and Setup) through Gate 6 (Certification and Closure).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Pre-tool hook validation completes in under 3 milliseconds (99th percentile) across all inspected shell commands.
- **SC-002**: Complete cycle initialization (`init`) finishes execution in under 250 milliseconds when network connectivity to external board APIs is responsive.
- **SC-003**: 100% of JSON payloads generated in `--json` mode contain zero occurrences of `: null` across all success and error scenarios.
- **SC-004**: 100% of terminal and file outputs contain zero Unicode emojis, emoticons, or dingbat symbols.
- **SC-005**: Zero loss or corruption of pre-existing uncommitted files during cycle initialization and execution.
- **SC-006**: Status inspection displays accurate gate and cycle information in under 3 milliseconds (99th percentile) for active local cycles.
- **SC-007**: 100% of handled system failures exit with clean POSIX exit codes (0 to 6) accompanied by structured error descriptions, with zero unhandled panics or stack traces in production execution.

## Assumptions

- **Target Operating Environment:** The host system has standard POSIX shell tools, Git 2.38+, and authenticated GitHub CLI access (`gh`) available in the system PATH.
- **Project Structure:** Cycles operate within standard Git repositories following a branch-per-feature workflow based on `develop`.
- **Offline Resilience:** If external project board APIs are temporarily unavailable, local operations can proceed with offline event queueing without halting development.
- **Language and Tooling:** The underlying implementation delivers a compiled native binary optimized for fast start-up without external interpreter dependencies.
