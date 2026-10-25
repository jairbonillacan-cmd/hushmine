//! Bestiario hostil del subsuelo y motor de IA determinista.
//!
//! Los enemigos surgen donde la luz es baja (≤ 7) a 24-64 bloques del
//! jugador y se eliminan a más de 128. El Bregador localiza por el sonido
//! de pasos y picos; el Acechador silba 1.5 s antes de deflagrar; el
//! Artillero dispara a distancia y mantiene el retroceso; la Alimaña trepa
//! muros y emerge en enjambre al picar grava profunda.

use grisu_voxel::{FIXED_ONE, VoxelWorld};
use serde::{Deserialize, Serialize};

use super::path;

/// Ticks de simulación del modo historia.
pub const STORY_TICK_RATE: u32 = 20;

/// Luz máxima de la celda para que genere un enemigo.
pub const SPAWN_MAX_LIGHT: u8 = 7;
/// Distancia mínima y máxima de aparición al jugador (bloques).
pub const SPAWN_MIN_DISTANCE: i32 = 24;
pub const SPAWN_MAX_DISTANCE: i32 = 64;
/// Más allá de esta distancia el enemigo se elimina.
pub const DESPAWN_DISTANCE: i32 = 128;
/// Intentos de generación por intervalo.
pub const SPAWN_ATTEMPTS: u32 = 6;
/// Intervalo entre tandas de generación.
pub const SPAWN_INTERVAL_TICKS: u32 = 100;
/// Máximo de enemigos simultáneos.
pub const MAX_MOBS: usize = 12;
/// Radio en el que el Bregador oye ruido.
pub const HEARING_RADIUS: i32 = 32;
/// Distancia a la que el Acechador enciende la mecha.
pub const FUSE_DISTANCE: i32 = 3;
/// Mecha del Acechador: 1.5 s.
pub const FUSE_TICKS: u32 = 30;
/// Radio de la deflagración del Acechador.
pub const BLAST_RADIUS: i32 = 2;

/// Tipología de enemigo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobKind {
    /// Humanoide ciego y nudoso que ataca cuerpo a cuerpo.
    TunnelBrawler,
    /// Criatura silenciosa que deflagra junto al jugador.
    DustStalker,
    /// Restos esqueléticos que disparan pedernal a distancia.
    FossilGunner,
    /// Insecto blindado que trepa muros.
    CreviceVermin,
}

impl MobKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::TunnelBrawler => "BREGADOR DEL TUNEL",
            Self::DustStalker => "ACECHADOR DEL POLVO",
            Self::FossilGunner => "ARTILLERO FOSIL",
            Self::CreviceVermin => "ALIMANA DE LA GRIETA",
        }
    }

    /// Puntos de vida.
    pub fn health(self) -> u8 {
        match self {
            Self::TunnelBrawler => 14,
            Self::DustStalker => 8,
            Self::FossilGunner => 10,
            Self::CreviceVermin => 4,
        }
    }

    /// Velocidad en bloques/tick (punto fijo).
    pub fn speed(self) -> i32 {
        let blocks_per_s = match self {
            Self::TunnelBrawler => 3,
            Self::DustStalker => 4,
            Self::FossilGunner => 2,
            Self::CreviceVermin => 3,
        };
        blocks_per_s * FIXED_ONE / STORY_TICK_RATE as i32
    }

    /// Daño de contacto por golpe.
    pub fn melee_damage(self) -> u8 {
        match self {
            Self::TunnelBrawler => 5,
            Self::DustStalker => 2,
            Self::FossilGunner => 2,
            Self::CreviceVermin => 1,
        }
    }
}

/// Estado interno de la máquina de estados.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum MobState {
    /// Patrulla sin rumbo.
    Wander,
    /// Va hacia una posición oída o vista.
    Seek,
    /// Persigue al jugador.
    Chase,
    /// Acechador: mecha encendida.
    Fuse(u32),
    /// Artillero: mantiene la distancia y dispara.
    KeepDistance,
}

