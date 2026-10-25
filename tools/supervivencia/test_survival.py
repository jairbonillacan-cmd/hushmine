"""Pruebas del verificador de datos de supervivencia de HUSHMINE.

Ejecución desde la raíz del repositorio:
    python -m unittest discover -s tools/supervivencia -v

Las pruebas sobre `data/supervivencia/` se saltan con "faltan datos" si los
archivos RON aún no existen; las dos pruebas autocontenidas (con RON de
muestra en línea) pasan siempre.
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path
from typing import Any

_THIS_DIR = Path(__file__).resolve().parent
if str(_THIS_DIR) not in sys.path:
    sys.path.insert(0, str(_THIS_DIR))

import check_survival  # noqa: E402
import ron_lite  # noqa: E402

ROOT = check_survival.ROOT
DATA_DIR = check_survival.DATA_DIR
FILES = check_survival.FILES

# Clave raíz de cada archivo RON, para el test de parseo general.
ROOT_KEYS: dict[str, str] = {
    "objetos.ron": "items",
    "tecnologias.ron": "techs",
    "recetas.ron": "recipes",
    "criaturas.ron": "creatures",
    "jefes.ron": "bosses",
    "dimensiones.ron": "dimensions",
    "comercio.ron": "traders",
    "encargos.ron": "orders",
}

# RON de muestra en línea: dos items con valores de oro conocidos.
ITEMS_SNIPPET = """\
(
    items: [
        ( id: "scrap", name: "Scrap", category: "material", tier: 1,
          max_stack: 50, durability: 0, weight: 1, base_value_gold: 10 ),
        ( id: "lamp", name: "Lamp", category: "lamp", tier: 2,
          max_stack: 1, durability: 120, weight: 2, base_value_gold: 30 ),
    ],
)
"""

# Receta equilibrada: consume 4 x 10 = 40 de valor y produce 1 x 30 = 30.
BALANCED_RECIPE_SNIPPET = """\
(
    recipes: [
        ( id: "r_ok", name: "Assembly", tech_required: "t0", station: "workbench",
          inputs: [ ( item_id: "scrap", qty: 4 ) ],
          outputs: [ ( item_id: "lamp", qty: 1 ) ],
          craft_ticks: 10, noise_emitted: 5 ),
    ],
)
"""

# Receta con ganancia: consume 4 x 10 = 40 de valor y produce 2 x 30 = 60.
PROFIT_RECIPE_SNIPPET = """\
(
    recipes: [
        ( id: "r_bad", name: "Free Gold", tech_required: "t0", station: "workbench",
          inputs: [ ( item_id: "scrap", qty: 4 ) ],
          outputs: [ ( item_id: "lamp", qty: 2 ) ],
          craft_ticks: 10, noise_emitted: 5 ),
    ],
)
"""


def _snippet_docs(recipe_ron: str) -> dict[str, Any]:
    """Construye un `docs` mínimo (objetos + recetas) desde los snippets."""
    return {
        "objetos.ron": ron_lite.loads(ITEMS_SNIPPET),
        "recetas.ron": ron_lite.loads(recipe_ron),
    }


class HelperSnippetsTest(unittest.TestCase):
    """Pruebas autocontenidas de las funciones auxiliares, sin datos en disco."""

    def test_recipe_profit_invariant_on_snippets(self) -> None:
        """La receta con ganancia se detecta y la equilibrada pasa (RON en línea)."""
        errors_ok: list[str] = []
        check_survival.check_no_profit(_snippet_docs(BALANCED_RECIPE_SNIPPET), errors_ok)
        self.assertEqual(errors_ok, [], "La receta equilibrada no debe marcar fallos")

        errors_bad: list[str] = []
        check_survival.check_no_profit(_snippet_docs(PROFIT_RECIPE_SNIPPET), errors_bad)
        self.assertEqual(len(errors_bad), 1, "Debe detectarse exactamente un fallo de ganancia")
        self.assertTrue(errors_bad[0].startswith("[FALLO]"), "El fallo debe llevar el prefijo [FALLO]")
        self.assertIn("r_bad", errors_bad[0], "El fallo debe citar la receta infractora")

    def test_float_rejection_and_schema_on_snippets(self) -> None:
        """Un RON con flotante se rechaza y un item fuera de rango se detecta."""
        self.assertTrue(ron_lite.has_decimal_numbers("( v: 1.5 )"), "Debe detectar el decimal 1.5")
        with self.assertRaises(ValueError, msg="ron_lite.loads debe rechazar los flotantes"):
            ron_lite.loads("( v: 1.5 )")
        self.assertEqual(ron_lite.loads("( v: 65_536 )")["v"], 65_536, "Los enteros con _ deben parsear")

        bad_items = ron_lite.loads(
            '( items: [ ( id: "x", name: "X", category: "mineral", tier: 9,'
            " max_stack: 0, durability: 0, weight: 0, base_value_gold: 0 ) ] )"
        )
        errors: list[str] = []
        check_survival.check_items(bad_items, errors)
        self.assertTrue(any("tier" in error for error in errors), "Debe detectar tier=9 fuera de 1..=5")
        self.assertTrue(any("category" in error for error in errors), "Debe detectar la categoría desconocida")
        self.assertTrue(any("max_stack" in error for error in errors), "Debe detectar max_stack=0")

    def test_rows_that_are_not_structs_are_reported_not_crashed(self) -> None:
        """Una fila que no es struct (p. ej. `items: [1]`) da fallos, no una excepción."""
        docs = {"objetos.ron": ron_lite.loads("( items: [ 1 ] )"),
                "recetas.ron": ron_lite.loads("( recipes: [ 2 ] )")}
        errors: list[str] = []
        check_survival.check_items(docs["objetos.ron"], errors)
        check_survival.check_cross_refs(docs, errors)
        check_survival.check_no_profit(docs, errors)
        self.assertTrue(errors, "La fila no-struct debe notificarse como fallo")


class _SurvivalDataCase(unittest.TestCase):
    """Base común: carga los datos una vez o salta las pruebas si faltan."""

    texts: dict[str, str]
    docs: dict[str, Any]

    @classmethod
    def setUpClass(cls) -> None:
        """Carga los 8 documentos, o salta la clase si aún no existen en disco."""
        missing = [name for name in FILES if not (DATA_DIR / name).is_file()]
        if missing:
            raise unittest.SkipTest("faltan datos")
        errors: list[str] = []
        cls.texts = {name: (DATA_DIR / name).read_text(encoding="utf-8") for name in FILES}
        cls.docs = check_survival.load_all(ROOT, errors)
        cls.hygiene_errors: list[str] = []
        for name, text in cls.texts.items():
            check_survival.check_file_hygiene(name, text, cls.hygiene_errors)


class DataSchemaTest(_SurvivalDataCase):
    """Esquema, higiene y parseo de los 8 archivos de datos en disco."""

    def test_all_files_parse(self) -> None:
        """Los 8 archivos existen, parsean como struct y traen su lista raíz."""
        for name, root_key in ROOT_KEYS.items():
            self.assertIn(name, self.docs, f"Falta el documento {name}")
            doc = self.docs[name]
            self.assertIsInstance(doc, dict, f"La raíz de {name} debe ser un struct")
            self.assertIn(root_key, doc, f"{name} debe traer la lista raíz '{root_key}'")
            self.assertIsInstance(doc[root_key], list, f"'{root_key}' de {name} debe ser lista")

    def test_no_floats(self) -> None:
        """Ningún archivo contiene números con punto decimal."""
        for name, text in self.texts.items():
            self.assertFalse(
                ron_lite.has_decimal_numbers(text),
                f"{name} contiene números con punto decimal prohibidos",
            )

    def test_line_limit_and_lf(self) -> None:
        """Solo LF, máximo 400 líneas y salto de línea final en cada archivo."""
        for name in FILES:
            self.assertEqual(
                [e for e in self.hygiene_errors if e.startswith(f"[FALLO] {name}")],
                [],
                f"{name} incumple la higiene de archivos",
            )
            text = self.texts[name]
            self.assertNotIn("\r", text, f"{name} contiene CR")
            self.assertLessEqual(len(text.splitlines()), check_survival.MAX_LINES, f"{name} supera 400 líneas")
            self.assertTrue(text.endswith("\n"), f"{name} no termina en salto de línea")

    def test_items_unique_and_min_count(self) -> None:
        """objetos.ron: al menos 36 items con ids únicos y esquema válido."""
        errors: list[str] = []
        check_survival.check_items(self.docs["objetos.ron"], errors)
        self.assertEqual(errors, [], "check_items no debe marcar fallos")
        items = self.docs["objetos.ron"]["items"]
        self.assertGreaterEqual(len(items), check_survival.MIN_ITEMS, "Faltan items")
        ids = [entry["id"] for entry in items]
        self.assertEqual(len(ids), len(set(ids)), "Hay ids de item duplicados")

    def test_tech_branches_and_tiers(self) -> None:
        """tecnologias.ron: 6 ramas con cobertura completa de tiers 1..=5."""
        errors: list[str] = []
        check_survival.check_techs(self.docs["tecnologias.ron"], errors)
        self.assertEqual(errors, [], "check_techs no debe marcar fallos")
        techs = self.docs["tecnologias.ron"]["techs"]
        self.assertGreaterEqual(len(techs), check_survival.MIN_TECHS, "Faltan tecnologías")
        coverage: dict[str, set[int]] = {}
        for tech in techs:
            coverage.setdefault(tech["branch"], set()).add(tech["tier"])
        for branch in sorted(check_survival.BRANCHES):
            self.assertTrue(
                set(range(1, 6)) <= coverage.get(branch, set()),
                f"La rama {branch} no cubre los tiers 1..=5",
            )

    def test_creatures_schema(self) -> None:
        """criaturas.ron: al menos 14 criaturas con esquema y drops válidos."""
        errors: list[str] = []
        check_survival.check_creatures(self.docs["criaturas.ron"], errors)
        self.assertEqual(errors, [], "check_creatures no debe marcar fallos")
        creatures = self.docs["criaturas.ron"]["creatures"]
        self.assertGreaterEqual(len(creatures), check_survival.MIN_CREATURES, "Faltan criaturas")
        for creature in creatures:
            self.assertIn(
                creature["light_reaction"],
                check_survival.LIGHT_REACTIONS,
                f"{creature['id']} con light_reaction inválida",
            )
            for drop in creature["drops"]:
                self.assertLessEqual(drop["min_qty"], drop["max_qty"], f"{creature['id']} con drops incoherentes")
        waker = next(c for c in creatures if c["id"] == "waker")
        self.assertEqual(waker["habitat"], "anywhere", "Los Desvelos aparecen en cualquier sitio")

    def test_bosses_phases_and_weaknesses(self) -> None:
        """jefes.ron: al menos 5 jefes, cada uno con >= 2 fases y debilidades 0..=100."""
        errors: list[str] = []
        check_survival.check_bosses(self.docs["jefes.ron"], errors)
        self.assertEqual(errors, [], "check_bosses no debe marcar fallos")
        bosses = self.docs["jefes.ron"]["bosses"]
        self.assertGreaterEqual(len(bosses), check_survival.MIN_BOSSES, "Faltan jefes")
        for boss in bosses:
            self.assertGreaterEqual(len(boss["phases"]), check_survival.MIN_BOSS_PHASES, f"{boss['id']} sin fases")
            self.assertTrue(0 <= boss["acoustic_weakness"] <= 100, f"{boss['id']} fuera de rango acústico")
            self.assertTrue(0 <= boss["luminous_weakness"] <= 100, f"{boss['id']} fuera de rango luminoso")

    def test_dimensions_complete(self) -> None:
        """dimensiones.ron: exactamente las 5 dimensiones con índices y luz únicos."""
        errors: list[str] = []
        check_survival.check_dimensions(self.docs["dimensiones.ron"], errors)
        self.assertEqual(errors, [], "check_dimensions no debe marcar fallos")
        dims = {dim["id"]: dim for dim in self.docs["dimensiones.ron"]["dimensions"]}
        self.assertEqual(set(dims), set(check_survival.EXPECTED_DIMENSIONS), "Los ids de dimensión no coinciden")
        for dim_id, index in check_survival.EXPECTED_DIMENSIONS.items():
            self.assertEqual(dims[dim_id]["index"], index, f"{dim_id} con índice incorrecto")
            self.assertNotEqual(dims[dim_id]["gravity_q16"], 0, f"{dim_id} con gravedad nula")

    def test_orders_schema(self) -> None:
        """encargos.ron: al menos 12 encargos con tipo, facción y recompensa válidas."""
        errors: list[str] = []
        check_survival.check_orders(self.docs["encargos.ron"], errors)
        self.assertEqual(errors, [], "check_orders no debe marcar fallos")
        orders = self.docs["encargos.ron"]["orders"]
        self.assertGreaterEqual(len(orders), check_survival.MIN_ORDERS, "Faltan encargos")
        for order in orders:
            self.assertIn(order["kind"], check_survival.ORDER_KINDS, f"{order['id']} con kind inválido")
            self.assertGreaterEqual(order["target_qty"], 1, f"{order['id']} con target_qty < 1")


class DataInvariantsTest(_SurvivalDataCase):
    """Invariantes cruzadas y económicas entre los catálogos cargados."""

    def test_recipe_item_and_tech_refs(self) -> None:
        """recetas.ron: todo item de inputs/outputs y todo tech_required existen."""
        errors: list[str] = []
        check_survival.check_recipes(self.docs["recetas.ron"], errors)
        self.assertEqual(errors, [], "check_recipes no debe marcar fallos")
        item_ids = {entry["id"] for entry in self.docs["objetos.ron"]["items"]}
        tech_ids = {entry["id"] for entry in self.docs["tecnologias.ron"]["techs"]}
        for recipe in self.docs["recetas.ron"]["recipes"]:
            for side in ("inputs", "outputs"):
                for row in recipe[side]:
                    self.assertIn(row["item_id"], item_ids, f"{recipe['id']} ({side})")
            self.assertIn(recipe["tech_required"], tech_ids, f"{recipe['id']} (tech_required)")

    def test_recipes_no_profit_cycle(self) -> None:
        """Ninguna receta produce más valor en oro del que consume."""
        errors: list[str] = []
        check_survival.check_no_profit(self.docs, errors)
        self.assertEqual(errors, [], "Hay recetas con ganancia de valor prohibida")

    def test_trade_no_arbitrage(self) -> None:
        """comercio.ron: en toda oferta buy_price_gold > sell_price_gold >= 1."""
        errors: list[str] = []
        check_survival.check_commerce(self.docs["comercio.ron"], errors)
        self.assertEqual(errors, [], "check_commerce no debe marcar fallos")
        check_survival.check_no_arbitrage(self.docs, errors)
        self.assertEqual(errors, [], "Hay ofertas con arbitraje prohibido")
        mandatory = set(check_survival.MANDATORY_TRADERS)
        trader_ids = {trader["id"] for trader in self.docs["comercio.ron"]["traders"]}
        self.assertTrue(mandatory <= trader_ids, "Faltan traders obligatorios")
        self.assertGreaterEqual(len(trader_ids), check_survival.MIN_TRADERS, "Faltan traders")


if __name__ == "__main__":
    unittest.main()
