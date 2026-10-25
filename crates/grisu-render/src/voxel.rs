use std::collections::BTreeMap;
use std::time::Instant;

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use grisu_voxel::{ChunkPos, ChunkedVoxels, LightMap, PackedFace, mesh_chunk};
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
}

/// Ajustes de dibujo del terreno que cambian cada fotograma.
#[derive(Clone, Copy, Debug)]
pub struct TerrainStyle {
    /// Distancia, en bloques, a la que la niebla tapa todo.
    pub fog_distance: f32,
    /// Bloque señalado por el jugador, que se resalta.
    pub selected: Option<[i32; 3]>,
    /// Alcance de la lámpara del casco, en bloques (0 = apagada).
    pub lamp_radius: f32,
}

impl Default for TerrainStyle {
    fn default() -> Self {
        Self {
            fog_distance: 48.0,
            selected: None,
            lamp_radius: 14.0,
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

/// Origen de un chunk, relativo al anchor de la vista, listo para el shader.
type ChunkOrigin = [f32; 4];

/// Dibuja un mundo de chunks ([`grisu_voxel::ChunkedVoxels`]): una malla por
/// chunk, regenerada solo cuando el chunk está sucio.
///
/// Todo el dibujo es **relativo a la cámara** (ADR 0002): la vista trae el
/// bloque anchor y, en el shader, cada cara local al chunk se desplaza con el
/// origen de su chunk relativo a ese anchor. Así a millones de bloques del
/// origen el shader solo ve números pequeños.
pub struct VoxelRenderer {
    pipeline: wgpu::RenderPipeline,
    globals: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    chunks: BTreeMap<ChunkPos, Option<ChunkMesh>>,
    origins_layout: wgpu::BindGroupLayout,
    origins_bg: wgpu::BindGroup,
    origins: wgpu::Buffer,
    origins_capacity: usize,
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
        // Origen del chunk de la cara que se dibuja, relativo al anchor de
        // la cámara, en su propio grupo para poder crecer sin recrear el de
        // texturas. Un vec4 por chunk, con offset dinámico (wgpu alinea los
        // dinámicos a 256 bytes).
        let origins_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("voxel-origins-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(wgpu::BufferSize::new(16).unwrap()),
                },
                count: None,
            }],
        });
        let origins_capacity = Self::INITIAL_ORIGIN_CAPACITY;
        let origins = Self::origin_buffer(device, origins_capacity);
        let origins_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("voxel-origins-bg"),
            layout: &origins_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: Self::origins_binding(&origins),
            }],
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
            bind_group_layouts: &[Some(&bind_group_layout), Some(&origins_layout)],
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
            chunks: BTreeMap::new(),
            origins_layout,
            origins_bg,
            origins,
            origins_capacity,
        })
    }

    /// Orígenes de chunk que caben en el búfer al crearlo.
    const INITIAL_ORIGIN_CAPACITY: usize = 4096;
    /// Bytes por entrada del búfer de orígenes: un vec4, alineado al mínimo
    /// de wgpu para offsets dinámicos de uniformes (256).
    const ORIGIN_SLOT_BYTES: u32 = 256;

    /// Ventana de 16 bytes (un vec4) del búfer de orígenes, para el bind
    /// group con offset dinámico.
    fn origins_binding(buffer: &wgpu::Buffer) -> wgpu::BindingResource<'_> {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer,
            offset: 0,
            size: wgpu::BufferSize::new(size_of::<ChunkOrigin>() as u64),
        })
    }

    fn origin_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("voxel-chunk-origins"),
            size: u64::from(Self::ORIGIN_SLOT_BYTES) * capacity as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Regenera las mallas de los chunks sucios y las suba a la GPU. La luz
    /// se calcula fuera (con [`LightMap`], en el juego) y aquí solo se usa
    /// para teñir las caras.
    pub fn update<W: ChunkedVoxels + ?Sized>(
        &mut self,
        device: &wgpu::Device,
        world: &mut W,
        light: Option<&LightMap>,
    ) -> MeshStats {
        let start = Instant::now();
        let dirty = world.take_dirty_chunks();
        let mut stats = MeshStats {
            chunks_meshed: dirty.len(),
            ..MeshStats::default()
        };
        for pos in dirty {
            if !world.is_loaded(pos) {
                // El chunk se descargó: su malla desaparece.
                self.chunks.remove(&pos);
                continue;
            }
            let faces = mesh_chunk(world, light, pos);
            stats.faces += faces.len();
            let mesh = (!faces.is_empty()).then(|| ChunkMesh {
                faces: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("chunk-faces"),
                    contents: bytemuck::cast_slice(&faces),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
                count: faces.len() as u32,
            });
            self.chunks.insert(pos, mesh);
        }
        stats.millis = start.elapsed().as_secs_f64() * 1e3;
        stats
    }

    /// Caras visibles en total (para el HUD de depuración).
    pub fn face_count(&self) -> usize {
        self.chunks
            .values()
            .flatten()
            .map(|c| c.count as usize)
            .sum()
    }

    /// Graba el dibujo del terreno sobre `color`, borrándolo antes.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        depth: &DepthTarget,
        view: &View,
        style: &TerrainStyle,
    ) {
        // Origen de cada chunk con malla, relativo al anchor de la vista, en
        // el mismo orden en que se dibujan.
        let origins: Vec<ChunkOrigin> = self
            .chunks
            .iter()
            .filter_map(|(pos, mesh)| {
                mesh.as_ref()?;
                let rel = view.relative_block(pos.origin());
                Some([rel[0] as f32, rel[1] as f32, rel[2] as f32, 0.0])
            })
            .collect();
        if origins.len() > self.origins_capacity {
            self.origins_capacity = origins.len().next_power_of_two();
            self.origins = Self::origin_buffer(device, self.origins_capacity);
            self.origins_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("voxel-origins-bg"),
                layout: &self.origins_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: Self::origins_binding(&self.origins),
                }],
            });
        }
        // Cada origen vive en su slot de 256 bytes, el hueco que saltará el
        // offset dinámico.
        for (i, origin) in origins.iter().enumerate() {
            queue.write_buffer(
                &self.origins,
                u64::from(Self::ORIGIN_SLOT_BYTES) * i as u64,
                bytemuck::bytes_of(origin),
            );
        }

        let fog = [
            CLEAR_COLOR.r as f32,
            CLEAR_COLOR.g as f32,
            CLEAR_COLOR.b as f32,
            style.fog_distance.max(1.0),
        ];
        // El bloque señalado, en coordenadas relativas al anchor (las mismas
        // en las que el shader sitúa las caras).
        let selected = style
            .selected
            .map(|b| view.relative_block(b))
            .map_or([0, 0, 0, 0], |r| [r[0], r[1], r[2], 1]);
        let globals = Globals {
            view_proj: view.view_proj.to_cols_array_2d(),
            eye: view.eye.extend(style.lamp_radius).to_array(),
            fog,
            selected,
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
        for (i, mesh) in self.chunks.values().flatten().enumerate() {
            pass.set_vertex_buffer(0, mesh.faces.slice(..));
            pass.set_bind_group(1, &self.origins_bg, &[(i as u32) * Self::ORIGIN_SLOT_BYTES]);
            pass.draw(0..6, 0..mesh.count);
        }
    }
}
