//! Unión por UDP real (loopback) con reloj real: el cliente recibe el mapa,
//! cava y el servidor lo confirma.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use grisu_net::{PeerAddr, UdpTransport};
use hush_net::{Action, ClientState, NetClient, Server, ServerConfig};
use hush_sim::MoveInput;

#[test]
fn client_joins_and_digs_over_udp_loopback() {
    let localhost = SocketAddr::from(([127, 0, 0, 1], 0));
    let Ok(mut server_socket) = UdpTransport::bind(localhost) else {
        eprintln!("sin UDP local, prueba omitida");
        return;
    };
    let server_addr = PeerAddr::Udp(server_socket.local_addr().unwrap());
    let mut client_socket = UdpTransport::bind(localhost).unwrap();
    let mut server = Server::new(ServerConfig::default()).unwrap();
    let mut client = NetClient::new(server_addr, "udp", 0);

    let start = Instant::now();
    let mut ticks = 0u64;
    let mut dug = None;
    while start.elapsed() < Duration::from_secs(10) {
        let now = start.elapsed().as_millis() as u64;
        server.update(now, &mut server_socket);
        client.poll(now, &mut client_socket);
        if ticks < now * 30 / 1000 {
            ticks += 1;
            let mut action = None;
            // Tras un segundo de juego, cava el bloque bajo los pies.
            if *client.state() == ClientState::Playing && ticks > 60 && dug.is_none() {
                let feet = client.body().feet.map(grisu_voxel::cell_of);
                let cell = [feet[0], feet[1] - 1, feet[2]];
                action = Some(Action::Dig(cell));
                dug = Some(cell);
            }
            client.tick(MoveInput::default(), action);
        }
        client.flush(now, &mut client_socket);
        if let Some([x, y, z]) = dug
            && !server.sim().world().is_solid(x, y, z)
            && client
                .world()
                .is_some_and(|w| w.content_hash() == server.sim().world().content_hash())
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    panic!(
        "no se completó: estado {:?}, cavado {:?}",
        client.state(),
        dug
    );
}
