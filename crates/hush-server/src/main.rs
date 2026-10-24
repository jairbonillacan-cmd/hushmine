//! Servidor dedicado de HUSHMINE sobre UDP.
//!
//! Uso: `hush-server [--port 47400] [--seed 42] [--bots 0]`.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;
use grisu_net::UdpTransport;
use hush_net::{DEFAULT_PORT, Server, ServerConfig};
use hush_sim::MatchConfig;

#[derive(Parser)]
#[command(name = "hush-server", version, about = "Servidor dedicado de HUSHMINE")]
struct Args {
    /// Puerto UDP en el que escucha.
    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,
    /// Semilla del mapa.
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Bots de prueba que cavan por el mapa.
    #[arg(long, default_value_t = 0)]
    bots: u32,
    /// Clientes como máximo.
    #[arg(long, default_value_t = 16)]
    max_clients: usize,
}

/// Cada cuánto se escribe el estado en el registro.
const STATUS_EVERY: Duration = Duration::from_secs(30);

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    let mut server = Server::new(ServerConfig {
        game: MatchConfig {
            seed: args.seed,
            max_ticks: u32::MAX,
            bots: args.bots,
            ..MatchConfig::default()
        },
        max_clients: args.max_clients,
        ..ServerConfig::default()
    })?;
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    let mut socket =
        UdpTransport::bind(addr).with_context(|| format!("no se pudo escuchar en {addr}"))?;
    log::info!(
        "servidor en {} (semilla {}, {} bots)",
        socket.local_addr()?,
        args.seed,
        args.bots
    );

    let start = Instant::now();
    let mut last_status = start;
    loop {
        let now_ms = start.elapsed().as_millis() as u64;
        server.update(now_ms, &mut socket);
        if last_status.elapsed() >= STATUS_EVERY {
            last_status = Instant::now();
            let names: Vec<&str> = server.players().map(|(name, _)| name).collect();
            log::info!(
                "tick {}, {} jugadores: {}",
                server.sim().tick(),
                names.len(),
                names.join(", ")
            );
        }
        // Un tick dura 33 ms: dormir 2 ms basta para responder a tiempo
        // sin gastar CPU.
        std::thread::sleep(Duration::from_millis(2));
    }
}
