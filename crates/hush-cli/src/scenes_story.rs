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

/// HUD del modo Historia sobre una expedición real (semilla 42), con la
/// barra rápida llena y el minero a media vida y algo de hambre.
pub(super) fn story_hud_scene(variant: u64) -> Result<Option<TerrainScene>> {
    use hush_client::story_hud;
    use hush_client::story_view::{CraftRow, HudView};
    use hush_sim::story::inventory::ItemStack;
    use hush_sim::story::items::{COAL, ItemId, ToolKind, tool_item};
    use hush_sim::story::{Expedition, StoryConfig};

    let mut e = Expedition::new(StoryConfig {
        seed: 42,
        ..StoryConfig::default()
    })?;
    // Que caiga y se asiente antes de mirar.
    for _ in 0..40 {
        e.step(&[]);
    }
    let pick = tool_item(1, ToolKind::Pickaxe);
    let stacks = [
        ItemStack::tool(pick, pick.max_durability() * 2 / 5),
        ItemStack::new(ItemId::of_block(BlockKind::Plank), 12),
        ItemStack::new(ItemId::of_block(BlockKind::Torch), 7),
        ItemStack::new(ItemId(COAL), 3),
        ItemStack::new(ItemId::of_block(BlockKind::Rock), 40),
    ];
    for stack in stacks {
        e.inventory_mut().add(stack);
    }
    let mut view = HudView::from_expedition(&e, None);
    view.vitals.health = 13;
    view.vitals.food = 9;
    match variant {
        2 => {
            view.craft_open = true;
            view.craft_rows = [
                ("Tablón ×4", true),
                ("Mango ×4", true),
                ("Antorcha ×4", true),
                ("Banco de trabajo", false),
                ("Horno de piedra", true),
                ("Pico de piedra", true),
                ("Pico de hierro", false),
            ]
            .map(|(label, available)| CraftRow {
                label: label.to_owned(),
                available,
            })
            .to_vec();
            view.craft_selected = 4;
        }
        3 => view.paused = true,
        4 => {
            view.f3_open = true;
            view.fps = (144.0, 98.0, 6.94);
            view.facing = "norte";
        }
        5 => view.dead = true,
        _ => view.banner = Some(("Edad de piedra", 2.0)),
    }
    let mut ui = UiBatch::new();
    story_hud::draw(&mut ui, [1024, 640], &view);
    let eye = e.body().eye().map(|c| c as f64 / FIXED_ONE as f64);
    Ok(Some(TerrainScene {
        world: SceneWorld::Story(e.world().clone()),
        camera: Camera::Fly(FlyCamera {
            position: DVec3::from_array(eye),
            pitch: -10f32.to_radians(),
            ..FlyCamera::default()
        }),
        boxes: Vec::new(),
        ui,
        style: TerrainStyle {
            fog_distance: 56.0,
            ..TerrainStyle::default()
        },
    }))
}
