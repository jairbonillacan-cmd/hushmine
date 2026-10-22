//! Cliente de HUSHMINE. En el hito 0 muestra un cubo de mineral girando.

use grisu_platform::App;
use grisu_render::{CubeRenderer, DepthTarget, Gpu, OrbitCamera, TextureData};

/// Velocidad de giro del cubo de demostración, en radianes por segundo.
const SPIN_SPEED: f32 = 0.6;

#[derive(Default)]
struct CubeDemo {
    cube: Option<CubeRenderer>,
    camera: OrbitCamera,
}

impl App for CubeDemo {
    fn init(&mut self, gpu: &Gpu, surface_format: wgpu::TextureFormat) {
        self.cube = Some(CubeRenderer::new(
            &gpu.device,
            &gpu.queue,
            surface_format,
            &TextureData::test_ore(),
        ));
    }

    fn update(&mut self, dt: f32) {
        self.camera.yaw += SPIN_SPEED * dt;
    }

    fn render(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        depth: &DepthTarget,
    ) {
        if let Some(cube) = &self.cube {
            cube.render(&gpu.queue, encoder, color, depth, &self.camera);
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    grisu_platform::run("HUSHMINE", CubeDemo::default())
}
