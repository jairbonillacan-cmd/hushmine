"""Síntesis procedural de efectos de radio (El Vigía, 1979) e interfaz (UI)."""

from __future__ import annotations

import math
from typing import Final

from tools.audio import synth

RADIO_UI_SOUND_IDS: Final[tuple[str, ...]] = (
    "radio_click_on",
    "radio_click_off",
    "radio_static_loop",
    "ui_select",
    "ui_back",
    "ui_error",
)


def synth_radio_click_on() -> list[float]:
    """Apertura del pulsador PTT de baquelita y enganche de portadora del Vigía."""
    n = synth.samples_for("radio_click_on")
    buf = [0.0] * n

    # 1. Chasquido mecánico inicial del botón de baquelita en t=0
    click1_len = 1100
    click1_raw = synth.mix_tracks(
        [
            synth.white_noise(click1_len, seed=501, amp=0.75),
            synth.sine_wave(click1_len, freq_hz=1850.0, amp=0.55),
            synth.triangle_wave(click1_len, freq_hz=820.0, amp=0.45),
        ],
        soft_clip_drive=1.4,
    )
    click1_env = synth.exp_decay_envelope(click1_len, attack_s=0.0005, tau_s=0.0045)
    click1 = synth.bandpass_one_pole(synth.apply_envelope(click1_raw, click1_env), 600.0, 3200.0)
    synth.place_at(buf, click1, offset_samples=0, gain=0.95)

    # 2. Segundo contacto del muelle metálico interno y relé a los ~18 ms
    click2_offset = int(round(0.018 * synth.SAMPLE_RATE))
    click2_len = 900
    click2_raw = synth.mix_tracks(
        [
            synth.white_noise(click2_len, seed=502, amp=0.60),
            synth.fm_tone(click2_len, carrier_hz=2240.0, mod_hz=680.0, mod_index=1.4, amp=0.55),
        ],
        soft_clip_drive=1.3,
    )
    click2_env = synth.exp_decay_envelope(click2_len, attack_s=0.0004, tau_s=0.0038)
    click2 = synth.bandpass_one_pole(synth.apply_envelope(click2_raw, click2_env), 750.0, 3400.0)
    synth.place_at(buf, click2, offset_samples=click2_offset, gain=0.80)

    # 3. Breve enganche de portadora de onda corta (950 -> 1480 Hz) con estática
    carrier_offset = int(round(0.022 * synth.SAMPLE_RATE))
    carrier_len = n - carrier_offset
    carrier_sweep = synth.freq_sweep(
        carrier_len, f_start=950.0, f_end=1480.0, amp=0.42, waveform="sine", exp_curve=True
    )
    static_burst = synth.white_noise(carrier_len, seed=503, amp=0.58)
    carrier_mix = synth.mix_tracks([carrier_sweep, static_burst], weights=[0.55, 0.45], soft_clip_drive=1.2)
    carrier_env = synth.adsr_envelope(
        carrier_len, attack_s=0.004, decay_s=0.024, sustain_level=0.28, release_s=0.040
    )
    carrier_band = synth.bandpass_one_pole(
        synth.apply_envelope(carrier_mix, carrier_env), 300.0, 3000.0
    )
    synth.place_at(buf, carrier_band, offset_samples=carrier_offset, gain=0.65)

    return synth.finalize_sound("radio_click_on", buf)


