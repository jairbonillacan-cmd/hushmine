//! Descubrimiento de versiones instaladas, compilación con Cargo y arranque de
//! procesos de `hushmine.exe` y `hush-server.exe`.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Tipo de versión o perfil ejecutable desde el Launcher.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VersionKind {
    /// Binario optimizado (`target/release/hushmine.exe`).
    Release,
    /// Binario de desarrollo (`target/debug/hushmine.exe`).
    Debug,
    /// Servidor dedicado local + cliente conectado por UDP (`127.0.0.1:47400`).
    LanHostAndClient,
    /// Ejecutable archivado en la carpeta `versions/` (p. ej. hitos anteriores).
    Archived(PathBuf),
}

/// Entrada de versión mostrada en el selector del Launcher.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameVersion {
    pub label: String,
    pub subtitle: String,
    pub kind: VersionKind,
    pub binary_path: PathBuf,
}

impl GameVersion {
    pub fn is_installed(&self) -> bool {
        self.binary_path.is_file()
    }

    pub fn status_tag(&self) -> &'static str {
        if self.is_installed() {
            "INSTALADO"
        } else {
            "SIN COMPILAR"
        }
    }
}

/// Modo en el que arranca el cliente al pulsar Jugar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StartMode {
    /// Entra directamente en la mina con la semilla y los bots elegidos.
    #[default]
    DirectMatch,
    /// Abre primero la Caseta del Vigía dentro del cliente.
    InGameMenu,
    /// Se conecta a un servidor LAN en `127.0.0.1:47400`.
    ConnectLocalLan,
}

impl StartMode {
    pub fn next(self) -> Self {
        match self {
            Self::DirectMatch => Self::InGameMenu,
            Self::InGameMenu => Self::ConnectLocalLan,
            Self::ConnectLocalLan => Self::DirectMatch,
        }
    }

    /// Índice estable para persistir en el archivo de ajustes.
    pub fn index(self) -> usize {
        match self {
            Self::DirectMatch => 0,
            Self::InGameMenu => 1,
            Self::ConnectLocalLan => 2,
        }
    }

    /// Inverso de [`StartMode::index`]; los valores fuera de rango dan vueltas.
    pub fn from_index(idx: usize) -> Self {
        match idx % 3 {
            0 => Self::DirectMatch,
            1 => Self::InGameMenu,
            _ => Self::ConnectLocalLan,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::DirectMatch => "DIRECTO A PARTIDA",
            Self::InGameMenu => "CASETA DEL VIGIA (MENU)",
            Self::ConnectLocalLan => "CONECTAR A LAN (127.0.0.1)",
        }
    }
}

/// Raíz del proyecto o carpeta desde donde corre el Launcher.
pub fn workspace_root() -> PathBuf {
    if let Ok(exe) = env::current_exe()
        && let Some(parent) = exe.parent()
    {
        if (parent.ends_with("release") || parent.ends_with("debug"))
            && let Some(root) = parent.parent().and_then(Path::parent)
            && root.join("Cargo.toml").is_file()
        {
            return root.to_path_buf();
        }
        if parent.join("Cargo.toml").is_file() {
            return parent.to_path_buf();
        }
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn exe_name(stem: &str) -> String {
    if cfg!(windows) {
        format!("{stem}.exe")
    } else {
        stem.to_owned()
    }
}

/// Descubre todas las versiones disponibles (perfiles de Cargo + carpeta `versions/`).
pub fn discover_versions(root: &Path) -> Vec<GameVersion> {
    let rel_bin = root
        .join("target")
        .join("release")
        .join(exe_name("hushmine"));
    let dbg_bin = root.join("target").join("debug").join(exe_name("hushmine"));

    let mut list = vec![
        GameVersion {
            label: "v0.2.0 RELEASE (OPTIMIZADO)".into(),
            subtitle: "target/release/hushmine.exe · Maximo rendimiento".into(),
            kind: VersionKind::Release,
            binary_path: rel_bin.clone(),
        },
        GameVersion {
            label: "v0.2.0 DEBUG (DESARROLLO)".into(),
            subtitle: "target/debug/hushmine.exe · Compilacion rapida".into(),
            kind: VersionKind::Debug,
            binary_path: dbg_bin,
        },
        GameVersion {
            label: "v0.2.0 SERVIDOR + CLIENTE LAN".into(),
            subtitle: "Arranca hush-server UDP (47400) + cliente conectado".into(),
            kind: VersionKind::LanHostAndClient,
            binary_path: rel_bin,
        },
    ];

    let versions_dir = root.join("versions");
    if let Ok(entries) = fs::read_dir(&versions_dir) {
        let mut custom: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && p.extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
            })
            .collect();
        custom.sort();
        for path in custom {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                list.push(GameVersion {
                    label: format!("ARCHIVADA: {}", stem.to_uppercase()),
                    subtitle: format!(
                        "versions/{}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ),
                    kind: VersionKind::Archived(path.clone()),
                    binary_path: path,
                });
            }
        }
    }
    list
}

