# Prompts para el orquestador (Gemini) · ronda 2

Segunda tanda de cinco encargos, con las mismas reglas que la [ronda 1](gemini-orquestador.md):

- al menos 4 subagentes cada uno;
- nada que compile wgpu ni winit;
- cada encargo escribe solo en sus carpetas;
- se pueden lanzar en cualquier orden, cada uno en una sesión nueva.

**Cómo usarlos:** pega el **bloque común de la ronda 2** y, debajo, uno de los cinco prompts.

La diferencia con la ronda 1 es que el trabajo de esa ronda ya está en `main` y ahora es **la fuente de verdad**: historia, diseño, datos, texturas y documentación. Estos encargos lo leen pero no lo modifican.

---

## Bloque común de la ronda 2 (pegar siempre antes del prompt)

```text
Eres el ORQUESTADOR de un equipo de subagentes que trabaja en el repositorio
de HUSHMINE, un juego competitivo de minería en vóxeles (sucesor espiritual
de Infiniminer), con un motor propio llamado Grisú en Rust + wgpu.

══════════ FUENTES (léelas tú; pasa a cada subagente solo lo necesario) ══════════
- AGENTS.md           → reglas del repositorio (obligatorias).
- PLAN_v4.md          → plan completo. PLAN.md, _v2 y _v3 están obsoletos.
- docs/README.md      → índice de toda la documentación.
- docs/design/        → reglas en enteros: Sismógrafo, clases, economía,
                        bloques, invariantes. data/*.ron tiene los valores.
- docs/lore/          → historia: los 41, el Vigía, Anima Consolidated.
- docs/art/ y tools/textures/palette.py → paleta y guía de arte.
- assets/textures/blocks/manifest.ron   → texturas de bloques existentes.
- crates/hush-sim/    → simulación: punto fijo, bloques, Sismógrafo, altar
                        y registro de eventos.

══════════ INDEPENDENCIA ENTRE ENCARGOS ══════════
Este encargo es uno de cinco que pueden ejecutarse en cualquier orden.
- Escribe SOLO en las carpetas de tu encargo (tabla de abajo).
- Todo lo que ya existe fuera de ellas es de SOLO LECTURA. Si encuentras
  un error ahí, NO lo corrijas: anótalo en tu resumen final.
- Puedes IMPORTAR (sin modificar) los módulos de Python ya existentes,
  por ejemplo tools/textures/png.py y tools/textures/palette.py.

  Encargo                   | Carpetas propias
  --------------------------|------------------------------------------------
  6 Localización            | assets/i18n/, tools/i18n/, docs/i18n/
  7 Sonido                  | assets/audio/, tools/audio/, docs/audio/
  8 Mapas y escenas         | tools/mapgen/, docs/mapgen/, tests/scenes/
  9 Economía en hush-sim    | crates/hush-sim/ (y nada más)
 10 Interfaz y marca        | assets/ui/, tools/ui/, docs/ui/

══════════ CONVENCIONES (vienen de la ronda 1; no se cambian) ══════════
A) Ids de bloque u8, en snake_case:
     0 air, 1 dirt, 2 rock, 3 hard_rock, 4 ore, 5 gold, 6 diamond, 7 lava,
     8 team_block, 9 force_field, 10 ladder, 11 jump_pad, 12 electric,
     13 rail, 14 beacon, 15 explosive, 16 bucket_station, 17 bucket_post.
   El equipo va en los metadatos del bloque, no en el id.
B) 30 ticks por segundo. Sismógrafo en u32, de 0 a 100_000; umbrales en
   25_000, 50_000, 75_000 y 100_000.
C) Punto fijo Q16.16. Dinero en u32. Multiplicadores como fracción entera.
D) Los valores numéricos están en data/*.ron. Si un documento contradice
   a data/, manda data/.
E) Glosario de PLAN_v4.md: the Sleeper = el Durmiente, Shift = Turno,
   Brass tally = Ficha de latón, Mother Lode = Veta Madre, etc.
F) Equipos: Copper (naranja) y Cobalt (azul), con los colores de
   tools/textures/palette.py.

══════════ RESTRICCIONES DE LA MÁQUINA ══════════
- El PC se traba con facilidad. PROHIBIDO: `cargo build`,
  `cargo clippy --workspace`, `cargo nextest run --workspace`,
  `cargo run` de cualquier paquete, y cualquier comando que compile
  grisu-render, grisu-platform, hush-client o hush-cli.
- Cargo solo si el prompt lo permite, de uno en uno y solo el orquestador.
- Python 3 con la biblioteca estándar únicamente. Nada de pip install ni de
  descargas de internet.

══════════ REGLAS DEL REPOSITORIO ══════════
- Identificadores en inglés. Comentarios y documentación en español. El
  texto visible del juego, primero en inglés.
- Máximo 400 líneas por archivo, también en Markdown y Python.
- En hush-sim: nada de f32/f64 en el estado, nada de aleatoriedad del
  sistema, nada de HashMap iterado, nada de panic!/unwrap/expect fuera de
  las pruebas, nada de `unsafe`. No añadas dependencias de Rust.
- Todo contenido es ORIGINAL y propio del proyecto.
- Los archivos de texto usan saltos de línea LF (lo fija .gitattributes).
  Si un script compara archivos byte a byte, que escriba en LF.

══════════ PROTOCOLO ══════════
INICIO
  1. Ejecuta `git status`. Si hay cambios sin confirmar, PARA y avisa.
  2. `git switch main`, `git pull` y `git switch -c gemini/<tema>`.
  3. Lee las fuentes y escribe un plan corto: qué hace cada subagente, qué
     archivos crea y en qué orden. DOS SUBAGENTES NUNCA ESCRIBEN EL MISMO
     ARCHIVO.
PASOS
  4. Encargos autocontenidos: contexto mínimo, archivos exactos, formato y
     criterio de terminado. Copia en cada encargo las convenciones que le
     afecten.
  5. En paralelo los independientes; en serie los que dependen de otro.
  6. El último subagente es un REVISOR que no crea contenido y devuelve una
     lista de problemas. Tú los corriges.
  7. Ejecuta las PRUEBAS del prompt. Como máximo 3 rondas de corrección; si
     siguen fallando, para y entrega el error exacto.
CIERRE
  8. `git add` solo de tus carpetas y un commit con el mensaje en español.
     NO hagas push.
  9. Resumen final con:
     - los archivos creados;
     - la salida de las pruebas;
     - las decisiones abiertas;
     - los errores encontrados en archivos ajenos.
```

