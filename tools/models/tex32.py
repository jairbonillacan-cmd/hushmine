"""Generador procedural determinista de texturas 32×32 y mip 16×16 para HUSHMINE.

Genera los 7 bloques naturales (`dirt`, `rock`, `hard_rock`, `ore`, `gold`,
`diamond` y `lava`, fotograma único estilo lava_0) con la paleta cerrada de
`tools/textures/palette.py`, aritmética exclusivamente entera y continuidad
toroidal estricta: todo patrón es función de cantidades módulo 32, así los
bordes empalman sin costura. Expone `generate_textures()` y `downsample_2x2()`
para importación externa (`from tools.models import tex32`).

**Regla del mip de 16×16** (docs/graficos/03-texturas-32.md, §4): cada textura
32×32 se reduce a 16×16 promediando cada bloque 2×2 en espacio lineal (tablas
enteras sRGB↔lineal precomputadas al cargar, división entera con redondeo) y
ajustando el resultado al color de paleta más cercano (distancia euclídea RGB²;
desempate: primer color insertado en PALETTE). El mip debe superar las mismas
pruebas que la textura grande: paleta, banda de luminancia, distinción contra
`rock` (visión normal, protanopía y deuteranopía) y mosaico sin costuras.

**Métrica de mosaico** (verificación e): media de |Δ luminancia BT.709| entre la
última y la primera columna (fila) —costura, N pares— frente a la media entre
columnas (filas) interiores adyacentes, N·(N-1) pares; empalma si la costura no
supera 1.5 veces la interior, en ambos ejes, en 32×32 y en mip.
"""

from __future__ import annotations

import sys
from collections.abc import Callable
from pathlib import Path

REPO_ROOT: Path = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.textures import palette, png  # noqa: E402

TEX: int = 32
MIP: int = 16
BLOCKS: tuple[str, ...] = ("dirt", "rock", "hard_rock", "ore", "gold", "diamond", "lava")
PAIRS_VS_ROCK: tuple[str, ...] = ("gold", "ore", "diamond", "lava")
OUT_DIR: Path = REPO_ROOT / "out"

# Tablas enteras de 256 entradas sRGB↔lineal (se construyen una vez, al cargar).
LINEAR_TAB: tuple[int, ...] = tuple(
    int(255.0 * (c / 255.0 / 12.92 if c <= 10 else ((c / 255.0 + 0.055) / 1.055) ** 2.4) + 0.5)
    for c in range(256)
)
SRGB_TAB: tuple[int, ...] = tuple(
    min(255, max(0, int(255.0 * (1.055 * (l / 255.0) ** (1.0 / 2.4) - 0.055) + 0.5)))
    for l in range(256)
)

def _smooth_noise(x: int, y: int, seed: int) -> int:
    """Ruido entero suavizado 3×3 sobre el toro 32×32 (rango aproximado 0..255)."""
    weights = ((1, 2, 1), (2, 4, 2), (1, 2, 1))
    acc = 0
    for dy in (-1, 0, 1):
        ny = (y + dy) % TEX
        for dx in (-1, 0, 1):
            acc += (palette.hash2d((x + dx) % TEX, ny, seed) & 0xFF) * weights[dy + 1][dx + 1]
    return acc >> 4

def _torus_delta(a: int, b: int) -> int:
    """Desplazamiento con signo más corto (a - b) en el toro de lado 32."""
    d = (a - b) % TEX
    return d - TEX if d > TEX // 2 else d

def _min_dist_sq(x: int, y: int, points: tuple[tuple[int, int], ...]) -> int:
    """Distancia euclídea al cuadrado mínima desde (x, y) sobre el toro 32×32."""
    best = 1 << 30
    for cx, cy in points:
        dx, dy = _torus_delta(x, cx), _torus_delta(y, cy)
        best = min(best, dx * dx + dy * dy)
    return best

