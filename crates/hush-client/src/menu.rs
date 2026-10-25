//! Estado y navegación del menú (Launcher inicial, Pausa con Escape y
//! configuración de controles).

use grisu_platform::{Input, KeyCode, MouseButton};
use grisu_render::CameraMode;

use crate::controls::{BindAction, Controls, key_label};
use crate::hud::row_rect;

/// Eventos que el menú devuelve a `Game` cuando el jugador elige una acción.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuCommand {
    None,
    /// Inicia o reinicia una partida local con `(seed, bots)`.
    StartLocal {
        seed: u64,
        bots: u32,
    },
    /// Cierra la ventana del juego.
    Exit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuPage {
    Hidden,
    Launcher,
    Pause,
    Controls { from_launcher: bool },
}

pub struct MenuState {
    pub page: MenuPage,
    pub selected: usize,
    pub rebinding: Option<BindAction>,
    pub launcher_seed: u64,
    pub launcher_bots: u32,
    pub remote_session: bool,
}

impl MenuState {
    pub fn new(start_in_launcher: bool, seed: u64, bots: u32, remote_session: bool) -> Self {
        let page = if start_in_launcher {
            MenuPage::Launcher
        } else {
            MenuPage::Hidden
        };
        Self {
            page,
            selected: 0,
            rebinding: None,
            launcher_seed: seed,
            launcher_bots: bots,
            remote_session,
        }
    }

    pub fn is_open(&self) -> bool {
        self.page != MenuPage::Hidden
    }

    fn item_count(&self) -> usize {
        match self.page {
            MenuPage::Hidden => 0,
            MenuPage::Launcher => 6,
            MenuPage::Pause => 7,
            MenuPage::Controls { .. } => 11,
        }
    }

    /// Procesa el teclado y el ratón cuando el menú está abierto (o `Escape`).
    pub fn update(
        &mut self,
        input: &Input,
        controls: &mut Controls,
        camera_mode: &mut CameraMode,
    ) -> MenuCommand {
        if let Some(action) = self.rebinding {
            if let Some(key) = input.last_key_pressed() {
                if key != KeyCode::Escape {
                    controls.set(action, key);
                    controls.save_default_path();
                }
                self.rebinding = None;
            }
            return MenuCommand::None;
        }

        if input.was_pressed(KeyCode::Escape) {
            match self.page {
                MenuPage::Hidden => {
                    self.page = MenuPage::Pause;
                    self.selected = 0;
                }
                MenuPage::Pause => self.page = MenuPage::Hidden,
                MenuPage::Controls { from_launcher } => {
                    controls.save_default_path();
                    self.page = if from_launcher {
                        MenuPage::Launcher
                    } else {
                        MenuPage::Pause
                    };
                    self.selected = 0;
                }
                MenuPage::Launcher => {}
            }
            return MenuCommand::None;
        }

        if !self.is_open() {
            return MenuCommand::None;
        }

        let count = self.item_count();
        if input.was_pressed(KeyCode::ArrowUp) || input.was_pressed(KeyCode::KeyW) {
            self.selected = (self.selected + count - 1) % count;
        }
        if input.was_pressed(KeyCode::ArrowDown) || input.was_pressed(KeyCode::KeyS) {
            self.selected = (self.selected + 1) % count;
        }

        let (win_w, win_h) = input.window_size();
        let (mx, my) = input.cursor_pos();
        let mut clicked_dir: Option<i32> = None;
        if win_w > 0 && win_h > 0 {
            for idx in 0..count {
                let row = row_rect(self.page, idx, win_w as f32, win_h as f32);
                if row.contains(mx, my) {
                    self.selected = idx;
                    if input.was_button_pressed(MouseButton::Left) {
                        let mid = row.rect[0] + row.rect[2] * 0.55;
                        clicked_dir = Some(if mx < mid { -1 } else { 1 });
                    }
                    break;
                }
            }
        }

        let left = input.was_pressed(KeyCode::ArrowLeft) || input.was_pressed(KeyCode::KeyA);
        let right = input.was_pressed(KeyCode::ArrowRight) || input.was_pressed(KeyCode::KeyD);
        let enter = input.was_pressed(KeyCode::Enter) || input.was_pressed(KeyCode::Space);
        let dir = i32::from(right) - i32::from(left);

        if dir != 0 {
            self.adjust_item(self.selected, dir, controls, camera_mode);
        }
        if enter || clicked_dir.is_some() {
            return self.activate_item(
                self.selected,
                clicked_dir.unwrap_or(1),
                controls,
                camera_mode,
            );
        }
        MenuCommand::None
    }

