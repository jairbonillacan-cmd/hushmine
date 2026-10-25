"""Generadores procedurales deterministas de los bloques del modo historia.

Produce las 11 texturas de 16×16 (RGBA8, 1024 bytes) de los IDs de bloque
18..28 de la Convención A ampliada: carpintería minera de 1931 (tablón,
puntal, antorcha, banco, horno, arcón) y materiales naturales del modo
historia (grava, agua, obsidiana, mena de carbón y tronco). Sigue la
paleta cerrada y las bandas de luminancia de ``palette.py``; las texturas
que hacen de terreno (tablón, grava, agua, obsidiana, carbón y tronco)
tienen continuidad toroidal estricta en sus cuatro bordes.
"""

from __future__ import annotations

from typing import Callable

try:
    from . import palette
except ImportError:
    import palette  # type: ignore[no-redef]

TEXTURE_SIZE: int = 16

STORY_KEYS: tuple[str, ...] = (
    "plank",
    "prop_beam",
    "torch",
    "workbench",
    "furnace",
    "chest",
    "gravel",
    "water",
    "obsidian",
    "coal_ore",
    "log",
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


def _torus_delta(a: int, b: int) -> int:
    """Devuelve el desplazamiento con signo más corto (a - b) en un toro de tamaño 16."""
    d = (a - b) % TEXTURE_SIZE
    return d - TEXTURE_SIZE if d > TEXTURE_SIZE // 2 else d


def _plank_pixel(x: int, y: int) -> str:
    """Tablón (ID 18): cuatro tablas horizontales con juntas, vetas y clavos."""
    board = y // 4
    seam = y % 4 == 0
    # Junta vertical de cada tabla, desplazada para que no alineen columnas.
    joint_x = (2 + board * 5) % TEXTURE_SIZE
    joint = _torus_delta(x, joint_x) == 0
    n = palette.hash2d(x, y, 1801) & 7

    if seam:
        return "peat_dark" if (x + y) % 2 == 0 else "timber_dark"
    if joint:
        return "peat_dark"
    # Clavos junto a las juntas verticales, en la fila central de cada tabla.
    if y % 4 == 2 and abs(_torus_delta(x, joint_x)) == 1 and (board + x) % 2 == 0:
        return "steel_mid" if n < 4 else "steel_light"
    # Cara de la tabla clara; la veta y las fisuras la oscurecen por zonas.
    grain = (palette.hash2d(x // 2, board, 1802 + y // 4) & 3) == 0
    if grain and n != 0:
        return "timber_mid"
    if n == 0:
        return "timber_dark"
    return "timber_light"


def _prop_beam_pixel(x: int, y: int) -> str:
    """Puntal (ID 19): marco de madera en periferia y aspas en X con placas de acero."""
    on_frame = x in (0, 1, 14, 15) or y in (0, 1, 14, 15)
    corner = x in (0, 1, 14, 15) and y in (0, 1, 14, 15)
    diag_a = abs(x - y) <= 1
    diag_b = abs((15 - x) - y) <= 1
    n = palette.hash2d(x, y, 1901) & 3

    if corner:
        # Placas de esquina remachadas.
        return "steel_dark" if (x + y) % 2 == 0 else "steel_mid"
    if on_frame:
        return "timber_mid" if n != 0 else "timber_dark"
    if x in (6, 7, 8, 9) and y in (6, 7, 8, 9):
        # Nudo central de acero del cruce de aspas.
        if x in (7, 8) and y in (7, 8):
            return "steel_light"
        return "steel_mid" if n != 0 else "steel_dark"
    if diag_a or diag_b:
        return "timber_light" if n != 0 else "ochre_dirt"
    return "timber_mid" if n != 0 else "timber_dark"


def _torch_pixel(x: int, y: int) -> str:
    """Antorcha (ID 20): placa de hollín con antorcha encendida, emblema del prospector."""
    n = palette.hash2d(x, y, 2001) & 3
    # Palo vertical.
    if x in (7, 8) and 5 <= y <= 13:
        if y in (6, 7):
            # Atadura de la mecha.
            return "ochre_dirt" if x == 7 else "peat_dark"
        return "timber_mid" if (x + y) % 2 == 0 else "timber_light"
    # Llama: núcleo blanco, cuerpo amarillo y borde anaranjado.
    if 3 <= y <= 4 and x in (7, 8):
        return "lava_core"
    if y == 2 and x in (7, 8):
        return "lava_yellow"
    if y == 5 and x in (6, 7, 8, 9):
        return "lava_orange"
    if y in (3, 4) and x in (6, 9):
        return "lava_yellow" if n != 0 else "lava_orange"
    if y in (1, 2) and x in (6, 7, 8, 9) and not (y == 1 and x in (7, 8)):
        return "sodium_warm" if n != 0 else "lava_orange"
    # Halo tenue de la llama sobre la placa.
    if 2 <= y <= 6 and 5 <= x <= 10:
        return "soot_dark" if n == 0 else "shadow_slate"
    return "soot_dark" if n != 0 else "void_black"


def _workbench_pixel(x: int, y: int) -> str:
    """Banco de trabajo (ID 21): tablero superior claro y frente con herramientas."""
    n = palette.hash2d(x, y, 2101) & 3
    # Marco perimetral de tablones.
    if x in (0, 15) or y in (0, 15):
        return "timber_dark" if (x + y) % 2 == 0 else "peat_dark"
    # Tablero superior con marcas de regla.
    if 1 <= y <= 4:
        if y == 4:
            return "steel_dark" if x % 4 == 2 else "steel_mid"
        if x % 4 == 2 and y == 2:
            return "slate_mid"
        return "timber_light" if n != 0 else "timber_mid"
    # Frente liso con juntas de tablas; las herramientas van encima.
    tool = _workbench_tool_pixel(x, y, n)
    if tool is not None:
        return tool
    if x in (5, 10):
        return "peat_dark"
    return "timber_mid"


def _workbench_tool_pixel(x: int, y: int, n: int) -> str | None:
    """Silueta del martillo y el formón del frente del banco; `None` es fondo."""
    # Martillo: cabeza de acero de 5×2 y mango en diagonal.
    if 3 <= x <= 7 and y in (7, 8):
        if y == 7:
            return "steel_light" if x not in (3, 7) else "steel_mid"
        return "steel_mid" if x not in (3, 7) else "steel_dark"
    if (x, y) in ((7, 9), (8, 10), (9, 11), (10, 12)):
        return "timber_dark" if (x + y) % 2 == 0 else "peat_dark"
    # Formón: hoja clara, manguito de latón y puntera oscura.
    if x == 12 and 7 <= y <= 13:
        if y <= 9:
            return "steel_light"
        if y == 10:
            return "gold_bright" if n != 0 else "gold_gleam"
        if y == 11:
            return "gold_shadow"
        return "steel_dark"
    if x == 13 and y in (10, 11):
        return "gold_shadow"
    return None


def _furnace_pixel(x: int, y: int) -> str:
    """Horno (ID 22): mampostería de pizarra con boca encendida y dintel de acero."""
    n = palette.hash2d(x, y, 2201) & 3
    course = y // 4
    # Aparejo corrido: juntas verticales desplazadas media pieza por hilada.
    joint = _torus_delta(x, (3 + 4 * course) % TEXTURE_SIZE) == 0 and y % 4 != 0
    in_mouth = 4 <= x <= 11 and 8 <= y <= 13 and not (x in (4, 11) and y == 8)

    if in_mouth:
        # Hogar: brasas al fondo con lenguas arriba.
        flame = palette.hash2d(x, y, 2202) % 5
        if y >= 12 or flame == 0:
            return "lava_orange" if flame < 3 else "lava_yellow"
        if flame == 1:
            return "lava_yellow"
        return "magma_crust" if flame == 2 else "lava_red"
    if x in (4, 11) and y in (8, 13):
        return "basalt_deep"
    if y == 14 and 4 <= x <= 11:
        return "basalt_deep"
    # Dintel de acero sobre la boca con remaches.
    if y in (6, 7) and 3 <= x <= 12:
        if y == 6 and x % 3 == 0:
            return "steel_light"
        return "steel_dark" if (x + y) % 2 == 0 else "steel_mid"
    # Humareda: dos respiraderos en la coronación.
    if y in (1, 2) and x in (4, 5, 10, 11):
        return "void_black" if (x + y) % 2 == 0 else "soot_dark"
    if y % 4 == 0 or joint:
        return "shadow_slate"
    if n == 0:
        return "slate_light"
    return "slate_mid" if (x + y) % 2 == 0 else "slate_dark"


def _chest_pixel(x: int, y: int) -> str:
    """Arcón (ID 23): cofre de madera con flejes de acero y cerradura de latón."""
    n = palette.hash2d(x, y, 2301) & 3
    if x in (0, 15) or y in (0, 15):
        return "void_black" if (x + y) % 2 == 0 else "soot_dark"
    # Tapa: tablas claras con sombra en la bisagra.
    if 1 <= y <= 5:
        if y == 5:
            return "timber_dark"
        if y % 2 == 1:
            return "timber_light"
        return "timber_mid" if n != 0 else "timber_light"
    # Flejes verticales de acero.
    if x in (3, 12) and y >= 6:
        if y in (7, 13):
            return "steel_light"
        return "steel_dark" if (x + y) % 2 == 0 else "steel_mid"
    # Cerradura de latón centrada sobre la junta de la tapa.
    if 6 <= x <= 9 and 6 <= y <= 9:
        if (x, y) == (7, 8) or (x, y) == (8, 8):
            return "void_black"
        if x in (6, 9) or y in (6, 9):
            return "gold_shadow"
        return "gold_bright" if n != 0 else "gold_gleam"
    # Cuerpo: tablas verticales con veta clara.
    if y == 14:
        return "timber_dark"
    if x % 5 == 2 and n != 0:
        return "timber_light"
    return "timber_mid"


def _gravel_pixel(x: int, y: int) -> str:
    """Grava (ID 24): cantos rodados gruesos encajados entre huecos oscuros."""
    pebbles = ((2, 2), (8, 1), (13, 4), (5, 6), (11, 7), (1, 10), (7, 12), (13, 13))
    n = palette.hash2d(x, y, 2401) & 3
    best, second = 999, 999
    light_from_top = False
    for cx, cy in pebbles:
        dx = _torus_delta(x, cx)
        dy = _torus_delta(y, cy)
        d2 = dx * dx + dy * dy
        if d2 < best:
            second = best
            best = d2
            light_from_top = dy < 0
        elif d2 < second:
            second = d2
    gap = second - best

    if best > 12 or (gap <= 2 and best > 6):
        # Hendedura fina entre cantos.
        return "soot_dark" if n == 0 else "shadow_slate"
    if best <= 2 and light_from_top:
        return "shale_highlight"
    if best <= 8:
        return "slate_light"
    return "slate_light" if n != 0 else "slate_mid"


def _water_pixel(x: int, y: int) -> str:
    """Agua (ID 25): agua profunda con olas cruzadas y destellos de espuma."""
    wave_a = (0, 1, 1, 2, 2, 3, 3, 3, 2, 2, 1, 1, 0, 0, 0, 0)
    wave_b = (2, 2, 1, 1, 1, 0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 2)
    jag = (0, 1, 0, 2, 1, 0, 2, 1, 0, 1, 2, 0, 1, 1, 0, 2)
    n = palette.hash2d(x, y, 2501) & 3
    ripple = palette.hash2d((x + 2 * y) % TEXTURE_SIZE, y, 2502) & 7
    phase_a = (y + wave_a[x] + jag[(y + x // 4) % TEXTURE_SIZE]) % 6
    phase_b = (y + wave_b[x] + 3) % 6

    if phase_a == 0 and n == 0:
        return "diamond_deep"
    if ripple == 0 and n == 0 and phase_a not in (0, 5):
        # Destellos de espuma repartidos, no alineados con las crestas.
        return "diamond_deep"
    if phase_a in (1, 5):
        return "cobalt_mid" if n != 0 else "cobalt_dark"
    if phase_b == 0 and n != 0:
        return "cobalt_mid"
    return "cobalt_dark" if n != 0 else "shadow_slate"


def _obsidian_pixel(x: int, y: int) -> str:
    """Obsidiana (ID 26): vidrio volcánico negro con facetas y destellos fríos."""
    facet_x = (0, 0, 1, 2, 2, 1, 0, 0, 1, 2, 3, 3, 2, 1, 0, 0)
    facet = (y + facet_x[x]) % 6
    n = palette.hash2d(x, y, 2601) & 7

    if facet == 0:
        return "void_black"
    if facet == 3 and n == 0:
        # Filo de la faceta que atrapa algo de luz.
        return "basalt_mid"
    if n == 0:
        return "basalt_deep"
    if n == 7 and facet in (2, 4):
        # Destello violeta-azulado muy puntual.
        return "cobalt_dark"
    return "soot_dark" if (x + y) % 2 == 0 else "void_black"


def _coal_ore_pixel(x: int, y: int) -> str:
    """Mena de carbón (ID 27): pizarra con matas gruesas de antracita."""
    chunks = ((3, 3), (11, 2), (7, 8), (13, 11), (2, 13), (10, 13))
    n = palette.hash2d(x, y, 2701) & 7
    # Distancia de Manhattan mínima al centro de cada mata (toroidal).
    man = min(
        abs(_torus_delta(x, kx)) + abs(_torus_delta(y, ky)) for kx, ky in chunks
    )

    if man <= 1:
        # Corazón mate del carbón con brillo ocasional.
        return "basalt_mid" if n == 0 else "void_black"
    if man == 2:
        return "soot_dark" if n != 0 else "void_black"
    if man == 3 and n < 2:
        # Borde irregular: la mata muerde la pizarra en dientes sueltos.
        return "soot_dark"
    # Matriz de pizarra menos viva que la de la mena de hierro.
    if n == 0:
        return "slate_light"
    if n == 1:
        return "slate_dark"
    return "slate_mid" if (x + y) % 2 == 0 else "slate_dark"


def _log_pixel(x: int, y: int) -> str:
    """Tronco (ID 28): corteza con costillas verticales y dos nudos con vetas."""
    ridges = (1, 4, 8, 12, 15)
    knots = ((4, 5), (12, 11))
    ridge_dist = min(abs(_torus_delta(x, rx)) for rx in ridges)
    n = palette.hash2d(x, y // 2, 2801) & 3
    knot_sq = min(
        _torus_delta(x, kx) * _torus_delta(x, kx) + _torus_delta(y, ky) * _torus_delta(y, ky)
        for kx, ky in knots
    )

    if knot_sq == 0:
        return "peat_dark"
    if knot_sq <= 2:
        return "ochre_dirt" if n != 0 else "timber_light"
    if ridge_dist == 0:
        # Surco profundo entre costillas.
        return "peat_dark" if (y + n) % 2 == 0 else "timber_dark"
    if ridge_dist == 1:
        return "timber_dark"
    if ridge_dist == 2 and n == 0:
        return "timber_light"
    return "timber_mid" if n != 0 else "timber_dark"


_GENERATORS: dict[str, Callable[[int, int], str]] = {
    "plank": _plank_pixel,
    "prop_beam": _prop_beam_pixel,
    "torch": _torch_pixel,
    "workbench": _workbench_pixel,
    "furnace": _furnace_pixel,
    "chest": _chest_pixel,
    "gravel": _gravel_pixel,
    "water": _water_pixel,
    "obsidian": _obsidian_pixel,
    "coal_ore": _coal_ore_pixel,
    "log": _log_pixel,
}


def generate_story_textures() -> dict[str, bytes]:
    """Genera el diccionario ordenado con las 11 texturas RGBA8 (16×16) del modo historia."""
    return {name: _render_grid(_GENERATORS[name]) for name in STORY_KEYS}
