# Modo Historia · 6. Las dimensiones

Los cuatro mundos más allá del Silencio: su lore, cómo se llega, qué reglas cambian, quién vive allí, qué se saca y qué técnica enseñan. Al final, cómo se implementan. Índice en [README.md](README.md).

---

## 1. Resumen

| Dimensión | En una frase | Se llega con… | Rama que enseña | Guardián |
|---|---|---|---|---|
| **El Silencio** | El mundo de 15 M × 15 M bloques, de la superficie a las Raíces | — | Oficio, Luz | El Tapón |
| **El Sueño** (*the Dream*) | Lo que sueñan los Durmientes: un mundo de recuerdos que se reordena | El **Lecho** | Memoria | La Pesadilla |
| **El Hervor** (*the Seethe*) | La sangre de los Durmientes: mares de magma y las ruinas de los Fundidores | Una **Boca de Horno** | Presión | La Caldera y la Línea |
| **El Eco** (*the Echo*) | El sonido hecho materia: el hogar de los Silbadores | La **Campana** | Resonancia | El Coro |
| **El Revés** (*the Underside*) | El otro lado del Párpado, dominio del Insomne | La **Escala** | Párpado | El Insomne |

La Bóveda y las Raíces no son dimensiones: son capas profundas del Silencio (`docs/mundo/01-escala-y-motor.md`).

---

## 2. El Sueño

**Lore:** si los Durmientes sueñan oro y galerías, sueñan también lo que oyen y lo que se tragan. El Sueño es ese lugar: el sueño de todos los Durmientes del mundo a la vez. Cada región del Sueño pertenece a un Durmiente distinto. Para la Escuela del Cobalto es una alucinación compartida por gas y presión; para los Quedos, el sitio donde la montaña guarda lo que no quiere olvidar.

**Se llega:** se construye un **Lecho** (marco de oro soñado y cristal alrededor de una cama) sobre una zona en Respira o más, y se duerme en él. Al despertar en el Silencio se vuelve al Lecho.

**Cómo es:**
- **Geografía de recuerdos:** galerías imposibles, trozos de minas que no existen, pueblos a medio recordar, cielos con soles o lunas que nadie del Silencio ha visto. Los fragmentos salen de los recuerdos de los Llegados que el Durmiente ha oído.
- **Se reordena por épocas.** Cada época (unos 20 minutos de juego) los túneles cambian de sitio. Hay que orientarse con instrumentos y aprender sus patrones.
- **El oro soñado crece a la vista:** es la mejor fuente de oro soñado y la más peligrosa.

**Reglas propias:**
- **Morir en el Sueño es despertar** junto al Lecho, sin lo recogido en el Sueño. Tu equipo no se pierde.
- Lo que construyes **se borra al cambiar de época**, salvo lo que fijes con un cristal de memoria.
- La gravedad es más suave: se salta más alto y se cae más despacio.

**Habitantes:** **Soñados**, figuras neutrales con forma de persona que repiten un recuerdo y dan pistas sobre los Llegados. Ecos fuertes. Desvelos cuando el Insomne se acerca.

**Recuerdos jugables:** algunas fichas perdidas abren un **Recuerdo**: una noche en una mina de otro lugar en la que la montaña despierta. Se juega con **las reglas del modo Turno** (sismógrafo de `0` a `100_000`, cuatro umbrales y 90 s de Despertar) sobre un mapa del generador competitivo (`docs/mapgen/geologia.md`). Es barato porque todo eso ya existe.

**Recursos:** oro soñado, **ensueño** (su aleación), cristal de memoria.

**Guardián:** **la Pesadilla**: el Sueño se reordena sin parar para echarte, y hay que llegar a su centro levantando tu propio mapa.

**Tamaño:** acotado, unos 64 000 × 64 000 bloques y 1 024 de alto, con niebla en los bordes.

---

## 3. El Hervor

**Lore:** debajo de las raíces del mundo corre el calor de los Durmientes: un mar de roca fundida con archipiélagos de basalto. Allí vivieron **los Fundidores**, un pueblo que forjaba con ese calor. Construyeron ciudades-fragua, cadenas colosales y autómatas. Hicieron demasiado ruido, un Durmiente despertó y de ellos solo quedan las ruinas y los autómatas, que siguen trabajando. La Consolidada ha llegado allí antes que tú.

