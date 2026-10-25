// Terreno de vóxeles: cada instancia es una cara empaquetada en 8 bytes
// (ver grisu_voxel::PackedFace) que se expande aquí en dos triángulos.
//
// La posición de la cara es local a su chunk; el grupo 1 aporta el origen
// del chunk relativo al ancla de la cámara, así que las coordenadas del
// mundo pueden ser enormes o negativas sin perder precisión de f32.

struct Globals {
    view_proj: mat4x4<f32>,
    // xyz: ojo de la cámara; w: alcance de la lámpara del casco.
    eye: vec4<f32>,
    // rgb: color de la niebla; a: distancia a la que la niebla es total.
    fog: vec4<f32>,
    // xyz: bloque señalado, relativo al ancla; w: 1 si hay bloque señalado.
    selected: vec4<i32>,
    // Segundos desde el arranque, para animar la lava.
    time: f32,
    // Vista de depuración (DebugView en Rust); 0 = dibujo normal.
    debug_view: u32,
    // Multiplicador de la luz de cielo (0..1) según la hora del día.
    daylight: f32,
    // Relleno hasta el tamaño del uniforme (múltiplo de 16). Los offsets de
    // Rust y WGSL coinciden: view_proj 0..64, eye 64..80, fog 80..96,
    // selected 96..112, time 112..116, debug_view 116..120, daylight
    // 120..124, pad 124..128.
    _pad1: u32,
};

// Lava: identificador de bloque, capa de su fotograma 0 y capa del primer
// fotograma extra (ver hush_art::block_textures: 17 capas de bloque y los
// fotogramas 1-3 de la lava al final del array).
const LAVA_ID: u32 = 7u;
const LAVA_LAYER: u32 = 6u;
const LAVA_EXTRA_LAYER: u32 = 17u;
const LAVA_FRAMES: u32 = 4u;
// Bloques del modo Historia: sus texturas van después de los PNG (17 capas)
// y los fotogramas extra de la lava (3). Ver hush_art::STORY_LAYER_BASE.
const STORY_ID_MIN: u32 = 18u;
const STORY_LAYER_BASE: u32 = 20u;

