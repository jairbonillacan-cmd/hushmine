#!/usr/bin/env python3
"""Verificador de catálogos Project Fluent (.ftl) y glosario i18n de HUSHMINE."""

from __future__ import annotations

from pathlib import Path
import re
import sys

REPO_ROOT = Path(__file__).resolve().parents[2]
MAX_LINES, MAX_WATCHER_WORDS, MIN_REGIONAL_DIFFS = 400, 6, 15
LOCALES: tuple[str, ...] = ("en", "es-419", "es-ES")
FTL_FILES: tuple[str, ...] = ("game.ftl", "hud.ftl", "watcher.ftl", "signs.ftl", "chronicle.ftl")

ID_PATTERN = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")
ENTRY_PATTERN = re.compile(r"^([^\s=#]+)\s*=\s*(.*)$")
VAR_PATTERN = re.compile(r"\$([a-zA-Z_][a-zA-Z0-9_]*)")
SIMPLE_PLACEABLE_PATTERN = re.compile(r"^\s*\$([a-zA-Z_][a-zA-Z0-9_]*)\s*$")
SELECTOR_HEADER_PATTERN = re.compile(r"^\s*\$([a-zA-Z_][a-zA-Z0-9_]*)\s*->\s*(.+?)\s*$", re.DOTALL)
VARIANT_MARKER_PATTERN = re.compile(r"(\*?)\[([^\]]*)\]")
VARIANT_KEY_PATTERN = re.compile(r"^[a-zA-Z0-9_-]+$")
INVENTORY_ROW_PATTERN = re.compile(r"^\|\s*`([^`]+)`\s*\|\s*`([^`]+\.ftl)`\s*\|")
VIGIA_ROW_PATTERN = re.compile(r"^\|\s*([^|]+?)\s*\|\s*(watcher-[a-z0-9-]+)\s*\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|$")
FORBIDDEN_DORMIDO_PATTERN = re.compile(r"\b(?:(?:el|al|del)\s+[Dd]ormido|Dormido)\b")

ALLOWED_PREFIXES: dict[str, tuple[str, ...]] = {
    "game.ftl": ("game-", "store-", "team-", "mode-", "class-", "tool-", "block-", "seismo-", "lore-"),
    "hud.ftl": ("menu-", "hud-", "altar-", "death-"),
    "watcher.ftl": ("watcher-",),
    "signs.ftl": ("sign-", "memo-", "tally-board-"),
    "chronicle.ftl": ("chronicle-",),
}
EXPECTED_BLOCKS: tuple[str, ...] = (
    "block-air", "block-dirt", "block-rock", "block-hard-rock", "block-ore", "block-gold",
    "block-diamond", "block-lava", "block-team-block", "block-force-field", "block-ladder",
    "block-jump-pad", "block-electric", "block-rail", "block-beacon", "block-explosive",
    "block-bucket-station", "block-bucket-post",
)
CHRONICLE_TITLES: dict[str, str] = {
    "chronicle-title-awakener": "Despertador",
    "chronicle-title-slumber-guardian": "Guardián del sueño",
    "chronicle-title-mole": "Topo",
    "chronicle-title-bucket-thief": "Ladrón de cangilones",
}
FORBIDDEN_SPANISH_TERMS: tuple[str, ...] = (
    "Dormido", "El Que Duerme", "Veta Principal", "Filón Madre", "Línea de cubos",
    "Línea de baldes", "Chapa de bronce", "Cobre Crew", "Cobalto Crew",
)
GLOSSARY_ALIGNMENT: tuple[tuple[str, str, bool], ...] = (
    ("Sleeper", "durmiente", False), ("Mother Lode", "veta madre", False),
    ("Seismograph", "sismógrafo", False), ("Geophone", "geófono", False),
    ("Whistlers", "silbadores", False), ("HUSHMINE", "HUSHMINE", True),
)


