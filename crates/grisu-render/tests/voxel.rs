//! Prueba de integración: dibuja un terreno sin ventana y comprueba la
//! imagen. Se salta sin GPU salvo con `HUSH_REQUIRE_GPU=1`.

use glam::Vec3;
use grisu_render::{
    FlyCamera, Gpu, OFFSCREEN_FORMAT, TerrainStyle, TextureData, VoxelRenderer, capture,
};
use grisu_voxel::VoxelWorld;

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

#[test]
fn floor_and_pillar_render_with_their_textures() {
    let Some(gpu) = gpu() else { return };
    let mut world = VoxelWorld::new([32, 16, 32]).unwrap();
    world.fill([0, 0, 0], [31, 2, 31], 1);
    world.fill([16, 3, 10], [16, 8, 10], 2);

    let textures = [solid([40, 200, 40]), solid([220, 40, 40])];
    let mut terrain =
        VoxelRenderer::new(&gpu.device, &gpu.queue, OFFSCREEN_FORMAT, &textures).unwrap();
    let stats = terrain.update(&gpu.device, &mut world);
    assert_eq!(stats.chunks_meshed, world.chunk_count());
    assert!(terrain.face_count() > 0);
    // Una segunda actualización no regenera nada.
    assert_eq!(terrain.update(&gpu.device, &mut world).chunks_meshed, 0);

    // Cámara sobre el suelo mirando al pilar (hacia -Z).
    let camera = FlyCamera {
        position: Vec3::new(16.5, 6.0, 20.0),
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
