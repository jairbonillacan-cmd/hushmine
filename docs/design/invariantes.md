# Especificación Formal de Invariantes de Simulación (`hush-sim`)

Este documento define las doce invariantes formales de **HUSHMINE** que gobiernan la simulación determinista en `hush-sim` y la frontera de red del servidor autoritativo. Cada invariante se expresa como una especificación matemática sin código de implementación, diseñada para trasladarse directamente a baterías de pruebas basadas en propiedades con `proptest` y al verificador en vivo de `hush-cli sim` (que registra cualquier violación en la lista `broken_invariants` del informe).

---

## Convenciones Globales e Inmutables

1. **Reloj discreto:** frecuencia fija de $30\text{ ticks/s}$ ($1\,800\text{ ticks/min}$; $28\,800\text{ ticks} = 16\text{ min}$).
2. **Aritmética espacial:** punto fijo `Fixed` representado como entero con signo `i32` en formato `Q16.16` ($1\text{ bloque} = 65\,536\text{ subbloques}$). Prohibición absoluta de `f32` y `f64` en el estado de simulación.
3. **Aritmética contable y sísmica:** dinero, mena, unidades de oro/diamante y el medidor del Sismógrafo son enteros sin signo `u32`. Los multiplicadores se aplican mediante fracciones enteras con división euclídea truncada.
4. **Catálogo de bloques:** identificadores enteros en el rango cerrado `0..=17` (`0 air` hasta `17 bucket_post`), definidos en `data/blocks.ron`.
5. **Sismógrafo:** entero `u32` acotado en $[0, 100\,000]$, con umbrales en $25\,000$ (`Stir` / Revuelo), $50\,000$ (`Breathe` / Respira), $75\,000$ (`Restless` / Inquietud) y $100\,000$ (`Wake` / Despertar), definidos en `data/seismograph.ron`.

---

## Dominio 1 · Economía, Recursos y Aritmética Financiera

### INV-01 · Conservación contable del valor de oro y diamante

- **Identificador:** `INV-01` (`economy::bullion_value_conservation`)
- **Entrada generada (estrategia de `proptest`):**
  - Semilla de mundo `u64` arbitraria y tamaño de equipo en $1..=8$ (que selecciona `target_cash` y dimensiones de mapa en `data/economy.ron`).
  - Secuencia de longitud $N \in [1, 5\,000]$ de acciones válidas e inválidas intercaladas: picar bloques de oro (`id = 5`) y diamante (`id = 6`), recoger y soltar bolsas al morir, cargar líneas de cangilones (`id = 16`, `id = 17`), robar recursos del equipo rival, depositar en el banco del equipo o estación de cangilones (`id = 16`), realizar ofrendas en el altar y cruzar los umbrales sísmicos de $50\,000$ y $100\,000$.
- **Propiedad matemática / invariante exacta:**
  En cualquier tick $t \ge 0$, el balance contable cerrado del valor monetario cumple:
  $$V_{\text{mapa}}(t) + V_{\text{inv}}(t) + V_{\text{bolsas}}(t) + V_{\text{cangilones}}(t) + V_{\text{bancos}}(t) + V_{\text{ofrendas}}(t) - B_{\text{robo}}(t) - I_{\text{umbrales}}(t) = V_{\text{inicial}}(\text{seed})$$
  donde:
  - $V_{\text{mapa}}(t)$: suma del valor de todos los bloques `gold` ($1 \times 100 = 100$ por bloque) y `diamond` ($2 \times 200 = 400$ por bloque completo, `diamond_cash_value = 400`) aún presentes en la cuadrícula vóxel.
  - $V_{\text{inv}}(t)$: valor monetario base de las unidades de oro y diamante portadas por todos los jugadores vivos.
  - $V_{\text{bolsas}}(t)$: valor monetario base contenido en las bolsas de botín en el suelo (más el registro contable de bolsas destruidas por lava o expiradas tras `loot_bag_lifetime_ticks = 1_800`).
  - $V_{\text{cangilones}}(t)$: valor monetario base en tránsito dentro de todas las estaciones y postes de cangilones.
  - $V_{\text{bancos}}(t)$: suma de los saldos depositados en los bancos de ambos equipos (`Copper` y `Cobalt`).
  - $V_{\text{ofrendas}}(t)$: oro acumulado consumido en los altares (`offering_gold_cost = 100` por cada ofrenda ejecutada).
  - $B_{\text{robo}}(t) = \sum_{k} \left(\lfloor (v_k \times 5) / 4 \rfloor - v_k\right)$: suma acumulada de las primas enteras generadas al depositar botín robado de valor base $v_k$.
  - $I_{\text{umbrales}}(t)$: valor total inyectado en el mapa por eventos sísmicos ya activados en $[0, t]$ (`mother_lode_total_cash = 2_000` al emitir `Breathe` en $50\,000$ y `heart_total_cash = 1_500` al emitir `Wake` en $100\,000$).
  - $V_{\text{inicial}}(\text{seed})$: valor monetario de los bloques de oro y diamante generados en $t = 0$.
