//! `hush-cli world map` y `world slice`: mapas y cortes del mundo en PNG.
//!
//! El mapa es una vista cenital de `size` bloques centrada en `center` (con
//! un máximo de 1024×1024 píxeles: si la ventana es mayor, se muestrea cada
//! `step` bloques) y el corte es una rebanada vertical coloreada por bloque.
//! Ambos consultan el mundo con las funciones puras de `hush_worldgen`
//! (`surface_height` y `block_at`), sin generar chunks.

use std::path::Path;

use anyhow::{Context, Result, ensure};
use clap::ValueEnum;
use grisu_voxel::{AIR, BlockId};
use hush_worldgen::{
    AMPLITUDE, DIAMOND, DIRT, GOLD, HARD_ROCK, ORE, ROCK, block_at, surface_height,
};
use image::{Rgba, RgbaImage};

/// Lado máximo del PNG del mapa, en píxeles.
const MAX_MAP_DIM: u32 = 1024;
/// Bloques que la capa de menas escanea bajo la superficie de cada columna.
const ORE_SCAN_DEPTH: i32 = 32;
/// Cota fija que examina la capa de cuevas.
const CAVE_LEVEL: i64 = -8;
/// Máximo de ancho y alto del corte, en bloques.
const MAX_SLICE_DIM: i64 = 8192;
/// Lava: id 7 de la Convención A (sin constante en `hush_worldgen` aún).
const LAVA: BlockId = 7;

/// Paradas de la rampa hipsométrica: azul de hondo, verde de llano, marrón
/// de ladera y casi blanco en la cima.
const HYPSO: [[u8; 3]; 4] = [
    [24, 60, 120],
    [46, 110, 58],
    [150, 116, 64],
    [238, 238, 238],
];

/// Capa que dibuja `world map`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum MapLayer {
    /// Rampa hipsométrica del relieve con sombreado de ladera.
    Height,
    /// Proyección de la mena más valiosa de cada columna.
    Ores,
    /// Aire frente a sólido a una cota fija.
    Caves,
}

/// Eje perpendicular al corte de `world slice`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum SliceAxis {
    /// Corte en el plano «z-y» a la coordenada x = at.
    X,
    /// Corte en el plano «x-y» a la coordenada z = at.
    Z,
}

/// Analiza un par de coordenadas con la forma «x,z» o «h,y» (admite negativos).
pub fn parse_pair(text: &str) -> Option<(i32, i32)> {
    let mut parts = text.split(',');
    let first = parts.next()?.trim().parse::<i32>().ok()?;
    let second = parts.next()?.trim().parse::<i32>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((first, second))
}

/// Renderiza en memoria el mapa de `size` bloques centrado en `center`.
pub fn render_map(seed: u64, center: (i32, i32), size: u32, layer: MapLayer) -> RgbaImage {
    // Ventana mayor que el PNG: un píxel cada `step` bloques del mundo.
    let dim = size.clamp(1, MAX_MAP_DIM);
    // Con dim = 1024 el paso máximo es u32::MAX / 1024 ≈ 4,2 M: cabe en i32.
    let step = (size / dim).max(1) as i32;
    let origin = (center.0 - size as i32 / 2, center.1 - size as i32 / 2);
    RgbaImage::from_fn(dim, dim, |px, pz| {
        let (x, z) = (origin.0 + px as i32 * step, origin.1 + pz as i32 * step);
        match layer {
            MapLayer::Height => height_pixel(seed, x, z, step),
            MapLayer::Ores => ores_pixel(seed, x, z),
            MapLayer::Caves => caves_pixel(seed, x, z),
        }
    })
}

/// Píxel de la capa de altura: rampa hipsométrica más sombreado de ladera.
fn height_pixel(seed: u64, x: i32, z: i32, step: i32) -> Rgba<u8> {
    let h = surface_height(seed, x, z);
    // El relieve vive en −AMPLITUDE..=AMPLITUDE: t = 0 es lo más hondo.
    let t = (h + AMPLITUDE) as f32 / (2 * AMPLITUDE) as f32;
    let base = ramp_color(t);
    // Luz según la pendiente hacia el noroeste: las laderas que miran allá
    // se iluminan y las que dan la espalda se oscurecen.
    let dh = h - surface_height(seed, x - step, z - step);
    let light = (1.0 + 0.15 * dh as f32).clamp(0.55, 1.45);
    let shade = |c: u8| (c as f32 * light).clamp(0.0, 255.0) as u8;
    Rgba([shade(base[0]), shade(base[1]), shade(base[2]), 255])
}

