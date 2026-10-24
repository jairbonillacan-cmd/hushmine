# Mundo grande · 1. Escala y motor

Cómo tiene que estar hecho Grisú para que un mundo de 15 M × 15 M bloques quepa en un portátil con gráficos integrados. Índice en [README.md](README.md).

---

## 1. Dimensiones y capas verticales

| Eje | Rango | Tamaño |
|---|---|---|
| X (Oeste → Este) | −7 500 000 … 7 499 999 | 15 000 000 bloques |
| Z (Norte → Sur) | −7 500 000 … 7 499 999 | 15 000 000 bloques |
| Y (abajo → arriba) | −4 096 … 1 023 | 5 120 bloques |

Un bloque mide un metro. `y = 0` es el nivel del pozo donde despiertan los Llegados ([lore del modo Historia](../supervivencia/01-lore.md)).

| Capa | Y | Papel en el juego |
|---|---|---|
| Cielo | 600 … 1 023 | Solo los picos más altos. Por encima, la piedra del **Párpado**, que no se genera: se llega a su costura con la Escala ([dimensiones](../supervivencia/06-dimensiones.md)) |
| Superficie | ~−40 … 600 | Biomas, pueblos, castilletes, bocaminas. El macizo del Durmiente Mayor llega a ~900 |
| Corteza | 0 … −256 | Cuevas de caliza, acuíferos, minas de los Antiguos, carbón, cobre, hierro |
| Manto | −256 … −640 | Basalto, geodas, ríos de lava, bolsas de grisú, oro profundo. **El Corazón del Durmiente Mayor, en torno a −600** |
| La Bóveda | −640 … −3 072 | Vacío hueco de kilómetros bajo el Corazón. Pilares, islas de roca, cristal |
| Las Raíces | −3 072 … −4 096 | Las raíces de los Durmientes. Contenido final |
| Lecho | −4 096 | Roca indestructible |

Las cotas viven en `data/worldgen/mundo.ron`, no en el código.

**Borde del mundo:** columna de roca indestructible a partir de `|x|` o `|z|` = 7 500 000, y niebla que avisa desde 64 bloques antes. Ninguna estructura cruza el borde (`INV-W04`).

## 2. Coordenadas y punto fijo

Este es el punto delicado y **se decide en el hito 1** (W0).

| Magnitud | Tipo | Rango | Comentario |
|---|---|---|---|
| Posición de bloque | `i32` × 3 | ±2 147 483 647 | Sobra |
| Posición de chunk | `i32` × 3 (bloque `>> 4`) | ±134 217 727 | El desplazamiento aritmético redondea hacia −∞, que es lo correcto con negativos. Local: `& 15` |
| Posición de entidad | **`i64` con 16 bits fraccionarios** (Q47.16) | ±1,4·10¹⁴ bloques | **Q16.16 sobre `i32` no basta: solo llega a ±32 767 bloques** |
| Velocidad, aceleración, ángulos | Q16.16 (`hush_sim::Fixed`) | ±32 767 por tick | Se queda como está |
| Región de guardado | `i32` × 3 (chunk `>> 5`) | — | 32³ chunks por archivo |

**Consecuencia para el hito 1:** la tarea 3 (punto fijo) define también el tipo de posición mundial en `i64`, y la tarea 9 (colisión) lo usa desde el principio. En los mapas de 64³ no cambia nada visible. Mezclarlo después en todo el código de colisión, red y repeticiones sería caro.

**Coordenadas negativas:** todas las pruebas de chunk, raycast DDA y colisión incluyen casos en `x = −1`, `x = −16`, `x = −17` y en los extremos ±7 499 999. Es la clase de error que no aparece en un mapa de 64³ con coordenadas positivas.

## 3. Chunks con paleta e ids `u16`

El chunk de 16³ con un byte por bloque del hito 1 sirve para el modo Turno, pero la supervivencia necesita cientos de bloques (menas, madera, plantas, estructuras, piezas orientadas).

| Representación | Cuándo | Tamaño |
|---|---|---|
| **Uniforme** | Todo el chunk es un solo bloque (aire de la Bóveda, roca maciza) | 2 bytes |
| **Paleta de 1, 2, 4 u 8 bits** | El caso normal: 2–256 bloques distintos | 512 B – 4 KB + paleta |
| **Directa `u16`** | Más de 256 bloques distintos (raro) | 8 KB |

