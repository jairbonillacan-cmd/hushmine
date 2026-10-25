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
| Escenas de captura | `hush-cli shot --scene terrain/tunnel/shaft/dig` | Hecho |
| Banco de pruebas | `hush-cli bench` | Hecho |

## Medidas (release, seed 42, 64³)

| Medida | Valor |
|---|---|
| Generar terreno | 3,7 ms |
| Mallas completas (sin luz) | 3,8 ms |
| Luz completa | 1,5 ms |
| Mallas completas con luz | 3,8 ms |
| Cavar un bloque en esquina de chunk (8 chunks) | 0,25 ms |
| Cliente, RX 9060 XT, DX12 | ~3500 FPS |

**Condición de salida cumplida:** luz + mallas de 64³ en unos 5 ms (límite 50).

## Pendiente o mejorable

- **W0 (mundo grande) no está hecho:** el plan pedía preparar el motor para coordenadas `i64` y negativas dentro de este hito. Ver la nota de pendiente en `PLAN_v4.md` (sección 5) y `docs/mundo/04-verificacion-e-hitos.md`. Conviene hacerlo antes del hito 3.
- **Probar el cliente a mano:** ratón, cavar, poner, saltar. Las pruebas automáticas cubren la simulación y las capturas, pero no la sensación de control.
- El tiempo de pico por bloque (`BlockKind::dig_ticks`) existe pero no se aplica: hoy se cava con un clic. Llegará con las clases (hito 3).
- La lava no fluye ni hace daño (hito 4).
- Luz suavizada por vértice y HUD con texto (glyphon) para el bloque elegido y los FPS.
- Escenas de captura en archivos RON (`tests/scenes/*.ron`), como propone el plan, en lugar de escenas en código.
