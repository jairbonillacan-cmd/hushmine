# HUSHMINE — Kit de prensa / Press Kit

> ***Rob a sleeping mountain. Play dirty. Keep it hushed.***

Este documento reúne la información oficial, la ficha técnica bilingüe, la sinopsis narrativa,
la filosofía técnica del motor **Grisú**, el inventario de materiales gráficos y la lista de
comprobación previa al anuncio público de **HUSHMINE**.

---

## 1. Ficha técnica / Fact Sheet

| Campo / Field | Español | English |
|---|---|---|
| **Título / Title** | **HUSHMINE** | **HUSHMINE** |
| **Lema / Tagline** | *Rob a sleeping mountain. Play dirty. Keep it hushed.* | *Rob a sleeping mountain. Play dirty. Keep it hushed.* |
| **Género / Genre** | Minería competitiva en vóxeles por equipos y sabotaje industrial | Team-based competitive voxel mining & industrial sabotage |
| **Desarrollador / Developer** | Estudio independiente (desarrollo individual) | Independent studio (solo developer) |
| **Editor / Publisher** | Autopublicado | Self-published |
| **Motor / Engine** | **Grisú** (motor propio en Rust 2024 + wgpu 30 + winit 0.30 + glam; código abierto en el acceso anticipado) | **Grisú** (custom engine in Rust 2024 + wgpu 30 + winit 0.30 + glam; open-source at Early Access) |
| **Plataformas previstas / Target Platforms** | PC (Windows, Linux, macOS), Steam Deck y demo en navegador (WebGPU) | PC (Windows, Linux, macOS), Steam Deck, and playable browser demo (WebGPU) |
| **Jugadores / Players** | 1v1 a 8v8 (multijugador competitivo por equipos + bots + modo creativo Taller / *Workshop*) | 1v1 to 8v8 (team competitive multiplayer + bots + creative *Workshop* mode) |
| **Servidor / Server** | Servidor autoritativo determinista a 30 Hz en punto fijo (UDP, WebTransport, memoria y relé de Steam) | 30 Hz deterministic fixed-point authoritative server (UDP, WebTransport, in-memory, and Steam relay) |
| **Duración por partida / Match Length** | 9 a 14 minutos por Turno (*Shift*) | 9 to 14 minutes per *Shift* |
| **Idiomas / Languages** | Inglés (fuente) y español (ES/LATAM); previstos PT-BR, ZH-CN, RU, DE, FR y JA | English (source) and Spanish (ES/LATAM); planned PT-BR, ZH-CN, RU, DE, FR, and JA |
| **Contacto / Press Contact** | `<EMAIL>` | `<EMAIL>` |

---

## 2. Sinopsis / Pitch

### Pitch en una frase (ES / EN)

- **ES:** Dos cuadrillas mineras rivales se infiltran de noche en una montaña viva para saquear
  su oro, tender líneas de cangilones y sabotear los túneles enemigos sin despertar a la bestia
  geológica que duerme bajo sus pies.
- **EN:** Two rival mining crews slip past the fences at night to strip a living mountain of its
  gold, string up bucket lines, and sabotage enemy tunnels—without waking the geological beast
  sleeping beneath their boots.

### Premisa canónica en inglés (Storefront Copy)

> **1931.** Mount Anima was the richest mountain on Earth, and miners came from every corner of
> the world to dig it. Then, one night, the mountain woke up. The entire graveyard shift —
> forty-one miners from twelve countries — never came back up. Their brass tallies still hang
> missing from the board.
>
> **1979.** The mountain sleeps again… and it dreams of gold. Every time it settles, its tunnels
> shift and fresh veins bloom. Every night, rival crews slip past the fences to rob a sleeping
> giant.
>
> The old-timers have one rule: ***don't wake the Sleeper.***

### Premisa en español

En **1931**, el monte Anima era la montaña más rica de la Tierra y atrajo a mineros de todos los
rincones del mundo. Una noche, la montaña despertó y se tragó entero al turno de noche: cuarenta
y un mineros de doce países nunca regresaron a la superficie. Sus **41 fichas de latón**
(*Brass tallies*) siguen faltando en el tablón de entrada de la quebrada compañía
*Anima Consolidated*.

