# ZAPA: plan v3 (antes «Proyecto Veta»)

Revisión del 22 de octubre de 2026. Sustituye a `PLAN.md` y `PLAN_v2.md` como referencia de implementación.

**Supuestos fijados:** una sola persona desarrollando con IA (Claude Code u Opus); todo debe poder compilarse, ejecutarse, probarse y **verse** desde la terminal; prioridad al rendimiento y a las plataformas de escritorio; volver al espíritu de Infiniminer.

Las cifras de rendimiento y los plazos son objetivos por validar, no mediciones.

---

## 0. Qué cambia respecto a la v2 y por qué

| Cambio | Motivo |
|---|---|
| **Unity → C# / .NET 10 con MonoGame** (API de XNA, la misma que usó Infiniminer) | Sin editor, escenas binarias ni migraciones de Unity. La IA compila, prueba y hace capturas por línea de comandos. Un único entorno de ejecución en cliente y servidor |
| **Sin armas de fuego: el terreno mata** | Vuelve a Infiniminer, nos separa de todos los competidores de disparos con bloques y elimina la parte más difícil de la red (el registro de impactos con latencia) |
| **Dos equipos y cuatro clases clásicas**, de 1 contra 1 a 8 contra 8 | Es la fórmula de Infiniminer; los tres equipos pasan a ser una variante posterior |
| **Mapa de 64×64×64** (96×96×64 para 8 contra 8) | Combate más denso, partidas más cortas y un mapa de apenas 256 KB: red y rendimiento triviales |
| **Partidas de 12 minutos**: carrera a una meta de dinero con un Pulso final | Ritmo con eventos, remontadas y un final que importa |
| **La logística automática es opcional**: llevar el botín a mano o montar una línea de cangilones | La decisión entre ambas es el diferenciador (propuesta de GPT, aceptada) |
| **Bots que excavan** | Rellenan equipos cuando faltan personas y sirven como prueba de carga y regresión para la IA |
| **Sin progresión permanente ni modo Contrato en la 1.0** | Recortar alcance para una sola persona |
| **Ambientación mínima** | La historia la cuenta cada partida (crónica automática); el trasfondo queda en tres líneas |

### Respuesta a la revisión de GPT

Estoy **de acuerdo** con casi todo:

- **Validar antes de optimizar:** el prototipo feo va primero; el *greedy meshing*, el *cave culling* y la propagación acústica avanzada se dejan para después.
- **El alcance era excesivo para una persona.**
- **Partidas de 8 a 12 minutos.**
- **Niebla de mineral por equipo**, no global.
- **Steam y LiteNetLib no son intercambiables:** los mensajes del juego van por encima de dos transportes con implementaciones separadas.
- **Las cifras de compresión y de llamadas de dibujo deben medirse.**
- **El determinismo exige pruebas continuas**, no basta con usar punto fijo.

Sus objeciones sobre Unity (el servidor .NET 10 no puede ejecutarse dentro de Unity; cambiar a FishNet no sería sencillo; la incertidumbre de Unity 7) **desaparecen** con esta v3: todo usa el mismo entorno .NET 10 y el servidor local es literalmente el mismo código en otro hilo.

---

## 1. Identidad

### Nombre: **ZAPA**

- En español, la *zapa* es la pala del zapador, y una **«labor de zapa»** es un trabajo oculto para socavar al rival. Eso resume el juego: cavar y sabotear.
- La palabra viene del italiano *zappa*, igual que el inglés **sap/sapper** (zapa y zapador en asedios). Funciona en los dos idiomas; además, en inglés suena a *zap*, como el bloque eléctrico.
- Es corta, se pronuncia sin problemas y no se encontró ningún juego en Steam con ese nombre (hay que confirmarlo con una búsqueda de marcas antes de anunciarlo).
- **Alternativa:** **CONTRAMINA / COUNTERMINE**, el término militar real para un túnel cavado para interceptar la mina enemiga. Solo existe un programa de Commodore 64 de 1984 con ese nombre.
- **Lema:** *«Nadie dispara. El cerro mata.»*

