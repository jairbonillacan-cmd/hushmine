# Gráficos · 3. Texturas de 32×32

Cómo pasar de 16×16 a 32×32 sin perder lo que ya funciona: la paleta cerrada, la lectura en penumbra, la accesibilidad y la generación procedural. Índice en [README.md](README.md). Las reglas de arte actuales están en `docs/art/guia-texturas.md`.

---

## 1. Por qué 32×32 (y por qué no 64×64)

- **Lectura bajo la lámpara:** a 32×32, las vetas, las pepitas y las facetas del diamante se distinguen a media distancia. A 16×16 se convierten en manchas en cuanto hay niebla.
- **Materiales del modo Historia:** madera de varios tipos, cristal, hervorita, lumen y decenas de bloques de construcción necesitan más píxeles para diferenciarse.
- **Personajes:** caras, cascos, guantes y la silueta de cada oficio se leen mejor.
- **Sigue siendo pixel art:** con muestreo nítido y paleta cerrada. A 64×64 se pierde el estilo, el trabajo de arte se multiplica por 16 respecto a hoy y el T490s lo notaría.
- **Coste de memoria despreciable:** una capa de 32×32 RGBA8 con mipmaps ocupa ~5,3 KB. 500 capas, unos 2,7 MB.

## 2. Qué se mantiene de la guía actual

Todo lo que hace legible el juego sigue igual y se escala:

| Regla de `guia-texturas.md` | A 32×32 |
|---|---|
| **Paleta cerrada**: ningún píxel fuera de la paleta | Se mantiene, con la paleta ampliada (sección 3) |
| **Bandas de luminancia** (entorno oscuro, recursos más claros cuanto más valen, peligro en el pico) | Igual |
| **Firma morfológica** de cada bloque (lajas, basalto columnar, vetas a 45°, pepitas, rombos) | Igual, con más detalle dentro de la misma forma |
| **Doble codificación de equipo** (Copper con chevrones y remaches; Cobalt con paneles en cruz) | Igual |
| **Mosaico toroidal** sin marcos en los bloques naturales | Aritmética módulo 32 |
| **Agrupación de píxeles** (nada de ruido de 1 píxel) | Manchas de 3–8 píxeles conectados |
| **Marco estructural** en los bloques construidos | 2 píxeles |
| **Orden canónico de capas** (Convención A primero) | Igual: las capas 0–24 siguen siendo las del modo Turno |

## 3. Paleta ampliada

- **De 48 a ~96 colores**, todavía cerrada y definida en `tools/textures/palette.py`.
- Cada material gana una **rampa** de 4–6 tonos (sombra, base, medio, luz, brillo) en vez de 2–4.
- Se organiza en una **paleta común** (Turno y bloques compartidos) y **extensiones** del modo Historia: hervorita, lumen, cuarzo resonante, ensueño, maderas, vegetación del Silencio y tonos propios de cada dimensión.
- Siguen las pruebas de daltonismo (protanopia y deuteranopia) y de distancia mínima de luminancia entre bloques que puedan confundirse.

## 4. El generador

`tools/textures/` ya genera todas las texturas de forma procedural y determinista. Se amplía en vez de sustituirlo:

- **`TEXTURE_SIZE` pasa a ser un parámetro** (16 o 32) en `natural.py`, `built.py` y `build_all.py`. Las funciones de patrón trabajan en coordenadas relativas, no en píxeles fijos.
- **Retoque a mano permitido:** un PNG en `assets/textures/blocks/overrides/` sustituye al generado. Pasa por **las mismas pruebas** (paleta, luminancia, mosaico).
- **El manifiesto declara el tamaño** (`size: 32`), los fotogramas y los mapas extra. El motor lo lee y no supone 16 en ningún sitio.

### La regla del mip de 16×16

