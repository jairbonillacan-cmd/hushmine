use super::*;

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
