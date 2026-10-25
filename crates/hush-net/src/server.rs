//! Servidor autoritativo: recibe entradas, corre la simulación a 30 Hz y
//! envía el mapa al unirse, los cambios de bloques y una instantánea por tick.
//!
//! No hace E/S: [`Server::update`] recibe la hora y un [`Socket`].

use std::collections::BTreeMap;

use grisu_net::{Connection, PeerAddr, Socket};
use hush_sim::{BlockChange, Command, MatchConfig, MoveInput, Simulation, TICK_RATE};

use crate::proto::{
    Action, ClientMsg, InputFrame, MAX_REMOTES_PER_SNAPSHOT, PROTOCOL_VERSION, RemotePlayer,
    ServerMsg,
};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub game: MatchConfig,
    pub max_clients: usize,
    /// Sin paquetes durante este tiempo, el cliente se da por desconectado.
    pub timeout_ms: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            game: MatchConfig {
                max_ticks: u32::MAX,
                ..MatchConfig::default()
            },
            max_clients: 16,
            timeout_ms: 10_000,
        }
    }
}

/// Entradas guardadas como máximo por cliente.
const MAX_QUEUED_INPUTS: usize = 64;
/// Ticks como máximo por llamada a `update` (si el servidor se atrasa).
const MAX_TICKS_PER_UPDATE: u64 = 8;

struct RemoteClient {
    conn: Connection,
    name: String,
    player: Option<usize>,
    inputs: BTreeMap<u32, InputFrame>,
    last_processed: u32,
    last_movement: MoveInput,
}

pub struct Server {
    config: ServerConfig,
    sim: Simulation,
    clients: BTreeMap<PeerAddr, RemoteClient>,
    start_ms: Option<u64>,
    ticks_run: u64,
    malformed: u64,
}

impl Server {
    pub fn new(config: ServerConfig) -> Result<Self, grisu_voxel::WorldError> {
        Ok(Self {
            sim: Simulation::new(config.game.clone())?,
            config,
            clients: BTreeMap::new(),
            start_ms: None,
            ticks_run: 0,
            malformed: 0,
        })
    }

    pub fn sim(&self) -> &Simulation {
        &self.sim
    }

    /// Activa o detiene a los bots de la partida.
    pub fn set_bots_active(&mut self, active: bool) {
        self.sim.set_bots_active(active);
    }

