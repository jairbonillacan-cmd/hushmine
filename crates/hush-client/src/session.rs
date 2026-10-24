//! Conexión del juego con un servidor: uno local en memoria (partida de un
//! jugador, con el mismo código de red) o uno remoto por UDP.

use std::net::SocketAddr;
use std::time::Instant;

use anyhow::{Context, Result};
use grisu_net::{LinkConditions, PeerAddr, SimNetwork, UdpTransport};
use hush_net::{Action, NetClient, Server, ServerConfig};
use hush_sim::{MatchConfig, MoveInput};

const LOCAL_SERVER: PeerAddr = PeerAddr::Memory(0);
const LOCAL_CLIENT: PeerAddr = PeerAddr::Memory(1);

enum Link {
    Local {
        server: Box<Server>,
        net: SimNetwork,
    },
    Udp(UdpTransport),
}

pub struct Session {
    link: Link,
    client: NetClient,
    start: Instant,
}

impl Session {
    /// Partida local: el servidor corre dentro del cliente.
    pub fn local(seed: u64, bots: u32, name: &str) -> Result<Self> {
        let server = Server::new(ServerConfig {
            game: MatchConfig {
                seed,
                max_ticks: u32::MAX,
                bots,
                ..MatchConfig::default()
            },
            ..ServerConfig::default()
        })?;
        Ok(Self {
            link: Link::Local {
                server: Box::new(server),
                net: SimNetwork::new(LinkConditions::PERFECT, seed),
            },
            client: NetClient::new(LOCAL_SERVER, name, 0),
            start: Instant::now(),
        })
    }

    /// Partida en un servidor remoto.
    pub fn connect(server: SocketAddr, name: &str) -> Result<Self> {
        let local: SocketAddr = if server.is_ipv4() {
            ([0, 0, 0, 0], 0).into()
        } else {
            (std::net::Ipv6Addr::UNSPECIFIED, 0).into()
        };
        let socket = UdpTransport::bind(local).context("no se pudo abrir un socket UDP")?;
        Ok(Self {
            link: Link::Udp(socket),
            client: NetClient::new(PeerAddr::Udp(server), name, 0),
            start: Instant::now(),
        })
    }

    pub fn now_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    pub fn client(&self) -> &NetClient {
        &self.client
    }

    pub fn client_mut(&mut self) -> &mut NetClient {
        &mut self.client
    }

    /// Recibe lo pendiente (y, en local, hace avanzar al servidor).
    pub fn poll(&mut self) {
        let now = self.now_ms();
        match &mut self.link {
            Link::Local { server, net } => {
                server.update(now, &mut net.socket(LOCAL_SERVER, now));
                self.client.poll(now, &mut net.socket(LOCAL_CLIENT, now));
            }
            Link::Udp(socket) => self.client.poll(now, socket),
        }
    }

    pub fn tick(&mut self, movement: MoveInput, action: Option<Action>) {
        self.client.tick(movement, action);
    }

    pub fn flush(&mut self) {
        let now = self.now_ms();
        match &mut self.link {
            Link::Local { net, .. } => self.client.flush(now, &mut net.socket(LOCAL_CLIENT, now)),
            Link::Udp(socket) => self.client.flush(now, socket),
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // Despedida de cortesía; si se pierde, el servidor espera el tiempo
        // de desconexión.
        self.client.disconnect();
        self.flush();
    }
}
