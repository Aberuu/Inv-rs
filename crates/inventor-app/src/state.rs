use crate::commands::CommandRegistry;
use crate::sketch_session::{ActiveSketchSession, StoredSketch};
use crate::ui::ribbon_schema::RibbonSchema;
use glam::Vec3;
use inventor_core::{BooleanOp, Document, Feature, FeatureKind, HistoryStack, Parameter, Unit};
use inventor_kernel::{CadMesh, DualKernel, GeometryKernel, SolidShape};
use inventor_render::CadCamera;
use inventor_sketch::{Constraint, ConstraintKind, ConstraintSolver, Sketch2D};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum ActiveModal {
    None,
    CreateBox {
        width: f64,
        length: f64,
        height: f64,
    },
    CreateCylinder {
        radius: f64,
        height: f64,
    },
    Parameters,
    ExportStep,
    ExportStl,
    About,
    Options,
    DimensionInput {
        title: String,
        is_circle: bool,
        is_angle: bool,
        current_value: f64,
    },
    EditDimensionInput {
        constraint_id: Uuid,
        name: String,
        current_value: f64,
    },
    CreateExtrude {
        sketch_id: Option<Uuid>,
        distance: f64,
        is_cut: bool,
    },
    EditExtrude {
        feature_id: Uuid,
        name: String,
        distance: f64,
        is_cut: bool,
    },
    CreateRevolve {
        sketch_id: Option<Uuid>,
        angle_deg: f64,
        is_cut: bool,
    },
    EditRevolve {
        feature_id: Uuid,
        name: String,
        angle_deg: f64,
        is_cut: bool,
    },
    BooleanCombine {
        target_feature: Option<Uuid>,
        tool_feature: Option<Uuid>,
        op: BooleanOp,
    },
    CreateFillet {
        radius: f64,
    },
    EditFillet {
        feature_id: Uuid,
        name: String,
        radius: f64,
    },
    CreateChamfer {
        distance: f64,
    },
    EditChamfer {
        feature_id: Uuid,
        name: String,
        distance: f64,
    },
}

pub struct AppState {
    pub doc: Document,
    pub history: HistoryStack,
    pub camera: CadCamera,
    pub kernel: DualKernel,
    pub active_mesh: Option<CadMesh>,
    pub active_tab_id: String,
    pub ribbon_schema: RibbonSchema,
    pub command_registry: Arc<CommandRegistry>,
    pub command_search_query: String,
    pub active_modal: ActiveModal,
    pub selected_feature_id: Option<Uuid>,
    pub status_message: String,
    pub show_grid: bool,
    pub show_axes: bool,
    pub last_exported_path: Option<String>,
    pub active_sketch: Option<ActiveSketchSession>,
    pub sketches: HashMap<Uuid, StoredSketch>,
    pub wgpu_renderer: Option<Arc<Mutex<inventor_render::WgpuCadRenderer>>>,
    pub wgpu_render_state: Option<egui_wgpu::RenderState>,
    pub viewport_texture_id: Option<egui::TextureId>,
    pub viewport_texture_size: (u32, u32),
}

