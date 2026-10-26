//! Pruebas de `mobs`.

use super::*;

fn flat_world() -> VoxelWorld {
    let mut w = VoxelWorld::new([48, 16, 48]).unwrap();
    w.fill([0, 0, 0], [47, 2, 47], 1);
    w
}

impl<'a> TickContext<'a> {
    /// Contexto de prueba: mundo, jugador y luz constante.
    fn with(
        world: &'a VoxelWorld,
        player: [i32; 3],
        light: &'a dyn Fn(i32, i32, i32) -> u8,
        rng: &'a mut crate::Rng,
    ) -> TickContext<'a> {
        TickContext {
            world,
            player_cell: player,
            noise: None,
            light,
            rng,
        }
    }
}

#[test]
fn spawns_in_darkness_within_range() {
    let world = flat_world();
    let mut rng = crate::Rng::new(5);
    // Forzamos el intervalo.
    let mut mobs = Mobs {
        spawn_timer: SPAWN_INTERVAL_TICKS,
        ..Mobs::default()
    };
    let light = |_, _, _| 0u8;
    let mut ctx = TickContext::with(&world, [24, 3, 24], &light, &mut rng);
    let events = mobs.spawn_pass(&mut ctx);
    assert!(!events.is_empty(), "no generó nada en oscuridad total");
    assert!(mobs.list.len() <= MAX_MOBS);
    for mob in &mobs.list {
        let dist = manhattan(mob.cell, [24, 3, 24]);
        assert!((SPAWN_MIN_DISTANCE..=SPAWN_MAX_DISTANCE).contains(&dist));
    }
}

#[test]
fn does_not_spawn_in_light() {
    let world = flat_world();
    let mut rng = crate::Rng::new(5);
    let mut mobs = Mobs {
        spawn_timer: SPAWN_INTERVAL_TICKS,
        ..Mobs::default()
    };
    let light = |_, _, _| SPAWN_MAX_LIGHT + 1;
    let mut ctx = TickContext::with(&world, [24, 3, 24], &light, &mut rng);
    assert!(mobs.spawn_pass(&mut ctx).is_empty());
}

#[test]
fn stalker_hisses_then_explodes() {
    let world = flat_world();
    let mut rng = crate::Rng::new(1);
    let mut mobs = Mobs::default();
    mobs.list.push(Mob {
        kind: MobKind::DustStalker,
        cell: [24, 3, 26],
        health: 8,
        state: MobState::Wander,
        target: [24, 3, 26],
        cooldown: 0,
    });
    let player = [24, 3, 24];
    let mut hissed = false;
    let mut exploded = false;
    let light = |_, _, _| 0u8;
    for t in 0..(FUSE_TICKS + 2) {
        let mut ctx = TickContext::with(&world, player, &light, &mut rng);
        let (events, blasts) = mobs.step(t, &mut ctx);
        if events.contains(&MobEvent::Hiss([24, 3, 26])) {
            hissed = true;
        }
        if !blasts.is_empty() {
            exploded = true;
            break;
        }
    }
    assert!(hissed && exploded);
    assert!(
        mobs.list.is_empty(),
        "el acechador murió en la deflagración"
    );
}

#[test]
fn brawler_follows_noise_and_melees() {
    let world = flat_world();
    let mut rng = crate::Rng::new(2);
    let mut mobs = Mobs::default();
    mobs.list.push(Mob {
        kind: MobKind::TunnelBrawler,
        cell: [24, 3, 30],
        health: 14,
        state: MobState::Wander,
        target: [24, 3, 30],
        cooldown: 0,
    });
    let player = [24, 3, 24];
    let mut hit = false;
    let light = |_, _, _| 0u8;
    for t in 0..120 {
        let mut ctx = TickContext {
            noise: Some((player, t)),
            ..TickContext::with(&world, player, &light, &mut rng)
        };
        let (events, _) = mobs.step(t, &mut ctx);
        if events.iter().any(|e| matches!(e, MobEvent::HitPlayer(_))) {
            hit = true;
            break;
        }
    }
    assert!(hit, "el bregador nunca alcanzó al jugador");
}

#[test]
fn swarm_spawns_on_gravel_dig() {
    let mut mobs = Mobs::default();
    mobs.spawn_swarm([10, 3, 10], 3);
    assert_eq!(mobs.list.len(), 3);
    assert!(mobs.list.iter().all(|m| m.kind == MobKind::CreviceVermin));
}
