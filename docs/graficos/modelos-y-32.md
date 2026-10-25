# Modelos y animación 32 px: minero, herramientas, criaturas y clips

Estado: entregado. Fuente de verdad: los cinco `.ron` de `assets/models/` y
`assets/anim/`, el generador `tools/models/tex32.py` y las verificaciones
`tools/models/check_models.py` y `tools/models/test_models.py`.

## 1. Propósito

Esta entrega define cómo se ven y se mueven los personajes del juego con la
rejilla de 32 px: el esqueleto del minero con sus enganches, las seis
herramientas acoplables, las tres criaturas del yacimiento, los clips de
animación (tercera y primera persona) y las texturas procedurales de 32×32
con su mip de 16×16. Todo son datos declarativos en RON de enteros, sin
flotantes: la simulación y el cliente pueden cargarlos de forma determinista.
El motor solo interpola; las reglas viven en `hush-sim` y los datos aquí.

Convenciones de espacio, comunes a todos los modelos:

- **32 px = 1 bloque**; todas las medidas (pivotes, cuboides, offsets, `pos`)
  son píxeles de textura, enteros.
- **+Y arriba** y el modelo **mira hacia -Z**; el origen va en el suelo, bajo
  el centro del modelo (salvo las herramientas, cuyo origen es el punto de
  agarre).
- `rot_deg = (pitch, yaw, roll)` en **grados enteros**; el pitch positivo
  inclina la pieza hacia delante.
- Las hojas de textura son de 128×128 y las regiones UV van en múltiplos de 4
  sin solaparse.

## 2. Formato RON de modelos

Raíz `Model` para un modelo suelto (`miner.ron`) o `ModelSet(models: [...])`
para varios (`tools.ron`, `creatures.ron`):

```ron
Model(
    name: "miner",              // identificador del modelo
    texture: "characters/miner",// hoja de 128x128 en assets/
    slot: None,                 // Some("tool") o Some("lamp") en tools.ron
    parts: [ Part(...) ],       // piezas rígidas del esqueleto
    attachments: [ Attachment(...) ], // enganches para otros modelos
    team_tint_mask: Some("team_mask"),// región UV teñida por equipo
)
```

- `Part(id, parent, pivot, cuboid, uv)`: pieza rígida. `parent` es
  `Some("id")` o `None` (raíz). `pivot` es el punto de giro en espacio del
  modelo, `cuboid` son las esquinas `min`/`max` de la caja (siempre
  `min < max` componente a componente) y `uv` la esquina de su región en la
  hoja.
- `Attachment(id, part, offset)`: punto donde se acopla otro modelo, definido
  por la pieza que lo ancla y un offset en píxeles.

## 3. Esqueleto del minero (14 piezas) y enganches (3)

Raíz única `pelvis`; cadena más larga de 4 saltos. Lados en `-X` (izquierdo)
y `+X` (derecho).

| Pieza | Padre | Pivote (px) | Papel |
|---|---|---|---|
| `pelvis` | — (raíz) | (0, 12, 0) | cadera |
| `torso` | pelvis | (0, 14, 0) | pecho |
| `head` | torso | (0, 24, 0) | cabeza con casco |
| `upper_arm_l` / `upper_arm_r` | torso | (∓6, 22, 0) | hombro |
| `lower_arm_l` / `lower_arm_r` | `upper_arm_*` | (∓6, 16, 0) | codo |
| `hand_l` / `hand_r` | `lower_arm_*` | (∓6, 10, 0) | muñeca |
| `upper_leg_l` / `upper_leg_r` | pelvis | (∓2, 10, 0) | cadera de pierna |
| `lower_leg_l` / `lower_leg_r` | `upper_leg_*` | (∓2, 6, 0) | rodilla |
| `pack` | torso | (0, 19, 2) | saco de carga, a la espalda (+Z) |

Enganches: `lamp → head` offset (0, 6, 0) para la lámpara de casco,
`tool → hand_r` offset (0, -3, -2) para la herramienta en mano y
`back → torso` offset (0, 2, 3) para el bloque cargado.

## 4. Herramientas y criaturas

Las seis herramientas comparten la hoja `characters/tools`, miran a -Z desde
el punto de agarre y caben en `z` en (-22, 6) para no atravesar paredes al
acoplarse a un enganche.

| Modelo | Slot | Piezas | Descripción |
|---|---|---|---|
| `pickaxe` | tool | 2 | pico: mango con cabeza en T al fondo |
| `shovel` | tool | 2 | pala: mango largo y hoja ancha |
| `shears` | tool | 2 | cizalla: dos brazos que se abren |
| `geophone` | tool | 3 | geófono: caja, esfera y sonda frontal |
| `satchel_charge` | tool | 2 | carga de petardo con mecha |
| `helmet_lamp` | lamp | 2 | lámpara de casco con cristal emisivo |

Las criaturas usan una textura propia cada una y animan por piezas igual que
el minero:

| Modelo | Piezas | Descripción |
|---|---|---|
| `whistler` | 7 | cuadrúpedo lanudo: cuerpo, cabeza, cola y 4 patas |
| `ore_beetle` | 5 | escarabajo de mena: caparazón, cuerno y 2 bandas de patas |
| `stone_stalker` | 7 | bípedo acechante: torso, cabeza, brazos, piernas y cola |

## 5. Formato RON de clips

Raíz `ClipSet(clips: [...])` en `assets/anim/miner_clips.ron` (tercera
persona) y `first_person_clips.ron` (primera persona):

