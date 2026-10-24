"""Generador determinista de las 5 maquetas SVG de interfaz de HUSHMINE en 1920x1080 y 1280x800.

Produce las 10 vistas vectoriales autosuficientes (`hud`, `drop_in`, `chronicle`,
`main_menu`, `loading`) con estética industrial 1931/1979, comentarios de localización
Fluent (`<!-- ftl: ... -->`) y contraste verificado WCAG 2.1 AA (>= 4.5:1).
"""

from __future__ import annotations

import sys
from pathlib import Path

REPO_ROOT: Path = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.ui.ui_palette import (  # noqa: E402
    MOCKUP_SCREENS,
    SAFE_MARGINS,
    SCREEN_RESOLUTIONS,
    check_wcag_aa,
    ui_hex,
)

# Directorio canónico de salida para las 10 maquetas SVG.
MOCKUPS_DIR: Path = REPO_ROOT / "assets" / "ui" / "mockups"

# Colores frecuentes de la paleta cerrada 1931/1979.
C_VOID, C_SOOT, C_CRT, C_GRID = ui_hex("void_black"), ui_hex("soot_dark"), ui_hex("crt_bg"), ui_hex("crt_grid")
C_PHOS_MID, C_PHOS, C_PEAK = ui_hex("phosphor_mid"), ui_hex("phosphor_bright"), ui_hex("phosphor_peak")
C_CREAM, C_PAPER, C_INK = ui_hex("enamel_cream"), ui_hex("paper_aged"), ui_hex("ink_stamp")
C_AMBER, C_YEL, C_GOLD = ui_hex("sodium_warm"), ui_hex("lava_yellow"), ui_hex("gold_gleam")
C_COP_B, C_COP_G = ui_hex("copper_bright"), ui_hex("copper_glow")
C_COB_B, C_COB_G = ui_hex("cobalt_bright"), ui_hex("cobalt_glow")
C_BRASS_D, C_BRASS_M, C_BRASS = ui_hex("brass_dark"), ui_hex("brass_mid"), ui_hex("brass_bright")
C_ICE = ui_hex("diamond_white")


def _rect(
    x: int, y: int, w: int, h: int, fill: str, stroke: str | None = None, stroke_width: int = 2, rx: int = 4
) -> str:
    """Genera un elemento SVG <rect> con colores validados de la paleta."""
    stroke_attr = f' stroke="{stroke}" stroke-width="{stroke_width}"' if stroke else ""
    rx_attr = f' rx="{rx}"' if rx > 0 else ""
    return f'  <rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{fill}"{stroke_attr}{rx_attr}/>'


def _text(
    x: int, y: int, content: str, fill: str, bg: str, size: int = 16, anchor: str = "start", ftl: str | None = None
) -> str:
    """Genera un elemento SVG <text> precedido opcionalmente por su comentario Fluent."""
    if not check_wcag_aa(fill, bg, 4.5):
        raise ValueError(f"Contraste WCAG insuficiente para {content!r}: {fill} sobre {bg}")
    safe_xml = content.replace("&", "&amp;").replace("<", "&lt;")
    line = (
        f'  <text x="{x}" y="{y}" fill="{fill}" data-bg="{bg}" '
        f'font-family="monospace" font-size="{size}" font-weight="bold" '
        f'text-anchor="{anchor}">{safe_xml}</text>'
    )
    return f"  <!-- ftl: {ftl} -->\n{line}" if ftl else line


def _wave_points(x0: int, y_mid: int, width: int, amp: int) -> str:
    """Genera los puntos de una onda de sismógrafo determinista para <polyline>."""
    offsets = (0, -2, 3, -5, 7, -11, 15, -18, 12, -8, 19, -22, 14, -9, 6, -4, 2, -1, 0)
    steps = len(offsets) - 1
    return " ".join(
        f"{x0 + (width * i) // steps},{y_mid + (f * amp) // 22}" for i, f in enumerate(offsets)
    )


