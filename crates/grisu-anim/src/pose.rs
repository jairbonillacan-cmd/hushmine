//! Poses locales, mezcla por capas y matrices de mundo por pieza.

use crate::skeleton::RigModel;
use glam::{EulerRot, Mat4, Quat, Vec3};

/// Amplitud de la respiración en reposo, en radianes.
const BREATHING_REST_AMPLITUDE: f32 = 0.008;
/// Amplitud adicional al llegar al cansancio máximo, en radianes.
const BREATHING_MAX_EXTRA_AMPLITUDE: f32 = 0.05;
/// Periodo de la respiración en reposo, en segundos.
const BREATHING_REST_PERIOD_S: f32 = 3.5;
/// Acortamiento del periodo con el cansancio al máximo, en segundos.
const BREATHING_MAX_PERIOD_CUT_S: f32 = 1.5;
/// Fracción de la oscilación del torso con la que acompaña la cabeza.
const BREATHING_HEAD_RATIO: f32 = 0.6;

/// Transformación local de una pieza respecto a su padre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PartTransform {
    /// Rotación local alrededor del pivote de la pieza.
    pub rot: Quat,
    /// Desplazamiento adicional en píxeles, en el marco girado de la pieza.
    pub offset_px: Vec3,
}

impl PartTransform {
    /// Transformación neutra: sin rotación y sin desplazamiento.
    pub const IDENTITY: PartTransform = PartTransform {
        rot: Quat::IDENTITY,
        offset_px: Vec3::ZERO,
    };

    /// Desde ángulos de Euler en radianes (convención `EulerRot::XYZ` de
    /// glam) y desplazamiento en píxeles.
    pub fn from_euler_rad(rot_rad: [f32; 3], offset_px: [f32; 3]) -> Self {
        Self {
            rot: Quat::from_euler(EulerRot::XYZ, rot_rad[0], rot_rad[1], rot_rad[2]),
            offset_px: Vec3::from(offset_px),
        }
    }

    /// Mezcla con otra transformación: rotación con `Quat::slerp` y
    /// desplazamiento con lerp. `alpha` debe venir ya acotado a [0, 1].
    fn mixed(self, other: Self, alpha: f32) -> Self {
        Self {
            rot: self.rot.slerp(other.rot, alpha),
            offset_px: self.offset_px.lerp(other.offset_px, alpha),
        }
    }
}

/// Pose local completa: una transformación por pieza, en el orden del modelo.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalPose {
    parts: Vec<PartTransform>,
}

impl LocalPose {
    /// Pose de reposo del modelo: rotaciones identidad y sin desplazamientos.
    pub fn rest(model: &RigModel) -> Self {
        Self {
            parts: vec![PartTransform::IDENTITY; model.parts().len()],
        }
    }

    /// Transformación de una pieza, si el índice es válido.
    pub fn part(&self, part_index: usize) -> Option<PartTransform> {
        self.parts.get(part_index).copied()
    }

    /// Referencia mutable a la transformación de una pieza, si es válida.
    pub fn part_mut(&mut self, part_index: usize) -> Option<&mut PartTransform> {
        self.parts.get_mut(part_index)
    }

    /// Mezcla lineal con otra pose: rotaciones con `Quat::slerp` y
    /// desplazamientos con lerp. `alpha` se acota a [0, 1]; los valores no
    /// finitos cuentan como 0.
    ///
    /// La pose resultado conserva la longitud de `self`: si `other` es más
    /// corta, las piezas que faltan mantienen el valor de `self`.
    pub fn blend(&self, other: &Self, alpha: f32) -> Self {
        let alpha = bounded_weight(alpha);
        let mut blended = self.parts.clone();
        for (mine, theirs) in blended.iter_mut().zip(other.parts.iter()) {
            *mine = mine.mixed(*theirs, alpha);
        }
        Self { parts: blended }
    }

    /// Superpone una acción de cintura para arriba (subárbol de `torso`:
    /// torso, cabeza, brazos, manos y mochila) sobre esta locomoción, con el
    /// peso dado (acotado a [0, 1]). Piernas y pelvis quedan intactas; las
    /// piezas sin homónima en `action` tampoco cambian.
    pub fn layer_upper_body(&mut self, action: &Self, model: &RigModel, weight: f32) {
        // Sin torso no hay cintura que separar: la capa no aplica.
        let Some(torso) = model.part_index("torso") else {
            return;
        };
        let weight = bounded_weight(weight);
        let in_subtree = subtree_of(model, torso);
        for (index, mine) in self.parts.iter_mut().enumerate() {
            // Solo se mezclan las piezas del subárbol presentes en ambas
            // poses; el resto de `self` ni se visita.
            if !in_subtree.get(index).copied().unwrap_or(false) {
                continue;
            }
            if let Some(theirs) = action.parts.get(index) {
                *mine = mine.mixed(*theirs, weight);
            }
        }
    }

