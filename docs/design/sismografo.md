# Especificación de diseño: el Sismógrafo y el Durmiente

Este documento define las reglas deterministas, la representación numérica en punto fijo y los efectos sistémicos del **Sismógrafo** (*Seismograph*) durante un **Turno** (*Shift*) de HUSHMINE. La configuración de datos correspondiente se encuentra en `data/seismograph.ron`.

---

## 1. Propósito y representación en entero de punto fijo

El Sismógrafo mide la agitación acumulada de **el Durmiente** (*The Sleeper*) en las profundidades de la mina. Sustituye un reloj arbitrario de partida por un reloj termodinámico e industrial: avanza de forma inexorable con el tiempo, se acelera con la codicia extractiva de los equipos (explosivos, maquinaria pesada, cortocircuitos y fluidos) y marca el ritmo del Turno a través de cuatro umbrales sísmicos.

Para garantizar el determinismo absoluto en `hush-sim` sin recurrir a coma flotante (`f32`/`f64`), tanto el tiempo como la lectura del Sismógrafo se expresan exclusivamente mediante enteros sin signo (`u32`):

### Reloj de simulación (Convención B)
- **Frecuencia de simulación (`tick_rate`):** `30` ticks por segundo (`30 Hz`).
- **Equivalencias temporales:**
  - `1` segundo = `30` ticks.
  - `1` minuto = `1_800` ticks.
  - `16` minutos (duración base de un Turno sin ruido ni alivio) = `28_800` ticks.
  - `90` segundos (cuenta atrás del Despertar) = `2_700` ticks.

### Escala del medidor (Convención C)
- **Unidad:** milésimas de punto porcentual (`u32`).
- **Rango acotado:** de `0` (`0,000 %`) a `100_000` (`100,000 %`).
- **Equivalencias de lectura:**
  - `1_000` unidades = `1,0 %` del Sismógrafo.
  - `100` unidades = `0,1 %` del Sismógrafo.
  - `1` unidad = `0,001 %` del Sismógrafo.

---

## 2. Crecimiento pasivo con acumulador de resto entero

Incluso en ausencia total de actividad minera, el Durmiente se agita con el paso del tiempo hasta alcanzar el **Despertar** (*Wake*, `100_000`) exactamente al cumplirse los `16` minutos (`28_800` ticks).

Como `100_000` no es divisible de manera exacta entre `28_800` ($\frac{100\,000}{28\,800} = 3 + \frac{13\,600}{28\,800}$), la simulación emplea un **acumulador de resto entero** (`passive_accum: u64` o `u32`, inicializado en `0` al comenzar el Turno).

### Fórmula exacta por tick
En cada tick $t \in \{1, 2, \dots\}$ en el que el Sismógrafo aún no haya entrado en estado `Wake`:

```text
accum += 100_000
delta  = accum / 28_800   // división entera truncada
accum  = accum % 28_800   // resto entero conservado para el siguiente tick
meter  = min(100_000, meter + delta)
```

En cada tick individual, `delta` toma el valor `3` o `4` milésimas de punto porcentual según el estado del resto acumulado.

### Demostración de exactitud en la frontera (`t = 28_799` y `t = 28_800`)
Sin ninguna otra fuente de ruido ni de alivio, la suma acumulada de todos los incrementos `delta` desde el tick $1$ hasta el tick $t$ equivale exactamente a:

$$\text{meter}(t) = \left\lfloor \frac{t \times 100\,000}{28\,800} \right\rfloor, \qquad \text{accum}(t) = (t \times 100\,000) \bmod 28\,800$$

- **En el tick `t = 28_799`:**
  $$28\,799 \times 100\,000 = 2\,879\,900\,000 = 99\,996 \times 28\,800 + 15\,200$$
  Por tanto, al finalizar el tick `28_799`, el medidor vale **`99_996`** (`99,996 %`) y el acumulador conserva un resto de **`15_200`**.
- **En el tick `t = 28_800`:**
  - Se suma `100_000` al resto previo: `accum = 15_200 + 100_000 = 115_200`.
  - Se calcula el incremento entero: `delta = 115_200 / 28_800 = 4`.
  - Se actualiza el resto: `accum = 115_200 % 28_800 = 0`.
  - Se actualiza el medidor: `meter = 99_996 + 4 = 100_000`.

De este modo se garantiza matemáticamente que el crecimiento pasivo llega **exactamente a `100_000` en el tick `28_800`**, sin deriva de redondeo ni desfase de un solo tick.

---

## 3. Fuentes de ruido

