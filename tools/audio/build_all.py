"""Integrador y CLI de construcción determinista del catálogo de audio de HUSHMINE.

Genera y verifica de forma reproducible los 30 efectos de sonido WAV PCM de
16 bits a 44 100 Hz mono (`assets/audio/sfx/*.wav`) y el manifiesto ordenado
(`assets/audio/manifest.ron`) usando únicamente la biblioteca estándar de Python.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT: Path = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.audio import sfx_radio, sfx_tools, sfx_world, synth  # noqa: E402

AUDIO_DIR: Path = REPO_ROOT / "assets" / "audio"
SFX_DIR: Path = AUDIO_DIR / "sfx"
MANIFEST_PATH: Path = AUDIO_DIR / "manifest.ron"
LICENSE_PATH: Path = AUDIO_DIR / "LICENSE.md"
MAX_AUDIO_DIR_BYTES: int = 8 * 1024 * 1024

_MANIFEST_HEADER: str = (
    "// Manifiesto de efectos de sonido de HUSHMINE (WAV PCM 16 bits, 44100 Hz, mono).\n"
    "// Generado por tools/audio/build_all.py — no editar a mano.\n"
)

_MANIFEST_ENTRY_RE: re.Pattern[str] = re.compile(
    r"^\(\s*"
    r'id\s*:\s*"([^"]+)"\s*,\s*'
    r'file\s*:\s*"([^"]+)"\s*,\s*'
    r"samples\s*:\s*(\d+)\s*,\s*"
    r"looping\s*:\s*(true|false)\s*,\s*"
    r'category\s*:\s*"([^"]+)"\s*'
    r"\)\s*,?$"
)


def generate_all_sfx() -> dict[str, list[float]]:
    """Combina y valida los 30 efectos de sonido en el orden de `synth.SOUND_SPECS`."""
    tools_sfx = sfx_tools.generate_tools_sfx()
    world_sfx = sfx_world.generate_world_sfx()
    radio_sfx = sfx_radio.generate_radio_sfx()

    keys_tools = set(tools_sfx.keys())
    keys_world = set(world_sfx.keys())
    keys_radio = set(radio_sfx.keys())

    overlap = (
        (keys_tools & keys_world)
        | (keys_tools & keys_radio)
        | (keys_world & keys_radio)
    )
    if overlap:
        raise ValueError(
            f"Claves solapadas entre generadores de audio: {sorted(overlap)}"
        )

    combined: dict[str, list[float]] = {**tools_sfx, **world_sfx, **radio_sfx}
    expected_ids = [str(spec["id"]) for spec in synth.SOUND_SPECS]
    if len(expected_ids) != 30:
        raise ValueError(
            f"Se esperaban exactamente 30 sonidos en SOUND_SPECS, encontrados: {len(expected_ids)}"
        )

    expected_set = set(expected_ids)
    missing = [snd_id for snd_id in expected_ids if snd_id not in combined]
    extra = [snd_id for snd_id in combined if snd_id not in expected_set]
    if missing or extra:
        raise ValueError(
            f"Desajuste en el conjunto de sonidos: faltan={missing}, sobran={extra}"
        )

    ordered: dict[str, list[float]] = {}
    for spec in synth.SOUND_SPECS:
        snd_id = str(spec["id"])
        expected_samples = int(spec["samples"])  # type: ignore[arg-type]
        samples = combined[snd_id]
        if not isinstance(samples, list) or len(samples) != expected_samples:
            raise ValueError(
                f"El sonido {snd_id!r} debe tener {expected_samples} muestras, "
                f"encontradas: {len(samples)}"
            )
        ordered[snd_id] = samples

    return ordered


def format_manifest_ron(sound_specs: list[dict[str, object]] | None = None) -> str:
    """Formatea el contenido RON determinista de `assets/audio/manifest.ron`."""
    specs = synth.SOUND_SPECS if sound_specs is None else sound_specs
    lines: list[str] = [_MANIFEST_HEADER.rstrip("\n"), "["]
    for spec in specs:
        snd_id = str(spec["id"])
        rel_file = str(spec["file"])
        samples = int(spec["samples"])  # type: ignore[arg-type]
        looping_ron = "true" if bool(spec["looping"]) else "false"
        category = str(spec["category"])
        lines.append(
            f'    (id: "{snd_id}", file: "{rel_file}", samples: {samples}, '
            f'looping: {looping_ron}, category: "{category}"),'
        )
    lines.append("]\n")
    return "\n".join(lines)


def parse_manifest_ron(content: str) -> list[dict[str, object]]:
    """Analiza `assets/audio/manifest.ron` y devuelve la lista de diccionarios de sonidos."""
    active_lines: list[str] = []
    for raw_line in content.splitlines():
        code_part = raw_line.split("//", 1)[0].strip()
        if code_part:
            active_lines.append(code_part)

    if len(active_lines) < 2 or active_lines[0] != "[" or active_lines[-1] != "]":
        raise ValueError(
            "Formato RON inválido en manifest.ron: se esperaba una lista delimitada por '[' y ']'"
        )

    entries: list[dict[str, object]] = []
    for line in active_lines[1:-1]:
        match = _MANIFEST_ENTRY_RE.match(line)
        if match is None:
            raise ValueError(f"Entrada inválida en manifest.ron: {line!r}")
        snd_id, rel_file, samples_str, looping_str, category = match.groups()
        entries.append(
            {
                "id": snd_id,
                "file": rel_file,
                "samples": int(samples_str),
                "looping": looping_str == "true",
                "category": category,
            }
        )
    return entries


def build_all() -> None:
    """Genera y escribe los 30 archivos WAV en `assets/audio/sfx/` y `manifest.ron`."""
    all_sfx = generate_all_sfx()
    SFX_DIR.mkdir(parents=True, exist_ok=True)

    for spec in synth.SOUND_SPECS:
        snd_id = str(spec["id"])
        rel_file = str(spec["file"])
        synth.write_wav(AUDIO_DIR / rel_file, all_sfx[snd_id])

    manifest_text = format_manifest_ron(synth.SOUND_SPECS)
    MANIFEST_PATH.write_bytes(manifest_text.encode("utf-8"))


def check_all() -> tuple[bool, list[str]]:
    """Verifica que los 30 WAV, `manifest.ron` y `LICENSE.md` sean íntegros y reproducibles."""
    errors: list[str] = []
    all_sfx = generate_all_sfx()
    expected_wav_names = {Path(str(spec["file"])).name for spec in synth.SOUND_SPECS}

    if not SFX_DIR.is_dir():
        errors.append(f"No existe el directorio de efectos de sonido: {SFX_DIR}")
    else:
        existing_wav_names = {p.name for p in SFX_DIR.glob("*.wav") if p.is_file()}
        missing_wavs = sorted(expected_wav_names - existing_wav_names)
        extra_wavs = sorted(existing_wav_names - expected_wav_names)

        for name in missing_wavs:
            errors.append(f"Falta el archivo WAV esperado: {SFX_DIR / name}")
        for name in extra_wavs:
            errors.append(f"Archivo WAV inesperado en directorio sfx: {SFX_DIR / name}")

        for spec in synth.SOUND_SPECS:
            snd_id = str(spec["id"])
            wav_path = AUDIO_DIR / str(spec["file"])
            if not wav_path.is_file():
                continue
            expected_bytes = synth.encode_wav_bytes(all_sfx[snd_id])
            actual_bytes = wav_path.read_bytes()
            if actual_bytes != expected_bytes:
                errors.append(
                    f"El archivo {wav_path} no coincide byte a byte con el WAV generado en memoria"
                )

    expected_manifest_bytes = format_manifest_ron(synth.SOUND_SPECS).encode("utf-8")
    if not MANIFEST_PATH.is_file():
        errors.append(f"Falta el archivo de manifiesto: {MANIFEST_PATH}")
    else:
        actual_manifest_bytes = MANIFEST_PATH.read_bytes()
        if actual_manifest_bytes != expected_manifest_bytes:
            errors.append(
                f"El archivo {MANIFEST_PATH} no coincide byte a byte con el manifiesto esperado"
            )
        else:
            try:
                parsed = parse_manifest_ron(actual_manifest_bytes.decode("utf-8"))
                if len(parsed) != len(synth.SOUND_SPECS):
                    errors.append(
                        f"El manifiesto contiene {len(parsed)} entradas, "
                        f"se esperaban {len(synth.SOUND_SPECS)}"
                    )
            except ValueError as exc:
                errors.append(f"Error al analizar {MANIFEST_PATH}: {exc}")

    if not LICENSE_PATH.is_file():
        errors.append(f"Falta el archivo de licencia: {LICENSE_PATH}")
    elif LICENSE_PATH.stat().st_size == 0:
        errors.append(f"El archivo de licencia está vacío: {LICENSE_PATH}")

    if AUDIO_DIR.is_dir():
        total_bytes = sum(
            p.stat().st_size for p in AUDIO_DIR.rglob("*") if p.is_file()
        )
        if total_bytes > MAX_AUDIO_DIR_BYTES:
            errors.append(
                f"El directorio {AUDIO_DIR} ocupa {total_bytes} bytes, "
                f"superando el máximo de {MAX_AUDIO_DIR_BYTES} bytes (8 MB)"
            )

    return (len(errors) == 0, errors)


def main(argv: list[str] | None = None) -> int:
    """Punto de entrada de línea de comandos para generar o verificar el audio."""
    parser = argparse.ArgumentParser(
        description="Genera o verifica los 30 efectos de sonido WAV y el manifest.ron de HUSHMINE."
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Verifica que los archivos en disco coincidan byte a byte sin modificarlos.",
    )
    args = parser.parse_args(argv)

    if args.check:
        ok, errors = check_all()
        if ok:
            print(
                "Verificación completada con éxito: los 30 archivos WAV, manifest.ron "
                "y LICENSE.md son válidos y coinciden byte a byte."
            )
            return 0
        for err in errors:
            print(f"ERROR: {err}", file=sys.stderr)
        return 1

    build_all()
    print(
        f"Generados 30 efectos de sonido WAV en {SFX_DIR} y manifiesto en {MANIFEST_PATH}."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