En **1979**, el monte Anima vuelve a dormir… y sueña oro. Cada vez que se asienta, sus galerías
cambian de lugar y brotan vetas nuevas en la roca. Cada noche, dos cuadrillas rivales saltan las
vallas oxidadas para saquear al gigante dormido entre monitores de fósforo verde, cintas de
casete y lámparas de vapor de sodio:

- **Copper Crew (Cobre, naranja):** las familias antiguas del pueblo minero, descendientes
  directos del turno perdido en 1931. Bajan a las galerías por el oro… y para recuperar las
  fichas de los suyos.
- **Cobalt Crew (Cobalto, azul):** recién llegados equipados con material industrial de la
  antigua Compañía. Toman su nombre de los *kobolds* del folclore centroeuropeo: *«los mineros a
  los que los duendes no asustan»*.

Ambos equipos son idénticos en prestaciones y eligen entre cuatro oficios complementarios
(**Prospector**, **Minero**, **Ingeniero** y **Zapador**), guiados por la radio del **Vigía**
(*The Watcher*) y los silbidos diegéticos de **los Silbadores** (*The Whistlers*) que anuncian
cada derrumbe. Los veteranos solo tienen una regla: ***no despiertes al Durmiente***
(*The Sleeper*).

### Glosario esencial (English / Español)

| English | Español | Función en el juego / Role |
|---|---|---|
| **The Sleeper** | **el Durmiente** | La montaña viva que reacciona al ruido de ambas cuadrillas |
| **Seismograph** | **Sismógrafo** | Medidor compartido de 0 a 100 % que marca el riesgo de despertar |
| **Stir / Breathe / Restless / Wake** | **Revuelo / Respira / Inquietud / Despertar** | Los cuatro umbrales sísmicos (25 %, 50 %, 75 % y 100 %) |
| **Mother Lode** | **Veta Madre** | Caverna rica en mineral que se abre al llegar al 50 % |
| **The Heart** | **el Corazón** | Depósito final rodeado de lava que emerge al 100 % |
| **Offering** | **Ofrenda** | Oro depositado en el altar para reducir el Sismógrafo |
| **Brass tally** | **Ficha de latón** | Las 41 chapas de 1931; calman al Durmiente y revelan historia |
| **Bucket line** | **Línea de cangilones** | Cadena aérea automatizada para transportar oro al banco |
| **Geophone** | **Geófono** | Herramienta acústica para oír excavaciones a través de la roca |
| **Shift / Workshop** | **Turno / Taller** | Partida competitiva estándar / Modo creativo y editor libre |

---

## 3. Historia del proyecto y filosofía técnica

### El legado de Infiniminer y el sabotaje sin armas

En abril de 2009, Zachtronics publicó Infiniminer, el juego de minería competitiva por equipos
donde la gente descubrió el placer de excavar y construir en primera persona con bloques —la
chispa directa de la que nació Minecraft—. Sin embargo, aquel subgénero original de **minería
táctica competitiva e ingeniería bajo presión** quedó casi huérfano.

**HUSHMINE** retoma esa fórmula fundacional y la lleva más lejos. Aquí **no hay armas de fuego**:
el pico solo empuja al rival. En las profundidades del monte Anima se muere por caídas, bolsas de
lava, descargas de bloques eléctricos, derrumbes o cargas de demolición colocadas con astucia.
Cada cuadrilla combina cuatro oficios:

- **Prospector:** equipa detector de mineral y **geófono** para escuchar picadas enemigas a
  través de la roca sólida.
- **Minero (*Miner*):** excava a gran velocidad y carga el doble de mineral en su bolsa.
- **Ingeniero (*Engineer*):** levanta fortificaciones, campos de fuerza de equipo, plataformas de
  salto y tiende la **línea de cangilones** (*Bucket line*).
- **Zapador (*Sapper*):** abre brechas con explosivos y sabotea líneas rivales con la cizalla.

### El Sismógrafo: el ruido como recurso compartido

La gran aportación de diseño de **HUSHMINE** es el **Sismógrafo** (*Seismograph*), un medidor
global de 0 a 100 % visible para todos que registra cuánto falta para que el **Durmiente**
despierte:

- **Hacer ruido acelera la extracción, pero acerca el desastre:** detonar explosivos (+3 %),
  mantener líneas de cangilones en marcha (+0,3 % cada 10 s), activar bloques eléctricos o dejar
  fluir la lava (+0,2 %) hace subir la aguja.
- **La Ofrenda (*Offering*) y el dilema táctico:** quien va ganando quiere hacer ruido para cerrar
  la partida rápido; quien va perdiendo necesita silencio y tiempo. Depositando oro en el altar
  del equipo —una auténtica *hush money* o soborno a la montaña— o entregando una de las **41
  fichas de latón** encontradas en la roca, los jugadores reducen el Sismógrafo y compran minutos
  vitales. Incluso el sabotaje tiene dos caras: volar el túnel del líder añade ruido al cerro,
  mientras que cortar su línea de cangilones con la cizalla la apaga y silencia la mina.

Al terminar cada turno de 9 a 14 minutos, el juego genera una **Crónica posterior** automática de
5 a 7 momentos clave lista para compartir como imagen (con títulos como *Despertador*, *Guardián
del sueño*, *Topo* o *Ladrón de cangilones*). Además, el modo **Taller** (*Workshop*) ofrece
construcción libre sin límites y sirve como editor de mapas y plantillas para la comunidad.

### Por qué construimos nuestro propio motor: Grisú

Los juegos de vóxeles que perduran —como Minecraft, Vintage Story, Teardown, Veloren o Luanti—
comparten un rasgo esencial: **todos utilizan un motor propio**. En un mundo de bloques 100 %
destructible donde la geometría, la iluminación por túneles y el estado de red cambian a cada
segundo, un motor genérico como Unity no ahorra la parte difícil, y el caso reciente de Hytale
demostró el peligro de planear una migración de motor a mitad de camino en lugar de construir
desde el primer día el motor que vas a conservar.

Por eso **HUSHMINE** funciona sobre **Grisú** (del francés *grisou*, el gas inflamable de las
minas de carbón), un motor propio escrito desde cero en **Rust 2024** con **`wgpu` 30**,
**`winit` 0.30** y **`glam`**, documentado en
[0001-motor-propio-rust-wgpu.md](../adr/0001-motor-propio-rust-wgpu.md)
(`docs/adr/0001-motor-propio-rust-wgpu.md`):

- **Separación estricta entre motor y juego:** los crates del motor (`crates/grisu-render`,
  `crates/grisu-platform`) tienen prohibido depender de los crates del juego (`crates/hush-sim`,
  `crates/hush-client`, `crates/hush-cli`). Cuando **HUSHMINE** llegue a su lanzamiento en acceso
  anticipado, **Grisú se publicará como proyecto de código abierto** con licencia permisiva.
- **Rendimiento nativo y demo de un clic en navegador:** gracias a Rust y `wgpu`, el mismo código
  fuente compila de forma nativa sobre DirectX 12, Vulkan, Metal y OpenGL para escritorio y Steam
  Deck, y también se ejecuta en el navegador mediante **WebGPU** para ofrecer una demo
  instantánea de un clic al estilo de Minecraft Classic.
- **Determinismo e integridad:** toda la simulación en `crates/hush-sim` usa aritmética entera y
  punto fijo a 30 Hz, sin coma flotante en el estado ni pánicos ante datos externos, garantizando
  partidas idénticas byte a byte y verificación automática desde la terminal con `crates/hush-cli`.

---

## 4. Activos incluidos en el kit de prensa (Media & Assets)

El paquete de prensa descargable incluye los siguientes materiales listos para medios y creadores:

1. **Logotipos oficiales (`logos/`):**
   - Logotipo principal de **HUSHMINE** en alta resolución (SVG vectorial y PNG transparente a
     4K) en versiones a color, blanco sobre oscuro y negro monocromo.
   - Emblema técnico del motor **Grisú** (lámpara de seguridad minera y llama azulada) en SVG y
     PNG.
