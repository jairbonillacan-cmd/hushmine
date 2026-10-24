# Guía de Dirección de Arte y Texturas (16×16) — HUSHMINE

Este documento establece las reglas visuales, cromáticas, geométricas y de accesibilidad para el conjunto de texturas 16×16 de **HUSHMINE** (motor **Grisú**). Todas las texturas se generan de forma procedural determinista a partir de la paleta cerrada definida en `tools/textures/palette.py`.

> **Cambio previsto (propuesta, ADR 0003):** desde el hito 6 la resolución canónica pasa a **32×32**, con paleta ampliada. Las reglas de esta guía se mantienen y sus pruebas se aplican también a la versión reducida a 16×16. Ver `docs/graficos/03-texturas-32.md`.

---

## 1. Visión Estética: Estratigrafía Industrial (1931 / 1979)

El mundo subterráneo de HUSHMINE se apoya en dos capas históricas superpuestas:

1. **La concesión original de 1931 (*Anima Consolidated*):**
   - Estructuras de madera entibada oscurecida por la humedad (`timber_*`), remaches de fundición pesada, raíles oxidados y vagonetas de cangilones.
   - Tonos terrosos, pizarras apagadas, hollín y basalto profundo que absorben la luz y no compiten por la atención visual del jugador.
2. **La incursión clandestina de 1979:**
   - Equipos portátiles de extracción rápida, campos de fuerza experimentales, balizas de radiofrecuencia, plataformas de salto neumáticas y baterías industriales de alta tensión.
   - Iluminación dominada por **lámparas de vapor de sodio de baja presión** (`sodium_warm`, ámbar monocromático) y niebla densa en suspensión (polvo de carbón y grisú).

### Principio de jerarquía visual en penumbra
Bajo niebla densa y luz escasa, el ojo humano pierde sensibilidad cromática antes de perder sensibilidad al contraste acromático (luminancia) y a la silueta geométrica. Por ello:
- **El entorno pasivo** (`hard_rock`, `dirt`, `rock`) ocupa el tercio inferior de luminancia (`22..70`) con saturación mínima.
- **Los recursos extraíbles** (`ore`, `gold`, `diamond`) escalan progresivamente en luminancia (`88..170`) y nitidez geométrica conforme aumenta su valor económico.
- **El peligro letal** (`lava`, `electric`, `explosive`) utiliza los picos de luminancia (`188..215` en `lava`) o contrastes de señalización industrial de alto impacto (`hazard_red`, `electric_blue`, `sodium_warm`).

---

## 2. Paleta Cerrada de 48 Colores (`PALETTE_HEX`)

Ninguna textura del juego puede contener un píxel fuera de los **48 colores** definidos en `tools/textures/palette.py` (`PALETTE_SET`), todos con canal alfa opaco (`A = 255`). La luminancia relativa se calcula según la norma ITU-R BT.709 en escala `0.0..255.0`:

$$Y = 0.2126 \cdot R + 0.7152 \cdot G + 0.0722 \cdot B$$