La actividad industrial y destructiva de los mineros aporta incrementos enteros adicionales al Sismógrafo. Se dividen en fuentes continuas (evaluadas por tick) y eventos discretos (evaluados cuando suceden).

| Fuente de ruido | Identificador en `.ron` | Tipo | Incremento (`u32`) | Equivalencia porcentual / temporal |
|---|---|---|---:|---|
| **Explosión** | `explosion` | Evento discreto | `+3_000` | `+3,000 %` por detonación |
| **Descarga eléctrica** | `electric_discharge` | Evento discreto | `+200` | `+0,200 %` por cortocircuito / arco |
| **Flujo de lava** | `lava_flow` | Evento discreto | `+200` | `+0,200 %` por evento de avance de lava |
| **Línea de cangilones activa** | `bucket_line_per_tick` | Continua (por línea y tick) | `+1` | `+300` (`+0,300 %`) cada `10` s (`300` ticks) por línea |
| **Picar bloque normal** | `dig_block` | Acción manual | `0` | `0,000 %` (despreciable frente a explosivos y maquinaria) |

### Notas de diseño sobre las fuentes de ruido
- **Picar bloque normal (`dig_block: 0`):** el trabajo manual con pico no penaliza el reloj global; el jugador siempre puede excavar a mano sin temor a despertar al Durmiente antes de tiempo.
- **Línea de cangilones (`bucket_line_per_tick: 1`):** cada **línea de cangilones** (*bucket line*) que esté encendida y transportando mineral suma `+1` por tick. Cinco líneas operando simultáneamente durante un minuto (`1_800` ticks) añaden `9_000` unidades (`+9,0 %`) al Sismógrafo, creando una tensión directa entre velocidad logística y vida útil del Turno.
- **Geófono (`Geophone`):** los equipos pueden instalar geófonos en las galerías para detectar la dirección y magnitud de estas fuentes de ruido cuando el equipo rival detona cargas o activa líneas de cangilones.

---

## 4. Ofrenda y devolución de ficha de latón (Decisiones abiertas / Provisionales)

> [!IMPORTANT]
> **Decisión abierta de diseño:** Los valores numéricos de la **Ofrenda** (*Offering*) y de la devolución de la **Ficha de latón** (*Brass tally*) son **provisionales** (`provisional_relief` en `data/seismograph.ron`). Están parametrizados para su ajuste mediante telemetría de simulación en `hush-cli sim`.

Los equipos disponen de dos mecanismos rituales en el altar de su base para apaciguar temporalmente al Durmiente y alargar el Turno cuando necesitan tiempo extra para extraer una veta rica:

### 4.1. Ofrenda de oro (`offering`)
- **Coste (`offering_gold_cost`):** depositar un lote indivisible de `100` de oro sin amonedar en el altar del equipo.
- **Reducción (`offering_reduction`):** `-2_500` unidades (`-2,500 %` en el Sismógrafo).
- **Enfriamiento (`offering_cooldown_ticks`):** `150` ticks (`5` segundos) por equipo entre una ofrenda y la siguiente.
- **Dilema táctico:** el oro entregado como Ofrenda se destruye y no suma al marcador del equipo, obligando a elegir entre puntuación inmediata o comprar unos segundos adicionales de estabilidad sísmica.

### 4.2. Devolución de ficha de latón (`brass_tally`)
- **Origen:** las **fichas de latón** (*brass tallies*) son chapas de identificación de mineros de turnos antiguos enterradas en bolsillos de roca profunda o restos de derrumbes.
- **Reducción (`brass_tally_reduction`):** entregar una ficha de latón en el altar reduce el Sismógrafo en `-8_000` unidades (`-8,000 %`).
- **Límite por Turno (`max_tallies_per_shift`):** máximo `2` fichas de latón por Turno en toda la partida (hasta `-16_000` o `-16,000 %` de alivio total acumulado).

### 4.3. Reglas de saturación y bloqueo en el Despertar
1. **Saturación inferior en `0` (`saturating_sub`):** ninguna Ofrenda ni devolución de Ficha de latón puede reducir el Sismógrafo por debajo de `0`. Si el medidor está en `1_800` y se aplica una Ofrenda de `-2_500`, el valor resultante es `0` (nunca se produce *underflow*).
2. **No reactivación de umbrales pasados:** reducir el medidor por debajo de un umbral ya cruzado no revierte los cambios estructurales del mundo ni permite que ese umbral vuelva a dispararse al subir de nuevo.
3. **Bloqueo total en el Despertar (`Wake`):** una vez que el Sismógrafo alcanza `100_000` y comienza la fase de **Despertar**, el altar se cierra. No se aceptan nuevas ofrendas ni fichas de latón, y el medidor queda fijado permanentemente en `100_000` hasta el final del Turno.

