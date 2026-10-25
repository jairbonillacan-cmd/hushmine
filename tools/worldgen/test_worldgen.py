"""Pruebas de los datos de generación del mundo W1 (data/worldgen/*.ron).

Reutiliza el verificador check_worldgen para validar cotas, partición de capas,
curvas Q16, biomas, excavadores y menas. Se ejecuta desde la raíz del
repositorio con:

    python -m unittest discover -s tools/worldgen -v
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path
from typing import Any

_THIS_DIR = Path(__file__).resolve().parent
if str(_THIS_DIR) not in sys.path:
    sys.path.insert(0, str(_THIS_DIR))

import check_worldgen  # noqa: E402
import ron_lite  # noqa: E402


def _is_int(value: Any) -> bool:
    """Devuelve True si el valor es un int de Python pero no un bool."""
    return isinstance(value, int) and not isinstance(value, bool)


class WorldGenDataTest(unittest.TestCase):
    """Batería de invariantes de los cinco archivos data/worldgen/*.ron."""

    texts: dict[str, str]
    data: dict[str, dict[str, Any]]

    @classmethod
    def setUpClass(cls) -> None:
        """Carga una sola vez el texto crudo y la estructura de los cinco RON."""
        cls.texts = check_worldgen.load_texts()
        cls.data = check_worldgen.load_data()

    def test_worldgen_files_load_without_errors(self) -> None:
        """Los cinco archivos existen y cargan como structs no vacíos."""
        self.assertEqual(sorted(self.data), sorted(check_worldgen.WORLDGEN_FILES))
        for name, parsed in self.data.items():
            self.assertIsInstance(parsed, dict, f"La raíz de {name} debe ser un struct")
            self.assertGreater(len(parsed), 0, f"{name} no puede estar vacío")

    def test_no_decimal_numbers(self) -> None:
        """Ningún archivo contiene literales numéricos con punto decimal."""
        for name, text in self.texts.items():
            self.assertFalse(ron_lite.has_decimal_numbers(text), f"{name} contiene números decimales")

    def test_mundo_extents(self) -> None:
        """mundo.ron: versión, extensión horizontal/vertical y niebla de borde."""
        world = self.data["mundo.ron"]
        actual = (
            world["worldgen_version"], world["x_min"], world["x_max"],
            world["z_min"], world["z_max"], world["y_min"], world["y_max"],
            world["border_fog_warning_blocks"],
        )
        self.assertEqual(
            actual,
            (1, -7_500_000, 7_499_999, -7_500_000, 7_499_999, -4_096, 1_023, 64),
            "Las cotas del mundo no coinciden con W1 (15 M de lado, Y -4096..1023, niebla 64)",
        )

    def test_mundo_layers_partition(self) -> None:
        """mundo.ron: 7 capas con nombres exactos y partición sin huecos ni solapes."""
        layers = self.data["mundo.ron"]["layers"]
        self.assertEqual([layer["name"] for layer in layers], list(check_worldgen.LAYER_NAMES))
        ordered = sorted(layers, key=lambda layer: layer["y_min"])  # el archivo lista arriba a abajo
        covered = (
            ordered[0]["y_min"] == check_worldgen.WORLD_Y_MIN
            and ordered[-1]["y_max"] == check_worldgen.WORLD_Y_MAX
            and all(ordered[i + 1]["y_min"] == ordered[i]["y_max"] + 1 for i in range(len(ordered) - 1))
        )
        self.assertTrue(covered, "Las capas deben cubrir [-4096, 1023] sin huecos ni solapes")

    def test_mundo_sleeper_massif(self) -> None:
        """mundo.ron: macizo del Durmiente con radio 3000, pico 900 y spawn 2000..5000."""
        self.assertEqual(
            self.data["mundo.ron"]["sleeper_massif"],
            {"radius_blocks": 3_000, "peak_y": 900, "spawn_dist_min": 2_000, "spawn_dist_max": 5_000},
        )

    def test_relieve_noise_params(self) -> None:
        """relieve.ron: 6 parámetros con nombres exactos y enteros positivos."""
        params = self.data["relieve.ron"]["noise_params"]
        self.assertEqual([p["name"] for p in params], list(check_worldgen.NOISE_NAMES))
        for param in params:
            for key in ("period_shift", "octaves"):
                self.assertTrue(
                    _is_int(param[key]) and param[key] > 0,
                    f"{param['name']}.{key} debe ser entero positivo (es {param[key]!r})",
                )

    def test_relieve_curves(self) -> None:
        """relieve.ron: curvas con extremos -65536/65536, monotonía estricta y salidas en rango."""
        for curve_name in ("base_height_curve", "slope_factor_curve"):
            curve = self.data["relieve.ron"][curve_name]
            inputs = [point["input_q16"] for point in curve]
            self.assertEqual((inputs[0], inputs[-1]), (-65_536, 65_536), f"{curve_name}: extremos Q16")
            self.assertTrue(
                all(b > a for a, b in zip(inputs, inputs[1:])),
                f"{curve_name}: input_q16 debe ser estrictamente creciente",
            )
            self.assertTrue(
                all(-64 <= point["output"] <= 950 for point in curve),
                f"{curve_name}: las salidas deben estar en [-64, 950]",
            )

    def test_surface_biomes(self) -> None:
        """biomas.ron: 8 biomas de superficie en orden con ids únicos 0..=7."""
        surface = self.data["biomas.ron"]["surface_biomes"]
        self.assertEqual([biome["name"] for biome in surface], list(check_worldgen.SURFACE_BIOME_NAMES))
        self.assertEqual(sorted(biome["biome_id"] for biome in surface), list(range(8)))

    def test_underground_biomes(self) -> None:
        """biomas.ron: 9 subterráneos en orden, ids únicos y cota dentro de su capa."""
        biomes = self.data["biomas.ron"]
        underground = biomes["underground_biomes"]
        self.assertEqual([biome["name"] for biome in underground], list(check_worldgen.UNDERGROUND_BIOME_NAMES))
        self.assertEqual(sorted(biome["biome_id"] for biome in underground), list(range(9)))
        layer_ranges = {
            layer["name"]: (layer["y_min"], layer["y_max"]) for layer in self.data["mundo.ron"]["layers"]
        }
        for biome in underground:
            lo, hi = layer_ranges[biome["layer"]]
            self.assertTrue(
                lo <= biome["y_min"] <= biome["y_max"] <= hi,
                f"{biome['name']}: cota [{biome['y_min']}, {biome['y_max']}] fuera de su capa {biome['layer']!r}",
            )

    def test_block_ids_within_catalog(self) -> None:
        """Todos los ids de bloque (superficie, subsuelo, dominante y mena) están en 0..=17."""
        biomes = self.data["biomas.ron"]
        ids = [biome["surface_block"] for biome in biomes["surface_biomes"]]
        ids += [biome["subsurface_block"] for biome in biomes["surface_biomes"]]
        ids += [biome["dominant_block"] for biome in biomes["underground_biomes"]]
        ids += [material["block_id"] for material in self.data["menas.ron"]["materials"]]
        self.assertTrue(all(0 <= block_id <= 17 for block_id in ids), f"Ids de bloque fuera de 0..=17: {ids}")

    def test_caves_margins_and_carvers(self) -> None:
        """cuevas.ron: márgenes 4/2, 7 excavadores en orden y cotas válidas."""
        caves = self.data["cuevas.ron"]
        self.assertEqual((caves["bedrock_margin"], caves["river_roof_min"]), (4, 2))
        self.assertEqual([carver["name"] for carver in caves["carvers"]], list(check_worldgen.CARVER_NAMES))
        floor = check_worldgen.WORLD_Y_MIN + caves["bedrock_margin"]
        for carver in caves["carvers"]:
            self.assertTrue(
                carver["y_min"] >= floor and carver["y_max"] <= check_worldgen.WORLD_Y_MAX
                and carver["y_min"] <= carver["y_max"],
                f"{carver['name']}: rango de cota [{carver['y_min']}, {carver['y_max']}] inválido",
            )
            self.assertTrue(
                0 <= carver["threshold_q16"] <= 65_536,
                f"{carver['name']}: threshold_q16 fuera de [0, 65536]",
            )
        giant = next(carver for carver in caves["carvers"] if carver["name"] == "giant_chambers")
        self.assertTrue(
            giant["y_min"] >= -640 and giant["y_max"] <= -450,
            "giant_chambers debe quedar confinado a [-640, -450]",
        )

    def test_ore_materials_and_curves(self) -> None:
        """menas.ron: 11 materiales en orden con curvas peso/cota válidas."""
        materials = self.data["menas.ron"]["materials"]
        self.assertEqual([m["material"] for m in materials], list(check_worldgen.ORE_MATERIAL_NAMES))
        for material in materials:
            ys = [point["y"] for point in material["curve"]]
            weights = [point["weight_per_100k"] for point in material["curve"]]
            self.assertTrue(
                all(check_worldgen.WORLD_Y_MIN <= y <= check_worldgen.WORLD_Y_MAX for y in ys)
                and all(b > a for a, b in zip(ys, ys[1:])),
                f"{material['material']}: curva con cotas inválidas o no crecientes",
            )
            self.assertTrue(
                all(_is_int(w) and w >= 0 for w in weights)
                and ys[0] <= -256 and ys[-1] >= -256,
                f"{material['material']}: pesos o extremos de curva inválidos",
            )
            if ys[-1] < 0:
                self.assertEqual(weights[-1], 0, f"{material['material']}: debe cerrar con peso 0 (mena del manto)")

    def test_full_checker_reports_no_errors(self) -> None:
        """El verificador completo run_checks() no devuelve ningún error."""
        self.assertEqual(check_worldgen.run_checks(), [])


if __name__ == "__main__":
    unittest.main()