    /// Clientes conectados que ya juegan.
    pub fn players(&self) -> impl Iterator<Item = (&str, usize)> {
        self.clients
            .values()
            .filter_map(|c| c.player.map(|p| (c.name.as_str(), p)))
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Mensajes o paquetes mal formados descartados.
    pub fn malformed(&self) -> u64 {
        self.malformed
    }

    /// Recibe, simula los ticks que tocan y envía.
    pub fn update(&mut self, now_ms: u64, socket: &mut dyn Socket) {
        let start = *self.start_ms.get_or_insert(now_ms);
        self.receive(now_ms, socket);
        self.drop_timed_out(now_ms);
        let due = (now_ms - start) * u64::from(TICK_RATE) / 1000;
        let mut ran = 0;
        while self.ticks_run < due && ran < MAX_TICKS_PER_UPDATE {
            self.tick();
            self.ticks_run += 1;
            ran += 1;
        }
        // Si se atrasó demasiado, no intenta recuperar el tiempo perdido.
        self.ticks_run = self.ticks_run.max(due.saturating_sub(MAX_TICKS_PER_UPDATE));
        for (addr, client) in &mut self.clients {
            for packet in client.conn.flush(now_ms) {
                socket.send_to(*addr, &packet);
            }
        }
    }

    fn receive(&mut self, now_ms: u64, socket: &mut dyn Socket) {
        while let Some((addr, packet)) = socket.recv_from() {
            if !self.clients.contains_key(&addr) {
                if self.clients.len() >= self.config.max_clients {
                    continue;
                }
                self.clients.insert(
                    addr,
                    RemoteClient {
                        conn: Connection::new(now_ms),
                        name: String::new(),
                        player: None,
                        inputs: BTreeMap::new(),
                        last_processed: 0,
                        last_movement: MoveInput::default(),
                    },
                );
            }
            let Some(client) = self.clients.get_mut(&addr) else {
                continue;
            };
            if client.conn.receive(now_ms, &packet).is_err() {
                self.malformed += 1;
            }
        }
        let addrs: Vec<PeerAddr> = self.clients.keys().copied().collect();
        for addr in addrs {
            let messages: Vec<_> = match self.clients.get_mut(&addr) {
                Some(c) => c.conn.drain_messages().collect(),
                None => continue,
            };
            for message in messages {
                match ClientMsg::decode(&message.data) {
                    Ok(msg) => self.handle(addr, msg),
                    Err(err) => {
                        self.malformed += 1;
                        log::debug!("mensaje inválido de {addr}: {err}");
                    }
                }
            }
        }
    }

    fn handle(&mut self, addr: PeerAddr, msg: ClientMsg) {
        match msg {
            ClientMsg::Hello { version, name } => self.join(addr, version, name),
            ClientMsg::Inputs(frames) => {
                let Some(client) = self.clients.get_mut(&addr) else {
                    return;
                };
                for frame in frames {
                    if frame.seq > client.last_processed && client.inputs.len() < MAX_QUEUED_INPUTS
                    {
                        client.inputs.insert(frame.seq, frame);
                    }
                }
            }
            ClientMsg::Bye => self.disconnect(addr, "se despidió"),
        }
    }

    fn join(&mut self, addr: PeerAddr, version: u16, name: String) {
        let Some(client) = self.clients.get_mut(&addr) else {
            return;
        };
        if client.player.is_some() {
            return;
        }
        if version != PROTOCOL_VERSION {
            let reason = format!("versión {version}; el servidor usa {PROTOCOL_VERSION}");
            client
                .conn
                .send_reliable(&ServerMsg::Refused(reason).encode());
            return;
        }
        let player = self.sim.spawn_player();
        client.player = Some(player);
        client.name = name;
        log::info!("{} se unió desde {addr} como jugador {player}", client.name);
        let world = self.sim.world();
        let welcome = ServerMsg::Welcome {
            player: player as u16,
            map_size: world.size(),
            tick: self.sim.tick(),
        };
        client.conn.send_reliable(&welcome.encode());
        for index in 0..world.chunk_count() {
            let chunk = ServerMsg::Chunk {
                index: index as u16,
                data: world.chunk(index).encode(),
            };
            client.conn.send_reliable(&chunk.encode());
        }
        client.conn.send_reliable(&ServerMsg::MapDone.encode());
    }

    fn disconnect(&mut self, addr: PeerAddr, reason: &str) {
        if let Some(client) = self.clients.remove(&addr)
            && let Some(player) = client.player
        {
            self.sim.remove_player(player);
            log::info!("{} ({addr}) salió: {reason}", client.name);
        }
    }

    fn drop_timed_out(&mut self, now_ms: u64) {
        let timeout = self.config.timeout_ms;
        let gone: Vec<PeerAddr> = self
            .clients
            .iter()
            .filter(|(_, c)| now_ms.saturating_sub(c.conn.last_recv_ms()) > timeout)
            .map(|(a, _)| *a)
            .collect();
        for addr in gone {
            self.disconnect(addr, "tiempo de espera agotado");
        }
    }

    fn tick(&mut self) {
        for client in self.clients.values_mut() {
            let Some(player) = client.player else {
                continue;
            };
            // La siguiente entrada en orden; si se perdió, la primera que haya.
            let next = client
                .inputs
                .remove(&(client.last_processed + 1))
                .or_else(|| client.inputs.pop_first().map(|(_, f)| f));
            let movement = match next {
                Some(frame) => {
                    client.last_processed = frame.seq;
                    client.last_movement = frame.movement;
                    if let Some(action) = frame.action {
                        self.sim.submit(match action {
                            Action::Dig(cell) => Command::Dig { player, cell },
                            Action::Place(cell, block) => Command::Place {
                                player,
                                cell,
                                block,
                            },
                        });
                    }
                    frame.movement
                }
                // Sin entrada nueva, sigue con la última (sin saltar).
                None => MoveInput {
                    jump: false,
                    ..client.last_movement
                },
            };
            self.sim.submit(Command::Move {
                player,
                input: movement,
            });
        }
        self.sim.step();
        self.send_changes();
        self.send_snapshots();
    }

    fn send_changes(&mut self) {
        let changes = self.sim.take_changes();
        if !changes.is_empty() {
            let msg = ServerMsg::Changes(changes).encode();
            for client in self.clients.values_mut().filter(|c| c.player.is_some()) {
                client.conn.send_reliable(&msg);
            }
        }
        // Quien predijo una acción rechazada recibe el valor real de la celda.
        for rejection in self.sim.take_rejections() {
            let [x, y, z] = rejection.cell;
            if !self.sim.world().in_bounds(x, y, z) {
                continue;
            }
            let fix = ServerMsg::Changes(vec![BlockChange {
                cell: rejection.cell,
                block: self.sim.world().get(x, y, z),
            }]);
            let target = self
                .clients
                .values_mut()
                .find(|c| c.player == Some(rejection.player));
            if let Some(client) = target {
                client.conn.send_reliable(&fix.encode());
            }
        }
    }

    fn send_snapshots(&mut self) {
        let tick = self.sim.tick();
        let everyone: Vec<RemotePlayer> = self
            .sim
            .active_players()
            .map(|(id, body)| RemotePlayer {
                id: id as u16,
                feet: body.feet,
            })
            .collect();
        for client in self.clients.values_mut() {
            let Some(player) = client.player else {
                continue;
            };
            let you = self.sim.players()[player];
            let others = nearest_others(&everyone, player, you.feet);
            let msg = ServerMsg::Snapshot {
                tick,
                last_input: client.last_processed,
                you,
                others,
            };
            client.conn.send_unreliable(msg.encode());
        }
    }
}

/// Los jugadores más cercanos a `feet` (sin contar a `player`), como mucho
/// [`MAX_REMOTES_PER_SNAPSHOT`] para que la instantánea quepa en un paquete.
/// Los empates se deshacen por id, así que el resultado es determinista.
fn nearest_others(everyone: &[RemotePlayer], player: usize, feet: [i64; 3]) -> Vec<RemotePlayer> {
    let mut others: Vec<RemotePlayer> = everyone
        .iter()
        .filter(|o| usize::from(o.id) != player)
        .copied()
        .collect();
    if others.len() > MAX_REMOTES_PER_SNAPSHOT {
        // Distancia en bloques enteros: cabe en i64 en todo el mundo grande.
        let block = |c: i64| c >> grisu_voxel::FIXED_SHIFT;
        let distance2 = |o: &RemotePlayer| -> i64 {
            (0..3)
                .map(|i| (block(o.feet[i]) - block(feet[i])).pow(2))
                .sum()
        };
        others.sort_by_key(|o| (distance2(o), o.id));
        others.truncate(MAX_REMOTES_PER_SNAPSHOT);
    }
    others
}

#[cfg(test)]
mod tests {
    use super::*;
    use grisu_voxel::FIXED_ONE;

