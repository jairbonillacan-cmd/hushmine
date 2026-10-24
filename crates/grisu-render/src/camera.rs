use glam::camera::rh;
use glam::{Mat4, Vec3};

/// Plano cercano y lejano de todas las cámaras, en bloques.
const NEAR: f32 = 0.05;
const FAR: f32 = 500.0;

/// Lo que necesita un renderizador para dibujar desde un punto de vista.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub view_proj: Mat4,
    pub eye: Vec3,
}

/// Cámara que orbita alrededor de un punto. Sirve para las capturas desde
/// fuera del mapa.
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
        perspective(self.fov_y, aspect) * view
    }

    pub fn view(&self, aspect: f32) -> View {
        View {
            view_proj: self.view_proj(aspect),
            eye: self.eye(),
        }
    }
}

/// Cámara en primera persona. Con `yaw = 0` mira hacia -Z; `yaw` crece
/// girando hacia la izquierda (de -Z hacia -X).
#[derive(Clone, Copy, Debug)]
pub struct FlyCamera {
    pub position: Vec3,
    pub yaw: f32,
    /// Positivo = mirar hacia arriba. Se limita a ±89°.
    pub pitch: f32,
    pub fov_y: f32,
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            fov_y: 75f32.to_radians(),
        }
    }
}

impl FlyCamera {
    const MAX_PITCH: f32 = 89f32 * std::f32::consts::PI / 180.0;

    pub fn forward(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vec3::new(-sy * cp, sp, -cy * cp)
    }

    /// Dirección horizontal hacia delante (sin inclinación).
    pub fn flat_forward(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        Vec3::new(-sy, 0.0, -cy)
    }

    /// Dirección horizontal hacia la derecha.
    pub fn flat_right(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        Vec3::new(cy, 0.0, -sy)
    }

    /// Gira la cámara; la inclinación queda limitada.
    pub fn rotate(&mut self, d_yaw: f32, d_pitch: f32) {
        self.yaw = (self.yaw + d_yaw).rem_euclid(std::f32::consts::TAU);
        self.pitch = (self.pitch + d_pitch).clamp(-Self::MAX_PITCH, Self::MAX_PITCH);
    }

    pub fn view_proj(&self, aspect: f32) -> Mat4 {
        let target = self.position + self.forward();
        let view = rh::view::look_at_mat4(self.position, target, Vec3::Y);
        perspective(self.fov_y, aspect) * view
    }

    pub fn view(&self, aspect: f32) -> View {
        View {
            view_proj: self.view_proj(aspect),
            eye: self.position,
        }
    }
}

fn perspective(fov_y: f32, aspect: f32) -> Mat4 {
    rh::proj::directx::perspective(fov_y, aspect.max(0.01), NEAR, FAR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fly_camera_axes_are_consistent() {
        let mut cam = FlyCamera::default();
        assert!(cam.forward().abs_diff_eq(Vec3::NEG_Z, 1e-6));
        assert!(cam.flat_right().abs_diff_eq(Vec3::X, 1e-6));
        cam.rotate(std::f32::consts::FRAC_PI_2, 0.0);
        assert!(cam.forward().abs_diff_eq(Vec3::NEG_X, 1e-5));
        assert!(cam.flat_right().abs_diff_eq(Vec3::NEG_Z, 1e-5));
        cam.rotate(0.0, 10.0);
        assert!(cam.pitch < 1.56 && cam.forward().y > 0.99);
    }

    #[test]
    fn point_ahead_projects_to_screen_center() {
        let cam = FlyCamera {
            position: Vec3::new(3.0, 4.0, 5.0),
            yaw: 0.7,
            pitch: -0.3,
            ..FlyCamera::default()
        };
        let p = cam.position + cam.forward() * 10.0;
        let clip = cam.view_proj(16.0 / 9.0) * p.extend(1.0);
        let ndc = clip.truncate() / clip.w;
        assert!(ndc.x.abs() < 1e-4 && ndc.y.abs() < 1e-4);
        assert!((0.0..1.0).contains(&ndc.z));
    }
}
