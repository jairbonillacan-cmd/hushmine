"""Verificador de plantillas de estructuras y ensamblador determinista de planos.

Equivalente en Python de `hush-cli world piezas --check` (docs/mundo/
03-estructuras.md, secciones 2 y 3): valida familias y piezas contra la
Convencion A, comprueba que cada conector abierto tiene cierre compatible y
ensambla un plano con RNG entero splitmix64 (sin `random`, sin decimales ni
reloj). Uso: `python tools/estructuras/check_structures.py` (codigo 0 si va bien).
"""

import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "tools" / "balance"))

import ron_lite  # noqa: E402

# Convencion A de data/blocks.ron: ids de bloque 0 a 17.
BLOCK_ID_MAX = 17
KINDS = ("gallery", "shaft", "door", "street", "track")
DIRS = ("north", "south", "east", "west", "up", "down")
OPPOSITE = {"north": "south", "south": "north", "east": "west",
            "west": "east", "up": "down", "down": "up"}
ANCHORS = ("surface", "underground", "hanging", "floodable")
AGES = ("recent", "old", "ancient")
REQUIRED_FAMILY_IDS = ("ancient_mine", "prospect_cut", "consolidated_mine",
                       "arrival_village", "still_hamlet", "watchtower")
FAMILIAS_PATH = REPO_ROOT / "data" / "estructuras" / "familias.ron"
PIEZAS_DIR = REPO_ROOT / "assets" / "estructuras"

# Cara exterior que exige cada direccion: (eje, la cara es el cero del eje).
_FACE_OF = {"north": (2, True), "south": (2, False), "west": (0, True),
            "east": (0, False), "down": (1, True), "up": (1, False)}


def load_familias() -> dict:
    """Lee el catalogo de familias y lo indexa por id."""
    data = ron_lite.load(FAMILIAS_PATH)
    return {familia["id"]: familia for familia in data["families"]}


def load_pieces() -> dict:
    """Lee todas las piezas en orden determinista; el id debe ser el nombre."""
    piezas: dict = {}
    for path in sorted(PIEZAS_DIR.rglob("*.ron")):
        pieza = ron_lite.load(path)
        if pieza["id"] != path.stem:
            raise ValueError(f"la pieza {pieza['id']!r} vive en el archivo {path.name!r}")
        if pieza["id"] in piezas:
            raise ValueError(f"id de pieza duplicado: {pieza['id']!r}")
        piezas[pieza["id"]] = pieza
    return piezas


def _is_int(value) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def _connector_on_face(pos: list, size: list, direction: str) -> bool:
    """Comprueba que pos va EN la cara exterior que exige su direccion y
    dentro de la caja de la pieza en los otros dos ejes (0..size, como los
    fills): un conector fuera alinearia piezas que no se tocan."""
    axis, cara_cero = _FACE_OF[direction]
    if pos[axis] != (0 if cara_cero else size[axis]):
        return False
    return all(0 <= pos[i] <= size[i] for i in range(3) if i != axis)


def validate_familias(familias: dict) -> list:
    """Valida el catalogo: las 6 familias exactas y todos sus campos."""
    errores: list = []
    faltan = [f for f in REQUIRED_FAMILY_IDS if f not in familias]
    sobran = [f for f in familias if f not in REQUIRED_FAMILY_IDS]
    if faltan:
        errores.append(f"faltan familias: {', '.join(faltan)}")
    if sobran:
        errores.append(f"familias inesperadas: {', '.join(sobran)}")
    for fid, familia in familias.items():
        for campo in ("spacing_chunks", "separation_chunks", "max_radius_chunks",
                      "max_pieces", "max_depth"):
            if not _is_int(familia.get(campo)) or familia.get(campo) <= 0:
                errores.append(f"{fid}: {campo} debe ser un entero positivo")
        prob = familia.get("probability_q16")
        if not _is_int(prob) or not 1 <= prob <= 65536:
            errores.append(f"{fid}: probability_q16 fuera de 1..65536")
        if familia.get("anchor") not in ANCHORS:
            errores.append(f"{fid}: anchor invalido: {familia.get('anchor')!r}")
        if familia.get("age") not in AGES:
            errores.append(f"{fid}: age invalida: {familia.get('age')!r}")
        root = familia.get("root_piece")
        if not isinstance(root, str) or not root:
            errores.append(f"{fid}: root_piece debe ser una cadena no vacia")
        biomas = familia.get("biomes")
        if (not isinstance(biomas, list) or not biomas
                or not all(isinstance(b, str) and b for b in biomas)):
            errores.append(f"{fid}: biomes debe ser una lista no vacia de cadenas")
    return errores


