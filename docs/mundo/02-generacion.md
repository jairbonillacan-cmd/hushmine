# Mundo grande · 2. Generación del terreno

Ruido, relieve, biomas, cuevas, fluidos y menas. Las estructuras están en [03-estructuras.md](03-estructuras.md). Índice en [README.md](README.md).

---

## 1. Ruido entero (`grisu-noise`)

No se usan crates de ruido con `f32`: pierden precisión lejos del origen y no garantizan el mismo resultado en todas las plataformas (contracción FMA, funciones trascendentes). El ruido es propio, pequeño y entero.

| Primitiva | Definición |
|---|---|
| **Hash espacial** | `hash3d(x, y, z, semilla)` de `docs/mapgen/geologia.md` (mezcla SplitMix64), con coordenadas `i64` y una **sal** por capa de ruido |
| **Ruido de gradiente 3D y 2D** | Retícula de lado 2ᵏ, así que celda = `x >> k` y fracción = `(x & (2ᵏ − 1)) << (16 − k)` en Q16. Doce gradientes enteros elegidos por hash, producto escalar en `i32` y curva de suavizado quíntica `6t⁵ − 15t⁴ + 10t³` en Q16 con intermedios `i64`. Salida en `[−65 536, 65 536]` |
| **fBm** | Suma de octavas con frecuencias que son potencias de dos (solo desplazamientos) y amplitudes en Q16 desde RON |
| **Ruido en cresta** | `65 536 − |ruido|`: crestas de montaña y túneles largos |
| **Celular (Voronoi) entero** | Un punto por celda colocado por hash; distancia al cuadrado en `i64`. Para regiones de bioma subterráneo, fallas y cámaras |
| **Deformación de dominio** | Se desplaza la entrada con otro ruido antes de muestrear: formas orgánicas sin coste de octavas extra |
| **Rotación entera** | Rotación por una terna pitagórica (3-4-5) antes de muestrear, para que no se note la retícula alineada con los ejes |

**Muestreo grueso:** el ruido 3D de densidad se evalúa en una rejilla de 4 × 8 × 4 bloques (5 × 3 × 5 esquinas por chunk) e interpola de forma trilineal en punto fijo. Es unas 16 veces más barato que evaluar cada bloque y apenas se nota.

**Prueba de lejanía:** la varianza y el histograma del ruido en ±7 499 999 son estadísticamente iguales a los del origen (`INV-W03`).

## 2. Fases del generador

Cada fase es una función pura de la semilla, la versión y la posición. Solo la decoración lee algo más: la fase Base de sus vecinos, que también es pura.

| # | Fase | Ámbito | Qué hace |
|---|---|---|---|
| 1 | **Columna** | 2D, columna de 16 × 16, compartida por todos los chunks verticales | Clima, bioma de superficie, altura del terreno, fallas, ríos |
| 2 | **Densidad** | 3D | Relieve con salientes: piedra, aire o agua según densidad y nivel del mar |
| 3 | **Litología** | 3D | Tipo de roca según profundidad, bioma y fallas; suelo y superficie |
| 4 | **Cuevas** | 3D | Excavadores por ruido y gusanos acotados; simas y cañones |
| 5 | **Fluidos** | 3D | Acuíferos y lagos de lava estables |
| 6 | **Menas** | 3D | Vetas, filones hidrotermales, placeres y chimeneas de kimberlita |
| 7 | **Estructuras** | 3D | Rasterizado de las piezas que cortan este chunk |
| — | *Fin de la fase Base (pura)* | | |
| 8 | **Decoración** | 3D, lee la Base de los 26 vecinos y **solo escribe en su chunk** | Estalactitas, cristales, hongos, árboles, raíces, escombros |
| 9 | **Listo** | | Luz y malla en el cliente; simulación en el servidor |

La regla de la fase 8 es la que permite generar en cualquier orden y en paralelo: un árbol cuyo origen está en el chunk vecino se dibuja también aquí, recortado, porque ambos chunks calculan el mismo árbol.

## 3. Relieve de superficie

Parámetros 2D continuos por columna (todos en Q16):

| Parámetro | Escala típica | Controla |
|---|---|---|
| **Continentalidad** | 2–8 km | Llanura baja, mesetas, macizos |
| **Erosión** | 1–4 km | Picos escarpados frente a colinas suaves |
| **Picos y valles** | 300–1 000 m (ruido en cresta) | Crestas y fondos de valle |
| **Temperatura** | 4–12 km | Bioma |
| **Humedad** | 2–6 km | Bioma |
| **Rareza** | 1–3 km | Variante de bioma (p. ej. badlands con mesas) |

