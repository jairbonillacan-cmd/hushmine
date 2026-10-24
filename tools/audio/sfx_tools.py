"""Síntesis procedural de efectos de herramientas y bloques para HUSHMINE."""

from __future__ import annotations

import math
from typing import Final

from tools.audio import synth

TOOLS_SOUND_IDS: Final[tuple[str, ...]] = (
    "pick_hit_rock",
    "pick_hit_dirt",
    "block_break",
    "block_place",
    "explosion",
    "electric_zap",
    "lava_bubble",
    "bucket_line_loop",
    "footstep_1",
    "footstep_2",
    "footstep_3",
    "footstep_4",
)

# (semilla, f_inicio_tacón, f_fin_tacón, offset_puntera, f_puntera, bp_bajo, bp_alto, tau_grava)
_FOOTSTEP_VARIANTS: Final[dict[int, tuple[int, float, float, int, float, float, float, float]]] = {
    1: (10901, 118.0, 48.0, 1420, 156.0, 420.0, 2450.0, 0.032),
    2: (20902, 96.0, 42.0, 1740, 138.0, 340.0, 1950.0, 0.038),
    3: (30903, 134.0, 56.0, 1260, 174.0, 540.0, 2950.0, 0.027),
    4: (40904, 108.0, 45.0, 1580, 148.0, 380.0, 2200.0, 0.035),
}


def synth_pick_hit_rock() -> list[float]:
    """Genera el impacto metálico de pico sobre roca común, roca dura o mineral."""
    n = synth.samples_for("pick_hit_rock")
    rock_thud = synth.apply_envelope(
        synth.freq_sweep(n, 240.0, 85.0, amp=1.0, waveform="sine", exp_curve=True),
        synth.exp_decay_envelope(n, attack_s=0.001, tau_s=0.022),
    )
    steel_main = synth.apply_envelope(
        synth.fm_tone(n, carrier_hz=1420.0, mod_hz=2180.0, mod_index=2.6, amp=0.9),
        synth.exp_decay_envelope(n, attack_s=0.0008, tau_s=0.035),
    )
    steel_chime = synth.apply_envelope(
        synth.fm_tone(n, carrier_hz=2890.0, mod_hz=1910.0, mod_index=1.8, amp=0.6),
        synth.exp_decay_envelope(n, attack_s=0.0006, tau_s=0.020),
    )
    shards = synth.apply_envelope(
        synth.bandpass_one_pole(synth.white_noise(n, seed=10101, amp=1.0), 1800.0, 6500.0),
        synth.exp_decay_envelope(n, attack_s=0.0008, tau_s=0.025),
    )
    mixed = synth.mix_tracks(
        [rock_thud, steel_main, steel_chime, shards],
        weights=[0.95, 0.80, 0.38, 0.70],
        soft_clip_drive=1.35,
    )
    return synth.finalize_sound("pick_hit_rock", mixed)


def synth_pick_hit_dirt() -> list[float]:
    """Genera el golpe seco y amortiguado del pico contra tierra compacta."""
    n = synth.samples_for("pick_hit_dirt")
    low_thud = synth.apply_envelope(
        synth.freq_sweep(n, 150.0, 52.0, amp=1.0, waveform="sine", exp_curve=True),
        synth.exp_decay_envelope(n, attack_s=0.0015, tau_s=0.040),
    )
    body_thump = synth.apply_envelope(
        synth.freq_sweep(n, 112.0, 44.0, amp=0.8, waveform="triangle", exp_curve=True),
        synth.exp_decay_envelope(n, attack_s=0.002, tau_s=0.034),
    )
    soil_low = synth.apply_envelope(
        synth.lowpass_one_pole(synth.white_noise(n, seed=10201, amp=1.0), 950.0),
        synth.exp_decay_envelope(n, attack_s=0.002, tau_s=0.046),
    )
    clods_mid = synth.apply_envelope(
        synth.bandpass_one_pole(synth.white_noise(n, seed=10202, amp=1.0), 250.0, 1400.0),
        synth.exp_decay_envelope(n, attack_s=0.004, tau_s=0.050),
    )
    mixed = synth.mix_tracks(
        [low_thud, body_thump, soil_low, clods_mid],
        weights=[1.0, 0.55, 0.65, 0.75],
        soft_clip_drive=1.25,
    )
    return synth.finalize_sound("pick_hit_dirt", mixed)


