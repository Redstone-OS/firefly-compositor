//! # Surface (Superfície Gráfica)
//!
//! Representa uma janela ou superfície gráfica gerenciada pelo compositor.
//! Cada superfície possui um buffer de pixels em memória compartilhada
//! que é preenchido pelo cliente e lido pelo compositor durante a composição.
//!
//! ## Modelo de Memória
//!
//! O compositor (servidor) aloca a memória compartilhada e fornece o handle
//! ao cliente. O cliente mapeia essa memória e escreve pixels nela.
//! Durante a renderização, o compositor lê esses pixels e copia para
//! o backbuffer na posição correta.

use redpowder::ipc::{SharedMemory, ShmId};
use redpowder::syscall::SysResult;

// ============================================================================
// SURFACE
// ============================================================================

/// Representa uma superfície gráfica (janela) no compositor.
///
/// Cada superfície possui:
/// - Posição (x, y) no desktop
/// - Dimensões (width, height) em pixels
/// - Buffer de pixels em memória compartilhada
/// - Ordem Z para composição (maior = mais na frente)
pub struct Surface {
    /// Identificador único da superfície
    pub id: u32,

    /// Posição X no desktop (pode ser negativa para janelas fora da tela)
    pub x: i32,

    /// Posição Y no desktop
    pub y: i32,

    /// Largura em pixels
    pub width: u32,

    /// Altura em pixels
    pub height: u32,

    /// Memória compartilhada com o cliente
    pub shm: SharedMemory,

    /// Ordem de empilhamento (maior = mais na frente)
    pub z_order: u32,

    /// Flag indicando que a superfície foi modificada pelo cliente
    pub dirty: bool,

    /// Flag indicando se a superfície está visível
    pub visible: bool,
}

impl Surface {
    /// Cria uma nova superfície.
    ///
    /// # Parâmetros
    ///
    /// * `id` - Identificador único
    /// * `width` - Largura em pixels
    /// * `height` - Altura em pixels
    ///
    /// # Retorna
    ///
    /// `Ok(Surface)` com memória compartilhada alocada, ou `Err` em caso de falha.
    pub fn new(id: u32, width: u32, height: u32) -> SysResult<Self> {
        // Calcular tamanho do buffer (4 bytes por pixel - ARGB)
        let buffer_size = (width * height * 4) as usize;

        // Alocar memória compartilhada
        let shm = SharedMemory::create(buffer_size)?;

        Ok(Self {
            id,
            x: 0,
            y: 0,
            width,
            height,
            shm,
            z_order: 0,
            dirty: true,
            visible: true,
        })
    }

    /// Retorna o ID da memória compartilhada.
    ///
    /// Este ID é enviado ao cliente para que ele possa mapear
    /// a memória e escrever pixels.
    #[inline]
    pub fn shm_id(&self) -> ShmId {
        self.shm.id()
    }

    /// Move a superfície para uma nova posição.
    #[inline]
    pub fn move_to(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    /// Define a ordem Z da superfície.
    #[inline]
    pub fn set_z_order(&mut self, z: u32) {
        self.z_order = z;
    }

    /// Marca a superfície como visível ou invisível.
    #[inline]
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}
