//! Estado del launcher: páginas, selección, valores ajustables y acciones
//! pendientes. Puro y sin E/S para que sea fácil de probar.

use crate::thumbnail::Preview;
use crate::versions::{GameVersion, StartMode};

pub const PRESET_SEEDS: [u64; 6] = [42, 1931, 1979, 7, 256, 999];
pub const MINER_PROFILES: [&str; 5] = ["MINERO", "COBRE-1", "COBALTO-1", "VIGIA", "ZAPADOR"];
pub const MAX_BOTS: u32 = 8;

/// Páginas del panel, en el orden de la barra lateral.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Play,
    Instalaciones,
    Opciones,
    Acerca,
}

pub const PAGES: [Page; 4] = [
    Page::Play,
    Page::Instalaciones,
    Page::Opciones,
    Page::Acerca,
];

impl Page {
    pub fn label(self) -> &'static str {
        match self {
            Page::Play => "JUGAR",
            Page::Instalaciones => "INSTALACIONES",
            Page::Opciones => "OPCIONES",
            Page::Acerca => "ACERCA DE",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Page::Play => "ELIGE VERSION Y ENTRA A LA MINA",
            Page::Instalaciones => "BINARIOS DETECTADOS EN ESTE EQUIPO",
            Page::Opciones => "AJUSTES DE LA PROXIMA PARTIDA",
            Page::Acerca => "SOBRE HUSHMINE Y EL MOTOR GRISU",
        }
    }

    fn index(self) -> usize {
        PAGES
            .iter()
            .position(|p| *p == self)
            .expect("pagina valida")
    }
}

/// Acción que el estado pide ejecutar al anfitrión (procesos, E/S).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PendingAction {
    Launch {
        version: usize,
    },
    Compile {
        version: usize,
    },
    /// Arranca solo el servidor dedicado LAN con la semilla y bots elegidos.
    StartServer,
}

/// Qué hace un elemento al pulsarlo con el ratón en el centro.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    /// Mitades izquierda/derecha ajustan el valor.
    Adjuster,
    /// Un clic ejecuta la acción (botones y tarjetas ya seleccionadas).
    Button,
    /// Un clic la selecciona; otro clic sobre la ya elegida la lanza.
    Card,
}

/// Estado completo del panel, sin dependencias de ventana ni sistema.
pub struct LauncherState {
    pub versions: Vec<GameVersion>,
    pub version_idx: usize,
    pub start_mode: StartMode,
    pub bots: u32,
    pub seed_idx: usize,
    pub miner_idx: usize,
    pub close_on_launch: bool,
    pub status: String,
    pub status_ok: bool,
    pub mouse: (f32, f32),
    /// Segundos de animación acumulados; alimenta los pulsos del dibujo.
    pub animation: f32,
    /// Segundos desde el último cambio de página, para el fundido de entrada.
    pub page_fade: f32,
    /// Miniatura del terreno de la semilla elegida, si ya se generó.
    pub preview: Option<Preview>,
    page: Page,
    selected: [usize; PAGES.len()],
    last_played: Vec<(String, u64)>,
    now_secs: u64,
}

impl LauncherState {
    pub fn new(versions: Vec<GameVersion>, last_played: Vec<(String, u64)>, now_secs: u64) -> Self {
        Self {
            versions,
            version_idx: 0,
            start_mode: StartMode::DirectMatch,
            bots: 4,
            seed_idx: 0,
            miner_idx: 0,
            close_on_launch: false,
            status: "LISTO".into(),
            status_ok: true,
            mouse: (-1.0, -1.0),
            animation: 0.0,
            page_fade: 1.0,
            preview: None,
            page: Page::Play,
            selected: [1, 0, 0, 0],
            last_played,
            now_secs,
        }
    }

    pub fn page(&self) -> Page {
        self.page
    }

    pub fn selected(&self) -> usize {
        self.selected[self.page.index()]
    }

    pub fn select_page(&mut self, page: Page) {
        if self.page != page {
            self.page = page;
            self.page_fade = 0.0;
        }
    }

    pub fn cycle_page(&mut self, dir: i32) {
        let idx = self.page.index();
        let next = step(idx, PAGES.len(), dir);
        self.page = PAGES[next];
        self.page_fade = 0.0;
    }

    /// Avanza la animación de pulsos y el fundido de página.
    pub fn tick(&mut self, dt: f32) {
        self.animation += dt;
        self.page_fade = (self.page_fade + dt).min(crate::theme::FADE_TIME);
    }

    pub fn set_status(&mut self, msg: String, ok: bool) {
        self.status = msg;
        self.status_ok = ok;
    }

    pub fn item_count(&self, page: Page) -> usize {
        match page {
            Page::Play => 4,
            Page::Instalaciones => self.versions.len(),
            Page::Opciones => 5,
            Page::Acerca => 0,
        }
    }

    pub fn item_kind(&self, page: Page, idx: usize) -> ItemKind {
        match page {
            Page::Play => {
                if idx == 0 {
                    ItemKind::Adjuster
                } else {
                    ItemKind::Button
                }
            }
            Page::Instalaciones => ItemKind::Card,
            Page::Opciones => ItemKind::Adjuster,
            Page::Acerca => ItemKind::Button,
        }
    }

