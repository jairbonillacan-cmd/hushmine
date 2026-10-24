"""Generador determinista de activos de marca, logotipos, icono de aplicación y cápsulas de Steam.

Produce los 10 archivos SVG de `assets/ui/brand/` para HUSHMINE / Grisú empleando
exclusivamente la paleta combinada de 60 colores (`ALL_PALETTE_HEX`) y geometría
vectorial propia para el logotipo principal, sin dependencias externas.
"""

from __future__ import annotations

import sys
from pathlib import Path
import xml.etree.ElementTree as ET

REPO_ROOT: Path = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.ui.ui_palette import ALL_PALETTE_HEX, check_wcag_aa, ui_hex  # noqa: E402

BRAND_DIR: Path = REPO_ROOT / "assets" / "ui" / "brand"

BRAND_SPECS: dict[str, tuple[int, int]] = {
    "logo_a_stencil.svg": (800, 240),
    "logo_b_seismo.svg": (800, 240),
    "logo_c_chasm.svg": (800, 240),
    "app_icon.svg": (256, 256),
    "capsule_1_shears_460x215.svg": (460, 215),
    "capsule_1_shears_616x353.svg": (616, 353),
    "capsule_2_bridge_blast_460x215.svg": (460, 215),
    "capsule_2_bridge_blast_616x353.svg": (616, 353),
    "capsule_3_split_heist_460x215.svg": (460, 215),
    "capsule_3_split_heist_616x353.svg": (616, 353),
}

OFL_FONT_STACK: str = "'Chakra Petch', 'Share Tech Mono', monospace"

# Geometría vectorial propia de cada letra en una rejilla normalizada de 10x14 unidades.
_GLYPH_RECTS: dict[str, list[tuple[float, float, float, float]]] = {
    "H": [(0, 0, 2.8, 14), (7.2, 0, 2.8, 14), (2.8, 5.6, 4.4, 2.8)],
    "U": [(0, 0, 2.8, 11.4), (7.2, 0, 2.8, 11.4), (0, 11.2, 10, 2.8)],
    "S": [(0, 0, 10, 2.8), (0, 2.6, 2.8, 4.4), (0, 5.6, 10, 2.8), (7.2, 7.0, 2.8, 4.4), (0, 11.2, 10, 2.8)],
    "M": [(0, 0, 2.6, 14), (7.4, 0, 2.6, 14), (2.4, 1.2, 2.6, 4.8), (5.0, 1.2, 2.6, 4.8), (3.7, 4.4, 2.6, 5.2)],
    "I": [(0, 0, 10, 2.8), (3.6, 2.6, 2.8, 8.8), (0, 11.2, 10, 2.8)],
    "N": [(0, 0, 2.8, 14), (7.2, 0, 2.8, 14), (2.4, 2.0, 2.8, 4.5), (4.8, 6.0, 2.8, 4.5)],
    "E": [(0, 0, 2.8, 14), (2.6, 0, 7.4, 2.8), (2.6, 5.6, 5.8, 2.8), (2.6, 11.2, 7.4, 2.8)],
}


def _svg_doc(width: int, height: int, title: str, body: list[str]) -> str:
    """Envuelve los nodos SVG en un documento válido con viewBox exacto y saltos LF."""
    lines = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">',
        f"  <title>{title}</title>",
        *body,
        "</svg>",
    ]
    return "\n".join(lines) + "\n"


def _txt(
    x: float, y: float, content: str, fg_name: str, bg_name: str,
    size: int = 12, anchor: str = "middle", spacing: int = 2,
) -> str:
    """Genera una etiqueta <text> con fuentes OFL y contraste WCAG >= 4.5:1 verificado."""
    fg, bg = ui_hex(fg_name), ui_hex(bg_name)
    if not check_wcag_aa(fg, bg, 4.5):
        raise ValueError(f"Contraste WCAG insuficiente entre {fg_name} ({fg}) y {bg_name} ({bg})")
    return (
        f'  <text x="{x}" y="{y}" fill="{fg}" data-bg="{bg}" '
        f'font-family="{OFL_FONT_STACK}" font-size="{size}" font-weight="700" '
        f'letter-spacing="{spacing}" text-anchor="{anchor}">{content}</text>'
    )