    fn at(id: u16, x: i64) -> RemotePlayer {
        RemotePlayer {
            id,
            feet: [x * FIXED_ONE, 0, 0],
        }
    }

    #[test]
    fn few_players_are_all_sent_except_yourself() {
        let everyone = [at(0, 0), at(1, 5), at(2, -3)];
        let others = nearest_others(&everyone, 0, [0; 3]);
        assert_eq!(others, vec![at(1, 5), at(2, -3)]);
    }

    #[test]
    fn crowds_are_cut_to_the_nearest_that_fit() {
        // El jugador 0 en x = 0 y 100 más repartidos a ambos lados.
        let everyone: Vec<RemotePlayer> = (0..=100u16)
            .map(|id| at(id, (i64::from(id) - 50) * 3))
            .collect();
        let me = everyone[0];
        let others = nearest_others(&everyone, 0, me.feet);
        assert_eq!(others.len(), MAX_REMOTES_PER_SNAPSHOT);
        assert!(others.iter().all(|o| o.id != 0));
        let farthest = others.iter().map(|o| (o.feet[0] - me.feet[0]).abs()).max();
        let left_out = everyone
            .iter()
            .filter(|o| o.id != 0 && !others.contains(o))
            .map(|o| (o.feet[0] - me.feet[0]).abs())
            .min();
        assert!(farthest <= left_out, "se dejó fuera a alguien más cerca");
    }
}
