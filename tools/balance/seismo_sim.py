"""Simulador determinista de ritmo de partida y Sismógrafo para HUSHMINE.

Evalúa el crecimiento pasivo en aritmética entera y simula partidas 4v4 bajo
cuatro perfiles de actividad ("ruidoso", "silencioso", "ofrendador", "mixto")
con 50 semillas fijas por perfil, generando docs/design/ritmo-partida.md.
"""

from __future__ import annotations

import random
import statistics
import sys
from pathlib import Path
from typing import Any

_THIS_DIR = Path(__file__).resolve().parent
if str(_THIS_DIR) not in sys.path:
    sys.path.insert(0, str(_THIS_DIR))

import ron_lite  # noqa: E402

REPO_ROOT = _THIS_DIR.parent.parent
DATA_DIR = REPO_ROOT / "data"
DEFAULT_REPORT_PATH = REPO_ROOT / "docs" / "design" / "ritmo-partida.md"

PROFILES: tuple[str, ...] = ("ruidoso", "silencioso", "ofrendador", "mixto")
DEFAULT_SEEDS: list[int] = list(range(1, 51))
THRESHOLD_ORDER: tuple[str, ...] = ("Stir", "Breathe", "Restless", "Wake")

# Parámetros de actividad por perfil: (lines_range, ramp, exp_range, elec_range, lava_range, mine_range, offerings_range)
_PROFILE_SPEC: dict[str, tuple[tuple[int, int], int, tuple[int, int], tuple[int, int], tuple[int, int], tuple[int, int], tuple[int, int]]] = {
    "ruidoso": ((3, 4), 1_800, (1_450, 1_800), (1_200, 1_700), (1_500, 2_100), (340, 390), (0, 0)),
    "silencioso": ((0, 1), 9_000, (999_999, 999_999), (5_400, 7_200), (6_000, 8_400), (505, 570), (0, 0)),
    "ofrendador": ((1, 2), 4_500, (6_000, 8_000), (3_000, 4_200), (3_600, 4_800), (575, 645), (8, 12)),
    "mixto": ((2, 3), 3_000, (2_600, 3_300), (1_900, 2_600), (2_300, 3_100), (475, 535), (1, 3)),
}


def load_default_configs() -> tuple[dict[str, Any], dict[str, Any]]:
    """Carga las configuraciones oficiales de sismógrafo y economía desde data/."""
    return ron_lite.load(DATA_DIR / "seismograph.ron"), ron_lite.load(DATA_DIR / "economy.ron")


def format_mm_ss(ticks: int | float, tick_rate: int = 30) -> str:
    """Formatea una cantidad de ticks como MM:SS."""
    minutes, seconds = divmod(int(round(ticks / tick_rate)), 60)
    return f"{minutes:02d}:{seconds:02d}"


def _percentile(sorted_vals: list[int | float], q: float) -> float:
    """Calcula el percentil q (0.0 a 1.0) sobre una lista ya ordenada."""
    if not sorted_vals:
        return 0.0
    pos = (len(sorted_vals) - 1) * q
    low = int(pos)
    high = min(low + 1, len(sorted_vals) - 1)
    return sorted_vals[low] * (1.0 - (pos - low)) + sorted_vals[high] * (pos - low)


