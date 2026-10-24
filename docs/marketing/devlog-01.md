# Devlog #01 — Por qué estamos construyendo nuestro propio motor de vóxeles (Hito 0: Fundaciones)

> ***Rob a sleeping mountain. Play dirty. Keep it hushed.***

¡Hola a todos y bienvenidos al primer diario de desarrollo de **HUSHMINE**!

Si alguna vez jugaste a Infiniminer en 2009 —aquel experimento brillante de Zachtronics del que
nació Minecraft—, recordarás la adrenalina de excavar a ciegas con tu cuadrilla, escuchar un pico
enemigo al otro lado de la pared y tender conductos para llevar el oro a la superficie antes que
el equipo rival. **HUSHMINE** nace para recuperar esa tensión táctica de minería competitiva y
sabotaje industrial (de 1 contra 1 a 8 contra 8, con bots que completan huecos), pero añadiendo
un tercer actor en discordia: **el Durmiente** (*The Sleeper*).

Estamos en 1979, infiltrándonos de noche en el monte Anima para saquear las vetas que quedaron
abandonadas tras el desastre de 1931, cuando cuarenta y un mineros de doce países desaparecieron
en el turno de noche dejando sus **41 fichas de latón** (*Brass tallies*) sin recoger en el
tablón de entrada. Mientras **Copper Crew** y **Cobalt Crew** compiten por el oro mediante
**líneas de cangilones** (*Bucket lines*), explosivos, cizallas y **geófonos** (*Geophones*),
cada explosión y cada máquina en marcha alimenta un **Sismógrafo** (*Seismograph*) compartido de
0 a 100 %. Hacer ruido acelera tu victoria, pero despierta a la montaña; hacer una **Ofrenda**
(*Offering*) de oro en el altar calma al gigante y compra tiempo para remontar.

Hoy cerramos oficialmente el **Hito 0 (Fundaciones)** del proyecto. Antes de excavar la primera
galería hacia la **Veta Madre** (*Mother Lode*) o el **Corazón** (*The Heart*), quiero contaros
con total honestidad la decisión más importante que hemos tomado: **por qué estamos construyendo
nuestro propio motor en Rust, llamado Grisú, y cómo ha cobrado vida en un portátil modesto.**

---

## 1. La gran pregunta: ¿por qué un motor propio en lugar de un motor genérico?

Cuando empiezas un juego independiente en solitario, el consejo habitual es automático: *«No hagas
tu propio motor; usa Unity, Godot o cualquier herramienta comercial y empieza a prototipar hoy
mismo»*. Para el 95 % de los géneros es un consejo excelente. Para un juego de vóxeles
competitivo, en cambio, la historia cuenta algo muy distinto.

Antes de escribir la primera línea de código dedicamos tiempo a estudiar los juegos de vóxeles
que han triunfado y perdurado durante años, y dejamos nuestras conclusiones registradas en la
decisión de arquitectura [0001-motor-propio-rust-wgpu.md](../adr/0001-motor-propio-rust-wgpu.md)
(`docs/adr/0001-motor-propio-rust-wgpu.md`):

- **Minecraft** nació sobre un motor pequeño y propio en Java y OpenGL, y se propagó como la
  pólvora gracias a una demo jugable directamente en el navegador (Minecraft Classic).
- **Vintage Story**, **Teardown** y **Luanti** mantienen motores propios en C# y C++ porque la
  destrucción total del terreno, la iluminación por bloques y la generación de mallas son el
  corazón mismo del juego.
- **Veloren** demostró a toda la comunidad que un juego de vóxeles multijugador escrito en Rust
  con `wgpu` es perfectamente viable y rinde de maravilla.
- **La lección de Hytale:** quizá el aviso más claro de todos. Planear empezar con una tecnología
  temporal pensando *«ya migraremos o reescribiremos el motor más adelante»* es una trampa mortal;
  Hytale intentó reescribir su motor a mitad de camino para hacerlo multiplataforma y acabó
  descartando años de reescritura para volver a su versión anterior y poder salir en acceso
  anticipado. **No hay que planear una migración futura de motor: hay que construir desde el
  primer día el motor que vas a conservar.**

En un juego como **HUSHMINE**, un motor genérico como Unity no te ahorra el trabajo duro. La
estructura de datos de los bloques, la generación de mallas sin pausas, la propagación de luz y
sonido por túneles excavados en tiempo real y la sincronización determinista por deltas a través
de la red tienes que escribirlos a mano igualmente. A cambio de nada, cargas con un recolector de
basura que introduce tirones al regenerar geometría, binarios pesados y una barrera enorme para
ofrecer una versión web instantánea.

