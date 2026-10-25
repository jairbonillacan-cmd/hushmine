//! Pruebas de integración: dibuja un terreno sin ventana y comprueba la
//! imagen. Se salta sin GPU salvo con `HUSH_REQUIRE_GPU=1`.

use glam::DVec3;
use grisu_render::{
    FlyCamera, Gpu, MeshStats, OFFSCREEN_FORMAT, TerrainStyle, TextureData, VoxelRenderer, capture,
};
use grisu_voxel::{SparseWorld, VoxelWorld, VoxelsMut};

fn gpu() -> Option<Gpu> {
    match pollster::block_on(Gpu::headless()) {
        Ok(gpu) => Some(gpu),
        Err(err) if std::env::var("HUSH_REQUIRE_GPU").as_deref() != Ok("1") => {
            eprintln!("sin GPU, prueba omitida: {err:#}");
            None
        }
        Err(err) => panic!("se requiere GPU: {err:#}"),
    }
}

/// Textura lisa de un color, para reconocer cada bloque en la imagen.
fn solid(rgb: [u8; 3]) -> TextureData {
    TextureData {
        width: 16,
        height: 16,
        rgba: (0..256)
            .flat_map(|_| [rgb[0], rgb[1], rgb[2], 255])
            .collect(),
    }
}

/// Suelo de 8×8 con un pilar en el centro, cruzando límites de chunk.
fn build_hut(world: &mut SparseWorld, at: [i32; 3]) {
    for y in 0..2 {
        for z in 0..8 {
            for x in 0..8 {
                world.set(at[0] + x, at[1] + y, at[2] + z, 1);
            }
        }
    }
    for y in 2..6 {
        world.set(at[0] + 4, at[1] + y, at[2] + 4, 2);
    }
}

/// Malla y captura de `world` desde `camera` (posición de la cámara).
fn render_world(
    gpu: &Gpu,
    world: &mut SparseWorld,
    camera: DVec3,
) -> (MeshStats, image::RgbaImage) {
    let textures = [solid([40, 200, 40]), solid([220, 40, 40])];
    let mut terrain =
        VoxelRenderer::new(&gpu.device, &gpu.queue, OFFSCREEN_FORMAT, &textures).unwrap();
    let stats = terrain.update(&gpu.device, world, None);
    let camera = FlyCamera {
        position: camera,
        pitch: -12f32.to_radians(),
        ..FlyCamera::default()
    };
    let view = camera.view(1.0);
    let style = TerrainStyle {
        fog_distance: 200.0,
        selected: None,
        lamp_radius: 0.0,
        time_seconds: 0.0,
    };
    let image = capture(gpu, 96, 96, |encoder, color, depth| {
        terrain.render(&gpu.queue, encoder, color, depth, &view, &style);
    })
    .unwrap();
    (stats, image)
}

/// Algún píxel es verde (el suelo) y algún píxel es rojo (el pilar).
fn hut_is_visible(image: &image::RgbaImage) -> bool {
    image
        .pixels()
        .any(|p| p[1] > 100 && p[0] < 100 && p[2] < 100)
        && image
            .pixels()
            .any(|p| p[0] > 100 && p[1] < 100 && p[2] < 100)
}