// Origen del chunk que se dibuja, elegido con desplazamiento dinámico.
struct ChunkOrigin {
    // xyz: primer bloque del chunk relativo al ancla. w: sin uso.
    origin: vec4<i32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var block_textures: texture_2d_array<f32>;
@group(0) @binding(2) var block_sampler: sampler;
@group(1) @binding(0) var<uniform> chunk: ChunkOrigin;

struct FaceIn {
    @location(0) a: u32,
    @location(1) b: u32,
};

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) layer: u32,
    @location(2) light: f32,
    @location(3) world: vec3<f32>,
    @location(4) @interpolate(flat) highlight: f32,
    // Luz de la celda de delante de la cara, de 0 a 1.
    @location(5) @interpolate(flat) block_light: f32,
    // Índice 0..5 de la normal de la cara; las caras solo existen en el
    // vertex shader, así que las vistas de depuración la pasan plana.
    @location(6) @interpolate(flat) dir: u32,
    // Factor PURO de oclusión ambiental de la esquina (ao_curve, sin el
    // sombreado por dirección): lo pinta la vista `ao`, interpolado entre
    // esquinas igual que `light` para que se vea el degradado real.
    @location(7) ao: f32,
    // Tinte damero del chunk; el grupo 1 solo es visible en el vertex
    // shader, así que el color de la vista `chunks` se calcula aquí.
    @location(8) @interpolate(flat) chunk_tint: vec3<f32>,
    // Luz de cielo de la celda de delante de la cara, de 0 a 1 (igual que
    // block_light en las mallas sin luz doble).
    @location(9) @interpolate(flat) sky_light: f32,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex: u32, face: FaceIn) -> VertexOut {
    // Debe coincidir con FACE_AXES en grisu-voxel/src/mesh.rs.
    var normals = array<vec3<f32>, 6>(
        vec3(1.0, 0.0, 0.0), vec3(-1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0), vec3(0.0, -1.0, 0.0),
        vec3(0.0, 0.0, 1.0), vec3(0.0, 0.0, -1.0),
    );
    var us = array<vec3<f32>, 6>(
        vec3(0.0, 0.0, -1.0), vec3(0.0, 0.0, 1.0),
        vec3(1.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0), vec3(-1.0, 0.0, 0.0),
    );
    var vs = array<vec3<f32>, 6>(
        vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0),
        vec3(0.0, 0.0, -1.0), vec3(0.0, 0.0, 1.0),
        vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0),
    );
    // Sombreado plano por dirección: arriba claro, abajo oscuro.
    var shades = array<f32, 6>(0.80, 0.80, 1.00, 0.50, 0.65, 0.65);
    var corner_signs = array<vec2<f32>, 4>(
        vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0),
    );
    var ao_curve = array<f32, 4>(0.40, 0.62, 0.82, 1.00);
    // Dos triángulos por cara; la diagonal se elige según la oclusión para
    // evitar el artefacto de interpolación anisótropa.
    var normal_order = array<u32, 6>(0u, 1u, 2u, 0u, 2u, 3u);
    var flipped_order = array<u32, 6>(1u, 2u, 3u, 1u, 3u, 0u);

    // Posición local al chunk: 4 bits por eje.
    let cell = vec3<u32>(face.a & 0xFu, (face.a >> 4u) & 0xFu, (face.a >> 8u) & 0xFu);
    let dir = (face.a >> 12u) & 0x7u;
    // Oclusión de cada esquina: 2 bits desde el bit 15.
    let ao = vec4<u32>(
        (face.a >> 15u) & 0x3u,
        (face.a >> 17u) & 0x3u,
        (face.a >> 19u) & 0x3u,
        (face.a >> 21u) & 0x3u,
    );

    var corner = normal_order[vertex];
    if ao.y + ao.w > ao.x + ao.z {
        corner = flipped_order[vertex];
    }
    let s = corner_signs[corner];
    let n = normals[dir];
    let local = 0.5 * (n + us[dir] * s.x + vs[dir] * s.y);
    let world = vec3<f32>(chunk.origin.xyz) + vec3<f32>(cell) + vec3(0.5) + local;

    var out: VertexOut;
    out.clip = globals.view_proj * vec4<f32>(world, 1.0);
    out.uv = vec2(0.5 + 0.5 * s.x, 0.5 - 0.5 * s.y);
    // La capa i es el bloque i + 1; la lava (id 7) elige su fotograma con
    // el tiempo, a 4 fotogramas por segundo: el 0 está en LAVA_LAYER y los
    // demás en las capas extra del final. Con select (sin ramas) el
    // muestreo se queda en flujo de control uniforme.
    let block_id = face.b & 0xFFu;
    let frame = u32(max(globals.time, 0.0) * 4.0) % LAVA_FRAMES;
    let lava = select(LAVA_LAYER, LAVA_EXTRA_LAYER + frame - 1u, frame > 0u);
    let story = STORY_LAYER_BASE + block_id - STORY_ID_MIN;
    let plain = select(block_id - 1u, story, block_id >= STORY_ID_MIN);
    out.layer = select(plain, lava, block_id == LAVA_ID);
    out.light = shades[dir] * ao_curve[ao[corner]];
    out.block_light = f32((face.b >> 8u) & 0xFu) / 15.0;
    out.sky_light = f32((face.b >> 12u) & 0xFu) / 15.0;
    out.world = world;
    let sel = globals.selected;
    out.highlight = select(
        0.0,
        1.0,
        sel.w == 1 && all(chunk.origin.xyz + vec3<i32>(cell) == sel.xyz),
    );
    // Datos para las vistas de depuración: dirección de la cara, oclusión
    // ambiental pura de la esquina y damero 3D por chunk (los orígenes son
    // múltiplos de 16, así que >> 4 da la coordenada en la rejilla de
    // chunks y vecinos inmediatos quedan con tinte distinto).
    out.dir = dir;
    out.ao = ao_curve[ao[corner]];
    let parity = ((chunk.origin.x + chunk.origin.y + chunk.origin.z) >> 4) & 1;
    out.chunk_tint = select(
        vec3<f32>(1.0, 0.82, 0.55),
        vec3<f32>(0.55, 0.72, 1.0),
        parity == 1,
    );
    return out;
}

