"""Verificador de los datos de supervivencia de HUSHMINE (data/supervivencia/).

Candado de los 8 archivos RON del modo Historia: higiene (UTF-8, solo LF, <= 400
líneas, salto final), parseo sin flotantes, esquema, referencias cruzadas e
invariantes económicas. Ejecutar desde la raíz; código 0 si pasa, 1 si no.
"""
from __future__ import annotations

import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools" / "balance"))

import ron_lite  # noqa: E402

DATA_DIR = ROOT / "data" / "supervivencia"
MAX_LINES = 400  # regla de AGENTS.md: archivos pequeños

FILES: tuple[str, ...] = (  # los 8 archivos de datos, en orden de verificación
    "objetos.ron", "tecnologias.ron", "recetas.ron", "criaturas.ron",
    "jefes.ron", "dimensiones.ron", "comercio.ron", "encargos.ron")

MIN_ITEMS, MIN_TECHS, MIN_RECIPES, MIN_CREATURES = 36, 24, 28, 14
MIN_BOSSES, MIN_BOSS_PHASES, MIN_TRADERS, MIN_ORDERS = 5, 2, 5, 12

CATEGORIES = {"material", "tool", "lamp", "explosive", "machine_part", "relic", "consumable", "food"}
BRANCHES = {"excavation", "shoring_construction", "lighting_detection", "transport_machines", "silence_acoustics", "sleeper_craft"}
STATIONS = {"workbench", "forge", "pressure_workshop", "powder_magazine", "grow_table", "quiet_loom", "memory_altar"}
HABITATS = {"surface", "crust", "mantle", "vault", "roots", "the_dream", "the_seethe", "the_echo", "the_underside"}
LIGHT_REACTIONS = {"flee", "attract", "neutral", "petrify"}
LIGHT_RULES = {"lid_cycle", "drifting_epochs", "always_glow", "sound_light", "always_day"}
FACTIONS = {"arrivals", "the_still", "watchers", "consolidated"}
ORDER_KINDS = {"delivery", "survey", "drain", "reopen", "rescue", "escort", "offering", "contract", "listen", "message"}
MANDATORY_TRADERS = ("lamplighter", "blacksmith", "assayer", "cartographer", "quedo_barterer")

EXPECTED_DIMENSIONS = {  # dimensión -> índice exigido en dimensiones.ron
    "the_hush": 0, "the_dream": 1, "the_seethe": 2, "the_echo": 3, "the_underside": 4}

OK_GROUPS = ("[OK] higiene de archivos: UTF-8, solo LF, <= 400 líneas y salto final",
             "[OK] parseo RON correcto y sin números con punto decimal", "[OK] esquema e invariantes de los 8 catálogos",
             "[OK] integridad referencial cruzada", "[OK] sin ganancia de valor en recetas",
             "[OK] sin arbitraje en las ofertas de comercio")

def _is_int(value: Any) -> bool:  # los bool no cuentan como enteros
    return isinstance(value, int) and not isinstance(value, bool)

def _fail(errors: list[str], message: str) -> None:  # acumula un fallo estándar
    errors.append(f"[FALLO] {message}")

def _field(entry: Any, key: str) -> Any:  # valor bruto, tolera entradas no-struct
    return entry.get(key) if isinstance(entry, dict) else None

def _ctx(entry: Any, name: str, kind: str) -> str:  # etiqueta de mensaje con el id
    return f"{name} {kind} {entry['id']!r}" if isinstance(entry, dict) and isinstance(entry.get("id"), str) else f"{name} {kind} inválida"

def _rows_of(entry: Any, key: str) -> list[Any]:  # lista anidada válida, o vacía
    rows = _field(entry, key)
    return rows if isinstance(rows, list) else []

def _require_list(entry: Any, key: str, ctx: str, errors: list[str]) -> list[Any]:
    """Exige lista; devuelve la lista o una vacía tras anotar el fallo."""
    rows = _field(entry, key)
    if isinstance(rows, list):
        return rows
    _fail(errors, f"{ctx}: '{key}' debe ser lista")
    return []

def _id_set(docs: dict[str, Any], name: str, key: str) -> set[str] | None:  # None si falta el catálogo
    doc = docs.get(name)
    return {e["id"] for e in _rows_of(doc, key) if isinstance(e.get("id"), str)} if isinstance(doc, dict) else None

