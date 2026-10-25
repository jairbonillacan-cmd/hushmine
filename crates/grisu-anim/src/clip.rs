//! Clips de fotogramas clave, marcadores de eventos y sincronía por tick.

use crate::pose::{LocalPose, PartTransform};
use crate::skeleton::RigModel;

/// Tope defensivo de vueltas por marcador en [`AnimClip::crossed_markers`]:
/// un intervalo absurdo (horas de avance en una sola consulta) se recorta en
/// vez de generar millones de disparos.
const MAX_WRAPS_PER_MARKER: i64 = 4096;

/// Clase de evento marcado en la línea de tiempo de un clip.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MarkerKind {
    /// El golpe cae: coincide con el tick en que la simulación rompe o golpea.
    Impact,
    /// El pie izquierdo apoya.
    StepLeft,
    /// El pie derecho apoya.
    StepRight,
    /// La mano suelta: el bloque o la carga deja de estar en la mano.
    Release,
}

/// Marcador temporal de un clip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipMarker {
    pub kind: MarkerKind,
    pub time_s: f32,
}

/// Fotograma clave de una pieza dentro de un clip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PartKeyframe {
    pub time_s: f32,
    /// Rotación en radianes, convención `EulerRot::XYZ` de glam.
    pub rot_rad: [f32; 3],
    pub offset_px: [f32; 3],
}

/// Pista de una pieza dentro de un clip.
#[derive(Clone, Debug, PartialEq)]
pub struct PartTrack {
    pub part_index: usize,
    pub keyframes: Vec<PartKeyframe>,
}

/// Errores de validación de un clip contra un modelo.
#[derive(Clone, Debug, PartialEq)]
pub enum ClipError {
    /// Duración no finita o menor o igual que cero.
    BadDuration,
    /// Pista sin fotogramas clave.
    EmptyTrack { part_index: usize },
    /// Fotogramas desordenados o duplicados en el tiempo.
    UnsortedKeyframes { part_index: usize },
    /// Fotograma fuera de [0, duración].
    KeyframeOutOfRange { part_index: usize, time_s: f32 },
    /// La pista apunta a una pieza que no existe en el modelo.
    UnknownPart { part_index: usize },
    /// Marcador fuera de [0, duración).
    MarkerOutOfRange { kind: MarkerKind, time_s: f32 },
}

/// Clip de animación: pistas por pieza más marcadores de eventos.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimClip {
    pub id: String,
    pub duration_s: f32,
    pub looping: bool,
    pub tracks: Vec<PartTrack>,
    pub markers: Vec<ClipMarker>,
}

impl AnimClip {
    /// Valida el clip contra un modelo: duración positiva, pistas con al
    /// menos dos fotogramas ordenados y dentro de [0, duración], piezas
    /// existentes y marcadores dentro de [0, duración).
    pub fn validate(&self, model: &RigModel) -> Result<(), ClipError> {
        if !self.duration_s.is_finite() || self.duration_s <= 0.0 {
            return Err(ClipError::BadDuration);
        }
        for track in &self.tracks {
            if track.part_index >= model.parts().len() {
                return Err(ClipError::UnknownPart {
                    part_index: track.part_index,
                });
            }
            if track.keyframes.is_empty() {
                return Err(ClipError::EmptyTrack {
                    part_index: track.part_index,
                });
            }
            // Estrictamente crecientes: un tiempo repetido o retrocedido
            // deja el orden sin definir y se rechaza.
            for pair in track.keyframes.windows(2) {
                if pair[1].time_s <= pair[0].time_s {
                    return Err(ClipError::UnsortedKeyframes {
                        part_index: track.part_index,
                    });
                }
            }
            // El rango de claves es cerrado: la clave final puede caer justo
            // en la duración. (Los NaN también caen aquí: no pertenecen al
            // rango y no hay forma sana de muestrearlos.)
            for keyframe in &track.keyframes {
                if !(0.0..=self.duration_s).contains(&keyframe.time_s) {
                    return Err(ClipError::KeyframeOutOfRange {
                        part_index: track.part_index,
                        time_s: keyframe.time_s,
                    });
                }
            }
        }
        // El rango de marcadores es abierto por la derecha: el instante igual
        // a la duración ya es la vuelta siguiente en un clip cíclico.
        for marker in &self.markers {
            if !(0.0..self.duration_s).contains(&marker.time_s) {
                return Err(ClipError::MarkerOutOfRange {
                    kind: marker.kind,
                    time_s: marker.time_s,
                });
            }
        }
        Ok(())
    }

