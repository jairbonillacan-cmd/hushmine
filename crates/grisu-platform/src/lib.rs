//! Grisú: ventana y bucle principal.
//!
//! El juego implementa [`App`] y no toca winit ni la superficie de wgpu.

use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use grisu_render::{DepthTarget, Gpu};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

/// Lo que el juego aporta al bucle principal.
pub trait App: 'static {
    /// Se llama una vez, cuando ya existen la GPU y la superficie.
    fn init(&mut self, gpu: &Gpu, surface_format: wgpu::TextureFormat);
    /// Avanza la lógica de presentación `dt` segundos.
    fn update(&mut self, dt: f32);
    /// Graba el dibujo del fotograma.
    fn render(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        depth: &DepthTarget,
    );
}

/// Abre una ventana y ejecuta `app` hasta que se cierre.
pub fn run(title: &str, app: impl App) -> Result<()> {
    let event_loop = EventLoop::new().context("no se pudo crear el bucle de eventos")?;
    let mut runner = Runner {
        title: title.to_owned(),
        app,
        state: None,
        error: None,
    };
    event_loop.run_app(&mut runner)?;
    match runner.error {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

struct Runner<A: App> {
    title: String,
    app: A,
    state: Option<WindowState>,
    error: Option<anyhow::Error>,
}

struct WindowState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    gpu: Gpu,
    depth: DepthTarget,
    last_frame: Instant,
}

impl WindowState {
    fn new(event_loop: &ActiveEventLoop, title: &str) -> Result<Self> {
        let attributes = Window::default_attributes()
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
        let window = Arc::new(event_loop.create_window(attributes)?);

        let display = Box::new(event_loop.owned_display_handle());
        let desc = wgpu::InstanceDescriptor {
            backends: grisu_render::default_backends(),
            ..wgpu::InstanceDescriptor::new_with_display_handle(display)
        };
        let instance = wgpu::Instance::new(desc.with_env());
        let surface = instance.create_surface(window.clone())?;
        let gpu = pollster::block_on(Gpu::with_instance(instance, Some(&surface)))?;

        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&gpu.adapter, size.width.max(1), size.height.max(1))
            .context("la superficie no es compatible con el adaptador")?;
        // Preferimos un formato sRGB para que los colores coincidan con las capturas.
        let caps = surface.get_capabilities(&gpu.adapter);
        if let Some(srgb) = caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
        {
            config.format = srgb;
        }
        surface.configure(&gpu.device, &config);
        let depth = DepthTarget::new(&gpu.device, config.width, config.height);

        Ok(Self {
            window,
            surface,
            config,
            gpu,
            depth,
            last_frame: Instant::now(),
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.gpu.device, &self.config);
        self.depth = DepthTarget::new(&self.gpu.device, width, height);
    }

    fn frame(&mut self, app: &mut impl App) {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32().min(0.25);
        self.last_frame = now;
        app.update(dt);

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.gpu.device, &self.config);
                return;
            }
            other => {
                log::debug!("fotograma omitido: {other:?}");
                return;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        app.render(&self.gpu, &mut encoder, &view, &self.depth);
        self.gpu.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.gpu.queue.present(frame);
    }
}

impl<A: App> ApplicationHandler for Runner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        match WindowState::new(event_loop, &self.title) {
            Ok(state) => {
                self.app.init(&state.gpu, state.config.format);
                state.window.request_redraw();
                self.state = Some(state);
            }
            Err(err) => {
                self.error = Some(err);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.frame(&mut self.app);
                state.window.request_redraw();
            }
            _ => {}
        }
    }
}