def _entry_int(entry: Any, key: str, ctx: str, errors: list[str], min_v: int | None = None, max_v: int | None = None) -> int | None:
    """Valida un campo entero y su rango; devuelve el valor o None."""
    value = _field(entry, key)
    if not _is_int(value):
        _fail(errors, f"{ctx}: el campo '{key}' debe ser entero")
        return None
    if (min_v is not None and value < min_v) or (max_v is not None and value > max_v):
        _fail(errors, f"{ctx}: '{key}'={value} fuera de rango {min_v}..{max_v}")
    return value

def _entry_str(entry: Any, key: str, ctx: str, errors: list[str], allowed: set[str] | None = None) -> str | None:
    """Valida un campo de cadena y, si procede, su conjunto cerrado."""
    value = _field(entry, key)
    if not isinstance(value, str):
        _fail(errors, f"{ctx}: el campo '{key}' debe ser cadena")
        return None
    if allowed is not None and value not in allowed:
        _fail(errors, f"{ctx}: '{key}'={value!r} fuera del conjunto permitido {sorted(allowed)}")
    return value

def _check_fields(entry: Any, ctx: str, errors: list[str],
                  str_fields: tuple[tuple[str, set[str] | None], ...],
                  int_fields: tuple[tuple[str, int | None, int | None], ...]) -> None:
    """Valida en bloque los campos simples (cadenas y enteros con rango)."""
    for key, allowed in str_fields:
        _entry_str(entry, key, ctx, errors, allowed)
    for key, min_v, max_v in int_fields:
        _entry_int(entry, key, ctx, errors, min_v, max_v)

def _check_unique(entries: list[Any], name: str, errors: list[str]) -> None:
    """Comprueba que todos los ids de la lista sean únicos."""
    seen: set[str] = set()
    for entry in entries:
        if isinstance(entry, dict) and isinstance(entry.get("id"), str):
            if entry["id"] in seen:
                _fail(errors, f"{name}: id duplicado {entry['id']!r}")
            seen.add(entry["id"])

def _prepare(doc: Any, key: str, name: str, min_count: int, errors: list[str]) -> list[Any] | None:
    """Raíz, recuento mínimo e ids únicos; devuelve la lista o None tras anotar."""
    entries = doc.get(key) if isinstance(doc, dict) else None
    if not isinstance(entries, list):
        _fail(errors, f"{name}: la raíz debe ser un struct con la lista '{key}'")
        return None
    if len(entries) < min_count:
        _fail(errors, f"{name}: se requieren >= {min_count} entradas, hay {len(entries)}")
    _check_unique(entries, name, errors)
    return entries

SPEC: dict[str, tuple[str, int, str, tuple, tuple]] = {  # campos planos: (clave, mínimo, etiqueta, str, int)
    "objetos.ron": ("items", MIN_ITEMS, "item", (("id", None), ("name", None), ("category", CATEGORIES)),
                    (("tier", 1, 5), ("max_stack", 1, None), ("durability", 0, None), ("weight", 1, None), ("base_value_gold", 0, None))),
    "tecnologias.ron": ("techs", MIN_TECHS, "tech", (("id", None), ("name", None), ("branch", BRANCHES)), (("tier", 1, 5),)),
    "recetas.ron": ("recipes", MIN_RECIPES, "receta", (("id", None), ("name", None), ("tech_required", None), ("station", STATIONS)),
                    (("craft_ticks", 1, None), ("noise_emitted", 0, None))),
    "criaturas.ron": ("creatures", MIN_CREATURES, "criatura",
                      (("id", None), ("name", None), ("habitat", HABITATS), ("light_reaction", LIGHT_REACTIONS)),
                      (("hearing_threshold", 0, None), ("hp", 1, None), ("damage", 0, None), ("noise_on_alert", 0, None))),
    "jefes.ron": ("bosses", MIN_BOSSES, "jefe", (("id", None), ("name", None)),
                  (("arc", 1, 5), ("hp", 1, None), ("damage", 0, None), ("acoustic_weakness", 0, 100), ("luminous_weakness", 0, 100))),
    "comercio.ron": ("traders", MIN_TRADERS, "trader", (("id", None), ("name", None), ("faction", FACTIONS)), ()),
    "encargos.ron": ("orders", MIN_ORDERS, "encargo",
                     (("id", None), ("name", None), ("kind", ORDER_KINDS), ("faction", FACTIONS), ("target_item_id", None)),
                     (("target_qty", 1, None), ("reward_gold", 0, None), ("reward_reputation", -100, 100))),
}