def synth_block_break() -> list[float]:
    """Genera la fractura estructural y el desmoronamiento completo de un bloque."""
    n = synth.samples_for("block_break")
    base_drop = synth.apply_envelope(
        synth.freq_sweep(n, 190.0, 45.0, amp=1.0, waveform="sine", exp_curve=True),
        synth.exp_decay_envelope(n, attack_s=0.0015, tau_s=0.085),
    )
    crumble_bed = synth.apply_envelope(
        synth.bandpass_one_pole(synth.white_noise(n, seed=10301, amp=1.0), 300.0, 2400.0),
        synth.exp_decay_envelope(n, attack_s=0.002, tau_s=0.095),
    )
    debris_track = [0.0] * n
    fragments = (
        (0, 3600, 260.0, 88.0, 420.0, 2600.0, 0.90, 10311),
        (2150, 3400, 225.0, 76.0, 360.0, 2300.0, 0.75, 10312),
        (4700, 3200, 295.0, 102.0, 520.0, 2800.0, 0.66, 10313),
        (7600, 3100, 185.0, 66.0, 310.0, 1950.0, 0.56, 10314),
        (10700, 3600, 240.0, 80.0, 390.0, 2200.0, 0.45, 10315),
    )
    for offset, frag_len, f0, f1, bp_lo, bp_hi, gain, seed in fragments:
        env = synth.exp_decay_envelope(frag_len, attack_s=0.001, tau_s=0.028)
        knock = synth.freq_sweep(frag_len, f0, f1, amp=0.75, waveform="triangle", exp_curve=True)
        grit = synth.bandpass_one_pole(synth.white_noise(frag_len, seed=seed, amp=0.95), bp_lo, bp_hi)
        synth.place_at(debris_track, synth.apply_envelope([k + g for k, g in zip(knock, grit)], env), offset, gain)

    mixed = synth.mix_tracks([base_drop, crumble_bed, debris_track], weights=[0.90, 0.60, 0.95], soft_clip_drive=1.35)
    return synth.finalize_sound("block_break", mixed)


def synth_block_place() -> list[float]:
    """Genera el golpe firme de colocación seguido del anclaje mecánico de cerrojo."""
    n = synth.samples_for("block_place")
    canvas = [0.0] * n

    n_seat = 5600
    seat_env = synth.exp_decay_envelope(n_seat, attack_s=0.0012, tau_s=0.036)
    seat_sine = synth.freq_sweep(n_seat, 110.0, 65.0, amp=1.0, waveform="sine", exp_curve=True)
    seat_scrape = synth.bandpass_one_pole(synth.white_noise(n_seat, seed=10401, amp=0.55), 260.0, 1250.0)
    synth.place_at(canvas, synth.apply_envelope([a + b for a, b in zip(seat_sine, seat_scrape)], seat_env), 0, 1.0)

    # Clac metálico de remache/cerrojo a ~70 ms (offset 3087 muestras)
    n_latch = 4400
    latch_env = synth.exp_decay_envelope(n_latch, attack_s=0.0008, tau_s=0.022)
    latch_fm1 = synth.fm_tone(n_latch, carrier_hz=920.0, mod_hz=1380.0, mod_index=2.1, amp=0.80)
    latch_fm2 = synth.fm_tone(n_latch, carrier_hz=1380.0, mod_hz=2070.0, mod_index=1.4, amp=0.50)
    latch_click = synth.bandpass_one_pole(synth.white_noise(n_latch, seed=10402, amp=0.65), 900.0, 4200.0)
    latch_raw = [a + b + c for a, b, c in zip(latch_fm1, latch_fm2, latch_click)]
    synth.place_at(canvas, synth.apply_envelope(latch_raw, latch_env), 3087, gain=0.85)

    return synth.finalize_sound("block_place", synth.mix_tracks([canvas], weights=[1.0], soft_clip_drive=1.25))


