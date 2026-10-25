use std::time::Instant;

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use grisu_voxel::{DualLightMap, LightMap, PackedFace, VoxelWorld, mesh_chunk, mesh_chunk_dual};
use wgpu::util::DeviceExt;

use crate::texture_array::upload_texture_array;
use crate::{CLEAR_COLOR, DepthTarget, TextureData, View};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
    eye: [f32; 4],
    fog: [f32; 4],
    selected: [i32; 4],
    /// Segundos desde el arranque, para animar la lava.
    time: f32,
    /// Multiplicador de la luz de cielo (0..1) según la hora del día.
    daylight: f32,
    /// Relleno hasta el tamaño del uniforme (múltiplo de 16).
    pad: [f32; 2],
}

/// Ajustes de dibujo del terreno que cambian cada fotograma.
#[derive(Clone, Copy, Debug)]
pub struct TerrainStyle {
    /// Distancia, en bloques, a la que la niebla tapa todo.
    pub fog_distance: f32,
    /// Color de la niebla (rgb); de noche conviene oscurecerlo.
    pub fog_color: [f32; 3],
    /// Bloque señalado por el jugador, que se resalta.
    pub selected: Option<[i32; 3]>,
    /// Alcance de la lámpara del casco, en bloques (0 = apagada).
    pub lamp_radius: f32,
    /// Segundos desde el arranque; anima los bloques con fotogramas (lava).
    pub time_seconds: f32,
    /// Multiplicador de la luz de cielo (0..1) según la hora del día.
    pub daylight: f32,
}

impl Default for TerrainStyle {
    fn default() -> Self {
        Self {
            fog_distance: 48.0,
            fog_color: [
                crate::CLEAR_COLOR.r as f32,
                crate::CLEAR_COLOR.g as f32,
                crate::CLEAR_COLOR.b as f32,
            ],
            selected: None,
            lamp_radius: 14.0,
            time_seconds: 0.0,
            daylight: 1.0,
        }
    }
}

/// Malla de un chunk ya subida a la GPU.
struct ChunkMesh {
    faces: wgpu::Buffer,
    count: u32,
}

/// Estadísticas de la última actualización de mallas.
#[derive(Clone, Copy, Debug, Default)]
pub struct MeshStats {
    pub chunks_meshed: usize,
    pub faces: usize,
    /// Celdas cuya luz cambió en esta actualización.
    pub light_changed: usize,
    pub millis: f64,
}

/// Cómo se calcula la luz del terreno.
#[derive(Clone, Debug)]
enum Lighting {
    /// Un solo canal combinado (cielo y emisores juntos).
    Combined(LightMap),
    /// Canales separados de cielo y bloque para modular con el ciclo diurno.
    Dual(DualLightMap),
}

impl Lighting {
    /// Recalcula la luz y devuelve cuántas celdas cambiaron.
    fn update(&mut self, world: &mut VoxelWorld) -> usize {
        match self {
            Self::Combined(map) => map.update(world),
            Self::Dual(map) => map.update(world),
        }
    }
}

/// Dibuja un [`VoxelWorld`]: una malla por chunk, regenerada solo cuando el
/// chunk está sucio.
pub struct VoxelRenderer {
    pipeline: wgpu::RenderPipeline,
    globals: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    chunks: Vec<Option<ChunkMesh>>,
    /// Emisión por tipo de bloque; con `None` no se calcula luz.
    emission: Option<[u8; 256]>,
    light: Option<Lighting>,
    /// Luz dual (modo historia) en vez de canal único.
    dual: bool,
}

