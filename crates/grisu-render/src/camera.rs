use glam::camera::rh;
use glam::{DVec3, Mat4, Vec3};

/// Plano cercano y lejano de todas las cámaras, en bloques.
const NEAR: f32 = 0.05;
const FAR: f32 = 500.0;

/// Lo que necesita un renderizador para dibujar desde un punto de vista.
///
/// Todo se dibuja **relativo a la cámara** (ADR 0002): `anchor` es el bloque
/// entero en el que está la cámara y tanto `eye` como `view_proj` trabajan en
/// coordenadas relativas a ese bloque. Así al shader solo llegan números
/// pequeños y no hay temblores a millones de bloques del origen.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub view_proj: Mat4,
    /// Ojo relativo a `anchor`.
    pub eye: Vec3,
    /// Bloque de referencia, en coordenadas del mundo.
    pub anchor: [i32; 3],
}

impl View {
    /// Pasa un punto del mundo a coordenadas relativas a `anchor`.
    pub fn relative(&self, world: DVec3) -> Vec3 {
        (world - anchor_f64(self.anchor)).as_vec3()
    }

    /// Pasa un bloque del mundo a coordenadas relativas a `anchor`.
    pub fn relative_block(&self, block: [i32; 3]) -> [i32; 3] {
        std::array::from_fn(|i| block[i].wrapping_sub(self.anchor[i]))
    }
}

fn anchor_f64(anchor: [i32; 3]) -> DVec3 {
    DVec3::new(
        f64::from(anchor[0]),
        f64::from(anchor[1]),
        f64::from(anchor[2]),
    )
}

/// Modo de cámara del jugador (ver `docs/graficos/01-camaras.md`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CameraMode {
    /// Desde los ojos del minero (por defecto).
    #[default]
    FirstPerson,
    /// Detrás del hombro, mirando hacia donde apunta el minero.
    ThirdPersonBack,
    /// Delante del minero, mirándolo de frente.
    ThirdPersonFront,
}

impl CameraMode {
    /// Pasa al siguiente modo de cámara en ciclo.
    pub fn next(self) -> Self {
        match self {
            Self::FirstPerson => Self::ThirdPersonBack,
            Self::ThirdPersonBack => Self::ThirdPersonFront,
            Self::ThirdPersonFront => Self::FirstPerson,
        }
    }

    pub fn is_third_person(self) -> bool {
        !matches!(self, Self::FirstPerson)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::FirstPerson => "Primera persona",
            Self::ThirdPersonBack => "Desde atrás",
            Self::ThirdPersonFront => "Desde delante",
        }
    }
}

/// Cámara que orbita alrededor de un punto. Sirve para las capturas desde
/// fuera del mapa, cerca del origen.
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

    /// Vista con el origen del mundo como referencia.
    pub fn view(&self, aspect: f32) -> View {
        View {
            view_proj: self.view_proj(aspect),
            eye: self.eye(),
            anchor: [0; 3],
        }
    }
}