def synth_explosion() -> list[float]:
    """Genera la detonación de carga explosiva con onda subgrave y cola de escombros."""
    n = synth.samples_for("explosion")
    sub_wave = synth.apply_envelope(
        synth.freq_sweep(n, 140.0, 28.0, amp=1.0, waveform="sine", exp_curve=True),
        synth.exp_decay_envelope(n, attack_s=0.002, tau_s=0.38),
    )
    punch_wave = synth.apply_envelope(
        synth.freq_sweep(n, 95.0, 24.0, amp=0.8, waveform="triangle", exp_curve=True),
        synth.exp_decay_envelope(n, attack_s=0.0015, tau_s=0.28),
    )
    rumble_low = synth.apply_envelope(
        synth.lowpass_one_pole(synth.white_noise(n, seed=10501, amp=1.0), 420.0),
        synth.exp_decay_envelope(n, attack_s=0.003, tau_s=0.42),
    )
    blast_mid = synth.apply_envelope(
        synth.bandpass_one_pole(synth.white_noise(n, seed=10502, amp=1.0), 400.0, 1800.0),
        synth.exp_decay_envelope(n, attack_s=0.0015, tau_s=0.22),
    )

    tail_rocks = [0.0] * n
    n_crack = 4410
    crack_env = synth.exp_decay_envelope(n_crack, attack_s=0.0005, tau_s=0.016)
    crack_sig = synth.highpass_one_pole(synth.white_noise(n_crack, seed=10503, amp=1.0), 1200.0)
    synth.place_at(tail_rocks, synth.apply_envelope(crack_sig, crack_env), 0, gain=0.95)

    bounces = (
        (14500, 3800, 210.0, 72.0, 0.42, 10511),
        (21800, 3500, 185.0, 64.0, 0.35, 10512),
        (29400, 3300, 235.0, 82.0, 0.28, 10513),
        (36800, 3100, 168.0, 58.0, 0.22, 10514),
        (43200, 3400, 195.0, 68.0, 0.17, 10515),
    )
    for offset, b_len, f0, f1, gain, seed in bounces:
        b_env = synth.exp_decay_envelope(b_len, attack_s=0.001, tau_s=0.030)
        b_thud = synth.freq_sweep(b_len, f0, f1, amp=0.6, waveform="sine", exp_curve=True)
        b_grit = synth.bandpass_one_pole(synth.white_noise(b_len, seed=seed, amp=0.8), 350.0, 2200.0)
        synth.place_at(tail_rocks, synth.apply_envelope([a + b for a, b in zip(b_thud, b_grit)], b_env), offset, gain)

    mixed = synth.mix_tracks(
        [sub_wave, punch_wave, rumble_low, blast_mid, tail_rocks],
        weights=[1.05, 0.70, 0.95, 0.80, 0.75],
        soft_clip_drive=1.65,
    )
    return synth.finalize_sound("explosion", mixed)


def synth_electric_zap() -> list[float]:
    """Genera la descarga de arco voltaico con zumbido de red y chisporroteo FM."""
    n = synth.samples_for("electric_zap")
    main_env = synth.adsr_envelope(n, attack_s=0.004, decay_s=0.08, sustain_level=0.72, release_s=0.11)

    hum_120 = synth.saw_wave(n, 120.0, amp=0.55)
    hum_240 = synth.square_wave(n, 240.0, amp=0.40, duty=0.32)
    grid_hum = synth.apply_envelope([a + b for a, b in zip(hum_120, hum_240)], main_env)

    fm_arc1 = synth.fm_tone(n, carrier_hz=1650.0, mod_hz=115.0, mod_index=6.5, amp=0.75)
    fm_arc2 = synth.fm_tone(n, carrier_hz=2480.0, mod_hz=180.0, mod_index=5.2, amp=0.45)
    arc_fm = synth.apply_envelope([a + b for a, b in zip(fm_arc1, fm_arc2)], main_env)

    raw_sparks = synth.highpass_one_pole(synth.white_noise(n, seed=10601, amp=1.0), 2200.0)
    two_pi = 2.0 * math.pi
    spark_bursts: list[float] = []
    for i, s in enumerate(raw_sparks):
        t = i / synth.SAMPLE_RATE
        lfo = 0.55 + 0.45 * math.sin(two_pi * 29.0 * t) * math.cos(two_pi * 67.0 * t)
        spark_bursts.append(s * (1.0 if lfo > 0.38 else 0.18) * main_env[i])

    mixed = synth.mix_tracks([grid_hum, arc_fm, spark_bursts], weights=[0.65, 0.85, 0.80], soft_clip_drive=1.45)
    return synth.finalize_sound("electric_zap", mixed)


