//! Miniatura del terreno para el panel: mosaico de la superficie visto
//! desde arriba, coloreado con el tono medio de cada bloque y sombreado
//! por altura. Todo en CPU, sin GPU ni ventanas.

use crate::state::LauncherState;
use crate::theme::Color;
use grisu_render::TextureData;
use grisu_voxel::{AIR, VoxelWorld};
use hush_sim::{TerrainConfig, generate_terrain};

/// Celdas del mosaico dentro del panel de vista previa.
pub const PREVIEW_COLS: usize = 26;
pub const PREVIEW_ROWS: usize = 16;

/// Vista previa ya calculada para una semilla concreta.
#[derive(Clone, Debug, PartialEq)]
pub struct Preview {
    pub seed: u64,
    pub cols: usize,
    pub rows: usize,
    pub colors: Vec<Color>,
}

impl Preview {
    pub fn color(&self, col: usize, row: usize) -> Color {
        self.colors
            .get(row * self.cols + col)
            .copied()
            .unwrap_or([0.0; 4])
    }
}

/// Color medio de cada textura; el índice `i` corresponde al bloque `i + 1`.
pub fn block_colors(textures: &[TextureData]) -> Vec<Color> {
    textures
        .iter()
        .map(|tex| {
            if tex.rgba.is_empty() {
                return [0.2, 0.2, 0.2, 1.0];
            }
            let mut sum = [0u64; 3];
            let px = &tex.rgba;
            for i in (0..px.len()).step_by(4) {
                sum[0] += px[i] as u64;
                sum[1] += px[i + 1] as u64;
                sum[2] += px[i + 2] as u64;
            }
            let n = (tex.rgba.len() / 4) as f32;
            [
                sum[0] as f32 / (n * 255.0),
                sum[1] as f32 / (n * 255.0),
                sum[2] as f32 / (n * 255.0),
                1.0,
            ]
        })
        .collect()
}

/// Color del cielo para columnas sin bloque alguno.
const CIELO: Color = [0.09, 0.12, 0.17, 1.0];

/// Mosaico superior del mundo: una columna de muestra por celda, coloreada
/// con el tono del bloque superficial y sombreada por su altura relativa.
pub fn surface_mosaic(
    world: &VoxelWorld,
    block_colors: &[Color],
    cols: usize,
    rows: usize,
) -> Vec<Color> {
    let [sx, sy, sz] = world.size();
    let (cols, rows) = (cols.max(1), rows.max(1));
    let mut heights = vec![0i32; cols * rows];
    let mut cells = vec![CIELO; cols * rows];
    for row in 0..rows {
        for col in 0..cols {
            let x = ((col + 1) * sx as usize / (cols + 1)) as i32;
            let z = ((row + 1) * sz as usize / (rows + 1)) as i32;
            let mut top = None;
            for y in (0..sy as i32).rev() {
                let id = world.get(x, y, z);
                if id != AIR {
                    top = Some((y, id));
                    break;
                }
            }
            let slot = row * cols + col;
            match top {
                Some((y, id)) => {
                    heights[slot] = y;
                    // La textura `i` pertenece al bloque `i + 1`.
                    cells[slot] = if id >= 1 {
                        block_colors
                            .get(id as usize - 1)
                            .copied()
                            .unwrap_or([0.25, 0.25, 0.25, 1.0])
                    } else {
                        [0.25, 0.25, 0.25, 1.0]
                    };
                }
                None => heights[slot] = -1,
            }
        }
    }
    let solid: Vec<i32> = heights.iter().copied().filter(|h| *h >= 0).collect();
    if let (Some(&low), Some(&high)) = (solid.iter().min(), solid.iter().max()) {
        let span = (high - low).max(1) as f32;
        for (slot, cell) in cells.iter_mut().enumerate() {
            if heights[slot] >= 0 {
                let shade = 0.45 + 0.55 * (heights[slot] - low) as f32 / span;
                cell[0] *= shade;
                cell[1] *= shade;
                cell[2] *= shade;
            }
        }
    }
    cells
}

/// Regenera la vista previa del estado si la semilla elegida cambió.
/// Los fallos de generación dejan la vista previa anterior tal cual.
pub fn refresh(state: &mut LauncherState, textures: &[TextureData]) {
    let seed = state.current_seed();
    if state
        .preview
        .as_ref()
        .is_some_and(|p| p.seed == seed && p.cols == PREVIEW_COLS)
    {
        return;
    }
    let Ok(world) = generate_terrain(seed, &TerrainConfig::default()) else {
        return;
    };
    let colors = surface_mosaic(&world, &block_colors(textures), PREVIEW_COLS, PREVIEW_ROWS);
    state.preview = Some(Preview {
        seed,
        cols: PREVIEW_COLS,
        rows: PREVIEW_ROWS,
        colors,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_textures() -> Vec<TextureData> {
        vec![
            TextureData {
                width: 1,
                height: 1,
                rgba: vec![255, 0, 0, 255],
            },
            TextureData {
                width: 1,
                height: 1,
                rgba: vec![0, 255, 0, 255],
            },
        ]
    }

    #[test]
    fn block_color_is_the_texture_average() {
        let colors = block_colors(&sample_textures());
        assert_eq!(colors.len(), 2);
        assert_eq!(colors[0], [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(colors[1], [0.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn mosaic_maps_surface_blocks_with_height_shading() {
        let mut world = VoxelWorld::new([16, 16, 16]).expect("mundo pequeño");
        world.fill([0, 0, 0], [15, 2, 15], 1); // bloque 1 (textura roja)
        world.fill([0, 3, 0], [7, 3, 15], 2); // mitad izquierda alta, verde
        let colors = block_colors(&sample_textures());
        let cells = surface_mosaic(&world, &colors, 4, 4);
        assert_eq!(cells.len(), 16);
        // Todas las celdas tienen bloque: nada de cielo.
        assert!(cells.iter().all(|c| *c != CIELO));
        // Columnas de la izquierda (x <= 3) verdes; derechas, rojas.
        for row in 0..4 {
            for col in 0..4 {
                let cell = cells[row * 4 + col];
                if col < 2 {
                    assert!(cell[1] > cell[0], "col {col} deberia ser verde");
                } else {
                    assert!(cell[0] > cell[1], "col {col} deberia ser roja");
                }
            }
        }
        // El sombreado por altura deja ver el tono, no lo apaga.
        assert!(cells[0][1] >= 0.45 && cells[3][0] >= 0.45);
    }

    #[test]
    fn refresh_updates_when_the_seed_changes() {
        let mut state =
            LauncherState::new(crate::testutil::sample_versions_for_tests(), Vec::new(), 0);
        let textures = sample_textures();
        refresh(&mut state, &textures);
        let first = state.preview.clone().expect("vista previa inicial");
        assert_eq!(first.seed, state.current_seed());
        refresh(&mut state, &textures);
        assert_eq!(state.preview.as_ref(), Some(&first));
        state.seed_idx = 1;
        refresh(&mut state, &textures);
        let renewed = state.preview.clone().expect("vista previa nueva");
        assert_eq!(renewed.seed, state.current_seed());
    }
}
