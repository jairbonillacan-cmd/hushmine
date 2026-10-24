#!/usr/bin/env python3
"""Suite de pruebas unitarias e integrales para la localización Fluent (i18n) de HUSHMINE."""

from __future__ import annotations

from pathlib import Path
import sys
import unittest

TOOLS_DIR = Path(__file__).resolve().parent
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

import check_ftl  # noqa: E402


def _make_valid_synthetic_catalogs() -> dict[str, dict[str, dict[str, str]]]:
    """Construye un catálogo sintético mínimo válido para los 3 idiomas."""
    en_game = {
        "game-title": "HUSHMINE",
        "store-premise-rule": "Don't wake the Sleeper.",
        "tool-geophone": "Geophone",
        "seismo-title": "Seismograph",
        "seismo-stir-desc": "Heralded by the Whistlers.",
        "seismo-breathe-desc": "A Mother Lode cracks open.",
    }
    es419_game = {
        "game-title": "HUSHMINE",
        "store-premise-rule": "No despierten al Durmiente.",
        "tool-geophone": "Geófono",
        "seismo-title": "Sismógrafo",
        "seismo-stir-desc": "Anunciado por los Silbadores.",
        "seismo-breathe-desc": "Se abre una Veta Madre.",
    }
    eses_game = {
        "game-title": "HUSHMINE",
        "store-premise-rule": "No despertéis al Durmiente.",
        "tool-geophone": "Geófono",
        "seismo-title": "Sismógrafo",
        "seismo-stir-desc": "Anunciado por los Silbadores.",
        "seismo-breathe-desc": "Se abre una Veta Madre.",
    }
    for idx, block_id in enumerate(check_ftl.EXPECTED_BLOCKS):
        en_game[block_id] = f"Block {idx}"
        es419_game[block_id] = (
            "Riel" if block_id == "block-rail" else f"Bloque {idx}"
        )
        eses_game[block_id] = (
            "Raíl" if block_id == "block-rail" else f"Bloque {idx}"
        )

    en_hud = {
        "hud-offering": "Paid ${ $gold }.",
        "hud-ore": "{ $count -> [one] { $count } unit *[other] { $count } units }",
    }
    es419_hud = {
        "hud-offering": "Pagaron ${ $gold }.",
        "hud-ore": "{ $count -> [one] { $count } unidad *[other] { $count } unidades }",
    }
    eses_hud = {
        "hud-offering": "Habéis pagado ${ $gold }.",
        "hud-ore": "{ $count -> [one] { $count } unidad *[other] { $count } unidades }",
    }

    en_watcher: dict[str, str] = {}
    es419_watcher: dict[str, str] = {}
    eses_watcher: dict[str, str] = {}
    for idx in range(1, 16):
        key = f"watcher-test-{idx}"
        en_watcher[key] = "Short radio alert."
        es419_watcher[key] = f"Salgan ahora mismo ({idx})."
        eses_watcher[key] = f"Salid ahora mismo ({idx})."

    en_signs = {"sign-01-title": "SIGN-01"}
    es419_signs = {"sign-01-title": "CARTEL-01"}
    eses_signs = {"sign-01-title": "CARTEL-01"}

    en_chron = {
        "chronicle-title-awakener": "Awakener",
        "chronicle-title-slumber-guardian": "Slumber Guardian",
        "chronicle-title-mole": "Mole",
        "chronicle-title-bucket-thief": "Bucket Thief",
    }
    es_chron = dict(check_ftl.CHRONICLE_TITLES)

    return {
        "en": {
            "game.ftl": en_game,
            "hud.ftl": en_hud,
            "watcher.ftl": en_watcher,
            "signs.ftl": en_signs,
            "chronicle.ftl": en_chron,
        },
        "es-419": {
            "game.ftl": es419_game,
            "hud.ftl": es419_hud,
            "watcher.ftl": es419_watcher,
            "signs.ftl": es419_signs,
            "chronicle.ftl": dict(es_chron),
        },
        "es-ES": {
            "game.ftl": eses_game,
            "hud.ftl": eses_hud,
            "watcher.ftl": eses_watcher,
            "signs.ftl": eses_signs,
            "chronicle.ftl": dict(es_chron),
        },
    }


