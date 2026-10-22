# Proyecto Veta v2: sucesor espiritual de Infiniminer

Revisión del 22 de octubre de 2026 sobre `PLAN.md` (v1). Este documento es autónomo: sustituye a la v1 como referencia de implementación, incluida la asistida por Opus. Los objetivos de rendimiento siguen siendo criterios por validar, no resultados medidos.

---

## 0. Valoración de la v1 y resumen de cambios

### Lo que la v1 hace bien (y se conserva)

- **Alcance acotado y explícito**: mapa finito, 20 minutos, sin mundo infinito, sin ECS completo, lista clara de exclusiones.
- **Presupuestos de rendimiento medibles** sobre compilaciones reales, no sobre el editor.
- **Hitos guiados por riesgo**, con una primera demostración mínima muy bien elegida.
- **Colisión contra la cuadrícula voxel** en vez de colliders de malla. Es la mejor decisión técnica del plan y, como se verá, abre la puerta a un servidor sin Unity.
- **Lista de pruebas obligatorias** que ya contempla los casos difíciles: trabajos de malla fuera de orden, duplicados y extracción simultánea con muerte.
- **Método de trabajo con IA** basado en tareas pequeñas con prueba de aceptación.

### Lo que falla o falta

| # | Problema en v1 | Consecuencia | Cambio en v2 |
|---|---|---|---|
| 1 | **No hay historia, ambientación ni identidad** | El juego no se distingue en una tienda llena de juegos de minería; arte y sonido sin dirección | Sección 2 completa: el Cerro Ánima, el Pulso y la resonita |
| 2 | Morir = espectador el resto de la partida | En una partida de 20 minutos, morir en el minuto 5 son 15 minutos mirando | Reaparición por oleadas; la muerte definitiva solo ocurre durante el Pulso final |
| 3 | La «retirada voluntaria» termina tu participación, pero se gana por equipo | Evacuar antes de tiempo apenas tiene sentido; la mecánica queda huérfana | Las salidas son puntos de entrega reutilizables; evacuar solo es decisivo en el Pulso |
| 4 | Armas de proyectiles sin compensación de latencia | Con 50–150 ms, los disparos instantáneos se sienten injustos; el *rollback* de FishNet es de pago (Pro) | Herramientas-arma con proyectiles de viaje y cuerpo a cuerpo; no requieren rebobinado |
| 5 | Exige 9 jugadores (3×3) | Difícil de reunir para comunidades pequeñas; el juego muere si no se llenan salas | De 2 a 3 equipos de 1–4 jugadores, mapas escalados y un modo cooperativo **Contrato** en la v1 |
| 6 | Solo servidor dedicado o puertos abiertos | Fricción enorme para jugar con amigos | Partidas alojadas por un jugador mediante el relé de Steam, además de servidor dedicado |
| 7 | *Streaming* de chunks por cercanía | Complejidad innecesaria: el mapa completo ocupa unos 3,5 MB sin comprimir | Mapa completo comprimido al unirse y después solo diferencias |
| 8 | Sin solución para el *x-ray* | Si el cliente tiene el mapa, sabe dónde está todo el mineral; rompe la exploración | «Niebla de mineral»: el servidor oculta los minerales no expuestos ni escaneados |
| 9 | Servidor = Unity sin gráficos y FishNet | Mayor consumo de memoria, arranque lento y pruebas de red que necesitan Unity | Núcleo C# sin Unity, servidor .NET puro y LiteNetLib, con FishNet como alternativa evaluada |
| 10 | Excavación confirmada solo por el servidor | Retraso perceptible al picar; Infiniminer se sentía instantáneo | Predicción local de la edición con reversión si el servidor la rechaza |
| 11 | El progreso se guarda en el cliente, en JSON | Cualquiera puede editar sus desbloqueos que afectan al juego | Los desbloqueos de juego viven en el servidor de la comunidad; los cosméticos, en local |
| 12 | El sonido es mecánica central («su ruido delata el sector»), pero no está especificado | Sin propagación por túneles, el ruido no transmite información útil | Propagación acústica sobre la cuadrícula voxel |
| 13 | Faltan idiomas, accesibilidad, firma en macOS y equipos de gama baja | Problemas de lanzamiento y de alcance | Español e inglés desde el hito 0, accesibilidad sonora y de color, notarización y un objetivo con iGPU |
| 14 | No contempla la hoja de ruta de Unity | La 6.3 LTS termina su soporte en diciembre de 2027, en torno al lanzamiento | Ventana de migración planificada y código preparado para CoreCLR |

---

## 1. Visión y experiencia de juego

Juego de **minería, construcción y extracción en primera persona** donde cuadrillas rivales compiten dentro de una mina procedural que se regenera en cada partida.

