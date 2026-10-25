//! Controles configurables del cliente: teclas, modo de carrera, sensibilidad
//! del ratón, campo de visión y distancia de cámara en tercera persona.

use std::fs;
use std::path::Path;

use grisu_platform::KeyCode;

/// Ruta donde se guardan las preferencias locales del jugador.
pub const CONTROLS_FILE: &str = "out/controls.cfg";

/// Acciones cuyas teclas puede reasignar el jugador desde el menú.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindAction {
    Forward,
    Back,
    Left,
    Right,
    Jump,
    Sprint,
    Camera,
}

impl BindAction {
    pub const ALL: [Self; 7] = [
        Self::Forward,
        Self::Back,
        Self::Left,
        Self::Right,
        Self::Jump,
        Self::Sprint,
        Self::Camera,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Forward => "Avanzar",
            Self::Back => "Retroceder",
            Self::Left => "Izquierda",
            Self::Right => "Derecha",
            Self::Jump => "Saltar",
            Self::Sprint => "Correr",
            Self::Camera => "Cambiar de cámara",
        }
    }
}

/// Cómo se activa la carrera al pulsar la tecla asignada.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SprintMode {
    /// Corre mientras se mantiene pulsada la tecla (por defecto).
    #[default]
    Hold,
    /// Una pulsación activa o desactiva correr hasta parar.
    Toggle,
}

impl SprintMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Hold => Self::Toggle,
            Self::Toggle => Self::Hold,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Hold => "Mantener pulsada",
            Self::Toggle => "Pulsar una vez",
        }
    }
}

/// Configuración de controles y cámara del cliente.
#[derive(Clone, Debug, PartialEq)]
pub struct Controls {
    pub forward: KeyCode,
    pub back: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub jump: KeyCode,
    pub sprint: KeyCode,
    pub camera: KeyCode,
    pub sprint_mode: SprintMode,
    /// Sensibilidad del ratón en porcentaje (`20..=300`, por defecto `100`).
    pub sensitivity_pct: u32,
    /// Campo de visión vertical en grados (`60..=110`, por defecto `75`).
    pub fov_deg: u32,
    /// Distancia del brazo de tercera persona en bloques (`1.5..=6.0`).
    pub third_person_dist: f32,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            forward: KeyCode::KeyW,
            back: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            jump: KeyCode::Space,
            sprint: KeyCode::ShiftLeft,
            camera: KeyCode::KeyV,
            sprint_mode: SprintMode::Hold,
            sensitivity_pct: 100,
            fov_deg: 75,
            third_person_dist: 3.5,
        }
    }
}

impl Controls {
    pub fn load_or_default() -> Self {
        Self::load_from(Path::new(CONTROLS_FILE)).unwrap_or_default()
    }