def parse_ftl_text(text: str, source: str = "<memory>") -> tuple[dict[str, str], list[str]]:
    """Parsea un recurso .ftl y devuelve (mensajes, errores)."""
    messages: dict[str, str] = {}
    parts_by_id: dict[str, list[str]] = {}
    order: list[str] = []
    errors: list[str] = []
    current_id: str | None = None
    for lineno, raw_line in enumerate(text.splitlines(), start=1):
        if not raw_line.strip() or raw_line.startswith("#"):
            continue
        if raw_line[0] in (" ", "\t"):
            if current_id is None:
                errors.append(f"{source}:{lineno}: línea con sangría sin entrada activa.")
            else:
                parts_by_id[current_id].append(raw_line.strip())
            continue
        match = ENTRY_PATTERN.match(raw_line)
        if not match:
            errors.append(f"{source}:{lineno}: sintaxis Fluent inválida: {raw_line!r}")
            current_id = None
            continue
        msg_id, first_val = match.group(1), match.group(2).strip()
        if msg_id in parts_by_id:
            errors.append(f"{source}:{lineno}: ID duplicado {msg_id!r}.")
            current_id = None
            continue
        current_id = msg_id
        order.append(msg_id)
        parts_by_id[msg_id] = [first_val] if first_val else []

    for msg_id in order:
        value = " ".join(parts_by_id[msg_id]).strip()
        if not value:
            errors.append(f"{source}: el mensaje {msg_id!r} tiene un valor vacío.")
        messages[msg_id] = value
    return messages, errors


def validate_id_naming(messages: dict[str, str], filename: str | None = None, source: str = "<memory>") -> list[str]:
    """Comprueba que cada ID cumpla kebab-case y los prefijos permitidos por archivo."""
    errors: list[str] = []
    prefixes = ALLOWED_PREFIXES.get(filename or "", ())
    for msg_id in messages:
        if not ID_PATTERN.match(msg_id):
            errors.append(f"{source}: ID {msg_id!r} no cumple el formato kebab-case.")
        elif prefixes and not msg_id.startswith(prefixes):
            errors.append(f"{source}: ID {msg_id!r} no usa un prefijo permitido en {filename}.")
    return errors


def extract_variables(value: str) -> set[str]:
    """Extrae todas las variables $identificador de un valor Fluent."""
    return set(VAR_PATTERN.findall(value))


def _scan_braces(text: str, mask: bool = False) -> tuple[str, list[str], bool]:
    """Extrae bloques { ... } de primer nivel y opcionalmente los enmascara."""
    blocks: list[str] = []
    out: list[str] = []
    depth, start = 0, -1
    for idx, ch in enumerate(text):
        if ch == "{":
            if depth == 0:
                start = idx + 1
                if mask:
                    out.append(" __PLACEABLE__ ")
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth < 0:
                return "", [], False
            if depth == 0:
                blocks.append(text[start:idx])
        elif depth == 0 and mask:
            out.append(ch)
    return "".join(out), blocks, depth == 0


def validate_message_syntax(msg_id: str, value: str, source: str = "<memory>") -> list[str]:
    """Valida llaves balanceadas, interpolaciones { $var } y selectores { $var -> ... }."""
    errors: list[str] = []
    _, blocks, balanced = _scan_braces(value, mask=False)
    if not balanced:
        return [f"{source}:{msg_id}: llaves '{{' o '}}' desbalanceadas."]

    for block in blocks:
        masked, nested_blocks, ok = _scan_braces(block, mask=True)
        if not ok:
            errors.append(f"{source}:{msg_id}: llaves anidadas desbalanceadas en {block!r}.")
            continue
        if "->" in masked or "[" in masked or "]" in masked:
            if not SELECTOR_HEADER_PATTERN.match(block) or "->" not in masked:
                errors.append(f"{source}:{msg_id}: sintaxis de selector inválida en {{{block}}}.")
                continue
            _, masked_body = masked.split("->", 1)
            markers = list(VARIANT_MARKER_PATTERN.finditer(masked_body))
            if not markers:
                errors.append(f"{source}:{msg_id}: el selector no contiene variantes [clave].")
                continue
            if masked_body[: markers[0].start()].strip():
                errors.append(f"{source}:{msg_id}: texto inesperado antes de la primera variante.")

            seen_keys: set[str] = set()
            default_keys: list[str] = []
            for idx, marker in enumerate(markers):
                is_default, key = marker.group(1) == "*", marker.group(2)
                if not VARIANT_KEY_PATTERN.match(key) or key in seen_keys:
                    errors.append(f"{source}:{msg_id}: variante inválida o duplicada [{key}].")
                seen_keys.add(key)
                if is_default:
                    default_keys.append(key)
                seg_end = markers[idx + 1].start() if idx + 1 < len(markers) else len(masked_body)
                if not masked_body[marker.end() : seg_end].strip():
                    errors.append(f"{source}:{msg_id}: la variante [{key}] tiene contenido vacío.")
            if default_keys != ["other"]:
                errors.append(f"{source}:{msg_id}: falta variante por defecto única *[other] ({default_keys}).")
            for nested in nested_blocks:
                if not SIMPLE_PLACEABLE_PATTERN.match(nested):
                    errors.append(f"{source}:{msg_id}: interpolación anidada inválida {{{nested}}}.")
        elif not SIMPLE_PLACEABLE_PATTERN.match(block):
            errors.append(f"{source}:{msg_id}: expresión entre llaves inválida {{{block}}}.")
    return errors


