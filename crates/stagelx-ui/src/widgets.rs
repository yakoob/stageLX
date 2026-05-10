use bevy_egui::egui::{
    self, Color32, Pos2, StrokeKind, Rect, Response, RichText, Sense, Shape, Stroke,
    Ui, Vec2, Widget,
};

use crate::theme::*;

// ═══════════════════════════════════════════════════════════════════════════════
// StatusDot
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DotState {
    Live,
    Tx,
    Warn,
    Idle,
    Error,
}

impl DotState {
    pub fn color(self) -> Color32 {
        match self {
            DotState::Live => RX,
            DotState::Tx => ACCENT,
            DotState::Warn => WARNING,
            DotState::Idle => IDLE,
            DotState::Error => ERROR,
        }
    }

    pub fn glow(self) -> Option<Color32> {
        match self {
            DotState::Live => Some(GLOW_RX),
            DotState::Tx => Some(GLOW_TX),
            _ => None,
        }
    }
}

pub fn status_dot(ui: &mut Ui, state: DotState) -> Response {
    let size = Vec2::splat(6.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = rect.center();
        if let Some(glow) = state.glow() {
            painter.circle_filled(center, 6.0, glow);
        }
        painter.circle_filled(center, 3.0, state.color());
    }
    response
}

// ═══════════════════════════════════════════════════════════════════════════════
// Eyebrow widget (section header)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn eyebrow_widget(ui: &mut Ui, label: &str) -> Response {
    ui.label(crate::theme::eyebrow(label))
}

pub fn section_header(ui: &mut Ui, label: &str, hint: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label(crate::theme::eyebrow(label));
        if let Some(h) = hint {
            ui.label(hint_secondary(h));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(0.0);
        });
    });
}

// ═══════════════════════════════════════════════════════════════════════════════
// Vertical divider (Tier 1 #6)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn vertical_divider(ui: &mut Ui, height: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, height), Sense::hover());
    ui.painter().line_segment(
        [Pos2::new(rect.center().x, rect.min.y), Pos2::new(rect.center().x, rect.max.y)],
        Stroke::new(1.0, BORDER_SOFT));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Search input with drawn magnifier (Tier 1 #9)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn search_input(ui: &mut Ui, query: &mut String, hint: &str, width: f32) -> Response {
    let frame = egui::Frame::new()
        .fill(BG_INPUT)
        .stroke(Stroke::new(1.0, BORDER_SOFT))
        .corner_radius(3.0)
        .inner_margin(egui::Margin::symmetric(7, 4));
    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.set_min_size(Vec2::new(width, 24.0));
            // 12-px magnifier glyph (circle + handle)
            let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
            if ui.is_rect_visible(icon_rect) {
                let p = ui.painter();
                let c = icon_rect.center();
                p.circle_stroke(Pos2::new(c.x - 1.0, c.y - 1.0), 3.5, Stroke::new(1.2, FG_MUTED));
                p.line_segment(
                    [Pos2::new(c.x + 1.5, c.y + 1.5), Pos2::new(c.x + 4.0, c.y + 4.0)],
                    Stroke::new(1.2, FG_MUTED),
                );
            }
            ui.add_space(4.0);
            let edit_width = (width - 24.0).max(1.0);
            ui.add_sized([edit_width, 16.0], egui::TextEdit::singleline(query).hint_text(hint))
        }).inner
    }).inner
}

// ═══════════════════════════════════════════════════════════════════════════════
// Card helper (Tier 3 #18)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn card(ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(BG_INPUT)
        .stroke(Stroke::new(1.0, BORDER_SOFT))
        .corner_radius(3.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, content);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pill
// ═══════════════════════════════════════════════════════════════════════════════