def _render(color_at: Callable[[int, int], str]) -> bytes:
    """Recorre el toro 32×32 por filas y empaqueta los nombres que devuelve `color_at`."""
    names = [color_at(x, y) for y in range(TEX) for x in range(TEX)]
    return b"".join(bytes(palette.color(n)) for n in names)

def _nearest_palette(r: int, g: int, b: int) -> tuple[int, int, int, int]:
    """Color de paleta más cercano en RGB²; `min` conserva el primero en empates."""
    return min(palette.PALETTE.values(),
               key=lambda p: (r - p[0]) ** 2 + (g - p[1]) ** 2 + (b - p[2]) ** 2)

def downsample_2x2(img32: bytes) -> bytes:
    """Reduce una textura 32×32 a su mip 16×16 (media 2×2 lineal + ajuste a paleta)."""
    out = bytearray(MIP * MIP * 4)
    row = TEX * 4
    for my in range(MIP):
        for mx in range(MIP):
            o = (my * MIP + mx) * 4
            for ch in range(3):
                s = sum(LINEAR_TAB[img32[(2 * my + ky) * row + 2 * mx * 4 + 4 * kx + ch]]
                        for ky in range(2) for kx in range(2))
                out[o + ch] = SRGB_TAB[(s + 2) >> 2]
            out[o:o + 4] = bytes(_nearest_palette(out[o], out[o + 1], out[o + 2]))
    return bytes(out)

# ----------------------------------------------------------- patrones por bloque

_DARK_CLUMPS = ((5, 6), (15, 3), (26, 9), (8, 20), (21, 25), (27, 28), (16, 14), (10, 27))
_WARM_CLUMPS = ((10, 4), (23, 7), (3, 16), (18, 12), (28, 17), (12, 22), (25, 13), (6, 10))

def _dirt_at(x: int, y: int) -> str:
    """Gránulos oscuros (húmus) y cálidos (arcilla) agrupados sobre tierra media."""
    n1 = _smooth_noise(x, y, 101)
    n2 = _smooth_noise((x + 11) % TEX, (y + 7) % TEX, 102)
    dd, dw = _min_dist_sq(x, y, _DARK_CLUMPS), _min_dist_sq(x, y, _WARM_CLUMPS)
    if dd <= 8 or (dd <= 20 and n1 < 115):
        return "peat_dark" if n2 < 140 else "timber_dark"
    if dw <= 8 or (dw <= 20 and n1 > 130):
        return "ochre_dirt" if n2 < 135 else "timber_mid"
    if n1 > 168:
        return "ochre_dirt"
    if n1 < 88:
        return "timber_dark"
    return "loam_brown" if (palette.hash2d(x, y, 103) & 0xFF) < 186 else "timber_dark"

_WAVE = (0, 0, 1, 1, 2, 2, 2, 1, 1, 0, 0, -1, -1, -2, -2, -2,
         -1, -1, 0, 0, 1, 1, 2, 2, 2, 2, 1, 1, 0, 0, -1, -1)

def _rock_at(x: int, y: int) -> str:
    """Lajas diagonales (fase x + 2·y ondulada) con juntas oscuras y grano entero."""
    raw = palette.hash2d(x, y, 202) & 0xFF
    phase = (x + 2 * y + _WAVE[(x - y) % TEX] + 128) % 8
    if phase in (0, 4):
        return "shadow_slate" if raw < 12 else "slate_dark"
    if phase in (1, 5) and raw > 200:
        return "slate_light"
    if phase in (2, 7) and raw > 235:
        return "slate_light"
    if phase in (3, 6) and raw < 45:
        return "slate_dark"
    return "slate_mid"

_COLUMN_CENTERS = (
    (3, 2), (11, 1), (20, 3), (28, 2), (7, 8), (16, 7), (25, 9), (31, 8), (2, 14), (11, 15),
    (20, 14), (28, 16), (6, 22), (15, 21), (24, 23), (31, 22), (3, 29), (12, 29), (21, 30), (29, 28))

