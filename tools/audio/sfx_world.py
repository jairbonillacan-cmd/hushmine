"""Generadores procedurales de sismógrafo, silbadores y ambiente para HUSHMINE."""

from __future__ import annotations

import math
from typing import Final

from tools.audio import synth

WORLD_SOUND_IDS: Final[tuple[str, ...]] = (
    "seismo_tick",
    "seismo_stir",
    "seismo_breathe",
    "seismo_restless",
    "wake_rumble",
    "whistler_1",
    "whistler_2",
    "whistler_3",
    "geophone_dig",
    "offering_accept",
    "tally_return",
    "mine_ambience_loop",
)


def _gen_seismo_tick() -> list[float]:
    """Chasquido electromecánico de la plumilla entintada sobre el tambor de papel."""
    n = synth.samples_for("seismo_tick")
    sr = synth.SAMPLE_RATE
    nib_env = synth.exp_decay_envelope(n, attack_s=0.0005, tau_s=0.012)
    nib = synth.apply_envelope(synth.fm_tone(n, 1850.0, 2700.0, 1.6, amp=0.65), nib_env)

    sol_env = synth.exp_decay_envelope(n, attack_s=0.0008, tau_s=0.018)
    solenoid = synth.apply_envelope(synth.triangle_wave(n, 320.0, amp=0.75), sol_env)

    paper_env = synth.exp_decay_envelope(n, attack_s=0.0004, tau_s=0.008)
    paper_raw = synth.bandpass_one_pole(synth.white_noise(n, seed=401, amp=0.45), 1200.0, 4200.0)
    paper = synth.apply_envelope(paper_raw, paper_env)

    raw = synth.mix_tracks([nib, solenoid, paper], weights=[0.75, 0.85, 0.45], soft_clip_drive=1.3)
    ratchet_len = int(0.022 * sr)
    ratchet = synth.apply_envelope(
        synth.fm_tone(ratchet_len, 2150.0, 1450.0, 1.1, amp=0.28),
        synth.exp_decay_envelope(ratchet_len, attack_s=0.0004, tau_s=0.005),
    )
    synth.place_at(raw, ratchet, int(0.032 * sr), gain=0.55)
    return synth.finalize_sound("seismo_tick", raw)


def _gen_seismo_stir() -> list[float]:
    """Umbral 1 (Revuelo, 25 %): temblor subgrave y toque de aviso en tubo de bronce."""
    n = synth.samples_for("seismo_stir")
    sr = synth.SAMPLE_RATE
    body_env = synth.adsr_envelope(n, attack_s=0.08, decay_s=0.25, sustain_level=0.65, release_s=0.45)

    sub_sweep = synth.freq_sweep(n, 68.0, 42.0, amp=0.80, waveform="sine")
    sub_harm = synth.freq_sweep(n, 136.0, 84.0, amp=0.28, waveform="triangle")
    rumble_noise = synth.lowpass_one_pole(synth.white_noise(n, seed=402, amp=0.75), 140.0)
    tremor = [
        (sub_sweep[i] + sub_harm[i] + rumble_noise[i])
        * body_env[i]
        * (0.82 + 0.18 * math.sin(2.0 * math.pi * 6.5 * i / sr))
        for i in range(n)
    ]

    chime_len = int(0.95 * sr)
    chime_env = synth.exp_decay_envelope(chime_len, attack_s=0.004, tau_s=0.26)
    chime_a = synth.fm_tone(chime_len, 440.0, 660.0, 1.2, amp=0.55)
    chime_b = synth.sine_wave(chime_len, 880.0, amp=0.18)
    chime = [
        (chime_a[i] + chime_b[i]) * chime_env[i]
        for i in range(chime_len)
    ]

    raw = list(tremor)
    synth.place_at(raw, chime, int(0.08 * sr), gain=0.78)
    return synth.finalize_sound("seismo_stir", synth.mix_tracks([raw], soft_clip_drive=1.25))