impl Default for AppState {
    fn default() -> Self {
        let mut kernel = DualKernel::new();
        kernel.truck.tolerance = 0.05;

        let mut doc = Document::new("Part1");

        // Add initial parameters
        doc.parameters
            .set(Parameter::new("length", 50.0, Unit::Millimeter));
        doc.parameters
            .set(Parameter::new("width", 30.0, Unit::Millimeter));
        doc.parameters
            .set(Parameter::new("height", 20.0, Unit::Millimeter));

        // Create an initial parametric rectangular sketch on XY Plane
        let mut initial_sketch = Sketch2D::new("Sketch1", doc.xy_plane_id);
        let pt0 = initial_sketch.add_point(0.0, 0.0);
        let pt1 = initial_sketch.add_point(50.0, 0.0);
        let pt2 = initial_sketch.add_point(50.0, 30.0);
        let pt3 = initial_sketch.add_point(0.0, 30.0);

        let l_bottom = initial_sketch.add_line(pt0, pt1);
        let l_right = initial_sketch.add_line(pt1, pt2);
        let l_top = initial_sketch.add_line(pt3, pt2);
        let l_left = initial_sketch.add_line(pt0, pt3);

        let constraints = vec![
            Constraint::new(
                "Rect_H_Bottom",
                ConstraintKind::Horizontal { line: l_bottom },
            ),
            Constraint::new("Rect_H_Top", ConstraintKind::Horizontal { line: l_top }),
            Constraint::new("Rect_V_Left", ConstraintKind::Vertical { line: l_left }),
            Constraint::new("Rect_V_Right", ConstraintKind::Vertical { line: l_right }),
            Constraint::new(
                "FixOrigin",
                ConstraintKind::FixPoint {
                    point: pt0,
                    x: 0.0,
                    y: 0.0,
                },
            ),
            Constraint::new(
                "LengthDim",
                ConstraintKind::Distance {
                    point1: pt0,
                    point2: pt1,
                    distance: 50.0,
                },
            )
            .with_parameter("length"),
            Constraint::new(
                "WidthDim",
                ConstraintKind::Distance {
                    point1: pt0,
                    point2: pt3,
                    distance: 30.0,
                },
            )
            .with_parameter("width"),
        ];

        let sketch_id = initial_sketch.id;
        let mut sketches = HashMap::new();
        sketches.insert(
            sketch_id,
            StoredSketch {
                sketch: initial_sketch,
                constraints,
            },
        );

        // Add features to document feature tree
        let mut sketch_feat = Feature::new(
            "Sketch1",
            FeatureKind::Sketch {
                name: "Sketch1".into(),
                plane_id: doc.xy_plane_id,
            },
        );
        sketch_feat.id = sketch_id;
        doc.add_feature(sketch_feat);

        let mut extrude_feat = Feature::new(
            "Extrude1",
            FeatureKind::Extrude {
                name: "Extrude1".into(),
                sketch_id,
                distance: 20.0,
                is_cut: false,
            },
        );
        extrude_feat.visible = true;
        doc.add_feature(extrude_feat);

        let initial_pts = [[0.0, 0.0], [50.0, 0.0], [50.0, 30.0], [0.0, 30.0]];
        let initial_mesh = kernel.extrude_polygon(&initial_pts, 20.0).ok();

        Self {
            doc,
            history: HistoryStack::new(),
            camera: CadCamera::new(),
            kernel,
            active_mesh: initial_mesh,
            active_tab_id: "model_3d".to_string(),
            ribbon_schema: RibbonSchema::default_part_schema(),
            command_registry: Arc::new(CommandRegistry::default()),
            command_search_query: String::new(),
            active_modal: ActiveModal::None,
            selected_feature_id: None,
            status_message:
                "Ready. Rotate camera with MMB drag, pan with Right drag, zoom with Wheel.".into(),
            show_grid: true,
            show_axes: true,
            last_exported_path: None,
            active_sketch: None,
            sketches,
            wgpu_renderer: None,
            wgpu_render_state: None,
            viewport_texture_id: None,
            viewport_texture_size: (0, 0),
        }
    }
}

impl AppState {
    /// Enters 2D sketch mode on a specified datum plane.
    pub fn enter_sketch_on_plane(&mut self, plane_name: &str) {
        let next_sketch_num = self
            .doc
            .features
            .iter()
            .filter(|f| matches!(f.kind, FeatureKind::Sketch { .. }))
            .count()
            + 1;
        let sketch_name = format!("Sketch{}", next_sketch_num);

        let (plane_id, origin, x_axis, y_axis, normal) = match plane_name {
            "XZ Plane" => {
                self.camera.set_front_view();
                (self.doc.xz_plane_id, Vec3::ZERO, Vec3::X, Vec3::Z, -Vec3::Y)
            }
            "YZ Plane" => {
                self.camera.set_right_view();
                (self.doc.yz_plane_id, Vec3::ZERO, Vec3::Y, Vec3::Z, Vec3::X)
            }
            _ => {
                // Default: XY Plane
                self.camera.set_top_view();
                (self.doc.xy_plane_id, Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z)
            }
        };

        let session = ActiveSketchSession::new(
            sketch_name,
            plane_id,
            plane_name,
            origin,
            x_axis,
            y_axis,
            normal,
        );

        self.active_sketch = Some(session);
        self.active_tab_id = "sketch".to_string();
        self.active_modal = ActiveModal::None;
        self.status_message = format!(
            "2D Sketch mode active on {}. Draw entities or apply geometric constraints.",
            plane_name
        );
    }

