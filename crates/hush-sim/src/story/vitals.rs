//! Fisiología del minero: salud, nutrición, respiración y traumatismos.
//!
//! Todos los contadores son enteros y avanzan a 20 ticks por segundo.

use grisu_voxel::FIXED_ONE;
use serde::{Deserialize, Serialize};

/// Salud base: 20 puntos (10 corazones).
pub const MAX_HEALTH: u8 = 20;
/// Apetito máximo: 20 puntos (10 muslos).
pub const MAX_FOOD: u8 = 20;
/// Saturación máxima (variable oculta que se gasta antes que el apetito).
pub const MAX_SATURATION: u8 = 20;
/// Depósito de aire: 300 ticks (15 s) bajo el agua.
pub const MAX_AIR: u16 = 300;
/// Cada 4.0 puntos de fatiga consumen 1 punto de saturación o hambre.
pub const FATIGUE_PER_FOOD: u32 = 4 * 256;
/// Con nutrición >= 18, 1 punto de vida cada 4 segundos (80 ticks).
pub const REGEN_INTERVAL_TICKS: u32 = 80;
/// Inanición y asfixia: daño periódico cada 20 ticks (1 s).
pub const DAMAGE_INTERVAL_TICKS: u32 = 20;
/// Daño de asfixia por segundo.
pub const DROWNING_DAMAGE: u8 = 2;
/// La caída perdona los primeros 3 bloques.
pub const FALL_SAFE_BLOCKS: i32 = 3;
/// Fatiga (Q8) acumulada por acción: correr 0.1/tick, saltar 0.2, minar
/// roca compacta 0.05 y andar 0.005/tick.
pub const FATIGUE_SPRINT: u32 = 26;
pub const FATIGUE_JUMP: u32 = 51;
pub const FATIGUE_DIG: u32 = 13;
pub const FATIGUE_WALK: u32 = 1;

/// Estado vital del minero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vitals {
    pub health: u8,
    pub food: u8,
    /// Saturación oculta: se consume antes que el apetito.
    pub saturation: u8,
    /// Fatiga metabólica acumulada en Q8.
    pub fatigue: u32,
    /// Aire restante bajo el agua, en ticks.
    pub air: u16,
    /// Submerged: la cabeza está en un fluido este tick.
    pub submerged: bool,
    /// Altura (en punto fijo) desde la que started la caída actual.
    fall_peak: i64,
    falling: bool,
    regen_timer: u32,
    /// Temporizadores independientes de asfixia e inanición.
    drown_timer: u32,
    starve_timer: u32,
}

impl Default for Vitals {
    fn default() -> Self {
        Self {
            health: MAX_HEALTH,
            food: MAX_FOOD,
            saturation: 5,
            fatigue: 0,
            air: MAX_AIR,
            submerged: false,
            fall_peak: 0,
            falling: false,
            regen_timer: 0,
            drown_timer: 0,
            starve_timer: 0,
        }
    }
}

impl Vitals {
    /// Registra fatiga por acción (andar, correr, saltar, minar).
    pub fn add_fatigue(&mut self, amount: u32) {
        self.fatigue = self.fatigue.saturating_add(amount);
    }

    /// Come: `food` puntos de nutrición y `saturation` de saturación.
    pub fn eat(&mut self, food: u8, saturation: u8) {
        self.food = self.food.saturating_add(food).min(MAX_FOOD);
        self.saturation = self
            .saturation
            .saturating_add(saturation)
            .min(MAX_SATURATION);
    }

    /// Aplica daño y devuelve `true` si el minero ha desfallecido.
    pub fn damage(&mut self, amount: u8) -> bool {
        self.health = self.health.saturating_sub(amount.min(self.health));
        self.health == 0
    }

    /// Olvida la caída en curso (tras un teletransporte).
    pub fn stop_fall(&mut self) {
        self.falling = false;
        self.fall_peak = 0;
    }

    /// Daño al aterrizar tras una caída: `max(0, distancia - 3)`.
    /// Devuelve el daño aplicado (y si el minero muere).
    pub fn land(&mut self, feet_y: i64, on_ground: bool, in_fluid: bool) -> (u8, bool) {
        if in_fluid || !self.falling {
            self.falling = false;
            return (0, false);
        }
        if !on_ground {
            return (0, false);
        }
        self.falling = false;
        let fall_blocks = (self.fall_peak - feet_y) / FIXED_ONE;
        // Acotado: una caída enorme no da la vuelta al convertir a `u8`.
        let damage = (fall_blocks - i64::from(FALL_SAFE_BLOCKS)).clamp(0, 255) as u8;
        if damage == 0 {
            return (0, false);
        }
        let dead = self.damage(damage);
        (damage, dead)
    }

    /// Actualiza picos de caída: llamar cada tick antes de mover.
    pub fn pre_move(&mut self, feet_y: i64, on_ground: bool) {
        if !on_ground && !self.falling {
            self.falling = true;
            self.fall_peak = feet_y;
        } else if self.falling {
            self.fall_peak = self.fall_peak.max(feet_y);
        }
        if on_ground {
            self.falling = false;
        }
    }