- **Condiciones de borde:**
  - Muerte simultánea de un jugador en el mismo tick en que deposita en el banco o en que una explosión destruye una estación de cangilones con carga en tránsito.
  - Expiración de una bolsa exactamente en el tick $t_0 + 1\,800$ o inmersión de una bolsa en lava (`id = 7`).
  - Generación de la Veta Madre o del Corazón reemplazando vóxeles preexistentes de mineral (el generador de eventos debe descontar o preservar el mineral desplazado en el término $I_{\text{umbrales}}$).
- **Por qué importa en producción:**
  Impide errores de duplicación de dinero (*duping*) y pérdidas silenciosas de botín en transferencias entre jugador, bolsa, línea de cangilones y banco, garantizando que la victoria económica sea justa y auditable.

---

### INV-08 · No negatividad y conservación de la mena

- **Identificador:** `INV-08` (`economy::ore_non_negative_and_build_cost`)
- **Entrada generada (estrategia de `proptest`):**
  - Jugador con clase aleatoria $C \in \{\text{Prospector}, \text{Miner}, \text{Engineer}, \text{Sapper}\}$, saldo inicial de mena $\text{player\_ore} \in [0, 50]$ y secuencia de órdenes de construcción para cualquier identificador de bloque $b \in [0, 25]$ en coordenadas válidas e inválidas.
- **Propiedad matemática / invariante exacta:**
  1. En todo tick $t$, los saldos de mena, oro, diamante y dinero de cada jugador y equipo son enteros sin signo `u32` ($\ge 0$), y el inventario portátil de mena cumple $0 \le \text{player\_ore}(t) \le \text{max\_ore\_capacity}(C)$ (`20`, `40`, `30` o `20` según `data/classes.ron`).
  2. Sea $b \in \{8..=17\}$ un bloque construible con coste $\text{ore\_cost}(b) > 0$ definido en `data/blocks.ron`. Si la clase $C$ incluye $b$ en su lista `buildable_blocks`, la celda destino está libre y $\text{player\_ore}(t) \ge \text{ore\_cost}(b)$, la construcción tiene éxito y:
     $$\text{player\_ore}(t + 1) = \text{player\_ore}(t) - \text{ore\_cost}(b)$$
  3. Si $\text{player\_ore}(t) < \text{ore\_cost}(b)$, o $b \notin \text{buildable\_blocks}(C)$, o $b \in \{0..=7\}$ (`ore_cost = 0`, bloques naturales), la operación devuelve `Result::Err` (nunca `panic!`), el bloque no se coloca y $\text{player\_ore}(t + 1) = \text{player\_ore}(t)$.
- **Condiciones de borde:**
  - Saldo exacto igual al coste: $\text{player\_ore}(t) = \text{ore\_cost}(b)$, dejando el saldo en $0$ sin subdesbordamiento.
  - Intento de construir con $\text{player\_ore}(t) = 0$ o intento de construir la estación de cangilones (`id = 16`, `ore_cost = 12`) con `11` de mena.
  - Recogida de un bloque de mena (`ore_units_per_block = 4`) cuando al jugador solo le quedan $1$, $2$ o $3$ unidades libres hasta `max_ore_capacity(C)`.
- **Por qué importa en producción:**
  Un cliente modificado o con predicción desincronizada puede enviar paquetes de construcción sin saldo o de bloques prohibidos para su clase; el servidor debe rechazarlos limpiamente sin caer por resta con subdesbordamiento ni regalar estructuras gratuitas.

---

### INV-12 · Aritmética entera del botín robado y de la Fama

- **Identificador:** `INV-12` (`economy::integer_stolen_loot_and_fame`)
- **Entrada generada (estrategia de `proptest`):**
  - Pares de valores monetarios base $v_1, v_2 \in [0, 100\,000]$, saldos bancarios $\text{banco}_1, \text{banco}_2 \in [0, 50\,000]$ y metas de partida $\text{meta} \in \{2\,000, 3\,500, 5\,000, 6\,500, 8\,000, 9\,500, 11\,000, 12\,500\}$ tomadas de `team_goals` en `data/economy.ron`.
