//! Argumentos y despacho del subcomando `world`.
//!
//! Cada variante declara sus opciones con clap y [`run`] se encarga de
//! analizar las coordenadas en texto («x,z», «h,y», «cx,cy,cz») antes de
//! llamar a los módulos que hacen el trabajo: `world_img`, `world_chunk` y
//! `world_bench`.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Subcommand;

use crate::world_bench;
use crate::world_chunk;
use crate::world_img::{self, MapLayer, SliceAxis};

/// Subcomandos de `world`: mapa, corte, resumen de chunk y medida.
#[derive(Subcommand)]
pub enum WorldAction {
    /// Dibuja un mapa cenital del terreno en PNG.
    Map {
        /// Semilla del mundo.
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Centro del mapa, con la forma «x,z».
        #[arg(long, default_value = "0,0", allow_hyphen_values = true)]
        center: String,
        /// Lado del mapa en píxeles (1 píxel = 1 bloque).
        #[arg(long, default_value_t = 512)]
        size: u32,
        /// Capa que se dibuja.
        #[arg(long, value_enum, default_value = "height")]
        layer: MapLayer,
        /// Ruta del PNG de salida.
        #[arg(long, default_value = "out/world/map.png")]
        out: PathBuf,
    },
    /// Dibuja un corte vertical del terreno en PNG.
    Slice {
        /// Semilla del mundo.
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Eje perpendicular al corte: el plano queda a esa coordenada.
        #[arg(long, value_enum, default_value = "x")]
        axis: SliceAxis,
        /// Coordenada del plano de corte en ese eje.
        #[arg(long, default_value_t = 0, allow_hyphen_values = true)]
        at: i32,
        /// Esquina del corte, con la forma «h,y».
        #[arg(long, allow_hyphen_values = true)]
        from: String,
        /// Esquina opuesta del corte, con la forma «h,y».
        #[arg(long, allow_hyphen_values = true)]
        to: String,
        /// Ruta del PNG de salida.
        #[arg(long, default_value = "out/world/slice.png")]
        out: PathBuf,
    },
    /// Resumen SHA-256 y recuento de bloques de un chunk.
    Chunk {
        /// Semilla del mundo.
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Posición del chunk, con la forma «cx,cy,cz».
        #[arg(long, allow_hyphen_values = true)]
        chunk: String,
        /// Imprime también la línea hexadecimal del resumen.
        #[arg(long)]
        digest: bool,
    },
    /// Mide los tiempos de generación por chunk.
    Bench {
        /// Semilla del mundo.
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Número de chunks de la muestra.
        #[arg(long, default_value_t = 512)]
        chunks: usize,
        /// Ruta del CSV de detalle por chunk. Si se omite, no se escribe.
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

/// Ejecuta la subacción elegida de `world`.
pub fn run(action: WorldAction) -> Result<()> {
    match action {
        WorldAction::Map {
            seed,
            center,
            size,
            layer,
            out,
        } => {
            let center = parse_pair_arg(&center, "--center", "«x,z»")?;
            world_img::run_map(seed, center, size, layer, &out)
        }
        WorldAction::Slice {
            seed,
            axis,
            at,
            from,
            to,
            out,
        } => {
            let from = parse_pair_arg(&from, "--from", "«h,y»")?;
            let to = parse_pair_arg(&to, "--to", "«h,y»")?;
            world_img::run_slice(seed, axis, at, from, to, &out)
        }
        WorldAction::Chunk {
            seed,
            chunk,
            digest,
        } => {
            let chunk = world_chunk::parse_chunk(&chunk)
                .with_context(|| format!("--chunk debe tener la forma «cx,cy,cz», no «{chunk}»"))?;
            world_chunk::run(seed, chunk, digest)
        }
        WorldAction::Bench { seed, chunks, out } => world_bench::run(seed, chunks, out.as_deref()),
    }
}

/// Analiza un argumento de par de coordenadas («x,z» o «h,y») y devuelve un
/// error que explica la forma esperada.
fn parse_pair_arg(text: &str, flag: &str, shape: &str) -> Result<(i32, i32)> {
    world_img::parse_pair(text)
        .with_context(|| format!("{flag} debe tener la forma {shape}, no «{text}»"))
}