def _wordmark_paths(
    x0: float, y0: float, scale: float, hush_fill: str, mine_fill: str,
    shadow_fill: str, stroke_fill: str, stencil_cut: str | None = None, seismo_shift: bool = False,
) -> list[str]:
    """Dibuja las 8 letras de HUSHMINE mediante geometría vectorial <path> propia."""
    step = 12.2 * scale
    out: list[str] = []
    for idx, ch in enumerate("HUSHMINE"):
        lx = x0 + idx * step + (1.8 * scale if idx >= 4 else 0.0)
        ly = y0 + ((-2.5 if idx % 2 == 0 else 2.5) if seismo_shift and idx >= 4 else 0.0)
        fill = hush_fill if idx < 4 else mine_fill
        segs_shadow: list[str] = []
        segs_main: list[str] = []
        for rx, ry, rw, rh in _GLYPH_RECTS[ch]:
            px, py = round(lx + rx * scale, 1), round(ly + ry * scale, 1)
            pw, ph = round(rw * scale, 1), round(rh * scale, 1)
            sx, sy = round(px + 1.2 * scale, 1), round(py + 1.2 * scale, 1)
            segs_shadow.append(f"M{sx},{sy} h{pw} v{ph} h-{pw} Z")
            segs_main.append(f"M{px},{py} h{pw} v{ph} h-{pw} Z")
        out.append(f'  <path d="{" ".join(segs_shadow)}" fill="{shadow_fill}"/>')
        out.append(
            f'  <path d="{" ".join(segs_main)}" fill="{fill}" '
            f'stroke="{stroke_fill}" stroke-width="{max(1.0, round(0.35 * scale, 1))}"/>'
        )
        if stencil_cut is not None:
            cy, ch_h = round(ly + 6.7 * scale, 1), round(0.65 * scale, 1)
            out.append(
                f'  <rect x="{round(lx - 0.5 * scale, 1)}" y="{cy}" '
                f'width="{round(11.0 * scale, 1)}" height="{ch_h}" fill="{stencil_cut}"/>'
            )
    return out


def _build_logo_a_stencil() -> str:
    """Propuesta A: Troquel Industrial de Latón (1931) en 800x240."""
    c = ui_hex
    body: list[str] = [
        f'  <rect x="0" y="0" width="800" height="240" fill="{c("void_black")}"/>',
        f'  <polygon points="24,14 776,14 790,28 790,212 776,226 24,226 10,212 10,28" fill="{c("brass_dark")}" stroke="{c("brass_bright")}" stroke-width="3"/>',
        f'  <polygon points="34,24 766,24 778,36 778,176 766,188 34,188 22,176 22,36" fill="{c("brass_mid")}" stroke="{c("brass_bright")}" stroke-width="2"/>',
        f'  <rect x="38" y="34" width="724" height="142" fill="{c("soot_dark")}" stroke="{c("brass_dark")}" stroke-width="3"/>',
        f'  <line x1="38" y1="68" x2="762" y2="68" stroke="{c("steel_dark")}" stroke-width="1"/>',
        f'  <line x1="38" y1="140" x2="762" y2="140" stroke="{c("steel_dark")}" stroke-width="1"/>',
        *[
            f'  <circle cx="{rx}" cy="{ry}" r="5" fill="{c("void_black")}" stroke="{c("brass_bright")}" stroke-width="2"/>'
            for rx, ry in ((26, 28), (774, 28), (26, 180), (774, 180), (400, 20), (400, 184))
        ],
        *_wordmark_paths(
            66.0, 48.0, 6.8, c("enamel_cream"), c("brass_bright"),
            c("void_black"), c("ink_stamp"), stencil_cut=c("soot_dark"),
        ),
        f'  <polyline points="350,34 388,88 370,104 416,148 398,176" fill="none" stroke="{c("void_black")}" stroke-width="6"/>',
        f'  <polyline points="350,34 388,88 370,104 416,148 398,176" fill="none" stroke="{c("lava_orange")}" stroke-width="2.5"/>',
        f'  <rect x="38" y="188" width="724" height="30" fill="{c("void_black")}" stroke="{c("brass_mid")}" stroke-width="1.5"/>',
        f'  <polygon points="56,193 78,193 84,199 84,209 78,215 56,215 50,209 50,199" fill="{c("brass_bright")}" stroke="{c("ink_stamp")}" stroke-width="1.5"/>',
        _txt(67, 208, "41", "ink_stamp", "brass_bright", size=11, spacing=1),
        _txt(400, 208, "MOUNT ANIMA · 1931 / 1979", "enamel_cream", "void_black", size=14, spacing=4),
        _txt(742, 208, "ANIMA CONS.", "brass_bright", "void_black", size=11, anchor="end", spacing=2),
    ]
    return _svg_doc(800, 240, "HUSHMINE — Logo Proposal A: 1931 Brass Stencil", body)


