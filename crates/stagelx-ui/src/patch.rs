use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use stagelx_core::{fixture::FixtureInstance, types::{DmxAddress, FixtureId}};
use crate::{FixtureLibraryRes, PatchEditState, PatchRes, SpawnFixtureEvent};

pub fn patch_panel(
    mut ctx: EguiContexts,
    mut patch: ResMut<PatchRes>,
    library: Res<FixtureLibraryRes>,
    mut edit: ResMut<PatchEditState>,
    mut commands: Commands,
) {
    egui::Window::new("Patch")
        .default_pos([290.0, 10.0])
        .default_width(520.0)
        .default_height(300.0)
        .resizable(true)
        .show(&ctx.ctx_mut().expect("egui context"), |ui| {
            let count = patch.0.len();

            // ── Fixture list ─────────────────────────────────────────────────
            egui::ScrollArea::vertical()
                .max_height(180.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("patch_grid")
                        .num_columns(6)
                        .striped(true)
                        .spacing([10.0, 3.0])
                        .show(ui, |ui| {
                            ui.strong("#");
                            ui.strong("Name");
                            ui.strong("Fixture Type");
                            ui.strong("Mode");
                            ui.strong("Univ");
                            ui.strong("Ch");
                            ui.end_row();

                            let mut fixtures: Vec<_> = patch.0.fixtures().collect();
                            fixtures.sort_by_key(|f| f.id.0);

                            for f in fixtures {
                                ui.monospace(
                                    egui::RichText::new(format!("{:>3}", f.id.0 + 1))
                                        .color(egui::Color32::from_rgb(150, 200, 255)),
                                );
                                ui.label(&f.name);
                                ui.label(truncate(&f.fixture_type_id, 22));
                                ui.label(truncate(&f.dmx_mode, 12));
                                ui.monospace(format!("{}", f.address.universe));
                                ui.monospace(format!("{}", f.address.channel));
                                ui.end_row();
                            }
                        });
                });

            ui.separator();

            // ── Add Fixture form ──────────────────────────────────────────────
            ui.label(egui::RichText::new("Add Fixture").strong());

            // Fixture type selector
            let type_ids: Vec<String> = library.library.all()
                .map(|ft| ft.fixture_type_id.clone())
                .collect();
            let type_labels: Vec<String> = library.library.all()
                .map(|ft| format!("{} — {}", ft.manufacturer, ft.name))
                .collect();

            if type_ids.is_empty() {
                ui.colored_label(
                    egui::Color32::GRAY,
                    "Load a GDTF fixture first (Fixture Library panel).",
                );
            } else {
                // Ensure selection is valid
                if !type_ids.contains(&edit.selected_type_id) {
                    edit.selected_type_id = type_ids[0].clone();
                }

                egui::ComboBox::from_label("Fixture Type")
                    .selected_text(
                        type_labels
                            .get(type_ids.iter().position(|id| *id == edit.selected_type_id).unwrap_or(0))
                            .cloned()
                            .unwrap_or_default(),
                    )
                    .show_ui(ui, |ui| {
                        for (id, label) in type_ids.iter().zip(type_labels.iter()) {
                            ui.selectable_value(&mut edit.selected_type_id, id.clone(), label);
                        }
                    });

                // Mode selector
                let modes: Vec<String> = library
                    .library
                    .get(&edit.selected_type_id)
                    .map(|ft| ft.dmx_modes.iter().map(|m| m.name.clone()).collect())
                    .unwrap_or_default();

                if !modes.is_empty() {
                    if !modes.contains(&edit.selected_mode) {
                        edit.selected_mode = modes[0].clone();
                    }
                    egui::ComboBox::from_label("Mode")
                        .selected_text(&edit.selected_mode)
                        .show_ui(ui, |ui| {
                            for m in &modes {
                                ui.selectable_value(&mut edit.selected_mode, m.clone(), m);
                            }
                        });
                }

                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.add(egui::TextEdit::singleline(&mut edit.new_name).desired_width(120.0));
                    ui.label("Univ");
                    ui.add(egui::TextEdit::singleline(&mut edit.universe_str).desired_width(40.0));
                    ui.label("Ch");
                    ui.add(egui::TextEdit::singleline(&mut edit.channel_str).desired_width(40.0));
                });

                if ui.button("Add to Patch").clicked() {
                    match add_fixture(&mut patch, &mut edit, &mut commands) {
                        Ok(()) => {}
                        Err(e) => edit.add_error = Some(e),
                    }
                }

                if let Some(ref err) = edit.add_error.clone() {
                    ui.colored_label(egui::Color32::from_rgb(255, 80, 80), err);
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "{} fixture{} patched",
                        count,
                        if count == 1 { "" } else { "s" }
                    ))
                    .color(egui::Color32::GRAY)
                    .small(),
                );
            });
        });
}

fn add_fixture(
    patch: &mut PatchRes,
    edit: &mut PatchEditState,
    commands: &mut Commands,
) -> Result<(), String> {
    let universe: u16 = edit.universe_str.trim().parse()
        .map_err(|_| "Universe must be a number (1–32767)".to_string())?;
    let channel: u16 = edit.channel_str.trim().parse()
        .map_err(|_| "Channel must be 1–512".to_string())?;

    if universe == 0 {
        return Err("Universe must be ≥ 1".into());
    }
    if channel == 0 || channel > 512 {
        return Err("Channel must be 1–512".into());
    }
    if edit.selected_type_id.is_empty() {
        return Err("No fixture type selected".into());
    }

    let name = if edit.new_name.trim().is_empty() {
        format!("Fixture {}", patch.0.len() + 1)
    } else {
        edit.new_name.trim().to_string()
    };

    let id = patch.0.add(FixtureInstance {
        id: FixtureId(0),
        name,
        fixture_type_id: edit.selected_type_id.clone(),
        dmx_mode: edit.selected_mode.clone(),
        address: DmxAddress::new(universe, channel),
        position: [0.0, 6.0, 0.0],
        rotation: [0.0, 0.0, 0.0],
    });

    commands.trigger(SpawnFixtureEvent(id));

    edit.new_name.clear();
    edit.universe_str.clear();
    edit.channel_str.clear();
    edit.add_error = None;
    Ok(())
}

fn truncate(s: &str, max: usize) -> &str {
    match s.char_indices().nth(max) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}
