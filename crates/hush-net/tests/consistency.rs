//! Condición de salida del hito 2: dos clientes y seis bots siguen
//! consistentes con 150 ms de ida y vuelta y un 2 % de pérdida.

use grisu_net::LinkConditions;
use hush_net::harness::{NetTestConfig, run_net_test};

#[test]
fn two_clients_and_six_bots_stay_consistent_at_150ms_and_2_percent_loss() {
    let config = NetTestConfig {
        play_ms: 30_000,
        ..NetTestConfig::default()
    };
    let report = run_net_test(&config).unwrap();
    for c in &report.clients {
        eprintln!("{c:?}");
    }
    assert!(report.packets_lost > 0, "la red debería perder paquetes");
    assert!(
        report.blocks_dug > 100,
        "se cavó poco: {}",
        report.blocks_dug
    );
    assert!(report.blocks_placed > 0, "no se puso ningún bloque");
    assert_eq!(report.malformed, 0);
    assert!(
        report.consistent(),
        "clientes inconsistentes: {:#?}",
        report.clients
    );
    for c in &report.clients {
        assert!((140.0..260.0).contains(&c.rtt_ms), "rtt {}", c.rtt_ms);
        // Presupuesto del plan: 32 KB/s por cliente de media.
        assert!(
            c.down_bytes_per_s < 32_000.0,
            "bajada {} B/s",
            c.down_bytes_per_s
        );
        assert!(
            c.up_bytes_per_s < 8_000.0,
            "subida {} B/s",
            c.up_bytes_per_s
        );
    }
}

#[test]
fn crowded_servers_still_send_every_client_its_position() {
    // Con 80 bots, una instantánea con todos los jugadores no cabría en un
    // paquete; el servidor envía solo los más cercanos.
    let report = run_net_test(&NetTestConfig {
        clients: 1,
        bots: 80,
        conditions: LinkConditions::PERFECT,
        play_ms: 3_000,
        settle_ms: 1_000,
        ..NetTestConfig::default()
    })
    .unwrap();
    assert!(report.consistent(), "{:#?}", report.clients);
    assert!(report.clients[0].prediction.snapshots > 60);
}

#[test]
fn perfect_local_network_needs_no_corrections() {
    let report = run_net_test(&NetTestConfig {
        clients: 1,
        bots: 0,
        conditions: LinkConditions::PERFECT,
        play_ms: 10_000,
        settle_ms: 1_000,
        ..NetTestConfig::default()
    })
    .unwrap();
    assert!(report.consistent(), "{:#?}", report.clients);
    let prediction = report.clients[0].prediction;
    assert!(prediction.snapshots > 250);
    // Sin latencia la predicción coincide casi siempre con el servidor.
    assert!(
        prediction.corrections * 20 < prediction.snapshots,
        "{prediction:?}"
    );
}

#[test]
fn same_seed_same_outcome() {
    let config = NetTestConfig {
        play_ms: 5_000,
        settle_ms: 1_000,
        ..NetTestConfig::default()
    };
    let a = run_net_test(&config).unwrap();
    let b = run_net_test(&config).unwrap();
    assert_eq!(a.server_world_hash, b.server_world_hash);
    assert_eq!(a.blocks_dug, b.blocks_dug);
}
