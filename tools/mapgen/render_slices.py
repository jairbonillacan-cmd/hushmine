"""Visualizador determinista de cortes 2D y mapa de alturas para HUSHMINE.

Renderiza cortes horizontales (X×Z), cortes verticales Este-Oeste (X×Y) y
Norte-Sur (Z×Y), así como un mapa de alturas superior con estructuras clave
(bases Copper/Cobalt, Veta Madre y Corazón), exportando imágenes PNG RGBA8
mediante el codificador puro de `tools/textures/png.py` y la paleta cerrada
de `tools/textures/palette.py`.
"""

from __future__ import annotations

import argparse
from pathlib import Path
import sys

_REPO_ROOT = Path(__file__).resolve().parents[2]
if str(_REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT))

from tools.mapgen.generate import (  # noqa: E402
    AIR,
    BEACON,
    BLOCK_NAMES,
    BUCKET_POST,
    BUCKET_STATION,
    DIAMOND,
    DIRT,
    ELECTRIC,
    EXPLOSIVE,
    FORCE_FIELD,
    GOLD,
    HARD_ROCK,
    JUMP_PAD,
    LADDER,
    LAVA,
    ORE,
    RAIL,
    ROCK,
    TEAM_BLOCK,
    GeneratedMap,
    generate_map,
)
from tools.textures.palette import PALETTE  # noqa: E402
from tools.textures.png import draw_text, encode_png, read_png, write_png  # noqa: E402

# Mapa canónico de colores RGBA8 por identificador de bloque usando PALETTE.
BLOCK_COLOR_MAP: dict[int, tuple[int, int, int, int]] = {
    AIR: PALETTE["void_black"],
    DIRT: PALETTE["ochre_dirt"],
    ROCK: PALETTE["slate_mid"],
    HARD_ROCK: PALETTE["basalt_deep"],
    ORE: PALETTE["ore_amber"],
    GOLD: PALETTE["gold_gleam"],
    DIAMOND: PALETTE["diamond_cyan"],
    LAVA: PALETTE["lava_orange"],
    TEAM_BLOCK: PALETTE["steel_mid"],
    FORCE_FIELD: PALETTE["copper_glow"],
    LADDER: PALETTE["timber_light"],
    JUMP_PAD: PALETTE["sodium_warm"],
    ELECTRIC: PALETTE["electric_blue"],
    RAIL: PALETTE["steel_light"],
    BEACON: PALETTE["sodium_warm"],
    EXPLOSIVE: PALETTE["hazard_red"],
    BUCKET_STATION: PALETTE["copper_bright"],
    BUCKET_POST: PALETTE["timber_mid"],
}

# Variantes cromáticas por equipo para bloques con metadato en `gmap.team_meta`.
_TEAM_BLOCK_COLORS: dict[tuple[int, str], tuple[int, int, int, int]] = {
    (TEAM_BLOCK, "copper"): PALETTE["copper_mid"],
    (TEAM_BLOCK, "cobalt"): PALETTE["cobalt_mid"],
    (FORCE_FIELD, "copper"): PALETTE["copper_glow"],
    (FORCE_FIELD, "cobalt"): PALETTE["cobalt_glow"],
    (BUCKET_STATION, "copper"): PALETTE["copper_bright"],
    (BUCKET_STATION, "cobalt"): PALETTE["cobalt_bright"],
}

# Leyenda compacta (etiqueta, color) mostrada al pie de cada lámina.
_LEGEND_ITEMS: tuple[tuple[str, tuple[int, int, int, int]], ...] = (
    ("DIRT", PALETTE["ochre_dirt"]),
    ("ROCK", PALETTE["slate_mid"]),
    ("HARD", PALETTE["basalt_deep"]),
    ("ORE", PALETTE["ore_amber"]),
    ("GOLD", PALETTE["gold_gleam"]),
    ("DIAM", PALETTE["diamond_cyan"]),
    ("LAVA", PALETTE["lava_orange"]),
    ("COPR", PALETTE["copper_mid"]),
    ("COBL", PALETTE["cobalt_mid"]),
    ("LADR", PALETTE["timber_light"]),
)


def _fill_rect(
    canvas: bytearray, cw: int, ch: int, x: int, y: int, w: int, h: int, rgba: tuple[int, int, int, int]
) -> None:
    """Rellena un rectángulo sólido en el lienzo RGBA8 con recorte seguro."""
    x0, y0, x1, y1 = max(0, x), max(0, y), min(cw, x + w), min(ch, y + h)
    if x0 >= x1 or y0 >= y1:
        return
    row_bytes = bytes(rgba) * (x1 - x0)
    span = (x1 - x0) * 4
    for py in range(y0, y1):
        start = (py * cw + x0) * 4
        canvas[start : start + span] = row_bytes