- **Altura base:** curvas por tramos (puntos de control enteros en `data/worldgen/relieve.ron`) que combinan continentalidad, erosión y picos.
- **Densidad 3D cerca de la superficie:** `densidad = (altura − y) · pendiente + ruido3D`. Así salen salientes, arcos y acantilados sin bloques flotantes sueltos.
- **La altura depende de los parámetros, no del bioma.** Dos biomas vecinos comparten parámetros parecidos, así que no hay escalones en las fronteras.
- **Ríos:** ruido en cresta de baja frecuencia, que hunde la altura hasta el nivel del agua. Llevan oro de placer en la grava (sección 8).
- **El macizo del Durmiente Mayor:** una elevación de radio ~3 km y pico ~900 m que se suma a la continentalidad. Su centro sale de la semilla, a 2–5 km del punto de inicio. Es el ancla de la capa de historia (ver [03-estructuras.md](03-estructuras.md)).
- **Plantas indicadoras:** algunas plantas crecen sobre vetas concretas. Por ejemplo, unos arbustos de hojas cobrizas aparecen sobre el cobre. Un prospector que sabe leer la superficie encuentra menas sin picar.

## 4. Biomas de superficie

Se elige el bioma más cercano en el espacio de parámetros (distancia entera al cuadrado) según la tabla de `data/worldgen/biomas.ron`. Son paisajes propios del Silencio, pensados para la minería: cada uno ofrece recursos y dificultades distintas.

| Bioma (propuesta) | Clima | Superficie | Qué aporta |
|---|---|---|---|
| **Cordillera** | Frío, poca erosión | Roca, nieve, pedreras | Vistas, pasos de montaña, bocaminas altas |
| **Bosque de agujas** | Frío y húmedo | Tierra, hojarasca de agujas | Madera para entibar |
| **Altiplano** | Frío y seco, meseta | Pasto ralo, roca | Cobre y estaño someros; rebaños de lanudos |
| **Desierto rojo** | Cálido y seco | Arena roja, costras | Oro en superficie, pocas fuentes de agua |
| **Badlands con mesas** | Cálido, seco, rareza alta | Capas de arcilla de colores | Estratos visibles: ver la geología desde fuera |
| **Valle fluvial** | Templado, húmedo, baja altura | Grava, limo | Oro de placer, pueblos |
| **Tundra y turberas** | Muy frío | Musgo, charcas | Carbón somero (turba) |
| **Lagos y mares interiores** | Continentalidad baja | Agua, playas de grava | Límite natural, rutas |

**Terreno alterado por la minería:** alrededor de los pueblos y castilletes aparecen escombreras y balsas de relaves. No es un bioma: lo pinta la estructura en su zona de influencia.

## 5. Biomas subterráneos

Dependen de la profundidad, de un ruido celular 3D de gran escala y del bioma de la superficie, que «se filtra» hasta unos 100 m.

| Capa | Bioma (propuesta) | Rasgos | Sonido y luz |
|---|---|---|---|
| Corteza | **Cuevas de caliza** | Estalactitas, columnas, coladas, lagos | Goteo; agua que refleja |
| Corteza | **Cuevas húmedas** | Acuíferos, galerías inundadas, barro | Agua corriente |
| Corteza | **Galerías de arcilla** | Paredes blandas, derrumbes fáciles | Crujidos |
| Manto | **Cavernas de basalto** | Prismas, ríos de lava, azufre | Rumor grave, luz naranja |
| Manto | **Geodas y cuarzo** | Cámaras esféricas de cristal | Resonancia; luz que se refracta |
| Manto | **Grutas de grisú** | Aire viciado, bolsas de gas invisibles | Silbidos lejanos (los Silbadores) |
| Manto | **Hongos luminiscentes** | Única luz natural en la oscuridad | Silencio |
| Bóveda | **La Bóveda** | Vacío de kilómetros, pilares de roca, islas suspendidas sobre raíces de cristal | Eco enorme |
| Bóveda | **Vetas soñadas** | Oro que crece de nuevo (sección 10) | Latido del Durmiente |

**El grisú** da nombre al motor y encaja en la supervivencia: bolsas de gas (metadatos de volumen, no bloques) que estallan con llama o explosivos. El prospector las detecta con la lámpara de seguridad. Es una propuesta de mecánica, no una decisión.

## 6. Cuevas

Se combinan varios excavadores. Sus parámetros dependen de la profundidad y del bioma (`data/worldgen/cuevas.ron`).

| Excavador | Método | Resultado |
|---|---|---|
| **Queso** | Umbral sobre fBm 3D | Cavernas grandes con pilares |
| **Espagueti** | `|ruido₁| + |ruido₂| < umbral` (intersección de dos superficies) | Túneles largos y sinuosos que se cruzan |
| **Fideos** | Igual, con umbral fino | Pasadizos estrechos y gateras |
| **Gusanos acotados** | Un gusano por celda de 128³ con origen, dirección y longitud por hash. Cada chunk consulta solo las celdas vecinas que pueden alcanzarlo (longitud máxima ≤ 128) | Galerías con forma de tubo, cámaras en los cruces |
| **Simas** | Gusano casi vertical, poco frecuente | Pozos naturales de 50–300 m; atajos hacia abajo |
| **Cañones** | Elipse alargada en planta con perfil en V | Grietas bajo tierra y en superficie |
| **Cámaras gigantes** | Celular 3D en el manto | Salas de 100–300 m: lagos subterráneos, ríos de lava |