    /// Avanza un tick de fisiología. `head_submerged` indica si la cabeza
    /// está bajo un fluido. Devuelve el daño recibido este tick y si el
    /// minero ha muerto.
    pub fn step(&mut self, head_submerged: bool) -> (u8, bool) {
        self.submerged = head_submerged;
        let mut total_damage: u8 = 0;
        let mut dead = false;

        // Metabolismo: la fatiga se convierte en hambre.
        while self.fatigue >= FATIGUE_PER_FOOD {
            self.fatigue -= FATIGUE_PER_FOOD;
            if self.saturation > 0 {
                self.saturation -= 1;
            } else if self.food > 0 {
                self.food -= 1;
            }
        }

        // Respiración subacuática.
        if head_submerged {
            self.air = self.air.saturating_sub(1);
            if self.air == 0 {
                self.drown_timer += 1;
                if self.drown_timer >= DAMAGE_INTERVAL_TICKS {
                    self.drown_timer = 0;
                    total_damage += DROWNING_DAMAGE;
                    dead |= self.damage(DROWNING_DAMAGE);
                }
            }
        } else {
            self.air = (self.air + 16).min(MAX_AIR);
            self.drown_timer = 0;
        }

        // Inanición.
        if self.food == 0 {
            self.starve_timer += 1;
            if self.starve_timer >= DAMAGE_INTERVAL_TICKS {
                self.starve_timer = 0;
                total_damage += 1;
                dead |= self.damage(1);
            }
        } else {
            self.starve_timer = 0;
        }

        // Regeneración natural con buena nutrición.
        if self.food >= 18 && self.health < MAX_HEALTH {
            self.regen_timer += 1;
            if self.regen_timer >= REGEN_INTERVAL_TICKS {
                self.regen_timer = 0;
                self.health += 1;
                // Recuperar cuesta energía.
                self.add_fatigue(6 * 256);
            }
        } else {
            self.regen_timer = 0;
        }
        (total_damage, dead)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regenerates_with_good_nutrition_every_four_seconds() {
        let mut v = Vitals {
            health: 15,
            food: 20,
            saturation: 20,
            ..Vitals::default()
        };
        for _ in 0..REGEN_INTERVAL_TICKS {
            v.step(false);
        }
        assert_eq!(v.health, 16);
        // La regeneración consume reservas.
        assert!(v.fatigue > 0 || v.saturation < 20);
    }

    #[test]
    fn fatigue_drains_saturation_then_food() {
        let mut v = Vitals::default();
        v.add_fatigue(FATIGUE_PER_FOOD);
        v.step(false);
        // La saturación inicial es 5: el primer punto la gasta.
        assert_eq!((v.food, v.saturation), (20, 4));
        for _ in 0..4 {
            v.add_fatigue(FATIGUE_PER_FOOD);
        }
        v.step(false);
        // Los cuatro siguientes agotan la saturación; el apetito intacto.
        assert_eq!((v.food, v.saturation), (20, 0));
        v.add_fatigue(FATIGUE_PER_FOOD);
        v.step(false);
        assert_eq!(v.food, 19);
    }

    #[test]
    fn starvation_damages_every_second_until_death() {
        let mut v = Vitals {
            food: 0,
            saturation: 0,
            ..Vitals::default()
        };
        // 20 puntos de vida, 1 por segundo.
        let mut ticks = 0;
        while !v.step(false).1 {
            ticks += 1;
            assert!(ticks < 500, "nunca murió");
        }
        assert_eq!(v.health, 0);
        // El paso letal es el 400.º: el contador guarda los no letales.
        assert_eq!(ticks + 1, u32::from(MAX_HEALTH) * DAMAGE_INTERVAL_TICKS);
    }

    #[test]
    fn fall_damage_forgives_three_blocks() {
        let mut v = Vitals::default();
        let one = FIXED_ONE;
        v.pre_move(80 * one, false);
        v.pre_move(77 * one, false);
        let (damage, _) = v.land(77 * one, true, false);
        assert_eq!(damage, 0, "3 bloques se perdonan");
        v.pre_move(80 * one, false);
        let (damage, _) = v.land(75 * one, true, false);
        assert_eq!(damage, 2, "5 bloques = 2 de daño");
        // En fluido no hay daño.
        v.pre_move(80 * one, false);
        let (damage, _) = v.land(60 * one, true, true);
        assert_eq!(damage, 0);
    }

    #[test]
    fn drowning_depletes_air_in_fifteen_seconds() {
        let mut v = Vitals::default();
        let mut ticks = 0;
        while v.air > 0 {
            v.step(true);
            ticks += 1;
        }
        assert_eq!(ticks, MAX_AIR as u32);
        let health_before = v.health;
        for _ in 0..DAMAGE_INTERVAL_TICKS {
            v.step(true);
        }
        assert_eq!(v.health, health_before - DROWNING_DAMAGE);
        // Al salir, el aire se recupera.
        for _ in 0..30 {
            v.step(false);
        }
        assert_eq!(v.air, MAX_AIR);
    }
}
