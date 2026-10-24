//! Pruebas de integración del flujo completo del Sismógrafo, el Despertar y la Crónica.

use hush_sim::altar::{OFFERING_COOLDOWN_TICKS, OFFERING_REDUCTION, TALLY_RETURN_REDUCTION};
use hush_sim::seismograph::{AWAKENING_DURATION_TICKS, PASSIVE_TICKS_TO_WAKE};
use hush_sim::{
    AltarError, EventLog, MatchConfig, MatchEventKind, NoiseSource, SeismoThreshold, SimReport,
    Simulation, Team,
};

/// Ejecuta una partida guionizada determinista con acciones de ambos equipos.
fn run_scripted_match(seed: u64) -> SimReport {
    let mut sim = Simulation::new(MatchConfig {
        seed,
        max_ticks: 32_000,
    });
    sim.seismograph_mut().set_active_bucket_lines(2);

    while !sim.is_finished() && sim.awakening_tick().is_none() {
        sim.step();
        match sim.tick() {
            120 => {
                sim.add_noise(NoiseSource::Explosion, Some(Team::Copper));
            }
            240 => {
                sim.add_noise(NoiseSource::ElectricDischarge, Some(Team::Cobalt));
                sim.add_noise(NoiseSource::FlowingLava, None);
            }
            360 => {
                sim.offer_gold(Team::Cobalt, 100)
                    .expect("la ofrenda debe aceptarse");
            }
            600 => {
                sim.add_noise(NoiseSource::Explosion, Some(Team::Cobalt));
                sim.return_tally(Team::Copper)
                    .expect("la ficha debe aceptarse");
            }
            _ => {}
        }
    }

    sim.run_to_end()
}

#[test]
fn same_seed_produces_identical_sim_report() {
    let config = MatchConfig {
        seed: 42,
        max_ticks: 32_000,
    };
    let report_a = hush_sim::run(config.clone());
    let report_b = hush_sim::run(config);

    assert_eq!(report_a.seed, report_b.seed);
    assert_eq!(report_a.ticks, report_b.ticks);
    assert_eq!(report_a.state_hash, report_b.state_hash);
    assert_eq!(report_a.thresholds_reached, report_b.thresholds_reached);
    assert_eq!(report_a.awakening_tick, report_b.awakening_tick);
    assert!(report_a.broken_invariants.is_empty());
    assert!(report_b.broken_invariants.is_empty());

    // También verificamos determinismo con eventos de ruido, ofrendas y fichas.
    let scripted_a = run_scripted_match(77);
    let scripted_b = run_scripted_match(77);

    assert_eq!(scripted_a.seed, scripted_b.seed);
    assert_eq!(scripted_a.ticks, scripted_b.ticks);
    assert_eq!(scripted_a.state_hash, scripted_b.state_hash);
    assert_eq!(scripted_a.thresholds_reached, scripted_b.thresholds_reached);
    assert_eq!(scripted_a.awakening_tick, scripted_b.awakening_tick);
    assert!(scripted_a.broken_invariants.is_empty());
    assert!(scripted_b.broken_invariants.is_empty());
}

#[test]
fn with_max_ticks_32000_awakening_starts_at_most_at_tick_28800_and_ends_2700_ticks_later() {
    let config = MatchConfig {
        seed: 42,
        max_ticks: 32_000,
    };
    let report = hush_sim::run(config);

    let awakening_tick = report
        .awakening_tick
        .expect("el Despertar debe iniciarse antes de agotar max_ticks");
    assert!(awakening_tick <= PASSIVE_TICKS_TO_WAKE);
    assert_eq!(report.awakening_tick, Some(PASSIVE_TICKS_TO_WAKE));
    assert_eq!(
        report.thresholds_reached,
        vec![
            SeismoThreshold::Stir,
            SeismoThreshold::Breathe,
            SeismoThreshold::Restless,
            SeismoThreshold::Wake,
        ]
    );
    assert_eq!(report.ticks, awakening_tick + AWAKENING_DURATION_TICKS);
    assert_eq!(report.ticks, 31_500);
    assert!(report.ticks < 32_000);
    assert!(report.broken_invariants.is_empty());
}

