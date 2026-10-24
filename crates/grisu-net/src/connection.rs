//! Conexión sobre un transporte no fiable: secuencias, acuses, canal fiable
//! y ordenado (con fragmentación y reenvío) y canal no fiable.
//!
//! Formato de un paquete:
//!
//! ```text
//! magia u16 · secuencia u16 · último recibido u16 · 32 bits de acuses
//! entradas: etiqueta u8 (0 no fiable, 1 pieza fiable, 2 última pieza)
//!           [id u16 si es fiable] · longitud u16 · datos
//! ```

use std::collections::{HashMap, VecDeque};

use crate::bytes::{DecodeError, Reader, Writer};
use crate::socket::MAX_PACKET;

/// Identifica los paquetes de HUSHMINE ("HS").
pub const PROTOCOL_MAGIC: u16 = 0x4853;
const HEADER_SIZE: usize = 10;
const MAX_PAYLOAD: usize = MAX_PACKET - HEADER_SIZE;
/// Tamaño máximo de cada pieza de un mensaje fiable.
const PIECE_SIZE: usize = 1024;
/// Paquetes enviados que se recuerdan para procesar acuses.
const SENT_WINDOW: usize = 1024;
/// Piezas fiables que pueden estar en vuelo a la vez.
const RELIABLE_WINDOW: u16 = 4096;
/// Paquetes como máximo por llamada a `flush` (limita la ráfaga inicial).
const MAX_PACKETS_PER_FLUSH: usize = 24;
/// Si no se envía nada en este tiempo, se manda un paquete vacío con acuses.
const KEEPALIVE_MS: u64 = 100;

const TAG_UNRELIABLE: u8 = 0;
const TAG_PIECE: u8 = 1;
const TAG_LAST_PIECE: u8 = 2;

/// Mensaje recibido.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    pub reliable: bool,
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ConnectionStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub pieces_resent: u64,
    pub rtt_ms: f64,
}

struct Piece {
    id: u16,
    last: bool,
    data: Vec<u8>,
    sent_at: Option<u64>,
    acked: bool,
}

struct SentRecord {
    seq: u16,
    sent_at: u64,
    pieces: Vec<u16>,
}

/// ¿Es `a` más reciente que `b`, teniendo en cuenta la vuelta de los u16?
pub fn seq_newer(a: u16, b: u16) -> bool {
    a != b && a.wrapping_sub(b) < 0x8000
}

pub struct Connection {
    local_seq: u16,
    remote_seq: u16,
    received_any: bool,
    /// Bit `i`: se recibió el paquete `remote_seq - (i + 1)`.
    recv_bits: u32,
    sent: Vec<Option<SentRecord>>,
    out_reliable: VecDeque<Piece>,
    next_out_id: u16,
    out_unreliable: Vec<Vec<u8>>,
    in_pieces: HashMap<u16, (bool, Vec<u8>)>,
    next_in_id: u16,
    assembling: Vec<u8>,
    inbox: VecDeque<Message>,
    rtt_ms: Option<f64>,
    last_send_ms: Option<u64>,
    last_recv_ms: u64,
    stats: ConnectionStats,
}

impl Connection {
    pub fn new(now_ms: u64) -> Self {
        Self {
            // Empieza en 1: el primer acuse del otro extremo (0) no confirma nada.
            local_seq: 1,
            remote_seq: 0,
            received_any: false,
            recv_bits: 0,
            sent: (0..SENT_WINDOW).map(|_| None).collect(),
            out_reliable: VecDeque::new(),
            next_out_id: 0,
            out_unreliable: Vec::new(),
            in_pieces: HashMap::new(),
            next_in_id: 0,
            assembling: Vec::new(),
            inbox: VecDeque::new(),
            rtt_ms: None,
            last_send_ms: None,
            last_recv_ms: now_ms,
            stats: ConnectionStats::default(),
        }
    }

    pub fn stats(&self) -> ConnectionStats {
        ConnectionStats {
            rtt_ms: self.rtt_ms.unwrap_or(0.0),
            ..self.stats
        }
    }

    /// Instante del último paquete válido recibido.
    pub fn last_recv_ms(&self) -> u64 {
        self.last_recv_ms
    }