    pub fn save_default_path(&self) {
        let path = Path::new(CONTROLS_FILE);
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, self.serialize());
    }

    pub fn load_from(path: &Path) -> Option<Self> {
        let text = fs::read_to_string(path).ok()?;
        Some(Self::deserialize(&text))
    }

    pub fn get(&self, action: BindAction) -> KeyCode {
        match action {
            BindAction::Forward => self.forward,
            BindAction::Back => self.back,
            BindAction::Left => self.left,
            BindAction::Right => self.right,
            BindAction::Jump => self.jump,
            BindAction::Sprint => self.sprint,
            BindAction::Camera => self.camera,
        }
    }

    /// Asigna `new_key` a `action`. Si otra acción tenía esa tecla, las
    /// intercambia para no dejar acciones duplicadas.
    pub fn set(&mut self, action: BindAction, new_key: KeyCode) {
        let old_key = self.get(action);
        for other in BindAction::ALL {
            if other != action && self.get(other) == new_key {
                self.set_raw(other, old_key);
            }
        }
        self.set_raw(action, new_key);
    }

    fn set_raw(&mut self, action: BindAction, key: KeyCode) {
        match action {
            BindAction::Forward => self.forward = key,
            BindAction::Back => self.back = key,
            BindAction::Left => self.left = key,
            BindAction::Right => self.right = key,
            BindAction::Jump => self.jump = key,
            BindAction::Sprint => self.sprint = key,
            BindAction::Camera => self.camera = key,
        }
    }

    pub fn adjust_sensitivity(&mut self, delta: i32) {
        let next = (self.sensitivity_pct as i32 + delta).clamp(20, 300);
        self.sensitivity_pct = next as u32;
    }

    pub fn adjust_fov(&mut self, delta: i32) {
        let next = (self.fov_deg as i32 + delta).clamp(60, 110);
        self.fov_deg = next as u32;
    }

    pub fn adjust_third_person_dist(&mut self, delta: f32) {
        self.third_person_dist = (self.third_person_dist + delta).clamp(1.5, 6.0);
    }

    pub fn mouse_sensitivity(&self) -> f32 {
        0.0025 * (self.sensitivity_pct as f32 / 100.0)
    }

    pub fn fov_radians(&self) -> f32 {
        (self.fov_deg as f32).to_radians()
    }

    pub fn serialize(&self) -> String {
        format!(
            "forward={}\nback={}\nleft={}\nright={}\njump={}\nsprint={}\ncamera={}\nsprint_mode={}\nsensitivity={}\nfov={}\nthird_person={:.1}\n",
            key_label(self.forward),
            key_label(self.back),
            key_label(self.left),
            key_label(self.right),
            key_label(self.jump),
            key_label(self.sprint),
            key_label(self.camera),
            match self.sprint_mode {
                SprintMode::Hold => "hold",
                SprintMode::Toggle => "toggle",
            },
            self.sensitivity_pct,
            self.fov_deg,
            self.third_person_dist,
        )
    }

    pub fn deserialize(text: &str) -> Self {
        let mut c = Self::default();
        for line in text.lines() {
            let Some((k, v)) = line.trim().split_once('=') else {
                continue;
            };
            match k.trim() {
                "forward" => {
                    if let Some(code) = parse_key(v) {
                        c.forward = code;
                    }
                }
                "back" => {
                    if let Some(code) = parse_key(v) {
                        c.back = code;
                    }
                }
                "left" => {
                    if let Some(code) = parse_key(v) {
                        c.left = code;
                    }
                }
                "right" => {
                    if let Some(code) = parse_key(v) {
                        c.right = code;
                    }
                }
                "jump" => {
                    if let Some(code) = parse_key(v) {
                        c.jump = code;
                    }
                }
                "sprint" => {
                    if let Some(code) = parse_key(v) {
                        c.sprint = code;
                    }
                }
                "camera" => {
                    if let Some(code) = parse_key(v) {
                        c.camera = code;
                    }
                }
                "sprint_mode" => {
                    c.sprint_mode = if v.trim() == "toggle" {
                        SprintMode::Toggle
                    } else {
                        SprintMode::Hold
                    };
                }
                "sensitivity" => {
                    if let Ok(n) = v.trim().parse::<u32>() {
                        c.sensitivity_pct = n.clamp(20, 300);
                    }
                }
                "fov" => {
                    if let Ok(n) = v.trim().parse::<u32>() {
                        c.fov_deg = n.clamp(60, 110);
                    }
                }
                "third_person" => {
                    if let Ok(d) = v.trim().parse::<f32>()
                        && d.is_finite()
                    {
                        c.third_person_dist = d.clamp(1.5, 6.0);
                    }
                }
                _ => {}
            }
        }
        c
    }
}