    /// Muestrea la pose local en un instante dado. Interpolación suave
    /// (suavizado smoothstep) entre fotogramas clave; los clips cíclicos
    /// envuelven el tiempo y el tramo final enlaza con el primer fotograma.
    /// Las piezas sin pista quedan en reposo.
    pub fn sample_pose(&self, model: &RigModel, time_s: f32) -> LocalPose {
        let mut pose = LocalPose::rest(model);
        // Clip con duración inservible: pose de reposo, sin pánico.
        if !self.duration_s.is_finite() || self.duration_s <= 0.0 {
            return pose;
        }
        // Tiempo no finito (NaN o infinito): arranque del clip. El NaN se
        // trata aquí a propósito: `rem_euclid` lo propagaría.
        let t = if !time_s.is_finite() {
            0.0
        } else if self.looping {
            time_s.rem_euclid(self.duration_s)
        } else {
            time_s.clamp(0.0, self.duration_s)
        };
        for track in &self.tracks {
            // Índice de pieza fuera del modelo: la pista se ignora.
            if let Some(slot) = pose.part_mut(track.part_index) {
                *slot = sample_track(&track.keyframes, t, self.duration_s, self.looping);
            }
        }
        pose
    }

    /// Marcadores cruzados en el intervalo `(t0, t1]`, ordenados por tiempo
    /// de disparo. En clips cíclicos el intervalo puede envolver el final del
    /// clip; cada disparo aparece una vez por vuelta.
    pub fn crossed_markers(&self, t0: f32, t1: f32) -> Vec<ClipMarker> {
        // Intervalo vacío o con extremos inservibles: nada que cruzar.
        if !t0.is_finite() || !t1.is_finite() || t1 <= t0 {
            return Vec::new();
        }
        if !self.duration_s.is_finite() || self.duration_s <= 0.0 {
            return Vec::new();
        }
        let mut hits: Vec<(f32, ClipMarker)> = Vec::new();
        for marker in &self.markers {
            if !marker.time_s.is_finite() {
                continue;
            }
            if !self.looping {
                // Solo la vuelta 0 existe: el disparo es el propio marcador.
                if t0 < marker.time_s && marker.time_s <= t1 {
                    hits.push((marker.time_s, *marker));
                }
                continue;
            }
            // Instantes de disparo k·duración + time_s con k entero (también
            // negativo). Se barre el rango de k candidatas con una de margen
            // por cada lado y se comprueba cada instante de verdad: así los
            // redondeos del `floor` no cuelan ni tragan disparos del borde.
            let first_k = (((t0 - marker.time_s) / self.duration_s).floor() - 1.0) as i64;
            let last_k = (((t1 - marker.time_s) / self.duration_s).floor() + 1.0) as i64;
            let last_k = last_k.min(first_k.saturating_add(MAX_WRAPS_PER_MARKER));
            for k in first_k..=last_k {
                let instant = k as f32 * self.duration_s + marker.time_s;
                if t0 < instant && instant <= t1 {
                    hits.push((instant, *marker));
                }
            }
        }
        // Estable: los empates de instante conservan el orden de declaración.
        hits.sort_by(|a, b| a.0.total_cmp(&b.0));
        hits.into_iter().map(|(_, marker)| marker).collect()
    }

