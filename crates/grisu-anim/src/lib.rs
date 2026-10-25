//! Animación por piezas rígidas, solo CPU.
//!
//! Aquí vive la animación que decidió el ADR 0003 (plan en
//! `docs/graficos/02-animacion.md`): los personajes son árboles de piezas
//! rígidas (cajas texturizadas con pivote) cuyas poses se calculan en el
//! cliente y dibuja `grisu-render` en una sola llamada instanciada. El crate
//! es **presentación pura**: calcula poses y matrices, jamás escribe en la
//! simulación; el clip lo elige `hush-client` con el estado y los eventos de
//! `hush-sim`, así que no hay dependencias `hush-*` (la regla del workspace
//! lo impide) y la única dependencia es `glam`.
//!
//! - [`RigModel`]: árbol de piezas validado (sin ciclos, padres antes que
//!   hijos, enganches existentes) más el minero base, de 14 piezas y 3
//!   enganches. Las medidas son píxeles con la escala canónica
//!   [`PIXELS_PER_BLOCK`]: 32 px = 1 bloque.
//! - [`AnimClip`]: clip de fotogramas clave con interpolación suave y
//!   marcadores de eventos (`impact`, `step_l`, `step_r`, `release`). La
//!   sincronía con la simulación se hace por ticks:
//!   [`AnimClip::scale_for_impact_at_tick`] estira el clip para que el golpe
//!   caiga en el tick exacto en que la simulación golpea o rompe.
//! - [`LocalPose`]: una transformación por pieza y la mezcla por capas:
//!   locomoción, acción de cintura para arriba
//!   ([`LocalPose::layer_upper_body`]) y respiración aditiva
//!   ([`LocalPose::add_breathing`]).
//! - [`apply_foot_ik`], [`apply_look_angles`] y [`Spring1D`]: la capa
//!   procedural final (apoyo de pies, mirada, inercia de muelles).
//! - [`evaluate_world_pose`]: evalúa la cadena cinemática completa: una
//!   matriz por pieza y otra por enganche.
//!
//! El pipeline de principio a fin: modelo → clip con marcadores → mezcla por
//! capas (locomoción → acción de cintura para arriba → respiración →
//! procedural: mirada, IK de pies, muelles) → [`evaluate_world_pose`], con
//! matrices en píxeles del espacio del modelo listas para una sola llamada
//! instanciada.

pub mod clip;
pub mod pose;
pub mod procedural;
pub mod skeleton;

// Datos de los clips integrados del minero.
mod builtin;

pub use clip::{
    AnimClip, ClipError, ClipMarker, MarkerKind, PartKeyframe, PartTrack, builtin_miner_clips,
};
pub use pose::{LocalPose, PartTransform, WorldPose, evaluate_world_pose};
pub use procedural::{Spring1D, apply_foot_ik, apply_look_angles};
pub use skeleton::{AttachmentSpec, PIXELS_PER_BLOCK, PartSpec, RigError, RigModel};
