//! Pruebas de mirada, IK de pies y muelles.

use super::*;
use crate::pose::PartTransform;

/// Tolerancia para comparar radianes de mirada (componentes de cuaternión).
const EPS_RAD: f32 = 1e-4;
/// Tolerancia para comparar alturas de pies y pelvis, en píxeles.
const EPS_PX: f32 = 2.0;

/// Índice de una pieza del minero base.
fn part_of(model: &RigModel, id: &str) -> usize {
    model
        .part_index(id)
        .expect("pieza presente en el minero base")
}

/// ¿Están las dos rotaciones a menos de `EPS_RAD` por componente?
fn rot_close(actual: Quat, expected: Quat) -> bool {
    [
        (actual.x, expected.x),
        (actual.y, expected.y),
        (actual.z, expected.z),
        (actual.w, expected.w),
    ]
    .into_iter()
    .all(|(a, b)| (a - b).abs() < EPS_RAD)
}

/// Compara dos rotaciones por componentes con tolerancia.
fn assert_rot(actual: Quat, expected: Quat, ctx: &str) {
    assert!(
        rot_close(actual, expected),
        "{ctx}: {actual:?} ≠ {expected:?}"
    );
}

/// Punto local de la planta de un pie (bajo el pivote de la espinilla hasta
/// la cara inferior del cuboide), llevado al espacio del modelo.
fn sole_point(world: &WorldPose, model: &RigModel, lower_index: usize) -> Vec3 {
    let spec = model
        .parts()
        .get(lower_index)
        .expect("espinilla del minero base");
    let sole_local = Vec3::new(0.0, spec.cuboid_min_px[1] - spec.pivot_px[1], 0.0);
    world
        .parts
        .get(lower_index)
        .expect("matriz de la espinilla")
        .transform_point3(sole_local)
}

/// Desplazamiento local acumulado de la pelvis, componente Y.
fn pelvis_offset_y(pose: &LocalPose, model: &RigModel) -> f32 {
    pose.part(part_of(model, "pelvis"))
        .expect("pose completa")
        .offset_px
        .y
}

/// Exige que todas las matrices (piezas y enganches) sean finitas.
fn assert_matrices_finite(world: &WorldPose, ctx: &str) {
    for (kind, matrix) in world
        .parts
        .iter()
        .map(|matrix| ("pieza", matrix))
        .chain(world.attachments.iter().map(|matrix| ("enganche", matrix)))
    {
        for coeff in matrix.to_cols_array() {
            assert!(coeff.is_finite(), "{ctx}: matriz de {kind} con {coeff}");
        }
    }
}

#[test]
fn look_angles_split_35_65_between_torso_and_head() {
    let model = RigModel::miner_default();

    // Solo guiñada: cada pieza gira alrededor de Y su fracción exacta.
    let mut yaw_only = LocalPose::rest(&model);
    apply_look_angles(&mut yaw_only, &model, 0.5, 0.0);
    assert_rot(
        yaw_only
            .part(part_of(&model, "torso"))
            .expect("pose completa")
            .rot,
        Quat::from_rotation_y(0.5 * 0.35),
        "el torso toma el 35 % de la guiñada",
    );
    assert_rot(
        yaw_only
            .part(part_of(&model, "head"))
            .expect("pose completa")
            .rot,
        Quat::from_rotation_y(0.5 * 0.65),
        "la cabeza toma el 65 % de la guiñada",
    );

    // Solo cabeceo: mismo reparto alrededor de X.
    let mut pitch_only = LocalPose::rest(&model);
    apply_look_angles(&mut pitch_only, &model, 0.0, 0.4);
    assert_rot(
        pitch_only
            .part(part_of(&model, "torso"))
            .expect("pose completa")
            .rot,
        Quat::from_rotation_x(0.4 * 0.35),
        "el torso toma el 35 % del cabeceo",
    );
    assert_rot(
        pitch_only
            .part(part_of(&model, "head"))
            .expect("pose completa")
            .rot,
        Quat::from_rotation_x(0.4 * 0.65),
        "la cabeza toma el 65 % del cabeceo",
    );

    // El resto del cuerpo no acompaña la mirada.
    for id in ["pelvis", "upper_arm_l", "upper_leg_r", "pack"] {
        let transform = yaw_only.part(part_of(&model, id)).expect("pose completa");
        assert_eq!(
            transform,
            PartTransform::IDENTITY,
            "{id} no participa en la mirada"
        );
    }
}

