//! Planificador de streaming de chunks alrededor de los jugadores.
//!
//! Implementa la sección 4 de `docs/mundo/01-escala-y-motor.md`
//! («Streaming: cargar lo que se ve»): decide qué chunks entran y salen
//! con prioridades e histéresis. No consulta relojes ni azar: todo el
//! orden es total y determinista, así que el mismo estado produce
//! siempre el mismo plan.

use std::collections::BTreeSet;

use grisu_voxel::ChunkPos;

/// Fases por las que pasa un chunk hasta estar listo para dibujar.
/// El orden de la enum es el orden del cauce (PartialOrd derivado).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ChunkStage {
    /// Columna de bloques generada.
    Column,
    /// Base del terreno asentada.
    Base,
    /// Decorado aplicado (cuevas, menas, vegetación).
    Decorated,
    /// Luz propagada por el mundo.
    Lit,
    /// Malla construida y lista para dibujar.
    Meshed,
}

/// Chunks de margen extra antes de descargar (histéresis R + 2).
pub const HYSTERESIS: i32 = 2;

/// Presets de distancia de visión.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DistancePreset {
    Low,
    Medium,
    High,
}

impl DistancePreset {
    /// Radios `(radius_h, radius_v)` del preset: horizontal en X/Z y
    /// vertical en Y.
    pub fn radii(self) -> (i32, i32) {
        match self {
            DistancePreset::Low => (8, 5),
            DistancePreset::Medium => (12, 6),
            DistancePreset::High => (16, 8),
        }
    }
}

/// Jugador observado: chunk en el que está y dirección de avance entera.
struct Player {
    pos: ChunkPos,
    motion: [i32; 3],
}

/// Vector entero desde el jugador hasta la posición.
fn delta(pos: ChunkPos, origin: ChunkPos) -> (i32, i32, i32) {
    (pos.x - origin.x, pos.y - origin.y, pos.z - origin.z)
}

/// Distancia de Chebyshev combinada: `max(max(|dx|, |dz|), |dy|)`.
fn cheb((dx, dy, dz): (i32, i32, i32)) -> i32 {
    dx.abs().max(dz.abs()).max(dy.abs())
}

/// Métrica de desempate: `max(|dx|, |dz|) + |dy|`.
fn manh((dx, dy, dz): (i32, i32, i32)) -> i32 {
    dx.abs().max(dz.abs()) + dy.abs()
}

/// Producto escalar entero `(dx, dy, dz) · (mx, my, mz)`, con aritmética
/// saturada para no poder desbordar ante vectores de movimiento extremos.
fn align((dx, dy, dz): (i32, i32, i32), [mx, my, mz]: [i32; 3]) -> i32 {
    dx.saturating_mul(mx)
        .saturating_add(dy.saturating_mul(my))
        .saturating_add(dz.saturating_mul(mz))
}

/// Planificador de streaming alrededor de los jugadores.
///
/// Mantiene el conjunto de chunks activos y produce planes de carga y
/// descarga a partir de las posiciones de los jugadores. Quien ejecuta el
/// plan decide cuánto aplica y cuándo, y lo confirma con
/// [`StreamScheduler::commit`].
pub struct StreamScheduler {
    preset: DistancePreset,
    players: Vec<Player>,
    active: BTreeSet<ChunkPos>,
}

impl StreamScheduler {
    /// Crea un planificador con el preset dado, sin chunks activos ni
    /// jugadores.
    pub fn new(preset: DistancePreset) -> Self {
        Self {
            preset,
            players: Vec::new(),
            active: BTreeSet::new(),
        }
    }

    /// Cambia el preset de distancia de visión; afecta a los próximos
    /// planes.
    pub fn set_preset(&mut self, preset: DistancePreset) {
        self.preset = preset;
    }

    /// Reemplaza la lista de jugadores: posición de chunk y dirección de
    /// avance (vector entero; `[0; 3]` si está parado).
    pub fn update_players(&mut self, players: &[(ChunkPos, [i32; 3])]) {
        self.players = players
            .iter()
            .map(|(pos, motion)| Player {
                pos: *pos,
                motion: *motion,
            })
            .collect();
    }

    /// Conjunto deseado menos los activos, ordenado por prioridad de carga
    /// (ver [`StreamScheduler::load_key`]).
    pub fn plan_loads(&self) -> Vec<ChunkPos> {
        let mut candidates: Vec<ChunkPos> = self
            .desired()
            .into_iter()
            .filter(|pos| !self.active.contains(pos))
            .collect();
        candidates.sort_by_key(|pos| self.load_key(*pos));
        candidates
    }

