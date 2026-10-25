# Mundo grande · Datos de generación (W1)

Los cinco archivos de `data/worldgen/` son la fuente única de verdad de la
generación del mundo para el hito W1. Son **dato, no código**: todo entero
(sin un solo número con punto decimal), listo para que el motor los lea con el
parser RON y para que las herramientas Python los validen. Este documento
describe su esquema, sus invariantes y las decisiones de datos que se tomaron.

| Archivo | Struct raíz | Contenido |
|---|---|---|
| `mundo.ron` | `WorldGen` | Extensión horizontal, rango vertical, capas y macizo del Durmiente |
| `relieve.ron` | `Relief` | Ruido 2D por columna, curvas de altura y pendiente, umbral de ríos |
| `biomas.ron` | `Biomes` | 8 biomas de superficie y 9 subterráneos |
| `cuevas.ron` | `Caves` | Márgenes y 7 excavadores de cuevas |
| `menas.ron` | `Ores` | 11 materiales con curva peso/cota |

## Cómo validar

Los tres comandos usan solo la biblioteca estándar de Python y deben salir con
código 0:

```bash
python tools/worldgen/check_worldgen.py        # verificador: OK o lista de errores
python tools/worldgen/preview_curves.py        # escribe out/worldgen_curves.png
python -m unittest discover -s tools/worldgen -v   # batería de pruebas (13 casos)
```

El verificador imprime `OK: ...` o cada error en español y termina con código 1
si algo falla. La previsualización genera un PNG con tres paneles (curvas de
relieve, franja de capas a escala real y curvas de menas) que conviene abrir y
mirar tras cualquier cambio de datos. La CI ejecuta el descubrimiento de
`unittest` sobre cada carpeta de `tools/` que contenga `test_*.py`.

## Unidades y convenciones

- **Bloques:** todas las cotas `y` y las distancias horizontales son bloques
  enteros. El eje Y crece hacia arriba; el nivel del mar es `y = 0`.
- **Q16 (punto fijo 16.16 con signo):** `65_536` equivale a `1.0` y
  `-65_536` a `-1.0`. La salida del ruido de gradiente vive en
  `[-65_536, 65_536]` y los umbrales sobre ruido en `[0, 1]` viven en
  `[0, 65_536]`.
- **Milésimas:** la pendiente del relieve se expresa en milésimas de densidad
  por bloque (`950` = `0,95` bloques de densidad por bloque de desnivel).
- **Luz:** `base_light` usa la escala de 4 bits del motor, `0..=15`
  (`0` = oscuridad total).

## mundo.ron — `WorldGen`

| Campo | Unidad | Valor W1 | Significado |
|---|---|---|---|
| `worldgen_version` | — | `1` | Versión del esquema de datos |
| `x_min` / `x_max` | bloques | `-7_500_000` / `7_499_999` | Mundo de 15 M de lado |
| `z_min` / `z_max` | bloques | `-7_500_000` / `7_499_999` | Ídem en Z |
| `y_min` / `y_max` | bloques | `-4_096` / `1_023` | De las raíces al cielo |
| `border_fog_warning_blocks` | bloques | `64` | Aviso de niebla antes del muro (INV-W04) |
| `layers[]` | bloques | 7 capas | `name`, `y_min`, `y_max`, `description` |
| `sleeper_massif` | bloques | radio 3000, pico 900, spawn 2000..5000 | Macizo del Durmiente Mayor |

Capas en orden: `sky` (600..1023), `surface` (-40..599), `crust` (-256..-41),
`mantle` (-640..-257), `vault` (-3072..-641), `roots` (-4095..-3073) y
`bedrock` (-4096..-4096).

**Invariantes** (los comprueba `check_worldgen.py`): valores exactos de las
cotas y la niebla; las 7 capas con nombres exactos y en orden; partición
estricta de `[-4096, 1023]` sin huecos ni solapes
(`layers[i+1].y_min == layers[i].y_max + 1`); `sleeper_massif` con sus cuatro
valores exactos.

