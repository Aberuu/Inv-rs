use crate::constraint::{Constraint, ConstraintKind};
use crate::entity::Sketch2D;
use nalgebra::{DMatrix, DVector};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, PartialEq)]
pub enum SolverError {
    #[error("Overconstrained system: conflicting or redundant constraints detected")]
    Overconstrained,
    #[error("Solver failed to converge after {0} iterations (max residual: {1:.6e})")]
    ConvergenceFailed(usize, f64),
    #[error("Invalid constraint reference: entity not found")]
    EntityNotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintStatus {
    Underconstrained(usize),
    FullyConstrained,
    Overconstrained,
}

pub struct ConstraintSolver {
    pub max_iterations: usize,
    pub tolerance: f64,
    pub damping: f64,
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            tolerance: 1e-6,
            damping: 1e-4,
        }
    }
}

struct VariableIndex {
    point_map: HashMap<Uuid, usize>,
    circle_map: HashMap<Uuid, usize>,
    num_points: usize,
    _num_circles: usize,
    total_vars: usize,
}

impl VariableIndex {
    fn new(sketch: &Sketch2D) -> Self {
        let mut point_map = HashMap::new();
        for (i, p) in sketch.points.iter().enumerate() {
            point_map.insert(p.id, i);
        }
        let mut circle_map = HashMap::new();
        for (k, c) in sketch.circles.iter().enumerate() {
            circle_map.insert(c.id, k);
        }
        let num_points = sketch.points.len();
        let num_circles = sketch.circles.len();
        let total_vars = num_points * 2 + num_circles;
        Self {
            point_map,
            circle_map,
            num_points,
            _num_circles: num_circles,
            total_vars,
        }
    }

    fn to_vector(&self, sketch: &Sketch2D) -> DVector<f64> {
        let mut v = DVector::zeros(self.total_vars);
        for (i, p) in sketch.points.iter().enumerate() {
            v[2 * i] = p.x;
            v[2 * i + 1] = p.y;
        }
        for (k, c) in sketch.circles.iter().enumerate() {
            v[2 * self.num_points + k] = c.radius;
        }
        v
    }

    fn apply_to_sketch(&self, v: &DVector<f64>, sketch: &mut Sketch2D) {
        for (i, p) in sketch.points.iter_mut().enumerate() {
            p.x = v[2 * i];
            p.y = v[2 * i + 1];
        }
        for (k, c) in sketch.circles.iter_mut().enumerate() {
            c.radius = v[2 * self.num_points + k];
        }
    }

    fn point_indices(&self, pt_id: Uuid) -> Option<(usize, usize)> {
        let idx = *self.point_map.get(&pt_id)?;
        Some((2 * idx, 2 * idx + 1))
    }

    fn circle_radius_idx(&self, circle_id: Uuid) -> Option<usize> {
        let idx = *self.circle_map.get(&circle_id)?;
        Some(2 * self.num_points + idx)
    }
}

