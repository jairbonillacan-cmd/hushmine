# Proyecto Veta: sucesor espiritual de Infiniminer

Plan acordado el 22 de octubre de 2026. Documento de referencia para implementar el proyecto desde cero, incluyendo con Opus. Los objetivos de rendimiento son criterios por validar, no resultados ya medidos. No hay implementación previa que conservar en esta carpeta.

## 1. Visión y experiencia de juego

Crear desde cero un **juego de extracción, minería y construcción en primera persona**, con tres equipos de tres jugadores que compiten dentro de una mina procedural.

**La idea central: excavar modifica el campo de juego; construir permite sacar el botín; extraer revela tu posición.**

Conservaremos la minería competitiva y la libertad de construcción del [Infiniminer original](https://www.zachtronics.com/infiniminer/), desarrollando código, identidad visual y contenido propios. «Proyecto Veta» será el nombre provisional.

### El elemento distintivo

Cada equipo construye una **red de extracción** entre su acceso a la mina y los yacimientos: estaciones conectadas mediante cables cortos que requieren un recorrido despejado.

La red permite enviar minerales a la superficie. Perforar túneles directos facilita conectarla, pero expone sus instalaciones. Los rivales pueden cortar cables, atacar estaciones o robar cargamentos pendientes de enviar.

El mineral ya extraído queda asegurado. Las herramientas y desbloqueos permanentes nunca se pierden.

Ejemplo de partida: encuentras una veta valiosa, abres un túnel hasta ella y montas una estación. Al activarla, su ruido delata el sector. Otro equipo corta tu conexión; puedes repararla, defender el cargamento o cargarlo personalmente y buscar una salida.

### Reglas iniciales

- **9 jugadores:** tres equipos de tres. Las salas privadas permiten empezar con menos jugadores y equipos incompletos.
- **20 minutos por partida:** exploración inicial, disputa por yacimientos y evacuación final.
- **Mapa finito y regenerado por partida.** Se conserva la progresión del jugador, no el terreno.
- **Victoria:** mayor valor extraído por el equipo; los empates permanecen como empate.
- **Retirada voluntaria:** cualquier jugador puede evacuar por su acceso o por una salida neutral. Evacuar termina su participación en esa partida.
- **Caída y rescate:** un compañero puede reanimarte durante 30 segundos; al morir, dejas el botín transportado y pasas a espectador.
- **Peligro creciente:** sectores con gas y derrumbes anunciados. Los últimos tres minutos fuerzan la evacuación.
- **Construcción limitada:** consume un recurso común obtenido al excavar y tiene límites por equipo.
- **Sin fuego amigo en la primera versión.**

Los derrumbes serán eventos locales controlados: modifican bloques dentro de un volumen y generan efectos visuales. No se simulará estructuralmente toda la montaña.

### Contenido de la primera versión completa

- Un bioma industrial subterráneo con tres zonas de profundidad.
- Dos plantillas de distribución, variación procedural y posiciones iniciales equilibradas.
- Tres minerales de distinto valor y un material de construcción.
- Tres especialidades: **perforador, ingeniero y explorador**. Todas pueden excavar, construir, defenderse y extraer; cada una aporta una herramienta distintiva.
- Herramientas de minería, escáner, cargas de excavación y reparación.
- Plataformas, escaleras, barricadas, luces, cables y estaciones.
- Dos armas sencillas de proyectiles, con munición limitada. El combate protege o disputa la minería.
- Tutorial individual, campo de pruebas, sala privada, partida y resultados.
- Progresión horizontal: variantes de herramientas con ventajas y desventajas, más cosméticos.

La innovación que se validará es la combinación de **terreno editable, logística vulnerable y extracción voluntaria**. Su atractivo deberá demostrarse jugando.

## 2. Tecnologías elegidas

| Área | Decisión |
|---|---|
| Motor | **Unity 6.3 LTS**, fijando un parche concreto al iniciar |
| Lenguaje | **C#** para lógica, terreno, herramientas y servidor |
| Renderizado | **URP**, con materiales y efectos sencillos |
| Terreno voxel | Sistema propio específico para bloques, integrado en Unity |
| Cálculo intensivo | **Jobs + Burst + Collections + Mathematics** |
| Multijugador | **FishNet**, con transporte **Tugboat** |
| Servidor | Compilación dedicada Linux, sin gráficos |
| Interfaz | uGUI y TextMeshPro |
| Controles | Unity Input System: teclado, ratón y mando |
| Arte | Blender, texturas pequeñas y materiales compartidos |
| Persistencia inicial | Archivos JSON versionados y escritura atómica |
| Pruebas | Unity Test Framework y escenarios de rendimiento |
| Versionado | Git, Git LFS para binarios y dependencias bloqueadas |

**Esta combinación se elige por su equilibrio entre iteración con IA, herramientas disponibles y control del rendimiento.** Unity 6.3 es una rama LTS; Burst permite compilar las partes compatibles de C# a código nativo optimizado y ejecutar trabajos en paralelo. Eso encaja especialmente bien con generar y reconstruir terreno. [Soporte de Unity](https://unity.com/releases/unity-6/support), [documentación de Burst](https://docs.unity3d.com/Packages/com.unity.burst@1.8/manual/index.html).

FishNet proporciona mecanismos de predicción y reconciliación; Tugboat documenta compatibilidad con Windows, Linux y macOS. **La sincronización del terreno seguirá siendo trabajo específico del proyecto.** [Predicción](https://fish-networking.gitbook.io/docs/guides/features/prediction), [Tugboat](https://fish-networking.gitbook.io/docs/fishnet-building-blocks/transports/tugboat).

Godot sería una alternativa válida, pero para este alcance se elige el conjunto C#, Burst y herramientas multijugador de Unity. La arquitectura será convencional, con optimizaciones concentradas en los subsistemas costosos; no adoptaremos ECS completo.

### Plataformas

Primera versión: **Windows x64, Linux x64 y macOS Apple Silicon**, con juego cruzado entre ellas.

Steam Deck será un objetivo de controles y rendimiento, sin prometer certificación. Navegador, móviles y consolas quedan para proyectos de adaptación posteriores.

Las compilaciones multiplataforma se probarán desde el primer prototipo. La validación de macOS necesitará acceso a un Mac real.

## 3. Arquitectura y rendimiento

### Separación de responsabilidades

El proyecto tendrá módulos independientes para:

- **Simulación:** reglas, inventarios, daño, objetivos, extracción y progresión.
- **Mundo voxel:** datos, generación, consultas espaciales, edición y creación de mallas.
- **Red:** comandos, estados, revisiones de terreno y reconexión.
- **Presentación:** cámara, interfaz, animación, sonido y efectos.
- **Contenido:** definiciones de bloques, herramientas, recursos y especialidades.
- **Herramientas:** generación de escenas, pruebas, compilación y mediciones.

Las reglas centrales serán C# comprobable sin depender de escenas. Los ScriptableObjects definirán contenido mediante identificadores estables y se convertirán a datos de ejecución.

### Terreno

- Mapa inicial de **192 × 192 × 96 bloques**, con bloques de un metro.
- Chunks de **16³ bloques**, almacenados en arrays compactos.
- Ningún GameObject ni componente de red por bloque.
- Mallas con eliminación de caras ocultas y unión de caras compatibles —*greedy meshing*—.
- Generación y construcción de mallas mediante Jobs/Burst; publicación en Unity con presupuesto por fotograma.
- Reconstrucción únicamente de chunks modificados y vecinos afectados.
- Trabajos identificados por revisión: una malla antigua nunca sobrescribe cambios nuevos.
- Materiales compartidos, texturas agrupadas y reutilización de buffers.

Unity ofrece acceso a datos de mallas desde su sistema de trabajos, que utilizaremos para esta canalización. [MeshData](https://docs.unity.com/en-us/engine/6000.0/script-reference/unityengine/mesh/meshdata).

Los personajes usarán un controlador cinemático con consultas contra la cuadrícula voxel. Así, una excavación cambia la colisión de inmediato sin esperar a regenerar un collider de malla. Los escombros serán principalmente visuales.

La generación combinará galerías diseñadas y variación procedural. Un validador comprobará accesibilidad, salidas y reparto de recursos; si una semilla falla, se regenerará, con una plantilla válida como respaldo.

### Red y autoridad

- **Servidor autoritativo a 30 ticks por segundo.**
- Predicción del movimiento local y correcciones del servidor.
- Interpolación de jugadores remotos.
- Acciones de minería, construcción, disparo y extracción validadas por alcance, recursos, cadencia y estado.
- Efectos locales inmediatos; los cambios de terreno y recompensas se confirman desde el servidor.
- El cliente recibe snapshots de chunks cercanos y después diferencias fiables, ordenadas y versionadas.
- La transferencia de terreno tendrá un presupuesto separado para no bloquear movimiento y acciones.
- El servidor conserva el terreno completo; cada cliente carga únicamente su entorno necesario.
- Un cambio de terreno que invalide movimiento predicho fuerza una reconciliación; la primera versión no implementará rebobinado completo del mundo.

**Contratos internos principales:**

| Contrato | Responsabilidad |
|---|---|
| `PlayerCommand` | Entrada secuenciada y asociada a un tick |
| `PlayerSnapshot` | Estado autorizado y última entrada procesada |
| `ChunkSnapshot` / `ChunkDelta` | Terreno inicial y modificaciones con revisión |
| `MatchEvent` | Extracción, muerte, evacuación y final de partida |
| `ContentManifest` | Compatibilidad de versiones y contenido |

La conexión rechazará protocolos o catálogos incompatibles. Los comandos repetidos no podrán duplicar objetos ni recompensas.

Se permitirá reconectar durante 60 segundos; el personaje permanecerá vulnerable en el mundo. No habrá incorporación de jugadores nuevos a una partida empezada.

### Alojamiento y progreso

La primera versión ofrecerá:

- Host local para pruebas y LAN.
- Servidor dedicado accesible por dirección y puerto.
- Lista local de servidores favoritos.
- Servidor Linux configurable y empaquetado para despliegue sencillo.
- Sin cuentas centrales ni matchmaking público.

Las partidas por Internet requerirán un servidor accesible o configuración de puertos; no se prometerán conexiones automáticas a través de cualquier router.

El perfil se guardará localmente, con copia de respaldo y versión de formato. Será un modelo de confianza para amigos y comunidades pequeñas, sin economía comercial ni clasificación competitiva. El servidor seguirá controlando todo lo que sucede dentro de la partida.

Una caída del servidor finalizará esa expedición. Los envíos de botín ya confirmados se registrarán inmediatamente, con identificadores que impidan aplicar la misma recompensa dos veces.

### Presupuestos de rendimiento

Son **objetivos de aceptación**, pendientes de medición:

| Escenario | Objetivo |
|---|---|
| Ryzen 5 3600, GTX 1660, 16 GB | 1080p, calidad media, p95 de fotograma ≤16,7 ms |
| Steam Deck | 800p, calidad baja, p95 ≤25 ms |
| Memoria del cliente | ≤2 GB durante la partida de referencia |
| Servidor Linux, 4 núcleos modernos | 9 jugadores, p95 de tick ≤20 ms, RAM ≤1 GB |
| Red estable | Promedio ≤128 KB/s de descarga por cliente, aparte de carga inicial |
| Excavación y explosiones | Sin pausas superiores a 50 ms atribuibles al terreno |

El escenario de referencia incluirá nueve jugadores, excavación simultánea, redes activas y varias explosiones. Se medirán compilaciones ejecutables, no solamente el editor.

La estética industrial usará siluetas claras, colores de equipo, iluminación limitada y sonido direccional. No dependerá de iluminación global dinámica, fluidos complejos ni cientos de luces con sombras.

## 4. Desarrollo por hitos y pruebas

Cada hito terminará con una compilación jugable y criterios verificables.

| Hito | Entrega | Condición para avanzar |
|---|---|---|
| **0. Base reproducible** | Proyecto, dependencias fijadas, escenas mínimas y compilación automatizada | Cliente inicia en las tres plataformas; servidor Linux arranca sin gráficos |
| **1. Minería funcional** | Movimiento, generación, excavación, construcción y chunks | Terreno correcto en límites de chunks y mediciones dentro del presupuesto inicial |
| **2. Riesgo técnico multijugador** | Servidor, predicción, terreno sincronizado y reconexión | Nueve clientes permanecen consistentes con latencia y pérdida simuladas |
| **3. Partida completa** | Equipos, botín, red de extracción, sabotaje, muerte y evacuación | Una partida puede empezar, terminar y conceder resultados correctamente |
| **4. Prueba de identidad** | Especialidades, peligros y equilibrio inicial | Los jugadores utilizan rutas y extracción para tomar decisiones significativas |
| **5. Alfa privada** | Arte coherente, audio, tutorial, progresión, mando y servidor distribuible | Personas externas pueden instalar, conectarse y completar partidas |
| **6. Versión 1.0 comunitaria** | Correcciones, optimización y documentación | Supera pruebas prolongadas, multiplataforma y de recuperación |

**Primera demostración a construir:** dos jugadores excavan hacia una veta, montan una estación, cortan y reparan su conexión y extraen un cargamento. Será la unidad mínima que demuestre la propuesta.

### Pruebas obligatorias

- Generación reproducible, accesibilidad y equilibrio básico de semillas.
- Ediciones simultáneas y explosiones que cruzan chunks.
- Trabajos de malla que terminan fuera de orden.
- Consistencia de terreno tras pérdida de paquetes y reconexión.
- Movimiento con 50–150 ms de latencia y hasta 2 % de pérdida.
- Comandos duplicados, inventarios imposibles y acciones fuera de alcance.
- Cortar, reparar y destruir una red sin crear ni perder cargamentos indebidamente.
- Extracción al mismo tiempo que muerte o final de partida.
- Guardado interrumpido, recuperación de respaldo y migración del perfil.
- Sesiones de dos horas sin crecimiento sostenido de memoria.
- Partidas cruzadas entre Windows, Linux y macOS.

Antes de ampliar contenido se harán al menos tres sesiones con grupos externos. El criterio de diseño será observar decisiones repetidas de **arriesgar, construir, disputar o retirarse**. Si la minería y la red apenas influyen, se corregirá el bucle antes de añadir biomas.

## 5. Método de trabajo con IA y límites

El desarrollo asistido por IA se organizará en tareas pequeñas, cada una con comportamiento esperado, límites y prueba de aceptación.

- Mantener `AGENTS.md`, un documento de diseño y decisiones de arquitectura breves.
- Automatizar preparación de escenas y prefabs con scripts de editor reproducibles.
- Versionar escenas como texto, conservar archivos `.meta` y revisar cambios de assets.
- Una funcionalidad completa por cambio: código, integración, prueba y compilación.
- Verificar por separado los cambios en concurrencia, red, guardado y rendimiento.
- Registrar métricas antes y después de cada optimización.
- Evitar nuevas dependencias cuando las herramientas elegidas ya cubran la necesidad.

**Supuestos fijados:** desarrollo individual con asistencia de IA, presupuesto contenido, comunidades privadas, arte industrial estilizado y prioridad a una versión terminada y ampliable.

Quedan fuera de la primera versión: mundo infinito, supervivencia extensa, fabricación compleja, enemigos con navegación por terreno destructible, vehículos, mercado, pérdida permanente de equipo, matchmaking global, mods ejecutables y servidores masivos.

Después de validar la versión base, el orden de expansión será: **más contratos y biomas → nuevas herramientas de logística → expediciones cooperativas con criaturas → servicios públicos de conexión**. Cada ampliación deberá preservar los presupuestos de rendimiento.

El proyecto se considerará cumplido cuando una comunidad pueda descargarlo, levantar un servidor y jugar expediciones completas, estables y divertidas en las tres plataformas. La optimización será una condición de cada entrega.
