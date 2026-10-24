"""Generador procedural determinista de mapas para HUSHMINE.

Produce mapas 3D reproducibles bit a bit usando únicamente aritmética entera
de 64 bits (cero `float`), garantizando equidad competitiva entre Copper y
Cobalt, estratigrafía por profundidad, Veta Madre, Corazón y fronteras selladas.
"""

from __future__ import annotations

import argparse
from collections import deque
from dataclasses import dataclass, field
import hashlib

try:
    from tools.mapgen.rng import Xorshift64, hash3d
except ImportError:
    from rng import Xorshift64, hash3d  # type: ignore[no-redef]

BLOCK_NAMES: tuple[str, ...] = (
    "air", "dirt", "rock", "hard_rock", "ore", "gold", "diamond", "lava",
    "team_block", "force_field", "ladder", "jump_pad", "electric",
    "rail", "beacon", "explosive", "bucket_station", "bucket_post",
)
BLOCK_IDS: dict[str, int] = {name: idx for idx, name in enumerate(BLOCK_NAMES)}

AIR, DIRT, ROCK, HARD_ROCK, ORE, GOLD, DIAMOND, LAVA = 0, 1, 2, 3, 4, 5, 6, 7
TEAM_BLOCK, FORCE_FIELD, LADDER, JUMP_PAD, ELECTRIC = 8, 9, 10, 11, 12
RAIL, BEACON, EXPLOSIVE, BUCKET_STATION, BUCKET_POST = 13, 14, 15, 16, 17

# Metas y dimensiones por tamaño de equipo según `data/economy.ron`.
TEAM_GOALS: dict[int, tuple[int, int, int, int]] = {
    1: (2000, 64, 64, 64),
    2: (3500, 64, 64, 64),
    3: (5000, 64, 64, 64),
    4: (6500, 64, 64, 64),
    5: (8000, 64, 64, 64),
    6: (9500, 64, 64, 64),
    7: (11000, 96, 96, 64),
    8: (12500, 96, 96, 64),
}

GOLD_CASH_VALUE: int = 100
DIAMOND_CASH_VALUE: int = 400
ORE_UNITS_PER_BLOCK: int = 4
MOTHER_LODE_TOTAL_CASH: int = 2000
HEART_TOTAL_CASH: int = 1500


@dataclass
class GeneratedMap:
    """Mapa tridimensional generado de forma determinista."""

    seed: int
    team_size: int
    target_cash: int
    width: int
    height: int
    depth: int
    blocks: bytearray
    team_meta: dict[tuple[int, int, int], str] = field(default_factory=dict)
    copper_base: tuple[int, int, int] = (8, 52, 32)
    cobalt_base: tuple[int, int, int] = (55, 52, 32)
    copper_base_bounds: tuple[tuple[int, int, int], tuple[int, int, int]] = ((5, 51, 29), (11, 55, 35))
    cobalt_base_bounds: tuple[tuple[int, int, int], tuple[int, int, int]] = ((52, 51, 29), (58, 55, 35))
    mother_lode_center: tuple[int, int, int] = (32, 30, 32)
    heart_center: tuple[int, int, int] = (32, 6, 32)

    def index(self, x: int, y: int, z: int) -> int:
        """Índice lineal en orden `(y * depth + z) * width + x`."""
        return (y * self.depth + z) * self.width + x

    def in_bounds(self, x: int, y: int, z: int) -> bool:
        """Indica si la coordenada `(x, y, z)` pertenece al volumen del mapa."""
        return 0 <= x < self.width and 0 <= y < self.height and 0 <= z < self.depth

    def get_block(self, x: int, y: int, z: int) -> int:
        """Devuelve el identificador de bloque en `(x, y, z)`."""
        return self.blocks[self.index(x, y, z)]

    def set_block(self, x: int, y: int, z: int, block_id: int, team: str | None = None) -> None:
        """Escribe `block_id` en `(x, y, z)` y actualiza su metadato de equipo."""
        self.blocks[self.index(x, y, z)] = block_id
        if team is not None:
            self.team_meta[(x, y, z)] = team
        else:
            self.team_meta.pop((x, y, z), None)

    def to_bytes(self) -> bytes:
        """Devuelve el búfer inmutable de bloques."""
        return bytes(self.blocks)

    def sha256(self) -> str:
        """Calcula el resumen SHA-256 hexadecimal del búfer de bloques."""
        return hashlib.sha256(self.blocks).hexdigest()


