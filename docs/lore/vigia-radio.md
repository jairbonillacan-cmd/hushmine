# Guion de Radio del Vigía (1979)

## Perfil del personaje: El Vigía (*The Watcher*)

En la superficie de la concesión de **HUSHMINE**, lejos del polvo de carbón y del hedor a azufre del pozo, una única caseta de chapa aguanta el viento frío de 1979. Dentro trabaja **el Vigía** (*The Watcher*), un operador veterano de rescate minero que de niño ya escuchó cómo se cerraba la montaña en la noche de 1931. Nunca baja en la jaula. Su turno transcurre frente a un banco de válvulas calientes, un micrófono de baquelita con pulsador metálico y el tambor de papel continuo del sismógrafo acoplado a un tubo de fósforo verde.

El Vigía no pertenece a ninguna de las dos cuadrillas que compiten abajo. Desde la estación telegráfica lee la roca y canta los avisos por la frecuencia abierta de onda corta que comparten ambos equipos. Su voz suena seca, gastada por el tabaco y recortada por el chasquido del relé de transmisión: habla como quien sabe que cada palabra extra en el canal puede tapar el crujido de una viga o el aviso de los Silbadores. Cuando se refiere a la montaña o a la presencia que habita bajo la falla, a veces la llama *el Durmiente* (*the Sleeper*) y a veces simplemente *ella* (*she*), como hacen los mineros viejos cuando no quieren nombrar lo que tienen debajo de las botas.

### Dirección de voz y progresión dramática

- **Calma instrumental al inicio (`Revuelo`):** tono rutinario y clínico, con la vista clavada en la aguja y el cigarrillo apoyado en el borde del cenicero metálico.
- **Tensión contenida a mitad de escala (`Respira`, `Veta Madre`):** la voz baja media octava; el operador reconoce el patrón térmico de 1931 pero mantiene la disciplina de radio.
- **Alarma seca en zona roja (`Inquietud`, `Ofrenda`, `Ficha devuelta`):** frases cortadas al soltar el pulsador PTT; alivio breve y cansado cuando el altar traga oro o recupera una de las 41 fichas de latón.
- **Urgencia absoluta en el colapso (`Despertar`, `Corazón`):** sin dramatismo teatral ni gritos descontrolados; órdenes directas de evacuación mientras el papel del tambor se rasga contra el tope.
- **Cierre de turno (`Victoria`, `Derrota`):** frialdad contable al sellar el pozo, tanto si la línea de cangilones subió la cuota como si la montaña se quedó con el botín.

## Convenciones técnicas del Sismógrafo y del turno

Todas las intervenciones por radio del Vigía responden de forma determinista a los eventos de `hush-sim`, sin depender de temporizadores flotantes ni de lógica en el cliente:

- **Escala entera del Sismógrafo (`0` a `100_000`):** la tensión sísmica se mide en milésimas de punto porcentual (`u32`), donde `1_000` equivale al 1 % y `100_000` al 100 % (explosión `+3_000`, descarga eléctrica o lava que fluye `+200`, línea de cangilones activa `+1` por tick).
- **Umbral de `Revuelo` (`25_000` / 25 % — *Stir*):** el papel registra las primeras sacudidas anómalas por encima del ruido de pico y barreno; caen guijarros sueltos de las bóvedas, avisados por **los Silbadores** (*the Whistlers*).
- **Umbral de `Respira` (`50_000` / 50 % — *Breathe*):** una exhalación cálida recorre las galerías y la presión interna fractura la roca hasta abrir una **Veta Madre** (*Mother Lode*).
- **Umbral de `Inquietud` (`75_000` / 75 % — *Restless*):** el trazo entra en zona roja, se rompen bolsas de lava y la única forma de retrasar el colapso es llevar oro o una **ficha de latón** (*brass tally*) de 1931 al altar como **ofrenda** (*offering*).
- **Umbral de `Despertar` (`100_000` / 100 % — *Wake*):** la aguja golpea el tope mecánico y se abre la cámara profunda donde late **el Corazón** (*The Heart*) rodeado de lava.
- **Reloj de cierre a 30 ticks por segundo:** la secuencia del Despertar dura exactamente `2_700` ticks (90 segundos reales). Durante los primeros `1_800` ticks (60 segundos) las **líneas de cangilones** (*bucket lines*) siguen subiendo mineral a toda marcha; en los últimos `900` ticks (30 segundos finales) los motores de cangilones se bloquean y solo queda escapar hacia la superficie antes de que la galería quede sepultada.

## Reglas de localización y Project Fluent

El sistema de textos y subtítulos utiliza **Project Fluent** (`.ftl`) con el **inglés como idioma fuente** canónico del repositorio y traducción oficial al español desde el primer corte vertical:

