# Mundo grande · 3. Estructuras, minas e historia

Cómo se colocan y se construyen las estructuras sin romper la regla de que el mundo es una función pura. Índice en [README.md](README.md).

---

## 1. Principio: las estructuras consultan funciones, no chunks

Una estructura puede ocupar decenas de chunks y tiene que salir idéntica sin importar cuál de ellos se genere primero. Por eso:

1. **Dónde va** se decide con una rejilla y un hash, no buscando en el mundo ya generado.
2. **Cómo es** (su plano de piezas) es una función pura de su punto de partida.
3. Para adaptarse al terreno, el plano **solo consulta las funciones analíticas del generador**: altura de la columna, densidad, cuevas por ruido, gusanos, nivel de acuífero y menas. Nunca lee chunks generados ni otras estructuras.
4. Cada chunk **rasteriza solo las piezas que lo cortan**.

Así, todo se puede generar en paralelo y en cualquier orden, y `INV-W02` (sin costuras) se cumple por construcción.

## 2. Colocación por rejilla

Cada tipo de estructura define en `data/worldgen/estructuras.ron`:

| Campo | Significado |
|---|---|
| `espaciado` | Lado de la celda de rejilla, en chunks |
| `separacion` | Distancia mínima al borde de la celda, para que dos estructuras vecinas no se pisen |
| `probabilidad` | Probabilidad (en Q16) de que la celda tenga estructura |
| `radio_max` | Radio máximo del plano, en chunks. Acota cuántas celdas vecinas consulta cada chunk |
| `condiciones` | Biomas, profundidad, pendiente, distancia al origen, distancia a otras familias |

En cada celda, `hash(semilla, tipo, celda)` decide si hay estructura, en qué punto y con qué rotación (0, 90, 180 o 270°). Un chunk consulta solo las celdas a menos de `radio_max` y, para cada una que tenga estructura, pide su plano.

## 3. Planos por piezas

Los planos se ensamblan con piezas, como un puzle:

- **Pieza:** una plantilla de bloques con **conectores** (posición, orientación y tipo: `galeria`, `pozo`, `puerta`, `calle`, `via`).
- **Expansión:** desde la pieza inicial, en anchura. Para cada conector abierto se elige una pieza compatible por peso con `hash(partida, número de pieza)`. Se rechaza si choca con una pieza del mismo plano, si se sale de `radio_max` o si sus condiciones de terreno fallan.
- **Cierre:** al llegar al límite de profundidad o de piezas, los conectores abiertos se tapan con piezas finales: fondos de saco, derrumbes o puertas tapiadas.
- **Caché:** el plano se guarda en una LRU porque varios chunks lo piden. Es solo una optimización: recalcularlo da lo mismo.

### Adaptación al terreno

| Anclaje | Comportamiento |
|---|---|
| **Superficie** | Sigue la altura analítica. Pone cimientos hasta el suelo y despeja el aire por encima |
| **Subterráneo** | Sustituye roca. Si cruza una cueva, pone puentes y pilares de madera |
| **Colgante** | Cuelga del techo de una cámara (nidos, plataformas) |
| **Inundable** | Por debajo del acuífero local se llena de agua (galerías profundas de las minas) |

### Envejecimiento

Cada pieza tiene una edad: **reciente**, **vieja** o **antiquísima**. Un hash por bloque decide qué se pudre, se derrumba o se oxida: entibado roto, raíles sueltos, techos hundidos, escombros. Una mina de los Antiguos tiene que parecer abandonada desde hace generaciones.

### Plantillas

- Se dibujan en el **modo Taller** y se exportan a `assets/estructuras/<familia>/<pieza>.ron` (paleta + RLE + conectores + metadatos).
- `hush-cli world piezas --check` comprueba que cada conector tiene pareja y que ninguna plantilla supera su caja declarada.

## 4. Minas de verdad

Las minas son la seña del juego y tienen que parecer obra de mineros, no pasillos al azar. El generador de minas es un tipo especial de plano que **lee la función de menas**: las galerías van a donde está el mineral.

### Anatomía de una mina de los Antiguos

Los Antiguos son los Llegados que cavaron antes que nadie. No se sabe cuándo ([lore](../supervivencia/01-lore.md)).

| Elemento | Qué es | Cómo se genera |
|---|---|---|
| **Castillete y bocamina** | Torre del pozo en superficie, casa de máquinas, tablero de fichas | Pieza de superficie en la boca del pozo |
| **Pozo maestro** | Pozo vertical con jaula y escaleras de emergencia | Desde la superficie hasta el nivel más hondo |
| **Niveles** | Plantas cada 30–50 m | 3–8 niveles según el tamaño |
| **Estación de pozo** | Ensanche con enganches, vagonetas y señales en cada nivel | Pieza fija en cada nivel |
| **Galerías transversales** | Cruzan la roca estéril desde el pozo hasta la veta | Línea recta hacia la veta más cercana (consulta de menas) |
| **Galerías en dirección** | Siguen la veta a lo largo | Recorren la veta muestreando la función de menas |
| **Rebajes (cámaras de explotación)** | Huecos donde se sacó el mineral, con pilares de seguridad | Donde la veta es más gruesa |
| **Coladeros y pozos interiores** | Conectan niveles sin pasar por el pozo maestro | 1–3 por mina |
| **Chimeneas de ventilación** | Suben hasta la superficie | Una por mina; entrada alternativa |
| **Polvorín** | Almacén de explosivos, apartado | Lejos del pozo, al final de una galería |
| **Casa de bombas** | En el nivel más bajo; si no funciona, todo lo de abajo está inundado | Junto al pozo |
| **Hundimientos** | Tramos derrumbados que cortan la mina | Por hash, según edad y bioma |

