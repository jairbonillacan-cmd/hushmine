// Terreno de vóxeles: cada instancia es una cara empaquetada en 8 bytes
// (ver grisu_voxel::PackedFace) que se expande aquí en dos triángulos.

struct Globals {
    view_proj: mat4x4<f32>,
    // xyz: ojo de la cámara; w: alcance de la lámpara del casco.
    eye: vec4<f32>,
    // rgb: color de la niebla; a: distancia a la que la niebla es total.
    fog: vec4<f32>,
    // xyz: bloque señalado; w: 1 si hay bloque señalado.
    selected: vec4<i32>,
    // Segundos desde el arranque, para animar la lava.
    time: f32,
    // Multiplicador de la luz de cielo (0..1) según la hora del día.
    daylight: f32,
    // Relleno hasta el tamaño del uniforme (múltiplo de 16).
    _pad1: f32,
    _pad2: f32,
};

// Lava: identificador de bloque, capa de su fotograma 0 y capa del primer
// fotograma extra (ver hush_art::block_textures: 17 capas de bloque y los
// fotogramas 1-3 de la lava al final del array).
const LAVA_ID: u32 = 7u;
const LAVA_LAYER: u32 = 6u;
const LAVA_EXTRA_LAYER: u32 = 17u;
const LAVA_FRAMES: u32 = 4u;
// Bloques del modo historia: sus texturas van despues de los PNG (17 capas)
// y los fotogramas extra de la lava (3). Ver hush_art::STORY_LAYER_BASE.
const STORY_ID_MIN: u32 = 18u;
const STORY_LAYER_BASE: u32 = 20u;

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var block_textures: texture_2d_array<f32>;
@group(0) @binding(2) var block_sampler: sampler;

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
    // Luz de bloque de la celda de delante de la cara, de 0 a 1.
    @location(5) @interpolate(flat) block_light: f32,
    // Luz de cielo de la celda de delante de la cara, de 0 a 1.
    @location(6) @interpolate(flat) sky_light: f32,
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

    let block = vec3<f32>(
        f32(face.a & 0x7Fu),
        f32((face.a >> 7u) & 0x7Fu),
        f32((face.a >> 14u) & 0x7Fu),
    );
    let dir = (face.a >> 21u) & 0x7u;
    let ao = vec4<u32>(
        (face.a >> 24u) & 0x3u,
        (face.a >> 26u) & 0x3u,
        (face.a >> 28u) & 0x3u,
        (face.a >> 30u) & 0x3u,
    );

    var corner = normal_order[vertex];
    if ao.y + ao.w > ao.x + ao.z {
        corner = flipped_order[vertex];
    }
    let s = corner_signs[corner];
    let n = normals[dir];
    let local = 0.5 * (n + us[dir] * s.x + vs[dir] * s.y);
    let world = block + vec3(0.5) + local;

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
    out.highlight = select(0.0, 1.0, sel.w == 1 && all(vec3<i32>(block) == sel.xyz));
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let texel = textureSample(block_textures, block_sampler, in.uv, in.layer).rgb;
    let dist = distance(in.world, globals.eye.xyz);
    // Luz del mundo: cielo modulado por la hora y emisores de bloque, con
    // un mínimo para no ver negro puro; y la lámpara del casco, que alumbra
    // alrededor del jugador.
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
