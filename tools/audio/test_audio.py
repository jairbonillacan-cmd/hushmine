"""Suite de pruebas automatizadas del catálogo de audio de HUSHMINE.

Verifica el formato WAV PCM de 16 bits a 44 100 Hz mono, la ausencia de
saturación y silencio, la continuidad de los bucles en los extremos, el
determinismo byte a byte de la síntesis procedural, la correspondencia
uno a uno entre `manifest.ron`, `assets/audio/sfx/*.wav` y la guía de
sonido, y los presupuestos de tamaño (<= 8 MB) y longitud de archivo
(<= 400 líneas con saltos LF).
"""

from __future__ import annotations

import math
from pathlib import Path
import struct
import subprocess
import sys
import unittest
import wave

REPO_ROOT: Path = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.audio import build_all, synth  # noqa: E402

AUDIO_DIR: Path = REPO_ROOT / "assets" / "audio"
SFX_DIR: Path = AUDIO_DIR / "sfx"
MANIFEST_PATH: Path = AUDIO_DIR / "manifest.ron"
DOC_GUIDE_PATH: Path = REPO_ROOT / "docs" / "audio" / "guia-sonido.md"
TOOLS_AUDIO_DIR: Path = REPO_ROOT / "tools" / "audio"
DOCS_AUDIO_DIR: Path = REPO_ROOT / "docs" / "audio"


def _unpack_wav_pcm16(wav_path: Path) -> list[int]:
    """Abre un archivo WAV en modo lectura binaria y desempaqueta sus muestras int16."""
    with wave.open(str(wav_path), "rb") as wf:
        nframes = wf.getnframes()
        nchannels = wf.getnchannels()
        raw_bytes = wf.readframes(nframes)
    total_samples = nframes * nchannels
    return list(struct.unpack(f"<{total_samples}h", raw_bytes))


