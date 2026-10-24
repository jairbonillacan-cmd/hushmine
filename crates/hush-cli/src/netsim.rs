//! `hush-cli netsim`: partida en red simulada y comprobación de consistencia.

use std::path::Path;

use anyhow::{Result, ensure};
use grisu_net::LinkConditions;
use hush_net::harness::{NetTestConfig, run_net_test};

pub struct Options {
    pub seed: u64,
    pub clients: usize,
    pub bots: u32,
    pub rtt: u32,
    pub loss: f32,
    pub seconds: u64,
}

pub fn run(opts: Options, out: Option<&Path>) -> Result<()> {
    let config = NetTestConfig {
        seed: opts.seed,
        clients: opts.clients,
        bots: opts.bots,
        conditions: LinkConditions {
            latency_ms: opts.rtt / 2,
            jitter_ms: 10,
            loss_per_mille: (opts.loss * 10.0).round().clamp(0.0, 1000.0) as u32,
            duplicate_per_mille: 2,
        },
        play_ms: opts.seconds * 1000,
        settle_ms: 3000,
    };
    let report = run_net_test(&config)?;
    let clients: Vec<_> = report
        .clients
        .iter()
        .map(|c| {
            serde_json::json!({
                "name": c.name,
                "state": format!("{:?}", c.state),
                "world_matches": c.world_matches,
                "position_error": c.position_error,
                "snapshots": c.prediction.snapshots,
                "corrections": c.prediction.corrections,
                "max_prediction_error": c.prediction.max_error,
                "blocks_updated": c.prediction.blocks_updated,
                "rtt_ms": c.rtt_ms,
                "down_bytes_per_s": c.down_bytes_per_s,
                "up_bytes_per_s": c.up_bytes_per_s,
            })
        })
        .collect();
    let json = serde_json::json!({
        "seed": opts.seed,
        "rtt_ms": opts.rtt,
        "loss_percent": opts.loss,
        "seconds": opts.seconds,
        "bots": opts.bots,
        "consistent": report.consistent(),
        "server_world_hash": format!("{:016x}", report.server_world_hash),
        "blocks_dug": report.blocks_dug,
        "blocks_placed": report.blocks_placed,
        "packets_lost": report.packets_lost,
        "malformed": report.malformed,
        "clients": clients,
    });
    let text = serde_json::to_string_pretty(&json)?;
    match out {
        Some(path) => {
            crate::create_parent_dir(path)?;
            std::fs::write(path, &text)?;
            println!("informe guardado en {}", path.display());
        }
        None => println!("{text}"),
    }
    ensure!(
        report.consistent(),
        "los clientes no coinciden con el servidor"
    );
    Ok(())
}