---

## Prompt 6 · Localización con Fluent (inglés y español)

```text
ENCARGO 6 · rama: gemini/i18n · carpetas: assets/i18n/, tools/i18n/, docs/i18n/

OBJETIVO
Crear los archivos de idioma del juego con Project Fluent (.ftl):
- inglés como idioma fuente;
- español de Latinoamérica (es-419) y de España (es-ES) como primeras
  traducciones.
Reúne todo el texto visible que ya existe repartido en docs/:
- las frases del Vigía en docs/lore/vigia-radio.md;
- los carteles en docs/lore/anima-consolidated.md;
- clases, bloques y umbrales en docs/design/ y data/*.ron;
- el texto de la tienda en docs/marketing/steam-page.md.

INICIO
- Fija tú el esquema de identificadores, por ejemplo block-gold,
  class-prospector, seismo-stir, watcher-stir-1, hud-bank, menu-play, y la
  lista de archivos .ftl. Reutiliza los ids que ya usa vigia-radio.md.

PASOS (mínimo 5 subagentes)
1. INVENTARIO → docs/i18n/inventario.md
   Tabla de todas las cadenas que necesitará la 1.0: id, contexto (dónde se
   ve), longitud máxima y si lleva variables. Incluye el HUD, los menús, la
   Crónica (títulos Despertador, Guardián del sueño, Topo, Ladrón de
   cangilones), las muertes y los avisos del altar (los motivos de
   AltarError en crates/hush-sim/src/altar.rs).
2. INGLÉS FUENTE → assets/i18n/en/*.ftl
   game.ftl, hud.ftl, watcher.ftl, signs.ftl y chronicle.ftl. Usa variables
   y plurales de Fluent donde haga falta, por ejemplo
   { $count -> [one] … *[other] … }.
3. ESPAÑOL LATINOAMÉRICA → assets/i18n/es-419/*.ftl
4. ESPAÑOL ESPAÑA → assets/i18n/es-ES/*.ftl
   Mismos archivos e ids que en inglés. Cambia solo lo que de verdad varía
   entre regiones (tú/vos/vosotros, léxico) y documéntalo.
5. GUÍA DE ESTILO → docs/i18n/guia-estilo.md
   Tono de cada voz (Vigía, Compañía, interfaz), glosario obligatorio,
   reglas de longitud y lo que NO se traduce (nombres propios, HUSHMINE).
   Incluye una sección para los futuros traductores al portugués de Brasil,
   chino, ruso, alemán, francés y japonés: qué cadenas son delicadas y por
   qué.
6. VERIFICADOR → tools/i18n/check_ftl.py y tools/i18n/test_i18n.py
   Parser mínimo del subconjunto de Fluent que se use.
7. REVISOR (al final).

RESULTADO ESPERADO
Tres carpetas de idioma con los mismos archivos e ids, inventario, guía de
estilo y verificador.

PRUEBAS
`python -m unittest discover -s tools/i18n -v` en verde y
`python tools/i18n/check_ftl.py` con código 0. Deben comprobar:
- los tres idiomas tienen exactamente el mismo conjunto de ids;
- cada mensaje usa las mismas variables ($nombre) en los tres idiomas;
- todo selector de plural tiene variante por defecto (*[other]);
- ningún id está duplicado y todos siguen el esquema de nombres;
- cada fila de vigia-radio.md tiene su id en watcher.ftl;
- las frases del Vigía en inglés tienen 6 palabras como máximo;
- el glosario se respeta en español (por ejemplo, "Durmiente" y nunca
  "Dormido");
- ningún archivo pasa de 400 líneas.
NO ejecutes cargo.
```