def synth_lava_bubble() -> list[float]:
    """Genera el hervor magmático y el ascenso viscoso de burbujas de roca fundida."""
    n = synth.samples_for("lava_bubble")
    bed_env = synth.adsr_envelope(n, attack_s=0.04, decay_s=0.10, sustain_level=0.72, release_s=0.12)
    magma_noise = synth.lowpass_one_pole(synth.white_noise(n, seed=10701, amp=1.0), 220.0)
    magma_pulse = synth.sine_wave(n, 56.0, amp=0.35)
    magma_bed = synth.apply_envelope([a + b for a, b in zip(magma_noise, magma_pulse)], bed_env)

    bubbles = [0.0] * n
    bubble_specs = (
        (1100, 6800, 95.0, 290.0, 210.0, 74.0, 0.95),
        (8800, 7400, 120.0, 360.0, 245.0, 82.0, 0.88),
        (16200, 6200, 88.0, 255.0, 185.0, 68.0, 0.80),
    )
    for offset, b_len, f_start, f_end, pop_f0, pop_f1, gain in bubble_specs:
        rise_raw = synth.freq_sweep(b_len, f_start, f_end, amp=1.0, waveform="sine", exp_curve=True)
        rise_env = [math.sin(math.pi * (i / max(1, b_len - 1))) ** 1.4 for i in range(b_len)]
        synth.place_at(bubbles, synth.apply_envelope(rise_raw, rise_env), offset, gain=gain)

        pop_len = 1500
        pop_env = synth.exp_decay_envelope(pop_len, attack_s=0.001, tau_s=0.011)
        pop_drop = synth.freq_sweep(pop_len, pop_f0, pop_f1, amp=0.75, waveform="triangle", exp_curve=True)
        synth.place_at(bubbles, synth.apply_envelope(pop_drop, pop_env), offset + int(b_len * 0.78), gain=gain * 0.75)

    mixed = synth.mix_tracks([magma_bed, bubbles], weights=[0.70, 1.0], soft_clip_drive=1.20)
    return synth.finalize_sound("lava_bubble", mixed)