def _render_hud(width: int, height: int, m: int) -> list[str]:
    """Construye las capas SVG del HUD en partida (3 niveles jerárquicos)."""
    uw, uh = width - 2 * m, height - 2 * m
    s = 1.0 if width >= 1920 else 0.78
    f_lg, f_md, f_sm = max(18, int(20 * s)), max(15, int(16 * s)), max(14, int(14 * s))
    top_h, bank_w = int(118 * s), int(uw * 0.27)
    seis_x, seis_w = m + int(uw * 0.29), int(uw * 0.42)
    cob_x, cob_w = m + int(uw * 0.73), uw - int(uw * 0.73)
    bot_h, bot_y = int(172 * s), m + uh - int(172 * s)
    left_w, right_w = int(uw * 0.40), int(uw * 0.42)
    right_x = m + uw - right_w

    return [
        _rect(0, 0, width, height, C_VOID, rx=0),
        f'  <line x1="{m}" y1="{height // 2}" x2="{width - m}" y2="{height // 2}" stroke="{C_SOOT}" stroke-width="1"/>',
        f'  <line x1="{width // 2 - 10}" y1="{height // 2}" x2="{width // 2 + 10}" y2="{height // 2}" stroke="{C_CREAM}" stroke-width="2"/>',
        f'  <line x1="{width // 2}" y1="{height // 2 - 10}" x2="{width // 2}" y2="{height // 2 + 10}" stroke="{C_CREAM}" stroke-width="2"/>',
        # Nivel 1 Izquierda: Banco Copper Crew (chevrones diagonales naranja)
        _rect(m, m, bank_w, top_h, C_SOOT, C_COP_B, 2),
        f'  <path d="M {m + 14} {m + 20} L {m + 26} {m + 32} L {m + 14} {m + 44} M {m + 24} {m + 20} L {m + 36} {m + 32} L {m + 24} {m + 44}" fill="none" stroke="{C_COP_G}" stroke-width="3"/>',
        _text(m + 46, m + int(30 * s), "COPPER CREW [CHEVRON]", C_COP_G, C_SOOT, f_md, ftl="hud-bank-copper"),
        _text(m + 46, m + int(62 * s), "$4,250 / $6,500", C_CREAM, C_SOOT, f_lg),
        _text(m + 46, m + int(94 * s), "FAME ACTIVE ≥60% (>=60%)", C_YEL, C_SOOT, f_sm, ftl="hud-fame-badge"),
        # Nivel 1 Centro: Sismógrafo compartido CRT
        _rect(seis_x, m, seis_w, top_h, C_CRT, C_PHOS_MID, 2),
        f'  <line x1="{seis_x + 12}" y1="{m + top_h // 2}" x2="{seis_x + seis_w - 12}" y2="{m + top_h // 2}" stroke="{C_GRID}" stroke-width="1"/>',
        f'  <polyline fill="none" stroke="{C_PHOS}" stroke-width="2" points="{_wave_points(seis_x + 16, m + int(56 * s), seis_w - 32, int(20 * s))}"/>',
        "  <!-- ftl: hud-seismograph-title -->",
        _text(seis_x + 16, m + int(26 * s), "SEISMOGRAPH · 54.2% [BREATHE]", C_PHOS, C_CRT, f_md, ftl="hud-seismograph"),
        _text(seis_x + 16, m + int(84 * s), "25% STIR | 50% BREATHE | 75% RESTLESS | 100% WAKE", C_PEAK, C_CRT, f_sm),
        _text(seis_x + 16, m + int(106 * s), "BUCKET LINES: 3 ACTIVE", C_AMBER, C_CRT, f_sm, ftl="hud-bucket-lines"),
        # Nivel 1 Derecha: Banco Cobalt Crew (cruces ortogonales + azul cobalto)
        _rect(cob_x, m, cob_w, top_h, C_SOOT, C_COB_B, 2),
        f'  <path d="M {cob_x + 22} {m + 18} V {m + 42} M {cob_x + 10} {m + 30} H {cob_x + 34}" fill="none" stroke="{C_COB_G}" stroke-width="3"/>',
        _text(cob_x + 44, m + int(30 * s), "COBALT CREW [+ CROSS]", C_COB_G, C_SOOT, f_md, ftl="hud-bank-cobalt"),
        _text(cob_x + 44, m + int(62 * s), "$3,100 / $6,500", C_CREAM, C_SOOT, f_lg),
        _text(cob_x + 44, m + int(94 * s), "FAME INACTIVE (<60%)", C_ICE, C_SOOT, f_sm),
        # Nivel 2: Aviso de radio del Vigía bajo el Sismógrafo
        _rect(seis_x, m + top_h + 10, seis_w, int(58 * s), C_CRT, C_AMBER, 2),
        _text(seis_x + 16, m + top_h + int(30 * s), "WATCHER RADIO · 4.82 MHz PTT", C_AMBER, C_CRT, f_sm),
        _text(seis_x + 16, m + top_h + int(54 * s), '"She\'s breathing. Get out."', C_PHOS, C_CRT, f_md, ftl="watcher-breathe-1"),
        # Nivel 3 Izquierda: Clase y herramientas
        _rect(m, bot_y, left_w, bot_h, C_SOOT, C_COP_B, 2),
        _text(m + 18, bot_y + int(28 * s), "CLASS: SAPPER", C_AMBER, C_SOOT, f_lg, ftl="class-sapper"),
        _text(m + 18, bot_y + int(56 * s), "[1] BLASTING CAP   [2] WIRE CUTTERS", C_CREAM, C_SOOT, f_sm),
        _text(m + 18, bot_y + int(84 * s), "[3] TEAM BLOCK (1 ORE)", C_COP_G, C_SOOT, f_sm),
        _text(m + 18, bot_y + int(112 * s), "[4] LADDER (1 ORE)", C_CREAM, C_SOOT, f_sm),
        _text(m + 18, bot_y + int(140 * s), "[5] EXPLOSIVE (10 ORE)", C_YEL, C_SOOT, f_sm),
        # Nivel 3 Derecha: Carga personal, fichas de latón y altar
        _rect(right_x, bot_y, right_w, bot_h, C_SOOT, C_BRASS, 2),
        _text(right_x + 18, bot_y + int(32 * s), "ORE: 14 / 20", C_COP_G, C_SOOT, f_md, ftl="hud-cargo-ore"),
        _text(right_x + 18, bot_y + int(68 * s), "BULLION: 6 / 10 ($800 · STOLEN x1.25)", C_GOLD, C_SOOT, f_md, ftl="hud-cargo-bullion"),
        _text(right_x + 18, bot_y + int(104 * s), "BRASS TALLIES: 1 / 2 (#14 M. ALLER)", C_BRASS, C_SOOT, f_md, ftl="hud-brass-tallies"),
        _text(right_x + 18, bot_y + int(142 * s), "[E] ALTAR OFFERING: $100 -> -2.5% SEISMO", C_PHOS, C_SOOT, f_sm, ftl="hud-altar-offer"),
    ]


