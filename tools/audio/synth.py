"""Sintetizador base procedural y determinista en Python puro para HUSHMINE."""

from __future__ import annotations

import io
import math
from pathlib import Path
import struct
from typing import Final
import wave

SAMPLE_RATE: int = 44_100
NUM_CHANNELS: int = 1
SAMPLE_WIDTH_BYTES: int = 2
INT16_MAX: int = 32_767
MAX_PEAK_LIMIT: float = 0.98
CATEGORIES: tuple[str, ...] = ("tools", "blocks", "seismograph", "ambience", "radio", "ui")

_RAW_SPECS: Final[tuple[tuple[str, str, int, bool, str, float], ...]] = (
    ("pick_hit_rock", "sfx/pick_hit_rock.wav", 7938, False, "tools", 0.78),
    ("pick_hit_dirt", "sfx/pick_hit_dirt.wav", 7056, False, "tools", 0.70),
    ("block_break", "sfx/block_break.wav", 15435, False, "blocks", 0.84),
    ("block_place", "sfx/block_place.wav", 9702, False, "blocks", 0.76),
    ("explosion", "sfx/explosion.wav", 52920, False, "blocks", 0.94),
    ("electric_zap", "sfx/electric_zap.wav", 17640, False, "blocks", 0.86),
    ("lava_bubble", "sfx/lava_bubble.wav", 24255, False, "blocks", 0.72),
    ("bucket_line_loop", "sfx/bucket_line_loop.wav", 66150, True, "blocks", 0.68),
    ("footstep_1", "sfx/footstep_1.wav", 6174, False, "tools", 0.58),
    ("footstep_2", "sfx/footstep_2.wav", 6174, False, "tools", 0.58),
    ("footstep_3", "sfx/footstep_3.wav", 6174, False, "tools", 0.58),
    ("footstep_4", "sfx/footstep_4.wav", 6174, False, "tools", 0.58),
    ("seismo_tick", "sfx/seismo_tick.wav", 3969, False, "seismograph", 0.64),
    ("seismo_stir", "sfx/seismo_stir.wav", 52920, False, "seismograph", 0.88),
    ("seismo_breathe", "sfx/seismo_breathe.wav", 66150, False, "seismograph", 0.90),
    ("seismo_restless", "sfx/seismo_restless.wav", 70560, False, "seismograph", 0.92),
    ("wake_rumble", "sfx/wake_rumble.wav", 97020, False, "seismograph", 0.94),
    ("whistler_1", "sfx/whistler_1.wav", 48510, False, "ambience", 0.80),
    ("whistler_2", "sfx/whistler_2.wav", 50715, False, "ambience", 0.80),
    ("whistler_3", "sfx/whistler_3.wav", 52920, False, "ambience", 0.80),
    ("geophone_dig", "sfx/geophone_dig.wav", 19845, False, "ambience", 0.70),
    ("offering_accept", "sfx/offering_accept.wav", 39690, False, "seismograph", 0.84),
    ("tally_return", "sfx/tally_return.wav", 48510, False, "seismograph", 0.86),
    ("mine_ambience_loop", "sfx/mine_ambience_loop.wav", 132300, True, "ambience", 0.45),
    ("radio_click_on", "sfx/radio_click_on.wav", 5292, False, "radio", 0.74),
    ("radio_click_off", "sfx/radio_click_off.wav", 5292, False, "radio", 0.72),
    ("radio_static_loop", "sfx/radio_static_loop.wav", 66150, True, "radio", 0.48),
    ("ui_select", "sfx/ui_select.wav", 4410, False, "ui", 0.68),
    ("ui_back", "sfx/ui_back.wav", 4851, False, "ui", 0.65),
    ("ui_error", "sfx/ui_error.wav", 11025, False, "ui", 0.78),
)

SOUND_SPECS: list[dict[str, object]] = [
    {
        "id": snd_id,
        "file": rel_file,
        "samples": samples,
        "looping": looping,
        "category": category,
        "peak_target": peak_target,
    }
    for snd_id, rel_file, samples, looping, category, peak_target in _RAW_SPECS
]

SPEC_BY_ID: dict[str, dict[str, object]] = {str(s["id"]): s for s in SOUND_SPECS}


def samples_for(sound_id: str) -> int:
    """Devuelve el número exacto de muestras para un identificador canónico."""
    val = SPEC_BY_ID[sound_id]["samples"]
    assert isinstance(val, int)
    return val


