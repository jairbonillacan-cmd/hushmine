use anyhow::{Context, Result, ensure};
use image::RgbaImage;

use crate::{CubeRenderer, DepthTarget, Gpu, OrbitCamera, TextureData};

/// Formato del objetivo de color de las capturas.
pub const OFFSCREEN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const FORMAT: wgpu::TextureFormat = OFFSCREEN_FORMAT;

/// Dibuja la escena de prueba (el cubo del hito 0) sin ventana.
pub fn render_to_image(
    gpu: &Gpu,
    width: u32,
    height: u32,
    camera: &OrbitCamera,
    texture: &TextureData,
) -> Result<RgbaImage> {
    let cube = CubeRenderer::new(&gpu.device, &gpu.queue, FORMAT, texture);
    capture(gpu, width, height, |encoder, view, depth| {
        cube.render(&gpu.queue, encoder, view, depth, camera);
    })
}

/// Dibuja sin ventana y devuelve la imagen.
///
/// `draw` graba el dibujo en un objetivo de formato [`OFFSCREEN_FORMAT`].
/// Es la base de `hush-cli shot`: permite a la IA ver el resultado del
/// renderizado sin abrir el juego.
pub fn capture(
    gpu: &Gpu,
    width: u32,
    height: u32,
    draw: impl FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView, &DepthTarget),
) -> Result<RgbaImage> {
    ensure!(width > 0 && height > 0, "tamaño de imagen inválido");
    let device = &gpu.device;

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("offscreen-color"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let depth = DepthTarget::new(device, width, height);

    // Las filas copiadas a un búfer deben alinearse a 256 bytes.
    let unpadded_row = 4 * width;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_row = unpadded_row.div_ceil(align) * align;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("offscreen-readback"),
        size: u64::from(padded_row) * u64::from(height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("offscreen-encoder"),
    });
    draw(&mut encoder, &view, &depth);
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &target,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);

    let slice = readback.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .context("error esperando a la GPU")?;
    rx.recv()
        .context("la GPU no respondió a la lectura")?
        .context("no se pudo leer la imagen de la GPU")?;

    let mut pixels = Vec::with_capacity((unpadded_row * height) as usize);
    {
        let mapped = slice
            .get_mapped_range()
            .context("no se pudo acceder a la memoria leída")?;
        for row in mapped.chunks_exact(padded_row as usize) {
            pixels.extend_from_slice(&row[..unpadded_row as usize]);
        }
    }
    readback.unmap();

    RgbaImage::from_raw(width, height, pixels).context("tamaño de imagen inconsistente")
}
