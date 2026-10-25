//! Capa procedural final: mirada, IK de pies y muelles de inercia.

use crate::pose::{LocalPose, WorldPose, evaluate_world_pose};
use crate::skeleton::{PIXELS_PER_BLOCK, RigModel};
use glam::{EulerRot, Quat, Vec3};
use std::f32::consts::{FRAC_PI_2, PI};

/// Fracción del giro de mirada que acompaña el torso.
const LOOK_TORSO_FRACTION: f32 = 0.35;
/// Fracción del giro de mirada que completa la cabeza.
const LOOK_HEAD_FRACTION: f32 = 0.65;
/// Guiñada máxima del torso, en radianes (ancho de hombros).
const LOOK_TORSO_MAX_YAW_RAD: f32 = 0.6;
/// Cabeceo máximo del torso, en radianes (la lumbar no acompaña mucho).
const LOOK_TORSO_MAX_PITCH_RAD: f32 = 0.25;
/// Guiñada máxima de la cabeza, en radianes.
const LOOK_HEAD_MAX_YAW_RAD: f32 = 1.0;
/// Cabeceo máximo de la cabeza, en radianes (mentón a pecho y nuca al techo).
const LOOK_HEAD_MAX_PITCH_RAD: f32 = 0.7;

/// Escalón máximo admitido bajo cada pie, en bloques.
const IK_MAX_STEP_BLOCKS: f32 = 0.5;
/// Debajo de esta distancia cadera→planta (en píxeles) la pierna cuenta como
/// plegada por completo: la ley de cosenos dividiría entre cero.
const IK_MIN_REACH_PX: f32 = 1e-4;

/// Rigidez mínima del muelle (la máxima exige más de lo que amortigua).
const SPRING_MIN_STIFFNESS: f32 = 1.0;
/// Rigidez máxima del muelle: con subpasos de 1/240 s queda holgadamente
/// por debajo del límite de estabilidad del integrador semi-implícito.
const SPRING_MAX_STIFFNESS: f32 = 1000.0;
/// Amortiguación máxima del muelle.
const SPRING_MAX_DAMPING: f32 = 100.0;
/// Duración máxima de cada subpaso de integración, en segundos.
const SPRING_MAX_SUBSTEP_S: f32 = 1.0 / 240.0;
/// `dt` máximo admitido por llamada, en segundos: el resto se recorta.
const SPRING_MAX_DT_S: f32 = 0.25;

/// Reparte la mirada entre `torso` (35 %) y `head` (65 %), con límites
/// anatómicos por pieza. La rotación se compone sobre la pose existente.
pub fn apply_look_angles(pose: &mut LocalPose, model: &RigModel, yaw_rad: f32, pitch_rad: f32) {
    // Una mirada no finita no es una mirada: la pose queda como estaba.
    if !yaw_rad.is_finite() || !pitch_rad.is_finite() {
        return;
    }
    // Cada pieza recibe su fracción del giro, acotada a su propio rango
    // anatómico; así una mirada extrema deja el 100 % en la cabeza.
    let torso_yaw =
        (yaw_rad * LOOK_TORSO_FRACTION).clamp(-LOOK_TORSO_MAX_YAW_RAD, LOOK_TORSO_MAX_YAW_RAD);
    let torso_pitch = (pitch_rad * LOOK_TORSO_FRACTION)
        .clamp(-LOOK_TORSO_MAX_PITCH_RAD, LOOK_TORSO_MAX_PITCH_RAD);
    let head_yaw =
        (yaw_rad * LOOK_HEAD_FRACTION).clamp(-LOOK_HEAD_MAX_YAW_RAD, LOOK_HEAD_MAX_YAW_RAD);
    let head_pitch =
        (pitch_rad * LOOK_HEAD_FRACTION).clamp(-LOOK_HEAD_MAX_PITCH_RAD, LOOK_HEAD_MAX_PITCH_RAD);

    // Pre-multiplicar compone el giro en el marco del modelo, encima de lo
    // que la pose ya tenga (locomoción, acción, respiración). El cabeceo va
    // en X (positivo levanta la cara) y la guiñada en Y.
    if let Some(torso) = model
        .part_index("torso")
        .and_then(|index| pose.part_mut(index))
    {
        torso.rot = Quat::from_euler(EulerRot::XYZ, torso_pitch, torso_yaw, 0.0) * torso.rot;
    }
    if let Some(head) = model
        .part_index("head")
        .and_then(|index| pose.part_mut(index))
    {
        head.rot = Quat::from_euler(EulerRot::XYZ, head_pitch, head_yaw, 0.0) * head.rot;
    }
}

