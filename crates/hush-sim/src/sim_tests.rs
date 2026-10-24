use super::*;
use crate::{AltarError, NoiseSource, SeismoThreshold, Team};

fn config(seed: u64, ticks: u32, bots: u32) -> MatchConfig {
    MatchConfig {
        seed,
        max_ticks: ticks,
        bots,
        ..MatchConfig::default()
    }
}

#[test]
fn same_seed_gives_same_final_state() {
    let a = crate::run(config(42, 600, 4)).unwrap();
    let b = crate::run(config(42, 600, 4)).unwrap();
    assert_eq!(a.state_hash, b.state_hash);
    assert_eq!(a.ticks, 600);
    assert_eq!(a.edits, b.edits);
}

#[test]
fn different_seeds_diverge() {
    let a = crate::run(config(1, 10, 0)).unwrap();
    let b = crate::run(config(2, 10, 0)).unwrap();
    assert_ne!(a.state_hash, b.state_hash);
}

#[test]
fn bots_dig_without_breaking_invariants() {
    let report = crate::run(config(42, 1800, 6)).unwrap();
    assert!(
        report.broken_invariants.is_empty(),
        "{:?}",
        report.broken_invariants
    );
    let dug: u32 = report.edits.dug.values().sum();
    assert!(dug > 50, "los bots apenas cavaron: {:?}", report.edits);
}

#[test]
fn commands_from_unknown_players_are_rejected() {
    let mut sim = Simulation::new(config(5, 10, 0)).unwrap();
    sim.submit(Command::Dig {
        player: 99,
        cell: [5, 5, 5],
    });
    sim.step();
    assert_eq!(sim.stats().rejected.values().sum::<u32>(), 1);
}

#[test]
fn changes_rejections_and_leaving_players() {
    let mut sim = Simulation::new(config(5, 100, 0)).unwrap();
    let a = sim.spawn_player();
    for _ in 0..40 {
        sim.step();
    }
    let feet = sim.players()[a]
        .feet
        .map(|c| c.div_euclid(grisu_voxel::FIXED_ONE));
    let below = [feet[0], feet[1] - 1, feet[2]];
    sim.submit(Command::Dig {
        player: a,
        cell: below,
    });
    sim.submit(Command::Dig {
        player: a,
        cell: [
            if feet[0] < 32 {
                feet[0] + 10
            } else {
                feet[0] - 10
            },
            feet[1],
            feet[2],
        ],
    });
    sim.step();
    assert_eq!(
        sim.take_changes(),
        vec![BlockChange {
            cell: below,
            block: 0
        }]
    );
    let rejected = sim.take_rejections();
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected[0].error, EditError::TooFar);
    assert!(sim.take_changes().is_empty());

    sim.remove_player(a);
    assert!(!sim.is_active(a));
    assert_eq!(sim.active_players().count(), 0);
    sim.submit(Command::Dig {
        player: a,
        cell: below,
    });
    sim.step();
    assert_eq!(sim.take_rejections()[0].error, EditError::UnknownPlayer);
}

#[test]
fn invalid_map_size_is_an_error() {
    let mut c = config(1, 10, 0);
    c.terrain.size = [17, 64, 64];
    assert!(Simulation::new(c).is_err());
}

// Sismógrafo, altar y economía.

#[test]
fn altering_seismograph_changes_final_state_hash() {
    let config = MatchConfig {
        seed: 77,
        max_ticks: 100,
        ..MatchConfig::default()
    };
    let baseline = Simulation::new(config.clone()).unwrap().run_to_end();

    let mut noisy_sim = Simulation::new(config).unwrap();
    assert_eq!(
        noisy_sim.add_noise(NoiseSource::Explosion, Some(Team::Copper)),
        3_000
    );
    let noisy = noisy_sim.run_to_end();

    assert_ne!(baseline.state_hash, noisy.state_hash);
}

#[test]
fn simulation_records_events_and_finishes_when_awakening_ends() {
    let mut sim = Simulation::new(MatchConfig {
        seed: 11,
        max_ticks: 50_000,
        ..MatchConfig::default()
    })
    .unwrap();
    sim.add_noise(NoiseSource::Custom(50_000), Some(Team::Copper));
    assert_eq!(sim.offer_gold(Team::Cobalt, 100), Ok(2_500));
    assert_eq!(sim.return_tally(Team::Copper), Ok(8_000));
    sim.add_noise(NoiseSource::Custom(100_000), Some(Team::Cobalt));
    sim.step();
    // Tras el Despertar el altar queda cerrado.
    assert_eq!(
        sim.offer_gold(Team::Copper, 100),
        Err(AltarError::AltarClosed)
    );
    assert_eq!(sim.return_tally(Team::Cobalt), Err(AltarError::AltarClosed));

    let report = sim.run_to_end();
    assert_eq!(report.awakening_tick, Some(1));
    assert_eq!(report.ticks, 1 + 2_700);
    assert_eq!(
        report.thresholds_reached,
        vec![
            SeismoThreshold::Stir,
            SeismoThreshold::Breathe,
            SeismoThreshold::Restless,
            SeismoThreshold::Wake,
        ]
    );
    assert!(report.broken_invariants.is_empty());
}

#[test]
fn offer_gold_deducts_from_team_bank_and_rejects_insufficient_bank() {
    let mut sim = Simulation::new(MatchConfig {
        seed: 21,
        max_ticks: 1_000,
        ..MatchConfig::default()
    })
    .unwrap();
    sim.add_noise(NoiseSource::Custom(20_000), Some(Team::Copper));
    sim.mine_gold(1, Team::Copper, 1).unwrap();
    sim.deposit_player(1).unwrap();
    assert_eq!(sim.economy().bank(Team::Copper), 100);

    assert_eq!(sim.offer_gold(Team::Copper, 100), Ok(2_500));
    assert_eq!(sim.economy().bank(Team::Copper), 0);
    assert_eq!(sim.economy().total_offered_cash(), 100);
    assert!(sim.check_invariants().is_empty());

    // Avanzamos más allá del enfriamiento (150 ticks), pero el banco está a 0.
    for _ in 0..150 {
        sim.step();
    }
    assert_eq!(
        sim.offer_gold(Team::Copper, 100),
        Err(AltarError::NotEnoughGold { carried: 0 })
    );
}
