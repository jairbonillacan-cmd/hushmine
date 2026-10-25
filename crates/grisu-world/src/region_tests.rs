//! Pruebas de `region`: coordenadas, ida y vuelta por disco, atomicidad y
//! corrupción de bytes. Será declarado desde `lib.rs` con
//! `#[cfg(test)] mod region_tests;`.

use std::path::PathBuf;

use crate::region::{REGION_CHUNKS, RegionError, RegionFile, RegionPos};
use grisu_voxel::{Chunk, ChunkPos};

/// Directorio temporal único por prueba (misma base por proceso).
fn dir_temporal(sufijo: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("grisu-world-test-{}", std::process::id()))
        .join(sufijo);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("crear el directorio temporal");
    dir
}

/// Chunk con muchos ids distintos (paleta grande, tramos cortos).
fn chunk_variado() -> Chunk {
    let mut chunk = Chunk::default();
    for i in 0..4096usize {
        let x = i % 16;
        let y = i / 256;
        let z = (i % 256) / 16;
        chunk.set(x, y, z, ((i * 13) % 200) as u8);
    }
    chunk
}

/// Chunk con dos ids en tramos largos: roca (4) abajo, aire arriba.
fn chunk_tramos_largos() -> Chunk {
    let mut chunk = Chunk::filled(4);
    for y in 8..16 {
        for z in 0..16 {
            for x in 0..16 {
                chunk.set(x, y, z, 0);
            }
        }
    }
    chunk
}

#[test]
fn coordenadas_de_region_con_negativos() {
    let pos = RegionPos::from_chunk(ChunkPos::new(-1, -1, -1));
    assert_eq!(
        pos,
        RegionPos {
            rx: -1,
            ry: -1,
            rz: -1
        }
    );
    assert_eq!(RegionPos::local_index(ChunkPos::new(-1, -1, -1)), 32767);

    let pos = RegionPos::from_chunk(ChunkPos::new(31, 32, -33));
    assert_eq!(
        pos,
        RegionPos {
            rx: 0,
            ry: 1,
            rz: -2
        }
    );

    for p in [
        ChunkPos::new(-33, 1, 100),
        ChunkPos::new(0, 0, 0),
        ChunkPos::new(-16, -17, 31),
        ChunkPos::new(-1, -1, -1),
    ] {
        let region = RegionPos::from_chunk(p);
        let local = RegionPos::local_index(p);
        assert!(local < REGION_CHUNKS as u16);
        assert_eq!(region.chunk_pos(local), Some(p), "chunk {p:?}");
    }
    assert_eq!(RegionPos::default().chunk_pos(REGION_CHUNKS as u16), None);

    assert_eq!(
        RegionPos {
            rx: -3,
            ry: 5,
            rz: -7
        }
        .file_name(),
        "r.-3.5.-7.hreg"
    );
}

