//! GitCommitComposer GPUI Entity — SPEC-V3-008 MS-1.
//!
//! Renders a commit message composer with author display and action buttons.
//! REQ-G-020 ~ REQ-G-024: commit composer UI.

use crate::design::tokens;
use gpui::*;

/// GPUI Entity that displays a commit message composer.
///
/// Features:
/// - Message text area with placeholder
/// - Author display in secondary color
/// - Staged count badge in header
/// - Commit button (enabled only when message non-empty and staged_count > 0)
/// - Discard button (subtle/border style)
/// - Render-only: parent component wires actual git commit/discard calls
///
/// # SPEC trace
/// - REQ-G-020: message textarea + author display + Commit button + Discard button
/// - REQ-G-021: set_message() updates commit message
/// - REQ-G-022: set_author() updates author display
/// - REQ-G-023: set_staged_count() updates staged count badge
/// - REQ-G-024: Commit button enabled only when message non-empty and staged_count > 0
pub struct GitCommitComposer {
    /// Current commit message text.
    message: String,
    /// Author name/email for display.
    author: String,
    /// Number of staged files.
    staged_count: usize,
}

impl Default for GitCommitComposer {
    fn default() -> Self {
        Self::new()
    }
}

impl GitCommitComposer {
    /// Create a new GitCommitComposer with empty state.
    pub fn new() -> Self {
        Self {
            message: String::new(),
            author: String::new(),
            staged_count: 0,
        }
    }

    /// Set the commit message text.
    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }

    /// Set the author name/email for display.
    pub fn set_author(&mut self, author: String) {
        self.author = author;
    }

    /// Set the number of staged files.
    pub fn set_staged_count(&mut self, count: usize) {
        self.staged_count = count;
    }

    /// Returns the current commit message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the author string.
    pub fn author(&self) -> &str {
        &self.author
    }

    /// Returns the number of staged files.
    pub fn staged_count(&self) -> usize {
        self.staged_count
    }

    /// Returns whether the Commit button should be enabled.
    ///
    /// Per REQ-G-024: enabled only when message is non-empty and staged_count > 0.
    pub fn can_commit(&self) -> bool {
        !self.message.trim().is_empty() && self.staged_count > 0
    }
}

impl Render for GitCommitComposer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let commit_enabled = self.can_commit();

        let mut el = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(tokens::BG_PANEL));

        // Header with "Commit" title and staged count badge.
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
                        .flex_grow()
                        .text_sm()
                        .text_color(rgb(tokens::FG_PRIMARY))
                        .child("Commit"),
                )
                .child(
                    div()
                        .px_2()
                        .py(px(1.))
                        .rounded_md()
                        .bg(rgba(0x00_6b_5b_ff)) // tokens::semantic::INFO as Rgba
                        .text_xs()
                        .text_color(rgb(tokens::theme::dark::text::ON_PRIMARY))
                        .child(format!("{}", self.staged_count)),
                ),
        );

        // Author display line.
        if !self.author.is_empty() {
            el = el.child(
                div()
                    .px_3()
                    .py_1()
                    .text_xs()
                    .text_color(rgb(tokens::FG_SECONDARY))
                    .child(format!("Author: {}", self.author)),
            );
        }

        // Message text area placeholder.
        let message_text = if self.message.is_empty() {
            "Commit message..."
        } else {
            &self.message
        };
        let message_color = if self.message.is_empty() {
            rgb(tokens::FG_MUTED)
        } else {
            rgb(tokens::FG_PRIMARY)
        };

        el = el.child(
            div()
                .flex()
                .flex_col()
                .flex_grow()
                .px_3()
                .py_2()
                .gap(px(2.))
                .child(
                    div()
                        .flex_grow()
                        .w_full()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .border_1()
                        .border_color(rgb(tokens::BORDER_SUBTLE))
                        .bg(rgb(tokens::BG_SURFACE))
                        .text_sm()
                        .text_color(message_color)
                        .child(message_text.to_string()),
                )
                .child(
                    // Button row: Commit (primary) + Discard (secondary).
                    div()
                        .flex()
                        .flex_row()
                        .gap_2()
                        .child(
                            div()
                                .px_4()
                                .py(px(6.))
                                .rounded_md()
                                .bg(if commit_enabled {
                                    rgb(tokens::ACCENT)
                                } else {
                                    rgb(tokens::FG_MUTED)
                                })
                                .text_sm()
                                .text_color(rgb(tokens::theme::dark::text::ON_PRIMARY))
                                .cursor_pointer()
                                .child("Commit"),
                        )
                        .child(
                            div()
                                .px_4()
                                .py(px(6.))
                                .rounded_md()
                                .border_1()
                                .border_color(rgb(tokens::BORDER_SUBTLE))
                                .text_sm()
                                .text_color(rgb(tokens::FG_SECONDARY))
                                .cursor_pointer()
                                .child("Discard"),
                        ),
                ),
        );

        el.into_any_element()
    }
}
