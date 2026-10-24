# Diseño de Clases y Economía de HUSHMINE

Este documento especifica las reglas deterministas de clases, herramientas, economía doble, metas por tamaño de equipo y mecánicas de remontada de **HUSHMINE**. Los datos canónicos residen en `data/classes.ron` y `data/economy.ron`.

---

## 1. Convenciones de Simulación y Punto Fijo

Toda la simulación de `hush-sim` opera a **30 ticks por segundo** (`tick_rate: 30`) utilizando exclusivamente aritmética entera:

- **Coordenadas y velocidades (`Fixed`):** entero con signo `i32` en formato `Q16.16`, donde `1` bloque equivale a `65_536` subbloques.
- **Unidades económicas y tiempos:** enteros sin signo `u32` (unidades de mena, lingotes/gemas, dólares del banco y ticks).
- **Proporciones y multiplicadores:** expresados como pares `(numerador, denominador)` evaluados mediante división entera truncada.
- **Sin armas de fuego:** ningún jugador dispone de ataques letales directos. El golpe de pico aplica únicamente un empuje horizontal de `32_768` subbloques (`0,5` bloques) para desplazar rivales hacia fosos, lava o trampas.

---

## 2. Las Cuatro Clases

Cada jugador elige su clase antes de saltar del vagón aéreo o al reaparecer. Todas comparten una velocidad vertical de salto de `19_660` subbloques/tick (`0,30` bloques/tick, equivalentes a `9,0` bloques/s iniciales) y un empuje de pico de `32_768` subbloques.

### 2.1 Resumen de Atributos y Herramientas

| Clase (`id`) | Nombre (ES) | Velocidad (`Q16.16`) | Bloques/s (aprox.) | Cap. Mena (`max_ore`) | Cap. Tesoro (`max_bullion`) | Herramienta Primaria | Herramienta Secundaria |
|---|---|---:|---:|---:|---:|---|---|
| `Prospector` | Prospector | `9_284` | `4,25` | `20` | `10` | `"mineral_detector"` | `"geophone"` |
| `Miner` | Minero | `8_738` | `4,00` | `40` | `20` | `"heavy_pickaxe"` | `"cargo_harness"` |
| `Engineer` | Ingeniero | `8_738` | `4,00` | `30` | `10` | `"builder_hammer"` | `"bucket_wrench"` |
| `Sapper` | Zapador | `9_011` | `4,125` | `20` | `10` | `"blasting_cap"` | `"wire_cutters"` |

### 2.2 Papel Táctico y Herramientas Especiales

1. **Prospector (`Prospector`):**
   - **Papel táctico:** explorador veloz y guía del equipo. Abre rutas de reconocimiento y señala filones ricos con balizas (`beacon`).
   - **Detector de minerales (`mineral_detector`):** escanea un cono frontal corto a través de roca sólida e indica la presencia de mena (`ore`), oro (`gold`) o diamante (`diamond`).
   - **Geófono (`geophone`):** al apoyarse contra una pared o suelo, capta vibraciones acústicas de picos, pisadas, líneas de cangilones y explosivos en un radio amplio, permitiendo localizar excavaciones rivales o emboscadas.

2. **Minero (`Miner`):**
   - **Papel táctico:** columna vertebral de la extracción y el acarreo. Es la única clase capaz de perforar roca dura (`hard_rock`) en tiempo competitivo (`45` ticks frente a `90` ticks del Prospector/Ingeniero).
   - **Pico pesado (`heavy_pickaxe`):** reduce en un `33 %`–`50 %` los ticks necesarios para romper terreno natural (`dirt`, `rock`, `hard_rock`, `ore`, `gold`, `diamond`).
   - **Arnés de carga (`cargo_harness`):** duplica la capacidad de inventario tanto en mena (`40` unidades) como en oro/diamante (`20` unidades), minimizando los viajes de retorno a la superficie o a la estación de cangilones.

