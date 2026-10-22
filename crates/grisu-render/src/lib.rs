//! Grisú: renderizado con wgpu.
//!
//! El mismo código dibuja en una ventana (`grisu-platform`) o sin ventana
//! (`offscreen`), para que la IA pueda generar capturas desde la terminal.

mod camera;
mod cube;
mod depth;
mod gpu;
mod offscreen;
mod texture;

pub use camera::OrbitCamera;
pub use cube::CubeRenderer;
pub use depth::DepthTarget;
pub use gpu::{Gpu, default_backends};
pub use offscreen::render_to_image;
pub use texture::TextureData;

/// Color de fondo: oscuridad de mina, ligeramente azulada.
pub const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.02,
    g: 0.025,
    b: 0.035,
    a: 1.0,
};
