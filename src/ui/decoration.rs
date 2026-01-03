//! # Window Decorations
//!
//! Desenho de decorações de janelas (título, botões).

use gfx_types::color::Color;
use gfx_types::geometry::{Rect, Size};

use crate::render::Blitter;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Altura da barra de título.
pub const TITLEBAR_HEIGHT: u32 = 24;

/// Largura da borda.
pub const BORDER_WIDTH: u32 = 1;

/// Cor da barra de título (ativa).
pub const TITLEBAR_COLOR_ACTIVE: Color = Color(0xFF3d3d3d);

/// Cor da barra de título (inativa).
pub const TITLEBAR_COLOR_INACTIVE: Color = Color(0xFF2d2d2d);

/// Cor da borda (ativa).
pub const BORDER_COLOR_ACTIVE: Color = Color(0xFF505050);

/// Cor da borda (inativa).
pub const BORDER_COLOR_INACTIVE: Color = Color(0xFF3d3d3d);

/// Cor do texto.
pub const TEXT_COLOR: Color = Color::WHITE;

/// Tamanho dos botões.
pub const BTN_SIZE: u32 = 20;

/// Cor do botão fechar.
pub const BTN_CLOSE_COLOR: Color = Color::REDSTONE_ACCENT;

/// Cor do botão minimizar.
pub const BTN_MINIMIZE_COLOR: Color = Color(0xFF4a90d9);

// =============================================================================
// FUNÇÕES
// =============================================================================

/// Desenha decorações de janela.
pub fn draw_window_decoration(
    buffer: &mut [u32],
    buffer_size: Size,
    window_rect: Rect,
    title: &str,
    is_focused: bool,
) {
    let titlebar_color = if is_focused {
        TITLEBAR_COLOR_ACTIVE
    } else {
        TITLEBAR_COLOR_INACTIVE
    };

    let border_color = if is_focused {
        BORDER_COLOR_ACTIVE
    } else {
        BORDER_COLOR_INACTIVE
    };

    // 1. Barra de título
    let titlebar_rect = Rect::new(
        window_rect.x,
        window_rect.y,
        window_rect.width,
        TITLEBAR_HEIGHT,
    );
    Blitter::fill_rect(buffer, buffer_size, titlebar_rect, titlebar_color);

    // 2. Borda
    Blitter::stroke_rect(buffer, buffer_size, window_rect, BORDER_WIDTH, border_color);

    // 3. Botão fechar (X)
    let close_x = window_rect.right() - BTN_SIZE as i32 - 2;
    let close_y = window_rect.y + 2;
    let close_rect = Rect::new(close_x, close_y, BTN_SIZE, BTN_SIZE);
    Blitter::fill_rect(buffer, buffer_size, close_rect, BTN_CLOSE_COLOR);
    draw_close_icon(buffer, buffer_size, close_x + 4, close_y + 4);

    // 4. Botão minimizar (-)
    let min_x = close_x - BTN_SIZE as i32 - 4;
    let min_rect = Rect::new(min_x, close_y, BTN_SIZE, BTN_SIZE);
    Blitter::fill_rect(buffer, buffer_size, min_rect, BTN_MINIMIZE_COLOR);
    draw_minimize_icon(buffer, buffer_size, min_x + 4, close_y + 8);
}

/// Desenha ícone X (fechar).
fn draw_close_icon(buffer: &mut [u32], size: Size, x: i32, y: i32) {
    let color = Color::WHITE;
    for i in 0..12 {
        Blitter::put_pixel(buffer, size, x + i, y + i, color);
        Blitter::put_pixel(buffer, size, x + 11 - i, y + i, color);
    }
}

/// Desenha ícone - (minimizar).
fn draw_minimize_icon(buffer: &mut [u32], size: Size, x: i32, y: i32) {
    let color = Color::WHITE;
    Blitter::fill_rect(buffer, size, Rect::new(x, y, 12, 2), color);
}
