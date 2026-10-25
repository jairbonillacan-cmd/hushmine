//! Pruebas de integración: dibuja un terreno sin ventana y comprueba la
//! imagen. Se salta sin GPU salvo con `HUSH_REQUIRE_GPU=1`.

use glam::DVec3;
use grisu_render::{
    DebugView, FlyCamera, Gpu, MeshStats, OFFSCREEN_FORMAT, ShaderPrepError, TerrainStyle,
    TextureData, VoxelRenderer, capture, preprocess_wgsl, validate_all_shader_variants,
};
use grisu_voxel::{AIR, LightMap, SparseWorld, VoxelWorld, VoxelsMut, emission_of};

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

/// Mina para las vistas de depuración: suelo grande, macizo rocoso con dos
/// galerías y una lámpara (bloque 2, emisor de luz 15) al fondo de la
/// izquierda. El suelo y el macizo cruzan los límites de chunk x=16 y z=16,
/// así que la vista Chunks muestra al menos dos tintes del damero. La
/// galería derecha no tiene emisor y su fondo queda a oscuras: es el
/// extremo oscuro del mapa de luz.
fn build_mine(world: &mut VoxelWorld) {
    world.fill([0, 0, 0], [31, 2, 31], 1);
    world.fill([0, 3, 0], [31, 8, 19], 1);
    world.fill([10, 3, 4], [15, 5, 19], AIR);
    world.fill([17, 3, 2], [24, 5, 19], AIR);
    world.set(12, 4, 3, 2);
    world.set(13, 4, 3, 2);
}

/// Luminancia percibida aproximada de un píxel RGB, de 0 a 255.
fn luminance(p: &image::Rgba<u8>) -> u32 {
    (u32::from(p[0]) * 2126 + u32::from(p[1]) * 7152 + u32::from(p[2]) * 722) / 10_000
}

/// Píxeles que difieren entre dos imágenes del mismo tamaño.
fn diff_pixels(a: &image::RgbaImage, b: &image::RgbaImage) -> usize {
    a.pixels().zip(b.pixels()).filter(|(p, q)| p != q).count()
}

