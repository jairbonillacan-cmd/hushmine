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
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var block_textures: texture_2d_array<f32>;
@group(0) @binding(2) var block_sampler: sampler;
// Origen del chunk de la cara, relativo al anchor de la cámara (un vec4 por
// chunk, seleccionado con el offset dinámico del bind group 1).
@group(1) @binding(0) var<uniform> chunk_origin: vec4<f32>;

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
};

@vertex
fn vs_main(@builtin(vertex_index) vertex: u32, face: FaceIn) -> VertexOut {
    // Debe coincidir con el empaquetado de PackedFace en
    // grisu-voxel/src/mesh.rs.
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
        f32(face.a & 0xFu),
        f32((face.a >> 4u) & 0xFu),
        f32((face.a >> 8u) & 0xFu),
    );
    let dir = (face.a >> 12u) & 0x7u;
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
    // Posición relativa al anchor de la cámara: la cara es local al chunk y
    // el origen del chunk llega ya relativo (ADR 0002).
    let origin = chunk_origin.xyz;
    let world = block + origin + vec3(0.5) + local;

    var out: VertexOut;
    out.clip = globals.view_proj * vec4<f32>(world, 1.0);
    out.uv = vec2(0.5 + 0.5 * s.x, 0.5 - 0.5 * s.y);
    out.layer = (face.b & 0xFFu) - 1u;
    out.light = shades[dir] * ao_curve[ao[corner]];
    out.block_light = f32((face.b >> 8u) & 0xFu) / 15.0;
    out.world = world;
    let sel = globals.selected;
    let block_rel = block + origin;
    out.highlight =
        select(0.0, 1.0, sel.w == 1 && all(vec3<i32>(block_rel) == sel.xyz));
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let texel = textureSample(block_textures, block_sampler, in.uv, in.layer).rgb;
    let dist = distance(in.world, globals.eye.xyz);
    // Luz del mundo (cielo y emisores) con un mínimo para no ver negro puro,
    // y la lámpara del casco, que alumbra alrededor del jugador.
    let ambient = 0.04 + 0.96 * pow(in.block_light, 1.6);
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
