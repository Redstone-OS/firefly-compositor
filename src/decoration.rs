//! # Window Decoration - Firefly Compositor
//!
//! Desenha decorações de janela (título, bordas, botões).

use redpowder::graphics::{Color, Framebuffer};

// ============================================================================
// CONSTANTES
// ============================================================================

pub const TITLEBAR_HEIGHT: u32 = 24;
pub const BORDER_WIDTH: u32 = 2;

const TITLEBAR_COLOR_ACTIVE: Color = Color::WHITE; // User requested "faixa branca"
const TITLEBAR_COLOR_INACTIVE: Color = Color::rgb(200, 200, 200); // Light Gray
const BORDER_COLOR_ACTIVE: Color = Color::WHITE;
const BORDER_COLOR_INACTIVE: Color = Color::rgb(200, 200, 200);
const TEXT_COLOR: Color = Color::BLACK;

// Botões (X, _, etc)
const BTN_SIZE: u32 = TITLEBAR_HEIGHT - 4;
const BTN_CLOSE_COLOR: Color = Color::rgb(232, 17, 35); // Vermelho

// ============================================================================
// FUNÇÕES
// ============================================================================

/// Desenha a decoração completa de uma janela
pub fn draw_window_decoration(
    fb: &mut Framebuffer,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    title: &str,
    is_active: bool,
) {
    let title_color = if is_active {
        TITLEBAR_COLOR_ACTIVE
    } else {
        TITLEBAR_COLOR_INACTIVE
    };
    let border_color = if is_active {
        BORDER_COLOR_ACTIVE
    } else {
        BORDER_COLOR_INACTIVE
    };

    // Borda (retângulo preenchido maior - retângulo menor)
    // Ou desenhando 4 retângulos.
    // Top (Título)
    let _ = fb.fill_rect(x, y, w, TITLEBAR_HEIGHT, title_color);

    // Left
    let _ = fb.fill_rect(
        x,
        y + TITLEBAR_HEIGHT,
        BORDER_WIDTH,
        h - TITLEBAR_HEIGHT,
        border_color,
    );
    // Right
    let _ = fb.fill_rect(
        x + w - BORDER_WIDTH,
        y + TITLEBAR_HEIGHT,
        BORDER_WIDTH,
        h - TITLEBAR_HEIGHT,
        border_color,
    );
    // Bottom
    let _ = fb.fill_rect(x, y + h - BORDER_WIDTH, w, BORDER_WIDTH, border_color);

    // Botão Fechar (X)
    draw_close_button(fb, x + w - BTN_SIZE - 2, y + 2);

    // Título (texto simples - placeholder)
    // Como não temos fonte aqui (estava no shell), vamos desenhar um indicador simples
    // 3 pontos brancos
    let _ = fb.fill_rect(x + 10, y + 10, 4, 4, TEXT_COLOR);
    let _ = fb.fill_rect(x + 16, y + 10, 4, 4, TEXT_COLOR);
    let _ = fb.fill_rect(x + 22, y + 10, 4, 4, TEXT_COLOR);
}

fn draw_close_button(fb: &mut Framebuffer, x: u32, y: u32) {
    let _ = fb.fill_rect(x, y, BTN_SIZE, BTN_SIZE, BTN_CLOSE_COLOR);
    // X branco simples
    // diagonal 1
    let x_start = x + 4;
    let y_start = y + 4;
    let size = BTN_SIZE - 8;

    for i in 0..size {
        let _ = fb.put_pixel(x_start + i, y_start + i, Color::WHITE);
        let _ = fb.put_pixel(x_start + size - 1 - i, y_start + i, Color::WHITE);
    }
}
