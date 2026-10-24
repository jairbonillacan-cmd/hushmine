"""Integrador y CLI de construcción determinista de activos de interfaz y marca de HUSHMINE.

Regenera y verifica byte a byte los 50 iconos rasterizados PNG RGBA8
(`assets/ui/icons/`), las 10 maquetas vectoriales SVG (`assets/ui/mockups/`),
los 10 activos SVG de identidad de marca (`assets/ui/brand/`) y la hoja de
contacto de revisión visual (`out/ui_contact.png`) usando únicamente la
biblioteca estándar de Python.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

REPO_ROOT: Path = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.textures.png import _fill_rect, draw_text, encode_png, write_png  # noqa: E402
from tools.ui import brand, icons, mockups  # noqa: E402
from tools.ui.ui_palette import ICON_SIZES, ICON_STEMS, ui_color  # noqa: E402

ICONS_DIR: Path = icons.ICONS_DIR
MOCKUPS_DIR: Path = mockups.MOCKUPS_DIR
BRAND_DIR: Path = brand.BRAND_DIR
CONTACT_SHEET_PATH: Path = REPO_ROOT / "out" / "ui_contact.png"

_GRID_COLUMNS: int = 5
_GRID_ROWS: int = 5
_SCALE_16: int = 6
_SCALE_32: int = 3
_PREVIEW_PX: int = 96


def _blit_scaled_icon(
    canvas: bytearray,
    canvas_w: int,
    canvas_h: int,
    dest_x: int,
    dest_y: int,
    icon_rgba: bytes,
    src_size: int,
    pixel_scale: int,
) -> None:
    """Copia un icono cuadrado RGBA8 de `src_size×src_size` ampliado por `pixel_scale`."""
    for sy in range(src_size):
        row_offset = sy * src_size
        py = dest_y + sy * pixel_scale
        for sx in range(src_size):
            idx = (row_offset + sx) * 4
            color = (icon_rgba[idx], icon_rgba[idx + 1], icon_rgba[idx + 2], icon_rgba[idx + 3])
            _fill_rect(
                canvas,
                canvas_w,
                canvas_h,
                dest_x + sx * pixel_scale,
                py,
                pixel_scale,
                pixel_scale,
                color,
            )


def render_ui_contact_sheet(
    all_icons: dict[tuple[str, int], bytes] | None = None,
) -> tuple[int, int, bytes]:
    """Construye la hoja de contacto 5×5 con los 25 iconos a 16×16 (×6) y 32×32 (×3)."""
    icon_map = icons.generate_all_icons() if all_icons is None else all_icons
    if len(ICON_STEMS) != _GRID_COLUMNS * _GRID_ROWS:
        raise ValueError(
            f"Se esperaban {_GRID_COLUMNS * _GRID_ROWS} iconos en ICON_STEMS, "
            f"encontrados: {len(ICON_STEMS)}"
        )

    inner_gap, pad, header_h, label_area_h = 12, 12, 12, 15
    pair_w = _PREVIEW_PX * 2 + inner_gap
    max_label_px = max(max(0, len(stem) * 6 - 1) for stem in ICON_STEMS)
    content_w = max(pair_w, max_label_px + 8)
    cell_w = content_w + 2 * pad
    cell_h = pad + header_h + _PREVIEW_PX + 10 + label_area_h + pad

    outer_margin, grid_gap = 16, 10
    sheet_w = outer_margin * 2 + _GRID_COLUMNS * cell_w + (_GRID_COLUMNS - 1) * grid_gap
    sheet_h = outer_margin * 2 + _GRID_ROWS * cell_h + (_GRID_ROWS - 1) * grid_gap

    bg_void = ui_color("void_black")
    card_bg = ui_color("soot_dark")
    frame_col = ui_color("slate_dark")
    sub_16_col = ui_color("phosphor_bright")
    sub_32_col = ui_color("sodium_warm")
    label_col = ui_color("enamel_cream")

    canvas = bytearray(bytes(bg_void) * (sheet_w * sheet_h))

    for idx, stem in enumerate(ICON_STEMS):
        rgba_16 = icon_map[(stem, 16)]
        rgba_32 = icon_map[(stem, 32)]
        if len(rgba_16) != 16 * 16 * 4:
            raise ValueError(f"El icono {stem}_16 no mide 16×16 RGBA8")
        if len(rgba_32) != 32 * 32 * 4:
            raise ValueError(f"El icono {stem}_32 no mide 32×32 RGBA8")

        col, row = idx % _GRID_COLUMNS, idx // _GRID_COLUMNS
        cx = outer_margin + col * (cell_w + grid_gap)
        cy = outer_margin + row * (cell_h + grid_gap)

        # Marco exterior de 1 píxel y fondo de tarjeta oscura
        _fill_rect(canvas, sheet_w, sheet_h, cx, cy, cell_w, cell_h, frame_col)
        _fill_rect(canvas, sheet_w, sheet_h, cx + 1, cy + 1, cell_w - 2, cell_h - 2, card_bg)

        lx = cx + (cell_w - pair_w) // 2
        ly = cy + pad + header_h
        rx = lx + _PREVIEW_PX + inner_gap

        # Subtítulos técnicos de resolución y factor de ampliación
        draw_text(canvas, sheet_w, sheet_h, lx, cy + pad, "16X16 X6", sub_16_col, scale=1)
        draw_text(canvas, sheet_w, sheet_h, rx, cy + pad, "32X32 X3", sub_32_col, scale=1)

        # Marcos de 1 píxel alrededor de cada muestra de 96×96 píxeles
        _fill_rect(
            canvas, sheet_w, sheet_h, lx - 1, ly - 1, _PREVIEW_PX + 2, _PREVIEW_PX + 2, frame_col
        )
        _fill_rect(
            canvas, sheet_w, sheet_h, rx - 1, ly - 1, _PREVIEW_PX + 2, _PREVIEW_PX + 2, frame_col
        )

        # Muestra izquierda (16×16 ampliado ×6) y muestra derecha (32×32 ampliado ×3)
        _blit_scaled_icon(canvas, sheet_w, sheet_h, lx, ly, rgba_16, 16, _SCALE_16)
        _blit_scaled_icon(canvas, sheet_w, sheet_h, rx, ly, rgba_32, 32, _SCALE_32)

        # Nombre canónico del icono centrado en la base de la celda
        label_px = max(0, len(stem) * 6 - 1)
        tx = cx + (cell_w - label_px) // 2
        ty = ly + _PREVIEW_PX + 10
        draw_text(canvas, sheet_w, sheet_h, tx, ty, stem, label_col, scale=1)

    return (sheet_w, sheet_h, bytes(canvas))


def build_all(write_contact_sheet: bool = True) -> None:
    """Regenera y escribe en disco los 50 iconos, 10 maquetas, 10 activos de marca y hoja de contacto."""
    icons.write_all_icons(ICONS_DIR)
    mockups.write_all_mockups(MOCKUPS_DIR)
    brand.write_all_brand_svgs(BRAND_DIR)

    if write_contact_sheet:
        sheet_w, sheet_h, sheet_rgba = render_ui_contact_sheet()
        write_png(CONTACT_SHEET_PATH, sheet_w, sheet_h, sheet_rgba)


def _check_directory_files(
    directory: Path,
    expected_files: dict[str, bytes],
    category_label: str,
    errors: list[str],
) -> None:
    """Verifica que `directory` contenga exactamente `expected_files` y coincidan byte a byte."""
    if not directory.is_dir():
        errors.append(f"No existe el directorio de {category_label}: {directory}")
        return

    existing_names = {p.name for p in directory.iterdir() if p.is_file()}
    expected_names = set(expected_files.keys())

    for missing_name in sorted(expected_names - existing_names):
        errors.append(f"Falta archivo esperado en {category_label}: {directory / missing_name}")
    for extra_name in sorted(existing_names - expected_names):
        errors.append(f"Archivo inesperado en {category_label}: {directory / extra_name}")

    for filename, expected_bytes in expected_files.items():
        file_path = directory / filename
        if not file_path.is_file():
            continue
        actual_bytes = file_path.read_bytes()
        if actual_bytes != expected_bytes:
            errors.append(
                f"El archivo {file_path} no coincide byte a byte con el contenido generado en memoria"
            )


def check_all() -> tuple[bool, list[str]]:
    """Regenera todos los activos UI en memoria y los compara byte a byte con el disco."""
    errors: list[str] = []

    raw_icons = icons.generate_all_icons()
    expected_icons: dict[str, bytes] = {}
    for stem in ICON_STEMS:
        for size in ICON_SIZES:
            rgba = raw_icons[(stem, size)]
            expected_icons[f"{stem}_{size}.png"] = encode_png(size, size, rgba)
    _check_directory_files(ICONS_DIR, expected_icons, "iconos PNG", errors)

    expected_mockups = {
        filename: svg_str.encode("utf-8")
        for filename, svg_str in mockups.generate_all_mockups().items()
    }
    _check_directory_files(MOCKUPS_DIR, expected_mockups, "maquetas SVG", errors)

    expected_brand = {
        filename: svg_str.encode("utf-8")
        for filename, svg_str in brand.generate_all_brand_svgs().items()
    }
    _check_directory_files(BRAND_DIR, expected_brand, "marca SVG", errors)

    return (len(errors) == 0, errors)


def main(argv: list[str] | None = None) -> int:
    """Punto de entrada CLI para construir o verificar los activos de interfaz y marca."""
    parser = argparse.ArgumentParser(
        description=(
            "Genera o verifica los 50 iconos PNG, 10 maquetas SVG, 10 activos SVG de marca "
            "y la hoja de contacto de HUSHMINE."
        )
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Verifica que los 70 activos en disco coincidan byte a byte sin modificarlos.",
    )
    args = parser.parse_args(argv)

    if args.check:
        ok, errors = check_all()
        if ok:
            print(
                "Verificación completada con éxito: los 50 iconos PNG, 10 maquetas SVG "
                "y 10 activos SVG de marca coinciden byte a byte."
            )
            return 0
        for err in errors:
            print(f"ERROR: {err}", file=sys.stderr)
        return 1

    build_all(write_contact_sheet=True)
    print(
        f"Generados 50 iconos en {ICONS_DIR}, 10 maquetas en {MOCKUPS_DIR}, "
        f"10 activos de marca en {BRAND_DIR} y hoja de contacto en {CONTACT_SHEET_PATH}."
    )
    return 0


__all__ = [
    "BRAND_DIR",
    "CONTACT_SHEET_PATH",
    "ICONS_DIR",
    "MOCKUPS_DIR",
    "build_all",
    "check_all",
    "main",
    "render_ui_contact_sheet",
]

if __name__ == "__main__":
    raise SystemExit(main())