def _hard_rock_at(x: int, y: int) -> str:
    """Celdas Voronoi estiradas en vertical (columnas de basalto) y fractura al borde."""
    n, raw = _smooth_noise(x, y, 301), palette.hash2d(x, y, 302) & 0xFF
    d1 = d2 = 1 << 30
    bdx = bdy = 0
    for cx, cy in _COLUMN_CENTERS:
        dx, dy = _torus_delta(x, cx), _torus_delta(y, cy)
        dist = abs(dx) * 5 + abs(dy) * 2
        if dist < d1:
            d2, d1, bdx, bdy = d1, dist, dx, dy
        elif dist < d2:
            d2 = dist
    if d2 - d1 <= 4:
        return "void_black" if raw < 100 else "soot_dark"
    if bdx <= 0 and bdy <= 0:
        col = "basalt_mid" if n >= 140 else "basalt_dark"
    elif bdx >= 1 and bdy >= 1:
        col = "basalt_deep" if n < 140 else "basalt_dark"
    else:
        col = "basalt_dark" if n < 140 else "basalt_mid"
    if raw > 222 and col != "basalt_mid":
        return "basalt_mid"
    return "basalt_deep" if raw < 18 and col != "basalt_deep" else col

_JAG = (0, 1, 1, 0, -1, 0, 1, 0, 0, -1, -1, 0, 1, 0, 0, -1,
        0, 1, 1, 0, -1, 0, 1, 0, 0, -1, 1, 0, 0, 1, -1, 0)

def _ore_at(x: int, y: int) -> str:
    """Vetas ferrosas a 45° (familias en d = 0 y d = 16) sobre matriz pizarrosa."""
    n, raw = _smooth_noise(x, y, 401), palette.hash2d(x, y, 402) & 0xFF
    d = (x + y + _JAG[(x - y) % TEX] + 64) % TEX
    v1 = min(d, TEX - d)
    v2 = abs(((d + 16) % TEX) - 16)
    if v1 == 0 or v2 == 0:
        return "ore_highlight" if n > 160 else ("sodium_warm" if n > 80 else "ore_amber")
    if v1 == 1 or v2 == 1:
        return "ore_highlight" if n > 30 else "ore_amber"
    if v1 == 2 or v2 == 2:
        return "ore_amber" if raw > 70 else "ore_rust_mid"
    if v1 in (3, 4) or v2 in (3, 4):
        return "ore_rust_mid" if raw > 90 else "ore_rust_dark"
    if n > 120 or raw > 210:
        return "shale_highlight"
    return "slate_dark" if raw < 15 else "slate_light"

_NUGGETS = ((4, 5, 7, 13), (16, 3, 7, 13), (27, 6, 6, 12), (8, 13, 7, 13), (21, 14, 8, 14),
            (3, 21, 7, 13), (14, 24, 7, 13), (26, 27, 7, 13), (10, 30, 6, 12))

def _gold_at(x: int, y: int) -> str:
    """Pepitas circulares solapadas: núcleo brillante, cuerpo oro medio, borde en sombra."""
    n, raw = _smooth_noise(x, y, 501), palette.hash2d(x, y, 502) & 0xFF
    best_q, bd2, bdx, bdy, bro2 = 1 << 30, 1 << 30, 0, 0, 1
    for cx, cy, _rc, ro in _NUGGETS:
        dx, dy = _torus_delta(x, cx), _torus_delta(y, cy)
        d2 = dx * dx + dy * dy
        q = d2 * 256 // (ro * ro)
        if q < best_q:
            best_q, bd2, bdx, bdy, bro2 = q, d2, dx, dy, ro * ro
    if best_q > 235:
        return "slate_light" if n > 110 else "slate_mid"
    rel = bd2 * 256
    if rel <= bro2 * 30:
        if bdx <= 0 and bdy <= 0:
            return "gold_gleam" if rel <= bro2 * 3 else "gold_bright"
        return "gold_bright" if n > 175 else "gold_mid"
    if rel <= bro2 * 140:
        if bdx >= 1 and bdy >= 1:
            return "gold_shadow" if n > 75 else "gold_mid"
        return "gold_shadow" if raw < 45 else "gold_mid"
    return "gold_shadow" if n > 55 else "gold_mid"

