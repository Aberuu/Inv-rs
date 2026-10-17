mod app;
mod commands;
pub mod sketch_session;
mod state;
mod ui;

use app::InventorApp;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("inventor-rs : Parametric 3D Solid Modeler"),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "inventor-rs",
        native_options,
        Box::new(|cc| Ok(Box::new(InventorApp::new(cc)))),
    )
}