/// ¿Es el píxel (casi) el color `rgb`? La tolerancia cubre el redondeo de
/// f32 a unorm del shader.
fn tint_close(p: &image::Rgba<u8>, rgb: [u8; 3]) -> bool {
    (0..3).all(|c| i32::from(p[c]).abs_diff(i32::from(rgb[c])) <= 4)
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

/// Las 8 variantes de los shaders (4 presets × nativo o navegador)
/// preprocesan y validan sin GPU, y el preprocesador rechaza fuentes rotas.
#[test]
fn all_shader_variants_preprocess_cleanly() {
    assert_eq!(
        validate_all_shader_variants(),
        Ok(8),
        "las 8 variantes deberían preprocesar y validar sin errores"
    );

    // `#ifdef` sin `#endif`: el error señala la línea donde se abrió.
    let err = preprocess_wgsl("// cero\n#ifdef BROKEN\nalto", &[], &[]).unwrap_err();
    assert!(
        matches!(
            err,
            ShaderPrepError::UnterminatedConditional { line: 2, .. }
        ),
        "esperaba un condicional sin cerrar: {err:?}"
    );

    // Un include que se incluye a sí mismo es un ciclo.
    let cyclic = [("a.wgsl", "#include \"a.wgsl\"")];
    let err = preprocess_wgsl("#include \"a.wgsl\"", &cyclic, &[]).unwrap_err();
    assert!(
        matches!(err, ShaderPrepError::IncludeCycle { ref name, .. } if name == "a.wgsl"),
        "esperaba un ciclo de includes: {err:?}"
    );
}

/// La misma mina renderizada con cada vista de depuración produce capturas
/// distintas dos a dos y ninguna en blanco. La vista Light muestra el
/// gradiente real del `LightMap` (patio iluminado contra el fondo oscuro de
/// la galería sin emisor) y la vista Chunks al menos dos tintes del damero.
#[test]
fn debug_views_render_distinct_images() {
    let Some(gpu) = gpu() else { return };
    let mut world = VoxelWorld::new([32, 16, 32]).unwrap();
    build_mine(&mut world);
    // La lámpara (bloque 2) emite luz 15; el resto solo recibe luz de cielo.
    let light = LightMap::new(&world, emission_of(2, 15));

    let textures = [solid([40, 200, 40]), solid([220, 40, 40])];
    let mut terrain =
        VoxelRenderer::new(&gpu.device, &gpu.queue, OFFSCREEN_FORMAT, &textures).unwrap();
    assert_eq!(terrain.debug_view(), DebugView::None);
    assert_eq!(
        terrain
            .update(&gpu.device, &mut world, Some(&light))
            .chunks_meshed,
        world.chunk_count()
    );

    // Cámara en el patio, mirando a la boca de las dos galerías (hacia -Z).
    let camera = FlyCamera {
        position: DVec3::new(16.5, 6.5, 30.0),
        pitch: -14f32.to_radians(),
        ..FlyCamera::default()
    };
    let view = camera.view(1.0);
    let style = TerrainStyle {
        fog_distance: 200.0,
        selected: None,
        lamp_radius: 0.0,
        time_seconds: 0.0,
    };

    let views = [
        DebugView::None,
        DebugView::Light,
        DebugView::Ao,
        DebugView::Normals,
        DebugView::Chunks,
        DebugView::Sky,
    ];
    let mut shots = Vec::with_capacity(views.len());
    for view_mode in views {
        terrain.set_debug_view(view_mode);
        // El getter refleja el último set.
        assert_eq!(terrain.debug_view(), view_mode);
        let image = capture(&gpu, 96, 96, |encoder, color, depth| {
            terrain.render(&gpu.queue, encoder, color, depth, &view, &style);
        })
        .unwrap();
        shots.push((view_mode, image));
    }
    assert_eq!(
        terrain.debug_view(),
        DebugView::Sky,
        "el getter debería conservar la última vista activa"
    );

    // Ninguna captura es todo fondo: el color de fondo se muestrea del cielo
    // de la esquina superior izquierda.
    let background = *shots[0].1.get_pixel(5, 2);
    for (view_mode, image) in &shots {
        let visible = image.pixels().filter(|&&p| p != background).count();
        assert!(
            visible >= 50,
            "la vista {} debería mostrar terreno, no solo fondo ({visible} píxeles)",
            view_mode.name()
        );
    }

    // Las seis capturas difieren dos a dos (Sky usa el mismo nivel de luz
    // que Light, pero en azul frío en vez del mapa de calor ámbar).
    for (i, (view_a, image_a)) in shots.iter().enumerate() {
        for (view_b, image_b) in &shots[i + 1..] {
            let diff = diff_pixels(image_a, image_b);
            assert!(
                diff >= 20,
                "las vistas {} y {} casi no difieren ({diff} de 9216 píxeles)",
                view_a.name(),
                view_b.name()
            );
        }
    }

    // La vista Light tiene píxeles claros (el patio, a plena luz de cielo) y
    // oscuros (el fondo de la galería derecha). Se mira solo el centro de la
    // imagen para no contar el cielo como oscuridad.
    let light_image = &shots[1].1;
    let bright = light_image
        .pixels()
        .filter(|&p| luminance(p) >= 200)
        .count();
    let dark = light_image
        .enumerate_pixels()
        .filter(|(x, y, p)| (24..72).contains(x) && (24..72).contains(y) && luminance(p) <= 90)
        .count();
    assert!(
        bright >= 50,
        "la vista Light debería tener píxeles claros: {bright}"
    );
    assert!(
        dark >= 50,
        "la vista Light debería tener píxeles oscuros: {dark}"
    );

    // La vista Chunks pinta el damero: tinte cálido del chunk par y frío del
    // impar, ambos a ambos lados del límite x=16 del suelo. Los bytes son
    // sRGB (el objetivo de color es Rgba8UnormSrgb): lineal (1.0, 0.82, 0.55)
    // y (0.55, 0.72, 1.0) codificados.
    let chunks_image = &shots[4].1;
    let warm = chunks_image
        .pixels()
        .filter(|&p| tint_close(p, [255, 234, 196]))
        .count();
    let cool = chunks_image
        .pixels()
        .filter(|&p| tint_close(p, [196, 221, 255]))
        .count();
    assert!(
        warm >= 50 && cool >= 50,
        "la vista Chunks debería mostrar dos tintes del damero: cálido {warm}, frío {cool}"
    );
}