def synth_radio_click_off() -> list[float]:
    """Soltado del pulsador PTT, caída de portadora y cola de silenciador (squelch tail)."""
    n = synth.samples_for("radio_click_off")
    buf = [0.0] * n

    # 1. Cola de portadora que cae (1320 -> 680 Hz) + corte de estática (squelch tail)
    tail_len = 2650
    carrier_drop = synth.freq_sweep(
        tail_len, f_start=1320.0, f_end=680.0, amp=0.46, waveform="triangle", exp_curve=True
    )
    squelch_noise = synth.white_noise(tail_len, seed=511, amp=0.68)
    tail_mix = synth.mix_tracks([carrier_drop, squelch_noise], weights=[0.48, 0.52], soft_clip_drive=1.2)
    tail_env = synth.exp_decay_envelope(tail_len, attack_s=0.001, tau_s=0.017)
    tail_band = synth.bandpass_one_pole(synth.apply_envelope(tail_mix, tail_env), 300.0, 2900.0)
    synth.place_at(buf, tail_band, offset_samples=0, gain=0.72)

    # 2. Golpe seco de retorno del conmutador de baquelita a los ~14 ms
    thud_offset = int(round(0.014 * synth.SAMPLE_RATE))
    thud_len = 1350
    thud_raw = synth.mix_tracks(
        [
            synth.white_noise(thud_len, seed=512, amp=0.80),
            synth.freq_sweep(thud_len, f_start=1180.0, f_end=460.0, amp=0.70, waveform="sine"),
        ],
        soft_clip_drive=1.4,
    )
    thud_env = synth.exp_decay_envelope(thud_len, attack_s=0.0005, tau_s=0.0050)
    thud = synth.bandpass_one_pole(synth.apply_envelope(thud_raw, thud_env), 400.0, 2600.0)
    synth.place_at(buf, thud, offset_samples=thud_offset, gain=0.95)

    # 3. Rebote secundario del muelle de retorno a los ~31 ms
    bounce_offset = int(round(0.031 * synth.SAMPLE_RATE))
    bounce_len = 850
    bounce_raw = synth.mix_tracks(
        [
            synth.white_noise(bounce_len, seed=513, amp=0.48),
            synth.sine_wave(bounce_len, freq_hz=920.0, amp=0.42),
        ],
        soft_clip_drive=1.2,
    )
    bounce_env = synth.exp_decay_envelope(bounce_len, attack_s=0.0004, tau_s=0.0035)
    bounce = synth.bandpass_one_pole(synth.apply_envelope(bounce_raw, bounce_env), 400.0, 2600.0)
    synth.place_at(buf, bounce, offset_samples=bounce_offset, gain=0.60)

    return synth.finalize_sound("radio_click_off", buf)


def synth_radio_static_loop() -> list[float]:
    """Bucle continuo de estática analógica de onda corta (banda 300..3000 Hz, 1979)."""
    n = synth.samples_for("radio_static_loop")

    # Ruido blanco determinista en banda telefónica (300..3000 Hz) + siseo de cinta
    base_noise = synth.bandpass_one_pole(synth.white_noise(n, seed=521, amp=0.85), 300.0, 3000.0)
    tape_hiss = synth.bandpass_one_pole(synth.white_noise(n, seed=522, amp=0.35), 450.0, 2400.0)

    # Crepitaciones suaves de válvula termoiónica lejos de las fronteras del bucle
    crackle_raw = [0.0] * n
    rng = synth.DeterministicRng(523)
    margin = 3000
    for _ in range(58):
        pos = margin + (rng.next_u32() % (n - 2 * margin))
        amp = 0.25 + 0.45 * rng.next_float()
        sign = 1.0 if (rng.next_u32() & 1) == 0 else -1.0
        pulse_len = 18 + (rng.next_u32() % 28)
        for k in range(pulse_len):
            if pos + k < n:
                crackle_raw[pos + k] += sign * amp * math.exp(-k / 6.0)
    crackle = synth.bandpass_one_pole(crackle_raw, 500.0, 2800.0)

    # Modulación de desvanecimiento ionosférico lento (ciclos exactos en 1.50 s: 2 Hz y 4 Hz)
    # y tenue heterodino lejano (~1050 Hz, múltiplo exacto del periodo de 1.50 s)
    out: list[float] = [0.0] * n
    two_pi = 2.0 * math.pi
    for i in range(n):
        t = i / synth.SAMPLE_RATE
        ion_fade = 0.74 + 0.18 * math.sin(two_pi * 2.0 * t) + 0.08 * math.cos(two_pi * 4.0 * t)
        het_mod = 0.65 + 0.35 * math.sin(two_pi * 2.0 * t + 0.8)
        heterodyne = 0.045 * het_mod * math.sin(two_pi * 1050.0 * t)
        sample = (0.72 * base_noise[i] + 0.28 * tape_hiss[i]) * ion_fade + 0.32 * crackle[i] + heterodyne
        out[i] = synth.soft_clip(sample, drive=1.15)

    filtered = synth.bandpass_one_pole(out, 300.0, 3000.0)
    return synth.finalize_sound("radio_static_loop", filtered)


