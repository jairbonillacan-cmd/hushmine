# Gráficos · 2. Modelos y animación

Personajes, criaturas, máquinas y bloques que se mueven. Índice en [README.md](README.md).

---

## 1. Principios

- **Solo presentación.** La animación vive en el cliente. `hush-sim` decide qué pasa (golpe, caída, muerte) y el cliente lo dibuja. Una animación nunca cambia el resultado.
- **Guiada por la simulación:** el cliente elige el clip según el estado (velocidad, en el suelo, en escalera, herramienta, acción) y los **eventos con tick** (golpe de pico, bloque roto, explosión).
- **Peso y oficio:** los mineros se mueven como gente cansada que carga peso, no como atletas. El golpe de pico tiene que sentirse.
- **Legible en penumbra:** la silueta dice qué hace cada jugador a 30 bloques (picando, cargando, agachado en silencio, poniendo una carga).

## 2. Modelos por piezas

Cada modelo es un árbol de **piezas rígidas**: cajas texturizadas con un pivote. No hay deformación de malla. Encaja con el estilo de vóxeles y es muy barato: todas las piezas de todos los personajes se dibujan con **una llamada instanciada**.

### El minero (esqueleto base)

| Pieza (id) | Hija de | Nota |
|---|---|---|
| `pelvis` | — | Raíz |
| `torso` | `pelvis` | — |
| `head` | `torso` | Casco con su lámpara (enganche `lamp`) |
| `upper_arm_l` / `_r` | `torso` | — |
| `lower_arm_l` / `_r` | `upper_arm_*` | Codos |
| `hand_l` / `_r` | `lower_arm_*` | Enganche `tool` en la derecha |
| `upper_leg_l` / `_r` | `pelvis` | — |
| `lower_leg_l` / `_r` | `upper_leg_*` | Rodillas |
| `pack` | `torso` | Saco de carga: crece con lo que llevas |

Son 14 piezas y 3 enganches (`lamp`, `tool` y `back`). Las criaturas usan el mismo sistema con sus propios árboles.

### Formato (`assets/models/*.ron`)

Los identificadores y los campos van en inglés, como pide `AGENTS.md`; los comentarios, en español.

```ron
// Minero base: 14 piezas rígidas.
Model(
    name: "miner",
    texture: "characters/miner",
    parts: [
        Part(id: "pelvis", parent: None, pivot: (0, 12, 0), cuboid: (min: (-4, 10, -2), max: (4, 14, 2)), uv: (0, 32)),
        Part(id: "torso", parent: Some("pelvis"), pivot: (0, 14, 0), cuboid: (min: (-4, 14, -2), max: (4, 24, 2)), uv: (0, 0)),
        // ...
    ],
    attachments: [
        Attachment(id: "tool", part: "hand_r", offset: (0, -2, -1)),
    ],
    team_tint_mask: Some("team_mask"),
)
```

Las medidas están en píxeles de textura: 32 píxeles = 1 bloque. Las texturas de personaje usan la misma paleta que los bloques ([03-texturas-32.md](03-texturas-32.md)).

**Personalización** (lo pide el lore: mineros de cualquier origen): cabezas, peinados, tonos de piel, ropa y accesorios intercambiables por pieza. La silueta de equipo y de oficio no cambia, para no perder la lectura en el Turno.

**Formatos externos:** si un jefe necesita deformar la malla, se evaluará importar glTF con un ADR propio. No hace falta para empezar.

## 3. Clips y capas

### Clips (`assets/anim/*.ron`)
- Fotogramas clave por pieza: rotación y, si hace falta, posición. Interpolación suave entre claves.
- **Marcadores:** `impact`, `step_l`, `step_r`, `release`. Disparan sonidos y partículas y sirven para sincronizar con la simulación.
- **Variantes de primera persona:** los brazos de la vista de primera persona tienen sus propios clips con el mismo nombre (`dig@1p`).

### Capas, de abajo arriba
1. **Locomoción** (todo el cuerpo): parado, andar, correr, agachado, saltar, caer, escalera, nadar, vagoneta. Mezcla por velocidad.
2. **Acción** (de cintura para arriba): picar, poner, lanzar, escuchar con el geófono, cargar. Se superpone a la locomoción.
3. **Aditivas:** respiración, cansancio con carga, temblor cuando el sismógrafo está alto.
4. **Procedural, siempre al final:**
   - **IK de pies:** un rayo hacia abajo por pie apoya cada pierna en el escalón real. En un mundo de bloques, la diferencia se nota muchísimo.
   - **Mirada:** cabeza y torso siguen hacia dónde apunta el jugador.
   - **Inclinación** al acelerar y girar.
   - **Inercia de muelle** en la lámpara, la mochila y las cuerdas.

