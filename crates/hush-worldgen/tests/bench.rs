//! Medición de rendimiento (IGNORADA en las ejecuciones normales): tiempo
//! por chunk en release con el generador W1 (columna + densidad +
//! litología).
//!
//! ```bash
//! cargo test -p hush-worldgen --test bench --release -- --ignored --nocapture
//! ```
//!
//! Objetivo del hito W1: **p50 ≤ 1 ms y p95 ≤ 3 ms por chunk**
//! (docs/mundo/01-escala-y-motor.md §10, «Generación de un chunk»). Las
//! posiciones cubren todas las bandas del mundo: lecho y Raíces
//! (`chunk_y` −300/−200), Bóveda profunda e islas (−100), manto (−30),
//! frente de superficie (−2/−1/0), montañas (1) y cielo bajo la cota (40).
//! El `f64` vive solo en esta medición, nunca en el generador.

use std::hint::black_box;
use std::time::Instant;

use grisu_voxel::ChunkPos;
use hush_worldgen::generate_chunk;

const CHUNKS: usize = 256;
/// Objetivos, en microsegundos por chunk (p50 y p95).
const TARGET_P50_US: f64 = 1000.0;
const TARGET_P95_US: f64 = 3000.0;
/// Las bandas verticales que debe tocar la muestra (lecho/Raíces, Bóveda,
/// islas, manto, superficie, montañas y cielo bajo la cota).
const Y_BANDS: [i32; 9] = [-300, -200, -100, -30, -2, -1, 0, 1, 40];
/// Semiancho del mundo en chunks (±468 740 ≈ ±7,5 M bloques).
const FAR: u64 = 468_740;

#[test]
#[ignore = "medición de rendimiento: ejecútala con --release y --ignored"]
fn chunk_generation_meets_p50_and_p95_targets() {
    // Posiciones variadas y deterministas: X/Z del xorshift por todo el
    // mundo, ciclando `chunk_y` por las bandas verticales.
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
                (next() % (2 * FAR + 1)) as i32 - FAR as i32,
                Y_BANDS[i % Y_BANDS.len()],
                (next() % (2 * FAR + 1)) as i32 - FAR as i32,
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
        "generación W1: p50 = {p50:.1} µs/chunk, p95 = {p95:.1} µs/chunk \
         (objetivo p50 ≤ {TARGET_P50_US:.0} µs, p95 ≤ {TARGET_P95_US:.0} µs)"
    );

    // Los objetivos solo se exigen en release: en depuración el generador
    // es decenas de veces más lento y no dice nada del juego.
    if cfg!(not(debug_assertions)) {
        assert!(
            p50 <= TARGET_P50_US,
            "p50 = {p50:.1} µs/chunk supera el objetivo de {TARGET_P50_US:.0} µs"
        );
        assert!(
            p95 <= TARGET_P95_US,
            "p95 = {p95:.1} µs/chunk supera el objetivo de {TARGET_P95_US:.0} µs"
        );
    }
}