def _build_logo_b_seismo() -> str:
    """Propuesta B: Pulso de Fósforo Verde y Despertar (1979) en 800x240."""
    c = ui_hex
    body: list[str] = [
        f'  <rect x="0" y="0" width="800" height="240" fill="{c("void_black")}"/>',
        f'  <rect x="12" y="12" width="776" height="216" rx="14" fill="{c("steel_dark")}" stroke="{c("steel_mid")}" stroke-width="3"/>',
        f'  <rect x="26" y="24" width="748" height="192" rx="8" fill="{c("crt_bg")}" stroke="{c("phosphor_dim")}" stroke-width="2"/>',
        *[f'  <line x1="{gx}" y1="24" x2="{gx}" y2="216" stroke="{c("crt_grid")}" stroke-width="1"/>' for gx in range(60, 760, 60)],
        *[f'  <line x1="26" y1="{gy}" x2="774" y2="{gy}" stroke="{c("crt_grid")}" stroke-width="1"/>' for gy in (64, 104, 144, 184)],
        *_wordmark_paths(
            68.0, 58.0, 6.6, c("enamel_cream"), c("lava_orange"),
            c("phosphor_dim"), c("phosphor_bright"), seismo_shift=True,
        ),
        f'  <polyline points="36,172 340,172 362,162 378,184 394,116 408,194 424,68 442,198 462,86 482,188 504,96 528,176 556,112 584,174 764,172" fill="none" stroke="{c("phosphor_bright")}" stroke-width="3"/>',
        f'  <polyline points="394,116 408,194 424,68 442,198 462,86 482,188 504,96 528,176 556,112 584,174 764,172" fill="none" stroke="{c("lava_core")}" stroke-width="2.5"/>',
        f'  <circle cx="424" cy="68" r="5" fill="{c("lava_core")}" stroke="{c("hazard_red")}" stroke-width="2"/>',
        _txt(52, 46, "CRT SEISMOGRAPH P1 · STIR 12 HZ", "phosphor_bright", "crt_bg", size=12, anchor="start", spacing=2),
        _txt(748, 46, "100% WAKE · ERUPTION THRESHOLD", "lava_core", "crt_bg", size=12, anchor="end", spacing=2),
        _txt(400, 204, "SILENCE COLLAPSES INTO ERUPTION", "enamel_cream", "crt_bg", size=13, spacing=3),
    ]
    return _svg_doc(800, 240, "HUSHMINE — Logo Proposal B: 1979 CRT Seismograph Pulse", body)


def _build_logo_c_chasm() -> str:
    """Propuesta C: Falla Partida Copper vs. Cobalt en 800x240."""
    c = ui_hex
    body: list[str] = [
        f'  <rect x="0" y="0" width="800" height="240" fill="{c("void_black")}"/>',
        f'  <polygon points="18,22 392,22 374,186 18,186" fill="{c("copper_dark")}" stroke="{c("copper_bright")}" stroke-width="3"/>',
        f'  <polygon points="30,34 378,34 364,174 30,174" fill="{c("soot_dark")}" stroke="{c("copper_mid")}" stroke-width="2"/>',
        f'  <polygon points="408,22 782,22 782,186 426,186" fill="{c("cobalt_dark")}" stroke="{c("cobalt_bright")}" stroke-width="3"/>',
        f'  <polygon points="422,34 770,34 770,174 436,174" fill="{c("soot_dark")}" stroke="{c("cobalt_mid")}" stroke-width="2"/>',
        f'  <polygon points="384,14 416,14 434,226 366,226" fill="{c("magma_crust")}"/>',
        f'  <polygon points="392,18 408,18 422,222 378,222" fill="{c("lava_orange")}"/>',
        f'  <polygon points="397,24 403,24 410,218 390,218" fill="{c("lava_core")}"/>',
        *_wordmark_paths(54.0, 56.0, 6.4, c("copper_glow"), c("cobalt_glow"), c("void_black"), c("enamel_cream")),
        f'  <polygon points="382,178 418,178 430,190 430,212 418,224 382,224 370,212 370,190" fill="{c("brass_bright")}" stroke="{c("void_black")}" stroke-width="3"/>',
        _txt(400, 207, "#41", "ink_stamp", "brass_bright", size=16, spacing=1),
        _txt(196, 212, "COPPER SYNDICATE", "copper_glow", "void_black", size=13, spacing=3),
        _txt(604, 212, "COBALT TRUST", "cobalt_glow", "void_black", size=13, spacing=3),
    ]
    return _svg_doc(800, 240, "HUSHMINE — Logo Proposal C: Copper vs. Cobalt Chasm", body)


