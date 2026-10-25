# El mundo grande del modo Historia

Ampliación de `PLAN_v4.md` para el **modo Historia (supervivencia), que es la prioridad actual del proyecto**, en un mundo procedural de **15 millones × 15 millones de bloques** en horizontal, generado por chunks a medida que se explora.

Revisión del 24 de octubre de 2026. Las cifras son objetivos por validar con medidas en el T490s. Las decisiones técnicas están en [ADR 0002](../adr/0002-mundo-grande.md) (estado: aceptada).

---

## 1. Qué cambia y qué no

- **El modo Turno no cambia.** Sus mapas de 64³ y 96×96×64, su simetría cuasi-espejo y sus invariantes (`docs/mapgen/geologia.md`) siguen igual. El mundo grande es otro generador.
- **El orden del proyecto sí cambia (24 de octubre de 2026): el modo Historia va primero.** Tras los hitos 0–2 se hacen **W0** (en curso, rama `w0/base`) y después **W1–W7**. El modo Turno queda en pausa tras el hito 2 y se retoma cuando el modo Historia sea jugable.
- **Se comparte todo lo que se pueda:** chunks, luz, mallas, colisión, red, el catálogo de bloques y el hash espacial `hash3d` de `geologia.md`.

## 2. La idea en cinco líneas

1. **El mundo es una función pura:** `(semilla, versión del generador, posición del chunk) → chunk`. Da igual el orden, el hilo o la plataforma: el resultado es idéntico byte a byte.
2. **Solo se genera lo que alguien mira o pisa.** Un planificador carga chunks por distancia y prioridad; el resto no existe.
3. **Solo se guarda lo que alguien cambió.** Lo demás se regenera desde la semilla.
4. **Toda la generación usa enteros:** ruido entero propio, sin `f32` ni `f64`. Así no hay «Tierras Lejanas» (el ruido no se degrada lejos del origen) y nativo y web coinciden.
5. **La profundidad es la progresión.** En horizontal, el mundo da para explorar y asentarse. En vertical están la historia y el peligro: la corteza, el manto, el Corazón del Durmiente Mayor y la Bóveda bajo él.

## 3. Documentos

| Documento | Contenido |
|---|---|
| [01-escala-y-motor.md](01-escala-y-motor.md) | Dimensiones, coordenadas, punto fijo, chunks con paleta, streaming, luz, render relativo a la cámara, guardado, red y presupuestos |
| [02-generacion.md](02-generacion.md) | Ruido entero, fases del generador, relieve, biomas de superficie y subterráneos, cuevas, acuíferos, lava, menas |
| [03-estructuras.md](03-estructuras.md) | Sistema de estructuras sin estado, ensamblaje por piezas, minas de los Antiguos, pueblos, catálogo y capa de historia |
| [04-verificacion-e-hitos.md](04-verificacion-e-hitos.md) | Herramientas de `hush-cli world`, invariantes `INV-W*`, hitos W0–W7, riesgos y decisiones abiertas |

El diseño del juego que vive en este mundo (lore, historia, pueblos y comercio, criaturas y jefes, tecnologías, dimensiones) está en [`docs/supervivencia/`](../supervivencia/README.md).

## 4. Cifras clave

| Magnitud | Valor |
|---|---|
| Horizontal (X, Z) | −7 500 000 … 7 499 999 (15 M bloques por eje; 937 500 chunks por eje) |
| Vertical (Y) | −4 096 … 1 023 (5 120 bloques; 320 capas de chunks). `y = 0` es el nivel del pozo donde despiertan los Llegados |
| Chunk | 16³ cúbico, con paleta, ids de bloque `u16` |
| Cruzar el mundo andando | ~40 días reales a 4,3 bloques/s. Casi nadie llegará al borde, pero el borde tiene que funcionar |
| Generación por chunk (T490s, release, un núcleo) | p50 ≤ 1 ms, p95 ≤ 3 ms |
| Memoria del cliente | ≤ 400 MB, igual que en el modo Turno (distancia de visión ajustable) |
| Disco | Solo chunks modificados: unos pocos MB por hora de juego |

## 5. Nuevos crates (cuando toque, no antes)

| Crate | Papel | Depende de |
|---|---|---|
| `grisu-noise` | Hash espacial, ruido de gradiente entero, fBm, deformación de dominio, interpolación en punto fijo. Sin dependencias | — |
| `grisu-world` | Planificador de carga, cola de trabajos, caché LRU, archivos de región, interés por jugador. Genérico mediante un trait de generador | `grisu-voxel` |
| `hush-worldgen` | El contenido de HUSHMINE: relieve, biomas, cuevas, menas, estructuras y capa de historia. Implementa el trait de generador | `grisu-noise`, `grisu-voxel`, datos RON |

Se mantiene la regla: **ningún `grisu-*` depende de un `hush-*`**. `hush-worldgen` cumple las mismas reglas de determinismo que `hush-sim`: enteros, sin aleatoriedad del sistema y sin recorrer `HashMap` cuando el orden importa. Al crearlo, se añadirá a la tabla de `AGENTS.md`.

Los datos viven en `data/worldgen/*.ron` (capas, ruido, biomas, cuevas, menas, estructuras) y las plantillas de estructuras en `assets/estructuras/`. Las plantillas se dibujan en el **modo Taller**, que así gana un segundo uso.