#[test]
fn ida_y_vuelta_en_disco() {
    let dir = dir_temporal("ida-y-vuelta");
    let region_pos = RegionPos {
        rx: -3,
        ry: 5,
        rz: -7,
    };
    let mut region = RegionFile::new(region_pos, 7);
    let uniforme = Chunk::filled(9);
    let variado = chunk_variado();
    let tramos = chunk_tramos_largos();
    region
        .mark_modified(0, uniforme.clone())
        .expect("marcar el uniforme");
    region
        .mark_modified(1234, variado.clone())
        .expect("marcar el variado");
    region
        .mark_modified(32767, tramos.clone())
        .expect("marcar los tramos");
    region.save_atomic(&dir).expect("guardar la región");

    let cargada = RegionFile::load(&dir.join("r.-3.5.-7.hreg")).expect("cargar la región");
    assert_eq!(cargada.region_pos(), region_pos);
    assert_eq!(cargada.worldgen_version(), 7);
    assert_eq!(cargada.modified_indices(), vec![0, 1234, 32767]);
    assert_eq!(cargada.chunk(0), Some(&uniforme));
    assert_eq!(cargada.chunk(1234), Some(&variado));
    assert_eq!(cargada.chunk(32767), Some(&tramos));
    assert_eq!(cargada.chunk(1), None);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn solo_guarda_los_modificados() {
    let dir = dir_temporal("solo-modificados");
    let path = dir.join("r.0.0.0.hreg");
    let mut region = RegionFile::new(RegionPos::default(), 1);

    region.save_atomic(&dir).expect("guardar la región vacía");
    let cargada = RegionFile::load(&path).expect("cargar la región vacía");
    assert!(cargada.modified_indices().is_empty());
    let bytes = std::fs::read(&path).expect("leer el archivo vacío");
    assert_eq!(bytes.len(), 24); // solo cabecera: magia + pos + versión + total

    region
        .mark_modified(5, Chunk::filled(1))
        .expect("marcar el primero");
    region
        .mark_modified(7, Chunk::filled(2))
        .expect("marcar el segundo");
    region.save_atomic(&dir).expect("guardar de nuevo");
    let cargada = RegionFile::load(&path).expect("cargar la región con dos");
    assert_eq!(cargada.modified_indices(), vec![5, 7]);
    assert_eq!(cargada.chunk(5), Some(&Chunk::filled(1)));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn bytes_corruptos_son_error_no_panico() {
    let dir = dir_temporal("bytes-corruptos");
    let mut region = RegionFile::new(
        RegionPos {
            rx: 1,
            ry: 2,
            rz: 3,
        },
        1,
    );
    region
        .mark_modified(0, Chunk::filled(7))
        .expect("marcar el uniforme");
    region
        .mark_modified(1, chunk_variado())
        .expect("marcar el variado");
    region.save_atomic(&dir).expect("guardar la región");
    let validos = std::fs::read(dir.join("r.1.2.3.hreg")).expect("leer los bytes");

    // Truncados en varias longitudes, incluido el último byte del blob.
    for corte in [0usize, 3, 20, validos.len() - 1] {
        let path = dir.join(format!("truncado-{corte}.hreg"));
        std::fs::write(&path, &validos[..corte]).expect("escribir el truncado");
        let resultado = RegionFile::load(&path);
        assert!(resultado.is_err(), "un corte en {corte} debe dar error");
    }

    // Magia cambiada.
    let mut mala_magia = validos.clone();
    mala_magia[0..4].copy_from_slice(b"XREG");
    let path = dir.join("mala-magia.hreg");
    std::fs::write(&path, mala_magia).expect("escribir la mala magia");
    assert!(matches!(
        RegionFile::load(&path),
        Err(RegionError::Corrupt(_))
    ));

    // chunk_count imposible (u32::MAX en los bytes 20..24 de la cabecera).
    let mut mal_total = validos.clone();
    mal_total[20..24].copy_from_slice(&u32::MAX.to_le_bytes());
    let path = dir.join("mal-total.hreg");
    std::fs::write(&path, mal_total).expect("escribir el mal total");
    assert!(matches!(
        RegionFile::load(&path),
        Err(RegionError::Corrupt(_))
    ));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn escritura_atomica_no_deja_tmp() {
    let dir = dir_temporal("escritura-atomica");
    let mut region = RegionFile::new(
        RegionPos {
            rx: 2,
            ry: -2,
            rz: 2,
        },
        3,
    );
    region
        .mark_modified(10, Chunk::filled(5))
        .expect("marcar el chunk");
    region.save_atomic(&dir).expect("primer guardado");
    region
        .save_atomic(&dir)
        .expect("segundo guardado sobre el mismo archivo");

    assert!(dir.join("r.2.-2.2.hreg").exists());
    let temporales: Vec<String> = std::fs::read_dir(&dir)
        .expect("leer el directorio")
        .map(|entrada| {
            entrada
                .expect("entrada de directorio")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|nombre| nombre.ends_with(".tmp"))
        .collect();
    assert!(
        temporales.is_empty(),
        "no debe quedar ningún .tmp: {temporales:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn mark_modified_rechaza_indice_fuera_de_rango() {
    let mut region = RegionFile::new(RegionPos::default(), 1);
    let resultado = region.mark_modified(REGION_CHUNKS as u16, Chunk::filled(1));
    assert!(matches!(resultado, Err(RegionError::Corrupt(_))));
    assert!(region.modified_indices().is_empty());

    // El último índice válido sí entra.
    region
        .mark_modified(REGION_CHUNKS as u16 - 1, Chunk::filled(1))
        .expect("el índice 32767 es válido");
    assert_eq!(region.modified_indices(), vec![REGION_CHUNKS as u16 - 1]);
}
