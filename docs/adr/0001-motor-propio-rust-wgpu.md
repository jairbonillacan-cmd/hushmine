# ADR 0001: motor propio en Rust + wgpu

- **Fecha:** 22 de octubre de 2026
- **Estado:** aceptada

## Contexto

HUSHMINE lo desarrolla una sola persona con ayuda de IA y prioriza el rendimiento, el multiplataforma (escritorio, Steam Deck y navegador) y la posibilidad de crecer a largo plazo. Se evaluaron Unity, Godot, MonoGame/FNA, Bevy, C++ con SDL3 y C# con SDL3.

## Decisión

Motor propio (Grisú) en Rust 2024 con:

- **wgpu 30** (Vulkan, Metal, DX12, GL y WebGPU);
- **winit 0.30**, la última estable, porque 0.31 está en beta;
- **glam** para matemáticas.

El motor vive en crates `grisu-*` dentro del repositorio del juego y se publicará por separado cuando el juego esté en acceso anticipado.

## Consecuencias

- **Ventajas:** rendimiento nativo, ejecución en el navegador con WebGPU y un compilador estricto que ayuda a la IA.
- **Costes:** compilación más lenta que C# y cambios de API frecuentes en wgpu. Se fija la versión y se actualiza una vez por hito.
- **Consolas:** wgpu no tiene backend de consola. Si algún día son prioridad, habrá que reevaluar.
- **Windows:** DX12 es el backend predeterminado por el fallo de Vulkan con algunos drivers Intel (ver `AGENTS.md`).