def synth_ui_select() -> list[float]:
    """Confirmación en menús o terminal CRT de fósforo verde (660 Hz -> 990 Hz)."""
    n = synth.samples_for("ui_select")
    buf = [0.0] * n

    # 1. Clic táctil de tecla mecánica en t=0
    click_len = 520
    click_raw = synth.mix_tracks(
        [
            synth.white_noise(click_len, seed=531, amp=0.55),
            synth.sine_wave(click_len, freq_hz=2400.0, amp=0.45),
        ],
        soft_clip_drive=1.2,
    )
    click_env = synth.exp_decay_envelope(click_len, attack_s=0.0004, tau_s=0.0024)
    click = synth.bandpass_one_pole(synth.apply_envelope(click_raw, click_env), 800.0, 4200.0)
    synth.place_at(buf, click, offset_samples=0, gain=0.55)

    # 2. Primer escalón nítido de terminal CRT (660 Hz)
    step1_offset = 132
    step1_len = 1850
    step1_wave = synth.mix_tracks(
        [
            synth.triangle_wave(step1_len, freq_hz=660.0, amp=0.65),
            synth.sine_wave(step1_len, freq_hz=660.0, amp=0.45),
            synth.sine_wave(step1_len, freq_hz=1320.0, amp=0.12),
        ],
        soft_clip_drive=1.1,
    )
    step1_env = synth.exp_decay_envelope(step1_len, attack_s=0.0015, tau_s=0.024)
    synth.place_at(buf, synth.apply_envelope(step1_wave, step1_env), offset_samples=step1_offset, gain=0.85)

    # 3. Segundo escalón ascendente (990 Hz, quinta justa)
    step2_offset = 1764
    step2_len = n - step2_offset
    step2_wave = synth.mix_tracks(
        [
            synth.triangle_wave(step2_len, freq_hz=990.0, amp=0.68),
            synth.sine_wave(step2_len, freq_hz=990.0, amp=0.48),
            synth.sine_wave(step2_len, freq_hz=1980.0, amp=0.10),
        ],
        soft_clip_drive=1.1,
    )
    step2_env = synth.exp_decay_envelope(step2_len, attack_s=0.0015, tau_s=0.026)
    synth.place_at(buf, synth.apply_envelope(step2_wave, step2_env), offset_samples=step2_offset, gain=0.95)

    shaped = synth.lowpass_one_pole(buf, 3800.0)
    return synth.finalize_sound("ui_select", shaped)