struct Equation {
    residual: f64,
    gradients: Vec<(usize, f64)>,
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates all constraint equations and fixed points into residuals and Jacobian matrix.
    fn evaluate_equations(
        &self,
        x: &DVector<f64>,
        sketch: &Sketch2D,
        constraints: &[Constraint],
        index: &VariableIndex,
    ) -> Result<Vec<Equation>, SolverError> {
        let mut equations = Vec::new();

        // 1. Fixed points from point.is_fixed
        let explicitly_fixed: std::collections::HashSet<Uuid> = constraints
            .iter()
            .filter_map(|c| match c.kind {
                ConstraintKind::FixPoint { point, .. } => Some(point),
                _ => None,
            })
            .collect();

        for p in &sketch.points {
            if p.is_fixed
                && !explicitly_fixed.contains(&p.id)
                && let Some((xi, yi)) = index.point_indices(p.id)
            {
                equations.push(Equation {
                    residual: x[xi] - p.x,
                    gradients: vec![(xi, 1.0)],
                });
                equations.push(Equation {
                    residual: x[yi] - p.y,
                    gradients: vec![(yi, 1.0)],
                });
            }
        }

        // 2. Constraints from input
        for c in constraints {
            match c.kind {
                ConstraintKind::Coincident { point1, point2 } => {
                    let (x1_i, y1_i) = index
                        .point_indices(point1)
                        .ok_or(SolverError::EntityNotFound)?;
                    let (x2_i, y2_i) = index
                        .point_indices(point2)
                        .ok_or(SolverError::EntityNotFound)?;

                    equations.push(Equation {
                        residual: x[x1_i] - x[x2_i],
                        gradients: vec![(x1_i, 1.0), (x2_i, -1.0)],
                    });
                    equations.push(Equation {
                        residual: x[y1_i] - x[y2_i],
                        gradients: vec![(y1_i, 1.0), (y2_i, -1.0)],
                    });
                }
                ConstraintKind::FixPoint {
                    point,
                    x: tx,
                    y: ty,
                } => {
                    let (xi, yi) = index
                        .point_indices(point)
                        .ok_or(SolverError::EntityNotFound)?;

                    equations.push(Equation {
                        residual: x[xi] - tx,
                        gradients: vec![(xi, 1.0)],
                    });
                    equations.push(Equation {
                        residual: x[yi] - ty,
                        gradients: vec![(yi, 1.0)],
                    });
                }
                ConstraintKind::Horizontal { line } => {
                    let line_entity = sketch.find_line(line).ok_or(SolverError::EntityNotFound)?;
                    let (_, y1_i) = index
                        .point_indices(line_entity.start)
                        .ok_or(SolverError::EntityNotFound)?;
                    let (_, y2_i) = index
                        .point_indices(line_entity.end)
                        .ok_or(SolverError::EntityNotFound)?;

                    equations.push(Equation {
                        residual: x[y2_i] - x[y1_i],
                        gradients: vec![(y2_i, 1.0), (y1_i, -1.0)],
                    });
                }
                ConstraintKind::Vertical { line } => {
                    let line_entity = sketch.find_line(line).ok_or(SolverError::EntityNotFound)?;
                    let (x1_i, _) = index
                        .point_indices(line_entity.start)
                        .ok_or(SolverError::EntityNotFound)?;
                    let (x2_i, _) = index
                        .point_indices(line_entity.end)
                        .ok_or(SolverError::EntityNotFound)?;

                    equations.push(Equation {
                        residual: x[x2_i] - x[x1_i],
                        gradients: vec![(x2_i, 1.0), (x1_i, -1.0)],
                    });
                }
                ConstraintKind::Distance {
                    point1,
                    point2,
                    distance,
                } => {
                    let (x1_i, y1_i) = index
                        .point_indices(point1)
                        .ok_or(SolverError::EntityNotFound)?;
                    let (x2_i, y2_i) = index
                        .point_indices(point2)
                        .ok_or(SolverError::EntityNotFound)?;

                    let dx = x[x2_i] - x[x1_i];
                    let dy = x[y2_i] - x[y1_i];
                    let dist = dx.hypot(dy);
                    let safe_dist = dist.max(1e-12);

                    equations.push(Equation {
                        residual: dist - distance,
                        gradients: vec![
                            (x1_i, -dx / safe_dist),
                            (y1_i, -dy / safe_dist),
                            (x2_i, dx / safe_dist),
                            (y2_i, dy / safe_dist),
                        ],
                    });
                }
                ConstraintKind::Radius { circle, radius } => {
                    let ri = index
                        .circle_radius_idx(circle)
                        .ok_or(SolverError::EntityNotFound)?;
                    equations.push(Equation {
                        residual: x[ri] - radius,
                        gradients: vec![(ri, 1.0)],
                    });
                }
                ConstraintKind::Diameter { circle, diameter } => {
                    let ri = index
                        .circle_radius_idx(circle)
                        .ok_or(SolverError::EntityNotFound)?;
                    equations.push(Equation {
                        residual: 2.0 * x[ri] - diameter,
                        gradients: vec![(ri, 2.0)],
                    });
                }
                ConstraintKind::Parallel { line1, line2 } => {
                    let l1 = sketch.find_line(line1).ok_or(SolverError::EntityNotFound)?;
                    let l2 = sketch.find_line(line2).ok_or(SolverError::EntityNotFound)?;
                    let (x1a, y1a) = index
                        .point_indices(l1.start)
                        .ok_or(SolverError::EntityNotFound)?;
                    let (x1b, y1b) = index
                        .point_indices(l1.end)
                        .ok_or(SolverError::EntityNotFound)?;
                    let (x2a, y2a) = index
                        .point_indices(l2.start)
                        .ok_or(SolverError::EntityNotFound)?;
                    let (x2b, y2b) = index
                        .point_indices(l2.end)
                        .ok_or(SolverError::EntityNotFound)?;

                    let u_x = x[x1b] - x[x1a];
                    let u_y = x[y1b] - x[y1a];
                    let v_x = x[x2b] - x[x2a];
                    let v_y = x[y2b] - x[y2a];

                    let len1 = u_x.hypot(u_y).max(1e-9);
                    let len2 = v_x.hypot(v_y).max(1e-9);
                    let denom = (len1 * len2).max(1e-12);
                    let cross = u_x * v_y - u_y * v_x;
                    let res = cross / denom;

                    let mut grads = Vec::with_capacity(8);
                    let mut add_w = |idx: usize, dc: f64, dl1: f64, dl2: f64| {
                        let d_denom = dl1 * len2 + len1 * dl2;
                        let d_res = (dc * denom - cross * d_denom) / (denom * denom);
                        grads.push((idx, d_res));
                    };

                    add_w(x1a, -v_y, -u_x / len1, 0.0);
                    add_w(x1b, v_y, u_x / len1, 0.0);
                    add_w(y1a, v_x, -u_y / len1, 0.0);
                    add_w(y1b, -v_x, u_y / len1, 0.0);

                    add_w(x2a, u_y, 0.0, -v_x / len2);
                    add_w(x2b, -u_y, 0.0, v_x / len2);
                    add_w(y2a, -u_x, 0.0, -v_y / len2);
                    add_w(y2b, u_x, 0.0, v_y / len2);

                    equations.push(Equation {
                        residual: res,
                        gradients: grads,
                    });
                }
                ConstraintKind::Perpendicular { line1, line2 } => {
                    let l1 = sketch.find_line(line1).ok_or(SolverError::EntityNotFound)?;
                    let l2 = sketch.find_line(line2).ok_or(SolverError::EntityNotFound)?;
                    let (x1a, y1a) = index
                        .point_indices(l1.start)
                        .ok_or(SolverError::EntityNotFound)?;
                    let (x1b, y1b) = index
                        .point_indices(l1.end)
                        .ok_or(SolverError::EntityNotFound)?;
                    let (x2a, y2a) = index
                        .point_indices(l2.start)
                        .ok_or(SolverError::EntityNotFound)?;
                    let (x2b, y2b) = index
                        .point_indices(l2.end)
                        .ok_or(SolverError::EntityNotFound)?;

                    let u_x = x[x1b] - x[x1a];
                    let u_y = x[y1b] - x[y1a];
                    let v_x = x[x2b] - x[x2a];
                    let v_y = x[y2b] - x[y2a];

                    let len1 = u_x.hypot(u_y).max(1e-9);
                    let len2 = v_x.hypot(v_y).max(1e-9);
                    let denom = (len1 * len2).max(1e-12);
                    let dot = u_x * v_x + u_y * v_y;
                    let res = dot / denom;

                    let mut grads = Vec::with_capacity(8);
                    let mut add_w = |idx: usize, d_dot: f64, dl1: f64, dl2: f64| {
                        let d_denom = dl1 * len2 + len1 * dl2;
                        let d_res = (d_dot * denom - dot * d_denom) / (denom * denom);
                        grads.push((idx, d_res));
                    };

                    add_w(x1a, -v_x, -u_x / len1, 0.0);
                    add_w(x1b, v_x, u_x / len1, 0.0);
                    add_w(y1a, -v_y, -u_y / len1, 0.0);
                    add_w(y1b, v_y, u_y / len1, 0.0);

                    add_w(x2a, -u_x, 0.0, -v_x / len2);
                    add_w(x2b, u_x, 0.0, v_x / len2);
                    add_w(y2a, -u_y, 0.0, -v_y / len2);
                    add_w(y2b, u_y, 0.0, v_y / len2);

                    equations.push(Equation {
                        residual: res,
                        gradients: grads,
                    });
                }
                ConstraintKind::Equal { entity1, entity2 } => {
                    // Check if both are lines
                    if let (Some(l1), Some(l2)) =
                        (sketch.find_line(entity1), sketch.find_line(entity2))
                    {
                        let (x1a, y1a) = index
                            .point_indices(l1.start)
                            .ok_or(SolverError::EntityNotFound)?;
                        let (x1b, y1b) = index
                            .point_indices(l1.end)
                            .ok_or(SolverError::EntityNotFound)?;
                        let (x2a, y2a) = index
                            .point_indices(l2.start)
                            .ok_or(SolverError::EntityNotFound)?;
                        let (x2b, y2b) = index
                            .point_indices(l2.end)
                            .ok_or(SolverError::EntityNotFound)?;

                        let u_x = x[x1b] - x[x1a];
                        let u_y = x[y1b] - x[y1a];
                        let v_x = x[x2b] - x[x2a];
                        let v_y = x[y2b] - x[y2a];

                        let len1 = u_x.hypot(u_y);
                        let len2 = v_x.hypot(v_y);
                        let s1 = len1.max(1e-12);
                        let s2 = len2.max(1e-12);

                        equations.push(Equation {
                            residual: len1 - len2,
                            gradients: vec![
                                (x1a, -u_x / s1),
                                (y1a, -u_y / s1),
                                (x1b, u_x / s1),
                                (y1b, u_y / s1),
                                (x2a, v_x / s2),
                                (y2a, v_y / s2),
                                (x2b, -v_x / s2),
                                (y2b, -v_y / s2),
                            ],
                        });
                    } else if let (Some(_c1), Some(_c2)) =
                        (sketch.find_circle(entity1), sketch.find_circle(entity2))
                    {
                        let r1_i = index
                            .circle_radius_idx(entity1)
                            .ok_or(SolverError::EntityNotFound)?;
                        let r2_i = index
                            .circle_radius_idx(entity2)
                            .ok_or(SolverError::EntityNotFound)?;
                        equations.push(Equation {
                            residual: x[r1_i] - x[r2_i],
                            gradients: vec![(r1_i, 1.0), (r2_i, -1.0)],
                        });
                    } else {
                        return Err(SolverError::EntityNotFound);
                    }
                }
                ConstraintKind::Angle {
                    line1,
                    line2,
                    angle_deg,
                } => {
                    let l1 = sketch.find_line(line1).ok_or(SolverError::EntityNotFound)?;
                    let l2 = sketch.find_line(line2).ok_or(SolverError::EntityNotFound)?;
                    let (x1a, y1a) = index
                        .point_indices(l1.start)
                        .ok_or(SolverError::EntityNotFound)?;
                    let (x1b, y1b) = index
                        .point_indices(l1.end)
                        .ok_or(SolverError::EntityNotFound)?;
                    let (x2a, y2a) = index
                        .point_indices(l2.start)
                        .ok_or(SolverError::EntityNotFound)?;
                    let (x2b, y2b) = index
                        .point_indices(l2.end)
                        .ok_or(SolverError::EntityNotFound)?;

                    let u_x = x[x1b] - x[x1a];
                    let u_y = x[y1b] - x[y1a];
                    let v_x = x[x2b] - x[x2a];
                    let v_y = x[y2b] - x[y2a];

                    let len1_sq = (u_x * u_x + u_y * u_y).max(1e-12);
                    let len2_sq = (v_x * v_x + v_y * v_y).max(1e-12);

                    let alpha = u_y.atan2(u_x);
                    let beta = v_y.atan2(v_x);
                    let target_rad = angle_deg.to_radians();

                    let diff = (beta - alpha) - target_rad;
                    let pi = std::f64::consts::PI;
                    let wrapped = (diff + pi).rem_euclid(2.0 * pi) - pi;

                    equations.push(Equation {
                        residual: wrapped,
                        gradients: vec![
                            (x1a, -u_y / len1_sq),
                            (y1a, u_x / len1_sq),
                            (x1b, u_y / len1_sq),
                            (y1b, -u_x / len1_sq),
                            (x2a, v_y / len2_sq),
                            (y2a, -v_x / len2_sq),
                            (x2b, -v_y / len2_sq),
                            (y2b, v_x / len2_sq),
                        ],
                    });
                }
                ConstraintKind::Tangent { entity1, entity2 } => {
                    // Check Line & Circle or Circle & Line
                    let (line_opt, circle_opt) = if let Some(l) = sketch.find_line(entity1) {
                        (Some(l), sketch.find_circle(entity2))
                    } else if let Some(l) = sketch.find_line(entity2) {
                        (Some(l), sketch.find_circle(entity1))
                    } else {
                        (None, None)
                    };

                    if let (Some(l), Some(c)) = (line_opt, circle_opt) {
                        let (x1_i, y1_i) = index
                            .point_indices(l.start)
                            .ok_or(SolverError::EntityNotFound)?;
                        let (x2_i, y2_i) = index
                            .point_indices(l.end)
                            .ok_or(SolverError::EntityNotFound)?;
                        let (xc_i, yc_i) = index
                            .point_indices(c.center)
                            .ok_or(SolverError::EntityNotFound)?;
                        let r_i = index
                            .circle_radius_idx(c.id)
                            .ok_or(SolverError::EntityNotFound)?;

                        let x1 = x[x1_i];
                        let y1 = x[y1_i];
                        let x2 = x[x2_i];
                        let y2 = x[y2_i];
                        let xc = x[xc_i];
                        let yc = x[yc_i];
                        let r = x[r_i];

                        let u_x = x2 - x1;
                        let u_y = y2 - y1;
                        let len = u_x.hypot(u_y).max(1e-9);

                        // Signed distance numerator: (x1 - xc)*u_y - (y1 - yc)*u_x
                        let num = (x1 - xc) * u_y - (y1 - yc) * u_x;
                        let dist = num / len;
                        let side = if dist >= 0.0 { 1.0 } else { -1.0 };
                        let res = dist - side * r;

                        let dn_xc = -u_y;
                        let dn_yc = u_x;
                        let dn_x1 = yc - y2;
                        let dn_y1 = x2 - xc;
                        let dn_x2 = y1 - yc;
                        let dn_y2 = xc - x1;

                        let dl_x1 = -u_x / len;
                        let dl_y1 = -u_y / len;
                        let dl_x2 = u_x / len;
                        let dl_y2 = u_y / len;

                        let d_dist = |dn: f64, dl: f64| (dn * len - num * dl) / (len * len);

                        equations.push(Equation {
                            residual: res,
                            gradients: vec![
                                (xc_i, d_dist(dn_xc, 0.0)),
                                (yc_i, d_dist(dn_yc, 0.0)),
                                (x1_i, d_dist(dn_x1, dl_x1)),
                                (y1_i, d_dist(dn_y1, dl_y1)),
                                (x2_i, d_dist(dn_x2, dl_x2)),
                                (y2_i, d_dist(dn_y2, dl_y2)),
                                (r_i, -side),
                            ],
                        });
                    } else if let (Some(c1), Some(c2)) =
                        (sketch.find_circle(entity1), sketch.find_circle(entity2))
                    {
                        let (xc1_i, yc1_i) = index
                            .point_indices(c1.center)
                            .ok_or(SolverError::EntityNotFound)?;
                        let (xc2_i, yc2_i) = index
                            .point_indices(c2.center)
                            .ok_or(SolverError::EntityNotFound)?;
                        let r1_i = index
                            .circle_radius_idx(c1.id)
                            .ok_or(SolverError::EntityNotFound)?;
                        let r2_i = index
                            .circle_radius_idx(c2.id)
                            .ok_or(SolverError::EntityNotFound)?;

                        let dx = x[xc2_i] - x[xc1_i];
                        let dy = x[yc2_i] - x[yc1_i];
                        let dist = dx.hypot(dy);
                        let s_dist = dist.max(1e-12);

                        let r1 = x[r1_i];
                        let r2 = x[r2_i];

                        // External vs internal tangency
                        if dist >= (r1.max(r2) - 1e-3) {
                            // External
                            equations.push(Equation {
                                residual: dist - (r1 + r2),
                                gradients: vec![
                                    (xc1_i, -dx / s_dist),
                                    (yc1_i, -dy / s_dist),
                                    (xc2_i, dx / s_dist),
                                    (yc2_i, dy / s_dist),
                                    (r1_i, -1.0),
                                    (r2_i, -1.0),
                                ],
                            });
                        } else {
                            // Internal
                            let (larger_r_idx, smaller_r_idx, sign) = if r1 >= r2 {
                                (r1_i, r2_i, 1.0)
                            } else {
                                (r2_i, r1_i, -1.0)
                            };
                            equations.push(Equation {
                                residual: dist - (x[larger_r_idx] - x[smaller_r_idx]),
                                gradients: vec![
                                    (xc1_i, -dx / s_dist),
                                    (yc1_i, -dy / s_dist),
                                    (xc2_i, dx / s_dist),
                                    (yc2_i, dy / s_dist),
                                    (r1_i, -sign),
                                    (r2_i, sign),
                                ],
                            });
                        }
                    } else {
                        return Err(SolverError::EntityNotFound);
                    }
                }
            }
        }

        Ok(equations)
    }