def _gen_seismo_breathe() -> list[float]:
    """Umbral 2 (Respira, 50 %): exhalación cavernosa y resonancia de la Veta Madre."""
    n = synth.samples_for("seismo_breathe")
    sr = synth.SAMPLE_RATE
    breath_env = synth.adsr_envelope(n, attack_s=0.42, decay_s=0.35, sustain_level=0.68, release_s=0.52)

    wind_raw = synth.white_noise(n, seed=403, amp=0.95)
    wind_bp = synth.bandpass_one_pole(wind_raw, 120.0, 580.0)
    wind_deep = synth.lowpass_one_pole(wind_raw, 160.0)
    tectonic = synth.freq_sweep(n, 55.0, 36.0, amp=0.75, waveform="sine")

    base = [
        (0.65 * wind_bp[i] + 0.55 * wind_deep[i] + 0.70 * tectonic[i]) * breath_env[i]
        for i in range(n)
    ]

    quartz_len = int(1.15 * sr)
    q_env = synth.exp_decay_envelope(quartz_len, attack_s=0.035, tau_s=0.38)
    q_low = synth.fm_tone(quartz_len, 293.66, 587.32, 0.65, amp=0.42)
    q_high = synth.fm_tone(quartz_len, 440.0, 880.0, 0.45, amp=0.30)
    quartz = [(q_low[i] + q_high[i]) * q_env[i] for i in range(quartz_len)]

    synth.place_at(base, quartz, int(0.22 * sr), gain=0.80)
    return synth.finalize_sound("seismo_breathe", synth.mix_tracks([base], soft_clip_drive=1.2))


def _gen_seismo_restless() -> list[float]:
    """Umbral 3 (Inquietud, 75 %): torsión de roca, vigas y doble pulso de zona roja."""
    n = synth.samples_for("seismo_restless")
    sr = synth.SAMPLE_RATE
    env = synth.adsr_envelope(n, attack_s=0.06, decay_s=0.30, sustain_level=0.75, release_s=0.38)

    saw_groan = synth.lowpass_one_pole(
        synth.freq_sweep(n, 72.0, 40.0, amp=0.85, waveform="saw"), 240.0
    )
    sub_groan = synth.freq_sweep(n, 54.0, 31.0, amp=0.65, waveform="triangle")
    grit = synth.bandpass_one_pole(synth.white_noise(n, seed=404, amp=0.60), 90.0, 420.0)
    raw = [(0.65 * saw_groan[i] + 0.55 * sub_groan[i] + 0.45 * grit[i]) * env[i] for i in range(n)]

    rng = synth.DeterministicRng(4040)
    for snap_offset_s in (0.12, 0.48, 0.88, 1.18):
        snap_len = int(0.11 * sr)
        f0 = 260.0 + rng.next_float() * 160.0
        snap_env = synth.exp_decay_envelope(snap_len, attack_s=0.001, tau_s=0.025)
        snap_sig = synth.freq_sweep(snap_len, f0, 65.0, amp=0.55, waveform="triangle")
        snap_noise = synth.bandpass_one_pole(
            synth.white_noise(snap_len, seed=rng.next_u32(), amp=0.5), 300.0, 1600.0
        )
        crack = [(snap_sig[i] + snap_noise[i]) * snap_env[i] for i in range(snap_len)]
        synth.place_at(raw, crack, int(snap_offset_s * sr), gain=0.65)

    for pulse_s, freq in ((0.18, 620.0), (0.72, 465.0)):
        p_len = int(0.34 * sr)
        p_env = synth.adsr_envelope(p_len, 0.012, 0.08, 0.65, 0.12)
        p_wave = synth.lowpass_one_pole(synth.square_wave(p_len, freq, amp=0.48, duty=0.45), 1350.0)
        pulse = synth.apply_envelope(p_wave, p_env)
        synth.place_at(raw, pulse, int(pulse_s * sr), gain=0.62)

    return synth.finalize_sound("seismo_restless", synth.mix_tracks([raw], soft_clip_drive=1.35))


def _gen_wake_rumble() -> list[float]:
    """Umbral 4 (Despertar, 100 %): estruendo colosal, latidos del Corazón y campana de 1931."""
    n = synth.samples_for("wake_rumble")
    sr = synth.SAMPLE_RATE
    main_env = synth.adsr_envelope(n, attack_s=0.10, decay_s=0.40, sustain_level=0.82, release_s=0.48)

    sub_a = synth.freq_sweep(n, 48.0, 24.0, amp=0.90, waveform="sine")
    sub_b = synth.freq_sweep(n, 72.0, 32.0, amp=0.50, waveform="triangle")
    fault_roar = synth.lowpass_one_pole(synth.white_noise(n, seed=405, amp=0.95), 260.0)
    raw = [(0.75 * sub_a[i] + 0.45 * sub_b[i] + 0.65 * fault_roar[i]) * main_env[i] for i in range(n)]

    for beat_s in (0.25, 1.15):
        lub_len = int(0.26 * sr)
        lub = synth.apply_envelope(
            synth.freq_sweep(lub_len, 66.0, 27.0, amp=0.95, waveform="sine"),
            synth.exp_decay_envelope(lub_len, attack_s=0.008, tau_s=0.085),
        )
        dub_len = int(0.32 * sr)
        dub = synth.apply_envelope(
            synth.freq_sweep(dub_len, 54.0, 23.0, amp=0.80, waveform="sine"),
            synth.exp_decay_envelope(dub_len, attack_s=0.010, tau_s=0.110),
        )
        synth.place_at(raw, lub, int(beat_s * sr), gain=0.90)
        synth.place_at(raw, dub, int((beat_s + 0.19) * sr), gain=0.78)

    bell_hits = (0.12, 0.56, 1.00, 1.44, 1.82)
    for hit_s in bell_hits:
        b_len = int(0.36 * sr)
        b_env = synth.exp_decay_envelope(b_len, attack_s=0.002, tau_s=0.11)
        b_tone = synth.fm_tone(b_len, 784.0, 1110.0, 1.45, amp=0.42)
        b_filt = synth.lowpass_one_pole(synth.apply_envelope(b_tone, b_env), 2200.0)
        synth.place_at(raw, b_filt, int(hit_s * sr), gain=0.48)

    return synth.finalize_sound("wake_rumble", synth.mix_tracks([raw], soft_clip_drive=1.45))


