//! `hush-cli world chunk`: resumen SHA-256 y recuento de bloques de un chunk.
//!
//! El subcomando genera el chunk pedido con `hush_worldgen`, resume sus
//! 4096 bytes con un SHA-256 propio (FIPS 180-4, solo enteros `u32`, sin
//! dependencias) y cuenta sus bloques no vacíos. Con `--digest` imprime
//! primero la línea hexadecimal del resumen y después el informe JSON; sin
//! él, imprime solo el JSON sin el campo «sha256».

use std::collections::BTreeMap;

use anyhow::Result;
use grisu_voxel::{AIR, BlockId, ChunkPos};
use hush_worldgen::{DIAMOND, DIRT, GOLD, HARD_ROCK, ORE, ROCK, generate_chunk};

/// Constantes K de SHA-256 (FIPS 180-4, sección 4.2.2).
const K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// Analiza una coordenada de chunk con la forma «cx,cy,cz»: exactamente tres
/// valores `i32` separados por comas, tolerando espacios alrededor de cada
/// valor y negativos. Cualquier otra cosa devuelve `None`.
pub fn parse_chunk(text: &str) -> Option<(i32, i32, i32)> {
    let parts: Vec<&str> = text.split(',').collect();
    if parts.len() != 3 {
        return None;
    }
    let mut values = [0i32; 3];
    for (slot, part) in values.iter_mut().zip(parts) {
        *slot = part.trim().parse::<i32>().ok()?;
    }
    Some((values[0], values[1], values[2]))
}

/// SHA-256 puro de `data` (FIPS 180-4): relleno estándar con la longitud en
/// bits en big-endian y una compresión por cada bloque de 512 bits.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut state: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    let (blocks, rest) = data.as_chunks::<64>();
    for block in blocks {
        compress(&mut state, block);
    }
    // Cola con relleno: datos restantes, un 0x80, ceros y la longitud en
    // bits (big-endian). Con 56 bytes o más de resto la longitud no cabe en
    // el bloque y hace falta uno más.
    let mut tail = [0u8; 128];
    tail[..rest.len()].copy_from_slice(rest);
    tail[rest.len()] = 0x80;
    let padded = if rest.len() < 56 { 64 } else { 128 };
    let bits = (data.len() as u64) * 8;
    tail[padded - 8..padded].copy_from_slice(&bits.to_be_bytes());
    for block in tail[..padded].as_chunks::<64>().0 {
        compress(&mut state, block);
    }
    let mut digest = [0u8; 32];
    for (i, word) in state.iter().enumerate() {
        digest[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

/// Compresión de un bloque de 512 bits (FIPS 180-4, secciones 4.1.2 y 4.2.2):
/// calendario de mensaje `W[0..64]` y 64 rondas sobre el estado.
fn compress(state: &mut [u32; 8], block: &[u8; 64]) {
    let mut w = [0u32; 64];
    for (slot, word) in w.iter_mut().zip(block.as_chunks::<4>().0) {
        *slot = u32::from_be_bytes(*word);
    }
    for i in 16..64 {
        let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
        let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16]
            .wrapping_add(s0)
            .wrapping_add(w[i - 7])
            .wrapping_add(s1);
    }
    let mut v = *state;
    for i in 0..64 {
        let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
        let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
        let t1 = v[7]
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(K[i])
            .wrapping_add(w[i]);
        let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
        let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
        let t2 = s0.wrapping_add(maj);
        v = [
            t1.wrapping_add(t2),
            v[0],
            v[1],
            v[2],
            v[3].wrapping_add(t1),
            v[4],
            v[5],
            v[6],
        ];
    }
    for i in 0..8 {
        state[i] = state[i].wrapping_add(v[i]);
    }
}

/// SHA-256 de `data` en hexadecimal minúscula (exactamente 64 caracteres).
pub fn sha256_hex(data: &[u8]) -> String {
    sha256(data).iter().map(|b| format!("{b:02x}")).collect()
}

/// Nombre de informe para un id de bloque generado.
fn block_name(block: BlockId) -> String {
    let name = match block {
        DIRT => "DIRT",
        ROCK => "ROCK",
        HARD_ROCK => "HARD_ROCK",
        ORE => "ORE",
        GOLD => "GOLD",
        DIAMOND => "DIAMOND",
        7 => "LAVA",
        other => return format!("ID{other}"),
    };
    name.to_string()
}

/// Ejecuta el subcomando `world chunk`.
pub fn run(seed: u64, chunk: (i32, i32, i32), digest: bool) -> Result<()> {
    let (cx, cy, cz) = chunk;
    let generated = generate_chunk(seed, ChunkPos::new(cx, cy, cz));
    let summary = sha256_hex(generated.as_bytes());

    // Recuento de bloques no vacíos en un BTreeMap: orden determinista.
    let mut counts: BTreeMap<u8, usize> = BTreeMap::new();
    for &block in generated.as_bytes() {
        if block != AIR {
            *counts.entry(block).or_insert(0) += 1;
        }
    }
    let blocks: BTreeMap<String, usize> = counts
        .into_iter()
        .map(|(block, count)| (block_name(block), count))
        .collect();

    let report = if digest {
        println!("{summary}");
        serde_json::json!({
            "seed": seed,
            "chunk": [cx, cy, cz],
            "sha256": summary,
            "blocks": blocks,
        })
    } else {
        serde_json::json!({
            "seed": seed,
            "chunk": [cx, cy, cz],
            "blocks": blocks,
        })
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vectores FIPS 180-4 (más el caso que fuerza dos bloques).
    #[test]
    fn sha256_known_vectors() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(&[b'a'; 64]),
            "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb"
        );
        // 56 bytes: la longitud no cabe en el primer bloque de relleno.
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn sha256_hex_is_lowercase_hex_of_64_chars() {
        let hex = sha256_hex(b"hushmine");
        assert_eq!(hex.len(), 64);
        assert!(
            hex.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        );
    }

    #[test]
    fn parse_chunk_accepts_three_i32() {
        assert_eq!(
            parse_chunk("-468750,-1,468749"),
            Some((-468750, -1, 468749))
        );
        assert_eq!(parse_chunk(" -1 , 2 , 3 "), Some((-1, 2, 3)));
        assert_eq!(parse_chunk("0,0,0"), Some((0, 0, 0)));
    }

    #[test]
    fn parse_chunk_rejects_anything_else() {
        for bad in [
            "1,2", "a,b,c", "", "1,2,3,4", "1,,2", ",", "1,2,", "1.5,2,3", "  ",
        ] {
            assert_eq!(parse_chunk(bad), None, "debería rechazar «{bad}»");
        }
    }

    #[test]
    fn chunk_digest_is_deterministic_and_position_dependent() {
        // Chunk bajo la superficie (y = -16..-1): siempre con bloques.
        let pos = ChunkPos::new(0, -1, 0);
        let first = sha256_hex(generate_chunk(42, pos).as_bytes());
        let second = sha256_hex(generate_chunk(42, pos).as_bytes());
        assert_eq!(
            first, second,
            "la misma semilla y posición deben dar el mismo resumen"
        );
        let neighbor = sha256_hex(generate_chunk(42, ChunkPos::new(1, -1, 0)).as_bytes());
        assert_ne!(
            first, neighbor,
            "otra posición no debería chocar con el mismo resumen"
        );
    }
}
