use inventor_kernel::CadMesh;
use std::io::{self, Write};

pub struct StlExporter;

impl StlExporter {
    /// Export CadMesh to ASCII STL
    pub fn export_ascii<W: Write>(mesh: &CadMesh, name: &str, mut writer: W) -> io::Result<()> {
        writeln!(writer, "solid {}", name)?;
        for &[i0_u, i1_u, i2_u] in mesh.indices.as_chunks::<3>().0 {
            let i0 = i0_u as usize;
            let i1 = i1_u as usize;
            let i2 = i2_u as usize;

            let n = if i0 < mesh.normals.len() {
                mesh.normals[i0]
            } else {
                [0.0, 0.0, 1.0]
            };

            let v0 = mesh.positions[i0];
            let v1 = mesh.positions[i1];
            let v2 = mesh.positions[i2];

            writeln!(
                writer,
                "  facet normal {:.6e} {:.6e} {:.6e}",
                n[0], n[1], n[2]
            )?;
            writeln!(writer, "    outer loop")?;
            writeln!(
                writer,
                "      vertex {:.6e} {:.6e} {:.6e}",
                v0[0], v0[1], v0[2]
            )?;
            writeln!(
                writer,
                "      vertex {:.6e} {:.6e} {:.6e}",
                v1[0], v1[1], v1[2]
            )?;
            writeln!(
                writer,
                "      vertex {:.6e} {:.6e} {:.6e}",
                v2[0], v2[1], v2[2]
            )?;
            writeln!(writer, "    endloop")?;
            writeln!(writer, "  endfacet")?;
        }
        writeln!(writer, "endsolid {}", name)?;
        Ok(())
    }

    /// Export CadMesh to Binary STL
    pub fn export_binary<W: Write>(mesh: &CadMesh, mut writer: W) -> io::Result<()> {
        // 80-byte header
        let header = [0u8; 80];
        writer.write_all(&header)?;

        // Number of triangles (u32 little-endian)
        let tri_count = (mesh.indices.len() / 3) as u32;
        writer.write_all(&tri_count.to_le_bytes())?;

        for &[i0_u, i1_u, i2_u] in mesh.indices.as_chunks::<3>().0 {
            let i0 = i0_u as usize;
            let i1 = i1_u as usize;
            let i2 = i2_u as usize;

            let n = if i0 < mesh.normals.len() {
                mesh.normals[i0]
            } else {
                [0.0, 0.0, 1.0]
            };

            let v0 = mesh.positions[i0];
            let v1 = mesh.positions[i1];
            let v2 = mesh.positions[i2];

            // Normal (3 x f32)
            for val in n {
                writer.write_all(&val.to_le_bytes())?;
            }
            // Vertex 1
            for val in v0 {
                writer.write_all(&val.to_le_bytes())?;
            }
            // Vertex 2
            for val in v1 {
                writer.write_all(&val.to_le_bytes())?;
            }
            // Vertex 3
            for val in v2 {
                writer.write_all(&val.to_le_bytes())?;
            }
            // Attribute byte count (u16 = 0)
            writer.write_all(&0u16.to_le_bytes())?;
        }

        Ok(())
    }
}