- **Propiedad matemática / invariante exacta:**
  1. **Multiplicador de botín robado ($\times 5 / 4$):** para todo valor base $v \in \mathbb{N}$ procedente del equipo rival, el importe acreditado al depositarlo es:
     $$R(v) = \left\lfloor \frac{v \times 5}{4} \right\rfloor$$
     cumpliendo exactitud entera, ausencia de pérdida respecto al valor base ($R(v) \ge v$) y monotonía: $\forall v_1 \le v_2 \implies R(v_1) \le R(v_2)$.
  2. **Activación de Fama ($60 / 100$):** el estado de Fama de un equipo con saldo $\text{banco}$ y objetivo $\text{meta}$ viene dado exclusivamente por la desigualdad entera:
     $$\text{Fama}(\text{banco}, \text{meta}) \iff \text{banco} \times 100 \ge \text{meta} \times 60$$
     cumpliendo monotonía respecto al saldo depositado: si $\text{Fama}(\text{banco}_1, \text{meta})$ es verdadero y $\text{banco}_2 \ge \text{banco}_1$, entonces $\text{Fama}(\text{banco}_2, \text{meta})$ es verdadero.
- **Condiciones de borde:**
  - Valores $v$ no divisibles por $4$ (por ejemplo, si en el futuro se introducen fracciones de lote) y valores canónicos ($v = 100 \implies R(100) = 125$ por lingote de oro; $v = 200 \implies R(200) = 250$ por gema de diamante; $v = 400 \implies R(400) = 500$ por bloque completo de diamante).
  - Umbral exacto de Fama y una unidad por debajo: para $\text{meta} = 2\,000$, en $\text{banco} = 1\,199$ se tiene $119\,900 < 120\,000$ (sin Fama) y en $\text{banco} = 1\,200$ se tiene $120\,000 \ge 120\,000$ (con Fama).
  - Ausencia de desbordamiento en `u32` al multiplicar $\text{banco} \times 100$ y $v \times 5$ para saldos altos.
- **Por qué importa en producción:**
  Evita discrepancias de redondeo entre plataformas o entre el HUD del cliente y la regla del servidor en el instante exacto en que se enciende la baliza de Fama o se alcanza la meta de victoria con un último lingote robado.

---

## Dominio 2 · Determinismo y Reloj de Simulación

### INV-02 · Determinismo estricto multiplataforma

- **Identificador:** `INV-02` (`sim::cross_platform_determinism`)
- **Entrada generada (estrategia de `proptest`):**
  - Semilla `seed: u64` arbitraria en todo el espacio de $64\text{ bits}$.
  - Vector de comandos de entrada `(tick, player_id, command)` generado en orden arbitrario y posteriormente normalizado por clave canónica `(tick, player_id, seq)`, cubriendo movimiento en `Fixed` (`Q16.16`), salto, uso de pico, excavación, colocación de bloques, activación de explosivos y ofrendas a lo largo de $T \in [1, 3\,000]$ ticks.
- **Propiedad matemática / invariante exacta:**
  Dadas dos instancias independientes de la simulación $S_1$ y $S_2$ inicializadas con la misma semilla `seed` y alimentadas con la misma secuencia ordenada de entradas $\Sigma$:
  $$\forall t \in [0, T], \quad \text{state\_hash}(S_1, t) = \text{state\_hash}(S_2, t)$$
  donde `state_hash` es la huella criptográfica o de dispersión no conmutativa (`u64` FNV-1a / determinista) calculada sobre la totalidad del estado autoritativo: cuadrícula de bloques y metadatos, máscaras de niebla de mineral, posiciones y velocidades `Fixed` (`i32`) de jugadores, inventarios, bolsas, líneas de cangilones, celdas de lava, estado del Sismógrafo, estado interno de `hush_sim::Rng` y bancos. Ningún subsistema puede almacenar ni operar con `f32`/`f64` ni iterar sobre un `HashMap` o `HashSet` con orden dependiente de semilla aleatoria de proceso.
- **Condiciones de borde:**
  - Dos jugadores de equipos opuestos intentando picar, recoger una bolsa o colocar un bloque en la misma celda $(x, y, z)$ exactamente en el mismo tick $t$ (desempate determinista por orden canónico de `player_id`).
  - Propagación simultánea de lava (`id = 7`) y detonación en cadena de varios bloques explosivos (`id = 15`).
  - Ejecución cruzada entre arquitecturas nativas (`x86_64`, `aarch64`) y destino WebAssembly (`wasm32-unknown-unknown`).
- **Por qué importa en producción:**
  Es el pilar de las repeticiones compactas (`.zrep`), de la predicción y reconciliación en red, y del bucle de verificación automatizada (`hush-cli sim` y `hush-cli replay`). Una sola divergencia de un bit arruina la reproducción de fallos y desincroniza a los clientes.

