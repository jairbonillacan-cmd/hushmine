//! Subcomandos `story` y `region` del modo Historia: una expedición sin
//! gráficos con informe JSON, y la ida y vuelta de una región `.hush`.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::create_parent_dir;

/// Corre una expedición del modo historia con un minero automático simple
/// (camina, salta y pica hacia abajo cada cierto tiempo) y recopila el
/// informe en JSON.
pub fn story(seed: u64, ticks: u32, peaceful: bool, report_path: Option<&Path>) -> Result<()> {
    use hush_sim::MoveInput;
    use hush_sim::story::{Expedition, StoryConfig, StoryEventKind, StoryOrder};

    let config = StoryConfig {
        seed,
        peaceful,
        ..StoryConfig::default()
    };
    let mut expedition = Expedition::new(config)?;
    let mut milestones = Vec::new();
    let mut mined = 0u32;
    let mut damaged = 0u32;
    for t in 0..ticks {
        // Minero automático: avanza girando cada 300 ticks y pictea la
        // celda de sus pies cada 40 para abrir pozos de sondeo.
        let turn = (t / 300) % 4;
        let (wx, wz) = match turn {
            0 => (30, 0),
            1 => (0, 30),
            2 => (-30, 0),
            _ => (0, -30),
        };
        let input = MoveInput {
            wish_x: wx.clamp(-127, 127) as i8,
            wish_z: wz.clamp(-127, 127) as i8,
            jump: t % 90 == 0,
            sprint: false,
        };
        // Pica el bloque bajo los pies en tandas continuas: el progreso se
        // reinicia si la orden se alterna con `Mine(None)`.
        let mut orders = vec![StoryOrder::Move(input)];
        if t % 80 < 40 {
            let feet = expedition.feet_cell();
            orders.push(StoryOrder::Mine(Some([feet[0], feet[1] - 1, feet[2]])));
        }
        for event in expedition.step(&orders) {
            match event.kind {
                StoryEventKind::Mined(_) | StoryEventKind::CrushedToDust(_) => mined += 1,
                StoryEventKind::Damaged { amount, .. } => damaged += u32::from(amount),
                StoryEventKind::MilestoneReached(m) => milestones.push(m),
                _ => {}
            }
        }
        if expedition.is_dead() {
            break;
        }
    }
    let summary = serde_json::json!({
        "seed": seed,
        "peaceful": peaceful,
        "ticks": expedition.tick(),
        "dead": expedition.is_dead(),
        "state_hash": expedition.state_hash().to_string(),
        "position": expedition.feet_cell(),
        "stratum": expedition.bands().at(expedition.feet_cell()[1]).label(),
        "clock_tick": expedition.clock().tick(),
        "phase": format!("{:?}", expedition.clock().phase()),
        "health": expedition.vitals().health,
        "food": expedition.vitals().food,
        "air": expedition.vitals().air,
        "milestones": milestones,
        "mined_or_crushed": mined,
        "damage_taken": damaged,
        "mobs_alive": expedition.mobs().list.len(),
        "world_hash": expedition.world().content_hash().to_string(),
    });
    let json = serde_json::to_string_pretty(&summary)?;
    match report_path {
        Some(path) => {
            create_parent_dir(path)?;
            fs::write(path, &json)
                .with_context(|| format!("no se pudo escribir {}", path.display()))?;
            println!("informe guardado en {}", path.display());
        }
        None => println!("{json}"),
    }
    Ok(())
}

/// Guarda y recarga una región `.hush`, verificando la huella del mundo.
pub fn region(seed: u64, out: &Path) -> Result<()> {
    use hush_sim::story::generate_terrain as generate_story;

    let generated = generate_story(seed, &Default::default())?;
    let bytes = hush_sim::story::region::encode_world(&generated.world);
    create_parent_dir(out)?;
    hush_sim::story::region::save_world(&generated.world, out)
        .with_context(|| format!("no se pudo guardar {}", out.display()))?;
    let loaded = hush_sim::story::region::load_world(out)
        .with_context(|| format!("no se pudo recargar {}", out.display()))?;
    anyhow::ensure!(
        loaded.content_hash() == generated.world.content_hash(),
        "la huella del mundo cambió al recargar la región"
    );
    let solid = (0..loaded.chunk_count())
        .map(|i| {
            loaded
                .chunk(i)
                .as_bytes()
                .iter()
                .filter(|&&b| b != 0)
                .count()
        })
        .sum::<usize>();
    println!(
        "región {} ({} bytes, {} bloques sólidos): huella conservada",
        out.display(),
        bytes.len(),
        solid
    );
    Ok(())
}
