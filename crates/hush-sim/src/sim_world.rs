//! Terreno, mineros y bots dentro de la [`Simulation`]: altas y bajas de
//! jugadores, órdenes, cavar y poner, movimiento y sus invariantes.

use grisu_voxel::{FIXED_ONE, VoxelWorld};

use crate::player::spawn_point;
use crate::sim::{BlockChange, Command, EditStats, Rejection};
use crate::{EditError, MoveInput, PlayerBody, Simulation};

impl Simulation {
    /// Añade un jugador en un punto de la superficie y devuelve su índice.
    pub fn spawn_player(&mut self) -> usize {
        let [sx, _, sz] = self.world.size();
        let x = 2 + self.rng.below(sx - 4) as i32;
        let z = 2 + self.rng.below(sz - 4) as i32;
        self.players
            .push(PlayerBody::new(spawn_point(&self.world, x, z)));
        self.active.push(true);
        self.inputs.push(MoveInput::default());
        self.players.len() - 1
    }

    /// Retira a un jugador (desconexión). Su índice no se reutiliza.
    pub fn remove_player(&mut self, player: usize) {
        if let Some(active) = self.active.get_mut(player) {
            *active = false;
        }
    }

    pub fn is_active(&self, player: usize) -> bool {
        self.active.get(player).copied().unwrap_or(false)
    }

    /// Jugadores presentes, con su índice.
    pub fn active_players(&self) -> impl Iterator<Item = (usize, &PlayerBody)> {
        self.players
            .iter()
            .enumerate()
            .filter(|(i, _)| self.active[*i])
    }

    /// Bloques cambiados desde la última llamada, en orden.
    pub fn take_changes(&mut self) -> Vec<BlockChange> {
        std::mem::take(&mut self.changes)
    }

    /// Acciones rechazadas desde la última llamada.
    pub fn take_rejections(&mut self) -> Vec<Rejection> {
        std::mem::take(&mut self.rejections)
    }

    pub fn world(&self) -> &VoxelWorld {
        &self.world
    }

    /// Acceso mutable al terreno, para que el cliente recoja los chunks sucios.
    pub fn world_mut(&mut self) -> &mut VoxelWorld {
        &mut self.world
    }

    pub fn players(&self) -> &[PlayerBody] {
        &self.players
    }

    pub fn stats(&self) -> &EditStats {
        &self.stats
    }

    /// Activa o detiene a los bots. Detenidos, siguen en el mapa sin moverse.
    pub fn set_bots_active(&mut self, active: bool) {
        self.bots_active = active;
        if !active {
            for (player, _) in &self.bots {
                self.inputs[*player] = MoveInput::default();
            }
        }
    }

    /// Encola una orden para el próximo tick.
    pub fn submit(&mut self, command: Command) {
        self.pending.push(command);
    }

    /// Parte del tick que toca al terreno: órdenes y movimiento.
    pub(crate) fn step_world(&mut self) {
        let mut commands = std::mem::take(&mut self.pending);
        if self.bots_active {
            for (player, bot) in &mut self.bots {
                commands.extend(bot.think(*player, &self.players[*player], self.tick));
            }
        }
        for command in commands {
            if let Err(err) = self.apply(command) {
                *self.stats.rejected.entry(format!("{err:?}")).or_default() += 1;
                if let Command::Dig { player, cell } | Command::Place { player, cell, .. } = command
                {
                    self.rejections.push(Rejection {
                        player,
                        cell,
                        error: err,
                    });
                }
            }
        }
        for (i, (body, input)) in self.players.iter_mut().zip(&self.inputs).enumerate() {
            if self.active[i] {
                body.step(&self.world, *input);
            }
        }
        self.check_world_invariants();
    }

    fn apply(&mut self, command: Command) -> Result<(), EditError> {
        let player = match command {
            Command::Move { player, .. }
            | Command::Dig { player, .. }
            | Command::Place { player, .. } => player,
        };
        if !self.is_active(player) {
            return Err(EditError::UnknownPlayer);
        }
        let body = self.players[player];
        match command {
            Command::Move { input, .. } => self.inputs[player] = input,
            Command::Dig { cell, .. } => {
                let block = crate::edit::dig(&mut self.world, &body, cell)?;
                *self.stats.dug.entry(block.name().to_owned()).or_default() += 1;
                self.changes.push(BlockChange { cell, block: 0 });
            }
            Command::Place { cell, block, .. } => {
                let others: Vec<PlayerBody> = self.active_players().map(|(_, b)| *b).collect();
                crate::edit::place(&mut self.world, &body, &others, cell, block)?;
                self.stats.placed += 1;
                self.changes.push(BlockChange {
                    cell,
                    block: block.id(),
                });
            }
        }
        Ok(())
    }

    /// Ningún minero dentro de un bloque ni fuera del mapa.
    fn check_world_invariants(&mut self) {
        for (i, body) in self.players.iter().enumerate() {
            if !self.active[i] {
                continue;
            }
            if body.aabb().intersects_world(&self.world) {
                self.world_violations.push(format!(
                    "tick {}: jugador {i} dentro de un bloque",
                    self.tick
                ));
            }
            let [x, _, z] = body.feet.map(|c| c.div_euclid(FIXED_ONE));
            if !self.world.in_bounds(x, 0, z) || body.feet[1] < 0 {
                self.world_violations
                    .push(format!("tick {}: jugador {i} fuera del mapa", self.tick));
            }
        }
    }
}
