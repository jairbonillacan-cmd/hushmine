# Mundo grande · Plantillas de estructuras

Referencia de datos y herramientas del repositorio de plantillas: cómo se escriben
las piezas, cómo se rotan, cómo se ensamblan los planos y cómo se verifica todo.
Materializa el catálogo y las plantillas de [03-estructuras.md](03-estructuras.md)
(colocación por rejilla §2, planos por piezas §3, minas §4 y catálogo §5).

---

## 1. Propósito

Este repositorio de datos fija, antes de tocar Rust, el **formato de las plantillas**
de estructuras y el **ensamblado determinista** de planos:

- `data/estructuras/familias.ron`: catálogo de las 6 familias (rejilla, probabilidad,
  radio, anclaje, edad, raíz y biomas).
- `assets/estructuras/<familia>/<pieza>.ron`: 18 plantillas `Piece` con sus fills
  y conectores.
- `tools/estructuras/check_structures.py`: validador y ensamblador (equivalente en
  Python de `hush-cli world piezas --check`, §3 de 03-estructuras.md).
- `tools/estructuras/render_blueprints.py`: plano dibujado (planta y alzado) en PNG.
- `tools/estructuras/test_structures.py`: pruebas unitarias de todo lo anterior.

Cuando el generador en Rust (`hush-worldgen`) lea estas plantillas, seguirá las
mismas reglas que se documentan aquí; este conjunto de datos y sus scripts son la
especificación ejecutable.

## 2. Formato de plantilla `Piece`

```ron
Piece(
    id: "drift_gallery",          // igual al nombre del archivo .ron
    family: "ancient_mine",       // familia del catalogo
    is_cap: false,                // true en piezas de cierre
    size: [sx, sy, sz],           // dimensiones en bloques
    fills: [
        // [x0, y0, z0, x1, y1, z1, block_id]  (caja SEMIABIERTA)
        [0, 0, 0, 3, 1, 9, 2],    // suelo de roca
        [0, 1, 0, 3, 4, 9, 0],    // tunel excavado (0 = aire)
    ],
    connectors: [
        (pos: [1, 1, 0], dir: "north", kind: "track"),
    ],
)
```

- **Cajas semiabiertas:** la celda `(x, y, z)` pertenece a la caja si
  `x0 <= x < x1`, `y0 <= y < y1` y `z0 <= z < z1`; siempre `0 <= x0 <= x1 <= sx`
  (ídem `y` con `sy` y `z` con `sz`).
- **Celdas sin pintar = terreno original:** la plantilla no toca lo que no declara.
- **`block_id` 0 excava aire:** convierte la celda en hueco transitable
  (Convención A de `data/blocks.ron`, ids 0 a 17).
- **Orden de fills: la última gana.** Ante dos cajas que cubren la misma celda,
  el bloque que manda es el del `fill` más tarde en la lista (así un hueco
  declarado después recorta la roca de antes).

## 3. Coordenadas y direcciones

| Dirección | Cara exterior | Condición de `pos` |
|---|---|---|
| `north` | Norte | `pos.z == 0` |
| `south` | Sur | `pos.z == sz` |
| `west`  | Oeste | `pos.x == 0` |
| `east`  | Este  | `pos.x == sx` |
| `down`  | Abajo | `pos.y == 0` |
| `up`    | Arriba | `pos.y == sy` |

- Origen de la pieza: esquina `x = 0` (oeste), `y = 0` (abajo), `z = 0` (norte).
- `dir` apunta **hacia fuera** de la pieza y `pos` va **en** la cara.
- **Conectores compatibles:** mismo `kind` (`gallery`, `shaft`, `door`, `street`,
  `track`) y dirección opuesta (`north`↔`south`, `east`↔`west`, `up`↔`down`).

## 4. Política de cierres

Las piezas con `is_cap: true` forman un **pool global** (cualquier familia puede
usarlas) que tapa conectores al agotarse el hueco, el radio o el límite de piezas
(§3 de 03-estructuras.md, fase de cierre):

| Cierre | Kinds que tapa | Familia dueña |
|---|---|---|
| `collapse_cap` | `gallery`, `track`, `shaft` (arriba y abajo) | ancient_mine |
| `street_end`   | `street` (las cuatro horizontales) | arrival_village |
| `sealed_arch`  | `door` (las cuatro horizontales) | still_hamlet |

`collapse_cap` lleva conectores repetidos en posición con `kind` distinto
(`gallery` y `track` en las cuatro caras horizontales, `shaft` arriba y abajo)
porque un derrumbe corta por igual galerías, raíles y pozos: una sola pieza de
3×4×3 sirve de fondo de saco para los tres kinds. Los cierres **no encolan**
conectores nuevos: lo que sellan muere ahí.

## 5. Rotaciones

`r` pasos de 90° alrededor del eje `y` (0, 90, 180 y 270°). Con `size [sx, sy, sz]`:

| r | size rotado | Caja `[x0,y0,z0,x1,y1,z1]` → | `pos [x,y,z]` → | Direcciones |
|---|---|---|---|---|
| 0 | igual | igual | igual | igual |
| 1 | `[sz, sy, sx]` | `[sz - z1, y0, x0, sz - z0, y1, x1]` | `[sz - z, y, x]` | north→east, east→south, south→west, west→north; up/down fijos |
| 2 | igual | `[sx - x1, y0, sz - z1, sx - x0, y1, sz - z0]` | `[sx - x, y, sz - z]` | north↔south, east↔west; up/down fijos |
| 3 | `[sz, sy, sx]` | `[z0, y0, sx - x1, z1, y1, sx - x0]` | `[z, y, sx - x]` | north→west, west→south, south→east, east→north; up/down fijos |

En Python: `rotate_size`, `rotate_fill` y `rotate_connector` en
`tools/estructuras/check_structures.py`, todas enteras y puras.

## 6. Catálogo de familias

| Campo | ancient_mine | prospect_cut | consolidated_mine | arrival_village | still_hamlet | watchtower |
|---|---|---|---|---|---|---|
| `spacing_chunks` | 96 | 16 | 256 | 256 | 192 | 512 |
| `separation_chunks` | 16 | 4 | 24 | 32 | 24 | 48 |
| `probability_q16` | 32768 | 40960 | 24576 | 24576 | 16384 | 16384 |
| `max_radius_chunks` | 16 | 3 | 10 | 8 | 5 | 2 |
| `max_pieces` | 24 | 2 | 20 | 16 | 12 | 4 |
| `max_depth` | 320 | 24 | 200 | 16 | 64 | 24 |
| `anchor` | underground | surface | underground | surface | underground | surface |
| `age` | ancient | old | recent | recent | old | old |
| `root_piece` | headframe_entry | trench_entry | headframe_entry | lamp_house | quiet_plaza | seismo_post |

Justificación de las cifras, contra el catálogo de §5 de 03-estructuras.md:
`ancient_mine` (96 chunks = 1536 bloques, dentro de su banda de 1–2 km) y
`prospect_cut` (16 chunks = 256 bloques, tal cual el catálogo) salen directas;
`arrival_village` (256 chunks = 4 km) y `still_hamlet` (192 chunks = 3 km) caen
en sus bandas de 3–5 y 2–4 km; `watchtower` (512 chunks = 8 km) en la de 6–10 km;
`consolidated_mine` (256 chunks) acompaña a los pueblos porque comparte valles.
Las probabilidades Q16.16 usan 65536 = 100 %: 32768 = 50 %, 40960 = 62,5 %,
24576 = 37,5 % y 16384 = 25 %; los radios acotan cuántas celdas consulta cada
chunk (§2) y `max_pieces`/`max_depth` limitan el plano.

**Nota:** `consolidated_mine` reutiliza `headframe_entry` como raíz hasta tener
piezas propias (entibado de hierro, calderas, cangilones) en otro hito.

### Mapeo de biomas

| Id (inglés) | Bioma (español) |
|---|---|
| `mountain_range` | Cordillera |
| `needle_forest` | Bosque de agujas |
| `altiplano` | Altiplano |
| `red_desert` | Desierto rojo |
| `mesa_badlands` | Badlands con mesas |
| `river_valley` | Valle fluvial |
| `tundra` | Tundra y turberas |
| `inland_sea` | Lagos y mares interiores |

## 7. Inventario de las 18 piezas

Ids de bloque destacados: 0 aire, 1 tierra, 2 roca, 3 roca dura, 4 mena, 5 oro,
10 escalera, 12 eléctrico/bombas, 13 raíl, 14 baliza, 15 explosivo,
16 cangilones (vagoneta), 17 poste de cangilones.

### ancient_mine (9)

| Pieza | Papel | Tamaño | Conectores | Bloques destacados |
|---|---|---|---|---|
| `headframe_entry` | Raíz: castillete y bocamina | 9×14×9 | door north, shaft down | roca 2 |
| `main_shaft` | Pozo maestro apilable | 5×12×5 | shaft up, shaft down | escalera 10 |
| `shaft_station` | Estación de pozo por nivel | 11×5×7 | shaft up/down, track east/west, gallery north/south | raíl 13, baliza 14, vagoneta 16 |
| `drift_gallery` | Galería en dirección (sigue la veta) | 3×4×9 | track north/south | raíl 13, mena 4 |
| `crosscut_gallery` | Galería transversal hacia la veta | 9×4×3 | gallery west/east | — |
| `stope_chamber` | Rebaje con pilares | 11×6×9 | gallery west/east | oro 5 |
| `powder_room` | Polvorín apartado | 7×5×7 | gallery north | explosivo 15 |
| `pump_house` | Casa de bombas del nivel hondo | 7×5×5 | gallery north | bombas 12 |
| `collapse_cap` | **Cierre**: derrumbe | 3×4×3 | gallery ×4, track ×4, shaft up/down | tierra 1, roca 2 |

