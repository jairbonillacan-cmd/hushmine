//! Pruebas de la rotación entera exacta.
//!
//! Nada de `f32`/`f64`: todas las normas se comparan al cuadrado con
//! enteros, así que no hay epsilon de comparación ni redondeos ocultos.

use super::*;

/// Comprobación en contexto `const`: las dos rotaciones son `const fn`.
const ROT_2D: (i64, i64) = rotate2_pythagorean(3, 1);
const ROT_3D: (i64, i64, i64) = rotate3_orthogonal(1, -1, 0);

#[test]
fn valores_de_referencia() {
    // Demostración de que son `const fn`: compiladas en contexto const.
    assert_eq!(ROT_2D, (1, 3));
    assert_eq!(ROT_3D, (0, 1, -1));

    // Rellenados con la salida real; cualquier cambio accidental de los
    // numeradores, del denominador o del redondeo los rompe. Los casos con
    // división exacta conservan la norma: 100² + 35² == 88² + 59².
    assert_eq!(rotate2_pythagorean(3, 1), (1, 3));
    assert_eq!(rotate2_pythagorean(100, -35), (88, 59));
    assert_eq!(rotate2_pythagorean(1, 4), (-3, 3));
    assert_eq!(rotate2_pythagorean(-123, 456), (-439, 175));
    assert_eq!(
        rotate2_pythagorean(7_499_999, -7_499_999),
        (10_499_998, 1_499_999)
    );
    assert_eq!(
        rotate2_pythagorean(-7_499_999, 7_499_999),
        (-10_499_999, -1_500_000)
    );
    assert_eq!(
        rotate2_pythagorean(7_499_999, 7_499_999),
        (-1_500_000, 10_499_998)
    );

    assert_eq!(rotate3_orthogonal(1, 1, 1), (1, 1, 1));
    assert_eq!(rotate3_orthogonal(1, -1, 0), (0, 1, -1));
    assert_eq!(rotate3_orthogonal(100, -35, 7), (41, 83, -52));
    assert_eq!(rotate3_orthogonal(2, 1, -1), (2, 0, -1));
    assert_eq!(
        rotate3_orthogonal(-7_499_999, 0, 7_499_999),
        (-7_499_999, 0, 7_499_999)
    );
    assert_eq!(
        rotate3_orthogonal(7_499_999, 7_499_999, 7_499_999),
        (7_499_999, 7_499_999, 7_499_999)
    );
    assert_eq!(
        rotate3_orthogonal(7_499_999, -7_499_999, 12_345_678),
        (-4_115_226, 15_730_451, 730_453)
    );
}

#[test]
fn rotate2_preserva_la_norma_en_multiplos_de_5() {
    // Con `x = 3z + 5t` se cumple `x ≡ 3z (mod 5)`: los dos numeradores son
    // múltiplos de 5, la división es exacta y la norma al cuadrado se
    // conserva sin pérdida (la matriz es ortogonal de norma² 25).
    for z in -50..=50 {
        for t in -2..=2 {
            let x = 3 * z + 5 * t;
            let (rx, rz) = rotate2_pythagorean(x, z);
            assert_eq!(
                rx * rx + rz * rz,
                x * x + z * z,
                "norma 2D alterada en ({x}, {z})"
            );
        }
    }
}

#[test]
fn rotate3_preserva_la_norma_en_multiplos_de_3() {
    // Familias con `x + y + z ≡ 0 (mod 3)`: los tres numeradores son
    // múltiplos de 3 y la norma al cuadrado se conserva exactamente.
    for a in -8..=8 {
        for b in -8..=8 {
            let (x, y, z) = (3 * a, 3 * b, 0);
            let (rx, ry, rz) = rotate3_orthogonal(x, y, z);
            assert_eq!(
                rx * rx + ry * ry + rz * rz,
                x * x + y * y + z * z,
                "norma 3D alterada en ({x}, {y}, {z})"
            );
            for c in -8..=8 {
                let (x, y, z) = (3 * a + 1, 3 * b + 1, 3 * c + 1);
                let (rx, ry, rz) = rotate3_orthogonal(x, y, z);
                assert_eq!(
                    rx * rx + ry * ry + rz * rz,
                    x * x + y * y + z * z,
                    "norma 3D alterada en ({x}, {y}, {z})"
                );
            }
        }
    }
}

