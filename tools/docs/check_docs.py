#!/usr/bin/env python3
"""Verificador documental del Encargo 5 para HUSHMINE / Grisú.

Comprueba la existencia, longitud máxima, rutas en línea, identificadores de
código, enlaces relativos, diagramas Mermaid y restricciones específicas de los
documentos en `docs/engine/`, `docs/hitos/`, `docs/marketing/` y `docs/README.md`.
"""

from __future__ import annotations

import pathlib
import re
import sys

MAX_LINES = 400
MAX_SHORT_DESC_CHARS = 300
MIN_HITO_TASKS = 12
MAX_HITO_TASKS = 20

EXPECTED_FILES = (
    "docs/engine/grisu-render.md",
    "docs/engine/grisu-platform.md",
    "docs/engine/hush-sim.md",
    "docs/engine/hush-cli.md",
    "docs/engine/hush-client.md",
    "docs/hitos/hito-1-voxel.md",
    "docs/marketing/steam-page.md",
    "docs/marketing/press-kit.md",
    "docs/marketing/devlog-01.md",
    "docs/README.md",
    "tools/docs/check_docs.py",
)

ENGINE_DOCS = EXPECTED_FILES[:5]
TARGET_DOC_DIRS = ("docs/engine", "docs/hitos", "docs/marketing")

IDENT_TOKEN_RE = re.compile(r"\b[A-Za-z_][A-Za-z0-9_]*\b")
INLINE_CODE_RE = re.compile(r"`([^`\n]+)`")
MD_LINK_RE = re.compile(r"(?<!!)\[([^\]]+)\]\(([^)]+)\)")
FUNC_CALL_RE = re.compile(r"^(?:[A-Za-z_][A-Za-z0-9_]*::)*([a-z_][a-z0-9_]*)\(\)$")
PUBLIC_TYPE_RE = re.compile(r"^(?:[A-Za-z_][A-Za-z0-9_]*::)*([A-Z][A-Za-z0-9_]*)$")
TASK_HEADING_RE = re.compile(r"^#{2,4}\s+Tarea\s+\d+\b", re.IGNORECASE | re.MULTILINE)
COMMENT_RE = re.compile(r"//.*|/\*[\s\S]*?\*/")


def collect_crate_identifiers(repo_root: pathlib.Path) -> set[str]:
    """Extrae todos los identificadores léxicos de archivos `.rs` y `.wgsl` en `crates/`."""
    crates_dir = repo_root / "crates"
    identifiers: set[str] = set()
    if not crates_dir.is_dir():
        return identifiers
    for source_path in crates_dir.rglob("*"):
        if source_path.is_file() and source_path.suffix in (".rs", ".wgsl"):
            raw = source_path.read_text(encoding="utf-8")
            uncommented = COMMENT_RE.sub("", raw)
            identifiers.update(IDENT_TOKEN_RE.findall(uncommented))
    return identifiers


def collect_scope_files(repo_root: pathlib.Path) -> tuple[list[pathlib.Path], list[pathlib.Path]]:
    """Devuelve `(archivos_md, todos_los_archivos_del_alcance)` para el encargo."""
    md_files: list[pathlib.Path] = []
    for rel_dir in TARGET_DOC_DIRS:
        folder = repo_root / rel_dir
        if folder.is_dir():
            md_files.extend(sorted(folder.rglob("*.md")))

    readme_path = repo_root / "docs/README.md"
    if readme_path.is_file():
        md_files.append(readme_path)

    all_scope_files = list(md_files)
    tools_dir = repo_root / "tools/docs"
    if tools_dir.is_dir():
        for item in sorted(tools_dir.rglob("*")):
            if item.is_file() and "__pycache__" not in item.parts and item.suffix in (".py", ".md"):
                all_scope_files.append(item)
    return md_files, all_scope_files


def iter_non_fenced_lines(text: str) -> list[tuple[int, str]]:
    """Devuelve pares `(numero_linea, contenido)` omitiendo bloques cercados con ```."""
    result: list[tuple[int, str]] = []
    in_fence = False
    for idx, line in enumerate(text.splitlines(), start=1):
        if line.strip().startswith("```"):
            in_fence = not in_fence
            continue
        if not in_fence:
            result.append((idx, line))
    return result


def has_closed_mermaid_block(text: str) -> bool:
    """Comprueba que exista al menos un bloque ```mermaid cerrado correctamente con ```."""
    in_mermaid = False
    mermaid_lines = 0
    for line in text.splitlines():
        stripped = line.strip()
        if not in_mermaid:
            if stripped.startswith("```mermaid"):
                in_mermaid = True
                mermaid_lines = 0
        elif stripped == "```":
            if mermaid_lines > 0:
                return True
            in_mermaid = False
        elif stripped:
            mermaid_lines += 1
    return False


