use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use stagelx_gdtf::parse_mvr;
use stagelx_render::{VenueLoadState, VenueRoot, load_venue};
use crate::{FixtureLibraryRes, PatchRes, SpawnFixtureEvent};

pub fn library_panel(
    mut ctx: EguiContexts,
    mut res: ResMut<FixtureLibraryRes>,
    mut patch: ResMut<PatchRes>,
    mut venue_state: ResMut<VenueLoadState>,
    venue_query: Query<Entity, With<VenueRoot>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    egui::Window::new("Fixture Library")
        .default_pos([10.0, 370.0])
        .default_width(370.0)
        .default_height(280.0)
        .resizable(true)
        .show(&ctx.ctx_mut().expect("egui context"), |ui| {
            // ── Loaded fixture list ───────────────────────────────────────────
            if res.library.is_empty() {
                ui.label(egui::RichText::new("No GDTF fixtures loaded.").color(egui::Color32::GRAY));
            } else {
                egui::ScrollArea::vertical()
                    .id_salt("lib_scroll")
                    .max_height(120.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Grid::new("lib_grid")
                            .num_columns(3)
                            .striped(true)
                            .spacing([10.0, 3.0])
                            .show(ui, |ui| {
                                ui.strong("Manufacturer");
                                ui.strong("Name");
                                ui.strong("Modes");
                                ui.end_row();

                                for ft in res.library.all() {
                                    ui.label(&ft.manufacturer);
                                    ui.label(&ft.name);
                                    ui.monospace(format!("{}", ft.dmx_modes.len()));
                                    ui.end_row();
                                }
                            });
                    });
            }

            ui.separator();

            // ── Import GDTF ───────────────────────────────────────────────────
            ui.label(egui::RichText::new("Import GDTF").strong());
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut res.import_path)
                        .hint_text("Path to .gdtf file…")
                        .desired_width(250.0),
                );
                if ui.button("Load").clicked() {
                    load_gdtf(&mut res);
                }
            });

            if let Some(ref err) = res.import_error.clone() {
                ui.colored_label(egui::Color32::from_rgb(255, 80, 80), err);
            }

            ui.separator();

            // ── Import MVR ────────────────────────────────────────────────────
            ui.label(egui::RichText::new("Import MVR").strong());
            ui.label(
                egui::RichText::new("Loads embedded GDTFs and populates the patch from the scene.")
                    .small()
                    .color(egui::Color32::GRAY),
            );
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut res.mvr_import_path)
                        .hint_text("Path to .mvr file…")
                        .desired_width(250.0),
                );
                if ui.button("Import").clicked() {
                    load_mvr(&mut res, &mut patch, &mut commands);
                }
            });

            if let Some(ref err) = res.mvr_import_error.clone() {
                ui.colored_label(egui::Color32::from_rgb(255, 80, 80), err);
            }

            ui.separator();

            // ── Load Stage / Venue ─────────────────────────────────────────────
            ui.label(egui::RichText::new("Stage / Venue").strong());
            ui.label(
                egui::RichText::new("OBJ or GLB/glTF — replaces any previously loaded venue.")
                    .small()
                    .color(egui::Color32::GRAY),
            );
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut venue_state.import_path)
                        .hint_text("Path to .obj or .glb file…")
                        .desired_width(250.0),
                );
                if ui.button("Load").clicked() {
                    let path = venue_state.import_path.trim().to_string();
                    if path.is_empty() {
                        venue_state.import_error = Some("Please enter a file path.".into());
                    } else {
                        match load_venue(&path, &mut commands, &mut meshes, &mut materials, &venue_query) {
                            Ok(()) => {
                                venue_state.import_error = None;
                                venue_state.import_path.clear();
                            }
                            Err(e) => venue_state.import_error = Some(e),
                        }
                    }
                }
            });
            if let Some(ref err) = venue_state.import_error.clone() {
                ui.colored_label(egui::Color32::from_rgb(255, 80, 80), err);
            }
        });
}

fn load_gdtf(res: &mut FixtureLibraryRes) {
    let path = res.import_path.trim().to_string();
    if path.is_empty() {
        res.import_error = Some("Please enter a file path.".into());
        return;
    }
    match std::fs::read(&path) {
        Ok(data) => match res.library.load(&data) {
            Ok(_) => {
                res.import_error = None;
                res.import_path.clear();
            }
            Err(e) => res.import_error = Some(format!("Parse error: {e}")),
        },
        Err(e) => res.import_error = Some(format!("Cannot read file: {e}")),
    }
}

fn load_mvr(
    res: &mut FixtureLibraryRes,
    patch: &mut PatchRes,
    commands: &mut Commands,
) {
    let path = res.mvr_import_path.trim().to_string();
    if path.is_empty() {
        res.mvr_import_error = Some("Please enter an MVR file path.".into());
        return;
    }

    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(e) => {
            res.mvr_import_error = Some(format!("Cannot read file: {e}"));
            return;
        }
    };

    let scene = match parse_mvr(&data) {
        Ok(s) => s,
        Err(e) => {
            res.mvr_import_error = Some(format!("MVR parse error: {e}"));
            return;
        }
    };

    // Load all embedded GDTFs; build filename → type_id map for address fixup.
    let mut name_to_id: std::collections::HashMap<String, String> = Default::default();
    for (filename, bytes) in &scene.gdtf_files {
        match res.library.load(bytes) {
            Ok(type_id) => {
                // Key by the bare filename (without directory prefix) for matching.
                let key = filename.rsplit('/').next().unwrap_or(filename).to_string();
                name_to_id.insert(key, type_id);
            }
            Err(e) => {
                warn!("MVR: failed to load embedded GDTF '{}': {e}", filename);
            }
        }
    }

    // Add fixture instances from the MVR scene to the patch.
    let mut count = 0usize;
    for mut inst in scene.fixture_instances {
        // If the fixture_type_id is a filename rather than a GUID, look it up.
        if let Some(real_id) = name_to_id.get(&inst.fixture_type_id) {
            inst.fixture_type_id = real_id.clone();
        }
        // Verify the type is actually in the library.
        if res.library.get(&inst.fixture_type_id).is_none() {
            warn!("MVR: fixture '{}' references unknown type '{}'", inst.name, inst.fixture_type_id);
            continue;
        }
        let id = patch.0.add(inst);
        commands.trigger(SpawnFixtureEvent(id));
        count += 1;
    }

    res.mvr_import_error = None;
    res.mvr_import_path.clear();
    info!("MVR import complete: {} fixtures added from '{}'", count, path);
}
