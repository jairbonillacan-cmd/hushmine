//! Pruebas del ruido celular (Voronoi).

use super::*;

/// Comprueba las invariantes de orden de una muestra.
fn afirma_invariantes(s: CellularSample, contexto: &str) {
    assert!(
        (0..=(ONE as i32)).contains(&s.f1_sq),
        "{contexto}: f1_sq fuera de [0, ONE]: {}",
        s.f1_sq
    );
    assert!(
        (s.f1_sq..=(ONE as i32)).contains(&s.f2_sq),
        "{contexto}: f2_sq fuera de [f1_sq, ONE]: {} (f1_sq {})",
        s.f2_sq,
        s.f1_sq
    );
    // El borde sale de las distancias sin saturar: es la resta, o más si f2 satura.
    let resta = s.f2_sq - s.f1_sq;
    let ok = s.edge_q16 == resta || (s.f2_sq == ONE as i32 && s.edge_q16 > resta);
    assert!(ok, "{contexto}: edge_q16 incoherente");
}

/// Valores de referencia rellenados con la salida real; cualquier cambio
/// accidental del muestreo rompería mundos guardados.
#[test]
fn valores_de_referencia() {
    let cerca = CellularSample {
        cell_hash: 1_518_461_086_170_747_402,
        f1_sq: 39_569,
        f2_sq: 52_027,
        edge_q16: 12_458,
    };
    assert_eq!(
        Cellular2::new(42, 4).sample_block(100, -35),
        cerca,
        "referencia 2D con jitter completo"
    );
    assert_eq!(
        Cellular3::new(42, 4).sample_block(100, -35, 7),
        CellularSample {
            cell_hash: 1_518_461_086_170_747_402,
            f1_sq: 42_740,
            f2_sq: 52_335,
            edge_q16: 9_595,
        },
        "referencia 3D con jitter completo"
    );
    let medio = CellularSample {
        cell_hash: 11_685_150_768_351_990_497,
        f1_sq: 16_907,
        f2_sq: 43_331,
        edge_q16: 26_424,
    };
    assert_eq!(
        Cellular2::with_jitter(42, 4, (ONE as i32) / 2).sample_block(100, -35),
        medio,
        "referencia 2D con jitter mitad"
    );
    assert_eq!(
        Cellular3::with_jitter(42, 4, (ONE as i32) / 2).sample_block(100, -35, 7),
        CellularSample {
            cell_hash: 11_685_150_768_351_990_497,
            f1_sq: 17_143,
            f2_sq: 44_106,
            edge_q16: 26_963,
        },
        "referencia 3D con jitter mitad"
    );
    // Cae en la celda (0, 0), cuyo hash es el hash3d(0, 0, 0, 42) de hash.rs.
    let origen = CellularSample {
        cell_hash: 13_679_457_532_755_275_413,
        f1_sq: 16_276,
        f2_sq: 34_627,
        edge_q16: 18_351,
    };
    assert_eq!(
        Cellular2::new(42, 4).sample_block(-1, 2),
        origen,
        "referencia 2D junto al origen"
    );
}

/// En una rejilla con coordenadas negativas y positivas las distancias
/// quedan ordenadas y `edge_q16` es la diferencia exacta, para todas las
/// semillas, períodos y jitters probados.
#[test]
fn invariantes_de_orden_en_una_rejilla() {
    for seed in [42u64, 7, 0xDEAD_BEEF] {
        for k in [1u32, 4, 16] {
            for jitter in [0, (ONE as i32) / 2, ONE as i32] {
                let c2 = Cellular2::with_jitter(seed, k, jitter);
                let c3 = Cellular3::with_jitter(seed, k, jitter);
                for i in 0..24i64 {
                    for j in 0..24i64 {
                        let (x, z) = (i * 7 - 100, j * 5 + 40);
                        let contexto = format!("2D {seed}/{k}/{jitter} en ({x}, {z})");
                        afirma_invariantes(c2.sample_block(x, z), &contexto);
                        let contexto = format!("3D {seed}/{k}/{jitter} en ({x}, {z}, {})", i - j);
                        afirma_invariantes(c3.sample_block(x, z, i - j), &contexto);
                    }
                }
            }
        }
    }
}

