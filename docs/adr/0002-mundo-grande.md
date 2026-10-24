# ADR 0002: preparar el motor para un mundo de 15 M × 15 M bloques

- **Fecha:** 24 de octubre de 2026
- **Estado:** propuesta
- **Plan detallado:** `docs/mundo/`

## Contexto

El futuro modo Historia (supervivencia) necesita un mundo procedural de 15 millones de bloques por lado, generado al explorar. El modo Turno usa mapas de 64³ y 96×96×64, y el hito 1 está a punto de fijar los tipos básicos de chunks, posiciones y colisión. Algunas decisiones de ese hito son baratas ahora y muy caras de cambiar cuando haya red, repeticiones y colisión encima.

## Decisión

1. **Posiciones de entidad en `i64` con 16 bits fraccionarios** (Q47.16). Q16.16 sobre `i32` solo alcanza ±32 767 bloques. Velocidades y escalares siguen en Q16.16 (`hush_sim::Fixed`).
2. **Bloques y chunks en `i32`**, con pruebas en coordenadas negativas y en los extremos del mundo.
3. **El chunk se usa a través de una interfaz** (`get`, `set`, `fill`). Por dentro empieza como `u8` plano y más adelante pasará a paleta con ids `u16`. Los ids `0..=17` quedan reservados para la Convención A.
4. **Render relativo a la cámara:** al shader solo llegan desplazamientos pequeños respecto al chunk de la cámara.
5. **El generador es una función pura** `(semilla, versión, posición del chunk) → chunk`, con ruido entero propio y estructuras colocadas por rejilla con radio acotado. Nada de estado compartido entre chunks.
6. **Se guardan solo los chunks modificados**, con `worldgen_version` en cada mundo y región.
7. **Nuevos crates cuando toque:** `grisu-noise`, `grisu-world` y `hush-worldgen`. Se mantiene la regla de dependencias.

Los puntos 1 a 4 forman el hito **W0** y entran en el hito 1. Los puntos 5 a 7 se aplican a partir de W1, después del acceso anticipado del modo Turno.

## Alternativas descartadas

- **Mantener Q16.16 y recentrar el origen por zonas:** complica la red, las repeticiones y la colisión entre zonas a cambio de ahorrar 4 bytes por coordenada.
- **Ruido en `f32` de un crate existente:** pierde precisión lejos del origen y no garantiza el mismo resultado en nativo y wasm, que hace falta para generar en el cliente y para regenerar chunks no guardados.
- **Generación con estado compartido (proto-chunks que se escriben entre sí):** depende del orden y es difícil de paralelizar y de probar.
- **Guardar todos los chunks generados:** sencillo, pero el disco crece con la exploración y no con el trabajo del jugador.

## Consecuencias

- **Ventajas:** el modo Turno no cambia y el mundo grande se puede añadir más adelante sin reescribir chunks, colisión ni render. El determinismo entero permite que el cliente genere el terreno y el servidor solo envíe diferencias.
- **Costes:** de 2 a 4 días en el hito 1; aritmética `i64` en la colisión (despreciable); un ruido propio que hay que escribir y afinar en W1.
- **Riesgo:** que el ruido entero dé artefactos visibles. Se mitiga con rotación entera, deformación de dominio y revisión con `hush-cli world map`.
