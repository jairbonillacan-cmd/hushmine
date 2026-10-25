//! Herramientas de terminal de HUSHMINE.
//!
//! Permiten a la IA (y a cualquiera) simular partidas, generar capturas y
//! consultar la GPU sin abrir el juego.

mod bench;
mod netsim;
mod scenes;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use grisu_render::{
    BoxRenderer, Gpu, OFFSCREEN_FORMAT, OrbitCamera, TextureData, UiRenderer, VoxelRenderer,
    capture, render_to_image,
};
use hush_sim::MatchConfig;
use scenes::Scene;

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
        /// Escena que se dibuja.
        #[arg(long, value_enum, default_value = "cube")]
        scene: Scene,
        /// Semilla del terreno de la escena.
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Ruta del PNG de salida.
        #[arg(long, default_value = "out/shot.png")]
        out: PathBuf,
        #[arg(long, default_value_t = 512)]
        width: u32,
        #[arg(long, default_value_t = 512)]
        height: u32,
        /// Giro horizontal de la cámara del cubo, en grados.
        #[arg(long, default_value_t = 35.0)]
        yaw: f32,
        /// Inclinación de la cámara del cubo, en grados.
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
        /// Bots de prueba que cavan durante la partida.
        #[arg(long, default_value_t = 4)]
        bots: u32,
        /// Ruta del informe JSON. Si se omite, se imprime en pantalla.
        #[arg(long)]
        report: Option<PathBuf>,
    },
    /// Partida en red simulada: servidor, clientes y bots con latencia y
    /// pérdida. Falla si algún cliente termina distinto del servidor.
    Netsim {
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value_t = 2)]
        clients: usize,
        #[arg(long, default_value_t = 6)]
        bots: u32,
        /// Latencia de ida y vuelta, en milisegundos.
        #[arg(long, default_value_t = 150)]
        rtt: u32,
        /// Pérdida de paquetes, en tanto por ciento.
        #[arg(long, default_value_t = 2.0)]
        loss: f32,
        /// Segundos de juego.
        #[arg(long, default_value_t = 60)]
        seconds: u64,
        /// Ruta del informe JSON. Si se omite, se imprime en pantalla.
        #[arg(long)]
        report: Option<PathBuf>,
    },
    /// Mide la generación del terreno y de sus mallas.
    Bench {
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Repeticiones de cada medida (se informa la mediana).
        #[arg(long, default_value_t = 10)]
        runs: u32,
        /// Ruta del informe JSON. Si se omite, se imprime en pantalla.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Muestra la GPU y el backend que usaría el juego.
    Gpu,
    /// Corre una expedición del modo historia sin gráficos y escribe un
    /// informe JSON (hitos, fisiología, enemigos, eventos).
    Story {
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Ticks a simular (20 por segundo).
        #[arg(long, default_value_t = 6_000)]
        ticks: u32,
        /// Sin enemigos (modo tranquilo).
        #[arg(long, default_value_t = false)]
        peaceful: bool,
        /// Ruta del informe JSON. Si se omite, se imprime en pantalla.
        #[arg(long)]
        report: Option<PathBuf>,
    },
    /// Guarda el mundo de historia en una región `.hush` y la recarga para
    /// verificar que la huella no cambia.
    Region {
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Ruta del archivo `.hush`.
        #[arg(long, default_value = "out/region.hush")]
        out: PathBuf,
    },
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    match Cli::parse().command {
        Command::Shot {
            scene,
            seed,
            out,
            width,
            height,
            yaw,
            pitch,
        } => shot(scene, seed, &out, [width, height], yaw, pitch),
        Command::Sim {
            seed,
            ticks,
            bots,
            report,
        } => sim(seed, ticks, bots, report.as_deref()),
        Command::Netsim {
            seed,
            clients,
            bots,
            rtt,
            loss,
            seconds,
            report,
        } => netsim::run(
            netsim::Options {
                seed,
                clients,
                bots,
                rtt,
                loss,
                seconds,
            },
            report.as_deref(),
        ),
        Command::Bench { seed, runs, out } => bench::run(seed, runs, out.as_deref()),
        Command::Gpu => gpu_info(),
        Command::Story {
            seed,
            ticks,
            peaceful,
            report,
        } => story(seed, ticks, peaceful, report.as_deref()),
        Command::Region { seed, out } => region(seed, &out),
    }
}

/// Segundos de la captura, fijos para que el PNG sea reproducible. Con 0.5
/// la lava muestra su fotograma 2: la captura ejercita la animación.
const SHOT_TIME_SECONDS: f32 = 0.5;

fn shot(scene: Scene, seed: u64, out: &Path, size: [u32; 2], yaw: f32, pitch: f32) -> Result<()> {
    let [width, height] = size;
    let gpu = pollster::block_on(Gpu::headless())?;
    let image = match scenes::build(scene, seed)? {
        None => {
            let camera = OrbitCamera {
                yaw: yaw.to_radians(),
                pitch: pitch.to_radians(),
                ..OrbitCamera::default()
            };
            render_to_image(&gpu, width, height, &camera, &TextureData::test_ore())?
        }
        Some(mut scene) => {
            scene.style.time_seconds = SHOT_TIME_SECONDS;
            let textures = hush_art::block_textures()?;
            let mut terrain =
                VoxelRenderer::new(&gpu.device, &gpu.queue, OFFSCREEN_FORMAT, &textures)?;
            terrain.enable_lighting(hush_art::emission_table());
            let stats = terrain.update(&gpu.device, &mut scene.world);
            println!(
                "{} chunks, {} caras, mallas en {:.1} ms",
                stats.chunks_meshed, stats.faces, stats.millis
            );
            let view = scene.view(width as f32 / height as f32);
            let mut boxes = BoxRenderer::new(&gpu.device, OFFSCREEN_FORMAT);
            let mut ui = UiRenderer::new(&gpu.device, OFFSCREEN_FORMAT);
            capture(&gpu, width, height, |encoder, color, depth| {
                terrain.render(&gpu.queue, encoder, color, depth, &view, &scene.style);
                let fog = scene.style.fog_distance;
                let device = &gpu.device;
                boxes.render(
                    device,
                    &gpu.queue,
                    encoder,
                    color,
                    depth,
                    &view,
                    fog,
                    &scene.boxes,
                );
                ui.render(
                    device,
                    &gpu.queue,
                    encoder,
                    color,
                    [width, height],
                    &scene.ui,
                );
            })?
        }
    };
    create_parent_dir(out)?;
    image
        .save(out)
        .with_context(|| format!("no se pudo guardar {}", out.display()))?;
    println!("captura guardada en {} ({width}×{height})", out.display());
    Ok(())
}

