use crate::sketch_session::ActiveSketchSession;
use crate::state::{ActiveModal, AppState};
use egui::{Color32, RichText, ScrollArea, Ui};
use glam::Vec3;
use inventor_core::{FeatureKind, FeatureState, ReorderFeatureCmd, ToggleSuppressFeatureCmd};
use inventor_sketch::{ConstraintKind, ConstraintStatus};
use uuid::Uuid;

pub fn render_browser(ui: &mut Ui, state: &mut AppState) {
    ui.vertical(|ui| {
        ui.heading(RichText::new("Model Browser").size(14.0).strong());
        ui.separator();

        ScrollArea::vertical().show(ui, |ui| {
            // Root document Part node
            let part_label = format!("📦 {}", state.doc.title);
            ui.label(
                RichText::new(part_label)
                    .strong()
                    .color(Color32::from_rgb(56, 189, 248)),
            );

            // Origin Folder
            egui::CollapsingHeader::new(RichText::new("📁 Origin").strong())
                .default_open(true)
                .show(ui, |ui| {
                    let mut toggle_id = None;
                    let mut start_sketch_plane: Option<String> = None;

                    for feat in state.doc.features.iter().take(7) {
                        let is_origin_item = matches!(
                            feat.kind,
                            FeatureKind::OriginPlane { .. }
                                | FeatureKind::OriginAxis { .. }
                                | FeatureKind::OriginPoint
                        );

                        if is_origin_item {
                            let is_selected = state.selected_feature_id == Some(feat.id);
                            let eye_icon = if feat.visible { "👁" } else { "⊘" };

                            ui.horizontal(|ui| {
                                if ui
                                    .button(eye_icon)
                                    .on_hover_text("Toggle visibility")
                                    .clicked()
                                {
                                    toggle_id = Some(feat.id);
                                }

                                let item_text = if is_selected {
                                    RichText::new(&feat.name)
                                        .strong()
                                        .color(Color32::from_rgb(245, 158, 11))
                                } else {
                                    RichText::new(&feat.name)
                                        .color(Color32::from_rgb(226, 232, 240))
                                };

                                if ui.selectable_label(is_selected, item_text).clicked() {
                                    state.selected_feature_id = Some(feat.id);
                                    state.status_message =
                                        format!("Selected origin datum: {}", feat.name);
                                }

                                // Quick button to start sketch on this plane
                                if let FeatureKind::OriginPlane { name } = &feat.kind
                                    && state.active_sketch.is_none()
                                    && ui
                                        .small_button("✏")
                                        .on_hover_text(format!("Create 2D Sketch on {}", name))
                                        .clicked()
                                {
                                    start_sketch_plane = Some(name.clone());
                                }
                            });
                        }
                    }

                    if let Some(id) = toggle_id {
                        state.doc.toggle_visibility(id);
                    }

                    if let Some(plane_name) = start_sketch_plane {
                        state.enter_sketch_on_plane(&plane_name);
                    }
                });

            ui.separator();
            ui.label(RichText::new("Features History").small().weak());

            enum FeatureTreeAction {
                ToggleVisibility(Uuid),
                ToggleSuppress(Uuid),
                ReorderUp(Uuid),
                ReorderDown(Uuid),
                Select(Uuid, String),
                EditSketch(Uuid),
                EditExtrude {
                    feature_id: Uuid,
                    name: String,
                    distance: f64,
                    is_cut: bool,
                },
                EditRevolve {
                    feature_id: Uuid,
                    name: String,
                    angle_deg: f64,
                    is_cut: bool,
                },
                EditFillet {
                    feature_id: Uuid,
                    name: String,
                    radius: f64,
                },
                EditChamfer {
                    feature_id: Uuid,
                    name: String,
                    distance: f64,
                },
            }

            let mut actions = Vec::new();
            let total_features = state.doc.features.len();

            for (idx, feat) in state.doc.features.iter().enumerate().skip(7) {
                let is_selected = state.selected_feature_id == Some(feat.id);
                let eye = if feat.visible { "👁" } else { "⊘" };
                let is_sketch = matches!(feat.kind, FeatureKind::Sketch { .. });
                let is_suppressed = feat.state == FeatureState::Suppressed;
                let is_error = feat.state == FeatureState::Error;
                let can_move_up = idx > 7;
                let can_move_down = idx + 1 < total_features;

                ui.horizontal(|ui| {
                    if ui.button(eye).on_hover_text("Toggle visibility").clicked() {
                        actions.push(FeatureTreeAction::ToggleVisibility(feat.id));
                    }

                    if ui
                        .add_enabled(can_move_up, egui::Button::new("▲").small())
                        .on_hover_text("Move feature up in regeneration order")
                        .clicked()
                    {
                        actions.push(FeatureTreeAction::ReorderUp(feat.id));
                    }

                    if ui
                        .add_enabled(can_move_down, egui::Button::new("▼").small())
                        .on_hover_text("Move feature down in regeneration order")
                        .clicked()
                    {
                        actions.push(FeatureTreeAction::ReorderDown(feat.id));
                    }

                    let prefix = if is_sketch {
                        "📐 "
                    } else if matches!(feat.kind, FeatureKind::Revolve { .. }) {
                        "🔄 "
                    } else if matches!(feat.kind, FeatureKind::Boolean { .. }) {
                        "⧉ "
                    } else if matches!(feat.kind, FeatureKind::Fillet { .. }) {
                        "⌒ "
                    } else if matches!(feat.kind, FeatureKind::Chamfer { .. }) {
                        "◺ "
                    } else {
                        "🔹 "
                    };
                    let mut text = RichText::new(format!("{}{}", prefix, feat.name));
                    if is_suppressed {
                        text = text.strikethrough().color(Color32::from_rgb(148, 163, 184));
                    } else if is_error {
                        text = text.color(Color32::from_rgb(239, 68, 68));
                    } else if is_selected {
                        text = text.strong().color(Color32::from_rgb(245, 158, 11));
                    }

                    if ui.selectable_label(is_selected, text).clicked() {
                        actions.push(FeatureTreeAction::Select(feat.id, feat.name.clone()));
                    }

                    if is_error {
                        let err_tip = feat
                            .error_message
                            .as_deref()
                            .unwrap_or("Feature regeneration failed");
                        ui.label(
                            RichText::new("[⚠ Error]")
                                .small()
                                .strong()
                                .color(Color32::from_rgb(239, 68, 68)),
                        )
                        .on_hover_text(err_tip);
                    }

                    let supp_text = if is_suppressed {
                        "Unsuppress"
                    } else {
                        "Suppress"
                    };
                    if ui
                        .small_button(supp_text)
                        .on_hover_text(if is_suppressed {
                            "Unsuppress feature and restore solid geometry"
                        } else {
                            "Suppress feature without deleting it"
                        })
                        .clicked()
                    {
                        actions.push(FeatureTreeAction::ToggleSuppress(feat.id));
                    }

                    if is_sketch
                        && state.active_sketch.is_none()
                        && ui
                            .small_button("Edit")
                            .on_hover_text("Edit this 2D Sketch")
                            .clicked()
                    {
                        actions.push(FeatureTreeAction::EditSketch(feat.id));
                    } else if let FeatureKind::Extrude {
                        distance, is_cut, ..
                    } = feat.kind
                        && ui
                            .small_button("Edit")
                            .on_hover_text("Edit Extrude parameters")
                            .clicked()
                    {
                        actions.push(FeatureTreeAction::EditExtrude {
                            feature_id: feat.id,
                            name: feat.name.clone(),
                            distance,
                            is_cut,
                        });
                    } else if let FeatureKind::Revolve {
                        angle_rad, is_cut, ..
                    } = feat.kind
                        && ui
                            .small_button("Edit")
                            .on_hover_text("Edit Revolve parameters")
                            .clicked()
                    {
                        actions.push(FeatureTreeAction::EditRevolve {
                            feature_id: feat.id,
                            name: feat.name.clone(),
                            angle_deg: angle_rad.to_degrees(),
                            is_cut,
                        });
                    } else if let FeatureKind::Fillet { radius, .. } = feat.kind
                        && ui
                            .small_button("Edit")
                            .on_hover_text("Edit Fillet parameters")
                            .clicked()
                    {
                        actions.push(FeatureTreeAction::EditFillet {
                            feature_id: feat.id,
                            name: feat.name.clone(),
                            radius,
                        });
                    } else if let FeatureKind::Chamfer { distance, .. } = feat.kind
                        && ui
                            .small_button("Edit")
                            .on_hover_text("Edit Chamfer parameters")
                            .clicked()
                    {
                        actions.push(FeatureTreeAction::EditChamfer {
                            feature_id: feat.id,
                            name: feat.name.clone(),
                            distance,
                        });
                    }
                });
            }

            for action in actions {
                match action {
                    FeatureTreeAction::ToggleVisibility(id) => {
                        state.doc.toggle_visibility(id);
                        state.rebuild_solid();
                    }
                    FeatureTreeAction::ToggleSuppress(id) => {
                        state
                            .history
                            .execute(Box::new(ToggleSuppressFeatureCmd::new(id)), &mut state.doc);
                        state.rebuild_solid();
                    }
                    FeatureTreeAction::ReorderUp(id) => {
                        state
                            .history
                            .execute(Box::new(ReorderFeatureCmd::new(id, true)), &mut state.doc);
                        state.rebuild_solid();
                    }
                    FeatureTreeAction::ReorderDown(id) => {
                        state
                            .history
                            .execute(Box::new(ReorderFeatureCmd::new(id, false)), &mut state.doc);
                        state.rebuild_solid();
                    }
                    FeatureTreeAction::Select(id, name) => {
                        state.selected_feature_id = Some(id);
                        state.status_message = format!("Selected feature: {}", name);
                    }
                    FeatureTreeAction::EditSketch(sketch_id) => {
                        if let Some(stored) = state.sketches.get(&sketch_id).cloned() {
                            let plane_id = stored.sketch.plane_id;
                            let (plane_name, origin, x_axis, y_axis, normal) =
                                if plane_id == state.doc.xz_plane_id {
                                    state.camera.set_front_view();
                                    ("XZ Plane", Vec3::ZERO, Vec3::X, Vec3::Z, -Vec3::Y)
                                } else if plane_id == state.doc.yz_plane_id {
                                    state.camera.set_right_view();
                                    ("YZ Plane", Vec3::ZERO, Vec3::Y, Vec3::Z, Vec3::X)
                                } else {
                                    state.camera.set_top_view();
                                    ("XY Plane", Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z)
                                };

                            state.active_sketch = Some(ActiveSketchSession::from_stored(
                                stored, plane_name, origin, x_axis, y_axis, normal,
                            ));
                            state.active_tab_id = "sketch".to_string();
                            state.status_message = format!("Editing sketch on {}", plane_name);
                        }
                    }
                    FeatureTreeAction::EditExtrude {
                        feature_id,
                        name,
                        distance,
                        is_cut,
                    } => {
                        state.active_modal = ActiveModal::EditExtrude {
                            feature_id,
                            name,
                            distance,
                            is_cut,
                        };
                    }
                    FeatureTreeAction::EditRevolve {
                        feature_id,
                        name,
                        angle_deg,
                        is_cut,
                    } => {
                        state.active_modal = ActiveModal::EditRevolve {
                            feature_id,
                            name,
                            angle_deg,
                            is_cut,
                        };
                    }
                    FeatureTreeAction::EditFillet {
                        feature_id,
                        name,
                        radius,
                    } => {
                        state.active_modal = ActiveModal::EditFillet {
                            feature_id,
                            name,
                            radius,
                        };
                    }
                    FeatureTreeAction::EditChamfer {
                        feature_id,
                        name,
                        distance,
                    } => {
                        state.active_modal = ActiveModal::EditChamfer {
                            feature_id,
                            name,
                            distance,
                        };
                    }
                }
            }

            // 3. Active Sketch Inspection Panel (if currently in sketch mode)
            let mut to_edit_modal = None;
            let mut finish_clicked = false;

            if let Some(session) = state.active_sketch.as_mut() {
                ui.separator();
                ui.heading(
                    RichText::new(format!("✏ Active: {}", session.name))
                        .size(13.0)
                        .color(Color32::from_rgb(56, 189, 248)),
                );

                // DOF Status Badge
                match session.current_dof {
                    ConstraintStatus::FullyConstrained => {
                        ui.colored_label(
                            Color32::from_rgb(34, 197, 94),
                            "🟢 Fully Constrained (0 DOF)",
                        );
                    }
                    ConstraintStatus::Underconstrained(dof) => {
                        ui.colored_label(
                            Color32::from_rgb(56, 189, 248),
                            format!("🔵 Under-constrained ({} DOF)", dof),
                        );
                    }
                    ConstraintStatus::Overconstrained => {
                        ui.colored_label(
                            Color32::from_rgb(239, 68, 68),
                            "🔴 Over-constrained (Conflict)",
                        );
                    }
                }

                if let Some(err) = &session.solver_error {
                    ui.small(RichText::new(err).color(Color32::from_rgb(239, 68, 68)));
                }

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(RichText::new("✔ Finish Sketch").strong())
                        .clicked()
                    {
                        finish_clicked = true;
                    }
                });

                // Constraints list
                egui::CollapsingHeader::new(format!("Constraints ({})", session.constraints.len()))
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut to_edit = None;
                        let mut to_remove = None;

                        for c in &session.constraints {
                            ui.horizontal(|ui| {
                                let (label_text, is_dim, val) = match &c.kind {
                                    ConstraintKind::Distance { distance, .. } => {
                                        (format!("{}: {:.2} mm", c.name, distance), true, *distance)
                                    }
                                    ConstraintKind::Diameter { diameter, .. } => (
                                        format!("{}: ⌀{:.2} mm", c.name, diameter),
                                        true,
                                        *diameter,
                                    ),
                                    ConstraintKind::Radius { radius, .. } => {
                                        (format!("{}: R{:.2} mm", c.name, radius), true, *radius)
                                    }
                                    ConstraintKind::Angle { angle_deg, .. } => {
                                        (format!("{}: {:.1}°", c.name, angle_deg), true, *angle_deg)
                                    }
                                    ConstraintKind::Horizontal { .. } => {
                                        (format!("━ {}", c.name), false, 0.0)
                                    }
                                    ConstraintKind::Vertical { .. } => {
                                        (format!("↕ {}", c.name), false, 0.0)
                                    }
                                    ConstraintKind::Parallel { .. } => {
                                        (format!("∥ {}", c.name), false, 0.0)
                                    }
                                    ConstraintKind::Perpendicular { .. } => {
                                        (format!("⊥ {}", c.name), false, 0.0)
                                    }
                                    ConstraintKind::Coincident { .. } => {
                                        (format!("⨀ {}", c.name), false, 0.0)
                                    }
                                    ConstraintKind::Tangent { .. } => {
                                        (format!("⌒ {}", c.name), false, 0.0)
                                    }
                                    ConstraintKind::Equal { .. } => {
                                        (format!("= {}", c.name), false, 0.0)
                                    }
                                    ConstraintKind::FixPoint { .. } => {
                                        (format!("🔒 {}", c.name), false, 0.0)
                                    }
                                };

                                ui.label(RichText::new(label_text).small());

                                if is_dim
                                    && ui
                                        .small_button("✎")
                                        .on_hover_text("Modify dimension")
                                        .clicked()
                                {
                                    to_edit = Some((c.id, c.name.clone(), val));
                                }

                                if ui
                                    .small_button("✕")
                                    .on_hover_text("Delete constraint")
                                    .clicked()
                                {
                                    to_remove = Some(c.id);
                                }
                            });
                        }

                        if let Some((cid, name, val)) = to_edit {
                            to_edit_modal = Some((cid, name, val));
                        }

                        if let Some(cid) = to_remove {
                            session.remove_constraint(cid);
                        }
                    });
            }

            if let Some((cid, name, val)) = to_edit_modal {
                state.active_modal = ActiveModal::EditDimensionInput {
                    constraint_id: cid,
                    name,
                    current_value: val,
                };
            }

            if finish_clicked {
                let _ = state.finish_active_sketch();
            }
        });
    });
}
