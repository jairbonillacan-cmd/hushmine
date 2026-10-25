//! Pruebas del esqueleto y de `RigModel::miner_default`.

use super::*;

/// Tolerancia para comparar píxeles del modelo (valores `f32` exactos en
/// binario, pero comparamos con margen por si algún dato se toca).
const EPS_PX: f32 = 1e-4;

/// Compara un triple de píxeles con tolerancia.
fn assert_px(actual: [f32; 3], expected: [f32; 3], ctx: &str) {
    let mut ok = true;
    for axis in 0..3 {
        ok &= (actual[axis] - expected[axis]).abs() < EPS_PX;
    }
    assert!(ok, "{ctx}: esperaba {expected:?}, obtuve {actual:?}");
}

/// Pieza mínima para armar modelos de prueba.
fn stub_part(id: &str, parent: Option<usize>) -> PartSpec {
    PartSpec {
        id: id.into(),
        parent,
        pivot_px: [0.0, 0.0, 0.0],
        cuboid_min_px: [-1.0, -1.0, -1.0],
        cuboid_max_px: [1.0, 1.0, 1.0],
        uv: [0, 0],
    }
}

#[test]
fn miner_default_has_14_parts_and_3_attachments() {
    let model = RigModel::miner_default();

    // Las 14 piezas, con esos ids, en ese orden y con esos padres.
    let expected: [(&str, Option<usize>); 14] = [
        ("pelvis", None),
        ("torso", Some(0)),
        ("head", Some(1)),
        ("upper_arm_l", Some(1)),
        ("upper_arm_r", Some(1)),
        ("lower_arm_l", Some(3)),
        ("lower_arm_r", Some(4)),
        ("hand_l", Some(5)),
        ("hand_r", Some(6)),
        ("upper_leg_l", Some(0)),
        ("upper_leg_r", Some(0)),
        ("lower_leg_l", Some(9)),
        ("lower_leg_r", Some(10)),
        ("pack", Some(1)),
    ];
    assert_eq!(
        model.parts().len(),
        expected.len(),
        "el minero base tiene 14 piezas"
    );
    for (index, (part, (id, parent))) in model.parts().iter().zip(expected).enumerate() {
        assert_eq!(part.id, id, "la pieza {index} del minero debe ser {id}");
        assert_eq!(part.parent, parent, "padre de {id} (índice {index})");
    }

    // Pivotes de las articulaciones clave, en píxeles.
    assert_px(
        model.parts()[0].pivot_px,
        [0.0, 27.0, 0.0],
        "pivote de pelvis",
    );
    assert_px(
        model.parts()[2].pivot_px,
        [0.0, 44.0, 0.0],
        "pivote de head",
    );
    assert_px(
        model.parts()[4].pivot_px,
        [7.0, 42.0, 0.0],
        "pivote del hombro derecho",
    );
    assert_px(
        model.parts()[8].pivot_px,
        [7.0, 18.0, 0.0],
        "pivote de hand_r",
    );
    assert_px(
        model.parts()[10].pivot_px,
        [2.5, 24.0, 0.0],
        "pivote de la cadera derecha",
    );
    assert_px(
        model.parts()[12].pivot_px,
        [2.5, 12.0, 0.0],
        "pivote de la rodilla derecha",
    );

    // Silueta: pies en y=0 y coronilla en y=54 (54/32 = 1,6875 ≈ 1,7
    // bloques, la altura de colisión de `hush-sim`).
    let top = model
        .parts()
        .iter()
        .map(|part| part.cuboid_max_px[1])
        .fold(f32::MIN, f32::max);
    let bottom = model
        .parts()
        .iter()
        .map(|part| part.cuboid_min_px[1])
        .fold(f32::MAX, f32::min);
    assert!(
        (top - 54.0).abs() < EPS_PX,
        "la coronilla del minero debe estar a 54 px, está a {top}"
    );
    assert!(
        bottom.abs() < EPS_PX,
        "los pies del minero deben estar en y=0, el punto más bajo es {bottom}"
    );

    // La mochila cuelga a la espalda (+Z) de su pivote en el torso.
    let pack = &model.parts()[13];
    assert_px(pack.pivot_px, [0.0, 44.0, 3.0], "pivote de pack");
    assert_px(
        pack.cuboid_min_px,
        [-4.0, 32.0, 3.0],
        "cuboide mínimo de pack",
    );
    assert_px(
        pack.cuboid_max_px,
        [4.0, 44.0, 7.0],
        "cuboid máximo de pack",
    );

    // Los 3 enganches, en orden, con su pieza y su offset.
    let attachments = model.attachments();
    assert_eq!(attachments.len(), 3, "el minero base tiene 3 enganches");
    let head = model.part_index("head").expect("existe head");
    let hand_r = model.part_index("hand_r").expect("existe hand_r");
    let torso = model.part_index("torso").expect("existe torso");
    assert_eq!(attachments[0].id, "lamp", "primer enganche: lamp");
    assert_eq!(attachments[0].part_index, head, "lamp va en head");
    assert_px(
        attachments[0].offset_px,
        [0.0, 6.0, -6.0],
        "offset de lamp (al frente, −Z)",
    );
    assert_eq!(attachments[1].id, "tool", "segundo enganche: tool");
    assert_eq!(attachments[1].part_index, hand_r, "tool va en hand_r");
    assert_px(
        attachments[1].offset_px,
        [0.0, -2.0, -1.0],
        "offset de tool (el del ejemplo RON del doc)",
    );
    assert_eq!(attachments[2].id, "back", "tercer enganche: back");
    assert_eq!(attachments[2].part_index, torso, "back va en torso");
    assert_px(
        attachments[2].offset_px,
        [0.0, 8.0, 4.0],
        "offset de back (a la espalda, +Z)",
    );
}

