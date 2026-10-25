//! `hush-cli bench`: tiempos de generación del terreno y de sus mallas.
//!
//! Condición de salida del hito 1: regenerar todas las mallas de un mapa de
//! 64³ (con su luz) en menos de 50 ms. Hay que medir con `--release`.

use std::path::Path;
use std::time::Instant;

use anyhow::{Result, ensure};
use grisu_voxel::{AIR, LightMap, mesh_chunk};
use hush_sim::{TerrainConfig, generate_terrain};

/// Presupuesto para regenerar todas las mallas de 64³, en milisegundos.
const FULL_MESH_BUDGET_MS: f64 = 50.0;

pub fn run(seed: u64, runs: u32, out: Option<&Path>) -> Result<()> {
    let runs = runs.max(1);
    let config = TerrainConfig::default();

    let generate = median(runs, || Ok(generate_terrain(seed, &config).map(drop)?))?;
    let mut world = generate_terrain(seed, &config)?;

    let mut faces = 0;
    let full_mesh = median(runs, || {
        faces = (0..world.chunk_count())
            .map(|i| mesh_chunk(&world, None, world.chunk_pos(i)).len())
            .sum();
        Ok(())
    })?;

    let emission = hush_art::emission_table();
    let light_full = median(runs, || {
        std::hint::black_box(LightMap::new(&world, emission));
        Ok(())
    })?;
    let light = LightMap::new(&world, emission);
    let lit_mesh = median(runs, || {
        for i in 0..world.chunk_count() {
            std::hint::black_box(mesh_chunk(&world, Some(&light), world.chunk_pos(i)));
        }
        Ok(())
    })?;

    // Edición típica: cavar un bloque en una esquina de chunk (toca hasta 8
    // chunks) y regenerar solo los sucios.
    world.take_dirty();
    let mut remeshed = 0;
    let mut y = 40;
    let edit = median(runs, || {
        world.set(16, y, 16, AIR);
        y -= 1;
        let dirty = world.take_dirty();
        remeshed = dirty.len();
        for i in dirty {
            std::hint::black_box(mesh_chunk(&world, None, world.chunk_pos(i)));
        }
        Ok(())
    })?;

    let report = serde_json::json!({
        "seed": seed,
        "map": config.size,
        "profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        "terrain_generate_ms": generate,
        "full_mesh_ms": full_mesh,
        "full_mesh_budget_ms": FULL_MESH_BUDGET_MS,
        "faces": faces,
        "light_full_ms": light_full,
        "lit_full_mesh_ms": lit_mesh,
        "edit_remesh_ms": edit,
        "edit_chunks_remeshed": remeshed,
    });
    let json = serde_json::to_string_pretty(&report)?;
    match out {
        Some(path) => {
            crate::create_parent_dir(path)?;
            std::fs::write(path, &json)?;
            println!("informe guardado en {}", path.display());
        }
        None => println!("{json}"),
    }
    if cfg!(debug_assertions) {
        println!("aviso: perfil de depuración; usa --release para medir el presupuesto");
    } else {
        let total = light_full + lit_mesh;
        ensure!(
            total <= FULL_MESH_BUDGET_MS,
            "luz y mallas de 64³ en {total:.1} ms, por encima de {FULL_MESH_BUDGET_MS} ms"
        );
    }
    Ok(())
}

/// Mediana, en milisegundos, de `runs` ejecuciones de `f`.
fn median(runs: u32, mut f: impl FnMut() -> Result<()>) -> Result<f64> {
    let mut times = Vec::with_capacity(runs as usize);
    for _ in 0..runs {
        let start = Instant::now();
        f()?;
        times.push(start.elapsed().as_secs_f64() * 1e3);
    }
    times.sort_by(f64::total_cmp);
    Ok(times[times.len() / 2])
}