| # | Identificador | Hex | RGB `(R, G, B)` | Luminancia (`Y`) | Grupo y uso previsto |
|---:|:---|:---:|:---:|---:|:---|
| 1 | `void_black` | `#0A0B0E` | `(10, 11, 14)` | 11.0 | Sombra: grietas profundas, huecos internos y juntas |
| 2 | `soot_dark` | `#14161B` | `(20, 22, 27)` | 21.9 | Sombra: hollín de galería, biseles oscuros |
| 3 | `shadow_slate` | `#1E222A` | `(30, 34, 42)` | 33.7 | Sombra: transición de penumbra y oclusión |
| 4 | `peat_dark` | `#2B2118` | `(43, 33, 24)` | 34.5 | Tierra: turba húmeda y poros oscuros de `dirt` |
| 5 | `loam_brown` | `#3B2D20` | `(59, 45, 32)` | 47.0 | Tierra: tono base principal de `dirt` |
| 6 | `ochre_dirt` | `#4A3826` | `(74, 56, 38)` | 58.5 | Tierra: gránulos secos y relieves de `dirt` |
| 7 | `timber_dark` | `#332419` | `(51, 36, 25)` | 38.4 | Madera 1931: veta oscura en `ladder`, `rail`, `bucket_post` |
| 8 | `timber_mid` | `#4F3826` | `(79, 56, 38)` | 59.6 | Madera 1931: tablón estructural tratado con creosota |
| 9 | `timber_light` | `#6E5037` | `(110, 80, 55)` | 84.6 | Madera 1931: aristas desgastadas y peldaños de `ladder` |
| 10 | `basalt_deep` | `#16181F` | `(22, 24, 31)` | 24.1 | Roca dura: fractura profunda de `hard_rock` |
| 11 | `basalt_dark` | `#1C1F28` | `(28, 31, 40)` | 31.0 | Roca dura: masa basáltica principal de `hard_rock` |
| 12 | `basalt_mid` | `#222530` | `(34, 37, 48)` | 37.2 | Roca dura: facetas densas de `hard_rock` |
| 13 | `slate_dark` | `#353942` | `(53, 57, 66)` | 56.8 | Roca: lecho de pizarra oscuro de `rock` |
| 14 | `slate_mid` | `#3D424D` | `(61, 66, 77)` | 65.7 | Roca: matriz intermedia de `rock` y ganga mineral |
| 15 | `slate_light` | `#444A57` | `(68, 74, 87)` | 73.7 | Roca: lajas de pizarra iluminadas en `rock` |
| 16 | `shale_highlight` | `#505766` | `(80, 87, 102)` | 86.6 | Roca: aristas de esquisto y contraste pétreo |
| 17 | `ore_rust_dark` | `#6E4427` | `(110, 68, 39)` | 74.8 | Mena: borde oxidado de veta en `ore` |
| 18 | `ore_rust_mid` | `#8C5830` | `(140, 88, 48)` | 96.2 | Mena: cuerpo cuprífero/ferroso de `ore` |
| 19 | `ore_amber` | `#AD6E39` | `(173, 110, 57)` | 119.6 | Mena: veta enriquecida cálida de `ore` |
| 20 | `ore_highlight` | `#CF8948` | `(207, 137, 72)` | 147.2 | Mena: destello metálico de calcopirita en `ore` |
| 21 | `gold_shadow` | `#7A5C1E` | `(122, 92, 30)` | 93.9 | Oro: engaste oscuro de pepita en `gold` |
| 22 | `gold_mid` | `#A8822A` | `(168, 130, 42)` | 131.7 | Oro: cuerpo metálico denso de `gold` |
| 23 | `gold_bright` | `#CCA035` | `(204, 160, 53)` | 161.6 | Oro: cara bruñida de pepita en `gold` |
| 24 | `gold_gleam` | `#E8BE54` | `(232, 190, 84)` | 191.3 | Oro: brillo especular puro de `gold` |
| 25 | `diamond_deep` | `#3A788C` | `(58, 120, 140)` | 108.3 | Diamante: faceta refractada profunda en `diamond` |
| 26 | `diamond_teal` | `#4FA0B5` | `(79, 160, 181)` | 144.3 | Diamante: plano cristalino medio en `diamond` |
| 27 | `diamond_cyan` | `#66C2D6` | `(102, 194, 214)` | 175.9 | Diamante: cara octaédrica principal en `diamond` |
| 28 | `diamond_ice` | `#8CE0F0` | `(140, 224, 240)` | 207.3 | Diamante: arista de hielo de alto contraste |
| 29 | `diamond_white` | `#C4F4FC` | `(196, 244, 252)` | 234.4 | Diamante: vértice de destello cenital en `diamond` |
| 30 | `magma_crust` | `#8F3B1B` | `(143, 59, 27)` | 74.5 | Lava: costra piroclástica enfriada en `lava_0..3` |
| 31 | `lava_red` | `#D95B26` | `(217, 91, 38)` | 114.0 | Lava: borde de fisura magmática en `lava_0..3` |
| 32 | `lava_orange` | `#F28A2E` | `(242, 138, 46)` | 153.5 | Lava: corriente convectiva media en `lava_0..3` |
| 33 | `lava_yellow` | `#FCD153` | `(252, 209, 83)` | 209.0 | Lava: vena incandescente activa en `lava_0..3` |
| 34 | `lava_core` | `#FFF08A` | `(255, 240, 138)` | 235.8 | Lava: núcleo térmico máximo en `lava_0..3` |
| 35 | `copper_dark` | `#5E2C18` | `(94, 44, 24)` | 53.2 | Equipo Copper: sombra de blindaje y chasis |
| 36 | `copper_mid` | `#944824` | `(148, 72, 36)` | 85.6 | Equipo Copper: chapa industrial de cobre |
| 37 | `copper_bright` | `#C96832` | `(201, 104, 50)` | 120.7 | Equipo Copper: banda identificativa y remaches |
| 38 | `copper_glow` | `#F09246` | `(240, 146, 70)` | 160.5 | Equipo Copper: emisor energético (`force_field`, `beacon`) |
| 39 | `cobalt_dark` | `#1A2E54` | `(26, 46, 84)` | 44.5 | Equipo Cobalt: sombra de blindaje y chasis |
| 40 | `cobalt_mid` | `#284C85` | `(40, 76, 133)` | 72.5 | Equipo Cobalt: chapa industrial de cobalto |
| 41 | `cobalt_bright` | `#3B73C4` | `(59, 115, 196)` | 108.9 | Equipo Cobalt: panel técnico y cruces de equipo |
| 42 | `cobalt_glow` | `#5CA3F2` | `(92, 163, 242)` | 153.6 | Equipo Cobalt: emisor energético (`force_field`, `beacon`) |
| 43 | `steel_dark` | `#292E38` | `(41, 46, 56)` | 45.7 | Industrial: bastidor de hierro fundido y carcasas |
| 44 | `steel_mid` | `#485060` | `(72, 80, 96)` | 79.5 | Industrial: acero laminado en `rail`, `jump_pad`, `electric` |
| 45 | `steel_light` | `#6C778C` | `(108, 119, 140)` | 118.2 | Industrial: cabezal de carril bruñido y contactos |
| 46 | `hazard_red` | `#C9302C` | `(201, 48, 44)` | 80.2 | Peligro: cartuchos de dinamita y bandas de `explosive` |
| 47 | `electric_blue` | `#38B6FF` | `(56, 182, 255)` | 160.5 | Peligro: arco voltaico y aisladores de `electric` |
| 48 | `sodium_warm` | `#FAAE3C` | `(250, 174, 60)` | 181.9 | Peligro / Luz: lámpara de sodio, flechas de `jump_pad`, chispas |

