"""Generador procedural determinista de los 25 iconos de interfaz (16×16 y 32×32) de HUSHMINE.

Produce los 50 archivos PNG RGBA8 en `assets/ui/icons/<stem>_{16,32}.png` usando
exclusivamente la paleta cerrada `ALL_PALETTE_SET` (60 colores opacos, A=255).
"""

from __future__ import annotations

import sys
from pathlib import Path

REPO_ROOT: Path = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from tools.textures.png import encode_png, read_png, write_png  # noqa: E402
from tools.ui.ui_palette import (  # noqa: E402
    ALL_PALETTE,
    ALL_PALETTE_SET,
    ICON_SIZES,
    ICON_STEMS,
    ui_color,
)

# Directorio canónico de salida para los 50 iconos rasterizados.
ICONS_DIR: Path = REPO_ROOT / "assets" / "ui" / "icons"

# Tabla de símbolos de 1 carácter hacia nombres de color de ALL_PALETTE.
_CHAR_COLOR_NAMES: dict[str, str] = {
    ".": "void_black", ",": "soot_dark", ":": "shadow_slate", ";": "basalt_dark",
    "s": "steel_dark", "S": "steel_mid", "L": "steel_light", "k": "slate_dark",
    "K": "slate_mid", "Q": "slate_light", "H": "shale_highlight", "t": "timber_dark",
    "T": "timber_mid", "w": "timber_light", "o": "ore_rust_dark", "O": "ore_rust_mid",
    "a": "ore_amber", "A": "ore_highlight", "g": "gold_shadow", "G": "gold_mid",
    "y": "gold_bright", "Y": "gold_gleam", "b": "brass_dark", "B": "brass_mid",
    "M": "brass_bright", "E": "enamel_cream", "d": "diamond_deep", "D": "diamond_teal",
    "c": "diamond_cyan", "C": "diamond_ice", "W": "diamond_white", "m": "magma_crust",
    "r": "hazard_red", "R": "lava_red", "f": "lava_orange", "n": "sodium_warm",
    "F": "lava_yellow", "U": "lava_core", "e": "cobalt_mid", "Z": "cobalt_bright",
    "J": "electric_blue", "u": "copper_mid", "V": "copper_bright", "_": "crt_bg",
    "-": "crt_grid", "p": "phosphor_dim", "P": "phosphor_mid", "v": "phosphor_bright",
    "X": "phosphor_peak",
}

_CHAR_RGBA: dict[str, tuple[int, int, int, int]] = {
    ch: ui_color(name) for ch, name in _CHAR_COLOR_NAMES.items()
}

