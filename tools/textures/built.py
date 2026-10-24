"""Generadores procedurales deterministas para los bloques construidos de HUSHMINE.

Produce las 15 texturas de 16×16 píxeles (RGBA8, 1024 bytes por textura) de la
categoría ``built`` (IDs de bloque 8..17), combinando la carpintería minera y
fundición remachada de 1931 con la instrumentación eléctrica y neumática de la
incursión clandestina de 1979. Todas las variantes de equipo aplican doble
codificación cromática y geométrica (Copper: diagonales a 45° y remaches
redondos; Cobalt: cruces ortogonales a 90° y paneles rectos).
"""

from __future__ import annotations

from typing import Callable

try:
    from . import palette
except ImportError:
    import palette  # type: ignore[no-redef]

TEXTURE_SIZE: int = 16

BUILT_KEYS: tuple[str, ...] = (
    "team_block_copper",
    "team_block_cobalt",
    "force_field_copper",
    "force_field_cobalt",
    "ladder",
    "jump_pad",
    "electric",
    "rail",
    "beacon_copper",
    "beacon_cobalt",
    "explosive",
    "bucket_station_copper",
    "bucket_station_cobalt",
    "bucket_post_copper",
    "bucket_post_cobalt",
)


def _render_grid(pixel_fn: Callable[[int, int], str]) -> bytes:
    """Renderiza una textura 16×16 RGBA8 a partir de una función de nombres de color."""
    buf = bytearray(TEXTURE_SIZE * TEXTURE_SIZE * 4)
    offset = 0
    for y in range(TEXTURE_SIZE):
        for x in range(TEXTURE_SIZE):
            r, g, b, a = palette.color(pixel_fn(x, y))
            buf[offset : offset + 4] = (r, g, b, a)
            offset += 4
    return bytes(buf)


def _team_block_copper_pixel(x: int, y: int) -> str:
    """Blindaje pesado Copper: marco remachado y chevrones diagonales a 45°."""
    if (x in (0, 15)) and (y in (0, 15)):
        return "void_black"
    if x == 0 or y == 0:
        return "copper_mid" if (x + y) % 3 != 0 else "steel_mid"
    if x == 15 or y == 15:
        return "steel_dark" if (x + y) % 2 == 0 else "void_black"
    if (x in (2, 13)) and (y in (2, 13)):
        return "copper_glow"
    if x in (1, 14) or y in (1, 14):
        if (x in (2, 3, 12, 13) and y in (1, 14)) or (y in (2, 3, 12, 13) and x in (1, 14)):
            return "steel_dark"
        return "copper_dark"
    if x in (2, 13) or y in (2, 13):
        return "copper_dark" if (x + y) % 2 == 0 else "steel_dark"
    # Interior con chevrones diagonales a 45°
    band = ((x + y) if x < 8 else (15 - x + y)) % 4
    noise = palette.hash2d(x, y, 801) & 3
    if band == 0:
        return "copper_glow" if (y < 7 and noise == 0) else "copper_bright"
    if band == 1:
        return "copper_mid" if noise != 0 else "copper_bright"
    return "copper_dark" if (band == 2 or noise != 0) else "soot_dark"


def _team_block_cobalt_pixel(x: int, y: int) -> str:
    """Blindaje pesado Cobalt: marco biselado recto y nervadura ortogonal en cruz (+)."""
    if x == 0 or y == 0:
        return "steel_light" if (x > 0 and y == 0) else "cobalt_mid"
    if x == 15 or y == 15:
        return "void_black" if (x == 15 and y == 15) else "steel_dark"
    if (x in (1, 14) and y in (7, 8)) or (y in (1, 14) and x in (7, 8)):
        return "cobalt_glow"
    if x in (1, 14) or y in (1, 14):
        return "cobalt_dark" if (x + y) % 2 == 0 else "steel_dark"
    # Nervadura ortogonal central en cruz (+) y 4 paneles rectos
    if x in (7, 8) and y in (7, 8):
        return "cobalt_glow"
    if x in (7, 8) or y in (7, 8):
        return "cobalt_bright" if (x == 7 or y == 7) else "cobalt_mid"
    if x in (6, 9) or y in (6, 9):
        return "cobalt_dark"
    if x in (2, 10) or y in (2, 10):
        return "cobalt_bright"
    if x in (5, 13) or y in (5, 13):
        return "cobalt_dark"
    return "cobalt_bright" if (palette.hash2d(x, y, 802) & 3) == 0 else "cobalt_mid"


