# Especificación de Bloques de HUSHMINE

Este documento define el catálogo canónico de los **18 bloques** (`id: 0..=17`) de **HUSHMINE** según la **Convención A**, junto con su representación en memoria en `hush-sim` y sus reglas deterministas de simulación a **30 Hz**. La fuente de datos autoritativa en formato RON se encuentra en `data/blocks.ron`.

---

## 1. Representación en memoria y metadatos

Para garantizar un rendimiento predecible de caché y ancho de banda de red mínimo, el mundo se divide en **chunks cúbicos de `16³` vóxeles** (`4_096` celdas por chunk):

1. **Matriz densa de tipos (`[u8; 4096]`):**
   Cada celda almacena un único byte (`u8`) correspondiente al identificador estable del bloque (`0..=17`). Los valores `18..=255` son inválidos y son rechazados en la deserialización o validación del estado sin provocar `panic!`.
2. **Separación de identidad y pertenencia:**
   El equipo propietario (`Copper` o `Cobalt`) **nunca** forma parte del `id` del bloque. Tanto un bloque construido por `Copper` como uno construido por `Cobalt` comparten el mismo `id` en la matriz densa (`8..=17`).
3. **Almacenamiento disperso de metadatos por chunk:**
   Dado que la gran mayoría de los vóxeles del mapa son bloques naturales (`0..=7`) sin estado dinámico propio, los atributos adicionales se guardan en una tabla dispersa indexada por la coordenada local del vóxel dentro del chunk (`u16` de `0` a `4_095`, recorrida siempre en orden ascendente de índice para preservar el determinismo):
   - **`owner` (`Team`):** equipo propietario (`Copper` o `Cobalt`) para todos los bloques con `team_owned: true` (`8..=17`).
   - **`energized` (`bool`):** indica si el bloque recibe tensión activa desde un nodo `electric` en el tick actual.
   - **`charge_ticks` (`u16`):** temporizador de cebado en `explosive` o enfriamiento de descarga en redes eléctricas.
   - **`payload` (`u16`):** cantidad de mineral almacenado o en tránsito en nodos logísticos (`bucket_station`, `bucket_post`).

---

## 2. Tabla maestra de los 18 bloques (Convención A)

Todas las durezas están expresadas en **ticks de simulación a 30 Hz** (`30 ticks = 1 segundo`) según la clase que golpea (`Prospector / Miner / Engineer / Sapper`). Un valor de `0` significa que el bloque es inmune al pico. El coste de construcción (`ore_cost`) se abona en unidades de mena (`ore`) del inventario del jugador o reserva local.

| `id` | `name` | Dureza (`P / M / E / S`) | `solid` | `transmits_noise` | `conducts_electricity` | `team_owned` | `ore_cost` |
|---:|---|:---:|:---:|:---:|:---:|:---:|---:|
| `0` | `"air"` | `0 / 0 / 0 / 0` | `false` | `false` | `false` | `false` | `0` |
| `1` | `"dirt"` | `12 / 8 / 12 / 10` | `true` | `true` | `false` | `false` | `0` |
| `2` | `"rock"` | `24 / 15 / 24 / 20` | `true` | `true` | `false` | `false` | `0` |
| `3` | `"hard_rock"` | `90 / 45 / 90 / 60` | `true` | `true` | `false` | `false` | `0` |
| `4` | `"ore"` | `28 / 18 / 28 / 24` | `true` | `true` | `true` | `false` | `0` |
| `5` | `"gold"` | `36 / 24 / 36 / 30` | `true` | `true` | `true` | `false` | `0` |
| `6` | `"diamond"` | `54 / 36 / 54 / 45` | `true` | `true` | `false` | `false` | `0` |
| `7` | `"lava"` | `0 / 0 / 0 / 0` | `false` | `true` | `false` | `false` | `0` |
| `8` | `"team_block"` | `24 / 15 / 18 / 12` | `true` | `true` | `false` | `true` | `1` |
| `9` | `"force_field"` | `45 / 30 / 24 / 15` | `true`* | `true` | `false` | `true` | `4` |
| `10` | `"ladder"` | `15 / 10 / 10 / 8` | `false` | `true` | `false` | `true` | `1` |
| `11` | `"jump_pad"` | `30 / 20 / 15 / 10` | `true` | `true` | `false` | `true` | `6` |
| `12` | `"electric"` | `45 / 30 / 20 / 12` | `true` | `true` | `true` | `true` | `8` |
| `13` | `"rail"` | `18 / 12 / 10 / 8` | `false` | `true` | `true` | `true` | `2` |
| `14` | `"beacon"` | `20 / 20 / 15 / 10` | `true` | `true` | `false` | `true` | `5` |
| `15` | `"explosive"` | `30 / 30 / 24 / 6` | `true` | `true` | `false` | `true` | `10` |
| `16` | `"bucket_station"` | `45 / 35 / 20 / 15` | `true` | `true` | `true` | `true` | `12` |
| `17` | `"bucket_post"` | `30 / 20 / 12 / 8` | `true` | `true` | `true` | `true` | `3` |

