//! # Render Engine
//!
//! Motor de composição principal.

use super::blitter::Blitter;
use crate::scene::{DamageTracker, Layer, LayerManager, Window, WindowId};
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use gfx_types::{Color, DisplayInfo, Point, Rect, Size};
use redpowder::graphics::write_framebuffer;
use redpowder::ipc::SharedMemory;
use redpowder::syscall::SysResult;

/// Cor de fundo padrão.
// Cor de fundo: azul escuro para diferenciar das janelas
const BACKGROUND_COLOR: Color = Color(0xFF1a1a2e);

/// Motor de renderização.
pub struct RenderEngine {
    /// Informações do display.
    display_info: DisplayInfo,
    /// Backbuffer em RAM.
    backbuffer: Vec<u32>,
    /// Gerenciador de camadas.
    layers: LayerManager,
    /// Janelas registradas.
    windows: BTreeMap<u32, Window>,
    /// Tracker de damage.
    damage: DamageTracker,
    /// Próximo ID de janela.
    next_window_id: u32,
    /// Contador de frames.
    frame_count: u64,
}

impl RenderEngine {
    /// Cria novo motor de renderização.
    pub fn new(display_info: DisplayInfo) -> Self {
        let size = (display_info.width * display_info.height) as usize;
        let backbuffer = vec![BACKGROUND_COLOR.as_u32(); size];

        crate::println!(
            "[Render] Backbuffer criado: {}x{} ({} bytes)",
            display_info.width,
            display_info.height,
            size * 4
        );

        Self {
            display_info,
            backbuffer,
            layers: LayerManager::new(),
            windows: BTreeMap::new(),
            damage: DamageTracker::new(),
            next_window_id: 1,
            frame_count: 0,
        }
    }

    /// Retorna tamanho do display.
    pub fn size(&self) -> Size {
        Size::new(self.display_info.width, self.display_info.height)
    }

    /// Cria nova janela.
    pub fn create_window(&mut self, size: Size, shm: SharedMemory) -> u32 {
        let id = self.next_window_id;
        self.next_window_id += 1;

        let window = Window::new(id, size, shm);
        self.windows.insert(id, window);
        self.layers
            .add_window_to_layer(WindowId(id), gfx_types::LayerType::Normal);

        // Marcar área da janela como danificada
        self.damage.add(Rect::new(0, 0, size.width, size.height));

        crate::println!(
            "[Render] Janela {} criada ({}x{})",
            id,
            size.width,
            size.height
        );

        id
    }

    /// Obtém janela por ID.
    pub fn get_window(&self, id: u32) -> Option<&Window> {
        self.windows.get(&id)
    }

    /// Obtém janela mutável por ID.
    pub fn get_window_mut(&mut self, id: u32) -> Option<&mut Window> {
        self.windows.get_mut(&id)
    }

    /// Move janela para nova posição.
    pub fn move_window(&mut self, id: u32, x: i32, y: i32) {
        if let Some(window) = self.windows.get_mut(&id) {
            // Marcar posição antiga como danificada
            self.damage.add(window.rect());

            window.move_to(x, y);

            // Marcar nova posição como danificada
            self.damage.add(window.rect());
        }
    }

    /// Marca janela como modificada.
    pub fn mark_damage(&mut self, id: u32) {
        if let Some(window) = self.windows.get(&id) {
            self.damage.add(window.rect());
        }
    }

    /// Destrói janela.
    pub fn destroy_window(&mut self, id: u32) {
        if let Some(window) = self.windows.remove(&id) {
            self.damage.add(window.rect());
            self.layers.remove_window(WindowId(id));
            crate::println!("[Render] Janela {} destruída", id);
        }
    }

    /// Renderiza um frame completo.
    pub fn render(&mut self) -> SysResult<()> {
        self.frame_count += 1;

        if self.frame_count == 1 {
            crate::println!("[Render] Primeiro frame!");
        }

        // Log window count every 300 frames (~5 seconds)
        if self.frame_count % 300 == 0 {
            crate::println!(
                "[Render] Frame {}, {} janelas",
                self.frame_count,
                self.windows.len()
            );
        }

        // 1. Limpar backbuffer com cor de fundo
        let size = self.size();
        Blitter::fill_rect(
            &mut self.backbuffer,
            size,
            Rect::from_size(size),
            BACKGROUND_COLOR,
        );

        // TESTE: Desenhar retângulo vermelho para confirmar que o backbuffer funciona
        let test_rect = Rect::new(50, 50, 200, 100);
        Blitter::fill_rect(
            &mut self.backbuffer,
            size,
            test_rect,
            Color(0xFFFF0000), // Vermelho
        );

        // 2. Coletar IDs de TODAS as janelas (bypass layer system for now)
        let windows_to_render: Vec<u32> = self.windows.keys().copied().collect();

        // 3. Compor janelas
        for window_id in windows_to_render {
            self.composite_window_by_id(window_id);
        }

        // 4. Apresentar no display
        self.present()?;

        // 5. Limpar damage para próximo frame
        self.damage.clear();

        Ok(())
    }

