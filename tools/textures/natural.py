"""Generador procedural determinista de texturas 16×16 para bloques naturales.

Implementa las 10 capas de terreno natural de HUSHMINE (`dirt`, `rock`,
`hard_rock`, `ore`, `gold`, `diamond` y `lava_0..3`) siguiendo la paleta cerrada
y las bandas de luminancia de `palette.py`, con continuidad toroidal estricta
en bordes opuestos (aritmética módulo 16) y sin marcos perimetrales.
"""

from __future__ import annotations

try:
    from tools.textures import palette
except ImportError:
    import palette  # type: ignore[no-redef]

TEX_SIZE: int = 16


def _torus_delta(a: int, b: int) -> int:
    """Devuelve el desplazamiento con signo más corto (a - b) en un toro de tamaño 16."""
    d = (a - b) % TEX_SIZE
    return d - TEX_SIZE if d > TEX_SIZE // 2 else d


def _smooth_noise_torus(x: int, y: int, seed: int) -> int:
    """Ruido entero suavizado 3×3 sobre el toro 16×16 (rango aproximado 0..255)."""
    weights = ((1, 2, 1), (2, 4, 2), (1, 2, 1))
    acc = 0
    for dy in (-1, 0, 1):
        ny = (y + dy) % TEX_SIZE
        wy = weights[dy + 1]
        for dx in (-1, 0, 1):
            nx = (x + dx) % TEX_SIZE
            acc += (palette.hash2d(nx, ny, seed) & 0xFF) * wy[dx + 1]
    return acc >> 4


def _min_torus_dist_sq(x: int, y: int, points: tuple[tuple[int, int], ...]) -> int:
    """Devuelve la distancia euclídea al cuadrado mínima en el toro 16×16."""
    best = 999
    for cx, cy in points:
        dx = _torus_delta(x, cx)
        dy = _torus_delta(y, cy)
        d2 = dx * dx + dy * dy
        if d2 < best:
            best = d2
    return best


def _pack_pixels(names: list[str]) -> bytes:
    """Empaqueta una lista de 256 nombres de color de la paleta en 1024 bytes RGBA8."""
    buf = bytearray(TEX_SIZE * TEX_SIZE * 4)
    offset = 0
    for name in names:
        r, g, b, a = palette.color(name)
        buf[offset] = r
        buf[offset + 1] = g
        buf[offset + 2] = b
        buf[offset + 3] = a
        offset += 4
    return bytes(buf)


def make_dirt() -> bytes:
    """Genera `dirt` (ID 1): turba y tierra compacta con gránulos orgánicos agrupados.

    Banda de luminancia objetivo: 40.0 <= Y <= 54.0.
    """
    dark_clumps = ((2, 3), (7, 1), (13, 5), (4, 10), (11, 12), (15, 14), (8, 7))
    warm_clumps = ((5, 2), (11, 3), (1, 8), (9, 6), (14, 9), (6, 13), (12, 15), (3, 14))
    pixels: list[str] = []
    for y in range(TEX_SIZE):
        for x in range(TEX_SIZE):
            n_primary = _smooth_noise_torus(x, y, seed=101)
            n_detail = _smooth_noise_torus((x + 5) % TEX_SIZE, (y + 3) % TEX_SIZE, seed=102)
            min_dark_sq = _min_torus_dist_sq(x, y, dark_clumps)
            min_warm_sq = _min_torus_dist_sq(x, y, warm_clumps)

            if min_dark_sq <= 2 or (min_dark_sq <= 4 and n_primary < 115):
                col = "peat_dark" if n_detail < 140 else "timber_dark"
            elif min_warm_sq <= 2 or (min_warm_sq <= 4 and n_primary > 130):
                col = "ochre_dirt" if n_detail < 135 else "timber_mid"
            elif n_primary < 88:
                col = "timber_dark"
            elif n_primary > 168:
                col = "ochre_dirt"
            else:
                col = "loam_brown"
            pixels.append(col)
    return _pack_pixels(pixels)