def parse_inventory_ids(md_text: str) -> tuple[dict[str, set[str]], list[str]]:
    """Extrae del inventario markdown el conjunto de IDs por archivo .ftl."""
    by_file: dict[str, set[str]] = {name: set() for name in FTL_FILES}
    seen_all: set[str] = set()
    errors: list[str] = []
    for lineno, line in enumerate(md_text.splitlines(), start=1):
        match = INVENTORY_ROW_PATTERN.match(line.strip())
        if not match:
            continue
        msg_id, filename = match.group(1), match.group(2)
        if filename not in by_file:
            errors.append(f"inventario.md:{lineno}: archivo desconocido {filename!r}.")
            continue
        if msg_id in seen_all:
            errors.append(f"inventario.md:{lineno}: ID duplicado {msg_id!r}.")
        seen_all.add(msg_id)
        by_file[filename].add(msg_id)
    return by_file, errors


def parse_vigia_table(md_text: str) -> tuple[dict[str, tuple[str, str, str]], list[str]]:
    """Extrae las filas (Evento, ID, Inglés, Español) de docs/lore/vigia-radio.md."""
    rows: dict[str, tuple[str, str, str]] = {}
    errors: list[str] = []
    for lineno, line in enumerate(md_text.splitlines(), start=1):
        match = VIGIA_ROW_PATTERN.match(line.strip())
        if not match:
            continue
        event, msg_id, en_text, es_text = (g.strip() for g in match.groups())
        if msg_id in rows:
            errors.append(f"vigia-radio.md:{lineno}: ID duplicado {msg_id!r}.")
        rows[msg_id] = (event, en_text, es_text)
    if not rows:
        errors.append("vigia-radio.md: no se encontraron filas en la tabla maestra.")
    return rows, errors


def check_watcher_constraints(
    en_watcher: dict[str, str], vigia_rows: dict[str, tuple[str, str, str]] | None = None
) -> list[str]:
    """Comprueba el tope de 6 palabras en inglés y la correspondencia con vigia-radio.md."""
    errors: list[str] = []
    for msg_id, text in en_watcher.items():
        words = text.split()
        if not (1 <= len(words) <= MAX_WATCHER_WORDS):
            errors.append(f"en/watcher.ftl:{msg_id}: tiene {len(words)} palabras (máximo {MAX_WATCHER_WORDS}).")
    if vigia_rows is not None:
        table_ids, ftl_ids = set(vigia_rows.keys()), set(en_watcher.keys())
        if table_ids != ftl_ids:
            errors.append(f"watcher.ftl vs vigia-radio.md: faltan={sorted(table_ids - ftl_ids)}, sobran={sorted(ftl_ids - table_ids)}.")
        for msg_id, (_, expected_en, _) in vigia_rows.items():
            if msg_id in en_watcher and en_watcher[msg_id] != expected_en:
                errors.append(f"en/watcher.ftl:{msg_id}: {en_watcher[msg_id]!r} != vigia-radio.md {expected_en!r}.")
    return errors


