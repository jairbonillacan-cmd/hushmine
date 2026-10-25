//! Cliente con predicción: mueve a su minero al instante con las mismas
//! reglas que el servidor y se corrige al recibir el estado autoritativo.
//!
//! No hace E/S ni lleva reloj de ticks: el juego llama a [`NetClient::poll`]
//! al recibir, a [`NetClient::tick`] 30 veces por segundo y a
//! [`NetClient::flush`] para enviar.

use std::collections::VecDeque;

use grisu_net::{Connection, PeerAddr, Socket};
use grisu_voxel::{Chunk, VoxelWorld};
use hush_sim::{BlockChange, MoveInput, PlayerBody, try_dig, try_place};

use crate::interp::RemoteTracks;
use crate::proto::{
    Action, ClientMsg, INPUT_REDUNDANCY, InputFrame, PROTOCOL_VERSION, RemotePlayer, ServerMsg,
};

/// Entradas sin confirmar que se guardan como máximo.
const MAX_HISTORY: usize = 120;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientState {
    /// Esperando la bienvenida del servidor.
    Connecting,
    /// Recibiendo el mapa.
    Loading {
        chunks: usize,
        total: usize,
    },
    Playing,
    Disconnected(String),
}

/// Medidas de la predicción, para pruebas y depuración.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PredictionStats {
    pub snapshots: u64,
    /// Instantáneas en las que la posición predicha no coincidía.
    pub corrections: u64,
    /// Mayor diferencia encontrada, en punto fijo Q47.16 (65536 = un bloque).
    pub max_error: i64,
    /// Bloques de la réplica cambiados por el servidor (ediciones ajenas y
    /// correcciones de predicciones rechazadas).
    pub blocks_updated: u64,
}

pub struct NetClient {
    server: PeerAddr,
    conn: Connection,
    name: String,
    state: ClientState,
    player: Option<u16>,
    world: Option<VoxelWorld>,
    body: PlayerBody,
    /// ¿Llegó ya alguna instantánea con la posición real?
    has_body: bool,
    history: VecDeque<InputFrame>,
    next_seq: u32,
    last_snapshot_tick: Option<u32>,
    remotes: RemoteTracks,
    stats: PredictionStats,
}

