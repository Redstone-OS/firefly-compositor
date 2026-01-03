//! # Cursor
//!
//! Desenho do cursor do mouse.

use gfx_types::color::Color;
use gfx_types::geometry::Size;

// =============================================================================
// CONSTANTES
// =============================================================================

/// Largura do cursor.
const CURSOR_WIDTH: usize = 12;

/// Altura do cursor.
const CURSOR_HEIGHT: usize = 19;

/// Bitmap do cursor padrão (seta).
/// 0 = transparente, 1 = preto (contorno), 2 = branco (preenchimento)
#[rustfmt::skip]
const CURSOR_BITMAP: [[u8; CURSOR_WIDTH]; CURSOR_HEIGHT] = [
    [1,0,0,0,0,0,0,0,0,0,0,0],
    [1,1,0,0,0,0,0,0,0,0,0,0],
    [1,2,1,0,0,0,0,0,0,0,0,0],
    [1,2,2,1,0,0,0,0,0,0,0,0],
    [1,2,2,2,1,0,0,0,0,0,0,0],
    [1,2,2,2,2,1,0,0,0,0,0,0],
    [1,2,2,2,2,2,1,0,0,0,0,0],
    [1,2,2,2,2,2,2,1,0,0,0,0],
    [1,2,2,2,2,2,2,2,1,0,0,0],
    [1,2,2,2,2,2,2,2,2,1,0,0],
    [1,2,2,2,2,2,2,2,2,2,1,0],
    [1,2,2,2,2,2,2,1,1,1,1,1],
    [1,2,2,2,1,2,2,1,0,0,0,0],
    [1,2,2,1,0,1,2,2,1,0,0,0],
    [1,2,1,0,0,1,2,2,1,0,0,0],
    [1,1,0,0,0,0,1,2,2,1,0,0],
    [1,0,0,0,0,0,1,2,2,1,0,0],
    [0,0,0,0,0,0,0,1,2,1,0,0],
    [0,0,0,0,0,0,0,0,1,0,0,0],
];

/// Cor do contorno do cursor.
const CURSOR_OUTLINE: Color = Color::BLACK;

/// Cor do preenchimento do cursor.
const CURSOR_FILL: Color = Color::WHITE;

// =============================================================================
// FUNÇÕES
// =============================================================================

/// Desenha o cursor na posição especificada.
pub fn draw(buffer: &mut [u32], buffer_size: Size, x: i32, y: i32) {
    let stride = buffer_size.width as usize;

    for py in 0..CURSOR_HEIGHT {
        let screen_y = y as usize + py;
        if screen_y >= buffer_size.height as usize {
            continue;
        }

        for px in 0..CURSOR_WIDTH {
            let screen_x = x as usize + px;
            if screen_x >= buffer_size.width as usize {
                continue;
            }

            let pixel_type = CURSOR_BITMAP[py][px];
            if pixel_type == 0 {
                continue; // Transparente
            }

            let idx = screen_y * stride + screen_x;
            if idx < buffer.len() {
                buffer[idx] = match pixel_type {
                    1 => CURSOR_OUTLINE.as_u32(),
                    2 => CURSOR_FILL.as_u32(),
                    _ => continue,
                };
            }
        }
    }
}

/// Desenha cursor com cor customizada.
pub fn draw_colored(
    buffer: &mut [u32],
    buffer_size: Size,
    x: i32,
    y: i32,
    outline: Color,
    fill: Color,
) {
    let stride = buffer_size.width as usize;

    for py in 0..CURSOR_HEIGHT {
        let screen_y = y as usize + py;
        if screen_y >= buffer_size.height as usize {
            continue;
        }

        for px in 0..CURSOR_WIDTH {
            let screen_x = x as usize + px;
            if screen_x >= buffer_size.width as usize {
                continue;
            }

            let pixel_type = CURSOR_BITMAP[py][px];
            if pixel_type == 0 {
                continue;
            }

            let idx = screen_y * stride + screen_x;
            if idx < buffer.len() {
                buffer[idx] = match pixel_type {
                    1 => outline.as_u32(),
                    2 => fill.as_u32(),
                    _ => continue,
                };
            }
        }
    }
}