def synth_bucket_line_loop() -> list[float]:
    """Genera el bucle mecánico rítmico de la línea de cangilones (1.50 s, 3 pasos)."""
    n = synth.samples_for("bucket_line_loop")
    step_len = n // 3  # 22050 muestras (0.50 s por cangilón)

    # Frecuencias de motor múltiplos exactos de 1 / 1.50 s para fase continua en el bucle
    motor_40 = synth.sine_wave(n, 40.0, amp=0.50)
    motor_80 = synth.triangle_wave(n, 80.0, amp=0.35)
    motor_160 = synth.sine_wave(n, 160.0, amp=0.18)
    motor_bed = [a + b + c for a, b, c in zip(motor_40, motor_80, motor_160)]

    # Cable tractor con ruido periódico por tercio y modulación a 2 Hz y 4 Hz
    cable_step = synth.white_noise(step_len, seed=10801, amp=1.0)
    cable_raw = synth.bandpass_one_pole(cable_step * 3, 350.0, 1600.0)
    two_pi = 2.0 * math.pi
    cable_track = [
        s * (0.62 + 0.28 * math.sin(two_pi * 2.0 * (i / synth.SAMPLE_RATE))
             + 0.10 * math.cos(two_pi * 4.0 * (i / synth.SAMPLE_RATE)))
        for i, s in enumerate(cable_raw[:n])
    ]

    pulley_track = [0.0] * n
    clank_len = 4800
    clank_env = synth.exp_decay_envelope(clank_len, attack_s=0.001, tau_s=0.026)
    roller_thud = synth.freq_sweep(clank_len, 138.0, 58.0, amp=0.75, waveform="sine", exp_curve=True)
    pulley_fm = synth.fm_tone(clank_len, carrier_hz=680.0, mod_hz=1020.0, mod_index=1.9, amp=0.65)
    pulley_grit = synth.bandpass_one_pole(synth.white_noise(clank_len, seed=10802, amp=0.55), 700.0, 2600.0)
    main_clank = synth.apply_envelope([a + b + c for a, b, c in zip(roller_thud, pulley_fm, pulley_grit)], clank_env)

    ratchet_len = 2600
    ratchet_env = synth.exp_decay_envelope(ratchet_len, attack_s=0.0008, tau_s=0.015)
    ratchet_fm = synth.fm_tone(ratchet_len, carrier_hz=1120.0, mod_hz=1680.0, mod_index=1.4, amp=0.45)
    sub_clank = synth.apply_envelope(ratchet_fm, ratchet_env)

    for step_idx in range(3):
        base_offset = step_idx * step_len
        synth.place_at(pulley_track, main_clank, base_offset, gain=0.90)
        synth.place_at(pulley_track, sub_clank, base_offset + (step_len // 2), gain=0.45)

    mixed = synth.mix_tracks([motor_bed, cable_track, pulley_track], weights=[0.55, 0.50, 0.85], soft_clip_drive=1.25)
    return synth.finalize_sound("bucket_line_loop", mixed)


def synth_footstep(variant: int) -> list[float]:
    """Genera una de las 4 variantes de pisada de bota minera sobre grava y pizarra."""
    if variant not in _FOOTSTEP_VARIANTS:
        raise ValueError(f"Variante de pisada inválida: {variant}")
    sound_id = f"footstep_{variant}"
    n = synth.samples_for(sound_id)
    seed, heel_f0, heel_f1, toe_offset, toe_f0, bp_lo, bp_hi, gravel_tau = _FOOTSTEP_VARIANTS[variant]

    canvas = [0.0] * n
    heel_len = 3600
    heel_env = synth.exp_decay_envelope(heel_len, attack_s=0.001, tau_s=0.025)
    heel_thud = synth.freq_sweep(heel_len, heel_f0, heel_f1, amp=0.95, waveform="sine", exp_curve=True)
    synth.place_at(canvas, synth.apply_envelope(heel_thud, heel_env), 0, gain=1.0)

    toe_len = n - toe_offset
    toe_env = synth.exp_decay_envelope(toe_len, attack_s=0.0015, tau_s=gravel_tau)
    toe_scuff = synth.freq_sweep(toe_len, toe_f0, heel_f1 * 1.15, amp=0.45, waveform="triangle", exp_curve=True)
    synth.place_at(canvas, synth.apply_envelope(toe_scuff, toe_env), toe_offset, gain=0.75)

    gravel_raw = synth.bandpass_one_pole(synth.white_noise(n, seed=seed, amp=1.0), bp_lo, bp_hi)
    gravel = synth.apply_envelope(gravel_raw, synth.exp_decay_envelope(n, attack_s=0.002, tau_s=gravel_tau))

    mixed = synth.mix_tracks([canvas, gravel], weights=[0.90, 0.80], soft_clip_drive=1.20)
    return synth.finalize_sound(sound_id, mixed)


def generate_tools_sfx() -> dict[str, list[float]]:
    """Genera y valida los 12 efectos de sonido de herramientas y bloques."""
    sounds: dict[str, list[float]] = {
        "pick_hit_rock": synth_pick_hit_rock(),
        "pick_hit_dirt": synth_pick_hit_dirt(),
        "block_break": synth_block_break(),
        "block_place": synth_block_place(),
        "explosion": synth_explosion(),
        "electric_zap": synth_electric_zap(),
        "lava_bubble": synth_lava_bubble(),
        "bucket_line_loop": synth_bucket_line_loop(),
        "footstep_1": synth_footstep(1),
        "footstep_2": synth_footstep(2),
        "footstep_3": synth_footstep(3),
        "footstep_4": synth_footstep(4),
    }
    footsteps = [sounds[f"footstep_{i}"] for i in range(1, 5)]
    for i in range(len(footsteps)):
        for j in range(i + 1, len(footsteps)):
            if footsteps[i] == footsteps[j]:
                raise AssertionError(f"footstep_{i + 1} y footstep_{j + 1} son idénticos.")
    return {snd_id: sounds[snd_id] for snd_id in TOOLS_SOUND_IDS}
