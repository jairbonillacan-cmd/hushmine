use std::net::{SocketAddr, UdpSocket};

/// Dirección de un extremo, sea cual sea el transporte.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PeerAddr {
    Udp(SocketAddr),
    /// Extremo de una [`SimNetwork`](crate::SimNetwork) en memoria.
    Memory(u32),
}

impl std::fmt::Display for PeerAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Udp(addr) => write!(f, "{addr}"),
            Self::Memory(id) => write!(f, "mem:{id}"),
        }
    }
}

/// Transporte de paquetes sin conexión ni garantías (como UDP).
///
/// Los mensajes del juego no dependen del transporte: el servidor y el
/// cliente solo ven este trait.
pub trait Socket {
    fn send_to(&mut self, to: PeerAddr, data: &[u8]);
    /// Siguiente paquete recibido, si hay alguno. Nunca bloquea.
    fn recv_from(&mut self) -> Option<(PeerAddr, Vec<u8>)>;
}

/// Tamaño máximo de paquete que se envía o se acepta.
pub const MAX_PACKET: usize = 1400;

/// Socket UDP no bloqueante.
pub struct UdpTransport {
    socket: UdpSocket,
    buf: Vec<u8>,
}

impl UdpTransport {
    pub fn bind(addr: SocketAddr) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket,
            buf: vec![0; MAX_PACKET + 64],
        })
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.socket.local_addr()
    }
}

impl Socket for UdpTransport {
    fn send_to(&mut self, to: PeerAddr, data: &[u8]) {
        let PeerAddr::Udp(addr) = to else {
            log::warn!("dirección no UDP en un socket UDP: {to}");
            return;
        };
        if let Err(err) = self.socket.send_to(data, addr)
            && err.kind() != std::io::ErrorKind::WouldBlock
        {
            log::debug!("error enviando a {addr}: {err}");
        }
    }

    fn recv_from(&mut self) -> Option<(PeerAddr, Vec<u8>)> {
        loop {
            match self.socket.recv_from(&mut self.buf) {
                Ok((n, from)) if n <= MAX_PACKET => {
                    return Some((PeerAddr::Udp(from), self.buf[..n].to_vec()));
                }
                // Paquete demasiado grande: se descarta.
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => return None,
                // En Windows, un ICMP «puerto inalcanzable» llega como
                // `ConnectionReset` en el siguiente recv: se ignora.
                Err(err) => log::debug!("error recibiendo: {err}"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn udp_loopback_round_trip() {
        let localhost = SocketAddr::from(([127, 0, 0, 1], 0));
        let Ok(mut a) = UdpTransport::bind(localhost) else {
            eprintln!("sin UDP local, prueba omitida");
            return;
        };
        let mut b = UdpTransport::bind(localhost).unwrap();
        let b_addr = PeerAddr::Udp(b.local_addr().unwrap());
        a.send_to(b_addr, b"ping");
        // El paquete tarda un instante en llegar por loopback.
        let mut got = None;
        for _ in 0..200 {
            got = b.recv_from();
            if got.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        let (from, data) = got.expect("no llegó el paquete");
        assert_eq!(data, b"ping");
        assert_eq!(from, PeerAddr::Udp(a.local_addr().unwrap()));
        assert!(b.recv_from().is_none());
    }
}