---

### INV-07 · Duración máxima acotada de cualquier Turno

- **Identificador:** `INV-07` (`sim::bounded_shift_duration`)
- **Entrada generada (estrategia de `proptest`):**
  - Partidas simuladas donde ningún equipo deposita en el banco hasta alcanzar `target_cash`, combinadas con secuencias adversariales que intentan maximizar el tiempo de partida: ofrendas continuas cada vez que vence `offering_cooldown_ticks = 150` y entrega del máximo de fichas de latón permitidas (`max_tallies_per_shift = 2`).
- **Propiedad matemática / invariante exacta:**
  Sea $T_{\text{end}}$ el tick en que la simulación marca el Turno como finalizado (`is_finished() == true`). Bajo cualquier secuencia de entradas y semillas:
  $$0 < T_{\text{end}} \le 28\,800 + 2\,700 = 31\,500\text{ ticks}\quad (17\text{ min } 30\text{ s a } 30\text{ Hz})$$
  El límite superior duro del reloj de partida garantiza que la fase previa al Despertar no supere los $28\,800\text{ ticks}$ efectivos de tope y que la fase de Despertar (`Wake`) dure como máximo `duration_ticks = 2_700` ($90\text{ s}$).
- **Condiciones de borde:**
  - Partida completamente pasiva sin acciones de jugadores: entra en `Wake` exactamente en $t = 28\,800$ y concluye exactamente en $t = 31\,500$.
  - Victoria instantánea por alcanzar `target_cash` en el tick $31\,499$ frente a agotamiento del reloj en el tick $31\,500$ con empate de bancos.
- **Por qué importa en producción:**
  Protege a los servidores dedicados frente a partidas zombi o bloqueos infinitos (*stalls*) que agotarían la memoria y las ranuras de alojamiento si dos equipos pactaran no terminar nunca.

---

## Dominio 3 · Sismógrafo, Umbrales y el Durmiente

### INV-03 · Acotación del Sismógrafo en `[0, 100_000]`

- **Identificador:** `INV-03` (`seismograph::meter_bounds_saturation`)
- **Entrada generada (estrategia de `proptest`):**
  - Secuencias extremas por tick de fuentes de ruido definidas en `data/seismograph.ron`: ráfagas de explosiones (`+3_000`), descargas eléctricas (`+200`), avances de lava (`+200`) y múltiples líneas de cangilones (`+1` por tick), intercaladas con reducciones masivas por ofrendas (`-2_500`) y entregas de fichas de latón (`-8_000`) cuando el medidor está cerca de $0$ o cerca de $100\,000$.
- **Propiedad matemática / invariante exacta:**
  En todo tick $t \ge 0$, el valor del Sismógrafo $\text{meter}(t) \in \mathbb{N}$ satisface:
  $$0 \le \text{meter}(t) \le 100\,000$$
  Toda actualización en un tick se calcula con aritmética entera saturada:
  $$\text{meter}(t) = \min\Big(100\,000,\ \max\big(0,\ \text{meter}(t-1) + \Delta_{\text{pasivo}}(t) + \Delta_{\text{activo}}(t) - \Delta_{\text{alivio}}(t)\big)\Big)$$
  sin que jamás ocurra desbordamiento (`overflow`) ni subdesbordamiento (`underflow`) en el tipo `u32`.
- **Condiciones de borde:**
  - Entrega de una ficha de latón (`-8_000`) o una ofrenda (`-2_500`) en los primeros ticks de la partida cuando $\text{meter}(t-1) < 2\,500$: el resultado se clava en $0$.
  - Detonación simultánea de $40$ bloques explosivos en un solo tick ($\Delta_{\text{activo}} = +120\,000$): el resultado se clava exactamente en $100\,000$.
- **Por qué importa en producción:**
  Un subdesbordamiento en un entero sin signo `u32` al aplicar una ofrenda temprana envolvería el medidor cerca de $2^{32} - 1$, disparando instantáneamente todos los umbrales y destruyendo la partida en el primer minuto.

---

### INV-04 · Exactitud del crecimiento pasivo en el tick `28_800`

- **Identificador:** `INV-04` (`seismograph::exact_passive_growth_28800`)
- **Entrada generada (estrategia de `proptest`):**
  - Partidas sin eventos de ruido activo ($\Delta_{\text{activo}}(t) = 0$) ni reducciones por ofrendas o fichas ($\Delta_{\text{alivio}}(t) = 0$), muestreadas en cualquier tick arbitrario $t \in [0, 28\,800]$.
