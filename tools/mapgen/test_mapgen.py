"""Batería de pruebas unitarias para el generador procedural de mapas y escenas de HUSHMINE.

Verifica determinismo bit a bit del RNG y del generador (aritmética entera pura),
validez de identificadores de bloque y sellado de fronteras (INV-09), equidad
competitiva y seguridad frente a lava en 20 semillas, validez de las 7 escenas
RON de captura, renderizado PNG sin pérdida y cumplimiento de límites de líneas y LF.
"""

from __future__ import annotations

import ast
from pathlib import Path
import subprocess
import sys
import unittest

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.balance import ron_lite  # noqa: E402
from tools.mapgen.analyze import analyze_map  # noqa: E402
from tools.mapgen.generate import (  # noqa: E402
    BLOCK_NAMES,
    HARD_ROCK,
    LAVA,
    generate_map,
)
from tools.mapgen.render_slices import (  # noqa: E402
    render_heightmap,
    render_horizontal_slice,
    render_vertical_slice_x,
    render_vertical_slice_z,
)
from tools.mapgen.rng import Xorshift64, hash3d, splitmix64_step  # noqa: E402
from tools.textures.png import decode_png, encode_png  # noqa: E402

EXPECTED_SCENES: tuple[str, ...] = (
    "tunel_lava.ron",
    "veta_oro.ron",
    "base_copper.ron",
    "base_cobalt.ron",
    "campo_fuerza.ron",
    "cangilones.ron",
    "cueva_oscura.ron",
)


