use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Analytical inspection report for verifying closed (watertight) 2-manifold CAD mesh topology.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshQualityReport {
    /// True if every edge in the mesh is shared by exactly two triangles with opposing orientations (watertight).
    pub is_watertight: bool,
    /// True if no edge is shared by more than two triangles.
    pub is_manifold: bool,
    /// Total number of triangular faces.
    pub triangle_count: usize,
    /// Total number of unique geometric vertices after welding coincidences.
    pub unique_vertex_count: usize,
    /// Total number of unclosed boundary (open border) edges.
    pub boundary_edge_count: usize,
    /// Total number of non-manifold edges (> 2 incident triangles).
    pub non_manifold_edge_count: usize,
    /// Computed signed volume in cubic millimeters (positive for outward-oriented shells).
    pub volume: f64,
    /// Computed total surface area in square millimeters.
    pub surface_area: f64,
    /// Center of mass / centroid [X, Y, Z] in millimeters.
    pub centroid: [f64; 3],
    /// Bounding box dimensions [dX, dY, dZ] in millimeters.
    pub dimensions: [f64; 3],
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CadMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub edge_indices: Vec<u32>, // Line segments (pairs of indices) for CAD feature edges
}

impl CadMesh {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    pub fn edge_count(&self) -> usize {
        self.edge_indices.len() / 2
    }

    pub fn compute_bounding_box(&self) -> Option<([f32; 3], [f32; 3])> {
        if self.positions.is_empty() {
            return None;
        }
        let mut min = self.positions[0];
        let mut max = self.positions[0];
        for p in &self.positions[1..] {
            min[0] = min[0].min(p[0]);
            min[1] = min[1].min(p[1]);
            min[2] = min[2].min(p[2]);
            max[0] = max[0].max(p[0]);
            max[1] = max[1].max(p[1]);
            max[2] = max[2].max(p[2]);
        }
        Some((min, max))
    }

