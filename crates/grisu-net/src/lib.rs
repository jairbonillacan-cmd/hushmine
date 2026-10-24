//! Grisú: red.
//!
//! Transportes de paquetes ([`Socket`]: UDP o una red simulada en memoria)
//! y [`Connection`], que añade sobre ellos un canal fiable y ordenado y uno
//! no fiable. Nada lee el reloj del sistema: todo recibe `now_ms`, así que
//! las pruebas con latencia y pérdida son deterministas.

pub mod bytes;
mod connection;
mod sim;
mod socket;

pub use connection::{Connection, ConnectionStats, Message, PROTOCOL_MAGIC, seq_newer};
pub use sim::{LinkConditions, NetStats, SimNetwork, SimSocket};
pub use socket::{MAX_PACKET, PeerAddr, Socket, UdpTransport};
