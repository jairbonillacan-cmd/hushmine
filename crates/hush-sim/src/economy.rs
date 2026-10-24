//! Economía determinista del Turno: bancos, metas, Fama (`60 %`) y botín (`×5/4`).

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::Team;

pub const DROP_IN_PHASE_TICKS: u32 = 600; // Se moverá a data/*.ron
pub const RESPAWN_TICKS: u32 = 180; // Se moverá a data/*.ron
pub const FAME_THRESHOLD_NUM: u32 = 60; // Se moverá a data/*.ron
pub const FAME_THRESHOLD_DEN: u32 = 100; // Se moverá a data/*.ron
pub const STOLEN_LOOT_MULTIPLIER_NUM: u32 = 5; // Se moverá a data/*.ron
pub const STOLEN_LOOT_MULTIPLIER_DEN: u32 = 4; // Se moverá a data/*.ron
pub const ORE_UNITS_PER_BLOCK: u32 = 4; // Se moverá a data/*.ron
pub const GOLD_UNITS_PER_BLOCK: u32 = 1; // Se moverá a data/*.ron
pub const GOLD_CASH_VALUE: u32 = 100; // Se moverá a data/*.ron
pub const DIAMOND_UNITS_PER_BLOCK: u32 = 2; // Se moverá a data/*.ron
pub const DIAMOND_CASH_VALUE: u32 = 400; // Se moverá a data/*.ron
pub const DIAMOND_UNIT_CASH_VALUE: u32 = DIAMOND_CASH_VALUE / DIAMOND_UNITS_PER_BLOCK; // 200 $ por gema
pub const MOTHER_LODE_TOTAL_CASH: u32 = 2_000; // Se moverá a data/*.ron
pub const HEART_TOTAL_CASH: u32 = 1_500; // Se moverá a data/*.ron
pub const MAX_POSTS_PER_LINE: u32 = 16; // Se moverá a data/*.ron
pub const MAX_POST_SPACING_BLOCKS: u32 = 8; // Se moverá a data/*.ron
pub const TRANSFER_INTERVAL_TICKS: u32 = 60; // Se moverá a data/*.ron
pub const UNITS_PER_BUCKET: u32 = 1; // Se moverá a data/*.ron
pub const TEAM_TARGETS: [u32; 8] = [2_000, 3_500, 5_000, 6_500, 8_000, 9_500, 11_000, 12_500]; // Se moverá a data/*.ron
pub const DEFAULT_TEAM_SIZE: u8 = 4;

/// Resultado estándar de las operaciones económicas.
pub type EconomyResult<T> = Result<T, EconomyError>;

/// Errores de las operaciones económicas del Turno.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EconomyError {
    InvalidTeamSize(u8),
    Overflow,
    InsufficientFunds { available: u32, required: u32 },
    EmptyDeposit,
}

impl fmt::Display for EconomyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTeamSize(s) => write!(f, "tamaño de equipo inválido ({s}); rango 1..=8"),
            Self::Overflow => write!(f, "desbordamiento aritmético en economía"),
            Self::InsufficientFunds {
                available: a,
                required: r,
            } => write!(f, "fondos insuficientes: {a} $ < {r} $"),
            Self::EmptyDeposit => write!(f, "no hay oro ni diamantes para depositar"),
        }
    }
}

impl std::error::Error for EconomyError {}

fn add(a: u32, b: u32) -> EconomyResult<u32> {
    a.checked_add(b).ok_or(EconomyError::Overflow)
}

fn mul(a: u32, b: u32) -> EconomyResult<u32> {
    a.checked_mul(b).ok_or(EconomyError::Overflow)
}

/// Devuelve la meta de victoria en dólares para un tamaño de equipo (`1..=8`).
pub const fn target_cash_for_team_size(team_size: u8) -> EconomyResult<u32> {
    if team_size >= 1 && team_size <= 8 {
        Ok(TEAM_TARGETS[(team_size - 1) as usize])
    } else {
        Err(EconomyError::InvalidTeamSize(team_size))
    }
}

/// Indica si la Fama está activa (`bank_cash >= 60 %` exacto de `target_cash`).
pub const fn is_fame_active(bank: u32, target: u32) -> bool {
    let (b, t) = (bank as u64, target as u64);
    target > 0 && b * (FAME_THRESHOLD_DEN as u64) >= t * (FAME_THRESHOLD_NUM as u64)
}

