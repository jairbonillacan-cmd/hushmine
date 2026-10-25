//! Pruebas de poses, mezcla, capas y matrices.

use super::*;
use crate::skeleton::PartSpec;
use std::f32::consts::FRAC_PI_2;

/// Tolerancia para comparar componentes `f32` (radianes y píxeles).
const EPS: f32 = 1e-4;

/// Índice de una pieza del minero base.
fn part_of(model: &RigModel, id: &str) -> usize {
    model
        .part_index(id)
        .expect("pieza presente en el minero base")
}

/// Índice de un enganche del minero base.
fn attachment_of(model: &RigModel, id: &str) -> usize {
    model
        .attachment_index(id)
        .expect("enganche del minero base")
}

/// Longitud de una pose a través de su API pública.
fn pose_len(pose: &LocalPose) -> usize {
    (0..).take_while(|&i| pose.part(i).is_some()).count()
}

/// Punto local de una pieza llevado al espacio del modelo.
fn part_point(world: &WorldPose, index: usize, point: [f32; 3]) -> Vec3 {
    world.parts[index].transform_point3(Vec3::from(point))
}

/// Origen de la matriz de un enganche, en píxeles del modelo.
fn attachment_point(world: &WorldPose, index: usize) -> Vec3 {
    world.attachments[index].transform_point3(Vec3::ZERO)
}

/// ¿Están las dos rotaciones a menos de `EPS` por componente?
fn rot_close(a: Quat, b: Quat) -> bool {
    [(a.x, b.x), (a.y, b.y), (a.z, b.z), (a.w, b.w)]
        .into_iter()
        .all(|(x, y)| (x - y).abs() < EPS)
}

/// Ángulo de giro alrededor del eje X (0 en la identidad).
fn rot_angle_x(rot: Quat) -> f32 {
    2.0 * rot.x.abs().atan2(rot.w)
}

/// Exige que una pieza esté exactamente en la transformación neutra.
fn assert_rest(pose: &LocalPose, index: usize, ctx: &str) {
    assert_eq!(pose.part(index), Some(PartTransform::IDENTITY), "{ctx}");
}

/// Compara un punto con un triple esperado, con tolerancia.
fn assert_px(actual: Vec3, expected: [f32; 3], ctx: &str) {
    assert!(
        (actual - Vec3::from(expected)).length() < EPS,
        "{ctx}: esperaba {expected:?}, obtuve {actual:?}"
    );
}

/// Compara dos rotaciones por componentes con tolerancia.
fn assert_rot(actual: Quat, expected: Quat, ctx: &str) {
    assert!(
        rot_close(actual, expected),
        "{ctx}: {actual:?} ≠ {expected:?}"
    );
}

/// Compara dos desplazamientos con tolerancia.
fn assert_off(actual: Vec3, expected: Vec3, ctx: &str) {
    assert!(
        (actual - expected).length() < EPS,
        "{ctx}: {actual:?} ≠ {expected:?}"
    );
}

/// Compara dos poses completas: misma longitud y piezas tolerantes.
fn assert_pose(actual: &LocalPose, expected: &LocalPose, ctx: &str) {
    assert_eq!(
        pose_len(actual),
        pose_len(expected),
        "{ctx}: longitudes distintas"
    );
    for index in 0..pose_len(expected) {
        let a = actual.part(index).expect("pose completa");
        let b = expected.part(index).expect("pose completa");
        let ctx = format!("{ctx}: pieza {index}");
        assert_rot(a.rot, b.rot, &ctx);
        assert_off(a.offset_px, b.offset_px, &ctx);
    }
}

/// Pose de prueba con una rotación y un desplazamiento distintos por pieza.
fn displaced_pose(model: &RigModel) -> LocalPose {
    let mut pose = LocalPose::rest(model);
    for index in 0..model.parts().len() {
        let angle = 0.05 + 0.03 * index as f32;
        let euler = [angle, -0.5 * angle, 0.25 * angle];
        let t = PartTransform::from_euler_rad(euler, [index as f32, -2.0, 3.0]);
        *pose.part_mut(index).expect("pose completa") = t;
    }
    pose
}

