//! Pruebas del planificador de streaming (`scheduler.rs`).

use crate::scheduler::{ChunkStage, DistancePreset, HYSTERESIS, StreamScheduler};
use grisu_voxel::ChunkPos;

#[test]
fn low_pide_3179_y_empieza_por_el_chunk_del_jugador() {
    let mut scheduler = StreamScheduler::new(DistancePreset::Low);
    scheduler.update_players(&[(ChunkPos::new(0, 0, 0), [0; 3])]);
    let plan = scheduler.plan_loads();
    assert_eq!(plan.len(), 17 * 17 * 11);
    // El chunk del propio jugador es el más cercano y sale el primero.
    assert_eq!(plan.first().copied(), Some(ChunkPos::new(0, 0, 0)));
}

#[test]
fn medium_y_high_cuentan_sus_cajas() {
    let mut medium = StreamScheduler::new(DistancePreset::Medium);
    medium.update_players(&[(ChunkPos::new(0, 0, 0), [0; 3])]);
    assert_eq!(medium.plan_loads().len(), 25 * 25 * 13);

    let mut high = StreamScheduler::new(DistancePreset::High);
    high.update_players(&[(ChunkPos::new(0, 0, 0), [0; 3])]);
    assert_eq!(high.plan_loads().len(), 33 * 33 * 17);
}

#[test]
fn histeresis_evita_descargas_al_paso_y_volver() {
    assert_eq!(HYSTERESIS, 2);
    let mut scheduler = StreamScheduler::new(DistancePreset::Low);
    scheduler.update_players(&[(ChunkPos::new(0, 0, 0), [0; 3])]);
    let primera = scheduler.plan_loads();
    assert_eq!(primera.len(), 3179);
    scheduler.commit(&primera, &[]);
    assert_eq!(scheduler.active_len(), 3179);

    // Un paso lateral: nada sale (todo sigue dentro de R+2=10 / V+2=7) y
    // solo piden entrada los 187 chunks del borde nuevo.
    scheduler.update_players(&[(ChunkPos::new(1, 0, 0), [0; 3])]);
    assert!(scheduler.plan_unloads().is_empty());
    let segunda = scheduler.plan_loads();
    assert_eq!(segunda.len(), 17 * 11);
    for pos in &segunda {
        assert_eq!(pos.x, 9);
    }
    scheduler.commit(&segunda, &[]);

    // Volver al origen: todo lo deseado ya está activo y nada se descarga.
    scheduler.update_players(&[(ChunkPos::new(0, 0, 0), [0; 3])]);
    assert!(scheduler.plan_unloads().is_empty());
    assert!(scheduler.plan_loads().is_empty());
}

#[test]
fn alejarse_mucho_descarga_lo_que_sale_de_r_mas_2() {
    let mut scheduler = StreamScheduler::new(DistancePreset::Low);
    scheduler.update_players(&[(ChunkPos::new(0, 0, 0), [0; 3])]);
    let cargas = scheduler.plan_loads();
    scheduler.commit(&cargas, &[]);
    assert_eq!(scheduler.active_len(), 3179);

    // Con el jugador en (14, 0, 0) la histéresis (|x-14| <= 10) solo
    // retiene x en [4..=8]; z e y completos.
    scheduler.update_players(&[(ChunkPos::new(14, 0, 0), [0; 3])]);
    let descargas = scheduler.plan_unloads();
    assert_eq!(descargas.len(), 3179 - 5 * 17 * 11);
    assert!(descargas.iter().all(|pos| pos.x < 4 || pos.x > 8));

    // Tras aplicar el plan queda la franja cercana al jugador.
    scheduler.commit(&[], &descargas);
    assert_eq!(scheduler.active_len(), 5 * 17 * 11);
    assert!(scheduler.is_active(ChunkPos::new(6, 0, 0)));
    assert!(!scheduler.is_active(ChunkPos::new(0, 0, 0)));
}

#[test]
fn prioridad_prefiere_la_direccion_de_avance() {
    let mut scheduler = StreamScheduler::new(DistancePreset::Low);
    scheduler.update_players(&[(ChunkPos::new(0, 0, 0), [1, 0, 0])]);
    let plan = scheduler.plan_loads();
    let delante = plan.iter().position(|pos| *pos == ChunkPos::new(1, 0, 0));
    let detras = plan.iter().position(|pos| *pos == ChunkPos::new(-1, 0, 0));
    // Misma cheb y manh: gana la alineación con el avance.
    assert!(matches!((delante, detras), (Some(a), Some(b)) if a < b));
}

#[test]
fn orden_de_etapas_del_cauce() {
    assert!(ChunkStage::Column < ChunkStage::Base);
    assert!(ChunkStage::Base < ChunkStage::Decorated);
    assert!(ChunkStage::Decorated < ChunkStage::Lit);
    assert!(ChunkStage::Lit < ChunkStage::Meshed);
}

#[test]
fn sin_jugadores_todo_lo_activo_se_descarga() {
    let mut scheduler = StreamScheduler::new(DistancePreset::Low);
    scheduler.update_players(&[(ChunkPos::new(0, 0, 0), [0; 3])]);
    let cargas = scheduler.plan_loads();
    scheduler.commit(&cargas, &[]);
    assert!(scheduler.active_len() > 0);

    // Sin jugadores no hay caja de histéresis que retenga nada.
    scheduler.update_players(&[]);
    assert!(scheduler.plan_loads().is_empty());
    let descargas = scheduler.plan_unloads();
    assert_eq!(descargas.len(), scheduler.active_len());
}

#[test]
fn commit_es_idempotente_y_set_preset_agranda_el_deseado() {
    let mut scheduler = StreamScheduler::new(DistancePreset::Low);
    scheduler.update_players(&[(ChunkPos::new(0, 0, 0), [0; 3])]);
    let cargas = scheduler.plan_loads();
    scheduler.commit(&cargas, &[]);
    // Repetir cargas ya activas y descargar chunks que nunca estuvieron
    // activos no cambia nada.
    let lejanos = [ChunkPos::new(500, 0, 0), ChunkPos::new(-500, 33, 7)];
    scheduler.commit(&cargas, &lejanos);
    assert_eq!(scheduler.active_len(), 3179);

    // Al subir el preset piden entrada los chunks del anillo nuevo y nada
    // sale: lo activo sigue dentro de la histéresis del preset mayor.
    scheduler.set_preset(DistancePreset::Medium);
    let nuevas = scheduler.plan_loads();
    assert_eq!(nuevas.len(), 25 * 25 * 13 - 17 * 17 * 11);
    assert!(scheduler.plan_unloads().is_empty());
}

#[test]
fn dos_jugadores_deduplican_la_union_de_cajas() {
    let mut solo_uno = StreamScheduler::new(DistancePreset::Low);
    solo_uno.update_players(&[(ChunkPos::new(0, 0, 0), [0; 3])]);
    let esperado_solo = solo_uno.plan_loads().len();

    // Cajas desplazadas un chunk en X: la unión mide 18×17×11.
    let mut scheduler = StreamScheduler::new(DistancePreset::Low);
    scheduler.update_players(&[
        (ChunkPos::new(0, 0, 0), [0; 3]),
        (ChunkPos::new(1, 0, 0), [0; 3]),
    ]);
    let plan = scheduler.plan_loads();
    assert_eq!(plan.len(), esperado_solo + 17 * 11);
    // Deduplicación real: no hay posiciones repetidas en el plan.
    let mut unico = plan.clone();
    unico.sort();
    unico.dedup();
    assert_eq!(unico.len(), plan.len());
}