class DeterministicRng:
    """Generador congruencial lineal (LCG) de 32 bits 100 % determinista."""

    def __init__(self, seed: int) -> None:
        """Inicializa el estado interno con una semilla entera explícita."""
        self._state: int = (int(seed) & 0xFFFFFFFF) or 0x6D2B79F5

    def next_u32(self) -> int:
        """Avanza el estado LCG y devuelve un entero de 32 bits sin signo."""
        self._state = (1664525 * self._state + 1013904223) & 0xFFFFFFFF
        return self._state

    def next_float(self) -> float:
        """Devuelve un número en coma flotante en el intervalo [0.0, 1.0)."""
        return self.next_u32() / 4294967296.0

    def next_bipolar(self) -> float:
        """Devuelve una muestra uniforme en el intervalo [-1.0, 1.0]."""
        return (self.next_u32() / 2147483647.5) - 1.0


def white_noise(num_samples: int, seed: int, amp: float = 1.0) -> list[float]:
    """Genera ruido blanco determinista a partir de una semilla explícita."""
    rng = DeterministicRng(seed)
    return [rng.next_bipolar() * amp for _ in range(num_samples)]


def sine_wave(num_samples: int, freq_hz: float, amp: float = 1.0, phase: float = 0.0) -> list[float]:
    """Genera una onda senoidal pura de frecuencia y fase inicial dadas."""
    step = 2.0 * math.pi * freq_hz / SAMPLE_RATE
    return [amp * math.sin(phase + i * step) for i in range(num_samples)]


def square_wave(num_samples: int, freq_hz: float, amp: float = 1.0, duty: float = 0.5) -> list[float]:
    """Genera una onda cuadrada con ciclo de trabajo configurable."""
    step = freq_hz / SAMPLE_RATE
    return [amp if ((i * step) % 1.0) < duty else -amp for i in range(num_samples)]


def saw_wave(num_samples: int, freq_hz: float, amp: float = 1.0) -> list[float]:
    """Genera una onda en diente de sierra ascendente entre -amp y +amp."""
    step = freq_hz / SAMPLE_RATE
    return [amp * (2.0 * ((i * step) % 1.0) - 1.0) for i in range(num_samples)]


def triangle_wave(num_samples: int, freq_hz: float, amp: float = 1.0) -> list[float]:
    """Genera una onda triangular simétrica continua."""
    step = freq_hz / SAMPLE_RATE
    return [amp * (1.0 - 4.0 * abs(((i * step + 0.25) % 1.0) - 0.5)) for i in range(num_samples)]


def freq_sweep(
    num_samples: int,
    f_start: float,
    f_end: float,
    amp: float = 1.0,
    waveform: str = "sine",
    exp_curve: bool = False,
) -> list[float]:
    """Genera un barrido continuo de frecuencia integrando la fase instantánea."""
    if waveform not in ("sine", "triangle", "saw", "square"):
        raise ValueError(f"Forma de onda no soportada: {waveform}")
    out: list[float] = []
    phase = 0.0
    two_pi = 2.0 * math.pi
    denom = max(1, num_samples - 1)
    use_exp = exp_curve and f_start > 0.0 and f_end > 0.0
    ratio = (f_end / f_start) if use_exp else 1.0
    for i in range(num_samples):
        t = i / denom
        freq = (f_start * (ratio**t)) if use_exp else (f_start + (f_end - f_start) * t)
        if waveform == "sine":
            val = math.sin(phase)
        elif waveform == "square":
            val = 1.0 if ((phase / two_pi) % 1.0) < 0.5 else -1.0
        elif waveform == "saw":
            val = 2.0 * ((phase / two_pi) % 1.0) - 1.0
        else:
            val = 1.0 - 4.0 * abs((((phase / two_pi) + 0.25) % 1.0) - 0.5)
        out.append(amp * val)
        phase += two_pi * freq / SAMPLE_RATE
    return out


def fm_tone(
    num_samples: int, carrier_hz: float, mod_hz: float, mod_index: float, amp: float = 1.0
) -> list[float]:
    """Síntesis por modulación de frecuencia (FM) de un operador modulador."""
    c_step = 2.0 * math.pi * carrier_hz / SAMPLE_RATE
    m_step = 2.0 * math.pi * mod_hz / SAMPLE_RATE
    return [amp * math.sin(i * c_step + mod_index * math.sin(i * m_step)) for i in range(num_samples)]


