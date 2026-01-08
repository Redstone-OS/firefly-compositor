//! # Firefly Compositor
//!
//! Compositor gráfico do RedstoneOS.
//!
//! O Firefly é responsável por:
//! - Gerenciar superfícies (janelas) de aplicações
//! - Compor todas as superfícies em um único framebuffer
//! - Processar entrada de mouse e teclado
//! - Comunicar-se com clientes via protocolo IPC
//!
//! ## Arquitetura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                    Firefly Compositor               │
//! ├─────────────────────────────────────────────────────┤
//! │  Server      │  Compositor   │  Input    │  Render  │
//! │  (IPC)       │  (Scene)      │  (Mouse)  │  (FB)    │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Protocolo
//!
//! Clientes se comunicam via portas IPC nomeadas:
//! - `firefly.compositor` - Porta principal para requisições
//! - `win.r.<id>` - Portas de resposta por cliente

#![no_std]
#![no_main]

extern crate alloc;

// Módulos internos
mod input;
mod render;
mod scene;
mod server;
mod ui;

use core::panic::PanicInfo;
use redpowder::println;

// ============================================================================
// ALOCADOR
// ============================================================================

/// Alocador global usando syscalls do kernel.
#[global_allocator]
static ALLOCATOR: redpowder::mem::heap::SyscallAllocator = redpowder::mem::heap::SyscallAllocator;

// ============================================================================
// PONTO DE ENTRADA
// ============================================================================

/// Ponto de entrada do compositor.
///
/// Esta função é chamada quando o processo é iniciado pelo kernel.
/// Inicializa o servidor e entra no loop principal de renderização.
#[no_mangle]
#[link_section = ".text._start"]
pub extern "C" fn _start() -> ! {
    // Debug de baixo nível - escrita direta sem formatação para diagnosticar travamento
    // Usa write_str diretamente para evitar overhead de format_args!
    let _ = redpowder::console::write_str("[Firefly] ENTRY\n");

    println!("[Firefly] Compositor iniciando v0.0.1");

    // Inicializar e executar o servidor
    match server::Server::new() {
        Ok(mut server) => {
            println!("[Firefly] Servidor inicializado. Aguardando clientes.");

            if let Err(e) = server.run() {
                println!("[Firefly] FATAL: Servidor travou: {:?}", e);
            }
        }
        Err(e) => {
            println!("[Firefly] FATAL: Falha ao inicializar servidor: {:?}", e);
        }
    }

    println!("[Firefly] Compositor encerrado!");

    // Loop infinito para evitar retorno
    loop {
        let _ = redpowder::time::sleep(1000);
    }
}

// ============================================================================
// PANIC HANDLER
// ============================================================================

/// Handler de panic para ambiente no_std.
///
/// Em caso de panic, o compositor entra em loop infinito.
/// TODO: Implementar log de panic para debug.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("[Firefly] PANIC: {:?}", info);
    loop {}
}