Cada textura de 32×32 se reduce a 16×16 (promedio 2×2 en luz lineal, ajustado al color más cercano de la paleta) y **el resultado tiene que pasar las pruebas actuales de `test_textures.py`**: luminancia, distinción entre bloques y daltonismo. Así se garantiza que, de lejos y con niebla, cuando la GPU usa ese mip, el juego se lee igual de bien que hoy.

## 5. Variantes y menos repetición

- **Variantes:** los bloques naturales tienen 2–4 variantes. Cada bloque elige una con `hash(x, y, z)`, solo en el render: no afecta a la simulación ni a la red.
- **Giro y espejo:** los bloques naturales sin dirección se giran o se reflejan según el mismo hash. Se rompe la rejilla sin coste de arte.
- **Texturas conectadas (preset Medio en adelante):** muros de equipo, vidrio, raíles y entibados unen sus bordes con los vecinos iguales, mediante una máscara de 4 bits por cara. No cambia la información de juego.
- **Estratos que cruzan bloques:** en las paredes naturales, una franja de color de baja frecuencia varía con la altura `y` en el shader, como estratos de verdad, sin texturas nuevas.

## 6. Texturas animadas

- Los fotogramas son capas consecutivas del array. El manifiesto indica cuántos son y a qué velocidad van.
- **Lava** (8 fotogramas a 32×32, ahora 4), **agua**, **campo de fuerza**, **bloque eléctrico** y **cristales que laten** en el modo Historia.
- La animación avanza con el **tiempo de render**, no con la simulación: es decoración.

## 7. Mapas extra

| Mapa | Contenido | Presets | Para qué |
|---|---|---|---|
| **Emisión** | Qué píxeles brillan solos | Todos | Lava, balizas, cristales, ojos de criaturas. Alimenta el bloom |
| **Brillo metálico** | Qué píxeles devuelven la luz de tu lámpara | **Todos** | **El oro y las menas destellan bajo la lámpara.** Es información de juego, así que va en todos los presets en versión barata |
| **Relieve** (mapa de normales) | Pendiente de cada píxel | Alto y Ultra | La luz rasante de la lámpara marca grietas y vetas |
| **Altura** | Para generar el relieve | — | Solo en el generador |

**Empaquetado:** color en un array RGBA8 sRGB; un segundo array RGBA8 lineal con emisión (R), brillo (G) y normal comprimida (BA). El generador produce el relieve a partir de una altura procedural, así que no hay que pintarlo a mano.

## 8. Muestreo nítido sin parpadeo

- El pixel art pide muestreo «vecino más cercano», pero así las texturas parpadean al alejarse.
- **Solución:** filtro lineal + mipmaps en el sampler, y en el shader un **ajuste de coordenadas a la rejilla de texeles** con `fwidth` que mantiene los bordes nítidos de cerca y suaviza solo el texel de transición. Permite filtrado anisotrópico ×4 en los presets altos.
- Los mipmaps se generan en el empaquetado (no en la GPU), con el promedio en luz lineal de la sección 4.

## 9. Otras texturas a 32 píxeles por bloque

- **Personajes y criaturas:** la misma densidad, 32 píxeles por bloque ([02-animacion.md](02-animacion.md)).
- **Objetos en la mano** e iconos de inventario derivados de los modelos.
- **Partículas:** pequeñas hojas de 8×8 y 16×16 con la misma paleta.

## 10. Transición

| Fase | Texturas |
|---|---|
| Hitos 1–5 | Las 25 texturas de 16×16 actuales, como provisionales. **El motor ya lee el tamaño del manifiesto** desde el hito 1 |
| Hito 6 (Alfa) | El juego de 32×32 completo del modo Turno, con sus mapas de emisión y brillo |
| Modo Historia (W4–W7) | Extensiones de paleta y texturas de biomas, construcción y dimensiones |
| Opcional | Un **pack «Clásico» de 16×16** con el generador actual, para quien quiera el aspecto original y para equipos muy modestos |

`assets/textures/LICENSE.md` se actualiza al cambiar de tamaño: el arte sigue siendo 100 % original y generado por código propio.
