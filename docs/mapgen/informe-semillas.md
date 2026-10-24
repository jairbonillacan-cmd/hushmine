# Informe de análisis de semillas — Generador procedural HUSHMINE

Evaluación estadística y verificación de invariantes sobre el lote de **20 semillas
(`1..=20`)** más la **semilla de referencia `42`** para partidas `4v4`
(`64×64×64` = `262,144` vóxeles, meta de victoria `6,500 $`).

## 1. Resumen ejecutivo de invariantes

- **Semillas del lote (`1..=20`) que superan todos los chequeos:** `20/20` (`100 %`).
- **Semilla de referencia (`42`):** `PASS` (`sha256: 986cde4009c52e4a…`).
- **Criterios verificados automáticamente (`checks`):**
  - `valid_block_ids`: todos los vóxeles pertenecen al catálogo `0..=17`.
  - `copper_has_1_5x_target` y `cobalt_has_1_5x_target`: cada equipo dispone de $\ge 1.5\times$ la meta (`>= 9,750 $`) en su propio hemisferio.
  - `equity_within_10pct`: diferencia de oro alcanzable entre Copper y Cobalt $\le 10\%$.
  - `lava_safe_from_bases`: distancia Chebyshev ($L_\infty$) mínima de cualquier bloque de lava a las cajas de base $\ge 6$ celdas.

## 2. Métricas por semilla (`1..=20` y referencia `42`)

| Semilla | SHA-256 (prefijo) | Oro Copper ($) | Ratio Copper | Oro Cobalt ($) | Ratio Cobalt | Dif. ($) | Dif. (%) | Diamantes (Cu/Co/Tot) | Lava (bloques) | Dist. Lava $L_\infty$ (Cu/Co) | Dist. Lava $L_1$ (Cu/Co) | Estado |
|---:|:---|---:|---:|---:|---:|---:|---:|:---:|---:|:---:|:---:|:---:|
| 1 | `bc5673733657` | 16,000 | 2.462x | 16,100 | 2.477x | 100 | 0.62% | 12/12/24 | 200 | 31/31 | 43/43 | `PASS` |
| 2 | `db367f4fb6ca` | 16,300 | 2.508x | 16,300 | 2.508x | 0 | 0.00% | 9/9/18 | 202 | 30/30 | 56/56 | `PASS` |
| 3 | `ae65f230e391` | 16,200 | 2.492x | 16,200 | 2.492x | 0 | 0.00% | 12/12/24 | 212 | 33/33 | 57/57 | `PASS` |
| 4 | `95c8f797cbcf` | 16,000 | 2.462x | 16,100 | 2.477x | 100 | 0.62% | 10/10/20 | 200 | 31/31 | 49/49 | `PASS` |
| 5 | `f7f02d627455` | 16,200 | 2.492x | 16,400 | 2.523x | 200 | 1.22% | 12/12/24 | 248 | 35/35 | 58/58 | `PASS` |
| 6 | `2afa8de135a2` | 16,100 | 2.477x | 16,100 | 2.477x | 0 | 0.00% | 12/12/24 | 166 | 31/31 | 39/39 | `PASS` |
| 7 | `4bc7c4a9bbc4` | 16,200 | 2.492x | 16,400 | 2.523x | 200 | 1.22% | 12/12/24 | 244 | 31/31 | 45/45 | `PASS` |
| 8 | `11e4f8bbdaa4` | 16,300 | 2.508x | 16,500 | 2.538x | 200 | 1.21% | 12/12/24 | 236 | 30/30 | 49/49 | `PASS` |
| 9 | `db971ccf27b0` | 16,200 | 2.492x | 16,300 | 2.508x | 100 | 0.61% | 12/12/24 | 166 | 33/33 | 52/52 | `PASS` |
| 10 | `21148ee97d2b` | 16,300 | 2.508x | 16,400 | 2.523x | 100 | 0.61% | 12/12/24 | 212 | 33/33 | 46/46 | `PASS` |
| 11 | `0b3947c5cc78` | 15,900 | 2.446x | 16,000 | 2.462x | 100 | 0.62% | 12/12/24 | 168 | 39/39 | 46/46 | `PASS` |
| 12 | `d648f771f424` | 16,000 | 2.462x | 16,000 | 2.462x | 0 | 0.00% | 12/12/24 | 156 | 42/42 | 52/52 | `PASS` |
| 13 | `bc8d594b9286` | 16,300 | 2.508x | 16,400 | 2.523x | 100 | 0.61% | 10/10/20 | 212 | 30/30 | 51/51 | `PASS` |
| 14 | `9f16daad9248` | 16,200 | 2.492x | 16,400 | 2.523x | 200 | 1.22% | 12/12/24 | 214 | 30/30 | 52/52 | `PASS` |
| 15 | `e6880356a4b5` | 16,000 | 2.462x | 16,200 | 2.492x | 200 | 1.23% | 12/12/24 | 246 | 34/34 | 40/40 | `PASS` |
| 16 | `9486dd604ec3` | 16,400 | 2.523x | 16,400 | 2.523x | 0 | 0.00% | 12/12/24 | 188 | 32/32 | 42/42 | `PASS` |
| 17 | `6114b7f36df9` | 16,000 | 2.462x | 16,200 | 2.492x | 200 | 1.23% | 12/12/24 | 156 | 33/33 | 51/51 | `PASS` |
| 18 | `9e9d4384443e` | 15,900 | 2.446x | 16,100 | 2.477x | 200 | 1.24% | 12/12/24 | 246 | 30/30 | 40/40 | `PASS` |
| 19 | `582c986b9cd8` | 16,300 | 2.508x | 16,400 | 2.523x | 100 | 0.61% | 11/11/22 | 168 | 30/30 | 47/47 | `PASS` |
| 20 | `c6b31d0c98b8` | 16,000 | 2.462x | 16,200 | 2.492x | 200 | 1.23% | 12/12/24 | 258 | 33/33 | 49/49 | `PASS` |
| **42 (ref)** | `986cde4009c5` | 16,200 | 2.492x | 16,200 | 2.492x | 0 | 0.00% | 10/10/20 | 166 | 30/30 | 48/48 | `PASS` |