- **Propiedad matemática / invariante exacta:**
  El crecimiento pasivo reparte `total_value = 100_000` en `total_ticks = 28_800` mediante un acumulador de resto entero (algoritmo tipo Bresenham: en cada tick se suma $100\,000$ al acumulador de resto, se incrementa el medidor en $\lfloor \text{acc} / 28\,800 \rfloor$ y se conserva $\text{acc} \bmod 28\,800$). En consecuencia, en ausencia de eventos externos:
  $$\forall t \in [0, 28\,800], \quad \text{meter}(t) = \left\lfloor \frac{100\,000 \times t}{28\,800} \right\rfloor = \left\lfloor \frac{125 \times t}{36} \right\rfloor$$
  Por tanto:
  1. $\forall t < 28\,800, \quad \text{meter}(t) < 100\,000$.
  2. En el penúltimo tick: $\text{meter}(28\,799) = \lfloor 2\,879\,900\,000 / 28\,800 \rfloor = 99\,996$.
  3. En el tick exacto de los $16\text{ min}$: $\text{meter}(28\,800) = 100\,000$ (con resto acumulado igual a $0$).
- **Condiciones de borde:**
  - Ticks exactos de cruce pasivo de cada umbral:
    - `Stir` ($25\,000$) en $t = 7\,200$ (mientras que en $t = 7\,199$ vale $24\,996$).
    - `Breathe` ($50\,000$) en $t = 14\,400$ (en $t = 14\,399$ vale $49\,996$).
    - `Restless` ($75\,000$) en $t = 21\,600$ (en $t = 21\,599$ vale $74\,996$).
    - `Wake` ($100\,000$) en $t = 28\,800$ (en $t = 28\,799$ vale $99\,996$).
- **Por qué importa en producción:**
  Si se aproxima el incremento por tick a un entero truncado fijo ($\lfloor 100\,000 / 28\,800 \rfloor = 3$), en el tick $28\,800$ el medidor apenas llegaría a $86\,400$, retrasando el Despertar más de dos minutos y medio respecto al diseño canónico.

---

### INV-05 · Emisión única y monótona de umbrales sísmicos

- **Identificador:** `INV-05` (`seismograph::unique_monotonic_thresholds`)
- **Entrada generada (estrategia de `proptest`):**
  - Secuencias oscilantes de incrementos de ruido y reducciones (ofrendas y fichas de latón) que hacen que $\text{meter}(t)$ cruce hacia arriba un umbral $U_k$, descienda por debajo de $U_k$ y vuelva a superarlo repetidas veces, así como saltos bruscos que atraviesan dos o más umbrales en un mismo tick.
- **Propiedad matemática / invariante exacta:**
  Sea la tupla ordenada de umbrales $\mathcal{U} = (U_1, U_2, U_3, U_4) = (25\,000, 50\,000, 75\,000, 100\,000)$ y sea $E = (e_1, e_2, \dots, e_m)$ la secuencia total de eventos de umbral emitidos durante toda la partida, donde cada $e_r = (t_r, U_{k_r})$:
  1. **Unicidad (idempotencia por umbral):** $m \le 4$ y todos los umbrales emitidos son distintos ($\forall r \ne s,\ k_r \ne k_s$).
  2. **Monotonía estricta de orden:** para todo índice $r \in [1, m]$, se cumple $k_r = r$ (es decir, la secuencia emitida es siempre un prefijo de $(U_1, U_2, U_3, U_4)$) y los ticks de emisión cumplen $t_1 \le t_2 \le \dots \le t_m$.
  3. **Completitud al cruzar:** si en el tick $t$ se cumple $\text{meter}(t) \ge U_k$, entonces $U_k$ ha sido emitido exactamente una vez en algún tick $t_k \le t$.
- **Condiciones de borde:**
  - El medidor sube a $50\,100$ (emitiendo `Breathe`), una ficha de latón lo baja a $42\,100$ y una explosión lo sube a $45\,100$ y luego a $51\,000$: no se vuelve a generar una segunda Veta Madre ni se reemite `Breathe`.
  - Un salto extremo desde $24\,000$ hasta $76\,000$ en un único tick emite en ese mismo tick exactamente `[Stir, Breathe, Restless]` en ese orden estricto.
- **Por qué importa en producción:**
  Reemitir `Breathe` duplicaría la Veta Madre e inyectaría $+2\,000$ extra de oro rompiendo `INV-01`; emitir umbrales desordenados corrompería las máquinas de estados del mapa, la crónica de partida y las locuciones del Vigía.

---

### INV-06 · Inmutabilidad del Sismógrafo durante el Despertar y parada de líneas

