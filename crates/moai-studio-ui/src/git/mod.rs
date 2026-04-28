//! Git UI module — SPEC-V3-008 MS-1/MS-2/MS-3.
//!
//! GPUI Entity components for git status, commit composer, branch switching,
//! diff viewing, commit log, merge conflict resolution, and stash management.
//! REQ-G-001 ~ REQ-G-005 (StatusPanel), REQ-G-020 ~ REQ-G-024 (CommitComposer),
//! REQ-G-010 ~ REQ-G-015 (DiffViewer), REQ-G-030 ~ REQ-G-035 (BranchSwitcher),
//! REQ-G-040 ~ REQ-G-044 (LogView), REQ-G-050 ~ REQ-G-056 (MergeResolver),
//! REQ-G-060 ~ REQ-G-064 (StashPanel).

pub mod branch_switcher;
pub mod commit_composer;
pub mod diff_viewer;
pub mod log_view;
pub mod merge_resolver;
pub mod stash_panel;
pub mod status_panel;

// Re-export primary entity types for convenient access.
pub use branch_switcher::GitBranchSwitcher;
pub use commit_composer::GitCommitComposer;
pub use diff_viewer::GitDiffViewer;
pub use log_view::GitLogView;
pub use merge_resolver::GitMergeResolver;
pub use stash_panel::GitStashPanel;
pub use status_panel::{FileEntry, GitStatusPanel};
