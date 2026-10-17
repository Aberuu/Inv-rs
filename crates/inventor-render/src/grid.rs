pub use crate::pipeline::LineVertex;

pub struct GridGenerator;

impl GridGenerator {
    /// Generates vertices for the ground plane grid (XZ plane in CAD) and major datum axes
    pub fn generate_grid(extent: f32, step: f32) -> Vec<LineVertex> {
        let mut vertices = Vec::new();

        let grid_color = [0.35, 0.40, 0.45, 0.45];
        let major_grid_color = [0.45, 0.50, 0.58, 0.70];

        let num_lines = (extent / step).ceil() as i32;

        for i in -num_lines..=num_lines {
            let pos = i as f32 * step;
            let is_major = i % 5 == 0;
            let color = if is_major {
                major_grid_color
            } else {
                grid_color
            };

            // Lines parallel to Z axis (along X direction)
            if i != 0 {
                vertices.push(LineVertex {
                    position: [pos, 0.0, -extent],
                    color,
                });
                vertices.push(LineVertex {
                    position: [pos, 0.0, extent],
                    color,
                });

                // Lines parallel to X axis (along Z direction)
                vertices.push(LineVertex {
                    position: [-extent, 0.0, pos],
                    color,
                });
                vertices.push(LineVertex {
                    position: [extent, 0.0, pos],
                    color,
                });
            }
        }

        // Datum Axes:
        // X-Axis = Red
        let red = [0.95, 0.25, 0.25, 1.0];
        vertices.push(LineVertex {
            position: [0.0, 0.0, 0.0],
            color: red,
        });
        vertices.push(LineVertex {
            position: [extent * 1.2, 0.0, 0.0],
            color: red,
        });

        // Y-Axis = Green (Vertical in our coordinate space)
        let green = [0.20, 0.85, 0.35, 1.0];
        vertices.push(LineVertex {
            position: [0.0, 0.0, 0.0],
            color: green,
        });
        vertices.push(LineVertex {
            position: [0.0, extent * 1.2, 0.0],
            color: green,
        });

        // Z-Axis = Blue
        let blue = [0.25, 0.55, 0.95, 1.0];
        vertices.push(LineVertex {
            position: [0.0, 0.0, 0.0],
            color: blue,
        });
        vertices.push(LineVertex {
            position: [0.0, 0.0, extent * 1.2],
            color: blue,
        });

        vertices
    }
}