**Decisión de datos:** la tabla del doc §1 dejaba la banda `-40..0` ambigua
(Superficie y Corteza la reclamaban a la vez). La partición la asigna a
`surface` (-40..599), y la corteza queda delimitada por -256..-41. Ninguna
otra capa comparte cotas con su vecina.

## relieve.ron — `Relief`

| Campo | Unidad | Significado |
|---|---|---|
| `noise_params[]` | — | 6 parámetros 2D por columna: `name`, `period_shift` (celda de `2^K` bloques), `octaves` de fBm |
| `base_height_curve[]` | Q16 → bloques | Altura base por índice combinado, por tramos lineales |
| `slope_factor_curve[]` | Q16 → milésimas | Pendiente (densidad/bloque) por el mismo índice |
| `river_threshold_q16` | Q16 | Umbral de río sobre ruido en cresta (`58_982` ≈ 0,9) |

Los seis ruidos y su orden: `continentality` (period 12), `erosion` (11),
`peaks_valleys` (9), `temperature` (13), `humidity` (12), `weirdness` (10).

`input_q16` **no es un parámetro suelto** sino el índice combinado
normalizado al rango Q16 `[-65_536, 65_536]`: una mezcla de continentalidad,
erosión y picos y valles. Por eso `base_height_curve` no es monótona (mares
profundos en el extremo negativo, valle de llanuras hacia el centro,
montañas hacia el positivo); la monotonía exigida es la del propio índice.

**Invariantes:** 6 nombres exactos en orden con `period_shift`/`octaves`
enteros positivos; en las dos curvas, primera entrada `-65_536`, última
`65_536`, entradas estrictamente crecientes y salidas dentro de `[-64, 950]`;
`river_threshold_q16` en `[0, 65_536]`.

## biomas.ron — `Biomes`

**Superficie (8):** `biome_id` 0..=7, `name`, `center` con los cinco ejes de
clima en Q16 (`continentality_q16`, `erosion_q16`, `temperature_q16`,
`humidity_q16`, `weirdness_q16`, todos en `[-65_536, 65_536]`),
`surface_block`, `subsurface_block` y `description`. La selección elige el
bioma con menor distancia entera al cuadrado en el espacio de 5 ejes.
Orden fijo: `cordillera`, `needle_forest`, `highland`, `red_desert`,
`badlands`, `river_valley`, `tundra`, `inland_sea`.

**Subsuelo (9):** `biome_id` 0..=8, `name`, `layer` (nombre de capa de
`mundo.ron`), `y_min`/`y_max` en bloques, `dominant_block`, `base_light`
(0..=15) y `description`. Dentro de una capa el reparto es por ruido celular
3D, así que cada bioma cubre el rango completo de su capa. Orden fijo:
`limestone_caves`, `wet_caves`, `clay_galleries`, `basalt_caverns`,
`geodes_quartz`, `firedamp_grottos`, `glow_fungi`, `the_vault`,
`dreamed_veins`.

**Invariantes:** los 8 y los 9 nombres exactos en orden con ids únicos que
cubren sus rangos; centros Q16 en rango; `base_light` en `[0, 15]`; todo id
de bloque en `0..=17`; cruce con `mundo.ron`: cada `layer` existe y
`[y_min, y_max]` del bioma cae dentro del rango de esa capa.

**Decisión de datos (Convención A):** el catálogo de bloques aún no tiene
arena, nieve, grava, arcilla ni agua, así que `surface_block` /
`subsurface_block` son marcadores provisionales (`1` dirt, `2` rock,
`3` hard_rock); el material real de cada bioma queda anotado en su comentario
del `.ron` y el arte final fijará los ids definitivos.

## cuevas.ron — `Caves`

| Campo | Unidad | Valor W1 | Significado |
|---|---|---|---|
| `bedrock_margin` | bloques | `4` | Ningún excavador baja de `y_min + margen` |
| `river_roof_min` | bloques | `2` | Techo mínimo bajo el lecho de los ríos |
| `carvers[]` | — | 7 excavadores | `name`, `method`, `y_min`, `y_max`, `scale_shift`, `threshold_q16`, `description` |