# Plantillas base 16×16 (16 filas de 16 caracteres separadas por '/') para los 25 iconos.
_ICON_GRIDS: dict[str, str] = {
    "class_prospector": (
        "ssssssssssssssss/s,,,,,,,,,,,,,,s/s,,cc,,,,,,cc,,s/s,ccCC,,,,CCcc,s/"
        "s,cc,vvLLvv,cc,s/s,,,vvLLLLvv,,,s/s,,,PPSSLLPP,,,s/s,,,,PPSSPP,,,,s/"
        "s,,,,SSSSSS,,,,s/s,,,,,SSSS,,,,,s/s,,,,,,BB,,,,,,s/s,,,,,,BB,,,,,,s/"
        "s,,,,,bMMb,,,,,s/s,,,,,bMMb,,,,,s/s,,,,,,,,,,,,,,s/ssssssssssssssss"
    ),
    "class_miner": (
        "ssssssssssssssss/s,,,,LLLLLL,,,,s/s,,LLSSSSSSLL,,s/s,LLSS,ww,SSLL,s/"
        "s,LLS,,ww,,SLL,s/s,,,,,,TT,,,,,,s/s,,,,,wTTw,,,,,s/s,,,,,wTTggyy,,s/"
        "s,,,,wTTggYYyy,s/s,,,,TT,gYYYYy,s/s,,,wTT,gYYyyG,s/s,,,TT,,ggyyGG,s/"
        "s,,wTT,,,GGGG,,s/s,,wwT,,,,GG,,,s/s,,,,,,,,,,,,,,s/ssssssssssssssss"
    ),
    "class_engineer": (
        "ssssssssssssssss/s,,,SS,,,,SS,,,s/s,,,SSLLLLSS,,,s/s,SSLLLLLLLLSS,s/"
        "s,SLLLssssLLLS,s/s,,LLss,,ssLL,,s/s,SLLs,,,,sLLS,s/s,SLLss,,ssLLS,s/"
        "s,,LLLssssBBBB,s/s,SSLLLLLLBMMB,s/s,,,SSLLSSBMMB,s/s,,,SS,,,BBBB,,s/"
        "s,,,,,,,BBBB,,,s/s,,,,,,BBBB,,,,s/s,,,,,,,,,,,,,,s/ssssssssssssssss"
    ),
    "class_sapper": (
        "ssssssssssssssss/s,,,,,,QK,,,,,,s/s,,,,,,QK,,,,,,s/s,,LL,,QK,,LL,,s/"
        "s,,LLL,QK,LLL,,s/s,,,LLLffLLL,,,s/s,,,,LLUUFF,,,,s/s,,,rrfUUFfrr,,s/"
        "s,,rrrfFFffrrr,s/s,,,,SS,,SS,,,,s/s,,,SSS,QK,SSS,s/s,,rr,,,QK,,,rrs/"
        "s,,rr,,,QK,,,rrs/s,rrr,,,QK,,,rrs/s,,,,,,,,,,,,,,s/ssssssssssssssss"
    ),
    "res_ore": (
        "ssssssssssssssss/s,,,,,,,,,,,,,,s/s,,,,kkkkKK,,,,s/s,,,kkKKQQAA,,,s/"
        "s,,kkKQoOOAAa,,s/s,kkKQoOOAAaaK,s/s,kKQoOOAAaaoK,s/s,kKoOOAAaaoKK,s/"
        "s,kKoOAAaaoKQQ,s/s,kkKoAAaoKKQQ,s/s,kkkKoaOOoKkk,s/s,,kkkKOOokkk,,s/"
        "s,,,kkkkkkkk,,,s/s,,,,kkkkkk,,,,s/s,,,,,,,,,,,,,,s/ssssssssssssssss"
    ),
    "res_gold": (
        "ssssssssssssssss/s,,,,,,,,,,,,,,s/s,,,,,yyyy,,,,,s/s,,,yyYYYYyy,,,s/"
        "s,,yYYYYYYYYy,,s/s,,yYYUUUUYYG,,s/s,gyYYYYYYYYGG,s/s,gyyYYYYYYyGG,s/"
        "s,ggyyyyyyyyGG,s/s,ggGGGGGGGGgg,s/s,,ggGGGGGGgg,,s/s,,gggggggggg,,s/"
        "s,,,gggggggg,,,s/s,,,,,,,,,,,,,,s/s,,,,,,,,,,,,,,s/ssssssssssssssss"
    ),
    "res_diamond": (
        "ssssssssssssssss/s,,,,,,,,,,,,,,s/s,,,,,CCCC,,,,,s/s,,,CCWWWWCC,,,s/"
        "s,,CWWWWWWWWC,,s/s,ccWWCCCCWWdd,s/s,ccCCCCCCCCdd,s/s,,ccCCCCCCdd,,s/"
        "s,,,ccCCCCdd,,,s/s,,,,ccCCdd,,,,s/s,,,,ccCCdd,,,,s/s,,,,,ccdd,,,,,s/"
        "s,,,,,DDDD,,,,,s/s,,,,,,dd,,,,,,s/s,,,,,,,,,,,,,,s/ssssssssssssssss"
    ),
    "hud_bank": (
        "bbbbbbbbbbbbbbbb/b,,SSSSSSSSSS,,b/b,SSLLLLLLLLSS,b/b,SLssMMMMssLS,b/"
        "b,LLsMBssBMssL,b/b,SLsMMMMMMsLS,b/b,SLMMBYYBMMLS,b/b,SLMMByyBMMLS,b/"
        "b,SLsMMMMMMsLS,b/b,LLsMBssBMssL,b/b,SLssMMMMssLS,b/b,SSLLLLLLLLSS,b/"
        "b,,SSSSSSSSSS,,b/b,,ss,,,,,,ss,,b/b,,,,,,,,,,,,,,b/bbbbbbbbbbbbbbbb"
    ),
    "item_brass_tally": (
        "ssssssssssssssss/s,,,,bbbbbb,,,,s/s,,,bMMMMMMb,,,s/s,,bMM,,,,MMb,,s/"
        "s,bMMM,,,,MMMb,s/s,bMMMMMMMMMMb,s/s,bMMbbMMbbMMb,s/s,bMMbbMMbbMMb,s/"
        "s,bMMBBBBBBMMb,s/s,bMMbbMMbbMMb,s/s,bMMbbMMbbMMb,s/s,bMMBBBBBBMMb,s/"
        "s,,bBBBBBBBBb,,s/s,,,bbbbbbbb,,,s/s,,,,,,,,,,,,,,s/ssssssssssssssss"
    ),
    "action_offering": (
        "ssssssssssssssss/s,,,,,,vv,,,,,,s/s,,,,,,vv,,,,,,s/s,,,,vvvvvv,,,,s/"
        "s,,,,,vvvv,,,,,s/s,,,,,,PP,,,,,,s/s,,,,,yYYy,,,,,s/s,,,,yYYYYy,,,,s/"
        "s,,,,gyYYyg,,,,s/s,QQ,,gggg,,QQ,s/s,QQK,nnnn,KQQ,s/s,,QKKnnnnKKQ,,s/"
        "s,,,KKKKKKKK,,,s/s,,,,kkkkkk,,,,s/s,,,,kkkkkk,,,,s/ssssssssssssssss"
    ),
    "item_explosive": (
        "ssssssssssssssss/s,,,,,,,,UUFF,,s/s,,,,,,,UFFff,,s/s,,,,,,wwff,,,,s/"
        "s,,,,,ww,,,,,,,s/s,,,rrRRRRrr,,,s/s,,,rrRRRRrr,,,s/s,,BBBBBBBBBB,,s/"
        "s,,,rrRRRRrr,,,s/s,,,rrRRRRrr,,,s/s,,bbbbbbbbbb,,s/s,,,rrRRRRrr,,,s/"
        "s,,,rrRRRRrr,,,s/s,,,mmmmmmmm,,,s/s,,,,,,,,,,,,,,s/ssssssssssssssss"
    ),
    "tool_geophone": (
        "ssssssssssssssss/s,,,,SSSSSS,,,,s/s,,SSLLLLLLSS,,s/s,SLL,,,,,,LLS,s/"
        "s,SL,,vvvv,,LS,s/s,BB,vv,,vv,BB,s/s,BBvv,,,,vvBB,s/s,,,PP,,,,PP,,,s/"
        "s,,,,,,LL,,,,,,s/s,,,,,,LL,,,,,,s/s,,,,,BBBB,,,,,s/s,,,,,,SS,,,,,,s/"
        "s,,,KKKKKKKK,,,s/s,,kkkSSSSkkk,,s/s,,kkkkkkkkkk,,s/ssssssssssssssss"
    ),
    "status_fame": (
        "ssssssssssssssss/s,,,,,,YY,,,,,,s/s,,nn,YYYY,nn,,s/s,nnnyUUUUynnn,s/"
        "s,,nnyUUUUynn,,s/s,,,,yYYYYy,,,,s/s,nn,,yyyy,,nn,s/s,nn,BBBBBB,nn,s/"
        "s,,,,BMMMMB,,,,s/s,,,,BBBBBB,,,,s/s,,,SSBBBBSS,,,s/s,,,SSSSSSSS,,,s/"
        "s,,SSSSSSSSSS,,s/s,,ssssssssss,,s/s,,,,,,,,,,,,,,s/ssssssssssssssss"
    ),
    "seismo_stir": (
        "ssssssssssssssss/s______________s/s_------------_s/s______________s/"
        "s______________s/s_____vvvv_____s/s____vvnnvv____s/s_vvvv____vvvv_s/"
        "s______________s/s_------------_s/s______________s/s_vv_pp_pp_pp__s/"
        "s_vv_pp_pp_pp__s/s_vv_pp_pp_pp__s/s______________s/ssssssssssssssss"
    ),
    "seismo_breathe": (
        "ssssssssssssssss/s______________s/s_------------_s/s______YY______s/"
        "s_____yYYy_____s/s____nn__nn____s/s___nn____nn___s/s_nnn______nnn_s/"
        "s______yy______s/s_-----yy-----_s/s______________s/s_nn_nn_pp_pp__s/"
        "s_nn_nn_pp_pp__s/s_nn_nn_pp_pp__s/s______________s/ssssssssssssssss"
    ),
    "seismo_restless": (
        "ssssssssssssssss/s______________s/s___ff____ff___s/s___ff____ff___s/"
        "s__ffrr__rrff__s/s__ffrr__rrff__s/s_ff__r__r__ff_s/s_ff__rrrr__ff_s/"
        "s_____rrrr_____s/s_------------_s/s______________s/s_ff_ff_rr_pp__s/"
        "s_ff_ff_rr_pp__s/s_ff_ff_rr_pp__s/s______________s/ssssssssssssssss"
    ),
    "seismo_wake": (
        "ssssssssssssssss/srrrrrrrrrrrrrrs/sr____rrrr__UUrs/sr__rrffffrrUUrs/"
        "sr_rrffUUffrFFrs/sr_rffUUUUffFFrs/sr_rrffUUffrr_rs/sr__rrffffrr__rs/"
        "sr____rrrr____rs/srrrrrrrrrrrrrrs/s______________s/s_rr_rr_ff_UU__s/"
        "s_rr_rr_ff_UU__s/s_rr_rr_ff_UU__s/s______________s/ssssssssssssssss"
    ),
    "death_lava": (
        "ssssssssssssssss/s,,,,,,,,,,,,,,s/s,,,,,nnnn,,,,,s/s,,,BBBBBBBB,,,s/"
        "s,,EEEEEEEEEE,,s/s,,EE..EE..EE,,s/s,,EEEEEEEEEE,,s/s,,,EE.EE.EE,,,s/"
        "s,RRfEEEEEEfRR,s/sRRRRffffffRRRRs/sRffffFFFFffffRs/sffFFUUUUUUFFffs/"
        "smmRRRRRRRRRRmms/smmmmmmmmmmmmmms/sRRRRRRRRRRRRRRs/ssssssssssssssss"
    ),
    "death_electric": (
        "ssssssssssssssss/s;;;;;;JJJJ;;;;s/s;;;;;JJUUJJ;;;s/s;;;;JJUUUUJJ;;s/"
        "s;;;JJUUUUJJ;;;s/s;;JJUUUUJJ;;;;s/s;JJUUUUUUUUJJ;s/s;JJJUUUUUUUUJ;s/"
        "s;;;;;JJUUUUJJ;s/s;;;;;JJUUUUJJ;s/s;;;;JJUUUUJJ;;s/s;;;JJUUUUJJ;;;s/"
        "s;;;JJUUJJ;;;;;s/s;;JJJJ;;;;;;;;s/s;;;;;;;;;;;;;;s/ssssssssssssssss"
    ),
    "death_fall": (
        "ssssssssssssssss/s,,,rr,WW,rr,,,s/s,,,rr,WW,rr,,,s/s,,,,,,WW,,,,,,s/"
        "s,,,,LLWWLL,,,,s/s,,,,LLWWLL,,,,s/s,,,LLWWWWLL,,,s/s,,,,WWWWWW,,,,s/"
        "s,,,,,WWWW,,,,,s/s,,,ff,WW,ff,,,s/s,,ffUULLUUff,,s/s,QQffUUUUffQQ,s/"
        "s,QQKffffffKQQ,s/s,KKKKkkkkKKKK,s/s,kkkkkkkkkkkk,s/ssssssssssssssss"
    ),
    "death_cave_in": (
        "ssssssssssssssss/s,KKKK,kk,KKKK,s/s,KQQK,kk,KQQK,s/s,KKKK,QQ,KKKK,s/"
        "s,,,,,,QQ,,,,,,s/s,,rr,,ff,,rr,,s/s,,rr,,ff,,rr,,s/s,rrrr,ff,rrrr,s/"
        "s,,rr,,ff,,rr,,s/s,,,KKKKKKKK,,,s/s,,,KQQQQQQK,,,s/s,,,,nnnnnn,,,,s/"
        "s,,BBBBBBBBBB,,s/s,,bbbbbbbbbb,,s/s,,,,,,,,,,,,,,s/ssssssssssssssss"
    ),
    "title_wakener": (
        "ssssssssssssssss/s,,,MMMMMMMM,,,s/s,,,bbbbbbbb,,,s/s,,rr,,BB,,rr,,s/"
        "s,rrff,BB,ffrr,s/s,rfUU,BB,UUfr,s/s,rrff,BB,ffrr,s/s,,rr,RRRR,rr,,s/"
        "s,,,,RRRRRR,,,,s/s,,,tTTTTTTt,,,s/s,,,tTTRRTTt,,,s/s,,,tTRUUURt,,,s/"
        "s,,,tTTRRTTt,,,s/s,,,tttttttt,,,s/s,,,,,,,,,,,,,,s/ssssssssssssssss"
    ),
    "title_sleep_guardian": (
        "bbbbbbbbbbbbbbbb/b,,,MMMMMMMM,,,b/b,,MMBBBBBBMM,,b/b,MMBBBBBBBBMM,b/"
        "b,MBB,,,,,,BBM,b/b,MB,bb,,bb,BM,b/b,MB,bbMMbb,BM,b/b,MB,,bbbb,,BM,b/"
        "b,MBB,,,,,,BBM,b/b,MBvvvvvvvvBM,b/b,,MBPPPPPPBM,,b/b,,MMBBBBBBMM,,b/"
        "b,,,MMBBBBMM,,,b/b,,,,MMMMMM,,,,b/b,,,,,bbbb,,,,,b/bbbbbbbbbbbbbbbb"
    ),
    "title_mole": (
        "ssssssssssssssss/s,kkkk,,,,kkkk,s/s,kk,,nnnn,,kk,s/s,kk,nnUUnn,kk,s/"
        "s,,,BBBBBBBB,,,s/s,,BMMMMMMMMB,,s/s,,bbbbbbbbbb,,s/s,,,,,SSSS,,,,,s/"
        "s,,,,SLLLLS,,,,s/s,,,SLLWWLLS,,,s/s,,,,SLLLLS,,,,s/s,,,,,SLLS,,,,,s/"
        "s,kk,,,WW,,,kk,s/s,kkk,,WW,,kkk,s/s,kkkk,,,,kkkk,s/ssssssssssssssss"
    ),
    "title_bucket_thief": (
        "ssssssssssssssss/s,,,,,,LL,,,,,,s/s,,,,,,LLrr,,,,s/s,,,,,SS,rr,,,,s/"
        "s,,,SSSSSS,,,,,s/s,,SSLLLLSS,,,,s/s,,SLLssLLSyy,,s/s,,SLLssLLYYYY,s/"
        "s,,,SLLLLSYYYY,s/s,,,,SSSS,CCWW,s/s,,,,,,,,cCCWW,s/s,,,,,,,yyccCC,s/"
        "s,,,,,,yYYy,,,,s/s,,,,,,yyyy,CC,s/s,,,,,,,,,,,CC,s/ssssssssssssssss"
    ),
}


