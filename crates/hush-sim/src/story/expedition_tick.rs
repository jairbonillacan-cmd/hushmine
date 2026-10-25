//! Avance de la expedición un tick: órdenes validadas y sistemas (movimiento
//! con acecho y escalón, fisiología, minado, hornos, fluidos, enemigos,
//! hitos y asedio).

use grisu_voxel::{AIR, FIXED_ONE};

use super::expedition::{DamageCause, Expedition, StoryEvent, StoryEventKind, StoryOrder};
use super::items::ItemId;
use super::milestones::ProgressSignals;
use super::mobs::TickContext;
use super::vitals::{FATIGUE_JUMP, FATIGUE_SPRINT, FATIGUE_WALK};
use crate::rng::Rng;
use crate::{BlockKind, MoveInput, PlayerBody, SPRINT_PER_TICK, WALK_PER_TICK};

impl Expedition {
    /// Avanza un tick aplicando las órdenes del jugador. Devuelve los
    /// eventos del tick.
    pub fn step(&mut self, orders: &[StoryOrder]) -> Vec<StoryEvent> {
        self.tick += 1;
        self.clock.advance();
        if self.dead {
            return Vec::new();
        }
        for order in orders {
            self.apply(*order);
        }
        self.step_movement();
        self.step_vitals();
        self.step_mining_tick();
        self.step_furnaces();
        self.step_fluids();
        if !self.config.peaceful {
            self.step_mobs();
        }
        self.step_milestones();
        self.mix_hash();
        std::mem::take(&mut self.events)
    }

    fn apply(&mut self, order: StoryOrder) {
        match order {
            StoryOrder::Move(input) => self.pending_input = input,
            StoryOrder::Sneak(on) => self.sneak = on,
            StoryOrder::Select(i) => {
                if i < 9 {
                    self.inventory.selected = i;
                }
            }
            StoryOrder::Swap(a, b) => self.inventory.swap(a, b),
            StoryOrder::Mine(cell) => self.set_mining(cell),
            StoryOrder::Place(cell) => self.place_block(cell),
            StoryOrder::Craft { grid, workbench } => self.craft(grid, workbench),
            StoryOrder::Interact(cell) => self.interact(cell),
            StoryOrder::FurnaceLoad(cell, place) => self.furnace_load(cell, place),
            StoryOrder::FurnaceTake(cell, slot) => self.furnace_take(cell, slot),
            StoryOrder::ChestPut(cell, place) => self.chest_put(cell, place),
            StoryOrder::ChestTake(cell, slot) => self.chest_take(cell, slot),
            StoryOrder::Eat => self.eat(),
            StoryOrder::Attack(cell) => self.attack(cell),
        }
    }

    // ---- Movimiento: acecho, escalón y fatiga ----

    fn step_movement(&mut self) {
        let input = self.pending_input;
        let was_on_ground = self.body.on_ground;
        self.vitals.pre_move(self.body.feet[1], was_on_ground);
        let input = if self.sneak {
            MoveInput {
                wish_x: input.wish_x / 2,
                wish_z: input.wish_z / 2,
                ..input
            }
        } else {
            input
        };
        let start = self.body;
        self.body.step(&self.world, input);
        // Seguridad en bordes: agachado no se cae. Se deshace el paso entero
        // (no solo x y z: con la y ya bajada, el minero se hundiría en el
        // suelo).
        if self.sneak && was_on_ground && !self.body.on_ground && self.body.velocity[1] < 0 {
            self.body = PlayerBody {
                velocity: [0; 3],
                ..start
            };
        }
        let start = start.feet;
        // Mecánica de escalón: sube desniveles de un bloque sin saltar.
        self.try_step_up(input, start);
        // Fatiga metabólica y ruido de pasos.
        let moving = input.wish_x != 0 || input.wish_z != 0;
        if input.sprint && moving {
            self.vitals.add_fatigue(FATIGUE_SPRINT);
            if self.tick.is_multiple_of(10) {
                self.record_noise(self.feet_cell());
            }
        } else if moving {
            self.vitals.add_fatigue(FATIGUE_WALK);
        }
        if input.jump && was_on_ground {
            self.vitals.add_fatigue(FATIGUE_JUMP);
        }
        // Aterrizaje: daño por caída salvo en fluidos.
        if self.body.on_ground {
            let feet = self.feet_cell();
            let in_fluid = matches!(
                BlockKind::from_id(self.world.get(feet[0], feet[1], feet[2])),
                Ok(BlockKind::Water | BlockKind::Lava)
            );
            let (_, dead) = self.vitals.land(self.body.feet[1], true, in_fluid);
            if dead && !self.dead {
                self.dead = true;
                self.push_event(StoryEventKind::Died);
            }
        }
    }