def _build_app_icon() -> str:
    """Icono cuadrado de aplicación (256x256) legible desde 256x256 hasta 16x16."""
    c = ui_hex
    body: list[str] = [
        f'  <rect x="0" y="0" width="256" height="256" rx="28" fill="{c("void_black")}"/>',
        f'  <polygon points="68,14 188,14 242,68 242,188 188,242 68,242 14,188 14,68" fill="{c("brass_mid")}" stroke="{c("brass_bright")}" stroke-width="6"/>',
        f'  <polygon points="76,28 180,28 228,76 228,180 180,228 76,228 28,180 28,76" fill="{c("soot_dark")}" stroke="{c("brass_dark")}" stroke-width="4"/>',
        f'  <circle cx="128" cy="44" r="10" fill="{c("void_black")}" stroke="{c("brass_bright")}" stroke-width="3"/>',
        f'  <polygon points="124,62 48,188 124,188" fill="{c("copper_bright")}" stroke="{c("copper_glow")}" stroke-width="3"/>',
        f'  <polygon points="132,62 208,188 132,188" fill="{c("cobalt_bright")}" stroke="{c("cobalt_glow")}" stroke-width="3"/>',
        f'  <path d="M52,106 Q128,66 204,106 L192,124 Q128,92 64,124 Z" fill="{c("enamel_cream")}" stroke="{c("void_black")}" stroke-width="3"/>',
        f'  <rect x="120" y="84" width="16" height="96" fill="{c("brass_bright")}" stroke="{c("void_black")}" stroke-width="3"/>',
        f'  <polygon points="96,184 160,184 176,216 80,216" fill="{c("lava_red")}" stroke="{c("lava_orange")}" stroke-width="3"/>',
        f'  <polygon points="110,192 146,192 156,212 100,212" fill="{c("lava_core")}"/>',
        f'  <polyline points="20,144 82,144 98,122 114,168 128,96 144,172 160,126 174,144 236,144" fill="none" stroke="{c("void_black")}" stroke-width="10"/>',
        f'  <polyline points="20,144 82,144 98,122 114,168 128,96 144,172 160,126 174,144 236,144" fill="none" stroke="{c("phosphor_bright")}" stroke-width="5"/>',
        f'  <circle cx="128" cy="96" r="5" fill="{c("phosphor_peak")}"/>',
    ]
    return _svg_doc(256, 256, "HUSHMINE — Application Icon (256x256 to 16x16)", body)


