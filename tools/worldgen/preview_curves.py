"""Previsualización PNG de los datos de generación del mundo W1.

Dibuja out/worldgen_curves.png con tres paneles: curvas de relieve (altura base
y pendiente por índice Q16), franja vertical de capas a escala real de cota y
curvas peso/cota de las menas. Solo biblioteca estándar más ron_lite (parser)
y png/palette (imagen) de solo lectura. Ejecutar desde cualquier sitio:

    python tools/worldgen/preview_curves.py
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

_THIS_DIR = Path(__file__).resolve().parent
_REPO_ROOT = _THIS_DIR.parents[1]
for _candidate in (_THIS_DIR, _REPO_ROOT / "tools" / "balance", _REPO_ROOT / "tools" / "textures"):
    if str(_candidate) not in sys.path:
        sys.path.insert(0, str(_candidate))

import png  # noqa: E402
import ron_lite  # noqa: E402
from palette import PALETTE  # noqa: E402

DATA_DIR = _REPO_ROOT / "data" / "worldgen"
OUTPUT_PATH = _REPO_ROOT / "out" / "worldgen_curves.png"

CANVAS_W, CANVAS_H = 960, 640

# Geometría de los paneles (píxeles): curvas de relieve, franja de capas y menas.
CURVE = {"x0": 70, "x1": 470, "y0": 60, "y1": 300}
STRIP = {"x0": 700, "x1": 768, "y0": 60, "y1": 600}
ORES = {"x0": 70, "x1": 470, "y0": 380, "y1": 600}

# Rótulos de cada capa y colores de dibujo (solo mayúsculas sin acentos en pantalla).
LAYER_COLORS: dict[str, str] = {
    "sky": "slate_light", "surface": "loam_brown", "crust": "ochre_dirt",
    "mantle": "lava_orange", "vault": "steel_dark", "roots": "shadow_slate",
    "bedrock": "soot_dark",
}
ORE_COLORS: tuple[str, ...] = (
    "slate_light", "copper_bright", "shale_highlight", "ore_rust_mid", "steel_light",
    "diamond_ice", "cobalt_bright", "gold_bright", "diamond_cyan", "lava_yellow", "diamond_white",
)


def _color(name: str) -> tuple[int, int, int, int]:
    """Recupera un color de la paleta con reserva segura magenta si falta."""
    return PALETTE.get(name, (255, 0, 255, 255))


class _Scale:
    """Mapeo lineal de un dominio de datos a píxeles de pantalla."""

    def __init__(self, d0: float, d1: float, p0: int, p1: int) -> None:
        self.d0, self.d1, self.p0, self.p1 = d0, d1, p0, p1

    def __call__(self, value: float) -> int:
        ratio = (value - self.d0) / (self.d1 - self.d0)
        return round(self.p0 + ratio * (self.p1 - self.p0))


def _pixel(canvas: bytearray, w: int, h: int, x: float, y: float, rgba: tuple[int, int, int, int]) -> None:
    """Pinta un píxel con recorte seguro."""
    png._fill_rect(canvas, w, h, round(x), round(y), 1, 1, rgba)


def _line(
    canvas: bytearray,
    w: int,
    h: int,
    x0: float,
    y0: float,
    x1: float,
    y1: float,
    rgba: tuple[int, int, int, int],
) -> None:
    """Segmento por interpolación tipo Bresenham: paso por el eje dominante."""
    steps = max(abs(x1 - x0), abs(y1 - y0))
    if steps == 0:
        _pixel(canvas, w, h, x0, y0, rgba)
        return
    for i in range(round(steps) + 1):
        t = i / steps
        _pixel(canvas, w, h, x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, rgba)


def _frame(
    canvas: bytearray,
    w: int,
    h: int,
    x0: int,
    y0: int,
    x1: int,
    y1: int,
    rgba: tuple[int, int, int, int],
) -> None:
    """Rectángulo de 1 px de grosor con png._fill_rect en los cuatro lados."""
    png._fill_rect(canvas, w, h, x0, y0, x1 - x0 + 1, 1, rgba)
    png._fill_rect(canvas, w, h, x0, y1, x1 - x0 + 1, 1, rgba)
    png._fill_rect(canvas, w, h, x0, y0, 1, y1 - y0 + 1, rgba)
    png._fill_rect(canvas, w, h, x1, y0, 1, y1 - y0 + 1, rgba)


def _tick_label(
    canvas: bytearray,
    w: int,
    h: int,
    right_x: int,
    y: int,
    text: str,
    rgba: tuple[int, int, int, int],
) -> None:
    """Rótulo alineado a la derecha de right_x (fuente de 6 px por carácter)."""
    png.draw_text(canvas, w, h, right_x - len(text) * 6, y, text, rgba, 1)


def _plot_curve(canvas: bytearray, points: list[dict[str, Any]], sx: _Scale, sy: _Scale, rgba: tuple[int, int, int, int]) -> None:
    """Dibuja una curva por tramos: interpolación lineal entre puntos de control."""
    projected = [(sx(p["input_q16"]), sy(p["output"])) for p in points]
    for (xa, ya), (xb, yb) in zip(projected, projected[1:]):
        _line(canvas, CANVAS_W, CANVAS_H, xa, ya, xb, yb, rgba)


def _draw_curves_panel(canvas: bytearray, relief: dict[str, Any]) -> None:
    """Panel A: base_height y slope_factor contra el índice combinado Q16."""
    sx = _Scale(-65_536, 65_536, CURVE["x0"], CURVE["x1"])
    sy = _Scale(-100, 1_000, CURVE["y1"], CURVE["y0"])  # dominio con margen sobre [-64, 950]
    grid = _color("slate_dark")
    png._fill_rect(canvas, CANVAS_W, CANVAS_H, sx(0), CURVE["y0"], 1, CURVE["y1"] - CURVE["y0"] + 1, grid)
    png._fill_rect(canvas, CANVAS_W, CANVAS_H, CURVE["x0"], sy(0), CURVE["x1"] - CURVE["x0"] + 1, 1, grid)

    base_rgba = _color("sodium_warm")
    slope_rgba = _color("electric_blue")
    _plot_curve(canvas, relief["base_height_curve"], sx, sy, base_rgba)
    _plot_curve(canvas, relief["slope_factor_curve"], sx, sy, slope_rgba)
    _frame(canvas, CANVAS_W, CANVAS_H, CURVE["x0"], CURVE["y0"], CURVE["x1"], CURVE["y1"], _color("slate_light"))

    # Marcas de eje: entradas Q16 y salidas notables (mínimo, 0 y máximo).
    for value, label in ((-65_536, "-65536"), (0, "0"), (65_536, "65536")):
        x = sx(value)
        png._fill_rect(canvas, CANVAS_W, CANVAS_H, x, CURVE["y1"] + 1, 1, 3, _color("slate_mid"))
        png.draw_text(canvas, CANVAS_W, CANVAS_H, x - len(label) * 3, CURVE["y1"] + 6, label, _color("slate_light"), 1)
    for value in (950, 0, -64):
        y = sy(value)
        png._fill_rect(canvas, CANVAS_W, CANVAS_H, CURVE["x0"] - 3, y, 3, 1, _color("slate_mid"))
        _tick_label(canvas, CANVAS_W, CANVAS_H, CURVE["x0"] - 6, y - 3, str(value), _color("slate_light"))

    # Leyenda con muestras de color para las dos curvas.
    lx, ly = CURVE["x1"] - 138, CURVE["y0"] + 8
    png._fill_rect(canvas, CANVAS_W, CANVAS_H, lx, ly, 130, 30, _color("soot_dark"))
    _frame(canvas, CANVAS_W, CANVAS_H, lx, ly, lx + 129, ly + 29, _color("slate_mid"))
    png._fill_rect(canvas, CANVAS_W, CANVAS_H, lx + 8, ly + 6, 6, 6, base_rgba)
    png.draw_text(canvas, CANVAS_W, CANVAS_H, lx + 20, ly + 5, "BASE HEIGHT", _color("diamond_white"), 1)
    png._fill_rect(canvas, CANVAS_W, CANVAS_H, lx + 8, ly + 19, 6, 6, slope_rgba)
    png.draw_text(canvas, CANVAS_W, CANVAS_H, lx + 20, ly + 18, "SLOPE", _color("diamond_white"), 1)

    png.draw_text(canvas, CANVAS_W, CANVAS_H, CURVE["x0"], CURVE["y0"] + 4, "OUTPUT", _color("slate_mid"), 1)
    png.draw_text(canvas, CANVAS_W, CANVAS_H, (CURVE["x0"] + CURVE["x1"]) // 2 - 24, CURVE["y1"] + 18, "INPUT Q16", _color("slate_mid"), 1)


def _draw_layers_panel(canvas: bytearray, world: dict[str, Any]) -> None:
    """Panel B: franja vertical de las 7 capas a escala real de cota."""
    sy = _Scale(-4_096, 1_023, STRIP["y1"], STRIP["y0"])
    strip_w = STRIP["x1"] - STRIP["x0"] + 1
    for layer in world["layers"]:
        top = sy(layer["y_max"])
        bottom = sy(layer["y_min"])
        height = max(1, bottom - top)  # bedrock mide 1 bloque: franja mínima de 1 px
        rgba = _color(LAYER_COLORS.get(layer["name"], "slate_mid"))
        png._fill_rect(canvas, CANVAS_W, CANVAS_H, STRIP["x0"], top, strip_w, height, rgba)
        label = f"{layer['name'].upper()} {layer['y_min']}..{layer['y_max']}"
        png.draw_text(canvas, CANVAS_W, CANVAS_H, STRIP["x1"] + 8, (top + bottom) // 2 - 3, label, _color("diamond_white"), 1)

    _frame(canvas, CANVAS_W, CANVAS_H, STRIP["x0"], STRIP["y0"], STRIP["x1"], STRIP["y1"], _color("slate_light"))

    # Nivel del mar (cota 0) marcado sobre la franja.
    sea_y = sy(0)
    png._fill_rect(canvas, CANVAS_W, CANVAS_H, STRIP["x0"], sea_y, strip_w, 1, _color("sodium_warm"))
    png.draw_text(canvas, CANVAS_W, CANVAS_H, STRIP["x0"] - 12, sea_y - 3, "0", _color("sodium_warm"), 1)
    png.draw_text(canvas, CANVAS_W, CANVAS_H, STRIP["x0"], STRIP["y0"] - 16, "CAPAS: COTA REAL", _color("diamond_white"), 1)


def _draw_ores_panel(canvas: bytearray, ores: dict[str, Any]) -> None:
    """Panel C: curvas peso por cota de las 11 menas, con leyenda."""
    sx = _Scale(0, 200, ORES["x0"], ORES["x1"])
    sy = _Scale(-4_096, 1_023, ORES["y1"], ORES["y0"])  # cota crece hacia arriba
    grid = _color("slate_dark")
    png._fill_rect(canvas, CANVAS_W, CANVAS_H, sx(0), ORES["y0"], 1, ORES["y1"] - ORES["y0"] + 1, grid)
    png._fill_rect(canvas, CANVAS_W, CANVAS_H, ORES["x0"], sy(0), ORES["x1"] - ORES["x0"] + 1, 1, grid)

    for idx, material in enumerate(ores["materials"]):
        rgba = _color(ORE_COLORS[idx % len(ORE_COLORS)])
        projected = [(sx(p["weight_per_100k"]), sy(p["y"])) for p in material["curve"]]
        for (xa, ya), (xb, yb) in zip(projected, projected[1:]):
            _line(canvas, CANVAS_W, CANVAS_H, xa, ya, xb, yb, rgba)
    _frame(canvas, CANVAS_W, CANVAS_H, ORES["x0"], ORES["y0"], ORES["x1"], ORES["y1"], _color("slate_light"))

    for value in (0, 100, 200):
        x = sx(value)
        png._fill_rect(canvas, CANVAS_W, CANVAS_H, x, ORES["y1"] + 1, 1, 3, _color("slate_mid"))
        png.draw_text(canvas, CANVAS_W, CANVAS_H, x - len(str(value)) * 3, ORES["y1"] + 6, str(value), _color("slate_light"), 1)
    for value in (1_023, 0, -4_096):
        y = sy(value)
        png._fill_rect(canvas, CANVAS_W, CANVAS_H, ORES["x0"] - 3, y, 3, 1, _color("slate_mid"))
        _tick_label(canvas, CANVAS_W, CANVAS_H, ORES["x0"] - 6, y - 3, str(value), _color("slate_light"))

    # Leyenda a la derecha del panel: muestra de color y nombre del material.
    for idx, material in enumerate(ores["materials"]):
        ly = ORES["y0"] + 4 + idx * 16
        rgba = _color(ORE_COLORS[idx % len(ORE_COLORS)])
        png._fill_rect(canvas, CANVAS_W, CANVAS_H, 490, ly, 8, 8, rgba)
        png.draw_text(canvas, CANVAS_W, CANVAS_H, 504, ly, material["material"].upper(), _color("diamond_white"), 1)

    png.draw_text(canvas, CANVAS_W, CANVAS_H, ORES["x0"], ORES["y0"] - 18, "MENAS: PESO POR COTA (POR 100K)", _color("diamond_white"), 1)
    png.draw_text(canvas, CANVAS_W, CANVAS_H, (ORES["x0"] + ORES["x1"]) // 2 - 28, ORES["y1"] + 18, "PESO / 100K", _color("slate_mid"), 1)


def main() -> int:
    """Genera out/worldgen_curves.png a partir de mundo, relieve y menas."""
    mundo = ron_lite.load(DATA_DIR / "mundo.ron")
    relieve = ron_lite.load(DATA_DIR / "relieve.ron")
    menas = ron_lite.load(DATA_DIR / "menas.ron")

    canvas = bytearray(bytes(_color("void_black")) * (CANVAS_W * CANVAS_H))
    png.draw_text(canvas, CANVAS_W, CANVAS_H, 12, 10, "WORLDGEN W1: CURVAS, CAPAS Y MENAS", _color("diamond_white"), 2)
    png.draw_text(canvas, CANVAS_W, CANVAS_H, CURVE["x0"], CURVE["y0"] - 16, "RELIEVE: ALTURA BASE Y PENDIENTE", _color("diamond_white"), 1)
    _draw_curves_panel(canvas, relieve)
    _draw_layers_panel(canvas, mundo)
    _draw_ores_panel(canvas, menas)

    png.write_png(OUTPUT_PATH, CANVAS_W, CANVAS_H, bytes(canvas))
    print(f"OK: previsualización escrita en {OUTPUT_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
