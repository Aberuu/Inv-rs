pub mod camera;
pub mod grid;
pub mod pipeline;
pub mod wgpu_renderer;

pub use camera::*;
pub use grid::*;
pub use pipeline::*;
pub use wgpu_renderer::*;

#[cfg(test)]
mod tests {
    use super::*;
    use inventor_kernel::{GeometryKernel, TruckKernel};

    #[test]
    fn test_camera_matrices_and_preset() {
        let mut cam = CadCamera::new();
        cam.set_home_view();

        let aspect = 16.0 / 9.0;
        let view = cam.view_matrix();
        let proj = cam.projection_matrix(aspect);
        let vp = cam.view_proj_matrix(aspect);

        assert!(!view.is_nan());
        assert!(!proj.is_nan());
        assert!(!vp.is_nan());

        // Test ray generation
        let (origin, dir) = cam.screen_to_ray(400.0, 300.0, 800.0, 600.0);
        assert!(!origin.is_nan());
        assert!((dir.length() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_grid_generation() {
        let lines = GridGenerator::generate_grid(50.0, 10.0);
        assert!(!lines.is_empty());
        // Must contain XYZ datum axes (red, green, blue lines)
        assert!(lines.len() >= 6);
    }

    #[test]
    fn test_wgpu_renderer_pipeline_creation_and_offscreen_render() {
        // Create headless wgpu instance and device
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: false,
            compatible_surface: None,
            apply_limit_buckets: false,
        }));

        let adapter = match adapter {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Skipping hardware wgpu test: No compatible GPU adapter found: {e}");
                return;
            }
        };

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("Failed to create headless wgpu device");

        let target_format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let mut renderer = WgpuCadRenderer::new(&device, target_format);

        // Generate test box solid mesh
        let kernel = TruckKernel::new();
        let box_mesh = kernel
            .create_box(30.0, 20.0, 10.0)
            .expect("Create box mesh");
        assert!(!box_mesh.positions.is_empty());

        let mut cam = CadCamera::new();
        cam.set_home_view();

        let grid_lines = GridGenerator::generate_grid(40.0, 10.0);

        // Render offscreen to texture with depth buffer
        let (color_tex, _view) = renderer.render_offscreen(
            &device,
            &queue,
            640,
            480,
            &cam,
            Some(&box_mesh),
            Some(&grid_lines),
            [0.10, 0.12, 0.17, 1.0],
        );

        assert_eq!(color_tex.width(), 640);
        assert_eq!(color_tex.height(), 480);
        assert_eq!(color_tex.format(), target_format);

        // Verify mesh buffer creation directly
        let mesh_gpu = renderer.create_mesh_buffers(
            &device,
            &box_mesh,
            [0.3, 0.5, 0.8, 1.0],
            [0.1, 0.1, 0.1, 1.0],
        );
        assert_eq!(mesh_gpu.index_count, box_mesh.indices.len() as u32);
        assert!(mesh_gpu.edge_vertex_count > 0);
    }
}
