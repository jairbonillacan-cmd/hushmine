# AGENTS.md: HUSHMINE / Grisú

Guía para agentes de IA (Claude Code, Opus) y para personas que trabajen en el repositorio.
El plan completo está en `PLAN_v4.md`; las decisiones de arquitectura, en `docs/adr/`.

## Qué es

**HUSHMINE** es un juego competitivo de minería en vóxeles, sucesor espiritual de Infiniminer. **Grisú** es el motor propio en Rust + wgpu que vive dentro del mismo workspace.

## Estructura

| Crate | Papel |
|---|---|
| `grisu-voxel` | Chunks de 16³, edición, mallas con oclusión ambiental, luz BFS, raycast DDA, colisión en punto fijo |
| `grisu-render` | wgpu: GPU, cámaras, terreno, cruz de apuntar, captura sin ventana |
| `grisu-platform` | Ventana winit, entrada (teclado y ratón) y bucle principal (trait `App`) |
| `grisu-net` | Trait `Socket` (UDP y red simulada en memoria), `Connection` con canal fiable y ordenado y canal no fiable |
| `hush-sim` | Reglas y simulación determinista, sin gráficos: bloques, terreno, movimiento, cavar y poner |
| `hush-net` | Protocolo, servidor autoritativo y cliente con predicción, sin E/S; arnés de partidas en red simulada |
| `hush-art` | Texturas provisionales de los bloques y su emisión de luz |
| `hush-server` | Binario `hush-server` (servidor dedicado UDP) |
| `hush-client` | Binario `hushmine` (el juego; servidor local en memoria o `--connect`) |
| `hush-launcher` | Panel del lanzador (`hush-launcher.exe`): estado, geometría y dibujo del panel en biblioteca, más el binario con ventana |
| `hush-cli` | Binario `hush-cli` (sim, netsim, shot, bench, gpu, story, region) |

**Regla de dependencias:** ningún crate `grisu-*` depende de un crate `hush-*`.

## Comandos de verificación

Ejecútalos antes de dar una tarea por terminada:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo run -p hush-cli -- shot --scene tunnel --out out/tunnel.png
cargo run -p hush-cli -- sim --seed 42 --bots 4 --report out/sim.json
cargo run --release -p hush-cli -- netsim --rtt 150 --loss 2 --seconds 60 --report out/netsim.json
cargo run --release -p hush-cli -- bench --seed 42
cargo run -p hush-cli -- gpu
cargo run -p hush-client -- --seed 42
cargo deny check licenses bans sources
```

Qué hace cada uno, en orden:

1. Formatea el código.
2. Pasa el analizador con los avisos tratados como errores.
3. Ejecuta todas las pruebas.
4. Genera una captura PNG sin ventana. **Ábrela y mírala.** Escenas: `cube`, `terrain`, `tunnel`, `shaft`, `dig` y `players`.
5. Simula una partida con bots que cavan; `broken_invariants` debe estar vacío.
6. Juega una partida en red simulada (2 clientes, 6 bots); falla si algún cliente termina con otro mundo u otra posición que el servidor.
7. Mide terreno, luz y mallas de 64³; falla si pasan de 50 ms (solo con `--release`).
8. Muestra la GPU y el backend en uso.
9. Abre el juego con ventana (controles abajo).
10. Revisa las licencias de las dependencias.

**Modo historia:** `cargo run -p hush-client -- --story [--seed N]` abre la expedición local (supervivencia a 20 Hz: hitos, fisiología, enemigos, fabricación y clímax del Pozo Ancestral). En `hush-cli`, `story --seed N --ticks T [--peaceful] [--report out/story.json]` corre una expedición con un minero automático y `region --seed N` guarda y recarga una región `.hush` verificando la huella. La captura `shot --scene story` muestra una galería con antorchas al atardecer (luz dual). Las reglas viven en `hush_sim::story` (ver `docs/adr/0006-modo-historia.md`); el diseño original está en `HUSHMINE_Modo_Historia_Diseno.md`.

**Controles del modo historia:** igual que el competitivo, más MAYÚS (agacharse), F (fabricación), E (usar banco/horno/arcón/válvulas/engranajes/motor), R (comer), F3 (telemetría).

**Multijugador por LAN:** `cargo run --release -p hush-server -- --bots 4` en una máquina y `cargo run --release -p hush-client -- --connect IP` en las demás (puerto UDP 47400 por defecto). Sin `--connect`, el cliente juega en local con un servidor en memoria (`--bots N` añade bots).

**Red:** nada en `grisu-net` ni en `hush-net` lee el reloj del sistema; todo recibe `now_ms`. Así las pruebas con latencia y pérdida son deterministas. Mantenlo así.

**Controles del cliente:** clic para capturar el ratón; WASD para andar, espacio para saltar, clic izquierdo para cavar, clic derecho para poner y 1-6 para elegir bloque. Escape suelta el ratón y, si ya estaba suelto, cierra el juego.

Con `HUSH_REQUIRE_GPU=1`, las pruebas de GPU fallan en vez de saltarse cuando no hay adaptador.
**El directorio de compilación puede ser global** en algunas máquinas (por ejemplo, `C:\Users\T490s\fossil-target\`); `cargo metadata` indica cuál es.

## Reglas

- **Nada de `unsafe`** (`unsafe_code = "forbid"` en el workspace). Si algún día hace falta FFI, irá en un módulo aislado y revisado.
- **El servidor y la simulación no hacen `panic!`** ante datos externos: devuelven `Result`.
- **Simulación determinista:** en `hush-sim`, enteros y punto fijo. Ni `f32`/`f64` en el estado, ni aleatoriedad del sistema (usar `hush_sim::Rng`), ni recorrer un `HashMap` cuando el orden importa.
- **El cliente solo dibuja, suena y envía órdenes;** las reglas viven en `hush-sim`.
- **Archivos pequeños:** un sistema por archivo, 400 líneas como máximo.
- **Cada funcionalidad llega con pruebas.** Si es visual, también con una escena de captura que se pueda revisar.
- **Las versiones de wgpu y winit están fijadas.** Se actualizan solo en una tarea dedicada al cerrar un hito.
- **Nuevas dependencias:** solo con licencia permisiva (`deny.toml`) y un registro en `docs/adr/`.
- **Idioma:** identificadores en inglés; comentarios, mensajes y documentación en español. El texto visible del juego irá en inglés como idioma fuente (Fluent).

## Problemas conocidos

- **Windows + Intel UHD 620 (driver 31.0.101.2137):** el loader de Vulkan termina el proceso al crear la instancia. Por eso `grisu_render::default_backends()` usa DX12 + GL en Windows. Se puede forzar otro backend con `WGPU_BACKEND`.
- **Avisos de versiones duplicadas en `cargo deny`:** vienen de combinar winit 0.30 con wgpu 30. No bloquean y se revisarán al pasar a winit 0.31 cuando sea estable.