    /// Builds residual vector and Jacobian matrix from equations.
    fn build_system(
        &self,
        equations: &[Equation],
        total_vars: usize,
    ) -> (DVector<f64>, DMatrix<f64>) {
        let m = equations.len();
        let mut residuals = DVector::zeros(m);
        let mut jacobian = DMatrix::zeros(m, total_vars);

        for (row, eq) in equations.iter().enumerate() {
            residuals[row] = eq.residual;
            for &(col, val) in &eq.gradients {
                if col < total_vars {
                    jacobian[(row, col)] += val;
                }
            }
        }

        (residuals, jacobian)
    }

    /// Evaluates the system residuals and Jacobian matrix.
    fn evaluate_system(
        &self,
        x: &DVector<f64>,
        sketch: &Sketch2D,
        constraints: &[Constraint],
        index: &VariableIndex,
    ) -> Result<(DVector<f64>, DMatrix<f64>), SolverError> {
        let equations = self.evaluate_equations(x, sketch, constraints, index)?;
        Ok(self.build_system(&equations, index.total_vars))
    }

    /// Calculates remaining degrees of freedom accurately using Jacobian rank (SVD).
    pub fn calculate_dof(&self, sketch: &Sketch2D, constraints: &[Constraint]) -> ConstraintStatus {
        let index = VariableIndex::new(sketch);
        if index.total_vars == 0 {
            return ConstraintStatus::FullyConstrained;
        }

        let x = index.to_vector(sketch);
        let (residuals, jacobian) = match self.evaluate_system(&x, sketch, constraints, &index) {
            Ok(sys) => sys,
            Err(_) => return ConstraintStatus::Overconstrained,
        };

        let m = residuals.len();
        if m == 0 {
            return ConstraintStatus::Underconstrained(index.total_vars);
        }

        let svd = jacobian.svd(false, false);
        let tol = 1e-7;
        let rank = svd.singular_values.iter().filter(|&&s| s > tol).count();

        if m > rank {
            ConstraintStatus::Overconstrained
        } else if rank >= index.total_vars {
            ConstraintStatus::FullyConstrained
        } else {
            ConstraintStatus::Underconstrained(index.total_vars - rank)
        }
    }

