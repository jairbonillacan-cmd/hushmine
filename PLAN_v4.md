# HUSHMINE: plan v4 · juego y motor propios

Prefijo de los crates del juego: `hush-*`. Motor: `grisu-*`.

Revisión del 22 de octubre de 2026. Sustituye a las versiones anteriores (`PLAN.md`, `PLAN_v2.md`, `PLAN_v3.md`).

**Decisiones fijadas:**

- **Motor propio** con tecnologías modernas y librerías de código abierto.
- **Desarrollo individual con IA.**
- **Prioridades:** rendimiento y multiplataforma.
- **Modelo de estudio independiente:** publicar pronto, crecer con la comunidad y decidir después si el proyecto da para algo más grande.

Cifras y plazos son objetivos por validar.

---

## 0. La decisión de fondo: ¿motor propio, Unity o el camino de Minecraft?

### Los juegos de vóxeles que triunfan usan motor propio

| Juego | Motor | Lección |
|---|---|---|
| Minecraft (2009) | Propio: Java + LWJGL (OpenGL) | Motor pequeño, alfa vendida muy pronto y una **versión jugable en el navegador** (Minecraft Classic) que lo difundió |
| Vintage Story | Propio, en C# | Un estudio pequeño puede mantener su motor durante años |
| Teardown | Propio, en C++ | La destrucción voxel como seña de identidad exige control total del motor |
| Veloren | Propio, en Rust + wgpu, código abierto | Prueba que esta pila tecnológica sirve para un juego de vóxeles multijugador |
| Luanti (antes Minetest) | Propio, en C++ | Comunidad y mods a largo plazo |
| **Hytale** | Reescribió su motor para hacerlo multiplataforma… y **lo abandonó**. Volvió a una versión antigua para salir en acceso anticipado en enero de 2026 ([Hytale](https://hytale.com/news/2025/11/hytale-early-access-january-13-2026)) | **No hay que planear una migración de motor: hay que construir el motor que vas a conservar** |

### Conclusiones

1. **Para un juego de vóxeles, Unity no es «un motor mejor»: es un motor genérico.** El trabajo difícil (terreno editable, mallas, red del terreno) habría que hacerlo a mano igualmente. Migrar más adelante solo tendría sentido para llegar a consolas o móviles con presupuesto y equipo, y aun así el núcleo de simulación debería poder reutilizarse.
2. **El camino de Minecraft es el correcto**, con las tecnologías de 2026:
   - un motor pequeño y propio;
   - una **demo jugable en el navegador** (WebGPU ya funciona en Chrome, Edge, Firefox y Safari: [web.dev](https://web.dev/blog/webgpu-supported-major-browsers));
   - alfa temprana y comunidad;
   - un modo creativo, porque fue en Infiniminer donde la gente empezó a construir, y de ahí nació Minecraft.
3. **El motor se construye dentro del juego y no como proyecto aparte.** Los motores sin juego mueren. Se organiza en crates independientes desde el primer día y **se publica como proyecto de código abierto (nombre propuesto: Grisú) cuando el juego esté en acceso anticipado**, no antes.

---

## 1. Tecnología: Rust + wgpu

### Por qué Rust

| Criterio | Rust + wgpu |
|---|---|
| Rendimiento | Nativo, sin recolector de basura; paralelismo seguro con rayon |
| Multiplataforma | Windows, Linux, macOS y Steam Deck nativos, más **navegador** (WebGPU, con WebGL2 de respaldo), todo desde el mismo código |
| Moderno | wgpu (v29) implementa el estándar WebGPU sobre Vulkan, Metal, DX12 y GL. Los shaders se escriben en WGSL |
| Programación con IA | El compilador estricto es un verificador gratuito: si compila, se descartan clases enteras de errores (memoria, carreras de datos, nulos). Sus mensajes de error son muy precisos para que la IA los corrija |
| Servidores | Un servidor en Rust no se cae por excepciones de memoria y consume muy poco |
| Determinismo | La aritmética entera es idéntica en nativo y en WebAssembly, así que la simulación en punto fijo se comporta igual en todas partes |
| Mods a largo plazo | Mods en **WebAssembly con wasmtime**: aislados, seguros y multiplataforma. Es la respuesta moderna a la fragmentación que mató a Infiniminer |

### Alternativas evaluadas

| Opción | A favor | En contra | Veredicto |
|---|---|---|---|
| **Rust + wgpu** | Todo lo anterior | Compilación más lenta; wgpu cambia su API cada pocos meses (se fija la versión y se actualiza una vez por hito); no hay backends de consola | **Elegida** |
| C++23 + SDL3 (SDL_GPU) | Máximo control; SDL tiene ports a consola (con acuerdo de confidencialidad) | Errores de memoria, CMake, ciclo de IA más lento; la web solo por Emscripten y WebGL | Segunda opción si algún día las consolas son prioridad |
| C# .NET 10 + SDL3 + renderizador propio | La IA itera más rápido; el código sería portable a Unity | Recolector de basura; sin una buena salida web; menos «nuevo» | Válida, pero no cumple «lo más moderno y rápido» |
| Bevy | Motor Rust completo | No es «desde cero»; su API cambia mucho en cada versión | Se descarta, aunque se pueden usar sus ideas o crates sueltos |
| Zig u Odin | Modernos y rápidos | Ecosistema pequeño; la IA los conoce peor | Descartadas |

### Librerías de código abierto (todas MIT, Apache o Zlib)

| Área | Crate | Uso |
|---|---|---|
| Ventana y entrada | `winit` + `gilrs` | Ventana nativa y web; mandos y Steam Deck |
| Gráficos | `wgpu` + WGSL | Renderizado en todas las plataformas |
| Matemáticas | `glam` | Vectores y matrices SIMD (solo en presentación; la simulación usa punto fijo) |
| Texto | `glyphon` + `cosmic-text` | Texto de calidad para el HUD (acentos, ñ) |
| Interfaz de depuración y menús iniciales | `egui` | Inspectores, consola, menús provisionales |
| Audio | `kira` | Mezcla, efectos y audio espacial; la propagación por túneles es propia |
| Red | `renet2` | UDP nativo + **WebTransport** para navegador + memoria (servidor local), con varios transportes simultáneos en un servidor |
| Steam | `steamworks` (steamworks-rs) | Relé de Steam y amigos. Transporte propio separado, **por verificar en el hito 5** |
| Serialización | `bitcode` (red) y `serde` + `ron` (datos) | Paquetes compactos; datos legibles |
| Compresión | `lz4_flex` | Rust puro; funciona en WebAssembly |
| Paralelismo | `rayon` | Mallas y luz en varios hilos (en web, con un solo hilo de respaldo) |
| Base de datos (servidor) | `rusqlite` | Estadísticas, colecciones, bloqueos |
| Idiomas | `fluent` (Project Fluent) | Español e inglés con plurales y variables |
| Perfilado | `tracing` + **Tracy** | Perfilador moderno de fotogramas e hilos |
| Pruebas | `cargo-nextest`, `insta` (instantáneas), `proptest` (propiedades e invariantes) | Verificación automática |
| Licencias | `cargo-deny` | Bloquea dependencias con licencias incompatibles (GPL) |
| Mods (después de la 1.0) | `wasmtime` | Mods de servidor en WebAssembly |

### Estructura del repositorio (motor separable desde el primer día)

```
crates/
  grisu-voxel     chunks, edición, mallas (CPU), luz, raycast DDA, colisión
  grisu-render    wgpu: chunks, entidades de cubos, niebla, bloom, texto, captura sin ventana
  grisu-audio     kira + propagación sobre la cuadrícula
  grisu-net       transportes (UDP, WebTransport, memoria, Steam), marcos, deltas, predicción
  grisu-platform  ventana, entrada, tiempo, archivos (nativo y web)
  hush-sim        reglas del juego en punto fijo, sin gráficos: partida, clases, economía, lava, sismógrafo
  hush-client     binario nativo y wasm
  hush-server     binario del servidor dedicado (el mismo código va como servidor local dentro del cliente)
  hush-bots       bots que cavan y juegan
  hush-cli        sim, shot, replay, bench
assets/  data/  docs/  AGENTS.md
```

**Regla de dependencias:** ningún crate `grisu-*` depende de un crate `hush-*`. Así el motor se puede publicar por separado más adelante.

### El bucle de verificación de la IA

La IA puede comprobar su propio trabajo sin abrir el juego. Estos comandos se crean en el hito 0:

```bash
cargo clippy --workspace -- -D warnings
```
Errores y estilo en segundos.

```bash
cargo nextest run --workspace
```
Pruebas, instantáneas de mallas y del protocolo, y propiedades.

```bash
cargo run -p hush-cli -- sim --seed 42 --bots 8 --report out/sim.json
```
Partida completa sin gráficos. Devuelve un JSON con el resultado, los eventos, el p95 de tick y las invariantes que se rompieron.

```bash
cargo run -p hush-cli -- shot --scene tests/scenes/tunel_lava.ron --out out/tunel.png
```
Dibuja una escena **sin ventana** y la guarda como PNG; Claude la lee para comprobar el resultado. En CI se usa Vulkan por software (lavapipe) en Linux.

```bash
cargo run -p hush-cli -- replay out/partida.zrep --at 07:12 --shot out/momento.png
```
Reproduce un error exacto a partir de una repetición.

```bash
cargo run -p hush-cli --release -- bench --scenario ref --out out/bench.csv
```
Tiempos de fotograma y de tick, con Tracy disponible para el detalle.

**Invariantes verificadas con `proptest` y en cada simulación:**

- **Conservación del oro:** el total solo cambia al minar o al depositar.
- Ningún cliente recibe minerales que su equipo no conoce.
- El terreno del cliente coincide con el del servidor tras cada partida simulada.
- La simulación es determinista: mismas entradas, mismo estado, byte a byte, en nativo y en wasm.

**Ritmo de compilación:**

- Crates pequeños.
- Dependencias optimizadas también en el perfil de desarrollo (`[profile.dev.package."*"] opt-level = 3`).
- Enlazador rápido (`lld` o `mold`).
- Recarga en caliente de shaders WGSL y de datos RON.
- La recarga de código en caliente (`subsecond`) es experimental y opcional.

**Reglas para `AGENTS.md`:**

- Nada de `unsafe` fuera de un módulo de FFI revisado.
- El servidor nunca hace `panic!`: todo devuelve `Result`.
- Un sistema por archivo, de 400 líneas como máximo.
- El cliente no contiene reglas de juego.
- Cada funcionalidad llega con una prueba y, si es visual, con una escena de captura.
- **wgpu y winit se fijan a una versión** y solo se actualizan en una tarea dedicada al cerrar cada hito.

### Plataformas

| Plataforma | Cuándo | Notas |
|---|---|---|
| Windows x64 | Desde el hito 0 | DX12 o Vulkan |
| Linux x64 y **Steam Deck** | Desde el hito 0 | Vulkan; tu T490s sirve de banco de pruebas de gama baja |
| macOS Apple Silicon | Desde el hito 0 (en CI) | Metal; notarización para distribuir fuera de Steam |
| **Navegador** | Hito 5 | WebGPU (WebGL2 de respaldo) y WebTransport (disponible en todos los navegadores principales desde marzo de 2026, según [Utsubo](https://www.utsubo.com/blog/frontier-web-apis-2026-production-ready); hay que confirmarlo). Será la **demo de un clic**, como Minecraft Classic |
| Servidor | Desde el hito 2 | Linux, Windows, macOS y ARM64; un binario estático y una imagen Docker |
| Móvil | Futuro | wgpu admite iOS y Android; habría que adaptar los controles |
| Consolas | Solo si el juego triunfa | wgpu no tiene backend de consola. **Este es el único escenario que justificaría portar** (a SDL3/C++ o a otro motor), reutilizando el diseño de `hush-sim` |

---

## 2. Nombre, historia e identidad internacional

### Nombre comercial: **HUSHMINE** (elegido)

> ***Rob a sleeping mountain. Play dirty. Keep it hushed.***

- **El nombre es la mecánica principal.** *Hush* significa «silencio»: el ruido despierta al Durmiente, y cada jugador decide cuánto ruido se puede permitir.
- **Doble sentido:** *hush money* es un soborno para que alguien calle. La **ofrenda** es exactamente eso: pagar oro para que la montaña siga callada.
- **Fácil de pronunciar y recordar** en cualquier idioma: dos palabras básicas de inglés (*hush* + *mine*) unidas en un término inventado, buscable y sin competidores encontrados.
- **Su punto débil** es que suena a sigilo más que a competición. Por eso el **lema y la cápsula de Steam muestran el sabotaje**: alguien corta la línea rival mientras los cangilones caen y el sismógrafo se dispara.
- **Antes de anunciarlo:** comprobar marcas (USPTO y EUIPO), Steam, itch.io, el dominio (hushmine.com / .gg / .game) y las redes (@hushmine).

| Alternativa descartada | Motivo |
|---|---|
| DIG DIRTY | Más directo, pero menos propio y más difícil de buscar; queda como posible frase de marketing |
| DEEPWAKE | Atmosférico, pero no transmite el juego |
| *UnderMine*, *Deep Shift*, *Grave Shift*, *Claim Jumper*, *Countermine* | Ya existen o son confusos |

**El motor sigue llamándose Grisú** (del francés *grisou*: el gas de las minas). Es un término minero que existe en varios idiomas.

### Premisa (texto canónico en inglés para la tienda)

> **1931.** Mount Anima was the richest mountain on Earth, and miners came from every corner of the world to dig it. Then, one night, the mountain woke up. The entire graveyard shift — forty-one miners from twelve countries — never came back up. Their brass tallies still hang missing from the board.
>
> **1979.** The mountain sleeps again… and it dreams of gold. Every time it settles, its tunnels shift and fresh veins bloom. Every night, rival crews slip past the fences to rob a sleeping giant.
>
> The old-timers have one rule: ***don't wake the Sleeper.***

*En español:* En 1931, el monte Anima era la montaña más rica del mundo, y llegaron mineros de todos los rincones. Una noche despertó y se tragó entero el turno de noche: cuarenta y un mineros de doce países. En 1979 vuelve a dormir y sueña oro: sus túneles cambian de sitio y brotan vetas nuevas. Cada noche, cuadrillas rivales saltan las vallas para robar a un gigante dormido. Los veteranos solo tienen una regla: *no despiertes al Durmiente*.

### Por qué funciona en todo el mundo

- **Lugar ficticio sin país:** un pueblo minero de inmigrantes de todas partes, como los auges mineros reales (California, el Klondike, Potosí, Kalgoorlie). Cualquier jugador puede verse ahí, y los mineros son personalizables con nombres y aspectos de cualquier cultura.
- **Folclore universal:** casi todas las culturas mineras creyeron que la montaña estaba viva y avisaba o castigaba:
  - los *tommyknockers* de Cornualles golpeaban antes de los derrumbes, y los mineros les dejaban parte de su comida;
  - los *kobolds* alemanes dieron nombre al **cobalto**;
  - en los Andes se hacen ofrendas al Tío.

  Las criaturas del juego son originales, pero cualquiera reconoce la idea.
- **Un detalle histórico real:** las **fichas de latón** (*brass tallies*, *pit checks*). Cada minero cogía una al entrar y la devolvía al salir; una ficha que faltaba significaba un minero bajo tierra. Las **41 fichas que faltan** son el coleccionable y el corazón emocional de la historia.
- **Época retro global (1931/1979):** casetes, pantallas de fósforo verde, lámparas de sodio. Es una estética que no pertenece a ningún país.

### Personajes y mundo

- **Copper Crew** (naranja): las familias antiguas del pueblo, descendientes del turno perdido. Bajan por el oro… y por los suyos.
- **Cobalt Crew** (azul): toman su nombre de los kobolds: *«los mineros a los que los duendes no asustan»*. Recién llegados con material de la Compañía. Equilibrados en juego; distintos solo en estética.
- **The Watcher (el Vigía):** lleva el sismógrafo y la radio. Narra con frases muy cortas y fáciles de traducir: *«She's stirring…»*, *«She's breathing. Get out.»*
- **The Whistlers (los Silbadores):** silbidos antes de cada derrumbe. ¿Son ecos de los 41? Son el aviso diegético y la base de un futuro modo con criaturas.
- **Anima Consolidated:** la compañía quebrada. Quedan sus vallas, carteles de seguridad y memorandos, que dan humor ambiental y se entiende con iconos.

### Misterio a largo plazo (temporadas)

- **Temporada 1, *The Long Night*:** recuperar las 41 fichas (colección comunitaria por servidor) revela qué pasó en 1931.
- **Temporadas siguientes:** qué es el Durmiente, por qué sueña oro, qué hay bajo el Corazón.

**No hay campaña ni cinemáticas**, así que el coste se mantiene bajo.

### Glosario canónico (inglés → español)

| Inglés | Español | Inglés | Español |
|---|---|---|---|
| The Sleeper | el Durmiente | Shift (partida) | Turno |
| Seismograph | Sismógrafo | Stir / Breathe / Restless / Wake | Revuelo / Respira / Inquietud / Despertar |
| Mother Lode | Veta Madre | The Heart | el Corazón |
| Offering | Ofrenda | Brass tally | Ficha de latón |
| Bucket line | Línea de cangilones | Geophone | Geófono |
| Prospector · Miner · Engineer · Sapper | Prospector · Minero · Ingeniero · Zapador | Workshop | Taller |

### Idiomas

- **Todo el texto del juego se escribe primero en inglés**; el español es la primera traducción.
- **Orden de traducción previsto:** inglés, español (Latinoamérica y España), portugués de Brasil, chino simplificado, ruso, alemán, francés y japonés. Son mercados grandes de Steam.
- **Diseño con poco texto:** iconos, color, sonidos diegéticos y frases de radio cortas. Se usa Fluent y fuentes Noto (licencia OFL) con CJK desde el principio.
- **Revisión cultural** con jugadores de varias regiones antes de la alfa.

---

## 3. Reglas (Infiniminer + el Durmiente)

### Base heredada de Infiniminer

- **Dos equipos**, de 1 contra 1 a 8 contra 8. Los bots completan los huecos.
- **Mapa** de 64×64×64 (96×96×64 para 8 contra 8), nuevo en cada partida.
- **Economía doble:** la **mena** se gasta en construir; **oro y diamante** se depositan en el **banco** del equipo.
- **Clases:**
  - **Prospector:** detector y **geófono** para oír excavaciones a través de la roca.
  - **Minero:** pica rápido y carga el doble.
  - **Ingeniero:** construye más tipos de bloque y la **línea de cangilones**.
  - **Zapador:** explosivos y cizalla.
- **Bloques:**
  - naturales: tierra, roca, roca dura, mena, oro, diamante, **lava** que fluye;
  - de construcción: bloque de equipo, **campo de fuerza** que solo cruza tu equipo, escalera, plataforma de salto, **bloque eléctrico**, carril, baliza, explosivo, estación y poste de cangilones.
- **No hay armas.** Se muere por lava, electricidad, explosivos, caídas o derrumbes. El pico solo empuja.
- **Reaparición** en 6 s; al morir se suelta lo que se cargaba en una **bolsa** que cualquiera puede recoger.

### Lo nuevo: el Sismógrafo

Es un **medidor compartido de 0 a 100 %** que todos ven. Mide lo cerca que está el Durmiente de despertar.

| Fuente | Efecto |
|---|---|
| Explosión | +3 % |
| Línea de cangilones en marcha | +0,3 % por cada 10 s |
| Descarga de un bloque eléctrico, lava que fluye | +0,2 % |
| Picar | Casi nada |
| Paso del tiempo | Sube poco a poco: el despertar llega como muy tarde hacia el minuto 16 |
| **Ofrenda** (depositar oro en tu altar) | **Baja el medidor**: compras tiempo con dinero |
| **Devolver una chapa** al altar | Lo baja mucho y desbloquea un fragmento de historia |

**Umbrales:**

| Nivel | Nombre | Qué pasa |
|---|---|---|
| 25 % | **Revuelo** | Temblores y pequeños derrumbes, avisados por los Silbadores |
| 50 % | **Respira** | Se abre una caverna con una **Veta Madre**; el temblor indica más o menos dónde |
| 75 % | **Inquietud** | Se rompen bolsas de lava y se abren o cierran pasos |
| 100 % | **Despertar** | Quedan 90 s finales: la lava sube desde el fondo, las líneas se paran en los últimos 30 s y en el fondo aparece el **Corazón**, un depósito enorme rodeado de lava. Al terminar, el cerro se traga la mina |

**Victoria:** gana el primer equipo que llega a la meta de dinero (al instante). Si nadie llega, gana quien tenga más dinero cuando termine el Despertar. Los empates se mantienen.

### Por qué funciona como partida

- **Quien va ganando quiere hacer ruido** (cargas, líneas a pleno rendimiento) para acabar antes.
- **Quien va perdiendo quiere silencio y tiempo:** ofrenda oro y busca chapas… pero necesita ruido para remontar.
- Sabotear la línea del líder **la apaga y quita ruido**. Volar su túnel **añade ruido**. Cada acción tiene dos caras.
- **La duración la deciden los jugadores**, normalmente entre 9 y 14 minutos, y ninguna partida se parece a otra.
- Siguen activos los mecanismos de remontada de la v3:
  - **Fama:** el equipo que pasa del 60 % de la meta queda marcado con una baliza visible.
  - **Botín robado ×1,25.**
  - **El Corazón** como última apuesta.

### Desarrollo típico de una partida

1. **Bajada** (20 s): elección de clase y mapa geológico aproximado.
2. **Fiebre:** oro superficial, primeras fortificaciones.
3. **Revuelo:** primeros cruces de túneles y uso del geófono.
4. **Respira:** carrera hacia la Veta Madre.
5. **Inquietud:** el mapa cambia y empieza el sabotaje.
6. **Despertar:** o el Corazón o la salida.

### Crónica posterior

Es un resumen automático de 5 a 7 momentos clave sacados del registro de eventos. Por ejemplo:

> *«Min 9:40 · Cobalto despertó al Durmiente con tres cargas seguidas mientras Cobre ofrendaba oro para ganar tiempo.»*

Incluye títulos para los jugadores: **Despertador** (más ruido), **Guardián del sueño** (más ofrendas), **Topo** (más metros cavados), **Ladrón de cangilones**. Se puede compartir como imagen. El servidor también guarda una repetición completa.

### Modos

| Modo | Versión | Descripción |
|---|---|---|
| **Turno** (*Shift*) | 1.0 | El modo competitivo descrito arriba |
| **Taller** | 1.0 | Modo creativo libre: construir sin límites, que además sirve de **editor de mapas y plantillas**. Cuesta poco hacerlo y es la semilla de comunidad de Minecraft |
| **Práctica** | 1.0 | Contra bots |
| **Asedio** | Después de la 1.0 | Cavar bajo el banco enemigo y volarlo con una carga mayor (guerra de zapadores clásica) |
| **La Mina Eterna** | Si hay tracción | Servidor persistente en un cerro grande con temporadas: el «camino de Minecraft» |
| **Historia (supervivencia)** | Si hay tracción (hitos W1–W7) | Supervivencia y construcción en un mundo abierto y único por semilla, de 15 M × 15 M bloques, que se genera al explorar. Todo se pica y todo se construye. Ocurre en el Silencio, un lugar sin fechas que no es la Tierra, con historia opcional, pueblos con comercio, criaturas, jefes, tecnologías propias y cuatro dimensiones. Ver `docs/mundo/` y `docs/supervivencia/` |

---

## 4. Motor Grisú: arquitectura y rendimiento

### Terreno

- Chunks de 16³ con 1 byte por bloque más metadatos dispersos. Un mapa de 64³ ocupa 256 KB; uno de 96×96×64, 576 KB.
- **Primera versión:** eliminación de caras ocultas con rayon. **Después, si se mide la necesidad:** *greedy meshing* binario.
- **Caras empaquetadas** en 8 bytes y expandidas en el shader («*vertex pulling*»), más un *texture array* cuyo tamaño sale del manifiesto (16×16 provisional, 32×32 desde el hito 6; ver `docs/graficos/`). Para WebGL2 hay una ruta clásica de vértices.
- **Luz por bloques** (BFS) y oclusión ambiental por vértice. Más adelante, **recorte por conectividad de cuevas** y *multi-draw indirect* cuando la plataforma lo admita.
- **Colisión en punto fijo** contra la cuadrícula, compartida entre cliente y servidor.

### Simulación y red

- **Servidor autoritativo** a 30 Hz. `hush-sim` es determinista: enteros, generador aleatorio con semilla y orden de eventos fijo.
- **Predicción** del movimiento y de la excavación local; los demás jugadores se interpolan.
- **Mapa completo al unirse** (paleta + RLE + LZ4, hay que medirlo) y después deltas numerados.
- **Lava y Sismógrafo** se simulan solo en el servidor; los cambios se envían en lotes.
- **Niebla de mineral por equipo** (un bit por bloque y equipo).
- **Transportes:**
  - UDP (LAN y servidor dedicado);
  - WebTransport (navegador);
  - memoria (servidor local);
  - Steam (relé), con implementación propia separada.

  Los mensajes del juego no dependen del transporte.
- **WebTransport en servidores comunitarios** necesita certificados; los navegadores permiten certificados autofirmados de corta duración mediante su huella. Se documentará.

### Presupuestos

| Escenario | Objetivo |
|---|---|
| GTX 1660 | 1080p, p95 ≤6,9 ms (144 FPS) |
| Intel UHD 620 (T490s) | 1080p, p95 ≤16,7 ms |
| Steam Deck | 800p, p95 ≤16,7 ms |
| Navegador (WebGPU, portátil medio) | 1080p, p95 ≤16,7 ms |
| Memoria del cliente | ≤400 MB nativo |
| Servidor, 8 contra 8 | p95 de tick ≤2 ms, RAM ≤64 MB |
| Red | ≤32 KB/s por cliente de media |
| Arranque y unión | Menos de 2 s en nativo y menos de 5 s en web (primera carga) |

---

## 5. Hitos

Los plazos son aspiracionales. El motor propio añade unas 6–8 semanas respecto a usar un framework, pero ese tiempo se queda en el proyecto para siempre.

| Hito | Contenido | Condición de salida | Aprox. |
|---|---|---|---|
| **0. Fundaciones** | Workspace, ventana wgpu en Windows, Linux y macOS (CI), captura sin ventana, `hush-cli` vacío, `cargo-deny`, `AGENTS.md` | Claude genera y lee una captura de un cubo texturizado; CI en verde en las tres plataformas | 2–3 sem |
| **1. Vóxel** | Chunks, mallas, luz, cámara, colisión, cavar y poner | Escenas de captura de túneles correctas; 64³ se regenera completo en menos de 50 ms | 3–4 sem |
| **2. Red** | Servidor, UDP y memoria, predicción, unión, deltas, bots básicos | Dos clientes y seis bots consistentes con 150 ms de latencia y 2 % de pérdida | 3–4 sem |
| **3. Turno Cero** | Equipos, oro, banco, meta, cuatro clases, bloques clásicos, muerte | **Primera partida tipo Infiniminer completa con amigos por LAN** | 3–4 sem |
| **4. El Durmiente** | Sismógrafo, umbrales, lava, cangilones, geófono, niebla, ofrendas, chapas, crónica | **Prueba decisiva** con grupos externos (ver abajo) | 5–6 sem |
| **5. Web y en línea** | Versión WebGPU + WebTransport, relé de Steam, servidor dedicado, Steam Deck | Alguien entra a una partida desde un enlace del navegador; 8 contra 8 por Internet | 4–5 sem |
| **6. Alfa pública** | Arte de 32×32, audio, Vigía, interfaz, español e inglés, modo Taller | Demo web en itch.io y Steam Playtest; la gente vuelve a jugar | 6–8 sem |
| **7. Acceso anticipado** | Equilibrado, temporada 1, servidor comunitario documentado | Lanzamiento en Steam; publicación de Grisú como código abierto | — |

**La prueba decisiva del hito 4 consiste en observar si los jugadores, sin que nadie se lo pida:**

- desvían túneles para proteger sus líneas;
- abandonan una veta para atacar la del rival;
- discuten si ofrendar oro o seguir haciendo ruido;
- se arriesgan por el Corazón.

Si no aparecen esas decisiones, **se rediseña antes de pasar al hito 5**.

**Mundo grande (W0–W7):** el plan está en `docs/mundo/` y las decisiones, en `docs/adr/0002-mundo-grande.md`. Antes de la 1.0 solo entra **W0**, dentro del hito 1 (2–4 días): posiciones de entidad en `i64` Q47.16, chunks detrás de una interfaz, coordenadas negativas probadas y render relativo a la cámara. Los hitos W1–W7 llegan después del acceso anticipado y solo si el modo Turno tiene tracción.

> **Pendiente (24 de octubre de 2026): W0 no se hizo dentro del hito 1.** Los hitos 1 y 2 se programaron en paralelo a este plan y usan un mundo finito: coordenadas de bloque `i32` positivas, un máximo de 128 bloques por eje (`grisu_voxel::MAX_WORLD_SIZE`), posiciones en punto fijo `i32` con 1024 unidades por bloque (`FIXED_ONE`), chunks guardados directamente en `VoxelWorld` y cámara en coordenadas absolutas `f32`. Basta para el modo Turno (64³–96×96×64), pero no para el mundo grande.
>
> **Qué cambiaría al hacer W0** (2–4 días, mejor **antes del hito 3**, cuando aún hay poco código encima):
>
> - `PlayerBody`, `Aabb` y `move_aabb` pasan de `i32` con `FIXED_ONE` a `i64` Q47.16, y el protocolo de `hush-net` envía posiciones de 64 bits.
> - `VoxelWorld` queda detrás de una interfaz (`get`/`set`/`fill`) con `ChunkPos` que admite negativos; el mapa del Turno es una implementación finita de esa interfaz.
> - `raycast`, colisión, luz y mallas se prueban en coordenadas negativas y en ±7 499 999.
> - El render pasa a ser relativo a la cámara (las caras empaquetadas ya guardan coordenadas locales de 7 bits; falta el origen por chunk).
> - La generación de terreno de `hush-sim` toma la forma de función pura `(semilla, ChunkPos) → Chunk`.
>
> Criterio de salida, el de `docs/mundo/04-verificacion-e-hitos.md`: pruebas de chunk, DDA y colisión en coordenadas negativas y extremas en verde, y el mapa de 64³ sin cambios visibles (mismas capturas, `netsim` consistente).

**Gráficos:** el plan de cámaras (primera y tercera persona), animación por piezas, texturas de 32×32 y shaders propios está en `docs/graficos/` y las decisiones, en `docs/adr/0003-graficos.md`. En el hito 1 solo entra **G0** (2–3 días): tamaño de textura leído del manifiesto, luz de bloque en color, preprocesador de shaders y vistas de depuración. El resto se reparte entre los hitos 3 y 6 (unas 6–8 semanas en total).

---

## 6. Riesgos

| Riesgo | Mitigación |
|---|---|
| El motor se come el juego (el «síndrome del motor») | El motor solo hace lo que HUSHMINE necesita; nada genérico hasta el acceso anticipado; cada hito termina en algo jugable |
| La IA genera Rust que no entiendes | Crates pequeños, pruebas e invariantes, capturas, `clippy` estricto y documentación de cada módulo en `docs/` |
| Cambios de API de wgpu o winit | Versiones fijadas y actualización en tareas dedicadas, una vez por hito |
| Rendimiento o compatibilidad en web | La web llega en el hito 5, con WebGL2 de respaldo; si fallara, la demo web se retrasa sin bloquear el juego |
| Pocos jugadores | Bots, partidas desde 1 contra 1, demo de un clic en el navegador, relé de Steam |
| Trampas | Servidor autoritativo, niebla de mineral por equipo, simulación solo en el servidor |
| Consolas o móviles con presupuesto | Solo entonces se evalúa un port; el diseño de `hush-sim` es la especificación que se reutiliza |

---

## 7. Fuera de la 1.0

Consolas, móviles, mundo grande y supervivencia (planificados en `docs/mundo/`; solo W0 entra antes), fabricación, progresión que dé ventaja, mods en WebAssembly (llegan después), modo Asedio, La Mina Eterna y criaturas (los Silbadores).
