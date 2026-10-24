#!/usr/bin/env python3
"""Verificador automático de la documentación de historia y lore de HUSHMINE.

Se ejecuta desde la raíz del repositorio:
    python tools/lore/check_lore.py
"""

from __future__ import annotations

from collections import Counter
from pathlib import Path
import re
import sys
from typing import NamedTuple

MAX_LINES = 400
REQUIRED_LORE_FILES = (
    "README.md",
    "cronologia.md",
    "los-41.md",
    "vigia-radio.md",
    "anima-consolidated.md",
    "folclore.md",
)
EXPECTED_MINER_COUNT = 41
EXPECTED_COUNTRY_COUNT = 12
MIN_RADIO_EVENTS = 10
MIN_EVENT_VARIANTS = 4
MAX_EVENT_VARIANTS = 6
MAX_ENGLISH_WORDS = 6
REQUIRED_CHRONOLOGY_YEARS = ("1931", "1979")


class TableRow(NamedTuple):
    """Fila de datos dentro de una tabla Markdown."""

    line_number: int
    cells: list[str]


class MarkdownTable(NamedTuple):
    """Tabla Markdown con su cabecera y filas de datos."""

    header_line: int
    headers: list[str]
    rows: list[TableRow]


def clean_cell(raw_value: str) -> str:
    """Limpia espacios, formato Markdown básico y comillas o backticks."""
    text = raw_value.strip()
    text = re.sub(r"^\*{1,2}|\*{1,2}$", "", text).strip()
    text = text.strip("`\"'“”‘’«»").strip()
    return re.sub(r"^\*{1,2}|\*{1,2}$", "", text).strip()


def is_separator_row(cells: list[str]) -> bool:
    """Indica si una fila de tabla Markdown es el separador |---|---|."""
    return bool(cells) and all(bool(re.match(r"^:?-{2,}:?$", c.strip())) for c in cells)


def parse_markdown_tables(text: str) -> list[MarkdownTable]:
    """Extrae todas las tablas Markdown del texto conservando números de línea."""
    tables: list[MarkdownTable] = []
    block: list[TableRow] = []

    def flush(current: list[TableRow]) -> None:
        if len(current) >= 2 and is_separator_row(current[1].cells):
            headers = [clean_cell(c) for c in current[0].cells]
            rows = [r for r in current[2:] if not is_separator_row(r.cells)]
            tables.append(MarkdownTable(current[0].line_number, headers, rows))

    for line_num, raw_line in enumerate(text.splitlines(), start=1):
        stripped = raw_line.strip()
        if stripped.startswith("|") and stripped.endswith("|"):
            cells = [c.strip() for c in stripped[1:-1].split("|")]
            block.append(TableRow(line_num, cells))
        elif block:
            flush(block)
            block = []
    if block:
        flush(block)
    return tables


def check_files_and_line_limits(repo_root: Path, script_path: Path) -> list[str]:
    """Verifica la existencia de archivos en docs/lore/ y el límite de 400 líneas."""
    errors: list[str] = []
    lore_dir = repo_root / "docs" / "lore"
    if not lore_dir.is_dir():
        return ["No existe el directorio 'docs/lore/'."]

    for filename in REQUIRED_LORE_FILES:
        if not (lore_dir / filename).is_file():
            errors.append(f"Falta el archivo obligatorio 'docs/lore/{filename}'.")

    files_to_check = sorted(lore_dir.rglob("*.md"))
    if script_path.is_file():
        files_to_check.append(script_path)

    for file_path in files_to_check:
        rel_path = file_path.relative_to(repo_root).as_posix()
        line_count = len(file_path.read_text(encoding="utf-8").splitlines())
        if line_count > MAX_LINES:
            errors.append(
                f"'{rel_path}' tiene {line_count} líneas (máximo permitido: {MAX_LINES})."
            )
    return errors


def check_the_41(file_path: Path) -> list[str]:
    """Verifica las 41 fichas y los 12 países en docs/lore/los-41.md."""
    if not file_path.is_file():
        return []
    errors: list[str] = []
    tables = parse_markdown_tables(file_path.read_text(encoding="utf-8"))

    miner_rows: list[tuple[int, list[str], int]] = []
    for table in tables:
        if len(table.headers) < 5:
            continue
        lower_headers = [h.lower() for h in table.headers]
        has_miner_headers = any(
            "ficha" in h or h in ("#", "n.º", "nº", "id") for h in lower_headers
        ) and any("país" in h or "pais" in h for h in lower_headers)
        country_idx = next(
            (i for i, h in enumerate(lower_headers) if "país" in h or "pais" in h), 2
        )
        for row in table.rows:
            first_token = (
                re.sub(r"^[#*`\s]+|[#*`\s]+$", "", clean_cell(row.cells[0]))
                if row.cells
                else ""
            )
            if has_miner_headers or first_token.isdigit():
                miner_rows.append((row.line_number, row.cells, country_idx))

    if not miner_rows:
        return ["los-41.md: no se encontró la tabla de los 41 mineros."]

    tally_numbers: list[int] = []
    countries: set[str] = set()

    for line_num, cells, country_idx in miner_rows:
        if len(cells) < 6:
            errors.append(
                f"los-41.md (línea {line_num}): se esperaban 6 columnas y hay {len(cells)}."
            )
            continue
        cleaned = [clean_cell(c) for c in cells]
        for col_idx, val in enumerate(cleaned, start=1):
            if not val:
                errors.append(f"los-41.md (línea {line_num}): columna {col_idx} vacía.")
        raw_tally = re.sub(r"^[#*`\s]+|[#*`\s]+$", "", cleaned[0])
        try:
            tally_numbers.append(int(raw_tally))
        except ValueError:
            errors.append(
                f"los-41.md (línea {line_num}): ficha inválida '{cleaned[0]}'."
            )
        if cleaned[country_idx]:
            countries.add(cleaned[country_idx])

    expected_numbers = list(range(1, EXPECTED_MINER_COUNT + 1))
    if len(tally_numbers) != EXPECTED_MINER_COUNT:
        errors.append(
            f"los-41.md: hay {len(tally_numbers)} fichas (se esperaban {EXPECTED_MINER_COUNT})."
        )
    if sorted(tally_numbers) != expected_numbers:
        missing = sorted(set(expected_numbers) - set(tally_numbers))
        dups = sorted(n for n, c in Counter(tally_numbers).items() if c > 1)
        if missing:
            errors.append(f"los-41.md: faltan las fichas {missing}.")
        if dups:
            errors.append(f"los-41.md: fichas repetidas {dups}.")
    if len(countries) != EXPECTED_COUNTRY_COUNT:
        errors.append(
            f"los-41.md: hay {len(countries)} países distintos "
            f"(se esperaban {EXPECTED_COUNTRY_COUNT}): {sorted(countries)}."
        )
    return errors


