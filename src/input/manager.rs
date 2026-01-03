//! # Input Manager
//!
//! Gerenciador centralizado de entrada (mouse, teclado).

use gfx_types::geometry::Point;
use redpowder::input::{KeyCode, MouseButton, MouseState};

// =============================================================================
// INPUT MANAGER
// =============================================================================

/// Gerenciador centralizado de entrada.
pub struct InputManager {
    /// Estado atual do mouse.
    pub mouse: MouseState,
    /// Posição do mouse.
    pub mouse_pos: Point,
    /// Última tecla pressionada.
    pub last_key: Option<(KeyCode, bool)>,
    /// Botões de mouse pressionados no frame anterior.
    pub prev_buttons: u8,
}

impl InputManager {
    /// Cria novo gerenciador.
    pub fn new() -> Self {
        Self {
            mouse: MouseState::default(),
            mouse_pos: Point::ZERO,
            last_key: None,
            prev_buttons: 0,
        }
    }

    /// Atualiza estado do mouse.
    pub fn update_mouse(&mut self, x: i32, y: i32, buttons: u8) {
        self.prev_buttons = self.mouse.buttons;
        self.mouse.x = x;
        self.mouse.y = y;
        self.mouse.buttons = buttons;
        self.mouse_pos = Point::new(x, y);
    }

    /// Atualiza estado do teclado.
    pub fn update_keyboard(&mut self, keycode: KeyCode, pressed: bool) {
        self.last_key = Some((keycode, pressed));
    }

    /// Atualiza a partir de um evento do serviço de input.
    pub fn update_from_service(
        &mut self,
        event_type: u32,
        key_code: u32,
        pressed: u32,
        x: i32,
        y: i32,
        buttons: u32,
    ) {
        match event_type {
            1 => {
                // Evento de teclado
                let code = KeyCode::from_scancode(key_code as u8);
                self.last_key = Some((code, pressed == 1));
            }
            2 => {
                // Evento de mouse
                self.update_mouse(x, y, buttons as u8);
            }
            _ => {}
        }
    }

    /// Verifica se botão foi pressionado neste frame.
    pub fn button_just_pressed(&self, button: MouseButton) -> bool {
        let mask = button.mask();
        (self.mouse.buttons & mask) != 0 && (self.prev_buttons & mask) == 0
    }

    /// Verifica se botão foi solto neste frame.
    pub fn button_just_released(&self, button: MouseButton) -> bool {
        let mask = button.mask();
        (self.mouse.buttons & mask) == 0 && (self.prev_buttons & mask) != 0
    }

    /// Verifica se botão está pressionado.
    pub fn button_pressed(&self, button: MouseButton) -> bool {
        self.mouse.is_pressed(button)
    }

    /// Retorna a última tecla pressionada e limpa.
    pub fn take_key(&mut self) -> Option<(KeyCode, bool)> {
        self.last_key.take()
    }

    /// Limpa estado da tecla.
    pub fn clear_key(&mut self) {
        self.last_key = None;
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
