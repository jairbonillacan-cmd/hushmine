# Gráficos · 5. Verificación, presupuestos e hitos

Cómo se comprueba que el juego se ve bien sin abrirlo, cuánto puede costar cada pase y en qué hito entra cada cosa. Índice en [README.md](README.md).

---

## 1. Bucle de verificación de la IA

`hush-cli shot` ya genera capturas sin ventana. Se amplía:

```bash
cargo run -p hush-cli -- shot --scene tests/scenes/tunel_lava.ron --preset low --out out/tunel_bajo.png
```
La misma escena en cada preset (`low`, `medium`, `high`, `ultra`).

```bash
cargo run -p hush-cli -- shot --scene tests/scenes/base_copper.ron --camera third --anim miner:dig@0.4 --out out/picar_3p.png
```
Cámara en tercera persona y un personaje congelado en un instante de un clip.

```bash
cargo run -p hush-cli -- shot --scene tests/scenes/cueva_oscura.ron --debug light --out out/luz.png
```
Vistas de depuración ([04-shaders.md](04-shaders.md), sección 12).

```bash
cargo run -p hush-cli -- anim-sheet --model miner --clip dig --frames 12 --out out/picar_hoja.png
```
**Hoja de animación:** 12 fotogramas del clip en una cuadrícula, de frente y de perfil. La IA la abre y revisa la pose, el ritmo y si algo atraviesa el cuerpo.

```bash
cargo run -p hush-cli -- shader-check
```
Compila con naga todas las variantes de todos los shaders, sin GPU.

```bash
cargo run -p hush-cli --release -- bench --scenario ref --preset low --gpu-timings --out out/bench_gpu.csv
```
Tiempo de GPU por pase, con consultas de marca de tiempo cuando el adaptador las admite.

## 2. Capturas de referencia con tolerancia

- Cada escena tiene una **imagen de referencia por preset** en `tests/golden/<preset>/`, generada en CI con Vulkan por software (lavapipe).
- La prueba compara píxel a píxel con **tolerancia**: diferencia máxima por canal y porcentaje máximo de píxeles distintos. Las GPU reales no dan resultados idénticos, así que la tolerancia es más alta en local que en CI.
- Si un cambio de aspecto es intencionado, se regeneran las referencias con `--bless` y **la IA abre la imagen nueva antes de aceptarla**, como manda `AGENTS.md`.
- Nuevas escenas: personajes en cada animación clave, agua y lava, la lámpara en un túnel con polvo, cada bioma y cada dimensión, y la tercera persona en un túnel estrecho.

## 3. Pruebas automáticas

| Prueba | Qué garantiza |
|---|---|
| Todas las variantes de shader compilan | Ningún preset ni plataforma rompe el juego |
| **Regla del mip de 16×16** | Las texturas de 32×32, reducidas, pasan las pruebas actuales de luminancia y daltonismo |
| Paleta cerrada a 32×32 | Ningún píxel fuera de la paleta ampliada |
| Clips completos | Cada estado de la simulación tiene clip para cada modelo |
| Marcador `impact` en su tick | El golpe se ve cuando la simulación lo aplica |
| **Cámara en tercera persona** | Con túneles aleatorios (`proptest`), la cámara nunca queda dentro de un bloque sólido |
| La vista de brazos no atraviesa paredes | Pase con su propio rango de profundidad |
| Indicadores de ruido presentes | Activos en todos los presets |
| **Sin destellos** | Con la opción activada, la luminancia media de la pantalla nunca cambia más de un umbral entre fotogramas (secuencias de explosiones y del Revés) |
| Determinismo intacto | Las animaciones y los efectos no escriben en `hush-sim` (la regla de dependencias lo impide) |

## 4. Presupuestos por pase

**Intel UHD 620 (T490s), 1080p, preset Bajo, render al 75 %. Objetivo: p95 ≤ 16,7 ms.**