def adsr_envelope(
    num_samples: int, attack_s: float, decay_s: float, sustain_level: float, release_s: float
) -> list[float]:
    """Genera una envolvente ADSR lineal ajustada al número total de muestras."""
    if num_samples <= 0:
        return []
    n_att = max(0, int(round(attack_s * SAMPLE_RATE)))
    n_dec = max(0, int(round(decay_s * SAMPLE_RATE)))
    n_rel = max(0, int(round(release_s * SAMPLE_RATE)))
    total_fixed = n_att + n_dec + n_rel
    if total_fixed > num_samples and total_fixed > 0:
        scale = num_samples / total_fixed
        n_att = int(n_att * scale)
        n_dec = int(n_dec * scale)
        n_rel = max(0, num_samples - n_att - n_dec)
    n_sus = max(0, num_samples - n_att - n_dec - n_rel)

    env: list[float] = [(i + 1) / n_att for i in range(n_att)]
    env.extend(1.0 - (1.0 - sustain_level) * ((i + 1) / n_dec) for i in range(n_dec))
    env.extend([sustain_level] * n_sus)
    env.extend(sustain_level * (1.0 - ((i + 1) / n_rel)) for i in range(n_rel))
    if len(env) < num_samples:
        env.extend([0.0] * (num_samples - len(env)))
    return env[:num_samples]


def exp_decay_envelope(num_samples: int, attack_s: float = 0.002, tau_s: float = 0.1) -> list[float]:
    """Envolvente de percusión con ataque lineal breve y caída exponencial."""
    if num_samples <= 0:
        return []
    n_att = min(num_samples, max(1, int(round(attack_s * SAMPLE_RATE)))) if attack_s > 0.0 else 0
    inv_tau = 1.0 / (max(1e-6, tau_s) * SAMPLE_RATE)
    return [
        ((i + 1) / n_att) if i < n_att else math.exp(-(i - n_att) * inv_tau)
        for i in range(num_samples)
    ]


def apply_envelope(samples: list[float], envelope: list[float]) -> list[float]:
    """Multiplica una señal por su envolvente muestra a muestra."""
    env_len = len(envelope)
    return [s * (envelope[i] if i < env_len else 0.0) for i, s in enumerate(samples)]


def lowpass_one_pole(samples: list[float], cutoff_hz: float) -> list[float]:
    """Filtro IIR paso bajo de primer orden."""
    if not samples or cutoff_hz <= 0.0:
        return [0.0] * len(samples)
    dt = 1.0 / SAMPLE_RATE
    rc = 1.0 / (2.0 * math.pi * cutoff_hz)
    alpha = dt / (rc + dt)
    out: list[float] = []
    y = 0.0
    for x in samples:
        y = y + alpha * (x - y)
        out.append(y)
    return out


def highpass_one_pole(samples: list[float], cutoff_hz: float) -> list[float]:
    """Filtro IIR paso alto de primer orden."""
    if not samples:
        return []
    if cutoff_hz <= 0.0:
        return list(samples)
    dt = 1.0 / SAMPLE_RATE
    rc = 1.0 / (2.0 * math.pi * cutoff_hz)
    alpha = rc / (rc + dt)
    out: list[float] = []
    y = 0.0
    prev_x = 0.0
    for x in samples:
        y = alpha * (y + x - prev_x)
        prev_x = x
        out.append(y)
    return out


def bandpass_one_pole(samples: list[float], low_cutoff_hz: float, high_cutoff_hz: float) -> list[float]:
    """Filtro paso banda encadenando paso alto y paso bajo de un polo."""
    return lowpass_one_pole(highpass_one_pole(samples, low_cutoff_hz), high_cutoff_hz)


def soft_clip(sample: float, drive: float = 1.0) -> float:
    """Saturación suave basada en tangente hiperbólica normalizada."""
    if drive <= 0.0:
        return sample
    return math.tanh(sample * drive) / math.tanh(drive)


def mix_tracks(
    tracks: list[list[float]],
    weights: list[float] | None = None,
    soft_clip_drive: float | None = 1.2,
) -> list[float]:
    """Suma múltiples pistas con pesos opcionales y saturación suave opcional."""
    if not tracks:
        return []
    max_len = max(len(t) for t in tracks)
    w_list = weights if weights is not None else [1.0] * len(tracks)
    out = [0.0] * max_len
    for track, weight in zip(tracks, w_list):
        for i, s in enumerate(track):
            out[i] += s * weight
    if soft_clip_drive is not None and soft_clip_drive > 0.0:
        denom = math.tanh(soft_clip_drive)
        out = [math.tanh(s * soft_clip_drive) / denom for s in out]
    return out


def place_at(dest: list[float], src: list[float], offset_samples: int, gain: float = 1.0) -> None:
    """Suma una sub-pista en `dest` a partir de `offset_samples` (in-place)."""
    dest_len = len(dest)
    start_src = max(0, -offset_samples)
    end_src = min(len(src), dest_len - offset_samples)
    for i in range(start_src, end_src):
        dest[offset_samples + i] += src[i] * gain