def check_file_hygiene(name: str, text: str, errors: list[str]) -> None:
    """(a) UTF-8 ya decodificado: solo LF, <= 400 líneas y salto de línea final."""
    if "\r" in text:
        _fail(errors, f"{name}: contiene saltos CRLF; usa solo LF")
    if len(text.splitlines()) > MAX_LINES:
        _fail(errors, f"{name}: {len(text.splitlines())} líneas (máximo {MAX_LINES})")
    if not text.endswith("\n"):
        _fail(errors, f"{name}: no termina en salto de línea")

def load_all(root: str | Path, errors: list[str] | None = None) -> dict[str, Any]:
    """Carga los 8 RON; anota '[FALLO] falta ...'; lanza FileNotFoundError si `errors` es None."""
    docs: dict[str, Any] = {}
    for name in FILES:
        path = Path(root) / "data" / "supervivencia" / name
        if path.is_file():
            docs[name] = ron_lite.load(path)
        elif errors is None:
            raise FileNotFoundError(f"falta data/supervivencia/{name}")
        else:
            _fail(errors, f"falta data/supervivencia/{name}")
    return docs

def _check_flat(doc: Any, name: str, errors: list[str]) -> list[Any] | None:
    """Valida la parte plana del catálogo `name` según SPEC; devuelve las entradas."""
    key, min_count, kind, str_fields, int_fields = SPEC[name]
    entries = _prepare(doc, key, name, min_count, errors)
    for entry in entries or []:
        _check_fields(entry, _ctx(entry, name, kind), errors, str_fields, int_fields)
    return entries

def check_items(doc: Any, errors: list[str]) -> None:
    """Esquema de objetos.ron: catálogo de items con tier y apilado válidos."""
    _check_flat(doc, "objetos.ron", errors)

def check_orders(doc: Any, errors: list[str]) -> None:
    """Esquema de encargos.ron: tipos y facciones cerrados, recompensa acotada."""
    _check_flat(doc, "encargos.ron", errors)

def check_techs(doc: Any, errors: list[str]) -> None:
    """tecnologias.ron: prereqs existentes y de tier menor; 6 ramas × 5 tiers."""
    entries = _check_flat(doc, "tecnologias.ron", errors)
    tiers = {e["id"]: e["tier"] for e in entries or [] if isinstance(e, dict) and isinstance(e.get("id"), str)}
    coverage: dict[str, set[int]] = {}
    for entry in entries or []:
        ctx = _ctx(entry, "tecnologias.ron", "tech")
        branch, tier = _field(entry, "branch"), _field(entry, "tier")
        if isinstance(branch, str) and _is_int(tier):
            coverage.setdefault(branch, set()).add(tier)  # rama -> tiers presentes
        for prereq in _require_list(entry, "prereqs", ctx, errors):
            target_tier = tiers.get(prereq) if isinstance(prereq, str) else None
            if target_tier is None:
                _fail(errors, f"{ctx}: el prereq {prereq!r} no existe")
            elif _is_int(tier) and target_tier >= tier:
                _fail(errors, f"{ctx}: el prereq {prereq!r} debe ser de tier estrictamente menor")
        if not isinstance(_field(entry, "unlocks"), list):
            _fail(errors, f"{ctx}: 'unlocks' debe ser lista")
    for branch in sorted(BRANCHES):
        missing = [t for t in range(1, 6) if t not in coverage.get(branch, set())]
        if missing:
            _fail(errors, f"tecnologias.ron: la rama '{branch}' no cubre los tiers {missing}")

def check_recipes(doc: Any, errors: list[str]) -> None:
    """recetas.ron: estaciones cerradas; inputs/outputs con cantidad >= 1."""
    entries = _check_flat(doc, "recetas.ron", errors)
    for entry in entries or []:
        ctx = _ctx(entry, "recetas.ron", "receta")
        for side in ("inputs", "outputs"):
            rows = _require_list(entry, side, ctx, errors)
            if not rows:
                _fail(errors, f"{ctx}: '{side}' no puede estar vacía")
            for row in rows:
                _check_fields(row, f"{ctx} ({side})", errors, (("item_id", None),), (("qty", 1, None),))