def _voxel_color(gmap: GeneratedMap, x: int, y: int, z: int) -> tuple[int, int, int, int]:
    """Obtiene el color de paleta de `(x, y, z)` respetando `gmap.team_meta`."""
    block_id = gmap.get_block(x, y, z)
    team = gmap.team_meta.get((x, y, z))
    if team is not None and (block_id, team) in _TEAM_BLOCK_COLORS:
        return _TEAM_BLOCK_COLORS[(block_id, team)]
    return BLOCK_COLOR_MAP.get(block_id, PALETTE["void_black"])


def _compose_sheet(
    grid_w: int,
    grid_h: int,
    scale: int,
    title: str,
    subtitle: str,
    grid_colors: list[tuple[int, int, int, int]],
) -> tuple[int, int, bytes]:
    """Maqueta una cuadrícula de colores escalada con cabecera técnica y leyenda."""
    if scale <= 0:
        raise ValueError(f"El factor de escala debe ser positivo: {scale}")

    map_px_w, map_px_h = grid_w * scale, grid_h * scale
    pad, header_h, footer_h = 12, 32, 32
    content_w = max(map_px_w, 256)
    canvas_w = content_w + 2 * pad
    canvas_h = pad + header_h + map_px_h + footer_h + pad

    canvas = bytearray(bytes(PALETTE["soot_dark"]) * (canvas_w * canvas_h))
    map_x0 = (canvas_w - map_px_w) // 2
    map_y0 = pad + header_h

    draw_text(canvas, canvas_w, canvas_h, pad, pad, title, PALETTE["sodium_warm"], 1)
    draw_text(canvas, canvas_w, canvas_h, pad, pad + 12, subtitle, PALETTE["diamond_white"], 1)
    _fill_rect(
        canvas, canvas_w, canvas_h, map_x0 - 1, map_y0 - 1, map_px_w + 2, map_px_h + 2, PALETTE["slate_dark"]
    )

    for gy in range(grid_h):
        py = map_y0 + gy * scale
        row_offset = gy * grid_w
        for gx in range(grid_w):
            px = map_x0 + gx * scale
            _fill_rect(canvas, canvas_w, canvas_h, px, py, scale, scale, grid_colors[row_offset + gx])

    legend_y0 = map_y0 + map_px_h + 8
    col_step = content_w // 5
    for idx, (label, swatch_col) in enumerate(_LEGEND_ITEMS):
        row, col = idx // 5, idx % 5
        lx, ly = pad + col * col_step, legend_y0 + row * 12
        _fill_rect(canvas, canvas_w, canvas_h, lx, ly, 7, 7, PALETTE["slate_light"])
        _fill_rect(canvas, canvas_w, canvas_h, lx + 1, ly + 1, 5, 5, swatch_col)
        draw_text(canvas, canvas_w, canvas_h, lx + 10, ly, label, PALETTE["diamond_white"], 1)

    return (canvas_w, canvas_h, bytes(canvas))


def render_horizontal_slice(
    gmap: GeneratedMap, y: int, scale: int = 4
) -> tuple[int, int, bytes]:
    """Renderiza un corte horizontal `X × Z` a la cota `y` indicada."""
    if not (0 <= y < gmap.height):
        raise ValueError(f"Cota y fuera de rango (0..{gmap.height - 1}): {y}")

    w, d, h = gmap.width, gmap.depth, gmap.height
    hy = gmap.heart_center[1]
    grid_colors: list[tuple[int, int, int, int]] = []

    for z in range(d):
        for x in range(w):
            block_id = gmap.get_block(x, y, z)
            if block_id == AIR and y > 0 and gmap.get_block(x, y - 1, z) == TEAM_BLOCK:
                if y + 3 < h and gmap.get_block(x, y + 3, z) == BEACON:
                    color = _voxel_color(gmap, x, y + 3, z)
                else:
                    color = _voxel_color(gmap, x, y - 1, z)
            elif block_id == ROCK and y == hy and y >= 2:
                if gmap.get_block(x, y - 1, z) == HARD_ROCK:
                    color = BLOCK_COLOR_MAP[HARD_ROCK]
                elif any(gmap.get_block(x, py, z) == LAVA for py in range(4, min(h, 17))):
                    color = BLOCK_COLOR_MAP[LAVA]
                else:
                    color = _voxel_color(gmap, x, y, z)
            else:
                color = _voxel_color(gmap, x, y, z)
            grid_colors.append(color)

    if y == gmap.copper_base[1]:
        tag = "BASES"
    elif y == gmap.mother_lode_center[1]:
        tag = "MOTHER LODE"
    elif y == hy:
        tag = "HEART LAVA"
    else:
        tag = "STRATA"
    title = f"HUSHMINE SLICE Y:{y:02d} {tag}"
    subtitle = f"SEED:{gmap.seed} XZ:{w}X{d} W:COPPER E:COBALT"
    return _compose_sheet(w, d, scale, title, subtitle, grid_colors)