/// Un enemigo vivo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mob {
    pub kind: MobKind,
    /// Celda actual (bloques).
    pub cell: [i32; 3],
    pub health: u8,
    state: MobState,
    /// Objetivo temporal (posición oída).
    target: [i32; 3],
    /// Enfriamiento de ataque.
    cooldown: u8,
}

/// Proyectil de pedernal del Artillero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Projectile {
    /// Posición en punto fijo.
    pub pos: [i32; 3],
    /// Velocidad en punto fijo por tick.
    pub vel: [i32; 3],
    /// Ticks de vida restantes.
    pub ttl: u32,
}

/// Evento generado por el motor de IA en un tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MobEvent {
    Spawned(MobKind),
    Died(MobKind),
    /// El Acechador silba antes de deflagrar.
    Hiss([i32; 3]),
    /// Deflagración en `cell` con radio `BLAST_RADIUS`.
    Explosion([i32; 3]),
    /// Golpe de cuerpo a cuerpo al jugador.
    HitPlayer(u8),
    /// Disparo del Artillero.
    ShotFired([i32; 3]),
    /// El proyectil alcanzó al jugador.
    ProjectileHit(u8),
}

/// Sistema de enemigos de la expedición.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mobs {
    pub list: Vec<Mob>,
    pub projectiles: Vec<Projectile>,
    spawn_timer: u32,
    /// Intervalo actual entre tandas (el asedio lo acorta).
    spawn_interval: u32,
}

/// Contexto que la simulación pasa al sistema cada tick.
pub struct TickContext<'a> {
    pub world: &'a VoxelWorld,
    /// Posición de los pies del jugador (bloques).
    pub player_cell: [i32; 3],
    /// Último ruido oído (posición en bloques y tick).
    pub noise: Option<([i32; 3], u32)>,
    /// Luz efectiva (cielo modulado + bloque) por celda.
    pub light: &'a dyn Fn(i32, i32, i32) -> u8,
    /// Generador determinista de la simulación.
    pub rng: &'a mut crate::Rng,
}

impl Default for Mobs {
    fn default() -> Self {
        Self {
            list: Vec::new(),
            projectiles: Vec::new(),
            spawn_timer: 0,
            spawn_interval: SPAWN_INTERVAL_TICKS,
        }
    }
}

impl Mobs {
    /// Acorta el intervalo de generación (oleadas de asedio).
    pub fn accelerate_spawns(&mut self, factor: u32) {
        self.spawn_interval = (SPAWN_INTERVAL_TICKS / factor.max(1)).max(10);
    }

    /// Intenta generar enemigos según las reglas de luz y distancia.
    pub fn spawn_pass(&mut self, ctx: &mut TickContext) -> Vec<MobEvent> {
        let mut events = Vec::new();
        self.spawn_timer = self.spawn_timer.saturating_add(1);
        if self.spawn_timer < self.spawn_interval || self.list.len() >= MAX_MOBS {
            return events;
        }
        self.spawn_timer = 0;
        let [sx, sy, sz] = ctx.world.size().map(|s| s as i32);
        for _ in 0..SPAWN_ATTEMPTS {
            if self.list.len() >= MAX_MOBS {
                break;
            }
            let x = ctx.rng.below(sx as u32) as i32;
            let y = ctx.rng.below(sy as u32) as i32;
            let z = ctx.rng.below(sz as u32) as i32;
            if !path::walkable(ctx.world, x, y, z) {
                continue;
            }
            let dist = manhattan([x, y, z], ctx.player_cell);
            if !(SPAWN_MIN_DISTANCE..=SPAWN_MAX_DISTANCE).contains(&dist) {
                continue;
            }
            let light = (ctx.light)(x, y, z);
            if light > SPAWN_MAX_LIGHT {
                continue;
            }
            let kind = match ctx.rng.below(4) {
                0 => MobKind::TunnelBrawler,
                1 => MobKind::DustStalker,
                2 => MobKind::FossilGunner,
                _ => MobKind::CreviceVermin,
            };
            self.list.push(Mob {
                kind,
                cell: [x, y, z],
                health: kind.health(),
                state: MobState::Wander,
                target: [x, y, z],
                cooldown: 0,
            });
            events.push(MobEvent::Spawned(kind));
        }
        events
    }

