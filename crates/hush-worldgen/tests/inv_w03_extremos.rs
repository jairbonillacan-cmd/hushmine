//! `INV-W03`: coordenadas extremas — sin desbordamientos en ±7 499 999 y en
//! coordenadas negativas, y las proporciones de bloques se mantienen
//! parecidas a las del origen.
//!
//! Las pruebas se ejecutan en depuración: si alguna aritmética se
//! desbordara, sería un `panic!` antes de cualquier `assert`.

use grisu_voxel::{ChunkPos, SparseWorld, Voxels};
use hush_worldgen::{
    AMPLITUDE, DIAMOND, DIRT, GOLD, HARD_ROCK, ORE, ROCK, generate_chunk, load_around,
    surface_height,
};

const SEED: u64 = 42;
/// Borde del mundo en bloques (15 M de lado, ±7,5 M).
const FAR: i32 = 7_499_999;

#[test]
fn no_overflow_at_extreme_and_negative_blocks() {
    let corners = [
        (FAR, FAR),
        (-FAR, -FAR),
        (FAR, -FAR),
        (-FAR, FAR),
        (FAR - 15, -FAR + 15),
        (-1, -1),
    ];
    for &(bx, bz) in &corners {
        // La altura sigue en rango y el chunk se genera sin pánico. En el
        // W1 la superficie va de −64 a +AMPLITUDE, no de −6 a 6.
        let h = surface_height(SEED, bx, bz);
        assert!(
            (-AMPLITUDE..=AMPLITUDE).contains(&h),
            "altura {h} en ({bx}, {bz})"
        );

        let chunk = generate_chunk(SEED, ChunkPos::containing([bx, -1, bz]));
        // Debajo de la superficie hay sólido; un chunk más arriba, aire.
        assert!(!chunk.is_empty(), "chunk vacío bajo ({bx}, {bz})");
        // Cielo puro: por encima de `surface + 8` (la banda del frente)
        // jamás hay sólido, y con montañas hasta +AMPLITUDE el chunk de
        // y = 16 ya no es cielo; AMPLITUDE + 40 está siempre sobre la cota.
        let sky = generate_chunk(SEED, ChunkPos::containing([bx, AMPLITUDE + 40, bz]));
        assert!(sky.is_empty(), "bloques en el cielo de ({bx}, {bz})");
    }
}

/// Los extremos de `i32` también son seguros en la función de relieve (el
/// generador del mundo solo se usa dentro de ±7,5 M, pero el desplazamiento
/// y la interpolación no deben desbordarse antes).
#[test]
fn surface_height_survives_i32_extremes() {
    for (x, z) in [
        (i32::MIN, i32::MIN),
        (i32::MAX, i32::MAX),
        (i32::MIN, i32::MAX),
    ] {
        // `x + 1` desbordaría: se consultan tal cual, sin aritmética previa.
        // La cota del W1 es ±AMPLITUDE, no ±6.
        let h = surface_height(SEED, x, z);
        assert!((-AMPLITUDE..=AMPLITUDE).contains(&h));
    }
}

/// Un mundo cargado en la esquina del mapa funciona igual que en el origen.
#[test]
fn load_around_works_at_the_world_corner() {
    let mut world = SparseWorld::new();
    // Esquina del mundo en X/Z; en Y la ventana sigue a la superficie real
    // de esa columna (en el W1 esta esquina queda a h = 172 y cargar en
    // `cy = 0` dejaría el terreno fuera de la ventana).
    let (x, z) = (-FAR, FAR);
    let h = surface_height(SEED, x, z);
    let corner = ChunkPos::containing([x, h, z]);
    assert_eq!(load_around(&mut world, SEED, corner, 1, 1), 27);
    // Sondas relativas a la superficie (la garantía del generador es
    // sólido por debajo de `h − 8` y aire por encima de `h + 8`); ±16 cae
    // siempre dentro de la ventana cargada de ±1 chunk vertical, sea cual
    // sea `h`, de modo que ambas consultas leen chunks cargados de verdad.
    assert!(
        world.is_solid(x, h - 16, z),
        "sólido esperado en h − 16 (h = {h})"
    );
    assert!(
        !world.is_solid(x, h + 16, z),
        "aire esperado en h + 16 (h = {h})"
    );
    // Y coincide con la generación individual (otra vez `INV-W01` lejos).
    assert_eq!(world.chunk(corner), Some(&generate_chunk(SEED, corner)));
}

/// Proporciones de cada bloque sobre varias ventanas de chunks: dos capas
/// ancladas a la superficie de la columna central de la ventana (la que
/// contiene `h − 64` y la siguiente), donde conviven roca, mena y oro. El
/// anclaje relativo es lo que hace comparable el origen con las esquinas:
/// en el W1 la superficie lejana puede estar a 180 bloques y muestrear
/// capas fijas `cy = −2..0` compararía aire con roca.
fn proportions(centers: &[ChunkPos], side: i32) -> Vec<(u8, f64)> {
    let mut counts = std::collections::BTreeMap::new();
    let mut total = 0usize;
    for &center in centers {
        let h = surface_height(SEED, center.x * 16, center.z * 16);
        let base = ChunkPos::containing([center.x * 16, h - 64, center.z * 16]).y;
        for cy in base..base + 2 {
            for dz in 0..side {
                for dx in 0..side {
                    let pos = ChunkPos::new(center.x + dx, cy, center.z + dz);
                    for &b in generate_chunk(SEED, pos).as_bytes() {
                        if b != grisu_voxel::AIR {
                            *counts.entry(b).or_insert(0usize) += 1;
                            total += 1;
                        }
                    }
                }
            }
        }
    }
    assert!(total > 400_000, "muestra pequeña: {total}");
    [DIRT, ROCK, HARD_ROCK, ORE, GOLD, DIAMOND]
        .into_iter()
        .map(|b| {
            let n = counts.get(&b).copied().unwrap_or(0);
            (b, n as f64 / total as f64)
        })
        .collect()
}

/// La mezcla del hash no se degrada lejos del origen: en ±7 499 999, con
/// signos mezclados, la proporción de cada bloque sigue parecida a la del
/// origen. Márgenes holgados: ±3 puntos absolutos en las capas comunes
/// (≥ 5 %) y ±60 % relativo en las menas, que ya de por sí son raras.
#[test]
fn block_proportions_hold_far_from_the_origin() {
    let origin = proportions(&[ChunkPos::new(0, -1, 0)], 8);
    // Las cuatro esquinas del mundo (ventanas de 4 chunks que caben justo en
    // el borde), agrupadas en una sola muestra para que también el diamante,
    // el más escaso, tenga estadística suficiente.
    let edge = FAR / 16;
    let far_corners = [
        ChunkPos::new(edge - 3, -1, edge - 3),
        ChunkPos::new(-edge, -1, -edge),
        ChunkPos::new(edge - 3, -1, -edge),
        ChunkPos::new(-edge, -1, edge - 3),
    ];
    let remote = proportions(&far_corners, 4);
    for (&(block, here), &(_, there)) in origin.iter().zip(&remote) {
        let label = match block {
            DIRT => "tierra",
            ROCK => "roca",
            HARD_ROCK => "roca dura",
            ORE => "mena",
            GOLD => "oro",
            _ => "diamante",
        };
        let tolerance = if here >= 0.05 { 0.03 } else { 0.60 * here };
        assert!(
            (here - there).abs() <= tolerance,
            "{label}: {there:.5} lejos del origen frente a {here:.5} en el origen"
        );
    }
}
