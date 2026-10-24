# Inventario de Localización de HUSHMINE 1.0 (Project Fluent)

Catálogo maestro de las **232 claves inmutables** de localización para HUSHMINE 1.0 usando **Project Fluent** (`.ftl`), con `en` como idioma fuente canónico y soporte simétrico en `assets/i18n/{en,es-419,es-ES}/`.

## Arquitectura de archivos y convenciones

- **Estructura en `assets/i18n/{en,es-419,es-ES}/`:**
  - `game.ftl` (71 claves): marca, tienda, cuadrillas (`Copper Crew` naranja / `Cobalt Crew` azul), modos, 4 clases, 9 herramientas, los 18 bloques (`0..=17`) y el glosario del Sismógrafo (`0..=100_000` a `30 ticks/s`).
  - `hud.ftl` (47 claves): menús, indicadores en partida (dinero en `u32`), errores deterministas de `AltarError` (`crates/hush-sim/src/altar.rs`) y avisos de bajas sin armas de fuego.
  - `watcher.ftl` (50 claves): las 50 transmisiones por radio del Vigía (`docs/lore/vigia-radio.md`), limitadas a un máximo de 6 palabras en inglés y 48 caracteres.
  - `signs.ftl` (36 claves): los 10 carteles de seguridad esmaltados, los 6 memorandos internos (1928–1931) de `docs/lore/anima-consolidated.md` y el tablero de las 41 fichas de latón.
  - `chronicle.ftl` (28 claves): gaceta de fin de Turno (*Shift Chronicle*), hitos del `EventLog` y los 4 títulos honoríficos (*Despertador*, *Guardián del sueño*, *Topo*, *Ladrón de cangilones*).
- **Identificadores y formato:** todas las claves emplean `kebab-case` (`<prefijo>-<slug>`), nunca se concatenan en código y exponen selectores CLDR (`[one]` / `*[other]`) cuando dependen de cantidades enteras.

## 1. `game.ftl` — Identidad, equipos, clases, bloques y mundo (71 claves)