def _whistle_grain(
    num_samples: int,
    f_start: float,
    f_end: float,
    beat_hz: float,
    vibrato_hz: float,
    seed: int,
) -> list[float]:
    """Sintetiza un soplo de silbato de fisura con batimento inarmónico y aire."""
    sr = synth.SAMPLE_RATE
    env = synth.adsr_envelope(num_samples, attack_s=0.035, decay_s=0.08, sustain_level=0.78, release_s=0.11)
    air = synth.bandpass_one_pole(synth.white_noise(num_samples, seed=seed, amp=0.42), 900.0, 2600.0)
    out: list[float] = []
    p1 = 0.0
    p2 = 0.0
    two_pi = 2.0 * math.pi
    denom = max(1, num_samples - 1)
    for i in range(num_samples):
        t = i / denom
        vib = 6.0 * math.sin(two_pi * vibrato_hz * (i / sr))
        f_inst = f_start + (f_end - f_start) * t + vib
        p1 += two_pi * f_inst / sr
        p2 += two_pi * (f_inst + beat_hz) / sr
        tone = 0.58 * math.sin(p1) + 0.34 * math.sin(p2) + 0.16 * math.sin(2.0 * p1 + 0.4)
        out.append((tone + 0.32 * air[i]) * env[i])
    return out


def _place_with_cavern_echo(dest: list[float], src: list[float], offset: int) -> None:
    """Coloca un silbido en la pista junto con dos ecos de galería filtrados."""
    sr = synth.SAMPLE_RATE
    synth.place_at(dest, src, offset, gain=1.0)
    echo_1 = synth.lowpass_one_pole(src, 1250.0)
    echo_2 = synth.lowpass_one_pole(echo_1, 820.0)
    synth.place_at(dest, echo_1, offset + int(0.15 * sr), gain=0.42)
    synth.place_at(dest, echo_2, offset + int(0.31 * sr), gain=0.20)


def _gen_whistler_1() -> list[float]:
    """Silbador 1: llamada ascendente-descendente en fisura (780 -> 1180 -> 860 Hz)."""
    n = synth.samples_for("whistler_1")
    sr = synth.SAMPLE_RATE
    raw = synth.bandpass_one_pole(synth.white_noise(n, seed=406, amp=0.12), 400.0, 1800.0)
    seg_a = _whistle_grain(int(0.42 * sr), 780.0, 1180.0, beat_hz=5.5, vibrato_hz=5.2, seed=4061)
    seg_b = _whistle_grain(int(0.46 * sr), 1180.0, 860.0, beat_hz=6.0, vibrato_hz=4.8, seed=4062)
    _place_with_cavern_echo(raw, seg_a, int(0.03 * sr))
    _place_with_cavern_echo(raw, seg_b, int(0.36 * sr))
    return synth.finalize_sound("whistler_1", synth.mix_tracks([raw], soft_clip_drive=1.15))


def _gen_whistler_2() -> list[float]:
    """Silbador 2: doble toque de advertencia (1040 Hz corto y 820 -> 690 Hz largo)."""
    n = synth.samples_for("whistler_2")
    sr = synth.SAMPLE_RATE
    raw = synth.bandpass_one_pole(synth.white_noise(n, seed=407, amp=0.12), 380.0, 1750.0)
    pip = _whistle_grain(int(0.21 * sr), 1040.0, 1020.0, beat_hz=6.5, vibrato_hz=6.0, seed=4071)
    long_call = _whistle_grain(int(0.56 * sr), 820.0, 690.0, beat_hz=7.0, vibrato_hz=4.5, seed=4072)
    _place_with_cavern_echo(raw, pip, int(0.04 * sr))
    _place_with_cavern_echo(raw, long_call, int(0.31 * sr))
    return synth.finalize_sound("whistler_2", synth.mix_tracks([raw], soft_clip_drive=1.15))