    fn adjust_item(
        &mut self,
        idx: usize,
        dir: i32,
        controls: &mut Controls,
        camera_mode: &mut CameraMode,
    ) {
        match (self.page, idx) {
            (MenuPage::Launcher, 1) => {
                self.launcher_bots = (self.launcher_bots as i32 + dir).clamp(0, 8) as u32;
            }
            (MenuPage::Launcher, 2) => {
                self.launcher_seed = (self.launcher_seed as i64 + i64::from(dir)).max(1) as u64;
            }
            (MenuPage::Launcher, 3) | (MenuPage::Pause, 1) => *camera_mode = camera_mode.next(),
            (MenuPage::Pause, 2) => controls.adjust_third_person_dist(dir as f32 * 0.5),
            (MenuPage::Pause, 3) => {
                controls.sprint_mode = controls.sprint_mode.toggle();
                controls.save_default_path();
            }
            (MenuPage::Controls { .. }, 7) => {
                controls.adjust_sensitivity(dir * 10);
                controls.save_default_path();
            }
            (MenuPage::Controls { .. }, 8) => {
                controls.adjust_fov(dir * 5);
                controls.save_default_path();
            }
            _ => {}
        }
    }

    fn activate_item(
        &mut self,
        idx: usize,
        dir: i32,
        controls: &mut Controls,
        camera_mode: &mut CameraMode,
    ) -> MenuCommand {
        match (self.page, idx) {
            (MenuPage::Launcher, 0) => {
                self.page = MenuPage::Hidden;
                if !self.remote_session {
                    return MenuCommand::StartLocal {
                        seed: self.launcher_seed,
                        bots: self.launcher_bots,
                    };
                }
            }
            (MenuPage::Launcher, 1..=3)
            | (MenuPage::Pause, 1..=3)
            | (MenuPage::Controls { .. }, 7 | 8) => {
                self.adjust_item(idx, dir, controls, camera_mode);
            }
            (MenuPage::Launcher, 4) => {
                self.page = MenuPage::Controls {
                    from_launcher: true,
                };
                self.selected = 0;
            }
            (MenuPage::Launcher, 5) | (MenuPage::Pause, 6) => return MenuCommand::Exit,
            (MenuPage::Pause, 0) => self.page = MenuPage::Hidden,
            (MenuPage::Pause, 4) => {
                self.page = MenuPage::Controls {
                    from_launcher: false,
                };
                self.selected = 0;
            }
            (MenuPage::Pause, 5) => {
                self.page = MenuPage::Launcher;
                self.selected = 0;
            }
            (MenuPage::Controls { .. }, 0..=6) => self.rebinding = Some(BindAction::ALL[idx]),
            (MenuPage::Controls { .. }, 9) => {
                *controls = Controls::default();
                controls.save_default_path();
            }
            (MenuPage::Controls { from_launcher }, 10) => {
                controls.save_default_path();
                self.page = if from_launcher {
                    MenuPage::Launcher
                } else {
                    MenuPage::Pause
                };
                self.selected = 0;
            }
            _ => {}
        }
        MenuCommand::None
    }

    pub fn item_labels(&self, controls: &Controls, camera_mode: CameraMode) -> Vec<String> {
        match self.page {
            MenuPage::Launcher => vec![
                if self.remote_session {
                    "ENTRAR A LA PARTIDA EN RED".into()
                } else {
                    "INICIAR PARTIDA LOCAL".into()
                },
                format!("BOTS EN PARTIDA: < {} >", self.launcher_bots),
                format!("SEMILLA DEL MAPA: < {} >", self.launcher_seed),
                format!("CAMARA INICIAL: < {} >", camera_mode.label()),
                "CONFIGURAR CONTROLES Y RATON".into(),
                "SALIR DEL JUEGO".into(),
            ],
            MenuPage::Pause => vec![
                "CONTINUAR PARTIDA (ESC)".into(),
                format!("VISTA DE CAMARA: < {} >", camera_mode.label()),
                format!(
                    "DISTANCIA 3A PERSONA: < {:.1} BLOQUES >",
                    controls.third_person_dist
                ),
                format!("MODO DE CARRERA: < {} >", controls.sprint_mode.label()),
                "CAMBIAR CONTROLES Y SENSIBILIDAD".into(),
                "ABRIR LAUNCHER (NUEVA PARTIDA)".into(),
                "SALIR DEL JUEGO".into(),
            ],
            MenuPage::Controls { .. } => {
                let mut list: Vec<String> = BindAction::ALL
                    .iter()
                    .map(|&act| {
                        if self.rebinding == Some(act) {
                            format!("{}: [ PULSA UNA TECLA... ]", act.label())
                        } else {
                            format!("{}: [ {} ]", act.label(), key_label(controls.get(act)))
                        }
                    })
                    .collect();
                list.push(format!(
                    "SENSIBILIDAD RATON: < {}% >",
                    controls.sensitivity_pct
                ));
                list.push(format!("CAMPO DE VISION (FOV): < {} >", controls.fov_deg));
                list.push("RESTAURAR CONTROLES POR DEFECTO".into());
                list.push("GUARDAR Y VOLVER (ESC)".into());
                list
            }
            MenuPage::Hidden => Vec::new(),
        }
    }
}