def _build_capsule_1_shears(w: int, h: int) -> str:
    """Boceto 1 de cápsula de Steam: Corte de línea de cangilones sobre el magma."""
    c, sx, sy = ui_hex, w / 460.0, h / 215.0
    body: list[str] = [
        f'  <rect x="0" y="0" width="{w}" height="{h}" fill="{c("void_black")}"/>',
        f'  <polygon points="0,0 {w},0 {w},{int(155 * sy)} 0,{int(175 * sy)}" fill="{c("basalt_deep")}"/>',
        f'  <rect x="0" y="{int(168 * sy)}" width="{w}" height="{h - int(168 * sy)}" fill="{c("magma_crust")}"/>',
        f'  <polygon points="0,{int(178 * sy)} {int(120 * sx)},{int(170 * sy)} {int(260 * sx)},{int(182 * sy)} {int(390 * sx)},{int(172 * sy)} {w},{int(180 * sy)} {w},{h} 0,{h}" fill="{c("lava_red")}"/>',
        f'  <polygon points="0,{int(192 * sy)} {int(160 * sx)},{int(185 * sy)} {int(310 * sx)},{int(196 * sy)} {w},{int(188 * sy)} {w},{h} 0,{h}" fill="{c("lava_orange")}"/>',
        f'  <rect x="0" y="{int(203 * sy)}" width="{w}" height="{h - int(203 * sy)}" fill="{c("lava_core")}"/>',
        f'  <line x1="0" y1="{int(118 * sy)}" x2="{int(212 * sx)}" y2="{int(96 * sy)}" stroke="{c("cobalt_glow")}" stroke-width="4"/>',
        f'  <line x1="{int(238 * sx)}" y1="{int(102 * sy)}" x2="{w}" y2="{int(148 * sy)}" stroke="{c("cobalt_glow")}" stroke-width="4" stroke-dasharray="10,5"/>',
        f'  <g transform="translate({int(86 * sx)},{int(116 * sy)})">',
        f'    <rect x="0" y="0" width="{int(42 * sx)}" height="{int(30 * sy)}" fill="{c("cobalt_bright")}" stroke="{c("cobalt_glow")}" stroke-width="2"/>',
        f'    <polygon points="6,0 18,-10 28,0" fill="{c("gold_gleam")}"/>',
        f'    <polygon points="22,0 32,-12 40,0" fill="{c("diamond_white")}"/>',
        "  </g>",
        f'  <g transform="translate({int(264 * sx)},{int(134 * sy)}) rotate(28)">',
        f'    <rect x="0" y="0" width="{int(42 * sx)}" height="{int(30 * sy)}" fill="{c("cobalt_mid")}" stroke="{c("cobalt_glow")}" stroke-width="2"/>',
        f'    <rect x="8" y="34" width="10" height="10" fill="{c("gold_gleam")}"/>',
        f'    <polygon points="26,38 34,30 42,40 32,48" fill="{c("diamond_cyan")}"/>',
        "  </g>",
        f'  <polygon points="{int(140 * sx)},{int(18 * sy)} {int(222 * sx)},{int(92 * sy)} {int(204 * sx)},{int(108 * sy)} {int(118 * sx)},{int(34 * sy)}" fill="{c("copper_bright")}" stroke="{c("copper_glow")}" stroke-width="2.5"/>',
        f'  <polygon points="{int(144 * sx)},{int(168 * sy)} {int(222 * sx)},{int(102 * sy)} {int(204 * sx)},{int(86 * sy)} {int(122 * sx)},{int(150 * sy)}" fill="{c("copper_mid")}" stroke="{c("copper_glow")}" stroke-width="2.5"/>',
        f'  <polygon points="{int(212 * sx)},{int(82 * sy)} {int(258 * sx)},{int(68 * sy)} {int(232 * sx)},{int(100 * sy)}" fill="{c("enamel_cream")}" stroke="{c("void_black")}" stroke-width="2"/>',
        f'  <polygon points="{int(212 * sx)},{int(112 * sy)} {int(258 * sx)},{int(126 * sy)} {int(232 * sx)},{int(94 * sy)}" fill="{c("steel_light")}" stroke="{c("void_black")}" stroke-width="2"/>',
        f'  <circle cx="{int(216 * sx)}" cy="{int(97 * sy)}" r="{int(8 * sx)}" fill="{c("brass_bright")}" stroke="{c("void_black")}" stroke-width="2"/>',
        f'  <polygon points="{int(226 * sx)},{int(96 * sy)} {int(248 * sx)},{int(78 * sy)} {int(238 * sx)},{int(98 * sy)} {int(262 * sx)},{int(104 * sy)} {int(234 * sx)},{int(106 * sy)} {int(244 * sx)},{int(124 * sy)}" fill="{c("lava_core")}"/>',
        f'  <rect x="{int(266 * sx)}" y="{int(14 * sy)}" width="{int(180 * sx)}" height="{int(84 * sy)}" rx="4" fill="{c("crt_bg")}" stroke="{c("hazard_red")}" stroke-width="2.5"/>',
        *_wordmark_paths(round(275 * sx, 1), round(22 * sy, 1), round(1.65 * sx, 2), c("enamel_cream"), c("lava_orange"), c("void_black"), c("brass_bright")),
        f'  <polyline points="{int(276 * sx)},{int(74 * sy)} {int(310 * sx)},{int(74 * sy)} {int(322 * sx)},{int(56 * sy)} {int(334 * sx)},{int(88 * sy)} {int(348 * sx)},{int(52 * sy)} {int(362 * sx)},{int(86 * sy)} {int(378 * sx)},{int(60 * sy)} {int(436 * sx)},{int(74 * sy)}" fill="none" stroke="{c("lava_core")}" stroke-width="2.5"/>',
        _txt(int(356 * sx), int(92 * sy), "100% WAKE · CUT THE LINE", "lava_core", "crt_bg", size=max(9, int(10 * sy)), spacing=1),
    ]
    return _svg_doc(w, h, f"HUSHMINE — Steam Capsule 1: Bucket Line Sabotage ({w}x{h})", body)


