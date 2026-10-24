"""Paleta cerrada de colores, bandas de luminancia y especificación de capas de HUSHMINE.

Define la dirección de arte técnica para las texturas 16×16 del juego (estética
industrial minera 1931/1979 bajo lámparas de vapor de sodio), las métricas de
contraste por luminancia y las utilidades deterministas de validación cromática.
"""

from __future__ import annotations

# Umbral mínimo de distancia de luminancia media (escala 0..255) entre los
# bloques clave de jugabilidad: {rock, ore, gold, diamond, lava}.
MIN_LUMINANCE_DISTANCE: float = 15.0

# Bandas objetivo de luminancia media (escala 0..255) para garantizar por diseño
# que cualquier par entre {rock, ore, gold, diamond, lava} supere
# MIN_LUMINANCE_DISTANCE con al menos 18 puntos de separación entre bandas.
LUMINANCE_BANDS: dict[str, tuple[float, float]] = {
    "hard_rock": (22.0, 38.0),
    "dirt": (40.0, 54.0),
    "rock": (56.0, 70.0),
    "ore": (88.0, 102.0),
    "gold": (120.0, 136.0),
    "diamond": (154.0, 170.0),
    "lava": (188.0, 215.0),
}

# Paleta cerrada de 48 colores únicos en formato #RRGGBB (mayúsculas).
PALETTE_HEX: dict[str, str] = {
    # 1. Sombra, vacío y bordes oscuros (3)
    "void_black": "#0A0B0E",
    "soot_dark": "#14161B",
    "shadow_slate": "#1E222A",
    # 2. Tierra (dirt) y madera vieja de 1931 (6)
    "peat_dark": "#2B2118",
    "loam_brown": "#3B2D20",
    "ochre_dirt": "#4A3826",
    "timber_dark": "#332419",
    "timber_mid": "#4F3826",
    "timber_light": "#6E5037",
    # 3. Roca dura (hard_rock) y roca pizarrosa (rock) (7)
    "basalt_deep": "#16181F",
    "basalt_dark": "#1C1F28",
    "basalt_mid": "#222530",
    "slate_dark": "#353942",
    "slate_mid": "#3D424D",
    "slate_light": "#444A57",
    "shale_highlight": "#505766",
    # 4. Mena ferrosa/cuprífera (ore) (4)
    "ore_rust_dark": "#6E4427",
    "ore_rust_mid": "#8C5830",
    "ore_amber": "#AD6E39",
    "ore_highlight": "#CF8948",
    # 5. Oro nativo (gold) (4)
    "gold_shadow": "#7A5C1E",
    "gold_mid": "#A8822A",
    "gold_bright": "#CCA035",
    "gold_gleam": "#E8BE54",
    # 6. Diamante industrial / cian hielo (diamond) (5)
    "diamond_deep": "#3A788C",
    "diamond_teal": "#4FA0B5",
    "diamond_cyan": "#66C2D6",
    "diamond_ice": "#8CE0F0",
    "diamond_white": "#C4F4FC",
    # 7. Lava e incandescencia magmática (lava) (5)
    "magma_crust": "#8F3B1B",
    "lava_red": "#D95B26",
    "lava_orange": "#F28A2E",
    "lava_yellow": "#FCD153",
    "lava_core": "#FFF08A",
    # 8. Equipo Copper (cobre cálido) (4)
    "copper_dark": "#5E2C18",
    "copper_mid": "#944824",
    "copper_bright": "#C96832",
    "copper_glow": "#F09246",
    # 9. Equipo Cobalt (cobalto frío) (4)
    "cobalt_dark": "#1A2E54",
    "cobalt_mid": "#284C85",
    "cobalt_bright": "#3B73C4",
    "cobalt_glow": "#5CA3F2",
    # 10. Acero industrial y señalización de peligro 1979 (6)
    "steel_dark": "#292E38",
    "steel_mid": "#485060",
    "steel_light": "#6C778C",
    "hazard_red": "#C9302C",
    "electric_blue": "#38B6FF",
    "sodium_warm": "#FAAE3C",
}

def hex_to_rgba(hex_code: str) -> tuple[int, int, int, int]:
    """Convierte un código hexadecimal #RRGGBB en una tupla RGBA8 (r, g, b, 255)."""
    cleaned = hex_code.strip()
    if cleaned.startswith("#"):
        cleaned = cleaned[1:]
    if len(cleaned) != 6:
        raise ValueError(f"Código hexadecimal inválido: {hex_code!r}")
    r = int(cleaned[0:2], 16)
    g = int(cleaned[2:4], 16)
    b = int(cleaned[4:6], 16)
    return (r, g, b, 255)


# Diccionario RGBA8 generado a partir de PALETTE_HEX.
PALETTE: dict[str, tuple[int, int, int, int]] = {
    name: hex_to_rgba(hex_val) for name, hex_val in PALETTE_HEX.items()
}

# Conjunto inmutable de tuplas RGBA8 válidas para verificación O(1).
PALETTE_SET: frozenset[tuple[int, int, int, int]] = frozenset(PALETTE.values())


