"""Plano dibujado de la mina de los Antiguos: planta y alzado en un solo PNG.

Ensambla el plano determinista (semilla 42, familia ancient_mine, limite de 12
piezas) con check_structures y lo dibuja con el codificador PNG y la fuente
5x7 de tools/textures/png.py. Toda la aritmetica es entera: la escala sale de
una division entera y los colores de una tabla fija de ids de bloque
(Convencion A de data/blocks.ron). Sin relojes ni aleatoriedad.

Uso: `python tools/estructuras/render_blueprints.py`
Salida: out/estructuras/ancient_mine_plan.png
"""

from __future__ import annotations

import sys
from pathlib import Path

TOOLS_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = TOOLS_ROOT.parent
for _ruta in (TOOLS_ROOT / "textures", TOOLS_ROOT / "balance", TOOLS_ROOT / "estructuras"):
    if str(_ruta) not in sys.path:
        sys.path.insert(0, str(_ruta))

import png  # noqa: E402
import check_structures as cs  # noqa: E402

SEED = 42
FAMILY_ID = "ancient_mine"
PIECE_LIMIT = 12
OUTPUT_PATH = REPO_ROOT / "out" / "estructuras" / "ancient_mine_plan.png"
TARGET_WIDTH = 640  # anchura objetivo del plano en pixeles (division entera)

# Colores enteros (r, g, b) por id de bloque, Convencion A.
BLOCK_COLORS = {
    0: (235, 235, 235),   # air
    1: (134, 96, 67),     # dirt
    2: (128, 128, 128),   # rock
    3: (90, 90, 95),      # hard_rock
    4: (214, 134, 48),    # ore
    5: (250, 216, 80),    # gold
    6: (108, 232, 222),   # diamond
    7: (230, 80, 30),     # lava
    8: (70, 130, 200),    # team_block
    9: (140, 200, 255),   # force_field
    10: (196, 156, 110),  # ladder
    11: (90, 200, 90),    # jump_pad
    12: (80, 160, 255),   # electric
    13: (70, 70, 75),     # rail
    14: (255, 240, 140),  # beacon
    15: (220, 60, 60),    # explosive
    16: (150, 110, 80),   # bucket_station
    17: (170, 130, 90),   # bucket_post
}
BACKGROUND = (32, 32, 36)
FRAME = (96, 96, 104)
TEXT_COLOR = (255, 255, 255)
# png.draw_text pide RGBA de 4 canales; los rellenos propios usan RGB de 3.
TEXT_RGBA = (255, 255, 255, 255)
MARGIN = 12
LEGEND_PER_ROW = 9
SWATCH = 12
SLOT_WIDTH = 56


def fill_rect(canvas: bytearray, canvas_w: int, canvas_h: int,
              x: int, y: int, w: int, h: int, rgb: tuple) -> None:
    """Rectangulo relleno propio con recorte seguro (alfa fijo 255)."""
    x0, y0 = max(0, x), max(0, y)
    x1, y1 = min(canvas_w, x + w), min(canvas_h, y + h)
    if x0 >= x1 or y0 >= y1:
        return
    row = bytes((rgb[0], rgb[1], rgb[2], 255)) * (x1 - x0)
    span = (x1 - x0) * 4
    for py in range(y0, y1):
        start = (py * canvas_w + x0) * 4
        canvas[start:start + span] = row


def paint_frame(canvas: bytearray, canvas_w: int, canvas_h: int,
                x: int, y: int, w: int, h: int, rgb: tuple) -> None:
    """Marco de 1 pixel alrededor de un panel."""
    fill_rect(canvas, canvas_w, canvas_h, x - 1, y - 1, w + 2, 1, rgb)
    fill_rect(canvas, canvas_w, canvas_h, x - 1, y + h, w + 2, 1, rgb)
    fill_rect(canvas, canvas_w, canvas_h, x - 1, y, 1, h, rgb)
    fill_rect(canvas, canvas_w, canvas_h, x + w, y, 1, h, rgb)


def plan_bounds(plan: dict, piezas: dict) -> tuple:
    """Limites (min_x, min_y, min_z, max_x, max_y, max_z) de las cajas rotadas."""
    xs: list = []
    ys: list = []
    zs: list = []
    for entry in plan["pieces"]:
        rs = cs.rotate_size(piezas[entry["id"]]["size"], entry["rotation"])
        ox, oy, oz = entry["origin"]
        xs += [ox, ox + rs[0]]
        ys += [oy, oy + rs[1]]
        zs += [oz, oz + rs[2]]
    return min(xs), min(ys), min(zs), max(xs), max(ys), max(zs)


def top_view_colors(plan: dict, piezas: dict) -> dict:
    """Por cada columna (x, z): id del bloque relleno con MAYOR y (cumbre)."""
    colores: dict = {}
    alturas: dict = {}
    for entry in plan["pieces"]:
        pieza = piezas[entry["id"]]
        ox, oy, oz = entry["origin"]
        for fill in pieza["fills"]:
            caja = cs.rotate_fill(fill, pieza["size"], entry["rotation"])
            x0, z0 = ox + caja[0], oz + caja[2]
            x1, y1, z1 = ox + caja[3], oy + caja[4], oz + caja[5]
            cima = y1 - 1
            for bz in range(z0, z1):
                for bx in range(x0, x1):
                    previa = alturas.get((bx, bz))
                    if previa is None or cima > previa:
                        alturas[(bx, bz)] = cima
                        colores[(bx, bz)] = caja[6]
    return colores