/// Modelo de una sola pieza, sin `torso`, para pruebas de robustez.
fn single_part_model() -> RigModel {
    let blob = PartSpec {
        id: "blob".into(),
        parent: None,
        pivot_px: [0.0; 3],
        cuboid_min_px: [-1.0; 3],
        cuboid_max_px: [1.0; 3],
        uv: [0, 0],
    };
    RigModel::new(vec![blob], vec![]).expect("el modelo de una pieza es válido")
}

#[test]
fn rest_pose_has_identity_transform_per_part() {
    let model = RigModel::miner_default();
    let rest = LocalPose::rest(&model);
    assert_eq!(pose_len(&rest), 14, "el minero base tiene 14 piezas");
    for index in 0..model.parts().len() {
        assert_rest(&rest, index, "la pieza parte del reposo");
    }
}

#[test]
fn from_euler_rad_matches_glam_xyz() {
    let rot_rad = [0.3, -0.8, 1.25];
    let t = PartTransform::from_euler_rad(rot_rad, [1.5, -2.0, 0.25]);
    let expected = Quat::from_euler(EulerRot::XYZ, rot_rad[0], rot_rad[1], rot_rad[2]);
    assert_rot(t.rot, expected, "rotación desde ángulos de Euler XYZ");
    assert_px(
        t.offset_px,
        [1.5, -2.0, 0.25],
        "el desplazamiento pasa a Vec3",
    );
}

#[test]
fn blend_returns_endpoints_and_midpoint() {
    let model = RigModel::miner_default();
    let base = LocalPose::rest(&model);
    let target = displaced_pose(&model);

    // alpha 0, negativo y NaN cuentan como 0; 1 y mayores, como 1.
    for alpha in [0.0, -5.0, f32::NAN, 1.0, 9.0] {
        let expected = if alpha >= 0.5 { &target } else { &base };
        assert_pose(&base.blend(&target, alpha), expected, "alpha acotado");
    }

    // Punto medio: slerp en rotación y lerp en desplazamiento.
    let mid = base.blend(&target, 0.5);
    for index in 0..model.parts().len() {
        let a = base.part(index).expect("pose completa");
        let b = target.part(index).expect("pose completa");
        let m = mid.part(index).expect("pose completa");
        assert_rot(m.rot, a.rot.slerp(b.rot, 0.5), "rotación al 50 %");
        assert_off(
            m.offset_px,
            a.offset_px.lerp(b.offset_px, 0.5),
            "offset al 50 %",
        );
    }
}

#[test]
fn upper_body_layer_moves_arms_without_touching_legs() {
    let model = RigModel::miner_default();
    // La acción gira TODO el cuerpo, también las piernas: solo lo de cintura
    // para arriba debe colarse en la locomoción.
    let action = displaced_pose(&model);

    // Peso 1: el subárbol del torso toma los valores de la acción.
    let mut full = LocalPose::rest(&model);
    full.layer_upper_body(&action, &model, 1.0);
    for id in [
        "torso",
        "head",
        "upper_arm_l",
        "upper_arm_r",
        "lower_arm_l",
        "lower_arm_r",
        "hand_l",
        "hand_r",
        "pack",
    ] {
        let i = part_of(&model, id);
        let got = full.part(i).expect("pose completa");
        let want = action.part(i).expect("pose completa");
        let ctx = format!("peso 1 en {id}");
        assert_rot(got.rot, want.rot, &ctx);
        assert_off(got.offset_px, want.offset_px, &ctx);
    }
    // Pelvis y piernas quedan exactamente como en la base (reposo).
    for id in [
        "pelvis",
        "upper_leg_l",
        "upper_leg_r",
        "lower_leg_l",
        "lower_leg_r",
    ] {
        assert_rest(&full, part_of(&model, id), "queda intacta en reposo");
    }

    // Peso 0,5: el subárbol queda a mitad de camino desde el reposo.
    let mut half = LocalPose::rest(&model);
    half.layer_upper_body(&action, &model, 0.5);
    for id in ["torso", "head", "upper_arm_r", "hand_l", "pack"] {
        let i = part_of(&model, id);
        let got = half.part(i).expect("pose completa");
        let want = action.part(i).expect("pose completa");
        let ctx = format!("peso 0,5 en {id}");
        assert_rot(got.rot, Quat::IDENTITY.slerp(want.rot, 0.5), &ctx);
        assert_off(got.offset_px, Vec3::ZERO.lerp(want.offset_px, 0.5), &ctx);
    }
}