def _build_capsule_2_bridge_blast(w: int, h: int) -> str:
    """Boceto 2 de cápsula de Steam: Sabotaje cruzado y carga de demolición."""
    c, sx, sy = ui_hex, w / 460.0, h / 215.0
    body: list[str] = [
        f'  <rect x="0" y="0" width="{w}" height="{h}" fill="{c("void_black")}"/>',
        f'  <polygon points="{int(60 * sx)},0 {int(210 * sx)},0 {int(310 * sx)},{h} {int(10 * sx)},{h}" fill="{c("peat_dark")}"/>',
        f'  <polygon points="{int(210 * sx)},0 {int(340 * sx)},0 {int(420 * sx)},{h} {int(160 * sx)},{h}" fill="{c("basalt_deep")}"/>',
        f'  <circle cx="{int(276 * sx)}" cy="{int(108 * sy)}" r="{int(48 * sx)}" fill="{c("lava_red")}"/>',
        f'  <circle cx="{int(276 * sx)}" cy="{int(108 * sy)}" r="{int(30 * sx)}" fill="{c("lava_orange")}"/>',
        f'  <circle cx="{int(276 * sx)}" cy="{int(108 * sy)}" r="{int(16 * sx)}" fill="{c("lava_core")}"/>',
        f'  <rect x="{int(248 * sx)}" y="{int(118 * sy)}" width="{int(22 * sx)}" height="{int(90 * sy)}" fill="{c("cobalt_dark")}" stroke="{c("cobalt_bright")}" stroke-width="2"/>',
        f'  <g transform="translate({int(262 * sx)},{int(64 * sy)}) rotate(24)">',
        f'    <rect x="0" y="0" width="{int(22 * sx)}" height="{int(42 * sy)}" fill="{c("cobalt_bright")}" stroke="{c("cobalt_glow")}" stroke-width="2"/>',
        f'    <rect x="-12" y="8" width="{int(46 * sx)}" height="10" fill="{c("steel_light")}"/>',
        "  </g>",
        f'  <rect x="{int(320 * sx)}" y="{int(118 * sy)}" width="{int(34 * sx)}" height="{int(26 * sy)}" fill="{c("cobalt_mid")}" stroke="{c("cobalt_glow")}" stroke-width="2"/>',
        f'  <polygon points="{int(326 * sx)},{int(118 * sy)} {int(338 * sx)},{int(104 * sy)} {int(350 * sx)},{int(118 * sy)}" fill="{c("gold_gleam")}"/>',
        f'  <rect x="{int(34 * sx)}" y="{int(86 * sy)}" width="{int(56 * sx)}" height="{int(94 * sy)}" rx="6" fill="{c("copper_dark")}" stroke="{c("copper_glow")}" stroke-width="2.5"/>',
        f'  <rect x="{int(44 * sx)}" y="{int(62 * sy)}" width="{int(36 * sx)}" height="{int(28 * sy)}" rx="4" fill="{c("copper_bright")}" stroke="{c("enamel_cream")}" stroke-width="2"/>',
        f'  <rect x="{int(52 * sx)}" y="{int(70 * sy)}" width="{int(22 * sx)}" height="{int(10 * sy)}" fill="{c("sodium_warm")}"/>',
        f'  <rect x="{int(108 * sx)}" y="{int(142 * sy)}" width="{int(40 * sx)}" height="{int(52 * sy)}" fill="{c("brass_mid")}" stroke="{c("brass_bright")}" stroke-width="2.5"/>',
        f'  <rect x="{int(114 * sx)}" y="{int(120 * sy)}" width="{int(28 * sx)}" height="{int(8 * sy)}" fill="{c("brass_bright")}"/>',
        f'  <rect x="{int(125 * sx)}" y="{int(128 * sy)}" width="{int(6 * sx)}" height="{int(16 * sy)}" fill="{c("enamel_cream")}"/>',
        f'  <path d="M{int(148 * sx)},{int(176 * sy)} Q{int(210 * sx)},{int(196 * sy)} {int(276 * sx)},{int(108 * sy)}" fill="none" stroke="{c("lava_core")}" stroke-width="3" stroke-dasharray="6,4"/>',
        f'  <rect x="{int(14 * sx)}" y="{int(12 * sy)}" width="{int(224 * sx)}" height="{int(42 * sy)}" fill="{c("soot_dark")}" stroke="{c("brass_bright")}" stroke-width="2"/>',
        *_wordmark_paths(round(24 * sx, 1), round(17 * sy, 1), round(2.0 * sx, 2), c("enamel_cream"), c("sodium_warm"), c("void_black"), c("brass_bright")),
        f'  <rect x="{int(298 * sx)}" y="{int(14 * sy)}" width="{int(148 * sx)}" height="{int(62 * sy)}" rx="4" fill="{c("crt_bg")}" stroke="{c("hazard_red")}" stroke-width="2"/>',
        f'  <polyline points="{int(308 * sx)},{int(52 * sy)} {int(340 * sx)},{int(52 * sy)} {int(354 * sx)},{int(28 * sy)} {int(368 * sx)},{int(66 * sy)} {int(384 * sx)},{int(24 * sy)} {int(400 * sx)},{int(64 * sy)} {int(436 * sx)},{int(46 * sy)}" fill="none" stroke="{c("lava_core")}" stroke-width="2.5"/>',
        _txt(int(372 * sx), int(70 * sy), "WAKE · BRIDGE BLAST", "lava_core", "crt_bg", size=max(9, int(10 * sy)), spacing=1),
    ]
    return _svg_doc(w, h, f"HUSHMINE — Steam Capsule 2: Bridge Demolition ({w}x{h})", body)