    /// Solves geometric constraints using Levenberg-Marquardt with analytical Jacobian.
    pub fn solve(
        &self,
        sketch: &mut Sketch2D,
        constraints: &[Constraint],
    ) -> Result<ConstraintStatus, SolverError> {
        let index = VariableIndex::new(sketch);
        if index.total_vars == 0 {
            return Ok(ConstraintStatus::FullyConstrained);
        }

        let mut x = index.to_vector(sketch);
        let mut lambda = self.damping;

        // Check if explicitly overconstrained initially (m > rank or m > total_vars with conflicting rows)
        let initial_status = self.calculate_dof(sketch, constraints);
        if let ConstraintStatus::Overconstrained = initial_status {
            return Err(SolverError::Overconstrained);
        }

        let mut converged = false;
        let mut last_max_res = f64::MAX;

        for _iter in 0..self.max_iterations {
            let (residuals, jacobian) = self.evaluate_system(&x, sketch, constraints, &index)?;
            let max_res = residuals.amax();
            last_max_res = max_res;

            if max_res < self.tolerance {
                converged = true;
                break;
            }

            // Normal equations: (J^T * J + lambda * diag(J^T * J + I)) * delta = -J^T * residuals
            let j_t = jacobian.transpose();
            let j_t_j = &j_t * &jacobian;
            let mut a = j_t_j.clone();
            for i in 0..index.total_vars {
                a[(i, i)] += lambda * (j_t_j[(i, i)] + 1.0);
            }
            let b = -&j_t * &residuals;

            // Solve damped step
            let step_opt = a
                .clone()
                .cholesky()
                .map(|chol| chol.solve(&b))
                .or_else(|| a.col_piv_qr().solve(&b));

            if let Some(delta) = step_opt {
                let x_cand = &x + &delta;
                let (cand_res, _) = self.evaluate_system(&x_cand, sketch, constraints, &index)?;
                let cand_max_res = cand_res.amax();

                if cand_max_res < max_res {
                    // Step accepted: decrease damping
                    x = x_cand;
                    lambda = (lambda / 10.0).max(1e-12);
                } else {
                    // Step rejected: increase damping
                    lambda = (lambda * 10.0).min(1e12);
                }
            } else {
                lambda = (lambda * 10.0).min(1e12);
            }
        }

        // Apply solved variables back to sketch
        index.apply_to_sketch(&x, sketch);

        // Verify final convergence and status
        let final_status = self.calculate_dof(sketch, constraints);

        if !converged {
            let (final_res, _) = self.evaluate_system(&x, sketch, constraints, &index)?;
            if final_res.amax() > self.tolerance * 10.0 {
                // If it failed to converge, it's either overconstrained or convergence failure
                return if matches!(final_status, ConstraintStatus::Overconstrained) {
                    Err(SolverError::Overconstrained)
                } else {
                    Err(SolverError::ConvergenceFailed(
                        self.max_iterations,
                        last_max_res,
                    ))
                };
            }
        }

        if matches!(final_status, ConstraintStatus::Overconstrained) {
            return Err(SolverError::Overconstrained);
        }

        Ok(final_status)
    }
}