def check_expected_files(repo_root: pathlib.Path, errors: list[str]) -> None:
    """1. Verifica que todos los archivos requeridos por el Encargo 5 existan."""
    for rel_path in EXPECTED_FILES:
        if not (repo_root / rel_path).is_file():
            errors.append(f"[Existencia] Falta el archivo requerido: {rel_path}")


def check_line_limits(
    repo_root: pathlib.Path,
    scope_files: list[pathlib.Path],
    errors: list[str],
) -> None:
    """2. Verifica que ningún archivo del alcance supere las 400 líneas."""
    for file_path in scope_files:
        rel = file_path.relative_to(repo_root).as_posix()
        lines = file_path.read_text(encoding="utf-8").splitlines()
        if len(lines) > MAX_LINES:
            errors.append(
                f"[Longitud] {rel} tiene {len(lines)} líneas (máximo permitido: {MAX_LINES})."
            )


def check_inline_paths_and_identifiers(
    repo_root: pathlib.Path,
    md_files: list[pathlib.Path],
    crate_identifiers: set[str],
    errors: list[str],
) -> None:
    """3 y 4. Verifica rutas entre comillas invertidas e identificadores de `crates/`."""
    for md_path in md_files:
        rel_md = md_path.relative_to(repo_root).as_posix()
        text = md_path.read_text(encoding="utf-8")
        for line_no, line in iter_non_fenced_lines(text):
            for match in INLINE_CODE_RE.finditer(line):
                token = match.group(1).strip()
                if not token or " " in token:
                    continue

                if token.startswith(("crates/", "docs/", "tools/")):
                    cleaned = token.split("#", 1)[0].rstrip(".,;:)")
                    if not cleaned:
                        continue
                    target = repo_root / cleaned
                    exists = target.exists() or (
                        "*" in cleaned and bool(list(repo_root.glob(cleaned)))
                    )
                    if not exists:
                        errors.append(
                            f"[Ruta en línea] {rel_md}:{line_no}: la ruta `{token}` no existe."
                        )
                    continue

                if FUNC_CALL_RE.match(token):
                    parts = token[:-2].split("::")
                    missing = [part for part in parts if part not in crate_identifiers]
                    if missing:
                        errors.append(
                            f"[Identificador] {rel_md}:{line_no}: la función `{token}` "
                            f"no se encontró en `crates/` (faltan: {', '.join(missing)})."
                        )
                    continue

                type_match = PUBLIC_TYPE_RE.match(token)
                if type_match and any(ch.islower() for ch in type_match.group(1)):
                    parts = token.split("::")
                    missing = [part for part in parts if part not in crate_identifiers]
                    if missing:
                        errors.append(
                            f"[Identificador] {rel_md}:{line_no}: el tipo `{token}` "
                            f"no se encontró en `crates/` (faltan: {', '.join(missing)})."
                        )


def check_markdown_links(
    repo_root: pathlib.Path,
    md_files: list[pathlib.Path],
    errors: list[str],
) -> None:
    """5. Verifica que todos los enlaces relativos de Markdown apunten a archivos existentes."""
    for md_path in md_files:
        rel_md = md_path.relative_to(repo_root).as_posix()
        text = md_path.read_text(encoding="utf-8")
        for line_no, line in iter_non_fenced_lines(text):
            if "(pendiente)" in line.lower():
                continue
            line_without_code = INLINE_CODE_RE.sub("", line)
            for match in MD_LINK_RE.finditer(line_without_code):
                label, raw_target = match.group(1).strip(), match.group(2).strip()
                if "(pendiente)" in label.lower() or "(pendiente)" in raw_target.lower():
                    continue
                target_url = raw_target.split()[0] if raw_target else ""
                if not target_url or target_url.startswith(("#", "http://", "https://", "mailto:")):
                    continue
                path_part = target_url.split("#", 1)[0]
                if path_part and not (md_path.parent / path_part).resolve().exists():
                    errors.append(
                        f"[Enlace Markdown] {rel_md}:{line_no}: el destino `{raw_target}` no existe."
                    )


