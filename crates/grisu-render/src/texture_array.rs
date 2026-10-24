use anyhow::{Result, ensure};

use crate::TextureData;

/// Sube varias texturas del mismo tamaño como un `texture_2d_array`.
/// La capa `i` es `layers[i]`.
pub fn upload_texture_array(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layers: &[TextureData],
) -> Result<wgpu::TextureView> {
    let first = layers.first();
    ensure!(first.is_some(), "hace falta al menos una textura");
    let (width, height) = first.map_or((1, 1), |t| (t.width, t.height));
    for (i, layer) in layers.iter().enumerate() {
        ensure!(
            layer.width == width && layer.height == height,
            "la textura {i} mide {}×{}, se esperaba {width}×{height}",
            layer.width,
            layer.height
        );
        ensure!(
            layer.rgba.len() == (width * height * 4) as usize,
            "la textura {i} tiene un tamaño de datos inválido"
        );
    }
    let count = layers.len() as u32;
    ensure!(
        count <= device.limits().max_texture_array_layers,
        "demasiadas capas de textura ({count})"
    );

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("block-texture-array"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: count,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for (i, layer) in layers.iter().enumerate() {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: i as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &layer.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
    Ok(texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("block-texture-array-view"),
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    }))
}
