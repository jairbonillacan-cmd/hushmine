# Identidad Visual, Marca y Cápsulas de Tienda — HUSHMINE / Grisú

Este documento define la identidad de marca de **HUSHMINE** y del motor propio **Grisú**, las especificaciones técnicas de los tres candidatos de logotipo vectorial, el icono cuadrado de aplicación (`256×256` a `16×16`), los tres bocetos de cápsulas de Steam (`460×215` y `616×353`) y la arquitectura tipográfica con licencias abiertas **SIL Open Font License 1.1 (OFL-1.1)**.

Todos los activos vectoriales se generan de forma determinista mediante `tools/ui/brand.py` hacia `assets/ui/brand/`, empleando exclusivamente la paleta cerrada de 60 colores (`ALL_PALETTE_HEX`: 48 colores de mundo + 12 colores de interfaz 1931/1979) y garantizando un contraste mínimo WCAG 2.1 AA ($\ge 4{,}5:1$) en cada etiqueta de texto.

---

## 1. Concepto de marca: HUSHMINE y el motor Grisú

### 1.1. Dualidad cronológica (1931 / 1979) en el Monte Ánima

La identidad gráfica de **HUSHMINE** nace del choque entre dos estratos materiales superpuestos en las galerías del Monte Ánima:

1. **Estrato fundacional de 1931 (*Anima Consolidated Mining Co.*):**
   - Placas de latón troquelado (`brass_dark` `#594318`, `brass_mid` `#9E7B2F`, `brass_bright` `#D9B052`).
   - Carteles de acero esmaltado al horno (`enamel_cream` `#EDE6D3`) y tinta de sello (`ink_stamp` `#1C1917`).
   - Fichas octogonales de control de turno (`#41`) colgadas en el tablero de entrada de la galería.
   - Tipografía estarcida pesada con puentes mecánicos y remaches en las esquinas.
2. **Estrato instrumental de 1979 (expedición clandestina):**
   - Pantallas de tubo de rayos catódicos de fósforo verde P1 (`crt_bg` `#07120A`, `crt_grid` `#12301B`, `phosphor_dim` `#1E6F36`, `phosphor_mid` `#34B85A`, `phosphor_bright` `#68F08E`, `phosphor_peak` `#C8FFD6`).
   - Proyectores industriales de vapor de sodio (`sodium_warm` `#FAAE3C`) que tiñen la roca (`basalt_deep` `#16181F`, `peat_dark` `#2B2118`).
   - Alertas sísmicas de erupción cuando el Sismógrafo alcanza el umbral crítico (`hazard_red` `#C9302C`, `lava_red` `#D95B26`, `lava_orange` `#F28A2E`, `lava_core` `#FFF08A`).

### 1.2. Identidad del motor propio: Grisú

**Grisú** toma su nombre del gas metano de las minas de carbón: invisible, silencioso e inflamable al menor chispazo. Visualmente, Grisú se representa como el **sustrato técnico austero y determinista** bajo la marca del juego:

- **Símbolo técnico:** una retícula ortogonal de vóxeles (`crt_grid` `#12301B`) atravesada por una chispa de ignición (`lava_core` `#FFF08A`) contenida dentro de una lámpara de seguridad Davy octogonal en trazo monolineal (`brass_bright` `#D9B052`).
- **Principio arquitectónico:** al igual que ningún crate `grisu-*` depende de `hush-*`, la firma gráfica de Grisú emplea trazos técnicos puros de osciloscopio (`phosphor_bright` sobre `void_black` `#0A0B0E`), cediendo siempre el protagonismo comercial al logotipo de **HUSHMINE**.

---

## 2. Propuestas de logotipo de HUSHMINE (`800×240`)

### 2.1. Construcción geométrica vectorial propia (`_wordmark_paths`)

En las tres propuestas y en las seis cápsulas de tienda, las ocho letras de **HUSHMINE** se construyen con **geometría vectorial propia** (`<path>` y `<rect>` en `tools/ui/brand.py`), sin usar etiquetas `<text>` para el nombre del juego. Esto garantiza que el logotipo se vea idéntico en cualquier máquina aunque no tenga fuentes instaladas:

- **Rejilla normalizada por glifo:** cada letra (`H`, `U`, `S`, `H`, `M`, `I`, `N`, `E`) se define en una caja de $10 \times 14$ unidades con astas verticales de $2{,}6\text{--}2{,}8$ unidades de grosor y travesaños horizontales de $2{,}8$ unidades.
- **Paso horizontal e interletrado:** el avance entre letras es de $12{,}2 \times \text{escala}$, con una cesura adicional de $+1{,}8 \times \text{escala}$ entre el bloque `HUSH` (índices `0..3`) y el bloque `MINE` (índices `4..7`).
- **Sombra dura proyectada:** cada glifo duplica su `<path>` desplazado $(+1{,}2\times\text{escala}, +1{,}2\times\text{escala})$ en `void_black` o `phosphor_dim` antes de trazar la cara principal biselada.
- **Modificadores por propuesta:**
  - `stencil_cut`: inserta un puente horizontal de estarcido en $y = 6{,}7 \times \text{escala}$ que parte los fustes como una plantilla metálica de 1931 (Propuesta A).
  - `seismo_shift`: aplica un desfase vertical alterno $(\pm 2{,}5\text{ px})$ a las letras de `MINE` para simular la sacudida del Sismógrafo al entrar en erupción (Propuesta B).

### 2.2. Comparativa de las tres propuestas (`viewBox="0 0 800 240"`)

| Archivo SVG | Nombre de la propuesta | Construcción por capas | Paleta empleada (`ALL_PALETTE_HEX`) |
|---|---|---|---|
| `logo_a_stencil.svg` | **Propuesta A: Troquel Industrial de Latón (1931)** | Chapa octogonal de latón (`brass_dark`, `brass_mid`, bisel `brass_bright`) con 6 remaches circulares sobre fondo `soot_dark`. Letras estarcidas (`HUSH` en `enamel_cream`, `MINE` en `brass_bright`), quebradas en el centro por una falla sísmica (`void_black` + `lava_orange`). Banda inferior con ficha `#41` y leyenda `"MOUNT ANIMA · 1931 / 1979"`. | `#0A0B0E`, `#14161B`, `#1C1917`, `#292E38`, `#594318`, `#9E7B2F`, `#D9B052`, `#EDE6D3`, `#F28A2E` |
| `logo_b_seismo.svg` | **Propuesta B: Pulso de Fósforo Verde y Despertar (1979)** | Carcasa de instrumento (`steel_dark`, `steel_mid`) y pantalla CRT (`crt_bg` `#07120A`) con retícula ortogonal (`crt_grid` `#12301B`). `HUSH` permanece estático en `enamel_cream` con contorno `phosphor_bright`, mientras `MINE` vibra en `lava_orange` sobre una onda sísmica que pasa de `phosphor_bright` a `lava_core` y `hazard_red`. | `#0A0B0E`, `#292E38`, `#485060`, `#07120A`, `#12301B`, `#1E6F36`, `#68F08E`, `#EDE6D3`, `#F28A2E`, `#FFF08A`, `#C9302C` |
| `logo_c_chasm.svg` | **Propuesta C: Falla Partida Copper vs. Cobalt** | Dos bastidores trapezoidales enfrentados: blindaje diagonal Copper a la izquierda (`copper_dark`, `copper_mid`, `copper_bright`, letras en `copper_glow`) y chasis ortogonal Cobalt a la derecha (`cobalt_dark`, `cobalt_mid`, `cobalt_bright`, letras en `cobalt_glow`). En el eje $x=400$, una grieta de magma (`magma_crust`, `lava_orange`, `lava_core`) sostiene la ficha octogonal de latón `#41`. | `#0A0B0E`, `#14161B`, `#1C1917`, `#5E2C18`, `#944824`, `#C96832`, `#F09246`, `#1A2E54`, `#284C85`, `#3B73C4`, `#5CA3F2`, `#8F3B1B`, `#F28A2E`, `#FFF08A`, `#D9B052`, `#EDE6D3` |

### 2.3. Proporciones, área de respeto y versión monocroma