#[test]
fn floor_and_pillar_render_with_their_textures() {
    let Some(gpu) = gpu() else { return };
    let mut world = VoxelWorld::new([32, 16, 32]).unwrap();
    world.fill([0, 0, 0], [31, 2, 31], 1);
    world.fill([16, 3, 10], [16, 8, 10], 2);

    let textures = [solid([40, 200, 40]), solid([220, 40, 40])];
    let mut terrain =
        VoxelRenderer::new(&gpu.device, &gpu.queue, OFFSCREEN_FORMAT, &textures).unwrap();
    let stats = terrain.update(&gpu.device, &mut world, None);
    assert_eq!(stats.chunks_meshed, world.chunk_count());
    assert!(terrain.face_count() > 0);
    // Una segunda actualización no regenera nada.
    assert_eq!(
        terrain.update(&gpu.device, &mut world, None).chunks_meshed,
        0
    );

    // Cámara sobre el suelo mirando al pilar (hacia -Z).
    let camera = FlyCamera {
        position: DVec3::new(16.5, 6.0, 20.0),
        pitch: -10f32.to_radians(),
        ..FlyCamera::default()
    };
    let view = camera.view(1.0);
    let style = TerrainStyle {
        fog_distance: 200.0,
        selected: None,
        lamp_radius: 0.0,
        time_seconds: 0.0,
    };
    let image = capture(&gpu, 96, 96, |encoder, color, depth| {
        terrain.render(&gpu.queue, encoder, color, depth, &view, &style);
    })
    .unwrap();

    // Centro: el pilar rojo.
    let center = image.get_pixel(48, 48);
    assert!(
        center[0] > 100 && center[1] < 60,
        "el centro debería ser el pilar: {center:?}"
    );
    // Abajo: el suelo verde.
    let bottom = image.get_pixel(48, 92);
    assert!(
        bottom[1] > 100 && bottom[0] < 80,
        "abajo debería verse el suelo: {bottom:?}"
    );
    // Arriba: el fondo oscuro.
    let top = image.get_pixel(5, 2);
    assert!(
        top.0[..3].iter().all(|&c| c < 70),
        "fondo inesperado: {top:?}"
    );
}

/// La misma estructura a millones de bloques del origen, con la cámara
/// desplazada igual, produce la misma imagen byte a byte: el render es
/// relativo a la cámara (ADR 0002), no al origen del mundo.
#[test]
fn far_from_origin_renders_byte_identical_to_origin() {
    let Some(gpu) = gpu() else { return };
    let offset = [7_499_800_i32, -4000, -7_499_800];
    let camera = DVec3::new(17.5, 8.0, 20.5);

    let mut near = SparseWorld::new();
    build_hut(&mut near, [13, 0, 5]);
    let (near_stats, near_image) = render_world(&gpu, &mut near, camera);
    // La estructura cruza dos chunks en x.
    assert!(near_stats.chunks_meshed >= 2, "{near_stats:?}");
    assert!(hut_is_visible(&near_image), "debería verse la choza");

    let mut far = SparseWorld::new();
    build_hut(&mut far, [13 + offset[0], offset[1], 5 + offset[2]]);
    let far_camera = camera + DVec3::new(offset[0] as f64, offset[1] as f64, offset[2] as f64);
    let (far_stats, far_image) = render_world(&gpu, &mut far, far_camera);
    assert!(far_stats.chunks_meshed >= 2, "{far_stats:?}");
    assert!(
        hut_is_visible(&far_image),
        "la choza debería verse igual a millones de bloques"
    );

    assert_eq!(
        near_image.as_raw(),
        far_image.as_raw(),
        "las imágenes cerca y lejos del origen deberían ser idénticas byte a byte"
    );
}

/// Chunks con coordenadas negativas se mallan y dibujan igual.
#[test]
fn chunks_in_negative_coordinates_render() {
    let Some(gpu) = gpu() else { return };
    let mut world = SparseWorld::new();
    // Bloques de x -19..-12 e y -18..-13: chunks -2 y -1 en varios ejes.
    build_hut(&mut world, [-19, -18, -3]);
    let (stats, image) = render_world(&gpu, &mut world, DVec3::new(-14.5, -12.0, 8.5));
    assert!(stats.chunks_meshed >= 2, "{stats:?}");
    assert!(
        hut_is_visible(&image),
        "debería verse la choza en chunks negativos"
    );
}

#[test]
fn rejects_mismatched_textures() {
    let Some(gpu) = gpu() else { return };
    let mut odd = solid([1, 2, 3]);
    odd.width = 8;
    odd.rgba.truncate(8 * 16 * 4);
    let result = VoxelRenderer::new(
        &gpu.device,
        &gpu.queue,
        OFFSCREEN_FORMAT,
        &[solid([0, 0, 0]), odd],
    );
    assert!(result.is_err());
}
