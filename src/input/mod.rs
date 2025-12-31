//! # Input Manager
//!
//! Gerencia entrada de mouse e teclado.

use redpowder::input::{poll_keyboard, poll_mouse, KeyEvent, MouseState};
use redpowder::syscall::SysResult;

pub struct InputManager {
    pub mouse: MouseState,
    pub keyboard_buffer: [KeyEvent; 32],
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            mouse: MouseState::default(),
            keyboard_buffer: [KeyEvent::default(); 32],
        }
    }

    pub fn update(&mut self) -> SysResult<()> {
        // Atualizar Mouse
        if let Ok(state) = poll_mouse() {
            self.mouse = state;
        }

        // Atualizar Keyboard
        // Nota: O servidor pode processar eventos e enviar via IPC
        // Mas por enquanto vamos apenas drenar para não estourar o buffer do kernel
        poll_keyboard(&mut self.keyboard_buffer)?;

        Ok(())
    }
}