    /// Chunks activos que deben descargarse, ordenados por cercanía al
    /// jugador más próximo (ver [`StreamScheduler::unload_key`]).
    ///
    /// Un chunk sale solo si está fuera de la caja con histéresis
    /// (`radius_h + HYSTERESIS`, `radius_v + HYSTERESIS`) de **todos** los
    /// jugadores. Sin jugadores el conjunto deseado está vacío y no hay
    /// caja que retenga nada: todo lo activo aparece aquí, en el orden del
    /// conjunto interno (determinista).
    pub fn plan_unloads(&self) -> Vec<ChunkPos> {
        if self.players.is_empty() {
            return self.active.iter().copied().collect();
        }
        let (radius_h, radius_v) = self.preset.radii();
        let limit_h = radius_h + HYSTERESIS;
        let limit_v = radius_v + HYSTERESIS;
        let mut leaving: Vec<ChunkPos> = self
            .active
            .iter()
            .copied()
            .filter(|pos| {
                self.players.iter().all(|player| {
                    let d = delta(*pos, player.pos);
                    d.0.abs().max(d.2.abs()) > limit_h || d.1.abs() > limit_v
                })
            })
            .collect();
        leaving.sort_by_key(|pos| self.unload_key(*pos));
        leaving
    }

    /// Aplica un plan: añade las cargas y retira las descargas. Es
    /// idempotente: ignora en silencio cargas de chunks ya activos y
    /// descargas de chunks que no lo están. Si una posición apareciera en
    /// ambos listados, gana la descarga (se aplica al final).
    pub fn commit(&mut self, loads: &[ChunkPos], unloads: &[ChunkPos]) {
        for pos in loads {
            self.active.insert(*pos);
        }
        for pos in unloads {
            self.active.remove(pos);
        }
    }

    /// Si el chunk está en el conjunto activo.
    pub fn is_active(&self, pos: ChunkPos) -> bool {
        self.active.contains(&pos)
    }

    /// Número de chunks activos.
    pub fn active_len(&self) -> usize {
        self.active.len()
    }

    /// Unión deduplicada de las cajas deseadas de todos los jugadores:
    /// `max(|x-px|, |z-pz|) <= radius_h` y `|y-py| <= radius_v`
    /// (una caja de 17×17×11 con `Low`).
    fn desired(&self) -> BTreeSet<ChunkPos> {
        let (radius_h, radius_v) = self.preset.radii();
        let mut set = BTreeSet::new();
        for player in &self.players {
            for dy in -radius_v..=radius_v {
                for dz in -radius_h..=radius_h {
                    for dx in -radius_h..=radius_h {
                        set.insert(player.pos.offset(dx, dy, dz));
                    }
                }
            }
        }
        set
    }

    /// Jugador más cercano a `pos` según `cheb`; con empate gana el
    /// primero del listado de `update_players`, para que la elección sea
    /// determinista. Devuelve `None` si no hay jugadores.
    fn closest(&self, pos: ChunkPos) -> Option<&Player> {
        let mut best: Option<(&Player, i32)> = None;
        for player in &self.players {
            let dist = cheb(delta(pos, player.pos));
            let better = match best {
                None => true,
                Some((_, current)) => dist < current,
            };
            if better {
                best = Some((player, dist));
            }
        }
        best.map(|(player, _)| player)
    }

    /// Clave de prioridad de carga de un chunk respecto al jugador más
    /// cercano (el que minimiza `cheb`; con empate, el primero del
    /// listado). Orden total determinista:
    ///
    /// 1. `cheb = max(max(|dx|, |dz|), |dy|)` ascendente: lo más cercano
    ///    primero.
    /// 2. `manh = max(|dx|, |dz|) + |dy|` ascendente: desempata hacia el
    ///    horizonte antes que hacia arriba y abajo.
    /// 3. Alineación con la dirección de avance, `align = dx*mx + dy*my +
    ///    dz*mz`, **descendente**: lo que está por delante del jugador se
    ///    carga antes. Con movimiento `[0; 3]`, `align` es siempre 0 y no
    ///    reordena nada.
    /// 4. Posición `(y, z, x)` ascendente como desempate final.
    ///
    /// El tercer componente invierte `align` (con `saturating_neg`) para
    /// meter el orden descendente dentro de una clave ascendente. Sin
    /// jugadores no hay candidatos que ordenar; la rama defensiva devuelve
    /// solo el desempate de posición.
    fn load_key(&self, pos: ChunkPos) -> (i32, i32, i32, i32, i32, i32) {
        match self.closest(pos) {
            Some(player) => {
                let d = delta(pos, player.pos);
                (
                    cheb(d),
                    manh(d),
                    align(d, player.motion).saturating_neg(),
                    pos.y,
                    pos.z,
                    pos.x,
                )
            }
            None => (0, 0, 0, pos.y, pos.z, pos.x),
        }
    }

    /// Clave de orden de `plan_unloads`: `(cheb, manh, y, z, x)` ascendente
    /// respecto al jugador más cercano (la misma elección determinista que
    /// [`StreamScheduler::load_key`], sin alineación: da igual por dónde se
    /// descargue). Solo se usa con jugadores; sin jugadores la salida va en
    /// el orden del conjunto interno.
    fn unload_key(&self, pos: ChunkPos) -> (i32, i32, i32, i32, i32) {
        match self.closest(pos) {
            Some(player) => {
                let d = delta(pos, player.pos);
                (cheb(d), manh(d), pos.y, pos.z, pos.x)
            }
            None => (0, 0, pos.y, pos.z, pos.x),
        }
    }
}