/// Compila la versión seleccionada invocando `cargo build`.
pub fn compile_version(root: &Path, version: &GameVersion) -> Result<String, String> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(root).arg("build");
    match version.kind {
        VersionKind::Release => {
            cmd.arg("--release").arg("-p").arg("hush-client");
        }
        VersionKind::Debug => {
            cmd.arg("-p").arg("hush-client");
        }
        VersionKind::LanHostAndClient => {
            cmd.arg("--release")
                .arg("-p")
                .arg("hush-client")
                .arg("-p")
                .arg("hush-server");
        }
        VersionKind::Archived(_) => {
            return Ok("VERSION ARCHIVADA LISTA PARA EJECUTAR".into());
        }
    }
    match cmd.status() {
        Ok(status) if status.success() => Ok(format!("COMPILACION COMPLETADA: {}", version.label)),
        Ok(status) => Err(format!("CARGO TERMINO CON CODIGO {:?}", status.code())),
        Err(err) => Err(format!("ERROR AL EJECUTAR CARGO: {err}")),
    }
}

/// Ruta del binario del servidor dedicado, release si existe y debug si no.
pub fn server_binary(root: &Path) -> PathBuf {
    let srv_rel = root
        .join("target")
        .join("release")
        .join(exe_name("hush-server"));
    let srv_dbg = root
        .join("target")
        .join("debug")
        .join(exe_name("hush-server"));
    if srv_rel.is_file() { srv_rel } else { srv_dbg }
}

/// Argumentos de arranque del servidor dedicado, puros para poder probarlos.
pub fn server_args(seed: u64, bots: u32) -> Vec<String> {
    vec![
        "--seed".into(),
        seed.to_string(),
        "--bots".into(),
        bots.to_string(),
    ]
}

/// Compila el servidor dedicado si no existe el binario.
fn ensure_server_built(root: &Path) -> Result<(), String> {
    if server_binary(root).is_file() {
        return Ok(());
    }
    let status = Command::new("cargo")
        .current_dir(root)
        .arg("build")
        .arg("--release")
        .arg("-p")
        .arg("hush-server")
        .status();
    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("CARGO TERMINO CON CODIGO {:?}", status.code())),
        Err(err) => Err(format!("ERROR AL EJECUTAR CARGO: {err}")),
    }
}

/// Arranca el servidor dedicado LAN con la semilla y los bots elegidos.
pub fn launch_server(root: &Path, seed: u64, bots: u32) -> Result<String, String> {
    ensure_server_built(root)?;
    let bin = server_binary(root);
    let mut cmd = Command::new(&bin);
    cmd.current_dir(root);
    for arg in server_args(seed, bots) {
        cmd.arg(arg);
    }
    cmd.spawn()
        .map(|child| {
            format!(
                "SERVIDOR LAN EN MARCHA (PID {}) - PUERTO 47400 - SEMILLA {seed}",
                child.id()
            )
        })
        .map_err(|err| format!("NO SE PUDO LANZAR EL SERVIDOR: {err}"))
}

/// Lanza el proceso del juego con los parámetros configurados.
pub fn launch_game(
    root: &Path,
    version: &GameVersion,
    mode: StartMode,
    seed: u64,
    bots: u32,
    miner_name: &str,
) -> Result<String, String> {
    if !version.is_installed() {
        compile_version(root, version)?;
    }

    if version.kind == VersionKind::LanHostAndClient
        && let Ok(()) = ensure_server_built(root)
    {
        let srv_bin = server_binary(root);
        let mut srv = Command::new(&srv_bin);
        srv.current_dir(root);
        for arg in server_args(seed, bots) {
            srv.arg(arg);
        }
        let _ = srv.spawn();
    }

    let mut cmd = Command::new(&version.binary_path);
    cmd.current_dir(root)
        .arg("--seed")
        .arg(seed.to_string())
        .arg("--bots")
        .arg(bots.to_string())
        .arg("--name")
        .arg(miner_name);

    if version.kind == VersionKind::LanHostAndClient || mode == StartMode::ConnectLocalLan {
        cmd.arg("--connect").arg("127.0.0.1:47400");
    } else if mode == StartMode::InGameMenu {
        cmd.arg("--launcher");
    }

    cmd.spawn()
        .map(|child| format!("JUEGO LANZADO (PID {}) - {}", child.id(), version.label))
        .map_err(|err| format!("NO SE PUDO LANZAR EL BINARIO: {err}"))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_standard_versions_and_cycles_modes() {
        let root = workspace_root();
        let list = discover_versions(&root);
        assert!(list.len() >= 3);
        assert_eq!(list[0].kind, VersionKind::Release);
        assert_eq!(list[1].kind, VersionKind::Debug);
        assert_eq!(list[2].kind, VersionKind::LanHostAndClient);

        let m = StartMode::default();
        assert_eq!(m.next(), StartMode::InGameMenu);
        assert_eq!(m.next().next(), StartMode::ConnectLocalLan);
        assert_eq!(m.next().next().next(), StartMode::DirectMatch);
    }

    #[test]
    fn server_args_are_stable() {
        assert_eq!(
            server_args(42, 4),
            vec![
                "--seed".to_owned(),
                "42".to_owned(),
                "--bots".to_owned(),
                "4".to_owned()
            ]
        );
    }
}