**La idea central: excavar modifica el campo de juego; construir permite sacar el botín; extraer revela tu posición.**

### Tres pilares

1. **El terreno es el arma.** Toda ventaja táctica sale de picar, tapar, volar o fortificar.
2. **La logística es el objetivo.** No gana quien más mata, sino quien mantiene viva su línea de extracción.
3. **La luz y el ruido son información.** Tu lámpara te deja ver, pero también te delata. Tus máquinas trabajan, pero también se oyen. Cada recurso valioso tiene un coste de exposición.

### La red de extracción (definición concreta)

| Elemento | Función | Valores iniciales para ajustar |
|---|---|---|
| **Torno de bocamina** | Punto seguro del equipo, en su acceso. El mineral que llega aquí queda asegurado | Uno por equipo, indestructible |
| **Estación de carga** | Se monta junto a una veta y llena cangilones con el mineral depositado | Ciclo ruidoso de 8 s por cangilón |
| **Poste** | Relé barato que permite doblar esquinas | Límite por equipo; se destruye con 3 golpes |
| **Cable** | Une dos nodos en línea recta por bloques no sólidos (comprobación DDA 3D) | Largo máximo de 10 bloques; se corta si un bloque lo atraviesa |

Los cangilones **viajan visiblemente** por el cable a unos 4 bloques/s. Si alguien corta un cable, los cangilones en tránsito caen como cajas que **cualquiera puede recoger**: sabotear también es robar. Perforar túneles rectos facilita tender cable, pero también deja expuesta la línea.

### Reglas de partida

- **Jugadores:** 2 o 3 equipos de 1 a 4 jugadores. La configuración de referencia para equilibrio y rendimiento es 3×3. Las salas privadas permiten equipos incompletos.
- **Mapas escalados por plantilla:** S (2 equipos, ≤4 jugadores) 128×128×80; M 160×160×96; L (3 equipos, 9–12 jugadores) 192×192×96.
- **Duración de 15 a 20 minutos (configurable), en tres fases:**
  1. **Prospección** (0–4 min): exploración y primeras estaciones.
  2. **Disputa** (4 min–final −3 min): sabotaje, robo y fortificación.
  3. **Pulso** (últimos 3 min): el cerro empieza a moverse. Aumentan el gas y los derrumbes; la red de extracción se apaga en el último minuto; toda la extracción restante se hace cargando a mano.
- **Victoria:** gana el equipo con mayor valor asegurado; los empates se mantienen.
- **Salidas:** el torno propio y las salidas neutrales son **puntos de entrega** reutilizables: entregas lo que cargas y puedes volver a bajar. Quien siga dentro al terminar el Pulso pierde lo que lleva encima y no recibe el bono individual de «turno completo».
- **Caída y muerte:** al caer, un compañero tiene 30 s para reanimarte. Al morir, sueltas tu carga en una caja recuperable y **reapareces en tu bocamina** tras 15 s, con 5 s adicionales por cada muerte previa. **Durante el Pulso no hay reaparición:** morir te deja como espectador.
- **Construcción limitada:** consume *ripio* (obtenido al excavar) y tiene límites por equipo.
- **Sin fuego amigo** en la primera versión.
- **Derrumbes:** son eventos locales anunciados de forma diegética (crujidos y silbidos) que alteran bloques dentro de un volumen. No hay simulación estructural global.

### Combate: herramientas, no armas de fuego

El combate protege o disputa la minería y no debe convertirse en un juego de disparos. Por eso todo el daño sale de herramientas mineras con **proyectiles de viaje** o de **contacto**, que el servidor resuelve sin rebobinar el mundo:

- **Pico:** golpe cuerpo a cuerpo y de excavación.
- **Remachadora:** dispara remaches con tiempo de vuelo; cadencia baja y munición limitada.
- **Cargas de excavación:** se lanzan, rebotan y detonan con retardo. Abren túneles y hieren.

Si en las pruebas se demuestra que hace falta un arma instantánea, la compensación de latencia se implementará en el núcleo: historial de 1 s de cápsulas de jugador sobre la cuadrícula. Esa cifra es pequeña porque solo hay 9–12 jugadores.

### Niebla de mineral

El servidor envía a cada cliente **«roca»** en lugar del tipo real de cada mineral **no expuesto**. Un mineral se revela cuando una de sus caras queda al aire o cuando el escáner del explorador lo detecta. Así se evita el *x-ray* y el escáner gana valor de forma natural.

### Especialidades

Todas las especialidades pueden excavar, construir, defenderse y extraer. Cada una aporta una herramienta distintiva:

- **Perforador:** taladro de excavación rápida y más capacidad de cargas.
- **Ingeniero:** postes adicionales, reparación rápida y barricadas reforzadas.
- **Explorador:** escáner de resonancia (revela minerales a través de la roca y localiza ruidos más lejanos) y bengalas.

