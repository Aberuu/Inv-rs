use crate::mesh::CadMesh;
use crate::traits::{GeometryKernel, KernelError, SolidShape};
use inventor_core::BooleanOp;
use std::result::Result as StdResult;
use truck_meshalgo::tessellation::{MeshableShape, MeshedShape};
use truck_modeling::{Matrix4, Point3, Rad, Solid, Vector3, Vector4, Vertex, Wire, builder};

#[derive(Debug, Clone)]
pub struct TruckKernel {
    pub tolerance: f64,
}

impl Default for TruckKernel {
    fn default() -> Self {
        Self { tolerance: 0.01 }
    }
}

impl TruckKernel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Converts a truck Solid into a CadMesh with triangles and feature edges
    pub fn tessellate_solid(&self, solid: &Solid) -> StdResult<CadMesh, KernelError> {
        let tessellated = solid.triangulation(self.tolerance);
        let mesh = tessellated.to_polygon();
        let mut cad_mesh = crate::mesh::polygon_mesh_to_cad_mesh(&mesh);

        // Feature edges from tessellated shape edges
        for edge in tessellated.edge_iter() {
            let curve = edge.oriented_curve();
            let polyline = &curve.0;
            let start_idx = cad_mesh.positions.len() as u32;
            for pt in polyline {
                cad_mesh
                    .positions
                    .push([pt.x as f32, pt.y as f32, pt.z as f32]);
                cad_mesh.normals.push([0.0, 1.0, 0.0]);
            }
            let end_idx = cad_mesh.positions.len() as u32;
            for i in start_idx..end_idx.saturating_sub(1) {
                cad_mesh.edge_indices.push(i);
                cad_mesh.edge_indices.push(i + 1);
            }
        }

        Ok(cad_mesh)
    }

    pub fn build_box_solid(&self, dx: f64, dy: f64, dz: f64) -> Solid {
        let v = builder::vertex(Point3::new(-dx / 2.0, -dy / 2.0, -dz / 2.0));
        let e = builder::tsweep(&v, Vector3::unit_x() * dx);
        let f = builder::tsweep(&e, Vector3::unit_y() * dy);
        builder::tsweep(&f, Vector3::unit_z() * dz)
    }

    pub fn build_cylinder_solid(&self, radius: f64, height: f64) -> StdResult<Solid, KernelError> {
        let v = builder::vertex(Point3::new(radius, 0.0, 0.0));
        let wire = builder::rsweep(
            &v,
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_z(),
            Rad(std::f64::consts::TAU),
        );
        let face = builder::try_attach_plane(&[wire])
            .map_err(|e| KernelError::OperationFailed(format!("{:?}", e)))?;
        Ok(builder::tsweep(&face, Vector3::unit_z() * height))
    }

    pub fn build_extrude_solid(
        &self,
        points: &[[f64; 2]],
        height: f64,
    ) -> StdResult<Solid, KernelError> {
        if points.len() < 3 {
            return Err(KernelError::OperationFailed(
                "Polygon requires at least 3 points".into(),
            ));
        }

        let vertices: Vec<Vertex> = points
            .iter()
            .map(|p| builder::vertex(Point3::new(p[0], p[1], 0.0)))
            .collect();

        let mut edges = Vec::new();
        for i in 0..vertices.len() {
            let next = (i + 1) % vertices.len();
            edges.push(builder::line(&vertices[i], &vertices[next]));
        }

        let wire: Wire = edges.into();
        let face = builder::try_attach_plane(&[wire])
            .map_err(|e| KernelError::OperationFailed(format!("{:?}", e)))?;
        Ok(builder::tsweep(&face, Vector3::unit_z() * height))
    }

    pub fn build_revolve_solid(
        &self,
        points: &[[f64; 2]],
        origin: [f64; 3],
        axis: [f64; 3],
        angle_rad: f64,
    ) -> StdResult<Solid, KernelError> {
        if points.len() < 3 {
            return Err(KernelError::OperationFailed(
                "Revolve polygon requires at least 3 points".into(),
            ));
        }

        let vertices: Vec<Vertex> = points
            .iter()
            .map(|p| builder::vertex(Point3::new(p[0], p[1], 0.0)))
            .collect();

        let mut edges = Vec::new();
        for i in 0..vertices.len() {
            let next = (i + 1) % vertices.len();
            edges.push(builder::line(&vertices[i], &vertices[next]));
        }

        let wire: Wire = edges.into();
        let face = builder::try_attach_plane(&[wire])
            .map_err(|e| KernelError::OperationFailed(format!("{:?}", e)))?;

        let axis_vec = Vector3::new(axis[0], axis[1], axis[2]);
        let axis_len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        let normalized_axis = if axis_len > 1e-6 {
            axis_vec / axis_len
        } else {
            Vector3::unit_y()
        };

        Ok(builder::rsweep(
            &face,
            Point3::new(origin[0], origin[1], origin[2]),
            normalized_axis,
            Rad(angle_rad),
        ))
    }

    pub fn build_shape_solid(&self, shape: &SolidShape) -> StdResult<Solid, KernelError> {
        match shape {
            SolidShape::Box { dx, dy, dz } => Ok(self.build_box_solid(*dx, *dy, *dz)),
            SolidShape::Cylinder { radius, height } => self.build_cylinder_solid(*radius, *height),
            SolidShape::Extrude {
                points,
                height,
                origin,
                x_axis,
                y_axis,
                normal,
            } => {
                let local_solid = self.build_extrude_solid(points, *height)?;
                let mat = Matrix4::from_cols(
                    Vector4::new(x_axis[0], x_axis[1], x_axis[2], 0.0),
                    Vector4::new(y_axis[0], y_axis[1], y_axis[2], 0.0),
                    Vector4::new(normal[0], normal[1], normal[2], 0.0),
                    Vector4::new(origin[0], origin[1], origin[2], 1.0),
                );
                Ok(builder::transformed(&local_solid, mat))
            }
            SolidShape::Revolve {
                points,
                origin,
                axis,
                angle_rad,
            } => self.build_revolve_solid(points, *origin, *axis, *angle_rad),
            SolidShape::Fillet { radius } => Err(KernelError::OperationFailed(format!(
                "3D B-rep edge fillet (r={:.1} mm) is unsupported in truck kernel (evaluated in T8). Secondary kernel (OpenCASCADE) required.",
                radius
            ))),
            SolidShape::Chamfer { distance } => Err(KernelError::OperationFailed(format!(
                "3D B-rep edge chamfer (d={:.1} mm) is unsupported in truck kernel (evaluated in T8). Secondary kernel (OpenCASCADE) required.",
                distance
            ))),
        }
    }

    pub fn boolean_solids(
        &self,
        op: BooleanOp,
        solid_a: &Solid,
        solid_b: &Solid,
    ) -> StdResult<Solid, KernelError> {
        let tol = self.tolerance;
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match op {
            BooleanOp::Union => truck_shapeops::or(solid_a, solid_b, tol),
            BooleanOp::Intersect => truck_shapeops::and(solid_a, solid_b, tol),
            BooleanOp::Subtract => {
                let mut cut_b = solid_b.clone();
                cut_b.not();
                truck_shapeops::and(solid_a, &cut_b, tol)
            }
        }));

        match res {
            Ok(Some(solid)) => Ok(solid),
            Ok(None) => Err(KernelError::BooleanFailed(format!(
                "Boolean {:?} operation returned empty result or failed",
                op
            ))),
            Err(_) => Err(KernelError::BooleanFailed(format!(
                "Boolean {:?} operation encountered internal topology error",
                op
            ))),
        }
    }
}