def _force_field_copper_pixel(x: int, y: int) -> str:
    """Campo de fuerza Copper: bobinas perimetrales y celosía energética en X."""
    if x in (0, 15) or y in (0, 15):
        if (x in (0, 1, 14, 15)) and (y in (0, 1, 14, 15)):
            return "copper_glow" if (x in (0, 15) and y in (0, 15)) else "copper_bright"
        return "copper_mid" if (x + y) % 4 == 0 else "steel_dark"
    d1, d2 = abs(x - y), abs((15 - x) - y)
    if (d1 == 0 and d2 <= 1) or (d2 == 0 and d1 <= 1) or (x in (4, 11) and y in (4, 11)):
        return "sodium_warm"
    if d1 == 0 or d2 == 0:
        return "copper_glow" if (x + y) % 2 == 0 else "copper_bright"
    if d1 == 1 or d2 == 1:
        return "copper_mid" if (x + y) % 2 == 0 else "copper_dark"
    if d1 == 6 or d2 == 6:
        return "copper_bright" if (x + y) % 2 == 0 else "copper_mid"
    if d1 in (5, 7) or d2 in (5, 7):
        return "copper_dark"
    return "soot_dark" if (palette.hash2d(x, y, 901) & 3) == 0 else "void_black"


def _force_field_cobalt_pixel(x: int, y: int) -> str:
    """Campo de fuerza Cobalt: emisores ortogonales y retícula cuadrada (#)."""
    if x in (0, 15) or y in (0, 15):
        if x in (4, 7, 8, 11) or y in (4, 7, 8, 11):
            return "cobalt_glow" if (x in (4, 11) or y in (4, 11)) else "cobalt_bright"
        return "steel_dark" if (x + y) % 2 == 0 else "cobalt_dark"
    main_x, main_y = x in (4, 11), y in (4, 11)
    mid_x, mid_y = x in (7, 8), y in (7, 8)
    if (main_x and main_y) or (mid_x and mid_y):
        return "electric_blue"
    if (main_x and mid_y) or (mid_x and main_y):
        return "cobalt_glow"
    if main_x or main_y:
        return "cobalt_glow" if (x + y) % 2 == 0 else "cobalt_bright"
    if mid_x or mid_y:
        return "cobalt_bright" if (x + y) % 2 == 0 else "cobalt_mid"
    if x in (3, 5, 10, 12) or y in (3, 5, 10, 12):
        return "cobalt_dark"
    return "soot_dark" if (palette.hash2d(x, y, 902) & 3) == 0 else "void_black"


def _ladder_pixel(x: int, y: int) -> str:
    """Escalera minera de 1931: largueros continuos de madera y 4 peldaños."""
    if x in (2, 3, 12, 13):
        if y in (1, 5, 9, 13) and x in (2, 13):
            return "steel_mid"
        if x in (3, 12):
            return "timber_light" if (palette.hash2d(x, y, 1001) & 3) != 0 else "timber_mid"
        return "timber_mid" if (palette.hash2d(x, y, 1002) & 1) == 0 else "timber_dark"
    if 4 <= x <= 11:
        phase = y % 4
        if phase == 1:
            return "steel_light" if x in (4, 11) else ("timber_light" if x not in (7, 8) else "ochre_dirt")
        if phase == 2:
            return "steel_dark" if x in (4, 11) else ("timber_mid" if (x + y) % 2 == 0 else "timber_dark")
        if phase == 3:
            return "void_black"
    if x in (0, 1, 14, 15) and y in (2, 10):
        return "steel_dark" if x in (0, 15) else "steel_mid"
    h = palette.hash2d(x, y, 1003) & 7
    return "shadow_slate" if h == 0 else ("soot_dark" if h < 4 else "void_black")