3. **Ingeniero (`Engineer`):**
   - **Papel táctico:** arquitecto de la red logística y de las fortificaciones defensivas. Controla el flujo vertical y horizontal del mapa.
   - **Martillo de constructor (`builder_hammer`):** permite levantar todo el catálogo de infraestructura avanzada (`force_field`, `jump_pad`, `electric`, `rail`, `beacon`, `bucket_station`, `bucket_post`) y desmontar estructuras a buen ritmo.
   - **Llave de cangilones (`bucket_wrench`):** enlaza postes (`bucket_post`) con estaciones (`bucket_station`), invierte el sentido de una línea o repara tramos saboteados.

4. **Zapador (`Sapper`):**
   - **Papel táctico:** especialista en demolición, apertura de brechas e infiltración en la red logística enemiga.
   - **Detonador y cargas (`blasting_cap`):** coloca bloques explosivos (`explosive`, coste `10` de mena) capaces de volar cámaras enteras o abrir túneles instantáneos hacia la Veta Madre.
   - **Cizalla (`wire_cutters`):** herramienta de contra-ingeniería que desactiva cargas explosivas en apenas `6` ticks (`0,2` s) sin detonarlas y desmonta campos de fuerza, bloques eléctricos y postes de cangilones enemigos más rápido que ninguna otra clase.

### 2.3 Bloques Construibles y Costes en Mena (`ore_cost`)

| ID | Bloque (`block_name`) | Coste (`ore_cost`) | `Prospector` | `Miner` | `Engineer` | `Sapper` | Función principal |
|---:|---|---:|:---:|:---:|:---:|:---:|---|
| `8` | `team_block` | `1` | Sí | Sí | Sí | Sí | Bloque estructural con color del equipo |
| `9` | `force_field` | `4` | No | No | Sí | No | Barrera permeable solo para aliados |
| `10` | `ladder` | `1` | Sí | Sí | Sí | Sí | Escalada vertical en pozos |
| `11` | `jump_pad` | `6` | No | No | Sí | No | Impulso vertical rápido |
| `12` | `electric` | `8` | No | No | Sí | No | Trampa letal por contacto o caída |
| `13` | `rail` | `2` | No | Sí | Sí | No | Desplazamiento horizontal acelerado |
| `14` | `beacon` | `5` | Sí | No | Sí | No | Señal luminosa y marcador táctico |
| `15` | `explosive` | `10` | No | No | No | Sí | Carga de demolición con mecha/detonador |
| `16` | `bucket_station` | `12` | No | No | Sí | No | Terminal de carga/descarga de cangilones |
| `17` | `bucket_post` | `3` | No | No | Sí | No | Poste intermedio de línea de cangilones |

### 2.4 Tiempos de Picado por Bloque y Clase (en ticks a 30 Hz)

Los bloques `0` (`air`) y `7` (`lava`) no se pueden picar. El resto requiere exactamente los siguientes ticks de picado continuo (idénticos en `data/classes.ron` y `data/blocks.ron`):

| ID | Bloque | `Prospector` | `Miner` | `Engineer` | `Sapper` | Notas clave |
|---:|---|---:|---:|---:|---:|---|
| `1` | `dirt` | `12` | `8` | `12` | `10` | Tierra blanda superficial |
| `2` | `rock` | `24` | `15` | `24` | `20` | Roca común del subsuelo |
| `3` | `hard_rock` | `90` | `45` | `90` | `60` | Estrato duro que protege grandes vetas |
| `4` | `ore` | `28` | `18` | `28` | `24` | Mena de construcción (`+4` unidades) |
| `5` | `gold` | `36` | `24` | `36` | `30` | Oro de banco (`1` lingote = `100 $`) |
| `6` | `diamond` | `54` | `36` | `54` | `45` | Diamante profundo (`2` gemas = `400 $`) |
| `8` | `team_block` | `24` | `15` | `18` | `12` | Estructura básica de equipo |
| `9` | `force_field` | `45` | `30` | `24` | `15` | El Zapador abre brechas en `0,5` s |
| `10` | `ladder` | `15` | `10` | `10` | `8` | Escalera de mano |
| `11` | `jump_pad` | `30` | `20` | `15` | `10` | Plataforma de salto |
| `12` | `electric` | `45` | `30` | `20` | `12` | Bloque electrificado |
| `13` | `rail` | `18` | `12` | `10` | `8` | Carril de transporte |
| `14` | `beacon` | `20` | `20` | `15` | `10` | Baliza de señalización |
| `15` | `explosive` | `30` | `30` | `24` | `6` | Cizalla del Zapador desactiva en `6` ticks |
| `16` | `bucket_station` | `45` | `35` | `20` | `15` | Estación terminal de cangilones |
| `17` | `bucket_post` | `30` | `20` | `12` | `8` | Poste de línea de cangilones |