impl GeometryKernel for TruckKernel {
    fn kernel_name(&self) -> &'static str {
        "truck (Pure Rust B-rep)"
    }

    fn supports_brep_fillet(&self) -> bool {
        false
    }

    fn create_box(&self, dx: f64, dy: f64, dz: f64) -> StdResult<CadMesh, KernelError> {
        let solid = self.build_box_solid(dx, dy, dz);
        self.tessellate_solid(&solid)
    }

    fn create_cylinder(&self, radius: f64, height: f64) -> StdResult<CadMesh, KernelError> {
        let solid = self.build_cylinder_solid(radius, height)?;
        self.tessellate_solid(&solid)
    }

    fn extrude_polygon(&self, points: &[[f64; 2]], height: f64) -> StdResult<CadMesh, KernelError> {
        let solid = self.build_extrude_solid(points, height)?;
        self.tessellate_solid(&solid)
    }

    fn revolve_polygon(
        &self,
        points: &[[f64; 2]],
        origin: [f64; 3],
        axis: [f64; 3],
        angle_rad: f64,
    ) -> StdResult<CadMesh, KernelError> {
        let solid = self.build_revolve_solid(points, origin, axis, angle_rad)?;
        self.tessellate_solid(&solid)
    }

    fn boolean_shapes(
        &self,
        op: BooleanOp,
        shape_a: &SolidShape,
        shape_b: &SolidShape,
    ) -> StdResult<CadMesh, KernelError> {
        let solid_a = self.build_shape_solid(shape_a)?;
        let solid_b = self.build_shape_solid(shape_b)?;
        let result_solid = self.boolean_solids(op, &solid_a, &solid_b)?;
        self.tessellate_solid(&result_solid)
    }

    fn build_solid_mesh(
        &self,
        shapes_with_operations: &[(SolidShape, Option<BooleanOp>)],
    ) -> StdResult<CadMesh, KernelError> {
        if shapes_with_operations.is_empty() {
            return Err(KernelError::OperationFailed(
                "No solid shapes provided to build composite solid".into(),
            ));
        }

        let mut current_solid = self.build_shape_solid(&shapes_with_operations[0].0)?;
        let mut compound_solids: Vec<Solid> = Vec::new();

        for (shape, op) in shapes_with_operations.iter().skip(1) {
            let next_solid = self.build_shape_solid(shape)?;
            let bool_op = op.unwrap_or(BooleanOp::Union);
            match self.boolean_solids(bool_op, &current_solid, &next_solid) {
                Ok(combined) => {
                    current_solid = combined;
                }
                Err(e) => {
                    if bool_op == BooleanOp::Union {
                        // Disjoint or boundary-touching solid bodies form a compound solid
                        compound_solids.push(next_solid);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        let mut mesh = self.tessellate_solid(&current_solid)?;
        for extra in compound_solids {
            let extra_mesh = self.tessellate_solid(&extra)?;
            mesh.append(&extra_mesh);
        }

        Ok(mesh)
    }

    fn fillet_solid(&self, _mesh: &CadMesh, radius: f64) -> StdResult<CadMesh, KernelError> {
        Err(KernelError::OperationFailed(format!(
            "3D B-rep edge fillet (r={:.1} mm) is unsupported in pure-Rust truck kernel (evaluated in T8). Secondary kernel (OpenCASCADE) required for 3D edge blending.",
            radius
        )))
    }

    fn chamfer_solid(&self, _mesh: &CadMesh, distance: f64) -> StdResult<CadMesh, KernelError> {
        Err(KernelError::OperationFailed(format!(
            "3D B-rep edge chamfer (d={:.1} mm) is unsupported in pure-Rust truck kernel (evaluated in T8). Secondary kernel (OpenCASCADE) required for 3D edge beveling.",
            distance
        )))
    }
}