pub fn pill(ui: &mut Ui, label: impl Into<String>, state: Option<DotState>) -> Response {
    let label = label.into();
    let (bg, border, text_color) = match state {
        Some(DotState::Live) => (PILL_LIVE_BG, PILL_LIVE_BORDER, RX),
        Some(DotState::Idle) | None => (BG_RAISED, BORDER_SOFT, FG_SECONDARY),
        Some(s) => (BG_RAISED, BORDER_SOFT, s.color()),
    };

    let desired_size = {
        let galley = ui.painter().layout_no_wrap(
            label.clone(),
            font_body(),
            text_color,
        );
        let width = galley.size().x
            + if state.is_some() { 16.0 } else { 14.0 }
            + if state.is_some() { 5.0 } else { 0.0 };
        Vec2::new(width.max(28.0), 18.0)
    };

    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, 9.0, bg);
        painter.rect_stroke(rect, 9.0, Stroke::new(1.0, border), StrokeKind::Inside);

        let mut cursor = rect.min.x + 7.0;
        if let Some(st) = state {
            let dot_center = Pos2::new(cursor + 3.0, rect.center().y);
            painter.circle_filled(dot_center, 3.0, st.color());
            if let Some(glow) = st.glow() {
                painter.circle_filled(dot_center, 6.0, glow);
            }
            cursor += 11.0;
        }
        painter.text(
            Pos2::new(cursor, rect.center().y),
            egui::Align2::LEFT_CENTER,
            &label,
            font_body(),
            text_color,
        );
    }
    response
}

// ═══════════════════════════════════════════════════════════════════════════════
// Toggle (pill switch) — Tier 1 #8
// ═══════════════════════════════════════════════════════════════════════════════

pub fn toggle(ui: &mut Ui, on: &mut bool, label: &str) -> Response {
    let id = ui.id().with(label);
    let track_width = 32.0f32;
    let track_height = 22.0f32;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(track_width, track_height), Sense::click());

    if response.clicked() {
        *on = !*on;
        ui.ctx().data_mut(|d| d.insert_temp(id, if *on { 1.0f32 } else { 0.0f32 }));
    }

    // Animate thumb position
    let target = if *on { 1.0 } else { 0.0 };
    let dt = ui.ctx().input(|i| i.stable_dt);
    let thumb_pos: f32 = ui.ctx().data_mut(|d| {
        let current = d.get_temp_mut_or_insert_with(id, || target).clone();
        let next = current + (target - current) * dt * 6.67; // ~150ms
        *d.get_temp_mut_or_insert_with(id, || target) = next;
        next
    });

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let bg = if *on { ACCENT_BG } else { BG_INPUT };
        let border_color = if *on { ACCENT_DIM } else { BORDER_SOFT };
        painter.rect_filled(rect, 3.0, bg);
        painter.rect_stroke(rect, 3.0, Stroke::new(1.0, border_color), StrokeKind::Inside);

        // Track
        let track_rect = Rect::from_center_size(
            Pos2::new(rect.min.x + 16.0, rect.center().y),
            Vec2::new(16.0, 8.0),
        );
        painter.rect_filled(track_rect, 4.0, if *on { ACCENT } else { BORDER });

        // Thumb
        let thumb_x = track_rect.min.x + 1.0 + thumb_pos * 8.0;
        painter.circle_filled(
            Pos2::new(thumb_x, track_rect.center().y),
            3.0,
            if *on { Color32::WHITE } else { FG_MUTED },
        );
    }

    // Label as sibling outside the track rect
    let text_color = if *on { ACCENT } else { FG_MUTED };
    ui.add_space(4.0);
    let label_response = ui.add(egui::Label::new(
        RichText::new(label.to_uppercase()).size(11.0).color(text_color)
    ).selectable(false));

    response | label_response
}