---

## 3. Contraste en Escala de Grises y Accesibilidad (Protanopia / Deuteranopia)

### 3.1. Bandas de luminancia (`LUMINANCE_BANDS`) y umbral mínimo
Para que la lectura en movimiento o bajo visión periférica nunca dependa únicamente del matiz cromático, los bloques naturales siguen una escalera estricta de luminancia media (`mean_luminance` del bloque 16×16 completo):

| Bloque | Banda objetivo (`Y` media, 0..255) | Separación mínima al siguiente escalón |
|:---|:---:|---:|
| `hard_rock` | `22.0 – 38.0` | +2.0 respecto a `dirt` |
| `dirt` | `40.0 – 54.0` | +2.0 respecto a `rock` |
| `rock` | `56.0 – 70.0` | **+18.0** respecto a `ore` |
| `ore` | `88.0 – 102.0` | **+18.0** respecto a `gold` |
| `gold` | `120.0 – 136.0` | **+18.0** respecto a `diamond` |
| `diamond` | `154.0 – 170.0` | **+18.0** respecto a `lava` |
| `lava` (`0..3`) | `188.0 – 215.0` | Cima de luminancia del juego |

Cualquier par entre `{rock, ore, gold, diamond, lava}` mantiene por diseño una distancia de luminancia media $\Delta Y \ge 18.0$, superando con holgura el requisito técnico `MIN_LUMINANCE_DISTANCE = 15.0`. Incluso en un monitor monocromo (acromatopsia total), los cinco bloques críticos de jugabilidad son inequívocos por su brillo global.

