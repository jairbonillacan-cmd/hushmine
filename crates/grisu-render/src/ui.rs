//! Renderizador 2D de rectángulos y texto matricial para HUD, menús y launcher.

use bytemuck::{Pod, Zeroable};

use crate::ui_font::{ABOVE, CAP_HEIGHT, glyph};

/// Columnas vacías entre una letra y la siguiente.
const LETTER_SPACING: u32 = 1;

/// Rectángulo 2D en píxeles de pantalla (origen arriba a la izquierda).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct UiRect {
    pub rect: [f32; 4],
    pub color: [f32; 4],
}

impl UiRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) -> Self {
        Self {
            rect: [x, y, w, h],
            color,
        }
    }

    /// ¿Cae el punto `(px, py)` dentro de este rectángulo?
    pub fn contains(&self, px: f32, py: f32) -> bool {
        let [x, y, w, h] = self.rect;
        px >= x && px <= x + w && py >= y && py <= y + h
    }
}

/// Lote de primitivas 2D de un fotograma.
#[derive(Clone, Debug, Default)]
pub struct UiBatch {
    rects: Vec<UiRect>,
}

impl UiBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.rects.clear();
    }

    pub fn rects(&self) -> &[UiRect] {
        &self.rects
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        if w > 0.0 && h > 0.0 {
            self.rects.push(UiRect::new(x, y, w, h, color));
        }
    }

    /// Dibuja un panel con relleno y borde interior de `border_px` píxeles.
    #[allow(clippy::too_many_arguments)]
    pub fn panel(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        bg: [f32; 4],
        border: [f32; 4],
        border_px: f32,
    ) {
        self.rect(x, y, w, h, bg);
        let b = border_px.max(1.0);
        self.rect(x, y, w, b, border);
        self.rect(x, y + h - b, w, b, border);
        self.rect(x, y + b, b, (h - 2.0 * b).max(0.0), border);
        self.rect(x + w - b, y + b, b, (h - 2.0 * b).max(0.0), border);
    }

    /// Anchura en píxeles de una cadena con tamaño de píxel `scale`.
    pub fn text_width(scale: f32, text: &str) -> f32 {
        let columns: u32 = text
            .chars()
            .map(|ch| u32::from(glyph(ch).width) + LETTER_SPACING)
            .sum();
        columns.saturating_sub(LETTER_SPACING) as f32 * scale.max(1.0)
    }

    /// Altura de mayúscula en píxeles con tamaño de píxel `scale`. Las
    /// tildes de las mayúsculas suben 2 píxeles de fuente por encima y las
    /// descendentes bajan 2 por debajo.
    pub fn text_height(scale: f32) -> f32 {
        CAP_HEIGHT as f32 * scale
    }

    /// Dibuja texto con la fuente de mapa de bits. `y` es la parte de
    /// arriba de las mayúsculas. Une tramos horizontales contiguos para
    /// enviar menos rectángulos a la GPU.
    pub fn text(&mut self, x: f32, y: f32, scale: f32, color: [f32; 4], text: &str) {
        let s = scale.max(1.0);
        let mut pen = x;
        for ch in text.chars() {
            let g = glyph(ch);
            for (row_idx, &bits) in g.rows.iter().enumerate() {
                if bits == 0 {
                    continue;
                }
                let gy = y + (row_idx as f32 - ABOVE as f32) * s;
                let mut col = 0usize;
                while col < 8 {
                    if g.pixel(row_idx, col) {
                        let start = col;
                        while col < 8 && g.pixel(row_idx, col) {
                            col += 1;
                        }
                        let span = (col - start) as f32 * s;
                        self.rect(pen + start as f32 * s, gy, span, s, color);
                    } else {
                        col += 1;
                    }
                }
            }
            pen += f32::from(g.width + LETTER_SPACING as u8) * s;
        }
    }

    /// Dibuja texto centrado horizontalmente en `center_x`.
    pub fn text_centered(
        &mut self,
        center_x: f32,
        y: f32,
        scale: f32,
        color: [f32; 4],
        text: &str,
    ) {
        let w = Self::text_width(scale, text);
        self.text((center_x - w * 0.5).round(), y.round(), scale, color, text);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct ScreenUniform {
    size: [f32; 4],
}

const SHADER: &str = r"
struct Screen { size: vec4<f32> };
@group(0) @binding(0) var<uniform> screen: Screen;

struct Out {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) i: u32,
    @location(0) rect: vec4<f32>,
    @location(1) color: vec4<f32>,
) -> Out {
    var corners = array<vec2<f32>, 6>(
        vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
        vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0),
    );
    let px = rect.xy + corners[i] * rect.zw;
    let ndc = vec2(
        (px.x / screen.size.x) * 2.0 - 1.0,
        1.0 - (px.y / screen.size.y) * 2.0,
    );
    var out: Out;
    out.clip = vec4(ndc, 0.0, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: Out) -> @location(0) vec4<f32> {
    return in.color;
}
";

/// Renderizador de lotes `UiBatch` sobre la textura de color final.
pub struct UiRenderer {
    pipeline: wgpu::RenderPipeline,
    screen: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    instances: wgpu::Buffer,
    capacity: usize,
}

impl UiRenderer {
    const INITIAL_CAPACITY: usize = 2048;

    pub fn new(device: &wgpu::Device, color_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui-shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let screen = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui-screen"),
            size: size_of::<ScreenUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ui-bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen.as_entire_binding(),
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ui-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: size_of::<UiRect>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4],
                })],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            screen,
            bind_group,
            instances,
            capacity,
        }
    }

    fn instance_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui-instances"),
            size: (capacity * size_of::<UiRect>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        size: [u32; 2],
        batch: &UiBatch,
    ) {
        let [width, height] = size;
        let rects = batch.rects();
        if rects.is_empty() {
            return;
        }
        if rects.len() > self.capacity {
            self.capacity = rects.len().next_power_of_two();
            self.instances = Self::instance_buffer(device, self.capacity);
        }
        let uniform = ScreenUniform {
            size: [width.max(1) as f32, height.max(1) as f32, 0.0, 0.0],
        };
        queue.write_buffer(&self.screen, 0, bytemuck::bytes_of(&uniform));
        queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(rects));

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ui-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..6, 0..rects.len() as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_emits_rects_and_measures_width() {
        let mut batch = UiBatch::new();
        assert_eq!(UiBatch::text_width(2.0, "HUSH"), 46.0);
        batch.text(10.0, 20.0, 2.0, [1.0; 4], "HUSHMINE 123");
        assert!(!batch.rects().is_empty());
        let r = UiRect::new(10.0, 20.0, 100.0, 40.0, [1.0; 4]);
        assert!(r.contains(50.0, 30.0));
        assert!(!r.contains(5.0, 30.0));
    }
}
