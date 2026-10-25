//! Datos de los clips integrados del minero.
//!
//! Convención de rotaciones (`[X, Y, Z]` en radianes, `EulerRot::XYZ`): las
//! piezas que cuelgan (brazos y piernas) rotan hacia el frente (−Z) con X
//! positivo; torso y cabeza, que apuntan hacia arriba, se inclinan atrás con
//! X positivo. Es arte provisional de «minero cansado que carga peso»:
//! ángulos suaves, nada brusco. Las tablas llevan `#[rustfmt::skip]` para
//! mantener una pista por línea, con literales compactos (`.5`, `-.4`).

use crate::clip::{AnimClip, ClipMarker, MarkerKind, PartKeyframe, PartTrack};
use crate::skeleton::RigModel;

/// Tabla de pistas: id de pieza y claves `(tiempo, valores)`.
type TrackTable = &'static [(&'static str, &'static [(f32, [f32; 3])])];

/// Datos de un clip, listos para resolverse contra un modelo.
struct ClipData {
    id: &'static str,
    duration_s: f32,
    looping: bool,
    /// Pistas de rotación con offset cero.
    rot_tracks: TrackTable,
    /// Pistas de vaivén: los valores son desplazamientos, rotación cero.
    offset_tracks: TrackTable,
    /// Marcadores `(clase, tiempo)`.
    markers: &'static [(MarkerKind, f32)],
}

/// Pista desde la tabla: los valores son rotación, o desplazamiento si
/// `sway` indica pista de vaivén.
fn track(model: &RigModel, id: &str, keys: &'static [(f32, [f32; 3])], sway: bool) -> PartTrack {
    PartTrack {
        part_index: model.part_index(id).unwrap_or(0),
        keyframes: keys
            .iter()
            .map(|&(time_s, values)| PartKeyframe {
                time_s,
                rot_rad: if sway { [0.0, 0.0, 0.0] } else { values },
                offset_px: if sway { values } else { [0.0, 0.0, 0.0] },
            })
            .collect(),
    }
}

/// Resuelve los datos contra el modelo y arma el clip. Los índices de pieza
/// salen defensivos (0 si un id desapareciera): la validación de clips
/// contra el modelo es la red de seguridad.
fn build_clip(data: &ClipData, model: &RigModel) -> AnimClip {
    let mut tracks: Vec<PartTrack> = data
        .rot_tracks
        .iter()
        .map(|(id, keys)| track(model, id, keys, false))
        .collect();
    tracks.extend(
        data.offset_tracks
            .iter()
            .map(|(id, keys)| track(model, id, keys, true)),
    );
    AnimClip {
        id: data.id.into(),
        duration_s: data.duration_s,
        looping: data.looping,
        tracks,
        markers: data
            .markers
            .iter()
            .map(|&(kind, time_s)| ClipMarker { kind, time_s })
            .collect(),
    }
}

/// Los siete clips base del minero, con índices de pieza resueltos contra
/// `RigModel::miner_default()`.
pub(crate) fn miner_clips() -> Vec<AnimClip> {
    let model = RigModel::miner_default();
    vec![
        build_clip(&IDLE, &model),
        build_clip(&WALK, &model),
        build_clip(&RUN, &model),
        build_clip(&DIG, &model),
        build_clip(&PLACE, &model),
        build_clip(&LADDER, &model),
        build_clip(&CARRY, &model),
    ]
}

/// `idle`: parado, respirando; vaivén sutil (±0,02–0,06 rad) de torso,
/// cabeza y brazos, y un descenso de pelvis de 0,4 px a mitad de ciclo.
#[rustfmt::skip]
const IDLE: ClipData = ClipData {
    id: "idle",
    duration_s: 4.0,
    looping: true,
    rot_tracks: &[
        ("torso", &[(0., [0., 0., 0.]), (2., [0.03, 0., 0.01]), (4., [0., 0., 0.])]),
        ("head", &[(0., [0., 0., 0.]), (2., [-0.02, 0.05, 0.]), (4., [0., 0., 0.])]),
        ("upper_arm_l", &[(0., [0.02, 0., 0.03]), (2., [0.06, 0., 0.05]), (4., [0.02, 0., 0.03])]),
        ("upper_arm_r", &[(0., [0.02, 0., -0.03]), (2., [0.06, 0., -0.05]), (4., [0.02, 0., -0.03])]),
    ],
    offset_tracks: &[("pelvis", &[(0., [0., 0., 0.]), (2., [0., -0.4, 0.]), (4., [0., 0., 0.])])],
    markers: &[],
};