impl VoxelRenderer {
    /// `textures[i]` es la textura del bloque `i + 1`.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
        textures: &[TextureData],
    ) -> Result<Self> {
        let shader = device.create_shader_module(wgpu::include_wgsl!("voxel.wgsl"));
        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("voxel-globals"),
            size: size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let texture_view = upload_texture_array(device, queue, textures)?;
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("voxel-sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("voxel-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("voxel-bg"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("voxel-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("voxel-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                // Una instancia por cara; el vértice sale de `vertex_index`.
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: size_of::<PackedFace>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Uint32, 1 => Uint32],
                })],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DepthTarget::FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            pipeline,
            globals,
            bind_group,
            chunks: Vec::new(),
            emission: None,
            light: None,
            dual: false,
        })
    }

    /// Activa la luz por bloques. `emission[b]` es la luz que emite el
    /// bloque `b` (0-15). Sin llamar a esto, todo se dibuja con luz completa.
    pub fn enable_lighting(&mut self, emission: [u8; 256]) {
        self.emission = Some(emission);
        self.light = None;
        self.dual = false;
    }

    /// Activa la luz dual (cielo y bloque por separado), que el shader
    /// combina con el multiplicador de día de [`TerrainStyle`].
    pub fn enable_dual_lighting(&mut self, emission: [u8; 256]) {
        self.emission = Some(emission);
        self.light = None;
        self.dual = true;
    }

    /// Regenera las mallas de los chunks sucios y las sube a la GPU.
    pub fn update(&mut self, device: &wgpu::Device, world: &mut VoxelWorld) -> MeshStats {
        if self.chunks.len() != world.chunk_count() {
            self.chunks = (0..world.chunk_count()).map(|_| None).collect();
            world.mark_all_dirty();
        }
        let start = Instant::now();
        let mut light_changed = 0;
        if let Some(emission) = self.emission {
            match &mut self.light {
                Some(light) if world.has_dirty() => light_changed = light.update(world),
                Some(_) => {}
                None => {
                    self.light = Some(if self.dual {
                        Lighting::Dual(DualLightMap::new(world, emission))
                    } else {
                        Lighting::Combined(LightMap::new(world, emission))
                    });
                    world.mark_all_dirty();
                }
            }
        }
        let dirty = world.take_dirty();
        let mut stats = MeshStats {
            chunks_meshed: dirty.len(),
            light_changed,
            ..MeshStats::default()
        };
        for index in dirty {
            let faces = match self.light.as_ref() {
                Some(Lighting::Dual(map)) => mesh_chunk_dual(world, Some(map), index),
                Some(Lighting::Combined(map)) => mesh_chunk(world, Some(map), index),
                None => mesh_chunk(world, None, index),
            };
            stats.faces += faces.len();
            self.chunks[index] = (!faces.is_empty()).then(|| ChunkMesh {
                faces: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("chunk-faces"),
                    contents: bytemuck::cast_slice(&faces),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                count: faces.len() as u32,
            });
        }
        stats.millis = start.elapsed().as_secs_f64() * 1e3;
        stats
    }

    /// Caras visibles en total (para el HUD de depuración).
    pub fn face_count(&self) -> usize {
        self.chunks.iter().flatten().map(|c| c.count as usize).sum()
    }

    /// `(chunks con malla, chunks totales)` para la telemetría de F3.
    pub fn chunk_stats(&self) -> (usize, usize) {
        (self.chunks.iter().flatten().count(), self.chunks.len())
    }

    /// Graba el dibujo del terreno sobre `color`, borrándolo antes.
    pub fn render(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        depth: &DepthTarget,
        view: &View,
        style: &TerrainStyle,
    ) {
        let fog = [
            style.fog_color[0],
            style.fog_color[1],
            style.fog_color[2],
            style.fog_distance.max(1.0),
        ];
        let selected = style
            .selected
            .map_or([0, 0, 0, 0], |[x, y, z]| [x, y, z, 1]);
        let globals = Globals {
            view_proj: view.view_proj.to_cols_array_2d(),
            eye: view.eye.extend(style.lamp_radius).to_array(),
            fog,
            selected,
            time: style.time_seconds,
            daylight: style.daylight.clamp(0.0, 1.0),
            pad: [0.0; 2],
        };
        queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("voxel-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth.view,
                // Se guarda para que las entidades se dibujen después.
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        for chunk in self.chunks.iter().flatten() {
            pass.set_vertex_buffer(0, chunk.faces.slice(..));
            pass.draw(0..6, 0..chunk.count);
        }
    }
}