#[test]
fn look_angles_clamp_to_per_part_limits() {
    let model = RigModel::miner_default();
    for &(yaw, pitch) in &[(10.0_f32, 0.0_f32), (-10.0, 0.0), (0.0, 10.0), (0.0, -10.0)] {
        let mut pose = LocalPose::rest(&model);
        apply_look_angles(&mut pose, &model, yaw, pitch);
        let torso = pose
            .part(part_of(&model, "torso"))
            .expect("pose completa")
            .rot;
        let head = pose
            .part(part_of(&model, "head"))
            .expect("pose completa")
            .rot;
        let expected_torso = if pitch == 0.0 {
            Quat::from_rotation_y(0.6 * yaw.signum())
        } else {
            Quat::from_rotation_x(0.25 * pitch.signum())
        };
        let expected_head = if pitch == 0.0 {
            Quat::from_rotation_y(1.0 * yaw.signum())
        } else {
            Quat::from_rotation_x(0.7 * pitch.signum())
        };
        assert_rot(torso, expected_torso, "el torso queda en su límite");
        assert_rot(head, expected_head, "la cabeza queda en su límite");
    }
}

#[test]
fn look_angles_skip_missing_parts_and_non_finite_input() {
    // Modelo sin torso ni cabeza: no pasa nada, sin pánico.
    let empty = RigModel::new(vec![], vec![]).expect("modelo vacío válido");
    let mut pose = LocalPose::rest(&empty);
    apply_look_angles(&mut pose, &empty, 1.0, 1.0);
    assert_eq!(pose.part(0), None, "el modelo vacío no tiene piezas");

    // Entradas no finitas: la pose queda exactamente como estaba.
    let model = RigModel::miner_default();
    let mut still = LocalPose::rest(&model);
    apply_look_angles(&mut still, &model, f32::NAN, 0.3);
    apply_look_angles(&mut still, &model, 0.3, f32::INFINITY);
    for index in 0..model.parts().len() {
        assert_eq!(
            still.part(index),
            Some(PartTransform::IDENTITY),
            "una mirada no finita no toca la pieza {index}"
        );
    }
}

#[test]
fn foot_ik_raises_left_foot_on_high_step() {
    let model = RigModel::miner_default();
    let mut pose = LocalPose::rest(&model);
    apply_foot_ik(&mut pose, &model, 0.5, 0.0);
    let world = evaluate_world_pose(&model, &pose);
    assert_matrices_finite(&world, "IK sobre el reposo");

    // Medio bloque arriba son 16 px: la planta izquierda sube hasta allí.
    let left = sole_point(&world, &model, part_of(&model, "lower_leg_l"));
    assert!(
        (left.y - 0.5 * PIXELS_PER_BLOCK).abs() < EPS_PX,
        "el pie izquierdo apoya el escalón: {left:?}"
    );
    // Conserva su horizontal: solo el IK corrige la altura.
    assert!(
        (left.x - (-2.5)).abs() < EPS_PX && left.z.abs() < EPS_PX,
        "la planta no deriva lateralmente: {left:?}"
    );
    // La rodilla adelanta hacia el frente (−Z), no hacia la espalda: su
    // pivote es el de la espinilla, no el del muslo (ese es la cadera).
    let knee = world
        .parts
        .get(part_of(&model, "lower_leg_l"))
        .expect("matriz de la espinilla")
        .transform_point3(Vec3::ZERO);
    assert!(
        knee.z < -5.0,
        "la rodilla izquierda avanza al frente: {knee:?}"
    );

    // El pie derecho sigue en el suelo y la pelvis no baja.
    let right = sole_point(&world, &model, part_of(&model, "lower_leg_r"));
    assert!(
        right.y.abs() < EPS_PX,
        "el pie derecho queda en el suelo: {right:?}"
    );
    assert!(
        pelvis_offset_y(&pose, &model).abs() < EPS_PX,
        "un escalón alto no baja la pelvis"
    );
}

#[test]
fn foot_ik_drops_pelvis_for_down_step() {
    let model = RigModel::miner_default();
    let mut pose = LocalPose::rest(&model);
    apply_foot_ik(&mut pose, &model, -0.5, 0.0);
    let world = evaluate_world_pose(&model, &pose);
    assert_matrices_finite(&world, "IK con escalón hacia abajo");

    // Medio bloque abajo: la pelvis baja lo que le falta a la pierna.
    let drop = pelvis_offset_y(&pose, &model);
    assert!(
        (drop - (-0.5 * PIXELS_PER_BLOCK)).abs() < EPS_PX,
        "la pelvis baja 16 px: {drop}"
    );
    let left = sole_point(&world, &model, part_of(&model, "lower_leg_l"));
    assert!(
        (left.y - (-0.5 * PIXELS_PER_BLOCK)).abs() < EPS_PX,
        "el pie izquierdo apoya el foso: {left:?}"
    );
    let right = sole_point(&world, &model, part_of(&model, "lower_leg_r"));
    assert!(
        right.y.abs() < EPS_PX,
        "el pie derecho queda en el suelo: {right:?}"
    );
}

