//! `INV-W01`: determinismo — misma semilla y posición dan el mismo chunk
//! byte a byte, y semilla o posición distintas dan otro chunk.

use grisu_voxel::ChunkPos;
use hush_worldgen::generate_chunk;

#[test]
fn same_seed_and_position_give_the_same_bytes() {
    let positions = [
        ChunkPos::new(0, 0, 0),
        ChunkPos::new(-1, 0, 0),
        ChunkPos::new(3, -2, 5),
        ChunkPos::new(29, 7, -11),
        ChunkPos::new(-468_750, -300, 468_749),
        ChunkPos::new(468_749, -256, -468_750),
    ];
    for pos in positions {
        for seed in [0, 1, 42, u64::MAX] {
            let a = generate_chunk(seed, pos);
            let b = generate_chunk(seed, pos);
            assert_eq!(a.as_bytes(), b.as_bytes(), "semilla {seed}, chunk {pos:?}");
            // La codificación para red también es idéntica.
            assert_eq!(a.encode(), b.encode());
        }
    }
}

/// Dos generaciones del mismo chunk en «procesos» distintos (aquí, dos
/// llamadas sin nada compartido entre medias) coinciden incluso si entre
/// medias se generó otra cosa: no hay estado que contamine.
#[test]
fn interleaved_generation_does_not_carry_state() {
    let target = ChunkPos::new(-3, -1, 2);
    let clean = generate_chunk(42, target);
    // Generar de todo por medias, incluidos chunks vecinos y otra semilla.
    for dy in -1..=1 {
        for dz in -1..=1 {
            for dx in -1..=1 {
                let pos = target.offset(dx, dy, dz);
                let _ = generate_chunk(7, pos);
                let _ = generate_chunk(42, pos);
            }
        }
    }
    assert_eq!(generate_chunk(42, target).as_bytes(), clean.as_bytes());
}

#[test]
fn seed_and_position_change_the_chunk() {
    let base = generate_chunk(42, ChunkPos::new(0, -1, 0));
    let other_seed = generate_chunk(43, ChunkPos::new(0, -1, 0));
    let other_pos = generate_chunk(42, ChunkPos::new(1, -1, 0));
    assert_ne!(base.as_bytes(), other_seed.as_bytes());
    assert_ne!(base.as_bytes(), other_pos.as_bytes());
    // Chunks simétricos respecto al origen también difieren: el hash mezcla
    // el signo de las coordenadas.
    let mirrored = generate_chunk(42, ChunkPos::new(-1, -1, 0));
    assert_ne!(base.as_bytes(), mirrored.as_bytes());
}

/// El terreno solo contiene bloques de la Convención A que el generador
/// sabe poner: aire, tierra, roca, roca dura, mena, oro y diamante.
#[test]
fn only_expected_block_ids_appear() {
    let allowed = [
        grisu_voxel::AIR,
        hush_worldgen::DIRT,
        hush_worldgen::ROCK,
        hush_worldgen::HARD_ROCK,
        hush_worldgen::ORE,
        hush_worldgen::GOLD,
        hush_worldgen::DIAMOND,
    ];
    for pos in [
        ChunkPos::new(0, 0, 0),
        ChunkPos::new(-5, -2, 9),
        ChunkPos::new(31, -7, -12),
    ] {
        for &b in generate_chunk(42, pos).as_bytes() {
            assert!(allowed.contains(&b), "bloque inesperado {b} en {pos:?}");
        }
    }
}
