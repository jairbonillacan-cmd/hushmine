//! Verificador determinista de invariantes formales de la simulación (`hush-sim`).
//!
//! Implementa funciones puras que inspeccionan el estado del Sismógrafo, la
//! economía, las bolsas de botín y el [`EventLog`], devolviendo las violaciones
//! detectadas con el identificador formal de `docs/design/invariantes.md`.

use crate::economy::{Economy, is_fame_active, stolen_loot_cash};
use crate::events::{EventLog, MatchEventKind, Team};
use crate::loot::{LOOT_BAG_LIFETIME_TICKS, LootStore};
use crate::seismograph::{
    AWAKENING_BUCKET_STOP_REMAINING_TICKS, AWAKENING_DURATION_TICKS, MAX_LEVEL,
    PASSIVE_TICKS_TO_WAKE, SeismoThreshold, Seismograph,
};

/// Ejecuta todas las comprobaciones de invariantes en orden y devuelve el vector
/// con las violaciones detectadas (vacío si todo el estado es válido).
pub fn check_all(
    tick: u32,
    seismograph: &Seismograph,
    economy: &Economy,
    loot: &LootStore,
    event_log: &EventLog,
) -> Vec<String> {
    let mut broken = check_gold_conservation(economy, loot, event_log);
    broken.extend(check_seismograph_bounds(seismograph));
    broken.extend(check_thresholds_unique_and_monotonic(
        seismograph,
        event_log,
    ));
    broken.extend(check_wake_immutability(seismograph, event_log));
    broken.extend(check_shift_duration(tick, event_log));
    broken.extend(check_banks_and_fame(economy, loot, event_log));
    broken
}

/// Verifica la conservación contable cerrada del oro y diamantes (`INV-01`) y su
/// coherencia estricta con los eventos registrados en [`EventLog`].
pub fn check_gold_conservation(
    economy: &Economy,
    loot: &LootStore,
    event_log: &EventLog,
) -> Vec<String> {
    let mut errors = Vec::new();
    let carried_base = economy.total_carried_base_cash().ok();
    if carried_base.is_none() {
        errors.push("INV-01: desbordamiento en total_carried_base_cash".to_string());
    }
    let active_bags_base = loot.total_active_base_cash();
    if active_bags_base.is_none() {
        errors.push("INV-01: desbordamiento en total_active_base_cash".to_string());
    }

    let banks_and_offered = u64::from(economy.bank(Team::Copper))
        + u64::from(economy.bank(Team::Cobalt))
        + u64::from(economy.total_offered_cash());

    if let (Some(carried), Some(bags)) = (carried_base, active_bags_base) {
        let lhs = u64::from(carried)
            + u64::from(bags)
            + banks_and_offered
            + u64::from(economy.total_expired_bag_cash());
        let rhs =
            u64::from(economy.total_mined_cash()) + u64::from(economy.total_stolen_bonus_cash());
        if lhs != rhs {
            errors.push(format!(
                "INV-01: balance de conservación de oro descuadrado ({lhs} != {rhs})"
            ));
        }
    }

    let [
        mut mined,
        mut dep_base,
        mut dep_cred,
        mut bonus,
        mut expired,
    ] = [0u64; 5];
    for event in event_log.events() {
        match event.kind {
            MatchEventKind::GoldMined { cash_value, .. } => mined += u64::from(cash_value),
            MatchEventKind::Deposited {
                base_cash,
                credited_cash,
                ..
            } => {
                dep_base += u64::from(base_cash);
                dep_cred += u64::from(credited_cash);
                bonus += u64::from(credited_cash.saturating_sub(base_cash));
            }
            MatchEventKind::LootExpired { cash_value, .. } => expired += u64::from(cash_value),
            _ => {}
        }
    }

    if mined != u64::from(economy.total_mined_cash()) {
        errors.push(format!("INV-01: GoldMined ({mined}) != total_mined_cash"));
    }
    if dep_base != u64::from(economy.total_deposited_base_cash()) {
        errors.push(format!("INV-01: base_cash ({dep_base}) != depositado"));
    }
    if bonus != u64::from(economy.total_stolen_bonus_cash()) {
        errors.push(format!(
            "INV-01: prima ({bonus}) != total_stolen_bonus_cash"
        ));
    }
    if expired != u64::from(economy.total_expired_bag_cash()) {
        errors.push(format!("INV-01: LootExpired ({expired}) != expirado"));
    }
    if economy.is_active() && dep_cred != banks_and_offered {
        errors.push(format!("INV-01: cred ({dep_cred}) != {banks_and_offered}"));
    }

    for (&bag_id, bag) in loot.bags() {
        let Some(expected_cash) = bag.base_cash_value() else {
            errors.push(format!(
                "INV-01: bolsa {bag_id} desborda en base_cash_value"
            ));
            continue;
        };
        let mut dropped_cash = None;
        let mut closed = false;
        for event in event_log.events() {
            match event.kind {
                MatchEventKind::LootDropped {
                    bag_id: id,
                    cash_value,
                } if id == bag_id => dropped_cash = Some(cash_value),
                MatchEventKind::LootPicked { bag_id: id, .. }
                | MatchEventKind::LootExpired { bag_id: id, .. }
                    if id == bag_id =>
                {
                    closed = true;
                }
                _ => {}
            }
        }
        if dropped_cash != Some(expected_cash) {
            errors.push(format!("INV-01: bolsa {bag_id} sin LootDropped coherente"));
        }
        if closed {
            errors.push(format!("INV-01: bolsa {bag_id} ya cerrada en EventLog"));
        }
    }
    errors
}