    /// Genera un enjambre de alimañas (al picar un nido de roca falsa).
    pub fn spawn_swarm(&mut self, cell: [i32; 3], count: u8) -> MobEvent {
        for _ in 0..count {
            self.list.push(Mob {
                kind: MobKind::CreviceVermin,
                cell,
                health: MobKind::CreviceVermin.health(),
                state: MobState::Chase,
                target: cell,
                cooldown: 0,
            });
        }
        MobEvent::Spawned(MobKind::CreviceVermin)
    }

    /// Avanza la IA un tick. Devuelve los eventos y las celdas a destruir
    /// por deflagración (la simulación aplica los cambios al mundo).
    pub fn step(&mut self, tick: u32, ctx: &mut TickContext) -> (Vec<MobEvent>, Vec<[i32; 3]>) {
        let mut events = Vec::new();
        let mut blasts = Vec::new();
        events.extend(self.spawn_pass(ctx));

        // Depurar por distancia.
        let player = ctx.player_cell;
        self.list.retain(|mob| {
            let keep = manhattan(mob.cell, player) <= DESPAWN_DISTANCE;
            if !keep {
                events.push(MobEvent::Died(mob.kind));
            }
            keep
        });

        let mut melee_damage = 0u8;
        for mob in self.list.iter_mut() {
            mob.cooldown = mob.cooldown.saturating_sub(1);
            let dist = manhattan(mob.cell, player);
            match mob.kind {
                MobKind::TunnelBrawler => {
                    // Ciego: sigue el último ruido; a la vista, persigue.
                    if let Some((noise_pos, noise_tick)) = ctx.noise
                        && tick - noise_tick <= STORY_TICK_RATE * 4
                    {
                        mob.state = MobState::Seek;
                        mob.target = noise_pos;
                    }
                    if dist <= 2 && mob.cooldown == 0 {
                        mob.cooldown = 20;
                        melee_damage += mob.kind.melee_damage();
                    } else {
                        move_towards(mob, ctx.world, mob.target);
                    }
                }
                MobKind::DustStalker => {
                    if matches!(mob.state, MobState::Fuse(_)) {
                        let MobState::Fuse(t) = mob.state else {
                            unreachable!()
                        };
                        let fused = t + 1;
                        if dist > FUSE_DISTANCE + 2 {
                            // El jugador escapó: desactiva.
                            mob.state = MobState::Chase;
                        } else if fused >= FUSE_TICKS {
                            events.push(MobEvent::Explosion(mob.cell));
                            blasts.push(mob.cell);
                            mob.health = 0;
                        } else {
                            if fused == 1 {
                                events.push(MobEvent::Hiss(mob.cell));
                            }
                            mob.state = MobState::Fuse(fused);
                        }
                    } else if dist <= FUSE_DISTANCE {
                        mob.state = MobState::Fuse(0);
                    } else if dist <= SPAWN_MAX_DISTANCE {
                        mob.state = MobState::Chase;
                        move_towards(mob, ctx.world, player);
                    }
                }
                MobKind::FossilGunner => {
                    if dist < 8 {
                        // Retroceso táctico: se aleja un paso.
                        let away = [
                            mob.cell[0] + (mob.cell[0] - player[0]).signum(),
                            mob.cell[1],
                            mob.cell[2] + (mob.cell[2] - player[2]).signum(),
                        ];
                        if path::walkable(ctx.world, away[0], away[1], away[2]) {
                            mob.cell = away;
                        }
                        mob.state = MobState::KeepDistance;
                    } else if dist <= 24 && mob.cooldown == 0 {
                        mob.cooldown = 40;
                        if let Some(dir) = unit_dir(mob.cell, player) {
                            let speed = 18 * FIXED_ONE / 10;
                            self.projectiles.push(Projectile {
                                pos: [
                                    mob.cell[0] * FIXED_ONE + FIXED_ONE / 2,
                                    mob.cell[1] * FIXED_ONE + FIXED_ONE,
                                    mob.cell[2] * FIXED_ONE + FIXED_ONE / 2,
                                ],
                                vel: [dir[0] * speed, dir[1] * speed, dir[2] * speed],
                                ttl: 80,
                            });
                            events.push(MobEvent::ShotFired(mob.cell));
                        }
                    }
                }
                MobKind::CreviceVermin => {
                    if dist <= 1 && mob.cooldown == 0 {
                        mob.cooldown = 25;
                        melee_damage += mob.kind.melee_damage();
                    } else {
                        move_towards(mob, ctx.world, player);
                    }
                }
            }
        }
        self.list.retain(|mob| mob.health > 0);
        if melee_damage > 0 {
            events.push(MobEvent::HitPlayer(melee_damage));
        }
        events.extend(self.step_projectiles(ctx));
        (events, blasts)
    }

