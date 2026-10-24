//! Acciones de jugadores, economía, bolsas de botín e invariantes sobre [`Simulation`].

use crate::altar::AltarError;
use crate::economy::{DepositOutcome, Economy, EconomyError};
use crate::events::{MatchEventKind, Team};
use crate::loot::{GridPos, LootBag, LootContents, LootError, LootPickup, LootStore};
use crate::sim::Simulation;

impl Simulation {
    /// Devuelve una referencia inmutable a la economía de la partida.
    pub fn economy(&self) -> &Economy {
        &self.economy
    }

    /// Devuelve una referencia mutable a la economía de la partida.
    pub fn economy_mut(&mut self) -> &mut Economy {
        &mut self.economy
    }

    /// Devuelve una referencia inmutable al almacén de bolsas de botín activas.
    pub fn loot(&self) -> &LootStore {
        &self.loot
    }

    /// Devuelve una referencia mutable al almacén de bolsas de botín activas.
    pub fn loot_mut(&mut self) -> &mut LootStore {
        &mut self.loot
    }

    /// Configura el tamaño de equipo (`1..=8`) y actualiza la meta de victoria.
    pub fn set_team_size(&mut self, team_size: u8) -> Result<(), EconomyError> {
        self.economy.set_team_size(team_size)
    }

    /// Registra o actualiza el equipo de un jugador en la economía del Turno.
    pub fn register_player(&mut self, player_id: u32, team: Team) {
        self.economy.register_player(player_id, team);
    }

    /// Extrae bloques de mena para un jugador y devuelve las unidades añadidas.
    pub fn mine_ore(
        &mut self,
        player_id: u32,
        team: Team,
        blocks: u32,
    ) -> Result<u32, EconomyError> {
        self.economy.mine_ore(player_id, team, blocks)
    }

    /// Extrae bloques de oro para un jugador, anota `GoldMined` y devuelve el valor base (`$`).
    pub fn mine_gold(
        &mut self,
        player_id: u32,
        team: Team,
        blocks: u32,
    ) -> Result<u32, EconomyError> {
        let cash_value = self.economy.mine_gold(player_id, team, blocks)?;
        if cash_value > 0 {
            self.event_log.record(
                self.tick,
                MatchEventKind::GoldMined {
                    player_id,
                    cash_value,
                },
                Some(team),
            );
        }
        Ok(cash_value)
    }

    /// Extrae bloques de diamante para un jugador, anota `GoldMined` y devuelve el valor base (`$`).
    pub fn mine_diamond(
        &mut self,
        player_id: u32,
        team: Team,
        blocks: u32,
    ) -> Result<u32, EconomyError> {
        let cash_value = self.economy.mine_diamond(player_id, team, blocks)?;
        if cash_value > 0 {
            self.event_log.record(
                self.tick,
                MatchEventKind::GoldMined {
                    player_id,
                    cash_value,
                },
                Some(team),
            );
        }
        Ok(cash_value)
    }

    /// Deposita el oro y los diamantes que lleva el jugador en el banco de su equipo.
    pub fn deposit_player(&mut self, player_id: u32) -> Result<DepositOutcome, EconomyError> {
        let outcome = self.economy.deposit_player(player_id)?;
        let target_cash = self.economy.target_cash();
        self.event_log.record(
            self.tick,
            MatchEventKind::Deposited {
                player_id,
                base_cash: outcome.base_cash,
                credited_cash: outcome.credited_cash,
                new_balance: outcome.new_bank_balance,
            },
            Some(outcome.team),
        );
        if outcome.fame_newly_activated {
            self.event_log.record(
                self.tick,
                MatchEventKind::FameGained {
                    balance: outcome.new_bank_balance,
                    target_cash,
                },
                Some(outcome.team),
            );
        }
        if outcome.goal_reached {
            self.event_log.record(
                self.tick,
                MatchEventKind::GoalReached {
                    balance: outcome.new_bank_balance,
                    target_cash,
                },
                Some(outcome.team),
            );
        }
        Ok(outcome)
    }