### Modos de la primera versión

- **Expedición:** el modo competitivo descrito arriba.
- **Contrato (cooperativo, 1–4 jugadores):** la misma mina, con peligros que escalan y una cuota que hay que cumplir antes del Pulso. No requiere IA de enemigos (el cerro es el enemigo), permite jugar sin reunir a 9 personas y sirve para probar el bucle de minería y logística desde el principio.
- **Primer turno** (tutorial), **campo de pruebas** y **sala privada**.

### Contenido de la primera versión completa

- Un bioma (las Galerías de Ánima) con tres franjas de profundidad.
- Dos plantillas de distribución por tamaño de mapa, con variación procedural y posiciones iniciales equilibradas.
- Tres minerales: **cobre nativo**, **cinabrio** y **resonita**; un material de construcción, el **ripio**.
- Plataformas, escaleras, barricadas, lámparas, postes, cables y estaciones.
- Progresión horizontal: variantes de herramientas con ventajas y desventajas, cosméticos y ofrendas para el altar de tu cuadrilla.

La innovación que se validará es la combinación de **terreno editable, logística vulnerable e información sensorial**. Su atractivo deberá demostrarse jugando.

---

## 2. Historia y ambientación (nuevo)

### Premisa

> Cada veinte minutos, el **Cerro Ánima** se estremece y reordena sus entrañas. En la ventana entre dos Pulsos, las cuadrillas bajan a arrancarle la **resonita**, la piedra que canta. Gana quien suba más. Pierde quien siga dentro cuando el cerro vuelva a respirar.

La historia **justifica las reglas**, no al revés:

| Regla de juego | Explicación en el mundo |
|---|---|
| El mapa se regenera en cada partida | El **Pulso** reorganiza las galerías del cerro |
| La evacuación es obligatoria al final | Quedarse dentro durante el Pulso es una sentencia |
| Las estaciones hacen ruido | La resonita **vibra** al cargarse en los cangilones; cuanto más valiosa, más canta |
| Cargar resonita a mano es arriesgado | La llevas zumbando a la espalda; el ruido te delata |
| Hay galerías diseñadas dentro de lo procedural | Son **galerías fósiles** de explotaciones anteriores al Pulso, que el cerro no consiguió tragarse |
| Los derrumbes avisan | Los mineros los atribuyen a **los Silbadores**, que silban antes de que el techo caiga |
| Tres equipos rivales | La compañía subasta **ventanas de extracción** a varias cuadrillas a la vez y gana dinero gane quien gane |

### El mundo

- **Época y tono:** retrofuturo minero de finales de los setenta, en una cordillera ficticia de inspiración andina y mexicana. Hay radios de casete, escáneres de fósforo verde, lámparas de sodio, rayas de peligro y metal pintado a brocha. El humor es obrero: hay solidaridad contra la codicia y no busca ser oscuro ni solemne.
- **La Compañía Minera Ánima (CMA):** propietaria ausente que ya no puede explotar el cerro con seguridad, así que alquila ventanas de 20 minutos a cuadrillas independientes. Es el antagonista de fondo; nunca aparece, solo envía contratos y memorandos.
- **Radio Ánima:** la voz del despacho. Anuncia las fases, avisa del Pulso y pone sabor a los contratos. Comunica casi siempre con texto y pitidos, con pocas líneas grabadas, para mantener bajo el coste.
- **Las tres cuadrillas** (identidad cosmética; no hay asimetría de juego):
  - **Cooperativa Socavón** (ocre): veteranos remendados que siguen las reglas del sindicato.
  - **Hermanos Tarco** (cian): familia de *pirquineros*, maestros de la chatarra ingeniosa.
  - **Contrata Vértice** (violeta): subcontrata de la propia CMA, con uniformes impecables y pocos escrúpulos.
- **El misterio de fondo:** ¿qué es el Pulso? ¿Por qué la resonita crece de nuevo tras cada uno? ¿Qué son los Silbadores? Son ganchos para las expansiones, sobre todo para las expediciones cooperativas con criaturas, sin necesidad de campaña.

### Cómo se cuenta (sin campaña y con poco presupuesto)

- **Primer turno:** tutorial en el que un novato baja por primera vez guiado por Radio Ánima.
- **Contratos:** cada uno lleva un breve texto de la CMA, con su toque de humor corporativo.
- **Expedientes:** fichas de historia que se desbloquean con la progresión (memorandos, recortes de prensa, cartas de mineros).
- **El altar de la bocamina:** cada cuadrilla tiene un altar en su acceso donde se colocan las **ofrendas** cosméticas desbloqueadas. Es visible en la sala de espera. Une progresión, identidad y la tradición minera de pedir protección antes de bajar.
- **Narrativa ambiental:** galerías fósiles con objetos de explotaciones pasadas.

