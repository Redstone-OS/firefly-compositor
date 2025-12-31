//! # Compositor de Cena
//!
//! Este módulo é responsável por compor todas as superfícies (janelas)
//! em um único buffer e apresentá-lo no framebuffer físico.
//!
//! ## Fluxo de Renderização
//!
//! 1. Limpar backbuffer com cor de fundo
//! 2. Ordenar superfícies por Z-order
//! 3. Blitar cada superfície no backbuffer
//! 4. Desenhar cursor (se houver)
//! 5. Apresentar backbuffer no framebuffer físico

use super::surface::Surface;
use crate::render::Backbuffer;
use alloc::vec::Vec;
use redpowder::graphics::Color;
use redpowder::ipc::ShmId;
use redpowder::syscall::SysResult;

// ============================================================================
// CONSTANTES
// ============================================================================

/// Cor de fundo padrão do desktop (cinza escuro)
const BACKGROUND_COLOR: Color = Color(0xFF222222);

// ============================================================================
// COMPOSITOR
// ============================================================================

/// Compositor gráfico principal.
///
/// Gerencia a lista de superfícies e coordena a renderização
/// de cada frame no framebuffer físico.
pub struct Compositor {
    /// Lista de superfícies (janelas) registradas
    surfaces: Vec<Surface>,

    /// Buffer em RAM para composição
    backbuffer: Backbuffer,

    /// Próximo ID de superfície a ser atribuído
    next_surface_id: u32,
}

impl Compositor {
    /// Cria um novo compositor.
    ///
    /// Inicializa o backbuffer com as dimensões do framebuffer físico
    /// e prepara para receber conexões de clientes.
    pub fn new() -> SysResult<Self> {
        // Criar backbuffer (obtém dimensões automaticamente do kernel)
        let backbuffer = Backbuffer::new()?;

        crate::println!(
            "[Compositor] Backbuffer criado: {}x{} (stride={})",
            backbuffer.width,
            backbuffer.height,
            backbuffer.stride
        );

        Ok(Self {
            surfaces: Vec::new(),
            backbuffer,
            next_surface_id: 1,
        })
    }

    /// Cria uma nova superfície (janela).
    ///
    /// # Parâmetros
    ///
    /// * `width` - Largura da superfície em pixels
    /// * `height` - Altura da superfície em pixels
    ///
    /// # Retorna
    ///
    /// ID da superfície (> 0) ou 0 em caso de erro.
    pub fn create_surface(&mut self, width: u32, height: u32) -> u32 {
        let id = self.next_surface_id;
        self.next_surface_id += 1;

        match Surface::new(id, width, height) {
            Ok(surface) => {
                self.surfaces.push(surface);
                id
            }
            Err(_) => {
                crate::println!("[Compositor] Erro ao criar superfície {}", id);
                0
            }
        }
    }

    /// Obtém o ID do SHM associado a uma superfície.
    ///
    /// # Parâmetros
    ///
    /// * `id` - ID da superfície
    ///
    /// # Retorna
    ///
    /// `ShmId` válido ou `ShmId(0)` se superfície não encontrada.
    pub fn get_surface_shm(&self, id: u32) -> ShmId {
        self.surfaces
            .iter()
            .find(|s| s.id == id)
            .map(|s| s.shm_id())
            .unwrap_or(ShmId(0))
    }

    /// Marca uma superfície como "dirty" (precisa re-blit).
    ///
    /// Chamado quando um cliente envia `COMMIT_BUFFER`.
    pub fn mark_damage(&mut self, id: u32) {
        if let Some(surface) = self.surfaces.iter_mut().find(|s| s.id == id) {
            surface.dirty = true;
        }
    }

    /// Renderiza um frame completo.
    ///
    /// Esta função executa o pipeline de renderização completo:
    /// 1. Limpa o backbuffer com a cor de fundo
    /// 2. Desenha cada superfície (ordenadas por Z-order)
    /// 3. Apresenta o resultado no framebuffer físico
    ///
    /// # Retorna
    ///
    /// `Ok(())` se a renderização foi bem-sucedida.
    pub fn render(&mut self) -> SysResult<()> {
        // 1. Limpar com cor de fundo
        self.backbuffer.clear(BACKGROUND_COLOR);

        // 2. Desenhar cada superfície
        // TODO: Ordenar por z_order antes de iterar
        for surface in &self.surfaces {
            // Usar função estática para evitar conflito de borrow
            Self::blit_surface(&mut self.backbuffer, surface);
        }

        // 3. TODO: Desenhar cursor do mouse

        // 4. Apresentar no framebuffer físico
        if !self.backbuffer.present() {
            crate::println!("[Compositor] ERRO: present() falhou!");
        }

        Ok(())
    }

    /// Copia os pixels de uma superfície para o backbuffer.
    ///
    /// # Parâmetros
    ///
    /// * `backbuffer` - Buffer de destino
    /// * `surface` - Superfície a ser desenhada
    ///
    /// Pixels com alpha = 0 são ignorados (transparentes).
    fn blit_surface(backbuffer: &mut crate::render::Backbuffer, surface: &Surface) {
        // Obter slice dos pixels do SHM
        let pixel_count = (surface.width * surface.height) as usize;
        let src_pixels =
            unsafe { core::slice::from_raw_parts(surface.shm.as_ptr() as *const u32, pixel_count) };

        // Copiar pixel a pixel (com verificação de alpha)
        for y in 0..surface.height {
            for x in 0..surface.width {
                let idx = (y * surface.width + x) as usize;

                if idx >= src_pixels.len() {
                    continue;
                }

                let color = src_pixels[idx];
                let alpha = color >> 24;

                // Ignora pixels totalmente transparentes
                if alpha == 0 {
                    continue;
                }

                // TODO: Blending para alpha parcial
                let dest_x = surface.x + x as i32;
                let dest_y = surface.y + y as i32;

                backbuffer.put_pixel(dest_x, dest_y, Color(color));
            }
        }
    }
}