/// Con jitter 0 los puntos característicos están en las esquinas mínimas de
/// su celda, así que la frontera de Voronoi entre dos celdas vecinas es el
/// plano medio entre sus esquinas y, en los cruces de planos medios, gana
/// la celda lexicográficamente menor. En el punto medio exacto la distancia
/// es media celda: `(2^30) >> 16 == ONE/4`.
#[test]
fn frontera_entre_celdas_sin_jitter() {
    let k = 4;
    let span = 1i64 << (16 + k);
    let half = span >> 1;
    let seed = 42u64;
    let c2 = Cellular2::with_jitter(seed, k, 0);
    let c3 = Cellular3::with_jitter(seed, k, 0);
    let dentro = [0i64, span / 4, -span / 4, 123];

    // Par en x: frontera en x = m·span − span/2; gana (m−1, mz).
    for m in [-3i64, 0, 5, 40_000] {
        for mz in [-2i64, 1, 30_000] {
            for delta in dentro {
                let s = c2.sample(m * span - half, mz * span + delta);
                assert_eq!(s.f1_sq, s.f2_sq, "par x en ({m}, {mz}): sin empate");
                assert_eq!(s.edge_q16, 0, "par x en ({m}, {mz}): edge no nulo");
                assert_eq!(
                    s.cell_hash,
                    hash2d(m - 1, mz, seed),
                    "par x en ({m}, {mz}): no gana la celda menor"
                );
                if delta == 0 {
                    assert_eq!(s.f1_sq, (ONE as i32) / 4, "par x: distancia media");
                }
            }
        }
    }

    // Par en z: frontera en z = mz·span − span/2; gana (mx, mz−1).
    for mx in [-2i64, 1, 30_000] {
        for mz in [-3i64, 0, 5] {
            for delta in dentro {
                let s = c2.sample(mx * span + delta, mz * span - half);
                assert_eq!(s.f1_sq, s.f2_sq, "par z en ({mx}, {mz}): sin empate");
                assert_eq!(s.edge_q16, 0, "par z en ({mx}, {mz}): edge no nulo");
                assert_eq!(
                    s.cell_hash,
                    hash2d(mx, mz - 1, seed),
                    "par z en ({mx}, {mz}): no gana la celda menor"
                );
                if delta == 0 {
                    assert_eq!(s.f1_sq, (ONE as i32) / 4, "par z: distancia media");
                }
            }
        }
    }

    // Pares en x, y y z en 3D.
    for m in [-3i64, 0, 5] {
        for o in [-2i64, 1] {
            for p in [-1i64, 3] {
                for delta in dentro {
                    let s = c3.sample(m * span - half, o * span + delta, p * span - 77);
                    assert_eq!(s.f1_sq, s.f2_sq, "par 3D x en ({m}, {o}, {p})");
                    assert_eq!(s.edge_q16, 0);
                    assert_eq!(s.cell_hash, hash3d(m - 1, o, p, seed), "par 3D en x");
                    if delta == 0 {
                        assert_eq!(s.f1_sq, (ONE as i32) / 4, "par 3D x: distancia media");
                    }
                    let s = c3.sample(o * span + delta, m * span - half, p * span - 77);
                    assert_eq!(s.f1_sq, s.f2_sq, "par 3D y en ({m}, {o}, {p})");
                    assert_eq!(s.edge_q16, 0);
                    assert_eq!(s.cell_hash, hash3d(o, m - 1, p, seed), "par 3D en y");
                    if delta == 0 {
                        assert_eq!(s.f1_sq, (ONE as i32) / 4, "par 3D y: distancia media");
                    }
                    let s = c3.sample(o * span - 77, p * span + delta, m * span - half);
                    assert_eq!(s.f1_sq, s.f2_sq, "par 3D z en ({m}, {o}, {p})");
                    assert_eq!(s.edge_q16, 0);
                    assert_eq!(s.cell_hash, hash3d(o, p, m - 1, seed), "par 3D en z");
                    if delta == 0 {
                        assert_eq!(s.f1_sq, (ONE as i32) / 4, "par 3D z: distancia media");
                    }
                }
            }
        }
    }

    // Cruce de dos planos medios en 2D: empatan cuatro celdas a distancia
    // diagonal: (2·2^30) >> 16 == ONE/2.
    for m in [-3i64, 2] {
        for mz in [-1i64, 4] {
            let s = c2.sample(m * span - half, mz * span - half);
            assert_eq!(s.f1_sq, s.f2_sq, "cruce 2D en ({m}, {mz})");
            assert_eq!(s.edge_q16, 0, "cruce 2D en ({m}, {mz})");
            assert_eq!(s.f1_sq, (ONE as i32) / 2, "cruce 2D: distancia diagonal");
            assert_eq!(
                s.cell_hash,
                hash2d(m - 1, mz - 1, seed),
                "cruce 2D en ({m}, {mz}): no gana la menor"
            );
        }
    }

    // En 3D: cruce de dos planos (empate a cuatro, tercera coordenada dentro
    // de su banda) y de tres (empate a ocho, distancia (3·2^30) >> 16).
    for m in [-3i64, 2] {
        for o in [-1i64, 4] {
            for p in [-2i64, 3] {
                for delta in dentro {
                    let s = c3.sample(m * span - half, o * span - half, p * span + delta);
                    assert_eq!(
                        s.f1_sq, s.f2_sq,
                        "cruce 3D de dos planos en ({m}, {o}, {p})"
                    );
                    assert_eq!(s.edge_q16, 0);
                    assert_eq!(s.cell_hash, hash3d(m - 1, o - 1, p, seed));
                    if delta == 0 {
                        assert_eq!(s.f1_sq, (ONE as i32) / 2, "cruce 3D: distancia diagonal");
                    }
                }
                let s = c3.sample(m * span - half, o * span - half, p * span - half);
                assert_eq!(
                    s.f1_sq, s.f2_sq,
                    "cruce 3D de tres planos en ({m}, {o}, {p})"
                );
                assert_eq!(s.edge_q16, 0);
                assert_eq!(
                    s.f1_sq,
                    3 * (ONE as i32) / 4,
                    "cruce 3D: distancia tridiagonal"
                );
                assert_eq!(s.cell_hash, hash3d(m - 1, o - 1, p - 1, seed));
            }
        }
    }
}

