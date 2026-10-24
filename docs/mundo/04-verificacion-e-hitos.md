# Mundo grande · 4. Verificación, hitos y riesgos

Cómo se comprueba que el mundo es bueno sin tener que recorrerlo a mano, y en qué orden se construye. Índice en [README.md](README.md).

---

## 1. Bucle de verificación de la IA

Igual que en el modo Turno, la IA tiene que poder ver el mundo sin abrir el juego. Nuevo subcomando `hush-cli world`, todo en `--release`:

```bash
cargo run -p hush-cli --release -- world map --seed 42 --center 0,0 --size 4096 --layer biomes --out out/world/biomas.png
```
Mapa cenital de 4 096 × 4 096 bloques. Capas: `biomes`, `height`, `caves` (aire a una cota dada), `ores`, `structures`, `aquifers`.

```bash
cargo run -p hush-cli --release -- world slice --seed 42 --axis x --at 0 --from -512,-1024 --to 512,1023 --out out/world/corte.png
```
Corte vertical en color por bloque: estratos, cuevas, acuíferos, minas y el Corazón de un vistazo.

```bash
cargo run -p hush-cli -- shot --world 42 --at 1200,-300,-800 --look 0,-10 --out out/world/cueva.png
```
Captura 3D sin ventana dentro del mundo, con luz y niebla. **Ábrela y mírala.**

```bash
cargo run -p hush-cli --release -- world chunk --seed 42 --chunk -468750,-10,468749 --digest
```
Resumen SHA-256 de un chunk concreto, en coordenadas de chunk (aquí, la esquina del mundo).

```bash
cargo run -p hush-cli --release -- world stats --seed 42 --samples 4096 --report out/world/stats.json
```
Histogramas por profundidad (menas, aire, agua, lava), frecuencia de biomas y estructuras, y comparación con las curvas de `data/worldgen/*.ron`.

```bash
cargo run -p hush-cli --release -- world bench --seed 42 --chunks 4096 --out out/world/bench.csv
```
Tiempo por fase y por chunk (p50, p95, p99).

```bash
cargo run -p hush-cli --release -- world walk --seed 42 --km 10 --speed 16 --report out/world/walk.json
```
Simula un jugador en vagoneta: chunks pendientes, memoria y tiempo de fotograma estimado durante el recorrido.

**Escenas de mundo:** `tests/scenes/mundo/*.ron` guarda puntos de interés fijos (semilla, posición y cámara): una mina, un pueblo, una geoda, el Corazón, el borde del mundo. Cada cambio del generador regenera sus capturas para revisarlas.

**Recarga en caliente:** `hush-cli world map --watch` vuelve a dibujar el mapa al guardar un RON de `data/worldgen/`. Ajustar biomas y cuevas pasa a ser cuestión de segundos.

## 2. Invariantes del mundo

| Id | Invariante | Cómo se prueba |
|---|---|---|
| `INV-W01` | **Determinismo:** misma semilla, versión y posición dan el mismo chunk byte a byte, con cualquier número de hilos y en nativo y wasm | `proptest` + comparación de resúmenes nativo/wasm en CI |
| `INV-W02` | **Sin costuras:** generar una zona de 3 × 3 × 3 chunks en cualquier orden da el mismo resultado | `proptest` con órdenes aleatorios |
| `INV-W03` | **Coordenadas extremas:** sin desbordamiento en todo el rango y ruido con las mismas estadísticas que en el origen | `proptest` sobre todo el rango; estadísticas en ±7 499 999 |
| `INV-W04` | **Borde:** la columna del borde es indestructible y ninguna estructura lo cruza | Prueba de colocación en celdas del borde |
| `INV-W05` | **Fluidos estables:** ninguna celda de agua o lava puede fluir en el tick 0 | Un tick de simulación de fluidos sobre chunks de muestra no cambia nada |
| `INV-W06` | **Inicio seguro:** el punto de aparición está sobre suelo firme, con aire encima y sin lava ni agua profunda a menos de 16 bloques | Prueba con 1 000 semillas |
| `INV-W07` | **Distribución de menas:** dentro de ±15 % de las curvas de `menas.ron` por franja de profundidad | `world stats` en CI con 4 096 chunks |
| `INV-W08` | **Estructuras coherentes:** las piezas no se solapan, todos los conectores están cerrados y todas las salas de una mina se alcanzan desde su pozo | Pruebas sobre planos, sin generar chunks |
| `INV-W09` | **Historia:** hay camino desde el inicio hasta el Corazón del Durmiente Mayor y cada ficha de historia aparece exactamente una vez | BFS sobre la región de inicio, con 1 000 semillas |
| `INV-W10` | **Rendimiento:** presupuestos de [01-escala-y-motor.md](01-escala-y-motor.md) y memoria estable tras `world walk --km 10` | `world bench` y `world walk` en CI nocturna |
| `INV-W11` | **Versión congelada:** los resúmenes de 256 chunks de referencia no cambian sin subir `worldgen_version` | Instantáneas con `insta` |
| `INV-W12` | **Épocas del Sueño:** misma semilla y época dan el mismo Sueño, y ningún jugador queda dentro de roca al cambiar de época | `proptest` sobre cambios de época (`docs/supervivencia/06-dimensiones.md`) |
| `INV-W13` | **Eco acotado:** los bloques de eco activos por zona tienen un tope | `proptest` con ruido máximo sostenido |

