# Especificaciones de Diseño de Reglas y Datos (`HUSHMINE`)

Este directorio contiene las especificaciones formales de diseño de juego de **HUSHMINE** (sección 3 de `PLAN_v4.md`), expresadas íntegramente en aritmética entera y punto fijo `Q16.16` para su ejecución determinista a **30 Hz** en `hush-sim`.

---

## 1. Índice de Documentos y Datos

### Documentación de diseño (`docs/design/`)
- [sismografo.md](sismografo.md) — Escala en milésimas de punto porcentual (`0..=100_000`), acumulador de resto entero para el crecimiento pasivo (`100_000` en `28_800` ticks), fuentes de ruido, orden exacto de evaluación en 6 pasos dentro de cada tick, efectos de los cuatro umbrales (**Revuelo**, **Respira**, **Inquietud**, **Despertar**) y mecánicas provisionales de alivio (**Ofrenda** y **Ficha de latón**).
- [clases-economia.md](clases-economia.md) — Las cuatro clases (**Prospector**, **Minero**, **Ingeniero**, **Zapador**), velocidades en punto fijo `Q16.16`, capacidad de carga, herramientas primarias y secundarias, matriz de construcción y tiempos de picado, economía doble (mena frente a oro/diamante), línea de cangilones, metas `1v1`–`8v8`, **Fama** al `60 %`, **botín robado** `×5 / 4`, bolsa al morir y reaparición en `180` ticks.
- [bloques.md](bloques.md) — Catálogo canónico de los 18 bloques (`id: 0..=17`, Convención A), representación densa de `1 byte` por celda en chunks de `16³` con metadatos dispersos para equipo (`Copper` / `Cobalt`), tabla maestra de durezas en ticks por clase y reglas físicas, acústicas y eléctricas por familia.
- [ritmo-partida.md](ritmo-partida.md) — Informe estadístico generado determinísticamente por `tools/balance/seismo_sim.py` para los cuatro perfiles de equipo (`"ruidoso"`, `"silencioso"`, `"ofrendador"`, `"mixto"`, 50 semillas cada uno), verificando la curva pasiva exacta en el tick `28_800`, la mediana de `"mixto"` entre 9 y 14 minutos (`10:54`) y el techo duro de `31_500` ticks (`17:30`).
- [invariantes.md](invariantes.md) — Doce invariantes formales (`INV-01` a `INV-12`) especificadas como propiedades para `proptest` (sin código Rust), cubriendo conservación del oro y diamante, determinismo multiplataforma, acotación del Sismógrafo en `[0, 100_000]`, exactitud en `28_800` ticks, emisión única y monótona de umbrales, parada de cangilones en los últimos `900` ticks, no negatividad de mena y niebla de mineral por equipo.

### Archivos de datos RON (`data/`)
Todos los archivos `.ron` emplean un subconjunto estricto sin números en coma flotante ni punto decimal:
- `data/seismograph.ron` — Parámetros del Sismógrafo (`SeismographConfig`).
- `data/classes.ron` — Catálogo de las 4 clases, bloques construibles y ticks de picado (`ClassCatalog`).
- `data/economy.ron` — Valores de recursos, línea de cangilones, metas `1v1`–`8v8`, Fama y robo (`EconomyConfig`).
- `data/blocks.ron` — Catálogo maestro de los 18 bloques `0..=17` (`BlockCatalog`).

### Herramientas de verificación y balance (`tools/balance/`)
- `tools/balance/ron_lite.py` — Parser recursivo del subconjunto de RON (rechaza cualquier literal con punto decimal).
- `tools/balance/seismo_sim.py` — Simulador determinista del Sismógrafo y generador de `docs/design/ritmo-partida.md`.
- `tools/balance/test_balance.py` — Suite `unittest` de validación de datos `.ron`, convenciones A–D y ritmo de partida.

---

## 2. Decisiones Abiertas para Revisión Humana

Las siguientes magnitudes quedan marcadas como **provisionales (`decisión abierta`)** y parametrizadas en los archivos `.ron` para su ajuste tras las primeras pruebas jugables del hito 4:

1. **Ofrenda de oro en el altar (`provisional_relief` en `data/seismograph.ron`):**
   - **Propuesta actual:** entregar un lote indivisible de `100 $` (`1` bloque de oro, `offering_gold_cost: 100`) reduce el Sismógrafo en `-2_500` (`-2,5 %`, `offering_reduction: 2_500`), con un enfriamiento de `150` ticks (`5` s, `offering_cooldown_ticks: 150`) por equipo.
   - **Pregunta abierta:** ¿debe el coste de la Ofrenda escalar con el tamaño del equipo (`1v1` frente a `8v8`) o mantenerse fijo en `100 $` ajustando únicamente el enfriamiento?
