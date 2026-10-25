"""Verificador de los datos de generación del mundo W1 (data/worldgen/*.ron).

Comprueba cotas, partición de capas, curvas Q16, biomas, excavadores y menas
contra los invariantes de docs/mundo/02-generacion.md. Solo biblioteca estándar
más el parser ron_lite de tools/balance. Se puede importar sin efectos o
ejecutar directamente:

    python tools/worldgen/check_worldgen.py
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

_THIS_DIR = Path(__file__).resolve().parent
_REPO_ROOT = _THIS_DIR.parents[1]
for _candidate in (_THIS_DIR, _REPO_ROOT / "tools" / "balance"):
    if str(_candidate) not in sys.path:
        sys.path.insert(0, str(_candidate))

import ron_lite  # noqa: E402

DATA_DIR = _REPO_ROOT / "data" / "worldgen"

WORLDGEN_FILES: tuple[str, ...] = (
    "mundo.ron", "relieve.ron", "biomas.ron", "cuevas.ron", "menas.ron",
)

# Nombres y órdenes exactos que exigen los invariantes de W1.
LAYER_NAMES: tuple[str, ...] = ("sky", "surface", "crust", "mantle", "vault", "roots", "bedrock")
NOISE_NAMES: tuple[str, ...] = (
    "continentality", "erosion", "peaks_valleys", "temperature", "humidity", "weirdness",
)
SURFACE_BIOME_NAMES: tuple[str, ...] = (
    "cordillera", "needle_forest", "highland", "red_desert",
    "badlands", "river_valley", "tundra", "inland_sea",
)
UNDERGROUND_BIOME_NAMES: tuple[str, ...] = (
    "limestone_caves", "wet_caves", "clay_galleries", "basalt_caverns", "geodes_quartz",
    "firedamp_grottos", "glow_fungi", "the_vault", "dreamed_veins",
)
CARVER_NAMES: tuple[str, ...] = (
    "cheese", "spaghetti", "noodles", "bounded_worms", "sinkholes", "canyons", "giant_chambers",
)
ORE_MATERIAL_NAMES: tuple[str, ...] = (
    "coal", "copper", "tin", "iron", "tungsten", "silver_lead",
    "cobalt", "gold", "diamond", "sulfur", "quartz",
)
CENTER_KEYS: tuple[str, ...] = (
    "continentality_q16", "erosion_q16", "temperature_q16", "humidity_q16", "weirdness_q16",
)

# Cotas y rangos de W1, en bloques o Q16 (65_536 = 1.0).
Q16_MIN, Q16_MAX = -65_536, 65_536
WORLD_Y_MIN, WORLD_Y_MAX = -4_096, 1_023
BLOCK_ID_MAX = 17
LIGHT_MAX = 15
OUTPUT_MIN, OUTPUT_MAX = -64, 950
CURVE_OUTPUT_LIMITS = {"base_height_curve": (OUTPUT_MIN, OUTPUT_MAX), "slope_factor_curve": (OUTPUT_MIN, OUTPUT_MAX)}


def _is_int(value: Any) -> bool:
    """Devuelve True si el valor es un int de Python pero no un bool."""
    return isinstance(value, int) and not isinstance(value, bool)


def load_texts() -> dict[str, str]:
    """Lee el texto crudo de los cinco archivos RON de worldgen."""
    return {name: (DATA_DIR / name).read_text(encoding="utf-8") for name in WORLDGEN_FILES}


def load_data() -> dict[str, dict[str, Any]]:
    """Analiza los cinco archivos RON y devuelve su raíz como dict por archivo."""
    return {name: ron_lite.load(DATA_DIR / name) for name in WORLDGEN_FILES}


def _as_dict_list(value: Any, label: str, errors: list[str]) -> list[dict[str, Any]]:
    """Valida que value sea una lista de structs y la devuelve (vacía si no)."""
    if not isinstance(value, list) or not all(isinstance(item, dict) for item in value):
        errors.append(f"{label}: se esperaba una lista de structs")
        return []
    return value


def check_mundo(world: dict[str, Any]) -> list[str]:
    """Invariantes de mundo.ron: cotas, partición de capas y macizo del Durmiente."""
    errors: list[str] = []
    extents = {
        "worldgen_version": 1,
        "x_min": -7_500_000, "x_max": 7_499_999,
        "z_min": -7_500_000, "z_max": 7_499_999,
        "y_min": WORLD_Y_MIN, "y_max": WORLD_Y_MAX,
        "border_fog_warning_blocks": 64,
    }
    for key, expected in extents.items():
        if world.get(key) != expected:
            errors.append(f"mundo.ron: {key} debe ser {expected} (es {world.get(key)!r})")

    layers = _as_dict_list(world.get("layers"), "mundo.ron: layers", errors)
    if layers:
        names = [layer.get("name") for layer in layers]
        if names != list(LAYER_NAMES):
            errors.append(f"mundo.ron: las capas deben ser {list(LAYER_NAMES)} en orden (son {names})")
        # El archivo lista las capas de arriba a abajo (sky primero); la
        # partición se comprueba sobre la vista ordenada por y_min.
        ordered = sorted(layers, key=lambda layer: layer.get("y_min", 0))
        if ordered[0].get("y_min") != WORLD_Y_MIN:
            errors.append(f"mundo.ron: la capa más baja debe empezar en {WORLD_Y_MIN}")
        if ordered[-1].get("y_max") != WORLD_Y_MAX:
            errors.append(f"mundo.ron: la capa más alta debe terminar en {WORLD_Y_MAX}")
        for prev, nxt in zip(ordered, ordered[1:]):
            if nxt.get("y_min") != prev.get("y_max", 0) + 1:
                errors.append(
                    f"mundo.ron: la capa {nxt.get('name')!r} debe empezar en "
                    f"{prev.get('y_max', '?')} + 1 (partición sin huecos ni solapes)"
                )

    massif = world.get("sleeper_massif")
    expected_massif = {
        "radius_blocks": 3_000, "peak_y": 900, "spawn_dist_min": 2_000, "spawn_dist_max": 5_000,
    }
    if not isinstance(massif, dict):
        errors.append("mundo.ron: falta sleeper_massif")
    else:
        for key, expected in expected_massif.items():
            if massif.get(key) != expected:
                errors.append(f"mundo.ron: sleeper_massif.{key} debe ser {expected} (es {massif.get(key)!r})")
    return errors


def check_relieve(relief: dict[str, Any]) -> list[str]:
    """Invariantes de relieve.ron: ruido 2D, curvas Q16 y umbral de ríos."""
    errors: list[str] = []
    params = _as_dict_list(relief.get("noise_params"), "relieve.ron: noise_params", errors)
    if params:
        if [p.get("name") for p in params] != list(NOISE_NAMES):
            errors.append(f"relieve.ron: noise_params debe ser {list(NOISE_NAMES)} en orden")
        for p in params:
            for key in ("period_shift", "octaves"):
                value = p.get(key)
                if not _is_int(value) or value <= 0:
                    errors.append(f"relieve.ron: {p.get('name')}.{key} debe ser entero positivo (es {value!r})")

    for curve_name in ("base_height_curve", "slope_factor_curve"):
        curve = _as_dict_list(relief.get(curve_name), f"relieve.ron: {curve_name}", errors)
        if not curve:
            continue
        inputs = [point.get("input_q16") for point in curve]
        if not all(_is_int(v) for v in inputs):
            errors.append(f"relieve.ron: {curve_name} tiene valores input_q16 no enteros")
            continue
        if inputs[0] != Q16_MIN:
            errors.append(f"relieve.ron: {curve_name} debe empezar en input_q16 = -65_536 (empieza en {inputs[0]})")
        if inputs[-1] != Q16_MAX:
            errors.append(f"relieve.ron: {curve_name} debe terminar en input_q16 = 65_536 (termina en {inputs[-1]})")
        if any(b <= a for a, b in zip(inputs, inputs[1:])):
            errors.append(f"relieve.ron: input_q16 de {curve_name} debe ser estrictamente creciente")
        lo, hi = CURVE_OUTPUT_LIMITS[curve_name]
        for point in curve:
            output = point.get("output")
            if not _is_int(output) or not lo <= output <= hi:
                errors.append(f"relieve.ron: output de {curve_name} fuera de [{lo}, {hi}]: {output!r}")

    threshold = relief.get("river_threshold_q16")
    if not _is_int(threshold) or not 0 <= threshold <= Q16_MAX:
        errors.append(f"relieve.ron: river_threshold_q16 debe estar en [0, 65_536] (es {threshold!r})")
    return errors


def check_biomas(biomes: dict[str, Any], world: dict[str, Any]) -> list[str]:
    """Invariantes de biomas.ron, con cruce de capas y cotas contra mundo.ron."""
    errors: list[str] = []
    layer_ranges = {
        layer.get("name"): (layer.get("y_min"), layer.get("y_max"))
        for layer in world.get("layers", [])
        if isinstance(layer, dict)
    }

    surface = _as_dict_list(biomes.get("surface_biomes"), "biomas.ron: surface_biomes", errors)
    if surface:
        if [b.get("name") for b in surface] != list(SURFACE_BIOME_NAMES):
            errors.append(f"biomas.ron: surface_biomes debe ser {list(SURFACE_BIOME_NAMES)} en orden")
        ids = [b.get("biome_id") for b in surface]
        if sorted(ids) != list(range(len(SURFACE_BIOME_NAMES))):
            errors.append("biomas.ron: los biome_id de superficie deben ser únicos y cubrir 0..=7")
        for biome in surface:
            for key in CENTER_KEYS:
                value = biome.get("center", {}).get(key)
                if not _is_int(value) or not Q16_MIN <= value <= Q16_MAX:
                    errors.append(f"biomas.ron: center.{key} de {biome.get('name')!r} fuera de [{Q16_MIN}, {Q16_MAX}]: {value!r}")
            for key in ("surface_block", "subsurface_block"):
                value = biome.get(key)
                if not _is_int(value) or not 0 <= value <= BLOCK_ID_MAX:
                    errors.append(f"biomas.ron: {key} de {biome.get('name')!r} fuera de 0..={BLOCK_ID_MAX}: {value!r}")

    underground = _as_dict_list(biomes.get("underground_biomes"), "biomas.ron: underground_biomes", errors)
    if underground:
        if [b.get("name") for b in underground] != list(UNDERGROUND_BIOME_NAMES):
            errors.append(f"biomas.ron: underground_biomes debe ser {list(UNDERGROUND_BIOME_NAMES)} en orden")
        ids = [b.get("biome_id") for b in underground]
        if sorted(ids) != list(range(len(UNDERGROUND_BIOME_NAMES))):
            errors.append("biomas.ron: los biome_id subterráneos deben ser únicos y cubrir 0..=8")
        for biome in underground:
            name = biome.get("name")
            light = biome.get("base_light")
            if not _is_int(light) or not 0 <= light <= LIGHT_MAX:
                errors.append(f"biomas.ron: base_light de {name!r} fuera de [0, {LIGHT_MAX}]: {light!r}")
            block = biome.get("dominant_block")
            if not _is_int(block) or not 0 <= block <= BLOCK_ID_MAX:
                errors.append(f"biomas.ron: dominant_block de {name!r} fuera de 0..={BLOCK_ID_MAX}: {block!r}")
            layer_name = biome.get("layer")
            if layer_name not in layer_ranges:
                errors.append(f"biomas.ron: {name!r} referencia la capa desconocida {layer_name!r}")
                continue
            lo, hi = layer_ranges[layer_name]
            y_min, y_max = biome.get("y_min"), biome.get("y_max")
            if not (_is_int(y_min) and _is_int(y_max)) or y_min < lo or y_max > hi or y_min > y_max:
                errors.append(
                    f"biomas.ron: {name!r} tiene cota [{y_min}, {y_max}] fuera de su capa "
                    f"{layer_name!r} [{lo}, {hi}]"
                )
    return errors


def check_cuevas(caves: dict[str, Any]) -> list[str]:
    """Invariantes de cuevas.ron: márgenes, excavadores y confinamiento de cámaras."""
    errors: list[str] = []
    if caves.get("bedrock_margin") != 4:
        errors.append(f"cuevas.ron: bedrock_margin debe ser 4 (es {caves.get('bedrock_margin')!r})")
    if caves.get("river_roof_min") != 2:
        errors.append(f"cuevas.ron: river_roof_min debe ser 2 (es {caves.get('river_roof_min')!r})")

    carvers = _as_dict_list(caves.get("carvers"), "cuevas.ron: carvers", errors)
    if not carvers:
        return errors
    if [c.get("name") for c in carvers] != list(CARVER_NAMES):
        errors.append(f"cuevas.ron: carvers debe ser {list(CARVER_NAMES)} en orden")
    floor = WORLD_Y_MIN + 4  # bedrock_margin exigido arriba
    for carver in carvers:
        name = carver.get("name")
        y_min, y_max = carver.get("y_min"), carver.get("y_max")
        if not _is_int(y_min) or y_min < floor:
            errors.append(f"cuevas.ron: {name!r} baja de y_min = {WORLD_Y_MIN} + bedrock_margin = {floor} (y_min {y_min!r})")
        if not _is_int(y_max) or y_max > WORLD_Y_MAX:
            errors.append(f"cuevas.ron: {name!r} supera y_max = {WORLD_Y_MAX} (y_max {y_max!r})")
        if _is_int(y_min) and _is_int(y_max) and y_min > y_max:
            errors.append(f"cuevas.ron: {name!r} tiene y_min > y_max ({y_min} > {y_max})")
        threshold = carver.get("threshold_q16")
        if not _is_int(threshold) or not 0 <= threshold <= Q16_MAX:
            errors.append(f"cuevas.ron: threshold_q16 de {name!r} fuera de [0, 65_536]: {threshold!r}")
        if name == "giant_chambers" and _is_int(y_min) and _is_int(y_max):
            if y_min < -640 or y_max > -450:
                errors.append(
                    f"cuevas.ron: giant_chambers debe quedar confinado a [-640, -450] "
                    f"(lagos de lava por debajo de -450); es [{y_min}, {y_max}]"
                )
    return errors


def check_menas(ores: dict[str, Any]) -> list[str]:
    """Invariantes de menas.ron: materiales, ids de bloque y curvas peso/cota."""
    errors: list[str] = []
    materials = _as_dict_list(ores.get("materials"), "menas.ron: materials", errors)
    if not materials:
        return errors
    if [m.get("material") for m in materials] != list(ORE_MATERIAL_NAMES):
        errors.append(f"menas.ron: materials debe ser {list(ORE_MATERIAL_NAMES)} en orden")
    for material in materials:
        name = material.get("material")
        block_id = material.get("block_id")
        if not _is_int(block_id) or not 0 <= block_id <= BLOCK_ID_MAX:
            errors.append(f"menas.ron: block_id de {name!r} fuera de 0..={BLOCK_ID_MAX}: {block_id!r}")
        curve = _as_dict_list(material.get("curve"), f"menas.ron: curve de {name!r}", errors)
        if len(curve) < 2:
            errors.append(f"menas.ron: la curva de {name!r} necesita al menos 2 puntos")
            continue
        ys = [point.get("y") for point in curve]
        weights = [point.get("weight_per_100k") for point in curve]
        if not all(_is_int(y) for y in ys) or not all(_is_int(w) for w in weights):
            errors.append(f"menas.ron: la curva de {name!r} tiene valores no enteros")
            continue
        if any(not WORLD_Y_MIN <= y <= WORLD_Y_MAX for y in ys):
            errors.append(f"menas.ron: la curva de {name!r} sale de [{WORLD_Y_MIN}, {WORLD_Y_MAX}]")
        if any(b <= a for a, b in zip(ys, ys[1:])):
            errors.append(f"menas.ron: la curva de {name!r} debe tener y estrictamente creciente")
        if any(w < 0 for w in weights):
            errors.append(f"menas.ron: la curva de {name!r} tiene pesos negativos")
        if ys[0] > -256:
            errors.append(f"menas.ron: la curva de {name!r} debe arrancar en y <= -256 (arranca en {ys[0]})")
        if ys[-1] < -256:
            errors.append(f"menas.ron: la curva de {name!r} debe alcanzar y >= -256 (termina en {ys[-1]})")
        if ys[-1] < 0 and weights[-1] != 0:
            errors.append(
                f"menas.ron: {name!r} cierra por debajo de 0 y debe terminar con peso 0 "
                f"(menas del manto); cierra con {weights[-1]}"
            )
    return errors


def run_checks() -> list[str]:
    """Ejecuta todas las comprobaciones y devuelve la lista de errores en español."""
    errors: list[str] = []
    texts: dict[str, str] = {}
    data: dict[str, dict[str, Any]] = {}
    for name in WORLDGEN_FILES:
        path = DATA_DIR / name
        if not path.is_file():
            errors.append(f"{name}: falta el archivo requerido en {path}")
            continue
        try:
            texts[name] = path.read_text(encoding="utf-8")
            data[name] = ron_lite.loads(texts[name])
        except Exception as exc:  # el verificador informa y sigue con el resto
            errors.append(f"{name}: no se pudo cargar ({exc})")

    for name, text in texts.items():
        if ron_lite.has_decimal_numbers(text):
            errors.append(f"{name}: contiene números con punto decimal (prohibidos)")

    world = data.get("mundo.ron")
    if world is not None:
        errors.extend(check_mundo(world))
    relief = data.get("relieve.ron")
    if relief is not None:
        errors.extend(check_relieve(relief))
    biomes = data.get("biomas.ron")
    if biomes is not None:
        if world is None:
            errors.append("biomas.ron: se omite el cruce de capas porque mundo.ron no cargó")
        else:
            errors.extend(check_biomas(biomes, world))
    caves = data.get("cuevas.ron")
    if caves is not None:
        errors.extend(check_cuevas(caves))
    ores = data.get("menas.ron")
    if ores is not None:
        errors.extend(check_menas(ores))
    return errors


def main() -> int:
    """Punto de entrada: imprime OK o cada error y sale con código 1 si falla."""
    errors = run_checks()
    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        print(f"FALLO: {len(errors)} error(es) de worldgen")
        return 1
    print("OK: los 5 archivos de data/worldgen cumplen todos los invariantes de W1")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