/// Apoya cada pie en su escalón.
///
/// `left_step_offset_blocks` y `right_step_offset_blocks` son la altura del
/// escalón bajo cada pie en bloques, rango esperado [-0.5, 0.5] (positivo:
/// escalón hacia arriba). Ajusta muslos y rodillas (`upper_leg_*` y
/// `lower_leg_*`) y baja la pelvis lo justo para que la pierna más exigente
/// no se estire más allá de su longitud. Los pies quedan a la altura pedida
/// cuando la geometría lo permite; si no, el resultado se acota y las
/// matrices siguen siendo finitas.
pub fn apply_foot_ik(
    pose: &mut LocalPose,
    model: &RigModel,
    left_step_offset_blocks: f32,
    right_step_offset_blocks: f32,
) {
    // Entradas no finitas: nada que apoyar.
    if !left_step_offset_blocks.is_finite() || !right_step_offset_blocks.is_finite() {
        return;
    }
    // Sin pelvis o sin alguna de las dos piernas completas no hay cadena que
    // resolver.
    let Some(pelvis) = model.part_index("pelvis") else {
        return;
    };
    let Some(left) = leg_chain(model, "upper_leg_l", "lower_leg_l") else {
        return;
    };
    let Some(right) = leg_chain(model, "upper_leg_r", "lower_leg_r") else {
        return;
    };

    // Altura objetivo de cada planta en píxeles del modelo: los pies del
    // minero reposan en y = 0 y el escalón escala por píxeles por bloque.
    let left_target_y = clamp_step_blocks(left_step_offset_blocks) * PIXELS_PER_BLOCK;
    let right_target_y = clamp_step_blocks(right_step_offset_blocks) * PIXELS_PER_BLOCK;

    // Primera evaluación: caderas actuales. Bajar un pie en un escalón
    // pide más pierna de la que hay: la pelvis desciende lo que sobre a la
    // pierna más exigida. (Aproximación exacta si la pelvis no está
    // cabeceada: el descenso va en su `offset_px.y` local.)
    let world = evaluate_world_pose(model, pose);
    let drop = (hip_y(&world, left.upper_index) - left_target_y - left.total_px())
        .max(hip_y(&world, right.upper_index) - right_target_y - right.total_px())
        .max(0.0);
    if drop > 0.0
        && let Some(pelvis_part) = pose.part_mut(pelvis)
    {
        pelvis_part.offset_px.y -= drop;
    }

    // Segunda evaluación con la pelvis ya bajada: resolver cada pierna en el
    // plano sagital contra su escalón.
    let world = evaluate_world_pose(model, pose);
    solve_leg(pose, &left, hip_y(&world, left.upper_index), left_target_y);
    solve_leg(
        pose,
        &right,
        hip_y(&world, right.upper_index),
        right_target_y,
    );
}

/// Cadena estática de una pierna: índices de muslo y espinilla más las
/// longitudes de ambos huesos, en píxeles.
struct LegChain {
    upper_index: usize,
    lower_index: usize,
    thigh_px: f32,
    shin_px: f32,
}

impl LegChain {
    /// Longitud total de la pierna estirada, en píxeles.
    fn total_px(&self) -> f32 {
        self.thigh_px + self.shin_px
    }
}

/// Construye la cadena de una pierna desde el modelo: cadera y rodilla por
/// sus pivotes, y la planta por la cara inferior del cuboide de la espinilla.
/// `None` si falta una pieza o la geometría no da huesos útiles.
fn leg_chain(model: &RigModel, upper_id: &str, lower_id: &str) -> Option<LegChain> {
    let upper_index = model.part_index(upper_id)?;
    let lower_index = model.part_index(lower_id)?;
    let upper = model.parts().get(upper_index)?;
    let lower = model.parts().get(lower_index)?;
    let hip_y = upper.pivot_px[1];
    let knee_y = lower.pivot_px[1];
    let foot_y = lower.cuboid_min_px[1];
    let thigh_px = hip_y - knee_y;
    let shin_px = knee_y - foot_y;
    // Pivotes invertidos o no finitos fallan la comparación: la pierna no
    // existe como hueso y no hay IK que resolver.
    if !(thigh_px > 0.0 && shin_px > 0.0) {
        return None;
    }
    Some(LegChain {
        upper_index,
        lower_index,
        thigh_px,
        shin_px,
    })
}

/// Acota un escalón, en bloques, al rango admitido por el IK.
fn clamp_step_blocks(step_blocks: f32) -> f32 {
    step_blocks.clamp(-IK_MAX_STEP_BLOCKS, IK_MAX_STEP_BLOCKS)
}

/// Altura (en píxeles del modelo) del pivote de una cadera ya evaluada. La
/// posición del pivote no depende de la rotación del muslo, solo de la
/// pelvis de la que cuelga.
fn hip_y(world: &WorldPose, upper_index: usize) -> f32 {
    world
        .parts
        .get(upper_index)
        .map_or(0.0, |matrix| matrix.transform_point3(Vec3::ZERO).y)
}