- **Proporción del lienzo maestro:** $10:3$ (`800×240 px`).
- **Área de respeto (zona de exclusión):** margen perimetral mínimo de `24 px` en la escala maestra (equivalente a $0{,}25\times$ la altura de las letras de `HUSHMINE`), donde no debe invadir ningún borde de contenedor, botón ni texto ajeno.
- **Reducción mínima:**
  - Con subtítulo e inscripción secundaria: hasta `320×96 px`.
  - Solo bloque tipográfico `HUSHMINE` sin subtítulo: desde `319 px` hasta `160×48 px`.
  - Por debajo de `160 px` de ancho: usar exclusivamente el símbolo cuadrado `app_icon.svg`.
- **Versión monocroma:**
  - *Positivo sobre fondo oscuro (`void_black` `#0A0B0E` o `crt_bg` `#07120A`):* todos los rellenos de glifo y marco pasan a `enamel_cream` (`#EDE6D3`, contraste $16{,}4:1$) o `phosphor_bright` (`#68F08E`, contraste $13{,}1:1$), suprimiendo la sombra paralela.
  - *Negativo para impresión o papel (`enamel_cream` `#EDE6D3` / `paper_aged` `#D8CEB8`):* todos los trazos y glifos pasan a `ink_stamp` (`#1C1917`, contraste $\ge 11{,}8:1$).

### 2.4. Recomendación oficial

Se recomienda seleccionar la **Propuesta A (`logo_a_stencil.svg`) como logotipo principal de marca y tienda**, complementada por la **Propuesta B (`logo_b_seismo.svg`) como variante diegética dentro del cliente**:

1. **Solidez en cápsulas de Steam:** la placa octogonal de latón de 1931 de la Propuesta A aísla las letras del fondo abigarrado de las cavernas y garantiza lectura instantánea incluso en miniaturas de `231×87 px`.
2. **Coherencia con el mundo:** une en una sola pieza los tres símbolos de **HUSHMINE**: la placa de *Anima Consolidated*, la ficha `#41` y la fractura de lava que anuncia el Despertar.

---

## 3. Icono cuadrado de aplicación (`app_icon.svg`)

El archivo `assets/ui/brand/app_icon.svg` (`viewBox="0 0 256 256"`, `width="256"`, `height="256"`) condensa la identidad del juego en un emblema sin texto capaz de escalar desde `256×256` hasta `16×16` píxeles.

### 3.1. Capas compositivas del icono

1. **Base y silueta exterior (`void_black` `#0A0B0E` + octógono `brass_mid` / `brass_bright`):** marco octogonal de `6 px` de grosor con taladro superior circular (`cx="128"`, `cy="44"`, `r="10"`) que reproduce la geometría de una ficha de turno de 1931.
2. **Montaña bipolar Copper vs. Cobalt:** dos triángulos simétricos separados por el eje central (`x=128`): la ladera izquierda en `copper_bright` (`#C96832`) con borde `copper_glow` (`#F09246`) y la derecha en `cobalt_bright` (`#3B73C4`) con borde `cobalt_glow` (`#5CA3F2`).
3. **Pico minero cruzado:** hierro curvo superior en `enamel_cream` (`#EDE6D3`) y astil central vertical en `brass_bright` (`#D9B052`), ambos perfilados con `void_black` (`stroke-width="3"`).
4. **Pulso del Sismógrafo y Corazón:** en la base arde el trapecio del Corazón (`lava_red`, `lava_orange`, `lava_core`), mientras a media altura ($y=144$) cruza de borde a borde el latido verde del Sismógrafo (`phosphor_bright` `#68F08E`, pico en `phosphor_peak` `#C8FFD6`) protegido por una envolvente negra de `10 px`.

### 3.2. Comportamiento por escalón de resolución (`256×256` → `16×16`)

| Tamaño (px) | Destino de plataforma | Elementos legibles garantizados |
|---|---|---|
| `256×256` / `128×128` | Icono maestro de escritorio, biblioteca de Steam y macOS/Linux | Doble bisel octogonal, perforación de ficha, separación Copper/Cobalt, pico minero, Corazón de lava y onda completa del Sismógrafo. |
| `64×64` / `48×48` | Lanzadores de aplicaciones y conmutador `Alt+Tab` | Silueta octogonal dorada, contraste térmico izquierda/derecha, cruz del pico y trazo verde brillante central. |
| `32×32` | Barra de tareas de Windows y cabecera de ventana | El trazo envolvente negro (`10 px` $\rightarrow 1{,}25\text{ px}$ reales) impide que el pulso verde (`phosphor_bright`) se mezcle con el cobre o el cobalto. |
| `16×16` | Bandeja del sistema, árbol de procesos y pestañas | Cada celda de $16\times 16$ del lienzo equivale a $1\text{ px}$: se conserva el anillo octogonal de latón, la mitad cálida izquierda, la mitad fría derecha, la franja verde horizontal y el punto incandescente inferior. |