_CRYSTALS = ((3, 4, 6), (12, 2, 6), (22, 5, 6), (29, 13, 6), (7, 12, 7), (17, 15, 6),
             (26, 24, 6), (5, 23, 6), (14, 27, 6), (23, 30, 6))

def _diamond_at(x: int, y: int) -> str:
    """Cristales rómbicos (bola de Manhattan) con destello axial y faceta por cuadrante."""
    n = _smooth_noise(x, y, 601)
    best_margin, bdx, bdy, bmd, br = -(1 << 30), 0, 0, 0, 1
    for cx, cy, rad in _CRYSTALS:
        dx, dy = _torus_delta(x, cx), _torus_delta(y, cy)
        md = abs(dx) + abs(dy)
        if rad - md > best_margin:
            best_margin, bdx, bdy, bmd, br = rad - md, dx, dy, md, rad
    if bmd > br:
        if bmd == br + 1:
            return "diamond_teal" if n > 100 else "diamond_deep"
        return "diamond_deep" if n > 100 else "shale_highlight"
    if bmd == 0 or (bmd == 1 and (bdx == 0 or bdy == 0)):
        return "diamond_white"
    if bdx == 0 or bdy == 0:
        if bmd > br // 2:
            return "diamond_cyan"
        return "diamond_ice" if (bdx + bdy) > 0 else "diamond_white"
    if bdx <= 0 and bdy <= 0:
        return "diamond_ice" if n > 130 else "diamond_cyan"
    if bmd == br:
        return "diamond_teal"
    return "diamond_cyan" if n > 130 else "diamond_teal"

_LAVA_WAVE = (0, 1, 1, 2, 2, 1, 0, -1, -1, -2, -2, -1, 0, 1, 1, 2,
              2, 1, 0, -1, -1, -2, -2, -1, 0, 1, 1, 2, 2, 1, 0, 0)
_CRUST_PLATES = ((4, 4, 6), (15, 3, 8), (25, 7, 6), (8, 14, 8), (20, 16, 6))

def _lava_at(x: int, y: int) -> str:
    """Canales incandescentes diagonales (índice x + y ondulado) con placas de costra."""
    n, np_ = _smooth_noise(x, y, 701), _smooth_noise((x + 9) % TEX, (y + 17) % TEX, 702)
    band = ((x + y + _LAVA_WAVE[(x - y) % TEX] + 64) % TEX) * 10 // TEX
    best = 1 << 30
    for cx, cy, _r2 in _CRUST_PLATES:
        dx, dy = _torus_delta(x, cx), _torus_delta(y, cy)
        best = min(best, dx * dx + dy * dy)
    if best <= 6:
        return "magma_crust" if np_ < 140 else "lava_red"
    if best <= 12:
        return "lava_red" if np_ < 130 else "lava_orange"
    if band <= 4:
        return "lava_core" if n > 90 else "lava_yellow"
    if band <= 7:
        return "lava_core" if np_ < 55 else "lava_yellow"
    return "lava_yellow" if np_ > 140 else "lava_orange"

def generate_textures() -> dict[str, bytes]:
    """Devuelve las 7 texturas naturales 32×32 como buffers RGBA8 deterministas."""
    makers = (("dirt", _dirt_at), ("rock", _rock_at), ("hard_rock", _hard_rock_at),
              ("ore", _ore_at), ("gold", _gold_at), ("diamond", _diamond_at), ("lava", _lava_at))
    return {name: _render(at) for name, at in makers}

# ----------------------------------------------------------- verificación

def _apply_sim(buf: bytes, sim_fn: Callable[[tuple], tuple]) -> bytes:
    """Aplica una transformación de daltonismo píxel a píxel sobre un buffer RGBA8."""
    return b"".join(bytes(sim_fn(tuple(buf[i:i + 4]))) for i in range(0, len(buf), 4))