def _gen_whistler_3() -> list[float]:
    """Silbador 3: cadencia de tres pulsos de capataz y glissando frío (980 -> 540 Hz)."""
    n = synth.samples_for("whistler_3")
    sr = synth.SAMPLE_RATE
    raw = synth.bandpass_one_pole(synth.white_noise(n, seed=408, amp=0.12), 360.0, 1700.0)
    p1 = _whistle_grain(int(0.14 * sr), 1240.0, 1210.0, beat_hz=5.0, vibrato_hz=6.5, seed=4081)
    p2 = _whistle_grain(int(0.14 * sr), 1240.0, 1200.0, beat_hz=5.5, vibrato_hz=6.5, seed=4082)
    fall = _whistle_grain(int(0.52 * sr), 980.0, 540.0, beat_hz=6.8, vibrato_hz=4.2, seed=4083)
    _place_with_cavern_echo(raw, p1, int(0.03 * sr))
    _place_with_cavern_echo(raw, p2, int(0.21 * sr))
    _place_with_cavern_echo(raw, fall, int(0.40 * sr))
    return synth.finalize_sound("whistler_3", synth.mix_tracks([raw], soft_clip_drive=1.15))


def _gen_geophone_dig() -> list[float]:
    """Picado rival transmitido por la roca sólida y captado con membrana de geófono."""
    n = synth.samples_for("geophone_dig")
    sr = synth.SAMPLE_RATE
    raw = synth.lowpass_one_pole(synth.white_noise(n, seed=409, amp=0.08), 240.0)

    for hit_s, gain, seed in ((0.02, 1.0, 4091), (0.20, 0.58, 4092)):
        h_len = int(0.18 * sr)
        h_env = synth.exp_decay_envelope(h_len, attack_s=0.002, tau_s=0.038)
        thud = synth.freq_sweep(h_len, 280.0, 72.0, amp=0.85, waveform="triangle")
        clink = synth.fm_tone(h_len, 410.0, 615.0, 0.9, amp=0.45)
        mica = synth.sine_wave(h_len, 185.0, amp=0.50)
        rock_noise = synth.lowpass_one_pole(synth.white_noise(h_len, seed=seed, amp=0.55), 480.0)
        strike = [
            (thud[i] + clink[i] + mica[i] + rock_noise[i]) * h_env[i]
            for i in range(h_len)
        ]
        synth.place_at(raw, strike, int(hit_s * sr), gain=gain)

    muffled = synth.lowpass_one_pole(synth.lowpass_one_pole(raw, 390.0), 350.0)
    return synth.finalize_sound("geophone_dig", muffled)


def _gen_offering_accept() -> list[float]:
    """El altar acepta una Ofrenda de oro: lingote en tolva y acorde cálido de alivio."""
    n = synth.samples_for("offering_accept")
    sr = synth.SAMPLE_RATE
    raw = [0.0] * n

    for drop_s, freq, gain in ((0.015, 920.0, 0.75), (0.105, 680.0, 0.60)):
        d_len = int(0.14 * sr)
        d_env = synth.exp_decay_envelope(d_len, attack_s=0.001, tau_s=0.028)
        ingot = synth.fm_tone(d_len, freq, freq * 1.41, 1.1, amp=0.65)
        stone = synth.freq_sweep(d_len, 165.0, 68.0, amp=0.55, waveform="triangle")
        hit = [(ingot[i] + stone[i]) * d_env[i] for i in range(d_len)]
        synth.place_at(raw, hit, int(drop_s * sr), gain=gain)

    chord_len = n - int(0.08 * sr)
    c_env = synth.adsr_envelope(chord_len, attack_s=0.05, decay_s=0.25, sustain_level=0.55, release_s=0.42)
    v1 = synth.fm_tone(chord_len, 220.0, 440.0, 0.45, amp=0.48)
    v2 = synth.fm_tone(chord_len, 330.0, 660.0, 0.35, amp=0.38)
    v3 = synth.sine_wave(chord_len, 440.0, amp=0.30)
    sigh = synth.freq_sweep(chord_len, 110.0, 52.0, amp=0.45, waveform="sine")
    steam = synth.bandpass_one_pole(synth.white_noise(chord_len, seed=410, amp=0.25), 140.0, 480.0)
    relief = [
        (v1[i] + v2[i] + v3[i] + sigh[i] + steam[i]) * c_env[i]
        for i in range(chord_len)
    ]
    synth.place_at(raw, relief, int(0.08 * sr), gain=0.85)
    return synth.finalize_sound("offering_accept", synth.mix_tracks([raw], soft_clip_drive=1.2))


