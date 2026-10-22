use anyhow::{Context, Result};

/// Backends que se prueban por defecto (`WGPU_BACKEND` los sustituye).
///
/// En Windows se usa DX12 con GL de respaldo y no Vulkan: algunos drivers
/// (por ejemplo, Intel UHD 620 con el driver 31.0.101.2137) hacen que el
/// loader de Vulkan termine el proceso al crear la instancia.
pub fn default_backends() -> wgpu::Backends {
    if cfg!(windows) {
        wgpu::Backends::DX12 | wgpu::Backends::GL
    } else {
        wgpu::Backends::PRIMARY | wgpu::Backends::GL
    }
}

/// Instancia, adaptador, dispositivo y cola de wgpu.
pub struct Gpu {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Gpu {
    /// Crea un contexto sin superficie, para capturas y pruebas.
    ///
    /// Respeta las variables de entorno de wgpu (`WGPU_BACKEND`,
    /// `WGPU_ADAPTER_NAME`...), útiles en CI con Vulkan por software.
    pub async fn headless() -> Result<Self> {
        let desc = wgpu::InstanceDescriptor {
            backends: default_backends(),
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        };
        let instance = wgpu::Instance::new(desc.with_env());
        Self::with_instance(instance, None).await
    }

    /// Crea el contexto a partir de una instancia ya existente y, si se da,
    /// elige un adaptador compatible con la superficie.
    pub async fn with_instance(
        instance: wgpu::Instance,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: surface,
                ..Default::default()
            })
            .await
            .context("no se encontró ningún adaptador gráfico compatible")?;

        let info = adapter.get_info();
        log::info!("GPU: {} ({:?})", info.name, info.backend);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("grisu-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .context("no se pudo crear el dispositivo gráfico")?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }
}