---

## 4. Bocetos de cápsulas de Steam (`460×215` y `616×353`)

### 4.1. Justificación estratégica: mostrar sabotaje activo frente a la ambigüedad de *HUSH*

El nombre **HUSHMINE** transmite inmediatamente intriga y silencio, pero si la portada mostrara únicamente a un minero agachado en la oscuridad, el comprador de Steam inferiría erróneamente un juego lento de infiltración en solitario.

Para comunicar desde el primer vistazo la verdadera naturaleza del juego —**minería competitiva por equipos, robo de botín y sabotaje sucio bajo riesgo de erupción volcánica**—, los tres bocetos cumplen cuatro reglas compositivas:

1. **Acción de sabotaje en primer plano:** unas cizallas cortando el cable de cangilones rival o un Zapador accionando un detonador frente a un puente demolido.
2. **Consecuencia económica visible:** cangilones rivales cargados de oro (`gold_gleam`) y diamantes (`diamond_white` / `diamond_cyan`) precipitándose hacia la lava.
3. **Lectura del Sismógrafo al límite (`100% WAKE`):** un monitor CRT integrado en la composición con la onda en `lava_core` y marco `hazard_red`.
4. **Contraste de facciones Copper vs. Cobalt:** el saboteador viste siempre el naranja cálido de Copper atacando la línea azul fría de Cobalt, con separación de luminancia y matiz apta para deuteranopía y protanopía.

### 4.2. Especificación de los 6 archivos de cápsula

Cada uno de los tres bocetos se genera en las dos resoluciones estándar de tienda:
- **Header Capsule (`460×215`, `viewBox="0 0 460 215"`):** cabecera horizontal de la ficha de producto en Steam.
- **Main / Small Capsule (`616×353`, `viewBox="0 0 616 353"`):** cápsula de portada y carrusel con mayor rango vertical.

| Boceto | Archivos generados | Composición detallada | Jerarquía visual (1.º → 2.º → 3.º) |
|---|---|---|---|
| **Boceto 1: Corte de la línea de cangilones sobre el magma** | `capsule_1_shears_460x215.svg`<br>`capsule_1_shears_616x353.svg` | Primer plano diagonal de las cizallas del Zapador Copper (`copper_bright`, hojas `enamel_cream` y `steel_light`, perno `brass_bright`) cortando el cable de la línea de cangilones Cobalt (`cobalt_glow`). Saltan chispas (`lava_core`) en el punto de corte; dos cangilones cargados de oro y diamantes caen hacia el río de lava inferior (`magma_crust`, `lava_red`, `lava_orange`, `lava_core`). Arriba a la derecha, el módulo CRT muestra el logotipo **HUSHMINE** y el pulso `100% WAKE · CUT THE LINE`. | **1.º** Cizallas y chispas de corte en el centro.<br>**2.º** Logotipo **HUSHMINE** + Sismógrafo `100% WAKE`.<br>**3.º** Cangilones cayendo al río de magma. |
| **Boceto 2: Sabotaje cruzado y carga de demolición** | `capsule_2_bridge_blast_460x215.svg`<br>`capsule_2_bridge_blast_616x353.svg` | Caverna en estratos iluminada por haces diagonales de vapor de sodio (`peat_dark`, `basalt_deep`). A la izquierda, la silueta blindada del Zapador Copper (con visor `sodium_warm`) empuja el émbolo en T de un detonador de latón (`brass_mid`, `brass_bright`). Una línea de ignición (`lava_core`) conecta con la bola de fuego (`lava_red`, `lava_orange`, `lava_core`) que parte en dos el poste de relevo Cobalt. Placa superior con logotipo **HUSHMINE** y monitor CRT `WAKE · BRIDGE BLAST`. | **1.º** Explosión del poste Cobalt y cangilón desprendido.<br>**2.º** Zapador Copper accionando el detonador.<br>**3.º** Placa de logotipo **HUSHMINE** y alerta CRT. |
| **Boceto 3: Tensión en el Sismógrafo y caída al Corazón** | `capsule_3_split_heist_460x215.svg`<br>`capsule_3_split_heist_616x353.svg` | Composición partida en diagonal por una viga de latón (`brass_bright`). El triángulo superior izquierdo muestra el tambor CRT del Sismógrafo del Vigía (`crt_bg`, `crt_grid`) con la onda pasando de `phosphor_bright` a `lava_core` bajo el aviso `SEISMOGRAPH · 100% WAKE`. El triángulo derecho muestra la caverna profunda donde un minero Copper corta el cable Cobalt justo encima del Corazón geométrico incandescente. Faldón inferior con el logotipo **HUSHMINE** y el lema `"ROB A SLEEPING MOUNTAIN. PLAY DIRTY. KEEP IT HUSHED."` | **1.º** División diagonal entre el Sismógrafo al rojo y el Corazón de lava.<br>**2.º** Logotipo **HUSHMINE** en el faldón inferior.<br>**3.º** Lema en inglés de triple mandato. |