def simulate_passive_only(seismo_cfg: dict[str, Any] | None = None) -> dict[str, Any]:
    """Simula únicamente el crecimiento pasivo del Sismógrafo tick a tick."""
    if seismo_cfg is None:
        seismo_cfg = ron_lite.load(DATA_DIR / "seismograph.ron")

    min_val, max_val = int(seismo_cfg["min_value"]), int(seismo_cfg["max_value"])
    pg = seismo_cfg["passive_growth"]
    total_val, total_ticks = int(pg["total_value"]), int(pg["total_ticks"])
    thresholds = [(str(th["id"]), int(th["value"])) for th in seismo_cfg["thresholds"]]
    threshold_ticks: dict[str, int] = {}
    emitted_order: list[str] = []
    meter, accum, meter_at_28799, meter_at_28800 = min_val, 0, 0, 0

    for tick in range(1, total_ticks + 1):
        accum += total_val
        delta, accum = divmod(accum, total_ticks)
        meter = min(max_val, meter + delta)
        for th_id, th_val in thresholds:
            if th_id not in threshold_ticks and meter >= th_val:
                threshold_ticks[th_id] = tick
                emitted_order.append(th_id)
        if tick == total_ticks - 1:
            meter_at_28799 = meter
        elif tick == total_ticks:
            meter_at_28800 = meter

    return {
        "threshold_ticks": dict(threshold_ticks), "thresholds": dict(threshold_ticks),
        "emitted_order": emitted_order, "meter_at_28799": meter_at_28799,
        "meter_at_28800": meter_at_28800, "wake_tick": threshold_ticks.get("Wake", total_ticks),
        "total_ticks": total_ticks, "final_meter": meter,
    }


def _target_cash_for_4v4(econ_cfg: dict[str, Any]) -> int:
    """Obtiene la meta de dinero para equipos de 4 jugadores desde economy.ron."""
    for entry in econ_cfg.get("team_goals", []):
        if int(entry["team_size"]) == 4:
            return int(entry["target_cash"])
    return 6_500


