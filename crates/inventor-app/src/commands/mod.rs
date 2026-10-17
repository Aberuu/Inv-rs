//! Command subsystem for inventor-rs.
//!
//! Provides the central CommandRegistry, typed errors, and standard handlers
//! for executing CAD operations across Ribbon, menus, shortcuts, and palettes.

pub mod error;
pub mod handlers;
pub mod registry;

#[allow(unused_imports)]
pub use error::CommandError;
#[allow(unused_imports)]
pub use registry::{CommandCategory, CommandEntry, CommandRegistry};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{ActiveModal, AppState};
    use inventor_render::ProjectionType;

    #[test]
    fn test_default_registry_contains_expected_commands() {
        let reg = CommandRegistry::default();

        let expected_ids = [
            "model.box",
            "model.cylinder",
            "model.extrude",
            "model.revolve",
            "model.combine",
            "model.fillet",
            "model.chamfer",
            "file.new",
            "file.export_step",
            "file.export_stl",
            "file.exit",
            "edit.undo",
            "edit.redo",
            "view.home",
            "view.front",
            "view.top",
            "view.right",
            "view.proj_perspective",
            "view.proj_ortho",
            "view.toggle_grid",
            "view.toggle_axes",
            "inspect.measure",
            "tools.parameters",
            "tools.options",
            "sketch.new",
            "sketch.finish",
            "sketch.select",
            "sketch.line",
            "sketch.circle",
            "sketch.rectangle",
            "sketch.point",
            "sketch.dimension",
            "sketch.coincident",
            "sketch.horizontal",
            "sketch.vertical",
            "sketch.parallel",
            "sketch.perpendicular",
            "sketch.tangent",
            "sketch.equal",
            "sketch.fix",
            "help.about",
        ];

        for id in expected_ids {
            assert!(
                reg.is_registered(id),
                "Expected command '{}' to be registered",
                id
            );
        }
    }

    #[test]
    fn test_execute_box_and_cylinder_commands() {
        let reg = CommandRegistry::default();
        let mut state = AppState::default();

        assert_eq!(state.active_modal, ActiveModal::None);

        reg.execute("model.box", &mut state).unwrap();
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

        reg.execute("model.cylinder", &mut state).unwrap();
        match state.active_modal {
            ActiveModal::CreateCylinder { radius, height } => {
                assert_eq!(radius, 15.0);
                assert_eq!(height, 40.0);
            }
            _ => panic!("Expected CreateCylinder modal"),
        }
    }

    #[test]
    fn test_execute_camera_commands() {
        let reg = CommandRegistry::default();
        let mut state = AppState::default();

        reg.execute("view.proj_ortho", &mut state).unwrap();
        assert_eq!(state.camera.projection_type, ProjectionType::Orthographic);

        reg.execute("view.proj_perspective", &mut state).unwrap();
        assert_eq!(state.camera.projection_type, ProjectionType::Perspective);

        let grid_initial = state.show_grid;
        reg.execute("view.toggle_grid", &mut state).unwrap();
        assert_eq!(state.show_grid, !grid_initial);

        let axes_initial = state.show_axes;
        reg.execute("view.toggle_axes", &mut state).unwrap();
        assert_eq!(state.show_axes, !axes_initial);

        reg.execute("view.home", &mut state).unwrap();
        assert!(state.status_message.contains("Isometric Home"));
    }

    #[test]
    fn test_execute_undo_redo_predicates() {
        let reg = CommandRegistry::default();
        let mut state = AppState::default();

        // Initially no undo
        assert!(!reg.is_enabled("edit.undo", &state));
        assert!(reg.execute("edit.undo", &mut state).is_err());

        // Perform document command
        let feat = inventor_core::Feature::new("Feat1", inventor_core::FeatureKind::OriginPoint);
        state.history.execute(
            Box::new(inventor_core::AddFeatureCmd::new(feat)),
            &mut state.doc,
        );

        assert!(reg.is_enabled("edit.undo", &state));
        reg.execute("edit.undo", &mut state).unwrap();
        assert_eq!(state.status_message, "Undo performed.");

        // Redo is now enabled
        assert!(reg.is_enabled("edit.redo", &state));
        reg.execute("edit.redo", &mut state).unwrap();
        assert_eq!(state.status_message, "Redo performed.");
    }

    #[test]
    fn test_execute_export_commands_with_mesh() {
        let reg = CommandRegistry::default();
        let mut state = AppState::default();

        // With mesh (default state has a box mesh)
        assert!(state.active_mesh.is_some());
        assert!(reg.is_enabled("file.export_step", &state));
        assert!(reg.is_enabled("file.export_stl", &state));

        reg.execute("file.export_step", &mut state).unwrap();
        assert_eq!(state.active_modal, ActiveModal::ExportStep);

        reg.execute("file.export_stl", &mut state).unwrap();
        assert_eq!(state.active_modal, ActiveModal::ExportStl);

        // Without mesh
        state.active_mesh = None;
        assert!(!reg.is_enabled("file.export_step", &state));
        assert!(!reg.is_enabled("file.export_stl", &state));

        assert!(reg.execute("file.export_step", &mut state).is_err());
        assert!(reg.execute("file.export_stl", &mut state).is_err());
    }

    #[test]
    fn test_unregistered_command_returns_not_found() {
        let reg = CommandRegistry::default();
        let mut state = AppState::default();

        let result = reg.execute("nonexistent.command", &mut state);
        match result {
            Err(CommandError::NotFound(id)) => assert_eq!(id, "nonexistent.command"),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_sketch_commands_predicates_and_execution() {
        let reg = CommandRegistry::default();
        let mut state = AppState::default();

        // 1. Initially outside sketch mode: sketch.new is enabled, tools/finish are disabled
        assert!(reg.is_enabled("sketch.new", &state));
        assert!(!reg.is_enabled("sketch.finish", &state));
        assert!(!reg.is_enabled("sketch.line", &state));
        assert!(!reg.is_enabled("sketch.circle", &state));
        assert!(!reg.is_enabled("sketch.rectangle", &state));
        assert!(!reg.is_enabled("sketch.point", &state));
        assert!(!reg.is_enabled("sketch.dimension", &state));
        assert!(!reg.is_enabled("sketch.horizontal", &state));
        assert!(!reg.is_enabled("sketch.vertical", &state));
        assert!(!reg.is_enabled("sketch.coincident", &state));
        assert!(!reg.is_enabled("sketch.fix", &state));

        // 2. Execute sketch.new to enter sketch mode
        reg.execute("sketch.new", &mut state).unwrap();
        assert!(state.active_sketch.is_some());
        assert_eq!(state.active_tab_id, "sketch");

        // 3. In sketch mode: sketch.new is disabled, tools/finish are enabled
        assert!(!reg.is_enabled("sketch.new", &state));
        assert!(reg.is_enabled("sketch.finish", &state));
        assert!(reg.is_enabled("sketch.line", &state));
        assert!(reg.is_enabled("sketch.circle", &state));
        assert!(reg.is_enabled("sketch.rectangle", &state));
        assert!(reg.is_enabled("sketch.point", &state));
        assert!(reg.is_enabled("sketch.dimension", &state));
        assert!(reg.is_enabled("sketch.horizontal", &state));
        assert!(reg.is_enabled("sketch.vertical", &state));
        assert!(reg.is_enabled("sketch.coincident", &state));
        assert!(reg.is_enabled("sketch.fix", &state));

        // 4. Switch tools via registry
        reg.execute("sketch.line", &mut state).unwrap();
        assert_eq!(
            state.active_sketch.as_ref().unwrap().active_tool,
            crate::sketch_session::SketchTool::Line
        );

        reg.execute("sketch.circle", &mut state).unwrap();
        assert_eq!(
            state.active_sketch.as_ref().unwrap().active_tool,
            crate::sketch_session::SketchTool::Circle
        );

        reg.execute("sketch.rectangle", &mut state).unwrap();
        assert_eq!(
            state.active_sketch.as_ref().unwrap().active_tool,
            crate::sketch_session::SketchTool::Rectangle
        );

        reg.execute("sketch.point", &mut state).unwrap();
        assert_eq!(
            state.active_sketch.as_ref().unwrap().active_tool,
            crate::sketch_session::SketchTool::Point
        );

        reg.execute("sketch.dimension", &mut state).unwrap();
        assert_eq!(
            state.active_sketch.as_ref().unwrap().active_tool,
            crate::sketch_session::SketchTool::Dimension
        );

        reg.execute("sketch.select", &mut state).unwrap();
        assert_eq!(
            state.active_sketch.as_ref().unwrap().active_tool,
            crate::sketch_session::SketchTool::Select
        );

        // 5. Finish sketch via registry
        reg.execute("sketch.finish", &mut state).unwrap();
        assert!(state.active_sketch.is_none());
        assert_eq!(state.active_tab_id, "model_3d");
        assert!(reg.is_enabled("sketch.new", &state));
        assert!(!reg.is_enabled("sketch.finish", &state));
    }

    #[test]
    fn test_sketch_mode_lifecycle_and_rectangle_creation() {
        use glam::Vec2;
        use inventor_core::FeatureKind;
        use inventor_sketch::{Constraint, ConstraintKind, ConstraintStatus};

        let mut state = AppState::default();

        // 1. Enter sketch on XY plane
        state.enter_sketch_on_plane("XY Plane");
        assert!(state.active_sketch.is_some());

        let session = state.active_sketch.as_mut().unwrap();
        assert_eq!(session.plane_name, "XY Plane");
        assert_eq!(session.name, "Sketch2");

        // 2. Add rectangle (0,0) to (100,50)
        let (pts, lines) = session.add_rectangle(Vec2::new(0.0, 0.0), Vec2::new(100.0, 50.0));
        assert_eq!(pts.len(), 4);
        assert_eq!(lines.len(), 4);
        assert_eq!(session.constraints.len(), 4);

        // 4 points = 8 DOFs, 4 horizontal/vertical constraints = 4 DOFs remaining
        assert_eq!(session.current_dof, ConstraintStatus::Underconstrained(4));

        // 3. Fix corner pt0 at (0, 0) -> drops to 2 DOFs
        let fix_origin = Constraint::new(
            "FixOrigin",
            ConstraintKind::FixPoint {
                point: pts[0],
                x: 0.0,
                y: 0.0,
            },
        );
        let status = session.add_constraint(fix_origin).unwrap();
        assert_eq!(status, ConstraintStatus::Underconstrained(2));

        // 4. Add width distance constraint (100.0) -> drops to 1 DOF
        let width_dim = Constraint::new(
            "WidthDim",
            ConstraintKind::Distance {
                point1: pts[0],
                point2: pts[1],
                distance: 100.0,
            },
        );
        let status = session.add_constraint(width_dim).unwrap();
        assert_eq!(status, ConstraintStatus::Underconstrained(1));

        // 5. Add height distance constraint (50.0) -> drops to 0 DOF (FullyConstrained)
        let height_dim = Constraint::new(
            "HeightDim",
            ConstraintKind::Distance {
                point1: pts[0],
                point2: pts[3],
                distance: 50.0,
            },
        );
        let status = session.add_constraint(height_dim).unwrap();
        assert_eq!(status, ConstraintStatus::FullyConstrained);

        // Check solved rectangle vertices
        let p0 = session.sketch.find_point(pts[0]).unwrap();
        let p1 = session.sketch.find_point(pts[1]).unwrap();
        let p2 = session.sketch.find_point(pts[2]).unwrap();
        let p3 = session.sketch.find_point(pts[3]).unwrap();

        assert!((p0.x - 0.0).abs() < 1e-4);
        assert!((p0.y - 0.0).abs() < 1e-4);
        assert!((p1.x - 100.0).abs() < 1e-4);
        assert!((p1.y - 0.0).abs() < 1e-4);
        assert!((p2.x - 100.0).abs() < 1e-4);
        assert!((p2.y - 50.0).abs() < 1e-4);
        assert!((p3.x - 0.0).abs() < 1e-4);
        assert!((p3.y - 50.0).abs() < 1e-4);

        let sketch_id = session.id;

        // 6. Finish active sketch: commits to document and stored sketches
        state.finish_active_sketch().unwrap();
        assert!(state.active_sketch.is_none());
        assert_eq!(state.sketches.len(), 2);
        assert!(state.sketches.contains_key(&sketch_id));

        // Verify feature tree in document
        let feat = state
            .doc
            .features
            .iter()
            .find(|f| f.id == sketch_id)
            .expect("Sketch feature must exist in doc");
        assert_eq!(feat.name, "Sketch2");
        assert!(matches!(feat.kind, FeatureKind::Sketch { .. }));
    }

    #[test]
    fn test_sketch_mode_edit_stored_sketch() {
        use crate::sketch_session::ActiveSketchSession;
        use glam::{Vec2, Vec3};
        use inventor_sketch::{Constraint, ConstraintKind, ConstraintStatus};

        let mut state = AppState::default();

        // Create and finish a rectangle sketch
        state.enter_sketch_on_plane("XY Plane");
        let sketch_id;
        {
            let session = state.active_sketch.as_mut().unwrap();
            let (pts, _) = session.add_rectangle(Vec2::new(0.0, 0.0), Vec2::new(100.0, 50.0));
            session
                .add_constraint(Constraint::new(
                    "FixOrigin",
                    ConstraintKind::FixPoint {
                        point: pts[0],
                        x: 0.0,
                        y: 0.0,
                    },
                ))
                .unwrap();
            session
                .add_constraint(Constraint::new(
                    "WidthDim",
                    ConstraintKind::Distance {
                        point1: pts[0],
                        point2: pts[1],
                        distance: 100.0,
                    },
                ))
                .unwrap();
            session
                .add_constraint(Constraint::new(
                    "HeightDim",
                    ConstraintKind::Distance {
                        point1: pts[0],
                        point2: pts[3],
                        distance: 50.0,
                    },
                ))
                .unwrap();
            sketch_id = session.id;
        }
        state.finish_active_sketch().unwrap();

        // Re-enter sketch mode for editing from stored sketch
        let stored = state
            .sketches
            .get(&sketch_id)
            .cloned()
            .expect("Stored sketch must exist");
        let mut session = ActiveSketchSession::from_stored(
            stored,
            "XY Plane",
            Vec3::ZERO,
            Vec3::X,
            Vec3::Y,
            Vec3::Z,
        );
        assert_eq!(session.current_dof, ConstraintStatus::FullyConstrained);

        // Edit width dimension from 100.0 to 120.0
        let mut found_width = false;
        for c in &mut session.constraints {
            if let ConstraintKind::Distance {
                point1: _,
                point2: _,
                distance,
            } = &mut c.kind
                && (*distance - 100.0).abs() < 1e-4
            {
                *distance = 120.0;
                found_width = true;
                break;
            }
        }
        assert!(found_width, "Width constraint should be found");

        let status = session.solve_constraints().unwrap();
        assert_eq!(status, ConstraintStatus::FullyConstrained);

        // Check updated coordinates
        let p1 = session
            .sketch
            .points
            .iter()
            .find(|p| (p.x - 120.0).abs() < 1e-4 && (p.y - 0.0).abs() < 1e-4);
        assert!(p1.is_some(), "Point p1 should be at x = 120.0");
        let p2 = session
            .sketch
            .points
            .iter()
            .find(|p| (p.x - 120.0).abs() < 1e-4 && (p.y - 50.0).abs() < 1e-4);
        assert!(p2.is_some(), "Point p2 should be at x = 120.0, y = 50.0");

        // Commit edited sketch back
        state.active_sketch = Some(session);
        state.finish_active_sketch().unwrap();

        // Stored sketch reflects 120.0
        let updated_stored = state.sketches.get(&sketch_id).unwrap();
        let updated_width_c = updated_stored.constraints.iter().find(|c| {
            if let ConstraintKind::Distance { distance, .. } = c.kind {
                (distance - 120.0).abs() < 1e-4
            } else {
                false
            }
        });
        assert!(updated_width_c.is_some());
    }

    #[test]
    fn test_sketch_constraint_commands_with_selection() {
        let reg = CommandRegistry::default();
        let mut state = AppState::default();

        state.enter_sketch_on_plane("XY Plane");
        let (pt1, pt2, line_id);
        {
            let session = state.active_sketch.as_mut().unwrap();
            pt1 = session.add_point(0.0, 10.0);
            pt2 = session.add_point(50.0, 15.0);
            line_id = session.add_line(pt1, pt2);
            // Select the line
            session.selected_lines.push(line_id);
        }

        // Apply Horizontal constraint via command registry
        reg.execute("sketch.horizontal", &mut state).unwrap();

        {
            let session = state.active_sketch.as_mut().unwrap();
            assert_eq!(session.constraints.len(), 1);
            let p1 = session.sketch.find_point(pt1).unwrap();
            let p2 = session.sketch.find_point(pt2).unwrap();
            assert!((p1.y - p2.y).abs() < 1e-4);

            // Select 2 points for dimension
            session.selected_lines.clear();
            session.selected_points.push(pt1);
            session.selected_points.push(pt2);
        }

        // Apply dimension via command registry -> should open DimensionInput modal
        reg.execute("sketch.dimension", &mut state).unwrap();
        match state.active_modal {
            ActiveModal::DimensionInput {
                title,
                is_circle,
                is_angle,
                current_value,
            } => {
                assert!(title.contains("Distance"));
                assert!(!is_circle);
                assert!(!is_angle);
                assert!((current_value - 50.0).abs() < 1e-4);
            }
            _ => panic!("Expected DimensionInput modal"),
        }
    }

    #[test]
    fn test_parametric_extrude_and_regeneration_from_parameter_table() {
        use inventor_core::{Parameter, Unit};

        let mut state = AppState::default();

        // 1. Initial parametric state (50 x 30 x 20 mm)
        assert!(state.active_mesh.is_some());
        let (min, max) = state
            .active_mesh
            .as_ref()
            .unwrap()
            .compute_bounding_box()
            .unwrap();
        assert!((max[0] - min[0] - 50.0).abs() < 1e-1);
        assert!((max[1] - min[1] - 30.0).abs() < 1e-1);
        assert!((max[2] - min[2] - 20.0).abs() < 1e-1);

        // 2. Change length parameter to 120 mm and regenerate solid
        state
            .doc
            .parameters
            .set(Parameter::new("length", 120.0, Unit::Millimeter));
        state.rebuild_solid();

        let (min, max) = state
            .active_mesh
            .as_ref()
            .unwrap()
            .compute_bounding_box()
            .unwrap();
        assert!((max[0] - min[0] - 120.0).abs() < 1e-1);
        assert!((max[1] - min[1] - 30.0).abs() < 1e-1);
        assert!((max[2] - min[2] - 20.0).abs() < 1e-1);

        // 3. Change width parameter to 75 mm and regenerate solid
        state
            .doc
            .parameters
            .set(Parameter::new("width", 75.0, Unit::Millimeter));
        state.rebuild_solid();

        let (min, max) = state
            .active_mesh
            .as_ref()
            .unwrap()
            .compute_bounding_box()
            .unwrap();
        assert!((max[0] - min[0] - 120.0).abs() < 1e-1);
        assert!((max[1] - min[1] - 75.0).abs() < 1e-1);
        assert!((max[2] - min[2] - 20.0).abs() < 1e-1);

        // 4. Change height parameter to 45 mm and regenerate solid
        state
            .doc
            .parameters
            .set(Parameter::new("height", 45.0, Unit::Millimeter));
        state.rebuild_solid();

        let (min, max) = state
            .active_mesh
            .as_ref()
            .unwrap()
            .compute_bounding_box()
            .unwrap();
        assert!((max[0] - min[0] - 120.0).abs() < 1e-1);
        assert!((max[1] - min[1] - 75.0).abs() < 1e-1);
        assert!((max[2] - min[2] - 45.0).abs() < 1e-1);
    }

    #[test]
    fn test_extrude_new_sketch_profile_via_command() {
        use glam::Vec2;
        use inventor_core::{AddFeatureCmd, Feature, FeatureKind};

        let reg = CommandRegistry::default();
        let mut state = AppState::default();
        state.doc.features[8].state = inventor_core::FeatureState::Suppressed;

        // 1. Enter sketch on XZ Plane
        state.enter_sketch_on_plane("XZ Plane");
        {
            let session = state.active_sketch.as_mut().unwrap();
            let _ = session.add_rectangle(Vec2::new(0.0, 0.0), Vec2::new(60.0, 40.0));
        }
        let sketch_id = state.active_sketch.as_ref().unwrap().id;
        state.finish_active_sketch().unwrap();

        // 2. Execute model.extrude
        reg.execute("model.extrude", &mut state).unwrap();

        match state.active_modal {
            ActiveModal::CreateExtrude {
                sketch_id: sid,
                distance,
                is_cut,
            } => {
                assert_eq!(sid, Some(sketch_id));
                assert!((distance - 25.0).abs() < 1e-4);
                assert!(!is_cut);
            }
            _ => panic!("Expected CreateExtrude modal"),
        }

        // 3. Confirm extrude with distance 35.0 mm
        let feat = Feature::new(
            "Extrude2",
            FeatureKind::Extrude {
                name: "Extrude2".into(),
                sketch_id,
                distance: 35.0,
                is_cut: false,
            },
        );
        state
            .history
            .execute(Box::new(AddFeatureCmd::new(feat)), &mut state.doc);
        state.rebuild_solid();

        assert!(state.active_mesh.is_some());
        let (min, max) = state
            .active_mesh
            .as_ref()
            .unwrap()
            .compute_bounding_box()
            .unwrap();

        let dx = max[0] - min[0];
        let dy = max[1] - min[1];
        let dz = max[2] - min[2];

        assert!((dx - 60.0).abs() < 1e-1);
        assert!((dy - 35.0).abs() < 1e-1); // along Y (XZ plane normal)
        assert!((dz - 40.0).abs() < 1e-1); // along Z
    }

    #[test]
    fn test_feature_tree_suppress_and_unsuppress() {
        use inventor_core::{FeatureState, ToggleSuppressFeatureCmd};

        let mut state = AppState::default();
        let sketch1_id = state.doc.features[7].id;
        let extrude1_id = state.doc.features[8].id;

        // Initial state is active and solid mesh exists
        assert!(state.active_mesh.is_some());
        assert_eq!(state.doc.features[8].state, FeatureState::Active);

        // 1. Suppress Extrude1
        state.history.execute(
            Box::new(ToggleSuppressFeatureCmd::new(extrude1_id)),
            &mut state.doc,
        );
        state.rebuild_solid();

        assert_eq!(state.doc.features[8].state, FeatureState::Suppressed);
        assert!(state.active_mesh.is_none());

        // 2. Undo suppression of Extrude1
        state.history.undo(&mut state.doc);
        state.rebuild_solid();

        assert_eq!(state.doc.features[8].state, FeatureState::Active);
        assert!(state.active_mesh.is_some());

        // 3. Suppress Sketch1 (upstream dependency of Extrude1)
        state.history.execute(
            Box::new(ToggleSuppressFeatureCmd::new(sketch1_id)),
            &mut state.doc,
        );
        state.rebuild_solid();

        assert_eq!(state.doc.features[7].state, FeatureState::Suppressed);
        // Extrude1 should now fail with clear dependency error
        assert_eq!(state.doc.features[8].state, FeatureState::Error);
        assert!(
            state.doc.features[8]
                .error_message
                .as_ref()
                .unwrap()
                .contains("Missing dependency")
        );
        assert!(state.active_mesh.is_none());

        // 4. Unsuppress Sketch1 -> Extrude1 recovers to Active
        state.history.execute(
            Box::new(ToggleSuppressFeatureCmd::new(sketch1_id)),
            &mut state.doc,
        );
        state.rebuild_solid();

        assert_eq!(state.doc.features[7].state, FeatureState::Active);
        assert_eq!(state.doc.features[8].state, FeatureState::Active);
        assert!(state.doc.features[8].error_message.is_none());
        assert!(state.active_mesh.is_some());
    }

    #[test]
    fn test_feature_tree_reorder_and_dependency_error() {
        use inventor_core::{FeatureState, ReorderFeatureCmd};

        let mut state = AppState::default();
        let extrude1_id = state.doc.features[8].id;

        // Initial order: index 7 is Sketch1, index 8 is Extrude1
        assert_eq!(state.doc.features[7].name, "Sketch1");
        assert_eq!(state.doc.features[8].name, "Extrude1");
        assert!(state.active_mesh.is_some());

        // 1. Reorder Extrude1 up before Sketch1
        state.history.execute(
            Box::new(ReorderFeatureCmd::new(extrude1_id, true)),
            &mut state.doc,
        );
        state.rebuild_solid();

        // Extrude1 is now at index 7, Sketch1 is at index 8
        assert_eq!(state.doc.features[7].name, "Extrude1");
        assert_eq!(state.doc.features[8].name, "Sketch1");

        // Extrude1 should be in Error state because its prerequisite Sketch1 has not been evaluated yet
        assert_eq!(state.doc.features[7].state, FeatureState::Error);
        assert!(
            state.doc.features[7]
                .error_message
                .as_ref()
                .unwrap()
                .contains("Missing dependency")
        );
        assert!(state.active_mesh.is_none());

        // 2. Reorder Extrude1 back down after Sketch1
        state.history.execute(
            Box::new(ReorderFeatureCmd::new(extrude1_id, false)),
            &mut state.doc,
        );
        state.rebuild_solid();

        assert_eq!(state.doc.features[7].name, "Sketch1");
        assert_eq!(state.doc.features[8].name, "Extrude1");
        assert_eq!(state.doc.features[8].state, FeatureState::Active);
        assert!(state.doc.features[8].error_message.is_none());
        assert!(state.active_mesh.is_some());

        // 3. Undo reorder down via HistoryStack (moves Extrude1 back up)
        state.history.undo(&mut state.doc);
        state.rebuild_solid();

        assert_eq!(state.doc.features[7].name, "Extrude1");
        assert_eq!(state.doc.features[7].state, FeatureState::Error);

        // 4. Redo reorder via HistoryStack (restores correct order)
        state.history.redo(&mut state.doc);
        state.rebuild_solid();

        assert_eq!(state.doc.features[8].name, "Extrude1");
        assert_eq!(state.doc.features[8].state, FeatureState::Active);
        assert!(state.active_mesh.is_some());
    }

    #[test]
    fn test_feature_tree_edit_extrude_parameters() {
        use inventor_core::{FeatureKind, ModifyFeatureCmd, Parameter, Unit};

        let mut state = AppState::default();
        let sketch1_id = state.doc.features[7].id;
        let extrude1_id = state.doc.features[8].id;

        // Verify initial bounding box height: 20 mm
        let (min, max) = state
            .active_mesh
            .as_ref()
            .unwrap()
            .compute_bounding_box()
            .unwrap();
        assert!((max[2] - min[2] - 20.0).abs() < 1e-1);

        // 1. Edit Extrude1 distance to 55 mm
        let cmd = ModifyFeatureCmd::new(
            extrude1_id,
            FeatureKind::Extrude {
                name: "Extrude1".into(),
                sketch_id: sketch1_id,
                distance: 55.0,
                is_cut: false,
            },
        );
        state.history.execute(Box::new(cmd), &mut state.doc);
        state
            .doc
            .parameters
            .set(Parameter::new("height", 55.0, Unit::Millimeter));
        state.rebuild_solid();

        // Solid regenerated to 55 mm
        let (min, max) = state
            .active_mesh
            .as_ref()
            .unwrap()
            .compute_bounding_box()
            .unwrap();
        assert!((max[2] - min[2] - 55.0).abs() < 1e-1);

        // 2. Undo edit via HistoryStack
        state.history.undo(&mut state.doc);
        state
            .doc
            .parameters
            .set(Parameter::new("height", 20.0, Unit::Millimeter));
        state.rebuild_solid();

        // Solid returned to 20 mm
        let (min, max) = state
            .active_mesh
            .as_ref()
            .unwrap()
            .compute_bounding_box()
            .unwrap();
        assert!((max[2] - min[2] - 20.0).abs() < 1e-1);
    }

    #[test]
    fn test_revolve_command_and_feature_regeneration() {
        use crate::sketch_session::StoredSketch;
        use inventor_core::{AddFeatureCmd, Feature, FeatureKind, ModifyFeatureCmd};
        use inventor_sketch::Sketch2D;
        use std::f64::consts::TAU;

        let reg = CommandRegistry::default();
        let mut state = AppState::default();

        // 1. Verify model.revolve command opens CreateRevolve modal
        reg.execute("model.revolve", &mut state).unwrap();
        match state.active_modal {
            ActiveModal::CreateRevolve {
                sketch_id,
                angle_deg,
                is_cut,
            } => {
                assert!(sketch_id.is_some());
                assert_eq!(angle_deg, 360.0);
                assert!(!is_cut);
            }
            _ => panic!("Expected CreateRevolve modal"),
        }

        // 2. Create an offset rectangular sketch suitable for revolving into a turned ring/bushing
        let mut revolve_sketch = Sketch2D::new("RevolveSketch", state.doc.xy_plane_id);
        let p0 = revolve_sketch.add_point(10.0, 0.0);
        let p1 = revolve_sketch.add_point(20.0, 0.0);
        let p2 = revolve_sketch.add_point(20.0, 30.0);
        let p3 = revolve_sketch.add_point(10.0, 30.0);
        revolve_sketch.add_line(p0, p1);
        revolve_sketch.add_line(p1, p2);
        revolve_sketch.add_line(p2, p3);
        revolve_sketch.add_line(p3, p0);

        let rev_sketch_id = revolve_sketch.id;
        state.sketches.insert(
            rev_sketch_id,
            StoredSketch {
                sketch: revolve_sketch,
                constraints: Vec::new(),
            },
        );

        let mut sk_feat = Feature::new(
            "RevolveSketch",
            FeatureKind::Sketch {
                name: "RevolveSketch".into(),
                plane_id: state.doc.xy_plane_id,
            },
        );
        sk_feat.id = rev_sketch_id;
        state
            .history
            .execute(Box::new(AddFeatureCmd::new(sk_feat)), &mut state.doc);

        // Suppress Extrude1 so only the revolve solid is evaluated
        let _extrude1_id = state.doc.features[8].id;
        state.doc.features[8].state = inventor_core::FeatureState::Suppressed;

        // 3. Add Revolve feature (360 degrees)
        let rev_feat = Feature::new(
            "Revolve1",
            FeatureKind::Revolve {
                name: "Revolve1".into(),
                sketch_id: rev_sketch_id,
                angle_rad: TAU,
                is_cut: false,
            },
        );
        let rev_id = rev_feat.id;
        state
            .history
            .execute(Box::new(AddFeatureCmd::new(rev_feat)), &mut state.doc);
        state.rebuild_solid();

        assert!(state.active_mesh.is_some());
        let (min, max) = state
            .active_mesh
            .as_ref()
            .unwrap()
            .compute_bounding_box()
            .unwrap();
        let dx = max[0] - min[0];
        let dy = max[1] - min[1];
        let dz = max[2] - min[2];
        assert!((dx - 40.0).abs() < 1e-1);
        assert!((dy - 30.0).abs() < 1e-1);
        assert!((dz - 40.0).abs() < 1e-1);

        // 4. Modify revolve angle to 90 degrees (TAU / 4.0)
        let modify_cmd = ModifyFeatureCmd::new(
            rev_id,
            FeatureKind::Revolve {
                name: "Revolve1".into(),
                sketch_id: rev_sketch_id,
                angle_rad: TAU / 4.0,
                is_cut: false,
            },
        );
        state.history.execute(Box::new(modify_cmd), &mut state.doc);
        state.rebuild_solid();
        assert!(state.active_mesh.is_some());

        // 5. Undo modification via HistoryStack
        state.history.undo(&mut state.doc);
        state.rebuild_solid();
        let (min, max) = state
            .active_mesh
            .as_ref()
            .unwrap()
            .compute_bounding_box()
            .unwrap();
        assert!((max[0] - min[0] - 40.0).abs() < 1e-1);
    }

    #[test]
    fn test_boolean_combine_command_and_cut_regeneration() {
        use crate::sketch_session::StoredSketch;
        use inventor_core::{AddFeatureCmd, BooleanOp, Feature, FeatureKind};
        use inventor_sketch::Sketch2D;

        let reg = CommandRegistry::default();
        let mut state = AppState::default();

        // 1. Verify model.combine command opens BooleanCombine modal
        reg.execute("model.combine", &mut state).unwrap();
        match state.active_modal {
            ActiveModal::BooleanCombine {
                target_feature,
                tool_feature: _,
                op,
            } => {
                assert!(target_feature.is_some());
                assert_eq!(op, BooleanOp::Union);
            }
            _ => panic!("Expected BooleanCombine modal"),
        }

        // 2. Extrude1 has bounding box (50 x 30 x 20) with initial face/tri count
        let initial_tri_count = state.active_mesh.as_ref().unwrap().indices.len() / 3;

        // 3. Create a second sketch for a cutting tool (e.g. 20 x 20 notch inside base)
        let mut cut_sketch = Sketch2D::new("CutSketch", state.doc.xy_plane_id);
        let p0 = cut_sketch.add_point(10.0, 5.0);
        let p1 = cut_sketch.add_point(30.0, 5.0);
        let p2 = cut_sketch.add_point(30.0, 25.0);
        let p3 = cut_sketch.add_point(10.0, 25.0);
        cut_sketch.add_line(p0, p1);
        cut_sketch.add_line(p1, p2);
        cut_sketch.add_line(p2, p3);
        cut_sketch.add_line(p3, p0);

        let cut_sketch_id = cut_sketch.id;
        state.sketches.insert(
            cut_sketch_id,
            StoredSketch {
                sketch: cut_sketch,
                constraints: Vec::new(),
            },
        );

        let mut sk_feat = Feature::new(
            "CutSketch",
            FeatureKind::Sketch {
                name: "CutSketch".into(),
                plane_id: state.doc.xy_plane_id,
            },
        );
        sk_feat.id = cut_sketch_id;
        state
            .history
            .execute(Box::new(AddFeatureCmd::new(sk_feat)), &mut state.doc);

        // 4. Add Extrude2 as a Cut feature (is_cut: true)
        let cut_feat = Feature::new(
            "Extrude2",
            FeatureKind::Extrude {
                name: "Extrude2".into(),
                sketch_id: cut_sketch_id,
                distance: 20.0,
                is_cut: true,
            },
        );
        state
            .history
            .execute(Box::new(AddFeatureCmd::new(cut_feat)), &mut state.doc);
        state.rebuild_solid();

        assert!(state.active_mesh.is_some());
        let cut_mesh = state.active_mesh.as_ref().unwrap();
        let cut_tri_count = cut_mesh.indices.len() / 3;
        // B-rep boolean cut results in additional pocket/faces, increasing triangle count
        assert!(cut_tri_count > initial_tri_count);

        // 5. Undo the cut via HistoryStack
        state.history.undo(&mut state.doc);
        state.rebuild_solid();
        let undone_tri_count = state.active_mesh.as_ref().unwrap().indices.len() / 3;
        assert_eq!(undone_tri_count, initial_tri_count);

        // 6. Redo the cut
        state.history.redo(&mut state.doc);
        state.rebuild_solid();
        let redone_tri_count = state.active_mesh.as_ref().unwrap().indices.len() / 3;
        assert_eq!(redone_tri_count, cut_tri_count);
    }

    #[test]
    fn test_fillet_and_chamfer_commands_and_evaluation() {
        use inventor_core::{AddFeatureCmd, Feature, FeatureKind, FeatureState};
        use inventor_kernel::KernelBackend;

        let reg = CommandRegistry::default();
        let mut state = AppState::default();

        // 1. Initial part has Extrude1 evaluated
        state.rebuild_solid();
        assert!(state.active_mesh.is_some());

        // 2. Trigger model.fillet command from registry
        reg.execute("model.fillet", &mut state).unwrap();
        assert_eq!(
            state.active_modal,
            ActiveModal::CreateFillet { radius: 2.0 }
        );

        // Add Fillet1 feature to document
        let fillet_feat = Feature::new(
            "Fillet1",
            FeatureKind::Fillet {
                name: "Fillet1".into(),
                radius: 2.0,
            },
        );
        state
            .history
            .execute(Box::new(AddFeatureCmd::new(fillet_feat)), &mut state.doc);

        // 3. Trigger model.chamfer command from registry
        reg.execute("model.chamfer", &mut state).unwrap();
        assert_eq!(
            state.active_modal,
            ActiveModal::CreateChamfer { distance: 2.0 }
        );

        // Add Chamfer1 feature to document
        let chamfer_feat = Feature::new(
            "Chamfer1",
            FeatureKind::Chamfer {
                name: "Chamfer1".into(),
                distance: 2.0,
            },
        );
        state
            .history
            .execute(Box::new(AddFeatureCmd::new(chamfer_feat)), &mut state.doc);

        // 4. Evaluate under default Pure-Rust truck kernel
        assert_eq!(state.kernel.backend, KernelBackend::Truck);
        state.rebuild_solid();

        // The base solid mesh remains active and visible in the viewport
        assert!(state.active_mesh.is_some());

        // Fillet1 and Chamfer1 are marked as Error with diagnostic message explaining T8 evaluation
        let fillet_in_doc = state
            .doc
            .features
            .iter()
            .find(|f| f.name == "Fillet1")
            .unwrap();
        assert_eq!(fillet_in_doc.state, FeatureState::Error);
        assert!(
            fillet_in_doc
                .error_message
                .as_ref()
                .unwrap()
                .contains("evaluated in T8")
        );
        assert!(
            fillet_in_doc
                .error_message
                .as_ref()
                .unwrap()
                .contains("OpenCASCADE")
        );

        let chamfer_in_doc = state
            .doc
            .features
            .iter()
            .find(|f| f.name == "Chamfer1")
            .unwrap();
        assert_eq!(chamfer_in_doc.state, FeatureState::Error);
        assert!(
            chamfer_in_doc
                .error_message
                .as_ref()
                .unwrap()
                .contains("evaluated in T8")
        );
        assert!(
            chamfer_in_doc
                .error_message
                .as_ref()
                .unwrap()
                .contains("OpenCASCADE")
        );

        // 5. Switch to secondary OpenCASCADE backend behind the same GeometryKernel trait
        state.kernel.backend = KernelBackend::OpenCascade;
        state.rebuild_solid();

        // Now Fillet1 and Chamfer1 succeed and are Active
        let fillet_occt = state
            .doc
            .features
            .iter()
            .find(|f| f.name == "Fillet1")
            .unwrap();
        assert_eq!(fillet_occt.state, FeatureState::Active);
        assert!(fillet_occt.error_message.is_none());

        let chamfer_occt = state
            .doc
            .features
            .iter()
            .find(|f| f.name == "Chamfer1")
            .unwrap();
        assert_eq!(chamfer_occt.state, FeatureState::Active);
        assert!(chamfer_occt.error_message.is_none());

        // Mesh is modified and intact
        assert!(state.active_mesh.is_some());
    }

    #[test]
    fn test_fillet_chamfer_edit_parameters_and_undo_redo() {
        use inventor_core::{
            AddFeatureCmd, Feature, FeatureKind, FeatureState, ModifyFeatureCmd,
            ToggleSuppressFeatureCmd,
        };
        use inventor_kernel::KernelBackend;

        let mut state = AppState::default();
        state.kernel.backend = KernelBackend::OpenCascade;
        state.rebuild_solid();

        let fillet = Feature::new(
            "Fillet1",
            FeatureKind::Fillet {
                name: "Fillet1".into(),
                radius: 2.0,
            },
        );
        let fillet_id = fillet.id;
        state
            .history
            .execute(Box::new(AddFeatureCmd::new(fillet)), &mut state.doc);
        state.rebuild_solid();

        let f = state.doc.find_feature(fillet_id).unwrap();
        assert_eq!(f.state, FeatureState::Active);

        // Edit parameter: change radius from 2.0 to 5.0
        let edit_cmd = ModifyFeatureCmd::new(
            fillet_id,
            FeatureKind::Fillet {
                name: "Fillet1".into(),
                radius: 5.0,
            },
        );
        state.history.execute(Box::new(edit_cmd), &mut state.doc);
        state.rebuild_solid();

        let f_edited = state.doc.find_feature(fillet_id).unwrap();
        if let FeatureKind::Fillet { radius, .. } = f_edited.kind {
            assert_eq!(radius, 5.0);
        } else {
            panic!("Expected Fillet feature kind");
        }

        // Test Suppress
        state.history.execute(
            Box::new(ToggleSuppressFeatureCmd::new(fillet_id)),
            &mut state.doc,
        );
        state.rebuild_solid();
        assert_eq!(
            state.doc.find_feature(fillet_id).unwrap().state,
            FeatureState::Suppressed
        );

        // Test Undo
        state.history.undo(&mut state.doc);
        state.rebuild_solid();
        assert_eq!(
            state.doc.find_feature(fillet_id).unwrap().state,
            FeatureState::Active
        );

        // Test Redo
        state.history.redo(&mut state.doc);
        state.rebuild_solid();
        assert_eq!(
            state.doc.find_feature(fillet_id).unwrap().state,
            FeatureState::Suppressed
        );
    }

    #[test]
    fn test_golden_parametric_regeneration_determinism_and_sensitivity() {
        use inventor_core::{Parameter, SetParameterCmd, Unit};

        let mut state = AppState::default();
        state.rebuild_solid();

        // 1. Initial parametric state has length=50, width=30, height=20
        let mesh = state.active_mesh.as_ref().expect("Active mesh must exist");
        let initial_quality = mesh.inspect_watertight();

        assert!(
            initial_quality.is_watertight,
            "Initial extruded solid must be closed and watertight"
        );
        assert!(initial_quality.is_manifold);
        assert_eq!(initial_quality.boundary_edge_count, 0);

        let initial_expected_vol = 50.0 * 30.0 * 20.0; // 30,000 mm^3
        assert!(
            (initial_quality.volume - initial_expected_vol).abs() / initial_expected_vol < 0.001,
            "Initial volume mismatch: got {}, expected {}",
            initial_quality.volume,
            initial_expected_vol
        );

        // 2. Determinism Check: Regenerate 5 times consecutively
        for iter in 0..5 {
            state.rebuild_solid();
            let q = state.active_mesh.as_ref().unwrap().inspect_watertight();
            assert_eq!(
                q.triangle_count, initial_quality.triangle_count,
                "Iteration {}: triangle count must remain deterministic",
                iter
            );
            assert_eq!(
                q.unique_vertex_count, initial_quality.unique_vertex_count,
                "Iteration {}: vertex count must remain deterministic",
                iter
            );
            assert!(
                (q.volume - initial_quality.volume).abs() < 1e-6,
                "Iteration {}: volume must remain bitwise/numerically identical",
                iter
            );
        }

        // 3. Sensitivity Check: Increase length from 50.0 to 70.0 (+20 mm)
        let cmd = SetParameterCmd::new(Parameter::new("length", 70.0, Unit::Millimeter));
        state.history.execute(Box::new(cmd), &mut state.doc);
        state.rebuild_solid();

        let updated_quality = state.active_mesh.as_ref().unwrap().inspect_watertight();
        assert!(
            updated_quality.is_watertight,
            "Regenerated solid must remain closed and watertight"
        );
        assert_eq!(updated_quality.boundary_edge_count, 0);

        let updated_expected_vol = 70.0 * 30.0 * 20.0; // 42,000 mm^3
        assert!(
            (updated_quality.volume - updated_expected_vol).abs() / updated_expected_vol < 0.001,
            "Updated volume mismatch: got {}, expected {}",
            updated_quality.volume,
            updated_expected_vol
        );

        // Verified exact delta volume: +12,000 mm^3
        let delta_vol = updated_quality.volume - initial_quality.volume;
        assert!(
            (delta_vol - 12000.0).abs() < 1.0,
            "Delta volume mismatch: got {}, expected 12000.0",
            delta_vol
        );
    }

    #[test]
    fn test_golden_step_round_trip_and_mesh_inspection() {
        use inventor_io::StepImporter;

        let kernel = inventor_kernel::TruckKernel::new();
        let solid = kernel.build_box_solid(40.0, 30.0, 20.0);

        // Perform round trip: solid -> STEP string -> imported B-rep & CadMesh
        let (step_str, result) = StepImporter::round_trip_solid(&solid, 0.05).unwrap();

        // 1. Valid STEP exchange format headers
        assert!(step_str.contains("ISO-10303-21"));
        assert!(step_str.contains("HEADER;"));
        assert!(step_str.contains("DATA;"));
        assert!(step_str.contains("END-ISO-10303-21;"));

        // 2. Topological metadata verification
        assert_eq!(result.metadata.shell_count, 1);
        assert_eq!(result.metadata.face_count, 6);
        assert_eq!(result.metadata.vertex_count, 8);
        assert_eq!(result.metadata.edge_count, 12);

        // 3. Analytical closed-mesh quality inspection
        assert!(
            result.quality.is_watertight,
            "Round-tripped STEP mesh must be watertight"
        );
        assert!(result.quality.is_manifold);
        assert_eq!(result.quality.boundary_edge_count, 0);
        assert_eq!(result.quality.non_manifold_edge_count, 0);

        // 4. Volume verification
        let expected_vol = 40.0 * 30.0 * 20.0; // 24,000 mm^3
        assert!(
            (result.quality.volume - expected_vol).abs() / expected_vol < 0.001,
            "Volume difference too large: got {}, expected {}",
            result.quality.volume,
            expected_vol
        );

        // 5. Dimensions verification
        let dims = result.quality.dimensions;
        assert!((dims[0] - 40.0).abs() < 0.1);
        assert!((dims[1] - 30.0).abs() < 0.1);
        assert!((dims[2] - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_wgpu_hardware_renderer_integration_and_viewport_state() {
        use inventor_render::{GridGenerator, WgpuCadRenderer};
        use std::sync::{Arc, Mutex};

        let mut state = AppState::default();
        assert!(state.wgpu_renderer.is_none());
        assert!(state.wgpu_render_state.is_none());
        assert!(state.viewport_texture_id.is_none());
        assert!(state.active_mesh.is_some());

        // Create headless wgpu device for testing hardware pipeline
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: None,
            apply_limit_buckets: false,
        }));

        let adapter = match adapter {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Skipping hardware wgpu test on this platform: {e}");
                return;
            }
        };

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("Headless wgpu device request");

        let target_format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let renderer = WgpuCadRenderer::new(&device, target_format);
        let renderer_arc = Arc::new(Mutex::new(renderer));
        state.wgpu_renderer = Some(renderer_arc.clone());

        // Render offscreen via WgpuCadRenderer
        let grid_lines = GridGenerator::generate_grid(60.0, 10.0);
        let (color_tex, _color_view) = {
            let mut r = renderer_arc.lock().unwrap();
            r.render_offscreen(
                &device,
                &queue,
                800,
                600,
                &state.camera,
                state.active_mesh.as_ref(),
                Some(&grid_lines),
                [26.0 / 255.0, 32.0 / 255.0, 44.0 / 255.0, 1.0],
            )
        };

        assert_eq!(color_tex.width(), 800);
        assert_eq!(color_tex.height(), 600);
        assert_eq!(color_tex.format(), target_format);

        // Verify camera updates and mesh GPU buffers
        {
            let r = renderer_arc.lock().unwrap();
            r.update_camera(&queue, &state.camera, 800.0 / 600.0);
            let mesh_gpu = r.create_mesh_buffers(
                &device,
                state.active_mesh.as_ref().unwrap(),
                [0.4, 0.5, 0.7, 1.0],
                [0.1, 0.1, 0.1, 1.0],
            );
            assert!(mesh_gpu.index_count > 0);
            assert!(mesh_gpu.edge_vertex_count > 0);
        }
    }
}