    /// Procesa la muerte de un jugador soltando una bolsa de botín si llevaba recursos.
    pub fn kill_player(
        &mut self,
        player_id: u32,
        position: GridPos,
    ) -> Result<Option<LootBag>, LootError> {
        let Some((team, load)) = self.economy.take_player_load_for_death(player_id) else {
            return Ok(None);
        };
        if load.is_empty() {
            return Ok(None);
        }
        let contents = LootContents::new(load.ore, load.total_gold(), load.total_diamond());
        let bag = self.loot.drop_bag(self.tick, position, team, contents)?;
        self.event_log.record(
            self.tick,
            MatchEventKind::LootDropped {
                bag_id: bag.id,
                cash_value: bag.base_cash_value().unwrap_or(0),
            },
            Some(team),
        );
        Ok(Some(bag))
    }

    /// Recoge una bolsa de botín activa y transfiere sus recursos al jugador.
    pub fn pickup_loot(
        &mut self,
        player_id: u32,
        picker_team: Team,
        bag_id: u32,
    ) -> Result<LootPickup, LootError> {
        let pickup = self.loot.pickup(self.tick, bag_id, picker_team)?;
        if self
            .economy
            .give_loot(
                player_id,
                (picker_team, pickup.is_stolen),
                [
                    pickup.bag.contents.ore,
                    pickup.bag.contents.gold,
                    pickup.bag.contents.diamond,
                ],
            )
            .is_err()
        {
            self.loot.insert_bag_unchecked(pickup.bag);
            return Err(LootError::Overflow);
        }
        self.event_log.record(
            self.tick,
            MatchEventKind::LootPicked {
                bag_id,
                picker_id: player_id,
                origin_team: pickup.bag.origin_team,
                base_cash: pickup.base_cash_value,
                effective_cash: pickup.effective_cash_value,
            },
            Some(picker_team),
        );
        Ok(pickup)
    }

    /// Ofrenda un lote de oro en el altar descontando directamente del banco del equipo.
    pub fn offer_from_bank(&mut self, team: Team) -> Result<u32, AltarError> {
        self.offer_gold(team, self.economy.bank(team))
    }

    /// Ejecuta todas las comprobaciones de invariantes sobre el estado actual de la simulación.
    pub fn check_invariants(&self) -> Vec<String> {
        crate::invariants::check_all(
            self.tick,
            &self.seismograph,
            &self.economy,
            &self.loot,
            &self.event_log,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MatchConfig;

    #[test]
    fn full_economy_loot_and_fame_flow_preserves_invariants() {
        let mut sim = Simulation::new(MatchConfig {
            seed: 7,
            max_ticks: 5_000,
            ..MatchConfig::default()
        })
        .unwrap();
        sim.set_team_size(1).unwrap();
        sim.register_player(1, Team::Copper);
        sim.register_player(2, Team::Cobalt);

        sim.mine_ore(1, Team::Copper, 2).unwrap();
        assert_eq!(sim.mine_gold(1, Team::Copper, 4).unwrap(), 400);
        let bag = sim
            .kill_player(1, GridPos::new(2, 3, 4))
            .unwrap()
            .expect("debe soltar bolsa");
        assert_eq!(bag.base_cash_value(), Some(400));

        let pickup = sim.pickup_loot(2, Team::Cobalt, bag.id).unwrap();
        assert_eq!(pickup.effective_cash_value, 500);

        sim.mine_diamond(2, Team::Cobalt, 4).unwrap();
        let dep = sim.deposit_player(2).unwrap();
        assert_eq!(dep.credited_cash, 2_100);
        assert!(dep.fame_newly_activated);
        assert!(dep.goal_reached);
        assert!(sim.is_finished());
        assert!(sim.check_invariants().is_empty());
    }
}
