//! Bóveda del mundo grande (W1, fase 2): el vacío de kilómetros bajo el
//! manto, con pilares de roca que suben desde el lecho e islas suspendidas.
//!
//! Todo lo decide el ruido 2D de la columna (dos fBm con sales propias)
//! más dos hash para la altura y el grosor de las islas: sin flotantes ni
//! estado, así que [`vault_solid`] es una función pura de (semilla,
//! posición). Por columna, el pilar es el intervalo enraizado en el lecho
//! `y ≤ pillar_top` y la isla un intervalo flotante `[top − grosor, top]`;
//! cuando ambos se tocan, la columna es un único intervalo de sólido.
//! Solo se consulta con `y ≤ super::MANTO_BOTTOM` (desde
//! `super::raw_solid`); el lecho (`y ≤ super::BEDROCK_Y`) se decide antes
//! y siempre es sólido.

use grisu_noise::{Fbm2, ONE, hash2d};

use super::MANTO_BOTTOM;

/// Sal del fBm 2D de pilares de la Bóveda.
const SALT_VAULT_PILLAR: u64 = 0x22D0_0000_0000_0002;
/// Sal del fBm 2D de islas de la Bóveda.
const SALT_VAULT_ISLAND: u64 = 0x22D0_0000_0000_0003;
/// Sal del hash de profundidad del techo de las islas.
const SALT_ISLAND_DEPTH: u64 = 0x22D0_0000_0000_0004;
/// Sal del hash de grosor de las islas.
const SALT_ISLAND_THICKNESS: u64 = 0x22D0_0000_0000_0005;

/// Umbral del ruido de pilares: 0,18 en Q16.
///
/// Calibración (W1, semilla 42, 65 536 columnas en cuatro regiones): el
/// `Fbm2` de 3 octavas queda acotado por ±`ONE/2` y en la práctica llega a
/// ~0,26·ONE; con el umbral a 0,18·ONE pasa la prueba en torno al 2,4 % de
/// las columnas, suficiente para un bosque de pilares disperso.
const PILLAR_START: i64 = (18 * ONE) / 100;

/// Ruido de pilar que llena todo el vuelo: el máximo medido de la capa
/// (~0,26·ONE). Normalizar por él (y no por ONE, inalcanzable) hace que los
/// pilares altos de verdad recorran la Bóveda.
const PILLAR_FULL: i64 = (26 * ONE) / 100;

/// Umbral del ruido de islas: 0,20 en Q16 (~1 % de las columnas; el máximo
/// medido de su capa es ~0,29·ONE). Más raras que los pilares.
const ISLAND_START: i64 = (20 * ONE) / 100;

/// Vuelo máximo del pilar bajo `MANTO_BOTTOM`, en bloques: llega hasta
/// [`ROOTS_TOP`].
const PILLAR_DROP: i64 = 2400;

/// Techo del pilar de una columna con ruido `p > PILLAR_START`:
/// `MANTO_BOTTOM` menos un vuelo entero de 0 a [`PILLAR_DROP`] bloques
/// según la intensidad del ruido (división entera, normalizada por el
/// máximo real de la capa). Queda en
/// `[MANTO_BOTTOM - PILLAR_DROP, MANTO_BOTTOM]`, es decir [−3040, −640],
/// siempre por encima de `super::ROOTS_TOP` (−3072).
fn pillar_top(p: i64) -> i64 {
    let flight = ((p - PILLAR_START) * PILLAR_DROP) / (PILLAR_FULL - PILLAR_START);
    // Un pico raro por encima del máximo medido no puede bajar a las Raíces.
    (MANTO_BOTTOM - flight).max(super::ROOTS_TOP + 1)
}

/// Techo de la isla de la columna: entre −1600 y −2799 según un hash,
/// lejos del techo de la Bóveda.
fn island_top(seed: u64, x: i64, z: i64) -> i64 {
    -1600 - (hash2d(x, z, seed ^ SALT_ISLAND_DEPTH) % 1200) as i64
}

/// Grosor de la isla de la columna: entre 24 y 96 bloques según un hash.
fn island_thickness(seed: u64, x: i64, z: i64) -> i64 {
    24 + (hash2d(x, z, seed ^ SALT_ISLAND_THICKNESS) % 73) as i64
}