/// Ejecuta un escenario con ruido intenso y, opcionalmente, ofrendas y devolución de fichas.
fn run_noisy_scenario(with_offerings: bool) -> (Vec<SeismoThreshold>, SimReport) {
    let mut sim = Simulation::new(MatchConfig {
        seed: 108,
        max_ticks: 32_000,
    });
    sim.seismograph_mut().set_active_bucket_lines(3);

    while !sim.is_finished() && sim.awakening_tick().is_none() {
        sim.step();
        match sim.tick() {
            // Supera el primer umbral (Stir = 25_000) con explosiones y descargas.
            50 => {
                for _ in 0..9 {
                    sim.add_noise(NoiseSource::Explosion, Some(Team::Copper));
                }
                for _ in 0..5 {
                    sim.add_noise(NoiseSource::ElectricDischarge, Some(Team::Cobalt));
                }
            }
            // Si hay ofrendas, reducimos el nivel por debajo de Stir para cruzarlo de nuevo después.
            100 if with_offerings => {
                assert_eq!(sim.return_tally(Team::Copper), Ok(TALLY_RETURN_REDUCTION));
                assert_eq!(sim.offer_gold(Team::Cobalt, 100), Ok(OFFERING_REDUCTION));
                // Segunda ofrenda del mismo equipo sin esperar: rechazada.
                assert_eq!(
                    sim.offer_gold(Team::Cobalt, 100),
                    Err(AltarError::CoolingDown {
                        ready_at_tick: 100 + OFFERING_COOLDOWN_TICKS
                    })
                );
            }
            // Más actividad minera ruidosa en ambos escenarios.
            300 => {
                for _ in 0..8 {
                    sim.add_noise(NoiseSource::Explosion, Some(Team::Cobalt));
                }
                for _ in 0..10 {
                    sim.add_noise(NoiseSource::ElectricDischarge, Some(Team::Copper));
                }
            }
            600 => {
                for _ in 0..6 {
                    sim.add_noise(NoiseSource::Explosion, Some(Team::Copper));
                }
                for _ in 0..5 {
                    sim.add_noise(NoiseSource::ElectricDischarge, Some(Team::Cobalt));
                }
            }
            // Segunda ronda de ofrendas y fichas tras superar Breathe (50_000).
            650 if with_offerings => {
                assert_eq!(sim.return_tally(Team::Cobalt), Ok(TALLY_RETURN_REDUCTION));
                assert_eq!(sim.offer_gold(Team::Copper, 100), Ok(OFFERING_REDUCTION));
                // Ya se devolvieron las dos fichas del Turno.
                assert_eq!(
                    sim.return_tally(Team::Copper),
                    Err(AltarError::TallyLimitReached)
                );
            }
            _ => {}
        }
    }

    let thresholds_in_log = sim.event_log().thresholds_reached();
    let report = sim.run_to_end();
    (thresholds_in_log, report)
}

#[test]
fn noise_accelerates_awakening_and_offerings_delay_it() {
    let (thresholds_fast, report_fast) = run_noisy_scenario(false);
    let (thresholds_delayed, report_delayed) = run_noisy_scenario(true);

    let awakening_fast = report_fast
        .awakening_tick
        .expect("el escenario con ruido debe alcanzar el Despertar");
    let awakening_delayed = report_delayed
        .awakening_tick
        .expect("el escenario con ofrendas también debe alcanzar el Despertar");

    // El ruido acelera el Despertar mucho antes del límite pasivo de 28_800 ticks.
    assert!(awakening_fast < 10_000);
    assert!(awakening_fast < PASSIVE_TICKS_TO_WAKE);
    assert_eq!(report_fast.ticks, awakening_fast + AWAKENING_DURATION_TICKS);

    // Las ofrendas de oro y devolución de fichas retrasan el Despertar respecto al mismo ruido.
    assert!(awakening_delayed > awakening_fast);
    assert!(awakening_delayed < PASSIVE_TICKS_TO_WAKE);
    assert_eq!(
        report_delayed.ticks,
        awakening_delayed + AWAKENING_DURATION_TICKS
    );

    // Ningún umbral se registra más de una vez aunque el nivel baje y vuelva a subir.
    let expected_thresholds = vec![
        SeismoThreshold::Stir,
        SeismoThreshold::Breathe,
        SeismoThreshold::Restless,
        SeismoThreshold::Wake,
    ];
    assert_eq!(thresholds_fast, expected_thresholds);
    assert_eq!(thresholds_delayed, expected_thresholds);
    assert_eq!(report_fast.thresholds_reached, expected_thresholds);
    assert_eq!(report_delayed.thresholds_reached, expected_thresholds);
    assert!(report_fast.broken_invariants.is_empty());
    assert!(report_delayed.broken_invariants.is_empty());
}

