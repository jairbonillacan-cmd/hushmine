# Gráficos · 1. Cámaras

Primera persona, tercera persona, espectador y cámara de fotos. Índice en [README.md](README.md).

---

## 1. Modos de cámara

| Modo | Dónde se permite | Uso |
|---|---|---|
| **Primera persona** | Siempre | La vista por defecto |
| **Tercera persona (detrás)** | Taller, modo Historia, Turno informal | Ver tu personaje, construir en grande, moverse por la superficie |
| **Tercera persona (de frente)** | Taller, modo Historia | Ver tu cara y tu equipo, fotos |
| **Espectador** | Al morir (Turno, mientras esperas), repeticiones, servidores con espectadores | Seguir a un jugador o volar libre |
| **Fotos** | Pausa en el modo Historia y el Taller; repeticiones | Cámara libre, campo de visión, profundidad de campo, ocultar la interfaz |

**Regla competitiva:** en el Turno clasificatorio solo hay primera persona. Cada servidor puede permitir la tercera persona en partidas informales. Mientras esperas a reaparecer, el espectador solo puede seguir a **tu equipo** y nunca muestra menas que tu equipo no conoce (la niebla de mineral de `PLAN_v4.md`).

Una tecla cambia entre primera persona, tercera de espaldas y tercera de frente. Se puede reasignar.

## 2. Primera persona

### Lo que se ve
- **Brazos, herramienta y lámpara** en primer plano (la «vista de brazos»). Los guantes llevan el color del equipo en el Turno.
- **El cuerpo propio no se dibuja** en primera persona, salvo la sombra de tu lámpara en el modo Alto. Así se evitan cámaras que atraviesan el modelo.

### Cómo se dibuja
- **Pase propio** después del mundo, con **su propio campo de visión** (fijo, ~70°). La herramienta no se deforma aunque el jugador ponga el campo de visión del mundo a 110°.
- **Profundidad aparte:** el pase usa un rango de profundidad reservado, así la herramienta nunca atraviesa una pared.
- **Iluminado con la luz del mundo** en la posición del jugador: si entras en una galería oscura, tus manos también se oscurecen.

### Sensación
| Efecto | Cuándo | Se puede desactivar |
|---|---|---|
| Balanceo al andar | Andar y correr | Sí |
| Inercia de la herramienta | Girar rápido | Sí |
| Retroceso del golpe | Cada golpe de pico, sincronizado con el tick | No (es información) |
| Impacto al caer | Al aterrizar | Sí |
| **Temblor sísmico** | Según el sismógrafo y los derrumbes cercanos | Se puede reducir; se sustituye por un indicador en pantalla |
| Respiración | Quieto | Sí |

## 3. Tercera persona

- **Brazo de cámara:** la cámara se coloca detrás y por encima del hombro, a una distancia ajustable (1,5–6 bloques).
- **Choque con el terreno:** cada fotograma se lanza un rayo DDA (el mismo raycast del hito 1) del cuello a la posición deseada. Si choca, la cámara se acerca. Se suaviza la distancia para que no salte.
- **Túneles estrechos:** si el espacio libre no da para la cámara, el juego **pasa solo a primera persona** y vuelve cuando hay sitio. Es una mina: pasará a menudo.
- **Personaje cerca de la cámara:** se difumina con tramado (*dithering*) para no tapar la vista.
- **Cambio de hombro** con una tecla, para mirar por la izquierda o la derecha.
- **Apuntado:** la cruz apunta a donde mira la cámara, pero el raycast de picar sale **del personaje**, que es lo que valida el servidor. Si hay algo en medio, la cruz lo marca.

## 4. Espectador y repeticiones

- **Seguir a un jugador** en primera o tercera persona, o **volar libre**.
- Las repeticiones (`hush-cli replay`, ya en el plan) usan el mismo sistema: se puede mover la cámara con libertad por una partida grabada.
- **Cámara de director** para la crónica posterior y los vídeos de la comunidad: puntos de cámara, trayectorias y cortes simples.

## 5. Cámara de fotos

- Pausa el mundo (solo en el modo Historia, el Taller y las repeticiones).
- Campo de visión, inclinación, profundidad de campo, exposición, filtro de época y ocultar la interfaz.
- Exporta PNG a la resolución que se pida, con el mismo código que las capturas sin ventana.

## 6. Accesibilidad

- **Campo de visión** del mundo entre 60° y 110°.
- **Reducir movimiento:** quita balanceo, inercia, impactos y la mayor parte del temblor.
- **Temblor sísmico visual:** con el temblor apagado aparece un indicador en el borde de la pantalla que dice lo mismo.
- **Sensibilidad** e **inversión** de ejes, y mando con zona muerta ajustable (Steam Deck).

## 7. Implementación

| Pieza | Dónde |
|---|---|
| Cámara de primera persona, frustum | `crates/grisu-render/src/camera.rs` (hito 1, tarea 12) |
| Brazo de cámara de tercera persona, espectador, fotos | `grisu-render` (cámara) + `hush-client` (qué modo toca) |
| Raycast de la cámara | El DDA de `grisu-voxel` (hito 1, tarea 8) |
| Pase de la vista de brazos | `grisu-render`, pase propio |
| Reglas de qué cámara se permite | `hush-sim` (configuración de la partida); el cliente la respeta y el servidor no envía datos que la cámara no debería ver |

**Precisión:** todas las cámaras usan el render relativo a la cámara de `docs/mundo/01-escala-y-motor.md`. A 7 millones de bloques del origen, la vista de brazos y la tercera persona siguen sin temblar.