fn sim(seed: u64, ticks: Option<u32>, bots: u32, report_path: Option<&Path>) -> Result<()> {
    let mut config = MatchConfig {
        seed,
        bots,
        ..MatchConfig::default()
    };
    if let Some(ticks) = ticks {
        config.max_ticks = ticks;
    }
    let report = hush_sim::run(config)?;
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

/// Corre una expedición del modo historia con un minero automático simple
/// (camina, salta y pica hacia abajo cada cierto tiempo) y recopila el
/// informe en JSON.
fn story(seed: u64, ticks: u32, peaceful: bool, report_path: Option<&Path>) -> Result<()> {
    use hush_sim::MoveInput;
    use hush_sim::story::{Expedition, StoryConfig, StoryEventKind, StoryOrder};

    let config = StoryConfig {
        seed,
        peaceful,
        ..StoryConfig::default()
    };
    let mut expedition = Expedition::new(config)?;
    let mut milestones = Vec::new();
    let mut mined = 0u32;
    let mut damaged = 0u32;
    for t in 0..ticks {
        // Minero automático: avanza girando cada 300 ticks y pictea la
        // celda de sus pies cada 40 para abrir pozos de sondeo.
        let turn = (t / 300) % 4;
        let (wx, wz) = match turn {
            0 => (30, 0),
            1 => (0, 30),
            2 => (-30, 0),
            _ => (0, -30),
        };
        let input = MoveInput {
            wish_x: wx.clamp(-127, 127) as i8,
            wish_z: wz.clamp(-127, 127) as i8,
            jump: t % 90 == 0,
            sprint: false,
        };
        // Pica el bloque bajo los pies en tandas continuas: el progreso se
        // reinicia si la orden se alterna con `Mine(None)`.
        let mut orders = vec![StoryOrder::Move(input)];
        if t % 80 < 40 {
            let feet = expedition.feet_cell();
            orders.push(StoryOrder::Mine(Some([feet[0], feet[1] - 1, feet[2]])));
        }
        for event in expedition.step(&orders) {
            match event.kind {
                StoryEventKind::Mined(_) | StoryEventKind::CrushedToDust(_) => mined += 1,
                StoryEventKind::Damaged { amount, .. } => damaged += u32::from(amount),
                StoryEventKind::MilestoneReached(m) => milestones.push(m),
                _ => {}
            }
        }
        if expedition.is_dead() {
            break;
        }
    }
    let summary = serde_json::json!({
        "seed": seed,
        "peaceful": peaceful,
        "ticks": expedition.tick(),
        "dead": expedition.is_dead(),
        "state_hash": expedition.state_hash().to_string(),
        "position": expedition.feet_cell(),
        "stratum": expedition.bands().at(expedition.feet_cell()[1]).label(),
        "clock_tick": expedition.clock().tick(),
        "phase": format!("{:?}", expedition.clock().phase()),
        "health": expedition.vitals().health,
        "food": expedition.vitals().food,
        "air": expedition.vitals().air,
        "milestones": milestones,
        "mined_or_crushed": mined,
        "damage_taken": damaged,
        "mobs_alive": expedition.mobs().list.len(),
        "world_hash": expedition.world().content_hash().to_string(),
    });
    let json = serde_json::to_string_pretty(&summary)?;
    match report_path {
        Some(path) => {
            create_parent_dir(path)?;
            fs::write(path, &json)
                .with_context(|| format!("no se pudo escribir {}", path.display()))?;
            println!("informe guardado en {}", path.display());
        }
        None => println!("{json}"),
    }
    Ok(())
}

/// Guarda y recarga una región `.hush`, verificando la huella del mundo.
fn region(seed: u64, out: &Path) -> Result<()> {
    use hush_sim::story::generate_terrain as generate_story;

    let generated = generate_story(seed, &Default::default())?;
    let bytes = hush_sim::story::region::encode_world(&generated.world);
    create_parent_dir(out)?;
    hush_sim::story::region::save_world(&generated.world, out)
        .with_context(|| format!("no se pudo guardar {}", out.display()))?;
    let loaded = hush_sim::story::region::load_world(out)
        .with_context(|| format!("no se pudo recargar {}", out.display()))?;
    anyhow::ensure!(
        loaded.content_hash() == generated.world.content_hash(),
        "la huella del mundo cambió al recargar la región"
    );
    let solid = (0..loaded.chunk_count())
        .map(|i| {
            loaded
                .chunk(i)
                .as_bytes()
                .iter()
                .filter(|&&b| b != 0)
                .count()
        })
        .sum::<usize>();
    println!(
        "región {} ({} bytes, {} bloques sólidos): huella conservada",
        out.display(),
        bytes.len(),
        solid
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

pub(crate) fn create_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .with_context(|| format!("no se pudo crear {}", parent.display()))?;
    }
    Ok(())
}
