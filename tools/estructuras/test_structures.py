"""Pruebas de los datos de estructuras, del ensamblador y del plano PNG.

Ejecucion desde la raiz del clone:
`python -m unittest discover -s tools/estructuras -v`

Las rutas se resuelven desde __file__, nunca desde el directorio actual.
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
TOOLS_ROOT = HERE.parent
REPO_ROOT = TOOLS_ROOT.parent
for _ruta in (TOOLS_ROOT / "balance", TOOLS_ROOT / "textures"):
    if str(_ruta) not in sys.path:
        sys.path.insert(0, str(_ruta))
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import check_structures as cs  # noqa: E402
import png  # noqa: E402
import render_blueprints as rb  # noqa: E402

# Recuento esperado de piezas por familia en assets/estructuras/.
PIECES_PER_FAMILY = {
    "ancient_mine": 9,
    "arrival_village": 4,
    "prospect_cut": 1,
    "still_hamlet": 3,
    "watchtower": 1,
}


class EstructurasTest(unittest.TestCase):
    def setUp(self) -> None:
        self.familias = cs.load_familias()
        self.pieces = cs.load_pieces()

    def test_familias_catalog_loads_with_six_families(self) -> None:
        self.assertEqual(len(self.familias), 6)
        self.assertEqual(set(self.familias), set(cs.REQUIRED_FAMILY_IDS))

    def test_familias_fields_are_valid(self) -> None:
        self.assertEqual(cs.validate_familias(self.familias), [])

    def test_all_pieces_parse_and_match_filename(self) -> None:
        self.assertEqual(len(self.pieces), 18)
        por_familia: dict = {}
        for pid, pieza in self.pieces.items():
            self.assertEqual(pieza["id"], pid)  # el cargador ya exige id == stem
            por_familia[pieza["family"]] = por_familia.get(pieza["family"], 0) + 1
        self.assertEqual(por_familia, PIECES_PER_FAMILY)

    def test_fills_stay_within_declared_bounds(self) -> None:
        self.assertEqual(cs.validate_pieces(self.pieces, self.familias), [])
        cap = self.pieces["collapse_cap"]
        self.assertEqual(cap["size"], [3, 4, 3])
        for fill in cap["fills"]:
            x0, y0, z0, x1, y1, z1, block = fill
            self.assertTrue(0 <= x0 <= x1 <= 3)
            self.assertTrue(0 <= y0 <= y1 <= 4)
            self.assertTrue(0 <= z0 <= z1 <= 3)
            self.assertTrue(0 <= block <= cs.BLOCK_ID_MAX)

    def test_connectors_rest_on_exterior_faces(self) -> None:
        self.assertEqual(cs.validate_pieces(self.pieces, self.familias), [])
        shaft = self.pieces["main_shaft"]
        subidas = [c for c in shaft["connectors"] if c["dir"] == "up"]
        self.assertEqual(len(subidas), 1)
        self.assertEqual(subidas[0]["pos"][1], shaft["size"][1])

    def test_block_ids_within_convention_a(self) -> None:
        for pieza in self.pieces.values():
            for fill in pieza["fills"]:
                self.assertTrue(0 <= fill[6] <= cs.BLOCK_ID_MAX,
                                f"{pieza['id']}: block_id {fill[6]} fuera de rango")

    def test_every_kind_has_normal_and_cap_pieces(self) -> None:
        for kind in cs.KINDS:
            normales = [p["id"] for p in self.pieces.values()
                        if not p.get("is_cap") and kind in cs._kinds_of(p)]
            cierres = [p["id"] for p in self.pieces.values()
                       if p.get("is_cap") and kind in cs._kinds_of(p)]
            self.assertTrue(normales, f"kind {kind} sin pieza normal")
            self.assertTrue(cierres, f"kind {kind} sin pieza de cierre")

    def test_every_open_connector_has_compatible_cap(self) -> None:
        self.assertEqual(cs.validate_cap_coverage(self.pieces), [])

    def test_root_pieces_exist_for_all_families(self) -> None:
        self.assertEqual(cs.validate_root_pieces(self.familias, self.pieces), [])
        for familia in self.familias.values():
            self.assertIn(familia["root_piece"], self.pieces)

    def test_assembly_plan_of_twelve_pieces_has_no_overlap(self) -> None:
        plan = cs.assemble_plan(42, "ancient_mine", 12)
        self.assertEqual(len(plan["pieces"]), 12)
        self.assertEqual(plan["stats"]["placed"], 12)
        self.assertEqual(cs.check_plan_overlap(plan, self.pieces), [])
        radio = plan["radius"]
        for entry in plan["pieces"]:
            rs = cs.rotate_size(self.pieces[entry["id"]]["size"], entry["rotation"])
            ox, oy, oz = entry["origin"]
            self.assertGreaterEqual(ox, -radio)
            self.assertLessEqual(ox + rs[0], radio)
            self.assertGreaterEqual(oz, -radio)
            self.assertLessEqual(oz + rs[2], radio)

    def test_assembly_is_deterministic(self) -> None:
        plan_a = cs.assemble_plan(42, "ancient_mine", 12)
        plan_b = cs.assemble_plan(42, "ancient_mine", 12)
        claves = ("id", "rotation", "origin")
        piezas_a = [{k: entry[k] for k in claves} for entry in plan_a["pieces"]]
        piezas_b = [{k: entry[k] for k in claves} for entry in plan_b["pieces"]]
        self.assertEqual(piezas_a, piezas_b)
        self.assertEqual(plan_a["sealed"], plan_b["sealed"])

    def test_blueprint_png_is_written_and_decodable(self) -> None:
        ruta = rb.render_plan_png()
        self.assertEqual(ruta, REPO_ROOT / "out" / "estructuras" / "ancient_mine_plan.png")
        self.assertTrue(ruta.exists())
        crudos = ruta.read_bytes()
        self.assertTrue(crudos.startswith(png.PNG_SIGNATURE))
        ancho, alto, datos = png.read_png(ruta)
        self.assertGreater(ancho, 0)
        self.assertGreater(alto, 0)
        self.assertEqual(len(datos), ancho * alto * 4)


if __name__ == "__main__":
    unittest.main()