// ═══════════════════════════════════════════════════════════════════════════════
// Banner (inline status row)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn banner(ui: &mut Ui, state: DotState, message: &str) -> Response {
    let desired_size = Vec2::new(ui.available_width(), 28.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let bg = match state {
            DotState::Warn => WARNING_BG,
            _ => BG_INPUT,
        };
        let border_color = match state {
            DotState::Warn => WARNING_DIM,
            _ => BORDER_SOFT,
        };
        painter.rect_filled(rect, 3.0, bg);
        painter.rect_stroke(rect, 3.0, Stroke::new(1.0, border_color), StrokeKind::Inside);

        let mut cursor_x = rect.min.x + 8.0;
        let center_y = rect.center().y;
        let dot_center = Pos2::new(cursor_x + 3.0, center_y);
        painter.circle_filled(dot_center, 3.0, state.color());
        if let Some(glow) = state.glow() {
            painter.circle_filled(dot_center, 6.0, glow);
        }
        cursor_x += 14.0;

        let text_color = match state {
            DotState::Warn => WARNING,
            _ => FG_SECONDARY,
        };
        painter.text(
            Pos2::new(cursor_x, center_y),
            egui::Align2::LEFT_CENTER,
            message,
            font_body(),
            text_color,
        );
    }
    response
}

// ═══════════════════════════════════════════════════════════════════════════════
// Swatch
// ═══════════════════════════════════════════════════════════════════════════════

pub fn swatch(
    ui: &mut Ui,
    color: Color32,
    label: &str,
    selected: bool,
) -> Response {
    let desired_size = Vec2::new(42.0, 40.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());
    if response.clicked() {
        // caller handles state change
    }
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        if selected {
            painter.rect_filled(rect, 3.0, BG_RAISED);
            painter.rect_stroke(rect, 3.0, Stroke::new(1.0, ACCENT_DIM), StrokeKind::Inside);
        }
        let chip_rect = Rect::from_center_size(
            Pos2::new(rect.center().x, rect.min.y + 12.0),
            Vec2::new(28.0, 18.0),
        );
        painter.rect_filled(chip_rect, 2.0, color);
        painter.rect_stroke(chip_rect, 2.0, Stroke::new(1.0, Color32::from_rgba_premultiplied(0, 0, 0, 102)), StrokeKind::Inside);
        if selected {
            painter.rect_stroke(chip_rect, 2.0, Stroke::new(1.0, ACCENT), StrokeKind::Inside);
        }
        painter.text(
            Pos2::new(rect.center().x, rect.max.y - 2.0),
            egui::Align2::CENTER_BOTTOM,
            label,
            font_body(),
            if selected { FG } else { FG_MUTED },
        );
    }
    response
}

// ═══════════════════════════════════════════════════════════════════════════════
// Fader — Tier 1 #10
// ═══════════════════════════════════════════════════════════════════════════════

pub struct Fader<'a> {
    pub value: &'a mut f32,
    pub min: f32,
    pub max: f32,
    pub label: &'a str,
    pub unit: &'a str,
    pub format: fn(f32) -> String,
    pub accent: Color32,
    pub height: f32,
}

impl<'a> Fader<'a> {
    pub fn new(value: &'a mut f32, label: &'a str) -> Self {
        Self {
            value,
            min: 0.0,
            max: 100.0,
            label,
            unit: "%",
            format: |v| format!("{:.0}", v),
            accent: ACCENT,
            height: 130.0,
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn unit(mut self, unit: &'a str) -> Self {
        self.unit = unit;
        self
    }

    pub fn format(mut self, f: fn(f32) -> String) -> Self {
        self.format = f;
        self
    }

    pub fn accent(mut self, accent: Color32) -> Self {
        self.accent = accent;
        self
    }
}

impl<'a> Widget for Fader<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(48.0, self.height + 50.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::drag());

        let norm = (*self.value - self.min) / (self.max - self.min);

        if response.dragged() {
            let delta = response.drag_delta().y;
            let range = self.max - self.min;
            *self.value = (*self.value - delta / self.height * range).clamp(self.min, self.max);
        }
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let track_top = rect.min.y + 28.0;
                let rel_y = pos.y - track_top;
                let new_norm = (1.0 - rel_y / self.height).clamp(0.0, 1.0);
                *self.value = self.min + new_norm * (self.max - self.min);
            }
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center_x = rect.center().x;

            // Readout — 14 px monospace per spec
            let readout = (self.format)(*self.value);
            painter.text(
                Pos2::new(center_x, rect.min.y + 8.0),
                egui::Align2::CENTER_CENTER,
                &readout,
                font_fader_readout(),
                FG,
            );
            painter.text(
                Pos2::new(center_x + 16.0, rect.min.y + 8.0),
                egui::Align2::LEFT_CENTER,
                self.unit,
                font_fader_readout(),
                FG_MUTED,
            );

