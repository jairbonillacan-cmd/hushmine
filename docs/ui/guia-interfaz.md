# Guía de Diseño de Interfaz y Marca — HUSHMINE (Motor Grisú)

Este documento define las reglas visuales, ergonómicas, cromáticas y de accesibilidad para la interfaz de usuario (HUD, menús, pantallas de transición y marca corporativa) de **HUSHMINE**. Toda la interfaz se genera y valida de forma determinista desde `tools/ui/` empleando la paleta unificada de 60 colores (`ALL_PALETTE_HEX` en `tools/ui/ui_palette.py`).

---

## 1. Dirección Estética: Instrumentación Analógica (1931 / 1979)

La interfaz de HUSHMINE evita el minimalismo flotante moderno: cada indicador responde a la **estratigrafía material** del pozo minero:

1. **Estrato de 1931 (*Anima Consolidated*):**
   - **Chapas de latón troqueladas (`brass_*`):** fichas octogonales numeradas del `01` al `41` (*Brass tallies*) de los mineros desaparecidos en el cierre de 1931, placas remachadas de inventario y sellos de la compañía.
   - **Cartelería de esmalte vitrificado (`enamel_cream`, `ink_stamp`, `hazard_red`):** letreros industriales de prevención de riesgos (`SIGN-01`..`SIGN-10`) fijados sobre bastidores de acero y madera creosotada.
   - **Papel continuo de tambor (`paper_aged`):** bobinas perforadas donde la aguja mecánica y la impresora matricial registran la **Crónica** (*Chronicle*) del Turno.
2. **Estrato de 1979 (Incursión clandestina):**
   - **Monitores CRT de fósforo verde P1 (`crt_bg`, `crt_grid`, `phosphor_*`):** tubos de rayos catódicos encastrados en chasis de acero (`steel_dark`, `steel_mid`) para el **Sismógrafo compartido** y el geófono.
   - **Lámparas de vapor de sodio (`sodium_warm`):** indicadores ámbar de tensión, avisos de radio del **Vigía** (*The Watcher*) y contadores electromecánicos de banco.

### Principio de mínimo texto («Un icono para cada cosa»)
En plena excavación bajo niebla densa, el jugador no lee párrafos: interpreta **siluetas, posiciones fijas y saltos de luminancia**.
- Todo estado de juego cuenta con un **icono dedicado** dentro del catálogo canónico de 25 iconos (`ICON_STEMS`).
- El idioma fuente del texto visible es **inglés** (con equivalencia en español y claves de localización **Project Fluent**), restringiendo las intervenciones de radio del Vigía a un máximo de **6 palabras**.

---

## 2. Paleta de Interfaz (`tools/ui/ui_palette.py`)

El módulo `tools/ui/ui_palette.py` importa sin alterar los **48 colores de bloques** de `tools/textures/palette.py` (`PALETTE_HEX`) y añade **12 colores exclusivos de interfaz** (`UI_PALETTE_HEX`), conformando un espacio cerrado de **60 colores** (`ALL_PALETTE_HEX`).

La luminancia relativa WCAG 2.1 ($L \in [0.0, 1.0]$) y la razón de contraste ($CR \in [1.0, 21.0]$) se calculan según la especificación oficial W3C:

$$L = 0.2126 \cdot R_{\text{lin}} + 0.7152 \cdot G_{\text{lin}} + 0.0722 \cdot B_{\text{lin}}, \qquad CR = \frac{L_{\text{claro}} + 0.05}{L_{\text{oscuro}} + 0.05}$$

