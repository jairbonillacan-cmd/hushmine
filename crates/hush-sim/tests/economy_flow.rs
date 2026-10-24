//! Pruebas de integración del flujo económico, bolsas de botín e invariantes en [`Simulation`].

use hush_sim::altar::{OFFERING_GOLD_COST, OFFERING_REDUCTION};
use hush_sim::loot::LOOT_BAG_LIFETIME_TICKS;
use hush_sim::{
    GridPos, MatchConfig, MatchEvent, MatchEventKind, NoiseSource, SimReport, Simulation, Team,
};

/// Verifica que los invariantes formales y la ecuación contable cerrada del oro se cumplen.
fn assert_gold_accounting_and_invariants(sim: &Simulation) {
    let broken = sim.check_invariants();
    assert!(
        broken.is_empty(),
        "no debe haber invariantes rotas en el tick {}: {broken:?}",
        sim.tick()
    );

    let econ = sim.economy();
    let carried = econ
        .total_carried_base_cash()
        .expect("la carga portada no debe desbordar");
    let active_bags = sim
        .loot()
        .total_active_base_cash()
        .expect("las bolsas activas no deben desbordar");

    let lhs = carried
        + active_bags
        + econ.bank(Team::Copper)
        + econ.bank(Team::Cobalt)
        + econ.total_offered_cash()
        + econ.total_expired_bag_cash();
    let rhs = econ.total_mined_cash() + econ.total_stolen_bonus_cash();

    assert_eq!(
        lhs,
        rhs,
        "la ecuación contable del oro debe cuadrar al dólar en el tick {}",
        sim.tick()
    );
}

