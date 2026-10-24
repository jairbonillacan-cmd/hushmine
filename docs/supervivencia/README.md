# Modo Historia (supervivencia)

Diseño y lore del modo abierto de HUSHMINE. El mundo técnico (generación de 15 M × 15 M bloques, chunks, estructuras) está en [`docs/mundo/`](../mundo/README.md).

Revisión del 24 de octubre de 2026. **Todo es propuesta:** las cifras se ajustan cuando haya algo jugable. Llega después del acceso anticipado del modo Turno, en los hitos W1–W7.

---

## 1. Qué es el modo

Un juego de **supervivencia y construcción en un mundo abierto**:

- **Cada semilla es un mundo único:** relieve, cuevas, biomas, pueblos, ruinas, la posición de los Durmientes, los nombres de los lugares e incluso el número de tu ficha. No hay dos partidas iguales.
- **Todo se puede picar y todo se puede construir.** Cada bloque del mundo se excava; cada casa, mina, máquina o ciudad la levanta el jugador.
- **Se empieza sin nada:** con las manos, una lámpara y una ficha de latón. Se recolecta, se fabrica, se sobrevive a la primera noche y se va bajando.
- **El mundo no se acaba:** se genera al explorar, hasta 15 millones de bloques por lado, con capas que bajan kilómetros.
- **Tiene historia, pero no la impone.** Hay un hilo principal con arcos, jefes, dimensiones y tres finales. Se puede seguir, ignorar o desactivar al crear el mundo (opción **Libre**). Después del final, el mundo sigue.
- **Se juega solo o con otros:** cooperativo con amigos o servidores persistentes. La **Mina Eterna** de `PLAN_v4.md` se convierte en este modo.

## 2. Pilares

1. **El ruido es el enemigo.** Picar, volar roca, encender máquinas o pelear hace ruido. El ruido atrae criaturas y despierta a los Durmientes. Construir en grande obliga a aprender a construir **en silencio**.
2. **Minero, no soldado.** No hay armas de fuego. Las herramientas son las armas y el terreno es el arma principal.
3. **La profundidad es la progresión.** Cada capa y cada dimensión pide mejor equipo y más oficio.
4. **El oro tiene dos destinos:** venderlo para progresar u **ofrendarlo** para calmar a la montaña.
5. **Nadie sabe dónde está este mundo.** No hay fechas, no es la Tierra y el juego nunca lo aclara.
6. **Doble lectura de todo lo extraño:** cada fenómeno admite una explicación «técnica» y otra «mítica». El juego no elige.

## 3. Documentos

| Documento | Contenido |
|---|---|
| [01-lore.md](01-lore.md) | El Silencio: el cielo-Párpado, los Durmientes, los Llegados y sus fichas, los pueblos y facciones, el Insomne |
| [02-historia.md](02-historia.md) | El hilo principal: prólogo, cinco arcos, jefes, finales y qué cambia en cada semilla |
| [03-pueblos-y-comercio.md](03-pueblos-y-comercio.md) | Lampisterías, oficios de los NPC, moneda, precios, encargos, reputación, concesiones y pueblos propios |
| [04-criaturas-y-jefes.md](04-criaturas-y-jefes.md) | Fauna, criaturas de la montaña y de cada dimensión, guardianes, combate por ruido y luz |
| [05-tecnologias.md](05-tecnologias.md) | Las seis ramas tecnológicas, materiales, herramientas, armas, máquinas, automatización y fabricación |
| [06-dimensiones.md](06-dimensiones.md) | El Sueño, el Hervor, el Eco y el Revés: lore, reglas, habitantes, materiales e implementación |

## 4. Glosario (inglés fuente → español)

| Inglés | Español | Qué es |
|---|---|---|
| The Hush | el Silencio | El mundo del modo Historia |
| The Lid | el Párpado | El cielo de roca que se abre (día) y se cierra (noche) |
| Sleepers / Great Sleeper | los Durmientes / el Durmiente Mayor | Colosos geológicos dormidos bajo las montañas |
| The Sleepless | el Insomne | Lo único que nunca duerme en el Silencio |
| Arrivals | los Llegados | Quienes despiertan en el Silencio sin memoria y con una ficha |
| The Still | los Quedos | Pueblo nativo de las profundidades que vive en silencio |
| The Consolidated | la Consolidada | Consorcio de Llegados que quiere vaciar el mundo |
| Watchers | los Vigías | Orden que escucha a los Durmientes |
| Smelters | los Fundidores | Pueblo desaparecido del Hervor |
| Brass tally · Board · Lamp house | Ficha de latón · Tablero · Lampistería | Identidad, registro y refugio de cada Llegado |
| Dreamed gold | Oro soñado | Oro que crean los Durmientes al soñar |
| The Dream · The Seethe · The Echo · The Underside | el Sueño · el Hervor · el Eco · el Revés | Las cuatro dimensiones |
| Lullaby | la Nana | La máquina que puede devolver al sueño a un Durmiente |

Se mantienen los términos de HUSHMINE: Sismógrafo, Ofrenda, Veta Madre, Corazón, Silbadores, geófono y línea de cangilones.

## 5. Qué hace falta en el motor y la simulación

| Necesidad | Dónde | Nota |
|---|---|---|
| Inventario, durabilidad, fabricación, automatización | `hush-sim` | Enteros y datos RON, como el resto de reglas |
| NPC con horarios, tiendas y encargos | `hush-sim` | El cliente solo los dibuja |
| IA de criaturas que oye por la cuadrícula | `hush-sim` | Propagación de sonido entera, la misma idea que `grisu-audio` |
| Dimensiones y épocas | `grisu-world`, `hush-worldgen` | Clave de chunk = (dimensión, posición) |
| Nombres generados por semilla | `hush-worldgen` | Sílabas neutras, válidas en todos los idiomas |
| Diálogo y textos | Fluent | Inglés como fuente; frases cortas |
| Guardado de entidades, inventarios y progreso | `rusqlite` | Ver `docs/mundo/01-escala-y-motor.md` |

Datos previstos: `data/supervivencia/` con `objetos.ron`, `recetas.ron`, `tecnologias.ron`, `criaturas.ron`, `jefes.ron`, `comercio.ron`, `encargos.ron` y `dimensiones.ron`.
