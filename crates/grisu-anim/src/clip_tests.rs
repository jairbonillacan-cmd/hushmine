//! Pruebas de clips, marcadores y sincronía por tick (subagente 2).

use super::*;
use crate::pose::{LocalPose, PartTransform};
use crate::skeleton::RigModel;
use glam::Quat;

/// Tolerancia para comparar componentes `f32` (radianes y píxeles).
const EPS: f32 = 1e-4;
/// Tolerancia estricta para la aritmética de sincronía por tick.
const SYNC_EPS: f32 = 1e-6;
/// Triple cero, para abreviar claves.
const CERO: [f32; 3] = [0.0; 3];
/// Los siete ids de la biblioteca base, ordenados.
const BASE_IDS: [&str; 7] = ["carry", "dig", "idle", "ladder", "place", "run", "walk"];

/// El clip base con ese id, de la biblioteca integrada.
fn clip_by_id(id: &str) -> AnimClip {
    builtin_miner_clips()
        .into_iter()
        .find(|clip| clip.id == id)
        .unwrap_or_else(|| panic!("falta el clip {id} en la biblioteca base"))
}

/// Índice de una pieza del minero base.
fn part_of(model: &RigModel, id: &str) -> usize {
    model
        .part_index(id)
        .unwrap_or_else(|| panic!("falta la pieza {id}"))
}

/// Transformación de una pieza al muestrear un clip, para abreviar.
fn sampled(clip: &AnimClip, model: &RigModel, t: f32, part: usize) -> PartTransform {
    clip.sample_pose(model, t)
        .part(part)
        .expect("pieza con pista")
}

/// Clave con rotación dada y offset cero.
fn key_rot(t: f32, rot_rad: [f32; 3]) -> PartKeyframe {
    PartKeyframe {
        time_s: t,
        rot_rad,
        offset_px: CERO,
    }
}

/// Clave neutra en el instante `t`.
fn key(t: f32) -> PartKeyframe {
    key_rot(t, CERO)
}

