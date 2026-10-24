# Licencia y Autoría de la Interfaz y Marca (`assets/ui/`)

## Autoría y Origen

Todos los recursos gráficos, iconos, maquetas y emblemas contenidos en `assets/ui/` son **creaciones 100 % originales del proyecto HUSHMINE / Grisú**:

- **Iconos rasterizados (`assets/ui/icons/*_16.png` y `assets/ui/icons/*_32.png`):** 50 archivos PNG RGBA8 (25 identificadores canónicos en dos resoluciones: `16×16` y `32×32`) y su hoja de contacto (`out/ui_contact.png`), generados píxel a píxel sobre la paleta cerrada de 60 colores (`ALL_PALETTE_HEX`).
- **Maquetas de pantalla (`assets/ui/mockups/*.svg`):** 10 documentos vectoriales SVG correspondientes a las 5 pantallas del juego (`hud`, `drop_in`, `chronicle`, `main_menu`, `loading`) en las resoluciones `1920×1080` (escritorio 16:9) y `1280×800` (Steam Deck 16:10).
- **Identidad y marca (`assets/ui/brand/*.svg`):** 10 documentos vectoriales SVG que comprenden las 3 propuestas de logotipo de **HUSHMINE** (`logo_a_stencil.svg`, `logo_b_seismo.svg`, `logo_c_chasm.svg` en `800×240`), el icono cuadrado de la aplicación (`app_icon.svg` en `256×256`) y los 6 bocetos de cápsula de Steam centrados en el sabotaje (`capsule_1_shears_*`, `capsule_2_bridge_blast_*`, `capsule_3_split_heist_*` en `460×215` y `616×353`).
- **Generación procedural determinista:** la totalidad de los archivos se construye mediante código propio ubicado en `tools/ui/` (`ui_palette.py`, `icons.py`, `mockups.py`, `brand.py` y `build_all.py`), empleando exclusivamente la biblioteca estándar de Python 3.
- **Sin material propietario de terceros:** no se ha utilizado, calcado ni derivado ningún logotipo, icono, textura ni recurso con derechos de autor de terceros (incluyendo *Infiniminer* o cualquier otro título).

## Tipografía y Fuentes de Referencia

1. **Rótulos y logotipos en SVG (`assets/ui/brand/` y `assets/ui/mockups/`):**
   - Los logotipos principales utilizan trazos geométricos vectoriales (`<path>`, `<polygon>`, `<rect>`) diseñados específicamente para el repositorio, sin incrustar binarios tipográficos privativos.
2. **Pilas tipográficas de interfaz y distribución en el cliente (`PLAN_v4.md`):**
   - Las etiquetas de interfaz declaran pilas compatibles con las familias tipográficas de código abierto previstas para el cliente del juego, todas ellas distribuidas bajo licencia permisiva **SIL Open Font License 1.1 (`OFL-1.1`)**:
     - **Cabeceras industriales y rótulos técnicos:** *Chakra Petch* (`OFL-1.1`).
     - **Lecturas numéricas, reloj sísmico y monitores CRT:** *Share Tech Mono* y *JetBrains Mono* (`OFL-1.1`).
     - **Cuerpo de texto multilingüe y cobertura internacional (incluyendo CJK):** *Noto Sans* / *Noto Sans CJK* (`OFL-1.1`).

## Reproducibilidad

Cualquier copia del repositorio puede regenerar y verificar byte a byte todos los artefactos de interfaz y marca mediante los siguientes comandos:

```bash
python -B tools/ui/build_all.py
python -B tools/ui/build_all.py --check
```

## Licencia

Estos activos gráficos y vectoriales forman parte integrante del código fuente de **HUSHMINE / Grisú** y se distribuyen bajo los mismos términos y condiciones de licencia del repositorio principal del proyecto.
