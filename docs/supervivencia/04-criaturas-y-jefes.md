# Modo Historia · 4. Criaturas, jefes y combate

Qué vive en el Silencio y en sus dimensiones, cómo se pelea y cómo son los guardianes. Índice en [README.md](README.md).

---

## 1. Reglas de diseño

1. **Bestiario 100 % original:** nada copiado de mitologías reales ni de otros juegos.
2. **Doble lectura:** cada criatura extraña admite una explicación técnica y otra mítica.
3. **El ruido atrae y la luz decide.** Las criaturas oyen por la cuadrícula: el sonido viaja por el aire y los túneles y la roca lo apaga. La luz espanta a unas y atrae a otras.
4. **Ni combate a muerte contra humanos ni armas de fuego.** A la Consolidada se le gana con sabotaje, economía, reputación y terreno.
5. **Un guardián es un problema que se resuelve con un oficio** (escuchar, ventilar, frenar, afinar, orientarse), no una bolsa de vida.

## 2. Ruido, luz y sismógrafo local

| Acción | Ruido (provisional) |
|---|---|
| Andar / correr | 1 / 4 |
| Picar tierra / roca / roca dura | 3 / 6 / 10 |
| Martillo de vapor | 20 |
| Vagoneta / fuelle / caldera | 8 / 12 / 15 |
| Golpe en combate | 6 |
| Explosión | 60 |

- **Propagación:** BFS entero por las celdas de aire, con atenuación por distancia; a través de la roca, mucho mayor. Se calcula en `hush-sim` para que la IA sea determinista.
- **Amortiguadores:** la tecnología de la Resonancia ([05-tecnologias.md](05-tecnologias.md)) reduce el ruido de máquinas y bases. Sin ella, una fábrica grande despierta a la zona.
- **Sismógrafo local:** cada región de 512 × 512 bloques tiene su medidor de `0` a `100_000`. Sube con el ruido y baja con el tiempo, el silencio, las ofrendas y las nanas. Usa los umbrales del modo Turno:

| Umbral | En la zona |
|---|---|
| **Revuelo** (25 %) | Silbidos, polvo, aparecen Ecos |
| **Respira** (50 %) | Los Selladores se activan, el Filón errante se mueve |
| **Inquietud** (75 %) | Aire malo, Escorias, derrumbes |
| **Despertar** (100 %) | Evento de 90 s: derrumbes graves, lava y guardianes cercanos que despiertan. Después, el medidor baja a 50 % y **brota oro soñado** |

Hacer ruido es peligroso, pero cada Despertar enriquece la zona. Es el ciclo del modo Turno en un mundo abierto.

## 3. Fauna del Silencio

| Animal | Dónde | Comportamiento | Da |
|---|---|---|---|
| **Lanudos** (*Woolies*) | Praderas y mesetas | Rebaños que se quedan quietos al oír ruido y huyen si sigue. Se crían | Lana, carne, cuero |
| **Porteadores** (*Haulers*) | Valles | Bestias de carga lentas y fuertes; se doman | Montura y carga |
| **Piadores** (*Peepers*) | Entradas de cueva | Pájaros pequeños que se llevan en jaula: **se desmayan con el gas antes que tú** | Aviso de gas |
| **Aleteos** (*Flutters*) | Cuevas | Bandadas que salen volando con el ruido, y **la bandada te delata** | Guano (salitre) |
| **Ratas de veta** (*Vein rats*) | Minas | Roban comida; **huyen antes de un derrumbe** | Aviso |
| **Cavadores** (*Burrowers*) | Tierra blanda | Animales grandes que excavan siguiendo las vibraciones | Cuero duro |
| **Acechadores** (*Stalkers*) | Superficie, con el Párpado cerrado | Depredadores que cazan por el oído y huyen de la luz fuerte | Colmillos, pieles |

## 4. Criaturas de la montaña

