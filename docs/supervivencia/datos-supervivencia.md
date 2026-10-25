# Datos de supervivencia (`data/supervivencia/`)

Estado de los 8 archivos de datos del modo Historia, su esquema exacto y el
candado automático que los verifica. El diseño (lore, historia, economía,
criaturas, tecnologías y dimensiones) está en el [README del modo](README.md);
esta página es la referencia técnica de los datos que consumirá `hush-sim`.

**Índice**

1. [Cómo verificar](#1-cómo-verificar)
2. [Reglas comunes](#2-reglas-comunes)
3. [objetos.ron](#3-objetosron)
4. [tecnologias.ron](#4-tecnologiasron)
5. [recetas.ron](#5-recetasron)
6. [criaturas.ron](#6-criaturasron)
7. [jefes.ron](#7-jefesron)
8. [dimensiones.ron](#8-dimensionesron)
9. [comercio.ron](#9-comercioron)
10. [encargos.ron](#10-encargosron)
11. [Glosario de ids](#11-glosario-de-ids)

---

## 1. Cómo verificar

El verificador es el candado final de los datos: si falla algo, no se mergea.

```bash
# Verificador completo (código 0 si todo pasa, 1 si hay fallos)
python tools/supervivencia/check_survival.py

# Pruebas unitarias del verificador y de los datos
python -m unittest discover -s tools/supervivencia -v
```

Ambos se ejecutan desde la raíz del repositorio. El verificador solo usa la
biblioteca estándar de Python y el parser `tools/balance/ron_lite.py`; agrupa
los fallos por fases en este orden:

| Fase | Qué comprueba |
|---|---|
| a) Higiene | Cada archivo existe, es UTF-8, usa solo LF, tiene ≤ 400 líneas y termina en salto de línea |
| b) Parseo | Cada archivo parsea con `ron_lite` y no contiene números con punto decimal |
| c) Esquema | Rangos, conjuntos cerrados, mínimos y unicidad de ids de cada archivo |
| d) Referencias | Todo `item_id` y `tech_id` referenciado existe en su catálogo de origen |
| e) Anti-ganancia | Ninguna receta produce más valor en oro del que consume |
| f) Anti-arbitraje | En toda oferta `buy_price_gold > sell_price_gold` |

Los fallos se imprimen como `[FALLO] ...` (en español); si todo pasa, se
imprime un `[OK]` por fase. Las pruebas de `test_survival.py` se saltan con
`faltan datos` si los RON aún no existen; las dos pruebas autocontenidas
(RON de muestra en línea) pasan siempre.

## 2. Reglas comunes

- **Solo enteros, nunca flotantes.** El parser prohíbe literales con punto
  decimal. Para magnitudes fraccionarias se usa punto fijo Q16.16 en un
  entero: `65_536` representa `1.0` (por ejemplo, la gravedad de 1 g).
  Un entero negativo invierte la gravedad; `0` está prohibido en
  `gravity_q16`.
- **Archivos pequeños (regla de AGENTS.md):** máximo 400 líneas por archivo,
  saltos de línea LF (ningún byte CR), UTF-8 sin BOM y salto de línea final.
- **Identificadores en inglés; comentarios y documentación en español.**
  Los `id` de los datos son el texto fuente en inglés; el texto visible del
  juego se escribe en Fluent (inglés como idioma fuente).
- **Determinismo:** nada de aleatoriedad del sistema ni recorrido de
  `HashMap` donde el orden importe; los catálogos son listas ordenadas.

## 3. objetos.ron

Catálogo de items del modo Historia. Mínimo **36** items, ids únicos.
Diseño: [05-tecnologias.md](05-tecnologias.md) y [03-pueblos-y-comercio.md](03-pueblos-y-comercio.md).

Raíz: `( items: [ ... ] )`.

| Campo | Tipo | Restricción |
|---|---|---|
| `id` | cadena | Único en el archivo |
| `name` | cadena | Nombre fuente en inglés |
| `category` | cadena | `material`, `tool`, `lamp`, `explosive`, `machine_part`, `relic`, `consumable`, `food` |
| `tier` | entero | `1..=5` |
| `max_stack` | entero | `>= 1` |
| `durability` | entero | `>= 0` |
| `weight` | entero | `>= 1` |
| `base_value_gold` | entero | `>= 0`; base del valor en oro |

## 4. tecnologias.ron

Árbol tecnológico de las seis ramas. Mínimo **24** nodos, ids únicos, y
**cada rama con al menos un nodo en cada tier 1..=5**.
Diseño: [05-tecnologias.md](05-tecnologias.md).

Raíz: `( techs: [ ... ] )`.

| Campo | Tipo | Restricción |
|---|---|---|
| `id` | cadena | Único en el archivo |
| `name` | cadena | Nombre fuente en inglés |
| `branch` | cadena | `excavation`, `shoring_construction`, `lighting_detection`, `transport_machines`, `silence_acoustics`, `sleeper_craft` |
| `tier` | entero | `1..=5` |
| `prereqs` | lista de ids | Cada id existe en este archivo y su `tier` es estrictamente menor |
| `unlocks` | lista de ids | Cada id existe en `objetos.ron` |

## 5. recetas.ron

Fabricación en estaciones. Mínimo **28** recetas, ids únicos.
Diseño: [05-tecnologias.md](05-tecnologias.md).

Raíz: `( recipes: [ ... ] )`.

| Campo | Tipo | Restricción |
|---|---|---|
| `id` | cadena | Único en el archivo |
| `name` | cadena | Nombre fuente en inglés |
| `tech_required` | cadena | Id existente en `tecnologias.ron` |
| `station` | cadena | `workbench`, `forge`, `pressure_workshop`, `powder_magazine`, `grow_table`, `quiet_loom`, `memory_altar` |
| `inputs` | lista de `(item_id, qty)` | `item_id` existe en `objetos.ron`; `qty >= 1`; lista no vacía |
| `outputs` | lista de `(item_id, qty)` | Igual que `inputs` |
| `craft_ticks` | entero | `>= 1` |
| `noise_emitted` | entero | `>= 0`; ruido por crafteo (el pilar «el ruido es el enemigo») |

**Invariante anti-ganancia:** por receta,
`sum(qty × base_value_gold de outputs) <= sum(qty × base_value_gold de inputs)`.
Sin ella, un bucle de crafteo sería una máquina de oro infinita.

## 6. criaturas.ron

Fauna por hábitat, con oído y reacción a la luz. Mínimo **14** criaturas,
ids únicos. Diseño: [04-criaturas-y-jefes.md](04-criaturas-y-jefes.md).

Raíz: `( creatures: [ ... ] )`.

| Campo | Tipo | Restricción |
|---|---|---|
| `id` | cadena | Único en el archivo |
| `name` | cadena | Nombre fuente en inglés |
| `habitat` | cadena | `surface`, `crust`, `mantle`, `vault`, `roots`, `the_dream`, `the_seethe`, `the_echo`, `the_underside` |
| `hearing_threshold` | entero | `>= 0`; ruido mínimo que la despierta |
| `light_reaction` | cadena | `flee`, `attract`, `neutral`, `petrify` |
| `hp` | entero | `>= 1` |
| `damage` | entero | `>= 0` |
| `noise_on_alert` | entero | `>= 0`; ruido que emite al alertarse |
| `drops` | lista de `(item_id, min_qty, max_qty)` | `item_id` existe en `objetos.ron`; `0 <= min_qty <= max_qty` |

## 7. jefes.ron

Guardianes de los cinco arcos. Mínimo **5** jefes, ids únicos, cada uno con
**al menos 2 fases**. Diseño: [04-criaturas-y-jefes.md](04-criaturas-y-jefes.md)
y [02-historia.md](02-historia.md).

Raíz: `( bosses: [ ... ] )`.

| Campo | Tipo | Restricción |
|---|---|---|
| `id` | cadena | Único en el archivo |
| `name` | cadena | Nombre fuente en inglés |
| `arc` | entero | `1..=5` (un arco de la historia) |
| `arena` | cadena | Lugar del combate |
| `hp` | entero | `>= 1` |
| `damage` | entero | `>= 0` |
| `acoustic_weakness` | entero | `0..=100`; debilidad al silencio/ruido |
| `luminous_weakness` | entero | `0..=100`; debilidad a la luz |
| `phases` | lista de `(id, name, hp_below_pct, tactic)` | Al menos 2; `hp_below_pct` en `0..=100` |
| `rewards` | lista de `(item_id, qty)` | `item_id` existe en `objetos.ron`; la lista puede estar vacía |

## 8. dimensiones.ron

Las cinco dimensiones del mundo. Exactamente **5** entradas con estos ids e
índices. Diseño: [06-dimensiones.md](06-dimensiones.md).

Raíz: `( dimensions: [ ... ] )`.

| Campo | Tipo | Restricción |
|---|---|---|
| `id` | cadena | Exactamente: `the_hush` (index 0), `the_dream` (1), `the_seethe` (2), `the_echo` (3), `the_underside` (4) |
| `index` | entero | El índice exigido por el id |
| `name` | cadena | Nombre fuente en inglés |
| `gravity_q16` | entero | Distinto de `0`; en Q16.16 `65_536` = 1 g; negativo = gravedad invertida |
| `light_rule` | cadena | `lid_cycle`, `drifting_epochs`, `always_glow`, `sound_light`, `always_day`; **cada valor se usa exactamente una vez** |
| `noise_multiplier_q16` | entero | `>= 1`; multiplicador de ruido en Q16.16 |
| `exclusive_material` | cadena | Id existente en `objetos.ron`; solo `the_hush` admite la cadena vacía |

## 9. comercio.ron

Comerciantes, precios y reputación. Mínimo **5** traders, incluyendo
obligatoriamente: `lamplighter`, `blacksmith`, `assayer`, `cartographer` y
`quedo_barterer`. Diseño: [03-pueblos-y-comercio.md](03-pueblos-y-comercio.md).

Raíz: `( traders: [ ... ] )`.

| Campo | Tipo | Restricción |
|---|---|---|
| `id` | cadena | Único; los cinco obligatorios deben estar |
| `name` | cadena | Nombre fuente en inglés |
| `faction` | cadena | `arrivals`, `the_still`, `watchers`, `consolidated` |
| `offers` | lista de `(item_id, buy_price_gold, sell_price_gold)` | `item_id` existe en `objetos.ron`; `buy_price_gold > sell_price_gold >= 1` |
| `reputation_tiers` | lista de `(min_reputation, discount_pct)` | Ordenado ascendente por `min_reputation` en `-100..=100`; `discount_pct` en `0..=60` no decreciente |

Convención de precios: `buy` es lo que paga el jugador al comprar y `sell` lo
que recibe al vender. **Invariante anti-arbitraje:** como `buy > sell`, nunca
se gana oro comprando y revendiendo al mismo comerciante.

## 10. encargos.ron

Encargos de las facciones. Mínimo **12** encargos, ids únicos.
Diseño: [03-pueblos-y-comercio.md](03-pueblos-y-comercio.md).

Raíz: `( orders: [ ... ] )`.

| Campo | Tipo | Restricción |
|---|---|---|
| `id` | cadena | Único en el archivo |
| `name` | cadena | Nombre fuente en inglés |
| `kind` | cadena | `delivery`, `survey`, `drain`, `reopen`, `rescue`, `escort`, `offering`, `contract`, `listen`, `message` |
| `faction` | cadena | `arrivals`, `the_still`, `watchers`, `consolidated` |
| `target_item_id` | cadena | Existe en `objetos.ron` |
| `target_qty` | entero | `>= 1` |
| `reward_gold` | entero | `>= 0` |
| `reward_reputation` | entero | `-100..=100` |

## 11. Glosario de ids

Inglés fuente → español, tal como lo usan los ids de estos archivos
(glosario completo en el [README](README.md)):

| Inglés (ids) | Español |
|---|---|
| `the_hush` / The Hush | el Silencio (el mundo) |
| `the_dream` / The Dream | el Sueño |
| `the_seethe` / The Seethe | el Hervor |
| `the_echo` / The Echo | el Eco |
| `the_underside` / The Underside | el Revés |
| The Lid / `lid_cycle` | el Párpado (el cielo que se abre y se cierra) |
| sleepers / Great Sleeper | los Durmientes / el Durmiente Mayor |
| The Sleepless | el Insomne |
| arrivals | los Llegados |
| `the_still` / The Still | los Quedos |
| watchers | los Vigías |
| consolidated | la Consolidada |
| smelters | los Fundidores |
| brass tally · board · lamp house (`lamplighter`) | ficha de latón · tablero · lampistería |
| dreamed gold | oro soñado |
| lullaby | la Nana |
