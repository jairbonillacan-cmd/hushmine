//! Vistas de depuración del terreno (docs/graficos/04-shaders.md, sección 12).
//!
//! Un [`DebugView`] cambia el fragment shader del terreno para enseñar un
//! canal interno —luz, oclusión ambiental, normales, límites de chunk— en
//! vez del dibujo normal. Con una sola captura se puede comprobar si la luz
//! se propaga bien o si el mallado encaja con los chunks.

/// Canal interno que muestra el terreno en vez del dibujo normal.
///
/// El valor viaja en `Globals::debug_view` (uniforme del grupo 0) y el
/// shader ramifica sobre él; los nombres son los previstos para el futuro
/// `hush-cli shot --debug` (docs/graficos/04-shaders.md, sección 12).
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DebugView {
    /// Dibujo normal: texturas, niebla, lámpara y resaltado.
    #[default]
    None = 0,
    /// Mapa de calor de la luz de bloque: negro → ámbar → blanco.
    Light = 1,
    /// Luz disponible en azul frío; usará la luz de cielo real cuando
    /// exista un canal propio.
    Sky = 2,
    /// Factor puro de oclusión ambiental de la esquina, en escala de grises.
    Ao = 3,
    /// Color RGB por dirección de la cara (vista de normales estándar).
    Normals = 4,
    /// Tinte damero por chunk para ver los límites de la malla.
    Chunks = 5,
}

impl DebugView {
    /// Valor que viaja en el uniforme del shader.
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    /// Nombre en minúscula, el mismo que acepta [`DebugView::from_name`].
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Light => "light",
            Self::Sky => "sky",
            Self::Ao => "ao",
            Self::Normals => "normals",
            Self::Chunks => "chunks",
        }
    }

    /// Acepta los nombres de [`DebugView::name`], sin importar mayúsculas.
    pub fn from_name(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_str() {
            "none" => Self::None,
            "light" => Self::Light,
            "sky" => Self::Sky,
            "ao" => Self::Ao,
            "normals" => Self::Normals,
            "chunks" => Self::Chunks,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::DebugView;

    #[test]
    fn names_round_trip() {
        for view in [
            DebugView::None,
            DebugView::Light,
            DebugView::Sky,
            DebugView::Ao,
            DebugView::Normals,
            DebugView::Chunks,
        ] {
            assert_eq!(DebugView::from_name(view.name()), Some(view));
        }
    }

    #[test]
    fn from_name_ignores_case_and_rejects_unknown() {
        assert_eq!(DebugView::from_name("LIGHT"), Some(DebugView::Light));
        assert_eq!(DebugView::from_name("Chunks"), Some(DebugView::Chunks));
        assert_eq!(DebugView::from_name("nada"), None);
        assert_eq!(DebugView::from_name(""), None);
    }
}