- **Identificador:** `INV-06` (`seismograph::wake_immutability_and_bucket_halt`)
- **Entrada generada (estrategia de `proptest`):**
  - Partidas que alcanzan `Wake` ($\text{meter}(t_{\text{wake}}) = 100\,000$) con múltiples líneas de cangilones activas cargadas de mineral, seguidas durante los `2_700` ticks del Despertar por intentos de ofrendas en el altar, entregas de fichas de latón, explosiones y operaciones logísticas.
- **Propiedad matemática / invariante exacta:**
  Sea $t_{\text{wake}}$ el único tick en el que se emite el umbral `Wake`. Para todo tick $t \in [t_{\text{wake}}, t_{\text{wake}} + 2\,700]$:
  1. **Clavado del medidor:** $\text{meter}(t) = 100\,000$.
  2. **Bloqueo de alivio:** toda orden de ofrenda de oro o entrega de ficha de latón en $t \ge t_{\text{wake}}$ devuelve `Result::Err` sin consumir oro ni fichas.
  3. **Parada logística en los últimos 30 segundos (`bucket_stop_remaining_ticks = 900`):** sea $t_{\text{rem}} = (t_{\text{wake}} + 2\,700) - t$ el tiempo restante del Despertar. Para todo $t$ tal que $0 \le t_{\text{rem}} \le 900$ (es decir, $t \in [t_{\text{wake}} + 1\,800, t_{\text{wake}} + 2\,700]$):
     - La cantidad de recursos transferidos entre postes y estaciones de cangilones es exactamente $0$.
     - El ruido aportado por las líneas de cangilones es exactamente $0$.
- **Condiciones de borde:**
  - Transición exacta entre $t_{\text{rem}} = 901$ (último tick en que una línea con temporizador `transfer_interval_ticks = 60` cumplido aún puede mover un cangilón) y $t_{\text{rem}} = 900$ (parada total inmediata).
  - Los recursos detenidos dentro de los cangilones en $t_{\text{rem}} \le 900$ no desaparecen: permanecen contabilizados en $V_{\text{cangilones}}(t)$ (o caen en bolsa si el poste es destruido), preservando `INV-01`.
- **Por qué importa en producción:**
  Garantiza que el Despertar sea irreversible (nadie puede comprar tiempo una vez despierto el Durmiente) y que los últimos $30\text{ s}$ obliguen a los jugadores a transportar el botín del Corazón a pie bajo la subida de lava, tal como exige el diseño del clímax.

---

## Dominio 4 · Integridad del Mundo Vóxel y Ciclo de Vida del Jugador

### INV-09 · Integridad de los ids de bloque y fronteras del mapa

- **Identificador:** `INV-09` (`voxel::valid_block_ids_and_bedrock_borders`)
- **Entrada generada (estrategia de `proptest`):**
  - Coordenadas tridimensionales arbitrarias $(x, y, z) \in [-128, 256]^3$, identificadores de bloque candidatos $b \in [0, 255]$, y secuencias de excavación, colocación, detonación de explosivos (`id = 15`), derrumbes sísmicos (`Stir` / `Restless`) y subida de lava (`Wake`).
- **Propiedad matemática / invariante exacta:**
  Sea el dominio espacial del mapa $\Omega = [0, W-1] \times [0, H-1] \times [0, D-1]$ con $(W, D, H) \in \{(64, 64, 64), (96, 96, 64)\}$ según `data/economy.ron`:
  1. **Validez de catálogo:** $\forall (x, y, z) \in \Omega,\ \text{block\_id}(x, y, z) \in [0, 17]$.
  2. **Indestructibilidad de la frontera:** toda celda en la envolvente de contención del mapa (lecho inferior $y = 0$ y muros perimetrales $x \in \{0, W-1\}$ o $z \in \{0, D-1\}$) es indestructible; ninguna acción de jugador, explosión ni evento sísmico puede transformarla en `air` (`id = 0`) ni permitir que un jugador salga de $\Omega$.
  3. **Seguridad fuera de rango:** toda operación que apunte a $(x, y, z) \notin \Omega$ o intente escribir un identificador $b > 17$ devuelve `Result::Err` sin hacer `panic!` y deja el estado del mundo intacto.
- **Condiciones de borde:**
  - Explosión detonada en $(1, 1, 1)$ adyacente a las tres caras de frontera $(0, y, z)$, $(x, 0, z)$ y $(x, y, 0)$: la onda expansiva destruye los bloques internos destructibles pero respeta la frontera y recorta los rayos que saldrían a coordenadas negativas.
  - Jugador empujado por un pico (`pickaxe_push_subblocks = 32_768`) o una plataforma de salto (`id = 11`) contra el techo $y = H - 1$ o las paredes laterales.