**Mezclas:** transiciones de 0,1–0,25 s entre clips. Los cambios bruscos de la simulación (un empujón, por ejemplo) se suavizan sin retrasar la posición real.

## 4. Sincronía con la simulación

- **El golpe cae en su tick.** La duración de picar sale de `data/classes.ron` (ticks por bloque y clase). El clip de picar se escala para que el marcador `impact` coincida con el tick en que la simulación aplica el golpe. Es la misma regla para el bloque que se rompe.
- **Otros jugadores:** el cliente interpola su posición (como ya prevé el plan) y elige el clip con su velocidad interpolada y sus eventos. No se envían datos de animación por la red: solo estado y eventos, que ya existen.
- **Predicción local:** tu propio golpe empieza al pulsar. Si el servidor lo rechaza, el clip se corta con suavidad.

## 5. Lista de animaciones

### Turno (hito 3–4)

| Grupo | Animaciones |
|---|---|
| Locomoción | Parado, andar, correr, agachado y moverse agachado (silencio), saltar, caer, aterrizar, subir y bajar escalera, plataforma de salto |
| Herramientas | Picar (según dureza), empujar con el pico, poner bloque, colocar y detonar explosivo, cizalla, detector, **geófono (de rodillas, oído en el suelo)** |
| Economía | Cargar mineral (mochila que crece), depositar en el banco, **ofrenda en el altar**, devolver una ficha |
| Reacciones | Empujado, aturdido, caída fuerte |
| Muertes | Lava, electricidad, explosión, caída y derrumbe. **Sin sangre ni desmembramientos**: fundido, silueta y polvo |
| Expresión | Saludar, señalar, «silencio» (dedo en los labios), celebrar |

### Modo Historia (añadidos)

| Grupo | Animaciones |
|---|---|
| Supervivencia | Comer, beber, dormir, curarse, cargar a hombros, empujar vagoneta, montar en porteador |
| Oficios | Entibar, sanear el techo, afinar un resonador, topografiar con teodolito, forjar, cultivar |
| Social | Comerciar, colgar la ficha en el tablero, lengua de golpes de los Quedos (golpes con ritmo) |
| NPC | Rutinas de jornada: abrir la tienda, beber en la cantina, barrer, dormir |

### Criaturas y jefes

| Criatura | Técnica |
|---|---|
| Fauna (lanudos, porteadores, acechadores) | Clips por piezas como el minero |
| Selladores, Raicillas | **Procedural:** crecen y reptan por la superficie del vóxel siguiendo la pared |
| Escorias | Rodar con física simple y enfriamiento visual |
| Ecos, Vigilias | Clips del minero con distorsión en el shader |
| **La Línea** | **Cadena procedural:** cada cangilón sigue al anterior por la trayectoria. Cientos de segmentos instanciados |
| **El Tapón**, la Caldera | Deformación por bloques: el jefe es terreno que se mueve, animado bloque a bloque |
| El Coro | Partículas y luz que bailan con el audio |

## 6. Bloques y mundo que se mueven

- **Texturas animadas:** lava, agua, campo de fuerza, bloque eléctrico. Varias capas del array de texturas que el shader recorre con el tiempo (ya hay 4 fotogramas de lava en la guía de arte).
- **Bloques con movimiento:** puertas, compuertas, cangilones en marcha, vagonetas, jaulas de pozo. Se dibujan como entidades instanciadas mientras se mueven y vuelven a ser bloques al parar.
- **Plantas y colgantes:** en el Silencio casi no hay viento, así que **no se mecen**… salvo cuando un Durmiente respira: entonces todo tiembla a la vez. Es un aviso visual del lore.
- **Bloques que caen y escombros:** entidades con física simple; los derrumbes sueltan polvo.

## 7. Rendimiento

- **Instancias:** cada pieza es una instancia con su matriz. 64 personajes × 14 piezas = 896 instancias en una sola llamada.
- **Animación a menos frecuencia de lejos:** a más de 32 bloques se actualiza a 15 Hz y a más de 64, a 7,5 Hz. La IK de pies solo cerca.
- **Presupuesto:** la animación de 64 personajes y 48 criaturas cabe en **0,5 ms de CPU** en el T490s.

## 8. Implementación

| Pieza | Dónde |
|---|---|
| Árbol de piezas, clips, mezcla, capas, IK, muelles | `grisu-anim` (nuevo, solo CPU) |
| Carga de modelos y clips RON, recarga en caliente | `grisu-anim` |
| Dibujo instanciado de piezas | `grisu-render` |
| Elegir el clip según estado y eventos | `hush-client` |
| Hojas de animación para revisar | `hush-cli anim-sheet` ([05-verificacion-y-hitos.md](05-verificacion-y-hitos.md)) |
| Pruebas | Instantáneas de poses con `insta`; que todos los clips existan para todos los estados; que el marcador `impact` caiga en su tick |