/// Píxel de la capa de menas: color de la mena más valiosa de la columna
/// (diamante > oro > mena) o fondo gris teñido por la altura.
fn ores_pixel(seed: u64, x: i32, z: i32) -> Rgba<u8> {
    let surface = surface_height(seed, x, z);
    let (mut best_rank, mut color) = (0u8, [0u8; 3]);
    for depth in 1..=ORE_SCAN_DEPTH {
        let y = surface - depth;
        let block = block_at(seed, i64::from(x), i64::from(y), i64::from(z));
        let rank = ore_rank(block);
        if rank > best_rank {
            best_rank = rank;
            color = match block {
                DIAMOND => [0, 255, 255],
                GOLD => [255, 215, 0],
                _ => [184, 115, 51],
            };
        }
    }
    if best_rank == 0 {
        // Sin mena: un gris con leve matiz por altura para que el relieve
        // se lea aunque la columna esté vacía.
        let gray = (96 + surface * 8).clamp(0, 255) as u8;
        color = [gray, gray, gray];
    }
    Rgba([color[0], color[1], color[2], 255])
}

/// Valor de una mena para `ores_pixel`: 3 diamante, 2 oro, 1 mena, 0 nada.
fn ore_rank(block: BlockId) -> u8 {
    match block {
        ORE => 1,
        GOLD => 2,
        DIAMOND => 3,
        _ => 0,
    }
}

/// Píxel de la capa de cuevas: qué hay a la cota fija `CAVE_LEVEL`.
fn caves_pixel(seed: u64, x: i32, z: i32) -> Rgba<u8> {
    let block = block_at(seed, i64::from(x), CAVE_LEVEL, i64::from(z));
    let color = match block {
        AIR => [12, 12, 24],
        LAVA => [255, 120, 0],
        _ => [198, 198, 198],
    };
    Rgba([color[0], color[1], color[2], 255])
}

/// Interpola linealmente las paradas de `HYPSO` en `t` (recortado a 0..=1).
fn ramp_color(t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    let scaled = t * (HYPSO.len() - 1) as f32;
    let index = (scaled.floor() as usize).min(HYPSO.len() - 2);
    let blend = scaled - index as f32;
    let mix = |channel: usize| {
        let a = f32::from(HYPSO[index][channel]);
        let b = f32::from(HYPSO[index + 1][channel]);
        (a + (b - a) * blend) as u8
    };
    [mix(0), mix(1), mix(2)]
}

/// Renderiza en memoria el corte vertical entre `from` y `to`.
pub fn render_slice(
    seed: u64,
    axis: SliceAxis,
    at: i32,
    from: (i32, i32),
    to: (i32, i32),
) -> RgbaImage {
    // `from`/`to` pares «h,y» sin orden garantizado: normalízalos.
    let (h0, h1) = (from.0.min(to.0), from.0.max(to.0));
    let (y0, y1) = (from.1.min(to.1), from.1.max(to.1));
    // Dimensiones en i64: los extremos de i32 no desbordan al restar.
    let width = (i64::from(h1) - i64::from(h0) + 1) as u32;
    let height = (i64::from(y1) - i64::from(y0) + 1) as u32;
    RgbaImage::from_fn(width, height, |px, py| {
        // La fila superior del PNG es la cota más alta del corte.
        let h = h0 + px as i32;
        let y = y1 - py as i32;
        let block = match axis {
            SliceAxis::X => block_at(seed, i64::from(at), i64::from(y), i64::from(h)),
            SliceAxis::Z => block_at(seed, i64::from(h), i64::from(y), i64::from(at)),
        };
        slice_color(block)
    })
}

/// Color de cada bloque del corte; el magenta delata ids desconocidos.
fn slice_color(block: BlockId) -> Rgba<u8> {
    let rgb = match block {
        AIR => [12, 12, 36],
        DIRT => [134, 87, 51],
        ROCK => [128, 128, 128],
        HARD_ROCK => [72, 78, 86],
        ORE => [184, 115, 51],
        GOLD => [255, 208, 60],
        DIAMOND => [0, 255, 255],
        LAVA => [255, 120, 0],
        _ => [255, 0, 255],
    };
    Rgba([rgb[0], rgb[1], rgb[2], 255])
}