def _jump_pad_pixel(x: int, y: int) -> str:
    """Plataforma neumática 1979: chasis de acero y doble flecha ascendente (^)."""
    corner_dist = min(x + y, (15 - x) + y, x + (15 - y), (15 - x) + (15 - y))
    if x in (0, 1, 14, 15) or y in (0, 1, 14, 15):
        if corner_dist <= 3:
            return "sodium_warm" if (x + y) % 2 == 0 else "soot_dark"
        if y == 0 or x == 0:
            return "steel_light" if (x + y) % 2 == 0 else "steel_mid"
        return "steel_dark" if (y == 15 or x == 15) else "steel_mid"
    if x in (2, 13):
        return "steel_light" if y % 2 == 0 else "steel_dark"
    dx = (7 - x) if x <= 7 else (x - 8)
    if dx <= 4:
        rel_top, rel_bot = y - dx, y - dx - 5
        if rel_top == 2 or rel_bot == 2:
            return "lava_core" if dx <= 1 else "gold_gleam"
        if rel_top == 3 or rel_bot == 3:
            return "sodium_warm"
        if rel_top == 4 or rel_bot == 4:
            return "copper_mid"
    if y in (12, 13) and 6 <= x <= 9:
        return "steel_light" if x in (7, 8) else "steel_mid"
    return "void_black" if (x + y) % 2 == 0 else "soot_dark"


def _electric_pixel(x: int, y: int) -> str:
    """Acumulador de alta tensión: aisladores cerámicos, bobina y arco voltaico."""
    if x in (0, 15) or y in (0, 15):
        return "void_black" if (x in (0, 15) and y in (0, 15)) else "steel_dark"
    if (x in (1, 2, 3, 12, 13, 14)) and (y in (1, 2, 3, 12, 13, 14)):
        if (x in (2, 13)) and (y in (2, 13)):
            return "diamond_cyan"
        return "diamond_teal" if (x + y) % 2 == 0 else "steel_mid"
    if x in (1, 14) or y in (1, 14):
        return "steel_mid" if (x + y) % 3 == 0 else "steel_dark"
    if x in (2, 3, 12, 13) and 5 <= y <= 10:
        return ("copper_bright" if x in (3, 12) else "copper_mid") if y % 2 == 0 else "copper_dark"
    bolt_x = (8, 8, 7, 6, 5, 7, 9, 10, 8, 6, 5, 6, 7, 8, 8, 8)[y]
    dist = abs(x - bolt_x)
    if dist == 0:
        return "diamond_white"
    if dist == 1:
        return "diamond_ice" if y % 2 == 0 else "electric_blue"
    if dist == 2 or (y in (7, 8) and 4 <= x <= 11):
        return "electric_blue" if (x + y) % 2 == 0 else "cobalt_bright"
    if dist == 3 and (palette.hash2d(x, y, 1201) & 1) == 0:
        return "cobalt_dark"
    return "void_black" if (palette.hash2d(x, y, 1202) & 3) != 0 else "soot_dark"


def _rail_pixel(x: int, y: int) -> str:
    """Vía de vagoneta 1931: balasto de pizarra, traviesas y carriles continuos."""
    if x in (4, 10):
        return "steel_light" if (palette.hash2d(x, y, 1301) & 7) != 0 else "shale_highlight"
    if x in (5, 11):
        return "steel_mid" if y % 2 == 0 else "steel_dark"
    tie_phase = y % 4
    if 1 <= x <= 14 and tie_phase in (1, 2):
        if x in (3, 6, 9, 12):
            return "steel_dark" if tie_phase == 2 else "steel_mid"
        if tie_phase == 1:
            return "timber_light" if (palette.hash2d(x, y, 1302) & 3) == 0 else "timber_mid"
        return "timber_dark" if (x + y) % 3 != 0 else "peat_dark"
    if (1 <= x <= 14 and tie_phase == 3) or x in (0, 15):
        return "void_black" if (x + y) % 2 == 0 else "soot_dark"
    return ("soot_dark", "basalt_dark", "slate_dark", "shadow_slate")[palette.hash2d(x, y, 1303) & 3]


