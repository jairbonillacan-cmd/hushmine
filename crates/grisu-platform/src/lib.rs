//! Grisú: ventana y bucle principal.
//!
//! El juego implementa [`App`] y no toca winit ni la superficie de wgpu.
//!
//! Un clic izquierdo captura el cursor (primera persona); Escape lo suelta
//! y, si ya estaba suelto, cierra la ventana.

mod input;

pub use input::{Input, KeyCode, MouseButton};

use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use grisu_render::{DepthTarget, Gpu};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{CursorGrabMode, Window, WindowId};

/// Lo que el juego aporta al bucle principal.
pub trait App: 'static {
    /// Se llama una vez, cuando ya existen la GPU y la superficie.
    fn init(&mut self, gpu: &Gpu, surface_format: wgpu::TextureFormat);
    /// Avanza la lógica de presentación `dt` segundos.
    fn update(&mut self, dt: f32, input: &Input);
    /// Graba el dibujo del fotograma.
    fn render(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        depth: &DepthTarget,
    );
    /// ¿Debe capturarse el cursor al hacer clic en la ventana?
    /// Devuelve `false` cuando hay un menú interactivo abierto.
    fn wants_cursor_grab(&self) -> bool {
        true
    }
    /// ¿Ha pedido el juego cerrar la ventana (por ejemplo, desde el menú)?
    fn should_exit(&self) -> bool {
        false
    }
}

/// Abre una ventana de 1280×720 y ejecuta `app` hasta que se cierre.
pub fn run(title: &str, app: impl App) -> Result<()> {
    run_with_size(title, [1280, 720], app)
}

/// Abre una ventana con el tamaño lógico indicado y ejecuta `app`.
pub fn run_with_size(title: &str, initial_size: [u32; 2], app: impl App) -> Result<()> {
    let event_loop = EventLoop::new().context("no se pudo crear el bucle de eventos")?;
    let mut runner = Runner {
        title: title.to_owned(),
        initial_size,
        app,
        state: None,
        input: Input::default(),
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
    initial_size: [u32; 2],
    app: A,
    state: Option<WindowState>,
    input: Input,
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
    fn new(event_loop: &ActiveEventLoop, title: &str, [w, h]: [u32; 2]) -> Result<Self> {
        let attributes = Window::default_attributes()
            .with_title(title)
            .with_visible(true)
            .with_inner_size(winit::dpi::LogicalSize::new(f64::from(w), f64::from(h)));
        let window = Arc::new(event_loop.create_window(attributes)?);
        window.set_visible(true);
        window.focus_window();

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

    /// Captura o suelta el cursor. Devuelve si quedó capturado.
    fn grab_cursor(&self, grab: bool) -> bool {
        if !grab {
            let _ = self.window.set_cursor_grab(CursorGrabMode::None);
            self.window.set_cursor_visible(true);
            return false;
        }
        // `Locked` no existe en Windows y `Confined` no existe en macOS.
        let grabbed = self
            .window
            .set_cursor_grab(CursorGrabMode::Locked)
            .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined))
            .is_ok();
        if grabbed {
            self.window.set_cursor_visible(false);
        } else {
            log::warn!("no se pudo capturar el cursor");
        }
        grabbed
    }

    fn frame(&mut self, app: &mut impl App, input: &Input) {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32().min(0.25);
        self.last_frame = now;
        app.update(dt, input);

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
        match WindowState::new(event_loop, &self.title, self.initial_size) {
            Ok(state) => {
                self.input
                    .set_window_size(state.config.width, state.config.height);
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
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                let down = key_state == ElementState::Pressed;
                self.input.key(code, down);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input
                    .set_cursor_pos(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => {
                let down = button_state == ElementState::Pressed;
                if down && !self.input.cursor_grabbed() && self.app.wants_cursor_grab() {
                    // El clic que captura el cursor no llega al juego.
                    let grabbed = state.grab_cursor(true);
                    self.input.set_grabbed(grabbed);
                } else {
                    self.input.button(button, down);
                }
            }
            WindowEvent::Focused(false) => {
                let grabbed = state.grab_cursor(false);
                self.input.set_grabbed(grabbed);
            }
            WindowEvent::Resized(size) => {
                state.resize(size.width, size.height);
                self.input.set_window_size(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                state.frame(&mut self.app, &self.input);
                if !self.app.wants_cursor_grab() && self.input.cursor_grabbed() {
                    let grabbed = state.grab_cursor(false);
                    self.input.set_grabbed(grabbed);
                }
                self.input.end_frame();
                if self.app.should_exit() {
                    event_loop.exit();
                } else {
                    state.window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.input.motion(delta.0, delta.1);
        }
    }
}
