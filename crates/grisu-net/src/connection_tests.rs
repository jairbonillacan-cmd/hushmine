use super::*;
use crate::{LinkConditions, PeerAddr, SimNetwork, Socket};

const A: PeerAddr = PeerAddr::Memory(1);
const B: PeerAddr = PeerAddr::Memory(2);

/// Dos conexiones unidas por una red simulada. `step` avanza 10 ms.
struct Pair {
    net: SimNetwork,
    a: Connection,
    b: Connection,
    now: u64,
}

impl Pair {
    fn new(conditions: LinkConditions, seed: u64) -> Self {
        Self {
            net: SimNetwork::new(conditions, seed),
            a: Connection::new(0),
            b: Connection::new(0),
            now: 0,
        }
    }

    fn step(&mut self) -> (Vec<Message>, Vec<Message>) {
        self.now += 10;
        let now = self.now;
        for packet in self.a.flush(now) {
            self.net.socket(A, now).send_to(B, &packet);
        }
        for packet in self.b.flush(now) {
            self.net.socket(B, now).send_to(A, &packet);
        }
        while let Some((_, packet)) = self.net.socket(B, now).recv_from() {
            self.b.receive(now, &packet).unwrap();
        }
        while let Some((_, packet)) = self.net.socket(A, now).recv_from() {
            self.a.receive(now, &packet).unwrap();
        }
        (
            self.a.drain_messages().collect(),
            self.b.drain_messages().collect(),
        )
    }
}

fn bad_network() -> LinkConditions {
    LinkConditions {
        latency_ms: 75,
        jitter_ms: 30,
        loss_per_mille: 100,
        duplicate_per_mille: 20,
    }
}

#[test]
fn reliable_messages_arrive_once_and_in_order_over_a_bad_network() {
    let mut pair = Pair::new(bad_network(), 7);
    let big: Vec<u8> = (0..50_000u32).map(|i| (i % 251) as u8).collect();
    let mut expected = Vec::new();
    for i in 0..200u32 {
        let msg = if i == 50 {
            big.clone()
        } else {
            i.to_le_bytes().to_vec()
        };
        pair.a.send_reliable(&msg);
        expected.push(msg);
    }
    let mut got = Vec::new();
    for _ in 0..1000 {
        let (_, at_b) = pair.step();
        got.extend(at_b.into_iter().map(|m| {
            assert!(m.reliable);
            m.data
        }));
        if got.len() == expected.len() && pair.a.pending_reliable() == 0 {
            break;
        }
    }
    assert_eq!(got, expected);
    assert_eq!(
        pair.a.pending_reliable(),
        0,
        "quedaron piezas sin confirmar"
    );
    assert!(
        pair.a.stats().pieces_resent > 0,
        "con pérdida debería reenviar"
    );
}

#[test]
fn unreliable_messages_mostly_arrive_and_never_repeat() {
    let mut pair = Pair::new(bad_network(), 3);
    let mut got = Vec::new();
    for i in 0..500u32 {
        pair.a.send_unreliable(i.to_le_bytes().to_vec());
        let (_, at_b) = pair.step();
        got.extend(at_b);
    }
    for _ in 0..50 {
        got.extend(pair.step().1);
    }
    let mut ids: Vec<u32> = got
        .iter()
        .map(|m| u32::from_le_bytes(m.data[..4].try_into().unwrap()))
        .collect();
    let received = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), received, "llegaron duplicados");
    assert!((400..500).contains(&received), "llegaron {received} de 500");
}

#[test]
fn rtt_estimate_matches_round_trip() {
    let mut pair = Pair::new(
        LinkConditions {
            latency_ms: 75,
            ..LinkConditions::PERFECT
        },
        1,
    );
    for _ in 0..300 {
        pair.a.send_unreliable(vec![1]);
        pair.b.send_unreliable(vec![2]);
        pair.step();
    }
    let rtt = pair.a.stats().rtt_ms;
    // 150 ms de ida y vuelta más hasta un paso de espera en cada lado.
    assert!((150.0..=175.0).contains(&rtt), "rtt {rtt}");
}

#[test]
fn survives_sequence_wrap_around() {
    let mut pair = Pair::new(LinkConditions::PERFECT, 1);
    let mut got = 0;
    for i in 0..70_000u32 {
        if i % 100 == 0 {
            pair.a.send_reliable(&i.to_le_bytes());
        }
        pair.a.send_unreliable(vec![0]);
        got += pair.step().1.iter().filter(|m| m.reliable).count();
    }
    assert_eq!(got, 700);
}

#[test]
fn garbage_is_rejected_without_panicking() {
    let mut conn = Connection::new(0);
    assert!(conn.receive(0, &[]).is_err());
    assert!(
        conn.receive(0, &[0x12, 0x34, 0, 0, 0, 0, 0, 0, 0, 0])
            .is_err()
    );
    let mut header = Writer::new();
    header.u16(PROTOCOL_MAGIC).u16(5).u16(0).u32(0);
    let mut bad = header.into_bytes();
    bad.extend_from_slice(&[9, 1, 2]);
    assert!(conn.receive(0, &bad).is_err());
    // Otro número de secuencia: un duplicado se descartaría sin leerlo.
    let mut truncated = Writer::new();
    truncated.u16(PROTOCOL_MAGIC).u16(6).u16(0).u32(0);
    let mut truncated = truncated.into_bytes();
    truncated.extend_from_slice(&[TAG_PIECE, 0, 0, 200, 0, 1]);
    assert!(conn.receive(0, &truncated).is_err());
    // Bytes pseudoaleatorios con la magia correcta.
    let mut seed: u32 = 1;
    for len in 0..300 {
        let mut packet = PROTOCOL_MAGIC.to_le_bytes().to_vec();
        for _ in 0..len {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
            packet.push((seed >> 16) as u8);
        }
        let _ = conn.receive(0, &packet);
    }
}

#[test]
fn sequence_comparison_wraps() {
    assert!(seq_newer(1, 0));
    assert!(seq_newer(0, 65535));
    assert!(!seq_newer(65535, 0));
    assert!(!seq_newer(7, 7));
}
