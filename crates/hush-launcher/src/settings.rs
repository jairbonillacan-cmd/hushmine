//! Ajustes del launcher que se recuerdan entre sesiones: versión, modo,
//! bots, semilla, perfil y cierre automático. Una línea TSV validada.

use std::fs;
use std::path::{Path, PathBuf};

use crate::state::{LauncherState, MAX_BOTS, MINER_PROFILES, PRESET_SEEDS};
use crate::versions::StartMode;

/// Instantánea de los valores configurables del panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SavedSettings {
    pub version_idx: usize,
    pub mode_idx: usize,
    pub bots: u32,
    pub seed_idx: usize,
    pub miner_idx: usize,
    pub close_on_launch: bool,
}

impl SavedSettings {
    pub fn from_state(state: &LauncherState) -> Self {
        Self {
            version_idx: state.version_idx,
            mode_idx: state.start_mode.index(),
            bots: state.bots,
            seed_idx: state.seed_idx,
            miner_idx: state.miner_idx,
            close_on_launch: state.close_on_launch,
        }
    }

    /// Restaura los valores en el panel, acotados a los rangos válidos.
    pub fn apply(&self, state: &mut LauncherState) {
        state.version_idx = self.version_idx.min(state.versions.len().saturating_sub(1));
        state.start_mode = StartMode::from_index(self.mode_idx);
        state.bots = self.bots.min(MAX_BOTS);
        state.seed_idx = self.seed_idx % PRESET_SEEDS.len();
        state.miner_idx = self.miner_idx % MINER_PROFILES.len();
        state.close_on_launch = self.close_on_launch;
    }

    fn serialize(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            self.version_idx,
            self.mode_idx,
            self.bots,
            self.seed_idx,
            self.miner_idx,
            u8::from(self.close_on_launch)
        )
    }
}

/// Ruta del archivo de ajustes junto al registro de últimos lanzamientos.
pub fn settings_path(root: &Path) -> PathBuf {
    root.join("versions").join("settings.tsv")
}

/// Carga los ajustes guardados; `None` si no existen o están corruptos.
pub fn load_settings(root: &Path) -> Option<SavedSettings> {
    fs::read_to_string(settings_path(root))
        .ok()
        .and_then(|text| parse_settings(&text))
}

/// Guarda los ajustes; los fallos de disco se ignoran (no bloquean el panel).
pub fn save_settings(root: &Path, settings: &SavedSettings) {
    let _ = fs::create_dir_all(root.join("versions"));
    let _ = fs::write(settings_path(root), settings.serialize());
}

fn parse_settings(text: &str) -> Option<SavedSettings> {
    let line = text.lines().next()?;
    let fields: Vec<&str> = line.split('\t').collect();
    if fields.len() != 6 {
        return None;
    }
    Some(SavedSettings {
        version_idx: fields[0].trim().parse().ok()?,
        mode_idx: fields[1].trim().parse().ok()?,
        bots: fields[2].trim().parse().ok()?,
        seed_idx: fields[3].trim().parse().ok()?,
        miner_idx: fields[4].trim().parse().ok()?,
        close_on_launch: fields[5].trim() == "1",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::sample_versions_for_tests;

    fn state() -> LauncherState {
        let mut s = LauncherState::new(sample_versions_for_tests(), Vec::new(), 0);
        s.bots = 3;
        s.seed_idx = 2;
        s.miner_idx = 4;
        s.close_on_launch = true;
        s
    }

    #[test]
    fn settings_round_trip_and_apply() {
        let mut s = state();
        s.adjust(1); // versión siguiente
        let saved = SavedSettings::from_state(&s);
        let text = saved.serialize();
        let parsed = parse_settings(&text).expect("ajustes validos");
        assert_eq!(parsed, saved);
        let mut fresh = LauncherState::new(sample_versions_for_tests(), Vec::new(), 0);
        parsed.apply(&mut fresh);
        assert_eq!(fresh.version_idx, s.version_idx);
        assert_eq!(fresh.bots, 3);
        assert_eq!(fresh.seed_idx, 2);
        assert_eq!(fresh.miner_idx, 4);
        assert!(fresh.close_on_launch);
    }

    #[test]
    fn corrupt_settings_return_none_and_apply_clamps() {
        assert_eq!(parse_settings("basura"), None);
        assert_eq!(parse_settings("1\t2\t3\n"), None);
        assert_eq!(parse_settings("a\tb\tc\td\te\tf\n"), None);
        let raw = SavedSettings {
            version_idx: 99,
            mode_idx: 7,
            bots: 99,
            seed_idx: 41,
            miner_idx: 12,
            close_on_launch: true,
        };
        let mut s = LauncherState::new(sample_versions_for_tests(), Vec::new(), 0);
        raw.apply(&mut s);
        // Con 3 versiones de muestra, el índice 99 se acota a la última.
        assert_eq!(s.version_idx, 2);
        assert_eq!(s.bots, MAX_BOTS);
        assert_eq!(s.current_seed(), PRESET_SEEDS[41 % PRESET_SEEDS.len()]);
        assert_eq!(s.current_miner(), MINER_PROFILES[12 % MINER_PROFILES.len()]);
    }
}
