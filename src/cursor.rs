//! # Cursor do Mouse - Firefly Compositor
//!
//! Desenho do cursor na tela.

use redpowder::graphics::{Color, Framebuffer};

/// Dados do cursor em forma de seta (12x18 pixels)
/// 0 = transparente, 1 = branco (borda), 2 = preto (preenchimento)
pub const CURSOR_DATA: [[u8; 12]; 18] = [
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0],
    [1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0],
    [1, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1],
    [1, 2, 2, 2, 1, 2, 2, 1, 0, 0, 0, 0],
    [1, 2, 2, 1, 0, 1, 2, 2, 1, 0, 0, 0],
    [1, 2, 1, 0, 0, 1, 2, 2, 1, 0, 0, 0],
    [1, 1, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0],
    [1, 0, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
];

pub const CURSOR_WIDTH: u32 = 12;
pub const CURSOR_HEIGHT: u32 = 18;

/// Desenha cursor na posição especificada
pub fn draw(fb: &mut Framebuffer, x: i32, y: i32) {
    for dy in 0..CURSOR_HEIGHT {
        for dx in 0..CURSOR_WIDTH {
            let px = x + dx as i32;
            let py = y + dy as i32;

            // Verificar bounds
            if px < 0 || py < 0 || px >= fb.width() as i32 || py >= fb.height() as i32 {
                continue;
            }

            let pixel = CURSOR_DATA[dy as usize][dx as usize];
            let color = match pixel {
                1 => Some(Color::WHITE), // Borda branca
                2 => Some(Color::BLACK), // Preenchimento preto
                _ => None,               // Transparente
            };

            if let Some(c) = color {
                let _ = fb.put_pixel(px as u32, py as u32, c);
            }
        }
    }
}

/// Apaga cursor desenhando o fundo na posição
pub fn erase(fb: &mut Framebuffer, x: i32, y: i32, bg_color: Color) {
    for dy in 0..CURSOR_HEIGHT {
        for dx in 0..CURSOR_WIDTH {
            let px = x + dx as i32;
            let py = y + dy as i32;

            // Verificar bounds
            if px < 0 || py < 0 || px >= fb.width() as i32 || py >= fb.height() as i32 {
                continue;
            }

            // Apenas apagar pixels não-transparentes do cursor
            let pixel = CURSOR_DATA[dy as usize][dx as usize];
            if pixel != 0 {
                let _ = fb.put_pixel(px as u32, py as u32, bg_color);
            }
        }
    }
}
