pub mod constraint;
pub mod entity;
pub mod solver;

pub use constraint::*;
pub use entity::*;
pub use solver::*;

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_sketch_entities() {
        let mut sketch = Sketch2D::new("Sketch1", Uuid::new_v4());
        let p1 = sketch.add_point(0.0, 0.0);
        let p2 = sketch.add_point(10.0, 5.0);
        let l1 = sketch.add_line(p1, p2);

        assert_eq!(sketch.points.len(), 2);
        assert_eq!(sketch.lines.len(), 1);
        assert_eq!(sketch.lines[0].id, l1);
    }

    #[test]
    fn test_under_constrained_sketch() {
        let mut sketch = Sketch2D::new("Sketch1", Uuid::new_v4());
        let p1 = sketch.add_point(0.0, 10.0);
        let p2 = sketch.add_point(20.0, 14.0);
        let l1 = sketch.add_line(p1, p2);

        let solver = ConstraintSolver::new();

        // 1 line with 2 points = 4 DOFs
        let status = solver.calculate_dof(&sketch, &[]);
        assert_eq!(status, ConstraintStatus::Underconstrained(4));

        // Adding Horizontal constraint removes 1 DOF -> 3 DOFs remaining
        let c_h = Constraint::new("H1", ConstraintKind::Horizontal { line: l1 });
        let status = solver
            .solve(&mut sketch, std::slice::from_ref(&c_h))
            .unwrap();
        assert_eq!(status, ConstraintStatus::Underconstrained(3));

        let p1_after = sketch.find_point(p1).unwrap();
        let p2_after = sketch.find_point(p2).unwrap();
        assert!((p1_after.y - p2_after.y).abs() < 1e-5);

        // Adding Distance constraint removes another 1 DOF -> 2 DOFs remaining
        let c_d = Constraint::new(
            "D1",
            ConstraintKind::Distance {
                point1: p1,
                point2: p2,
                distance: 50.0,
            },
        );
        let status = solver.solve(&mut sketch, &[c_h, c_d]).unwrap();
        assert_eq!(status, ConstraintStatus::Underconstrained(2));

        let p1_after = sketch.find_point(p1).unwrap();
        let p2_after = sketch.find_point(p2).unwrap();
        let length = (p2_after.x - p1_after.x).hypot(p2_after.y - p1_after.y);
        assert!((length - 50.0).abs() < 1e-5);
    }

    #[test]
    fn test_fully_constrained_rectangle() {
        let mut sketch = Sketch2D::new("RectangleSketch", Uuid::new_v4());
        let p1 = sketch.add_point(0.0, 0.0);
        let p2 = sketch.add_point(90.0, 5.0);
        let p3 = sketch.add_point(85.0, 45.0);
        let p4 = sketch.add_point(-5.0, 40.0);

        let l1 = sketch.add_line(p1, p2); // bottom
        let l2 = sketch.add_line(p2, p3); // right
        let l3 = sketch.add_line(p3, p4); // top
        let l4 = sketch.add_line(p4, p1); // left

        // 4 points = 8 DOFs
        // Constraints to fully constrain:
        // 1. Fix corner p1 at (0, 0) [2 DOFs]
        // 2. l1 is Horizontal [1 DOF]
        // 3. l1 length = 100.0 [1 DOF]
        // 4. l4 is Vertical [1 DOF]
        // 5. l4 length = 50.0 [1 DOF]
        // 6. l3 is Parallel to l1 [1 DOF]
        // 7. l2 is Parallel to l4 [1 DOF]
        // Total = 8 equations, rank = 8, DOF = 0 -> FullyConstrained
        let constraints = vec![
            Constraint::new(
                "FixOrigin",
                ConstraintKind::FixPoint {
                    point: p1,
                    x: 0.0,
                    y: 0.0,
                },
            ),
            Constraint::new("BottomHoriz", ConstraintKind::Horizontal { line: l1 }),
            Constraint::new(
                "Width",
                ConstraintKind::Distance {
                    point1: p1,
                    point2: p2,
                    distance: 100.0,
                },
            ),
            Constraint::new("LeftVert", ConstraintKind::Vertical { line: l4 }),
            Constraint::new(
                "Height",
                ConstraintKind::Distance {
                    point1: p1,
                    point2: p4,
                    distance: 50.0,
                },
            ),
            Constraint::new(
                "TopParallel",
                ConstraintKind::Parallel {
                    line1: l1,
                    line2: l3,
                },
            ),
            Constraint::new(
                "RightParallel",
                ConstraintKind::Parallel {
                    line1: l2,
                    line2: l4,
                },
            ),
        ];

        let solver = ConstraintSolver::new();
        let status = solver.solve(&mut sketch, &constraints).unwrap();
        assert_eq!(status, ConstraintStatus::FullyConstrained);

        let p1_after = sketch.find_point(p1).unwrap();
        let p2_after = sketch.find_point(p2).unwrap();
        let p3_after = sketch.find_point(p3).unwrap();
        let p4_after = sketch.find_point(p4).unwrap();

        assert!((p1_after.x - 0.0).abs() < 1e-4);
        assert!((p1_after.y - 0.0).abs() < 1e-4);
        assert!((p2_after.x - 100.0).abs() < 1e-4);
        assert!((p2_after.y - 0.0).abs() < 1e-4);
        assert!((p3_after.x - 100.0).abs() < 1e-4);
        assert!((p3_after.y - 50.0).abs() < 1e-4);
        assert!((p4_after.x - 0.0).abs() < 1e-4);
        assert!((p4_after.y - 50.0).abs() < 1e-4);
    }

    #[test]
    fn test_over_constrained_sketch_conflicting() {
        let mut sketch = Sketch2D::new("Overconstrained", Uuid::new_v4());
        let p1 = sketch.add_point(0.0, 0.0);
        let p2 = sketch.add_point(10.0, 0.0);

        // Two conflicting distance constraints on the same points
        let constraints = vec![
            Constraint::new(
                "D1",
                ConstraintKind::Distance {
                    point1: p1,
                    point2: p2,
                    distance: 30.0,
                },
            ),
            Constraint::new(
                "D2",
                ConstraintKind::Distance {
                    point1: p1,
                    point2: p2,
                    distance: 60.0,
                },
            ),
        ];

        let solver = ConstraintSolver::new();
        let dof_status = solver.calculate_dof(&sketch, &constraints);
        assert_eq!(dof_status, ConstraintStatus::Overconstrained);

        let solve_res = solver.solve(&mut sketch, &constraints);
        assert_eq!(solve_res, Err(SolverError::Overconstrained));
    }

    #[test]
    fn test_over_constrained_sketch_redundant() {
        let mut sketch = Sketch2D::new("RedundantSketch", Uuid::new_v4());
        let p1 = sketch.add_point(0.0, 0.0);
        let p2 = sketch.add_point(20.0, 5.0);
        let l1 = sketch.add_line(p1, p2);

        // Two identical horizontal constraints on the same line
        let constraints = vec![
            Constraint::new("H1", ConstraintKind::Horizontal { line: l1 }),
            Constraint::new("H2", ConstraintKind::Horizontal { line: l1 }),
        ];

        let solver = ConstraintSolver::new();
        let dof_status = solver.calculate_dof(&sketch, &constraints);
        assert_eq!(dof_status, ConstraintStatus::Overconstrained);

        let solve_res = solver.solve(&mut sketch, &constraints);
        assert_eq!(solve_res, Err(SolverError::Overconstrained));
    }

    #[test]
    fn test_diameter_and_radius_constraints() {
        let mut sketch = Sketch2D::new("CircleSketch", Uuid::new_v4());
        let center = sketch.add_point(10.0, 10.0);
        let circle = sketch.add_circle(center, 5.0);

        let solver = ConstraintSolver::new();

        // 1 point (2 DOFs) + 1 circle radius (1 DOF) = 3 DOFs
        assert_eq!(
            solver.calculate_dof(&sketch, &[]),
            ConstraintStatus::Underconstrained(3)
        );

        // Constrain Diameter = 50.0 (radius should become 25.0)
        let c_dia = vec![Constraint::new(
            "Dia50",
            ConstraintKind::Diameter {
                circle,
                diameter: 50.0,
            },
        )];
        let status = solver.solve(&mut sketch, &c_dia).unwrap();
        assert_eq!(status, ConstraintStatus::Underconstrained(2));

        let c_after = sketch.find_circle(circle).unwrap();
        assert!((c_after.radius - 25.0).abs() < 1e-5);

        // Now constrain Radius = 15.0
        let c_rad = vec![Constraint::new(
            "Rad15",
            ConstraintKind::Radius {
                circle,
                radius: 15.0,
            },
        )];
        let status = solver.solve(&mut sketch, &c_rad).unwrap();
        assert_eq!(status, ConstraintStatus::Underconstrained(2));

        let c_after2 = sketch.find_circle(circle).unwrap();
        assert!((c_after2.radius - 15.0).abs() < 1e-5);
    }

    #[test]
    fn test_equal_constraint_lines_and_circles() {
        let mut sketch = Sketch2D::new("EqualSketch", Uuid::new_v4());
        let p1 = sketch.add_point(0.0, 0.0);
        let p2 = sketch.add_point(30.0, 0.0);
        let l1 = sketch.add_line(p1, p2);

        let p3 = sketch.add_point(0.0, 20.0);
        let p4 = sketch.add_point(10.0, 20.0);
        let l2 = sketch.add_line(p3, p4);

        let c1_center = sketch.add_point(50.0, 50.0);
        let c1 = sketch.add_circle(c1_center, 12.0);

        let c2_center = sketch.add_point(80.0, 50.0);
        let c2 = sketch.add_circle(c2_center, 25.0);

        let constraints = vec![
            Constraint::new(
                "L1_Len",
                ConstraintKind::Distance {
                    point1: p1,
                    point2: p2,
                    distance: 40.0,
                },
            ),
            Constraint::new(
                "EqualLines",
                ConstraintKind::Equal {
                    entity1: l1,
                    entity2: l2,
                },
            ),
            Constraint::new(
                "C1_Rad",
                ConstraintKind::Radius {
                    circle: c1,
                    radius: 16.0,
                },
            ),
            Constraint::new(
                "EqualCircles",
                ConstraintKind::Equal {
                    entity1: c1,
                    entity2: c2,
                },
            ),
        ];

        let solver = ConstraintSolver::new();
        solver.solve(&mut sketch, &constraints).unwrap();

        let l2_p3 = sketch.find_point(p3).unwrap();
        let l2_p4 = sketch.find_point(p4).unwrap();
        let l2_len = (l2_p4.x - l2_p3.x).hypot(l2_p4.y - l2_p3.y);
        assert!((l2_len - 40.0).abs() < 1e-4);

        let c2_after = sketch.find_circle(c2).unwrap();
        assert!((c2_after.radius - 16.0).abs() < 1e-4);
    }

    #[test]
    fn test_angle_constraint() {
        let mut sketch = Sketch2D::new("AngleSketch", Uuid::new_v4());
        let p_origin = sketch.add_point(0.0, 0.0);
        let p_x = sketch.add_point(50.0, 0.0);
        let p_ang = sketch.add_point(30.0, 10.0);

        let l_base = sketch.add_line(p_origin, p_x);
        let l_arm = sketch.add_line(p_origin, p_ang);

        let constraints = vec![
            Constraint::new(
                "FixO",
                ConstraintKind::FixPoint {
                    point: p_origin,
                    x: 0.0,
                    y: 0.0,
                },
            ),
            Constraint::new("BaseH", ConstraintKind::Horizontal { line: l_base }),
            Constraint::new(
                "BaseLen",
                ConstraintKind::Distance {
                    point1: p_origin,
                    point2: p_x,
                    distance: 50.0,
                },
            ),
            Constraint::new(
                "ArmLen",
                ConstraintKind::Distance {
                    point1: p_origin,
                    point2: p_ang,
                    distance: 50.0,
                },
            ),
            Constraint::new(
                "Angle60",
                ConstraintKind::Angle {
                    line1: l_base,
                    line2: l_arm,
                    angle_deg: 60.0,
                },
            ),
        ];

        let solver = ConstraintSolver::new();
        solver.solve(&mut sketch, &constraints).unwrap();

        let p_ang_after = sketch.find_point(p_ang).unwrap();
        // At 60 deg with length 50: x = 50 * cos(60) = 25.0, y = 50 * sin(60) = 43.30127
        assert!((p_ang_after.x - 25.0).abs() < 1e-4);
        assert!((p_ang_after.y - (50.0 * 60.0_f64.to_radians().sin())).abs() < 1e-4);
    }

    #[test]
    fn test_tangent_constraint_line_circle() {
        let mut sketch = Sketch2D::new("TangentLineCircle", Uuid::new_v4());
        let p1 = sketch.add_point(-50.0, 0.0);
        let p2 = sketch.add_point(50.0, 0.0);
        let line = sketch.add_line(p1, p2);

        let center = sketch.add_point(0.0, 25.0);
        let circle = sketch.add_circle(center, 15.0);

        let constraints = vec![
            Constraint::new(
                "FixP1",
                ConstraintKind::FixPoint {
                    point: p1,
                    x: -50.0,
                    y: 0.0,
                },
            ),
            Constraint::new(
                "FixP2",
                ConstraintKind::FixPoint {
                    point: p2,
                    x: 50.0,
                    y: 0.0,
                },
            ),
            Constraint::new(
                "Radius15",
                ConstraintKind::Radius {
                    circle,
                    radius: 15.0,
                },
            ),
            Constraint::new(
                "TangentLC",
                ConstraintKind::Tangent {
                    entity1: line,
                    entity2: circle,
                },
            ),
        ];

        let solver = ConstraintSolver::new();
        solver.solve(&mut sketch, &constraints).unwrap();

        let center_after = sketch.find_point(center).unwrap();
        // Since the line is y = 0 and circle was at y = 25, distance to line is |y|.
        // Tangency with radius 15 should move center to y = 15.0
        assert!((center_after.y - 15.0).abs() < 1e-4);
    }

    #[test]
    fn test_tangent_constraint_circle_circle() {
        let mut sketch = Sketch2D::new("TangentCircleCircle", Uuid::new_v4());
        let c1_center = sketch.add_point(0.0, 0.0);
        let c1 = sketch.add_circle(c1_center, 20.0);

        let c2_center = sketch.add_point(45.0, 0.0);
        let c2 = sketch.add_circle(c2_center, 10.0);

        let constraints = vec![
            Constraint::new(
                "FixC1",
                ConstraintKind::FixPoint {
                    point: c1_center,
                    x: 0.0,
                    y: 0.0,
                },
            ),
            Constraint::new(
                "RadC1",
                ConstraintKind::Radius {
                    circle: c1,
                    radius: 20.0,
                },
            ),
            Constraint::new(
                "RadC2",
                ConstraintKind::Radius {
                    circle: c2,
                    radius: 10.0,
                },
            ),
            Constraint::new(
                "TangentCC",
                ConstraintKind::Tangent {
                    entity1: c1,
                    entity2: c2,
                },
            ),
        ];

        let solver = ConstraintSolver::new();
        solver.solve(&mut sketch, &constraints).unwrap();

        let c1_pt = sketch.find_point(c1_center).unwrap();
        let c2_pt = sketch.find_point(c2_center).unwrap();
        let dist = (c2_pt.x - c1_pt.x).hypot(c2_pt.y - c1_pt.y);
        // External tangent distance = 20.0 + 10.0 = 30.0
        assert!((dist - 30.0).abs() < 1e-4);
    }

    #[test]
    fn test_interdependent_constraints_stably_converge() {
        let mut sketch = Sketch2D::new("Interdependent", Uuid::new_v4());
        let p_origin = sketch.add_point(0.0, 0.0);
        let p_x = sketch.add_point(70.0, 5.0);
        let p_y = sketch.add_point(5.0, 65.0);

        let l_base = sketch.add_line(p_origin, p_x);
        let l_vert = sketch.add_line(p_origin, p_y);
        let l_hyp = sketch.add_line(p_x, p_y);

        let c_center = sketch.add_point(25.0, 25.0);
        let circle = sketch.add_circle(c_center, 10.0);

        // Interdependent system:
        // - Right triangle with legs of equal length (l_base == l_vert)
        // - Base is horizontal, length = 80.0
        // - Leg is vertical
        // - Circle diameter = 30.0
        // - Circle is tangent to hypotenuse
        let constraints = vec![
            Constraint::new(
                "FixO",
                ConstraintKind::FixPoint {
                    point: p_origin,
                    x: 0.0,
                    y: 0.0,
                },
            ),
            Constraint::new("BaseH", ConstraintKind::Horizontal { line: l_base }),
            Constraint::new(
                "BaseLen",
                ConstraintKind::Distance {
                    point1: p_origin,
                    point2: p_x,
                    distance: 80.0,
                },
            ),
            Constraint::new("VertV", ConstraintKind::Vertical { line: l_vert }),
            Constraint::new(
                "EqualLegs",
                ConstraintKind::Equal {
                    entity1: l_base,
                    entity2: l_vert,
                },
            ),
            Constraint::new(
                "CirDia",
                ConstraintKind::Diameter {
                    circle,
                    diameter: 30.0,
                },
            ),
            Constraint::new(
                "HypTangent",
                ConstraintKind::Tangent {
                    entity1: l_hyp,
                    entity2: circle,
                },
            ),
        ];

        let solver = ConstraintSolver::new();
        let status = solver.solve(&mut sketch, &constraints).unwrap();
        assert!(matches!(
            status,
            ConstraintStatus::Underconstrained(_) | ConstraintStatus::FullyConstrained
        ));

        let p_x_after = sketch.find_point(p_x).unwrap();
        let p_y_after = sketch.find_point(p_y).unwrap();
        let c_pt_after = sketch.find_point(c_center).unwrap();
        let cir_after = sketch.find_circle(circle).unwrap();

        assert!((p_x_after.x - 80.0).abs() < 1e-4);
        assert!((p_x_after.y - 0.0).abs() < 1e-4);
        assert!((p_y_after.x - 0.0).abs() < 1e-4);
        assert!((p_y_after.y - 80.0).abs() < 1e-4);
        assert!((cir_after.radius - 15.0).abs() < 1e-4);

        // Distance from circle center to hypotenuse line (80,0) -> (0,80)
        // Hypotenuse line eq: x + y = 80 => (x + y - 80) / sqrt(2)
        let dist_to_hyp = ((c_pt_after.x + c_pt_after.y - 80.0).abs()) / (2.0_f64.sqrt());
        assert!((dist_to_hyp - 15.0).abs() < 1e-4);
    }

    #[test]
    fn test_extract_closed_profiles() {
        let mut sketch = Sketch2D::new("ProfileTest", Uuid::new_v4());

        // 1. Initially empty: 0 profiles
        assert!(sketch.extract_closed_profiles().is_empty());

        // 2. Open lines: 0 profiles
        let p0 = sketch.add_point(0.0, 0.0);
        let p1 = sketch.add_point(50.0, 0.0);
        let p2 = sketch.add_point(50.0, 30.0);
        sketch.add_line(p0, p1);
        sketch.add_line(p1, p2);
        assert!(sketch.extract_closed_profiles().is_empty());

        // 3. Complete closed rectangle: exactly 1 profile of 4 vertices
        let p3 = sketch.add_point(0.0, 30.0);
        sketch.add_line(p2, p3);
        sketch.add_line(p3, p0);

        let profiles = sketch.extract_closed_profiles();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].len(), 4);

        // Check CCW area > 0
        let poly = &profiles[0];
        let mut area = 0.0;
        for i in 0..poly.len() {
            let next = (i + 1) % poly.len();
            area += poly[i][0] * poly[next][1] - poly[next][0] * poly[i][1];
        }
        assert!(area > 0.0, "Extracted profile must be oriented CCW");
        assert!(
            (area / 2.0 - 1500.0).abs() < 1e-4,
            "Expected area = 50 * 30 = 1500"
        );

        // 4. Standalone circle fallback profile
        let mut circle_sketch = Sketch2D::new("CircleProfile", Uuid::new_v4());
        let center = circle_sketch.add_point(10.0, 20.0);
        circle_sketch.add_circle(center, 15.0);

        let c_profiles = circle_sketch.extract_closed_profiles();
        assert_eq!(c_profiles.len(), 1);
        assert_eq!(c_profiles[0].len(), 48);
    }
}