/// Verifica la acotación del Sismógrafo en `[0, MAX_LEVEL]` y su resto pasivo (`INV-03`).
pub fn check_seismograph_bounds(seismograph: &Seismograph) -> Vec<String> {
    let mut errors = Vec::new();
    if seismograph.level() > MAX_LEVEL {
        errors.push(format!(
            "INV-03: nivel ({}) > MAX_LEVEL",
            seismograph.level()
        ));
    }
    if seismograph.passive_remainder() >= PASSIVE_TICKS_TO_WAKE {
        errors.push(format!(
            "INV-03: passive_remainder ({}) >= {PASSIVE_TICKS_TO_WAKE}",
            seismograph.passive_remainder()
        ));
    }
    errors
}

/// Verifica que los umbrales sísmicos se emitan una sola vez, en orden estricto
/// `[Stir, Breathe, Restless, Wake]` y con ticks monótonos (`INV-05`).
pub fn check_thresholds_unique_and_monotonic(
    seismograph: &Seismograph,
    event_log: &EventLog,
) -> Vec<String> {
    let mut errors = Vec::new();
    let crossed: Vec<(u32, SeismoThreshold)> = event_log
        .events()
        .iter()
        .filter_map(|e| match e.kind {
            MatchEventKind::ThresholdCrossed(t) => Some((e.tick, t)),
            _ => None,
        })
        .collect();

    if crossed.len() > SeismoThreshold::ALL.len() {
        errors.push(format!("INV-05: > 4 umbrales emitidos ({})", crossed.len()));
    }
    for (idx, &(_, threshold)) in crossed.iter().enumerate() {
        if crossed[..idx].iter().any(|&(_, prev)| prev == threshold) {
            errors.push(format!("INV-05: umbral sísmico duplicado ({threshold:?})"));
        }
        if let Some(&expected) = SeismoThreshold::ALL.get(idx)
            && threshold != expected
        {
            errors.push(format!(
                "INV-05: umbral desordenado en {idx} ({threshold:?} != {expected:?})"
            ));
        }
    }
    for pair in crossed.windows(2) {
        if pair[0].0 > pair[1].0 {
            errors.push("INV-05: ticks de umbrales decrecientes".to_string());
        }
    }
    if seismograph.has_awakened() && !crossed.iter().any(|&(_, t)| t == SeismoThreshold::Wake) {
        errors.push("INV-05: Sismógrafo despertó sin emitir Wake".to_string());
    }
    errors
}