def validate_pieces(pieces: dict, familias: dict) -> list:
    """Valida familia, cajas de fills dentro del size y conectores en cara."""
    errores: list = []
    for pid, pieza in pieces.items():
        if pieza.get("family") not in familias:
            errores.append(f"{pid}: family desconocida: {pieza.get('family')!r}")
            continue
        size = pieza.get("size")
        if (not isinstance(size, list) or len(size) != 3
                or not all(_is_int(v) and v > 0 for v in size)):
            errores.append(f"{pid}: size deben ser 3 enteros positivos")
            continue
        for i, fill in enumerate(pieza.get("fills", [])):
            tag = f"{pid}: fill #{i}"
            if not isinstance(fill, list) or len(fill) != 7 or not all(_is_int(v) for v in fill):
                errores.append(f"{tag}: deben ser 7 enteros")
                continue
            x0, y0, z0, x1, y1, z1, block = fill
            for b0, b1, tope, eje in ((x0, x1, size[0], "x"),
                                      (y0, y1, size[1], "y"),
                                      (z0, z1, size[2], "z")):
                if not 0 <= b0 <= b1 <= tope:
                    errores.append(f"{tag}: eje {eje} fuera de 0..{tope}")
            if not 0 <= block <= BLOCK_ID_MAX:
                errores.append(f"{tag}: block_id fuera de 0..{BLOCK_ID_MAX}")
        for i, conn in enumerate(pieza.get("connectors", [])):
            tag = f"{pid}: connector #{i}"
            pos, direction = conn.get("pos"), conn.get("dir")
            if direction not in DIRS or conn.get("kind") not in KINDS:
                errores.append(f"{tag}: dir o kind invalidos")
                continue
            if not isinstance(pos, list) or len(pos) != 3 or not all(_is_int(v) for v in pos):
                errores.append(f"{tag}: pos deben ser 3 enteros")
                continue
            if not _connector_on_face(pos, size, direction):
                errores.append(f"{tag}: pos {pos} no esta en la cara {direction} de la pieza")
    return errores


def _kinds_of(pieza: dict) -> set:
    return {c["kind"] for c in pieza.get("connectors", [])}


def validate_cap_coverage(pieces: dict) -> list:
    """Cada kind con pieza normal y de cierre; todo conector abierto con cierre."""
    errores: list = []
    normales = [p for p in pieces.values() if not p.get("is_cap")]
    cierres = [p for p in pieces.values() if p.get("is_cap")]
    if not cierres:
        return ["no hay ninguna pieza de cierre (is_cap: true)"]
    for kind in KINDS:
        if not any(kind in _kinds_of(p) for p in normales):
            errores.append(f"kind {kind}: sin ninguna pieza normal")
        if not any(kind in _kinds_of(p) for p in cierres):
            errores.append(f"kind {kind}: sin pieza de cierre")
    for pieza in normales:
        for conn in pieza.get("connectors", []):
            necesaria = OPPOSITE[conn["dir"]]
            cubierto = any(
                any(x["dir"] == necesaria and x["kind"] == conn["kind"]
                    for x in cierre.get("connectors", []))
                for cierre in cierres
            )
            if not cubierto:
                errores.append(
                    f"{pieza['id']}: conector {conn['dir']}/{conn['kind']} sin cierre "
                    f"con {necesaria}/{conn['kind']}"
                )
    return errores


def validate_root_pieces(familias: dict, pieces: dict) -> list:
    """Cada familia debe apuntar a una pieza raiz que exista."""
    return [f"{fid}: la raiz {familia['root_piece']!r} no existe entre las piezas"
            for fid, familia in familias.items()
            if isinstance(familia.get("root_piece"), str)
            and familia["root_piece"] not in pieces]

# Ensamblado determinista: RNG splitmix64 y rotaciones enteras.
MASK = (1 << 64) - 1


class SplitMix64:
    """RNG entero splitmix64; reproducible por semilla y sin estado global."""

    def __init__(self, seed: int) -> None:
        self.state = seed & MASK

    def next(self) -> int:
        self.state = (self.state + 0x9E3779B97F4A7C15) & MASK
        z = self.state
        z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK
        z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK
        return z ^ (z >> 31)

    def take_int(self, limit: int) -> int:
        """Entero uniforme en 0..limit-1 (exige limit > 0)."""
        return self.next() % limit


