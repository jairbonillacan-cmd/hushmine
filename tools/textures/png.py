"""Codificador y decodificador PNG RGBA8 puro, fuente 5×7 y hoja de contacto para HUSHMINE.

Implementa lectura y escritura de archivos PNG de 8 bits por canal (RGBA, tipo de
color 6) usando únicamente la biblioteca estándar de Python (`zlib` y `struct`),
junto con una fuente bitmap de 5×7 píxeles hecha a mano y el generador de hojas
de contacto para inspección de detalle (×8) y continuidad de mosaico (2×2).
"""

from __future__ import annotations

import struct
import zlib
from pathlib import Path

try:
    from .palette import PALETTE
except ImportError:
    from palette import PALETTE  # type: ignore[no-redef]

# Firma estándar de 8 bytes de todo archivo PNG válido (RFC 2083).
PNG_SIGNATURE: bytes = b"\x89PNG\r\n\x1a\n"

# Fuente bitmap 5×7 hecha a mano. Cada tupla contiene 7 filas de 5 bits (bit 4 a
# la izquierda, bit 0 a la derecha) para rotular hojas de contacto sin dependencias.
FONT_5X7: dict[str, tuple[int, int, int, int, int, int, int]] = {
    "A": (0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001),
    "B": (0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110),
    "C": (0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110),
    "D": (0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100),
    "E": (0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111),
    "F": (0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000),
    "G": (0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110),
    "H": (0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001),
    "I": (0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110),
    "J": (0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100),
    "K": (0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001),
    "L": (0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111),
    "M": (0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001),
    "N": (0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001),
    "O": (0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110),
    "P": (0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000),
    "Q": (0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101),
    "R": (0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001),
    "S": (0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110),
    "T": (0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100),
    "U": (0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110),
    "V": (0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100),
    "W": (0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001),
    "X": (0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001),
    "Y": (0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100),
    "Z": (0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111),
    "0": (0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110),
    "1": (0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110),
    "2": (0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111),
    "3": (0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110),
    "4": (0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010),
    "5": (0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110),
    "6": (0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110),
    "7": (0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000),
    "8": (0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110),
    "9": (0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110),
    "_": (0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111),
    "-": (0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000),
    ".": (0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100),
    ":": (0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000),
    "/": (0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000),
    "#": (0b01010, 0b11111, 0b01010, 0b01010, 0b11111, 0b01010, 0b00000),
    " ": (0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000),
}


def _make_chunk(chunk_type: bytes, chunk_data: bytes) -> bytes:
    """Empaqueta un bloque PNG con longitud big-endian, tipo, datos y CRC32."""
    crc = zlib.crc32(chunk_type + chunk_data) & 0xFFFFFFFF
    return struct.pack(">I", len(chunk_data)) + chunk_type + chunk_data + struct.pack(">I", crc)


def _paeth_predictor(a: int, b: int, c: int) -> int:
    """Calcula el predictor lineal de Paeth (RFC 2083, sección 6.6)."""
    p = a + b - c
    pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
    if pa <= pb and pa <= pc:
        return a
    return b if pb <= pc else c


def encode_png(width: int, height: int, rgba_bytes: bytes) -> bytes:
    """Codifica un buffer RGBA8 en formato PNG determinista sin librerías externas."""
    if width <= 0 or height <= 0:
        raise ValueError(f"Dimensiones PNG inválidas: {width}x{height}")
    if len(rgba_bytes) != width * height * 4:
        raise ValueError(f"Longitud RGBA8 inválida: {len(rgba_bytes)} != {width * height * 4}")

    ihdr_data = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    stride = width * 4
    raw_scanlines = b"".join(
        b"\x00" + rgba_bytes[row * stride : (row + 1) * stride] for row in range(height)
    )
    compressed_idat = zlib.compress(raw_scanlines, level=9)
    return (
        PNG_SIGNATURE
        + _make_chunk(b"IHDR", ihdr_data)
        + _make_chunk(b"IDAT", compressed_idat)
        + _make_chunk(b"IEND", b"")
    )