### prospect_cut (1)

| Pieza | Papel | Tamaño | Conectores | Bloques destacados |
|---|---|---|---|---|
| `trench_entry` | Raíz: trinchera de cateo | 9×4×5 | gallery west | tierra 1, mena 4 |

### arrival_village (4)

| Pieza | Papel | Tamaño | Conectores | Bloques destacados |
|---|---|---|---|---|
| `lamp_house` | Raíz: lampistería | 9×6×7 | door south, street east/west | baliza 14, banco 16 |
| `wooden_street` | Tramo de calle | 5×2×9 | street north/south | piedra 2 |
| `assay_workshop` | Taller de ensaye | 7×5×7 | door north, street south | horno 12, oro 5 |
| `street_end` | **Cierre**: explanada | 5×2×5 | street ×4 | piedra 2 |

### still_hamlet (3)

| Pieza | Papel | Tamaño | Conectores | Bloques destacados |
|---|---|---|---|---|
| `quiet_plaza` | Raíz: plaza de los Quedos | 11×5×11 | door north/south/east | roca dura 3 |
| `carved_dwelling` | Vivienda excavada | 7×4×7 | door north/south | mena 4 |
| `sealed_arch` | **Cierre**: arco tapiado | 5×4×3 | door ×4 | tierra 1, roca 2 |

### watchtower (1)

| Pieza | Papel | Tamaño | Conectores | Bloques destacados |
|---|---|---|---|---|
| `seismo_post` | Raíz: torre del Vigía | 7×21×7 | door south | sismógrafo 12, baliza 14 |

## 8. Ensamblado determinista

`check_structures.assemble_plan(seed, family_id, piece_limit)` resume lo que hará
el generador (§3 de 03-estructuras.md):

1. Radio del plano: `max_radius_chunks * 16` bloques; todo origen debe dejar la
   pieza dentro de `-radius..radius` en x y z (y sin límite).
2. RNG **splitmix64 entero** propio (`SplitMix64`), semilla 42; nada de
   `random` del sistema.
3. Se coloca la raíz con rotación 0 y origen `[-sx // 2, 0, -sz // 2]`.
4. Cola FIFO de conectores abiertos `(pieza, conector)`. Para cada uno se
   prueban las piezas normales de la familia en sus 4 rotaciones, emparejando
   el primer conector compatible (kind igual, dir opuesta) y apoyando el origen
   en la celda del conector del padre. Se descarta lo que se sale del radio o
   solape. La elección entre válidas sale de `rng.take_int`.
5. Sin candidata normal, se intenta el pool global de cierres; si tampoco cabe,
   el conector queda **sellado** (registrado con su celda y el motivo: ningún
   jugador lo verá abierto, el cierre lo tapa con derrumbe, explanada o arco).
6. Al alcanzar el límite de piezas, la cola restante se cierra solo con cierres
   o sellos.
7. Invariante **INV-W08**: `check_plan_overlap` re-verifica después que no hay
   solape estricto entre cajas colocadas; el contacto de caras es legal (así se
   emparejan los conectores).

El plano de prueba (semilla 42, `ancient_mine`, 12 piezas) coloca la raíz, otro
castillete, tramos de pozo, tres estaciones, dos galerías con raíles y deja el
resto de conectores sellados; todo dentro del radio de 256 bloques.

## 9. Cómo verificar

Desde la raíz del clone, los tres comandos deben terminar con código 0:

| Comando | Qué comprueba |
|---|---|
| `python tools/estructuras/check_structures.py` | Catálogo y piezas contra las reglas (§§ 2–5), cierres compatibles, ensambla el plano de prueba y verifica INV-W08 |
| `python tools/estructuras/render_blueprints.py` | Dibuja `out/estructuras/ancient_mine_plan.png` (planta X-Z y alzado X-Y con leyenda) y lo relee para verificarlo |
| `python -m unittest discover -s tools/estructuras -v` | 12 pruebas: carga, validez, caras de conectores, ids de bloque, cierres, raíces, ensamblado sin solapes, determinismo y PNG decodificable |

## 10. Convenciones de repositorio

- Archivos con saltos de línea **LF** y **400 líneas como máximo**.
- **Solo enteros**: `ron_lite` rechaza cualquier literal con punto decimal; el
  ensamblador y el renderizador tampoco usan `float` ni `random` del sistema.
- Comentarios, mensajes y documentación en **español**; identificadores en inglés.
- Prohibido `cargo` y `git` en estas herramientas: solo biblioteca estándar de
  Python 3 y los módulos de `tools/balance` y `tools/textures` en modo lectura.