    /// Si el avance horizontal quedó bloqueado por un escalón de un bloque,
    /// lo sube si hay hueco para la cabeza.
    fn try_step_up(&mut self, input: MoveInput, start: [i64; 3]) {
        let wanted_x = i64::from(input.wish_x) * i64::from(self.speed(input)) / 127;
        let wanted_z = i64::from(input.wish_z) * i64::from(self.speed(input)) / 127;
        let got_x = self.body.feet[0] - start[0];
        let got_z = self.body.feet[2] - start[2];
        // Bloqueado si avanzó menos de la mitad de lo pedido en algún eje. (Se
        // comparaba con medio bloque, que por tick nunca se alcanza: el
        // escalón no subía nunca.)
        let short = |wanted: i64, got: i64| wanted != 0 && got.abs() * 2 < wanted.abs();
        let blocked = short(wanted_x, got_x) || short(wanted_z, got_z);
        if !blocked || !self.body.on_ground {
            return;
        }
        let mut lifted = PlayerBody {
            feet: [start[0], start[1] + FIXED_ONE, start[2]],
            velocity: [0; 3],
            on_ground: false,
        };
        lifted.step(&self.world, input);
        let lifted_gain = (lifted.feet[0] - start[0]).abs() + (lifted.feet[2] - start[2]).abs();
        if lifted_gain > got_x.abs() + got_z.abs() {
            self.body = lifted;
        }
    }

    fn speed(&self, input: MoveInput) -> i32 {
        if input.sprint {
            SPRINT_PER_TICK
        } else {
            WALK_PER_TICK
        }
    }

    fn step_vitals(&mut self) {
        let head = self.head_cell();
        let head_submerged = self.world.get(head[0], head[1], head[2]) == BlockKind::Water.id();
        // Quemaduras de magma al contacto.
        let feet = self.feet_cell();
        let body_in_lava = self.world.get(feet[0], feet[1], feet[2]) == BlockKind::Lava.id()
            || self.world.get(feet[0], feet[1] + 1, feet[2]) == BlockKind::Lava.id();
        let (damage, dead) = self.vitals.step(head_submerged);
        if body_in_lava && self.tick.is_multiple_of(10) {
            let dead = self.vitals.damage(2) || dead;
            self.push_event(StoryEventKind::Damaged {
                amount: 2,
                cause: DamageCause::Lava,
            });
            if dead && !self.dead {
                self.dead = true;
                self.push_event(StoryEventKind::Died);
            }
        }
        if damage > 0 {
            let cause = if head_submerged {
                DamageCause::Drowning
            } else {
                DamageCause::Starvation
            };
            self.push_event(StoryEventKind::Damaged {
                amount: damage,
                cause,
            });
        }
        if dead && !self.dead {
            self.dead = true;
            self.push_event(StoryEventKind::Died);
        }
    }

    // ---- Sistemas ----

    fn step_furnaces(&mut self) {
        let mut smelted = Vec::new();
        for furnace in self.furnaces.values_mut() {
            if let Some(stack) = furnace.step() {
                smelted.push(stack.id.0);
            }
        }
        for id in smelted {
            self.push_event(StoryEventKind::Smelted(id));
        }
    }

    fn step_fluids(&mut self) {
        let reactions = self.fluids.step(&mut self.world);
        for reaction in reactions {
            self.push_event(StoryEventKind::Fluid(reaction));
        }
    }

