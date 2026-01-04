//! # Render Engine
//!
//! Motor de composição principal do Firefly.
//!
//! ## Responsabilidades
//!
//! - Gerenciar o backbuffer
//! - Compor janelas de todas as camadas
//! - Desenhar cursor e efeitos
//! - Apresentar frames no display

use super::blitter::Blitter;
use crate::scene::{DamageTracker, LayerManager, Window, WindowId};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use gfx_types::color::Color;
use gfx_types::display::DisplayInfo;
use gfx_types::geometry::{Point, Rect, Size};
use gfx_types::window::LayerType;
use redpowder::graphics::write_pixels;
use redpowder::ipc::SharedMemory;
use redpowder::syscall::SysResult;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Cor de fundo padrão (quando não há wallpaper).
const BACKGROUND_COLOR: Color = Color::REDSTONE_SECONDARY;

/// Cor da sombra das janelas.
const SHADOW_COLOR: Color = Color(0x40000000);

/// Offset da sombra.
const SHADOW_OFFSET: Point = Point { x: 4, y: 4 };

/// Blur radius da sombra.
const SHADOW_BLUR: u32 = 8;

// =============================================================================
// RENDER ENGINE
// =============================================================================

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
    /// Janela com foco.
    focused_window: Option<u32>,
    /// Posição do cursor.
    cursor_pos: Point,
    /// Cursor visível.
    cursor_visible: bool,
}

impl RenderEngine {
    /// Cria novo motor de renderização.
    pub fn new(display_info: DisplayInfo) -> Self {
        let size = (display_info.width * display_info.height) as usize;
        let backbuffer = vec![BACKGROUND_COLOR.as_u32(); size];

        redpowder::println!(
            "[Render] Backbuffer criado: {}x{} ({} KB)",
            display_info.width,
            display_info.height,
            size * 4 / 1024
        );

        let mut damage = DamageTracker::new();
        damage.set_size(display_info.width, display_info.height);

        Self {
            display_info,
            backbuffer,
            layers: LayerManager::new(),
            windows: BTreeMap::new(),
            damage,
            next_window_id: 1,
            frame_count: 0,
            focused_window: None,
            cursor_pos: Point::ZERO,
            cursor_visible: true,
        }
    }

    // =========================================================================
    // PROPRIEDADES
    // =========================================================================

