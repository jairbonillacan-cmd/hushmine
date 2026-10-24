"""Extensión de paleta de interfaz (1931/1979), contraste WCAG 2.1 y constantes de UI para HUSHMINE.

Reexporta sin modificar la paleta de 48 colores de bloques (`tools.textures.palette`)
y añade 12 colores específicos para instrumentos de fósforo verde CRT P1 de 1979,
fichas de latón troqueladas de 1931, carteles esmaltados de Anima Consolidated y
papel continuo del sismógrafo y de la Crónica.
"""

from __future__ import annotations

import sys
from pathlib import Path

REPO_ROOT: Path = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.textures.palette import (  # noqa: E402
    PALETTE,
    PALETTE_HEX,
    PALETTE_SET,
    hex_to_rgba,
    relative_luminance,
    simulate_deuteranopia,
    simulate_protanopia,
)

# Extensión de 12 colores para la interfaz de usuario (1931 / 1979) en formato #RRGGBB.
UI_PALETTE_HEX: dict[str, str] = {
    # 1. Fósforo verde CRT P1 (osciloscopio y sismógrafo de 1979) (6)
    "crt_bg": "#07120A",
    "crt_grid": "#12301B",
    "phosphor_dim": "#1E6F36",
    "phosphor_mid": "#34B85A",
    "phosphor_bright": "#68F08E",
    "phosphor_peak": "#C8FFD6",
    # 2. Latón de 1931, esmalte de Anima Consolidated y papel continuo (6)
    "brass_dark": "#594318",
    "brass_mid": "#9E7B2F",
    "brass_bright": "#D9B052",
    "enamel_cream": "#EDE6D3",
    "paper_aged": "#D8CEB8",
    "ink_stamp": "#1C1917",
}

# Diccionario RGBA8 de los 12 colores exclusivos de interfaz.
UI_PALETTE: dict[str, tuple[int, int, int, int]] = {
    name: hex_to_rgba(hex_val) for name, hex_val in UI_PALETTE_HEX.items()
}

# Conjunto inmutable de tuplas RGBA8 de la extensión de interfaz.
UI_PALETTE_SET: frozenset[tuple[int, int, int, int]] = frozenset(UI_PALETTE.values())

# Paleta combinada de 60 colores (48 de bloques + 12 de interfaz).
ALL_PALETTE_HEX: dict[str, str] = {**PALETTE_HEX, **UI_PALETTE_HEX}
ALL_PALETTE: dict[str, tuple[int, int, int, int]] = {**PALETTE, **UI_PALETTE}
ALL_PALETTE_SET: frozenset[tuple[int, int, int, int]] = PALETTE_SET | UI_PALETTE_SET

# Resoluciones objetivo de referencia (1080p escritorio y 800p Steam Deck).
SCREEN_RESOLUTIONS: tuple[tuple[int, int], ...] = ((1920, 1080), (1280, 800))

# Márgenes seguros perimetrales (zona libre del 90 % central) por resolución.
SAFE_MARGINS: dict[tuple[int, int], int] = {
    (1920, 1080): 48,
    (1280, 800): 32,
}

# Identificadores canónicos de las 5 pantallas de maqueta SVG.
MOCKUP_SCREENS: tuple[str, ...] = (
    "hud",
    "drop_in",
    "chronicle",
    "main_menu",
    "loading",
)

# Tamaños canónicos en píxeles de los iconos rasterizados RGBA8.
ICON_SIZES: tuple[int, int] = (16, 32)

# Los 25 identificadores canónicos de iconos de interfaz en orden fijo.
ICON_STEMS: tuple[str, ...] = (
    "class_prospector",
    "class_miner",
    "class_engineer",
    "class_sapper",
    "res_ore",
    "res_gold",
    "res_diamond",
    "hud_bank",
    "item_brass_tally",
    "action_offering",
    "item_explosive",
    "tool_geophone",
    "status_fame",
    "seismo_stir",
    "seismo_breathe",
    "seismo_restless",
    "seismo_wake",
    "death_lava",
    "death_electric",
    "death_fall",
    "death_cave_in",
    "title_wakener",
    "title_sleep_guardian",
    "title_mole",
    "title_bucket_thief",
)


