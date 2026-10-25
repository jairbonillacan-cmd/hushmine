//! grisu-world: la base del mundo grande de HUSHMINE.
//!
//! Este crate reúne las cuatro piezas de infraestructura con las que el plan
//! W2 (`docs/mundo/01-escala-y-motor.md`, secciones 3, 4 y 8, y el ADR
//! `docs/adr/0008-grisu-world.md`) pasa de los mapas fijos del modo Turno a
//! un mundo generado al explorar: chunks con paleta e ids de 16 bits, un
//! planificador de streaming con histéresis, una caché LRU determinista y el
//! guardado por regiones en archivos `.hreg`.
//!
//! Es un crate de **motor** (`grisu-*`): no dibuja, no suena y no conoce las
//! reglas del juego (la simulación vive en `hush-sim` y la generación en
//! `hush-worldgen`), y su única dependencia es `grisu-voxel`, del que toma
//! `Chunk`, `ChunkPos` y la geometría del chunk de 16³. Como el resto del
//! motor, todo el orden interno es total y determinista (`BTreeMap`,
//! `BTreeSet` y un reloj lógico): ni `HashMap` ni el reloj del sistema, para
//! que la misma secuencia de operaciones produzca siempre el mismo resultado
//! en cualquier plataforma.
//!
//! # Chunks con paleta
//!
//! [`PalettedChunk`] guarda un chunk de 16³ bloques con ids [`WideBlockId`]
//! (`u16`) en la representación más pequeña posible: uniforme (2 bytes)
//! cuando todo el chunk es un mismo bloque, paleta de 1, 2, 4 u 8 bits por
//! posición mientras haya 256 bloques distintos o menos, y los 4096 ids
//! directos (8 KB) cuando la paleta ya no cabe. [`PalettedChunk::set`]
//! promociona la representación al usarla, [`PalettedChunk::compact`] la
//! reduce al mínimo tras editar y [`PalettedChunk::to_chunk_lossy`] tiende el
//! puente al `u8` del motor actual (los ids mayores que 255 quedan como
//! aire). Los ids `0..=17` siguen siendo la Convención A del hito 1,
//! intactos.
//!
//! # Streaming con histéresis
//!
//! [`StreamScheduler`] decide qué chunks entran y salen alrededor de los
//! jugadores según los radios de [`DistancePreset`]: produce planes de carga
//! ordenados por prioridad (distancia, dirección de avance y posición) y
//! planes de descarga para lo que queda fuera. La descarga lleva histéresis
//! R+2 y V+2 ([`HYSTERESIS`] niveles extra sobre el radio horizontal y el
//! vertical): un chunk no sale del conjunto activo hasta sobrepasar la caja
//! de visión de todos los jugadores con margen, para no regenerarlo al ir y
//! volver. [`ChunkStage`] nombra las fases del cauce (Column, Base,
//! Decorated, Lit y Meshed) como vocabulario común de generación, luz y
//! mallado.
//!
//! # Caché LRU determinista
//!
//! [`BaseChunkCache`] guarda chunks en estado `Base` alrededor del borde
//! cargado, para que la fase de decoración no repita trabajo. Marca el uso
//! con un reloj lógico `u64` (nunca el reloj del sistema) y guarda las
//! entradas en un `BTreeMap`, de modo que la misma secuencia de operaciones
//! expulsa siempre a las mismas entradas, en cualquier plataforma y
//! ejecución.
//!
//! # Regiones `.hreg`
//!
//! El mundo se reparte en regiones de [`REGION_SIZE`]³ chunks
//! ([`REGION_CHUNKS`] en total), cada una en su archivo `r.{x}.{y}.{z}.hreg`
//! gestionado por [`RegionFile`]: encabezado con la magia `HREG`, la
//! posición de la región y `worldgen_version`, tabla de chunks modificados
//! (índice local, offset y longitud) y blobs codificados «paleta + RLE».
//! [`RegionPos`] convierte entre chunk y región con un desplazamiento
//! aritmético que también acierta con coordenadas negativas. Solo se guardan
//! los chunks modificados: el resto se regenera desde la semilla. La
//! escritura es atómica (`.tmp` y renombrado al final) y la lectura valida
//! todos los bytes y devuelve [`RegionError`] ante cualquier corrupción:
//! jamás entra en pánico.

mod lru;
mod palette;
mod region;
mod scheduler;

pub use lru::BaseChunkCache;
pub use palette::{PalettedChunk, WideBlockId};
pub use region::{REGION_CHUNKS, REGION_SHIFT, REGION_SIZE, RegionError, RegionFile, RegionPos};
pub use scheduler::{ChunkStage, DistancePreset, HYSTERESIS, StreamScheduler};

#[cfg(test)]
mod lru_tests;
#[cfg(test)]
mod palette_tests;
#[cfg(test)]
mod region_tests;
#[cfg(test)]
mod scheduler_tests;