- **Entradas desde la superficie:** los espaguetis y las simas pueden cortar la superficie, con una frecuencia que depende del bioma (más en caliza y cordillera).
- **Las cuevas no cortan el lecho** ni el borde del mundo, y dejan al menos 2 bloques de techo bajo el lecho de los ríos.
- **Calidad visible:** se revisa con cortes verticales PNG (`hush-cli world slice`) y capturas 3D en cada cambio (ver [04-verificacion-e-hitos.md](04-verificacion-e-hitos.md)).

## 7. Acuíferos, ríos y lava

- **Acuíferos:** el espacio se divide en celdas de 16 × 12 × 16. Cada una tiene un nivel de agua decidido por hash y por la altura de la superficie. El aire de una cueva por debajo del nivel de su celda se llena de agua. Entre celdas con niveles distintos se deja una pared sólida. Resultado: lagos subterráneos a varias alturas que **no se desbordan al cargar** (`INV-W05`).
- **Nivel freático:** las galerías profundas de las minas antiguas quedan inundadas por debajo del acuífero local. Hay que bombear para bajar: un objetivo de supervivencia.
- **Lava:** lagos en las cámaras gigantes del manto por debajo de −450 y ríos de lava en el basalto. Se aplica la misma regla de estabilidad y un margen de 6 bloques de roca respecto a cualquier acuífero.
- **Lava en el modo Turno:** sigue rigiéndose por `docs/mapgen/geologia.md`. No hay cambios.

## 8. Litología y menas

### Rocas

| Profundidad | Rocas dominantes | Notas |
|---|---|---|
| Superficie | Tierra, arena, grava, arcilla | Según el bioma |
| Corteza | Caliza, arenisca, pizarra | Estratos ondulados con la deformación de dominio |
| Manto | Granito, basalto, roca dura | Más dureza con la profundidad |
| Bóveda y Raíces | Basalto, cristal, roca del Durmiente | Por definir con arte y lore |

- **Fallas:** planos por macrocelda de 512 bloques. Desplazan los estratos unos bloques y concentran mineralización: un prospector que sabe leer la roca encuentra más.
- **Chimeneas de kimberlita:** conos verticales estrechos que suben desde el manto. Ahí, y solo ahí, aparece el diamante. Es geología real y una pista visible.

### Menas (propuesta para supervivencia; nombres finales en diseño)

| Mena | Dónde | Forma |
|---|---|---|
| Carbón | Corteza y turberas | Estratos horizontales |
| Cobre | Corteza, altiplano | Vetas y manchas someras |
| Estaño, hierro | Corteza | Vetas (elipsoides de `geologia.md`) |
| Wolframio (tungsteno) | Manto, en granito | Vetas finas; endurece el acero negro |
| Plata y plomo | Corteza profunda | Filones en fallas |
| Cobalto | Manto | Nódulos; material de nivel 5 |
| **Oro** | En todas partes, cada vez más rico abajo | Filones de cuarzo en fallas, placeres en ríos, vetas en el manto |
| **Diamante** | Manto | Solo en chimeneas de kimberlita |
| Azufre | Junto a la lava | Costras |
| Cuarzo y cristal | Geodas | Revestimiento |

La distribución por profundidad (curvas por tramos) vive en `data/worldgen/menas.ron`. `INV-W07` mide la distribución real en miles de chunks de muestra y la compara con esas curvas.

## 9. Decoración

Elementos pequeños (radio ≤ 8 bloques): estalactitas, estalagmitas, columnas, cristales, hongos, raíces colgantes, árboles, rocas sueltas, escombro y huesos de entibado viejo. Cada chunk calcula sus orígenes por hash, dibuja también los de los 26 vecinos que lo alcanzan y recorta.

## 10. El Durmiente sueña oro

En el modo Turno, la montaña desplaza túneles y hace brotar vetas. En la supervivencia, la misma idea mantiene viva la economía de un mundo enorme:

- En las **Vetas soñadas** (la Bóveda) y en las zonas marcadas de la corteza del Durmiente Mayor, el oro **se regenera** por épocas. Una época tiene una duración fija de días de juego. La posición de cada veta nueva se decide con `hash(semilla, época, celda)`, así que es determinista y no hay que guardarla.
- Lo que el jugador excavó en esa celda se respeta: solo cambian los bloques intactos.
- Es una **propuesta de diseño** que habrá que validar con lore y balance.