def decode_png(png_bytes: bytes) -> tuple[int, int, bytes]:
    """Decodifica un archivo PNG RGBA8 verificando firma, CRC32 y los 5 filtros PNG."""
    if len(png_bytes) < 8 or png_bytes[:8] != PNG_SIGNATURE:
        raise ValueError("Firma de cabecera PNG inválida")

    offset = 8
    width = height = 0
    ihdr_seen = iend_seen = False
    idat_parts: list[bytes] = []

    while offset < len(png_bytes):
        if offset + 8 > len(png_bytes):
            raise ValueError("Cabecera de chunk PNG truncada")
        length, chunk_type = struct.unpack(">I4s", png_bytes[offset : offset + 8])
        data_start = offset + 8
        data_end = data_start + length
        crc_end = data_end + 4
        if crc_end > len(png_bytes):
            raise ValueError(f"Datos de chunk PNG truncados en {chunk_type!r}")

        chunk_data = png_bytes[data_start:data_end]
        expected_crc = struct.unpack(">I", png_bytes[data_end:crc_end])[0]
        if (zlib.crc32(chunk_type + chunk_data) & 0xFFFFFFFF) != expected_crc:
            raise ValueError(f"CRC32 inválido en chunk {chunk_type!r}")

        if chunk_type == b"IHDR":
            if ihdr_seen or length != 13:
                raise ValueError("Chunk IHDR duplicado o con longitud inválida")
            width, height, bit_depth, color_type, comp, filt, interlace = struct.unpack(
                ">IIBBBBB", chunk_data
            )
            if width <= 0 or height <= 0:
                raise ValueError(f"Dimensiones IHDR inválidas: {width}x{height}")
            if (bit_depth, color_type, comp, filt, interlace) != (8, 6, 0, 0, 0):
                raise ValueError("Solo se soporta PNG RGBA8 sin entrelazado (color_type=6)")
            ihdr_seen = True
        elif chunk_type == b"IDAT":
            if not ihdr_seen:
                raise ValueError("Chunk IDAT encontrado antes de IHDR")
            idat_parts.append(chunk_data)
        elif chunk_type == b"IEND":
            if length != 0:
                raise ValueError("Chunk IEND debe tener longitud cero")
            iend_seen = True
            break

        offset = crc_end

    if not ihdr_seen or not iend_seen or not idat_parts:
        raise ValueError("Archivo PNG incompleto: faltan IHDR, IDAT o IEND")

    raw = zlib.decompress(b"".join(idat_parts))
    stride = width * 4
    if len(raw) != height * (1 + stride):
        raise ValueError(f"Longitud descomprimida inválida: {len(raw)}")

    out = bytearray(width * height * 4)
    for y in range(height):
        row_raw = y * (1 + stride)
        filter_type = raw[row_raw]
        scanline = raw[row_raw + 1 : row_raw + 1 + stride]
        row_out = y * stride
        prev_out = (y - 1) * stride

        for i in range(stride):
            x_byte = scanline[i]
            a = out[row_out + i - 4] if i >= 4 else 0
            b = out[prev_out + i] if y > 0 else 0
            c = out[prev_out + i - 4] if (y > 0 and i >= 4) else 0

            if filter_type == 0:
                val = x_byte
            elif filter_type == 1:
                val = (x_byte + a) & 0xFF
            elif filter_type == 2:
                val = (x_byte + b) & 0xFF
            elif filter_type == 3:
                val = (x_byte + ((a + b) >> 1)) & 0xFF
            elif filter_type == 4:
                val = (x_byte + _paeth_predictor(a, b, c)) & 0xFF
            else:
                raise ValueError(f"Filtro PNG desconocido: {filter_type}")
            out[row_out + i] = val

    return (width, height, bytes(out))


def write_png(path: str | Path, width: int, height: int, rgba_bytes: bytes) -> None:
    """Crea el directorio padre si no existe y escribe una imagen PNG RGBA8 en disco."""
    target = Path(path)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(encode_png(width, height, rgba_bytes))


def read_png(path: str | Path) -> tuple[int, int, bytes]:
    """Lee un archivo PNG desde disco y devuelve (width, height, rgba_bytes)."""
    return decode_png(Path(path).read_bytes())


def _fill_rect(
    canvas: bytearray,
    canvas_w: int,
    canvas_h: int,
    x: int,
    y: int,
    w: int,
    h: int,
    rgba: tuple[int, int, int, int],
) -> None:
    """Rellena un rectángulo sólido dentro del lienzo RGBA8 con recorte seguro."""
    x0, y0 = max(0, x), max(0, y)
    x1, y1 = min(canvas_w, x + w), min(canvas_h, y + h)
    if x0 >= x1 or y0 >= y1:
        return
    row_bytes = bytes(rgba) * (x1 - x0)
    span = (x1 - x0) * 4
    for py in range(y0, y1):
        start = (py * canvas_w + x0) * 4
        canvas[start : start + span] = row_bytes


def draw_text(
    canvas: bytearray,
    canvas_w: int,
    canvas_h: int,
    x: int,
    y: int,
    text: str,
    rgba: tuple[int, int, int, int],
    scale: int = 1,
) -> None:
    """Dibuja texto en mayúsculas usando la fuente bitmap 5×7 a la escala indicada."""
    if scale <= 0:
        return
    cursor_x = x
    for ch in text.upper():
        glyph = FONT_5X7.get(ch, FONT_5X7[" "])
        for row_idx in range(7):
            row_bits = glyph[row_idx]
            if row_bits == 0:
                continue
            for col_idx in range(5):
                if (row_bits >> (4 - col_idx)) & 1:
                    _fill_rect(
                        canvas,
                        canvas_w,
                        canvas_h,
                        cursor_x + col_idx * scale,
                        y + row_idx * scale,
                        scale,
                        scale,
                        rgba,
                    )
        cursor_x += 6 * scale


