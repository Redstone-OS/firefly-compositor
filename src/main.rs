//! # Firefly Compositor
//!
//! Compositor gráfico do RedstoneOS.
//! Gerencia janelas e compõe no framebuffer.

#![no_std]
#![no_main]

extern crate alloc;

mod input;
mod render;
mod scenegraph;
mod server;
mod ui;

use core::panic::PanicInfo;
use redpowder::println;

#[global_allocator]
static ALLOCATOR: redpowder::mem::heap::SyscallAllocator = redpowder::mem::heap::SyscallAllocator;

// ============================================================================
// ENTRADA
// ============================================================================

#[no_mangle]
#[link_section = ".text._start"]
pub extern "C" fn _start() -> ! {
    println!("[Firefly] Compositor starting (Server Mode v2.0)...");

    match server::Server::new() {
        Ok(mut server) => {
            println!("[Firefly] Server initialized. Listening for clients.");

            if let Err(e) = server.run() {
                println!("[Firefly] FATAL: Server crashed: {:?}", e);
            }
        }
        Err(e) => {
            println!("[Firefly] FATAL: Failed to init server: {:?}", e);
        }
    }

    println!("[Firefly] Component Exited!");
    loop {
        let _ = redpowder::time::sleep(1000);
    }
}

// ============================================================================
// PANIC HANDLER
// ============================================================================

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
