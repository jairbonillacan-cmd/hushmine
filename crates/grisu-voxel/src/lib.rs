//! Grisú: terreno de vóxeles.
//!
//! Chunks de 16³ con un byte por bloque, edición con marcado de chunks
//! sucios, mallas por caras visibles con oclusión ambiental, luz por BFS,
//! raycast DDA y colisión en punto fijo. No sabe nada de gráficos ni de las reglas del
//! juego: para el motor un bloque es un `u8` y `0` es aire.

mod chunk;
mod collide;
mod light;
mod light_dual;
mod mesh;
mod raycast;
mod world;

pub use chunk::{CHUNK_SIZE, CHUNK_VOLUME, Chunk, ChunkDecodeError};
pub use collide::{Aabb, FIXED_ONE, MoveResult, move_aabb};
pub use light::{LightMap, MAX_LIGHT, emission_of};
pub use light_dual::DualLightMap;
pub use mesh::{Direction, PackedFace, mesh_chunk, mesh_chunk_dual};
pub use raycast::{RayHit, raycast};
pub use world::{MAX_WORLD_SIZE, VoxelWorld, WorldError};

/// Tipo de bloque tal como lo ve el motor. `AIR` es el único valor especial.
pub type BlockId = u8;

/// Bloque vacío.
pub const AIR: BlockId = 0;