| # | Identificador | Hex | RGB `(R, G, B)` | $L_{\text{WCAG}}$ | Contraste vs `void_black` (`#0A0B0E`) | Contraste vs `crt_bg` (`#07120A`) | Contraste vs `paper_aged` (`#D8CEB8`) | Uso principal en la interfaz |
|---:|:---|:---:|:---:|---:|---:|---:|---:|:---|
| 1 | `crt_bg` | `#07120A` | `(7, 18, 10)` | `0.0050` | `1.03:1` | `1.00:1` | `12.22:1` | Fondo de pantalla CRT del sismógrafo y visores tácticos |
| 2 | `crt_grid` | `#12301B` | `(18, 48, 27)` | `0.0232` | `1.37:1` | `1.33:1` | `9.18:1` | Retícula grabada del osciloscopio (decorativa, no textual) |
| 3 | `phosphor_dim` | `#1E6F36` | `(30, 111, 54)` | `0.1191` | `3.17:1` | `3.07:1` | `3.97:1` | Estela secundaria de barrido y bordes pasivos del tubo CRT |
| 4 | `phosphor_mid` | `#34B85A` | `(52, 184, 90)` | `0.3575` | `7.64:1` | `7.41:1` | `1.65:1` | Trazo activo de onda en fase *Stir* (`25 %`) y etiquetas CRT |
| 5 | `phosphor_bright` | `#68F08E` | `(104, 240, 142)` | `0.6722` | `13.53:1` | `13.13:1` | `1.07:1` | Lectura numérica principal en pantallas CRT (WCAG AAA) |
| 6 | `phosphor_peak` | `#C8FFD6` | `(200, 255, 214)` | `0.8865` | `17.55:1` | `17.03:1` | `1.39:1` | Cresta de pulso sísmico y destello del haz electrónico |
| 7 | `brass_dark` | `#594318` | `(89, 67, 24)` | `0.0620` | `2.10:1` | `2.04:1` | `6.00:1` | Bisel en sombra y troquelado interior de chapas de latón |
| 8 | `brass_mid` | `#9E7B2F` | `(158, 123, 47)` | `0.2164` | `4.99:1` | `4.84:1` | `2.52:1` | Cuerpo bruñido de fichas de 1931 y marcos de medallas |
| 9 | `brass_bright` | `#D9B052` | `(217, 176, 82)` | `0.4641` | `9.64:1` | `9.35:1` | `1.31:1` | Arista especular de latón y numeración resaltada sobre oscuro |
| 10 | `enamel_cream` | `#EDE6D3` | `(237, 230, 211)` | `0.7930` | `15.80:1` | `15.33:1` | `1.25:1` | Texto principal sobre fondos oscuros y fondo de cartel esmaltado |
| 11 | `paper_aged` | `#D8CEB8` | `(216, 206, 184)` | `0.6220` | `12.60:1` | `12.22:1` | `1.00:1` | Papel continuo de la Crónica y gráficas de tambor |
| 12 | `ink_stamp` | `#1C1917` | `(28, 25, 23)` | `0.0100` | `1.13:1` | `1.09:1` | `11.19:1` | Tinta carbón sobre `paper_aged` (`11.19:1`) y `enamel_cream` (`14.13:1`) |

### Combinación con los 48 colores de `palette.py`
- **Chasis y paneles:** `void_black` (`#0A0B0E`), `soot_dark` (`#14161B`), `steel_dark` (`#292E38`), `steel_mid` (`#485060`) y `steel_light` (`#6C778C`).
- **Equipos:** **Copper** usa `copper_dark`..`copper_glow` (`#5E2C18`..`#F09246`); **Cobalt** usa `cobalt_dark`..`cobalt_glow` (`#1A2E54`..`#5CA3F2`).
- **Recursos y alertas:** mena con `ore_amber`/`ore_highlight`, oro con `gold_bright`/`gold_gleam`, diamante con `diamond_cyan`/`diamond_white`, peligro crítico y fase *Wake* con `hazard_red` (`#C9302C`), `lava_orange` (`#F28A2E`) y `lava_core` (`#FFF08A`).

---

## 3. Jerarquía de la Información en Partida (HUD)

El HUD organiza sus elementos en tres niveles de prioridad visual para mantener despejado el **90 % central** de la vista en primera persona:

