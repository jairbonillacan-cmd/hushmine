//! Guardia de coherencia entre las constantes públicas de `hush-sim` y `data/*.ron`.
//!
//! Verifica mediante búsqueda textual determinista (sin depender de un parser RON)
//! que cada constante pública de `hush_sim` aparezca con el mismo valor exacto en
//! `data/seismograph.ron`, `data/economy.ron` y `data/blocks.ron`. Si se modifica
//! un archivo `.ron` sin actualizar las constantes de Rust (o viceversa), estas
//! pruebas fallan de inmediato.

use hush_sim::altar::{
    MAX_TALLIES_PER_SHIFT, OFFERING_COOLDOWN_TICKS, OFFERING_GOLD_COST, OFFERING_REDUCTION,
    TALLY_RETURN_REDUCTION,
};
use hush_sim::economy::{
    DIAMOND_CASH_VALUE, DIAMOND_UNIT_CASH_VALUE, DIAMOND_UNITS_PER_BLOCK, DROP_IN_PHASE_TICKS,
    FAME_THRESHOLD_DEN, FAME_THRESHOLD_NUM, GOLD_CASH_VALUE, GOLD_UNITS_PER_BLOCK,
    HEART_TOTAL_CASH, MAX_POST_SPACING_BLOCKS, MAX_POSTS_PER_LINE, MOTHER_LODE_TOTAL_CASH,
    ORE_UNITS_PER_BLOCK, RESPAWN_TICKS, STOLEN_LOOT_MULTIPLIER_DEN, STOLEN_LOOT_MULTIPLIER_NUM,
    TEAM_TARGETS, TRANSFER_INTERVAL_TICKS, UNITS_PER_BUCKET, target_cash_for_team_size,
};
use hush_sim::loot::{LOOT_BAG_DROP_DEN, LOOT_BAG_DROP_NUM, LOOT_BAG_LIFETIME_TICKS};
use hush_sim::seismograph::{
    AWAKENING_BUCKET_STOP_REMAINING_TICKS, AWAKENING_DURATION_TICKS, MAX_LEVEL,
    NOISE_BUCKET_LINE_PER_TICK, NOISE_ELECTRIC_DISCHARGE, NOISE_EXPLOSION, NOISE_FLOWING_LAVA,
    PASSIVE_TICKS_TO_WAKE, THRESHOLD_BREATHE, THRESHOLD_RESTLESS, THRESHOLD_STIR, THRESHOLD_WAKE,
};
use hush_sim::{BlockKind, SeismoThreshold, TICK_RATE};

const SEISMOGRAPH_RON: &str = include_str!("../../../data/seismograph.ron");
const ECONOMY_RON: &str = include_str!("../../../data/economy.ron");
const BLOCKS_RON: &str = include_str!("../../../data/blocks.ron");

/// Formatea un entero `u32` siguiendo la convención de literales en `data/*.ron`:
/// valores `>= 1_000` con separador de miles (`_`) y valores `< 1_000` sin guion bajo.
fn format_ron_u32(value: u32) -> String {
    if value >= 1_000 {
        format!("{}_{:03}", value / 1_000, value % 1_000)
    } else {
        value.to_string()
    }
}

/// Comprueba que un archivo `.ron` contenga el par `key: <valor>,` exacto.
fn check_field_u32(ron: &str, file_label: &str, key: &str, value: u32) -> Result<(), String> {
    let expected = format!("{key}: {},", format_ron_u32(value));
    if ron.contains(&expected) {
        Ok(())
    } else {
        Err(format!(
            "No se encontró `{expected}` en `{file_label}` (constante con valor {value})"
        ))
    }
}

/// Comprueba que un bloque `Threshold( id: <id>, ... value: <valor>, )` exista en `seismograph.ron`.
fn check_threshold_entry(ron: &str, id: &str, value: u32) -> Result<(), String> {
    let id_marker = format!("id: {id},");
    let value_marker = format!("value: {},", format_ron_u32(value));

    let Some(start) = ron.find(&id_marker) else {
        return Err(format!(
            "No se encontró el umbral `{id_marker}` en `data/seismograph.ron`"
        ));
    };
    let tail = &ron[start..];
    let Some(end) = tail.find("),") else {
        return Err(format!(
            "Bloque de umbral `{id}` mal cerrado en `data/seismograph.ron`"
        ));
    };
    let block = &tail[..end];
    if block.contains(&value_marker) {
        Ok(())
    } else {
        Err(format!(
            "El umbral `{id}` no contiene `{value_marker}` en `data/seismograph.ron`"
        ))
    }
}

