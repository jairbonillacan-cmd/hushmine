//! `hush-cli world bench`: tiempos de generación de chunks del mundo grande.
//!
//! Mide cuánto tarda [`hush_worldgen::generate_chunk`] en una muestra de
//! posiciones deterministas que cubre la zona cercana al origen y la esquina
//! del mundo en coordenadas de chunk (±468_740), con chunks verticales de
//! -4 a 3 (la superficie del mundo de prueba queda cerca de `y = 0`).
//! Presupuesto del hito W1: p95 por debajo de 3 000 µs, medido en release.

use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result, ensure};
use grisu_voxel::ChunkPos;
use hush_worldgen::generate_chunk;
use serde_json::json;

/// Esquina del mundo en coordenadas de chunk (donde más se aleja el hash).
const WORLD_CORNER: i64 = 468_740;
/// Presupuesto por chunk del hito W1, en microsegundos (solo release).
const CHUNK_BUDGET_US: u64 = 3_000;

/// Posiciones de chunk que mide `world bench`, deterministas (solo
/// aritmética entera, sin azar del sistema).
///
/// El primer cuarto de los índices recorre la zona cercana al origen
/// (±16 chunks); el resto hace un barrido lejano hasta ±468_740, con
/// matemática de `i64` para no desbordar. El índice 0 siempre es el origen.
pub fn chunk_grid(chunks: usize) -> Vec<(i32, i32, i32)> {
    let n = chunks.max(1);
    let near = n / 4;
    let far = (n - near).max(1);
    (0..n)
        .map(|i| {
            if i == 0 {
                return (0, 0, 0);
            }
            let cy = (i % 8) as i32 - 4;
            let (cx, cz) = if i < near {
                (((i % 33) as i32) - 16, (((i / 33) % 33) as i32) - 16)
            } else {
                let j = (i - near) as i64;
                let m = far as i64;
                let cx = -WORLD_CORNER + (2 * WORLD_CORNER * j) / m;
                let cz = WORLD_CORNER - (2 * WORLD_CORNER * ((3 * j + 1) % m)) / m;
                (cx as i32, cz as i32)
            };
            (cx, cy, cz)
        })
        .collect()
}

/// Ejecuta el subcomando `world bench`.
pub fn run(seed: u64, chunks: usize, out: Option<&Path>) -> Result<()> {
    ensure!(chunks >= 1, "la medición necesita al menos 1 chunk");

    let grid = chunk_grid(chunks);
    let mut micros = Vec::with_capacity(grid.len());
    for &(cx, cy, cz) in &grid {
        let start = Instant::now();
        let chunk = generate_chunk(seed, ChunkPos::new(cx, cy, cz));
        std::hint::black_box(&chunk);
        micros.push(start.elapsed().as_micros() as u64);
    }

    let mut sorted = micros.clone();
    sorted.sort_unstable();
    let n = sorted.len();
    let total: u64 = micros.iter().sum();
    let mean_us = total / n as u64;
    let p50_us = percentile(&sorted, 50);
    let p95_us = percentile(&sorted, 95);
    let p99_us = percentile(&sorted, 99);
    let max_us = sorted[n - 1];

    let report = json!({
        "seed": seed,
        "chunks": n,
        "profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        "mean_us": mean_us,
        "p50_us": p50_us,
        "p95_us": p95_us,
        "p99_us": p99_us,
        "max_us": max_us,
    });
    println!("{}", serde_json::to_string_pretty(&report)?);

    if let Some(path) = out {
        crate::create_parent_dir(path)?;
        let mut csv = String::from("chunk_index,cx,cy,cz,micros\n");
        for (index, (&(cx, cy, cz), &us)) in grid.iter().zip(&micros).enumerate() {
            csv.push_str(&format!("{index},{cx},{cy},{cz},{us}\n"));
        }
        std::fs::write(path, csv)
            .with_context(|| format!("no se pudo escribir {}", path.display()))?;
        println!("detalle por chunk guardado en {}", path.display());
    }

    if cfg!(debug_assertions) {
        println!("aviso: perfil de depuración; usa --release para medir el presupuesto");
    } else {
        ensure!(
            p95_us <= CHUNK_BUDGET_US,
            "p95 de {p95_us} µs por chunk supera el presupuesto de 3 000 µs"
        );
    }
    Ok(())
}

/// Percentil de rango más cercano sobre una lista ya ordenada: índice
/// `ceil(p·n/100) − 1`, acotado al rango de la lista.
fn percentile(sorted: &[u64], p: u64) -> u64 {
    let n = sorted.len();
    if n == 0 {
        return 0;
    }
    let rank = (p as usize * n).div_ceil(100).saturating_sub(1).min(n - 1);
    sorted[rank]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_covers_origin_near_and_far_zones() {
        let grid = chunk_grid(64);
        assert_eq!(grid.len(), 64);
        assert!(grid.contains(&(0, 0, 0)));
        let cerca = grid
            .iter()
            .any(|&(cx, _, cz)| cx.abs() <= 16 && cz.abs() <= 16);
        assert!(cerca, "falta alguna posición cercana al origen");
        let lejos_x = grid.iter().any(|&(cx, _, _)| cx.abs() >= 400_000);
        assert!(lejos_x, "falta alguna posición con |cx| lejano");
        let lejos_z = grid.iter().any(|&(_, _, cz)| cz.abs() >= 400_000);
        assert!(lejos_z, "falta alguna posición con |cz| lejano");
        assert!(grid.iter().all(|&(_, cy, _)| (-4..=3).contains(&cy)));
    }

    #[test]
    fn grid_is_deterministic() {
        assert_eq!(chunk_grid(32), chunk_grid(32));
    }

    #[test]
    fn single_chunk_grid_is_the_origin() {
        assert_eq!(chunk_grid(1), vec![(0, 0, 0)]);
    }

    #[test]
    fn percentile_uses_nearest_rank() {
        let sorted = [10, 20, 30, 40];
        assert_eq!(percentile(&sorted, 50), 20);
        assert_eq!(percentile(&sorted, 95), 40);
        assert_eq!(percentile(&sorted, 99), 40);
        assert_eq!(percentile(&sorted, 100), 40);
    }

    #[test]
    fn run_measures_a_minimal_sample() {
        assert!(run(42, 2, None).is_ok());
    }

    #[test]
    fn run_with_path_writes_the_csv() {
        let dir = std::env::temp_dir().join("hush_world_bench_test");
        let path = dir.join("bench.csv");
        run(42, 3, Some(&path)).unwrap();
        let csv = std::fs::read_to_string(&path).unwrap();
        let lineas: Vec<&str> = csv.lines().collect();
        assert_eq!(lineas[0], "chunk_index,cx,cy,cz,micros");
        assert_eq!(lineas.len(), 4);
        assert!(lineas[1].starts_with("0,0,0,0,"));
        let _ = std::fs::remove_file(&path);
    }
}
