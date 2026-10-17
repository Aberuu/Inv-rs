use crate::state::AppState;
use crate::ui::ribbon_schema::{RibbonButtonSize, RibbonItem};
use egui::{Color32, RichText, Ui};

/// Renders the data-driven ribbon UI.
pub fn render_ribbon(ui: &mut Ui, state: &mut AppState) {
    let mut switch_to_tab: Option<String> = None;
    let mut command_to_run: Option<String> = None;
    let mut clear_search = false;

    ui.vertical(|ui| {
        // 1. Tab Headers & Command Search Bar
        ui.horizontal(|ui| {
            ui.visuals_mut().button_frame = true;

            // Render Tab Buttons from Schema
            for tab in &state.ribbon_schema.tabs {
                let is_selected = state.active_tab_id == tab.id;

                if tab.enabled {
                    let text = if is_selected {
                        RichText::new(&tab.label)
                            .strong()
                            .color(Color32::from_rgb(56, 189, 248))
                    } else {
                        RichText::new(&tab.label).color(Color32::from_rgb(203, 213, 225))
                    };

                    let mut resp = ui.selectable_label(is_selected, text);
                    if let Some(tooltip) = &tab.tooltip {
                        resp = resp.on_hover_text(tooltip);
                    }
                    if resp.clicked() {
                        switch_to_tab = Some(tab.id.clone());
                    }
                } else {
                    let text = RichText::new(&tab.label).color(Color32::from_rgb(100, 116, 139));
                    let tooltip = tab.tooltip.as_deref().unwrap_or("Belum tersedia");
                    ui.add_enabled(false, egui::Button::new(text))
                        .on_hover_text(tooltip);
                }
            }

            // Command Search Box on Right Side of Ribbon
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if !state.command_search_query.is_empty()
                    && ui.small_button("✖").on_hover_text("Clear search").clicked()
                {
                    clear_search = true;
                }

                ui.add(
                    egui::TextEdit::singleline(&mut state.command_search_query)
                        .hint_text("🔍 Search commands...")
                        .desired_width(170.0),
                );
            });
        });

        ui.separator();

        // 2. Ribbon Content Area: Either Search Results or Active Tab Panels
        if !state.command_search_query.trim().is_empty() {
            render_search_results(ui, state, &mut command_to_run);
        } else {
            render_active_tab_panels(ui, state, &mut command_to_run);
        }
    });

    // Apply state changes outside UI closures
    if clear_search {
        state.command_search_query.clear();
    }
    if let Some(tab_id) = switch_to_tab {
        state.active_tab_id = tab_id;
    }
    if let Some(cmd) = command_to_run {
        dispatch_ribbon_command(&cmd, state);
    }
}

/// Renders panels and items for the currently active tab.
fn render_active_tab_panels(ui: &mut Ui, state: &AppState, command_to_run: &mut Option<String>) {
    let current_tab = state
        .ribbon_schema
        .tabs
        .iter()
        .find(|t| t.id == state.active_tab_id)
        .or_else(|| state.ribbon_schema.tabs.iter().find(|t| t.enabled));

    if let Some(tab) = current_tab {
        ui.horizontal(|ui| {
            for panel in &tab.panels {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        // Upper Item Row
                        ui.horizontal(|ui| {
                            for item in &panel.items {
                                render_item(ui, item, state, command_to_run);
                            }
                        });

                        // Lower Panel Title
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new(&panel.title).weak().size(10.0));
                        });
                    });
                });
            }
        });
    }
}