            // Track
            let track_rect = Rect::from_center_size(
                Pos2::new(center_x, rect.min.y + 28.0 + self.height / 2.0),
                Vec2::new(28.0, self.height),
            );
            painter.rect_filled(track_rect, 3.0, BG_INPUT);
            painter.rect_stroke(track_rect, 3.0, Stroke::new(1.0, BORDER), StrokeKind::Inside);

            // Tick marks
            for t in [0.0, 0.25, 0.5, 0.75, 1.0f32] {
                let tick_y = track_rect.max.y - t * self.height;
                painter.line_segment(
                    [
                        Pos2::new(track_rect.max.x + 2.0, tick_y),
                        Pos2::new(track_rect.max.x + 6.0, tick_y),
                    ],
                    Stroke::new(1.0, BORDER),
                );
            }

            // Fill — two-stop vertical gradient via mesh (accent at top, FADER_GRADIENT_BOTTOM at bottom)
            let fill_height = norm * self.height - 2.0;
            if fill_height > 0.0 {
                let fill_rect = Rect::from_min_max(
                    Pos2::new(track_rect.min.x + 1.0, track_rect.max.y - fill_height),
                    Pos2::new(track_rect.max.x - 1.0, track_rect.max.y - 1.0),
                );
                let mut mesh = egui::epaint::Mesh::default();
                mesh.colored_vertex(fill_rect.left_top(),     self.accent);
                mesh.colored_vertex(fill_rect.right_top(),    self.accent);
                mesh.colored_vertex(fill_rect.right_bottom(), FADER_GRADIENT_BOTTOM);
                mesh.colored_vertex(fill_rect.left_bottom(),  FADER_GRADIENT_BOTTOM);
                mesh.add_triangle(0, 1, 2);
                mesh.add_triangle(0, 2, 3);
                painter.add(Shape::mesh(mesh));
            }

            // Cap
            let cap_y = track_rect.max.y - norm * self.height;
            let cap_rect = Rect::from_min_max(
                Pos2::new(track_rect.min.x - 3.0, cap_y - 7.0),
                Pos2::new(track_rect.max.x + 3.0, cap_y + 7.0),
            );
            painter.rect_filled(cap_rect, 2.0, CAP_BG_TOP);
            painter.rect_stroke(cap_rect, 2.0, Stroke::new(1.0, BORDER_STRONG), StrokeKind::Inside);
            painter.line_segment(
                [
                    Pos2::new(cap_rect.min.x + 2.0, cap_rect.center().y),
                    Pos2::new(cap_rect.max.x - 2.0, cap_rect.center().y),
                ],
                Stroke::new(1.0, self.accent.linear_multiply(0.9)),
            );

            // Label
            painter.text(
                Pos2::new(center_x, rect.max.y - 4.0),
                egui::Align2::CENTER_BOTTOM,
                self.label.to_uppercase(),
                font_body(),
                FG_SECONDARY,
            );
        }
        response
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Encoder — Tier 2 #11
// ═══════════════════════════════════════════════════════════════════════════════

pub struct Encoder<'a> {
    pub value: &'a mut f32,
    pub label: &'a str,
    pub unit: &'a str,
    pub min: f32,
    pub max: f32,
    pub default_value: f32,
    pub decimals: usize,
    pub sub: Option<&'a str>,
    pub size: f32,
    pub accent: Color32,
}

