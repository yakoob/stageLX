use bevy::prelude::ResMut;
use bevy_egui::{egui, EguiContexts};
use crate::IoConfig;

pub fn io_panel(mut ctx: EguiContexts, mut cfg: ResMut<IoConfig>) {
    egui::Window::new("DMX I/O")
        .default_pos([1300.0, 10.0])
        .default_width(260.0)
        .resizable(false)
        .show(&ctx.ctx_mut().expect("egui context"), |ui| {
            // ── Art-Net ───────────────────────────────────────────────────────
            ui.label(
                egui::RichText::new("ART-NET")
                    .strong()
                    .color(egui::Color32::from_rgb(120, 220, 255)),
            );

            ui.horizontal(|ui| {
                ui.label("Bind:");
                ui.add(
                    egui::TextEdit::singleline(&mut cfg.artnet_ip)
                        .hint_text("0.0.0.0")
                        .desired_width(110.0),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Dest:");
                ui.add(
                    egui::TextEdit::singleline(&mut cfg.artnet_dest_ip)
                        .hint_text("255.255.255.255")
                        .desired_width(110.0),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Universe:");
                ui.add(egui::DragValue::new(&mut cfg.artnet_out_universe).range(0_u16..=32767_u16));
            });

            ui.checkbox(&mut cfg.artnet_rx_enabled, "Enable RX");
            ui.horizontal(|ui| {
                ui.label("Allow src:");
                ui.add(
                    egui::TextEdit::singleline(&mut cfg.artnet_allowed_sources)
                        .hint_text("any  (e.g. 192.168.1.10,192.168.1.11)")
                        .desired_width(180.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&cfg.artnet_status).small().color(egui::Color32::LIGHT_GRAY));
            });
            ui.monospace(format!("TX {}  RX {}", cfg.artnet_tx_count, cfg.artnet_rx_count));

            ui.add_space(6.0);
            ui.separator();

            // ── sACN (E1.31) ──────────────────────────────────────────────────
            ui.label(
                egui::RichText::new("sACN  (E1.31)")
                    .strong()
                    .color(egui::Color32::from_rgb(180, 255, 180)),
            );

            ui.horizontal(|ui| {
                ui.checkbox(&mut cfg.sacn_tx_enabled, "TX");
                ui.checkbox(&mut cfg.sacn_rx_enabled, "RX");
            });

            ui.horizontal(|ui| {
                ui.label("Universe:");
                ui.add(egui::DragValue::new(&mut cfg.sacn_out_universe).range(1_u16..=63999_u16));
                ui.label("Pri:");
                ui.add(egui::DragValue::new(&mut cfg.sacn_priority).range(1_u8..=200_u8));
            });
            ui.horizontal(|ui| {
                ui.label("Dest:");
                ui.add(
                    egui::TextEdit::singleline(&mut cfg.sacn_dest_ip)
                        .hint_text("239.255.X.X (multicast)")
                        .desired_width(160.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&cfg.sacn_status).small().color(egui::Color32::LIGHT_GRAY));
            });
            ui.monospace(format!("TX {}  RX {}", cfg.sacn_tx_count, cfg.sacn_rx_count));

            ui.add_space(6.0);
            ui.separator();

            // ── USB DMX (Enttec USB Pro) ───────────────────────────────────────
            ui.label(
                egui::RichText::new("USB DMX  (Enttec)")
                    .strong()
                    .color(egui::Color32::from_rgb(255, 220, 100)),
            );

            ui.checkbox(&mut cfg.usb_tx_enabled, "Enable TX");

            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.add(
                    egui::TextEdit::singleline(&mut cfg.usb_port)
                        .hint_text("/dev/tty.usbserial-… or COM3")
                        .desired_width(170.0),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Universe:");
                ui.add(egui::DragValue::new(&mut cfg.usb_universe).range(1_u16..=32767_u16));
            });

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&cfg.usb_status).small().color(egui::Color32::LIGHT_GRAY));
            });
            ui.monospace(format!("TX {}", cfg.usb_tx_count));

            ui.add_space(6.0);
            ui.separator();

            // ── MIDI Input ────────────────────────────────────────────────────
            ui.label(
                egui::RichText::new("MIDI Input")
                    .strong()
                    .color(egui::Color32::from_rgb(255, 180, 100)),
            );

            ui.checkbox(&mut cfg.midi_enabled, "Enable");
            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.add(
                    egui::TextEdit::singleline(&mut cfg.midi_port)
                        .hint_text("Exact port name…")
                        .desired_width(160.0),
                );
            });

            ui.label(egui::RichText::new("CC Mapping").small().strong());
            egui::Grid::new("midi_cc_grid")
                .num_columns(4)
                .spacing([6.0, 2.0])
                .show(ui, |ui| {
                    ui.label("Dimmer"); ui.add(egui::DragValue::new(&mut cfg.midi_cc_dimmer).range(0_u8..=127_u8));
                    ui.label("Pan");    ui.add(egui::DragValue::new(&mut cfg.midi_cc_pan).range(0_u8..=127_u8));
                    ui.end_row();
                    ui.label("Tilt");   ui.add(egui::DragValue::new(&mut cfg.midi_cc_tilt).range(0_u8..=127_u8));
                    ui.label("Zoom");   ui.add(egui::DragValue::new(&mut cfg.midi_cc_zoom).range(0_u8..=127_u8));
                    ui.end_row();
                    ui.label("Red");    ui.add(egui::DragValue::new(&mut cfg.midi_cc_red).range(0_u8..=127_u8));
                    ui.label("Green");  ui.add(egui::DragValue::new(&mut cfg.midi_cc_green).range(0_u8..=127_u8));
                    ui.end_row();
                    ui.label("Blue");   ui.add(egui::DragValue::new(&mut cfg.midi_cc_blue).range(0_u8..=127_u8));
                    ui.label("Strobe"); ui.add(egui::DragValue::new(&mut cfg.midi_cc_strobe).range(0_u8..=127_u8));
                    ui.end_row();
                });

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&cfg.midi_status).small().color(egui::Color32::LIGHT_GRAY));
            });
            ui.monospace(format!("RX {}", cfg.midi_rx_count));

            ui.add_space(6.0);
            ui.separator();

            // ── OSC Input ─────────────────────────────────────────────────────
            ui.label(
                egui::RichText::new("OSC Input  (UDP)")
                    .strong()
                    .color(egui::Color32::from_rgb(180, 255, 120)),
            );

            ui.checkbox(&mut cfg.osc_enabled, "Enable");
            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.add(egui::DragValue::new(&mut cfg.osc_port).range(1024_u16..=65535_u16));
            });
            ui.label(
                egui::RichText::new("/fixture/{id}/{attr}  f32")
                    .small()
                    .color(egui::Color32::GRAY),
            );

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&cfg.osc_status).small().color(egui::Color32::LIGHT_GRAY));
            });
            ui.monospace(format!("RX {}", cfg.osc_rx_count));
        });
}