def _render_drop_in(width: int, height: int, m: int) -> list[str]:
    """Construye las capas SVG de la pantalla de Elección de clase en la Bajada."""
    uw, uh = width - 2 * m, height - 2 * m
    s = 1.0 if width >= 1920 else 0.76
    f_lg, f_md, f_sm = max(18, int(20 * s)), max(15, int(16 * s)), max(14, int(14 * s))
    hdr_h = int(84 * s)
    lines: list[str] = [
        _rect(0, 0, width, height, C_VOID, rx=0),
        _rect(m, m, uw, hdr_h, C_SOOT, C_AMBER, 2),
        _text(m + 20, m + int(34 * s), "DESCENT CAGE · SHIFT #04 · DROP IN 00:14", C_AMBER, C_SOOT, f_lg, ftl="drop-in-header"),
        _text(m + 20, m + int(64 * s), "COPPER CREW — WEST SHAFT [CHEVRON]", C_COP_G, C_SOOT, f_md),
    ]
    cards = (
        ("class-prospector", "PROSPECTOR", "ORE 20 · BULLION 10 · MINERAL DETECTOR + GEOPHONE", "BUILDS: TEAM BLOCK, LADDER, BEACON"),
        ("class-miner", "MINER", "ORE 40 · BULLION 20 · HEAVY PICKAXE + CARGO HARNESS", "BUILDS: TEAM BLOCK, LADDER, RAIL"),
        ("class-engineer", "ENGINEER", "ORE 30 · BULLION 10 · BUILDER HAMMER + BUCKET WRENCH", "BUILDS: FORCE FIELD, JUMP PAD, ELECTRIC, BUCKET LINE"),
        ("class-sapper", "SAPPER", "ORE 20 · BULLION 10 · BLASTING CAP + WIRE CUTTERS", "BUILDS: TEAM BLOCK, LADDER, EXPLOSIVE"),
    )
    body_y, body_h = m + hdr_h + int(16 * s), uh - hdr_h - int(16 * s)
    left_w, gap = int(uw * 0.57), int(12 * s)
    card_h = (body_h - 3 * gap) // 4
    for idx, (ftl_id, title, stats, builds) in enumerate(cards):
        cy = body_y + idx * (card_h + gap)
        lines.extend([
            _rect(m, cy, left_w, card_h, C_SOOT, C_YEL if idx == 3 else C_COP_B, 2),
            _text(m + 18, cy + int(32 * s), title, C_AMBER, C_SOOT, f_lg, ftl=ftl_id),
            _text(m + 18, cy + int(68 * s), stats, C_CREAM, C_SOOT, f_sm),
            _text(m + 18, cy + int(102 * s), builds, C_PHOS, C_SOOT, f_sm),
        ])
    geo_x, geo_w = m + int(uw * 0.59), uw - int(uw * 0.59)
    lines.extend([
        _rect(geo_x, body_y, geo_w, body_h, C_CRT, C_PHOS_MID, 2),
        _text(geo_x + 18, body_y + int(36 * s), "GEOLOGICAL SURVEY · 64×64×64 (64x64x64) CROSS-SECTION", C_PHOS, C_CRT, f_md, ftl="drop-in-geology"),
        _text(geo_x + 18, body_y + int(86 * s), "UPPER STRATA: DIRT (Y: 48..64)", C_CREAM, C_CRT, f_md),
        _text(geo_x + 18, body_y + int(136 * s), "MID STRATA: ROCK & ORE VEINS (Y: 20..48)", C_COP_G, C_CRT, f_md),
        _text(geo_x + 18, body_y + int(186 * s), "50% BREATHE: MOTHER LODE ($2,000)", C_GOLD, C_CRT, f_md),
        _text(geo_x + 18, body_y + int(236 * s), "DEEP STRATA: HARD ROCK (Y: 00..20)", C_ICE, C_CRT, f_md),
        _text(geo_x + 18, body_y + int(286 * s), "100% WAKE ABYSS: THE HEART ($1,500)", C_YEL, C_CRT, f_md),
        f'  <polyline fill="none" stroke="{C_PHOS}" stroke-width="2" points="{_wave_points(geo_x + 18, body_y + body_h - int(48 * s), geo_w - 36, int(24 * s))}"/>',
    ])
    return lines


