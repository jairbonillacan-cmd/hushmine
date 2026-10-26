//! Grisú: renderizado con wgpu.
//!
//! El mismo código dibuja en una ventana (`grisu-platform`) o sin ventana
//! (`offscreen`), para que la IA pueda generar capturas desde la terminal.

mod boxes;
mod camera;
mod chunk_origins;
mod crosshair;
mod cube;
mod debug_view;
mod depth;
mod gpu;
mod offscreen;
mod shader_prep;
mod terrain_style;
mod texture;
mod texture_array;
mod ui;
mod ui_fit;
mod ui_font;
mod ui_font_table;
mod voxel;

pub use boxes::{BoxRenderer, ColorBox};
pub use camera::{CameraMode, FlyCamera, OrbitCamera, View};
pub use crosshair::CrosshairRenderer;
pub use cube::CubeRenderer;
pub use debug_view::DebugView;
pub use depth::DepthTarget;
pub use gpu::{Gpu, default_backends};
pub use offscreen::{OFFSCREEN_FORMAT, capture, render_to_image};
pub use shader_prep::{
    QualityPreset, ShaderConfig, ShaderPrepError, preprocess_wgsl, validate_all_shader_variants,
};
pub use terrain_style::TerrainStyle;
pub use texture::TextureData;
pub use texture_array::upload_texture_array;
pub use ui::{UiBatch, UiRect, UiRenderer};
pub use ui_fit::{fit_text, wrap_text};
pub use voxel::{MeshStats, VoxelRenderer};

/// Color de fondo: oscuridad de mina, ligeramente azulada.
pub const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.02,
    g: 0.025,
    b: 0.035,
    a: 1.0,
};
