//! GitStatusPanel GPUI Entity — SPEC-V3-008 MS-1.
//!
//! Renders a file status list grouped by staged/unstaged/untracked.
//! REQ-G-001 ~ REQ-G-005: status panel UI.

use crate::design::tokens;
use gpui::*;

/// Simple file entry representing a single changed file.
///
/// Not a GPUI Entity — plain data structure used by GitStatusPanel.
#[derive(Clone, Debug)]
pub struct FileEntry {
    /// Relative path from repository root.
    pub path: String,
    /// Git status code (e.g., "M", "A", "D", "??").
    pub status: String,
    /// Whether the file is staged for commit.
    pub staged: bool,
}

/// GPUI Entity that displays a grouped file status list.
///
/// Features:
/// - Three sections: Staged, Unstaged, Untracked
/// - Color-coded status badges (green/yellow/gray)
/// - Empty state when not in a git repository
/// - Render-only: parent component wires actual git calls
///
/// # SPEC trace
/// - REQ-G-001: GPUI Entity implementing Render
/// - REQ-G-002: files() returns Vec<FileEntry>
/// - REQ-G-003: set_files() populates file list
/// - REQ-G-004: grouped rendering (staged/unstaged/untracked)
/// - REQ-G-005: empty state for non-git repo
pub struct GitStatusPanel {
    /// All file entries loaded from git status.
    files: Vec<FileEntry>,
}

impl Default for GitStatusPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl GitStatusPanel {
    /// Create a new GitStatusPanel with empty state.
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
        }
    }

    /// Set the file list. Typically called after fetching from GitRepo::status_map().
    pub fn set_files(&mut self, files: Vec<FileEntry>) {
        self.files = files;
    }

    /// Returns a reference to all files.
    pub fn files(&self) -> &[FileEntry] {
        &self.files
    }

    /// Returns whether the panel is empty (no files or non-git repo).
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

impl Render for GitStatusPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut el = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(tokens::BG_PANEL));

        // Empty state for non-git repositories.
        if self.files.is_empty() {
            return el
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .flex_grow()
                        .text_sm()
                        .text_color(rgb(tokens::FG_MUTED))
                        .child("Not a git repository"),
                )
                .into_any_element();
        }

        // Group files into three sections.
        let staged: Vec<&FileEntry> = self.files.iter().filter(|f| f.staged).collect();
        let unstaged: Vec<&FileEntry> = self
            .files
            .iter()
            .filter(|f| !f.staged && f.status != "??")
            .collect();
        let untracked: Vec<&FileEntry> = self
            .files
            .iter()
            .filter(|f| !f.staged && f.status == "??")
            .collect();

        // Header with total file count.
        el = el.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .w_full()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(rgb(tokens::BORDER_SUBTLE))
                .bg(rgb(tokens::BG_SURFACE))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(tokens::FG_SECONDARY))
                        .child(format!("Changes ({})", self.files.len())),
                ),
        );

        let mut list = div()
            .flex()
            .flex_col()
            .flex_grow()
            .overflow_y_hidden()
            .px_2()
            .py_1()
            .gap(px(2.));

        // Staged section.
        if !staged.is_empty() {
            list = list.child(
                div()
                    .px_2()
                    .py_1()
                    .text_xs()
                    .text_color(rgb(tokens::FG_MUTED))
                    .child(format!("Staged ({})", staged.len())),
            );
            for file in staged {
                list = list.child(self.render_file_row(file, rgb(tokens::semantic::SUCCESS)));
            }
        }

        // Unstaged section.
        if !unstaged.is_empty() {
            list = list.child(
                div()
                    .px_2()
                    .py_1()
                    .text_xs()
                    .text_color(rgb(tokens::FG_MUTED))
                    .child(format!("Unstaged ({})", unstaged.len())),
            );
            for file in unstaged {
                list = list.child(self.render_file_row(file, rgb(tokens::semantic::WARNING)));
            }
        }

        // Untracked section.
        if !untracked.is_empty() {
            list = list.child(
                div()
                    .px_2()
                    .py_1()
                    .text_xs()
                    .text_color(rgb(tokens::FG_MUTED))
                    .child(format!("Untracked ({})", untracked.len())),
            );
            for file in untracked {
                list = list.child(self.render_file_row(file, rgb(tokens::FG_MUTED)));
            }
        }

        el.child(list).into_any_element()
    }
}

impl GitStatusPanel {
    /// Render a single file row with status badge.
    fn render_file_row(&self, file: &FileEntry, badge_color: Rgba) -> Div {
        div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .px_2()
            .py(px(2.))
            .rounded_md()
            .hover(|s| s.bg(rgb(tokens::BG_ELEVATED)))
            .cursor_pointer()
            .text_xs()
            .child(
                div()
                    .px_2()
                    .py(px(1.))
                    .rounded_md()
                    .bg(badge_color)
                    .text_color(rgb(tokens::theme::dark::text::ON_PRIMARY))
                    .child(file.status.clone()),
            )
            .child(
                div()
                    .ml_2()
                    .flex_grow()
                    .text_color(rgb(tokens::FG_PRIMARY))
                    .child(file.path.clone()),
            )
    }
}
