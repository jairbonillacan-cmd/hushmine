# Catálogo de escenas deterministas de captura (`tests/scenes/`)

Este documento especifica el formato y el contenido de las siete escenas canónicas de captura visual situadas en `tests/scenes/*.ron`. Estas escenas alimentan el subcomando sin ventana `hush-cli shot --scene <ruta> --out <png>` a partir del Hito 1.

---

## 1. Propósito en el bucle de verificación sin ventana

En HUSHMINE / Grisú, toda funcionalidad gráfica del motor (`grisu-render`) debe poder verificarse de forma determinista y sin necesidad de abrir una ventana interactiva (`winit`). El flujo de trabajo se apoya en tres pilares:

1. **Reproducibilidad exacta:** cada archivo `tests/scenes/<nombre>.ron` fija las dimensiones del volumen, la pose de cámara en coordenadas enteras y de punto fijo (`Q16.16`) y la disposición exacta de cada bloque y metadato de equipo (`Copper` o `Cobalt`).
2. **Inspección visual por agentes y desarrolladores:** tras ejecutar `cargo run -p hush-cli -- shot --scene tests/scenes/<nombre>.ron --out out/<nombre>.png`, el agente abre el archivo PNG resultante para comprobar que el mallado (*greedy meshing*), el atlas procedural, la oclusión ambiental por vértice, la iluminación celular BFS y la niebla de distancia se comportan según el diseño artístico e industrial del juego.
3. **Cobertura completa del catálogo de bloques (Convención A, `0..=17`):** el conjunto de las siete escenas ejercita los 18 tipos de bloque del catálogo maestro (`data/blocks.ron`), incluyendo terrenos naturales, minerales de banco, fluidos letales y construcciones de ambos equipos con siluetas diferenciadas para daltonismo.

---

## 2. Especificación del formato RON (`SceneDef`)

Los archivos de escena emplean el subconjunto estricto de RON validado por `tools/balance/ron_lite.py`.

### Regla de determinismo numérico

`tools/balance/ron_lite.py` **prohíbe cualquier literal con punto decimal** (como `1.5` o `0.5`). Todos los valores numéricos de una escena son enteros puros:

- Las coordenadas de celdas de vóxel (`min`, `max`, `pos`, `position_blocks`, `look_at_blocks`) se expresan en bloques enteros `(x, y, z)` dentro de `0..width-1`, `0..height-1`, `0..depth-1`.
- La posición subbloque de la cámara (`position_subblocks`) se expresa en punto fijo `Q16.16`, donde `1 bloque = 65_536` unidades (`position_subblocks = position_blocks * 65_536`).
- Los ángulos de orientación y apertura de cámara (`yaw_deg`, `pitch_deg`, `fov_deg`) se expresan en grados enteros.

### Estructura de datos

- **`SceneDef`**: nodo raíz del archivo.
  - `name`: identificador canónico de la escena (coincide con el nombre del archivo sin extensión).
  - `description`: descripción en español de los elementos visuales, geométricos y lumínicos que deben apreciarse en la captura.
  - `dimensions`: `Dimensions(width: u32, height: u32, depth: u32)`.
  - `camera`: `CameraPose` con `position_blocks`, `position_subblocks`, `look_at_blocks`, `yaw_deg`, `pitch_deg` y `fov_deg`. La celda indicada por `position_blocks` debe quedar ocupada por aire (`0`, `"air"`) tras construir la escena.
  - `regions`: lista ordenada de `BlockRegion(min, max, block_id, block_name, team)`. Cada región rellena el paralelepípedo inclusivo `min..=max`.
  - `blocks`: lista ordenada de `BlockPlacement(pos, block_id, block_name, team)` para colocaciones puntuales de detalle.

### Orden de aplicación y metadatos de equipo

El cargador de escenas inicializa el volumen en aire (`0`) y aplica las operaciones en este orden estricto:

1. Se recorren las entradas de `regions` en el orden exacto de la lista; cada `BlockRegion` sobrescribe las celdas del rango inclusivo `[min.x..=max.x] × [min.y..=max.y] × [min.z..=max.z]`.
2. Se recorren las entradas de `blocks` en el orden de la lista; cada `BlockPlacement` sobrescribe la celda `pos`.

El campo `team` sigue la **Convención A**:

- `None` para bloques naturales y neutros (`0..=7`: `air`, `dirt`, `rock`, `hard_rock`, `ore`, `gold`, `diamond`, `lava`).
- `Some("copper")` o `Some("cobalt")` para bloques propiedad de un equipo (`8..=17`: `team_block`, `force_field`, `ladder`, `jump_pad`, `electric`, `rail`, `beacon`, `explosive`, `bucket_station`, `bucket_post`).

---

## 3. Catálogo de las 7 escenas canónicas

### Tabla resumen

| Escena | Archivo | Dimensiones (`W×H×D`) | Cámara `position_blocks` (`Q16.16`) | `look_at_blocks` | `yaw` / `pitch` / `fov` | Bloques probados (`id: nombre`) |
|---|---|---|---|---|---|---|
| `tunel_lava` | `tests/scenes/tunel_lava.ron` | `16×16×16` | `(6, 6, 3)` (`393_216, 393_216, 196_608`) | `(7, 4, 12)` | `6°` / `-12°` / `70°` | `0: air`, `2: rock`, `3: hard_rock`, `4: ore`, `7: lava`, `8: team_block` (Copper), `14: beacon` (Copper) |
| `veta_oro` | `tests/scenes/veta_oro.ron` | `16×16×16` | `(7, 6, 3)` (`458_752, 393_216, 196_608`) | `(8, 5, 10)` | `8°` / `-8°` / `70°` | `0: air`, `1: dirt`, `2: rock`, `4: ore`, `5: gold`, `6: diamond`, `10: ladder` (Copper), `14: beacon` (Copper) |
| `base_copper` | `tests/scenes/base_copper.ron` | `16×16×16` | `(8, 7, 3)` (`524_288, 458_752, 196_608`) | `(8, 4, 11)` | `0°` / `-18°` / `70°` | `0: air`, `1: dirt`, `2: rock`, `8: team_block`, `10: ladder`, `11: jump_pad`, `14: beacon`, `16: bucket_station`, `17: bucket_post` (todos Copper) |
| `base_cobalt` | `tests/scenes/base_cobalt.ron` | `16×16×16` | `(8, 7, 3)` (`524_288, 458_752, 196_608`) | `(8, 4, 11)` | `0°` / `-18°` / `70°` | `0: air`, `1: dirt`, `2: rock`, `8: team_block`, `9: force_field`, `11: jump_pad`, `13: rail`, `14: beacon`, `16: bucket_station`, `17: bucket_post` (todos Cobalt) |
| `campo_fuerza` | `tests/scenes/campo_fuerza.ron` | `16×16×16` | `(7, 7, 3)` (`458_752, 458_752, 196_608`) | `(7, 5, 9)` | `0°` / `-16°` / `70°` | `0: air`, `2: rock`, `3: hard_rock`, `4: ore`, `8: team_block`, `9: force_field`, `12: electric`, `14: beacon` (Copper/Cobalt), `15: explosive` (Cobalt) |
| `cangilones` | `tests/scenes/cangilones.ron` | `16×16×16` | `(7, 6, 3)` (`458_752, 393_216, 196_608`) | `(8, 5, 11)` | `6°` / `-7°` / `70°` | `0: air`, `2: rock`, `4: ore`, `5: gold`, `8: team_block`, `13: rail`, `14: beacon`, `16: bucket_station`, `17: bucket_post` (Cobalt) |
| `cueva_oscura` | `tests/scenes/cueva_oscura.ron` | `24×16×24` | `(12, 6, 3)` (`786_432, 393_216, 196_608`) | `(11, 5, 16)` | `-4°` / `-5°` / `70°` | `0: air`, `2: rock`, `3: hard_rock`, `6: diamond`, `7: lava`, `14: beacon` (Copper) |

---

### Detalle y criterios de inspección visual por escena

#### 1. `tunel_lava` (`tests/scenes/tunel_lava.ron`)
- **Composición:** galería longitudinal excavada sobre un zócalo de `hard_rock` (`3`) y bóveda de `rock` (`2`). A la derecha discurre un canal de `lava` (`7`) contenido por un pretil de `team_block` (`8`, equipo `Copper`) con bolardos. En el hastial izquierdo aflora una veta de `ore` (`4`), y sobre la calzada descansa una `beacon` (`14`) de `Copper`.
- **Criterio de inspección visual:** verificar la emisión lumínica naranja-rojiza de la lava combinada con el tono cálido de la baliza `Copper`, el contraste de textura entre `rock` y `hard_rock`, y el bisel remachado de `team_block` de `Copper`.