---

## 5. Orden exacto de ejecución dentro de un tick

Para que la simulación sea `100 %` reproducible entre cliente, servidor y pruebas sin cabecera (`hush-cli sim`), cada tick ejecuta la actualización del Sismógrafo siguiendo estrictamente estos **seis pasos en orden**:

```text
[Paso 1: Fase Wake activa] ──(si ya está en Wake)──> Avanzar cuenta atrás, lava cada 90 ticks,
         │                                           parar cangilones en <= 900 ticks y salir.
         ▼ (si aún NO está en Wake)
[Paso 2: Crecimiento pasivo] ──> accum += 100_000; delta = accum / 28_800; accum %= 28_800;
         │                       meter += delta
         ▼
[Paso 3: Ruido continuo]     ──> meter += active_bucket_lines * 1
         ▼
[Paso 4: Ruido discreto]     ──> Sumar explosiones (+3_000), descargas (+200) y flujo de lava (+200)
         │                       ordenados por coordenada (z, y, x)
         ▼
[Paso 5: Alivio ritual]      ──> Si meter < 100_000, restar ofrendas (-2_500) y fichas (-8_000)
         │                       con saturating_sub(0)
         ▼
[Paso 6: Acotación y umbrales]─> meter = clamp(meter, 0, 100_000); disparar umbrales con triggered == false
```

### Detalle paso a paso

1. **Paso 1 — Evaluación de estado `Wake` (Despertar activo):**
   - Si el Sismógrafo ya entró en estado `Wake` en un tick anterior:
     - El medidor permanece bloqueado en `100_000`.
     - Se decrementa el contador restante: `wake_remaining_ticks = wake_remaining_ticks.saturating_sub(1)`.
     - Si `(2_700 - wake_remaining_ticks) % 90 == 0`, se ejecuta un pulso de ascenso de lava desde la cota inferior.
     - Si `wake_remaining_ticks <= 900` (últimos `30` segundos), todas las **líneas de cangilones** (*bucket lines*) quedan mecánicamente bloqueadas y dejan de transportar mineral.
     - Si `wake_remaining_ticks == 0`, el Turno concluye de inmediato.
     - **Se omiten los pasos 2 a 6.**

2. **Paso 2 — Crecimiento pasivo del tick:**
   - Si aún no está en `Wake`, se actualiza el acumulador de resto entero:
     - `accum += 100_000`
     - `delta = accum / 28_800`
     - `accum %= 28_800`
     - `meter += delta`

3. **Paso 3 — Fuentes de ruido continuas:**
   - Se cuenta el número total de líneas de cangilones activas en el mapa (`active_bucket_lines`) y se añade su contribución:
     - `meter += active_bucket_lines * 1`

4. **Paso 4 — Eventos discretos de ruido:**
   - Se recogen todos los eventos discretos generados durante el tick (detonaciones de explosivos, descargas eléctricas y avances de bloques de lava).
   - Para garantizar el determinismo, los eventos se procesan en orden lexicográfico estricto de sus coordenadas de vóxel `(z, y, x)` (donde `y` es la cota vertical de altura/profundidad, `z` el eje norte-sur y `x` el eje este-oeste) y se suman sus incrementos (`+3_000`, `+200`, `+200`).

5. **Paso 5 — Aplicación de ofrendas y fichas de latón:**
   - **Regla de prioridad del Despertar:** si tras completar el Paso 4 el valor provisional cumple `meter >= 100_000`, el Despertar es irreversible y **se ignoran** todas las ofrendas o fichas de latón entregadas en ese mismo tick.
   - En caso contrario (`meter < 100_000`), se procesan las entregas válidas por orden de identificador de equipo (`team_id` ascendente), verificando el enfriamiento de `150` ticks para las ofrendas y el tope global de `2` fichas por Turno:
     - `meter = meter.saturating_sub(total_relief)`

6. **Paso 6 — Acotación superior y disparo único de umbrales:**
   - Se acota el medidor al rango válido: `meter = meter.min(100_000)`.
   - Se recorren en orden ascendente los cuatro umbrales (`25_000`, `50_000`, `75_000`, `100_000`).
   - Cada umbral posee una bandera booleana de estado en la partida (`triggered: bool`, inicializada en `false`).
   - Si `meter >= threshold.value` y `!threshold.triggered`:
     - Se marca `threshold.triggered = true`.
     - Se emite el evento sísmico correspondiente **una única vez en toda la partida** (incluso si múltiples umbrales se cruzan en el mismo tick por una gran cadena de explosiones, se emiten en orden `Stir` -> `Breathe` -> `Restless` -> `Wake`).
     - Si el umbral disparado es `Wake` (`100_000`), se inicializa `wake_remaining_ticks = 2_700` para que comience a descontarse a partir del Paso 1 del tick siguiente.

