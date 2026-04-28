//! Image Viewer with zoom and pan support (C-5).
//!
//! SPEC: C-5 Image Surface zoom/pan.
//! SPEC-V3-016 MS-1: Actual image decoding and rendering (REQ-IV-001~006).
//! Features: Mouse wheel zoom, click-drag pan, fit-to-view reset.

use crate::design::tokens as tok;
use crate::viewer::image_data::ImageData;
use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};

/// Image viewer state with zoom and pan (REQ-IV-001).
pub struct ImageViewer {
    /// Current zoom level (1.0 = 100%).
    zoom: f32,
    /// Pan offset X (pixels).
    pan_x: f32,
    /// Pan offset Y (pixels).
    pan_y: f32,
    /// Whether currently dragging to pan.
    is_dragging: bool,
    /// Last mouse position for drag delta calculation.
    last_mouse_x: f32,
    last_mouse_y: f32,
    /// Decoded image data (REQ-IV-001, REQ-IV-005).
    image_data: Option<ImageData>,
    /// Error message if image loading failed (REQ-IV-003).
    error_message: Option<String>,
}

impl ImageViewer {
    /// Create a new image viewer.
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            is_dragging: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
            image_data: None,
            error_message: None,
        }
    }

    /// Load decoded image data (REQ-IV-002, REQ-IV-005).
    pub fn load_image(&mut self, data: ImageData, _cx: &mut Context<Self>) {
        let width = data.width as f32;
        let height = data.height as f32;
        self.image_data = Some(data);
        self.error_message = None;
        self.reset_view();
        // Set initial zoom to fit image within typical viewport
        self.fit_to_view(width, height);
    }

    /// Set error message if image loading failed (REQ-IV-003).
    pub fn set_error(&mut self, message: String, _cx: &mut Context<Self>) {
        self.error_message = Some(message);
        self.image_data = None;
    }

    /// Calculate zoom to fit image within bounds (helper for REQ-IV-023).
    fn fit_to_view(&mut self, img_w: f32, img_h: f32) {
        const VIEWPORT_W: f32 = 800.0;
        const VIEWPORT_H: f32 = 600.0;

        let scale_x = VIEWPORT_W / img_w;
        let scale_y = VIEWPORT_H / img_h;
        self.zoom = scale_x.min(scale_y).min(1.0); // Don't zoom in beyond 100%
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    /// Reset zoom and pan to fit-to-view.
    pub fn reset_view(&mut self) {
        self.zoom = 1.0;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    /// Handle mouse wheel zoom.
    pub fn handle_wheel(&mut self, delta: f32) {
        const ZOOM_FACTOR: f32 = 0.1;
        const ZOOM_MIN: f32 = 0.1;
        const ZOOM_MAX: f32 = 10.0;

        if delta < 0.0 {
            // Zoom in
            self.zoom = (self.zoom + ZOOM_FACTOR).min(ZOOM_MAX);
        } else {
            // Zoom out
            self.zoom = (self.zoom - ZOOM_FACTOR).max(ZOOM_MIN);
        }
    }

    /// Handle mouse down for drag start.
    pub fn handle_mouse_down(&mut self, x: f32, y: f32) {
        self.is_dragging = true;
        self.last_mouse_x = x;
        self.last_mouse_y = y;
    }

    /// Handle mouse up for drag end.
    pub fn handle_mouse_up(&mut self) {
        self.is_dragging = false;
    }

    /// Handle mouse move for panning.
    pub fn handle_mouse_move(&mut self, x: f32, y: f32) {
        if self.is_dragging {
            let dx = x - self.last_mouse_x;
            let dy = y - self.last_mouse_y;
            self.pan_x += dx;
            self.pan_y += dy;
            self.last_mouse_x = x;
            self.last_mouse_y = y;
        }
    }
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for ImageViewer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Show error if loading failed (REQ-IV-003)
        if let Some(ref error) = self.error_message {
            return self
                .render_error_message(error)
                .into_any_element();
        }

        // If no image loaded, show placeholder (REQ-IV-006)
        let Some(ref data) = self.image_data else {
            return self.render_placeholder().into_any_element();
        };

        let img_w = data.width as f32;
        let img_h = data.height as f32;
        let display_w = img_w * self.zoom;
        let display_h = img_h * self.zoom;

        div()
            .w_full()
            .h_full()
            .bg(rgb(tok::BG_PANEL))
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .relative()
            // Image container (REQ-IV-005)
            .child(
                div()
                    .relative()
                    .w(px(display_w))
                    .h(px(display_h))
                    .bg(rgb(tok::BG_SURFACE))
                    .border_1()
                    .border_color(rgb(tok::BORDER_SUBTLE))
                    // Apply pan offset
                    .ml(px(self.pan_x))
                    .mt(px(self.pan_y))
                    // TODO: Render actual image pixels (REQ-IV-005)
                    // For now, show dimensions as placeholder
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .text_sm()
                            .child(format!("{} x {} pixels", data.width, data.height)),
                    )
                    // Zoom info overlay (REQ-IV-025)
                    .child(
                        div()
                            .absolute()
                            .top_4()
                            .left_4()
                            .px(px(8.))
                            .py(px(4.))
                            .rounded_md()
                            .bg(rgb(0x00000080))
                            .text_color(rgb(0xFFFFFF))
                            .text_sm()
                            .child(format!("Zoom: {:.0}%", self.zoom * 100.0)),
                    )
                    // Controls hint
                    .child(
                        div()
                            .absolute()
                            .bottom_4()
                            .right_4()
                            .px(px(8.))
                            .py(px(4.))
                            .rounded_md()
                            .bg(rgb(0x00000080))
                            .text_color(rgb(0xFFFFFF))
                            .text_xs()
                            .child("Scroll to zoom • Drag to pan"),
                    ),
            )
            .into_any_element()
    }
}

impl ImageViewer {
    fn render_placeholder(&self) -> gpui::Div {
        div()
            .flex()
            .items_center()
            .justify_center()
            .h_full()
            .text_lg()
            .text_color(rgb(tok::FG_SECONDARY))
            .child("Image Viewer (C-5)")
    }

    fn render_error_message(&self, message: &str) -> gpui::Div {
        div()
            .flex()
            .items_center()
            .justify_center()
            .h_full()
            .text_lg()
            .text_color(rgb(tok::FG_SECONDARY))
            .child(format!("Failed to load image: {}", message))
    }
}