---

## 3. Economía Doble y Línea de Cangilones

HUSHMINE separa estrictamente el **material de obra** del **tesoro de victoria**, eliminando el dilema de gastar puntos de victoria al construir.

### 3.1 Recursos del Subsuelo (`ResourceValues`)

| Recurso | Bloque origen | Unidades por bloque | Valor en banco por bloque | Valor unitario (`u32`) | Destino |
|---|---|---:|---:|---:|---|
| **Mena (`ore`)** | `4` (`ore`) | `4` unidades | `0 $` | `0 $` | Inventario personal para construir bloques |
| **Oro (`gold`)** | `5` (`gold`) | `1` unidad | `100 $` | `100 $` | Depósito en banco del equipo o estación |
| **Diamante (`diamond`)** | `6` (`diamond`) | `2` unidades | `400 $` | `200 $` | Depósito en banco del equipo o estación |

- **Hitos geológicos del mapa:**
  - **Veta Madre (`mother_lode_total_cash`):** cámara central rica en oro y diamante con un valor conjunto de `2_000 $`.
  - **Corazón (`heart_total_cash`):** núcleo profundo de altísimo riesgo rodeado de lava y roca dura con un valor conjunto de `1_500 $`.

### 3.2 Funcionamiento de la Línea de Cangilones (`BucketLineEconomy`)

En lugar de obligar a los mineros a subir a pie hasta la superficie en cada viaje, el **Ingeniero** puede tender una línea aérea de cangilones desde el frente de excavación hasta el banco del equipo:

- **Longitud máxima por línea (`max_posts_per_line`):** hasta `16` postes (`bucket_post`) conectados en serie entre dos estaciones (`bucket_station`).
- **Separación máxima entre tramos (`max_post_spacing_blocks`):** `8` bloques en línea de visión libre de obstáculos sólidos.
- **Cadencia de transferencia (`transfer_interval_ticks`):** cada `60` ticks (`2,0` s) la estación de origen despacha un cangilón cargado hacia la estación de destino.
- **Capacidad por cangilón (`units_per_bucket`):** `1` unidad de tesoro (`1` lingote de oro de `100 $` o `1` gema de diamante de `200 $`).
- **Vulnerabilidad física:** cada cangilón viaja físicamente por el cable. Si un rival destruye un poste (`bucket_post`) o intercepta el cangilón en tránsito, la carga cae al suelo como botín etiquetado con el equipo extractor original, listo para ser robado.

---

## 4. Escalado por Tamaño de Equipo (`1v1` a `8v8`)

Las dimensiones del mundo y la meta de dinero (`target_cash`) escalan de forma lineal (+`1_500 $` por jugador adicional por equipo) para mantener partidas tensas de **9 a 14 minutos**:

| Formato | `team_size` | Meta de Dinero (`target_cash`) | Umbral de Fama (`60 %`) | Dimensiones del Mapa (`X × Z × Y`) |
|---|---:|---:|---:|---|
| `1v1` | `1` | `2_000 $` | `1_200 $` | `64 × 64 × 64` |
| `2v2` | `2` | `3_500 $` | `2_100 $` | `64 × 64 × 64` |
| `3v3` | `3` | `5_000 $` | `3_000 $` | `64 × 64 × 64` |
| `4v4` | `4` | `6_500 $` | `3_900 $` | `64 × 64 × 64` |
| `5v5` | `5` | `8_000 $` | `4_800 $` | `64 × 64 × 64` |
| `6v6` | `6` | `9_500 $` | `5_700 $` | `64 × 64 × 64` |
| `7v7` | `7` | `11_000 $` | `6_600 $` | `96 × 96 × 64` |
| `8v8` | `8` | `12_500 $` | `7_500 $` | `96 × 96 × 64` |

