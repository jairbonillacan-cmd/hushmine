# ADR 0003: cámaras, animación por piezas, texturas de 32×32 y shaders propios

- **Fecha:** 24 de octubre de 2026
- **Estado:** propuesta
- **Plan detallado:** `docs/graficos/`

## Contexto

Hasta ahora el plan preveía texturas de 16×16, una cámara en primera persona y un renderizador sencillo. Para el Alfa y el modo Historia hacen falta personajes animados, primera y tercera persona, más detalle en las texturas y una iluminación que haga de la mina el centro del arte. Todo tiene que seguir funcionando en una Intel UHD 620 y en el navegador.

## Decisión

1. **Cámaras:** primera persona por defecto, tercera persona con brazo de cámara que choca con el terreno, y espectador. **En el Turno clasificatorio, solo primera persona.**
2. **Modelos por piezas rígidas** (cajas con pivote) en un formato RON propio, dibujados con instancias. Sin deformación de malla de momento.
3. **Animación en el cliente:** clips de fotogramas clave más capas procedurales (IK de pies, mirada, inercia), en un crate nuevo `grisu-anim` solo de CPU. La elección de clip la guían el estado y los eventos de `hush-sim`; nunca escribe en ella.
4. **Texturas de 32×32** como resolución canónica desde el hito 6, con paleta cerrada ampliada (~96 colores), generadas con el `tools/textures/` actual. **Su versión reducida a 16×16 tiene que pasar las pruebas de legibilidad actuales.** El motor lee el tamaño del manifiesto desde el hito 1.
5. **Luz de bloque en color** (RGB 4:4:4) + luz de cielo de 4 bits: 16 bits por bloque en chunks iluminados.
6. **Shaders WGSL propios** con un preprocesador mínimo sin dependencias, variantes por preset y validación de todas las variantes en las pruebas.
7. **Posproceso HDR** con adaptación a la oscuridad, bloom, mapeo de tonos, gradación por zona y escalado nítido. Sin antialias temporal.
8. **Accesibilidad visual del sonido:** indicadores de ruido en todos los presets.

## Alternativas descartadas

- **Mallas deformables (glTF y esqueletos con pesos) desde el principio:** más potentes, pero más caras de producir y de dibujar, y peores para que la IA las escriba y las revise. Se reservan para un jefe concreto si algún día hacen falta.
- **Texturas de 64×64:** pierden el estilo pixel art, multiplican el arte y no aportan legibilidad en penumbra.
- **Seguir en 16×16:** se queda corto para el modo Historia y para leer menas bajo la lámpara.
- **Un preprocesador o sistema de shaders externo:** con pocas decenas de variantes, uno propio de un archivo basta y evita una dependencia.
- **Antialias temporal:** emborrona el pixel art y añade fantasmas en movimiento rápido.

## Consecuencias

- **Ventajas:** el juego gana personajes y ambiente sin cambiar la simulación; todo se puede verificar con capturas, hojas de animación y tiempos por pase.
- **Costes:** 2–3 días en el hito 1 (G0) y unas 6–8 semanas repartidas entre los hitos 3 a 6.
- **Cambios en otros documentos:** la luz por bloque de `docs/mundo/01-escala-y-motor.md` pasa de 8 a 16 bits; `docs/art/guia-texturas.md` se actualizará a 32×32 en el hito 6.
- **Dependencias:** ninguna nueva.