/// Resuelve una pierna en el plano sagital para apoyar su planta a
/// `target_y`: adelanta el muslo y pliega la rodilla hacia atrás. Con huesos
/// de igual longitud la planta conserva su posición horizontal por
/// construcción (el avance del muslo compensa el retroceso de la espinilla).
fn solve_leg(pose: &mut LocalPose, leg: &LegChain, hip_y_world: f32, target_y: f32) {
    // Distancia vertical disponible; si el objetivo queda más allá de la
    // pierna (o por encima de la cadera), se acota: apoyo lo más cercano.
    let reach = (hip_y_world - target_y).clamp(0.0, leg.total_px());
    let (thigh_angle, knee_bend) = two_bone_sagittal_angles(leg.thigh_px, leg.shin_px, reach);
    if let Some(thigh) = pose.part_mut(leg.upper_index) {
        // Giro X positivo: la rodilla avanza hacia el frente (−Z).
        thigh.rot = Quat::from_rotation_x(thigh_angle) * thigh.rot;
    }
    if let Some(shin) = pose.part_mut(leg.lower_index) {
        // La espinilla se pliega hacia atrás bajo la rodilla adelantada.
        shin.rot = Quat::from_rotation_x(-knee_bend) * shin.rot;
    }
}

/// Ángulos del IK analítico de dos huesos en el plano sagital: giro del
/// muslo desde la vertical y flexión de rodilla, para una distancia
/// cadera→planta de `d_px`. Todos los cocientes van acotados: jamás NaN.
fn two_bone_sagittal_angles(thigh_px: f32, shin_px: f32, d_px: f32) -> (f32, f32) {
    // Con la pierna plegada por completo no hay triángulo que resolver: el
    // divisor de la ley de cosenos se anularía.
    if d_px <= IK_MIN_REACH_PX {
        return (FRAC_PI_2, PI);
    }
    let cos_thigh = ((thigh_px * thigh_px + d_px * d_px - shin_px * shin_px)
        / (2.0 * thigh_px * d_px))
        .clamp(-1.0, 1.0);
    let cos_knee = ((thigh_px * thigh_px + shin_px * shin_px - d_px * d_px)
        / (2.0 * thigh_px * shin_px))
        .clamp(-1.0, 1.0);
    (cos_thigh.acos(), PI - cos_knee.acos())
}

/// Amortiguador de muelle de un grado de libertad, estable ante pasos de
/// tiempo grandes: integración semi-implícita en subpasos acotados.
///
/// Pensado para la inercia de la mochila y el balanceo de la lámpara.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring1D {
    value: f32,
    velocity: f32,
    stiffness: f32,
    damping: f32,
}

impl Spring1D {
    /// Crea el muelle en reposo en 0. `stiffness` y `damping` se acotan a
    /// rangos estables; los valores no finitos se tratan como los extremos.
    pub fn new(stiffness: f32, damping: f32) -> Self {
        let stiffness = if stiffness.is_finite() {
            stiffness
        } else {
            SPRING_MIN_STIFFNESS
        }
        .clamp(SPRING_MIN_STIFFNESS, SPRING_MAX_STIFFNESS);
        let damping =
            if damping.is_finite() { damping } else { 0.0 }.clamp(0.0, SPRING_MAX_DAMPING);
        Self {
            value: 0.0,
            velocity: 0.0,
            stiffness,
            damping,
        }
    }

    /// Avanza el muelle hacia `target` durante `dt_s` segundos. El paso se
    /// acota y se integra en subpasos de hasta 1/240 s; entradas no finitas
    /// o negativas se ignoran sin pánico.
    pub fn update(&mut self, target: f32, dt_s: f32) {
        if !target.is_finite() || !dt_s.is_finite() || dt_s <= 0.0 {
            return;
        }
        // Un `dt` enorme se recorta y se trocea: con k ≤ 1000 y c ≤ 100 el
        // semi-implícito es estable con subpasos de 1/240 s.
        let dt_s = dt_s.min(SPRING_MAX_DT_S);
        let substeps = (dt_s / SPRING_MAX_SUBSTEP_S).ceil().max(1.0) as u32;
        let h = dt_s / substeps as f32;
        for _ in 0..substeps {
            // Semi-implícito: la fuerza sale del estado actual y la posición
            // avanza con la velocidad ya corregida.
            let force = -self.stiffness * (self.value - target) - self.damping * self.velocity;
            self.velocity += force * h;
            self.value += self.velocity * h;
        }
    }

    /// Posición actual.
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Velocidad actual.
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    /// Fija posición y anula la velocidad (teletransportes).
    pub fn reset(&mut self, value: f32) {
        self.velocity = 0.0;
        self.value = if value.is_finite() { value } else { 0.0 };
    }
}

#[cfg(test)]
#[path = "procedural_tests.rs"]
mod tests;