- **Por qué importa en producción:**
  Evita pánicos por indexación fuera de límites en los vectores de chunks, fugas de lava fuera de la memoria del mapa y caídas de jugadores al vacío infinito.

---

### INV-11 · Bolsas al morir y reaparición en `180` ticks

- **Identificador:** `INV-11` (`player::death_loot_bag_and_respawn_180`)
- **Entrada generada (estrategia de `proptest`):**
  - Jugador vivo en el tick $t_{\text{death}}$ con inventario arbitrario $(o, g, d)$ donde $o \in [0, 40]$, $g + d \in [0, 20]$, sometido a una causa letal (contacto con lava `id = 7`, descarga de bloque eléctrico `id = 12` enemigo, radio de explosivo `id = 15`, caída letal o derrumbe), seguido de comandos de excavación, construcción y movimiento durante los ticks $[t_{\text{death}}, t_{\text{death}} + 250]$.
- **Propiedad matemática / invariante exacta:**
  Si un jugador $p$ muere en el tick $t_{\text{death}}$ con inventario previo $(o_0, g_0, d_0)$:
  1. **Vaciado y caída íntegra (`loot_bag_drop_num = 1`, `loot_bag_drop_den = 1`):** en el propio tick $t_{\text{death}}$, el inventario de $p$ pasa a $(0, 0, 0)$. Si $o_0 + g_0 + d_0 > 0$, se crea en el mundo exactamente una bolsa de botín con contenido $(o_0, g_0, d_0)$ y tiempo de vida `loot_bag_lifetime_ticks = 1_800`. Si $o_0 + g_0 + d_0 = 0$, no se crea ninguna bolsa vacía.
  2. **Inacción durante la muerte:** para todo tick $t \in [t_{\text{death}}, t_{\text{death}} + 179]$, el estado del jugador es `Dead` y cualquier comando de movimiento, excavación, construcción, recogida de bolsas u ofrenda emitido por $p$ devuelve `Result::Err` sin alterar el mundo.
  3. **Reaparición exacta (`respawn_ticks = 180`):** el jugador vuelve al estado `Alive` en su punto de aparición de equipo exactamente en el tick $t_{\text{respawn}} = t_{\text{death}} + 180$ ($6\text{ s}$ a $30\text{ Hz}$).
- **Condiciones de borde:**
  - Jugador que muere con el inventario completamente vacío $(0, 0, 0)$: no genera entidad de bolsa parásita.
  - Comando enviado por el jugador en $t_{\text{death}} + 179$ (rechazado) frente a comando enviado en $t_{\text{death}} + 180$ (aceptado).
  - Dos jugadores que mueren en la misma celda y en el mismo tick: ambos inventarios se conservan en bolsas sin sobrescribirse entre sí.
- **Por qué importa en producción:**
  Garantiza que no existan «fantasmas» capaces de seguir picando o construyendo mientras están muertos, mantiene la penalización táctica de $6\text{ s}$ y asegura que la muerte nunca destruya ni duplique recursos (`INV-01`).

---

## Dominio 5 · Seguridad de Red y Aislamiento de Información

### INV-10 · Niebla de mineral por equipo (anti-trampas)

- **Identificador:** `INV-10` (`net::team_mineral_fog_of_war`)
- **Entrada generada (estrategia de `proptest`):**
  - Mapa generado con semilla arbitraria, máscara de bits de descubrimiento por equipo $\text{known}_T(x, y, z) \in \{0, 1\}$ para $T \in \{\text{Copper}, \text{Cobalt}\}$, y secuencias de excavación, escaneo con detector de minerales del Prospector y explosiones que descubren caras adyacentes o el radio del detector.
- **Propiedad matemática / invariante exacta:**
  Sea $\mathcal{M} = \{4\ (\text{ore}),\ 5\ (\text{gold}),\ 6\ (\text{diamond})\}$ el conjunto de bloques de mineral ocultables, y sea $\Pi_T(t)$ cualquier carga útil de red (instantánea inicial del mapa completo o paquete de delta de terreno) construida en el tick $t$ para un cliente perteneciente al equipo $T$:
  $$\forall (x, y, z) \in \Omega, \quad \text{block\_id}_{\text{server}}(x, y, z) \in \mathcal{M} \land \text{known}_T(x, y, z, t) = 0 \implies \text{block\_id}_{\Pi_T}(x, y, z, t) = 2\ (\text{rock})$$
  El identificador real de un vóxel en $\mathcal{M}$ solo puede serializarse hacia el equipo $T$ a partir del tick en que $\text{known}_T(x, y, z, t) = 1$ (porque al menos una de sus $6$ caras ortogonales ha quedado expuesta a un bloque no opaco o porque ha entrado en el alcance válido del `mineral_detector` de un Prospector del equipo $T$).