### Nota de respeto cultural

La ambientación se inspira en culturas mineras reales: Potosí y la figura del Tío, el Muqui peruano, los pirquineros chilenos y los reales de minas mexicanos. **Por eso las entidades y nombres del juego serán originales** (los Silbadores, el Cerro Ánima) y no copias de figuras sagradas o folclóricas concretas. Se recomienda documentarse y recoger la opinión de personas de esas comunidades antes de la alfa.

### Nombre

- **«Veta»** se mantiene como nombre en clave interno.
- **Candidatos comerciales:** **Filón** (en español, «veta rica» y también «oportunidad»), **Bocamina** o **Claimjump** (para el mercado anglosajón: *claim jumping* es robar la concesión ajena). Existe un juego de Atari de 1982 llamado *Claim Jumper*, así que hay que comprobar marcas y Steam antes de decidir.

### Dirección de arte orientada al rendimiento

- Texturas de **32×32 por bloque** en un *Texture2DArray*, con filtro de punto y *mipmaps*: legibles, baratas y coherentes.
- **La oscuridad es mecánica:** la luz por bloques y la lámpara del casco hacen casi todo el trabajo visual. No hay iluminación global dinámica.
- **Siluetas y colores de equipo** reforzados con **patrones e iconos**, no solo con color, por accesibilidad.
- **Paleta limitada:** sodio naranja, fósforo verde, óxido y los tres colores de cuadrilla.

---

## 3. Tecnologías (revisadas)

### Decisión principal: núcleo C# sin Unity y servidor .NET puro

La v1 ya decide que los personajes colisionan contra la **cuadrícula voxel** y no contra PhysX. Eso significa que **el servidor no necesita Unity**: la simulación completa (terreno, movimiento, proyectiles, red de extracción y reglas) puede escribirse en C# puro.

| Componente | Tecnología | Por qué |
|---|---|---|
| **Veta.Core** | C# 9, `netstandard2.1`, **sin referencias a UnityEngine** | Mismo código en el cliente Unity y en el servidor .NET. Se prueba con `dotnet test` en segundos, sin licencia ni editor |
| **Veta.Server** | **.NET 10 LTS**, aplicación de consola | Arranque instantáneo; menos memoria que Unity sin gráficos; se ejecuta en Linux x64/ARM64, Windows y macOS; imágenes Docker pequeñas |
| **Veta.Bots** | .NET 10, clientes sin interfaz | Nueve o más clientes simulados en una sola máquina para pruebas de carga, latencia y pérdida |
| **Veta.Client** | Unity: presentación, mallas con Burst, audio, interfaz y entrada | Unity hace lo que mejor sabe hacer |
| **Host local** | El mismo `Veta.Core` y el mismo servidor, en un hilo dentro del cliente | Un único camino de código para LAN, partidas alojadas por un jugador y servidor dedicado |

El núcleo vive como **paquete UPM local** con un `asmdef` marcado como `noEngineReferences: true`, más un `.csproj` que compila **los mismos archivos** para .NET. Unity 6 admite C# 9, así que `LangVersion` queda fijado en 9 en ambos lados.

**Beneficio adicional:** si Unity cambia de licencia o de rumbo, el juego, el servidor y las pruebas no dependen de Unity. Cambiar de motor solo obligaría a rehacer la presentación.

### Red