def check_creatures(doc: Any, errors: list[str]) -> None:
    """criaturas.ron: drops con 0 <= min_qty <= max_qty."""
    entries = _check_flat(doc, "criaturas.ron", errors)
    for entry in entries or []:
        ctx = _ctx(entry, "criaturas.ron", "criatura")
        for row in _require_list(entry, "drops", ctx, errors):
            _check_fields(row, f"{ctx} (drops)", errors, (("item_id", None),), (("min_qty", 0, None), ("max_qty", 0, None)))
            min_qty, max_qty = _field(row, "min_qty"), _field(row, "max_qty")
            if _is_int(min_qty) and _is_int(max_qty) and min_qty > max_qty:
                _fail(errors, f"{ctx} (drops): min_qty ({min_qty}) > max_qty ({max_qty})")

def check_bosses(doc: Any, errors: list[str]) -> None:
    """jefes.ron: >= 2 fases con hp_below_pct 0..=100 y recompensas válidas."""
    entries = _check_flat(doc, "jefes.ron", errors)
    for entry in entries or []:
        ctx = _ctx(entry, "jefes.ron", "jefe")
        phases = _require_list(entry, "phases", ctx, errors)
        if len(phases) < MIN_BOSS_PHASES:
            _fail(errors, f"{ctx}: debe tener >= {MIN_BOSS_PHASES} fases")
        seen: set[str] = set()
        for phase in phases:
            pctx = _ctx(phase, ctx, "fase")
            phase_id = _entry_str(phase, "id", pctx, errors)
            if isinstance(phase_id, str) and (phase_id in seen or seen.add(phase_id)):  # add -> None
                _fail(errors, f"{ctx}: fase duplicada {phase_id!r}")
            _check_fields(phase, pctx, errors, (("name", None), ("tactic", None)), (("hp_below_pct", 0, 100),))
        for row in _require_list(entry, "rewards", ctx, errors):
            _check_fields(row, f"{ctx} (rewards)", errors, (("item_id", None),), (("qty", 1, None),))

def check_dimensions(doc: Any, errors: list[str]) -> None:
    """dimensiones.ron: exactamente las 5, índices correctos y luz sin repetir."""
    entries = _prepare(doc, "dimensions", "dimensiones.ron", len(EXPECTED_DIMENSIONS), errors)
    rules_seen: list[str] = []
    for entry in entries or []:
        ctx = _ctx(entry, "dimensiones.ron", "dimensión")
        dim_id = _entry_str(entry, "id", ctx, errors)
        if isinstance(dim_id, str) and dim_id not in EXPECTED_DIMENSIONS:
            _fail(errors, f"{ctx}: id desconocido; esperados {sorted(EXPECTED_DIMENSIONS)}")
        _check_fields(entry, ctx, errors, (("name", None),), (("index", None, None), ("noise_multiplier_q16", 1, None)))
        index = _field(entry, "index")
        if isinstance(dim_id, str) and _is_int(index) and index != EXPECTED_DIMENSIONS.get(dim_id):
            _fail(errors, f"{ctx}: 'index' debe ser {EXPECTED_DIMENSIONS.get(dim_id, '?')}")
        if _entry_int(entry, "gravity_q16", ctx, errors) == 0:
            _fail(errors, f"{ctx}: gravity_q16 no puede ser 0 (65_536 representa 1g)")
        rule = _entry_str(entry, "light_rule", ctx, errors, allowed=LIGHT_RULES)
        if isinstance(rule, str):
            rules_seen.append(rule)
        material = _field(entry, "exclusive_material")
        if not isinstance(material, str):
            _fail(errors, f"{ctx}: 'exclusive_material' debe ser cadena")
        elif material == "" and dim_id != "the_hush":
            _fail(errors, f"{ctx}: solo the_hush admite exclusive_material vacío")
    if len(entries or []) == len(EXPECTED_DIMENSIONS) and sorted(rules_seen) != sorted(LIGHT_RULES):
        _fail(errors, f"dimensiones.ron: cada light_rule debe aparecer una vez; visto {rules_seen}")

