//! # Render Engine
//!
//! Motor de composição principal.

use super::blitter::Blitter;
use crate::scene::{DamageTracker, Layer, LayerManager, Window, WindowId};
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;
use gfx_types::{Color, DisplayInfo, LayerType, Point, Rect, Size};
use redpowder::graphics::write_framebuffer;
use redpowder::ipc::SharedMemory;
use redpowder::syscall::SysResult;

/// Cor de fundo padrão.
/// Cor de fundo padrão (quando não há wallpaper)
const BACKGROUND_COLOR: Color = Color(0xFF2d2d2d);

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

        redpowder::println!(
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

    /// Cria nova janela em uma camada específica.
    pub fn create_window(&mut self, size: Size, shm: SharedMemory, layer: LayerType) -> u32 {
        let id = self.next_window_id;
        self.next_window_id += 1;

        let mut window = Window::new(id, size, shm);
        window.layer = layer;
        self.windows.insert(id, window);
        self.layers.add_window_to_layer(WindowId(id), layer);

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

    /// Traz janela para a frente de sua camada.
    pub fn bring_to_front(&mut self, id: u32) {
        if let Some(win) = self.windows.get(&id) {
            let layer_type = win.layer;
            let window_id = crate::scene::window::WindowId(id);
            self.layers.get_mut(layer_type).remove_window(window_id);
            self.layers.get_mut(layer_type).add_window(window_id);

            let size = self.display_info.size();
            self.damage.damage_full(size.width, size.height);
        }
    }

    /// Marca janela como modificada.
    pub fn mark_damage(&mut self, id: u32) {
        if let Some(window) = self.windows.get(&id) {
            self.damage.add(window.rect());
        }
    }

    /// Marca a tela inteira como danificada.
    pub fn full_screen_damage(&mut self) {
        let size = self.display_info.size();
        self.damage.damage_full(size.width, size.height);
    }

    /// Retorna ID da janela na posição dada (se houver).
    /// Procura de cima para baixo (janela mais ao topo primeiro).
    pub fn window_at_point(&self, x: i32, y: i32) -> Option<u32> {
        for window_id in self.layers.iter_top_to_bottom() {
            if let Some(window) = self.windows.get(&window_id.0) {
                if window.visible && window.has_content {
                    let rect = window.rect();
                    if x >= rect.x
                        && x < rect.x + rect.width as i32
                        && y >= rect.y
                        && y < rect.y + rect.height as i32
                    {
                        return Some(window_id.0);
                    }
                }
            }
        }
        None
    }

    /// Destrói janela.
    pub fn destroy_window(&mut self, id: u32) {
        if let Some(window) = self.windows.remove(&id) {
            self.damage.add(window.rect());
            self.layers.remove_window(WindowId(id));
            crate::println!("[Render] Janela {} destruída", id);
        }
    }

    /// Altera o layer de uma janela.
    pub fn set_window_layer(&mut self, id: u32, layer: gfx_types::LayerType) {
        if let Some(window) = self.windows.get_mut(&id) {
            window.set_layer(layer);
            crate::println!("[Render] Janela {} -> layer {:?}", id, layer);
        }
    }

    /// Marca que a janela recebeu conteúdo (pelo menos um commit).
    pub fn mark_window_has_content(&mut self, id: u32) {
        if let Some(window) = self.windows.get_mut(&id) {
            if !window.has_content {
                window.set_has_content();
                crate::println!("[Render] Janela {} recebeu primeiro conteúdo", id);
            }
        }
    }

    /// Renderiza um frame completo se houver damage.
    pub fn render(&mut self) -> SysResult<()> {
        // Se não há nada para redesenhar, economizar CPU
        if !self.damage.has_damage() && self.frame_count > 0 {
            return Ok(());
        }

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

        // 2. Coletar IDs de janelas com conteúdo e ordenar por layer (Background primeiro)
        let mut windows_to_render: Vec<u32> = self
            .windows
            .iter()
            .filter(|(_, w)| w.has_content) // Só renderizar janelas que já receberam conteúdo
            .map(|(id, _)| *id)
            .collect();

        // Ordenar por layer type (Background=0 primeiro, Normal=1 depois, etc.)
        windows_to_render.sort_by(|a, b| {
            let layer_a = self
                .windows
                .get(a)
                .map(|w| w.layer)
                .unwrap_or(LayerType::Normal);
            let layer_b = self
                .windows
                .get(b)
                .map(|w| w.layer)
                .unwrap_or(LayerType::Normal);
            layer_a.cmp(&layer_b)
        });

        // 3. Compor janelas (na ordem: Background -> Normal -> Panel -> Overlay)
        for window_id in windows_to_render {
            self.composite_window_by_id(window_id);
        }

        // 4. Apresentar no display
        self.present()?;

        // 5. Limpar damage para próximo frame
        self.damage.clear();

        Ok(())
    }

    /// Renderiza um frame com cursor na posição especificada.
    pub fn render_with_cursor(&mut self, mouse_x: i32, mouse_y: i32) -> SysResult<()> {
        self.frame_count += 1;

        // Log periódico (a cada ~100 frames)
        if self.frame_count % 500 == 0 {
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

        // 2. Coletar IDs de janelas com conteúdo e ordenar por layer (Background primeiro)
        let mut windows_to_render: Vec<u32> = self
            .windows
            .iter()
            .filter(|(_, w)| w.has_content)
            .map(|(id, _)| *id)
            .collect();

        windows_to_render.sort_by(|a, b| {
            let layer_a = self
                .windows
                .get(a)
                .map(|w| w.layer)
                .unwrap_or(LayerType::Normal);
            let layer_b = self
                .windows
                .get(b)
                .map(|w| w.layer)
                .unwrap_or(LayerType::Normal);
            layer_a.cmp(&layer_b)
        });

        // 3. Compor janelas
        for window_id in windows_to_render {
            self.composite_window_by_id(window_id);
        }

        // 4. Desenhar cursor do mouse (por cima de tudo)
        crate::ui::cursor::draw(&mut self.backbuffer, size, mouse_x, mouse_y);

        // 5. Apresentar no display
        self.present()?;

        // 6. Limpar damage para próximo frame
        self.damage.clear();

        Ok(())
    }

    /// Compõe uma janela no backbuffer por ID.
    fn composite_window_by_id(&mut self, window_id: u32) {
        // Extrair dados necessários
        let window = match self.windows.get(&window_id) {
            Some(w) => w,
            None => return,
        };

        let src_size = window.size;
        let position = window.position;
        let is_transparent = window.flags.has(gfx_types::WindowFlags::TRANSPARENT);
        let src_pixels = window.pixels();
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
                &src_pixels,
                src_size,
                Rect::from_size(src_size),
                dst_point,
            );
        } else {
            Blitter::blit_opaque(
                &mut self.backbuffer,
                dst_size,
                &src_pixels,
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