/// Marcador de Impact en `t`, para abreviar.
fn impact_at(t: f32) -> ClipMarker {
    ClipMarker {
        kind: MarkerKind::Impact,
        time_s: t,
    }
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

/// Compara dos poses por pieza, con tolerancia.
fn assert_poses_close(a: &LocalPose, b: &LocalPose, ctx: &str) {
    let mut i = 0;
    while let (Some(pa), Some(pb)) = (a.part(i), b.part(i)) {
        assert!(rot_close(pa.rot, pb.rot), "{ctx}: rot en pieza {i}");
        let d_off = (pa.offset_px - pb.offset_px).length();
        assert!(d_off < EPS, "{ctx}: offset en pieza {i}");
        i += 1;
    }
}

/// Clip de prueba de una sola pista (brazo izquierdo), offset cero.
fn arm_clip(m: &RigModel, dur: f32, looping: bool, keys: &[(f32, [f32; 3])]) -> AnimClip {
    let keyframes = keys.iter().map(|&(t, rot)| key_rot(t, rot)).collect();
    let arm = part_of(m, "upper_arm_l");
    let track = PartTrack {
        part_index: arm,
        keyframes,
    };
    AnimClip {
        id: "prueba".into(),
        duration_s: dur,
        looping,
        tracks: vec![track],
        markers: Vec::new(),
    }
}

/// Reproduce `clip` tick a tick (`rate` t/s, tiempo de clip `(n/rate)/scale`)
/// y devuelve los ticks en los que dispara el marcador Impact.
fn impact_ticks_at_rate(clip: &AnimClip, scale: f32, rate: u32, horizon: u32) -> Vec<u32> {
    let mut fired = Vec::new();
    let mut prev = 0.0f32;
    for tick in 1..=horizon {
        let cur = (tick as f32 / rate as f32) / scale;
        for marker in clip.crossed_markers(prev, cur) {
            if marker.kind == MarkerKind::Impact {
                fired.push(tick);
            }
        }
        prev = cur;
    }
    fired
}

#[test]
fn builtin_returns_exactly_the_seven_ids() {
    let clips = builtin_miner_clips();
    let mut ids: Vec<&str> = clips.iter().map(|clip| clip.id.as_str()).collect();
    ids.sort_unstable();
    assert_eq!(ids, BASE_IDS);
}

#[test]
fn all_builtin_clips_are_valid() {
    let model = RigModel::miner_default();
    for clip in builtin_miner_clips() {
        clip.validate(&model)
            .unwrap_or_else(|e| panic!("el clip {} no valida: {e:?}", clip.id));
    }
}

#[test]
fn dig_scaled_lands_the_impact_on_the_requested_tick() {
    let dig = clip_by_id("dig");
    let impact_s = dig
        .markers
        .iter()
        .find(|m| m.kind == MarkerKind::Impact)
        .expect("dig lleva un marcador Impact")
        .time_s;

    // 15 ticks a 30 t/s: el impacto estirado cae en 0,5 s reales exactos.
    let s15 = dig.scale_for_impact_at_tick(15, 30);
    assert!((s15 - 1.0).abs() < SYNC_EPS);
    assert!((impact_s * s15 - 0.5).abs() < SYNC_EPS);

    // Reproducción tick a tick: cae al procesar el tick 15, ni antes ni
    // después (la siguiente vuelta sería el tick 45).
    let fired15 = impact_ticks_at_rate(&dig, s15, 30, 44);
    assert_eq!(fired15, vec![15]);

    // 30 ticks pedidos: la escala dobla y cae en el tick 30 (la siguiente
    // vuelta sería el tick 90).
    let s30 = dig.scale_for_impact_at_tick(30, 30);
    assert!((s30 - 2.0).abs() < SYNC_EPS);
    let fired30 = impact_ticks_at_rate(&dig, s30, 30, 89);
    assert_eq!(fired30, vec![30]);
}

#[test]
fn crossed_markers_fire_only_what_was_crossed() {
    let walk = clip_by_id("walk");

    // (0.1, 0.3] cruza el paso izquierdo (0.25) y no el derecho (0.75).
    let fired = walk.crossed_markers(0.1, 0.3);
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].kind, MarkerKind::StepLeft);
    assert_eq!(fired[0].time_s, 0.25);

    // (0.3, 0.5] ya no incluye el 0.25: el borde izquierdo es abierto.
    assert!(walk.crossed_markers(0.3, 0.5).is_empty());

    // Intervalos inservibles: nada que cruzar.
    assert!(
        walk.crossed_markers(0.5, 0.5).is_empty(),
        "vacío (t1 == t0)"
    );
    assert!(walk.crossed_markers(0.8, 0.3).is_empty(), "vacío (t1 < t0)");
    assert!(walk.crossed_markers(f32::NAN, 1.0).is_empty(), "t0 NaN");
    assert!(
        walk.crossed_markers(0.0, f32::INFINITY).is_empty(),
        "t1 infinito"
    );
}

#[test]
fn crossed_markers_wrap_the_cycle_in_looping_clips() {
    let walk = clip_by_id("walk");

    // (0.7, 1.3]: StepRight en 0.75 y después StepLeft en 1.25 (vuelta
    // siguiente), en ese orden y con su time_s original del clip.
    let fired = walk.crossed_markers(0.7, 1.3);
    assert_eq!(fired.len(), 2, "dos disparos con envoltura");
    assert_eq!(fired[0].kind, MarkerKind::StepRight, "primero el de 0.75");
    assert_eq!(fired[0].time_s, 0.75);
    assert_eq!(fired[1].kind, MarkerKind::StepLeft, "después el de 1.25");
    assert_eq!(fired[1].time_s, 0.25);

    // Hacia atrás también: (-0.3, 0.1] solo cruza el StepRight de la
    // vuelta anterior (instante −0.25).
    let fired = walk.crossed_markers(-0.3, 0.1);
    assert_eq!(fired.len(), 1, "un solo disparo con k = −1");
    assert_eq!(fired[0].kind, MarkerKind::StepRight);
    assert_eq!(fired[0].time_s, 0.75);
}

#[test]
fn crossed_markers_do_not_wrap_in_non_looping_clips() {
    let place = clip_by_id("place");

    // (0.0, 0.6]: el Release de esta pasada (0.35) sí dispara…
    let fired = place.crossed_markers(0.0, 0.6);
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].kind, MarkerKind::Release);
    assert_eq!(fired[0].time_s, 0.35);

    // …pero (0.4, 0.9] no dispara nada: el 0.85 sería una vuelta que no
    // existe en un clip no cíclico.
    assert!(place.crossed_markers(0.4, 0.9).is_empty());
}