/// `walk`: andar a paso tranquilo; piernas ±0,55 rad y brazos en contrafase.
/// Cada pie pasa bajo el cuerpo (apoya) a un cuarto y a tres cuartos del
/// ciclo, de ahí los marcadores; la rodilla se flexiona (X negativo) en la
/// mitad de balanceo de cada pierna.
#[rustfmt::skip]
const WALK: ClipData = ClipData {
    id: "walk",
    duration_s: 1.0,
    looping: true,
    rot_tracks: &[
        ("torso", &[(0., [0.05, 0., 0.]), (0.5, [0.05, 0.05, 0.]), (1., [0.05, 0., 0.])]),
        ("head", &[(0., [0., 0., 0.]), (0.5, [0., -0.06, 0.]), (1., [0., 0., 0.])]),
        ("upper_arm_l", &[(0., [-0.4, 0., 0.04]), (0.5, [0.4, 0., 0.04]), (1., [-0.4, 0., 0.04])]),
        ("upper_arm_r", &[(0., [0.4, 0., -0.04]), (0.5, [-0.4, 0., -0.04]), (1., [0.4, 0., -0.04])]),
        ("lower_arm_l", &[(0., [0.15, 0., 0.]), (0.5, [0.3, 0., 0.]), (1., [0.15, 0., 0.])]),
        ("lower_arm_r", &[(0., [0.3, 0., 0.]), (0.5, [0.15, 0., 0.]), (1., [0.3, 0., 0.])]),
        ("upper_leg_l", &[(0., [0.55, 0., 0.]), (0.5, [-0.55, 0., 0.]), (1., [0.55, 0., 0.])]),
        ("upper_leg_r", &[(0., [-0.55, 0., 0.]), (0.5, [0.55, 0., 0.]), (1., [-0.55, 0., 0.])]),
        ("lower_leg_l", &[(0., [-0.08, 0., 0.]), (0.5, [-0.2, 0., 0.]), (0.75, [-0.65, 0., 0.]), (1., [-0.08, 0., 0.])]),
        ("lower_leg_r", &[(0., [-0.2, 0., 0.]), (0.25, [-0.65, 0., 0.]), (0.5, [-0.08, 0., 0.]), (1., [-0.2, 0., 0.])]),
    ],
    offset_tracks: &[("pelvis", &[(0., [0., 0., 0.]), (0.5, [0., -0.8, 0.]), (1., [0., 0., 0.])])],
    markers: &[(MarkerKind::StepLeft, 0.25), (MarkerKind::StepRight, 0.75)],
};

/// `run`: trote; zancada más amplia (±0,75), torso inclinado al frente
/// (X negativo al apuntar arriba), codos bien flexionados y rebote de
/// pelvis más alto.
#[rustfmt::skip]
const RUN: ClipData = ClipData {
    id: "run",
    duration_s: 0.6,
    looping: true,
    rot_tracks: &[
        ("torso", &[(0., [-0.16, 0.06, 0.]), (0.3, [-0.18, -0.06, 0.]), (0.6, [-0.16, 0.06, 0.])]),
        ("head", &[(0., [0.1, -0.05, 0.]), (0.3, [0.12, 0.05, 0.]), (0.6, [0.1, -0.05, 0.])]),
        ("upper_arm_l", &[(0., [-0.7, 0., 0.05]), (0.3, [0.7, 0., 0.05]), (0.6, [-0.7, 0., 0.05])]),
        ("upper_arm_r", &[(0., [0.7, 0., -0.05]), (0.3, [-0.7, 0., -0.05]), (0.6, [0.7, 0., -0.05])]),
        ("lower_arm_l", &[(0., [0.9, 0., 0.]), (0.3, [1.1, 0., 0.]), (0.6, [0.9, 0., 0.])]),
        ("lower_arm_r", &[(0., [1.1, 0., 0.]), (0.3, [0.9, 0., 0.]), (0.6, [1.1, 0., 0.])]),
        ("upper_leg_l", &[(0., [0.75, 0., 0.]), (0.3, [-0.75, 0., 0.]), (0.6, [0.75, 0., 0.])]),
        ("upper_leg_r", &[(0., [-0.75, 0., 0.]), (0.3, [0.75, 0., 0.]), (0.6, [-0.75, 0., 0.])]),
        ("lower_leg_l", &[(0., [-0.15, 0., 0.]), (0.3, [-0.3, 0., 0.]), (0.45, [-0.95, 0., 0.]), (0.6, [-0.15, 0., 0.])]),
        ("lower_leg_r", &[(0., [-0.3, 0., 0.]), (0.15, [-0.95, 0., 0.]), (0.3, [-0.15, 0., 0.]), (0.6, [-0.3, 0., 0.])]),
    ],
    offset_tracks: &[("pelvis", &[(0., [0., 0., 0.]), (0.3, [0., -1.2, 0.]), (0.6, [0., 0., 0.])])],
    markers: &[(MarkerKind::StepLeft, 0.15), (MarkerKind::StepRight, 0.45)],
};

