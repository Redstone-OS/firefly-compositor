//! # Firefly Compositor
//!
//! Compositor gráfico do RedstoneOS.
//! Gerencia janelas e compõe no framebuffer.

#![no_std]
#![no_main]

mod cursor;

use core::panic::PanicInfo;
use redpowder::graphics::{Color, Framebuffer};
use redpowder::println;

// ============================================================================
// CONSTANTES
// ============================================================================

/// Cor de fundo coral (#EE6A50)
const BG_COLOR: Color = Color::ORANGE;

/// Nome da porta do compositor
const COMPOSITOR_PORT: &str = "firefly.compositor";

// ============================================================================
// ENTRADA
// ============================================================================

#[no_mangle]
#[link_section = ".text._start"]
pub extern "C" fn _start() -> ! {
    println!("[Firefly] Compositor starting...");

    // Inicializar framebuffer
    if let Ok(mut fb) = Framebuffer::new() {
        println!("[Firefly] FB OK");

        // Limpar tela com fundo
        let _ = fb.clear(BG_COLOR);
        println!("[Firefly] Background cleared");

        // Desenhar cursor no centro
        let w = fb.width();
        let h = fb.height();
        cursor::draw(&mut fb, (w / 2) as i32, (h / 2) as i32);
        println!("[Firefly] Cursor drawn");

        // TODO: Criar porta para receber mensagens de clientes
        // Por enquanto, apenas mostra o fundo e cursor
        println!("[Firefly] Ready!");

        // Loop principal do compositor
        // Faz yield para permitir que outros processos executem
        loop {
            redpowder::process::yield_now();
        }
    } else {
        println!("[Firefly] FB FAIL");
    }

    println!("[Firefly] Done!");
    loop {
        core::hint::spin_loop();
    }
}

// ============================================================================
// PANIC HANDLER
// ============================================================================

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
