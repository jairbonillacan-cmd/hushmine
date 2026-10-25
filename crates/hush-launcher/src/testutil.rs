//! Datos de muestra para las pruebas del panel, sin E/S real.

use std::path::PathBuf;

use crate::versions::{GameVersion, VersionKind};

/// Versiones de muestra: release, debug y servidor LAN.
pub fn sample_versions_for_tests() -> Vec<GameVersion> {
    vec![
        GameVersion {
            label: "V1 RELEASE".into(),
            subtitle: "target/release/hushmine.exe".into(),
            kind: VersionKind::Release,
            binary_path: PathBuf::from("release/hushmine.exe"),
        },
        GameVersion {
            label: "V1 DEBUG".into(),
            subtitle: "target/debug/hushmine.exe".into(),
            kind: VersionKind::Debug,
            binary_path: PathBuf::from("debug/hushmine.exe"),
        },
        GameVersion {
            label: "V1 SERVIDOR + CLIENTE LAN".into(),
            subtitle: "hush-server UDP (47400) + cliente".into(),
            kind: VersionKind::LanHostAndClient,
            binary_path: PathBuf::from("release/hushmine.exe"),
        },
    ]
}
