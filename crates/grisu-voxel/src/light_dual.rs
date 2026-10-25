//! Luz dual: cielo y bloques emisores en dos canales independientes.
//!
//! Cada celda guarda dos niveles de 0 a [`MAX_LIGHT`]: el de cielo (baja sin
//! perder intensidad por las columnas abiertas y luego se difunde en
//! horizontal perdiendo 1 por bloque) y el de bloque (solo emisores, que
//! pierde 1 por bloque). Separarlos permite modular el canal de cielo con el
//! ciclo diurno sin tocar la luz de las antorchas.

use std::collections::VecDeque;

use crate::{AIR, MAX_LIGHT, VoxelWorld};

/// Niveles de luz de cielo y de bloque de todo el mundo.
#[derive(Clone, Debug)]
pub struct DualLightMap {
    size: [u32; 3],
    sky: Vec<u8>,
    block: Vec<u8>,
    /// Luz que emite cada tipo de bloque (0 = ninguna).
    emission: [u8; 256],
}

impl DualLightMap {
    /// Calcula la luz dual de `world`. `emission[b]` es la luz del bloque `b`.
    pub fn new(world: &VoxelWorld, emission: [u8; 256]) -> Self {
        let volume = world.size().iter().product::<u32>() as usize;
        let mut map = Self {
            size: world.size(),
            sky: vec![0; volume],
            block: vec![0; volume],
            emission: emission.map(|e| e.min(MAX_LIGHT)),
        };
        map.recompute(world);
        map
    }

    /// Luz de cielo de una celda; fuera del mundo, cielo abierto.
    pub fn sky(&self, x: i32, y: i32, z: i32) -> u8 {
        match self.index(x, y, z) {
            Some(i) => self.sky[i],
            None => MAX_LIGHT,
        }
    }

    /// Luz de bloque (emisores) de una celda; fuera del mundo, 0.
    pub fn block(&self, x: i32, y: i32, z: i32) -> u8 {
        match self.index(x, y, z) {
            Some(i) => self.block[i],
            None => 0,
        }
    }

    /// Nivel combinado: el máximo de ambos canales.
    pub fn combined(&self, x: i32, y: i32, z: i32) -> u8 {
        self.sky(x, y, z).max(self.block(x, y, z))
    }

    /// Recalcula ambos canales tras editar el mundo y marca como sucios los
    /// chunks cuya luz cambió. Devuelve cuántas celdas cambiaron.
    pub fn update(&mut self, world: &mut VoxelWorld) -> usize {
        if world.size() != self.size {
            *self = Self::new(world, self.emission);
            world.mark_all_dirty();
            return self.sky.len();
        }
        let fresh_sky = self.compute_sky(world);
        let fresh_block = self.compute_block(world);
        let mut changed = 0;
        let [sx, _, sz] = self.size.map(|s| s as usize);
        for i in 0..self.sky.len() {
            let changed_here = fresh_sky[i] != self.sky[i] || fresh_block[i] != self.block[i];
            if changed_here {
                changed += 1;
                let (x, z, y) = (i % sx, (i / sx) % sz, i / (sx * sz));
                world.mark_dirty_around(x as i32, y as i32, z as i32);
            }
        }
        self.sky = fresh_sky;
        self.block = fresh_block;
        changed
    }

    /// Tabla de emisión con la que se calculó el mapa.
    pub fn emission(&self) -> [u8; 256] {
        self.emission
    }

