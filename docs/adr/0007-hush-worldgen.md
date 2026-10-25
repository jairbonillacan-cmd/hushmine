# ADR 0007: crate `hush-worldgen` con el generador de prueba del hito W0

- **Fecha:** 24 de octubre de 2026
- **Estado:** aceptada
- **Plan detallado:** `docs/mundo/02-generacion.md` y `docs/mundo/04-verificacion-e-hitos.md`

## Contexto

El hito W0 dejó el motor listo para el mundo grande (`ChunkPos` con negativos, `SparseWorld`, render relativo), pero sin generador: el plan pedía «un generador de mina de prueba con forma de función pura `(semilla, ChunkPos) → Chunk`» (ADR 0002, punto 5). Hace falta algo que produzca terreno en cualquier coordenada —también en ±7 499 999 y en negativas— para ejercer el acceso por chunks hoy y para que W1 (ruido y relieve de verdad) tenga un punto de partida sustituible.

## Decisión

- **Nuevo crate `hush-worldgen`**, registrado en el workspace. Depende solo de `grisu-voxel`: ni de `hush-sim` ni de crates externos.
- **`generate_chunk(seed, pos) -> Chunk` es una función pura**, sin estado compartido entre chunks. Todas las capas se calculan desde las coordenadas globales de cada bloque, así que el resultado no depende del orden de generación ni de los vecinos (esto es `INV-W02`, no una consecuencia a vigilar).
- **Ruido entero propio, sin `f32`**: el `hash3d` de `docs/mapgen/geologia.md` (mezcla SplitMix64) con una sal por capa. El relieve interpola bilinealmente una rejilla de esquinas por hash en punto fijo Q16 con intermedios `i64`. Igual que decidió el ADR 0002 para el ruido de W1, la portabilidad (nativo y wasm, cliente y servidor) exige enteros.
- **Mundo de prueba**: superficie cerca de `y = 0` (amplitud ±6), capas de tierra (3–5), roca y roca dura (desde 20–24 de profundidad), y mena (1/256), oro (1/1024, hondo) y diamante (1/4096, más hondo) por hash de bloque.
- **Ids de bloque duplicados como constantes `u8`** (`DIRT = 1`, `ROCK = 2`, …) en vez de depender de `hush_sim::BlockKind`: el generador no debe conocer las reglas del juego y la Convención A reserva `0..=17`. El precio es la duplicación; la vigila la prueba de ids esperados de `tests/inv_w01_determinismo.rs`.
- **`load_around(world, seed, center, radius, vertical)`** envuelve `SparseWorld::ensure_loaded` para cargar la zona alrededor de un punto.
- **Invariantes desde el primer día** (`tests/`): `INV-W01` (misma semilla y posición, mismo chunk byte a byte, sin estado entre llamadas), `INV-W02` (3×3×3 en órdenes distintos; las costuras coinciden con la función pura) y `INV-W03` (±7 499 999 y negativas sin desbordamientos, y proporciones de bloques parecidas a las del origen). El rendimiento se mide en una prueba ignorada: p50 ≤ 1 ms por chunk en release.

## Alternativas descartadas

- **Depender de `hush-sim` para los ids:** acoplaría el generador a las reglas y rompería la regla de dependencias del plan (el mundo se genera igual con otras reglas encima).
- **Un crate de ruido (`noise`, `fastnoise`):** todo en `f32`, pierde precisión lejos del origen y no garantiza el mismo resultado en nativo y wasm (ADR 0002, alternativas).
- **Generar dentro de `grisu-voxel`:** el motor no debe conocer terreno; `grisu-noise` y el generador real llegarán en W1 como crates propios.
- **Estado compartido (proto-chunks que se escriben entre sí):** dependería del orden y no se podría paralelizar ni probar tan directo.

## Consecuencias

- **Ventajas:** cualquier consumidor (pruebas, cliente, futuro servidor) genera el mismo terreno sin coordinación; las invariantes quedan ejecutándose en CI desde ya, no en W1.
- **Costes:** el mundo es deliberadamente simple (sin cuevas, biomas ni estructuras); los ids de bloque están duplicados; las probabilidades de menas son constantes del código, no datos.
- **Siguiente paso (W1):** `grisu-noise` con ruido de gradiente, fBm y muestreo grueso, más el parámetro de versión del generador (`worldgen_version`) y `INV-W11` (resúmenes congelados). Este crate queda entonces como generador de pruebas o se reconstruye sobre `grisu-noise`.
