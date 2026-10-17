use glam::{Vec2, Vec3};
use inventor_sketch::{Constraint, ConstraintKind, ConstraintSolver, ConstraintStatus, Sketch2D};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SketchTool {
    #[default]
    Select,
    Line,
    Rectangle,
    Circle,
    Point,
    Dimension,
}

#[derive(Debug, Clone)]
pub struct StoredSketch {
    pub sketch: Sketch2D,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub struct ActiveSketchSession {
    pub id: Uuid,
    pub name: String,
    pub plane_id: Uuid,
    pub plane_name: String,
    pub plane_origin: Vec3,
    pub plane_x_axis: Vec3,
    pub plane_y_axis: Vec3,
    pub plane_normal: Vec3,
    pub sketch: Sketch2D,
    pub constraints: Vec<Constraint>,
    pub current_dof: ConstraintStatus,
    pub active_tool: SketchTool,
    pub pending_start: Option<Vec2>,
    pub selected_points: Vec<Uuid>,
    pub selected_lines: Vec<Uuid>,
    pub selected_circles: Vec<Uuid>,
    pub solver_error: Option<String>,
}

impl ActiveSketchSession {
    pub fn new(
        name: impl Into<String>,
        plane_id: Uuid,
        plane_name: impl Into<String>,
        plane_origin: Vec3,
        plane_x_axis: Vec3,
        plane_y_axis: Vec3,
        plane_normal: Vec3,
    ) -> Self {
        let name_str = name.into();
        let sketch = Sketch2D::new(&name_str, plane_id);
        let id = sketch.id;

        let mut session = Self {
            id,
            name: name_str,
            plane_id,
            plane_name: plane_name.into(),
            plane_origin,
            plane_x_axis: plane_x_axis.normalize(),
            plane_y_axis: plane_y_axis.normalize(),
            plane_normal: plane_normal.normalize(),
            sketch,
            constraints: Vec::new(),
            current_dof: ConstraintStatus::FullyConstrained,
            active_tool: SketchTool::Select,
            pending_start: None,
            selected_points: Vec::new(),
            selected_lines: Vec::new(),
            selected_circles: Vec::new(),
            solver_error: None,
        };

        session.recalculate_dof();
        session
    }

    /// Creates an active session from an existing stored sketch.
    pub fn from_stored(
        stored: StoredSketch,
        plane_name: impl Into<String>,
        plane_origin: Vec3,
        plane_x_axis: Vec3,
        plane_y_axis: Vec3,
        plane_normal: Vec3,
    ) -> Self {
        let mut session = Self {
            id: stored.sketch.id,
            name: stored.sketch.name.clone(),
            plane_id: stored.sketch.plane_id,
            plane_name: plane_name.into(),
            plane_origin,
            plane_x_axis: plane_x_axis.normalize(),
            plane_y_axis: plane_y_axis.normalize(),
            plane_normal: plane_normal.normalize(),
            sketch: stored.sketch,
            constraints: stored.constraints,
            current_dof: ConstraintStatus::FullyConstrained,
            active_tool: SketchTool::Select,
            pending_start: None,
            selected_points: Vec::new(),
            selected_lines: Vec::new(),
            selected_circles: Vec::new(),
            solver_error: None,
        };

        session.recalculate_dof();
        session
    }

    /// Transforms 2D sketch plane coordinates (u, v) to 3D world space.
    pub fn plane_to_world(&self, p: Vec2) -> Vec3 {
        self.plane_origin + self.plane_x_axis * p.x + self.plane_y_axis * p.y
    }

    /// Transforms a 3D point in world space to 2D sketch plane coordinates (u, v).
    pub fn world_to_plane(&self, w: Vec3) -> Vec2 {
        let rel = w - self.plane_origin;
        Vec2::new(rel.dot(self.plane_x_axis), rel.dot(self.plane_y_axis))
    }

    /// Recalculates degrees of freedom using the Jacobian SVD rank solver.
    pub fn recalculate_dof(&mut self) {
        let solver = ConstraintSolver::new();
        self.current_dof = solver.calculate_dof(&self.sketch, &self.constraints);
    }