- **`BlockId(u16)`** con los ids `0..=17` reservados para la Convención A. Las escenas, `data/blocks.ron` y la red del modo Turno siguen funcionando sin cambios.
- **Metadatos dispersos** como ahora: equipo, orientación, nivel de fluido, contenido de un cofre.
- **Estimación:** un chunk subterráneo típico usa 6–12 bloques distintos, así que ocupa unos 2 KB con 4 bits por bloque. Más de la mitad de los chunks profundos son uniformes.

**W0:** la tarea 2 del hito 1 expone el chunk a través de una interfaz (`get`, `set`, `fill`, recorrido por secciones) y no de un `[u8; 4096]` público. La primera implementación puede seguir siendo un `u8` plano; cambiar a la paleta después será un cambio interno.

## 4. Streaming: cargar lo que se ve

### Planificador

- Cada jugador tiene un **radio de interés**: horizontal R y vertical V, en chunks. El servidor une los radios de todos los jugadores.
- **Prioridad:** distancia, si está en el frustum, si es aire visible (conectividad de cuevas) y dirección de movimiento.
- **Estados de un chunk:** `Columna → Base → Decorado → Iluminado → Mallado`. Las fases del generador están en [02-generacion.md](02-generacion.md).
- **Trabajadores:** un grupo de hilos (`rayon` o hilos propios con cola) para generar y mallar. En el T490s se dejan libres el hilo principal y el de render: 3 trabajadores.
- **Caché LRU** de chunks en estado `Base` alrededor del borde cargado, para que la fase de decoración no repita trabajo.
- **Histéresis:** un chunk se descarga cuando sale de R + 2, no de R, para no regenerar al ir y volver.

### Presets de distancia

| Preset | R (horizontal) | V (vertical) | Chunks cargados | Bloques en RAM (~2,5 KB/chunk) |
|---|---|---|---|---|
| Bajo (T490s, navegador) | 8 | ±5 | 17 × 17 × 11 = 3 179 | ~8 MB |
| Medio | 12 | ±6 | 25 × 25 × 13 = 8 125 | ~20 MB |
| Alto | 16 | ±8 | 33 × 33 × 17 = 18 513 | ~46 MB |

Bajo tierra, la oscuridad y la niebla son parte de la estética. **Un radio corto no es una concesión sino el ambiente**, y es la gran ventaja de un juego de minas frente a uno de superficie.

### ¿Llegan los trabajadores?

Caso peor razonable en el preset Bajo: vagoneta a 16 bloques/s (1 chunk/s) genera una cara nueva de 17 × 11 = 187 chunks por segundo. Con 1 ms por chunk son 0,19 núcleos. Cayendo por un pozo a 50 bloques/s (3 chunks/s) salen 17 × 17 × 3 = 867 chunks/s: 0,87 núcleos de generación más el mallado. **Cabe en tres trabajadores** si se cumple el presupuesto de 1 ms. El planificador puede limitar la velocidad máxima de caída o cargar por adelantado los pozos.

### Lo que hay en la superficie lejana

En la superficie, un radio de 128–256 bloques se queda corto para ver montañas. Hay un **horizonte de bajo detalle** que no usa chunks: una malla en anillos (resolución de 16, 64 y 256 bloques) generada directamente desde la altura 2D del generador, hasta 2–4 km. Es barata porque no evalúa cuevas ni estructuras. No se dibuja bajo tierra.

## 5. Luz en un mundo de chunks cúbicos

- **Luz de bloque** (lava, balizas, lámparas): BFS como en el hito 1, más una cola de **bordes pendientes**: al cargar un vecino, la luz cruza la frontera y se propaga.
- **Luz de cielo:** no hace falta cargar la columna entera. El generador conoce de forma analítica la altura de la superficie de cada columna. Se guarda un **mapa de alturas por columna** de 16 × 16 que se corrige al cavar o construir. Por debajo de la superficie, la luz de cielo solo entra por aberturas, con un BFS limitado.
- **Luz de bloque en color (RGB, 4 bits por canal) y cuatro bits de cielo:** 16 bits por bloque, solo en chunks no uniformes y cercanos (`docs/graficos/04-shaders.md`).

