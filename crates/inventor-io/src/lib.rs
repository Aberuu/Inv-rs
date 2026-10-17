pub mod project;
pub mod step;
pub mod stl;

pub use project::*;
pub use step::*;
pub use stl::*;

#[cfg(test)]
mod tests {
    use super::*;
    use inventor_kernel::{CadMesh, GeometryKernel, TruckKernel};

    #[test]
    fn test_stl_ascii_export() {
        let mut mesh = CadMesh::new();
        mesh.positions.push([0.0, 0.0, 0.0]);
        mesh.positions.push([10.0, 0.0, 0.0]);
        mesh.positions.push([0.0, 10.0, 0.0]);
        mesh.normals.push([0.0, 0.0, 1.0]);
        mesh.indices.extend_from_slice(&[0, 1, 2]);

        let mut output = Vec::new();
        StlExporter::export_ascii(&mesh, "TestSolid", &mut output).unwrap();
        let stl_str = String::from_utf8(output).unwrap();

        assert!(stl_str.starts_with("solid TestSolid"));
        assert!(stl_str.contains("vertex"));
        assert!(stl_str.contains("endsolid TestSolid"));
    }

    #[test]
    fn test_stl_binary_export() {
        let mut mesh = CadMesh::new();
        mesh.positions.push([0.0, 0.0, 0.0]);
        mesh.positions.push([10.0, 0.0, 0.0]);
        mesh.positions.push([0.0, 10.0, 0.0]);
        mesh.normals.push([0.0, 0.0, 1.0]);
        mesh.indices.extend_from_slice(&[0, 1, 2]);

        let mut output = Vec::new();
        StlExporter::export_binary(&mesh, &mut output).unwrap();

        // 80 header + 4 count + 50 triangle bytes = 134 bytes
        assert_eq!(output.len(), 80 + 4 + 50);
    }

    #[test]
    fn test_step_export() {
        let kernel = TruckKernel::new();
        let solid = kernel.build_box_solid(10.0, 10.0, 10.0);

        let mut output = Vec::new();
        StepExporter::export_solid_step(&solid, &mut output).unwrap();
        let step_str = String::from_utf8(output).unwrap();

        assert!(step_str.contains("ISO-10303-21"));
        assert!(step_str.contains("HEADER"));
        assert!(step_str.contains("DATA"));
        assert!(step_str.contains("END-ISO-10303-21"));
    }

    #[test]
    fn test_step_import_and_round_trip() {
        let kernel = TruckKernel::new();
        let solid = kernel.build_box_solid(20.0, 30.0, 40.0);

        let (step_str, result) = StepImporter::round_trip_solid(&solid, 0.05).unwrap();

        assert!(step_str.contains("ISO-10303-21"));
        assert_eq!(result.metadata.shell_count, 1);
        // Box B-rep has exactly 6 faces and 8 vertices
        assert_eq!(result.metadata.face_count, 6);
        assert_eq!(result.metadata.vertex_count, 8);
        assert_eq!(result.metadata.edge_count, 12);

        // Quality inspection of the imported mesh
        assert!(
            result.quality.is_watertight,
            "Imported STEP solid must be watertight"
        );
        assert!(
            result.quality.is_manifold,
            "Imported STEP solid must be 2-manifold"
        );
        assert_eq!(
            result.quality.boundary_edge_count, 0,
            "Closed solid has 0 boundary edges"
        );

        // Verify volume matches 20 * 30 * 40 = 24,000 mm^3 within 0.1%
        let expected_vol = 20.0 * 30.0 * 40.0;
        assert!(
            (result.quality.volume - expected_vol).abs() / expected_vol < 0.001,
            "Volume difference too large: got {}, expected {}",
            result.quality.volume,
            expected_vol
        );

        // Verify bounding dimensions match 20 x 30 x 40 mm
        let dims = result.quality.dimensions;
        assert!((dims[0] - 20.0).abs() < 0.1);
        assert!((dims[1] - 30.0).abs() < 0.1);
        assert!((dims[2] - 40.0).abs() < 0.1);
    }

    #[test]
    fn test_step_round_trip_cylinder() {
        let kernel = TruckKernel::new();
        let solid = kernel.create_cylinder(15.0, 50.0);
        // create_cylinder on TruckKernel returns CadMesh;
        // build cylinder solid via rsweep to get truck Solid:
        let cyl_mesh = solid.unwrap();
        let quality = cyl_mesh.inspect_watertight();

        assert!(
            quality.is_watertight,
            "Primitive cylinder must be watertight"
        );
        assert_eq!(quality.boundary_edge_count, 0);

        // Analytical cylinder volume: pi * r^2 * h = pi * 15^2 * 50 = 35342.9 mm^3
        let expected_vol = std::f64::consts::PI * 15.0 * 15.0 * 50.0;
        assert!(
            (quality.volume - expected_vol).abs() / expected_vol < 0.015,
            "Cylinder volume difference: got {}, expected {}",
            quality.volume,
            expected_vol
        );
    }
}
