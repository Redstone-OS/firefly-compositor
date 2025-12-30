//! # Firefly Compositor
//!
//! Compositor gráfico do RedstoneOS.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use redpowder::graphics::{Color, Framebuffer};
use redpowder::println;
use redpowder::process::yield_now;

/// Cor de fundo coral (#EE6A50)
const BG_COLOR: Color = Color::ORANGE;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("[Firefly] Compositor starting...");

    // Inicializar framebuffer
    let mut fb = match Framebuffer::new() {
        Ok(f) => {
            println!("[Firefly] FB OK");
            f
        }
        Err(_) => {
            println!("[Firefly] FB FAIL");
            loop {
                let _ = yield_now();
            }
        }
    };

    // Limpar com cor de fundo
    println!("[Firefly] Clearing...");
    let _ = fb.clear(BG_COLOR);
    println!("[Firefly] Screen coral!");

    // Loop principal
    loop {
        let _ = yield_now();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("[Firefly] PANIC!");
    loop {}
}