#[test]
fn rotate2_cota_el_error_en_el_rango_del_mundo() {
    // Barrido determinista sin RNG en ±7 500 000. El redondeo hacia −∞ no
    // encoge la norma en los dos ejes a la vez: una componente negativa
    // redondea alejándose del origen (p. ej. `(1, 4) -> (−3, 3)`), así que
    // la diferencia de normas² puede ser positiva y se acota en valor
    // absoluto: |salida − entrada| ≤ 4·(|x| + |z|) + 8.
    for i in 0..20_000i64 {
        let x = (i * 7919) % 15_000_001 - 7_500_000;
        let z = (i * 104_729 + 123_457) % 15_000_001 - 7_500_000;
        let (rx, rz) = rotate2_pythagorean(x, z);
        let entrada = x * x + z * z;
        let salida = rx * rx + rz * rz;
        let cota = 4 * (x.abs() + z.abs()) + 8;
        assert!(
            (salida - entrada).abs() <= cota,
            "error 2D fuera de cota en ({x}, {z}): {salida} vs {entrada}"
        );
    }
}

#[test]
fn rotate3_cota_el_error_en_el_rango_del_mundo() {
    // Igual que en 2D: con restos en [0, 3) la diferencia de normas² queda
    // acotada por 6·(|x| + |y| + |z|) + 12 en valor absoluto.
    for i in 0..20_000i64 {
        let x = (i * 7919) % 15_000_001 - 7_500_000;
        let y = (i * 104_729 + 123_457) % 15_000_001 - 7_500_000;
        let z = (i * 65_537 + 9_000_001) % 15_000_001 - 7_500_000;
        let (rx, ry, rz) = rotate3_orthogonal(x, y, z);
        let entrada = x * x + y * y + z * z;
        let salida = rx * rx + ry * ry + rz * rz;
        let cota = 6 * (x.abs() + y.abs() + z.abs()) + 12;
        assert!(
            (salida - entrada).abs() <= cota,
            "error 3D fuera de cota en ({x}, {y}, {z}): {salida} vs {entrada}"
        );
    }
}

#[test]
fn extremos_i64_sin_panico_y_deterministas() {
    // Los intermedios en `i128` absorben cualquier trío de `i64` y el
    // truncamiento final `as i64` está definido: en debug, cualquier
    // desborde aritmético haría fallar esta prueba.
    let extremos = [i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX - 1, i64::MAX];
    for &x in &extremos {
        for &z in &extremos {
            let una = rotate2_pythagorean(x, z);
            assert_eq!(
                rotate2_pythagorean(x, z),
                una,
                "2D no determinista en ({x}, {z})"
            );
            for &y in &extremos {
                let una = rotate3_orthogonal(x, y, z);
                assert_eq!(
                    rotate3_orthogonal(x, y, z),
                    una,
                    "3D no determinista en ({x}, {y}, {z})"
                );
            }
        }
    }
    // Valores de referencia en los extremos: el truncamiento de `i128` a
    // `i64` es el envolvimiento en complemento a dos, no un pánico.
    assert_eq!(
        rotate2_pythagorean(i64::MAX, i64::MAX),
        (-1_844_674_407_370_955_162, -5_534_023_222_112_865_487)
    );
    assert_eq!(
        rotate2_pythagorean(i64::MIN, i64::MIN),
        (1_844_674_407_370_955_161, 5_534_023_222_112_865_484)
    );
    assert_eq!(
        rotate2_pythagorean(i64::MIN, i64::MAX),
        (5_534_023_222_112_865_485, -1_844_674_407_370_955_163)
    );
    // 3·i64::MIN / 3 es exacto en i128: el mundo entero cabe sin truncar.
    assert_eq!(
        rotate3_orthogonal(i64::MIN, i64::MIN, i64::MIN),
        (i64::MIN, i64::MIN, i64::MIN)
    );
    assert_eq!(
        rotate3_orthogonal(i64::MAX, i64::MAX, i64::MAX),
        (i64::MAX, i64::MAX, i64::MAX)
    );
    assert_eq!(
        rotate3_orthogonal(i64::MIN, i64::MAX, 0),
        (-1, i64::MIN, i64::MAX)
    );
}