Detalles que dan vida: carteles de seguridad solo con iconos (en el estilo de `docs/lore/anima-consolidated.md`, sin texto ni fechas), vagonetas volcadas, lámparas apagadas, raíles con desvíos, un tablero de fichas en la bocamina y cuadernos en las oficinas.

### Otras familias de minas

- **Cateos:** galerías cortas de prospección (10–40 m), muy frecuentes. Enseñan a leer la roca.
- **Minas de la Consolidada:** obras industriales activas: entibado de hierro, calderas, cangilones, guardias y planos que se pueden robar.
- **Minas a cielo abierto y trincheras** en el desierto rojo y el altiplano.

## 5. Catálogo de estructuras (propuesta)

| Estructura | Dónde | Espaciado aprox. | Tamaño | Qué ofrece |
|---|---|---|---|---|
| Cateo | Corteza somera | 256 bloques | 10–40 m | Mineral fácil, aprendizaje |
| **Mina de los Antiguos** | Corteza y manto alto | 1–2 km | Radio ≤ 256 m, 3–8 niveles | Herramientas, raíles, mapas, fichas perdidas |
| Castillete y bocamina | Superficie, sobre cada mina | — | 20–40 m | Punto de referencia visible a kilómetros |
| **Pueblo de Llegados** (vivo o fantasma) | Valles, altiplano, desierto | 3–5 km | 100–250 m | Lampistería, tiendas, encargos; o ruinas y botín |
| **Aldea de los Quedos** | Cuevas grandes de corteza y manto | 2–4 km | 60–150 m | Trueque silencioso, Resonancia |
| **Campamento de la Consolidada** | Junto a minas grandes y bocas de horno | 3–6 km | 50–120 m | Vales, contratos, maquinaria |
| **Torre de los Vigías** | Cumbres | 6–10 km | 10–30 m, muy alta | Mapas, radio, avisos |
| **Ruinas de los Fundidores** | Manto profundo (y el Hervor) | 4 km | 40–200 m | Planos de la Presión, Forjados |
| **Campana del Eco** (ruina) | Geodas del manto | 5 km | 10–20 m | Paso al Eco |
| Caminos y vías estrechas | Entre pueblos vecinos | — | Lineal | Navegación; cada pueblo enlaza solo con los de las celdas vecinas, así que el radio sigue acotado |
| Campamento de buscadores | Superficie y entradas de cueva | 512 bloques | 10–20 m | Provisiones, pistas |
| Santuario de ofrenda | Corteza y manto | 768 bloques | 3–8 m | Ofrenda para calmar al Durmiente (sección 7) |
| Gruta de los Silbadores | Grutas de grisú del manto | 2 km | 40–80 m | Criaturas (cuando existan) |
| Geoda gigante | Manto | 1 km | 10–30 m | Cristal, luz |
| Puente de cuerda, andamios | Sobre cámaras gigantes y la Bóveda | Según cueva | Variable | Rutas peligrosas |

**Ofrendas:** los santuarios son grietas y conductos para devolver oro soñado. No reproducen altares ni figuras de cultos reales, como pide `docs/lore/folclore.md`.

## 6. Capa de historia

La capa de historia usa **anclas** en vez de la rejilla: estructuras únicas cuya posición sale de la semilla, alrededor de la región de inicio. Están presentes en el modo Historia y desaparecen en el modo Libre. El lore y la historia completos están en `docs/supervivencia/`.

| Ancla | Posición | Contenido |
|---|---|---|
| **El pozo del despertar** | Punto de inicio | Donde despiertan el jugador y todos los Llegados |
| **Pueblo de inicio** | Junto al pozo | Siete, la lampista, y el primer tablero |
| **Macizo del Durmiente Mayor** | A 2–5 km del inicio, según la semilla | La montaña que domina la región. Su nombre se genera con la semilla |
| **Torre del Vigía Mayor** | La cumbre más alta cercana | Radio, mapas y el plano de la Escala |
| **La gran mina de los Antiguos** | Bajo el macizo, de la superficie a ~−600 | La mina más grande del mundo; el Tapón sella su pozo maestro |
| **El Corazón del Durmiente Mayor** | Hacia −600 bajo el macizo | El escenario del Despertar y de la Nana |
| **Base de la Consolidada** | En la falda del macizo | Contratos, la cabecera de la Línea |
| **El tablero de cuarenta y un ganchos** | Escondido; la posición sale de la semilla | Un misterio que no se explica |
| **Fichas de historia** | Repartidas por anclas y estructuras de la región | Cada una aparece una sola vez (`INV-W09`) |

- **La profundidad es la progresión.** Cada capa pide mejor equipo (luz, entibado, bombas, protección frente al calor), igual que un minero de verdad.
- **Ruta garantizada:** desde el inicio hasta el Corazón existe al menos un camino transitable por el pozo maestro y los niveles (`INV-W09`). El jugador puede tomar otros.
- **Sin cinemáticas**, como dice el plan: la historia se cuenta con fichas, cuadernos, carteles, radio y diálogos cortos.
- **Más allá de la región de inicio:** el mundo sigue siendo enorme y procedural. Los otros Durmientes del mundo están bajo otras cordilleras, con su propio sismógrafo local y su propia región en el Sueño.

## 7. El sismógrafo local

El ruido sigue siendo el centro del juego. Cada región de 512 × 512 bloques tiene un **sismógrafo local** que sube con explosiones, maquinaria y lava, y baja con el tiempo, el silencio, las ofrendas y las nanas. En los umbrales hay temblores, derrumbes y grietas que liberan lava o grisú. Reutiliza las reglas de `hush-sim`. El detalle está en `docs/supervivencia/04-criaturas-y-jefes.md`.