/// Verifica la inmutabilidad del Sismógrafo en `MAX_LEVEL` durante el Despertar,
/// la parada de líneas con `<= 900` ticks restantes y el cierre del altar (`INV-06`).
pub fn check_wake_immutability(seismograph: &Seismograph, event_log: &EventLog) -> Vec<String> {
    let mut errors = Vec::new();
    if seismograph.has_awakened() {
        if seismograph.level() != MAX_LEVEL {
            errors.push(format!(
                "INV-06: nivel en Despertar ({}) != {MAX_LEVEL}",
                seismograph.level()
            ));
        }
        if let Some(rem) = seismograph.awakening_ticks_remaining()
            && rem <= AWAKENING_BUCKET_STOP_REMAINING_TICKS
            && !seismograph.are_bucket_lines_stopped()
        {
            errors.push(format!("INV-06: líneas activas quedando {rem} ticks"));
        }
    }

    let wake_tick = event_log.events().iter().find_map(|e| match e.kind {
        MatchEventKind::AwakeningStarted => Some(e.tick),
        _ => None,
    });
    if let Some(wt) = wake_tick {
        for event in event_log.events() {
            if matches!(
                event.kind,
                MatchEventKind::OfferingMade { .. } | MatchEventKind::TallyReturned { .. }
            ) && event.tick > wt
            {
                errors.push(format!(
                    "INV-06: alivio en tick {} > AwakeningStarted ({wt})",
                    event.tick
                ));
            }
        }
    }
    errors
}

/// Verifica que la duración del Turno no supere `31_500` ticks y que la cronología
/// del [`EventLog`] sea monótona y no supere el tick actual (`INV-07`).
pub fn check_shift_duration(tick: u32, event_log: &EventLog) -> Vec<String> {
    let mut errors = Vec::new();
    let max_shift_ticks = PASSIVE_TICKS_TO_WAKE + AWAKENING_DURATION_TICKS;
    if tick > max_shift_ticks {
        errors.push(format!(
            "INV-07: tick ({tick}) > máximo ({max_shift_ticks})"
        ));
    }
    let mut prev_tick: Option<u32> = None;
    for event in event_log.events() {
        if event.tick > tick {
            errors.push("INV-07: evento con tick futuro".to_string());
        }
        if let Some(prev) = prev_tick
            && event.tick < prev
        {
            errors.push("INV-07: eventos desordenados en EventLog".to_string());
        }
        prev_tick = Some(event.tick);
    }
    errors
}

