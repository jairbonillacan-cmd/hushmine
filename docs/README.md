# Documentación de HUSHMINE y el motor Grisú

Índice general de la carpeta `docs/` del repositorio. Esta documentación abarca la arquitectura técnica del motor propio **Grisú** y del juego **HUSHMINE** (tal como se encuentran en la rama principal tras el Hito 0), la planificación del Hito 1, las decisiones de arquitectura (ADR) y el material de comunicación y tienda.

---

## 1. Decisiones de arquitectura (`docs/adr/`)

Registro de decisiones técnicas clave del proyecto:

- [ADR 0001: motor propio en Rust + wgpu](adr/0001-motor-propio-rust-wgpu.md) (`docs/adr/0001-motor-propio-rust-wgpu.md`) — Contexto, alternativas evaluadas y motivos para construir Grisú con Rust 2024, `wgpu`, `winit` y `glam`.
- [ADR 0002: preparar el motor para un mundo de 15 M × 15 M bloques](adr/0002-mundo-grande.md) (`docs/adr/0002-mundo-grande.md`) — Posiciones en `i64`, chunks detrás de una interfaz, render relativo a la cámara y generación como función pura (propuesta).

---

## 2. Motor y arquitectura técnica (`docs/engine/`)

Guías pensadas para comprender cada módulo del repositorio en `crates/` sin necesidad de dominar previamente Rust ni programación gráfica:

- [Renderizador: `grisu-render`](engine/grisu-render.md) (`docs/engine/grisu-render.md`) — Inicialización de `Gpu`, cámara orbital `OrbitCamera`, búfer de profundidad `DepthTarget`, textura procedural `TextureData`, pipeline de `CubeRenderer`, shader WGSL y captura sin ventana con `render_to_image()`.
- [Plataforma y ventana: `grisu-platform`](engine/grisu-platform.md) (`docs/engine/grisu-platform.md`) — Bucle de eventos, separación entre ventana y juego mediante el trait `App` y recorrido de un fotograma desde el sistema operativo hasta la pantalla.
- [Simulación determinista: `hush-sim`](engine/hush-sim.md) (`docs/engine/hush-sim.md`) — Reloj a 30 Hz (`TICK_RATE`), generador `Rng`, huella `state_hash`, métricas del informe JSON (`SimReport`), reglas de determinismo y guía práctica *«Cómo leer un error de Rust»* (préstamos, tipos y `Result`).
- [Herramientas de terminal: `hush-cli`](engine/hush-cli.md) (`docs/engine/hush-cli.md`) — Funcionamiento de los subcomandos `shot`, `sim` y `gpu` para que personas y agentes de IA verifiquen el proyecto sin abrir ventana.
- [Cliente del juego: `hush-client`](engine/hush-client.md) (`docs/engine/hush-client.md`) — Binario `hushmine`, demostración del Hito 0 (`CubeDemo`) y frontera entre presentación (`grisu-*`) y reglas (`hush-sim`).

---

## 3. Planificación de hitos (`docs/hitos/`)

Desglose de tareas, validación y dependencias por hito de desarrollo:

- [Hito 1: Vóxel (`hito-1-voxel.md`)](hitos/hito-1-voxel.md) (`docs/hitos/hito-1-voxel.md`) — Hoja de ruta en 16 tareas para chunks de 16³, mallas en CPU, iluminación y oclusión ambiental, cámara en primera persona, colisión en punto fijo, raycast DDA, excavar y colocar bloques, diferenciando tareas de compilación ligera y pesada.

---

### Mundo grande (`docs/mundo/`)

Plan para un futuro modo Supervivencia / Historia en un mundo procedural de 15 millones × 15 millones de bloques:

- [Índice y resumen (`README.md`)](mundo/README.md) — Qué cambia, cifras clave y nuevos crates.
- [Escala y motor (`01-escala-y-motor.md`)](mundo/01-escala-y-motor.md) — Coordenadas, chunks con paleta, streaming, luz, render, guardado y red.
- [Generación (`02-generacion.md`)](mundo/02-generacion.md) — Ruido entero, relieve, biomas, cuevas, acuíferos y menas.
- [Estructuras (`03-estructuras.md`)](mundo/03-estructuras.md) — Colocación sin estado, minas de Anima Consolidated, pueblos y capa de historia.
- [Verificación e hitos (`04-verificacion-e-hitos.md`)](mundo/04-verificacion-e-hitos.md) — Herramientas, invariantes, hitos W0–W7 y riesgos.