| Pase | Presupuesto |
|---|---|
| Agrupar luces + terreno opaco | 6,0 ms |
| Entidades y personajes | 1,5 ms |
| Transparencias (agua, cristal) | 1,0 ms |
| Partículas | 0,5 ms |
| Posproceso + escalado | 2,0 ms |
| Vista de brazos | 0,3 ms |
| Interfaz | 1,0 ms |
| **Margen** | 4,4 ms |

**GTX 1660, 1080p, preset Alto:** p95 ≤ 6,9 ms en total, como fija `PLAN_v4.md`. **Steam Deck:** 800p en Medio a 16,7 ms. **Navegador:** Bajo, a 16,7 ms en un portátil medio.

**CPU (T490s):** animación ≤ 0,5 ms, partículas en CPU ≤ 0,5 ms, preparar el fotograma ≤ 1 ms.

Si un pase se pasa de su presupuesto en el bench, se trata como un fallo de prueba, no como una nota.

## 5. Qué entra en cada hito

| Hito de `PLAN_v4.md` | Gráficos | Tiempo añadido aprox. |
|---|---|---|
| **1. Vóxel** | **G0** (decisiones baratas): el tamaño de textura sale del manifiesto; **luz de bloque en color** desde el BFS; preprocesador de shaders y `shader-check`; vistas de depuración `light`, `ao` y `chunks`; primera persona | 2–3 días |
| **3. Turno Cero** | `grisu-anim`; modelo del minero; clips básicos (andar, picar, escalera, poner, cargar); vista de brazos; tercera persona (Taller y pruebas); tinte de equipo; partículas en CPU; `anim-sheet` | +2 sem |
| **4. El Durmiente** | Lava y agua; luces dinámicas y lámpara de casco; HDR, adaptación y bloom; onda sísmica; **indicadores de ruido**; visión de geófono | +1–2 sem |
| **5. Web y en línea** | Rutas de WebGL2, presets, escalado nítido, ajuste para Steam Deck | +1 sem |
| **6. Alfa pública** | **Texturas de 32×32 completas** con paleta ampliada y mapas de emisión y brillo; todas las animaciones del Turno; posproceso completo; espectador y cámara de fotos; accesibilidad completa | +2–3 sem (en lugar del «arte de 16×16») |
| **7. Acceso anticipado** | Ultra, sombras de varias lámparas, cámara de director para la crónica | — |
| **Mundo W4–W7** | Párpado y horizonte, gradación por bioma, estilos de cada dimensión, animaciones y criaturas del modo Historia | Dentro de cada hito W |

## 6. Riesgos

| Riesgo | Mitigación |
|---|---|
| La UHD 620 no llega con luces dinámicas y posproceso | Presupuestos por pase medidos en el bench; el preset Bajo quita ambiente, nunca información; render al 75 % |
| El arte de 32×32 multiplica el trabajo | Generador procedural ampliado y retoques a mano solo en bloques clave |
| La animación crece sin fin | Lista cerrada por hito (sección 5); hojas de animación para revisar rápido |
| Demasiadas variantes de shader | Variantes solo por preset y plataforma; `shader-check` en CI |
| La tercera persona rompe el competitivo | Solo primera persona en clasificatorias; el servidor no envía lo que la cámara no debería ver |
| Mareo por movimiento de cámara | «Reducir movimiento» desde el hito 3 y campo de visión ajustable |
| WebGL2 se queda atrás | Rutas de respaldo definidas desde el principio; la web puede ir en Bajo |

## 7. Decisiones abiertas

1. **Pack «Clásico» de 16×16:** ¿se mantiene como opción o se retira con la Alfa?
2. **Tercera persona en el Turno informal:** ¿permitida por defecto o solo si el servidor la activa?
3. **Importar glTF** para jefes con deformación de malla: solo si las piezas rígidas no bastan, con un ADR propio.
4. **Sombras de todas las lámparas** en Ultra: depende de lo que mida el bench en una GTX 1660.
