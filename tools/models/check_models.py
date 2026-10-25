"""Verificador de modelos, enganches, clips y mip 32 px de HUSHMINE.

Comprueba los cinco archivos RON de la entrega (assets/models/miner.ron,
tools.ron y creatures.ron, assets/anim/miner_clips.ron y
first_person_clips.ron) y la regla del mip de 16×16 de tex32. Imprime una
línea [OK]/[FALLO] por comprobación y termina con 0 (todo correcto) o 1.

Uso, desde la raíz del repositorio:

    python tools/models/check_models.py
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

REPO_ROOT: Path = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.balance import ron_lite  # noqa: E402
from tools.models import tex32  # noqa: E402

MODELS_DIR: Path = REPO_ROOT / "assets" / "models"
ANIM_DIR: Path = REPO_ROOT / "assets" / "anim"

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

# Herramientas: nombre -> slot esperado.
TOOL_SLOTS: dict[str, str] = {
    "pickaxe": "tool",
    "shovel": "tool",
    "shears": "tool",
    "geophone": "tool",
    "satchel_charge": "tool",
    "helmet_lamp": "lamp",
}

CREATURE_NAMES: tuple[str, ...] = ("whistler", "ore_beetle", "stone_stalker")

MINER_CLIPS: tuple[str, ...] = (
    "idle", "walk", "run", "crouch", "dig", "place", "ladder", "carry", "geophone_listen",
)
FIRST_PERSON_CLIPS: tuple[str, ...] = ("idle_1p", "dig_1p", "place_1p", "carry_1p")

MARKER_KINDS: frozenset[str] = frozenset({"impact", "step_l", "step_r", "release"})

# Límite de saltos padre-arriba en el minero: 14 piezas, cadena máxima de 13.
MINER_MAX_HOPS: int = 13

FAILURES: list[str] = []


def unpack_option(value: Any) -> Any:
    """Convierte el parseo de ron_lite: Some(x) -> x y None -> None."""
    if value == "None":
        return None
    if isinstance(value, tuple) and len(value) == 1:
        return value[0]
    return value


def check(ok: bool, message: str, detail: str = "") -> bool:
    """Imprime una línea del informe y registra el fallo si lo hay."""
    if ok:
        print(f"[OK] {message}")
    else:
        suffix = f": {detail}" if detail else ""
        print(f"[FALLO] {message}{suffix}")
        FAILURES.append(message)
    return ok


def load_ron(path: Path) -> Any | None:
    """Carga un .ron comprobando que no tenga decimales (una línea de informe)."""
    label = path.relative_to(REPO_ROOT).as_posix()
    if not path.is_file():
        check(False, f"{label} carga sin decimales", "no se encuentra el archivo")
        return None
    text = path.read_text(encoding="utf-8")
    if ron_lite.has_decimal_numbers(text):
        check(False, f"{label} carga sin decimales", "contiene números con punto decimal")
        return None
    try:
        data = ron_lite.load(path)
    except ValueError as exc:
        check(False, f"{label} carga sin decimales", str(exc))
        return None
    check(True, f"{label} carga sin decimales")
    return data


def validate_tree(parts: list[dict], label: str, max_hops: int | None = None) -> None:
    """Comprueba padres existentes, raíz única y ausencia de ciclos en un modelo."""
    problems: list[str] = []
    ids = [p.get("id") for p in parts]
    if not parts:
        problems.append("sin piezas")
    if len(set(ids)) != len(ids):
        problems.append("ids de pieza duplicados")
    by_id = {p.get("id"): p for p in parts}
    for part in parts:
        parent = unpack_option(part.get("parent"))
        if parent is not None and parent not in by_id:
            problems.append(f"{part.get('id')}: padre {parent!r} inexistente")
    roots = [i for i in ids if unpack_option(by_id[i].get("parent")) is None]
    if len(roots) != 1:
        problems.append(f"{len(roots)} raíces (se esperaba 1)")
    limit = max_hops if max_hops is not None else len(parts)
    for part in parts:
        current, hops = part, 0
        while True:
            parent = unpack_option(current.get("parent"))
            if parent is None or parent not in by_id:
                break
            current = by_id[parent]
            hops += 1
            if hops > limit:
                problems.append(f"{part.get('id')}: ciclo o cadena de más de {limit} pasos")
                break
    check(not problems, f"{label}: árbol de piezas válido", "; ".join(problems[:4]))


def invalid_cuboids(parts: list[dict]) -> list[str]:
    """Ids de piezas cuyo cuboid no cumple min < max componente a componente."""
    bad: list[str] = []
    for part in parts:
        cub = part.get("cuboid")
        lo = cub.get("min") if isinstance(cub, dict) else None
        hi = cub.get("max") if isinstance(cub, dict) else None
        if not (isinstance(lo, tuple) and isinstance(hi, tuple)
                and len(lo) == 3 and len(hi) == 3):
            bad.append(f"{part.get('id')}: cuboid malformado")
        elif not all(lo[i] < hi[i] for i in range(3)):
            bad.append(f"{part.get('id')}: min >= max")
    return bad


def clip_errors(clips: list[dict], valid_parts: set[str] | None) -> list[str]:
    """Claves en rango y crecientes, markers válidos y piezas existentes."""
    errors: list[str] = []
    for clip in clips:
        name, duration = clip.get("name"), clip.get("duration_ms")
        for track in clip.get("tracks", []):
            part = track.get("part")
            if valid_parts is not None and part not in valid_parts:
                errors.append(f"{name}: pieza {part!r} no está en el modelo")
            previous = -1
            for key in track.get("keys", []):
                t = key.get("t_ms")
                if not isinstance(t, int) or t < 0 or t > duration:
                    errors.append(f"{name}/{part}: t_ms {t} fuera de [0, {duration}]")
                elif t <= previous:
                    errors.append(f"{name}/{part}: t_ms {t} no creciente")
                previous = t if isinstance(t, int) else previous
        for marker in clip.get("markers", []):
            kind, t = marker.get("kind"), marker.get("t_ms")
            if kind not in MARKER_KINDS:
                errors.append(f"{name}: marker {kind!r} no válido")
            if not isinstance(t, int) or not 0 < t < duration:
                errors.append(f"{name}: marker {kind!r} con t_ms {t} fuera de (0, {duration})")
    return errors


def box_errors(clips: list[dict]) -> list[str]:
    """En reach_boxes: min < max componente a componente y min[2] >= -24."""
    errors: list[str] = []
    for clip in clips:
        for box in clip.get("reach_boxes", []):
            bid, lo, hi = box.get("id"), box.get("min"), box.get("max")
            label = f"{clip.get('name')}/{bid}"
            if not (isinstance(lo, tuple) and isinstance(hi, tuple)
                    and len(lo) == 3 and len(hi) == 3):
                errors.append(f"{label}: caja malformada")
                continue
            if not all(lo[i] < hi[i] for i in range(3)):
                errors.append(f"{label}: min >= max")
            if lo[2] < -24:
                errors.append(f"{label}: min[2] = {lo[2]} < -24")
    return errors


def impacts_of(clips: list[dict], name: str) -> list[dict]:
    """Markers impact del clip pedido (lista vacía si el clip no existe)."""
    clip = next((c for c in clips if c.get("name") == name), None)
    if clip is None:
        return []
    return [m for m in clip.get("markers", []) if m.get("kind") == "impact"]


def review_miner(miner: dict | None) -> None:
    """Comprobaciones del esqueleto y los enganches del minero."""
    if miner is None:
        return
    parts = miner.get("parts", [])
    ids = [p.get("id") for p in parts]
    check(len(ids) == 14 and set(ids) == set(MINER_PARENTS),
          "miner.ron: 14 piezas con el conjunto exacto de ids",
          f"encontradas {len(ids)}: {sorted(str(i) for i in ids)}")
    parents = {p.get("id"): unpack_option(p.get("parent")) for p in parts}
    diffs = {k: (parents.get(k), expected)
             for k, expected in MINER_PARENTS.items() if parents.get(k) != expected}
    check(parents == MINER_PARENTS, "miner.ron: relaciones de parent según la tabla",
          f"diferencias (id: encontrado, esperado): {diffs}")
    attachments = {a.get("id"): a.get("part") for a in miner.get("attachments", [])}
    check(len(miner.get("attachments", [])) == 3 and attachments == MINER_ATTACHMENTS,
          "miner.ron: 3 enganches lamp/tool/back en head/hand_r/torso",
          f"encontrados: {attachments}")
    validate_tree(parts, "miner.ron", max_hops=MINER_MAX_HOPS)
    bad = invalid_cuboids(parts)
    check(not bad, "miner.ron: cuboides con min < max", "; ".join(bad[:3]))


def review_models(data: dict | None, label: str, slots: dict[str, str] | None,
                  min_parts: int) -> None:
    """Comprobaciones comunes de un ModelSet (herramientas o criaturas)."""
    if data is None:
        return
    models = data.get("models", [])
    names = [m.get("name") for m in models]
    expected = set(slots) if slots is not None else set(names)
    check(len(names) == len(expected) and set(names) == expected,
          f"{label}: modelos exactos {sorted(expected)}",
          f"encontrados {len(names)}: {sorted(str(n) for n in names)}")
    if slots is not None:
        bad = [n for n, m in zip(names, models)
               if unpack_option(m.get("slot")) != slots.get(n)]
        check(not bad, f"{label}: slots correctos (tool×5, lamp para helmet_lamp)",
              f"slot equivocado en {bad}")
    for model in models:
        name, parts = model.get("name"), model.get("parts", [])
        check(len(parts) >= min_parts,
              f"{label}/{name}: al menos {min_parts} pieza(s)",
              f"tiene {len(parts)}")
        validate_tree(parts, f"{label}/{name}")
    bad = [f for m in models for f in invalid_cuboids(m.get("parts", []))]
    check(not bad, f"{label}: cuboides con min < max", "; ".join(bad[:3]))


def review_clips(data: dict | None, label: str, expected: tuple[str, ...],
                 valid_parts: set[str] | None, with_boxes: bool) -> None:
    """Comprobaciones comunes de un ClipSet (tercera o primera persona)."""
    if data is None:
        return
    clips = data.get("clips", [])
    names = [c.get("name") for c in clips]
    check(len(names) == len(expected) and set(names) == set(expected),
          f"{label}: clips exactos {list(expected)}",
          f"encontrados {len(names)}: {sorted(str(n) for n in names)}")
    errors = clip_errors(clips, valid_parts)
    check(not errors, f"{label}: pistas y markers válidos", "; ".join(errors[:4]))
    target = "dig" if "dig" in names else ("dig_1p" if "dig_1p" in names else None)
    if target is not None:
        impacts = impacts_of(clips, target)
        check(len(impacts) == 1, f"{label}: {target} con exactamente un marker impact",
              f"tiene {len(impacts)}")
    if with_boxes:
        found_box_errors = box_errors(clips)
        check(not found_box_errors, f"{label}: reach_boxes con min < max y z_min >= -24",
              "; ".join(found_box_errors[:4]))


def main() -> None:
    """Ejecuta todas las comprobaciones y termina con 0 o 1."""
    print("Verificación de modelos, clips y mip 32 px de HUSHMINE")
    miner = load_ron(MODELS_DIR / "miner.ron")
    tools = load_ron(MODELS_DIR / "tools.ron")
    creatures = load_ron(MODELS_DIR / "creatures.ron")
    miner_clips = load_ron(ANIM_DIR / "miner_clips.ron")
    fp_clips = load_ron(ANIM_DIR / "first_person_clips.ron")

    review_miner(miner)
    review_models(tools, "tools.ron", TOOL_SLOTS, min_parts=1)
    review_models(creatures, "creatures.ron", None, min_parts=3)

    review_clips(miner_clips, "miner_clips.ron", MINER_CLIPS,
                 set(MINER_PARENTS), with_boxes=False)
    # Los ids de pieza en primera persona (arm_l/arm_r) son de otro modelo: no
    # se validan contra miner.ron.
    review_clips(fp_clips, "first_person_clips.ron", FIRST_PERSON_CLIPS,
                 None, with_boxes=True)

    mip_ok, mip_errors = tex32.verify_mip_rule()
    check(mip_ok, "tex32: regla del mip de 16×16 (paleta, bandas, distinción y mosaico)",
          "; ".join(mip_errors[:4]))

    print()
    if FAILURES:
        print(f"VERIFICACIÓN FALLIDA: {len(FAILURES)} comprobaciones con fallo")
        sys.exit(1)
    print("VERIFICACIÓN OK: modelos, clips y mip 32 px correctos")
    sys.exit(0)


if __name__ == "__main__":
    main()