def _decode_grid_16(stem: str) -> list[list[tuple[int, int, int, int]]]:
    """Convierte la plantilla textual 16×16 de un icono en una matriz de tuplas RGBA8."""
    if stem not in _ICON_GRIDS:
        raise KeyError(f"Identificador de icono desconocido: {stem!r}")
    rows = _ICON_GRIDS[stem].split("/")
    if len(rows) != 16 or any(len(r) != 16 for r in rows):
        raise ValueError(f"Plantilla 16×16 inválida para {stem!r}")
    return [[_CHAR_RGBA[ch] for ch in row] for row in rows]


def generate_icon_16(stem: str) -> bytes:
    """Genera el buffer RGBA8 de 16×16 píxeles (1024 bytes) para el icono `stem`."""
    grid = _decode_grid_16(stem)
    return b"".join(bytes(rgba) for row in grid for rgba in row)


def generate_icon_32(stem: str) -> bytes:
    """Genera el buffer RGBA8 de 32×32 píxeles (4096 bytes) con suavizado EPX y bisel."""
    g = _decode_grid_16(stem)
    out: list[list[tuple[int, int, int, int]]] = [[g[0][0]] * 32 for _ in range(32)]

    # Escalado 2× con suavizado de diagonales EPX (Scale2x) sobre la paleta cerrada.
    for y in range(16):
        for x in range(16):
            p = g[y][x]
            u = g[y - 1][x] if y > 0 else p
            d = g[y + 1][x] if y < 15 else p
            l_px = g[y][x - 1] if x > 0 else p
            r_px = g[y][x + 1] if x < 15 else p
            if l_px != r_px and u != d:
                p0 = u if u == l_px else p
                p1 = u if u == r_px else p
                p2 = d if d == l_px else p
                p3 = d if d == r_px else p
            else:
                p0 = p1 = p2 = p3 = p
            y2, x2 = y * 2, x * 2
            out[y2][x2], out[y2][x2 + 1] = p0, p1
            out[y2 + 1][x2], out[y2 + 1][x2 + 1] = p2, p3

    # Refinamiento 32×32: chaflán exterior, bisel iluminado, remaches y retícula CRT.
    void_c = _CHAR_RGBA["."]
    is_brass = g[0][0] == _CHAR_RGBA["b"]
    bezel_hi = _CHAR_RGBA["B" if is_brass else "S"]
    bezel_lo = _CHAR_RGBA["b" if is_brass else "s"]
    rivet_c = _CHAR_RGBA["M" if is_brass else "L"]
    crt_bg_c, crt_grid_c = _CHAR_RGBA["_"], _CHAR_RGBA["-"]

    for i in range(2, 30):
        out[0][i] = out[i][0] = bezel_hi
        out[31][i] = out[i][31] = bezel_lo

    for cy, cx in (
        (0, 0), (0, 1), (1, 0), (0, 30), (0, 31), (1, 31),
        (30, 0), (31, 0), (31, 1), (30, 31), (31, 30), (31, 31),
    ):
        out[cy][cx] = void_c

    for ry, rx in ((2, 2), (2, 28), (28, 2), (28, 28)):
        out[ry][rx] = rivet_c
        out[ry][rx + 1] = out[ry + 1][rx] = bezel_hi
        out[ry + 1][rx + 1] = bezel_lo

    if stem.startswith("seismo_"):
        for y in range(4, 28, 4):
            for x in range(3, 29):
                if out[y][x] == crt_bg_c:
                    out[y][x] = crt_grid_c

    return b"".join(bytes(rgba) for row in out for rgba in row)