class TestAudioCatalog(unittest.TestCase):
    """Pruebas de integridad acústica, determinismo y formato del catálogo de HUSHMINE."""

    def test_wav_format_pcm16_44100_mono(self) -> None:
        """Comprueba que los 30 archivos WAV sean PCM 16-bit, 44 100 Hz, mono y de longitud exacta."""
        self.assertEqual(
            len(synth.SOUND_SPECS), 30, "SOUND_SPECS debe definir exactamente 30 sonidos."
        )
        manifest_entries = build_all.parse_manifest_ron(
            MANIFEST_PATH.read_text(encoding="utf-8")
        )
        manifest_by_id = {str(entry["id"]): entry for entry in manifest_entries}

        for spec in synth.SOUND_SPECS:
            snd_id = str(spec["id"])
            rel_file = str(spec["file"])
            expected_samples = int(spec["samples"])  # type: ignore[arg-type]
            wav_path = AUDIO_DIR / rel_file

            self.assertTrue(wav_path.is_file(), f"No existe el WAV esperado: {wav_path}")
            self.assertIn(snd_id, manifest_by_id, f"Falta {snd_id!r} en manifest.ron.")
            self.assertEqual(
                manifest_by_id[snd_id]["samples"],
                expected_samples,
                f"Muestras en manifest.ron no coinciden con SOUND_SPECS para {snd_id!r}.",
            )

            with wave.open(str(wav_path), "rb") as wf:
                self.assertEqual(
                    wf.getnchannels(), 1, f"{wav_path.name} debe ser mono (1 canal)."
                )
                self.assertEqual(
                    wf.getsampwidth(), 2, f"{wav_path.name} debe ser de 16 bits (2 bytes)."
                )
                self.assertEqual(
                    wf.getframerate(), 44_100, f"{wav_path.name} debe estar a 44 100 Hz."
                )
                self.assertEqual(
                    wf.getcomptype(), "NONE", f"{wav_path.name} debe ser PCM ('NONE')."
                )
                self.assertEqual(
                    wf.getnframes(),
                    expected_samples,
                    f"{wav_path.name} tiene {wf.getnframes()} muestras, "
                    f"se esperaban {expected_samples}.",
                )

    def test_no_saturation_and_not_silent(self) -> None:
        """Verifica que ningún archivo WAV sature (> 0,98) ni esté en silencio."""
        for spec in synth.SOUND_SPECS:
            snd_id = str(spec["id"])
            wav_path = AUDIO_DIR / str(spec["file"])
            pcm = _unpack_wav_pcm16(wav_path)

            self.assertGreater(len(pcm), 0, f"El archivo {wav_path.name} está vacío.")
            max_abs = max(abs(s) for s in pcm)
            peak_pos = max_abs / 32_767.0
            peak_neg = max_abs / 32_768.0
            rms = math.sqrt(sum((s / 32_767.0) ** 2 for s in pcm) / len(pcm))

            self.assertLessEqual(
                max_abs, 32_111, f"{snd_id!r} supera el techo entero 32 111 ({max_abs})."
            )
            self.assertLessEqual(
                peak_pos, 0.98, f"{snd_id!r} satura sobre 32767.0 ({peak_pos:.5f} > 0.98)."
            )
            self.assertLessEqual(
                peak_neg, 0.98, f"{snd_id!r} satura sobre 32768.0 ({peak_neg:.5f} > 0.98)."
            )
            self.assertGreaterEqual(
                peak_pos, 0.20, f"{snd_id!r} tiene pico demasiado bajo ({peak_pos:.5f} < 0.20)."
            )
            self.assertGreaterEqual(
                rms, 0.01, f"{snd_id!r} es prácticamente silencioso (RMS={rms:.5f} < 0.01)."
            )

    def test_loops_seamless_zero_endpoints(self) -> None:
        """Comprueba que todos los bucles comiencen y terminen cerca de cero sin salto ni chasquido."""
        manifest_entries = build_all.parse_manifest_ron(
            MANIFEST_PATH.read_text(encoding="utf-8")
        )
        loop_entries = [
            entry
            for entry in manifest_entries
            if bool(entry["looping"]) or str(entry["id"]).endswith("_loop")
        ]
        loop_ids = {str(entry["id"]) for entry in loop_entries}
        expected_loops = {"bucket_line_loop", "mine_ambience_loop", "radio_static_loop"}

        self.assertGreaterEqual(
            len(loop_entries), 3, "Deben existir al menos 3 efectos de sonido en bucle."
        )
        self.assertTrue(
            expected_loops.issubset(loop_ids),
            f"Faltan bucles obligatorios en el manifiesto: {expected_loops - loop_ids}",
        )

        for entry in loop_entries:
            snd_id = str(entry["id"])
            self.assertTrue(
                bool(entry["looping"]) and snd_id.endswith("_loop"),
                f"Inconsistencia de nomenclatura o bandera de bucle en {snd_id!r}.",
            )
            pcm = _unpack_wav_pcm16(AUDIO_DIR / str(entry["file"]))
            self.assertGreaterEqual(len(pcm), 4, f"El bucle {snd_id!r} es demasiado corto.")

            start_norm = abs(pcm[0]) / 32_767.0
            end_norm = abs(pcm[-1]) / 32_767.0
            wrap_delta = abs(pcm[-1] - pcm[0]) / 32_767.0
            start_slope = abs(pcm[1] - pcm[0]) / 32_767.0
            end_slope = abs(pcm[-1] - pcm[-2]) / 32_767.0

            self.assertLessEqual(
                start_norm, 0.005, f"{snd_id!r} no empieza cerca de cero ({start_norm:.6f})."
            )
            self.assertLessEqual(
                end_norm, 0.005, f"{snd_id!r} no termina cerca de cero ({end_norm:.6f})."
            )
            self.assertLessEqual(
                wrap_delta, 0.005, f"Salto excesivo final-inicio en {snd_id!r} ({wrap_delta:.6f})."
            )
            self.assertLessEqual(
                start_slope, 0.02, f"Pendiente inicial brusca en {snd_id!r} ({start_slope:.6f})."
            )
            self.assertLessEqual(
                end_slope, 0.02, f"Pendiente final brusca en {snd_id!r} ({end_slope:.6f})."
            )

    def test_deterministic_generation(self) -> None:
        """Verifica que la síntesis sea determinista y coincida byte a byte con el disco."""
        run_a = build_all.generate_all_sfx()
        run_b = build_all.generate_all_sfx()

        self.assertEqual(
            list(run_a.keys()),
            list(run_b.keys()),
            "El orden o conjunto de claves varía entre dos ejecuciones consecutivas.",
        )

        encoded_by_id: dict[str, bytes] = {}
        for spec in synth.SOUND_SPECS:
            snd_id = str(spec["id"])
            wav_path = AUDIO_DIR / str(spec["file"])
            self.assertEqual(
                run_a[snd_id], run_b[snd_id], f"Muestras float no deterministas en {snd_id!r}."
            )

            bytes_a = synth.encode_wav_bytes(run_a[snd_id])
            bytes_b = synth.encode_wav_bytes(run_b[snd_id])
            disk_bytes = wav_path.read_bytes()

            self.assertEqual(bytes_a, bytes_b, f"Bytes WAV difieren entre pasadas en {snd_id!r}.")
            self.assertEqual(
                bytes_a,
                disk_bytes,
                f"El archivo en disco {wav_path.name} no coincide byte a byte con el generado.",
            )
            encoded_by_id[snd_id] = bytes_a

        footstep_ids = [f"footstep_{i}" for i in range(1, 5)]
        self.assertEqual(
            len({encoded_by_id[k] for k in footstep_ids}),
            len(footstep_ids),
            "Las 4 variantes footstep_1..4 deben producir bytes distintos entre sí.",
        )

        whistler_ids = [f"whistler_{i}" for i in range(1, 4)]
        self.assertEqual(
            len({encoded_by_id[k] for k in whistler_ids}),
            len(whistler_ids),
            "Las 3 variantes whistler_1..3 deben producir bytes distintos entre sí.",
        )

    def test_manifest_and_files_one_to_one(self) -> None:
        """Comprueba la correspondencia biunívoca entre manifest.ron, disco y guia-sonido.md."""
        entries = build_all.parse_manifest_ron(MANIFEST_PATH.read_text(encoding="utf-8"))
        self.assertEqual(
            len(entries), 30, f"manifest.ron debe tener 30 entradas, tiene {len(entries)}."
        )

        manifest_ids = [str(e["id"]) for e in entries]
        manifest_files = [str(e["file"]) for e in entries]
        spec_ids = [str(s["id"]) for s in synth.SOUND_SPECS]
        spec_files = [str(s["file"]) for s in synth.SOUND_SPECS]

        self.assertEqual(len(set(manifest_ids)), 30, "Hay IDs duplicados en manifest.ron.")
        self.assertEqual(len(set(manifest_files)), 30, "Hay archivos duplicados en manifest.ron.")
        self.assertEqual(manifest_ids, spec_ids, "IDs de manifest.ron != synth.SOUND_SPECS.")
        self.assertEqual(manifest_files, spec_files, "Rutas de manifest.ron != synth.SOUND_SPECS.")

        expected_wav_names = {Path(f).name for f in manifest_files}
        disk_wav_names = {p.name for p in SFX_DIR.glob("*.wav") if p.is_file()}
        self.assertEqual(
            disk_wav_names,
            expected_wav_names,
            f"Desajuste entre WAV en disco y manifest.ron: "
            f"faltan={sorted(expected_wav_names - disk_wav_names)}, "
            f"sobran={sorted(disk_wav_names - expected_wav_names)}.",
        )

        self.assertTrue(DOC_GUIDE_PATH.is_file(), f"No existe la guía: {DOC_GUIDE_PATH}")
        guide_text = DOC_GUIDE_PATH.read_text(encoding="utf-8")
        for snd_id in manifest_ids:
            self.assertIn(
                f"`{snd_id}`",
                guide_text,
                f"El identificador {snd_id!r} no está documentado en docs/audio/guia-sonido.md.",
            )

    def test_total_size_under_8mb_and_line_limits(self) -> None:
        """Verifica el presupuesto <= 8 MB en assets/audio/ y <= 400 líneas con LF en código y docs."""
        audio_files = [p for p in AUDIO_DIR.rglob("*") if p.is_file()]
        total_bytes = sum(p.stat().st_size for p in audio_files)

        self.assertGreater(total_bytes, 0, "assets/audio/ no puede tener tamaño total cero.")
        self.assertLessEqual(
            total_bytes,
            8 * 1024 * 1024,
            f"El tamaño de assets/audio/ ({total_bytes} B) supera el límite de 8 MB.",
        )

        checked_dirs = (TOOLS_AUDIO_DIR, DOCS_AUDIO_DIR, AUDIO_DIR)
        checked_files_count = 0
        for directory in checked_dirs:
            for file_path in sorted(directory.rglob("*")):
                if not file_path.is_file() or file_path.suffix not in (".py", ".md", ".ron"):
                    continue
                if "__pycache__" in file_path.parts:
                    continue
                checked_files_count += 1
                raw = file_path.read_bytes()
                self.assertNotIn(
                    b"\r\n",
                    raw,
                    f"El archivo {file_path} contiene saltos CRLF (\\r\\n); debe usar LF (\\n).",
                )
                line_count = len(raw.decode("utf-8").splitlines())
                self.assertLessEqual(
                    line_count,
                    400,
                    f"El archivo {file_path} tiene {line_count} líneas (máximo permitido: 400).",
                )

        self.assertGreaterEqual(
            checked_files_count, 8, "Se esperaban al menos 8 archivos (.py, .md, .ron)."
        )

    def test_build_all_check_cli(self) -> None:
        """Comprueba la función check_all() y la invocación por CLI de build_all.py --check."""
        ok, errors = build_all.check_all()
        self.assertTrue(ok, f"build_all.check_all() devolvió errores: {errors}")
        self.assertEqual(errors, [], f"La lista de errores debe estar vacía: {errors}")

        proc = subprocess.run(
            [sys.executable, "-B", "tools/audio/build_all.py", "--check"],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(
            proc.returncode,
            0,
            f"El CLI 'build_all.py --check' falló (código {proc.returncode}): "
            f"stdout={proc.stdout!r}, stderr={proc.stderr!r}",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
