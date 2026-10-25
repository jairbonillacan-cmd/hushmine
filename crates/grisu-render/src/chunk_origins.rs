//! Grupo 1 del renderizador de vóxeles: el origen del chunk que se dibuja,
//! relativo al ancla de la cámara.
//!
//! Una ranura por chunk en un único uniform con desplazamiento dinámico
//! (funciona también en WebGL2): cada draw elige la suya. Los orígenes
//! viajan en enteros de 32 bits y ya relativos al ancla: a 7,5 millones de
//! bloques un `f32` no distingue ni medio bloque, y al shader solo llegan
//! números pequeños.

use bytemuck::{Pod, Zeroable};

/// Origen de un chunk relativo al ancla de la cámara.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Slot {
    /// xyz: primer bloque del chunk menos el ancla. w: sin uso.
    block: [i32; 4],
}

/// Búfer de orígenes por chunk con su bind group (grupo 1).
pub(super) struct ChunkOrigins {
    layout: wgpu::BindGroupLayout,
    buffer: wgpu::Buffer,
    group: wgpu::BindGroup,
    /// Bytes entre ranuras: la alineación que exige el dispositivo.
    stride: u32,
    capacity: usize,
}

impl ChunkOrigins {
    const INITIAL_CAPACITY: usize = 64;

    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("voxel-chunk-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let stride = device
            .limits()
            .min_uniform_buffer_offset_alignment
            .max(size_of::<Slot>() as u32);
        let capacity = Self::INITIAL_CAPACITY;
        let buffer = Self::buffer(device, capacity, stride);
        let group = Self::make_group(device, &layout, &buffer, stride);
        Self {
            layout,
            buffer,
            group,
            stride,
            capacity,
        }
    }

    fn buffer(device: &wgpu::Device, capacity: usize, stride: u32) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("voxel-chunk-origins"),
            size: capacity as u64 * u64::from(stride),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn make_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        buffer: &wgpu::Buffer,
        stride: u32,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("voxel-chunk-bg"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer,
                    offset: 0,
                    // Solo una ranura: el desplazamiento dinámico elige cuál.
                    size: wgpu::BufferSize::new(u64::from(stride)),
                }),
            }],
        })
    }

    /// Diseño del grupo 1, para el diseño del pipeline.
    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    /// Bind group que se fija en cada draw.
    pub fn group(&self) -> &wgpu::BindGroup {
        &self.group
    }

    /// Bytes entre ranuras.
    pub fn stride(&self) -> u32 {
        self.stride
    }

    /// Garantiza sitio para `chunks` ranuras. Recrea el búfer si hace falta;
    /// llámalo tras cambiar el número de chunks, nunca en el camino del
    /// dibujo.
    pub fn ensure_capacity(&mut self, device: &wgpu::Device, chunks: usize) {
        if chunks <= self.capacity {
            return;
        }
        self.capacity = chunks.max(self.capacity * 2);
        self.buffer = Self::buffer(device, self.capacity, self.stride);
        self.group = Self::make_group(device, &self.layout, &self.buffer, self.stride);
    }

    /// Sube un origen relativo por chunk, en el orden de dibujo.
    pub fn write(&self, queue: &wgpu::Queue, origins: &[[i32; 3]]) {
        if origins.is_empty() {
            return;
        }
        debug_assert!(origins.len() <= self.capacity);
        let stride = self.stride as usize;
        let mut data = vec![0u8; origins.len() * stride];
        for (i, origin) in origins.iter().enumerate() {
            let slot = Slot {
                block: [origin[0], origin[1], origin[2], 0],
            };
            data[i * stride..i * stride + size_of::<Slot>()]
                .copy_from_slice(bytemuck::bytes_of(&slot));
        }
        queue.write_buffer(&self.buffer, 0, &data);
    }
}
