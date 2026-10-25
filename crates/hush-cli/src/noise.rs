//! Mapa en escala de grises de un fBm 2D, para revisar el ruido a ojo.

use std::path::Path;

use anyhow::{Context, Result};
use grisu_noise::{Fbm2, ONE, ridge};
use image::{GrayImage, Luma};

/// Sal de la capa de ruido que dibuja el mapa.
const SALT: u64 = 0x5EED;

/// Opciones del subcomando `noise`.
pub struct Options {
    pub seed: u64,
    /// Lado del PNG en píxeles (1 píxel = 1 bloque).
    pub size: u32,
    pub octaves: u32,
    /// Período de la octava base, en bloques (potencia de dos).
    pub period: u32,
    /// Dibuja `65 536 − |ruido|` en vez del ruido tal cual.
    pub ridged: bool,
}

/// Dibuja el mapa y lo guarda como PNG.
pub fn run(options: &Options, out: &Path) -> Result<()> {
    anyhow::ensure!(
        options.period.is_power_of_two(),
        "el período {} no es una potencia de dos",
        options.period
    );
    let fbm = Fbm2::new(
        options.seed,
        SALT,
        options.period.trailing_zeros(),
        options.octaves,
    );
    let map = GrayImage::from_fn(options.size, options.size, |px, py| {
        let n = fbm.sample_block(i64::from(px), i64::from(py));
        // [-ONE, ONE] → [0, 255]; en cresta, [0, ONE] → [0, 255].
        let gray = if options.ridged {
            (ridge(n) >> 8).min(255)
        } else {
            ((n + ONE) >> 9).min(255)
        };
        Luma([gray as u8])
    });
    super::create_parent_dir(out)?;
    map.save(out)
        .with_context(|| format!("no se pudo guardar {}", out.display()))?;
    let kind = if options.ridged { "en cresta" } else { "fBm" };
    println!(
        "mapa de ruido ({kind}) guardado en {} ({}×{}, {} octavas, período {} bloques)",
        out.display(),
        options.size,
        options.size,
        options.octaves,
        options.period
    );
    Ok(())
}