def _tiling_stats(buf: bytes, size: int) -> tuple[float, float, float, float]:
    """Medias de |Δ luminancia| BT.709: (columnas interior/costura, filas interior/costura)."""
    lums = [0.2126 * buf[i] + 0.7152 * buf[i + 1] + 0.0722 * buf[i + 2]
            for i in range(0, len(buf), 4)]
    col_int = sum(abs(lums[y * size + x] - lums[y * size + x + 1])
                  for y in range(size) for x in range(size - 1))
    col_seam = sum(abs(lums[y * size + size - 1] - lums[y * size]) for y in range(size))
    row_int = sum(abs(lums[y * size + x] - lums[(y + 1) * size + x])
                  for y in range(size - 1) for x in range(size))
    row_seam = sum(abs(lums[(size - 1) * size + x] - lums[x]) for x in range(size))
    n = size * (size - 1)
    return (col_int / n, col_seam / size, row_int / n, row_seam / size)

def verify_mip_rule(textures: dict[str, bytes] | None = None,
                    mips: dict[str, bytes] | None = None) -> tuple[bool, list[str]]:
    """Comprueba paleta, bandas de luminancia, distinción (con daltonismo) y mosaico."""
    textures = textures if textures is not None else generate_textures()
    mips = mips if mips is not None else {n: downsample_2x2(b) for n, b in textures.items()}
    samples = (("32×32", textures), ("mip", mips))
    errors: list[str] = []
    for name, (lo, hi) in palette.LUMINANCE_BANDS.items():
        for tag, bufs in samples:
            buf = bufs[name]
            bad = [i // 4 for i in range(0, len(buf), 4)
                   if tuple(buf[i:i + 4]) not in palette.PALETTE_SET]
            if bad:
                errors.append(f"{name}: {len(bad)} píxeles fuera de paleta en {tag} (ej. {bad[:3]})")
            mean_y = palette.mean_luminance(buf)
            if not lo <= mean_y <= hi:
                errors.append(f"{name}: luminancia media {tag} {mean_y:.1f} fuera de banda [{lo}, {hi}]")
    sims = (("visión normal", None), ("protanopía", palette.simulate_protanopia),
            ("deuteranopía", palette.simulate_deuteranopia))
    for name in PAIRS_VS_ROCK:
        for label, sim in sims:
            for tag, bufs in samples:
                pair = (bufs[k] if sim is None else _apply_sim(bufs[k], sim) for k in (name, "rock"))
                dist = palette.luminance_distance(*pair)
                if dist < palette.MIN_LUMINANCE_DISTANCE:
                    errors.append(f"{name} vs rock ({tag}, {label}): {dist:.1f} < "
                                  f"{palette.MIN_LUMINANCE_DISTANCE}")
    for name in BLOCKS:
        for tag, buf, size in (("32×32", textures[name], TEX), ("mip", mips[name], MIP)):
            col_i, col_s, row_i, row_s = _tiling_stats(buf, size)
            if col_s > 1.5 * col_i:
                errors.append(f"{name} ({tag}): costura vertical {col_s:.2f} > 1.5 × interior {col_i:.2f}")
            if row_s > 1.5 * row_i:
                errors.append(f"{name} ({tag}): costura horizontal {row_s:.2f} > 1.5 × interior {row_i:.2f}")
    return (not errors, errors)

# ----------------------------------------------------------- hoja de contacto