#[test]
fn miner_default_is_topological_and_rebuildable() {
    let model = RigModel::miner_default();
    for (index, part) in model.parts().iter().enumerate() {
        if let Some(parent) = part.parent {
            assert!(
                parent < index,
                "el padre {parent} de {} debe ir antes que el hijo {index}",
                part.id
            );
        }
    }
    let rebuilt = RigModel::new(model.parts().to_vec(), model.attachments().to_vec())
        .expect("reconstruir el minero con sus propias specs debe validar");
    assert_eq!(
        rebuilt.parts().len(),
        14,
        "el minero reconstruido conserva sus piezas"
    );
    assert_eq!(rebuilt.attachments().len(), 3, "y sus enganches");
}

#[test]
fn indices_by_id() {
    let model = RigModel::miner_default();
    assert_eq!(model.part_index("pelvis"), Some(0), "la raíz es pelvis");
    assert_eq!(model.part_index("head"), Some(2));
    assert_eq!(
        model.part_index("pack"),
        Some(13),
        "pack es la última pieza"
    );
    assert_eq!(
        model.part_index("cabeza"),
        None,
        "los ids van en inglés; uno desconocido da None"
    );
    assert_eq!(model.attachment_index("lamp"), Some(0));
    assert_eq!(model.attachment_index("tool"), Some(1));
    assert_eq!(model.attachment_index("back"), Some(2));
    assert_eq!(
        model.attachment_index("mochila"),
        None,
        "enganche desconocido"
    );

    // Con ids repetidos gana la primera coincidencia.
    let twin = RigModel::new(
        vec![stub_part("dupe", None), stub_part("dupe", Some(0))],
        Vec::new(),
    )
    .expect("los ids repetidos no invalidan el modelo");
    assert_eq!(twin.part_index("dupe"), Some(0), "primera coincidencia");
}

#[test]
fn self_parenting_is_a_cycle() {
    let err = RigModel::new(vec![stub_part("a", Some(0))], Vec::new())
        .expect_err("una pieza que es su propio padre debe dar ParentCycle");
    assert_eq!(err, RigError::ParentCycle { part_index: 0 });
}

#[test]
fn two_part_cycle_yields_parent_cycle() {
    let parts = vec![stub_part("a", Some(1)), stub_part("b", Some(0))];
    let err = RigModel::new(parts, Vec::new())
        .expect_err("un ciclo de dos piezas debe dar ParentCycle aunque también incumpla el orden");
    assert_eq!(err, RigError::ParentCycle { part_index: 0 });
}

#[test]
fn cycle_reached_through_a_tail_reports_the_repeated_part() {
    // Cola (0, 1) que entra en un ciclo 2 → 3 → 2: el ciclo se detecta al
    // volver a la pieza 2, no en la pieza desde la que se empezó a subir.
    let parts = vec![
        stub_part("cola", None),
        stub_part("medio", Some(2)),
        stub_part("a", Some(3)),
        stub_part("b", Some(2)),
    ];
    let err =
        RigModel::new(parts, Vec::new()).expect_err("la cola debe llegar al ciclo y reportarlo");
    assert_eq!(err, RigError::ParentCycle { part_index: 2 });
}

#[test]
fn parent_with_higher_or_out_of_range_index_is_invalid() {
    let parts = vec![
        stub_part("raiz", None),
        stub_part("a", Some(0)),
        stub_part("b", Some(5)),
    ];
    let err = RigModel::new(parts, Vec::new())
        .expect_err("un padre fuera de rango debe dar ParentNotBeforeChild");
    assert_eq!(
        err,
        RigError::ParentNotBeforeChild {
            part_index: 2,
            parent_index: 5,
        }
    );
}

#[test]
fn parent_listed_after_child_is_invalid() {
    let parts = vec![stub_part("a", Some(1)), stub_part("b", None)];
    let err = RigModel::new(parts, Vec::new())
        .expect_err("un padre de índice mayor que el hijo debe dar ParentNotBeforeChild");
    assert_eq!(
        err,
        RigError::ParentNotBeforeChild {
            part_index: 0,
            parent_index: 1,
        }
    );
}

#[test]
fn attachment_to_missing_part_is_invalid() {
    let attachments = vec![AttachmentSpec {
        id: "linterna".into(),
        part_index: 7,
        offset_px: [0.0, 0.0, 0.0],
    }];
    let err = RigModel::new(vec![stub_part("a", None)], attachments).expect_err(
        "un enganche a la pieza 7 de un modelo de una pieza debe dar UnknownAttachmentPart",
    );
    assert_eq!(
        err,
        RigError::UnknownAttachmentPart {
            attachment_id: "linterna".into(),
            part_index: 7,
        }
    );
}

#[test]
fn empty_model_is_valid() {
    let model =
        RigModel::new(Vec::new(), Vec::new()).expect("un modelo sin piezas ni enganches es válido");
    assert!(model.parts().is_empty(), "sin piezas");
    assert!(model.attachments().is_empty(), "sin enganches");
    assert_eq!(model.part_index("pelvis"), None, "no hay nada que resolver");
    assert_eq!(model.attachment_index("lamp"), None);
}