#[test]
fn foot_ik_extreme_opposite_steps_stay_finite() {
    let model = RigModel::miner_default();
    let mut pose = LocalPose::rest(&model);
    apply_foot_ik(&mut pose, &model, 0.5, -0.5);
    let world = evaluate_world_pose(&model, &pose);
    assert_matrices_finite(&world, "IK con escalones opuestos al límite");

    // Y los offsets de entrada fuera de rango se acotan sin romperse.
    let mut clamped = LocalPose::rest(&model);
    apply_foot_ik(&mut clamped, &model, 12.0, f32::NEG_INFINITY);
    assert!(
        clamped
            .part(part_of(&model, "lower_leg_l"))
            .expect("pose completa")
            .rot
            .is_finite(),
        "un escalón absurdo sigue dando rotaciones finitas"
    );
}

#[test]
fn foot_ik_on_swinging_legs_keeps_feet_near_targets() {
    let model = RigModel::miner_default();
    let mut pose = LocalPose::rest(&model);
    // Marcha: el muslo izquierdo adelanta y el derecho retrae.
    pose.part_mut(part_of(&model, "upper_leg_l"))
        .expect("pose completa")
        .rot = Quat::from_rotation_x(0.3);
    pose.part_mut(part_of(&model, "upper_leg_r"))
        .expect("pose completa")
        .rot = Quat::from_rotation_x(-0.3);
    apply_foot_ik(&mut pose, &model, 0.5, 0.0);
    let world = evaluate_world_pose(&model, &pose);
    assert_matrices_finite(&world, "IK sobre piernas oscilando");

    // La corrección sagital es aproximada sobre una pose ya rotada: los pies
    // se acercan a sus objetivos sin atravesar el suelo.
    let left = sole_point(&world, &model, part_of(&model, "lower_leg_l"));
    assert!(
        (left.y - 0.5 * PIXELS_PER_BLOCK).abs() < EPS_PX,
        "el pie izquierdo en oscilación llega al escalón: {left:?}"
    );
    let right = sole_point(&world, &model, part_of(&model, "lower_leg_r"));
    assert!(
        right.y.abs() < EPS_PX && right.y > -EPS_PX,
        "el pie derecho queda sobre el suelo: {right:?}"
    );
}

#[test]
fn spring_converges_and_tolerates_huge_steps() {
    let mut spring = Spring1D::new(100.0, 10.0);
    // Dos segundos a 60 Hz: de 0 a 1, con margen.
    let dt = 1.0 / 60.0;
    for _ in 0..120 {
        spring.update(1.0, dt);
    }
    assert!(
        (spring.value() - 1.0).abs() < 0.01,
        "el muelle converge al objetivo: {}",
        spring.value()
    );

    // Pasos enormes: se acotan y se trocean; nada explota ni da NaN.
    for _ in 0..8 {
        spring.update(100.0, 1.0);
    }
    assert!(
        spring.value().is_finite() && spring.velocity().is_finite(),
        "un dt enorme no genera NaN: {}",
        spring.value()
    );
    assert!(
        (spring.value() - 100.0).abs() < 1.0,
        "tampoco explota: {}",
        spring.value()
    );

    // `reset` fija la posición y anula la velocidad: en su objetivo se para.
    spring.reset(5.0);
    assert!(
        (spring.value() - 5.0).abs() < EPS_RAD,
        "reset fija el valor"
    );
    assert_eq!(spring.velocity(), 0.0, "reset anula la velocidad");
    spring.update(5.0, dt);
    assert!(
        (spring.value() - 5.0).abs() < EPS_RAD && spring.velocity().abs() < EPS_RAD,
        "sin fuerza residual, el muelle no se mueve"
    );

    // Entradas no finitas o de tiempo nulo/negativo: no cambian nada.
    let before = (spring.value(), spring.velocity());
    spring.update(f32::NAN, dt);
    spring.update(1.0, f32::NAN);
    spring.update(1.0, 0.0);
    spring.update(1.0, -1.0);
    assert_eq!(
        (spring.value(), spring.velocity()),
        before,
        "las entradas inválidas dejan el muelle quieto"
    );
}

#[test]
fn spring_construction_clamps_non_finite_parameters() {
    // Rigidez y amortiguación no finitas caen a los extremos inferiores: el
    // muelle suave y sin amortiguar, pero finito y funcional.
    let mut spring = Spring1D::new(f32::NAN, f32::NAN);
    spring.update(1.0, 0.25);
    assert!(
        spring.value().is_finite() && spring.value() > 0.0,
        "con parámetros NaN el muelle sigue vivo y finito: {}",
        spring.value()
    );
    // Sin amortiguación la oscilación no decae (damping NaN → 0): a los 4 s
    // aún no se ha pegado al objetivo.
    for _ in 0..16 {
        spring.update(1.0, 0.25);
    }
    assert!(
        (spring.value() - 1.0).abs() > 0.1,
        "sin amortiguación no converge: {}",
        spring.value()
    );

    // Un reset con NaN deja el valor en cero, no en NaN.
    spring.reset(f32::NAN);
    assert_eq!(spring.value(), 0.0, "reset con NaN cae a cero");
}
