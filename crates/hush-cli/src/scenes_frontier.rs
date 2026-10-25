//! Escena de captura lejos del origen, para comprobar a ojo que el render
//! relativo al anchor no tiembla a millones de bloques.

use super::*;

/// El «Hito de la Frontera»: una pirámide con obelisco, pilares y vetas de
/// color sobre una losa, a 7,5 millones de bloques del origen. Un `f32` ahí
/// solo distingue bloques enteros; con el render relativo al anchor la
/// escena debe verse idéntica a como se vería junto al origen.
pub(super) fn frontier_scene() -> TerrainScene {
    /// Esquina del mundo grande donde se apoya la escena (Q47.16 alcanza de
    /// sobra: ±7,5 M bloques con 16 bits de fracción).
    const BASE: [i32; 3] = [7_499_800, 0, -7_499_800];
    let mut world = SparseWorld::new();
    let at = |dx: i32, dy: i32, dz: i32| [BASE[0] + dx, BASE[1] + dy, BASE[2] + dz];
    // Losa de roca dura de 25×25, un bloque de grueso.
    world.fill(at(-12, -1, -12), at(12, -1, 12), BlockKind::HardRock.id());
    // Pirámide de tres escalones con un obelisco coronado en oro.
    world.fill(at(-4, 0, -4), at(4, 0, 4), BlockKind::Rock.id());
    world.fill(at(-3, 1, -3), at(3, 1, 3), BlockKind::Rock.id());
    world.fill(at(-2, 2, -2), at(2, 2, 2), BlockKind::Rock.id());
    world.fill(at(0, 3, 0), at(0, 9, 0), BlockKind::Rock.id());
    world.fill(at(0, 10, 0), at(0, 10, 0), BlockKind::Gold.id());
    // Cuatro pilares de mena en las esquinas de la losa.
    for [sx, sz] in [[-9, -9], [9, -9], [-9, 9], [9, 9]] {
        world.fill(at(sx, 0, sz), at(sx, 3, sz), BlockKind::Ore.id());
    }
    // Franjas de diamante y oro para ver las texturas de lejos.
    world.fill(at(-10, 0, -2), at(-8, 0, 2), BlockKind::Diamond.id());
    world.fill(at(8, 0, -2), at(10, 0, 2), BlockKind::Gold.id());
    // Los chunks quedan cargados y marcados como sucios por `set`/`fill`.
    assert!(world.has_dirty_chunks());
    assert!(world.is_loaded(ChunkPos::containing(BASE)));
    let base = DVec3::new(f64::from(BASE[0]), f64::from(BASE[1]), f64::from(BASE[2]));
    TerrainScene {
        world: SceneWorld::Sparse(world),
        camera: Camera::Fly(FlyCamera {
            // Desde una esquina, mirando al obelisco en diagonal (+X, +Z).
            position: base + DVec3::new(-11.5, 6.5, -11.5),
            yaw: -3.0 * std::f32::consts::FRAC_PI_4,
            pitch: -14f32.to_radians(),
            ..FlyCamera::default()
        }),
        boxes: Vec::new(),
        ui: UiBatch::new(),
        style: TerrainStyle {
            fog_distance: 90.0,
            selected: Some(at(0, 10, 0)),
            lamp_radius: 0.0,
            time_seconds: 0.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontier_structure_is_far_from_the_origin_and_loaded() {
        let scene = frontier_scene();
        let SceneWorld::Sparse(world) = &scene.world else {
            unreachable!("la frontera es un SparseWorld");
        };
        assert!(!world.is_empty());
        // La cámara está a millones de bloques del origen.
        let Camera::Fly(camera) = &scene.camera else {
            unreachable!();
        };
        assert!(camera.position.x.abs() > 7.0e6);
        assert!(camera.position.z.abs() > 7.0e6);
        // Y el bloque señalado es el remate del obelisco.
        assert_eq!(scene.style.selected, Some([7_499_800, 10, -7_499_800]));
    }
}