---

## Prompt 7 · Sonido procedural

```text
ENCARGO 7 · rama: gemini/sonido · carpetas: assets/audio/, tools/audio/, docs/audio/

OBJETIVO
Crear el primer juego de efectos de sonido. Se generan con Python (módulos
wave, struct y math de la biblioteca estándar), así que son reproducibles y
del proyecto. Formato: WAV PCM de 16 bits, 44_100 Hz, mono.
El sonido es una mecánica: el ruido despierta al Durmiente, el geófono deja
oír excavaciones a través de la roca y los Silbadores avisan de los
derrumbes. Lee PLAN_v4.md (secciones 2 y 3) y docs/lore/.

INICIO
- Fija la lista de sonidos con su id en snake_case y su categoría:
  herramientas, bloques, Sismógrafo, ambiente, radio e interfaz.

PASOS (mínimo 5 subagentes)
1. DIRECTOR DE SONIDO → docs/audio/guia-sonido.md
   Identidad sonora: mina de 1979, radio de onda corta y casetes. Lista de
   sonidos con duración, volumen relativo, si es posicional y qué mecánica
   comunica. Reglas de mezcla para el futuro motor (kira): prioridades y
   cuántas voces simultáneas.
2. SINTETIZADOR BASE → tools/audio/synth.py
   Osciladores, ruido determinista (LCG con semilla, nunca random sin
   semilla), envolventes ADSR, filtros paso bajo y paso alto de un polo,
   mezcla con recorte suave y escritura de WAV.
3. HERRAMIENTAS Y BLOQUES → tools/audio/sfx_tools.py
   pick_hit_rock, pick_hit_dirt, block_break, block_place, explosion,
   electric_zap, lava_bubble, bucket_line_loop (que se pueda repetir en
   bucle sin chasquido) y footsteps (4 variantes).
4. SISMÓGRAFO Y AMBIENTE → tools/audio/sfx_world.py
   seismo_tick, seismo_stir, seismo_breathe, seismo_restless, wake_rumble,
   whistlers (silbidos inquietantes y originales, 3 variantes),
   geophone_dig (un picado oído a través de la roca: filtrado y lejano),
   offering_accept, tally_return y mine_ambience_loop.
5. RADIO E INTERFAZ → tools/audio/sfx_radio.py
   radio_click_on, radio_click_off, radio_static_loop, ui_select, ui_back
   y ui_error.
6. INTEGRADOR → tools/audio/build_all.py y assets/audio/manifest.ron
   Genera todos los WAV en assets/audio/sfx/, el manifiesto (id, archivo,
   duración en muestras, si es un bucle, categoría) y
   assets/audio/LICENSE.md. `--check` regenera en memoria y compara byte a
   byte con lo que hay en disco.
7. PRUEBAS → tools/audio/test_audio.py
8. REVISOR (al final): coherencia entre la guía y el manifiesto, nombres,
   duraciones y licencias.

RESULTADO ESPERADO
Unos 30 WAV, el manifiesto, la guía y los scripts.
La carpeta assets/audio/ no pasa de 8 MB.

PRUEBAS
`python -m unittest discover -s tools/audio -v` en verde y
`python tools/audio/build_all.py --check` con código 0. Deben comprobar:
- cada WAV es PCM de 16 bits, 44_100 Hz y mono, y se lee con el módulo
  wave;
- ningún archivo satura: el pico es menor o igual que 0,98 de la escala
  completa y no está todo en silencio;
- los bucles (*_loop) empiezan y terminan cerca de cero y la diferencia
  entre la última muestra y la primera es pequeña;
- la generación es determinista: dos ejecuciones producen los mismos bytes;
- el manifiesto y los archivos coinciden uno a uno;
- el tamaño total es de 8 MB como máximo.
NO ejecutes cargo.
```