def normalize_peak(samples: list[float], target_peak: float = 0.85) -> list[float]:
    """Escala la señal para que su pico máximo sea exactamente min(target_peak, 0.95)."""
    if not samples:
        raise ValueError("No se puede normalizar una lista vacía de muestras.")
    peak = max(abs(s) for s in samples)
    if peak <= 1e-12:
        raise ValueError("La señal es completamente silenciosa (pico cero).")
    scale = min(float(target_peak), 0.95) / peak
    return [s * scale for s in samples]


def make_seamless_loop(
    samples: list[float], crossfade_samples: int = 2205, endpoint_taper: int = 64
) -> list[float]:
    """Convierte una pista de longitud N en un bucle continuo sin salto en los extremos."""
    n = len(samples)
    if n == 0:
        return []
    mean = sum(samples) / n
    src = [s - mean for s in samples]
    out = list(src)
    cf = min(crossfade_samples, n // 4)
    if cf > 1:
        for i in range(cf):
            w = 0.5 * (1.0 - math.cos(math.pi * (i / (cf - 1))))
            a, b = src[i], src[n - cf + i]
            out[i] = (0.5 + 0.5 * w) * a + (0.5 - 0.5 * w) * b
            out[n - cf + i] = (1.0 - 0.5 * w) * b + (0.5 * w) * a
    taper = min(endpoint_taper, n // 4)
    if taper > 0:
        for i in range(taper):
            w = 0.5 * (1.0 - math.cos(math.pi * i / taper))
            out[i] *= w
            out[n - 1 - i] *= w
        out[0] = 0.0
        out[-1] = 0.0
    return out


def finalize_sound(sound_id: str, raw_samples: list[float]) -> list[float]:
    """Ajusta longitud, aplica cierre de bucle o micro-fundido y normaliza el pico."""
    spec = SPEC_BY_ID[sound_id]
    target_len = samples_for(sound_id)
    looping = bool(spec["looping"])
    peak_target_obj = spec["peak_target"]
    assert isinstance(peak_target_obj, (int, float))
    samples = (
        list(raw_samples[:target_len])
        if len(raw_samples) >= target_len
        else list(raw_samples) + [0.0] * (target_len - len(raw_samples))
    )
    if looping:
        samples = make_seamless_loop(samples)
    else:
        fade = min(32, len(samples) // 2)
        for i in range(fade):
            w = i / fade
            samples[i] *= w
            samples[-1 - i] *= w
    samples = normalize_peak(samples, target_peak=float(peak_target_obj))
    if looping and samples:
        samples[0] = 0.0
        samples[-1] = 0.0
    return samples


def floats_to_pcm16_bytes(samples: list[float]) -> bytes:
    """Empaqueta muestras en [-1.0, 1.0] a bytes PCM de 16 bits con signo little-endian."""
    ints = [int(round(max(-MAX_PEAK_LIMIT, min(MAX_PEAK_LIMIT, s)) * INT16_MAX)) for s in samples]
    return struct.pack(f"<{len(ints)}h", *ints)


def encode_wav_bytes(samples: list[float], sample_rate: int = SAMPLE_RATE) -> bytes:
    """Codifica una lista de muestras en un archivo RIFF WAV PCM 16-bit mono en memoria."""
    buf = io.BytesIO()
    with wave.open(buf, "wb") as wf:
        wf.setnchannels(NUM_CHANNELS)
        wf.setsampwidth(SAMPLE_WIDTH_BYTES)
        wf.setframerate(sample_rate)
        wf.writeframes(floats_to_pcm16_bytes(samples))
    return buf.getvalue()


def write_wav(path: str | Path, samples: list[float], sample_rate: int = SAMPLE_RATE) -> bytes:
    """Escribe un archivo WAV en disco y devuelve los bytes exactos generados."""
    target = Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    data = encode_wav_bytes(samples, sample_rate=sample_rate)
    target.write_bytes(data)
    return data


def read_wav_samples(path_or_bytes: str | Path | bytes) -> tuple[int, int, int, list[int]]:
    """Lee un WAV desde ruta o bytes y devuelve (canales, bytes_muestra, rate, muestras_int16)."""
    source: io.BytesIO | str = (
        io.BytesIO(path_or_bytes) if isinstance(path_or_bytes, (bytes, bytearray)) else str(path_or_bytes)
    )
    with wave.open(source, "rb") as wf:
        nchannels, sampwidth, framerate, nframes = (
            wf.getnchannels(),
            wf.getsampwidth(),
            wf.getframerate(),
            wf.getnframes(),
        )
        raw = wf.readframes(nframes)
    samples = list(struct.unpack(f"<{nframes * nchannels}h", raw))
    return nchannels, sampwidth, framerate, samples
