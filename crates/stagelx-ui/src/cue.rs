//! Cue panel — list view, GO/BACK/RECORD, keyboard shortcuts.

use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, Pos2, RichText, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::theme::*;
// use crate::widgets;
use stagelx_show::{
    BackCueEvent, CaptureMode, CuePlayhead, CueStack, DeleteCueEvent, GoCueEvent,
    LoadCueIntoProgrammerEvent, PlayheadState, RecordCueEvent, RecordStageCueEvent,
    SaveShowEvent, UpdateCueEvent,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Cue panel (docked / inline)
// ═══════════════════════════════════════════════════════════════════════════════

pub fn cue_panel_docked(
    ui: &mut Ui,
    stack: &mut CueStack,
    playhead: &CuePlayhead,
    capture_mode: &mut CaptureMode,
    commands: &mut Commands,
) {
    // ── Capture mode toggle ───────────────────────────────────────────────────
    ui.horizontal(|ui| {
        let w = ui.available_width();
        let item_spacing_x = ui.spacing().item_spacing.x;
        let btn_w = (w - item_spacing_x - 4.0) / 2.0;

        let pgm_active = *capture_mode == CaptureMode::Programmer;
        let stage_active = *capture_mode == CaptureMode::Stage;

        let pgm_btn = egui::Button::new(RichText::new("PGM").size(10.0).strong())
            .fill(if pgm_active { ACCENT_BG } else { BG_RAISED })
            .stroke(Stroke::new(1.0, if pgm_active { ACCENT_DIM } else { BORDER }))
            .min_size(Vec2::new(btn_w, 22.0));
        if ui.add_sized([btn_w, 22.0], pgm_btn).clicked() {
            *capture_mode = CaptureMode::Programmer;
        }

        ui.add_space(4.0);

        let stage_btn = egui::Button::new(RichText::new("STAGE").size(10.0).strong())
            .fill(if stage_active { ACCENT_BG } else { BG_RAISED })
            .stroke(Stroke::new(1.0, if stage_active { ACCENT_DIM } else { BORDER }))
            .min_size(Vec2::new(btn_w, 22.0));
        if ui.add_sized([btn_w, 22.0], stage_btn).clicked() {
            *capture_mode = CaptureMode::Stage;
        }
    });

    ui.add_space(2.0);

    // ── Toolbar ───────────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        let w = ui.available_width();
        let item_spacing_x = ui.spacing().item_spacing.x;
        let btn_w = (w - 3.0 * item_spacing_x - 3.0 * 4.0) / 4.0;

        // BACK
        let back_btn = egui::Button::new(RichText::new("◀ BACK").size(10.0).strong().color(FG))
            .fill(BG_RAISED)
            .stroke(Stroke::new(1.0, BORDER))
            .min_size(Vec2::new(btn_w, 28.0));
        if ui.add_sized([btn_w, 28.0], back_btn).clicked() {
            commands.trigger(BackCueEvent);
        }
        ui.add_space(4.0);

        // GO
        let go_btn = egui::Button::new(RichText::new("GO ▶").size(10.0).strong().color(FG))
            .fill(ACCENT_BG)
            .stroke(Stroke::new(1.0, ACCENT_DIM))
            .min_size(Vec2::new(btn_w, 28.0));
        if ui.add_sized([btn_w, 28.0], go_btn).clicked() {
            commands.trigger(GoCueEvent);
        }
        ui.add_space(4.0);

        // RECORD
        let rec_label = if *capture_mode == CaptureMode::Stage {
            "● STAGE"
        } else {
            "● REC"
        };
        let rec_btn = egui::Button::new(RichText::new(rec_label).size(10.0).strong().color(RX))
            .fill(BG_RAISED)
            .stroke(Stroke::new(1.0, BORDER))
            .min_size(Vec2::new(btn_w, 28.0));
        if ui.add_sized([btn_w, 28.0], rec_btn).clicked() {
            if *capture_mode == CaptureMode::Stage {
                commands.trigger(RecordStageCueEvent);
            } else {
                commands.trigger(RecordCueEvent);
            }
        }
        ui.add_space(4.0);

        // UPDATE
        let update_btn = egui::Button::new(RichText::new("UPDATE").size(10.0).strong().color(FG))
            .fill(BG_RAISED)
            .stroke(Stroke::new(1.0, BORDER))
            .min_size(Vec2::new(btn_w, 28.0));
        if ui.add_sized([btn_w, 28.0], update_btn).clicked() {
            commands.trigger(UpdateCueEvent);
        }
    });

    ui.add_space(4.0);

    // ── Cue list ──────────────────────────────────────────────────────────────
    let active_idx = playhead.current_cue_index;

    // Compute fade progress for visual indicator.
    let fade_progress: Option<f32> = match &playhead.state {
        PlayheadState::Fading { start, duration_ms, .. } => {
            let elapsed = start.elapsed().as_secs_f32() * 1000.0;
            Some((elapsed / *duration_ms as f32).clamp(0.0, 1.0))
        }
        PlayheadState::Idle => None,
    };

    egui::ScrollArea::vertical()
        .max_height(120.0)
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for (i, cue) in stack.cues.iter().enumerate() {
                let is_active = active_idx == Some(i);
                let row_h = if is_active && fade_progress.is_some() { 32.0 } else { 28.0 };
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
                    let text_y = if is_active && fade_progress.is_some() {
                        rect.min.y + 14.0
                    } else {
                        rect.center().y
                    };
                    painter.text(
                        Pos2::new(rect.min.x + 8.0, text_y),
                        egui::Align2::LEFT_CENTER,
                        &cue.id,
                        font_encoder_readout(),
                        if is_active { ACCENT } else { FG_MUTED },
                    );

                    // Label
                    painter.text(
                        Pos2::new(rect.min.x + 40.0, text_y),
                        egui::Align2::LEFT_CENTER,
                        &cue.label,
                        font_status(),
                        if is_active { FG } else { FG_SECONDARY },
                    );

                    // Fade time (or progress when active and fading)
                    let right_text = if is_active {
                        if let Some(t) = fade_progress {
                            format!("{:.0}%", t * 100.0)
                        } else if cue.fade_in_ms > 0 {
                            format!("{}s", cue.fade_in_ms as f32 / 1000.0)
                        } else {
                            "—".into()
                        }
                    } else if cue.fade_in_ms > 0 {
                        format!("{}s", cue.fade_in_ms as f32 / 1000.0)
                    } else {
                        "—".into()
                    };
                    painter.text(
                        Pos2::new(rect.max.x - 8.0, text_y),
                        egui::Align2::RIGHT_CENTER,
                        right_text,
                        font_hint(),
                        if is_active && fade_progress.is_some() { ACCENT } else { FG_FAINT },
                    );

                    // Fade progress bar on active cue
                    if is_active {
                        if let Some(t) = fade_progress {
                            let bar_h = 3.0;
                            let bar_y = rect.max.y - bar_h - 2.0;
                            let bar_w = rect.width() - 16.0;
                            let bar_rect = egui::Rect::from_min_size(
                                Pos2::new(rect.min.x + 8.0, bar_y),
                                Vec2::new(bar_w, bar_h),
                            );
                            painter.rect_filled(bar_rect, 1.5, BG_RAISED);
                            let fill_w = bar_w * t;
                            let fill_rect = egui::Rect::from_min_size(
                                Pos2::new(bar_rect.min.x, bar_y),
                                Vec2::new(fill_w, bar_h),
                            );
                            painter.rect_filled(fill_rect, 1.5, ACCENT);
                        }
                    }
                }

                if response.clicked() {
                    commands.trigger(LoadCueIntoProgrammerEvent(i));
                }
                if response.secondary_clicked() {
                    commands.trigger(DeleteCueEvent(i));
                }
            }
        });

    // ── Selected cue detail editor ────────────────────────────────────────────
    if let Some(idx) = playhead.current_cue_index {
        if idx < stack.cues.len() {
            ui.separator();

            // Collect current values before the closures.
            let mut new_label = stack.cues[idx].label.clone();
            let mut fade_in_s = stack.cues[idx].fade_in_ms as f32 / 1000.0;
            let mut fade_out_s = stack.cues[idx].fade_out_ms as f32 / 1000.0;

            // Label edit
            ui.horizontal(|ui| {
                ui.label(RichText::new("Label").size(11.0).color(FG_MUTED));
                ui.add(egui::TextEdit::singleline(&mut new_label).font(font_status()));
            });

            ui.add_space(2.0);

            // Fade times
            ui.horizontal(|ui| {
                ui.label(RichText::new("Fade").size(11.0).color(FG_MUTED));

                ui.add(
                    egui::DragValue::new(&mut fade_in_s)
                        .speed(0.1)
                        .range(0.0..=60.0)
                        .suffix("s"),
                );

                ui.label(RichText::new("→").size(11.0).color(FG_FAINT));

                ui.add(
                    egui::DragValue::new(&mut fade_out_s)
                        .speed(0.1)
                        .range(0.0..=60.0)
                        .suffix("s"),
                );
            });

            // Apply changes outside the closures.
            let cue = &mut stack.cues[idx];
            let mut changed = false;
            if new_label != cue.label {
                cue.label = new_label;
                changed = true;
            }
            let new_fade_in = (fade_in_s * 1000.0) as u32;
            if new_fade_in != cue.fade_in_ms {
                cue.fade_in_ms = new_fade_in;
                changed = true;
            }
            let new_fade_out = (fade_out_s * 1000.0) as u32;
            if new_fade_out != cue.fade_out_ms {
                cue.fade_out_ms = new_fade_out;
                changed = true;
            }
            if changed {
                commands.trigger(SaveShowEvent);
            }
        }
    }

    // ── Keyboard shortcuts ────────────────────────────────────────────────────
    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if ui.input(|i| i.modifiers.shift) {
            commands.trigger(BackCueEvent);
        } else {
            commands.trigger(GoCueEvent);
        }
    }
}