- **Regla de brevedad (máximo 6 palabras en inglés):** cada línea fuente en inglés tiene un tope estricto de seis palabras separadas por espacios. En plena partida competitiva, cuando el jugador está picando oro, vigilando a la cuadrilla rival y huyendo de un derrumbe, el subtítulo debe leerse de un solo vistazo periférico en menos de un segundo.
- **Margen de expansión lingüística:** al limitar el original inglés a entre 2 y 5 palabras (nunca más de 6), las traducciones a idiomas romances como el español mantienen una longitud compacta que no desborda el recuadro del HUD ni se solapa con el siguiente evento del sismógrafo.
- **Identificadores estables (`watcher-<slug>-<n>`):** cada variante posee una clave inmutable en Fluent que permite seleccionar la línea de forma determinista a partir de `hush_sim::Rng` y asociarle en el futuro su archivo de audio correspondiente sin acoplar la simulación al texto.
- **Glosario canónico compartido:** las traducciones respetan siempre los términos fijados para el proyecto (*el Durmiente*, *Veta Madre*, *el Corazón*, *ficha de latón*, *ofrenda*, *línea de cangilones*), evitando sinónimos ambiguos que confundan las mecánicas de juego.
- **Sin modismos intraducibles:** el vocabulario evita juegos de palabras locales y se apoya en imágenes físicas universales de la minería de 1979 (aguja, tambor, jaula, pozo, cangilones, latón, oro, piedra y lava).

## Tabla maestra de líneas de radio

| Evento | ID | Inglés | Español |
| --- | --- | --- | --- |
| Revuelo | watcher-stir-1 | She's stirring... | Se está removiendo... |
| Revuelo | watcher-stir-2 | The needle just twitched. | La aguja acaba de temblar. |
| Revuelo | watcher-stir-3 | Something moved down there. | Algo se movió ahí abajo. |
| Revuelo | watcher-stir-4 | First tremor on the drum. | Primer temblor en el tambor. |
| Revuelo | watcher-stir-5 | Easy with those charges. | Cuidado con esas cargas. |
| Respira | watcher-breathe-1 | She's breathing. Get out. | Está respirando. Salgan de ahí. |
| Respira | watcher-breathe-2 | Warm air from the shaft. | Sube aire caliente del pozo. |
| Respira | watcher-breathe-3 | The mountain just exhaled. | La montaña acaba de exhalar. |
| Respira | watcher-breathe-4 | Seismograph passed fifty percent. | El sismógrafo pasó del cincuenta. |
| Respira | watcher-breathe-5 | Hear that slow breath? | ¿Oyen esa respiración lenta? |
| Inquietud | watcher-restless-1 | The Sleeper grows restless. | El Durmiente se inquieta. |
| Inquietud | watcher-restless-2 | Supports are groaning now. | Las entibaciones ya están crujiendo. |
| Inquietud | watcher-restless-3 | Red zone on the trace. | Zona roja en el trazo. |
| Inquietud | watcher-restless-4 | Pay the altar or run. | Paguen al altar o corran. |
| Inquietud | watcher-restless-5 | Stone is falling everywhere. | Cae piedra por todas partes. |
| Despertar | watcher-wake-1 | The Sleeper is awake. | El Durmiente ha despertado. |
| Despertar | watcher-wake-2 | Ninety seconds. Drop the tools. | Noventa segundos. Suelten las herramientas. |
| Despertar | watcher-wake-3 | Needle pinned to the stop. | Aguja clavada contra el tope. |
| Despertar | watcher-wake-4 | Bucket lines stop in sixty. | Cangilones paran en sesenta segundos. |
| Despertar | watcher-wake-5 | Run for the surface cage. | Corran a la jaula exterior. |
| Ofrenda | watcher-offering-1 | The altar took the gold. | El altar aceptó el oro. |
| Ofrenda | watcher-offering-2 | Offering accepted. Tremors easing. | Ofrenda aceptada. Bajan los temblores. |
| Ofrenda | watcher-offering-3 | You bought us a minute. | Nos compraron un minuto más. |
| Ofrenda | watcher-offering-4 | The needle is dropping back. | La aguja vuelve a bajar. |
| Ofrenda | watcher-offering-5 | Gold quieted the deep. | El oro calmó la hondura. |
| Ficha devuelta | watcher-tally-1 | Brass tally returned to altar. | Ficha de latón devuelta abajo. |
| Ficha devuelta | watcher-tally-2 | One dead miner rests easier. | Un minero muerto descansa mejor. |
| Ficha devuelta | watcher-tally-3 | Thirty-one knew that number. | El treinta y uno recordaba. |
| Ficha devuelta | watcher-tally-4 | Tally paid. The rock settles. | Ficha entregada. La roca cede. |
| Ficha devuelta | watcher-tally-5 | Another brass name comes home. | Otro nombre de latón regresa. |
| Veta Madre | watcher-mother-lode-1 | A Mother Lode cracked open. | Se abrió una Veta Madre. |
| Veta Madre | watcher-mother-lode-2 | Rich cavern open down below. | Caverna rica abierta ahí abajo. |
| Veta Madre | watcher-mother-lode-3 | Pure gold in the fault. | Oro puro en la falla. |
| Veta Madre | watcher-mother-lode-4 | Both crews see that vein. | Ambas cuadrillas ven esa veta. |
| Veta Madre | watcher-mother-lode-5 | Fresh cavity full of ore. | Cavidad nueva llena de mineral. |
| Corazón | watcher-heart-1 | The Heart is exposed. | El Corazón ha quedado expuesto. |
| Corazón | watcher-heart-2 | Lava around the core deposit. | Lava alrededor del depósito central. |
| Corazón | watcher-heart-3 | The Heart beats in magma. | El Corazón late entre magma. |
| Corazón | watcher-heart-4 | Core chamber open. Watch lava. | Cámara central abierta. Hay lava. |
| Corazón | watcher-heart-5 | Last fortune in the furnace. | Última fortuna en el horno. |
| Victoria | watcher-victory-1 | Quota met. Lock the buckets. | Cuota cumplida. Cierren los cangilones. |
| Victoria | watcher-victory-2 | Contract paid. Come on up. | Contrato pagado. Suban a superficie. |
| Victoria | watcher-victory-3 | Winning bucket line locked in. | Línea de cangilones ganadora cerrada. |
| Victoria | watcher-victory-4 | Full haul. Clear the shaft. | Botín completo. Despejen el pozo. |
| Victoria | watcher-victory-5 | Shift won. Cage is waiting. | Turno ganado. Espera la jaula. |
| Derrota | watcher-defeat-1 | Rival crew took the contract. | La otra cuadrilla cobró antes. |
| Derrota | watcher-defeat-2 | Shaft sealed. Nothing comes up. | Pozo sellado. Ya nada sube. |
| Derrota | watcher-defeat-3 | Empty buckets. Shift is over. | Cangilones vacíos. Terminó el turno. |
| Derrota | watcher-defeat-4 | The mountain kept the gold. | La montaña guardó el oro. |
| Derrota | watcher-defeat-5 | Too late. Shut it down. | Demasiado tarde. Apaguen todo ya. |