class TestMapgenSuite(unittest.TestCase):
    """Pruebas integrales del generador de mapas, analizador, visualizador y escenas."""

    def test_rng_and_sha256_determinism(self) -> None:
        """Verifica SplitMix64, determinismo SHA-256 por semilla y ausencia de coma flotante."""
        # Coherencia exacta con `crates/hush-sim/src/rng.rs`
        self.assertEqual(splitmix64_step(0), 0xE220A8397B1DCDAF)

        rng_a = Xorshift64(12345)
        rng_b = Xorshift64(12345)
        seq_a = [rng_a.next_u64() for _ in range(32)]
        seq_b = [rng_b.next_u64() for _ in range(32)]
        self.assertEqual(seq_a, seq_b)
        self.assertEqual(hash3d(10, 20, 30, 42), hash3d(10, 20, 30, 42))
        self.assertNotEqual(hash3d(10, 20, 30, 42), hash3d(11, 20, 30, 42))

        # Determinismo bit a bit para la semilla de referencia 42
        gmap_a = generate_map(42)
        gmap_b = generate_map(42)
        self.assertEqual(gmap_a.to_bytes(), gmap_b.to_bytes())
        self.assertEqual(gmap_a.sha256(), gmap_b.sha256())
        self.assertEqual(gmap_a.team_meta, gmap_b.team_meta)

        # Semillas distintas (1..=10) producen volúmenes y hashes todos distintos entre sí
        hashes: set[str] = set()
        payloads: set[bytes] = set()
        for seed in range(1, 11):
            gmap = generate_map(seed)
            hashes.add(gmap.sha256())
            payloads.add(gmap.to_bytes())
        self.assertEqual(len(hashes), 10)
        self.assertEqual(len(payloads), 10)

        # Ausencia total de literales float, división real (/) o llamadas a float()
        for rel_path in ("tools/mapgen/rng.py", "tools/mapgen/generate.py"):
            source = (REPO_ROOT / rel_path).read_text(encoding="utf-8")
            tree = ast.parse(source, filename=rel_path)
            for node in ast.walk(tree):
                if isinstance(node, ast.Constant):
                    self.assertNotIsInstance(
                        node.value,
                        float,
                        f"Literal float prohibido en {rel_path}:{getattr(node, 'lineno', 0)}",
                    )
                if isinstance(node, ast.Div):
                    self.fail(f"División en coma flotante '/' prohibida en {rel_path}")
                if isinstance(node, ast.Call) and isinstance(node.func, ast.Name):
                    self.assertNotEqual(
                        node.func.id,
                        "float",
                        f"Llamada a float() prohibida en {rel_path}",
                    )

    def test_valid_block_ids_and_borders(self) -> None:
        """Verifica rango de bloques 0..=17 y sellado de las 6 caras con hard_rock (INV-09)."""
        for team_size, expected_dims in ((4, (64, 64, 64)), (8, (96, 64, 96))):
            gmap = generate_map(seed=42, team_size=team_size)
            w, h, d = gmap.width, gmap.height, gmap.depth
            self.assertEqual((w, h, d), expected_dims)
            self.assertEqual(len(gmap.blocks), w * h * d)

            # Todo byte del volumen pertenece a 0..=17 (Convención A)
            for b in gmap.blocks:
                self.assertTrue(0 <= b <= 17, f"ID de bloque fuera de rango 0..=17: {b}")

            # Las 6 caras exteriores están íntegramente selladas con HARD_ROCK (id = 3)
            for z in range(d):
                for x in range(w):
                    self.assertEqual(gmap.get_block(x, 0, z), HARD_ROCK)
                    self.assertEqual(gmap.get_block(x, h - 1, z), HARD_ROCK)
            for y in range(h):
                for x in range(w):
                    self.assertEqual(gmap.get_block(x, y, 0), HARD_ROCK)
                    self.assertEqual(gmap.get_block(x, y, d - 1), HARD_ROCK)
                for z in range(d):
                    self.assertEqual(gmap.get_block(0, y, z), HARD_ROCK)
                    self.assertEqual(gmap.get_block(w - 1, y, z), HARD_ROCK)

    def test_twenty_seeds_equity_gold_and_lava_safety(self) -> None:
        """Verifica oro >= 1.5x meta, asimetría <= 10% y distancia a lava >= 6 en 20 semillas."""
        seeds = list(range(1, 21))
        for seed in seeds:
            gmap = generate_map(seed=seed, team_size=4)
            report = analyze_map(gmap)

            target_cash = int(report["target_cash"])  # type: ignore[arg-type]
            c_cash = int(report["copper_reachable_gold_cash"])  # type: ignore[arg-type]
            b_cash = int(report["cobalt_reachable_gold_cash"])  # type: ignore[arg-type]
            diff_pct = float(report["gold_diff_pct"])  # type: ignore[arg-type]

            # Al menos 1,5 veces la meta en cada equipo (comprobado en float y en entero)
            self.assertGreaterEqual(c_cash, 1.5 * target_cash, f"Semilla {seed}: Copper < 1.5x")
            self.assertGreaterEqual(b_cash, 1.5 * target_cash, f"Semilla {seed}: Cobalt < 1.5x")
            self.assertGreaterEqual(c_cash * 2, target_cash * 3, f"Semilla {seed}: Copper int < 1.5x")
            self.assertGreaterEqual(b_cash * 2, target_cash * 3, f"Semilla {seed}: Cobalt int < 1.5x")

            # Diferencia de oro alcanzable <= 10%
            self.assertLessEqual(diff_pct, 10.0, f"Semilla {seed}: diff_pct={diff_pct}% > 10%")
            self.assertLessEqual(
                abs(c_cash - b_cash) * 10,
                max(c_cash, b_cash),
                f"Semilla {seed}: diferencia entera de oro > 10%",
            )

            # Seguridad frente a lava (id = 7) a distancia >= 6 de ambas bases
            self.assertGreater(int(report["block_counts"]["lava"]), 0)  # type: ignore[index]
            self.assertEqual(LAVA, 7)
            self.assertGreaterEqual(int(report["min_lava_dist_copper"]), 6)  # type: ignore[arg-type]
            self.assertGreaterEqual(int(report["min_lava_dist_cobalt"]), 6)  # type: ignore[arg-type]
            self.assertGreaterEqual(int(report["min_lava_dist_manhattan_copper"]), 6)  # type: ignore[arg-type]
            self.assertGreaterEqual(int(report["min_lava_dist_manhattan_cobalt"]), 6)  # type: ignore[arg-type]

            checks = report["checks"]  # type: ignore[assignment]
            self.assertTrue(bool(checks["all_passed"]), f"Semilla {seed} falló checks: {checks}")  # type: ignore[index]

    def test_scenes_ron_valid(self) -> None:
        """Verifica que las 7 escenas .ron existen, se parsean con ron_lite y usan bloques válidos."""
        scenes_dir = REPO_ROOT / "tests" / "scenes"
        self.assertTrue(scenes_dir.is_dir(), "No existe el directorio tests/scenes/")

        for filename in EXPECTED_SCENES:
            scene_path = scenes_dir / filename
            self.assertTrue(scene_path.is_file(), f"Falta la escena esperada: {filename}")

            data = ron_lite.load(scene_path)
            self.assertIsInstance(data, dict)

            for required_key in ("name", "description", "dimensions", "camera", "regions", "blocks"):
                self.assertIn(required_key, data, f"Falta clave {required_key!r} en {filename}")

            self.assertEqual(data["name"], scene_path.stem)
            desc = data["description"]
            self.assertIsInstance(desc, str)
            self.assertGreater(len(desc.strip()), 10, f"Descripción vacía en {filename}")

            dims = data["dimensions"]
            self.assertIsInstance(dims, dict)
            for dim_key in ("width", "height", "depth"):
                self.assertIn(dim_key, dims)
                self.assertIsInstance(dims[dim_key], int)
                self.assertGreater(dims[dim_key], 0)

            cam = data["camera"]
            self.assertIsInstance(cam, dict)
            for cam_key in (
                "position_blocks",
                "position_subblocks",
                "look_at_blocks",
                "yaw_deg",
                "pitch_deg",
                "fov_deg",
            ):
                self.assertIn(cam_key, cam, f"Falta {cam_key!r} en camera de {filename}")

            regions = data["regions"]
            blocks = data["blocks"]
            self.assertIsInstance(regions, list)
            self.assertIsInstance(blocks, list)
            self.assertGreater(len(regions), 0, f"Escena sin regions: {filename}")

            for entry in list(regions) + list(blocks):
                self.assertIsInstance(entry, dict)
                bid = entry["block_id"]
                bname = entry["block_name"]
                self.assertIsInstance(bid, int)
                self.assertTrue(0 <= bid <= 17, f"block_id inválido {bid} en {filename}")
                self.assertEqual(
                    bname,
                    BLOCK_NAMES[bid],
                    f"block_name {bname!r} no coincide con BLOCK_NAMES[{bid}] en {filename}",
                )

    def test_visualizer_and_cli(self) -> None:
        """Verifica cortes PNG sin pérdida, CLI de generate.py y límites de líneas y LF."""
        gmap = generate_map(seed=42, team_size=4)
        slices = (
            render_horizontal_slice(gmap, 52, scale=2),
            render_vertical_slice_z(gmap, gmap.depth // 2, scale=2),
            render_vertical_slice_x(gmap, gmap.width // 2, scale=2),
            render_heightmap(gmap, scale=2),
        )
        for width, height, rgba in slices:
            self.assertGreater(width, 0)
            self.assertGreater(height, 0)
            self.assertEqual(len(rgba), width * height * 4)
            png_bytes = encode_png(width, height, rgba)
            dec_w, dec_h, dec_rgba = decode_png(png_bytes)
            self.assertEqual((dec_w, dec_h), (width, height))
            self.assertEqual(dec_rgba, rgba)

        # Ejecución CLI de generate.py --seed 42 --report
        proc = subprocess.run(
            [sys.executable, str(REPO_ROOT / "tools" / "mapgen" / "generate.py"), "--seed", "42", "--report"],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(proc.returncode, 0, f"Fallo en generate.py CLI: {proc.stderr}")
        self.assertTrue(proc.stdout.strip(), "La salida de generate.py --report está vacía")
        self.assertIn("seed: 42", proc.stdout)

        # Comprobación de <= 400 líneas y saltos de línea LF en todos los archivos del encargo
        checked_files: list[Path] = []
        checked_files.extend(sorted((REPO_ROOT / "tools" / "mapgen").glob("*.py")))
        checked_files.extend(sorted((REPO_ROOT / "docs" / "mapgen").glob("*.md")))
        checked_files.extend(sorted((REPO_ROOT / "tests" / "scenes").glob("*.ron")))
        self.assertGreaterEqual(len(checked_files), 14)

        for file_path in checked_files:
            raw = file_path.read_bytes()
            self.assertNotIn(
                b"\r\n",
                raw,
                f"El archivo {file_path.relative_to(REPO_ROOT)} contiene CRLF (\\r\\n)",
            )
            line_count = len(raw.decode("utf-8").splitlines())
            self.assertLessEqual(
                line_count,
                400,
                f"El archivo {file_path.relative_to(REPO_ROOT)} supera las 400 líneas ({line_count})",
            )


if __name__ == "__main__":
    unittest.main()