| ID | Archivo | Contexto | Máx. car. | Variables / Plural |
|---|---|---|---|---|
| `game-title` | `game.ftl` | Cabecera principal y título oficial del juego | 16 | `-` |
| `store-tagline` | `game.ftl` | Lema corto de portada y tienda | 60 | `-` |
| `store-short-desc` | `game.ftl` | Descripción corta de tienda (Steam <= 300 car.) | 300 | `-` |
| `store-premise-1931` | `game.ftl` | Premisa histórica: desastre del 14-XI-1931 (41 fichas) | 320 | `-` |
| `store-premise-1979` | `game.ftl` | Premisa histórica: regreso clandestino en 1979 | 260 | `-` |
| `store-premise-rule` | `game.ftl` | Regla de los veteranos sobre el Durmiente | 80 | `-` |
| `store-season-one` | `game.ftl` | Título promocional de la Temporada 1 | 48 | `-` |
| `team-copper` | `game.ftl` | Nombre completo de la cuadrilla naranja (Copper Crew) | 24 | `-` |
| `team-copper-short` | `game.ftl` | Nombre abreviado de la cuadrilla naranja para tablas y HUD | 12 | `-` |
| `team-copper-motto` | `game.ftl` | Lema de cuadrilla de Copper Crew | 64 | `-` |
| `team-cobalt` | `game.ftl` | Nombre completo de la cuadrilla azul (Cobalt Crew) | 24 | `-` |
| `team-cobalt-short` | `game.ftl` | Nombre abreviado de la cuadrilla azul para tablas y HUD | 12 | `-` |
| `team-cobalt-motto` | `game.ftl` | Lema de cuadrilla de Cobalt Crew | 64 | `-` |
| `mode-shift` | `game.ftl` | Modo competitivo principal: Turno (*Shift* 4v4) | 24 | `-` |
| `mode-shift-desc` | `game.ftl` | Descripción del modo Turno y cuota de extracción | 140 | `-` |
| `mode-practice` | `game.ftl` | Modo de entrenamiento en solitario o cooperativo | 24 | `-` |
| `mode-practice-desc` | `game.ftl` | Descripción del modo de práctica sin presión rival | 140 | `-` |
| `mode-workshop` | `game.ftl` | Modo Taller (*Workshop*) de inspección y pruebas | 24 | `-` |
| `mode-workshop-desc` | `game.ftl` | Descripción del Taller de galerías y mecanismos | 140 | `-` |
| `class-prospector` | `game.ftl` | Nombre de clase: Prospector (*Prospector*) | 20 | `-` |
| `class-prospector-desc` | `game.ftl` | Función táctica del Prospector (detección y escucha) | 120 | `-` |
| `class-miner` | `game.ftl` | Nombre de clase: Minero de frente (*Miner*) | 20 | `-` |
| `class-miner-desc` | `game.ftl` | Función táctica del Minero (excavación pesada y carga) | 120 | `-` |
| `class-engineer` | `game.ftl` | Nombre de clase: Ingeniero (*Engineer*) | 20 | `-` |
| `class-engineer-desc` | `game.ftl` | Función táctica del Ingeniero (infraestructura y cangilones) | 120 | `-` |
| `class-sapper` | `game.ftl` | Nombre de clase: Zapador (*Sapper*) | 20 | `-` |
| `class-sapper-desc` | `game.ftl` | Función táctica del Zapador (voladuras y sabotaje) | 120 | `-` |
| `tool-mineral-detector` | `game.ftl` | Herramienta: detector electromagnético de mineral | 28 | `-` |
| `tool-geophone` | `game.ftl` | Herramienta: Geófono (*Geophone*) de escucha sísmica | 28 | `-` |
| `tool-heavy-pickaxe` | `game.ftl` | Herramienta: pico pesado capaz de romper roca dura | 28 | `-` |
| `tool-cargo-harness` | `game.ftl` | Herramienta: arnés de carga de gran capacidad | 28 | `-` |
| `tool-builder-hammer` | `game.ftl` | Herramienta: martillo de entibación y montaje | 28 | `-` |
| `tool-bucket-wrench` | `game.ftl` | Herramienta: llave de tracción para línea de cangilones | 28 | `-` |
| `tool-blasting-cap` | `game.ftl` | Herramienta: explosor y fulminante de demolición | 28 | `-` |
| `tool-wire-cutters` | `game.ftl` | Herramienta: cizalla aislada para sabotaje de cables | 28 | `-` |
| `tool-pickaxe` | `game.ftl` | Herramienta básica común: pico estándar de mano | 28 | `-` |
| `block-air` | `game.ftl` | Bloque `0` (`air`): galería abierta / aire | 24 | `-` |
| `block-dirt` | `game.ftl` | Bloque `1` (`dirt`): tierra blanda excavable | 24 | `-` |
| `block-rock` | `game.ftl` | Bloque `2` (`rock`): roca común de galería | 24 | `-` |
| `block-hard-rock` | `game.ftl` | Bloque `3` (`hard_rock`): roca dura (requiere pico pesado/carga) | 24 | `-` |
| `block-ore` | `game.ftl` | Bloque `4` (`ore`): veta de mineral común | 24 | `-` |
| `block-gold` | `game.ftl` | Bloque `5` (`gold`): filón de oro nativo | 24 | `-` |
| `block-diamond` | `game.ftl` | Bloque `6` (`diamond`): depósito de diamante profundo | 24 | `-` |
| `block-lava` | `game.ftl` | Bloque `7` (`lava`): bolsa de roca fundida / magma | 24 | `-` |
| `block-team-block` | `game.ftl` | Bloque `8` (`team_block`): entibación blindada de cuadrilla | 24 | `-` |
| `block-force-field` | `game.ftl` | Bloque `9` (`force_field`): barrera de contención de equipo | 24 | `-` |
| `block-ladder` | `game.ftl` | Bloque `10` (`ladder`): escala vertical de madera y hierro | 24 | `-` |
| `block-jump-pad` | `game.ftl` | Bloque `11` (`jump_pad`): plataforma neumática de impulso | 24 | `-` |
| `block-electric` | `game.ftl` | Bloque `12` (`electric`): tendido eléctrico de alta tensión | 24 | `-` |
| `block-rail` | `game.ftl` | Bloque `13` (`rail`): riel de vagoneta y guiado | 24 | `-` |
| `block-beacon` | `game.ftl` | Bloque `14` (`beacon`): baliza de posición y relevo | 24 | `-` |
| `block-explosive` | `game.ftl` | Bloque `15` (`explosive`): carga explosiva (`+3_000` al Sismógrafo) | 24 | `-` |
| `block-bucket-station` | `game.ftl` | Bloque `16` (`bucket_station`): estación de línea de cangilones | 24 | `-` |
| `block-bucket-post` | `game.ftl` | Bloque `17` (`bucket_post`): poste tensor de línea de cangilones | 24 | `-` |
| `seismo-title` | `game.ftl` | Nombre del instrumento: Sismógrafo (`0..=100_000`) | 24 | `-` |
| `seismo-stir` | `game.ftl` | Umbral 25 % (`25_000`): Revuelo (*Stir*) | 18 | `-` |
| `seismo-stir-desc` | `game.ftl` | Efecto de Revuelo: primeros desprendimientos y Silbadores | 120 | `-` |
| `seismo-breathe` | `game.ftl` | Umbral 50 % (`50_000`): Respira (*Breathe*) | 18 | `-` |
| `seismo-breathe-desc` | `game.ftl` | Efecto de Respira: exhalación térmica y apertura de Veta Madre | 120 | `-` |
| `seismo-restless` | `game.ftl` | Umbral 75 % (`75_000`): Inquietud (*Restless*) | 18 | `-` |
| `seismo-restless-desc` | `game.ftl` | Efecto de Inquietud: rotura de bolsas de lava y cierre de pasos | 120 | `-` |
| `seismo-wake` | `game.ftl` | Umbral 100 % (`100_000`): Despertar (*Wake*) | 18 | `-` |
| `seismo-wake-desc` | `game.ftl` | Efecto de Despertar: 90 s (`2_700` ticks) y exposición del Corazón | 120 | `-` |
| `lore-sleeper` | `game.ftl` | Entrada de glosario: el Durmiente (*The Sleeper*) | 140 | `-` |
| `lore-watcher` | `game.ftl` | Entrada de glosario: el Vigía (*The Watcher*) en superficie | 140 | `-` |
| `lore-whistlers` | `game.ftl` | Entrada de glosario: los Silbadores (*The Whistlers*) | 140 | `-` |
| `lore-mother-lode` | `game.ftl` | Entrada de glosario: Veta Madre (*Mother Lode*) | 140 | `-` |
| `lore-heart` | `game.ftl` | Entrada de glosario: el Corazón (*The Heart*) | 140 | `-` |
| `lore-offering` | `game.ftl` | Entrada de glosario: Ofrenda (`100` de oro, `-2_500`, `150` ticks) | 140 | `-` |
| `lore-brass-tally` | `game.ftl` | Entrada de glosario: Ficha de latón (`-8_000`, máx. `2` por Turno) | 140 | `-` |
| `lore-bucket-line` | `game.ftl` | Entrada de glosario: Línea de cangilones (*Bucket line*) | 140 | `-` |

