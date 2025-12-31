//! # Gerenciador de Entrada (Input Manager)
//!
//! Módulo responsável por capturar e processar eventos de mouse e teclado
//! do kernel e disponibilizá-los para o compositor.
//!
//! ## Funcionamento
//!
//! O InputManager faz polling periódico das syscalls de input do kernel:
//! - `poll_mouse()` - Retorna o estado atual do mouse (posição, botões)
//! - `poll_keyboard()` - Retorna eventos de tecla pendentes
//!
//! ## Uso
//!
//! ```rust
//! let mut input = InputManager::new();
//!
//! // No loop principal:
//! input.update(); // Ignora erros (graceful degradation)
//! let mouse_pos = input.mouse_position();
//! ```

use redpowder::input::{poll_keyboard, poll_mouse, KeyEvent, MouseState};
use redpowder::syscall::SysResult;

// ============================================================================
// CONSTANTES
// ============================================================================

/// Tamanho máximo do buffer de eventos de teclado
const KEYBOARD_BUFFER_SIZE: usize = 32;

// ============================================================================
// INPUT MANAGER
// ============================================================================

/// Gerenciador centralizado de entrada.
///
/// Mantém o estado atual do mouse e buffer de eventos de teclado.
pub struct InputManager {
    /// Estado atual do mouse (posição, botões)
    mouse: MouseState,

    /// Buffer circular de eventos de teclado
    keyboard_buffer: [KeyEvent; KEYBOARD_BUFFER_SIZE],

    /// Quantidade de eventos no buffer
    keyboard_count: usize,
}

impl InputManager {
    /// Cria um novo gerenciador de entrada.
    pub fn new() -> Self {
        Self {
            mouse: MouseState::default(),
            keyboard_buffer: [KeyEvent::default(); KEYBOARD_BUFFER_SIZE],
            keyboard_count: 0,
        }
    }

    /// Atualiza o estado de entrada consultando o kernel.
    ///
    /// Esta função deve ser chamada uma vez por frame no loop principal.
    /// Erros são silenciosamente ignorados para permitir graceful degradation
    /// quando os drivers de input não estão disponíveis.
    ///
    /// # Retorna
    ///
    /// `Ok(())` sempre (erros são absorvidos internamente).
    pub fn update(&mut self) -> SysResult<()> {
        // Atualizar estado do mouse
        // Nota: Pode falhar se o driver de mouse não estiver disponível
        if let Ok(state) = poll_mouse() {
            self.mouse = state;
        }

        // Capturar eventos de teclado
        // Drena o buffer do kernel para evitar overflow
        if let Ok(count) = poll_keyboard(&mut self.keyboard_buffer) {
            self.keyboard_count = count;
        }

        Ok(())
    }

    /// Retorna a posição atual do mouse.
    #[inline]
    pub fn mouse_position(&self) -> (i32, i32) {
        (self.mouse.x, self.mouse.y)
    }

    /// Verifica se o botão esquerdo do mouse está pressionado.
    #[inline]
    pub fn is_left_button_pressed(&self) -> bool {
        self.mouse.buttons & 0x01 != 0
    }

    /// Verifica se o botão direito do mouse está pressionado.
    #[inline]
    pub fn is_right_button_pressed(&self) -> bool {
        self.mouse.buttons & 0x02 != 0
    }

    /// Retorna os eventos de teclado pendentes.
    pub fn keyboard_events(&self) -> &[KeyEvent] {
        &self.keyboard_buffer[..self.keyboard_count]
    }

    /// Limpa o buffer de eventos de teclado.
    pub fn clear_keyboard_events(&mut self) {
        self.keyboard_count = 0;
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