class TestFluentParserAndValidators(unittest.TestCase):
    """Pruebas unitarias con casos sintéticos positivos y negativos."""

    def test_parse_valid_single_and_multiline_entries(self) -> None:
        ftl = (
            "# Comentario inicial\n"
            "game-title = HUSHMINE\n"
            "hud-ore-carried =\n"
            "    { $count ->\n"
            "        [one] { $count } unidad\n"
            "       *[other] { $count } unidades\n"
            "    }\n"
        )
        msgs, errors = check_ftl.parse_ftl_text(ftl)
        self.assertEqual(errors, [])
        self.assertEqual(msgs["game-title"], "HUSHMINE")
        self.assertIn("*[other]", msgs["hud-ore-carried"])

    def test_parse_detects_duplicate_id_in_same_file(self) -> None:
        ftl = "game-title = HUSHMINE\ngame-title = Otro título\n"
        _, errors = check_ftl.parse_ftl_text(ftl)
        self.assertTrue(any("ID duplicado" in err for err in errors), errors)

    def test_validate_id_rejects_snake_case_and_wrong_prefix(self) -> None:
        msgs = {"game_title": "HUSHMINE", "watcher-stir-1": "She's stirring..."}
        errors = check_ftl.validate_id_naming(msgs, filename="game.ftl")
        self.assertTrue(any("game_title" in err for err in errors), errors)
        self.assertTrue(any("watcher-stir-1" in err for err in errors), errors)

    def test_selector_requires_default_other_variant(self) -> None:
        no_default = "{ $count -> [one] 1 mineral [other] { $count } minerales }"
        errs_no_def = check_ftl.validate_message_syntax("hud-ore", no_default)
        self.assertTrue(any("*[other]" in err for err in errs_no_def), errs_no_def)

        wrong_default = "{ $count -> *[one] 1 mineral }"
        errs_wrong_def = check_ftl.validate_message_syntax("hud-ore", wrong_default)
        self.assertTrue(
            any("*[other]" in err for err in errs_wrong_def), errs_wrong_def
        )

    def test_selector_detects_unbalanced_braces(self) -> None:
        broken = "{ $count -> [one] { $count } mineral *[other] { $count } minerales"
        errors = check_ftl.validate_message_syntax("hud-ore", broken)
        self.assertTrue(any("desbalanceadas" in err for err in errors), errors)

    def test_catalog_detects_missing_ids_and_cross_file_duplicates(self) -> None:
        catalogs = _make_valid_synthetic_catalogs()
        del catalogs["es-419"]["hud.ftl"]["hud-ore"]
        catalogs["es-ES"]["signs.ftl"]["game-title"] = "Duplicado entre archivos"
        errors = check_ftl.check_catalogs_symmetry_and_vars(catalogs)
        self.assertTrue(any("hud-ore" in err for err in errors), errors)
        self.assertTrue(
            any("duplicado entre" in err for err in errors), errors
        )

    def test_catalog_detects_omitted_variable_in_translation(self) -> None:
        catalogs = _make_valid_synthetic_catalogs()
        catalogs["es-419"]["hud.ftl"]["hud-offering"] = "Pagaron oro sin variable."
        errors = check_ftl.check_catalogs_symmetry_and_vars(catalogs)
        self.assertTrue(any("hud-offering" in err for err in errors), errors)

    def test_watcher_rejects_seven_word_english_line(self) -> None:
        en_watcher = {
            "watcher-stir-1": "One two three four five six seven."
        }
        errors = check_ftl.check_watcher_constraints(en_watcher)
        self.assertTrue(any("7 palabras" in err for err in errors), errors)

    def test_spanish_glossary_rejects_forbidden_terms(self) -> None:
        for forbidden in (
            "el Dormido",
            "El Que Duerme",
            "Veta Principal",
            "Filón Madre",
            "Línea de cubos",
            "Línea de baldes",
            "Chapa de bronce",
            "Cobre Crew",
            "Cobalto Crew",
        ):
            catalogs = _make_valid_synthetic_catalogs()
            catalogs["es-419"]["signs.ftl"]["sign-01-title"] = (
                f"Aviso sobre {forbidden} en la galería."
            )
            errors = check_ftl.check_spanish_glossary_and_variants(catalogs)
            self.assertTrue(
                any("término prohibido" in err for err in errors),
                f"No falló con el término prohibido {forbidden!r}: {errors}",
            )

    def test_spanish_glossary_detects_missing_canonical_term_and_regional_diffs(
        self,
    ) -> None:
        catalogs = _make_valid_synthetic_catalogs()
        catalogs["es-ES"]["game.ftl"]["store-premise-rule"] = (
            "No despertéis al gigante."
        )
        for key in catalogs["es-ES"]["watcher.ftl"]:
            catalogs["es-ES"]["watcher.ftl"][key] = catalogs["es-419"][
                "watcher.ftl"
            ][key]
        errors = check_ftl.check_spanish_glossary_and_variants(catalogs)
        self.assertTrue(any("durmiente" in err for err in errors), errors)
        self.assertTrue(
            any("Variantes regionales insuficientes" in err for err in errors),
            errors,
        )


class TestRepositoryI18nIntegrity(unittest.TestCase):
    """Ejecuta la verificación completa sobre los archivos reales del repositorio."""

    def test_repository_check_all_passes_without_errors(self) -> None:
        errors = check_ftl.check_all(check_ftl.REPO_ROOT)
        self.assertEqual(
            errors,
            [],
            "Falló la verificación i18n del repositorio:\n" + "\n".join(errors),
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
