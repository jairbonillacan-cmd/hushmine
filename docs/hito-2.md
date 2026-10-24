# Hito 2: Red

**Condición de salida (PLAN_v4):** dos clientes y seis bots consistentes con 150 ms de latencia y 2 % de pérdida.

## Estado

| Pieza | Dónde | Estado |
|---|---|---|
| Transporte UDP y red simulada (latencia, variación, pérdida, duplicados) | `grisu-net/src/socket.rs`, `sim.rs` | Hecho |
| Conexión: secuencias, acuses, canal fiable y ordenado con fragmentación, canal no fiable | `grisu-net/src/connection.rs` | Hecho |
| Protocolo binario (nunca `panic!` con datos externos) | `hush-net/src/proto.rs` | Hecho |
| Servidor autoritativo a 30 Hz, unión con mapa completo (RLE), deltas, correcciones, desconexión por tiempo | `hush-net/src/server.rs` | Hecho |
| Cliente con predicción y reconciliación, réplica del mundo | `hush-net/src/client.rs` | Hecho |
| Interpolación de los demás jugadores (100 ms) | `hush-net/src/interp.rs` | Hecho |
| Bots (cavan desde el servidor) | `hush-sim/src/bot.rs` | Hecho (básicos) |
| Servidor dedicado | `hush-server` | Hecho |
| Juego en red: local en memoria o `--connect` | `hush-client/src/session.rs`, `game.rs` | Hecho; probado por UDP local, no en LAN real |
| Otros jugadores visibles (cajas de cuadrilla) | `grisu-render/src/boxes.rs` | Hecho |
| Prueba de consistencia y `hush-cli netsim` | `hush-net/tests/consistency.rs`, `src/harness.rs` | Hecho |

ADR: `docs/adr/0003-red-propia.md` (capa de red propia en lugar de `renet2`, por ahora).

## Medidas (`netsim`, 60 s, 2 clientes, 6 bots, 150 ms de ida y vuelta, 2 % de pérdida)

| Medida | Valor |
|---|---|
| Consistencia final (mundo y posición) | Sí, en 8 semillas |
| Bajada por cliente | ~5,5 KB/s (presupuesto: 32 KB/s) |
| Subida por cliente | ~2,6 KB/s |
| Correcciones de predicción | ~10-15 % de las instantáneas; error máximo 1-2 bloques |
| Prueba dura: 4 clientes, 300 ms, 10 % de pérdida | Consistente, ~6,6 KB/s de bajada |
| Cliente en release, local con 4 bots (RX 9060 XT) | ~2000 FPS |

## Pendiente o mejorable

- **Probar en LAN real** con dos máquinas y comprobar la sensación con latencia (las correcciones se aplican de golpe; se puede suavizar el error visual).
- El servidor repite la última entrada si no ha llegado la siguiente; eso adelanta al jugador un tick y causa parte de las correcciones. Un búfer de entradas adaptativo lo reduciría.
- La luz se recalcula entera con cada cambio de bloques; con muchos bots en modo depuración baja los FPS. Pasar a luz incremental.
- Los bots siguen siendo los de prueba (andan y cavan al azar). Bots que jueguen de verdad llegan con las reglas del hito 3.
- Sin cifrado ni autenticación: el servidor acepta a cualquiera hasta el máximo de clientes.
- WebTransport y relé de Steam: hito 5, como nuevas implementaciones de `Socket`.
