# Guía de estilo de localización e internacionalización (`i18n`)

Este documento define las normas lingüísticas, técnicas y culturales para localizar **HUSHMINE** a partir de su fuente canónica en inglés. Toda traducción oficial o comunitaria debe respetar esta guía para preservar la claridad táctica en partida, la atmósfera de 1931/1979 y la compatibilidad técnica con **Project Fluent** y el renderizador de texto del motor **Grisú**.

---

## 1. Arquitectura de localización en Project Fluent (`.ftl`)

**HUSHMINE** gestiona todas sus cadenas visibles mediante **Project Fluent** (`fluent-rs`). Ningún sistema de `hush-client` ni de `hush-sim` contiene texto literal incrustado en el código fuente: la simulación emite eventos deterministas e identificadores tipados, y el cliente resuelve la cadena localizada en el idioma activo.

### 1.1. Idioma fuente y variantes iniciales

- **Inglés (`assets/i18n/en/`):** idioma fuente canónico. Todo término nuevo se redacta y valida primero en inglés.
- **Español de Latinoamérica (`assets/i18n/es-419/`):** primera traducción completa de referencia para Hispanoamérica.
- **Español de España (`assets/i18n/es-ES/`):** primera traducción completa adaptada a la norma peninsular.

### 1.2. Estructura de los 5 archivos por idioma

Cada carpeta de idioma contiene exactamente cinco recursos `.ftl` con responsabilidades separadas:

| Archivo | Propósito y alcance |
| :--- | :--- |
| `game.ftl` | Marca y tienda (`game-*`, `store-*`), equipos (`team-*`), modos (`mode-*`), clases (`class-*`), herramientas (`tool-*`), bloques (`block-*`), umbrales del Sismógrafo (`seismo-*`) y glosario del mundo (`lore-*`). |
| `hud.ftl` | Menús de navegación (`menu-*`), indicadores en partida (`hud-*`), avisos y errores del altar (`altar-*`) y causas de muerte (`death-*`). |
| `watcher.ftl` | Las 50 líneas de radio del Vigía (`watcher-<slug>-<n>`) agrupadas en los 10 eventos sísmicos y de turno. |
| `signs.ftl` | Los 10 carteles de seguridad (`sign-01`..`sign-10`), los 6 memorandos (`memo-01`..`memo-06`) y el tablero (`tally-board-*`). |
| `chronicle.ftl` | Plantillas del resumen de fin de turno, títulos honoríficos y eventos del `EventLog` (`chronicle-*`). |

### 1.3. Esquema de identificadores (`kebab-case`)

Todos los identificadores se escriben exclusivamente en inglés y en `kebab-case` (minúsculas separadas por guiones). Las claves son inmutables entre idiomas:

- `game-*`, `store-*`, `team-*`, `mode-*`, `lore-*`: identidad comercial, textos de tienda, cuadrillas, modos de juego y entradas de glosario.
- `block-*`: nombres de los 18 vóxeles (`block-dirt`, `block-hard-rock`, `block-force-field`, `block-rail`, `block-bucket-post`).
- `class-*`: nombres y descripciones breves de las cuatro clases (`class-prospector`, `class-miner`, `class-engineer`, `class-sapper`).
- `tool-*`: equipamiento activo (`tool-pickaxe`, `tool-geophone`, `tool-wire-cutters`, `tool-blasting-cap`).
- `seismo-*`: etiquetas y alertas de los cuatro umbrales (`seismo-stir`, `seismo-breathe`, `seismo-restless`, `seismo-wake`).
- `watcher-*`: subtítulos de radio (`watcher-stir-1` a `watcher-defeat-5`).
- `hud-*` y `menu-*`: marcadores tácticos, banco de equipo, temporizadores y navegación principal.
- `altar-*` y `death-*`: interacciones con el conducto de ofrendas (`AltarError`) y mensajes de baja o reaparición.
- `sign-*`, `memo-*` y `tally-board-*`: textos diegéticos de Anima Consolidated y archivo de las 41 fichas de latón.
- `chronicle-*`: encabezados, premios y entradas cronológicas de la Crónica de fin de turno.

### 1.4. Uso de variables y selectores de plural

Las variables dinámicas inyectadas desde el cliente se escriben con la sintaxis `{ $var }` (por ejemplo, `{ $player }`, `{ $team }`, `{ $count }`, `{ $seconds }`, `{ $number }`, `{ $cash }`). Cuando una cadena dependa de una cantidad numérica, debe emplearse un selector de plural CLDR donde la variante por defecto `*[other]` es **obligatoria**:

```ftl
hud-gold-carried = { $count -> [one] { $count } lingote de oro *[other] { $count } lingotes de oro }

tally-board-progress = { $count -> [one] { $count } de 41 Fichas de latón devueltas al tablero *[other] { $count } de 41 Fichas de latón devueltas al tablero }
```

Nunca se concatenan fragmentos de frases en código Rust: el orden sintáctico completo debe residir dentro de la plantilla `.ftl` para permitir que idiomas con estructura SOV (como el japonés) o posposiciones reordenen libremente `{ $player }` y `{ $count }`.

---

## 2. Tono y registro de las tres voces del juego

**HUSHMINE** no tiene un narrador omnisciente único. Todo texto pertenece a una de las tres voces diegéticas o funcionales del universo del monte Anima.

### 2.1. Voz 1 — El Vigía (`watcher.ftl`)

- **Contexto:** 1979. Un operador veterano de rescate minero vigila en solitario la caseta de superficie frente a un banco de válvulas calientes, una radio de onda corta y el tambor de papel continuo del sismógrafo. De niño escuchó cerrarse la montaña en 1931.
- **Tono y progresión:** seco, profesional, gastado por el tabaco y libre de melodrama. Empieza con calma clínica e instrumental en **Revuelo** (*«La aguja acaba de temblar»*), baja la voz con tensión contenida en **Respira**, pasa a alarma cortante en **Inquietud** y ordena la evacuación con urgencia absoluta pero disciplinada en **Despertar** (*«Noventa segundos. Suelten las herramientas»*).
- **Regla estricta de brevedad:** en el original en inglés, ninguna línea del Vigía puede superar las **6 palabras** separadas por espacios. En español y demás lenguas meta, las frases deben conservar esa misma contundencia telegráfica de radio analógica (con corte de pulsador PTT) para que el jugador pueda leerlas de un vistazo periférico en menos de un segundo mientras escapa de la lava o defiende su línea de cangilones.

### 2.2. Voz 2 — Anima Consolidated (`signs.ftl`)

- **Contexto:** documentación industrial de entreguerras (1919–1932) de una corporación minera apátrida que quebró tras el desastre del 14 de noviembre de 1931.
- **Carteles de seguridad (`sign-01`..`sign-10`):** placas de acero esmaltado al fuego redactadas en imperativo formal e impersonal (`usted` en español: *«DESCUELGUE SU NÚMERO ANTES DE BAJAR»*, *«SI LA VETA SILBA, DESALOJE LA GALERÍA»*). Su función real es enseñar las reglas del subsuelo sin romper la ficción.
- **Memorandos internos (`memo-01`..`memo-06`):** correspondencia mecanografiada entre **Eleanor Sterling** (Directora de Operaciones Regionales), **Arthur Vance** (Ingeniero Jefe de Superficie) y los capataces de 1931. Su sello estilístico es la **ironía contable**: tratan una entidad geológica viva que respira y sueña oro como si fuera un problema de amortización de maquinaria o un desfase de balance (*«alimentar con oro de la Compañía una grieta del suelo es robo»*).

### 2.3. Voz 3 — Interfaz, HUD y Crónica (`game.ftl`, `hud.ftl`, `chronicle.ftl`)

- **Interfaz y HUD (`game.ftl`, `hud.ftl`):** voz táctica, inmediata y sin adornos literarios. Está diseñada para leerse sobre monitores de fósforo verde y ámbar de 1979 renderizados con `glyphon` + `cosmic-text` (fuentes Noto con cobertura CJK). Los nombres de bloques, herramientas y avisos priorizan sustantivos concretos de minería e ingeniería.
- **La Crónica (`chronicle.ftl`):** al concluir el turno, el historial determinista del `EventLog` se presenta como un parte de operaciones mineras conciso y con humor seco. Registra quién abrió la Veta Madre, quién saboteó la línea de cangilones rival, quién apaciguó el altar y quién provocó el Despertar.

---

## 3. Glosario canónico obligatorio y términos prohibidos

El siguiente glosario es de **cumplimiento estricto** en todas las traducciones al español (`es-419` y `es-ES`). La sustitución de cualquiera de estos términos por sinónimos rompe la comunicación entre jugadores y la coherencia con las mecánicas de `hush-sim`.