def check_catalogs_symmetry_and_vars(
    catalogs: dict[str, dict[str, dict[str, str]]], inventory_by_file: dict[str, set[str]] | None = None
) -> list[str]:
    """Verifica unicidad entre archivos, paridad de IDs e igualdad de variables."""
    errors: list[str] = []
    for locale, files_map in catalogs.items():
        seen_in_locale: dict[str, str] = {}
        for filename, messages in files_map.items():
            for msg_id in messages:
                if msg_id in seen_in_locale:
                    errors.append(f"{locale}: ID {msg_id!r} duplicado entre {seen_in_locale[msg_id]} y {filename}.")
                else:
                    seen_in_locale[msg_id] = filename

    en_files = catalogs.get("en", {})
    for filename in FTL_FILES:
        en_msgs = en_files.get(filename, {})
        en_ids = set(en_msgs.keys())
        if inventory_by_file is not None and filename in inventory_by_file:
            inv_ids = inventory_by_file[filename]
            if en_ids != inv_ids:
                errors.append(f"inventario.md vs en/{filename}: faltan={sorted(inv_ids - en_ids)}, sobran={sorted(en_ids - inv_ids)}.")
        for locale in LOCALES:
            if locale == "en" or locale not in catalogs:
                continue
            loc_msgs = catalogs[locale].get(filename, {})
            loc_ids = set(loc_msgs.keys())
            if en_ids != loc_ids:
                errors.append(f"{locale}/{filename}: IDs distintos a en/{filename} (faltan={sorted(en_ids - loc_ids)}, sobran={sorted(loc_ids - en_ids)}).")
            for msg_id in sorted(en_ids & loc_ids):
                en_vars, loc_vars = extract_variables(en_msgs[msg_id]), extract_variables(loc_msgs[msg_id])
                if en_vars != loc_vars:
                    errors.append(f"{locale}/{filename}:{msg_id}: variables {sorted(loc_vars)} != en {sorted(en_vars)}.")
    return errors


def check_spanish_glossary_and_variants(catalogs: dict[str, dict[str, dict[str, str]]]) -> list[str]:
    """Comprueba términos prohibidos, glosario alineado, bloques, títulos y variantes regionales."""
    errors: list[str] = []
    en_files = catalogs.get("en", {})
    for locale in ("es-419", "es-ES"):
        if locale not in catalogs:
            continue
        loc_files = catalogs[locale]
        for filename, messages in loc_files.items():
            en_msgs = en_files.get(filename, {})
            for msg_id, text in messages.items():
                lower_text = text.lower()
                for forbidden in FORBIDDEN_SPANISH_TERMS:
                    hit = bool(FORBIDDEN_DORMIDO_PATTERN.search(text)) if forbidden == "Dormido" else forbidden.lower() in lower_text
                    if hit:
                        errors.append(f"{locale}/{filename}:{msg_id}: contiene el término prohibido {forbidden!r}.")
                en_text = en_msgs.get(msg_id, "")
                for en_term, es_term, case_sensitive in GLOSSARY_ALIGNMENT:
                    if en_term in en_text:
                        haystack = text if case_sensitive else lower_text
                        needle = es_term if case_sensitive else es_term.lower()
                        if needle not in haystack:
                            errors.append(f"{locale}/{filename}:{msg_id}: falta el término canónico {es_term!r} para {en_term!r}.")

        game_msgs = loc_files.get("game.ftl", {})
        for block_id in EXPECTED_BLOCKS:
            if not game_msgs.get(block_id, "").strip():
                errors.append(f"{locale}/game.ftl: falta el bloque obligatorio {block_id!r}.")
        chron_msgs = loc_files.get("chronicle.ftl", {})
        for title_id, expected_title in CHRONICLE_TITLES.items():
            if expected_title.lower() not in chron_msgs.get(title_id, "").lower():
                errors.append(f"{locale}/chronicle.ftl:{title_id}: falta el título {expected_title!r}.")

    if "es-419" in catalogs and "es-ES" in catalogs:
        diffs = sum(
            1
            for fname in FTL_FILES
            for k in set(catalogs["es-419"].get(fname, {})) & set(catalogs["es-ES"].get(fname, {}))
            if catalogs["es-419"][fname][k] != catalogs["es-ES"][fname][k]
        )
        if diffs < MIN_REGIONAL_DIFFS:
            errors.append(f"Variantes regionales insuficientes entre es-419 y es-ES: {diffs} diferencias (mínimo {MIN_REGIONAL_DIFFS}).")
    return errors