/// Ejecuta una partida 1v1 guionizada completa y devuelve los eventos junto al informe final.
fn run_scripted_match(seed: u64) -> (Vec<MatchEvent>, SimReport) {
    let mut sim = Simulation::new(MatchConfig {
        seed,
        max_ticks: 10_000,
    });
    sim.set_team_size(1).expect("tamaño de equipo 1v1 válido");
    assert_eq!(sim.economy().target_cash(), 2_000);

    sim.register_player(1, Team::Copper);
    sim.register_player(2, Team::Cobalt);
    assert_gold_accounting_and_invariants(&sim);

    // Fase 1: Copper mina mena, oro y diamante, y deposita en su banco.
    for _ in 0..10 {
        sim.step();
    }
    assert_eq!(sim.mine_ore(1, Team::Copper, 3).unwrap(), 12);
    assert_eq!(sim.mine_gold(1, Team::Copper, 2).unwrap(), 200);
    assert_eq!(sim.mine_diamond(1, Team::Copper, 1).unwrap(), 400);
    assert_gold_accounting_and_invariants(&sim);

    let dep_copper = sim.deposit_player(1).unwrap();
    assert_eq!(dep_copper.base_cash, 600);
    assert_eq!(dep_copper.credited_cash, 600);
    assert_eq!(dep_copper.new_bank_balance, 600);
    assert!(!dep_copper.fame_newly_activated);
    assert!(!dep_copper.goal_reached);
    assert_gold_accounting_and_invariants(&sim);

    // Fase 2: Ruido sísmico y ofrenda de Copper en el altar desde su banco.
    for _ in 0..5 {
        sim.step();
    }
    assert_eq!(
        sim.add_noise(NoiseSource::Explosion, Some(Team::Copper)),
        3_000
    );
    let level_before = sim.seismograph().level();
    let bank_before = sim.economy().bank(Team::Copper);

    let reduction = sim.offer_from_bank(Team::Copper).unwrap();
    assert_eq!(reduction, OFFERING_REDUCTION);
    assert_eq!(
        sim.economy().bank(Team::Copper),
        bank_before - OFFERING_GOLD_COST
    );
    assert_eq!(sim.economy().total_offered_cash(), OFFERING_GOLD_COST);
    assert_eq!(sim.seismograph().level(), level_before - OFFERING_REDUCTION);
    assert_gold_accounting_and_invariants(&sim);

    // Fase 3: Copper vuelve a minar, muere soltando bolsa y Cobalt la roba (×5/4) y deposita.
    for _ in 0..5 {
        sim.step();
    }
    assert_eq!(sim.mine_gold(1, Team::Copper, 2).unwrap(), 200);
    assert_eq!(sim.mine_diamond(1, Team::Copper, 1).unwrap(), 400);
    assert_gold_accounting_and_invariants(&sim);

    let bag1 = sim
        .kill_player(1, GridPos::new(10, 4, -3))
        .unwrap()
        .expect("Copper llevaba recursos y debe soltar bolsa");
    assert_eq!(bag1.origin_team, Team::Copper);
    assert_eq!(bag1.base_cash_value(), Some(600));
    assert_gold_accounting_and_invariants(&sim);

    for _ in 0..5 {
        sim.step();
    }
    let pickup = sim.pickup_loot(2, Team::Cobalt, bag1.id).unwrap();
    assert!(pickup.is_stolen);
    assert_eq!(pickup.base_cash_value, 600);
    assert_eq!(pickup.effective_cash_value, 750);
    assert_gold_accounting_and_invariants(&sim);

    let dep_stolen = sim.deposit_player(2).unwrap();
    assert_eq!(dep_stolen.base_cash, 600);
    assert_eq!(dep_stolen.credited_cash, 750);
    assert_eq!(dep_stolen.stolen_bonus, 150);
    assert_eq!(dep_stolen.new_bank_balance, 750);
    assert!(!dep_stolen.fame_newly_activated);
    assert_gold_accounting_and_invariants(&sim);

    // Fase 4: Segunda bolsa soltada que nadie recoge; caduca tras LOOT_BAG_LIFETIME_TICKS.
    assert_eq!(sim.mine_gold(1, Team::Copper, 1).unwrap(), 100);
    let bag2 = sim
        .kill_player(1, GridPos::new(-5, 2, 8))
        .unwrap()
        .expect("debe soltar segunda bolsa");
    assert_eq!(bag2.base_cash_value(), Some(100));
    assert_gold_accounting_and_invariants(&sim);

    for _ in 0..(LOOT_BAG_LIFETIME_TICKS - 1) {
        sim.step();
    }
    assert_eq!(sim.loot().len(), 1);
    assert_eq!(sim.economy().total_expired_bag_cash(), 0);

    sim.step();
    assert!(sim.loot().is_empty());
    assert_eq!(sim.economy().total_expired_bag_cash(), 100);
    assert_gold_accounting_and_invariants(&sim);

    // Fase 5: Cobalt cruza primero el 60 % de Fama (1_200 $) y luego alcanza la meta (2_000 $).
    assert_eq!(sim.mine_diamond(2, Team::Cobalt, 1).unwrap(), 400);
    assert_eq!(sim.mine_gold(2, Team::Cobalt, 1).unwrap(), 100);
    let dep_fame = sim.deposit_player(2).unwrap();
    assert_eq!(dep_fame.new_bank_balance, 1_250);
    assert!(dep_fame.fame_newly_activated);
    assert!(!dep_fame.goal_reached);
    assert!(sim.economy().has_fame(Team::Cobalt));
    assert!(!sim.is_finished());
    assert_gold_accounting_and_invariants(&sim);

    sim.step();
    assert_eq!(sim.mine_diamond(2, Team::Cobalt, 2).unwrap(), 800);
    let dep_goal = sim.deposit_player(2).unwrap();
    assert_eq!(dep_goal.new_bank_balance, 2_050);
    assert!(!dep_goal.fame_newly_activated);
    assert!(dep_goal.goal_reached);
    assert!(sim.is_finished());
    assert_eq!(sim.economy().winner(), Some(Team::Cobalt));
    assert_gold_accounting_and_invariants(&sim);

    let events = sim.event_log().events().to_vec();
    let report = sim.run_to_end();
    assert!(report.broken_invariants.is_empty());
    (events, report)
}