    pub fn move_selection(&mut self, dir: i32) {
        let count = self.item_count(self.page);
        if count > 0 {
            let slot = self.page.index();
            self.selected[slot] = step(self.selected[slot], count, dir);
        }
    }

    /// Fija el elemento seleccionado si existe en la página actual.
    pub fn select_item(&mut self, item: usize) {
        if item < self.item_count(self.page) {
            self.selected[self.page.index()] = item;
        }
    }

    pub fn adjust(&mut self, dir: i32) {
        match self.page {
            Page::Play if self.selected() == 0 => {
                self.version_idx = step(self.version_idx, self.versions.len(), dir);
            }
            Page::Opciones => self.adjust_option(self.selected(), dir),
            _ => {}
        }
    }

    fn adjust_option(&mut self, row: usize, dir: i32) {
        match row {
            0 => self.start_mode = self.start_mode.next(),
            1 => self.bots = (self.bots as i32 + dir).clamp(0, MAX_BOTS as i32) as u32,
            2 => self.seed_idx = step(self.seed_idx, PRESET_SEEDS.len(), dir),
            3 => self.miner_idx = step(self.miner_idx, MINER_PROFILES.len(), dir),
            4 => self.close_on_launch = !self.close_on_launch,
            _ => {}
        }
    }

    /// Enter o clic central: devuelve la acción pendiente, si la hay.
    pub fn activate(&mut self) -> Option<PendingAction> {
        match self.page {
            Page::Play => match self.selected() {
                0 => {
                    self.adjust(1);
                    None
                }
                1 => Some(self.launch_action()),
                2 => Some(self.compile_action()),
                3 => Some(PendingAction::StartServer),
                _ => None,
            },
            Page::Instalaciones => Some(self.launch_action()),
            Page::Opciones => {
                self.adjust_option(self.selected(), 1);
                None
            }
            Page::Acerca => None,
        }
    }

    /// Compilar la versión vigente (atajo `C` en Jugar e Instalaciones).
    pub fn compile_selected(&self) -> PendingAction {
        match self.page {
            Page::Instalaciones => PendingAction::Compile {
                version: self.selected(),
            },
            _ => self.compile_action(),
        }
    }

    pub fn launch_action(&self) -> PendingAction {
        PendingAction::Launch {
            version: self.current_version_idx(),
        }
    }

    pub fn compile_action(&self) -> PendingAction {
        PendingAction::Compile {
            version: self.current_version_idx(),
        }
    }

    /// Índice de versión vigente: la elegida, o la tarjeta de Instalaciones.
    pub fn current_version_idx(&self) -> usize {
        match self.page {
            Page::Instalaciones => self.selected().min(self.versions.len() - 1),
            _ => self.version_idx,
        }
    }

    pub fn current_version(&self) -> &GameVersion {
        &self.versions[self.current_version_idx()]
    }

    pub fn current_seed(&self) -> u64 {
        PRESET_SEEDS[self.seed_idx]
    }

    pub fn current_miner(&self) -> &'static str {
        MINER_PROFILES[self.miner_idx]
    }

    /// Registra un lanzamiento y anota el momento para las tarjetas.
    pub fn record_launch(&mut self, version: usize, now_secs: u64) {
        self.now_secs = now_secs;
        if let Some(ver) = self.versions.get(version) {
            crate::history::merge_entry(&mut self.last_played, &ver.label, now_secs);
        }
    }

    /// Texto de último uso para la tarjeta `idx`.
    pub fn last_played_text(&self, idx: usize) -> String {
        match self.versions.get(idx) {
            Some(ver) => match self.last_played_secs(&ver.label) {
                Some(secs) => crate::history::format_age(self.now_secs, secs),
                None => "NUNCA JUGADO".into(),
            },
            None => String::new(),
        }
    }

    fn last_played_secs(&self, label: &str) -> Option<u64> {
        self.last_played
            .iter()
            .find(|(l, _)| l == label)
            .map(|(_, secs)| *secs)
    }

    /// Filas (etiqueta, valor) de la página Opciones.
    pub fn option_rows(&self) -> [(String, String); 5] {
        [
            (
                "MODO DE ARRANQUE".into(),
                format!("< {} >", self.start_mode.label()),
            ),
            ("BOTS EN PARTIDA".into(), format!("< {} >", self.bots)),
            (
                "SEMILLA DEL MAPA".into(),
                format!("< {} >", self.current_seed()),
            ),
            (
                "PERFIL DE MINERO".into(),
                format!("< {} >", self.current_miner()),
            ),
            (
                "AL LANZAR".into(),
                format!(
                    "< {} >",
                    if self.close_on_launch {
                        "CERRAR LAUNCHER"
                    } else {
                        "DEJAR ABIERTO"
                    }
                ),
            ),
        ]
    }

    /// Línea resumen de la página Jugar.
    pub fn summary_line(&self) -> String {
        format!(
            "MODO: {} - BOTS: {} - SEMILLA: {} - PERFIL: {}",
            self.start_mode.label(),
            self.bots,
            self.current_seed(),
            self.current_miner()
        )
    }
}

/// Índice anterior/siguiente con vuelta circular.
fn step(cur: usize, len: usize, dir: i32) -> usize {
    if len == 0 {
        return 0;
    }
    if dir >= 0 {
        (cur + 1) % len
    } else {
        (cur + len - 1) % len
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
