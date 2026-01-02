//! # Window Decoration - Firefly Compositor
//!
//! Desenha decorações de janela (título, bordas, botões).

use gfx_types::{Color, Size};

// ============================================================================
// CONSTANTES
// ============================================================================

pub const TITLEBAR_HEIGHT: u32 = 24;
pub const BORDER_WIDTH: u32 = 2;

const TITLEBAR_COLOR_ACTIVE: Color = Color::WHITE;
const TITLEBAR_COLOR_INACTIVE: Color = Color::rgb(200, 200, 200);
const BORDER_COLOR_ACTIVE: Color = Color::WHITE;
const BORDER_COLOR_INACTIVE: Color = Color::rgb(200, 200, 200);
const TEXT_COLOR: Color = Color::BLACK;

const BTN_SIZE: u32 = TITLEBAR_HEIGHT - 4;
const BTN_CLOSE_COLOR: Color = Color::rgb(232, 17, 35);
const BTN_MIN_COLOR: Color = Color::rgb(200, 200, 200); // Cinza para minimizar

// ============================================================================
// FUNÇÕES AUXILIARES
// ============================================================================

/// Preenche retângulo em um buffer.
fn fill_rect(buffer: &mut [u32], buffer_size: Size, x: i32, y: i32, w: u32, h: u32, color: Color) {
    let color_u32 = color.as_u32();
    for dy in 0..h {
        let py = y + dy as i32;
        if py < 0 || py >= buffer_size.height as i32 {
            continue;
        }
        for dx in 0..w {
            let px = x + dx as i32;
            if px < 0 || px >= buffer_size.width as i32 {
                continue;
            }
            let idx = (py as usize * buffer_size.width as usize) + px as usize;
            if idx < buffer.len() {
                buffer[idx] = color_u32;
            }
        }
    }
}

/// Desenha um pixel em um buffer.
fn put_pixel(buffer: &mut [u32], buffer_size: Size, x: i32, y: i32, color: Color) {
    if x < 0 || y < 0 || x >= buffer_size.width as i32 || y >= buffer_size.height as i32 {
        return;
    }
    let idx = (y as usize * buffer_size.width as usize) + x as usize;
    if idx < buffer.len() {
        buffer[idx] = color.as_u32();
    }
}

// ============================================================================
// FUNÇÕES PÚBLICAS
// ============================================================================

/// Desenha a decoração completa de uma janela.
pub fn draw_window_decoration(
    buffer: &mut [u32],
    buffer_size: Size,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    _title: &str,
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

    // Top (Título)
    fill_rect(
        buffer,
        buffer_size,
        x as i32,
        y as i32,
        w,
        TITLEBAR_HEIGHT,
        title_color,
    );

    // Left border
    fill_rect(
        buffer,
        buffer_size,
        x as i32,
        (y + TITLEBAR_HEIGHT) as i32,
        BORDER_WIDTH,
        h - TITLEBAR_HEIGHT,
        border_color,
    );

    // Right border
    fill_rect(
        buffer,
        buffer_size,
        (x + w - BORDER_WIDTH) as i32,
        (y + TITLEBAR_HEIGHT) as i32,
        BORDER_WIDTH,
        h - TITLEBAR_HEIGHT,
        border_color,
    );

    // Bottom border
    fill_rect(
        buffer,
        buffer_size,
        x as i32,
        (y + h - BORDER_WIDTH) as i32,
        w,
        BORDER_WIDTH,
        border_color,
    );

    // Botão Fechar (X)
    draw_close_button(buffer, buffer_size, x + w - BTN_SIZE - 2, y + 2);

    // Botão Minimizar (-)
    draw_minimize_button(buffer, buffer_size, x + w - (BTN_SIZE * 2) - 6, y + 2);

    // Título indicador (3 pontos)
    fill_rect(
        buffer,
        buffer_size,
        (x + 10) as i32,
        (y + 10) as i32,
        4,
        4,
        TEXT_COLOR,
    );
    fill_rect(
        buffer,
        buffer_size,
        (x + 16) as i32,
        (y + 10) as i32,
        4,
        4,
        TEXT_COLOR,
    );
    fill_rect(
        buffer,
        buffer_size,
        (x + 22) as i32,
        (y + 10) as i32,
        4,
        4,
        TEXT_COLOR,
    );
}

fn draw_close_button(buffer: &mut [u32], buffer_size: Size, x: u32, y: u32) {
    fill_rect(
        buffer,
        buffer_size,
        x as i32,
        y as i32,
        BTN_SIZE,
        BTN_SIZE,
        BTN_CLOSE_COLOR,
    );

    // X branco simples (diagonal)
    let x_start = x + 4;
    let y_start = y + 4;
    let size = BTN_SIZE - 8;

    for i in 0..size {
        put_pixel(
            buffer,
            buffer_size,
            (x_start + i) as i32,
            (y_start + i) as i32,
            Color::WHITE,
        );
        put_pixel(
            buffer,
            buffer_size,
            (x_start + size - 1 - i) as i32,
            (y_start + i) as i32,
            Color::WHITE,
        );
    }
}

fn draw_minimize_button(buffer: &mut [u32], buffer_size: Size, x: u32, y: u32) {
    fill_rect(
        buffer,
        buffer_size,
        x as i32,
        y as i32,
        BTN_SIZE,
        BTN_SIZE,
        BTN_MIN_COLOR,
    );

    // Linha branca horizontal
    fill_rect(
        buffer,
        buffer_size,
        (x + 4) as i32,
        (y + BTN_SIZE - 6) as i32,
        BTN_SIZE - 8,
        2,
        Color::BLACK,
    );
}