def synth_ui_back() -> list[float]:
    """Cancelación o vuelta atrás con clic de conmutador y quinta descendente (740 -> 493.88 Hz)."""
    n = synth.samples_for("ui_back")
    buf = [0.0] * n

    # 1. Clic de conmutador mecánico en t=0
    click_len = 580
    click_raw = synth.mix_tracks(
        [
            synth.white_noise(click_len, seed=541, amp=0.50),
            synth.triangle_wave(click_len, freq_hz=1600.0, amp=0.45),
        ],
        soft_clip_drive=1.2,
    )
    click_env = synth.exp_decay_envelope(click_len, attack_s=0.0005, tau_s=0.0028)
    click = synth.bandpass_one_pole(synth.apply_envelope(click_raw, click_env), 500.0, 3200.0)
    synth.place_at(buf, click, offset_samples=0, gain=0.55)

    # 2. Primer tono analógico cálido (740.0 Hz)
    tone1_offset = 132
    tone1_len = 2050
    tone1_wave = synth.mix_tracks(
        [
            synth.triangle_wave(tone1_len, freq_hz=740.0, amp=0.65),
            synth.sine_wave(tone1_len, freq_hz=740.0, amp=0.45),
            synth.saw_wave(tone1_len, freq_hz=370.0, amp=0.15),
        ],
        soft_clip_drive=1.15,
    )
    tone1_env = synth.exp_decay_envelope(tone1_len, attack_s=0.0015, tau_s=0.022)
    synth.place_at(buf, synth.apply_envelope(tone1_wave, tone1_env), offset_samples=tone1_offset, gain=0.90)

    # 3. Segundo tono descendente (493.88 Hz, quinta descendente)
    tone2_offset = 1984
    tone2_len = n - tone2_offset
    tone2_wave = synth.mix_tracks(
        [
            synth.triangle_wave(tone2_len, freq_hz=493.88, amp=0.70),
            synth.sine_wave(tone2_len, freq_hz=493.88, amp=0.48),
            synth.saw_wave(tone2_len, freq_hz=246.94, amp=0.18),
        ],
        soft_clip_drive=1.15,
    )
    tone2_env = synth.exp_decay_envelope(tone2_len, attack_s=0.0015, tau_s=0.026)
    synth.place_at(buf, synth.apply_envelope(tone2_wave, tone2_env), offset_samples=tone2_offset, gain=0.88)

    warm = synth.lowpass_one_pole(buf, 2800.0)
    return synth.finalize_sound("ui_back", warm)


def synth_ui_error() -> list[float]:
    """Doble zumbido grave de relé bloqueado (165 Hz + 155 Hz, dos pulsos de 0.09 s)."""
    n = synth.samples_for("ui_error")
    buf = [0.0] * n

    pulse_len = int(round(0.09 * synth.SAMPLE_RATE))  # 3969 muestras (0.09 s)
    gap_len = int(round(0.03 * synth.SAMPLE_RATE))  # 1323 muestras (0.03 s)
    offsets = (353, 353 + pulse_len + gap_len)

    for idx, offset in enumerate(offsets):
        # Chasquido seco de armadura de relé al inicio de cada pulso
        snap_len = 480
        snap_raw = synth.white_noise(snap_len, seed=550 + idx * 2, amp=0.65)
        snap_env = synth.exp_decay_envelope(snap_len, attack_s=0.0004, tau_s=0.0025)
        snap = synth.bandpass_one_pole(synth.apply_envelope(snap_raw, snap_env), 450.0, 2400.0)
        synth.place_at(buf, snap, offset_samples=offset, gain=0.45)

        # Zumbido grave disonante (saw 165 Hz + square 155 Hz + sub-octava 82.5 Hz)
        buzz_raw = synth.mix_tracks(
            [
                synth.saw_wave(pulse_len, freq_hz=165.0, amp=0.58),
                synth.square_wave(pulse_len, freq_hz=155.0, amp=0.52, duty=0.46),
                synth.triangle_wave(pulse_len, freq_hz=82.5, amp=0.28),
                synth.white_noise(pulse_len, seed=560 + idx, amp=0.10),
            ],
            soft_clip_drive=1.35,
        )
        buzz_env = synth.adsr_envelope(
            pulse_len, attack_s=0.003, decay_s=0.015, sustain_level=0.78, release_s=0.018
        )
        gain = 0.95 if idx == 0 else 0.90
        synth.place_at(buf, synth.apply_envelope(buzz_raw, buzz_env), offset_samples=offset, gain=gain)

    filtered = synth.highpass_one_pole(synth.lowpass_one_pole(buf, 1100.0), 55.0)
    return synth.finalize_sound("ui_error", filtered)


def generate_radio_sfx() -> dict[str, list[float]]:
    """Genera el diccionario completo de los 6 sonidos de radio e interfaz."""
    return {
        "radio_click_on": synth_radio_click_on(),
        "radio_click_off": synth_radio_click_off(),
        "radio_static_loop": synth_radio_static_loop(),
        "ui_select": synth_ui_select(),
        "ui_back": synth_ui_back(),
        "ui_error": synth_ui_error(),
    }
