//! Red simulada en memoria con latencia, variación, pérdida y duplicados.
//!
//! Es determinista (semilla propia, reloj virtual): una prueba con 150 ms y
//! un 2 % de pérdida da siempre el mismo resultado.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

use crate::{PeerAddr, Socket};

/// Condiciones de cada sentido de un enlace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkConditions {
    /// Latencia de ida, en milisegundos.
    pub latency_ms: u32,
    /// Variación máxima añadida a la latencia (uniforme en `0..=jitter`).
    pub jitter_ms: u32,
    /// Paquetes perdidos por cada mil.
    pub loss_per_mille: u32,
    /// Paquetes duplicados por cada mil.
    pub duplicate_per_mille: u32,
}

impl LinkConditions {
    /// Red perfecta e instantánea (servidor local).
    pub const PERFECT: Self = Self {
        latency_ms: 0,
        jitter_ms: 0,
        loss_per_mille: 0,
        duplicate_per_mille: 0,
    };
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NetStats {
    pub sent: u64,
    pub lost: u64,
    pub duplicated: u64,
    pub delivered: u64,
    pub bytes_sent: u64,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Flight {
    deliver_at: u64,
    /// Orden de envío, para desempatar de forma determinista.
    order: u64,
    from: PeerAddr,
    data: Vec<u8>,
}

pub struct SimNetwork {
    conditions: LinkConditions,
    rng: u64,
    order: u64,
    queues: BTreeMap<PeerAddr, BinaryHeap<Reverse<Flight>>>,
    stats: NetStats,
}

impl SimNetwork {
    pub fn new(conditions: LinkConditions, seed: u64) -> Self {
        Self {
            conditions,
            rng: seed,
            order: 0,
            queues: BTreeMap::new(),
            stats: NetStats::default(),
        }
    }

    pub fn stats(&self) -> NetStats {
        self.stats
    }

    pub fn set_conditions(&mut self, conditions: LinkConditions) {
        self.conditions = conditions;
    }

    /// Socket del extremo `me` en el instante `now_ms`.
    pub fn socket(&mut self, me: PeerAddr, now_ms: u64) -> SimSocket<'_> {
        SimSocket {
            net: self,
            me,
            now_ms,
        }
    }

    fn next_random(&mut self) -> u64 {
        // SplitMix64.
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn roll(&mut self, per_mille: u32) -> bool {
        per_mille > 0 && self.next_random() % 1000 < u64::from(per_mille)
    }

    fn send(&mut self, now_ms: u64, from: PeerAddr, to: PeerAddr, data: &[u8]) {
        self.stats.sent += 1;
        self.stats.bytes_sent += data.len() as u64;
        if self.roll(self.conditions.loss_per_mille) {
            self.stats.lost += 1;
            return;
        }
        let copies = if self.roll(self.conditions.duplicate_per_mille) {
            self.stats.duplicated += 1;
            2
        } else {
            1
        };
        for _ in 0..copies {
            let jitter = match self.conditions.jitter_ms {
                0 => 0,
                j => self.next_random() % (u64::from(j) + 1),
            };
            let deliver_at = now_ms + u64::from(self.conditions.latency_ms) + jitter;
            self.order += 1;
            let flight = Flight {
                deliver_at,
                order: self.order,
                from,
                data: data.to_vec(),
            };
            self.queues.entry(to).or_default().push(Reverse(flight));
        }
    }

    fn recv(&mut self, now_ms: u64, me: PeerAddr) -> Option<(PeerAddr, Vec<u8>)> {
        let queue = self.queues.get_mut(&me)?;
        if queue
            .peek()
            .is_some_and(|Reverse(f)| f.deliver_at <= now_ms)
        {
            let Reverse(flight) = queue.pop()?;
            self.stats.delivered += 1;
            return Some((flight.from, flight.data));
        }
        None
    }
}

/// Vista de la red simulada desde un extremo, en un instante dado.
pub struct SimSocket<'a> {
    net: &'a mut SimNetwork,
    me: PeerAddr,
    now_ms: u64,
}

impl Socket for SimSocket<'_> {
    fn send_to(&mut self, to: PeerAddr, data: &[u8]) {
        self.net.send(self.now_ms, self.me, to, data);
    }

    fn recv_from(&mut self) -> Option<(PeerAddr, Vec<u8>)> {
        self.net.recv(self.now_ms, self.me)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: PeerAddr = PeerAddr::Memory(1);
    const B: PeerAddr = PeerAddr::Memory(2);

    #[test]
    fn delivers_after_latency() {
        let mut net = SimNetwork::new(
            LinkConditions {
                latency_ms: 75,
                ..LinkConditions::PERFECT
            },
            1,
        );
        net.socket(A, 0).send_to(B, b"hola");
        assert!(net.socket(B, 74).recv_from().is_none());
        assert_eq!(net.socket(B, 75).recv_from(), Some((A, b"hola".to_vec())));
        assert!(net.socket(B, 1000).recv_from().is_none());
    }

    #[test]
    fn loss_rate_is_close_to_configured_and_deterministic() {
        let run = || {
            let mut net = SimNetwork::new(
                LinkConditions {
                    latency_ms: 10,
                    jitter_ms: 20,
                    loss_per_mille: 20,
                    duplicate_per_mille: 0,
                },
                99,
            );
            for i in 0..10_000u32 {
                net.socket(A, u64::from(i)).send_to(B, &i.to_le_bytes());
            }
            let mut order = Vec::new();
            while let Some((_, data)) = net.socket(B, 1_000_000).recv_from() {
                order.push(u32::from_le_bytes([data[0], data[1], data[2], data[3]]));
            }
            (net.stats(), order)
        };
        let (stats, order) = run();
        assert!((150..250).contains(&stats.lost), "perdidos: {}", stats.lost);
        assert_eq!(order.len() as u64, stats.sent - stats.lost);
        // Con variación hay desorden.
        let sorted = order.windows(2).all(|w| w[0] <= w[1]);
        assert!(!sorted || order.is_empty());
        assert_eq!(run().1, order);
    }
}
