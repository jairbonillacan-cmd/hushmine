"""Suite de pruebas automatizadas del generador de texturas 16×16 de HUSHMINE.

Verifica las invariantes de formato PNG RGBA8, pertenencia estricta a la paleta
cerrada de 48 colores, cobertura completa de la Convención A, determinismo byte
a byte, separabilidad por luminancia y daltonismo, integridad de `manifest.ron`
y funcionamiento del modo `--check` de `build_all.py`.
"""

from __future__ import annotations

import struct
import subprocess
import sys
import unittest
import zlib
from pathlib import Path

REPO_ROOT: Path = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.textures import build_all, palette, png  # noqa: E402

CONVENTION_A_BLOCKS: dict[int, str] = {
    1: "dirt",
    2: "rock",
    3: "hard_rock",
    4: "ore",
    5: "gold",
    6: "diamond",
    7: "lava",
    8: "team_block",
    9: "force_field",
    10: "ladder",
    11: "jump_pad",
    12: "electric",
    13: "rail",
    14: "beacon",
    15: "explosive",
    16: "bucket_station",
    17: "bucket_post",
    18: "plank",
    19: "prop_beam",
    20: "torch",
    21: "workbench",
    22: "furnace",
    23: "chest",
    24: "gravel",
    25: "water",
    26: "obsidian",
    27: "coal_ore",
    28: "log",
}

TEAM_BLOCKS: frozenset[str] = frozenset(
    {"team_block", "force_field", "beacon", "bucket_station", "bucket_post"}
)


def _apply_filter_scanline(filter_type: int, row: bytes, prev_row: bytes) -> bytes:
    """Codifica una fila RGBA8 con uno de los 5 filtros estándar de PNG (0..4)."""
    stride = len(row)
    encoded = bytearray(1 + stride)
    encoded[0] = filter_type
    for i in range(stride):
        x = row[i]
        a = row[i - 4] if i >= 4 else 0
        b = prev_row[i]
        c = prev_row[i - 4] if i >= 4 else 0
        if filter_type == 0:
            val = x
        elif filter_type == 1:
            val = (x - a) & 0xFF
        elif filter_type == 2:
            val = (x - b) & 0xFF
        elif filter_type == 3:
            val = (x - ((a + b) >> 1)) & 0xFF
        elif filter_type == 4:
            val = (x - png._paeth_predictor(a, b, c)) & 0xFF
        else:
            raise ValueError(f"Filtro inválido: {filter_type}")
        encoded[1 + i] = val
    return bytes(encoded)


def _simulate_buffer(rgba_bytes: bytes, sim_fn: object) -> bytes:
    """Aplica una transformación de daltonismo píxel a píxel sobre un buffer RGBA8."""
    out = bytearray(len(rgba_bytes))
    for idx in range(0, len(rgba_bytes), 4):
        px = (rgba_bytes[idx], rgba_bytes[idx + 1], rgba_bytes[idx + 2], rgba_bytes[idx + 3])
        out[idx : idx + 4] = sim_fn(px)  # type: ignore[operator]
    return bytes(out)