| Término en inglés (Fuente) | Español canónico (`es-419` / `es-ES`) | Falsos amigos y traducciones PROHIBIDAS | Notas de uso y contexto mecánico |
| :--- | :--- | :--- | :--- |
| *The Sleeper* | **el Durmiente** | *el Dormido*, *el Que Duerme*, *la Durmiente*, *el Soñador* | Entidad geológica bajo el monte Anima. El artículo va en minúscula salvo a inicio de frase. |
| *Shift* | **Turno** | *Partida*, *Ronda*, *Jornada*, *Asalto* | Modo competitivo principal de 1 contra 1 hasta 8 contra 8. |
| *Seismograph* | **Sismógrafo** | *Sismómetro*, *Medidor sísmico*, *Sensor de ruido* | Medidor compartido de 0 a 100 % (`0` a `100_000` en punto fijo). |
| *Stir* (25 %) | **Revuelo** | *Agitación*, *Movimiento*, *Desperezarse* | Primer umbral (`25_000`): temblores iniciales y aviso de los Silbadores. |
| *Breathe* (50 %) | **Respira** | *Respiración*, *Aliento*, *Exhalación* | Segundo umbral (`50_000`): forma verbal conjugada; abre una Veta Madre. |
| *Restless* (75 %) | **Inquietud** | *Inquieto*, *Intranquilidad*, *Nerviosismo* | Tercer umbral (`75_000`): sustantivo abstracto; rotura de bolsas de lava. |
| *Wake* (100 %) | **Despertar** | *Despierto*, *El Despertar*, *Colapso* | Cuarto umbral (`100_000`): 90 s finales de evacuación y apertura del Corazón. |
| *Mother Lode* | **Veta Madre** | *Veta Principal*, *Filón Madre*, *Veta Maestra* | Cavidad rica en oro que se fractura al cruzar el umbral del 50 %. |
| *The Heart* | **el Corazón** | *el Núcleo*, *el Centro*, *la Cámara Central* | Depósito masivo rodeado de lava que queda expuesto al 100 %. |
| *Offering* | **Ofrenda** | *Soborno*, *Tributo*, *Sacrificio*, *Donación* | Oro o ficha de latón entregados en el altar para enfriar el Sismógrafo. |
| *Brass tally* | **Ficha de latón** | *Chapa de bronce*, *Placa de bronce*, *Medalla* | Control real de descenso minero; coleccionable de los 41 mineros de 1931. |
| *Bucket line* | **Línea de cangilones** | *Línea de cubos*, *Línea de baldes*, *Cinta transportadora* | Cadena aérea de tracción continua que sube mineral al banco del equipo. |
| *Geophone* | **Geófono** | *Sensor sónico*, *Micrófono de roca*, *Hidrófono* | Herramienta acústica del Prospector para escuchar excavaciones y pulsos. |
| *The Watcher* | **el Vigía** | *el Vigilante*, *el Observador*, *el Guardián* | Operador de radio y sismógrafo en la caseta de superficie (1979). |
| *The Whistlers* | **los Silbadores** | *los Silbidos*, *los Susurradores*, *los Aulladores* | Armónicos diegéticos que avisan de un derrumbe inminente desde el 25 %. |
| *Prospector* | **Prospector** | *Explorador*, *Buscador*, *Cateador* | Clase de reconocimiento equipada con detector y geófono. |
| *Miner* | **Minero** | *Excavador*, *Picador*, *Barrenero* | Clase de extracción rápida y doble capacidad de carga. |
| *Engineer* | **Ingeniero** | *Constructor*, *Mecánico*, *Arquitecto* | Clase que tiende la línea de cangilones y bloques avanzados. |
| *Sapper* | **Zapador** | *Artificiero*, *Demoledor*, *Saboteador* | Clase de sabotaje equipada con cargas explosivas y cizalla. |
| *Practice* / *Workshop* | **Práctica** / **Taller** | *Entrenamiento* / *Modo Creativo*, *Forja* | Modos de prueba individual y de construcción libre / creación de mapas. |
| *Awakener* | **Despertador** | *El que despierta*, *Provocador* | Título de la Crónica para quien causó más subida en el Sismógrafo. |
| *Slumber Guardian* | **Guardián del sueño** | *Protector del letargo*, *Pacificador* | Título de la Crónica para quien más redujo el Sismógrafo con ofrendas. |
| *Mole* | **Topo** | *Excavador jefe*, *Tuneladora* | Título de la Crónica para el jugador con más bloques excavados. |
| *Bucket Thief* | **Ladrón de cangilones** | *Saboteador de cubos*, *Pirata de línea* | Título de la Crónica para quien más mineral rival interceptó o cortó. |