def color(name: str) -> tuple[int, int, int, int]:
    """Devuelve la tupla RGBA8 asociada al nombre de color en la paleta."""
    if name in PALETTE:
        return PALETTE[name]
    raise KeyError(f"Color no encontrado en la paleta: {name!r}")


def relative_luminance(rgba: tuple[int, int, int, int]) -> float:
    """Calcula la luminancia relativa ITU-R BT.709 en escala 0.0 a 255.0."""
    r, g, b, _ = rgba
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def mean_luminance(rgba_bytes: bytes) -> float:
    """Calcula la luminancia media de un buffer de píxeles RGBA8."""
    if len(rgba_bytes) == 0 or len(rgba_bytes) % 4 != 0:
        raise ValueError("El buffer RGBA8 debe tener longitud múltiplo de 4 y no vacía")
    pixel_count = len(rgba_bytes) // 4
    total = 0.0
    for idx in range(0, len(rgba_bytes), 4):
        r = rgba_bytes[idx]
        g = rgba_bytes[idx + 1]
        b = rgba_bytes[idx + 2]
        total += 0.2126 * r + 0.7152 * g + 0.0722 * b
    return total / pixel_count


def luminance_distance(rgba_a: bytes, rgba_b: bytes) -> float:
    """Devuelve la diferencia absoluta de luminancia media entre dos buffers RGBA8."""
    return abs(mean_luminance(rgba_a) - mean_luminance(rgba_b))


def _clamp_u8(value: float) -> int:
    """Redondea y acota un valor en coma flotante al rango entero 0..255."""
    ival = int(round(value))
    if ival < 0:
        return 0
    if ival > 255:
        return 255
    return ival


def simulate_protanopia(rgba: tuple[int, int, int, int]) -> tuple[int, int, int, int]:
    """Aproxima la visión bajo protanopia mediante la matriz de Viénot (1999)."""
    r, g, b, a = rgba
    rp = _clamp_u8(0.56667 * r + 0.43333 * g)
    gp = _clamp_u8(0.55833 * r + 0.44167 * g)
    bp = _clamp_u8(0.24167 * g + 0.75833 * b)
    return (rp, gp, bp, a)


def simulate_deuteranopia(rgba: tuple[int, int, int, int]) -> tuple[int, int, int, int]:
    """Aproxima la visión bajo deuteranopia mediante la matriz de Viénot (1999)."""
    r, g, b, a = rgba
    rd = _clamp_u8(0.625 * r + 0.375 * g)
    gd = _clamp_u8(0.700 * r + 0.300 * g)
    bd = _clamp_u8(0.300 * g + 0.700 * b)
    return (rd, gd, bd, a)


def hash2d(x: int, y: int, seed: int = 0) -> int:
    """Hash entero determinista de 32 bits basado en TextureData::test_ore."""
    h = (
        ((x & 0xFFFFFFFF) * 0x9E3779B1)
        ^ ((y & 0xFFFFFFFF) * 0x85EBCA77)
        ^ ((seed & 0xFFFFFFFF) * 0xC2B2AE35)
    ) & 0xFFFFFFFF
    h ^= h >> 15
    h = (h * 0x2C1B3C6D) & 0xFFFFFFFF
    return (h ^ (h >> 12)) & 0xFFFFFFFF


def _build_layer_specs() -> list[dict[str, object]]:
    """Construye la lista canónica de 25 capas del texture array (Convención A)."""
    raw_blocks: list[tuple[int, str, bool, int]] = [
        (1, "dirt", False, 1),
        (2, "rock", False, 1),
        (3, "hard_rock", False, 1),
        (4, "ore", False, 1),
        (5, "gold", False, 1),
        (6, "diamond", False, 1),
        (7, "lava", False, 4),
        (8, "team_block", True, 1),
        (9, "force_field", True, 1),
        (10, "ladder", False, 1),
        (11, "jump_pad", False, 1),
        (12, "electric", False, 1),
        (13, "rail", False, 1),
        (14, "beacon", True, 1),
        (15, "explosive", False, 1),
        (16, "bucket_station", True, 1),
        (17, "bucket_post", True, 1),
    ]
    specs: list[dict[str, object]] = []
    layer_idx = 0
    for block_id, block_name, has_teams, frames in raw_blocks:
        category = "natural" if block_id <= 7 else "built"
        if has_teams:
            for variant in ("copper", "cobalt"):
                specs.append(
                    {
                        "layer": layer_idx,
                        "block_id": block_id,
                        "block_name": block_name,
                        "variant": variant,
                        "frame": 0,
                        "stem": f"{block_name}_{variant}",
                        "category": category,
                    }
                )
                layer_idx += 1
        elif frames > 1:
            for frame in range(frames):
                specs.append(
                    {
                        "layer": layer_idx,
                        "block_id": block_id,
                        "block_name": block_name,
                        "variant": None,
                        "frame": frame,
                        "stem": f"{block_name}_{frame}",
                        "category": category,
                    }
                )
                layer_idx += 1
        else:
            specs.append(
                {
                    "layer": layer_idx,
                    "block_id": block_id,
                    "block_name": block_name,
                    "variant": None,
                    "frame": 0,
                    "stem": block_name,
                    "category": category,
                }
            )
            layer_idx += 1
    return specs


LAYER_SPECS: list[dict[str, object]] = _build_layer_specs()