def _build_capsule_3_split_heist(w: int, h: int) -> str:
    """Boceto 3 de cápsula de Steam: Tensión en el Sismógrafo y caída al Corazón."""
    c, sx, sy = ui_hex, w / 460.0, h / 215.0
    body: list[str] = [
        f'  <rect x="0" y="0" width="{w}" height="{h}" fill="{c("void_black")}"/>',
        f'  <polygon points="0,0 {int(270 * sx)},0 {int(180 * sx)},{h} 0,{h}" fill="{c("crt_bg")}"/>',
        f'  <polygon points="{int(270 * sx)},0 {w},0 {w},{h} {int(180 * sx)},{h}" fill="{c("basalt_deep")}"/>',
        *[f'  <line x1="0" y1="{int(gy * sy)}" x2="{int(240 * sx)}" y2="{int(gy * sy)}" stroke="{c("crt_grid")}" stroke-width="1.5"/>' for gy in (36, 72, 108, 144)],
        f'  <polyline points="{int(14 * sx)},{int(112 * sy)} {int(64 * sx)},{int(112 * sy)} {int(84 * sx)},{int(74 * sy)} {int(102 * sx)},{int(146 * sy)} {int(124 * sx)},{int(48 * sy)} {int(146 * sx)},{int(154 * sy)} {int(172 * sx)},{int(56 * sy)} {int(210 * sx)},{int(92 * sy)}" fill="none" stroke="{c("phosphor_bright")}" stroke-width="3"/>',
        f'  <polyline points="{int(102 * sx)},{int(146 * sy)} {int(124 * sx)},{int(48 * sy)} {int(146 * sx)},{int(154 * sy)} {int(172 * sx)},{int(56 * sy)} {int(210 * sx)},{int(92 * sy)}" fill="none" stroke="{c("lava_core")}" stroke-width="3"/>',
        _txt(int(20 * sx), int(26 * sy), "SEISMOGRAPH · 100% WAKE", "lava_core", "crt_bg", size=max(10, int(11 * sy)), anchor="start", spacing=1),
        f'  <polygon points="{int(266 * sx)},0 {int(278 * sx)},0 {int(188 * sx)},{h} {int(176 * sx)},{h}" fill="{c("brass_bright")}" stroke="{c("void_black")}" stroke-width="2"/>',
        f'  <circle cx="{int(348 * sx)}" cy="{int(136 * sy)}" r="{int(48 * sx)}" fill="{c("magma_crust")}" stroke="{c("lava_orange")}" stroke-width="3"/>',
        f'  <polygon points="{int(348 * sx)},{int(98 * sy)} {int(384 * sx)},{int(136 * sy)} {int(348 * sx)},{int(174 * sy)} {int(312 * sx)},{int(136 * sy)}" fill="{c("lava_red")}" stroke="{c("lava_core")}" stroke-width="2"/>',
        f'  <polygon points="{int(348 * sx)},{int(114 * sy)} {int(368 * sx)},{int(136 * sy)} {int(348 * sx)},{int(158 * sy)} {int(328 * sx)},{int(136 * sy)}" fill="{c("lava_core")}"/>',
        f'  <line x1="{int(250 * sx)}" y1="{int(72 * sy)}" x2="{w}" y2="{int(98 * sy)}" stroke="{c("cobalt_glow")}" stroke-width="3.5"/>',
        f'  <rect x="{int(304 * sx)}" y="{int(56 * sy)}" width="{int(28 * sx)}" height="{int(32 * sy)}" fill="{c("copper_bright")}" stroke="{c("enamel_cream")}" stroke-width="2"/>',
        f'  <circle cx="{int(332 * sx)}" cy="{int(82 * sy)}" r="{int(8 * sx)}" fill="{c("lava_core")}"/>',
        f'  <rect x="0" y="{int(162 * sy)}" width="{w}" height="{h - int(162 * sy)}" fill="{c("void_black")}" stroke="{c("brass_mid")}" stroke-width="2"/>',
        *_wordmark_paths(round(136 * sx, 1), round(166 * sy, 1), round(1.9 * sx, 2), c("enamel_cream"), c("phosphor_bright"), c("soot_dark"), c("brass_bright")),
        _txt(int(230 * sx), int(209 * sy), "ROB A SLEEPING MOUNTAIN. PLAY DIRTY. KEEP IT HUSHED.", "enamel_cream", "void_black", size=max(9, int(9.5 * sy)), spacing=1),
    ]
    return _svg_doc(w, h, f"HUSHMINE — Steam Capsule 3: Seismograph and Heart Heist ({w}x{h})", body)