### 3.2. Redundancia por forma y patrón geométrico
En daltonismo rojo-verde (**protanopia** y **deuteranopia**, simulados con las matrices de Viénot 1999 en `palette.py`), los tonos rojizos/naranjas (`ore`, `copper_*`, `lava_red`) y amarillos/verdosos (`gold`) convergen hacia un eje ocre-pardo. Además del escalonamiento de luminancia, cada bloque natural posee una **firma morfológica única**:

- **`rock` (Pizarra estratificada):** Lajas horizontales escalonadas con fisuras finas y grano sedimentario suave, sin incrustaciones brillantes.
- **`hard_rock` (Basalto columnar denso):** Bloques poligonales oscuros y angulosos de grano compacto que transmiten impenetrabilidad.
- **`ore` (Vetas diagonales quebradas):** Filones minerales irregulares trazados en diagonal (`45°`) con ramificaciones dentadas que atraviesan la roca matriz.
- **`gold` (Pepitas redondeadas):** Nódulos y racimos globulares de bordes curvos con un punto especular (`gold_gleam`) en el cuadrante superior izquierdo de cada pepita.
- **`diamond` (Cristales rómbicos / octaédricos):** Geometría facetada de silueta romboidal (`|dx| + |dy| <= r`) con cruces axiales de alto contraste (`diamond_white` sobre `diamond_deep`). En protanopia/deuteranopia conserva su componente azul-cian intacta.
- **`lava` (Corteza magmática fisurada animada en 4 fotogramas):** Placas oscuras (`magma_crust`) separadas por canales incandescentes anchos (`lava_yellow`, `lava_core`) que se desplazan de forma cíclica entre `lava_0`, `lava_1`, `lava_2` y `lava_3`.

### 3.3. Identidad visual de equipos: Copper Crew vs. Cobalt Crew
Todos los bloques con variante de equipo (`team_block`, `force_field`, `beacon`, `bucket_station`, `bucket_post`) aplican **doble codificación (color + geometría)**:

| Atributo | Equipo **Copper** (`_copper`) | Equipo **Cobalt** (`_cobalt`) |
|:---|:---|:---|
| **Gama cromática** | Cobre oxidado a ámbar cálido (`#5E2C18` → `#F09246`) | Azul prusia a cobalto eléctrico (`#1A2E54` → `#5CA3F2`) |
| **Respuesta en daltonismo** | Eje cálido ocre-dorado de baja componente azul ($B \le 70$) | Eje frío azulado de alta componente azul ($B \ge 84$ hasta $242$) |
| **Geometría estructural** | **Bandas diagonales a 45° (chevrones) y remaches circulares** en las esquinas | **Paneles ortogonales en cruz (+), biseles rectos a 90°** y nervaduras verticales/horizontales |
| **Patrón en `force_field`** | Malla diagonal en celosía (`X`) con pulsos ámbar | Retícula ortogonal cuadrada (`#`) con nodos cobalto |

---

## 4. Reglas de Mosaico (Tiling 16×16) y Lectura en Penumbra

### 4.1. Bloques naturales (`category = "natural"`, IDs 1..7)
Los bloques naturales forman grandes paredes continuas en las galerías excavadas. Para evitar costuras artificiales (efecto rejilla no deseado en la roca):
1. **Continuidad toroidal en bordes opuestos:** Las funciones generadoras deben operar en aritmética modular $\pmod{16}$ o garantizar que las alturas de estratos y vetas en $x = 0$ coincidan con $x = 15$, y las de $y = 0$ con $y = 15$.
2. **Prohibición de marcos perimetrales:** Ningún bloque natural debe oscurecer artificialmente su borde de 1 píxel (`x in {0, 15}` o `y in {0, 15}`), evitando el aspecto de «baldosas de baño» sobre la pared de la mina.
3. **Agrupación de píxeles (clustering):** Evitar el ruido blanco de 1 píxel aislado; agrupar los colores en manchas o vetas de al menos 2–4 píxeles conectados para que se lean con nitidez a media distancia y bajo niebla.