2. **Devolución de Ficha de latón (`brass_tally` en `data/seismograph.ron`):**
   - **Propuesta actual:** devolver una ficha al altar reduce el Sismógrafo en `-8_000` (`-8,0 %`, `brass_tally_reduction: 8_000`), con un máximo de `2` fichas generadas/entregables por Turno (`max_tallies_per_shift: 2`).
   - **Pregunta abierta:** ¿debe la entrega de una Ficha de latón otorgar además una pequeña bonificación de mena al equipo que la devuelve, o su valor debe ser puramente temporal (`-8,0 %` de Sismógrafo) y narrativo (desbloqueo de crónica de 1931)?
3. **Expiración de bolsas de botín (`loot_bag_lifetime_ticks` en `data/economy.ron`):**
   - **Propuesta actual:** `1_800` ticks (`60` s) antes de expirar si nadie la recoge, para acotar el número de entidades activas en el servidor.
   - **Pregunta abierta:** decidir si al expirar una bolsa su oro desaparece del turno o si las bolsas de alto valor (`>= 500 $`) deben persistir hasta el final del Turno salvo que caigan en lava.
4. **Valor exacto de la Veta Madre y del Corazón (`data/economy.ron` y `data/seismograph.ron`):**
   - **Propuesta actual:** `mother_lode_total_cash: 2_000` (en `50_000` **Respira**) y `heart_total_cash: 1_500` (en `100_000` **Despertar**).
   - **Pregunta abierta:** validar en partidas `1v1` y `8v8` si el valor del Corazón debe ser un porcentaje fijo de `target_cash` (p. ej. `25 %`) en lugar de `1_500 $` planos.

---

## 3. Cambios respecto a `PLAN_v4.md`

No se ha contradicho ninguna regla ni convención de `PLAN_v4.md`. Los siguientes puntos de la sección 3 de `PLAN_v4.md` estaban descritos de forma cualitativa y se han concretado numéricamente para eliminar toda ambigüedad en `hush-sim`:

| Aspecto en `PLAN_v4.md` | Concreción en `docs/design/` y `data/*.ron` | Justificación |
|---|---|---|
| **«Picar: Casi nada»** (tabla del Sismógrafo) | `dig_block: 0` en `data/seismograph.ron` (mientras que el pico sí transmite vibración local por bloques con `transmits_noise: true` hacia el geófono). | Evita penalizar la excavación manual básica en el medidor global y reserva el avance sísmico al tiempo pasivo, líneas de cangilones (`+1/tick`), descargas/lava (`+200`) y explosivos (`+3_000`). |
| **Mapa `64×64×64` (`96×96×64` para `8v8`)** | Se aplica `64×64×64` de `1v1` a `6v6` y se amplía a `96×96×64` tanto en `7v7` como en `8v8` (`data/economy.ron`). | Evita saturación espacial en partidas de 14 jugadores (`7v7`), manteniendo la misma densidad por minero que en `8v8`. |
| **Ofrenda y devolución de ficha («Baja el medidor» / «Lo baja mucho»)** | Fijados provisionalmente en `-2_500` (`-2,5 %` por `100 $` de oro, enfriamiento `150` ticks) y `-8_000` (`-8,0 %` por ficha, máx. `2` por Turno). | Permite que el perfil `"ofrendador"` retrase los umbrales de forma palpable sin superar el techo de `16 min + 90 s` (`31_500` ticks) y mantiene la mediana del perfil `"mixto"` en `10:54` (dentro de `9–14` min). |
| **Tope temporal de 16 minutos frente a ofrendas** | El reloj del Turno fuerza la entrada en **Despertar** como muy tarde en el tick `28_800` (`16:00`), e impide ofrendas una vez iniciado el Despertar (`2_700` ticks). | Garantiza la invariante `INV-07`: ninguna partida puede superar jamás `16 min + 90 s` (`31_500` ticks), incluso si ambos equipos ofrendan continuamente. |
| **Valores de oro, diamante, Veta Madre, Corazón y metas `1v1`–`8v8`** | `ore: 4` uds/bloque; `gold: 100 $` (`1` lingote); `diamond: 400 $` (`2` gemas de `200 $`); Veta Madre `2_000 $`; el Corazón `1_500 $`; metas de `2_000 $` (`1v1`) a `12_500 $` (`8v8`, `+1_500 $` por jugador). | Proporciona enteros cerrados compatibles con el multiplicador de robo `×5 / 4` (`125 $` y `250 $`/`500 $`) y el umbral de Fama `60 / 100`. |
| **Bolsas al morir y cadencia de cangilones y lava** | Expiración de bolsa en `1_800` ticks (`60` s); línea de cangilones de hasta `16` postes espaciados `<= 8` bloques que transfiere `1` unidad cada `60` ticks (`2` s); ascenso de lava en el Despertar cada `90` ticks (`3` s). | Acota las entidades activas en memoria y formaliza en ticks enteros las mecánicas logísticas y de final de partida. |