#[test]
fn sample_pose_takes_exact_keyframe_values() {
    let model = RigModel::miner_default();
    let arm = part_of(&model, "upper_arm_l");
    let keys = [(0.0, CERO), (1.0, [1.0, 0.5, 0.25])];
    let mut clip = arm_clip(&model, 1.0, false, &keys);
    clip.tracks[0].keyframes[1].offset_px = [1.5, -0.5, 2.0];

    // En el instante de la clave final, sus valores exactos.
    let want = PartTransform::from_euler_rad([1.0, 0.5, 0.25], [1.5, -0.5, 2.0]);
    let got = sampled(&clip, &model, 1.0, arm);
    assert!(rot_close(got.rot, want.rot), "rotación de la clave");
    assert_eq!(got.offset_px, want.offset_px, "offset de la clave");

    // Piezas sin pista: identidad.
    let pelvis = part_of(&model, "pelvis");
    let pose = clip.sample_pose(&model, 0.4);
    assert_eq!(pose.part(pelvis), Some(PartTransform::IDENTITY));
}

#[test]
fn sample_pose_is_periodic_in_looping_clips() {
    let model = RigModel::miner_default();
    let walk = clip_by_id("walk");
    let at_t = walk.sample_pose(&model, 0.3);
    assert_poses_close(&at_t, &walk.sample_pose(&model, 1.3), "t y t+duración");
    assert_poses_close(&at_t, &walk.sample_pose(&model, -0.7), "t y t−duración");

    // Un tiempo NaN se trata como el arranque del clip.
    let start = walk.sample_pose(&model, 0.0);
    assert_poses_close(&walk.sample_pose(&model, f32::NAN), &start, "NaN = 0");

    // Las piezas sin pista del walk (manos y mochila) quedan en identidad.
    let pose = walk.sample_pose(&model, 0.37);
    let neutral = Some(PartTransform::IDENTITY);
    for id in ["hand_l", "hand_r", "pack"] {
        assert_eq!(pose.part(part_of(&model, id)), neutral, "{id} en reposo");
    }
}

#[test]
fn sample_pose_clamps_in_non_looping_clips() {
    let model = RigModel::miner_default();
    let place = clip_by_id("place");
    let end = place.sample_pose(&model, 10.0);
    assert_poses_close(&end, &place.sample_pose(&model, 0.5), "tras el final");
    let start = place.sample_pose(&model, -2.0);
    assert_poses_close(&start, &place.sample_pose(&model, 0.0), "antes del inicio");
}

#[test]
fn sample_pose_interpolates_smoothly_and_wraps() {
    let model = RigModel::miner_default();
    let arm = part_of(&model, "upper_arm_l");

    // A un cuarto del tramo, smoothstep(0.25) = 0.15625 del giro; y en
    // cualquier punto intermedio, entre las dos claves.
    let keys = [(0.0, CERO), (1.0, [1.0, 0.0, 0.0])];
    let clip = arm_clip(&model, 1.0, true, &keys);
    let angle = |t: f32| rot_angle_x(sampled(&clip, &model, t, arm).rot);
    assert!((angle(0.25) - 0.15625).abs() < EPS);
    for step in 0..=10 {
        let t = step as f32 / 10.0;
        let a = angle(t);
        assert!((-EPS..=1.0 + EPS).contains(&a), "desborda en t = {t}");
    }

    // Clave final en 0.5: el tramo 0.5 → 1.0 vuelve hacia la clave
    // inicial, como si estuviera en duración + primer.time_s.
    let keys = [(0.0, CERO), (0.5, [1.0, 0.0, 0.0])];
    let clip = arm_clip(&model, 1.0, true, &keys);
    let mid = rot_angle_x(sampled(&clip, &model, 0.75, arm).rot);
    assert!((mid - 0.5).abs() < EPS, "a medio camino (dio {mid})");
}

#[test]
fn validate_rejects_bad_durations() {
    let model = RigModel::miner_default();
    let keys = [(0.0, CERO), (1.0, [0.1, 0.0, 0.0])];
    for bad in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let mut clip = arm_clip(&model, 1.0, false, &keys);
        clip.duration_s = bad;
        assert_eq!(clip.validate(&model), Err(ClipError::BadDuration));
    }
}

