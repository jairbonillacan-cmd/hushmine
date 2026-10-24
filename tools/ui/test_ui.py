"""Suite de pruebas automatizadas de interfaz de usuario y marca de HUSHMINE.

Verifica la validez XML/SVG, dimensiones y ausencia de referencias externas en las
10 maquetas y 10 activos de marca, la integridad y pertenencia a la paleta cerrada
de 60 colores de los 50 iconos PNG (16×16 y 32×32), el cumplimiento del contraste
WCAG 2.1 AA (>= 4.5:1) tanto declarativo como geométrico, el determinismo byte a
byte, el modo `--check` de `build_all.py` y los límites de 400 líneas con saltos LF.
"""

from __future__ import annotations

from pathlib import Path
import subprocess
import sys
import unittest
import xml.etree.ElementTree as ET

REPO_ROOT: Path = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.textures import palette, png  # noqa: E402
from tools.ui import brand, build_all, icons, mockups, ui_palette  # noqa: E402

# Los 4 títulos obligatorios de la Crónica posterior a la partida (inglés y español).
CHRONICLE_REQUIRED_TITLES: tuple[tuple[str, str], ...] = (
    ("WAKENER", "DESPERTADOR"),
    ("SLEEP GUARDIAN", "GUARDIAN DEL SUENO"),
    ("MOLE", "TOPO"),
    ("BUCKET THIEF", "LADRON DE CANGILONES"),
)


def _expected_svg_dimensions(filename: str) -> tuple[int, int]:
    """Determina las dimensiones (ancho, alto) esperadas según el nombre del archivo SVG."""
    if filename.endswith("_1920x1080.svg"):
        return (1920, 1080)
    if filename.endswith("_1280x800.svg"):
        return (1280, 800)
    if filename.endswith("_460x215.svg"):
        return (460, 215)
    if filename.endswith("_616x353.svg"):
        return (616, 353)
    if filename.startswith("logo_") and filename.endswith(".svg"):
        return (800, 240)
    if filename == "app_icon.svg":
        return (256, 256)
    raise ValueError(f"Nombre de archivo SVG no reconocido para dimensiones: {filename!r}")