def _blit_scaled_tile(
    canvas: bytearray,
    canvas_w: int,
    canvas_h: int,
    dest_x: int,
    dest_y: int,
    tex_rgba: bytes,
    grid_repeats: int,
    pixel_scale: int,
) -> None:
    """Copia una textura 16×16 repetida grid_repeats×grid_repeats y ampliada."""
    logical_size = 16 * grid_repeats
    for ly in range(logical_size):
        src_y = ly & 15
        for lx in range(logical_size):
            src_x = lx & 15
            idx = (src_y * 16 + src_x) * 4
            color = (tex_rgba[idx], tex_rgba[idx + 1], tex_rgba[idx + 2], tex_rgba[idx + 3])
            _fill_rect(
                canvas,
                canvas_w,
                canvas_h,
                dest_x + lx * pixel_scale,
                dest_y + ly * pixel_scale,
                pixel_scale,
                pixel_scale,
                color,
            )


def render_contact_sheet(
    textures: list[tuple[str, bytes]],
    columns: int = 5,
    scale: int = 8,
) -> tuple[int, int, bytes]:
    """Genera una hoja de contacto con vista ×scale (izquierda) y mosaico 2×2 (derecha)."""
    if not textures:
        raise ValueError("La lista de texturas para la hoja de contacto está vacía")
    if columns <= 0 or scale < 2:
        raise ValueError("Parámetros inválidos: se requiere columns > 0 y scale >= 2")

    for label, tex_bytes in textures:
        if len(tex_bytes) != 16 * 16 * 4:
            raise ValueError(f"La textura {label!r} no mide 16×16 RGBA8 ({len(tex_bytes)} bytes)")

    zoom_scale = scale
    tile_scale = max(1, scale // 2)
    preview_w = 16 * zoom_scale
    tile_w = 32 * tile_scale
    inner_gap, pad = 12, 12
    text_scale = 2 if scale >= 6 else 1

    max_label_px = max(max(0, len(lbl) * 6 * text_scale - text_scale) for lbl, _ in textures)
    pair_w = preview_w + inner_gap + tile_w
    content_w = max(pair_w, max_label_px + 8)
    cell_w = content_w + 2 * pad
    header_h = 12
    label_area_h = 7 * text_scale + 8
    preview_h = max(preview_w, tile_w)
    cell_h = pad + header_h + preview_h + 10 + label_area_h + pad

    outer_margin, grid_gap = 16, 10
    rows = (len(textures) + columns - 1) // columns
    sheet_w = outer_margin * 2 + columns * cell_w + (columns - 1) * grid_gap
    sheet_h = outer_margin * 2 + rows * cell_h + (rows - 1) * grid_gap

    bg_void = PALETTE["void_black"]
    card_bg = PALETTE["soot_dark"]
    frame_col = PALETTE["slate_dark"]
    text_col = PALETTE["diamond_white"]
    accent_col = PALETTE["sodium_warm"]

    canvas = bytearray(bytes(bg_void) * (sheet_w * sheet_h))

    for idx, (label, tex_rgba) in enumerate(textures):
        col, row = idx % columns, idx // columns
        cx = outer_margin + col * (cell_w + grid_gap)
        cy = outer_margin + row * (cell_h + grid_gap)

        # Marco exterior de la tarjeta y fondo oscuro de celda
        _fill_rect(canvas, sheet_w, sheet_h, cx, cy, cell_w, cell_h, frame_col)
        _fill_rect(canvas, sheet_w, sheet_h, cx + 1, cy + 1, cell_w - 2, cell_h - 2, card_bg)

        lx = cx + (cell_w - pair_w) // 2
        ly = cy + pad + header_h
        rx = lx + preview_w + inner_gap

        # Subtítulos técnicos de cada recuadro (zoom individual vs mosaico 2×2)
        draw_text(canvas, sheet_w, sheet_h, lx, cy + pad, f"#{idx:02d} X{zoom_scale}", accent_col, 1)
        draw_text(canvas, sheet_w, sheet_h, rx, cy + pad, "2X2 TILE", accent_col, 1)

        # Marcos de 1 píxel alrededor de las dos muestras
        _fill_rect(canvas, sheet_w, sheet_h, lx - 1, ly - 1, preview_w + 2, preview_w + 2, frame_col)
        _fill_rect(canvas, sheet_w, sheet_h, rx - 1, ly - 1, tile_w + 2, tile_w + 2, frame_col)

        # Muestra izquierda (16×16 ampliada ×scale) y muestra derecha (mosaico 2×2)
        _blit_scaled_tile(canvas, sheet_w, sheet_h, lx, ly, tex_rgba, 1, zoom_scale)
        _blit_scaled_tile(canvas, sheet_w, sheet_h, rx, ly, tex_rgba, 2, tile_scale)

        # Nombre completo del bloque centrado bajo ambas muestras, sin truncar
        label_px = max(0, len(label) * 6 * text_scale - text_scale)
        tx = cx + (cell_w - label_px) // 2
        ty = ly + preview_h + 10
        draw_text(canvas, sheet_w, sheet_h, tx, ty, label, text_col, scale=text_scale)

    return (sheet_w, sheet_h, bytes(canvas))
