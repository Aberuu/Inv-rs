use crate::state::{ActiveModal, AppState};
use egui::{Color32, Context, RichText, Window};
use inventor_core::{
    AddFeatureCmd, BooleanOp, Feature, FeatureKind, ModifyFeatureCmd, Parameter, Unit,
};
use inventor_io::{StepExporter, StlExporter};
use inventor_kernel::{GeometryKernel, KernelBackend};

pub fn render_dialogs(ctx: &Context, state: &mut AppState) {
    let mut close_modal = false;

    match &mut state.active_modal {
        ActiveModal::CreateBox {
            width,
            length,
            height,
        } => {
            Window::new("Create Box Solid")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Specify dimensions for the 3D box primitive:");
                    ui.add_space(8.0);

                    egui::Grid::new("box_inputs")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Length (X) [mm]:");
                            ui.add(egui::DragValue::new(length).range(1.0..=5000.0).speed(1.0));
                            ui.end_row();

                            ui.label("Width (Y) [mm]:");
                            ui.add(egui::DragValue::new(width).range(1.0..=5000.0).speed(1.0));
                            ui.end_row();

                            ui.label("Height (Z) [mm]:");
                            ui.add(egui::DragValue::new(height).range(1.0..=5000.0).speed(1.0));
                            ui.end_row();
                        });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Create Solid").strong()).clicked() {
                            match state.kernel.create_box(*length, *width, *height) {
                                Ok(mesh) => {
                                    state.active_mesh = Some(mesh);
                                    let feat_num = state.doc.features.len() - 6;
                                    let feat_name = format!("Box{}", feat_num);
                                    let feat = Feature::new(
                                        feat_name.clone(),
                                        FeatureKind::Extrude {
                                            name: feat_name,
                                            sketch_id: state.doc.xy_plane_id,
                                            distance: *height,
                                            is_cut: false,
                                        },
                                    );
                                    state.history.execute(
                                        Box::new(AddFeatureCmd::new(feat)),
                                        &mut state.doc,
                                    );
                                    state.status_message = format!(
                                        "Created box solid: {:.1} x {:.1} x {:.1} mm.",
                                        *length, *width, *height
                                    );
                                    close_modal = true;
                                }
                                Err(err) => {
                                    state.status_message =
                                        format!("Kernel error creating box: {}", err);
                                }
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }
                    });
                });
        }
        ActiveModal::CreateCylinder { radius, height } => {
            Window::new("Create Cylinder Solid")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Specify dimensions for the 3D cylinder primitive:");
                    ui.add_space(8.0);

                    egui::Grid::new("cyl_inputs")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Radius [mm]:");
                            ui.add(egui::DragValue::new(radius).range(1.0..=2000.0).speed(0.5));
                            ui.end_row();

                            ui.label("Height [mm]:");
                            ui.add(egui::DragValue::new(height).range(1.0..=5000.0).speed(1.0));
                            ui.end_row();
                        });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Create Solid").strong()).clicked() {
                            match state.kernel.create_cylinder(*radius, *height) {
                                Ok(mesh) => {
                                    state.active_mesh = Some(mesh);
                                    let feat_num = state.doc.features.len() - 6;
                                    let feat_name = format!("Cylinder{}", feat_num);
                                    let feat = Feature::new(
                                        feat_name.clone(),
                                        FeatureKind::Revolve {
                                            name: feat_name,
                                            sketch_id: state.doc.xy_plane_id,
                                            angle_rad: std::f64::consts::TAU,
                                            is_cut: false,
                                        },
                                    );
                                    state.history.execute(
                                        Box::new(AddFeatureCmd::new(feat)),
                                        &mut state.doc,
                                    );
                                    state.status_message = format!(
                                        "Created cylinder solid: Radius {:.1} mm, Height {:.1} mm.",
                                        *radius, *height
                                    );
                                    close_modal = true;
                                }
                                Err(err) => {
                                    state.status_message =
                                        format!("Kernel error creating cylinder: {}", err);
                                }
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }
                    });
                });
        }
        ActiveModal::Parameters => {
            Window::new("Parameters (fx)")
                .default_size([520.0, 360.0])
                .show(ctx, |ui| {
                    ui.label(RichText::new("Document Parameters Table").strong());
                    ui.separator();

                    let params = state.doc.parameters.list();
                    egui::Grid::new("param_table")
                        .striped(true)
                        .num_columns(4)
                        .spacing([16.0, 8.0])
                        .show(ui, |ui| {
                            ui.label(RichText::new("Parameter Name").strong());
                            ui.label(RichText::new("Expression").strong());
                            ui.label(RichText::new("Value").strong());
                            ui.label(RichText::new("Unit").strong());
                            ui.end_row();

                            for p in params {
                                ui.label(&p.name);
                                ui.label(&p.expression);
                                ui.label(format!("{:.4}", p.value));
                                ui.label(format!("{:?}", p.unit));
                                ui.end_row();
                            }
                        });

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Done").clicked() {
                            close_modal = true;
                        }
                    });
                });
        }
        ActiveModal::ExportStep => {
            Window::new("Export STEP (ISO 10303)")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(
                        "Export current 3D parametric solid to standard STEP AP203/214 format.",
                    );
                    ui.add_space(8.0);

                    if state.active_mesh.is_some() {
                        let l = state.doc.parameters.get_value("length").unwrap_or(50.0);
                        let w = state.doc.parameters.get_value("width").unwrap_or(30.0);
                        let h = state.doc.parameters.get_value("height").unwrap_or(20.0);

                        ui.label(format!(
                            "Solid geometry bounds: {:.1} x {:.1} x {:.1} mm",
                            l, w, h
                        ));
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            if ui
                                .button(RichText::new("Export STEP File").strong())
                                .clicked()
                            {
                                let solid = state.kernel.build_box_solid(l, w, h);
                                let out_path = "export_model.step";
                                if let Ok(mut file) = std::fs::File::create(out_path)
                                    && StepExporter::export_solid_step(&solid, &mut file).is_ok()
                                {
                                    state.status_message = format!(
                                        "Successfully exported ISO 10303 STEP solid to {}",
                                        out_path
                                    );
                                    state.last_exported_path = Some(out_path.into());
                                    close_modal = true;
                                }
                            }

                            if ui.button("Cancel").clicked() {
                                close_modal = true;
                            }
                        });
                    } else {
                        ui.colored_label(
                            Color32::from_rgb(239, 68, 68),
                            "No solid model present in active viewport to export.",
                        );
                        if ui.button("Close").clicked() {
                            close_modal = true;
                        }
                    }
                });
        }
        ActiveModal::ExportStl => {
            Window::new("Export STL Mesh")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Export current 3D solid as STL format for 3D printing.");
                    ui.add_space(8.0);

                    if let Some(mesh) = &state.active_mesh {
                        ui.label(format!("Triangles: {} facets", mesh.triangle_count()));
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            if ui
                                .button(RichText::new("Export Binary STL").strong())
                                .clicked()
                            {
                                let out_path = "export_model.stl";
                                if let Ok(mut file) = std::fs::File::create(out_path)
                                    && StlExporter::export_binary(mesh, &mut file).is_ok()
                                {
                                    state.status_message =
                                        format!("Successfully exported binary STL to {}", out_path);
                                    state.last_exported_path = Some(out_path.into());
                                    close_modal = true;
                                }
                            }

                            if ui.button("Export ASCII STL").clicked() {
                                let out_path = "export_model_ascii.stl";
                                if let Ok(mut file) = std::fs::File::create(out_path)
                                    && StlExporter::export_ascii(mesh, "Part1", &mut file).is_ok()
                                {
                                    state.status_message =
                                        format!("Successfully exported ASCII STL to {}", out_path);
                                    state.last_exported_path = Some(out_path.into());
                                    close_modal = true;
                                }
                            }

                            if ui.button("Cancel").clicked() {
                                close_modal = true;
                            }
                        });
                    } else {
                        ui.colored_label(
                            Color32::from_rgb(239, 68, 68),
                            "No solid mesh available in active viewport to export.",
                        );
                        if ui.button("Close").clicked() {
                            close_modal = true;
                        }
                    }
                });
        }
        ActiveModal::DimensionInput {
            title,
            is_circle,
            is_angle,
            current_value,
        } => {
            let title_clone = title.clone();
            let is_cir = *is_circle;
            let is_ang = *is_angle;
            let mut apply_dim = false;
            Window::new(format!("Edit Dimension: {}", title_clone))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Enter dimension parameter value:");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(if is_ang {
                            "Angle [deg]:"
                        } else {
                            "Value [mm]:"
                        });
                        ui.add(
                            egui::DragValue::new(current_value)
                                .speed(1.0)
                                .range(0.01..=10000.0),
                        );
                    });
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Apply (Enter)").strong()).clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            apply_dim = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if apply_dim {
                let val = *current_value;
                if let Some(session) = state.active_sketch.as_mut() {
                    if is_cir && session.selected_circles.len() == 1 {
                        let c_id = session.selected_circles[0];
                        let c_name = format!("d{}", session.constraints.len() + 1);
                        let _ = session.add_constraint(inventor_sketch::Constraint::new(
                            c_name,
                            inventor_sketch::ConstraintKind::Diameter {
                                circle: c_id,
                                diameter: val,
                            },
                        ));
                    } else if is_ang && session.selected_lines.len() >= 2 {
                        let l1 = session.selected_lines[0];
                        let l2 = session.selected_lines[1];
                        let c_name = format!("d{}", session.constraints.len() + 1);
                        let _ = session.add_constraint(inventor_sketch::Constraint::new(
                            c_name,
                            inventor_sketch::ConstraintKind::Angle {
                                line1: l1,
                                line2: l2,
                                angle_deg: val,
                            },
                        ));
                    } else if session.selected_lines.len() == 1 {
                        let l_id = session.selected_lines[0];
                        if let Some(l) = session.sketch.find_line(l_id) {
                            let c_name = format!("d{}", session.constraints.len() + 1);
                            let _ = session.add_constraint(inventor_sketch::Constraint::new(
                                c_name,
                                inventor_sketch::ConstraintKind::Distance {
                                    point1: l.start,
                                    point2: l.end,
                                    distance: val,
                                },
                            ));
                        }
                    } else if session.selected_points.len() == 2 {
                        let p1 = session.selected_points[0];
                        let p2 = session.selected_points[1];
                        let c_name = format!("d{}", session.constraints.len() + 1);
                        let _ = session.add_constraint(inventor_sketch::Constraint::new(
                            c_name,
                            inventor_sketch::ConstraintKind::Distance {
                                point1: p1,
                                point2: p2,
                                distance: val,
                            },
                        ));
                    }
                }
                close_modal = true;
            }
        }
        ActiveModal::EditDimensionInput {
            constraint_id,
            name,
            current_value,
        } => {
            let cid = *constraint_id;
            let name_clone = name.clone();
            let mut apply_edit = false;
            Window::new(format!("Modify Dimension: {}", name_clone))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Enter new target value:");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("Value:");
                        ui.add(
                            egui::DragValue::new(current_value)
                                .speed(1.0)
                                .range(0.01..=10000.0),
                        );
                    });
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button(RichText::new("Update (Enter)").strong())
                            .clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            apply_edit = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if apply_edit {
                let val = *current_value;
                if let Some(session) = state.active_sketch.as_mut() {
                    if let Some(c) = session.constraints.iter_mut().find(|c| c.id == cid) {
                        match &mut c.kind {
                            inventor_sketch::ConstraintKind::Distance { distance, .. } => {
                                *distance = val;
                            }
                            inventor_sketch::ConstraintKind::Diameter { diameter, .. } => {
                                *diameter = val;
                            }
                            inventor_sketch::ConstraintKind::Radius { radius, .. } => {
                                *radius = val;
                            }
                            inventor_sketch::ConstraintKind::Angle { angle_deg, .. } => {
                                *angle_deg = val;
                            }
                            _ => {}
                        }
                    }
                    let _ = session.solve_constraints();
                }
                close_modal = true;
            }
        }
        ActiveModal::CreateExtrude {
            sketch_id,
            distance,
            is_cut,
        } => {
            let mut apply_extrude = false;
            let target_sid = *sketch_id;
            let dist_val = *distance;
            let cut_val = *is_cut;

            Window::new("Extrude Feature")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Create 3D solid feature by extruding a 2D sketch profile:");
                    ui.add_space(8.0);

                    let sketch_name = target_sid
                        .and_then(|id| state.sketches.get(&id))
                        .map(|s| s.sketch.name.clone())
                        .unwrap_or_else(|| "None".into());

                    egui::Grid::new("extrude_inputs")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Profile Sketch:");
                            ui.label(
                                RichText::new(&sketch_name)
                                    .strong()
                                    .color(Color32::from_rgb(56, 189, 248)),
                            );
                            ui.end_row();

                            ui.label("Distance (d) [mm]:");
                            ui.add(
                                egui::DragValue::new(distance)
                                    .range(0.1..=5000.0)
                                    .speed(1.0),
                            );
                            ui.end_row();

                            ui.label("Operation:");
                            ui.horizontal(|ui| {
                                ui.selectable_value(is_cut, false, "Join / Solid");
                                ui.selectable_value(is_cut, true, "Cut / Subtract");
                            });
                            ui.end_row();
                        });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Extrude (OK)").strong()).clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            apply_extrude = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if apply_extrude {
                if let Some(sid) = target_sid {
                    let next_extrude_num = state
                        .doc
                        .features
                        .iter()
                        .filter(|f| matches!(f.kind, FeatureKind::Extrude { .. }))
                        .count()
                        + 1;
                    let feat_name = format!("Extrude{}", next_extrude_num);
                    let feat = Feature::new(
                        feat_name.clone(),
                        FeatureKind::Extrude {
                            name: feat_name.clone(),
                            sketch_id: sid,
                            distance: dist_val,
                            is_cut: cut_val,
                        },
                    );
                    state
                        .history
                        .execute(Box::new(AddFeatureCmd::new(feat)), &mut state.doc);
                    state.rebuild_solid();
                    state.status_message =
                        format!("Created Extrude feature '{}' ({} mm).", feat_name, dist_val);
                }
                close_modal = true;
            }
        }
        ActiveModal::EditExtrude {
            feature_id,
            name,
            distance,
            is_cut,
        } => {
            let fid = *feature_id;
            let feat_name = name.clone();
            let dist_val = *distance;
            let cut_val = *is_cut;
            let mut apply_edit = false;

            Window::new(format!("Edit Feature: {}", feat_name))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Modify parameters for extrusion '{}':", feat_name));
                    ui.add_space(8.0);

                    egui::Grid::new("edit_extrude_inputs")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Distance (d) [mm]:");
                            ui.add(
                                egui::DragValue::new(distance)
                                    .range(0.1..=5000.0)
                                    .speed(1.0),
                            );
                            ui.end_row();

                            ui.label("Operation:");
                            ui.horizontal(|ui| {
                                ui.selectable_value(is_cut, false, "Join / Solid");
                                ui.selectable_value(is_cut, true, "Cut / Subtract");
                            });
                            ui.end_row();
                        });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Apply (OK)").strong()).clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            apply_edit = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if apply_edit {
                let existing_sketch_id = state.doc.find_feature(fid).and_then(|f| {
                    if let FeatureKind::Extrude { sketch_id, .. } = f.kind {
                        Some(sketch_id)
                    } else {
                        None
                    }
                });

                if let Some(sid) = existing_sketch_id {
                    let cmd = ModifyFeatureCmd::new(
                        fid,
                        FeatureKind::Extrude {
                            name: feat_name.clone(),
                            sketch_id: sid,
                            distance: dist_val,
                            is_cut: cut_val,
                        },
                    );
                    state.history.execute(Box::new(cmd), &mut state.doc);
                    if state.doc.parameters.contains(&feat_name) {
                        state.doc.parameters.set(Parameter::new(
                            &feat_name,
                            dist_val,
                            Unit::Millimeter,
                        ));
                    }
                    if (feat_name == "Extrude1" || feat_name == "Box1")
                        && state.doc.parameters.contains("height")
                    {
                        state.doc.parameters.set(Parameter::new(
                            "height",
                            dist_val,
                            Unit::Millimeter,
                        ));
                    }
                    state.rebuild_solid();
                    state.status_message = format!(
                        "Updated Extrude feature '{}' (distance: {:.1} mm).",
                        feat_name, dist_val
                    );
                }
                close_modal = true;
            }
        }
        ActiveModal::CreateRevolve {
            sketch_id,
            angle_deg,
            is_cut,
        } => {
            let mut apply_revolve = false;
            let target_sid = *sketch_id;
            let angle_val = *angle_deg;
            let cut_val = *is_cut;

            Window::new("Revolve Feature")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(
                        "Create 3D solid feature by revolving a 2D sketch profile about an axis:",
                    );
                    ui.add_space(8.0);

                    let sketch_name = target_sid
                        .and_then(|id| state.sketches.get(&id))
                        .map(|s| s.sketch.name.clone())
                        .unwrap_or_else(|| "None".into());

                    egui::Grid::new("revolve_inputs")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Profile Sketch:");
                            ui.label(
                                RichText::new(&sketch_name)
                                    .strong()
                                    .color(Color32::from_rgb(56, 189, 248)),
                            );
                            ui.end_row();

                            ui.label("Angle [deg]:");
                            ui.add(
                                egui::DragValue::new(angle_deg)
                                    .range(1.0..=360.0)
                                    .speed(1.0)
                                    .suffix("°"),
                            );
                            ui.end_row();

                            ui.label("Operation:");
                            ui.horizontal(|ui| {
                                ui.selectable_value(is_cut, false, "Join / Solid");
                                ui.selectable_value(is_cut, true, "Cut / Subtract");
                            });
                            ui.end_row();
                        });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Revolve (OK)").strong()).clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            apply_revolve = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if apply_revolve {
                if let Some(sid) = target_sid {
                    let next_revolve_num = state
                        .doc
                        .features
                        .iter()
                        .filter(|f| matches!(f.kind, FeatureKind::Revolve { .. }))
                        .count()
                        + 1;
                    let feat_name = format!("Revolve{}", next_revolve_num);
                    let angle_rad = angle_val.to_radians();
                    let feat = Feature::new(
                        feat_name.clone(),
                        FeatureKind::Revolve {
                            name: feat_name.clone(),
                            sketch_id: sid,
                            angle_rad,
                            is_cut: cut_val,
                        },
                    );
                    state
                        .history
                        .execute(Box::new(AddFeatureCmd::new(feat)), &mut state.doc);
                    state.rebuild_solid();
                    state.status_message = format!(
                        "Created Revolve feature '{}' ({:.1}°).",
                        feat_name, angle_val
                    );
                }
                close_modal = true;
            }
        }
        ActiveModal::EditRevolve {
            feature_id,
            name,
            angle_deg,
            is_cut,
        } => {
            let fid = *feature_id;
            let feat_name = name.clone();
            let angle_val = *angle_deg;
            let cut_val = *is_cut;
            let mut apply_edit = false;

            Window::new(format!("Edit Feature: {}", feat_name))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Modify parameters for revolution '{}':", feat_name));
                    ui.add_space(8.0);

                    egui::Grid::new("edit_revolve_inputs")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Angle [deg]:");
                            ui.add(
                                egui::DragValue::new(angle_deg)
                                    .range(1.0..=360.0)
                                    .speed(1.0)
                                    .suffix("°"),
                            );
                            ui.end_row();

                            ui.label("Operation:");
                            ui.horizontal(|ui| {
                                ui.selectable_value(is_cut, false, "Join / Solid");
                                ui.selectable_value(is_cut, true, "Cut / Subtract");
                            });
                            ui.end_row();
                        });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Apply (OK)").strong()).clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            apply_edit = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if apply_edit {
                let existing_sketch_id = state.doc.find_feature(fid).and_then(|f| {
                    if let FeatureKind::Revolve { sketch_id, .. } = f.kind {
                        Some(sketch_id)
                    } else {
                        None
                    }
                });

                if let Some(sid) = existing_sketch_id {
                    let angle_rad = angle_val.to_radians();
                    let cmd = ModifyFeatureCmd::new(
                        fid,
                        FeatureKind::Revolve {
                            name: feat_name.clone(),
                            sketch_id: sid,
                            angle_rad,
                            is_cut: cut_val,
                        },
                    );
                    state.history.execute(Box::new(cmd), &mut state.doc);
                    state.rebuild_solid();
                    state.status_message = format!(
                        "Updated Revolve feature '{}' (angle: {:.1}°).",
                        feat_name, angle_val
                    );
                }
                close_modal = true;
            }
        }
        ActiveModal::BooleanCombine {
            target_feature,
            tool_feature,
            op,
        } => {
            let mut apply_combine = false;
            let target_val = *target_feature;
            let tool_val = *tool_feature;
            let op_val = *op;

            let solid_features: Vec<(uuid::Uuid, String)> = state
                .doc
                .features
                .iter()
                .filter(|f| {
                    matches!(
                        f.kind,
                        FeatureKind::Extrude { .. } | FeatureKind::Revolve { .. }
                    )
                })
                .map(|f| (f.id, f.name.clone()))
                .collect();

            Window::new("Combine Solid Features")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Perform a boolean operation between solid features:");
                    ui.add_space(8.0);

                    egui::Grid::new("combine_inputs")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Target Body:");
                            let target_name = target_feature
                                .and_then(|id| {
                                    solid_features
                                        .iter()
                                        .find(|(fid, _)| *fid == id)
                                        .map(|(_, n)| n.as_str())
                                })
                                .unwrap_or("Select Target Body");
                            egui::ComboBox::from_id_salt("target_body_combo")
                                .selected_text(target_name)
                                .show_ui(ui, |ui| {
                                    for (id, name) in &solid_features {
                                        ui.selectable_value(target_feature, Some(*id), name);
                                    }
                                });
                            ui.end_row();

                            ui.label("Tool Body:");
                            let tool_name = tool_feature
                                .and_then(|id| {
                                    solid_features
                                        .iter()
                                        .find(|(fid, _)| *fid == id)
                                        .map(|(_, n)| n.as_str())
                                })
                                .unwrap_or("Select Tool Body");
                            egui::ComboBox::from_id_salt("tool_body_combo")
                                .selected_text(tool_name)
                                .show_ui(ui, |ui| {
                                    for (id, name) in &solid_features {
                                        ui.selectable_value(tool_feature, Some(*id), name);
                                    }
                                });
                            ui.end_row();

                            ui.label("Operation:");
                            ui.horizontal(|ui| {
                                ui.selectable_value(op, BooleanOp::Union, "Union (Join)");
                                ui.selectable_value(op, BooleanOp::Subtract, "Subtract (Cut)");
                                ui.selectable_value(op, BooleanOp::Intersect, "Intersect");
                            });
                            ui.end_row();
                        });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        let can_apply = target_feature.is_some()
                            && tool_feature.is_some()
                            && target_feature != tool_feature;
                        if ui
                            .add_enabled(
                                can_apply,
                                egui::Button::new(RichText::new("Combine (OK)").strong()),
                            )
                            .clicked()
                            || (can_apply && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        {
                            apply_combine = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if apply_combine {
                if let (Some(target_id), Some(tool_id)) = (target_val, tool_val) {
                    let next_combine_num = state
                        .doc
                        .features
                        .iter()
                        .filter(|f| matches!(f.kind, FeatureKind::Boolean { .. }))
                        .count()
                        + 1;
                    let feat_name = format!("Combine{}", next_combine_num);
                    let feat = Feature::new(
                        feat_name.clone(),
                        FeatureKind::Boolean {
                            name: feat_name.clone(),
                            op: op_val,
                            target_id,
                            tool_id,
                        },
                    );
                    state
                        .history
                        .execute(Box::new(AddFeatureCmd::new(feat)), &mut state.doc);
                    state.rebuild_solid();
                    state.status_message =
                        format!("Created Combine feature '{}' ({:?}).", feat_name, op_val);
                }
                close_modal = true;
            }
        }
        ActiveModal::CreateFillet { radius } => {
            let mut apply_fillet = false;
            let rad_val = *radius;

            Window::new("Create Fillet")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Apply circular fillet blend to solid edges:");
                    ui.add_space(8.0);

                    egui::Grid::new("create_fillet_inputs")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Radius (r) [mm]:");
                            ui.add(egui::DragValue::new(radius).range(0.1..=500.0).speed(0.5));
                            ui.end_row();
                        });

                    ui.add_space(8.0);
                    if !state.kernel.supports_brep_fillet() {
                        ui.colored_label(
                            Color32::from_rgb(245, 158, 11),
                            "Note: Active kernel (truck) lacks 3D B-rep edge blending.\nFeature will be recorded with diagnostic status (evaluated in T8).",
                        );
                    }

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Apply (OK)").strong()).clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            apply_fillet = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if apply_fillet {
                let next_fillet_num = state
                    .doc
                    .features
                    .iter()
                    .filter(|f| matches!(f.kind, FeatureKind::Fillet { .. }))
                    .count()
                    + 1;
                let feat_name = format!("Fillet{}", next_fillet_num);
                let feat = Feature::new(
                    feat_name.clone(),
                    FeatureKind::Fillet {
                        name: feat_name.clone(),
                        radius: rad_val,
                    },
                );
                state
                    .history
                    .execute(Box::new(AddFeatureCmd::new(feat)), &mut state.doc);
                state.rebuild_solid();
                state.status_message = format!(
                    "Created Fillet feature '{}' (radius: {:.1} mm).",
                    feat_name, rad_val
                );
                close_modal = true;
            }
        }
        ActiveModal::EditFillet {
            feature_id,
            name,
            radius,
        } => {
            let fid = *feature_id;
            let feat_name = name.clone();
            let rad_val = *radius;
            let mut apply_edit = false;

            Window::new(format!("Edit Feature: {}", feat_name))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Modify parameters for fillet '{}':", feat_name));
                    ui.add_space(8.0);

                    egui::Grid::new("edit_fillet_inputs")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Radius (r) [mm]:");
                            ui.add(egui::DragValue::new(radius).range(0.1..=500.0).speed(0.5));
                            ui.end_row();
                        });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Apply (OK)").strong()).clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            apply_edit = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if apply_edit {
                let cmd = ModifyFeatureCmd::new(
                    fid,
                    FeatureKind::Fillet {
                        name: feat_name.clone(),
                        radius: rad_val,
                    },
                );
                state.history.execute(Box::new(cmd), &mut state.doc);
                state.rebuild_solid();
                state.status_message = format!(
                    "Updated Fillet feature '{}' (radius: {:.1} mm).",
                    feat_name, rad_val
                );
                close_modal = true;
            }
        }
        ActiveModal::CreateChamfer { distance } => {
            let mut apply_chamfer = false;
            let dist_val = *distance;

            Window::new("Create Chamfer")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Apply beveled chamfer to solid edges:");
                    ui.add_space(8.0);

                    egui::Grid::new("create_chamfer_inputs")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Distance (d) [mm]:");
                            ui.add(
                                egui::DragValue::new(distance)
                                    .range(0.1..=500.0)
                                    .speed(0.5),
                            );
                            ui.end_row();
                        });

                    ui.add_space(8.0);
                    if !state.kernel.supports_brep_fillet() {
                        ui.colored_label(
                            Color32::from_rgb(245, 158, 11),
                            "Note: Active kernel (truck) lacks 3D B-rep edge chamfering.\nFeature will be recorded with diagnostic status (evaluated in T8).",
                        );
                    }

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Apply (OK)").strong()).clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            apply_chamfer = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if apply_chamfer {
                let next_chamfer_num = state
                    .doc
                    .features
                    .iter()
                    .filter(|f| matches!(f.kind, FeatureKind::Chamfer { .. }))
                    .count()
                    + 1;
                let feat_name = format!("Chamfer{}", next_chamfer_num);
                let feat = Feature::new(
                    feat_name.clone(),
                    FeatureKind::Chamfer {
                        name: feat_name.clone(),
                        distance: dist_val,
                    },
                );
                state
                    .history
                    .execute(Box::new(AddFeatureCmd::new(feat)), &mut state.doc);
                state.rebuild_solid();
                state.status_message = format!(
                    "Created Chamfer feature '{}' (distance: {:.1} mm).",
                    feat_name, dist_val
                );
                close_modal = true;
            }
        }
        ActiveModal::EditChamfer {
            feature_id,
            name,
            distance,
        } => {
            let fid = *feature_id;
            let feat_name = name.clone();
            let dist_val = *distance;
            let mut apply_edit = false;

            Window::new(format!("Edit Feature: {}", feat_name))
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label(format!("Modify parameters for chamfer '{}':", feat_name));
                    ui.add_space(8.0);

                    egui::Grid::new("edit_chamfer_inputs")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Distance (d) [mm]:");
                            ui.add(egui::DragValue::new(distance).range(0.1..=500.0).speed(0.5));
                            ui.end_row();
                        });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("Apply (OK)").strong()).clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            apply_edit = true;
                        }
                        if ui.button("Cancel").clicked() {
                            close_modal = true;
                        }
                    });
                });

            if apply_edit {
                let cmd = ModifyFeatureCmd::new(
                    fid,
                    FeatureKind::Chamfer {
                        name: feat_name.clone(),
                        distance: dist_val,
                    },
                );
                state.history.execute(Box::new(cmd), &mut state.doc);
                state.rebuild_solid();
                state.status_message = format!(
                    "Updated Chamfer feature '{}' (distance: {:.1} mm).",
                    feat_name, dist_val
                );
                close_modal = true;
            }
        }
        ActiveModal::Options => {
            Window::new("Application & Kernel Options")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Geometric Kernel Backend");
                    ui.label("Select active B-rep geometry kernel (PRD Section 12 & Task T8):");
                    ui.add_space(6.0);

                    let current_backend = state.kernel.backend;
                    if ui
                        .selectable_label(
                            current_backend == KernelBackend::Truck,
                            "truck (Pure Rust B-rep)",
                        )
                        .clicked()
                    {
                        state.kernel.backend = KernelBackend::Truck;
                        state.rebuild_solid();
                        state.status_message = "Switched to Pure-Rust truck kernel backend.".into();
                    }
                    if ui
                        .selectable_label(
                            current_backend == KernelBackend::OpenCascade,
                            "OpenCASCADE (Secondary Backend via FFI)",
                        )
                        .clicked()
                    {
                        state.kernel.backend = KernelBackend::OpenCascade;
                        state.rebuild_solid();
                        state.status_message =
                            "Switched to OpenCASCADE secondary kernel backend.".into();
                    }

                    ui.add_space(10.0);
                    ui.label(RichText::new("Kernel Capabilities:").strong());
                    ui.label(format!("• Active Backend: {}", state.kernel.kernel_name()));
                    ui.label(format!(
                        "• 3D B-rep Fillet / Chamfer: {}",
                        if state.kernel.supports_brep_fillet() {
                            "Supported (OCCT)"
                        } else {
                            "Unsupported in pure-Rust truck (evaluated in T8)"
                        }
                    ));

                    ui.add_space(12.0);
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
        }
        ActiveModal::About => {
            Window::new("About inventor-rs")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading(RichText::new("inventor-rs").color(Color32::from_rgb(56, 189, 248)));
                    ui.label("Clean-room Parametric 3D Solid Modeler in Rust.");
                    ui.label("Geometric Kernel: truck (Pure Rust B-rep)");
                    ui.label("UI & Render: egui + wgpu");
                    ui.add_space(8.0);
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
        }
        _ => {}
    }

    if close_modal {
        state.active_modal = ActiveModal::None;
    }
}