## 3. Hitos del mundo

Los hitos W1–W7 llegan **después** del acceso anticipado del modo Turno (hito 7 de `PLAN_v4.md`) y solo si el juego tiene tracción. Hacerlos antes iría contra la regla del plan: el motor solo hace lo que HUSHMINE necesita en cada momento. **W0 es la excepción**: cuesta días ahora y semanas después.

| Hito | Contenido | Condición de salida | Aprox. |
|---|---|---|---|
| **W0. Puertas abiertas** (dentro del hito 1) | Posición mundial `i64` Q47.16; chunk detrás de una interfaz (`get`/`set`/`fill`); `ChunkPos` con negativos; render relativo a la cámara; generador de mina de prueba con forma de función pura `(semilla, ChunkPos) → Chunk` | Pruebas de chunk, DDA y colisión en coordenadas negativas y en ±7 499 999 en verde. Mapa de 64³ sin cambios visibles | 2–4 días |
| **W1. Ruido y relieve** | `grisu-noise`; fases Columna, Densidad y Litología; `world map`, `world slice`, `world chunk`, `world bench` | Mapas y cortes PNG coherentes; p95 ≤ 3 ms por chunk; `INV-W01`, `W03` y `W11` en verde | 2–3 sem |
| **W2. Mundo recorrible** | `grisu-world`: planificador, trabajadores, LRU, luz entre chunks, mallado incremental, archivos de región, horizonte lejano | Recorrer 10 km en el T490s sin tirones y con memoria estable; guardar, salir y volver deja el mundo igual (`INV-W02`, `W10`) | 3–4 sem |
| **W3. Cuevas, fluidos y menas** | Todos los excavadores, acuíferos, lava, rocas, fallas, kimberlita, menas, decoración | `INV-W05` y `W07` en verde; capturas de escenas de cueva aprobadas a ojo | 3–4 sem |
| **W4. Biomas** | Biomas de superficie y subterráneos, sonido y luz ambiental por bioma | Mapa de biomas sin escalones en las fronteras; cada bioma tiene su escena de captura | 2–3 sem |
| **W5. Estructuras** | Sistema de piezas, minas de los Antiguos, cateos, castilletes, pueblos, aldeas de los Quedos, caminos; exportación de plantillas desde el Taller | `INV-W04` y `W08` en verde; una mina generada «se lee» como mina en capturas y en juego | 4–6 sem |
| **W6. Historia y multijugador** | Anclas de la región de inicio, gran mina de los Antiguos, Corazón, Bóveda, fichas; red con generación local y revelación de menas; servidor persistente | `INV-W09` en verde; cuatro jugadores en un servidor por Internet sin rayos X | 4–6 sem |
| **W7. Dimensiones** | Clave de chunk con dimensión, perfiles de generador, gravedad configurable; el Sueño con épocas y Recuerdos, el Hervor, el Eco y el Revés | `INV-W12` y `W13` en verde; se entra y se sale de cada dimensión sin perder nada propio | 6–10 sem |

**Diseño del juego:** el modo Historia (lore, historia, pueblos, comercio, criaturas, jefes, tecnologías y dimensiones) está en `docs/supervivencia/`. Este plan asegura que el mundo esté listo para él.

## 4. Riesgos

| Riesgo | Mitigación |
|---|---|
| El mundo grande se come el juego competitivo | Solo W0 antes de la 1.0. W1–W7 dependen de que el modo Turno tenga tracción |
| El ruido entero sale feo o con artefactos de retícula | Rotación entera, deformación de dominio y revisión visual con `world map` desde W1. Si fallara, se prueba ruido *simplex* entero antes de pensar en `f32` |
| La generación no cumple 1 ms por chunk en el T490s | Muestreo grueso con interpolación, caché por columna, perfilado con Tracy y, como último recurso, preset Bajo con R = 6 |
| Costuras entre chunks o estructuras cortadas | Regla de funciones puras, `INV-W02` y `INV-W08` con `proptest` |
| Un cambio en el generador rompe mundos guardados | `worldgen_version`, `world freeze` e instantáneas `INV-W11` |
| Trampas de rayos X en servidores | Generación local con menas ocultas y revelación desde el servidor |
| Navegador sin hilos | Un solo trabajador y preset Bajo; la supervivencia en web no es prioritaria |
| Demasiado contenido para una persona | Biomas y estructuras en datos RON y plantillas del Taller. La IA genera variantes y las capturas permiten revisarlas rápido |

## 5. Decisiones abiertas

1. **Altura final del mundo:** −4 096 … 1 023 es la propuesta. ¿Hace falta tanta Bóveda? Se decide al tener W3 y poder recorrerla.
2. **Otros Durmientes:** ¿cuántos hay por mundo y cada cuánto? ¿Tienen historia propia o solo región en el Sueño?
3. **Nombres de la Bóveda, las Raíces y las Vetas soñadas:** propuestos; se confirman con el resto del lore del modo Historia.
4. **Lista final de menas y bloques de supervivencia:** pendiente del documento de diseño de la supervivencia.
5. **Mecánica del grisú y sismógrafo local:** propuesta; hay que probarlas.
6. **Regeneración de oro por épocas:** propuesta; depende del balance.
7. **Radio del hilo principal:** la historia ocurre en la región del Durmiente Mayor y las dimensiones. ¿Hace falta algo de la historia lejos de esa región?