## 6. Render relativo a la cámara

A 7,5 millones de bloques, un `f32` tiene una precisión de medio bloque y los vértices temblarían.

- La CPU calcula `(chunk − chunk_de_cámara)` en enteros y le suma la parte fraccionaria de la cámara. Al shader solo llegan números pequeños (cientos de bloques), con precisión de sobra.
- Las caras empaquetadas de 8 bytes ya son locales al chunk. Solo cambia el desplazamiento por chunk que se pasa al shader.
- **W0:** la tarea 13 del hito 1 recibe el origen de cada chunk relativo a la cámara desde el primer día.

**Recorte:** frustum y recorte por conectividad de cuevas (ya previstos en el plan). Bajo tierra, la conectividad elimina la mayor parte de lo cargado.

## 7. Simulación activa

- **Zona activa:** a 6–8 chunks de cada jugador se simulan fluidos, plantas, criaturas y máquinas. Fuera de ella, el chunk está congelado.
- **Fluidos estables al generarse:** el generador nunca deja agua ni lava que pueda fluir en el tick 0 (`INV-W05`). Así un chunk recién cargado no provoca una avalancha de actualizaciones.
- Los fluidos y la caída de bloques usan las mismas reglas enteras y deterministas que `hush-sim`.

## 8. Guardado

- **Archivos de región** de 32³ chunks: `regiones/r.{x}.{y}.{z}.hreg`, con una tabla de índices y bloques comprimidos con LZ4 (`lz4_flex`, ya previsto en el plan).
- **Solo se guardan los chunks modificados.** Los demás se regeneran desde la semilla.
- **Escritura atómica:** archivo temporal y renombrado. Un corte de luz no corrompe el mundo.
- **Versión del generador:** cada mundo y cada región guardan `worldgen_version`. Si el generador cambia, el comando `hush-cli world freeze` guarda completos los chunks ya visitados con la versión vieja antes de actualizar. Así no aparecen costuras en zonas exploradas. Las pruebas de instantánea (`INV-W11`) evitan cambios de versión por accidente.
- **Entidades, inventarios y progreso de la historia** van aparte, en `rusqlite` (ya previsto para el servidor).

## 9. Red

El modo Turno envía el mapa completo al unirse. En el mundo grande no se puede.

| Opción | Ancho de banda | Trampas (rayos X) |
|---|---|---|
| El servidor envía chunks completos | ~1–2 KB por chunk: ~190 KB/s en vagoneta | Hay que ocultar menas en el servidor |
| **El cliente genera y el servidor envía diferencias** | Diferencias + revelaciones: pocos KB/s | El cliente genera **con las menas ocultas** y el servidor revela las que quedan al aire en su radio |

**Recomendación:** la segunda. Solo funciona porque el generador es determinista byte a byte en nativo y en WebAssembly, otra razón para usar enteros. Al unirse, el cliente comprueba la semilla y `worldgen_version` contra un resumen del servidor.

- **Un jugador:** el servidor local en memoria hace todo. No hay red.
- **Navegador:** los hilos de WebAssembly exigen las cabeceras COOP/COEP. Sin ellas se usa un solo trabajador y el preset Bajo.

## 10. Presupuestos (por validar)

| Escenario | Objetivo |
|---|---|
| Generación de un chunk (release, un núcleo, T490s) | p50 ≤ 1 ms, p95 ≤ 3 ms |
| Mallado de un chunk | p50 ≤ 0,5 ms |
| Fotograma moviéndose en vagoneta (T490s, preset Bajo) | p95 ≤ 16,7 ms, sin tirones por carga |
| Memoria del cliente (preset Medio) | ≤ 400 MB |
| Servidor de supervivencia, 16 jugadores dispersos | ≤ 1,5 GB de RAM; p95 de tick ≤ 10 ms |
| Red por cliente (con generación local) | ≤ 32 KB/s de media |
| Recorrer 10 km en línea recta | Memoria estable (sin crecer) y sin tirones |