## Criterios de disparo e integración en el cliente

Para mantener la separación estricta entre `hush-sim` y `hush-client`, la reproducción de las líneas del Vigía sigue estas pautas:

- **Selección determinista de variante:** cuando `hush-sim` emite un evento de umbral o de altar en el registro del tick, incluye el índice de variante (`1..=5`) extraído de `hush_sim::Rng`, de modo que ambos equipos y cualquier repetición (*replay*) escuchen exactamente la misma transmisión.
- **Prioridad de canal en onda corta:** los eventos de cambio de fase del sismógrafo (`Revuelo`, `Respira`, `Inquietud`, `Despertar`) y los de fin de partida (`Victoria`, `Derrota`) interrumpen cualquier aviso menor en curso con un corte de estática de 150 ms.
- **Cola de eventos simultáneos:** si una apertura de `Veta Madre` coincide en el mismo tick con el umbral de `Respira`, el cliente emite primero la alerta de umbral y encola el aviso de `Veta Madre` con una pausa de 600 ms.
- **Cohesión con el efecto de audio del Vigía:** todas las líneas pasan por un filtro paso banda de radio analógica (300 Hz a 3 kHz) acompañado del golpe seco del conmutador PTT (*push-to-talk*) al abrir y cerrar la portadora.

## Correspondencia con archivos `.ftl`

Cuando se integre el catálogo de localización en `hush-client`, las entradas de la tabla se volcarán directamente en dos recursos simétricos:

- `locales/en/watcher.ftl`: contiene las 50 claves `watcher-<slug>-<n>` con el texto de la columna `Inglés` tal cual figura en este documento.
- `locales/es/watcher.ftl`: contiene las mismas 50 claves con el texto de la columna `Español`, garantizando cobertura completa sin recurrir a cadenas de respaldo durante el juego.

### Verificación automática del catálogo

El banco de pruebas de documentación e internacionalización comprueba tres invariantes sobre esta tabla maestra:

- Presencia exacta de los 10 eventos obligatorios del turno minero con entre 4 y 6 variantes por evento.
- Unicidad absoluta de cada identificador `watcher-<slug>-<n>` en las 50 filas.
- Cumplimiento del límite de 6 palabras como máximo en todas las frases en inglés, asegurando subtítulos legibles en cualquier resolución de ventana.