#[test]
fn breathing_scales_with_exhaustion_and_skips_legs() {
    let model = RigModel::miner_default();
    let torso = part_of(&model, "torso");
    let leg = part_of(&model, "upper_leg_l");

    // Con time_s = 0,5 ambos niveles respiran lejos del cruce por cero.
    let mut fresh = LocalPose::rest(&model);
    fresh.add_breathing(&model, 0.5, 0.0);
    let mut tired = LocalPose::rest(&model);
    tired.add_breathing(&model, 0.5, 1.0);

    // Las piernas ni se enteran.
    assert_rest(&fresh, leg, "la respiración no toca piernas");
    assert_rest(&tired, leg, "la respiración no toca piernas");

    // Mismo instante, distinto cansancio: gira distinto y con más amplitud.
    let fresh_torso = fresh.part(torso).expect("pose completa");
    let tired_torso = tired.part(torso).expect("pose completa");
    let same = rot_close(fresh_torso.rot, tired_torso.rot);
    assert!(!same, "el cansancio cambia la respiración del torso");
    assert!(
        rot_angle_x(fresh_torso.rot) < rot_angle_x(tired_torso.rot),
        "el torso cansado respira con más amplitud"
    );

    // La cabeza acompaña amortiguada al 60 % del torso.
    let fresh_head = fresh.part(part_of(&model, "head")).expect("pose completa");
    let damped = 0.6 * rot_angle_x(fresh_torso.rot);
    assert!(
        (rot_angle_x(fresh_head.rot) - damped).abs() < EPS,
        "la cabeza acompaña al torso amortiguada"
    );

    // Tiempo negativo o no finito: no hace nada.
    let mut still = LocalPose::rest(&model);
    still.add_breathing(&model, -1.0, 1.0);
    still.add_breathing(&model, f32::NAN, 1.0);
    assert_rest(&still, torso, "tiempo inválido no respira");
}

#[test]
fn rest_world_pose_aligns_head_and_plants_feet() {
    let model = RigModel::miner_default();
    let world = evaluate_world_pose(&model, &LocalPose::rest(&model));
    assert_eq!(world.parts.len(), 14, "una matriz por pieza");
    assert_eq!(world.attachments.len(), 3, "una matriz por enganche");

    // El pivote de la cabeza queda a 44 px del suelo.
    let head = part_point(&world, part_of(&model, "head"), [0.0, 0.0, 0.0]);
    assert_px(head, [0.0, 44.0, 0.0], "cabeza a 44 px del suelo");

    // Centro de la planta del pie (12 px bajo el tobillo): apoya en y = 0.
    let sole = part_point(&world, part_of(&model, "lower_leg_l"), [0.0, -12.0, 0.0]);
    assert_px(sole, [-2.5, 0.0, 0.0], "el pie izquierdo apoya en y = 0");
}

#[test]
fn rest_attachments_rest_on_pivot_plus_offset() {
    let model = RigModel::miner_default();
    let world = evaluate_world_pose(&model, &LocalPose::rest(&model));

    // Pivote de la mano derecha (7, 18, 0) más el offset (0, −2, −1).
    let tool = attachment_point(&world, attachment_of(&model, "tool"));
    assert_px(tool, [7.0, 16.0, -1.0], "herramienta en la mano derecha");
    // Pivote de la cabeza (0, 44, 0) más el offset de la lámpara (0, 6, −6).
    let lamp = attachment_point(&world, attachment_of(&model, "lamp"));
    assert_px(lamp, [0.0, 50.0, -6.0], "lámpara al frente del casco");
}

