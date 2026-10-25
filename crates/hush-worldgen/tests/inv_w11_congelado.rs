//! `INV-W11`: resumen congelado del generador del mundo grande (W1).
//!
//! Un cambio involuntario del generador (refactor, orden de evaluación,
//! constante de ruido) produce chunks distintos en alguna banda y rompe el
//! resumen; un cambio intencionado exige subir `WORLDGEN_VERSION` y volver
//! a congelar la tabla. La receta del digest documenta la aritmética exacta
//! para poder reproducirla fuera de Rust.

use grisu_voxel::ChunkPos;
use hush_worldgen::{WORLDGEN_VERSION, generate_chunk, sample_column};

/// Semilla única de los chunks de referencia.
const SEED: u64 = 42;

/// Base de FNV-1a de 64 bits.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// Primo de FNV-1a de 64 bits.
const FNV_PRIME: u64 = 0x0000_0100_0000_001b;

/// Resumen FNV-1a de 64 bits del chunk con la versión mezclada al final.
///
/// Receta exacta (reproducible fuera de Rust): se inicializa `h` con el
/// `offset_basis` de FNV-1a de 64 bits; por cada uno de los 4096 bytes del
/// chunk (orden lineal de `as_bytes`: `x + z·16 + y·256`) se hace
/// `h ^= byte; h = h · prime (módulo 2⁶⁴, aritmética envolvente)`. Después,
/// por cada uno de los 8 bytes little-endian de `WORLDGEN_VERSION` como
/// `u64`, otro paso idéntico. El resultado es el digest congelado.
fn chunk_digest(seed: u64, pos: ChunkPos) -> u64 {
    let mut h = FNV_OFFSET_BASIS;
    for &b in generate_chunk(seed, pos).as_bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    for b in u64::from(WORLDGEN_VERSION).to_le_bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// Los 32 chunks de referencia de `SEED`, por bandas del mundo:
///
/// · Origen: el anillo `(±1, −1, ±1)` y vecinos con el frente de
///   superficie (banda con sólido y aire), uno más profundo (`cy = −2`) y
///   el cielo vacío sobre el origen (atajo de chunk entero).
/// · Coordenadas negativas puras (las tres componentes negativas).
/// · Bóveda: un pilar (`chunk_y` −100) y una isla (−150); Raíces con oro
///   y diamante (`chunk_y` −200) y un hueco remoto de la Bóveda.
/// · Lecho (`chunk_y` −260, todo roca dura) también en una esquina lejana.
/// · Manto (`chunk_y` −30, vetas y bolsas de lava), su frontera con la
///   Bóveda (`chunk_y` −40) y una muestra en el borde exacto del mundo.
/// · Las cuatro esquinas `±468_740` en la banda de superficie y varios
///   `±468_749` con signos mezclados (borde exacto del mundo).
// Congelados con WORLDGEN_VERSION = 1; se recalculan al subir la versión.
const EXPECTED: [([i32; 3], u64); 32] = [
    ([0, -1, 0], 0x1835_47c9_5d21_cca5),
    ([1, -1, 0], 0x644a_3ac8_35b4_48d5),
    ([-1, -1, 0], 0xfedd_0dff_fc66_1e33),
    ([0, -1, 1], 0x7ba5_765c_7f59_4207),
    ([1, -1, 1], 0xbadb_0df3_756c_647e),
    ([-1, -1, -1], 0xf00e_6a65_8f3d_69f3),
    ([-1, -1, 1], 0xf872_4fe3_76a3_b0e0),
    ([1, -1, -1], 0x8f5b_bde1_ea54_ebce),
    ([0, -2, 0], 0x54a6_859e_1291_ed74),
    ([0, 0, 0], 0x8d70_3da8_bfb7_82a4),
    ([-3, -1, -5], 0x23f9_e346_8706_30c2),
    ([-2, -2, -2], 0xae07_e428_9df7_1701),
    ([-7, -1, -9], 0x4003_25c5_a019_fac3),
    ([-4, -100, -6], 0x5639_68e1_d3de_f764),
    ([5, -150, -2], 0xbae7_0f60_2975_1ca9),
    ([-4, -200, -6], 0x64ee_3784_2e1e_a310),
    ([0, -260, 0], 0x86ce_2a04_1d93_52a4),
    ([468_740, -260, -468_740], 0x86ce_2a04_1d93_52a4),
    ([0, -30, 0], 0x5864_f32e_2a16_b378),
    ([3, -30, -2], 0x24fe_3d85_c508_9eea),
    ([0, -40, 0], 0x90c0_4bf1_954c_dbd4),
    ([468_740, 0, 468_740], 0xf979_74eb_4b46_98be),
    ([-468_740, 0, -468_740], 0xacd1_68d3_0022_a588),
    ([468_740, 0, -468_740], 0xbd53_b362_3bf6_a38a),
    ([-468_740, 0, 468_740], 0x6ec5_bc84_9973_838d),
    ([468_749, 1, -468_749], 0x6c4d_3fcb_19de_b060),
    ([-468_749, -1, 468_749], 0x5871_5d05_f95e_7588),
    ([-468_749, 0, 468_749], 0x3b24_d5de_2be5_87f4),
    ([468_740, -30, -468_740], 0x5336_1fdc_4fc0_d423),
    ([468_749, -30, -468_749], 0xca97_2d23_6d12_8335),
    ([468_740, -100, 468_740], 0x8d70_3da8_bfb7_82a4),
    ([300, -1, -700], 0x5d94_88a9_c60b_e3d9),
];

/// Cada chunk de referencia genera exactamente el digest congelado: el
/// generador W1 queda «congelado» hasta que alguien suba la versión a
/// propósito.
#[test]
fn frozen_chunks_match_the_expected_digests() {
    assert_eq!(WORLDGEN_VERSION, 1, "la tabla se congeló con la versión 1");
    for &(pos, want) in &EXPECTED {
        let got = chunk_digest(SEED, ChunkPos::new(pos[0], pos[1], pos[2]));
        assert_eq!(
            got, want,
            "chunk {pos:?}: digest {got:#018x} frente al congelado {want:#018x}"
        );
    }
}

/// Barrida de clima lejos del origen (paso 256, centrada en −8 000): el
/// selector de biomas debe usar al menos 5 de los 8 biomas. Se congela el
/// mínimo, no la lista exacta.
#[test]
fn biome_sweep_far_from_origin_uses_at_least_five_biomes() {
    let mut seen = [false; 8];
    for i in 0..64i32 {
        for j in 0..64i32 {
            seen[sample_column(SEED, i * 256 - 8_000, j * 256 - 8_000).biome as usize] = true;
        }
    }
    let distinct = seen.iter().fold(0, |n, &s| n + usize::from(s));
    assert!(
        distinct >= 5,
        "solo {distinct} biomas en la barrida de 64×64"
    );
}

/// Barato y diario: el digest es estable entre llamadas y la semilla
/// participa en él (en un chunk de terreno, no en los atajos de lecho o
/// cielo, que son iguales para cualquier semilla).
#[test]
fn digest_is_stable_and_depends_on_the_seed() {
    let pos = ChunkPos::new(0, -1, 0);
    let a = chunk_digest(SEED, pos);
    let b = chunk_digest(SEED, pos);
    assert_eq!(a, b, "dos llamadas del mismo chunk difieren");
    assert_ne!(
        a,
        chunk_digest(SEED + 1, pos),
        "la semilla no cambia el digest"
    );
}
