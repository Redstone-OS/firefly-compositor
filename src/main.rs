//! # Firefly Compositor
//!
//! Compositor gráfico do RedstoneOS.

#![no_std]
#![no_main]

mod cursor;

use core::panic::PanicInfo;
use redpowder::graphics::{Color, Framebuffer};
use redpowder::println;

/// Cor de fundo coral (#EE6A50)
const BG_COLOR: Color = Color::ORANGE;

#[no_mangle]
#[link_section = ".text._start"]
pub extern "C" fn _start() -> ! {
    println!("[Firefly] Start!");

    // Inicializar framebuffer
    if let Ok(mut fb) = Framebuffer::new() {
        println!("[Firefly] FB OK");
        let _ = fb.clear(BG_COLOR);
        println!("[Firefly] Clear OK");

        // Desenhar retângulo simples (janela de teste)
        let w = fb.width();
        let h = fb.height();
        let win_x = w / 4;
        let win_y = h / 4;
        let win_w = w / 2;
        let win_h = h / 2;

        // Borda da janela
        for x in win_x..(win_x + win_w) {
            let _ = fb.put_pixel(x, win_y, Color::WHITE);
            let _ = fb.put_pixel(x, win_y + win_h - 1, Color::WHITE);
        }
        for y in win_y..(win_y + win_h) {
            let _ = fb.put_pixel(win_x, y, Color::WHITE);
            let _ = fb.put_pixel(win_x + win_w - 1, y, Color::WHITE);
        }

        println!("[Firefly] Window done!");

        // Desenhar cursor no centro da tela
        let cursor_x = (w / 2) as i32;
        let cursor_y = (h / 2) as i32;
        cursor::draw(&mut fb, cursor_x, cursor_y);

        println!("[Firefly] Cursor drawn!");
    } else {
        println!("[Firefly] FB FAIL");
    }

    println!("[Firefly] Done!");
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
