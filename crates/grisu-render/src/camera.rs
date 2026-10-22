use glam::camera::rh;
use glam::{Mat4, Vec3};

/// Cámara que orbita alrededor de un punto. Suficiente para el hito 0;
/// la cámara en primera persona llega con el terreno.
#[derive(Clone, Copy, Debug)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    /// Giro horizontal en radianes.
    pub yaw: f32,
    /// Inclinación en radianes (positiva = mirar desde arriba).
    pub pitch: f32,
    pub fov_y: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 3.2,
            yaw: 0.6,
            pitch: 0.5,
            fov_y: 50f32.to_radians(),
        }
    }
}

impl OrbitCamera {
    pub fn eye(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        self.target + Vec3::new(cp * sy, sp, cp * cy) * self.distance
    }

    /// Matriz vista-proyección. wgpu usa la convención de DirectX
    /// (Z de 0 a 1, Y hacia arriba).
    pub fn view_proj(&self, aspect: f32) -> Mat4 {
        let view = rh::view::look_at_mat4(self.eye(), self.target, Vec3::Y);
        let proj = rh::proj::directx::perspective(self.fov_y, aspect.max(0.01), 0.05, 500.0);
        proj * view
    }
}