def check_engine_mermaid_blocks(repo_root: pathlib.Path, errors: list[str]) -> None:
    """6. Verifica que cada documento de `docs/engine/` contenga al menos un bloque Mermaid."""
    for rel_path in ENGINE_DOCS:
        full_path = repo_root / rel_path
        if full_path.is_file() and not has_closed_mermaid_block(full_path.read_text(encoding="utf-8")):
            errors.append(
                f"[Mermaid] {rel_path}: no contiene ningún bloque ```mermaid cerrado correctamente."
            )


def extract_short_descriptions(steam_text: str) -> list[str]:
    """Extrae los párrafos de descripción corta en `docs/marketing/steam-page.md`."""
    in_section = False
    section_level = 2
    collected: list[str] = []

    for line in steam_text.splitlines():
        stripped = line.strip()
        heading_match = re.match(r"^(#{1,6})\s+(.*)$", stripped)
        if heading_match:
            level = len(heading_match.group(1))
            title = heading_match.group(2).lower()
            if not in_section:
                if any(k in title for k in ("descripción corta", "descripcion corta", "short description")):
                    in_section = True
                    section_level = level
                continue
            if level <= section_level:
                break
            continue

        if not in_section or not stripped or stripped.startswith(("---", "```")):
            continue

        cleaned = re.sub(r"^[>\-\*]+\s*", "", stripped)
        cleaned = re.sub(
            r"^\*{0,2}(?:ES|EN|Español|English|Spanish)\*{0,2}\s*:\s*\*{0,2}\s*",
            "",
            cleaned,
            flags=re.IGNORECASE,
        ).strip()
        if not cleaned or re.match(r"^\*?\(?(?:longitud|\d+\s*caracteres)\b", cleaned, re.IGNORECASE):
            continue
        collected.append(cleaned)

    return collected


def check_steam_short_description(repo_root: pathlib.Path, errors: list[str]) -> None:
    """7. Verifica que la descripción corta de `steam-page.md` tenga <= 300 caracteres."""
    steam_path = repo_root / "docs/marketing/steam-page.md"
    if not steam_path.is_file():
        return
    descriptions = extract_short_descriptions(steam_path.read_text(encoding="utf-8"))
    if not descriptions:
        errors.append(
            "[Steam] docs/marketing/steam-page.md: no se encontró ninguna descripción corta "
            "bajo el encabezado 'Descripción corta' / 'Short Description'."
        )
        return
    for idx, desc in enumerate(descriptions, start=1):
        if len(desc) > MAX_SHORT_DESC_CHARS:
            errors.append(
                f"[Steam] docs/marketing/steam-page.md: la descripción corta #{idx} tiene "
                f"{len(desc)} caracteres (máximo permitido: {MAX_SHORT_DESC_CHARS})."
            )


def check_hito_tasks(repo_root: pathlib.Path, errors: list[str]) -> None:
    """8. Verifica que `hito-1-voxel.md` tenga entre 12 y 20 tareas y un bloque Mermaid."""
    hito_path = repo_root / "docs/hitos/hito-1-voxel.md"
    if not hito_path.is_file():
        return
    text = hito_path.read_text(encoding="utf-8")
    task_count = len(TASK_HEADING_RE.findall(text))
    if not (MIN_HITO_TASKS <= task_count <= MAX_HITO_TASKS):
        errors.append(
            f"[Hito 1] docs/hitos/hito-1-voxel.md: se encontraron {task_count} tareas "
            f"(se requieren entre {MIN_HITO_TASKS} y {MAX_HITO_TASKS})."
        )
    if not has_closed_mermaid_block(text):
        errors.append(
            "[Hito 1] docs/hitos/hito-1-voxel.md: falta el bloque ```mermaid del grafo de dependencias."
        )


def main() -> int:
    """Punto de entrada principal del verificador documental."""
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")

    repo_root = pathlib.Path(__file__).resolve().parents[2]
    errors: list[str] = []

    check_expected_files(repo_root, errors)
    md_files, scope_files = collect_scope_files(repo_root)
    crate_identifiers = collect_crate_identifiers(repo_root)

    check_line_limits(repo_root, scope_files, errors)
    check_inline_paths_and_identifiers(repo_root, md_files, crate_identifiers, errors)
    check_markdown_links(repo_root, md_files, errors)
    check_engine_mermaid_blocks(repo_root, errors)
    check_steam_short_description(repo_root, errors)
    check_hito_tasks(repo_root, errors)

    if errors:
        print(f"ERROR: Se encontraron {len(errors)} problemas en la documentación del Encargo 5:")
        for err in errors:
            print(f"  - {err}")
        return 1

    print(
        f"OK: Verificación documental completada sin errores "
        f"({len(scope_files)} archivos revisados, {len(crate_identifiers)} identificadores indexados)."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
