//! Biomas de superficie por cercanía al centroide.
//!
//! La elección es una función pura de los tres parámetros interpolados de la
//! columna (temperatura, humedad y continentalidad en Q16): gana el bioma
//! con el centroide más cercano por distancia al cuadrado y, en caso de
//! empate exacto, el de índice menor.

use grisu_noise::ONE;

use super::SurfaceBiome;

/// Centroides de los biomas: (temperatura, humedad, continentalidad) en Q16.
/// El orden es el de los discriminantes de [`SurfaceBiome`].
// Se moverá a data/worldgen/biomas.ron
pub(super) const BIOME_CENTROIDS: [(i64, i64, i64); 8] = [
    ((-60 * ONE) / 100, (-20 * ONE) / 100, (75 * ONE) / 100), // Cordillera
    ((-40 * ONE) / 100, (60 * ONE) / 100, (30 * ONE) / 100),  // NeedleForest
    ((-30 * ONE) / 100, (-50 * ONE) / 100, (60 * ONE) / 100), // Highland
    ((70 * ONE) / 100, (-70 * ONE) / 100, (20 * ONE) / 100),  // RedDesert
    ((60 * ONE) / 100, (-40 * ONE) / 100, (40 * ONE) / 100),  // Badlands
    ((20 * ONE) / 100, (80 * ONE) / 100, (-30 * ONE) / 100),  // RiverValley
    ((-90 * ONE) / 100, (20 * ONE) / 100, (10 * ONE) / 100),  // Tundra
    (0, 0, (-90 * ONE) / 100),                                // InlandSea
];

/// Bioma de superficie para los parámetros interpolados de una columna.
///
/// Distancia al cuadrado `Σ (p − c)²` en `i64`: los deltas caben de sobra
/// (como mucho `2 · ONE` por eje, tres ejes). El mejor solo se actualiza
/// con `<`, así que el empate exacto lo gana el índice menor.
pub(super) fn select_biome(temperature: i64, humidity: i64, continentality: i64) -> SurfaceBiome {
    let mut best = 0;
    let mut best_dist = i64::MAX;
    for (i, &(ct, ch, cc)) in BIOME_CENTROIDS.iter().enumerate() {
        let dt = temperature - ct;
        let dh = humidity - ch;
        let dc = continentality - cc;
        let dist = dt * dt + dh * dh + dc * dc;
        if dist < best_dist {
            best_dist = dist;
            best = i;
        }
    }
    match best {
        0 => SurfaceBiome::Cordillera,
        1 => SurfaceBiome::NeedleForest,
        2 => SurfaceBiome::Highland,
        3 => SurfaceBiome::RedDesert,
        4 => SurfaceBiome::Badlands,
        5 => SurfaceBiome::RiverValley,
        6 => SurfaceBiome::Tundra,
        _ => SurfaceBiome::InlandSea,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Los ocho biomas en el orden de la tabla.
    const BIOMAS: [SurfaceBiome; 8] = [
        SurfaceBiome::Cordillera,
        SurfaceBiome::NeedleForest,
        SurfaceBiome::Highland,
        SurfaceBiome::RedDesert,
        SurfaceBiome::Badlands,
        SurfaceBiome::RiverValley,
        SurfaceBiome::Tundra,
        SurfaceBiome::InlandSea,
    ];

    #[test]
    fn table_has_eight_entries_aligned_with_discriminants() {
        assert_eq!(BIOME_CENTROIDS.len(), 8);
        for (i, bioma) in BIOMAS.iter().enumerate() {
            assert_eq!(*bioma as u8, i as u8, "discriminante de {bioma:?}");
        }
    }

    #[test]
    fn selection_is_deterministic_and_each_centroid_wins_at_its_point() {
        for (i, &(t, h, c)) in BIOME_CENTROIDS.iter().enumerate() {
            assert_eq!(select_biome(t, h, c), BIOMAS[i], "centroid {i}");
            assert_eq!(
                select_biome(t, h, c),
                select_biome(t, h, c),
                "no es determinista en {i}"
            );
        }
    }
}