const KEY_TABLE: &[(KeyCode, &str)] = &[
    (KeyCode::KeyA, "A"),
    (KeyCode::KeyB, "B"),
    (KeyCode::KeyC, "C"),
    (KeyCode::KeyD, "D"),
    (KeyCode::KeyE, "E"),
    (KeyCode::KeyF, "F"),
    (KeyCode::KeyG, "G"),
    (KeyCode::KeyH, "H"),
    (KeyCode::KeyI, "I"),
    (KeyCode::KeyJ, "J"),
    (KeyCode::KeyK, "K"),
    (KeyCode::KeyL, "L"),
    (KeyCode::KeyM, "M"),
    (KeyCode::KeyN, "N"),
    (KeyCode::KeyO, "O"),
    (KeyCode::KeyP, "P"),
    (KeyCode::KeyQ, "Q"),
    (KeyCode::KeyR, "R"),
    (KeyCode::KeyS, "S"),
    (KeyCode::KeyT, "T"),
    (KeyCode::KeyU, "U"),
    (KeyCode::KeyV, "V"),
    (KeyCode::KeyW, "W"),
    (KeyCode::KeyX, "X"),
    (KeyCode::KeyY, "Y"),
    (KeyCode::KeyZ, "Z"),
    (KeyCode::Space, "ESPACIO"),
    (KeyCode::ShiftLeft, "SHIFT IZQ"),
    (KeyCode::ShiftRight, "SHIFT DER"),
    (KeyCode::ControlLeft, "CTRL IZQ"),
    (KeyCode::ControlRight, "CTRL DER"),
    (KeyCode::AltLeft, "ALT IZQ"),
    (KeyCode::AltRight, "ALT DER"),
    (KeyCode::Tab, "TAB"),
    (KeyCode::CapsLock, "BLOQ MAYUS"),
    (KeyCode::Enter, "ENTER"),
    (KeyCode::Backspace, "RETROCESO"),
    (KeyCode::ArrowUp, "FLECHA ARR"),
    (KeyCode::ArrowDown, "FLECHA ABA"),
    (KeyCode::ArrowLeft, "FLECHA IZQ"),
    (KeyCode::ArrowRight, "FLECHA DER"),
    (KeyCode::F1, "F1"),
    (KeyCode::F2, "F2"),
    (KeyCode::F3, "F3"),
    (KeyCode::F4, "F4"),
    (KeyCode::F5, "F5"),
    (KeyCode::F6, "F6"),
    (KeyCode::F7, "F7"),
    (KeyCode::F8, "F8"),
];

pub fn key_label(key: KeyCode) -> &'static str {
    KEY_TABLE
        .iter()
        .find_map(|&(k, name)| (k == key).then_some(name))
        .unwrap_or("TECLA")
}

/// Nombre de la tecla para mostrarlo en pantalla. `key_label` es el del
/// archivo de controles y no puede cambiar sin romper los ya guardados.
pub fn key_name(key: KeyCode) -> &'static str {
    match key {
        KeyCode::Space => "Espacio",
        KeyCode::ShiftLeft => "Mayús izq.",
        KeyCode::ShiftRight => "Mayús der.",
        KeyCode::ControlLeft => "Ctrl izq.",
        KeyCode::ControlRight => "Ctrl der.",
        KeyCode::AltLeft => "Alt izq.",
        KeyCode::AltRight => "Alt der.",
        KeyCode::Tab => "Tab",
        KeyCode::CapsLock => "Bloq Mayús",
        KeyCode::Enter => "Enter",
        KeyCode::Backspace => "Retroceso",
        KeyCode::ArrowUp => "Flecha arriba",
        KeyCode::ArrowDown => "Flecha abajo",
        KeyCode::ArrowLeft => "Flecha izquierda",
        KeyCode::ArrowRight => "Flecha derecha",
        other => match key_label(other) {
            "TECLA" => "Otra tecla",
            name => name,
        },
    }
}

fn parse_key(name: &str) -> Option<KeyCode> {
    let trimmed = name.trim();
    KEY_TABLE
        .iter()
        .find_map(|&(k, label)| (label == trimmed).then_some(k))
}

#[cfg(test)]
#[path = "controls_tests.rs"]
mod tests;