/// Media, varianza e histograma de 16 cubetas de una muestra. `f64` solo
/// aquí, como en `fbm.rs`; las muestras son enteras.
fn stats(values: &[i32]) -> (f64, f64, [u32; 16]) {
    let n = f64::from(u32::try_from(values.len()).unwrap());
    let mean = values.iter().map(|&v| f64::from(v)).sum::<f64>() / n;
    let variance = values
        .iter()
        .map(|&v| (f64::from(v) - mean) * (f64::from(v) - mean))
        .sum::<f64>()
        / n;
    let mut histogram = [0u32; 16];
    for &v in values {
        // El campo vive en [0, ONE]: el histograma cubre ese rango entero.
        let bucket = u32::try_from(v).unwrap() * 16 / (ONE as u32 + 1);
        histogram[bucket.min(15) as usize] += 1;
    }
    (mean, variance, histogram)
}

/// Compara la distribución de una región lejana con la del origen.
fn compara_distribucion(origen: &[i32], lejos: &[i32], tol_media: f64, contexto: &str) {
    let (ma, va, ha) = stats(origen);
    let (mb, vb, hb) = stats(lejos);
    assert!(
        (ma - mb).abs() < tol_media,
        "{contexto}: medias distintas: origen {ma}, lejos {mb}"
    );
    assert!(
        (va - vb).abs() / va.max(1.0) < 0.2,
        "{contexto}: varianzas distintas: origen {va}, lejos {vb}"
    );
    for (bucket, (a, b)) in ha.iter().zip(&hb).enumerate() {
        let tolerancia = 512.max(a.max(b) / 3);
        assert!(
            a.abs_diff(*b) <= tolerancia,
            "{contexto}: histograma {bucket}: origen {a}, lejos {b}"
        );
    }
}

