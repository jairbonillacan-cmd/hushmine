# Hito 1: Vóxel

**Condición de salida (PLAN_v4):** escenas de captura de túneles correctas; 64³ se regenera completo en menos de 50 ms.

## Estado

| Pieza | Dónde | Estado |
|---|---|---|
| Chunks de 16³ y edición con chunks sucios | `grisu-voxel/src/chunk.rs`, `world.rs` | Hecho |
| Mallas con oclusión ambiental | `grisu-voxel/src/mesh.rs` | Hecho |
| Luz de cielo y de lava (BFS) | `grisu-voxel/src/light.rs` | Hecho |
| Raycast DDA | `grisu-voxel/src/raycast.rs` | Hecho |
| Colisión en punto fijo | `grisu-voxel/src/collide.rs` | Hecho |
| Bloques y generación del monte Anima | `hush-sim/src/block.rs`, `terrain.rs` | Hecho |
| Movimiento del minero (andar, saltar, gravedad) | `hush-sim/src/player.rs` | Hecho |
| Cavar y poner con validación | `hush-sim/src/edit.rs` | Hecho |
| Bot de prueba que cava | `hush-sim/src/bot.rs` | Hecho |
| Renderizado de chunks, lámpara del casco, niebla, resaltado | `grisu-render/src/voxel.rs`, `voxel.wgsl` | Hecho |
| Cámara en primera persona y cruz de apuntar | `grisu-render/src/camera.rs`, `crosshair.rs` | Hecho |
| Entrada de teclado y ratón | `grisu-platform/src/input.rs` | Hecho |
| Cliente jugable (un jugador, local) | `hush-client/src/game.rs` | Hecho, sin probar a mano |
| Escenas de captura | `hush-cli shot --scene terrain/tunnel/shaft/dig/players/frontier` | Hecho |
| Banco de pruebas | `hush-cli bench` | Hecho |

## W0 (mundo grande), dentro del hito 1

**Hecho** (después del hito 1 y del 2; decisión y criterio de salida en
`docs/adr/0002-mundo-grande.md` y `docs/mundo/04-verificacion-e-hitos.md`):

| Pieza | Dónde | Estado |
|---|---|---|
| Interfaz `Voxels`/`VoxelsMut`/`ChunkedVoxels` | `grisu-voxel/src/access.rs` | Hecho |
| `ChunkPos` con coordenadas negativas (desplazamiento aritmético) | `grisu-voxel/src/pos.rs` | Hecho |
| `SparseWorld`: chunks bajo demanda, borde en ±7 500 000 | `grisu-voxel/src/sparse.rs` | Hecho |
| Posiciones de entidad `i64` Q47.16 (`FIXED_ONE` = 65536) | `grisu-voxel/src/collide.rs`, `hush-sim/src/player.rs` | Hecho |
| Raycast en `f64`, genérico sobre `Voxels` | `grisu-voxel/src/raycast.rs` | Hecho |
| Mallas locales al chunk (4 bits por eje) y genéricas | `grisu-voxel/src/mesh.rs` | Hecho |
| Render relativo a la cámara (anchor + origen por chunk) | `grisu-render/src/camera.rs`, `voxel.rs`, `voxel.wgsl` | Hecho |
| Protocolo con posiciones de 64 bits (versión 2); interpolación y cliente en `f64` | `grisu-net/src/bytes.rs`, `hush-net/src/proto.rs`, `interp.rs` | Hecho |
| Pruebas en coordenadas negativas y en ±7 499 999 | `grisu-voxel`, `hush-net/tests/consistency.rs` | Hecho |
| Escena `frontier`: estructura reconocible a 7,5 M bloques del origen | `hush-cli/src/scenes.rs` | Hecho |

Las capturas de `cube`, `terrain`, `tunnel`, `shaft` y `players` salen
píxel a píxel iguales (o casi: bordes de interpolación) que en `main`; la
de `dig` cambia porque el recorrido del bot depende de la precisión de la
física, que pasó de Q22.10 a Q47.16.

## Medidas (release, seed 42, 64³, RX 9060 XT, tras W0)

| Medida | Valor |
|---|---|
| Generar terreno | 4,0 ms |
| Mallas completas (sin luz) | 5,3 ms |
| Luz completa | 1,8 ms |
| Mallas completas con luz | 4,9 ms |
| Cavar un bloque en esquina de chunk (8 chunks) | 0,25 ms |
| Cliente, RX 9060 XT, DX12 | ~1250 FPS |

**Condición de salida cumplida:** luz + mallas de 64³ en unos 7 ms (límite 50).

## Pendiente o mejorable

- **Probar el cliente a mano:** ratón, cavar, poner, saltar. Las pruebas automáticas cubren la simulación y las capturas, pero no la sensación de control.
- El tiempo de pico por bloque (`BlockKind::dig_ticks`) existe pero no se aplica: hoy se cava con un clic. Llegará con las clases (hito 3).
- La lava no fluye ni hace daño (hito 4).
- Luz suavizada por vértice y HUD con texto (glyphon) para el bloque elegido y los FPS.
- Escenas de captura en archivos RON (`tests/scenes/*.ron`), como propone el plan, en lugar de escenas en código.
