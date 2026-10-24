"""Integrador y CLI de construcción determinista del atlas de texturas de HUSHMINE.

Genera y verifica de forma reproducible las 25 texturas 16×16 RGBA8 de bloques
(`assets/textures/blocks/*.png`), el manifiesto ordenado de capas
(`assets/textures/blocks/manifest.ron`) y la hoja de contacto de inspección
(`out/textures_contact.png`) usando únicamente la biblioteca estándar de Python.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT: Path = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.textures import built, natural, palette, png  # noqa: E402

TEXTURE_SIZE: int = 16
BLOCKS_DIR: Path = REPO_ROOT / "assets" / "textures" / "blocks"
MANIFEST_PATH: Path = BLOCKS_DIR / "manifest.ron"
CONTACT_SHEET_PATH: Path = REPO_ROOT / "out" / "textures_contact.png"

_MANIFEST_HEADER: str = (
    "// Manifiesto de capas del texture array de bloques de HUSHMINE (16x16 RGBA8, sRGB).\n"
    "// Generado por tools/textures/build_all.py — no editar a mano.\n"
)

_MANIFEST_ENTRY_RE: re.Pattern[str] = re.compile(
    r"^\(\s*"
    r"layer\s*:\s*(\d+)\s*,\s*"
    r"block_id\s*:\s*(\d+)\s*,\s*"
    r'block_name\s*:\s*"([^"]+)"\s*,\s*'
    r'variant\s*:\s*(None|Some\("([^"]+)"\))\s*,\s*'
    r"frame\s*:\s*(\d+)\s*"
    r"\)\s*,?$"
)


def generate_all_textures() -> dict[str, bytes]:
    """Combina y valida las 25 texturas 16×16 RGBA8 en el orden de `LAYER_SPECS`."""
    natural_textures = natural.generate_natural_textures()
    built_textures = built.generate_built_textures()

    overlap = set(natural_textures.keys()) & set(built_textures.keys())
    if overlap:
        raise ValueError(
            f"Claves duplicadas entre generadores natural y built: {sorted(overlap)}"
        )

    combined = {**natural_textures, **built_textures}
    expected_stems = [str(spec["stem"]) for spec in palette.LAYER_SPECS]
    if len(expected_stems) != 25:
        raise ValueError(
            f"Se esperaban exactamente 25 capas en LAYER_SPECS, encontradas: {len(expected_stems)}"
        )

    expected_set = set(expected_stems)
    missing = [stem for stem in expected_stems if stem not in combined]
    extra = [stem for stem in combined if stem not in expected_set]
    if missing or extra:
        raise ValueError(
            f"Desajuste en el conjunto de texturas: faltan={missing}, sobran={extra}"
        )

    expected_byte_len = TEXTURE_SIZE * TEXTURE_SIZE * 4
    ordered: dict[str, bytes] = {}
    for stem in expected_stems:
        rgba = combined[stem]
        if not isinstance(rgba, bytes) or len(rgba) != expected_byte_len:
            raise ValueError(
                f"La textura {stem!r} debe tener {expected_byte_len} bytes RGBA8, "
                f"encontrados: {len(rgba)}"
            )
        ordered[stem] = rgba

    return ordered


def format_manifest_ron(layer_specs: list[dict[str, object]] | None = None) -> str:
    """Formatea el contenido RON determinista de `assets/textures/blocks/manifest.ron`."""
    specs = palette.LAYER_SPECS if layer_specs is None else layer_specs
    lines: list[str] = [_MANIFEST_HEADER.rstrip("\n"), "["]
    for spec in specs:
        layer = int(spec["layer"])  # type: ignore[arg-type]
        block_id = int(spec["block_id"])  # type: ignore[arg-type]
        block_name = str(spec["block_name"])
        variant = spec["variant"]
        frame = int(spec["frame"])  # type: ignore[arg-type]

        variant_ron = "None" if variant is None else f'Some("{variant}")'
        lines.append(
            f'    (layer: {layer}, block_id: {block_id}, block_name: "{block_name}", '
            f"variant: {variant_ron}, frame: {frame}),"
        )
    lines.append("]\n")
    return "\n".join(lines)


def parse_manifest_ron(content: str) -> list[dict[str, object]]:
    """Analiza `manifest.ron` y devuelve la lista de diccionarios de capas."""
    active_lines: list[str] = []
    for raw_line in content.splitlines():
        code_part = raw_line.split("//", 1)[0].strip()
        if code_part:
            active_lines.append(code_part)

    if len(active_lines) < 2 or active_lines[0] != "[" or active_lines[-1] != "]":
        raise ValueError(
            "Formato RON inválido en manifest.ron: se esperaba una lista delimitada por '[' y ']'"
        )

    entries: list[dict[str, object]] = []
    for line in active_lines[1:-1]:
        match = _MANIFEST_ENTRY_RE.match(line)
        if match is None:
            raise ValueError(f"Entrada inválida en manifest.ron: {line!r}")
        layer_str, block_id_str, block_name, variant_raw, variant_inner, frame_str = (
            match.groups()
        )
        variant_val: str | None = None if variant_raw == "None" else variant_inner
        entries.append(
            {
                "layer": int(layer_str),
                "block_id": int(block_id_str),
                "block_name": block_name,
                "variant": variant_val,
                "frame": int(frame_str),
            }
        )
    return entries


def build_all(write_contact_sheet: bool = True) -> None:
    """Escribe las 25 texturas PNG, `manifest.ron` y opcionalmente la hoja de contacto."""
    textures = generate_all_textures()
    BLOCKS_DIR.mkdir(parents=True, exist_ok=True)

    for spec in palette.LAYER_SPECS:
        stem = str(spec["stem"])
        png.write_png(
            BLOCKS_DIR / f"{stem}.png",
            TEXTURE_SIZE,
            TEXTURE_SIZE,
            textures[stem],
        )

    manifest_text = format_manifest_ron(palette.LAYER_SPECS)
    MANIFEST_PATH.write_bytes(manifest_text.encode("utf-8"))

    if write_contact_sheet:
        sheet_items = [
            (str(spec["stem"]), textures[str(spec["stem"])])
            for spec in palette.LAYER_SPECS
        ]
        sheet_w, sheet_h, sheet_rgba = png.render_contact_sheet(sheet_items)
        png.write_png(CONTACT_SHEET_PATH, sheet_w, sheet_h, sheet_rgba)


def check_all() -> tuple[bool, list[str]]:
    """Verifica que los PNG y `manifest.ron` en disco coincidan byte a byte."""
    errors: list[str] = []
    textures = generate_all_textures()
    expected_stems = [str(spec["stem"]) for spec in palette.LAYER_SPECS]
    expected_png_names = {f"{stem}.png" for stem in expected_stems}

    if not BLOCKS_DIR.is_dir():
        errors.append(f"No existe el directorio de bloques: {BLOCKS_DIR}")
        return (False, errors)

    existing_png_names = {p.name for p in BLOCKS_DIR.glob("*.png") if p.is_file()}
    missing_pngs = sorted(expected_png_names - existing_png_names)
    extra_pngs = sorted(existing_png_names - expected_png_names)

    for name in missing_pngs:
        errors.append(f"Falta el archivo PNG esperado: {BLOCKS_DIR / name}")
    for name in extra_pngs:
        errors.append(f"Archivo PNG inesperado en directorio de bloques: {BLOCKS_DIR / name}")

    for stem in expected_stems:
        png_path = BLOCKS_DIR / f"{stem}.png"
        if not png_path.is_file():
            continue
        expected_bytes = png.encode_png(TEXTURE_SIZE, TEXTURE_SIZE, textures[stem])
        actual_bytes = png_path.read_bytes()
        if actual_bytes != expected_bytes:
            errors.append(
                f"El archivo {png_path} no coincide byte a byte con la textura generada en memoria"
            )

    expected_manifest_text = format_manifest_ron(palette.LAYER_SPECS)
    expected_manifest_bytes = expected_manifest_text.encode("utf-8")
    if not MANIFEST_PATH.is_file():
        errors.append(f"Falta el archivo de manifiesto: {MANIFEST_PATH}")
    else:
        actual_manifest_bytes = MANIFEST_PATH.read_bytes()
        if actual_manifest_bytes != expected_manifest_bytes:
            errors.append(
                f"El archivo {MANIFEST_PATH} no coincide byte a byte con el manifiesto esperado"
            )
        else:
            try:
                parsed = parse_manifest_ron(actual_manifest_bytes.decode("utf-8"))
                if len(parsed) != len(palette.LAYER_SPECS):
                    errors.append(
                        f"El manifiesto contiene {len(parsed)} capas, se esperaban {len(palette.LAYER_SPECS)}"
                    )
            except ValueError as exc:
                errors.append(f"Error al analizar {MANIFEST_PATH}: {exc}")

    return (len(errors) == 0, errors)


def main(argv: list[str] | None = None) -> int:
    """Punto de entrada de línea de comandos para generar o verificar las texturas."""
    parser = argparse.ArgumentParser(
        description="Genera o verifica las 25 texturas 16×16 RGBA8 y el manifest.ron de HUSHMINE."
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Verifica que los archivos en disco coincidan byte a byte sin modificarlos.",
    )
    args = parser.parse_args(argv)

    if args.check:
        ok, errors = check_all()
        if ok:
            print(
                "Verificación completada con éxito: las 25 texturas PNG y manifest.ron coinciden byte a byte."
            )
            return 0
        for err in errors:
            print(f"ERROR: {err}", file=sys.stderr)
        return 1

    build_all(write_contact_sheet=True)
    print(
        f"Generadas 25 texturas en {BLOCKS_DIR}, manifiesto en {MANIFEST_PATH} "
        f"y hoja de contacto en {CONTACT_SHEET_PATH}."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