/// INV-W03: la media, la varianza y el histograma de `f1_sq` y `edge_q16`
/// en ±7 499 999 son estadísticamente iguales a los del origen.
///
/// Con jitter ONE/2 la distribución es continua (una muestra a media celda
/// del punto más cercano da ONE/4). Las regiones cubren 8 × 8 celdas del
/// período base en 2D y 6 × 6 × 6 en 3D con paso 2, suficientes para que la
/// comparación no dependa de unos pocos puntos característicos. La media
/// admite ~1024 porque con tan pocas celdas el ruido de muestreo entre
/// regiones llega a ±550 (medido); una deriva de precisión lejos del origen
/// la desplazaría mucho más.
#[test]
fn inv_w03_estadisticas_en_7_499_999() {
    let edge = 7_499_999i64;
    let jitter = (ONE as i32) / 2;
    let c2 = Cellular2::with_jitter(42, 4, jitter);
    let region2 = |bx: i64, bz: i64| {
        let mut f1 = Vec::with_capacity(128 * 128);
        let mut borde = Vec::with_capacity(128 * 128);
        for dz in 0..128i64 {
            for dx in 0..128i64 {
                let s = c2.sample_block(bx + dx, bz + dz);
                f1.push(s.f1_sq);
                borde.push(s.edge_q16);
            }
        }
        (f1, borde)
    };
    let (f1_origen, borde_origen) = region2(0, 0);
    let (f1_a, borde_a) = region2(edge, -edge);
    let (f1_b, borde_b) = region2(-edge, edge);
    for (f1_lejos, borde_lejos) in [(&f1_a, &borde_a), (&f1_b, &borde_b)] {
        compara_distribucion(&f1_origen, f1_lejos, 1024.0, "f1_sq 2D");
        compara_distribucion(&borde_origen, borde_lejos, 1024.0, "edge_q16 2D");
    }

    let c3 = Cellular3::with_jitter(42, 4, jitter);
    let region3 = |bx: i64, by: i64, bz: i64| {
        let mut f1 = Vec::with_capacity(48 * 48 * 48);
        let mut borde = Vec::with_capacity(48 * 48 * 48);
        for cz in 0..48i64 {
            for cy in 0..48i64 {
                for cx in 0..48i64 {
                    let s = c3.sample_block(bx + 2 * cx, by + 2 * cy, bz + 2 * cz);
                    f1.push(s.f1_sq);
                    borde.push(s.edge_q16);
                }
            }
        }
        (f1, borde)
    };
    let (f1_origen, borde_origen) = region3(0, 0, 0);
    let (f1_a, borde_a) = region3(edge, -edge, edge);
    let (f1_b, borde_b) = region3(-edge, edge, edge);
    for (f1_lejos, borde_lejos) in [(&f1_a, &borde_a), (&f1_b, &borde_b)] {
        compara_distribucion(&f1_origen, f1_lejos, 1024.0, "f1_sq 3D");
        compara_distribucion(&borde_origen, borde_lejos, 1024.0, "edge_q16 3D");
    }
}

/// Los extremos de `i64` no desbordan con ningún período ni jitter: en
/// debug, los chequeos de desbordamiento romperían la prueba.
#[test]
fn extremos_i64_sin_panico_ni_desborde() {
    let extremos = [
        i64::MIN,
        i64::MIN + 1,
        -(1i64 << 62),
        1i64 << 62,
        i64::MAX - 1,
        i64::MAX,
    ];
    for k in 1..=16u32 {
        for jitter in [0, ONE as i32] {
            let c2 = Cellular2::with_jitter(42, k, jitter);
            let c3 = Cellular3::with_jitter(42, k, jitter);
            for &e in &extremos {
                // wrapping: negar i64::MIN desbordaría en la propia prueba.
                let w = e.wrapping_neg();
                afirma_invariantes(c2.sample(e, e), "2D (e, e)");
                afirma_invariantes(c2.sample(e, w), "2D (e, -e)");
                afirma_invariantes(c3.sample(e, e, e), "3D (e, e, e)");
                afirma_invariantes(c3.sample(e, w, e), "3D (e, -e, e)");
            }
        }
    }
}

/// La misma llamada dos veces da el mismo resultado y dos semillas dan
/// hashes de celda distintos en casi todas las muestras.
#[test]
fn es_determinista_y_sensible_a_la_semilla() {
    let a2 = Cellular2::new(42, 4);
    let b2 = Cellular2::new(43, 4);
    let a3 = Cellular3::new(42, 4);
    let b3 = Cellular3::new(43, 4);
    let (mut distintas, mut total) = (0u32, 0u32);
    for i in 0..32i64 {
        for j in 0..32i64 {
            let (x, z) = (i * 13 - 200, j * 9 - 100);
            let s = a2.sample_block(x, z);
            assert_eq!(s, a2.sample_block(x, z), "2D no determinista");
            let t = a3.sample_block(x, z, i - j);
            assert_eq!(t, a3.sample_block(x, z, i - j), "3D no determinista");
            if s.cell_hash != b2.sample_block(x, z).cell_hash {
                distintas += 1;
            }
            total += 1;
            if t.cell_hash != b3.sample_block(x, z, i - j).cell_hash {
                distintas += 1;
            }
            total += 1;
        }
    }
    assert!(
        distintas > total * 9 / 10,
        "la semilla casi no cambia nada: {distintas}/{total}"
    );
}
