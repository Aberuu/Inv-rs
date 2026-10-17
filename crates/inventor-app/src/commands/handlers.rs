//! Concrete command handlers registering CAD application features.

use super::error::CommandError;
use super::registry::{CommandCategory, CommandEntry, CommandRegistry};
use crate::sketch_session::SketchTool;
use crate::state::{ActiveModal, AppState};
use inventor_core::{BooleanOp, FeatureKind};
use inventor_render::ProjectionType;
use inventor_sketch::{Constraint, ConstraintKind};

/// Populates and returns the default CommandRegistry with all standard CAD actions.
pub fn build_default_registry() -> CommandRegistry {
    let mut reg = CommandRegistry::new();

    // 1. Model Primitives & Features
    reg.register(CommandEntry::new(
        "model.box",
        "Box Primitive",
        "Create a 3D box solid primitive with length, width, and height",
        CommandCategory::Model,
        handle_create_box,
    ));

    reg.register(CommandEntry::new(
        "model.cylinder",
        "Cylinder Primitive",
        "Create a 3D cylinder solid primitive with radius and height",
        CommandCategory::Model,
        handle_create_cylinder,
    ));

    reg.register(CommandEntry::new(
        "model.extrude",
        "Extrude Profile",
        "Extrude a closed 2D sketch profile into a 3D solid",
        CommandCategory::Model,
        handle_extrude,
    ));

    reg.register(CommandEntry::new(
        "model.revolve",
        "Revolve Profile",
        "Revolve a closed 2D sketch profile about an axis into a 3D solid",
        CommandCategory::Model,
        handle_revolve,
    ));

    reg.register(CommandEntry::new(
        "model.combine",
        "Combine (Boolean)",
        "Perform Union, Subtract, or Intersect boolean operation between solid bodies",
        CommandCategory::Model,
        handle_combine,
    ));

    reg.register(CommandEntry::new(
        "model.fillet",
        "Fillet Feature",
        "Create rounded blend on solid edges (evaluated in T8)",
        CommandCategory::Model,
        handle_fillet,
    ));

    reg.register(CommandEntry::new(
        "model.chamfer",
        "Chamfer Feature",
        "Create beveled edge on solid bodies (evaluated in T8)",
        CommandCategory::Model,
        handle_chamfer,
    ));

    // 2. File Operations & Export
    reg.register(
        CommandEntry::new(
            "file.new",
            "New Part",
            "Initialize a new blank parametric part document",
            CommandCategory::File,
            handle_file_new,
        )
        .with_shortcut("Ctrl+N"),
    );

    reg.register(
        CommandEntry::new(
            "file.export_step",
            "Export STEP (ISO 10303)",
            "Export active 3D solid geometry as STEP AP203/214 file",
            CommandCategory::File,
            handle_export_step,
        )
        .with_predicate(|state| state.active_mesh.is_some()),
    );

    reg.register(
        CommandEntry::new(
            "file.export_stl",
            "Export STL Mesh",
            "Export active 3D solid geometry as binary or ASCII STL mesh",
            CommandCategory::File,
            handle_export_stl,
        )
        .with_predicate(|state| state.active_mesh.is_some()),
    );

    reg.register(CommandEntry::new(
        "file.exit",
        "Exit Application",
        "Close and terminate the inventor-rs session",
        CommandCategory::File,
        handle_file_exit,
    ));

    // 3. Edit & History Stack
    reg.register(
        CommandEntry::new(
            "edit.undo",
            "Undo",
            "Revert the most recent document change",
            CommandCategory::Edit,
            handle_edit_undo,
        )
        .with_shortcut("Ctrl+Z")
        .with_predicate(|state| state.history.can_undo()),
    );

    reg.register(
        CommandEntry::new(
            "edit.redo",
            "Redo",
            "Re-apply the previously undone document change",
            CommandCategory::Edit,
            handle_edit_redo,
        )
        .with_shortcut("Ctrl+Y")
        .with_predicate(|state| state.history.can_redo()),
    );

    // 4. View & Camera Navigation
    reg.register(
        CommandEntry::new(
            "view.home",
            "Home View",
            "Orient viewport camera to standard Isometric Home view",
            CommandCategory::View,
            handle_view_home,
        )
        .with_shortcut("F6"),
    );

    reg.register(CommandEntry::new(
        "view.front",
        "Front View",
        "Orient camera to orthogonal Front view",
        CommandCategory::View,
        handle_view_front,
    ));

    reg.register(CommandEntry::new(
        "view.top",
        "Top View",
        "Orient camera to orthogonal Top view",
        CommandCategory::View,
        handle_view_top,
    ));

    reg.register(CommandEntry::new(
        "view.right",
        "Right View",
        "Orient camera to orthogonal Right view",
        CommandCategory::View,
        handle_view_right,
    ));

    reg.register(CommandEntry::new(
        "view.proj_perspective",
        "Perspective Projection",
        "Set viewport camera to realistic perspective projection",
        CommandCategory::View,
        handle_view_perspective,
    ));

    reg.register(CommandEntry::new(
        "view.proj_ortho",
        "Orthographic Projection",
        "Set viewport camera to parallel orthographic projection",
        CommandCategory::View,
        handle_view_ortho,
    ));

    reg.register(CommandEntry::new(
        "view.toggle_grid",
        "Toggle Grid",
        "Toggle ground reference plane grid visibility",
        CommandCategory::View,
        handle_view_toggle_grid,
    ));

    reg.register(CommandEntry::new(
        "view.toggle_axes",
        "Toggle Axes",
        "Toggle 3D coordinate axes display indicator",
        CommandCategory::View,
        handle_view_toggle_axes,
    ));

    // 5. Inspect & Measurement
    reg.register(
        CommandEntry::new(
            "inspect.measure",
            "Measure Bounding Box",
            "Compute bounding dimensions and triangle count of active 3D solid",
            CommandCategory::Inspect,
            handle_inspect_measure,
        )
        .with_predicate(|state| state.active_mesh.is_some()),
    );

    // 6. Tools & Parameters
    reg.register(CommandEntry::new(
        "tools.parameters",
        "Parameters (fx)",
        "Open parametric variables and dimensions table",
        CommandCategory::Tools,
        handle_tools_parameters,
    ));

    reg.register(CommandEntry::new(
        "tools.options",
        "Options",
        "Configure application and kernel backend options",
        CommandCategory::Tools,
        handle_tools_options,
    ));

    // 7. 2D Sketching Environment & Tools
    reg.register(
        CommandEntry::new(
            "sketch.new",
            "Create 2D Sketch",
            "Begin 2D sketching session on a plane or planar face",
            CommandCategory::Sketch,
            handle_sketch_new,
        )
        .with_predicate(|state| state.active_sketch.is_none()),
    );

    reg.register(
        CommandEntry::new(
            "sketch.finish",
            "Finish Sketch",
            "Exit 2D sketch mode and commit profile to 3D solid environment",
            CommandCategory::Sketch,
            handle_sketch_finish,
        )
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    reg.register(
        CommandEntry::new(
            "sketch.select",
            "Select Tool",
            "Select sketch points, lines, or curves",
            CommandCategory::Sketch,
            handle_sketch_select,
        )
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    reg.register(
        CommandEntry::new(
            "sketch.line",
            "Line Tool",
            "Draw straight line segments in active sketch plane",
            CommandCategory::Sketch,
            handle_sketch_line,
        )
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    reg.register(
        CommandEntry::new(
            "sketch.circle",
            "Circle Tool",
            "Draw a circle defined by center point and radius",
            CommandCategory::Sketch,
            handle_sketch_circle,
        )
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    reg.register(
        CommandEntry::new(
            "sketch.rectangle",
            "Rectangle Tool",
            "Draw a rectangle defined by opposite corner points",
            CommandCategory::Sketch,
            handle_sketch_rectangle,
        )
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    reg.register(
        CommandEntry::new(
            "sketch.point",
            "Point Tool",
            "Place standalone sketch points or reference vertices",
            CommandCategory::Sketch,
            handle_sketch_point,
        )
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    reg.register(
        CommandEntry::new(
            "sketch.dimension",
            "General Dimension",
            "Apply distance, length, radius, or angle dimension constraint",
            CommandCategory::Sketch,
            handle_sketch_dimension,
        )
        .with_shortcut("D")
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    // Geometric Constraints
    reg.register(
        CommandEntry::new(
            "sketch.coincident",
            "Coincident Constraint",
            "Constrain two points to share the exact same coordinates",
            CommandCategory::Sketch,
            handle_sketch_coincident,
        )
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    reg.register(
        CommandEntry::new(
            "sketch.horizontal",
            "Horizontal Constraint",
            "Constrain selected lines parallel to sketch X axis",
            CommandCategory::Sketch,
            handle_sketch_horizontal,
        )
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    reg.register(
        CommandEntry::new(
            "sketch.vertical",
            "Vertical Constraint",
            "Constrain selected lines parallel to sketch Y axis",
            CommandCategory::Sketch,
            handle_sketch_vertical,
        )
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    reg.register(
        CommandEntry::new(
            "sketch.parallel",
            "Parallel Constraint",
            "Constrain two lines to remain parallel",
            CommandCategory::Sketch,
            handle_sketch_parallel,
        )
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    reg.register(
        CommandEntry::new(
            "sketch.perpendicular",
            "Perpendicular Constraint",
            "Constrain two lines to meet at 90 degrees",
            CommandCategory::Sketch,
            handle_sketch_perpendicular,
        )
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    reg.register(
        CommandEntry::new(
            "sketch.tangent",
            "Tangent Constraint",
            "Constrain line tangent to circle or two circles tangent",
            CommandCategory::Sketch,
            handle_sketch_tangent,
        )
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    reg.register(
        CommandEntry::new(
            "sketch.equal",
            "Equal Constraint",
            "Constrain two lines to equal length or circles to equal radius",
            CommandCategory::Sketch,
            handle_sketch_equal,
        )
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    reg.register(
        CommandEntry::new(
            "sketch.fix",
            "Fix Constraint",
            "Lock point coordinates at their current position",
            CommandCategory::Sketch,
            handle_sketch_fix,
        )
        .with_predicate(|state| state.active_sketch.is_some()),
    );

    // 8. Help
    reg.register(CommandEntry::new(
        "help.about",
        "About inventor-rs",
        "Display application and kernel version info dialog",
        CommandCategory::Help,
        handle_help_about,
    ));

    reg
}

fn handle_create_box(state: &mut AppState) -> Result<(), CommandError> {
    state.active_modal = ActiveModal::CreateBox {
        width: 50.0,
        length: 30.0,
        height: 20.0,
    };
    state.status_message = "Specify box dimensions in the dialog.".into();
    Ok(())
}

fn handle_create_cylinder(state: &mut AppState) -> Result<(), CommandError> {
    state.active_modal = ActiveModal::CreateCylinder {
        radius: 15.0,
        height: 40.0,
    };
    state.status_message = "Specify cylinder dimensions in the dialog.".into();
    Ok(())
}

fn handle_extrude(state: &mut AppState) -> Result<(), CommandError> {
    // If in sketch mode, finish the sketch first so it's committed to sketches
    if state.active_sketch.is_some() {
        let _ = state.finish_active_sketch();
    }

    // Determine target sketch:
    // 1. Selected feature if it's a sketch
    // 2. Or the most recent sketch in doc
    // 3. Or any sketch in state.sketches
    let target_sketch_id = state
        .selected_feature_id
        .filter(|id| state.sketches.contains_key(id))
        .or_else(|| {
            state
                .doc
                .features
                .iter()
                .rev()
                .find(|f| matches!(f.kind, FeatureKind::Sketch { .. }))
                .map(|f| f.id)
        })
        .or_else(|| state.sketches.keys().copied().next());

    if let Some(sketch_id) = target_sketch_id {
        state.active_modal = ActiveModal::CreateExtrude {
            sketch_id: Some(sketch_id),
            distance: 25.0,
            is_cut: false,
        };
        state.status_message = "Specify extrusion parameters in the Extrude dialog.".into();
        Ok(())
    } else {
        state.status_message =
            "No sketches available. Create a 2D sketch first with 'Create 2D Sketch'.".into();
        Ok(())
    }
}

fn handle_revolve(state: &mut AppState) -> Result<(), CommandError> {
    if state.active_sketch.is_some() {
        let _ = state.finish_active_sketch();
    }

    let target_sketch_id = state
        .selected_feature_id
        .filter(|id| state.sketches.contains_key(id))
        .or_else(|| {
            state
                .doc
                .features
                .iter()
                .rev()
                .find(|f| matches!(f.kind, FeatureKind::Sketch { .. }))
                .map(|f| f.id)
        })
        .or_else(|| state.sketches.keys().copied().next());

    if let Some(sketch_id) = target_sketch_id {
        state.active_modal = ActiveModal::CreateRevolve {
            sketch_id: Some(sketch_id),
            angle_deg: 360.0,
            is_cut: false,
        };
        state.status_message = "Specify revolve parameters in the Revolve dialog.".into();
        Ok(())
    } else {
        state.status_message =
            "No sketches available. Create a 2D sketch first with 'Create 2D Sketch'.".into();
        Ok(())
    }
}

fn handle_combine(state: &mut AppState) -> Result<(), CommandError> {
    let solid_features: Vec<_> = state
        .doc
        .features
        .iter()
        .filter(|f| {
            matches!(
                f.kind,
                FeatureKind::Extrude { .. } | FeatureKind::Revolve { .. }
            )
        })
        .map(|f| f.id)
        .collect();

    let target_feature = solid_features.first().copied();
    let tool_feature = solid_features.get(1).copied();

    state.active_modal = ActiveModal::BooleanCombine {
        target_feature,
        tool_feature,
        op: BooleanOp::Union,
    };
    state.status_message = "Select solid features and boolean operation in Combine dialog.".into();
    Ok(())
}

fn handle_fillet(state: &mut AppState) -> Result<(), CommandError> {
    state.active_modal = ActiveModal::CreateFillet { radius: 2.0 };
    state.status_message = "Specify fillet radius in the Fillet dialog.".into();
    Ok(())
}

fn handle_chamfer(state: &mut AppState) -> Result<(), CommandError> {
    state.active_modal = ActiveModal::CreateChamfer { distance: 2.0 };
    state.status_message = "Specify chamfer distance in the Chamfer dialog.".into();
    Ok(())
}

fn handle_file_new(state: &mut AppState) -> Result<(), CommandError> {
    *state = AppState::default();
    state.status_message = "Created new part document.".into();
    Ok(())
}

fn handle_export_step(state: &mut AppState) -> Result<(), CommandError> {
    if state.active_mesh.is_none() {
        return Err(CommandError::Disabled(
            "file.export_step".into(),
            "No active solid model in viewport to export".into(),
        ));
    }
    state.active_modal = ActiveModal::ExportStep;
    Ok(())
}

fn handle_export_stl(state: &mut AppState) -> Result<(), CommandError> {
    if state.active_mesh.is_none() {
        return Err(CommandError::Disabled(
            "file.export_stl".into(),
            "No active solid model in viewport to export".into(),
        ));
    }
    state.active_modal = ActiveModal::ExportStl;
    Ok(())
}

fn handle_file_exit(state: &mut AppState) -> Result<(), CommandError> {
    state.status_message = "Application exit requested.".into();
    Ok(())
}

fn handle_edit_undo(state: &mut AppState) -> Result<(), CommandError> {
    if state.history.undo(&mut state.doc) {
        state.rebuild_solid();
        state.status_message = "Undo performed.".into();
        Ok(())
    } else {
        Err(CommandError::Disabled(
            "edit.undo".into(),
            "No actions remaining on the undo stack".into(),
        ))
    }
}

fn handle_edit_redo(state: &mut AppState) -> Result<(), CommandError> {
    if state.history.redo(&mut state.doc) {
        state.rebuild_solid();
        state.status_message = "Redo performed.".into();
        Ok(())
    } else {
        Err(CommandError::Disabled(
            "edit.redo".into(),
            "No actions remaining on the redo stack".into(),
        ))
    }
}

fn handle_view_home(state: &mut AppState) -> Result<(), CommandError> {
    state.camera.set_home_view();
    state.status_message = "Camera reset to standard Isometric Home view (F6).".into();
    Ok(())
}

fn handle_view_front(state: &mut AppState) -> Result<(), CommandError> {
    state.camera.set_front_view();
    state.status_message = "Camera set to Front view.".into();
    Ok(())
}

fn handle_view_top(state: &mut AppState) -> Result<(), CommandError> {
    state.camera.set_top_view();
    state.status_message = "Camera set to Top view.".into();
    Ok(())
}

fn handle_view_right(state: &mut AppState) -> Result<(), CommandError> {
    state.camera.set_right_view();
    state.status_message = "Camera set to Right view.".into();
    Ok(())
}

fn handle_view_perspective(state: &mut AppState) -> Result<(), CommandError> {
    state.camera.projection_type = ProjectionType::Perspective;
    state.status_message = "Projection set to Perspective.".into();
    Ok(())
}

fn handle_view_ortho(state: &mut AppState) -> Result<(), CommandError> {
    state.camera.projection_type = ProjectionType::Orthographic;
    state.status_message = "Projection set to Orthographic.".into();
    Ok(())
}

fn handle_view_toggle_grid(state: &mut AppState) -> Result<(), CommandError> {
    state.show_grid = !state.show_grid;
    state.status_message = format!(
        "Grid visibility: {}",
        if state.show_grid { "On" } else { "Off" }
    );
    Ok(())
}

fn handle_view_toggle_axes(state: &mut AppState) -> Result<(), CommandError> {
    state.show_axes = !state.show_axes;
    state.status_message = format!(
        "Axes visibility: {}",
        if state.show_axes { "On" } else { "Off" }
    );
    Ok(())
}

fn handle_inspect_measure(state: &mut AppState) -> Result<(), CommandError> {
    if let Some((min, max)) = state
        .active_mesh
        .as_ref()
        .and_then(|m| m.compute_bounding_box())
    {
        let dx = (max[0] - min[0]).abs();
        let dy = (max[1] - min[1]).abs();
        let dz = (max[2] - min[2]).abs();
        let triangles = state.active_mesh.as_ref().map_or(0, |m| m.triangle_count());
        state.status_message = format!(
            "Dimensions: X: {:.2} mm, Y: {:.2} mm, Z: {:.2} mm (Triangles: {})",
            dx, dy, dz, triangles
        );
        return Ok(());
    }
    Err(CommandError::Disabled(
        "inspect.measure".into(),
        "No solid model available in viewport to measure".into(),
    ))
}

fn handle_tools_parameters(state: &mut AppState) -> Result<(), CommandError> {
    state.active_modal = ActiveModal::Parameters;
    Ok(())
}

fn handle_tools_options(state: &mut AppState) -> Result<(), CommandError> {
    state.active_modal = ActiveModal::Options;
    Ok(())
}

fn handle_sketch_new(state: &mut AppState) -> Result<(), CommandError> {
    if state.active_sketch.is_some() {
        state.status_message = "Already in 2D sketch mode.".into();
        return Ok(());
    }

    let selected_plane = state.selected_feature_id.and_then(|sel_id| {
        state
            .doc
            .features
            .iter()
            .find(|f| f.id == sel_id)
            .and_then(|f| {
                if let FeatureKind::OriginPlane { name } = &f.kind {
                    Some(name.clone())
                } else {
                    None
                }
            })
    });

    let plane_name = selected_plane.unwrap_or_else(|| "XY Plane".to_string());
    state.enter_sketch_on_plane(&plane_name);
    Ok(())
}

fn handle_sketch_finish(state: &mut AppState) -> Result<(), CommandError> {
    state
        .finish_active_sketch()
        .map_err(|e| CommandError::ExecutionFailed("sketch.finish".into(), e))
}

fn handle_sketch_select(state: &mut AppState) -> Result<(), CommandError> {
    if let Some(session) = state.active_sketch.as_mut() {
        session.active_tool = SketchTool::Select;
        session.pending_start = None;
        state.status_message = "Select tool active. Click sketch entities to select.".into();
    }
    Ok(())
}

fn handle_sketch_line(state: &mut AppState) -> Result<(), CommandError> {
    if let Some(session) = state.active_sketch.as_mut() {
        session.active_tool = SketchTool::Line;
        session.pending_start = None;
        state.status_message = "Line tool active. Click start point on sketch plane.".into();
    }
    Ok(())
}

fn handle_sketch_circle(state: &mut AppState) -> Result<(), CommandError> {
    if let Some(session) = state.active_sketch.as_mut() {
        session.active_tool = SketchTool::Circle;
        session.pending_start = None;
        state.status_message = "Circle tool active. Click center point on sketch plane.".into();
    }
    Ok(())
}

fn handle_sketch_rectangle(state: &mut AppState) -> Result<(), CommandError> {
    if let Some(session) = state.active_sketch.as_mut() {
        session.active_tool = SketchTool::Rectangle;
        session.pending_start = None;
        state.status_message =
            "Rectangle tool active. Click corner 1 then opposite corner 2.".into();
    }
    Ok(())
}

fn handle_sketch_point(state: &mut AppState) -> Result<(), CommandError> {
    if let Some(session) = state.active_sketch.as_mut() {
        session.active_tool = SketchTool::Point;
        session.pending_start = None;
        state.status_message = "Point tool active. Click to place sketch vertices.".into();
    }
    Ok(())
}

fn handle_sketch_dimension(state: &mut AppState) -> Result<(), CommandError> {
    let Some(session) = state.active_sketch.as_ref() else {
        return Err(CommandError::Disabled(
            "sketch.dimension".into(),
            "No active sketch".into(),
        ));
    };

    if session.selected_lines.len() == 1 {
        let line_id = session.selected_lines[0];
        if let Some(line) = session.sketch.find_line(line_id)
            && let (Some(p1), Some(p2)) = (
                session.sketch.find_point(line.start),
                session.sketch.find_point(line.end),
            )
        {
            let len = (p2.x - p1.x).hypot(p2.y - p1.y);
            state.active_modal = ActiveModal::DimensionInput {
                title: "Line Length (mm)".into(),
                is_circle: false,
                is_angle: false,
                current_value: len,
            };
            return Ok(());
        }
    }

    if session.selected_points.len() == 2 {
        let p1_id = session.selected_points[0];
        let p2_id = session.selected_points[1];
        if let (Some(p1), Some(p2)) = (
            session.sketch.find_point(p1_id),
            session.sketch.find_point(p2_id),
        ) {
            let dist = (p2.x - p1.x).hypot(p2.y - p1.y);
            state.active_modal = ActiveModal::DimensionInput {
                title: "Distance (mm)".into(),
                is_circle: false,
                is_angle: false,
                current_value: dist,
            };
            return Ok(());
        }
    }

    if session.selected_circles.len() == 1 {
        let circle_id = session.selected_circles[0];
        if let Some(c) = session.sketch.find_circle(circle_id) {
            state.active_modal = ActiveModal::DimensionInput {
                title: "Diameter (mm)".into(),
                is_circle: true,
                is_angle: false,
                current_value: c.radius * 2.0,
            };
            return Ok(());
        }
    }

    if let Some(session_mut) = state.active_sketch.as_mut() {
        session_mut.active_tool = SketchTool::Dimension;
    }
    state.status_message =
        "Dimension tool active. Select a line, two points, or a circle to add dimension.".into();
    Ok(())
}

fn handle_sketch_horizontal(state: &mut AppState) -> Result<(), CommandError> {
    let Some(session) = state.active_sketch.as_mut() else {
        return Err(CommandError::Disabled(
            "sketch.horizontal".into(),
            "No active sketch".into(),
        ));
    };

    if session.selected_lines.is_empty() {
        state.status_message =
            "Select one or more lines first to apply Horizontal constraint.".into();
        return Ok(());
    }

    for line_id in session.selected_lines.clone() {
        let c_name = format!("H{}", session.constraints.len() + 1);
        let _ = session.add_constraint(Constraint::new(
            c_name,
            ConstraintKind::Horizontal { line: line_id },
        ));
    }
    state.status_message = "Horizontal constraint applied.".into();
    Ok(())
}

fn handle_sketch_vertical(state: &mut AppState) -> Result<(), CommandError> {
    let Some(session) = state.active_sketch.as_mut() else {
        return Err(CommandError::Disabled(
            "sketch.vertical".into(),
            "No active sketch".into(),
        ));
    };

    if session.selected_lines.is_empty() {
        state.status_message =
            "Select one or more lines first to apply Vertical constraint.".into();
        return Ok(());
    }

    for line_id in session.selected_lines.clone() {
        let c_name = format!("V{}", session.constraints.len() + 1);
        let _ = session.add_constraint(Constraint::new(
            c_name,
            ConstraintKind::Vertical { line: line_id },
        ));
    }
    state.status_message = "Vertical constraint applied.".into();
    Ok(())
}

fn handle_sketch_coincident(state: &mut AppState) -> Result<(), CommandError> {
    let Some(session) = state.active_sketch.as_mut() else {
        return Err(CommandError::Disabled(
            "sketch.coincident".into(),
            "No active sketch".into(),
        ));
    };

    if session.selected_points.len() < 2 {
        state.status_message = "Select two points to apply Coincident constraint.".into();
        return Ok(());
    }

    let p1 = session.selected_points[0];
    let p2 = session.selected_points[1];
    let c_name = format!("Coin{}", session.constraints.len() + 1);
    let _ = session.add_constraint(Constraint::new(
        c_name,
        ConstraintKind::Coincident {
            point1: p1,
            point2: p2,
        },
    ));
    state.status_message = "Coincident constraint applied.".into();
    Ok(())
}

fn handle_sketch_parallel(state: &mut AppState) -> Result<(), CommandError> {
    let Some(session) = state.active_sketch.as_mut() else {
        return Err(CommandError::Disabled(
            "sketch.parallel".into(),
            "No active sketch".into(),
        ));
    };

    if session.selected_lines.len() < 2 {
        state.status_message = "Select two lines to apply Parallel constraint.".into();
        return Ok(());
    }

    let l1 = session.selected_lines[0];
    let l2 = session.selected_lines[1];
    let c_name = format!("Para{}", session.constraints.len() + 1);
    let _ = session.add_constraint(Constraint::new(
        c_name,
        ConstraintKind::Parallel {
            line1: l1,
            line2: l2,
        },
    ));
    state.status_message = "Parallel constraint applied.".into();
    Ok(())
}

fn handle_sketch_perpendicular(state: &mut AppState) -> Result<(), CommandError> {
    let Some(session) = state.active_sketch.as_mut() else {
        return Err(CommandError::Disabled(
            "sketch.perpendicular".into(),
            "No active sketch".into(),
        ));
    };

    if session.selected_lines.len() < 2 {
        state.status_message = "Select two lines to apply Perpendicular constraint.".into();
        return Ok(());
    }

    let l1 = session.selected_lines[0];
    let l2 = session.selected_lines[1];
    let c_name = format!("Perp{}", session.constraints.len() + 1);
    let _ = session.add_constraint(Constraint::new(
        c_name,
        ConstraintKind::Perpendicular {
            line1: l1,
            line2: l2,
        },
    ));
    state.status_message = "Perpendicular constraint applied.".into();
    Ok(())
}

fn handle_sketch_tangent(state: &mut AppState) -> Result<(), CommandError> {
    let Some(session) = state.active_sketch.as_mut() else {
        return Err(CommandError::Disabled(
            "sketch.tangent".into(),
            "No active sketch".into(),
        ));
    };

    let pair = if session.selected_lines.len() == 1 && session.selected_circles.len() == 1 {
        Some((session.selected_lines[0], session.selected_circles[0]))
    } else if session.selected_circles.len() >= 2 {
        Some((session.selected_circles[0], session.selected_circles[1]))
    } else {
        None
    };

    if let Some((e1, e2)) = pair {
        let c_name = format!("Tan{}", session.constraints.len() + 1);
        let _ = session.add_constraint(Constraint::new(
            c_name,
            ConstraintKind::Tangent {
                entity1: e1,
                entity2: e2,
            },
        ));
        state.status_message = "Tangent constraint applied.".into();
    } else {
        state.status_message =
            "Select a line and a circle (or two circles) to apply Tangent constraint.".into();
    }
    Ok(())
}

fn handle_sketch_equal(state: &mut AppState) -> Result<(), CommandError> {
    let Some(session) = state.active_sketch.as_mut() else {
        return Err(CommandError::Disabled(
            "sketch.equal".into(),
            "No active sketch".into(),
        ));
    };

    let pair = if session.selected_lines.len() >= 2 {
        Some((session.selected_lines[0], session.selected_lines[1]))
    } else if session.selected_circles.len() >= 2 {
        Some((session.selected_circles[0], session.selected_circles[1]))
    } else {
        None
    };

    if let Some((e1, e2)) = pair {
        let c_name = format!("Eq{}", session.constraints.len() + 1);
        let _ = session.add_constraint(Constraint::new(
            c_name,
            ConstraintKind::Equal {
                entity1: e1,
                entity2: e2,
            },
        ));
        state.status_message = "Equal constraint applied.".into();
    } else {
        state.status_message = "Select two lines or two circles to apply Equal constraint.".into();
    }
    Ok(())
}

fn handle_sketch_fix(state: &mut AppState) -> Result<(), CommandError> {
    let Some(session) = state.active_sketch.as_mut() else {
        return Err(CommandError::Disabled(
            "sketch.fix".into(),
            "No active sketch".into(),
        ));
    };

    if session.selected_points.is_empty() {
        state.status_message = "Select one or more points to fix position.".into();
        return Ok(());
    }

    for pt_id in session.selected_points.clone() {
        if let Some(pt) = session.sketch.find_point(pt_id) {
            let (x, y) = (pt.x, pt.y);
            let c_name = format!("Fix{}", session.constraints.len() + 1);
            let _ = session.add_constraint(Constraint::new(
                c_name,
                ConstraintKind::FixPoint { point: pt_id, x, y },
            ));
        }
    }
    state.status_message = "Fix constraint applied to selected point(s).".into();
    Ok(())
}

fn handle_help_about(state: &mut AppState) -> Result<(), CommandError> {
    state.active_modal = ActiveModal::About;
    Ok(())
}
