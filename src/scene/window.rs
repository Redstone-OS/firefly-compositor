//! # Window
//!
//! Representa uma janela no compositor.

use gfx_types::{BufferHandle, Point, Rect, Size, WindowFlags};
use redpowder::ipc::SharedMemory;

/// ID único de janela.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId(pub u32);

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
    /// Janela está visível.
    pub visible: bool,
    /// Janela precisa ser redesenhada.
    pub dirty: bool,
    /// Camada da janela.
    pub layer: gfx_types::LayerType,
    /// Indica se a janela já recebeu conteúdo (pelo menos um commit).
    /// Janelas sem conteúdo não são renderizadas.
    pub has_content: bool,
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
            visible: true,
            dirty: true,
            layer: gfx_types::LayerType::Normal,
            has_content: false,
        }
    }

    /// Altera layer da janela.
    pub fn set_layer(&mut self, layer: gfx_types::LayerType) {
        self.layer = layer;
    }

    /// Marca que a janela recebeu conteúdo.
    pub fn set_has_content(&mut self) {
        self.has_content = true;
    }

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

    /// Move a janela para uma nova posição.
    #[inline]
    pub fn move_to(&mut self, x: i32, y: i32) {
        self.position = Point::new(x, y);
        self.dirty = true;
    }

    /// Redimensiona a janela.
    #[inline]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = Size::new(width, height);
        self.dirty = true;
    }

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
}
