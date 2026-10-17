use glam::{Mat4, Vec3, Vec4};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectionType {
    Perspective,
    Orthographic,
}

#[derive(Debug, Clone)]
pub struct CadCamera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,   // Horizontal angle (radians)
    pub pitch: f32, // Vertical angle (radians)
    pub fov_y: f32, // Radians
    pub ortho_size: f32,
    pub projection_type: ProjectionType,
    pub near: f32,
    pub far: f32,
}

impl Default for CadCamera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 120.0,
            yaw: 45.0_f32.to_radians(),
            pitch: 35.264_f32.to_radians(), // Standard isometric pitch
            fov_y: 45.0_f32.to_radians(),
            ortho_size: 100.0,
            projection_type: ProjectionType::Perspective,
            near: 0.1,
            far: 5000.0,
        }
    }
}

impl CadCamera {
    pub fn new() -> Self {
        Self::default()
    }

    /// Eye position computed from target, yaw, pitch, distance
    pub fn eye(&self) -> Vec3 {
        let cos_pitch = self.pitch.cos();
        let sin_pitch = self.pitch.sin();
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();

        let dir = Vec3::new(cos_pitch * sin_yaw, sin_pitch, cos_pitch * cos_yaw);
        self.target + dir * self.distance
    }

    /// View Matrix
    pub fn view_matrix(&self) -> Mat4 {
        glam::camera::rh::view::look_at_mat4(self.eye(), self.target, Vec3::Y)
    }

    /// Projection Matrix for a given viewport aspect ratio (width / height)
    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        match self.projection_type {
            ProjectionType::Perspective => glam::camera::rh::proj::directx::perspective(
                self.fov_y, aspect, self.near, self.far,
            ),
            ProjectionType::Orthographic => {
                let half_h = self.ortho_size / 2.0;
                let half_w = half_h * aspect;
                glam::camera::rh::proj::directx::orthographic(
                    -half_w, half_w, -half_h, half_h, self.near, self.far,
                )
            }
        }
    }

    pub fn view_proj_matrix(&self, aspect: f32) -> Mat4 {
        self.projection_matrix(aspect) * self.view_matrix()
    }

    /// Orbit camera around target
    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        let sensitivity = 0.008;
        self.yaw -= delta_x * sensitivity;
        self.pitch += delta_y * sensitivity;

        // Clamp pitch to avoid gimbal lock / flipping upside down
        let max_pitch = 89.0_f32.to_radians();
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);
    }

    /// Pan camera parallel to the view plane
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let eye = self.eye();
        let forward = (self.target - eye).normalize();
        let right = forward.cross(Vec3::Y).normalize_or_zero();
        let up = right.cross(forward).normalize_or_zero();

        let scale = match self.projection_type {
            ProjectionType::Perspective => self.distance * 0.0015,
            ProjectionType::Orthographic => self.ortho_size * 0.0015,
        };

        let move_vec = (-right * delta_x + up * delta_y) * scale;
        self.target += move_vec;
    }

    /// Zoom in / out (wheel delta)
    pub fn zoom(&mut self, delta: f32) {
        let factor = (1.0 - delta * 0.0015).clamp(0.1, 3.0);
        self.distance = (self.distance * factor).clamp(1.0, 50000.0);
        self.ortho_size = (self.ortho_size * factor).clamp(1.0, 50000.0);
    }

    /// Set camera to standard Isometric Home view
    pub fn set_home_view(&mut self) {
        self.yaw = 45.0_f32.to_radians();
        self.pitch = 35.264_f32.to_radians();
        self.target = Vec3::ZERO;
    }

    /// Set camera to Front view
    pub fn set_front_view(&mut self) {
        self.yaw = 0.0;
        self.pitch = 0.0;
    }

    /// Set camera to Top view
    pub fn set_top_view(&mut self) {
        self.yaw = 0.0;
        self.pitch = 89.9_f32.to_radians();
    }

    /// Set camera to Right view
    pub fn set_right_view(&mut self) {
        self.yaw = 90.0_f32.to_radians();
        self.pitch = 0.0;
    }

    /// Generate world ray from screen coordinates (x, y in viewport pixels)
    pub fn screen_to_ray(
        &self,
        screen_x: f32,
        screen_y: f32,
        width: f32,
        height: f32,
    ) -> (Vec3, Vec3) {
        let ndc_x = (2.0 * screen_x / width) - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_y / height);

        let aspect = width / height.max(1.0);
        let inv_vp = self.view_proj_matrix(aspect).inverse();

        let near_clip = Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
        let far_clip = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

        let mut near_world = inv_vp * near_clip;
        let mut far_world = inv_vp * far_clip;

        near_world /= near_world.w;
        far_world /= far_world.w;

        let origin = near_world.truncate();
        let dir = (far_world.truncate() - origin).normalize();

        (origin, dir)
    }
}