#[test]
fn event_log_chronicle_sequence_is_consistent_and_serializable() {
    let mut sim = Simulation::new(MatchConfig {
        seed: 2026,
        max_ticks: 32_000,
    });
    sim.seismograph_mut().set_active_bucket_lines(3);

    while !sim.is_finished() {
        sim.step();
        match sim.tick() {
            80 => {
                for _ in 0..10 {
                    sim.add_noise(NoiseSource::Explosion, Some(Team::Copper));
                }
            }
            160 => {
                sim.offer_gold(Team::Cobalt, 100)
                    .expect("la ofrenda debe aceptarse");
                sim.return_tally(Team::Copper)
                    .expect("la ficha debe aceptarse");
            }
            320 => {
                for _ in 0..12 {
                    sim.add_noise(NoiseSource::Explosion, Some(Team::Cobalt));
                }
                for _ in 0..5 {
                    sim.add_noise(NoiseSource::ElectricDischarge, Some(Team::Copper));
                }
            }
            500 => {
                sim.offer_gold(Team::Copper, 100)
                    .expect("la ofrenda debe aceptarse");
                sim.return_tally(Team::Cobalt)
                    .expect("la ficha debe aceptarse");
            }
            700 => {
                for _ in 0..15 {
                    sim.add_noise(NoiseSource::Explosion, Some(Team::Copper));
                }
            }
            _ => {}
        }
    }

    let event_log = sim.event_log().clone();
    let events = event_log.events();
    assert!(!events.is_empty());

    // 1. Los ticks de la Crónica son monótonamente no decrecientes.
    for pair in events.windows(2) {
        assert!(
            pair[0].tick <= pair[1].tick,
            "los ticks deben ser monótonamente no decrecientes: {} > {}",
            pair[0].tick,
            pair[1].tick
        );
    }

    // 2. Existen eventos de ambos equipos (Copper y Cobalt).
    assert!(events.iter().any(|e| e.team == Some(Team::Copper)));
    assert!(events.iter().any(|e| e.team == Some(Team::Cobalt)));

    // 3. Verificación de tiempos exactos del ciclo del Despertar.
    let awakening_started_tick = events
        .iter()
        .find_map(|e| matches!(e.kind, MatchEventKind::AwakeningStarted).then_some(e.tick))
        .expect("debe existir el evento AwakeningStarted");
    let bucket_stopped_tick = events
        .iter()
        .find_map(|e| matches!(e.kind, MatchEventKind::BucketLinesStopped).then_some(e.tick))
        .expect("debe existir el evento BucketLinesStopped");
    let awakening_ended_tick = events
        .iter()
        .find_map(|e| matches!(e.kind, MatchEventKind::AwakeningEnded).then_some(e.tick))
        .expect("debe existir el evento AwakeningEnded");

    assert_eq!(bucket_stopped_tick, awakening_started_tick + 1_800);
    assert_eq!(
        awakening_ended_tick,
        awakening_started_tick + AWAKENING_DURATION_TICKS
    );

    // 4. Serialización y deserialización sin pérdida de EventLog y SimReport.
    let log_json = serde_json::to_string(&event_log).expect("EventLog debe serializar a JSON");
    let decoded_log: EventLog =
        serde_json::from_str(&log_json).expect("EventLog debe deserializar desde JSON");
    assert_eq!(decoded_log, event_log);

    let report = sim.run_to_end();
    let report_json = serde_json::to_string(&report).expect("SimReport debe serializar a JSON");
    let decoded_report: SimReport =
        serde_json::from_str(&report_json).expect("SimReport debe deserializar desde JSON");

    assert_eq!(decoded_report.seed, report.seed);
    assert_eq!(decoded_report.ticks, report.ticks);
    assert_eq!(decoded_report.state_hash, report.state_hash);
    assert_eq!(decoded_report.thresholds_reached, report.thresholds_reached);
    assert_eq!(decoded_report.awakening_tick, report.awakening_tick);
    assert_eq!(decoded_report.broken_invariants, report.broken_invariants);
    assert!(decoded_report.broken_invariants.is_empty());
}