/// Calcula el valor acreditado por botín robado (`×5/4 = 125 %`), truncando hacia cero
/// (`floor(base_cash * 5 / 4)` para `u32`, p. ej. `100 -> 125`, `200 -> 250`, `400 -> 500`).
pub const fn stolen_loot_cash(base_cash: u32) -> EconomyResult<u32> {
    match base_cash.checked_mul(STOLEN_LOOT_MULTIPLIER_NUM) {
        Some(v) => Ok(v / STOLEN_LOOT_MULTIPLIER_DEN),
        None => Err(EconomyError::Overflow),
    }
}

const fn team_slot(team: Team) -> usize {
    match team {
        Team::Copper => 0,
        Team::Cobalt => 1,
    }
}

fn bullion_cash(g: u32, d: u32) -> EconomyResult<u32> {
    add(mul(g, GOLD_CASH_VALUE)?, mul(d, DIAMOND_UNIT_CASH_VALUE)?)
}

/// Recursos que un jugador lleva encima antes de depositarlos o perderlos al morir.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerLoad {
    pub ore: u32,
    pub own_gold: u32,
    pub own_diamond: u32,
    pub stolen_gold: u32,
    pub stolen_diamond: u32,
}

impl PlayerLoad {
    pub const fn is_empty(&self) -> bool {
        self.ore == 0 && self.total_gold() == 0 && self.total_diamond() == 0
    }

    pub const fn total_gold(&self) -> u32 {
        self.own_gold.saturating_add(self.stolen_gold)
    }

    pub const fn total_diamond(&self) -> u32 {
        self.own_diamond.saturating_add(self.stolen_diamond)
    }

    pub fn base_cash_value(&self) -> EconomyResult<u32> {
        let own = bullion_cash(self.own_gold, self.own_diamond)?;
        add(own, bullion_cash(self.stolen_gold, self.stolen_diamond)?)
    }
}

/// Resultado detallado de un depósito en el banco de una cuadrilla.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositOutcome {
    pub team: Team,
    pub base_cash: u32,
    pub credited_cash: u32,
    pub stolen_bonus: u32,
    pub new_bank_balance: u32,
    pub fame_newly_activated: bool,
    pub goal_reached: bool,
}

/// Estado económico determinista de la partida (`totals`: `[mined, dep_base, bonus, offered, expired]`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Economy {
    team_size: u8,
    target_cash: u32,
    banks: [u32; 2],
    fame_active: [bool; 2],
    winner: Option<Team>,
    players: BTreeMap<u32, (Team, PlayerLoad)>,
    totals: [u32; 5],
}

impl Economy {
    pub fn new(team_size: u8) -> EconomyResult<Self> {
        let mut econ = Self::with_default_team_size();
        econ.set_team_size(team_size)?;
        Ok(econ)
    }

    pub const fn with_default_team_size() -> Self {
        Self {
            team_size: DEFAULT_TEAM_SIZE,
            target_cash: TEAM_TARGETS[(DEFAULT_TEAM_SIZE - 1) as usize],
            banks: [0; 2],
            fame_active: [false; 2],
            winner: None,
            players: BTreeMap::new(),
            totals: [0; 5],
        }
    }

    pub const fn target_cash(&self) -> u32 {
        self.target_cash
    }

    pub const fn bank(&self, team: Team) -> u32 {
        self.banks[team_slot(team)]
    }

    pub const fn has_fame(&self, team: Team) -> bool {
        is_fame_active(self.bank(team), self.target_cash)
    }

    pub const fn winner(&self) -> Option<Team> {
        self.winner
    }

    pub const fn is_goal_reached(&self) -> bool {
        let t = self.target_cash;
        self.winner.is_some() || self.banks[0] >= t || self.banks[1] >= t
    }

    pub fn is_active(&self) -> bool {
        let [m, d, _, o, _] = self.totals;
        (m | d | o | self.banks[0] | self.banks[1]) > 0 || !self.players.is_empty()
    }

