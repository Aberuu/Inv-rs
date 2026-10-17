use inventor_kernel::{CadMesh, MeshQualityReport};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use thiserror::Error;
use truck_meshalgo::tessellation::{MeshableShape, MeshedShape};
use truck_stepio::r#in::Table;

#[derive(Error, Debug)]
pub enum StepError {
    #[error("STEP IO error: {0}")]
    Io(#[from] io::Error),
    #[error("STEP generation failed: {0}")]
    GenerationFailed(String),
    #[error("STEP parse failed: {0}")]
    ParseFailed(String),
    #[error("STEP topology reconstruction failed: {0}")]
    TopologyError(String),
}

/// Metadata describing topological entities parsed from a STEP file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepImportMetadata {
    pub shell_count: usize,
    pub face_count: usize,
    pub edge_count: usize,
    pub vertex_count: usize,
}

/// Comprehensive result of importing and reconstructing a STEP model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepImportResult {
    pub metadata: StepImportMetadata,
    pub mesh: CadMesh,
    pub quality: MeshQualityReport,
}

pub struct StepExporter;

impl StepExporter {
    /// Save truck solid as ISO 10303-21 STEP format string.
    pub fn export_solid_step<W: Write>(
        solid: &truck_modeling::Solid,
        mut writer: W,
    ) -> Result<(), StepError> {
        let compressed = solid.compress();
        let step_string = truck_stepio::out::CompleteStepDisplay::new(
            truck_stepio::out::StepModel::from(&compressed),
            truck_stepio::out::StepHeaderDescriptor {
                organization_system: "inventor-rs Clean-Room CAD".into(),
                ..Default::default()
            },
        )
        .to_string();

        writer.write_all(step_string.as_bytes())?;
        Ok(())
    }
}

pub struct StepImporter;

impl StepImporter {
    /// Imports a solid from an ISO 10303-21 STEP format string, parses entities,
    /// reconstructs topological B-rep shells, and tessellates to a CadMesh.
    pub fn import_solid_step(
        step_str: &str,
        tolerance: f64,
    ) -> Result<StepImportResult, StepError> {
        let table = Table::from_step(step_str).ok_or_else(|| {
            StepError::ParseFailed("Failed to parse STEP ISO-10303-21 exchange structure".into())
        })?;

        if table.shell.is_empty() {
            return Err(StepError::ParseFailed(
                "No B-rep Shell entities found in STEP data".into(),
            ));
        }

        let mut composite_mesh = CadMesh::new();
        let mut total_faces = 0usize;
        let mut total_edges = 0usize;
        let mut total_vertices = 0usize;
        let shell_count = table.shell.len();

        for shell_holder in table.shell.values() {
            let cshell = table.to_compressed_shell(shell_holder).map_err(|e| {
                StepError::TopologyError(format!("Failed to build CompressedShell: {}", e))
            })?;

            total_faces += cshell.faces.len();
            total_edges += cshell.edges.len();
            total_vertices += cshell.vertices.len();

            let shell = truck_topology::Shell::extract(cshell).map_err(|e| {
                StepError::TopologyError(format!("Failed to extract B-rep Shell: {}", e))
            })?;

            let solid = truck_topology::Solid::try_new(vec![shell]).map_err(|e| {
                StepError::TopologyError(format!("Failed to construct B-rep Solid: {}", e))
            })?;

            let tessellated = solid.triangulation(tolerance);
            let poly_mesh = tessellated.to_polygon();
            let mut mesh = inventor_kernel::polygon_mesh_to_cad_mesh(&poly_mesh);

            // Extract feature edge lines from tessellated solid edges
            for edge in tessellated.edge_iter() {
                let curve = edge.oriented_curve();
                let polyline = &curve.0;
                let start_idx = mesh.positions.len() as u32;
                for pt in polyline {
                    mesh.positions.push([pt.x as f32, pt.y as f32, pt.z as f32]);
                    mesh.normals.push([0.0, 1.0, 0.0]);
                }
                for idx in 0..polyline.len().saturating_sub(1) {
                    mesh.edge_indices.push(start_idx + idx as u32);
                    mesh.edge_indices.push(start_idx + idx as u32 + 1);
                }
            }

            composite_mesh.append(&mesh);
        }

        let quality = composite_mesh.inspect_watertight();

        Ok(StepImportResult {
            metadata: StepImportMetadata {
                shell_count,
                face_count: total_faces,
                edge_count: total_edges,
                vertex_count: total_vertices,
            },
            mesh: composite_mesh,
            quality,
        })
    }

    /// Round-trip helper: exports a solid to STEP and immediately imports it back,
    /// returning both the raw STEP string and the verified StepImportResult.
    pub fn round_trip_solid(
        solid: &truck_modeling::Solid,
        tolerance: f64,
    ) -> Result<(String, StepImportResult), StepError> {
        let mut step_bytes = Vec::new();
        StepExporter::export_solid_step(solid, &mut step_bytes)?;
        let step_str = String::from_utf8(step_bytes).map_err(|e| {
            StepError::GenerationFailed(format!("Invalid UTF-8 in STEP output: {}", e))
        })?;
        let result = Self::import_solid_step(&step_str, tolerance)?;
        Ok((step_str, result))
    }
}