    /// Respiración aditiva en torso y cabeza; `exhaustion` en [0, 1] escala
    /// la amplitud y acorta el periodo. No toca piernas ni brazos.
    ///
    /// La amplitud es `0,008 + 0,05·e²` radianes y el periodo `3,5 − 1,5·e`
    /// segundos; la cabeza acompaña al torso amortiguada al 60 %. Un tiempo
    /// negativo o no finito no hace nada.
    pub fn add_breathing(&mut self, model: &RigModel, time_s: f32, exhaustion: f32) {
        if !time_s.is_finite() || time_s < 0.0 {
            return;
        }
        let exhaustion = bounded_weight(exhaustion);
        let amplitude =
            BREATHING_REST_AMPLITUDE + BREATHING_MAX_EXTRA_AMPLITUDE * exhaustion * exhaustion;
        // Con `exhaustion` acotado, el periodo queda en [2, 3,5] s: nunca cero.
        let period_s = BREATHING_REST_PERIOD_S - BREATHING_MAX_PERIOD_CUT_S * exhaustion;
        let delta = amplitude * (std::f32::consts::TAU * time_s / period_s).sin();
        // Se pre-multiplica para que la oscilación viva en el marco del
        // modelo, no en el ya rotado de la pieza.
        if let Some(torso) = model
            .part_index("torso")
            .and_then(|index| self.part_mut(index))
        {
            torso.rot = Quat::from_rotation_x(delta) * torso.rot;
        }
        if let Some(head) = model
            .part_index("head")
            .and_then(|index| self.part_mut(index))
        {
            head.rot = Quat::from_rotation_x(BREATHING_HEAD_RATIO * delta) * head.rot;
        }
    }
}

/// Matrices por pieza y por enganche en el espacio del modelo (píxeles),
/// listas para enviar a una llamada instanciada.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldPose {
    pub parts: Vec<Mat4>,
    pub attachments: Vec<Mat4>,
}

/// Evalúa la cadena cinemática completa.
///
/// `M_hijo = M_padre · T(pivote_hijo − pivote_padre) · R · T(offset)`, y la
/// raíz cuelga de `T(pivote_raíz) · R · T(offset)`. Los enganches son
/// `A = M_pieza · T(offset)`. El que dibuja escala por `1 / PIXELS_PER_BLOCK`
/// para pasar al espacio del mundo.
///
/// Las piezas ausentes de la pose usan la transformación identidad, y las
/// entradas no finitas se tratan igual: ninguna matriz sale con NaN.
pub fn evaluate_world_pose(model: &RigModel, local: &LocalPose) -> WorldPose {
    let mut parts = Vec::with_capacity(model.parts().len());
    for (index, spec) in model.parts().iter().enumerate() {
        let transform = local
            .part(index)
            .map_or(PartTransform::IDENTITY, finite_or_identity);
        // Traslado hasta el pivote: la raíz desde el origen del modelo y las
        // hijas desde el pivote de su padre (ya evaluado: el modelo va en
        // orden topológico).
        let joint = match spec.parent {
            None => Mat4::from_translation(finite_or_zero(Vec3::from(spec.pivot_px))),
            Some(parent) => {
                let delta = model.parts().get(parent).map_or(Vec3::ZERO, |parent_spec| {
                    Vec3::from(spec.pivot_px) - Vec3::from(parent_spec.pivot_px)
                });
                parts.get(parent).copied().unwrap_or(Mat4::IDENTITY)
                    * Mat4::from_translation(finite_or_zero(delta))
            }
        };
        parts.push(
            joint * Mat4::from_quat(transform.rot) * Mat4::from_translation(transform.offset_px),
        );
    }
    let attachments = model
        .attachments()
        .iter()
        .map(|attachment| {
            // Un enganche fuera de rango no puede existir en un modelo
            // validado; por robustez, matriz identidad en su lugar.
            parts
                .get(attachment.part_index)
                .copied()
                .unwrap_or(Mat4::IDENTITY)
                * Mat4::from_translation(finite_or_zero(Vec3::from(attachment.offset_px)))
        })
        .collect();
    WorldPose { parts, attachments }
}

/// Acota un peso de mezcla a [0, 1]; los valores no finitos cuentan como 0.
fn bounded_weight(weight: f32) -> f32 {
    if weight.is_finite() {
        weight.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Marca el subárbol de `root`: ella misma y todos sus descendientes.
///
/// Las piezas del modelo van en orden topológico, así que una sola pasada
/// basta: al visitar cada pieza, la marca de su padre ya está calculada.
fn subtree_of(model: &RigModel, root: usize) -> Vec<bool> {
    let mut in_subtree = vec![false; model.parts().len()];
    for (index, part) in model.parts().iter().enumerate() {
        let parent_in = part
            .parent
            .and_then(|parent| in_subtree.get(parent).copied())
            .unwrap_or(false);
        in_subtree[index] = index == root || parent_in;
    }
    in_subtree
}

/// La transformación si es finita; si no, identidad, para no propagar NaN.
fn finite_or_identity(transform: PartTransform) -> PartTransform {
    if transform.rot.is_finite() && transform.offset_px.is_finite() {
        transform
    } else {
        PartTransform::IDENTITY
    }
}

/// El vector si es finito; si no, cero, para no construir matrices con NaN.
fn finite_or_zero(vector: Vec3) -> Vec3 {
    if vector.is_finite() {
        vector
    } else {
        Vec3::ZERO
    }
}

#[cfg(test)]
#[path = "pose_tests.rs"]
mod tests;