### Por qué elegimos Rust 2024 + wgpu 30

Decidimos construir exactamente lo que **HUSHMINE** necesita apoyándonos en librerías de código
abierto con licencias permisivas: **Rust 2024**, **`wgpu` 30**, **`winit` 0.30** y **`glam`**.

1. **Rendimiento nativo sin recolector de basura:** cuando una carga del Zapador vuela media
   caverna durante los 90 segundos finales del **Despertar** (*Wake*) tras superar **Revuelo**
   (*Stir*), **Respira** (*Breathe*) e **Inquietud** (*Restless*), el motor debe recalcular mallas
   e iluminación al vuelo sin que un recolector de basura congele el fotograma.
2. **Multiplataforma real y demo de un clic en el navegador:** gracias a **`wgpu`**, nuestro
   código gráfico habla de forma nativa con DirectX 12, Vulkan, Metal y OpenGL en Windows, Linux,
   macOS y Steam Deck, y además compila a WebAssembly + **WebGPU** para el navegador. Queremos
   que en el futuro cualquiera pueda entrar a un **Turno** (*Shift*) o probar el modo creativo
   **Taller** (*Workshop*) haciendo clic en un enlace, igual que hacíamos con Minecraft Classic.
3. **Un compilador estricto como red de seguridad:** desarrollar en solitario con ayuda de
   herramientas de inteligencia artificial tiene un riesgo evidente: que el código crezca rápido
   pero esconda errores sutiles de memoria o concurrencia. En Rust, el compilador y el sistema de
   tipos actúan como un auditor implacable. Si añadimos la prohibición total de código inseguro
   en el espacio de trabajo y reglas estrictas donde el servidor jamás puede abortar ante datos
   externos, descartamos clases enteras de fallos antes siquiera de abrir la ventana.

---

## 2. Qué es Grisú y qué hemos construido en el Hito 0

Nuestro motor se llama **Grisú** (del francés *grisou*, el gas metano que se acumula en las
galerías de las minas de carbón: invisible, silencioso y explosivo si pierdes de vista la lámpara
de seguridad).

Hay una regla de oro que nos hemos autoimpuesto para no caer en el clásico «síndrome del motor
eterno»: **los motores sin juego mueren**. Por eso **Grisú** vive dentro del mismo repositorio que
**HUSHMINE** y solo implementa lo que el juego necesita en cada hito. Sin embargo, desde el
primer día hemos trazado una frontera férrea en la arquitectura:

| Crate en el repositorio | Capa | Responsabilidad en el Hito 0 |
|---|---|---|
| `crates/grisu-render` | Motor **Grisú** | Inicialización de GPU con `wgpu`, cámara 3D, cubo con perspectiva y captura sin ventana |
| `crates/grisu-platform` | Motor **Grisú** | Ventana con `winit`, bucle principal de eventos y gestión de entrada |
| `crates/hush-sim` | Juego **HUSHMINE** | Reglas y simulación determinista a 30 Hz en punto fijo, sin gráficos |
| `crates/hush-client` | Juego **HUSHMINE** | Binario ejecutable del juego que une plataforma, renderizado y simulación |
| `crates/hush-cli` | Herramientas | Interfaz de línea de órdenes para simular partidas, generar capturas e inspeccionar la GPU |

Los crates del motor (`crates/grisu-render` y `crates/grisu-platform`) **tienen estrictamente
prohibido depender de cualquier crate del juego** (`crates/hush-sim`, `crates/hush-client` o
`crates/hush-cli`). Cuando **HUSHMINE** llegue a su lanzamiento en acceso anticipado,
**publicaremos Grisú como un motor de vóxeles de código abierto** para que otros desarrolladores
puedan crear sus propios mundos subterráneos.

### Lo que ya funciona hoy al cerrar el Hito 0

Puede que un primer hito de fundaciones suene poco vistoso desde fuera, pero lo que ya late bajo
el capó nos da una base de roca madre para todo lo que viene:

- **Nuestro primer cubo de mineral en 3D (`crates/grisu-render` y `crates/grisu-platform`):** ya
  tenemos nuestra tubería gráfica en **`wgpu`** inicializando la GPU, gestionando la ventana y el
  bucle de eventos con **`winit`**, aplicando perspectiva con **`glam`**, búfer de profundidad y
  sombreado por cara para dibujar nuestro cubo fundacional girando con suavidad.
