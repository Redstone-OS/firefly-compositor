//! # Firefly Compositor
//!
//! Compositor gráfico do RedstoneOS.
//! Gerencia janelas e compõe no framebuffer.

#![no_std]
#![no_main]

extern crate alloc;

mod cursor;
mod decoration;

use alloc::vec::Vec;
use core::panic::PanicInfo;
use redpowder::graphics::{Color, Framebuffer};
use redpowder::println;

#[global_allocator]
static ALLOCATOR: redpowder::mem::heap::SyscallAllocator = redpowder::mem::heap::SyscallAllocator;

// ============================================================================
// CONSTANTES
// ============================================================================

/// Cor de fundo do desktop (vista apenas se o Shell morrer) - Laranja
const BG_COLOR: Color = Color::rgb(231, 132, 66);

/// Nome da porta do compositor
const COMPOSITOR_PORT: &str = "firefly.compositor";

// ============================================================================
// ESTRUTURA DO COMPOSITOR
// ============================================================================

struct Compositor {
    fb: Framebuffer,
    windows: Vec<WindowInfo>,
    cursor_x: i32,
    cursor_y: i32,
}

/// Informações de uma janela gerenciada
#[derive(Clone)]
struct WindowInfo {
    id: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    title: &'static str,
    is_active: bool,
}

impl Compositor {
    fn new() -> Result<Self, ()> {
        let fb = Framebuffer::new().map_err(|_| ())?;
        let screen_w = fb.width();
        let screen_h = fb.height();

        // Janela de teste (Terminal Placeholder)
        // Isso simula o terminal que será desenhado na tela
        // No futuro, isso e criado via IPC
        let win_w = (screen_w * 60) / 100;
        let win_h = (screen_h * 50) / 100;
        let win_x = (screen_w - win_w) / 2;
        let win_y = (screen_h - win_h) / 2; // Centralizado

        let test_window = WindowInfo {
            id: 1,
            x: win_x,
            y: win_y,
            width: win_w,
            height: win_h,
            title: "Terminal",
            is_active: true,
        };

        Ok(Self {
            fb,
            windows: alloc::vec![test_window],
            cursor_x: (screen_w / 2) as i32,
            cursor_y: (screen_h / 2) as i32,
        })
    }

    fn draw_screen(&mut self) {
        // 1. Limpa fundo (caso Shell não tenha desenhado wallpaper)
        // Como o Shell é cooperativo, não limpamos para não piscar o wallpaper dele
        // Mas se não houver shell, ficaria lixo.
        // Para garantir, vamos não limpar se assumirmos que o Shell roda.
        // let _ = self.fb.clear(BG_COLOR);

        // 2. Desenhar decorações das janelas (do fundo para o topo)
        for win in &self.windows {
            // OBS: Passamos apenas a referência para o FB e para a janela,
            // evitando borrow mutável de 'self' inteiro enquanto iteramos 'self.windows'.
            Self::draw_window_frame(&mut self.fb, win);
        }

        // 3. TODO: Desenhar conteúdo das janelas (buffer do cliente)

        // 4. Desenhar cursor por último (topo)
        cursor::draw(&mut self.fb, self.cursor_x, self.cursor_y);
    }

    fn draw_window_frame(fb: &mut Framebuffer, win: &WindowInfo) {
        // Desenhar decoração (barra de título, botões, bordas)
        decoration::draw_window_decoration(
            fb,
            win.x,
            win.y,
            win.width,
            win.height,
            win.title,
            win.is_active,
        );

        // Área interna da janela (conteúdo)
        // NÃO DESENHAR CONTEÚDO AQUI se o app cliente desenha.
        // Como o Terminal App desenha no mesmo lugar, remover para evitar flicker.
        // let content_x = win.x + decoration::BORDER_WIDTH;
        // let content_y = win.y + decoration::TITLEBAR_HEIGHT;
        // let content_w = win.width - decoration::BORDER_WIDTH * 2;
        // let content_h = win.height - decoration::TITLEBAR_HEIGHT - decoration::BORDER_WIDTH;
        // let content_bg = Color::rgb(20, 20, 30);
        // let _ = fb.fill_rect(content_x, content_y, content_w, content_h, content_bg);
    }
}

// ============================================================================
// ENTRADA
// ============================================================================

#[no_mangle]
#[link_section = ".text._start"]
pub extern "C" fn _start() -> ! {
    println!("[Firefly] Compositor starting...");

    match Compositor::new() {
        Ok(mut compositor) => {
            println!("[Firefly] Compositor initialized");

            // Desenhar tela inicial (Decorations)
            compositor.draw_screen();

            // Loop principal
            loop {
                // TODO: Processar eventos de input

                // Redesenhar tela
                compositor.draw_screen();

                // Throttle (60 FPS approx)
                for _ in 0..100000 {
                    core::hint::spin_loop();
                }

                // Yield para shell rodar
                redpowder::process::yield_now();
            }
        }
        Err(_) => {
            println!("[Firefly] FATAL: FB Init Failed");
        }
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