impl<'a> Encoder<'a> {
    pub fn new(value: &'a mut f32, label: &'a str) -> Self {
        Self {
            value,
            label,
            unit: "",
            min: 0.0,
            max: 100.0,
            default_value: 0.0,
            decimals: 0,
            sub: None,
            size: 76.0,
            accent: ACCENT,
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn default_value(mut self, v: f32) -> Self {
        self.default_value = v;
        self
    }

    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = d;
        self
    }

    pub fn unit(mut self, unit: &'a str) -> Self {
        self.unit = unit;
        self
    }

    pub fn sub(mut self, sub: &'a str) -> Self {
        self.sub = Some(sub);
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl<'a> Widget for Encoder<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(self.size, self.size + 24.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::drag());
        if response.dragged() {
            let delta = -response.drag_delta().y; // Y-axis, up = increase
            let range = self.max - self.min;
            let mut sens = 0.002 * range; // default ~0.2% of range per pixel
            if ui.input(|i| i.modifiers.shift) {
                sens *= 0.1;
            } else if ui.input(|i| i.modifiers.command || i.modifiers.ctrl) {
                sens *= 5.0;
            }
            *self.value = (*self.value + delta * sens).clamp(self.min, self.max);
        }
        if response.double_clicked() {
            *self.value = self.default_value;
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let cx = rect.center().x;
            let cy = rect.min.y + self.size / 2.0;
            let r = self.size / 2.0 - 6.0;
            let start_deg = -135.0f32;
            let end_deg = 135.0f32;
            let norm = (*self.value - self.min) / (self.max - self.min);
            let angle_deg = start_deg + norm * (end_deg - start_deg);

            // Track arc
            painter.add(Shape::Path(epaint::PathShape::line(
                arc_points(cx, cy, r, start_deg, end_deg),
                Stroke::new(2.0, BORDER),
            )));

            // Fill arc
            painter.add(Shape::Path(epaint::PathShape::line(
                arc_points(cx, cy, r, start_deg, angle_deg),
                Stroke::new(2.0, self.accent),
            )));

            // Indicator dot
            let ind_rad = (angle_deg - 90.0).to_radians();
            let ind_r = r - 3.0;
            let ind_x = cx + ind_r * ind_rad.cos();
            let ind_y = cy + ind_r * ind_rad.sin();
            painter.circle_filled(Pos2::new(ind_x, ind_y), 2.5, self.accent);

            // Hub
            let inner_r = r - 7.0;
            painter.circle_filled(Pos2::new(cx, cy), inner_r, BG_INPUT);
            painter.circle_stroke(Pos2::new(cx, cy), inner_r, Stroke::new(1.0, BORDER));

            // Center readout — 18 px monospace per spec
            let value_text = format!("{:.*}{}", self.decimals, self.value, self.unit);
            painter.text(
                Pos2::new(cx, cy - if self.sub.is_some() { 4.0 } else { 0.0 }),
                egui::Align2::CENTER_CENTER,
                value_text,
                font_encoder_readout(),
                FG,
            );
            if let Some(sub) = self.sub {
                painter.text(
                    Pos2::new(cx, cy + 10.0),
                    egui::Align2::CENTER_CENTER,
                    sub,
                    font_status(),
                    FG_MUTED,
                );
            }

            // Label below
            painter.text(
                Pos2::new(cx, rect.max.y - 4.0),
                egui::Align2::CENTER_BOTTOM,
                self.label.to_uppercase(),
                font_body(),
                FG_SECONDARY,
            );
        }
        response
    }
}

fn arc_points(cx: f32, cy: f32, r: f32, start_deg: f32, end_deg: f32) -> Vec<Pos2> {
    let steps = 64;
    // Tier 3 #22: don't swap — respect caller's direction
    (0..=steps)
        .map(|i| {
            let t = i as f32 / steps as f32;
            let deg = start_deg + t * (end_deg - start_deg);
            let rad = (deg - 90.0).to_radians();
            Pos2::new(cx + r * rad.cos(), cy + r * rad.sin())
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dropzone
// ═══════════════════════════════════════════════════════════════════════════════

/// Dropzone widget. Returns `true` if the Browse button was clicked.
/// Callers handle the file-picker themselves so no closure capture is needed.
pub fn dropzone(ui: &mut Ui, label: &str, hint: &str) -> bool {
    let desired_size = Vec2::new(ui.available_width(), 52.0);
    let (rect, _response) = ui.allocate_exact_size(desired_size, Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, 3.0, Color32::from_rgba_premultiplied(10, 11, 13, 153));
        painter.rect_stroke(rect, 3.0, Stroke::new(1.0, BORDER_STRONG), StrokeKind::Inside);

        // Icon tile
        let tile_rect = Rect::from_center_size(
            Pos2::new(rect.min.x + 28.0, rect.center().y),
            Vec2::splat(28.0),
        );
        painter.rect_filled(tile_rect, 3.0, BG_RAISED);
        painter.rect_stroke(tile_rect, 3.0, Stroke::new(1.0, BORDER), StrokeKind::Inside);

        // Label + hint
        painter.text(
            Pos2::new(rect.min.x + 48.0, rect.center().y - 6.0),
            egui::Align2::LEFT_CENTER,
            label,
            font_body(),
            FG,
        );
        painter.text(
            Pos2::new(rect.min.x + 48.0, rect.center().y + 8.0),
            egui::Align2::LEFT_CENTER,
            hint,
            font_hint(),
            FG_MUTED,
        );
    }

    // Browse button overlaid inside the right edge of the allocated rect
    let btn_rect = Rect::from_min_size(
        Pos2::new(rect.max.x - 68.0, rect.center().y - 12.0),
        Vec2::new(60.0, 24.0),
    );
    let mut clicked = false;
    ui.scope_builder(egui::UiBuilder::new().max_rect(btn_rect), |ui| {
        if ui.add_sized([60.0, 24.0], egui::Button::new("Browse").fill(BG_RAISED).stroke(Stroke::new(1.0, BORDER))).clicked() {
            clicked = true;
        }
    });
    clicked
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tab helpers
// ═══════════════════════════════════════════════════════════════════════════════

pub fn tab_button(ui: &mut Ui, label: &str, active: bool, badge: Option<usize>) -> Response {
    let badge_str = badge.map(|b| format!(" {}", b)).unwrap_or_default();
    let text = format!("{}{}", label, badge_str);
    let mut rich = RichText::new(&text)
        .size(11.0)
        .color(if active { FG } else { FG_SECONDARY });
    if active {
        rich = rich.strong();
    }
    let button = egui::Button::new(rich)
        .fill(if active { BG_PANEL } else { Color32::TRANSPARENT })
        .stroke(if active { Stroke::new(1.0, BORDER) } else { Stroke::NONE })
        .corner_radius(3.0)
        .min_size(Vec2::new(0.0, 26.0));
    ui.add(button)
}

/// Tier 2 #15: library tab with explicit FontId, no dead RichText
pub fn library_tab(ui: &mut Ui, label: &str, active: bool, badge: Option<usize>) -> Response {
    let galley = ui.painter().layout_no_wrap(label.to_string(), font_body(), if active { FG } else { FG_SECONDARY });
    let width = galley.size().x + 32.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 26.0), Sense::click());
    if response.clicked() {
        // caller handles tab switch
    }
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        if active {
            painter.line_segment(
                [Pos2::new(rect.min.x, rect.max.y - 1.0), Pos2::new(rect.max.x, rect.max.y - 1.0)],
                Stroke::new(1.0, ACCENT),
            );
        }
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            font_body(),
            if active { FG } else { FG_SECONDARY },
        );
        if let Some(b) = badge {
            if b > 0 {
                let badge_text = format!("{}", b);
                let badge_pos = Pos2::new(rect.center().x + galley.size().x * 0.5 + 8.0, rect.center().y);
                painter.text(
                    badge_pos,
                    egui::Align2::LEFT_CENTER,
                    &badge_text,
                    font_body(),
                    if active { ACCENT } else { FG_MUTED },
                );
            }
        }
    }
    response
}