## 2. `hud.ftl` — Menús, indicadores, altar y bajas (47 claves)

| ID | Archivo | Contexto | Máx. car. | Variables / Plural |
|---|---|---|---|---|
| `menu-play` | `hud.ftl` | Botón principal para iniciar un Turno competitivo | 24 | `-` |
| `menu-practice` | `hud.ftl` | Botón para abrir una sesión de práctica | 24 | `-` |
| `menu-workshop` | `hud.ftl` | Botón para entrar al Taller (*Workshop*) | 24 | `-` |
| `menu-tallies` | `hud.ftl` | Botón para consultar el tablero de las 41 fichas de latón | 28 | `-` |
| `menu-settings` | `hud.ftl` | Botón de ajustes generales | 24 | `-` |
| `menu-language` | `hud.ftl` | Selector de idioma (`en`, `es-419`, `es-ES`) | 24 | `-` |
| `menu-audio` | `hud.ftl` | Pestaña de volumen y filtro de radio del Vigía | 24 | `-` |
| `menu-controls` | `hud.ftl` | Pestaña de asignación de teclas y mando | 24 | `-` |
| `menu-video` | `hud.ftl` | Pestaña de resolución y opciones gráficas | 24 | `-` |
| `menu-resume` | `hud.ftl` | Botón del menú de pausa para volver al pozo | 24 | `-` |
| `menu-leave` | `hud.ftl` | Botón para abandonar el Turno en curso | 28 | `-` |
| `menu-quit` | `hud.ftl` | Botón para cerrar HUSHMINE y salir al escritorio | 24 | `-` |
| `menu-choose-crew` | `hud.ftl` | Cabecera de selección entre Copper Crew y Cobalt Crew | 32 | `-` |
| `menu-choose-class` | `hud.ftl` | Cabecera de selección de oficio minero | 32 | `-` |
| `hud-bank` | `hud.ftl` | Marcador de fondos depositados frente a la cuota (`u32`) | 36 | `$cash`, `$target` |
| `hud-ore-carried` | `hud.ftl` | Indicador de mineral común en el arnés | 32 | `$count` [plural] |
| `hud-gold-carried` | `hud.ftl` | Indicador de oro crudo en el arnés | 32 | `$count` [plural] |
| `hud-diamond-carried` | `hud.ftl` | Indicador de diamantes en el arnés | 32 | `$count` [plural] |
| `hud-tally-carried` | `hud.ftl` | Indicador de ficha de latón de 1931 portada (`#1..#41`) | 36 | `$number` |
| `hud-seismo-level` | `hud.ftl` | Lectura porcentual del Sismógrafo en el HUD | 28 | `$percent` |
| `hud-drop-in-timer` | `hud.ftl` | Cuenta atrás en la jaula antes de abrir compuertas | 48 | `$seconds` [plural] |
| `hud-wake-timer` | `hud.ftl` | Reloj de colapso de 90 s (`2_700` ticks) durante Despertar | 48 | `$seconds` [plural] |
| `hud-buckets-stopping` | `hud.ftl` | Aviso de cierre de líneas de cangilones (primeros 60 s) | 64 | `$seconds` [plural] |
| `hud-buckets-halted` | `hud.ftl` | Alerta fija en los últimos 30 s (`900` ticks): cangilones parados | 60 | `-` |
| `hud-fame-active` | `hud.ftl` | Aviso de racha de Fama activa para una cuadrilla | 48 | `$team` |
| `hud-stolen-bonus` | `hud.ftl` | Notificación de prima por mineral robado al rival (`u32`) | 44 | `$cash` |
| `hud-loot-bag-pickup` | `hud.ftl` | Aviso al recoger un saco de botín caído (`u32`) | 48 | `$cash` |
| `hud-respawn-timer` | `hud.ftl` | Cuenta atrás para reaparecer en la jaula de superficie | 48 | `$seconds` [plural] |
| `hud-geophone-contact` | `hud.ftl` | Lectura acústica del Geófono con contactos y distancia | 64 | `$count` [plural], `$distance` |
| `hud-offering-prompt` | `hud.ftl` | Indicación contextual frente al altar (`100` de oro) | 64 | `$cost` |
| `hud-tally-prompt` | `hud.ftl` | Indicación contextual para devolver una ficha de latón | 60 | `-` |
| `altar-err-closed` | `hud.ftl` | `AltarError::AltarClosed`: el Despertar bloqueó el altar | 84 | `-` |
| `altar-err-not-enough-gold` | `hud.ftl` | `AltarError::NotEnoughGold`: falta oro para el lote de `100` | 72 | `$required`, `$carried` |
| `altar-err-cooling-down` | `hud.ftl` | `AltarError::CoolingDown`: enfriamiento de `150` ticks (5 s) | 84 | `$seconds` [plural], `$ready_at_tick` |
| `altar-err-tally-limit` | `hud.ftl` | `AltarError::TallyLimitReached`: tope de `2` fichas por Turno | 76 | `$max` [plural] |
| `altar-offering-success` | `hud.ftl` | Confirmación de Ofrenda aceptada (`-2_500` al Sismógrafo) | 88 | `$team`, `$gold`, `$reduction` |
| `altar-tally-success` | `hud.ftl` | Confirmación de ficha devuelta (`-8_000` al Sismógrafo) | 96 | `$team`, `$number`, `$reduction` |
| `death-lava` | `hud.ftl` | Aviso de baja por inmersión en bolsa de lava | 64 | `$player` |
| `death-electric` | `hud.ftl` | Aviso de baja por descarga en tendido eléctrico | 64 | `$player` |
| `death-explosive` | `hud.ftl` | Aviso de baja por onda expansiva de demolición | 64 | `$player` |
| `death-fall` | `hud.ftl` | Aviso de baja por caída libre en el pozo | 64 | `$player` |
| `death-rockfall` | `hud.ftl` | Aviso de baja por desprendimiento de bóveda | 72 | `$player` |
| `death-shove-lava` | `hud.ftl` | Baja causada por empujón rival hacia la lava | 72 | `$attacker`, `$victim` |
| `death-shove-fall` | `hud.ftl` | Baja causada por empujón rival al vacío | 72 | `$attacker`, `$victim` |
| `death-shove-electric` | `hud.ftl` | Baja causada por empujón rival contra cable activo | 72 | `$attacker`, `$victim` |
| `death-collapse` | `hud.ftl` | Baja al quedar sepultado tras agotarse el Despertar | 68 | `$player` |
| `death-dropped-loot` | `hud.ftl` | Aviso de saco de botín suelto al caer un minero | 68 | `$cash`, `$ore` |