def _beacon_pixel(x: int, y: int, is_copper: bool) -> str:
    """Baliza direccional con lente Fresnel y jaula de equipo (45° vs 90°)."""
    corner = min(x + y, (15 - x) + y, x + (15 - y), (15 - x) + (15 - y))
    if corner < 3:
        return "void_black" if corner < 2 else "steel_dark"
    if x in (0, 15) or y in (0, 15) or corner == 3:
        return "steel_light" if y == 0 else "steel_dark"
    dx, dy = x - 7.5, y - 7.5
    r2 = dx * dx + dy * dy
    if is_copper:
        is_rib = abs(abs(dx) - abs(dy)) <= 0.5
        if (x in (2, 13)) and (y in (2, 13)):
            return "copper_glow"
        if r2 <= 2.5:
            return "lava_core"
        if r2 <= 6.5:
            return "lava_yellow" if is_rib else "sodium_warm"
        if r2 <= 13.0:
            return "copper_glow" if is_rib else "copper_bright"
        if r2 <= 22.0:
            return "copper_bright" if is_rib else "copper_mid"
        if r2 <= 30.0:
            return "copper_mid" if is_rib else "copper_dark"
        return "steel_mid" if (x + y) % 2 == 0 else "steel_dark"
    is_rib = abs(dx) <= 0.6 or abs(dy) <= 0.6
    if (x in (1, 14) and y in (7, 8)) or (y in (1, 14) and x in (7, 8)):
        return "cobalt_glow"
    if r2 <= 2.5:
        return "diamond_white"
    if r2 <= 6.5:
        return "electric_blue" if is_rib else "diamond_ice"
    if r2 <= 13.0:
        return "electric_blue" if is_rib else "cobalt_glow"
    if r2 <= 22.0:
        return "cobalt_glow" if is_rib else "cobalt_bright"
    if r2 <= 30.0:
        return "cobalt_bright" if is_rib else "cobalt_mid"
    return "cobalt_dark" if is_rib else "steel_dark"


def _explosive_pixel(x: int, y: int) -> str:
    """Carga de demolición: 3 cartuchos verticales en hazard_red, flejes y mecha."""
    if x in (0, 15) or y in (0, 15):
        return "void_black" if (x in (0, 15) and y in (0, 15)) else "timber_dark"
    if y in (1, 2):
        if x in (7, 8) and y == 1:
            return "lava_core"
        if x in (6, 7, 8, 9):
            return "sodium_warm" if y == 1 else "lava_orange"
        return "copper_bright" if (y == 2 and x in (3, 4, 5, 10, 11, 12)) else "soot_dark"
    if y in (5, 11) and 2 <= x <= 13:
        if x in (5, 10):
            return "steel_dark"
        return "steel_light" if x in (2, 6, 7, 11) else "steel_mid"
    if 3 <= y <= 14:
        if y in (3, 14) and x not in (1, 5, 10, 14):
            return "timber_light" if x in (2, 6, 11) else "timber_mid"
        if x in (2, 6, 11):
            return "sodium_warm" if y == 8 else "lava_orange"
        if x in (3, 7, 8, 12):
            return ("gold_gleam" if x in (7, 8) else "sodium_warm") if y == 8 else "hazard_red"
        if x in (4, 9, 13):
            return "copper_mid" if y == 8 else "copper_dark"
    return "void_black" if (x + y) % 2 == 0 else "soot_dark"