### Modo Historia (supervivencia) (`docs/supervivencia/`)

Diseño y lore del modo abierto que vive en el mundo grande, todo en estado de propuesta. Ocurre en el Silencio, un mundo sin fechas que no es la Tierra:

- [Índice, pilares y glosario (`README.md`)](supervivencia/README.md) — Qué es el modo, mundos únicos por semilla y términos nuevos.
- [Lore (`01-lore.md`)](supervivencia/01-lore.md) — El cielo-Párpado, los Durmientes, los Llegados y sus fichas, pueblos, facciones y el Insomne.
- [Historia (`02-historia.md`)](supervivencia/02-historia.md) — Prólogo, cinco arcos, jefes, tres finales y qué cambia con la semilla.
- [Pueblos y comercio (`03-pueblos-y-comercio.md`)](supervivencia/03-pueblos-y-comercio.md) — Lampisterías, oficios de los NPC, los Quedos, precios, encargos, reputación y pueblos propios.
- [Criaturas y jefes (`04-criaturas-y-jefes.md`)](supervivencia/04-criaturas-y-jefes.md) — Fauna, criaturas de la montaña y de cada dimensión, guardianes y combate por ruido y luz.
- [Tecnologías (`05-tecnologias.md`)](supervivencia/05-tecnologias.md) — Seis ramas, materiales, herramientas, armas de minero, máquinas y automatización.
- [Dimensiones (`06-dimensiones.md`)](supervivencia/06-dimensiones.md) — El Sueño, el Hervor, el Eco y el Revés, con su lore, sus reglas y su implementación.

---

## 4. Comunicación, tienda y prensa (`docs/marketing/`)

Material bilingüe (inglés como fuente y español) para preparar el anuncio público y la página de tienda:

- [Página de tienda en Steam (`steam-page.md`)](marketing/steam-page.md) (`docs/marketing/steam-page.md`) — Lema oficial (*"Rob a sleeping mountain. Play dirty. Keep it hushed."*), descripción corta ($\le 300$ caracteres), descripción larga con la premisa de 1931/1979, 6 viñetas, 5 ideas de captura, concepto de cápsula centrado en el sabotaje y etiquetas.
- [Kit de prensa (`press-kit.md`)](marketing/press-kit.md) (`docs/marketing/press-kit.md`) — Ficha técnica bilingüe, sinopsis, historia del proyecto, activos de prensa y lista de comprobación previa al anuncio (USPTO, EUIPO, Steam, itch.io, dominios y redes).
- [Devlog #01: Hito 0 y el nacimiento de Grisú (`devlog-01.md`)](marketing/devlog-01.md) (`docs/marketing/devlog-01.md`) — Artículo de presentación en tono cercano explicando por qué construir un motor propio, qué hace hoy Grisú y qué llega en el Hito 1.

---

## 5. Otras áreas de documentación (otros encargos)

Carpetas correspondientes a los demás encargos independientes del repositorio:

- `lore/` — Biblia narrativa del monte Anima: cronología 1931–1979, los 41 mineros y sus fichas de latón, radio del Vigía, memorandos de Anima Consolidated y folclore minero (pendiente).
- `art/` — Dirección de arte, paleta cerrada de colores y guía de texturas pixel art de 16×16 para bloques naturales y de construcción (pendiente).
- `design/` — Especificación de reglas en punto fijo y datos RON: Sismógrafo, clases, economía, bloques, ritmo de partida e invariantes (pendiente).
- [Prompts de orquestación (`prompts/gemini-orquestador.md`)](prompts/gemini-orquestador.md) — Guías de trabajo para los cinco encargos de subagentes (pendiente).

---

## 6. Verificación automática (`tools/docs/`)

La coherencia de esta documentación se valida con el script [`check_docs.py`](../tools/docs/check_docs.py) (`tools/docs/check_docs.py`), que comprueba la existencia de rutas citadas, identificadores del código en `crates/`, enlaces relativos, bloques Mermaid, longitud de la descripción corta de tienda, número de tareas del Hito 1 y el límite máximo de 400 líneas por archivo.
