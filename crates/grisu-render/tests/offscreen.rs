//! Prueba de integración: dibuja el cubo sin ventana y comprueba la imagen.
//!
//! Si no hay GPU (algunos runners de CI), la prueba se salta, salvo que
//! `HUSH_REQUIRE_GPU=1` obligue a tenerla (Linux en CI usa lavapipe).

use grisu_render::{Gpu, OrbitCamera, TextureData, render_to_image};

#[test]
fn offscreen_cube_renders_over_clear_color() {
    let gpu = match pollster::block_on(Gpu::headless()) {
        Ok(gpu) => gpu,
        Err(err) if std::env::var("HUSH_REQUIRE_GPU").as_deref() != Ok("1") => {
            eprintln!("sin GPU, prueba omitida: {err:#}");
            return;
        }
        Err(err) => panic!("se requiere GPU: {err:#}"),
    };

    let image = render_to_image(
        &gpu,
        128,
        128,
        &OrbitCamera::default(),
        &TextureData::test_ore(),
    )
    .expect("la captura debería funcionar");

    // La esquina es fondo: oscuro (el color lineal de borrado se guarda en sRGB,
    // por eso ronda 40-50 y no 5).
    let corner = image.get_pixel(0, 0);
    assert!(
        corner[0] < 70 && corner[1] < 70 && corner[2] < 70,
        "fondo inesperado: {corner:?}"
    );

    // El centro es el cubo: claramente más brillante que el fondo.
    let center = image.get_pixel(64, 64);
    let brightness = u32::from(center[0]) + u32::from(center[1]) + u32::from(center[2]);
    assert!(
        brightness > 120,
        "el centro debería ser el cubo: {center:?}"
    );

    // Hay mineral naranja visible en alguna cara.
    let orange = image
        .pixels()
        .filter(|p| p[0] > 150 && p[0] > p[2] + 80)
        .count();
    assert!(
        orange > 20,
        "no se ve mineral en el cubo ({orange} píxeles)"
    );
}
