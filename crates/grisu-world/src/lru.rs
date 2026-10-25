//! Caché LRU determinista de chunks en estado `Base`.
//!
//! Sección 4 de `docs/mundo/01-escala-y-motor.md`: mantener alrededor del
//! borde cargado los chunks ya generados en estado `Base`, para que la fase
//! de decoración no repita trabajo.

use std::collections::BTreeMap;

use grisu_voxel::ChunkPos;

/// Caché LRU determinista de chunks en estado Base.
///
/// Usa un reloj lógico monótono (`u64`) en vez del reloj del sistema y un
/// `BTreeMap` en vez de `HashMap`, para que las expulsiones sean iguales en
/// todas las plataformas y ejecuciones.
///
/// # Orden de expulsión
///
/// Se expulsa la entrada con la marca de uso más antigua. Entre marcas iguales
/// gana la primera en el orden de `ChunkPos`: la iteración de un `BTreeMap`
/// sigue el orden de las claves y `Iterator::min_by_key` conserva la primera
/// de las iguales, así que el desempate también es determinista. Con el reloj
/// en `u64` las marcas solo podrían empatar si el contador desbordara algún
/// día; el desempate cubre ese caso.
pub struct BaseChunkCache<V> {
    /// Capacidad máxima de entradas.
    capacity: usize,
    /// Reloj lógico: marca de uso de la última operación `get`/`insert`.
    clock: u64,
    /// Entradas por posición; el valor es `(marca de uso, valor)`.
    entries: BTreeMap<ChunkPos, (u64, V)>,
}

impl<V> BaseChunkCache<V> {
    /// Crea una caché con capacidad máxima de `capacity` entradas.
    ///
    /// Con `capacity == 0` no guarda nada: cada `insert` devuelve su propio
    /// par `(posición, valor)`.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            clock: 0,
            entries: BTreeMap::new(),
        }
    }

    /// Devuelve el valor de `pos` y refresca su marca de uso.
    ///
    /// El préstamo mutable sirve para adelantar el reloj lógico; en ningún
    /// caso se consulta el reloj del sistema.
    pub fn get(&mut self, pos: ChunkPos) -> Option<&V> {
        let (mark, val) = self.entries.get_mut(&pos)?;
        self.clock = self.clock.wrapping_add(1);
        *mark = self.clock;
        Some(&*val)
    }

    /// Inserta `val` en `pos`, o reemplaza el valor si la posición ya existe.
    ///
    /// - Si `pos` ya existe, refresca su marca de uso y no expulsa a nadie
    ///   (devuelve `None`).
    /// - Si al insertar una posición nueva se supera la capacidad, expulsa la
    ///   entrada con la marca de uso más antigua (ver el desempate en la
    ///   documentación del tipo) y la devuelve como `(posición, valor)`.
    /// - Con capacidad 0 no guarda nada y devuelve `Some((pos, val))`.
    ///
    /// La entrada recién insertada nunca es la expulsada: primero se hace
    /// sitio (solo si la caché está llena y la clave no existe) y después se
    /// inserta con la marca de uso más reciente.
    pub fn insert(&mut self, pos: ChunkPos, val: V) -> Option<(ChunkPos, V)> {
        if self.capacity == 0 {
            return Some((pos, val));
        }
        if let Some((mark, slot)) = self.entries.get_mut(&pos) {
            self.clock = self.clock.wrapping_add(1);
            *mark = self.clock;
            *slot = val;
            return None;
        }
        let evicted = if self.entries.len() == self.capacity {
            self.evict_oldest()
        } else {
            None
        };
        self.clock = self.clock.wrapping_add(1);
        self.entries.insert(pos, (self.clock, val));
        evicted
    }

    /// Saca `pos` de la caché y devuelve su valor, si estaba guardado.
    pub fn remove(&mut self, pos: ChunkPos) -> Option<V> {
        self.entries.remove(&pos).map(|(_, val)| val)
    }

    /// Número de entradas guardadas.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` si no hay ninguna entrada guardada.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Capacidad máxima con la que se creó la caché.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Expulsa la entrada con la marca de uso más antigua.
    ///
    /// Entre marcas iguales gana la primera en el orden de `ChunkPos` (la
    /// iteración del `BTreeMap` va en ese orden y `min_by_key` conserva la
    /// primera de las iguales).
    fn evict_oldest(&mut self) -> Option<(ChunkPos, V)> {
        let oldest = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.0)
            .map(|(pos, _)| *pos)?;
        self.entries.remove(&oldest).map(|(_, val)| (oldest, val))
    }
}