    /// Solves geometric constraints and updates entity positions.
    pub fn solve_constraints(&mut self) -> Result<ConstraintStatus, String> {
        let solver = ConstraintSolver::new();
        match solver.solve(&mut self.sketch, &self.constraints) {
            Ok(status) => {
                self.current_dof = status;
                self.solver_error = None;
                Ok(status)
            }
            Err(e) => {
                let err_msg = e.to_string();
                self.solver_error = Some(err_msg.clone());
                self.current_dof = ConstraintStatus::Overconstrained;
                Err(err_msg)
            }
        }
    }

    /// Adds a 2D point to the sketch.
    pub fn add_point(&mut self, u: f64, v: f64) -> Uuid {
        let id = self.sketch.add_point(u, v);
        self.recalculate_dof();
        id
    }

    /// Adds a straight line between two point IDs.
    pub fn add_line(&mut self, start: Uuid, end: Uuid) -> Uuid {
        let id = self.sketch.add_line(start, end);
        self.recalculate_dof();
        id
    }

    /// Adds a rectangle from two corner coordinates in sketch space.
    /// Creates 4 points, 4 lines, and horizontal/vertical constraints.
    pub fn add_rectangle(&mut self, p1: Vec2, p2: Vec2) -> (Vec<Uuid>, Vec<Uuid>) {
        let x_min = p1.x.min(p2.x) as f64;
        let x_max = p1.x.max(p2.x) as f64;
        let y_min = p1.y.min(p2.y) as f64;
        let y_max = p1.y.max(p2.y) as f64;

        let pt0 = self.add_point(x_min, y_min);
        let pt1 = self.add_point(x_max, y_min);
        let pt2 = self.add_point(x_max, y_max);
        let pt3 = self.add_point(x_min, y_max);

        let l_bottom = self.add_line(pt0, pt1);
        let l_right = self.add_line(pt1, pt2);
        let l_top = self.add_line(pt3, pt2);
        let l_left = self.add_line(pt0, pt3);

        // Add structural constraints for the rectangle
        let c_bottom_h = Constraint::new(
            "Rect_H_Bottom",
            ConstraintKind::Horizontal { line: l_bottom },
        );
        let c_top_h = Constraint::new("Rect_H_Top", ConstraintKind::Horizontal { line: l_top });
        let c_left_v = Constraint::new("Rect_V_Left", ConstraintKind::Vertical { line: l_left });
        let c_right_v = Constraint::new("Rect_V_Right", ConstraintKind::Vertical { line: l_right });

        self.constraints.push(c_bottom_h);
        self.constraints.push(c_top_h);
        self.constraints.push(c_left_v);
        self.constraints.push(c_right_v);

        let _ = self.solve_constraints();
        (
            vec![pt0, pt1, pt2, pt3],
            vec![l_bottom, l_right, l_top, l_left],
        )
    }

    /// Adds a circle from center coordinates and radius.
    pub fn add_circle(&mut self, center: Vec2, radius: f64) -> (Uuid, Uuid) {
        let center_id = self.add_point(center.x as f64, center.y as f64);
        let circle_id = self.sketch.add_circle(center_id, radius);
        self.recalculate_dof();
        (center_id, circle_id)
    }

    /// Adds a constraint and immediately runs the solver.
    pub fn add_constraint(&mut self, constraint: Constraint) -> Result<ConstraintStatus, String> {
        self.constraints.push(constraint);
        self.solve_constraints()
    }

    /// Removes a constraint by ID and recalculates DOF.
    pub fn remove_constraint(&mut self, constraint_id: Uuid) {
        self.constraints.retain(|c| c.id != constraint_id);
        let _ = self.solve_constraints();
    }

    /// Clears active selection.
    pub fn clear_selection(&mut self) {
        self.selected_points.clear();
        self.selected_lines.clear();
        self.selected_circles.clear();
    }

    /// Converts active session into a StoredSketch for persistence.
    pub fn to_stored(&self) -> StoredSketch {
        StoredSketch {
            sketch: self.sketch.clone(),
            constraints: self.constraints.clone(),
        }
    }
}