/// Verifica la ausencia de desbordamientos en bancos y cargas (`INV-08`), la
/// validez de bolsas activas (`INV-11`) y las reglas enteras de Fama y robo (`INV-12`).
pub fn check_banks_and_fame(
    economy: &Economy,
    loot: &LootStore,
    event_log: &EventLog,
) -> Vec<String> {
    let mut errors = Vec::new();
    if economy.target_cash() == 0 {
        errors.push("INV-08: target_cash no puede ser 0".to_string());
    }
    let banks_sum = economy
        .bank(Team::Copper)
        .checked_add(economy.bank(Team::Cobalt));
    let carried_res = economy.total_carried_base_cash().ok();
    match (banks_sum, carried_res) {
        (Some(b), Some(c)) if b.checked_add(c).is_some() => {}
        _ => errors.push("INV-08: desbordamiento en suma de bancos y cargas".to_string()),
    }

    for (&bag_id, bag) in loot.bags() {
        if bag.contents.is_empty() {
            errors.push(format!("INV-11: la bolsa activa {bag_id} está vacía"));
        }
        let expected_exp = bag.spawned_at_tick.saturating_add(LOOT_BAG_LIFETIME_TICKS);
        if bag.expires_at_tick != expected_exp {
            errors.push(format!("INV-11: caducidad inválida en bolsa {bag_id}"));
        }
    }

    for team in [Team::Copper, Team::Cobalt] {
        if economy.has_fame(team) != is_fame_active(economy.bank(team), economy.target_cash()) {
            errors.push(format!("INV-12: Fama incoherente para {team:?}"));
        }
    }
    for event in event_log.events() {
        if let MatchEventKind::Deposited {
            base_cash,
            credited_cash,
            ..
        } = event.kind
        {
            let max_cred = stolen_loot_cash(base_cash).ok();
            if credited_cash < base_cash || Some(credited_cash) > max_cred {
                errors.push(format!(
                    "INV-12: Deposited inválido (base={base_cash}, credited={credited_cash})"
                ));
            }
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loot::{GridPos, LootBag, LootContents};

    #[test]
    fn clean_initial_state_has_no_broken_invariants() {
        let seismo = Seismograph::new();
        let economy = Economy::with_default_team_size();
        let loot = LootStore::new();
        let log = EventLog::new();
        assert!(check_all(0, &seismo, &economy, &loot, &log).is_empty());
    }

    #[test]
    fn tampering_bank_without_events_breaks_inv_01() {
        let seismo = Seismograph::new();
        let mut economy = Economy::with_default_team_size();
        economy.set_bank_unchecked(Team::Copper, 500);
        let broken = check_all(10, &seismo, &economy, &LootStore::new(), &EventLog::new());
        assert!(broken.iter().any(|msg| msg.contains("INV-01")));
    }

    #[test]
    fn duplicate_or_out_of_order_thresholds_break_inv_05() {
        let mut log = EventLog::new();
        let kind = MatchEventKind::ThresholdCrossed(SeismoThreshold::Breathe);
        log.record(100, kind.clone(), None);
        log.record(200, kind, None);
        let broken = check_thresholds_unique_and_monotonic(&Seismograph::new(), &log);
        assert!(broken.iter().any(|msg| msg.contains("INV-05")));
    }

    #[test]
    fn tick_exceeding_31500_breaks_inv_07() {
        let broken = check_shift_duration(31_501, &EventLog::new());
        assert!(broken.iter().any(|msg| msg.contains("INV-07")));
    }

    #[test]
    fn empty_active_loot_bag_breaks_inv_11() {
        let mut loot = LootStore::new();
        loot.insert_bag_unchecked(LootBag {
            id: 1,
            position: GridPos::new(0, 0, 0),
            contents: LootContents::new(0, 0, 0),
            origin_team: Team::Copper,
            spawned_at_tick: 10,
            expires_at_tick: 10 + LOOT_BAG_LIFETIME_TICKS,
        });
        let econ = Economy::with_default_team_size();
        let broken = check_banks_and_fame(&econ, &loot, &EventLog::new());
        assert!(broken.iter().any(|msg| msg.contains("INV-11")));
    }

    #[test]
    fn invalid_deposit_credit_or_bank_overflow_breaks_inv_12_and_inv_08() {
        let mut economy = Economy::with_default_team_size();
        let mut log = EventLog::new();
        economy.set_bank_unchecked(Team::Copper, u32::MAX - 10);
        economy.set_bank_unchecked(Team::Cobalt, 100);
        let bad_dep = MatchEventKind::Deposited {
            player_id: 1,
            base_cash: 200,
            credited_cash: 150,
            new_balance: 150,
        };
        log.record(20, bad_dep, Some(Team::Copper));
        let broken = check_banks_and_fame(&economy, &LootStore::new(), &log);
        assert!(broken.iter().any(|msg| msg.contains("INV-08")));
        assert!(broken.iter().any(|msg| msg.contains("INV-12")));
    }
}
