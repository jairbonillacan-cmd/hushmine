//! Luz por bloques: luz de cielo y bloques emisores, propagada por BFS.
//!
//! Es presentación: solo la usa el cliente para las mallas. Cada celda de
//! aire guarda un nivel de 0 a [`MAX_LIGHT`]. La luz del cielo baja sin
//! perder intensidad hasta el primer bloque sólido; después, y desde los
//! emisores, pierde un nivel por bloque.

use std::collections::VecDeque;

use crate::{AIR, BlockId, VoxelWorld};

pub const MAX_LIGHT: u8 = 15;

/// Niveles de luz de todo el mundo.
#[derive(Clone, Debug)]
pub struct LightMap {
    size: [u32; 3],
    levels: Vec<u8>,
    /// Luz que emite cada tipo de bloque (0 = ninguna).
    emission: [u8; 256],
}

impl LightMap {
    /// Calcula la luz de `world`. `emission[b]` es la luz del bloque `b`.
    pub fn new(world: &VoxelWorld, emission: [u8; 256]) -> Self {
        let [sx, sy, sz] = world.size();
        let mut map = Self {
            size: world.size(),
            levels: vec![0; (sx * sy * sz) as usize],
            emission: emission.map(|e| e.min(MAX_LIGHT)),
        };
        map.levels = map.compute(world);
        map
    }

    /// Nivel de luz de una celda; fuera del mundo, luz de cielo completa.
    #[inline]
    pub fn get(&self, x: i32, y: i32, z: i32) -> u8 {
        match self.index(x, y, z) {
            Some(i) => self.levels[i],
            None => MAX_LIGHT,
        }
    }

    /// Recalcula la luz tras editar el mundo y marca como sucios los chunks
    /// cuya iluminación cambió. Devuelve cuántas celdas cambiaron.
    pub fn update(&mut self, world: &mut VoxelWorld) -> usize {
        if world.size() != self.size {
            *self = Self::new(world, self.emission);
            world.mark_all_dirty();
            return self.levels.len();
        }
        let fresh = self.compute(world);
        let [sx, _, sz] = self.size.map(|s| s as usize);
        let mut changed = 0;
        for (i, (old, new)) in self.levels.iter().zip(&fresh).enumerate() {
            if old != new {
                changed += 1;
                let (x, z, y) = (i % sx, (i / sx) % sz, i / (sx * sz));
                world.mark_dirty_around(x as i32, y as i32, z as i32);
            }
        }
        self.levels = fresh;
        changed
    }

    fn index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        let [sx, sy, sz] = self.size;
        let inside =
            x >= 0 && y >= 0 && z >= 0 && (x as u32) < sx && (y as u32) < sy && (z as u32) < sz;
        inside.then(|| (x as u32 + z as u32 * sx + y as u32 * sx * sz) as usize)
    }

    fn compute(&self, world: &VoxelWorld) -> Vec<u8> {
        let [sx, sy, sz] = self.size.map(|s| s as i32);
        let mut levels = vec![0u8; self.levels.len()];
        let mut queue = VecDeque::new();

        // Cielo: columnas de aire desde arriba hasta el primer sólido.
        for z in 0..sz {
            for x in 0..sx {
                for y in (0..sy).rev() {
                    if world.is_solid(x, y, z) {
                        break;
                    }
                    let i = self.index(x, y, z).unwrap_or(0);
                    levels[i] = MAX_LIGHT;
                    queue.push_back([x, y, z]);
                }
            }
        }
        // Emisores: su luz se guarda en la propia celda sólida y desde ahí se
        // propaga al aire vecino.
        for y in 0..sy {
            for z in 0..sz {
                for x in 0..sx {
                    let e = self.emission[usize::from(world.get(x, y, z))];
                    if e > 0 {
                        let i = self.index(x, y, z).unwrap_or(0);
                        levels[i] = levels[i].max(e);
                        queue.push_back([x, y, z]);
                    }
                }
            }
        }

        const NEIGHBOURS: [[i32; 3]; 6] = [
            [1, 0, 0],
            [-1, 0, 0],
            [0, 1, 0],
            [0, -1, 0],
            [0, 0, 1],
            [0, 0, -1],
        ];
        while let Some(p) = queue.pop_front() {
            let level = self.index(p[0], p[1], p[2]).map_or(0, |i| levels[i]);
            if level <= 1 {
                continue;
            }
            for d in NEIGHBOURS {
                let q = [p[0] + d[0], p[1] + d[1], p[2] + d[2]];
                let Some(i) = self.index(q[0], q[1], q[2]) else {
                    continue;
                };
                if world.get(q[0], q[1], q[2]) == AIR && levels[i] < level - 1 {
                    levels[i] = level - 1;
                    queue.push_back(q);
                }
            }
        }
        levels
    }
}

/// Tabla de emisión vacía con un único bloque emisor (para pruebas y
/// configuraciones sencillas).
pub fn emission_of(block: BlockId, level: u8) -> [u8; 256] {
    let mut table = [0; 256];
    table[usize::from(block)] = level;
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bloque de roca de 32×16×32 con una galería cerrada en y = 4..6.
    fn closed_gallery() -> VoxelWorld {
        let mut w = VoxelWorld::new([32, 16, 32]).unwrap();
        w.fill([0, 0, 0], [31, 15, 31], 1);
        w.fill([2, 4, 2], [29, 6, 29], AIR);
        w
    }

    #[test]
    fn open_sky_is_fully_lit_and_closed_caves_are_dark() {
        let mut w = closed_gallery();
        w.fill([0, 10, 0], [31, 15, 31], AIR);
        let light = LightMap::new(&w, [0; 256]);
        assert_eq!(light.get(5, 12, 5), MAX_LIGHT);
        assert_eq!(light.get(5, 5, 5), 0);
        assert_eq!(light.get(-1, 5, 5), MAX_LIGHT);
    }

    #[test]
    fn emitters_fade_one_level_per_block() {
        let mut w = closed_gallery();
        w.set(10, 4, 10, 7);
        let light = LightMap::new(&w, emission_of(7, 12));
        assert_eq!(light.get(10, 5, 10), 11);
        assert_eq!(light.get(13, 5, 10), 8);
        assert_eq!(light.get(25, 5, 10), 0);
    }

    #[test]
    fn digging_a_shaft_lets_light_in_and_marks_chunks() {
        let mut w = closed_gallery();
        let mut light = LightMap::new(&w, [0; 256]);
        assert_eq!(light.get(16, 5, 16), 0);
        w.take_dirty();
        w.fill([16, 7, 16], [16, 15, 16], AIR);
        w.take_dirty();
        let changed = light.update(&mut w);
        assert!(changed > 10);
        assert_eq!(light.get(16, 4, 16), MAX_LIGHT);
        assert_eq!(light.get(18, 5, 16), MAX_LIGHT - 2);
        assert!(!w.take_dirty().is_empty());
    }
}
