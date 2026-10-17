use crate::sketch_session::SketchTool;
use crate::state::AppState;
use egui::{Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use glam::{Vec2 as GVec2, Vec3, Vec4};
use inventor_core::FeatureKind;
use inventor_sketch::ConstraintStatus;

fn render_wgpu_scene(ui: &mut Ui, state: &mut AppState, rect: Rect) -> bool {
    let (Some(renderer_arc), Some(wgpu_state)) = (&state.wgpu_renderer, &state.wgpu_render_state)
    else {
        return false;
    };

    let ppp = ui.ctx().pixels_per_point();
    let px_w = (rect.width() * ppp).round().max(1.0) as u32;
    let px_h = (rect.height() * ppp).round().max(1.0) as u32;

    let is_in_sketch_mode = state.active_sketch.is_some();
    let grid_lines = if state.show_grid && !is_in_sketch_mode {
        Some(inventor_render::GridGenerator::generate_grid(100.0, 10.0))
    } else {
        None
    };

    let (_color_tex, color_view) = {
        let mut renderer = renderer_arc.lock().unwrap();
        renderer.render_offscreen(
            &wgpu_state.device,
            &wgpu_state.queue,
            px_w,
            px_h,
            &state.camera,
            state.active_mesh.as_ref(),
            grid_lines.as_deref(),
            [26.0 / 255.0, 32.0 / 255.0, 44.0 / 255.0, 1.0], // Deep CAD Navy Slate
        )
    };

    let texture_id = if let Some(tex_id) = state.viewport_texture_id {
        if state.viewport_texture_size == (px_w, px_h) {
            wgpu_state
                .renderer
                .write()
                .update_egui_texture_from_wgpu_texture(
                    &wgpu_state.device,
                    &color_view,
                    wgpu::FilterMode::Linear,
                    tex_id,
                );
            tex_id
        } else {
            let new_id = wgpu_state.renderer.write().register_native_texture(
                &wgpu_state.device,
                &color_view,
                wgpu::FilterMode::Linear,
            );
            state.viewport_texture_id = Some(new_id);
            state.viewport_texture_size = (px_w, px_h);
            new_id
        }
    } else {
        let new_id = wgpu_state.renderer.write().register_native_texture(
            &wgpu_state.device,
            &color_view,
            wgpu::FilterMode::Linear,
        );
        state.viewport_texture_id = Some(new_id);
        state.viewport_texture_size = (px_w, px_h);
        new_id
    };

    let painter = ui.painter_at(rect);
    painter.image(
        texture_id,
        rect,
        Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
        Color32::WHITE,
    );

    true
}

pub fn render_viewport(ui: &mut Ui, state: &mut AppState) {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
    let painter = ui.painter_at(rect);

    // 1. Hardware-accelerated wgpu viewport rendering (with graceful fallback)
    let rendered_with_wgpu = render_wgpu_scene(ui, state, rect);
    if !rendered_with_wgpu {
        painter.rect_filled(
            rect,
            0.0,
            Color32::from_rgb(26, 32, 44), // Deep CAD Navy Slate
        );
    }

    let aspect = rect.width() / rect.height().max(1.0);

    // Mouse Interaction: Orbit, Pan, Zoom
    let is_in_sketch_mode = state.active_sketch.is_some();
    let is_drawing_tool = if let Some(sk) = &state.active_sketch {
        matches!(
            sk.active_tool,
            SketchTool::Line | SketchTool::Rectangle | SketchTool::Circle | SketchTool::Point
        )
    } else {
        false
    };

    if response.dragged() {
        let delta = response.drag_delta();
        let input = ui.input(|i| {
            (
                i.modifiers.shift,
                i.modifiers.ctrl,
                i.pointer.button_down(egui::PointerButton::Middle),
                i.pointer.button_down(egui::PointerButton::Secondary),
            )
        });

        let is_middle = input.2;
        let is_right = input.3;
        let shift = input.0;
        let ctrl = input.1;

        if (is_middle && !shift && !ctrl) || (is_right && shift) {
            // Orbit camera
            state.camera.orbit(delta.x, delta.y);
        } else if (is_right && !shift) || (is_middle && ctrl) {
            // Pan camera
            state.camera.pan(delta.x, delta.y);
        }
    }

    // Scroll wheel zoom
    let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
    if scroll_delta != 0.0 && response.hovered() {
        state.camera.zoom(scroll_delta);
    }

    // View-Projection Matrix
    let vp = state.camera.view_proj_matrix(aspect);
    let inv_vp = vp.inverse();

    // Helper: World to Screen space projection
    let world_to_screen = |world_pt: Vec3| -> Option<Pos2> {
        let clip = vp * Vec4::new(world_pt.x, world_pt.y, world_pt.z, 1.0);
        if clip.w <= 0.001 {
            return None; // Behind camera
        }
        let ndc_x = clip.x / clip.w;
        let ndc_y = clip.y / clip.w;

        // Map NDC (-1..1) to screen rect
        let screen_x = rect.left() + (ndc_x + 1.0) * 0.5 * rect.width();
        let screen_y = rect.top() + (1.0 - ndc_y) * 0.5 * rect.height();
        Some(Pos2::new(screen_x, screen_y))
    };

    // Helper: Screen to Plane ray-intersection
    let screen_to_plane = |screen_pos: Pos2,
                           origin: Vec3,
                           normal: Vec3,
                           x_axis: Vec3,
                           y_axis: Vec3|
     -> Option<GVec2> {
        let ndc_x = ((screen_pos.x - rect.left()) / rect.width()) * 2.0 - 1.0;
        let ndc_y = 1.0 - ((screen_pos.y - rect.top()) / rect.height()) * 2.0;

        let p_near_clip = inv_vp * Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
        let p_far_clip = inv_vp * Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

        if p_near_clip.w.abs() < 1e-6 || p_far_clip.w.abs() < 1e-6 {
            return None;
        }

        let p_near = p_near_clip.truncate() / p_near_clip.w;
        let p_far = p_far_clip.truncate() / p_far_clip.w;
        let ray_dir = (p_far - p_near).normalize_or_zero();

        let denom = ray_dir.dot(normal);
        if denom.abs() < 1e-6 {
            return None;
        }

        let t = (origin - p_near).dot(normal) / denom;
        let hit_world = p_near + ray_dir * t;
        let rel = hit_world - origin;
        Some(GVec2::new(rel.dot(x_axis), rel.dot(y_axis)))
    };

    // 2. Render Ground Grid (software fallback when wgpu is inactive)
    if !rendered_with_wgpu && state.show_grid && !is_in_sketch_mode {
        let grid_extent = 100.0_f32;
        let step = 10.0_f32;
        let num_lines = (grid_extent / step) as i32;

        let grid_color = Color32::from_rgba_unmultiplied(100, 116, 139, 45);
        let major_color = Color32::from_rgba_unmultiplied(148, 163, 184, 80);

        for i in -num_lines..=num_lines {
            let pos = i as f32 * step;
            let stroke = Stroke::new(1.0, if i % 5 == 0 { major_color } else { grid_color });

            // Lines parallel to Z
            if let (Some(p1), Some(p2)) = (
                world_to_screen(Vec3::new(pos, 0.0, -grid_extent)),
                world_to_screen(Vec3::new(pos, 0.0, grid_extent)),
            ) {
                painter.line_segment([p1, p2], stroke);
            }

            // Lines parallel to X
            if let (Some(p1), Some(p2)) = (
                world_to_screen(Vec3::new(-grid_extent, 0.0, pos)),
                world_to_screen(Vec3::new(grid_extent, 0.0, pos)),
            ) {
                painter.line_segment([p1, p2], stroke);
            }
        }
    }

    // 3. Render Origin Datum Planes & Axes
    if state.show_axes && !is_in_sketch_mode {
        let origin = Vec3::ZERO;
        let axis_len = 60.0_f32;

        if let Some(p0) = world_to_screen(origin) {
            // X Axis = Red
            if let Some(px) = world_to_screen(Vec3::new(axis_len, 0.0, 0.0)) {
                if !rendered_with_wgpu {
                    painter
                        .line_segment([p0, px], Stroke::new(2.5, Color32::from_rgb(239, 68, 68)));
                }
                painter.text(
                    px,
                    egui::Align2::LEFT_CENTER,
                    "X",
                    egui::FontId::monospace(11.0),
                    Color32::from_rgb(239, 68, 68),
                );
            }
            // Y Axis = Green
            if let Some(py) = world_to_screen(Vec3::new(0.0, axis_len, 0.0)) {
                if !rendered_with_wgpu {
                    painter
                        .line_segment([p0, py], Stroke::new(2.5, Color32::from_rgb(34, 197, 94)));
                }
                painter.text(
                    py,
                    egui::Align2::LEFT_CENTER,
                    "Y",
                    egui::FontId::monospace(11.0),
                    Color32::from_rgb(34, 197, 94),
                );
            }
            // Z Axis = Blue
            if let Some(pz) = world_to_screen(Vec3::new(0.0, 0.0, axis_len)) {
                if !rendered_with_wgpu {
                    painter
                        .line_segment([p0, pz], Stroke::new(2.5, Color32::from_rgb(59, 130, 246)));
                }
                painter.text(
                    pz,
                    egui::Align2::LEFT_CENTER,
                    "Z",
                    egui::FontId::monospace(11.0),
                    Color32::from_rgb(59, 130, 246),
                );
            }
        }

        // Draw Origin Planes if visible
        for feat in &state.doc.features {
            if feat.visible
                && let FeatureKind::OriginPlane { name } = &feat.kind
            {
                let plane_size = 40.0_f32;
                let corners = match name.as_str() {
                    "XY Plane" => [
                        Vec3::new(-plane_size, -plane_size, 0.0),
                        Vec3::new(plane_size, -plane_size, 0.0),
                        Vec3::new(plane_size, plane_size, 0.0),
                        Vec3::new(-plane_size, plane_size, 0.0),
                    ],
                    "XZ Plane" => [
                        Vec3::new(-plane_size, 0.0, -plane_size),
                        Vec3::new(plane_size, 0.0, -plane_size),
                        Vec3::new(plane_size, 0.0, plane_size),
                        Vec3::new(-plane_size, 0.0, plane_size),
                    ],
                    "YZ Plane" => [
                        Vec3::new(0.0, -plane_size, -plane_size),
                        Vec3::new(0.0, plane_size, -plane_size),
                        Vec3::new(0.0, plane_size, plane_size),
                        Vec3::new(0.0, -plane_size, plane_size),
                    ],
                    _ => [Vec3::ZERO; 4],
                };

                let pts: Vec<Pos2> = corners
                    .iter()
                    .filter_map(|&pt| world_to_screen(pt))
                    .collect();
                if pts.len() == 4 {
                    let shape = egui::Shape::convex_polygon(
                        pts,
                        Color32::from_rgba_unmultiplied(56, 189, 248, 25),
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(56, 189, 248, 120)),
                    );
                    painter.add(shape);
                }
            }
        }
    }

    // 4. Render 3D CAD Solid Mesh (software fallback when wgpu is inactive)
    if !rendered_with_wgpu && let Some(mesh) = &state.active_mesh {
        let light_dir = (state.camera.eye() - state.camera.target).normalize();
        let mut triangles_to_draw = Vec::new();

        for &[i0_u, i1_u, i2_u] in mesh.indices.as_chunks::<3>().0 {
            let i0 = i0_u as usize;
            let i1 = i1_u as usize;
            let i2 = i2_u as usize;

            if i0 < mesh.positions.len() && i1 < mesh.positions.len() && i2 < mesh.positions.len() {
                let p0 = Vec3::from(mesh.positions[i0]);
                let p1 = Vec3::from(mesh.positions[i1]);
                let p2 = Vec3::from(mesh.positions[i2]);

                let n0 = if i0 < mesh.normals.len() {
                    Vec3::from(mesh.normals[i0])
                } else {
                    Vec3::Y
                };

                let ndotl = n0.dot(light_dir).max(0.15);
                let shade = (ndotl * 200.0) as u8 + 45;
                let face_color = if is_in_sketch_mode {
                    // Dim solid mesh while sketching
                    Color32::from_rgba_unmultiplied(shade / 3, shade / 3, shade / 3, 100)
                } else {
                    Color32::from_rgb(shade / 2 + 30, shade / 2 + 45, shade + 10)
                };

                if let (Some(s0), Some(s1), Some(s2)) = (
                    world_to_screen(p0),
                    world_to_screen(p1),
                    world_to_screen(p2),
                ) {
                    let eye = state.camera.eye();
                    let depth = (p0.distance_squared(eye)
                        + p1.distance_squared(eye)
                        + p2.distance_squared(eye))
                        / 3.0;

                    triangles_to_draw.push((depth, [s0, s1, s2], face_color));
                }
            }
        }

        triangles_to_draw
            .sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        for (_, pts, color) in triangles_to_draw {
            let shape = egui::Shape::convex_polygon(pts.to_vec(), color, Stroke::NONE);
            painter.add(shape);
        }

        let edge_stroke = Stroke::new(
            1.2,
            if is_in_sketch_mode {
                Color32::from_rgba_unmultiplied(100, 116, 139, 120)
            } else {
                Color32::from_rgb(15, 23, 42)
            },
        );
        for &[i0_u, i1_u] in mesh.edge_indices.as_chunks::<2>().0 {
            let i0 = i0_u as usize;
            let i1 = i1_u as usize;

            if i0 < mesh.positions.len() && i1 < mesh.positions.len() {
                let p0 = Vec3::from(mesh.positions[i0]);
                let p1 = Vec3::from(mesh.positions[i1]);

                if let (Some(s0), Some(s1)) = (world_to_screen(p0), world_to_screen(p1)) {
                    painter.line_segment([s0, s1], edge_stroke);
                }
            }
        }
    }

    // 5. Render Stored Wireframe Sketches (when NOT in sketch mode)
    if !is_in_sketch_mode {
        for stored in state.sketches.values() {
            let plane_id = stored.sketch.plane_id;
            let (origin, x_axis, y_axis) = if plane_id == state.doc.xz_plane_id {
                (Vec3::ZERO, Vec3::X, Vec3::Z)
            } else if plane_id == state.doc.yz_plane_id {
                (Vec3::ZERO, Vec3::Y, Vec3::Z)
            } else {
                (Vec3::ZERO, Vec3::X, Vec3::Y)
            };

            let wire_stroke = Stroke::new(1.8, Color32::from_rgb(56, 189, 248)); // Cyan profile

            for line in &stored.sketch.lines {
                if let (Some(p1), Some(p2)) = (
                    stored.sketch.find_point(line.start),
                    stored.sketch.find_point(line.end),
                ) {
                    let w1 = origin + x_axis * p1.x as f32 + y_axis * p1.y as f32;
                    let w2 = origin + x_axis * p2.x as f32 + y_axis * p2.y as f32;
                    if let (Some(s1), Some(s2)) = (world_to_screen(w1), world_to_screen(w2)) {
                        painter.line_segment([s1, s2], wire_stroke);
                    }
                }
            }

            for circle in &stored.sketch.circles {
                if let Some(center) = stored.sketch.find_point(circle.center) {
                    let segments = 36;
                    let mut prev_screen = None;
                    let mut first_screen = None;

                    for s in 0..=segments {
                        let theta = s as f32 * std::f32::consts::TAU / segments as f32;
                        let px = center.x as f32 + circle.radius as f32 * theta.cos();
                        let py = center.y as f32 + circle.radius as f32 * theta.sin();
                        let w = origin + x_axis * px + y_axis * py;

                        if let Some(sc) = world_to_screen(w) {
                            if let Some(prev) = prev_screen {
                                painter.line_segment([prev, sc], wire_stroke);
                            } else {
                                first_screen = Some(sc);
                            }
                            prev_screen = Some(sc);
                        }
                    }
                    if let (Some(last), Some(first)) = (prev_screen, first_screen) {
                        painter.line_segment([last, first], wire_stroke);
                    }
                }
            }
        }
    }

    // 6. Active 2D Sketch Mode: Plane Grid, Entities, Rubber-banding, & Interactive Picking
    let mut cursor_plane_coords: Option<GVec2> = None;
    let mut clicked_plane_point: Option<GVec2> = None;

    if let Some(session) = state.active_sketch.as_mut() {
        let p_origin = session.plane_origin;
        let p_norm = session.plane_normal;
        let p_x = session.plane_x_axis;
        let p_y = session.plane_y_axis;

        // Render Sketch Plane Grid
        let sketch_grid_extent = 120.0_f32;
        let step = 10.0_f32;
        let num_lines = (sketch_grid_extent / step) as i32;
        let grid_col = Color32::from_rgba_unmultiplied(56, 189, 248, 30);
        let major_col = Color32::from_rgba_unmultiplied(56, 189, 248, 65);

        for i in -num_lines..=num_lines {
            let offset = i as f32 * step;
            let stroke = Stroke::new(1.0, if i % 5 == 0 { major_col } else { grid_col });

            // Parallel to sketch Y axis
            let w1 = p_origin + p_x * offset - p_y * sketch_grid_extent;
            let w2 = p_origin + p_x * offset + p_y * sketch_grid_extent;
            if let (Some(s1), Some(s2)) = (world_to_screen(w1), world_to_screen(w2)) {
                painter.line_segment([s1, s2], stroke);
            }

            // Parallel to sketch X axis
            let w3 = p_origin - p_x * sketch_grid_extent + p_y * offset;
            let w4 = p_origin + p_x * sketch_grid_extent + p_y * offset;
            if let (Some(s1), Some(s2)) = (world_to_screen(w3), world_to_screen(w4)) {
                painter.line_segment([s1, s2], stroke);
            }
        }

        // Render Sketch Plane Axes
        let axis_len = 50.0_f32;
        if let Some(s0) = world_to_screen(p_origin) {
            if let Some(sx) = world_to_screen(p_origin + p_x * axis_len) {
                painter.line_segment([s0, sx], Stroke::new(2.0, Color32::from_rgb(239, 68, 68)));
                painter.text(
                    sx,
                    egui::Align2::LEFT_CENTER,
                    "X",
                    egui::FontId::monospace(10.0),
                    Color32::from_rgb(239, 68, 68),
                );
            }
            if let Some(sy) = world_to_screen(p_origin + p_y * axis_len) {
                painter.line_segment([s0, sy], Stroke::new(2.0, Color32::from_rgb(34, 197, 94)));
                painter.text(
                    sy,
                    egui::Align2::LEFT_CENTER,
                    "Y",
                    egui::FontId::monospace(10.0),
                    Color32::from_rgb(34, 197, 94),
                );
            }
        }

        // Mouse hover on sketch plane
        if let Some(hover_pos) = ui.input(|i| i.pointer.hover_pos())
            && rect.contains(hover_pos)
            && let Some(mut plane_pos) = screen_to_plane(hover_pos, p_origin, p_norm, p_x, p_y)
        {
            // Snap to existing points within 12 screen pixels
            for pt in &session.sketch.points {
                let pt_w = session.plane_to_world(GVec2::new(pt.x as f32, pt.y as f32));
                if let Some(pt_screen) = world_to_screen(pt_w)
                    && pt_screen.distance(hover_pos) < 12.0
                {
                    plane_pos = GVec2::new(pt.x as f32, pt.y as f32);
                    painter.circle_stroke(
                        pt_screen,
                        7.0,
                        Stroke::new(2.0, Color32::from_rgb(250, 204, 21)),
                    );
                    break;
                }
            }

            cursor_plane_coords = Some(plane_pos);

            if ui.input(|i| i.pointer.primary_clicked()) {
                clicked_plane_point = Some(plane_pos);
            }
        }

        // Entity Color coding based on DOF status
        let is_fully_constrained =
            matches!(session.current_dof, ConstraintStatus::FullyConstrained);
        let entity_color = if is_fully_constrained {
            Color32::from_rgb(34, 197, 94) // Autodesk Inventor vibrant green!
        } else {
            Color32::from_rgb(56, 189, 248) // Under-constrained cyan
        };
        let selected_color = Color32::from_rgb(245, 158, 11); // Amber
        let normal_stroke = Stroke::new(2.5, entity_color);
        let selected_stroke = Stroke::new(3.0, selected_color);

        // Render Sketch Lines
        for line in &session.sketch.lines {
            let is_sel = session.selected_lines.contains(&line.id);
            if let (Some(p1), Some(p2)) = (
                session.sketch.find_point(line.start),
                session.sketch.find_point(line.end),
            ) {
                let w1 = session.plane_to_world(GVec2::new(p1.x as f32, p1.y as f32));
                let w2 = session.plane_to_world(GVec2::new(p2.x as f32, p2.y as f32));
                if let (Some(s1), Some(s2)) = (world_to_screen(w1), world_to_screen(w2)) {
                    painter.line_segment(
                        [s1, s2],
                        if is_sel {
                            selected_stroke
                        } else {
                            normal_stroke
                        },
                    );
                }
            }
        }

        // Render Sketch Circles
        for circle in &session.sketch.circles {
            let is_sel = session.selected_circles.contains(&circle.id);
            let stroke = if is_sel {
                selected_stroke
            } else {
                normal_stroke
            };

            if let Some(center) = session.sketch.find_point(circle.center) {
                let segments = 48;
                let mut prev_screen = None;
                let mut first_screen = None;

                for s in 0..=segments {
                    let theta = s as f32 * std::f32::consts::TAU / segments as f32;
                    let px = center.x as f32 + circle.radius as f32 * theta.cos();
                    let py = center.y as f32 + circle.radius as f32 * theta.sin();
                    let w = session.plane_to_world(GVec2::new(px, py));

                    if let Some(sc) = world_to_screen(w) {
                        if let Some(prev) = prev_screen {
                            painter.line_segment([prev, sc], stroke);
                        } else {
                            first_screen = Some(sc);
                        }
                        prev_screen = Some(sc);
                    }
                }
                if let (Some(last), Some(first)) = (prev_screen, first_screen) {
                    painter.line_segment([last, first], stroke);
                }
            }
        }

        // Render Sketch Points (Handles)
        for pt in &session.sketch.points {
            let is_sel = session.selected_points.contains(&pt.id);
            let w = session.plane_to_world(GVec2::new(pt.x as f32, pt.y as f32));
            if let Some(s) = world_to_screen(w) {
                let pt_color = if is_sel {
                    selected_color
                } else if pt.is_fixed || is_fully_constrained {
                    Color32::from_rgb(34, 197, 94)
                } else {
                    Color32::from_rgb(56, 189, 248)
                };
                painter.circle_filled(s, 4.5, pt_color);
                painter.circle_stroke(s, 4.5, Stroke::new(1.0, Color32::from_rgb(15, 23, 42)));
            }
        }

        // Render Live Rubber-band Drawing Previews
        if let (Some(start), Some(cursor)) = (session.pending_start, cursor_plane_coords) {
            let preview_stroke =
                Stroke::new(1.5, Color32::from_rgba_unmultiplied(250, 204, 21, 200));

            match session.active_tool {
                SketchTool::Line => {
                    let w1 = session.plane_to_world(start);
                    let w2 = session.plane_to_world(cursor);
                    if let (Some(s1), Some(s2)) = (world_to_screen(w1), world_to_screen(w2)) {
                        painter.line_segment([s1, s2], preview_stroke);
                        let length = start.distance(cursor);
                        painter.text(
                            Pos2::new((s1.x + s2.x) * 0.5 + 8.0, (s1.y + s2.y) * 0.5 - 8.0),
                            egui::Align2::LEFT_BOTTOM,
                            format!("{:.1} mm", length),
                            egui::FontId::monospace(10.0),
                            Color32::from_rgb(250, 204, 21),
                        );
                    }
                }
                SketchTool::Rectangle => {
                    let x_min = start.x.min(cursor.x);
                    let x_max = start.x.max(cursor.x);
                    let y_min = start.y.min(cursor.y);
                    let y_max = start.y.max(cursor.y);

                    let c0 = session.plane_to_world(GVec2::new(x_min, y_min));
                    let c1 = session.plane_to_world(GVec2::new(x_max, y_min));
                    let c2 = session.plane_to_world(GVec2::new(x_max, y_max));
                    let c3 = session.plane_to_world(GVec2::new(x_min, y_max));

                    if let (Some(s0), Some(s1), Some(s2), Some(s3)) = (
                        world_to_screen(c0),
                        world_to_screen(c1),
                        world_to_screen(c2),
                        world_to_screen(c3),
                    ) {
                        painter.line_segment([s0, s1], preview_stroke);
                        painter.line_segment([s1, s2], preview_stroke);
                        painter.line_segment([s2, s3], preview_stroke);
                        painter.line_segment([s3, s0], preview_stroke);

                        let w = (x_max - x_min).abs();
                        let h = (y_max - y_min).abs();
                        painter.text(
                            s2,
                            egui::Align2::LEFT_TOP,
                            format!("{:.1} x {:.1} mm", w, h),
                            egui::FontId::monospace(10.0),
                            Color32::from_rgb(250, 204, 21),
                        );
                    }
                }
                SketchTool::Circle => {
                    let radius = start.distance(cursor);
                    let segments = 36;
                    let mut prev_sc = None;
                    let mut first_sc = None;

                    for s in 0..=segments {
                        let theta = s as f32 * std::f32::consts::TAU / segments as f32;
                        let px = start.x + radius * theta.cos();
                        let py = start.y + radius * theta.sin();
                        let w = session.plane_to_world(GVec2::new(px, py));

                        if let Some(sc) = world_to_screen(w) {
                            if let Some(prev) = prev_sc {
                                painter.line_segment([prev, sc], preview_stroke);
                            } else {
                                first_sc = Some(sc);
                            }
                            prev_sc = Some(sc);
                        }
                    }
                    if let (Some(last), Some(first)) = (prev_sc, first_sc) {
                        painter.line_segment([last, first], preview_stroke);
                    }

                    if let Some(s_cur) = world_to_screen(session.plane_to_world(cursor)) {
                        painter.text(
                            s_cur,
                            egui::Align2::LEFT_BOTTOM,
                            format!("R{:.1} mm (⌀{:.1})", radius, radius * 2.0),
                            egui::FontId::monospace(10.0),
                            Color32::from_rgb(250, 204, 21),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    // Process Active Sketch Drawing Clicks
    let mut trigger_dim_prompt = false;

    if let (Some(session), Some(click_pt)) = (state.active_sketch.as_mut(), clicked_plane_point) {
        match session.active_tool {
            SketchTool::Line => {
                if let Some(start) = session.pending_start {
                    let p1 = session.add_point(start.x as f64, start.y as f64);
                    let p2 = session.add_point(click_pt.x as f64, click_pt.y as f64);
                    session.add_line(p1, p2);
                    session.pending_start = Some(click_pt);
                    state.status_message = "Line created. Continue line chain or press Esc.".into();
                } else {
                    session.pending_start = Some(click_pt);
                    state.status_message = "Line start point set. Click end point.".into();
                }
            }
            SketchTool::Rectangle => {
                if let Some(start) = session.pending_start.take() {
                    session.add_rectangle(start, click_pt);
                    state.status_message = "Rectangle created with parametric constraints.".into();
                } else {
                    session.pending_start = Some(click_pt);
                    state.status_message = "Corner 1 set. Click opposite corner 2.".into();
                }
            }
            SketchTool::Circle => {
                if let Some(center) = session.pending_start.take() {
                    let radius = center.distance(click_pt) as f64;
                    session.add_circle(center, radius.max(0.5));
                    state.status_message = "Circle created.".into();
                } else {
                    session.pending_start = Some(click_pt);
                    state.status_message = "Center point set. Click to specify radius.".into();
                }
            }
            SketchTool::Point => {
                session.add_point(click_pt.x as f64, click_pt.y as f64);
                state.status_message = "Point placed.".into();
            }
            SketchTool::Select | SketchTool::Dimension => {
                // Select entity closest to click
                let mut found_selection = false;
                for pt in &session.sketch.points {
                    let d = GVec2::new(pt.x as f32, pt.y as f32).distance(click_pt);
                    if d < 3.0 {
                        if session.selected_points.contains(&pt.id) {
                            session.selected_points.retain(|&id| id != pt.id);
                        } else {
                            session.selected_points.push(pt.id);
                        }
                        found_selection = true;
                        break;
                    }
                }

                if !found_selection {
                    for line in &session.sketch.lines {
                        if let (Some(p1), Some(p2)) = (
                            session.sketch.find_point(line.start),
                            session.sketch.find_point(line.end),
                        ) {
                            let a = GVec2::new(p1.x as f32, p1.y as f32);
                            let b = GVec2::new(p2.x as f32, p2.y as f32);
                            let ab = b - a;
                            let len_sq = ab.length_squared();
                            let t = ((click_pt - a).dot(ab) / len_sq).clamp(0.0, 1.0);
                            let proj = a + ab * t;
                            if proj.distance(click_pt) < 3.0 {
                                if session.selected_lines.contains(&line.id) {
                                    session.selected_lines.retain(|&id| id != line.id);
                                } else {
                                    session.selected_lines.push(line.id);
                                }
                                found_selection = true;
                                break;
                            }
                        }
                    }
                }

                if !found_selection {
                    for circle in &session.sketch.circles {
                        if let Some(c) = session.sketch.find_point(circle.center) {
                            let center = GVec2::new(c.x as f32, c.y as f32);
                            let dist = center.distance(click_pt);
                            if (dist - circle.radius as f32).abs() < 3.0 || dist < 3.0 {
                                if session.selected_circles.contains(&circle.id) {
                                    session.selected_circles.retain(|&id| id != circle.id);
                                } else {
                                    session.selected_circles.push(circle.id);
                                }
                                found_selection = true;
                                break;
                            }
                        }
                    }
                }

                if !found_selection {
                    session.clear_selection();
                } else if session.active_tool == SketchTool::Dimension {
                    trigger_dim_prompt = true;
                }
            }
        }
    }

    if trigger_dim_prompt {
        let reg = state.command_registry.clone();
        let _ = reg.execute("sketch.dimension", state);
    }

    // Cancel drawing tool on secondary click or Escape
    if ui.input(|i| i.pointer.secondary_clicked())
        && is_drawing_tool
        && let Some(session) = state.active_sketch.as_mut()
    {
        session.pending_start = None;
        session.active_tool = SketchTool::Select;
        state.status_message = "Cancelled drawing. Select tool active.".into();
    }

    // 7. Interactive ViewCube (Top-Right Corner)
    let cube_size = 72.0;
    let cube_pos = Pos2::new(rect.right() - cube_size - 16.0, rect.top() + 16.0);
    let cube_rect = Rect::from_min_size(cube_pos, Vec2::splat(cube_size));

    painter.rect(
        cube_rect,
        6.0,
        Color32::from_rgba_unmultiplied(30, 41, 59, 210),
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 116, 139, 150)),
        StrokeKind::Inside,
    );

    let home_btn_rect = Rect::from_min_size(
        Pos2::new(cube_rect.left() + 4.0, cube_rect.top() + 4.0),
        Vec2::new(20.0, 20.0),
    );
    painter.text(
        home_btn_rect.center(),
        egui::Align2::CENTER_CENTER,
        "🏠",
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );

    let mouse_pos = ui.input(|i| i.pointer.hover_pos());
    let mouse_clicked = ui.input(|i| i.pointer.primary_clicked());

    if let Some(pos) = mouse_pos
        && home_btn_rect.contains(pos)
        && mouse_clicked
    {
        state.camera.set_home_view();
        state.status_message = "Reset view to Isometric Home (F6).".into();
    }

    let text_color = Color32::from_rgb(226, 232, 240);
    painter.text(
        cube_rect.center(),
        egui::Align2::CENTER_CENTER,
        "TOP\n/ FRONT",
        egui::FontId::monospace(10.0),
        text_color,
    );

    // 8. Active Sketch Mode Overlay HUD (Top-Left Pill & DOF Badge)
    if let Some(session) = &state.active_sketch {
        let hud_pos = Pos2::new(rect.left() + 16.0, rect.top() + 16.0);
        let hud_size = Vec2::new(260.0, 78.0);
        let hud_rect = Rect::from_min_size(hud_pos, hud_size);

        painter.rect(
            hud_rect,
            6.0,
            Color32::from_rgba_unmultiplied(15, 23, 42, 220),
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(56, 189, 248, 100)),
            StrokeKind::Inside,
        );

        painter.text(
            Pos2::new(hud_rect.left() + 10.0, hud_rect.top() + 8.0),
            egui::Align2::LEFT_TOP,
            format!("📐 2D Sketch: {} ({})", session.name, session.plane_name),
            egui::FontId::proportional(12.0),
            Color32::from_rgb(56, 189, 248),
        );

        let tool_name = match session.active_tool {
            SketchTool::Select => "Select",
            SketchTool::Line => "Line",
            SketchTool::Rectangle => "Rectangle",
            SketchTool::Circle => "Circle",
            SketchTool::Point => "Point",
            SketchTool::Dimension => "Dimension",
        };

        let coords_str = if let Some(c) = cursor_plane_coords {
            format!("X: {:.1}, Y: {:.1} mm", c.x, c.y)
        } else {
            "---".into()
        };

        painter.text(
            Pos2::new(hud_rect.left() + 10.0, hud_rect.top() + 28.0),
            egui::Align2::LEFT_TOP,
            format!("Tool: {}  |  {}", tool_name, coords_str),
            egui::FontId::monospace(10.5),
            Color32::from_rgb(226, 232, 240),
        );

        // DOF Status Badge
        let (dof_text, dof_color) = match session.current_dof {
            ConstraintStatus::FullyConstrained => (
                "🟢 Fully Constrained (0 DOF)".to_string(),
                Color32::from_rgb(34, 197, 94),
            ),
            ConstraintStatus::Underconstrained(dof) => (
                format!("🔵 Under-constrained ({} DOF)", dof),
                Color32::from_rgb(56, 189, 248),
            ),
            ConstraintStatus::Overconstrained => (
                "🔴 Over-constrained (Conflict)".to_string(),
                Color32::from_rgb(239, 68, 68),
            ),
        };

        painter.text(
            Pos2::new(hud_rect.left() + 10.0, hud_rect.top() + 50.0),
            egui::Align2::LEFT_TOP,
            dof_text,
            egui::FontId::proportional(11.5),
            dof_color,
        );
    }
}