def render_vertical_slice_z(
    gmap: GeneratedMap, z: int, scale: int = 4
) -> tuple[int, int, bytes]:
    """Renderiza un corte vertical Este-Oeste `X × Y` en `z` fijo (`y=H-1` arriba)."""
    if not (0 <= z < gmap.depth):
        raise ValueError(f"Coordenada z fuera de rango (0..{gmap.depth - 1}): {z}")

    w, h, d = gmap.width, gmap.height, gmap.depth
    grid_colors: list[tuple[int, int, int, int]] = []

    for row in range(h):
        y = h - 1 - row
        for x in range(w):
            block_id = gmap.get_block(x, y, z)
            if block_id in (DIRT, ROCK, AIR) and z + 2 < d and gmap.get_block(x, y, z + 2) == LADDER:
                color = _voxel_color(gmap, x, y, z + 2)
            elif block_id == AIR and z >= 2 and gmap.get_block(x, y, z - 2) in (BUCKET_STATION, BUCKET_POST):
                color = _voxel_color(gmap, x, y, z - 2)
            else:
                color = _voxel_color(gmap, x, y, z)
            grid_colors.append(color)

    title = f"HUSHMINE SLICE Z:{z:02d} EAST-WEST"
    subtitle = f"SEED:{gmap.seed} XY:{w}X{h} TOP:Y{h - 1} BOT:Y00"
    return _compose_sheet(w, h, scale, title, subtitle, grid_colors)


def render_vertical_slice_x(
    gmap: GeneratedMap, x: int, scale: int = 4
) -> tuple[int, int, bytes]:
    """Renderiza un corte vertical Norte-Sur `Z × Y` en `x` fijo (`y=H-1` arriba)."""
    if not (0 <= x < gmap.width):
        raise ValueError(f"Coordenada x fuera de rango (0..{gmap.width - 1}): {x}")

    d, h = gmap.depth, gmap.height
    grid_colors = [
        _voxel_color(gmap, x, h - 1 - row, z) for row in range(h) for z in range(d)
    ]
    title = f"HUSHMINE SLICE X:{x:02d} NORTH-SOUTH"
    subtitle = f"SEED:{gmap.seed} ZY:{d}X{h} TOP:Y{h - 1} BOT:Y00"
    return _compose_sheet(d, h, scale, title, subtitle, grid_colors)


def _column_heightmap_color(
    gmap: GeneratedMap, x: int, z: int
) -> tuple[int, int, int, int]:
    """Codifica la columna `(x, z)` según estructuras, riqueza mineral y cota."""
    w, h, d = gmap.width, gmap.height, gmap.depth
    if x == 0 or x == w - 1 or z == 0 or z == d - 1:
        return PALETTE["basalt_deep"]

    mx, _, mz = gmap.mother_lode_center
    if mx - 3 <= x <= mx + 2 and abs(z - mz) <= 2:
        if x in (mx - 1, mx) and z == mz:
            return PALETTE["diamond_white"]
        if mx - 2 <= x <= mx + 1 and abs(z - mz) <= 1:
            return PALETTE["diamond_cyan"]
        return PALETTE["gold_gleam"]

    (cbx0, _, cbz0), (cbx1, _, cbz1) = gmap.copper_base_bounds
    if cbx0 <= x <= cbx1 and cbz0 <= z <= cbz1:
        if (x, z) == (gmap.copper_base[0], gmap.copper_base[2]):
            return PALETTE["sodium_warm"]
        if x in (cbx0, cbx1) or z in (cbz0, cbz1):
            return PALETTE["copper_glow"]
        return PALETTE["copper_mid"]

    (bbx0, _, bbz0), (bbx1, _, bbz1) = gmap.cobalt_base_bounds
    if bbx0 <= x <= bbx1 and bbz0 <= z <= bbz1:
        if (x, z) == (gmap.cobalt_base[0], gmap.cobalt_base[2]):
            return PALETTE["sodium_warm"]
        if x in (bbx0, bbx1) or z in (bbz0, bbz1):
            return PALETTE["cobalt_glow"]
        return PALETTE["cobalt_mid"]

    top_diamond_y = top_gold_y = top_ore_y = top_lava_y = top_cave_y = -1
    dirt_floor_y = 46
    for y in range(h - 2, 0, -1):
        b = gmap.get_block(x, y, z)
        if b == DIRT:
            dirt_floor_y = y
        elif b == DIAMOND and top_diamond_y < 0:
            top_diamond_y = y
        elif b == GOLD and top_gold_y < 0:
            top_gold_y = y
        elif b == ORE and top_ore_y < 0:
            top_ore_y = y
        elif b == LAVA and top_lava_y < 0:
            top_lava_y = y
        elif b == AIR and top_cave_y < 0:
            top_cave_y = y

    if top_diamond_y >= 0:
        return PALETTE["diamond_ice"] if top_diamond_y >= 22 else PALETTE["diamond_cyan"]
    if top_gold_y >= 0:
        return PALETTE["gold_gleam"] if top_gold_y >= 42 else (PALETTE["gold_bright"] if top_gold_y >= 26 else PALETTE["gold_mid"])
    if top_ore_y >= 0:
        return PALETTE["ore_highlight"] if top_ore_y >= 38 else PALETTE["ore_amber"]
    if top_lava_y >= 0:
        return PALETTE["lava_orange"] if top_lava_y >= 12 else PALETTE["lava_red"]
    if top_cave_y >= 0:
        return PALETTE["ochre_dirt"] if top_cave_y >= 32 else PALETTE["loam_brown"]

    strata_ramp = (PALETTE["slate_dark"], PALETTE["slate_mid"], PALETTE["slate_light"])
    return strata_ramp[(dirt_floor_y - 46) % len(strata_ramp)]