*\*Nota sobre `force_field` (`id: 9`): se define con `solid: true` en el catálogo general porque actúa como colisionador rígido ante el equipo rival y fluidos, pero la resolución de movimiento aplica una excepción de permeabilidad para los jugadores aliados del mismo `owner`.*

---

## 3. Reglas detalladas por familia de bloques

### 3.1. Bloques naturales y minerales (`0..=6`)

Constituyen la geología inicial generada a partir de la semilla determinista (`hush_sim::Rng`):

- **`0 "air"`:** espacio vacío respirable y transitable (`solid: false`). Es el único bloque del catálogo con `transmits_noise: false`: actúa como aislante acústico frente al geófono pasivo del Prospector, interrumpiendo la transmisión directa por roca de golpes de pico distantes.
- **`1 "dirt"`:** estrato blando superior y relleno de cavernas. Se excava rápidamente (`8` ticks con el Minero) y sirve para abrir galerías iniciales de bajo riesgo.
- **`2 "rock"`:** matriz pétrea estándar que compone el grueso del mapa subterráneo.
- **`3 "hard_rock"`:** diques basálticos de alta densidad y muros de contención natural alrededor de bolsas de magma. Requiere `45` ticks del Minero (`90` para Prospector/Ingeniero) o el uso de cargas explosivas del Zapador para abrir brechas eficientes.
- **`4 "ore"`:** mena ferrosa. Al extraerse no otorga dinero para el banco del equipo, sino `4` unidades de material de construcción (`ore`) para levantar bloques de los ids `8..=17`. Además, es **conductor eléctrico natural** (`conducts_electricity: true`), por lo que una veta sin extraer puede electrificarse como trampa.
- **`5 "gold"`:** mineral noble de valor medio para el banco (`100 $` por bloque) al depositarse en el banco del equipo o en una estación de cangilones (`"bucket_station"`). Como el `ore`, tiene `conducts_electricity: true`, obligando a los mineros a vigilar si la veta está conectada a un bloque `electric` enemigo antes de tocarla.
- **`6 "diamond"`:** cristal puro situado en cotas profundas y próximas a la lava. Otorga la mayor recompensa de dinero para el banco (`400 $` por bloque, repartidos en `2` gemas de `200 $`). A diferencia del oro y la mena, es un dieléctrico perfecto (`conducts_electricity: false`).

### 3.2. Fluido celular (`7 "lava"`)

- **Inmunidad e intangibilidad:** tiene `hardness: 0` para todas las clases (no puede picarse) y `solid: false` (los cuerpos caen dentro de ella). Cualquier bloque construible colocado directamente para reemplazar una celda de lava la sofoca en esa coordenada si el reglamento de construcción lo permite, o bien la contiene desde las celdas adyacentes.
- **Autómata celular determinista:** el paso de fluido evalúa las celdas de `lava` en intervalos fijos de ticks y en orden estricto de coordenada `(z, y, x)`. Una celda de `lava` intenta propagarse primero hacia abajo (`y - 1`) si la celda destino es `"air"`, y en su defecto hacia los cuatro vecinos ortogonales horizontales (`±x`, `±z`) que sean `"air"` (o bloques frágiles no sólidos como `"ladder"` y `"rail"`, que quedan destruidos al ser inundados).
- **Retroalimentación con el Sismógrafo:** cada vez que la lava avanza e inunda una nueva celda, inyecta **`+200` puntos de ruido** en el Sismógrafo global. Asimismo, los temblores sísmicos de nivel alto fracturan diques de contención y aceleran el drenaje de bolsas de lava hacia las cotas inferiores.
- **Letalidad:** el contacto del volumen de colisión de un jugador con cualquier celda `"lava"` provoca muerte instantánea (o daño letal por tick sin margen de supervivencia prolongada), haciendo caer al suelo o destruyendo el mineral no asegurado.

### 3.3. Bloques estructurales y de movilidad (`8..=11`, `13`)

Todos los bloques construibles (`8..=17`) tienen `team_owned: true` y guardan en metadatos si pertenecen a `Copper` o `Cobalt`:

- **`8 "team_block"` (`ore_cost: 1`):** bloque macizo básico para sellar brechas, construir puentes, levantar parapetos y contener inundaciones de `"lava"`. El Zapador enemigo lo derriba en solo `12` ticks y el Minero en `15`.
- **`9 "force_field"` (`ore_cost: 4`):** barrera energética selectiva. Tiene `solid: true` en la definición física general: bloquea por completo el paso de jugadores del equipo contrario y detiene el flujo de `"lava"`. Sin embargo, el motor de colisiones permite a los jugadores cuyo equipo coincida con el `owner` del bloque atravesarlo libremente como si fuera aire.
- **`10 "ladder"` (`ore_cost: 1`):** estructura de ascenso vertical (`solid: false`). Cualquier jugador cuyo volumen de colisión intersecte una celda `"ladder"` anula su aceleración gravitatoria de caída y puede desplazarse verticalmente a velocidad fija en punto fijo (`Q16.16`) manteniendo salto o avance hacia arriba/abajo.
- **`11 "jump_pad"` (`ore_cost: 6`):** plataforma impulsora sólida (`solid: true`). Cuando un jugador aterriza o camina sobre su cara superior (`+Y`), la simulación sustituye su velocidad vertical `vy` por un impulso ascendente constante en punto fijo (`Q16.16`), permitiendo salvar pozos profundos sin necesidad de columnas de escaleras.
- **`13 "rail"` (`ore_cost: 2`):** carril guía de baja fricción (`solid: false`, `conducts_electricity: true`). Acelera el desplazamiento horizontal de los jugadores que caminan sobre la hilera de raíles y sirve simultáneamente como cableado barato (`2` de mena) para tender líneas eléctricas entre salas.

### 3.4. Red eléctrica, información y demolición (`12`, `14`, `15`)

- **`12 "electric"` (`ore_cost: 8`):** generador de alta tensión (`solid: true`, `conducts_electricity: true`).
  - **Propagación conductiva:** en cada actualización eléctrica, todo bloque `"electric"` energiza por búsqueda en anchura acotada (recorrida en orden canónico de coordenadas) todos los bloques conectados ortogonalmente que tengan `conducts_electricity: true`: `"ore"`, `"gold"`, `"electric"`, `"rail"`, `"bucket_station"` y `"bucket_post"`.
  - **Descarga y ruido:** cuando un jugador del equipo contrario al `owner` de la fuente toca un bloque energizado, sufre una descarga eléctrica dañina que aplica empuje de retroceso y suma **`+200` puntos de ruido** al Sismógrafo global.
- **`14 "beacon"` (`ore_cost: 5`):** baliza luminosa y marcador táctico del equipo (`solid: true`). Permite al Prospector y al Ingeniero señalizar filones ricos, rutas de retorno o peligros en las galerías para sus aliados; además, cuando un equipo supera el `60 %` de la meta y entra en **Fama**, su baliza principal revela su posición.
- **`15 "explosive"` (`ore_cost: 10`):** carga de demolición pesada (`solid: true`).
  - **Detonación y radio:** al activarse y completar su cuenta atrás determinista (o por detonación en cadena desde otra explosión), destruye los bloques dentro de su radio esférico en vóxeles (incluyendo `"hard_rock"` y estructuras enemigas, salvo los límites indestructibles del mapa) y resulta letal para los jugadores cercanos.
  - **Impacto sísmico:** cada detonación inyecta **`+3_000` puntos de ruido** en el Sismógrafo, pudiendo desencadenar derrumbes o fisuras de lava si el acumulador cruza un umbral sísmico.
  - **Desactivación con cizalla por el Zapador:** mientras que el Prospector y el Minero tardan `30` ticks y el Ingeniero `24` ticks en desmontar un `"explosive"`, el **Zapador (`sapper`) tiene una dureza específica de `6` ticks** (`0,2 s` a 30 Hz), lo que representa el corte rápido de mecha/cable con cizalla antes de que la carga detone.

### 3.5. Logística de cangilones (`16`, `17`)

El sistema de transporte automatizado permite mover `"gold"` y `"diamond"` (además de suministros) desde el frente de excavación hasta el banco de la superficie o una estación receptora (`"bucket_station"`) sin acarreo manual:

- **`16 "bucket_station"` (`ore_cost: 12`):** estación terminal de carga y descarga (`solid: true`, `conducts_electricity: true`). Los mineros depositan mineral en una estación emisora y ésta lo despacha a través de la línea hacia la estación receptora.
- **`17 "bucket_post"` (`ore_cost: 3`):** poste de soporte intermedio (`solid: true`, `conducts_electricity: true`). Conecta tramos rectilíneos entre dos nodos de la red de cangilones dentro del alcance máximo de vano permitido por la simulación.
- **Coste sísmico continuo y vulnerabilidad:**
  - Cada línea de cangilones activa que esté transportando carga genera **`+1` punto de ruido por tick** (`+30` por segundo) en el Sismógrafo global, además de propagar vibraciones locales (`transmits_noise: true`) detectables por el geófono del Prospector rival.
  - Como tanto `"bucket_station"` como `"bucket_post"` tienen `conducts_electricity: true`, un Zapador o Ingeniero enemigo puede adosar un bloque `"electric"` a cualquier poste de la línea para electrificar toda la infraestructura logística rival.