// ═══════════════════════════════════════════════════════════════════════════════
// Panel chrome helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn icon_btn(ui: &mut Ui, paint: impl Fn(&egui::Painter, Rect, Color32)) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let color = if response.hovered() { FG } else { FG_MUTED };
        paint(ui.painter(), rect, color);
    }
    response
}

/// Corners-out glyph — use for panel detach.
pub fn icon_btn_detach(ui: &mut Ui) -> Response {
    icon_btn(ui, |p, rect, color| {
        let stroke = Stroke::new(1.5, color);
        let c = rect.center();
        let r = 4.5_f32;
        let arm = 3.0_f32;
        p.line_segment([Pos2::new(c.x - r,       c.y - r),       Pos2::new(c.x - r + arm, c.y - r      )], stroke);
        p.line_segment([Pos2::new(c.x - r,       c.y - r),       Pos2::new(c.x - r,       c.y - r + arm)], stroke);
        p.line_segment([Pos2::new(c.x + r - arm, c.y - r),       Pos2::new(c.x + r,       c.y - r      )], stroke);
        p.line_segment([Pos2::new(c.x + r,       c.y - r),       Pos2::new(c.x + r,       c.y - r + arm)], stroke);
        p.line_segment([Pos2::new(c.x - r,       c.y + r - arm), Pos2::new(c.x - r,       c.y + r      )], stroke);
        p.line_segment([Pos2::new(c.x - r,       c.y + r),       Pos2::new(c.x - r + arm, c.y + r      )], stroke);
        p.line_segment([Pos2::new(c.x + r,       c.y + r - arm), Pos2::new(c.x + r,       c.y + r      )], stroke);
        p.line_segment([Pos2::new(c.x + r,       c.y + r),       Pos2::new(c.x + r - arm, c.y + r      )], stroke);
    })
}