def check_watcher_radio(file_path: Path) -> list[str]:
    """Verifica las frases del Vigía por radio en docs/lore/vigia-radio.md."""
    if not file_path.is_file():
        return []
    errors: list[str] = []
    tables = parse_markdown_tables(file_path.read_text(encoding="utf-8"))

    radio_rows: list[TableRow] = []
    for table in tables:
        lower = [h.lower() for h in table.headers]
        if len(lower) >= 4 and "evento" in lower[0] and "id" in lower[1]:
            radio_rows.extend(table.rows)

    if not radio_rows:
        return ["vigia-radio.md: no se encontró la tabla 'Evento | ID | Inglés | Español'."]

    seen_ids: dict[str, int] = {}
    event_counts: Counter[str] = Counter()

    for row in radio_rows:
        if len(row.cells) < 4:
            errors.append(f"vigia-radio.md (línea {row.line_number}): menos de 4 columnas.")
            continue
        event_name, phrase_id, english_text, spanish_text = (
            clean_cell(row.cells[0]),
            clean_cell(row.cells[1]),
            clean_cell(row.cells[2]),
            clean_cell(row.cells[3]),
        )
        if not all((event_name, phrase_id, english_text, spanish_text)):
            errors.append(f"vigia-radio.md (línea {row.line_number}): hay celdas vacías.")
            continue
        if phrase_id in seen_ids:
            errors.append(
                f"vigia-radio.md (línea {row.line_number}): ID '{phrase_id}' repetido "
                f"(visto en línea {seen_ids[phrase_id]})."
            )
        else:
            seen_ids[phrase_id] = row.line_number

        words = english_text.split()
        if not (1 <= len(words) <= MAX_ENGLISH_WORDS):
            errors.append(
                f"vigia-radio.md (línea {row.line_number}): '{english_text}' tiene "
                f"{len(words)} palabras (máximo {MAX_ENGLISH_WORDS})."
            )
        event_counts[event_name] += 1

    if len(event_counts) < MIN_RADIO_EVENTS:
        errors.append(
            f"vigia-radio.md: hay {len(event_counts)} eventos (mínimo {MIN_RADIO_EVENTS})."
        )
    for event_name, count in event_counts.items():
        if not (MIN_EVENT_VARIANTS <= count <= MAX_EVENT_VARIANTS):
            errors.append(
                f"vigia-radio.md: el evento '{event_name}' tiene {count} variantes "
                f"(rango permitido: {MIN_EVENT_VARIANTS}..{MAX_EVENT_VARIANTS})."
            )
    return errors


def check_chronology(file_path: Path) -> list[str]:
    """Verifica la presencia de los años clave en docs/lore/cronologia.md."""
    if not file_path.is_file():
        return []
    text = file_path.read_text(encoding="utf-8")
    return [
        f"cronologia.md: no aparece el año obligatorio '{year}'."
        for year in REQUIRED_CHRONOLOGY_YEARS
        if not re.search(rf"\b{re.escape(year)}\b", text)
    ]


def main() -> int:
    """Punto de entrada principal del verificador de lore."""
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8")
    script_path = Path(__file__).resolve()
    repo_root = script_path.parents[2]
    lore_dir = repo_root / "docs" / "lore"

    errors = (
        check_files_and_line_limits(repo_root, script_path)
        + check_the_41(lore_dir / "los-41.md")
        + check_watcher_radio(lore_dir / "vigia-radio.md")
        + check_chronology(lore_dir / "cronologia.md")
    )
    if errors:
        print("Verificación de lore FALLIDA:")
        for err in errors:
            print(f"  - {err}")
        return 1

    print(
        "Resumen de comprobaciones:\n"
        f"  - Archivos en docs/lore/: {len(REQUIRED_LORE_FILES)} presentes ( todos <= {MAX_LINES} líneas )\n"
        f"  - los-41.md: {EXPECTED_MINER_COUNT} fichas (1..41) en {EXPECTED_COUNTRY_COUNT} países distintos\n"
        f"  - vigia-radio.md: >= {MIN_RADIO_EVENTS} eventos, {MIN_EVENT_VARIANTS}..{MAX_EVENT_VARIANTS} variantes/evento, inglés <= {MAX_ENGLISH_WORDS} palabras\n"
        f"  - cronologia.md: años {', '.join(REQUIRED_CHRONOLOGY_YEARS)} verificados\n"
        "Verificación de lore completada con éxito (0 errores)."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
