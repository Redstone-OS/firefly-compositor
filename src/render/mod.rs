//! # Software Renderer
//!
//! Implementação de um buffer de vídeo em RAM (Backbuffer).
//! Todas as operações de desenho ocorrem aqui para evitar syscalls excessivas.

use alloc::vec;
use alloc::vec::Vec;
use redpowder::graphics::{write_framebuffer, Color, FramebufferInfo};

pub struct Backbuffer {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u32>, // Buffer ARGB
}

impl Backbuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        // Inicializa com preto transparente
        let buffer = vec![0; size];

        Self {
            width,
            height,
            buffer,
        }
    }

    /// Desenha um pixel no buffer (sem syscall)
    #[inline]
    pub fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        let offset = (y as usize * self.width as usize) + x as usize;
        self.buffer[offset] = color.0;
    }

    /// Preenche um retângulo no buffer (otimizado)
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color) {
        // Clips
        let x1 = x.max(0);
        let y1 = y.max(0);
        let x2 = (x + w as i32).min(self.width as i32);
        let y2 = (y + h as i32).min(self.height as i32);

        if x1 >= x2 || y1 >= y2 {
            return;
        }

        let rect_w = (x2 - x1) as usize;
        let color_u32 = color.0;

        for cur_y in y1..y2 {
            let start_offset = (cur_y as usize * self.width as usize) + x1 as usize;
            let end_offset = start_offset + rect_w;

            // Preenchimento rápido de linha
            self.buffer[start_offset..end_offset].fill(color_u32);
        }
    }

    /// Limpa todo o buffer com uma cor
    pub fn clear(&mut self, color: Color) {
        self.buffer.fill(color.0);
    }

    /// Envia o buffer para o Kernel (Syscall pesada)
    /// Retorna verdadeiro se sucesso
    pub fn present(&self) -> bool {
        // Converte slice de u32 para u8 para a syscall
        // Safety: Vec<u32> tem alinhamento compatível e layout denso.
        let byte_slice = unsafe {
            core::slice::from_raw_parts(self.buffer.as_ptr() as *const u8, self.buffer.len() * 4)
        };

        // Escreve tudo de uma vez
        write_framebuffer(0, byte_slice).is_ok()
    }
}
