# ADR 0009: crate de animación por piezas rígidas (grisu-anim)

- **Fecha:** 25 de octubre de 2026
- **Estado:** aceptada
- **Plan detallado:** `docs/graficos/02-animacion.md`

## Contexto

El ADR 0003 decidió personajes por piezas rígidas animados en el cliente, sin deformación de malla. Toca materializarlo en el hito 3 (Turno Cero): hace falta un crate **solo CPU** que concentre el posado (esqueletos, clips, capas, IK, muelles) sin gráficos ni E/S, consumible por `grisu-render` —que solo necesita matrices para su dibujo instanciado— y dirigido por `hush-client`, que elige el clip según el estado y los eventos con tick que ya produce `hush-sim`. La animación es presentación: nunca cambia el resultado de una partida.

## Decisión

Crate de motor `grisu-anim`, dependiente únicamente de `glam` y sin `unsafe`:

1. **Esqueletos de piezas rígidas en píxeles**, con la escala canónica de 32 px = 1 bloque (`PIXELS_PER_BLOCK`) y validación estructural al construir el `RigModel`: sin ciclos de paternidad, todo padre con índice menor que su hijo (orden topológico) y todo enganche apuntando a una pieza existente; los fallos son `RigError`, no pánicos. `RigModel::miner_default()` integra el minero base: **14 piezas y 3 enganches** (`lamp`, `tool` y `back`), con 54 px de altura ≈ 1,7 bloques, de modo que la silueta calza con la hitbox de `hush-sim` (`HEIGHT = 17/10` bloques).
2. **Clips de fotogramas clave con marcadores**: cada pista anima una pieza con rotación y desplazamiento interpolados con suavizado smoothstep (`slerp` entre rotaciones); los marcadores `impact`, `step_l`, `step_r` y `release` disparan sonidos y sincronizan con la simulación. **Sincronía por tick:** `AnimClip::scale_for_impact_at_tick` estira el clip de picar para que su marcador `impact` (a los 0,5 s del `dig` base) caiga en el tick exacto en que la simulación golpea o rompe el bloque.
3. **Capas en orden fijo:** locomoción (mezcla lineal de poses) → acción de cintura para arriba (solo el subárbol de `torso`; las piernas quedan intactas) → aditivas (respiración: amplitud `0,008 + 0,05·e²` rad y periodo `3,5 − 1,5·e` s según el cansancio) → **procedural siempre al final**: mirada repartida 35 % torso / 65 % cabeza con topes anatómicos por pieza, IK de pies analítico de dos huesos con descenso de pelvis (escalones de hasta ±0,5 bloque) y muelles de inercia `Spring1D` (rigidez acotada a 1–1000, amortiguación a 100 como mucho, integración semi-implícita en subpasos de 1/240 s).
4. **Salida para el dibujo:** `evaluate_world_pose` evalúa la cadena cinemática y devuelve una matriz por pieza y por enganche, en píxeles del espacio del modelo, listas para una sola llamada instanciada. Las entradas no finitas se tratan como identidad o cero: ninguna matriz sale con NaN. Quien dibuja escala por `1/PIXELS_PER_BLOCK`.
5. **Sin escritura en `hush-sim` y sin dependencias `hush-*`:** la regla de dependencias del workspace lo garantiza. Tampoco hay reloj: el tiempo de clip y los `dt` llegan como parámetros, así que la pose es reproducible.
6. **Pruebas:** 41 pruebas unitarias cubren las 14 piezas y 3 enganches del minero, la validez de los 7 clips base (`idle`, `walk`, `run`, `dig`, `place`, `ladder` y `carry`) contra el modelo, el impacto del `dig` en el tick exacto, que la capa de cintura para arriba no toca piernas, y el IK del escalón alto con matrices finitas.

## Alternativas descartadas

- **Mallas deformables (glTF y esqueletos con pesos):** más caras de producir y de dibujar, y ajenas al estilo de vóxeles; el ADR 0003 ya las descartó (y deja abierto importarlas para un jefe con ADR propio).
- **Bibliotecas externas de animación (`bevy_animation` y similares):** añadirían una dependencia al motor, traen su propio estado global y su bucle, y no encajan con un contrato de solo-presentación dirigida por la simulación.
- **Animar en la GPU:** complica la verificación en CPU (pruebas y hojas de animación sin abrir el juego) y el presupuesto de 0,5 ms para 64 personajes cabe de sobra en CPU.

## Consecuencias

- **Ventajas:** el posado entero se verifica sin abrir el juego (41 pruebas ahora y las hojas `anim-sheet` después), es barato (el presupuesto de `docs/graficos/05-verificacion-y-hitos.md` da a la animación **≤ 0,5 ms de CPU para 64 personajes en el T490s**) y es determinista en la elección y el muestreo del clip, porque nada lee el reloj.
- **Quedan para tareas posteriores:** el formato RON de modelos y clips (`assets/models/*.ron`, `assets/anim/*.ron`) y su carga en caliente; el subcomando `hush-cli anim-sheet`; y el consumo real desde `grisu-render` (dibujo instanciado) e `hush-client` (elección de clip).
- **Apuntado en el plan:** las variantes de primera persona (`dig@1p`) para la vista de brazos. El arte integrado del minero y de sus 7 clips es provisional, «minero cansado que carga peso», a retocar cuando lleguen las texturas de personajes de 32×32.
