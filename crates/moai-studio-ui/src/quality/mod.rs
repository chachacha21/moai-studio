//! TRUST 5 Quality Dashboard UI components (SPEC-V3-017 MS-2)
//!
//! Provides GPUI views for the TRUST 5 quality dashboard:
//! - `RadarChartView`: 5-axis radar chart for dimension scores
//! - `QualityGateView`: Horizontal gate indicator bar
//! - `QualityDashboardView`: Combined container
//!
//! REQ-QD-010~014: Radar chart rendering.
//! REQ-QD-015~017: Quality gate display.

pub mod quality_gate_view;
pub mod radar_chart_view;

// Re-export main types
pub use quality_gate_view::{DimensionGate, GateStatus, QualityGateView};
pub use radar_chart_view::RadarChartView;

use gpui::{
    Context, IntoElement, ParentElement, Pixels, Render, Styled, Window, canvas, div, point, px,
    rgb, prelude::FluentBuilder,
};
use moai_studio_agent::quality::Trust5Score;

use crate::design::tokens as tok;

/// Helper: extract f32 from Pixels.
#[inline]
fn pf(p: Pixels) -> f32 {
    f32::from(p)
}

/// Build a stroke path from line segments connecting points.
fn stroke_path_from_points(points: &[(f32, f32)], ox: f32, oy: f32, line_width: f32) -> gpui::Path<Pixels> {
    let mut builder = gpui::PathBuilder::stroke(px(line_width));
    if let Some(&(fx, fy)) = points.first() {
        builder.move_to(point(px(ox + fx), px(oy + fy)));
        for &(x, y) in &points[1..] {
            builder.line_to(point(px(ox + x), px(oy + y)));
        }
        builder.close();
    }
    builder.build().unwrap_or_else(|_| gpui::PathBuilder::stroke(px(line_width)).build().unwrap())
}

/// Build a fill path from line segments connecting points.
fn fill_path_from_points(points: &[(f32, f32)], ox: f32, oy: f32) -> gpui::Path<Pixels> {
    let mut builder = gpui::PathBuilder::fill();
    if let Some(&(fx, fy)) = points.first() {
        builder.move_to(point(px(ox + fx), px(oy + fy)));
        for &(x, y) in &points[1..] {
            builder.line_to(point(px(ox + x), px(oy + y)));
        }
        builder.close();
    }
    builder.build().unwrap_or_else(|_| gpui::PathBuilder::fill().build().unwrap())
}

/// Build a stroke line between two points.
fn line_path(x1: f32, y1: f32, x2: f32, y2: f32, width: f32) -> gpui::Path<Pixels> {
    let mut builder = gpui::PathBuilder::stroke(px(width));
    builder.move_to(point(px(x1), px(y1)));
    builder.line_to(point(px(x2), px(y2)));
    builder.build().unwrap_or_else(|_| gpui::PathBuilder::stroke(px(width)).build().unwrap())
}

/// Build an approximate circle path (8-segment polygon, filled).
fn circle_path(cx: f32, cy: f32, r: f32) -> gpui::Path<Pixels> {
    let mut builder = gpui::PathBuilder::fill();
    for j in 0..8 {
        let angle = (j as f32 / 8.0) * 2.0 * std::f32::consts::PI;
        let px_val = cx + r * angle.cos();
        let py_val = cy + r * angle.sin();
        if j == 0 {
            builder.move_to(point(px(px_val), px(py_val)));
        } else {
            builder.line_to(point(px(px_val), px(py_val)));
        }
    }
    builder.close();
    builder.build().unwrap_or_else(|_| gpui::PathBuilder::fill().build().unwrap())
}

/// Combined container for the TRUST 5 quality dashboard.
///
/// Renders the radar chart and gate indicators together.
/// MS-2 simplified integration: both views stacked vertically.
pub struct QualityDashboardView {
    /// Radar chart component.
    pub radar: RadarChartView,
    /// Quality gate bar component.
    pub gate: QualityGateView,
}

impl QualityDashboardView {
    /// Create a new dashboard with default settings.
    pub fn new() -> Self {
        Self {
            radar: RadarChartView::new(),
            gate: QualityGateView::new(),
        }
    }

    /// Create a dashboard with the given score and default threshold.
    pub fn with_score(score: Trust5Score) -> Self {
        Self {
            radar: RadarChartView::with_score(score),
            gate: QualityGateView::with_score_and_threshold(score, 0.75),
        }
    }

    /// Update both views with a new score.
    pub fn set_score(&mut self, score: Trust5Score) {
        self.radar.set_score(score);
        self.gate.score = score;
    }

    /// Update threshold for both views.
    pub fn set_threshold(&mut self, threshold: f32) {
        self.radar.set_threshold(threshold);
        self.gate.threshold = threshold.clamp(0.0, 1.0);
    }
}

impl Default for QualityDashboardView {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for QualityDashboardView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let size = self.radar.size;
        let center = size / 2.0;
        let radius = size / 2.0 - 30.0;
        let score = self.radar.score;
        let threshold = self.radar.threshold;

        // Precompute label positions for div overlays
        let label_data: Vec<(f32, f32, f32, u32)> = (0..5)
            .map(|i| {
                let (lx, ly) = radar_chart_view::label_position(center, center, radius, i);
                let dim_score = score.as_slice()[i];
                let label_color = if dim_score >= threshold {
                    tok::semantic::SUCCESS
                } else {
                    tok::semantic::DANGER
                };
                (lx, ly, dim_score, label_color)
            })
            .collect();

