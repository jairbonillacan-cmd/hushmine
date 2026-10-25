"""Pruebas de los modelos RON, los clips y el mip 32×32 → 16×16 de HUSHMINE.

Cubre el parseo entero de los seis .ron de la entrega, el esqueleto de 14
piezas y los 3 enganches del minero, la estructura de herramientas y
criaturas, los invariantes de los clips (tercera y primera persona) y el
generador de texturas `tex32` (mip en paleta, verificación y determinismo).
Las texturas se generan en memoria: no depende de `out/`.

Ejecución, desde la raíz del repositorio:

    python -m unittest discover -s tools/models -v
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path
from typing import Any

REPO_ROOT: Path = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.balance import ron_lite  # noqa: E402
from tools.models import check_models, tex32  # noqa: E402
from tools.textures import palette  # noqa: E402

MODELS_DIR: Path = REPO_ROOT / "assets" / "models"
ANIM_DIR: Path = REPO_ROOT / "assets" / "anim"

RON_FILES: tuple[Path, ...] = (
    MODELS_DIR / "miner.ron",
    MODELS_DIR / "tools.ron",
    MODELS_DIR / "creatures.ron",
    ANIM_DIR / "miner_clips.ron",
    ANIM_DIR / "first_person_clips.ron",
    MODELS_DIR / "first_person_arms.ron",
)

# Esqueleto del minero: id de pieza -> padre (None para la raíz).
MINER_PARENTS: dict[str, str | None] = {
    "pelvis": None,
    "torso": "pelvis",
    "head": "torso",
    "upper_arm_l": "torso",
    "upper_arm_r": "torso",
    "lower_arm_l": "upper_arm_l",
    "lower_arm_r": "upper_arm_r",
    "hand_l": "lower_arm_l",
    "hand_r": "lower_arm_r",
    "upper_leg_l": "pelvis",
    "upper_leg_r": "pelvis",
    "lower_leg_l": "upper_leg_l",
    "lower_leg_r": "upper_leg_r",
    "pack": "torso",
}

# Enganches del minero: id -> pieza anclada.
MINER_ATTACHMENTS: dict[str, str] = {"lamp": "head", "tool": "hand_r", "back": "torso"}

TOOL_SLOTS: dict[str, str] = {
    "pickaxe": "tool",
    "shovel": "tool",
    "shears": "tool",
    "geophone": "tool",
    "satchel_charge": "tool",
    "helmet_lamp": "lamp",
}

CREATURE_NAMES: frozenset[str] = frozenset({"whistler", "ore_beetle", "stone_stalker"})
MINER_CLIPS: frozenset[str] = frozenset(
    {"idle", "walk", "run", "crouch", "dig", "place", "ladder", "carry", "geophone_listen"}
)
FIRST_PERSON_CLIPS: frozenset[str] = frozenset({"idle_1p", "dig_1p", "place_1p", "carry_1p"})
MARKER_KINDS: frozenset[str] = frozenset({"impact", "step_l", "step_r", "release"})


def unpack_option(value: Any) -> Any:
    """Convierte el parseo de ron_lite: Some(x) -> x y None -> None."""
    if value == "None":
        return None
    if isinstance(value, tuple) and len(value) == 1:
        return value[0]
    return value


def _parts_of(model: dict) -> list[dict]:
    """Lista de piezas de un Model o Modelo (vacía si falta el campo)."""
    return model.get("parts", [])


def _parents_of(parts: list[dict]) -> dict[str, str | None]:
    """Mapa id de pieza -> padre ya desenvolvido."""
    return {p["id"]: unpack_option(p.get("parent")) for p in parts}


def _tree_errors(parts: list[dict]) -> list[str]:
    """Padres inexistentes, raíces que no son una o ciclos en un modelo."""
    errors: list[str] = []
    by_id = {p["id"]: p for p in parts}
    roots = [i for i, p in by_id.items() if unpack_option(p.get("parent")) is None]
    if len(roots) != 1:
        errors.append(f"{len(roots)} raíces")
    for part in parts:
        current, steps = part, 0
        while True:
            parent = unpack_option(current.get("parent"))
            if parent is None:
                break
            if parent not in by_id:
                errors.append(f"{part['id']}: padre {parent!r} inexistente")
                break
            current = by_id[parent]
            steps += 1
            if steps > len(parts):
                errors.append(f"{part['id']}: ciclo en el grafo de padres")
                break
    return errors


def _key_errors(clips: list[dict]) -> list[str]:
    """Claves con t_ms fuera de [0, duration_ms] o no crecientes por pista."""
    errors: list[str] = []
    for clip in clips:
        duration = clip["duration_ms"]
        for track in clip.get("tracks", []):
            label = f"{clip['name']}/{track['part']}"
            previous = -1
            for key in track.get("keys", []):
                t = key["t_ms"]
                if t < 0 or t > duration:
                    errors.append(f"{label}: t_ms {t} fuera de rango")
                if t <= previous:
                    errors.append(f"{label}: t_ms {t} no creciente")
                previous = t
    return errors


def _marker_errors(clips: list[dict]) -> list[str]:
    """Markers con kind desconocido o t_ms fuera de (0, duration_ms)."""
    errors: list[str] = []
    for clip in clips:
        duration = clip["duration_ms"]
        for marker in clip.get("markers", []):
            if marker["kind"] not in MARKER_KINDS:
                errors.append(f"{clip['name']}: kind {marker['kind']!r} inválido")
            if not 0 < marker["t_ms"] < duration:
                errors.append(f"{clip['name']}: t_ms {marker['t_ms']} fuera de rango")
    return errors


def _impacts_of(clips: list[dict], name: str) -> list[dict]:
    """Markers impact del clip pedido."""
    clip = next(c for c in clips if c["name"] == name)
    return [m for m in clip.get("markers", []) if m["kind"] == "impact"]


MINER: dict = ron_lite.load(MODELS_DIR / "miner.ron")
TOOLS: dict = ron_lite.load(MODELS_DIR / "tools.ron")
CREATURES: dict = ron_lite.load(MODELS_DIR / "creatures.ron")
MINER_CLIPS_DATA: dict = ron_lite.load(ANIM_DIR / "miner_clips.ron")
FP_CLIPS: dict = ron_lite.load(ANIM_DIR / "first_person_clips.ron")

MINER_PARTS: list[dict] = _parts_of(MINER)
TOOL_MODELS: dict[str, dict] = {m["name"]: m for m in TOOLS["models"]}
CREATURE_MODELS: dict[str, dict] = {m["name"]: m for m in CREATURES["models"]}
CLIPS_3P: list[dict] = MINER_CLIPS_DATA["clips"]
CLIPS_1P: list[dict] = FP_CLIPS["clips"]
TEXTURES: dict[str, bytes] = tex32.generate_textures()


class TestRonParsing(unittest.TestCase):
    """Los seis archivos cargan con ron_lite y no tienen decimales."""

    def test_five_ron_files_parse_without_decimals(self) -> None:
        for path in RON_FILES:
            with self.subTest(filename=path.name):
                text = path.read_text(encoding="utf-8")
                self.assertFalse(ron_lite.has_decimal_numbers(text))
                self.assertIsNotNone(ron_lite.loads(text))


class TestMinerModel(unittest.TestCase):
    """Esqueleto de 14 piezas, padres exactos y 3 enganches."""

    def test_miner_has_exact_14_part_ids(self) -> None:
        ids = [p["id"] for p in MINER_PARTS]
        self.assertEqual(len(ids), 14)
        self.assertEqual(set(ids), set(MINER_PARENTS))

    def test_miner_parents_match_table(self) -> None:
        self.assertEqual(_parents_of(MINER_PARTS), MINER_PARENTS)

    def test_miner_has_three_expected_attachments(self) -> None:
        attachments = {a["id"]: a["part"] for a in MINER.get("attachments", [])}
        self.assertEqual(len(MINER.get("attachments", [])), 3)
        self.assertEqual(attachments, MINER_ATTACHMENTS)

    def test_miner_skeleton_acyclic_with_single_root(self) -> None:
        self.assertEqual(_tree_errors(MINER_PARTS), [])
        roots = [i for i, p in _parents_of(MINER_PARTS).items() if p is None]
        self.assertEqual(roots, ["pelvis"])
        by_id = {p["id"]: p for p in MINER_PARTS}
        for part in MINER_PARTS:
            current, steps = part, 0
            while unpack_option(current.get("parent")) is not None:
                current = by_id[unpack_option(current.get("parent"))]
                steps += 1
            self.assertLessEqual(steps, 13, f"{part['id']} no llega a la raíz en 13 pasos")


class TestToolsAndCreatures(unittest.TestCase):
    """ModelSets: nombres, slots, estructura de árbol y cuboides."""

    def test_tools_models_and_slots(self) -> None:
        self.assertEqual(set(TOOL_MODELS), set(TOOL_SLOTS))
        for name, expected_slot in TOOL_SLOTS.items():
            self.assertEqual(unpack_option(TOOL_MODELS[name].get("slot")), expected_slot,
                             f"slot de {name}")

    def test_tools_structure_and_cuboids(self) -> None:
        for name, model in TOOL_MODELS.items():
            parts = _parts_of(model)
            self.assertGreaterEqual(len(parts), 1, f"{name} sin piezas")
            self.assertEqual(_tree_errors(parts), [], f"árbol de {name}")
            for part in parts:
                lo, hi = part["cuboid"]["min"], part["cuboid"]["max"]
                self.assertTrue(all(lo[i] < hi[i] for i in range(3)),
                                f"cuboid de {name}/{part['id']}")

    def test_creatures_models_and_structure(self) -> None:
        self.assertEqual(set(CREATURE_MODELS), CREATURE_NAMES)
        for name, model in CREATURE_MODELS.items():
            parts = _parts_of(model)
            self.assertGreaterEqual(len(parts), 3, f"{name} con pocas piezas")
            self.assertEqual(_tree_errors(parts), [], f"árbol de {name}")


class TestClips(unittest.TestCase):
    """Invariantes de los clips de tercera y primera persona."""

    def test_miner_clips_exact_set(self) -> None:
        names = {c["name"] for c in CLIPS_3P}
        self.assertEqual(len(CLIPS_3P), len(MINER_CLIPS))
        self.assertEqual(names, MINER_CLIPS)

    def test_first_person_clips_exact_set(self) -> None:
        names = {c["name"] for c in CLIPS_1P}
        self.assertEqual(len(CLIPS_1P), len(FIRST_PERSON_CLIPS))
        self.assertEqual(names, FIRST_PERSON_CLIPS)

    def test_miner_clip_tracks_reference_miner_parts(self) -> None:
        part_ids = set(MINER_PARENTS)
        for clip in CLIPS_3P:
            for track in clip["tracks"]:
                self.assertIn(track["part"], part_ids,
                              f"{clip['name']}: pieza {track['part']!r}")

    def test_clip_keys_sorted_and_in_range(self) -> None:
        self.assertEqual(_key_errors(CLIPS_3P), [])
        self.assertEqual(_key_errors(CLIPS_1P), [])

    def test_clip_markers_valid_and_in_range(self) -> None:
        self.assertEqual(_marker_errors(CLIPS_3P), [])
        self.assertEqual(_marker_errors(CLIPS_1P), [])

    def test_dig_clips_have_exactly_one_impact(self) -> None:
        self.assertEqual(len(_impacts_of(CLIPS_3P, "dig")), 1)
        self.assertEqual(len(_impacts_of(CLIPS_1P, "dig_1p")), 1)

    def test_first_person_reach_boxes_constrained(self) -> None:
        seen = 0
        for clip in CLIPS_1P:
            for box in clip.get("reach_boxes", []):
                lo, hi = box["min"], box["max"]
                seen += 1
                self.assertTrue(all(lo[i] < hi[i] for i in range(3)),
                                f"{clip['name']}/{box['id']}: min >= max")
                self.assertGreaterEqual(lo[2], -24,
                                        f"{clip['name']}/{box['id']}: min[2] < -24")
        self.assertGreater(seen, 0, "first_person_clips.ron sin reach_boxes")

    def test_first_person_clips_animate_the_arms_model(self) -> None:
        arms = ron_lite.load(MODELS_DIR / "first_person_arms.ron")
        ids = {p["id"] for p in arms["parts"]}
        self.assertEqual(ids, set(check_models.FP_ARM_PARTS))
        used = {t["part"] for c in CLIPS_1P for t in c["tracks"]}
        self.assertTrue(used <= ids, f"pistas 1P sin pieza en el modelo: {used - ids}")
        self.assertEqual([(a["id"], a["part"]) for a in arms["attachments"]], [("tool", "arm_r")])

    def test_miner_limbs_swing_on_pitch_and_alternate(self) -> None:
        self.assertEqual(check_models.limb_errors(CLIPS_3P), [])
        # Un clip con el braceo en roll, un codo al revés y muslos a la vez.
        bad = [{"name": "walk", "markers": [{"kind": "step_l"}, {"kind": "step_r"}], "tracks": [
            {"part": "upper_leg_l", "keys": [{"t_ms": 0, "rot_deg": (14, 0, 0)}]},
            {"part": "upper_leg_r", "keys": [{"t_ms": 0, "rot_deg": (14, 0, 0)}]},
            {"part": "upper_arm_l", "keys": [{"t_ms": 0, "rot_deg": (0, 0, 30)}]},
            {"part": "lower_arm_r", "keys": [{"t_ms": 0, "rot_deg": (-5, 0, 0)}]},
        ]}]
        errors = check_models.limb_errors(bad)
        self.assertEqual(len(errors), 3, errors)


class TestTextures32(unittest.TestCase):
    """Generador tex32: mip, verificación pública y determinismo."""

    def test_downsample_2x2_returns_16x16_palette(self) -> None:
        self.assertEqual(len(TEXTURES), len(tex32.BLOCKS))
        for name in tex32.BLOCKS:
            with self.subTest(block=name):
                mip = tex32.downsample_2x2(TEXTURES[name])
                self.assertEqual(len(mip), 16 * 16 * 4)
                for i in range(0, len(mip), 4):
                    self.assertIn(tuple(mip[i:i + 4]), palette.PALETTE_SET,
                                  f"{name}: píxel {i // 4} fuera de paleta")

    def test_tex32_verify_mip_rule_ok(self) -> None:
        ok, errors = tex32.verify_mip_rule(TEXTURES)
        self.assertTrue(ok, f"verify_mip_rule falló: {errors}")

    def test_generate_textures_deterministic(self) -> None:
        self.assertEqual(tex32.generate_textures(), tex32.generate_textures())


if __name__ == "__main__":
    unittest.main()
