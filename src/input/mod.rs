//! # Gerenciador de Entrada (Input Manager)
//!
//! Módulo responsável por manter o estado global do input.

use redpowder::event::event_type;
use redpowder::input::{KeyCode, KeyEvent, MouseState};

/// Gerenciador centralizado de entrada.
pub struct InputManager {
    /// Estado atual do mouse
    pub mouse: MouseState,
    /// Última tecla pressionada (estado simplificado)
    pub last_key: Option<(KeyCode, bool)>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            mouse: MouseState::default(),
            last_key: None,
        }
    }

    /// Atualiza o estado com dados vindos do serviço de input
    pub fn update_from_service(
        &mut self,
        event_type: u32,
        key_code: u32,
        pressed: u32,
        x: i32,
        y: i32,
        buttons: u32,
    ) {
        if event_type == 1 {
            // Key
            let code = unsafe { core::mem::transmute::<u8, KeyCode>(key_code as u8) };
            self.last_key = Some((code, pressed == 1));
        } else if event_type == 2 {
            // Mouse
            self.mouse.x = x;
            self.mouse.y = y;
            self.mouse.buttons = buttons as u8;
        }
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