#[test]
fn los_wrappers_rotan_antes_de_muestrear() {
    let n2 = Noise2::new(42, 7, 5);
    let f2 = Fbm2::new(42, 1, 6, 4);
    let n3 = Noise3::new(42, 7, 6);
    let f3 = Fbm3::new(42, 1, 6, 4);
    // Los fBm no son Copy: se envuelven una vez y se reutilizan.
    let env2 = SampleRotated2::new(n2);
    let env2_fbm = SampleRotated2::new(f2.clone());
    let env3 = SampleRotated3::new(n3);
    let env3_fbm = SampleRotated3::new(f3.clone());
    let puntos: [(i64, i64); 6] = [
        (0, 0),
        (13, -7),
        (-250, 111),
        (4_321, 987),
        (-65_536, 65_536),
        (7_499_999, -7_499_999),
    ];
    for &(x, z) in &puntos {
        let (rx, rz) = rotate2_pythagorean(x, z);
        assert_eq!(
            env2.sample_q16(x, z),
            n2.sample_q16(rx, rz),
            "2D no rota antes de muestrear en ({x}, {z})"
        );
        assert_eq!(
            env2_fbm.sample_q16(x, z),
            f2.sample_q16(rx, rz),
            "fBm 2D no rota antes de muestrear en ({x}, {z})"
        );
        // Paridad de `sample_block`: desplazar y muestrear es lo mismo que
        // muestrear en Q16 con las coordenadas desplazadas.
        assert_eq!(env2.sample_block(x, z), env2.sample_q16(x << 16, z << 16));
        assert_eq!(
            env2_fbm.sample_block(x, z),
            env2_fbm.sample_q16(x << 16, z << 16)
        );
    }
    let puntos3: [(i64, i64, i64); 5] = [
        (0, 0, 0),
        (13, -7, 5),
        (-250, 111, -3),
        (4_321, 987, -12_345),
        (7_499_999, -7_499_999, 123),
    ];
    for &(x, y, z) in &puntos3 {
        let (rx, ry, rz) = rotate3_orthogonal(x, y, z);
        assert_eq!(
            env3.sample_q16(x, y, z),
            n3.sample_q16(rx, ry, rz),
            "3D no rota antes de muestrear en ({x}, {y}, {z})"
        );
        assert_eq!(
            env3_fbm.sample_q16(x, y, z),
            f3.sample_q16(rx, ry, rz),
            "fBm 3D no rota antes de muestrear en ({x}, {y}, {z})"
        );
        assert_eq!(
            env3.sample_block(x, y, z),
            env3.sample_q16(x << 16, y << 16, z << 16)
        );
        assert_eq!(
            env3_fbm.sample_block(x, y, z),
            env3_fbm.sample_q16(x << 16, y << 16, z << 16)
        );
    }
    // El acceso al ruido envuelto devuelve el mismo muestreo.
    assert_eq!(env2.inner().sample_q16(19, -4), n2.sample_q16(19, -4));
    assert_eq!(
        env3_fbm.inner().sample_block(3, -9, 27),
        f3.sample_block(3, -9, 27)
    );
}
