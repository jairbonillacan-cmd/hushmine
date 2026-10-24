# Modo Historia · 3. Pueblos, NPC y comercio

Dónde se vende lo que se saca, qué se compra y por qué volver a la superficie. Índice en [README.md](README.md). La generación de los asentamientos está en `docs/mundo/03-estructuras.md`.

---

## 1. Tipos de asentamiento

| Asentamiento | Dónde | Quién vive | Qué ofrece |
|---|---|---|---|
| **Pueblo de inicio** | Junto al pozo donde despiertas | Llegados; Siete, la lampista | Todos los oficios básicos y el primer tablero |
| **Pueblos de Llegados** | Valles, mesetas y orillas; más cerca unos de otros en la región del Durmiente Mayor | Llegados | Tiendas, ensayador, encargos. Crecen |
| **Aldeas de los Quedos** | Cuevas grandes de la corteza y el manto | Quedos | Trueque silencioso, tecnología de la Resonancia, nanas |
| **Campamentos de la Consolidada** | Junto a minas grandes y bocas de horno | Personal de la Consolidada | Vales, contratos bien pagados, maquinaria |
| **Torres de los Vigías** | Cumbres | Un Vigía | Mapas, avisos, la red de radio |
| **Lampisterías aisladas** | Bocaminas y cruces | Un lampista | Reaparición, lámparas, guardarropa |
| **Pueblos fantasma** | Todo el mundo | Nadie | Ruinas de Llegados que se fueron, o que bajaron y no volvieron |

**Gradiente de frontera:** cuanto más lejos de la región inicial, menos pueblos vivos, más peligro y mejores menas.

## 2. La lampistería

Es el centro de cada pueblo de Llegados:

- **Tu ficha:** al bajar la descuelgas y tu gancho queda vacío; al subir la cuelgas. En multijugador, el tablero dice **quién sigue abajo**.
- **Reaparición** en la última lampistería donde colgaste la ficha.
- **Rescate:** si mueres abajo, tu carga queda en una bolsa donde caíste. Puedes ir a buscarla o pagar a la cuadrilla de rescate, que tarda y cobra según la profundidad.
- **Lámparas:** combustible, cargas y reparaciones.

## 3. Oficios de los NPC

| Oficio | Compra | Vende y ofrece |
|---|---|---|
| **Ensayador** | Todo el mineral | Tasación; distingue el oro soñado |
| **Almacenero** | Lana, madera, comida | Comida, cuerda, combustible de lámpara, sacos |
| **Herrero** | Metales, carbón | Herramientas; reparación y mejora |
| **Lampista** | Cristales y hongos luminosos | Lámparas, piadores en jaula; reaparición |
| **Entibador** | Madera | Puntales, marcos, planos de entibado |
| **Polvorinero** | Salitre, azufre | Explosivos. **Pide licencia** (reputación o examen) |
| **Mecánico** | Cobre, chatarra | Vagonetas, raíles, bombas, fuelles, máquinas |
| **Topógrafo** | Mapas hechos por el jugador | Instrumentos, mapas, planos de ruinas |
| **Curandero** | Hongos, resinas | Vendas, antídotos de gas |
| **Cantinero** | — | Comida caliente, rumores, encargos, ayudantes |
| **Oficina de concesiones** | — | Registro de terrenos (sección 7) |
| **Maestros de Escuela** | — | Cobre o Cobalto: enseñan técnicas de oficio a cambio de reputación |

**Ayudantes:** un peón para cargar, un entibador que asegura tu galería o un escucha con geófono. Cobran por jornada y se niegan a bajar si el sismógrafo está alto.

**Horarios:** los NPC tienen jornada según el Párpado: tienda con luz, cantina al cerrarse, casa de noche. La lampistería nunca cierra.

### Los Quedos

- **Trueque, no dinero:** dan objetos por objetos y valoran lo que cuesta hacer, no lo que brilla.
- **Etiqueta del silencio:** en una aldea no se corre, no se pica y no se habla alto. Un ruido y se cierran las puertas. Con buena reputación, enseñan la lengua de golpes y comercian a través de ella.
- **Ofrecen** lo que nadie más tiene: amortiguadores, calzado silencioso, cristal cultivado, nanas pequeñas para calmar una zona.

## 4. Dinero y precios