- **Transporte: [LiteNetLib](https://www.nuget.org/packages/LiteNetLib/) 2.x** (MIT), UDP fiable o no fiable con canales. Es la misma base que usa Tugboat, funciona en `netstandard2.1` y `net8.0+` y es compatible con Unity.
- **Abstracción `ITransport`** con dos implementaciones:
  1. **LiteNetLib directo:** LAN y servidores dedicados por IP y puerto.
  2. **Steam Networking Sockets** mediante Facepunch.Steamworks: partidas alojadas por un jugador **a través del relé de Steam (SDR)**. No hay que abrir puertos y el tráfico atraviesa NAT. Funciona en Windows, Linux, macOS y Steam Deck.
- **Protocolo propio** sobre los contratos de la v1 (`PlayerCommand`, `PlayerSnapshot`, `ChunkSnapshot`/`ChunkDelta`, `MatchEvent`, `ContentManifest`). Usa serialización a nivel de bits sobre `Span<byte>` sin asignaciones de memoria y snapshots comprimidas por diferencia respecto a la última confirmada.
- **Movimiento en punto fijo:** posiciones y velocidades en enteros (1/256 de bloque, unos 4 mm). El resultado es idéntico en x64 y ARM, y en IL2CPP y CoreCLR, así que las reconciliaciones casi nunca corrigen nada y desaparece una fuente clásica de errores.
- **Alternativa evaluada: FishNet.** Ofrece predicción y reconciliación ya hechas, pero obliga a que el servidor sea Unity, y el *rollback* de colisionadores es una función de pago ([docs](https://fish-networking.gitbook.io/docs/fishnet-building-blocks/components/colliderrollback)). Como el terreno, los contratos y la predicción de la excavación son trabajo propio en cualquier caso, su ventaja real se reduce. **Punto de decisión:** a las 3 semanas del hito 1, si el *spike* de predicción propia no cumple, se pasa a FishNet sin tocar `Veta.Core`.

### Motor y versiones

- **Unity 6.3 LTS**, con un parche concreto fijado al empezar. Es la LTS vigente, con soporte hasta **diciembre de 2027** ([Unity](https://unity.com/releases/unity-6/support)).
- **Hoja de ruta que hay que tener presente:** 6.7 LTS será la **última versión basada en Mono** e incluirá un reproductor CoreCLR experimental. En 6.8 y Unity 7 se pasa a CoreCLR, .NET 10 y C# 14; la beta de Unity 7 está anunciada para diciembre de 2026 ([estado de CoreCLR](https://discussions.unity.com/t/coreclr-scripting-and-ecs-status-update-march-2026/1711852), [hoja de ruta de Unite 2026](https://www.invenglobal.com/articles/24003/unity-engine-7-changing-the-development-paradigm-and-the-roadmap-ahead)).
- **Política:**
  - Activar desde el primer día *Enter Play Mode* sin recarga de dominio. Nada de estado estático sin reinicio explícito, que es lo que más rompe la migración a CoreCLR.
  - No depender de `InstanceID`.
  - Reservar **una semana durante el hito 3** para evaluar la migración a 6.7 LTS, porque el soporte de 6.3 acaba en torno al lanzamiento.
  - **Unity 7 no se adopta antes de la versión 1.0.**
- **Compilación:** IL2CPP en todos los clientes de lanzamiento; Mono solo en el editor.

### Renderizado y terreno (cliente)

| Área | Decisión |
|---|---|
| Canalización | **URP Forward+**: muchas lámparas sin límite por objeto |
| API gráfica | Windows: DX12 con DX11 de respaldo · Linux y Steam Deck: Vulkan · macOS: Metal |
| Material de terreno | **Un único material** con *Texture2DArray* y SRP Batcher: una llamada de dibujo por chunk |
| Formato de vértice | Comprimido (8–12 bytes por vértice: posición local en bytes, índice de textura, oclusión ambiental y luz) |
| Mallas | **Greedy meshing binario** (máscaras de bits por columna) con Jobs y Burst; la fusión respeta las diferencias de oclusión y luz |
| Iluminación | **Luz por bloques con propagación BFS** en Burst, más oclusión ambiental por vértice. Luces dinámicas solo para lámparas de casco: con sombra para la propia y sin sombra para las demás |
| Visibilidad | **Culling de cuevas por conectividad de chunks** (algoritmo *cave culling* de Tommaso Checchi): en una mina, la mayoría de los chunks no se ven aunque estén en el frustum |
| Escalado | **STP** de Unity (multiplataforma) para Steam Deck y equipos de gama baja |
| GPU Resident Drawer | **Solo si las mediciones lo justifican**: con unos 1000 chunks el SRP Batcher puede bastar, y el GRD añade coste de GPU, que es el recurso más justo en la Deck |
| Chunks | Almacenamiento de 16³ (1 byte por bloque más una tabla dispersa de daño). El mapa L completo ocupa unos 3,5 MB en memoria |

### Resto de la tecnología

| Área | Decisión |
|---|---|
| Interfaz | **UI Toolkit** (UXML y USS: texto revisable y editable por IA) para menús y HUD; uGUI solo si hace falta interfaz en el mundo |
| Entrada | Input System: teclado, ratón y mando; Steam Input en la Deck |
| Audio | Audio de Unity más **propagación acústica sobre la cuadrícula** (BFS en celdas de 4³ por los huecos): el sonido «sale» por el túnel y no atraviesa la roca. FMOD queda como opción posterior |
| Idiomas | **Español e inglés desde el hito 0**, con tablas de cadenas compartidas por cliente y servidor |
| Persistencia del cliente | JSON versionado con escritura atómica y copia de respaldo (solo ajustes y cosméticos) |
| Persistencia del servidor | **SQLite** (Microsoft.Data.Sqlite) para perfiles y desbloqueos de la comunidad: transaccional, sin servicios externos |
| Compresión | Paleta más RLE por chunk y después Deflate (`System.IO.Compression`, disponible en ambos lados) |
| Errores en producción | Sentry (SDK de Unity y de .NET), opcional para el jugador |
| Pruebas | `dotnet test` (xUnit) para el núcleo; Unity Test Framework para la presentación; escenarios de rendimiento con bots |
| CI/CD | GitHub Actions: `dotnet test` en cada push (Linux x64 y macOS ARM64, para verificar el determinismo); GameCI para compilaciones de Unity; **un runner macOS para IL2CPP de Mac** |
| Distribución | **Steam** (SteamPipe, ramas beta, **Steam Playtest** para la alfa) e itch.io sin DRM con conexión directa por IP |
| macOS | Firma *Developer ID* y **notarización** de Apple (cuenta de desarrollador de 99 USD al año); sin ella, Gatekeeper bloquea el juego fuera de Steam |
| Arte | Blender, con texturas pequeñas y materiales compartidos |
| Versionado | Git, Git LFS para binarios y dependencias bloqueadas |

### ¿Por qué no Godot, Bevy o un motor propio?

| Opción | A favor | En contra | Veredicto |
|---|---|---|---|
| **Unity 6.3 + núcleo .NET** | Burst para mallas e iluminación; IL2CPP; exportación madura a las tres plataformas; herramientas de perfilado | Hoja de ruta en transición (CoreCLR); escenas menos amigables para la IA | **Elegida** |
| Godot 4.6 con C# | Código abierto; escenas en texto; .NET moderno | Sin equivalente a Burst; red y predicción hechas a mano igualmente; menos herramientas de perfilado | Alternativa razonable; con el núcleo separado, migrar sería viable |
| Bevy (Rust) | Rendimiento y ECS nativo | API inestable entre versiones, sin editor, ecosistema joven | Demasiado riesgo para un desarrollador individual |
| Motor propio | Control total | Años de trabajo antes de que el juego sea jugable | Descartado |

### Plataformas

- **Versión 1.0:** Windows x64, Linux x64 y macOS Apple Silicon, con juego cruzado.
- **Steam Deck:** objetivo de rendimiento y controles. Se medirán **tanto la compilación nativa de Linux como la de Windows con Proton** y se publicará la que rinda mejor.
- **Servidor dedicado:** Linux x64 y **ARM64** (VPS ARM baratos o una Raspberry Pi 5 para comunidades pequeñas), además de Windows.
- **Fuera de alcance:** navegador, móviles y consolas. El núcleo sin Unity facilita adaptaciones futuras.
- **Equipo de desarrollo:** la máquina actual (por la ruta del usuario, parece una ThinkPad T490s con gráficos Intel UHD 620) es **un banco de pruebas de gama baja excelente**, pero se quedará corta como estación principal para el editor de Unity y las compilaciones IL2CPP. Conviene trabajar en un equipo más potente y reservar este para validar.
- La validación de macOS sigue necesitando un Mac real (por ejemplo, un Mac mini M-series de segunda mano).

---

## 4. Arquitectura y rendimiento

### Módulos

| Módulo | Ubicación | Responsabilidad |
|---|---|---|
| Simulación | Core | Reglas, inventarios, daño, red de extracción, fases, puntuación |
| Mundo voxel | Core | Datos, generación, consultas, edición, niebla de mineral, propagación acústica |
| Protocolo | Core | Mensajes, serialización, predicción y reconciliación |
| Servidor | Server | Sesiones, transporte, persistencia y registro de partidas |
| Presentación | Client | Mallas, luz, cámara, interfaz, animación, sonido y efectos |
| Contenido | Core + Client | Definiciones con identificadores estables; los ScriptableObjects se exportan a datos del núcleo |
| Herramientas | Client + bots | Generación de escenas, pruebas, compilación y métricas |

### Terreno

- Generación **solo en el servidor** (.NET, con `Parallel.For` y `Vector128` si hace falta: un mapa L tarda menos de 1 s). Así se **evita el problema del determinismo de coma flotante entre plataformas**, porque el cliente nunca genera.
- Al unirse, el cliente recibe **el mapa completo comprimido** (objetivo: ≤1 MB para el mapa L, ya con la niebla de mineral aplicada) y después solo diferencias fiables, ordenadas y versionadas.
- Ningún GameObject ni componente de red por bloque.
- Reconstrucción solo de los chunks modificados y sus vecinos. Los trabajos se identifican por revisión, de modo que una malla antigua nunca sobrescribe un cambio nuevo.
- **Excavación predicha:** el cliente oculta el bloque al instante, pone la edición en cola con su número de comando y la revierte si el servidor la rechaza.
- Controlador cinemático en punto fijo contra la cuadrícula, idéntico en cliente y servidor.
- Generador = galerías fósiles diseñadas + variación procedural. Un validador comprueba accesibilidad, salidas, distancias a vetas y reparto por equipo. Si una semilla falla, se regenera; si falla de nuevo, se usa una plantilla válida.

### Red y autoridad

- **Servidor autoritativo a 30 ticks por segundo**; snapshots a 20–30 Hz según el ancho de banda.
- Predicción del movimiento y de la excavación local; interpolación de los jugadores remotos.
- Validación de cada acción por alcance, recursos, cadencia y estado. Los comandos son idempotentes por número de secuencia.
- Con 9–12 jugadores **no hace falta gestión de interés** para las entidades: se envían todas. **Opcional en el hito 5:** ocultar las posiciones de enemigos que no se ven ni se oyen (anti *wallhack*), reutilizando la propagación acústica.
- Reconexión durante 60 s; el personaje sigue siendo vulnerable en el mundo. Nadie puede unirse a una partida ya empezada.
- La conexión rechaza protocolos o catálogos de contenido incompatibles.

### Alojamiento y progreso

- **Partida alojada por un jugador mediante el relé de Steam:** «invitar a un amigo» sin abrir puertos. Es el camino principal para grupos privados.
- **Servidor dedicado** .NET por IP y puerto, con archivo de configuración y una imagen Docker oficial.
- **Navegador de servidores** mediante la API de servidores de Steam (hito 5 o 6). Así hay descubrimiento público sin infraestructura propia.
- Lista local de favoritos e itch.io con conexión directa por IP.
- **Progreso:** los desbloqueos que afectan al juego se guardan en el SQLite del servidor de la comunidad, identificados por SteamID o por una clave local en itch.io. Los cosméticos y los ajustes se guardan en el cliente. Es el modelo «cada servidor es su comunidad», como en los servidores de Minecraft.
- Una caída del servidor termina la expedición. Los envíos ya asegurados se registran al momento con identificadores que impiden aplicar la misma recompensa dos veces.

### Presupuestos de rendimiento

| Escenario | Objetivo |
|---|---|
| Ryzen 5 3600, GTX 1660, 16 GB | 1080p, calidad media, p95 de fotograma ≤16,7 ms |
| Steam Deck | 800p con STP, calidad baja, p95 ≤25 ms |
| **Gama baja: iGPU Intel UHD 620 / Iris Xe, 8 GB** | **720p con STP, calidad mínima, p95 ≤33 ms** |
| Memoria del cliente | ≤1,5 GB durante la partida de referencia |
| **Servidor .NET**, 2 núcleos modernos | 12 jugadores, p95 de tick ≤8 ms, RAM ≤256 MB |
| Red estable | Promedio ≤48 KB/s de descarga por cliente; pico ≤128 KB/s |
| Unión a partida | Mapa ≤1 MB; del clic a jugar ≤5 s en LAN |
| Excavación propia | El bloque desaparece en el mismo fotograma; la malla, en ≤2 fotogramas |
| Excavación y explosiones | Sin pausas superiores a 33 ms atribuibles al terreno |

El escenario de referencia es **una partida L con 9 bots** que excavan, montan redes, se cortan entre sí y detonan cargas, reproducible desde una semilla y un guion. Se mide en compilaciones ejecutables, nunca en el editor.

---

## 5. Hitos

Cada hito termina con una compilación jugable y criterios verificables. Las duraciones son orientativas para un desarrollador individual con IA.

| Hito | Entrega | Condición para avanzar | Duración aprox. |
|---|---|---|---|
| **0. Base reproducible** | Repositorio, Core, servidor y cliente vacíos; CI con `dotnet test` y compilaciones de las tres plataformas; tablas de idioma | El cliente arranca en las tres plataformas; el servidor .NET arranca en Linux; el cliente se conecta por *loopback* | 2–3 semanas |
| **1. Minería en red** | Servidor autoritativo desde el primer día; movimiento y excavación predichos; mallas, luz y culling | Dos clientes por LAN ven exactamente el mismo terreno; presupuestos iniciales cumplidos. **Semana 3: decisión entre red propia y FishNet** | 6–8 semanas |
| **2. Primera demostración** | Estación, poste, cable, cangilones, corte, reparación y robo de cajas; bots | **Dos jugadores excavan, montan, cortan, reparan y extraen.** Nueve bots se mantienen consistentes con 150 ms y 2 % de pérdida | 6 semanas |
| **3. Partida completa** | Equipos, fases, Pulso, muerte y reaparición, entregas, puntuación y modo Contrato | Una Expedición y un Contrato empiezan, terminan y conceden resultados. **Evaluación de Unity 6.7 LTS** | 8 semanas |
| **4. Prueba de identidad** | Especialidades, niebla de mineral, propagación acústica, lámparas, peligros, primer equilibrado | Tres sesiones con grupos externos: se observan decisiones de **arriesgar, construir, disputar o retirarse** | 8 semanas |
| **5. Alfa privada** | Arte coherente, audio, Primer turno, progresión en servidor, mando, Steam Playtest, relé de Steam, notarización | Personas externas instalan, invitan a amigos sin abrir puertos y completan partidas en las tres plataformas | 10–12 semanas |
| **6. Versión 1.0 comunitaria** | Correcciones, optimización, navegador de servidores, Docker y documentación | Supera las pruebas prolongadas, multiplataforma y de recuperación | 8 semanas |

En total son unos **12–14 meses**. Si una prueba con grupos externos muestra que la minería y la red de extracción apenas influyen en las decisiones, **se corrige el bucle antes de añadir contenido**.

### Pruebas obligatorias

Se mantienen todas las de la v1 y se añaden:

- **Determinismo del núcleo:** las mismas entradas producen el mismo estado, byte a byte, en x64 y ARM64.
- **Niebla de mineral:** un cliente nunca recibe el tipo de un mineral no expuesto ni escaneado. La prueba inspecciona los paquetes.
- **Predicción de la excavación:** una edición rechazada se revierte sin dejar artefactos visuales ni de colisión.
- **Cangilones:** cortar un cable con cangilones en tránsito los deja caer exactamente una vez; nunca se duplican ni se pierden.
- **Fuzzing del protocolo:** paquetes truncados, aleatorios o maliciosos no tumban el servidor.
- **Propagación acústica:** un sonido no atraviesa roca sólida y sí sale por un túnel.
- **Carga:** 12 bots durante dos horas, sin crecimiento sostenido de memoria en el cliente ni en el servidor.
- **Partidas alojadas mediante relé** entre Windows, Linux, macOS y Steam Deck.

---

## 6. Método de trabajo con IA

- Mantener `AGENTS.md` (o `CLAUDE.md`), el documento de diseño y registros de decisiones de arquitectura breves.
- **Prioridad de la IA: `Veta.Core`.** Es C# puro con `dotnet test`, así que Opus puede escribir, probar y depurar la mayor parte del juego sin abrir Unity. Es la mayor mejora de productividad de esta revisión.
- Escenas mínimas generadas por scripts de editor reproducibles; interfaz en UXML y USS; contenido definido en datos.
- Versionar las escenas como texto, conservar los `.meta` y revisar los cambios de assets.
- Una funcionalidad completa por cambio: código, integración, prueba y compilación.
- Verificar por separado los cambios de concurrencia, red, guardado y rendimiento.
- Registrar métricas antes y después de cada optimización.
- No añadir dependencias si las elegidas ya cubren la necesidad.

---

## 7. Riesgos principales

| Riesgo | Probabilidad | Mitigación |
|---|---|---|
| No se reúnen jugadores suficientes | Alta | Equipos de 1–4, mapas S/M/L, modo Contrato, invitación por Steam sin puertos |
| La red propia se retrasa | Media | *Spike* temprano, punto de decisión en la semana 3 del hito 1 y FishNet como alternativa sin tocar Core |
| El bucle de logística no resulta divertido | Media | Primera demostración en el hito 2, pruebas externas en el hito 4 y corrección antes de añadir contenido |
| Trampas (*x-ray*, *wallhack*, perfiles editados) | Media | Niebla de mineral, servidor autoritativo, progreso en servidor y ocultación opcional de posiciones |
| La transición de Unity (6.7 LTS / CoreCLR / Unity 7) rompe el proyecto | Media | Núcleo sin Unity, nada de recarga de dominio ni estático sin reinicio, ventana de migración en el hito 3 |
| macOS sin un Mac de pruebas | Media | Mac mini de segunda mano antes del hito 2, runner macOS en CI y notarización en el hito 5 |
| El arte crece sin control | Media | Texturas de 32×32, paleta limitada, una sola mina y la oscuridad como recurso estético |

---

## 8. Fuera de alcance y expansiones

**Fuera de la versión 1.0:** mundo infinito, supervivencia extensa, fabricación compleja, enemigos con navegación por terreno destructible, vehículos, mercado, pérdida permanente de equipo, matchmaking global propio, mods ejecutables y servidores masivos.

**Orden de expansión tras la 1.0:**

1. Más contratos, biomas y plantillas.
2. Nuevas herramientas de logística (vagonetas, bombas de gas, tornos dobles).
3. **Los Silbadores:** expediciones cooperativas con criaturas que viven en el terreno y responden al ruido y la luz. La propagación acústica de la 1.0 ya les da su comportamiento base.
4. Servicios públicos de conexión (emparejamiento mediante salas de Steam).

Cada ampliación deberá preservar los presupuestos de rendimiento.

**El proyecto se considerará cumplido** cuando una comunidad pueda descargarlo, invitar a sus amigos o levantar un servidor, y jugar expediciones completas, estables y divertidas en las tres plataformas.