def _blit_scaled(canvas: bytearray, cw: int, ch: int, dx: int, dy: int,
                 tex: bytes, size: int, scale: int) -> None:
    """Copia una textura ampliada (vecino más cercano) sobre el lienzo."""
    for i in range(0, size * size * 4, 4):
        px = (i // 4) % size, (i // 4) // size
        png._fill_rect(canvas, cw, ch, dx + px[0] * scale, dy + px[1] * scale, scale, scale,
                       tuple(tex[i:i + 4]))

def _build_contact_sheet(textures: dict[str, bytes],
                         mips: dict[str, bytes]) -> tuple[int, int, bytes]:
    """Hoja comparativa: 32×32 ×8 junto a su mip 16×16 ×8, con rótulos de la fuente 5×7."""
    scale, pad, head, gap, label_h = 8, 14, 12, 16, 20
    tile_w, mip_w = TEX * scale, MIP * scale
    pair_w = tile_w + gap + mip_w
    cell_w, cell_h = pair_w + 2 * pad, pad + head + tile_w + 8 + label_h + pad
    cols, rows, margin, grid = 2, 4, 16, 12
    sheet_w = margin * 2 + cols * cell_w + (cols - 1) * grid
    sheet_h = margin * 2 + rows * cell_h + (rows - 1) * grid
    canvas = bytearray(bytes(palette.color("void_black")) * (sheet_w * sheet_h))
    card, frame = palette.color("soot_dark"), palette.color("slate_dark")
    text, accent = palette.color("diamond_white"), palette.color("sodium_warm")
    for idx, name in enumerate(BLOCKS):
        cx = margin + (idx % cols) * (cell_w + grid)
        cy = margin + (idx // cols) * (cell_h + grid)
        png._fill_rect(canvas, sheet_w, sheet_h, cx, cy, cell_w, cell_h, frame)
        png._fill_rect(canvas, sheet_w, sheet_h, cx + 1, cy + 1, cell_w - 2, cell_h - 2, card)
        lx, ly, rx = cx + pad, cy + pad + head, cx + pad + tile_w + gap
        png.draw_text(canvas, sheet_w, sheet_h, lx, cy + pad, "32X32", accent, 1)
        png.draw_text(canvas, sheet_w, sheet_h, rx, cy + pad, "MIP 16X16", accent, 1)
        png._fill_rect(canvas, sheet_w, sheet_h, lx - 1, ly - 1, tile_w + 2, tile_w + 2, frame)
        png._fill_rect(canvas, sheet_w, sheet_h, rx - 1, ly - 1, mip_w + 2, mip_w + 2, frame)
        _blit_scaled(canvas, sheet_w, sheet_h, lx, ly, textures[name], TEX, scale)
        _blit_scaled(canvas, sheet_w, sheet_h, rx, ly, mips[name], MIP, scale)
        png.draw_text(canvas, sheet_w, sheet_h, cx + pad + (pair_w - len(name) * 12 + 2) // 2,
                      ly + tile_w + 8, name, text, 2)
    return (sheet_w, sheet_h, bytes(canvas))

def main() -> None:
    """Genera, escribe los 8 PNG, verifica la regla del mip y termina con 0 o 1."""
    textures = generate_textures()
    mips = {name: downsample_2x2(buf) for name, buf in textures.items()}
    for name, buf in textures.items():
        png.write_png(OUT_DIR / "tex32" / f"{name}.png", TEX, TEX, buf)
    ok, errors = verify_mip_rule(textures, mips)
    w, h, sheet = _build_contact_sheet(textures, mips)
    png.write_png(OUT_DIR / "tex32_contact.png", w, h, sheet)
    print(f"TEX32: 7 texturas 32×32 y mips 16×16 escritos en {OUT_DIR / 'tex32'}")
    for name in BLOCKS:
        print(f"  {name:<10} Y32={palette.mean_luminance(textures[name]):6.2f}"
              f"  Ymip={palette.mean_luminance(mips[name]):6.2f}")
    print(f"  hoja de contacto: out/tex32_contact.png ({w}×{h})")
    if ok:
        print("VERIFICACIÓN OK: paleta, bandas, distinción y mosaico correctos en 32×32 y mip")
        sys.exit(0)
    print("VERIFICACIÓN FALLIDA:")
    for msg in errors:
        print(f"  - {msg}")
    sys.exit(1)

if __name__ == "__main__":
    main()