#### 2. `veta_oro` (`tests/scenes/veta_oro.ron`)
- **Composición:** cámara amplia que corta el contacto entre el estrato superior de `dirt` (`1`) y la matriz de `rock` (`2`), dejando al descubierto la Veta Madre: un macizo frontal y lateral de `gold` (`5`) salpicado de cristales de `diamond` (`6`) y bordeado de `ore` (`4`). Una `ladder` (`10`) asciende por la pared izquierda y una `beacon` (`14`) ilumina el frente.
- **Criterio de inspección visual:** comprobar que el oro (`5`) y el diamante (`6`) se distinguen de inmediato por color y patrón geométrico de facetas en el atlas procedural, y que los travesaños de la escalera (`10`) se leen con claridad sobre la roca.

#### 3. `base_copper` (`tests/scenes/base_copper.ron`)
- **Composición:** sala principal de la base `Copper` con losa y pilares estructurales de `team_block` (`8`), estación receptora `bucket_station` (`16`) escoltada por dos `bucket_post` (`17`) y tres `beacon` (`14`), una `jump_pad` (`11`) en el flanco izquierdo y un pozo vertical con `ladder` (`10`) a la derecha.
- **Criterio de inspección visual:** validar la paleta cálida cobriza y el motivo industrial biselado propio de `Copper`, así como la silueta reconocible de la estación de cangilones y la plataforma de salto.

#### 4. `base_cobalt` (`tests/scenes/base_cobalt.ron`)
- **Composición:** sala simétrica del equipo `Cobalt` con pavimento y nervios verticales de `team_block` (`8`), `bucket_station` (`16`) central con `bucket_post` (`17`), compuerta trasera sellada por una pantalla de `force_field` (`9`), línea rápida de `rail` (`13`) a la izquierda y `jump_pad` (`11`) a la derecha.
- **Criterio de inspección visual:** confirmar que la mampostería `Cobalt` presenta paneles ortogonales y tono azul cobalto que la diferencian de `base_copper` incluso en escala de grises (prueba de daltonismo), además de la transparencia o rejilla energética del `force_field`.

#### 5. `campo_fuerza` (`tests/scenes/campo_fuerza.ron`)
- **Composición:** intersección de dos galerías sobre un foso profundo tapizado de bloques `electric` (`12`) alimentados por `ore` (`4`). Un puente estrecho de `team_block` (`8`, `Copper`) cruza el foso y queda cerrado por un `force_field` (`9`, `Copper`). Junto a la jamba izquierda del puente hay un `explosive` (`15`, `Cobalt`) listo para demoler la barrera.
- **Criterio de inspección visual:** evaluar la lectura táctica inmediata del peligro: el brillo eléctrico azulado en el fondo del foso, la barrera de energía `Copper` sobre el puente y el bloque de carga explosiva `Cobalt` adosado al marco.

#### 6. `cangilones` (`tests/scenes/cangilones.ron`)
- **Composición:** galería de extracción en dos niveles conectados por una terraza de `team_block` (`8`, `Cobalt`). Una fila de `bucket_post` (`17`) sube desde la galería inferior hasta la `bucket_station` (`16`) superior, acompañada en paralelo por una vía de `rail` (`13`) y flanqueada por filones de `ore` (`4`) y `gold` (`5`).
- **Criterio de inspección visual:** comprobar la alineación visual de los postes de soporte (`17`) hacia la estación (`16`) y la continuidad del carril (`13`) en el desnivel.

#### 7. `cueva_oscura` (`tests/scenes/cueva_oscura.ron`)
- **Composición:** caverna profunda de `24×16×24` excavada en `hard_rock` (`3`) y `rock` (`2`) con recodos laterales, un pilar macizo en medio de la bóveda, una única `beacon` (`14`) en el suelo, una grieta profunda de `lava` (`7`) al fondo y racimos de `diamond` (`6`) incrustados en el techo.
- **Criterio de inspección visual:** escena de referencia para auditar el oscurecimiento de esquinas por oclusión ambiental (AO), la atenuación gradual de la luz BFS alrededor de la baliza y la grieta de lava, y la transición suave hacia la niebla de distancia en el fondo de la caverna.