def check_file_limits_and_lf(root: Path = REPO_ROOT) -> list[str]:
    """Verifica que ningún archivo en assets/i18n, docs/i18n o tools/i18n supere 400 líneas ni use CRLF."""
    errors: list[str] = []
    for sub in ("assets/i18n", "docs/i18n", "tools/i18n"):
        base_dir = root / sub
        if not base_dir.exists():
            continue
        for path in sorted(base_dir.rglob("*")):
            if not path.is_file() or path.suffix not in {".ftl", ".md", ".py"}:
                continue
            rel, raw = path.relative_to(root).as_posix(), path.read_bytes()
            if b"\r\n" in raw:
                errors.append(f"{rel}: contiene saltos de línea CRLF (\\r\\n).")
            line_count = len(raw.decode("utf-8", errors="replace").splitlines())
            if line_count > MAX_LINES:
                errors.append(f"{rel}: tiene {line_count} líneas (máximo permitido: {MAX_LINES}).")
    return errors


def check_all(root: Path = REPO_ROOT) -> list[str]:
    """Ejecuta todas las validaciones estructurales, sintácticas y de glosario."""
    errors: list[str] = []
    for rel_req in ("docs/i18n/inventario.md", "docs/i18n/guia-estilo.md", "docs/lore/vigia-radio.md", "tools/i18n/check_ftl.py", "tools/i18n/test_i18n.py"):
        if not (root / rel_req).is_file():
            errors.append(f"Falta el archivo obligatorio: {rel_req}")

    catalogs: dict[str, dict[str, dict[str, str]]] = {}
    for locale in LOCALES:
        loc_dir = root / "assets" / "i18n" / locale
        if not loc_dir.is_dir():
            errors.append(f"Falta el directorio de idioma: assets/i18n/{locale}")
            continue
        actual_ftl = {p.name for p in loc_dir.glob("*.ftl")}
        if actual_ftl != set(FTL_FILES):
            errors.append(f"assets/i18n/{locale}: archivos .ftl esperados {FTL_FILES}, encontrados {sorted(actual_ftl)}.")
        catalogs[locale] = {}
        for filename in FTL_FILES:
            fpath = loc_dir / filename
            if not fpath.is_file():
                continue
            rel = fpath.relative_to(root).as_posix()
            msgs, parse_errs = parse_ftl_text(fpath.read_text(encoding="utf-8"), source=rel)
            errors.extend(parse_errs)
            errors.extend(validate_id_naming(msgs, filename=filename, source=rel))
            for msg_id, val in msgs.items():
                errors.extend(validate_message_syntax(msg_id, val, source=rel))
            catalogs[locale][filename] = msgs

    errors.extend(check_file_limits_and_lf(root))
    inv_path, vigia_path = root / "docs" / "i18n" / "inventario.md", root / "docs" / "lore" / "vigia-radio.md"
    inv_by_file, vigia_rows = None, None
    if inv_path.is_file():
        inv_by_file, inv_errs = parse_inventory_ids(inv_path.read_text(encoding="utf-8"))
        errors.extend(inv_errs)
    if vigia_path.is_file():
        vigia_rows, vigia_errs = parse_vigia_table(vigia_path.read_text(encoding="utf-8"))
        errors.extend(vigia_errs)
    if len(catalogs) == len(LOCALES):
        errors.extend(check_catalogs_symmetry_and_vars(catalogs, inv_by_file))
        errors.extend(check_watcher_constraints(catalogs["en"].get("watcher.ftl", {}), vigia_rows))
        errors.extend(check_spanish_glossary_and_variants(catalogs))
    return errors


def main() -> int:
    """Punto de entrada CLI del verificador Fluent."""
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    errors = check_all(REPO_ROOT)
    if errors:
        print(f"ERROR: Se encontraron {len(errors)} problemas de localización:", file=sys.stderr)
        for err in errors:
            print(f"  - {err}", file=sys.stderr)
        return 1
    print("OK: Catálogos Fluent (.ftl), inventario, Vigía y glosario verificados sin errores.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
