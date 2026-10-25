//! Paleta y métricas del panel del launcher: un solo sitio para que el
//! dibujo y la interacción usen los mismos colores y tamaños.

/// Color RGBA lineal.
pub type Color = [f32; 4];

pub const FONDO: Color = [0.02, 0.03, 0.04, 0.10];
pub const PANEL: Color = [0.05, 0.07, 0.09, 0.55];
pub const PANEL_CLARO: Color = [0.09, 0.12, 0.15, 0.96];
pub const FRANJA: Color = [0.03, 0.04, 0.06, 0.96];
pub const LATO_ANCHO: f32 = 190.0;
pub const MARGEN: f32 = 12.0;

pub const LATERAL: Color = [0.04, 0.06, 0.08, 0.92];
pub const LATERAL_BORDE: Color = [0.24, 0.27, 0.33, 1.0];
pub const BOTON: Color = [0.07, 0.09, 0.12, 0.95];
pub const BOTON_ACTIVO: Color = [0.12, 0.16, 0.20, 0.98];
pub const TEXTO: Color = [0.86, 0.86, 0.82, 1.0];
pub const TEXTO_SUAVE: Color = [0.55, 0.58, 0.60, 1.0];
pub const TEXTO_CALIDO: Color = [0.95, 0.88, 0.65, 1.0];
pub const LATON: Color = [0.85, 0.69, 0.32, 1.0];
pub const VERDE: Color = [0.45, 0.96, 0.58, 1.0];
pub const VERDE_RELLENO: Color = [0.16, 0.42, 0.22, 0.99];
pub const VERDE_OSCURO: Color = [0.10, 0.26, 0.14, 0.99];
pub const AZUL_BORDE: Color = [0.36, 0.48, 0.62, 1.0];
pub const AZUL_RELLENO: Color = [0.10, 0.15, 0.22, 0.97];
pub const ROJO: Color = [0.95, 0.45, 0.40, 1.0];
pub const VERDE_OK: Color = [0.45, 0.95, 0.60, 1.0];

/// Rectángulo y colores de un control ya resuelto para dibujar.
pub struct Estilo {
    pub fondo: Color,
    pub borde: Color,
    pub texto: Color,
}

/// Resuelve el aspecto de un control según selección y tipo.
pub const fn estilo_control(activo: bool, destacado: bool) -> Estilo {
    if destacado {
        Estilo {
            fondo: VERDE_RELLENO,
            borde: VERDE,
            texto: [0.85, 1.0, 0.88, 1.0],
        }
    } else if activo {
        Estilo {
            fondo: BOTON_ACTIVO,
            borde: LATON,
            texto: TEXTO_CALIDO,
        }
    } else {
        Estilo {
            fondo: BOTON,
            borde: [0.26, 0.30, 0.36, 1.0],
            texto: TEXTO,
        }
    }
}

/// Periodo del pulso, en segundos.
pub const PULSE_PERIOD: f32 = 1.6;

/// Duración del fundido de entrada de una página, en segundos.
pub const FADE_TIME: f32 = 0.2;

/// Progreso del fundido en `[0, 1]`: 0 recién cambiada de página, 1 asentada.
pub fn fade_alpha(page_fade: f32) -> f32 {
    (page_fade / FADE_TIME).clamp(0.0, 1.0)
}

/// Velo que tapa el contenido mientras entra la página.
pub fn fade_cover(page_fade: f32) -> Color {
    let a = (1.0 - fade_alpha(page_fade)) * 0.6;
    [FONDO[0], FONDO[1], FONDO[2], a]
}

/// Onda triangular 0→1→0 en `[0, 1]`, para brillos que respiran.
pub fn pulse(time: f32) -> f32 {
    if PULSE_PERIOD <= 0.0 {
        return 0.0;
    }
    let t = (time / PULSE_PERIOD).rem_euclid(1.0);
    if t < 0.5 { t * 2.0 } else { 2.0 - t * 2.0 }
}

/// Interpola línea recta entre dos colores.
pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulse_is_periodic_and_bounded() {
        assert!((pulse(0.0) - 0.0).abs() < 1e-5);
        assert!((pulse(PULSE_PERIOD) - pulse(0.0)).abs() < 1e-5);
        assert!((pulse(PULSE_PERIOD * 0.25) - 0.5).abs() < 1e-5);
        assert!((pulse(PULSE_PERIOD * 0.5) - 1.0).abs() < 1e-5);
        for step in 0..32 {
            let p = pulse(PULSE_PERIOD * step as f32 / 32.0);
            assert!((0.0..=1.0).contains(&p));
        }
    }

    #[test]
    fn lerp_color_interpolates_each_channel() {
        let mid = lerp_color([0.0, 0.0, 0.0, 1.0], [1.0, 2.0, 4.0, 1.0], 0.5);
        assert_eq!(mid, [0.5, 1.0, 2.0, 1.0]);
        assert_eq!(lerp_color([0.0; 4], [1.0; 4], 2.0), [1.0; 4]);
        assert_eq!(lerp_color([0.0; 4], [1.0; 4], -1.0), [0.0; 4]);
    }

    #[test]
    fn fade_cover_starts_opaque_and_vanishes() {
        assert_eq!(fade_cover(0.0)[3], 0.6);
        assert_eq!(fade_cover(FADE_TIME)[3], 0.0);
        assert_eq!(fade_cover(FADE_TIME * 0.5)[3], 0.3);
        assert_eq!(fade_cover(FADE_TIME * 10.0)[3], 0.0);
    }
}
