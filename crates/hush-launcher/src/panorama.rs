//! Panorama de fondo del launcher: el terreno del juego girando despacio
//! detrás del panel, con la lava animada por el tiempo acumulado.

use anyhow::Result;
use glam::Vec3;
use grisu_render::{DepthTarget, Gpu, OrbitCamera, TerrainStyle, VoxelRenderer};
use grisu_voxel::VoxelWorld;
use hush_art::{block_textures, emission_table};
use hush_sim::{TerrainConfig, generate_terrain};

/// Semilla fija del panorama para que el fondo sea siempre reconocible.
pub const PANORAMA_SEED: u64 = 42;

pub struct PanoramaRenderer {
    voxel: VoxelRenderer,
    angle: f32,
    time: f32,
}

impl PanoramaRenderer {
    /// Genera y malló el terreno una sola vez al arrancar.
    pub fn new(gpu: &Gpu, color_format: wgpu::TextureFormat, seed: u64) -> Result<Self> {
        let mut world: VoxelWorld = generate_terrain(seed, &TerrainConfig::default())?;
        let textures = block_textures()?;
        let mut voxel = VoxelRenderer::new(&gpu.device, &gpu.queue, color_format, &textures)?;
        voxel.enable_lighting(emission_table());
        voxel.update(&gpu.device, &mut world);
        Ok(Self {
            voxel,
            angle: 0.9,
            time: 0.0,
        })
    }

    /// Avanza el giro y el tiempo de la lava.
    pub fn advance(&mut self, dt: f32) {
        self.angle += dt * 0.05;
        self.time += dt;
    }

    /// Dibuja el panorama: borra el color y deja la profundidad lista.
    pub fn render(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        depth: &DepthTarget,
    ) {
        let aspect = depth.width.max(1) as f32 / depth.height.max(1) as f32;
        let view = panorama_camera(self.angle).view(aspect);
        let style = TerrainStyle {
            fog_distance: 320.0,
            time_seconds: self.time,
            ..TerrainStyle::default()
        };
        self.voxel
            .render(queue, encoder, color, depth, &view, &style);
    }
}

/// Cámara orbital del panorama, pura para poder probarla.
pub fn panorama_camera(angle: f32) -> OrbitCamera {
    OrbitCamera {
        target: Vec3::new(32.0, 28.0, 32.0),
        distance: 72.0,
        yaw: angle,
        pitch: 0.38,
        fov_y: 50f32.to_radians(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_orbits_without_changing_distance_or_height() {
        let a = panorama_camera(0.0);
        let b = panorama_camera(1.5);
        assert_eq!(a.distance, b.distance);
        assert_eq!(a.target, b.target);
        assert!((a.eye().y - b.eye().y).abs() < 1e-4);
        assert_ne!(a.eye().x, b.eye().x);
        // Giro lento: en un segundo cambia poco.
        let small = panorama_camera(0.05);
        let delta = (small.eye().x - a.eye().x).abs();
        assert!(delta < 5.0);
    }
}