def _render_chronicle(width: int, height: int, m: int) -> list[str]:
    """Construye las capas SVG de la Crónica posterior a la partida."""
    uw, uh = width - 2 * m, height - 2 * m
    s = 1.0 if width >= 1920 else 0.76
    f_lg, f_md, f_sm = max(18, int(20 * s)), max(15, int(16 * s)), max(14, int(14 * s))
    hdr_h = int(90 * s)
    body_y, body_h = m + hdr_h + int(16 * s), uh - hdr_h - int(16 * s)
    left_w = int(uw * 0.51)
    lines: list[str] = [
        _rect(0, 0, width, height, C_VOID, rx=0),
        _rect(m, m, uw, hdr_h, C_PAPER, C_BRASS_M, 2),
        _text(m + 24, m + int(36 * s), "POST-SHIFT CHRONICLE · MOUNT ANIMA LOG 1979", C_INK, C_PAPER, f_lg, ftl="chronicle-title"),
        _text(m + 24, m + int(68 * s), "COPPER CREW VICTORY — $6,500 / $6,500 AT 11:24", C_VOID, C_PAPER, f_md),
        _rect(m, body_y, left_w, body_h, C_CRT, C_PHOS_MID, 2),
        _text(m + 18, body_y + int(36 * s), "SEISMOGRAPH TIMELINE & 6 SHIFT MILESTONES", C_PHOS, C_CRT, f_md, ftl="chronicle-event-log"),
        _text(m + 18, body_y + int(84 * s), "02:38 [25% STIR] FIRST TREMORS & WHISTLERS", C_CREAM, C_CRT, f_sm),
        _text(m + 18, body_y + int(126 * s), "04:55 COPPER OFFERING ($100 -> -2.5%)", C_PHOS, C_CRT, f_sm),
        _text(m + 18, body_y + int(168 * s), "05:40 [50% BREATHE] MOTHER LODE CAVERN CRACKED OPEN", C_AMBER, C_CRT, f_sm),
        _text(m + 18, body_y + int(210 * s), "07:15 BRASS TALLY #14 RETURNED (-8.0%)", C_PHOS, C_CRT, f_sm),
        _text(m + 18, body_y + int(252 * s), "09:40 COBALT TRIGGERED 3 CHARGES -> [100% WAKE]", C_YEL, C_CRT, f_sm),
        _text(m + 18, body_y + int(294 * s), "11:24 COPPER LOCKED WINNING BUCKET LINE ($6,500)", C_COP_G, C_CRT, f_sm),
        f'  <polyline fill="none" stroke="{C_PHOS}" stroke-width="2" points="{_wave_points(m + 18, body_y + body_h - int(48 * s), left_w - 36, int(26 * s))}"/>',
    ]
    awards = (
        ("chronicle-award-wakener", "THE WAKENER / DESPERTADOR", "Most Seismic Noise Added: +18.4%", C_YEL),
        ("chronicle-award-sleep-guardian", "SLEEP GUARDIAN / GUARDIAN DEL SUENO", "Most Offerings & Tallies: $400 + 1 Tally", C_PHOS),
        ("chronicle-award-mole", "THE MOLE / TOPO", "Most Rock & Ore Dug: 312 Blocks", C_COP_G),
        ("chronicle-award-bucket-thief", "BUCKET THIEF / LADRON DE CANGILONES", "Most Stolen Loot x1.25: $1,125", C_COB_G),
    )
    rx, rw, gap = m + int(uw * 0.53), uw - int(uw * 0.53), int(12 * s)
    card_h = (body_h - 3 * gap) // 4
    for idx, (ftl_id, title, desc, color) in enumerate(awards):
        cy = body_y + idx * (card_h + gap)
        lines.extend([
            _rect(rx, cy, rw, card_h, C_SOOT, color, 2),
            _text(rx + 18, cy + int(38 * s), title, color, C_SOOT, f_md, ftl=ftl_id),
            _text(rx + 18, cy + int(76 * s), desc, C_CREAM, C_SOOT, f_sm),
        ])
    return lines


