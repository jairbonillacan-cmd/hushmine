//! Bolsas de botín soltadas al morir, caducidad determinista y recogida con
//! multiplicador de robo.
//!
//! Al morir con recursos, el jugador suelta una [`LootBag`] en su [`GridPos`]
//! activa durante [`LOOT_BAG_LIFETIME_TICKS`] ticks (`60 s` a `30 Hz`). El
//! mismo equipo recupera su valor base; el rival aplica `×5/4` al liquidar.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::Team;
use crate::economy::{
    DIAMOND_UNIT_CASH_VALUE, GOLD_CASH_VALUE, STOLEN_LOOT_MULTIPLIER_DEN,
    STOLEN_LOOT_MULTIPLIER_NUM,
};

/// Duración de una bolsa de botín en el suelo antes de caducar (`60 s` a `30 Hz`).
pub const LOOT_BAG_LIFETIME_TICKS: u32 = 1_800; // Se moverá a data/*.ron
/// Numerador de la fracción del inventario que cae al suelo al morir (`100 %`).
pub const LOOT_BAG_DROP_NUM: u32 = 1; // Se moverá a data/*.ron
/// Denominador de la fracción del inventario que cae al suelo al morir (`100 %`).
pub const LOOT_BAG_DROP_DEN: u32 = 1; // Se moverá a data/*.ron

/// Coordenada entera de celda en la cuadrícula tridimensional de vóxeles.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GridPos {
    /// Construye una coordenada entera de celda `(x, y, z)`.
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

/// Recursos almacenados dentro de una bolsa de botín.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootContents {
    pub ore: u32,
    pub gold: u32,
    pub diamond: u32,
}

impl LootContents {
    /// Construye el contenido de una bolsa con las cantidades de mena, oro y diamante indicadas.
    pub const fn new(ore: u32, gold: u32, diamond: u32) -> Self {
        Self { ore, gold, diamond }
    }

    /// Indica si la bolsa no contiene ningún recurso.
    pub const fn is_empty(&self) -> bool {
        self.ore == 0 && self.gold == 0 && self.diamond == 0
    }

    /// Calcula los recursos que caen en la bolsa a partir del inventario portado al morir.
    pub const fn dropped_from(ore: u32, gold: u32, diamond: u32) -> Self {
        Self {
            ore: ore.saturating_mul(LOOT_BAG_DROP_NUM) / LOOT_BAG_DROP_DEN,
            gold: gold.saturating_mul(LOOT_BAG_DROP_NUM) / LOOT_BAG_DROP_DEN,
            diamond: diamond.saturating_mul(LOOT_BAG_DROP_NUM) / LOOT_BAG_DROP_DEN,
        }
    }

    /// Calcula el valor monetario base (`$`) del oro y los diamantes contenidos.
    pub const fn base_cash_value(&self) -> Option<u32> {
        let Some(gold_cash) = self.gold.checked_mul(GOLD_CASH_VALUE) else {
            return None;
        };
        let Some(diamond_cash) = self.diamond.checked_mul(DIAMOND_UNIT_CASH_VALUE) else {
            return None;
        };
        gold_cash.checked_add(diamond_cash)
    }

    /// Calcula el valor monetario efectivo (`$`) según el equipo de origen y el equipo que recoge.
    pub const fn cash_value_for_team(&self, origin_team: Team, picker_team: Team) -> Option<u32> {
        let Some(base) = self.base_cash_value() else {
            return None;
        };
        if same_team(origin_team, picker_team) {
            Some(base)
        } else {
            let Some(scaled) = base.checked_mul(STOLEN_LOOT_MULTIPLIER_NUM) else {
                return None;
            };
            Some(scaled / STOLEN_LOOT_MULTIPLIER_DEN)
        }
    }
}

const fn same_team(a: Team, b: Team) -> bool {
    matches!(
        (a, b),
        (Team::Copper, Team::Copper) | (Team::Cobalt, Team::Cobalt)
    )
}

/// Bolsa física soltada en el mundo al morir un jugador.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootBag {
    pub id: u32,
    pub position: GridPos,
    pub contents: LootContents,
    pub origin_team: Team,
    pub spawned_at_tick: u32,
    pub expires_at_tick: u32,
}

impl LootBag {
    /// Indica si la bolsa ya ha caducado en el `tick` indicado (`tick >= expires_at_tick`).
    pub const fn is_expired_at(&self, tick: u32) -> bool {
        tick >= self.expires_at_tick
    }

    /// Devuelve el valor monetario base (`$`) del contenido de la bolsa.
    pub const fn base_cash_value(&self) -> Option<u32> {
        self.contents.base_cash_value()
    }