- **El grano:** la moneda de los Llegados es oro ensayado por peso. El ensayador convierte mineral en granos.
- **Los vales de la Consolidada:** pagan más, pero solo valen en sus economatos. Es la trampa de los pueblos de compañía convertida en decisión de juego.
- **Oferta y demanda:** cada venta baja el precio de ese mineral en ese pueblo, y se recupera con las jornadas. Todo son enteros y deterministas. En un servidor, los jugadores comparten el mercado de cada pueblo.
- **Diferencias entre pueblos:** el bioma, la distancia y la facción cambian los precios. La madera es barata en el bosque y cara en el desierto. Llevar mercancía de un sitio a otro es una forma de jugar.

### El dilema del oro soñado

| Destino | Valor |
|---|---|
| **Venderlo** | El ensayador paga ~70 % del oro normal |
| **Ofrendarlo** | Calma al Durmiente de la zona tanto como el oro normal o más |
| **Venderlo a la Consolidada** | Lo paga completo, en vales. ¿Para qué lo quiere? (pista del arco III) |

Las cifras son provisionales y se fijan en `data/supervivencia/comercio.ron`.

## 5. Encargos

Los pueblos piden lo que les falta. Se generan con las necesidades del pueblo y la semilla de la jornada. Los de la historia son fijos.

| Tipo | Ejemplo | Oficio |
|---|---|---|
| **Entrega** | 40 puntales al pueblo de la meseta | Logística |
| **Topografía** | Levanta el plano de la mina de los Antiguos del valle norte | Mapas |
| **Desagüe** | Bombea un nivel inundado | Máquinas |
| **Reapertura** | Despeja y entiba un pozo hundido | Entibado |
| **Rescate** | Una ficha lleva dos jornadas sin volver | Exploración |
| **Escolta** | Acompaña a la Medidora a medir una sima | Silencio |
| **Ofrenda** | Lleva oro soñado a la grieta del este | Escuela del Cobre |
| **Contrato de la Consolidada** | Vuela este pilar de seguridad, y no preguntes | Moral: paga mucho y tiene consecuencias |
| **Escucha** | Graba con el geófono el silbido de una gruta | Vigías; da pistas de la historia |
| **Mensaje a los Quedos** | Lleva una frase en lengua de golpes | Reputación con los Quedos |

## 6. Reputación

Cada facción y cada pueblo llevan una reputación entera de −100 a +100.

- **Sube** con encargos, comercio justo, ofrendas y silencio.
- **Baja** con sabotajes, robos en concesiones ajenas o despertando a un Durmiente cerca del pueblo.
- **Efectos:** precios, licencias, planos, ayudantes, técnicas de las Escuelas y diálogos de historia. Ayudar a la Consolidada cierra puertas con los Quedos, y al revés.

## 7. Concesiones (multijugador)

- Se registran en la oficina de un pueblo. Cubren columnas de 16 × 16 chunks hasta la profundidad que se pague.
- En tu concesión, los demás no pueden picar ni construir sin permiso: es la protección de los servidores persistentes.
- Tienen un canon por jornadas. Si no se paga, caducan.

## 8. Fundar un pueblo

Los jugadores pueden atraer Llegados a su asentamiento:

1. **Lampistería con tablero:** obligatoria. Da reaparición.
2. **Antena** conectada a la red de los Vigías: el asentamiento sale en el mapa de radio.
3. **Una casa con luz, puerta y cama** por cada NPC.
4. **Suministro:** cada oficio pide algo (madera para el entibador, carbón para el herrero).

Cuando se cumplen las condiciones, llega por un camino un Llegado con el oficio que falta y **cuelga su ficha**. El pueblo crece a medida que se le abastece. Es la manera de dejar huella en un mundo de 15 millones de bloques.

## 9. Implementación

- NPC, tiendas, precios, encargos y reputación viven en `hush-sim`: enteros, orden fijo y guardado en `rusqlite`.
- Tablas en `data/supervivencia/comercio.ron` y `encargos.ron`.
- Diálogos cortos en Fluent, con el inglés como idioma fuente.
- **Invariante de economía:** el dinero solo se crea al vender a un NPC y solo se destruye al comprarle, al pagar cánones o al ofrendar. Es la conservación del oro del modo Turno (`docs/design/invariantes.md`) aplicada a este modo.
