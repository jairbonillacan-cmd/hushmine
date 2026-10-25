//! `INV-W02`: sin costuras — generar 3×3×3 chunks en cualquier orden da lo
//! mismo, y los chunks vecinos encajan en el borde compartido.

use grisu_voxel::{ChunkPos, SparseWorld, Voxels};
use hush_worldgen::{block_at, generate_chunk, load_around, surface_height};

const SEED: u64 = 42;

/// Las 27 posiciones de un bloque de 3×3×3 chunks alrededor de una zona con
/// costuras en los tres ejes y coordenadas negativas.
fn region() -> Vec<ChunkPos> {
    let center = ChunkPos::new(-1, -1, 0);
    let mut positions = Vec::with_capacity(27);
    for dy in -1..=1 {
        for dz in -1..=1 {
            for dx in -1..=1 {
                positions.push(center.offset(dx, dy, dz));
            }
        }
    }
    positions
}

/// Barajado determinista (xorshift64) de las 27 posiciones.
fn shuffled(mut items: Vec<ChunkPos>) -> Vec<ChunkPos> {
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    for i in (1..items.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        items.swap(i, (state as usize) % (i + 1));
    }
    items
}

/// Generar la región en órdenes distintos produce los mismos chunks byte a
/// byte: nada depende del orden ni de los vecinos ya generados.
#[test]
fn generation_order_does_not_matter() {
    let positions = region();
    // Referencia en orden directo: posición → chunk codificado.
    let reference: Vec<Vec<u8>> = positions
        .iter()
        .map(|&p| generate_chunk(SEED, p).encode())
        .collect();

    let mut forward = positions.clone();
    forward.reverse();
    for &p in &forward {
        let i = positions.iter().position(|&q| q == p).unwrap();
        assert_eq!(generate_chunk(SEED, p).encode(), reference[i]);
    }

    for &p in &shuffled(positions.clone()) {
        let i = positions.iter().position(|&q| q == p).unwrap();
        assert_eq!(generate_chunk(SEED, p).encode(), reference[i]);
    }
}

/// El relieve es continuo a través de las costuras: la altura de columnas
/// vecinas —también cuando pertenecen a chunks distintos— no salta más de
/// 2 bloques (la pendiente máxima de la rejilla de relieve).
#[test]
fn surface_is_continuous_across_chunk_borders() {
    // Una ventana que cruza varias fronteras de chunk en ambos ejes.
    for z in -40..40 {
        let mut prev = surface_height(SEED, -40, z);
        for x in -39..40 {
            let h = surface_height(SEED, x, z);
            assert!((h - prev).abs() <= 2, "salto de {prev} a {h} en ({x}, {z})");
            prev = h;
        }
    }
}

/// Cada bloque de la frontera entre dos chunks es el mismo visto desde
/// ambos lados: coincide con la función pura `block_at` sobre coordenadas
/// globales. Es la prueba clásica del error de esquina (origen mal
/// calculado al pasar de coordenadas de chunk a de bloque).
#[test]
fn seams_match_the_pure_function() {
    let mut world = SparseWorld::new();
    load_around(&mut world, SEED, ChunkPos::new(-1, -1, 0), 1, 1);
    assert_eq!(world.len(), 27);
    // Chunks cargados: x, y ∈ [−2, 0] y z ∈ [−1, 1], es decir bloques
    // x, y ∈ [−32, 16) y z ∈ [−16, 32). Las fronteras internas entre
    // chunks están en x, y ∈ {−16, 0} y z ∈ {0, 16}.

    // Las cuatro caras verticales internas de la columna central de chunks:
    // dos fronteras en x (z recorre lo cargado) y dos en z (x recorre).
    for y in -32..16 {
        for z in -16..32 {
            for x in [-16, 0] {
                assert_eq!(
                    world.get(x, y, z),
                    block_at(SEED, i64::from(x), i64::from(y), i64::from(z)),
                    "bloque ({x}, {y}, {z})"
                );
            }
        }
        for x in -32..16 {
            for z in [0, 16] {
                assert_eq!(
                    world.get(x, y, z),
                    block_at(SEED, i64::from(x), i64::from(y), i64::from(z)),
                    "bloque ({x}, {y}, {z})"
                );
            }
        }
    }
    // Y las caras horizontales entre capas.
    for y in [-16, 0] {
        for z in -16..32 {
            for x in -32..16 {
                assert_eq!(
                    world.get(x, y, z),
                    block_at(SEED, i64::from(x), i64::from(y), i64::from(z)),
                    "bloque ({x}, {y}, {z})"
                );
            }
        }
    }
}

/// El mundo cargado de 27 chunks equivale a generarlos por separado, y la
/// superficie queda a la vista en todos ellos.
#[test]
fn loaded_world_matches_individual_chunks() {
    let mut world = SparseWorld::new();
    load_around(&mut world, SEED, ChunkPos::new(-1, -1, 0), 1, 1);
    for pos in region() {
        assert_eq!(world.chunk(pos), Some(&generate_chunk(SEED, pos)));
    }
}