---

## Prompt 8 · Generador de mapas y escenas de captura

```text
ENCARGO 8 · rama: gemini/mapas · carpetas: tools/mapgen/, docs/mapgen/, tests/scenes/

OBJETIVO
Prototipar en Python el generador de mapas que luego se pasará a Rust, y
escribir las escenas de captura que pide PLAN_v4.md (por ejemplo,
tests/scenes/tunel_lava.ron). Mapa de 64×64×64, o de 96×96×64 según
data/economy.ron. Debe ser determinista: la misma semilla da el mismo
mapa, usando solo aritmética entera, para que el port a Rust sea idéntico.

INICIO
- Lee data/blocks.ron, data/economy.ron, docs/design/ y la sección 4 de
  PLAN_v4.md (chunks de 16³, 1 byte por bloque).
- Fija el generador de números: un PCG o un xorshift de 64 bits, en enteros
  y con la semilla como entrada. Documenta que el port a Rust debe usar el
  mismo algoritmo.

PASOS (mínimo 5 subagentes)
1. DISEÑADOR DE GEOLOGÍA → docs/mapgen/geologia.md
   Capas por profundidad (tierra, roca, roca dura), vetas de mena, oro y
   diamante, bolsas de lava, cuevas, bases de los equipos en lados
   opuestos, simetría o equidad entre los dos equipos, y dónde puede
   aparecer la Veta Madre al llegar a Respira. Valores objetivo: cuánto oro
   hay por mapa en relación con la meta de dinero de data/economy.ron.
2. GENERADOR → tools/mapgen/rng.py y tools/mapgen/generate.py
   Genera un volumen de bytes con los ids de la convención A. Solo enteros
   en el núcleo; nada de floats en las decisiones.
3. ANALIZADOR → tools/mapgen/analyze.py
   Recuento de bloques por tipo, oro alcanzable desde cada base, diferencia
   de oro entre equipos y volumen de lava. Salida en JSON y en Markdown.
4. VISUALIZADOR → tools/mapgen/render_slices.py
   Cortes horizontales y verticales en PNG, con un color por bloque sacado
   de palette.py, y un mapa de alturas visto desde arriba. Importa
   tools/textures/png.py sin modificarlo. Salida en out/mapgen/.
5. ESCENAS DE CAPTURA → tests/scenes/*.ron y docs/mapgen/escenas.md
   Entre 6 y 8 escenas pequeñas y fijas para el futuro
   `hush-cli shot --scene`: tunel_lava, veta_oro, base_copper, base_cobalt,
   campo_fuerza, cangilones y cueva_oscura. Cada una indica bloques (por
   rangos o listas), posición y orientación de la cámara, y una
   descripción en español de lo que debe verse en la captura (será el
   criterio de revisión).
6. PRUEBAS → tools/mapgen/test_mapgen.py
7. REVISOR (al final): mira los PNG de out/mapgen/ y comprueba que el mapa
   es jugable y equitativo.

RESULTADO ESPERADO
Generador, analizador, visualizador, documento de geología, escenas .ron y
un informe de 20 semillas en docs/mapgen/informe-semillas.md.

PRUEBAS
`python -m unittest discover -s tools/mapgen -v` en verde. Deben
comprobar:
- la misma semilla produce el mismo hash SHA-256 del volumen, y semillas
  distintas producen mapas distintos;
- todo byte del volumen es un id válido (0–17);
- en 20 semillas, cada equipo tiene oro alcanzable de al menos 1,5 veces
  la meta, y la diferencia entre equipos no pasa del 10 %;
- ninguna base tiene lava a menos de 6 bloques;
- las escenas .ron se leen (importa tools/balance/ron_lite.py sin
  modificarlo), solo usan ids válidos y tienen cámara y descripción.
Además, `python tools/mapgen/generate.py --seed 42 --report` termina con
código 0. NO ejecutes cargo.
```

---

## Prompt 9 · Economía e invariantes en hush-sim