### Nivel 1 — Crítico / Siempre visible (Banda superior central)
1. **Sismógrafo compartido (`0` a `100_000` milésimas = `0.0 %` a `100.0 %`):**
   - Monitor CRT central con onda en tiempo real y cuatro marcadores de umbral:
     - **`25 %` (`25_000`) — `Stir` / Revuelo** (`seismo-stir`, icono `seismo_stir`, trazo `phosphor_mid`).
     - **`50 %` (`50_000`) — `Breathe` / Respira** (`seismo-breathe`, icono `seismo_breathe`, trazo `sodium_warm`).
     - **`75 %` (`75_000`) — `Restless` / Inquietud** (`seismo-restless`, icono `seismo_restless`, trazo `lava_orange`).
     - **`100 %` (`100_000`) — `Wake` / Despertar** (`seismo-wake`, icono `seismo_wake`, alarma `hazard_red` + `lava_core`). Activa el **reloj de cuenta atrás de `90 s` (`2_700` ticks)** hacia la jaula de extracción; a falta de **`30 s` (`900` ticks)** las líneas de cangilones se detienen.
2. **Bancos de los dos equipos (`hud_bank`) y estado de Fama (`status_fame`):**
   - Flanquean al sismógrafo: **COPPER CREW** a la izquierda (marco biselado a 45° con chevrones) y **COBALT CREW** a la derecha (marco ortogonal a 90° con cruces `+`).
   - Muestran el efectivo depositado frente a la meta de la partida (`$6,500` en 4v4; escala de `$2,000` en 1v1 a `$12,500` en 8v8).
   - Al alcanzar el **`60 %` de la meta** (`cash * 100 >= target * 60`, p. ej. `$3,900` en 4v4), se enciende el sello de **Fama** (*Fame* / Fama, `hud-fame-active`, icono `status_fame`).

### Nivel 2 — Táctico / Contextual (Subtítulo de radio bajo el sismógrafo)
- **Avisos del Vigía (*The Watcher*):** banda compacta estilo receptor de radio con indicador `sodium_warm` y texto en `enamel_cream` sobre fondo `soot_dark`.
- Máximo **6 palabras en inglés** (ej.: `"Fifty percent. It just breathed."` / «Cincuenta por ciento. Acaba de respirar.», clave Fluent `watcher-seismo-breathe`; `"Thirty seconds. Bucket lines dead."` / «Treinta segundos. Cangilones muertos.», clave Fluent `watcher-wake-buckets-stopped`).

### Nivel 3 — Estado personal (Banda inferior)
1. **Carga actual (Inferior izquierda):**
   - **Mena (`res_ore`):** contador `0..max_ore` (`4` uds/bloque; se gasta al construir).
   - **Tesoro (`res_gold`, `res_diamond`):** lingotes y gemas en mochila (`0..max_bullion`; oro `$100`, gema de diamante `$200`, bloque de diamante `$400`, Veta Madre `$2,000`, El Corazón `$1,500`) y valor en dólares pendiente de ingresar.
   - **Indicador de botín robado:** cuando el jugador porta tesoro sustraído al banco rival, muestra la etiqueta **`STOLEN ×1.25`** (*Botín robado ×1,25*, clave `hud-stolen-bonus`: lingote `$125`, gema `$250`).
2. **Clase activa, herramientas y costes (Inferior centro):**
   - Emblema de clase (`class_prospector`, `class_miner`, `class_engineer`, `class_sapper`), par de herramientas equipadas y regleta rápida de costes en mena de los bloques construibles (`team_block`, `ladder`, `force_field`, `jump_pad`, `beacon`, `explosive`, `bucket_station`, `bucket_post`).
3. **Fichas de latón y Ofrenda al altar (Inferior derecha):**
   - **Fichas de latón (`item_brass_tally`):** ranuras `0/2` para el Turno actual (cada ficha entregada en la caseta reduce `-8_000` = **`-8.0 %`** el sismógrafo; colección global de `41` fichas de 1931).
   - **Acción de Ofrenda (`action_offering`):** indicador contextual junto a altares subterráneos (`OFFERING $100 → -2.5%` / «Ofrenda 100 $ → -2,5 %», clave `hud-action-offering`, enfriamiento `5 s` = `150` ticks).

---

## 4. Catálogo de las 5 Pantallas y 2 Resoluciones (`MOCKUP_SCREENS`)

Cada pantalla del juego se especifica y genera como maqueta vectorial SVG en las dos resoluciones objetivo (`1920×1080` escritorio 16:9 y `1280×800` Steam Deck 16:10):