/// `dig`: el pico sube atrás, cae y el golpe cae en el Impact de 0,5 (valor
/// exacto del que depende la sincronía por tick); a 0,75 el pico se suelta
/// de la roca (Release). El brazo derecho manda y el torso acompaña con un
/// pequeño giro.
#[rustfmt::skip]
const DIG: ClipData = ClipData {
    id: "dig",
    duration_s: 1.0,
    looping: true,
    rot_tracks: &[
        ("torso", &[(0., [-0.06, -0.28, 0.05]), (0.5, [0.14, 0.32, -0.03]), (0.75, [0.08, 0.2, 0.]), (1., [-0.06, -0.28, 0.05])]),
        ("head", &[(0., [0.12, -0.15, 0.]), (0.5, [-0.08, 0.12, 0.]), (1., [0.12, -0.15, 0.])]),
        ("upper_arm_r", &[(0., [-2.35, 0., -0.15]), (0.5, [0.55, 0., -0.1]), (0.75, [0.3, 0., -0.1]), (1., [-2.35, 0., -0.15])]),
        ("lower_arm_r", &[(0., [0.65, 0., 0.]), (0.5, [0.1, 0., 0.]), (0.75, [0.3, 0., 0.]), (1., [0.65, 0., 0.])]),
        ("hand_r", &[(0., [-0.25, 0., 0.]), (0.5, [0.2, 0., 0.]), (1., [-0.25, 0., 0.])]),
        ("upper_arm_l", &[(0., [0.35, 0., 0.15]), (0.5, [-0.3, 0., 0.15]), (1., [0.35, 0., 0.15])]),
        ("lower_arm_l", &[(0., [0.2, 0., 0.]), (0.5, [0.4, 0., 0.]), (1., [0.2, 0., 0.])]),
        ("upper_leg_l", &[(0., [0.18, 0., 0.]), (0.5, [-0.12, 0., 0.]), (1., [0.18, 0., 0.])]),
        ("upper_leg_r", &[(0., [-0.12, 0., 0.]), (0.5, [0.18, 0., 0.]), (1., [-0.12, 0., 0.])]),
    ],
    offset_tracks: &[],
    markers: &[(MarkerKind::Impact, 0.5), (MarkerKind::Release, 0.75)],
};

/// `place`: ambas manos llevan el bloque al frente y lo sueltan en el
/// Release de 0,35; después recogen. No es cíclico: se reproduce por golpe.
#[rustfmt::skip]
const PLACE: ClipData = ClipData {
    id: "place",
    duration_s: 0.5,
    looping: false,
    rot_tracks: &[
        ("torso", &[(0., [-0.04, 0., 0.]), (0.2, [-0.12, 0., 0.]), (0.5, [-0.03, 0., 0.])]),
        ("head", &[(0., [-0.18, 0., 0.]), (0.2, [-0.22, 0., 0.]), (0.5, [-0.1, 0., 0.])]),
        ("upper_arm_l", &[(0., [0.85, 0., 0.1]), (0.2, [1.15, 0., 0.06]), (0.35, [1.3, 0., 0.04]), (0.5, [0.45, 0., 0.08])]),
        ("upper_arm_r", &[(0., [0.85, 0., -0.1]), (0.2, [1.15, 0., -0.06]), (0.35, [1.3, 0., -0.04]), (0.5, [0.45, 0., -0.08])]),
        ("lower_arm_l", &[(0., [0.35, 0., 0.]), (0.35, [0.5, 0., 0.]), (0.5, [0.25, 0., 0.])]),
        ("lower_arm_r", &[(0., [0.35, 0., 0.]), (0.35, [0.5, 0., 0.]), (0.5, [0.25, 0., 0.])]),
        ("hand_l", &[(0., [0., 0., 0.]), (0.35, [-0.35, 0., 0.]), (0.5, [0., 0., 0.])]),
        ("hand_r", &[(0., [0., 0., 0.]), (0.35, [-0.35, 0., 0.]), (0.5, [0., 0., 0.])]),
    ],
    offset_tracks: &[],
    markers: &[(MarkerKind::Release, 0.35)],
};

