//! # Firefly Compositor
//!
//! Compositor gráfico do RedstoneOS.
//! Gerencia janelas e compõe no framebuffer.

#![no_std]
#![no_main]

extern crate alloc;

mod render;
mod ui;

use crate::render::Backbuffer;
use crate::ui::{cursor, decoration};
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
    #[allow(dead_code)]
    hw_fb: Framebuffer, // Usado para obter info, mas desenho é feito no backbuffer
    backbuffer: Backbuffer,
    windows: Vec<WindowInfo>,
    cursor_x: i32,
    cursor_y: i32,
    dirty: bool,
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
        let hw_fb = Framebuffer::new().map_err(|_| ())?;
        let screen_w = hw_fb.width();
        let screen_h = hw_fb.height();

        // Criar Backbuffer em RAM
        let backbuffer = Backbuffer::new(screen_w, screen_h);

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
            hw_fb,
            backbuffer,
            windows: alloc::vec![test_window],
            cursor_x: (screen_w / 2) as i32,
            cursor_y: (screen_h / 2) as i32,
            dirty: true, // Forçar primeiro redesenho
        })
    }

    fn draw_screen(&mut self) {
        if !self.dirty {
            return;
        }

        // 1. Limpa fundo (no backbuffer)
        // Como o Shell é cooperativo, não limpamos para não piscar o wallpaper dele
        // Mas se não houver shell, ficaria lixo.
        // Para simplificar e garantir limpeza no backbuffer:
        // self.backbuffer.clear(BG_COLOR); // Descomentar se quiser fundo sólido

        // 2. Desenhar decorações das janelas (do fundo para o topo)
        for win in &self.windows {
            Self::draw_window_frame(&mut self.backbuffer, win);
        }

        // 3. TODO: Desenhar conteúdo das janelas (buffer do cliente)

        // 4. Desenhar cursor por último (topo)
        cursor::draw(&mut self.backbuffer, self.cursor_x, self.cursor_y);

        // 5. Enviar para tela (Flush)
        self.backbuffer.present();

        // Limpar dirty flag
        self.dirty = false;
    }

    fn draw_window_frame(fb: &mut Backbuffer, win: &WindowInfo) {
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
            println!("[Firefly] Compositor initialized (Double Buffered)");

            // Loop principal
            loop {
                // TODO: Processar eventos de input
                // Se houver input (eg mouse move), atualizar cursor_x/y e setar dirty = true
                // Por enqunato, forçamos dirty a cada N frames para teste de animação,
                // ou apenas deixamos estático.

                // Simulação de movimento (apenas para testar dirty check)
                // compositor.cursor_x = (compositor.cursor_x + 1) % 800;
                // compositor.dirty = true;

                // Redesenhar tela (se dirty)
                compositor.draw_screen();

                // Throttle (60 FPS)
                // Usar sleep para dormir e economizar CPU
                let _ = redpowder::time::sleep(16);
            }
        }
        Err(_) => {
            println!("[Firefly] FATAL: FB Init Failed");
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
