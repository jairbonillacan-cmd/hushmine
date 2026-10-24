# Gráficos: cámaras, animación, texturas de 32×32 y shaders

Plan del salto visual de HUSHMINE y Grisú: cámaras en primera y tercera persona, personajes animados, texturas de 32×32 y shaders propios con posproceso. Vale para el modo Turno, el Taller y el modo Historia.

Revisión del 24 de octubre de 2026. Las decisiones están en [ADR 0003](../adr/0003-graficos.md) (estado: propuesta). Las cifras son objetivos por validar en el T490s.

---

## 1. Decisiones

| Tema | Decisión | Por qué |
|---|---|---|
| **Cámaras** | Primera persona por defecto, tercera persona con brazo de cámara que choca con los vóxeles, y espectador libre. En el Turno **clasificatorio**, solo primera persona (configurable por servidor) | En túneles, la tercera persona deja mirar tras las esquinas sin asomarse: es una ventaja injusta en competitivo |
| **Personajes** | **Modelos por piezas rígidas** (cajas con pivotes) en un formato RON propio, dibujados con instancias | Encaja con el estilo de vóxeles, es barato en la UHD 620, se lee con facilidad en un diff y la IA puede escribirlo y revisarlo |
| **Animación** | Clips de fotogramas clave + capas procedurales (IK de pies, mirada, inercia). **Solo en el cliente**, guiada por el estado y los eventos de `hush-sim` | La regla «el cliente solo dibuja» se mantiene: la animación no cambia el resultado de la partida |
| **Texturas** | **32×32 como resolución canónica** desde el hito 6 (Alfa). El motor lee el tamaño del manifiesto (16, 32 o 64) | Más detalle para leer menas y materiales bajo la lámpara. Las de 16×16 actuales siguen como provisionales |
| **Iluminación** | **Luz de bloque en color** (RGB 4:4:4) + luz de cielo + luces dinámicas (la lámpara de casco) | En una mina, la luz es la mitad del arte: lava naranja, balizas del color del equipo, cristales |
| **Shaders** | WGSL propio con un **preprocesador mínimo** (inclusiones y variantes por calidad), sin dependencias nuevas | Pocos shaders y bien controlados; el preprocesador cabe en un archivo |
| **Posproceso** | HDR, adaptación a la oscuridad, bloom, mapeo de tonos, gradación por bioma y dimensión, escalado de resolución | Sensación de mina sin coste prohibitivo |
| **Accesibilidad** | **Indicadores visuales del ruido**, reducir movimiento, sin destellos, campo de visión ajustable | El sonido es la mecánica central: quien no oye bien tiene que poder jugar |

## 2. Documentos

| Documento | Contenido |
|---|---|
| [01-camaras.md](01-camaras.md) | Primera persona (vista de brazos y herramienta), tercera persona, espectador, cámara de fotos y repeticiones, sensación de cámara |
| [02-animacion.md](02-animacion.md) | Modelos por piezas, formato RON, clips, capas, IK, sincronía con los ticks, listas de animaciones del Turno y del modo Historia |
| [03-texturas-32.md](03-texturas-32.md) | Paso a 32×32: paleta ampliada, generador, variantes, texturas animadas, mapas de relieve, emisión y pruebas en el mip de 16×16 |
| [04-shaders.md](04-shaders.md) | Terreno, fluidos, personajes, partículas, luz en color, lámpara de casco, posproceso y estilos de cada dimensión |
| [05-verificacion-y-hitos.md](05-verificacion-y-hitos.md) | Capturas con tolerancia, hojas de animación, tiempos por pase, presets, qué entra en cada hito y riesgos |

## 3. Presets de calidad

| Función | **Bajo** (UHD 620, navegador) | **Medio** | **Alto** (GTX 1660) | **Ultra** |
|---|---|---|---|---|
| Escala de render | 75 % + escalado nítido | 100 % | 100 % | 100 % |
| Texturas | 32×32 con mipmaps | 32×32 | 32×32 + relieve | 32×32 + relieve |
| Luz de bloque en color | Sí | Sí | Sí | Sí |
| Oclusión ambiental por vértice | Sí | Sí | Sí | Sí |
| Luces dinámicas | 8, sin sombras | 16, sin sombras | 32, sombra de tu lámpara | 64, sombras de 4 lámparas |
| Haz volumétrico de la lámpara | No | Barato (8 pasos) | 16 pasos | 32 pasos |
| Partículas | 2 000 (CPU) | 8 000 | 32 000 (GPU) | 64 000 |
| Bloom | Ligero | Sí | Sí | Sí |
| Agua | Transparente | + reflejo falso | + refracción | + refracción y cáusticas |
| Distancia (modo Historia) | R = 8 | R = 12 | R = 16 | R = 20 |

Todo lo que da información de juego (luz de bloque, menas legibles, indicadores de ruido) está en **todos** los presets. Los presets altos solo añaden ambiente.

## 4. Dónde vive cada cosa

| Pieza | Crate | Nota |
|---|---|---|
| Cámaras, pases de render, shaders, posproceso, partículas | `grisu-render` | Sin dependencias `hush-*` |
| Esqueletos de piezas, clips, mezcla, IK | `grisu-anim` (nuevo) | Solo CPU, sin `wgpu`; se prueba en segundos |
| Modelos y clips de HUSHMINE | `assets/models/`, `assets/anim/` | RON |
| Qué animación toca según la simulación | `hush-client` | Lee estado y eventos de `hush-sim`; no escribe nada en ella |
| Generación de texturas | `tools/textures/` | El generador procedural actual, ampliado a 32×32 |
| Capturas y hojas de animación | `hush-cli` | Ver [05-verificacion-y-hitos.md](05-verificacion-y-hitos.md) |

Se mantiene la regla: ningún `grisu-*` depende de un `hush-*`. La animación usa `f32` porque es presentación: la prohibición de coma flotante vale para el estado de `hush-sim`, no para el cliente.