# Giros de direcciones para r pasos de 90 grados (indice r de la tupla).
_DIR_ROT = (
    {},
    {"north": "east", "east": "south", "south": "west", "west": "north", "up": "up", "down": "down"},
    {"north": "south", "south": "north", "east": "west", "west": "east", "up": "up", "down": "down"},
    {"north": "west", "west": "south", "south": "east", "east": "north", "up": "up", "down": "down"},
)


def rotate_dir(direction: str, r: int) -> str:
    return direction if r == 0 else _DIR_ROT[r][direction]


def rotate_size(size: list, r: int) -> list:
    """Size rotado r pasos de 90 grados alrededor del eje y."""
    sx, sy, sz = size
    if r in (1, 3):
        return [sz, sy, sx]
    return [sx, sy, sz]


def rotate_fill(fill: list, size: list, r: int) -> list:
    """Caja semiabierta [x0, y0, z0, x1, y1, z1, block] rotada r pasos."""
    sx, sy, sz = size
    x0, y0, z0, x1, y1, z1, block = fill
    if r == 0:
        return [x0, y0, z0, x1, y1, z1, block]
    if r == 1:
        return [sz - z1, y0, x0, sz - z0, y1, x1, block]
    if r == 2:
        return [sx - x1, y0, sz - z1, sx - x0, y1, sz - z0, block]
    return [z0, y0, sx - x1, z1, y1, sx - x0, block]  # r == 3


def rotate_connector(connector: dict, size: list, r: int) -> dict:
    """Conector rotado r pasos: pos y dir giran, kind se conserva."""
    sx, sy, sz = size
    x, y, z = connector["pos"]
    if r == 0:
        pos = [x, y, z]
    elif r == 1:
        pos = [sz - z, y, x]
    elif r == 2:
        pos = [sx - x, y, sz - z]
    else:
        pos = [z, y, sx - x]
    return {"pos": pos, "dir": rotate_dir(connector["dir"], r),
            "kind": connector["kind"]}


def _boxes_overlap(a: list, b: list) -> bool:
    """Solape ESTRICTO de cajas; el contacto de caras no es solape (INV-W08)."""
    return (a[0] < b[3] and b[0] < a[3]
            and a[1] < b[4] and b[1] < a[4]
            and a[2] < b[5] and b[2] < a[5])


def placed_box(entry: dict, piezas: dict) -> list:
    """Caja mundo [x0, y0, z0, x1, y1, z1] de una pieza colocada y rotada."""
    rs = rotate_size(piezas[entry["id"]]["size"], entry["rotation"])
    ox, oy, oz = entry["origin"]
    return [ox, oy, oz, ox + rs[0], oy + rs[1], oz + rs[2]]


