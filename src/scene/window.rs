//! # Scene - Window
//!
//! Representa uma janela gerenciada pelo compositor.

use alloc::string::String;
use gfx_types::color::Color;
use gfx_types::geometry::{Point, Rect, Size};
use gfx_types::window::{LayerType, WindowFlags, WindowState};
use redpowder::ipc::SharedMemory;

// =============================================================================
// WINDOW ID
// =============================================================================

/// ID único de janela.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(pub u32);

impl WindowId {
    pub const INVALID: Self = Self(0);

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

// =============================================================================
// WINDOW
// =============================================================================

/// Janela gerenciada pelo compositor.
pub struct Window {
    /// ID único.
    pub id: WindowId,
    /// Posição no desktop.
    pub position: Point,
    /// Tamanho da janela.
    pub size: Size,
    /// Memória compartilhada com o cliente.
    pub shm: SharedMemory,
    /// Flags de comportamento.
    pub flags: WindowFlags,
    /// Estado atual da janela.
    pub state: WindowState,
    /// Camada da janela.
    pub layer: LayerType,
    /// Janela precisa ser redesenhada.
    pub dirty: bool,
    /// Indica se a janela já recebeu conteúdo (pelo menos um commit).
    pub has_content: bool,
    /// Título da janela.
    pub title: String,
    /// Retângulo anterior (para restauração).
    pub restore_rect: Option<Rect>,
    /// Z-order dentro da camada (maior = mais na frente).
    pub z_order: u32,
    /// Opacidade global (0-255).
    pub opacity: u8,
    /// Cor de borda (se aplicável).
    pub border_color: Color,
}

impl Window {
    /// Cria nova janela.
    pub fn new(id: u32, size: Size, shm: SharedMemory) -> Self {
        Self {
            id: WindowId(id),
            position: Point::ZERO,
            size,
            shm,
            flags: WindowFlags::NONE,
            state: WindowState::Normal,
            layer: LayerType::Normal,
            dirty: true,
            has_content: false,
            title: String::new(),
            restore_rect: None,
            z_order: 0,
            opacity: 255,
            border_color: Color::TRANSPARENT,
        }
    }

    // =========================================================================
    // PROPRIEDADES
    // =========================================================================

    /// Retorna o retângulo da janela.
    #[inline]
    pub fn rect(&self) -> Rect {
        Rect::new(
            self.position.x,
            self.position.y,
            self.size.width,
            self.size.height,
        )
    }

    /// Retorna se a janela está visível.
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.state != WindowState::Minimized && self.has_content
    }

    /// Retorna se a janela é transparente.
    #[inline]
    pub fn is_transparent(&self) -> bool {
        self.flags.has(WindowFlags::TRANSPARENT) || self.opacity < 255
    }

    /// Retorna se a janela tem decorações.
    #[inline]
    pub fn has_decorations(&self) -> bool {
        !self.flags.has(WindowFlags::BORDERLESS)
    }

    /// Retorna se a janela tem sombra.
    #[inline]
    pub fn has_shadow(&self) -> bool {
        self.flags.has(WindowFlags::HAS_SHADOW)
    }

    // =========================================================================
    // MODIFICAÇÕES
    // =========================================================================

    /// Altera layer da janela.
    pub fn set_layer(&mut self, layer: LayerType) {
        self.layer = layer;
        self.dirty = true;
    }

    /// Marca que a janela recebeu conteúdo.
    pub fn set_has_content(&mut self) {
        self.has_content = true;
        self.dirty = true;
    }

    /// Move a janela para uma nova posição.
    #[inline]
    pub fn move_to(&mut self, x: i32, y: i32) {
        self.position = Point::new(x, y);
        self.dirty = true;
    }

    /// Move a janela por um delta.
    #[inline]
    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.position.x += dx;
        self.position.y += dy;
        self.dirty = true;
    }

    /// Redimensiona a janela.
    #[inline]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = Size::new(width, height);
        self.dirty = true;
    }

    /// Define o estado da janela.
    pub fn set_state(&mut self, state: WindowState) {
        if state == WindowState::Maximized && self.state == WindowState::Normal {
            self.restore_rect = Some(self.rect());
        }
        self.state = state;
        self.dirty = true;
    }

    /// Minimiza a janela.
    pub fn minimize(&mut self) {
        if self.state != WindowState::Minimized {
            self.restore_rect = Some(self.rect());
            self.state = WindowState::Minimized;
            self.dirty = true;
        }
    }

    /// Restaura a janela.
    pub fn restore(&mut self) {
        if let Some(rect) = self.restore_rect.take() {
            self.position = Point::new(rect.x, rect.y);
            self.size = Size::new(rect.width, rect.height);
        }
        self.state = WindowState::Normal;
        self.dirty = true;
    }

    /// Maximiza a janela.
    pub fn maximize(&mut self, screen_size: Size) {
        if self.state != WindowState::Maximized {
            self.restore_rect = Some(self.rect());
            self.position = Point::ZERO;
            self.size = screen_size;
            self.state = WindowState::Maximized;
            self.dirty = true;
        }
    }

    // =========================================================================
    // ACESSO AOS PIXELS
    // =========================================================================

    /// Retorna pixels da janela como slice (acesso direto à SHM).
    ///
    /// # Safety
    /// O caller deve estar ciente de que o conteúdo pode ser alterado pelo cliente
    /// concorrentemente. No entanto, para composição, um blit sequencial é aceitável.
    pub fn pixels(&self) -> &[u32] {
        let count = (self.size.width * self.size.height) as usize;
        let src_ptr = self.shm.as_ptr() as *const u32;
        unsafe { core::slice::from_raw_parts(src_ptr, count) }
    }

    /// Verifica se um ponto está dentro da janela.
    #[inline]
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        self.rect().contains_point(Point::new(x, y))
    }

    /// Converte coordenadas globais para locais da janela.
    #[inline]
    pub fn to_local(&self, x: i32, y: i32) -> Point {
        Point::new(x - self.position.x, y - self.position.y)
    }
}