def generate_all_brand_svgs() -> dict[str, str]:
    """Genera en memoria los 10 archivos SVG de identidad de marca y cápsulas de Steam."""
    return {
        "logo_a_stencil.svg": _build_logo_a_stencil(),
        "logo_b_seismo.svg": _build_logo_b_seismo(),
        "logo_c_chasm.svg": _build_logo_c_chasm(),
        "app_icon.svg": _build_app_icon(),
        "capsule_1_shears_460x215.svg": _build_capsule_1_shears(460, 215),
        "capsule_1_shears_616x353.svg": _build_capsule_1_shears(616, 353),
        "capsule_2_bridge_blast_460x215.svg": _build_capsule_2_bridge_blast(460, 215),
        "capsule_2_bridge_blast_616x353.svg": _build_capsule_2_bridge_blast(616, 353),
        "capsule_3_split_heist_460x215.svg": _build_capsule_3_split_heist(460, 215),
        "capsule_3_split_heist_616x353.svg": _build_capsule_3_split_heist(616, 353),
    }


def write_all_brand_svgs(output_dir: Path = BRAND_DIR) -> list[Path]:
    """Escribe los 10 activos SVG de marca en disco con saltos de línea LF y validación XML."""
    output_dir.mkdir(parents=True, exist_ok=True)
    svgs = generate_all_brand_svgs()
    written: list[Path] = []
    for filename, (expected_w, expected_h) in BRAND_SPECS.items():
        content = svgs[filename]
        root = ET.fromstring(content)
        expected_vb = f"0 0 {expected_w} {expected_h}"
        if root.attrib.get("viewBox") != expected_vb:
            raise ValueError(f"viewBox inválido en {filename}: se esperaba {expected_vb}")
        out_path = output_dir / filename
        out_path.write_bytes(content.encode("utf-8"))
        written.append(out_path)
    return written


__all__ = [
    "ALL_PALETTE_HEX",
    "BRAND_DIR",
    "BRAND_SPECS",
    "OFL_FONT_STACK",
    "generate_all_brand_svgs",
    "write_all_brand_svgs",
]


if __name__ == "__main__":
    paths = write_all_brand_svgs()
    print(f"Generados {len(paths)} archivos SVG de marca en {BRAND_DIR}")
