//! Lo que el juego de Turno dibuja y registra: la vista de la cámara, las cajas de los mineros y las estadísticas.

use super::*;

/// Alto de la caja de un minero agachado, en bloques.
const CROUCH_BOX_HEIGHT: f32 = 1.35;

impl Game {
    /// Vista según el modo de cámara. En tercera persona, el brazo se
    /// recorta contra el terreno para no atravesar paredes.
    pub(super) fn active_view(&self, aspect: f32) -> View {
        if !self.camera_mode.is_third_person() {
            return self.camera.view(aspect);
        }
        let front = self.camera_mode == CameraMode::ThirdPersonFront;
        let desired = f64::from(self.controls.third_person_dist);
        let dir = self.camera.third_person_arm_dir(front).as_dvec3();
        let clipped = self
            .session
            .client()
            .world()
            .and_then(|w| raycast(w, self.camera.position.to_array(), dir.to_array(), desired))
            .map_or(desired, |hit| {
                (hit.distance - THIRD_PERSON_WALL_MARGIN).max(0.0)
            });
        self.camera.third_person_view(aspect, clipped as f32, front)
    }

    /// Cajas de los demás jugadores y, en tercera persona, del propio minero
    /// con su lámpara.
    pub(super) fn entity_boxes(&self) -> Vec<ColorBox> {
        let Some(tick) = self.session.client().last_server_tick() else {
            return Vec::new();
        };
        let at = tick as f64 + self.accumulator as f64 / TICK_SECONDS as f64 - REMOTE_DELAY_TICKS;
        let width = 2.0 * HALF_WIDTH as f64 / FIXED_ONE as f64;
        let height = HEIGHT as f64 / FIXED_ONE as f64;
        let size = Vec3::new(width as f32, height as f32, width as f32);
        // Agachado, la caja baja: la silueta dice que va en silencio.
        let crouched = Vec3::new(width as f32, CROUCH_BOX_HEIGHT, width as f32);
        let client = self.session.client();
        let crew = |id: u16| CREW_COLORS[usize::from(id) % CREW_COLORS.len()];
        let mut list: Vec<ColorBox> = self
            .session
            .client()
            .remote_players(at)
            .into_iter()
            .map(|(id, [x, y, z])| ColorBox {
                min: DVec3::new(x - width / 2.0, y, z - width / 2.0),
                size: if client.remote_crouching(id) {
                    crouched
                } else {
                    size
                },
                color: crew(id),
            })
            .collect();
        if self.camera_mode.is_third_person() {
            let me = client.body();
            let eye = if me.crouching {
                CROUCH_EYE_HEIGHT
            } else {
                EYE_HEIGHT
            };
            let feet = self.camera.position - DVec3::Y * (eye as f64 / FIXED_ONE as f64);
            list.push(ColorBox {
                min: DVec3::new(feet.x - width / 2.0, feet.y, feet.z - width / 2.0),
                size: if me.crouching { crouched } else { size },
                color: crew(self.session.client().player().unwrap_or(0)),
            });
            let lamp =
                self.camera.position + self.camera.flat_forward().as_dvec3() * (width * 0.45);
            list.push(ColorBox {
                min: lamp - DVec3::splat(0.12),
                size: Vec3::splat(0.24),
                color: HELMET_LAMP_COLOR,
            });
        }
        list
    }

    pub(super) fn log_stats(&mut self, dt: f32) {
        self.frames += 1;
        self.stats_timer += dt;
        if self.stats_timer >= 5.0 {
            let fps = self.frames as f32 / self.stats_timer;
            let client = self.session.client();
            let net = client.connection().stats();
            let p = client.stats();
            log::info!(
                "{fps:.0} FPS, rtt {:.0} ms, {} correcciones de {} instantáneas",
                net.rtt_ms,
                p.corrections,
                p.snapshots
            );
            self.frames = 0;
            self.stats_timer = 0.0;
        }
    }
}