```text
ENCARGO 9 · rama: gemini/economia · carpetas: crates/hush-sim/ (y nada más)

OBJETIVO
Llevar a hush-sim la economía de docs/design/clases-economia.md y
data/economy.ron, y rellenar `broken_invariants` en SimReport con las
invariantes de docs/design/invariantes.md. Hoy la simulación ya tiene
punto fijo, bloques, Sismógrafo, altar y registro de eventos.
No hay crate de RON (no se añaden dependencias): los valores van como
constantes con el comentario "// Se moverá a data/*.ron", y una prueba
vigila que coincidan con los archivos de data/.

INICIO
- Lee crates/hush-sim/src/ completo e imita su estilo: documentación en
  español, `mod tests` al final y Result en vez de panic.
- Ejecuta tú una vez `cargo test -p hush-sim -j 2` para confirmar que la
  base está en verde.

PASOS (mínimo 5 subagentes; cada uno es dueño de UN archivo nuevo)
1. ECONOMY → crates/hush-sim/src/economy.rs
   Banco por equipo (u32), carga de cada jugador (mena, oro y diamante),
   depósito en el banco, meta según el tamaño de equipo, Fama al pasar el
   60 % de la meta, botín robado ×5/4 con redondeo documentado, y victoria
   instantánea al llegar a la meta. Todo con checked_* o saturating_*.
2. LOOT → crates/hush-sim/src/loot.rs
   Bolsas que se sueltan al morir: posición en enteros, contenido, equipo
   de origen, recogida por cualquiera (con el ×5/4 si la recoge el rival)
   y caducidad en loot_bag_lifetime_ticks. Guardadas en un Vec o un
   BTreeMap, nunca en un HashMap.
3. INVARIANTS → crates/hush-sim/src/invariants.rs
   Funciones puras que reciben el estado y devuelven `Vec<String>` con las
   invariantes rotas: conservación del oro (solo cambia al minar, depositar,
   ofrendar o caducar una bolsa, y siempre con un evento), Sismógrafo
   dentro de [0, 100_000], umbrales emitidos una sola vez, ningún banco
   negativo ni desbordado. Cada invariante lleva el id de
   docs/design/invariantes.md (por ejemplo, INV-07).
4. DATA GUARD → crates/hush-sim/tests/data_matches_constants.rs
   Con include_str! de data/economy.ron y data/seismograph.ron, comprueba
   que cada constante de Rust aparece con el mismo valor en el .ron. Basta
   con buscar texto ("offering_reduction: 2_500"); no hace falta un parser.
   Si alguien cambia data/ sin cambiar el código, esta prueba falla.
5. INTEGRADOR (después de 1–3; es el ÚNICO que toca archivos existentes)
   → lib.rs, sim.rs, report.rs y events.rs
   - Simulation tiene la economía y las bolsas, y offer_gold descuenta el
     oro del banco del equipo.
   - Nuevos eventos: GoldMined, Deposited, FameGained, LootDropped,
     LootPicked, LootExpired y GoalReached.
   - Al final de run_to_end se ejecutan las invariantes y el resultado va a
     broken_invariants.
   - Si sim.rs pasara de 400 líneas, saca las acciones de los jugadores a
     un archivo nuevo (por ejemplo, actions.rs).
6. PRUEBAS DE INTEGRACIÓN → crates/hush-sim/tests/economy_flow.rs
   Una partida guionizada en la que se mina, se deposita, se muere, se roba
   una bolsa y se ofrenda, con el oro cuadrando al final.
7. REVISOR (al final): f32/f64 en el estado, HashMap, panic/unwrap fuera
   de las pruebas, archivos de más de 400 líneas, comentarios en inglés y
   contradicciones con data/.

RESULTADO ESPERADO
Economía, bolsas e invariantes funcionando en la simulación, con
broken_invariants rellenado de verdad y una prueba que avisa si data/ y el
código se separan. hush-cli no se toca: `hush_sim::run(config)` y
`report.broken_invariants` deben seguir existiendo con la misma forma.

PRUEBAS (mínimo)
- economy: la Fama se activa exactamente al 60 %; el botín ×5/4 de 100 da
  125; desbordar el banco devuelve error y no hace panic.
- loot: una bolsa caduca justo en el tick que toca; la recoge una sola
  persona; si la recoge el rival se aplica el ×5/4.
- invariants: un estado manipulado a mano rompe la invariante esperada y
  devuelve su id.
- data guard: todas las constantes coinciden con data/.
- integración: la partida guionizada termina con broken_invariants vacío,
  y una partida simulada por defecto también.

COMANDOS PERMITIDOS: solo el orquestador, de uno en uno, al final:
    cargo fmt -p hush-sim
    cargo clippy -p hush-sim --all-targets -j 2 -- -D warnings
    cargo test -p hush-sim -j 2
NO ejecutes nada que compile otros crates.
```

