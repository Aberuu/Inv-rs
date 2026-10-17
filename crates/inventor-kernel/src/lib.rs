pub mod mesh;
pub mod opencascade_impl;
pub mod traits;
pub mod truck_impl;

pub use mesh::*;
pub use opencascade_impl::*;
pub use traits::*;
pub use truck_impl::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truck_box_primitive() {
        let kernel = TruckKernel::new();
        let mesh = kernel
            .create_box(40.0, 30.0, 20.0)
            .expect("Failed to create box");

        assert!(!mesh.is_empty());
        assert!(mesh.triangle_count() >= 12);
        assert!(mesh.edge_count() >= 12);

        let (min, max) = mesh
            .compute_bounding_box()
            .expect("Bounding box must exist");
        let dx = max[0] - min[0];
        let dy = max[1] - min[1];
        let dz = max[2] - min[2];

        assert!((dx - 40.0).abs() < 1e-1);
        assert!((dy - 30.0).abs() < 1e-1);
        assert!((dz - 20.0).abs() < 1e-1);
    }

    #[test]
    fn test_truck_cylinder_primitive() {
        let kernel = TruckKernel::new();
        let mesh = kernel
            .create_cylinder(15.0, 50.0)
            .expect("Failed to create cylinder");

        assert!(!mesh.is_empty());
        assert!(mesh.triangle_count() > 20);

        let (min, max) = mesh
            .compute_bounding_box()
            .expect("Bounding box must exist");
        let dz = max[2] - min[2];
        assert!((dz - 50.0).abs() < 1e-1);
    }

    #[test]
    fn test_truck_extrude_polygon() {
        let kernel = TruckKernel::new();
        let rect_pts = [[0.0, 0.0], [100.0, 0.0], [100.0, 50.0], [0.0, 50.0]];
        let mesh = kernel
            .extrude_polygon(&rect_pts, 25.0)
            .expect("Failed to extrude rectangle");

        assert!(!mesh.is_empty());
        assert!(mesh.triangle_count() >= 12);
        let (min, max) = mesh
            .compute_bounding_box()
            .expect("Bounding box must exist");
        let dx = max[0] - min[0];
        let dy = max[1] - min[1];
        let dz = max[2] - min[2];
        assert!((dx - 100.0).abs() < 1e-1);
        assert!((dy - 50.0).abs() < 1e-1);
        assert!((dz - 25.0).abs() < 1e-1);
    }

    #[test]
    fn test_cad_mesh_transform_by_basis() {
        let kernel = TruckKernel::new();
        let rect_pts = [[0.0, 0.0], [100.0, 0.0], [100.0, 50.0], [0.0, 50.0]];
        let mut mesh = kernel
            .extrude_polygon(&rect_pts, 25.0)
            .expect("Failed to extrude rectangle");

        // Transform to XZ Plane: X maps to X, Y maps to Z, normal (+Z) maps to Y
        mesh.transform_by_basis(
            [10.0, 20.0, 30.0], // origin offset
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
        );

        let (min, max) = mesh
            .compute_bounding_box()
            .expect("Bounding box must exist");
        let dx = max[0] - min[0];
        let dy = max[1] - min[1];
        let dz = max[2] - min[2];

        assert!((dx - 100.0).abs() < 1e-1);
        assert!((dy - 25.0).abs() < 1e-1); // height along Y
        assert!((dz - 50.0).abs() < 1e-1); // width along Z
        assert!((min[0] - 10.0).abs() < 1e-1);
        assert!((min[1] - 20.0).abs() < 1e-1);
        assert!((min[2] - 30.0).abs() < 1e-1);
    }

    #[test]
    fn test_truck_revolve_polygon() {
        let kernel = TruckKernel::new();
        let pts = [[10.0, 0.0], [20.0, 0.0], [20.0, 30.0], [10.0, 30.0]];

        // Full 360 deg revolve
        let mesh = kernel
            .revolve_polygon(
                &pts,
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                std::f64::consts::TAU,
            )
            .expect("Revolve 360 deg failed");

        assert!(!mesh.is_empty());
        let (min, max) = mesh.compute_bounding_box().unwrap();
        let dx = max[0] - min[0];
        let dy = max[1] - min[1];
        let dz = max[2] - min[2];
        assert!((dx - 40.0).abs() < 1e-1);
        assert!((dy - 30.0).abs() < 1e-1);
        assert!((dz - 40.0).abs() < 1e-1);

        // Partial angle revolve: 90 degrees (FRAC_PI_2)
        let mesh_90 = kernel
            .revolve_polygon(
                &pts,
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                std::f64::consts::FRAC_PI_2,
            )
            .expect("Revolve 90 deg failed");
        assert!(!mesh_90.is_empty());
    }

    #[test]
    fn test_truck_booleans() {
        use inventor_core::BooleanOp;

        let kernel = TruckKernel::new();
        let shape_a = SolidShape::Box {
            dx: 40.0,
            dy: 40.0,
            dz: 40.0,
        };
        let shape_b = SolidShape::Box {
            dx: 20.0,
            dy: 20.0,
            dz: 60.0,
        };

        // 1. Intersect
        let and_mesh = kernel
            .boolean_shapes(BooleanOp::Intersect, &shape_a, &shape_b)
            .expect("Boolean Intersect failed");
        let (and_min, and_max) = and_mesh.compute_bounding_box().unwrap();
        assert!(((and_max[0] - and_min[0]) - 20.0).abs() < 1e-1);
        assert!(((and_max[1] - and_min[1]) - 20.0).abs() < 1e-1);
        assert!(((and_max[2] - and_min[2]) - 40.0).abs() < 1e-1);

        // 2. Union
        let or_mesh = kernel
            .boolean_shapes(BooleanOp::Union, &shape_a, &shape_b)
            .expect("Boolean Union failed");
        let (or_min, or_max) = or_mesh.compute_bounding_box().unwrap();
        assert!(((or_max[0] - or_min[0]) - 40.0).abs() < 1e-1);
        assert!(((or_max[1] - or_min[1]) - 40.0).abs() < 1e-1);
        assert!(((or_max[2] - or_min[2]) - 60.0).abs() < 1e-1);

        // 3. Subtract (Cut)
        let sub_mesh = kernel
            .boolean_shapes(BooleanOp::Subtract, &shape_a, &shape_b)
            .expect("Boolean Subtract failed");
        let (sub_min, sub_max) = sub_mesh.compute_bounding_box().unwrap();
        assert!(((sub_max[0] - sub_min[0]) - 40.0).abs() < 1e-1);
        assert!(((sub_max[1] - sub_min[1]) - 40.0).abs() < 1e-1);
        assert!(((sub_max[2] - sub_min[2]) - 40.0).abs() < 1e-1);
        assert!(sub_mesh.triangle_count() > 12);
    }

    #[test]
    fn test_truck_build_solid_mesh_composite() {
        use inventor_core::BooleanOp;

        let kernel = TruckKernel::new();
        let base = SolidShape::Box {
            dx: 50.0,
            dy: 50.0,
            dz: 20.0,
        };
        let hole = SolidShape::Cylinder {
            radius: 8.0,
            height: 30.0,
        };

        let mesh = kernel
            .build_solid_mesh(&[(base, None), (hole, Some(BooleanOp::Subtract))])
            .expect("Failed to build composite solid");

        assert!(!mesh.is_empty());
        let (min, max) = mesh.compute_bounding_box().unwrap();
        assert!(((max[0] - min[0]) - 50.0).abs() < 1e-1);
        assert!(((max[1] - min[1]) - 50.0).abs() < 1e-1);
        assert!(((max[2] - min[2]) - 20.0).abs() < 1e-1);
    }

    #[test]
    fn test_extrude_cut_and_compound_shapes() {
        use inventor_core::BooleanOp;

        let kernel = TruckKernel::new();
        let pts_a = [[0.0, 0.0], [50.0, 0.0], [50.0, 30.0], [0.0, 30.0]];
        let pts_b = [[10.0, 5.0], [30.0, 5.0], [30.0, 25.0], [10.0, 25.0]];

        let eps = 0.1;
        let shape_a = SolidShape::Extrude {
            points: pts_a.to_vec(),
            height: 20.0,
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        };
        let shape_cut = SolidShape::Extrude {
            points: pts_b.to_vec(),
            height: 20.0 + 2.0 * eps,
            origin: [0.0, 0.0, -eps],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        };

        let mesh = kernel
            .boolean_shapes(BooleanOp::Subtract, &shape_a, &shape_cut)
            .expect("Boolean Subtract with epsilon tool must succeed");
        assert!(mesh.triangle_count() > 12);

        // Test compound shapes (disjoint bodies or boundary contact)
        let pts_xz = [[0.0, 0.0], [60.0, 0.0], [60.0, 40.0], [0.0, 40.0]];
        let shape_xz = SolidShape::Extrude {
            points: pts_xz.to_vec(),
            height: 35.0,
            origin: [0.0, 0.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 0.0, 1.0],
            normal: [0.0, -1.0, 0.0],
        };
        let compound_mesh = kernel
            .build_solid_mesh(&[(shape_a, None), (shape_xz, Some(BooleanOp::Union))])
            .expect("Compound solid mesh build must succeed");
        assert!(!compound_mesh.is_empty());
    }

    #[test]
    fn test_truck_fillet_chamfer_capability_evaluation() {
        let kernel = TruckKernel::new();
        assert_eq!(kernel.kernel_name(), "truck (Pure Rust B-rep)");
        assert!(!kernel.supports_brep_fillet());

        let box_mesh = kernel
            .create_box(20.0, 20.0, 20.0)
            .expect("Create box must succeed");

        // Evaluates that TruckKernel clearly reports lack of 3D B-rep edge blending
        let fillet_err = kernel.fillet_solid(&box_mesh, 2.0).unwrap_err();
        assert!(
            fillet_err
                .to_string()
                .contains("unsupported in pure-Rust truck kernel")
        );
        assert!(fillet_err.to_string().contains("OpenCASCADE"));

        let chamfer_err = kernel.chamfer_solid(&box_mesh, 2.0).unwrap_err();
        assert!(
            chamfer_err
                .to_string()
                .contains("unsupported in pure-Rust truck kernel")
        );
        assert!(chamfer_err.to_string().contains("OpenCASCADE"));
    }

    #[test]
    fn test_opencascade_and_dual_kernel_fillet_chamfer() {
        let occ_kernel = OpenCascadeKernel::new();
        assert_eq!(
            occ_kernel.kernel_name(),
            "OpenCASCADE Technology (OCCT B-rep via FFI)"
        );
        assert!(occ_kernel.supports_brep_fillet());

        let box_mesh = occ_kernel
            .create_box(30.0, 30.0, 30.0)
            .expect("Create box must succeed");

        let filleted = occ_kernel
            .fillet_solid(&box_mesh, 3.0)
            .expect("OCCT fillet must succeed");
        assert!(!filleted.is_empty());

        let chamfered = occ_kernel
            .chamfer_solid(&box_mesh, 2.5)
            .expect("OCCT chamfer must succeed");
        assert!(!chamfered.is_empty());

        // Test DualKernel switching
        let mut dual = DualKernel::new();
        assert_eq!(dual.kernel_name(), "truck (Pure Rust B-rep)");
        assert!(!dual.supports_brep_fillet());

        dual = dual.with_backend(KernelBackend::OpenCascade);
        assert_eq!(
            dual.kernel_name(),
            "OpenCASCADE Technology (OCCT B-rep via FFI)"
        );
        assert!(dual.supports_brep_fillet());

        let dual_filleted = dual
            .fillet_solid(&box_mesh, 2.0)
            .expect("DualKernel with OCCT must succeed on fillet");
        assert!(!dual_filleted.is_empty());
    }

    #[test]
    fn test_mesh_watertight_inspection() {
        let kernel = TruckKernel::new();
        let box_mesh = kernel.create_box(20.0, 20.0, 20.0).unwrap();

        let report = box_mesh.inspect_watertight();
        assert!(
            report.is_watertight,
            "Solid box mesh must be closed and watertight"
        );
        assert!(report.is_manifold, "Solid box mesh must be 2-manifold");
        assert_eq!(
            report.boundary_edge_count, 0,
            "Watertight box has 0 boundary edges"
        );
        assert_eq!(report.non_manifold_edge_count, 0);

        // Exact analytical volume = 20 * 20 * 20 = 8000 mm^3
        let expected_vol = 8000.0;
        assert!(
            (report.volume - expected_vol).abs() < 1.0,
            "Box volume mismatch: got {}, expected {}",
            report.volume,
            expected_vol
        );

        // Surface area of 6 faces = 6 * (20 * 20) = 2400 mm^2
        let expected_area = 2400.0;
        assert!(
            (report.surface_area - expected_area).abs() < 1.0,
            "Box area mismatch: got {}, expected {}",
            report.surface_area,
            expected_area
        );

        // Test non-watertight open mesh (single quad made of 2 triangles)
        let mut open_mesh = CadMesh::new();
        open_mesh.positions.push([0.0, 0.0, 0.0]);
        open_mesh.positions.push([10.0, 0.0, 0.0]);
        open_mesh.positions.push([10.0, 10.0, 0.0]);
        open_mesh.positions.push([0.0, 10.0, 0.0]);
        open_mesh.normals.resize(4, [0.0, 0.0, 1.0]);
        open_mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);

        let open_report = open_mesh.inspect_watertight();
        assert!(
            !open_report.is_watertight,
            "Open sheet mesh must NOT be watertight"
        );
        assert_eq!(
            open_report.boundary_edge_count, 4,
            "Open quad sheet has 4 border edges"
        );
        assert_eq!(open_report.volume, 0.0, "Planar sheet has zero volume");
    }
}