def check_commerce(doc: Any, errors: list[str]) -> None:
    """comercio.ron: traders obligatorios y reputación ordenada con descuentos."""
    entries = _check_flat(doc, "comercio.ron", errors)
    by_id = {e["id"] for e in entries or [] if isinstance(e, dict) and isinstance(e.get("id"), str)}
    for required in MANDATORY_TRADERS:
        if required not in by_id:
            _fail(errors, f"comercio.ron: falta el trader obligatorio {required!r}")
    for entry in entries or []:
        ctx = _ctx(entry, "comercio.ron", "trader")
        for row in _require_list(entry, "offers", ctx, errors):
            _check_fields(row, f"{ctx} (oferta)", errors, (("item_id", None),), (("buy_price_gold", 1, None), ("sell_price_gold", 1, None)))
        prev_min: int | None = None
        prev_discount: int | None = None
        for row in _require_list(entry, "reputation_tiers", ctx, errors):
            _check_fields(row, f"{ctx} (reputation_tiers)", errors, (), (("min_reputation", -100, 100), ("discount_pct", 0, 60)))
            min_rep, discount = _field(row, "min_reputation"), _field(row, "discount_pct")
            if _is_int(min_rep) and _is_int(prev_min) and min_rep < prev_min:
                _fail(errors, f"{ctx} (reputation_tiers): debe estar ordenado ascendente por min_reputation")
            if _is_int(discount) and _is_int(prev_discount) and discount < prev_discount:
                _fail(errors, f"{ctx} (reputation_tiers): discount_pct debe ser no decreciente")
            prev_min, prev_discount = (min_rep if _is_int(min_rep) else prev_min), (discount if _is_int(discount) else prev_discount)

def check_cross_refs(docs: dict[str, Any], errors: list[str]) -> None:
    """(d) Todo item_id y tech_id referenciado existe en su catálogo de origen."""
    item_ids = _id_set(docs, "objetos.ron", "items")
    tech_ids = _id_set(docs, "tecnologias.ron", "techs")

    def ref(value: Any, catalog: set[str] | None, label: str, ctx: str) -> None:
        # catálogo ausente (ya notificado) o tipo inválido (ya en esquema): se omite
        if catalog is not None and isinstance(value, str) and value not in catalog:
            _fail(errors, f"{ctx}: la referencia {value!r} no existe en {label}")

    for recipe in _rows_of(docs.get("recetas.ron"), "recipes"):
        ctx = _ctx(recipe, "recetas.ron", "receta")
        for side in ("inputs", "outputs"):
            for row in _rows_of(recipe, side):
                ref(_field(row, "item_id"), item_ids, "objetos.ron", f"{ctx} ({side})")
        ref(_field(recipe, "tech_required"), tech_ids, "tecnologias.ron", ctx)
    for tech in _rows_of(docs.get("tecnologias.ron"), "techs"):
        ctx = _ctx(tech, "tecnologias.ron", "tech")
        for item_id in _rows_of(tech, "unlocks"):
            ref(item_id, item_ids, "objetos.ron", ctx)
        for prereq in _rows_of(tech, "prereqs"):
            ref(prereq, tech_ids, "tecnologias.ron", ctx)
    # Drops de criaturas, recompensas de jefes y ofertas apuntan al catálogo de items.
    for name, key, row_key in (("criaturas.ron", "creatures", "drops"), ("jefes.ron", "bosses", "rewards"),
                               ("comercio.ron", "traders", "offers")):
        for holder in _rows_of(docs.get(name), key):
            for row in _rows_of(holder, row_key):
                ref(_field(row, "item_id"), item_ids, "objetos.ron", _ctx(holder, name, key))
    for order in _rows_of(docs.get("encargos.ron"), "orders"):
        ref(_field(order, "target_item_id"), item_ids, "objetos.ron", _ctx(order, "encargos.ron", "encargo"))
    for dim in _rows_of(docs.get("dimensiones.ron"), "dimensions"):  # "" solo vale en the_hush
        if _field(dim, "exclusive_material") != "":
            ref(_field(dim, "exclusive_material"), item_ids, "objetos.ron", _ctx(dim, "dimensiones.ron", "dimensión"))