/// Valida todas las constantes de Sismógrafo y Altar contra el texto de `data/seismograph.ron`.
fn verify_seismograph_ron(ron: &str) -> Result<(), String> {
    check_field_u32(ron, "data/seismograph.ron", "tick_rate", TICK_RATE)?;
    check_field_u32(ron, "data/seismograph.ron", "max_value", MAX_LEVEL)?;
    check_field_u32(ron, "data/seismograph.ron", "total_value", MAX_LEVEL)?;
    check_field_u32(
        ron,
        "data/seismograph.ron",
        "total_ticks",
        PASSIVE_TICKS_TO_WAKE,
    )?;

    check_threshold_entry(ron, "Stir", THRESHOLD_STIR)?;
    check_threshold_entry(ron, "Breathe", THRESHOLD_BREATHE)?;
    check_threshold_entry(ron, "Restless", THRESHOLD_RESTLESS)?;
    check_threshold_entry(ron, "Wake", THRESHOLD_WAKE)?;

    check_field_u32(ron, "data/seismograph.ron", "explosion", NOISE_EXPLOSION)?;
    check_field_u32(
        ron,
        "data/seismograph.ron",
        "electric_discharge",
        NOISE_ELECTRIC_DISCHARGE,
    )?;
    check_field_u32(ron, "data/seismograph.ron", "lava_flow", NOISE_FLOWING_LAVA)?;
    check_field_u32(
        ron,
        "data/seismograph.ron",
        "bucket_line_per_tick",
        NOISE_BUCKET_LINE_PER_TICK,
    )?;

    check_field_u32(
        ron,
        "data/seismograph.ron",
        "duration_ticks",
        AWAKENING_DURATION_TICKS,
    )?;
    check_field_u32(
        ron,
        "data/seismograph.ron",
        "bucket_stop_remaining_ticks",
        AWAKENING_BUCKET_STOP_REMAINING_TICKS,
    )?;
    check_field_u32(
        ron,
        "data/seismograph.ron",
        "heart_cash_value",
        HEART_TOTAL_CASH,
    )?;

    check_field_u32(
        ron,
        "data/seismograph.ron",
        "offering_gold_cost",
        OFFERING_GOLD_COST,
    )?;
    check_field_u32(
        ron,
        "data/seismograph.ron",
        "offering_reduction",
        OFFERING_REDUCTION,
    )?;
    check_field_u32(
        ron,
        "data/seismograph.ron",
        "offering_cooldown_ticks",
        OFFERING_COOLDOWN_TICKS,
    )?;
    check_field_u32(
        ron,
        "data/seismograph.ron",
        "brass_tally_reduction",
        TALLY_RETURN_REDUCTION,
    )?;
    check_field_u32(
        ron,
        "data/seismograph.ron",
        "max_tallies_per_shift",
        MAX_TALLIES_PER_SHIFT,
    )?;

    Ok(())
}

/// Valida todas las constantes de Economía y Botín contra el texto de `data/economy.ron`.
fn verify_economy_ron(ron: &str) -> Result<(), String> {
    check_field_u32(ron, "data/economy.ron", "tick_rate", TICK_RATE)?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "drop_in_phase_ticks",
        DROP_IN_PHASE_TICKS,
    )?;
    check_field_u32(ron, "data/economy.ron", "respawn_ticks", RESPAWN_TICKS)?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "loot_bag_lifetime_ticks",
        LOOT_BAG_LIFETIME_TICKS,
    )?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "loot_bag_drop_num",
        LOOT_BAG_DROP_NUM,
    )?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "loot_bag_drop_den",
        LOOT_BAG_DROP_DEN,
    )?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "fame_threshold_num",
        FAME_THRESHOLD_NUM,
    )?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "fame_threshold_den",
        FAME_THRESHOLD_DEN,
    )?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "stolen_loot_multiplier_num",
        STOLEN_LOOT_MULTIPLIER_NUM,
    )?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "stolen_loot_multiplier_den",
        STOLEN_LOOT_MULTIPLIER_DEN,
    )?;

    check_field_u32(
        ron,
        "data/economy.ron",
        "ore_units_per_block",
        ORE_UNITS_PER_BLOCK,
    )?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "gold_units_per_block",
        GOLD_UNITS_PER_BLOCK,
    )?;
    check_field_u32(ron, "data/economy.ron", "gold_cash_value", GOLD_CASH_VALUE)?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "diamond_units_per_block",
        DIAMOND_UNITS_PER_BLOCK,
    )?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "diamond_cash_value",
        DIAMOND_CASH_VALUE,
    )?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "mother_lode_total_cash",
        MOTHER_LODE_TOTAL_CASH,
    )?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "heart_total_cash",
        HEART_TOTAL_CASH,
    )?;

    check_field_u32(
        ron,
        "data/economy.ron",
        "max_posts_per_line",
        MAX_POSTS_PER_LINE,
    )?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "max_post_spacing_blocks",
        MAX_POST_SPACING_BLOCKS,
    )?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "transfer_interval_ticks",
        TRANSFER_INTERVAL_TICKS,
    )?;
    check_field_u32(
        ron,
        "data/economy.ron",
        "units_per_bucket",
        UNITS_PER_BUCKET,
    )?;

    for (index, &target_cash) in TEAM_TARGETS.iter().enumerate() {
        let team_size = u8::try_from(index + 1).expect("índice de equipo válido 1..=8");
        assert_eq!(target_cash_for_team_size(team_size), Ok(target_cash));
        let expected_goal = format!(
            "team_size: {team_size}, target_cash: {},",
            format_ron_u32(target_cash)
        );
        if !ron.contains(&expected_goal) {
            return Err(format!(
                "No se encontró `{expected_goal}` en `data/economy.ron`"
            ));
        }
    }

    Ok(())
}