def _render_main_menu(width: int, height: int, m: int) -> list[str]:
    """Construye las capas SVG del Menú principal en la caseta del Vigía (1979)."""
    uw, uh = width - 2 * m, height - 2 * m
    s = 1.0 if width >= 1920 else 0.76
    f_xl, f_md, f_sm = max(22, int(30 * s)), max(15, int(16 * s)), max(14, int(14 * s))
    hdr_h = int(124 * s)
    lines: list[str] = [
        _rect(0, 0, width, height, C_VOID, rx=0),
        _rect(m, m, uw, hdr_h, C_SOOT, C_BRASS, 2),
        _text(m + 24, m + int(44 * s), "HUSHMINE", C_AMBER, C_SOOT, f_xl, ftl="menu-title"),
        _text(m + 24, m + int(80 * s), '"Rob a sleeping mountain. Play dirty. Keep it hushed."', C_CREAM, C_SOOT, f_md, ftl="menu-tagline"),
        _text(m + 24, m + int(108 * s), "WATCHER SHACK CONSOLE · 1979 SODIUM LAMP & 1931 TALLY BOARD", C_BRASS, C_SOOT, f_sm),
    ]
    buttons = (
        ("menu-play-shift", "[PLAY SHIFT] COMPETITIVE 1v1 TO 8v8", C_AMBER),
        ("menu-play-practice", "[PRACTICE] OFFLINE SHIFT VS BOTS", C_CREAM),
        ("menu-play-workshop", "[WORKSHOP] CREATIVE BUILD & MAP EDITOR", C_CREAM),
        ("menu-tally-board", "[BRASS TALLY BOARD] 12 / 41 RECOVERED (SEASON 1: THE LONG NIGHT)", C_BRASS),
        ("menu-settings", "[SETTINGS] AUDIO · VIDEO · ACCESSIBILITY · EN / ES", C_ICE),
        ("menu-quit", "[QUIT TO DESKTOP]", C_COP_G),
    )
    body_y, body_h = m + hdr_h + int(16 * s), uh - hdr_h - int(16 * s)
    left_w, gap = int(uw * 0.52), int(10 * s)
    btn_h = (body_h - 5 * gap) // 6
    for idx, (ftl_id, label, col) in enumerate(buttons):
        by = body_y + idx * (btn_h + gap)
        lines.extend([
            _rect(m, by, left_w, btn_h, C_SOOT, col, 2),
            _text(m + 18, by + btn_h // 2 + 5, label, col, C_SOOT, f_sm, ftl=ftl_id),
        ])
    rx, rw = m + int(uw * 0.54), uw - int(uw * 0.54)
    top_rh = int(body_h * 0.54)
    bot_ry, bot_rh = body_y + top_rh + gap, body_h - top_rh - gap
    lines.extend([
        _rect(rx, body_y, rw, top_rh, C_SOOT, C_BRASS_M, 2),
        _text(rx + 18, body_y + int(36 * s), "1931 BRASS TALLY BOARD · 41 HOOKS (12 RECOVERED)", C_BRASS, C_SOOT, f_md),
        _text(rx + 18, body_y + int(68 * s), "OCTOBER 14, 1931 SHIFT · #01 TO #41 STAMPED BRASS TAGS", C_CREAM, C_SOOT, f_sm),
        f'  <circle cx="{rx + 36}" cy="{body_y + int(112 * s)}" r="12" fill="{C_BRASS}"/>',
        f'  <circle cx="{rx + 72}" cy="{body_y + int(112 * s)}" r="12" fill="{C_BRASS}"/>',
        f'  <circle cx="{rx + 108}" cy="{body_y + int(112 * s)}" r="12" fill="{ui_hex("peat_dark")}" stroke="{C_BRASS}" stroke-width="1"/>',
        _rect(rx, bot_ry, rw, bot_rh, C_CRT, C_PHOS_MID, 2),
        _text(rx + 18, bot_ry + int(36 * s), "CRT P1 MONITOR · SEISMIC STANDBY 04.1% [CALM]", C_PHOS, C_CRT, f_md),
        _text(rx + 18, bot_ry + int(68 * s), "EXTERIOR SODIUM VAPOR LAMP · WEST SHAFT HEADFRAME", C_AMBER, C_CRT, f_sm),
        f'  <polyline fill="none" stroke="{C_PHOS}" stroke-width="2" points="{_wave_points(rx + 18, bot_ry + bot_rh - int(38 * s), rw - 36, int(18 * s))}"/>',
    ])
    return lines


def _render_loading(width: int, height: int, m: int) -> list[str]:
    """Construye las capas SVG de la Pantalla de carga (cartel esmaltado 1931)."""
    uw, uh = width - 2 * m, height - 2 * m
    s = 1.0 if width >= 1920 else 0.76
    f_xl, f_lg, f_md = max(16, int(22 * s)), max(14, int(18 * s)), max(12, int(15 * s))
    sign_x, sign_w, sign_h = m + int(uw * 0.06), int(uw * 0.88), int(uh * 0.68)
    pic_x, pic_w = sign_x + int(sign_w * 0.12), int(sign_w * 0.76)
    pic_y, pic_h = m + int(110 * s), int(160 * s)
    prog_y, prog_h = m + sign_h + int(20 * s), uh - sign_h - int(20 * s)

    return [
        _rect(0, 0, width, height, C_VOID, rx=0),
        _rect(sign_x, m, sign_w, sign_h, C_CREAM, C_BRASS_D, 4, rx=8),
        _text(width // 2, m + int(44 * s), "ANIMA CONSOLIDATED MINING & SMELTING CO. · SAFETY SIGN #04", C_INK, C_CREAM, f_lg, anchor="middle", ftl="loading-sign-header"),
        _text(width // 2, m + int(78 * s), "ESTABLISHED 1931 · UNDERGROUND SEISMIC SAFETY REGULATION", C_VOID, C_CREAM, f_md, anchor="middle"),
        _rect(pic_x, pic_y, pic_w, pic_h, C_SOOT, C_INK, 2),
        f'  <polyline fill="none" stroke="{C_PHOS}" stroke-width="3" points="{_wave_points(pic_x + 24, pic_y + pic_h // 2, pic_w - 48, int(36 * s))}"/>',
        _text(width // 2, pic_y + int(34 * s), "SEISMOGRAPH NEEDLE + BLASTING CHARGE WARNING", C_AMBER, C_SOOT, f_md, anchor="middle"),
        _text(width // 2, pic_y + pic_h + int(56 * s), '"WATCH THE SEISMOGRAPH NEEDLE. NEVER BLAST ABOVE BREATHE."', C_VOID, C_CREAM, f_xl, anchor="middle", ftl="sign-04-text"),
        _text(width // 2, pic_y + pic_h + int(102 * s), '"THE OLD-TIMERS HAVE ONE RULE: DON\'T WAKE THE SLEEPER."', C_INK, C_CREAM, f_lg, anchor="middle", ftl="loading-rule"),
        _rect(sign_x, prog_y, sign_w, prog_h, C_CRT, C_PHOS_MID, 2),
        _text(sign_x + 24, prog_y + int(38 * s), "CARVING 64x64x64 VOXEL MOUNTAIN · SEED 42 · 78%", C_PHOS, C_CRT, f_lg, ftl="loading-progress"),
        _text(sign_x + 24, prog_y + int(70 * s), "TAPE DRIVE 1979 · GENERATING STRATA, ORE VEINS & MOTHER LODE", C_PEAK, C_CRT, f_md),
        _rect(sign_x + 24, prog_y + prog_h - int(34 * s), int((sign_w - 48) * 0.78), int(18 * s), C_PHOS, rx=2),
    ]


def render_mockup_svg(screen: str, width: int, height: int) -> str:
    """Genera el documento SVG completo (con saltos LF) para una pantalla y resolución."""
    if screen not in MOCKUP_SCREENS:
        raise ValueError(f"Pantalla de maqueta desconocida: {screen!r}")
    if (width, height) not in SAFE_MARGINS:
        raise ValueError(f"Resolución no soportada: {width}x{height}")

    margin = SAFE_MARGINS[(width, height)]
    renderers = {
        "hud": _render_hud,
        "drop_in": _render_drop_in,
        "chronicle": _render_chronicle,
        "main_menu": _render_main_menu,
        "loading": _render_loading,
    }
    body = renderers[screen](width, height, margin)
    header = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        f"  <!-- HUSHMINE UI Mockup: {screen} ({width}x{height}) | Safe margin: {margin}px -->",
    ]
    return "\n".join([*header, *body, "</svg>"]) + "\n"


def generate_all_mockups() -> dict[str, str]:
    """Genera en memoria el diccionario {nombre_archivo: contenido_svg} de las 10 maquetas."""
    return {
        f"{screen}_{w}x{h}.svg": render_mockup_svg(screen, w, h)
        for screen in MOCKUP_SCREENS
        for w, h in SCREEN_RESOLUTIONS
    }


def write_all_mockups(output_dir: Path = MOCKUPS_DIR) -> list[Path]:
    """Escribe las 10 maquetas SVG en disco con codificación UTF-8 y saltos de línea LF."""
    output_dir.mkdir(parents=True, exist_ok=True)
    written: list[Path] = []
    for filename, svg_text in generate_all_mockups().items():
        path = output_dir / filename
        path.write_bytes(svg_text.encode("utf-8"))
        written.append(path)
    return written


def main() -> int:
    """Punto de entrada CLI para generar las 10 maquetas SVG."""
    paths = write_all_mockups()
    print(f"Generadas {len(paths)} maquetas SVG en {MOCKUPS_DIR}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
