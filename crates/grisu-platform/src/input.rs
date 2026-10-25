use std::collections::HashSet;

pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;

/// Estado de teclado y ratón de un fotograma.
///
/// «Pulsado» significa que empezó a pulsarse en este fotograma; «mantenido»,
/// que sigue abajo.
#[derive(Debug, Default)]
pub struct Input {
    held: HashSet<KeyCode>,
    pressed: HashSet<KeyCode>,
    last_key_pressed: Option<KeyCode>,
    buttons_held: HashSet<MouseButton>,
    buttons_pressed: HashSet<MouseButton>,
    mouse_delta: (f32, f32),
    cursor_pos: (f32, f32),
    window_size: (u32, u32),
    cursor_grabbed: bool,
}

impl Input {
    pub fn is_held(&self, key: KeyCode) -> bool {
        self.held.contains(&key)
    }

    pub fn was_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    /// Última tecla pulsada en este fotograma (útil para reasignar controles).
    pub fn last_key_pressed(&self) -> Option<KeyCode> {
        self.last_key_pressed
    }

    pub fn is_button_held(&self, button: MouseButton) -> bool {
        self.buttons_held.contains(&button)
    }

    pub fn was_button_pressed(&self, button: MouseButton) -> bool {
        self.buttons_pressed.contains(&button)
    }

    /// Movimiento del ratón desde el fotograma anterior, en píxeles. Solo
    /// se acumula con el cursor capturado.
    pub fn mouse_delta(&self) -> (f32, f32) {
        self.mouse_delta
    }

    /// Posición actual del cursor dentro de la ventana, en píxeles físicos.
    pub fn cursor_pos(&self) -> (f32, f32) {
        self.cursor_pos
    }

    /// Tamaño actual del área cliente de la ventana, en píxeles físicos.
    pub fn window_size(&self) -> (u32, u32) {
        self.window_size
    }

    /// ¿Está el cursor capturado por el juego (modo de primera persona)?
    pub fn cursor_grabbed(&self) -> bool {
        self.cursor_grabbed
    }

    pub(crate) fn key(&mut self, key: KeyCode, down: bool) {
        if down {
            if self.held.insert(key) {
                self.pressed.insert(key);
                self.last_key_pressed = Some(key);
            }
        } else {
            self.held.remove(&key);
        }
    }

    pub(crate) fn button(&mut self, button: MouseButton, down: bool) {
        if down {
            if self.buttons_held.insert(button) {
                self.buttons_pressed.insert(button);
            }
        } else {
            self.buttons_held.remove(&button);
        }
    }

    pub(crate) fn motion(&mut self, dx: f64, dy: f64) {
        if self.cursor_grabbed {
            self.mouse_delta.0 += dx as f32;
            self.mouse_delta.1 += dy as f32;
        }
    }

    pub(crate) fn set_cursor_pos(&mut self, x: f32, y: f32) {
        self.cursor_pos = (x, y);
    }

    pub(crate) fn set_window_size(&mut self, width: u32, height: u32) {
        self.window_size = (width, height);
    }

    pub(crate) fn set_grabbed(&mut self, grabbed: bool) {
        self.cursor_grabbed = grabbed;
        if !grabbed {
            self.held.clear();
            self.buttons_held.clear();
        }
    }

    /// Olvida lo que solo vale para un fotograma.
    pub(crate) fn end_frame(&mut self) {
        self.pressed.clear();
        self.last_key_pressed = None;
        self.buttons_pressed.clear();
        self.mouse_delta = (0.0, 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressed_lasts_one_frame_and_held_until_release() {
        let mut input = Input::default();
        input.key(KeyCode::KeyW, true);
        assert!(input.was_pressed(KeyCode::KeyW) && input.is_held(KeyCode::KeyW));
        input.end_frame();
        // La repetición del teclado no vuelve a contar como pulsación.
        input.key(KeyCode::KeyW, true);
        assert!(!input.was_pressed(KeyCode::KeyW) && input.is_held(KeyCode::KeyW));
        input.key(KeyCode::KeyW, false);
        assert!(!input.is_held(KeyCode::KeyW));
    }

    #[test]
    fn mouse_motion_only_counts_when_grabbed() {
        let mut input = Input::default();
        input.motion(5.0, 3.0);
        assert_eq!(input.mouse_delta(), (0.0, 0.0));
        input.set_grabbed(true);
        input.motion(5.0, 3.0);
        input.motion(1.0, -1.0);
        assert_eq!(input.mouse_delta(), (6.0, 2.0));
        input.end_frame();
        assert_eq!(input.mouse_delta(), (0.0, 0.0));
    }
}