| ID (`MOCKUP_SCREENS`) | Archivos SVG (`assets/ui/mockups/`) | Propósito y elementos obligatorios |
|:---|:---|:---|
| `hud` | `hud_1920x1080.svg`<br>`hud_1280x800.svg` | **HUD en partida:** vista en primera persona con mira central, Sismógrafo CRT (`Stir`/`Breathe`/`Restless`/`Wake`), bancos Copper y Cobalt con meta `$6,500` y sello de Fama (`60 %`), subtítulo de radio del Vigía, inventario de mena/oro/diamante (`STOLEN ×1.25`), ranura de clase y fichas de latón (`0..2`) + Ofrenda (`-2.5 %`). |
| `drop_in` | `drop_in_1920x1080.svg`<br>`drop_in_1280x800.svg` | **Bajada en la jaula (*Drop-in*):** cuenta atrás de **`20 s`**, corte geológico esquemático del mapa por estratos (superficie, galerías, Veta Madre, cámara de El Corazón y magma) y tarjetas de selección de las 4 clases con sus capacidades exactas (`Prospector 20/10`, `Miner 40/20`, `Engineer 30/10`, `Sapper 20/10`). |
| `chronicle` | `chronicle_1920x1080.svg`<br>`chronicle_1280x800.svg` | **Crónica del Turno (*Shift Chronicle*):** hoja de papel continuo (`paper_aged`) con curva histórica del sismógrafo, hitos del `EventLog` (primera veta de oro, Fama al 60 %, robo de banco, ofrendas, Despertar), resultado de evacuación y los **4 títulos honoríficos** con su icono: *The Wakener* (`title_wakener`), *Sleep Guardian* (`title_sleep_guardian`), *The Mole* (`title_mole`) y *Bucket Thief* (`title_bucket_thief`). |
| `main_menu` | `main_menu_1920x1080.svg`<br>`main_menu_1280x800.svg` | **Menú principal (Caseta del Vigía):** logotipo industrial de **HUSHMINE**, botonera de modos (**`SHIFT`** / Turno competitivo, **`PRACTICE`** / Práctica, **`WORKSHOP`** / Taller de mapas y **`QUIT`**), monitor de estado y tablón de madera con las **41 alcayatas de fichas de latón de 1931**. |
| `loading` | `loading_1920x1080.svg`<br>`loading_1280x800.svg` | **Pantalla de carga:** cartel de seguridad esmaltado de *Anima Consolidated* (`SIGN-01`..`SIGN-10`, p. ej. `SIGN-04: "DO NOT FEED THE FAULT — OFFERINGS Strictly Regulated"` / «NO ALIMENTE LA FALLA»), barra de progreso de presurización de jaula y sello del motor **Grisú**. |

---

## 5. Retícula, Tamaños Mínimos para Steam Deck (`1280×800`) y Márgenes Seguros

Toda la interfaz se alinea sobre una **retícula base de `8 px`** (con submúltiplo de `4 px` para bordes de chasis y separadores):

| Parámetro ergonómico | Escritorio (`1920×1080`, 16:9) | Steam Deck (`1280×800`, 16:10) | Regla de verificación |
|:---|:---:|:---:|:---|
| **Margen seguro perimetral** | `48 px` por cada borde | `32 px` por cada borde | Ningún texto ni icono cae fuera del rectángulo interior (`1824×984` o `1216×736`) |
| **Zona libre central (HUD)** | Banda central despejada | Banda central despejada | Los paneles del HUD se ciñen al borde superior (`y <= 234`) e inferior (`y >= H - 220`) |
| **Fuente: etiquetas secundarias** | $\ge 14\text{ px}$ | **$\ge 14\text{ px}$** | Teclas de acceso rápido, unidades y notas al pie |
| **Fuente: lecturas de HUD y Vigía** | $16\text{–}22\text{ px}$ | **$15\text{–}18\text{ px}$** | Subtítulos de radio (`watcher-*`), carga de recursos y nombres de clase |
| **Fuente: cabeceras y contadores** | $20\text{–}30\text{ px}$ | **$18\text{–}24\text{ px}$** | Dólares en banco, porcentaje del sismógrafo y titulares principales |
| **Tamaño de iconos en HUD** | `32×32 px` (`64×64` en emblemas) | **`32×32 px` mínimo** | Sprite `32×32` nativo o `16×16` escalado `×2` por vecino más próximo |
| **Área interactiva (botones/mando)** | $\ge 48\times 48\text{ px}$ | **$\ge 44\times 44\text{ px}$** | Separación mínima de `8 px` entre tarjetas o botones adyacentes |