## 3. `watcher.ftl` — Transmisiones de radio del Vigía (50 claves)

Todas las entradas de `watcher.ftl` respetan el tope estricto de **6 palabras como máximo en inglés** y **48 caracteres** para lectura periférica inmediata en el HUD.

| ID | Archivo | Contexto | Máx. car. | Variables / Plural |
|---|---|---|---|---|
| `watcher-stir-1` | `watcher.ftl` | Radio del Vigía — Revuelo (25 %, variante 1) | 48 | `-` |
| `watcher-stir-2` | `watcher.ftl` | Radio del Vigía — Revuelo (25 %, variante 2) | 48 | `-` |
| `watcher-stir-3` | `watcher.ftl` | Radio del Vigía — Revuelo (25 %, variante 3) | 48 | `-` |
| `watcher-stir-4` | `watcher.ftl` | Radio del Vigía — Revuelo (25 %, variante 4) | 48 | `-` |
| `watcher-stir-5` | `watcher.ftl` | Radio del Vigía — Revuelo (25 %, variante 5) | 48 | `-` |
| `watcher-breathe-1` | `watcher.ftl` | Radio del Vigía — Respira (50 %, variante 1) | 48 | `-` |
| `watcher-breathe-2` | `watcher.ftl` | Radio del Vigía — Respira (50 %, variante 2) | 48 | `-` |
| `watcher-breathe-3` | `watcher.ftl` | Radio del Vigía — Respira (50 %, variante 3) | 48 | `-` |
| `watcher-breathe-4` | `watcher.ftl` | Radio del Vigía — Respira (50 %, variante 4) | 48 | `-` |
| `watcher-breathe-5` | `watcher.ftl` | Radio del Vigía — Respira (50 %, variante 5) | 48 | `-` |
| `watcher-restless-1` | `watcher.ftl` | Radio del Vigía — Inquietud (75 %, variante 1) | 48 | `-` |
| `watcher-restless-2` | `watcher.ftl` | Radio del Vigía — Inquietud (75 %, variante 2) | 48 | `-` |
| `watcher-restless-3` | `watcher.ftl` | Radio del Vigía — Inquietud (75 %, variante 3) | 48 | `-` |
| `watcher-restless-4` | `watcher.ftl` | Radio del Vigía — Inquietud (75 %, variante 4) | 48 | `-` |
| `watcher-restless-5` | `watcher.ftl` | Radio del Vigía — Inquietud (75 %, variante 5) | 48 | `-` |
| `watcher-wake-1` | `watcher.ftl` | Radio del Vigía — Despertar (100 %, variante 1) | 48 | `-` |
| `watcher-wake-2` | `watcher.ftl` | Radio del Vigía — Despertar (100 %, variante 2) | 48 | `-` |
| `watcher-wake-3` | `watcher.ftl` | Radio del Vigía — Despertar (100 %, variante 3) | 48 | `-` |
| `watcher-wake-4` | `watcher.ftl` | Radio del Vigía — Despertar (100 %, variante 4) | 48 | `-` |
| `watcher-wake-5` | `watcher.ftl` | Radio del Vigía — Despertar (100 %, variante 5) | 48 | `-` |
| `watcher-offering-1` | `watcher.ftl` | Radio del Vigía — Ofrenda de oro aceptada (variante 1) | 48 | `-` |
| `watcher-offering-2` | `watcher.ftl` | Radio del Vigía — Ofrenda de oro aceptada (variante 2) | 48 | `-` |
| `watcher-offering-3` | `watcher.ftl` | Radio del Vigía — Ofrenda de oro aceptada (variante 3) | 48 | `-` |
| `watcher-offering-4` | `watcher.ftl` | Radio del Vigía — Ofrenda de oro aceptada (variante 4) | 48 | `-` |
| `watcher-offering-5` | `watcher.ftl` | Radio del Vigía — Ofrenda de oro aceptada (variante 5) | 48 | `-` |
| `watcher-tally-1` | `watcher.ftl` | Radio del Vigía — Ficha de latón devuelta (variante 1) | 48 | `-` |
| `watcher-tally-2` | `watcher.ftl` | Radio del Vigía — Ficha de latón devuelta (variante 2) | 48 | `-` |
| `watcher-tally-3` | `watcher.ftl` | Radio del Vigía — Ficha de latón devuelta (variante 3) | 48 | `-` |
| `watcher-tally-4` | `watcher.ftl` | Radio del Vigía — Ficha de latón devuelta (variante 4) | 48 | `-` |
| `watcher-tally-5` | `watcher.ftl` | Radio del Vigía — Ficha de latón devuelta (variante 5) | 48 | `-` |
| `watcher-mother-lode-1` | `watcher.ftl` | Radio del Vigía — Apertura de Veta Madre (variante 1) | 48 | `-` |
| `watcher-mother-lode-2` | `watcher.ftl` | Radio del Vigía — Apertura de Veta Madre (variante 2) | 48 | `-` |
| `watcher-mother-lode-3` | `watcher.ftl` | Radio del Vigía — Apertura de Veta Madre (variante 3) | 48 | `-` |
| `watcher-mother-lode-4` | `watcher.ftl` | Radio del Vigía — Apertura de Veta Madre (variante 4) | 48 | `-` |
| `watcher-mother-lode-5` | `watcher.ftl` | Radio del Vigía — Apertura de Veta Madre (variante 5) | 48 | `-` |
| `watcher-heart-1` | `watcher.ftl` | Radio del Vigía — Exposición del Corazón (variante 1) | 48 | `-` |
| `watcher-heart-2` | `watcher.ftl` | Radio del Vigía — Exposición del Corazón (variante 2) | 48 | `-` |
| `watcher-heart-3` | `watcher.ftl` | Radio del Vigía — Exposición del Corazón (variante 3) | 48 | `-` |
| `watcher-heart-4` | `watcher.ftl` | Radio del Vigía — Exposición del Corazón (variante 4) | 48 | `-` |
| `watcher-heart-5` | `watcher.ftl` | Radio del Vigía — Exposición del Corazón (variante 5) | 48 | `-` |
| `watcher-victory-1` | `watcher.ftl` | Radio del Vigía — Victoria de cuadrilla (variante 1) | 48 | `-` |
| `watcher-victory-2` | `watcher.ftl` | Radio del Vigía — Victoria de cuadrilla (variante 2) | 48 | `-` |
| `watcher-victory-3` | `watcher.ftl` | Radio del Vigía — Victoria de cuadrilla (variante 3) | 48 | `-` |
| `watcher-victory-4` | `watcher.ftl` | Radio del Vigía — Victoria de cuadrilla (variante 4) | 48 | `-` |
| `watcher-victory-5` | `watcher.ftl` | Radio del Vigía — Victoria de cuadrilla (variante 5) | 48 | `-` |
| `watcher-defeat-1` | `watcher.ftl` | Radio del Vigía — Derrota / cierre de pozo (variante 1) | 48 | `-` |
| `watcher-defeat-2` | `watcher.ftl` | Radio del Vigía — Derrota / cierre de pozo (variante 2) | 48 | `-` |
| `watcher-defeat-3` | `watcher.ftl` | Radio del Vigía — Derrota / cierre de pozo (variante 3) | 48 | `-` |
| `watcher-defeat-4` | `watcher.ftl` | Radio del Vigía — Derrota / cierre de pozo (variante 4) | 48 | `-` |
| `watcher-defeat-5` | `watcher.ftl` | Radio del Vigía — Derrota / cierre de pozo (variante 5) | 48 | `-` |

