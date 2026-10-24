//! Partida en red completa dentro de un proceso: un servidor, varios
//! clientes con un guion (andar, cavar, poner) y bots, sobre una red
//! simulada con latencia y pérdida. La usan las pruebas y `hush-cli netsim`.

use grisu_net::{LinkConditions, PeerAddr, SimNetwork};
use grisu_voxel::{FIXED_ONE, raycast};
use hush_sim::{BlockKind, MatchConfig, MoveInput, Rng, TICK_RATE};

use crate::{Action, ClientState, NetClient, PredictionStats, Server, ServerConfig};

#[derive(Clone, Debug)]
pub struct NetTestConfig {
    pub seed: u64,
    pub clients: usize,
    pub bots: u32,
    pub conditions: LinkConditions,
    /// Tiempo de juego activo.
    pub play_ms: u64,
    /// Tiempo final sin acciones para que todo llegue.
    pub settle_ms: u64,
}

impl Default for NetTestConfig {
    /// La condición de salida del hito 2: 2 clientes, 6 bots, 150 ms de ida
    /// y vuelta y un 2 % de pérdida.
    fn default() -> Self {
        Self {
            seed: 42,
            clients: 2,
            bots: 6,
            conditions: LinkConditions {
                latency_ms: 75,
                jitter_ms: 10,
                loss_per_mille: 20,
                duplicate_per_mille: 2,
            },
            play_ms: 60_000,
            settle_ms: 3_000,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClientReport {
    pub name: String,
    pub state: ClientState,
    pub world_matches: bool,
    /// Diferencia final entre la posición predicha y la del servidor.
    pub position_error: i32,
    pub prediction: PredictionStats,
    pub rtt_ms: f64,
    /// Media de bytes por segundo del servidor a este cliente y al revés.
    pub down_bytes_per_s: f64,
    pub up_bytes_per_s: f64,
}

#[derive(Clone, Debug)]
pub struct NetTestReport {
    pub clients: Vec<ClientReport>,
    pub server_world_hash: u64,
    pub blocks_dug: u32,
    pub blocks_placed: u32,
    pub packets_lost: u64,
    pub malformed: u64,
}

impl NetTestReport {
    /// ¿Terminaron todos los clientes con el mismo mundo y la misma
    /// posición que el servidor?
    pub fn consistent(&self) -> bool {
        self.clients
            .iter()
            .all(|c| c.state == ClientState::Playing && c.world_matches && c.position_error == 0)
    }
}

const SERVER: PeerAddr = PeerAddr::Memory(0);
/// Paso del reloj virtual.
const STEP_MS: u64 = 5;

/// Comportamiento guionizado de un cliente de prueba.
struct Script {
    rng: Rng,
    heading: [f32; 2],
}

impl Script {
    fn decide(
        &mut self,
        client: &NetClient,
        tick: u64,
        active: bool,
    ) -> (MoveInput, Option<Action>) {
        if !active {
            return (MoveInput::default(), None);
        }
        if tick.is_multiple_of(60) {
            let angle = self.rng.below(360) as f32 * std::f32::consts::PI / 180.0;
            self.heading = [angle.cos(), angle.sin()];
        }
        let movement =
            MoveInput::from_direction(self.heading[0], self.heading[1], self.rng.below(25) == 0);
        let mut action = None;
        if tick.is_multiple_of(6)
            && let Some(world) = client.world()
        {
            let eye = client.body().eye().map(|c| c as f32 / FIXED_ONE as f32);
            let down = if self.rng.below(3) == 0 { -1.5 } else { -0.5 };
            let aim = [self.heading[0], down, self.heading[1]];
            if let Some(hit) = raycast(world, eye, aim, 4.0) {
                action = Some(if self.rng.below(8) == 0 {
                    let cell = std::array::from_fn(|i| hit.block[i] + hit.normal[i]);
                    Action::Place(cell, BlockKind::Rock)
                } else {
                    Action::Dig(hit.block)
                });
            }
        }
        (movement, action)
    }
}

pub fn run_net_test(config: &NetTestConfig) -> Result<NetTestReport, grisu_voxel::WorldError> {
    let mut server = Server::new(ServerConfig {
        game: MatchConfig {
            seed: config.seed,
            max_ticks: u32::MAX,
            bots: config.bots,
            ..MatchConfig::default()
        },
        ..ServerConfig::default()
    })?;
    let mut net = SimNetwork::new(config.conditions, config.seed ^ 0x5EED);
    let mut clients: Vec<(PeerAddr, NetClient, Script)> = (0..config.clients)
        .map(|i| {
            let addr = PeerAddr::Memory(i as u32 + 1);
            let script = Script {
                rng: Rng::new(config.seed.wrapping_add(i as u64 * 7919)),
                heading: [1.0, 0.0],
            };
            (
                addr,
                NetClient::new(SERVER, &format!("prueba{i}"), 0),
                script,
            )
        })
        .collect();

    let end = config.play_ms + config.settle_ms;
    let mut client_ticks = 0u64;
    let mut now = 0;
    let mut bots_paused = false;
    while now <= end {
        // Al final, nadie actúa: así todo lo enviado llega antes de comparar.
        if now >= config.play_ms && !bots_paused {
            server.set_bots_active(false);
            bots_paused = true;
        }
        server.update(now, &mut net.socket(SERVER, now));
        let due = now * u64::from(TICK_RATE) / 1000;
        let ticking = client_ticks < due;
        for (addr, client, script) in &mut clients {
            client.poll(now, &mut net.socket(*addr, now));
            if ticking {
                let (movement, action) = script.decide(client, client_ticks, now < config.play_ms);
                client.tick(movement, action);
            }
            client.flush(now, &mut net.socket(*addr, now));
        }
        if ticking {
            client_ticks += 1;
        }
        now += STEP_MS;
    }

    let world = server.sim().world();
    let server_hash = world.content_hash();
    let seconds = (end as f64 / 1000.0).max(0.001);
    let up_total: u64 = clients
        .iter()
        .map(|(_, c, _)| c.connection().stats().bytes_sent)
        .sum();
    let down_total = net.stats().bytes_sent.saturating_sub(up_total);
    let reports = clients
        .iter()
        .map(|(_, client, _)| {
            let server_body = client
                .player()
                .and_then(|p| server.sim().players().get(usize::from(p)));
            let position_error = server_body.map_or(i32::MAX, |b| {
                (0..3)
                    .map(|i| (b.feet[i] - client.body().feet[i]).abs())
                    .max()
                    .unwrap_or(0)
            });
            let stats = client.connection().stats();
            ClientReport {
                name: client.name().to_owned(),
                state: client.state().clone(),
                world_matches: client.world().map(|w| w.content_hash()) == Some(server_hash),
                position_error,
                prediction: client.stats(),
                rtt_ms: stats.rtt_ms,
                down_bytes_per_s: down_total as f64 / config.clients.max(1) as f64 / seconds,
                up_bytes_per_s: stats.bytes_sent as f64 / seconds,
            }
        })
        .collect();
    let edits = server.sim().stats();
    Ok(NetTestReport {
        clients: reports,
        server_world_hash: server_hash,
        blocks_dug: edits.dug.values().sum(),
        blocks_placed: edits.placed,
        packets_lost: net.stats().lost,
        malformed: server.malformed(),
    })
}