    pub fn set_team_size(&mut self, team_size: u8) -> EconomyResult<()> {
        let t = target_cash_for_team_size(team_size)?;
        (self.team_size, self.target_cash) = (team_size, t);
        self.fame_active = [0, 1].map(|i| is_fame_active(self.banks[i], t));
        let [b0, b1] = self.banks;
        let auto = (b0 >= t && b0 >= b1)
            .then_some(Team::Copper)
            .or((b1 >= t).then_some(Team::Cobalt));
        self.winner = self.winner.filter(|&w| self.bank(w) >= t).or(auto);
        Ok(())
    }

    pub fn register_player(&mut self, id: u32, team: Team) {
        self.players.insert(id, (team, self.load_of(id)));
    }

    pub fn player_load(&self, player_id: u32) -> Option<&PlayerLoad> {
        self.players.get(&player_id).map(|(_, load)| load)
    }

    fn load_of(&self, id: u32) -> PlayerLoad {
        self.players.get(&id).map(|(_, l)| *l).unwrap_or_default()
    }

    pub fn mine_ore(&mut self, id: u32, team: Team, blocks: u32) -> EconomyResult<u32> {
        let (mut l, units) = (self.load_of(id), mul(blocks, ORE_UNITS_PER_BLOCK)?);
        l.ore = add(l.ore, units)?;
        self.players.insert(id, (team, l));
        Ok(units)
    }

    pub fn mine_gold(&mut self, id: u32, team: Team, blocks: u32) -> EconomyResult<u32> {
        self.mine_bullion(id, team, blocks, false)
    }

    pub fn mine_diamond(&mut self, id: u32, team: Team, blocks: u32) -> EconomyResult<u32> {
        self.mine_bullion(id, team, blocks, true)
    }

    fn mine_bullion(&mut self, id: u32, team: Team, n: u32, dia: bool) -> EconomyResult<u32> {
        let mut l = self.load_of(id);
        let (u, c, s) = if dia {
            (
                DIAMOND_UNITS_PER_BLOCK,
                DIAMOND_CASH_VALUE,
                &mut l.own_diamond,
            )
        } else {
            (GOLD_UNITS_PER_BLOCK, GOLD_CASH_VALUE, &mut l.own_gold)
        };
        let (units, cash) = (mul(n, u)?, mul(n, c)?);
        (*s, self.totals[0]) = (add(*s, units)?, add(self.totals[0], cash)?);
        self.players.insert(id, (team, l));
        Ok(cash)
    }

    pub fn deposit_cash(&mut self, t: Team, cash: u32, rob: bool) -> EconomyResult<DepositOutcome> {
        self.apply_deposit(t, if rob { 0 } else { cash }, if rob { cash } else { 0 })
    }

    pub fn deposit_player(&mut self, id: u32) -> EconomyResult<DepositOutcome> {
        let &(t, mut l) = self.players.get(&id).ok_or(EconomyError::EmptyDeposit)?;
        let own = bullion_cash(l.own_gold, l.own_diamond)?;
        let out = self.apply_deposit(t, own, bullion_cash(l.stolen_gold, l.stolen_diamond)?)?;
        (l.own_gold, l.own_diamond, l.stolen_gold, l.stolen_diamond) = (0, 0, 0, 0);
        self.players.insert(id, (t, l));
        Ok(out)
    }

    fn apply_deposit(&mut self, team: Team, own: u32, rob: u32) -> EconomyResult<DepositOutcome> {
        let base = add(own, rob)?;
        if base == 0 {
            return Err(EconomyError::EmptyDeposit);
        }
        let rc = stolen_loot_cash(rob)?;
        let (stolen_bonus, cred) = (rc.saturating_sub(rob), add(own, rc)?);
        let s = team_slot(team);
        let (nb, nd) = (add(self.banks[s], cred)?, add(self.totals[1], base)?);
        self.totals[2] = add(self.totals[2], stolen_bonus)?;
        let (was_fame, now_fame) = (self.fame_active[s], is_fame_active(nb, self.target_cash));
        (self.banks[s], self.fame_active[s], self.totals[1]) = (nb, now_fame, nd);
        let goal_reached = nb >= self.target_cash;
        if goal_reached && self.winner.is_none() {
            self.winner = Some(team);
        }
        Ok(DepositOutcome {
            team,
            base_cash: base,
            credited_cash: cred,
            stolen_bonus,
            new_bank_balance: nb,
            fame_newly_activated: !was_fame && now_fame,
            goal_reached,
        })
    }