2. **Icono y cápsulas de tienda (`capsules/`):**
   - Icono cuadrado de aplicación (512×512 y 1024×1024 px).
   - Ilustración clave (*Key Art*) y cápsulas horizontal, vertical y de cabecera mostrando el
     sabotaje en acción: un Zapador cortando la línea de cangilones rival sobre un pozo de lava
     mientras la aguja del Sismógrafo entra en zona roja.
3. **Hoja de colores e identidad de equipos (`branding/color-sheet.pdf`):**
   - Paleta industrial de 1979: verde fósforo de terminal (`#33FF66`), ámbar de vapor de sodio
     (`#FFB000`), latón envejecido (`#C59B27`) y pizarra profunda (`#14181D`).
   - Contraste de equipos diseñado para lectura instantánea en túneles oscuros y accesibilidad
     para daltonismo: **Copper Crew** (naranja cálido `#E86A17`) frente a **Cobalt Crew** (azul
     cobalto `#1D63DC`).
4. **Capturas de pantalla (`screenshots/`):**
   - Serie de capturas PNG a 1080p y 4K sin compresión con pérdidas, generadas de forma
     reproducible mediante escenas del motor (`hush-cli shot`).
5. **GIFs y clips de jugabilidad (`gifs/`):**
   - *Clip 01 — El Sismógrafo y los umbrales:* transición de *Revuelo* (25 %) a *Despertar*
     (100 %) con caída de escombros y ascenso de lava hacia el Corazón.
   - *Clip 02 — Línea de cangilones y sabotaje:* cadena de cangilones transportando mineral de
     oro en serie hasta que una cizalla rival corta el cable y hace caer el botín.
   - *Clip 03 — Guerra acústica con geófono:* un Prospector apoyando el geófono en la pared para
     detectar picadas enemigas al otro lado de la veta.
   - *Clip 04 — La Ofrenda:* entrega de oro y de una ficha de latón de 1931 en el altar del
     equipo para apaciguar al Durmiente en el último segundo.

---

## 5. Lista de comprobación previa al anuncio (Pre-announcement Checklist)

Antes de publicar el primer tráiler, abrir la página de tienda o enlazar el repositorio público,
se deben completar y verificar todos los puntos siguientes:

- [ ] **Registro de marcas en USPTO y EUIPO:**
  - Comprobar disponibilidad fonética y gráfica de **HUSHMINE** en las bases de datos de la
    **USPTO** (Estados Unidos) y de la **EUIPO** (Unión Europea).
  - Presentar la solicitud de registro de marca en la **Clase 9** (software de videojuegos
    descargable y grabado) y en la **Clase 41** (servicios de entretenimiento y videojuegos en
    línea).
- [ ] **Reserva en Steam (Steamworks):**
  - Completar el alta de desarrollador en **Steam** (Steamworks), abonar la tasa de aplicación y
    reservar el identificador único (AppID) para **HUSHMINE**.
  - Configurar la página «Próximamente» en inglés y español con las cápsulas de sabotaje,
    etiquetas de género y botón de lista de deseados (*Wishlists*), dejando preparado el entorno
    de *Steam Playtest* para el Hito 6.
- [ ] **Reserva en itch.io:**
  - Crear la cuenta oficial y reclamar la URL del proyecto en **itch.io** (`hushmine.itch.io`).
  - Preparar la ficha para alojar tanto las compilaciones nativas de la alfa como la demo jugable
    de un clic en el navegador con WebGPU.
- [ ] **Registro de dominios web:**
  - Adquirir y bloquear los dominios **`hushmine.com`**, **`hushmine.gg`** y **`hushmine.game`**.
  - Configurar DNS, certificados TLS/HTTPS y registros de correo para el buzón de prensa
    (`<EMAIL>`).
- [ ] **Reserva de cuentas en redes sociales y comunidades (`@hushmine`):**
  - Reclamar el identificador unificado **`@hushmine`** en **YouTube**, **X/Twitter**,
    **Bluesky**, **TikTok**, **Reddit** (incluyendo el subreddit oficial `r/hushmine`) y
    **Discord**.
  - Configurar la estructura inicial del servidor de Discord con canales de anuncios, registro de
    cambios (*devlogs*), reporte de errores y coordinación de partidas de prueba por equipos.