def _recipe_value(recipe: Any, side: str, values: dict[str, int]) -> int:
    """Valor en oro de un lado de la receta; -1 si el esquema no es fiable aún."""
    subtotal = 0
    for row in _rows_of(recipe, side):
        qty, value = (_field(row, "qty"), values.get(_field(row, "item_id"))) if isinstance(row, dict) else (None, None)
        if not _is_int(qty) or not _is_int(value):
            return -1  # esquema o referencias inválidas: ya notificados en otras fases
        subtotal += qty * value
    return subtotal

def check_no_profit(docs: dict[str, Any], errors: list[str]) -> None:
    """(e) Ninguna receta produce más valor en oro del que consume."""
    if "objetos.ron" not in docs or "recetas.ron" not in docs:
        return  # catálogos ausentes: ya notificados como falta de archivo
    values = {e["id"]: e["base_value_gold"] for e in _rows_of(docs.get("objetos.ron"), "items")
              if isinstance(e.get("id"), str) and _is_int(e.get("base_value_gold"))}
    for recipe in _rows_of(docs.get("recetas.ron"), "recipes"):
        spent, produced = _recipe_value(recipe, "inputs", values), _recipe_value(recipe, "outputs", values)
        if 0 <= spent < produced:
            _fail(errors, f"recetas.ron receta {_field(recipe, 'id')!r}: produce valor {produced} > consume {spent} (ganancia prohibida)")

def check_no_arbitrage(docs: dict[str, Any], errors: list[str]) -> None:
    """(f) Ningún comerciante compra más caro de lo que vende (buy > sell)."""
    for trader in _rows_of(docs.get("comercio.ron"), "traders"):
        for row in _rows_of(trader, "offers"):
            buy, sell = _field(row, "buy_price_gold"), _field(row, "sell_price_gold")
            if _is_int(buy) and _is_int(sell) and buy <= sell:
                _fail(errors, f"{_ctx(trader, 'comercio.ron', 'trader')}: la oferta {_field(row, 'item_id')!r} tiene buy_price_gold ({buy}) <= sell_price_gold ({sell}); no puede haber arbitraje")

SCHEMA_CHECKS = (("objetos.ron", check_items), ("tecnologias.ron", check_techs),
                 ("recetas.ron", check_recipes), ("criaturas.ron", check_creatures),
                 ("jefes.ron", check_bosses), ("dimensiones.ron", check_dimensions),
                 ("comercio.ron", check_commerce), ("encargos.ron", check_orders))

def run_checks(root: str | Path = ROOT) -> list[str]:
    """Ejecuta todas las comprobaciones en orden y devuelve la lista de fallos."""
    errors: list[str] = []
    texts: dict[str, str] = {}
    docs: dict[str, Any] = {}
    data_dir = Path(root) / "data" / "supervivencia"
    for name in FILES:  # (a) higiene: existencia, UTF-8, LF, 400 líneas, salto final
        path = data_dir / name
        if not path.is_file():
            _fail(errors, f"falta data/supervivencia/{name}")
            continue
        try:
            texts[name] = path.read_bytes().decode("utf-8")
        except UnicodeDecodeError:
            _fail(errors, f"{name}: no es UTF-8 válido")
            continue
        check_file_hygiene(name, texts[name], errors)
    for name, text in texts.items():  # (b) parseo RON y prohibición de flotantes
        if ron_lite.has_decimal_numbers(text):
            _fail(errors, f"{name}: contiene números con punto decimal (prohibidos)")
            continue
        try:
            docs[name] = ron_lite.loads(text)
        except ValueError as exc:
            _fail(errors, f"{name}: no parsea como RON ({exc})")
    for name, check in SCHEMA_CHECKS:  # (c) esquema e invariantes por archivo
        if name in docs:
            check(docs[name], errors)
    check_cross_refs(docs, errors)
    check_no_profit(docs, errors)
    check_no_arbitrage(docs, errors)
    return errors

def main() -> int:
    """Punto de entrada: imprime fallos (código 1) o los [OK] por grupo (código 0)."""
    errors = run_checks(ROOT)
    for line in errors or OK_GROUPS:
        print(line)
    print(f"Verificación de datos de supervivencia: {'todo correcto' if not errors else str(len(errors)) + ' fallo(s)'}")
    return 1 if errors else 0

if __name__ == "__main__":
    raise SystemExit(main())