class TestUiAndBrandAssets(unittest.TestCase):
    """Pruebas integrales de verificación para los activos de interfaz y marca (Encargo 10)."""

    def test_svg_validity_viewbox_and_no_external_refs(self) -> None:
        """Verifica XML, viewBox, ausencia de enlaces externos, comentarios Fluent y Crónica."""
        self.assertTrue(mockups.MOCKUPS_DIR.is_dir(), f"Falta directorio: {mockups.MOCKUPS_DIR}")
        self.assertTrue(brand.BRAND_DIR.is_dir(), f"Falta directorio: {brand.BRAND_DIR}")

        mockup_files = sorted(mockups.MOCKUPS_DIR.glob("*.svg"))
        brand_files = sorted(brand.BRAND_DIR.glob("*.svg"))
        self.assertEqual(len(mockup_files), 10, f"Se esperaban 10 maquetas SVG, hay {len(mockup_files)}")
        self.assertEqual(len(brand_files), 10, f"Se esperaban 10 SVG de marca, hay {len(brand_files)}")

        for svg_path in [*mockup_files, *brand_files]:
            content = svg_path.read_text(encoding="utf-8")
            try:
                root = ET.fromstring(content)
            except ET.ParseError as exc:
                self.fail(f"Error de parseo XML en {svg_path.name}: {exc}")

            self.assertTrue(root.tag.endswith("svg"), f"Raíz inválida en {svg_path.name}: {root.tag!r}")
            expected_w, expected_h = _expected_svg_dimensions(svg_path.name)
            expected_vb = f"0 0 {expected_w} {expected_h}"
            self.assertEqual(
                root.attrib.get("viewBox"),
                expected_vb,
                f"viewBox inválido en {svg_path.name}: {root.attrib.get('viewBox')!r} != {expected_vb!r}",
            )
            self.assertEqual(
                root.attrib.get("width"),
                str(expected_w),
                f"Atributo width inválido en {svg_path.name}: {root.attrib.get('width')!r}",
            )
            self.assertEqual(
                root.attrib.get("height"),
                str(expected_h),
                f"Atributo height inválido en {svg_path.name}: {root.attrib.get('height')!r}",
            )

            # Ignora únicamente la declaración estándar del espacio de nombres SVG.
            sanitized_text = content.replace('xmlns="http://www.w3.org/2000/svg"', "")
            self.assertNotIn("http://", sanitized_text, f"Referencia http:// externa en {svg_path.name}")
            self.assertNotIn("https://", sanitized_text, f"Referencia https:// externa en {svg_path.name}")

            for elem in root.iter():
                for attr_name, attr_val in elem.attrib.items():
                    local_attr = attr_name.split("}")[-1]
                    if local_attr in {"href", "src", "style"} or "href" in attr_name:
                        self.assertNotIn("http://", attr_val, f"Atributo {attr_name} externo en {svg_path.name}")
                        self.assertNotIn("https://", attr_val, f"Atributo {attr_name} externo en {svg_path.name}")
                    self.assertNotIn("http://", attr_val, f"Atributo {attr_name} con http:// en {svg_path.name}")
                    self.assertNotIn("https://", attr_val, f"Atributo {attr_name} con https:// en {svg_path.name}")
                for text_fragment in (elem.text, elem.tail):
                    if text_fragment:
                        self.assertNotIn("http://", text_fragment, f"Texto con http:// en {svg_path.name}")
                        self.assertNotIn("https://", text_fragment, f"Texto con https:// en {svg_path.name}")

        # Verifica comentarios Fluent en las 10 maquetas y los 4 títulos en las 2 vistas de Crónica.
        for mockup_path in mockup_files:
            mockup_text = mockup_path.read_text(encoding="utf-8")
            self.assertIn(
                "<!-- ftl: ",
                mockup_text,
                f"La maqueta {mockup_path.name} no incluye comentarios con ids de Fluent (<!-- ftl: ... -->)",
            )

        for chronicle_name in ("chronicle_1920x1080.svg", "chronicle_1280x800.svg"):
            chronicle_path = mockups.MOCKUPS_DIR / chronicle_name
            self.assertTrue(chronicle_path.is_file(), f"No existe la maqueta: {chronicle_path}")
            chronicle_content = chronicle_path.read_text(encoding="utf-8")
            for en_title, es_title in CHRONICLE_REQUIRED_TITLES:
                self.assertIn(en_title, chronicle_content, f"Falta título {en_title!r} en {chronicle_name}")
                self.assertIn(es_title, chronicle_content, f"Falta título {es_title!r} en {chronicle_name}")

    def test_png_icons_dimensions_and_pairs(self) -> None:
        """Verifica que existan los 50 iconos PNG (25 pares 16×16 y 32×32) y sean únicos por tamaño."""
        self.assertEqual(len(ui_palette.ICON_STEMS), 25, "ICON_STEMS debe tener exactamente 25 elementos")
        self.assertEqual(ui_palette.ICON_SIZES, (16, 32), "ICON_SIZES debe ser (16, 32)")
        self.assertTrue(icons.ICONS_DIR.is_dir(), f"No existe el directorio: {icons.ICONS_DIR}")

        png_files = sorted(icons.ICONS_DIR.glob("*.png"))
        self.assertEqual(
            len(png_files),
            50,
            f"Se esperaban exactamente 50 archivos PNG en {icons.ICONS_DIR}, hay {len(png_files)}",
        )

        seen_16: set[bytes] = set()
        seen_32: set[bytes] = set()

        for stem in ui_palette.ICON_STEMS:
            path_16 = icons.ICONS_DIR / f"{stem}_16.png"
            path_32 = icons.ICONS_DIR / f"{stem}_32.png"
            self.assertTrue(path_16.is_file(), f"Falta el icono 16×16: {path_16}")
            self.assertTrue(path_32.is_file(), f"Falta el icono 32×32: {path_32}")

            w16, h16, rgba_16 = png.read_png(path_16)
            self.assertEqual((w16, h16), (16, 16), f"Dimensiones inválidas en {path_16.name}: {(w16, h16)}")
            self.assertEqual(len(rgba_16), 1024, f"Longitud RGBA8 inválida en {path_16.name}: {len(rgba_16)}")
            seen_16.add(rgba_16)

            w32, h32, rgba_32 = png.read_png(path_32)
            self.assertEqual((w32, h32), (32, 32), f"Dimensiones inválidas en {path_32.name}: {(w32, h32)}")
            self.assertEqual(len(rgba_32), 4096, f"Longitud RGBA8 inválida en {path_32.name}: {len(rgba_32)}")
            seen_32.add(rgba_32)

        self.assertEqual(len(seen_16), 25, f"Los 25 iconos de 16×16 deben ser distintos: {len(seen_16)}")
        self.assertEqual(len(seen_32), 25, f"Los 25 iconos de 32×32 deben ser distintos: {len(seen_32)}")

    def test_icon_colors_in_allowed_palettes(self) -> None:
        """Comprueba que las paletas midan 48 + 12 = 60 colores y ningún píxel de icono salga de ellas."""
        self.assertEqual(len(palette.PALETTE_SET), 48, "palette.PALETTE_SET debe tener 48 colores")
        self.assertEqual(len(ui_palette.UI_PALETTE_SET), 12, "ui_palette.UI_PALETTE_SET debe tener 12 colores")
        combined_palette = palette.PALETTE_SET | ui_palette.UI_PALETTE_SET
        self.assertEqual(combined_palette, ui_palette.ALL_PALETTE_SET, "ALL_PALETTE_SET inconsistente")
        self.assertEqual(len(ui_palette.ALL_PALETTE_SET), 60, "ALL_PALETTE_SET debe tener 60 colores")

        png_files = sorted(icons.ICONS_DIR.glob("*.png"))
        self.assertEqual(len(png_files), 50, "Deben inspeccionarse exactamente 50 iconos PNG")

        for png_path in png_files:
            width, height, rgba = png.read_png(png_path)
            for px_idx in range(width * height):
                off = px_idx * 4
                px = (rgba[off], rgba[off + 1], rgba[off + 2], rgba[off + 3])
                self.assertIn(
                    px,
                    combined_palette,
                    f"El píxel #{px_idx} ({px}) en {png_path.name} no pertenece a ALL_PALETTE_SET",
                )

    def test_mockups_wcag_contrast_aa(self) -> None:
        """Verifica contraste WCAG AA (>= 4.5:1) declarativo y geométrico en todos los <text> de maquetas."""
        allowed_hex = {hex_val.strip().upper() for hex_val in ui_palette.ALL_PALETTE_HEX.values()}
        mockup_files = sorted(mockups.MOCKUPS_DIR.glob("*.svg"))
        self.assertEqual(len(mockup_files), 10, "Se esperaban las 10 maquetas SVG para validar WCAG AA")

        for svg_path in mockup_files:
            root = ET.fromstring(svg_path.read_text(encoding="utf-8"))
            prior_rects: list[tuple[float, float, float, float, str]] = []
            text_count = 0

            for elem in root.iter():
                tag = elem.tag.split("}")[-1]
                if tag == "rect":
                    rx, ry = float(elem.attrib["x"]), float(elem.attrib["y"])
                    rw, rh = float(elem.attrib["width"]), float(elem.attrib["height"])
                    rfill = elem.attrib.get("fill", "").strip().upper()
                    self.assertIn(rfill, allowed_hex, f"Color <rect> inválido {rfill!r} en {svg_path.name}")
                    prior_rects.append((rx, ry, rw, rh, rfill))
                elif tag == "text":
                    text_count += 1
                    fill = elem.attrib.get("fill", "").strip().upper()
                    data_bg = elem.attrib.get("data-bg", "").strip().upper()
                    label = (elem.text or "").strip()

                    self.assertIn(fill, allowed_hex, f"fill={fill!r} fuera de paleta en {svg_path.name}")
                    self.assertIn(data_bg, allowed_hex, f"data-bg={data_bg!r} fuera de paleta en {svg_path.name}")

                    declared_ratio = ui_palette.wcag_contrast_ratio(fill, data_bg)
                    self.assertGreaterEqual(
                        declared_ratio,
                        4.5,
                        f"Contraste declarativo < 4.5 en {svg_path.name} ({label!r}): {declared_ratio:.2f}",
                    )

                    tx, ty = float(elem.attrib["x"]), float(elem.attrib["y"])
                    containing_rects = [
                        r
                        for r in prior_rects
                        if r[0] <= tx <= r[0] + r[2] and r[1] <= ty <= r[1] + r[3]
                    ]
                    self.assertTrue(
                        containing_rects,
                        f"Ningún <rect> previo en {svg_path.name} contiene ({tx}, {ty}) para {label!r}",
                    )

                    top_rect_fill = containing_rects[-1][4]
                    geom_ratio = ui_palette.wcag_contrast_ratio(fill, top_rect_fill)
                    self.assertGreaterEqual(
                        geom_ratio,
                        4.5,
                        f"Contraste geométrico < 4.5 en {svg_path.name} ({label!r}): {geom_ratio:.2f}",
                    )

            self.assertGreater(text_count, 0, f"La maqueta {svg_path.name} no contiene elementos <text>")

    def test_deterministic_generation_and_check_mode(self) -> None:
        """Comprueba determinismo en doble ejecución, hoja de contacto y modo `--check` de build_all.py."""
        icons_a = icons.generate_all_icons()
        icons_b = icons.generate_all_icons()
        self.assertEqual(icons_a, icons_b, "icons.generate_all_icons() no es determinista")

        mockups_a = mockups.generate_all_mockups()
        mockups_b = mockups.generate_all_mockups()
        self.assertEqual(mockups_a, mockups_b, "mockups.generate_all_mockups() no es determinista")

        brand_a = brand.generate_all_brand_svgs()
        brand_b = brand.generate_all_brand_svgs()
        self.assertEqual(brand_a, brand_b, "brand.generate_all_brand_svgs() no es determinista")

        sheet_a = build_all.render_ui_contact_sheet(icons_a)
        sheet_b = build_all.render_ui_contact_sheet(icons_b)
        self.assertEqual(sheet_a, sheet_b, "build_all.render_ui_contact_sheet() no es determinista")

        # out/ está en .gitignore: en un checkout limpio (o en CI) la hoja aún
        # no existe, así que solo se compara si ya se generó.
        if build_all.CONTACT_SHEET_PATH.is_file():
            disk_w, disk_h, disk_rgba = png.read_png(build_all.CONTACT_SHEET_PATH)
            self.assertEqual(
                (disk_w, disk_h, disk_rgba),
                sheet_a,
                "La hoja de contacto out/ui_contact.png no coincide con render_ui_contact_sheet()",
            )

        ok, errors = build_all.check_all()
        self.assertTrue(ok, f"build_all.check_all() falló con errores: {errors}")
        self.assertEqual(errors, [], f"Se esperaba lista de errores vacía en check_all(): {errors}")

        proc = subprocess.run(
            [sys.executable, "-B", "tools/ui/build_all.py", "--check"],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(
            proc.returncode,
            0,
            f"tools/ui/build_all.py --check devolvió código {proc.returncode}: {proc.stderr!r}",
        )

    def test_line_limits_and_lf_endings(self) -> None:
        """Verifica que todos los archivos (.py, .md, .svg) en assets/ui, tools/ui y docs/ui usen LF y <= 400 líneas."""
        target_dirs = (
            REPO_ROOT / "assets" / "ui",
            REPO_ROOT / "tools" / "ui",
            REPO_ROOT / "docs" / "ui",
        )
        text_suffixes = {".py", ".md", ".svg"}

        for directory in target_dirs:
            self.assertTrue(directory.is_dir(), f"No existe el directorio: {directory}")
            files = sorted(p for p in directory.rglob("*") if p.is_file() and p.suffix in text_suffixes)
            self.assertGreater(len(files), 0, f"No hay archivos {text_suffixes} en {directory}")
            for file_path in files:
                raw_bytes = file_path.read_bytes()
                rel_path = file_path.relative_to(REPO_ROOT).as_posix()
                self.assertNotIn(
                    b"\r\n",
                    raw_bytes,
                    f"El archivo {rel_path} contiene CRLF (\\r\\n); debe usar LF (\\n)",
                )
                line_count = len(raw_bytes.decode("utf-8").splitlines())
                self.assertLessEqual(
                    line_count,
                    400,
                    f"El archivo {rel_path} supera el límite de 400 líneas: tiene {line_count} líneas",
                )


if __name__ == "__main__":
    unittest.main()
