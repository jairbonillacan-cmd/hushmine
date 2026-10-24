"""Suite de pruebas de equilibrio, datos RON y simulador sísmico de HUSHMINE.

Verifica la carga sin errores de data/*.ron, la prohibición estricta de números
con punto decimal, las convenciones A (bloques) y C (sismógrafo), la coherencia
cruzada entre catálogos y el ritmo de partida del simulador determinista.
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path
from typing import Any

_THIS_DIR = Path(__file__).resolve().parent
if str(_THIS_DIR) not in sys.path:
    sys.path.insert(0, str(_THIS_DIR))

import ron_lite  # noqa: E402
import seismo_sim  # noqa: E402

REPO_ROOT = _THIS_DIR.parent.parent
DATA_DIR = REPO_ROOT / "data"
REPORT_PATH = REPO_ROOT / "docs" / "design" / "ritmo-partida.md"

RON_FILES: tuple[str, ...] = ("seismograph.ron", "classes.ron", "economy.ron", "blocks.ron")
EXPECTED_BLOCK_NAMES: list[str] = [
    "air", "dirt", "rock", "hard_rock", "ore", "gold", "diamond", "lava",
    "team_block", "force_field", "ladder", "jump_pad", "electric", "rail",
    "beacon", "explosive", "bucket_station", "bucket_post",
]


def _is_pure_int(value: Any) -> bool:
    """Devuelve True si el valor es un int de Python pero no un bool."""
    return isinstance(value, int) and not isinstance(value, bool)


class BalanceDataAndSimulationTest(unittest.TestCase):
    """Pruebas integrales de los datos RON y del simulador de ritmo de partida."""

    data: dict[str, dict[str, Any]]
    raw_texts: dict[str, str]
    sim_results: dict[str, dict[str, Any]]

    @classmethod
    def setUpClass(cls) -> None:
        """Carga una sola vez los cuatro archivos RON y ejecuta el simulador."""
        cls.data = {}
        cls.raw_texts = {}
        for filename in RON_FILES:
            path = DATA_DIR / filename
            cls.raw_texts[filename] = path.read_text(encoding="utf-8")
            cls.data[filename] = ron_lite.load(path)
        cls.sim_results = seismo_sim.run_all_profiles()

    def _assert_no_floats_recursive(self, node: Any, path_str: str) -> None:
        """Recorre recursivamente una estructura verificando que no haya floats."""
        self.assertFalse(isinstance(node, float), f"Se encontró un float prohibido en {path_str}: {node!r}")
        if isinstance(node, (int, float)) and not isinstance(node, bool):
            self.assertIsInstance(node, int, f"El valor numérico en {path_str} debe ser int")
        elif isinstance(node, dict):
            for key, val in node.items():
                self._assert_no_floats_recursive(key, f"{path_str}.<key:{key}>")
                self._assert_no_floats_recursive(val, f"{path_str}.{key}")
        elif isinstance(node, (list, tuple)):
            for idx, item in enumerate(node):
                self._assert_no_floats_recursive(item, f"{path_str}[{idx}]")

    def test_01_load_all_ron_files_without_errors(self) -> None:
        """Verifica que los cuatro archivos .ron existen y se cargan sin errores."""
        for filename in RON_FILES:
            path = DATA_DIR / filename
            self.assertTrue(path.is_file(), f"Falta el archivo requerido: {path}")
            loaded = ron_lite.load(path)
            self.assertIsInstance(loaded, dict, f"La raíz de {filename} debe ser un struct (dict en Python)")
            self.assertGreater(len(loaded), 0, f"El archivo {filename} no puede estar vacío")

    def test_02_no_decimal_numbers_anywhere(self) -> None:
        """Comprueba la ausencia total de números con punto decimal y el rechazo en el parser."""
        for filename, raw_text in self.raw_texts.items():
            self.assertFalse(
                ron_lite.has_decimal_numbers(raw_text),
                f"El archivo {filename} contiene números con punto decimal prohibidos",
            )
            self._assert_no_floats_recursive(self.data[filename], filename)

        for invalid_ron in ("Foo(val: 1.25)", "Foo(val: 0.3)"):
            self.assertTrue(
                ron_lite.has_decimal_numbers(invalid_ron),
                f"has_decimal_numbers debería detectar el decimal en {invalid_ron!r}",
            )
            with self.assertRaises(ValueError, msg=f"ron_lite.loads({invalid_ron!r}) debe lanzar ValueError"):
                ron_lite.loads(invalid_ron)

    def test_03_blocks_convention_a_and_cross_class_coherence(self) -> None:
        """Verifica la Convención A en blocks.ron y la coherencia cruzada con classes.ron."""
        blocks = self.data["blocks.ron"]["blocks"]
        self.assertEqual(len(blocks), 18, "blocks.ron debe contener exactamente 18 bloques (0..17)")
        self.assertEqual([b["id"] for b in blocks], list(range(18)), "Los ids deben ir de 0 a 17 en orden")
        self.assertEqual(
            [b["name"] for b in blocks],
            EXPECTED_BLOCK_NAMES,
            "Los nombres de los 18 bloques no coinciden con la Convención A",
        )

        required_fields = {
            "id", "name", "hardness", "solid", "transmits_noise",
            "conducts_electricity", "team_owned", "ore_cost",
        }
        class_keys = ("prospector", "miner", "engineer", "sapper")
        blocks_by_id: dict[int, dict[str, Any]] = {}

        for block in blocks:
            b_id = block["id"]
            self.assertEqual(set(block.keys()), required_fields, f"Campos incorrectos en el bloque id={b_id}")
            self.assertTrue(_is_pure_int(block["id"]), f"id debe ser int en bloque {b_id}")
            self.assertIsInstance(block["name"], str, f"name debe ser str en bloque {b_id}")
            self.assertIsInstance(block["hardness"], dict, f"hardness debe ser dict en bloque {b_id}")
            self.assertEqual(tuple(block["hardness"].keys()), class_keys, f"hardness incompleto en bloque {b_id}")
            for ck in class_keys:
                h_val = block["hardness"][ck]
                self.assertTrue(_is_pure_int(h_val) and h_val >= 0, f"Dureza inválida para {ck} en bloque {b_id}")
            for bool_field in ("solid", "transmits_noise", "conducts_electricity", "team_owned"):
                self.assertIsInstance(block[bool_field], bool, f"{bool_field} debe ser bool en bloque {b_id}")
            self.assertTrue(
                _is_pure_int(block["ore_cost"]) and block["ore_cost"] >= 0,
                f"ore_cost debe ser un entero no negativo en bloque {b_id}",
            )
            blocks_by_id[b_id] = block

        # Coherencia cruzada con classes.ron
        classes = self.data["classes.ron"]["classes"]
        expected_mineable_ids = [b["id"] for b in blocks if b["hardness"]["prospector"] > 0]
        self.assertEqual(
            expected_mineable_ids,
            [1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
            "Todos los bloques salvo air (0) y lava (7) deben ser picables",
        )

        for cls_def in classes:
            cls_id = cls_def["id"]
            cls_key = cls_id.lower()
            for buildable in cls_def["buildable_blocks"]:
                b_id = buildable["block_id"]
                self.assertIn(b_id, blocks_by_id, f"Bloque construible {b_id} no existe")
                blk = blocks_by_id[b_id]
                self.assertEqual(buildable["block_name"], blk["name"], f"Nombre incoherente en {cls_id} id={b_id}")
                self.assertEqual(buildable["ore_cost"], blk["ore_cost"], f"ore_cost incoherente en {cls_id} id={b_id}")
                self.assertTrue(blk["team_owned"], f"El bloque construible {blk['name']} debe tener team_owned=true")

            mining_entries = cls_def["mining_speeds"]
            self.assertEqual(
                [m["block_id"] for m in mining_entries],
                expected_mineable_ids,
                f"Lista de bloques picables en {cls_id} no coincide con blocks.ron",
            )
            for entry in mining_entries:
                blk = blocks_by_id[entry["block_id"]]
                self.assertEqual(entry["block_name"], blk["name"], f"block_name incoherente en {cls_id}")
                self.assertEqual(
                    entry["ticks"],
                    blk["hardness"][cls_key],
                    f"Ticks de picado en {cls_id} para {blk['name']} no coinciden con hardness en blocks.ron",
                )

    def test_04_seismograph_convention_c(self) -> None:
        """Verifica la Convención C en seismograph.ron (escala 0..100_000 y tiempos en ticks)."""
        seismo = self.data["seismograph.ron"]
        self.assertEqual(seismo["min_value"], 0, "min_value debe ser 0")
        self.assertEqual(seismo["max_value"], 100_000, "max_value debe ser 100_000")
        self.assertEqual(seismo["tick_rate"], 30, "tick_rate debe ser 30 Hz")

        pg = seismo["passive_growth"]
        self.assertEqual(pg["total_value"], 100_000, "passive_growth.total_value debe ser 100_000")
        self.assertEqual(pg["total_ticks"], 28_800, "passive_growth.total_ticks debe ser 28_800 (16 min)")

        thresholds = [(t["id"], t["value"]) for t in seismo["thresholds"]]
        self.assertEqual(
            thresholds,
            [("Stir", 25_000), ("Breathe", 50_000), ("Restless", 75_000), ("Wake", 100_000)],
            "Los umbrales deben ser Stir(25_000), Breathe(50_000), Restless(75_000) y Wake(100_000)",
        )

        noise = seismo["noise_sources"]
        self.assertEqual(noise["explosion"], 3_000, "explosion debe sumar 3_000")
        self.assertEqual(noise["electric_discharge"], 200, "electric_discharge debe sumar 200")
        self.assertEqual(noise["lava_flow"], 200, "lava_flow debe sumar 200")
        self.assertEqual(noise["bucket_line_per_tick"], 1, "bucket_line_per_tick debe sumar 1 por tick")

        wake = seismo["wake"]
        self.assertEqual(wake["duration_ticks"], 2_700, "wake.duration_ticks debe ser 2_700 (90 s)")
        self.assertEqual(wake["bucket_stop_remaining_ticks"], 900, "bucket_stop_remaining_ticks debe ser 900 (30 s)")

        relief = seismo["provisional_relief"]
        for key in (
            "offering_gold_cost", "offering_reduction", "offering_cooldown_ticks",
            "brass_tally_reduction", "max_tallies_per_shift",
        ):
            val = relief[key]
            self.assertTrue(_is_pure_int(val) and val > 0, f"provisional_relief.{key} debe ser entero positivo")

    def test_05_passive_growth_exactness(self) -> None:
        """Comprueba que el crecimiento pasivo alcance 100_000 exactamente en el tick 28_800."""
        passive = seismo_sim.simulate_passive_only(self.data["seismograph.ron"])
        self.assertEqual(passive["meter_at_28799"], 99_996, "En el tick 28_799 el medidor debe valer 99_996")
        self.assertLess(passive["meter_at_28799"], 100_000, "El medidor no puede llegar a 100_000 antes de 28_800")
        self.assertEqual(passive["meter_at_28800"], 100_000, "En el tick 28_800 el medidor debe valer 100_000")
        self.assertEqual(
            passive["threshold_ticks"],
            {"Stir": 7_200, "Breathe": 14_400, "Restless": 21_600, "Wake": 28_800},
            "Los umbrales pasivos deben alcanzarse exactamente en 7_200, 14_400, 21_600 y 28_800 ticks",
        )

    def test_06_classes_and_economy_rules(self) -> None:
        """Verifica las 4 clases, la capacidad doble del Minero y las reglas de economy.ron."""
        classes_list = self.data["classes.ron"]["classes"]
        by_id = {c["id"]: c for c in classes_list}
        self.assertEqual(
            list(by_id.keys()),
            ["Prospector", "Miner", "Engineer", "Sapper"],
            "Deben existir exactamente las 4 clases: Prospector, Miner, Engineer y Sapper",
        )

        miner = by_id["Miner"]
        for other_id in ("Prospector", "Sapper"):
            other = by_id[other_id]
            self.assertEqual(
                miner["max_ore_capacity"], 2 * other["max_ore_capacity"],
                f"El Minero debe cargar el doble de mena que {other_id}",
            )
            self.assertEqual(
                miner["max_bullion_capacity"], 2 * other["max_bullion_capacity"],
                f"El Minero debe cargar el doble de oro/diamante que {other_id}",
            )

        econ = self.data["economy.ron"]
        self.assertEqual(econ["respawn_ticks"], 180, "respawn_ticks debe ser 180 (6 s a 30 Hz)")
        self.assertEqual((econ["fame_threshold_num"], econ["fame_threshold_den"]), (60, 100), "Fama debe ser 60 / 100")
        self.assertEqual(
            (econ["stolen_loot_multiplier_num"], econ["stolen_loot_multiplier_den"]),
            (5, 4),
            "El multiplicador de botín robado debe ser 5 / 4",
        )

        team_goals = econ["team_goals"]
        self.assertEqual([g["team_size"] for g in team_goals], list(range(1, 9)), "Metas requeridas para 1v1 a 8v8")
        for goal in team_goals:
            self.assertTrue(_is_pure_int(goal["target_cash"]) and goal["target_cash"] > 0, "target_cash inválido")

    def test_07_match_pacing_invariants_and_report(self) -> None:
        """Verifica las medianas de ritmo, techo duro de 31_500 ticks, invariantes e informe."""
        mixto = self.sim_results["mixto"]
        self.assertGreaterEqual(mixto["median_ticks"], 16_200, "La mediana mixta es menor a 9 min (16_200 ticks)")
        self.assertLessEqual(mixto["median_ticks"], 25_200, "La mediana mixta supera 14 min (25_200 ticks)")
        self.assertGreaterEqual(mixto["median_minutes"], 9.0, "Mediana mixta < 9.0 min")
        self.assertLessEqual(mixto["median_minutes"], 14.0, "Mediana mixta > 14.0 min")

        expected_order = list(seismo_sim.THRESHOLD_ORDER)
        for profile in ("ruidoso", "silencioso", "ofrendador", "mixto"):
            self.assertIn(profile, self.sim_results, f"Falta el perfil {profile} en resultados")
            prof_res = self.sim_results[profile]
            self.assertEqual(len(prof_res["runs"]), 50, f"El perfil {profile} debe simular 50 semillas")
            self.assertTrue(prof_res["all_invariants_ok"], f"Fallaron las invariantes en el perfil {profile}")
            self.assertLessEqual(prof_res["max_ticks"], 31_500, f"El perfil {profile} superó 31_500 ticks")
            for run in prof_res["runs"]:
                self.assertLessEqual(run["duration_ticks"], 31_500, f"Semilla {run['seed']} superó 31_500 ticks")
                self.assertGreaterEqual(run["min_meter_observed"], 0, "Medidor por debajo de 0")
                self.assertLessEqual(run["max_meter_observed"], 100_000, "Medidor por encima de 100_000")
                emitted = run["emitted_order"]
                self.assertEqual(len(emitted), len(set(emitted)), "Umbral emitido más de una vez")
                self.assertEqual(emitted, expected_order[: len(emitted)], f"Orden inválido: {emitted}")

        self.assertTrue(REPORT_PATH.is_file(), f"No existe el informe en {REPORT_PATH}")
        expected_md = seismo_sim.generate_markdown_report(self.sim_results)
        actual_md = REPORT_PATH.read_text(encoding="utf-8")
        self.assertEqual(
            actual_md,
            expected_md,
            "El contenido de docs/design/ritmo-partida.md no coincide con generate_markdown_report()",
        )


if __name__ == "__main__":
    unittest.main()