class TestBlockTextures(unittest.TestCase):
    """Pruebas de regresión y validación técnica del atlas de bloques 16×16."""

    def test_all_png_dimensions_and_rgba8(self) -> None:
        """Verifica que los 36 PNG existan, midan 16×16 RGBA8 (1024 B) y el códec PNG funcione."""
        self.assertTrue(
            build_all.BLOCKS_DIR.is_dir(),
            f"No existe el directorio de texturas: {build_all.BLOCKS_DIR}",
        )
        png_files = sorted(build_all.BLOCKS_DIR.glob("*.png"))
        self.assertEqual(len(png_files), 36, f"Se esperaban 36 archivos PNG, hay {len(png_files)}")

        for spec in palette.LAYER_SPECS:
            stem = str(spec["stem"])
            png_path = build_all.BLOCKS_DIR / f"{stem}.png"
            self.assertTrue(png_path.is_file(), f"Falta el archivo PNG de {stem}: {png_path}")
            width, height, rgba = png.read_png(png_path)
            self.assertEqual(
                (width, height), (16, 16), f"Dimensiones inválidas en {stem}.png: {width}×{height}"
            )
            self.assertEqual(
                len(rgba), 1024, f"Longitud RGBA8 inválida en {stem}.png: {len(rgba)} != 1024"
            )
            dec_w, dec_h, dec_rgba = png.decode_png(png.encode_png(width, height, rgba))
            self.assertEqual((dec_w, dec_h), (16, 16), "Fallo de dimensiones tras recodificar PNG")
            self.assertEqual(dec_rgba, rgba, f"Fallo en ciclo encode/decode para {stem}.png")

        # Verifica explícitamente la decodificación de los 5 filtros PNG (0..4).
        w, h, stride = 8, 5, 32
        sample = bytes((idx * 37 + 13) & 0xFF for idx in range(w * h * 4))
        raw_rows, prev = bytearray(), bytes(stride)
        for row_idx in range(h):
            cur = sample[row_idx * stride : (row_idx + 1) * stride]
            raw_rows.extend(_apply_filter_scanline(row_idx % 5, cur, prev))
            prev = cur
        ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)
        custom_png = (
            png.PNG_SIGNATURE
            + png._make_chunk(b"IHDR", ihdr)
            + png._make_chunk(b"IDAT", zlib.compress(bytes(raw_rows), level=9))
            + png._make_chunk(b"IEND", b"")
        )
        fw, fh, f_rgba = png.decode_png(custom_png)
        self.assertEqual((fw, fh), (w, h), "Dimensiones erróneas al decodificar los 5 filtros PNG")
        self.assertEqual(f_rgba, sample, "Reconstrucción errónea de los filtros PNG 0..4")

    def test_no_pixel_outside_palette(self) -> None:
        """Comprueba que la paleta tenga <= 48 colores únicos y ningún píxel salga de ella."""
        self.assertLessEqual(
            len(palette.PALETTE_HEX), 48, f"La paleta supera 48 colores: {len(palette.PALETTE_HEX)}"
        )
        normalized_hex = [val.strip().upper() for val in palette.PALETTE_HEX.values()]
        self.assertEqual(
            len(set(normalized_hex)),
            len(normalized_hex),
            "Existen códigos hexadecimales duplicados en palette.PALETTE_HEX",
        )
        self.assertEqual(
            len(palette.PALETTE_SET),
            len(palette.PALETTE_HEX),
            "El tamaño de PALETTE_SET no coincide con PALETTE_HEX",
        )

        for spec in palette.LAYER_SPECS:
            stem = str(spec["stem"])
            width, height, rgba = png.read_png(build_all.BLOCKS_DIR / f"{stem}.png")
            self.assertEqual(width * height, 256, f"El archivo {stem}.png no contiene 256 píxeles")
            for px_idx in range(256):
                off = px_idx * 4
                pixel = (rgba[off], rgba[off + 1], rgba[off + 2], rgba[off + 3])
                self.assertIn(
                    pixel,
                    palette.PALETTE_SET,
                    f"El píxel #{px_idx} ({pixel}) de {stem}.png no pertenece a PALETTE_SET",
                )

    def test_convention_a_coverage_and_variants(self) -> None:
        """Verifica la cobertura exacta de los IDs 1..28 de la Convención A ampliada y sus variantes."""
        specs = palette.LAYER_SPECS
        self.assertEqual(len(specs), 36, f"Se esperaban 36 capas en LAYER_SPECS, hay {len(specs)}")

        block_ids_in_specs = {int(s["block_id"]) for s in specs}  # type: ignore[arg-type]
        self.assertNotIn(0, block_ids_in_specs, "El bloque 0 (air) no debe tener textura asignada")
        self.assertEqual(
            block_ids_in_specs,
            set(CONVENTION_A_BLOCKS.keys()),
            "Los IDs de bloque en LAYER_SPECS no cubren exactamente los IDs 1..28 de Convención A",
        )
        self.assertFalse(
            (build_all.BLOCKS_DIR / "air.png").exists(),
            "No debe existir un archivo air.png en el directorio de bloques",
        )

        for block_id, expected_name in CONVENTION_A_BLOCKS.items():
            entries = [s for s in specs if int(s["block_id"]) == block_id]  # type: ignore[arg-type]
            self.assertTrue(entries, f"El bloque {block_id} ({expected_name}) no tiene capas")
            for entry in entries:
                self.assertEqual(entry["block_name"], expected_name, f"Nombre erróneo en ID {block_id}")

            if expected_name in TEAM_BLOCKS:
                variants = [entry["variant"] for entry in entries]
                self.assertEqual(
                    variants,
                    ["copper", "cobalt"],
                    f"El bloque de equipo {expected_name} debe tener variantes ['copper', 'cobalt']",
                )
                copper_path = build_all.BLOCKS_DIR / f"{expected_name}_copper.png"
                cobalt_path = build_all.BLOCKS_DIR / f"{expected_name}_cobalt.png"
                self.assertTrue(copper_path.is_file() and cobalt_path.is_file(), f"Falta PNG de {expected_name}")
                self.assertNotEqual(
                    png.read_png(copper_path)[2],
                    png.read_png(cobalt_path)[2],
                    f"Las variantes copper y cobalt de {expected_name} no pueden ser idénticas",
                )
            elif expected_name == "lava":
                frames = [int(entry["frame"]) for entry in entries]  # type: ignore[arg-type]
                self.assertEqual(frames, [0, 1, 2, 3], f"Lava debe tener 4 frames (0..3), tiene {frames}")
                for entry in entries:
                    self.assertIsNone(entry["variant"], "Lava no debe tener variante de equipo")
                lava_buffers = [
                    png.read_png(build_all.BLOCKS_DIR / f"lava_{f}.png")[2] for f in range(4)
                ]
                self.assertEqual(len(set(lava_buffers)), 4, "Los 4 fotogramas de lava deben ser distintos")
            else:
                self.assertEqual(len(entries), 1, f"{expected_name} debe tener una única capa")
                self.assertIsNone(entries[0]["variant"], f"{expected_name} no debe tener variante")
                self.assertEqual(entries[0]["frame"], 0, f"{expected_name} debe tener frame 0")

    def test_deterministic_generation(self) -> None:
        """Comprueba que la generación y codificación PNG sean 100 % deterministas."""
        run_a = build_all.generate_all_textures()
        run_b = build_all.generate_all_textures()
        self.assertEqual(list(run_a.keys()), list(run_b.keys()), "Orden de claves no determinista")

        for stem, rgba_a in run_a.items():
            rgba_b = run_b[stem]
            self.assertEqual(rgba_a, rgba_b, f"El buffer RGBA8 de {stem} difiere entre ejecuciones")
            png_a = png.encode_png(16, 16, rgba_a)
            png_b = png.encode_png(16, 16, rgba_b)
            self.assertEqual(png_a, png_b, f"La codificación PNG de {stem} no es determinista")
            disk_path = build_all.BLOCKS_DIR / f"{stem}.png"
            self.assertEqual(
                disk_path.read_bytes(),
                png_a,
                f"El archivo {disk_path.name} no coincide byte a byte con el PNG generado",
            )

    def test_luminance_distinguishability(self) -> None:
        """Verifica bandas de luminancia, separación >= 15.0 y distinguibilidad en daltonismo."""
        textures = build_all.generate_all_textures()

        for spec in palette.LAYER_SPECS:
            if spec["category"] != "natural":
                continue
            stem, block_name = str(spec["stem"]), str(spec["block_name"])
            low, high = palette.LUMINANCE_BANDS[block_name]
            mean_y = palette.mean_luminance(textures[stem])
            self.assertGreaterEqual(
                mean_y, low, f"Luminancia media de {stem} ({mean_y:.2f}) < mínimo de banda ({low})"
            )
            self.assertLessEqual(
                mean_y, high, f"Luminancia media de {stem} ({mean_y:.2f}) > máximo de banda ({high})"
            )

        key_groups: dict[str, list[str]] = {
            "rock": ["rock"],
            "ore": ["ore"],
            "gold": ["gold"],
            "diamond": ["diamond"],
            "lava": ["lava_0", "lava_1", "lava_2", "lava_3"],
        }
        group_names = list(key_groups.keys())
        for i in range(len(group_names)):
            for j in range(i + 1, len(group_names)):
                for stem_a in key_groups[group_names[i]]:
                    for stem_b in key_groups[group_names[j]]:
                        dist = palette.luminance_distance(textures[stem_a], textures[stem_b])
                        self.assertGreaterEqual(
                            dist,
                            palette.MIN_LUMINANCE_DISTANCE,
                            f"Distancia de luminancia insuficiente entre {stem_a} y {stem_b}: "
                            f"{dist:.2f} < {palette.MIN_LUMINANCE_DISTANCE}",
                        )

        for block_name in sorted(TEAM_BLOCKS):
            copper_rgba = textures[f"{block_name}_copper"]
            cobalt_rgba = textures[f"{block_name}_cobalt"]
            for mode_name, sim_fn in (
                ("protanopia", palette.simulate_protanopia),
                ("deuteranopia", palette.simulate_deuteranopia),
            ):
                sim_copper = _simulate_buffer(copper_rgba, sim_fn)
                sim_cobalt = _simulate_buffer(cobalt_rgba, sim_fn)
                diff_pixels = sum(
                    1
                    for idx in range(0, 1024, 4)
                    if sim_copper[idx : idx + 4] != sim_cobalt[idx : idx + 4]
                )
                self.assertGreaterEqual(
                    diff_pixels,
                    64,
                    f"Variantes de {block_name} poco distinguibles en {mode_name} ({diff_pixels} px)",
                )

    def test_manifest_ron_integrity(self) -> None:
        """Verifica que `manifest.ron` exista, tenga las capas 0..35 sin huecos y coincida con LAYER_SPECS."""
        self.assertTrue(
            build_all.MANIFEST_PATH.is_file(),
            f"No existe el archivo de manifiesto: {build_all.MANIFEST_PATH}",
        )
        content = build_all.MANIFEST_PATH.read_text(encoding="utf-8")
        self.assertEqual(
            content,
            build_all.format_manifest_ron(palette.LAYER_SPECS),
            "El contenido de manifest.ron no coincide exactamente con format_manifest_ron()",
        )

        entries = build_all.parse_manifest_ron(content)
        self.assertEqual(len(entries), 36, f"Se esperaban 36 capas en manifest.ron, hay {len(entries)}")
        layers = [int(entry["layer"]) for entry in entries]  # type: ignore[arg-type]
        self.assertEqual(layers, list(range(36)), f"Capas no correlativas de 0 a 35: {layers}")

        for entry, spec in zip(entries, palette.LAYER_SPECS):
            for key in ("layer", "block_id", "block_name", "variant", "frame"):
                self.assertEqual(
                    entry[key],
                    spec[key],
                    f"Discrepancia en capa {spec['layer']} campo {key!r}: {entry[key]!r} != {spec[key]!r}",
                )

    def test_build_all_check_mode(self) -> None:
        """Comprueba que `build_all.check_all()` y `build_all.py --check` terminen con código 0."""
        ok, errors = build_all.check_all()
        self.assertTrue(ok, f"build_all.check_all() devolvió errores: {errors}")
        self.assertEqual(errors, [], f"La lista de errores debería estar vacía: {errors}")

        script_path = REPO_ROOT / "tools" / "textures" / "build_all.py"
        proc = subprocess.run(
            [sys.executable, "-B", str(script_path), "--check"],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(
            proc.returncode,
            0,
            f"build_all.py --check falló ({proc.returncode}): {proc.stderr!r}",
        )


if __name__ == "__main__":
    unittest.main()