**Se llega:** por una **Boca de Horno**, un respiradero natural en los lagos de lava del manto. Se cruza con traje de hervorita. Más tarde se puede construir una **esclusa térmica**.

**Cómo es:**
- **Un mar de magma** a cota fija, islas de basalto, géiseres de vapor, niebla de ceniza y un resplandor rojo constante. Nunca es de noche.
- **Ruinas de los Fundidores:** fraguas, hornos altos, puentes de cadena, depósitos de planos.
- **Corrientes de magma navegables:** con una **barcaza de hervorita** se recorren grandes distancias deprisa. Es el viaje largo del Hervor.
- **Las estaciones bajas de la Línea** de la Consolidada.

**Reglas propias:**
- **Calor:** sin traje de hervorita, el calor hace daño continuo.
- **El agua hierve.** No se puede llevar suelta: hay que usar condensadores.
- **Presión:** las máquinas de vapor rinden el doble, pero las calderas revientan más fácil.

**Habitantes:** **Forjados** (autómatas que atacan cualquier fuente de ruido y se pueden desactivar o reprogramar con planos), **Rescoldos** (enjambres de brasas que prenden el gas) y **Escorias madre**.

**Recursos:** **hervorita** (piedra que guarda el calor), cobalto rico, azufre, latón de los Fundidores y planos de la Presión.

**Guardianes:** **la Caldera** (la gran caldera viva de los Fundidores: hay que bajarle la presión con sus válvulas, en orden) y **la Línea** (hay que frenarla, no destruirla).

**Tamaño:** unos 128 000 × 128 000 bloques y 512 de alto.

---

## 4. El Eco

**Lore:** todo sonido del Silencio llega a alguna parte, y esa parte es el Eco. En el Eco **el sonido es materia**: un paso deja un escalón, un grito levanta un puente y el silencio lo deshace todo. Allí viven los Silbadores, que oyen cada derrumbe antes de que ocurra y lo avisan al Silencio. Los Quedos aprendieron de ellos a callar y a cantar nanas.

**Se llega:** con una **Campana** de cuarzo resonante afinada a la nota del Durmiente de la zona. Se toca **en silencio perfecto**: si suena otra cosa, no funciona.

**Cómo es:**
- Islas de cristal suspendidas en un vacío de colores. Cada color es una frecuencia.
- **Cuerdas resonantes** que unen las islas y se pueden pulsar para cruzar.
- Las ciudades de los Silbadores: estructuras que cantan.

**Reglas propias, las más raras del juego:**
- **Hacer ruido crea plataformas** de cristal sonoro. Su tamaño depende del ruido y desaparecen al cabo de un tiempo.
- **El silencio las erosiona:** si nadie hace ruido cerca, se deshacen antes.
- El **silbato de notas** (Resonancia) permite «tararear» puentes con precisión.
- Gritar construye mucho, pero atrae **Disonancias**.

**Habitantes:** **Silbadores** (neutrales; se comunican por tonos y enseñan la Resonancia) y **Disonancias** (bestias de sonido que rompen tus plataformas).

**Recursos:** **cuarzo resonante** y **notas**: sonidos atrapados en cristal que sirven de combustible a la Resonancia.

**Guardián:** **el Coro**, los Silbadores desafinados por el Insomne. Se le **afina**, voz a voz.

**Tamaño:** unos 32 000 × 32 000 bloques y 512 de alto.

---

## 5. El Revés

**Lore:** el Párpado tiene otra cara. Al otro lado de la costura, la piedra del cielo es el suelo de un paisaje pálido bajo un cielo negro. Por sus grietas se ve el Silencio entero, muy abajo. Allí vive **el Insomne**, que quiere abrir el Párpado del todo. Nadie había llegado antes, salvo quien levantó la primera Escala, que nunca volvió.

**Se llega:** por la **Escala**, la torre que se construye con el plano de los Vigías hasta la costura del cielo. Es el final del arco V. Después del final, el paso queda abierto.