### Nota gramatical sobre el pronombre *she* en inglés

En inglés, el Vigía alterna entre *the Sleeper* y el pronombre femenino afectivo-temeroso *she* (*«She's stirring...»*, *«She's breathing. Get out.»*), habitual en la jerga anglosajona para referirse a una mina, un buque o una montaña viva. En español **está prohibido traducirlo como «Ella se está removiendo» o «la Durmiente»**: debe usarse sujeto tácito (*«Se está removiendo...»*, *«Está respirando»*), que mantiene la ambigüedad entre **la montaña** (femenino) y **el Durmiente** (masculino) con total naturalidad idiomática.

---

## 4. Lo que NO se traduce (Invariantes de marca y nombres propios)

Para mantener una identidad internacional unificada en retransmisiones, torneos y servidores multirregionales, los siguientes elementos permanecen invariables:

1. **HUSHMINE y Grisú:** el título comercial del juego (**HUSHMINE**, siempre en mayúsculas) y el nombre del motor gráfico y de simulación (**Grisú**, con tilde en la *u*) jamás se traducen ni adaptan.
2. **Anima Consolidated y Mount Anima:** el nombre corporativo **Anima Consolidated** (o *Anima Consolidated Mining & Smelting Co.*) se mantiene en inglés. En el texto corrido en español se escribe **el monte Anima**, conservando siempre **Anima sin tilde** (nunca *Monte Ánima* ni *Consolidada Ánima*).
3. **Copper Crew y Cobalt Crew:** los nombres oficiales completos de las dos facciones son **Copper Crew** y **Cobalt Crew**. En espacios reducidos del HUD o de la Crónica donde solo quepa el identificador corto de equipo, se admite **Cobre** / **Cobalto** (o *Copper* / *Cobalt*), pero **quedan terminantemente prohibidos los híbridos** como *Cobre Crew*, *Cuadrilla Copper* o *Equipo Cobalt*.
4. **The Long Night:** cuando se cite como subtítulo oficial de la Temporada 1, se conserva **The Long Night** o se presenta en formato bilingüe (*Temporada 1: The Long Night / La Larga Noche*).
5. **Los 41 nombres propios de 1931 y el personal directivo:** nunca se castellanizan, traducen ni pierden sus signos diacríticos originales los nombres de la dirección (**Eleanor Sterling**, **Arthur Vance**) ni los de los **41 mineros del turno del 14 de noviembre de 1931** pertenecientes a las 12 diásporas:
   - *Ejemplos de grafía obligatoria:* **Thomas Trevithick** (#1, Reino Unido), **Lukas Bergmann** (#6, Alemania), **Valentín Condori** (#10, Bolivia), **Mateo Aller** (#14, España), **Bridget Gallagher** (#29, Irlanda), **Stanisław Kowalczyk** y **Tadeusz Zieliński** (Polonia, conservando `ł` y `ń`), **Sven-Olof Åström** y **Nils Söderberg** (Suecia, conservando `å` y `ö`), **Saburō Watanabe** (Japón, con macrón en `ō`), **Pádraig O'Sullivan** y **Seán Kavanagh** (Irlanda, con tilde aguda gaélica).

---

## 5. Criterios de divergencia entre `es-419` (Latinoamérica) y `es-ES` (España)

### 5.1. Regla de oro de interoperabilidad

Un jugador de Ciudad de México, Bogotá o Buenos Aires (`es-419`) y otro de Madrid o Vigo (`es-ES`) comparten partida, canal de voz y servidores. Por ello, **el 100 % del glosario mecánico y de los nombres de clases, herramientas, umbrales del sismógrafo y elementos del altar es idéntico en ambas variantes** (*Turno*, *Sismógrafo*, *Revuelo / Respira / Inquietud / Despertar*, *Veta Madre*, *el Corazón*, *Ofrenda*, *Ficha de latón*, *Línea de cangilones*, *Geófono*).

La divergencia entre `assets/i18n/es-419/` y `assets/i18n/es-ES/` se aplica **únicamente** donde la gramática alocutiva o el vocabulario técnico cotidiano sonarían artificiales para el hablante nativo de cada región:

1. **Tratamiento plural de segunda persona en radio (`watcher.ftl`) y memorandos (`signs.ftl`):**
   - En `es-419` se emplea el paradigma de **ustedes** (imperativos y desinencias en `-en` / `-an`).
   - En `es-ES` se emplea el paradigma de **vosotros** cuando el Vigía se dirige con urgencia a la cuadrilla por radio (imperativos en `-ad` / `-ed` / `-id` y formas como *oís* o *habéis*), mientras que los carteles corporativos `SIGN-01`..`SIGN-10` mantienen el imperativo formal de `usted` en ambas variantes por tratarse de órdenes patronales de 1931.
2. **Tiempos verbales de pasado reciente en eventos del turno (`watcher.ftl`, `hud.ftl`):**
   - En `es-419` predomina el **pretérito perfecto simple** para acciones recién ocurridas durante el turno (*«Nos compraron un minuto más»*, *«La otra cuadrilla cobró antes»*).
   - En `es-ES` se emplea el **pretérito perfecto compuesto** para hechos ocurridos dentro del turno en curso (*«Nos habéis comprado un minuto más»*, *«La otra cuadrilla ha cobrado antes»*).
3. **Léxico técnico y de configuración (`game.ftl`, `hud.ftl`, `signs.ftl`):**
   - Diferencias léxicas consolidadas como **Riel** (`es-419`) frente a **Raíl** (`es-ES`) para el bloque `block-rail`, o **Video** (sin tilde, grave en `es-419`) frente a **Vídeo** (con tilde, esdrújula en `es-ES`) en `menu-video`.

### 5.2. Tabla documentada de divergencias aplicadas en `.ftl`

| Archivo y clave `.ftl` | Inglés (`en`) | Español de Latinoamérica (`es-419`) | Español de España (`es-ES`) | Motivo de la divergencia |
| :--- | :--- | :--- | :--- | :--- |
| `watcher-breathe-1` | `She's breathing. Get out.` | `Está respirando. Salgan de ahí.` | `Está respirando. Salid de ahí.` | Imperativo plural (`ustedes` vs. `vosotros`). |
| `watcher-breathe-5` | `Hear that slow breath?` | `¿Oyen esa respiración lenta?` | `¿Oís esa respiración lenta?` | Flexión verbal de 2.ª persona plural. |
| `watcher-restless-4` | `Pay the altar or run.` | `Paguen al altar o corran.` | `Pagad al altar o corred.` | Imperativo plural en alerta de radio. |
| `watcher-wake-2` | `Ninety seconds. Drop the tools.` | `Noventa segundos. Suelten las herramientas.` | `Noventa segundos. Soltad las herramientas.` | Orden de evacuación (`suelten` vs. `soltad`). |
| `watcher-wake-5` | `Run for the surface cage.` | `Corran a la jaula exterior.` | `Corred a la jaula exterior.` | Imperativo plural de emergencia. |
| `watcher-offering-3` | `You bought us a minute.` | `Nos compraron un minuto más.` | `Nos habéis comprado un minuto más.` | Pasado reciente + alocutivo plural. |
| `watcher-victory-1` | `Quota met. Lock the buckets.` | `Cuota cumplida. Cierren los cangilones.` | `Cuota cumplida. Cerrad los cangilones.` | Orden operativa al cierre del turno. |
| `watcher-victory-2` | `Contract paid. Come on up.` | `Contrato pagado. Suban a superficie.` | `Contrato pagado. Subid a superficie.` | Imperativo plural de ascenso. |
| `watcher-victory-4` | `Full haul. Clear the shaft.` | `Botín completo. Despejen el pozo.` | `Botín completo. Despejad el pozo.` | Imperativo plural de maniobra. |
| `watcher-defeat-1` | `Rival crew took the contract.` | `La otra cuadrilla cobró antes.` | `La otra cuadrilla ha cobrado antes.` | Pretérito simple vs. compuesto reciente. |
| `watcher-defeat-5` | `Too late. Shut it down.` | `Demasiado tarde. Apaguen todo ya.` | `Demasiado tarde. Apagadlo todo ya.` | Imperativo plural con pronombre enclítico. |
| `game.ftl` · `block-rail` | `Rail` | `Riel` | `Raíl` | Léxico ferroviario/minero estándar por región. |
| `hud.ftl` · `menu-video` | `Video` | `Video` | `Vídeo` | Acentuación normativa RAE según pronunciación. |
| `hud.ftl` · `menu-settings` | `Settings` | `Configuración` | `Ajustes` | Terminología estándar de menú por región. |
| `hud.ftl` · `hud-buckets-halted` | `Bucket lines halted — haul on foot!` | `¡Líneas de cangilones detenidas — suban el botín a pie!` | `¡Líneas de cangilones detenidas — subid el botín a pie!` | Imperativo plural (`ustedes` vs. `vosotros`). |
| `signs.ftl` · `memo-02-body` | `...Bolt SIGN-02 at every chute and dock the wages...` | `...Atornillen el cartel SIGN-02 en cada tolva y descuéntenle el jornal...` | `...Atornillad el cartel SIGN-02 en cada tolva y descontadle el jornal...` | Orden directiva de Sterling a sus subordinados. |
| `signs.ftl` · `memo-06-body` | `...blast the chamber roof and ride the bucket chain out.` | `...vuelen la bóveda de la cámara y salgan aferrados a la cadena de cangilones.` | `...volad la bóveda de la cámara y salid aferrados a la cadena de cangilones.` | Directiva urgente al turno de noche en cierre. |

---

## 6. Reglas de longitud y expansión de texto

El HUD de **HUSHMINE** simula monitores industriales e instrumentación de 1979 con cajas de tamaño fijo. Las lenguas romances y germánicas suelen expandirse entre un 20 % y un 35 % respecto al inglés, por lo que toda cadena localizada debe verificarse contra estos límites máximos de caracteres (contando espacios):

| Elemento de interfaz / archivo | Límite en inglés (`en`) | Límite en traducciones (`es-*` y otros) | Consecuencia de incumplimiento |
| :--- | :--- | :--- | :--- |
| Subtítulo de radio del Vigía (`watcher-*`) | **<= 6 palabras** (estricto) | **<= 48 caracteres** | Solapamiento o truncamiento en el recuadro inferior del HUD. |
| Botones de menú principal (`menu-*`) | <= 20 caracteres | **<= 28 caracteres** | Desbordamiento de las teclas biseladas de la consola de 1979. |
| Nombres de bloque y clase (`block-*`, `class-*`) | <= 18 caracteres | **<= 24 caracteres** | Corte en el selector radial de herramientas y en el inventario. |
| Avisos de umbral del Sismógrafo (`seismo-*`) | <= 12 caracteres | **<= 16 caracteres** | Desalineación sobre la escala graduada del tambor sísmico. |
| Descripción corta de tienda (Steam) | <= 260 caracteres | **<= 300 caracteres** | Rechazo o corte automático en la ficha de la tienda de Steam. |

Si una traducción literal supera el límite, el traductor debe condensar la estructura sintáctica (por ejemplo, suprimiendo artículos redundantes en etiquetas de HUD) sin alterar jamás los sustantivos del glosario canónico.

---

## 7. Guía para futuros traductores (`pt-BR`, `zh-Hans`, `ru`, `de`, `fr`, `ja`)

Conforme a la hoja de ruta de `PLAN_v4.md`, tras el inglés y el español (`es-419` y `es-ES`), **HUSHMINE** se localizará a los seis grandes mercados de Steam indicados a continuación. Cada equipo de traducción deberá aplicar estas directrices específicas:

### 7.1. Portugués de Brasil (`pt-BR`)

- **Doble sentido de *Hush / Offering*:** el título **HUSHMINE** no se traduce, pero en los textos de tienda y memorandos debe preservarse la tensión entre el silencio acústico (*silêncio na galeria*) y el pago para callar a la montaña (*Oferenda*, evitando *suborno* o *propina* como nombre de mecánica en el HUD, aunque puede insinuarse en la prosa de `memo-02`).
- **Género de *The Sleeper* y el pronombre *she*:** tradúzcase *The Sleeper* en masculino como **o Adormecido** (o **o Dormente**), evitando *a Adormecida*. En las líneas del Vigía con *she* (`watcher-stir-1`, `watcher-breathe-1`), utilícese sujeto elíptico (*«Está se mexendo...»*, *«Está respirando. Saiam daí.»*) para que pueda referirse tanto a *a montanha* como a *o Adormecido*.
- **Terminología minera:** *Bucket line* debe traducirse por su término técnico industrial real (**Linha de caçambas**), nunca *linha de baldes*; *Brass tally* como **Ficha de latão**.

### 7.2. Chino simplificado (`zh-Hans`)

- **Ausencia de plural gramatical y clasificadores:** en `fluent-rs`, el chino simplificado utiliza únicamente la rama `*[other]`. Al traducir cadenas con `{ $count }`, es obligatorio insertar el clasificador (量词) adecuado entre la variable y el sustantivo: `枚` para las fichas de latón (`{ $count } 枚黄铜工牌`) y `根` o `锭` para los lingotes de oro (`hud-gold-carried`):

```ftl
tally-board-progress =
    { $count ->
       *[other] 41 枚黄铜工牌中已归还 { $count } 枚至考勤板
    }
```

- **Doble sentido de *Hush* y concisión en `watcher.ftl`:** la brevedad del chino favorece enormemente la lectura periférica del Vigía (4 a 8 sinogramas por alerta). En la descripción de tienda, el doble sentido de *hush money* (封口费) y guardar silencio (噤声) puede reflejarse contraponiendo el silencio físico del pozo con el oro entregado al altar (**祭坛供奉** para *Offering*; **沉睡者** para *The Sleeper*).
- **Tipografía y nombres propios:** las fuentes **Noto Sans CJK** integradas en `grisu-render` cubren todo el repertorio. Los nombres de los 3 mineros chinos de 1931 deben restituirse con sus caracteres hanzi históricos exactos en `tally-board-*`, mientras que **Anima Consolidated** puede transliterarse o mantenerse acompañada de su forma original.

### 7.3. Ruso (`ru`)

- **Categorías de plural en Fluent (`[one]`, `[few]`, `[many]`, `*[other]`):** a diferencia del inglés y del español (que solo usan `[one]` y `*[other]`), toda plantilla `.ftl` en ruso con variables numéricas (`{ $count }`, `{ $seconds }`, `{ $blocks }`) **debe implementar las cuatro ramas CLDR** (`[one]` para 1, 21, 31...; `[few]` para 2–4, 22–24...; `[many]` para 0, 5–20, 25–30...; y `*[other]` para fracciones o respaldo) con la declinación de caso correspondiente:

```ftl
tally-board-progress =
    { $count ->
        [one] На доску возвращён { $count } из 41 латунного жетона
        [few] На доску возвращено { $count } из 41 латунного жетона
        [many] На доску возвращено { $count } из 41 латунного жетона
       *[other] На доску возвращено { $count } из 41 латунного жетона
    }
```

- **Género de *The Sleeper* vs. *she*:** *The Sleeper* se traduce como participio sustantivado masculino (**Спящий**), mientras que «montaña» (*гора*) y «mina» (*шахта*) son femeninos en ruso. En `watcher-stir-1` (*«She's stirring...»*), dado que el pasado verbal ruso marca género, debe emplearse el neutro impersonal (*«Внизу зашевелилось...»*) o referirse explícitamente a la montaña (*«Гора ворочается...»*) para no contradecir el masculino de **Спящий**.
- **Expansión en el HUD:** las palabras técnicas rusas en cirílico tienden a ser largas (*Сейсмограф*, *Ковшовая линия*). En botones de `menu-*` y etiquetas `hud-*`, utilícense sustantivos verbales cortos e imperativos perfectivos en las órdenes del Vigía.

### 7.4. Alemán (`de`)

- **Longitud de palabras compuestas y límites del HUD:** los compuestos nominales alemanes (*Eimerkettenseilbahn*, *Seismographennadel*, *Messingkontrollmarke*) pueden rebasar fácilmente el tope de 24–28 caracteres en el HUD. Se recomienda usar formas técnicas concisas e históricas del léxico minero alemán (*Eimerkette* o *Becherwerk* para `Bucket line`, *Fahrmarke* o *Messingmarke* para `Brass tally`, *Geophon* para `Geophone`).
- **Herencia histórica de los *kobolds* y del Harz:** Alemania es una de las tres cunas folclóricas del juego (4 mineros de 1931, incluido **Lukas Bergmann**, Ficha #6). El lema de la **Cobalt Crew** (*«Los mineros a los que los duendes no asustan»*) alude directamente a la etimología sajona de *Kobold* → *Kobalt*. La traducción alemana debe emplear el término histórico auténtico **Kobolde** y el vocabulario minero clásico del Erzgebirge y del Harz (*Stollen*, *Schacht*, *Erzgang*, **Der Schläfer** para *The Sleeper*).
- **Tratamiento alocutivo:** el Vigía se dirige a la cuadrilla con el plural cercano `ihr` (*«Hört ihr diesen langsamen Atem?»*, *«Lasst das Werkzeug fallen»*), mientras que los carteles `SIGN-01`..`SIGN-10` emplean el infinitivo imperativo industrial alemán (*«VOR DER EINFAHRT FAHRMARKE ABNEHMEN»*).

### 7.5. Francés (`fr`)

- **Expansión sintáctica y puntuación tipográfica:** el francés suele alargar las cadenas entre un 25 % y un 35 % y exige espacio fino irrompible (`\u00A0` o `\u202F`) antes de signos dobles (`:`, `;`, `!`, `?`, `%`) y dentro de comillas angulares (`« … »`). Para no desbordar los 48 caracteres en `watcher.ftl`, deben preferirse construcciones nominales secas de radio (*« Aiguille en zone rouge. »*, *« Quatre-vingt-dix secondes. Lâchez les outils. »*).
- **Vínculo con el motor Grisú y léxico minero:** el nombre del motor **Grisú** procede directamente del francés minero *grisou* (metano de hullera). El francés cuenta con una riquísima terminología de ingeniería de minas que debe aprovecharse con rigor: **le Dormeur** (*The Sleeper*), **Filon-mère** (*Mother Lode*), **Chaîne à godets** (*Bucket line*), **Jeton de présence en laiton** o **Méreau de laiton** (*Brass tally*), **Sapeur** (*Sapper*).
- **Género en `watcher.ftl`:** al igual que en español, *la montagne* es femenino y *le Dormeur* es masculino; en `watcher-stir-1` y `watcher-breathe-1` conviene usar *« Ça bouge là-dessous... »* o *« Elle respire. Sortez de là. »* (remitiendo implícitamente a *la montagne*).

### 7.6. Japonés (`ja`)

- **Respeto a la tradición del *Yama-no-Kami* y los mineros de 1931:** Japón aporta 2 mineros de alta especialización al turno de 1931 (como **Saburō Watanabe**, procedentes de las minas de cobre de Ashio y Besshi). Según se establece en `docs/lore/folclore.md`, **El Durmiente** (*The Sleeper*) **no debe traducirse como `山の神` (*Yama-no-Kami*)**, ya que eso lo confundiría con la deidad sintoísta real y rompería la regla de mitología 100 % original. Debe traducirse como una entidad geológica dormida: **眠れる者** (*Nemureru Mono*) o **眠れる巨山**, y *Offering* como **供物** (*Kumotsu*) o **奉納金** en su sentido secular-táctico.
- **Voz del Vigía sin pronombres de género ni plurales:** el japonés prescinde de selectores de plural (usa solo `*[other]` con contadores como `{ $count }枚` para las fichas y `{ $seconds }秒` para el reloj) y omite naturalmente el sujeto en las frases de radio. Esto resuelve con elegancia la ambigüedad de *«She's stirring...»* (`蠢いているぞ…` / `山が息をしている。脱出しろ。`), empleando un registro masculino veterano, seco y técnico de operador de 1979 (terminaciones en `-zo`, `-na`, e imperativos directos en emergencia) sin caer en jerga caricaturesca de anime.
- **Préstamos en katakana vs. kanji minero:** para los nombres de facción (**Copper Crew** / **Cobalt Crew**) se admite katakana (`カッパークルー` / `コバルトクルー`), pero los términos del glosario mecánico deben usar compuestos en kanji legibles de un vistazo en el HUD (`地震計` para *Seismograph*, `本脈` o `マザーロード` para *Mother Lode*, `心臓部` para *The Heart*, `真鍮の札` para *Brass tally*).

---

## 8. Lista de verificación antes de integrar un paquete `.ftl`

Antes de fusionar cualquier cambio en `assets/i18n/<locale>/`, el autor o traductor debe comprobar:

1. **Paridad de claves:** los cinco archivos (`game.ftl`, `hud.ftl`, `watcher.ftl`, `signs.ftl`, `chronicle.ftl`) contienen exactamente los mismos identificadores `kebab-case` que `assets/i18n/en/`, sin claves huérfanas ni ausentes.
2. **Paridad de variables y plurales:** toda variable `{ $var }` del original en inglés aparece en la traducción, y todo selector `{ $var -> ... }` incluye la rama por defecto `*[other]`.
3. **Cumplimiento del glosario e invariantes:** ningún término prohibido aparece en el texto y los nombres propios (**HUSHMINE**, **Grisú**, **Anima Consolidated**, **Copper Crew**, **Cobalt Crew** y los 41 mineros de 1931) conservan su grafía y diacríticos exactos.
4. **Límites de longitud y codificación:** ninguna línea excede los topes de la sección 6 y el archivo está guardado en UTF-8 con saltos de línea LF (`\n`).