---

## 5. Tipografía y licencias abiertas (SIL OFL-1.1)

### 5.1. Pila tipográfica oficial

Todas las fuentes seleccionadas para **HUSHMINE** se distribuyen bajo la **SIL Open Font License 1.1 (OFL-1.1)**, lo que permite empaquetarlas sin restricciones en los recursos del cliente y utilizarlas en material promocional sin costes de licencia por impresiones ni por copias vendidas:

1. **Logotipo (`HUSHMINE`):** geometría vectorial propia (`_wordmark_paths` en `tools/ui/brand.py`), libre de dependencias de archivos de fuentes.
2. **Titulares, rótulos de latón y alertas (`Chakra Petch`, Bold 700 / SemiBold 600):**
   - *Justificación:* sus chaflanes mecánicos a $45^\circ$ en las curvas exteriores encajan con los octógonos de las fichas de 1931 y los chasis de acero de 1979.
3. **Telemetría CRT, marcadores numéricos, semillas y cronómetro (`Share Tech Mono` / `JetBrains Mono`, Bold 700 / Regular 400):**
   - *Justificación:* avance tabular constante (`monospace`) imprescindible para lecturas de frecuencia (`12 HZ`), porcentajes (`100% WAKE`), marcadores de mineral y tablas de la Crónica.
4. **Cuerpo de interfaz y localización internacional (`Noto Sans` / `Noto Sans CJK`):**
   - *Justificación:* cobertura completa de alfabetos latino extendido, cirílico y CJK para las cadenas localizadas mediante **Fluent**, manteniendo legibilidad alta en `1280×800` (Steam Deck).

### 5.2. Auditoría de contraste WCAG 2.1 AA en los activos de marca

Cada etiqueta `<text>` emitida por `tools/ui/brand.py` incluye los atributos `fill="#RRGGBB"` y `data-bg="#RRGGBB"` pertenecientes a `ALL_PALETTE_HEX` y se valida en tiempo de generación con `check_wcag_aa(fg, bg, minimum=4.5)`:

| Par de colores (`fill` sobre `data-bg`) | Códigos Hex | Razón de contraste WCAG 2.1 | Uso en activos de `assets/ui/brand/` |
|---|---|---|---|
| `enamel_cream` sobre `void_black` | `#EDE6D3` / `#0A0B0E` | **16,45 : 1** (AAA) | Subtítulos de `logo_a_stencil.svg` y lema de `capsule_3_split_heist_*.svg` |
| `lava_core` sobre `crt_bg` | `#FFF08A` / `#07120A` | **17,52 : 1** (AAA) | Alertas `100% WAKE` en `logo_b_seismo.svg` y en las 6 cápsulas de Steam |
| `enamel_cream` sobre `crt_bg` | `#EDE6D3` / `#07120A` | **16,66 : 1** (AAA) | Pie técnico de `logo_b_seismo.svg` |
| `phosphor_bright` sobre `crt_bg` | `#68F08E` / `#07120A` | **13,27 : 1** (AAA) | Telemetría `CRT SEISMOGRAPH P1 · STIR 12 HZ` en `logo_b_seismo.svg` |
| `brass_bright` sobre `void_black` | `#D9B052` / `#0A0B0E` | **9,33 : 1** (AAA) | Sello `"ANIMA CONS."` en `logo_a_stencil.svg` |
| `copper_glow` sobre `void_black` | `#F09246` / `#0A0B0E` | **8,06 : 1** (AAA) | Facción `"COPPER SYNDICATE"` en `logo_c_chasm.svg` |
| `cobalt_glow` sobre `void_black` | `#5CA3F2` / `#0A0B0E` | **7,35 : 1** (AAA) | Facción `"COBALT TRUST"` en `logo_c_chasm.svg` |
| `ink_stamp` sobre `brass_bright` | `#1C1917` / `#D9B052` | **7,21 : 1** (AAA) | Número troquelado `"#41"` en `logo_a_stencil.svg` y `logo_c_chasm.svg` |