def ui_color(name: str) -> tuple[int, int, int, int]:
    """Devuelve la tupla RGBA8 asociada al nombre de color en ALL_PALETTE."""
    if name in ALL_PALETTE:
        return ALL_PALETTE[name]
    raise KeyError(f"Color no encontrado en ALL_PALETTE: {name!r}")


def ui_hex(name: str) -> str:
    """Devuelve el código hexadecimal #RRGGBB asociado al nombre en ALL_PALETTE_HEX."""
    if name in ALL_PALETTE_HEX:
        return ALL_PALETTE_HEX[name]
    raise KeyError(f"Color no encontrado en ALL_PALETTE_HEX: {name!r}")


def _resolve_rgba(color_or_hex: tuple[int, int, int, int] | str) -> tuple[int, int, int, int]:
    """Normaliza un nombre de color, código #RRGGBB o tupla RGBA8 a una tupla RGBA8."""
    if isinstance(color_or_hex, tuple):
        if len(color_or_hex) == 4:
            return color_or_hex
        if len(color_or_hex) == 3:
            return (color_or_hex[0], color_or_hex[1], color_or_hex[2], 255)
        raise ValueError(f"Tupla de color inválida: {color_or_hex!r}")
    if isinstance(color_or_hex, str):
        cleaned = color_or_hex.strip()
        if cleaned in ALL_PALETTE:
            return ALL_PALETTE[cleaned]
        return hex_to_rgba(cleaned)
    raise TypeError(f"Tipo de color no soportado: {type(color_or_hex)!r}")


def srgb_channel_to_linear(channel_u8: int) -> float:
    """Convierte un canal sRGB de 8 bits (0..255) a luz lineal según el estándar WCAG 2.1."""
    if channel_u8 < 0 or channel_u8 > 255:
        raise ValueError(f"Canal sRGB fuera de rango 0..255: {channel_u8!r}")
    c = channel_u8 / 255.0
    if c <= 0.04045:
        return c / 12.92
    return float(((c + 0.055) / 1.055) ** 2.4)


def wcag_luminance(color_or_hex: tuple[int, int, int, int] | str) -> float:
    """Calcula la luminancia relativa WCAG 2.1 (0.0 a 1.0) de un color sRGB."""
    r, g, b, _ = _resolve_rgba(color_or_hex)
    r_lin = srgb_channel_to_linear(r)
    g_lin = srgb_channel_to_linear(g)
    b_lin = srgb_channel_to_linear(b)
    return 0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin


def wcag_contrast_ratio(
    fg: tuple[int, int, int, int] | str,
    bg: tuple[int, int, int, int] | str,
) -> float:
    """Calcula la razón de contraste WCAG 2.1 (1.0 a 21.0) entre primer plano y fondo."""
    lum_fg = wcag_luminance(fg)
    lum_bg = wcag_luminance(bg)
    l_light = max(lum_fg, lum_bg)
    l_dark = min(lum_fg, lum_bg)
    return (l_light + 0.05) / (l_dark + 0.05)


def check_wcag_aa(
    fg: tuple[int, int, int, int] | str,
    bg: tuple[int, int, int, int] | str,
    minimum: float = 4.5,
) -> bool:
    """Comprueba si el par de colores alcanza el umbral de contraste WCAG 2.1 AA (por defecto 4.5:1)."""
    return wcag_contrast_ratio(fg, bg) >= minimum


__all__ = [
    "ALL_PALETTE",
    "ALL_PALETTE_HEX",
    "ALL_PALETTE_SET",
    "ICON_SIZES",
    "ICON_STEMS",
    "MOCKUP_SCREENS",
    "PALETTE",
    "PALETTE_HEX",
    "PALETTE_SET",
    "SAFE_MARGINS",
    "SCREEN_RESOLUTIONS",
    "UI_PALETTE",
    "UI_PALETTE_HEX",
    "UI_PALETTE_SET",
    "check_wcag_aa",
    "hex_to_rgba",
    "relative_luminance",
    "simulate_deuteranopia",
    "simulate_protanopia",
    "srgb_channel_to_linear",
    "ui_color",
    "ui_hex",
    "wcag_contrast_ratio",
    "wcag_luminance",
]