    /// Retorna tamanho do display.
    #[inline]
    pub fn size(&self) -> Size {
        Size::new(self.display_info.width, self.display_info.height)
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Retorna informações do display.
    #[inline]
    pub fn display_info(&self) -> &DisplayInfo {
        &self.display_info
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Retorna número de frames renderizados.
    #[inline]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Retorna estatísticas.
    pub fn stats(&self) -> (u64, usize) {
        (self.frame_count, self.windows.len())
    }

    // =========================================================================
    // JANELAS
    // =========================================================================

    /// Cria nova janela.
    pub fn create_window(
        &mut self,
        size: Size,
        shm: SharedMemory,
        layer: LayerType,
        title: String,
    ) -> u32 {
        let id = self.next_window_id;
        self.next_window_id += 1;

        let mut window = Window::new(id, size, shm);
        window.layer = layer;
        window.title = title.clone();

        redpowder::println!(
            "[Render] Janela {} criada ({}x{}) layer={:?} '{}'",
            id,
            size.width,
            size.height,
            layer,
            title
        );

        self.windows.insert(id, window);
        self.layers.add_window_to_layer(WindowId(id), layer);
        self.damage.add(Rect::from_size(size));

        id
    }

    /// Obtém janela por ID.
    #[inline]
    pub fn get_window(&self, id: u32) -> Option<&Window> {
        self.windows.get(&id)
    }

    /// Obtém janela mutável por ID.
    #[inline]
    pub fn get_window_mut(&mut self, id: u32) -> Option<&mut Window> {
        self.windows.get_mut(&id)
    }

    /// Destrói janela.
    pub fn destroy_window(&mut self, id: u32) {
        if let Some(window) = self.windows.remove(&id) {
            self.damage.add(window.rect());
            self.layers.remove_window(WindowId(id));

            if self.focused_window == Some(id) {
                self.focused_window = None;
            }

            redpowder::println!("[Render] Janela {} destruída", id);
        }
    }

    /// Move janela para nova posição.
    pub fn move_window(&mut self, id: u32, x: i32, y: i32) {
        if let Some(window) = self.windows.get_mut(&id) {
            self.damage.add(window.rect());
            window.move_to(x, y);
            self.damage.add(window.rect());
        }
    }

    /// Traz janela para a frente.
    pub fn bring_to_front(&mut self, id: u32) {
        if let Some(window) = self.windows.get(&id) {
            let layer = window.layer;
            self.layers.get_mut(layer).bring_to_front(WindowId(id));
            self.damage.add(window.rect());
        }
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Envia janela para trás.
    pub fn send_to_back(&mut self, id: u32) {
        if let Some(window) = self.windows.get(&id) {
            let layer = window.layer;
            self.layers.get_mut(layer).send_to_back(WindowId(id));
            self.damage.add(window.rect());
        }
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Altera layer de uma janela.
    pub fn set_window_layer(&mut self, id: u32, new_layer: LayerType) {
        if let Some(window) = self.windows.get_mut(&id) {
            let old_layer = window.layer;
            if old_layer != new_layer {
                self.layers.move_window(WindowId(id), old_layer, new_layer);
                window.set_layer(new_layer);
                self.damage.add(window.rect());
            }
        }
    }

    /// Marca que janela recebeu conteúdo.
    pub fn mark_window_has_content(&mut self, id: u32) {
        if let Some(window) = self.windows.get_mut(&id) {
            if !window.has_content {
                window.set_has_content();
                self.damage.add(window.rect());
            }
        }
    }

    /// Marca janela como danificada.
    pub fn mark_damage(&mut self, id: u32) {
        if let Some(window) = self.windows.get(&id) {
            self.damage.add(window.rect());
        }
    }

    /// Marca tela inteira como danificada.
    pub fn full_screen_damage(&mut self) {
        self.damage
            .damage_full(self.display_info.width, self.display_info.height);
    }

    // =========================================================================
    // HIT TESTING
    // =========================================================================

    /// Retorna ID da janela na posição dada (se houver).
    pub fn window_at_point(&self, x: i32, y: i32) -> Option<u32> {
        for window_id in self.layers.iter_top_to_bottom() {
            if let Some(window) = self.windows.get(&window_id.0) {
                if window.is_visible() && window.contains_point(x, y) {
                    return Some(window_id.0);
                }
            }
        }
        None
    }

    // =========================================================================
    // FOCO
    // =========================================================================

    /// Define janela com foco.
    pub fn set_focus(&mut self, id: Option<u32>) {
        if self.focused_window != id {
            // Marcar janela antiga como danificada (para remover indicador de foco)
            if let Some(old_id) = self.focused_window {
                if let Some(window) = self.windows.get(&old_id) {
                    self.damage.add(window.rect());
                }
            }

            self.focused_window = id;

            // Marcar nova janela como danificada
            if let Some(new_id) = id {
                if let Some(window) = self.windows.get(&new_id) {
                    self.damage.add(window.rect());
                }
            }
        }
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Retorna janela com foco.
    #[inline]
    pub fn focused_window(&self) -> Option<u32> {
        self.focused_window
    }

    // =========================================================================
    // CURSOR
    // =========================================================================

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Atualiza posição do cursor.
    pub fn set_cursor_position(&mut self, x: i32, y: i32) {
        self.cursor_pos = Point::new(x, y);
    }

    // TODO: Revisar no futuro
    #[allow(unused)]
    /// Define visibilidade do cursor.
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    // =========================================================================
    // RENDERIZAÇÃO
    // =========================================================================

    /// Renderiza um frame com cursor.
    pub fn render(&mut self, mouse_x: i32, mouse_y: i32) -> SysResult<()> {
        self.cursor_pos = Point::new(mouse_x, mouse_y);
        self.frame_count += 1;

        // Log periódico
        if self.frame_count % 500 == 0 {
            redpowder::println!(
                "[Render] Frame {}, {} janelas, foco={:?}",
                self.frame_count,
                self.windows.len(),
                self.focused_window
            );
        }

        // 1. Limpar backbuffer
        let size = self.size();
        Blitter::fill_rect(
            &mut self.backbuffer,
            size,
            Rect::from_size(size),
            BACKGROUND_COLOR,
        );

        // 2. Coletar janelas para renderizar (ordenadas por layer)
        let windows_to_render: Vec<u32> = self
            .layers
            .iter_bottom_to_top()
            .filter(|id| {
                self.windows
                    .get(&id.0)
                    .map(|w| w.is_visible())
                    .unwrap_or(false)
            })
            .map(|id| id.0)
            .collect();

        // 3. Compor janelas
        for window_id in windows_to_render {
            self.composite_window(window_id);
        }

        // 4. Desenhar cursor
        if self.cursor_visible {
            crate::ui::cursor::draw(&mut self.backbuffer, size, mouse_x, mouse_y);
        }

        // 5. Apresentar
        self.present()?;

        // 6. Limpar damage
        self.damage.clear();

        Ok(())
    }

    /// Compõe uma janela no backbuffer.
    fn composite_window(&mut self, id: u32) {
        let window = match self.windows.get(&id) {
            Some(w) => w,
            None => return,
        };

        let src_pixels = window.pixels();
        let src_size = window.size;
        let dst_size = self.size();
        let position = window.position;

        // Desenhar sombra se habilitado
        if window.has_shadow() {
            Blitter::draw_shadow(
                &mut self.backbuffer,
                dst_size,
                window.rect(),
                SHADOW_OFFSET,
                SHADOW_BLUR,
                SHADOW_COLOR,
            );
        }

        // Blit
        if window.is_transparent() {
            Blitter::blit_alpha(
                &mut self.backbuffer,
                dst_size,
                src_pixels,
                src_size,
                Rect::from_size(src_size),
                position,
            );
        } else {
            Blitter::blit_opaque(
                &mut self.backbuffer,
                dst_size,
                src_pixels,
                src_size,
                Rect::from_size(src_size),
                position,
            );
        }

        // Indicador de foco (borda colorida)
        if self.focused_window == Some(id) && window.has_decorations() {
            Blitter::stroke_rect(
                &mut self.backbuffer,
                dst_size,
                window.rect(),
                2,
                Color::REDSTONE_ACCENT,
            );
        }
    }

    /// Envia backbuffer para o display.
    fn present(&self) -> SysResult<()> {
        let byte_slice = unsafe {
            core::slice::from_raw_parts(
                self.backbuffer.as_ptr() as *const u8,
                self.backbuffer.len() * 4,
            )
        };

        write_pixels(0, byte_slice)?;
        Ok(())
    }
}