#[test]
fn default_simulation_finishes_with_empty_broken_invariants() {
    let default_report = hush_sim::run(MatchConfig::default());
    assert_eq!(default_report.ticks, 28_800);
    assert_eq!(default_report.awakening_tick, Some(28_800));
    assert!(
        default_report.broken_invariants.is_empty(),
        "la simulación por defecto no debe romper invariantes: {:?}",
        default_report.broken_invariants
    );

    let full_awakening_report = hush_sim::run(MatchConfig {
        seed: 42,
        max_ticks: 32_000,
    });
    assert_eq!(full_awakening_report.ticks, 31_500);
    assert_eq!(full_awakening_report.awakening_tick, Some(28_800));
    assert!(
        full_awakening_report.broken_invariants.is_empty(),
        "el ciclo completo hasta fin del Despertar no debe romper invariantes: {:?}",
        full_awakening_report.broken_invariants
    );
}

#[test]
fn scripted_match_mines_deposits_dies_steals_bag_offers_and_balances_gold() {
    let (events, report) = run_scripted_match(42);
    assert!(report.broken_invariants.is_empty());

    assert!(events.iter().any(|e| matches!(
        e.kind,
        MatchEventKind::GoldMined {
            player_id: 1,
            cash_value: 200
        }
    )));
    assert!(events.iter().any(|e| matches!(
        e.kind,
        MatchEventKind::Deposited {
            player_id: 2,
            base_cash: 600,
            credited_cash: 750,
            new_balance: 750,
        }
    )));
    assert!(events.iter().any(|e| matches!(
        e.kind,
        MatchEventKind::OfferingMade {
            gold: OFFERING_GOLD_COST,
            reduction: OFFERING_REDUCTION,
        }
    ) && e.team == Some(Team::Copper)));
    assert!(events.iter().any(|e| matches!(
        e.kind,
        MatchEventKind::LootDropped {
            bag_id: 1,
            cash_value: 600,
        }
    )));
    assert!(events.iter().any(|e| matches!(
        e.kind,
        MatchEventKind::LootPicked {
            bag_id: 1,
            picker_id: 2,
            origin_team: Team::Copper,
            base_cash: 600,
            effective_cash: 750,
        }
    ) && e.team == Some(Team::Cobalt)));
    assert!(events.iter().any(|e| matches!(
        e.kind,
        MatchEventKind::LootExpired {
            bag_id: 2,
            cash_value: 100,
        }
    )));
    assert!(events.iter().any(|e| matches!(
        e.kind,
        MatchEventKind::FameGained {
            balance: 1_250,
            target_cash: 2_000,
        }
    ) && e.team == Some(Team::Cobalt)));
    assert!(events.iter().any(|e| matches!(
        e.kind,
        MatchEventKind::GoalReached {
            balance: 2_050,
            target_cash: 2_000,
        }
    ) && e.team == Some(Team::Cobalt)));
}

#[test]
fn determinism_of_scripted_economy_match() {
    let (events_a, report_a) = run_scripted_match(1337);
    let (events_b, report_b) = run_scripted_match(1337);

    assert!(report_a.broken_invariants.is_empty());
    assert!(report_b.broken_invariants.is_empty());
    assert_eq!(report_a.ticks, report_b.ticks);
    assert_eq!(report_a.state_hash, report_b.state_hash);
    assert_eq!(events_a, events_b);
}

#[test]
fn tampered_simulation_state_populates_broken_invariants_in_report() {
    let mut sim = Simulation::new(MatchConfig {
        seed: 7,
        max_ticks: 30,
    });
    sim.economy_mut().set_bank_unchecked(Team::Copper, 999);

    let report = sim.run_to_end();
    assert!(
        !report.broken_invariants.is_empty(),
        "manipular el banco sin eventos debe poblar broken_invariants"
    );
    assert!(
        report
            .broken_invariants
            .iter()
            .any(|msg| msg.contains("INV-01")),
        "debe señalarse la violación de INV-01: {:?}",
        report.broken_invariants
    );
}
