//! Reproduce el ciclo del cliente sin gráficos: apuntar con un raycast desde
//! los ojos, pedir a la simulación que cave y avanzar hacia el hueco.

use grisu_voxel::{FIXED_ONE, raycast};
use hush_sim::{Command, MatchConfig, MoveInput, Simulation};

fn eye(sim: &Simulation, player: usize) -> [f32; 3] {
    sim.players()[player]
        .eye()
        .map(|c| c as f32 / FIXED_ONE as f32)
}

#[test]
fn aiming_and_digging_carves_a_walkable_tunnel() {
    let mut sim = Simulation::new(MatchConfig {
        seed: 11,
        max_ticks: u32::MAX,
        ..MatchConfig::default()
    })
    .unwrap();
    let player = sim.spawn_player();
    // Cae hasta el suelo.
    for _ in 0..60 {
        sim.step();
    }
    let start = sim.players()[player].feet;
    // Mira en diagonal hacia abajo, en dirección al centro del mapa.
    let toward_center = if start[0] < 32 * FIXED_ONE { 1.0 } else { -1.0 };
    let aim = [toward_center, -0.6, 0.0];

    let mut dug = 0;
    for _ in 0..120 {
        if let Some(hit) = raycast(sim.world(), eye(&sim, player), aim, 4.0) {
            sim.submit(Command::Dig {
                player,
                cell: hit.block,
            });
        }
        sim.submit(Command::Move {
            player,
            input: MoveInput::from_direction(toward_center, 0.0, false),
        });
        let before: u32 = sim.stats().dug.values().sum();
        sim.step();
        dug += sim.stats().dug.values().sum::<u32>() - before;
    }

    let end = sim.players()[player].feet;
    assert!(dug >= 10, "solo se cavaron {dug} bloques");
    assert!(end[1] < start[1], "el minero debería haber bajado");
    assert!(
        (end[0] - start[0]).abs() > 3 * FIXED_ONE,
        "el minero debería haber avanzado"
    );
    assert!(
        sim.stats().rejected.is_empty(),
        "órdenes rechazadas: {:?}",
        sim.stats().rejected
    );
}
