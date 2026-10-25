//! Escena de captura del modo Historia.

use super::*;

/// Mundo del modo historia: estratos generados, una galería con antorchas
/// y el pozo ancestral, visto al atardecer con luz dual.
pub(super) fn story_scene(seed: u64) -> Result<Option<TerrainScene>> {
    use hush_sim::story::{DayCycle, generate_terrain as generate_story};

    let generated = generate_story(seed, &Default::default())?;
    let mut world = generated.world;
    let bands = generated.bands;

    // Galería iluminada en el estrato sedimentario: así la captura enseña
    // antorchas (luz de bloque) contra el atardecer (luz de cielo baja).
    let y = bands.sedimentary - 6;
    world.fill([20, y, 40], [70, y + 2, 44], AIR);
    for x in (24..70).step_by(8) {
        world.set(x, y + 1, 41, BlockKind::Torch.id());
        world.set(x, y + 1, 43, BlockKind::Torch.id());
    }
    world.set(46, y, 42, BlockKind::Rail.id());
    world.set(48, y, 42, BlockKind::Chest.id());
    world.set(50, y + 1, 43, BlockKind::PropBeam.id());

    let scene = TerrainScene {
        world: SceneWorld::Story(world),
        camera: Camera::Fly(FlyCamera {
            position: DVec3::new(22.5, f64::from(y + 1) + 0.55, 42.5),
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: -2f32.to_radians(),
            ..FlyCamera::default()
        }),
        boxes: Vec::new(),
        ui: UiBatch::new(),
        style: TerrainStyle {
            fog_distance: 48.0,
            selected: Some([48, y + 1, 42]),
            // Mitad del ocaso: la galería se sostiene con las antorchas.
            daylight: DayCycle::new(12_900).sky_factor_q8() as f32 / 256.0,
            fog_color: [0.05, 0.06, 0.10],
            lamp_radius: 0.0,
            time_seconds: 0.0,
        },
    };
    Ok(Some(scene))
}