    /// Compõe uma janela no backbuffer por ID.
    fn composite_window_by_id(&mut self, window_id: u32) {
        // Extrair dados necessários primeiro
        let (src_size, position, is_transparent, shm_ptr, shm_size) = {
            let window = match self.windows.get(&window_id) {
                Some(w) => w,
                None => return,
            };
            (
                window.size,
                window.position,
                window.flags.has(gfx_types::WindowFlags::TRANSPARENT),
                window.shm.as_ptr(),
                window.shm.size(),
            )
        };

        // Debug: log window info on first few frames
        static mut DEBUG_COUNT: u32 = 0;
        unsafe {
            if DEBUG_COUNT < 3 {
                DEBUG_COUNT += 1;
                crate::println!(
                    "[Composite] Window {} at ({}, {})",
                    window_id,
                    position.x,
                    position.y
                );
                crate::println!("[Composite] Size: {}x{}", src_size.width, src_size.height);
                crate::println!("[Composite] SHM ptr: {:p}, size: {}", shm_ptr, shm_size);

                // Check first few pixels
                if shm_size > 0 {
                    let pixels = shm_ptr as *const u32;
                    let p0 = core::ptr::read_volatile(pixels);
                    let p1 = core::ptr::read_volatile(pixels.add(1));
                    let p2 = core::ptr::read_volatile(pixels.add(2));
                    crate::println!("[Composite] First 3 pixels: {:#x} {:#x} {:#x}", p0, p1, p2);
                }
            }
        }

        // Obter pixels do window
        let window = match self.windows.get(&window_id) {
            Some(w) => w,
            None => return,
        };
        let src_pixels: Vec<u32> = window.pixels().to_vec();

        // Debug: check src_pixels
        unsafe {
            static mut PIXELS_DEBUG: bool = false;
            if !PIXELS_DEBUG {
                PIXELS_DEBUG = true;
                crate::println!("[Composite] src_pixels len: {}", src_pixels.len());
                if src_pixels.len() >= 3 {
                    crate::println!(
                        "[Composite] Vec pixels: {:#x} {:#x} {:#x}",
                        src_pixels[0],
                        src_pixels[1],
                        src_pixels[2]
                    );
                }
            }
        }

        let dst_size = self.size();

        // Fazer blit
        if is_transparent {
            Blitter::blit_alpha(
                &mut self.backbuffer,
                dst_size,
                &src_pixels,
                src_size,
                Rect::from_size(src_size),
                position,
            );
        } else {
            Blitter::blit_opaque(
                &mut self.backbuffer,
                dst_size,
                &src_pixels,
                src_size,
                Rect::from_size(src_size),
                position,
            );
        }
    }

    /// Compõe uma janela no backbuffer.
    fn composite_window(&mut self, window: &Window) {
        let src_pixels = window.pixels();
        let src_size = window.size;
        let dst_size = self.size();
        let dst_point = window.position;

        // Usar blit com alpha se janela suporta transparência
        if window.flags.has(gfx_types::WindowFlags::TRANSPARENT) {
            Blitter::blit_alpha(
                &mut self.backbuffer,
                dst_size,
                src_pixels,
                src_size,
                Rect::from_size(src_size),
                dst_point,
            );
        } else {
            Blitter::blit_opaque(
                &mut self.backbuffer,
                dst_size,
                src_pixels,
                src_size,
                Rect::from_size(src_size),
                dst_point,
            );
        }
    }

    /// Envia backbuffer para o display.
    fn present(&self) -> SysResult<()> {
        // Converter para slice de bytes
        let byte_slice = unsafe {
            core::slice::from_raw_parts(
                self.backbuffer.as_ptr() as *const u8,
                self.backbuffer.len() * 4,
            )
        };

        // Enviar para framebuffer via syscall
        write_framebuffer(0, byte_slice)?;

        Ok(())
    }

    /// Retorna estatísticas.
    pub fn stats(&self) -> (u64, usize) {
        (self.frame_count, self.windows.len())
    }
}