    fn index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        let [sx, sy, sz] = self.size;
        let inside =
            x >= 0 && y >= 0 && z >= 0 && (x as u32) < sx && (y as u32) < sy && (z as u32) < sz;
        inside.then(|| (x as u32 + z as u32 * sx + y as u32 * sx * sz) as usize)
    }

    fn recompute(&mut self, world: &VoxelWorld) {
        self.sky = self.compute_sky(world);
        self.block = self.compute_block(world);
    }

    /// Canal de cielo: inyección cenital (columnas de aire abiertas a 15) y
    /// difusión lateral perdiendo un nivel por vóxel.
    fn compute_sky(&self, world: &VoxelWorld) -> Vec<u8> {
        let [sx, sy, sz] = self.size.map(|s| s as i32);
        let mut levels = vec![0u8; self.sky.len()];
        let mut queue = VecDeque::new();
        for z in 0..sz {
            for x in 0..sx {
                for y in (0..sy).rev() {
                    if world.is_solid(x, y, z) {
                        break;
                    }
                    if let Some(i) = self.index(x, y, z) {
                        levels[i] = MAX_LIGHT;
                        queue.push_back([x, y, z]);
                    }
                }
            }
        }
        self.spread(world, &mut levels, queue);
        levels
    }

    /// Canal de bloque: los emisores guardan su nivel en la propia celda y
    /// desde ahí se propaga al aire vecino.
    fn compute_block(&self, world: &VoxelWorld) -> Vec<u8> {
        let [sx, sy, sz] = self.size.map(|s| s as i32);
        let mut levels = vec![0u8; self.block.len()];
        let mut queue = VecDeque::new();
        for y in 0..sy {
            for z in 0..sz {
                for x in 0..sx {
                    let e = self.emission[usize::from(world.get(x, y, z))];
                    if e > 0
                        && let Some(i) = self.index(x, y, z)
                    {
                        levels[i] = e;
                        queue.push_back([x, y, z]);
                    }
                }
            }
        }
        self.spread(world, &mut levels, queue);
        levels
    }

    /// BFS común: cada paso resta un nivel y no atraviesa bloques sólidos
    /// (solo llena celdas de aire).
    fn spread(&self, world: &VoxelWorld, levels: &mut [u8], mut queue: VecDeque<[i32; 3]>) {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emission_of;

    /// Roca maciza de 32×16×32 con una galería cerrada en y = 4..6 y el
    /// cielo abierto a partir de y = 10.
    fn gallery() -> VoxelWorld {
        let mut w = VoxelWorld::new([32, 16, 32]).unwrap();
        w.fill([0, 0, 0], [31, 9, 31], 1);
        w.fill([2, 4, 2], [29, 6, 29], AIR);
        w.fill([0, 10, 0], [31, 15, 31], AIR);
        w
    }

    #[test]
    fn sky_fades_horizontally_under_overhang() {
        let w = gallery();
        let light = DualLightMap::new(&w, [0; 256]);
        // Cielo abierto: 15. Galería junto al hueco: menos.
        assert_eq!(light.sky(5, 12, 5), MAX_LIGHT);
        // La celda (5, 6, 5) está justo bajo el techo de roca (y=9): su luz
        // de cielo viene de la difusión lateral desde la columna abierta.
        assert_eq!(light.sky(5, 6, 5), 0);
        // Fuera del mundo: cielo completo.
        assert_eq!(light.sky(-1, 5, 5), MAX_LIGHT);
        assert_eq!(light.combined(-1, 5, 5), MAX_LIGHT);
    }

    #[test]
    fn emitters_only_light_the_block_channel() {
        let mut w = gallery();
        w.set(10, 4, 10, 7);
        let light = DualLightMap::new(&w, emission_of(7, 14));
        assert_eq!(light.block(10, 4, 10), 14);
        assert_eq!(light.block(13, 4, 10), 11);
        assert_eq!(light.sky(10, 4, 10), 0);
        assert_eq!(light.combined(13, 4, 10), 11);
    }

    #[test]
    fn placing_a_torch_marks_chunks_and_removing_extinguishes() {
        let mut w = gallery();
        let mut light = DualLightMap::new(&w, emission_of(7, 14));
        w.take_dirty();
        w.set(16, 5, 16, 7);
        let changed = light.update(&mut w);
        assert!(changed > 10, "cambiaron {changed} celdas");
        assert_eq!(light.block(19, 5, 16), 11);
        assert!(!w.take_dirty().is_empty());
        w.set(16, 5, 16, AIR);
        assert_eq!(light.update(&mut w), changed);
        assert_eq!(light.block(19, 5, 16), 0);
    }

    #[test]
    fn sky_channel_spreads_underground_through_open_shaft() {
        let mut w = gallery();
        // Pozo vertical abierto desde la superficie hasta la galería.
        w.fill([20, 4, 20], [20, 9, 20], AIR);
        let light = DualLightMap::new(&w, [0; 256]);
        assert_eq!(light.sky(20, 5, 20), MAX_LIGHT);
        // Un bloque a la redonda: 14.
        assert_eq!(light.sky(21, 5, 20), MAX_LIGHT - 1);
        // A seis bloques del pozo, la luz de cielo baja un nivel por paso.
        assert_eq!(light.sky(26, 5, 20), MAX_LIGHT - 6);
    }
}
