# Git Worktree Isolation Workflow

Safely provision an isolated Git Worktree with Copy-on-Write (reflink) support for concurrent task execution.

## Usage

Provision a new worktree for an isolated task:
```bash
cargo run --bin trustkit -- worktree create --branch task/$TASK_ID --path ../worktrees/$TASK_ID
```

Remove and clean up worktree after completion:
```bash
cargo run --bin trustkit -- worktree clean --path ../worktrees/$TASK_ID
```
