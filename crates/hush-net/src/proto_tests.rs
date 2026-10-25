use super::*;

fn frame(seq: u32, action: Option<Action>) -> InputFrame {
    InputFrame {
        seq,
        movement: MoveInput {
            wish_x: -127,
            wish_z: 64,
            jump: true,
        },
        action,
    }
}

#[test]
fn client_messages_round_trip() {
    let msgs = [
        ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            name: "Minera ñ".into(),
        },
        ClientMsg::Inputs(vec![
            frame(1, None),
            frame(2, Some(Action::Dig([1, -2, 3]))),
            frame(3, Some(Action::Place([4, 5, 6], BlockKind::Gold))),
        ]),
        ClientMsg::Bye,
    ];
    for msg in msgs {
        assert_eq!(ClientMsg::decode(&msg.encode()), Ok(msg));
    }
}

#[test]
fn server_messages_round_trip() {
    let body = PlayerBody {
        feet: [1024, 2048, -5],
        velocity: [0, -31, 7],
        on_ground: true,
    };
    let msgs = [
        ServerMsg::Welcome {
            player: 3,
            map_size: [64, 64, 64],
            tick: 99,
        },
        ServerMsg::Chunk {
            index: 7,
            data: vec![1, 2, 3],
        },
        ServerMsg::MapDone,
        ServerMsg::Changes(vec![BlockChange {
            cell: [1, 2, 3],
            block: 4,
        }]),
        ServerMsg::Snapshot {
            tick: 5,
            last_input: 4,
            you: body,
            others: vec![RemotePlayer {
                id: 2,
                feet: [9, 8, 7],
            }],
        },
        ServerMsg::Refused("lleno".into()),
    ];
    for msg in msgs {
        assert_eq!(ServerMsg::decode(&msg.encode()), Ok(msg));
    }
}

#[test]
fn malformed_messages_are_errors() {
    assert!(ClientMsg::decode(&[]).is_err());
    assert!(ClientMsg::decode(&[99]).is_err());
    assert!(ClientMsg::decode(&[C_INPUTS, 200]).is_err());
    assert!(ClientMsg::decode(&[C_BYE, 0]).is_err());
    let long_name = ClientMsg::Hello {
        version: 1,
        name: "x".repeat(MAX_NAME_LEN + 1),
    };
    assert!(ClientMsg::decode(&long_name.encode()).is_err());
    assert!(ServerMsg::decode(&[S_CHANGES, 255, 255, 255, 255]).is_err());
    // Todos los prefijos de un mensaje válido fallan sin `panic!`.
    let snapshot = ServerMsg::Snapshot {
        tick: 1,
        last_input: 1,
        you: PlayerBody::new([0, 0, 0]),
        others: vec![RemotePlayer {
            id: 1,
            feet: [0; 3],
        }],
    }
    .encode();
    for len in 0..snapshot.len() {
        assert!(ServerMsg::decode(&snapshot[..len]).is_err());
    }
}

#[test]
fn fullest_snapshot_fits_in_one_unreliable_message() {
    let remote = |id: u16| RemotePlayer {
        id,
        feet: [i64::MAX, i64::MIN, -1],
    };
    let snapshot = |n: usize| ServerMsg::Snapshot {
        tick: u32::MAX,
        last_input: u32::MAX,
        you: PlayerBody::new([i64::MIN; 3]),
        others: (0..n as u16).map(remote).collect(),
    };
    let full = snapshot(MAX_REMOTES_PER_SNAPSHOT).encode();
    assert_eq!(
        full.len(),
        SNAPSHOT_FIXED_BYTES + MAX_REMOTES_PER_SNAPSHOT * SNAPSHOT_REMOTE_BYTES,
        "las constantes de tamaño no coinciden con la codificación"
    );
    assert!(full.len() <= grisu_net::MAX_UNRELIABLE);
    assert!(snapshot(MAX_REMOTES_PER_SNAPSHOT + 1).encode().len() > grisu_net::MAX_UNRELIABLE);
}