// Vista `light`: mapa de calor de la luz de bloque (0..1) con dos tramos,
// negro → ámbar hasta 0.5 y ámbar → blanco hasta 1.
fn debug_light(l: f32) -> vec3<f32> {
    let amber = vec3<f32>(1.0, 0.55, 0.1);
    let lower = clamp(l / 0.5, 0.0, 1.0);
    let upper = clamp((l - 0.5) / 0.5, 0.0, 1.0);
    return mix(mix(vec3<f32>(0.0), amber, lower), vec3<f32>(1.0), upper);
}

// Vista `sky`: hoy no existe canal de luz de cielo, así que representamos el
// nivel de luz disponible con azul frío; usará la luz de cielo real cuando
// exista.
fn debug_sky(l: f32) -> vec3<f32> {
    return vec3<f32>(0.25, 0.5, 1.0) * l;
}

// Vista `ao`: la oclusión ambiental pura de la esquina, en escala de grises.
fn debug_ao(ao: f32) -> vec3<f32> {
    return vec3<f32>(ao);
}

// Vista `normals`: color por dirección de la cara (n * 0.5 + 0.5: +X
// rojizo claro, -X azulado, etc.). Debe coincidir con `normals` de vs_main.
fn debug_normals(dir: u32) -> vec3<f32> {
    var normals = array<vec3<f32>, 6>(
        vec3(1.0, 0.0, 0.0), vec3(-1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0), vec3(0.0, -1.0, 0.0),
        vec3(0.0, 0.0, 1.0), vec3(0.0, 0.0, -1.0),
    );
    return normals[dir] * 0.5 + vec3<f32>(0.5);
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let texel = textureSample(block_textures, block_sampler, in.uv, in.layer).rgb;
    // Vistas de depuración (docs/graficos/04-shaders.md, sección 12):
    // muestran un canal interno, ignoran textura, niebla, lámpara y
    // resaltado, y devuelven pronto. El control de flujo depende solo de un
    // uniforme, así que el muestreo de arriba sigue siendo válido.
    if globals.debug_view > 0u {
        // 5 Chunks es el valor por defecto; los demás lo sobrescriben.
        var color = in.chunk_tint;
        if globals.debug_view == 1u {
            color = debug_light(in.block_light);
        } else if globals.debug_view == 2u {
            color = debug_sky(in.sky_light);
        } else if globals.debug_view == 3u {
            color = debug_ao(in.ao);
        } else if globals.debug_view == 4u {
            color = debug_normals(in.dir);
        }
        return vec4<f32>(color, 1.0);
    }
    let dist = distance(in.world, globals.eye.xyz);
    // Luz del mundo (cielo y emisores) con un mínimo para no ver negro puro,
    // y la lámpara del casco, que alumbra alrededor del jugador.
    // Con luz doble, el cielo se atenúa con la hora; los emisores, no.
    let world_light = max(in.block_light, in.sky_light * globals.daylight);
    let ambient = 0.04 + 0.96 * pow(world_light, 1.6);
    let lamp_radius = globals.eye.w;
    var lamp = 0.0;
    if lamp_radius > 0.0 {
        lamp = 0.9 * (1.0 - smoothstep(1.0, lamp_radius, dist));
    }
    var color = texel * in.light * max(ambient, lamp);
    // Bloque señalado: más claro y con borde.
    if in.highlight > 0.5 {
        let edge = min(min(in.uv.x, 1.0 - in.uv.x), min(in.uv.y, 1.0 - in.uv.y));
        color = select(color * 1.35 + vec3(0.04), vec3(0.95, 0.9, 0.7), edge < 0.04);
    }
    // Niebla: la mina se pierde en la oscuridad.
    let fog = smoothstep(globals.fog.a * 0.25, globals.fog.a, dist);
    return vec4<f32>(mix(color, globals.fog.rgb, fog), 1.0);
}
