use bytemuck::{Pod, Zeroable};
use glam::{DVec3, Vec3};

use crate::{CLEAR_COLOR, DepthTarget, View};

/// Caja de color alineada con los ejes (jugadores, marcadores...).
///
/// `min` es un punto del mundo en `f64`: a millones de bloques del origen
/// sigue siendo exacto, y al dibujar se hace relativo a la cámara.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorBox {
    pub min: DVec3,
    pub size: Vec3,
    /// RGB lineal.
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Instance {
    min: [f32; 3],
    size: [f32; 3],
    color: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
    eye: [f32; 4],
    fog: [f32; 4],
}

const SHADER: &str = r"
struct Globals { view_proj: mat4x4<f32>, eye: vec4<f32>, fog: vec4<f32> };
@group(0) @binding(0) var<uniform> globals: Globals;

struct Out {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) world: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) i: u32,
    @location(0) min: vec3<f32>,
    @location(1) size: vec3<f32>,
    @location(2) color: vec3<f32>,
) -> Out {
    // Seis caras de dos triángulos; esquinas en el cubo unitario.
    var corners = array<vec3<f32>, 36>(
        vec3(1.,0.,1.), vec3(1.,0.,0.), vec3(1.,1.,0.), vec3(1.,0.,1.), vec3(1.,1.,0.), vec3(1.,1.,1.),
        vec3(0.,0.,0.), vec3(0.,0.,1.), vec3(0.,1.,1.), vec3(0.,0.,0.), vec3(0.,1.,1.), vec3(0.,1.,0.),
        vec3(0.,1.,1.), vec3(1.,1.,1.), vec3(1.,1.,0.), vec3(0.,1.,1.), vec3(1.,1.,0.), vec3(0.,1.,0.),
        vec3(0.,0.,0.), vec3(1.,0.,0.), vec3(1.,0.,1.), vec3(0.,0.,0.), vec3(1.,0.,1.), vec3(0.,0.,1.),
        vec3(0.,0.,1.), vec3(1.,0.,1.), vec3(1.,1.,1.), vec3(0.,0.,1.), vec3(1.,1.,1.), vec3(0.,1.,1.),
        vec3(1.,0.,0.), vec3(0.,0.,0.), vec3(0.,1.,0.), vec3(1.,0.,0.), vec3(0.,1.,0.), vec3(1.,1.,0.),
    );
    var shades = array<f32, 6>(0.80, 0.80, 1.00, 0.50, 0.65, 0.65);
    let world = min + corners[i] * size;
    var out: Out;
    out.clip = globals.view_proj * vec4(world, 1.0);
    out.color = color * shades[i / 6u];
    out.world = world;
    return out;
}

@fragment
fn fs_main(in: Out) -> @location(0) vec4<f32> {
    let dist = distance(in.world, globals.eye.xyz);
    let fog = smoothstep(globals.fog.a * 0.25, globals.fog.a, dist);
    return vec4(mix(in.color, globals.fog.rgb, fog), 1.0);
}
";

/// Dibuja cajas de color encima del terreno, respetando la profundidad.
pub struct BoxRenderer {
    pipeline: wgpu::RenderPipeline,
    globals: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    instances: wgpu::Buffer,
    capacity: usize,
}

impl BoxRenderer {
    const INITIAL_CAPACITY: usize = 64;

    pub fn new(device: &wgpu::Device, color_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("boxes-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let globals = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("boxes-globals"),
            size: size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("boxes-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("boxes-bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals.as_entire_binding(),
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("boxes-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("boxes-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: size_of::<Instance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x3, 1 => Float32x3, 2 => Float32x3
                    ],
                })],
            },
            primitive: wgpu::PrimitiveState {
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
        let capacity = Self::INITIAL_CAPACITY;
        let instances = Self::instance_buffer(device, capacity);
        Self {
            pipeline,
            globals,
            bind_group,
            instances,
            capacity,
        }
    }

    fn instance_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("boxes-instances"),
            size: (capacity * size_of::<Instance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Dibuja `boxes` sobre `color` usando la profundidad que dejó el
    /// terreno (debe haberse guardado, no descartado).
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        depth: &DepthTarget,
        view: &View,
        fog_distance: f32,
        boxes: &[ColorBox],
    ) {
        if boxes.is_empty() {
            return;
        }
        if boxes.len() > self.capacity {
            self.capacity = boxes.len().next_power_of_two();
            self.instances = Self::instance_buffer(device, self.capacity);
        }
        let instances: Vec<Instance> = boxes
            .iter()
            .map(|b| Instance {
                // Relativo al ancla de la cámara: coordenadas pequeñas.
                min: view.relative(b.min).to_array(),
                size: b.size.to_array(),
                color: b.color,
            })
            .collect();
        queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(&instances));
        let globals = Globals {
            view_proj: view.view_proj.to_cols_array_2d(),
            eye: view.eye.extend(1.0).to_array(),
            fog: [
                CLEAR_COLOR.r as f32,
                CLEAR_COLOR.g as f32,
                CLEAR_COLOR.b as f32,
                fog_distance.max(1.0),
            ],
        };
        queue.write_buffer(&self.globals, 0, bytemuck::bytes_of(&globals));

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("boxes-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
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
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..36, 0..boxes.len() as u32);
    }
}