def assemble_plan(seed: int, family_id: str, piece_limit: int) -> dict:
    """Ensambla un plano por expansion en anchura desde la pieza raiz.

    Devuelve {"family", "seed", "piece_limit", "radius", "pieces", "sealed",
    "stats"}; cada pieza colocada es {"id", "rotation", "origin"}.
    """
    familia = load_familias()[family_id]  # KeyError si la familia no existe
    piezas = load_pieces()
    radius = familia["max_radius_chunks"] * 16
    rng = SplitMix64(seed)
    pool_normal = [p for p in piezas.values()
                   if p["family"] == family_id and not p.get("is_cap")]
    pool_cierres = [p for p in piezas.values() if p.get("is_cap")]
    placed: list = []
    queue: list = []  # pares (indice_de_pieza_colocada, indice_de_conector)
    sealed: list = []

    def fits(rot_size: list, origin: list) -> bool:
        ox, oy, oz = origin
        if (ox < -radius or ox + rot_size[0] > radius
                or oz < -radius or oz + rot_size[2] > radius):
            return False
        caja = [ox, oy, oz, ox + rot_size[0], oy + rot_size[1], oz + rot_size[2]]
        return all(not _boxes_overlap(caja, placed_box(e, piezas)) for e in placed)

    def candidates(pool: list, parent_dir: str, parent_kind: str, world_pos: list) -> list:
        found: list = []
        for pieza in pool:
            for rotation in range(4):
                # Primer conector (menor indice) con dir opuesta y kind igual.
                rotado = None
                for connector in pieza["connectors"]:
                    candidato = rotate_connector(connector, pieza["size"], rotation)
                    if (candidato["dir"] == OPPOSITE[parent_dir]
                            and candidato["kind"] == parent_kind):
                        rotado = candidato
                        break
                if rotado is None:
                    continue
                origin = [world_pos[k] - rotado["pos"][k] for k in range(3)]
                if fits(rotate_size(pieza["size"], rotation), origin):
                    found.append({"id": pieza["id"], "rotation": rotation, "origin": origin})
        return found

    def connector_world(pair: tuple):
        index, ci = pair
        entry = placed[index]
        pieza = piezas[entry["id"]]
        rotado = rotate_connector(pieza["connectors"][ci], pieza["size"], entry["rotation"])
        return ([entry["origin"][k] + rotado["pos"][k] for k in range(3)],
                rotado["dir"], rotado["kind"])

    def attach(pair: tuple, allow_normal: bool) -> None:
        world_pos, parent_dir, parent_kind = connector_world(pair)
        choice = None
        if allow_normal:
            normales = candidates(pool_normal, parent_dir, parent_kind, world_pos)
            if normales:
                choice = normales[rng.take_int(len(normales))]
        # Los cierres tambien respetan el limite de piezas; si no cabe, sellado.
        if choice is None and len(placed) < piece_limit:
            cierres = candidates(pool_cierres, parent_dir, parent_kind, world_pos)
            if cierres:
                choice = cierres[rng.take_int(len(cierres))]
        if choice is None:
            sealed.append({"cell": world_pos, "reason": "sin pieza compatible que quepa"})
            return
        pieza = piezas[choice["id"]]
        placed.append(choice)
        if not pieza.get("is_cap"):
            index = len(placed) - 1
            for ci in range(len(pieza["connectors"])):
                queue.append((index, ci))

    root = piezas[familia["root_piece"]]
    sx, sy, sz = root["size"]
    placed.append({"id": root["id"], "rotation": 0, "origin": [-sx // 2, 0, -sz // 2]})
    for ci in range(len(root["connectors"])):
        queue.append((0, ci))

    # Expansion normal mientras quede hueco; agotado el limite, solo cierres.
    while queue:
        attach(queue.pop(0), allow_normal=len(placed) < piece_limit)

    stats = {"placed": len(placed),
             "caps": sum(1 for e in placed if piezas[e["id"]].get("is_cap")),
             "paired": len(placed) - 1, "sealed": len(sealed)}
    return {"family": family_id, "seed": seed, "piece_limit": piece_limit,
            "radius": radius, "pieces": placed, "sealed": sealed, "stats": stats}


def check_plan_overlap(plan: dict, pieces: dict) -> list:
    """Verificacion independiente posterior: solapes estrictos entre colocadas."""
    entries = plan["pieces"]
    boxes = [placed_box(entry, pieces) for entry in entries]
    return [f"solape estricto entre {entries[i]['id']} #{i} y {entries[j]['id']} #{j}"
            for i in range(len(boxes)) for j in range(i + 1, len(boxes))
            if _boxes_overlap(boxes[i], boxes[j])]


def main() -> int:
    """Ejecuta todas las validaciones, ensambla el plano de prueba e informa."""
    familias = load_familias()
    pieces = load_pieces()
    errores: list = []
    errores += validate_familias(familias)
    errores += validate_pieces(pieces, familias)
    errores += validate_cap_coverage(pieces)
    errores += validate_root_pieces(familias, pieces)

    print("== Verificacion de plantillas de estructuras ==")
    print(f"Familias cargadas: {len(familias)} ({', '.join(sorted(familias))})")
    resumen = ", ".join(
        f"{f}: {sum(1 for p in pieces.values() if p['family'] == f)}"
        for f in sorted({p["family"] for p in pieces.values()}))
    print(f"Piezas cargadas: {len(pieces)} ({resumen})")
    for kind in KINDS:
        cierres = sorted(p["id"] for p in pieces.values()
                         if p.get("is_cap") and kind in _kinds_of(p))
        print(f"Cierres para kind {kind}: {', '.join(cierres) if cierres else '(ninguno)'}")

    plan = assemble_plan(42, "ancient_mine", 12)
    errores += check_plan_overlap(plan, pieces)
    stats = plan["stats"]
    print(f"Ensamblado (seed 42, ancient_mine, limite 12): {stats['placed']} piezas "
          f"({stats['caps']} de cierre), {stats['paired']} emparejados, "
          f"{stats['sealed']} sellados, radio {plan['radius']}")
    for entry in plan["pieces"]:
        print(f"  - {entry['id']} rot {entry['rotation']} origen {entry['origin']}")

    if errores:
        print(f"\nERRORES ({len(errores)}):")
        for error in errores:
            print(f"  ERROR: {error}")
        return 1
    print("\nTodo correcto: catalogo, piezas, cierres y plano sin solapes (INV-W08).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