    /// Daño de espada a un enemigo en una celda.
    pub fn damage_at(&mut self, cell: [i32; 3], amount: u8) -> Option<MobEvent> {
        let mob = self
            .list
            .iter_mut()
            .find(|m| m.cell == cell || manhattan(m.cell, cell) <= 1)?;
        mob.health = mob.health.saturating_sub(amount);
        if mob.health == 0 {
            Some(MobEvent::Died(mob.kind))
        } else {
            None
        }
    }

    fn step_projectiles(&mut self, ctx: &TickContext) -> Vec<MobEvent> {
        let mut events = Vec::new();
        let player_center = [
            ctx.player_cell[0] * FIXED_ONE + FIXED_ONE / 2,
            ctx.player_cell[1] * FIXED_ONE + FIXED_ONE / 2,
            ctx.player_cell[2] * FIXED_ONE + FIXED_ONE / 2,
        ];
        self.projectiles.retain_mut(|p| {
            p.ttl = p.ttl.saturating_sub(1);
            if p.ttl == 0 {
                return false;
            }
            for axis in 0..3 {
                p.pos[axis] += p.vel[axis];
            }
            let cell = p.pos.map(|c| c.div_euclid(FIXED_ONE));
            if ctx.world.blocks_movement(cell[0], cell[1], cell[2]) {
                return false; // Chocó contra la roca.
            }
            // Distancia en cuartos de bloque: el proyectil golpea si pasa
            // a menos de media diagonal del jugador.
            let d2: i64 = (0..3)
                .map(|i| {
                    (i64::from(p.pos[i]) - i64::from(player_center[i])) / (FIXED_ONE as i64 / 4)
                })
                .map(|d| d * d)
                .sum();
            if d2 <= 1 {
                events.push(MobEvent::ProjectileHit(4));
                return false;
            }
            true
        });
        events
    }
}

/// Da un paso hacia el objetivo con A* local.
fn move_towards(mob: &mut Mob, world: &VoxelWorld, target: [i32; 3]) {
    if let Some(next) = path::next_step(world, mob.cell, target) {
        mob.cell = next;
    }
}