    /// Performs an analytical closed-mesh inspection checking 2-manifold topology,
    /// boundary edge gaps, orientation consistency, and computing exact volume and area.
    pub fn inspect_watertight(&self) -> MeshQualityReport {
        let tri_count = self.triangle_count();
        if tri_count == 0 || self.positions.is_empty() {
            return MeshQualityReport {
                is_watertight: false,
                is_manifold: true,
                triangle_count: 0,
                unique_vertex_count: 0,
                boundary_edge_count: 0,
                non_manifold_edge_count: 0,
                volume: 0.0,
                surface_area: 0.0,
                centroid: [0.0, 0.0, 0.0],
                dimensions: [0.0, 0.0, 0.0],
            };
        }

        // 1. Cluster coincident vertices within 1 micron tolerance (0.001 mm)
        let mut pos_to_welded = HashMap::new();
        let mut weld_map = Vec::with_capacity(self.positions.len());
        let mut unique_vertices = 0usize;

        for p in &self.positions {
            let key = (
                (p[0] * 1000.0).round() as i64,
                (p[1] * 1000.0).round() as i64,
                (p[2] * 1000.0).round() as i64,
            );
            let id = *pos_to_welded.entry(key).or_insert_with(|| {
                let curr = unique_vertices;
                unique_vertices += 1;
                curr
            });
            weld_map.push(id);
        }

        // 2. Build edge incidence maps
        let mut directed_edges: HashMap<(usize, usize), usize> = HashMap::new();
        let mut undirected_edges: HashMap<(usize, usize), usize> = HashMap::new();

        for &[i0, i1, i2] in self.indices.as_chunks::<3>().0 {
            let w0 = weld_map[i0 as usize];
            let w1 = weld_map[i1 as usize];
            let w2 = weld_map[i2 as usize];

            // Ignore collapsed/degenerate triangles
            if w0 == w1 || w1 == w2 || w2 == w0 {
                continue;
            }

            let half_edges = [(w0, w1), (w1, w2), (w2, w0)];
            for (u, v) in half_edges {
                *directed_edges.entry((u, v)).or_insert(0) += 1;
                let pair = if u < v { (u, v) } else { (v, u) };
                *undirected_edges.entry(pair).or_insert(0) += 1;
            }
        }

        // 3. Inspect manifold and watertight conditions
        let mut boundary_edge_count = 0usize;
        let mut non_manifold_edge_count = 0usize;
        let mut inconsistent_orientation_count = 0usize;

        for (&(u, v), &count) in &undirected_edges {
            if count == 1 {
                boundary_edge_count += 1;
            } else if count > 2 {
                non_manifold_edge_count += 1;
            } else if count == 2 {
                let forward = directed_edges.get(&(u, v)).copied().unwrap_or(0);
                let backward = directed_edges.get(&(v, u)).copied().unwrap_or(0);
                if forward != 1 || backward != 1 {
                    inconsistent_orientation_count += 1;
                }
            }
        }

        let is_manifold = non_manifold_edge_count == 0;
        let is_watertight =
            is_manifold && boundary_edge_count == 0 && inconsistent_orientation_count == 0;

        // 4. Compute Volume via Divergence Theorem and Surface Area
        let mut signed_vol_sum = 0.0f64;
        let mut surface_area = 0.0f64;
        let mut weighted_centroid = [0.0f64; 3];

        for &[i0, i1, i2] in self.indices.as_chunks::<3>().0 {
            let p0 = self.positions[i0 as usize];
            let p1 = self.positions[i1 as usize];
            let p2 = self.positions[i2 as usize];

            let ax = p0[0] as f64;
            let ay = p0[1] as f64;
            let az = p0[2] as f64;

            let bx = p1[0] as f64;
            let by = p1[1] as f64;
            let bz = p1[2] as f64;

            let cx = p2[0] as f64;
            let cy = p2[1] as f64;
            let cz = p2[2] as f64;

            // Signed volume of tetrahedron from origin: (A . (B x C)) / 6
            let tet_vol =
                (ax * (by * cz - bz * cy) + ay * (bz * cx - bx * cz) + az * (bx * cy - by * cx))
                    / 6.0;
            signed_vol_sum += tet_vol;

            let tet_centroid = [
                (ax + bx + cx) * 0.25,
                (ay + by + cy) * 0.25,
                (az + bz + cz) * 0.25,
            ];
            weighted_centroid[0] += tet_centroid[0] * tet_vol;
            weighted_centroid[1] += tet_centroid[1] * tet_vol;
            weighted_centroid[2] += tet_centroid[2] * tet_vol;

            // Surface Area via cross product
            let e1 = [bx - ax, by - ay, bz - az];
            let e2 = [cx - ax, cy - ay, cz - az];
            let cross = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let tri_area =
                0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
            surface_area += tri_area;
        }

        let volume = signed_vol_sum.abs();
        let centroid = if volume > 1e-6 {
            [
                weighted_centroid[0] / signed_vol_sum,
                weighted_centroid[1] / signed_vol_sum,
                weighted_centroid[2] / signed_vol_sum,
            ]
        } else if let Some((min, max)) = self.compute_bounding_box() {
            [
                (min[0] + max[0]) as f64 * 0.5,
                (min[1] + max[1]) as f64 * 0.5,
                (min[2] + max[2]) as f64 * 0.5,
            ]
        } else {
            [0.0, 0.0, 0.0]
        };

        let dimensions = if let Some((min, max)) = self.compute_bounding_box() {
            [
                (max[0] - min[0]).abs() as f64,
                (max[1] - min[1]).abs() as f64,
                (max[2] - min[2]).abs() as f64,
            ]
        } else {
            [0.0, 0.0, 0.0]
        };

        MeshQualityReport {
            is_watertight,
            is_manifold,
            triangle_count: tri_count,
            unique_vertex_count: unique_vertices,
            boundary_edge_count,
            non_manifold_edge_count,
            volume,
            surface_area,
            centroid,
            dimensions,
        }
    }