    /// Escala de tiempo para que el primer marcador [`MarkerKind::Impact`]
    /// caiga exactamente en el tick de simulación pedido.
    ///
    /// Devuelve `s` tal que, estirando el clip (duración por `s`, avance del
    /// tiempo de clip a `1 / s` segundos de clip por segundo real), el
    /// impacto ocurre a los `target_ticks / tick_rate` segundos. Si no hay
    /// marcador de impacto, si su tiempo no es positivo o si `tick_rate` es
    /// cero, devuelve 1.0.
    pub fn scale_for_impact_at_tick(&self, target_ticks: u32, tick_rate: u32) -> f32 {
        // El impacto de referencia es el de menor tiempo: la lista puede
        // venir desordenada y `total_cmp` decide también ante NaN.
        let Some(impact) = self
            .markers
            .iter()
            .filter(|marker| marker.kind == MarkerKind::Impact)
            .min_by(|a, b| a.time_s.total_cmp(&b.time_s))
        else {
            return 1.0;
        };
        // Tasa nula, impacto no positivo o no finito: escala neutra.
        if tick_rate == 0 || impact.time_s <= 0.0 || impact.time_s.is_nan() {
            return 1.0;
        }
        let scale = (target_ticks as f32 / tick_rate as f32) / impact.time_s;
        if scale.is_finite() { scale } else { 1.0 }
    }
}

/// Transformación directa de un fotograma clave.
fn keyframe_transform(keyframe: &PartKeyframe) -> PartTransform {
    PartTransform::from_euler_rad(keyframe.rot_rad, keyframe.offset_px)
}

/// Mezcla dos fotogramas con peso `s` ya acotado a [0, 1]: rotación con
/// `Quat::slerp` y desplazamiento con lerp, igual que hace `pose.rs`.
fn blend_keyframes(from: &PartKeyframe, to: &PartKeyframe, s: f32) -> PartTransform {
    let a = keyframe_transform(from);
    let b = keyframe_transform(to);
    PartTransform {
        rot: a.rot.slerp(b.rot, s),
        offset_px: a.offset_px.lerp(b.offset_px, s),
    }
}

/// Suavizado smoothstep de `u` acotado a [0, 1]: `u·u·(3 − 2·u)`.
fn smoothstep(u: f32) -> f32 {
    if !u.is_finite() {
        return 0.0;
    }
    let u = u.clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

/// Muestrea una pista en `t`, ya normalizado al rango del clip.
///
/// Antes del primer fotograma queda el primero y después del último, el
/// último; en clips cíclicos el tramo desde el último fotograma hasta el fin
/// de la vuelta enlaza interpolando hacia el PRIMER fotograma, como si
/// estuviera en `duración + primer.time_s`. Con claves desordenadas (clip sin
/// validar) hace lo que puede y nunca entra en pánico.
fn sample_track(keys: &[PartKeyframe], t: f32, duration_s: f32, looping: bool) -> PartTransform {
    let Some(first) = keys.first() else {
        return PartTransform::IDENTITY; // Pista vacía: transformación neutra.
    };
    let Some(last) = keys.last() else {
        return PartTransform::IDENTITY;
    };
    if t <= first.time_s {
        return keyframe_transform(first);
    }
    if t >= last.time_s {
        if !looping {
            return keyframe_transform(last);
        }
        // Tramo de envoltura hacia el primer fotograma de la vuelta siguiente.
        let wrap_end = duration_s + first.time_s;
        let span = wrap_end - last.time_s;
        if span > 0.0 && t < wrap_end {
            let u = ((t - last.time_s) / span).clamp(0.0, 1.0);
            return blend_keyframes(last, first, smoothstep(u));
        }
        return keyframe_transform(last);
    }
    // Entre dos claves: el primer segmento que rodea a `t`.
    for pair in keys.windows(2) {
        let (k0, k1) = (&pair[0], &pair[1]);
        if k1.time_s > k0.time_s && t >= k0.time_s && t <= k1.time_s {
            let u = ((t - k0.time_s) / (k1.time_s - k0.time_s)).clamp(0.0, 1.0);
            return blend_keyframes(k0, k1, smoothstep(u));
        }
    }
    // Solo alcanzable con claves desordenadas: se queda en la última.
    keyframe_transform(last)
}

/// Biblioteca base de clips del minero: `idle`, `walk`, `run`, `dig`,
/// `place`, `ladder` y `carry`.
pub fn builtin_miner_clips() -> Vec<AnimClip> {
    crate::builtin::miner_clips()
}

#[cfg(test)]
#[path = "clip_tests.rs"]
mod tests;
