# Guía de Dirección de Sonido y Arquitectura Acústica (`grisu-audio`)

Este documento define la identidad acústica de **HUSHMINE**, las especificaciones técnicas de los **30 efectos de sonido (SFX)** de la versión base (`assets/audio/manifest.ron`) y la arquitectura de propagación vóxel y mezcla sobre `kira` para el futuro crate `grisu-audio` del motor **Grisú**.

---

## 1. Identidad sonora de HUSHMINE (1931 / 1979)

En **HUSHMINE**, el sonido no es un adorno cosmético: es el eje táctico de la partida (*hush* significa «silencio»). El paisaje acústico superpone dos estratos históricos separados por casi medio siglo dentro de las entrañas del **monte Anima** (*Mount Anima*):

### 1.1. La memoria enterrada de 1931 («La Larga Noche»)
En la madrugada del **14 al 15 de noviembre de 1931**, durante el desastre conocido como **La Larga Noche** (*The Long Night*), la montaña operada por la quebrada **Anima Consolidated** despertó y sepultó al turno de noche completo: **41 mineros de 12 países** (entre ellos Thomas Trevithick #1, Lukas Bergmann #6, Valentín Condori #10, Mateo Aller #14 y Bridget Gallagher #29). Sus **41 fichas de latón** (*brass tallies*) siguen faltando en el tablero de la lampistería.

Ese estrato de 1931 suena a metal noble envejecido, bronce resonante, silbatos de alarma bitonales de capataz, madera de entibación sometida a compresión, pizarra húmeda y cavidades abovedadas que devuelven ecos largos y huecos.

### 1.2. La intrusión analógica de otoño de 1979
En el otoño de **1979**, dos cuadrillas rivales —**Copper Crew** (naranja, descendientes de las familias de 1931 que conocen las galerías viejas) y **Cobalt Crew** (azul, técnicos contratados con instrumental excedente de la Compañía)— saltan la valla oxidada de noche para saquear el oro antes de que la montaña despierte.

Toda la tecnología que portan las cuadrillas pertenece a la ingeniería electromecánica e industrial de finales de los años setenta:
- **Conmutación pesada:** relés de baquelita, contactores de muelle, interruptores de palanca y chasquidos secos de portadora PTT (*Push-To-Talk*).
- **Electrónica de válvulas y transformadores:** zumbido de red de 50/60 Hz y sus armónicos impares (100/120 Hz, 180 Hz) procedentes de balastos de lámparas de vapor de sodio y grupos electrógenos en superficie.
- **Radio de onda corta y cinta magnética:** el enlace con **el Vigía** (*The Watcher*) en superficie se filtra en la banda telefónica de **300 Hz a 3 kHz**, con saturación asimétrica de válvula, llovizna de estática HF y compresión de casete.
- **Terminales de fósforo verde:** los sonidos de interfaz (`ui`) imitan pitidos cuadrados y triangulares de osciloscopio y microordenadores industriales de 1979, sin brillos digitales modernos.

| Estrato acústico | Materiales y fuentes físicas | Banda dominante | Función narrativa y jugable |
|---|---|---|---|
| **1931 (La Larga Noche)** | Latón octogonal, bronce de campana, silbato de capataz, pizarra y basalto | `250 Hz – 2,8 kHz` (modos inarmónicos) | Avisos de derrumbe (**los Silbadores**), retorno de las 41 fichas (`tally_return`) y ofrendas (`offering_accept`). |
| **1979 (Incursión clandestina)** | Acero de pico, relés de baquelita, cangilones, válvulas HF, lámparas de sodio | `50 Hz – 180 Hz` (red) y `300 Hz – 4 kHz` | Herramientas (`tools`), bloques de ingeniería (`blocks`), radio del **Vigía** (`radio`) e interfaz (`ui`). |
| **El Monte Anima (El Durmiente)** | Pulso tectónico, bolsas de magma, exhalación térmica en caverna | `24 Hz – 120 Hz` (infrasonido y subgrave) | Escalada de los 4 umbrales del **Sismógrafo** (`Stir`, `Breathe`, `Restless`, `Wake`). |

### 1.3. El silencio como mecánica central
Cada decisión del jugador tiene una huella acústica vinculada a cuatro sistemas de `hush-sim` (simulación determinista a **30 Hz**, donde un **Turno** o *Shift* dura hasta `16` minutos = `28_800` ticks por crecimiento pasivo):
1. **El Sismógrafo (*Seismograph*):** mide la agitación de **el Durmiente** (*The Sleeper*) en entero `u32` de `0` a `100_000` (milésimas de punto porcentual). Picar a mano es silencioso para la montaña (`dig_block: 0`), pero las líneas de cangilones (`bucket_line_per_tick: +1`, equivalente a `+300` o `+0,3 %` cada `10 s`), las descargas eléctricas (`electric_discharge: +200` / `+0,2 %`), la lava en movimiento (`lava_flow: +200` / `+0,2 %`) y los explosivos (`explosion: +3_000` / `+3,0 %`) aceleran su despertar.
2. **El geófono (*Geophone*) del Prospector:** aunque picar un bloque no altera al Durmiente, las vibraciones del pico viajan por los vóxeles materiales (`transmits_noise: true` en `data/blocks.ron`), permitiendo al Prospector rival escuchar excavaciones a través de la roca viva (`geophone_dig`).
3. **Los Silbadores (*The Whistlers*):** en la v1.0 son un fenómeno **100 % acústico e invisible**. A partir del umbral **Revuelo** (`Stir`, `25_000`), el techo que va a derrumbarse emite un silbido posicionado en 3D (`whistler_1..3`). Su timbre mantiene una ambigüedad deliberada entre gas a presión escapando por microfisuras de pizarra, el aliento del Durmiente y las cadencias de los silbatos de alarma de los 41 mineros de 1931.
4. **El Vigía (*The Watcher*):** operador de cada cuadrilla apostado en superficie junto al camión. Canta por radio los **10 eventos clave** del Turno con frases urgentes de **máximo 6 palabras en inglés**, enmarcadas siempre por `radio_click_on`, el fondo `radio_static_loop` y `radio_click_off`.

---

## 2. Estándar técnico y generación procedural reproducible

Todos los activos sonoros de la versión base se generan de forma **100 % procedural y determinista** mediante los módulos de `tools/audio/` (`synth.py`, `sfx_tools.py`, `sfx_world.py`, `sfx_radio.py` y `build_all.py`, usando únicamente la biblioteca estándar de Python: `math`, `struct`, `wave`) y se validan con `tools/audio/test_audio.py` y `python tools/audio/build_all.py --check`.

### 2.1. Especificación de formato WAV canónico
Cada archivo listado en `assets/audio/manifest.ron` y almacenado en `assets/audio/sfx/<id>.wav` cumple sin excepción:
- **Contenedor y codificación:** RIFF WAVE estándar, **PCM lineal de 16 bits con signo** (`i16`, *little-endian*, rango `-32768..=32767`).
- **Frecuencia de muestreo (`sample_rate`):** `44_100` Hz.
- **Canales (`channels`):** `1` (mono). El motor `grisu-audio` aplica panorámica e intensidad 3D en tiempo de ejecución para los sonidos posicionales y reproduce en centro dual los sonidos 2D.
- **Techo de pico (*headroom*):** amplitud máxima normalizada `<= 0,98` de escala completa (`32_111` en `i16`, equivalente a `-0,18 dBFS`) para evitar recorte (*clipping*) en el acumulador antes del limitador del bus maestro.
- **Ausencia de silencio muerto:** ningún efecto comienza ni termina con colas vacías; todos tienen energía útil (`RMS > 0,01`) y envolvente de salida limpia a cero.

### 2.2. Síntesis determinista, bucles sin chasquido y presupuesto de RAM
- **Síntesis física, FM y sustractiva:** `tools/audio/synth.py` proporciona osciladores (`sine_wave`, `square_wave`, `saw_wave`, `triangle_wave`, `freq_sweep`, `fm_tone`), envolventes (`adsr_envelope`, `exp_decay_envelope`), filtros IIR de un polo (`lowpass_one_pole`, `highpass_one_pole`, `bandpass_one_pole`) y mezcla con recorte suave (`soft_clip`, `mix_tracks`).
- **Bucles sin chasquido (`looping: true`):** los tres bucles continuos (`bucket_line_loop`, `mine_ambience_loop` y `radio_static_loop`) emplean `make_seamless_loop` con ventana de fundido cruzado en los extremos y **cruce por cero exacto** en la primera y última muestra (`sample[0] == 0` y `sample[N - 1] == 0`), evitando discontinuidades de fase al ciclar en `kira`.
- **PRNG determinista (LCG):** el ruido se sintetiza con `DeterministicRng` (generador congruencial lineal de 32 bits con semilla explícita por sonido, sin usar `random`), garantizando que `python tools/audio/build_all.py` produzca exactamente los mismos bytes en Windows, Linux y macOS.
- **Presupuesto de memoria:** el límite arquitectónico para `assets/audio/` es **8 MB como máximo**. Los 30 archivos WAV suman exactamente `1_005_921` muestras (`22,81 s` de audio total), lo que ocupa **1,92 MB** (`2_013_162` bytes en disco incluyendo cabeceras RIFF de 44 bytes), dejando más de 6 MB de margen libre.

### 2.3. Esquema del manifiesto (`assets/audio/manifest.ron`)
El catálogo se serializa en RON siguiendo el mismo estilo de tuplas con nombre que `assets/textures/blocks/manifest.ron`:
```ron
// Manifiesto de efectos de sonido de HUSHMINE (WAV PCM 16 bits, 44100 Hz, mono).
// Generado por tools/audio/build_all.py — no editar a mano.
[
    (id: "pick_hit_rock", file: "sfx/pick_hit_rock.wav", samples: 7938, looping: false, category: "tools"),
    (id: "bucket_line_loop", file: "sfx/bucket_line_loop.wav", samples: 66150, looping: true, category: "blocks"),
]
```

---

## 3. Catálogo maestro de los 30 efectos de sonido

Los 30 sonidos se agrupan en **6 categorías canónicas** (`category` en `assets/audio/manifest.ron`):

1. **`tools` (6 sonidos):** impactos directos de excavación manual (`pick_hit_rock`, `pick_hit_dirt`) y las 4 variantes de pisada de bota minera con puntera de acero (`footstep_1..4`). Timbre seco, cercano y rico en transitorios de `1,5 kHz` a `4,2 kHz` para respuesta inmediata al jugador.
2. **`blocks` (6 sonidos):** eventos físicos de la cuadrícula vóxel (`block_break`, `block_place`, `explosion`, `electric_zap`, `lava_bubble`, `bucket_line_loop`). Combinan síntesis modal de roca/metal con ruido filtrado y modulación de amplitud mecánica.
3. **`seismograph` (7 sonidos):** lenguaje acústico del Durmiente y del altar (`seismo_tick`, `seismo_stir`, `seismo_breathe`, `seismo_restless`, `wake_rumble`, `offering_accept`, `tally_return`). Ocupan la octava subgrave (`26 Hz – 110 Hz`) para los umbrales sísmicos y los armónicos inarmónicos del bronce/latón (`520 Hz – 2,8 kHz`) para las ofrendas y fichas de 1931.
4. **`ambience` (5 sonidos):** atmósfera subterránea continua (`mine_ambience_loop`), los tres avisos espaciales 3D de **los Silbadores** (`whistler_1..3`) y la resonancia pétrea del **geófono** (`geophone_dig`).
5. **`radio` (3 sonidos):** envolvente física del transceptor de onda corta del **Vigía** (`radio_click_on`, `radio_click_off`, `radio_static_loop`).
6. **`ui` (3 sonidos):** retroalimentación instantánea de menús, selección de clase y errores de acción o del altar (`ui_select`, `ui_back`, `ui_error`).

### 3.1. Tabla maestra de los 30 sonidos (`assets/audio/manifest.ron`)

| # | `id` | Archivo (`file`) | Categoría (`category`) | Duración (s) | Muestras (`samples`) | Bucle (`looping`) | Posicional | Vol. rel. (dBFS / lin.) | Mecánica que comunica |
|---:|---|---|---|---:|---:|:---:|:---:|---|---|
| 1 | `pick_hit_rock` | `sfx/pick_hit_rock.wav` | `tools` | 0,18 | `7938` | `false` | Sí (3D) | `-3,0 dB` (`0,71`) | Impacto metálico de pico contra roca (`rock`, `hard_rock`, `ore`, `gold`, `diamond`). Suma `0` al Sismógrafo (`dig_block: 0`), pero se propaga por vóxeles con `transmits_noise: true`. |
| 2 | `pick_hit_dirt` | `sfx/pick_hit_dirt.wav` | `tools` | 0,16 | `7056` | `false` | Sí (3D) | `-4,0 dB` (`0,63`) | Golpe seco y amortiguado contra tierra (`dirt`, id 1). Suma `0` al Sismógrafo. |
| 3 | `block_break` | `sfx/block_break.wav` | `blocks` | 0,35 | `15435` | `false` | Sí (3D) | `-2,0 dB` (`0,79`) | Ruptura final de un bloque al agotar sus ticks de dureza (`hardness`), soltando recurso o abriendo paso. |
| 4 | `block_place` | `sfx/block_place.wav` | `blocks` | 0,22 | `9702` | `false` | Sí (3D) | `-3,0 dB` (`0,71`) | Colocación y anclaje de bloque construible (`8..=17`) consumiendo mena (`ore_cost`). |
| 5 | `explosion` | `sfx/explosion.wav` | `blocks` | 1,20 | `52920` | `false` | Sí (3D) | `0,0 dB` (`1,00`) | Detonación de `explosive` (id 15). Destruye roca dura y suma `+3_000` (`+3,0 %`) al Sismógrafo. |
| 6 | `electric_zap` | `sfx/electric_zap.wav` | `blocks` | 0,40 | `17640` | `false` | Sí (3D) | `-1,5 dB` (`0,84`) | Descarga en red conectada a `electric` (id 12). Empuja y suma `+200` (`+0,2 %`) al Sismógrafo. |
| 7 | `lava_bubble` | `sfx/lava_bubble.wav` | `blocks` | 0,55 | `24255` | `false` | Sí (3D) | `-3,5 dB` (`0,67`) | Burbujeo y avance de `lava` (id 7). Peligro letal por contacto y `+200` (`+0,2 %`) al Sismógrafo al fluir. |
| 8 | `bucket_line_loop` | `sfx/bucket_line_loop.wav` | `blocks` | 1,50 | `66150` | `true` | Sí (3D) | `-4,5 dB` (`0,60`) | Bucle mecánico de `bucket_station` (id 16) y `bucket_post` (id 17). Indica línea activa (`+1`/tick = `+300` cada 10 s) y permite localizarla para sabotaje. Calla en los últimos `900` ticks (30 s) del Despertar. |
| 9 | `footstep_1` | `sfx/footstep_1.wav` | `tools` | 0,14 | `6174` | `false` | Sí (3D) | `-6,0 dB` (`0,50`) | Pisada de bota minera sobre roca (variante 1: impacto grave de tacón). Delata movimiento cercano. |
| 10 | `footstep_2` | `sfx/footstep_2.wav` | `tools` | 0,14 | `6174` | `false` | Sí (3D) | `-6,0 dB` (`0,50`) | Pisada de bota minera con roce de gravilla (variante 2). |
| 11 | `footstep_3` | `sfx/footstep_3.wav` | `tools` | 0,14 | `6174` | `false` | Sí (3D) | `-6,0 dB` (`0,50`) | Pisada de bota minera sobre pizarra seca (variante 3). |
| 12 | `footstep_4` | `sfx/footstep_4.wav` | `tools` | 0,14 | `6174` | `false` | Sí (3D) | `-6,0 dB` (`0,50`) | Pisada de bota minera con crujido de escombro (variante 4). |
| 13 | `seismo_tick` | `sfx/seismo_tick.wav` | `seismograph` | 0,09 | `3969` | `false` | No (2D) | `-5,0 dB` (`0,56`) | Golpe seco de la aguja del Sismógrafo en el HUD al registrar un salto brusco de ruido. |
| 14 | `seismo_stir` | `sfx/seismo_stir.wav` | `seismograph` | 1,20 | `52920` | `false` | No (2D) | `-1,0 dB` (`0,89`) | Cruce del umbral **Revuelo** (`Stir`, `25_000` / 25 %). Primer temblor y activación de microderrumbes. |
| 15 | `seismo_breathe` | `sfx/seismo_breathe.wav` | `seismograph` | 1,50 | `66150` | `false` | No (2D) | `-0,5 dB` (`0,94`) | Cruce del umbral **Respira** (`Breathe`, `50_000` / 50 %). Exhalación térmica y apertura de la **Veta Madre**. |
| 16 | `seismo_restless` | `sfx/seismo_restless.wav` | `seismograph` | 1,60 | `70560` | `false` | No (2D) | `0,0 dB` (`1,00`) | Cruce del umbral **Inquietud** (`Restless`, `75_000` / 75 %). Alerta de zona roja, rotura de bolsas de lava y cierre de galerías. |
| 17 | `wake_rumble` | `sfx/wake_rumble.wav` | `seismograph` | 2,20 | `97020` | `false` | No (2D) | `0,0 dB` (`1,00`) | Cruce del umbral **Despertar** (`Wake`, `100_000` / 100 %). Colapso sísmico de 90 s (`2_700` ticks), exposición de **el Corazón** y ascenso de lava cada 90 ticks. |
| 18 | `whistler_1` | `sfx/whistler_1.wav` | `ambience` | 1,10 | `48510` | `false` | Sí (3D) | `-2,0 dB` (`0,79`) | **Los Silbadores** (variante 1: cadencia de silbato de 1931 en fisura). Aviso espacial 3D en el techo que va a derrumbarse (desde `25_000`). |
| 19 | `whistler_2` | `sfx/whistler_2.wav` | `ambience` | 1,15 | `50715` | `false` | Sí (3D) | `-2,0 dB` (`0,79`) | **Los Silbadores** (variante 2: doble armónico hueco con batimento). Localización direccional de derrumbe inminente. |
| 20 | `whistler_3` | `sfx/whistler_3.wav` | `ambience` | 1,20 | `52920` | `false` | Sí (3D) | `-2,0 dB` (`0,79`) | **Los Silbadores** (variante 3: tres notas con glissando descendente). Aviso diegético de colapso de bóveda. |
| 21 | `geophone_dig` | `sfx/geophone_dig.wav` | `ambience` | 0,45 | `19845` | `false` | Sí (3D) | `-3,5 dB` (`0,67`) | Excavación rival escuchada con el **geófono** (`geophone`) del Prospector a través de bloques con `transmits_noise: true` (filtrado paso bajo profundo y resonancia pétrea). |
| 22 | `offering_accept` | `sfx/offering_accept.wav` | `seismograph` | 0,90 | `39690` | `false` | Sí (3D) | `-1,5 dB` (`0,84`) | **Ofrenda** aceptada en el altar (`100 $` de oro -> `-2_500` al Sismógrafo, enfriamiento `150` ticks). |
| 23 | `tally_return` | `sfx/tally_return.wav` | `seismograph` | 1,10 | `48510` | `false` | Sí (3D) | `-1,0 dB` (`0,89`) | **Ficha de latón** (1–41) devuelta al altar (`-8_000` al Sismógrafo, máx. `2` por Turno). |
| 24 | `mine_ambience_loop` | `sfx/mine_ambience_loop.wav` | `ambience` | 3,00 | `132300` | `true` | No (2D) | `-9,0 dB` (`0,35`) | Bucle de fondo subterráneo: pulso infrasónico del monte Anima, ventilación lejana y zumbido de transformadores de 1979. |
| 25 | `radio_click_on` | `sfx/radio_click_on.wav` | `radio` | 0,12 | `5292` | `false` | No (2D) | `-3,0 dB` (`0,71`) | Apertura de portadora PTT y relé del **Vigía** antes de transmitir un aviso de radio. |
| 26 | `radio_click_off` | `sfx/radio_click_off.wav` | `radio` | 0,12 | `5292` | `false` | No (2D) | `-3,0 dB` (`0,71`) | Cierre del conmutador PTT al concluir la frase del Vigía. |
| 27 | `radio_static_loop` | `sfx/radio_static_loop.wav` | `radio` | 1,50 | `66150` | `true` | No (2D) | `-7,5 dB` (`0,42`) | Bucle de estática de onda corta (300 Hz – 3 kHz) durante las intervenciones del Vigía. |
| 28 | `ui_select` | `sfx/ui_select.wav` | `ui` | 0,10 | `4410` | `false` | No (2D) | `-4,0 dB` (`0,63`) | Confirmación en menús, selección de clase o terminal de fósforo verde. |
| 29 | `ui_back` | `sfx/ui_back.wav` | `ui` | 0,11 | `4851` | `false` | No (2D) | `-4,5 dB` (`0,60`) | Cancelación o retorno en interfaz. |
| 30 | `ui_error` | `sfx/ui_error.wav` | `ui` | 0,25 | `11025` | `false` | No (2D) | `-2,5 dB` (`0,75`) | Rechazo de acción: falta de mena/oro (`NotEnoughGold`), enfriamiento (`CoolingDown`), límite de fichas (`TallyLimitReached`) o altar cerrado (`AltarClosed`). |

### 3.2. Progresión acústica del Sismógrafo y del Altar
- **Escalada por umbrales (`data/seismograph.ron`):**
  1. `Stir` (`25_000` / 25 %): `seismo_stir` introduce un temblor subgrave en `42 Hz` con crujido de pizarra y habilita las apariciones de `whistler_1..3` antes de cada microderrumbe.
  2. `Breathe` (`50_000` / 50 %): `seismo_breathe` añade un barrido resonante cálido (`90 Hz -> 260 Hz -> 70 Hz`) que evoca una inhalación/exhalación ciclópea al abrirse la **Veta Madre** (*Mother Lode*, valor `2_000 $`).
  3. `Restless` (`75_000` / 75 %): `seismo_restless` suma pulsos de alarma sísmica y crepitación de magma cuando se rompen bolsas de lava y se cierran o abren pasos.
  4. `Wake` (`100_000` / 100 %): `wake_rumble` libera un impacto tectónico de `2,20 s` con caída infrasónica (`58 Hz -> 26 Hz`). Inicia la cuenta atrás final de `2_700` ticks (`90 s`), expone **el Corazón** (*The Heart*, valor `1_500 $`), eleva la lava cada `90` ticks (`3 s`) y silencia de golpe todos los bucles `bucket_line_loop` cuando restan `900` ticks (`30 s`).
- **Alivio en el altar (valores provisionales de `ReliefConfig`):**
  - `offering_accept`: cuando un jugador entrega un lote de `100 $` de oro del banco, suena una caída pesada de lingotes seguida de una nota grave de campana mineral que resta `-2_500` (`-2,5 %`, enfriamiento de `150` ticks = `5 s` por equipo).
  - `tally_return`: devolver una **ficha de latón** numerada (1 a 41) en su gancho produce un chasquido metálico octogonal y un acorde de latón solemne que resta `-8_000` (`-8,0 %`, máximo `2` fichas por Turno entre ambos equipos). Si se intenta usar el altar con error (`NotEnoughGold`, `CoolingDown`, `TallyLimitReached` o `AltarClosed` tras `100_000`), se dispara `ui_error`.

### 3.3. Recetas de síntesis por familia acústica (`tools/audio/sfx_*.py`)
Cada familia emplea una arquitectura de síntesis diferenciada para evitar solapamiento espectral en combate:
- **Impactos y herramientas (`tools`, `sfx_tools.py`):**
  - `pick_hit_rock`: golpe grave sobre roca (`240 -> 85 Hz`) + doble operador FM inarmónico de acero templado (`1_420 / 2_180 Hz` y `2_890 / 1_910 Hz`) + esquirlas agudas en paso banda (`1_800..6_500 Hz`).
  - `pick_hit_dirt`: golpe grave amortiguado (`150 -> 52 Hz`) con cuerpo triangular y doble capa de terrones (`lowpass 950 Hz` y `bandpass 250..1_400 Hz`) en `160 ms`.
  - `footstep_1..4`: impacto de bota (`96 Hz – 134 Hz`) combinado con cuatro texturas granulares distintas y semillas propias (roca firme, gravilla suelta, laja de pizarra y escombro).
- **Física de bloques y maquinaria (`blocks`, `sfx_tools.py`):**
  - `block_break` y `block_place`: caída estructural (`190 -> 45 Hz`) con 5 fragmentos escalonados en `350 ms` frente a encaje firme (`110 -> 65 Hz`) seguido a los `70 ms` de un remache FM (`920 Hz + 1_380 Hz`) en `220 ms`.
  - `explosion`: onda expansiva subgrave exponencial (`140 -> 28 Hz`), rugido de escombros (`lowpass 420 Hz`) y 5 rebotes de roca en la cola durante `1,20 s`.
  - `electric_zap`: zumbido de red (`120 Hz` diente de sierra + `240 Hz` cuadrada), arco FM (`1_650 Hz`, modulador `115 Hz`) y ráfagas de chispas (`highpass 2_200 Hz`).
  - `lava_bubble`: hervor magmático (`lowpass 220 Hz` + pulso a `56 Hz`) y 3 burbujas viscosas ascendentes (`95 -> 290 Hz`, `120 -> 360 Hz`, `88 -> 255 Hz`) con *pop* grave al estallar.
  - `bucket_line_loop`: motor en múltiplos armónicos del periodo (`40 / 80 / 160 Hz`), 3 pasos de cangilón cada `0,50 s` (`22_050` muestras) y roce de cable modulado a `2 / 4 Hz`.
- **Los Silbadores y el geófono (`ambience`, `sfx_world.py`):**
  - `whistler_1..3`: síntesis híbrida de silbato antiguo de 1931 (`540 Hz – 1_240 Hz`) con batimento inarmónico, soplo de aire en fisura (`900..2_600 Hz`) y doble eco de caverna filtrado.
  - `geophone_dig`: doble picado distante con resonancia de membrana de mica (`185 Hz`) y doble filtro paso bajo en cascada (`390 Hz` y `350 Hz`).
- **Radio de onda corta e interfaz (`radio` y `ui`, `sfx_radio.py`):**
  - `radio_click_on` / `radio_click_off`: chasquido mecánico de muelle de baquelita (`600..3_200 Hz`) + enganche/caída de portadora (`950 -> 1_480 Hz` al abrir y `1_320 -> 680 Hz` al cerrar).
  - `radio_static_loop`: ruido blanco filtrado entre `300 Hz` y `3_000 Hz` con modulación ionosférica lenta (`2 / 4 Hz`) y heterodino tenue (`1_050 Hz`).
  - `ui_select`, `ui_back`, `ui_error`: ondas triangulares, senoidales y cuadradas estilo terminal CRT de 1979 (`660 -> 990 Hz` al confirmar, `740 -> 493,88 Hz` al volver y doble pulso grave disonante `165 Hz / 155 Hz` en error).

---

## 4. Propagación por la cuadrícula vóxel y el geófono (`grisu-audio`)

El catálogo `data/blocks.ron` (Convención A, `0..=17`) define la propiedad booleana `transmits_noise`, que divide el espacio acústico de HUSHMINE en dos regímenes complementarios:

### 4.1. Propagación aérea (túneles de `0 air`) vs. propagación sísmica (roca `1..=17`)
- **Propagación aérea estándar (oído desnudo):**
  - Todos los efectos 3D (`pick_hit_*`, `block_*`, `explosion`, `electric_zap`, `lava_bubble`, `bucket_line_loop`, `footstep_1..4`, `whistler_1..3`, `offering_accept`, `tally_return`) usan panorámica estereofónica de potencia constante (*equal-power panning*) y atenuación por distancia con radio mínimo `d_min = 2,0` bloques y radio máximo `d_max = 28,0` bloques.
  - Si el segmento entre emisor y oyente atraviesa bloques con `solid: true`, se aplica una pérdida por oclusión de `-3,5 dB` y reducción del filtro paso bajo por cada vóxel sólido interpuesto (hasta un tope de `-18,0 dB`).
- **Propagación sísmica por roca sólida (`geophone_dig`):**
  - Cuando el **Prospector** apoya el **geófono** contra una pared o suelo, el motor invierte la lógica acústica y escucha únicamente la vibración transmitida por la masa sólida.
  - Los **17 bloques materiales (`1..=17`)** —`1 dirt`, `2 rock`, `3 hard_rock`, `4 ore`, `5 gold`, `6 diamond`, `7 lava`, `8 team_block`, `9 force_field`, `10 ladder`, `11 jump_pad`, `12 electric`, `13 rail`, `14 beacon`, `15 explosive`, `16 bucket_station` y `17 bucket_post`— tienen `transmits_noise: true` y conducen la onda elástica.
  - Únicamente **`0 air`** tiene `transmits_noise: false`. Una galería vacía, trinchera o cámara de aire actúa como **aislante acústico sísmico**: corta la cadena de transmisión por roca, impidiendo que la vibración cruce el vacío hacia el geófono. Esto permite contrajuego táctico: excavar una zanja de aire (`0 air`) alrededor de una bóveda aisla acústicamente la operación frente al Prospector enemigo.

### 4.2. Filtrado paso bajo y atenuación por distancia en bloques (`d_roca`)
Para resolver `geophone_dig`, `grisu-audio` ejecuta una búsqueda en anchura (BFS) acotada a `24` pasos ortogonales exclusivamente por celdas con `transmits_noise: true` desde el bloque picado hasta el bloque auscultado por el Prospector:

| Distancia por roca (`d_roca` en bloques) | Ganancia relativa (dB) | Frecuencia de corte paso bajo (`cutoff_hz`) | Lectura táctica para el Prospector |
|---|---|---:|---|
| `1 ..= 4` | `0,0 dB` a `-2,0 dB` | `950 Hz` | Golpe pétreo nítido y grave: el rival está al otro lado del muro inmediato. |
| `5 ..= 10` | `-2,5 dB` a `-6,5 dB` | `580 Hz` | Pulso amortiguado pero direccional: excavación activa en la galería contigua. |
| `11 ..= 17` | `-7,0 dB` a `-12,5 dB` | `320 Hz` | Latido sordo de baja frecuencia: actividad minera a media distancia en el mismo macizo. |
| `18 ..= 24` | `-13,0 dB` a `-20,0 dB` | `170 Hz` | Rumor subgrave límite antes del umbral de silencio (`> 24` bloques o cortado por `0 air`). |

---

## 5. Arquitectura de mezcla para `kira` (`grisu-audio`)

El crate `grisu-audio` encapsulará `kira::AudioManager` sin exponer tipos de audio en `hush-sim` (respetando la regla de que `hush-sim` es 100 % determinista y sin gráficos ni sonido; `grisu-audio` no depende de `hush-*`).

### 5.1. Jerarquía de buses (`kira::track`)
```text
master (limitador suave de techo a -0,2 dBFS)
 ├── seismograph   (2D/3D crítico: seismo_*, wake_rumble, offering_accept, tally_return)
 ├── radio         (2D banda estrecha 300 Hz - 3 kHz: radio_click_*, radio_static_loop + Vigía)
 ├── sfx_world_3d  (bus espacializado con atenuación y filtro de oclusión; recibe ducking)
 │    ├── tools        (pick_hit_*, footstep_1..4)
 │    ├── blocks       (block_*, explosion, electric_zap, lava_bubble, bucket_line_loop)
 │    └── ambience_3d  (whistler_1..3, geophone_dig)
 ├── ambience_2d   (mine_ambience_loop; recibe ducking)
 └── ui            (2D sin efectos: ui_select, ui_back, ui_error)
```

### 5.2. Presupuesto de 32 voces, prioridades y *voice stealing*
Para mantener un coste de CPU predecible e impedir el enmascaramiento caótico en combates o demoliciones múltiples, el motor fija un **tope global de 32 voces activas simultáneas** gobernado por 5 niveles de prioridad (`P0` máxima a `P4` mínima):

| Prioridad | Categoría / Grupo de sonidos | Tope del grupo | Tope por `id` | Política de *voice stealing* al saturar el cupo |
|---|---|---:|---:|---|
| **`P0` (Crítica)** | Umbrales (`seismo_stir`, `seismo_breathe`, `seismo_restless`, `wake_rumble`) y altar (`offering_accept`, `tally_return`) | `3` | `1` (`2` en altar) | **Nunca se roban.** Un umbral nuevo reemplaza al umbral anterior si aún resonaba. |
| **`P1` (Táctica alta)** | `explosion`, `whistler_1..3`, `radio_click_*`, `radio_static_loop`, `seismo_tick` | `8` | `4` (`whistler_*`), `2` (`explosion`), `1` (resto) | Si hay más de `4` Silbadores o `2` explosiones simultáneas, se roba la instancia **más lejana** al oyente. |
| **`P2` (Táctica media)** | `geophone_dig`, `electric_zap`, `block_break`, `block_place`, `ui_*` | `8` | `3` (`geophone_dig`, `block_*`), `2` (`electric_zap`, `ui_*`) | Se roba la voz con **menor ganancia efectiva** (combinación de edad de envolvente y distancia). |
| **`P3` (Continua / Acción)** | `pick_hit_rock`, `pick_hit_dirt`, `lava_bubble`, `bucket_line_loop` | `8` | `4` (`pick_hit_*`), `3` (`bucket_line_loop`), `2` (`lava_bubble`) | Agrupación espacial para `bucket_line_loop`: solo suenan los `3` nodos más cercanos al jugador. En picos, se roba el golpe **más antiguo**. |
| **`P4` (Detalle / Fondo)** | `footstep_1..4`, `mine_ambience_loop` | `5` | `4` (`footstep_*` combinados), `1` (`mine_ambience_loop`) | `mine_ambience_loop` es fijo (`1` voz). Las pisadas ceden su voz inmediatamente ante cualquier petición `P0..P3` si el total global alcanza `32`. |

> **Variación anti-fatiga en `tools` y `blocks`:** al disparar `pick_hit_*`, `block_break`, `block_place` o `footstep_1..4`, `grisu-audio` aplica una microvariación determinista de tono (`playback_rate` entre `0,96` y `1,04`, aprox. `+/- 70 cents`) y de ganancia (`+/- 0,8 dB`) para evitar el efecto ametralladora sin alterar la identidad del activo WAV.

### 5.3. Reglas de *sidechain ducking* (prioridad de lectura de partida)
En un juego donde escuchar un derrumbe o el aviso de apertura de la Veta Madre decide el Turno, la mezcla aplica atenuación automática (*ducking*) mediante automatización de volumen de `kira::track`:

1. **Disparo de umbral del Sismógrafo (`seismo_stir`, `seismo_breathe`, `seismo_restless`, `wake_rumble`):**
   - Atenúa `sfx_world_3d/tools`, `sfx_world_3d/blocks` y `ambience_2d` en **`-6,0 dB`** (ataque de `25 ms`, mantenimiento durante el cuerpo del efecto y liberación exponencial de `400 ms`).
   - El sub-bus `ambience_3d` (`whistler_1..3`) **no** se atenúa para que los avisos de derrumbe sigan siendo localizables durante el temblor.
2. **Transmisión del Vigía por `radio` (`radio_click_on` -> locución -> `radio_click_off`):**
   - Mientras la portadora PTT está abierta, atenúa `sfx_world_3d` y `ambience_2d` en **`-4,5 dB`** (ataque de `30 ms`, liberación de `250 ms` tras `radio_click_off`), garantizando que las frases cortas del Vigía (máximo 6 palabras en inglés) se entiendan con claridad sobre las líneas de cangilones y las piquetas.
3. **Parada sísmica de los últimos 30 segundos (`bucket_stop_remaining_ticks: 900`):**
   - Cuando la cuenta atrás de `Wake` baja a `900` ticks (`30 s`), `grisu-audio` aplica un descenso de velocidad de reproducción (`playback_rate: 1,0 -> 0,35`) y fundido de salida de `450 ms` sobre todas las instancias de `bucket_line_loop`, comunicando acústicamente que la maquinaria de extracción ha muerto y solo queda escapar hacia la superficie.

---

## 6. Mapeo de eventos de simulación (`hush-sim`) a disparos acústicos

El cliente traduce los eventos deterministas emitidos por `hush-sim` al final de cada tick (`30 Hz`) sin acoplar la lógica del juego al motor de audio:

| Evento en `hush-sim` / UI | Sonido (`id`) | Condición / Regla de disparo en el cliente |
|---|---|---|
| Golpe de pico en bloque `2..=6` o `8..=17` | `pick_hit_rock` | Posición 3D del centro del vóxel; si hay un Prospector con geófono en red conectada (`transmits_noise: true`), emite también `geophone_dig` filtrado para ese Prospector. |
| Golpe de pico en bloque `1 dirt` | `pick_hit_dirt` | Posición 3D del vóxel; también genera pulso sísmico `geophone_dig` si existe camino continuo sin `0 air`. |
| Ruptura de bloque (`hardness` agotada) | `block_break` | Se reproduce en la celda destruida al pasar a `0 air`. |
| Construcción de bloque (`8..=17`) | `block_place` | Se reproduce al descontar `ore_cost` y fijar el vóxel en la cuadrícula. |
| Detonación de `15 explosive` | `explosion` + `seismo_tick` | `explosion` en 3D sobre el epicentro; `seismo_tick` en 2D al sumar `+3_000` al Sismógrafo. |
| Pulso en red de `12 electric` | `electric_zap` | Posición 3D del contacto eléctrico; suma `+200` al Sismógrafo. |
| Avance celular de `7 lava` | `lava_bubble` | Posición 3D del frente de lava; suma `+200` al Sismógrafo. |
| Línea activa (`16 bucket_station` / `17 bucket_post`) | `bucket_line_loop` | Bucle 3D continuo en los `3` nodos activos más cercanos; se detiene cuando restan `900` ticks de `Wake`. |
| Desplazamiento sobre suelo sólido | `footstep_1..4` | Selección cíclica pseudoaleatoria cada `10` ticks de marcha (`3` pasos por segundo). |
| Cruce de umbral `25_000` / `50_000` / `75_000` / `100_000` | `seismo_stir` / `seismo_breathe` / `seismo_restless` / `wake_rumble` | Disparo 2D global (`P0`) + *ducking* de `-6,0 dB` + aviso del Vigía (`radio_click_on`, `radio_static_loop`, `radio_click_off`). |
| Preaviso de microderrumbe (`>= 25_000`) | `whistler_1..3` | Disparo 3D en el vóxel de techo inestable antes del desprendimiento. |
| Ofrenda (`100 $`) o Ficha (1–41) en el altar | `offering_accept` / `tally_return` | Disparo 3D en el altar (`-2_500` o `-8_000` al Sismógrafo); si falla la validación, suena `ui_error` (2D). |
| Navegación y acciones rechazadas en HUD | `ui_select` / `ui_back` / `ui_error` | Disparo 2D inmediato para el jugador local sin propagación al resto de la partida. |