def generate_all_icons() -> dict[tuple[str, int], bytes]:
    """Genera el diccionario completo {(stem, size): rgba_bytes} para los 50 iconos."""
    result: dict[tuple[str, int], bytes] = {}
    for stem in ICON_STEMS:
        result[(stem, 16)] = generate_icon_16(stem)
        result[(stem, 32)] = generate_icon_32(stem)
    return result


def write_all_icons(output_dir: Path = ICONS_DIR) -> list[Path]:
    """Escribe los 50 archivos PNG (`<stem>_16.png` y `<stem>_32.png`) en disco."""
    target_dir = Path(output_dir)
    target_dir.mkdir(parents=True, exist_ok=True)
    written: list[Path] = []
    for (stem, size), rgba_bytes in generate_all_icons().items():
        path = target_dir / f"{stem}_{size}.png"
        write_png(path, size, size, rgba_bytes)
        written.append(path)
    return written


def _verify_icons(output_dir: Path = ICONS_DIR) -> None:
    """Verifica dimensiones, unicidad y pertenencia estricta a ALL_PALETTE_SET."""
    seen_16: set[bytes] = set()
    seen_32: set[bytes] = set()
    for stem in ICON_STEMS:
        for size in ICON_SIZES:
            path = output_dir / f"{stem}_{size}.png"
            w, h, data = read_png(path)
            if (w, h) != (size, size) or len(data) != size * size * 4:
                raise ValueError(f"Dimensiones inválidas en {path}")
            for idx in range(0, len(data), 4):
                px = (data[idx], data[idx + 1], data[idx + 2], data[idx + 3])
                if px not in ALL_PALETTE_SET:
                    raise ValueError(f"Píxel fuera de ALL_PALETTE_SET en {path}: {px}")
            (seen_16 if size == 16 else seen_32).add(data)
    if len(seen_16) != len(ICON_STEMS) or len(seen_32) != len(ICON_STEMS):
        raise ValueError("Existen iconos duplicados en 16×16 o 32×32")


def main() -> int:
    """Punto de entrada CLI para generar y verificar los 50 iconos de interfaz."""
    paths = write_all_icons(ICONS_DIR)
    _verify_icons(ICONS_DIR)
    print(f"Generados y verificados {len(paths)} iconos PNG en {ICONS_DIR}")
    return 0


__all__ = [
    "ALL_PALETTE",
    "ALL_PALETTE_SET",
    "ICONS_DIR",
    "ICON_SIZES",
    "ICON_STEMS",
    "encode_png",
    "generate_all_icons",
    "generate_icon_16",
    "generate_icon_32",
    "read_png",
    "ui_color",
    "write_all_icons",
    "write_png",
]

if __name__ == "__main__":
    raise SystemExit(main())