### 4.2. Bloques construidos (`category = "built"`, IDs 8..17)
Los bloques colocados por jugadores o pertenecientes a la infraestructura industrial deben distinguirse al instante del terreno natural:
1. **Marco estructural explícito:** Pueden y deben emplear bordes de chasis (`steel_dark`, `void_black`, `timber_dark` o el tono `*_dark` del equipo) de 1 o 2 píxeles que delimiten cada unidad modular de 1×1×1 metros.
2. **Alineación funcional al apilarse:**
   - `ladder` (10), `rail` (13) y `bucket_post` (17) deben mantener continuidad vertical en sus largueros (`x` fijos en los bordes superior e inferior) para que al apilarse formen una estructura ininterrumpida.
   - `beacon` (14), `jump_pad` (11), `electric` (12), `explosive` (15) y `bucket_station` (16) deben incluir un punto focal central de alta luminancia (`sodium_warm`, `electric_blue`, `hazard_red`, `copper_glow` o `cobalt_glow`) visible a través de la penumbra.

---

## 5. Orden Canónico de Capas del Texture Array (Convención A)

El `Texture2DArray` del motor carga exactamente **25 capas** (`0` a `24`), ordenadas por `block_id` ascendente (`1..17`, ya que `0 air` no tiene textura), expandiendo los 4 fotogramas de `lava` (`0..3`) y las variantes de equipo en orden `("copper", "cobalt")`:

| Capa (`layer`) | `block_id` | `block_name` | `variant` | `frame` | Archivo (`stem.png`) | Categoría |
|---:|---:|:---|:---:|---:|:---|:---:|
| 0 | 1 | `dirt` | — | 0 | `dirt.png` | `natural` |
| 1 | 2 | `rock` | — | 0 | `rock.png` | `natural` |
| 2 | 3 | `hard_rock` | — | 0 | `hard_rock.png` | `natural` |
| 3 | 4 | `ore` | — | 0 | `ore.png` | `natural` |
| 4 | 5 | `gold` | — | 0 | `gold.png` | `natural` |
| 5 | 6 | `diamond` | — | 0 | `diamond.png` | `natural` |
| 6 | 7 | `lava` | — | 0 | `lava_0.png` | `natural` |
| 7 | 7 | `lava` | — | 1 | `lava_1.png` | `natural` |
| 8 | 7 | `lava` | — | 2 | `lava_2.png` | `natural` |
| 9 | 7 | `lava` | — | 3 | `lava_3.png` | `natural` |
| 10 | 8 | `team_block` | `copper` | 0 | `team_block_copper.png` | `built` |
| 11 | 8 | `team_block` | `cobalt` | 0 | `team_block_cobalt.png` | `built` |
| 12 | 9 | `force_field` | `copper` | 0 | `force_field_copper.png` | `built` |
| 13 | 9 | `force_field` | `cobalt` | 0 | `force_field_cobalt.png` | `built` |
| 14 | 10 | `ladder` | — | 0 | `ladder.png` | `built` |
| 15 | 11 | `jump_pad` | — | 0 | `jump_pad.png` | `built` |
| 16 | 12 | `electric` | — | 0 | `electric.png` | `built` |
| 17 | 13 | `rail` | — | 0 | `rail.png` | `built` |
| 18 | 14 | `beacon` | `copper` | 0 | `beacon_copper.png` | `built` |
| 19 | 14 | `beacon` | `cobalt` | 0 | `beacon_cobalt.png` | `built` |
| 20 | 15 | `explosive` | — | 0 | `explosive.png` | `built` |
| 21 | 16 | `bucket_station` | `copper` | 0 | `bucket_station_copper.png` | `built` |
| 22 | 16 | `bucket_station` | `cobalt` | 0 | `bucket_station_cobalt.png` | `built` |
| 23 | 17 | `bucket_post` | `copper` | 0 | `bucket_post_copper.png` | `built` |
| 24 | 17 | `bucket_post` | `cobalt` | 0 | `bucket_post_cobalt.png` | `built` |