/// Nombre en español de cada capa, para los mensajes.
fn layer_name(layer: MapLayer) -> &'static str {
    match layer {
        MapLayer::Height => "altura",
        MapLayer::Ores => "menas",
        MapLayer::Caves => "cuevas",
    }
}

/// Nombre en español de cada eje, para los mensajes.
fn axis_name(axis: SliceAxis) -> &'static str {
    match axis {
        SliceAxis::X => "x",
        SliceAxis::Z => "z",
    }
}

/// Ejecuta el subcomando `world map`.
pub fn run_map(
    seed: u64,
    center: (i32, i32),
    size: u32,
    layer: MapLayer,
    out: &Path,
) -> Result<()> {
    ensure!(size >= 1, "el lado del mapa debe ser al menos 1, no {size}");
    let map = render_map(seed, center, size, layer);
    crate::create_parent_dir(out)?;
    let (width, height) = map.dimensions();
    map.save(out)
        .with_context(|| format!("no se pudo guardar {}", out.display()))?;
    println!(
        "mapa de {} guardado en {} ({width}×{height}, semilla {seed})",
        layer_name(layer),
        out.display()
    );
    Ok(())
}

/// Ejecuta el subcomando `world slice`.
pub fn run_slice(
    seed: u64,
    axis: SliceAxis,
    at: i32,
    from: (i32, i32),
    to: (i32, i32),
    out: &Path,
) -> Result<()> {
    let (h0, h1) = (from.0.min(to.0), from.0.max(to.0));
    let (y0, y1) = (from.1.min(to.1), from.1.max(to.1));
    let width = i64::from(h1) - i64::from(h0) + 1;
    let height = i64::from(y1) - i64::from(y0) + 1;
    let limit = MAX_SLICE_DIM;
    ensure!(
        (1..=limit).contains(&width) && (1..=limit).contains(&height),
        "el corte quedaría de {width}×{height} bloques; cada dimensión debe estar entre 1 y {limit}"
    );
    let slice = render_slice(seed, axis, at, from, to);
    crate::create_parent_dir(out)?;
    let (width, height) = slice.dimensions();
    slice
        .save(out)
        .with_context(|| format!("no se pudo guardar {}", out.display()))?;
    println!(
        "corte en el eje {} a {} = {at} guardado en {} ({width}×{height}, semilla {seed})",
        axis_name(axis),
        axis_name(axis),
        out.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pair_accepts_two_i32() {
        assert_eq!(parse_pair("0,0"), Some((0, 0)));
        assert_eq!(parse_pair("-468750,468749"), Some((-468750, 468749)));
        assert_eq!(parse_pair(" -1 , 2 "), Some((-1, 2)));
    }

    #[test]
    fn parse_pair_rejects_anything_else() {
        for bad in ["1", "1,2,3", "a,b", "", "1,", ",2", "1.5,2", "  "] {
            assert_eq!(parse_pair(bad), None, "debería rechazar «{bad}»");
        }
    }

    #[test]
    fn height_map_is_opaque_and_has_relief() {
        let map = render_map(42, (0, 0), 8, MapLayer::Height);
        assert_eq!(map.dimensions(), (8, 8));
        let mut colors = std::collections::HashSet::new();
        for pixel in map.pixels() {
            assert_eq!(pixel[3], 255, "el mapa debe ser opaco");
            colors.insert(pixel.0);
        }
        assert!(
            colors.len() >= 2,
            "el relieve de una ventana 8×8 no debería ser plano"
        );
    }

    #[test]
    fn map_respects_the_1024_maximum() {
        let map = render_map(42, (0, 0), 2048, MapLayer::Caves);
        assert_eq!(map.dimensions(), (1024, 1024));
    }

    #[test]
    fn slice_dimensions_and_normalization() {
        let slice = render_slice(42, SliceAxis::X, 0, (0, 0), (15, 8));
        assert_eq!(slice.dimensions(), (16, 9));
        // Con `from` y `to` invertidos las dimensiones son las mismas.
        let swapped = render_slice(42, SliceAxis::X, 0, (15, 8), (0, 0));
        assert_eq!(swapped.dimensions(), (16, 9));
    }

    #[test]
    fn slice_below_surface_shows_solids() {
        let slice = render_slice(42, SliceAxis::X, 0, (-4, -4), (4, 4));
        let dirt = Rgba([134, 87, 51, 255]);
        let rock = Rgba([128, 128, 128, 255]);
        assert!(
            slice.pixels().any(|p| *p == dirt || *p == rock),
            "bajo y = 0 debería verse tierra o roca"
        );
    }
}