    /// Piezas fiables aún sin confirmar.
    pub fn pending_reliable(&self) -> usize {
        self.out_reliable.len()
    }

    /// Encola un mensaje que llegará entero y en orden.
    pub fn send_reliable(&mut self, data: &[u8]) {
        let mut chunks: Vec<&[u8]> = data.chunks(PIECE_SIZE).collect();
        if chunks.is_empty() {
            chunks.push(&[]);
        }
        let count = chunks.len();
        for (i, chunk) in chunks.into_iter().enumerate() {
            self.out_reliable.push_back(Piece {
                id: self.next_out_id,
                last: i + 1 == count,
                data: chunk.to_vec(),
                sent_at: None,
                acked: false,
            });
            self.next_out_id = self.next_out_id.wrapping_add(1);
        }
    }

    /// Encola un mensaje que puede perderse. Se descarta si no cabe en un
    /// paquete.
    pub fn send_unreliable(&mut self, data: Vec<u8>) {
        if data.len() + 3 > MAX_PAYLOAD {
            log::warn!("mensaje no fiable de {} bytes descartado", data.len());
            return;
        }
        self.out_unreliable.push(data);
    }

    /// Mensajes recibidos desde la última llamada.
    pub fn drain_messages(&mut self) -> impl Iterator<Item = Message> + '_ {
        self.inbox.drain(..)
    }

    fn resend_ms(&self) -> u64 {
        let rtt = self.rtt_ms.unwrap_or(150.0);
        (rtt * 1.5 + 10.0).clamp(40.0, 1000.0) as u64
    }

    /// Construye los paquetes que hay que enviar ahora.
    pub fn flush(&mut self, now_ms: u64) -> Vec<Vec<u8>> {
        let resend = self.resend_ms();
        let window = self.out_reliable.len().min(usize::from(RELIABLE_WINDOW));
        let mut unreliable = std::mem::take(&mut self.out_unreliable)
            .into_iter()
            .peekable();
        let mut index = 0;
        let mut packets = Vec::new();

        while packets.len() < MAX_PACKETS_PER_FLUSH {
            let mut body = Writer::new();
            let mut pieces = Vec::new();
            while index < window {
                let piece = &mut self.out_reliable[index];
                let due = !piece.acked && piece.sent_at.is_none_or(|t| now_ms >= t + resend);
                if !due {
                    index += 1;
                    continue;
                }
                if body.len() + 5 + piece.data.len() > MAX_PAYLOAD {
                    break;
                }
                if piece.sent_at.is_some() {
                    self.stats.pieces_resent += 1;
                }
                let tag = if piece.last {
                    TAG_LAST_PIECE
                } else {
                    TAG_PIECE
                };
                body.u8(tag).u16(piece.id).u16(piece.data.len() as u16);
                body.raw(&piece.data);
                piece.sent_at = Some(now_ms);
                pieces.push(piece.id);
                index += 1;
            }
            while let Some(msg) = unreliable.peek() {
                if body.len() + 3 + msg.len() > MAX_PAYLOAD {
                    break;
                }
                body.u8(TAG_UNRELIABLE).u16(msg.len() as u16).raw(msg);
                unreliable.next();
            }
            let keepalive =
                packets.is_empty() && self.last_send_ms.is_none_or(|t| now_ms >= t + KEEPALIVE_MS);
            if body.is_empty() && !keepalive {
                break;
            }
            packets.push(self.wrap(now_ms, body, pieces));
        }
        if unreliable.next().is_some() {
            log::debug!("mensajes no fiables descartados por falta de espacio");
        }
        packets
    }

    fn wrap(&mut self, now_ms: u64, body: Writer, pieces: Vec<u16>) -> Vec<u8> {
        let seq = self.local_seq;
        self.local_seq = self.local_seq.wrapping_add(1);
        let mut packet = Writer::new();
        packet
            .u16(PROTOCOL_MAGIC)
            .u16(seq)
            .u16(self.remote_seq)
            .u32(self.recv_bits)
            .raw(&body.into_bytes());
        self.sent[usize::from(seq) % SENT_WINDOW] = Some(SentRecord {
            seq,
            sent_at: now_ms,
            pieces,
        });
        self.last_send_ms = Some(now_ms);
        let bytes = packet.into_bytes();
        self.stats.packets_sent += 1;
        self.stats.bytes_sent += bytes.len() as u64;
        bytes
    }

    /// Procesa un paquete recibido. Un paquete mal formado devuelve error,
    /// pero los acuses ya leídos se aplican igualmente.
    pub fn receive(&mut self, now_ms: u64, packet: &[u8]) -> Result<(), DecodeError> {
        let mut r = Reader::new(packet);
        if r.u16()? != PROTOCOL_MAGIC {
            return Err(DecodeError::Invalid("magia"));
        }
        let seq = r.u16()?;
        let ack = r.u16()?;
        let ack_bits = r.u32()?;
        self.last_recv_ms = now_ms;
        self.stats.packets_received += 1;
        let fresh = self.track_received(seq);
        self.process_acks(now_ms, ack, ack_bits);
        if !fresh {
            return Ok(());
        }
        while !r.is_empty() {
            match r.u8()? {
                TAG_UNRELIABLE => {
                    let len = usize::from(r.u16()?);
                    let data = r.raw(len)?.to_vec();
                    self.inbox.push_back(Message {
                        reliable: false,
                        data,
                    });
                }
                tag @ (TAG_PIECE | TAG_LAST_PIECE) => {
                    let id = r.u16()?;
                    let len = usize::from(r.u16()?);
                    let data = r.raw(len)?;
                    let offset = id.wrapping_sub(self.next_in_id);
                    if offset < RELIABLE_WINDOW {
                        self.in_pieces
                            .entry(id)
                            .or_insert_with(|| (tag == TAG_LAST_PIECE, data.to_vec()));
                    }
                }
                _ => return Err(DecodeError::Invalid("etiqueta de entrada")),
            }
        }
        self.assemble();
        Ok(())
    }

    /// Anota el paquete `seq` como recibido. Devuelve `false` si es un
    /// duplicado o demasiado antiguo para saberlo.
    fn track_received(&mut self, seq: u16) -> bool {
        if !self.received_any {
            self.received_any = true;
            self.remote_seq = seq;
            self.recv_bits = 0;
            return true;
        }
        if seq_newer(seq, self.remote_seq) {
            let shift = u64::from(seq.wrapping_sub(self.remote_seq));
            let bits = if shift > 32 {
                0
            } else {
                ((u64::from(self.recv_bits) << shift) | (1 << (shift - 1))) as u32
            };
            self.recv_bits = bits;
            self.remote_seq = seq;
            return true;
        }
        let offset = self.remote_seq.wrapping_sub(seq);
        if offset == 0 || offset > 32 {
            return false;
        }
        let bit = 1u32 << (offset - 1);
        let seen = self.recv_bits & bit != 0;
        self.recv_bits |= bit;
        !seen
    }

    fn process_acks(&mut self, now_ms: u64, ack: u16, bits: u32) {
        for i in 0..=32u16 {
            if i > 0 && bits & (1 << (i - 1)) == 0 {
                continue;
            }
            let seq = ack.wrapping_sub(i);
            let slot = &mut self.sent[usize::from(seq) % SENT_WINDOW];
            let Some(record) = slot.take_if(|r| r.seq == seq) else {
                continue;
            };
            let sample = now_ms.saturating_sub(record.sent_at) as f64;
            self.rtt_ms = Some(match self.rtt_ms {
                None => sample,
                Some(rtt) => rtt + (sample - rtt) * 0.1,
            });
            if let Some(front) = self.out_reliable.front().map(|p| p.id) {
                for id in record.pieces {
                    let index = usize::from(id.wrapping_sub(front));
                    if let Some(piece) = self.out_reliable.get_mut(index) {
                        piece.acked = true;
                    }
                }
            }
        }
        while self.out_reliable.front().is_some_and(|p| p.acked) {
            self.out_reliable.pop_front();
        }
    }

    fn assemble(&mut self) {
        while let Some((last, data)) = self.in_pieces.remove(&self.next_in_id) {
            self.assembling.extend_from_slice(&data);
            self.next_in_id = self.next_in_id.wrapping_add(1);
            if last {
                self.inbox.push_back(Message {
                    reliable: true,
                    data: std::mem::take(&mut self.assembling),
                });
            }
        }
    }
}

#[cfg(test)]
#[path = "connection_tests.rs"]
mod tests;