#[test]
fn validate_rejects_bad_tracks() {
    let model = RigModel::miner_default();
    let arm = part_of(&model, "upper_arm_l");
    let keys = [(0.0, CERO), (1.0, [0.1, 0.0, 0.0])];
    let fresh = || arm_clip(&model, 1.0, false, &keys);

    // Pieza inexistente en el modelo.
    let mut clip = fresh();
    clip.tracks[0].part_index = model.parts().len();
    let res = clip.validate(&model);
    assert!(matches!(res, Err(ClipError::UnknownPart { .. })));

    // Pista vacía.
    let mut clip = fresh();
    clip.tracks[0].keyframes.clear();
    let want = Err(ClipError::EmptyTrack { part_index: arm });
    assert_eq!(clip.validate(&model), want, "pista sin claves");

    // Claves desordenadas y repetidas.
    for bad in [[key(0.2), key(0.1)], [key(0.3), key(0.3)]] {
        let mut clip = fresh();
        clip.tracks[0].keyframes = bad.to_vec();
        let want = Err(ClipError::UnsortedKeyframes { part_index: arm });
        assert_eq!(clip.validate(&model), want, "orden no estricto");
    }

    // Claves fuera de [0, duración], con el orden estricto intacto: una
    // negativa a la cabeza y otra sobre la duración al final.
    let mut clip = fresh();
    clip.tracks[0].keyframes = vec![key(0.0), key(1.5)];
    let want = Err(ClipError::KeyframeOutOfRange {
        part_index: arm,
        time_s: 1.5,
    });
    assert_eq!(clip.validate(&model), want, "clave sobre la duración");
    let mut clip = fresh();
    clip.tracks[0].keyframes = vec![key(-0.1), key(0.5)];
    let res = clip.validate(&model);
    assert!(matches!(res, Err(ClipError::KeyframeOutOfRange { .. })));

    // Marcadores fuera de [0, duración): en la duración exacta ya es la
    // vuelta siguiente, y un tiempo negativo no existe.
    for t in [1.0, -0.2] {
        let mut clip = fresh();
        clip.markers = vec![impact_at(t)];
        let want = Err(ClipError::MarkerOutOfRange {
            kind: MarkerKind::Impact,
            time_s: t,
        });
        assert_eq!(clip.validate(&model), want, "marcador en {t}");
    }
}

#[test]
fn scale_for_impact_degenerate_cases() {
    let model = RigModel::miner_default();
    let keys = [(0.0, CERO), (1.0, [0.1, 0.0, 0.0])];

    // Sin marcador Impact, la escala es neutra.
    let clip = arm_clip(&model, 1.0, false, &keys);
    let s = clip.scale_for_impact_at_tick(15, 30);
    assert!((s - 1.0).abs() < SYNC_EPS, "sin Impact");

    // Impacto en 0 (nada que estirar) o tasa nula (nada que dividir).
    let mut clip = arm_clip(&model, 1.0, false, &keys);
    clip.markers = vec![impact_at(0.0)];
    let s = clip.scale_for_impact_at_tick(15, 30);
    assert!((s - 1.0).abs() < SYNC_EPS, "Impact@0");
    let mut clip = arm_clip(&model, 1.0, false, &keys);
    clip.markers = vec![impact_at(0.5)];
    let s = clip.scale_for_impact_at_tick(15, 0);
    assert!((s - 1.0).abs() < SYNC_EPS, "tick_rate 0");

    // Con dos Impact desordenados manda el de menor tiempo (0.5, no 0.8).
    let mut clip = arm_clip(&model, 1.0, false, &keys);
    clip.markers = vec![impact_at(0.8), impact_at(0.5)];
    let s = clip.scale_for_impact_at_tick(15, 30);
    assert!((s - 1.0).abs() < SYNC_EPS, "usa el Impact menor");
    let s = clip.scale_for_impact_at_tick(30, 30);
    assert!((s - 2.0).abs() < SYNC_EPS);

    // Con un solo Impact en 0.8: 0.5 s reales / 0.8 s de clip = 0.625.
    let mut clip = arm_clip(&model, 1.0, false, &keys);
    clip.markers = vec![impact_at(0.8)];
    let s = clip.scale_for_impact_at_tick(15, 30);
    assert!((s - 0.625).abs() < SYNC_EPS);
}