    /// Devuelve el valor monetario efectivo (`$`) si la bolsa es recogida por `picker_team`.
    pub const fn cash_value_for(&self, picker_team: Team) -> Option<u32> {
        self.contents
            .cash_value_for_team(self.origin_team, picker_team)
    }
}

/// Error devuelto al intentar soltar o recoger una bolsa de botín.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LootError {
    /// El inventario estaba vacío, por lo que no se genera bolsa al morir.
    EmptyContents,
    /// No existe ninguna bolsa activa con ese identificador (ya se recogió o caducó).
    BagNotFound(u32),
    /// La bolsa ya ha caducado en el tick indicado.
    BagExpired { bag_id: u32, expired_at_tick: u32 },
    /// El valor monetario de la bolsa desborda `u32`.
    Overflow,
}

impl fmt::Display for LootError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyContents => write!(f, "inventario vacío: no se genera bolsa"),
            Self::BagNotFound(id) => write!(f, "no existe bolsa activa con id {id}"),
            Self::BagExpired {
                bag_id,
                expired_at_tick,
            } => write!(f, "la bolsa {bag_id} caducó en el tick {expired_at_tick}"),
            Self::Overflow => write!(f, "el valor monetario de la bolsa desborda u32"),
        }
    }
}

impl std::error::Error for LootError {}

/// Recibo generado cuando un equipo recoge una bolsa de botín activa.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootPickup {
    pub bag: LootBag,
    pub picker_team: Team,
    pub is_stolen: bool,
    pub base_cash_value: u32,
    pub effective_cash_value: u32,
}

/// Almacén determinista de bolsas de botín activas indexado por identificador ascendente.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LootStore {
    bags: BTreeMap<u32, LootBag>,
    next_id: u32,
}

impl Default for LootStore {
    fn default() -> Self {
        Self::new()
    }
}

impl LootStore {
    /// Crea un almacén de bolsas vacío con contador inicial en `1`.
    pub const fn new() -> Self {
        Self {
            bags: BTreeMap::new(),
            next_id: 1,
        }
    }

    /// Devuelve el número de bolsas activas en el almacén.
    pub fn len(&self) -> usize {
        self.bags.len()
    }

    /// Indica si no hay ninguna bolsa activa en el almacén.
    pub fn is_empty(&self) -> bool {
        self.bags.is_empty()
    }

    /// Busca una bolsa activa por su identificador.
    pub fn get(&self, bag_id: u32) -> Option<&LootBag> {
        self.bags.get(&bag_id)
    }

    /// Devuelve el mapa determinista de bolsas activas ordenadas por identificador.
    pub fn bags(&self) -> &BTreeMap<u32, LootBag> {
        &self.bags
    }

    /// Suma el valor monetario base (`$`) de todas las bolsas activas.
    pub fn total_active_base_cash(&self) -> Option<u32> {
        let mut total = 0u32;
        for bag in self.bags.values() {
            total = total.checked_add(bag.base_cash_value()?)?;
        }
        Some(total)
    }

    /// Crea e inserta una nueva bolsa de botín aplicando la fracción de caída configurada.
    pub fn drop_bag(
        &mut self,
        tick: u32,
        position: GridPos,
        origin_team: Team,
        contents: LootContents,
    ) -> Result<LootBag, LootError> {
        let dropped = LootContents::dropped_from(contents.ore, contents.gold, contents.diamond);
        if dropped.is_empty() {
            return Err(LootError::EmptyContents);
        }
        if dropped.base_cash_value().is_none() {
            return Err(LootError::Overflow);
        }

        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        let bag = LootBag {
            id,
            position,
            contents: dropped,
            origin_team,
            spawned_at_tick: tick,
            expires_at_tick: tick.saturating_add(LOOT_BAG_LIFETIME_TICKS),
        };
        self.bags.insert(id, bag.clone());
        Ok(bag)
    }

    /// Recoge y retira del mundo una bolsa activa antes de que caduque.
    pub fn pickup(
        &mut self,
        tick: u32,
        bag_id: u32,
        picker_team: Team,
    ) -> Result<LootPickup, LootError> {
        let bag_ref = self
            .bags
            .get(&bag_id)
            .ok_or(LootError::BagNotFound(bag_id))?;
        if bag_ref.is_expired_at(tick) {
            return Err(LootError::BagExpired {
                bag_id,
                expired_at_tick: bag_ref.expires_at_tick,
            });
        }
        let base_cash_value = bag_ref.base_cash_value().ok_or(LootError::Overflow)?;
        let effective_cash_value = bag_ref
            .cash_value_for(picker_team)
            .ok_or(LootError::Overflow)?;

        let Some(bag) = self.bags.remove(&bag_id) else {
            return Err(LootError::BagNotFound(bag_id));
        };
        let is_stolen = picker_team != bag.origin_team;

        Ok(LootPickup {
            bag,
            picker_team,
            is_stolen,
            base_cash_value,
            effective_cash_value,
        })
    }