- **Condiciones de borde:**
  - El equipo `Copper` excava un túnel en su base y descubre una veta de diamante ($\text{known}_{\text{Copper}} = 1$, $\text{known}_{\text{Cobalt}} = 0$): el delta de red enviado a `Copper` revela `6` (`diamond`), pero para `Cobalt` esa celda sigue enmascarada como `2` (`rock`) hasta que `Cobalt` la descubra o exponga el túnel.
  - Veta Madre generada en `Breathe` ($50\,000$): los bloques de oro expuestos al aire interior de la nueva caverna pasan a ser visibles al descubrirse la caverna, pero los minerales aún enterrados en la pared sin cara expuesta permanecen como `2` (`rock`).
- **Por qué importa en producción:**
  Neutraliza de raíz las trampas de visión a través de paredes (*wallhacks* y *X-ray* de paquetes de red): como el cliente jamás recibe los bytes de los minerales ocultos, ningún mod o analizador de tráfico puede revelar dónde está el oro ni el diamante.

---

## Tabla Resumen de Dependencias con Archivos de Datos (`.ron`)

| Invariante | Dominio | Archivos `.ron` de los que depende | Parámetros clave leídos de `.ron` |
|---|---|---|---|
| **INV-01** | Economía y Conservación | `data/economy.ron`, `data/seismograph.ron`, `data/blocks.ron` | `gold_units_per_block`, `gold_cash_value`, `diamond_units_per_block`, `diamond_cash_value`, `mother_lode_total_cash`, `heart_total_cash`, `stolen_loot_multiplier_num/den`, `offering_gold_cost`, ids `5`, `6`, `16`, `17` |
| **INV-02** | Determinismo | `data/blocks.ron`, `data/classes.ron`, `data/economy.ron`, `data/seismograph.ron` | Todos los parámetros enteros y en `Q16.16` (`move_speed_subblocks_per_tick`, `jump_velocity_subblocks_per_tick`, `tick_rate`) |
| **INV-03** | Sismógrafo | `data/seismograph.ron` | `min_value` (`0`), `max_value` (`100_000`), `noise_sources` (`explosion`, `electric_discharge`, `lava_flow`, `bucket_line_per_tick`), `provisional_relief` |
| **INV-04** | Sismógrafo | `data/seismograph.ron` | `passive_growth.total_value` (`100_000`), `passive_growth.total_ticks` (`28_800`), `tick_rate` (`30`) |
| **INV-05** | Sismógrafo | `data/seismograph.ron`, `data/economy.ron` | `thresholds` (`Stir: 25_000`, `Breathe: 50_000`, `Restless: 75_000`, `Wake: 100_000`), `provisional_relief` |
| **INV-06** | Sismógrafo y Logística | `data/seismograph.ron`, `data/economy.ron` | `wake.duration_ticks` (`2_700`), `wake.bucket_stop_remaining_ticks` (`900`), `bucket_line.transfer_interval_ticks` (`60`) |
| **INV-07** | Reloj de Partida | `data/seismograph.ron`, `data/economy.ron` | `passive_growth.total_ticks` (`28_800`), `wake.duration_ticks` (`2_700`), `max_tallies_per_shift` (`2`), `offering_cooldown_ticks` (`150`) |
| **INV-08** | Economía y Construcción | `data/blocks.ron`, `data/classes.ron`, `data/economy.ron` | `ore_cost` por bloque (`0..=12`), `max_ore_capacity`, `max_bullion_capacity`, `buildable_blocks`, `ore_units_per_block` (`4`) |
| **INV-09** | Mundo Vóxel | `data/blocks.ron`, `data/economy.ron` | Catálogo `id: 0..=17`, `hardness`, `team_goals` (`map_width`, `map_depth`, `map_height`: `64x64x64` y `96x96x64`) |
| **INV-10** | Seguridad de Red | `data/blocks.ron`, `data/classes.ron` | Ids `2` (`rock`), `4` (`ore`), `5` (`gold`), `6` (`diamond`); `primary_tool: "mineral_detector"` (`Prospector`) |
| **INV-11** | Ciclo de Vida Jugador | `data/economy.ron`, `data/classes.ron` | `respawn_ticks` (`180`), `loot_bag_lifetime_ticks` (`1_800`), `loot_bag_drop_num` (`1`), `loot_bag_drop_den` (`1`) |
| **INV-12** | Aritmética Financiera | `data/economy.ron` | `stolen_loot_multiplier_num` (`5`), `stolen_loot_multiplier_den` (`4`), `fame_threshold_num` (`60`), `fame_threshold_den` (`100`), `team_goals.target_cash` |
