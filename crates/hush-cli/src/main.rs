//! Herramientas de terminal de HUSHMINE.
//!
//! Permiten a la IA (y a cualquiera) simular partidas, generar capturas y
//! consultar la GPU sin abrir el juego.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use grisu_render::{Gpu, OrbitCamera, TextureData, render_to_image};
use hush_sim::MatchConfig;

#[derive(Parser)]
#[command(name = "hush-cli", version, about = "Herramientas de HUSHMINE")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Dibuja una escena sin ventana y la guarda como PNG.
    Shot {
        /// Ruta del PNG de salida.
        #[arg(long, default_value = "out/shot.png")]
        out: PathBuf,
        #[arg(long, default_value_t = 512)]
        width: u32,
        #[arg(long, default_value_t = 512)]
        height: u32,
        /// Giro horizontal de la cámara, en grados.
        #[arg(long, default_value_t = 35.0)]
        yaw: f32,
        /// Inclinación de la cámara, en grados.
        #[arg(long, default_value_t = 28.0)]
        pitch: f32,
    },
    /// Simula una partida sin gráficos y escribe un informe JSON.
    Sim {
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Ticks a simular (30 por segundo). Por defecto, 16 minutos.
        #[arg(long)]
        ticks: Option<u32>,
        /// Ruta del informe JSON. Si se omite, se imprime en pantalla.
        #[arg(long)]
        report: Option<PathBuf>,
    },
    /// Muestra la GPU y el backend que usaría el juego.
    Gpu,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    match Cli::parse().command {
        Command::Shot {
            out,
            width,
            height,
            yaw,
            pitch,
        } => shot(&out, width, height, yaw, pitch),
        Command::Sim {
            seed,
            ticks,
            report,
        } => sim(seed, ticks, report.as_deref()),
        Command::Gpu => gpu_info(),
    }
}

fn shot(out: &Path, width: u32, height: u32, yaw: f32, pitch: f32) -> Result<()> {
    let gpu = pollster::block_on(Gpu::headless())?;
    let camera = OrbitCamera {
        yaw: yaw.to_radians(),
        pitch: pitch.to_radians(),
        ..OrbitCamera::default()
    };
    let image = render_to_image(&gpu, width, height, &camera, &TextureData::test_ore())?;
    create_parent_dir(out)?;
    image
        .save(out)
        .with_context(|| format!("no se pudo guardar {}", out.display()))?;
    println!("captura guardada en {} ({width}×{height})", out.display());
    Ok(())
}

fn sim(seed: u64, ticks: Option<u32>, report_path: Option<&Path>) -> Result<()> {
    let mut config = MatchConfig {
        seed,
        ..MatchConfig::default()
    };
    if let Some(ticks) = ticks {
        config.max_ticks = ticks;
    }
    let report = hush_sim::run(config);
    let json = serde_json::to_string_pretty(&report)?;
    match report_path {
        Some(path) => {
            create_parent_dir(path)?;
            fs::write(path, &json)
                .with_context(|| format!("no se pudo escribir {}", path.display()))?;
            println!("informe guardado en {}", path.display());
        }
        None => println!("{json}"),
    }
    anyhow::ensure!(
        report.broken_invariants.is_empty(),
        "invariantes rotas: {:?}",
        report.broken_invariants
    );
    Ok(())
}

fn gpu_info() -> Result<()> {
    let gpu = pollster::block_on(Gpu::headless())?;
    let info = gpu.adapter.get_info();
    println!("GPU:     {}", info.name);
    println!("Backend: {:?}", info.backend);
    println!("Tipo:    {:?}", info.device_type);
    println!("Driver:  {} {}", info.driver, info.driver_info);
    Ok(())
}

fn create_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .with_context(|| format!("no se pudo crear {}", parent.display()))?;
    }
    Ok(())
}