def _bucket_station_pixel(x: int, y: int, is_copper: bool) -> str:
    """Estación motriz de cangilones: bastidor de equipo, tolvas y polea dentada."""
    bright = "copper_bright" if is_copper else "cobalt_bright"
    mid = "copper_mid" if is_copper else "cobalt_mid"
    glow = "copper_glow" if is_copper else "cobalt_glow"
    if x in (0, 1, 2, 13, 14, 15):
        if x in (1, 14) and (y in (1, 14) if is_copper else y in (4, 11)):
            return glow
        if x in (0, 15) or y in (0, 15):
            return "steel_dark"
        return bright if ((x + y) % 3 == 0 if is_copper else y % 3 == 0) else mid
    if y in (0, 1, 14, 15):
        return bright if y in (1, 14) else "steel_dark"
    dx, dy = x - 7.5, y - 7.5
    r2 = dx * dx + dy * dy
    if r2 <= 2.5:
        return "sodium_warm" if is_copper else "electric_blue"
    if r2 <= 6.0:
        return glow
    if 9.0 <= r2 <= 18.5:
        return "steel_light" if (x + y) % 2 == 0 else "steel_mid"
    spoke = abs(abs(dx) - abs(dy)) <= 0.8 if is_copper else (abs(dx) <= 0.6 or abs(dy) <= 0.6)
    if r2 < 9.0 and spoke:
        return bright
    if y in (11, 12) and 6 <= x <= 9:
        return glow if y == 11 else "steel_light"
    if x in (4, 11):
        return "steel_mid" if y % 2 == 0 else "steel_dark"
    seed = 1601 if is_copper else 1602
    return "soot_dark" if (palette.hash2d(x, y, seed) & 1) == 0 else "void_black"


def _bucket_post_pixel(x: int, y: int, is_copper: bool) -> str:
    """Poste de soporte de cangilones: columna continua, cruceta y rodillos guía."""
    glow = "copper_glow" if is_copper else "cobalt_glow"
    if y in (2, 3, 4, 5) and x in (1, 2, 3, 12, 13, 14):
        if x in (2, 13) and y in (3, 4):
            return glow
        return "steel_light" if y in (2, 3) else "steel_mid"
    if y in (3, 4) and 4 <= x <= 11:
        if is_copper:
            return "timber_light" if y == 3 else "timber_mid"
        return "steel_light" if y == 3 else "steel_mid"
    if is_copper and (x in (4, 11)) and y in (5, 6):
        return "copper_mid"
    if (not is_copper) and y == 5 and (x in (4, 11)):
        return "cobalt_mid"
    if x in (5, 10):
        return "steel_mid" if y % 2 == 0 else "steel_dark"
    if 6 <= x <= 9:
        if is_copper:
            if (x in (6, 9)) and (y in (1, 9)):
                return "copper_glow"
            diag = (x + y) % 4
            return ("copper_bright", "copper_mid", "timber_dark", "void_black")[diag]
        if x in (7, 8) and y in (2, 10):
            return "cobalt_glow"
        if y % 4 == 0 or x in (7, 8):
            return "cobalt_bright" if y % 2 == 0 else "cobalt_mid"
        return "cobalt_dark" if (x + y) % 2 == 0 else "timber_dark"
    seed = 1701 if is_copper else 1702
    return "soot_dark" if (palette.hash2d(x, y, seed) & 3) != 0 else "shadow_slate"


_GENERATORS: dict[str, Callable[[int, int], str]] = {
    "team_block_copper": _team_block_copper_pixel,
    "team_block_cobalt": _team_block_cobalt_pixel,
    "force_field_copper": _force_field_copper_pixel,
    "force_field_cobalt": _force_field_cobalt_pixel,
    "ladder": _ladder_pixel,
    "jump_pad": _jump_pad_pixel,
    "electric": _electric_pixel,
    "rail": _rail_pixel,
    "beacon_copper": lambda x, y: _beacon_pixel(x, y, True),
    "beacon_cobalt": lambda x, y: _beacon_pixel(x, y, False),
    "explosive": _explosive_pixel,
    "bucket_station_copper": lambda x, y: _bucket_station_pixel(x, y, True),
    "bucket_station_cobalt": lambda x, y: _bucket_station_pixel(x, y, False),
    "bucket_post_copper": lambda x, y: _bucket_post_pixel(x, y, True),
    "bucket_post_cobalt": lambda x, y: _bucket_post_pixel(x, y, False),
}


def generate_built_textures() -> dict[str, bytes]:
    """Genera el diccionario ordenado con las 15 texturas RGBA8 (16×16) construidas."""
    return {name: _render_grid(_GENERATORS[name]) for name in BUILT_KEYS}
