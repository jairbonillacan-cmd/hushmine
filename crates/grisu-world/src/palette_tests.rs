use crate::palette::*;
use grisu_voxel::{AIR, CHUNK_SIZE, CHUNK_VOLUME, Chunk};

/// Coordenadas (x, y, z) del índice lineal `i`, inverso de `Chunk::index`.
fn coords(i: usize) -> (usize, usize, usize) {
    (
        i % CHUNK_SIZE,
        i / (CHUNK_SIZE * CHUNK_SIZE),
        (i / CHUNK_SIZE) % CHUNK_SIZE,
    )
}

/// Coloca `id` en el índice lineal `i` del chunk.
fn place(pc: &mut PalettedChunk, i: usize, id: WideBlockId) {
    let (x, y, z) = coords(i);
    pc.set(x, y, z, id);
}

/// Exige que `pc` sea `Palette` y devuelve sus tres campos.
fn as_palette(pc: &PalettedChunk) -> (&u8, &Vec<WideBlockId>, &Vec<u64>) {
    match pc {
        PalettedChunk::Palette {
            bits,
            entries,
            packed,
        } => (bits, entries, packed),
        _ => panic!("se esperaba una variante Palette"),
    }
}

#[test]
fn filled_es_uniforme_y_get_devuelve_el_id_en_todas_partes() {
    let pc = PalettedChunk::filled(42);
    assert!(matches!(pc, PalettedChunk::Uniform(42)));
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                assert_eq!(pc.get(x, y, z), 42);
            }
        }
    }
}

#[test]
fn promociona_de_uniform_a_paleta_de_8_bits_y_direct() {
    let mut pc = PalettedChunk::filled(0);
    assert!(matches!(pc, PalettedChunk::Uniform(0)));

    // 2 distintos (el 0 de fondo y el 1): paleta de 1 bit.
    place(&mut pc, 0, 1);
    assert!(matches!(pc, PalettedChunk::Palette { bits: 1, .. }));

    // 3 distintos: 2 bits. 4 distintos: sigue en 2 bits.
    place(&mut pc, 1, 2);
    assert!(matches!(pc, PalettedChunk::Palette { bits: 2, .. }));
    place(&mut pc, 2, 3);
    assert!(matches!(pc, PalettedChunk::Palette { bits: 2, .. }));

    // 5 distintos: 4 bits. 16 distintos: sigue en 4 bits.
    place(&mut pc, 3, 4);
    assert!(matches!(pc, PalettedChunk::Palette { bits: 4, .. }));
    for k in 5..=15u16 {
        place(&mut pc, usize::from(k - 1), k);
    }
    assert!(matches!(pc, PalettedChunk::Palette { bits: 4, .. }));

    // 17 distintos: 8 bits. 256 distintos: sigue en 8 bits.
    place(&mut pc, 15, 16);
    assert!(matches!(pc, PalettedChunk::Palette { bits: 8, .. }));
    for k in 17..=255u16 {
        place(&mut pc, usize::from(k - 1), k);
    }
    assert!(matches!(pc, PalettedChunk::Palette { bits: 8, .. }));

    // El 257º distinto: representación directa.
    place(&mut pc, 255, 256);
    assert!(matches!(pc, PalettedChunk::Direct(_)));
}

#[test]
fn get_devuelve_exactamente_los_ids_colocados_tras_las_promociones() {
    let mut pc = PalettedChunk::filled(0);
    // 300 ids distintos recorren toda la cadena de promociones.
    for k in 1..=300u16 {
        place(&mut pc, usize::from(k - 1), k);
        // Todo lo colocado hasta el momento sigue legible.
        for j in 1..=k {
            let at = usize::from(j - 1);
            let (x, y, z) = coords(at);
            assert_eq!(pc.get(x, y, z), j, "la posición {at} debería valer {j}");
        }
    }
    assert!(matches!(pc, PalettedChunk::Direct(_)));
    // Una posición nunca tocada sigue a 0.
    let (x, y, z) = coords(400);
    assert_eq!(pc.get(x, y, z), 0);
}

