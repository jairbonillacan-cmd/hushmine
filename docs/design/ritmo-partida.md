# Análisis de Ritmo de Partida y Sismógrafo (HUSHMINE)

Documento generado de forma determinista por `tools/balance/seismo_sim.py` a partir de
`data/seismograph.ron` y `data/economy.ron` (50 semillas fijas `1..50` por perfil, a `30 Hz`).

## 1. Verificación del Crecimiento Pasivo Puro (Aritmética Entera)

El Sismógrafo utiliza un acumulador entero sin coma flotante (`total_value = 100_000`, `total_ticks = 28_800`):

- **Fórmula por tick:** `accum += 100_000; delta = accum // 28_800; accum %= 28_800`
- **Medidor en tick 28_799:** `99996` (a 4 milésimas del tope)
- **Medidor en tick 28_800:** `100000` (exactamente `16:00` sin deriva)

| Umbral | Nombre (ES) | Valor | Tick pasivo | Tiempo pasivo (`MM:SS`) |
| :--- | :--- | ---: | ---: | :---: |
| `Stir` | Revuelo | `25_000` | `7200` | `04:00` |
| `Breathe` | Respira | `50_000` | `14400` | `08:00` |
| `Restless` | Inquietud | `75_000` | `21600` | `12:00` |
| `Wake` | Despertar | `100_000` | `28800` | `16:00` |

## 2. Distribución de Duración de Partida (4v4, Meta `6_500 $`)

| Perfil | Mín (`MM:SS`) | P25 (`MM:SS`) | Mediana (`MM:SS`) | Media (`MM:SS`) | P75 (`MM:SS`) | Máx (`MM:SS`) | Mediana (ticks) | Máx (ticks) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | ---: | ---: |
| **ruidoso** | `07:34` | `08:25` | **`08:32`** | `08:32` | `08:49` | `09:04` | `15349` | `16328` |
| **silencioso** | `12:36` | `14:12` | **`14:54`** | `14:50` | `15:17` | `17:01` | `26805` | `30616` |
| **ofrendador** | `13:42` | `15:41` | **`16:21`** | `16:22` | `17:08` | `17:30` | `29442` | `31500` |
| **mixto** | `10:15` | `10:39` | **`10:54`** | `11:02` | `11:21` | `12:07` | `19606` | `21797` |

## 3. Mediana de Cruce de Umbrales y Condición de Victoria

| Perfil | Revuelo (25%) | Respira (50%) | Inquietud (75%) | Despertar (100%) | % Despertar | % Victoria Meta | % Victoria Reloj |
| :--- | :---: | :---: | :---: | :---: | ---: | ---: | ---: |
| **ruidoso** | `02:29` | `04:11` | `05:44` | `07:12` | `100.0%` | `46.0%` | `54.0%` |
| **silencioso** | `03:56` | `07:50` | `11:45` | `13:26` | `62.0%` | `82.0%` | `18.0%` |
| **ofrendador** | `03:16` | `08:20` | `11:39` | `15:17` | `96.0%` | `36.0%` | `64.0%` |
| **mixto** | `02:42` | `05:19` | `07:33` | `09:24` | `100.0%` | `8.0%` | `92.0%` |

## 4. Actividad Sísmica Media y Alivio Provisional por Partida

| Perfil | Explosiones medias | Ofrendas de oro medias | Fichas de latón medias | Invariantes OK |
| :--- | ---: | ---: | ---: | :---: |
| **ruidoso** | `7.8` | `0.0` | `0.0` | Sí (`50/50`) |
| **silencioso** | `0.0` | `0.0` | `0.0` | Sí (`50/50`) |
| **ofrendador** | `4.2` | `9.7` | `2.0` | Sí (`50/50`) |
| **mixto** | `6.1` | `1.7` | `0.5` | Sí (`50/50`) |

## 5. Conclusiones de Equilibrio y Cumplimiento de Invariantes

1. **Objetivo de duración competitiva (9 a 14 min):** El perfil **`mixto`** presenta una mediana de **`10:54`** (`10.89` min, `19606` ticks), dentro de la ventana objetivo `[16_200, 25_200]` ticks.
2. **Compresión por ruido agresivo:** El perfil **`ruidoso`** adelanta el Despertar a ~7,5 min gracias al uso intensivo de cargas de demolición (`+3_000`) y múltiples líneas de cangilones (`+1/tick`), cerrando partidas entre 8 y 9,5 minutos.
3. **Ritmo metódico sin explosivos:** El perfil **`silencioso`** retrasa todos los umbrales casi hasta su curva pasiva y resuelve las partidas alrededor de 13,5 a 15,2 minutos.
4. **Techo duro de partida (`<= 31_500` ticks):** Incluso en el perfil **`ofrendador`**, donde ambos equipos invierten oro en el altar (`-2_500`) y consumen las 2 fichas de latón (`-8_000`), el cierre forzoso de turno en `28_800` ticks más los `2_700` ticks del Despertar garantiza que ninguna partida supere `31_500` ticks (`17:30`). Máximo observado: `31500` ticks (`17:30`).