---

## 6. Efectos de cada umbral en el mundo y avisos diegéticos

Cada umbral transforma la topología del mapa, la información disponible para **el Vigía** (*The Watcher*) y la respuesta de los instrumentos diegéticos de la mina.

| Umbral | ID (`.ron`) | Valor (`u32`) | % | Tiempo pasivo (sin ruido) | Efecto principal en el mundo |
|---|---|---:|---:|---:|---|
| **Revuelo** | `Stir` | `25_000` | `25 %` | `7_200` ticks (`4` min) | Silbido de **los Silbadores**, temblor y microderrumbes |
| **Respira** | `Breathe` | `50_000` | `50 %` | `14_400` ticks (`8` min) | Apertura de la **Veta Madre** y vector sísmico en el Vigía |
| **Inquietud** | `Restless` | `75_000` | `75 %` | `21_600` ticks (`12` min) | Rotura de bolsas de lava colgadas y colapso/apertura de galerías |
| **Despertar** | `Wake` | `100_000` | `100 %` | `28_800` ticks (`16` min) | Cuenta atrás de `90` s, aparición de **el Corazón**, ascenso de lava y parada final de cangilones |

### 6.1. `25_000` — Revuelo (`Stir`)
- **Aviso diegético anticipado:** pocos segundos antes de cruzar `25_000`, los conductos acústicos naturales de la roca conocidos como **los Silbadores** (*The Whistlers*) emiten un silbido grave y modulado que resuena en las galerías.
- **Respuesta física:** sacudida de cámara en todos los jugadores subterráneos, caída de polvo del techo y **microderrumbes locales** (desprendimiento de bloques sueltos de grava/escombro en techos no entibados).
- **Lectura en cabina:** la aguja del Sismógrafo en el puesto de **el Vigía** (*The Watcher*) entra en la primera franja ámbar y deja un trazo de tinta más ancho sobre el tambor de papel.

### 6.2. `50_000` — Respira (`Breathe`)
- **Transformación del mapa:** una fractura tectónica abre las paredes selladas de la caverna central donde yace la **Veta Madre** (*Mother Lode*), la mayor concentración de mineral de alto valor a media profundidad.
- **Instrumentación y HUD:** la onda de choque queda registrada en los geófonos y proporciona al Vigía y al HUD de los mineros un **vector sísmico aproximado** (rumbo y cota estimada) hacia la caverna recién abierta de la Veta Madre, forzando el encuentro entre ambos equipos.

### 6.3. `75_000` — Inquietud (`Restless`)
- **Peligro ambiental:** la presión térmica agrieta los estratos intermedios, provocando la **rotura de bolsas de lava colgadas** sobre las galerías de tránsito y generando nuevos flujos de lava (que a su vez aportan `+200` por evento de avance al Sismógrafo).
- **Reconfiguración de rutas:** se producen **colapsos de túneles antiguos y aperturas de grietas laterales**, cortando caminos seguros hacia la superficie y abriendo atajos peligrosos hacia las cotas profundas.

### 6.4. `100_000` — Despertar (`Wake`)
- **Inicio de la cuenta atrás final (`duration_ticks: 2_700`):** el Durmiente despierta. Comienza una secuencia de evacuación y saqueo final de exactamente `2_700` ticks (`90` segundos).
- **Aparición de el Corazón (`heart_cash_value: 1_500`):** en la cámara más profunda del mapa se abre el cráter abisal y queda expuesto **el Corazón** (*The Heart*), una pieza única rodeada de lava con un valor de rescate de `1_500` en efectivo para el equipo que logre extraerla y subirla a la jaula antes del cierre.
- **Ascenso periódico de lava (`lava_rise_interval_ticks: 90`):** cada `90` ticks (`3` segundos), el nivel global de lava del fondo asciende un estrato vertical, inundando progresivamente las cotas inferiores y obligando a los mineros a trepar hacia la superficie.
- **Parada de líneas de cangilones (`bucket_stop_remaining_ticks: 900`):** cuando restan `900` ticks (`30` segundos) o menos para el final del Despertar (`wake_remaining_ticks <= 900`), el suministro eléctrico industrial cae y **todas las líneas de cangilones se detienen por completo**. Durante los últimos `30` segundos, cualquier mineral o el propio Corazón solo puede subirse a pulso por los jugadores hasta el punto de extracción antes de que el contador llegue a `0`.
