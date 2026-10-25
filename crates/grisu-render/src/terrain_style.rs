//! Ajustes de dibujo del terreno de cada fotograma.

use crate::CLEAR_COLOR;

/// Ajustes de dibujo del terreno que cambian cada fotograma.
#[derive(Clone, Copy, Debug)]
pub struct TerrainStyle {
    /// Distancia, en bloques, a la que la niebla tapa todo.
    pub fog_distance: f32,
    /// Bloque señalado por el jugador (coordenadas del mundo), que se resalta.
    pub selected: Option<[i32; 3]>,
    /// Alcance de la lámpara del casco, en bloques (0 = apagada).
    pub lamp_radius: f32,
    /// Segundos desde el arranque; anima los bloques con fotogramas (lava).
    pub time_seconds: f32,
    /// Color de la niebla y del fondo (rgb lineal); de noche, más oscuro.
    pub fog_color: [f32; 3],
    /// Multiplicador de la luz de cielo (0-1) según la hora del día. Solo
    /// cambia algo con mallas de luz doble ([`VoxelRenderer::update_dual`]).
    pub daylight: f32,
}

impl Default for TerrainStyle {
    fn default() -> Self {
        Self {
            fog_distance: 48.0,
            selected: None,
            lamp_radius: 14.0,
            time_seconds: 0.0,
            fog_color: [
                CLEAR_COLOR.r as f32,
                CLEAR_COLOR.g as f32,
                CLEAR_COLOR.b as f32,
            ],
            daylight: 1.0,
        }
    }
}