**Cómo es:**
- **Gravedad invertida** respecto al Silencio: se camina por el dorso del Párpado.
- **Grietas** por las que se ve el Silencio a kilómetros de profundidad. Usa el horizonte lejano de `docs/mundo/01-escala-y-motor.md`.
- **Bosques de lumen:** cristal de luz sólida que crece como un bosque.
- **El ojo:** una luz enorme en el horizonte que nunca se apaga.

**Reglas propias:**
- **Siempre es de día.** La luz fuerte ciega: hacen falta capa de sombra y lámparas oscuras.
- **Lo que se cae por una grieta** vuelve al Silencio y se pierde lejos.
- Las Vigilias ven la luz, no el ruido. Las reglas del sigilo se invierten: aquí hay que esconderse en la sombra.

**Habitantes:** **Vigilias** (seres de luz que ciegan) y Desvelos.

**Recursos:** **lumen**, cristal de luz y la técnica del Párpado.

**Guardián:** **el Insomne** (jefe final, [04-criaturas-y-jefes.md](04-criaturas-y-jefes.md)).

**Después del final:** con **Despertar**, el Revés queda abierto y el Párpado no se cierra más. Con **Arrullar**, se puede volver por la Escala, pero el Insomne ya no está. Con **Quedarse**, tu antiguo personaje vigila desde aquí.

**Tamaño:** unos 64 000 × 64 000 bloques sobre la región del Durmiente Mayor y 512 de alto.

---

## 6. Implementación

| Pieza | Diseño |
|---|---|
| **Identificador** | `DimensionId(u16)`: `0` el Silencio, `1` el Sueño, `2` el Hervor, `3` el Eco, `4` el Revés |
| **Clave de chunk** | `(dimensión, posición)`. Llega en W2 con el planificador de `grisu-world`; no afecta a W0 |
| **Semilla** | `hash(semilla del mundo, dimensión)`. Cada mundo tiene sus propias dimensiones |
| **Perfil de generador** | `data/worldgen/dimensiones/<nombre>.ron`: límites, capas, ruido, biomas, estructuras y reglas |
| **Reglas por dimensión** | Gravedad (signo y fuerza en Q16.16), calor, ciclo de luz, épocas, sonido-materia. Viven en `hush-sim` y `data/supervivencia/dimensiones.ron` |
| **Guardado** | `regiones/<dimensión>/`. El Sueño solo guarda lo fijado con memoria |
| **Pasos** | Estructuras (Lecho, Boca de Horno, Campana, Escala) que mueven la entidad del jugador. El servidor descarga la dimensión que nadie usa |

**Gravedad configurable:** el Revés invierte la gravedad y el Sueño la suaviza. La colisión del hito 1 debería recibir la gravedad como parámetro y no como constante. Cuesta lo mismo ahora y ahorra reescribirla después.

### El Sueño y las épocas

- `chunk = f(semilla, dimensión, posición, época)` con `época = tick / ticks_por_época`. Sigue siendo una función pura.
- Al cambiar de época, los chunks se regeneran y **se descartan las modificaciones** salvo las fijadas con memoria. Los objetos que llevan los jugadores se quedan con ellos.
- El cambio se anuncia (silbidos, temblor) y se aplica en un tick fijo para todos.
- **`INV-W12`:** misma semilla y época dan el mismo Sueño, y ningún jugador queda dentro de roca al cambiar de época: se le recoloca en el aire más cercano.

### El Eco: el sonido hecho materia

- Cada evento de ruido de intensidad N crea un volumen de **bloques de eco** proporcional a N, con un tiempo de vida en ticks.
- Un contador por zona mide el silencio reciente y acelera el borrado.
- Son bloques normales con caducidad y entran en la simulación determinista. El campo de sonido de la IA es la misma fuente de verdad.
- **Invariante propuesta `INV-W13`:** el número de bloques de eco activos está acotado por zona, para que nadie bloquee el servidor gritando.

### Orden recomendado

1. **El Sueño**: estrena dimensiones y épocas, y reutiliza el modo Turno en los Recuerdos.
2. **El Hervor**: un perfil de generador más, con calor y barcazas.
3. **El Eco**: la mecánica más original, así que se prototipa en una escena aislada antes de construir la dimensión.
4. **El Revés**: gravedad invertida y el jefe final.