---

## 6. Accesibilidad, Redundancia Visual y Catálogo de 25 Iconos

### 6.1. Cumplimiento WCAG 2.1 AA y daltonismo (Protanopia / Deuteranopia)
1. **Contraste de texto e iconos informativos:** toda combinación de texto o pictograma sobre su panel de fondo debe alcanzar **$CR \ge 4.5:1$** (WCAG AA) y preferiblemente **$CR \ge 7.0:1$** (WCAG AAA), verificable con `check_wcag_aa(fg, bg)` en `tools/ui/ui_palette.py`.
2. **Doble codificación de equipos:**
   - **Copper Crew:** gama cálida naranja (`#5E2C18`..`#F09246`) + **bandas diagonales a 45° (chevrones `//`)** + **remaches y marcos circulares/octogonales**.
   - **Cobalt Crew:** gama fría azul (`#1A2E54`..`#5CA3F2`) + **cruces ortogonales a 90° (`+`)** + **marcos cuadrados con esquinas en escuadra**.
3. **Cuádruple redundancia en el Sismógrafo:** cada umbral modifica simultáneamente:
   - **Color:** `phosphor_mid` (`25 %`) $\to$ `sodium_warm` (`50 %`) $\to$ `lava_orange` (`75 %`) $\to$ `hazard_red`/`lava_core` (`100 %`).
   - **Icono dedicado:** `seismo_stir` (onda simple) $\to$ `seismo_breathe` (doble onda simétrica) $\to$ `seismo_restless` (diente de sierra agudo) $\to$ `seismo_wake` (fractura vertical con destello).
   - **Frecuencia y amplitud de la curva CRT:** de senoidal suave a picos saturados contra el borde del visor.
   - **Marco del instrumento:** borde simple en reposo, doble línea en `Restless` y banda diagonal de emergencia en `Wake`.

### 6.2. Catálogo Canónico de los 25 Iconos (`ICON_STEMS`)
Todos los iconos se generan en **`16×16`** (`assets/ui/icons/<stem>_16.png`) y **`32×32`** (`assets/ui/icons/<stem>_32.png`) con silueta contrastada sobre placa industrial usando exclusivamente `ALL_PALETTE_SET`:

| # | Identificador (`stem`) | Categoría | Etiqueta EN / ES (Clave Fluent) | Geometría distintiva y colores clave |
|---:|:---|:---|:---|:---|
| 1 | `class_prospector` | Clase | *Prospector* / Prospector (`class-prospector`) | Diapasón/antena de geófono con ondas concéntricas (`phosphor_bright`, `brass_bright`) |
| 2 | `class_miner` | Clase | *Miner* / Minero (`class-miner`) | Pico pesado de doble punta cruzado sobre escudo octogonal (`steel_light`, `ore_highlight`) |
| 3 | `class_engineer` | Clase | *Engineer* / Ingeniero (`class-engineer`) | Llave de cangilones y rueda dentada de 8 dientes (`sodium_warm`, `steel_light`) |
| 4 | `class_sapper` | Clase | *Sapper* / Zapador (`class-sapper`) | Detonador de émbolo en T con chispa eléctrica (`hazard_red`, `lava_yellow`) |
| 5 | `res_ore` | Recurso | *Ore* / Mena (`res-ore`) | Bloque quebrado con veta diagonal a 45° (`ore_rust_dark`..`ore_highlight`) |
| 6 | `res_gold` | Recurso | *Gold Bullion* / Lingote de oro (`res-gold`) | Lingote trapezoidal biselado con destello superior (`gold_shadow`..`gold_gleam`) |
| 7 | `res_diamond` | Recurso | *Diamond Gem* / Diamante (`res-diamond`) | Cristal octaédrico/romboidal facetado con cruz interna (`diamond_deep`..`diamond_white`) |
| 8 | `hud_bank` | HUD | *Team Bank* / Banco de equipo (`hud-bank`) | Caja fuerte acorazada con volante de 4 radios (`steel_mid`, `brass_bright`) |
| 9 | `item_brass_tally` | Objeto 1931 | *Brass Tally* / Ficha de latón (`item-brass-tally`) | Chapa octogonal troquelada con orificio superior y marca `31` (`brass_dark`..`brass_bright`) |
| 10 | `action_offering` | Acción | *Fault Offering* / Ofrenda (`action-offering`) | Cáliz/hendidura de altar recibiendo lingote hacia abajo (`gold_bright`, `phosphor_bright`) |
| 11 | `item_explosive` | Herramienta | *Dynamite Charge* / Carga explosiva (`item-explosive`) | Tres cartuchos atados con mecha encendida (`hazard_red`, `lava_core`) |
| 12 | `tool_geophone` | Herramienta | *Geophone* / Geófono (`tool-geophone`) | Esfera de osciloscopio portátil con retícula y pulso verde (`crt_bg`, `phosphor_bright`) |
| 13 | `status_fame` | Estado | *Fame (60%)* / Fama (`status-fame`) | Estrella industrial de 6 puntas sobre corona de latón (`gold_gleam`, `sodium_warm`) |
| 14 | `seismo_stir` | Sismógrafo | *25% Stir* / Revuelo (`seismo-stir`) | Onda senoidal suave de baja amplitud (`crt_bg`, `phosphor_mid`) |
| 15 | `seismo_breathe` | Sismógrafo | *50% Breathe* / Respira (`seismo-breathe`) | Doble onda de expansión torácica simétrica (`sodium_warm`, `phosphor_bright`) |
| 16 | `seismo_restless` | Sismógrafo | *75% Restless* / Inquietud (`seismo-restless`) | Trazo en diente de sierra de alta frecuencia (`lava_orange`, `lava_yellow`) |
| 17 | `seismo_wake` | Sismógrafo | *100% Wake* / Despertar (`seismo-wake`) | Grieta sísmica vertical abierta con pico incandescente (`hazard_red`, `lava_core`) |
| 18 | `death_lava` | Crónica | *Magma Incineration* / Lava (`death-lava`) | Cráneo/casco sumergido en triple lengua magmática (`lava_red`..`lava_core`) |
| 19 | `death_electric` | Crónica | *High Voltage* / Electrocución (`death-electric`) | Rayo quebrado de alta tensión en triángulo aislador (`electric_blue`, `diamond_white`) |
| 20 | `death_fall` | Crónica | *Shaft Fall* / Caída libre (`death-fall`) | Silueta descendente con triple chevron de velocidad (`enamel_cream`, `hazard_red`) |
| 21 | `death_cave_in` | Crónica | *Cave-In / Blast* / Derrumbe (`death-cave-in`) | Bloques de basalto desplomándose sobre casco minero (`slate_light`, `hazard_red`) |
| 22 | `title_wakener` | Título | *The Wakener* / El Despertador (`title-wakener`) | Campana/sirena quebrada sobre aguja sísmica al 100 % (`hazard_red`, `lava_yellow`) |
| 23 | `title_sleep_guardian` | Título | *Sleep Guardian* / Guardián del sueño (`title-sleep-guardian`) | Escudo de latón con onda plana en calma y llave (`brass_bright`, `phosphor_bright`) |
| 24 | `title_mole` | Título | *The Mole* / El Topo (`title-mole`) | Pala/garra excavadora abriendo galería profunda (`ore_highlight`, `enamel_cream`) |
| 25 | `title_bucket_thief` | Título | *Bucket Thief* / Ladrón de cangilones (`title-bucket-thief`) | Cangilón inclinado en gancho con destello de oro robado (`gold_gleam`, `copper_glow`) |