/// Renders a single ribbon item (Large button, small button, dropdown, or separator).
fn render_item(
    ui: &mut Ui,
    item: &RibbonItem,
    state: &AppState,
    command_to_run: &mut Option<String>,
) {
    match item {
        RibbonItem::Button {
            label,
            icon,
            tooltip,
            size,
            command_id,
            enabled,
            ..
        } => {
            let is_cmd_enabled = if let Some(cmd) = command_id {
                state.command_registry.is_enabled(cmd, state)
            } else {
                false
            };
            let effective_enabled = *enabled && is_cmd_enabled;

            let effective_tooltip = if !*enabled {
                if tooltip.is_empty() {
                    "Belum tersedia"
                } else {
                    tooltip.as_str()
                }
            } else if !is_cmd_enabled {
                "Command prerequisites not satisfied in current state"
            } else if tooltip.is_empty() {
                label.as_str()
            } else {
                tooltip.as_str()
            };

            match size {
                RibbonButtonSize::Large => {
                    let text = if let Some(ic) = icon {
                        format!("{}\n{}", ic, label)
                    } else {
                        label.clone()
                    };
                    let btn = egui::Button::new(RichText::new(text).size(11.0))
                        .min_size(egui::vec2(52.0, 46.0));
                    let resp = ui
                        .add_enabled(effective_enabled, btn)
                        .on_hover_text(effective_tooltip);
                    if resp.clicked() {
                        *command_to_run = command_id.clone();
                    }
                }
                RibbonButtonSize::Small => {
                    let text = if let Some(ic) = icon {
                        format!("{} {}", ic, label)
                    } else {
                        label.clone()
                    };
                    let btn = egui::Button::new(RichText::new(text).size(11.0))
                        .min_size(egui::vec2(0.0, 22.0));
                    let resp = ui
                        .add_enabled(effective_enabled, btn)
                        .on_hover_text(effective_tooltip);
                    if resp.clicked() {
                        *command_to_run = command_id.clone();
                    }
                }
            }
        }
        RibbonItem::Dropdown {
            label,
            icon,
            tooltip,
            options,
            enabled,
            ..
        } => {
            let has_any_enabled = options.iter().any(|opt| {
                opt.enabled
                    && opt
                        .command_id
                        .as_deref()
                        .is_none_or(|cmd| state.command_registry.is_enabled(cmd, state))
            });
            let effective_enabled = *enabled && has_any_enabled;

            let effective_tooltip = if !*enabled || !has_any_enabled {
                if tooltip.is_empty() {
                    "Belum tersedia"
                } else {
                    tooltip.as_str()
                }
            } else {
                tooltip.as_str()
            };

            let menu_label = if let Some(ic) = icon {
                format!("{} {} ▾", ic, label)
            } else {
                format!("{} ▾", label)
            };

            if effective_enabled {
                ui.menu_button(RichText::new(menu_label).size(11.0), |ui| {
                    for opt in options {
                        let is_opt_enabled = opt.enabled
                            && opt
                                .command_id
                                .as_deref()
                                .is_none_or(|cmd| state.command_registry.is_enabled(cmd, state));

                        let opt_text = if let Some(ic) = &opt.icon {
                            format!("{} {}", ic, opt.label)
                        } else {
                            opt.label.clone()
                        };
                        let opt_tooltip = if is_opt_enabled {
                            opt.tooltip.as_str()
                        } else if opt.tooltip.is_empty() {
                            "Belum tersedia"
                        } else {
                            opt.tooltip.as_str()
                        };
                        let resp = ui
                            .add_enabled(is_opt_enabled, egui::Button::new(opt_text))
                            .on_hover_text(opt_tooltip);
                        if resp.clicked() {
                            *command_to_run = opt.command_id.clone();
                            ui.close();
                        }
                    }
                })
                .response
                .on_hover_text(effective_tooltip);
            } else {
                let btn = egui::Button::new(RichText::new(menu_label).size(11.0))
                    .min_size(egui::vec2(0.0, 22.0));
                ui.add_enabled(false, btn).on_hover_text(effective_tooltip);
            }
        }
        RibbonItem::Separator => {
            ui.separator();
        }
    }
}