### Qué conservamos de Infiniminer

Infiniminer (Zachtronics, 2009, XNA) era un juego competitivo de dos equipos que minaban para ganar dinero, con cuatro clases (prospector, minero, ingeniero y zapador), bloques de equipo, campos de fuerza que solo dejan pasar a tu equipo, escaleras, plataformas de salto, bloques eléctricos para matar intrusos, explosivos, balizas y un banco donde depositar el mineral. Ganaba el equipo que reunía más dinero ([Wikipedia](https://en.wikipedia.org/wiki/Infiniminer), [MobyGames](https://mobygames.com/game/214999/infiniminer)).

**Murió por tres razones:** su código no estaba ofuscado y se extendieron los clientes trucados; las versiones modificadas fragmentaron a la comunidad; y no generaba ingresos. Su autor lo abandonó y acabó publicándolo con licencia MIT ([Wikipedia](https://en.wikipedia.org/wiki/Infiniminer), [Crosscut](https://crosscut.com/2015/08/the-godfather-of-minecraft-a-conversation-with-indie-gamer-zachary-barth)).

ZAPA conserva **la carrera económica por equipos, las clases, los bloques tácticos, el estilo visual simple y legible, y la ausencia de armas**. Corrige sus errores con un servidor autoritativo, niebla de mineral, un único protocolo versionado con servidores comunitarios configurables (en vez de versiones bifurcadas) y la venta en Steam.

### Qué añadimos (el diferenciador)

1. **Logística saboteable y opcional.** El botín se puede **llevar a mano** (silencioso, ocupa a un jugador, y si mueres lo sueltas) o enviar por una **línea de cangilones** (automática y de gran capacidad, pero ruidosa, y si alguien la corta, la carga cae y se puede robar).
2. **Guerra de contraminas.** Los túneles enemigos se detectan **escuchando**. El **geófono** del prospector capta excavaciones a través de la roca. Interceptar un túnel rival es una jugada, no una casualidad.
3. **La luz y el ruido son información.** Tu lámpara te deja ver y te delata; tus máquinas trabajan y se oyen.
4. **Partida con estructura:** eventos, remontada, un Pulso final y una crónica automática.

### Frente a la competencia

| Juego | Qué comparte | Qué hace ZAPA distinto |
|---|---|---|
| Minecraft | Bloques y minería | Partidas competitivas cortas; nada de supervivencia |
| Deep Rock Galactic | Minería en equipo, clases, evacuación | JcJ entre cuadrillas; sin armas ni enemigos |
| Ace of Spades, Block N Load | JcJ con bloques editables | No es de disparos: matan el terreno y las trampas |
| THE FINALS (Cashout) | Equipos que disputan dinero en escenarios destructibles | Hay que **cavar** para encontrarlo y **transportarlo** |
| Dome Keeper Versus (abril de 2026) | Minería competitiva con robo de recursos ([Xbox Wire](https://news.xbox.com/en-us/2026/04/13/dome-keeper-multiplayer/)) | 3D, en primera persona, en un mapa compartido con túneles que se cruzan |
| Hydroneer | Logística minera satisfactoria | La logística existe para disputarla, no para optimizarla |

**Posicionamiento:** *el juego de minería competitiva sin armas: cava, roba la veta y córtale la línea al rival antes de que el cerro despierte.*

---

## 2. Reglas

### Formato

- **Dos equipos:** **Cobre** (naranja) y **Cobalto** (azul). Es el par de colores más distinguible para el daltonismo y homenajea al rojo contra azul de Infiniminer. Cada equipo lleva además iconos y patrones propios.
- **De 1 contra 1 a 8 contra 8.** Los bots completan los equipos si el servidor lo permite.
- **Mapa generado en cada partida:** 64×64×64 hasta 4 contra 4; 96×96×64 hasta 8 contra 8. La superficie tiene un campamento por equipo y el subsuelo se vuelve más rico cuanto más hondo.

### Economía doble (como Infiniminer)

- **Mena:** aparece por todas partes y se gasta en construir. Se lleva encima como recurso personal.
- **Oro y diamante:** son el dinero. Hay que **depositarlos en el banco** del equipo para que cuenten. Si mueres, sueltas una **bolsa** que cualquiera puede recoger.
- **Victoria:** gana el primer equipo que alcanza la **meta de dinero**, que se ajusta según el número de jugadores. Si nadie llega antes del final del Pulso, gana el que más tiene; los empates se mantienen.
- **El banco es indestructible**, pero sus accesos se pueden enterrar, inundar de lava o llenar de trampas.

### Clases

| Clase | Herramienta distintiva | Papel |
|---|---|---|
| **Prospector** | Detector (revela oro y diamante en un cono) y **geófono** (oye excavaciones a través de la roca); balizas | Encontrar y avisar |
| **Minero** | Pico rápido; doble capacidad de carga | Extraer y transportar |
| **Ingeniero** | Más tipos de bloque; **línea de cangilones** | Fortificar y automatizar |
| **Zapador** | Explosivos; cizalla para cortar líneas | Abrir paso y sabotear |

Todas las clases pueden cavar, construir bloques básicos y llevar botín. Puedes cambiar de clase al reaparecer.

### Bloques

- **Naturales:** tierra, roca, roca dura (lenta de picar), mena, oro, diamante y **lava**. La lava fluye (autómata celular simulado en el servidor a 4 Hz).
- **De construcción (cuestan mena):**
  - bloque de equipo;
  - **campo de fuerza** (tu equipo lo atraviesa y el rival no);
  - escalera;
  - plataforma de salto;
  - **bloque eléctrico** (electrocuta a los rivales que lo tocan);
  - carril (más velocidad);
  - baliza;
  - explosivo (solo el zapador);
  - estación y poste de cangilones (solo el ingeniero).

### Muerte y combate

- **No hay armas.** Se muere por lava, electricidad, explosivos, caídas o derrumbes, es decir, por el terreno que otros preparan.
- **El pico empuja** (retroceso, sin daño): sirve para tirar a un rival a la lava o a un pozo.
- **Reaparición en el campamento** tras 6 s (10 s durante el Pulso). Al morir pierdes lo que llevabas, no tiempo de partida.

### Línea de cangilones (ingeniero)

- **Elementos:** una estación de carga junto a la veta, postes cada 10 bloques como máximo en línea recta por el aire y llegada al banco.
- **Funcionamiento:** depositas el botín en la estación y los cangilones lo llevan solos a 4 bloques/s. Hace ruido y la línea es visible.
- **Sabotaje:** si alguien corta la línea (con la cizalla o metiendo un bloque en su trayectoria), los cangilones en tránsito caen como bolsas.
- **Reparación:** un poste dañado se repara en 2 s; no hace falta reconstruir la línea.
- **Equilibrio:** los cables se colocan rápido (de poste a poste, con conexión automática si hay línea de visión) para que montar una línea no sea una tarea repetitiva.

### Información

- **Luz:** la lámpara del casco se puede apagar. Las balizas y las líneas de cangilones son visibles desde lejos.
- **Sonido por el aire:** viaja por los túneles (búsqueda en anchura sobre una cuadrícula gruesa) y no atraviesa la roca.
- **Vibración por la roca:** quien cava, explota o mueve una línea emite vibraciones. A pocos bloques se oyen; con el geófono, desde lejos.
- **Niebla de mineral por equipo:** el servidor guarda, para cada equipo, qué minerales conoce (un bit por bloque: 32 KB por equipo). Un mineral se revela cuando un compañero ve una de sus caras expuestas o cuando lo detecta el prospector. El cliente nunca recibe minerales desconocidos.

---

## 3. La partida de 12 minutos

La «historia» de ZAPA no es una campaña: **es lo que pasa en cada partida.** La estructura sirve para que cada una tenga planteamiento, nudo y desenlace.

| Minuto | Fase | Qué ocurre | Qué decisión provoca |
|---|---|---|---|
| 0:00–0:20 | **Bajada** | Elección de clase y vista del mapa geológico aproximado (bandas de riqueza, sin posiciones exactas) | ¿Por dónde cavamos? |
| 0:20–3:00 | **Fiebre** | El oro somero se agota pronto; aparecen los primeros depósitos y fortificaciones | Repartirse o ir juntos |
| **3:00** | **Sondeo** | Radio anuncia la zona aproximada de una **Veta Madre** (mucho diamante) entre ambos campamentos | Ir rápido o prepararse; ¿línea o a mano? |
| 3:00–8:00 | **Asedio** | Túneles que se cruzan, geófonos, trampas eléctricas y líneas cortadas | Atacar la línea rival o defender la propia |
| **6:00** | **Fama** | El equipo que pasa del 60 % de la meta queda **marcado**: su banco emite una baliza visible para todos | Quien lidera tiene que defenderse; quien va detrás sabe adónde ir |
| **8:00** | **Sacudida** | Un evento al azar: derrumbe que abre una caverna, bolsa de lava liberada o segunda Veta Madre (algo más cerca del equipo que va perdiendo) | Adaptarse a un mapa que cambia |
| **10:00** | **Pulso** | La lava sube desde el fondo, una capa cada pocos segundos. Las líneas se apagan en el último minuto. En el fondo aparece el **Corazón**, un depósito de enorme valor rodeado de lava | Asegurar lo que hay o arriesgarlo todo por el Corazón |
| 12:00 | **Cierre** | Resultado (o antes, si un equipo llega a la meta) | — |

### Mecanismos de remontada (sin regalar puntos)

- **Fama:** quien lidera queda expuesto (posición del banco y líneas más ruidosas).
- **Botín robado ×1,25:** el oro arrebatado al rival vale más al depositarlo. Robar compensa, pero exige riesgo.
- **La segunda Veta Madre** puede aparecer algo más cerca del equipo que pierde (ajustable).
- **El Corazón** da una última oportunidad real a quien va detrás.

### Después de la partida: la crónica

El servidor registra cada `MatchEvent`. Al terminar, se genera una **crónica** con los 5–7 momentos clave, su minuto y un minimapa, por ejemplo:

> «Min 7:12 · Lucía (Zapadora, Cobalto) cortó la línea de Cobre y robó 1.400 $.»

Incluye además estadísticas curiosas por clase (más metros cavados, más botín robado, más muertes en la lava). Así cada partida cuenta su propia historia y se puede compartir con una captura.

El servidor también guarda una **repetición** (comandos y deltas) que se puede volver a ver y sirve para depurar.

### Variantes (configurables en el servidor, después de la 1.0)

- **Tres equipos.**
- **Partida corta** de 8 minutos.
- **Sin línea de cangilones**, solo a mano (modo clásico).
- **Fiebre infinita** (sin Pulso).

---

## 4. Ambientación (mínima)

El **Cerro Ánima** se reordena con cada **Pulso**, lo que explica que el mapa se regenere y que la lava suba al final. Una compañía minera alquila ventanas de extracción a cuadrillas rivales y gana dinero gane quien gane. **Radio Ánima** narra la partida con texto y pitidos.

Eso es todo. El estilo visual es de **pixel art de 16×16 con bloom**, como Infiniminer, en una paleta de mina retro (sodio, fósforo verde, cobre y cobalto). Personajes y objetos se construyen **con cubos desde el código**, con animación procedural, sin necesidad de Blender.

---

## 5. Tecnología pensada para programar con IA

### Principio

**Todo es código o texto, y todo se ejecuta desde la terminal.** Si la IA no puede compilar, probar y **ver** un resultado sin abrir una interfaz gráfica, esa herramienta no entra en el proyecto.

### Pila

| Área | Decisión | Por qué ayuda a la IA |
|---|---|---|
| Lenguaje y entorno | **C# 14 / .NET 10 LTS** en cliente, servidor, bots y pruebas | Un lenguaje, un entorno, compilación en segundos; la IA domina C# |
| Framework de juego | **[MonoGame 3.8.5](https://monogame.net/blog/2026-07-15-3.8.5-release-2026/)** (API de XNA 4, la de Infiniminer) | Solo código, paquetes NuGet y `dotnet new`; quince años de ejemplos de XNA/MonoGame en los datos de entrenamiento. Incluye paquetes ARM64, un objetivo Vulkan (DesktopVK) y DX12 |
| Plan B gráfico | **[FNA](https://github.com/FNA-XNA/FNA3D)** (también XNA 4, con SDL3/SDL_GPU: Vulkan, Metal y D3D12) | Al compartir la API, cambiar cuesta días, no meses. Se activa si MonoGame falla en macOS o Steam Deck |
| Red | **LiteNetLib 2.x** + un transporte Steam aparte (Facepunch.Steamworks y su relé) | Una biblioteca pequeña y conocida; los mensajes del juego no dependen del transporte |
| Interfaz | **Gum** en modo solo código para menús y HUD; **ImGui.NET** para depuración | Sin editor visual obligatorio |
| Audio | `SoundEffect` de MonoGame con OGG (NVorbis) y propagación propia sobre la cuadrícula | Sin herramientas externas |
| Recursos | PNG y WAV/OGG cargados en tiempo de ejecución; **sin pipeline de contenido** (salvo los shaders `.fx`, compilados en Windows y versionados) | Archivos simples, revisables en Git |
| Texturas | Pixel art de 16×16; un script puede generarlas y retocarse con Pixelorama o Aseprite | La IA puede escribir el generador |
| Datos | JSON para bloques, clases, reglas y eventos | Ajustar el equilibrio sin tocar código |
| Persistencia | JSON en el cliente (ajustes); SQLite en el servidor (estadísticas y lista de bloqueados) | Estándar y fiable |
| Pruebas | xUnit, pruebas de capturas e invariantes | Ejecutables con `dotnet test` |
| CI | GitHub Actions con matriz Windows, Ubuntu y macOS: compilar, probar y publicar ejecutables | Sin licencias ni editores en CI |
| Distribución | Steam (Playtest para la alfa) e itch.io | — |
| Entorno de trabajo | VS Code con C# Dev Kit (o Rider), Claude Code y `dotnet watch` (recarga en caliente) | — |

### El bucle de verificación de la IA

Son los comandos que hay que crear en el hito 0, porque convierten a Claude en alguien que puede comprobar su propio trabajo:

```bash
dotnet test
```
Reglas, terreno, protocolo e invariantes en segundos.

```bash
zapa sim --seed 42 --bots 8 --minutes 12 --report out/sim.json
```
Partida completa sin gráficos con bots. Devuelve un JSON con el resultado, los eventos, el p95 de tick y las invariantes que se rompieron.

```bash
zapa shot --scene tests/scenes/tunel_lava.json --out out/tunel.png
```
Dibuja una escena en una textura fuera de pantalla y la guarda como PNG. **Claude puede leer la imagen** para comprobar mallas, luz o interfaz.

```bash
zapa replay out/partida.zrep --at 07:12 --shot out/momento.png
```
Reproduce un error exacto a partir de una repetición.

```bash
zapa bench --scenario ref --frames 3000 --out out/bench.csv
```
Tiempos de fotograma para comparar antes y después de optimizar.

**Invariantes comprobadas en todas las simulaciones:**

- **Conservación del dinero:** banco + bolsas + cargado + en tránsito + en el mapa solo cambia al minar o al depositar.
- Ningún cliente recibe minerales desconocidos para su equipo.
- Todo bloque tiene un tipo válido; el cliente y el servidor tienen el mismo terreno tras cada partida simulada.

**Reglas para `AGENTS.md`:**

- Un sistema por archivo, de 400 líneas como máximo.
- Ninguna lógica de juego en el cliente: el cliente solo dibuja, oye y envía órdenes.
- Cada funcionalidad nueva llega con una prueba y, si es visual, con una escena de captura.
- Las reglas ajustables viven en JSON.
- No se añade ninguna dependencia nueva sin un registro de decisión.

### Estructura del repositorio

```
/src/Zapa.Core      reglas, terreno, simulación, protocolo (sin gráficos)
/src/Zapa.Server    consola: servidor dedicado y servidor local en un hilo
/src/Zapa.Client    MonoGame: dibujo, audio, entrada, interfaz
/src/Zapa.Bots      bots que juegan (para rellenar equipos y para pruebas)
/src/Zapa.Cli       sim, shot, replay, bench
/tests/             xUnit y escenas de captura
/content/           png, ogg, json, shaders
/docs/              diseño y decisiones
AGENTS.md
```

### ¿Por qué no Unity, Godot u otros?

| Opción | Problema para una persona con IA |
|---|---|
| Unity | Depende del editor, las escenas y prefabs son difíciles de revisar, la instalación es enorme, necesita licencia en CI y está migrando a CoreCLR y Unity 7 en pleno desarrollo |
| Godot (C#) | Mejor que Unity, pero sigue girando alrededor del editor; las escenas son texto, aunque frágiles de editar a mano |
| Bevy (Rust) | API que cambia en cada versión (la IA genera código desfasado) y compilación lenta |
| TypeScript + Three.js | La IA lo domina y permite jugar en el navegador, pero rinde menos y no hay UDP nativo en navegador. Queda como posible versión web futura |

**Riesgo aceptado:** MonoGame no trae editor, iluminación ni físicas. En ZAPA eso es una ventaja: el mundo es una cuadrícula, la colisión es contra esa cuadrícula y la luz se calcula por bloques. No necesitamos nada más.

---

## 6. Arquitectura y rendimiento

### Simulación y red

- **Servidor autoritativo a 30 Hz.** El cliente predice su movimiento y su excavación; los demás jugadores se interpolan.
- **Movimiento en punto fijo** (enteros, 1/256 de bloque), con pruebas de determinismo en CI en x64 y ARM64.
- **Mapa completo al unirse:** 64³ ocupa 256 KB sin comprimir; con paleta, RLE y Deflate se espera bastante menos (hay que medirlo). Después solo se envían diferencias numeradas.
- **Lava** simulada solo en el servidor a 4 Hz; los cambios se envían como deltas agrupados.
- **Sin compensación de latencia:** no hay disparos. El pico (empuje) se valida por alcance con margen.
- Reconexión durante 60 s. No se admite unirse con la partida empezada en la 1.0 (en partidas con bots, un humano puede ocupar el sitio de un bot, como mejora posterior).
- **Alojamiento:** servidor local dentro del cliente (el mismo código en un hilo), invitación por el relé de Steam sin abrir puertos, y servidor dedicado `dotnet` para Linux, Windows, macOS y ARM64, con imagen Docker.

### Cliente

| Técnica | Primera versión | Más adelante, si se mide la necesidad |
|---|---|---|
| Mallas | Eliminación de caras ocultas por chunk de 16³ | *Greedy meshing* |
| Luz | Luz por bloques (BFS) y oclusión ambiental por vértice | Actualización incremental |
| Visibilidad | Recorte por frustum | Recorte por conectividad de cuevas |
| Dibujo | Un atlas de texturas y un búfer de vértices por chunk | — |
| Audio | Atenuación por distancia y oclusión simple | Propagación por túneles |

Con 64 chunks (o 144 en el mapa grande) y texturas de 16×16, la carga gráfica es mínima.

### Presupuestos

| Escenario | Objetivo |
|---|---|
| GTX 1660 / Ryzen 5 3600 | 1080p, p95 ≤6,9 ms (144 FPS) |
| **Intel UHD 620 (tu T490s)** | 1080p, p95 ≤16,7 ms (60 FPS) |
| Steam Deck | 800p, p95 ≤16,7 ms |
| Memoria del cliente | ≤600 MB |
| Servidor, 8 contra 8 con lava | p95 de tick ≤3 ms, RAM ≤150 MB |
| Red | ≤32 KB/s por cliente de media |
| Unión a partida | ≤3 s en LAN |

---

## 7. Hitos

Cada hito termina con algo **jugable con gente real**. Los plazos son aspiracionales, no estimaciones fiables.

| Hito | Contenido | Condición de salida | Aprox. |
|---|---|---|---|
| **0. Esqueleto** | Solución .NET, ventana MonoGame en Windows, Linux y macOS (CI), servidor local, `zapa shot` y `zapa sim` vacíos | Un cubo texturizado se ve en las tres plataformas y Claude obtiene su captura por línea de comandos | 1–2 sem |
| **1. Turno Cero** (el Infiniminer mínimo) | Mapa de 64³, movimiento, cavar y poner bloques, oro y banco, dos equipos, meta de dinero, LAN | **Dos personas juegan una partida completa por LAN y gana alguien** | 4–6 sem |
| **2. Bloques y clases** | Las cuatro clases, campo de fuerza, eléctrico, explosivos, escaleras, saltos, lava, muertes, bolsas, bots | 4 contra 4 con bots; la invariante de conservación del dinero se cumple en 1000 simulaciones | 4–6 sem |
| **3. Diferenciador** | Línea de cangilones, geófono, luz y ruido, niebla de mineral por equipo, eventos de partida, Pulso y crónica | **Prueba decisiva:** en partidas con grupos externos se observa que la gente desvía túneles para proteger líneas, deja la veta para atacar, esconde infraestructura y elige entre asegurar o arriesgar | 6 sem |
| **4. En línea** | Relé de Steam, servidor dedicado, reconexión, Steam Deck, rendimiento | 8 contra 8 por Internet sin abrir puertos | 4–6 sem |
| **5. Alfa (Steam Playtest)** | Arte final de 16×16, audio, tutorial corto, español e inglés, opciones, mando | Gente desconocida instala, juega y vuelve a jugar | 6–8 sem |
| **6. 1.0** | Pulido, variantes de reglas, documentación del servidor | Partidas largas estables en las tres plataformas | 6 sem |

**Si en el hito 3 la logística y las contraminas no generan esas decisiones, se rediseña antes de pasar al hito 4.** Hasta entonces el juego ya es un «Infiniminer moderno» jugable, lo que tiene valor por sí mismo.

---

## 8. Riesgos

| Riesgo | Mitigación |
|---|---|
| Pocos jugadores | Bots que excavan, partidas desde 1 contra 1, invitación por Steam, partidas cortas |
| MonoGame flojea en macOS o Steam Deck | Prueba en el hito 0; FNA como plan B con la misma API |
| La IA acumula complejidad innecesaria | Hitos pequeños, invariantes, límite de líneas por archivo y optimización solo tras medir |
| Trampas | Servidor autoritativo, niebla de mineral por equipo, sin información de más en el cliente |
| Destrozo gratuito («si no gano, lo entierro todo») | Banco indestructible, bloques de equipo resistentes, límites de construcción, el robo vale más que destruir |
| La logística no resulta divertida | Prueba decisiva en el hito 3 y modo clásico sin línea de cangilones como alternativa |
| Equipo de desarrollo modesto | MonoGame y .NET son ligeros; la T490s sirve para desarrollar y como banco de pruebas de gama baja |

---

## 9. Fuera de la 1.0

Progresión permanente, cosméticos de pago, modo cooperativo contra el cerro, tres equipos (llegará como variante), criaturas, mundo infinito, fabricación, emparejamiento propio, mods ejecutables (sí habrá reglas por JSON) y versión web.