## 3. Estadísticos agregados del lote (`1..=20`)

| Métrica | Media | Mínimo | Máximo | Umbral / Invariante |
|:---|---:|---:|---:|:---|
| Oro alcanzable Copper ($) | 16,140.0 | 15,900 | 16,400 | $\ge 9,750\ \$$ (`1.5x`) |
| Oro alcanzable Cobalt ($) | 16,255.0 | 16,000 | 16,500 | $\ge 9,750\ \$$ (`1.5x`) |
| Ratio oro / meta Copper | 2.483x | 2.446x | 2.523x | $\ge 1.500\times$ |
| Ratio oro / meta Cobalt | 2.501x | 2.462x | 2.538x | $\ge 1.500\times$ |
| Diferencia entre equipos (%) | 0.71% | 0.00% | 1.24% | $\le 10.00\%$ |
| Volumen de lava (bloques) | 204.9 | 156 | 258 | Bolsas profundas $y \in [4, 23]$ |
| Distancia mínima base-lava $L_\infty$ | 32.5 | 30 | 42 | $\ge 6$ celdas |
| Distancia mínima base-lava $L_1$ | 48.2 | 39 | 58 | $\ge 6$ celdas |

## 4. Distribución media de bloques por tipo (`1..=20`)

| ID | Nombre (`BlockId`) | Media (bloques) | Mínimo | Máximo | Proporción media (%) |
|---:|:---|---:|---:|---:|---:|
| 0 | `air` | 1,370.0 | 1,086 | 1,682 | 0.5226% |
| 1 | `dirt` | 60,832.9 | 60,694 | 60,944 | 23.2059% |
| 2 | `rock` | 170,455.3 | 169,676 | 171,060 | 65.0235% |
| 3 | `hard_rock` | 28,531.4 | 28,038 | 29,052 | 10.8839% |
| 4 | `ore` | 222.3 | 204 | 240 | 0.0848% |
| 5 | `gold` | 323.9 | 319 | 328 | 0.1236% |
| 6 | `diamond` | 23.2 | 18 | 24 | 0.0089% |
| 7 | `lava` | 204.9 | 156 | 258 | 0.0782% |
| 8 | `team_block` | 94.0 | 94 | 94 | 0.0359% |
| 9 | `force_field` | 18.0 | 18 | 18 | 0.0069% |
| 10 | `ladder` | 62.0 | 62 | 62 | 0.0237% |
| 11 | `jump_pad` | 0.0 | 0 | 0 | 0.0000% |
| 12 | `electric` | 0.0 | 0 | 0 | 0.0000% |
| 13 | `rail` | 0.0 | 0 | 0 | 0.0000% |
| 14 | `beacon` | 2.0 | 2 | 2 | 0.0008% |
| 15 | `explosive` | 0.0 | 0 | 0 | 0.0000% |
| 16 | `bucket_station` | 2.0 | 2 | 2 | 0.0008% |
| 17 | `bucket_post` | 2.0 | 2 | 2 | 0.0008% |

## 5. Conclusiones de diseño y validación

1. **Suficiencia económica holgada (`> 2.4x` la meta por hemisferio):** En todas las semillas evaluadas,
   tanto Copper como Cobalt cuentan con entre `15,900 $` y `16,500 $` en bloques de oro alcanzables dentro
   de su propia mitad del mapa (sin contar diamantes ni el Corazón), superando ampliamente el mínimo de
   `1.5x` (`9,750 $` para `4v4`).
2. **Equidad competitiva estricta (`< 1.3 %` de asimetría):** Las vetas principales de mena, oro y diamante se
   generan por reflexión especular `x <-> W - 1 - x` y la perturbación asimétrica controlada (`2..=5` bloques
   de oro extra repartidos entre ambos lados) mantiene la diferencia entre equipos en `1.24 %` como máximo,
   muy dentro del margen permitido del `10 %`.
3. **Seguridad térmica en bases (`L_inf >= 30`):** Las bolsas de lava están confinadas al estrato profundo
   (`y <= 23`, `17 <= x <= W - 18`), garantizando una distancia Chebyshev mínima de `30` celdas (y Manhattan
   mínima de `39` celdas) respecto a las cajas de base de Copper y Cobalt, muy superior al umbral de `6` celdas.