---

## Prompt 10 · Interfaz, HUD e identidad visual

```text
ENCARGO 10 · rama: gemini/interfaz · carpetas: assets/ui/, tools/ui/, docs/ui/

OBJETIVO
Diseñar la interfaz del juego y la identidad visual:
- maquetas del HUD y de los menús;
- iconos;
- logotipo de HUSHMINE;
- bocetos de la cápsula de Steam.
Todo en SVG escrito a mano o generado con Python (biblioteca estándar), y
los iconos pixel art en PNG. Estética de 1979: pantallas de fósforo verde,
chapas de latón, sodio y carteles de Anima Consolidated. Muy poco texto, un
icono para cada cosa y legible en un monitor de 1080p y en la Steam Deck
(1280×800).

INICIO
- Lee la paleta en tools/textures/palette.py y docs/art/guia-texturas.md.
  Puedes AÑADIR colores de interfaz en tu propio archivo, nunca cambiar los
  de la paleta.
- Fija la lista de pantallas: HUD en partida, elección de clase en la
  Bajada, Crónica posterior, menú principal y pantalla de carga.

PASOS (mínimo 5 subagentes)
1. DIRECTOR DE UI → docs/ui/guia-interfaz.md y tools/ui/ui_palette.py
   Jerarquía de la información en partida: Sismógrafo siempre visible,
   banco de los dos equipos, carga, clase, chapas encontradas y avisos del
   Vigía. Tamaños mínimos para 1280×800, márgenes seguros y accesibilidad
   (no depender solo del color).
2. ICONOS → tools/ui/icons.py → assets/ui/icons/*.png (16×16 y 32×32)
   Cuatro clases, mena, oro, diamante, banco, ficha de latón, ofrenda,
   explosivo, geófono, Fama, las cuatro fases del Sismógrafo, muerte por
   lava, electricidad, caída y derrumbe. Importa tools/textures/png.py sin
   modificarlo.
3. MAQUETAS → tools/ui/mockups.py → assets/ui/mockups/*.svg (1920×1080 y
   1280×800)
   HUD, Bajada, Crónica (con los títulos Despertador, Guardián del sueño,
   Topo y Ladrón de cangilones), menú principal y carga. Texto en inglés
   con los ids de Fluent como comentario dentro del SVG.
4. MARCA → assets/ui/brand/*.svg y docs/ui/marca.md
   Tres propuestas de logotipo de HUSHMINE, un símbolo cuadrado para el
   icono de la aplicación y tres bocetos de cápsula de Steam (460×215 y
   616×353) que muestren el sabotaje: alguien corta la línea rival mientras
   el sismógrafo se dispara. Tipografía: solo formas dibujadas o fuentes
   con licencia OFL, y anota cuál. Nada de imitar logotipos existentes.
5. EXPORTADOR → tools/ui/build_all.py
   Genera todo y la hoja de revisión out/ui_contact.png con los iconos
   ampliados. `--check` regenera en memoria y compara con lo que hay en
   disco.
6. PRUEBAS → tools/ui/test_ui.py
7. REVISOR (al final): abre out/ui_contact.png y los SVG, y revisa la
   legibilidad a 16×16, la coherencia con las texturas y la jerarquía del
   HUD.

RESULTADO ESPERADO
Guía de interfaz, unos 25 iconos en dos tamaños, maquetas SVG de 5
pantallas en 2 resoluciones, propuestas de marca, assets/ui/LICENSE.md y
scripts.

PRUEBAS
`python -m unittest discover -s tools/ui -v` en verde y
`python tools/ui/build_all.py --check` con código 0. Deben comprobar:
- todos los SVG son XML válido (xml.etree), tienen viewBox con el tamaño
  esperado y no cargan recursos externos (ningún href http);
- los iconos PNG miden exactamente 16×16 o 32×32 y cada icono existe en
  los dos tamaños;
- los colores de los iconos están en palette.py o en ui_palette.py;
- en las maquetas, el texto sobre su fondo tiene un contraste WCAG de al
  menos 4,5:1;
- la generación es determinista;
- ningún archivo de texto pasa de 400 líneas.
NO ejecutes cargo.
```