    /// Exits sketch mode and commits the active sketch into the document features.
    pub fn finish_active_sketch(&mut self) -> Result<(), String> {
        if let Some(session) = self.active_sketch.take() {
            let sketch_id = session.id;
            let sketch_name = session.name.clone();
            let plane_id = session.plane_id;

            // Store sketch in persistent sketch storage
            self.sketches.insert(sketch_id, session.to_stored());

            // Add or update Feature in Document
            let existing_idx = self
                .doc
                .features
                .iter()
                .position(|f| f.id == sketch_id || f.name == sketch_name);

            if let Some(idx) = existing_idx {
                self.doc.features[idx].state = inventor_core::FeatureState::Active;
            } else {
                let mut feat = Feature::new(
                    sketch_name.clone(),
                    FeatureKind::Sketch {
                        name: sketch_name.clone(),
                        plane_id,
                    },
                );
                feat.id = sketch_id;
                self.doc.add_feature(feat);
            }

            self.active_tab_id = "model_3d".to_string();
            self.status_message = format!(
                "Finished sketch '{}'. Profile is ready for 3D modeling features.",
                sketch_name
            );
            Ok(())
        } else {
            Err("No active sketch session to finish.".into())
        }
    }

    pub fn rebuild_solid(&mut self) {
        use std::collections::HashSet;

        // 1. Sync parameter values from ParameterTable into stored sketches and re-solve
        for stored in self.sketches.values_mut() {
            let mut any_param_updated = false;
            for c in &mut stored.constraints {
                let target_param = c.parameter_name.as_deref().or_else(|| {
                    if self.doc.parameters.contains(&c.name) {
                        Some(c.name.as_str())
                    } else {
                        None
                    }
                });

                if let Some(param_name) = target_param
                    && let Some(val) = self.doc.parameters.get_value(param_name)
                {
                    c.update_from_parameter_value(val);
                    any_param_updated = true;
                }
            }

            if any_param_updated {
                let solver = ConstraintSolver::new();
                let _ = solver.solve(&mut stored.sketch, &stored.constraints);
            }
        }

        // 2. Sequential dependency evaluation in document feature order
        let mut evaluated_features: HashSet<Uuid> = HashSet::new();
        // Origin features are always valid base dependencies
        evaluated_features.insert(self.doc.xy_plane_id);
        evaluated_features.insert(self.doc.xz_plane_id);
        evaluated_features.insert(self.doc.yz_plane_id);
        evaluated_features.insert(self.doc.x_axis_id);
        evaluated_features.insert(self.doc.y_axis_id);
        evaluated_features.insert(self.doc.z_axis_id);
        evaluated_features.insert(self.doc.center_point_id);

        let mut solid_shapes: Vec<(SolidShape, Option<BooleanOp>, usize)> = Vec::new();

        enum DressUpOp {
            Fillet(f64),
            Chamfer(f64),
        }
        let mut dress_up_ops: Vec<(DressUpOp, usize)> = Vec::new();

        for i in 0..self.doc.features.len() {
            let feat = &self.doc.features[i];

            // Ignore origin features
            if i < 7 {
                continue;
            }

            // Suppressed features are skipped from evaluation
            if feat.state == inventor_core::FeatureState::Suppressed {
                continue;
            }

            match &feat.kind {
                FeatureKind::Sketch { name, plane_id } => {
                    let feat_id = feat.id;
                    let p_id = *plane_id;
                    let sk_name = name.clone();

                    if !evaluated_features.contains(&p_id) {
                        let f_mut = &mut self.doc.features[i];
                        f_mut.state = inventor_core::FeatureState::Error;
                        f_mut.error_message = Some(format!(
                            "Missing dependency: Reference plane for sketch '{}' is missing or suppressed.",
                            sk_name
                        ));
                    } else if let Some(stored) = self.sketches.get(&feat_id) {
                        // Check if sketch has a solver error
                        let solver = ConstraintSolver::new();
                        let status = solver.calculate_dof(&stored.sketch, &stored.constraints);
                        let f_mut = &mut self.doc.features[i];
                        if matches!(status, inventor_sketch::ConstraintStatus::Overconstrained) {
                            f_mut.state = inventor_core::FeatureState::Error;
                            f_mut.error_message = Some(format!(
                                "Sketch '{}' is over-constrained with conflicting constraints.",
                                sk_name
                            ));
                        } else {
                            f_mut.state = inventor_core::FeatureState::Active;
                            f_mut.error_message = None;
                            evaluated_features.insert(feat_id);
                        }
                    } else {
                        let f_mut = &mut self.doc.features[i];
                        f_mut.state = inventor_core::FeatureState::Error;
                        f_mut.error_message =
                            Some(format!("Sketch data for '{}' not found.", sk_name));
                    }
                }
                FeatureKind::Extrude {
                    name,
                    sketch_id,
                    distance,
                    is_cut,
                } => {
                    let feat_id = feat.id;
                    let s_id = *sketch_id;
                    let dist = *distance;
                    let feat_name = name.clone();
                    let cut = *is_cut;

                    // Check sequential dependency: was sketch evaluated before this feature?
                    if !evaluated_features.contains(&s_id) {
                        let f_mut = &mut self.doc.features[i];
                        f_mut.state = inventor_core::FeatureState::Error;
                        f_mut.error_message = Some(format!(
                            "Missing dependency: Sketch was not evaluated before '{}' (it may be reordered after this feature or suppressed).",
                            feat_name
                        ));
                    } else if let Some(stored) = self.sketches.get(&s_id) {
                        let actual_distance = self
                            .doc
                            .parameters
                            .get_value(&feat_name)
                            .or_else(|| {
                                if feat_name == "Extrude1" || feat_name == "Box1" {
                                    self.doc.parameters.get_value("height")
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(dist);

                        let profiles = stored.sketch.extract_closed_profiles();
                        if let Some(profile) = profiles.first() {
                            let (origin, x_axis, y_axis, normal) =
                                if stored.sketch.plane_id == self.doc.xz_plane_id {
                                    (
                                        [0.0, 0.0, 0.0],
                                        [1.0, 0.0, 0.0],
                                        [0.0, 0.0, 1.0],
                                        [0.0, -1.0, 0.0],
                                    )
                                } else if stored.sketch.plane_id == self.doc.yz_plane_id {
                                    (
                                        [0.0, 0.0, 0.0],
                                        [0.0, 1.0, 0.0],
                                        [0.0, 0.0, 1.0],
                                        [1.0, 0.0, 0.0],
                                    )
                                } else {
                                    (
                                        [0.0, 0.0, 0.0],
                                        [1.0, 0.0, 0.0],
                                        [0.0, 1.0, 0.0],
                                        [0.0, 0.0, 1.0],
                                    )
                                };

                            let (shape_origin, shape_height) = if cut {
                                let eps = 0.1;
                                let orig = [
                                    origin[0] - normal[0] * eps,
                                    origin[1] - normal[1] * eps,
                                    origin[2] - normal[2] * eps,
                                ];
                                (orig, actual_distance + 2.0 * eps)
                            } else {
                                (origin, actual_distance)
                            };

                            let shape = SolidShape::Extrude {
                                points: profile.clone(),
                                height: shape_height,
                                origin: shape_origin,
                                x_axis,
                                y_axis,
                                normal,
                            };
                            let op = if cut {
                                Some(BooleanOp::Subtract)
                            } else if solid_shapes.is_empty() {
                                None
                            } else {
                                Some(BooleanOp::Union)
                            };

                            if feat.visible {
                                solid_shapes.push((shape, op, i));
                            }
                            let f_mut = &mut self.doc.features[i];
                            f_mut.state = inventor_core::FeatureState::Active;
                            f_mut.error_message = None;
                            evaluated_features.insert(feat_id);
                        } else {
                            let f_mut = &mut self.doc.features[i];
                            f_mut.state = inventor_core::FeatureState::Error;
                            f_mut.error_message =
                                Some("Extrusion failed: No closed profile found in sketch.".into());
                        }
                    } else {
                        // Fallback primitive
                        let l = self.doc.parameters.get_value("length").unwrap_or(50.0);
                        let w = self.doc.parameters.get_value("width").unwrap_or(30.0);
                        let h = dist;
                        let shape = SolidShape::Box {
                            dx: l,
                            dy: w,
                            dz: h,
                        };
                        let op = if cut {
                            Some(BooleanOp::Subtract)
                        } else if solid_shapes.is_empty() {
                            None
                        } else {
                            Some(BooleanOp::Union)
                        };

                        if feat.visible {
                            solid_shapes.push((shape, op, i));
                        }
                        let f_mut = &mut self.doc.features[i];
                        f_mut.state = inventor_core::FeatureState::Active;
                        f_mut.error_message = None;
                        evaluated_features.insert(feat_id);
                    }
                }
                FeatureKind::Revolve {
                    name,
                    sketch_id,
                    angle_rad,
                    is_cut,
                } => {
                    let feat_id = feat.id;
                    let s_id = *sketch_id;
                    let feat_name = name.clone();
                    let angle = *angle_rad;
                    let cut = *is_cut;

                    if !evaluated_features.contains(&s_id) {
                        let f_mut = &mut self.doc.features[i];
                        f_mut.state = inventor_core::FeatureState::Error;
                        f_mut.error_message = Some(format!(
                            "Missing dependency: Sketch was not evaluated before '{}' (it may be reordered after this feature or suppressed).",
                            feat_name
                        ));
                    } else if let Some(stored) = self.sketches.get(&s_id) {
                        let profiles = stored.sketch.extract_closed_profiles();
                        if let Some(profile) = profiles.first() {
                            let (origin, axis) = if stored.sketch.plane_id == self.doc.xz_plane_id {
                                ([0.0, 0.0, 0.0], [0.0, 0.0, 1.0])
                            } else {
                                ([0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
                            };

                            let shape = SolidShape::Revolve {
                                points: profile.clone(),
                                origin,
                                axis,
                                angle_rad: angle,
                            };
                            let op = if cut {
                                Some(BooleanOp::Subtract)
                            } else if solid_shapes.is_empty() {
                                None
                            } else {
                                Some(BooleanOp::Union)
                            };

                            if feat.visible {
                                solid_shapes.push((shape, op, i));
                            }
                            let f_mut = &mut self.doc.features[i];
                            f_mut.state = inventor_core::FeatureState::Active;
                            f_mut.error_message = None;
                            evaluated_features.insert(feat_id);
                        } else {
                            let f_mut = &mut self.doc.features[i];
                            f_mut.state = inventor_core::FeatureState::Error;
                            f_mut.error_message =
                                Some("Revolve failed: No closed profile found in sketch.".into());
                        }
                    } else {
                        // Fallback cylinder primitive
                        let shape = SolidShape::Cylinder {
                            radius: 15.0,
                            height: 40.0,
                        };
                        let op = if cut {
                            Some(BooleanOp::Subtract)
                        } else if solid_shapes.is_empty() {
                            None
                        } else {
                            Some(BooleanOp::Union)
                        };

                        if feat.visible {
                            solid_shapes.push((shape, op, i));
                        }
                        let f_mut = &mut self.doc.features[i];
                        f_mut.state = inventor_core::FeatureState::Active;
                        f_mut.error_message = None;
                        evaluated_features.insert(feat_id);
                    }
                }
                FeatureKind::Boolean {
                    name: _,
                    op: _,
                    target_id,
                    tool_id,
                } => {
                    let feat_id = feat.id;
                    let t_id = *target_id;
                    let tl_id = *tool_id;

                    if !evaluated_features.contains(&t_id) || !evaluated_features.contains(&tl_id) {
                        let f_mut = &mut self.doc.features[i];
                        f_mut.state = inventor_core::FeatureState::Error;
                        f_mut.error_message = Some(
                            "Missing dependency: Target or Tool feature is missing or suppressed."
                                .into(),
                        );
                    } else {
                        let f_mut = &mut self.doc.features[i];
                        f_mut.state = inventor_core::FeatureState::Active;
                        f_mut.error_message = None;
                        evaluated_features.insert(feat_id);
                    }
                }
                FeatureKind::Fillet { name, radius } => {
                    let feat_id = feat.id;
                    let rad = *radius;
                    let feat_name = name.clone();

                    if solid_shapes.is_empty() {
                        let f_mut = &mut self.doc.features[i];
                        f_mut.state = inventor_core::FeatureState::Error;
                        f_mut.error_message = Some(format!(
                            "Missing dependency: No solid body exists prior to Fillet '{}'.",
                            feat_name
                        ));
                    } else if !self.kernel.supports_brep_fillet() {
                        let f_mut = &mut self.doc.features[i];
                        f_mut.state = inventor_core::FeatureState::Error;
                        f_mut.error_message = Some(format!(
                            "Unsupported by active kernel '{}': 3D B-rep edge filleting (radius = {:.2} mm) requires OpenCASCADE backend (evaluated in T8).",
                            self.kernel.kernel_name(),
                            rad
                        ));
                    } else {
                        let f_mut = &mut self.doc.features[i];
                        f_mut.state = inventor_core::FeatureState::Active;
                        f_mut.error_message = None;
                        evaluated_features.insert(feat_id);
                        dress_up_ops.push((DressUpOp::Fillet(rad), i));
                    }
                }
                FeatureKind::Chamfer { name, distance } => {
                    let feat_id = feat.id;
                    let dist = *distance;
                    let feat_name = name.clone();

                    if solid_shapes.is_empty() {
                        let f_mut = &mut self.doc.features[i];
                        f_mut.state = inventor_core::FeatureState::Error;
                        f_mut.error_message = Some(format!(
                            "Missing dependency: No solid body exists prior to Chamfer '{}'.",
                            feat_name
                        ));
                    } else if !self.kernel.supports_brep_fillet() {
                        let f_mut = &mut self.doc.features[i];
                        f_mut.state = inventor_core::FeatureState::Error;
                        f_mut.error_message = Some(format!(
                            "Unsupported by active kernel '{}': 3D B-rep edge chamfering (distance = {:.2} mm) requires OpenCASCADE backend (evaluated in T8).",
                            self.kernel.kernel_name(),
                            dist
                        ));
                    } else {
                        let f_mut = &mut self.doc.features[i];
                        f_mut.state = inventor_core::FeatureState::Active;
                        f_mut.error_message = None;
                        evaluated_features.insert(feat_id);
                        dress_up_ops.push((DressUpOp::Chamfer(dist), i));
                    }
                }
                _ => {
                    let feat_id = feat.id;
                    let f_mut = &mut self.doc.features[i];
                    f_mut.state = inventor_core::FeatureState::Active;
                    f_mut.error_message = None;
                    evaluated_features.insert(feat_id);
                }
            }
        }

        // Evaluate solid shapes into composite CadMesh
        let built_mesh = if solid_shapes.is_empty() {
            None
        } else if solid_shapes.len() == 1 {
            let (shape, _, feat_idx) = &solid_shapes[0];
            match self.kernel.build_solid_mesh(&[(shape.clone(), None)]) {
                Ok(mesh) => Some(mesh),
                Err(e) => {
                    let f_mut = &mut self.doc.features[*feat_idx];
                    f_mut.state = inventor_core::FeatureState::Error;
                    f_mut.error_message = Some(format!("Solid generation failed: {}", e));
                    None
                }
            }
        } else {
            let shapes_ops: Vec<(SolidShape, Option<BooleanOp>)> = solid_shapes
                .iter()
                .map(|(s, op, _)| (s.clone(), *op))
                .collect();
            match self.kernel.build_solid_mesh(&shapes_ops) {
                Ok(mesh) => Some(mesh),
                Err(e) => {
                    let last_idx = solid_shapes.last().map(|(_, _, idx)| *idx).unwrap_or(7);
                    let f_mut = &mut self.doc.features[last_idx];
                    f_mut.state = inventor_core::FeatureState::Error;
                    f_mut.error_message = Some(format!("Boolean modeling failed: {}", e));
                    None
                }
            }
        };

        // Apply sequential dress-up operations (fillet, chamfer) on the mesh
        if let Some(mut mesh) = built_mesh {
            for (op, feat_idx) in dress_up_ops {
                let res = match op {
                    DressUpOp::Fillet(r) => self.kernel.fillet_solid(&mesh, r),
                    DressUpOp::Chamfer(d) => self.kernel.chamfer_solid(&mesh, d),
                };
                match res {
                    Ok(new_mesh) => {
                        mesh = new_mesh;
                    }
                    Err(e) => {
                        let f_mut = &mut self.doc.features[feat_idx];
                        f_mut.state = inventor_core::FeatureState::Error;
                        f_mut.error_message = Some(format!("Dress-up modification failed: {}", e));
                    }
                }
            }
            self.active_mesh = Some(mesh);
        } else {
            self.active_mesh = None;
        }
    }
}