`scale_shift` K hace que la celda del ruido mida `2^K` bloques por eje;
`threshold_q16` es un umbral sobre ruido en `[0, 1]` (65_536 = 1.0). Los
siete, en orden: `cheese`, `spaghetti`, `noodles`, `bounded_worms`,
`sinkholes`, `canyons`, `giant_chambers`.

**Invariantes:** márgenes exactos 4 y 2; los 7 nombres en orden; por
excavador, `y_min >= -4_096 + 4 = -4_092`, `y_max <= 1_023`, `y_min <= y_max`
y `threshold_q16` en `[0, 65_536]`; `giant_chambers` confinado a
`[-640, -450]` porque los lagos de lava viven por debajo de -450 (doc §7).

## menas.ron — `Ores`

| Campo | Unidad | Significado |
|---|---|---|
| `note` | — | Nota en inglés para herramientas (texto fuente) |
| `materials[]` | — | 11 materiales: `material`, `block_id`, `ore_shape`, `curve[]` |
| `curve[]` | bloques → peso | Puntos `(y, weight_per_100k)`, por tramos lineales |

`weight_per_100k` es el peso por cada 100 000 bloques en esa cota; entre dos
puntos se interpola linealmente. Orden fijo: `coal`, `copper`, `tin`, `iron`,
`tungsten`, `silver_lead`, `cobalt`, `gold`, `diamond`, `sulfur`, `quartz`.

**Invariantes:** los 11 materiales en orden; `block_id` en `0..=17`; curva con
`y` estrictamente creciente dentro de `[-4_096, 1_023]`; pesos enteros `>= 0`;
primer `y <= -256` y último `y >= -256`; y si el último `y` es negativo, su
peso debe ser `0`.

**Decisiones de datos:**

- **Bloques provisionales (Convención A):** mientras no haya bloques
  dedicados, `block_id 4` es la mena genérica para `coal`, `copper`, `tin`,
  `iron`, `tungsten`, `silver_lead`, `cobalt`, `sulfur` y `quartz`, que se
  distinguen por el nombre del material de supervivencia. `gold` usa `5` y
  `diamond` usa `6`.
- **Menas del manto:** `tungsten`, `cobalt`, `diamond`, `sulfur` y `quartz`
  cierran su curva en `(-256, 0)`: por encima del manto su peso es nulo. La
  comprobación "último `y < 0` implica peso `0`" lo garantiza.
- INV-W07 medirá la distribución real en miles de chunks de muestra y la
  comparará con estas curvas.

## Resumen de decisiones de datos

1. Partición estricta de capas; la banda ambigua `-40..0` se asigna a
   `surface` (la corteza queda en -256..-41).
2. `slope_factor_curve` se expresa en milésimas de densidad por bloque
   (950 = 0,95), no en bloques.
3. Bloques provisionales de la Convención A para superficie (dirt/rock/
   hard_rock como marcadores).
4. `block_id 4` = mena genérica mientras no haya bloques dedicados;
   `gold = 5`, `diamond = 6`.
5. Las menas del manto cierran su curva en `(-256, 0)`.
6. Umbrales Q16 de los excavadores sobre ruido en `[0, 1]`
   (65_536 = 1.0); `giant_chambers` no baja de -450 ni sube de -640.

## Referencias

- `docs/mundo/01-escala-y-motor.md` §1 (dimensiones y capas verticales) y §5
  (luz en 4 bits).
- `docs/mundo/02-generacion.md` §3 (relieve), §4 (biomas de superficie),
  §5 (biomas subterráneos), §6 (cuevas), §7 (acuíferos, ríos y lava) y
  §8 (litología y menas).
- Herramientas: `tools/worldgen/check_worldgen.py` (verificador),
  `tools/worldgen/preview_curves.py` (previsualización) y
  `tools/worldgen/test_worldgen.py` (pruebas).