    /// Transforms all mesh positions and normals according to an origin and 3D basis frame.
    ///
    /// Given local plane coordinates $(u, v, w)$, maps to:
    /// $P_{\text{world}} = \text{origin} + u \cdot X + v \cdot Y + w \cdot Z_{\text{normal}}$
    pub fn transform_by_basis(
        &mut self,
        origin: [f32; 3],
        x_axis: [f32; 3],
        y_axis: [f32; 3],
        normal: [f32; 3],
    ) {
        for p in &mut self.positions {
            let u = p[0];
            let v = p[1];
            let w = p[2];
            p[0] = origin[0] + u * x_axis[0] + v * y_axis[0] + w * normal[0];
            p[1] = origin[1] + u * x_axis[1] + v * y_axis[1] + w * normal[1];
            p[2] = origin[2] + u * x_axis[2] + v * y_axis[2] + w * normal[2];
        }
        for n in &mut self.normals {
            let nu = n[0];
            let nv = n[1];
            let nw = n[2];
            n[0] = nu * x_axis[0] + nv * y_axis[0] + nw * normal[0];
            n[1] = nu * x_axis[1] + nv * y_axis[1] + nw * normal[1];
            n[2] = nu * x_axis[2] + nv * y_axis[2] + nw * normal[2];
        }
    }

    /// Appends another CadMesh into this CadMesh (merging vertex buffers and edge indices).
    pub fn append(&mut self, other: &CadMesh) {
        let base_vertex = self.positions.len() as u32;
        self.positions.extend_from_slice(&other.positions);
        self.normals.extend_from_slice(&other.normals);
        for idx in &other.indices {
            self.indices.push(base_vertex + idx);
        }
        for edge_idx in &other.edge_indices {
            self.edge_indices.push(base_vertex + edge_idx);
        }
    }
}

/// Converts a truck PolygonMesh into a CadMesh with triangles and feature edge lines.
pub fn polygon_mesh_to_cad_mesh(mesh: &truck_polymesh::PolygonMesh) -> CadMesh {
    let mut cad_mesh = CadMesh::new();
    for pos in mesh.positions() {
        cad_mesh
            .positions
            .push([pos.x as f32, pos.y as f32, pos.z as f32]);
    }

    for tri in mesh.tri_faces() {
        cad_mesh.indices.push(tri[0].pos as u32);
        cad_mesh.indices.push(tri[1].pos as u32);
        cad_mesh.indices.push(tri[2].pos as u32);
    }

    for quad in mesh.quad_faces() {
        cad_mesh.indices.push(quad[0].pos as u32);
        cad_mesh.indices.push(quad[1].pos as u32);
        cad_mesh.indices.push(quad[2].pos as u32);

        cad_mesh.indices.push(quad[0].pos as u32);
        cad_mesh.indices.push(quad[2].pos as u32);
        cad_mesh.indices.push(quad[3].pos as u32);
    }

    cad_mesh
        .normals
        .resize(cad_mesh.positions.len(), [0.0, 0.0, 0.0]);
    for &[i0_u, i1_u, i2_u] in cad_mesh.indices.as_chunks::<3>().0 {
        let i0 = i0_u as usize;
        let i1 = i1_u as usize;
        let i2 = i2_u as usize;

        let p0 = glam::Vec3::from(cad_mesh.positions[i0]);
        let p1 = glam::Vec3::from(cad_mesh.positions[i1]);
        let p2 = glam::Vec3::from(cad_mesh.positions[i2]);

        let normal = (p1 - p0).cross(p2 - p0).normalize_or_zero();
        for &idx in &[i0, i1, i2] {
            cad_mesh.normals[idx][0] += normal.x;
            cad_mesh.normals[idx][1] += normal.y;
            cad_mesh.normals[idx][2] += normal.z;
        }
    }

    for n in &mut cad_mesh.normals {
        let v = glam::Vec3::from(*n).normalize_or_zero();
        *n = [v.x, v.y, v.z];
    }

    cad_mesh
}