/// `ladder`: trepa de cara a la escalera; los brazos alcanzan alto y tiran,
/// y las piernas suben el pie al peldaño (apoyos a 0,3 y 0,9) alternando.
#[rustfmt::skip]
const LADDER: ClipData = ClipData {
    id: "ladder",
    duration_s: 1.2,
    looping: true,
    rot_tracks: &[
        ("torso", &[(0., [-0.1, 0., 0.]), (0.6, [-0.15, 0., 0.]), (1.2, [-0.1, 0., 0.])]),
        ("head", &[(0., [0.15, 0., 0.]), (0.6, [0.1, 0., 0.]), (1.2, [0.15, 0., 0.])]),
        ("upper_arm_l", &[(0., [2.55, 0., 0.1]), (0.6, [0.55, 0., 0.1]), (1.2, [2.55, 0., 0.1])]),
        ("upper_arm_r", &[(0., [0.55, 0., -0.1]), (0.6, [2.55, 0., -0.1]), (1.2, [0.55, 0., -0.1])]),
        ("lower_arm_l", &[(0., [0.25, 0., 0.]), (0.6, [0.45, 0., 0.]), (1.2, [0.25, 0., 0.])]),
        ("lower_arm_r", &[(0., [0.45, 0., 0.]), (0.6, [0.25, 0., 0.]), (1.2, [0.45, 0., 0.])]),
        ("upper_leg_l", &[(0., [0.85, 0., 0.]), (0.3, [0.45, 0., 0.]), (0.6, [0.15, 0., 0.]), (1.2, [0.85, 0., 0.])]),
        ("upper_leg_r", &[(0., [0.15, 0., 0.]), (0.6, [0.85, 0., 0.]), (0.9, [0.45, 0., 0.]), (1.2, [0.15, 0., 0.])]),
        ("lower_leg_l", &[(0., [-1.1, 0., 0.]), (0.3, [-0.5, 0., 0.]), (0.6, [-0.25, 0., 0.]), (1.2, [-1.1, 0., 0.])]),
        ("lower_leg_r", &[(0., [-0.25, 0., 0.]), (0.6, [-1.1, 0., 0.]), (0.9, [-0.5, 0., 0.]), (1.2, [-0.25, 0., 0.])]),
    ],
    offset_tracks: &[],
    markers: &[(MarkerKind::StepLeft, 0.3), (MarkerKind::StepRight, 0.9)],
};

/// `carry`: andar cargado; torso inclinado ~0,2 al frente, pasos más cortos
/// (±0,38) y brazos algo atrás para contrarrestar el peso. La mochila
/// acompaña el paso con un vaivén lateral.
#[rustfmt::skip]
const CARRY: ClipData = ClipData {
    id: "carry",
    duration_s: 1.2,
    looping: true,
    rot_tracks: &[
        ("torso", &[(0., [-0.2, 0.04, 0.]), (0.6, [-0.22, -0.04, 0.]), (1.2, [-0.2, 0.04, 0.])]),
        ("head", &[(0., [0.18, 0., 0.]), (0.6, [0.2, 0., 0.]), (1.2, [0.18, 0., 0.])]),
        ("pack", &[(0., [0., 0., 0.04]), (0.6, [0., 0., -0.04]), (1.2, [0., 0., 0.04])]),
        ("upper_arm_l", &[(0., [-0.35, 0., 0.08]), (0.6, [-0.2, 0., 0.08]), (1.2, [-0.35, 0., 0.08])]),
        ("upper_arm_r", &[(0., [-0.2, 0., -0.08]), (0.6, [-0.35, 0., -0.08]), (1.2, [-0.2, 0., -0.08])]),
        ("lower_arm_l", &[(0., [0.2, 0., 0.]), (0.6, [0.25, 0., 0.]), (1.2, [0.2, 0., 0.])]),
        ("lower_arm_r", &[(0., [0.25, 0., 0.]), (0.6, [0.2, 0., 0.]), (1.2, [0.25, 0., 0.])]),
        ("upper_leg_l", &[(0., [0.38, 0., 0.]), (0.6, [-0.38, 0., 0.]), (1.2, [0.38, 0., 0.])]),
        ("upper_leg_r", &[(0., [-0.38, 0., 0.]), (0.6, [0.38, 0., 0.]), (1.2, [-0.38, 0., 0.])]),
        ("lower_leg_l", &[(0., [-0.12, 0., 0.]), (0.6, [-0.25, 0., 0.]), (0.9, [-0.5, 0., 0.]), (1.2, [-0.12, 0., 0.])]),
        ("lower_leg_r", &[(0., [-0.25, 0., 0.]), (0.3, [-0.5, 0., 0.]), (0.6, [-0.12, 0., 0.]), (1.2, [-0.25, 0., 0.])]),
    ],
    offset_tracks: &[("pelvis", &[(0., [0., 0., 0.]), (0.6, [0., -0.5, 0.]), (1.2, [0., 0., 0.])])],
    markers: &[(MarkerKind::StepLeft, 0.3), (MarkerKind::StepRight, 0.9)],
};
