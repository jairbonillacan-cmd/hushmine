# ADR 0005: capa de red propia sobre UDP (y no `renet2` por ahora)

- **Fecha:** 23 de octubre de 2026
- **Estado:** aceptada; se revisa en el hito 5 (navegador y Steam)

## Contexto

El plan proponía `renet2` porque trae UDP, WebTransport y memoria. La condición de salida del hito 2 es que dos clientes y seis bots sigan consistentes con 150 ms de latencia y 2 % de pérdida. Para comprobarlo en CI hace falta simular la red de forma **determinista y con reloj virtual**, sin sockets ni esperas reales.

## Decisión

- **`grisu-net`**, propio y sin dependencias nuevas:
  - trait `Socket` (enviar y recibir paquetes con una dirección), con dos implementaciones: `SimNetwork` (memoria, con latencia, variación y pérdida deterministas) y UDP de la biblioteca estándar;
  - `Connection`: números de secuencia de 16 bits, acuses con campo de 32 bits, canal **fiable y ordenado** con fragmentación y reenvío según el RTT, y canal no fiable;
  - nada usa `Instant`: todo recibe `now_ms`, así que las pruebas avanzan el reloj a mano.
- **`hush-net`**: protocolo del juego, servidor y cliente como lógica pura (sin E/S), que reciben un `Socket`. El binario `hush-server` solo añade UDP y un bucle.
- **Serialización a mano** (`Writer`/`Reader` con errores, nunca `panic!`): los mensajes son pocos y pequeños. Se puede pasar a `bitcode` si crecen.
- **Mapa al unirse:** chunks en RLE por el canal fiable y, detrás, los cambios de bloques en el mismo canal, de modo que el orden garantiza la consistencia. LZ4 se medirá más adelante.
- **Predicción:** el cliente mueve a su minero con el mismo `PlayerBody::step` que el servidor y, al recibir el estado autoritativo, lo adopta y vuelve a aplicar las entradas aún no confirmadas. Las acciones (cavar, poner) viajan dentro de la entrada numerada de cada tick; las rechazadas producen una corrección.

## Consecuencias

- La prueba de consistencia corre en milisegundos y siempre da el mismo resultado.
- WebTransport y el relé de Steam no están resueltos: en el hito 5 se añadirán como nuevas implementaciones de `Socket`, propias o con `renet2`/`wtransport`, sin cambiar el protocolo.
- Mantener la capa de fiabilidad es responsabilidad nuestra; está cubierta por pruebas con pérdida y desorden.
