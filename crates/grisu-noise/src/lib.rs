//! Ruido entero determinista en punto fijo Q16.16.
//!
//! Primitivas de ruido del mundo grande (ADR 0006): nada de `f32`/`f64` ni
//! funciones trascendentes, porque el cliente genera terreno en wasm y el
//! servidor en nativo y ambos tienen que obtener **el mismo valor bit a bit**
//! hasta ±7 499 999 bloques (INV-W03).
//!
//! - [`hash3d`]: hash espacial de `docs/mapgen/geologia.md` (SplitMix64),
//!   con coordenadas `i64` y una sal por capa ([`layer_seed`]).
//! - [`Noise2`] y [`Noise3`]: ruido de gradiente con retícula de período
//!   potencia de dos, curva quíntica en Q16 y salida en `[-ONE, ONE]`.
//! - [`Fbm2`] y [`Fbm3`]: suma de octavas con frecuencia doble y amplitud
//!   mitad. [`ridge`]: ruido en cresta. [`warp2`] y [`warp3`]: deformación
//!   de dominio.
//! - [`Coarse3`]: muestreo grueso 4×8×4 por chunk con interpolación
//!   trilineal en punto fijo.
//!
//! Todo son funciones puras de (semilla, coordenadas): sin reloj, sin estado
//! global y sin desbordamientos (aritmética `wrapping` o rangos acotados).

mod coarse;
mod fbm;
mod fixed;
mod gradient;
mod hash;
mod ridge;
mod warp;

pub use coarse::Coarse3;
pub use fbm::{Fbm2, Fbm3};
pub use fixed::{ONE, lerp, quintic};
pub use gradient::{Noise2, Noise3};
pub use hash::{hash2d, hash3d, layer_seed, splitmix64_step};
pub use ridge::ridge;
pub use warp::{warp2, warp3};
