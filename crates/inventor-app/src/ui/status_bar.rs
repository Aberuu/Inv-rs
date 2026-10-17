use crate::state::AppState;
use egui::{Color32, RichText, Ui};

pub fn render_status_bar(ui: &mut Ui, state: &AppState) {
    ui.horizontal(|ui| {
        // Status message
        ui.label(RichText::new(&state.status_message).color(Color32::from_rgb(226, 232, 240)));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Camera position details
            let cam_text = format!(
                "Yaw: {:.1}°, Pitch: {:.1}°, Dist: {:.1} mm",
                state.camera.yaw.to_degrees(),
                state.camera.pitch.to_degrees(),
                state.camera.distance
            );
            ui.label(RichText::new(cam_text).weak().small());

            ui.separator();

            // Mesh info
            if let Some(mesh) = &state.active_mesh {
                let stats = format!(
                    "Triangles: {} | Edges: {}",
                    mesh.triangle_count(),
                    mesh.edge_count()
                );
                ui.label(
                    RichText::new(stats)
                        .color(Color32::from_rgb(56, 189, 248))
                        .small(),
                );
            } else {
                ui.label(RichText::new("Empty Model").weak().small());
            }
        });
    });
}