- **Simulación determinista a 30 Hz (`crates/hush-sim`):** el corazón lógico del juego no sabe
  qué es una tarjeta gráfica ni utiliza números en coma flotante para su estado. Todo funciona
  con enteros y punto fijo a 30 ticks por segundo y un generador pseudoaleatorio propio con
  semilla explícita. El esqueleto del bucle determinista, la huella de estado y el informe de
  integridad ya están en marcha y listos para verificar en milisegundos las invariantes matemáticas
  de la partida —como la **conservación exacta del oro**, para garantizar que ni una sola pepita
  aparezca o desaparezca por arte de magia a medida que entren las reglas de economía—.
- **Verificación sin ventana desde la terminal (`crates/hush-cli`):** esta es una de mis piezas
  favoritas del Hito 0. Ejecutando `hush-cli shot` desde la consola, el motor levanta el
  adaptador gráfico sin crear ninguna ventana en el sistema operativo, renderiza la escena en una
  textura fuera de pantalla y guarda un archivo PNG listo para inspeccionar. Junto con
  `hush-cli sim` y `hush-cli gpu`, tanto yo como los agentes de verificación automática podemos
  comprobar visual y numéricamente cada cambio en segundos.

### La trinchera del hardware real: la anécdota del ThinkPad T490s y Vulkan

Construir un motor propio te pone cara a cara con la realidad del hardware desde el primer día, y
no quería desarrollar **HUSHMINE** únicamente en una estación de trabajo de gama alta para
descubrir dentro de un año que el juego se arrastra en equipos modestos. Mi banco de pruebas
diario es un portátil **Lenovo ThinkPad T490s con gráfica integrada Intel UHD 620** en Windows:
si el motor vuela aquí a 1080p, volará en cualquier parte, incluida la Steam Deck.

Durante el Hito 0, al poner en marcha las primeras pruebas de `crates/grisu-render`, nos topamos
con un misterio digno de los túneles del monte Anima: el proceso se cerraba en seco sin llegar
siquiera a devolver un error de Rust. Tras investigar a fondo con `hush-cli gpu`, descubrimos al
culpable: el cargador de Vulkan del driver de Intel en Windows (versión 31.0.101.2137) abortaba
el proceso nativo en el instante mismo de enumerar la instancia.

¿La solución? Configuramos **Grisú** para que en Windows utilice **DirectX 12** (con OpenGL como
respaldo) de forma predeterminada, dejando Vulkan para Linux/Steam Deck y Metal para macOS, pero
permitiendo forzar cualquier backend mediante variables de entorno. En cuanto activamos DirectX 12
por defecto, la Intel UHD 620 despertó al instante, renderizó el cubo a la perfección y generó
nuestras capturas sin ventana sin pestañear. Ese tipo de control inmediato sobre la pila gráfica
es exactamente la razón por la que merece la pena tener motor propio.

---

## 3. Qué viene en el Hito 1: ¡hora de picar roca!

Con las fundaciones asentadas, las licencias auditadas y el bucle de verificación en verde, nos
adentramos en el **Hito 1 (Vóxel)**. Aquí es donde ese cubo solitario deja paso a una mina viva:

- **Un mundo subterráneo de 64³ bloques dividido en chunks de 16³:** compacto en memoria (apenas
  256 KB por mapa completo de 64×64×64), diseñado para viajar rápido por la red y regenerarse en
  paralelo.
- **Mallas rápidas en CPU e iluminación de cavernas:** implementaremos eliminación de caras
  ocultas, caras empaquetadas de forma ultracompacta para la GPU, propagación de luz por bloques
  y **oclusión ambiental por vértice** para que cada esquina de roca y cada galería transmitan
  profundidad y claustrofobia.
- **Cámara en primera persona y colisiones en punto fijo:** movimiento fluido con física de
  cuadrícula compartida entre cliente y servidor, para que asomarse a una grieta o saltar sobre
  un pozo de lava sea preciso y predecible.
- **Cavar y construir en menos de 50 ms:** lanzaremos rayos (mediante el algoritmo DDA sobre la
  cuadrícula) para picar nuestros primeros bloques de tierra, roca y mineral de oro, colocar los
  primeros bloques de construcción y medir con nuestro banco de pruebas que un mapa entero de
  64³ se regenera en menos de 50 milisegundos incluso en la gráfica integrada del T490s.

Gracias de corazón por leer hasta aquí y por acompañarme desde el kilómetro cero de esta
excavación. En el próximo devlog os enseñaré las primeras capturas dentro de los túneles
generadas directamente desde **Grisú**.

Mientras tanto, recordad el consejo de los viejos mineros del monte Anima: **preparad el pico,
vigilad la aguja del Sismógrafo… y no despertéis al Durmiente.**