#[test]
fn torso_rotation_drags_head_and_pack() {
    let model = RigModel::miner_default();
    let mut local = LocalPose::rest(&model);
    let torso = part_of(&model, "torso");
    local.part_mut(torso).expect("pose completa").rot = Quat::from_rotation_x(FRAC_PI_2);
    let world = evaluate_world_pose(&model, &local);

    // La cabeza gira alrededor del pivote del torso: baja al frente.
    let head = part_point(&world, part_of(&model, "head"), [0.0, 0.0, 0.0]);
    assert_px(head, [0.0, 30.0, 14.0], "la cabeza gira con el torso");
    // La mochila también se mueve (en reposo estaba en (0, 44, 3)).
    let pack = part_point(&world, part_of(&model, "pack"), [0.0, 0.0, 0.0]);
    assert_px(pack, [0.0, 27.0, 14.0], "la mochila acompaña al torso");
    assert!(
        (pack - Vec3::new(0.0, 44.0, 3.0)).length() > 1.0,
        "la mochila se mueve respecto al reposo"
    );
}

#[test]
fn degenerate_inputs_do_nothing() {
    let model = RigModel::miner_default();
    let rest = LocalPose::rest(&model);

    // Índices fuera de rango: None, sin pánico.
    assert_eq!(rest.part(999), None, "part fuera de rango no existe");
    let mut mutable = rest.clone();
    assert!(mutable.part_mut(999).is_none(), "part_mut fuera de rango");

    // Mezcla con poses de distinta longitud: la larga conserva su cola.
    let empty_model = RigModel::new(vec![], vec![]).expect("modelo vacío válido");
    let empty = LocalPose::rest(&empty_model);
    assert_pose(
        &rest.blend(&empty, 0.7),
        &rest,
        "la vacía no recorta la base",
    );
    assert_pose(
        &empty.blend(&rest, 0.7),
        &empty,
        "la base no estira la corta",
    );

    // La capa con una acción vacía tampoco cambia nada.
    let mut layered = rest.clone();
    layered.layer_upper_body(&empty, &model, 1.0);
    assert_pose(&layered, &rest, "la acción vacía no cambia la locomoción");

    // Modelo sin torso: la capa y la respiración no hacen nada.
    let blob_model = single_part_model();
    let mut blob = LocalPose::rest(&blob_model);
    blob.layer_upper_body(&displaced_pose(&model), &blob_model, 1.0);
    blob.add_breathing(&blob_model, 1.0, 1.0);
    assert_rest(&blob, 0, "sin torso no hay capa ni respiración");

    // Evaluar con una pose vacía: las piezas caen a la identidad.
    let world = evaluate_world_pose(&model, &empty);
    assert_eq!(world.parts.len(), 14, "evalúa todas las piezas del modelo");
    let head = part_point(&world, part_of(&model, "head"), [0.0, 0.0, 0.0]);
    assert_px(
        head,
        [0.0, 44.0, 0.0],
        "pieza ausente evaluada como identidad",
    );

    // Pose con valores no finitos: las matrices salen finitas.
    let mut poisoned = LocalPose::rest(&model);
    let torso = part_of(&model, "torso");
    let torso = poisoned.part_mut(torso).expect("pose completa");
    torso.rot = Quat::from_xyzw(f32::NAN, 0.0, 0.0, f32::NAN);
    torso.offset_px = Vec3::new(f32::NAN, f32::INFINITY, 0.0);
    let world = evaluate_world_pose(&model, &poisoned);
    let sample = part_point(&world, part_of(&model, "hand_r"), [1.0, 2.0, 3.0]);
    assert!(sample.is_finite(), "la pose con NaN no genera matrices NaN");
}