        // Canvas for drawing radar chart
        let chart_canvas = canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _state, window, _cx| {
                let ox = pf(bounds.origin.x);
                let oy = pf(bounds.origin.y);

                // Axis lines from center to perimeter
                for i in 0..5 {
                    let (ex, ey) = radar_chart_view::axis_position(center, center, radius, i, 1.0);
                    let path = line_path(ox + center, oy + center, ox + ex, oy + ey, 1.0);
                    window.paint_path(path, gpui::rgba(tok::FG_MUTED | 0x4d000000));
                }

                // Threshold reference polygon
                let threshold_points: Vec<(f32, f32)> = (0..5)
                    .map(|i| radar_chart_view::axis_position(center, center, radius, i, threshold))
                    .collect();
                let threshold_path = stroke_path_from_points(&threshold_points, ox, oy, 1.0);
                window.paint_path(threshold_path, gpui::rgba(tok::FG_MUTED | 0x60000000));

                // Score polygon: fill
                let score_points = radar_chart_view::polygon_vertices(&score, center, center, radius);
                let fill_p = fill_path_from_points(&score_points, ox, oy);
                window.paint_path(fill_p, gpui::rgba(tok::ACCENT | 0x33000000));

                // Score polygon: stroke
                let stroke_p = stroke_path_from_points(&score_points, ox, oy, 2.0);
                window.paint_path(stroke_p, gpui::rgb(tok::ACCENT));

                // Score dots
                for (i, &(sx, sy)) in score_points.iter().enumerate() {
                    let dim_score = score.as_slice()[i];
                    let dot_color = if dim_score >= threshold {
                        gpui::rgb(tok::semantic::SUCCESS)
                    } else {
                        gpui::rgb(tok::semantic::DANGER)
                    };
                    let path = circle_path(ox + sx, oy + sy, 4.0);
                    window.paint_path(path, dot_color);
                }

                // Center dot
                let center_dot = circle_path(ox + center, oy + center, 2.0);
                window.paint_path(center_dot, gpui::rgb(tok::FG_MUTED));
            },
        )
        .size_full();

        // Radar chart container
        let mut radar_chart = div()
            .relative()
            .w(px(size))
            .h(px(size))
            .bg(rgb(tok::BG_PANEL))
            .rounded_lg()
            .border_1()
            .border_color(rgb(tok::BORDER_SUBTLE))
            .child(chart_canvas);

        for (i, &(lx, ly, dim_score, label_color)) in label_data.iter().enumerate() {
            radar_chart = radar_chart.child(
                div()
                    .absolute()
                    .left(px(lx - 16.0))
                    .top(px(ly - 10.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(label_color))
                            .child(radar_chart_view::DIMENSION_LABELS[i].to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .child(format!("{:.2}", dim_score)),
                    ),
            );
        }

        // Gate indicators
        let gates = self.gate.dimension_gates();
        let all_pass = self.gate.all_pass();
        let failing = self.gate.failing_dimensions();

        let (badge_text, badge_bg) = if all_pass {
            ("GATE: PASS", tok::semantic::SUCCESS)
        } else {
            ("GATE: FAIL", tok::semantic::DANGER)
        };
        let badge_text_color = tok::theme::dark::text::ON_PRIMARY;

        let mut gate_bar = div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(8.))
            .p(px(12.))
            .bg(rgb(tok::BG_PANEL))
            .rounded_lg()
            .border_1()
            .border_color(rgb(tok::BORDER_SUBTLE));

        gate_bar = gate_bar.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .px(px(8.))
                        .py(px(2.))
                        .rounded_md()
                        .bg(rgb(badge_bg))
                        .text_xs()
                        .text_color(rgb(badge_text_color))
                        .child(badge_text.to_string()),
                )
                .when(!all_pass, |el: gpui::Div| {
                    let fail_text = failing.join(", ");
                    el.child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::semantic::DANGER))
                            .child(format!("({})", fail_text)),
                    )
                }),
        );

        let mut indicators = div().flex().flex_row().gap(px(8.)).w_full();
        for gate in &gates {
            let (status_text, status_color) = match gate.status {
                GateStatus::Pass => ("PASS", tok::semantic::SUCCESS),
                GateStatus::Fail => ("FAIL", tok::semantic::DANGER),
            };
            indicators = indicators.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(2.))
                    .flex_grow()
                    .p(px(6.))
                    .rounded_md()
                    .bg(rgb(tok::BG_SURFACE))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::FG_PRIMARY))
                            .child(gate.label.to_string()),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .child(format!("{:.2}", gate.score)),
                    )
                    .child(
                        div()
                            .px(px(4.))
                            .py(px(1.))
                            .rounded_sm()
                            .bg(rgb(status_color))
                            .text_xs()
                            .text_color(rgb(badge_text_color))
                            .child(status_text.to_string()),
                    ),
            );
        }
        gate_bar = gate_bar.child(indicators);

        // Combined layout
        div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .p(px(16.))
            .bg(rgb(tok::BG_APP))
            .child(radar_chart)
            .child(gate_bar)
    }
}