/// Valida que todos los bloques de `BlockKind::ALL` coincidan en id, nombre y pertenencia con `data/blocks.ron`.
fn verify_blocks_ron(ron: &str) -> Result<(), String> {
    for kind in BlockKind::ALL {
        let id_marker = format!("id: {},", kind.id());
        let name_marker = format!("name: \"{}\",", kind.name());
        let team_marker = format!("team_owned: {},", kind.is_team_block());

        let Some(start) = ron.find(&id_marker) else {
            return Err(format!(
                "No se encontró `{id_marker}` en `data/blocks.ron` para {kind:?}"
            ));
        };
        let tail = &ron[start..];
        let end = tail[1..]
            .find("BlockDef(")
            .map_or(tail.len(), |pos| pos + 1);
        let section = &tail[..end];

        for marker in [&name_marker, &team_marker] {
            if !section.contains(marker) {
                return Err(format!(
                    "El bloque {kind:?} (`{id_marker}`) no contiene `{marker}` en `data/blocks.ron`"
                ));
            }
        }
    }
    Ok(())
}

#[test]
fn format_ron_u32_formats_thousands_with_underscore() {
    assert_eq!(format_ron_u32(0), "0");
    assert_eq!(format_ron_u32(1), "1");
    assert_eq!(format_ron_u32(30), "30");
    assert_eq!(format_ron_u32(999), "999");
    assert_eq!(format_ron_u32(1_000), "1_000");
    assert_eq!(format_ron_u32(1_500), "1_500");
    assert_eq!(format_ron_u32(25_000), "25_000");
    assert_eq!(format_ron_u32(100_000), "100_000");
}

#[test]
fn seismograph_constants_match_seismograph_ron() {
    assert_eq!(SeismoThreshold::Stir.target_level(), THRESHOLD_STIR);
    assert_eq!(SeismoThreshold::Breathe.target_level(), THRESHOLD_BREATHE);
    assert_eq!(SeismoThreshold::Restless.target_level(), THRESHOLD_RESTLESS);
    assert_eq!(SeismoThreshold::Wake.target_level(), THRESHOLD_WAKE);

    verify_seismograph_ron(SEISMOGRAPH_RON).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn economy_and_loot_constants_match_economy_ron() {
    assert_eq!(
        DIAMOND_UNIT_CASH_VALUE * DIAMOND_UNITS_PER_BLOCK,
        DIAMOND_CASH_VALUE
    );
    verify_economy_ron(ECONOMY_RON).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn block_catalog_matches_blocks_ron() {
    verify_blocks_ron(BLOCKS_RON).unwrap_or_else(|err| panic!("{err}"));
}

#[test]
fn simulated_ron_drift_is_detected_by_verifiers() {
    let drifted_seismo = SEISMOGRAPH_RON.replace("explosion: 3_000,", "explosion: 3_500,");
    assert!(verify_seismograph_ron(&drifted_seismo).is_err());

    let drifted_threshold = SEISMOGRAPH_RON.replace("value: 25_000,", "value: 26_000,");
    assert!(verify_seismograph_ron(&drifted_threshold).is_err());

    let drifted_economy = ECONOMY_RON.replace("target_cash: 6_500,", "target_cash: 6_000,");
    assert!(verify_economy_ron(&drifted_economy).is_err());

    let drifted_blocks = BLOCKS_RON.replace("name: \"bucket_post\",", "name: \"post\",");
    assert!(verify_blocks_ron(&drifted_blocks).is_err());
}
