# Licencia y Autoría del Audio (`assets/audio/`)

## Autoría y Origen

Todos los efectos de sonido contenidos en `assets/audio/sfx/` (30 archivos WAV
PCM de 16 bits con signo little-endian, 44 100 Hz, mono) y su manifiesto de
efectos (`assets/audio/manifest.ron`) son **audio procedural 100 % original del
proyecto HUSHMINE / Grisú**.

- **Generación procedural determinista:** la totalidad de las muestras acústicas
  se sintetiza íntegramente mediante código propio ubicado en `tools/audio/`
  (`synth.py`, `sfx_tools.py`, `sfx_world.py`, `sfx_radio.py` y `build_all.py`),
  empleando únicamente la biblioteca estándar de Python 3 (`math`, `struct`,
  `wave`) y un generador congruencial lineal (LCG) determinista con semilla fija.
- **Sin material de terceros:** no se ha utilizado, derivado ni incorporado
  ninguna grabación externa, biblioteca de sonidos comercial o libre, ni
  muestras con derechos de autor de terceros (incluyendo *Infiniminer* o
  cualquier otro título).

## Reproducibilidad

Cualquier copia del repositorio puede regenerar y verificar byte a byte todos
los artefactos sonoros mediante los siguientes comandos:

```bash
python -B tools/audio/build_all.py
python -B tools/audio/build_all.py --check
```

## Licencia

Estos activos de audio forman parte integrante del código fuente de **HUSHMINE /
Grisú** y se distribuyen bajo los mismos términos y condiciones de licencia del
repositorio principal del proyecto.