```ron
Clip(
    name: "dig",
    duration_ms: 600,          // entero, en milisegundos
    loop: false,               // los de ciclo cierran: última clave = primera
    tracks: [ Track(part: "upper_arm_r", keys: [
        Key(t_ms: 0, rot_deg: (0, 0, 15)),              // sin pos: solo giro
        Key(t_ms: 360, rot_deg: (8, 14, 0), pos: Some((0, -1, 0))),
    ]) ],
    markers: [ Marker(kind: "impact", t_ms: 360) ],     // hitos para sonido/lógica
    reach_boxes: [ Box(id: "arm_r", min: (1, -14, -20), max: (13, 4, 6)) ],
)
```

- `Track`: una por pieza animada; sus `Key` van ordenadas por `t_ms` sin
  duplicados, con `0 <= t_ms <= duration_ms`. Entre claves el motor interpola
  linealmente.
- `Key`: `rot_deg` en grados enteros y `pos` opcional (`Some((x, y, z))`) en
  píxeles, desplazamiento relativo al pivote de la pieza.
- `Marker`: `kind` ∈ {`impact`, `step_l`, `step_r`, `release`} con
  `0 < t_ms < duration_ms`; disparan sonido de golpe, pasos y suelta de bloque.
- `Box` (solo primera persona): `reach_boxes` con `id`, `min` y `max`, la caja
  conservadora que contiene todo el barrido del brazo o la herramienta.

Clips de tercera persona incluidos: `idle` (1200 ms, ciclo), `walk` (800,
ciclo), `run` (560, ciclo), `crouch` (1000, ciclo), `dig` (600, impact 360),
`place` (400, release 260), `ladder` (720, ciclo), `carry` (800, ciclo) y
`geophone_listen` (2000, ciclo). Los de ciclo marcan `step_l`/`step_r` a
mitad de zancada.

Clips de primera persona: `idle_1p` (1600 ms, ciclo), `dig_1p` (600, impact
360), `place_1p` (400, release 260) y `carry_1p` (800, ciclo). Animar solo
`arm_l`/`arm_r` (modelo propio de brazos) y, en el espacio de cámara, el
origen es el ojo.

**Regla de primera persona (no atravesar paredes):** en toda `reach_boxes`,
`min < max` componente a componente y `min[2] >= -24`: nada del brazo o de la
herramienta pasa de 24 px delante de la cámara, así el golpe visual nunca
atraviesa el bloque que estás picando.

## 6. Texturas 32×32 y la regla del mip de 16×16

`tools/models/tex32.py` genera las 7 texturas naturales (`dirt`, `rock`,
`hard_rock`, `ore`, `gold`, `diamond`, `lava`) con la paleta cerrada de
`tools/textures/palette.py`, aritmética entera y continuidad toroidal.
Expone `generate_textures() -> dict[str, bytes]` (buffers RGBA8 de 32×32) y
`downsample_2x2(bytes) -> bytes` (mip de 16×16).

**La regla del mip** (docs/graficos/03-texturas-32.md, §4): cada textura se
reduce a 16×16 promediando cada bloque 2×2 **en luz lineal** (tablas enteras
sRGB↔lineal, división con redondeo) y ajustando el resultado al **color de
paleta más cercano** (distancia RGB²; desempate: primer color insertado en la
paleta). El mip debe superar las mismas pruebas que la textura grande:

- **Paleta cerrada:** todo píxel, en 32×32 y en mip, está en `PALETTE_SET`.
- **Bandas de luminancia:** la luminancia media de cada bloque dentro de su
  banda en `LUMINANCE_BANDS`.
- **Distinción contra `rock`:** distancia de luminancia >= 15 contra `gold`,
  `ore`, `diamond` y `lava`, en visión normal, protanopía y deuteranopía.
- **Mosaico toroidal:** la costura del borde no supera 1.5 veces la variación
  interior, en ambos ejes y en ambas escalas.

Se comprueba con `verify_mip_rule() -> (ok, errores)` en `tex32.py`, que
llaman tanto el script como las pruebas (véase el punto siguiente) y cuya
lógica no se duplica en ningún otro sitio.

## 7. Cómo ejecutar las verificaciones

Desde la raíz del repositorio (las tres deben terminar con código 0):

```bash
python tools/models/check_models.py          # informe [OK]/[FALLO] de todo
python tools/models/tex32.py                 # escribe out/tex32/*.png + hoja de contacto
python -m unittest discover -s tools/models -v
```

`check_models.py` valida los cinco `.ron` (sin decimales, conjuntos exactos
de ids, árboles sin ciclos, claves y markers en rango, cajas de primera
persona) y lanza la verificación del mip de `tex32`. `tex32.py`, además de
verificar, escribe los PNG para revisión visual: abre `out/tex32_contact.png`
y mírala. Las pruebas generan las texturas en memoria y no dependen de `out/`.

## 8. Inventario de la entrega

| Archivo | Contenido |
|---|---|
| `assets/models/miner.ron` | esqueleto del minero: 14 piezas y 3 enganches |
| `assets/models/tools.ron` | 6 herramientas (5 slot `tool`, 1 slot `lamp`) |
| `assets/models/creatures.ron` | 3 criaturas del yacimiento |
| `assets/anim/miner_clips.ron` | 9 clips de tercera persona |
| `assets/anim/first_person_clips.ron` | 4 clips de primera persona con `reach_boxes` |
| `tools/models/tex32.py` | generador determinista de texturas 32×32 y mip 16×16 |
| `tools/models/check_models.py` | verificador completo de la entrega (exit 0/1) |
| `tools/models/test_models.py` | batería unittest (18 pruebas) |
| `docs/graficos/modelos-y-32.md` | este documento |