    fn step_mobs(&mut self) {
        let player_cell = self.feet_cell();
        let noise = self.noise;
        let tick = self.tick;
        let sky = self.clock.sky_factor_q8();
        // Durante el asedio las oleadas se aceleran.
        if self.milestones.siege_started && self.milestones.siege_ticks_left > 0 {
            self.mobs.accelerate_spawns(4);
        }
        let mut rng = std::mem::replace(&mut self.rng, Rng::new(0));
        let world = &self.world;
        let light = move |x: i32, y: i32, z: i32| -> u8 {
            // Luz de cielo de la columna, modulada por la hora, contra los
            // emisores cercanos (antorchas y lava).
            let open = ((y + 1)..world.size()[1] as i32).all(|yy| !world.is_solid(x, yy, z));
            let sky_l = if open {
                u16::from(15u8) * sky / 256
            } else {
                0u16
            };
            (sky_l as u8).max(local_emission(world, x, y, z))
        };
        let mut ctx = TickContext {
            world: &self.world,
            player_cell,
            noise,
            light: &light,
            rng: &mut rng,
        };
        let (events, blasts) = self.mobs.step(tick, &mut ctx);
        self.rng = rng;
        for event in events {
            if let super::mobs::MobEvent::HitPlayer(d) | super::mobs::MobEvent::ProjectileHit(d) =
                event
            {
                let cause = if matches!(event, super::mobs::MobEvent::HitPlayer(_)) {
                    DamageCause::Mob
                } else {
                    DamageCause::Projectile
                };
                self.push_event(StoryEventKind::Damaged { amount: d, cause });
                if self.vitals.damage(d) && !self.dead {
                    self.dead = true;
                    self.push_event(StoryEventKind::Died);
                }
            } else {
                self.push_event(StoryEventKind::Mob(event));
            }
        }
        // Deflagraciones: destruyen el terreno alrededor.
        for blast in blasts {
            let r = super::mobs::BLAST_RADIUS;
            for dy in -r..=r {
                for dz in -r..=r {
                    for dx in -r..=r {
                        let c = [blast[0] + dx, blast[1] + dy, blast[2] + dz];
                        if c[1] >= self.bands.bedrock {
                            self.world.set(c[0], c[1], c[2], AIR);
                        }
                    }
                }
            }
            // Daño por proximidad.
            let feet = self.feet_cell();
            let dist = (feet[0] - blast[0]).abs()
                + (feet[1] - blast[1]).abs()
                + (feet[2] - blast[2]).abs();
            if dist <= 3 {
                let amount = 12u8.saturating_sub(u8::try_from(dist * 3).unwrap_or(12));
                self.push_event(StoryEventKind::Damaged {
                    amount,
                    cause: DamageCause::Mob,
                });
                if self.vitals.damage(amount) && !self.dead {
                    self.dead = true;
                    self.push_event(StoryEventKind::Died);
                }
            }
        }
    }

    fn step_milestones(&mut self) {
        let signals = ProgressSignals {
            has_iron_ingot: self.inventory.count_of(ItemId(super::items::IRON_INGOT)) > 0,
            has_steel_tool: self
                .inventory
                .hotbar
                .iter()
                .chain(&self.inventory.backpack)
                .flatten()
                .any(|s| s.id.tool_tier() == Some(3)),
            has_diamond: self.inventory.count_of(ItemId(super::items::DIAMOND_GEM)) > 0,
            player_y: self.feet_cell()[1],
            on_surface: self.feet_cell()[1] >= self.well.escape_y,
        };
        let inventory = &self.inventory;
        let bands = self.bands;
        if let Some(milestone) = self.milestones.check(signals, inventory, &bands) {
            self.push_event(StoryEventKind::MilestoneReached(milestone as u8));
        }
        let alive = !self.dead;
        let on_surface = signals.on_surface;
        match self.milestones.step_siege(alive, on_surface) {
            Some(true) => {
                self.push_event(StoryEventKind::Victory);
            }
            Some(false) if !self.dead => {
                self.dead = true;
                self.push_event(StoryEventKind::Died);
            }
            Some(false) | None => {}
        }
    }

    pub(super) fn record_noise(&mut self, cell: [i32; 3]) {
        self.noise = Some((cell, self.tick));
    }

    fn mix_hash(&mut self) {
        let mut values = Vec::new();
        values.push(u64::from(self.tick));
        values.push(u64::from(self.vitals.health));
        values.push(u64::from(self.vitals.food));
        values.extend(self.body.feet.iter().map(|&c| u64::from(c as u32)));
        values.push(self.mobs.list.len() as u64);
        values.push(u64::from(self.milestones.reached));
        for value in values {
            self.state_hash = (self.state_hash ^ value).wrapping_mul(0x0000_0100_0000_01B3);
        }
    }
}

/// Emisores cercanos a una celda (para las reglas de generación).
fn local_emission(world: &grisu_voxel::VoxelWorld, x: i32, y: i32, z: i32) -> u8 {
    const R: i32 = 7;
    let mut best = 0u8;
    for dy in -R..=R {
        for dz in -R..=R {
            for dx in -R..=R {
                let dist = dx.abs() + dy.abs() + dz.abs();
                if dist > R {
                    continue;
                }
                let e = super::expedition::emission_of(world.get(x + dx, y + dy, z + dz));
                if e > 0 {
                    best = best.max(e.saturating_sub(dist as u8));
                }
            }
        }
    }
    best
}