/// Single-bar glyph — use for panel minimize / restore.
pub fn icon_btn_minimize(ui: &mut Ui) -> Response {
    icon_btn(ui, |p, rect, color| {
        let c = rect.center();
        p.line_segment(
            [Pos2::new(c.x - 4.5, c.y), Pos2::new(c.x + 4.5, c.y)],
            Stroke::new(1.5, color),
        );
    })
}

pub fn panel_titlebar(
    ui: &mut Ui,
    title: &str,
    subtitle: Option<&str>,
    on_detach: Option<impl FnOnce()>,
    on_minimize: Option<impl FnOnce()>,
) {
    ui.horizontal(|ui| {
        ui.set_min_size(Vec2::new(ui.available_width(), 28.0));
        ui.label(panel_title(title));
        if let Some(sub) = subtitle {
            ui.label(RichText::new(sub).size(10.0).monospace().color(FG_MUTED));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(cb) = on_minimize {
                if icon_btn_minimize(ui).on_hover_text("Minimize").clicked() {
                    cb();
                }
            }
            if let Some(cb) = on_detach {
                if icon_btn_detach(ui).on_hover_text("Detach").clicked() {
                    cb();
                }
            }
        });
    });
    // Tier 1 #7 + Tier 3 #19: hairline instead of ui.separator()
    let p = ui.available_rect_before_wrap();
    ui.painter().line_segment(
        [Pos2::new(p.min.x, p.min.y), Pos2::new(p.max.x, p.min.y)],
        Stroke::new(1.0, BORDER),
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Segmented control
// ═══════════════════════════════════════════════════════════════════════════════

pub fn segmented_control(ui: &mut Ui, options: &[&str], selected: &mut usize) -> Response {
    ui.horizontal(|ui| {
        let mut overall_response: Option<Response> = None;
        for (i, opt) in options.iter().enumerate() {
            let is_active = *selected == i;
            let btn = egui::Button::new(
                RichText::new(*opt).size(10.0).color(if is_active { ACCENT } else { FG_SECONDARY }),
            )
            .fill(if is_active { BG_RAISED } else { Color32::TRANSPARENT })
            .corner_radius(2.0)
            .min_size(Vec2::new(0.0, 20.0));
            let r = ui.add(btn);
            if r.clicked() {
                *selected = i;
            }
            overall_response = Some(match overall_response {
                Some(prev) => prev | r,
                None => r,
            });
        }
        overall_response.unwrap_or_else(|| ui.allocate_response(Vec2::ZERO, Sense::hover()))
    }).inner
}

use bevy_egui::egui::epaint;