fn manhattan(a: [i32; 3], b: [i32; 3]) -> i32 {
    (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs()
}

/// Dirección entera unitaria (para el disparo del Artillero).
fn unit_dir(from: [i32; 3], to: [i32; 3]) -> Option<[i32; 3]> {
    let d = [
        to[0] * FIXED_ONE - from[0] * FIXED_ONE,
        (to[1] + 1) * FIXED_ONE - from[1] * FIXED_ONE,
        to[2] * FIXED_ONE - from[2] * FIXED_ONE,
    ];
    let len = d.iter().map(|&c| i64::from(c) * i64::from(c)).sum::<i64>();
    if len == 0 {
        return None;
    }
    // Normalización entera aproximada con la suma de valores absolutos.
    let norm: i32 = d.iter().map(|c| c.abs()).sum();
    Some(d.map(|c| {
        let scaled = i64::from(c) * 256 / i64::from(norm.max(1));
        scaled.clamp(-256, 256) as i32
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_world() -> VoxelWorld {
        let mut w = VoxelWorld::new([48, 16, 48]).unwrap();
        w.fill([0, 0, 0], [47, 2, 47], 1);
        w
    }

    impl<'a> TickContext<'a> {
        /// Contexto de prueba: mundo, jugador y luz constante.
        fn with(
            world: &'a VoxelWorld,
            player: [i32; 3],
            light: &'a dyn Fn(i32, i32, i32) -> u8,
            rng: &'a mut crate::Rng,
        ) -> TickContext<'a> {
            TickContext {
                world,
                player_cell: player,
                noise: None,
                light,
                rng,
            }
        }
    }

    #[test]
    fn spawns_in_darkness_within_range() {
        let world = flat_world();
        let mut rng = crate::Rng::new(5);
        // Forzamos el intervalo.
        let mut mobs = Mobs {
            spawn_timer: SPAWN_INTERVAL_TICKS,
            ..Mobs::default()
        };
        let light = |_, _, _| 0u8;
        let mut ctx = TickContext::with(&world, [24, 3, 24], &light, &mut rng);
        let events = mobs.spawn_pass(&mut ctx);
        assert!(!events.is_empty(), "no generó nada en oscuridad total");
        assert!(mobs.list.len() <= MAX_MOBS);
        for mob in &mobs.list {
            let dist = manhattan(mob.cell, [24, 3, 24]);
            assert!((SPAWN_MIN_DISTANCE..=SPAWN_MAX_DISTANCE).contains(&dist));
        }
    }

    #[test]
    fn does_not_spawn_in_light() {
        let world = flat_world();
        let mut rng = crate::Rng::new(5);
        let mut mobs = Mobs {
            spawn_timer: SPAWN_INTERVAL_TICKS,
            ..Mobs::default()
        };
        let light = |_, _, _| SPAWN_MAX_LIGHT + 1;
        let mut ctx = TickContext::with(&world, [24, 3, 24], &light, &mut rng);
        assert!(mobs.spawn_pass(&mut ctx).is_empty());
    }

    #[test]
    fn stalker_hisses_then_explodes() {
        let world = flat_world();
        let mut rng = crate::Rng::new(1);
        let mut mobs = Mobs::default();
        mobs.list.push(Mob {
            kind: MobKind::DustStalker,
            cell: [24, 3, 26],
            health: 8,
            state: MobState::Wander,
            target: [24, 3, 26],
            cooldown: 0,
        });
        let player = [24, 3, 24];
        let mut hissed = false;
        let mut exploded = false;
        let light = |_, _, _| 0u8;
        for t in 0..(FUSE_TICKS + 2) {
            let mut ctx = TickContext::with(&world, player, &light, &mut rng);
            let (events, blasts) = mobs.step(t, &mut ctx);
            if events.contains(&MobEvent::Hiss([24, 3, 26])) {
                hissed = true;
            }
            if !blasts.is_empty() {
                exploded = true;
                break;
            }
        }
        assert!(hissed && exploded);
        assert!(
            mobs.list.is_empty(),
            "el acechador murió en la deflagración"
        );
    }

    #[test]
    fn brawler_follows_noise_and_melees() {
        let world = flat_world();
        let mut rng = crate::Rng::new(2);
        let mut mobs = Mobs::default();
        mobs.list.push(Mob {
            kind: MobKind::TunnelBrawler,
            cell: [24, 3, 30],
            health: 14,
            state: MobState::Wander,
            target: [24, 3, 30],
            cooldown: 0,
        });
        let player = [24, 3, 24];
        let mut hit = false;
        let light = |_, _, _| 0u8;
        for t in 0..120 {
            let mut ctx = TickContext {
                noise: Some((player, t)),
                ..TickContext::with(&world, player, &light, &mut rng)
            };
            let (events, _) = mobs.step(t, &mut ctx);
            if events.iter().any(|e| matches!(e, MobEvent::HitPlayer(_))) {
                hit = true;
                break;
            }
        }
        assert!(hit, "el bregador nunca alcanzó al jugador");
    }

    #[test]
    fn swarm_spawns_on_gravel_dig() {
        let mut mobs = Mobs::default();
        mobs.spawn_swarm([10, 3, 10], 3);
        assert_eq!(mobs.list.len(), 3);
        assert!(mobs.list.iter().all(|m| m.kind == MobKind::CreviceVermin));
    }
}