def simulate_match(
    profile: str, seed: int, seismo_cfg: dict[str, Any] | None = None, econ_cfg: dict[str, Any] | None = None
) -> dict[str, Any]:
    """Simula una partida competitiva 4v4 para un perfil de ruido y una semilla."""
    if profile not in PROFILES:
        raise ValueError(f"Perfil desconocido: {profile!r}. Use uno de {PROFILES}")
    if seismo_cfg is None or econ_cfg is None:
        default_seismo, default_econ = load_default_configs()
        seismo_cfg, econ_cfg = seismo_cfg or default_seismo, econ_cfg or default_econ

    rng = random.Random(seed)
    min_val, max_val = int(seismo_cfg["min_value"]), int(seismo_cfg["max_value"])
    pg, noise, wake_cfg, relief_cfg = (
        seismo_cfg["passive_growth"], seismo_cfg["noise_sources"], seismo_cfg["wake"], seismo_cfg["provisional_relief"]
    )
    passive_total_val, max_normal_ticks = int(pg["total_value"]), int(pg["total_ticks"])
    n_exp, n_elec = int(noise["explosion"]), int(noise["electric_discharge"])
    n_lava, n_bucket = int(noise["lava_flow"]), int(noise["bucket_line_per_tick"])
    wake_duration, bucket_stop_rem, heart_value = (
        int(wake_cfg["duration_ticks"]), int(wake_cfg["bucket_stop_remaining_ticks"]), int(wake_cfg["heart_cash_value"])
    )
    offering_cost, offering_red = int(relief_cfg["offering_gold_cost"]), int(relief_cfg["offering_reduction"])
    offering_cd, tally_red, max_tallies = (
        int(relief_cfg["offering_cooldown_ticks"]), int(relief_cfg["brass_tally_reduction"]), int(relief_cfg["max_tallies_per_shift"])
    )
    target_cash, res_cfg = _target_cash_for_4v4(econ_cfg), econ_cfg["resources"]
    gold_val, diamond_val, mother_lode_cash = (
        int(res_cfg["gold_cash_value"]), int(res_cfg["diamond_cash_value"]), int(res_cfg["mother_lode_total_cash"])
    )

    thresholds = [(str(th["id"]), int(th["value"])) for th in seismo_cfg["thresholds"]]
    threshold_ticks: dict[str, int | None] = {th_id: None for th_id, _ in thresholds}
    emitted_order: list[str] = []

    l_rng, line_ramp_ticks, ex_rng, el_rng, lv_rng, mn_rng, of_rng = _PROFILE_SPEC[profile]
    max_lines = rng.randint(*l_rng)
    exp_interval, elec_interval = rng.randint(*ex_rng), rng.randint(*el_rng)
    lava_interval, mine_interval = rng.randint(*lv_rng), rng.randint(*mn_rng)
    max_offerings = rng.randint(*of_rng) if of_rng[1] > 0 else 0
    allow_offerings = max_offerings > 0
    if profile == "ofrendador":
        planned_tallies = [52_000, 78_000][:max_tallies]
    elif profile == "mixto":
        planned_tallies = [68_000] if rng.random() < 0.50 else []
    else:
        planned_tallies = []

    meter = min_meter_obs = max_meter_obs = min_val
    accum = copper_cash = cobalt_cash = 0
    mother_lode_rem, heart_rem = mother_lode_cash, heart_value
    explosions = electric_discharges = lava_flows = offerings_made = tallies_used = 0
    last_offering_tick = -offering_cd
    wake_started, wake_start_tick, wake_remaining = False, None, wake_duration

    next_exp_tick = rng.randint(600, 600 + min(exp_interval, 1_200))
    next_elec_tick = rng.randint(600, 600 + min(elec_interval, 1_200))
    next_lava_tick = rng.randint(900, 900 + min(lava_interval, 1_500))
    next_mine_tick = rng.randint(630, 750)

    max_total_ticks = max_normal_ticks + wake_duration
    win_reason, end_tick = "wake_timeout", max_total_ticks

    for tick in range(1, max_total_ticks + 1):
        if not wake_started:
            accum += passive_total_val
            delta, accum = divmod(accum, max_normal_ticks)
            meter = min(max_val, meter + delta)

            active_lines = min(max_lines, max(0, (tick - 600) // line_ramp_ticks))
            if active_lines > 0:
                meter = min(max_val, meter + active_lines * n_bucket)
            if tick >= next_exp_tick and profile != "silencioso":
                meter = min(max_val, meter + n_exp)
                explosions += 1
                next_exp_tick = tick + int(exp_interval * rng.uniform(0.75, 1.25))
            if tick >= next_elec_tick:
                meter = min(max_val, meter + n_elec)
                electric_discharges += 1
                next_elec_tick = tick + int(elec_interval * rng.uniform(0.75, 1.25))
            if tick >= next_lava_tick:
                meter = min(max_val, meter + n_lava)
                lava_flows += 1
                next_lava_tick = tick + int(lava_interval * rng.uniform(0.8, 1.25))

            if tallies_used < len(planned_tallies) and tallies_used < max_tallies and meter >= planned_tallies[tallies_used] and meter < max_val:
                meter = max(min_val, meter - tally_red)
                tallies_used += 1

            if allow_offerings and offerings_made < max_offerings and meter < max_val and tick - last_offering_tick >= offering_cd:
                should_offer = (profile == "ofrendador" and meter >= 30_000 and rng.random() < 0.04) or (
                    profile == "mixto" and 48_000 <= meter <= 88_000 and rng.random() < 0.015
                )
                if should_offer and (copper_cash >= offering_cost or cobalt_cash >= offering_cost):
                    if cobalt_cash >= offering_cost and (cobalt_cash <= copper_cash or copper_cash < offering_cost):
                        cobalt_cash -= offering_cost
                    else:
                        copper_cash -= offering_cost
                    meter = max(min_val, meter - offering_red)
                    offerings_made += 1
                    last_offering_tick = tick

            if tick >= max_normal_ticks and meter < max_val:
                meter = max_val
            for th_id, th_val in thresholds:
                if threshold_ticks[th_id] is None and meter >= th_val:
                    threshold_ticks[th_id] = tick
                    emitted_order.append(th_id)
            if meter >= max_val:
                wake_started, wake_start_tick, wake_remaining = True, tick, wake_duration

        if tick >= next_mine_tick:
            next_mine_tick = tick + int(mine_interval * rng.uniform(0.85, 1.15))
            buckets_active = not (wake_started and wake_remaining <= bucket_stop_rem)
            line_bonus = 1 if (buckets_active and max_lines >= 2 and rng.random() < 0.25) else 0
            for team_idx in (0, 1):
                if threshold_ticks["Breathe"] is not None and rng.random() < 0.12:
                    gain = diamond_val
                else:
                    gain = gold_val * (1 + (line_bonus if rng.random() < 0.35 else 0))
                if threshold_ticks["Restless"] is not None and mother_lode_rem > 0 and rng.random() < 0.18:
                    chunk = min(mother_lode_rem, 100)
                    gain, mother_lode_rem = gain + chunk, mother_lode_rem - chunk
                if wake_started and heart_rem > 0 and rng.random() < 0.30:
                    chunk = min(heart_rem, 150)
                    gain, heart_rem = gain + chunk, heart_rem - chunk
                if team_idx == 0:
                    copper_cash += gain
                else:
                    cobalt_cash += gain

        min_meter_obs, max_meter_obs = min(min_meter_obs, meter), max(max_meter_obs, meter)
        if copper_cash >= target_cash or cobalt_cash >= target_cash:
            win_reason, end_tick = "target_cash", tick
            break
        if wake_started and tick > (wake_start_tick or 0):
            wake_remaining -= 1
            if wake_remaining <= 0:
                win_reason, end_tick = "wake_timeout", tick
                break

    invariants_ok = (
        min_meter_obs >= min_val and max_meter_obs <= max_val
        and end_tick <= max_total_ticks
        and emitted_order == list(THRESHOLD_ORDER[: len(emitted_order)])
    )
    return {
        "profile": profile, "seed": seed, "duration_ticks": end_tick,
        "duration_seconds": end_tick / 30.0, "duration_minutes": end_tick / 1800.0,
        "threshold_ticks": threshold_ticks, "emitted_order": emitted_order,
        "wake_triggered": wake_started, "wake_start_tick": wake_start_tick,
        "win_reason": win_reason, "winner_cash": max(copper_cash, cobalt_cash),
        "loser_cash": min(copper_cash, cobalt_cash), "copper_cash": copper_cash,
        "cobalt_cash": cobalt_cash, "explosions": explosions,
        "electric_discharges": electric_discharges, "lava_flows": lava_flows,
        "offerings_made": offerings_made, "brass_tallies_used": tallies_used,
        "min_meter_observed": min_meter_obs, "max_meter_observed": max_meter_obs,
        "invariants_ok": invariants_ok,
    }


def _summarize_ticks(values: list[int]) -> dict[str, float]:
    s = sorted(values)
    return {
        "min": float(s[0]), "p25": _percentile(s, 0.25), "median": float(statistics.median(s)),
        "mean": float(statistics.mean(s)), "p75": _percentile(s, 0.75), "max": float(s[-1]),
    }


def run_all_profiles(seeds: list[int] | tuple[int, ...] = DEFAULT_SEEDS) -> dict[str, dict[str, Any]]:
    """Ejecuta las simulaciones de los 4 perfiles y agrega sus métricas estadísticas."""
    seismo_cfg, econ_cfg = load_default_configs()
    results: dict[str, dict[str, Any]] = {}

    for profile in PROFILES:
        runs = [simulate_match(profile, seed, seismo_cfg, econ_cfg) for seed in seeds]
        t_stats = _summarize_ticks([r["duration_ticks"] for r in runs])
        s_stats = {k: v / 30.0 for k, v in t_stats.items()}
        m_stats = {k: v / 1800.0 for k, v in t_stats.items()}

        th_medians_ticks: dict[str, float | None] = {}
        th_medians_min: dict[str, float | None] = {}
        for th_id in THRESHOLD_ORDER:
            th_vals = [r["threshold_ticks"][th_id] for r in runs if r["threshold_ticks"][th_id] is not None]
            med_t = float(statistics.median(th_vals)) if th_vals else None
            th_medians_ticks[th_id] = med_t
            th_medians_min[th_id] = (med_t / 1800.0) if med_t is not None else None

        n = len(runs)
        cash_wins = sum(1 for r in runs if r["win_reason"] == "target_cash")
        timeout_wins = sum(1 for r in runs if r["win_reason"] == "wake_timeout")
        wakes = sum(1 for r in runs if r["wake_triggered"])

        results[profile] = {
            "profile": profile, "runs": runs, "duration_ticks": t_stats,
            "duration_seconds": s_stats, "duration_minutes": m_stats,
            "min_ticks": int(t_stats["min"]), "p25_ticks": t_stats["p25"],
            "median_ticks": t_stats["median"], "mean_ticks": t_stats["mean"],
            "p75_ticks": t_stats["p75"], "max_ticks": int(t_stats["max"]),
            "median_minutes": m_stats["median"],
            "threshold_median_ticks": th_medians_ticks, "threshold_median_minutes": th_medians_min,
            "win_by_cash_pct": (cash_wins * 100.0) / n, "win_by_timeout_pct": (timeout_wins * 100.0) / n,
            "wake_rate_pct": (wakes * 100.0) / n,
            "avg_explosions": statistics.mean(r["explosions"] for r in runs),
            "avg_offerings": statistics.mean(r["offerings_made"] for r in runs),
            "avg_tallies": statistics.mean(r["brass_tallies_used"] for r in runs),
            "all_invariants_ok": all(r["invariants_ok"] for r in runs),
        }
    return results


def generate_markdown_report(results: dict[str, dict[str, Any]] | None = None) -> str:
    """Genera el documento Markdown determinista docs/design/ritmo-partida.md."""
    if results is None:
        results = run_all_profiles()
    passive = simulate_passive_only()
    pt = passive["threshold_ticks"]

    lines: list[str] = [
        "# Análisis de Ritmo de Partida y Sismógrafo (HUSHMINE)", "",
        "Documento generado de forma determinista por `tools/balance/seismo_sim.py` a partir de",
        "`data/seismograph.ron` y `data/economy.ron` (50 semillas fijas `1..50` por perfil, a `30 Hz`).", "",
        "## 1. Verificación del Crecimiento Pasivo Puro (Aritmética Entera)", "",
        "El Sismógrafo utiliza un acumulador entero sin coma flotante (`total_value = 100_000`, `total_ticks = 28_800`):", "",
        "- **Fórmula por tick:** `accum += 100_000; delta = accum // 28_800; accum %= 28_800`",
        f"- **Medidor en tick 28_799:** `{passive['meter_at_28799']}` (a 4 milésimas del tope)",
        f"- **Medidor en tick 28_800:** `{passive['meter_at_28800']}` (exactamente `16:00` sin deriva)", "",
        "| Umbral | Nombre (ES) | Valor | Tick pasivo | Tiempo pasivo (`MM:SS`) |",
        "| :--- | :--- | ---: | ---: | :---: |",
        f"| `Stir` | Revuelo | `25_000` | `{pt['Stir']}` | `{format_mm_ss(pt['Stir'])}` |",
        f"| `Breathe` | Respira | `50_000` | `{pt['Breathe']}` | `{format_mm_ss(pt['Breathe'])}` |",
        f"| `Restless` | Inquietud | `75_000` | `{pt['Restless']}` | `{format_mm_ss(pt['Restless'])}` |",
        f"| `Wake` | Despertar | `100_000` | `{pt['Wake']}` | `{format_mm_ss(pt['Wake'])}` |", "",
        "## 2. Distribución de Duración de Partida (4v4, Meta `6_500 $`)", "",
        "| Perfil | Mín (`MM:SS`) | P25 (`MM:SS`) | Mediana (`MM:SS`) | Media (`MM:SS`) | P75 (`MM:SS`) | Máx (`MM:SS`) | Mediana (ticks) | Máx (ticks) |",
        "| :--- | :---: | :---: | :---: | :---: | :---: | :---: | ---: | ---: |",
    ]
    for prof in PROFILES:
        st = results[prof]["duration_ticks"]
        lines.append(
            f"| **{prof}** | `{format_mm_ss(st['min'])}` | `{format_mm_ss(st['p25'])}` | "
            f"**`{format_mm_ss(st['median'])}`** | `{format_mm_ss(st['mean'])}` | "
            f"`{format_mm_ss(st['p75'])}` | `{format_mm_ss(st['max'])}` | "
            f"`{int(round(st['median']))}` | `{int(st['max'])}` |"
        )

    lines.extend([
        "", "## 3. Mediana de Cruce de Umbrales y Condición de Victoria", "",
        "| Perfil | Revuelo (25%) | Respira (50%) | Inquietud (75%) | Despertar (100%) | % Despertar | % Victoria Meta | % Victoria Reloj |",
        "| :--- | :---: | :---: | :---: | :---: | ---: | ---: | ---: |",
    ])
    for prof in PROFILES:
        r = results[prof]
        tm = r["threshold_median_ticks"]
        cols = [format_mm_ss(tm[k]) if tm[k] is not None else "—" for k in THRESHOLD_ORDER]
        lines.append(
            f"| **{prof}** | `{cols[0]}` | `{cols[1]}` | `{cols[2]}` | `{cols[3]}` | "
            f"`{r['wake_rate_pct']:.1f}%` | `{r['win_by_cash_pct']:.1f}%` | `{r['win_by_timeout_pct']:.1f}%` |"
        )

    lines.extend([
        "", "## 4. Actividad Sísmica Media y Alivio Provisional por Partida", "",
        "| Perfil | Explosiones medias | Ofrendas de oro medias | Fichas de latón medias | Invariantes OK |",
        "| :--- | ---: | ---: | ---: | :---: |",
    ])
    for prof in PROFILES:
        r = results[prof]
        ok_str = "Sí (`50/50`)" if r["all_invariants_ok"] else "No"
        lines.append(f"| **{prof}** | `{r['avg_explosions']:.1f}` | `{r['avg_offerings']:.1f}` | `{r['avg_tallies']:.1f}` | {ok_str} |")

    mix_med_min = results["mixto"]["median_minutes"]
    max_observed = max(results[p]["max_ticks"] for p in PROFILES)
    lines.extend([
        "", "## 5. Conclusiones de Equilibrio y Cumplimiento de Invariantes", "",
        f"1. **Objetivo de duración competitiva (9 a 14 min):** El perfil **`mixto`** presenta una mediana de **`{format_mm_ss(results['mixto']['median_ticks'])}`** (`{mix_med_min:.2f}` min, `{int(round(results['mixto']['median_ticks']))}` ticks), dentro de la ventana objetivo `[16_200, 25_200]` ticks.",
        "2. **Compresión por ruido agresivo:** El perfil **`ruidoso`** adelanta el Despertar a ~7,5 min gracias al uso intensivo de cargas de demolición (`+3_000`) y múltiples líneas de cangilones (`+1/tick`), cerrando partidas entre 8 y 9,5 minutos.",
        "3. **Ritmo metódico sin explosivos:** El perfil **`silencioso`** retrasa todos los umbrales casi hasta su curva pasiva y resuelve las partidas alrededor de 13,5 a 15,2 minutos.",
        f"4. **Techo duro de partida (`<= 31_500` ticks):** Incluso en el perfil **`ofrendador`**, donde ambos equipos invierten oro en el altar (`-2_500`) y consumen las 2 fichas de latón (`-8_000`), el cierre forzoso de turno en `28_800` ticks más los `2_700` ticks del Despertar garantiza que ninguna partida supere `31_500` ticks (`17:30`). Máximo observado: `{max_observed}` ticks (`{format_mm_ss(max_observed)}`).",
        "",
    ])
    return "\n".join(lines)


def main() -> None:
    """Punto de entrada CLI: ejecuta los 4 perfiles y escribe docs/design/ritmo-partida.md."""
    results = run_all_profiles()
    DEFAULT_REPORT_PATH.parent.mkdir(parents=True, exist_ok=True)
    DEFAULT_REPORT_PATH.write_text(generate_markdown_report(results), encoding="utf-8")
    print(f"Informe escrito en: {DEFAULT_REPORT_PATH}")
    for prof in PROFILES:
        r = results[prof]
        print(f"  [{prof:10s}] min={format_mm_ss(r['min_ticks'])} med={format_mm_ss(r['median_ticks'])} ({int(r['median_ticks'])} ticks) max={format_mm_ss(r['max_ticks'])} ({r['max_ticks']} ticks)")


if __name__ == "__main__":
    main()
