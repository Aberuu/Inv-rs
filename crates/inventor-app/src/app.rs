use crate::state::{ActiveModal, AppState};
use crate::ui::{browser, dialogs, ribbon, status_bar, viewport};
use eframe::egui::{self, CentralPanel, Key, MenuBar, Panel};

#[derive(Default)]
pub struct InventorApp {
    pub state: AppState,
}

impl InventorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut state = AppState::default();
        if let Some(wgpu_state) = &cc.wgpu_render_state {
            log::info!("Hardware wgpu render state detected, initializing WgpuCadRenderer");
            let renderer =
                inventor_render::WgpuCadRenderer::new(&wgpu_state.device, wgpu_state.target_format);
            state.wgpu_renderer = Some(std::sync::Arc::new(std::sync::Mutex::new(renderer)));
            state.wgpu_render_state = Some(wgpu_state.clone());
        }
        Self { state }
    }
}

impl eframe::App for InventorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let registry = self.state.command_registry.clone();

        // Keyboard shortcuts routed via CommandRegistry
        ctx.input(|i| {
            if i.key_pressed(Key::F6) {
                let _ = registry.execute("view.home", &mut self.state);
            }
            if i.key_pressed(Key::Escape) {
                self.state.active_modal = ActiveModal::None;
                self.state.status_message = "Cancelled active tool.".into();
            }
            if i.modifiers.command && i.key_pressed(Key::Z) {
                let _ = registry.execute("edit.undo", &mut self.state);
            }
            if i.modifiers.command && i.key_pressed(Key::Y) {
                let _ = registry.execute("edit.redo", &mut self.state);
            }
            if i.modifiers.command && i.key_pressed(Key::N) {
                let _ = registry.execute("file.new", &mut self.state);
            }
        });

        // 1. Top Panel: Main Menu Bar & Ribbon
        Panel::top("top_panel").show(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Part (Ctrl+N)").clicked() {
                        let _ = registry.execute("file.new", &mut self.state);
                        ui.close();
                    }
                    ui.separator();
                    let can_step = registry.is_enabled("file.export_step", &self.state);
                    if ui
                        .add_enabled(can_step, egui::Button::new("Export STEP (ISO 10303)..."))
                        .clicked()
                    {
                        let _ = registry.execute("file.export_step", &mut self.state);
                        ui.close();
                    }

                    let can_stl = registry.is_enabled("file.export_stl", &self.state);
                    if ui
                        .add_enabled(can_stl, egui::Button::new("Export STL Mesh..."))
                        .clicked()
                    {
                        let _ = registry.execute("file.export_stl", &mut self.state);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        let _ = registry.execute("file.exit", &mut self.state);
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    let can_undo = registry.is_enabled("edit.undo", &self.state);
                    if ui
                        .add_enabled(can_undo, egui::Button::new("Undo (Ctrl+Z)"))
                        .clicked()
                    {
                        let _ = registry.execute("edit.undo", &mut self.state);
                        ui.close();
                    }

                    let can_redo = registry.is_enabled("edit.redo", &self.state);
                    if ui
                        .add_enabled(can_redo, egui::Button::new("Redo (Ctrl+Y)"))
                        .clicked()
                    {
                        let _ = registry.execute("edit.redo", &mut self.state);
                        ui.close();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About inventor-rs").clicked() {
                        let _ = registry.execute("help.about", &mut self.state);
                        ui.close();
                    }
                });

                ui.separator();

                // Quick Access Toolbar routed via CommandRegistry
                if ui.button("📄").on_hover_text("New Part (Ctrl+N)").clicked() {
                    let _ = registry.execute("file.new", &mut self.state);
                }

                ui.add_enabled(false, egui::Button::new("📂"))
                    .on_hover_text("Belum tersedia");

                ui.add_enabled(false, egui::Button::new("💾"))
                    .on_hover_text("Belum tersedia");

                let can_undo = registry.is_enabled("edit.undo", &self.state);
                if ui
                    .add_enabled(can_undo, egui::Button::new("↶"))
                    .on_hover_text(if can_undo {
                        "Undo (Ctrl+Z)"
                    } else {
                        "Nothing to undo"
                    })
                    .clicked()
                {
                    let _ = registry.execute("edit.undo", &mut self.state);
                }

                let can_redo = registry.is_enabled("edit.redo", &self.state);
                if ui
                    .add_enabled(can_redo, egui::Button::new("↷"))
                    .on_hover_text(if can_redo {
                        "Redo (Ctrl+Y)"
                    } else {
                        "Nothing to redo"
                    })
                    .clicked()
                {
                    let _ = registry.execute("edit.redo", &mut self.state);
                }

                if ui
                    .button("🏠")
                    .on_hover_text("Reset Camera Home View (F6)")
                    .clicked()
                {
                    let _ = registry.execute("view.home", &mut self.state);
                }
            });

            ui.separator();

            // Ribbon Toolbar
            ribbon::render_ribbon(ui, &mut self.state);
        });

        // 2. Bottom Panel: Status Bar
        Panel::bottom("bottom_panel").show(ui, |ui| {
            status_bar::render_status_bar(ui, &self.state);
        });

        // 3. Left Panel: Model Browser
        Panel::left("browser_panel")
            .resizable(true)
            .default_size(220.0)
            .show(ui, |ui| {
                browser::render_browser(ui, &mut self.state);
            });

        // 4. Central Panel: 3D Viewport
        CentralPanel::default().show(ui, |ui| {
            viewport::render_viewport(ui, &mut self.state);
        });

        // 5. Active Modals & Dialogs
        dialogs::render_dialogs(&ctx, &mut self.state);
    }
}
