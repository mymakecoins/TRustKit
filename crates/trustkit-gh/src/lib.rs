//! TRustKit GH: GitHub CLI, board synchronization and PR gateway adapters

pub mod board_synchronizer;
pub mod git_inspector;
pub mod identity_guard;
pub mod issue_body_atomizer;
pub mod pr_gateway;

pub use board_synchronizer::ProjectBoardSynchronizer;
pub use board_synchronizer::ProjectBoardSynchronizer as BoardSynchronizer;
pub use git_inspector::GitInspector;
pub use identity_guard::GitHubIdentityGuard;
pub use issue_body_atomizer::IssueBodyAtomizer;
pub use pr_gateway::PullRequestGateway;
pub use pr_gateway::PullRequestGateway as PrGateway;