/// Renders search results matching the query string in real-time.
fn render_search_results(ui: &mut Ui, state: &AppState, command_to_run: &mut Option<String>) {
    let results = state
        .ribbon_schema
        .search_items(&state.command_search_query);

    ui.horizontal(|ui| {
        ui.label(RichText::new("Search Results:").strong());

        if results.is_empty() {
            ui.label(RichText::new("No matching ribbon commands found.").weak());
        } else {
            for (tab, panel, item) in results.iter().take(8) {
                match item {
                    RibbonItem::Button {
                        label,
                        command_id,
                        enabled,
                        tooltip,
                        ..
                    } => {
                        let btn_label = format!("{}: {} > {}", tab.label, panel.title, label);
                        let is_cmd_enabled = if let Some(cmd) = command_id {
                            state.command_registry.is_enabled(cmd, state)
                        } else {
                            false
                        };
                        let effective_enabled = *enabled && is_cmd_enabled;

                        let effective_tooltip = if effective_enabled {
                            tooltip.as_str()
                        } else {
                            "Belum tersedia"
                        };

                        let resp = ui
                            .add_enabled(
                                effective_enabled,
                                egui::Button::new(RichText::new(btn_label).size(11.0)),
                            )
                            .on_hover_text(effective_tooltip);
                        if resp.clicked() {
                            *command_to_run = command_id.clone();
                        }
                    }
                    RibbonItem::Dropdown {
                        label,
                        options,
                        enabled,
                        ..
                    } => {
                        let btn_label = format!("{}: {} > {}...", tab.label, panel.title, label);
                        if *enabled {
                            ui.menu_button(RichText::new(btn_label).size(11.0), |ui| {
                                for opt in options {
                                    let is_opt_enabled = opt.enabled
                                        && opt.command_id.as_deref().is_none_or(|cmd| {
                                            state.command_registry.is_enabled(cmd, state)
                                        });

                                    let opt_tooltip = if is_opt_enabled {
                                        opt.tooltip.as_str()
                                    } else {
                                        "Belum tersedia"
                                    };
                                    let resp = ui
                                        .add_enabled(is_opt_enabled, egui::Button::new(&opt.label))
                                        .on_hover_text(opt_tooltip);
                                    if resp.clicked() {
                                        *command_to_run = opt.command_id.clone();
                                        ui.close();
                                    }
                                }
                            });
                        } else {
                            ui.add_enabled(false, egui::Button::new(btn_label))
                                .on_hover_text("Belum tersedia");
                        }
                    }
                    RibbonItem::Separator => {}
                }
            }
        }
    });
}

/// Dispatches ribbon command strings to active modals or application state modifications via CommandRegistry.
pub fn dispatch_ribbon_command(command_id: &str, state: &mut AppState) {
    let registry = state.command_registry.clone();
    if let Err(err) = registry.execute(command_id, state) {
        state.status_message = format!("Command error: {}", err);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ActiveModal;
    use inventor_render::ProjectionType;

    #[test]
    fn test_dispatch_box_command() {
        let mut state = AppState::default();
        dispatch_ribbon_command("model.box", &mut state);

        match state.active_modal {
            ActiveModal::CreateBox {
                width,
                length,
                height,
            } => {
                assert_eq!(width, 50.0);
                assert_eq!(length, 30.0);
                assert_eq!(height, 20.0);
            }
            _ => panic!("Expected CreateBox modal"),
        }
    }

    #[test]
    fn test_dispatch_sketch_new_and_finish() {
        let mut state = AppState::default();
        assert_eq!(state.active_tab_id, "model_3d");

        dispatch_ribbon_command("sketch.new", &mut state);
        assert_eq!(state.active_tab_id, "sketch");

        dispatch_ribbon_command("sketch.finish", &mut state);
        assert_eq!(state.active_tab_id, "model_3d");
    }

    #[test]
    fn test_dispatch_view_toggles() {
        let mut state = AppState::default();
        let initial_grid = state.show_grid;
        dispatch_ribbon_command("view.toggle_grid", &mut state);
        assert_eq!(state.show_grid, !initial_grid);

        let initial_axes = state.show_axes;
        dispatch_ribbon_command("view.toggle_axes", &mut state);
        assert_eq!(state.show_axes, !initial_axes);
    }

    #[test]
    fn test_dispatch_projection_switch() {
        let mut state = AppState::default();
        dispatch_ribbon_command("view.proj_ortho", &mut state);
        assert_eq!(state.camera.projection_type, ProjectionType::Orthographic);

        dispatch_ribbon_command("view.proj_perspective", &mut state);
        assert_eq!(state.camera.projection_type, ProjectionType::Perspective);
    }
}