impl NetClient {
    pub fn new(server: PeerAddr, name: &str, now_ms: u64) -> Self {
        let mut conn = Connection::new(now_ms);
        let hello = ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            name: name.chars().take(crate::proto::MAX_NAME_LEN / 4).collect(),
        };
        conn.send_reliable(&hello.encode());
        Self {
            server,
            conn,
            name: name.to_owned(),
            state: ClientState::Connecting,
            player: None,
            world: None,
            body: PlayerBody::new([0; 3]),
            has_body: false,
            history: VecDeque::new(),
            next_seq: 1,
            last_snapshot_tick: None,
            remotes: RemoteTracks::default(),
            stats: PredictionStats::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state(&self) -> &ClientState {
        &self.state
    }

    pub fn player(&self) -> Option<u16> {
        self.player
    }

    /// Réplica local del terreno (con las predicciones aplicadas).
    pub fn world(&self) -> Option<&VoxelWorld> {
        self.world.as_ref()
    }

    pub fn world_mut(&mut self) -> Option<&mut VoxelWorld> {
        self.world.as_mut()
    }

    /// Cuerpo predicho del jugador local.
    pub fn body(&self) -> &PlayerBody {
        &self.body
    }

    pub fn stats(&self) -> PredictionStats {
        self.stats
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn last_server_tick(&self) -> Option<u32> {
        self.last_snapshot_tick
    }

    /// Otros jugadores interpolados en el tick `server_tick` (en bloques,
    /// con decimales, en `f64` para conservar precisión lejos del origen).
    pub fn remote_players(&self, server_tick: f64) -> Vec<(u16, [f64; 3])> {
        self.remotes.sample(server_tick)
    }

    /// Si otro jugador iba agachado en la última instantánea.
    pub fn remote_crouching(&self, id: u16) -> bool {
        self.remotes.is_crouching(id)
    }

    /// Recibe y procesa lo que haya llegado del servidor.
    pub fn poll(&mut self, now_ms: u64, socket: &mut dyn Socket) {
        while let Some((from, packet)) = socket.recv_from() {
            if from != self.server {
                continue;
            }
            if let Err(err) = self.conn.receive(now_ms, &packet) {
                log::debug!("paquete inválido del servidor: {err}");
            }
        }
        let messages: Vec<_> = self.conn.drain_messages().collect();
        for message in messages {
            match ServerMsg::decode(&message.data) {
                Ok(msg) => self.handle(msg),
                Err(err) => log::warn!("mensaje inválido del servidor: {err}"),
            }
        }
    }

    pub fn flush(&mut self, now_ms: u64, socket: &mut dyn Socket) {
        for packet in self.conn.flush(now_ms) {
            socket.send_to(self.server, &packet);
        }
    }

    /// Se despide del servidor (el mensaje sale en el próximo `flush`).
    pub fn disconnect(&mut self) {
        self.conn.send_reliable(&ClientMsg::Bye.encode());
        self.state = ClientState::Disconnected("desconectado".into());
    }

    fn handle(&mut self, msg: ServerMsg) {
        match msg {
            ServerMsg::Welcome {
                player, map_size, ..
            } => match VoxelWorld::new(map_size) {
                Ok(world) => {
                    self.state = ClientState::Loading {
                        chunks: 0,
                        total: world.chunk_count(),
                    };
                    self.world = Some(world);
                    self.player = Some(player);
                }
                Err(err) => self.state = ClientState::Disconnected(err.to_string()),
            },
            ServerMsg::Chunk { index, data } => {
                let (Some(world), Ok(chunk)) = (self.world.as_mut(), Chunk::decode(&data)) else {
                    log::warn!("chunk {index} inválido o sin mapa");
                    return;
                };
                world.replace_chunk(usize::from(index), chunk);
                if let ClientState::Loading { chunks, .. } = &mut self.state {
                    *chunks += 1;
                }
            }
            ServerMsg::MapDone => {
                if self.world.is_some() {
                    self.state = ClientState::Playing;
                }
            }
            ServerMsg::Changes(changes) => self.apply_changes(&changes),
            ServerMsg::Snapshot {
                tick,
                last_input,
                you,
                others,
            } => self.reconcile(tick, last_input, you, &others),
            ServerMsg::Refused(reason) => self.state = ClientState::Disconnected(reason),
        }
    }

    fn apply_changes(&mut self, changes: &[BlockChange]) {
        let Some(world) = self.world.as_mut() else {
            return;
        };
        for c in changes {
            let [x, y, z] = c.cell;
            if world.get(x, y, z) != c.block {
                world.set(x, y, z, c.block);
                self.stats.blocks_updated += 1;
            }
        }
    }

    /// Adopta el estado del servidor y vuelve a aplicar las entradas que
    /// aún no ha procesado.
    fn reconcile(&mut self, tick: u32, last_input: u32, you: PlayerBody, others: &[RemotePlayer]) {
        if self.last_snapshot_tick.is_some_and(|t| tick <= t) {
            return;
        }
        self.last_snapshot_tick = Some(tick);
        self.remotes.push(tick, others);
        let Some(world) = self.world.as_ref() else {
            return;
        };
        // Hasta conocer la posición real no hay nada que predecir.
        if self.state != ClientState::Playing || !self.has_body {
            self.body = you;
            self.has_body = true;
            return;
        }
        self.stats.snapshots += 1;
        while self.history.front().is_some_and(|f| f.seq <= last_input) {
            self.history.pop_front();
        }
        let predicted = self.body;
        let mut body = you;
        for frame in &self.history {
            body.step(world, frame.movement);
        }
        let error = (0..3)
            .map(|i| (predicted.feet[i] - body.feet[i]).abs())
            .max()
            .unwrap_or(0);
        if error > 0 {
            self.stats.corrections += 1;
            self.stats.max_error = self.stats.max_error.max(error);
        }
        self.body = body;
    }

    /// Un tick de juego local: predice la acción y el movimiento y envía la
    /// entrada (junto con las anteriores, por si se pierden paquetes).
    pub fn tick(&mut self, movement: MoveInput, action: Option<Action>) {
        if self.state != ClientState::Playing || !self.has_body {
            return;
        }
        let Some(world) = self.world.as_mut() else {
            return;
        };
        // Predicción optimista: si el servidor la rechaza, mandará el valor
        // real de la celda.
        match action {
            Some(Action::Dig(cell)) => {
                let _ = try_dig(world, &self.body, cell);
            }
            Some(Action::Place(cell, block)) => {
                let _ = try_place(world, &self.body, &[], cell, block);
            }
            None => {}
        }
        self.body.step(world, movement);
        let frame = InputFrame {
            seq: self.next_seq,
            movement,
            action,
        };
        self.next_seq += 1;
        self.history.push_back(frame);
        if self.history.len() > MAX_HISTORY {
            self.history.pop_front();
        }
        let recent: Vec<InputFrame> = self
            .history
            .iter()
            .rev()
            .take(INPUT_REDUNDANCY)
            .rev()
            .copied()
            .collect();
        self.conn
            .send_unreliable(ClientMsg::Inputs(recent).encode());
    }
}