def side_view_colors(plan: dict, piezas: dict) -> dict:
    """Por cada celda (x, y): id del bloque relleno con MENOR z (desde el norte)."""
    colores: dict = {}
    cercanias: dict = {}
    for entry in plan["pieces"]:
        pieza = piezas[entry["id"]]
        ox, oy, oz = entry["origin"]
        for fill in pieza["fills"]:
            caja = cs.rotate_fill(fill, pieza["size"], entry["rotation"])
            x0, y0, z0 = ox + caja[0], oy + caja[1], oz + caja[2]
            x1, y1 = ox + caja[3], oy + caja[4]
            for by in range(y0, y1):
                for bx in range(x0, x1):
                    previa = cercanias.get((bx, by))
                    if previa is None or z0 < previa:
                        cercanias[(bx, by)] = z0
                        colores[(bx, by)] = caja[6]
    return colores


def render_plan_png() -> Path:
    """Ensambla el plano, dibuja planta, alzado y leyenda, y verifica el PNG."""
    plan = cs.assemble_plan(SEED, FAMILY_ID, PIECE_LIMIT)
    piezas = cs.load_pieces()
    min_x, min_y, min_z, max_x, max_y, max_z = plan_bounds(plan, piezas)
    span_x = max_x - min_x
    scale = max(1, TARGET_WIDTH // span_x)

    title = "PLANO MINA DE LOS ANTIGUOS - SEMILLA 42"
    title_scale = 2
    title_h = 7 * title_scale
    label_h = 13
    planta_w = span_x * scale
    planta_h = (max_z - min_z) * scale
    alzado_h = (max_y - min_y) * scale
    legend_rows = (len(BLOCK_COLORS) + LEGEND_PER_ROW - 1) // LEGEND_PER_ROW
    legend_h = legend_rows * (SWATCH + 8)
    canvas_w = MARGIN * 2 + max(planta_w, len(title) * 6 * title_scale,
                                LEGEND_PER_ROW * SLOT_WIDTH)
    head_h = MARGIN + title_h + 8 + label_h
    canvas_h = head_h + planta_h + 18 + label_h + alzado_h + 20 + legend_h + MARGIN
    canvas = bytearray(bytes((*BACKGROUND, 255)) * (canvas_w * canvas_h))

    png.draw_text(canvas, canvas_w, canvas_h, MARGIN, MARGIN, title, TEXT_RGBA,
                  scale=title_scale)

    # Panel PLANTA: vista superior X-Z, el norte (z menor) arriba.
    label_y = MARGIN + title_h + 8
    png.draw_text(canvas, canvas_w, canvas_h, MARGIN, label_y,
                  "PLANTA X-Z - NORTE ARRIBA", TEXT_RGBA, scale=1)
    planta_y = label_y + label_h
    paint_frame(canvas, canvas_w, canvas_h, MARGIN, planta_y, planta_w, planta_h, FRAME)
    colores = top_view_colors(plan, piezas)
    for bz in range(min_z, max_z):
        for bx in range(min_x, max_x):
            hallado = colores.get((bx, bz))
            color = BLOCK_COLORS[hallado] if hallado is not None else BACKGROUND
            fill_rect(canvas, canvas_w, canvas_h,
                      MARGIN + (bx - min_x) * scale, planta_y + (bz - min_z) * scale,
                      scale, scale, color)

    # Panel ALZADO: vista lateral X-Y proyectada desde el norte.
    alzado_label_y = planta_y + planta_h + 18
    png.draw_text(canvas, canvas_w, canvas_h, MARGIN, alzado_label_y,
                  "ALZADO X-Y - VISTA DESDE EL NORTE", TEXT_RGBA, scale=1)
    alzado_y = alzado_label_y + label_h
    paint_frame(canvas, canvas_w, canvas_h, MARGIN, alzado_y, planta_w, alzado_h, FRAME)
    colores = side_view_colors(plan, piezas)
    for by in range(min_y, max_y):
        fila = alzado_y + (max_y - 1 - by) * scale
        for bx in range(min_x, max_x):
            hallado = colores.get((bx, by))
            color = BLOCK_COLORS[hallado] if hallado is not None else BACKGROUND
            fill_rect(canvas, canvas_w, canvas_h,
                      MARGIN + (bx - min_x) * scale, fila, scale, scale, color)

    # Leyenda: 18 muestras de color con su id de bloque.
    legend_y = alzado_y + alzado_h + 20
    for block_id in sorted(BLOCK_COLORS):
        row, col = divmod(block_id, LEGEND_PER_ROW)
        ex = MARGIN + col * SLOT_WIDTH
        ey = legend_y + row * (SWATCH + 8)
        fill_rect(canvas, canvas_w, canvas_h, ex, ey, SWATCH, SWATCH, BLOCK_COLORS[block_id])
        png.draw_text(canvas, canvas_w, canvas_h, ex + SWATCH + 4, ey + 2,
                      str(block_id), TEXT_RGBA, scale=1)

    png.write_png(OUTPUT_PATH, canvas_w, canvas_h, bytes(canvas))
    ancho, alto, datos = png.read_png(OUTPUT_PATH)
    if (ancho, alto) != (canvas_w, canvas_h) or len(datos) != ancho * alto * 4:
        raise ValueError("el PNG releido no coincide con el plano dibujado")

    stats = plan["stats"]
    print(f"Plano escrito en {OUTPUT_PATH}")
    print(f"  dimensiones: {canvas_w}x{canvas_h} pixeles; escala {scale} px por bloque")
    print(f"  piezas colocadas: {stats['placed']} ({stats['caps']} de cierre); "
          f"conectores sellados: {stats['sealed']}")
    print(f"  extension del plano: x [{min_x}, {max_x}], "
          f"y [{min_y}, {max_y}], z [{min_z}, {max_z}]")
    return OUTPUT_PATH


def main() -> int:
    """Dibuja el plano determinista y devuelve 0."""
    render_plan_png()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