#[test]
fn compact_reduce_la_representacion_al_minimo() {
    // Un bloque distinto sobre Uniform: compactado queda en paleta de 1 bit.
    let mut pc = PalettedChunk::filled(0);
    pc.set(0, 0, 0, 1);
    pc.compact();
    assert!(matches!(pc, PalettedChunk::Palette { bits: 1, .. }));
    // Devuelto a 0 y compactado: Uniform de nuevo.
    pc.set(0, 0, 0, 0);
    pc.compact();
    assert!(matches!(pc, PalettedChunk::Uniform(0)));

    // Paleta de 4 bits con 16 distintos que se quedan en 2: baja a 1 bit.
    let mut pc = PalettedChunk::filled(0);
    for k in 1..=15u16 {
        place(&mut pc, usize::from(k - 1), k);
    }
    assert!(matches!(pc, PalettedChunk::Palette { bits: 4, .. }));
    for k in 2..=15u16 {
        place(&mut pc, usize::from(k - 1), 1);
    }
    pc.compact();
    assert!(matches!(pc, PalettedChunk::Palette { bits: 1, .. }));
    // Los valores se conservan: posiciones 0..14 a 1 y el resto a 0.
    for i in 0..15usize {
        let (x, y, z) = coords(i);
        assert_eq!(pc.get(x, y, z), 1);
    }
    let (x, y, z) = coords(100);
    assert_eq!(pc.get(x, y, z), 0);

    // Un Direct con un único id se vuelve Uniform.
    let mut pc = PalettedChunk::Direct(Box::new([7; CHUNK_VOLUME]));
    pc.compact();
    assert!(matches!(pc, PalettedChunk::Uniform(7)));
}

#[test]
fn ida_y_vuelta_con_chunk_de_grisu_voxel() {
    // Patrón variado más un tramo largo de un mismo id.
    let mut chunk = Chunk::default();
    for i in 0..CHUNK_VOLUME {
        let (x, y, z) = coords(i);
        chunk.set(x, y, z, ((i * 7) % 6) as u8);
    }
    for i in 512..1024 {
        let (x, y, z) = coords(i);
        chunk.set(x, y, z, 5);
    }
    // 6 ids distintos: la paleta mínima es de 4 bits.
    let pc = PalettedChunk::from_chunk(&chunk);
    assert!(matches!(pc, PalettedChunk::Palette { bits: 4, .. }));
    assert_eq!(pc.to_chunk_lossy(), chunk);

    // Un chunk todo aire es Uniform(0).
    let aire = PalettedChunk::from_chunk(&Chunk::default());
    assert!(matches!(aire, PalettedChunk::Uniform(0)));
}

#[test]
fn to_chunk_lossy_convierte_los_ids_altos_en_aire() {
    let pc = PalettedChunk::Direct(Box::new([300; CHUNK_VOLUME]));
    let chunk = pc.to_chunk_lossy();
    assert_eq!(chunk, Chunk::default());
    let (x, y, z) = coords(123);
    assert_eq!(chunk.get(x, y, z), AIR);
}

#[test]
fn set_con_el_mismo_id_no_promociona_uniform() {
    let mut pc = PalettedChunk::filled(9);
    pc.set(3, 4, 5, 9);
    assert!(matches!(pc, PalettedChunk::Uniform(9)));
    assert_eq!(pc.get(3, 4, 5), 9);
}

#[test]
fn empaqueta_lsb_first_con_tamanos_exactos() {
    let mut pc = PalettedChunk::filled(0);
    pc.set(0, 0, 0, 1);
    // 2 distintos: 1 bit, 64 palabras, entradas en orden de aparición y el
    // índice 1 de la posición 0 en el bit menos significativo de la palabra 0.
    let (bits, entries, packed) = as_palette(&pc);
    assert_eq!(*bits, 1);
    assert_eq!(entries, &[0, 1]);
    assert_eq!(packed.len(), CHUNK_VOLUME / 64);
    assert_eq!(packed.first(), Some(&1u64));

    // 4 distintos: 2 bits y 128 palabras.
    place(&mut pc, 1, 2);
    place(&mut pc, 2, 3);
    let (bits, _, packed) = as_palette(&pc);
    assert_eq!(*bits, 2);
    assert_eq!(packed.len(), CHUNK_VOLUME / 32);

    // 16 distintos: 4 bits y 256 palabras.
    for k in 4..=15u16 {
        place(&mut pc, usize::from(k - 1), k);
    }
    let (bits, _, packed) = as_palette(&pc);
    assert_eq!(*bits, 4);
    assert_eq!(packed.len(), CHUNK_VOLUME / 16);

    // 17 distintos: 8 bits y 512 palabras.
    place(&mut pc, 15, 16);
    let (bits, _, packed) = as_palette(&pc);
    assert_eq!(*bits, 8);
    assert_eq!(packed.len(), CHUNK_VOLUME / 8);
}
