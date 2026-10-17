use crate::camera::CadCamera;
use crate::grid::LineVertex;
use crate::pipeline::{CameraUniform, LightUniform, MeshVertex};

use inventor_kernel::CadMesh;
use log::info;
use wgpu::util::DeviceExt;

/// GPU buffers holding tessellated 3D solid geometry and sharp feature edges.
pub struct GpuMeshBuffers {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub edge_vertex_buffer: Option<wgpu::Buffer>,
    pub edge_vertex_count: u32,
}

/// GPU buffer holding wireframe lines (ground grid, datum axes).
pub struct GpuLineBuffers {
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
}

/// Hardware-accelerated CAD renderer using wgpu.
/// Manages graphics pipelines, shaders, uniform buffers, and hardware depth testing.
pub struct WgpuCadRenderer {
    pub solid_pipeline: wgpu::RenderPipeline,
    pub line_pipeline: wgpu::RenderPipeline,
    pub camera_buffer: wgpu::Buffer,
    pub light_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub target_format: wgpu::TextureFormat,
    pub depth_format: wgpu::TextureFormat,
    pub depth_texture: Option<wgpu::Texture>,
    pub depth_view: Option<wgpu::TextureView>,
    pub depth_size: (u32, u32),
}

impl WgpuCadRenderer {
    pub const DEFAULT_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// Creates a new CAD renderer with default Depth32Float depth buffer.
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        Self::new_with_depth_format(device, target_format, Self::DEFAULT_DEPTH_FORMAT)
    }

    /// Creates a new CAD renderer with a specified target format and depth format.
    pub fn new_with_depth_format(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        info!(
            "Initializing WgpuCadRenderer (Target: {:?}, Depth: {:?})",
            target_format, depth_format
        );

        // 1. Load WGSL Shaders
        let mesh_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("CAD Mesh Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/cad_mesh.wgsl").into()),
        });

        let line_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("CAD Line Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/cad_line.wgsl").into()),
        });

        // 2. Uniform Buffers (Camera & Light)
        let initial_camera = CameraUniform::default();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[initial_camera]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let initial_light = LightUniform::default();
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Uniform Buffer"),
            contents: bytemuck::cast_slice(&[initial_light]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // 3. Bind Group Layout & Bind Group
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("CAD Renderer Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("CAD Renderer Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("CAD Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        // 4. Solid Mesh Render Pipeline
        let solid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Solid Mesh Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &mesh_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(MeshVertex::desc())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &mesh_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Double-sided rendering for open/closed B-rep surfaces
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                // Bias pushed solid surfaces slightly back so wireframe lines don't z-fight
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // 5. Line Render Pipeline (Edges, Grid, Axes)
        let line_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &line_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(LineVertex::desc())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &line_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            solid_pipeline,
            line_pipeline,
            camera_buffer,
            light_buffer,
            bind_group,
            target_format,
            depth_format,
            depth_texture: None,
            depth_view: None,
            depth_size: (0, 0),
        }
    }

    /// Resizes or initializes the depth texture if dimensions have changed.
    pub fn ensure_depth_texture(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> &wgpu::TextureView {
        let width = width.max(1);
        let height = height.max(1);

        if self.depth_texture.is_none() || self.depth_size != (width, height) {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("CAD Viewport Depth Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.depth_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.depth_texture = Some(texture);
            self.depth_view = Some(view);
            self.depth_size = (width, height);
        }

        self.depth_view.as_ref().unwrap()
    }

    /// Updates the camera uniform buffer with the current view-projection matrix and eye position.
    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &CadCamera, aspect: f32) {
        let view_proj = camera.view_proj_matrix(aspect);
        let eye = camera.eye();
        let uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            eye_pos: [eye.x, eye.y, eye.z, 1.0],
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }

    /// Updates lighting parameters.
    pub fn update_light(&self, queue: &wgpu::Queue, light: &LightUniform) {
        queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[*light]));
    }

    /// Creates GPU buffers from a `CadMesh`.
    pub fn create_mesh_buffers(
        &self,
        device: &wgpu::Device,
        mesh: &CadMesh,
        face_color: [f32; 4],
        edge_color: [f32; 4],
    ) -> GpuMeshBuffers {
        let vertex_count = mesh.positions.len();
        let mut vertices = Vec::with_capacity(vertex_count);

        for i in 0..vertex_count {
            let pos = mesh.positions[i];
            let norm = if i < mesh.normals.len() {
                mesh.normals[i]
            } else {
                [0.0, 1.0, 0.0]
            };
            vertices.push(MeshVertex {
                position: pos,
                normal: norm,
                color: face_color,
            });
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create edge lines buffer from feature edges
        let edge_vertex_buffer = if !mesh.edge_indices.is_empty() {
            let mut line_verts = Vec::with_capacity(mesh.edge_indices.len());
            for &[i0, i1] in mesh.edge_indices.as_chunks::<2>().0 {
                let u0 = i0 as usize;
                let u1 = i1 as usize;
                if u0 < mesh.positions.len() && u1 < mesh.positions.len() {
                    line_verts.push(LineVertex {
                        position: mesh.positions[u0],
                        color: edge_color,
                    });
                    line_verts.push(LineVertex {
                        position: mesh.positions[u1],
                        color: edge_color,
                    });
                }
            }
            let buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Mesh Edge Line Buffer"),
                contents: bytemuck::cast_slice(&line_verts),
                usage: wgpu::BufferUsages::VERTEX,
            });
            Some(buf)
        } else {
            None
        };

        let edge_vertex_count = if edge_vertex_buffer.is_some() {
            mesh.edge_indices.len() as u32
        } else {
            0
        };

        GpuMeshBuffers {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
            edge_vertex_buffer,
            edge_vertex_count,
        }
    }

    /// Creates GPU line buffers for wireframes, datum axes, or ground grid.
    pub fn create_line_buffers(
        &self,
        device: &wgpu::Device,
        lines: &[LineVertex],
    ) -> GpuLineBuffers {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Vertex Buffer"),
            contents: bytemuck::cast_slice(lines),
            usage: wgpu::BufferUsages::VERTEX,
        });

        GpuLineBuffers {
            vertex_buffer,
            vertex_count: lines.len() as u32,
        }
    }

    /// Records draw calls for solid mesh, edges, and grid into an active render pass.
    pub fn render<'a>(
        &'a self,
        rpass: &mut wgpu::RenderPass<'a>,
        mesh_buffers: Option<&'a GpuMeshBuffers>,
        grid_buffers: Option<&'a GpuLineBuffers>,
    ) {
        // 1. Draw Grid Lines & Axes (First, under the model)
        if let Some(grid) = grid_buffers
            && grid.vertex_count > 0
        {
            rpass.set_pipeline(&self.line_pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffer(0, grid.vertex_buffer.slice(..));
            rpass.draw(0..grid.vertex_count, 0..1);
        }

        // 2. Draw Solid Faces (With hardware depth testing & bias)
        if let Some(mesh) = mesh_buffers {
            if mesh.index_count > 0 {
                rpass.set_pipeline(&self.solid_pipeline);
                rpass.set_bind_group(0, &self.bind_group, &[]);
                rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                rpass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }

            // 3. Draw Sharp Feature Edges
            if let Some(ref edge_buf) = mesh.edge_vertex_buffer
                && mesh.edge_vertex_count > 0
            {
                rpass.set_pipeline(&self.line_pipeline);
                rpass.set_bind_group(0, &self.bind_group, &[]);
                rpass.set_vertex_buffer(0, edge_buf.slice(..));
                rpass.draw(0..mesh.edge_vertex_count, 0..1);
            }
        }
    }

    /// Renders the scene to an offscreen texture with color and depth attachment.
    #[allow(clippy::too_many_arguments)]
    pub fn render_offscreen(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        camera: &CadCamera,
        mesh: Option<&CadMesh>,
        grid_lines: Option<&[LineVertex]>,
        clear_color: [f64; 4],
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let width = width.max(1);
        let height = height.max(1);
        let aspect = width as f32 / height as f32;

        self.update_camera(queue, camera, aspect);

        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("CAD Offscreen Color Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.target_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.ensure_depth_texture(device, width, height);

        let mesh_gpu = mesh.map(|m| {
            self.create_mesh_buffers(
                device,
                m,
                [0.45, 0.55, 0.70, 1.0], // CAD Slate Steel
                [0.10, 0.12, 0.18, 1.0], // Dark boundary edges
            )
        });

        let grid_gpu = grid_lines.map(|g| self.create_line_buffers(device, g));

        let depth_view = self.depth_view.as_ref().unwrap();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Offscreen CAD Render Encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("CAD Viewport Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color[0],
                            g: clear_color[1],
                            b: clear_color[2],
                            a: clear_color[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.render(&mut rpass, mesh_gpu.as_ref(), grid_gpu.as_ref());
        }

        queue.submit(std::iter::once(encoder.finish()));

        (color_texture, color_view)
    }
}