    /// Retira en orden ascendente de identificador todas las bolsas que hayan caducado en `tick`.
    pub fn expire_due_bags(&mut self, tick: u32) -> Vec<LootBag> {
        let expired_ids: Vec<u32> = self
            .bags
            .iter()
            .filter_map(|(&id, bag)| bag.is_expired_at(tick).then_some(id))
            .collect();
        let mut expired = Vec::with_capacity(expired_ids.len());
        for id in expired_ids {
            if let Some(bag) = self.bags.remove(&id) {
                expired.push(bag);
            }
        }
        expired
    }

    /// Inserta una bolsa arbitraria sin validar invariantes (auxiliar para pruebas).
    pub fn insert_bag_unchecked(&mut self, bag: LootBag) {
        if bag.id >= self.next_id {
            self.next_id = bag.id.saturating_add(1);
        }
        self.bags.insert(bag.id, bag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn(store: &mut LootStore, tick: u32, gold: u32, diamond: u32) -> LootBag {
        let contents = LootContents::new(0, gold, diamond);
        store
            .drop_bag(tick, GridPos::new(1, 2, 3), Team::Copper, contents)
            .unwrap()
    }

    #[test]
    fn bag_is_active_until_lifetime_and_expires_at_exact_tick() {
        let mut store = LootStore::new();
        let spawn_tick = 120;
        let bag = spawn(&mut store, spawn_tick, 1, 0);

        assert_eq!(bag.id, 1);
        assert_eq!(bag.expires_at_tick, spawn_tick + LOOT_BAG_LIFETIME_TICKS);

        let before_expiry = spawn_tick + LOOT_BAG_LIFETIME_TICKS - 1;
        assert!(!bag.is_expired_at(before_expiry));
        assert!(store.expire_due_bags(before_expiry).is_empty());
        assert_eq!(store.len(), 1);

        let at_expiry = spawn_tick + LOOT_BAG_LIFETIME_TICKS;
        assert!(bag.is_expired_at(at_expiry));
        assert_eq!(
            store.pickup(at_expiry, bag.id, Team::Copper),
            Err(LootError::BagExpired {
                bag_id: bag.id,
                expired_at_tick: at_expiry,
            })
        );
        assert_eq!(store.expire_due_bags(at_expiry), vec![bag]);
        assert!(store.is_empty());
    }

    #[test]
    fn bag_can_only_be_picked_up_once() {
        let mut store = LootStore::new();
        let bag = spawn(&mut store, 10, 1, 1);

        let first = store.pickup(20, bag.id, Team::Copper).unwrap();
        assert_eq!(first.bag.id, bag.id);
        assert_eq!(
            store.pickup(20, bag.id, Team::Cobalt),
            Err(LootError::BagNotFound(bag.id))
        );
    }

    #[test]
    fn same_team_keeps_base_value_and_rival_gets_stolen_multiplier() {
        let mut store = LootStore::new();
        let gold_bag = spawn(&mut store, 50, 1, 0);
        let diamond_bag = spawn(&mut store, 50, 0, 1);
        let ally_bag = spawn(&mut store, 50, 2, 0);

        let stolen_gold = store.pickup(60, gold_bag.id, Team::Cobalt).unwrap();
        assert!(stolen_gold.is_stolen);
        assert_eq!(stolen_gold.base_cash_value, 100);
        assert_eq!(stolen_gold.effective_cash_value, 125);

        let stolen_diamond = store.pickup(60, diamond_bag.id, Team::Cobalt).unwrap();
        assert!(stolen_diamond.is_stolen);
        assert_eq!(stolen_diamond.base_cash_value, 200);
        assert_eq!(stolen_diamond.effective_cash_value, 250);

        let recovered_ally = store.pickup(60, ally_bag.id, Team::Copper).unwrap();
        assert!(!recovered_ally.is_stolen);
        assert_eq!(recovered_ally.base_cash_value, 200);
        assert_eq!(recovered_ally.effective_cash_value, 200);
    }

    #[test]
    fn empty_inventory_does_not_spawn_bag() {
        let mut store = LootStore::new();
        let pos = GridPos::new(0, 0, 0);
        let res = store.drop_bag(0, pos, Team::Cobalt, LootContents::new(0, 0, 0));
        assert_eq!(res, Err(LootError::EmptyContents));
        assert!(store.is_empty());
    }
}
