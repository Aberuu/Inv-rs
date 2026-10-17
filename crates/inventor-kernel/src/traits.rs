use crate::mesh::CadMesh;
use inventor_core::BooleanOp;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KernelError {
    #[error("Geometry modeling operation failed: {0}")]
    OperationFailed(String),
    #[error("Tessellation error: {0}")]
    TessellationFailed(String),
    #[error("Boolean error: {0}")]
    BooleanFailed(String),
    #[error("STEP export/import error: {0}")]
    IoError(String),
}

/// Solid shape specification used by the geometric kernel to generate and combine B-rep solids.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SolidShape {
    Box {
        dx: f64,
        dy: f64,
        dz: f64,
    },
    Cylinder {
        radius: f64,
        height: f64,
    },
    Extrude {
        points: Vec<[f64; 2]>,
        height: f64,
        origin: [f64; 3],
        x_axis: [f64; 3],
        y_axis: [f64; 3],
        normal: [f64; 3],
    },
    Revolve {
        points: Vec<[f64; 2]>,
        origin: [f64; 3],
        axis: [f64; 3],
        angle_rad: f64,
    },
    Fillet {
        radius: f64,
    },
    Chamfer {
        distance: f64,
    },
}

pub trait GeometryKernel: Send + Sync {
    /// Returns the descriptive name of the geometric kernel backend.
    fn kernel_name(&self) -> &'static str;

    /// Checks whether this geometric kernel supports 3D B-rep edge filleting and chamfering.
    fn supports_brep_fillet(&self) -> bool;

    /// Create a 3D box primitive (centered or from corner)
    fn create_box(&self, dx: f64, dy: f64, dz: f64) -> Result<CadMesh, KernelError>;

    /// Create a 3D cylinder primitive
    fn create_cylinder(&self, radius: f64, height: f64) -> Result<CadMesh, KernelError>;

    /// Extrude 2D polyline/polygon into 3D solid
    fn extrude_polygon(&self, points: &[[f64; 2]], height: f64) -> Result<CadMesh, KernelError>;

    /// Revolve 2D polyline/polygon around an axis into 3D solid
    fn revolve_polygon(
        &self,
        points: &[[f64; 2]],
        origin: [f64; 3],
        axis: [f64; 3],
        angle_rad: f64,
    ) -> Result<CadMesh, KernelError>;

    /// Perform a boolean operation (Union, Subtract, Intersect) between two solid shapes
    fn boolean_shapes(
        &self,
        op: BooleanOp,
        shape_a: &SolidShape,
        shape_b: &SolidShape,
    ) -> Result<CadMesh, KernelError>;

    /// Build a composite solid from a sequence of shapes and boolean operations
    fn build_solid_mesh(
        &self,
        shapes_with_operations: &[(SolidShape, Option<BooleanOp>)],
    ) -> Result<CadMesh, KernelError>;

    /// Apply an edge fillet (rounding) with given radius to solid geometry.
    fn fillet_solid(&self, mesh: &CadMesh, radius: f64) -> Result<CadMesh, KernelError>;

    /// Apply an edge chamfer (bevel) with given distance to solid geometry.
    fn chamfer_solid(&self, mesh: &CadMesh, distance: f64) -> Result<CadMesh, KernelError>;
}
