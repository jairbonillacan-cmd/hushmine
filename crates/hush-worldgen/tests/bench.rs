//! Medición de rendimiento (IGNORADA en las ejecuciones normales): tiempo
//! por chunk en release.
//!
//! ```bash
//! cargo nextest run --release -p hush-worldgen --run-ignored only -- bench
//! # o, para ver la medida:
//! cargo test --release -p hush-worldgen -- --ignored --nocapture bench
//! ```
//!
//! Objetivo del hito W0: **p50 ≤ 1 ms por chunk** con el generador de
//! prueba (docs/mundo/04-verificacion-e-hitos.md, riesgo «la generación no
//! cumple 1 ms por chunk»). El `f64` vive solo en esta medición, nunca en
//! el generador.

use std::hint::black_box;
use std::time::Instant;

use grisu_voxel::ChunkPos;
use hush_worldgen::generate_chunk;

const CHUNKS: usize = 256;
/// El objetivo, en microsegundos por chunk.
const TARGET_US: f64 = 1000.0;

#[test]
#[ignore = "medición de rendimiento: ejecútala con --release y --run-ignored only"]
fn chunk_generation_p50_is_under_1ms() {
    // Posiciones variadas y deterministas, con chunks de cielo, superficie
    // y profundidad.
    let mut state = 0x1234_5678_9ABC_DEF0u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let positions: Vec<ChunkPos> = (0..CHUNKS)
        .map(|i| {
            ChunkPos::new(
                (next() % 4096) as i32 - 2048,
                i as i32 % 5 - 2,
                (next() % 4096) as i32 - 2048,
            )
        })
        .collect();

    // Calentamiento (no se mide).
    for &pos in &positions {
        black_box(generate_chunk(42, pos));
    }

    let mut nanos: Vec<u64> = Vec::with_capacity(CHUNKS);
    for &pos in &positions {
        let start = Instant::now();
        black_box(generate_chunk(42, pos));
        nanos.push(start.elapsed().as_nanos() as u64);
    }
    nanos.sort_unstable();
    let at = |q: usize| nanos[q * (CHUNKS - 1) / 100] as f64 / 1000.0;
    let (p50, p95) = (at(50), at(95));
    println!(
        "generación: p50 = {p50:.1} µs/chunk, p95 = {p95:.1} µs/chunk \
         (objetivo p50 ≤ {TARGET_US:.0} µs)"
    );

    // El objetivo solo se exige en release: en depuración el generador es
    // decenas de veces más lento y no dice nada del juego.
    if cfg!(not(debug_assertions)) {
        assert!(
            p50 <= TARGET_US,
            "p50 = {p50:.1} µs/chunk supera el objetivo de {TARGET_US:.0} µs"
        );
    }
}