def make_rock() -> bytes:
    """Genera `rock` (ID 2): pizarra estratificada en lajas horizontales/diagonales suaves.

    Banda de luminancia objetivo: 56.0 <= Y <= 70.0.
    """
    wave_x = (0, 0, 1, 1, 1, 2, 2, 1, 1, 0, 0, -1, -1, -1, 0, 0)
    pixels: list[str] = []
    for y in range(TEX_SIZE):
        for x in range(TEX_SIZE):
            n_smooth = _smooth_noise_torus(x, y, seed=201)
            stratum_y = (y + wave_x[x % TEX_SIZE] + (x // 4)) % TEX_SIZE
            in_band = stratum_y % 4
            seam_phase = (x + (stratum_y // 4) * 5) % TEX_SIZE

            if in_band == 0:
                if seam_phase in (2, 3, 4, 10, 11) and n_smooth < 135:
                    col = "shadow_slate"
                else:
                    col = "slate_dark"
            elif in_band == 1:
                if seam_phase in (5, 6, 7, 13, 14) and n_smooth > 125:
                    col = "shale_highlight"
                elif n_smooth > 95:
                    col = "slate_light"
                else:
                    col = "slate_mid"
            elif in_band == 2:
                if n_smooth > 155:
                    col = "slate_light"
                elif n_smooth < 90:
                    col = "slate_dark"
                else:
                    col = "slate_mid"
            else:
                col = "slate_dark" if n_smooth < 135 else "slate_mid"
            pixels.append(col)
    return _pack_pixels(pixels)


def make_hard_rock() -> bytes:
    """Genera `hard_rock` (ID 3): basalto columnar denso con fracturas poligonales.

    Banda de luminancia objetivo: 22.0 <= Y <= 38.0.
    """
    centers = ((2, 2), (8, 3), (13, 2), (5, 7), (11, 8), (15, 7), (3, 12), (9, 13), (14, 13))
    pixels: list[str] = []
    for y in range(TEX_SIZE):
        for x in range(TEX_SIZE):
            n_smooth = _smooth_noise_torus(x, y, seed=301)
            d1, d2, best_dx, best_dy = 999, 999, 0, 0
            for cx, cy in centers:
                dx = _torus_delta(x, cx)
                dy = _torus_delta(y, cy)
                adx, ady = abs(dx), abs(dy)
                dist = (adx + ady) * 3 + max(adx, ady) * 2
                if dist < d1:
                    d2, d1, best_dx, best_dy = d1, dist, dx, dy
                elif dist < d2:
                    d2 = dist

            edge_gap = d2 - d1
            if edge_gap <= 2:
                col = "void_black" if n_smooth < 110 else "soot_dark"
            elif edge_gap <= 5:
                col = "basalt_deep" if (best_dx + best_dy) >= 0 else "basalt_dark"
            elif best_dx <= 0 and best_dy <= 0:
                col = "basalt_mid" if n_smooth >= 105 else "shadow_slate"
            elif best_dx >= 1 and best_dy >= 1:
                col = "basalt_deep"
            else:
                col = "basalt_dark" if n_smooth < 145 else "basalt_mid"
            pixels.append(col)
    return _pack_pixels(pixels)


def make_ore() -> bytes:
    """Genera `ore` (ID 4): matriz de pizarra con vetas diagonales quebradas a 45°.

    Banda de luminancia objetivo: 88.0 <= Y <= 102.0.
    """
    jag_a = (0, 1, 1, 0, 0, -1, 0, 1, 1, 0, -1, -1, 0, 1, 0, 0)
    jag_b = (0, -1, 0, 1, 1, 0, -1, 0, 0, 1, 1, 0, -1, 0, 1, 0)
    pixels: list[str] = []
    for y in range(TEX_SIZE):
        for x in range(TEX_SIZE):
            n_smooth = _smooth_noise_torus(x, y, seed=401)
            orth = (x - y) % TEX_SIZE
            diag_main = (x + y + jag_a[orth]) % TEX_SIZE
            conj = (x - y + jag_b[(x + y) % TEX_SIZE]) % TEX_SIZE

            dist_v1 = min(diag_main, TEX_SIZE - diag_main)
            dist_v2 = min((diag_main - 8) % TEX_SIZE, (8 - diag_main) % TEX_SIZE)
            in_branch = conj in (4, 11) and (diag_main in (3, 4, 5, 11, 12, 13))

            if dist_v1 == 0 or (dist_v2 == 0 and n_smooth > 115):
                col = "sodium_warm" if n_smooth > 165 else "ore_highlight"
            elif dist_v1 == 1 or dist_v2 == 0 or (in_branch and conj == 4):
                col = "ore_highlight" if n_smooth > 140 else "ore_amber"
            elif dist_v2 == 1 or in_branch or (dist_v1 == 2 and n_smooth > 120):
                col = "ore_amber" if n_smooth > 130 else "ore_rust_mid"
            elif (dist_v1 == 2 or dist_v2 == 2) and n_smooth > 100:
                col = "ore_rust_mid" if n_smooth > 135 else "ore_rust_dark"
            elif n_smooth > 150:
                col = "shale_highlight"
            elif n_smooth > 110:
                col = "slate_light"
            elif n_smooth > 75:
                col = "slate_mid"
            else:
                col = "slate_dark"
            pixels.append(col)
    return _pack_pixels(pixels)


def make_gold() -> bytes:
    """Genera `gold` (ID 5): matriz pétrea con pepitas redondeadas y racimos globulares.

    Banda de luminancia objetivo: 120.0 <= Y <= 136.0.
    """
    nuggets = ((3, 3, 4, 9), (11, 3, 4, 9), (7, 8, 5, 10), (14, 10, 4, 9), (3, 13, 4, 9), (10, 14, 4, 8))
    pixels: list[str] = []
    for y in range(TEX_SIZE):
        for x in range(TEX_SIZE):
            n_smooth = _smooth_noise_torus(x, y, seed=501)
            best_ratio = 999.0
            best_dx, best_dy, best_d2, best_core, best_outer = 0, 0, 999, 1, 1
            for cx, cy, r_core, r_outer in nuggets:
                dx = _torus_delta(x, cx)
                dy = _torus_delta(y, cy)
                d2 = dx * dx + dy * dy
                ratio = d2 / float(r_outer)
                if ratio < best_ratio:
                    best_ratio = ratio
                    best_dx, best_dy, best_d2, best_core, best_outer = dx, dy, d2, r_core, r_outer

            if best_d2 <= best_core:
                if best_dx <= 0 and best_dy <= 0 and best_d2 <= 2:
                    col = "gold_gleam"
                elif best_dx <= 0 or best_dy <= 0:
                    col = "gold_gleam" if n_smooth > 135 else "gold_bright"
                else:
                    col = "gold_bright" if n_smooth > 105 else "gold_mid"
            elif best_d2 <= best_outer:
                if best_dx <= 0 and best_dy <= 0:
                    col = "gold_bright" if n_smooth > 110 else "gold_mid"
                elif best_dx >= 1 and best_dy >= 1:
                    col = "gold_shadow"
                else:
                    col = "gold_mid" if n_smooth > 105 else "gold_shadow"
            elif n_smooth > 150:
                col = "gold_shadow"
            elif n_smooth > 115:
                col = "shale_highlight"
            elif n_smooth > 80:
                col = "slate_light"
            else:
                col = "slate_mid"
            pixels.append(col)
    return _pack_pixels(pixels)


def make_diamond() -> bytes:
    """Genera `diamond` (ID 6): cristales rómbicos/octaédricos facetados con destellos axiales.

    Banda de luminancia objetivo: 154.0 <= Y <= 170.0.
    """
    crystals = ((3, 3, 3), (11, 4, 3), (7, 9, 4), (14, 11, 3), (2, 12, 3), (10, 14, 3))
    pixels: list[str] = []
    for y in range(TEX_SIZE):
        for x in range(TEX_SIZE):
            n_smooth = _smooth_noise_torus(x, y, seed=601)
            best_margin, best_dx, best_dy, best_md, best_r = -999, 0, 0, 999, 1
            for cx, cy, rad in crystals:
                dx = _torus_delta(x, cx)
                dy = _torus_delta(y, cy)
                md = abs(dx) + abs(dy)
                margin = rad - md
                if margin > best_margin:
                    best_margin, best_dx, best_dy, best_md, best_r = margin, dx, dy, md, rad

            if best_md <= best_r:
                on_axis = best_dx == 0 or best_dy == 0
                if best_md == 0 or (best_md == 1 and on_axis):
                    col = "diamond_white"
                elif on_axis and best_md < best_r:
                    col = "diamond_ice" if (best_dx + best_dy) > 0 else "diamond_white"
                elif best_dx <= 0 and best_dy <= 0:
                    col = "diamond_ice"
                elif best_md == best_r:
                    col = "diamond_cyan" if (best_dx + best_dy) <= 0 else "diamond_teal"
                else:
                    col = "diamond_cyan" if n_smooth > 110 else "diamond_teal"
            elif best_md == best_r + 1:
                col = "diamond_teal" if n_smooth > 120 else "diamond_deep"
            elif n_smooth > 125:
                col = "diamond_deep"
            elif n_smooth > 85:
                col = "shale_highlight"
            else:
                col = "slate_light"
            pixels.append(col)
    return _pack_pixels(pixels)


def make_lava(frame: int) -> bytes:
    """Genera un fotograma `lava_0..3` (ID 7): corriente magmática animada en 4 pasos.

    Banda de luminancia objetivo en cada fotograma: 188.0 <= Y <= 215.0.
    """
    if frame not in (0, 1, 2, 3):
        raise ValueError(f"El fotograma de lava debe estar en 0..3, recibido: {frame!r}")

    crust_plates = ((3, 3, 2), (11, 4, 2), (6, 10, 2), (13, 12, 2), (2, 14, 1))
    wave_offset = (0, 1, 1, 0, -1, -1, 0, 1, 1, 0, -1, 0, 1, 1, 0, -1)
    pixels: list[str] = []
    for y in range(TEX_SIZE):
        for x in range(TEX_SIZE):
            sx = (x + frame * 4) % TEX_SIZE
            sy = (y + frame * 2) % TEX_SIZE
            n_flow = _smooth_noise_torus(sx, sy, seed=701)
            n_pulse = _smooth_noise_torus(
                (sx + frame * 3) % TEX_SIZE,
                (sy - frame * 5) % TEX_SIZE,
                seed=710 + frame,
            )

            min_plate_dist = 999
            for px, py, pr in crust_plates:
                ox = (px + ((frame + px) % 2)) % TEX_SIZE
                dx = _torus_delta(sx, ox)
                dy = _torus_delta(sy, py)
                d_sq = dx * dx + dy * dy - pr
                if d_sq < min_plate_dist:
                    min_plate_dist = d_sq

            channel = (sx + sy + wave_offset[sx] + frame) % 8
            if min_plate_dist <= 0:
                col = "magma_crust" if n_pulse < 140 else "lava_red"
            elif min_plate_dist == 1:
                col = "lava_red" if n_pulse < 120 else "lava_orange"
            elif min_plate_dist == 2:
                col = "lava_orange" if n_pulse < 110 else "sodium_warm"
            elif channel in (0, 1, 2) or n_flow > 125:
                col = "lava_core" if n_pulse >= 95 else "lava_yellow"
            elif channel in (3, 4, 5) or n_flow > 90:
                col = "lava_yellow" if n_pulse >= 75 else "sodium_warm"
            else:
                col = "sodium_warm" if n_pulse >= 90 else "lava_orange"
            pixels.append(col)
    return _pack_pixels(pixels)


def generate_natural_textures() -> dict[str, bytes]:
    """Genera y devuelve el diccionario ordenado con las 10 texturas naturales."""
    textures: dict[str, bytes] = {
        "dirt": make_dirt(),
        "rock": make_rock(),
        "hard_rock": make_hard_rock(),
        "ore": make_ore(),
        "gold": make_gold(),
        "diamond": make_diamond(),
    }
    for frame in range(4):
        textures[f"lava_{frame}"] = make_lava(frame)
    return textures