/// Crudeza de la Bóveda en `(x, y, z)`: cierto si la columna tiene pilar y
/// `y` queda por debajo de su techo, o si tiene isla y `y` cae dentro de
/// su intervalo flotante. Fuera de eso, aire.
#[must_use]
pub(crate) fn vault_solid(seed: u64, x: i64, y: i64, z: i64) -> bool {
    let pillar = Fbm2::new(seed, SALT_VAULT_PILLAR, 7, 3).sample_block(x, z);
    if pillar > PILLAR_START && y <= pillar_top(pillar) {
        return true;
    }
    let island = Fbm2::new(seed, SALT_VAULT_ISLAND, 6, 3).sample_block(x, z);
    if island > ISLAND_START {
        let top = island_top(seed, x, z);
        let bottom = top - island_thickness(seed, x, z);
        if bottom <= y && y <= top {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::super::{BEDROCK_Y, ROOTS_TOP, raw_solid};
    use super::*;

    /// Semilla fija de las pruebas de la Bóveda.
    const SEED: u64 = 42;

    #[test]
    fn vault_is_monotone_per_column() {
        // El sólido de cada columna de la muestra es un intervalo, quizá
        // vacío: sin huecos interiores ni alternancias. El barrido cubre
        // todo el rango vertical de la Bóveda.
        for ix in 0..4i64 {
            for iz in 0..4i64 {
                let (x, z) = (ix * 7 - 10, iz * 5 - 7);
                let mut first: Option<i64> = None;
                let mut last = 0;
                let mut count = 0u32;
                for y in (BEDROCK_Y + 1)..=MANTO_BOTTOM {
                    if vault_solid(SEED, x, y, z) {
                        first.get_or_insert(y);
                        last = y;
                        count += 1;
                    }
                }
                if let Some(first) = first {
                    assert_eq!(
                        last - first + 1,
                        i64::from(count),
                        "la columna ({x},{z}) tiene huecos en la Bóveda"
                    );
                }
            }
        }
    }

    #[test]
    fn calibrated_vault_is_not_empty() {
        // Con los umbrales calibrados (~2,4 % de pilares y ~1 % de islas),
        // una rejilla de 64×64 columnas siempre contiene algún sólido en la
        // vertical de la Bóveda.
        let mut solidas = 0usize;
        for ix in 0..64i64 {
            for iz in 0..64i64 {
                let (x, z) = (ix * 11 - 350, iz * 7 - 224);
                if (BEDROCK_Y + 1..=MANTO_BOTTOM).any(|y| vault_solid(SEED, x, y, z)) {
                    solidas += 1;
                }
            }
        }
        assert!(solidas > 0, "Bóveda vacía con la calibración actual");
    }

    #[test]
    fn pillar_tops_stay_inside_the_vault() {
        for p in (PILLAR_START + 1)..=PILLAR_FULL {
            let top = pillar_top(p);
            assert!(
                top <= MANTO_BOTTOM && top > ROOTS_TOP,
                "pilar con techo {top} fuera de la Bóveda"
            );
        }
        // El pilar recién nacido es el más alto; el ruido medido más alto
        // baja exactamente PILLAR_DROP bloques.
        assert_eq!(pillar_top(PILLAR_START + 1), MANTO_BOTTOM);
        assert_eq!(pillar_top(PILLAR_FULL), MANTO_BOTTOM - PILLAR_DROP);
        // Y por encima del máximo medido, el recorte a las Raíces.
        assert!(pillar_top(PILLAR_FULL + 1) > ROOTS_TOP);
        assert!(pillar_top(ONE) > ROOTS_TOP);
    }

    #[test]
    fn islands_stay_in_the_middle_band() {
        for i in 0..64i64 {
            let (x, z) = (i * 11 - 350, i * 7 - 200);
            let top = island_top(SEED, x, z);
            let thickness = island_thickness(SEED, x, z);
            assert!((-2799..=-1600).contains(&top), "techo {top} fuera de rango");
            assert!(
                (24..=96).contains(&thickness),
                "grosor {thickness} fuera de rango"
            );
            // Ni rozan el suelo del manto ni bajan a la zona de Raíces.
            assert!(top < MANTO_BOTTOM - 900, "isla pegada al manto: {top}");
            assert!(top - thickness > ROOTS_TOP, "isla en las Raíces: {top}");
        }
    }

    #[test]
    fn bedrock_is_always_solid() {
        let edge = 7_499_999i64;
        for &(x, z) in &[(0i64, 0i64), (edge, -edge), (-edge, edge), (17, -23)] {
            assert!(raw_solid(SEED, x, BEDROCK_Y, z), "lecho en ({x},{z})");
            assert!(
                raw_solid(SEED, x, BEDROCK_Y - 1, z),
                "bajo el lecho ({x},{z})"
            );
        }
    }
}