---

## 5. Mecánicas de Riesgo, Remontada y Victoria

### 5.1 Muerte, Bolsa de Botín y Reaparición

- **Fase de Bajada inicial (`drop_in_phase_ticks`):** los primeros `600` ticks (`20` s) corresponden al descenso desde el vagón aéreo hacia la superficie.
- **Bolsa al morir (`loot_bag_drop_num: 1, loot_bag_drop_den: 1`):** cuando un jugador muere (por caída, lava, bloque eléctrico o explosión), suelta en el lugar una bolsa física con el **100 %** de la mena, oro y diamantes que transportaba:
  $$\text{dropped\_amount} = \frac{\text{carried\_amount} \times \text{loot\_bag\_drop\_num}}{\text{loot\_bag\_drop\_den}}$$
- **Expiración de la bolsa (`loot_bag_lifetime_ticks`):** la bolsa permanece en el mundo durante `1_800` ticks (`60` s). Si nadie la recoge antes de ese límite, desaparece.
- **Reaparición (`respawn_ticks`):** el jugador reaparece en la base de su equipo tras `180` ticks (`6,0` s).

### 5.2 Fama al 60 % (`Fame`)

Cuando el banco de un equipo alcanza o supera el **60 %** de la meta de la partida, dicho equipo entra en estado de **Fama**:

$$\text{cash} \times \text{fame\_threshold\_den} \ge \text{target\_cash} \times \text{fame\_threshold\_num} \iff \text{cash} \times 100 \ge \text{target\_cash} \times 60$$

- La sirena de la mina anuncia públicamente que el equipo líder está en **Fama**.
- Las estaciones de cangilones y los portadores de gran botín del equipo en Fama emiten un pulso acústico más intenso detectable con el geófono, facilitando las incursiones del equipo en desventaja.

### 5.3 Multiplicador de Botín Robado (`×5 / 4`)

Todo lingote de oro o gema de diamante conserva el identificador del equipo que lo picó de la roca original (`origin_team`). Si el equipo contrario captura una bolsa de botín o intercepta un cangilón enemigo y lo deposita en su propio banco, recibe una bonificación del **25 %** (`5 / 4`):

$$\text{stolen\_cash\_value} = \frac{\text{base\_value} \times \text{stolen\_loot\_multiplier\_num}}{\text{stolen\_loot\_multiplier\_den}} = \frac{\text{base\_value} \times 5}{4}$$

- **Oro robado (`base_value = 100`):** $(100 \times 5) / 4 = 125\ \$$ por unidad.
- **Diamante robado por gema (`base_value = 200`):** $(200 \times 5) / 4 = 250\ \$$ por unidad (`500 $` por bloque completo de `2` gemas).

### 5.4 Condiciones de Victoria

1. **Victoria inmediata por Meta Económica:** en el mismo tick en que el saldo de un equipo cumple $\text{cash} \ge \text{target\_cash}$, la partida concluye inmediatamente con la victoria de ese equipo (si ambos equipos alcanzan o superan la meta en el mismo tick exacto, gana el que tenga mayor $\text{cash}$ total, o se declara empate si sus saldos son idénticos).
2. **Victoria al cierre del Despertar:** si ningún equipo alcanza $\text{target\_cash}$ antes de que concluya la fase final de evacuación y colapso del **Despertar**, gana el equipo con mayor saldo acumulado en su banco ($\text{cash}_A > \text{cash}_B$). Si ambos equipos terminan con exactamente el mismo dinero ($\text{cash}_A = \text{cash}_B$), **el empate se mantiene** como resultado legítimo de la simulación determinista.