## 4. `signs.ftl` — Carteles de seguridad, memorandos y tablero de fichas (36 claves)

| ID | Archivo | Contexto | Máx. car. | Variables / Plural |
|---|---|---|---|---|
| `sign-01-title` | `signs.ftl` | Cartel `SIGN-01`: título de control de fichas de latón | 64 | `-` |
| `sign-01-body` | `signs.ftl` | Cartel `SIGN-01`: obligación de descolgar ficha antes de bajar | 96 | `-` |
| `sign-02-title` | `signs.ftl` | Cartel `SIGN-02`: título contra ofrendas en grietas | 64 | `-` |
| `sign-02-body` | `signs.ftl` | Cartel `SIGN-02`: prohibición de arrojar oro a las fisuras | 96 | `-` |
| `sign-03-title` | `signs.ftl` | Cartel `SIGN-03`: título de alerta acústica (los Silbadores) | 64 | `-` |
| `sign-03-body` | `signs.ftl` | Cartel `SIGN-03`: orden de desalojar si la veta silba | 96 | `-` |
| `sign-04-title` | `signs.ftl` | Cartel `SIGN-04`: título de disciplina del Sismógrafo | 64 | `-` |
| `sign-04-body` | `signs.ftl` | Cartel `SIGN-04`: prohibido volar por encima de Respira | 96 | `-` |
| `sign-05-title` | `signs.ftl` | Cartel `SIGN-05`: título de seguridad en línea de cangilones | 64 | `-` |
| `sign-05-body` | `signs.ftl` | Cartel `SIGN-05`: aviso de cadena de tracción continua | 96 | `-` |
| `sign-06-title` | `signs.ftl` | Cartel `SIGN-06`: título de silencio junto a geófonos | 64 | `-` |
| `sign-06-body` | `signs.ftl` | Cartel `SIGN-06`: respeto al radio de diez pasos del escucha | 96 | `-` |
| `sign-07-title` | `signs.ftl` | Cartel `SIGN-07`: título de peligro eléctrico en roca húmeda | 64 | `-` |
| `sign-07-body` | `signs.ftl` | Cartel `SIGN-07`: mantener picos lejos de cables activos | 96 | `-` |
| `sign-08-title` | `signs.ftl` | Cartel `SIGN-08`: título de bolsas térmicas y lava | 64 | `-` |
| `sign-08-body` | `signs.ftl` | Cartel `SIGN-08`: sellar el barreno ante veta al rojo | 96 | `-` |
| `sign-09-title` | `signs.ftl` | Cartel `SIGN-09`: título de geometría móvil y cierre | 64 | `-` |
| `sign-09-body` | `signs.ftl` | Cartel `SIGN-09`: fiarse de la estaca topográfica | 96 | `-` |
| `sign-10-title` | `signs.ftl` | Cartel `SIGN-10`: título de Regla Corporativa n.º 44 | 64 | `-` |
| `sign-10-body` | `signs.ftl` | Cartel `SIGN-10`: hablar del Durmiente es motivo de despido | 96 | `-` |
| `memo-header-company` | `signs.ftl` | Membrete oficial de Anima Consolidated (1928–1931) | 64 | `-` |
| `memo-01-subject` | `signs.ftl` | `MEMO-01` (12-X-1928): asunto sobre anomalía bajo -400 m | 110 | `-` |
| `memo-01-body` | `signs.ftl` | `MEMO-01`: informe de Lukas Bergmann (#6) sobre el pulso | 1050 | `-` |
| `memo-02-subject` | `signs.ftl` | `MEMO-02` (19-VI-1929): asunto sobre merma por Ofrendas | 110 | `-` |
| `memo-02-body` | `signs.ftl` | `MEMO-02`: circular de Eleanor Sterling contra las Ofrendas | 1050 | `-` |
| `memo-03-subject` | `signs.ftl` | `MEMO-03` (4-IV-1930): asunto sobre galerías y Silbadores | 110 | `-` |
| `memo-03-body` | `signs.ftl` | `MEMO-03`: informe de Trevithick (#1) y Condori (#10) | 1000 | `-` |
| `memo-04-subject` | `signs.ftl` | `MEMO-04` (28-XI-1930): asunto sobre los 4 umbrales | 110 | `-` |
| `memo-04-body` | `signs.ftl` | `MEMO-04`: estudio técnico de Arthur Vance (25/50/75/100 %) | 1150 | `-` |
| `memo-05-subject` | `signs.ftl` | `MEMO-05` (17-X-1931): asunto de agitación de Veta Madre | 110 | `-` |
| `memo-05-body` | `signs.ftl` | `MEMO-05`: directiva reservada para forzar el umbral del 50 % | 1050 | `-` |
| `memo-06-subject` | `signs.ftl` | `MEMO-06` (14-XI-1931): orden de descenso hacia el Corazón | 110 | `-` |
| `memo-06-body` | `signs.ftl` | `MEMO-06`: orden final para las 41 fichas de latón a -620 m | 1050 | `-` |
| `tally-board-title` | `signs.ftl` | Título del tablero conmemorativo de las 41 fichas de 1931 | 68 | `-` |
| `tally-board-progress` | `signs.ftl` | Contador de fichas recuperadas sobre el total de 41 | 64 | `$count` [plural] |
| `tally-board-entry` | `signs.ftl` | Ficha individual del tablero (#1..#41, nombre, país y oficio) | 96 | `$number`, `$name`, `$country`, `$role` |

## 5. `chronicle.ftl` — Crónica del Turno, hitos y títulos (28 claves)

| ID | Archivo | Contexto | Máx. car. | Variables / Plural |
|---|---|---|---|---|
| `chronicle-header` | `chronicle.ftl` | Cabecera de la gaceta al finalizar el Turno (*Shift Chronicle*) | 40 | `-` |
| `chronicle-subheader` | `chronicle.ftl` | Subtítulo técnico con semilla determinista y duración | 64 | `$seed`, `$duration` |
| `chronicle-victory-quota` | `chronicle.ftl` | Titular de victoria por alcanzar la cuota de extracción (`u32`) | 84 | `$team`, `$cash` |
| `chronicle-victory-clock` | `chronicle.ftl` | Titular de victoria por mayor botín al cierre del reloj (`u32`) | 88 | `$team`, `$cash` |
| `chronicle-draw` | `chronicle.ftl` | Titular de empate exacto en fondos depositados (`u32`) | 72 | `$cash` |
| `chronicle-share-button` | `chronicle.ftl` | Botón para exportar resumen de la Crónica del Turno | 36 | `-` |
| `chronicle-save-replay` | `chronicle.ftl` | Botón para guardar la repetición determinista | 36 | `-` |
| `chronicle-event-stir` | `chronicle.ftl` | Hito del `EventLog`: cruce del umbral Revuelo (25 %) | 96 | `$time` |
| `chronicle-event-breathe` | `chronicle.ftl` | Hito del `EventLog`: cruce del umbral Respira (50 %) | 96 | `$time` |
| `chronicle-event-restless` | `chronicle.ftl` | Hito del `EventLog`: cruce del umbral Inquietud (75 %) | 96 | `$time` |
| `chronicle-event-wake` | `chronicle.ftl` | Hito del `EventLog`: Despertar (100 %) provocado por un equipo | 96 | `$time`, `$team` |
| `chronicle-event-wake-passive` | `chronicle.ftl` | Hito del `EventLog`: Despertar (100 %) por tracción pasiva | 96 | `$time` |
| `chronicle-event-buckets-stopped` | `chronicle.ftl` | Hito del `EventLog`: bloqueo de cangilones a los 60 s del Despertar | 96 | `$time` |
| `chronicle-event-awakening-ended` | `chronicle.ftl` | Hito del `EventLog`: cierre definitivo del pozo a los 90 s | 96 | `$time` |
| `chronicle-event-noise-blasts` | `chronicle.ftl` | Hito del `EventLog`: voladuras acumuladas y aporte sísmico | 104 | `$time`, `$team`, `$count` [plural], `$noise` |
| `chronicle-event-offering` | `chronicle.ftl` | Hito del `EventLog`: Ofrendas de oro y alivio total (`-2_500`/ud.) | 108 | `$time`, `$team`, `$count` [plural], `$gold`, `$reduction` |
| `chronicle-event-tally` | `chronicle.ftl` | Hito del `EventLog`: devolución de ficha de latón (`-8_000`) | 96 | `$time`, `$team`, `$number`, `$reduction` |
| `chronicle-event-fame` | `chronicle.ftl` | Hito del `EventLog`: racha de Fama completada con bonificación | 96 | `$time`, `$team`, `$cash` |
| `chronicle-event-heist` | `chronicle.ftl` | Hito del `EventLog`: robo de mineral en línea de cangilones rival | 96 | `$time`, `$team`, `$cash` |
| `chronicle-event-heart-claimed` | `chronicle.ftl` | Hito del `EventLog`: extracción y cobro del Corazón | 96 | `$time`, `$player`, `$team`, `$cash` |
| `chronicle-title-awakener` | `chronicle.ftl` | Título de jugador: Despertador (*Awakener*) | 28 | `-` |
| `chronicle-title-awakener-desc` | `chronicle.ftl` | Mérito del Despertador: mayor ruido aportado al Sismógrafo | 88 | `$player`, `$noise` |
| `chronicle-title-slumber-guardian` | `chronicle.ftl` | Título de jugador: Guardián del sueño (*Slumber Guardian*) | 28 | `-` |
| `chronicle-title-slumber-guardian-desc` | `chronicle.ftl` | Mérito del Guardián del sueño: más Ofrendas entregadas al altar | 92 | `$player`, `$count` [plural], `$gold` |
| `chronicle-title-mole` | `chronicle.ftl` | Título de jugador: Topo (*Mole*) | 28 | `-` |
| `chronicle-title-mole-desc` | `chronicle.ftl` | Mérito del Topo: mayor volumen de bloques excavados | 88 | `$player`, `$blocks` [plural] |
| `chronicle-title-bucket-thief` | `chronicle.ftl` | Título de jugador: Ladrón de cangilones (*Bucket Thief*) | 28 | `-` |
| `chronicle-title-bucket-thief-desc` | `chronicle.ftl` | Mérito del Ladrón de cangilones: mayor botín interceptado | 96 | `$player`, `$count` [plural], `$cash` |