def _gen_tally_return() -> list[float]:
    """Devolución de Ficha de latón (#1-#41 de 1931): enganche metálico y acorde solemne."""
    n = synth.samples_for("tally_return")
    sr = synth.SAMPLE_RATE
    raw = [0.0] * n

    for clink_s, c_freq, m_freq, gain in (
        (0.005, 1320.0, 1980.0, 0.82),
        (0.115, 1980.0, 2970.0, 0.72),
    ):
        c_len = int(0.24 * sr)
        c_env = synth.exp_decay_envelope(c_len, attack_s=0.0008, tau_s=0.055)
        brass = synth.fm_tone(c_len, c_freq, m_freq, 0.95, amp=0.70)
        shimmer = synth.sine_wave(c_len, c_freq * 2.0, amp=0.25)
        clink = [(brass[i] + shimmer[i]) * c_env[i] for i in range(c_len)]
        synth.place_at(raw, clink, int(clink_s * sr), gain=gain)

    body_len = n - int(0.10 * sr)
    b_env = synth.adsr_envelope(body_len, attack_s=0.06, decay_s=0.30, sustain_level=0.62, release_s=0.52)
    h1 = synth.fm_tone(body_len, 196.0, 392.0, 0.50, amp=0.45)
    h2 = synth.fm_tone(body_len, 293.66, 587.33, 0.35, amp=0.36)
    h3 = synth.sine_wave(body_len, 392.0, amp=0.30)
    h4 = synth.sine_wave(body_len, 587.33, amp=0.22)
    calm_sub = synth.freq_sweep(body_len, 98.0, 49.0, amp=0.48, waveform="sine")
    solemn = [
        (h1[i] + h2[i] + h3[i] + h4[i] + calm_sub[i]) * b_env[i]
        for i in range(body_len)
    ]
    synth.place_at(raw, solemn, int(0.10 * sr), gain=0.88)
    return synth.finalize_sound("tally_return", synth.mix_tracks([raw], soft_clip_drive=1.2))


def _gen_mine_ambience_loop() -> list[float]:
    """Bucle ambiental continuo (3.0 s) de galería profunda: dron, corriente de aire y sodio."""
    n = synth.samples_for("mine_ambience_loop")
    sr = synth.SAMPLE_RATE
    two_pi = 2.0 * math.pi

    air_low = synth.lowpass_one_pole(synth.white_noise(n, seed=412, amp=0.55), 190.0)
    air_mid = synth.bandpass_one_pole(synth.white_noise(n, seed=413, amp=0.40), 220.0, 520.0)

    raw: list[float] = []
    for i in range(n):
        t = i / sr
        lfo_1 = 0.78 + 0.22 * math.sin(two_pi * (i / n))
        lfo_2 = 0.75 + 0.25 * math.cos(two_pi * 2.0 * (i / n))
        drone = (
            0.48 * math.sin(two_pi * 42.0 * t) * lfo_1
            + 0.30 * math.sin(two_pi * 63.0 * t) * lfo_2
            + 0.14 * math.sin(two_pi * 84.0 * t)
        )
        lamp_hum = 0.06 * math.sin(two_pi * 100.0 * t) + 0.03 * math.sin(two_pi * 200.0 * t + 0.5)
        draft = 0.55 * air_low[i] * lfo_1 + 0.35 * air_mid[i] * lfo_2
        raw.append(drone + lamp_hum + draft)

    return synth.finalize_sound("mine_ambience_loop", synth.mix_tracks([raw], soft_clip_drive=1.15))


def generate_world_sfx() -> dict[str, list[float]]:
    """Genera el diccionario canónico de los 12 sonidos del sismógrafo y el ambiente."""
    generators = (
        ("seismo_tick", _gen_seismo_tick),
        ("seismo_stir", _gen_seismo_stir),
        ("seismo_breathe", _gen_seismo_breathe),
        ("seismo_restless", _gen_seismo_restless),
        ("wake_rumble", _gen_wake_rumble),
        ("whistler_1", _gen_whistler_1),
        ("whistler_2", _gen_whistler_2),
        ("whistler_3", _gen_whistler_3),
        ("geophone_dig", _gen_geophone_dig),
        ("offering_accept", _gen_offering_accept),
        ("tally_return", _gen_tally_return),
        ("mine_ambience_loop", _gen_mine_ambience_loop),
    )
    return {sound_id: fn() for sound_id, fn in generators}
