//! Mensajes entre cliente y servidor y su codificación binaria.
//!
//! Decodificar nunca hace `panic!`: un mensaje mal formado es un error que
//! el servidor registra y descarta.

use grisu_net::bytes::{DecodeError, Reader, Writer};
use hush_sim::{BlockChange, BlockKind, MoveInput, PlayerBody};

/// Se incrementa con cada cambio incompatible del protocolo.
pub const PROTOCOL_VERSION: u16 = 1;
/// Longitud máxima del nombre de un jugador, en bytes.
pub const MAX_NAME_LEN: usize = 32;
/// Entradas que repite cada paquete del cliente, por si se pierden algunos.
pub const INPUT_REDUNDANCY: usize = 12;

/// Acción de un tick, además del movimiento.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Dig([i32; 3]),
    Place([i32; 3], BlockKind),
}

/// Entrada numerada de un tick del cliente.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputFrame {
    /// Empieza en 1 y crece de uno en uno.
    pub seq: u32,
    pub movement: MoveInput,
    pub action: Option<Action>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientMsg {
    Hello {
        version: u16,
        name: String,
    },
    /// Últimas entradas, de la más antigua a la más reciente.
    Inputs(Vec<InputFrame>),
    Bye,
}

/// Posición de otro jugador en una instantánea.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemotePlayer {
    pub id: u16,
    pub feet: [i32; 3],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServerMsg {
    Welcome {
        player: u16,
        map_size: [u32; 3],
        tick: u32,
    },
    Chunk {
        index: u16,
        data: Vec<u8>,
    },
    /// Fin del mapa inicial: a partir de aquí solo llegan cambios.
    MapDone,
    Changes(Vec<BlockChange>),
    Snapshot {
        tick: u32,
        /// Última entrada del cliente que el servidor ya aplicó.
        last_input: u32,
        you: PlayerBody,
        others: Vec<RemotePlayer>,
    },
    Refused(String),
}

const C_HELLO: u8 = 1;
const C_INPUTS: u8 = 2;
const C_BYE: u8 = 3;

const S_WELCOME: u8 = 1;
const S_CHUNK: u8 = 2;
const S_MAP_DONE: u8 = 3;
const S_CHANGES: u8 = 4;
const S_SNAPSHOT: u8 = 5;
const S_REFUSED: u8 = 6;

const A_NONE: u8 = 0;
const A_DIG: u8 = 1;
const A_PLACE: u8 = 2;

fn put_cell(w: &mut Writer, c: [i32; 3]) {
    w.i32(c[0]).i32(c[1]).i32(c[2]);
}

fn get_cell(r: &mut Reader) -> Result<[i32; 3], DecodeError> {
    Ok([r.i32()?, r.i32()?, r.i32()?])
}

fn put_text(w: &mut Writer, text: &str) {
    w.bytes(text.as_bytes());
}

fn get_text(r: &mut Reader, max: usize) -> Result<String, DecodeError> {
    let bytes = r.bytes()?;
    if bytes.len() > max {
        return Err(DecodeError::Invalid("texto demasiado largo"));
    }
    String::from_utf8(bytes.to_vec()).map_err(|_| DecodeError::Invalid("texto no UTF-8"))
}

fn put_frame(w: &mut Writer, f: &InputFrame) {
    w.u32(f.seq)
        .i8(f.movement.wish_x)
        .i8(f.movement.wish_z)
        .bool(f.movement.jump);
    match f.action {
        None => {
            w.u8(A_NONE);
        }
        Some(Action::Dig(cell)) => {
            w.u8(A_DIG);
            put_cell(w, cell);
        }
        Some(Action::Place(cell, block)) => {
            w.u8(A_PLACE);
            put_cell(w, cell);
            w.u8(block.id());
        }
    }
}

fn get_frame(r: &mut Reader) -> Result<InputFrame, DecodeError> {
    let seq = r.u32()?;
    let movement = MoveInput {
        wish_x: r.i8()?,
        wish_z: r.i8()?,
        jump: r.bool()?,
    };
    let action = match r.u8()? {
        A_NONE => None,
        A_DIG => Some(Action::Dig(get_cell(r)?)),
        A_PLACE => {
            let cell = get_cell(r)?;
            let block = BlockKind::from_id(r.u8()?).map_err(|_| DecodeError::Invalid("bloque"))?;
            Some(Action::Place(cell, block))
        }
        _ => return Err(DecodeError::Invalid("acción")),
    };
    Ok(InputFrame {
        seq,
        movement,
        action,
    })
}

