//! Cue panel — list view, GO/BACK/RECORD, keyboard shortcuts.

use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, Pos2, RichText, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::theme::*;
// use crate::widgets;
use stagelx_show::{
    BackCueEvent, CuePlayhead, CueStack, DeleteCueEvent, GoCueEvent, RecordCueEvent,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Cue panel (docked / inline)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn cue_panel_docked(
    ui: &mut Ui,
    stack: &CueStack,
    playhead: &CuePlayhead,
    commands: &mut Commands,
) {
    // ── Toolbar ───────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.set_min_size(Vec2::new(ui.available_width(), 32.0));

        let btn_w = (ui.available_width() - 8.0) / 3.0;

        // BACK
        let back_btn = egui::Button::new(RichText::new("◀ BACK").size(11.0).strong().color(FG))
            .fill(BG_RAISED)
            .stroke(Stroke::new(1.0, BORDER))
            .min_size(Vec2::new(btn_w, 28.0));
        if ui.add_sized([btn_w, 28.0], back_btn).clicked() {
            commands.trigger(BackCueEvent);
        }
        ui.add_space(4.0);

        // GO
        let go_btn = egui::Button::new(RichText::new("GO ▶").size(11.0).strong().color(FG))
            .fill(ACCENT_BG)
            .stroke(Stroke::new(1.0, ACCENT_DIM))
            .min_size(Vec2::new(btn_w, 28.0));
        if ui.add_sized([btn_w, 28.0], go_btn).clicked() {
            commands.trigger(GoCueEvent);
        }
        ui.add_space(4.0);

        // RECORD
        let rec_btn = egui::Button::new(RichText::new("● REC").size(11.0).strong().color(RX))
            .fill(BG_RAISED)
            .stroke(Stroke::new(1.0, BORDER))
            .min_size(Vec2::new(btn_w, 28.0));
        if ui.add_sized([btn_w, 28.0], rec_btn).clicked() {
            commands.trigger(RecordCueEvent);
        }
    });

    ui.add_space(4.0);

    // ── Cue list ──────────────────────────────────────────────────────────────
    let active_idx = playhead.current_cue_index;

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for (i, cue) in stack.cues.iter().enumerate() {
                let is_active = active_idx == Some(i);
                let row_h = 28.0;
                let row_w = ui.available_width();
                let (rect, response) = ui.allocate_exact_size(Vec2::new(row_w, row_h), Sense::click());

                if ui.is_rect_visible(rect) {
                    let painter = ui.painter();

                    // Background
                    let bg = if is_active {
                        ROW_SELECTED
                    } else if i % 2 == 0 {
                        Color32::TRANSPARENT
                    } else {
                        STRIPE_ODD
                    };
                    painter.rect_filled(rect, 2.0, bg);

                    if is_active {
                        painter.rect_stroke(rect, 2.0, Stroke::new(1.0, ACCENT_DIM), StrokeKind::Inside);
                    }

                    // Cue number
                    painter.text(
                        Pos2::new(rect.min.x + 8.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &cue.id,
                        font_encoder_readout(),
                        if is_active { ACCENT } else { FG_MUTED },
                    );

                    // Label
                    painter.text(
                        Pos2::new(rect.min.x + 40.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &cue.label,
                        font_status(),
                        if is_active { FG } else { FG_SECONDARY },
                    );

                    // Fade time
                    let fade = if cue.fade_in_ms > 0 {
                        format!("{}s", cue.fade_in_ms as f32 / 1000.0)
                    } else {
                        "—".into()
                    };
                    painter.text(
                        Pos2::new(rect.max.x - 8.0, rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        fade,
                        font_hint(),
                        FG_FAINT,
                    );
                }

                if response.clicked() {
                    // TODO: load cue into programmer for editing
                }
                if response.secondary_clicked() {
                    commands.trigger(DeleteCueEvent(i));
                }
            }
        });

    // ── Keyboard shortcuts ────────────────────────────────────────────────────
    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if ui.input(|i| i.modifiers.shift) {
            commands.trigger(BackCueEvent);
        } else {
            commands.trigger(GoCueEvent);
        }
    }
}
