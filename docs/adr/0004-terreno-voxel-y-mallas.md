# ADR 0004: terreno de vóxeles, mallas y luz

- **Fecha:** 23 de octubre de 2026
- **Estado:** aceptada

## Contexto

El hito 1 necesita terreno editable que se dibuje rápido en Windows, Linux, macOS y, en el hito 5, en el navegador (WebGPU con WebGL2 de respaldo). La simulación del servidor debe ser determinista; la presentación no.

## Decisión

- **Crate `grisu-voxel`**, sin dependencias de gráficos ni del juego (solo `bytemuck`, ya presente en el workspace). Para el motor, un bloque es un `u8` y `0` es aire.
- **Chunks de 16³**, un byte por bloque, en un mundo finito de como máximo 128 bloques por eje. Fuera del mundo todo es aire; el juego decide si los bordes se pueden cavar.
- **Mallas por eliminación de caras ocultas**, sin *greedy meshing*: el mapa completo de 64³ se regenera en unos 4 ms (release, AMD RX 9060 XT / Ryzen), muy por debajo de los 50 ms del hito. No hace falta `rayon` todavía.
- **Caras empaquetadas en 8 bytes** (posición, dirección, oclusión de las 4 esquinas, bloque y luz) y expandidas en el shader. Se dibujan como **instancias** (`draw(0..6, 0..caras)`) y no con un búfer de almacenamiento: así funciona también en WebGL2 sin una ruta aparte.
- **Oclusión ambiental por vértice** con la fórmula clásica y elección de diagonal para evitar el artefacto anisótropo.
- **Luz por BFS en el cliente** (`LightMap`): luz de cielo que baja sin perder intensidad y bloques emisores (la lava) que pierden un nivel por bloque. Se recalcula entera tras cada edición (1,5 ms en 64³) y solo se regeneran los chunks cuya luz cambió. El servidor no calcula luz.
- **Colisión en punto fijo** (1024 unidades por bloque) compartida por cliente y servidor, con barrido por ejes Y, X, Z.
- **Raycast DDA en `f32`**, solo para apuntar en el cliente. El servidor valida cada orden en enteros (alcance, bordes, bloque).

## Consecuencias

- Las mallas y la luz son baratas y simples de razonar; si el mapa crece a 96×96×64 sigue habiendo margen.
- La luz completa por edición es O(volumen). Si algún día se mide como cuello de botella (mapas mayores, lava que fluye), se pasará a una actualización incremental.
- La luz por cara (no por vértice) da un aspecto algo plano en las transiciones; se puede suavizar más adelante sin cambiar el formato.
