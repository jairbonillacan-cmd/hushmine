# ADR 0006: Modo historia, aventura y supervivencia

- Estado: aceptado
- Fecha: 2026-10-25
- Documento de origen: `HUSHMINE_Modo_Historia_Diseno.md` (maestro de diseño)

## Contexto

El diseño maestro del modo historia describe un juego de supervivencia
completo: inventario y fabricación matricial, horno y metalurgia,
iluminación dual con ciclo diurno, fisiología (salud, nutrición, caídas,
ahogo), autómata de fluidos, estratos geológicos, dureza y tiers de
herramientas, bestiario con IA, F3 de telemetría, árbol de hitos con el
clímax del Pozo Ancestral y persistencia en regiones.

El documento asume un mundo infinite-ish (Y de -64 a 128, regiones 32×32
chunks en disco, chunks cargados bajo demanda, varios hilos). El motor
Grisú real es un mundo finito de hasta 128³ por eje, con una sola hebra de
simulación y sin dependencias nuevas permitidas a la ligera.

## Decisión

Adaptar el diseño al motor existente en lugar de reescribir el motor:

1. **Módulo `hush_sim::story`** con la simulación completa a 20 Hz
   (`STORY_TICK_RATE`), determinista (enteros y Q10/Q8; el punto fijo de
   Grisú es `FIXED_ONE = 1024`). El modo competitivo (`Simulation` a 30 Hz)
   queda intacto.
2. **Mundo finito**: las cotas del documento se reparten proporcionalmente
   a la altura del mapa (`Bands::new`): superficie = alto - cielo, y de
   ahí sedimentario (3/4), cristalino (1/2), abisal (1/4), basáltico
   (1/12 + 2) y lecho de roca madre (y < 2, infranqueable).
3. **Bloques**: la Convención A se amplía con ids 18..=28 (tablón, puntal,
   antorcha, banco, horno, arcón, grava, agua, obsidiana, mena de carbón y
   tronco). El catálogo `data/blocks.ron` crece con ellos; las texturas de
   los nuevos bloques son procedurales (capas extra del texture array tras
   los PNG y los fotogramas de lava).
4. **Luz dual**: `grisu_voxel::DualLightMap` calcula dos canales (cielo
   por inyección cenital y difusión lateral; bloque por emisores BFS).
   `PackedFace` empaqueta ambos en los 4 bits altos de `b` (el tamaño
   sigue siendo 8 bytes). El shader combina `max(block, sky * daylight)`
   con el multiplicador del ciclo de 24 000 ticks (día 1.0, ocaso hasta
   0.08, noche, amanecer).
5. **F3**: la columna izquierda usa datos de la simulación (posición fija,
   bloque, chunk, región, rumbo, estrato, luces duales, vitals); la derecha
   expone métricas de malla del renderer y el raycast de objetivo. Memoria
   RSS/VRAM y telemetría de red quedan fuera: no hay dependencia permitida
   para leerlas y el modo historia es local.
6. **Hito simplificado sin perder el bucle**: los nueve hitos y el clímax
   (4 válvulas + 4 engranajes + asedio de 6 000 ticks + regreso a
   superficie) están completos; la reparación usa tablones y lingotes de
   hierro como materiales de sellado.
7. **Regiones `.hush`**: formato propio con mágica `HUSHRG01`, versión,
   tamaño y chunks en RLE de pares `(bloque, longitud)`. El byte de códec
   reserva el 1 para `zstd` en el futuro sin romper compatibilidad. Con
   RLE, un mundo 96³ ocupa ~105 KB.
8. **IA**: A* vóxel 3D con caja de búsqueda local y reintentos crecientes,
   presupuesto de nodos y celdas dentro del mundo. El Bregador sigue el
   ruido del pico y los pasos; el Acechador silba 30 ticks y deflagra; el
   Artillero dispara proyectiles y mantiene distancia; la Alimaña surge en
   enjambres al picar grava profunda. Generación con luz efectiva ≤ 7 y
   distancia de Manhattan 24..64; despawn a 128.
9. **Cliente**: `--story` arranca `StoryApp` (HUD de corazones y comida,
   barra rápida con durabilidad, menú de fabricación compacto, F3,
   banners de hitos). El modo competitivo no cambia.
10. **Fluidos**: el agua se expande hasta 7 bloques y prefiere desniveles;
    el magma hasta 3 y quema madera. Agua sobre manantial de magma crea
    obsidiana; contra magma en movimiento, roca. El campo de fluidos se
    siembra desde el terreno abierto a la excavación.

## Consecuencias

- El modo historia es jugable de principio a fin (despertar → ascenso)
  con una sola semilla y sin red.
- El mundo grande (W0, región múltiple con carga por demanda) sigue
  pendiente: hoy una expedición vive en un solo archivo de región.
- El audio espacial (sección 10 del documento) queda fuera de este ADR:
  no hay motor de sonido en el workspace todavía.
- Las compilaciones de zstd y el hilo de presentación separado quedan
  anotados para cuando el mundo grande los exija.