def render_heightmap(gmap: GeneratedMap, scale: int = 4) -> tuple[int, int, bytes]:
    """Renderiza el mapa de alturas y estructuras visto desde arriba (`X × Z`)."""
    w, d = gmap.width, gmap.depth
    grid_colors = [_column_heightmap_color(gmap, x, z) for z in range(d) for x in range(w)]
    title = "HUSHMINE TOP HEIGHTMAP AND LODE"
    subtitle = f"SEED:{gmap.seed} XZ:{w}X{d} BASES AND MOTHER LODE"
    return _compose_sheet(w, d, scale, title, subtitle, grid_colors)


def render_all_slices(
    gmap: GeneratedMap, out_dir: str | Path = "out/mapgen", scale: int = 4
) -> list[Path]:
    """Genera y escribe los 6 archivos PNG de inspección del mapa en `out_dir`."""
    target_dir = Path(out_dir)
    target_dir.mkdir(parents=True, exist_ok=True)

    jobs: list[tuple[str, tuple[int, int, bytes]]] = [
        ("slice_y52_bases.png", render_horizontal_slice(gmap, 52, scale=scale)),
        ("slice_y30_mother_lode.png", render_horizontal_slice(gmap, 30, scale=scale)),
        ("slice_y06_heart_lava.png", render_horizontal_slice(gmap, 6, scale=scale)),
        ("slice_z32_east_west.png", render_vertical_slice_z(gmap, gmap.depth // 2, scale=scale)),
        ("slice_x32_north_south.png", render_vertical_slice_x(gmap, gmap.width // 2, scale=scale)),
        ("heightmap_top.png", render_heightmap(gmap, scale=scale)),
    ]

    written_paths: list[Path] = []
    for filename, (img_w, img_h, rgba_bytes) in jobs:
        out_path = target_dir / filename
        write_png(out_path, img_w, img_h, rgba_bytes)
        written_paths.append(out_path)

    return written_paths


def main() -> int:
    """Punto de entrada CLI para renderizar cortes de un mapa HUSHMINE."""
    parser = argparse.ArgumentParser(description="Renderizador de cortes 2D y mapa de alturas para HUSHMINE.")
    parser.add_argument("--seed", type=int, default=42, help="Semilla entera de 64 bits.")
    parser.add_argument("--team-size", type=int, default=4, help="Tamaño de equipo (1..8).")
    parser.add_argument("--out-dir", type=Path, default=Path("out/mapgen"), help="Directorio de salida PNG.")
    parser.add_argument("--scale", type=int, default=4, help="Factor de escala por celda en píxeles.")
    args = parser.parse_args()

    gmap = generate_map(seed=args.seed, team_size=args.team_size)
    paths = render_all_slices(gmap, out_dir=args.out_dir, scale=args.scale)
    for path in paths:
        w, h, rgba = read_png(path)
        assert encode_png(w, h, rgba) == path.read_bytes()
        print(f"wrote: {path.as_posix()} ({w}x{h}) blocks={len(BLOCK_NAMES)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