def _is_reserved(x: int, y: int, z: int, w: int, d: int) -> bool:
    """Comprueba si `(x, y, z)` cae en zonas reservadas (bases, pozos, nodos centrales)."""
    cx, cz = w // 2, d // 2
    if (3 <= x <= 13 or w - 14 <= x <= w - 4) and 20 <= y <= 57 and cz - 5 <= z <= cz + 5:
        return True
    if cx - 6 <= x <= cx + 5 and abs(z - cz) <= 5 and (abs(y - 30) <= 5 or y <= 14):
        return True
    return False


def _place_symmetric_cluster(
    gmap: GeneratedMap, coords: list[tuple[int, int, int]], block_id: int
) -> int:
    """Coloca un grupo reflejado en `x <-> W - 1 - x` rodeado de roca excavable."""
    w, d, h = gmap.width, gmap.height, gmap.depth
    valid: list[tuple[int, int, int]] = []
    for x, y, z in coords:
        mx = w - 1 - x
        if not (3 <= x <= (w // 2) - 3 and 2 <= y <= h - 3 and 3 <= z <= d - 4):
            continue
        if _is_reserved(x, y, z, w, d) or _is_reserved(mx, y, z, w, d):
            continue
        if gmap.get_block(x, y, z) == LAVA or gmap.get_block(mx, y, z) == LAVA:
            continue
        valid.append((x, y, z))

    placed_per_side = 0
    for x, y, z in valid:
        mx = w - 1 - x
        for dx in (-1, 0, 1):
            for dy in (-1, 0, 1):
                for dz in (-1, 0, 1):
                    for px in (x + dx, mx - dx):
                        py, pz = y + dy, z + dz
                        if 1 <= px < w - 1 and 1 <= py < h - 1 and 1 <= pz < d - 1:
                            if gmap.get_block(px, py, pz) == HARD_ROCK:
                                gmap.set_block(px, py, pz, ROCK)
        if gmap.get_block(x, y, z) not in (GOLD, DIAMOND):
            gmap.set_block(x, y, z, block_id)
            gmap.set_block(mx, y, z, block_id)
            placed_per_side += 1
    return placed_per_side


def _build_base(gmap: GeneratedMap, bx0: int, bx1: int, cx_base: int, cz: int, team: str) -> None:
    """Construye la base de un equipo, sus accesorios y su pozo vertical con escalera."""
    for y in range(52, 56):
        for z in range(cz - 3, cz + 4):
            for x in range(bx0, bx1 + 1):
                gmap.set_block(x, y, z, AIR)
    for z in range(cz - 3, cz + 4):
        for x in range(bx0, bx1 + 1):
            gmap.set_block(x, 51, z, TEAM_BLOCK, team=team)

    door_x = bx1 if team == "copper" else bx0
    for y in range(52, 55):
        for z in range(cz - 1, cz + 2):
            gmap.set_block(door_x, y, z, FORCE_FIELD, team=team)

    gmap.set_block(cx_base, 55, cz, BEACON, team=team)
    offset = 1 if team == "copper" else -1
    gmap.set_block(cx_base - offset, 52, cz - 2, BUCKET_STATION, team=team)
    gmap.set_block(cx_base + offset, 52, cz - 2, BUCKET_POST, team=team)
    gmap.set_block(cx_base, 52, cz, AIR)

    for y in range(22, 53):
        gmap.set_block(cx_base, y, cz + 2, LADDER, team=team)
        gmap.set_block(cx_base, y, cz + 1, AIR)


def generate_map(seed: int, team_size: int = 4) -> GeneratedMap:
    """Genera un mapa competitivo completo y determinista para `seed` y `team_size`."""
    if team_size not in TEAM_GOALS:
        raise ValueError(f"team_size fuera de rango (1..8): {team_size}")

    target_cash, w, d, h = TEAM_GOALS[team_size]
    rng = Xorshift64(seed)
    cx, cz = w // 2, d // 2

    gmap = GeneratedMap(
        seed=seed, team_size=team_size, target_cash=target_cash,
        width=w, height=h, depth=d, blocks=bytearray(w * h * d),
        copper_base=(8, 52, cz), cobalt_base=(w - 9, 52, cz),
        copper_base_bounds=((5, 51, cz - 3), (11, 55, cz + 3)),
        cobalt_base_bounds=((w - 12, 51, cz - 3), (w - 6, 55, cz + 3)),
        mother_lode_center=(cx, 30, cz), heart_center=(cx, 6, cz),
    )

    # Paso 1: Estratigrafía base (tierra superficial, roca media y basalto profundo).
    for z in range(1, d - 1):
        for x in range(1, w - 1):
            sym_x = x if x < cx else (w - 1 - x)
            dirt_floor = 46 + (hash3d(sym_x, 0, z, seed) % 3)
            for y in range(1, h - 1):
                if y >= dirt_floor:
                    gmap.set_block(x, y, z, DIRT)
                elif y >= 16:
                    gmap.set_block(x, y, z, ROCK)
                else:
                    vein_h = hash3d(sym_x // 2, y // 2, z // 2, seed ^ 0x11)
                    if vein_h % 7 == 0 and (y < 8 or (sym_x % 2 == 0 and z % 2 == 0)):
                        gmap.set_block(x, y, z, HARD_ROCK)
                    else:
                        gmap.set_block(x, y, z, ROCK)

    # Paso 2: Cuevas y galerías naturales cuasi-simétricas en y in [14, 50].
    for _ in range(rng.randint(6, 9)):
        rx, ry, rz = rng.randint(8, cx - 4), rng.randint(18, 46), rng.randint(6, d - 7)
        rad_x, rad_y, rad_z = rng.randint(2, 3), rng.randint(1, 2), rng.randint(2, 3)
        for dy in range(-rad_y, rad_y + 1):
            for dz in range(-rad_z, rad_z + 1):
                for dx in range(-rad_x, rad_x + 1):
                    if dx * dx + dy * dy * 2 + dz * dz <= rad_x * rad_z + 1:
                        py, pz = ry + dy, rz + dz
                        if 14 <= py <= 50 and 3 <= pz <= d - 4:
                            for px in (rx + dx, w - 1 - (rx + dx)):
                                if 3 <= px < w - 3:
                                    gmap.set_block(px, py, pz, AIR)

    # Paso 3: Bolsas de lava profundas exclusivamente en y in [4, 24] y x in [16, W - 17].
    for _ in range(rng.randint(3, 5)):
        lx, ly, lz = rng.randint(17, cx - 5), rng.randint(6, 20), rng.randint(6, d - 7)
        if abs(lz - cz) <= 5 and lx >= cx - 7:
            lz = max(6, cz - 8)
        for dy in range(-1, 2):
            for dz in range(-2, 3):
                for dx in range(-2, 3):
                    py, pz = ly + dy, lz + dz
                    for px in (lx + dx, w - 1 - (lx + dx)):
                        if not (16 <= px <= w - 17 and 4 <= py <= 24 and 4 <= pz <= d - 5):
                            continue
                        if abs(dx) + abs(dy) + abs(dz) <= 2 and 17 <= px <= w - 18 and 5 <= py <= 23:
                            gmap.set_block(px, py, pz, LAVA)
                        elif gmap.get_block(px, py, pz) == AIR:
                            gmap.set_block(px, py, pz, ROCK)

    # Paso 4: Vetas simétricas de mena, oro y diamante más variación asimétrica controlada.
    for _ in range(24 if w == 64 else 38):
        vx, vy, vz = rng.randint(4, cx - 4), rng.randint(20, 58), rng.randint(4, d - 5)
        cluster = [(vx, vy, vz), (vx + 1, vy, vz), (vx, vy + 1, vz), (vx, vy, vz + 1), (vx - 1, vy, vz)]
        _place_symmetric_cluster(gmap, cluster, ORE)

    for _ in range(5 if w == 64 else 8):
        vx, vy, vz = rng.randint(6, cx - 5), rng.randint(10, 31), rng.randint(5, d - 6)
        _place_symmetric_cluster(gmap, [(vx, vy, vz), (vx + 1, vy, vz)], DIAMOND)

    min_gold_per_team = (target_cash * 23) // 1000
    gold_per_team, attempts = 0, 0
    while gold_per_team < min_gold_per_team and attempts < 600:
        attempts += 1
        vx, vy, vz = rng.randint(4, cx - 4), rng.randint(14, 56), rng.randint(4, d - 5)
        cluster = [(vx, vy, vz), (vx + 1, vy, vz), (vx, vy + 1, vz), (vx, vy, vz + 1)]
        gold_per_team += _place_symmetric_cluster(gmap, cluster, GOLD)

    extra_gold_total = rng.randint(2, 5)
    copper_extra = (extra_gold_total + 1) // 2
    cobalt_extra = extra_gold_total - copper_extra
    if rng.below(2) == 1:
        copper_extra, cobalt_extra = cobalt_extra, copper_extra

    for side, count in (("copper", copper_extra), ("cobalt", cobalt_extra)):
        placed, tries = 0, 0
        while placed < count and tries < 200:
            tries += 1
            ax = rng.randint(5, cx - 5) if side == "copper" else rng.randint(cx + 4, w - 6)
            ay, az = rng.randint(24, 48), rng.randint(5, d - 6)
            if not _is_reserved(ax, ay, az, w, d) and gmap.get_block(ax, ay, az) in (DIRT, ROCK):
                gmap.set_block(ax, ay, az, GOLD)
                placed += 1

    # Paso 5: Veta Madre en (W // 2, 30, D // 2) y Corazón en (W // 2, 6, D // 2).
    for dy in range(-3, 4):
        for dz in range(-3, 4):
            for dx in range(-4, 4):
                gmap.set_block(cx + dx, 30 + dy, cz + dz, ROCK)
    for gx, gy, gz in (
        (cx - 2, 30, cz - 1), (cx - 2, 30, cz), (cx - 2, 30, cz + 1),
        (cx - 1, 30, cz - 1), (cx - 1, 30, cz + 1), (cx - 1, 31, cz),
        (cx, 30, cz - 1), (cx, 30, cz + 1), (cx, 31, cz),
        (cx + 1, 30, cz - 1), (cx + 1, 30, cz), (cx + 1, 30, cz + 1),
    ):
        gmap.set_block(gx, gy, gz, GOLD)
    gmap.set_block(cx - 1, 30, cz, DIAMOND)
    gmap.set_block(cx, 30, cz, DIAMOND)

    for dy in range(-2, 4):
        for dz in range(-3, 4):
            for dx in range(-4, 4):
                gmap.set_block(cx + dx, 6 + dy, cz + dz, ROCK)
    for dz in range(-2, 3):
        for dx in range(-3, 3):
            gmap.set_block(cx + dx, 3, cz + dz, HARD_ROCK)
            gmap.set_block(cx + dx, 4, cz + dz, LAVA)
            if dx in (-3, 2) or abs(dz) == 2:
                gmap.set_block(cx + dx, 5, cz + dz, HARD_ROCK)
    for gx, gy, gz in (
        (cx - 1, 6, cz - 1), (cx - 1, 6, cz + 1), (cx - 1, 7, cz),
        (cx, 6, cz - 1), (cx, 6, cz + 1), (cx, 7, cz), (cx + 1, 6, cz),
    ):
        gmap.set_block(gx, gy, gz, GOLD)
    gmap.set_block(cx - 1, 6, cz, DIAMOND)
    gmap.set_block(cx, 6, cz, DIAMOND)
    for y in range(8, 30):
        for dz in (-1, 0, 1):
            for dx in (-2, -1, 0, 1):
                if gmap.get_block(cx + dx, y, cz + dz) in (HARD_ROCK, LAVA):
                    gmap.set_block(cx + dx, y, cz + dz, ROCK)

    # Paso 6: Bases de Copper y Cobalt, pozos verticales y galería de conexión.
    _build_base(gmap, 5, 11, 8, cz, "copper")
    _build_base(gmap, w - 12, w - 6, w - 9, cz, "cobalt")
    for x in range(8, w - 8):
        for y in (22, 30):
            if gmap.get_block(x, y, cz + 1) in (HARD_ROCK, LAVA):
                gmap.set_block(x, y, cz + 1, ROCK)

    # Paso 7: Sellado final de las 6 fronteras exteriores con HARD_ROCK (INV-09).
    for z in range(d):
        for x in range(w):
            gmap.set_block(x, 0, z, HARD_ROCK)
            gmap.set_block(x, h - 1, z, HARD_ROCK)
    for y in range(h):
        for x in range(w):
            gmap.set_block(x, y, 0, HARD_ROCK)
            gmap.set_block(x, y, d - 1, HARD_ROCK)
        for z in range(d):
            gmap.set_block(0, y, z, HARD_ROCK)
            gmap.set_block(w - 1, y, z, HARD_ROCK)

    return gmap


def _print_report(gmap: GeneratedMap) -> None:
    """Imprime las métricas del mapa generado usando únicamente aritmética entera."""
    counts = [0] * len(BLOCK_NAMES)
    for b in gmap.blocks:
        counts[b] += 1

    w, h, d = gmap.width, gmap.height, gmap.depth
    visited = bytearray(w * h * d)
    queue: deque[tuple[int, int, int]] = deque([gmap.copper_base, gmap.cobalt_base])
    for sx, sy, sz in queue:
        visited[gmap.index(sx, sy, sz)] = 1

    copper_gold_cash, cobalt_gold_cash = 0, 0
    while queue:
        x, y, z = queue.popleft()
        if gmap.get_block(x, y, z) == GOLD:
            if x < w // 2:
                copper_gold_cash += GOLD_CASH_VALUE
            else:
                cobalt_gold_cash += GOLD_CASH_VALUE
        for dx, dy, dz in ((1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)):
            nx, ny, nz = x + dx, y + dy, z + dz
            if gmap.in_bounds(nx, ny, nz):
                idx = gmap.index(nx, ny, nz)
                if not visited[idx] and gmap.blocks[idx] not in (HARD_ROCK, LAVA):
                    visited[idx] = 1
                    queue.append((nx, ny, nz))

    min_lava_dist = w + h + d
    c_b0, c_b1 = gmap.copper_base_bounds
    b_b0, b_b1 = gmap.cobalt_base_bounds
    for y in range(h):
        for z in range(d):
            for x in range(w):
                if gmap.get_block(x, y, z) == LAVA:
                    for (x0, y0, z0), (x1, y1, z1) in ((c_b0, c_b1), (b_b0, b_b1)):
                        dx = x0 - x if x < x0 else (x - x1 if x > x1 else 0)
                        dy = y0 - y if y < y0 else (y - y1 if y > y1 else 0)
                        dz = z0 - z if z < z0 else (z - z1 if z > z1 else 0)
                        min_lava_dist = min(min_lava_dist, dx + dy + dz)

    max_cash = max(copper_gold_cash, cobalt_gold_cash, 1)
    diff_bp = (abs(copper_gold_cash - cobalt_gold_cash) * 10000) // max_cash
    print(f"seed: {gmap.seed} | team_size: {gmap.team_size} | sha256: {gmap.sha256()}")
    print(f"dimensions: {w}x{d}x{h} | target_cash: {gmap.target_cash}")
    print(
        f"reachable_gold_cash: copper={copper_gold_cash} cobalt={cobalt_gold_cash} "
        f"diff={diff_bp // 100}.{diff_bp % 100:02d}% | min_base_lava_dist={min_lava_dist}"
    )
    summary = ", ".join(f"{BLOCK_NAMES[i]}={counts[i]}" for i in range(len(BLOCK_NAMES)) if counts[i])
    print(f"blocks: {summary}")


def main() -> int:
    """Punto de entrada de línea de comandos para generar e inspeccionar un mapa."""
    parser = argparse.ArgumentParser(description="Generador determinista de mapas HUSHMINE.")
    parser.add_argument("--seed", type=int, default=42, help="Semilla entera de 64 bits.")
    parser.add_argument("--team-size", type=int, default=4, help="Tamaño de equipo (1..8).")
    parser.add_argument("--report", action="store_true", help="Imprime informe de métricas.")
    args = parser.parse_args()

    gmap = generate_map(seed=args.seed, team_size=args.team_size)
    if args.report:
        _print_report(gmap)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