fn put_body(w: &mut Writer, b: &PlayerBody) {
    put_cell(w, b.feet);
    put_cell(w, b.velocity);
    w.bool(b.on_ground);
}

fn get_body(r: &mut Reader) -> Result<PlayerBody, DecodeError> {
    Ok(PlayerBody {
        feet: get_cell(r)?,
        velocity: get_cell(r)?,
        on_ground: r.bool()?,
    })
}

impl ClientMsg {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        match self {
            Self::Hello { version, name } => {
                w.u8(C_HELLO).u16(*version);
                put_text(&mut w, name);
            }
            Self::Inputs(frames) => {
                w.u8(C_INPUTS).u8(frames.len() as u8);
                frames.iter().for_each(|f| put_frame(&mut w, f));
            }
            Self::Bye => {
                w.u8(C_BYE);
            }
        }
        w.into_bytes()
    }

    pub fn decode(data: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(data);
        let msg = match r.u8()? {
            C_HELLO => Self::Hello {
                version: r.u16()?,
                name: get_text(&mut r, MAX_NAME_LEN)?,
            },
            C_INPUTS => {
                let count = usize::from(r.u8()?);
                if count > INPUT_REDUNDANCY * 2 {
                    return Err(DecodeError::Invalid("demasiadas entradas"));
                }
                Self::Inputs(
                    (0..count)
                        .map(|_| get_frame(&mut r))
                        .collect::<Result<_, _>>()?,
                )
            }
            C_BYE => Self::Bye,
            _ => return Err(DecodeError::Invalid("mensaje de cliente")),
        };
        r.finish()?;
        Ok(msg)
    }
}

impl ServerMsg {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        match self {
            Self::Welcome {
                player,
                map_size,
                tick,
            } => {
                w.u8(S_WELCOME).u16(*player);
                map_size.iter().for_each(|&s| {
                    w.u32(s);
                });
                w.u32(*tick);
            }
            Self::Chunk { index, data } => {
                w.u8(S_CHUNK).u16(*index).bytes(data);
            }
            Self::MapDone => {
                w.u8(S_MAP_DONE);
            }
            Self::Changes(changes) => {
                w.u8(S_CHANGES).u32(changes.len() as u32);
                for c in changes {
                    put_cell(&mut w, c.cell);
                    w.u8(c.block);
                }
            }
            Self::Snapshot {
                tick,
                last_input,
                you,
                others,
            } => {
                w.u8(S_SNAPSHOT).u32(*tick).u32(*last_input);
                put_body(&mut w, you);
                w.u16(others.len() as u16);
                for o in others {
                    w.u16(o.id);
                    put_cell(&mut w, o.feet);
                }
            }
            Self::Refused(reason) => {
                w.u8(S_REFUSED);
                put_text(&mut w, reason);
            }
        }
        w.into_bytes()
    }

    pub fn decode(data: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(data);
        let msg = match r.u8()? {
            S_WELCOME => Self::Welcome {
                player: r.u16()?,
                map_size: [r.u32()?, r.u32()?, r.u32()?],
                tick: r.u32()?,
            },
            S_CHUNK => Self::Chunk {
                index: r.u16()?,
                data: r.bytes()?.to_vec(),
            },
            S_MAP_DONE => Self::MapDone,
            S_CHANGES => {
                let count = r.u32()? as usize;
                // Cada cambio ocupa 13 bytes: se valida antes de reservar.
                if count.saturating_mul(13) > r.remaining() {
                    return Err(DecodeError::UnexpectedEnd);
                }
                let mut changes = Vec::with_capacity(count);
                for _ in 0..count {
                    changes.push(BlockChange {
                        cell: get_cell(&mut r)?,
                        block: r.u8()?,
                    });
                }
                Self::Changes(changes)
            }
            S_SNAPSHOT => {
                let tick = r.u32()?;
                let last_input = r.u32()?;
                let you = get_body(&mut r)?;
                let count = usize::from(r.u16()?);
                if count.saturating_mul(14) > r.remaining() {
                    return Err(DecodeError::UnexpectedEnd);
                }
                let mut others = Vec::with_capacity(count);
                for _ in 0..count {
                    others.push(RemotePlayer {
                        id: r.u16()?,
                        feet: get_cell(&mut r)?,
                    });
                }
                Self::Snapshot {
                    tick,
                    last_input,
                    you,
                    others,
                }
            }
            S_REFUSED => Self::Refused(get_text(&mut r, 256)?),
            _ => return Err(DecodeError::Invalid("mensaje de servidor")),
        };
        r.finish()?;
        Ok(msg)
    }
}

#[cfg(test)]
#[path = "proto_tests.rs"]
mod tests;