---

## 6. Inventario canónico de archivos en `assets/ui/brand/`

La constante `BRAND_SPECS` de `tools/ui/brand.py` define el contrato exacto de dimensiones y nombres de archivo verificados por la batería de pruebas de interfaz:

| Archivo en `assets/ui/brand/` | Dimensiones (`width × height`) | `viewBox` exigido | Función principal |
|---|---|---|---|
| `logo_a_stencil.svg` | `800 × 240` | `0 0 800 240` | Logotipo principal recomendado (troquel de latón de 1931 y fractura sísmica) |
| `logo_b_seismo.svg` | `800 × 240` | `0 0 800 240` | Variante diegética de menú (osciloscopio CRT P1 de 1979 y onda en fase `WAKE`) |
| `logo_c_chasm.svg` | `800 × 240` | `0 0 800 240` | Variante competitiva de facciones (Copper vs. Cobalt separados por la ficha `#41`) |
| `app_icon.svg` | `256 × 256` | `0 0 256 256` | Icono cuadrado de aplicación y ventana escalable de `256×256` a `16×16` |
| `capsule_1_shears_460x215.svg` | `460 × 215` | `0 0 460 215` | Boceto 1 (*Header Capsule*): cizallas Copper cortando la línea de cangilones Cobalt |
| `capsule_1_shears_616x353.svg` | `616 × 353` | `0 0 616 353` | Boceto 1 (*Main Capsule*): cizallas Copper cortando la línea de cangilones Cobalt |
| `capsule_2_bridge_blast_460x215.svg` | `460 × 215` | `0 0 460 215` | Boceto 2 (*Header Capsule*): demolición con detonador del puente y poste rival |
| `capsule_2_bridge_blast_616x353.svg` | `616 × 353` | `0 0 616 353` | Boceto 2 (*Main Capsule*): demolición con detonador del puente y poste rival |
| `capsule_3_split_heist_460x215.svg` | `460 × 215` | `0 0 460 215` | Boceto 3 (*Header Capsule*): composición diagonal Sismógrafo + robo en el Corazón |
| `capsule_3_split_heist_616x353.svg` | `616 × 353` | `0 0 616 353` | Boceto 3 (*Main Capsule*): composición diagonal Sismógrafo + robo en el Corazón |

### Garantías técnicas de los activos SVG
1. **Validación XML estricta:** todos los documentos se analizan con `xml.etree.ElementTree.fromstring` antes de escribirse en disco.
2. **Aislamiento de red (cero recursos externos):** ningún SVG incluye referencias `http://` ni `https://` salvo la declaración estándar del espacio de nombres `xmlns="http://www.w3.org/2000/svg"`.
3. **Paleta cerrada:** todos los atributos cromáticos (`fill`, `stroke`, `data-bg`) provienen exclusivamente de `ALL_PALETTE_HEX`.

---

## 7. Regeneración y verificación determinista

Para regenerar los 10 archivos SVG de marca y comprobar su validez estructural:

```bash
python tools/ui/brand.py
```

El generador verifica automáticamente que cada archivo tenga el `viewBox` exacto especificado en `BRAND_SPECS`, que todos los colores pertenezcan a `ALL_PALETTE_HEX`, que el contraste WCAG 2.1 supere $4{,}5:1$ y que los archivos se escriban con saltos de línea LF (`\n`).