/// Cámara en primera persona. Con `yaw = 0` mira hacia -Z; `yaw` crece
/// girando hacia la izquierda (de -Z hacia -X).
///
/// La posición es `f64`: precisa al milímetro a 7,5 millones de bloques.
#[derive(Clone, Copy, Debug)]
pub struct FlyCamera {
    pub position: DVec3,
    pub yaw: f32,
    /// Positivo = mirar hacia arriba. Se limita a ±89°.
    pub pitch: f32,
    pub fov_y: f32,
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self {
            position: DVec3::ZERO,
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

    /// Vista relativa al bloque que contiene la cámara.
    pub fn view(&self, aspect: f32) -> View {
        let floor = self.position.floor();
        let anchor = [floor.x as i32, floor.y as i32, floor.z as i32];
        let eye = (self.position - floor).as_vec3();
        let view = rh::view::look_at_mat4(eye, eye + self.forward(), Vec3::Y);
        View {
            view_proj: perspective(self.fov_y, aspect) * view,
            eye,
            anchor,
        }
    }

    /// Dirección unitaria desde los ojos del minero hacia el brazo de cámara
    /// en tercera persona (`front = false` detrás, `front = true` delante).
    pub fn third_person_arm_dir(&self, front: bool) -> Vec3 {
        let base = if front {
            self.forward()
        } else {
            -self.forward()
        };
        (base + Vec3::Y * 0.14).normalize_or_zero()
    }

    /// Vista en tercera persona con el brazo a `distance` bloques (ya
    /// recortado contra el terreno). Por debajo de 0,45 bloques cae a
    /// primera persona. Como `view`, es relativa al bloque del ojo.
    pub fn third_person_view(&self, aspect: f32, distance: f32, front: bool) -> View {
        if distance < 0.45 {
            return self.view(aspect);
        }
        let arm = self.third_person_arm_dir(front).as_dvec3() * f64::from(distance);
        let eye_world = self.position + arm;
        let target_world = if front {
            self.position - DVec3::Y * 0.35
        } else {
            self.position + self.forward().as_dvec3() * 8.0
        };
        let floor = eye_world.floor();
        let anchor = [floor.x as i32, floor.y as i32, floor.z as i32];
        let eye = (eye_world - floor).as_vec3();
        let target = (target_world - floor).as_vec3();
        let view = rh::view::look_at_mat4(eye, target, Vec3::Y);
        View {
            view_proj: perspective(self.fov_y, aspect) * view,
            eye,
            anchor,
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
            position: DVec3::new(3.0, 4.0, 5.0),
            yaw: 0.7,
            pitch: -0.3,
            ..FlyCamera::default()
        };
        let view = cam.view(16.0 / 9.0);
        let p = cam.position + cam.forward().as_dvec3() * 10.0;
        let clip = view.view_proj * view.relative(p).extend(1.0);
        let ndc = clip.truncate() / clip.w;
        assert!(ndc.x.abs() < 1e-4 && ndc.y.abs() < 1e-4);
        assert!((0.0..1.0).contains(&ndc.z));
    }

    #[test]
    fn far_cameras_see_exactly_what_near_ones_do() {
        let near = FlyCamera {
            position: DVec3::new(2.25, 3.5, -0.75),
            yaw: 0.4,
            pitch: -0.2,
            ..FlyCamera::default()
        };
        let offset = DVec3::new(7_499_900.0, -4000.0, -7_499_900.0);
        let far = FlyCamera {
            position: near.position + offset,
            ..near
        };
        let (a, b) = (near.view(1.5), far.view(1.5));
        // Misma fracción dentro del bloque: misma matriz, bit a bit.
        assert_eq!(a.view_proj, b.view_proj);
        assert_eq!(a.eye, b.eye);
        assert_eq!(b.anchor, [7_499_902, -3997, -7_499_901]);
        // Y un punto del mundo cae en el mismo sitio relativo.
        let p = DVec3::new(5.5, 1.0, -8.25);
        assert_eq!(a.relative(p), b.relative(p + offset));
    }

    #[test]
    fn third_person_camera_cycles_and_offsets_eye() {
        let mode = CameraMode::default();
        assert_eq!(mode, CameraMode::FirstPerson);
        assert_eq!(mode.next(), CameraMode::ThirdPersonBack);
        assert_eq!(mode.next().next(), CameraMode::ThirdPersonFront);
        assert_eq!(mode.next().next().next(), CameraMode::FirstPerson);

        for base in [
            DVec3::new(10.5, 20.5, 10.5),
            DVec3::new(-7_499_800.5, 600.5, 7_499_800.5),
        ] {
            let cam = FlyCamera {
                position: base,
                ..FlyCamera::default()
            };
            let world_eye = |v: View| v.eye.as_dvec3() + anchor_f64(v.anchor);
            // Con yaw = 0 la cámara mira hacia -Z: detrás es +Z y delante es -Z.
            let back = world_eye(cam.third_person_view(16.0 / 9.0, 3.5, false));
            assert!(back.z > cam.position.z + 3.0, "{base}: {back}");
            let front = world_eye(cam.third_person_view(16.0 / 9.0, 3.5, true));
            assert!(front.z < cam.position.z - 3.0, "{base}: {front}");
            let cramped = world_eye(cam.third_person_view(16.0 / 9.0, 0.2, false));
            assert!((cramped - cam.position).length() < 1e-6);
        }
    }
}