| Criatura | Dónde | Comportamiento | Contra ella | Lectura técnica | Lectura mítica |
|---|---|---|---|---|---|
| **Ecos** (*Echoes*) | Galerías con el medidor en Revuelo o más | Siluetas en la niebla que **imitan la voz de tus compañeros** para llevarte a galerías que se hunden | Luz fuerte; la radio confirma dónde está de verdad tu compañero | Infrasonido y gases que provocan alucinaciones | La montaña aprende a hablar con las voces que oye |
| **Selladores** (*Sealers*) | Corteza profunda y manto | Costras de cuarzo que reptan y **vuelven a cerrar las galerías** | Resonadores, agua fría, picos duros | Precipitación mineral acelerada | El Durmiente cierra sus heridas. A veces llevan **una ficha perdida dentro** |
| **Aire malo** (*Bad air*) | Grutas de gas, minas sin ventilar | Nubes que derivan hacia el calor: gas explosivo, asfixiante o venenoso | Ventilación, lámparas sin llama, prender a distancia | Gases de mina | El aliento del Durmiente |
| **Escorias** (*Slag*) | Manto y Hervor | Basalto con el núcleo fundido que rueda hacia las vibraciones | Agua: se enfrían y se vuelven roca minable | Lava empujada por la presión | La sangre del Durmiente |
| **Filón errante** (*Wandering lode*) | Corteza y manto en Respira | Una veta de oro que **huye por la roca** del ruido. No ataca | Silencio, geófono y cortarle el paso | Inyección mineral móvil | Oro soñado que aún no ha decidido dónde quedarse |
| **Polillas de mica** (*Mica moths*) | La Bóveda | Enjambres que **se comen la luz** | Lámparas tapadas; bengalas como señuelo | Insectos que se alimentan de luz | Lo que vive en el sueño |
| **Raicillas** (*Rootlets*) | Las Raíces | Filamentos de cristal que atrapan al oír ruido | Silencio absoluto, cizalla | Cristalización ramificada | Los nervios del Durmiente |
| **Desvelos** (*Wakers*) | En cualquier sitio, más cuanto más crece el Insomne | **Hacen ruido a propósito:** chillan, golpean y derriban para despertar lo que haya cerca | Callarlos rápido: trampas, cámaras sordas, luz | — | Los mensajeros del Insomne |

**Presencias que no son enemigos:**
- **Los Silbadores** silban antes de cada derrumbe y nunca atacan.
- **Los Guías:** siluetas con lámpara que señalan salidas durante un Despertar. Nunca hablan y nadie sabe quiénes son.

## 5. Criaturas de las dimensiones

| Dimensión | Criaturas | Notas |
|---|---|---|
| **El Sueño** | **Soñados** (neutrales; recuerdos con forma de persona), Ecos fuertes, Desvelos | Morir en el Sueño es despertar ([06-dimensiones.md](06-dimensiones.md)) |
| **El Hervor** | **Forjados** (autómatas de los Fundidores que atacan cualquier fuente de ruido), **Rescoldos** (enjambres de brasas que prenden el gas), Escorias madre | Calor constante |
| **El Eco** | Silbadores (neutrales), **Disonancias** (bestias de sonido que rompen tus plataformas) | El sonido es materia |
| **El Revés** | **Vigilias** (seres de luz que ciegan), Desvelos | Dominio del Insomne |

**Aparición:** depende del bioma, la profundidad, la luz, el sismógrafo local, el ruido reciente y la fuerza del Insomne, **no de temporizadores al azar**. Hay un tope de criaturas activas por zona. Si se respeta el silencio, la montaña apenas reacciona.

## 6. Combate

- **Vida** entera (100) y tipos de daño: golpe, caída, aplastamiento, quemadura, gas, electricidad y frío.
- **Las armas son herramientas** ([05-tecnologias.md](05-tecnologias.md)). Cada una sirve contra unas criaturas y no contra otras.
- **El terreno es el arma principal:** derrumbar un techo bien entibado, inundar una galería, cerrar una puerta de ventilación o electrificar un suelo.
- **Cada golpe hace ruido.** Pelear mucho sube el sismógrafo. Huir y esconderse suele ser lo mejor.
- **Dificultades:** Tranquila (pocas criaturas, sin Despertares locales), Normal, Dura y Extrema (una sola vida).