    pub fn spend_for_offering(&mut self, team: Team, amount: u32) -> EconomyResult<u32> {
        let (s, a) = (team_slot(team), self.bank(team));
        let err = EconomyError::InsufficientFunds {
            available: a,
            required: amount,
        };
        let nb = a.checked_sub(amount).ok_or(err)?;
        self.totals[3] = add(self.totals[3], amount)?;
        (self.banks[s], self.fame_active[s]) = (nb, is_fame_active(nb, self.target_cash));
        Ok(nb)
    }

    pub fn take_player_load_for_death(&mut self, player_id: u32) -> Option<(Team, PlayerLoad)> {
        let (team, load) = self.players.get_mut(&player_id)?;
        Some((*team, std::mem::take(load)))
    }

    pub fn give_loot(&mut self, id: u32, (t, r): (Team, bool), v: [u32; 3]) -> EconomyResult<()> {
        let mut l = self.load_of(id);
        l.ore = add(l.ore, v[0])?;
        if r {
            (l.stolen_gold, l.stolen_diamond) =
                (add(l.stolen_gold, v[1])?, add(l.stolen_diamond, v[2])?);
        } else {
            (l.own_gold, l.own_diamond) = (add(l.own_gold, v[1])?, add(l.own_diamond, v[2])?);
        }
        self.players.insert(id, (t, l));
        Ok(())
    }

    pub fn record_expired_bag_cash(&mut self, base_cash: u32) -> EconomyResult<()> {
        self.totals[4] = add(self.totals[4], base_cash)?;
        Ok(())
    }

    pub const fn total_mined_cash(&self) -> u32 {
        self.totals[0]
    }

    pub const fn total_deposited_base_cash(&self) -> u32 {
        self.totals[1]
    }

    pub const fn total_stolen_bonus_cash(&self) -> u32 {
        self.totals[2]
    }

    pub const fn total_offered_cash(&self) -> u32 {
        self.totals[3]
    }

    pub const fn total_expired_bag_cash(&self) -> u32 {
        self.totals[4]
    }

    pub fn total_carried_base_cash(&self) -> EconomyResult<u32> {
        let mut sum = 0u32;
        for (_, load) in self.players.values() {
            sum = add(sum, load.base_cash_value()?)?;
        }
        Ok(sum)
    }

    pub fn set_bank_unchecked(&mut self, team: Team, bal: u32) {
        let s = team_slot(team);
        (self.banks[s], self.fame_active[s]) = (bal, is_fame_active(bal, self.target_cash));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fame_stolen_loot_and_victory_thresholds() {
        assert!(!is_fame_active(1_199, 2_000) && is_fame_active(1_200, 2_000));
        assert!(!is_fame_active(3_899, 6_500) && is_fame_active(3_900, 6_500));
        assert_eq!(stolen_loot_cash(100), Ok(125));
        assert_eq!(stolen_loot_cash(200), Ok(250));
        assert_eq!(stolen_loot_cash(400), Ok(500));
        assert_eq!(stolen_loot_cash(u32::MAX), Err(EconomyError::Overflow));

        let mut econ = Economy::new(1).unwrap();
        let out1 = econ.deposit_cash(Team::Cobalt, 1_200, false).unwrap();
        assert!(out1.fame_newly_activated && !out1.goal_reached && econ.winner().is_none());
        let out2 = econ.deposit_cash(Team::Cobalt, 800, false).unwrap();
        assert!(!out2.fame_newly_activated && out2.goal_reached);
        assert_eq!(econ.winner(), Some(Team::Cobalt));
    }

    #[test]
    fn bank_overflow_does_not_panic_or_corrupt_balance() {
        let mut econ = Economy::new(4).unwrap();
        let err = Err(EconomyError::Overflow);
        econ.set_bank_unchecked(Team::Copper, u32::MAX - 50);
        assert_eq!(econ.deposit_cash(Team::Copper, 100, false), err);
        assert_eq!(econ.bank(Team::Copper), u32::MAX - 50);
        econ.mine_gold(1, Team::Copper, 1).unwrap();
        assert_eq!(econ.deposit_player(1), err);
        assert_eq!(econ.bank(Team::Copper), u32::MAX - 50);
        assert_eq!(econ.player_load(1).unwrap().own_gold, 1);
    }
}
