# ADR 0006: ruido entero propio (`grisu-noise`)

- **Fecha:** 24 de octubre de 2026
- **Estado:** aceptada; se revisa en W2 cuando el generador la use de verdad
- **Plan detallado:** `docs/mundo/02-generacion.md` (sección 1) y `docs/adr/0002-mundo-grande.md`

## Contexto

El mundo grande necesita ruido 3D de densidad y 2D de parámetros evaluado hasta ±7 499 999 bloques, con el **mismo valor bit a bit en nativo y en wasm**: el cliente genera terreno y regenera chunks no guardados, y el servidor solo envía diferencias. Los crates de ruido existentes trabajan en `f32`: pierden precisión lejos del origen y no garantizan el mismo resultado entre plataformas (contracción FMA, funciones trascendentes). Además `docs/mapgen/geologia.md` ya congela `hash3d` (SplitMix64) con equivalencia bit a bit con el prototipo de Python, así que el hash iba a ser propio de todos modos.

## Decisión

Crate de motor `grisu-noise`, **sin dependencias** y sin `unsafe`, solo aritmética entera:

- **Punto fijo Q16.16** en todo el crate; las salidas de ruido quedan en `[-65 536, 65 536]` (`ONE`). Nada de `f32`/`f64` en el código (solo en las pruebas estadísticas, que no forman parte del estado).
- **`hash3d(x, y, z, seed)`** copia exacta de `docs/mapgen/geologia.md`: coordenadas `i64`, aritmética `wrapping` y las mismas constantes hexadecimales. `layer_seed` mezcla la semilla del mundo con la sal de cada capa de ruido.
- **Ruido de gradiente 2D y 3D** (Perlin): retícula de período potencia de dos, así que celda y fracción salen con desplazamientos; doce gradientes enteras en 3D y ocho en 2D elegidas por hash; producto escalar entero y curva quíntica `6t⁵ − 15t⁴ + 10t³` en Q16 con intermedios `i64`. La salida se divide entre 2 para normalizar (el producto escalar llega a ±2·ONE).
- **fBm** por octavas: frecuencia doble (un desplazamiento menos por octava), amplitud mitad y suma normalizada con la amplitud total, de modo que el rango no depende del número de octavas. **Ruido en cresta** `65 536 − |ruido|`. **Deformación de dominio**: la entrada se desplaza con otros fBm antes de muestrear.
- **Muestreo grueso `Coarse3`**: rejilla 4 × 8 × 4 por chunk (5 × 3 × 5 = 75 esquinas) e interpolación trilineal en punto fijo para la densidad 3D; unas 16 veces más barato que evaluar cada bloque.
- Sin reloj y sin estado global: funciones puras de (semilla, coordenadas). Con `i64` extremos no hay desbordamientos (`wrapping` o rangos acotados por construcción).

## Pruebas

Determinismo y **valores de referencia fijos** (los de `hash3d` verificados contra la aritmética exacta de la especificación en Python) para detectar cambios accidentales; rango de salida; continuidad entre bloques vecinos (también con paso fino en la deformación); **INV-W03**: media, varianza y histograma en ±7 499 999 estadísticamente iguales a los del origen; esquinas exactas y saturación sin pánico en `Coarse3`. La herramienta `hush-cli noise` dibuja un mapa 512 × 512 en gris de un fBm 2D para revisar el ruido a ojo.

## Alternativas descartadas

- **`noise`/`fastnoise`/`opensimplex`:** `f32`, sin garantías bit a bit entre plataformas ni de precisión lejos del origen; además arrastran dependencias a un crate de motor.
- **Ruido de valor en vez de gradiente:** algo más barato, pero con artefactos de retícula visibles en el relieve.
- **Derivar las sales de capa con XOR directo (`seed ^ salt`):** sales consecutivas (las octavas) quedan casi iguales; `layer_seed` las pasa por SplitMix64.
- **fBm sin normalizar:** la suma de octavas se saldría de `[-ONE, ONE]` según cuántas haya.

## Consecuencias

- El hash y las primitivas quedan congelados: cualquier cambio rompe mundos guardados y las pruebas de referencia lo detectan (la futura `INV-W11` de resúmenes se apoyará en esto).
- Las gradientes 2D son ocho y no doce: en el plano las doce de 3D se reducen a cuatro direcciones, peor calidad; se documenta aquí la desviación menor respecto a la tabla de `docs/mundo/02-generacion.md`.
- Los parámetros de relieve (octavas, períodos, fuerzas) llegarán en RON desde `hush-worldgen`; este crate solo aporta las primitivas.
- Quedan para W3 si los mapas lo piden: rotación entera (terna 3-4-5) para disimular la retícula alineada con los ejes y ruido celular (Voronoi) entero para regiones de bioma.