## 7. Guardianes

### De la historia

| Guardián | Dónde | Qué es | Cómo se le vence | Oficio | Recompensa |
|---|---|---|---|---|---|
| **El Tapón** (*The Plug*) | Arco I · pozo maestro de los Antiguos | Roca viva que sella el pozo y vuelve a cerrar lo que se cava | Encontrar con el geófono sus **costuras frías**, barrenar y volar al ritmo de su pulso, entibando para no quedar aplastado | Escuchar y volar | Acceso al fondo de la gran mina |
| **La Pesadilla** (*The Nightmare*) | Arco II · el Sueño | El Sueño reordenándose para echarte | Llegar al centro mientras el plano cambia, levantando tu propio mapa | Orientarse | Nota del Sueño |
| **La Caldera** (*The Boiler*) | Arco III · el Hervor | La gran caldera de los Fundidores, viva y a punto de reventar | Abrir y cerrar sus válvulas en el orden justo para bajarle la presión mientras sus Forjados te buscan | Presión | Nota del Hervor |
| **La Línea** (*The Line*) | Arco III · del Hervor al Corazón | La cadena de cangilones colosal de la Consolidada. Caza el ruido | **No se destruye: se frena.** Hay que bloquear sus estaciones motrices en orden y en silencio | Mecánica | La Línea parada (y una pieza para la Nana) |
| **El Coro** (*The Choir*) | Arco IV · el Eco | Los Silbadores desafinados por los Desvelos | **Afinarlo:** encontrar la nota de cada voz con resonadores mientras las Disonancias rompen el suelo | Resonar | Nota del Eco |
| **El Despertar del Durmiente Mayor** | Arco V · el Corazón | No es un jefe: es el evento de mundo | Sobrevivir, montar la Nana y llegar a la Escala | Todo | El camino al Párpado |
| **El Insomne** (*The Sleepless*) | Arco V · el Revés | Lo que nunca duerme | **Jefe final.** Por fases: apagar sus Vigilias con sombra, callar a sus Desvelos y tocar las tres Notas mientras intenta mantener abierto el Párpado | Todos | La decisión final |

### Opcionales y procedurales

- **El Pulmón** (*The Lung*): una caverna de gas que respira, en las grutas del manto. Al inspirar arrastra y aviva las llamas; al espirar inunda de gas. Se le vence con ventilación.
- **La Reina de Cuarzo** (*The Quartz Queen*): la colonia madre de los Selladores, en una geoda gigante de la Bóveda. Se rompe con resonadores afinados a su frecuencia.
- **Variantes por semilla:** Tapones menores en minas selladas, Pulmones en grutas de gas, colmenas de Selladores en geodas, Escorias madre en lagos de lava. Sus guaridas son estructuras generadas (`docs/mundo/03-estructuras.md`).

## 8. Implementación

| Pieza | Dónde | Nota |
|---|---|---|
| IA por estados (patrulla, oír, investigar, huir, atacar) | `hush-sim` | Enteros; orden de actualización fijo por id de entidad |
| Búsqueda de caminos | `hush-sim` | A* sobre la cuadrícula con presupuesto de nodos por tick |
| Oído | `hush-sim` | Campo de sonido entero por zona activa, recalculado por eventos |
| Datos | `data/supervivencia/criaturas.ron`, `jefes.ron` | Vida, daño, oído, luz, aparición, botín |
| Escenas de captura | `tests/scenes/supervivencia/` | Una por criatura y una por guarida |
| Pruebas | `proptest` | Determinismo de la IA, tope de criaturas, ninguna criatura atraviesa sólidos |

**Presupuesto (por validar):** como máximo 48 criaturas activas por jugador. La IA de un servidor con 16 jugadores tiene que caber en 3 ms del tick de 10 ms de `docs/mundo/01-escala-y-motor.md`.
