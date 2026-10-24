# Licencia y Autoría de las Texturas (`assets/textures/`)

## Autoría y Origen

Todas las texturas contenidas en `assets/textures/blocks/` (25 archivos PNG de
16×16 píxeles en formato RGBA8 sRGB) y su manifiesto de capas
(`assets/textures/blocks/manifest.ron`) son **arte pixel art 16×16 100 %
original del proyecto HUSHMINE / Grisú**.

- **Generación procedural determinista:** la totalidad de los píxeles y de la
  paleta cerrada de 48 colores se genera mediante código propio ubicado en
  `tools/textures/` (`palette.py`, `natural.py`, `built.py`, `png.py` y
  `build_all.py`), empleando exclusivamente la biblioteca estándar de Python 3.
- **Sin material de terceros:** no se ha utilizado, derivado ni incorporado
  ningún sprite, textura, paleta externa, paquete de recursos ni material con
  derechos de autor de terceros (incluyendo *Infiniminer* o cualquier otro
  título).

## Reproducibilidad

Cualquier copia del repositorio puede regenerar y verificar byte a byte todos
los artefactos gráficos mediante los siguientes comandos:

```bash
python -B tools/textures/build_all.py
python -B tools/textures/build_all.py --check
```

## Licencia

Estos activos gráficos forman parte integrante del código fuente de **HUSHMINE /
Grisú** y se distribuyen bajo los mismos términos y condiciones de licencia del
repositorio principal del proyecto.
