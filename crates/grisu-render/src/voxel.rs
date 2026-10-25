use std::collections::BTreeMap;
use std::time::Instant;

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use grisu_voxel::{ChunkPos, ChunkedVoxels, LightMap, PackedFace, mesh_chunk};
use wgpu::util::DeviceExt;

use crate::chunk_origins::ChunkOrigins;
use crate::debug_view::DebugView;
use crate::shader_prep::{ShaderConfig, preprocess_wgsl};
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
    /// Vista de depuración del terreno ([`DebugView`]); 0 = dibujo normal.
    debug_view: u32,
    /// Relleno hasta el tamaño del uniforme (múltiplo de 16: 128 bytes).
    pad: [u32; 2],
}

/// Ajustes de dibujo del terreno que cambian cada fotograma.
#[derive(Clone, Copy, Debug)]
pub struct TerrainStyle {
    /// Distancia, en bloques, a la que la niebla tapa todo.
    pub fog_distance: f32,
    /// Bloque señalado por el jugador (coordenadas del mundo), que se resalta.
    pub selected: Option<[i32; 3]>,
    /// Alcance de la lámpara del casco, en bloques (0 = apagada).
    pub lamp_radius: f32,
    /// Segundos desde el arranque; anima los bloques con fotogramas (lava).
    pub time_seconds: f32,
}

impl Default for TerrainStyle {
    fn default() -> Self {
        Self {
            fog_distance: 48.0,
            selected: None,
            lamp_radius: 14.0,
            time_seconds: 0.0,
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
    pub millis: f64,
}

/// Dibuja cualquier mundo por chunks ([`ChunkedVoxels`]): una malla por
/// chunk, regenerada solo cuando el chunk está sucio.
///
/// Cada draw usa el grupo 1 para elegir, con un desplazamiento dinámico, el
/// origen de ese chunk relativo a la cámara. Así el mismo código dibuja el
/// mapa finito del modo Turno y el mundo grande sin perder precisión.
pub struct VoxelRenderer {
    pipeline: wgpu::RenderPipeline,
    globals: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    /// Orígenes por chunk relativos a la cámara (grupo 1).
    origins: ChunkOrigins,
    /// Mallas por chunk; el orden del mapa fija el orden de dibujo.
    chunks: BTreeMap<ChunkPos, ChunkMesh>,
    /// Vista de depuración activa; el shader ramifica sobre ese uniforme.
    debug_view: DebugView,
}

impl VoxelRenderer {
    /// `textures[i]` es la textura del bloque `i + 1`.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color_format: wgpu::TextureFormat,
        textures: &[TextureData],
    ) -> Result<Self> {
        // Preprocesamos el shader con la configuración por defecto (calidad
        // alta, nativo); voxel.wgsl aún no usa directivas condicionales.
        let defines = ShaderConfig::default().defines();
        let source = preprocess_wgsl(include_str!("voxel.wgsl"), &[], &defines)?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("voxel-shader"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
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
        let origins = ChunkOrigins::new(device);

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("voxel-layout"),
            bind_group_layouts: &[Some(&bind_group_layout), Some(origins.layout())],
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
            origins,
            chunks: BTreeMap::new(),
            debug_view: DebugView::None,
        })
    }

    /// Elige la vista de depuración del terreno; `None` devuelve el dibujo
    /// normal. Surte efecto en el `render` siguiente.
    pub fn set_debug_view(&mut self, view: DebugView) {
        self.debug_view = view;
    }

    /// Vista de depuración activa.
    pub fn debug_view(&self) -> DebugView {
        self.debug_view
    }

    /// Regenera las mallas de los chunks sucios y las sube a la GPU. `light`
    /// es el mapa de luz de las caras; calcularlo es cosa de quien llama.
    pub fn update<W: ChunkedVoxels + ?Sized>(
        &mut self,
        device: &wgpu::Device,
        world: &mut W,
        light: Option<&LightMap>,
    ) -> MeshStats {
        let start = Instant::now();
        // Los chunks descargados sueltan su malla, aunque no estén sucios.
        self.chunks.retain(|&pos, _| world.is_loaded(pos));
        let dirty = world.take_dirty_chunks();
        let mut stats = MeshStats::default();
        for pos in dirty {
            if !world.is_loaded(pos) {
                self.chunks.remove(&pos);
                continue;
            }
            stats.chunks_meshed += 1;
            let faces = mesh_chunk(world, light, pos);
            stats.faces += faces.len();
            if faces.is_empty() {
                self.chunks.remove(&pos);
            } else {
                self.chunks.insert(
                    pos,
                    ChunkMesh {
                        faces: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("chunk-faces"),
                            contents: bytemuck::cast_slice(&faces),
                            usage: wgpu::BufferUsages::VERTEX,
                        }),
                        count: faces.len() as u32,
                    },
                );
            }
        }
        self.origins.ensure_capacity(device, self.chunks.len());
        stats.millis = start.elapsed().as_secs_f64() * 1e3;
        stats
    }

    /// Caras visibles en total (para el HUD de depuración).
    pub fn face_count(&self) -> usize {
        self.chunks.values().map(|c| c.count as usize).sum()
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
            CLEAR_COLOR.r as f32,
            CLEAR_COLOR.g as f32,
            CLEAR_COLOR.b as f32,
            style.fog_distance.max(1.0),
        ];
        // El bloque señalado también viaja relativo al ancla.
        let selected = style.selected.map_or([0, 0, 0, 0], |block| {
            let relative = view.relative_block(block);
            [relative[0], relative[1], relative[2], 1]
        });
        let globals = Globals {
            view_proj: view.view_proj.to_cols_array_2d(),
            eye: view.eye.extend(style.lamp_radius).to_array(),
            fog,
            selected,
            time: style.time_seconds,
            debug_view: self.debug_view as u32,
            pad: [0, 0],
        };
        queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));

        // Origen de cada chunk relativo al ancla de la cámara, en enteros.
        // La ranura i corresponde al chunk i-ésimo del mapa (orden estable).
        let origins: Vec<[i32; 3]> = self
            .chunks
            .keys()
            .map(|&pos| view.relative_block(pos.origin()))
            .collect();
        self.origins.write(queue, &origins);

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
        for (i, chunk) in self.chunks.values().enumerate() {
            let slot = (i as u32) * self.origins.stride();
            pass.set_bind_group(1, self.origins.group(), &[slot]);
            pass.set_vertex_buffer(0, chunk.faces.slice(..));
            pass.draw(0..6, 0..chunk.count);
        }
    }
}
