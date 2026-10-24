//! Red de HUSHMINE: protocolo, servidor autoritativo y cliente con
//! predicción. Todo es lógica pura sobre un [`grisu_net::Socket`], así que
//! se prueba con la red simulada y se usa igual con UDP.

mod client;
pub mod harness;
mod interp;
pub mod proto;
mod server;

pub use client::{ClientState, NetClient, PredictionStats};
pub use proto::{Action, PROTOCOL_VERSION};
pub use server::{Server, ServerConfig};

/// Puerto UDP por defecto del servidor.
pub const DEFAULT_PORT: u16 = 47_400;
